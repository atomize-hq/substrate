//! Demonstrates attaching an image while streaming Codex JSONL output quietly.
//! Usage:
//! ```powershell
//! cargo run -p unified-agent-api-codex --example image_json -- "C:\path\to\image.png" "Describe the screenshot"
//! ```

use codex::CodexClient;
use std::{env, error::Error, path::PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();
    if matches!(args.peek().map(|s| s.as_str()), Some("--")) {
        args.next();
    }
    let image_path = args
        .next()
        .ok_or("Provide an image path followed by a prompt")?;
    let prompt_parts: Vec<String> = args.collect();
    if prompt_parts.is_empty() {
        return Err("Provide a prompt after the image path".into());
    }
    let prompt = prompt_parts.join(" ");

    let client = CodexClient::builder()
        .image(PathBuf::from(image_path))
        .json(true)
        .quiet(true)
        .mirror_stdout(false)
        .build();
    let response = client.send_prompt(&prompt).await?;
    println!("{response}");
    Ok(())
}
