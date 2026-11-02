//! Demonstrates `codex review` and `codex exec review` wrapper APIs.
//!
//! These commands may contact the network / model provider and are only run when explicitly
//! requested.
//!
//! Usage:
//! - Show help (default): `cargo run -p unified-agent-api-codex --example review_commands`
//! - Run (requires repo + auth): `CODEX_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-codex --example review_commands -- run --prompt "Review this diff"`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.
//! - `CODEX_EXAMPLE_LIVE=1` to allow executing review commands.

use std::env;

use codex::{ExecReviewCommandRequest, HelpCommandRequest, HelpScope, ReviewCommandRequest};

#[path = "support/real_cli.rs"]
mod real_cli;

fn env_truthy(key: &str) -> bool {
    matches!(
        env::var(key).ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("help");

    let client = real_cli::default_client();

    match subcommand {
        "run" => {
            if !env_truthy("CODEX_EXAMPLE_LIVE") {
                eprintln!(
                    "Set CODEX_EXAMPLE_LIVE=1 to run review commands (they may have side effects)."
                );
                return Ok(());
            }

            let prompt = take_value(&mut args, "--prompt")
                .or_else(|| args.get(1).cloned())
                .unwrap_or_else(|| "Review the current changes".to_string());

            let review = client
                .review(
                    ReviewCommandRequest::new()
                        .uncommitted(true)
                        .prompt(prompt.clone()),
                )
                .await?;
            println!("--- codex review ---");
            print!("{}", review.stdout);
            if !review.stderr.is_empty() {
                eprintln!("{}", review.stderr);
            }

            let exec_review = client
                .exec_review(
                    ExecReviewCommandRequest::new()
                        .uncommitted(true)
                        .json(true)
                        .prompt(prompt),
                )
                .await?;
            println!("--- codex exec review ---");
            print!("{}", exec_review.stdout);
            if !exec_review.stderr.is_empty() {
                eprintln!("{}", exec_review.stderr);
            }
        }
        _ => {
            let root_help = client
                .help(HelpCommandRequest::new(HelpScope::Root).command(["review"]))
                .await?;
            println!("--- codex help review ---");
            print!("{}", root_help.stdout);

            let exec_help = client
                .help(HelpCommandRequest::new(HelpScope::Exec).command(["review"]))
                .await?;
            println!("--- codex exec help review ---");
            print!("{}", exec_help.stdout);
        }
    }

    Ok(())
}

fn take_value(args: &mut Vec<String>, key: &str) -> Option<String> {
    if let Some(pos) = args.iter().position(|arg| arg == key) {
        if pos + 1 < args.len() {
            let value = args.remove(pos + 1);
            args.remove(pos);
            return Some(value);
        }
    }
    None
}
