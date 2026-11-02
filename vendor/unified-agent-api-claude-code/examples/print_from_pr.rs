//! Demonstrates `--from-pr [value]` (opt-in).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_FROM_PR=123 cargo run -p unified-agent-api-claude-code --example print_from_pr`
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_FROM_PR=https://github.com/org/repo/pull/123 cargo run -p unified-agent-api-claude-code --example print_from_pr`
//!
//! Notes:
//! - This may open an interactive picker depending on CLI behavior and the value provided.

use std::error::Error;

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_from_pr")?;
        return Ok(());
    }

    let from_pr = match real_cli::require_env(real_cli::ENV_EXAMPLE_FROM_PR, "print_from_pr") {
        Some(v) => v,
        None => return Ok(()),
    };

    let client = real_cli::maybe_isolated_client("print_from_pr")?;
    let res = client
        .print(
            real_cli::default_print_request(
                "Summarize the current state of this PR in 1 sentence.",
            )
            .no_prompt()
            .output_format(ClaudeOutputFormat::Text)
            .from_pr_value(from_pr),
        )
        .await?;

    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
