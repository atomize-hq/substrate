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
