#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_core::Stream;

mod agent_kind;
mod cancel_settlement;
use cancel_settlement::AgentWrapperCancelInner;
pub use cancel_settlement::{DynAgentWrapperDisposalRequested, DynAgentWrapperSettlement,
    DynAgentWrapperProcessSettlement, AgentWrapperProcessTerminationObservation, AgentWrapperProcessSettlementObservation};
mod runtime_support;

use crate::mcp::{
    normalize_mcp_add_request, normalize_mcp_get_request, normalize_mcp_remove_request,
    AgentWrapperMcpAddRequest, AgentWrapperMcpCommandOutput, AgentWrapperMcpGetRequest,
    AgentWrapperMcpListRequest, AgentWrapperMcpRemoveRequest, CAPABILITY_MCP_ADD_V1,
    CAPABILITY_MCP_GET_V1, CAPABILITY_MCP_LIST_V1, CAPABILITY_MCP_REMOVE_V1,
};
use agent_kind::validate_agent_kind;

#[cfg(any(
    feature = "codex",
    feature = "claude_code",
    feature = "aider",
    feature = "gemini_cli",
    feature = "opencode"
))]
mod bounds;

#[cfg(any(
    feature = "codex",
    feature = "claude_code",
    feature = "aider",
    feature = "gemini_cli",
    feature = "opencode"
))]
mod run_handle_gate;

#[cfg(any(
    feature = "codex",
    feature = "claude_code",
    feature = "aider",
    feature = "gemini_cli",
    feature = "opencode"
))]
mod backend_harness;

pub mod backends;
pub mod mcp;
pub use runtime_support::{list_runtime_support, resolve_runtime_support, RuntimeSupportRecord};

const CAPABILITY_CONTROL_CANCEL_V1: &str = "agent_api.control.cancel.v1";
#[allow(dead_code)]
#[cfg(any(
    feature = "codex",
    feature = "claude_code",
    feature = "aider",
    feature = "gemini_cli",
    feature = "opencode",
    test
))]
pub(crate) const EXT_AGENT_API_CONFIG_MODEL_V1: &str = "agent_api.config.model.v1";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AgentWrapperKind(String);

impl AgentWrapperKind {
    /// Creates an agent kind from a string.
    ///
    /// The value MUST follow `capabilities-schema-spec.md` naming rules.
    pub fn new(value: impl Into<String>) -> Result<Self, AgentWrapperError> {
        let value = value.into();
        validate_agent_kind(&value)?;
        Ok(Self(value))
    }

    /// Returns the canonical string id.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AgentWrapperCapabilities {
    /// Set of namespaced capability ids (see `capabilities-schema-spec.md`).
    pub ids: BTreeSet<String>,
}

impl AgentWrapperCapabilities {
    pub fn contains(&self, capability_id: &str) -> bool {
        self.ids.contains(capability_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentWrapperEventKind {
    TextOutput,
    ToolCall,
    ToolResult,
    Status,
    Error,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentWrapperEvent {
    pub agent_kind: AgentWrapperKind,
    pub kind: AgentWrapperEventKind,
    pub channel: Option<String>,
    /// Stable payload for `TextOutput` events.
    pub text: Option<String>,
    /// Stable payload for `Status` and `Error` events.
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default)]
pub struct AgentWrapperRunRequest {
    pub prompt: String,
    pub working_dir: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub env: BTreeMap<String, String>,
    /// Extension options are namespaced keys with JSON values.
    pub extensions: BTreeMap<String, serde_json::Value>,
}

pub type DynAgentWrapperEventStream = Pin<Box<dyn Stream<Item = AgentWrapperEvent> + Send>>;
pub type DynAgentWrapperCompletion =
    Pin<Box<dyn Future<Output = Result<AgentWrapperCompletion, AgentWrapperError>> + Send>>;

pub struct AgentWrapperRunHandle {
    pub events: DynAgentWrapperEventStream,
    pub completion: DynAgentWrapperCompletion,
}

impl std::fmt::Debug for AgentWrapperRunHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentWrapperRunHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .finish()
    }
}

#[derive(Clone)]
pub struct AgentWrapperCancelHandle {
    inner: Arc<AgentWrapperCancelInner>,
}

impl std::fmt::Debug for AgentWrapperCancelHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentWrapperCancelHandle")
            .finish_non_exhaustive()
    }
}

