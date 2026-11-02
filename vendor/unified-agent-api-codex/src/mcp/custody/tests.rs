use super::*;
use crate::mcp::test_support::{test_client, test_config, write_fake_app_server_fork_v1};
use crate::mcp::{CodexAppServer, McpError};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
use tokio::sync::oneshot;

fn held_gate(slot: &Mutex<Option<oneshot::Receiver<()>>>) -> oneshot::Sender<()> {
    let (release, gate) = oneshot::channel();
    *slot.lock().unwrap() = Some(gate);
    release
}
async fn observed(signal: &Notify) {
    tokio::time::timeout(Duration::from_secs(3), signal.notified())
        .await
        .expect("actual owner reached acknowledgment boundary");
}
async fn pending(custody: &McpProcessCustody) {
    assert!(
        tokio::time::timeout(Duration::from_millis(30), custody.settle())
            .await
            .is_err()
    );
}
async fn settled(custody: &McpProcessCustody) -> Result<(), McpSettlementError> {
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .expect("finite checked settlement")
}
async fn start(custody: &McpProcessCustody) -> (tempfile::TempDir, CodexAppServer) {
    let (dir, binary, _) = write_fake_app_server_fork_v1();
    let server = CodexAppServer::with_capabilities_and_custody(
        test_config(binary),
        test_client(),
        serde_json::json!({"experimentalApi":true}),
        custody.clone(),
    )
    .await
    .expect("start actual fixture child");
    (dir, server)
}

#[tokio::test]
async fn test_protocol_response_and_child_exit_do_not_ack_pending_stdout_owner() {
    let custody = McpProcessCustody::new();
    let release = held_gate(&custody.inner.pipe_gate);
    let (_dir, server) = start(&custody).await;
    let server = Arc::new(server);
    // Retaining another server Arc cannot block checked shutdown.
    let retained = server.clone();
    let shutdown = tokio::spawn(async move { server.shutdown().await });
    observed(&custody.inner.pipes_observed).await;
    pending(&custody).await;
    assert!(
        !shutdown.is_finished(),
        "shutdown must join the actual stdout owner"
    );
    release.send(()).unwrap();
    shutdown.await.unwrap().expect("shutdown checked");
    settled(&custody).await.unwrap();
    drop(retained);
}

#[tokio::test]
async fn test_drop_requests_cleanup_and_checked_wait_ack_remains_pending() {
    let custody = McpProcessCustody::new();
    let release = held_gate(&custody.inner.wait_gate);
    let (_dir, server) = start(&custody).await;
    drop(server);
    observed(&custody.inner.wait_observed).await;
    pending(&custody).await;
    release.send(()).unwrap();
    settled(&custody).await.unwrap();
}

#[tokio::test]
async fn test_cancel_is_not_ack_and_late_child_spawn_is_refused() {
    let custody = McpProcessCustody::new();
    let release = held_gate(&custody.inner.pipe_gate);
    let (_dir, server) = start(&custody).await;
    custody.request_termination();
    observed(&custody.inner.pipes_observed).await;
    pending(&custody).await;
    let mut command = Command::new("/bin/false");
    assert!(
        matches!(custody.spawn(&mut command), Err(error) if error.kind() == io::ErrorKind::Interrupted)
    );
    release.send(()).unwrap();
    settled(&custody).await.unwrap();
    drop(server);
}

fn silent_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("silent-app-server");
    let marker = dir.path().join("started");
    fs::write(&binary, "#!/usr/bin/env python3\nimport os,sys,time\nopen(os.environ['MCP_START_MARKER'],'w').close()\nfor line in sys.stdin:\n time.sleep(30)\n").unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
    (dir, binary, marker)
}

#[tokio::test]
async fn test_initialize_error_retains_wait_owner_until_ack() {
    let custody = McpProcessCustody::new();
    let release = held_gate(&custody.inner.wait_gate);
    let (_dir, binary, marker) = silent_fixture();
    let mut config = test_config(binary);
    config.startup_timeout = Duration::from_millis(30);
    config
        .env
        .push(("MCP_START_MARKER".into(), marker.into_os_string()));
    let observe = custody.clone();
    let startup = tokio::spawn(async move {
        CodexAppServer::with_capabilities_and_custody(
            config,
            test_client(),
            serde_json::json!({}),
            observe,
        )
        .await
    });
    observed(&custody.inner.wait_observed).await;
    assert!(
        !startup.is_finished(),
        "handshake failure cannot outrun checked wait"
    );
    pending(&custody).await;
    release.send(()).unwrap();
    assert!(matches!(
        startup.await.unwrap(),
        Err(McpError::Handshake(_))
    ));
    settled(&custody).await.unwrap();
}

