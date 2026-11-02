//! Demonstrates stdout/stderr mirroring configuration on the wrapper client.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-claude-code --example mirror_output`

use std::error::Error;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client =
        real_cli::maybe_isolated_builder_with_mirroring("mirror_output", true, true)?.build();
    let out = client.version().await?;
    println!("exit: {}", out.status);
    println!("captured stdout bytes: {}", out.stdout.len());
    println!("captured stderr bytes: {}", out.stderr.len());
    Ok(())
}
