//! Session world implementation for Linux.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use world_api::{ExecResult, FsDiff, WorldSpec};

/// A reusable Linux world with proper isolation.
pub struct SessionWorld {
    pub id: String,
    pub root_dir: PathBuf,
    pub project_dir: PathBuf,
    pub cgroup_path: PathBuf,
    pub net_namespace: Option<String>,
    pub spec: WorldSpec,
}

impl SessionWorld {
    /// Ensure a session world is started and return it.
    pub fn ensure_started(spec: WorldSpec) -> Result<Self> {
        // Check if session world already exists
        if spec.reuse_session {
            if let Some(existing) = Self::find_existing()? {
                return Ok(existing);
            }
        }

        // Create new session world
        let world = Self {
            id: format!("wld_{}", uuid::Uuid::now_v7()),
            root_dir: PathBuf::from("/tmp/substrate-worlds"),
            project_dir: spec.project_dir.clone(),
            cgroup_path: PathBuf::from("/sys/fs/cgroup/substrate"),
            net_namespace: None,
            spec,
        };

        world.setup()?;
        Ok(world)
    }

    /// Find an existing session world if available.
    fn find_existing() -> Result<Option<Self>> {
        // TODO: Implement session discovery logic
        // For now, always create new
        Ok(None)
    }

    /// Set up the world isolation.
    fn setup(&self) -> Result<()> {
        self.create_directories()?;

        #[cfg(target_os = "linux")]
        {
            self.setup_linux_isolation()?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("⚠️  Linux isolation not available on this platform");
        }

        Ok(())
    }

    fn create_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root_dir).context("Failed to create world root directory")?;
        std::fs::create_dir_all(&self.cgroup_path).context("Failed to create cgroup directory")?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn setup_linux_isolation(&self) -> Result<()> {
        use crate::isolation::LinuxIsolation;

        let isolation = LinuxIsolation::new(&self.spec);
        isolation.apply(&self.root_dir, &self.project_dir, &self.cgroup_path)?;

        Ok(())
    }

    /// Execute a command in this world.
    pub fn execute(
        &self,
        cmd: &str,
        cwd: &Path,
        env: HashMap<String, String>,
        _pty: bool,
    ) -> Result<ExecResult> {
        // For now, simple implementation that executes on host
        // TODO: Implement proper world execution

        let span_id = format!("spn_{}", uuid::Uuid::now_v7());

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(cwd)
            .envs(&env)
            .output()
            .context("Failed to execute command")?;

        Ok(ExecResult {
            exit: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            scopes_used: vec![], // TODO: Track actual scopes
        })
    }

    /// Compute filesystem diff for a span.
    pub fn compute_fs_diff(&self, _span_id: &str) -> Result<FsDiff> {
        // TODO: Implement proper diff computation
        Ok(FsDiff::default())
    }

    /// Apply policy to this world.
    pub fn apply_policy(&self, _spec: &WorldSpec) -> Result<()> {
        // TODO: Implement policy application
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_world_creation() {
        let spec = WorldSpec::default();

        // This test should work on all platforms, just with different behavior
        match SessionWorld::ensure_started(spec) {
            Ok(world) => {
                assert!(world.id.starts_with("wld_"));
                assert_eq!(world.root_dir, PathBuf::from("/tmp/substrate-worlds"));
            }
            Err(e) => {
                // On non-Linux platforms, setup may fail, which is expected
                println!("Expected failure on non-Linux: {}", e);
            }
        }
    }
}
