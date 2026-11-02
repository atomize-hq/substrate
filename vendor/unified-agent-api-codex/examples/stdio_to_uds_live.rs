#[cfg(not(unix))]
fn main() {
    eprintln!("This example is Unix-only (stdio-to-uds uses Unix domain sockets).");
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use codex::{CodexClient, StdioToUdsRequest};
    use std::{
        env,
        io::{Read, Write},
        os::unix::net::UnixListener,
        path::PathBuf,
        time::Duration,
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        sync::oneshot,
        time,
    };

    let binary = env::var_os("CODEX_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"));

    let temp = tempfile::tempdir()?;
    let socket_path = temp.path().join("bridge.sock");

    // Minimal server that echoes a reply back over the socket.
    let (ready_tx, ready_rx) = oneshot::channel();
    let server_path = socket_path.clone();
    let server = std::thread::spawn(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let listener = UnixListener::bind(&server_path)?;
            let _ = ready_tx.send(());

            let (mut stream, _) = listener.accept()?;
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf)?;
            if n > 0 {
                println!(
                    "[listener] received: {}",
                    String::from_utf8_lossy(&buf[..n]).trim_end()
                );
                stream.write_all(b"pong\n")?;
            }
            Ok(())
        },
    );

    ready_rx.await.ok();

    let client = CodexClient::builder()
        .binary(&binary)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let mut bridge = client
        .stdio_to_uds(StdioToUdsRequest::new(&socket_path))
        .expect("spawn stdio-to-uds");

    let mut stdin = bridge.stdin.take().ok_or("bridge stdin unavailable")?;
    let mut stdout =
        BufReader::new(bridge.stdout.take().ok_or("bridge stdout unavailable")?).lines();

    stdin.write_all(b"ping\n").await?;
    stdin.flush().await?;

    let echoed = time::timeout(Duration::from_secs(2), stdout.next_line())
        .await
        .map_err(|_| "timed out waiting for echoed data")??;

    match echoed {
        Some(line) => println!("[bridge] echoed: {}", line.trim_end()),
        None => println!("[bridge] no data echoed"),
    }

    let _ = bridge.start_kill();
    let _ = bridge.wait().await;
    let _ = server.join();
    Ok(())
}
