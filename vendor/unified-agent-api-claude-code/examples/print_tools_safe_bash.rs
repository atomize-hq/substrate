//! Demonstrates tool usage in a safe sandboxed working directory.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_tools_safe_bash`
//!
//! Notes:
//! - Uses a temp working directory and restricts tool access to it with `--add-dir`.
//! - Sets `--dangerously-skip-permissions` to avoid headless permission prompts.

use std::{error::Error, fs};

use claude_code::{parse_stream_json_lines, ClaudeOutputFormat, StreamJsonLineOutcome};

#[path = "support/real_cli.rs"]
mod real_cli;

fn extract_assistant_text(outcomes: &[StreamJsonLineOutcome]) -> String {
    let mut out = String::new();
    for o in outcomes {
        let StreamJsonLineOutcome::Ok { value, .. } = o else {
            continue;
        };
        if value.get("type").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }
        if let Some(items) = value
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
        {
            for item in items {
                if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                    if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                        out.push_str(t);
                    }
                }
            }
        }
    }
    out
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_tools_safe_bash")?;
        return Ok(());
    }

    let work = real_cli::example_working_dir("print_tools_safe_bash")?;
    let work_path = work.path().to_string_lossy().to_string();

    let input = work.path().join("input.txt");
    fs::write(&input, "hello from tools\n")?;

    let client =
        real_cli::maybe_isolated_builder_with_mirroring("print_tools_safe_bash", false, false)?
            .working_dir(work.path())
            .build();

    let prompt = "You are running in a sandbox directory.\n\
Only use tools within this directory.\n\
1) Use the Read tool to read input.txt.\n\
2) Use Bash to run: ls -la\n\
3) Reply with a single sentence summarizing what you found.\n"
        .to_string();

    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::StreamJson)
                .permission_mode("bypassPermissions")
                .tools(["Bash,Read"])
                .allowed_tools(["Bash", "Read"])
                .add_dirs([work_path]),
        )
        .await?;

    println!("exit: {}", res.output.status);
    let raw = String::from_utf8_lossy(&res.output.stdout);
    let outcomes = parse_stream_json_lines(&raw);
    let text = extract_assistant_text(&outcomes);
    if !text.is_empty() {
        println!("assistant_text:\n{text}");
    } else {
        println!("no assistant message text found; raw stdout follows:");
        print!("{raw}");
    }

    Ok(())
}
