//! Consume the JSONL event stream (`codex exec --json`) and print turn/item events.
//! Events include thread + turn lifecycle plus item created/updated variants such as
//! `agent_message`, `reasoning`, `command_execution`, `file_change`, and `mcp_tool_call`
//! (with thread/turn IDs and status).
//!
//! Flags:
//! - `--sample` to replay demo events from `crates/codex/examples/fixtures/streaming.jsonl` without
//!   invoking Codex (useful when the binary is absent).
//! - Otherwise, ensure `CODEX_BINARY` or a `codex` binary is on PATH.
//!
//! Example:
//! ```bash
//! cargo run -p unified-agent-api-codex --example stream_events -- "Summarize repo status"
//! cargo run -p unified-agent-api-codex --example stream_events -- --sample
//! ```

use std::{env, error::Error, path::Path, path::PathBuf, time::Duration};

#[path = "support/fixtures.rs"]
mod fixtures;

use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time,
};

#[derive(Debug, Deserialize)]
struct StreamEvent {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    thread_id: Option<String>,
    #[serde(default)]
    turn_id: Option<String>,
    #[serde(default)]
    item: Option<StreamItem>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamItem {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let use_sample = take_flag(&mut args, "--sample");
    let prompt = if args.is_empty() {
        "Summarize this repository".to_string()
    } else {
        args.join(" ")
    };

    if use_sample {
        println!(
            "Replaying streaming fixture from {} (pass a prompt to hit a real binary)...",
            fixtures::STREAMING_FIXTURE_PATH
        );
        for line in fixtures::streaming_events() {
            handle_line(line);
        }
        return Ok(());
    }

    let binary = resolve_binary();
    if !binary_exists(&binary) {
        eprintln!(
            "codex binary not found at {}. Set CODEX_BINARY or use --sample.",
            binary.display()
        );
        for line in fixtures::streaming_events() {
            handle_line(line);
        }
        return Ok(());
    }

    stream_from_codex(&binary, &prompt).await
}

async fn stream_from_codex(binary: &Path, prompt: &str) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new(binary);
    command
        .args(["exec", "--json", "--skip-git-repo-check"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let mut child = command.spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.shutdown().await?;
    }

    let mut stdout_lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut stderr_lines = BufReader::new(child.stderr.take().unwrap()).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_lines.next_line().await {
            eprintln!("[codex stderr] {line}");
        }
    });

    let idle_timeout = Duration::from_secs(30);
    loop {
        let maybe_line = time::timeout(idle_timeout, stdout_lines.next_line()).await;
        match maybe_line {
            Ok(Ok(Some(line))) => handle_line(&line),
            Ok(Ok(None)) => break,
            Ok(Err(error)) => {
                eprintln!("Failed to read codex output: {error}");
                break;
            }
            Err(_) => {
                eprintln!("No events within {idle_timeout:?}; treating as idle timeout.");
                break;
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        eprintln!("codex exited with {status}");
    }

    Ok(())
}

fn handle_line(line: &str) {
    match serde_json::from_str::<StreamEvent>(line) {
        Ok(event) => print_event(event),
        Err(_) => println!("(unparsed) {line}"),
    }
}

fn print_event(event: StreamEvent) {
    match event.kind.as_str() {
        "thread.started" => println!(
            "Thread started: {}",
            event.thread_id.as_deref().unwrap_or("-")
        ),
        "turn.started" => {
            let turn_id = event.turn_id.as_deref().unwrap_or("-");
            if let Some(thread_id) = event.thread_id.as_deref() {
                println!("Turn started: {turn_id} (thread {thread_id})");
            } else {
                println!("Turn started: {turn_id}");
            }
        }
        "turn.completed" => {
            let turn_id = event.turn_id.as_deref().unwrap_or("-");
            let suffix = event
                .thread_id
                .as_deref()
                .map(|thread_id| format!(" (thread {thread_id})"))
                .unwrap_or_default();
            println!("Turn completed: {turn_id}{suffix}");
        }
        "turn.failed" => {
            let turn_id = event.turn_id.as_deref().unwrap_or("-");
            let message = event.message.as_deref().unwrap_or("Unknown failure");
            println!("Turn failed: {turn_id} — {message}");
        }
        kind if kind.starts_with("item.") => {
            if let Some(item) = event.item {
                let body = item
                    .content
                    .as_deref()
                    .unwrap_or("(no content provided by Codex)");
                let status = item
                    .status
                    .as_deref()
                    .map(|value| format!(" [{value}]"))
                    .unwrap_or_default();
                println!(
                    "Item {}: {} — {body}{}",
                    item.kind,
                    item.id.unwrap_or_default(),
                    status
                );
            } else {
                println!("Item event ({kind})");
            }
        }
        "error" => {
            let message = event
                .error
                .or(event.message)
                .unwrap_or_else(|| "Unknown error".to_string());
            println!("Error event: {message}");
        }
        other => {
            if let Some(message) = event.message.as_deref() {
                println!("Event: {other} — {message}");
            } else {
                println!("Event: {other}");
            }
        }
    }
}

fn resolve_binary() -> PathBuf {
    env::var_os("CODEX_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"))
}

fn binary_exists(path: &Path) -> bool {
    if path.is_absolute() || path.components().count() > 1 {
        std::fs::metadata(path).is_ok()
    } else {
        env::var_os("PATH")
            .and_then(|paths| {
                env::split_paths(&paths)
                    .map(|dir| dir.join(path))
                    .find(|candidate| std::fs::metadata(candidate).is_ok())
            })
            .is_some()
    }
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|value| value != flag);
    before != args.len()
}
