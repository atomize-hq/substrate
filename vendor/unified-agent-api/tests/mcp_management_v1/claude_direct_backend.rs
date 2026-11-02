use std::collections::BTreeMap;

use agent_api::{
    backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
    mcp::{
        AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext,
        AgentWrapperMcpGetRequest, AgentWrapperMcpRemoveRequest,
    },
    AgentWrapperBackend, AgentWrapperError,
};

use super::{
    claude_support::{
        assert_unsupported_capability, claude_config_env, claude_get_supported,
        CAPABILITY_MCP_ADD_V1,
    },
    support::McpTestSandbox,
};

const ERR_MCP_SERVER_NAME_EMPTY: &str = "mcp server name must be non-empty";
const ERR_MCP_ADD_STDIO_ITEM_EMPTY: &str = "mcp add stdio.command[0] must be non-empty";
const ERR_MCP_ADD_URL_INVALID: &str = "mcp add url must be an absolute http or https URL";
const ERR_CLAUDE_URL_BEARER_ENV_UNSUPPORTED: &str =
    "claude mcp add url transport does not support bearer_token_env_var";

#[tokio::test]
async fn direct_claude_mcp_get_rejects_empty_name_without_spawning() {
    if !claude_get_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("direct_claude_mcp_get_invalid_name").expect("sandbox");
    let backend = claude_backend(&sandbox, false);

    let err = backend
        .mcp_get(AgentWrapperMcpGetRequest {
            name: "   ".to_string(),
            context: AgentWrapperMcpCommandContext::default(),
        })
        .await
        .expect_err("whitespace-only names must fail closed");

    assert_invalid_request(err, ERR_MCP_SERVER_NAME_EMPTY, &["   "]);
    assert!(
        !sandbox.record_path().exists(),
        "invalid direct mcp_get request should not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn direct_claude_mcp_remove_rejects_empty_name_without_spawning() {
    if !claude_get_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("direct_claude_mcp_remove_invalid_name").expect("sandbox");
    let backend = claude_backend(&sandbox, true);

    let err = backend
        .mcp_remove(AgentWrapperMcpRemoveRequest {
            name: " \n\t ".to_string(),
            context: AgentWrapperMcpCommandContext::default(),
        })
        .await
        .expect_err("whitespace-only names must fail closed");

    assert_invalid_request(err, ERR_MCP_SERVER_NAME_EMPTY, &[" \n\t "]);
    assert!(
        !sandbox.record_path().exists(),
        "invalid direct mcp_remove request should not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn direct_claude_mcp_add_rejects_whitespace_stdio_command_without_spawning() {
    if !claude_get_supported() {
        return;
    }

    let sandbox =
        McpTestSandbox::new("direct_claude_mcp_add_invalid_stdio_command").expect("sandbox");
    let backend = claude_backend(&sandbox, true);

    let err = backend
        .mcp_add(AgentWrapperMcpAddRequest {
            name: "demo".to_string(),
            transport: AgentWrapperMcpAddTransport::Stdio {
                command: vec!["   ".to_string()],
                args: vec!["server.js".to_string()],
                env: BTreeMap::new(),
            },
            context: AgentWrapperMcpCommandContext::default(),
        })
        .await
        .expect_err("whitespace-only stdio.command items must fail closed");

    assert_invalid_request(err, ERR_MCP_ADD_STDIO_ITEM_EMPTY, &["   "]);
    assert!(
        !sandbox.record_path().exists(),
        "invalid direct mcp_add stdio request should not spawn the fake claude binary"
    );
}

#[tokio::test]
async fn direct_claude_mcp_add_rejects_invalid_url_without_spawning() {
    if !claude_get_supported() {
        return;
    }

    for (label, raw) in [
        ("relative", "/relative"),
        ("missing_authority", "https:example.test/mcp"),
    ] {
        let sandbox = McpTestSandbox::new(&format!("direct_claude_mcp_add_invalid_url_{label}"))
            .expect("sandbox");
        let backend = claude_backend(&sandbox, true);

        let err = backend
            .mcp_add(AgentWrapperMcpAddRequest {
                name: "demo".to_string(),
                transport: AgentWrapperMcpAddTransport::Url {
                    url: raw.to_string(),
                    bearer_token_env_var: None,
                },
                context: AgentWrapperMcpCommandContext::default(),
            })
            .await
            .expect_err("invalid URLs must fail closed");

        assert_invalid_request(err, ERR_MCP_ADD_URL_INVALID, &[raw]);
        assert!(
            !sandbox.record_path().exists(),
            "invalid direct mcp_add url request should not spawn the fake claude binary"
        );
    }
}

#[tokio::test]
async fn direct_claude_mcp_add_rejects_bearer_env_var_without_spawning() {
    if !claude_get_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("direct_claude_mcp_add_bearer_env").expect("sandbox");
    let backend = claude_backend(&sandbox, true);

    let err = backend
        .mcp_add(AgentWrapperMcpAddRequest {
            name: "demo".to_string(),
            transport: AgentWrapperMcpAddTransport::Url {
                url: "https://example.test/mcp".to_string(),
                bearer_token_env_var: Some("TOKEN".to_string()),
            },
            context: AgentWrapperMcpCommandContext::default(),
        })
        .await
        .expect_err("bearer_token_env_var must stay rejected for claude");

    assert_invalid_request(err, ERR_CLAUDE_URL_BEARER_ENV_UNSUPPORTED, &["TOKEN"]);
    assert!(
        !sandbox.record_path().exists(),
        "claude-specific add rejection should happen before spawning the fake claude binary"
    );
}

#[tokio::test]
async fn direct_claude_mcp_add_trims_name_and_stdio_argv_before_spawn() {
    if !claude_get_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("direct_claude_mcp_add_trims_argv").expect("sandbox");
    let backend = claude_backend(&sandbox, true);

    let output = backend
        .mcp_add(AgentWrapperMcpAddRequest {
            name: "  demo-server  ".to_string(),
            transport: AgentWrapperMcpAddTransport::Stdio {
                command: vec!["  node  ".to_string()],
                args: vec!["  server.js  ".to_string(), "  --flag  ".to_string()],
                env: BTreeMap::from([("ALPHA_ENV".to_string(), "1".to_string())]),
            },
            context: AgentWrapperMcpCommandContext::default(),
        })
        .await
        .expect("trimmed direct mcp_add request should succeed");

    assert!(output.status.success(), "expected success status");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(
        record.args,
        vec![
            "mcp",
            "add",
            "--transport",
            "stdio",
            "--env",
            "ALPHA_ENV=1",
            "demo-server",
            "node",
            "server.js",
            "--flag",
        ]
    );
    assert!(
        record.args.iter().all(|arg| arg != "--"),
        "claude add stdio must not use the codex separator"
    );
}

#[tokio::test]
async fn direct_claude_mcp_add_preserves_capability_error_ordering() {
    let sandbox = McpTestSandbox::new("direct_claude_mcp_add_error_ordering").expect("sandbox");
    let backend = claude_backend(&sandbox, false);

    let err = backend
        .mcp_add(AgentWrapperMcpAddRequest {
            name: "   ".to_string(),
            transport: AgentWrapperMcpAddTransport::Stdio {
                command: vec!["   ".to_string()],
                args: Vec::new(),
                env: BTreeMap::new(),
            },
            context: AgentWrapperMcpCommandContext::default(),
        })
        .await
        .expect_err("write-disabled add should fail closed before normalization");

    assert_unsupported_capability(err, CAPABILITY_MCP_ADD_V1);
    assert!(
        !sandbox.record_path().exists(),
        "write-disabled direct mcp_add request should not spawn the fake claude binary"
    );
}

fn claude_backend(sandbox: &McpTestSandbox, allow_mcp_write: bool) -> ClaudeCodeBackend {
    let binary = sandbox.install_fake_claude().expect("install fake claude");
    ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(binary),
        claude_home: Some(sandbox.claude_home().to_path_buf()),
        env: claude_config_env(sandbox, std::iter::empty()),
        allow_mcp_write,
        ..Default::default()
    })
}

fn assert_invalid_request(
    err: AgentWrapperError,
    expected_message: &str,
    redacted_values: &[&str],
) {
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, expected_message);
            for value in redacted_values {
                assert!(
                    !message.contains(value),
                    "message leaked raw input `{value}`: {message}"
                );
            }
        }
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}
