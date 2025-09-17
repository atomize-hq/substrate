//! Overlayfs-based filesystem diff tracking for Linux.
//!
//! This module provides overlayfs-based filesystem isolation and diff tracking
//! for commands that need to be executed in an isolated environment.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Child;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
use substrate_common::FsDiff;
use walkdir::WalkDir;

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
        let base_dir = choose_base_dir()?;
        std::fs::create_dir_all(&base_dir).context("Failed to create overlay base directory")?;

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

        // Create overlay directories
        std::fs::create_dir_all(&self.upper_dir).context("Failed to create upper directory")?;
        std::fs::create_dir_all(&self.work_dir).context("Failed to create work directory")?;
        std::fs::create_dir_all(&self.merged_dir).context("Failed to create merged directory")?;

        // Mount overlay filesystem
        #[cfg(target_os = "linux")]
        {
            self.lower_dir = Some(lower_dir.to_path_buf());
            self.mount_linux(lower_dir)?;
            self.is_mounted = true;
            Ok(self.merged_dir.clone())
        }

        #[cfg(not(target_os = "linux"))]
        {
            anyhow::bail!("Overlayfs is only supported on Linux");
        }
    }

    #[cfg(target_os = "linux")]
    fn mount_linux(&mut self, lower_dir: &Path) -> Result<()> {
        use nix::mount::{mount, umount2, MntFlags, MsFlags};
        use std::thread::sleep;
        use std::time::Duration;

        // Ensure overlay roots exist
        std::fs::create_dir_all(&self.upper_dir)?;
        std::fs::create_dir_all(&self.work_dir)?;
        std::fs::create_dir_all(&self.merged_dir)?;

        // Bind-mount the requested lower_dir into a stable path under the overlay root
        let bind_lower = self.overlay_dir.join("lower");
        std::fs::create_dir_all(&bind_lower)?;
        // Best-effort unmount if already mounted
        let _ = umount2(&bind_lower, MntFlags::MNT_DETACH);
        mount(
            Some(lower_dir),
            &bind_lower,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )
        .with_context(|| {
            format!(
                "Failed to bind-mount lower {} -> {}",
                lower_dir.display(),
                bind_lower.display()
            )
        })?;
        self.bind_lower_dir = Some(bind_lower.clone());

        let options = format!(
            "lowerdir={},upperdir={},workdir={}",
            bind_lower.display(),
            self.upper_dir.display(),
            self.work_dir.display()
        );

        match mount(
            Some("overlay"),
            &self.merged_dir,
            Some("overlay"),
            MsFlags::empty(),
            Some(options.as_bytes()),
        ) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Try FUSE fallback
                let fuse_bin = which::which("fuse-overlayfs").map_err(|which_err| {
                    anyhow::anyhow!(
                        "Failed to mount overlayfs: {e}. Also missing fuse-overlayfs binary: {which_err}"
                    )
                })?;
                let fuse_opts = format!(
                    "lowerdir={},upperdir={},workdir={}",
                    bind_lower.display(),
                    self.upper_dir.display(),
                    self.work_dir.display()
                );
                let mut child = Command::new(&fuse_bin)
                    .arg("-o")
                    .arg(&fuse_opts)
                    .arg(&self.merged_dir)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .context("Failed to spawn fuse-overlayfs")?;
                // Wait up to ~1s for mount readiness by parsing /proc/self/mounts
                let mut ready = false;
                for _ in 0..30 {
                    if let Ok(Some(fs_type)) = is_path_mounted(&self.merged_dir) {
                        if fs_type.contains("fuse") || fs_type.contains("fuse-overlayfs") {
                            ready = true;
                            break;
                        }
                    }
                    sleep(Duration::from_millis(33));
                }
                if !ready {
                    // Kill child and return error so caller can fallback further
                    let _ = child.kill();
                    Err(anyhow::anyhow!(
                        "fuse-overlayfs did not mount {} within timeout",
                        self.merged_dir.display()
                    ))
                } else {
                    self.using_fuse = true;
                    self.fuse_child = Some(child);
                    Ok(())
                }
            }
        }
    }

    /// Unmount the overlayfs.
    pub fn unmount(&mut self) -> Result<()> {
        if !self.is_mounted {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            use nix::mount::{umount2, MntFlags};
            if self.using_fuse {
                // Try fusermount3 first
                let _status = Command::new("fusermount3")
                    .arg("-u")
                    .arg(&self.merged_dir)
                    .status();
                // Fallback to lazy umount
                let _ = umount2(&self.merged_dir, MntFlags::MNT_DETACH);
                // Terminate fuse child if still running
                if let Some(mut ch) = self.fuse_child.take() {
                    let _ = ch.kill();
                }
            } else {
                umount2(&self.merged_dir, MntFlags::MNT_DETACH)
                    .context("Failed to unmount overlayfs")?;
            }
            // Best-effort unmount of bind-lower mountpoint
            if let Some(ref bind_lower) = self.bind_lower_dir {
                let _ = umount2(bind_lower, MntFlags::MNT_DETACH);
            }
        }

        self.is_mounted = false;
        Ok(())
    }

    /// Compute the filesystem diff from the upper layer.
    pub fn compute_diff(&self) -> Result<FsDiff> {
        if !self.upper_dir.exists() {
            return Ok(FsDiff::default());
        }

        let mut diff = FsDiff::default();
        let mut file_count = 0;
        let mut dir_count = 0;
        let mut total_size = 0;

        // Track original files for modification detection
        let mut original_files = HashSet::new();

        // Walk through the upper directory to find changes
        for entry in WalkDir::new(&self.upper_dir) {
            let entry = entry?;
            let path = entry.path();

            // Skip the upper directory itself
            if path == self.upper_dir {
                continue;
            }

            // Get relative path from upper directory
            let rel_path = path.strip_prefix(&self.upper_dir)?;
            let rel_pathbuf = rel_path.to_path_buf();

            // Check for whiteout files (deleted files in overlayfs)
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if let Some(deleted_name) = name_str.strip_prefix(".wh.") {
                        // This is a whiteout file indicating deletion
                        let deleted_path = rel_path
                            .parent()
                            .map(|p| p.join(deleted_name))
                            .unwrap_or_else(|| PathBuf::from(deleted_name));
                        diff.deletes.push(deleted_path);
                        continue;
                    }
                }
            }

            if entry.file_type().is_file() {
                file_count += 1;

                // Track file size for truncation decisions
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len() as usize;
                }

                // Check if we should truncate
                if file_count > MAX_TRACKED_FILES || total_size > MAX_DIFF_SIZE_BYTES {
                    diff.truncated = true;
                    break;
                }

                // Check if file existed in lower layer (modification) or is new (write)
                if self.is_modification(&rel_pathbuf) {
                    diff.mods.push(rel_pathbuf.clone());
                } else {
                    diff.writes.push(rel_pathbuf.clone());
                }

                original_files.insert(rel_pathbuf);
            } else if entry.file_type().is_dir() {
                dir_count += 1;
                if dir_count <= MAX_TRACKED_DIRS {
                    // Track new directories as writes
                    if !self.is_modification(&rel_pathbuf) {
                        diff.writes.push(rel_pathbuf);
                    }
                }
            }
        }

        // Add summary if truncated
        if diff.truncated {
            diff.tree_hash = Some(self.compute_tree_hash()?);
            diff.summary = Some(format!(
                "{} files, {} dirs (truncated at {}MB)",
                file_count,
                dir_count,
                total_size / (1024 * 1024)
            ));
        } else if file_count > 10 || dir_count > 5 {
            // Add summary for large but not truncated diffs
            diff.summary = Some(format!("{} files, {} dirs", file_count, dir_count));
        }

        Ok(diff)
    }

    /// Check if a file is a modification of an existing file.
    fn is_modification(&self, rel_path: &Path) -> bool {
        if let Some(lower) = &self.lower_dir {
            let candidate = lower.join(rel_path);
            return candidate.exists();
        }
        false
    }

    /// Compute a hash of the entire upper directory tree.
    fn compute_tree_hash(&self) -> Result<String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let mut hasher = Sha256::new();

        // Walk the upper directory in sorted order for deterministic hashing
        let mut entries: Vec<_> = WalkDir::new(&self.upper_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by_key(|e| e.path().to_path_buf());

        for entry in entries {
            let path = entry.path();

            // Hash the relative path
            if let Ok(rel_path) = path.strip_prefix(&self.upper_dir) {
                hasher.update(rel_path.to_string_lossy().as_bytes());
            }

            // Hash file metadata and content
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    hasher.update(metadata.len().to_le_bytes());

                    // Hash first 1KB of file content for efficiency
                    if let Ok(mut file) = std::fs::File::open(path) {
                        let mut buffer = [0; 1024];
                        if let Ok(n) = file.read(&mut buffer) {
                            hasher.update(&buffer[..n]);
                        }
                    }
                }
            }
        }

        Ok(format!("sha256:{:x}", hasher.finalize()))
    }

    /// Clean up the overlay directories.
    pub fn cleanup(&mut self) -> Result<()> {
        // Ensure unmounted first
        self.unmount()?;

        // Remove overlay directories
        if self.overlay_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.overlay_dir) {
                // Best-effort: if busy, leave for GC but do not fail replay
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
        use nix::mount::{mount, umount2, MntFlags, MsFlags};
        use std::thread::sleep;
        use std::time::Duration;

        // Prepare dirs
        std::fs::create_dir_all(&self.upper_dir)?;
        std::fs::create_dir_all(&self.work_dir)?;
        std::fs::create_dir_all(&self.merged_dir)?;

        // Bind-mount lower into a stable path
        let bind_lower = self.overlay_dir.join("lower");
        std::fs::create_dir_all(&bind_lower)?;
        let _ = umount2(&bind_lower, MntFlags::MNT_DETACH);
        mount(
            Some(lower_dir),
            &bind_lower,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )
        .with_context(|| {
            format!(
                "Failed to bind-mount lower {} -> {}",
                lower_dir.display(),
                bind_lower.display()
            )
        })?;

        let fuse_bin =
            which::which("fuse-overlayfs").context("fuse-overlayfs binary not found in PATH")?;
        let fuse_opts = format!(
            "lowerdir={},upperdir={},workdir={}",
            bind_lower.display(),
            self.upper_dir.display(),
            self.work_dir.display()
        );
        let mut child = Command::new(&fuse_bin)
            .arg("-o")
            .arg(&fuse_opts)
            .arg(&self.merged_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn fuse-overlayfs")?;

        // Wait up to ~1s for mount
        let mut ready = false;
        for _ in 0..30 {
            if let Ok(Some(fs_type)) = is_path_mounted(&self.merged_dir) {
                if fs_type.contains("fuse") || fs_type.contains("fuse-overlayfs") {
                    ready = true;
                    break;
                }
            }
            sleep(Duration::from_millis(33));
        }
        if !ready {
            let _ = child.kill();
            anyhow::bail!(
                "fuse-overlayfs did not mount {} within timeout",
                self.merged_dir.display()
            );
        }

        self.lower_dir = Some(lower_dir.to_path_buf());
        self.is_mounted = true;
        self.using_fuse = true;
        self.fuse_child = Some(child);
        Ok(self.merged_dir.clone())
    }
}

