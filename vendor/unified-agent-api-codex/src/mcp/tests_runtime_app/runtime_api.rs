use super::super::test_support::{prelude::*, *};
use super::super::*;

#[test]
fn runtime_api_lists_launchers_without_changing_config() {
    let (dir, manager) = temp_config_manager();
    let stdio_env_key = "MCP_RUNTIME_API_STDIO_ENV";
    let request_env_key = "MCP_RUNTIME_API_REQUEST_ENV";
    let http_env_key = "MCP_RUNTIME_API_HTTP_ENV";
    env::set_var(http_env_key, "token-api");

    let mut stdio = stdio_definition("runtime-api-stdio");
    stdio.description = Some("stdio runtime".into());
    stdio.tags = vec!["local".into()];
    stdio.tools = Some(McpToolConfig {
        enabled: vec!["fmt".into()],
        disabled: vec!["lint".into()],
    });
    if let McpTransport::Stdio(ref mut stdio_def) = stdio.transport {
        stdio_def.args.push("--flag".into());
        stdio_def
            .env
            .insert(stdio_env_key.into(), "runtime-env".into());
        stdio_def.timeout_ms = Some(2400);
    }

    let mut env_map = BTreeMap::new();
    env_map.insert(request_env_key.to_string(), "injected".to_string());

    manager
        .add_server(AddServerRequest {
            name: "local-api".into(),
            definition: stdio,
            overwrite: false,
            env: env_map,
            bearer_token: None,
        })
        .expect("add stdio server");

    let mut http = streamable_definition("https://example.test/runtime-api", http_env_key);
    http.description = Some("http runtime".into());
    http.tags = vec!["remote".into()];
    http.tools = Some(McpToolConfig {
        enabled: vec!["alpha".into()],
        disabled: vec!["beta".into()],
    });
    if let McpTransport::StreamableHttp(ref mut http_def) = http.transport {
        http_def.headers.insert("X-Req".into(), "true".into());
        http_def.request_timeout_ms = Some(2200);
    }

    manager
        .add_server(AddServerRequest {
            name: "remote-api".into(),
            definition: http,
            overwrite: false,
            env: BTreeMap::new(),
            bearer_token: None,
        })
        .expect("add http server");

    let before = fs::read_to_string(manager.config_path()).expect("read config before");
    let cwd = dir.path().join("cwd");

    let defaults = StdioServerConfig {
        binary: PathBuf::from("codex"),
        code_home: Some(dir.path().to_path_buf()),
        current_dir: Some(cwd.clone()),
        env: vec![
            (OsString::from("DEFAULT_ONLY"), OsString::from("default")),
            (
                OsString::from(request_env_key),
                OsString::from("base-default"),
            ),
        ],
        app_server_analytics_default_enabled: false,
        mirror_stdio: true,
        startup_timeout: Duration::from_secs(3),
    };

    let api = McpRuntimeApi::from_config(&manager, &defaults).expect("runtime api");

    let available = api.available();
    assert_eq!(available.len(), 2);

    let stdio_summary = available
        .iter()
        .find(|entry| entry.name == "local-api")
        .expect("stdio summary");
    assert_eq!(stdio_summary.transport, McpRuntimeSummaryTransport::Stdio);
    let stdio_tools = stdio_summary.tools.as_ref().expect("stdio tools");
    assert_eq!(stdio_tools.enabled, vec!["fmt".to_string()]);
    assert_eq!(stdio_tools.disabled, vec!["lint".to_string()]);

    let stdio_launcher = api.stdio_launcher("local-api").expect("stdio launcher");
    assert_eq!(stdio_launcher.args, vec!["--flag".to_string()]);
    assert_eq!(stdio_launcher.timeout, Duration::from_millis(2400));
    assert!(stdio_launcher.mirror_stdio);
    assert_eq!(stdio_launcher.current_dir.as_deref(), Some(cwd.as_path()));

    let env_map: HashMap<OsString, OsString> = stdio_launcher.env.into_iter().collect();
    assert_eq!(
        env_map.get(&OsString::from("CODEX_HOME")),
        Some(&dir.path().as_os_str().to_os_string())
    );
    assert_eq!(
        env_map.get(&OsString::from("DEFAULT_ONLY")),
        Some(&OsString::from("default"))
    );
    assert_eq!(
        env_map.get(&OsString::from(request_env_key)),
        Some(&OsString::from("injected"))
    );
    assert_eq!(
        env_map.get(&OsString::from(stdio_env_key)),
        Some(&OsString::from("runtime-env"))
    );

    let http_connector = api.http_connector("remote-api").expect("http connector");
    assert_eq!(http_connector.bearer_token.as_deref(), Some("token-api"));
    assert_eq!(
        http_connector
            .headers
            .get("Authorization")
            .map(String::as_str),
        Some("Bearer token-api")
    );
    assert_eq!(
        http_connector.headers.get("X-Req").map(String::as_str),
        Some("true")
    );
    assert_eq!(
        http_connector.request_timeout,
        Some(Duration::from_millis(2200))
    );

    let http_tools = available
        .iter()
        .find(|entry| entry.name == "remote-api")
        .and_then(|entry| entry.tools.as_ref())
        .expect("http tools");
    assert_eq!(http_tools.enabled, vec!["alpha".to_string()]);
    assert_eq!(http_tools.disabled, vec!["beta".to_string()]);

    match api.stdio_launcher("remote-api") {
        Err(McpRuntimeError::UnsupportedTransport {
            name,
            expected,
            actual,
        }) => {
            assert_eq!(name, "remote-api");
            assert_eq!(expected, "stdio");
            assert_eq!(actual, "streamable_http");
        }
        other => panic!("unexpected result: {other:?}"),
    }

    match api.http_connector("local-api") {
        Err(McpRuntimeError::UnsupportedTransport {
            name,
            expected,
            actual,
        }) => {
            assert_eq!(name, "local-api");
            assert_eq!(expected, "streamable_http");
            assert_eq!(actual, "stdio");
        }
        other => panic!("unexpected http result: {other:?}"),
    }

    let after = fs::read_to_string(manager.config_path()).expect("read config after");
    assert_eq!(before, after);

    env::remove_var(http_env_key);
    env::remove_var(request_env_key);
}

