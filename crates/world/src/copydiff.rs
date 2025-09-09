//! Userspace copy-diff fallback when overlay/fuse are unavailable.
//!
//! Strategy:
//! - Snapshot base and work trees using `cp -a --reflink=auto` when available.
//! - Execute the command inside the `work` copy at the appropriate cwd.
//! - Compare base vs work to produce FsDiff (writes/mods/deletes).
//! - Enforce limits and set `truncated`/`summary` when exceeded.

use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use substrate_common::FsDiff;
use walkdir::WalkDir;

const MAX_ENTRIES: usize = 10_000;
const MAX_BYTES_SAMPLE: usize = 8 * 1024; // 8KB sample for content compare

fn choose_base_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let uid = nix::unistd::Uid::current();
        if uid.is_root() {
            return PathBuf::from("/var/lib/substrate/copydiff");
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("substrate/copydiff");
            }
        }
        let run_user = PathBuf::from(format!("/run/user/{}/substrate/copydiff", uid.as_raw()));
        if run_user.parent().unwrap_or(Path::new("/run")).exists() {
            return run_user;
        }
        return PathBuf::from(format!("/tmp/substrate-{}-copydiff", uid.as_raw()));
    }
    #[cfg(not(target_os = "linux"))]
    {
        PathBuf::from("/tmp/substrate-copydiff")
    }
}

/// Execute a command by copying the project dir, then diff base vs work.
pub fn execute_with_copydiff(
    world_id: &str,
    cmd: &str,
    project_dir: &Path,
    cwd: &Path,
    env: &std::collections::HashMap<String, String>,
) -> Result<(std::process::Output, FsDiff)> {
    let root = choose_base_dir();
    fs::create_dir_all(&root).context("failed to create copydiff base dir")?;
    let base = root.join(format!("{}-base", world_id));
    let work = root.join(format!("{}-work", world_id));

    // Clean any leftovers and recreate
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&base)?;
    fs::create_dir_all(&work)?;

    // Snapshot base and work using cp -a --reflink=auto when available
    copy_tree(project_dir, &base)?;
    copy_tree(project_dir, &work)?;

    // Execute under work at the mapped cwd
    let mut rel = if cwd.starts_with(project_dir) {
        cwd.strip_prefix(project_dir).unwrap_or_else(|_| Path::new(".")).to_path_buf()
    } else {
        PathBuf::from(".")
    };
    if rel.as_os_str().is_empty() { rel = PathBuf::from("."); }
    let target_dir = work.join(&rel);
    let output = Command::new("sh")
        .arg("-lc")
        .arg(cmd)
        .current_dir(&target_dir)
        .envs(env)
        .output()
        .context("failed executing command under copydiff work dir")?;

    // Compute diff base vs work
    let diff = compute_diff(&base, &work)?;

    // Cleanup
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&work);

    Ok((output, diff))
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    // Prefer cp -a --reflink=auto for speed; fall back to recursive copy in Rust
    let status = Command::new("cp")
        .arg("-a")
        .arg("--reflink=auto")
        .arg(format!("{}/.", from.display()))
        .arg(to)
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            // Fallback: walk + copy
            for entry in WalkDir::new(from) {
                let entry = entry?;
                let rel = entry.path().strip_prefix(from).unwrap();
                if rel.as_os_str().is_empty() { continue; }
                let dest = to.join(rel);
                if entry.file_type().is_dir() {
                    fs::create_dir_all(&dest)?;
                } else if entry.file_type().is_file() {
                    if let Some(parent) = dest.parent() { fs::create_dir_all(parent)?; }
                    fs::copy(entry.path(), &dest).with_context(|| format!("copy {} -> {}", entry.path().display(), dest.display()))?;
                } else if entry.file_type().is_symlink() {
                    // Preserve symlink
                    if let Ok(target) = fs::read_link(entry.path()) {
                        if let Some(parent) = dest.parent() { fs::create_dir_all(parent)?; }
                        #[cfg(unix)]
                        std::os::unix::fs::symlink(target, &dest).with_context(|| format!("symlink {}", dest.display()))?;
                    }
                }
            }
            Ok(())
        }
    }
}

