#![cfg(unix)]
use codex::{CodexClient, ExecProcessCustody, ExecStreamRequest, ResumeRequest};
use futures_util::StreamExt;
use std::{
    collections::BTreeMap,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

fn fixture(dir: &Path) -> PathBuf {
    let path = dir.join("fake-custody-codex");
    std::fs::write(
        &path,
        "#!/bin/sh\nprintf '%s' \"$*\" > \"$READY\"\nexec /bin/sleep 30\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}
async fn admitted(path: &Path) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if std::fs::metadata(path)
                .map(|m| m.len() != 0)
                .unwrap_or(false)
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("real fixture child admission");
}
fn request(prompt: String) -> ExecStreamRequest {
    ExecStreamRequest {
        prompt,
        ephemeral: false,
        ignore_rules: false,
        ignore_user_config: false,
        idle_timeout: None,
        output_last_message: None,
        output_schema: None,
        json_event_log: None,
    }
}

#[tokio::test]
async fn test_startup_drop_during_real_blocked_stdin_has_checked_settlement() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("admitted");
    let env = BTreeMap::from([("READY".to_string(), marker.display().to_string())]);
    let client = CodexClient::builder()
        .binary(fixture(dir.path()))
        .working_dir(dir.path())
        .quiet(true)
        .mirror_stdout(false)
        .build();
    let custody = ExecProcessCustody::new();
    let mut startup = Box::pin(client.stream_exec_with_env_overrides_custody(
        request("x".repeat(4 * 1024 * 1024)),
        &env,
        &custody,
    ));
    tokio::select! {
        result = &mut startup => panic!("stdin fixture must block startup: {}", result.is_ok()),
        _ = admitted(&marker) => {}
    }
    drop(startup);
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_capability_probe_drop_retains_its_actual_child_owner() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("probe");
    let env = BTreeMap::from([("READY".to_string(), marker.display().to_string())]);
    let client = CodexClient::builder()
        .binary(fixture(dir.path()))
        .working_dir(dir.path())
        .build();
    let custody = ExecProcessCustody::new();
    let mut probe = Box::pin(client.probe_capabilities_with_env_overrides_custody(&env, &custody));
    tokio::select! {
        _ = &mut probe => panic!("version fixture must remain running"),
        _ = admitted(&marker) => {}
    }
    assert!(std::fs::read_to_string(&marker)
        .unwrap()
        .contains("--version"));
    drop(probe);
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_resume_completion_drop_keeps_owned_wait_and_stdio_cleanup_alive() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("resume");
    let env = BTreeMap::from([("READY".to_string(), marker.display().to_string())]);
    let client = CodexClient::builder()
        .binary(fixture(dir.path()))
        .working_dir(dir.path())
        .quiet(true)
        .mirror_stdout(false)
        .build();
    let custody = ExecProcessCustody::new();
    let control = client
        .stream_resume_with_env_overrides_custody(ResumeRequest::with_id("session"), &env, &custody)
        .await
        .unwrap();
    admitted(&marker).await;
    assert!(std::fs::read_to_string(&marker)
        .unwrap()
        .contains("resume session"));
    drop(control.completion);
    drop(control.events);
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_sealed_delayed_startup_refuses_before_spawn() {
    let client = CodexClient::builder()
        .binary("nonexistent-must-not-spawn")
        .build();
    let custody = ExecProcessCustody::new();
    let env = BTreeMap::new();
    let startup =
        client.stream_exec_with_env_overrides_custody(request("hello".into()), &env, &custody);
    custody.settle().await.unwrap();
    assert!(startup.await.is_err());
}

#[tokio::test]
async fn test_closed_before_any_spawn_requires_explicit_admission_closeout() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("must-not-run");
    let env = BTreeMap::from([("READY".to_string(), marker.display().to_string())]);
    let client = CodexClient::builder()
        .binary(fixture(dir.path()))
        .working_dir(dir.path())
        .build();
    let custody = ExecProcessCustody::new();
    assert!(custody.observed_process_terminations().is_empty());
    assert!(!custody.observed_closed_before_any_spawn());
    custody.request_termination();
    assert!(client
        .stream_exec_with_env_overrides_custody(request("hello".into()), &env, &custody)
        .await
        .is_err());
    custody.settle().await.unwrap();
    assert!(custody.observed_closed_before_any_spawn());
    assert!(custody.observed_process_terminations().is_empty());
    let _ = client
        .probe_capabilities_with_env_overrides_custody(&env, &custody)
        .await;
    assert!(custody.observed_closed_before_any_spawn());
    assert!(!marker.exists());

    let failed = ExecProcessCustody::new();
    let missing = CodexClient::builder()
        .binary(dir.path().join("missing-binary"))
        .build();
    assert!(missing
        .stream_exec_with_env_overrides_custody(request("hello".into()), &env, &failed)
        .await
        .is_err());
    failed.settle().await.unwrap();
    assert!(!failed.observed_closed_before_any_spawn());
}

#[tokio::test]
async fn test_retained_termination_closes_events_without_completion_poll() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("exec");
    let env = BTreeMap::from([("READY".to_string(), marker.display().to_string())]);
    let client = CodexClient::builder()
        .binary(fixture(dir.path()))
        .working_dir(dir.path())
        .quiet(true)
        .mirror_stdout(false)
        .build();
    let custody = ExecProcessCustody::new();
    let mut control = client
        .stream_exec_with_env_overrides_custody(request("hello".into()), &env, &custody)
        .await
        .unwrap();
    admitted(&marker).await;
    control.termination.request_termination();
    assert!(
        tokio::time::timeout(Duration::from_secs(3), control.events.next())
            .await
            .unwrap()
            .is_none()
    );
    custody.settle().await.unwrap();
    assert!(control.completion.await.is_err());
}

