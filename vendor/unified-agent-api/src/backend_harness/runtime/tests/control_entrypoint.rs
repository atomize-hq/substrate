use super::*;

use super::support::ControlledEndStream;

#[tokio::test]
async fn control_entrypoint_exposes_cancel_handle_wired_to_driver() {
    struct ControlAdapter {
        events_finish_rx: Mutex<Option<oneshot::Receiver<()>>>,
        backend_done_rx: Mutex<Option<oneshot::Receiver<()>>>,
    }

    impl BackendHarnessAdapter for ControlAdapter {
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
        type BackendError = ();

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
            let finish_rx = self
                .events_finish_rx
                .lock()
                .unwrap()
                .take()
                .expect("events gate available once");
            let backend_done_rx = self
                .backend_done_rx
                .lock()
                .unwrap()
                .take()
                .expect("backend gate available once");

            let events = ControlledEndStream::<String, ()> {
                first: Some(Ok::<String, ()>("one".to_string())),
                finish_rx,
            };
            let completion: DynBackendCompletionFuture<(), ()> = Box::pin(async move {
                backend_done_rx.await.expect("backend completion released");
                Ok::<(), ()>(())
            });

            Box::pin(async move {
                Ok(BackendSpawn {
                    events: Box::pin(events),
                    completion,
                    events_observability: None,
                })
            })
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(event),
                message: None,
                data: None,
            }]
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
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (events_finish_tx, events_finish_rx) = oneshot::channel::<()>();
    let (backend_done_tx, backend_done_rx) = oneshot::channel::<()>();

    let adapter = std::sync::Arc::new(ControlAdapter {
        events_finish_rx: Mutex::new(Some(events_finish_rx)),
        backend_done_rx: Mutex::new(Some(backend_done_rx)),
    });

    let request = crate::AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let crate::AgentWrapperRunControl { mut handle, cancel } =
        run_harnessed_backend_control(adapter, BackendDefaults::default(), request, None)
            .await
            .expect("control entrypoint succeeds");

    let first = handle.events.next().await.expect("first event forwarded");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);

    cancel.cancel();
    tokio::task::yield_now().await;

    assert!(
        tokio::time::timeout(Duration::from_secs(1), handle.events.next())
            .await
            .expect("events do not hang")
            .is_none(),
        "events stream must be closed after cancellation"
    );

    {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(
            handle.completion.as_mut().poll(&mut cx),
            Poll::Pending
        ));
    }

    let _ = backend_done_tx.send(());
    let outcome = handle.completion.await;
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "completion must resolve to the pinned cancelled error after backend exit"
    );

    let _ = events_finish_tx.send(());
}

