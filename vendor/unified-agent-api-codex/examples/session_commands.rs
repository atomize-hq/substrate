//! Demonstrates `codex resume` and `codex fork` wrapper APIs.
//!
//! These commands may contact the network / model provider and are only run when explicitly
//! requested.
//!
//! Usage:
//! - Show help (default): `cargo run -p unified-agent-api-codex --example session_commands`
//! - Resume: `CODEX_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-codex --example session_commands -- resume --last --prompt \"continue\"`
//! - Fork: `CODEX_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-codex --example session_commands -- fork --last --prompt \"branch off\"`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.
//! - `CODEX_EXAMPLE_LIVE=1` to allow executing resume/fork commands.

use std::env;

use codex::{ForkSessionRequest, HelpCommandRequest, HelpScope, ResumeSessionRequest};

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
    let subcommand = args.first().cloned().unwrap_or_else(|| "help".to_string());

    let client = real_cli::default_client();

    match subcommand.as_str() {
        "resume" | "fork" => {
            if !env_truthy("CODEX_EXAMPLE_LIVE") {
                eprintln!("Set CODEX_EXAMPLE_LIVE=1 to run session commands (they may have side effects).");
                return Ok(());
            }

            let last = take_flag(&mut args, "--last");
            let all = take_flag(&mut args, "--all");
            let session_id = take_value(&mut args, "--session-id");
            let prompt = take_value(&mut args, "--prompt").or_else(|| args.get(1).cloned());

            if subcommand == "resume" {
                let mut req = ResumeSessionRequest::new().last(last).all(all);
                if let Some(id) = session_id {
                    req = req.session_id(id);
                }
                if let Some(prompt) = prompt {
                    req = req.prompt(prompt);
                }
                let out = client.resume_session(req).await?;
                print!("{}", out.stdout);
            } else {
                let mut req = ForkSessionRequest::new().last(last).all(all);
                if let Some(id) = session_id {
                    req = req.session_id(id);
                }
                if let Some(prompt) = prompt {
                    req = req.prompt(prompt);
                }
                let out = client.fork_session(req).await?;
                print!("{}", out.stdout);
            }
        }
        _ => {
            let resume_help = client
                .help(HelpCommandRequest::new(HelpScope::Root).command(["resume"]))
                .await?;
            println!("--- codex help resume ---");
            print!("{}", resume_help.stdout);

            let fork_help = client
                .help(HelpCommandRequest::new(HelpScope::Root).command(["fork"]))
                .await?;
            println!("--- codex help fork ---");
            print!("{}", fork_help.stdout);
        }
    }

    Ok(())
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(pos) = args.iter().position(|arg| arg == flag) {
        args.remove(pos);
        true
    } else {
        false
    }
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
