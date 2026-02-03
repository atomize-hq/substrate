use crate::enforcement_plan;
use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use serde_json::json;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};

pub const LANDLOCK_EXEC_ARG: &str = "__substrate_world_landlock_exec";

const INNER_CMD_ENV: &str = "SUBSTRATE_INNER_CMD";
const INNER_LOGIN_SHELL_ENV: &str = "SUBSTRATE_INNER_LOGIN_SHELL";
const MOUNT_CWD_ENV: &str = "SUBSTRATE_MOUNT_CWD";
#[cfg(target_os = "linux")]
const MOUNT_FS_MODE_ENV: &str = "SUBSTRATE_MOUNT_FS_MODE";
#[cfg(target_os = "linux")]
const MOUNT_PROJECT_DIR_ENV: &str = "SUBSTRATE_MOUNT_PROJECT_DIR";
#[cfg(target_os = "linux")]
const WORLD_FS_ISOLATION_ENV: &str = "SUBSTRATE_WORLD_FS_ISOLATION";

const LANDLOCK_READ_ENV: &str = "SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST";
const LANDLOCK_WRITE_ENV: &str = "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST";
const LANDLOCK_DISCOVER_ENV: &str = "SUBSTRATE_WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST";

pub fn run_landlock_exec() -> Result<()> {
    let enforcement_plan = match enforcement_plan::read_from_env_and_validate() {
        Ok(v) => v,
        Err(err) => {
            #[cfg(target_os = "linux")]
            {
                eprintln!(
                    "substrate: error: invalid enforcement plan: {}",
                    serde_json::to_string(&json!({
                        "feature": "world-fs-enforcement-plan",
                        "env": enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV,
                        "reason": err.to_string(),
                        "remediation": "ensure SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64 is base64 JSON matching the v1 schema; unknown fields are rejected",
                    }))
                    .unwrap_or_else(|_| "<unserializable>".to_string())
                );
            }
            #[cfg(not(target_os = "linux"))]
            {
                eprintln!(
                    "substrate: error: invalid enforcement plan (world-fs-enforcement-plan, env={}): {err}",
                    enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV
                );
            }
            std::process::exit(4);
        }
    };

    let read_paths = parse_allowlist_env(LANDLOCK_READ_ENV);
    let write_paths = parse_allowlist_env(LANDLOCK_WRITE_ENV);
    let discover_paths = parse_allowlist_env(LANDLOCK_DISCOVER_ENV);
    let discover_paths = if discover_paths.is_empty() {
        read_paths.clone()
    } else {
        discover_paths
    };

    #[cfg(target_os = "linux")]
    {
        let mut write_paths = write_paths;
        let mut deny_report: Option<DenyMaskApplyReport> = None;

        fn extend_with_overlayfs_backing_dirs_strict(
            policy: &mut world::landlock::LandlockFilesystemPolicy,
            mount_point: &str,
        ) -> Result<()> {
            let backing =
                world::mountinfo::overlay_backing_dirs_for_mount_point_strict(mount_point)?;

            policy.write_paths.push(backing.upperdir);
            policy.write_paths.push(backing.workdir);
            for lower in backing.lowerdirs {
                policy.read_paths.push(lower);
            }

            Ok(())
        }

        let isolation_full = std::env::var(WORLD_FS_ISOLATION_ENV)
            .ok()
            .is_some_and(|raw| raw.trim().eq_ignore_ascii_case("full"));

        if let Some(plan) = enforcement_plan.as_ref() {
            if !isolation_full {
                eprintln!(
                    "substrate: error: deny lists are only supported in full isolation: {}",
                    serde_json::to_string(&json!({
                        "feature": "world-fs-deny-masking",
                        "reason": "enforcement plan present but SUBSTRATE_WORLD_FS_ISOLATION was not 'full'",
                        "remediation": "ensure world_fs.isolation=full when using deny_list",
                    }))
                    .unwrap_or_else(|_| "<unserializable>".to_string())
                );
                std::process::exit(4);
            }

            let project_dir = std::env::var(MOUNT_PROJECT_DIR_ENV)
                .ok()
                .map(|raw| raw.trim().to_string())
                .filter(|t| !t.is_empty());

            let Some(project_dir) = project_dir else {
                eprintln!(
                    "substrate: error: deny masking prerequisites missing: {}",
                    serde_json::to_string(&json!({
                        "feature": "world-fs-deny-masking",
                        "reason": "missing or empty SUBSTRATE_MOUNT_PROJECT_DIR",
                        "remediation": "this exec path requires SUBSTRATE_MOUNT_PROJECT_DIR to apply deny masks to all in-world project views",
                    }))
                    .unwrap_or_else(|_| "<unserializable>".to_string())
                );
                std::process::exit(4);
            };

            match apply_deny_masks_linux(plan, &project_dir) {
                Ok(report) => {
                    write_paths.extend(report.write_allowlist_paths.clone());
                    deny_report = Some(report);
                }
                Err(err) => {
                    eprintln!(
                        "substrate: error: deny masking failed: {}",
                        serde_json::to_string(&json!({
                            "feature": "world-fs-deny-masking",
                            "reason": err.to_string(),
                            "remediation": "deny masks must be applied before executing user code; check mount namespace prerequisites and deny patterns",
                        }))
                        .unwrap_or_else(|_| "<unserializable>".to_string())
                    );
                    std::process::exit(4);
                }
            }
        }

        let cleanup_needed = deny_report
            .as_ref()
            .is_some_and(|r| !r.cleanup_created_dirs.is_empty());

        let cwd = std::env::var(MOUNT_CWD_ENV).unwrap_or_else(|_| "/".to_string());
        let cmd = std::env::var(INNER_CMD_ENV).context("missing SUBSTRATE_INNER_CMD")?;
        let login_shell = std::env::var(INNER_LOGIN_SHELL_ENV)
            .ok()
            .is_some_and(|raw| raw.trim() == "1");

        let apply_workload_restrictions = |mut read_paths: Vec<String>,
                                           mut write_paths: Vec<String>,
                                           mut discover_paths: Vec<String>|
         -> Result<()> {
            if isolation_full {
                let landlock_intended =
                    !(read_paths.is_empty() && write_paths.is_empty() && discover_paths.is_empty());
                let landlock_support = world::landlock::detect_support();
                let landlock_supported = landlock_support.supported;

                if landlock_intended && landlock_supported {
                    let mut policy = world::landlock::LandlockFilesystemPolicy {
                        exec_paths: vec!["/".to_string(), "/project".to_string()],
                        discover_paths: Vec::new(),
                        read_paths: vec![
                            "/usr".to_string(),
                            "/bin".to_string(),
                            "/lib".to_string(),
                            "/lib64".to_string(),
                            "/etc".to_string(),
                            "/proc".to_string(),
                        ],
                        write_paths: vec![
                            "/tmp".to_string(),
                            "/dev".to_string(),
                            "/var/lib/substrate/world-deps".to_string(),
                        ],
                    };

                    if let Ok(project_dir) = std::env::var(MOUNT_PROJECT_DIR_ENV) {
                        if !project_dir.trim().is_empty() {
                            policy.exec_paths.push(project_dir);
                        }
                    }

                    policy.discover_paths.append(&mut discover_paths);
                    policy.read_paths.append(&mut read_paths);
                    policy.write_paths.append(&mut write_paths);

                    let mount_fs_mode =
                        std::env::var(MOUNT_FS_MODE_ENV).unwrap_or_else(|_| "writable".to_string());
                    let fs_mode_writable = !mount_fs_mode.trim().eq_ignore_ascii_case("read_only");

                    let derivation_required = fs_mode_writable;

                    if derivation_required {
                        let project_dir = std::env::var(MOUNT_PROJECT_DIR_ENV)
                            .ok()
                            .map(|raw| raw.trim().to_string())
                            .filter(|t| !t.is_empty());
                        let mount_point = project_dir
                            .clone()
                            .unwrap_or_else(|| "<missing>".to_string());

                        let Some(project_dir) = project_dir else {
                            eprintln!(
                                "substrate: error: full isolation landlock prerequisites missing: {}",
                                serde_json::to_string(&json!({
                                    "feature": "full-isolation-landlock-overlayfs-compat",
                                    "mount_point": mount_point,
                                    "reason": "missing or empty SUBSTRATE_MOUNT_PROJECT_DIR",
                                    "remediation": "this full-isolation exec requires deriving overlayfs backing dirs from /proc/self/mountinfo",
                                }))
                                .unwrap_or_else(|_| "<unserializable>".to_string())
                            );
                            std::process::exit(4);
                        };

                        if let Err(err) =
                            extend_with_overlayfs_backing_dirs_strict(&mut policy, &project_dir)
                        {
                            eprintln!(
                                "substrate: error: full isolation landlock prerequisites missing: {}",
                                serde_json::to_string(&json!({
                                    "feature": "full-isolation-landlock-overlayfs-compat",
                                    "mount_point": project_dir,
                                    "reason": err.to_string(),
                                    "remediation": "this full-isolation exec requires deriving overlayfs backing dirs from /proc/self/mountinfo",
                                }))
                                .unwrap_or_else(|_| "<unserializable>".to_string())
                            );
                            std::process::exit(4);
                        }
                    }

                    policy.discover_paths.sort();
                    policy.discover_paths.dedup();
                    policy.read_paths.sort();
                    policy.read_paths.dedup();
                    policy.write_paths.sort();
                    policy.write_paths.dedup();

                    let report = world::landlock::apply_filesystem_policy(&policy);
                    if report.attempted && !report.applied {
                        eprintln!(
                            "substrate: error: landlock apply failed: {}",
                            serde_json::to_string(&json!({
                                "supported": report.support.supported,
                                "abi": report.support.abi,
                                "attempted": report.attempted,
                                "applied": report.applied,
                                "rules_added": report.rules_added,
                                "reason": report.reason,
                            }))
                            .unwrap_or_else(|_| "<unserializable>".to_string())
                        );
                        std::process::exit(4);
                    }
                }
            } else {
                // Workspace isolation keeps host paths readable, but should prevent writes outside the
                // project and a few scratch locations.
                let mut write_paths = vec![
                    "/tmp".to_string(),
                    "/var/tmp".to_string(),
                    "/dev".to_string(),
                    "/var/lib/substrate/world-deps".to_string(),
                ];

                if let Ok(project_dir) = std::env::var(MOUNT_PROJECT_DIR_ENV) {
                    let trimmed = project_dir.trim();
                    if !trimmed.is_empty() {
                        write_paths.push(trimmed.to_string());
                    }
                }

                write_paths.sort();
                write_paths.dedup();

                let _report = world::landlock::apply_write_only_allowlist(&write_paths);
                let _ = _report;
            }

            if let Some(plan) = enforcement_plan.as_ref() {
                drop_caps_for_deny_enforcement(plan)?;
            }

            std::env::set_current_dir(&cwd)
                .with_context(|| format!("failed to set cwd to {cwd:?}"))?;

            let mut command = std::process::Command::new("sh");
            if login_shell {
                command.arg("-lc");
            } else {
                command.arg("-c");
            }
            command.arg(&cmd);

            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let err = command.exec();
                Err(anyhow::anyhow!("failed to exec inner command: {err}"))
            }

            #[cfg(not(unix))]
            {
                let status = command
                    .status()
                    .context("failed to run inner command under landlock exec wrapper")?;
                std::process::exit(status.code().unwrap_or(1));
            }
        };

        if cleanup_needed {
            let pid = unsafe { libc::fork() };
            if pid < 0 {
                return Err(std::io::Error::last_os_error()).context("fork");
            }
            if pid == 0 {
                // Child: apply Landlock + drop caps, then exec the workload.
                return apply_workload_restrictions(read_paths, write_paths, discover_paths);
            }

            // Parent: wait, then cleanup any synthetic deny mountpoints we created in the overlay.
            let mut status: libc::c_int = 0;
            let rc = unsafe { libc::waitpid(pid, &mut status as *mut _, 0) };
            if rc < 0 {
                return Err(std::io::Error::last_os_error()).context("waitpid");
            }

            if let Some(report) = deny_report.as_ref() {
                cleanup_deny_mountpoint_artifacts(report)?;
            }

            std::process::exit(wait_status_to_exit_code(status));
        }

        // Fast path: no synthetic mountpoints were created, so we can just exec in-place.
        apply_workload_restrictions(read_paths, write_paths, discover_paths)?;
        unreachable!("workload exec should not return");
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (
            &read_paths,
            &write_paths,
            &discover_paths,
            &enforcement_plan,
        );
    }

    #[cfg(not(target_os = "linux"))]
    let cwd = std::env::var(MOUNT_CWD_ENV).unwrap_or_else(|_| "/".to_string());
    #[cfg(not(target_os = "linux"))]
    std::env::set_current_dir(&cwd).with_context(|| format!("failed to set cwd to {cwd:?}"))?;

    #[cfg(not(target_os = "linux"))]
    let cmd = std::env::var(INNER_CMD_ENV).context("missing SUBSTRATE_INNER_CMD")?;
    #[cfg(not(target_os = "linux"))]
    let login_shell = std::env::var(INNER_LOGIN_SHELL_ENV)
        .ok()
        .is_some_and(|raw| raw.trim() == "1");

    #[cfg(not(target_os = "linux"))]
    let mut command = std::process::Command::new("sh");
    #[cfg(not(target_os = "linux"))]
    if login_shell {
        command.arg("-lc");
    } else {
        command.arg("-c");
    }
    #[cfg(not(target_os = "linux"))]
    command.arg(cmd);

    #[cfg(all(unix, not(target_os = "linux")))]
    {
        use std::os::unix::process::CommandExt;
        let err = command.exec();
        Err(anyhow::anyhow!("failed to exec inner command: {err}"))
    }

    #[cfg(all(not(unix), not(target_os = "linux")))]
    {
        let status = command
            .status()
            .context("failed to run inner command under landlock exec wrapper")?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn parse_allowlist_env(key: &str) -> Vec<String> {
    std::env::var(key)
        .ok()
        .map(|raw| {
            raw.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
#[derive(Debug, Default)]
struct DenyMaskApplyReport {
    /// Absolute paths to add to the Landlock write allowlist so write-deny mountpoints are still
    /// reachable (writes fail with EROFS from the read-only bind mount, not EACCES from Landlock).
    write_allowlist_paths: Vec<String>,
    /// Read-only bind-mount targets to unmount after the workload exits (only used when we had to
    /// create mountpoints to deny future writes under deny prefixes).
    cleanup_readonly_mount_targets: Vec<PathBuf>,
    /// Directories created to establish deny mountpoints (remove in reverse order after unmount).
    cleanup_created_dirs: Vec<PathBuf>,
}

#[cfg(target_os = "linux")]
fn wait_status_to_exit_code(status: libc::c_int) -> i32 {
    // Mirrors common shell semantics:
    // - exited normally: return the exit status
    // - terminated by signal: return 128 + signal
    let signal = status & 0x7f;
    if signal == 0 {
        (status >> 8) & 0xff
    } else {
        128 + signal
    }
}

#[cfg(target_os = "linux")]
fn cleanup_deny_mountpoint_artifacts(report: &DenyMaskApplyReport) -> Result<()> {
    fn umount_detach(target: &Path) -> Result<()> {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let target_c = CString::new(target.as_os_str().as_bytes())
            .with_context(|| format!("path contained NUL byte: {}", target.display()))?;
        let rc = unsafe { libc::umount2(target_c.as_ptr(), libc::MNT_DETACH) };
        if rc != 0 {
            let err = std::io::Error::last_os_error();
            // Ignore common cleanup races: ENOENT (already gone) or EINVAL (not a mountpoint).
            if matches!(
                err.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
            ) {
                return Ok(());
            }
            return Err(err).with_context(|| format!("umount2 {}", target.display()));
        }
        Ok(())
    }

    for target in &report.cleanup_readonly_mount_targets {
        umount_detach(target)?;
    }

    let mut dirs = report.cleanup_created_dirs.clone();
    dirs.sort_by(|a, b| {
        b.components()
            .count()
            .cmp(&a.components().count())
            .then_with(|| a.display().to_string().cmp(&b.display().to_string()))
    });
    for dir in dirs {
        match std::fs::remove_dir(&dir) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
            Err(err) => return Err(err).with_context(|| format!("rmdir {}", dir.display())),
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DenyMaskAction {
    AccessDenied,
    ReadOnly,
}

#[cfg(target_os = "linux")]
fn apply_deny_masks_linux(
    plan: &enforcement_plan::EnforcementPlanV1,
    mount_project_dir: &str,
) -> Result<DenyMaskApplyReport> {
    let project_root = Path::new("/project");
    let alt_root = Path::new(mount_project_dir);

    let entries = collect_project_entries_no_follow(project_root)
        .with_context(|| format!("scan project tree at {}", project_root.display()))?;

    let (access_denied, write_readonly) = resolve_deny_actions(
        &entries,
        &plan.read_deny,
        &plan.discover_deny,
        &plan.write_deny,
    )?;

    let mask_sources = ensure_deny_mask_sources()?;

    // Apply write-deny mounts first, then apply access-denied mounts so they can override.
    let mut report = DenyMaskApplyReport::default();
    apply_deny_mounts_for_root(
        alt_root,
        &write_readonly,
        DenyMaskAction::ReadOnly,
        &mask_sources,
        Some(&mut report),
    )?;
    apply_deny_mounts_for_root(
        project_root,
        &write_readonly,
        DenyMaskAction::ReadOnly,
        &mask_sources,
        Some(&mut report),
    )?;

    apply_deny_mounts_for_root(
        alt_root,
        &access_denied,
        DenyMaskAction::AccessDenied,
        &mask_sources,
        None,
    )?;
    apply_deny_mounts_for_root(
        project_root,
        &access_denied,
        DenyMaskAction::AccessDenied,
        &mask_sources,
        None,
    )?;

    report.write_allowlist_paths.sort();
    report.write_allowlist_paths.dedup();

    report
        .cleanup_readonly_mount_targets
        .sort_by_key(|a| a.display().to_string());
    report.cleanup_readonly_mount_targets.dedup();
    report
        .cleanup_created_dirs
        .sort_by_key(|a| a.display().to_string());
    report.cleanup_created_dirs.dedup();

    Ok(report)
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
struct ProjectEntry {
    rel: String,
    kind: EntryKind,
    create_if_missing: bool,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    File,
    Dir,
    Symlink,
    Other,
}

#[cfg(target_os = "linux")]
fn collect_project_entries_no_follow(root: &Path) -> Result<Vec<ProjectEntry>> {
    let mut out = Vec::new();
    walk_dir_no_follow(root, root, &mut out)?;
    Ok(out)
}

#[cfg(target_os = "linux")]
fn walk_dir_no_follow(root: &Path, dir: &Path, out: &mut Vec<ProjectEntry>) -> Result<()> {
    let rd =
        std::fs::read_dir(dir).with_context(|| format!("read_dir failed for {}", dir.display()))?;
    for entry in rd {
        let entry =
            entry.with_context(|| format!("read_dir entry failed for {}", dir.display()))?;
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path)
            .with_context(|| format!("symlink_metadata failed for {}", path.display()))?;
        let ft = meta.file_type();
        let kind = if ft.is_symlink() {
            EntryKind::Symlink
        } else if ft.is_dir() {
            EntryKind::Dir
        } else if ft.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };

        let rel = path
            .strip_prefix(root)
            .with_context(|| {
                format!(
                    "failed to strip_prefix({}, {})",
                    root.display(),
                    path.display()
                )
            })?
            .to_string_lossy()
            .to_string();
        if rel.is_empty() || rel == "." {
            continue;
        }

        out.push(ProjectEntry {
            rel: rel.clone(),
            kind,
            create_if_missing: false,
        });
        if kind == EntryKind::Dir {
            // Do not follow symlink dirs: symlink_metadata + kind check ensures we only recurse
            // into real directories.
            walk_dir_no_follow(root, &path, out)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn resolve_deny_actions(
    entries: &[ProjectEntry],
    read_deny: &[String],
    discover_deny: &[String],
    write_deny: &[String],
) -> Result<(Vec<ProjectEntry>, Vec<ProjectEntry>)> {
    use std::collections::{BTreeMap, BTreeSet};

    let mut by_rel: BTreeMap<&str, EntryKind> = BTreeMap::new();
    for entry in entries {
        by_rel.insert(entry.rel.as_str(), entry.kind);
    }

    let mut access_denied_rels: BTreeSet<String> = BTreeSet::new();
    let mut write_readonly_rels: BTreeSet<String> = BTreeSet::new();
    let mut write_readonly_create_if_missing_rels: BTreeSet<String> = BTreeSet::new();

    for raw in read_deny {
        add_denies(raw, &by_rel, &mut access_denied_rels)?;
    }
    for raw in discover_deny {
        add_denies(raw, &by_rel, &mut access_denied_rels)?;
    }
    for raw in write_deny {
        add_denies(raw, &by_rel, &mut write_readonly_rels)?;
        if let Some(prefix) = write_deny_recursive_prefix_dir(raw)? {
            write_readonly_rels.insert(prefix.clone());
            if !by_rel.contains_key(prefix.as_str()) {
                write_readonly_create_if_missing_rels.insert(prefix);
            }
        }
    }

    // AccessDenied wins over ReadOnly for the same path.
    for rel in &access_denied_rels {
        write_readonly_rels.remove(rel);
    }

    let access_denied = collapse_descendants(project_entries_from_rels(
        &by_rel,
        &access_denied_rels,
        None,
    ));
    let write_readonly = collapse_descendants(project_entries_from_rels(
        &by_rel,
        &write_readonly_rels,
        Some(&write_readonly_create_if_missing_rels),
    ));

    Ok((access_denied, write_readonly))
}

#[cfg(target_os = "linux")]
fn add_denies(
    raw_pattern: &str,
    entries_by_rel: &std::collections::BTreeMap<&str, EntryKind>,
    out_rels: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    let normalized = normalize_project_pattern(raw_pattern)
        .with_context(|| format!("invalid deny pattern {raw_pattern:?}"))?;

    // `.` means the entire project.
    if normalized == "." {
        out_rels.insert(".".to_string());
        return Ok(());
    }

    if !normalized.contains('*') {
        out_rels.insert(normalized);
        return Ok(());
    }

    // Wildcard snapshot semantics: enumerate matches that exist at exec start.
    for rel in entries_by_rel.keys() {
        if wildcard_match(&normalized, rel) {
            out_rels.insert((*rel).to_string());
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn project_entries_from_rels(
    entries_by_rel: &std::collections::BTreeMap<&str, EntryKind>,
    rels: &std::collections::BTreeSet<String>,
    create_if_missing_rels: Option<&std::collections::BTreeSet<String>>,
) -> Vec<ProjectEntry> {
    rels.iter()
        .map(|rel| {
            let kind = entries_by_rel
                .get(rel.as_str())
                .copied()
                .unwrap_or_else(|| {
                    if create_if_missing_rels.is_some_and(|rels| rels.contains(rel.as_str())) {
                        EntryKind::Dir
                    } else {
                        EntryKind::Other
                    }
                });
            ProjectEntry {
                rel: rel.clone(),
                kind,
                create_if_missing: create_if_missing_rels
                    .is_some_and(|rels| rels.contains(rel.as_str())),
            }
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn write_deny_recursive_prefix_dir(raw_pattern: &str) -> Result<Option<String>> {
    let normalized = normalize_project_pattern(raw_pattern)
        .with_context(|| format!("invalid deny pattern {raw_pattern:?}"))?;

    // For recursive directory denies (`foo/bar/**`), deny must prevent new path creation under the
    // prefix even when it doesn't exist at exec start (e.g. `mkdir -p foo/bar/x`).
    let Some(prefix) = normalized.strip_suffix("/**") else {
        return Ok(None);
    };

    let prefix = prefix.trim_matches('/');
    if prefix.is_empty() || prefix == "." {
        return Ok(None);
    }

    // Defensive: do not attempt to synthesize mountpoints for patterns where the prefix itself
    // contains wildcards.
    if prefix.contains('*') {
        return Ok(None);
    }

    Ok(Some(prefix.to_string()))
}

#[cfg(target_os = "linux")]
fn collapse_descendants(mut entries: Vec<ProjectEntry>) -> Vec<ProjectEntry> {
    use std::collections::BTreeSet;

    entries.sort_by(|a, b| {
        a.rel
            .len()
            .cmp(&b.rel.len())
            .then_with(|| a.rel.cmp(&b.rel))
    });

    let mut denied_dirs: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<ProjectEntry> = Vec::new();

    for entry in entries {
        if entry.rel == "." {
            out.push(entry);
            denied_dirs.insert(".".to_string());
            continue;
        }

        if has_denied_ancestor(&denied_dirs, &entry.rel) {
            continue;
        }

        if entry.kind == EntryKind::Dir {
            denied_dirs.insert(entry.rel.clone());
        }

        out.push(entry);
    }

    out
}

#[cfg(target_os = "linux")]
fn has_denied_ancestor(denied_dirs: &std::collections::BTreeSet<String>, rel: &str) -> bool {
    if denied_dirs.contains(".") {
        return true;
    }
    let mut cur = rel;
    while let Some((parent, _)) = cur.rsplit_once('/') {
        if denied_dirs.contains(parent) {
            return true;
        }
        cur = parent;
    }
    false
}

#[cfg(target_os = "linux")]
fn wildcard_match(pattern: &str, rel: &str) -> bool {
    if pattern == "." {
        return true;
    }
    if rel == "." {
        return pattern == ".";
    }
    let p_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let r_segs: Vec<&str> = rel.split('/').filter(|s| !s.is_empty()).collect();
    wildcard_match_segments(&p_segs, &r_segs)
}

#[cfg(target_os = "linux")]
fn wildcard_match_segments(pattern: &[&str], rel: &[&str]) -> bool {
    fn seg_match(pat: &str, text: &str) -> bool {
        // Simple glob: '*' matches within a segment.
        let mut p = 0usize;
        let mut t = 0usize;
        let pat_bytes = pat.as_bytes();
        let text_bytes = text.as_bytes();
        let mut star: Option<usize> = None;
        let mut star_text = 0usize;

        while t < text_bytes.len() {
            if p < pat_bytes.len() && pat_bytes[p] == b'*' {
                star = Some(p);
                star_text = t;
                p += 1;
                continue;
            }
            if p < pat_bytes.len() && pat_bytes[p] == text_bytes[t] {
                p += 1;
                t += 1;
                continue;
            }
            if let Some(star_p) = star {
                p = star_p + 1;
                star_text += 1;
                t = star_text;
                continue;
            }
            return false;
        }

        while p < pat_bytes.len() && pat_bytes[p] == b'*' {
            p += 1;
        }

        p == pat_bytes.len()
    }

    fn rec(pat: &[&str], rel: &[&str], i: usize, j: usize) -> bool {
        if i == pat.len() {
            return j == rel.len();
        }
        if pat[i] == "**" {
            // `**` matches zero or more segments.
            if rec(pat, rel, i + 1, j) {
                return true;
            }
            if j < rel.len() {
                return rec(pat, rel, i, j + 1);
            }
            return false;
        }
        if j >= rel.len() {
            return false;
        }
        if !seg_match(pat[i], rel[j]) {
            return false;
        }
        rec(pat, rel, i + 1, j + 1)
    }

    rec(pattern, rel, 0, 0)
}

#[cfg(target_os = "linux")]
fn normalize_project_pattern(raw: &str) -> Result<String> {
    let mut pattern = raw.trim();
    if pattern.is_empty() {
        anyhow::bail!("pattern must be non-empty");
    }
    if pattern.starts_with('/') {
        anyhow::bail!("absolute paths are not allowed");
    }

    while let Some(stripped) = pattern.strip_prefix("./") {
        pattern = stripped;
    }

    let mut normalized = pattern.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        normalized = ".".to_string();
    }

    if normalized.split('/').any(|segment| segment == "..") {
        anyhow::bail!("path segments must not be '..'");
    }

    Ok(normalized)
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct DenyMaskSources {
    /// A directory used as a bind-mount source for access-denied masks.
    ///
    /// This path is intentionally placed outside the Landlock allowlists so that attempts to
    /// traverse/list it yield `EACCES` (and not "empty directory" semantics).
    deny_dir: PathBuf,
    /// A file used as a bind-mount source for access-denied masks.
    ///
    /// This path is intentionally placed outside the Landlock allowlists so that attempts to read
    /// it yield `EACCES` (and not "empty file" semantics).
    deny_file: PathBuf,
}

#[cfg(target_os = "linux")]
fn ensure_deny_mask_sources() -> Result<DenyMaskSources> {
    fn enforce_mode_000(path: &Path) -> Result<()> {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o000))
            .with_context(|| format!("chmod 000 {}", path.display()))?;
        let mode = std::fs::metadata(path)
            .with_context(|| format!("stat {}", path.display()))?
            .permissions()
            .mode()
            & 0o777;
        if mode != 0 {
            anyhow::bail!(
                "deny mask source {} has mode {:03o}, expected 000",
                path.display(),
                mode
            );
        }
        Ok(())
    }

    // Keep these sources out of the default Landlock allowlists (which include `/tmp`) so that
    // access-denied masks deterministically fail with `EACCES` even when `/project` is broadly
    // allowed (e.g., allow_list += '.').
    let root = PathBuf::from("/var/lib/substrate/deny-mask");
    std::fs::create_dir_all(&root).context("create deny mask root")?;

    let deny_dir = root.join("deny_dir");
    std::fs::create_dir_all(&deny_dir).context("create deny_dir source")?;
    enforce_mode_000(&deny_dir)?;

    let deny_file = root.join("deny_file");
    if !deny_file.exists() {
        std::fs::write(&deny_file, []).context("create deny_file source")?;
    }
    enforce_mode_000(&deny_file)?;

    Ok(DenyMaskSources {
        deny_dir,
        deny_file,
    })
}

#[cfg(target_os = "linux")]
fn drop_caps_for_deny_enforcement(plan: &enforcement_plan::EnforcementPlanV1) -> Result<()> {
    #[repr(C)]
    struct CapHeader {
        version: u32,
        pid: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CapData {
        effective: u32,
        permitted: u32,
        inheritable: u32,
    }

    // From linux/capability.h
    const LINUX_CAPABILITY_VERSION_3: u32 = 0x20080522;
    const CAP_DAC_OVERRIDE: u32 = 1;
    const CAP_DAC_READ_SEARCH: u32 = 2;
    const CAP_SYS_ADMIN: u32 = 21;

    // Glibc exposes capget/capset wrappers; the libc crate doesn't currently surface them.
    extern "C" {
        fn capget(hdrp: *mut CapHeader, datap: *mut CapData) -> libc::c_int;
        fn capset(hdrp: *mut CapHeader, datap: *const CapData) -> libc::c_int;
    }

    fn clear_ambient_caps() {
        const PR_CAP_AMBIENT: libc::c_int = 47;
        const PR_CAP_AMBIENT_CLEAR_ALL: libc::c_ulong = 4;
        unsafe {
            let _ = libc::prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_CLEAR_ALL, 0, 0, 0);
        }
    }

    fn drop_caps(caps: &[u32]) -> Result<()> {
        let mut header = CapHeader {
            version: LINUX_CAPABILITY_VERSION_3,
            pid: 0,
        };
        let mut data = [
            CapData {
                effective: 0,
                permitted: 0,
                inheritable: 0,
            },
            CapData {
                effective: 0,
                permitted: 0,
                inheritable: 0,
            },
        ];

        let rc = unsafe { capget(&mut header, data.as_mut_ptr()) };
        if rc != 0 {
            return Err(std::io::Error::last_os_error()).context("capget");
        }

        for cap in caps {
            let idx = (cap / 32) as usize;
            let bit = 1u32 << (cap % 32);
            if idx < data.len() {
                data[idx].effective &= !bit;
                data[idx].permitted &= !bit;
                data[idx].inheritable &= !bit;
            }
        }

        let rc = unsafe { capset(&mut header, data.as_ptr()) };
        if rc != 0 {
            return Err(std::io::Error::last_os_error()).context("capset");
        }

        Ok(())
    }

    // Ensure deny-masked paths produce deterministic EACCES for the workload (even though the
    // workload runs as uid=0 inside the mount namespace) by dropping DAC bypass capabilities.
    drop_caps(&[CAP_DAC_OVERRIDE, CAP_DAC_READ_SEARCH])?;

    if plan.enforcement == enforcement_plan::EnforcementPlanModeV1::Strict {
        // Strict-mode safety: do not allow the workload to undo deny masks via mount syscalls.
        drop_caps(&[CAP_SYS_ADMIN])?;
    }

    clear_ambient_caps();
    Ok(())
}

#[cfg(target_os = "linux")]
fn apply_deny_mounts_for_root(
    root: &Path,
    entries: &[ProjectEntry],
    action: DenyMaskAction,
    sources: &DenyMaskSources,
    report: Option<&mut DenyMaskApplyReport>,
) -> Result<()> {
    let mut report = report;
    for entry in entries {
        if entry.rel == "." {
            apply_action_for_target(root, root, EntryKind::Dir, action, sources, &mut report)?;
            continue;
        }

        let target = root.join(&entry.rel);
        let meta = match std::fs::symlink_metadata(&target) {
            Ok(m) => Some(m),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => return Err(err).with_context(|| format!("stat {}", target.display())),
        };

        let mut created = Vec::new();
        if meta.is_none() && entry.create_if_missing && action == DenyMaskAction::ReadOnly {
            created = create_dir_all_tracking(&target)
                .with_context(|| format!("mkdir -p {}", target.display()))?;
        }

        let meta = match meta {
            Some(m) => Some(m),
            None => std::fs::symlink_metadata(&target).ok(),
        };
        let Some(meta) = meta else { continue };
        let ft = meta.file_type();
        let kind = if ft.is_dir() {
            EntryKind::Dir
        } else if ft.is_file() {
            EntryKind::File
        } else if ft.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::Other
        };

        apply_action_for_target(root, &target, kind, action, sources, &mut report)?;
        if entry.create_if_missing && action == DenyMaskAction::ReadOnly {
            if let Some(report) = report.as_deref_mut() {
                report.cleanup_readonly_mount_targets.push(target.clone());
                if !created.is_empty() {
                    report.cleanup_created_dirs.extend(created);
                }
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn create_dir_all_tracking(target: &Path) -> Result<Vec<PathBuf>> {
    let mut created = Vec::new();
    if target.exists() {
        return Ok(created);
    }

    let mut ancestor = target.to_path_buf();
    while !ancestor.exists() {
        let Some(parent) = ancestor.parent() else {
            break;
        };
        ancestor = parent.to_path_buf();
    }

    let rel = target
        .strip_prefix(&ancestor)
        .with_context(|| format!("strip_prefix({}, {})", ancestor.display(), target.display()))?;

    let mut cur = ancestor;
    for component in rel.components() {
        cur.push(component);
        if cur.exists() {
            continue;
        }
        match std::fs::create_dir(&cur) {
            Ok(()) => created.push(cur.clone()),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err).with_context(|| format!("create_dir {}", cur.display())),
        }
    }

    Ok(created)
}

#[cfg(target_os = "linux")]
fn apply_action_for_target(
    _root: &Path,
    target: &Path,
    kind: EntryKind,
    action: DenyMaskAction,
    sources: &DenyMaskSources,
    report: &mut Option<&mut DenyMaskApplyReport>,
) -> Result<()> {
    match action {
        DenyMaskAction::AccessDenied => match kind {
            EntryKind::Dir => bind_mount(&sources.deny_dir, target)
                .with_context(|| format!("bind-mount deny_dir -> {}", target.display()))?,
            EntryKind::File | EntryKind::Symlink | EntryKind::Other => {
                bind_mount(&sources.deny_file, target)
                    .with_context(|| format!("bind-mount deny_file -> {}", target.display()))?
            }
        },
        DenyMaskAction::ReadOnly => {
            bind_mount(target, target)
                .with_context(|| format!("bind-mount self -> {}", target.display()))?;
            remount_read_only(target)
                .with_context(|| format!("remount ro {}", target.display()))?;
            if let Some(report) = report.as_deref_mut() {
                report
                    .write_allowlist_paths
                    .push(target.display().to_string());
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn bind_mount(src: &Path, dst: &Path) -> Result<()> {
    mount_syscall(Some(src), dst, libc::MS_BIND, None)
}

#[cfg(target_os = "linux")]
fn remount_read_only(dst: &Path) -> Result<()> {
    mount_syscall(
        None::<&Path>,
        dst,
        libc::MS_BIND | libc::MS_REMOUNT | libc::MS_RDONLY,
        None,
    )
}

#[cfg(target_os = "linux")]
fn mount_syscall(
    src: Option<&Path>,
    dst: &Path,
    flags: libc::c_ulong,
    data: Option<&str>,
) -> Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let dst_c = CString::new(dst.as_os_str().as_bytes())
        .map_err(|e| anyhow::anyhow!("invalid mount target {}: {e}", dst.display()))?;
    let src_c = match src {
        Some(p) => Some(
            CString::new(p.as_os_str().as_bytes())
                .map_err(|e| anyhow::anyhow!("invalid mount source {}: {e}", p.display()))?,
        ),
        None => None,
    };
    let data_c = match data {
        Some(d) => Some(CString::new(d).map_err(|e| anyhow::anyhow!("invalid mount data: {e}"))?),
        None => None,
    };

    let src_ptr = src_c
        .as_ref()
        .map(|c| c.as_ptr())
        .unwrap_or(std::ptr::null());
    let data_ptr = data_c
        .as_ref()
        .map(|c| c.as_ptr() as *const libc::c_void)
        .unwrap_or(std::ptr::null());

    let rc = unsafe { libc::mount(src_ptr, dst_c.as_ptr(), std::ptr::null(), flags, data_ptr) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).with_context(|| {
            format!(
                "mount syscall failed (dst={}, flags=0x{:x})",
                dst.display(),
                flags
            )
        });
    }
    Ok(())
}
