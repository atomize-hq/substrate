use crate::bridge;
use crate::config::ForwarderConfig;
use anyhow::anyhow;
use std::ffi::c_void;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, PipeMode, ServerOptions};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use transport_api_types::normalize_windows_pipe_path;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, BOOL, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    GetSecurityDescriptorLength, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
};

const SECURITY_DESCRIPTOR: &str = "D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;OW)(A;;GA;;;IU)";
const ERROR_PIPE_CONNECTED: i32 = 535;

#[derive(Debug)]
struct PipeInner {
    path: Arc<str>,
    first_instance: AtomicBool,
}

type SecurityDescriptor = Vec<u8>;

#[derive(Clone, Debug)]
pub struct PipeListener {
    inner: Arc<PipeInner>,
}

impl PipeListener {
    pub fn new(config: &ForwarderConfig) -> anyhow::Result<Self> {
        let verified = bridge::verify_forwarder_projection(config, None)?;
        let normalized = normalize_path(verified.pipe_path)?;
        Ok(Self {
            inner: Arc::new(PipeInner {
                path: Arc::from(normalized),
                first_instance: AtomicBool::new(true),
            }),
        })
    }

    fn create_server(&self) -> io::Result<NamedPipeServer> {
        let mut descriptor = build_security_descriptor().map_err(io::Error::other)?;
        let mut attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.as_mut_ptr() as *mut c_void,
            bInheritHandle: BOOL::from(false),
        };
        let attrs_ptr: *mut c_void = (&mut attributes) as *mut SECURITY_ATTRIBUTES as *mut c_void;

        let mut options = ServerOptions::new();
        if self.inner.first_instance.swap(false, Ordering::SeqCst) {
            options.first_pipe_instance(true);
        }
        options.reject_remote_clients(true);
        options.pipe_mode(PipeMode::Byte);
        options.max_instances(64);

        let server = unsafe {
            match options.create_with_security_attributes_raw(&*self.inner.path, attrs_ptr) {
                Ok(s) => Ok(s),
                Err(err) => {
                    if err.raw_os_error() == Some(5)
                        || err.kind() == io::ErrorKind::PermissionDenied
                    {
                        Err(io::Error::new(
                            io::ErrorKind::AddrInUse,
                            format!(
                                "another process already owns named pipe {} (ACCESS_DENIED)",
                                &*self.inner.path
                            ),
                        ))
                    } else {
                        Err(err)
                    }
                }
            }
        }?;
        Ok(server)
    }

    pub fn create_listening_instance(&self) -> io::Result<NamedPipeServer> {
        let server = self.create_server()?;
        tracing::info!(pipe = %self.inner.path, "named pipe instance ready");
        Ok(server)
    }

    pub fn detect_existing_server(&self) -> bool {
        match ClientOptions::new().open(&*self.inner.path) {
            Ok(client) => {
                drop(client);
                true
            }
            Err(_) => false,
        }
    }

    pub async fn accept_with(
        &self,
        server: NamedPipeServer,
    ) -> io::Result<(NamedPipeServer, NamedPipeServer)> {
        match server.connect().await {
            Ok(()) => {}
            Err(err) if err.raw_os_error() == Some(ERROR_PIPE_CONNECTED) => {}
            Err(err) => return Err(err),
        }

        let next = self.create_listening_instance()?;
        Ok((server, next))
    }

    #[cfg(test)]
    fn pipe_path(&self) -> Arc<str> {
        Arc::clone(&self.inner.path)
    }
}

