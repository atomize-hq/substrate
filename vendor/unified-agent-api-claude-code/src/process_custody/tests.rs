use super::*;
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn test_sealed_attempt_rejects_delayed_spawn() {
    let custody = ClaudeProcessCustody::new();
    custody.settle().await.unwrap();
    let mut command = Command::new("unused-binary-must-not-be-launched");
    assert!(custody.spawn(&mut command, Path::new("unused")).is_err());
}

#[tokio::test]
async fn test_cancelled_attempt_rejects_delayed_spawn() {
    let custody = ClaudeProcessCustody::new();
    custody.request_termination();
    let mut command = Command::new("unused-binary-must-not-be-launched");
    assert!(custody.spawn(&mut command, Path::new("unused")).is_err());
    custody.settle().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_real_child_wait_and_stdout_ack_both_gate_settlement() {
    let custody = ClaudeProcessCustody::new();
    let (wait_tx, wait_rx) = oneshot::channel();
    *custody.inner.wait_observer.lock().unwrap() = Some(wait_tx);
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf owned-output"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let (release_tx, release_rx) = oneshot::channel();
    let (read_tx, read_rx) = oneshot::channel();
    let (ready_tx, ready_rx) = oneshot::channel();
    let driver = tokio::spawn(drive_child(
        child,
        owner,
        None,
        true,
        None,
        StderrMode::Capture { mirror: false },
        ready_tx,
        move |mut stdout| async move {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).await?;
            read_tx.send(bytes.clone()).unwrap();
            release_rx.await.unwrap();
            Ok(bytes)
        },
    ));
    ready_rx.await.unwrap().unwrap();
    assert_eq!(read_rx.await.unwrap(), b"owned-output");
    wait_rx.await.unwrap(); // Sent only after actual Child::wait returned Ok.
    let settlement = custody.settle();
    tokio::pin!(settlement);
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut settlement)
            .await
            .is_err()
    );
    release_tx.send(()).unwrap();
    driver.await.unwrap().status.unwrap();
    settlement.await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_real_child_reader_failure_is_not_successful_settlement() {
    let custody = ClaudeProcessCustody::new();
    let mut sibling_command = Command::new("/bin/sh");
    sibling_command
        .args(["-c", "exec /bin/sleep 30"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (sibling, sibling_owner) = custody
        .spawn(&mut sibling_command, Path::new("/bin/sh"))
        .unwrap();
    let (sibling_ready_tx, sibling_ready_rx) = oneshot::channel();
    let sibling_driver = tokio::spawn(drive_child(
        sibling,
        sibling_owner,
        None,
        true,
        None,
        StderrMode::Capture { mirror: false },
        sibling_ready_tx,
        |mut stdout| async move {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).await?;
            Ok(bytes)
        },
    ));
    sibling_ready_rx.await.unwrap().unwrap();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf x; exec /bin/sleep 30"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let (ready_tx, ready_rx) = oneshot::channel();
    let driver = tokio::spawn(drive_child(
        child,
        owner,
        None,
        true,
        None,
        StderrMode::Capture { mirror: false },
        ready_tx,
        |mut stdout| async move {
            let mut byte = [0];
            stdout.read_exact(&mut byte).await?;
            Err::<(), _>(io::Error::other("injected reader failure"))
        },
    ));
    let _ = ready_rx.await;
    let result = driver.await.unwrap();
    assert!(
        result.status.is_ok(),
        "actual Child wait must still acknowledge cleanup"
    );
    assert!(result.stdout.is_err());
    assert!(result.settlement_failed);
    // No settle/seal call has run: the failed owner itself must close admission.
    assert!(custody.ensure_open().is_err());
    let mut delayed = Command::new("unused-binary-must-not-be-launched");
    assert!(custody.spawn(&mut delayed, Path::new("unused")).is_err());
    let sibling_result = tokio::time::timeout(Duration::from_secs(3), sibling_driver)
        .await
        .expect("owner failure must terminate other admitted children")
        .unwrap();
    assert!(!sibling_result.status.unwrap().success());
    assert!(
        !sibling_result.settlement_failed,
        "sibling wait and stdio must acknowledge"
    );
    let failure = custody.settle().await.unwrap_err();
    assert_ne!(failure.mask & STDOUT_FAILED, 0);
}

#[cfg(unix)]
#[tokio::test]
async fn test_real_child_reader_panic_is_checked_and_retained() {
    let custody = ClaudeProcessCustody::new();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf x; exec /bin/sleep 30"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let (ready_tx, _) = oneshot::channel();
    let result = drive_child(
        child,
        owner,
        None,
        true,
        None,
        StderrMode::Capture { mirror: false },
        ready_tx,
        |_| async {
            if std::hint::black_box(true) {
                panic!("injected reader join failure");
            }
            Ok(())
        },
    )
    .await;
    assert!(result.status.is_ok());
    assert_ne!(custody.settle().await.unwrap_err().mask & STDOUT_FAILED, 0);
}

#[tokio::test]
async fn test_unacknowledged_owner_never_reports_success() {
    let custody = ClaudeProcessCustody::new();
    {
        let mut state = custody.inner.state.lock().unwrap();
        state.active += 1;
    }
    drop(ChildOwner {
        custody: custody.clone(),
        acknowledged: false,
    });
    assert!(
        custody.ensure_open().is_err(),
        "abandonment must close admission before settle"
    );
    assert_ne!(custody.settle().await.unwrap_err().mask & OWNER_FAILED, 0);
}

#[cfg(unix)]
#[tokio::test]
async fn test_real_child_setup_failure_waits_for_cleanup_before_ready_error() {
    let custody = ClaudeProcessCustody::new();
    let (wait_tx, wait_rx) = oneshot::channel();
    *custody.inner.wait_observer.lock().unwrap() = Some(wait_tx);
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "exec /bin/sleep 30"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let (ready_tx, ready_rx) = oneshot::channel();
    let driver = tokio::spawn(drive_child(
        child,
        owner,
        None,
        true,
        None,
        StderrMode::Capture { mirror: false },
        ready_tx,
        |_| async { Ok(()) },
    ));
    assert!(ready_rx.await.unwrap().is_err());
    wait_rx.await.unwrap();
    assert!(driver.await.unwrap().status.is_ok());
    assert_ne!(custody.settle().await.unwrap_err().mask & STDOUT_FAILED, 0);
}

#[cfg(unix)]
#[tokio::test]
async fn test_wait_failure_and_second_stdio_failure_remain_visible_before_completion() {
    let custody = ClaudeProcessCustody::new();
    custody.inner.fail_wait.store(true, Ordering::SeqCst);
    custody.inner.fail_stderr.store(true, Ordering::SeqCst);
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf owned"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let (ready_tx, _) = oneshot::channel();
    let result = drive_child(
        child,
        owner,
        None,
        true,
        None,
        StderrMode::Capture { mirror: false },
        ready_tx,
        |mut stdout| async move {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).await?;
            Ok(bytes)
        },
    )
    .await;
    assert!(result.status.is_err());
    let failure = custody.settle().await.unwrap_err();
    assert_ne!(failure.mask & WAIT_FAILED, 0);
    assert_ne!(failure.mask & STDERR_FAILED, 0);
}

