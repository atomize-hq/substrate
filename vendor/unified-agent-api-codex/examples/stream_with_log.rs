//! Stream Codex JSON output while teeing every line to a log file.
//!
//! This is a lightweight stand-in for a built-in log tee option: it mirrors Codex stdout to the
//! console and appends the same lines to `CODEX_LOG_PATH` (default: `codex-stream.log`).
//! Use `--sample` to avoid spawning Codex and log the demo events from
//! `crates/codex/examples/fixtures/streaming.jsonl` instead.
//!
//! Example:
//! ```bash
//! CODEX_LOG_PATH=/tmp/codex.log \
//!   cargo run -p unified-agent-api-codex --example stream_with_log -- "Stream with logging"
//! cargo run -p unified-agent-api-codex --example stream_with_log -- --sample
//! ```

use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[path = "support/fixtures.rs"]
mod fixtures;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let use_sample = take_flag(&mut args, "--sample");
    let prompt = if args.is_empty() {
        "Show a streaming log tee".to_string()
    } else {
        args.join(" ")
    };

    let log_path = log_path();
    prepare_log_dir(&log_path)?;

    let binary = resolve_binary();
    if use_sample || !binary_exists(&binary) {
        eprintln!(
            "Using sample events from {}; set CODEX_BINARY and drop --sample to stream from the real binary.",
            fixtures::STREAMING_FIXTURE_PATH
        );
        append_sample_events(&log_path)?;
        println!("Log written to {}", log_path.display());
        return Ok(());
    }

    stream_and_log(&binary, &prompt, &log_path).await?;
    println!("Stream captured in {}", log_path.display());
    Ok(())
}

async fn stream_and_log(
    binary: &Path,
    prompt: &str,
    log_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let mut command = Command::new(binary);
    command
        .args(["exec", "--json", "--skip-git-repo-check"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true);

    let mut child = command.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.shutdown().await?;
    }

    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    while let Some(line) = lines.next_line().await? {
        println!("{line}");
        writeln!(log_file, "{line}")?;
    }

    let status = child.wait().await?;
    if !status.success() {
        eprintln!("codex exited with {status}");
    }

    Ok(())
}

fn log_path() -> PathBuf {
    env::var_os("CODEX_LOG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex-stream.log"))
}

fn prepare_log_dir(path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn append_sample_events(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for line in fixtures::streaming_events() {
        writeln!(file, "{line}")?;
        println!("{line}");
    }
    Ok(())
}

fn resolve_binary() -> PathBuf {
    env::var_os("CODEX_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"))
}

fn binary_exists(path: &Path) -> bool {
    if path.is_absolute() || path.components().count() > 1 {
        fs::metadata(path).is_ok()
    } else {
        env::var_os("PATH")
            .and_then(|paths| {
                env::split_paths(&paths)
                    .map(|dir| dir.join(path))
                    .find(|candidate| fs::metadata(candidate).is_ok())
            })
            .is_some()
    }
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|value| value != flag);
    before != args.len()
}
