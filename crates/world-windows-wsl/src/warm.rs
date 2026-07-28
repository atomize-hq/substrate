use anyhow::{anyhow, Context, Result};
#[cfg(test)]
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[cfg(test)]
use std::sync::{Arc, Mutex};

#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub(crate) struct WarmInvocation {
    pub(crate) args: Vec<String>,
    pub(crate) removed_env: Vec<String>,
    pub(crate) set_env: BTreeMap<String, String>,
}

pub struct WarmCmd {
    pub(crate) distro: String,
    pub(crate) project_path: PathBuf,
    pub(crate) pipe_path: String,
    pub(crate) install_prefix: String,
    pub(crate) install_bootstrap_context_v1: String,
    pub(crate) platform_bootstrap_mapping_v1: String,
    pub(crate) enabled: bool,
    #[cfg(test)]
    pub(crate) invocations: Arc<std::sync::atomic::AtomicUsize>,
    #[cfg(test)]
    pub(crate) last_invocation: Arc<Mutex<Option<WarmInvocation>>>,
}

impl WarmCmd {
    pub fn enabled(
        distro: String,
        project_path: PathBuf,
        pipe_path: String,
        install_prefix: String,
        install_bootstrap_context_v1: String,
        platform_bootstrap_mapping_v1: String,
    ) -> Self {
        Self {
            distro,
            project_path,
            pipe_path,
            install_prefix,
            install_bootstrap_context_v1,
            platform_bootstrap_mapping_v1,
            enabled: true,
            #[cfg(test)]
            invocations: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            #[cfg(test)]
            last_invocation: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(test)]
    pub fn disabled(
        distro: String,
        project_path: PathBuf,
        pipe_path: String,
        install_prefix: String,
        install_bootstrap_context_v1: String,
        platform_bootstrap_mapping_v1: String,
    ) -> (
        Self,
        Arc<std::sync::atomic::AtomicUsize>,
        Arc<Mutex<Option<WarmInvocation>>>,
    ) {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let last_invocation = Arc::new(Mutex::new(None));
        (
            Self {
                distro,
                project_path,
                pipe_path,
                install_prefix,
                install_bootstrap_context_v1,
                platform_bootstrap_mapping_v1,
                enabled: false,
                invocations: counter.clone(),
                last_invocation: last_invocation.clone(),
            },
            counter,
            last_invocation,
        )
    }

    pub fn run(&self) -> Result<()> {
        #[cfg(test)]
        let args = vec![
            "-NoProfile".to_string(),
            "-NoLogo".to_string(),
            "-File".to_string(),
            self.project_path
                .join("scripts/windows/wsl-warm.ps1")
                .to_string_lossy()
                .to_string(),
            "-DistroName".to_string(),
            self.distro.clone(),
            "-ProjectPath".to_string(),
            self.project_path.to_string_lossy().to_string(),
            "-PipePath".to_string(),
            self.pipe_path.clone(),
            "-InstallPrefix".to_string(),
            self.install_prefix.clone(),
            "-InstallBootstrapContextV1".to_string(),
            self.install_bootstrap_context_v1.clone(),
            "-PlatformBootstrapMappingV1".to_string(),
            self.platform_bootstrap_mapping_v1.clone(),
        ];
        let removed_env = vec![
            "HOME".to_string(),
            "LOCALAPPDATA".to_string(),
            "SUBSTRATE_FORWARDER_PIPE".to_string(),
            "SUBSTRATE_FORWARDER_TCP".to_string(),
            "SUBSTRATE_FORWARDER_TCP_ADDR".to_string(),
            "SUBSTRATE_FORWARDER_TCP_HOST".to_string(),
            "SUBSTRATE_FORWARDER_TCP_PORT".to_string(),
            "SUBSTRATE_PROJECT_PATH".to_string(),
            "SUBSTRATE_WSL_DISTRO".to_string(),
            "USERPROFILE".to_string(),
            "WSLENV".to_string(),
        ];

        #[cfg(test)]
        {
            *self.last_invocation.lock().expect("warm invocation mutex") = Some(WarmInvocation {
                args: args.clone(),
                removed_env: removed_env.clone(),
                set_env: BTreeMap::new(),
            });
            if !self.enabled {
                self.invocations
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                return Ok(());
            }
        }

        if !self.enabled {
            return Ok(());
        }

        let script_path = self.project_path.join("scripts/windows/wsl-warm.ps1");
        let script = script_path
            .to_str()
            .ok_or_else(|| anyhow!("warm script path contains invalid UTF-8"))?;

        info!(
            target: "world_windows_wsl::backend",
            distro = %self.distro,
            "running wsl warm script"
        );

        let mut command = Command::new("pwsh");
        command
            .arg("-NoProfile")
            .arg("-NoLogo")
            .arg("-File")
            .arg(script)
            .arg("-DistroName")
            .arg(&self.distro)
            .arg("-ProjectPath")
            .arg(&self.project_path)
            .arg("-PipePath")
            .arg(&self.pipe_path)
            .arg("-InstallPrefix")
            .arg(&self.install_prefix)
            .arg("-InstallBootstrapContextV1")
            .arg(&self.install_bootstrap_context_v1)
            .arg("-PlatformBootstrapMappingV1")
            .arg(&self.platform_bootstrap_mapping_v1);

        for key in &removed_env {
            command.env_remove(key);
        }

        #[cfg(test)]
        let mut set_env = BTreeMap::new();

        if std::env::var_os("SUBSTRATE_WINDOWS_CARGO_EXE").is_none() {
            if let Some(cargo_exe) = resolve_windows_cargo_exe_from_host_env() {
                #[cfg(test)]
                set_env.insert(
                    "SUBSTRATE_WINDOWS_CARGO_EXE".to_string(),
                    cargo_exe.to_string_lossy().to_string(),
                );
                command.env("SUBSTRATE_WINDOWS_CARGO_EXE", cargo_exe);
            }
        }

        #[cfg(test)]
        {
            *self.last_invocation.lock().expect("warm invocation mutex") = Some(WarmInvocation {
                args,
                removed_env,
                set_env,
            });
        }

        let output = command
            .output()
            .context("failed to spawn pwsh for warm script")?;

        if output.status.success() {
            Ok(())
        } else {
            let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            const MAX: usize = 8 * 1024;
            if stdout.len() > MAX {
                stdout.truncate(MAX);
                stdout.push_str("\n... (truncated)");
            }
            if stderr.len() > MAX {
                stderr.truncate(MAX);
                stderr.push_str("\n... (truncated)");
            }
            Err(anyhow!(
                "wsl warm script exited with status {}\nstdout:\n{}\nstderr:\n{}",
                output.status.code().unwrap_or(-1),
                stdout.trim(),
                stderr.trim()
            ))
        }
    }
}

fn resolve_windows_cargo_exe_from_host_env() -> Option<PathBuf> {
    fn existing_path(candidate: Option<std::ffi::OsString>) -> Option<PathBuf> {
        let path = candidate.map(PathBuf::from)?;
        path.is_file().then_some(path)
    }

    existing_path(std::env::var_os("CARGO").filter(|value| {
        value
            .to_string_lossy()
            .to_ascii_lowercase()
            .ends_with("cargo.exe")
    }))
    .or_else(|| {
        existing_path(std::env::var_os("CARGO_HOME").map(|home| {
            PathBuf::from(home)
                .join("bin")
                .join("cargo.exe")
                .into_os_string()
        }))
    })
    .or_else(|| {
        existing_path(std::env::var_os("SUBSTRATE_HOST_USERPROFILE").map(|home| {
            PathBuf::from(home)
                .join(".cargo")
                .join("bin")
                .join("cargo.exe")
                .into_os_string()
        }))
    })
    .or_else(|| {
        existing_path(std::env::var_os("USERPROFILE").map(|home| {
            PathBuf::from(home)
                .join(".cargo")
                .join("bin")
                .join("cargo.exe")
                .into_os_string()
        }))
    })
}
