//! Demonstrates `codex features enable` and `codex features disable` via the wrapper.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-codex --example features_toggle -- enable unified_exec`
//! - `cargo run -p unified-agent-api-codex --example features_toggle -- disable unified_exec`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.

use codex::{FeaturesDisableRequest, FeaturesEnableRequest};

#[path = "support/real_cli.rs"]
mod real_cli;

fn usage() {
    eprintln!("usage: features_toggle <enable|disable> <FEATURE>");
    eprintln!("examples:");
    eprintln!("  features_toggle enable unified_exec");
    eprintln!("  features_toggle disable unified_exec");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let Some(action) = args.next() else {
        usage();
        return Ok(());
    };
    let Some(feature) = args.next() else {
        usage();
        return Ok(());
    };

    let client = real_cli::default_client();

    match action.to_ascii_lowercase().as_str() {
        "enable" => {
            let output = client
                .features_enable(FeaturesEnableRequest::new(feature))
                .await?;
            print!("{}", output.stdout);
            if !output.stderr.is_empty() {
                eprintln!("{}", output.stderr);
            }
        }
        "disable" => {
            let output = client
                .features_disable(FeaturesDisableRequest::new(feature))
                .await?;
            print!("{}", output.stdout);
            if !output.stderr.is_empty() {
                eprintln!("{}", output.stderr);
            }
        }
        _ => {
            usage();
        }
    }

    Ok(())
}
