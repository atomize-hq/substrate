use std::collections::BTreeMap;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    mcp::{
        AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext,
        AgentWrapperMcpGetRequest, AgentWrapperMcpRemoveRequest,
    },
    AgentWrapperBackend, AgentWrapperError,
};

use super::support::McpTestSandbox;

const ERR_MCP_SERVER_NAME_EMPTY: &str = "mcp server name must be non-empty";
const ERR_MCP_ADD_STDIO_ITEM_EMPTY: &str = "mcp add stdio.command[0] must be non-empty";
const ERR_MCP_ADD_URL_INVALID: &str = "mcp add url must be an absolute http or https URL";
const FAKE_CODEX_RECORD_PATH_ENV: &str = "FAKE_CODEX_MCP_RECORD_PATH";
const FAKE_CODEX_RECORD_ENV_KEYS_ENV: &str = "FAKE_CODEX_MCP_RECORD_ENV_KEYS";
const ALL_RECORDED_ENV_KEYS: &str =
    "CLI_ONLY,CONFIG_ONLY,OVERRIDE_ME,REQUEST_ONLY,MY_TOKEN,MCP_SERVER_ENV";

#[tokio::test]
async fn direct_codex_mcp_get_rejects_empty_name_without_spawning() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("direct_codex_mcp_get_invalid_name").expect("sandbox");
    let backend = codex_backend(&sandbox, false);

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
        "invalid direct mcp_get request should not spawn the fake codex binary"
    );
}

#[tokio::test]
async fn direct_codex_mcp_remove_rejects_empty_name_without_spawning() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("direct_codex_mcp_remove_invalid_name").expect("sandbox");
    let backend = codex_backend(&sandbox, true);

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
        "invalid direct mcp_remove request should not spawn the fake codex binary"
    );
}

#[tokio::test]
async fn direct_codex_mcp_add_rejects_whitespace_stdio_command_without_spawning() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox =
        McpTestSandbox::new("direct_codex_mcp_add_invalid_stdio_command").expect("sandbox");
    let backend = codex_backend(&sandbox, true);

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
        "invalid direct mcp_add stdio request should not spawn the fake codex binary"
    );
}

#[tokio::test]
async fn direct_codex_mcp_add_rejects_invalid_url_without_spawning() {
    if !codex_mcp_supported() {
        return;
    }

    for (label, raw) in [
        ("relative", "/relative"),
        ("missing_authority", "https:example.test/mcp"),
    ] {
        let sandbox = McpTestSandbox::new(&format!("direct_codex_mcp_add_invalid_url_{label}"))
            .expect("sandbox");
        let backend = codex_backend(&sandbox, true);

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
            "invalid direct mcp_add url request should not spawn the fake codex binary"
        );
    }
}

#[tokio::test]
async fn direct_codex_mcp_add_trims_name_and_stdio_argv_before_spawn() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("direct_codex_mcp_add_trims_argv").expect("sandbox");
    let backend = codex_backend(&sandbox, true);

    let output = backend
        .mcp_add(AgentWrapperMcpAddRequest {
            name: "  demo-server  ".to_string(),
            transport: AgentWrapperMcpAddTransport::Stdio {
                command: vec!["  node  ".to_string()],
                args: vec!["  server.js  ".to_string(), "  --flag  ".to_string()],
                env: BTreeMap::from([("MCP_SERVER_ENV".to_string(), "1".to_string())]),
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
            "demo-server",
            "--env",
            "MCP_SERVER_ENV=1",
            "--",
            "node",
            "server.js",
            "--flag",
        ]
    );
}

fn codex_backend(sandbox: &McpTestSandbox, allow_mcp_write: bool) -> CodexBackend {
    let binary = sandbox.install_fake_codex().expect("install fake codex");
    CodexBackend::new(CodexBackendConfig {
        binary: Some(binary),
        codex_home: Some(sandbox.codex_home().to_path_buf()),
        env: codex_config_env(sandbox),
        allow_mcp_write,
        ..Default::default()
    })
}

fn codex_config_env(sandbox: &McpTestSandbox) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            FAKE_CODEX_RECORD_PATH_ENV.to_string(),
            sandbox.record_path().to_string_lossy().into_owned(),
        ),
        (
            FAKE_CODEX_RECORD_ENV_KEYS_ENV.to_string(),
            ALL_RECORDED_ENV_KEYS.to_string(),
        ),
    ])
}

fn codex_mcp_supported() -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
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
