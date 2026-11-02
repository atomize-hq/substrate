//! Demonstrates the multi-step `claude setup-token` flow via `ClaudeSetupTokenSession`.
//!
//! Usage:
//! - `CLAUDE_EXAMPLE_LIVE=1 cargo run -p unified-agent-api-claude-code --example setup_token_flow`
//! - Provide the code via env: `CLAUDE_SETUP_TOKEN_CODE=...`
//! - Or paste the code when prompted.
//!
//! Notes:
//! - This example uses the real `claude` binary and may require network/auth.
//! - Optional isolation: `CLAUDE_EXAMPLE_ISOLATED_HOME=1`

use std::{
    env,
    error::Error,
    io::{self, Write},
};

use claude_code::ClaudeSetupTokenRequest;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !real_cli::live_enabled() {
        real_cli::require_live("setup_token_flow")?;
        return Ok(());
    }

    // Do not mirror raw PTY output: `claude setup-token` emits terminal control sequences
    // (Ink UI) which looks noisy when mirrored. We capture output and print a clean URL.
    let client = real_cli::maybe_isolated_client_with_mirroring("setup_token_flow", false, false)?;
    let mut session = client
        .setup_token_start_with(ClaudeSetupTokenRequest::new().timeout(None))
        .await?;

    if let Some(url) = session
        .wait_for_url(std::time::Duration::from_secs(30))
        .await?
    {
        // Some terminals render stray control sequences if other output left the cursor mid-line.
        // Clear the current line before printing our own clean summary.
        clear_current_line();
        println!("Open this URL to authenticate:\n{url}");
        println!(
            "Complete the browser flow. If you're shown a one-time code, paste it below (Enter skips)."
        );
    } else {
        clear_current_line();
        println!("No OAuth URL detected yet.");
        println!(
            "If `claude` opened a browser window, complete the flow there. If you're shown a one-time code, paste it below."
        );
    }

    if let Some(code) = read_code()? {
        let out = session.submit_code(&code).await?;
        println!("exit: {}", out.status);
        print!("{}", String::from_utf8_lossy(&out.stdout));
        eprint!("{}", String::from_utf8_lossy(&out.stderr));
        return Ok(());
    }

    let out = session.wait().await?;
    println!("exit: {}", out.status);
    print!("{}", String::from_utf8_lossy(&out.stdout));
    eprint!("{}", String::from_utf8_lossy(&out.stderr));
    Ok(())
}

fn clear_current_line() {
    // Best-effort ANSI line clear; ignore errors for non-tty stdout.
    let mut out = std::io::stdout().lock();
    let _ = write!(out, "\r\x1b[2K");
    let _ = out.flush();
}

fn read_code() -> Result<Option<String>, Box<dyn Error>> {
    if let Ok(code) = env::var("CLAUDE_SETUP_TOKEN_CODE") {
        if !code.trim().is_empty() {
            return Ok(Some(code));
        }
    }

    println!("If prompted for a code, paste it here and press Enter (or press Enter to skip):");
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let code = line.trim().to_string();
    Ok((!code.is_empty()).then_some(code))
}
