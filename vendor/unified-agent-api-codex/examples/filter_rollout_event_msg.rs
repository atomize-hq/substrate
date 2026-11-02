//! Filter Codex CLI rollout JSONL (`rollout-*.jsonl`) records by `event_msg` and `response_item`
//! payload types.
//!
//! Usage:
//! - Filter a known rollout file (multiple filters allowed):
//!   `cargo run -p unified-agent-api-codex --example filter_rollout_event_msg -- --path ~/.codex/sessions/.../rollout-....jsonl --event-msg-type token_count --event-msg-type user_message --response-item-type message`
//! - Find a rollout file by id substring (typically the session id embedded in the filename):
//!   `cargo run -p unified-agent-api-codex --example filter_rollout_event_msg -- --session-id <ID> --event-msg-type agent_reasoning`
//! - Parse from stdin:
//!   `cat rollout.jsonl | cargo run -p unified-agent-api-codex --example filter_rollout_event_msg -- --stdin --event-msg-type token_count`
//!
//! Notes:
//! - Rollout logs are not the same schema as `codex exec --json`.
//! - `--event-msg-type` may be repeated; comma-separated values are also accepted.
//! - `--response-item-type` may be repeated; comma-separated values are also accepted.
//! - `--response-role` may be repeated; comma-separated values are also accepted.
//! - Output snippets are truncated by default; use `--max-chars` / `--no-truncate` to control it.

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use codex::RolloutResponseItemPayload;
use codex::{
    find_rollout_file_by_id, rollout_jsonl_file, rollout_jsonl_reader, RolloutEvent,
    RolloutEventMsgPayload,
};

#[derive(Debug)]
enum Source {
    Path(PathBuf),
    Stdin,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let mut list_types = false;
    let mut stdin = false;
    let mut no_truncate = false;
    let mut max_chars = None::<String>;
    let mut path = None::<PathBuf>;
    let mut session_id = None::<String>;
    let mut filters: Vec<String> = Vec::new();
    let mut response_filters: Vec<String> = Vec::new();
    let mut response_roles: Vec<String> = Vec::new();

    // Lightweight argv parser:
    // - Multi-value options accept either repetition or space-separated lists until the next `--flag`.
    // - Comma-separated values are accepted and split later.
    let mut idx = 0usize;
    while idx < args.len() {
        let arg = args[idx].as_str();
        match arg {
            "--list-types" => {
                list_types = true;
                idx += 1;
            }
            "--stdin" => {
                stdin = true;
                idx += 1;
            }
            "--no-truncate" => {
                no_truncate = true;
                idx += 1;
            }
            "--max-chars" => {
                let value = args
                    .get(idx + 1)
                    .ok_or("Provide a value after --max-chars")?;
                max_chars = Some(value.to_string());
                idx += 2;
            }
            "--path" => {
                let value = args.get(idx + 1).ok_or("Provide a value after --path")?;
                path = Some(PathBuf::from(value));
                idx += 2;
            }
            "--session-id" | "--conversation-id" => {
                let value = args
                    .get(idx + 1)
                    .ok_or("Provide a value after --session-id/--conversation-id")?;
                session_id = Some(value.to_string());
                idx += 2;
            }
            "--event-msg-type" | "--type" => {
                idx += 1;
                let mut took = 0usize;
                while idx < args.len() && !args[idx].starts_with("--") {
                    filters.push(args[idx].clone());
                    idx += 1;
                    took += 1;
                }
                if took == 0 {
                    return Err("Provide at least one value after --event-msg-type/--type".into());
                }
            }
            "--response-item-type" | "--response-type" => {
                idx += 1;
                let mut took = 0usize;
                while idx < args.len() && !args[idx].starts_with("--") {
                    response_filters.push(args[idx].clone());
                    idx += 1;
                    took += 1;
                }
                if took == 0 {
                    return Err(
                        "Provide at least one value after --response-item-type/--response-type"
                            .into(),
                    );
                }
            }
            "--response-role" | "--role" => {
                idx += 1;
                let mut took = 0usize;
                while idx < args.len() && !args[idx].starts_with("--") {
                    response_roles.push(args[idx].clone());
                    idx += 1;
                    took += 1;
                }
                if took == 0 {
                    return Err("Provide at least one value after --response-role/--role".into());
                }
            }
            other => {
                return Err(format!("Unrecognized argument: {other:?}. See --help.").into());
            }
        }
    }

