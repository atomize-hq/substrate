//! Minimal cgroups v2 helper for per-world attach/teardown.
//!
//! Best-effort only: failures should be handled by callers with warnings and
//! never cause replay to fail.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Manage a per-world cgroup v2 subtree at `/sys/fs/cgroup/substrate/<world_id>`.
pub struct CgroupManager {
    world_id: String,
    base_dir: PathBuf,
    path: PathBuf,
    active: bool,
}

impl CgroupManager {
    pub fn new(world_id: &str) -> Self {
        let base_dir = PathBuf::from("/sys/fs/cgroup/substrate");
        let path = base_dir.join(world_id);
        Self {
            world_id: world_id.to_string(),
            base_dir,
            path,
            active: false,
        }
    }

    /// Attempt to enable controllers on the root cgroup and create the world subtree.
    /// Returns Ok(true) when the subtree exists and is usable, Ok(false) on graceful
    /// degradation cases (e.g., insufficient privilege), and Err for unexpected errors.
    pub fn setup(&mut self) -> Result<bool> {
        // Require cgroup v2 mount presence
        let controllers = Path::new("/sys/fs/cgroup/cgroup.controllers");
        if !controllers.exists() {
            return Ok(false);
        }

        // Best-effort: enable common controllers on the root
        let subtree_ctrl = Path::new("/sys/fs/cgroup/cgroup.subtree_control");
        if subtree_ctrl.exists() {
            // Attempt to enable pids/cpu/memory; permission errors are OK
            let _ = std::fs::write(subtree_ctrl, b"+pids +cpu +memory");
        }

        // Create substrate namespace and world cgroup
        if let Err(e) = std::fs::create_dir_all(&self.base_dir) {
            // Surface permission errors so caller can log WARN explicitly
            if e.kind() == std::io::ErrorKind::PermissionDenied || e.kind() == std::io::ErrorKind::ReadOnlyFilesystem {
                return Err(anyhow::anyhow!("create {}: {}", self.base_dir.display(), e));
            }
            return Err(e).context("failed creating cgroup base dir")?;
        }
        if let Err(e) = std::fs::create_dir_all(&self.path) {
            if e.kind() == std::io::ErrorKind::PermissionDenied || e.kind() == std::io::ErrorKind::ReadOnlyFilesystem {
                return Err(anyhow::anyhow!("create {}: {}", self.path.display(), e));
            }
            return Err(e).context("failed creating per-world cgroup dir")?;
        }

        self.active = true;
        Ok(true)
    }

    /// Write a PID to the world cgroup.procs. Returns Ok(true) when attached,
    /// Ok(false) when permission denied or inactive.
    pub fn attach_pid(&self, pid: u32) -> Result<bool> {
        if !self.active { return Ok(false); }
        let procs = self.path.join("cgroup.procs");
        match std::fs::write(&procs, pid.to_string()) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    return Ok(false);
                }
                Err(e).context("failed to attach pid to cgroup")
            }
        }
    }

    /// Convenience to attach the current process.
    pub fn attach_current(&self) -> Result<bool> {
        self.attach_pid(std::process::id())
    }

    /// Remove the world cgroup directory. Returns Ok(()) on best-effort cleanup.
    pub fn teardown(&mut self) -> Result<()> {
        if !self.path.exists() { return Ok(()); }
        match std::fs::remove_dir(&self.path) {
            Ok(_) => { self.active = false; Ok(()) }
            Err(e) => {
                // Busy or not empty: leave for GC; also tolerate ENOENT
                if matches!(e.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Other) {
                    // On Some systems busy manifests as Other; ignore here.
                    return Ok(());
                }
                // Try a recursive remove as last resort; ignore errors
                let _ = std::fs::remove_dir_all(&self.path);
                Ok(())
            }
        }
    }

    pub fn path(&self) -> &Path { &self.path }
    pub fn is_active(&self) -> bool { self.active }
    pub fn world_id(&self) -> &str { &self.world_id }
}

impl Drop for CgroupManager {
    fn drop(&mut self) {
        let _ = self.teardown();
    }
}
