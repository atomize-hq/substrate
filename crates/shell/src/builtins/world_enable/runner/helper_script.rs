//! Helper script execution for `substrate world enable`.

use super::log_ops::append_log_line;
use crate::WorldEnableArgs;
use anyhow::{anyhow, bail, Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub(super) fn run_helper_script(
    script: &Path,
    args: &WorldEnableArgs,
    prefix: &Path,
    log_path: &Path,
    socket_override: Option<&Path>,
) -> Result<()> {
    append_log_line(
        log_path,
        &format!(
            "running helper {} (dry_run={}, verbose={}, force={})",
            script.display(),
            args.dry_run,
            args.verbose,
            args.force
        ),
    )?;

    let mut cmd = Command::new(script);
    cmd.arg("--prefix").arg(prefix);
    cmd.arg("--profile").arg(&args.profile);
    if args.dry_run {
        cmd.arg("--dry-run");
    }
    if args.verbose {
        cmd.arg("--verbose");
    }
    if args.force {
        cmd.arg("--force");
    }
    if let Some(socket_path) = socket_override {
        cmd.env("SUBSTRATE_WORLD_SOCKET", socket_path);
    }

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to launch {}", script.display()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("helper stdout missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("helper stderr missing"))?;

    let log_file = OpenOptions::new()
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open {}", log_path.display()))?;
    let log = Arc::new(Mutex::new(log_file));

    let threads = vec![
        stream_reader(stdout, "stdout", Arc::clone(&log), args.verbose),
        stream_reader(stderr, "stderr", Arc::clone(&log), args.verbose),
    ];

    let status = child
        .wait()
        .with_context(|| format!("failed to wait on {}", script.display()))?;
    for handle in threads {
        handle.join().unwrap()?;
    }

    if !status.success() {
        bail!(
            "world enable helper exited with status {}. See {} for details, then run 'substrate world doctor --json' for diagnostics.",
            status,
            log_path.display()
        );
    }

    Ok(())
}

fn stream_reader<R>(
    reader: R,
    label: &'static str,
    log: Arc<Mutex<File>>,
    echo: bool,
) -> thread::JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = buf_reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            {
                let mut file = log.lock().unwrap();
                write!(file, "[{}] {}", label, line)?;
                file.flush().ok();
            }
            if echo {
                if label == "stderr" {
                    eprint!("[{}] {}", label, line);
                } else {
                    print!("[{}] {}", label, line);
                }
            }
        }
        Ok(())
    })
}
