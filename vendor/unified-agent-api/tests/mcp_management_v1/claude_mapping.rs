#![allow(clippy::await_holding_lock)]

use std::{collections::BTreeMap, fs, sync::Arc, time::Duration};

use agent_api::backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig};
use agent_api::mcp::{
    AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext,
    AgentWrapperMcpGetRequest, AgentWrapperMcpListRequest, AgentWrapperMcpRemoveRequest,
};
use agent_api::{AgentWrapperBackend, AgentWrapperGateway};

use super::{
    claude_support::{
        claude_config_env, claude_gateway, claude_gateway_with_home, claude_get_supported,
        claude_list_supported, ALL_RECORDED_ENV_KEYS, FAKE_CLAUDE_SCENARIO_ENV,
    },
    support::{
        collect_fake_mcp_sentinels, fake_mcp_sentinel, process_env_lock,
        EnvGuard as SupportEnvGuard, McpTestSandbox,
    },
};

const MCP_OUTPUT_BOUND_BYTES: usize = 65_536;
const TRUNCATION_SUFFIX: &str = "…(truncated)";
const CLAUDE_BINARY_ENV: &str = "CLAUDE_BINARY";
const CLAUDE_HOME_ENV: &str = "CLAUDE_HOME";
const MY_TOKEN_ENV: &str = "MY_TOKEN";
const PATH_ENV: &str = "PATH";

#[tokio::test]
async fn claude_mcp_list_records_pinned_argv_and_request_context_on_supported_targets() {
    if !claude_list_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_list_records_pinned_argv").expect("sandbox");
    let default_cwd = sandbox.root().join("default-list-cwd");
    let request_cwd = sandbox.root().join("request-list-cwd");
    fs::create_dir_all(&default_cwd).expect("create default cwd");
    fs::create_dir_all(&request_cwd).expect("create request cwd");

    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(
            &sandbox,
            [
                ("CONFIG_ONLY".to_string(), "config-only".to_string()),
                ("OVERRIDE_ME".to_string(), "config-value".to_string()),
            ],
        ),
        Some(default_cwd),
        Some(Duration::from_secs(5)),
    );

    let output = gateway
        .mcp_list(
            &kind,
            AgentWrapperMcpListRequest {
                context: AgentWrapperMcpCommandContext {
                    working_dir: Some(request_cwd.clone()),
                    env: BTreeMap::from([
                        ("CLI_ONLY".to_string(), "cli-value".to_string()),
                        ("OVERRIDE_ME".to_string(), "request-value".to_string()),
                        ("REQUEST_ONLY".to_string(), "request-only".to_string()),
                    ]),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("supported list should succeed");

    assert!(output.status.success(), "expected success status");
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "list"]);
    assert_eq!(
        fs::canonicalize(&record.cwd).expect("canonicalize recorded cwd"),
        fs::canonicalize(&request_cwd).expect("canonicalize request cwd")
    );
    assert_eq!(
        record.env.get("CONFIG_ONLY").map(String::as_str),
        Some("config-only")
    );
    assert_eq!(
        record.env.get("OVERRIDE_ME").map(String::as_str),
        Some("request-value")
    );
    assert_eq!(
        record.env.get("REQUEST_ONLY").map(String::as_str),
        Some("request-only")
    );
    assert_eq!(
        record.env.get("CLI_ONLY").map(String::as_str),
        Some("cli-value")
    );
}

