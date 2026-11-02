#![allow(clippy::await_holding_lock)]

use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    mcp::{
        AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext,
        AgentWrapperMcpGetRequest, AgentWrapperMcpListRequest, AgentWrapperMcpRemoveRequest,
    },
    AgentWrapperBackend, AgentWrapperError, AgentWrapperGateway, AgentWrapperKind,
};

use super::support::McpTestSandbox;
use super::support::{process_env_lock, EnvGuard};

const FAKE_CODEX_RECORD_PATH_ENV: &str = "FAKE_CODEX_MCP_RECORD_PATH";
const FAKE_CODEX_RECORD_ENV_KEYS_ENV: &str = "FAKE_CODEX_MCP_RECORD_ENV_KEYS";
const FAKE_CODEX_SCENARIO_ENV: &str = "FAKE_CODEX_MCP_SCENARIO";
const ALL_RECORDED_ENV_KEYS: &str =
    "CLI_ONLY,CONFIG_ONLY,OVERRIDE_ME,REQUEST_ONLY,MY_TOKEN,MCP_SERVER_ENV";
const MCP_OUTPUT_BOUND_BYTES: usize = 65_536;
const TRUNCATION_SUFFIX: &str = "…(truncated)";
const TIMEOUT_STDOUT_SENTINEL: &str = "fake_codex_mcp timeout stdout sentinel";
const TIMEOUT_STDERR_SENTINEL: &str = "fake_codex_mcp timeout stderr sentinel";
const FAST_EXIT_STDOUT_SENTINEL: &str = "fake_codex_mcp fast-exit stdout sentinel";
const FAST_EXIT_STDERR_SENTINEL: &str = "fake_codex_mcp fast-exit stderr sentinel";
const CODEX_BINARY_ENV: &str = "CODEX_BINARY";
const PATH_ENV: &str = "PATH";

#[tokio::test]
async fn codex_mcp_nonzero_exit_returns_output_in_ok_result() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_nonzero_exit").expect("sandbox");
    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "nonzero_exit".to_string(),
            )],
        ),
        None,
    );

    let output = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect("non-zero exit must still produce Ok(output)");

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, "fake_codex_mcp nonzero stdout\n");
    assert_eq!(output.stderr, "fake_codex_mcp nonzero stderr\n");
    assert!(!output.stdout_truncated);
    assert!(!output.stderr_truncated);
}

#[tokio::test]
async fn codex_mcp_oversized_output_is_truncated_and_flagged() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_oversized_output").expect("sandbox");
    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "oversized_output".to_string(),
            )],
        ),
        None,
    );

    let output = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect("oversized output should still succeed");

    assert!(output.status.success(), "expected success status");
    assert!(output.stdout_truncated);
    assert!(output.stderr_truncated);
    assert_eq!(output.stdout.len(), MCP_OUTPUT_BOUND_BYTES);
    assert_eq!(output.stderr.len(), MCP_OUTPUT_BOUND_BYTES);
    assert!(output.stdout.ends_with(TRUNCATION_SUFFIX));
    assert!(output.stderr.ends_with(TRUNCATION_SUFFIX));
    assert!(output.stdout.starts_with("codex-mcp-stdout:"));
    assert!(output.stderr.starts_with("codex-mcp-stderr:"));
}

#[tokio::test]
async fn codex_mcp_timeout_returns_backend_error_without_leaking_partial_output() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_timeout").expect("sandbox");
    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "sleep_for_timeout".to_string(),
            )],
        ),
        None,
    );

    let err = gateway
        .mcp_list(
            &kind,
            AgentWrapperMcpListRequest {
                context: AgentWrapperMcpCommandContext {
                    timeout: Some(Duration::from_millis(50)),
                    ..Default::default()
                },
            },
        )
        .await
        .expect_err("timeout should return Backend error");

    let message = backend_error_message(err);
    assert!(
        !message.contains(TIMEOUT_STDOUT_SENTINEL),
        "timeout error leaked stdout sentinel: {message}"
    );
    assert!(
        !message.contains(TIMEOUT_STDERR_SENTINEL),
        "timeout error leaked stderr sentinel: {message}"
    );
    assert!(
        sandbox.record_path().exists(),
        "timeout path should have spawned the fake codex binary"
    );
}

