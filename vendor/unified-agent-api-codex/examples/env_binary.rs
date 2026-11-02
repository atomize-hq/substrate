//! Demonstrates running Codex through an alternate binary path.
//! Set `CODEX_BINARY` before invoking this example:
//! ```powershell
//! $env:CODEX_BINARY="C:\bin\codex-nightly.exe"
//! cargo run -p unified-agent-api-codex --example env_binary -- "Nightly sanity check"
//! ```

use codex::CodexClient;
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let binary =
        env::var("CODEX_BINARY").map_err(|_| "Set CODEX_BINARY before running this example")?;
    println!("Using CODEX_BINARY={binary}");

    let prompt = collect_prompt()?;
    let client = CodexClient::builder().build();
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
