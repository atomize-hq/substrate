use anyhow::{Context, Result};

pub const LANDLOCK_EXEC_ARG: &str = "__substrate_world_landlock_exec";

const INNER_CMD_ENV: &str = "SUBSTRATE_INNER_CMD";
const INNER_LOGIN_SHELL_ENV: &str = "SUBSTRATE_INNER_LOGIN_SHELL";
const MOUNT_CWD_ENV: &str = "SUBSTRATE_MOUNT_CWD";

const LANDLOCK_READ_ENV: &str = "SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST";
const LANDLOCK_WRITE_ENV: &str = "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST";

pub fn run_landlock_exec() -> Result<()> {
    let read_paths = parse_allowlist_env(LANDLOCK_READ_ENV);
    let write_paths = parse_allowlist_env(LANDLOCK_WRITE_ENV);

    #[cfg(target_os = "linux")]
    {
        let _report = world::landlock::apply_path_allowlists(&read_paths, &write_paths);
        let _ = _report;
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
