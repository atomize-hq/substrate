use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use substrate_common::paths;
use tempfile::Builder;

use crate::lock::ProcessLock;

#[derive(Debug, PartialEq)]
pub enum DeploymentStatus {
    Current,        // Shims are up to date
    Deployed,       // New shims were deployed
    Failed(String), // Deployment failed (but non-fatal)
    Skipped,        // User opted to skip
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionInfo {
    version: String,
    deployed_at: SystemTime,
    commands: Vec<String>,
}

pub struct ShimDeployer {
    shims_dir: PathBuf,
    version_file: PathBuf,
    lock_file: PathBuf,
    skip_deployment: bool,
}

impl ShimDeployer {
    pub fn new() -> Result<Self> {
        Self::with_skip(false)
    }

    pub fn with_skip(skip: bool) -> Result<Self> {
        Ok(Self {
            shims_dir: paths::shims_dir()?,
            version_file: paths::version_file()?,
            lock_file: paths::lock_file()?,
            skip_deployment: skip,
        })
    }

    pub fn ensure_deployed(&self) -> Result<DeploymentStatus> {
        // Check if user wants to skip
        if self.skip_deployment || std::env::var("SUBSTRATE_NO_SHIMS").is_ok() {
            return Ok(DeploymentStatus::Skipped);
        }

        // Check if deployment needed
        if !self.is_deployment_needed()? {
            return Ok(DeploymentStatus::Current);
        }

        // Acquire lock with timeout
        let _lock = match ProcessLock::acquire(&self.lock_file, Duration::from_secs(5)) {
            Ok(lock) => lock,
            Err(e) => {
                eprintln!("Warning: Could not acquire lock for shim deployment: {e}");
                return Ok(DeploymentStatus::Failed(e.to_string()));
            }
        };

        // Double-check after acquiring lock
        if !self.is_deployment_needed()? {
            return Ok(DeploymentStatus::Current);
        }

        // Check for migration from old directory
        if let Ok(old_dir) = paths::old_shims_dir() {
            if old_dir.exists() && !self.shims_dir.exists() {
                self.migrate_old_shims(&old_dir)?;
            }
        }

        // Perform deployment (status messages go to stderr to keep stdout clean)
        eprintln!("Setting up command tracing (one-time setup)...");
        match self.deploy_shims() {
            Ok(()) => {
                self.write_version_file()?;
                eprintln!("Command tracing setup complete.");
                Ok(DeploymentStatus::Deployed)
            }
            Err(e) => {
                eprintln!("Warning: Failed to deploy shims: {e}");
                Ok(DeploymentStatus::Failed(e.to_string()))
            }
        }
    }

    fn is_deployment_needed(&self) -> Result<bool> {
        // Check if shims directory exists
        if !self.shims_dir.exists() {
            return Ok(true);
        }

        // Check if version file exists
        if !self.version_file.exists() {
            return Ok(true);
        }

        // Read version file
        let contents =
            fs::read_to_string(&self.version_file).context("Failed to read version file")?;
        let version_info: VersionInfo =
            serde_json::from_str(&contents).context("Failed to parse version file")?;

        // Get current shim version from environment
        let current_version = env!("CARGO_PKG_VERSION");

        // Check version match
        if !version_info.version.starts_with(current_version) {
            return Ok(true);
        }

        // Check if all expected commands exist
        let commands = self.get_commands_to_shim();
        for cmd in &commands {
            let shim_path = self.shims_dir.join(cmd);
            if !shim_path.exists() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn deploy_shims(&self) -> Result<()> {
        // Get the substrate-shim binary path
        let shim_binary = self.get_shim_binary_path()?;

        // Create temporary directory for atomic deployment
        let temp_dir = Builder::new()
            .prefix("substrate-shims-")
            .tempdir_in(self.shims_dir.parent().unwrap_or(Path::new("/")))
            .context("Failed to create temporary directory")?;

        // Deploy all shims to temp directory
        let commands = self.get_commands_to_shim();
        for cmd in &commands {
            let temp_shim = temp_dir.path().join(cmd);

            // Create symlink to substrate-shim
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&shim_binary, &temp_shim)
                    .with_context(|| format!("Failed to create symlink for {cmd}"))?;
            }

            #[cfg(not(unix))]
            {
                // On non-Unix, copy the binary
                fs::copy(&shim_binary, &temp_shim)
                    .with_context(|| format!("Failed to copy shim for {}", cmd))?;
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = self.shims_dir.parent() {
            fs::create_dir_all(parent).context("Failed to create parent directory for shims")?;
        }

        // Remove old shims directory if it exists
        if self.shims_dir.exists() {
            fs::remove_dir_all(&self.shims_dir).context("Failed to remove old shims directory")?;
        }

        // Atomically move temp directory to final location
        fs::rename(temp_dir.path(), &self.shims_dir)
            .context("Failed to move shims to final location")?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&shim_binary)?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&shim_binary, perms)
                .context("Failed to set executable permissions")?;
        }

