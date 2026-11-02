//! Demonstrates `--output-format json`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_json -- "hello"`

use std::error::Error;

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_json")?;
        return Ok(());
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello".to_string());
    let client = real_cli::maybe_isolated_client("print_json")?;
    let res = client
        .print(real_cli::default_print_request(prompt).output_format(ClaudeOutputFormat::Json))
        .await?;

    let v: serde_json::Value = serde_json::from_slice(&res.output.stdout)?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}
