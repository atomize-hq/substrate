use std::process::Stdio;
use std::time::Duration;

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn wait_for_child_exit_returns_status_when_deadline_has_elapsed() {
    use crate::{
        process_custody::{drive_child, StderrMode},
        ClaudeProcessCustody,
    };
    use tokio::io::AsyncReadExt;

    let custody = ClaudeProcessCustody::new();
    let mut command = tokio::process::Command::new("/bin/sh");
    command
        .args(["-c", "exit 0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let (mut child, owner) = custody
        .spawn(&mut command, std::path::Path::new("/bin/sh"))
        .expect("spawn child");
    let actual_exit = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(status) = child.try_wait().expect("check real child waitability") {
                break status;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("child actually exits before entering the elapsed-deadline owner");
    assert!(actual_exit.success());

    let (ready_tx, _ready_rx) = tokio::sync::oneshot::channel();
    // Arm the actual production deadline, let it elapse, then poll that same owner again.
    // This exercises the replacement wait path rather than an obsolete helper.
    let mut driver = Box::pin(drive_child(
        child,
        owner,
        None,
        false,
        Some(Duration::from_millis(1)),
        StderrMode::Capture { mirror: false },
        ready_tx,
        |mut stdout| async move {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).await?;
            Ok(bytes)
        },
    ));
    assert!(futures_util::poll!(driver.as_mut()).is_pending());
    tokio::time::sleep(Duration::from_millis(10)).await;
    let outcome = driver.await;
    assert!(
        !outcome.timed_out,
        "an already-exited child is not timed out"
    );
    let status = outcome
        .status
        .expect("production owner returns actual child status");
    assert!(status.success()); // Original elapsed-deadline status assertion retained.
    assert_eq!(status, actual_exit);
    if let Some(bytes) = outcome.stdout.expect("stdout acknowledged") {
        assert!(bytes.is_empty());
    }
    if let Some(bytes) = outcome.stderr.expect("stderr acknowledged") {
        assert!(bytes.is_empty());
    }
    assert!(!outcome.settlement_failed);
    custody
        .settle()
        .await
        .expect("checked child and stdio settlement");
}
