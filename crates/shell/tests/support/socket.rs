#![cfg(unix)]

use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

/// Behavior for socket stubs spawned during tests.
#[derive(Clone, Copy)]
pub enum SocketResponse {
    /// Responds to `/v1/capabilities` requests with a JSON payload that
    /// advertises socket activation mode.
    Capabilities,
    /// Accepts connections but never returns a response (simulates a stuck
    /// systemd-managed socket where the service failed to start).
    Silent,
}

/// Minimal Unix socket server used to simulate socket-activated world-agent
/// listeners.
pub struct AgentSocket {
    path: PathBuf,
    shutdown: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl AgentSocket {
    /// Spawn a new stub server bound to the provided path.
    pub fn start(path: &Path, response: SocketResponse) -> Self {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create socket parent");
        }
        let _ = fs::remove_file(path);
        let listener = UnixListener::bind(path).expect("failed to bind stub socket");
        listener
            .set_nonblocking(true)
            .expect("failed to configure stub socket");

        let socket_path = path.to_path_buf();
        let cleanup_path = socket_path.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();

        let handle = thread::spawn(move || {
            while !shutdown_flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _addr)) => {
                        match response {
                            SocketResponse::Capabilities => {
                                let mut buf = [0u8; 2048];
                                let _ = stream.read(&mut buf);
                                let body = concat!(
                                    r#"{"version":"v1","features":["execute"],"#,
                                    r#"\"backend\":\"world-agent\",\"platform\":\"linux\","#,
                                    r#"\"listener_mode\":\"socket_activation\"}"#
                                );
                                let reply = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    body.len(),
                                    body
                                );
                                let _ = stream.write_all(reply.as_bytes());
                            }
                            SocketResponse::Silent => {
                                // Read and drop the request to simulate a hung service.
                                let mut buf = [0u8; 512];
                                let _ = stream.read(&mut buf);
                            }
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }

            let _ = fs::remove_file(&socket_path);
        });

        Self {
            path: cleanup_path,
            shutdown,
            handle: Some(handle),
        }
    }

    /// Return the on-disk socket path for the stub.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for AgentSocket {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = UnixStream::connect(&self.path);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
