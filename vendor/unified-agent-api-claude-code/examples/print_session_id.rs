//! Prints the current session ID from a real `claude --print --output-format stream-json` run.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_session_id -- "hello"`
//! - Optional isolation: `CLAUDE_EXAMPLE_ISOLATED_HOME=1`

use std::error::Error;

use claude_code::{parse_stream_json_lines, ClaudeOutputFormat, StreamJsonLineOutcome};

#[path = "support/real_cli.rs"]
mod real_cli;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_session_id")?;
        return Ok(());
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello".to_string());
    let client = real_cli::maybe_isolated_client("print_session_id")?;
    let res = client
        .print(
            real_cli::default_print_request(prompt).output_format(ClaudeOutputFormat::StreamJson),
        )
        .await?;

    let text = String::from_utf8_lossy(&res.output.stdout);
    let outcomes = parse_stream_json_lines(&text);

    if let Some(session_id) = extract_session_id(&outcomes) {
        println!("session_id: {session_id}");
    } else {
        eprintln!("no session_id found in stream-json output");
    }

    Ok(())
}
