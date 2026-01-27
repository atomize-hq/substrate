use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use serde_json::json;

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

pub fn run_landlock_exec() -> Result<()> {
    let read_paths = parse_allowlist_env(LANDLOCK_READ_ENV);
    let write_paths = parse_allowlist_env(LANDLOCK_WRITE_ENV);

    #[cfg(target_os = "linux")]
    {
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

        if isolation_full {
            let landlock_intended = !(read_paths.is_empty() && write_paths.is_empty());
            let landlock_support = world::landlock::detect_support();
            let landlock_supported = landlock_support.supported;

            if landlock_intended && landlock_supported {
                let mut read_paths = read_paths;
                let mut write_paths = write_paths;

                let mut policy = world::landlock::LandlockFilesystemPolicy {
                    exec_paths: vec!["/".to_string(), "/project".to_string()],
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
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (&read_paths, &write_paths);
    }

    let cwd = std::env::var(MOUNT_CWD_ENV).unwrap_or_else(|_| "/".to_string());
    std::env::set_current_dir(&cwd).with_context(|| format!("failed to set cwd to {cwd:?}"))?;

    let cmd = std::env::var(INNER_CMD_ENV).context("missing SUBSTRATE_INNER_CMD")?;
    let login_shell = std::env::var(INNER_LOGIN_SHELL_ENV)
        .ok()
        .is_some_and(|raw| raw.trim() == "1");

    let mut command = std::process::Command::new("sh");
    if login_shell {
        command.arg("-lc");
    } else {
        command.arg("-c");
    }
    command.arg(cmd);

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