impl AgentWrapperCancelHandle {
    #[allow(dead_code)]
    pub(crate) fn new(cancel: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(AgentWrapperCancelInner {
                called: AtomicBool::new(false),
                cancel: Box::new(cancel),
                settlement: None,
                disposal: None,
                process_settlement: None,
            }),
        }
    }

    /// Requests best-effort cancellation of the underlying backend process.
    ///
    /// This method MUST be idempotent.
    ///
    /// If cancellation is requested before `AgentWrapperRunHandle.completion` resolves, the completion
    /// MUST resolve to `Err(AgentWrapperError::Backend { message: "cancelled" })`.
    ///
    /// Canonical semantics: `docs/specs/unified-agent-api/run-protocol-spec.md` ("Explicit
    /// cancellation semantics").
    pub fn cancel(&self) {
        if self.inner.called.swap(true, Ordering::SeqCst) {
            return;
        }

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (self.inner.cancel)();
        }));
    }
}

#[derive(Debug)]
pub struct AgentWrapperRunControl {
    pub handle: AgentWrapperRunHandle,
    pub cancel: AgentWrapperCancelHandle,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperCompletion {
    pub status: ExitStatus,
    /// A backend may populate `final_text` when it can deterministically extract it.
    pub final_text: Option<String>,
    /// Optional backend-specific completion payload.
    ///
    /// This payload MUST obey the bounds and enforcement behavior defined in
    /// `event-envelope-schema-spec.md` (see "Completion payload bounds").
    pub data: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperRunResult {
    pub completion: AgentWrapperCompletion,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentWrapperError {
    #[error("unknown backend: {agent_kind}")]
    UnknownBackend { agent_kind: String },
    #[error("unknown runtime family: {runtime_family}")]
    UnknownRuntimeFamily { runtime_family: String },
    #[error("unsupported target triple for {runtime_family}: {target_triple}")]
    UnsupportedTargetTriple {
        runtime_family: String,
        target_triple: String,
    },
    #[error("missing validated runtime for {runtime_family}: {target_triple}")]
    MissingValidatedRuntime {
        runtime_family: String,
        target_triple: String,
    },
    #[error("unsupported capability for {agent_kind}: {capability}")]
    UnsupportedCapability {
        agent_kind: String,
        capability: String,
    },
    #[error("invalid agent kind: {message}")]
    InvalidAgentKind { message: String },
    #[error("invalid request: {message}")]
    InvalidRequest { message: String },
    #[error("backend error: {message}")]
    Backend { message: String },
}

pub trait AgentWrapperBackend: Send + Sync {
    fn kind(&self) -> AgentWrapperKind;
    fn capabilities(&self) -> AgentWrapperCapabilities;

    /// Starts a run and returns a handle producing events and a completion result.
    ///
    /// Backends MUST enforce capability gating per `run-protocol-spec.md`.
    fn run(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>>;

    /// Starts a run and returns a handle plus an explicit cancellation handle.
    ///
    /// Backends that do not advertise `agent_api.control.cancel.v1` MUST return:
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability: "agent_api.control.cancel.v1" }`,
    /// where `agent_kind == self.kind().as_str().to_string()`.
    fn run_control(
        &self,
        _request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunControl, AgentWrapperError>> + Send + '_>>
    {
        let agent_kind = self.kind().as_str().to_string();
        Box::pin(async move {
            Err(AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability: CAPABILITY_CONTROL_CANCEL_V1.to_string(),
            })
        })
    }

    /// Runs `mcp list` and returns bounded command output.
    ///
    /// Backends that do not advertise `agent_api.tools.mcp.list.v1` MUST return:
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability: "agent_api.tools.mcp.list.v1" }`,
    /// where `agent_kind == self.kind().as_str().to_string()`.
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
        let agent_kind = self.kind().as_str().to_string();
        Box::pin(async move {
            Err(AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability: CAPABILITY_MCP_LIST_V1.to_string(),
            })
        })
    }

    /// Runs `mcp get` and returns bounded command output.
    ///
    /// Backends that do not advertise `agent_api.tools.mcp.get.v1` MUST return:
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability: "agent_api.tools.mcp.get.v1" }`,
    /// where `agent_kind == self.kind().as_str().to_string()`.
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
        let agent_kind = self.kind().as_str().to_string();
        Box::pin(async move {
            Err(AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability: CAPABILITY_MCP_GET_V1.to_string(),
            })
        })
    }

    /// Runs `mcp add` and returns bounded command output.
    ///
    /// Backends that do not advertise `agent_api.tools.mcp.add.v1` MUST return:
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability: "agent_api.tools.mcp.add.v1" }`,
    /// where `agent_kind == self.kind().as_str().to_string()`.
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
        let agent_kind = self.kind().as_str().to_string();
        Box::pin(async move {
            Err(AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability: CAPABILITY_MCP_ADD_V1.to_string(),
            })
        })
    }

    /// Runs `mcp remove` and returns bounded command output.
    ///
    /// Backends that do not advertise `agent_api.tools.mcp.remove.v1` MUST return:
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability: "agent_api.tools.mcp.remove.v1" }`,
    /// where `agent_kind == self.kind().as_str().to_string()`.
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
        let agent_kind = self.kind().as_str().to_string();
        Box::pin(async move {
            Err(AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability: CAPABILITY_MCP_REMOVE_V1.to_string(),
            })
        })
    }
}

#[derive(Clone, Default)]
pub struct AgentWrapperGateway {
    backends: BTreeMap<AgentWrapperKind, Arc<dyn AgentWrapperBackend>>,
}

impl AgentWrapperGateway {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a backend.
    ///
    /// If a backend with the same `AgentWrapperKind` is already registered, this MUST return an error.
    pub fn register(
        &mut self,
        backend: Arc<dyn AgentWrapperBackend>,
    ) -> Result<(), AgentWrapperError> {
        let kind = backend.kind();
        if self.backends.contains_key(&kind) {
            return Err(AgentWrapperError::InvalidRequest {
                message: format!("backend already registered: {}", kind.as_str()),
            });
        }
        self.backends.insert(kind, backend);
        Ok(())
    }

    /// Resolves a backend by `AgentWrapperKind`.
    pub fn backend(&self, agent_kind: &AgentWrapperKind) -> Option<Arc<dyn AgentWrapperBackend>> {
        self.backends.get(agent_kind).cloned()
    }

    /// Convenience entrypoint: resolves a backend and starts a run.
    ///
    /// This MUST return `AgentWrapperError::UnknownBackend` when no backend is registered for `agent_kind`.
    pub fn run(
        &self,
        agent_kind: &AgentWrapperKind,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>>
    {
        let backend = self.backends.get(agent_kind).cloned();
        let agent_kind = agent_kind.as_str().to_string();
        Box::pin(async move {
            let backend = backend.ok_or(AgentWrapperError::UnknownBackend { agent_kind })?;
            backend.run(request).await
        })
    }

    /// Starts a run and returns a control object including an explicit cancellation handle.
    ///
    /// This MUST return `AgentWrapperError::UnknownBackend { agent_kind }` when no backend is
    /// registered for the requested `agent_kind`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    ///
    /// If the resolved backend does not advertise `agent_api.control.cancel.v1`, this MUST return:
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability: "agent_api.control.cancel.v1" }`,
    /// where `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    ///
    /// Cancellation is best-effort and is defined by
    /// `docs/specs/unified-agent-api/run-protocol-spec.md`, including the pinned `"cancelled"`
    /// completion outcome.
    pub fn run_control(
        &self,
        agent_kind: &AgentWrapperKind,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunControl, AgentWrapperError>> + Send + '_>>
    {
        let backend = self.backends.get(agent_kind).cloned();
        let agent_kind = agent_kind.as_str().to_string();
        Box::pin(async move {
            let backend = backend.ok_or(AgentWrapperError::UnknownBackend {
                agent_kind: agent_kind.clone(),
            })?;

            if !backend
                .capabilities()
                .contains(CAPABILITY_CONTROL_CANCEL_V1)
            {
                return Err(AgentWrapperError::UnsupportedCapability {
                    agent_kind,
                    capability: CAPABILITY_CONTROL_CANCEL_V1.to_string(),
                });
            }

            backend.run_control(request).await
        })
    }

    /// Resolves a backend and runs `mcp list`.
    ///
    /// This MUST return `AgentWrapperError::UnknownBackend { agent_kind }` when no backend is
    /// registered for the requested `agent_kind`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    ///
    /// If the resolved backend does not advertise `agent_api.tools.mcp.list.v1`, this MUST
    /// return `AgentWrapperError::UnsupportedCapability { agent_kind, capability:
    /// "agent_api.tools.mcp.list.v1" }`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    pub fn mcp_list(
        &self,
        agent_kind: &AgentWrapperKind,
        request: AgentWrapperMcpListRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let backend = self.backends.get(agent_kind).cloned();
        let agent_kind = agent_kind.as_str().to_string();
        Box::pin(async move {
            let backend = backend.ok_or(AgentWrapperError::UnknownBackend {
                agent_kind: agent_kind.clone(),
            })?;

            if !backend.capabilities().contains(CAPABILITY_MCP_LIST_V1) {
                return Err(AgentWrapperError::UnsupportedCapability {
                    agent_kind,
                    capability: CAPABILITY_MCP_LIST_V1.to_string(),
                });
            }

            backend.mcp_list(request).await
        })
    }

    /// Resolves a backend and runs `mcp get`.
    ///
    /// This MUST return `AgentWrapperError::UnknownBackend { agent_kind }` when no backend is
    /// registered for the requested `agent_kind`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    ///
    /// If the resolved backend does not advertise `agent_api.tools.mcp.get.v1`, this MUST return
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability:
    /// "agent_api.tools.mcp.get.v1" }`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    pub fn mcp_get(
        &self,
        agent_kind: &AgentWrapperKind,
        request: AgentWrapperMcpGetRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let backend = self.backends.get(agent_kind).cloned();
        let agent_kind = agent_kind.as_str().to_string();
        Box::pin(async move {
            let backend = backend.ok_or(AgentWrapperError::UnknownBackend {
                agent_kind: agent_kind.clone(),
            })?;

            if !backend.capabilities().contains(CAPABILITY_MCP_GET_V1) {
                return Err(AgentWrapperError::UnsupportedCapability {
                    agent_kind,
                    capability: CAPABILITY_MCP_GET_V1.to_string(),
                });
            }

            let request = normalize_mcp_get_request(request)?;
            backend.mcp_get(request).await
        })
    }

    /// Resolves a backend and runs `mcp add`.
    ///
    /// This MUST return `AgentWrapperError::UnknownBackend { agent_kind }` when no backend is
    /// registered for the requested `agent_kind`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    ///
    /// If the resolved backend does not advertise `agent_api.tools.mcp.add.v1`, this MUST return
    /// `AgentWrapperError::UnsupportedCapability { agent_kind, capability:
    /// "agent_api.tools.mcp.add.v1" }`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    pub fn mcp_add(
        &self,
        agent_kind: &AgentWrapperKind,
        request: AgentWrapperMcpAddRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let backend = self.backends.get(agent_kind).cloned();
        let agent_kind = agent_kind.as_str().to_string();
        Box::pin(async move {
            let backend = backend.ok_or(AgentWrapperError::UnknownBackend {
                agent_kind: agent_kind.clone(),
            })?;

            if !backend.capabilities().contains(CAPABILITY_MCP_ADD_V1) {
                return Err(AgentWrapperError::UnsupportedCapability {
                    agent_kind,
                    capability: CAPABILITY_MCP_ADD_V1.to_string(),
                });
            }

            let request = normalize_mcp_add_request(request)?;
            backend.mcp_add(request).await
        })
    }

    /// Resolves a backend and runs `mcp remove`.
    ///
    /// This MUST return `AgentWrapperError::UnknownBackend { agent_kind }` when no backend is
    /// registered for the requested `agent_kind`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    ///
    /// If the resolved backend does not advertise `agent_api.tools.mcp.remove.v1`, this MUST
    /// return `AgentWrapperError::UnsupportedCapability { agent_kind, capability:
    /// "agent_api.tools.mcp.remove.v1" }`, where
    /// `agent_kind == <requested AgentWrapperKind>.as_str().to_string()`.
    pub fn mcp_remove(
        &self,
        agent_kind: &AgentWrapperKind,
        request: AgentWrapperMcpRemoveRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        let backend = self.backends.get(agent_kind).cloned();
        let agent_kind = agent_kind.as_str().to_string();
        Box::pin(async move {
            let backend = backend.ok_or(AgentWrapperError::UnknownBackend {
                agent_kind: agent_kind.clone(),
            })?;

            if !backend.capabilities().contains(CAPABILITY_MCP_REMOVE_V1) {
                return Err(AgentWrapperError::UnsupportedCapability {
                    agent_kind,
                    capability: CAPABILITY_MCP_REMOVE_V1.to_string(),
                });
            }

            let request = normalize_mcp_remove_request(request)?;
            backend.mcp_remove(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Wake, Waker};

    use super::*;
    use crate::mcp::{
        AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext,
        AgentWrapperMcpGetRequest, CAPABILITY_MCP_ADD_V1, CAPABILITY_MCP_GET_V1,
    };

    #[test]
    fn cancel_handle_is_idempotent() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_cancel = Arc::clone(&calls);
        let cancel = AgentWrapperCancelHandle::new(move || {
            calls_for_cancel.fetch_add(1, Ordering::SeqCst);
        });

        cancel.cancel();
        cancel.cancel();
        cancel.cancel();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn mcp_add_returns_unknown_backend_before_validation() {
        let gateway = AgentWrapperGateway::new();
        let agent_kind = AgentWrapperKind::new("codex").expect("valid kind");
        let secret = "SECRET_BACKEND_VALUE";

        let err = block_on_immediate(gateway.mcp_add(
            &agent_kind,
            AgentWrapperMcpAddRequest {
                name: format!("  {secret}  "),
                transport: AgentWrapperMcpAddTransport::Url {
                    url: format!("relative/{secret}"),
                    bearer_token_env_var: None,
                },
                context: AgentWrapperMcpCommandContext::default(),
            },
        ))
        .expect_err("unknown backend should fail first");

        match err {
            AgentWrapperError::UnknownBackend { agent_kind } => {
                assert_eq!(agent_kind, "codex");
            }
            other => panic!("expected UnknownBackend, got {other:?}"),
        }
    }

    #[test]
    fn mcp_add_returns_unsupported_capability_before_validation_and_without_hook_call() {
        let backend = Arc::new(TestBackend::new(BTreeSet::new()));
        let mut gateway = AgentWrapperGateway::new();
        gateway.register(backend.clone()).expect("register backend");
        let agent_kind = backend.kind();
        let secret = "SECRET_UNSUPPORTED_VALUE";

        let err = block_on_immediate(gateway.mcp_add(
            &agent_kind,
            AgentWrapperMcpAddRequest {
                name: format!("  {secret}  "),
                transport: AgentWrapperMcpAddTransport::Url {
                    url: format!("relative/{secret}"),
                    bearer_token_env_var: None,
                },
                context: AgentWrapperMcpCommandContext::default(),
            },
        ))
        .expect_err("unsupported capability should fail before validation");

        match err {
            AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability,
            } => {
                assert_eq!(agent_kind, "test_backend");
                assert_eq!(capability, CAPABILITY_MCP_ADD_V1);
            }
            other => panic!("expected UnsupportedCapability, got {other:?}"),
        }

        assert_eq!(backend.mcp_add_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn mcp_add_returns_invalid_request_before_hook_when_capability_is_advertised() {
        let backend = Arc::new(TestBackend::new(BTreeSet::from([
            CAPABILITY_MCP_ADD_V1.to_string()
        ])));
        let mut gateway = AgentWrapperGateway::new();
        gateway.register(backend.clone()).expect("register backend");
        let agent_kind = backend.kind();
        let secret = "SECRET_INVALID_VALUE";

        let err = block_on_immediate(gateway.mcp_add(
            &agent_kind,
            AgentWrapperMcpAddRequest {
                name: "  example  ".to_string(),
                transport: AgentWrapperMcpAddTransport::Url {
                    url: format!("https:{secret}.example.com"),
                    bearer_token_env_var: None,
                },
                context: AgentWrapperMcpCommandContext::default(),
            },
        ))
        .expect_err("invalid request should fail before hook");

        match err {
            AgentWrapperError::InvalidRequest { message } => {
                assert_eq!(message, "mcp add url must be an absolute http or https URL");
                assert!(!message.contains(secret));
            }
            other => panic!("expected InvalidRequest, got {other:?}"),
        }

        assert_eq!(backend.mcp_add_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn mcp_get_passes_normalized_request_to_hook() {
        let backend = Arc::new(TestBackend::new(BTreeSet::from([
            CAPABILITY_MCP_GET_V1.to_string()
        ])));
        let mut gateway = AgentWrapperGateway::new();
        gateway.register(backend.clone()).expect("register backend");
        let agent_kind = backend.kind();

        let output = block_on_immediate(gateway.mcp_get(
            &agent_kind,
            AgentWrapperMcpGetRequest {
                name: "  normalized-name  ".to_string(),
                context: AgentWrapperMcpCommandContext::default(),
            },
        ))
        .expect("normalized request should reach hook");

        assert_eq!(output.stdout, "ok");
        assert_eq!(backend.mcp_get_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            backend
                .last_get_name
                .lock()
                .expect("get name mutex")
                .as_deref(),
            Some("normalized-name")
        );
    }

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn block_on_immediate<F>(future: F) -> F::Output
    where
        F: Future,
    {
        let waker = Waker::from(Arc::new(NoopWake));
        let mut future = Box::pin(future);
        let mut context = Context::from_waker(&waker);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
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

    fn success_output() -> AgentWrapperMcpCommandOutput {
        AgentWrapperMcpCommandOutput {
            status: success_exit_status(),
            stdout: "ok".to_string(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    struct TestBackend {
        capabilities: AgentWrapperCapabilities,
        mcp_add_calls: AtomicUsize,
        mcp_get_calls: AtomicUsize,
        last_get_name: Mutex<Option<String>>,
    }

    impl TestBackend {
        fn new(capabilities: BTreeSet<String>) -> Self {
            Self {
                capabilities: AgentWrapperCapabilities { ids: capabilities },
                mcp_add_calls: AtomicUsize::new(0),
                mcp_get_calls: AtomicUsize::new(0),
                last_get_name: Mutex::new(None),
            }
        }
    }

    impl AgentWrapperBackend for TestBackend {
        fn kind(&self) -> AgentWrapperKind {
            AgentWrapperKind("test_backend".to_string())
        }

        fn capabilities(&self) -> AgentWrapperCapabilities {
            self.capabilities.clone()
        }

        fn run(
            &self,
            _request: AgentWrapperRunRequest,
        ) -> Pin<
            Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>,
        > {
            Box::pin(async {
                Err(AgentWrapperError::Backend {
                    message: "run not used in tests".to_string(),
                })
            })
        }

        fn mcp_get(
            &self,
            request: AgentWrapperMcpGetRequest,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                    + Send
                    + '_,
            >,
        > {
            self.mcp_get_calls.fetch_add(1, Ordering::SeqCst);
            *self.last_get_name.lock().expect("last get name mutex") = Some(request.name);
            Box::pin(async move { Ok(success_output()) })
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
            self.mcp_add_calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { Ok(success_output()) })
        }
    }
}
