//! Platform-specific world detection and context (macOS + Linux compatibility shell-facing API).

#[cfg(target_os = "windows")]
pub mod windows;
use anyhow::Result;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use world_api::WorldBackend;

#[derive(Clone, Debug)]
pub enum WorldTransport {
    Unix(PathBuf),
    Tcp { host: String, port: u16 },
    Vsock { port: u16 },
}

impl fmt::Display for WorldTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorldTransport::Unix(p) => write!(f, "unix:{}", p.display()),
            WorldTransport::Tcp { host, port } => write!(f, "tcp:{}:{}", host, port),
            WorldTransport::Vsock { port } => write!(f, "vsock:{}", port),
        }
    }
}

pub struct PlatformWorldContext {
    pub backend: Arc<dyn WorldBackend>,
    pub transport: WorldTransport,
    #[allow(dead_code)]
    pub socket_path: PathBuf,
    pub ensure_ready: Box<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
}

static GLOBAL_CTX: OnceLock<Arc<PlatformWorldContext>> = OnceLock::new();

pub fn store_context_globally(ctx: PlatformWorldContext) {
    let _ = GLOBAL_CTX.set(Arc::new(ctx));
}

pub fn get_context() -> Option<Arc<PlatformWorldContext>> {
    GLOBAL_CTX.get().cloned()
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
    let ensure_ready = Box::new(move || {
        use world_api::{ResourceLimits, WorldBackend as _, WorldSpec};
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            always_isolate: false,
        };
        ensure_ready_backend.ensure_session(&spec).map(|_| ())
    });

    Ok(PlatformWorldContext {
        backend,
        transport,
        socket_path: default_sock,
        ensure_ready,
    })
}

#[cfg(target_os = "linux")]
pub fn detect() -> Result<PlatformWorldContext> {
    // Preserve Linux behavior: local world backend
    use world::LinuxLocalBackend;
    use world_api::{ResourceLimits, WorldBackend as _, WorldSpec};

    let backend = Arc::new(LinuxLocalBackend::new());
    let ensure_ready_backend = backend.clone();
    let ensure_ready = Box::new(move || {
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            always_isolate: false,
        };
        ensure_ready_backend.ensure_session(&spec).map(|_| ())
    });

    // Native Linux agent socket path
    let sock = PathBuf::from("/run/substrate.sock");
    let transport = WorldTransport::Unix(sock.clone());

    Ok(PlatformWorldContext {
        backend,
        transport,
        socket_path: sock,
        ensure_ready,
    })
}

#[cfg(test)]
mod tests {
    // tests placeholder (no mac-only overrides; parity with Linux)

    // No mac-only env overrides; parity with Linux
}
