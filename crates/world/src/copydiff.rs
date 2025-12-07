//! Userspace copy-diff fallback when overlay/fuse are unavailable.
//!
//! Strategy:
//! - Snapshot base and work trees using `cp -a --reflink=auto` when available.
//! - Execute the command inside the `work` copy at the appropriate cwd.
//! - Compare base vs work to produce FsDiff (writes/mods/deletes).
//! - Enforce limits and set `truncated`/`summary` when exceeded.

use anyhow::{anyhow, Context, Result};
use nix::libc;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error as StdError;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use substrate_common::FsDiff;
use walkdir::WalkDir;

const MAX_ENTRIES: usize = 10_000;
const MAX_BYTES_SAMPLE: usize = 8 * 1024; // 8KB sample for content compare

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopyDiffRootSource {
    EnvOverride,
    VarLib,
    XdgRuntime,
    Run,
    Tmp,
    VarTmp,
    Fallback,
}

impl CopyDiffRootSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            CopyDiffRootSource::EnvOverride => "env:SUBSTRATE_COPYDIFF_ROOT",
            CopyDiffRootSource::VarLib => "/var/lib/substrate/copydiff",
            CopyDiffRootSource::XdgRuntime => "xdg-runtime",
            CopyDiffRootSource::Run => "/run",
            CopyDiffRootSource::Tmp => "/tmp",
            CopyDiffRootSource::VarTmp => "/var/tmp",
            CopyDiffRootSource::Fallback => "auto",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyDiffCandidate {
    path: PathBuf,
    source: CopyDiffRootSource,
}

