//! Demonstrates `codex exec resume` (streaming) and `codex apply` via the wrapper.
//!
//! Usage:
//! - `CODEX_TASK_ID=... cargo run -p unified-agent-api-codex --example exec_resume_apply_wrapper -- --last --prompt \"continue\"`
//! - `cargo run -p unified-agent-api-codex --example exec_resume_apply_wrapper -- --resume-id <SESSION_ID> --prompt \"continue\"`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.
//! - `CODEX_TASK_ID` (optional): when set, `apply` runs after the resume completes.

use std::{env, error::Error};

use codex::{ResumeRequest, ResumeSelector};
use futures_util::StreamExt;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let _last = take_flag(&mut args, "--last");
    let all = take_flag(&mut args, "--all");
    let resume_id = take_value(&mut args, "--resume-id");
    let prompt = take_value(&mut args, "--prompt").unwrap_or_else(|| "continue".to_string());

    let selector = if let Some(id) = resume_id {
        ResumeSelector::Id(id)
    } else if all {
        ResumeSelector::All
    } else {
        ResumeSelector::Last
    };

    let client = real_cli::default_client();

    let mut stream = client
        .stream_resume(ResumeRequest {
            selector,
            prompt: Some(prompt),
            ephemeral: false,
            ignore_rules: false,
            ignore_user_config: false,
            idle_timeout: None,
            output_last_message: None,
            output_schema: None,
            json_event_log: None,
            overrides: Default::default(),
        })
        .await?;

    while let Some(evt) = stream.events.next().await {
        match evt {
            Ok(event) => println!("{}", serde_json::to_string(&event)?),
            Err(err) => {
                eprintln!("stream error: {err}");
                break;
            }
        }
    }

    let _ = stream.completion.await?;

    if let Ok(task_id) = env::var("CODEX_TASK_ID") {
        let output = client.apply_task(task_id).await?;
        print!("{}", output.stdout);
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
