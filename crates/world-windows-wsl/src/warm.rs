use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[cfg(test)]
use std::sync::Arc;

pub struct WarmCmd {
    pub(crate) distro: String,
    pub(crate) project_path: PathBuf,
    pub(crate) enabled: bool,
    #[cfg(test)]
    pub(crate) invocations: Arc<std::sync::atomic::AtomicUsize>,
}

impl WarmCmd {
    pub fn enabled(distro: String, project_path: PathBuf) -> Self {
        Self {
            distro,
            project_path,
            enabled: true,
            #[cfg(test)]
            invocations: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    #[cfg(test)]
    pub fn disabled(
        distro: String,
        project_path: PathBuf,
    ) -> (Self, Arc<std::sync::atomic::AtomicUsize>) {
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        (
            Self {
                distro,
                project_path,
                enabled: false,
                invocations: counter.clone(),
            },
            counter,
        )
    }

    pub fn run(&self) -> Result<()> {
        #[cfg(test)]
        if !self.enabled {
            self.invocations
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Ok(());
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

        let output = Command::new("pwsh")
            .arg("-NoLogo")
            .arg("-File")
            .arg(script)
            .arg("-DistroName")
            .arg(&self.distro)
            .arg("-ProjectPath")
            .arg(&self.project_path)
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
