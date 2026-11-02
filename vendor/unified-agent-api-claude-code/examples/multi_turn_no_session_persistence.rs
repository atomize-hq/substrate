//! Demonstrates `--no-session-persistence`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example multi_turn_no_session_persistence`
//!
//! Notes:
//! - Runs in a temp working directory.
//! - Best-effort demonstration: if upstream behavior differs, the example prints a note instead of failing.

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("multi_turn_no_session_persistence")?;
        return Ok(());
    }

    let work = real_cli::example_working_dir("multi_turn_no_session_persistence")?;
    let session_id = pseudo_uuid();
    let client = real_cli::maybe_isolated_builder_with_mirroring(
        "multi_turn_no_session_persistence",
        false,
        false,
    )?
    .working_dir(work.path())
    .build();

    let res1 = client
        .print(
            real_cli::default_print_request("Turn 1: say 'ready' and keep it short.")
                .output_format(ClaudeOutputFormat::StreamJson)
                .no_session_persistence(true)
                .session_id(session_id.clone()),
        )
        .await?;
    println!("turn1 exit: {}", res1.output.status);

    let raw1 = String::from_utf8_lossy(&res1.output.stdout);
    let out1 = parse_stream_json_lines(&raw1);
    println!("requested session_id: {session_id}");
    println!(
        "observed1 session_id: {}",
        extract_session_id(&out1).as_deref().unwrap_or("<none>")
    );

    let res2 = client
        .print(
            real_cli::default_print_request("Turn 2: attempt a resume.")
                .output_format(ClaudeOutputFormat::StreamJson)
                .no_session_persistence(true)
                .resume_value(session_id.clone()),
        )
        .await?;
    println!("turn2 exit: {}", res2.output.status);

    if res2.output.status.success() {
        println!(
            "note: resume succeeded even with --no-session-persistence; upstream behavior may differ"
        );
    } else {
        println!("resume failed as expected (non-success status)");
    }

    Ok(())
}
