//! Demonstrates `codex features` and `codex features list` via the wrapper.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-codex --example features_cmd`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.

use codex::{FeaturesCommandRequest, FeaturesListFormat, FeaturesListRequest};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = real_cli::default_client();

    let features = client.features(FeaturesCommandRequest::new()).await?;
    println!("--- codex features ---");
    print!("{}", features.stdout);
    if !features.stderr.is_empty() {
        eprintln!("{}", features.stderr);
    }

    let list = client
        .list_features(FeaturesListRequest::new().json(false))
        .await?;
    println!("--- codex features list ({:?}) ---", list.format);
    println!("features: {}", list.features.len());
    if matches!(list.format, FeaturesListFormat::Text) {
        // Printing the full table is noisy; show the first few lines.
        for line in list.stdout.lines().take(10) {
            println!("{line}");
        }
    }

    Ok(())
}
