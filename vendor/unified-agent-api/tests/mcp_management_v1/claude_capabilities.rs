use agent_api::mcp::{
    AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpGetRequest,
    AgentWrapperMcpListRequest, AgentWrapperMcpRemoveRequest,
};

use super::{
    claude_support::{
        assert_unsupported_capability, claude_config_env, claude_gateway, claude_get_supported,
        claude_list_supported, CAPABILITY_MCP_ADD_V1, CAPABILITY_MCP_GET_V1,
        CAPABILITY_MCP_LIST_V1, CAPABILITY_MCP_REMOVE_V1,
    },
    support::McpTestSandbox,
};

#[tokio::test]
async fn claude_gateway_mcp_list_fails_closed_without_spawn_on_unsupported_targets() {
    if claude_list_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("claude_gateway_list_fail_closed").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let err = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect_err("unsupported list must fail closed");

    assert_unsupported_capability(err, CAPABILITY_MCP_LIST_V1);
    assert!(
        !sandbox.record_path().exists(),
        "unsupported list must not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn claude_gateway_mcp_get_fails_closed_without_spawn_off_win32_x64() {
    if claude_get_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("claude_gateway_get_fail_closed").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let err = gateway
        .mcp_get(
            &kind,
            AgentWrapperMcpGetRequest {
                name: "demo".to_string(),
                context: Default::default(),
            },
        )
        .await
        .expect_err("unsupported get must fail closed");

    assert_unsupported_capability(err, CAPABILITY_MCP_GET_V1);
    assert!(
        !sandbox.record_path().exists(),
        "unsupported get must not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn claude_gateway_mcp_add_fails_closed_without_spawn_when_write_disabled() {
    let sandbox = McpTestSandbox::new("claude_gateway_add_write_disabled").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let err = gateway
        .mcp_add(
            &kind,
            AgentWrapperMcpAddRequest {
                name: "demo".to_string(),
                transport: AgentWrapperMcpAddTransport::Stdio {
                    command: vec!["node".to_string()],
                    args: vec!["server.js".to_string()],
                    env: Default::default(),
                },
                context: Default::default(),
            },
        )
        .await
        .expect_err("write-disabled add must fail closed");

    assert_unsupported_capability(err, CAPABILITY_MCP_ADD_V1);
    assert!(
        !sandbox.record_path().exists(),
        "write-disabled add must not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn claude_gateway_mcp_remove_fails_closed_without_spawn_when_write_disabled() {
    let sandbox = McpTestSandbox::new("claude_gateway_remove_write_disabled").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let err = gateway
        .mcp_remove(
            &kind,
            AgentWrapperMcpRemoveRequest {
                name: "demo".to_string(),
                context: Default::default(),
            },
        )
        .await
        .expect_err("write-disabled remove must fail closed");

    assert_unsupported_capability(err, CAPABILITY_MCP_REMOVE_V1);
    assert!(
        !sandbox.record_path().exists(),
        "write-disabled remove must not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn claude_gateway_mcp_add_fails_closed_without_spawn_off_win32_x64_even_with_write_enabled() {
    if claude_get_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("claude_gateway_add_target_gated").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        true,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let err = gateway
        .mcp_add(
            &kind,
            AgentWrapperMcpAddRequest {
                name: "demo".to_string(),
                transport: AgentWrapperMcpAddTransport::Stdio {
                    command: vec!["node".to_string()],
                    args: vec!["server.js".to_string()],
                    env: Default::default(),
                },
                context: Default::default(),
            },
        )
        .await
        .expect_err("target-gated add must fail closed");

    assert_unsupported_capability(err, CAPABILITY_MCP_ADD_V1);
    assert!(
        !sandbox.record_path().exists(),
        "target-gated add must not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn claude_gateway_mcp_remove_fails_closed_without_spawn_off_win32_x64_even_with_write_enabled(
) {
    if claude_get_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("claude_gateway_remove_target_gated").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        true,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let err = gateway
        .mcp_remove(
            &kind,
            AgentWrapperMcpRemoveRequest {
                name: "demo".to_string(),
                context: Default::default(),
            },
        )
        .await
        .expect_err("target-gated remove must fail closed");

    assert_unsupported_capability(err, CAPABILITY_MCP_REMOVE_V1);
    assert!(
        !sandbox.record_path().exists(),
        "target-gated remove must not spawn the fake claude binary"
    );
}
