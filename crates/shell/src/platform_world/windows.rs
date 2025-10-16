use super::{PlatformWorldContext, WorldTransport};
use crate::Cli;
use agent_api_client::{AgentClient, Transport};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::{Arc, Once, OnceLock};
use world_api::{ResourceLimits, WorldBackend, WorldSpec};
use world_windows_wsl::WindowsWslBackend;

struct WindowsContext {
    backend: Arc<WindowsWslBackend>,
    backend_trait: Arc<dyn WorldBackend>,
}

static CONTEXT: OnceLock<WindowsContext> = OnceLock::new();

fn context() -> Result<&'static WindowsContext> {
    if let Some(ctx) = CONTEXT.get() {
        return Ok(ctx);
    }

    let backend = Arc::new(WindowsWslBackend::new()?);
    let backend_trait: Arc<dyn WorldBackend> = backend.clone();
    let ctx = WindowsContext {
        backend,
        backend_trait,
    };

    let _ = CONTEXT.set(ctx);
    Ok(CONTEXT.get().expect("windows context initialized"))
}

fn to_world_transport(transport: &Transport) -> WorldTransport {
    match transport {
        Transport::UnixSocket { path } => WorldTransport::Unix(path.clone()),
        Transport::Tcp { host, port } => WorldTransport::Tcp {
            host: host.clone(),
            port: *port,
        },
        Transport::NamedPipe { path } => WorldTransport::NamedPipe(path.clone()),
    }
}

fn socket_path_from_transport(transport: &Transport) -> PathBuf {
    match transport {
        Transport::UnixSocket { path } => path.clone(),
        Transport::NamedPipe { path } => path.clone(),
        Transport::Tcp { .. } => PathBuf::new(),
    }
}
pub fn ensure_world_ready(cli: &Cli) -> Result<Option<String>> {
    ensure_world_ready_impl(cli, get_backend)
}

fn ensure_world_ready_impl<F>(cli: &Cli, backend_provider: F) -> Result<Option<String>>
where
    F: FnOnce() -> Result<Arc<dyn WorldBackend>>,
{
    if cli.no_world {
        return Ok(None);
    }

    let world_env = std::env::var("SUBSTRATE_WORLD").unwrap_or_default();
    if world_env == "disabled" {
        return Ok(None);
    }

    let backend = backend_provider()?;
    let spec = world_spec();
    match backend.ensure_session(&spec) {
        Ok(handle) => {
            std::env::set_var("SUBSTRATE_WORLD", "enabled");
            std::env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
            Ok(Some(handle.id))
        }
        Err(err) => {
            warn_once(format!(
                "substrate: warn: windows world ensure_session failed: {err}"
            ));
            Ok(None)
        }
    }
}

pub fn get_backend() -> Result<Arc<dyn WorldBackend>> {
    Ok(context()?.backend_trait.clone())
}

pub fn world_spec() -> WorldSpec {
    WorldSpec {
        reuse_session: true,
        isolate_network: true,
        limits: ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: substrate_broker::allowed_domains(),
        project_dir: current_dir(),
        always_isolate: false,
    }
}

fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn warn_once(message: String) {
    static WARN_ONCE: Once = Once::new();
    WARN_ONCE.call_once(move || {
        eprintln!("{}", message);
    });
}

pub fn detect() -> Result<PlatformWorldContext> {
    let ctx = context()?;
    let backend = ctx.backend_trait.clone();
    let transport_config = ctx.backend.agent_transport();
    let transport = to_world_transport(&transport_config);
    let socket_path = socket_path_from_transport(&transport_config);
    let ensure_backend = ctx.backend_trait.clone();
    let ensure_ready = Box::new(move || {
        let spec = world_spec();
        ensure_backend.ensure_session(&spec).map(|_| ())
    });

    Ok(PlatformWorldContext {
        backend,
        transport,
        socket_path,
        ensure_ready,
    })
}

pub fn build_agent_client() -> Result<AgentClient> {
    let ctx = context()?;
    Ok(ctx.backend.build_agent_client())
}

/// Convert a host Windows path to the corresponding WSL path string using the active backend.
pub fn to_wsl_path_string(path: &std::path::Path) -> Result<String> {
    // If relative, resolve against current_dir then convert
    let path = if path.is_relative() {
        let joined = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path);
        joined
    } else {
        path.to_path_buf()
    };
    let raw = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path is not valid UTF-8: {}", path.display()))?;
    let normalized = raw.replace('\\', "/");
    if normalized.starts_with("//") {
        // UNC path: //server/share/dir -> /mnt/unc/server/share/dir
        let rest = normalized.trim_start_matches('/');
        Ok(format!("/mnt/unc/{}", rest))
    } else if let Some((drive, rest)) = normalized.split_once(':') {
        // Drive letter path: C:/foo -> /mnt/c/foo
        let rest = rest.trim_start_matches('/');
        Ok(format!("/mnt/{}/{}", drive.to_lowercase(), rest))
    } else {
        // Already a Unix-style path
        Ok(normalized)
    }
}

/// Convert current working directory to a WSL path string.
pub fn current_dir_wsl() -> Result<String> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    to_wsl_path_string(&cwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use clap::Parser;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use world_api::{ExecResult, FsDiff, WorldHandle};

    #[derive(Clone)]
    struct StubBackend {
        handle: WorldHandle,
        ensure_calls: Arc<AtomicUsize>,
    }

    impl StubBackend {
        fn new(id: &str, ensure_calls: Arc<AtomicUsize>) -> Self {
            Self {
                handle: WorldHandle { id: id.to_string() },
                ensure_calls,
            }
        }
    }

    impl WorldBackend for StubBackend {
        fn ensure_session(&self, _spec: &WorldSpec) -> Result<WorldHandle> {
            self.ensure_calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.handle.clone())
        }

        fn exec(&self, _world: &WorldHandle, _req: ExecRequest) -> Result<ExecResult> {
            Err(anyhow!("exec not implemented in stub"))
        }

        fn fs_diff(&self, _world: &WorldHandle, _span_id: &str) -> Result<FsDiff> {
            Ok(FsDiff::default())
        }

        fn apply_policy(&self, _world: &WorldHandle, _spec: &WorldSpec) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn ensure_world_ready_sets_env_on_success() {
        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");

        let cli = Cli::parse_from(["substrate"]);
        let calls = Arc::new(AtomicUsize::new(0));
        let backend = Arc::new(StubBackend::new("wld_test", calls.clone()));

        let result = ensure_world_ready_impl(&cli, || Ok(backend.clone())).unwrap();
        assert_eq!(result.as_deref(), Some("wld_test"));
        assert_eq!(std::env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
        assert_eq!(std::env::var("SUBSTRATE_WORLD_ID").unwrap(), "wld_test");
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");
    }

    #[test]
    fn ensure_world_ready_respects_no_world_flag() {
        std::env::remove_var("SUBSTRATE_WORLD");
        std::env::remove_var("SUBSTRATE_WORLD_ID");

        let cli = Cli::parse_from(["substrate", "--no-world"]);
        let calls = Arc::new(AtomicUsize::new(0));
        let backend = Arc::new(StubBackend::new("wld_test", calls.clone()));

        let result = ensure_world_ready_impl(&cli, || Ok(backend.clone())).unwrap();
        assert!(result.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