#[test]
fn runtime_api_prepare_http_is_non_destructive() {
    let (dir, manager) = temp_config_manager();
    let env_var = "MCP_RUNTIME_API_PREPARE";
    env::set_var(env_var, "prepare-token");

    let mut http = streamable_definition("https://example.test/prepare", env_var);
    http.tags = vec!["prepare".into()];
    http.tools = Some(McpToolConfig {
        enabled: vec!["delta".into()],
        disabled: vec![],
    });

    manager
        .add_server(AddServerRequest {
            name: "prepare-http".into(),
            definition: http,
            overwrite: false,
            env: BTreeMap::new(),
            bearer_token: None,
        })
        .expect("add http server");

    let before = fs::read_to_string(manager.config_path()).expect("read config before");

    let defaults = StdioServerConfig {
        binary: PathBuf::from("codex"),
        code_home: Some(dir.path().to_path_buf()),
        current_dir: None,
        env: Vec::new(),
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(2),
    };

    let api = McpRuntimeApi::from_config(&manager, &defaults).expect("runtime api");
    let handle = api.prepare("prepare-http").expect("prepare http");

    match handle {
        McpRuntimeHandle::StreamableHttp(http_handle) => {
            assert_eq!(http_handle.name, "prepare-http");
            assert_eq!(
                http_handle.connector.bearer_token.as_deref(),
                Some("prepare-token")
            );
            assert_eq!(
                http_handle
                    .connector
                    .headers
                    .get("Authorization")
                    .map(String::as_str),
                Some("Bearer prepare-token")
            );
            let tools = http_handle.tools.expect("tool hints");
            assert_eq!(tools.enabled, vec!["delta".to_string()]);
        }
        other => panic!("expected http handle, got {other:?}"),
    }

    let after = fs::read_to_string(manager.config_path()).expect("read config after");
    assert_eq!(before, after);

    env::remove_var(env_var);
}