fn normalize_path(path: String) -> anyhow::Result<String> {
    normalize_windows_pipe_path(&path).map_err(|_| {
        anyhow!(
            "named pipe path must use the canonical local form {expected} (got {path})",
            expected = r"\\.\pipe\",
            path = path
        )
    })
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
        .map_err(anyhow::Error::from)?;

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
    let listener = PipeListener::new(&config)?;
    tracing::info!(
        pipe = %listener.inner.path,
        target_mode = config.target_mode(),
        target = %config.target(),
        "listening on named pipe"
    );

    // Single-instance preflight: if another server is already listening, exit with a friendly error
    if listener.detect_existing_server() {
        tracing::error!(
            pipe = %config.pipe_path,
            "another forwarder appears to be serving this pipe; run wsl-stop.ps1 or choose a different --pipe/-PipePath"
        );
        return Err(anyhow!(
            "pipe {} already has a server; stop it or choose a different name",
            config.pipe_path
        ));
    }

    let mut sessions: JoinSet<anyhow::Result<()>> = JoinSet::new();
    let mut counter: u64 = 0;
    let initial_server = listener.create_listening_instance()?;
    let mut accept_fut = Box::pin(listener.accept_with(initial_server));

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
                        Ok(Err(err)) => tracing::warn!(error = ?err, "pipe session ended with error"),
                        Err(join_err) => tracing::warn!("pipe session panicked: {join_err}"),
                    }
                }
            }
            res = &mut accept_fut => {
                match res {
                    Ok((server, next)) => {
                        counter = counter.wrapping_add(1);
                        tracing::debug!(session = counter, "accepted named pipe client");
                        let cfg = config.clone();
                        sessions.spawn(async move {
                            bridge::run_pipe_session(server, cfg, counter).await
                        });
                        accept_fut = Box::pin(listener.accept_with(next));
                    }
                    Err(err) => {
                        tracing::error!(error = ?err, "failed to accept pipe connection");
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let retry = listener.create_listening_instance()?;
                        accept_fut = Box::pin(listener.accept_with(retry));
                    }
                }
            }
        }
    }

    while let Some(res) = sessions.join_next().await {
        match res {
            Ok(Ok(())) => {}
            Ok(Err(err)) => tracing::warn!(error = ?err, "pipe session ended with error"),
            Err(join_err) => tracing::warn!("pipe session panicked: {join_err}"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::{lock_test_projection_env_guard, set_test_projection_env};
    use std::path::PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
        WindowsForwarderScopeV1,
    };

    use crate::config::{
        set_test_current_windows_host_observation, set_test_file_settings_override,
    };

    fn sample_internal_config(pipe_path: &str) -> ForwarderConfig {
        set_test_file_settings_override(Some(None));
        set_test_current_windows_host_observation(Some(Ok((
            r"ACME\Alice".to_string(),
            "S-1-5-21-1000".to_string(),
            r"C:\Users\Alice\AppData\Local".to_string(),
        ))));

        let host_carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_windows(
                r"C:\Users\Alice\AppData\Local\Substrate",
                r"ACME\Alice",
                "S-1-5-21-1000",
            )
            .unwrap(),
        )
        .unwrap();
        let scope = WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            pipe_path,
        )
        .unwrap();
        let mapping = PlatformBootstrapMappingV1::new_wsl(
            &host_carrier,
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            &format!(
                r"C:\Users\Alice\AppData\Local\Substrate\forwarder\{}",
                scope.0
            ),
            "/home/substrate/.substrate",
            "substrate",
            1000,
            pipe_path,
            "/run/substrate.sock",
        )
        .unwrap();

        ForwarderConfig::load_internal(
            "Substrate-WSL".to_string(),
            pipe_path.to_string(),
            None,
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\forwarder\forwarder.toml"),
            PathBuf::from(r"C:\Users\Alice\AppData\Local\Substrate\forwarder\logs"),
            &host_carrier.encode().unwrap(),
            &mapping.encode(&host_carrier).unwrap(),
        )
        .unwrap()
    }

    fn reset_test_state() {
        set_test_current_windows_host_observation(None);
        set_test_file_settings_override(None);
    }

    #[test]
    fn normalize_path_canonicalizes_windows_pipe_aliases() {
        assert_eq!(
            normalize_path(r"\\.\pipe\Substrate-Agent".to_string()).unwrap(),
            r"\\.\pipe\substrate-agent"
        );
        assert_eq!(
            normalize_path(r"\\.\pipe\SUBSTRATE-AGENT".to_string()).unwrap(),
            r"\\.\pipe\substrate-agent"
        );
    }

    #[test]
    fn normalize_path_rejects_invalid_pipe_forms() {
        for invalid in [
            r"",
            r"\\.\pipe\",
            r"//./pipe/substrate-agent",
            r"\\server\pipe\substrate-agent",
            r"\\.\pipe\nested\name",
            r"\\.\pipe\has space",
            "\\\\.\\pipe\\line\nbreak",
            r"\\.\pipe\name$",
        ] {
            let result = normalize_path(invalid.to_string());
            assert!(result.is_err(), "invalid pipe should reject: {invalid}");
        }
    }

    #[test]
    fn pipe_listener_requires_authenticated_projection_before_bind() {
        let _guard = lock_test_projection_env_guard();
        let config = sample_internal_config(r"\\.\pipe\substrate-agent");
        let _projection = set_test_projection_env(
            Some(
                &InstallBootstrapContextCarrierV1::from_context(
                    InstallBootstrapContextV1::new_windows(
                        r"C:\Users\Alice\AppData\Local\Substrate",
                        r"ACME\Alice",
                        "S-1-5-21-1000",
                    )
                    .unwrap(),
                )
                .unwrap()
                .encode()
                .unwrap(),
            ),
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"),
        );
        let listener = PipeListener::new(&config).unwrap();
        assert_eq!(&*listener.pipe_path(), r"\\.\pipe\substrate-agent");

        let _projection = set_test_projection_env(None, None);
        let err = PipeListener::new(&config).unwrap_err();
        assert!(
            err.to_string()
                .contains("requires a projected install bootstrap carrier"),
            "unexpected error: {err}"
        );

        reset_test_state();
    }

    #[test]
    fn pipe_listener_preserves_legacy_forwarder_startup() {
        let _guard = lock_test_projection_env_guard();
        set_test_file_settings_override(Some(None));

        let config = ForwarderConfig::load(
            "Substrate-WSL".to_string(),
            r"\\.\pipe\Substrate-Agent".to_string(),
            None,
            None,
        )
        .unwrap();
        let listener = PipeListener::new(&config).unwrap();
        assert_eq!(&*listener.pipe_path(), r"\\.\pipe\substrate-agent");

        reset_test_state();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pipe_listener_accepts_connection() {
        let _guard = lock_test_projection_env_guard();
        let pipe_name = format!(
            r"\\.\pipe\substrate-forwarder-test-{}",
            uuid::Uuid::now_v7()
        );
        let config = sample_internal_config(&pipe_name);
        let _projection = set_test_projection_env(
            Some(
                &InstallBootstrapContextCarrierV1::from_context(
                    InstallBootstrapContextV1::new_windows(
                        r"C:\Users\Alice\AppData\Local\Substrate",
                        r"ACME\Alice",
                        "S-1-5-21-1000",
                    )
                    .unwrap(),
                )
                .unwrap()
                .encode()
                .unwrap(),
            ),
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"),
        );
        let listener = PipeListener::new(&config).expect("valid pipe");
        let client_path = listener.pipe_path();
        let pending = listener
            .create_listening_instance()
            .expect("create pending");
        let listener_clone = listener.clone();

        let server_task = tokio::spawn(async move {
            let (mut server, _next) = listener_clone.accept_with(pending).await.expect("accept");
            let mut buf = [0u8; 5];
            server.read_exact(&mut buf).await.expect("read");
            assert_eq!(&buf, b"hello");
            server.write_all(b"world").await.expect("write");
            server.flush().await.expect("flush");
            server.disconnect().expect("disconnect");
            io::Result::Ok(())
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut client = ClientOptions::new().open(&*client_path).expect("client");
        client.write_all(b"hello").await.expect("write");
        let mut buf = [0u8; 5];
        client.read_exact(&mut buf).await.expect("read");
        assert_eq!(&buf, b"world");

        server_task.await.unwrap().unwrap();
        reset_test_state();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn pipe_listener_handles_multiple_clients() {
        let _guard = lock_test_projection_env_guard();
        let pipe_name = format!(
            r"\\.\pipe\substrate-forwarder-test-multi-{}",
            uuid::Uuid::now_v7()
        );
        let config = sample_internal_config(&pipe_name);
        let _projection = set_test_projection_env(
            Some(
                &InstallBootstrapContextCarrierV1::from_context(
                    InstallBootstrapContextV1::new_windows(
                        r"C:\Users\Alice\AppData\Local\Substrate",
                        r"ACME\Alice",
                        "S-1-5-21-1000",
                    )
                    .unwrap(),
                )
                .unwrap()
                .encode()
                .unwrap(),
            ),
            Some("3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7"),
        );
        let listener = PipeListener::new(&config).expect("valid pipe");
        let client_path = listener.pipe_path();
        let pending = listener
            .create_listening_instance()
            .expect("create pending");
        let listener_clone = listener.clone();

        let server_task = tokio::spawn(async move {
            let (mut first, next) = listener_clone
                .accept_with(pending)
                .await
                .expect("accept first");
            let mut buf = [0u8; 3];
            first.read_exact(&mut buf).await.expect("read first");
            assert_eq!(&buf, b"one");
            first.write_all(b"uno").await.expect("write first");
            first.flush().await.expect("flush first");
            first.disconnect().expect("disconnect first");

            let (mut second, _final_next) = listener_clone
                .accept_with(next)
                .await
                .expect("accept second");
            let mut buf2 = [0u8; 3];
            second.read_exact(&mut buf2).await.expect("read second");
            assert_eq!(&buf2, b"two");
            second.write_all(b"dos").await.expect("write second");
            second.flush().await.expect("flush second");
            second.disconnect().expect("disconnect second");

            io::Result::Ok(())
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut client_one = ClientOptions::new()
            .open(&*client_path)
            .expect("client one");
        client_one.write_all(b"one").await.expect("write one");
        let mut buf = [0u8; 3];
        client_one.read_exact(&mut buf).await.expect("read uno");
        assert_eq!(&buf, b"uno");
        drop(client_one);

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let mut client_two = ClientOptions::new()
            .open(&*client_path)
            .expect("client two");
        client_two.write_all(b"two").await.expect("write two");
        let mut buf2 = [0u8; 3];
        client_two.read_exact(&mut buf2).await.expect("read dos");
        assert_eq!(&buf2, b"dos");

        server_task.await.unwrap().unwrap();
        reset_test_state();
    }
}
