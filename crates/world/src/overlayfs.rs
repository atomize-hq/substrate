//! Overlayfs-based filesystem diff tracking for Linux.
//!
//! This module provides overlayfs-based filesystem isolation and diff tracking
//! for commands that need to be executed in an isolated environment.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use substrate_common::FsDiff;
use walkdir::WalkDir;

const MAX_TRACKED_FILES: usize = 1000;
const MAX_TRACKED_DIRS: usize = 100;
const MAX_DIFF_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10MB

/// Overlayfs manager for filesystem isolation and diff tracking.
pub struct OverlayFs {
    base_dir: PathBuf,
    world_id: String,
    overlay_dir: PathBuf,
    upper_dir: PathBuf,
    work_dir: PathBuf,
    merged_dir: PathBuf,
    is_mounted: bool,
}

impl OverlayFs {
    /// Create a new overlayfs instance for the given world.
    pub fn new(world_id: &str) -> Result<Self> {
        let base_dir = PathBuf::from("/var/lib/substrate/overlay");
        std::fs::create_dir_all(&base_dir)
            .context("Failed to create overlay base directory")?;

        let overlay_dir = base_dir.join(world_id);
        let upper_dir = overlay_dir.join("upper");
        let work_dir = overlay_dir.join("work");
        let merged_dir = overlay_dir.join("merged");

        Ok(Self {
            base_dir,
            world_id: world_id.to_string(),
            overlay_dir,
            upper_dir,
            work_dir,
            merged_dir,
            is_mounted: false,
        })
    }

    /// Mount the overlayfs with the given lower directory.
    pub fn mount(&mut self, #[allow(unused_variables)] lower_dir: &Path) -> Result<PathBuf> {
        if self.is_mounted {
            return Ok(self.merged_dir.clone());
        }

        // Create overlay directories
        std::fs::create_dir_all(&self.upper_dir)
            .context("Failed to create upper directory")?;
        std::fs::create_dir_all(&self.work_dir)
            .context("Failed to create work directory")?;
        std::fs::create_dir_all(&self.merged_dir)
            .context("Failed to create merged directory")?;

        // Mount overlay filesystem
        #[cfg(target_os = "linux")]
        {
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
    fn mount_linux(&self, lower_dir: &Path) -> Result<()> {
        use nix::mount::{mount, MsFlags};

        let options = format!(
            "lowerdir={},upperdir={},workdir={}",
            lower_dir.display(),
            self.upper_dir.display(),
            self.work_dir.display()
        );

        mount(
            Some("overlay"),
            &self.merged_dir,
            Some("overlay"),
            MsFlags::empty(),
            Some(options.as_str()),
        )
        .context("Failed to mount overlayfs")?;

        Ok(())
    }

    /// Unmount the overlayfs.
    pub fn unmount(&mut self) -> Result<()> {
        if !self.is_mounted {
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            use nix::mount::{umount2, MntFlags};
            umount2(&self.merged_dir, MntFlags::MNT_DETACH)
                .context("Failed to unmount overlayfs")?;
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
                    if name_str.starts_with(".wh.") {
                        // This is a whiteout file indicating deletion
                        let deleted_name = &name_str[4..]; // Remove ".wh." prefix
                        let deleted_path = rel_path.parent()
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
    fn is_modification(&self, _rel_path: &Path) -> bool {
        // In overlayfs, we can check if a file is modified by looking for
        // certain metadata or by comparing with the lower layer.
        // For now, we'll consider all files as new writes unless we implement
        // more sophisticated detection.
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
                    hasher.update(&metadata.len().to_le_bytes());
                    
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
            std::fs::remove_dir_all(&self.overlay_dir)
                .context("Failed to remove overlay directory")?;
        }

        Ok(())
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
    
    let mut overlay = OverlayFs::new(world_id)?;
    
    // Mount overlay with project directory as lower layer
    let merged_dir = overlay.mount(project_dir)?;
    
    // Calculate the working directory within the merged filesystem
    let exec_cwd = if cwd.starts_with(project_dir) {
        let rel_path = cwd.strip_prefix(project_dir)?;
        merged_dir.join(rel_path)
    } else {
        merged_dir.clone()
    };

    // Execute command in the merged directory
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(&exec_cwd)
        .envs(env)
        .output()
        .context("Failed to execute command in overlay")?;

    // Compute diff before cleanup
    let diff = overlay.compute_diff()?;

    // Cleanup overlay
    overlay.cleanup()?;

    Ok((output, diff))
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
        
        // Mount should succeed
        let merged = overlay.mount(lower_dir).unwrap();
        assert!(merged.exists());
        assert!(overlay.is_mounted);
        
        // Unmount should succeed
        overlay.unmount().unwrap();
        assert!(!overlay.is_mounted);
        
        // Cleanup
        overlay.cleanup().unwrap();
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
            base_dir: temp_dir.path().to_path_buf(),
            world_id: "test".to_string(),
            overlay_dir: temp_dir.path().to_path_buf(),
            upper_dir,
            work_dir: temp_dir.path().join("work"),
            merged_dir: temp_dir.path().join("merged"),
            is_mounted: false,
        };
        
        let diff = overlay.compute_diff().unwrap();
        assert!(!diff.writes.is_empty());
        assert!(!diff.truncated);
    }
}