#[tokio::test]
async fn codex_mcp_zero_timeout_returns_backend_error_without_leaking_partial_output() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_zero_timeout").expect("sandbox");
    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "sleep_for_timeout".to_string(),
            )],
        ),
        None,
    );

    let err = gateway
        .mcp_list(
            &kind,
            AgentWrapperMcpListRequest {
                context: AgentWrapperMcpCommandContext {
                    timeout: Some(Duration::ZERO),
                    ..Default::default()
                },
            },
        )
        .await
        .expect_err("zero timeout should return Backend error");

    let message = backend_error_message(err);
    assert!(
        !message.contains(TIMEOUT_STDOUT_SENTINEL),
        "zero-timeout error leaked stdout sentinel: {message}"
    );
    assert!(
        !message.contains(TIMEOUT_STDERR_SENTINEL),
        "zero-timeout error leaked stderr sentinel: {message}"
    );
    assert!(
        message.contains("timeout"),
        "zero-timeout failures should stay redacted but mention timeout: {message}"
    );
    assert!(
        !sandbox.record_path().exists(),
        "zero-timeout path must fail before spawning the fake codex binary"
    );
}

#[tokio::test]
async fn codex_mcp_zero_timeout_fast_exit_still_returns_timeout_error() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_zero_timeout_fast_exit").expect("sandbox");
    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "fast_exit_with_output".to_string(),
            )],
        ),
        None,
    );

    let err = gateway
        .mcp_list(
            &kind,
            AgentWrapperMcpListRequest {
                context: AgentWrapperMcpCommandContext {
                    timeout: Some(Duration::ZERO),
                    ..Default::default()
                },
            },
        )
        .await
        .expect_err("zero timeout should fail even for fast-exit commands");

    let message = backend_error_message(err);
    assert!(
        !message.contains(FAST_EXIT_STDOUT_SENTINEL),
        "zero-timeout error leaked fast-exit stdout sentinel: {message}"
    );
    assert!(
        !message.contains(FAST_EXIT_STDERR_SENTINEL),
        "zero-timeout error leaked fast-exit stderr sentinel: {message}"
    );
    assert!(
        message.contains("timeout"),
        "zero-timeout fast-exit failures should stay redacted but mention timeout: {message}"
    );
    assert!(
        !sandbox.record_path().exists(),
        "zero-timeout fast-exit path must fail before spawning the fake codex binary"
    );
}

#[tokio::test]
async fn codex_mcp_drift_json_rejection_returns_backend_error_without_mutating_capabilities() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_drift_json").expect("sandbox");
    let (backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [(FAKE_CODEX_SCENARIO_ENV.to_string(), "drift".to_string())],
        ),
        None,
    );
    let before = backend.capabilities().ids.clone();

    let err = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect_err("drift must fail closed");

    let message = backend_error_message(err);
    assert!(
        !message.contains("unexpected argument '--json'"),
        "drift error leaked subprocess stderr: {message}"
    );
    assert_eq!(backend.capabilities().ids, before);
}

#[tokio::test]
async fn codex_mcp_drift_unknown_subcommand_returns_backend_error_without_mutating_capabilities() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_drift_subcommand").expect("sandbox");
    let (backend, gateway, kind) = codex_gateway(
        &sandbox,
        true,
        codex_config_env(
            &sandbox,
            [(FAKE_CODEX_SCENARIO_ENV.to_string(), "drift".to_string())],
        ),
        None,
    );
    let before = backend.capabilities().ids.clone();

    let err = gateway
        .mcp_remove(
            &kind,
            AgentWrapperMcpRemoveRequest {
                name: "demo".to_string(),
                context: AgentWrapperMcpCommandContext::default(),
            },
        )
        .await
        .expect_err("drift must fail closed");

    let message = backend_error_message(err);
    assert!(
        !message.contains("unknown subcommand 'mcp'"),
        "drift error leaked subprocess stderr: {message}"
    );
    assert_eq!(backend.capabilities().ids, before);
}

