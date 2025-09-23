use crate::bridge;
use crate::config::ForwarderConfig;
use anyhow::bail;
use std::ffi::c_void;
use std::io;
use std::sync::Arc;
use tokio::net::windows::named_pipe::{NamedPipeServer, PipeMode, ServerOptions};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, BOOL, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    GetSecurityDescriptorLength, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
};

const SECURITY_DESCRIPTOR: &str = "D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)";
const ERROR_PIPE_CONNECTED: i32 = 535;

type SecurityDescriptor = Vec<u8>;

#[derive(Clone, Debug)]
pub struct PipeListener {
    path: Arc<str>,
}

impl PipeListener {
    pub fn new(path: String) -> anyhow::Result<Self> {
        let normalized = normalize_path(path)?;
        Ok(Self {
            path: Arc::from(normalized),
        })
    }

    fn create_server(&self) -> io::Result<NamedPipeServer> {
        let mut descriptor =
            build_security_descriptor().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let mut attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.as_mut_ptr() as *mut c_void,
            bInheritHandle: BOOL::from(false),
        };
        let attrs_ptr: *mut c_void = (&mut attributes) as *mut SECURITY_ATTRIBUTES as *mut c_void;

        let server = unsafe {
            ServerOptions::new()
                .pipe_mode(PipeMode::Byte)
                .max_instances(64)
                .create_with_security_attributes_raw(&*self.path, attrs_ptr)?
        };
        Ok(server)
    }

    pub async fn accept(&self) -> io::Result<NamedPipeServer> {
        let server = self.create_server()?;
        match server.connect().await {
            Ok(()) => Ok(server),
            Err(err) if err.raw_os_error() == Some(ERROR_PIPE_CONNECTED) => Ok(server),
            Err(err) => Err(err),
        }
    }
}

fn normalize_path(path: String) -> anyhow::Result<String> {
    let trimmed = path.trim_start_matches('\\');
    if !trimmed.starts_with('.') {
        bail!(
            "named pipe path must start with {expected} (got {path})",
            expected = r"\\.\pipe\",
            path = path
        );
    }
    let after_dot = &trimmed[1..];
    let after_slash = after_dot.trim_start_matches('\\');
    if !after_slash.starts_with("pipe\\") {
        bail!(
            "named pipe path must start with {expected} (got {path})",
            expected = r"\\.\pipe\",
            path = path
        );
    }
    let rest = &after_slash["pipe\\".len()..];
    Ok(format!(r"\\.\pipe\{}", rest))
}

fn build_security_descriptor() -> anyhow::Result<SecurityDescriptor> {
    let mut wide: Vec<u16> = SECURITY_DESCRIPTOR.encode_utf16().collect();
    wide.push(0);

    let mut raw_descriptor = PSECURITY_DESCRIPTOR::default();
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(wide.as_ptr()),
            SDDL_REVISION_1,
            &mut raw_descriptor,
            None,
        )
        .map_err(|err| anyhow::Error::from(err))?;

        let ptr = raw_descriptor.0;
        let length = GetSecurityDescriptorLength(raw_descriptor) as usize;
        let slice = std::slice::from_raw_parts(ptr as *const u8, length);
        let mut buffer = Vec::with_capacity(length);
        buffer.extend_from_slice(slice);

        let _ = LocalFree(HLOCAL(ptr));
        Ok(buffer)
    }
}

pub async fn serve(config: Arc<ForwarderConfig>, cancel: CancellationToken) -> anyhow::Result<()> {
    let listener = PipeListener::new(config.pipe_path.clone())?;
    tracing::info!(pipe = %config.pipe_path, "listening on named pipe");

    let mut sessions: JoinSet<anyhow::Result<()>> = JoinSet::new();
    let mut counter: u64 = 0;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("pipe listener shutting down");
                break;
            }
            maybe_res = sessions.join_next() => {
                if let Some(res) = maybe_res {
                    match res {
                        Ok(Ok(())) => {}
                        Ok(Err(err)) => tracing::warn!(error = %err, "pipe session ended with error"),
                        Err(join_err) => tracing::warn!("pipe session panicked: {join_err}"),
                    }
                }
            }
            accept_res = listener.accept() => {
                match accept_res {
                    Ok(server) => {
                        counter = counter.wrapping_add(1);
                        let cfg = config.clone();
                        sessions.spawn(async move {
                            bridge::run_pipe_session(server, cfg, counter).await
                        });
                    }
                    Err(err) => {
                        tracing::error!(error = %err, "failed to accept pipe connection");
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }
            }
        }
    }

    while let Some(res) = sessions.join_next().await {
        match res {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::warn!(error = %err, "pipe session ended with error"),
            Err(join_err) => tracing::warn!("pipe session panicked: {join_err}"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

    #[tokio::test(flavor = "current_thread")]
    async fn pipe_listener_accepts_connection() {
        let pipe_name = format!(
            r"\\\\.\\pipe\\substrate-forwarder-test-{}",
            uuid::Uuid::now_v7()
        );
        let listener = PipeListener::new(pipe_name.clone()).expect("valid pipe");
        let client_path = listener.path.clone();

        let server_task = tokio::spawn(async move {
            let mut server = listener.accept().await.expect("accept");
            let mut buf = [0u8; 5];
            server.read_exact(&mut buf).await.expect("read");
            assert_eq!(&buf, b"hello");
            server.write_all(b"world").await.expect("write");
            io::Result::Ok(())
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut client = ClientOptions::new().open(&*client_path).expect("client");
        client.write_all(b"hello").await.expect("write");
        let mut buf = [0u8; 5];
        client.read_exact(&mut buf).await.expect("read");
        assert_eq!(&buf, b"world");

        server_task.await.unwrap().unwrap();
    }
}
