//! Glue barriers; actual SDK Child/pipe acknowledgment is a separate integration obligation.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

struct Resource(Arc<AtomicBool>);

#[test]
fn test_checked_process_closeout_never_infers_absence_from_empty_waits() {
    use std::os::unix::process::ExitStatusExt;
    // Consumer cases only; these constructed snapshots are not native proof.
    let snapshot = |closed, observations| AgentWrapperProcessSettlementObservation {
        acknowledged: true,
        observations,
        closed_before_any_spawn: closed,
    };
    let outcome = |process_settlement| NativeTerminalOutcome {
        raw: Err(cancelled()),
        settlement: NativeSettlementOutcome::Checked(Ok(())),
        process_settlement,
    };
    let waits = vec![AgentWrapperProcessTerminationObservation {
        process_id: Some(123),
        status: Some(std::process::ExitStatus::from_raw(9)),
        recovery_status: Some(std::process::ExitStatus::from_raw(7 << 8)),
    }];
    for value in [
        None,
        Some(snapshot(None, Some(vec![]))),
        Some(snapshot(Some(false), Some(vec![]))),
        Some(snapshot(Some(true), None)),
        Some(snapshot(Some(true), Some(waits.clone()))),
        Some(snapshot(
            Some(false),
            Some(vec![AgentWrapperProcessTerminationObservation {
                process_id: None,
                status: Some(std::process::ExitStatus::from_raw(9)),
                recovery_status: None,
            }]),
        )),
    ] {
        let value = outcome(value);
        assert!(value.require_checked_process_closeout().is_err());
        assert!(value.initial_bootstrap_process_closeout().is_err());
    }
    let positive = outcome(Some(snapshot(Some(true), Some(vec![]))));
    assert!(matches!(
        positive.require_checked_process_closeout().unwrap(),
        CheckedNativeProcessCloseout::AdmissionClosedBeforeSpawn
    ));
    assert!(matches!(
        positive.initial_bootstrap_process_closeout().unwrap(),
        transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1::ClosedBeforeAnySpawn {}
    ));
    let waited = outcome(Some(snapshot(Some(false), Some(waits))));
    assert!(
        matches!(waited.require_checked_process_closeout().unwrap(), CheckedNativeProcessCloseout::Processes(values) if values.len() == 1 && values[0].status.unwrap().signal() == Some(9))
    );
    let transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1::ProcessesWaited {
        processes,
    } = waited.initial_bootstrap_process_closeout().unwrap()
    else {
        panic!("actual primary wait changed");
    };
    assert_eq!(processes[0].pid, 123);
    assert_eq!(
        processes[0].termination,
        transport_api_types::NativeProcessTerminationV1::Signaled { signal: 9 }
    );
    assert_eq!(
        processes[0].recovery_termination,
        Some(transport_api_types::NativeProcessTerminationV1::Exited { exit_code: 7 })
    );
    let mut unacknowledged = outcome(Some(snapshot(Some(true), Some(vec![]))));
    unacknowledged
        .process_settlement
        .as_mut()
        .unwrap()
        .acknowledged = false;
    assert!(unacknowledged.require_checked_process_closeout().is_err());
    assert!(unacknowledged.initial_bootstrap_process_closeout().is_err());
    unacknowledged
        .process_settlement
        .as_mut()
        .unwrap()
        .acknowledged = true;
    unacknowledged.settlement =
        NativeSettlementOutcome::Checked(Err(backend_error("actual unit acknowledgment failed")));
    assert!(unacknowledged.require_checked_process_closeout().is_err());
    assert!(unacknowledged.initial_bootstrap_process_closeout().is_err());
    assert!(
        matches!(positive.raw, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
}

impl Drop for Resource {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
fn cancelled() -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: "cancelled".into(),
    }
}

#[tokio::test]
async fn test_raw_cancelled_stays_exact_while_settlement_fails() {
    let (release, held) = tokio::sync::oneshot::channel();
    let (raw, terminal) = retain_native_completion(
        Box::pin(async { Err(cancelled()) }),
        Some(Box::pin(async move {
            held.await.unwrap();
            Err(backend_error("pipe acknowledgment failed"))
        })),
        (),
        || {},
        None,
        None,
    );
    let mut raw = raw;
    assert!(tokio::time::timeout(Duration::from_millis(10), &mut raw)
        .await
        .is_err());
    release.send(()).unwrap();
    assert!(
        matches!(raw.await, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
    let outcome = terminal.wait().await;
    assert!(
        matches!(&outcome.raw, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
    assert!(outcome.require_checked().is_err());
    assert!(
        matches!(outcome.enclosing_result(outcome.raw.as_ref().cloned().map_err(clone_wrapper_error)),
        Err(AgentWrapperError::Backend { message }) if message.contains("submission_outcome_unknown") && message.contains("cancelled"))
    );
}

#[tokio::test]
async fn test_terminal_waits_for_settlement_and_retains_resources() {
    let dropped = Arc::new(AtomicBool::new(false));
    let (release, held) = tokio::sync::oneshot::channel();
    let (raw, terminal) = retain_native_completion(
        Box::pin(async { Err(backend_error("provider error")) }),
        Some(Box::pin(async move {
            held.await.unwrap();
            Ok(())
        })),
        Resource(Arc::clone(&dropped)),
        || {},
        None,
        None,
    );
    let mut observation = Box::pin(terminal.wait());
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut observation)
            .await
            .is_err()
    );
    assert!(!dropped.load(Ordering::SeqCst));
    release.send(()).unwrap();
    let outcome = observation.await;
    assert!(outcome.require_checked().is_ok());
    assert!(dropped.load(Ordering::SeqCst));
    assert!(
        matches!(raw.await, Err(AgentWrapperError::Backend { message }) if message == "provider error")
    );
}

#[tokio::test]
async fn test_dropped_completion_requests_cancel_and_keeps_owner() {
    let dropped = Arc::new(AtomicBool::new(false));
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let (release, held) = tokio::sync::oneshot::channel();
    let (raw, terminal) = retain_native_completion(
        Box::pin(async move {
            cancelled.await.unwrap();
            Err(backend_error("cancelled"))
        }),
        Some(Box::pin(async move {
            held.await.unwrap();
            Ok(())
        })),
        Resource(Arc::clone(&dropped)),
        move || {
            let _ = cancel.send(());
        },
        None,
        None,
    );
    drop(raw);
    let mut observation = Box::pin(terminal.wait());
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut observation)
            .await
            .is_err()
    );
    assert!(!dropped.load(Ordering::SeqCst));
    release.send(()).unwrap();
    assert!(observation.await.require_checked().is_ok());
    assert!(dropped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_unsupported_is_not_checked_success() {
    let (raw, terminal) = retain_native_completion(
        Box::pin(async { Err(cancelled()) }),
        None,
        (),
        || {},
        None,
        None,
    );
    let raw = raw.await;
    let outcome = terminal.wait().await;
    assert!(matches!(
        &outcome.settlement,
        NativeSettlementOutcome::Unsupported
    ));
    assert!(outcome.require_checked().is_err());
    // Existing unsupported Fork completion remains functional and is not fabricated as checked.
    assert!(matches!(outcome.enclosing_result(raw),
        Err(AgentWrapperError::Backend { message }) if message == "cancelled"));
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        let (raw, terminal) = retain_native_completion(
            Box::pin(async {
                Ok(AgentWrapperCompletion {
                    status: std::process::ExitStatus::from_raw(0),
                    final_text: Some("unchanged initial Fork result".into()),
                    data: Some(serde_json::json!({"session":{"id":"fork-session"}})),
                })
            }),
            None,
            (),
            || {},
            None,
            None,
        );
        let raw = raw.await;
        let observed = terminal.wait().await;
        assert!(observed.require_checked().is_err());
        let unqualified = observed.enclosing_result(raw).unwrap();
        assert!(unqualified.status.success());
        assert_eq!(
            unqualified.final_text.as_deref(),
            Some("unchanged initial Fork result")
        );
        assert_eq!(
            unqualified.data,
            Some(serde_json::json!({"session":{"id":"fork-session"}}))
        );
    }
}

#[tokio::test]
async fn test_publisher_abandonment_cannot_succeed() {
    let (sender, receiver) = tokio::sync::watch::channel(None);
    drop(sender);
    let outcome = NativeTerminalObservation {
        receiver,
        route_disposal: tokio_util::sync::CancellationToken::new(),
    }
    .wait()
    .await;
    assert!(outcome.require_checked().is_err());
    assert!(outcome.raw.is_err());
}

#[tokio::test]
async fn test_route_disposal_signal_precedes_checked_acknowledgment() {
    let (release, held) = oneshot::channel();
    let (mut raw, terminal) = retain_native_completion(
        Box::pin(async { Err(backend_error("original failure")) }),
        Some(Box::pin(async move {
            held.await.unwrap();
            Ok(())
        })),
        (),
        || {},
        None,
        None,
    );
    tokio::time::timeout(Duration::from_secs(2), terminal.route_disposal_requested())
        .await
        .unwrap();
    assert!(tokio::time::timeout(Duration::from_millis(10), &mut raw)
        .await
        .is_err());
    let mut checked = Box::pin(terminal.wait());
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut checked)
            .await
            .is_err()
    );
    release.send(()).unwrap();
    assert!(checked.await.require_checked().is_ok());
    assert!(
        matches!(raw.await, Err(AgentWrapperError::Backend { message }) if message == "original failure")
    );
}