#[cfg(unix)]
#[tokio::test]
async fn test_cancellation_waits_for_real_child_wait_and_reader_disposal_ack() {
    let custody = ClaudeProcessCustody::new();
    let (wait_tx, wait_rx) = oneshot::channel();
    *custody.inner.wait_observer.lock().unwrap() = Some(wait_tx);
    let (release_tx, release_rx) = oneshot::channel();
    *custody.inner.stdout_ack_gate.lock().unwrap() = Some(release_rx);
    let (disposed_tx, disposed_rx) = oneshot::channel();
    *custody.inner.stdout_ack_observer.lock().unwrap() = Some(disposed_tx);
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "exec /bin/sleep 30"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let (ready_tx, ready_rx) = oneshot::channel();
    let driver = tokio::spawn(drive_child(
        child,
        owner,
        None,
        true,
        None,
        StderrMode::Capture { mirror: false },
        ready_tx,
        |mut stdout| async move {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).await?;
            Ok(bytes)
        },
    ));
    ready_rx.await.unwrap().unwrap();
    custody.request_termination();
    disposed_rx.await.unwrap(); // The read future has been disposed; its task has not acknowledged yet.
    wait_rx.await.unwrap(); // Actual Child::wait succeeded, independent of the held stdio acknowledgment.
    let mut settlement = Box::pin(custody.settle());
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut settlement)
            .await
            .is_err()
    );
    release_tx.send(()).unwrap();
    assert!(driver.await.unwrap().status.is_ok());
    settlement.await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_elapsed_deadline_preserves_actual_already_exited_status_and_stdio() {
    let custody = ClaudeProcessCustody::new();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf terminal-output; exit 7"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (mut child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let actual_exit = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("actual child exit before elapsed deadline");
    assert_eq!(actual_exit.code(), Some(7));
    let (ready_tx, _ready_rx) = oneshot::channel();
    let mut driver = Box::pin(drive_child(
        child,
        owner,
        None,
        true,
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
    let result = driver.await;
    assert!(
        !result.timed_out,
        "ready process status must win over elapsed deadline"
    );
    assert_eq!(result.status.unwrap(), actual_exit);
    // Disposal is acknowledged honestly; only an actual completed read claims captured bytes.
    if let Some(bytes) = result.stdout.unwrap() {
        assert_eq!(bytes, b"terminal-output");
    }
    if let Some(bytes) = result.stderr.unwrap() {
        assert!(bytes.is_empty());
    }
    assert!(result.stdin.is_ok());
    assert!(!result.settlement_failed);
    custody.settle().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_elapsed_deadline_still_terminates_a_real_live_child_with_checked_ack() {
    let custody = ClaudeProcessCustody::new();
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "exec /bin/sleep 30"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (mut child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    assert!(
        child.try_wait().unwrap().is_none(),
        "fixture is actually running at the deadline"
    );
    let (ready_tx, _ready_rx) = oneshot::channel();
    let result = drive_child(
        child,
        owner,
        None,
        true,
        Some(Duration::ZERO),
        StderrMode::Capture { mirror: false },
        ready_tx,
        |mut stdout| async move {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).await?;
            Ok(bytes)
        },
    )
    .await;
    assert!(result.timed_out);
    assert!(
        !result.status.unwrap().success(),
        "checked wait must observe actual terminated status"
    );
    assert!(
        !result.settlement_failed,
        "timeout does not replace checked wait/stdio acknowledgment"
    );
    custody.settle().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_exited_child_held_stdout_still_reaches_run_deadline_and_checked_disposal() {
    let custody = ClaudeProcessCustody::new();
    let (wait_tx, wait_rx) = oneshot::channel();
    *custody.inner.wait_observer.lock().unwrap() = Some(wait_tx);
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf held-output; exit 0"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel::<()>();
    let (ready_tx, ready_rx) = oneshot::channel();
    let driver = tokio::spawn(drive_child(
        child,
        owner,
        None,
        true,
        Some(Duration::from_millis(500)),
        StderrMode::Capture { mirror: false },
        ready_tx,
        move |mut stdout| async move {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).await?;
            entered_tx.send(bytes).unwrap();
            let _ = release_rx.await;
            Ok(())
        },
    ));
    ready_rx.await.unwrap().unwrap();
    assert_eq!(entered_rx.await.unwrap(), b"held-output");
    wait_rx.await.unwrap(); // Actual owned child wait completed while stdout owner is still held.
    let result = tokio::time::timeout(Duration::from_secs(3), driver)
        .await
        .expect("run deadline must dispose held stdout even after child exit")
        .unwrap();
    assert!(result.status.unwrap().success());
    assert!(
        !result.timed_out,
        "Claude preserves already-exited success while disposing held stdout"
    );
    assert!(
        result.stdout.unwrap().is_none(),
        "disposal must not claim full capture"
    );
    assert!(!result.settlement_failed);
    assert!(
        release_tx.send(()).is_err(),
        "the disposed reader future must no longer hold its gate"
    );
    custody.settle().await.unwrap();
}

struct CountingStderr {
    bytes: Arc<std::sync::atomic::AtomicUsize>,
}
impl tokio::io::AsyncWrite for CountingStderr {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        bytes: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        self.bytes.fetch_add(bytes.len(), Ordering::SeqCst);
        std::task::Poll::Ready(Ok(bytes.len()))
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn test_stderr_mirror_forwards_large_stream_without_retaining_capture() {
    let bytes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let writer = CountingStderr {
        bytes: bytes.clone(),
    };
    let total = 8 * 1024 * 1024;
    let output = read_stderr_to(
        tokio::io::repeat(b'm').take(total as u64),
        writer,
        StderrMode::Mirror,
    )
    .await
    .unwrap();
    assert_eq!(
        bytes.load(Ordering::SeqCst),
        total,
        "every mirrored byte must be forwarded"
    );
    assert!(
        output.is_empty(),
        "streaming mirror must not accumulate discarded stderr bytes"
    );
}

#[tokio::test]
async fn test_stderr_capture_preserves_exact_command_output_without_mirroring() {
    let bytes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let writer = CountingStderr {
        bytes: bytes.clone(),
    };
    let output = read_stderr_to(
        tokio::io::repeat(b'c').take(8193),
        writer,
        StderrMode::Capture { mirror: false },
    )
    .await
    .unwrap();
    assert_eq!(output, vec![b'c'; 8193]);
    assert_eq!(bytes.load(Ordering::SeqCst), 0);
}

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn test_elapsed_deadline_keeps_checked_readiness_distinct_from_terminal_ack() {
    let custody = ClaudeProcessCustody::new();
    let (ack_release_tx, ack_release_rx) = oneshot::channel();
    *custody.inner.stdout_ack_gate.lock().unwrap() = Some(ack_release_rx);
    let (ack_reached_tx, ack_reached_rx) = oneshot::channel();
    *custody.inner.stdout_ack_observer.lock().unwrap() = Some(ack_reached_tx);
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf setup-complete; exit 0"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let (mut child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
    let actual_exit = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(actual_exit.success());
    let (ready_tx, mut ready_rx) = oneshot::channel();
    let mut driver = Box::pin(drive_child(
        child,
        owner,
        None,
        true,
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
    ack_reached_rx.await.unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        tokio::select! {
            result = &mut driver => panic!("terminal result cannot precede held stdio ACK: {}", result.status.is_ok()),
            ready = &mut ready_rx => ready.expect("startup observation").expect("successful stdin setup must remain ready"),
        }
    }).await.expect("deadline must not erase checked startup readiness");
    assert!(tokio::time::timeout(Duration::from_millis(25), &mut driver)
        .await
        .is_err());
    ack_release_tx.send(()).unwrap();
    let outcome = driver.await;
    assert_eq!(outcome.status.unwrap(), actual_exit);
    assert!(
        !outcome.timed_out,
        "Claude already-exited status remains distinct from pipe disposal/readiness"
    );
    assert!(!outcome.settlement_failed);
    custody.settle().await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_checked_nonzero_exit_keeps_fallback_admission_open() {
    let custody = ClaudeProcessCustody::new();
    for exit_code in [7, 0] {
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", &format!("printf checked-output; exit {exit_code}")])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        let (child, owner) = custody.spawn(&mut command, Path::new("/bin/sh")).unwrap();
        let (ready_tx, ready_rx) = oneshot::channel();
        let driver = tokio::spawn(drive_child(
            child,
            owner,
            None,
            true,
            None,
            StderrMode::Capture { mirror: false },
            ready_tx,
            |mut stdout| async move {
                let mut bytes = Vec::new();
                stdout.read_to_end(&mut bytes).await?;
                Ok(bytes)
            },
        ));
        ready_rx.await.unwrap().unwrap();
        let result = tokio::time::timeout(Duration::from_secs(3), driver)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.status.unwrap().code(), Some(exit_code));
        assert_eq!(result.stdout.unwrap().unwrap(), b"checked-output");
        assert!(result.stderr.unwrap().unwrap().is_empty());
        assert!(result.stdin.is_ok());
        assert!(!result.settlement_failed);
        assert!(!result.timed_out);
        custody
            .ensure_open()
            .expect("checked nonzero exit must preserve fallback");
    }
    custody.settle().await.unwrap();
}
