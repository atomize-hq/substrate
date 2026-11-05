use super::super::super::settlement::{BackendAttemptCustody, SettlementObservation};
use super::*;
use std::sync::Arc;
use tokio::sync::Notify;

struct CustodyAdapter {
    entered: Arc<Notify>,
    settling: Arc<Notify>,
    release: Arc<Notify>,
    hold_startup: bool,
    fail_startup: bool,
    fail_settlement: bool,
    fail_completion: bool,
}
impl BackendHarnessAdapter for CustodyAdapter {
    fn kind(&self) -> crate::AgentWrapperKind {
        toy_kind()
    }
    fn supported_extension_keys(&self) -> &'static [&'static str] {
        &[]
    }
    type Policy = ();
    type BackendEvent = ToyEvent;
    type BackendCompletion = ToyCompletion;
    type BackendError = ToyBackendError;
    fn validate_and_extract_policy(
        &self,
        _: &AgentWrapperRunRequest,
    ) -> Result<(), AgentWrapperError> {
        Ok(())
    }
    fn attempt_custody(&self, _: &NormalizedRequest<()>) -> Option<BackendAttemptCustody> {
        let settling = self.settling.clone();
        let release = self.release.clone();
        let fail = self.fail_settlement;
        Some(BackendAttemptCustody {
            observe_closed_before_any_spawn: None,
            observe_processes: None,
            request_termination: Arc::new(|| {}),
            settle: Arc::new(move || {
                let settling = settling.clone();
                let release = release.clone();
                Box::pin(async move {
                    settling.notify_one();
                    release.notified().await;
                    if fail {
                        Err(AgentWrapperError::Backend {
                            message: "injected checked settlement failure".into(),
                        })
                    } else {
                        Ok(())
                    }
                })
            }),
        })
    }
    #[allow(clippy::type_complexity)]
    fn spawn(
        &self,
        _: NormalizedRequest<()>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BackendSpawn<ToyEvent, ToyCompletion, ToyBackendError>,
                        ToyBackendError,
                    >,
                > + Send,
        >,
    > {
        let entered = self.entered.clone();
        let hold = self.hold_startup;
        let fail = self.fail_startup;
        let fail_completion = self.fail_completion;
        Box::pin(async move {
            entered.notify_one();
            if hold {
                std::future::pending::<()>().await;
            }
            if fail {
                return Err(ToyBackendError {
                    secret: "not public".into(),
                });
            }
            Ok(BackendSpawn {
                events: Box::pin(futures_util::stream::empty()),
                completion: Box::pin(async move {
                    if fail_completion {
                        Err(ToyBackendError {
                            secret: "not public".into(),
                        })
                    } else {
                        Ok(ToyCompletion)
                    }
                }),
                events_observability: None,
            })
        })
    }
    fn map_event(&self, event: ToyEvent) -> Vec<crate::AgentWrapperEvent> {
        ToyAdapter { fail_spawn: false }.map_event(event)
    }
    fn map_completion(
        &self,
        completion: ToyCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError> {
        ToyAdapter { fail_spawn: false }.map_completion(completion)
    }
    fn redact_error(&self, phase: BackendHarnessErrorPhase, error: &ToyBackendError) -> String {
        ToyAdapter { fail_spawn: false }.redact_error(phase, error)
    }
}
fn adapter(hold_startup: bool, fail_startup: bool, fail_settlement: bool) -> Arc<CustodyAdapter> {
    Arc::new(CustodyAdapter {
        entered: Arc::new(Notify::new()),
        settling: Arc::new(Notify::new()),
        release: Arc::new(Notify::new()),
        hold_startup,
        fail_startup,
        fail_settlement,
        fail_completion: false,
    })
}
fn request() -> AgentWrapperRunRequest {
    AgentWrapperRunRequest {
        prompt: "owned attempt".into(),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_startup_cancel_waits_for_ack_and_keeps_failed_settlement_separate() {
    let adapter = adapter(true, false, true);
    let control =
        run_harnessed_backend_control(adapter.clone(), BackendDefaults::default(), request(), None)
            .await
            .unwrap();
    let mut settlement = control
        .cancel
        .settlement()
        .expect("actual selected custody supplied");
    adapter.entered.notified().await;
    control.cancel.cancel();
    let mut events = control.handle.events;
    let mut completion = control.handle.completion;
    adapter.settling.notified().await;
    assert!(tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .unwrap()
        .is_none());
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut completion)
            .await
            .is_err()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut settlement)
            .await
            .is_err()
    );
    adapter.release.notify_one();
    assert!(
        matches!(completion.await, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
    // Mandatory enclosing join: the completion's normative cancellation value does not prove
    // cleanup. An enclosing terminal publisher checking this result must refuse success.
    assert!(settlement.await.is_err());
}

#[tokio::test]
async fn test_startup_error_does_not_publish_terminal_before_ack() {
    let adapter = adapter(false, true, false);
    let control =
        run_harnessed_backend_control(adapter.clone(), BackendDefaults::default(), request(), None)
            .await
            .unwrap();
    let mut settlement = control.cancel.settlement().unwrap();
    drop(control.handle.events);
    let mut completion = control.handle.completion;
    adapter.settling.notified().await;
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut completion)
            .await
            .is_err()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut settlement)
            .await
            .is_err()
    );
    adapter.release.notify_one();
    assert!(completion.await.is_err());
    settlement.await.unwrap();
}

