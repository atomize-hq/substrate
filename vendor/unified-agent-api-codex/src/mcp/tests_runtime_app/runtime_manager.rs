use super::super::test_support::{prelude::*, *};
use super::super::*;

#[tokio::test]
async fn runtime_manager_starts_and_stops_stdio() {
    let (_dir, script) = write_env_probe_server("MCP_RUNTIME_ENV_E8");
    let code_home = tempfile::tempdir().expect("code_home");

    let defaults = StdioServerConfig {
        binary: PathBuf::from("codex"),
        code_home: Some(code_home.path().to_path_buf()),
        current_dir: None,
        env: vec![(
            OsString::from("MCP_RUNTIME_ENV_E8"),
            OsString::from("manager-ok"),
        )],
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(5),
    };

    let runtime = McpRuntimeServer {
        name: "env-probe".into(),
        transport: McpRuntimeTransport::Stdio(StdioServerDefinition {
            command: "python3".to_string(),
            args: vec![script.to_string_lossy().to_string()],
            env: BTreeMap::new(),
            timeout_ms: Some(1500),
        }),
        description: None,
        tags: vec!["local".into()],
        tools: Some(McpToolConfig {
            enabled: vec!["tool-x".into()],
            disabled: vec![],
        }),
    };

    let launcher = runtime.into_launcher(&defaults);
    let manager = McpRuntimeManager::new(vec![launcher]);

    let mut handle = match manager.prepare("env-probe").expect("prepare stdio") {
        McpRuntimeHandle::Stdio(handle) => handle,
        other => panic!("expected stdio handle, got {other:?}"),
    };

    let mut reader = BufReader::new(handle.stdout_mut());
    let mut line = String::new();
    let _ = time::timeout(Duration::from_secs(2), reader.read_line(&mut line))
        .await
        .expect("read timeout")
        .expect("read env line");
    assert_eq!(line.trim(), "manager-ok");

    let tools = handle.tools().expect("tool hints");
    assert_eq!(tools.enabled, vec!["tool-x".to_string()]);

    handle.stop().await.expect("stop server");
}

#[test]
fn runtime_manager_propagates_tool_hints_for_http() {
    let env_var = "MCP_HTTP_TOKEN_E8_HINTS";
    env::set_var(env_var, "token-hints");

    let mut http = StreamableHttpDefinition {
        url: "https://example.test/hints".into(),
        headers: BTreeMap::new(),
        bearer_env_var: Some(env_var.to_string()),
        connect_timeout_ms: Some(1200),
        request_timeout_ms: Some(2400),
    };
    http.headers.insert("X-Test".into(), "true".into());

    let runtime = McpRuntimeServer::from_definition(
        "remote-http",
        McpServerDefinition {
            transport: McpTransport::StreamableHttp(http),
            description: Some("http runtime".into()),
            tags: vec!["http".into()],
            tools: Some(McpToolConfig {
                enabled: vec!["alpha".into()],
                disabled: vec!["beta".into()],
            }),
        },
    );

    let defaults = StdioServerConfig {
        binary: PathBuf::from("codex"),
        code_home: None,
        current_dir: None,
        env: Vec::new(),
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(2),
    };

    let launcher = runtime.into_launcher(&defaults);
    let manager = McpRuntimeManager::new(vec![launcher]);

    let available = manager.available();
    assert_eq!(available.len(), 1);
    let summary = &available[0];
    assert_eq!(summary.name, "remote-http");
    assert_eq!(
        summary.transport,
        McpRuntimeSummaryTransport::StreamableHttp
    );
    let summary_tools = summary.tools.as_ref().expect("tool hints present");
    assert_eq!(summary_tools.enabled, vec!["alpha".to_string()]);
    assert_eq!(summary_tools.disabled, vec!["beta".to_string()]);

    match manager.prepare("remote-http").expect("prepare http") {
        McpRuntimeHandle::StreamableHttp(http_handle) => {
            let tools = http_handle.tools.as_ref().expect("tool hints on handle");
            assert_eq!(tools.enabled, vec!["alpha".to_string()]);
            assert_eq!(tools.disabled, vec!["beta".to_string()]);
            assert_eq!(
                http_handle.connector.bearer_token.as_deref(),
                Some("token-hints")
            );
        }
        other => panic!("expected http handle, got {other:?}"),
    }

    env::remove_var(env_var);
}

#[test]
fn http_connector_retrieval_is_non_destructive() {
    let env_var = "MCP_HTTP_TOKEN_E8_REUSE";
    env::set_var(env_var, "token-reuse");

    let runtime = McpRuntimeServer::from_definition(
        "remote-reuse",
        McpServerDefinition {
            transport: McpTransport::StreamableHttp(StreamableHttpDefinition {
                url: "https://example.test/reuse".into(),
                headers: BTreeMap::new(),
                bearer_env_var: Some(env_var.to_string()),
                connect_timeout_ms: Some(1500),
                request_timeout_ms: Some(3200),
            }),
            description: None,
            tags: vec!["http".into()],
            tools: Some(McpToolConfig {
                enabled: vec!["one".into()],
                disabled: vec![],
            }),
        },
    );

    let defaults = StdioServerConfig {
        binary: PathBuf::from("codex"),
        code_home: None,
        current_dir: None,
        env: Vec::new(),
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(2),
    };

    let launcher = runtime.into_launcher(&defaults);
    let manager = McpRuntimeManager::new(vec![launcher]);

    let first = manager.prepare("remote-reuse").expect("first prepare");
    let second = manager.prepare("remote-reuse").expect("second prepare");

    let first_token = match first {
        McpRuntimeHandle::StreamableHttp(handle) => handle.connector.bearer_token,
        other => panic!("expected http handle, got {other:?}"),
    };
    let second_token = match second {
        McpRuntimeHandle::StreamableHttp(handle) => handle.connector.bearer_token,
        other => panic!("expected http handle, got {other:?}"),
    };

    assert_eq!(first_token.as_deref(), Some("token-reuse"));
    assert_eq!(second_token.as_deref(), Some("token-reuse"));

    let summary = manager
        .available()
        .into_iter()
        .find(|s| s.name == "remote-reuse")
        .expect("summary present");
    assert_eq!(
        summary.transport,
        McpRuntimeSummaryTransport::StreamableHttp
    );
    let tools = summary.tools.as_ref().expect("tool hints preserved");
    assert_eq!(tools.enabled, vec!["one".to_string()]);

    env::remove_var(env_var);
}
