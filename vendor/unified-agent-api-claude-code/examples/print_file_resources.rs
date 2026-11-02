//! Demonstrates `--file <specs...>` (opt-in).
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 CLAUDE_EXAMPLE_FILE_SPECS=\"file_abc:doc.txt file_def:image.png\" cargo run -p unified-agent-api-claude-code --example print_file_resources -- \"describe the downloaded files\"`
//!
//! Notes:
//! - `--file` expects file resource specs in the form `file_id:relative_path`.

use std::{error::Error, fs};

use claude_code::ClaudeOutputFormat;

#[path = "support/real_cli.rs"]
mod real_cli;

fn split_whitespace_preserving_nonempty(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.to_string())
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("print_file_resources")?;
        return Ok(());
    }

    let specs =
        match real_cli::require_env(real_cli::ENV_EXAMPLE_FILE_SPECS, "print_file_resources") {
            Some(v) => v,
            None => return Ok(()),
        };
    let specs = split_whitespace_preserving_nonempty(&specs);
    if specs.is_empty() {
        eprintln!(
            "skipped print_file_resources: {} is empty",
            real_cli::ENV_EXAMPLE_FILE_SPECS
        );
        return Ok(());
    }

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Describe the downloaded files briefly.".to_string());

    let work = real_cli::example_working_dir("print_file_resources")?;
    let client =
        real_cli::maybe_isolated_builder_with_mirroring("print_file_resources", false, false)?
            .working_dir(work.path())
            .build();

    let res = client
        .print(
            real_cli::default_print_request(prompt)
                .output_format(ClaudeOutputFormat::Text)
                .files(specs),
        )
        .await?;

    println!("exit: {}", res.output.status);
    print!("{}", String::from_utf8_lossy(&res.output.stdout));

    // If the CLI downloaded files into the working directory, list a small preview.
    if let Ok(entries) = fs::read_dir(work.path()) {
        println!("\n--- working dir contents ---");
        for e in entries.flatten().take(50) {
            println!("{}", e.path().display());
        }
    }

    Ok(())
}
