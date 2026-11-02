//! Demonstrates non-mutating MCP queries (`mcp list` + `mcp reset-project-choices`).
//!
//! Usage:
//! - `cargo run -p unified-agent-api-claude-code --example mcp_list`
//! - Optional isolation: `CLAUDE_EXAMPLE_ISOLATED_HOME=1`

use std::error::Error;

use claude_code::ClaudeCommandRequest;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = real_cli::maybe_isolated_client("mcp_list")?;

    // `mcp` root help / command listing (no typed API; use the generic command request).
    let root = client
        .run_command(ClaudeCommandRequest::new(["mcp", "--help"]))
        .await?;
    println!("mcp exit: {}", root.status);
    print!("{}", String::from_utf8_lossy(&root.stdout));
    eprint!("{}", String::from_utf8_lossy(&root.stderr));

    let out = client.mcp_list().await?;
    println!("mcp list exit: {}", out.status);
    print!("{}", String::from_utf8_lossy(&out.stdout));
    eprint!("{}", String::from_utf8_lossy(&out.stderr));

    let reset = client.mcp_reset_project_choices().await?;
    println!("mcp reset-project-choices exit: {}", reset.status);
    print!("{}", String::from_utf8_lossy(&reset.stdout));
    eprint!("{}", String::from_utf8_lossy(&reset.stderr));

    Ok(())
}