#[tokio::test]
async fn test_terminal_retains_resources_until_process_observation() {
    // Glue barrier only. This empty synthetic snapshot grants no native report,
    // process-absence assertion, service retirement or source permission.
    let dropped = Arc::new(AtomicBool::new(false));
    let (release, held) = tokio::sync::oneshot::channel();
    let (raw, terminal) = retain_native_completion_with_process_observation(
        Box::pin(async { Err(cancelled()) }),
        Some(Box::pin(async { Ok(()) })),
        Some(Box::pin(async move {
            held.await.unwrap();
            agent_api::AgentWrapperProcessSettlementObservation {
                acknowledged: true,
                observations: Some(Vec::new()),
                closed_before_any_spawn: None,
            }
        })),
        Resource(Arc::clone(&dropped)),
        || {},
        None,
        None,
    );
    let mut pending = Box::pin(terminal.wait());
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut pending)
            .await
            .is_err()
    );
    assert!(!dropped.load(Ordering::SeqCst));
    release.send(()).unwrap();
    let observed = pending.await;
    assert!(dropped.load(Ordering::SeqCst));
    assert!(observed.require_checked().is_ok());
    assert!(observed
        .require_checked_process_observations()
        .unwrap()
        .is_empty());
    assert!(
        matches!(raw.await, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
}
