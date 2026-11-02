//! Demonstrates `codex update` via the wrapper (real CLI only).
//!
//! This may mutate the local Codex installation, so it requires:
//! - `CODEX_EXAMPLE_ALLOW_MUTATION=1`
//!
//! Usage:
//! - `CODEX_EXAMPLE_ALLOW_MUTATION=1 cargo run -p unified-agent-api-codex --example update`

use std::{env, error::Error};

use codex::UpdateCommandRequest;

#[path = "support/real_cli.rs"]
mod real_cli;

const ENV_ALLOW_MUTATION: &str = "CODEX_EXAMPLE_ALLOW_MUTATION";

fn mutation_enabled() -> bool {
    matches!(
        env::var(ENV_ALLOW_MUTATION).ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !mutation_enabled() {
        eprintln!(
            "skipping codex update example; set {}=1 to allow self-update",
            ENV_ALLOW_MUTATION
        );
        return Ok(());
    }

    let client = real_cli::default_client();
    let output = client.update(UpdateCommandRequest::new()).await?;
    println!("exit: {}", output.status);
    print!("{}", output.stdout);
    if !output.stderr.is_empty() {
        eprint!("{}", output.stderr);
    }
    Ok(())
}
