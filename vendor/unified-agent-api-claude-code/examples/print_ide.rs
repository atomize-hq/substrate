//! Demonstrates `--ide` (opt-in).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_ALLOW_IDE=1 cargo run -p unified-agent-api-claude-code --example print_ide -- "hello"`

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
        real_cli::require_live("print_ide")?;
        return Ok(());
    }
    if !matches!(
        env::var(real_cli::ENV_EXAMPLE_ALLOW_IDE).ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    ) {
        eprintln!(
            "skipped print_ide: set {}=1 to run IDE integration examples",
            real_cli::ENV_EXAMPLE_ALLOW_IDE
        );
        return Ok(());
    }

    let prompt = collect_prompt()?;
    let client = real_cli::maybe_isolated_client("print_ide")?;
    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::Text)
                .ide(true),
        )
        .await?;

    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
