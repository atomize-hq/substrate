//! Demonstrates `--plugin-dir <paths...>` (opt-in).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_PLUGIN_DIRS=\"/path/to/plugins\" cargo run -p unified-agent-api-claude-code --example print_plugin_dirs -- \"hello\"`
//!
//! Notes:
//! - This does not install plugins. It only shows how to pass plugin search paths for a session.

use std::error::Error;

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

fn split_whitespace(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.to_string())
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_plugin_dirs")?;
        return Ok(());
    }

    let dirs = match real_cli::require_env(real_cli::ENV_EXAMPLE_PLUGIN_DIRS, "print_plugin_dirs") {
        Some(v) => v,
        None => return Ok(()),
    };
    let dirs = split_whitespace(&dirs);
    if dirs.is_empty() {
        eprintln!(
            "skipped print_plugin_dirs: {} is empty",
            real_cli::ENV_EXAMPLE_PLUGIN_DIRS
        );
        return Ok(());
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello".to_string());
    let client = real_cli::maybe_isolated_client("print_plugin_dirs")?;
    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::Text)
                .plugin_dirs(dirs),
        )
        .await?;

    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
