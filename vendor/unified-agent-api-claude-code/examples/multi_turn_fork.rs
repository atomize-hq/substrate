//! Demonstrates resuming a session with `--fork-session` to create a new session ID.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example multi_turn_fork`

use std::{error::Error, time::SystemTime};

use claude_code::{parse_stream_json_lines, ClaudeOutputFormat, StreamJsonLineOutcome};

#[path = "support/real_cli.rs"]
mod real_cli;

fn pseudo_uuid() -> String {
    use std::hash::{Hash, Hasher};

    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    std::process::id().hash(&mut h1);
    SystemTime::now().hash(&mut h1);
    let a = h1.finish();

    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    "salt".hash(&mut h2);
    std::process::id().hash(&mut h2);
    SystemTime::now().hash(&mut h2);
    let b = h2.finish();

    let hex = format!("{a:016x}{b:016x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn extract_session_id(outcomes: &[StreamJsonLineOutcome]) -> Option<String> {
    for o in outcomes {
        let StreamJsonLineOutcome::Ok { value, .. } = o else {
            continue;
        };
        let id = value
            .get("session_id")
            .or_else(|| value.get("sessionId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if id.is_some() {
            return id;
        }
    }
    None
}

fn outcomes_from_stdout(bytes: &[u8]) -> Vec<StreamJsonLineOutcome> {
    let text = String::from_utf8_lossy(bytes);
    parse_stream_json_lines(&text)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("multi_turn_fork")?;
        return Ok(());
    }

    let work = real_cli::example_working_dir("multi_turn_fork")?;
    let session_id = pseudo_uuid();

    let client = real_cli::maybe_isolated_builder_with_mirroring("multi_turn_fork", false, false)?
        .working_dir(work.path())
        .build();

    let res1 = client
        .print(
            real_cli::default_print_request("Turn 1: say 'ready' and keep it short.")
                .output_format(ClaudeOutputFormat::StreamJson)
                .session_id(session_id.clone()),
        )
        .await?;
    let out1 = outcomes_from_stdout(&res1.output.stdout);
    let observed1 = extract_session_id(&out1);

    let res2 = client
        .print(
            real_cli::default_print_request("Turn 2: confirm you remember the previous message.")
                .output_format(ClaudeOutputFormat::StreamJson)
                .resume_value(session_id.clone())
                .fork_session(true),
        )
        .await?;
    let out2 = outcomes_from_stdout(&res2.output.stdout);
    let observed2 = extract_session_id(&out2);

    println!("requested session_id: {session_id}");
    println!("observed1: {}", observed1.as_deref().unwrap_or("<none>"));
    println!("observed2: {}", observed2.as_deref().unwrap_or("<none>"));
    if let (Some(a), Some(b)) = (&observed1, &observed2) {
        if a != b {
            println!("fork-session: session_id changed as expected");
        } else {
            println!("fork-session: session_id did not change (may be CLI/version-dependent)");
        }
    }

    Ok(())
}
