//! Lima VM lifecycle management.

use crate::limactl;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::time::{Duration, Instant};

/// Lima VM manager for substrate.
pub struct LimaVM {
    name: String,
    host_account_home: Option<PathBuf>,
    control_root: Option<PathBuf>,
}

impl LimaVM {
    pub fn new(name: String) -> Self {
        Self {
            name,
            host_account_home: None,
            control_root: None,
        }
    }

    fn new_with_command_context(
        name: String,
        host_account_home: PathBuf,
        control_root: PathBuf,
    ) -> Self {
        Self {
            name,
            host_account_home: Some(host_account_home),
            control_root: Some(control_root),
        }
    }

    /// Check if Lima is installed and available.
    pub fn check_lima_available() -> Result<()> {
        limactl::path().context("limactl not found. Install Lima with: brew install lima")?;
        Ok(())
    }

    /// Get the status of this VM.
    pub fn status(&self) -> Result<VmStatus> {
        let output = run_limactl(
            ["list", &self.name, "--json"],
            Duration::from_secs(10),
            self.command_context()?,
        )
        .context("Failed to execute limactl list")?;

        if !output.status.success() {
            anyhow::bail!(
                "limactl list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[derive(Deserialize)]
        struct Instance {
            name: String,
            status: String,
            #[serde(default)]
            #[allow(dead_code)]
            dir: String,
            #[serde(default)]
            #[allow(dead_code)]
            arch: String,
        }

        match decode_matching_instance::<Instance, _>(&output.stdout, &self.name, |instance| {
            &instance.name
        })? {
            Some(instance) => match instance.status.as_str() {
                "Running" => Ok(VmStatus::Running),
                "Stopped" => Ok(VmStatus::Stopped),
                "Starting" => Ok(VmStatus::Starting),
                "Stopping" => Ok(VmStatus::Stopping),
                status => Ok(VmStatus::Unknown(status.to_string())),
            },
            None => Ok(VmStatus::NotFound),
        }
    }

    /// Start the VM if it's not running.
    pub fn ensure_running(&self) -> Result<()> {
        match self.status()? {
            VmStatus::Running => return Ok(()),
            VmStatus::Starting => {
                // Wait for it to finish starting
                self.wait_for_running()?;
                return Ok(());
            }
            VmStatus::NotFound => {
                anyhow::bail!(
                    "VM '{}' not found. Create it first with substrate-lima-init or limactl start",
                    self.name
                );
            }
            _ => {
                // Start the VM
                self.start()?;
            }
        }

        Ok(())
    }

    fn start(&self) -> Result<()> {
        println!("Starting Lima VM '{}'...", self.name);

        let output = run_limactl(
            ["start", &self.name, "--tty=false"],
            Duration::from_secs(300),
            self.command_context()?,
        )
        .context("Failed to execute limactl start")?;

        if !output.status.success() {
            anyhow::bail!("Failed to start Lima VM '{}'", self.name);
        }

        self.wait_for_running()?;
        Ok(())
    }

    fn wait_for_running(&self) -> Result<()> {
        let max_attempts = 60; // 2 minutes
        let mut attempts = 0;

        while attempts < max_attempts {
            match self.status()? {
                VmStatus::Running => return Ok(()),
                VmStatus::NotFound => anyhow::bail!("VM disappeared during startup"),
                _ => {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    attempts += 1;
                }
            }
        }

        anyhow::bail!("VM failed to start within timeout")
    }

    /// Stop the VM.
    pub fn stop(&self) -> Result<()> {
        let output = run_limactl(
            ["stop", &self.name],
            Duration::from_secs(60),
            self.command_context()?,
        )
        .context("Failed to execute limactl stop")?;

        if !output.status.success() {
            anyhow::bail!("Failed to stop Lima VM '{}'", self.name);
        }

        Ok(())
    }

    /// Execute a command inside the VM via SSH.
    pub fn exec(&self, cmd: &str) -> Result<std::process::Output> {
        let output = run_limactl(
            ["shell", &self.name, "sh", "-c", cmd],
            Duration::from_secs(60),
            self.command_context()?,
        )
        .context("Failed to execute command in VM")?;

        Ok(output)
    }

    /// Get VM info.
    pub fn info(&self) -> Result<VmInfo> {
        let output = run_limactl(
            ["list", &self.name, "--json"],
            Duration::from_secs(10),
            self.command_context()?,
        )
        .context("Failed to get VM info")?;

        if !output.status.success() {
            anyhow::bail!(
                "limactl list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[derive(Deserialize)]
        struct Instance {
            name: String,
            status: String,
            dir: String,
            arch: String,
            #[serde(default)]
            cpus: Option<u32>,
            #[serde(default)]
            memory: Option<String>,
        }

        let instance =
            decode_matching_instance::<Instance, _>(&output.stdout, &self.name, |candidate| {
                &candidate.name
            })?
            .ok_or_else(|| anyhow::anyhow!("VM not found"))?;

        Ok(VmInfo {
            name: instance.name,
            status: instance.status,
            dir: instance.dir,
            arch: instance.arch,
            cpus: instance.cpus,
            memory: instance.memory,
        })
    }

    fn command_context(&self) -> Result<Option<(&Path, &Path)>> {
        match (&self.host_account_home, &self.control_root) {
            (Some(home), Some(root)) => Ok(Some((home.as_path(), root.as_path()))),
            (None, None) => Ok(None),
            _ => anyhow::bail!("typed Lima command context is incomplete"),
        }
    }
}

fn run_limactl<const N: usize>(
    args: [&str; N],
    timeout: Duration,
    command_context: Option<(&Path, &Path)>,
) -> Result<std::process::Output> {
    let mut child = build_limactl_command(args, command_context)?
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn limactl {:?}", args))?;

    let start = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("limactl {:?} timed out after {:?}", args, timeout);
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(child.wait_with_output()?)
}

fn build_limactl_command<const N: usize>(
    args: [&str; N],
    command_context: Option<(&Path, &Path)>,
) -> Result<Command> {
    let mut command = match command_context {
        Some((host_account_home, control_root)) => {
            limactl::command_for_control_root(host_account_home, control_root)
        }
        None => limactl::command(),
    }
    .with_context(|| format!("failed to construct limactl command {:?}", args))?;
    command.args(args);
    Ok(command)
}

fn decode_matching_instance<T, F>(
    stdout: &[u8],
    expected_name: &str,
    name_of: F,
) -> Result<Option<T>>
where
    T: DeserializeOwned,
    F: Fn(&T) -> &str,
{
    let instances = if let Ok(instance) = serde_json::from_slice::<T>(stdout) {
        vec![instance]
    } else {
        serde_json::from_slice::<Vec<T>>(stdout).context("Failed to parse limactl output")?
    };

    let mut matches = instances
        .into_iter()
        .filter(|instance| name_of(instance) == expected_name);
    let first = matches.next();
    if first.is_some() && matches.next().is_some() {
        anyhow::bail!("limactl returned ambiguous records for '{expected_name}'");
    }
    Ok(first)
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    NotFound,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct VmInfo {
    pub name: String,
    pub status: String,
    pub dir: String,
    pub arch: String,
    pub cpus: Option<u32>,
    pub memory: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    fn command_env_value(command: &Command, key: &str) -> Option<String> {
        command
            .get_envs()
            .find_map(|(name, value)| (name == std::ffi::OsStr::new(key)).then_some(value))
            .flatten()
            .map(|value| value.to_string_lossy().into_owned())
    }

    fn write_executable(path: &Path) {
        fs::write(path, "#!/bin/sh\nexit 0\n").expect("write executable");
        let mut permissions = fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("set permissions");
    }

    #[test]
    fn test_vm_creation() {
        let vm = LimaVM::new("test".to_string());
        assert_eq!(vm.name, "test");
        assert!(vm.host_account_home.is_none());
        assert!(vm.control_root.is_none());
    }

    #[test]
    fn test_lima_availability() {
        match LimaVM::check_lima_available() {
            Ok(()) => println!("Lima is available"),
            Err(e) => println!("Lima not available: {}", e),
        }
        // Don't assert since Lima may not be installed
    }

    #[test]
    fn test_vm_status() {
        let vm = LimaVM::new("nonexistent".to_string());

        // This should work even if Lima is not installed
        match vm.status() {
            Ok(VmStatus::NotFound) => println!("VM correctly reported as not found"),
            Ok(status) => println!("VM status: {:?}", status),
            Err(e) => println!("Expected error when Lima not available: {}", e),
        }
    }

    #[test]
    fn test_typed_command_context_requires_home_and_control_root_together() {
        let vm = LimaVM {
            name: "substrate".to_string(),
            host_account_home: Some(PathBuf::from("/Users/alice")),
            control_root: None,
        };
        assert!(vm.command_context().is_err());
    }

    #[test]
    fn test_decode_matching_instance_rejects_ambiguous_records() {
        #[derive(Debug, Deserialize)]
        struct Instance {
            name: String,
        }

        let err = decode_matching_instance::<Instance, _>(
            br#"[{"name":"substrate"},{"name":"substrate"}]"#,
            "substrate",
            |instance| &instance.name,
        )
        .unwrap_err();
        assert!(err.to_string().contains("ambiguous"));
    }

    #[test]
    fn test_decode_matching_instance_accepts_single_object() {
        #[derive(Deserialize)]
        struct Instance {
            name: String,
        }

        let instance = decode_matching_instance::<Instance, _>(
            br#"{"name":"substrate"}"#,
            "substrate",
            |candidate| &candidate.name,
        )
        .unwrap()
        .expect("matching instance");
        assert_eq!(instance.name, "substrate");
    }

    #[test]
    fn test_typed_command_builder_preserves_exact_vm_args_and_env() {
        let _env_guard = crate::test_util::lock_env();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let limactl_path = tempdir.path().join("limactl");
        write_executable(&limactl_path);
        let prev_limactl = std::env::var_os("SUBSTRATE_TEST_LIMACTL_PATH");
        std::env::set_var("SUBSTRATE_TEST_LIMACTL_PATH", &limactl_path);

        let vm = LimaVM::new_with_command_context(
            "substrate".to_string(),
            PathBuf::from("/Users/alice"),
            PathBuf::from("/Users/alice/.lima"),
        );

        let command = build_limactl_command(
            ["list", "substrate", "--json"],
            vm.command_context().expect("command context"),
        )
        .expect("typed command");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(command.get_program(), limactl_path.as_os_str());
        assert_eq!(args, vec!["list", "substrate", "--json"]);
        assert_eq!(
            command_env_value(&command, "HOME").as_deref(),
            Some("/Users/alice")
        );
        assert_eq!(
            command_env_value(&command, "LIMA_HOME").as_deref(),
            Some("/Users/alice/.lima")
        );

        match prev_limactl {
            Some(value) => std::env::set_var("SUBSTRATE_TEST_LIMACTL_PATH", value),
            None => std::env::remove_var("SUBSTRATE_TEST_LIMACTL_PATH"),
        }
    }
}
