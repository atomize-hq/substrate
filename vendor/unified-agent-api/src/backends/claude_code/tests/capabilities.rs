use std::fs;

use tempfile::tempdir;

use super::support::*;

#[test]
fn claude_backend_advertises_agent_api_config_model_v1() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    assert!(backend
        .capabilities()
        .contains(crate::EXT_AGENT_API_CONFIG_MODEL_V1));
}

#[test]
fn claude_backend_reports_required_capabilities() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    let capabilities = backend.capabilities();
    assert!(capabilities.contains("agent_api.run"));
    assert!(capabilities.contains("agent_api.events"));
    assert!(capabilities.contains("agent_api.events.live"));
    assert!(capabilities.contains(crate::CAPABILITY_CONTROL_CANCEL_V1));
    assert!(capabilities.contains(CAP_TOOLS_STRUCTURED_V1));
    assert!(capabilities.contains(CAP_TOOLS_RESULTS_V1));
    assert!(capabilities.contains(CAP_ARTIFACTS_FINAL_TEXT_V1));
    assert!(capabilities.contains(CAP_SESSION_HANDLE_V1));
    assert!(capabilities.contains(crate::EXT_AGENT_API_CONFIG_MODEL_V1));
    assert!(capabilities.contains(EXT_ADD_DIRS_V1));
    assert!(capabilities.contains(EXT_SESSION_RESUME_V1));
    assert!(capabilities.contains(EXT_SESSION_FORK_V1));
}

#[test]
fn claude_add_dirs_capability_and_supported_key_surfaces_stay_aligned() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    assert!(backend.capabilities().contains(EXT_ADD_DIRS_V1));

    let default_adapter = new_adapter();
    assert!(default_adapter
        .supported_extension_keys()
        .contains(&EXT_ADD_DIRS_V1));

    let external_sandbox_adapter = new_adapter_with_config(ClaudeCodeBackendConfig {
        allow_external_sandbox_exec: true,
        ..Default::default()
    });
    assert!(external_sandbox_adapter
        .supported_extension_keys()
        .contains(&EXT_ADD_DIRS_V1));
}

#[test]
fn claude_add_dirs_normalize_request_accepts_supported_key_and_extracts_policy() {
    let temp = tempdir().expect("tempdir");
    let working_dir = temp.path().join("workspace");
    let child_dir = working_dir.join("child");
    fs::create_dir_all(&child_dir).expect("create child directory");

    let adapter = new_adapter();
    let defaults = crate::backend_harness::BackendDefaults::default();
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(working_dir),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["child"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    };

    let normalized = crate::backend_harness::normalize_request(&adapter, &defaults, request)
        .expect("add_dirs should pass R0 gating and policy extraction");

    assert_eq!(normalized.policy.add_dirs, vec![child_dir]);
}

#[test]
fn claude_backend_mcp_write_capabilities_are_disabled_by_default() {
    assert!(!ClaudeCodeBackendConfig::default().allow_mcp_write);

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    let capabilities = backend.capabilities();
    assert_eq!(
        capabilities.contains(CAPABILITY_MCP_LIST_V1),
        claude_mcp_list_supported_on_target()
    );
    assert_eq!(
        capabilities.contains(CAPABILITY_MCP_GET_V1),
        claude_mcp_get_supported_on_target()
    );
    assert!(!capabilities.contains(CAPABILITY_MCP_ADD_V1));
    assert!(!capabilities.contains(CAPABILITY_MCP_REMOVE_V1));
}

#[test]
fn claude_backend_mcp_write_capabilities_require_opt_in_and_target_support() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        allow_mcp_write: true,
        ..Default::default()
    });
    let capabilities = backend.capabilities();
    assert_eq!(
        capabilities.contains(CAPABILITY_MCP_LIST_V1),
        claude_mcp_list_supported_on_target()
    );
    assert_eq!(
        capabilities.contains(CAPABILITY_MCP_GET_V1),
        claude_mcp_get_supported_on_target()
    );
    assert_eq!(
        capabilities.contains(CAPABILITY_MCP_ADD_V1),
        claude_mcp_get_supported_on_target()
    );
    assert_eq!(
        capabilities.contains(CAPABILITY_MCP_REMOVE_V1),
        claude_mcp_get_supported_on_target()
    );
}

#[tokio::test]
async fn claude_backend_mcp_list_fails_closed_when_read_capability_is_unavailable() {
    if claude_mcp_list_supported_on_target() {
        return;
    }

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    let err = backend
        .mcp_list(crate::mcp::AgentWrapperMcpListRequest::default())
        .await
        .expect_err("unsupported target should fail closed");

    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "claude_code");
            assert_eq!(capability, CAPABILITY_MCP_LIST_V1);
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
}

#[tokio::test]
async fn claude_backend_mcp_get_fails_closed_when_read_capability_is_unavailable() {
    if claude_mcp_get_supported_on_target() {
        return;
    }

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    let err = backend
        .mcp_get(crate::mcp::AgentWrapperMcpGetRequest {
            name: "demo".to_string(),
            context: Default::default(),
        })
        .await
        .expect_err("unsupported target should fail closed");

    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "claude_code");
            assert_eq!(capability, CAPABILITY_MCP_GET_V1);
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
}

#[tokio::test]
async fn claude_backend_mcp_add_fails_closed_when_write_capability_is_disabled() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    let err = backend
        .mcp_add(sample_mcp_add_request())
        .await
        .expect_err("write support should stay disabled by default");

    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "claude_code");
            assert_eq!(capability, CAPABILITY_MCP_ADD_V1);
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
}

#[tokio::test]
async fn claude_backend_mcp_remove_fails_closed_when_write_capability_is_disabled() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    let err = backend
        .mcp_remove(sample_mcp_remove_request())
        .await
        .expect_err("write support should stay disabled by default");

    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "claude_code");
            assert_eq!(capability, CAPABILITY_MCP_REMOVE_V1);
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
}

#[test]
fn claude_backend_registers_under_claude_code_kind_id() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    assert_eq!(backend.kind().as_str(), "claude_code");
}
