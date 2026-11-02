use super::super::test_support::{prelude::*, *};
use super::super::*;

#[test]
fn add_stdio_server_injects_env_and_persists() {
    let (dir, manager) = temp_config_manager();
    let env_key = "MCP_STDIO_TEST_KEY";
    env::remove_var(env_key);

    let mut env_map = BTreeMap::new();
    env_map.insert(env_key.to_string(), "secret".to_string());

    let added = manager
        .add_server(AddServerRequest {
            name: "local".into(),
            definition: stdio_definition("my-mcp"),
            overwrite: false,
            env: env_map,
            bearer_token: None,
        })
        .expect("add server");

    match added.definition.transport {
        McpTransport::Stdio(def) => {
            assert_eq!(def.command, "my-mcp");
            assert_eq!(def.env.get(env_key), Some(&"secret".to_string()));
        }
        _ => panic!("expected stdio transport"),
    }

    let listed = manager.list_servers().expect("list servers");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "local");

    let fetched = manager.get_server("local").expect("get server");
    match fetched.definition.transport {
        McpTransport::Stdio(def) => {
            assert_eq!(def.env.get(env_key), Some(&"secret".to_string()))
        }
        _ => panic!("expected stdio transport"),
    }

    let config_path = dir.path().join(DEFAULT_CONFIG_FILE);
    let serialized = fs::read_to_string(config_path).expect("read config");
    let value: TomlValue = serialized.parse().expect("parse toml");
    let table = value.as_table().expect("table root");
    let servers_table = table.get("mcp_servers").expect("mcp_servers");
    let decoded: BTreeMap<String, McpServerDefinition> = servers_table
        .clone()
        .try_into()
        .expect("decode mcp_servers");
    let stored = decoded.get("local").expect("stored server");
    match &stored.transport {
        McpTransport::Stdio(def) => {
            assert_eq!(def.env.get(env_key), Some(&"secret".to_string()))
        }
        _ => panic!("expected stdio transport"),
    }

    assert_eq!(env::var(env_key).unwrap(), "secret");
    env::remove_var(env_key);
}

#[test]
fn add_streamable_http_sets_token_and_allows_login_logout() {
    let (_dir, manager) = temp_config_manager();
    let env_var = "MCP_HTTP_TOKEN_E5";
    env::remove_var(env_var);

    let mut definition = streamable_definition("https://example.test/mcp", env_var);
    if let McpTransport::StreamableHttp(ref mut http) = definition.transport {
        http.headers.insert("X-Test".into(), "true".into());
    }

    let _added = manager
        .add_server(AddServerRequest {
            name: "remote".into(),
            definition,
            overwrite: false,
            env: BTreeMap::new(),
            bearer_token: Some("token-a".into()),
        })
        .expect("add server");

    assert_eq!(env::var(env_var).unwrap(), "token-a");

    let logout = manager.logout("remote").expect("logout");
    assert_eq!(logout.env_var.as_deref(), Some(env_var));
    assert!(logout.cleared);
    assert!(env::var(env_var).is_err());

    let login = manager.login("remote", "token-b").expect("login");
    assert_eq!(login.env_var.as_deref(), Some(env_var));
    assert_eq!(env::var(env_var).unwrap(), "token-b");

    env::remove_var(env_var);
}

#[test]
fn remove_server_prunes_config() {
    let (_dir, manager) = temp_config_manager();

    manager
        .add_server(AddServerRequest {
            name: "one".into(),
            definition: stdio_definition("one"),
            overwrite: false,
            env: BTreeMap::new(),
            bearer_token: None,
        })
        .expect("add first");

    manager
        .add_server(AddServerRequest {
            name: "two".into(),
            definition: stdio_definition("two"),
            overwrite: false,
            env: BTreeMap::new(),
            bearer_token: None,
        })
        .expect("add second");

    let removed = manager.remove_server("one").expect("remove");
    assert!(removed.is_some());

    let listed = manager.list_servers().expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "two");

    let config = fs::read_to_string(manager.config_path()).expect("read config");
    let value: TomlValue = config.parse().expect("parse config");
    let table = value.as_table().expect("table root");
    let servers_value = table.get("mcp_servers").cloned().expect("servers");
    let servers: BTreeMap<String, McpServerDefinition> =
        servers_value.try_into().expect("decode servers");
    assert!(!servers.contains_key("one"));
    assert!(servers.contains_key("two"));
}

