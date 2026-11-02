//! Demonstrates `claude update` via the wrapper (real CLI only).
//!
//! This may mutate the local Claude installation, so it requires:
//! - `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_ALLOW_MUTATION=1 cargo run -p unified-agent-api-claude-code --example update`

use std::error::Error;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    real_cli::require_mutation("update")?;
    if !real_cli::mutation_enabled() {
        return Ok(());
    }

    let client = real_cli::maybe_isolated_client("update")?;
    let out = client.update().await?;
    println!("exit: {}", out.status);
    print!("{}", String::from_utf8_lossy(&out.stdout));
    eprint!("{}", String::from_utf8_lossy(&out.stderr));
    Ok(())
}
