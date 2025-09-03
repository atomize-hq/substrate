//! Lima VM lifecycle management.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

/// Lima VM manager for substrate.
pub struct LimaVM {
    name: String,
}

impl LimaVM {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Check if Lima is installed and available.
    pub fn check_lima_available() -> Result<()> {
        which::which("limactl")
            .context("limactl not found. Install Lima with: brew install lima")?;
        Ok(())
    }

    /// Get the status of this VM.
    pub fn status(&self) -> Result<VmStatus> {
        let output = Command::new("limactl")
            .args(["list", "--json"])
            .output()
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
            dir: String,
            #[serde(default)]
            arch: String,
        }

        let instances: Vec<Instance> =
            serde_json::from_slice(&output.stdout).context("Failed to parse limactl output")?;

        match instances.iter().find(|i| i.name == self.name) {
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

        let status = Command::new("limactl")
            .args(["start", &self.name, "--tty=false"])
            .status()
            .context("Failed to execute limactl start")?;

        if !status.success() {
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
        let status = Command::new("limactl")
            .args(["stop", &self.name])
            .status()
            .context("Failed to execute limactl stop")?;

        if !status.success() {
            anyhow::bail!("Failed to stop Lima VM '{}'", self.name);
        }

        Ok(())
    }

    /// Execute a command inside the VM via SSH.
    pub fn exec(&self, cmd: &str) -> Result<std::process::Output> {
        let output = Command::new("limactl")
            .args(["shell", &self.name, "sh", "-c", cmd])
            .output()
            .context("Failed to execute command in VM")?;

        Ok(output)
    }

    /// Get VM info.
    pub fn info(&self) -> Result<VmInfo> {
        let output = Command::new("limactl")
            .args(["list", &self.name, "--json"])
            .output()
            .context("Failed to get VM info")?;

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

        let instances: Vec<Instance> =
            serde_json::from_slice(&output.stdout).context("Failed to parse VM info")?;

        let instance = instances
            .into_iter()
            .find(|i| i.name == self.name)
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

    #[test]
    fn test_vm_creation() {
        let vm = LimaVM::new("test".to_string());
        assert_eq!(vm.name, "test");
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
}
