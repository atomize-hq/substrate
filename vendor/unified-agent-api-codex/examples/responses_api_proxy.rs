use std::time::Duration;

use codex::{CodexClient, ResponsesApiProxyRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Maps to: echo "<API_KEY>" | codex responses-api-proxy [--port <PORT>] [--server-info <FILE>] [--http-shutdown] [--upstream-url <URL>]
    // Reads an API key from stdin, starts the proxy, and optionally parses the server-info JSON.
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let use_sample = take_flag(&mut args, "--sample");

    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("CODEX_API_KEY"))
        .unwrap_or_default();
    let missing_key = api_key.trim().is_empty();

    let server_info_dir = tempfile::tempdir()?;
    let server_info_path = server_info_dir.path().join("responses-api-proxy.json");

    if use_sample || missing_key {
        write_sample_server_info(&server_info_path)?;
        println!(
            "responses-api-proxy ({}) listening on 127.0.0.1:38483 (pid 1234); info at {}",
            if missing_key {
                "sample: missing API key"
            } else {
                "sample"
            },
            server_info_path.display(),
        );
        return Ok(());
    }

    let client = CodexClient::builder()
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let mut proxy = client
        .start_responses_api_proxy(
            ResponsesApiProxyRequest::new(api_key)
                .http_shutdown(true)
                .server_info(&server_info_path),
        )
        .await?;

    let mut info = None;
    for _ in 0..10 {
        match proxy.read_server_info().await {
            Ok(Some(found)) => {
                info = Some(found);
                break;
            }
            Ok(None) => break,
            Err(codex::CodexError::ResponsesApiProxyInfoRead { .. }) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(other) => return Err(other.into()),
        }
    }

    match info {
        Some(info) => println!(
            "responses-api-proxy listening on 127.0.0.1:{} (pid {})",
            info.port, info.pid
        ),
        None => println!("responses-api-proxy started (no server-info file was written)"),
    }

    if let Some(pid) = proxy.child.id() {
        println!("proxy pid: {pid}");
    }

    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = proxy.child.start_kill();
    let _ = proxy.child.wait().await;
    Ok(())
}

fn write_sample_server_info(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, r#"{ "port": 38483, "pid": 1234 }"#)?;
    Ok(())
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|value| value != flag);
    before != args.len()
}
