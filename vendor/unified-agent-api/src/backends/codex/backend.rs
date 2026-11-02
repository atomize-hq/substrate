use std::{collections::BTreeSet, future::Future, pin::Pin, sync::Arc};

use super::{
    harness::{new_harness_adapter, CodexTerminationHandle},
    mcp_management, CodexBackend, CAP_ARTIFACTS_FINAL_TEXT_V1, CAP_SESSION_HANDLE_V1,
    CAP_TOOLS_RESULTS_V1, CAP_TOOLS_STRUCTURED_V1, EXT_ADD_DIRS_V1, EXT_CODEX_APPROVAL_POLICY,
    EXT_CODEX_SANDBOX_MODE, EXT_EXTERNAL_SANDBOX_V1, EXT_NON_INTERACTIVE,
};
use crate::{
    backend_harness::BackendDefaults,
    mcp::{
        normalize_mcp_add_request, normalize_mcp_get_request, normalize_mcp_remove_request,
        AgentWrapperMcpAddRequest, AgentWrapperMcpCommandOutput, AgentWrapperMcpGetRequest,
        AgentWrapperMcpListRequest, AgentWrapperMcpRemoveRequest, CAPABILITY_MCP_ADD_V1,
        CAPABILITY_MCP_GET_V1, CAPABILITY_MCP_LIST_V1, CAPABILITY_MCP_REMOVE_V1,
    },
    AgentWrapperBackend, AgentWrapperCapabilities, AgentWrapperError, AgentWrapperKind,
    AgentWrapperRunControl, AgentWrapperRunHandle, AgentWrapperRunRequest,
    EXT_AGENT_API_CONFIG_MODEL_V1,
};

use super::super::session_selectors::{EXT_SESSION_FORK_V1, EXT_SESSION_RESUME_V1};

fn unsupported_capability<T>(
    agent_kind: String,
    capability: &'static str,
) -> Pin<Box<dyn Future<Output = Result<T, AgentWrapperError>> + Send + 'static>> {
    Box::pin(async move {
        Err(AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability: capability.to_string(),
        })
    })
}

impl AgentWrapperBackend for CodexBackend {
    fn kind(&self) -> AgentWrapperKind {
        AgentWrapperKind("codex".to_string())
    }

    fn capabilities(&self) -> AgentWrapperCapabilities {
        let mut ids = BTreeSet::new();
        ids.insert("agent_api.run".to_string());
        ids.insert("agent_api.events".to_string());
        ids.insert("agent_api.events.live".to_string());
        ids.insert(crate::CAPABILITY_CONTROL_CANCEL_V1.to_string());
        ids.insert(CAP_TOOLS_STRUCTURED_V1.to_string());
        ids.insert(CAP_TOOLS_RESULTS_V1.to_string());
        ids.insert(CAP_ARTIFACTS_FINAL_TEXT_V1.to_string());
        ids.insert(CAP_SESSION_HANDLE_V1.to_string());
        ids.insert(EXT_AGENT_API_CONFIG_MODEL_V1.to_string());
        ids.insert("backend.codex.exec_stream".to_string());
        ids.insert(EXT_ADD_DIRS_V1.to_string());
        ids.insert(EXT_NON_INTERACTIVE.to_string());
        ids.insert(EXT_CODEX_APPROVAL_POLICY.to_string());
        ids.insert(EXT_CODEX_SANDBOX_MODE.to_string());
        ids.insert(EXT_SESSION_RESUME_V1.to_string());
        ids.insert(EXT_SESSION_FORK_V1.to_string());
        if super::codex_mcp_supported_on_target() {
            ids.insert(CAPABILITY_MCP_LIST_V1.to_string());
            ids.insert(CAPABILITY_MCP_GET_V1.to_string());
            if self.config.allow_mcp_write {
                ids.insert(CAPABILITY_MCP_ADD_V1.to_string());
                ids.insert(CAPABILITY_MCP_REMOVE_V1.to_string());
            }
        }
        if self.config.allow_external_sandbox_exec {
            ids.insert(EXT_EXTERNAL_SANDBOX_V1.to_string());
        }
        AgentWrapperCapabilities { ids }
    }

