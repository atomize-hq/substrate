use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::Result;
use sha2::{Digest, Sha256};
use substrate_common::FsDiff;
use walkdir::WalkDir;

pub(crate) fn compute_diff(
    upper_dir: &Path,
    lower_dir: Option<&Path>,
    max_files: usize,
    max_dirs: usize,
    max_diff_size_bytes: usize,
) -> Result<FsDiff> {
    let mut diff = FsDiff::default();
    let mut file_count = 0;
    let mut dir_count = 0;
    let mut total_size = 0;

    for entry in WalkDir::new(upper_dir) {
        let entry = entry?;
        let path = entry.path();

        if path == upper_dir {
            continue;
        }

        let rel_path = path.strip_prefix(upper_dir)?;
        let rel_pathbuf = rel_path.to_path_buf();

        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if let Some(deleted_name) = name_str.strip_prefix(".wh.") {
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

            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len() as usize;
            }

            if file_count > max_files || total_size > max_diff_size_bytes {
                diff.truncated = true;
                break;
            }

            if is_modification(lower_dir, &rel_pathbuf) {
                diff.mods.push(rel_pathbuf.clone());
            } else {
                diff.writes.push(rel_pathbuf.clone());
            }
        } else if entry.file_type().is_dir() {
            dir_count += 1;
            if dir_count <= max_dirs && !is_modification(lower_dir, &rel_pathbuf) {
                diff.writes.push(rel_pathbuf);
            }
        }
    }

    if diff.truncated {
        diff.tree_hash = Some(compute_tree_hash(upper_dir)?);
        diff.summary = Some(format!(
            "{} files, {} dirs (truncated at {}MB)",
            file_count,
            dir_count,
            total_size / (1024 * 1024)
        ));
    } else if file_count > 10 || dir_count > 5 {
        diff.summary = Some(format!("{} files, {} dirs", file_count, dir_count));
    }

    Ok(diff)
}

fn is_modification(lower_dir: Option<&Path>, rel_path: &Path) -> bool {
    if let Some(lower) = lower_dir {
        let candidate = lower.join(rel_path);
        return candidate.exists();
    }
    false
}

fn compute_tree_hash(upper_dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();

    let mut entries: Vec<_> = WalkDir::new(upper_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in entries {
        let path = entry.path();

        if let Ok(rel_path) = path.strip_prefix(upper_dir) {
            hasher.update(rel_path.to_string_lossy().as_bytes());
        }

        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                hasher.update(metadata.len().to_le_bytes());

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

/// Helper to check if a path is currently a mountpoint by inspecting /proc/self/mounts.
#[cfg(target_os = "linux")]
pub(crate) fn is_path_mounted(path: &Path) -> Result<Option<String>> {
    let mounts =
        std::fs::read_to_string("/proc/self/mounts").context("failed reading /proc/self/mounts")?;
    let target = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    for line in mounts.lines() {
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
