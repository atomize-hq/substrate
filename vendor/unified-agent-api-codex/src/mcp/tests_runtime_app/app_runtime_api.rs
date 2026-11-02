use super::super::test_support::{prelude::*, *};
use super::super::*;

#[test]
fn app_runtime_api_lists_and_merges_without_writes() {
    let (dir, manager) = temp_config_manager();

    let alpha_home = dir.path().join("app-home-a");
    let alpha_cwd = dir.path().join("app-cwd-a");
    let mut alpha_env = BTreeMap::new();
    alpha_env.insert("APP_RUNTIME_ENV".into(), "alpha".into());
    alpha_env.insert("OVERRIDE_ME".into(), "runtime".into());

    manager
        .add_app_runtime(AddAppRuntimeRequest {
            name: "alpha".into(),
            definition: AppRuntimeDefinition {
                description: Some("local app".into()),
                tags: vec!["local".into()],
                env: alpha_env,
                code_home: Some(alpha_home.clone()),
                current_dir: Some(alpha_cwd.clone()),
                mirror_stdio: Some(true),
                startup_timeout_ms: Some(4200),
                binary: Some(PathBuf::from("/bin/app-alpha")),
                metadata: serde_json::json!({"thread": "t-alpha"}),
            },
            overwrite: false,
        })
        .expect("add alpha app runtime");

    let mut beta_env = BTreeMap::new();
    beta_env.insert("APP_RUNTIME_ENV".into(), "beta".into());

    manager
        .add_app_runtime(AddAppRuntimeRequest {
            name: "beta".into(),
            definition: AppRuntimeDefinition {
                description: None,
                tags: vec!["default".into()],
                env: beta_env,
                code_home: None,
                current_dir: None,
                mirror_stdio: None,
                startup_timeout_ms: None,
                binary: None,
                metadata: serde_json::json!({"resume": true}),
            },
            overwrite: false,
        })
        .expect("add beta app runtime");

    let before = fs::read_to_string(manager.config_path()).expect("read config before");

    let default_home = dir.path().join("default-home");
    let default_cwd = dir.path().join("default-cwd");
    let defaults = StdioServerConfig {
        binary: PathBuf::from("codex"),
        code_home: Some(default_home.clone()),
        current_dir: Some(default_cwd.clone()),
        env: vec![
            (OsString::from("DEFAULT_ONLY"), OsString::from("base")),
            (OsString::from("OVERRIDE_ME"), OsString::from("base")),
        ],
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(3),
    };

    let api = AppRuntimeApi::from_config(&manager, &defaults).expect("app runtime api");

    let available = api.available();
    assert_eq!(available.len(), 2);

    let alpha_summary = available
        .iter()
        .find(|entry| entry.name == "alpha")
        .expect("alpha summary");
    assert_eq!(alpha_summary.description.as_deref(), Some("local app"));
    assert_eq!(alpha_summary.tags, vec!["local".to_string()]);
    assert_eq!(
        alpha_summary.metadata,
        serde_json::json!({"thread": "t-alpha"})
    );

    let alpha = api.prepare("alpha").expect("prepare alpha");
    assert_eq!(alpha.name, "alpha");
    assert_eq!(alpha.metadata, serde_json::json!({"thread": "t-alpha"}));
    assert_eq!(alpha.config.binary, PathBuf::from("/bin/app-alpha"));
    assert_eq!(
        alpha.config.code_home.as_deref(),
        Some(alpha_home.as_path())
    );
    assert_eq!(
        alpha.config.current_dir.as_deref(),
        Some(alpha_cwd.as_path())
    );
    assert!(alpha.config.mirror_stdio);
    assert_eq!(alpha.config.startup_timeout, Duration::from_millis(4200));

    let alpha_env: HashMap<OsString, OsString> = alpha.config.env.into_iter().collect();
    assert_eq!(
        alpha_env.get(&OsString::from("CODEX_HOME")),
        Some(&alpha_home.as_os_str().to_os_string())
    );
    assert_eq!(
        alpha_env.get(&OsString::from("DEFAULT_ONLY")),
        Some(&OsString::from("base"))
    );
    assert_eq!(
        alpha_env.get(&OsString::from("OVERRIDE_ME")),
        Some(&OsString::from("runtime"))
    );
    assert_eq!(
        alpha_env.get(&OsString::from("APP_RUNTIME_ENV")),
        Some(&OsString::from("alpha"))
    );

    let beta = api.stdio_config("beta").expect("beta config");
    assert_eq!(beta.binary, PathBuf::from("codex"));
    assert_eq!(beta.code_home.as_deref(), Some(default_home.as_path()));
    assert_eq!(beta.current_dir.as_deref(), Some(default_cwd.as_path()));
    assert!(!beta.mirror_stdio);
    assert_eq!(beta.startup_timeout, Duration::from_secs(3));

    let beta_env: HashMap<OsString, OsString> = beta.env.into_iter().collect();
    assert_eq!(
        beta_env.get(&OsString::from("CODEX_HOME")),
        Some(&default_home.as_os_str().to_os_string())
    );
    assert_eq!(
        beta_env.get(&OsString::from("DEFAULT_ONLY")),
        Some(&OsString::from("base"))
    );
    assert_eq!(
        beta_env.get(&OsString::from("OVERRIDE_ME")),
        Some(&OsString::from("base"))
    );
    assert_eq!(
        beta_env.get(&OsString::from("APP_RUNTIME_ENV")),
        Some(&OsString::from("beta"))
    );

    let beta_summary = available
        .iter()
        .find(|entry| entry.name == "beta")
        .expect("beta summary");
    assert_eq!(beta_summary.metadata, serde_json::json!({"resume": true}));

    let after = fs::read_to_string(manager.config_path()).expect("read config after");
    assert_eq!(before, after);
}