#[tokio::test]
async fn test_successful_non_cancel_completion_cannot_hide_failed_settlement() {
    let adapter = adapter(false, false, true);
    let control =
        run_harnessed_backend_control(adapter.clone(), BackendDefaults::default(), request(), None)
            .await
            .unwrap();
    let settlement = control.cancel.settlement().unwrap();
    drop(control.handle.events);
    let mut completion = control.handle.completion;
    adapter.settling.notified().await;
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut completion)
            .await
            .is_err()
    );
    adapter.release.notify_one();
    assert!(completion.await.is_err());
    assert!(settlement.await.is_err());
}

#[tokio::test]
async fn test_full_consumer_drop_retains_real_settlement_observer() {
    let adapter = adapter(true, false, true);
    let control =
        run_harnessed_backend_control(adapter.clone(), BackendDefaults::default(), request(), None)
            .await
            .unwrap();
    let cancel = control.cancel.clone();
    let settlement = cancel.settlement().unwrap();
    adapter.entered.notified().await;
    drop(control);
    cancel.cancel();
    adapter.settling.notified().await;
    adapter.release.notify_one();
    assert!(settlement.await.is_err());
}

#[tokio::test]
async fn test_missing_ack_is_none_and_abandoned_publisher_is_error() {
    let cancel = AgentWrapperCancelHandle::new(|| {});
    assert!(cancel.settlement().is_none());
    let observation = SettlementObservation::default();
    let observer = (observation.factory())();
    drop(observation.publisher());
    assert!(observer.await.is_err());
}

// The ACK gate is deliberate harness injection, not an actual OS Child.wait probe.
// These tests exercise the SDK's real public cancellation/observation handle and
// the hidden failure branches before their settlement await/public completion.
#[tokio::test]
async fn test_disposal_observation_precedes_hidden_failure_and_cancel_ack() {
    for kind in ["startup-error", "completion-error", "cancel"] {
        for fail_ack in [false, true] {
            let mut adapter = adapter(kind == "cancel", kind == "startup-error", fail_ack);
            Arc::get_mut(&mut adapter).unwrap().fail_completion = kind == "completion-error";
            let expected = if kind == "cancel" {
                "cancelled".to_string()
            } else if fail_ack {
                if kind == "startup-error" {
                    "backend process settlement failed".to_string()
                } else {
                    "injected checked settlement failure".to_string()
                }
            } else {
                adapter.redact_error(
                    if kind == "startup-error" {
                        BackendHarnessErrorPhase::Spawn
                    } else {
                        BackendHarnessErrorPhase::Completion
                    },
                    &ToyBackendError {
                        secret: "not public".into(),
                    },
                )
            };
            let control = run_harnessed_backend_control(
                adapter.clone(),
                BackendDefaults::default(),
                request(),
                None,
            )
            .await
            .unwrap();
            let requested = control
                .cancel
                .disposal_requested()
                .expect("supported actual handle");
            let mut settlement = control.cancel.settlement().unwrap();
            if kind == "cancel" {
                adapter.entered.notified().await;
                control.cancel.cancel();
            }
            tokio::time::timeout(Duration::from_secs(2), requested)
                .await
                .unwrap();
            // A new observer acquired after publication must see the same latch.
            tokio::time::timeout(
                Duration::from_secs(2),
                control.cancel.disposal_requested().unwrap(),
            )
            .await
            .unwrap();
            adapter.settling.notified().await;
            let mut events = control.handle.events;
            let mut completion = control.handle.completion;
            if kind == "startup-error" {
                assert!(
                    tokio::time::timeout(Duration::from_millis(25), events.next())
                        .await
                        .is_err(),
                    "startup error is still hidden behind the held ACK"
                );
            }
            drop(events);
            assert!(
                tokio::time::timeout(Duration::from_millis(25), &mut completion)
                    .await
                    .is_err()
            );
            assert!(
                tokio::time::timeout(Duration::from_millis(25), &mut settlement)
                    .await
                    .is_err()
            );
            adapter.release.notify_one();
            assert!(
                matches!(completion.await, Err(AgentWrapperError::Backend { message }) if message == expected)
            );
            assert_eq!(settlement.await.is_err(), fail_ack);
        }
    }
}

#[tokio::test]
async fn test_disposal_publisher_drop_is_fail_closed_without_success_ack() {
    let unsupported = AgentWrapperCancelHandle::new(|| {});
    assert!(unsupported.disposal_requested().is_none());
    assert!(unsupported.settlement().is_none());
    let observation = SettlementObservation::default();
    let disposal = (observation.disposal_factory())();
    let settlement = (observation.factory())();
    drop(observation.publisher());
    tokio::time::timeout(Duration::from_secs(2), disposal)
        .await
        .unwrap();
    assert!(settlement.await.is_err());
}
