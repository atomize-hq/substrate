//! Demonstrates running Codex inside a specific working directory.
//! Usage:
//! ```powershell
//! cargo run -p unified-agent-api-codex --example working_dir -- "C:\path\to\repo" "List files here"
//! ```

use codex::CodexClient;
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();
    if matches!(args.peek().map(|s| s.as_str()), Some("--")) {
        args.next();
    }
    let dir = args
        .next()
        .ok_or("Provide a working directory followed by a prompt")?;
    let prompt_parts: Vec<String> = args.collect();
    if prompt_parts.is_empty() {
        return Err("Provide a prompt after the directory".into());
    }
    let prompt = prompt_parts.join(" ");

    let client = CodexClient::builder().working_dir(&dir).build();
    let response = client.send_prompt(&prompt).await?;
    println!("{response}");
    Ok(())
}
