//! Demonstrates how examples resolve the real `claude` binary.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-claude-code --example env_binary`
//! - Override: `CLAUDE_BINARY=/path/to/claude cargo run -p unified-agent-api-claude-code --example env_binary`

use std::error::Error;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let binary = real_cli::resolve_binary();
    println!("resolved claude binary: {}", binary.display());

    let client = real_cli::default_client();
    let out = client.version().await?;
    print!("{}", String::from_utf8_lossy(&out.stdout));
    Ok(())
}
