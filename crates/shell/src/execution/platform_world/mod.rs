//! Platform-specific world detection and context (macOS + Linux compatibility shell-facing API).

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub fn detect() -> Result<PlatformWorldContext> {
    windows::detect()
}

#[cfg(not(target_os = "windows"))]
use crate::execution::policy_snapshot::bootstrap_world_spec;
#[cfg(not(target_os = "windows"))]
use crate::execution::settings;
use agent_api_types::SharedWorldOwnerSpec;
use anyhow::{Context, Result};
use std::fmt;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use world_api::WorldBackend;

#[derive(Clone, Debug)]
pub enum WorldTransport {
    Unix(PathBuf),
    Tcp {
        host: String,
        port: u16,
    },
    Vsock {
        port: u16,
    },
    #[cfg(target_os = "windows")]
    NamedPipe(PathBuf),
}

#[cfg(target_os = "windows")]
const _: Option<WorldTransport> = Some(WorldTransport::Vsock { port: 0 });

impl fmt::Display for WorldTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorldTransport::Unix(p) => write!(f, "unix:{}", p.display()),
            WorldTransport::Tcp { host, port } => write!(f, "tcp:{}:{}", host, port),
            WorldTransport::Vsock { port } => write!(f, "vsock:{}", port),
            #[cfg(target_os = "windows")]
            WorldTransport::NamedPipe(p) => write!(f, "pipe:{}", p.display()),
        }
    }
}

pub struct PlatformWorldContext {
    pub backend: Arc<dyn WorldBackend>,
    pub transport: WorldTransport,
    #[allow(dead_code)]
    pub socket_path: PathBuf,
    pub ensure_ready: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    #[allow(dead_code)]
    pub ensure_persistent_session_ready_async: Box<PersistentSessionReadyFn>,
}

pub type PersistentSessionReadyFuture =
    Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'static>>;
pub type PersistentSessionReadyFn = dyn Fn() -> PersistentSessionReadyFuture + Send + Sync;

static GLOBAL_CTX: OnceLock<Arc<PlatformWorldContext>> = OnceLock::new();

fn validate_shared_owner_request_support(
    request: Option<&SharedWorldOwnerSpec>,
    operation: &str,
) -> Result<()> {
    let Some(request) = request else {
        return Ok(());
    };

    #[cfg(target_os = "linux")]
    {
        let _ = operation;
        let _ = request;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        if std::env::var_os("SUBSTRATE_WORLD_SOCKET").is_some() {
            anyhow::bail!(
                "{} rejects explicit shared-owner world reuse when SUBSTRATE_WORLD_SOCKET overrides the Lima-backed transport (orchestration_session_id={})",
                operation,
                request.orchestration_session_id
            );
        }

        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        anyhow::bail!(
            "{} rejects explicit shared-owner world reuse on this platform (orchestration_session_id={})",
            operation,
            request.orchestration_session_id
        );
    }
}

pub(crate) fn reject_non_linux_shared_owner_request(
    request: Option<&SharedWorldOwnerSpec>,
    operation: &str,
) -> Result<()> {
    validate_shared_owner_request_support(request, operation)
}

pub(crate) fn with_supported_shared_world_request<T, F>(
    request: Option<&SharedWorldOwnerSpec>,
    operation: &str,
    on_supported: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    validate_shared_owner_request_support(request, operation)?;
    on_supported()
}

pub fn store_context_globally(ctx: PlatformWorldContext) {
    let _ = GLOBAL_CTX.set(Arc::new(ctx));
}

pub fn get_context() -> Option<Arc<PlatformWorldContext>> {
    GLOBAL_CTX.get().cloned()
}

#[allow(dead_code)]
pub async fn ensure_persistent_session_ready_async(ctx: &PlatformWorldContext) -> Result<()> {
    (ctx.ensure_persistent_session_ready_async.as_ref())().await
}