#[test]
fn runtime_stdio_server_resolves_env_and_tools() {
    let (_dir, manager) = temp_config_manager();
    let mut definition = stdio_definition("my-mcp");
    definition.description = Some("local mcp".into());
    definition.tags = vec!["dev".into(), "local".into()];
    definition.tools = Some(McpToolConfig {
        enabled: vec!["tool-a".into()],
        disabled: vec!["tool-b".into()],
    });

    if let McpTransport::Stdio(ref mut stdio) = definition.transport {
        stdio.args = vec!["--flag".into()];
        stdio.env.insert("EXAMPLE".into(), "value".into());
        stdio.timeout_ms = Some(2500);
    }

    let mut injected = BTreeMap::new();
    injected.insert("MCP_STDIO_INJECT_E6".into(), "yes".into());

    manager
        .add_server(AddServerRequest {
            name: "local".into(),
            definition,
            overwrite: false,
            env: injected,
            bearer_token: None,
        })
        .expect("add server");

    let runtime = manager.runtime_server("local").expect("runtime server");
    assert_eq!(runtime.name, "local");
    assert_eq!(runtime.description.as_deref(), Some("local mcp"));
    assert_eq!(runtime.tags, vec!["dev".to_string(), "local".to_string()]);

    let tools = runtime.tools.as_ref().expect("tool hints");
    assert_eq!(tools.enabled, vec!["tool-a".to_string()]);
    assert_eq!(tools.disabled, vec!["tool-b".to_string()]);

    match &runtime.transport {
        McpRuntimeTransport::Stdio(def) => {
            assert_eq!(def.command, "my-mcp");
            assert_eq!(def.args, vec!["--flag".to_string()]);
            assert_eq!(def.timeout_ms, Some(2500));
            assert_eq!(def.env.get("EXAMPLE").map(String::as_str), Some("value"));
            assert_eq!(
                def.env.get("MCP_STDIO_INJECT_E6").map(String::as_str),
                Some("yes")
            );
        }
        other => panic!("expected stdio transport, got {other:?}"),
    }

    serde_json::to_string(&runtime).expect("serialize runtime");
    env::remove_var("MCP_STDIO_INJECT_E6");
}

#[test]
fn runtime_http_resolves_bearer_and_sets_header() {
    let (_dir, manager) = temp_config_manager();
    let env_var = "MCP_HTTP_TOKEN_E6";
    env::set_var(env_var, "token-123");

    let mut definition = streamable_definition("https://example.test/mcp", env_var);
    if let McpTransport::StreamableHttp(ref mut http) = definition.transport {
        http.headers.insert("X-Test".into(), "true".into());
        http.connect_timeout_ms = Some(1200);
        http.request_timeout_ms = Some(3400);
    }

    manager
        .add_server(AddServerRequest {
            name: "remote".into(),
            definition,
            overwrite: false,
            env: BTreeMap::new(),
            bearer_token: None,
        })
        .expect("add server");

    let runtime = manager.runtime_server("remote").expect("runtime server");
    match &runtime.transport {
        McpRuntimeTransport::StreamableHttp(def) => {
            assert_eq!(def.url, "https://example.test/mcp");
            assert_eq!(def.bearer_env_var.as_deref(), Some(env_var));
            assert_eq!(def.bearer_token.as_deref(), Some("token-123"));
            assert_eq!(def.headers.get("X-Test").map(String::as_str), Some("true"));
            assert_eq!(
                def.headers.get("Authorization").map(String::as_str),
                Some("Bearer token-123")
            );
            assert_eq!(def.connect_timeout_ms, Some(1200));
            assert_eq!(def.request_timeout_ms, Some(3400));
        }
        other => panic!("expected streamable_http transport, got {other:?}"),
    }

    let serialized = serde_json::to_value(&runtime).expect("serialize runtime");
    assert!(serialized.get("transport").is_some());

    env::remove_var(env_var);
}

#[test]
fn runtime_http_preserves_existing_auth_header() {
    let (_dir, manager) = temp_config_manager();
    let env_var = "MCP_HTTP_TOKEN_E6B";
    env::set_var(env_var, "token-override");

    let mut definition = streamable_definition("https://example.test/custom", env_var);
    if let McpTransport::StreamableHttp(ref mut http) = definition.transport {
        http.headers
            .insert("Authorization".into(), "Custom 123".into());
    }

    manager
        .add_server(AddServerRequest {
            name: "remote-custom".into(),
            definition,
            overwrite: false,
            env: BTreeMap::new(),
            bearer_token: None,
        })
        .expect("add server");

    let runtime = manager
        .runtime_server("remote-custom")
        .expect("runtime server");
    match &runtime.transport {
        McpRuntimeTransport::StreamableHttp(def) => {
            assert_eq!(def.bearer_token.as_deref(), Some("token-override"));
            assert_eq!(
                def.headers.get("Authorization").map(String::as_str),
                Some("Custom 123")
            );
        }
        other => panic!("expected streamable_http transport, got {other:?}"),
    }

    env::remove_var(env_var);
}

