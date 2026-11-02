//! Demonstrates `codex execpolicy check` via the wrapper.
//!
//! Note: not every Codex CLI release includes this command. When the binary is missing it,
//! this example prints a short skip message.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-codex --example execpolicy_check`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.

use std::{ffi::OsString, fs, path::PathBuf};

use codex::{CodexError, ExecPolicyCheckRequest};

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = real_cli::default_client();

    let dir = tempfile::tempdir()?;
    let policy = dir.path().join("policy.codexpolicy");
    fs::write(&policy, "")?;

    let request = ExecPolicyCheckRequest::new([OsString::from("echo"), OsString::from("ok")])
        .policy(PathBuf::from(&policy));

    match client.check_execpolicy(request).await {
        Ok(result) => {
            println!("decision: {:?}", result.decision());
            println!("{}", result.stdout);
        }
        Err(CodexError::NonZeroExit { stderr, .. })
            if stderr.to_lowercase().contains("unknown") =>
        {
            eprintln!("execpolicy check not available in this Codex binary; skipping.");
        }
        Err(err) => return Err(err.into()),
    }

    Ok(())
}
