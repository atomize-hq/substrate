//! Demonstrates `--debug` and `--debug-file`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example print_debug_file -- "hello"`

use std::{env, error::Error, fs};

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
        real_cli::require_live("print_debug_file")?;
        return Ok(());
    }

    let prompt = collect_prompt()?;
    let work = real_cli::example_working_dir("print_debug_file")?;
    let debug_path = work.path().join("claude-debug.log");

    let client = real_cli::maybe_isolated_builder_with_mirroring("print_debug_file", false, false)?
        .working_dir(work.path())
        .build();

    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::Text)
                .debug(true)
                .debug_file(debug_path.to_string_lossy()),
        )
        .await?;

    println!("exit: {}", res.output.status);
    print!("{}", String::from_utf8_lossy(&res.output.stdout));

    match fs::read_to_string(&debug_path) {
        Ok(s) => {
            println!(
                "\n--- debug-file: {} ({} bytes) ---",
                debug_path.display(),
                s.len()
            );
            for line in s.lines().take(20) {
                println!("{line}");
            }
        }
        Err(e) => {
            eprintln!("no debug file found at {}: {e}", debug_path.display());
        }
    }

    Ok(())
}
