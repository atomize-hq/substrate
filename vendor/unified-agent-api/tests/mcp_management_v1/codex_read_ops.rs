#![allow(clippy::await_holding_lock)]

use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc, time::Duration};

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    mcp::{AgentWrapperMcpCommandContext, AgentWrapperMcpGetRequest, AgentWrapperMcpListRequest},
    AgentWrapperBackend, AgentWrapperGateway, AgentWrapperKind,
};

use super::support::{process_env_lock, EnvGuard, McpTestSandbox};

const FAKE_CODEX_RECORD_PATH_ENV: &str = "FAKE_CODEX_MCP_RECORD_PATH";
const FAKE_CODEX_RECORD_ENV_KEYS_ENV: &str = "FAKE_CODEX_MCP_RECORD_ENV_KEYS";
const ALL_RECORDED_ENV_KEYS: &str =
    "CLI_ONLY,CONFIG_ONLY,OVERRIDE_ME,REQUEST_ONLY,MY_TOKEN,MCP_SERVER_ENV";
const CODEX_HOME_ENV: &str = "CODEX_HOME";
const MY_TOKEN_ENV: &str = "MY_TOKEN";
const CODEX_BINARY_ENV: &str = "CODEX_BINARY";
const PATH_ENV: &str = "PATH";

#[tokio::test]
async fn codex_mcp_list_records_pinned_argv_and_request_context() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_list_records_pinned_argv").expect("sandbox");
    let request_cwd = sandbox.root().join("list-request-cwd");
    let request_home = sandbox.root().join("list-request-home");
    fs::create_dir_all(&request_cwd).expect("create request cwd");
    fs::create_dir_all(&request_home).expect("create request home");

    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [
                ("CONFIG_ONLY".to_string(), "config-only".to_string()),
                ("OVERRIDE_ME".to_string(), "config-value".to_string()),
            ],
        ),
        Some(sandbox.root().join("default-list-cwd")),
        Some(Duration::from_secs(5)),
    );

    let output = gateway
        .mcp_list(
            &kind,
            AgentWrapperMcpListRequest {
                context: AgentWrapperMcpCommandContext {
                    working_dir: Some(request_cwd.clone()),
                    timeout: Some(Duration::from_millis(50)),
                    env: BTreeMap::from([
                        ("CLI_ONLY".to_string(), "cli-value".to_string()),
                        ("OVERRIDE_ME".to_string(), "request-value".to_string()),
                        ("REQUEST_ONLY".to_string(), "request-only".to_string()),
                        (
                            "CODEX_HOME".to_string(),
                            request_home.to_string_lossy().into_owned(),
                        ),
                    ]),
                },
            },
        )
        .await
        .expect("mcp list should succeed");

    assert!(output.status.success(), "expected success status");
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "list", "--json"]);
    assert_eq!(record.cwd, request_cwd);
    assert_eq!(
        record.env.get("CODEX_HOME").map(String::as_str),
        Some(request_home.to_string_lossy().as_ref())
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
async fn codex_mcp_get_records_pinned_argv_and_request_context() {
    if !codex_mcp_supported() {
        return;
    }

    let sandbox = McpTestSandbox::new("codex_mcp_get_records_pinned_argv").expect("sandbox");
    let request_cwd = sandbox.root().join("get-request-cwd");
    let request_home = sandbox.root().join("get-request-home");
    fs::create_dir_all(&request_cwd).expect("create request cwd");
    fs::create_dir_all(&request_home).expect("create request home");

    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [
                ("CONFIG_ONLY".to_string(), "config-only".to_string()),
                ("OVERRIDE_ME".to_string(), "config-value".to_string()),
            ],
        ),
        Some(sandbox.root().join("default-get-cwd")),
        Some(Duration::from_secs(5)),
    );

    let output = gateway
        .mcp_get(
            &kind,
            AgentWrapperMcpGetRequest {
                name: "demo".to_string(),
                context: AgentWrapperMcpCommandContext {
                    working_dir: Some(request_cwd.clone()),
                    timeout: Some(Duration::from_millis(50)),
                    env: BTreeMap::from([
                        ("CLI_ONLY".to_string(), "cli-value".to_string()),
                        ("OVERRIDE_ME".to_string(), "request-value".to_string()),
                        ("REQUEST_ONLY".to_string(), "request-only".to_string()),
                        (
                            "CODEX_HOME".to_string(),
                            request_home.to_string_lossy().into_owned(),
                        ),
                    ]),
                },
            },
        )
        .await
        .expect("mcp get should succeed");

    assert!(output.status.success(), "expected success status");
    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, "");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "get", "--json", "demo"]);
    assert_eq!(record.cwd, request_cwd);
    assert_eq!(
        record.env.get("CODEX_HOME").map(String::as_str),
        Some(request_home.to_string_lossy().as_ref())
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
async fn codex_mcp_list_does_not_inherit_ambient_env_outside_resolved_context() {
    if !codex_mcp_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("codex_mcp_list_no_ambient_env").expect("sandbox");
    let ambient_home = sandbox.root().join("ambient-codex-home");
    let _ambient_home = EnvGuard::set(CODEX_HOME_ENV, ambient_home.as_os_str().to_os_string());
    let _ambient_token = EnvGuard::set(MY_TOKEN_ENV, "ambient-secret");

    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
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
        record.env.get(CODEX_HOME_ENV).map(String::as_str),
        Some(sandbox.codex_home().to_string_lossy().as_ref())
    );
    assert!(
        !record.env.contains_key(MY_TOKEN_ENV),
        "ambient bearer token env must not leak into the spawned codex process"
    );
}

