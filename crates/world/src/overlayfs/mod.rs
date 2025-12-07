//! Overlayfs-based filesystem diff tracking for Linux.
//!
//! This module provides overlayfs-based filesystem isolation and diff tracking
//! for commands that need to be executed in an isolated environment.

use crate::guard::{should_guard_anchor, wrap_with_anchor_guard};
use std::path::{Path, PathBuf};
use std::process::Child;

use anyhow::{Context, Result};
use substrate_common::FsDiff;

mod layering;
mod utils;

const MAX_TRACKED_FILES: usize = 1000;
const MAX_TRACKED_DIRS: usize = 100;
const MAX_DIFF_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10MB

/// Overlayfs manager for filesystem isolation and diff tracking.
pub struct OverlayFs {
    world_id: String,
    overlay_dir: PathBuf,
    upper_dir: PathBuf,
    work_dir: PathBuf,
    merged_dir: PathBuf,
    lower_dir: Option<PathBuf>,
    #[allow(dead_code)]
    bind_lower_dir: Option<PathBuf>,
    is_mounted: bool,
    using_fuse: bool,
    #[allow(dead_code)]
    fuse_child: Option<Child>,
}

impl OverlayFs {
    /// Create a new overlayfs instance for the given world.
    pub fn new(world_id: &str) -> Result<Self> {
        let base_dir = layering::choose_base_dir()?;
        std::fs::create_dir_all(&base_dir)?;

        let overlay_dir = base_dir.join(world_id);
        let upper_dir = overlay_dir.join("upper");
        let work_dir = overlay_dir.join("work");
        let merged_dir = overlay_dir.join("merged");

        Ok(Self {
            world_id: world_id.to_string(),
            overlay_dir,
            upper_dir,
            work_dir,
            merged_dir,
            lower_dir: None,
            bind_lower_dir: None,
            is_mounted: false,
            using_fuse: false,
            fuse_child: None,
        })
    }

    /// Mount the overlayfs with the given lower directory.
    pub fn mount(&mut self, #[allow(unused_variables)] lower_dir: &Path) -> Result<PathBuf> {
        if self.is_mounted {
            return Ok(self.merged_dir.clone());
        }

        layering::prepare_overlay_dirs(&self.upper_dir, &self.work_dir, &self.merged_dir)?;

        #[cfg(target_os = "linux")]
        {
            self.lower_dir = Some(lower_dir.to_path_buf());
            layering::mount_linux(self, lower_dir)?;
            self.is_mounted = true;
            Ok(self.merged_dir.clone())
        }

        #[cfg(not(target_os = "linux"))]
        {
            anyhow::bail!("Overlayfs is only supported on Linux");
        }
    }

    /// Mount the overlayfs in read-only mode (no upper/copy-diff layer).
    pub fn mount_read_only(
        &mut self,
        #[allow(unused_variables)] lower_dir: &Path,
    ) -> Result<PathBuf> {
        if self.is_mounted {
            return Ok(self.merged_dir.clone());
        }

        #[cfg(target_os = "linux")]
        {
            std::fs::create_dir_all(&self.overlay_dir)?;
            std::fs::create_dir_all(&self.merged_dir)?;
            self.lower_dir = Some(lower_dir.to_path_buf());
            layering::mount_linux_read_only(self, lower_dir)?;
            self.is_mounted = true;
            Ok(self.merged_dir.clone())
        }

        #[cfg(not(target_os = "linux"))]
        {
            anyhow::bail!("Overlayfs is only supported on Linux");
        }
    }

