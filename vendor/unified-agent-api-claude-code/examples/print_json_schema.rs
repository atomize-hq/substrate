//! Demonstrates `--output-format json` with `--json-schema`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_json_schema -- \"Return a name and a number\"`

use std::{env, error::Error};

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

fn collect_prompt() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("Provide a prompt string".into());
    }
    Ok(args.join(" "))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_json_schema")?;
        return Ok(());
    }

    let prompt = collect_prompt()?;
    let schema = r#"{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "count": { "type": "number" }
  },
  "required": ["name", "count"]
}"#;

    let client = real_cli::maybe_isolated_client("print_json_schema")?;
    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::Json)
                .json_schema(schema),
        )
        .await?;

    let v: serde_json::Value = serde_json::from_slice(&res.output.stdout)?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}
