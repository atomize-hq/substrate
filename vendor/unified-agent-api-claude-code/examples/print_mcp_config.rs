//! Demonstrates `--mcp-config`, `--strict-mcp-config`, and `--mcp-debug`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_MCP_CONFIG=/path/to/mcp.json cargo run -p unified-agent-api-claude-code --example print_mcp_config -- "hello"`
//!
//! Notes:
//! - This example is opt-in because MCP configuration schemas and local setup vary.

use std::{env, error::Error};

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

fn collect_prompt() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("Provide a prompt string".into());
    }
    Ok(args.join(" "))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_mcp_config")?;
        return Ok(());
    }

    let config = match real_cli::require_env(real_cli::ENV_EXAMPLE_MCP_CONFIG, "print_mcp_config") {
        Some(v) => v,
        None => return Ok(()),
    };
    let strict = !matches!(
        env::var("CLAUDE_EXAMPLE_MCP_STRICT").ok().as_deref(),
        Some("0") | Some("false") | Some("no")
    );
    let debug = !matches!(
        env::var("CLAUDE_EXAMPLE_MCP_DEBUG").ok().as_deref(),
        Some("0") | Some("false") | Some("no")
    );

    let prompt = collect_prompt()?;
    let client = real_cli::maybe_isolated_client("print_mcp_config")?;
    let mut req = real_cli::default_print_request(prompt)
        .output_format(ClaudeOutputFormat::Text)
        .mcp_config(config);
    req = req.strict_mcp_config(strict);
    req = req.mcp_debug(debug);

    let res = client.print(req).await?;
    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
