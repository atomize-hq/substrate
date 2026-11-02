#[cfg(feature = "claude_code")]
mod claude_code {
    use std::{collections::BTreeMap, path::PathBuf};

    use agent_api::{
        backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
        AgentWrapperBackend, AgentWrapperError, AgentWrapperRunRequest,
    };
    use serde_json::Value;

    use super::super::support::McpTestSandbox;

    const CAPABILITY_MCP_LIST_V1: &str = "agent_api.tools.mcp.list.v1";
    const CAPABILITY_MCP_GET_V1: &str = "agent_api.tools.mcp.get.v1";
    const CAPABILITY_MCP_ADD_V1: &str = "agent_api.tools.mcp.add.v1";
    const CAPABILITY_MCP_REMOVE_V1: &str = "agent_api.tools.mcp.remove.v1";
    const ALL_MCP_CAPABILITIES: [&str; 4] = [
        CAPABILITY_MCP_LIST_V1,
        CAPABILITY_MCP_GET_V1,
        CAPABILITY_MCP_ADD_V1,
        CAPABILITY_MCP_REMOVE_V1,
    ];

    fn claude_backend(
        sandbox: &McpTestSandbox,
        allow_mcp_write: bool,
    ) -> (ClaudeCodeBackend, PathBuf) {
        let binary = sandbox.install_fake_claude().expect("install fake claude");
        let record_path = sandbox.record_path().to_path_buf();
        let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
            binary: Some(binary),
            claude_home: Some(sandbox.claude_home().to_path_buf()),
            env: [(
                "FAKE_CLAUDE_MCP_RECORD_PATH".to_string(),
                record_path.to_string_lossy().into_owned(),
            )]
            .into_iter()
            .collect(),
            allow_mcp_write,
            ..Default::default()
        });
        (backend, record_path)
    }

    fn assert_capability_state(
        capabilities: &std::collections::BTreeSet<String>,
        id: &str,
        expected: bool,
    ) {
        assert_eq!(
            capabilities.contains(id),
            expected,
            "unexpected capability posture for {id}: {capabilities:?}"
        );
    }

    #[test]
    fn default_capability_posture_matches_pinned_target_matrix() {
        let sandbox =
            McpTestSandbox::new("claude_default_capability_posture").expect("create sandbox");
        let (backend, _record_path) = claude_backend(&sandbox, false);
        let capabilities = backend.capabilities().ids;

        assert_capability_state(
            &capabilities,
            CAPABILITY_MCP_LIST_V1,
            claude_list_supported(),
        );
        assert_capability_state(&capabilities, CAPABILITY_MCP_GET_V1, claude_get_supported());
        assert_capability_state(&capabilities, CAPABILITY_MCP_ADD_V1, false);
        assert_capability_state(&capabilities, CAPABILITY_MCP_REMOVE_V1, false);
    }

    #[test]
    fn write_capabilities_require_win32_x64_and_opt_in() {
        let sandbox =
            McpTestSandbox::new("claude_write_capability_posture").expect("create sandbox");
        let (backend, _record_path) = claude_backend(&sandbox, true);
        let capabilities = backend.capabilities().ids;

        assert_capability_state(
            &capabilities,
            CAPABILITY_MCP_LIST_V1,
            claude_list_supported(),
        );
        assert_capability_state(&capabilities, CAPABILITY_MCP_GET_V1, claude_get_supported());
        assert_capability_state(&capabilities, CAPABILITY_MCP_ADD_V1, claude_get_supported());
        assert_capability_state(
            &capabilities,
            CAPABILITY_MCP_REMOVE_V1,
            claude_get_supported(),
        );
    }

    #[tokio::test]
    async fn run_extensions_reject_all_mcp_capability_ids_without_spawning() {
        for capability in ALL_MCP_CAPABILITIES {
            let sandbox =
                McpTestSandbox::new(&format!("claude_non_run_{}", capability.replace('.', "_")))
                    .expect("create sandbox");
            let (backend, record_path) = claude_backend(&sandbox, true);

            let err = backend
                .run(AgentWrapperRunRequest {
                    prompt: "hello".to_string(),
                    extensions: [(capability.to_string(), Value::Bool(true))]
                        .into_iter()
                        .collect::<BTreeMap<_, _>>(),
                    ..Default::default()
                })
                .await
                .expect_err("MCP capability ids must be rejected as run extensions");

            assert_unsupported_capability(err, "claude_code", capability);
            assert!(
                !record_path.exists(),
                "rejecting {capability} should not spawn the fake claude binary"
            );
        }
    }

    fn claude_list_supported() -> bool {
        cfg!(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "windows", target_arch = "x86_64")
        ))
    }

    fn claude_get_supported() -> bool {
        cfg!(all(target_os = "windows", target_arch = "x86_64"))
    }

    fn assert_unsupported_capability(
        err: AgentWrapperError,
        expected_agent_kind: &str,
        expected_capability: &str,
    ) {
        match err {
            AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability,
            } => {
                assert_eq!(agent_kind, expected_agent_kind);
                assert_eq!(capability, expected_capability);
            }
            other => panic!("expected UnsupportedCapability, got {other:?}"),
        }
    }
}