#[tokio::test]
async fn codex_mcp_drift_unknown_get_subcommand_returns_backend_error_without_mutating_capabilities(
) {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_drift_get_subcommand").expect("sandbox");
    let (backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "operation_subcommand_drift".to_string(),
            )],
        ),
        None,
    );
    let before = backend.capabilities().ids.clone();

    let err = gateway
        .mcp_get(
            &kind,
            AgentWrapperMcpGetRequest {
                name: "demo".to_string(),
                context: AgentWrapperMcpCommandContext::default(),
            },
        )
        .await
        .expect_err("per-operation drift must fail closed");

    let message = backend_error_message(err);
    assert!(
        !message.contains("unknown subcommand 'get'"),
        "drift error leaked subprocess stderr: {message}"
    );
    assert_eq!(backend.capabilities().ids, before);
}

#[tokio::test]
async fn codex_mcp_drift_unknown_remove_subcommand_returns_backend_error_without_mutating_capabilities(
) {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_drift_remove_subcommand").expect("sandbox");
    let (backend, gateway, kind) = codex_gateway(
        &sandbox,
        true,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "operation_subcommand_drift".to_string(),
            )],
        ),
        None,
    );
    let before = backend.capabilities().ids.clone();

    let err = gateway
        .mcp_remove(
            &kind,
            AgentWrapperMcpRemoveRequest {
                name: "demo".to_string(),
                context: AgentWrapperMcpCommandContext::default(),
            },
        )
        .await
        .expect_err("per-operation drift must fail closed");

    let message = backend_error_message(err);
    assert!(
        !message.contains("unknown subcommand 'remove'"),
        "drift error leaked subprocess stderr: {message}"
    );
    assert_eq!(backend.capabilities().ids, before);
}

#[tokio::test]
async fn codex_mcp_add_legacy_usage_drift_returns_backend_error_without_mutating_capabilities() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_add_legacy_usage_drift").expect("sandbox");
    let (backend, gateway, kind) = codex_gateway(
        &sandbox,
        true,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "legacy_add_drift".to_string(),
            )],
        ),
        None,
    );
    let before = backend.capabilities().ids.clone();

    let err = gateway
        .mcp_add(
            &kind,
            AgentWrapperMcpAddRequest {
                name: "demo".to_string(),
                transport: AgentWrapperMcpAddTransport::Stdio {
                    command: vec!["node".to_string()],
                    args: vec!["server.js".to_string()],
                    env: BTreeMap::from([("MCP_SERVER_ENV".to_string(), "1".to_string())]),
                },
                context: AgentWrapperMcpCommandContext::default(),
            },
        )
        .await
        .expect_err("legacy add usage drift must fail closed");

    let message = backend_error_message(err);
    assert!(
        !message.contains("unexpected argument '--env'"),
        "drift error leaked subprocess stderr: {message}"
    );
    assert!(
        !message.contains("usage: codex mcp add"),
        "drift error leaked subprocess usage text: {message}"
    );
    assert_eq!(backend.capabilities().ids, before);
}

#[tokio::test]
async fn codex_mcp_add_url_flag_drift_returns_backend_error_without_mutating_capabilities() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_add_url_flag_drift").expect("sandbox");
    let (backend, gateway, kind) = codex_gateway(
        &sandbox,
        true,
        codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_SCENARIO_ENV.to_string(),
                "url_add_drift".to_string(),
            )],
        ),
        None,
    );
    let before = backend.capabilities().ids.clone();

    let err = gateway
        .mcp_add(
            &kind,
            AgentWrapperMcpAddRequest {
                name: "demo".to_string(),
                transport: AgentWrapperMcpAddTransport::Url {
                    url: "https://example.test/mcp".to_string(),
                    bearer_token_env_var: Some("MY_TOKEN".to_string()),
                },
                context: AgentWrapperMcpCommandContext::default(),
            },
        )
        .await
        .expect_err("url add flag drift must fail closed");

    let message = backend_error_message(err);
    assert!(
        !message.contains("unexpected argument '--url'"),
        "drift error leaked subprocess stderr: {message}"
    );
    assert!(
        !message.contains("unexpected argument '--bearer-token-env-var'"),
        "drift error leaked subprocess stderr: {message}"
    );
    assert_eq!(backend.capabilities().ids, before);
}

