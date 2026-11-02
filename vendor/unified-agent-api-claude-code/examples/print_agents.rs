//! Demonstrates `--agent` / `--agents` (opt-in).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_AGENT=reviewer cargo run -p unified-agent-api-claude-code --example print_agents -- \"hello\"`
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_AGENTS_JSON='{\"reviewer\": {\"description\": \"Reviews code\", \"prompt\": \"You are a code reviewer\"}}' cargo run -p unified-agent-api-claude-code --example print_agents -- \"hello\"`

use std::error::Error;

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_agents")?;
        return Ok(());
    }

    let agents_json = std::env::var(real_cli::ENV_EXAMPLE_AGENTS_JSON).ok();
    let agent = std::env::var(real_cli::ENV_EXAMPLE_AGENT).ok();
    if agents_json.as_deref().unwrap_or("").trim().is_empty()
        && agent.as_deref().unwrap_or("").trim().is_empty()
    {
        eprintln!(
            "skipped print_agents: set {} or {}",
            real_cli::ENV_EXAMPLE_AGENTS_JSON,
            real_cli::ENV_EXAMPLE_AGENT
        );
        return Ok(());
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Say hello.".to_string());
    let client = real_cli::maybe_isolated_client("print_agents")?;

    let mut req = real_cli::default_print_request(prompt).output_format(ClaudeOutputFormat::Text);
    if let Some(json) = agents_json.filter(|s| !s.trim().is_empty()) {
        req = req.agents(json);
    }
    if let Some(name) = agent.filter(|s| !s.trim().is_empty()) {
        req = req.agent(name);
    }

    let res = client.print(req).await?;
    print!("{}", String::from_utf8_lossy(&res.output.stdout));
    Ok(())
}