fn compute_diff(base: &Path, work: &Path) -> Result<FsDiff> {
    let mut diff = FsDiff::default();
    let mut base_map = BTreeMap::new();
    let mut work_map = BTreeMap::new();

    for entry in WalkDir::new(base) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(base).unwrap().to_path_buf();
        if rel.as_os_str().is_empty() { continue; }
        base_map.insert(rel, meta_of(entry.path()));
    }
    for entry in WalkDir::new(work) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(work).unwrap().to_path_buf();
        if rel.as_os_str().is_empty() { continue; }
        work_map.insert(rel, meta_of(entry.path()));
    }

    let base_keys: BTreeSet<_> = base_map.keys().cloned().collect();
    let work_keys: BTreeSet<_> = work_map.keys().cloned().collect();

    // Writes (only in work)
    for rel in work_keys.difference(&base_keys) {
        if diff.total_changes() >= MAX_ENTRIES { diff.truncated = true; break; }
        diff.writes.push(rel.clone());
    }

    // Deletes (only in base)
    for rel in base_keys.difference(&work_keys) {
        if diff.total_changes() >= MAX_ENTRIES { diff.truncated = true; break; }
        diff.deletes.push(rel.clone());
    }

    // Mods or type-changes
    for rel in base_keys.intersection(&work_keys) {
        if diff.total_changes() >= MAX_ENTRIES { diff.truncated = true; break; }
        let b = base_map.get(rel).unwrap();
        let w = work_map.get(rel).unwrap();
        match (&b.kind[..], &w.kind[..]) {
            ("d", "d") => { /* ignore dir metadata for now to reduce noise */ }
            ("f", "f") => {
                let content_changed = files_differ(&b.path, &w.path);
                let meta_changed = meta_differs(b, w);
                if content_changed || meta_changed {
                    diff.mods.push(rel.clone());
                }
            }
            ("l", "l") => {
                let target_changed = b.symlink_target != w.symlink_target;
                let meta_changed = meta_differs(b, w);
                if target_changed || meta_changed {
                    diff.mods.push(rel.clone());
                }
            }
            // Type change: model as delete+write
            _ => {
                diff.deletes.push(rel.clone());
                diff.writes.push(rel.clone());
            }
        }
    }

    if diff.truncated {
        diff.summary = Some(format!(
            "copy-diff: compared {}→{} entries (truncated)",
            base_map.len(), work_map.len()
        ));
    } else {
        diff.summary = Some(format!(
            "copy-diff: compared {}→{} entries",
            base_map.len(), work_map.len()
        ));
    }

    Ok(diff)
}

struct Meta {
    path: PathBuf,
    kind: String, // f|d|l|?
    size: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    mtime_sec: i64,
    mtime_nsec: i64,
    symlink_target: Option<PathBuf>,
}

fn meta_of(path: &Path) -> Meta {
    let md = fs::symlink_metadata(path).ok();
    let mut kind = "?".to_string();
    let mut size = 0u64;
    let mut mode = 0u32;
    let mut uid = 0u32;
    let mut gid = 0u32;
    let mut mtime_sec = 0i64;
    let mut mtime_nsec = 0i64;
    let mut symlink_target: Option<PathBuf> = None;

    if let Some(m) = md.as_ref() {
        let ft = m.file_type();
        if ft.is_dir() { kind = "d".into(); }
        else if ft.is_file() { kind = "f".into(); }
        else if ft.is_symlink() { kind = "l".into(); }
        size = m.len();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            mode = m.mode();
            uid = m.uid();
            gid = m.gid();
            mtime_sec = m.mtime();
            mtime_nsec = m.mtime_nsec();
        }
        if ft.is_symlink() {
            symlink_target = fs::read_link(path).ok();
        }
    }

    Meta { path: path.to_path_buf(), kind, size, mode, uid, gid, mtime_sec, mtime_nsec, symlink_target }
}

fn meta_differs(a: &Meta, b: &Meta) -> bool {
    if a.mode != b.mode || a.uid != b.uid || a.gid != b.gid { return true; }
    if a.mtime_sec != b.mtime_sec || a.mtime_nsec != b.mtime_nsec { return true; }
    false
}

fn files_differ(a: &Path, b: &Path) -> bool {
    let ma = fs::metadata(a).ok();
    let mb = fs::metadata(b).ok();
    if let (Some(ma), Some(mb)) = (ma, mb) {
        if ma.len() != mb.len() { return true; }
    }
    // Compare first N bytes
    let mut fa = match fs::File::open(a) { Ok(f) => f, Err(_) => return true };
    let mut fb = match fs::File::open(b) { Ok(f) => f, Err(_) => return true };
    let mut ba = vec![0u8; MAX_BYTES_SAMPLE];
    let mut bb = vec![0u8; MAX_BYTES_SAMPLE];
    let ra = fa.read(&mut ba).unwrap_or(0);
    let rb = fb.read(&mut bb).unwrap_or(0);
    if ra != rb { return true; }
    ba[..ra] != bb[..rb]
}
