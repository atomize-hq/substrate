//! Continue an existing Codex MCP session via the `codex-reply` tool.
//!
//! Requirements:
//! - `CODEX_BINARY` (optional) to point at the Codex CLI.
//! - `CODEX_HOME` (optional) for app-scoped state.
//! - `CODEX_CONVERSATION_ID` must be set (or pass one as the first argument). On 0.61.0, use the `session_id` from the `session_configured` event.
//! - Use `--sample` to see mocked notifications without spawning Codex.
//!
//! Example:
//! ```bash
//! CODEX_CONVERSATION_ID=abc123 \
//!   cargo run -p unified-agent-api-codex --example mcp_codex_reply -- "Continue the prior run"
//! ```

use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time::{self, Duration},
};

const SAMPLE_NOTIFICATIONS: &[&str] = &[
    r#"{"jsonrpc":"2.0","method":"codex/event","params":{"type":"approval_required","kind":"apply","message":"Apply staged diff?","thread_id":"demo-thread","turn_id":"turn-2"}}"#,
    r#"{"jsonrpc":"2.0","method":"codex/event","params":{"type":"task_complete","message":"Conversation resumed","turn_id":"turn-2","thread_id":"demo-thread"}}"#,
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let use_sample = take_flag(&mut args, "--sample");
    let conversation_id_arg = if !args.is_empty() {
        Some(args.remove(0))
    } else {
        None
    };
    let conversation_id = conversation_id_arg
        .or_else(|| env::var("CODEX_CONVERSATION_ID").ok())
        .as_deref()
        .map(normalize_conversation_id);
    let prompt = if args.is_empty() {
        "Resume the last Codex turn".to_string()
    } else {
        args.join(" ")
    };

    if conversation_id.is_none() {
        eprintln!("Set CODEX_CONVERSATION_ID or pass a conversation id as the first argument.");
        print_sample_flow();
        return Ok(());
    }

    if use_sample {
        print_sample_flow();
        return Ok(());
    }

    if !looks_like_uuid(conversation_id.as_deref()) {
        eprintln!("Conversation ID must be a UUID (optionally prefixed with urn:uuid:). Provide the `session_id` from `session_configured` (or another valid conversationId).");
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

    demo_codex_reply(&binary, conversation_id.as_deref().unwrap(), &prompt).await?;
    Ok(())
}

async fn demo_codex_reply(
    binary: &Path,
    conversation_id: &str,
    prompt: &str,
) -> Result<(), Box<dyn Error>> {
    println!(
        "Starting `codex mcp-server` then calling codex-reply for conversation {conversation_id}"
    );

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

    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "client": {
                "name": "codex-mcp-reply-example",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    // Try to resume the conversation before issuing codex-reply. 0.61.0 does not rehydrate
    // mcp-server sessions from disk, but this makes the intent explicit and future-proofs
    // when resume is available.
    let resume_path = find_rollout_path(conversation_id);
    if let Some(path) = resume_path.as_ref() {
        eprintln!("Attempting resumeConversation from {path:?}");
    } else {
        eprintln!(
            "Resume path not found; relying on in-memory session (may fail across processes)"
        );
    }

    let resume = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resumeConversation",
        "params": {
            "conversation_id": conversation_id,
            "conversationId": conversation_id,
            "path": resume_path.as_ref().map(|p| p.to_string_lossy().to_string())
        }
    });

    let resume_v2 = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "thread/resume",
        "params": {
            "thread_id": conversation_id,
            "threadId": conversation_id,
            "path": resume_path.as_ref().map(|p| p.to_string_lossy().to_string())
        }
    });

    let request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "codex-reply",
            "arguments": {
                "conversationId": conversation_id,
                "prompt": prompt
            }
        },
    });

    stdin.write_all(initialize.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.write_all(resume.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.write_all(resume_v2.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.write_all(request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    let mut seen = 0;
    while seen < 8 {
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

fn normalize_conversation_id(id: &str) -> String {
    if id.starts_with("urn:uuid:") {
        id.to_string()
    } else {
        format!("urn:uuid:{id}")
    }
}

fn looks_like_uuid(id: Option<&str>) -> bool {
    let Some(mut value) = id else { return false };
    if let Some(stripped) = value.strip_prefix("urn:uuid:") {
        value = stripped;
    }
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (idx, b) in bytes.iter().enumerate() {
        let is_hyphen = *b == b'-';
        let should_hyphen = matches!(idx, 8 | 13 | 18 | 23);
        if should_hyphen != is_hyphen {
            return false;
        }
        if !should_hyphen && !b.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn print_sample_flow() {
    println!("Sample codex-reply notifications:");
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

fn find_rollout_path(conversation_id: &str) -> Option<PathBuf> {
    let needle = conversation_id
        .strip_prefix("urn:uuid:")
        .unwrap_or(conversation_id);
    let code_home = env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".codex"))
        })
        .unwrap_or_else(|| PathBuf::from(".codex"));
    let sessions = code_home.join("sessions");
    if !sessions.exists() {
        return None;
    }

    let mut stack = vec![sessions];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.contains(needle) && name.starts_with("rollout-") {
                    return Some(path);
                }
            }
        }
    }

    None
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|value| value != flag);
    before != args.len()
}