#[derive(Clone, Debug)]
pub struct CopyDiffOutcome {
    pub output: std::process::Output,
    pub fs_diff: FsDiff,
    pub child_pid: Option<u32>,
    pub root: PathBuf,
    pub root_source: CopyDiffRootSource,
}

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
        PathBuf::from(format!("/tmp/substrate-{}-copydiff", uid.as_raw()))
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
    netns: Option<&str>,
) -> Result<CopyDiffOutcome> {
    let mut last_error: Option<anyhow::Error> = None;
    for candidate in candidate_roots() {
        match execute_with_copydiff_root(
            &candidate.path,
            world_id,
            cmd,
            project_dir,
            cwd,
            env,
            netns,
        ) {
            Ok((output, fs_diff, child_pid_opt)) => {
                return Ok(CopyDiffOutcome {
                    output,
                    fs_diff,
                    child_pid: child_pid_opt,
                    root: candidate.path,
                    root_source: candidate.source,
                })
            }
            Err(err) => {
                log_copydiff_failure(&candidate, &err);
                last_error = Some(err);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("copydiff execution failed")))
}

fn override_root_from_env() -> Option<PathBuf> {
    std::env::var_os("SUBSTRATE_COPYDIFF_ROOT")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
}

fn root_source_for(path: &Path, env_override: Option<&Path>) -> CopyDiffRootSource {
    if env_override.is_some_and(|env| env == path) {
        return CopyDiffRootSource::EnvOverride;
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let xdg_path = PathBuf::from(xdg);
        if !xdg_path.as_os_str().is_empty() && path.starts_with(&xdg_path) {
            return CopyDiffRootSource::XdgRuntime;
        }
    }
    if path.starts_with("/var/lib/substrate/copydiff") {
        return CopyDiffRootSource::VarLib;
    }
    if path.starts_with("/run/user/") || path.starts_with("/run/substrate/") {
        return CopyDiffRootSource::Run;
    }
    if path.starts_with("/tmp") {
        return CopyDiffRootSource::Tmp;
    }
    if path.starts_with("/var/tmp") {
        return CopyDiffRootSource::VarTmp;
    }
    if path.starts_with("/var/run") || path.starts_with("/run") {
        return CopyDiffRootSource::Run;
    }
    CopyDiffRootSource::Fallback
}

fn push_candidate(
    seen: &mut HashSet<PathBuf>,
    roots: &mut Vec<CopyDiffCandidate>,
    path: PathBuf,
    source: CopyDiffRootSource,
) {
    if seen.insert(path.clone()) {
        roots.push(CopyDiffCandidate { path, source });
    }
}

pub(crate) fn candidate_roots() -> Vec<CopyDiffCandidate> {
    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    let env_override = override_root_from_env();
    if let Some(root) = env_override.clone() {
        push_candidate(&mut seen, &mut roots, root, CopyDiffRootSource::EnvOverride);
    }

    let primary = choose_base_dir();
    let primary_source = root_source_for(&primary, env_override.as_deref());
    push_candidate(&mut seen, &mut roots, primary.clone(), primary_source);

    #[cfg(target_os = "linux")]
    {
        let uid = nix::unistd::Uid::current().as_raw();
        let run_root = if uid == 0 {
            PathBuf::from("/run/substrate/copydiff")
        } else {
            PathBuf::from(format!("/run/user/{}/substrate/copydiff", uid))
        };
        if run_root.parent().unwrap_or(Path::new("/run")).exists() {
            push_candidate(&mut seen, &mut roots, run_root, CopyDiffRootSource::Run);
        }

        let run_root = PathBuf::from("/run/substrate/copydiff");
        push_candidate(&mut seen, &mut roots, run_root, CopyDiffRootSource::Run);

        let tmp_root = PathBuf::from(format!("/tmp/substrate-{}-copydiff", uid));
        push_candidate(&mut seen, &mut roots, tmp_root, CopyDiffRootSource::Tmp);

        let var_tmp_root = PathBuf::from(format!("/var/tmp/substrate-{}-copydiff", uid));
        push_candidate(
            &mut seen,
            &mut roots,
            var_tmp_root,
            CopyDiffRootSource::VarTmp,
        );
    }

    #[cfg(not(target_os = "linux"))]
    {
        let fallback = PathBuf::from("/tmp/substrate-copydiff");
        push_candidate(&mut seen, &mut roots, fallback, CopyDiffRootSource::Tmp);
    }

    roots
}

fn log_copydiff_failure(candidate: &CopyDiffCandidate, err: &anyhow::Error) {
    static WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let key = format!("{}::{}", candidate.path.display(), err);
    let mut seen = WARNED
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !seen.insert(key) {
        return;
    }

    if is_enospc(err) {
        eprintln!(
            "[replay] warn: copy-diff storage {} ({}) ran out of space; retrying fallback location",
            candidate.path.display(),
            candidate.source.as_str()
        );
    } else {
        eprintln!(
            "[replay] warn: copy-diff failed in {} ({}); retrying fallback location",
            candidate.path.display(),
            err
        );
    }
}

fn execute_with_copydiff_root(
    root: &Path,
    world_id: &str,
    cmd: &str,
    project_dir: &Path,
    cwd: &Path,
    env: &std::collections::HashMap<String, String>,
    netns: Option<&str>,
) -> Result<(std::process::Output, FsDiff, Option<u32>)> {
    fs::create_dir_all(root).context("failed to create copydiff base dir")?;
    let base = root.join(format!("{}-base", world_id));
    let work = root.join(format!("{}-work", world_id));

    // Clean any leftovers and recreate
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&base)?;
    fs::create_dir_all(&work)?;

    let result = (|| -> Result<(std::process::Output, FsDiff, Option<u32>)> {
        // Snapshot base and work using cp -a --reflink=auto when available
        copy_tree(project_dir, &base)?;
        copy_tree(project_dir, &work)?;

        // Execute under work at the mapped cwd
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
        let target_dir = work.join(&rel);
        let mut command = if netns.is_some() {
            Command::new("ip")
        } else {
            Command::new("sh")
        };
        if let Some(ns) = netns {
            command.args(["netns", "exec", ns, "sh", "-lc", cmd]);
        } else {
            command.args(["-lc", cmd]);
        }
        let child = command
            .current_dir(&target_dir)
            .envs(env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed spawning command under copydiff work dir")?;
        let child_pid = Some(child.id());
        let output = child
            .wait_with_output()
            .context("failed waiting for command under copydiff work dir")?;

        // Compute diff base vs work
        let diff = compute_diff(&base, &work)?;

        Ok((output, diff, child_pid))
    })();

    // Cleanup regardless of success
    let _ = fs::remove_dir_all(&base);
    let _ = fs::remove_dir_all(&work);

    result
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    // Prefer cp -a --reflink=auto for speed; fall back to recursive copy in Rust
    let status = Command::new("cp")
        .arg("-a")
        .arg("--reflink=auto")
        .arg(format!("{}/.", from.display()))
        .arg(to)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            // Fallback: walk + copy
            for entry in WalkDir::new(from) {
                let entry = entry?;
                let rel = entry.path().strip_prefix(from).with_context(|| {
                    format!("computing relative path for {}", entry.path().display())
                })?;
                if rel.as_os_str().is_empty() {
                    continue;
                }
                let dest = to.join(rel);
                if entry.file_type().is_dir() {
                    fs::create_dir_all(&dest)?;
                } else if entry.file_type().is_file() {
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(entry.path(), &dest).with_context(|| {
                        format!("copy {} -> {}", entry.path().display(), dest.display())
                    })?;
                } else if entry.file_type().is_symlink() {
                    #[cfg(unix)]
                    {
                        if let Ok(target) = fs::read_link(entry.path()) {
                            if let Some(parent) = dest.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            std::os::unix::fs::symlink(target, &dest)
                                .with_context(|| format!("symlink {}", dest.display()))?;
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

fn is_enospc(err: &anyhow::Error) -> bool {
    let mut current: Option<&(dyn StdError + 'static)> = Some(err.as_ref());
    while let Some(err) = current {
        if let Some(io) = err.downcast_ref::<std::io::Error>() {
            if io.raw_os_error() == Some(libc::ENOSPC) {
                return true;
            }
        }
        current = err.source();
    }
    false
}

fn compute_diff(base: &Path, work: &Path) -> Result<FsDiff> {
    let mut diff = FsDiff::default();
    let mut base_map = BTreeMap::new();
    let mut work_map = BTreeMap::new();

    for entry in WalkDir::new(base) {
        let entry = entry?;
        let rel = entry
            .path()
            .strip_prefix(base)
            .with_context(|| {
                format!(
                    "computing base relative path for {}",
                    entry.path().display()
                )
            })?
            .to_path_buf();
        if rel.as_os_str().is_empty() {
            continue;
        }
        base_map.insert(rel, meta_of(entry.path()));
    }
    for entry in WalkDir::new(work) {
        let entry = entry?;
        let rel = entry
            .path()
            .strip_prefix(work)
            .with_context(|| {
                format!(
                    "computing work relative path for {}",
                    entry.path().display()
                )
            })?
            .to_path_buf();
        if rel.as_os_str().is_empty() {
            continue;
        }
        work_map.insert(rel, meta_of(entry.path()));
    }

    let base_keys: BTreeSet<_> = base_map.keys().cloned().collect();
    let work_keys: BTreeSet<_> = work_map.keys().cloned().collect();

    // Writes (only in work)
    for rel in work_keys.difference(&base_keys) {
        if diff.total_changes() >= MAX_ENTRIES {
            diff.truncated = true;
            break;
        }
        diff.writes.push(rel.clone());
    }

    // Deletes (only in base)
    for rel in base_keys.difference(&work_keys) {
        if diff.total_changes() >= MAX_ENTRIES {
            diff.truncated = true;
            break;
        }
        diff.deletes.push(rel.clone());
    }

    // Mods or type-changes
    for rel in base_keys.intersection(&work_keys) {
        if diff.total_changes() >= MAX_ENTRIES {
            diff.truncated = true;
            break;
        }
        let b = base_map
            .get(rel)
            .with_context(|| format!("missing base entry for {}", rel.display()))?;
        let w = work_map
            .get(rel)
            .with_context(|| format!("missing work entry for {}", rel.display()))?;
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
            base_map.len(),
            work_map.len()
        ));
    } else {
        diff.summary = Some(format!(
            "copy-diff: compared {}→{} entries",
            base_map.len(),
            work_map.len()
        ));
    }

    Ok(diff)
}

struct Meta {
    path: PathBuf,
    kind: String, // f|d|l|?
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
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut mode = 0u32;
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut uid = 0u32;
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut gid = 0u32;
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut mtime_sec = 0i64;
    #[cfg_attr(not(unix), allow(unused_mut))]
    let mut mtime_nsec = 0i64;
    let mut symlink_target: Option<PathBuf> = None;

    if let Some(m) = md.as_ref() {
        let ft = m.file_type();
        if ft.is_dir() {
            kind = "d".into();
        } else if ft.is_file() {
            kind = "f".into();
        } else if ft.is_symlink() {
            kind = "l".into();
        }
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

    Meta {
        path: path.to_path_buf(),
        kind,
        mode,
        uid,
        gid,
        mtime_sec,
        mtime_nsec,
        symlink_target,
    }
}

fn meta_differs(a: &Meta, b: &Meta) -> bool {
    if a.mode != b.mode || a.uid != b.uid || a.gid != b.gid {
        return true;
    }
    if a.mtime_sec != b.mtime_sec || a.mtime_nsec != b.mtime_nsec {
        return true;
    }
    false
}

fn files_differ(a: &Path, b: &Path) -> bool {
    let ma = fs::metadata(a).ok();
    let mb = fs::metadata(b).ok();
    if let (Some(ma), Some(mb)) = (ma, mb) {
        if ma.len() != mb.len() {
            return true;
        }
    }
    // Compare first N bytes
    let mut fa = match fs::File::open(a) {
        Ok(f) => f,
        Err(_) => return true,
    };
    let mut fb = match fs::File::open(b) {
        Ok(f) => f,
        Err(_) => return true,
    };
    let mut ba = vec![0u8; MAX_BYTES_SAMPLE];
    let mut bb = vec![0u8; MAX_BYTES_SAMPLE];
    let ra = fa.read(&mut ba).unwrap_or(0);
    let rb = fb.read(&mut bb).unwrap_or(0);
    if ra != rb {
        return true;
    }
    ba[..ra] != bb[..rb]
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn clean_copydiff_roots() {
        for root in candidate_roots() {
            let _ = std::fs::remove_dir_all(&root.path);
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_copydiff_detects_metadata_mod() {
        clean_copydiff_roots();
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        std::fs::write(project.join("file.txt"), b"data").unwrap();

        let env = std::collections::HashMap::new();
        let outcome = execute_with_copydiff(
            "test-md",
            "sh -lc 'echo data > file.txt'",
            project,
            project,
            &env,
            None,
        )
        .unwrap();

        let diff = outcome.fs_diff;
        assert!(diff.mods.iter().any(|p| p.to_string_lossy() == "file.txt"));
    }

    #[cfg(unix)]
    #[test]
    fn test_copydiff_detects_write_and_delete() {
        clean_copydiff_roots();
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        std::fs::write(project.join("old.txt"), b"old").unwrap();

        let env = std::collections::HashMap::new();
        let outcome = execute_with_copydiff(
            "test-wd",
            "sh -lc 'rm -f old.txt && mkdir -p demo && echo data > demo/file.txt'",
            project,
            project,
            &env,
            None,
        )
        .unwrap();

        let diff = outcome.fs_diff;
        assert!(diff.writes.iter().any(|p| p.to_string_lossy() == "demo"));
        assert!(diff
            .writes
            .iter()
            .any(|p| p.to_string_lossy() == "demo/file.txt"));
        assert!(diff
            .deletes
            .iter()
            .any(|p| p.to_string_lossy() == "old.txt"));
    }
}
