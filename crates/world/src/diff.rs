//! Filesystem difference computation and overlayfs support.

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use world_api::FsDiff;

const MAX_TRACKED_DIRS: usize = 100;
const MAX_FILE_LIST: usize = 1000;

/// Compute filesystem differences with smart truncation.
pub fn compute_fs_diff_smart(upper: &Path) -> Result<FsDiff> {
    let mut diff = FsDiff::default();
    let mut dir_count = 0;

    // For huge installs, track top-level changes + hash
    for entry in WalkDir::new(upper).max_depth(3) {
        let entry = entry?;

        if entry.file_type().is_dir() {
            dir_count += 1;
            if dir_count <= MAX_TRACKED_DIRS {
                // diff.created_dirs.push(entry.path().to_path_buf());
                // Note: FsDiff doesn't have created_dirs field, using writes instead
                diff.writes.push(entry.path().to_path_buf());
            }
        } else if diff.writes.len() < MAX_FILE_LIST {
            diff.writes.push(entry.path().to_path_buf());
        }
    }

    // If we hit limits, add summary + tree hash
    if dir_count > MAX_TRACKED_DIRS || diff.writes.len() >= MAX_FILE_LIST {
        diff.truncated = true;
        diff.tree_hash = Some(hash_directory_tree(upper)?);
        diff.summary = Some(format!(
            "{} dirs, {} files (truncated, see tree_hash)",
            dir_count,
            count_files(upper)?
        ));
    }

    Ok(diff)
}

/// Execute a command with overlayfs isolation.
pub fn execute_with_overlay(cmd: &str, project_dir: &Path) -> Result<FsDiff> {
    // CRITICAL: upper/work must be on same filesystem
    let overlay_base = PathBuf::from("/var/lib/substrate/overlay");
    std::fs::create_dir_all(&overlay_base)?;

    let world_id = format!("ovl_{}", uuid::Uuid::now_v7());
    let overlay_dir = overlay_base.join(&world_id);

    let upper = overlay_dir.join("upper");
    let work = overlay_dir.join("work");
    let merged = overlay_dir.join("merged");

    std::fs::create_dir_all(&upper)?;
    std::fs::create_dir_all(&work)?;
    std::fs::create_dir_all(&merged)?;

    // Mount overlay
    mount_overlay(project_dir, &upper, &work, &merged)?;

    // Execute command with merged as root
    let _result = execute_in_dir(cmd, &merged)?;

    // Compute diff from upper directory (with size limits)
    let diff = compute_fs_diff_smart(&upper)?;

    // Cleanup
    cleanup_overlay(&merged, &overlay_dir)?;

    Ok(diff)
}

#[cfg(target_os = "linux")]
fn mount_overlay(lower: &Path, upper: &Path, work: &Path, merged: &Path) -> Result<()> {
    use nix::mount::{mount, MsFlags};

    let options = format!(
        "lowerdir={},upperdir={},workdir={}",
        lower.display(),
        upper.display(),
        work.display()
    );

    mount(
        Some("overlay"),
        merged,
        Some("overlay"),
        MsFlags::empty(),
        Some(options.as_bytes()),
    )?;

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn mount_overlay(_lower: &Path, _upper: &Path, _work: &Path, _merged: &Path) -> Result<()> {
    anyhow::bail!("Overlay filesystem not supported on this platform")
}

#[cfg(target_os = "linux")]
fn cleanup_overlay(merged: &Path, overlay_dir: &Path) -> Result<()> {
    use nix::mount::{umount2, MntFlags};

    // Unmount overlay
    umount2(merged, MntFlags::MNT_DETACH)?;

    // Remove overlay directories
    std::fs::remove_dir_all(overlay_dir)?;

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn cleanup_overlay(_merged: &Path, overlay_dir: &Path) -> Result<()> {
    // Just remove the directories
    std::fs::remove_dir_all(overlay_dir)?;
    Ok(())
}

fn execute_in_dir(cmd: &str, dir: &Path) -> Result<std::process::Output> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(dir)
        .output()?;

    Ok(output)
}

fn hash_directory_tree(dir: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut hasher = Sha256::new();

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let path = entry.path();

        // Hash the path
        hasher.update(path.to_string_lossy().as_bytes());

        // Hash file content if it's a file
        if entry.file_type().is_file() {
            if let Ok(mut file) = std::fs::File::open(path) {
                let mut buffer = [0; 8192];
                loop {
                    match file.read(&mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => hasher.update(&buffer[..n]),
                        Err(_) => break, // Skip unreadable files
                    }
                }
            }
        }
    }

    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn count_files(dir: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_fs_diff_computation() {
        let temp_dir = TempDir::new().unwrap();
        let upper = temp_dir.path();

        // Create some test files
        std::fs::write(upper.join("test1.txt"), "content1").unwrap();
        std::fs::write(upper.join("test2.txt"), "content2").unwrap();

        let diff = compute_fs_diff_smart(upper).unwrap();
        // Expect 3 entries: 1 directory (root) + 2 files
        assert_eq!(diff.writes.len(), 3);
        assert!(!diff.truncated);
    }

    #[test]
    fn test_directory_tree_hashing() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path();

        std::fs::write(dir.join("test.txt"), "content").unwrap();

        let hash1 = hash_directory_tree(dir).unwrap();
        let hash2 = hash_directory_tree(dir).unwrap();

        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("sha256:"));
    }
}
