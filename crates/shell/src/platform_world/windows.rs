use crate::Cli;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Once, OnceLock};
use world_api::{ExecRequest, ResourceLimits, WorldBackend, WorldSpec};

static BACKEND: OnceLock<Arc<dyn WorldBackend>> = OnceLock::new();

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
    BACKEND
        .get_or_try_init(|| world_backend_factory::factory())
        .map(Arc::clone)
}

pub fn to_exec_request(cmd: &str, span_id: Option<String>) -> ExecRequest {
    ExecRequest {
        cmd: cmd.to_string(),
        cwd: current_dir(),
        env: collect_env(),
        pty: false,
        span_id,
    }
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

fn collect_env() -> HashMap<String, String> {
    std::env::vars().collect()
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

    #[test]
    fn to_exec_request_captures_environment() {
        std::env::set_var("SUBSTRATE_TEST_ENV", "1");
        let req = to_exec_request("echo hi", Some("span".to_string()));
        assert_eq!(req.cmd, "echo hi");
        assert_eq!(req.pty, false);
        assert_eq!(req.span_id.as_deref(), Some("span"));
        assert_eq!(req.cwd, super::current_dir());
        assert_eq!(req.env.get("SUBSTRATE_TEST_ENV"), Some(&"1".to_string()));
        std::env::remove_var("SUBSTRATE_TEST_ENV");
    }
}