#[cfg(target_os = "macos")]
pub fn detect() -> Result<PlatformWorldContext> {
    use world_mac_lima::MacLimaBackend;

    // Default UDS path on host for SSH UDS forwarding
    let default_sock = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".substrate/sock/agent.sock");

    // Auto-detect transport preference (do not start VM/tunnels here)
    let transport_pref = world_mac_lima::Transport::auto_select().unwrap_or_default();

    let transport = match transport_pref {
        world_mac_lima::Transport::UnixSocket => WorldTransport::Unix(default_sock.clone()),
        // VSock is proxied to local TCP port by vsock-proxy (host loopback)
        world_mac_lima::Transport::VSock => WorldTransport::Vsock { port: 17788 },
        // TCP fallback (if ever used) will be loopback
        world_mac_lima::Transport::TCP => WorldTransport::Tcp {
            host: "127.0.0.1".into(),
            port: 17788,
        },
    };

    let backend = Arc::new(MacLimaBackend::new()?);
    let ensure_ready_backend = backend.clone();
    let ensure_persistent_session_ready_async_backend = backend.clone();
    let ensure_ready = Box::new(move || {
        use world_api::WorldBackend as _;
        let spec = bootstrap_world_spec(
            settings::world_root_from_env().path,
            substrate_broker::world_fs_mode(),
        );
        ensure_ready_backend.ensure_session(&spec).map(|_| ())
    });
    let ensure_persistent_session_ready_async = Box::new(move || {
        let backend = ensure_persistent_session_ready_async_backend.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                use world_api::WorldBackend as _;
                let spec = bootstrap_world_spec(
                    settings::world_root_from_env().path,
                    substrate_broker::world_fs_mode(),
                );
                backend.ensure_session(&spec).map(|_| ())
            })
            .await
            .context("persistent-session readiness join failure")?
        }) as PersistentSessionReadyFuture
    });

    Ok(PlatformWorldContext {
        backend,
        transport,
        socket_path: default_sock,
        ensure_ready,
        ensure_persistent_session_ready_async,
    })
}

#[cfg(target_os = "linux")]
pub fn detect() -> Result<PlatformWorldContext> {
    // Preserve Linux behavior: local world backend
    use world::LinuxLocalBackend;
    use world_api::WorldBackend as _;

    let backend = Arc::new(LinuxLocalBackend::new());
    let ensure_ready_backend = backend.clone();
    let ensure_persistent_session_ready_async_backend = backend.clone();
    let ensure_ready = Box::new(move || {
        let spec = bootstrap_world_spec(
            settings::world_root_from_env().path,
            substrate_broker::world_fs_mode(),
        );
        ensure_ready_backend.ensure_session(&spec).map(|_| ())
    });
    let ensure_persistent_session_ready_async = Box::new(move || {
        let backend = ensure_persistent_session_ready_async_backend.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                use world_api::WorldBackend as _;
                let spec = bootstrap_world_spec(
                    settings::world_root_from_env().path,
                    substrate_broker::world_fs_mode(),
                );
                backend.ensure_session(&spec).map(|_| ())
            })
            .await
            .context("persistent-session readiness join failure")?
        }) as PersistentSessionReadyFuture
    });

    // Native Linux agent socket path
    let sock = PathBuf::from("/run/substrate.sock");
    let transport = WorldTransport::Unix(sock.clone());

    Ok(PlatformWorldContext {
        backend,
        transport,
        socket_path: sock,
        ensure_ready,
        ensure_persistent_session_ready_async,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn vsock_variant_displays_port() {
        let transport = WorldTransport::Vsock { port: 17788 };
        assert_eq!(transport.to_string(), "vsock:17788");
    }

    #[test]
    fn unix_and_tcp_transports_format_endpoints() {
        let unix = WorldTransport::Unix(PathBuf::from("/tmp/substrate.sock"));
        assert_eq!(unix.to_string(), "unix:/tmp/substrate.sock");

        let tcp = WorldTransport::Tcp {
            host: "127.0.0.1".into(),
            port: 9001,
        };
        assert_eq!(tcp.to_string(), "tcp:127.0.0.1:9001");
    }
}
