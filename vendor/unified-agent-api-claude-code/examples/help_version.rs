//! Demonstrates `claude --help` and `claude --version` via the wrapper (real CLI only).
//!
//! Usage:
//! - `cargo run -p unified-agent-api-claude-code --example help_version`
//! - Optional isolation: `CLAUDE_EXAMPLE_ISOLATED_HOME=1`

use std::error::Error;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = real_cli::maybe_isolated_client("help_version")?;

    let help = client.help().await?;
    println!("--help exit: {}", help.status);
    print!("{}", String::from_utf8_lossy(&help.stdout));
    eprint!("{}", String::from_utf8_lossy(&help.stderr));

    let version = client.version().await?;
    println!("--version exit: {}", version.status);
    print!("{}", String::from_utf8_lossy(&version.stdout));
    eprint!("{}", String::from_utf8_lossy(&version.stderr));

    Ok(())
}
