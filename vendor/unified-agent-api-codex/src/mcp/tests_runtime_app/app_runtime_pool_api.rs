use super::super::test_support::{prelude::*, *};
use super::super::*;

#[tokio::test]
async fn app_runtime_pool_api_reuses_and_restarts_stdio() {
    let (config_dir, manager) = temp_config_manager();
    let (_server_dir, server_path) = write_fake_app_server();
    let code_home = config_dir.path().join("app-pool-home");

    let mut env_map = BTreeMap::new();
    env_map.insert("APP_POOL_ENV".into(), "runtime".into());

    let metadata = serde_json::json!({"resume_thread": "thread-pool"});
    manager
        .add_app_runtime(AddAppRuntimeRequest {
            name: "pooled".into(),
            definition: AppRuntimeDefinition {
                description: Some("pooled app".into()),
                tags: vec!["pool".into()],
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
        env: vec![
            (OsString::from("APP_POOL_ENV"), OsString::from("default")),
            (OsString::from("POOL_ONLY"), OsString::from("base")),
        ],
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(3),
    };

    let before = fs::read_to_string(manager.config_path()).expect("read config before");
    let api = AppRuntimePoolApi::from_config(&manager, &defaults).expect("build pool api");
    let client = test_client();

    let available = api.available();
    assert_eq!(available.len(), 1);
    let pooled_summary = &available[0];
    assert_eq!(pooled_summary.name, "pooled");
    assert_eq!(pooled_summary.metadata, metadata);

    let launcher = api.launcher("pooled").expect("pooled launcher");
    assert_eq!(launcher.description.as_deref(), Some("pooled app"));
    assert_eq!(launcher.metadata, metadata);

    let launcher_config = launcher.config.clone();
    assert_eq!(launcher_config.binary, server_path);
    assert_eq!(
        launcher_config.code_home.as_deref(),
        Some(code_home.as_path())
    );
    assert_eq!(launcher_config.startup_timeout, Duration::from_secs(5));

    let launcher_env: HashMap<OsString, OsString> = launcher_config.env.into_iter().collect();
    assert_eq!(
        launcher_env.get(&OsString::from("CODEX_HOME")),
        Some(&code_home.as_os_str().to_os_string())
    );
    assert_eq!(
        launcher_env.get(&OsString::from("POOL_ONLY")),
        Some(&OsString::from("base"))
    );
    assert_eq!(
        launcher_env.get(&OsString::from("APP_POOL_ENV")),
        Some(&OsString::from("runtime"))
    );

    let stdio_config = api
        .stdio_config("pooled")
        .expect("pooled stdio config without starting");
    assert_eq!(stdio_config.binary, server_path);
    assert_eq!(stdio_config.code_home.as_deref(), Some(code_home.as_path()));
    let stdio_env: HashMap<OsString, OsString> = stdio_config.env.into_iter().collect();
    assert_eq!(
        stdio_env.get(&OsString::from("POOL_ONLY")),
        Some(&OsString::from("base"))
    );
    assert_eq!(
        stdio_env.get(&OsString::from("CODEX_HOME")),
        Some(&code_home.as_os_str().to_os_string())
    );
    assert_eq!(
        stdio_env.get(&OsString::from("APP_POOL_ENV")),
        Some(&OsString::from("runtime"))
    );

    assert!(api.running().await.is_empty());

    let runtime = api
        .start("pooled", client.clone())
        .await
        .expect("start pooled runtime");
    assert_eq!(runtime.name, "pooled");
    assert_eq!(runtime.metadata, metadata);

    let env_values: HashMap<OsString, OsString> = runtime.config.env.iter().cloned().collect();
    assert_eq!(
        env_values.get(&OsString::from("CODEX_HOME")),
        Some(&code_home.as_os_str().to_os_string())
    );
    assert_eq!(
        env_values.get(&OsString::from("POOL_ONLY")),
        Some(&OsString::from("base"))
    );
    assert_eq!(
        env_values.get(&OsString::from("APP_POOL_ENV")),
        Some(&OsString::from("runtime"))
    );

    let thread = runtime
        .server
        .thread_start(ThreadStartParams {
            thread_id: None,
            metadata: serde_json::json!({"from": "pool"}),
        })
        .await
        .expect("thread start");
    let response = time::timeout(Duration::from_secs(2), thread.response)
        .await
        .expect("thread response timeout")
        .expect("recv thread response")
        .expect("thread response ok");
    let thread_id = response
        .get("thread_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(!thread_id.is_empty());

    let running = api.running().await;
    let running_summary = running
        .iter()
        .find(|summary| summary.name == "pooled")
        .expect("running summary present");
    assert_eq!(running_summary.metadata, metadata);

    let reused = api
        .start("pooled", client.clone())
        .await
        .expect("reuse pooled runtime");
    assert!(Arc::ptr_eq(&runtime, &reused));

    api.stop("pooled").await.expect("stop pooled runtime");
    match api.stop("pooled").await {
        Err(AppRuntimeError::NotFound(name)) => assert_eq!(name, "pooled"),
        other => panic!("expected not found on second stop, got {other:?}"),
    }

    assert!(api.running().await.is_empty());

    let restarted = api
        .start("pooled", client)
        .await
        .expect("restart pooled runtime");
    assert!(!Arc::ptr_eq(&runtime, &restarted));
    assert_eq!(restarted.metadata, metadata);

    let prepared = api.prepare("pooled").expect("prepare after restart");
    assert_eq!(prepared.metadata, metadata);

    let after = fs::read_to_string(manager.config_path()).expect("read config after");
    assert_eq!(before, after);
}

#[tokio::test]
async fn app_runtime_pool_api_stop_all_shuts_down_runtimes() {
    let (config_dir, manager) = temp_config_manager();
    let (_server_dir, server_path) = write_fake_app_server();
    let code_home = config_dir.path().join("app-pool-stop-home");

    let alpha_metadata = serde_json::json!({"resume_thread": "alpha"});
    manager
        .add_app_runtime(AddAppRuntimeRequest {
            name: "alpha".into(),
            definition: AppRuntimeDefinition {
                description: Some("alpha runtime".into()),
                tags: vec!["pool".into()],
                env: BTreeMap::new(),
                code_home: None,
                current_dir: None,
                mirror_stdio: Some(false),
                startup_timeout_ms: Some(5000),
                binary: None,
                metadata: alpha_metadata.clone(),
            },
            overwrite: false,
        })
        .expect("add alpha runtime");

    let beta_metadata = serde_json::json!({"resume_thread": "beta"});
    manager
        .add_app_runtime(AddAppRuntimeRequest {
            name: "beta".into(),
            definition: AppRuntimeDefinition {
                description: Some("beta runtime".into()),
                tags: vec!["pool".into()],
                env: BTreeMap::new(),
                code_home: None,
                current_dir: None,
                mirror_stdio: Some(false),
                startup_timeout_ms: Some(5000),
                binary: None,
                metadata: beta_metadata.clone(),
            },
            overwrite: false,
        })
        .expect("add beta runtime");

    let defaults = StdioServerConfig {
        binary: server_path.clone(),
        code_home: Some(code_home.clone()),
        current_dir: None,
        env: Vec::new(),
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(3),
    };

    let before = fs::read_to_string(manager.config_path()).expect("read config before");
    let api = AppRuntimePoolApi::from_config(&manager, &defaults).expect("build pool api");
    let client = test_client();

    assert!(api.running().await.is_empty());

    let alpha = api
        .start("alpha", client.clone())
        .await
        .expect("start alpha runtime");
    let beta = api
        .start("beta", client.clone())
        .await
        .expect("start beta runtime");

    assert_eq!(alpha.metadata, alpha_metadata);
    assert_eq!(beta.metadata, beta_metadata);

    let mut running = api.running().await;
    running.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(running.len(), 2);
    assert_eq!(running[0].name, "alpha");
    assert_eq!(running[0].metadata, alpha_metadata);
    assert_eq!(running[1].name, "beta");
    assert_eq!(running[1].metadata, beta_metadata);

    let alpha_thread = alpha
        .server
        .thread_start(ThreadStartParams {
            thread_id: None,
            metadata: serde_json::json!({"from": "alpha"}),
        })
        .await
        .expect("alpha thread start");
    let _ = time::timeout(Duration::from_secs(2), alpha_thread.response)
        .await
        .expect("alpha thread response timeout")
        .expect("alpha response recv")
        .expect("alpha ok");

    api.stop_all().await.expect("stop all runtimes");
    assert!(api.running().await.is_empty());

    let restarted_alpha = api
        .start("alpha", client.clone())
        .await
        .expect("restart alpha");
    assert!(!Arc::ptr_eq(&alpha, &restarted_alpha));
    assert_eq!(restarted_alpha.metadata, alpha_metadata);

    let restarted_beta = api.start("beta", client).await.expect("restart beta");
    assert!(!Arc::ptr_eq(&beta, &restarted_beta));
    assert_eq!(restarted_beta.metadata, beta_metadata);

    let prepared_alpha = api.prepare("alpha").expect("prepare alpha");
    assert_eq!(prepared_alpha.metadata, alpha_metadata);
    let prepared_beta = api.prepare("beta").expect("prepare beta");
    assert_eq!(prepared_beta.metadata, beta_metadata);

    let after = fs::read_to_string(manager.config_path()).expect("read config after");
    assert_eq!(before, after);
}
