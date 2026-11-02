//! Demonstrates `--continue` in a temp working directory.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example multi_turn_continue`
//!
//! Notes:
//! - `--continue` continues the most recent conversation in the current directory.
//! - This example uses a temp working directory to avoid touching the repo.

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

fn outcomes_from_stdout(bytes: &[u8]) -> Vec<StreamJsonLineOutcome> {
    let text = String::from_utf8_lossy(bytes);
    parse_stream_json_lines(&text)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("multi_turn_continue")?;
        return Ok(());
    }

    let work = real_cli::example_working_dir("multi_turn_continue")?;
    let client =
        real_cli::maybe_isolated_builder_with_mirroring("multi_turn_continue", false, false)?
            .working_dir(work.path())
            .build();

    let res1 = client
        .print(
            real_cli::default_print_request("Turn 1: say 'ready' and keep it short.")
                .output_format(ClaudeOutputFormat::StreamJson),
        )
        .await?;
    let out1 = outcomes_from_stdout(&res1.output.stdout);
    let observed1 = extract_session_id(&out1);

    let res2 = client
        .print(
            real_cli::default_print_request("Turn 2: confirm you remember the previous message.")
                .output_format(ClaudeOutputFormat::StreamJson)
                .continue_session(true),
        )
        .await?;
    let out2 = outcomes_from_stdout(&res2.output.stdout);
    let observed2 = extract_session_id(&out2);

    println!(
        "observed1 session_id: {}",
        observed1.as_deref().unwrap_or("<none>")
    );
    println!(
        "observed2 session_id: {}",
        observed2.as_deref().unwrap_or("<none>")
    );

    Ok(())
}