#[tokio::test]
async fn claude_mcp_list_does_not_inherit_ambient_env_outside_resolved_context() {
    if !claude_list_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let _ambient_token = SupportEnvGuard::set(MY_TOKEN_ENV, "ambient-secret");

    let sandbox = McpTestSandbox::new("claude_mcp_list_no_ambient_env").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(
            &sandbox,
            [("CONFIG_ONLY".to_string(), "config-only".to_string())],
        ),
        None,
        None,
    );

    let output = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect("mcp list should succeed");

    assert!(output.status.success(), "expected success status");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(
        record.env.get("CLAUDE_HOME").map(String::as_str),
        Some(sandbox.claude_home().to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("HOME").map(String::as_str),
        Some(sandbox.claude_home().to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("CONFIG_ONLY").map(String::as_str),
        Some("config-only")
    );
    assert!(
        !record.env.contains_key(MY_TOKEN_ENV),
        "ambient bearer token env must not leak into the spawned claude process"
    );
}

#[tokio::test]
async fn claude_mcp_list_request_env_overrides_injected_home_and_xdg_values() {
    if !claude_list_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_list_env_override").expect("sandbox");
    let fresh_claude_home = sandbox.root().join("fresh-claude-home");
    let override_home = sandbox.root().join("override-home");
    let override_xdg_config = sandbox.root().join("override-xdg-config");
    let override_xdg_data = sandbox.root().join("override-xdg-data");
    let override_xdg_cache = sandbox.root().join("override-xdg-cache");
    for dir in [
        &override_home,
        &override_xdg_config,
        &override_xdg_data,
        &override_xdg_cache,
    ] {
        fs::create_dir_all(dir).expect("create override dir");
    }
    assert!(
        !fresh_claude_home.exists(),
        "test requires an unmaterialized configured claude_home"
    );

    let (_backend, gateway, kind) = claude_gateway_with_home(
        &sandbox,
        fresh_claude_home.clone(),
        false,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let output = gateway
        .mcp_list(
            &kind,
            AgentWrapperMcpListRequest {
                context: AgentWrapperMcpCommandContext {
                    env: BTreeMap::from([
                        (
                            "HOME".to_string(),
                            override_home.to_string_lossy().into_owned(),
                        ),
                        (
                            "XDG_CONFIG_HOME".to_string(),
                            override_xdg_config.to_string_lossy().into_owned(),
                        ),
                        (
                            "XDG_DATA_HOME".to_string(),
                            override_xdg_data.to_string_lossy().into_owned(),
                        ),
                        (
                            "XDG_CACHE_HOME".to_string(),
                            override_xdg_cache.to_string_lossy().into_owned(),
                        ),
                    ]),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("supported list should succeed");

    assert!(output.status.success(), "expected success status");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "list"]);
    assert_eq!(
        record.env.get("CLAUDE_HOME").map(String::as_str),
        Some(fresh_claude_home.to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("HOME").map(String::as_str),
        Some(override_home.to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("XDG_CONFIG_HOME").map(String::as_str),
        Some(override_xdg_config.to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("XDG_DATA_HOME").map(String::as_str),
        Some(override_xdg_data.to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("XDG_CACHE_HOME").map(String::as_str),
        Some(override_xdg_cache.to_string_lossy().as_ref())
    );
    assert!(
        fresh_claude_home.is_dir(),
        "configured claude_home should be materialized when CLAUDE_HOME still targets it"
    );
    assert!(
        fresh_claude_home.join(".config").is_dir(),
        "configured claude_home should prepare its default xdg config dir"
    );
    assert!(
        fresh_claude_home.join(".local").join("share").is_dir(),
        "configured claude_home should prepare its default xdg data dir"
    );
    assert!(
        fresh_claude_home.join(".cache").is_dir(),
        "configured claude_home should prepare its default xdg cache dir"
    );
}

#[tokio::test]
async fn claude_mcp_list_uses_ambient_claude_home_when_config_home_is_absent() {
    if !claude_list_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_list_uses_ambient_claude_home").expect("sandbox");
    let binary = sandbox.install_fake_claude().expect("install fake claude");
    let ambient_claude_home = sandbox.root().join("ambient-claude-home");
    assert!(
        !ambient_claude_home.exists(),
        "test requires an unmaterialized ambient claude home"
    );

    let _ambient_claude_home = SupportEnvGuard::set(
        CLAUDE_HOME_ENV,
        ambient_claude_home.as_os_str().to_os_string(),
    );

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(binary),
        claude_home: None,
        env: claude_config_env(&sandbox, std::iter::empty()),
        ..Default::default()
    });

    let kind = backend.kind();
    let mut gateway = AgentWrapperGateway::new();
    gateway
        .register(Arc::new(backend))
        .expect("register claude backend");

    let output = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect("supported list should succeed");

    assert!(output.status.success(), "expected success status");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(
        record.env.get("CLAUDE_HOME").map(String::as_str),
        Some(ambient_claude_home.to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("HOME").map(String::as_str),
        Some(ambient_claude_home.to_string_lossy().as_ref())
    );
    assert_eq!(
        record.env.get("XDG_CONFIG_HOME").map(String::as_str),
        Some(
            ambient_claude_home
                .join(".config")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert_eq!(
        record.env.get("XDG_DATA_HOME").map(String::as_str),
        Some(
            ambient_claude_home
                .join(".local")
                .join("share")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert_eq!(
        record.env.get("XDG_CACHE_HOME").map(String::as_str),
        Some(
            ambient_claude_home
                .join(".cache")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert!(
        ambient_claude_home.is_dir(),
        "ambient claude home should exist"
    );
    assert!(
        ambient_claude_home.join(".config").is_dir(),
        "ambient xdg config dir should exist"
    );
    assert!(
        ambient_claude_home.join(".local").join("share").is_dir(),
        "ambient xdg data dir should exist"
    );
    assert!(
        ambient_claude_home.join(".cache").is_dir(),
        "ambient xdg cache dir should exist"
    );
}

#[tokio::test]
async fn claude_mcp_list_resolves_ambient_path_before_env_clear() {
    if !claude_list_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_list_resolves_ambient_path").expect("sandbox");
    let fake_claude = sandbox.install_fake_claude().expect("install fake claude");
    let _ambient_path =
        SupportEnvGuard::set(PATH_ENV, sandbox.bin_dir().as_os_str().to_os_string());
    let _ambient_token = SupportEnvGuard::set(MY_TOKEN_ENV, "ambient-secret");
    let _ambient_binary = SupportEnvGuard::unset(CLAUDE_BINARY_ENV);

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: None,
        claude_home: Some(sandbox.claude_home().to_path_buf()),
        env: claude_config_env(
            &sandbox,
            [(
                "FAKE_CLAUDE_MCP_RECORD_ENV_KEYS".to_string(),
                format!("{ALL_RECORDED_ENV_KEYS},PATH"),
            )],
        ),
        ..Default::default()
    });

    let kind = backend.kind();
    let mut gateway = AgentWrapperGateway::new();
    gateway
        .register(Arc::new(backend))
        .expect("register claude backend");

    let output = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect("supported list should succeed");

    assert!(output.status.success(), "expected success status");
    assert!(
        fake_claude.is_file(),
        "fake claude must exist on ambient PATH"
    );

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "list"]);
    assert_eq!(
        record.env.get(PATH_ENV).map(String::as_str),
        Some(sandbox.bin_dir().to_string_lossy().as_ref())
    );
    assert!(
        !record.env.contains_key(MY_TOKEN_ENV),
        "ambient bearer token env must not leak into the spawned claude process"
    );
}

#[tokio::test]
async fn claude_mcp_get_records_pinned_argv_on_win32_x64() {
    if !claude_get_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_get_records_pinned_argv").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let output = gateway
        .mcp_get(
            &kind,
            AgentWrapperMcpGetRequest {
                name: "demo".to_string(),
                context: Default::default(),
            },
        )
        .await
        .expect("supported get should succeed");

    assert!(output.status.success(), "expected success status");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "get", "demo"]);
}

#[tokio::test]
async fn claude_mcp_add_stdio_records_sorted_env_without_separator_and_writes_sentinel() {
    if !claude_get_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_add_stdio_records").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        true,
        claude_config_env(
            &sandbox,
            [("CLI_ONLY".to_string(), "config-cli".to_string())],
        ),
        None,
        None,
    );

    let output = gateway
        .mcp_add(
            &kind,
            AgentWrapperMcpAddRequest {
                name: "demo".to_string(),
                transport: AgentWrapperMcpAddTransport::Stdio {
                    command: vec!["node".to_string()],
                    args: vec!["server.js".to_string()],
                    env: BTreeMap::from([
                        ("ZETA_ENV".to_string(), "2".to_string()),
                        ("ALPHA_ENV".to_string(), "1".to_string()),
                    ]),
                },
                context: AgentWrapperMcpCommandContext {
                    env: BTreeMap::from([("CLI_ONLY".to_string(), "cli-value".to_string())]),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("supported stdio add should succeed");

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
            "--env",
            "ZETA_ENV=2",
            "demo",
            "node",
            "server.js",
        ]
    );
    assert!(
        record.args.iter().all(|arg| arg != "--"),
        "claude add stdio must not use the codex separator"
    );
    assert!(
        !record.env.contains_key("ALPHA_ENV") && !record.env.contains_key("ZETA_ENV"),
        "stdio transport env must not leak into the CLI process env"
    );
    assert_eq!(
        record.env.get("CLI_ONLY").map(String::as_str),
        Some("cli-value")
    );

    let expected = fake_mcp_sentinel(sandbox.claude_home(), "add");
    assert!(
        expected.is_file(),
        "expected add sentinel at {:?}",
        expected
    );
    assert_eq!(
        collect_fake_mcp_sentinels(sandbox.root()).expect("collect sentinels"),
        vec![expected]
    );
}

#[tokio::test]
async fn claude_mcp_add_url_records_pinned_argv_on_win32_x64() {
    if !claude_get_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_add_url_records").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        true,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let output = gateway
        .mcp_add(
            &kind,
            AgentWrapperMcpAddRequest {
                name: "demo".to_string(),
                transport: AgentWrapperMcpAddTransport::Url {
                    url: "https://example.test/mcp".to_string(),
                    bearer_token_env_var: None,
                },
                context: Default::default(),
            },
        )
        .await
        .expect("supported url add should succeed");

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
            "http",
            "demo",
            "https://example.test/mcp",
        ]
    );
}

#[tokio::test]
async fn claude_mcp_remove_records_pinned_argv_and_writes_sentinel_on_win32_x64() {
    if !claude_get_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_remove_records").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        true,
        claude_config_env(&sandbox, std::iter::empty()),
        None,
        None,
    );

    let output = gateway
        .mcp_remove(
            &kind,
            AgentWrapperMcpRemoveRequest {
                name: "demo".to_string(),
                context: Default::default(),
            },
        )
        .await
        .expect("supported remove should succeed");

    assert!(output.status.success(), "expected success status");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "remove", "demo"]);

    let expected = fake_mcp_sentinel(sandbox.claude_home(), "remove");
    assert!(
        expected.is_file(),
        "expected remove sentinel at {:?}",
        expected
    );
    assert_eq!(
        collect_fake_mcp_sentinels(sandbox.root()).expect("collect sentinels"),
        vec![expected]
    );
}

#[tokio::test]
async fn claude_mcp_add_zero_timeout_fails_before_spawn_or_write() {
    if !claude_get_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_add_zero_timeout").expect("sandbox");
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
                    env: BTreeMap::new(),
                },
                context: AgentWrapperMcpCommandContext {
                    timeout: Some(Duration::ZERO),
                    ..Default::default()
                },
            },
        )
        .await
        .expect_err("zero-timeout add should fail before spawning");

    let message = super::claude_support::backend_error_message(err);
    assert!(
        message.contains("timeout"),
        "zero-timeout add should return a timeout backend error: {message}"
    );
    assert!(
        !sandbox.record_path().exists(),
        "zero-timeout add must not create an invocation record"
    );
    assert!(
        collect_fake_mcp_sentinels(sandbox.root())
            .expect("collect sentinels")
            .is_empty(),
        "zero-timeout add must not write MCP sentinels"
    );
}

