//! Demonstrates the spawned stdio server helpers for `codex app-server proxy` and
//! `codex exec-server`.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-codex --example server_helpers -- app-proxy [SOCK_PATH]`
//! - `cargo run -p unified-agent-api-codex --example server_helpers -- exec-server [LISTEN_ADDR]`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.

use std::{env, time::Duration};

use codex::{AppServerProxyRequest, ExecServerRequest};

#[path = "support/real_cli.rs"]
mod real_cli;

fn usage() {
    eprintln!("usage:");
    eprintln!("  server_helpers app-proxy [SOCK_PATH]");
    eprintln!("  server_helpers exec-server [LISTEN_ADDR]");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(command) = args.first() else {
        usage();
        return Ok(());
    };

    let client = real_cli::default_client();

    match command.as_str() {
        "app-proxy" => {
            let temp = tempfile::tempdir()?;
            let socket_path = args
                .get(1)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| temp.path().join("app-server.sock"));
            let mut child = client
                .start_app_server_proxy(AppServerProxyRequest::new().socket_path(&socket_path))?;
            println!(
                "spawned app-server proxy pid={:?} sock={}",
                child.id(),
                socket_path.display()
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
        "exec-server" => {
            let mut request = ExecServerRequest::new();
            if let Some(listen) = args.get(1) {
                request = request.listen(listen.clone());
            }
            let mut child = client.start_exec_server(request)?;
            println!("spawned exec-server pid={:?}", child.id());
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
        _ => usage(),
    }

    Ok(())
}