#[tokio::test]
async fn test_cancel_or_drop_during_initialize_retains_actual_child_and_pipes() {
    let custody = McpProcessCustody::new();
    let release = held_gate(&custody.inner.pipe_gate);
    let (_dir, binary, marker) = silent_fixture();
    let mut config = test_config(binary);
    config.startup_timeout = Duration::from_secs(10);
    config
        .env
        .push(("MCP_START_MARKER".into(), marker.clone().into_os_string()));
    let observe = custody.clone();
    let startup = tokio::spawn(async move {
        CodexAppServer::with_capabilities_and_custody(
            config,
            test_client(),
            serde_json::json!({}),
            observe,
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(3), async {
        while !marker.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("actual fixture child entered");
    startup.abort();
    match startup.await {
        Err(error) => assert!(error.is_cancelled()),
        Ok(_) => panic!("startup was not cancelled"),
    }
    observed(&custody.inner.pipes_observed).await;
    pending(&custody).await;
    release.send(()).unwrap();
    settled(&custody).await.unwrap();
}

#[tokio::test]
async fn test_failed_first_wait_retains_child_through_recovery_and_all_pipe_acks() {
    let custody = McpProcessCustody::new();
    let first_wait_release = held_gate(&custody.inner.before_wait_gate);
    let cleanup_wait_release = held_gate(&custody.inner.cleanup_wait_gate);
    let stdout_release = held_gate(&custody.inner.pipe_gate);
    custody
        .inner
        .fail_before_wait
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let (_dir, server) = start(&custody).await;
    // The real child's initialize response arrived while the first actual wait
    // was withheld. The injected error therefore does not follow a real wait.
    observed(&custody.inner.before_wait_observed).await;
    first_wait_release.send(()).unwrap();
    observed(&custody.inner.cleanup_wait_observed).await;
    {
        let state = custody.inner.state.lock().unwrap();
        assert!(state.sealed && state.terminate);
        assert_eq!(state.active, 1, "actual child registration retained");
    }
    assert!(!custody
        .inner
        .cleanup_wait_succeeded
        .load(std::sync::atomic::Ordering::SeqCst));
    pending(&custody).await;
    let mut command = Command::new("/bin/false");
    assert!(
        matches!(custody.spawn(&mut command), Err(error) if error.kind() == io::ErrorKind::Interrupted)
    );
    cleanup_wait_release.send(()).unwrap();
    observed(&custody.inner.cleanup_wait_completed).await;
    assert!(
        custody
            .inner
            .cleanup_wait_succeeded
            .load(std::sync::atomic::Ordering::SeqCst),
        "recovery checked the actual Child.wait result"
    );
    observed(&custody.inner.pipes_observed).await;
    pending(&custody).await;
    assert_eq!(
        custody.inner.state.lock().unwrap().active,
        1,
        "stdout owner still holds registration after actual recovery wait"
    );
    stdout_release.send(()).unwrap();
    let error = settled(&custody).await.unwrap_err();
    assert_ne!(
        error.failures & WAIT_FAILED,
        0,
        "successful cleanup cannot erase first wait failure"
    );
    assert_eq!(custody.inner.state.lock().unwrap().active, 0);
    drop(server);
}

#[tokio::test]
async fn test_post_wait_and_stdio_failure_categories_survive_cleanup() {
    for failure in [WAIT_FAILED, STDERR_FAILED] {
        let custody = McpProcessCustody::new();
        custody
            .inner
            .inject_failures
            .store(failure, std::sync::atomic::Ordering::SeqCst);
        let (_dir, server) = start(&custody).await;
        let result = server.shutdown().await;
        assert!(
            matches!(result, Err(McpError::Transport(message)) if message.starts_with("MCP process settlement failed"))
        );
        let error = settled(&custody).await.unwrap_err();
        assert_ne!(error.failures & failure, 0);
    }
}

#[tokio::test]
async fn test_spawn_failure_has_no_unowned_child_and_still_seals_attempt() {
    let custody = McpProcessCustody::new();
    let dir = tempfile::tempdir().unwrap();
    let result = CodexAppServer::with_capabilities_and_custody(
        test_config(dir.path().join("absent")),
        test_client(),
        serde_json::json!({}),
        custody.clone(),
    )
    .await;
    assert!(matches!(result, Err(McpError::Spawn { .. })));
    settled(&custody).await.unwrap();
    assert!(custody.inner.state.lock().unwrap().sealed);
}

#[tokio::test]
async fn test_response_receiver_loss_cannot_supply_stdio_ack() {
    let custody = McpProcessCustody::new();
    let release = held_gate(&custody.inner.pipe_gate);
    let (_dir, server) = start(&custody).await;
    let call = server
        .turn_start_v2(crate::mcp::TurnStartParamsV2 {
            thread_id: "component-fixture-thread".into(),
            input: vec![crate::mcp::UserInputV2::text("component fixture")],
            approval_policy: None,
            cwd: None,
        })
        .await
        .unwrap();
    drop(call.response);
    drop(call.events);
    let shutdown = tokio::spawn(async move { server.shutdown().await });
    observed(&custody.inner.pipes_observed).await;
    pending(&custody).await;
    assert!(!shutdown.is_finished());
    release.send(()).unwrap();
    shutdown.await.unwrap().unwrap();
    settled(&custody).await.unwrap();
}

#[tokio::test]
async fn test_natural_exit_drains_buffered_initialize_response() {
    let custody = McpProcessCustody::new();
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("initialize-then-exit");
    fs::write(&binary, "#!/usr/bin/env python3\nimport json,sys\nm=json.loads(sys.stdin.readline())\nprint(json.dumps({'jsonrpc':'2.0','id':m['id'],'result':{'ready':True}}),flush=True)\n").unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
    let server = CodexAppServer::with_capabilities_and_custody(
        test_config(binary),
        test_client(),
        serde_json::json!({}),
        custody.clone(),
    )
    .await
    .expect("natural exit must preserve buffered protocol response");
    settled(&custody).await.unwrap();
    drop(server);
}
