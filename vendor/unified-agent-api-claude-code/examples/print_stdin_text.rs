//! Demonstrates supplying input via stdin (without a prompt positional).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_stdin_text`
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_stdin_text -- \"hello from stdin\"`

use std::error::Error;

use claude_code::{ClaudeInputFormat, ClaudeOutputFormat};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_stdin_text")?;
        return Ok(());
    }

    let text = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello from stdin".to_string());

    let client = real_cli::maybe_isolated_client("print_stdin_text")?;
    let res = client
        .print(
            real_cli::default_print_request("ignored")
                .no_prompt()
                .stdin_bytes(text.into_bytes())
                .input_format(ClaudeInputFormat::Text)
                .output_format(ClaudeOutputFormat::Text),
        )
        .await?;

    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
