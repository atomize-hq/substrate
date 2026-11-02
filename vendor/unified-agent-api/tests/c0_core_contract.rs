use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use agent_api::mcp::{
    AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext,
    AgentWrapperMcpCommandOutput, AgentWrapperMcpGetRequest, AgentWrapperMcpListRequest,
    AgentWrapperMcpRemoveRequest,
};
use agent_api::{
    AgentWrapperBackend, AgentWrapperCapabilities, AgentWrapperError, AgentWrapperGateway,
    AgentWrapperKind, AgentWrapperRunHandle, AgentWrapperRunRequest,
};

fn noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

fn block_on_ready<F: Future>(mut future: F) -> F::Output {
    let waker = noop_waker();
    let mut context = Context::from_waker(&waker);
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    for _ in 0..32 {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        std::thread::yield_now();
    }

    panic!("future did not resolve quickly (expected Ready)");
}

fn success_exit_status() -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
}

#[derive(Clone)]
struct DummyBackend {
    kind: AgentWrapperKind,
    capabilities: AgentWrapperCapabilities,
}

impl DummyBackend {
    fn new(kind: AgentWrapperKind) -> Self {
        let mut ids = BTreeSet::new();
        ids.insert("agent_api.run".to_string());
        ids.insert("agent_api.events".to_string());
        Self {
            kind,
            capabilities: AgentWrapperCapabilities { ids },
        }
    }
}

impl AgentWrapperBackend for DummyBackend {
    fn kind(&self) -> AgentWrapperKind {
        self.kind.clone()
    }

    fn capabilities(&self) -> AgentWrapperCapabilities {
        self.capabilities.clone()
    }

    fn run(
        &self,
        _request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>>
    {
        Box::pin(async move {
            Err(AgentWrapperError::Backend {
                message: "dummy backend".to_string(),
            })
        })
    }
}

#[derive(Clone, Default)]
struct McpCounters {
    list: Arc<AtomicUsize>,
    get: Arc<AtomicUsize>,
    add: Arc<AtomicUsize>,
    remove: Arc<AtomicUsize>,
}

#[derive(Clone)]
struct McpBackend {
    kind: AgentWrapperKind,
    capabilities: AgentWrapperCapabilities,
    counters: McpCounters,
    list_output: AgentWrapperMcpCommandOutput,
    get_output: AgentWrapperMcpCommandOutput,
    add_output: AgentWrapperMcpCommandOutput,
    remove_output: AgentWrapperMcpCommandOutput,
}

impl McpBackend {
    fn new(kind: AgentWrapperKind, capability_ids: &[&str]) -> Self {
        let capabilities = capability_ids
            .iter()
            .map(|id| (*id).to_string())
            .collect::<BTreeSet<_>>();

        Self {
            kind,
            capabilities: AgentWrapperCapabilities { ids: capabilities },
            counters: McpCounters::default(),
            list_output: mcp_output("list"),
            get_output: mcp_output("get"),
            add_output: mcp_output("add"),
            remove_output: mcp_output("remove"),
        }
    }
}

impl AgentWrapperBackend for McpBackend {
    fn kind(&self) -> AgentWrapperKind {
        self.kind.clone()
    }

    fn capabilities(&self) -> AgentWrapperCapabilities {
        self.capabilities.clone()
    }

