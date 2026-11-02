//! Demonstrates `--betas <betas...>` (opt-in).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_BETAS=\"beta1 beta2\" cargo run -p unified-agent-api-claude-code --example print_betas -- \"hello\"`

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
        real_cli::require_live("print_betas")?;
        return Ok(());
    }

    let betas = match real_cli::require_env(real_cli::ENV_EXAMPLE_BETAS, "print_betas") {
        Some(v) => v,
        None => return Ok(()),
    };
    let betas = split_whitespace(&betas);
    if betas.is_empty() {
        eprintln!(
            "skipped print_betas: {} is empty",
            real_cli::ENV_EXAMPLE_BETAS
        );
        return Ok(());
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello".to_string());
    let client = real_cli::maybe_isolated_client("print_betas")?;
    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::Text)
                .betas(betas),
        )
        .await?;

    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
