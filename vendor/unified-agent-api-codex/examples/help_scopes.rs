//! Prints help text for each `codex <scope> help` family supported by the wrapper.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-codex --example help_scopes`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.

use codex::{HelpCommandRequest, HelpScope};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = real_cli::default_client();

    let scopes = [
        HelpScope::Root,
        HelpScope::Exec,
        HelpScope::Features,
        HelpScope::Login,
        HelpScope::AppServer,
        HelpScope::Sandbox,
        HelpScope::Cloud,
        HelpScope::Mcp,
    ];

    for scope in scopes {
        let out = client.help(HelpCommandRequest::new(scope)).await?;
        println!("--- {:?} ---", scope);
        print!("{}", out.stdout);
        if !out.stderr.is_empty() {
            eprintln!("{}", out.stderr);
        }
        println!();
    }

    Ok(())
}
