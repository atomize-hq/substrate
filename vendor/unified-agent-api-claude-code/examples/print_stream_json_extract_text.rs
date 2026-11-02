//! Demonstrates extracting assistant text from `--output-format stream-json`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_stream_json_extract_text -- "hello"`

use std::{env, error::Error};

use claude_code::{parse_stream_json_lines, ClaudeOutputFormat, StreamJsonLineOutcome};

#[path = "support/real_cli.rs"]
mod real_cli;

fn collect_prompt() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("Provide a prompt string".into());
    }
    Ok(args.join(" "))
}

fn extract_session_id(outcomes: &[StreamJsonLineOutcome]) -> Option<String> {
    for o in outcomes {
        let StreamJsonLineOutcome::Ok { value, .. } = o else {
            continue;
        };
        let id = value
            .get("session_id")
            .or_else(|| value.get("sessionId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if id.is_some() {
            return id;
        }
    }
    None
}

fn extract_assistant_text(outcomes: &[StreamJsonLineOutcome]) -> String {
    let mut from_assistant = String::new();
    let mut from_deltas = String::new();

    for o in outcomes {
        let StreamJsonLineOutcome::Ok { value, .. } = o else {
            continue;
        };

        let ty = value.get("type").and_then(|v| v.as_str());
        match ty {
            Some("assistant") => {
                if let Some(items) = value
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())
                {
                    for item in items {
                        if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                            if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                                from_assistant.push_str(t);
                            }
                        }
                    }
                }
            }
            Some("stream_event") => {
                let delta = value.get("event").and_then(|e| e.get("delta"));
                let text = delta.and_then(|d| d.get("text")).and_then(|v| v.as_str());
                if let Some(t) = text {
                    from_deltas.push_str(t);
                }
            }
            _ => {}
        }
    }

    if !from_assistant.is_empty() {
        from_assistant
    } else {
        from_deltas
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_stream_json_extract_text")?;
        return Ok(());
    }

    let prompt = collect_prompt()?;
    let client = real_cli::maybe_isolated_client("print_stream_json_extract_text")?;
    let res = client
        .print(
            real_cli::default_print_request(prompt).output_format(ClaudeOutputFormat::StreamJson),
        )
        .await?;

    println!("exit: {}", res.output.status);
    let text = String::from_utf8_lossy(&res.output.stdout);
    let outcomes = parse_stream_json_lines(&text);

    if let Some(session_id) = extract_session_id(&outcomes) {
        println!("session_id: {session_id}");
    }

    let msg = extract_assistant_text(&outcomes);
    if msg.is_empty() {
        println!("no assistant text found in stream-json output");
    } else {
        println!("assistant_text:\n{msg}");
    }

    Ok(())
}