#[cfg(feature = "codex")]
mod codex {
    use std::{collections::BTreeMap, path::PathBuf};

    use agent_api::{
        backends::codex::{CodexBackend, CodexBackendConfig},
        AgentWrapperBackend, AgentWrapperError, AgentWrapperRunRequest,
    };
    use serde_json::Value;

    use super::super::support::McpTestSandbox;

    const CAPABILITY_MCP_LIST_V1: &str = "agent_api.tools.mcp.list.v1";
    const CAPABILITY_MCP_GET_V1: &str = "agent_api.tools.mcp.get.v1";
    const CAPABILITY_MCP_ADD_V1: &str = "agent_api.tools.mcp.add.v1";
    const CAPABILITY_MCP_REMOVE_V1: &str = "agent_api.tools.mcp.remove.v1";
    const ALL_MCP_CAPABILITIES: [&str; 4] = [
        CAPABILITY_MCP_LIST_V1,
        CAPABILITY_MCP_GET_V1,
        CAPABILITY_MCP_ADD_V1,
        CAPABILITY_MCP_REMOVE_V1,
    ];

    fn codex_backend(sandbox: &McpTestSandbox, allow_mcp_write: bool) -> (CodexBackend, PathBuf) {
        let binary = sandbox.install_fake_codex().expect("install fake codex");
        let record_path = sandbox.record_path().to_path_buf();
        let backend = CodexBackend::new(CodexBackendConfig {
            binary: Some(binary),
            codex_home: Some(sandbox.codex_home().to_path_buf()),
            env: [(
                "FAKE_CODEX_MCP_RECORD_PATH".to_string(),
                record_path.to_string_lossy().into_owned(),
            )]
            .into_iter()
            .collect(),
            allow_mcp_write,
            ..Default::default()
        });
        (backend, record_path)
    }

    fn assert_capability_state(
        capabilities: &std::collections::BTreeSet<String>,
        id: &str,
        expected: bool,
    ) {
        assert_eq!(
            capabilities.contains(id),
            expected,
            "unexpected capability posture for {id}: {capabilities:?}"
        );
    }

    #[test]
    fn default_capability_posture_matches_pinned_target_matrix() {
        let sandbox =
            McpTestSandbox::new("codex_default_capability_posture").expect("create sandbox");
        let (backend, _record_path) = codex_backend(&sandbox, false);
        let capabilities = backend.capabilities().ids;

        assert_capability_state(&capabilities, CAPABILITY_MCP_LIST_V1, codex_mcp_supported());
        assert_capability_state(&capabilities, CAPABILITY_MCP_GET_V1, codex_mcp_supported());
        assert_capability_state(&capabilities, CAPABILITY_MCP_ADD_V1, false);
        assert_capability_state(&capabilities, CAPABILITY_MCP_REMOVE_V1, false);
    }

    #[test]
    fn write_capabilities_require_opt_in_and_target_support() {
        let sandbox =
            McpTestSandbox::new("codex_write_capability_posture").expect("create sandbox");
        let (backend, _record_path) = codex_backend(&sandbox, true);
        let capabilities = backend.capabilities().ids;

        assert_capability_state(&capabilities, CAPABILITY_MCP_LIST_V1, codex_mcp_supported());
        assert_capability_state(&capabilities, CAPABILITY_MCP_GET_V1, codex_mcp_supported());
        assert_capability_state(&capabilities, CAPABILITY_MCP_ADD_V1, codex_mcp_supported());
        assert_capability_state(
            &capabilities,
            CAPABILITY_MCP_REMOVE_V1,
            codex_mcp_supported(),
        );
    }

    #[tokio::test]
    async fn run_extensions_reject_all_mcp_capability_ids_without_spawning() {
        for capability in ALL_MCP_CAPABILITIES {
            let sandbox =
                McpTestSandbox::new(&format!("codex_non_run_{}", capability.replace('.', "_")))
                    .expect("create sandbox");
            let (backend, record_path) = codex_backend(&sandbox, true);

            let err = backend
                .run(AgentWrapperRunRequest {
                    prompt: "hello".to_string(),
                    extensions: [(capability.to_string(), Value::Bool(true))]
                        .into_iter()
                        .collect::<BTreeMap<_, _>>(),
                    ..Default::default()
                })
                .await
                .expect_err("MCP capability ids must be rejected as run extensions");

            assert_unsupported_capability(err, "codex", capability);
            assert!(
                !record_path.exists(),
                "rejecting {capability} should not spawn the fake codex binary"
            );
        }
    }

    fn codex_mcp_supported() -> bool {
        cfg!(all(target_os = "linux", target_arch = "x86_64"))
    }

    fn assert_unsupported_capability(
        err: AgentWrapperError,
        expected_agent_kind: &str,
        expected_capability: &str,
    ) {
        match err {
            AgentWrapperError::UnsupportedCapability {
                agent_kind,
                capability,
            } => {
                assert_eq!(agent_kind, expected_agent_kind);
                assert_eq!(capability, expected_capability);
            }
            other => panic!("expected UnsupportedCapability, got {other:?}"),
        }
    }
}
