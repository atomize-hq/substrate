//! Demonstrates MCP management commands that are upstream-gated to Windows (`win32-x64`).
//!
//! Usage (Windows only):
//! - `cargo run -p unified-agent-api-claude-code --example mcp_manage -- list`
//! - Mutating commands require: `CLAUDE_EXAMPLE_ALLOW_MUTATION=1`
//!   - `... -- add <NAME> <COMMAND_OR_URL>`
//!   - `... -- remove <NAME>`
//!   - `... -- add-json <NAME> <JSON>`
//!   - `... -- serve`
//!   - `... -- add-from-claude-desktop`

use std::{env, error::Error};

use claude_code::{
    ClaudeCommandRequest, McpAddFromClaudeDesktopRequest, McpAddJsonRequest, McpAddRequest,
    McpRemoveRequest, McpScope, McpServeRequest,
};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !cfg!(windows) {
        eprintln!("skipped mcp_manage: upstream MCP management subcommands are win32-x64 only");
        return Ok(());
    }

    let client = real_cli::maybe_isolated_client("mcp_manage")?;
    let mut args = env::args().skip(1);
    let sub = args.next().unwrap_or_else(|| "list".to_string());

    match sub.as_str() {
        "list" => {
            let out = client.mcp_list().await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "get" => {
            let name = args.next().ok_or("usage: get <NAME>")?;
            let out = client
                .mcp_get(claude_code::McpGetRequest::new(name))
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "add" => {
            real_cli::require_mutation("mcp_manage add")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let name = args.next().ok_or("usage: add <NAME> <COMMAND_OR_URL>")?;
            let command = args.next().ok_or("usage: add <NAME> <COMMAND_OR_URL>")?;
            let out = client
                .mcp_add(
                    McpAddRequest::new(name, command)
                        .scope(McpScope::User)
                        .args(args),
                )
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "remove" => {
            real_cli::require_mutation("mcp_manage remove")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let name = args.next().ok_or("usage: remove <NAME>")?;
            let out = client
                .mcp_remove(McpRemoveRequest::new(name).scope(McpScope::User))
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "add-json" => {
            real_cli::require_mutation("mcp_manage add-json")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let name = args.next().ok_or("usage: add-json <NAME> <JSON>")?;
            let json = args.next().ok_or("usage: add-json <NAME> <JSON>")?;
            let out = client
                .mcp_add_json(McpAddJsonRequest::new(name, json).scope(McpScope::User))
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "serve" => {
            real_cli::require_mutation("mcp_manage serve")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let out = client.mcp_serve(McpServeRequest::new().args(args)).await?;
            println!("exit: {}", out.status);
        }
        "add-from-claude-desktop" => {
            real_cli::require_mutation("mcp_manage add-from-claude-desktop")?;
            if !real_cli::mutation_enabled() {
                return Ok(());
            }
            let out = client
                .mcp_add_from_claude_desktop(
                    McpAddFromClaudeDesktopRequest::new().scope(McpScope::User),
                )
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        "root" => {
            let out = client
                .run_command(ClaudeCommandRequest::new(["mcp"]))
                .await?;
            println!("exit: {}", out.status);
            print!("{}", String::from_utf8_lossy(&out.stdout));
        }
        other => {
            eprintln!("unknown subcommand: {other}");
        }
    }

    Ok(())
}
