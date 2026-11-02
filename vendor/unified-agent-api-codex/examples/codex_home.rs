//! Run Codex with an app-scoped `CODEX_HOME` so state stays isolated.
//!
//! Set `CODEX_HOME` to a writable directory (or pass nothing to use a temp dir):
//! ```bash
//! CODEX_HOME=/tmp/codex-demo \
//!   cargo run -p unified-agent-api-codex --example codex_home -- "List tmp files"
//! ```
//!
//! Codex will read/write under `CODEX_HOME` (config.toml, auth.json, .credentials.json,
//! history.jsonl, conversations/*.jsonl, logs/codex-*.log). This example prints the
//! selected directory before invoking Codex.

use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use codex::CodexClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt();
    let codex_home = select_home_dir();

    fs::create_dir_all(&codex_home)?;
    env::set_var("CODEX_HOME", &codex_home);
    println!("CODEX_HOME set to {}", codex_home.display());
    print_known_paths(&codex_home);

    let client = CodexClient::builder()
        .timeout(Duration::from_secs(45))
        .build();

    match client.send_prompt(&prompt).await {
        Ok(response) => println!("Codex replied:\n{response}"),
        Err(error) => {
            eprintln!("Codex invocation failed: {error}");
            eprintln!("Confirm the binary is installed and CODEX_HOME is writable.");
        }
    }

    Ok(())
}

fn collect_prompt() -> String {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        "Show the files under CODEX_HOME".to_string()
    } else {
        args.join(" ")
    }
}

fn select_home_dir() -> PathBuf {
    env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("codex-home-demo"))
}

fn print_known_paths(home: &Path) {
    println!("Codex will use:");
    println!("  - config: {}", home.join("config.toml").display());
    println!("  - auth: {}", home.join("auth.json").display());
    println!(
        "  - credentials: {}",
        home.join(".credentials.json").display()
    );
    println!("  - history: {}", home.join("history.jsonl").display());
    println!(
        "  - conversations: {}",
        home.join("conversations").display()
    );
    println!("  - logs: {}", home.join("logs").display());
}