#[tokio::test]
async fn codex_mcp_missing_binary_returns_backend_error_without_writing_record() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_missing_binary").expect("sandbox");
    let missing_binary = sandbox
        .bin_dir()
        .join(platform_binary_name("missing-codex"));
    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(&sandbox, std::iter::empty()),
        Some(missing_binary),
    );

    let err = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect_err("missing binary must fail with Backend");

    let message = backend_error_message(err);
    assert!(
        message.contains("spawn"),
        "spawn failures should mention spawn in the redacted backend error: {message}"
    );
    assert!(
        !sandbox.record_path().exists(),
        "spawn failure must not create an invocation record"
    );
}

#[tokio::test]
async fn codex_mcp_unresolved_default_binary_returns_backend_error_without_writing_record() {
    if !codex_mcp_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("codex_mcp_unresolved_default_binary").expect("sandbox");
    let empty_path_dir = sandbox.root().join("empty-path");
    std::fs::create_dir_all(&empty_path_dir).expect("create empty PATH dir");
    let _ambient_path = EnvGuard::set(PATH_ENV, empty_path_dir.as_os_str().to_os_string());
    let _codex_binary = EnvGuard::unset(CODEX_BINARY_ENV);

    let backend = Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: None,
        codex_home: Some(sandbox.codex_home().to_path_buf()),
        env: codex_config_env(&sandbox, std::iter::empty()),
        ..Default::default()
    }));

    let kind = backend.kind();
    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).expect("register codex backend");

    let err = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect_err("unresolved default binary must fail with Backend");

    let message = backend_error_message(err);
    assert!(
        message.contains("spawn"),
        "spawn failures should mention spawn in the redacted backend error: {message}"
    );
    assert!(
        !sandbox.record_path().exists(),
        "pre-spawn resolution failure must not create an invocation record"
    );
}

fn codex_gateway(
    sandbox: &McpTestSandbox,
    allow_mcp_write: bool,
    env: BTreeMap<String, String>,
    binary_override: Option<PathBuf>,
) -> (Arc<CodexBackend>, AgentWrapperGateway, AgentWrapperKind) {
    let binary = binary_override.unwrap_or_else(|| {
        sandbox
            .install_fake_codex()
            .expect("install fake codex binary")
    });
    let backend = Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: Some(binary),
        codex_home: Some(sandbox.codex_home().to_path_buf()),
        env,
        allow_mcp_write,
        ..Default::default()
    }));

    let kind = backend.kind();
    let mut gateway = AgentWrapperGateway::new();
    gateway
        .register(backend.clone())
        .expect("register codex backend");
    (backend, gateway, kind)
}

fn codex_config_env(
    sandbox: &McpTestSandbox,
    extra: impl IntoIterator<Item = (String, String)>,
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([
        (
            FAKE_CODEX_RECORD_PATH_ENV.to_string(),
            sandbox.record_path().to_string_lossy().into_owned(),
        ),
        (
            FAKE_CODEX_RECORD_ENV_KEYS_ENV.to_string(),
            ALL_RECORDED_ENV_KEYS.to_string(),
        ),
    ]);
    env.extend(extra);
    env
}

fn backend_error_message(err: AgentWrapperError) -> String {
    match err {
        AgentWrapperError::Backend { message } => message,
        other => panic!("expected Backend error, got {other:?}"),
    }
}

fn codex_mcp_supported() -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
}

fn platform_binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