    /// Unmount the overlayfs.
    pub fn unmount(&mut self) -> Result<()> {
        if !self.is_mounted {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        layering::unmount_linux(self)?;

        self.is_mounted = false;
        Ok(())
    }

    /// Compute the filesystem diff from the upper layer.
    pub fn compute_diff(&self) -> Result<FsDiff> {
        if !self.upper_dir.exists() {
            return Ok(FsDiff::default());
        }

        utils::compute_diff(
            &self.upper_dir,
            self.lower_dir.as_deref(),
            MAX_TRACKED_FILES,
            MAX_TRACKED_DIRS,
            MAX_DIFF_SIZE_BYTES,
        )
    }

    /// Clean up the overlay directories.
    pub fn cleanup(&mut self) -> Result<()> {
        self.unmount()?;

        if self.overlay_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.overlay_dir) {
                if std::env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1" {
                    eprintln!(
                        "[replay] warn: overlay cleanup left in place (world={} path={}): {}",
                        self.world_id,
                        self.overlay_dir.display(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Return true if the current mount is using fuse-overlayfs.
    pub fn is_using_fuse(&self) -> bool {
        self.using_fuse
    }

    /// Get merged directory path.
    pub fn merged_dir_path(&self) -> &Path {
        &self.merged_dir
    }

    /// Get upper directory path.
    pub fn upper_dir_path(&self) -> &Path {
        &self.upper_dir
    }

    /// Mount only via fuse-overlayfs (no kernel overlay attempt).
    #[cfg(target_os = "linux")]
    pub fn mount_fuse_only(&mut self, lower_dir: &Path) -> Result<PathBuf> {
        layering::prepare_overlay_dirs(&self.upper_dir, &self.work_dir, &self.merged_dir)?;
        layering::mount_fuse_only(self, lower_dir)?;
        self.lower_dir = Some(lower_dir.to_path_buf());
        self.is_mounted = true;
        Ok(self.merged_dir.clone())
    }
}

impl Drop for OverlayFs {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

/// Execute a command with overlayfs isolation and return the filesystem diff.
pub fn execute_with_overlay(
    world_id: &str,
    cmd: &str,
    project_dir: &Path,
    cwd: &Path,
    env: &std::collections::HashMap<String, String>,
) -> Result<(std::process::Output, FsDiff)> {
    let mut overlay = OverlayFs::new(world_id)?;

    let merged_dir = overlay.mount(project_dir)?;

    let mut rel = if cwd.starts_with(project_dir) {
        cwd.strip_prefix(project_dir)
            .unwrap_or_else(|_| Path::new("."))
            .to_path_buf()
    } else {
        PathBuf::from(".")
    };
    if rel.as_os_str().is_empty() {
        rel = PathBuf::from(".");
    }
    let target_dir = merged_dir.join(&rel);
    let mut command_to_run = cmd.to_string();
    if should_guard_anchor(env) {
        command_to_run = wrap_with_anchor_guard(cmd, &merged_dir);
    }
    let output = crate::exec::execute_shell_command(&command_to_run, &target_dir, env, true)
        .context("Failed to execute command in overlay")?;

    let diff = overlay.compute_diff()?;

    overlay.cleanup()?;

    Ok((output, diff))
}

/// Execute a command against a read-only overlay mount so writes fail.
pub fn execute_read_only(
    world_id: &str,
    cmd: &str,
    project_dir: &Path,
    cwd: &Path,
    env: &std::collections::HashMap<String, String>,
) -> Result<(std::process::Output, FsDiff)> {
    let mut overlay = OverlayFs::new(world_id)?;
    let merged_dir = overlay.mount_read_only(project_dir)?;

    let mut rel = if cwd.starts_with(project_dir) {
        cwd.strip_prefix(project_dir)
            .unwrap_or_else(|_| Path::new("."))
            .to_path_buf()
    } else {
        PathBuf::from(".")
    };
    if rel.as_os_str().is_empty() {
        rel = PathBuf::from(".");
    }
    let target_dir = merged_dir.join(&rel);
    let mut command_to_run = cmd.to_string();
    if should_guard_anchor(env) {
        command_to_run = wrap_with_anchor_guard(cmd, &merged_dir);
    }
    let output = crate::exec::execute_shell_command(&command_to_run, &target_dir, env, true)
        .context("Failed to execute command in read-only overlay")?;

    overlay.cleanup()?;

    Ok((output, FsDiff::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_overlayfs_creation() {
        let overlay = OverlayFs::new("test_world").unwrap();
        assert_eq!(overlay.world_id, "test_world");
        assert!(!overlay.is_mounted);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_overlay_mount_unmount() {
        if !nix::unistd::Uid::current().is_root() {
            println!("Skipping overlay mount test (requires root)");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let lower_dir = temp_dir.path();

        let mut overlay = OverlayFs::new("test_mount").unwrap();

        match overlay.mount(lower_dir) {
            Ok(merged) => {
                assert!(merged.exists());
                assert!(overlay.is_mounted);

                overlay.unmount().unwrap();
                assert!(!overlay.is_mounted);

                overlay.cleanup().unwrap();
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("EINVAL")
                    || error_str.contains("Invalid argument")
                    || error_str.contains("Failed to mount overlayfs")
                {
                    println!(
                        "Skipping overlay mount test (overlayfs not supported in this environment): {}",
                        e
                    );
                } else {
                    panic!("Unexpected error mounting overlayfs: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_diff_computation() {
        let temp_dir = TempDir::new().unwrap();
        let upper_dir = temp_dir.path().join("upper");
        std::fs::create_dir_all(&upper_dir).unwrap();

        std::fs::write(upper_dir.join("new_file.txt"), "content").unwrap();
        std::fs::create_dir_all(upper_dir.join("new_dir")).unwrap();

        let overlay = OverlayFs {
            world_id: "test".to_string(),
            overlay_dir: temp_dir.path().to_path_buf(),
            upper_dir,
            work_dir: temp_dir.path().join("work"),
            merged_dir: temp_dir.path().join("merged"),
            lower_dir: None,
            bind_lower_dir: None,
            is_mounted: false,
            using_fuse: false,
            fuse_child: None,
        };

        let diff = overlay.compute_diff().unwrap();
        assert!(!diff.writes.is_empty());
        assert!(!diff.truncated);
    }

    #[test]
    fn cleanup_removes_overlay_tree_when_unmounted() {
        let temp_dir = TempDir::new().unwrap();
        let overlay_dir = temp_dir.path().join("overlay");
        let upper_dir = overlay_dir.join("upper");
        let work_dir = overlay_dir.join("work");
        let merged_dir = overlay_dir.join("merged");
        std::fs::create_dir_all(&upper_dir).unwrap();
        std::fs::write(upper_dir.join("file.txt"), b"data").unwrap();

        let mut overlay = OverlayFs {
            world_id: "cleanup".to_string(),
            overlay_dir: overlay_dir.clone(),
            upper_dir,
            work_dir,
            merged_dir,
            lower_dir: None,
            bind_lower_dir: None,
            is_mounted: false,
            using_fuse: false,
            fuse_child: None,
        };

        overlay.cleanup().unwrap();
        assert!(
            !overlay_dir.exists(),
            "cleanup should remove overlay directory even when not mounted"
        );
    }
}
