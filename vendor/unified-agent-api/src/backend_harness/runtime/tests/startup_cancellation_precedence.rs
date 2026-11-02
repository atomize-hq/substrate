use super::*;

#[tokio::test]
async fn control_startup_failure_prefers_cancelled_when_cancel_arrives_before_failure_surfaces() {
    struct DelayedFailingSpawnAdapter {
        spawn_started_tx: Mutex<Option<oneshot::Sender<()>>>,
        spawn_release_rx: Mutex<Option<oneshot::Receiver<()>>>,
    }

    impl BackendHarnessAdapter for DelayedFailingSpawnAdapter {
        fn kind(&self) -> crate::AgentWrapperKind {
            toy_kind()
        }

        fn supported_extension_keys(&self) -> &'static [&'static str] {
            &[]
        }

        type Policy = ();

        fn validate_and_extract_policy(
            &self,
            _request: &crate::AgentWrapperRunRequest,
        ) -> Result<Self::Policy, crate::AgentWrapperError> {
            Ok(())
        }

        type BackendEvent = String;
        type BackendCompletion = ();
        type BackendError = ToyBackendError;

        fn spawn(
            &self,
            _req: contract::NormalizedRequest<Self::Policy>,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<
                            contract::BackendSpawn<
                                Self::BackendEvent,
                                Self::BackendCompletion,
                                Self::BackendError,
                            >,
                            Self::BackendError,
                        >,
                    > + Send
                    + 'static,
            >,
        > {
            let spawn_started_tx = self
                .spawn_started_tx
                .lock()
                .unwrap()
                .take()
                .expect("spawn started signal available once");
            let spawn_release_rx = self
                .spawn_release_rx
                .lock()
                .unwrap()
                .take()
                .expect("spawn release gate available once");

            Box::pin(async move {
                let _ = spawn_started_tx.send(());
                spawn_release_rx.await.expect("startup released");
                Err(ToyBackendError {
                    secret: "SECRET_SPAWN".to_string(),
                })
            })
        }

        fn map_event(&self, _event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            Vec::new()
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            Ok(crate::AgentWrapperCompletion {
                status: success_exit_status(),
                final_text: None,
                data: None,
            })
        }

        fn redact_error(
            &self,
            phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            let phase = match phase {
                BackendHarnessErrorPhase::Spawn => "spawn",
                BackendHarnessErrorPhase::Stream => "stream",
                BackendHarnessErrorPhase::Completion => "completion",
            };
            format!("toy backend error (redacted): phase={phase}")
        }
    }

    let (spawn_started_tx, spawn_started_rx) = oneshot::channel::<()>();
    let (spawn_release_tx, spawn_release_rx) = oneshot::channel::<()>();
    let adapter = std::sync::Arc::new(DelayedFailingSpawnAdapter {
        spawn_started_tx: Mutex::new(Some(spawn_started_tx)),
        spawn_release_rx: Mutex::new(Some(spawn_release_rx)),
    });
    let request = crate::AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let crate::AgentWrapperRunControl { mut handle, cancel } =
        run_harnessed_backend_control(adapter, BackendDefaults::default(), request, None)
            .await
            .expect("control entrypoint succeeds");

    spawn_started_rx.await.expect("startup task entered spawn");

    cancel.cancel();
    let _ = spawn_release_tx.send(());

    assert!(
        tokio::time::timeout(Duration::from_secs(1), handle.events.next())
            .await
            .expect("events do not hang")
            .is_none(),
        "startup failure must not surface an error event after cancellation"
    );

    let outcome = handle.completion.await;
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "startup failure must resolve to the pinned cancelled error when cancel wins"
    );
}

#[tokio::test]
async fn control_startup_failure_prefers_cancelled_while_error_send_is_backpressured() {
    let cancel_signal = HarnessCancelSignal::new();
    let term_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let request_termination: RequestTerminationHook = {
        let term_calls = term_calls.clone();
        std::sync::Arc::new(move || {
            term_calls.fetch_add(1, Ordering::SeqCst);
        })
    };

    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(1);
    tx.send(crate::AgentWrapperEvent {
        agent_kind: toy_kind(),
        kind: AgentWrapperEventKind::Status,
        channel: Some("status".to_string()),
        text: None,
        message: Some("prefill".to_string()),
        data: None,
    })
    .await
    .expect("prefill send succeeds");
    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let task = tokio::spawn(surface_control_startup_failure(
        toy_kind(),
        "toy backend error (redacted): phase=spawn".to_string(),
        cancel_signal.clone(),
        Some(request_termination),
        tx,
        completion_tx,
    ));

    tokio::task::yield_now().await;
    cancel_signal.cancel();

    let outcome = completion_rx.await.expect("completion sent");
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "cancellation must win while the spawn error send is pending"
    );
    assert_eq!(
        term_calls.load(Ordering::SeqCst),
        1,
        "request termination hook should run once when cancellation wins"
    );

    let first = rx.recv().await.expect("prefill event remains queued");
    assert_eq!(first.message.as_deref(), Some("prefill"));
    assert!(
        tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("receiver does not hang")
            .is_none(),
        "no spawn error event should be forwarded after cancellation wins"
    );

    task.await.expect("startup failure task completes");
}
