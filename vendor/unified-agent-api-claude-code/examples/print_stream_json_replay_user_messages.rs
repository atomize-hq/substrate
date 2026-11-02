//! Demonstrates `--input-format stream-json` with `--replay-user-messages`.
//!
//! Usage:
//! - Provide stream-json input via env:
//!   - `export CLAUDE_EXAMPLE_STREAM_JSON_INPUT='{"type":"user","session_id":"...","message":{"content":[{"type":"text","text":"hi"}]}}'`
//! - Then run:
//!   - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_stream_json_replay_user_messages`
//!
//! Notes:
//! - The stream-json input schema is CLI/version dependent; this example is opt-in.

use std::error::Error;

use claude_code::{ClaudeInputFormat, ClaudeOutputFormat};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_stream_json_replay_user_messages")?;
        return Ok(());
    }

    let input = match real_cli::require_env(
        real_cli::ENV_EXAMPLE_STREAM_JSON_INPUT,
        "print_stream_json_replay_user_messages",
    ) {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = real_cli::maybe_isolated_client("print_stream_json_replay_user_messages")?;
    let res = client
        .print(
            real_cli::default_print_request("ignored")
                .no_prompt()
                .stdin_bytes(input.into_bytes())
                .input_format(ClaudeInputFormat::StreamJson)
                .output_format(ClaudeOutputFormat::StreamJson)
                .replay_user_messages(true),
        )
        .await?;

    println!("exit: {}", res.output.status);
    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
