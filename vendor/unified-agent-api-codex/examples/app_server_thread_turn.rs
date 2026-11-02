//! Kick off an app-server thread/turn over stdio.
//!
//! Requirements:
//! - `CODEX_BINARY` (optional) pointing at the Codex CLI.
//! - `CODEX_HOME` (optional) for isolated state.
//! - `--sample` to print mocked notifications without spawning Codex.
//!
//! Example:
//! ```bash
//! cargo run -p unified-agent-api-codex --example app_server_thread_turn -- "Draft a release note"
//! cargo run -p unified-agent-api-codex --example app_server_thread_turn -- --sample
//! ```

use std::{env, error::Error, path::Path, path::PathBuf};

use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time::{self, Duration},
};

const SAMPLE_NOTIFICATIONS: &[&str] = &[
    r#"{"jsonrpc":"2.0","method":"notifications/thread.started","params":{"thread_id":"demo-thread"}}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/turn.started","params":{"turn_id":"turn-1","thread_id":"demo-thread"}}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/task_complete","params":{"thread_id":"demo-thread","turn_id":"turn-1","message":"Release note drafted."}}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/turn.failed","params":{"thread_id":"demo-thread","turn_id":"turn-2","message":"Sandbox approval timeout"}}"#,
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let use_sample = take_flag(&mut args, "--sample");
    let prompt = if args.is_empty() {
        "Draft a changelog entry for the latest commit".to_string()
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

    demo_app_server(&binary, &prompt).await?;
    Ok(())
}

async fn demo_app_server(binary: &Path, prompt: &str) -> Result<(), Box<dyn Error>> {
    println!("Starting `codex app-server` using {}", binary.display());

    let mut command = Command::new(binary);
    command
        .arg("app-server")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true);

    let mut child = command.spawn()?;
    let mut stdin = child.stdin.take().ok_or("stdin unavailable")?;
    let mut stdout = BufReader::new(child.stdout.take().ok_or("stdout unavailable")?).lines();

    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "client": {
                "name": "codex-app-thread-turn-example",
                "version": env!("CARGO_PKG_VERSION")
            },
            "clientInfo": {
                "name": "codex-app-thread-turn-example",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    let thread_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "thread/start",
        "params": {
            "metadata": {"source": "example"},
        }
    });
    stdin.write_all(initialize.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin
        .write_all(thread_request.to_string().as_bytes())
        .await?;
    stdin.write_all(b"\n").await?;

    let mut thread_id: Option<String> = None;
    while let Some(line) = stdout.next_line().await? {
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            match parse_request_id(&value) {
                Some(0) => println!("[initialize] {line}"),
                Some(1) => {
                    println!("[thread/start] {line}");
                    thread_id = extract_thread_id(&value);
                    break;
                }
                _ => println!("[app-server] {line}"),
            }
        } else {
            println!("[app-server] {line}");
        }
    }

    let thread_id =
        thread_id.ok_or("thread/start did not return a thread id; cannot start a turn")?;

    let turn_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "turn/start",
        "params": {
            "threadId": thread_id,
            "input": [{
                "type": "text",
                "text": prompt
            }],
            "model": "gpt-5-codex",
            "sandbox": true
        }
    });
    stdin.write_all(turn_request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

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
                eprintln!("Failed to read app-server output: {error}");
                break;
            }
            Err(_) => {
                eprintln!("Timed out waiting for app-server notification");
                break;
            }
        }
    }

    let _ = child.kill().await;
    Ok(())
}

fn print_sample_flow() {
    println!("Sample app-server notifications:");
    for line in SAMPLE_NOTIFICATIONS {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => println!("{}", serde_json::to_string_pretty(&value).unwrap()),
            Err(_) => println!("{line}"),
        }
    }
}

fn normalize_uuid(raw: &str) -> String {
    if raw.starts_with("urn:uuid:") {
        raw.to_string()
    } else {
        format!("urn:uuid:{raw}")
    }
}

fn parse_request_id(value: &Value) -> Option<u64> {
    value.get("id").and_then(|id| {
        id.as_u64()
            .or_else(|| id.as_str().and_then(|s| s.parse().ok()))
    })
}

fn extract_thread_id(value: &Value) -> Option<String> {
    value
        .get("result")
        .and_then(|result| {
            result
                .get("threadId")
                .or_else(|| result.get("thread_id"))
                .or_else(|| result.get("thread").and_then(|thread| thread.get("id")))
                .and_then(Value::as_str)
        })
        .map(normalize_uuid)
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
