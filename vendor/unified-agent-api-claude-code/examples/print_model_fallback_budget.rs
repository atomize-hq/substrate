//! Demonstrates `--model`, `--fallback-model`, and `--max-budget-usd`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_model_fallback_budget -- "hello"`
//!
//! Optional environment:
//! - `CLAUDE_EXAMPLE_MODEL`: overrides `--model` (default: `sonnet`)
//! - `CLAUDE_EXAMPLE_FALLBACK_MODEL`: overrides `--fallback-model` (default: `opus`)

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
        real_cli::require_live("print_model_fallback_budget")?;
        return Ok(());
    }

    let prompt = collect_prompt()?;
    let model = env::var("CLAUDE_EXAMPLE_MODEL").unwrap_or_else(|_| "sonnet".to_string());
    let fallback = env::var("CLAUDE_EXAMPLE_FALLBACK_MODEL").unwrap_or_else(|_| "opus".to_string());

    let client = real_cli::maybe_isolated_client("print_model_fallback_budget")?;
    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::Text)
                .model(model)
                .fallback_model(fallback)
                .max_budget_usd(0.05),
        )
        .await?;

    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
