//! Demonstrates forcing ANSI colors in Codex output.
//! Usage:
//! ```powershell
//! cargo run -p unified-agent-api-codex --example color_always -- "Show colorful output"
//! ```

use codex::{CodexClient, ColorMode};
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt()?;
    let client = CodexClient::builder().color_mode(ColorMode::Always).build();
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