#[tokio::test]
async fn test_each_selected_probe_branch_retains_custody_during_drop() {
    for selected in [
        "--version",
        "features list --json",
        "features list",
        "--help",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("probe-branches");
        std::fs::write(&path, "#!/bin/sh\nif [ \"$*\" = \"$BLOCK_PROBE\" ]; then printf '%s' \"$*\" > \"$READY\"; exec /bin/sleep 30; fi\nexit 0\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let marker = dir.path().join("admitted");
        let env = BTreeMap::from([
            ("READY".to_string(), marker.display().to_string()),
            ("BLOCK_PROBE".to_string(), selected.to_string()),
        ]);
        let client = CodexClient::builder()
            .binary(path)
            .working_dir(dir.path())
            .build();
        let custody = ExecProcessCustody::new();
        let mut probe =
            Box::pin(client.probe_capabilities_with_env_overrides_custody(&env, &custody));
        tokio::select! {
            _ = &mut probe => panic!("selected probe branch was not held: {selected}"),
            _ = admitted(&marker) => {}
        }
        assert_eq!(std::fs::read_to_string(marker).unwrap(), selected);
        drop(probe);
        tokio::time::timeout(Duration::from_secs(3), custody.settle())
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn test_observed_child_wait_retains_actual_signal_after_startup_drop() {
    use std::os::unix::process::ExitStatusExt;
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("actual-child-pid");
    let binary = dir.path().join("physical-wait-codex");
    std::fs::write(
        &binary,
        r#"#!/bin/sh
printf '%s' "$$" > "$READY"
exec /bin/sleep 30
"#,
    )
    .unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let env = BTreeMap::from([("READY".to_string(), marker.display().to_string())]);
    let client = CodexClient::builder()
        .binary(binary)
        .working_dir(dir.path())
        .quiet(true)
        .mirror_stdout(false)
        .build();
    let custody = ExecProcessCustody::new();
    assert!(
        custody.observed_process_terminations().is_empty(),
        "observation itself must not seal admission or spawn"
    );
    let mut startup = Box::pin(client.stream_exec_with_env_overrides_custody(
        request("x".repeat(4 * 1024 * 1024)),
        &env,
        &custody,
    ));
    tokio::select! {
        result = &mut startup => panic!("stdin fixture must block startup: {}", result.is_ok()),
        _ = admitted(&marker) => {},
    }
    let actual_pid = std::fs::read_to_string(&marker)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    drop(startup);
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .unwrap()
        .unwrap();
    let observations = custody.observed_process_terminations();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].process_id, Some(actual_pid));
    assert_eq!(
        observations[0].status.unwrap().signal(),
        Some(9),
        "actual checked start_kill wait, not a requested INT/TERM or synthetic completion status"
    );
    assert!(observations[0].recovery_status.is_none());
    assert!(!custody.observed_closed_before_any_spawn());
}

#[tokio::test]
async fn test_mcp_public_startup_drop_records_actual_wait_signal() {
    use codex::mcp::{ClientInfo, CodexAppServer, McpProcessCustody, StdioServerConfig};
    use std::{ffi::OsString, os::unix::process::ExitStatusExt};
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("actual-mcp-pid");
    let binary = dir.path().join("physical-mcp-wait");
    std::fs::write(
        &binary,
        r#"#!/bin/sh
printf '%s' "$$" > "$READY"
exec /bin/sleep 30
"#,
    )
    .unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let config = StdioServerConfig {
        binary,
        code_home: Some(dir.path().to_path_buf()),
        current_dir: Some(dir.path().to_path_buf()),
        env: vec![(OsString::from("READY"), marker.as_os_str().to_owned())],
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(10),
    };
    let custody = McpProcessCustody::new();
    assert!(custody.observed_process_terminations().is_empty());
    let mut startup = Box::pin(CodexAppServer::with_capabilities_and_custody(
        config,
        ClientInfo {
            name: "owned-physical-wait-fixture".into(),
            version: "1".into(),
        },
        serde_json::json!({}),
        custody.clone(),
    ));
    tokio::select! {
        result = &mut startup => panic!("synthetic app-server must wait for initialize: {}", result.is_ok()),
        _ = admitted(&marker) => {},
    }
    let actual_pid = std::fs::read_to_string(&marker)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    drop(startup);
    custody.request_termination();
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .unwrap()
        .unwrap();
    let observations = custody.observed_process_terminations();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].process_id, Some(actual_pid));
    assert_eq!(
        observations[0].status.unwrap().signal(),
        Some(9),
        "actual MCP child wait is independent from protocol synthetic success"
    );
    assert!(observations[0].recovery_status.is_none());
    assert!(!custody.observed_closed_before_any_spawn());
}