#[tokio::test]
async fn claude_mcp_remove_zero_timeout_fails_before_spawn_or_write() {
    if !claude_get_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_remove_zero_timeout").expect("sandbox");
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
                context: AgentWrapperMcpCommandContext {
                    timeout: Some(Duration::ZERO),
                    ..Default::default()
                },
            },
        )
        .await
        .expect_err("zero-timeout remove should fail before spawning");

    let message = super::claude_support::backend_error_message(err);
    assert!(
        message.contains("timeout"),
        "zero-timeout remove should return a timeout backend error: {message}"
    );
    assert!(
        !sandbox.record_path().exists(),
        "zero-timeout remove must not create an invocation record"
    );
    assert!(
        collect_fake_mcp_sentinels(sandbox.root())
            .expect("collect sentinels")
            .is_empty(),
        "zero-timeout remove must not write MCP sentinels"
    );
}

#[tokio::test]
async fn claude_mcp_oversized_output_is_truncated_and_flagged_on_supported_targets() {
    if !claude_list_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("claude_mcp_oversized_output").expect("sandbox");
    let (_backend, gateway, kind) = claude_gateway(
        &sandbox,
        false,
        claude_config_env(
            &sandbox,
            [(
                FAKE_CLAUDE_SCENARIO_ENV.to_string(),
                "oversized_output".to_string(),
            )],
        ),
        None,
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
    assert!(output.stdout.starts_with("claude-mcp-stdout:"));
    assert!(output.stderr.starts_with("claude-mcp-stderr:"));
}
