//! Demonstrate `--output-last-message` and `--output-schema` handling.
//!
//! Codex can write the final assistant message and the output schema to disk while streaming JSON.
//! This example runs `codex exec --json` with those flags, then reads the generated files. If the
//! binary is missing, it falls back to sample payloads so you can still see the shapes.
//!
//! Example:
//! ```bash
//! OUTPUT_DIR=/tmp/codex-artifacts \
//!   cargo run -p unified-agent-api-codex --example stream_last_message -- "Summarize repo status"
//! ```

use std::{env, error::Error, fs, path::Path, path::PathBuf};

use serde_json::{json, Value};
use tokio::{io::AsyncWriteExt, process::Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt();
    let (base_dir, _temp_guard) = match env::var_os("OUTPUT_DIR") {
        Some(dir) => {
            let path = PathBuf::from(dir);
            fs::create_dir_all(&path)?;
            (path, None)
        }
        None => {
            let temp = tempfile::tempdir()?;
            let path = temp.path().to_path_buf();
            (path, Some(temp))
        }
    };

    let last_message_path = base_dir.join("last_message.json");
    let schema_path = base_dir.join("output_schema.json");
    ensure_schema_file(&schema_path)?;

    let binary = resolve_binary();
    let ran_real_codex = if binary_exists(&binary) {
        run_codex(&binary, &prompt, &last_message_path, &schema_path)
            .await
            .is_ok()
    } else {
        false
    };

    if !ran_real_codex {
        eprintln!(
            "Falling back to sample last-message/schema payloads; set CODEX_BINARY to run Codex."
        );
        write_sample_outputs(&last_message_path, &schema_path)?;
    }

    print_json_preview("Last message", &last_message_path)?;
    print_json_preview("Output schema", &schema_path)?;
    println!("Artifacts stored under {}", base_dir.display());
    Ok(())
}

fn collect_prompt() -> String {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        "Summarize this repository in one paragraph".to_string()
    } else {
        args.join(" ")
    }
}

fn resolve_binary() -> PathBuf {
    env::var_os("CODEX_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"))
}

fn binary_exists(path: &Path) -> bool {
    if path.is_absolute() || path.components().count() > 1 {
        fs::metadata(path).is_ok()
    } else {
        env::var_os("PATH")
            .and_then(|paths| {
                env::split_paths(&paths)
                    .map(|dir| dir.join(path))
                    .find(|candidate| fs::metadata(candidate).is_ok())
            })
            .is_some()
    }
}

async fn run_codex(
    binary: &Path,
    prompt: &str,
    last_message_path: &Path,
    schema_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new(binary);
    command
        .args([
            "exec",
            "--json",
            "--skip-git-repo-check",
            "--output-last-message",
            last_message_path
                .to_str()
                .ok_or("Non-UTF8 last message path")?,
            "--output-schema",
            schema_path.to_str().ok_or("Non-UTF8 schema path")?,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true);

    let mut child = command.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.shutdown().await?;
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(format!("codex exited with {status}").into());
    }

    // Older binaries may not write these files; treat missing outputs as a soft failure so we can
    // fall back to samples.
    if !last_message_path.exists() || !schema_path.exists() {
        return Err("codex did not write output-last-message/schema files".into());
    }
    Ok(())
}

fn write_sample_outputs(
    last_message_path: &Path,
    schema_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let last_message = json!({
        "role": "assistant",
        "content": [
            {"type": "text", "text": "Summary: the repo contains the Codex crate and a growing example suite."}
        ],
        "metadata": {
            "thread_id": "demo-thread",
            "turn_id": "turn-1"
        }
    });
    let schema = json!({
        "title": "Command output schema",
        "type": "object",
        "properties": {
            "stdout": {"type": "string"},
            "stderr": {"type": "string"},
            "exit_code": {"type": "integer"}
        }
    });
    fs::write(
        last_message_path,
        serde_json::to_string_pretty(&last_message)?,
    )?;
    fs::write(schema_path, serde_json::to_string_pretty(&schema)?)?;
    Ok(())
}

fn print_json_preview(label: &str, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    println!("{label}: {}", path.display());
    let contents = fs::read_to_string(path)?;
    if let Ok(value) = serde_json::from_str::<Value>(&contents) {
        if let Some(text) = value
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|items| items.first())
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str())
        {
            println!("  text: {text}");
        }
    }
    println!("{contents}");
    Ok(())
}

fn ensure_schema_file(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        return Ok(());
    }
    // Minimal schema that satisfies the CLI schema validator (requires additionalProperties=false).
    let schema = json!({
        "type": "object",
        "properties": {
            "message": { "type": "string" }
        },
        "required": ["message"],
        "additionalProperties": false
    });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&schema)?)?;
    Ok(())
}