#[tokio::test]
async fn app_runtime_lifecycle_starts_and_stops_without_mutation() {
    let (config_dir, manager) = temp_config_manager();
    let (_server_dir, server_path) = write_fake_app_server();
    let code_home = config_dir.path().join("app-lifecycle-home");

    let mut env_map = BTreeMap::new();
    env_map.insert("APP_RUNTIME_LIFECYCLE".into(), "runtime-env".into());

    let metadata = serde_json::json!({"resume_thread": "thread-lifecycle"});
    manager
        .add_app_runtime(AddAppRuntimeRequest {
            name: "lifecycle".into(),
            definition: AppRuntimeDefinition {
                description: Some("app lifecycle".into()),
                tags: vec!["app".into()],
                env: env_map,
                code_home: None,
                current_dir: None,
                mirror_stdio: Some(true),
                startup_timeout_ms: Some(5000),
                binary: None,
                metadata: metadata.clone(),
            },
            overwrite: false,
        })
        .expect("add app runtime");

    let defaults = StdioServerConfig {
        binary: server_path.clone(),
        code_home: Some(code_home.clone()),
        current_dir: None,
        env: vec![(
            OsString::from("APP_RUNTIME_LIFECYCLE"),
            OsString::from("default"),
        )],
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(3),
    };

    let before = fs::read_to_string(manager.config_path()).expect("read config before");
    let api = AppRuntimeApi::from_config(&manager, &defaults).expect("build api");
    let client = test_client();

    let runtime = api
        .start("lifecycle", client.clone())
        .await
        .expect("start runtime");
    assert_eq!(runtime.name, "lifecycle");
    assert_eq!(runtime.metadata, metadata);

    let env_values: HashMap<OsString, OsString> = runtime.config.env.iter().cloned().collect();
    assert_eq!(
        env_values.get(&OsString::from("CODEX_HOME")),
        Some(&code_home.as_os_str().to_os_string())
    );
    assert_eq!(
        env_values.get(&OsString::from("APP_RUNTIME_LIFECYCLE")),
        Some(&OsString::from("runtime-env"))
    );

    let thread = runtime
        .server
        .thread_start(ThreadStartParams {
            thread_id: None,
            metadata: serde_json::json!({"from": "lifecycle"}),
        })
        .await
        .expect("thread start");
    let thread_response = time::timeout(Duration::from_secs(2), thread.response)
        .await
        .expect("thread response timeout")
        .expect("recv thread response")
        .expect("thread response ok");
    let thread_id = thread_response
        .get("thread_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(!thread_id.is_empty());

    runtime.stop().await.expect("shutdown runtime");

    let after = fs::read_to_string(manager.config_path()).expect("read config after");
    assert_eq!(before, after);

    let prepared = api.prepare("lifecycle").expect("prepare after stop");
    assert_eq!(prepared.metadata, metadata);
}

#[tokio::test]
async fn app_runtime_api_not_found_errors() {
    let api = AppRuntimeApi::new(Vec::new());
    match api.prepare("missing") {
        Err(AppRuntimeError::NotFound(name)) => assert_eq!(name, "missing"),
        other => panic!("unexpected result: {other:?}"),
    }

    let client = test_client();
    match api.start("missing", client).await {
        Err(AppRuntimeError::NotFound(name)) => assert_eq!(name, "missing"),
        other => panic!("unexpected start result: {other:?}"),
    }
}
