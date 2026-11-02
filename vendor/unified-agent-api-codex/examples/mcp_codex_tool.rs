//! Start `codex mcp-server` over stdio and issue a `codex` tool call.
//!
//! Requirements:
//! - `CODEX_BINARY` (optional) to point at the Codex CLI.
//! - `CODEX_HOME` (optional) to isolate state for the MCP process.
//! - Pass `--sample` to view mocked notifications without spawning Codex.
//!
//! Example:
//! ```bash
//! cargo run -p unified-agent-api-codex --example mcp_codex_tool -- "Summarize repo status"
//! cargo run -p unified-agent-api-codex --example mcp_codex_tool -- --sample
//! ```

use std::{env, error::Error, path::Path, path::PathBuf};

use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time::{self, Duration},
};

const SAMPLE_NOTIFICATIONS: &[&str] = &[
    r#"{"jsonrpc":"2.0","method":"codex/event","params":{"type":"approval_required","kind":"exec","prompt":"Summarize repo status","sandbox":true,"thread_id":"demo-thread","turn_id":"turn-1"}}"#,
    r#"{"jsonrpc":"2.0","method":"codex/event","params":{"type":"task_complete","message":"Summary ready","turn_id":"turn-1","thread_id":"demo-thread"}}"#,
];

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
        print_sample_flow();
        return Ok(());
    }

    let binary = resolve_binary();
    if !binary_exists(&binary) {
        eprintln!(
            "codex binary not found at {}. Set CODEX_BINARY or use --sample.",
            binary.display()
        );
        print_sample_flow();
        return Ok(());
    }

    demo_mcp_server(&binary, &prompt).await?;
    Ok(())
}

async fn demo_mcp_server(binary: &Path, prompt: &str) -> Result<(), Box<dyn Error>> {
    println!("Starting `codex mcp-server` using {}", binary.display());

    let mut command = Command::new(binary);
    command
        .arg("mcp-server")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true);

    let mut child = command.spawn()?;
    let mut stdin = child.stdin.take().ok_or("stdin unavailable")?;
    let mut stdout = BufReader::new(child.stdout.take().ok_or("stdout unavailable")?).lines();

    // Send a minimal codex tool call. Real deployments may need initialize/notifications plumbing.
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "codex",
            "arguments": {
                "prompt": prompt,
                "cwd": env::current_dir().unwrap_or_default(),
                "model": "gpt-5.1-codex",
                "sandbox": "read-only",
                "approval-policy": "never"
            }
        },
    });

    stdin.write_all(request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read a couple of notifications then exit.
    let mut seen = 0;
    while seen < 4 {
        let next = time::timeout(Duration::from_secs(5), stdout.next_line()).await;
        match next {
            Ok(Ok(Some(line))) => {
                seen += 1;
                println!("[notification] {line}");
            }
            Ok(Ok(None)) => break,
            Ok(Err(error)) => {
                eprintln!("Failed to read MCP output: {error}");
                break;
            }
            Err(_) => {
                eprintln!("Timed out waiting for MCP notification");
                break;
            }
        }
    }

    let _ = child.kill().await;
    Ok(())
}

fn print_sample_flow() {
    println!("Sample codex mcp-server notifications:");
    for line in SAMPLE_NOTIFICATIONS {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => println!("{}", serde_json::to_string_pretty(&value).unwrap()),
            Err(_) => println!("{line}"),
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
