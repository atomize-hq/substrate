//! Demonstrates `--include-partial-messages` with `--output-format stream-json`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_include_partial_messages -- "hello"`

use std::{collections::BTreeMap, error::Error};

use claude_code::{parse_stream_json_lines, ClaudeOutputFormat, StreamJsonLineOutcome};

#[path = "support/real_cli.rs"]
mod real_cli;

fn summarize(outcomes: &[StreamJsonLineOutcome]) -> (usize, usize, BTreeMap<String, usize>) {
    let mut ok = 0usize;
    let mut err = 0usize;
    let mut types: BTreeMap<String, usize> = BTreeMap::new();

    for o in outcomes {
        match o {
            StreamJsonLineOutcome::Ok { value, .. } => {
                ok += 1;
                let ty = value
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<?>")
                    .to_string();
                *types.entry(ty).or_default() += 1;
            }
            StreamJsonLineOutcome::Err { .. } => err += 1,
        }
    }

    (ok, err, types)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_include_partial_messages")?;
        return Ok(());
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello".to_string());
    let client = real_cli::maybe_isolated_client("print_include_partial_messages")?;

    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::StreamJson)
                .include_partial_messages(true),
        )
        .await?;

    let text = String::from_utf8_lossy(&res.output.stdout);
    let outcomes = parse_stream_json_lines(&text);

    let (ok, err, types) = summarize(&outcomes);
    println!("lines: ok={ok} err={err}");
    println!("type counts:");
    for (k, v) in types {
        println!("  {k}: {v}");
    }

    Ok(())
}