    fn mcp_list(
        &self,
        request: AgentWrapperMcpListRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        if !self.capabilities().contains(CAPABILITY_MCP_LIST_V1) {
            return unsupported_capability(
                self.kind().as_str().to_string(),
                CAPABILITY_MCP_LIST_V1,
            );
        }

        let config = self.config.clone();
        Box::pin(async move {
            mcp_management::run_codex_mcp(
                config,
                mcp_management::codex_mcp_list_argv(),
                request.context,
            )
            .await
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
        if !self.capabilities().contains(CAPABILITY_MCP_GET_V1) {
            return unsupported_capability(self.kind().as_str().to_string(), CAPABILITY_MCP_GET_V1);
        }

        let request = match normalize_mcp_get_request(request) {
            Ok(request) => request,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let config = self.config.clone();
        let argv = mcp_management::codex_mcp_get_argv(&request.name);
        Box::pin(async move { mcp_management::run_codex_mcp(config, argv, request.context).await })
    }

    fn mcp_add(
        &self,
        request: AgentWrapperMcpAddRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        if !self.capabilities().contains(CAPABILITY_MCP_ADD_V1) {
            return unsupported_capability(self.kind().as_str().to_string(), CAPABILITY_MCP_ADD_V1);
        }

        let request = match normalize_mcp_add_request(request) {
            Ok(request) => request,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let config = self.config.clone();
        let argv = mcp_management::codex_mcp_add_argv(&request.name, &request.transport);
        Box::pin(async move { mcp_management::run_codex_mcp(config, argv, request.context).await })
    }

    fn mcp_remove(
        &self,
        request: AgentWrapperMcpRemoveRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<AgentWrapperMcpCommandOutput, AgentWrapperError>>
                + Send
                + '_,
        >,
    > {
        if !self.capabilities().contains(CAPABILITY_MCP_REMOVE_V1) {
            return unsupported_capability(
                self.kind().as_str().to_string(),
                CAPABILITY_MCP_REMOVE_V1,
            );
        }

        let request = match normalize_mcp_remove_request(request) {
            Ok(request) => request,
            Err(err) => return Box::pin(async move { Err(err) }),
        };
        let config = self.config.clone();
        let argv = mcp_management::codex_mcp_remove_argv(&request.name);
        Box::pin(async move { mcp_management::run_codex_mcp(config, argv, request.context).await })
    }

    fn run(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunHandle, AgentWrapperError>> + Send + '_>>
    {
        let config = self.config.clone();
        let run_start_cwd = std::env::current_dir().ok();
        Box::pin(async move {
            let adapter = Arc::new(new_harness_adapter(config.clone(), run_start_cwd, None));

            let defaults = BackendDefaults {
                env: config.env,
                default_timeout: config.default_timeout,
            };

            crate::backend_harness::run_harnessed_backend(adapter, defaults, request).await
        })
    }

    fn run_control(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AgentWrapperRunControl, AgentWrapperError>> + Send + '_>>
    {
        if !self
            .capabilities()
            .contains(crate::CAPABILITY_CONTROL_CANCEL_V1)
        {
            return unsupported_capability(
                self.kind().as_str().to_string(),
                crate::CAPABILITY_CONTROL_CANCEL_V1,
            );
        }

        let config = self.config.clone();
        let run_start_cwd = std::env::current_dir().ok();
        Box::pin(async move {
            let termination_state: Arc<
                super::super::termination::TerminationState<CodexTerminationHandle>,
            > = Arc::new(super::super::termination::TerminationState::new());
            let request_termination: Option<Arc<dyn Fn() + Send + Sync + 'static>> = Some({
                let termination_state = Arc::clone(&termination_state);
                Arc::new(move || termination_state.request())
            });

            let adapter = Arc::new(new_harness_adapter(
                config.clone(),
                run_start_cwd,
                Some(termination_state),
            ));

            let defaults = BackendDefaults {
                env: config.env,
                default_timeout: config.default_timeout,
            };

            crate::backend_harness::run_harnessed_backend_control(
                adapter,
                defaults,
                request,
                request_termination,
            )
            .await
        })
    }
}