    fn run(
        &self,
        _request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>>
    {
        Box::pin(async move {
            Err(AgentWrapperError::Backend {
                message: "mcp backend does not support run".to_string(),
            })
        })
    }

    fn mcp_list(
        &self,
        _request: AgentWrapperMcpListRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let counters = self.counters.clone();
        let output = self.list_output.clone();
        Box::pin(async move {
            counters.list.fetch_add(1, Ordering::SeqCst);
            Ok(output)
        })
    }

    fn mcp_get(
        &self,
        _request: AgentWrapperMcpGetRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let counters = self.counters.clone();
        let output = self.get_output.clone();
        Box::pin(async move {
            counters.get.fetch_add(1, Ordering::SeqCst);
            Ok(output)
        })
    }

    fn mcp_add(
        &self,
        _request: AgentWrapperMcpAddRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let counters = self.counters.clone();
        let output = self.add_output.clone();
        Box::pin(async move {
            counters.add.fetch_add(1, Ordering::SeqCst);
            Ok(output)
        })
    }

    fn mcp_remove(
        &self,
        _request: AgentWrapperMcpRemoveRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let counters = self.counters.clone();
        let output = self.remove_output.clone();
        Box::pin(async move {
            counters.remove.fetch_add(1, Ordering::SeqCst);
            Ok(output)
        })
    }
}

fn mcp_output(stdout: &str) -> AgentWrapperMcpCommandOutput {
    AgentWrapperMcpCommandOutput {
        status: success_exit_status(),
        stdout: stdout.to_string(),
        stderr: String::new(),
        stdout_truncated: false,
        stderr_truncated: false,
    }
}

fn mcp_context() -> AgentWrapperMcpCommandContext {
    AgentWrapperMcpCommandContext {
        working_dir: None,
        timeout: None,
        env: BTreeMap::new(),
    }
}

fn mcp_list_request() -> AgentWrapperMcpListRequest {
    AgentWrapperMcpListRequest {
        context: mcp_context(),
    }
}

fn mcp_get_request() -> AgentWrapperMcpGetRequest {
    AgentWrapperMcpGetRequest {
        name: "server".to_string(),
        context: mcp_context(),
    }
}

fn mcp_add_request() -> AgentWrapperMcpAddRequest {
    AgentWrapperMcpAddRequest {
        name: "server".to_string(),
        transport: AgentWrapperMcpAddTransport::Url {
            url: "https://example.com/mcp".to_string(),
            bearer_token_env_var: None,
        },
        context: mcp_context(),
    }
}

fn mcp_remove_request() -> AgentWrapperMcpRemoveRequest {
    AgentWrapperMcpRemoveRequest {
        name: "server".to_string(),
        context: mcp_context(),
    }
}

fn assert_call_counts(counters: &McpCounters, list: usize, get: usize, add: usize, remove: usize) {
    assert_eq!(counters.list.load(Ordering::SeqCst), list);
    assert_eq!(counters.get.load(Ordering::SeqCst), get);
    assert_eq!(counters.add.load(Ordering::SeqCst), add);
    assert_eq!(counters.remove.load(Ordering::SeqCst), remove);
}

#[test]
fn agent_wrapper_kind_enforces_naming_rules() {
    let ok = AgentWrapperKind::new("codex").unwrap();
    assert_eq!(ok.as_str(), "codex");

    assert!(AgentWrapperKind::new("Codex").is_err());
    assert!(AgentWrapperKind::new("codex-1").is_err());
    assert!(AgentWrapperKind::new("1codex").is_err());
    assert!(AgentWrapperKind::new("").is_err());
}

#[test]
fn agent_wrapper_capabilities_contains_is_set_membership() {
    let mut capabilities = AgentWrapperCapabilities::default();
    assert!(!capabilities.contains("agent_api.run"));
    capabilities.ids.insert("agent_api.run".to_string());
    assert!(capabilities.contains("agent_api.run"));
    assert!(!capabilities.contains("agent_api.events"));
}

#[test]
fn gateway_register_and_backend_roundtrip() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(DummyBackend::new(kind.clone()));

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let resolved = gateway.backend(&kind).expect("backend");
    assert_eq!(resolved.kind(), kind);
}

#[test]
fn gateway_register_rejects_duplicate_kind() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend1 = Arc::new(DummyBackend::new(kind.clone()));
    let backend2 = Arc::new(DummyBackend::new(kind.clone()));

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend1).unwrap();

    let err = gateway.register(backend2).unwrap_err();
    assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
}

#[test]
fn gateway_run_unknown_backend_is_error() {
    let gateway = AgentWrapperGateway::new();
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let result = block_on_ready(gateway.run(&kind, request));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnknownBackend { .. })
    ));
}

#[test]
fn gateway_run_control_unknown_backend_is_error() {
    let gateway = AgentWrapperGateway::new();
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let result = block_on_ready(gateway.run_control(&kind, request));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnknownBackend { .. })
    ));
}

#[test]
fn gateway_run_control_missing_capability_is_unsupported() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(DummyBackend::new(kind.clone()));

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let result = block_on_ready(gateway.run_control(&kind, request));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.control.cancel.v1"
    ));
}

#[test]
fn backend_run_control_default_is_fail_closed() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = DummyBackend::new(kind);
    let request = AgentWrapperRunRequest::default();

    let result = block_on_ready(backend.run_control(request));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.control.cancel.v1"
    ));
}

#[test]
fn mcp_public_types_compile_surface() {
    use agent_api::mcp::{
        AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext,
        AgentWrapperMcpCommandOutput, AgentWrapperMcpGetRequest, AgentWrapperMcpListRequest,
        AgentWrapperMcpRemoveRequest,
    };

    let context = AgentWrapperMcpCommandContext {
        working_dir: None,
        timeout: None,
        env: BTreeMap::new(),
    };
    let output = AgentWrapperMcpCommandOutput {
        status: success_exit_status(),
        stdout: String::new(),
        stderr: String::new(),
        stdout_truncated: false,
        stderr_truncated: false,
    };
    let list_request = AgentWrapperMcpListRequest {
        context: context.clone(),
    };
    let get_request = AgentWrapperMcpGetRequest {
        name: "server".to_string(),
        context: context.clone(),
    };
    let add_request = AgentWrapperMcpAddRequest {
        name: "server".to_string(),
        transport: AgentWrapperMcpAddTransport::Stdio {
            command: vec!["cmd".to_string()],
            args: vec!["--flag".to_string()],
            env: BTreeMap::new(),
        },
        context: context.clone(),
    };
    let remove_request = AgentWrapperMcpRemoveRequest {
        name: "server".to_string(),
        context,
    };

    let _ = (
        output,
        list_request,
        get_request,
        add_request,
        remove_request,
    );
}

