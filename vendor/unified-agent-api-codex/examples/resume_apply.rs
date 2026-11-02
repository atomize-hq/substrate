//! Resume a Codex session and optionally apply a task.
//!
//! This example resumes via `codex exec resume` (using `--last` or a provided conversation ID).
//! When you also supply a `--task-id` (or `CODEX_TASK_ID`), it calls `codex apply <task-id>`
//! afterward. Pass `--sample` to replay bundled payloads from `crates/codex/examples/fixtures/`
//! when you do not have a Codex binary.
//!
//! Examples:
//! ```bash
//! cargo run -p unified-agent-api-codex --example resume_apply -- --sample
//! CODEX_CONVERSATION_ID=abc123 cargo run -p unified-agent-api-codex --example resume_apply
//! CODEX_TASK_ID=t-123 cargo run -p unified-agent-api-codex --example resume_apply -- --resume-id abc123
//! cargo run -p unified-agent-api-codex --example resume_apply -- --resume-id abc123 --task-id t-123 --no-apply
//! ```

use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process::Stdio,
};

#[path = "support/fixtures.rs"]
mod fixtures;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let use_sample = take_flag(&mut args, "--sample");
    let skip_apply = take_flag(&mut args, "--no-apply");
    let resume_id =
        take_value(&mut args, "--resume-id").or_else(|| env::var("CODEX_CONVERSATION_ID").ok());
    let resume_prompt =
        take_value(&mut args, "--resume-prompt").or_else(|| env::var("CODEX_RESUME_PROMPT").ok());
    let task_id = take_value(&mut args, "--task-id").or_else(|| env::var("CODEX_TASK_ID").ok());

    let binary = resolve_binary();
    if use_sample || !binary_exists(&binary) {
        eprintln!(
            "Using sample resume/apply payloads from {} and {}; set CODEX_BINARY and drop --sample to hit the real binary.",
            fixtures::RESUME_FIXTURE_PATH,
            fixtures::APPLY_FIXTURE_PATH
        );
        replay_samples(!skip_apply);
        return Ok(());
    }

    stream_resume(&binary, resume_id.as_deref(), resume_prompt.as_deref()).await?;
    if !skip_apply {
        if let Some(task_id) = task_id.as_deref() {
            run_apply(&binary, task_id).await?;
        } else {
            println!("--- apply skipped (no --task-id or CODEX_TASK_ID provided) ---");
        }
    }

    Ok(())
}

async fn stream_resume(
    binary: &Path,
    resume_id: Option<&str>,
    resume_prompt: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    println!("--- resume stream ---");

    let mut command = Command::new(binary);
    command.arg("exec").arg("resume");
    if let Some(id) = resume_id {
        command.arg(id);
    } else {
        command.arg("--last");
    }
    command.arg(resume_prompt.unwrap_or_default());
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);

    let mut child = command.spawn()?;
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    while let Some(line) = lines.next_line().await? {
        println!("{line}");
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(format!("codex resume exited with {status}").into());
    }

    Ok(())
}

async fn run_apply(binary: &Path, task_id: &str) -> Result<(), Box<dyn Error>> {
    println!("--- apply task {task_id} ---");
    let output = Command::new(binary)
        .args(["apply", task_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .await?;

    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.status.success() {
        return Err(format!("codex apply exited with {}", output.status).into());
    }

    Ok(())
}

fn replay_samples(include_apply: bool) {
    println!("--- resume stream (sample) ---");
    for line in fixtures::resume_events() {
        println!("{line}");
    }

    if include_apply {
        println!("--- apply (sample) ---");
        println!("{}", fixtures::apply_result());
    }
}

fn resolve_binary() -> PathBuf {
    env::var_os("CODEX_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"))
}

fn binary_exists(path: &Path) -> bool {
    if path.is_absolute() || path.components().count() > 1 {
        std::fs::metadata(path).is_ok()
    } else {
        env::var_os("PATH")
            .and_then(|paths| {
                env::split_paths(&paths)
                    .map(|dir| dir.join(path))
                    .find(|candidate| std::fs::metadata(candidate).is_ok())
            })
            .is_some()
    }
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|value| value != flag);
    before != args.len()
}

fn take_value(args: &mut Vec<String>, key: &str) -> Option<String> {
    let mut value = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == key {
            if i + 1 < args.len() {
                value = Some(args.remove(i + 1));
            }
            args.remove(i);
            break;
        }
        i += 1;
    }
    value
}