#[tokio::test]
async fn control_entrypoint_returns_before_spawn_finishes() {
    struct DeferredSpawnAdapter {
        spawn_started_tx: Mutex<Option<oneshot::Sender<()>>>,
        spawn_release_rx: Mutex<Option<oneshot::Receiver<()>>>,
    }

    impl BackendHarnessAdapter for DeferredSpawnAdapter {
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
        type BackendError = ();

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

                Ok(BackendSpawn {
                    events: Box::pin(futures_util::stream::empty()),
                    completion: Box::pin(async { Ok::<(), ()>(()) }),
                    events_observability: None,
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
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (spawn_started_tx, spawn_started_rx) = oneshot::channel::<()>();
    let (spawn_release_tx, spawn_release_rx) = oneshot::channel::<()>();
    let adapter = std::sync::Arc::new(DeferredSpawnAdapter {
        spawn_started_tx: Mutex::new(Some(spawn_started_tx)),
        spawn_release_rx: Mutex::new(Some(spawn_release_rx)),
    });
    let request = crate::AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let crate::AgentWrapperRunControl {
        mut handle,
        cancel: _,
    } = tokio::time::timeout(
        Duration::from_millis(100),
        run_harnessed_backend_control(adapter, BackendDefaults::default(), request, None),
    )
    .await
    .expect("control entrypoint should not wait for startup")
    .expect("control entrypoint succeeds");

    spawn_started_rx.await.expect("startup task entered spawn");

    {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(
            handle.completion.as_mut().poll(&mut cx),
            Poll::Pending
        ));
    }

    let _ = spawn_release_tx.send(());

    assert!(
        tokio::time::timeout(Duration::from_secs(1), handle.events.next())
            .await
            .expect("events do not hang")
            .is_none(),
        "empty events stream must close once startup finishes"
    );
    assert!(
        handle.completion.await.is_ok(),
        "completion should resolve successfully after deferred startup"
    );
}

#[tokio::test]
async fn run_entrypoint_returns_before_spawn_finishes() {
    struct DeferredSpawnAdapter {
        spawn_started_tx: Mutex<Option<oneshot::Sender<()>>>,
        spawn_release_rx: Mutex<Option<oneshot::Receiver<()>>>,
    }

    impl BackendHarnessAdapter for DeferredSpawnAdapter {
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
        type BackendError = ();

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

                Ok(BackendSpawn {
                    events: Box::pin(futures_util::stream::empty()),
                    completion: Box::pin(async { Ok::<(), ()>(()) }),
                    events_observability: None,
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
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (spawn_started_tx, spawn_started_rx) = oneshot::channel::<()>();
    let (spawn_release_tx, spawn_release_rx) = oneshot::channel::<()>();
    let adapter = std::sync::Arc::new(DeferredSpawnAdapter {
        spawn_started_tx: Mutex::new(Some(spawn_started_tx)),
        spawn_release_rx: Mutex::new(Some(spawn_release_rx)),
    });
    let request = crate::AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let mut handle = tokio::time::timeout(
        Duration::from_millis(100),
        run_harnessed_backend(adapter, BackendDefaults::default(), request),
    )
    .await
    .expect("run entrypoint should not wait for startup")
    .expect("run entrypoint succeeds");

    spawn_started_rx.await.expect("startup task entered spawn");

    {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(
            handle.completion.as_mut().poll(&mut cx),
            Poll::Pending
        ));
    }

    let _ = spawn_release_tx.send(());

    assert!(
        tokio::time::timeout(Duration::from_secs(1), handle.events.next())
            .await
            .expect("events do not hang")
            .is_none(),
        "empty events stream must close once startup finishes"
    );
    assert!(
        handle.completion.await.is_ok(),
        "completion should resolve successfully after deferred startup"
    );
}

#[tokio::test]
async fn control_entrypoint_cancels_while_startup_is_in_flight() {
    struct StartupDropSignal(Option<oneshot::Sender<()>>);

    impl Drop for StartupDropSignal {
        fn drop(&mut self) {
            if let Some(tx) = self.0.take() {
                let _ = tx.send(());
            }
        }
    }

    struct PendingSpawnAdapter {
        spawn_started_tx: Mutex<Option<oneshot::Sender<()>>>,
        spawn_dropped_tx: Mutex<Option<oneshot::Sender<()>>>,
    }

    impl BackendHarnessAdapter for PendingSpawnAdapter {
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
        type BackendError = ();

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
            let spawn_dropped_tx = self
                .spawn_dropped_tx
                .lock()
                .unwrap()
                .take()
                .expect("spawn dropped signal available once");

            Box::pin(async move {
                let _drop_signal = StartupDropSignal(Some(spawn_dropped_tx));
                let _ = spawn_started_tx.send(());
                futures_util::future::pending::<()>().await;
                unreachable!("pending startup future should be dropped on cancellation");
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
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (spawn_started_tx, spawn_started_rx) = oneshot::channel::<()>();
    let (spawn_dropped_tx, spawn_dropped_rx) = oneshot::channel::<()>();
    let adapter = std::sync::Arc::new(PendingSpawnAdapter {
        spawn_started_tx: Mutex::new(Some(spawn_started_tx)),
        spawn_dropped_tx: Mutex::new(Some(spawn_dropped_tx)),
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

    tokio::time::timeout(Duration::from_secs(1), spawn_dropped_rx)
        .await
        .expect("startup future should be dropped after cancellation")
        .expect("startup drop signal should be delivered");

    assert!(
        tokio::time::timeout(Duration::from_secs(1), handle.events.next())
            .await
            .expect("events do not hang")
            .is_none(),
        "events stream must close while startup is still pending"
    );

    let outcome = handle.completion.await;
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "startup cancellation must resolve to the pinned cancelled error"
    );
}
