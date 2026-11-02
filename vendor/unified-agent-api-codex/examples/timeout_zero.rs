//! Demonstrates disabling the Codex timeout entirely (`timeout = 0`).
//! Usage:
//! ```powershell
//! cargo run -p unified-agent-api-codex --example timeout_zero -- "Stream until completion"
//! ```

use codex::CodexClient;
use std::{env, error::Error, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt()?;
    let client = CodexClient::builder().timeout(Duration::ZERO).build();
    let response = client.send_prompt(&prompt).await?;
    println!("{response}");
    Ok(())
}

fn collect_prompt() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("Provide a prompt".into());
    }
    Ok(args.join(" "))
}
