//! Demonstrates `claude doctor` via the wrapper (real CLI only).
//!
//! Usage:
//! - `cargo run -p unified-agent-api-claude-code --example doctor`
//! - Optional isolation: `CLAUDE_EXAMPLE_ISOLATED_HOME=1`

use std::error::Error;
use std::time::Duration;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = real_cli::maybe_isolated_builder_with_mirroring("doctor", false, false)?
        // `claude doctor` can take a while depending on updater state; don't flake on the
        // wrapper default timeout.
        .timeout(Some(Duration::from_secs(600)))
        .build();
    let out = client.doctor().await?;
    println!("exit: {}", out.status);
    print!("{}", String::from_utf8_lossy(&out.stdout));
    eprint!("{}", String::from_utf8_lossy(&out.stderr));
    Ok(())
}