#[test]
fn backend_mcp_list_default_is_fail_closed() {
    let backend = DummyBackend::new(AgentWrapperKind::new("dummy").unwrap());
    let result = block_on_ready(backend.mcp_list(mcp_list_request()));

    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.list.v1"
    ));
}

#[test]
fn backend_mcp_get_default_is_fail_closed() {
    let backend = DummyBackend::new(AgentWrapperKind::new("dummy").unwrap());
    let result = block_on_ready(backend.mcp_get(mcp_get_request()));

    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.get.v1"
    ));
}

#[test]
fn backend_mcp_add_default_is_fail_closed() {
    let backend = DummyBackend::new(AgentWrapperKind::new("dummy").unwrap());
    let result = block_on_ready(backend.mcp_add(mcp_add_request()));

    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.add.v1"
    ));
}

#[test]
fn backend_mcp_remove_default_is_fail_closed() {
    let backend = DummyBackend::new(AgentWrapperKind::new("dummy").unwrap());
    let result = block_on_ready(backend.mcp_remove(mcp_remove_request()));

    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.remove.v1"
    ));
}

#[test]
fn gateway_mcp_list_unknown_backend_is_error() {
    let gateway = AgentWrapperGateway::new();
    let kind = AgentWrapperKind::new("dummy").unwrap();

    let result = block_on_ready(gateway.mcp_list(&kind, mcp_list_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnknownBackend { .. })
    ));
}

#[test]
fn gateway_mcp_list_missing_capability_is_unsupported_without_invoking_hook() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(kind.clone(), &[]));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_list(&kind, mcp_list_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.list.v1"
    ));
    assert_call_counts(&counters, 0, 0, 0, 0);
}

#[test]
fn gateway_mcp_list_calls_matching_hook_when_capability_is_advertised() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(
        kind.clone(),
        &["agent_api.tools.mcp.list.v1"],
    ));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_list(&kind, mcp_list_request())).unwrap();
    assert_eq!(result.stdout, "list");
    assert_call_counts(&counters, 1, 0, 0, 0);
}

#[test]
fn gateway_mcp_get_unknown_backend_is_error() {
    let gateway = AgentWrapperGateway::new();
    let kind = AgentWrapperKind::new("dummy").unwrap();

    let result = block_on_ready(gateway.mcp_get(&kind, mcp_get_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnknownBackend { .. })
    ));
}

#[test]
fn gateway_mcp_get_missing_capability_is_unsupported_without_invoking_hook() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(kind.clone(), &[]));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_get(&kind, mcp_get_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.get.v1"
    ));
    assert_call_counts(&counters, 0, 0, 0, 0);
}

#[test]
fn gateway_mcp_get_calls_matching_hook_when_capability_is_advertised() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(
        kind.clone(),
        &["agent_api.tools.mcp.get.v1"],
    ));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_get(&kind, mcp_get_request())).unwrap();
    assert_eq!(result.stdout, "get");
    assert_call_counts(&counters, 0, 1, 0, 0);
}

#[test]
fn gateway_mcp_add_unknown_backend_is_error() {
    let gateway = AgentWrapperGateway::new();
    let kind = AgentWrapperKind::new("dummy").unwrap();

    let result = block_on_ready(gateway.mcp_add(&kind, mcp_add_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnknownBackend { .. })
    ));
}

#[test]
fn gateway_mcp_add_missing_capability_is_unsupported_without_invoking_hook() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(kind.clone(), &[]));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_add(&kind, mcp_add_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.add.v1"
    ));
    assert_call_counts(&counters, 0, 0, 0, 0);
}

#[test]
fn gateway_mcp_add_calls_matching_hook_when_capability_is_advertised() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(
        kind.clone(),
        &["agent_api.tools.mcp.add.v1"],
    ));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_add(&kind, mcp_add_request())).unwrap();
    assert_eq!(result.stdout, "add");
    assert_call_counts(&counters, 0, 0, 1, 0);
}

#[test]
fn gateway_mcp_remove_unknown_backend_is_error() {
    let gateway = AgentWrapperGateway::new();
    let kind = AgentWrapperKind::new("dummy").unwrap();

    let result = block_on_ready(gateway.mcp_remove(&kind, mcp_remove_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnknownBackend { .. })
    ));
}

#[test]
fn gateway_mcp_remove_missing_capability_is_unsupported_without_invoking_hook() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(kind.clone(), &[]));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_remove(&kind, mcp_remove_request()));
    assert!(matches!(
        result,
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        }) if agent_kind == "dummy" && capability == "agent_api.tools.mcp.remove.v1"
    ));
    assert_call_counts(&counters, 0, 0, 0, 0);
}

#[test]
fn gateway_mcp_remove_calls_matching_hook_when_capability_is_advertised() {
    let kind = AgentWrapperKind::new("dummy").unwrap();
    let backend = Arc::new(McpBackend::new(
        kind.clone(),
        &["agent_api.tools.mcp.remove.v1"],
    ));
    let counters = backend.counters.clone();

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();

    let result = block_on_ready(gateway.mcp_remove(&kind, mcp_remove_request())).unwrap();
    assert_eq!(result.stdout, "remove");
    assert_call_counts(&counters, 0, 0, 0, 1);
}
