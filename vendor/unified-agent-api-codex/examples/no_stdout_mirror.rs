//! Demonstrates disabling stdout mirroring while still capturing Codex output.
//! Usage:
//! ```powershell
//! cargo run -p unified-agent-api-codex --example no_stdout_mirror -- "Stream quietly"
//! ```

use codex::CodexClient;
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt()?;
    let client = CodexClient::builder().mirror_stdout(false).build();
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
