//! Demonstrates wrapper-managed `CLAUDE_HOME` (app-scoped Claude CLI state).
//!
//! Usage:
//! - Use a fresh temp home (default):
//!   - `cargo run -p unified-agent-api-claude-code --example claude_home`
//! - Use a specific directory:
//!   - `cargo run -p unified-agent-api-claude-code --example claude_home -- --home /tmp/claude-home-demo`
//! - Seed from your current user profile (explicit opt-in):
//!   - `cargo run -p unified-agent-api-claude-code --example claude_home -- --seed minimal`
//!   - `cargo run -p unified-agent-api-claude-code --example claude_home -- --seed full`
//!
//! Notes:
//! - `--seed full` may copy large/sensitive browser/Electron profile data. Use only when needed.

use std::{env, error::Error, path::PathBuf};

use claude_code::{ClaudeClient, ClaudeHomeLayout, ClaudeHomeSeedLevel};
use tempfile::TempDir;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let mut home: Option<PathBuf> = None;
    let mut seed: Option<ClaudeHomeSeedLevel> = None;

    while let Some(a) = args.next() {
        match a.as_str() {
            "--home" => {
                let v = args.next().ok_or("missing value for --home")?;
                home = Some(PathBuf::from(v));
            }
            "--seed" => {
                let v = args.next().ok_or("missing value for --seed")?;
                seed = Some(match v.as_str() {
                    "minimal" => ClaudeHomeSeedLevel::MinimalAuth,
                    "full" => ClaudeHomeSeedLevel::FullProfile,
                    other => return Err(format!("unknown --seed value: {other}").into()),
                });
            }
            other => {
                return Err(format!("unknown arg: {other}").into());
            }
        }
    }

    let temp;
    let home = match home {
        Some(p) => p,
        None => {
            temp = TempDir::new()?;
            temp.path().join("claude-home")
        }
    };

    let mut builder = ClaudeClient::builder()
        .binary(real_cli::resolve_binary())
        .claude_home(&home);

    if let Some(level) = seed {
        builder = builder.seed_profile_from_current_user_home(level);
    }

    let client = builder.build();
    let layout = client
        .claude_home_layout()
        .unwrap_or_else(|| ClaudeHomeLayout::new(&home));

    println!("CLAUDE_HOME root: {}", layout.root().display());
    println!("xdg_config_home: {}", layout.xdg_config_home().display());
    println!("xdg_data_home:   {}", layout.xdg_data_home().display());
    println!("xdg_cache_home:  {}", layout.xdg_cache_home().display());
    println!(
        "macOS app support (under HOME): {}/Library/Application Support/Claude",
        layout.root().display()
    );

    let out = client.version().await?;
    print!("{}", String::from_utf8_lossy(&out.stdout));
    Ok(())
}
