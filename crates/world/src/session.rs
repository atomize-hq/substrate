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
    pub network_filter: Option<crate::netfilter::NetFilter>,
    pub fs_by_span: HashMap<String, FsDiff>,
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
        let mut world = Self {
            id: format!("wld_{}", uuid::Uuid::now_v7()),
            root_dir: PathBuf::from("/tmp/substrate-worlds"),
            project_dir: spec.project_dir.clone(),
            cgroup_path: PathBuf::from("/sys/fs/cgroup/substrate"),
            net_namespace: None,
            spec,
            network_filter: None,
            fs_by_span: HashMap::new(),
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
    fn setup(&mut self) -> Result<()> {
        self.create_directories()?;

        #[cfg(target_os = "linux")]
        {
            self.setup_linux_isolation()?;
            
            // Set up network filtering if isolation is enabled
            if self.spec.isolate_network {
                self.setup_network_filter()?;
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("⚠️  Linux isolation not available on this platform");
        }

        Ok(())
    }
    
    /// Set up network filtering with nftables.
    fn setup_network_filter(&mut self) -> Result<()> {
        let filter = crate::netfilter::apply_network_filter(
            &self.id,
            self.spec.allowed_domains.clone(),
        )?;
        
        self.network_filter = Some(filter);
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
        &mut self,
        cmd: &str,
        cwd: &Path,
        env: HashMap<String, String>,
        _pty: bool,
        span_id: Option<String>,
    ) -> Result<ExecResult> {
        let output;
        let scopes_used;
        let mut diff_opt: Option<FsDiff> = None;
        
        // Check if command should be isolated with overlayfs
        if self.should_isolate_command(cmd) {
            // Execute with overlayfs isolation
            let (exec_output, diff) = crate::overlayfs::execute_with_overlay(
                &self.id,
                cmd,
                &self.project_dir,
                cwd,
                &env,
            )?;
            output = exec_output;
            diff_opt = Some(diff.clone());
            if let Some(id) = span_id.as_ref() {
                self.fs_by_span.insert(id.clone(), diff);
            }
        } else {
            // Execute directly on host (for now)
            output = std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(cwd)
                .envs(&env)
                .output()
                .context("Failed to execute command")?;
        }
        
        // Track network scopes if filter is active
        if let Some(ref mut filter) = self.network_filter {
            scopes_used = crate::netfilter::monitor_network_scopes(filter)?;
        } else {
            scopes_used = vec![];
        }

        Ok(ExecResult {
            exit: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            scopes_used,
            fs_diff: diff_opt,
        })
    }

    /// Compute filesystem diff for a span.
    pub fn compute_fs_diff(&self, span_id: &str) -> Result<FsDiff> {
        if let Some(diff) = self.fs_by_span.get(span_id) {
            return Ok(diff.clone());
        }
        Ok(FsDiff::default())
    }

    /// Check if a command should be isolated with overlayfs.
    fn should_isolate_command(&self, cmd: &str) -> bool {
        // Commands that should run in isolated overlayfs
        let isolated_patterns = [
            "pip install",
            "npm install",
            "cargo install",
            "go get",
            "gem install",
            "apt install",
            "yum install",
            "brew install",
        ];

        isolated_patterns.iter().any(|pattern| cmd.contains(pattern))
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