#[test]
fn runtime_stdio_launcher_merges_env_timeout_and_tools() {
    let base_dir = tempfile::tempdir().expect("tempdir");
    let code_home = base_dir.path().join("code_home");

    let defaults = StdioServerConfig {
        binary: PathBuf::from("codex"),
        code_home: Some(code_home.clone()),
        current_dir: Some(base_dir.path().to_path_buf()),
        env: vec![
            (OsString::from("BASE_ONLY"), OsString::from("base")),
            (OsString::from("OVERRIDE_ME"), OsString::from("base")),
        ],
        app_server_analytics_default_enabled: false,
        mirror_stdio: true,
        startup_timeout: Duration::from_secs(5),
    };

    let mut definition = StdioServerDefinition {
        command: "my-mcp".into(),
        args: vec!["--flag".into()],
        env: BTreeMap::new(),
        timeout_ms: Some(1500),
    };
    definition
        .env
        .insert("OVERRIDE_ME".into(), "runtime".into());
    definition
        .env
        .insert("RUNTIME_ONLY".into(), "runtime-env".into());

    let runtime = McpRuntimeServer {
        name: "local".into(),
        transport: McpRuntimeTransport::Stdio(definition),
        description: Some("example".into()),
        tags: vec!["dev".into()],
        tools: Some(McpToolConfig {
            enabled: vec!["tool-1".into()],
            disabled: vec!["tool-2".into()],
        }),
    };

    let launcher = runtime.into_launcher(&defaults);
    assert_eq!(launcher.name, "local");
    assert_eq!(launcher.description.as_deref(), Some("example"));
    assert_eq!(launcher.tags, vec!["dev".to_string()]);

    let tools = launcher.tools.clone().expect("tool hints");
    assert_eq!(tools.enabled, vec!["tool-1".to_string()]);
    assert_eq!(tools.disabled, vec!["tool-2".to_string()]);

    match launcher.transport {
        McpServerLauncherTransport::Stdio(launch) => {
            assert_eq!(launch.command, PathBuf::from("my-mcp"));
            assert_eq!(launch.args, vec!["--flag".to_string()]);
            assert_eq!(launch.current_dir.as_ref(), defaults.current_dir.as_ref());
            assert_eq!(launch.timeout, Duration::from_millis(1500));
            assert!(launch.mirror_stdio);

            let env_map: HashMap<OsString, OsString> = launch.env.into_iter().collect();
            assert_eq!(
                env_map.get(&OsString::from("BASE_ONLY")),
                Some(&OsString::from("base"))
            );
            assert_eq!(
                env_map.get(&OsString::from("OVERRIDE_ME")),
                Some(&OsString::from("runtime"))
            );
            assert_eq!(
                env_map.get(&OsString::from("RUNTIME_ONLY")),
                Some(&OsString::from("runtime-env"))
            );
            assert_eq!(
                env_map.get(&OsString::from("CODEX_HOME")),
                Some(&code_home.as_os_str().to_os_string())
            );
        }
        other => panic!("expected stdio launcher, got {other:?}"),
    }
}

#[test]
fn streamable_http_connector_converts_timeouts_and_headers() {
    let env_var = "MCP_HTTP_TOKEN_E7";
    env::set_var(env_var, "token-launcher");

    let mut definition = StreamableHttpDefinition {
        url: "https://example.test/stream".into(),
        headers: BTreeMap::new(),
        bearer_env_var: Some(env_var.to_string()),
        connect_timeout_ms: Some(1200),
        request_timeout_ms: Some(3400),
    };
    definition.headers.insert("X-Test".into(), "true".into());

    let runtime = McpRuntimeServer::from_definition(
        "remote",
        McpServerDefinition {
            transport: McpTransport::StreamableHttp(definition),
            description: None,
            tags: vec!["http".into()],
            tools: Some(McpToolConfig {
                enabled: vec!["tool-a".into()],
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
    match launcher.transport {
        McpServerLauncherTransport::StreamableHttp(connector) => {
            assert_eq!(connector.url, "https://example.test/stream");
            assert_eq!(
                connector.headers.get("X-Test").map(String::as_str),
                Some("true")
            );
            assert_eq!(
                connector.headers.get("Authorization").map(String::as_str),
                Some("Bearer token-launcher")
            );
            assert_eq!(connector.connect_timeout, Some(Duration::from_millis(1200)));
            assert_eq!(connector.request_timeout, Some(Duration::from_millis(3400)));
            assert_eq!(connector.bearer_env_var.as_deref(), Some(env_var));
            assert_eq!(connector.bearer_token.as_deref(), Some("token-launcher"));

            let tools = launcher.tools.as_ref().expect("tool hints present");
            assert_eq!(tools.enabled, vec!["tool-a".to_string()]);
            assert!(tools.disabled.is_empty());
        }
        other => panic!("expected http connector, got {other:?}"),
    }

    env::remove_var(env_var);
}
