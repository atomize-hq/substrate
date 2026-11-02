//! Demonstrates `--chrome` and `--no-chrome`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_ALLOW_CHROME=1 cargo run -p unified-agent-api-claude-code --example print_chrome_flags -- chrome -- \"hello\"`
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_ALLOW_CHROME=1 cargo run -p unified-agent-api-claude-code --example print_chrome_flags -- no-chrome -- \"hello\"`

use std::{env, error::Error};

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

fn usage() -> &'static str {
    "usage: print_chrome_flags <chrome|no-chrome> <prompt...>"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_chrome_flags")?;
        return Ok(());
    }
    if !matches!(
        env::var(real_cli::ENV_EXAMPLE_ALLOW_CHROME).ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    ) {
        eprintln!(
            "skipped print_chrome_flags: set {}=1 to run chrome integration examples",
            real_cli::ENV_EXAMPLE_ALLOW_CHROME
        );
        return Ok(());
    }

    let mut args = env::args().skip(1);
    let mode = args.next().ok_or(usage())?;
    let prompt_parts: Vec<String> = args.collect();
    if prompt_parts.is_empty() {
        return Err(usage().into());
    }
    let prompt = prompt_parts.join(" ");

    let client = real_cli::maybe_isolated_client("print_chrome_flags")?;
    let mut req = real_cli::default_print_request(prompt).output_format(ClaudeOutputFormat::Text);
    req = match mode.as_str() {
        "chrome" => req.chrome(),
        "no-chrome" => req.no_chrome(),
        _ => return Err(usage().into()),
    };
    let res = client.print(req).await?;
    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
