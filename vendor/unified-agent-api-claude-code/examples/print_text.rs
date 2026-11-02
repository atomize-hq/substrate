//! Demonstrates `claude --print` via `ClaudeClient::print` (real CLI only).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_text -- "Hello"`
//! - Optional isolation: `CLAUDE_EXAMPLE_ISOLATED_HOME=1`
//!
//! Environment:
//! - `CLAUDE_BINARY` (optional): path to the `claude` CLI binary.
//! - `CLAUDE_EXAMPLE_LIVE=1`: enable live/auth-required examples.

use std::{env, error::Error};

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_text")?;
        return Ok(());
    }

    let prompt = collect_prompt()?;
    let client = real_cli::maybe_isolated_client("print_text")?;
    let req = real_cli::default_print_request(prompt).output_format(ClaudeOutputFormat::Text);
    let res = client.print(req).await?;
    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}

fn collect_prompt() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("Provide a prompt string".into());
    }
    Ok(args.join(" "))
}