#[tokio::test]
async fn test_universal_process_settlement_preserves_cancelled_and_actual_wait() {
    use agent_api::{
        backends::codex::{CodexBackend, CodexBackendConfig},
        AgentWrapperError, AgentWrapperGateway, AgentWrapperKind, AgentWrapperRunRequest,
    };
    use std::{os::unix::process::ExitStatusExt, sync::Arc};
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("universal-actual-pid");
    let binary = dir.path().join("physical-universal-wait");
    std::fs::write(
        &binary,
        r#"#!/bin/sh
printf '%s' "$$" > "$READY"
exec /bin/sleep 30
"#,
    )
    .unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let backend = Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: Some(binary),
        codex_home: Some(dir.path().to_path_buf()),
        default_working_dir: Some(dir.path().to_path_buf()),
        env: BTreeMap::from([("READY".into(), marker.display().to_string())]),
        ..Default::default()
    }));
    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();
    let kind = AgentWrapperKind::new("codex").unwrap();
    let control = gateway
        .run_control(
            &kind,
            AgentWrapperRunRequest {
                prompt: "owned synthetic process wait fixture".into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let mut process_observation = control
        .cancel
        .process_settlement()
        .expect("real Codex custody must support actual wait observation");
    // Merely polling the observation must not seal or interrupt native startup.
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut process_observation)
            .await
            .is_err()
    );
    admitted(&marker).await;
    let actual_pid = std::fs::read_to_string(&marker)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let unit_ack = control.cancel.settlement().unwrap();
    control.cancel.cancel();
    // Checked native custody is independent of consumer stream finality. Keep
    // both observations and drain the universal stream before gated completion.
    tokio::time::timeout(Duration::from_secs(3), unit_ack)
        .await
        .unwrap()
        .unwrap();
    let actual = tokio::time::timeout(Duration::from_secs(3), process_observation)
        .await
        .unwrap();
    let mut events = control.handle.events;
    tokio::time::timeout(Duration::from_secs(3), async {
        while events.next().await.is_some() {}
    })
    .await
    .unwrap();
    let raw = tokio::time::timeout(Duration::from_secs(3), control.handle.completion)
        .await
        .unwrap();
    assert!(
        matches!(raw, Err(AgentWrapperError::Backend { message }) if message == "cancelled"),
        "physical observation cannot replace normative raw cancellation"
    );
    assert!(actual.acknowledged);
    assert_eq!(actual.closed_before_any_spawn, Some(false));
    let observations = actual
        .observations
        .expect("supported actual wait snapshot absent");
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].process_id, Some(actual_pid));
    assert_eq!(observations[0].status.unwrap().signal(), Some(9));
    assert!(observations[0].recovery_status.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn test_universal_cancel_before_startup_has_positive_admission_closeout() {
    use agent_api::{
        backends::codex::{CodexBackend, CodexBackendConfig},
        AgentWrapperError, AgentWrapperGateway, AgentWrapperKind, AgentWrapperRunRequest,
    };
    use std::sync::Arc;
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("must-not-run");
    let backend = Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: Some(fixture(dir.path())),
        codex_home: Some(dir.path().to_path_buf()),
        default_working_dir: Some(dir.path().to_path_buf()),
        env: BTreeMap::from([("READY".into(), marker.display().to_string())]),
        ..Default::default()
    }));
    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).unwrap();
    let control = gateway
        .run_control(
            &AgentWrapperKind::new("codex").unwrap(),
            AgentWrapperRunRequest {
                prompt: "cancel before startup is polled".into(),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // This current-thread executor has not polled the spawned startup driver.
    let process = control.cancel.process_settlement().unwrap();
    let unit_ack = control.cancel.settlement().unwrap();
    control.cancel.cancel();
    tokio::time::timeout(Duration::from_secs(3), unit_ack)
        .await
        .unwrap()
        .unwrap();
    let observed = tokio::time::timeout(Duration::from_secs(3), process)
        .await
        .unwrap();
    assert!(observed.acknowledged);
    assert_eq!(observed.closed_before_any_spawn, Some(true));
    assert_eq!(observed.observations, Some(Vec::new()));
    let mut events = control.handle.events;
    tokio::time::timeout(Duration::from_secs(3), async {
        while events.next().await.is_some() {}
    })
    .await
    .unwrap();
    assert!(
        matches!(control.handle.completion.await, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
    assert!(!marker.exists());
}

#[tokio::test]
async fn test_closed_mcp_admission_refuses_actual_startup() {
    use codex::mcp::{ClientInfo, CodexAppServer, McpProcessCustody, StdioServerConfig};
    let dir = tempfile::tempdir().unwrap();
    let custody = McpProcessCustody::new();
    assert!(!custody.observed_closed_before_any_spawn());
    custody.request_termination();
    let config = StdioServerConfig {
        binary: dir.path().join("must-not-spawn"),
        code_home: Some(dir.path().to_path_buf()),
        current_dir: Some(dir.path().to_path_buf()),
        env: vec![],
        app_server_analytics_default_enabled: false,
        mirror_stdio: false,
        startup_timeout: Duration::from_secs(1),
    };
    assert!(CodexAppServer::with_capabilities_and_custody(
        config,
        ClientInfo {
            name: "closed-admission-fixture".into(),
            version: "1".into()
        },
        serde_json::json!({}),
        custody.clone()
    )
    .await
    .is_err());
    custody.settle().await.unwrap();
    assert!(custody.observed_closed_before_any_spawn());
    assert!(custody.observed_process_terminations().is_empty());
}