#[tokio::test]
async fn codex_mcp_get_does_not_inherit_ambient_env_outside_resolved_context() {
    if !codex_mcp_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("codex_mcp_get_no_ambient_env").expect("sandbox");
    let ambient_home = sandbox.root().join("ambient-codex-home");
    let _ambient_home = EnvGuard::set(CODEX_HOME_ENV, ambient_home.as_os_str().to_os_string());
    let _ambient_token = EnvGuard::set(MY_TOKEN_ENV, "ambient-secret");

    let (_backend, gateway, kind) = codex_gateway(
        &sandbox,
        false,
        codex_config_env(
            &sandbox,
            [("CONFIG_ONLY".to_string(), "config-only".to_string())],
        ),
        None,
        None,
    );

    let output = gateway
        .mcp_get(
            &kind,
            AgentWrapperMcpGetRequest {
                name: "demo".to_string(),
                context: AgentWrapperMcpCommandContext::default(),
            },
        )
        .await
        .expect("mcp get should succeed");

    assert!(output.status.success(), "expected success status");

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(
        record.env.get(CODEX_HOME_ENV).map(String::as_str),
        Some(sandbox.codex_home().to_string_lossy().as_ref())
    );
    assert!(
        !record.env.contains_key(MY_TOKEN_ENV),
        "ambient bearer token env must not leak into the spawned codex process"
    );
}

#[tokio::test]
async fn codex_mcp_list_resolves_ambient_path_before_env_clear() {
    if !codex_mcp_supported() {
        return;
    }

    let _env_lock = process_env_lock().lock().expect("lock process env");
    let sandbox = McpTestSandbox::new("codex_mcp_list_resolves_ambient_path").expect("sandbox");
    let fake_codex = sandbox.install_fake_codex().expect("install fake codex");
    let _ambient_path = EnvGuard::set(PATH_ENV, sandbox.bin_dir().as_os_str().to_os_string());
    let _ambient_token = EnvGuard::set(MY_TOKEN_ENV, "ambient-secret");
    let _ambient_binary = EnvGuard::unset(CODEX_BINARY_ENV);

    let backend = Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: None,
        codex_home: Some(sandbox.codex_home().to_path_buf()),
        env: codex_config_env(
            &sandbox,
            [(
                FAKE_CODEX_RECORD_ENV_KEYS_ENV.to_string(),
                format!("{ALL_RECORDED_ENV_KEYS},PATH"),
            )],
        ),
        ..Default::default()
    }));

    let kind = backend.kind();
    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).expect("register codex backend");

    let output = gateway
        .mcp_list(&kind, AgentWrapperMcpListRequest::default())
        .await
        .expect("supported list should succeed");

    assert!(output.status.success(), "expected success status");
    assert!(
        fake_codex.is_file(),
        "fake codex must exist on ambient PATH"
    );

    let record = sandbox
        .read_single_record()
        .expect("single invocation record");
    assert_eq!(record.args, vec!["mcp", "list", "--json"]);
    assert_eq!(
        record.env.get(PATH_ENV).map(String::as_str),
        Some(sandbox.bin_dir().to_string_lossy().as_ref())
    );
    assert!(
        !record.env.contains_key(MY_TOKEN_ENV),
        "ambient bearer token env must not leak into the spawned codex process"
    );
}

fn codex_gateway(
    sandbox: &McpTestSandbox,
    allow_mcp_write: bool,
    env: BTreeMap<String, String>,
    default_working_dir: Option<PathBuf>,
    default_timeout: Option<Duration>,
) -> (Arc<CodexBackend>, AgentWrapperGateway, AgentWrapperKind) {
    let binary = sandbox.install_fake_codex().expect("install fake codex");
    let backend = Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: Some(binary),
        codex_home: Some(sandbox.codex_home().to_path_buf()),
        default_timeout,
        default_working_dir,
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

fn codex_mcp_supported() -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
}