        Ok(())
    }

    fn get_shim_binary_path(&self) -> Result<PathBuf> {
        // First, try to find substrate-shim in the same directory as the current executable
        let current_exe =
            std::env::current_exe().context("Failed to get current executable path")?;
        let exe_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?;

        let shim_name = if cfg!(windows) {
            "substrate-shim.exe"
        } else {
            "substrate-shim"
        };

        let shim_path = exe_dir.join(shim_name);

        if shim_path.exists() {
            Ok(shim_path)
        } else {
            // Fall back to searching in PATH
            Ok(which::which(shim_name).context(
                "Failed to find substrate-shim binary. Please ensure it's built and in PATH.",
            )?)
        }
    }

    fn write_version_file(&self) -> Result<()> {
        let version_info = VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            deployed_at: SystemTime::now(),
            commands: self.get_commands_to_shim(),
        };

        let json = serde_json::to_string_pretty(&version_info)
            .context("Failed to serialize version info")?;

        fs::write(&self.version_file, json).context("Failed to write version file")?;

        Ok(())
    }

    fn migrate_old_shims(&self, old_dir: &Path) -> Result<()> {
        println!("Migrating shims from old location...");

        // Create parent directory if needed
        if let Some(parent) = self.shims_dir.parent() {
            fs::create_dir_all(parent).context("Failed to create parent directory")?;
        }

        // Move old directory to new location
        fs::rename(old_dir, &self.shims_dir).context("Failed to migrate old shims directory")?;

        println!("Migration complete.");
        Ok(())
    }

    fn get_commands_to_shim(&self) -> Vec<String> {
        vec![
            "git", "npm", "npx", "node", "pnpm", "bun", "python", "python3", "pip", "pip3", "jq",
            "curl", "wget", "tar", "unzip", "make", "go", "cargo", "deno", "docker", "kubectl",
            "rg", "fd", "bat", "codex",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_status_enum() {
        let status = DeploymentStatus::Current;
        assert_eq!(status, DeploymentStatus::Current);

        let status = DeploymentStatus::Failed("test".to_string());
        matches!(status, DeploymentStatus::Failed(_));
    }

    #[test]
    fn test_get_commands_to_shim() {
        let deployer = ShimDeployer::with_skip(true).unwrap();
        let commands = deployer.get_commands_to_shim();

        assert!(commands.contains(&"git".to_string()));
        assert!(commands.contains(&"npm".to_string()));
        assert!(commands.contains(&"python".to_string()));
        assert!(commands.len() > 20); // Should have a reasonable number of commands
    }

    #[test]
    fn test_skip_deployment() {
        let deployer = ShimDeployer::with_skip(true).unwrap();
        let status = deployer.ensure_deployed().unwrap();
        assert_eq!(status, DeploymentStatus::Skipped);
    }
}
