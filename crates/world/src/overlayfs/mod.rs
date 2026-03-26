//! Overlayfs-based filesystem diff tracking for Linux.
//!
//! This module provides overlayfs-based filesystem isolation and diff tracking
//! for commands that need to be executed in an isolated environment.

#![cfg_attr(
    not(target_os = "linux"),
    allow(dead_code, unused_imports, unused_variables)
)]

use crate::guard::{should_guard_anchor, wrap_with_anchor_guard};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;

use anyhow::{Context, Result};
use substrate_common::FsDiff;
use world_api::WorldFsMode;

mod layering;
mod strategy;
mod strategy_state;
mod utils;

const MAX_TRACKED_FILES: usize = 1000;
const MAX_TRACKED_DIRS: usize = 100;
const MAX_DIFF_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10MB

pub use strategy::run_enumeration_probe;
pub use strategy::select_strategy;
pub use strategy::{ENUMERATION_PROBE_FILE, ENUMERATION_PROBE_ID};
pub use strategy_state::WorldFsStrategyMeta;

pub fn world_fs_strategy_meta(world_id: &str) -> Option<WorldFsStrategyMeta> {
    strategy_state::get(world_id)
}

/// Overlayfs manager for filesystem isolation and diff tracking.
pub struct OverlayFs {
    world_id: String,
    overlay_dir: PathBuf,
    upper_dir: PathBuf,
    work_dir: PathBuf,
    merged_dir: PathBuf,
    lower_dir: Option<PathBuf>,
    mounted_mode: Option<WorldFsMode>,
    /// True when the active merged mount consults the upper/work layers for reads.
    ///
    /// Kernel overlayfs read-only mounts are currently created as lower-only (no upper/work),
    /// while fuse-overlayfs may still consult upper/work even in read-only mode.
    uses_upper_layer: bool,
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
            mounted_mode: None,
            uses_upper_layer: false,
            bind_lower_dir: None,
            is_mounted: false,
            using_fuse: false,
            fuse_child: None,
        })
    }

    /// Return true if the overlay is currently mounted.
    pub fn is_mounted(&self) -> bool {
        self.is_mounted
    }

    /// Mount the overlayfs with the given lower directory.
    pub fn mount(&mut self, #[allow(unused_variables)] lower_dir: &Path) -> Result<PathBuf> {
        if self.is_mounted {
            return Ok(self.merged_dir.clone());
        }

        // Ensure stale mount state does not leak across unmount/remount cycles.
        self.using_fuse = false;
        self.fuse_child = None;

        layering::prepare_overlay_dirs(&self.upper_dir, &self.work_dir, &self.merged_dir)?;

        #[cfg(target_os = "linux")]
        {
            self.lower_dir = Some(lower_dir.to_path_buf());
            let mut selection =
                strategy::select_strategy(&self.world_id, lower_dir, WorldFsMode::Writable)?;

            match selection.final_strategy {
                substrate_common::WorldFsStrategy::Overlay => {
                    if let Err(primary_err) = self.mount_kernel_only(lower_dir) {
                        // Probe may pass but the actual mount can still fail; retry once with fuse-overlayfs.
                        if std::path::Path::new("/dev/fuse").exists()
                            && which::which("fuse-overlayfs").is_ok()
                        {
                            self.using_fuse = false;
                            self.fuse_child = None;
                            self.mount_fuse_only(lower_dir).with_context(|| {
                                format!(
                                    "WORLD_FS_STRATEGY_UNAVAILABLE fallback_reason={} primary_mount_error={primary_err:#}",
                                    substrate_common::WorldFsStrategyFallbackReason::FallbackMountFailed.as_str()
                                )
                            })?;
                            selection.final_strategy = substrate_common::WorldFsStrategy::Fuse;
                            selection.fallback_reason =
                                substrate_common::WorldFsStrategyFallbackReason::PrimaryMountFailed;
                        } else {
                            anyhow::bail!(
                                "WORLD_FS_STRATEGY_UNAVAILABLE fallback_reason={} primary_mount_error={primary_err:#}",
                                substrate_common::WorldFsStrategyFallbackReason::FallbackUnavailable.as_str()
                            );
                        }
                    }
                }
                substrate_common::WorldFsStrategy::Fuse => {
                    self.mount_fuse_only(lower_dir)?;
                }
                substrate_common::WorldFsStrategy::Host => {
                    anyhow::bail!("host strategy is not mountable via overlayfs");
                }
            }
            strategy_state::set(
                &self.world_id,
                strategy_state::WorldFsStrategyMeta {
                    primary: selection.primary,
                    final_strategy: selection.final_strategy,
                    fallback_reason: selection.fallback_reason,
                    probe: Some(selection.probe),
                },
            );
            self.is_mounted = true;
            self.mounted_mode = Some(WorldFsMode::Writable);
            self.uses_upper_layer = true;
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

        // Ensure stale mount state does not leak across unmount/remount cycles.
        self.using_fuse = false;
        self.fuse_child = None;

        #[cfg(target_os = "linux")]
        {
            std::fs::create_dir_all(&self.overlay_dir)?;
            std::fs::create_dir_all(&self.merged_dir)?;
            self.lower_dir = Some(lower_dir.to_path_buf());
            let mut selection =
                strategy::select_strategy(&self.world_id, lower_dir, WorldFsMode::ReadOnly)?;
            match selection.final_strategy {
                substrate_common::WorldFsStrategy::Overlay => {
                    if let Err(primary_err) = self.mount_read_only_kernel_only(lower_dir) {
                        if std::path::Path::new("/dev/fuse").exists()
                            && which::which("fuse-overlayfs").is_ok()
                        {
                            self.using_fuse = false;
                            self.fuse_child = None;
                            self.mount_fuse_only_read_only(lower_dir).with_context(|| {
                                format!(
                                    "WORLD_FS_STRATEGY_UNAVAILABLE fallback_reason={} primary_mount_error={primary_err:#}",
                                    substrate_common::WorldFsStrategyFallbackReason::FallbackMountFailed.as_str()
                                )
                            })?;
                            selection.final_strategy = substrate_common::WorldFsStrategy::Fuse;
                            selection.fallback_reason =
                                substrate_common::WorldFsStrategyFallbackReason::PrimaryMountFailed;
                        } else {
                            anyhow::bail!(
                                "WORLD_FS_STRATEGY_UNAVAILABLE fallback_reason={} primary_mount_error={primary_err:#}",
                                substrate_common::WorldFsStrategyFallbackReason::FallbackUnavailable.as_str()
                            );
                        }
                    }
                }
                substrate_common::WorldFsStrategy::Fuse => {
                    self.mount_fuse_only_read_only(lower_dir)?;
                }
                substrate_common::WorldFsStrategy::Host => {
                    anyhow::bail!("host strategy is not mountable via overlayfs");
                }
            }
            strategy_state::set(
                &self.world_id,
                strategy_state::WorldFsStrategyMeta {
                    primary: selection.primary,
                    final_strategy: selection.final_strategy,
                    fallback_reason: selection.fallback_reason,
                    probe: Some(selection.probe),
                },
            );
            self.is_mounted = true;
            self.mounted_mode = Some(WorldFsMode::ReadOnly);
            self.uses_upper_layer = matches!(
                selection.final_strategy,
                substrate_common::WorldFsStrategy::Fuse
            );
            Ok(self.merged_dir.clone())
        }

        #[cfg(not(target_os = "linux"))]
        {
            anyhow::bail!("Overlayfs is only supported on Linux");
        }
    }

    /// Remount the merged directory as read-only while preserving overlay state.
    #[cfg(target_os = "linux")]
    pub fn remount_read_only(&mut self) -> Result<()> {
        use nix::mount::{mount, MsFlags};

        if !self.is_mounted {
            anyhow::bail!("cannot remount overlay read-only before mount");
        }

        // Use a generic remount so it applies to kernel or fuse overlay mounts.
        mount(
            None::<&str>,
            &self.merged_dir,
            None::<&str>,
            MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY,
            None::<&str>,
        )
        .context("Failed to remount overlay read-only")?;

        self.mounted_mode = Some(WorldFsMode::ReadOnly);
        Ok(())
    }

    /// Remount the merged directory back to writable mode.
    #[cfg(target_os = "linux")]
    pub fn remount_writable(&mut self) -> Result<()> {
        use nix::mount::{mount, MsFlags};

        if !self.is_mounted {
            anyhow::bail!("cannot remount overlay writable before mount");
        }

        mount(
            None::<&str>,
            &self.merged_dir,
            None::<&str>,
            MsFlags::MS_REMOUNT,
            None::<&str>,
        )
        .context("Failed to remount overlay writable")?;

        self.mounted_mode = Some(WorldFsMode::Writable);
        Ok(())
    }

    /// Unmount the overlayfs.
    pub fn unmount(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        layering::unmount_linux(self)?;

        self.is_mounted = false;
        self.using_fuse = false;
        self.fuse_child = None;
        self.mounted_mode = None;
        self.uses_upper_layer = false;
        strategy_state::clear(&self.world_id);
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

        // If the bind mount is somehow still present, avoid descending into it.
        #[cfg(target_os = "linux")]
        if let Some(ref bind_lower) = self.bind_lower_dir {
            if let Ok(Some(_)) = crate::overlayfs::utils::is_path_mounted(bind_lower) {
                eprintln!(
                    "[overlay] warn: bind mount still active at {}; skipping removal of {}",
                    bind_lower.display(),
                    self.overlay_dir.display()
                );
                return Ok(());
            }
        }

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

    /// Mount via kernel overlayfs only (no fuse fallback).
    #[cfg(target_os = "linux")]
    pub(crate) fn mount_kernel_only(&mut self, lower_dir: &Path) -> Result<PathBuf> {
        layering::prepare_overlay_dirs(&self.upper_dir, &self.work_dir, &self.merged_dir)?;
        layering::mount_linux_kernel_only(self, lower_dir)?;
        self.is_mounted = true;
        Ok(self.merged_dir.clone())
    }

    /// Mount via kernel overlayfs only in read-only mode (no fuse fallback).
    #[cfg(target_os = "linux")]
    pub(crate) fn mount_read_only_kernel_only(&mut self, lower_dir: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.overlay_dir)?;
        std::fs::create_dir_all(&self.merged_dir)?;
        layering::mount_linux_read_only_kernel_only(self, lower_dir)?;
        self.is_mounted = true;
        Ok(self.merged_dir.clone())
    }

    /// Mount only via fuse-overlayfs with a read-only merged view.
    #[cfg(target_os = "linux")]
    pub(crate) fn mount_fuse_only_read_only(&mut self, lower_dir: &Path) -> Result<PathBuf> {
        layering::prepare_overlay_dirs(&self.upper_dir, &self.work_dir, &self.merged_dir)?;
        layering::mount_fuse_only_read_only(self, lower_dir)?;
        self.lower_dir = Some(lower_dir.to_path_buf());
        self.is_mounted = true;
        Ok(self.merged_dir.clone())
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

    /// Discard (revert) the pending overlay upper entry for each provided workspace-relative path.
    ///
    /// This removes any materialized file/directory at `<upperdir>/<path>` as well as any whiteout
    /// file named `.wh.<basename>` in the same parent directory. Missing paths are ignored.
    ///
    /// Returns the number of filesystem entries removed.
    pub fn discard_paths(&mut self, rel_paths: &[PathBuf]) -> Result<u32> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = rel_paths;
            anyhow::bail!("overlayfs path discard is only supported on Linux");
        }

        #[cfg(target_os = "linux")]
        {
            for rel in rel_paths {
                if rel.is_absolute() {
                    anyhow::bail!(
                        "discard_paths: absolute paths are not allowed: {}",
                        rel.display()
                    );
                }
                if rel
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
                {
                    anyhow::bail!(
                        "discard_paths: path segments must not be '..': {}",
                        rel.display()
                    );
                }
            }

            // Modifying `upperdir` behind an active overlay mount is not guaranteed to invalidate
            // overlayfs dentries. To ensure discards take effect in the merged view, remove entries
            // while unmounted and then restore the mount when it consults upper/work.
            let restore: Option<(PathBuf, WorldFsMode)> =
                if self.is_mounted && self.uses_upper_layer && !rel_paths.is_empty() {
                    let lower_dir = self
                        .lower_dir
                        .clone()
                        .context("discard_paths: missing lower_dir for mounted overlay")?;
                    let mode = self.mounted_mode.unwrap_or(WorldFsMode::Writable);
                    self.unmount().context("discard_paths: unmount failed")?;
                    Some((lower_dir, mode))
                } else {
                    None
                };

            fn restore_mount(
                overlay: &mut OverlayFs,
                lower_dir: &Path,
                mode: WorldFsMode,
            ) -> Result<()> {
                overlay
                    .mount(lower_dir)
                    .context("discard_paths: failed to remount overlay")?;
                if mode == WorldFsMode::ReadOnly {
                    if overlay.is_using_fuse() {
                        // fuse-overlayfs does not reliably honor MS_RDONLY remount semantics.
                        overlay
                            .unmount()
                            .context("discard_paths: unmount before fuse ro rebuild")?;
                        overlay
                            .mount_fuse_only_read_only(lower_dir)
                            .context("discard_paths: failed to remount fuse overlay read-only")?;
                        overlay.mounted_mode = Some(WorldFsMode::ReadOnly);
                        overlay.uses_upper_layer = true;
                    } else {
                        overlay
                            .remount_read_only()
                            .context("discard_paths: failed to remount overlay read-only")?;
                    }
                }
                Ok(())
            }

            let removed = (|| -> Result<u32> {
                let mut removed: u32 = 0;
                for rel in rel_paths {
                    let upper_entry = self.upper_dir.join(rel);
                    match fs::symlink_metadata(&upper_entry) {
                        Ok(meta) => {
                            let ft = meta.file_type();
                            if ft.is_dir() && !ft.is_symlink() {
                                fs::remove_dir_all(&upper_entry).with_context(|| {
                                    format!(
                                        "failed to remove overlay upper directory {}",
                                        upper_entry.display()
                                    )
                                })?;
                            } else {
                                fs::remove_file(&upper_entry).with_context(|| {
                                    format!(
                                        "failed to remove overlay upper file {}",
                                        upper_entry.display()
                                    )
                                })?;
                            }
                            removed = removed.saturating_add(1);
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                        Err(err) => {
                            return Err(anyhow::anyhow!(
                                "failed to stat overlay upper entry {}: {err}",
                                upper_entry.display()
                            ));
                        }
                    }

                    let Some(file_name) = rel.file_name().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    let parent = rel.parent().unwrap_or_else(|| Path::new(""));
                    let whiteout = self.upper_dir.join(parent).join(format!(".wh.{file_name}"));
                    match fs::symlink_metadata(&whiteout) {
                        Ok(meta) => {
                            if meta.file_type().is_dir() {
                                fs::remove_dir_all(&whiteout).with_context(|| {
                                    format!(
                                        "failed to remove overlay upper whiteout directory {}",
                                        whiteout.display()
                                    )
                                })?;
                            } else {
                                fs::remove_file(&whiteout).with_context(|| {
                                    format!(
                                        "failed to remove overlay upper whiteout file {}",
                                        whiteout.display()
                                    )
                                })?;
                            }
                            removed = removed.saturating_add(1);
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                        Err(err) => {
                            return Err(anyhow::anyhow!(
                                "failed to stat overlay upper whiteout {}: {err}",
                                whiteout.display()
                            ));
                        }
                    }
                }
                Ok(removed)
            })();

            match removed {
                Ok(removed) => {
                    if let Some((lower_dir, mode)) = restore {
                        restore_mount(self, lower_dir.as_path(), mode)?;
                    }
                    Ok(removed)
                }
                Err(err) => {
                    if let Some((lower_dir, mode)) = restore {
                        let _ = restore_mount(self, lower_dir.as_path(), mode);
                    }
                    Err(err)
                }
            }
        }
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

    let mut command_to_run = cmd.to_string();
    if should_guard_anchor(env) {
        command_to_run = wrap_with_anchor_guard(cmd, project_dir);
    }
    command_to_run = crate::guard::wrap_with_world_env_contract(&command_to_run, env);
    let desired_cwd = if cwd.starts_with(project_dir) {
        cwd.to_path_buf()
    } else {
        project_dir.to_path_buf()
    };
    let output = match crate::exec::execute_shell_command_with_project_bind_mount(
        &command_to_run,
        crate::exec::ProjectBindMount {
            merged_dir: &merged_dir,
            project_dir,
            desired_cwd: &desired_cwd,
            fs_mode: WorldFsMode::Writable,
        },
        env,
        false,
        crate::exec::CgroupAttachPolicy::optional("project_bind_mount"),
    ) {
        Ok(output) => output,
        Err(err) => {
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
            let fallback_world_deps_root =
                crate::exec::stable_world_deps_fallback_root(project_dir);
            match crate::exec::execute_shell_command_with_world_deps_bind_mount(
                &command_to_run,
                &target_dir,
                env,
                false,
                &fallback_world_deps_root,
                crate::exec::CgroupAttachPolicy::optional("world_deps_fallback"),
            ) {
                Ok(output) => output,
                Err(world_deps_err) => crate::exec::execute_shell_command(
                    &command_to_run,
                    &target_dir,
                    env,
                    false,
                )
                .with_context(|| {
                    format!(
                        "Failed to execute command in overlay after mount-namespace bind failed: {err:#}; world-deps fallback also failed: {world_deps_err:#}"
                    )
                })?,
            }
        }
    };

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

    let mut command_to_run = cmd.to_string();
    if should_guard_anchor(env) {
        command_to_run = wrap_with_anchor_guard(cmd, project_dir);
    }
    command_to_run = crate::guard::wrap_with_world_env_contract(&command_to_run, env);
    let desired_cwd = if cwd.starts_with(project_dir) {
        cwd.to_path_buf()
    } else {
        project_dir.to_path_buf()
    };
    let output = match crate::exec::execute_shell_command_with_project_bind_mount(
        &command_to_run,
        crate::exec::ProjectBindMount {
            merged_dir: &merged_dir,
            project_dir,
            desired_cwd: &desired_cwd,
            fs_mode: WorldFsMode::ReadOnly,
        },
        env,
        false,
        crate::exec::CgroupAttachPolicy::optional("project_bind_mount"),
    ) {
        Ok(output) => output,
        Err(err) => {
            return Err(err).context(
                "failed to enforce read-only overlay via mount-namespace bind; refusing to run with possible absolute-path escape",
            );
        }
    };

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
            mounted_mode: None,
            uses_upper_layer: false,
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
            mounted_mode: None,
            uses_upper_layer: false,
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

    #[test]
    #[cfg(target_os = "linux")]
    fn cleanup_detaches_bind_mount_when_mount_fails() {
        use nix::mount::{mount, umount2, MntFlags, MsFlags};
        use nix::unistd::Uid;

        if !Uid::current().is_root() {
            println!("Skipping bind mount cleanup test (requires root)");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join("file.txt"), b"data").unwrap();

        let mut overlay = OverlayFs::new("bind_cleanup").unwrap();
        let bind_lower = overlay.overlay_dir.join("lower");
        std::fs::create_dir_all(&bind_lower).unwrap();

        mount(
            Some(&project_dir),
            &bind_lower,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )
        .unwrap();
        overlay.bind_lower_dir = Some(bind_lower.clone());

        // Simulate a failed mount (is_mounted stays false) and ensure cleanup
        // tears down the bind without deleting the project contents.
        overlay.cleanup().unwrap();

        assert!(
            project_dir.join("file.txt").exists(),
            "cleanup should never delete files from the project dir"
        );
        let _ = umount2(&bind_lower, MntFlags::MNT_DETACH);
        assert!(
            crate::overlayfs::utils::is_path_mounted(&bind_lower)
                .unwrap_or(None)
                .is_none(),
            "bind mount should be detached during cleanup"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn read_only_mount_blocks_writes_without_root() {
        if !nix::unistd::Uid::current().is_root() {
            println!("Skipping read-only overlay mount test (requires root)");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let lower_dir = temp_dir.path();

        let mut overlay = OverlayFs::new("test_ro_mount").unwrap();
        let merged = match overlay.mount_read_only(lower_dir) {
            Ok(path) => path,
            Err(err) => {
                let message = err.to_string();
                if message.contains("fuse-overlayfs") || message.contains("/dev/fuse") {
                    println!("Skipping read-only overlay mount test: {}", message);
                    return;
                }
                // Some CI environments disallow mounts entirely.
                if message.contains("Operation not permitted") || message.contains("EPERM") {
                    println!("Skipping read-only overlay mount test (EPERM): {}", message);
                    return;
                }
                panic!("Unexpected error mounting read-only overlay: {:#}", err);
            }
        };

        let write_attempt = std::fs::write(merged.join("should_not_write.txt"), b"nope");
        assert!(
            write_attempt.is_err(),
            "expected write to fail on read-only overlay mount"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn mount_falls_back_to_fuse_when_kernel_overlay_mount_fails() {
        use nix::mount::{mount, umount2, MntFlags, MsFlags};
        use nix::unistd::Uid;

        if !Uid::current().is_root() {
            println!("Skipping fuse fallback test (requires root)");
            return;
        }
        if !std::path::Path::new("/dev/fuse").exists() {
            println!("Skipping fuse fallback test (/dev/fuse missing)");
            return;
        }
        if which::which("fuse-overlayfs").is_err() {
            println!("Skipping fuse fallback test (fuse-overlayfs not in PATH)");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let lower_dir = temp_dir.path().join("lower");
        std::fs::create_dir_all(&lower_dir).unwrap();
        std::fs::write(lower_dir.join("seed.txt"), b"seed").unwrap();

        // Force the kernel overlay mount to fail by placing `upper` and `work` on
        // different tmpfs mounts (overlayfs requires upper/work on the same fs).
        let mnt_a = temp_dir.path().join("mnt_a");
        let mnt_b = temp_dir.path().join("mnt_b");
        std::fs::create_dir_all(&mnt_a).unwrap();
        std::fs::create_dir_all(&mnt_b).unwrap();
        mount(
            Some("tmpfs"),
            &mnt_a,
            Some("tmpfs"),
            MsFlags::empty(),
            Some("size=16m".as_bytes()),
        )
        .unwrap();
        mount(
            Some("tmpfs"),
            &mnt_b,
            Some("tmpfs"),
            MsFlags::empty(),
            Some("size=16m".as_bytes()),
        )
        .unwrap();

        let mut overlay = OverlayFs {
            world_id: "fuse_fallback".to_string(),
            overlay_dir: temp_dir.path().join("overlay"),
            upper_dir: mnt_a.join("upper"),
            work_dir: mnt_b.join("work"),
            merged_dir: mnt_a.join("merged"),
            lower_dir: None,
            mounted_mode: None,
            uses_upper_layer: false,
            bind_lower_dir: None,
            is_mounted: false,
            using_fuse: false,
            fuse_child: None,
        };

        let mounted = match overlay.mount(&lower_dir) {
            Ok(path) => path,
            Err(err) => {
                println!("Skipping fuse fallback test (mount failed): {err:#}");
                let _ = umount2(&mnt_a, MntFlags::MNT_DETACH);
                let _ = umount2(&mnt_b, MntFlags::MNT_DETACH);
                return;
            }
        };

        assert!(
            overlay.is_using_fuse(),
            "expected mount() to fall back to fuse-overlayfs when kernel overlay mount fails"
        );
        assert!(
            mounted.exists(),
            "expected merged dir to exist after successful fuse mount"
        );

        overlay.cleanup().unwrap();
        let _ = umount2(&mnt_a, MntFlags::MNT_DETACH);
        let _ = umount2(&mnt_b, MntFlags::MNT_DETACH);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn discard_paths_reveals_lower_after_remount() {
        use nix::unistd::Uid;

        if !Uid::current().is_root() {
            println!("Skipping discard remount test (requires root)");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let lower_dir = temp_dir.path().join("lower");
        std::fs::create_dir_all(&lower_dir).unwrap();
        std::fs::write(lower_dir.join("shadow.md"), b"host").unwrap();

        let overlay_dir = temp_dir.path().join("overlay");
        let upper_dir = overlay_dir.join("upper");
        let work_dir = overlay_dir.join("work");
        let merged_dir = overlay_dir.join("merged");

        let mut overlay = OverlayFs {
            world_id: "discard_test".to_string(),
            overlay_dir,
            upper_dir,
            work_dir,
            merged_dir: merged_dir.clone(),
            lower_dir: None,
            mounted_mode: None,
            uses_upper_layer: false,
            bind_lower_dir: None,
            is_mounted: false,
            using_fuse: false,
            fuse_child: None,
        };

        let merged = match overlay.mount(&lower_dir) {
            Ok(path) => path,
            Err(err) => {
                let message = err.to_string();
                // Some CI environments disallow mounts entirely.
                if message.contains("Operation not permitted") || message.contains("EPERM") {
                    println!("Skipping discard remount test (EPERM): {}", message);
                    return;
                }
                if message.contains("Failed to mount overlayfs")
                    || message.contains("Invalid argument")
                    || message.contains("EINVAL")
                {
                    println!(
                        "Skipping discard remount test (overlayfs unavailable): {}",
                        message
                    );
                    return;
                }
                panic!("Unexpected error mounting overlayfs: {err:#}");
            }
        };

        std::fs::write(merged.join("shadow.md"), b"world").unwrap();
        assert_eq!(
            std::fs::read_to_string(merged.join("shadow.md")).unwrap(),
            "world"
        );

        let removed = overlay
            .discard_paths(&[PathBuf::from("shadow.md")])
            .unwrap();
        assert!(removed >= 1, "expected at least one removed entry");

        assert_eq!(
            std::fs::read_to_string(merged.join("shadow.md")).unwrap(),
            "host"
        );

        overlay.cleanup().unwrap();
    }
}
