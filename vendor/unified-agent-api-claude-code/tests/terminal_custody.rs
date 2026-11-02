#![cfg(unix)]
use claude_code::{ClaudeClient, ClaudePrintRequest, ClaudeProcessCustody};
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

fn fixture(dir: &Path) -> PathBuf {
    let path = dir.join("fake-custody-claude");
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
fn client(dir: &Path, marker: &Path) -> ClaudeClient {
    ClaudeClient::builder()
        .binary(fixture(dir))
        .working_dir(dir)
        .env("READY", marker.display().to_string())
        .build()
}

#[tokio::test]
async fn test_argv_prompt_completion_drop_retains_checked_cleanup() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("argv");
    let client = client(dir.path(), &marker);
    let custody = ClaudeProcessCustody::new();
    let control = client
        .print_stream_json_control_with_custody(ClaudePrintRequest::new("argv-prompt"), &custody)
        .await
        .unwrap();
    admitted(&marker).await;
    assert!(std::fs::read_to_string(&marker)
        .unwrap()
        .contains("argv-prompt"));
    drop(control);
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_stdin_startup_drop_retains_child_before_first_write_await() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("stdin");
    let client = client(dir.path(), &marker);
    let custody = ClaudeProcessCustody::new();
    let mut startup = Box::pin(client.print_stream_json_control_with_custody(
        ClaudePrintRequest::new("argv-with-stdin").stdin_bytes(vec![b'x'; 4 * 1024 * 1024]),
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
async fn test_help_preflight_drop_retains_actual_owner() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("help");
    let client = client(dir.path(), &marker);
    let custody = ClaudeProcessCustody::new();
    let mut help = Box::pin(client.help_with_custody(&custody));
    tokio::select! {
        _ = &mut help => panic!("help fixture must remain running"),
        _ = admitted(&marker) => {}
    }
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "--help");
    drop(help);
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_cancelled_delayed_startup_refuses_before_spawn() {
    let client = ClaudeClient::builder()
        .binary("nonexistent-must-not-spawn")
        .build();
    let custody = ClaudeProcessCustody::new();
    let startup =
        client.print_stream_json_control_with_custody(ClaudePrintRequest::new("hello"), &custody);
    custody.request_termination();
    assert!(startup.await.is_err());
    custody.settle().await.unwrap();
}

#[tokio::test]
async fn test_idle_event_receiver_drop_terminates_child_while_completion_is_retained() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("idle-events-drop");
    let client = client(dir.path(), &marker);
    let custody = ClaudeProcessCustody::new();
    let control = client
        .print_stream_json_control_with_custody(ClaudePrintRequest::new("quiet-child"), &custody)
        .await
        .unwrap();
    admitted(&marker).await;
    // The fixture has emitted no stdout and remains alive in sleep. Retain both completion
    // and the termination handle; only dropping the public receiver requests termination.
    let termination = control.termination;
    let completion = control.completion;
    drop(control.events);
    let status = tokio::time::timeout(Duration::from_secs(3), completion)
        .await
        .expect("idle receiver drop must terminate without another event")
        .expect("public completion must carry the actual checked child status");
    assert!(
        !status.success(),
        "the long-lived fixture must have been terminated"
    );
    tokio::time::timeout(Duration::from_secs(3), custody.settle())
        .await
        .expect("every real stdio owner must acknowledge cleanup")
        .expect("checked settlement must succeed");
    drop(termination);
}