    let max_chars: usize = if no_truncate {
        usize::MAX
    } else if let Some(raw) = max_chars {
        raw.parse::<usize>()
            .map_err(|_| format!("Invalid --max-chars value: {raw:?}"))?
    } else {
        140
    };

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
            return Err("Provide exactly one of: --stdin, --path <FILE>, --session-id <ID>".into())
        }
    };

    if !list_types && filters.is_empty() && response_filters.is_empty() {
        return Err(
            "Provide at least one --event-msg-type <KIND> and/or --response-item-type <KIND> (or use --list-types)"
                .into(),
        );
    }

    let mut want_event = BTreeSet::new();
    for raw in filters {
        for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            want_event.insert(part.to_string());
        }
    }

    let mut want_response = BTreeSet::new();
    for raw in response_filters {
        for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            want_response.insert(part.to_string());
        }
    }

    let mut want_roles = BTreeSet::new();
    for raw in response_roles {
        for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            want_roles.insert(part.to_string());
        }
    }

    let mut out = io::stdout().lock();
    let mut event_msg_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut response_item_counts: BTreeMap<String, usize> = BTreeMap::new();

    let mut ok = 0usize;
    let mut err = 0usize;

    match source {
        Source::Path(path) => {
            eprintln!("Parsing {}", path.display());
            for record in rollout_jsonl_file(path)? {
                match record.outcome {
                    Ok(event) => {
                        ok += 1;
                        if let Err(error) = process_event(
                            &mut out,
                            &mut event_msg_counts,
                            &mut response_item_counts,
                            list_types,
                            &want_event,
                            &want_response,
                            &want_roles,
                            max_chars,
                            record.line_number,
                            event,
                        ) {
                            if error.kind() == io::ErrorKind::BrokenPipe {
                                break;
                            }
                            return Err(error.into());
                        }
                    }
                    Err(error) => {
                        err += 1;
                        if let Err(error) =
                            writeln!(out, "{:>6}  error: {error}", record.line_number)
                        {
                            if error.kind() == io::ErrorKind::BrokenPipe {
                                break;
                            }
                        }
                    }
                }
            }
        }
        Source::Stdin => {
            eprintln!("Parsing stdin...");
            let stdin = io::stdin();
            let lock = stdin.lock();
            for record in rollout_jsonl_reader(lock) {
                match record.outcome {
                    Ok(event) => {
                        ok += 1;
                        if let Err(error) = process_event(
                            &mut out,
                            &mut event_msg_counts,
                            &mut response_item_counts,
                            list_types,
                            &want_event,
                            &want_response,
                            &want_roles,
                            max_chars,
                            record.line_number,
                            event,
                        ) {
                            if error.kind() == io::ErrorKind::BrokenPipe {
                                break;
                            }
                            return Err(error.into());
                        }
                    }
                    Err(error) => {
                        err += 1;
                        if let Err(error) =
                            writeln!(out, "{:>6}  error: {error}", record.line_number)
                        {
                            if error.kind() == io::ErrorKind::BrokenPipe {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    if list_types {
        writeln!(out, "event_msg kinds:")?;
        for (kind, count) in event_msg_counts {
            writeln!(out, "{:>8}  {}", count, kind)?;
        }
        writeln!(out, "\nresponse_item kinds:")?;
        for (kind, count) in response_item_counts {
            writeln!(out, "{:>8}  {}", count, kind)?;
        }
    }

    eprintln!("Done. ok={ok} err={err}");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_event(
    out: &mut impl Write,
    event_msg_counts: &mut BTreeMap<String, usize>,
    response_item_counts: &mut BTreeMap<String, usize>,
    list_types: bool,
    want_event: &BTreeSet<String>,
    want_response: &BTreeSet<String>,
    want_roles: &BTreeSet<String>,
    max_chars: usize,
    line_number: usize,
    event: RolloutEvent,
) -> io::Result<()> {
    match event {
        RolloutEvent::EventMsg(msg) => {
            let kind = msg.payload.kind.as_deref().unwrap_or("-").to_string();
            *event_msg_counts.entry(kind.clone()).or_insert(0) += 1;

            if !list_types && want_event.contains(&kind) {
                let snippet =
                    event_msg_snippet(&msg.payload, max_chars).unwrap_or_else(|| "-".to_string());
                writeln!(
                    out,
                    "{:>6}  event_msg:{:<14}  {}",
                    line_number, kind, snippet
                )?;
            }
        }
        RolloutEvent::ResponseItem(item) => {
            let kind = item.payload.kind.as_deref().unwrap_or("-").to_string();
            *response_item_counts.entry(kind.clone()).or_insert(0) += 1;

            if !list_types && want_response.contains(&kind) {
                if !want_roles.is_empty() {
                    let role = item.payload.role.as_deref().unwrap_or("");
                    if !want_roles.contains(role) {
                        return Ok(());
                    }
                }
                let snippet = response_item_snippet(&item.payload, max_chars)
                    .unwrap_or_else(|| "-".to_string());
                writeln!(
                    out,
                    "{:>6}  response_item:{:<10}  {}",
                    line_number, kind, snippet
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn event_msg_snippet(payload: &RolloutEventMsgPayload, max_chars: usize) -> Option<String> {
    if let Some(value) = payload.extra.get("message").and_then(|v| v.as_str()) {
        return Some(truncate(value, max_chars));
    }
    if let Some(value) = payload.extra.get("text").and_then(|v| v.as_str()) {
        return Some(truncate(value, max_chars));
    }
    if payload.kind.as_deref() == Some("token_count") {
        let total_tokens = payload
            .extra
            .get("info")
            .and_then(|v| v.get("total_token_usage"))
            .and_then(|v| v.get("total_tokens"))
            .and_then(|v| v.as_u64());
        if let Some(total_tokens) = total_tokens {
            return Some(format!("total_tokens={total_tokens}"));
        }
    }
    None
}

fn response_item_snippet(payload: &RolloutResponseItemPayload, max_chars: usize) -> Option<String> {
    match payload.kind.as_deref()? {
        "message" => {
            let role = payload.role.as_deref().unwrap_or("-");
            let text = payload
                .content
                .as_deref()
                .and_then(|parts| parts.iter().find_map(|p| p.text.as_deref()))
                .map(|t| truncate(t, max_chars))
                .unwrap_or_else(|| "-".to_string());
            Some(format!("role={role} text={text}"))
        }
        "reasoning" => {
            let text = payload
                .summary
                .as_deref()
                .and_then(|parts| parts.iter().find_map(|p| p.text.as_deref()))
                .map(|t| truncate(t, max_chars))
                .unwrap_or_else(|| "-".to_string());
            Some(format!("summary={text}"))
        }
        "function_call" => Some(format!(
            "name={} call_id={}",
            payload.name.as_deref().unwrap_or("-"),
            payload.call_id.as_deref().unwrap_or("-")
        )),
        "function_call_output" => {
            let call_id = payload.call_id.as_deref().unwrap_or("-");
            let first = payload
                .output
                .as_deref()
                .and_then(|s| s.lines().next())
                .unwrap_or("-");
            Some(format!(
                "call_id={call_id} first_line={}",
                truncate(first, max_chars.min(120))
            ))
        }
        other => Some(other.to_string()),
    }
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

fn find_rollout_path(session_id: &str) -> Option<PathBuf> {
    let needle = session_id.strip_prefix("urn:uuid:").unwrap_or(session_id);
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

fn print_help() {
    eprintln!(
        "Filter rollout JSONL by event_msg / response_item payload type\n\n\
USAGE:\n\
  filter_rollout_event_msg --path <FILE> [--event-msg-type <KIND> ...] [--response-item-type <KIND> ...]\n\
  filter_rollout_event_msg --session-id <ID> [--event-msg-type <KIND> ...] [--response-item-type <KIND> ...]\n\
  filter_rollout_event_msg --stdin [--event-msg-type <KIND> ...] [--response-item-type <KIND> ...]\n\
  filter_rollout_event_msg <SOURCE> --list-types\n\n\
FLAGS:\n\
  --event-msg-type <KIND>   Filter to matching event kinds (repeatable; accepts comma-separated or space-separated).\n\
  --response-item-type <KIND> Filter to matching response kinds (repeatable; accepts comma-separated or space-separated).\n\
  --response-role <ROLE>    Filter response_item records by role (repeatable; accepts comma-separated or space-separated).\n\
  --list-types              Print a frequency table of observed kinds.\n\
  --max-chars <N>           Truncate text snippets to N characters (default: 140).\n\
  --no-truncate             Disable snippet truncation.\n\
  --type <KIND>             Alias for --event-msg-type.\n\n\
  --response-type <KIND>    Alias for --response-item-type.\n\n\
  --role <ROLE>             Alias for --response-role.\n\n\
SOURCE:\n\
  Exactly one of: --stdin, --path <FILE>, --session-id <ID>.\n\
  <ID> may be prefixed with urn:uuid:. It is matched against the rollout filename and the\n\
  `session_meta.payload.id` value inside the file.\n"
    );
}