fn choose_base_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let uid = nix::unistd::Uid::current();
        if uid.is_root() {
            return Ok(PathBuf::from("/var/lib/substrate/overlay"));
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return Ok(PathBuf::from(xdg).join("substrate/overlay"));
            }
        }
        let run_user = PathBuf::from(format!("/run/user/{}/substrate/overlay", uid.as_raw()));
        if run_user.parent().unwrap_or(Path::new("/run")).exists() {
            return Ok(run_user);
        }
        Ok(PathBuf::from(format!(
            "/tmp/substrate-{}-overlay",
            uid.as_raw()
        )))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(PathBuf::from("/tmp/substrate-overlay"))
    }
}

impl Drop for OverlayFs {
    fn drop(&mut self) {
        // Best effort cleanup on drop
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
    // no chroot: rely on cd into target path under merged

    let mut overlay = OverlayFs::new(world_id)?;

    // Mount overlay with project directory as lower layer
    let merged_dir = overlay.mount(project_dir)?;

    // Execute command in merged by cd into the equivalent path under merged
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
    let output = std::process::Command::new("sh")
        .arg("-lc")
        .arg(cmd)
        .current_dir(&target_dir)
        .envs(env)
        .output()
        .context("Failed to execute command in overlay")?;

    // Compute diff before cleanup
    let diff = overlay.compute_diff()?;

    // Cleanup overlay
    overlay.cleanup()?;

    Ok((output, diff))
}

/// Helper to check if a path is currently a mountpoint by inspecting /proc/self/mounts.
#[cfg(target_os = "linux")]
fn is_path_mounted(path: &Path) -> Result<Option<String>> {
    let mounts =
        std::fs::read_to_string("/proc/self/mounts").context("failed reading /proc/self/mounts")?;
    let target = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    for line in mounts.lines() {
        // Format: src mountpoint fstype options ...
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let mp = parts[1];
            let fstype = parts[2];
            if let Ok(mp_real) = std::fs::canonicalize(mp) {
                if mp_real == target {
                    return Ok(Some(fstype.to_string()));
                }
            }
        }
    }
    Ok(None)
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
        // This test requires root privileges to mount overlayfs
        if !nix::unistd::Uid::current().is_root() {
            println!("Skipping overlay mount test (requires root)");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let lower_dir = temp_dir.path();

        let mut overlay = OverlayFs::new("test_mount").unwrap();

        // Mount may fail in containers with user namespaces disabled
        match overlay.mount(lower_dir) {
            Ok(merged) => {
                assert!(merged.exists());
                assert!(overlay.is_mounted);

                // Unmount should succeed
                overlay.unmount().unwrap();
                assert!(!overlay.is_mounted);

                // Cleanup
                overlay.cleanup().unwrap();
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("EINVAL")
                    || error_str.contains("Invalid argument")
                    || error_str.contains("Failed to mount overlayfs")
                {
                    println!("Skipping overlay mount test (overlayfs not supported in this environment): {}", e);
                    return;
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

        // Create some test files
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
}
