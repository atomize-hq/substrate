//! Demonstrates suppressing Codex stderr mirroring (`quiet(true)`).
//! Usage:
//! ```powershell
//! cargo run -p unified-agent-api-codex --example quiet -- "Run without tool noise"
//! ```

use codex::CodexClient;
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt()?;
    let client = CodexClient::builder().quiet(true).build();
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
