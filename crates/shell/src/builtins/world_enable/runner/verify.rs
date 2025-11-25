//! Verification helpers for world enable.

use super::log_ops::{append_log_line, write_world_doctor_output};
use anyhow::{bail, Context, Result};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub(super) fn verify_world_health(
    log_path: &Path,
    timeout: Duration,
    verbose: bool,
    socket_path: Option<&Path>,
) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        wait_for_socket(socket_path, timeout, log_path)?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (timeout, socket_path);
    }
    run_world_doctor(log_path, verbose)
}

#[cfg(target_os = "linux")]
fn wait_for_socket(
    socket_override: Option<&Path>,
    timeout: Duration,
    log_path: &Path,
) -> Result<()> {
    use std::thread;
    use std::time::Instant;

    let socket_path = socket_override.unwrap_or_else(|| Path::new("/run/substrate.sock"));
    let deadline = Instant::now() + timeout;
    while Instant::now() <= deadline {
        if socket_path.exists() {
            append_log_line(
                log_path,
                &format!("socket: {} detected", socket_path.display()),
            )?;
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    append_log_line(
        log_path,
        &format!("socket: timeout waiting for {}", socket_path.display()),
    )?;
    bail!(
        "Timed out waiting for {} after {} seconds. See {} for logs, then run 'substrate world doctor --json' to inspect the backend.",
        socket_path.display(),
        timeout.as_secs(),
        log_path.display()
    )
}

fn run_world_doctor(log_path: &Path, verbose: bool) -> Result<()> {
    if env::var("SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR")
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE"))
        .unwrap_or(false)
    {
        append_log_line(
            log_path,
            "skipping: substrate world doctor --json (SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR=1)",
        )?;
        return Ok(());
    }
    append_log_line(log_path, "running: substrate world doctor --json")?;
    let exe = env::current_exe().context("failed to locate current executable")?;
    let output = Command::new(exe)
        .arg("world")
        .arg("doctor")
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run 'substrate world doctor --json'")?;

    write_world_doctor_output(log_path, "stdout", &output.stdout, verbose)?;
    write_world_doctor_output(log_path, "stderr", &output.stderr, verbose)?;

    if !output.status.success() {
        bail!(
            "'substrate world doctor --json' failed (status {}). Review {} for details.",
            output.status,
            log_path.display()
        );
    }
    Ok(())
}
