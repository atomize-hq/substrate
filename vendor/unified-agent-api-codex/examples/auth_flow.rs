//! Demonstrates `codex login`, `codex login status`, and `codex logout` via the wrapper.
//!
//! Usage:
//! - Status (default): `cargo run -p unified-agent-api-codex --example auth_flow`
//! - Interactive login: `cargo run -p unified-agent-api-codex --example auth_flow -- login`
//! - API key login: `CODEX_API_KEY=... cargo run -p unified-agent-api-codex --example auth_flow -- login-api-key`
//! - Logout: `cargo run -p unified-agent-api-codex --example auth_flow -- logout`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.
//! - `CODEX_API_KEY` or `OPENAI_API_KEY` for `login-api-key`.

use std::env;

#[path = "support/real_cli.rs"]
mod real_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let subcommand = args.next().unwrap_or_else(|| "status".to_string());

    let client = real_cli::default_client();

    match subcommand.as_str() {
        "status" => {
            let status = client.login_status().await?;
            println!("login status: {:?}", status);
        }
        "login" => {
            let mut child = client.spawn_login_process()?;
            let status = child.wait().await?;
            println!("login exited: {status}");
        }
        "login-api-key" => {
            let key = env::var("CODEX_API_KEY")
                .or_else(|_| env::var("OPENAI_API_KEY"))
                .map_err(|_| "set CODEX_API_KEY (or OPENAI_API_KEY) for login-api-key")?;
            let status = client.login_with_api_key(key).await?;
            println!("api-key login status: {:?}", status);
        }
        "logout" => {
            let status = client.logout().await?;
            println!("logout status: {:?}", status);
        }
        other => {
            eprintln!("unknown subcommand: {other} (expected status/login/login-api-key/logout)");
        }
    }

    Ok(())
}
