//! Offline parsing for Codex CLI rollout JSONL (`rollout-*.jsonl`) files.
//!
//! Usage:
//! - Parse an explicit path:
//!   `cargo run -p unified-agent-api-codex --example parse_rollout_jsonl -- --path ~/.codex/sessions/.../rollout-....jsonl`
//! - Find a rollout file by id substring (typically the session id embedded in the filename):
//!   `cargo run -p unified-agent-api-codex --example parse_rollout_jsonl -- --session-id <ID>`
//! - Parse from stdin:
//!   `cat rollout.jsonl | cargo run -p unified-agent-api-codex --example parse_rollout_jsonl -- --stdin`
//!
//! Notes:
//! - Rollout logs are not the same schema as `codex exec --json`.
//! - Malformed/unrecognized lines surface as per-line errors; parsing continues (unknown record
//!   `type`s are preserved as `RolloutEvent::Unknown`).

use std::{
    env,
    error::Error,
    io::{self},
    path::PathBuf,
};

use codex::{
    find_rollout_file_by_id, rollout_jsonl_file, rollout_jsonl_reader, RolloutEvent,
    RolloutJsonlRecord, RolloutUnknown,
};
use std::io::Write;

#[derive(Debug)]
enum Source {
    Path(PathBuf),
    Stdin,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let stdin = take_flag(&mut args, "--stdin");
    let path = take_value(&mut args, "--path").map(PathBuf::from);
    let session_id = take_value(&mut args, "--session-id")
        .or_else(|| take_value(&mut args, "--conversation-id"));

    let source = match (stdin, path, session_id) {
        (true, None, None) => Source::Stdin,
        (false, Some(path), None) => Source::Path(path),
        (false, None, Some(session_id)) => {
            let path = find_rollout_path(&session_id).ok_or(
                "Rollout file not found under $CODEX_HOME/sessions (or ~/.codex/sessions)",
            )?;
            Source::Path(path)
        }
        _ => {
            return Err("Provide exactly one of: --stdin, --path <FILE>, --session-id <ID>".into());
        }
    };

    let (ok, err) = match source {
        Source::Path(path) => {
            eprintln!("Parsing {}", path.display());
            let reader = rollout_jsonl_file(path)?;
            consume_records(reader)
        }
        Source::Stdin => {
            eprintln!("Parsing stdin...");
            let stdin = io::stdin();
            let lock = stdin.lock();
            let reader = rollout_jsonl_reader(lock);
            consume_records(reader)
        }
    };

    eprintln!("Done. ok={ok} err={err}");
    Ok(())
}

fn consume_records(reader: impl Iterator<Item = RolloutJsonlRecord>) -> (usize, usize) {
    let mut ok = 0usize;
    let mut err = 0usize;
    let mut out = io::stdout().lock();
    for record in reader {
        match record.outcome {
            Ok(event) => {
                ok += 1;
                if let Err(error) = writeln!(
                    out,
                    "{:>6}  {}",
                    record.line_number,
                    summarize_event(&event)
                ) {
                    if error.kind() == io::ErrorKind::BrokenPipe {
                        break;
                    }
                }
            }
            Err(error) => {
                err += 1;
                if let Err(error) = writeln!(out, "{:>6}  error: {error}", record.line_number) {
                    if error.kind() == io::ErrorKind::BrokenPipe {
                        break;
                    }
                }
            }
        }
    }
    (ok, err)
}

fn summarize_event(event: &RolloutEvent) -> String {
    match event {
        RolloutEvent::SessionMeta(meta) => format!(
            "session_meta id={} cli_version={} cwd={}",
            meta.payload.id.as_deref().unwrap_or("-"),
            meta.payload.cli_version.as_deref().unwrap_or("-"),
            meta.payload.cwd.as_deref().unwrap_or("-")
        ),
        RolloutEvent::EventMsg(msg) => {
            format!("event_msg {}", msg.payload.kind.as_deref().unwrap_or("-"))
        }
        RolloutEvent::ResponseItem(item) => {
            summarize_response_item(item.payload.kind.as_deref().unwrap_or("-"), item)
        }
        RolloutEvent::Unknown(unknown) => summarize_unknown(unknown),
    }
}

fn summarize_response_item(kind: &str, item: &codex::RolloutResponseItem) -> String {
    match kind {
        "message" => {
            let role = item.payload.role.as_deref().unwrap_or("-");
            let text = first_text(item.payload.content.as_deref());
            format!("response_item message role={role} {text}")
        }
        "reasoning" => {
            let text = first_text(item.payload.summary.as_deref());
            format!("response_item reasoning {text}")
        }
        "function_call" => format!(
            "response_item function_call name={} call_id={}",
            item.payload.name.as_deref().unwrap_or("-"),
            item.payload.call_id.as_deref().unwrap_or("-")
        ),
        "function_call_output" => {
            let call_id = item.payload.call_id.as_deref().unwrap_or("-");
            let first = item
                .payload
                .output
                .as_deref()
                .and_then(|s| s.lines().next())
                .unwrap_or("-");
            format!(
                "response_item function_call_output call_id={call_id} first_line={}",
                truncate(first, 120)
            )
        }
        _ => format!("response_item {kind}"),
    }
}

fn summarize_unknown(unknown: &RolloutUnknown) -> String {
    format!("unknown {}", unknown.record_type)
}

fn first_text(parts: Option<&[codex::RolloutContentPart]>) -> String {
    let Some(parts) = parts else {
        return "text=-".to_string();
    };
    for part in parts {
        if let Some(text) = part.text.as_deref() {
            return format!("text={}", truncate(text, 120));
        }
    }
    "text=-".to_string()
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.replace('\n', "\\n");
    }
    let mut out = String::new();
    for (idx, ch) in value.chars().enumerate() {
        if idx >= max {
            break;
        }
        if ch == '\n' {
            out.push_str("\\n");
        } else {
            out.push(ch);
        }
    }
    out.push('…');
    out
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
    find_rollout_file_by_id(&code_home, needle)
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|value| value != flag);
    before != args.len()
}

fn take_value(args: &mut Vec<String>, key: &str) -> Option<String> {
    if let Some(pos) = args.iter().position(|arg| arg == key) {
        if pos + 1 < args.len() {
            let value = args.remove(pos + 1);
            args.remove(pos);
            return Some(value);
        }
    }
    None
}

fn print_help() {
    eprintln!(
        "Offline Codex JSONL parser (rollout files)\n\n\
USAGE:\n\
  parse_rollout_jsonl --path <FILE>\n\
  parse_rollout_jsonl --session-id <ID>\n\
  parse_rollout_jsonl --stdin\n\n\
NOTES:\n\
  - Searches for rollout files under $CODEX_HOME/sessions/ (or ~/.codex/sessions/).\n\
  - <ID> may be prefixed with urn:uuid:. It is matched against the rollout filename and the\n\
    `session_meta.payload.id` value inside the file.\n"
    );
}
