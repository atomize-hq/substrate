use super::*;

use super::support::{ControlledEndStream, CountingStream};

#[tokio::test]
async fn pump_backend_events_smoke_forwards_in_order() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let events = futures_util::stream::iter([
        Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text("one".to_string())),
        Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text("two".to_string())),
    ]);
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let (tx, mut rx) = mpsc::channel::<AgentWrapperEvent>(DEFAULT_EVENT_CHANNEL_CAPACITY);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx, None));

    let mut texts = Vec::<String>::new();
    while let Some(ev) = rx.recv().await {
        if ev.kind == AgentWrapperEventKind::TextOutput {
            if let Some(text) = ev.text {
                texts.push(text);
            }
        }
    }

    handle.await.expect("pump task completes");
    assert_eq!(texts, vec!["one".to_string(), "two".to_string()]);
}

#[tokio::test]
async fn pump_blocks_under_backpressure_until_receiver_polls() {
    #[derive(Default)]
    struct BackpressureAdapter {
        call_count: AtomicUsize,
        second_mapped_tx: Mutex<Option<oneshot::Sender<()>>>,
    }

    impl BackendHarnessAdapter for BackpressureAdapter {
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
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            let mapped = self.call_count.fetch_add(1, Ordering::SeqCst);
            if mapped == 1 {
                if let Some(tx) = self.second_mapped_tx.lock().unwrap().take() {
                    let _ = tx.send(());
                }
            }

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
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let (second_mapped_tx, second_mapped_rx) = oneshot::channel::<()>();
    let adapter = std::sync::Arc::new(BackpressureAdapter {
        call_count: AtomicUsize::new(0),
        second_mapped_tx: Mutex::new(Some(second_mapped_tx)),
    });

    let events = futures_util::stream::iter([
        Ok::<String, ()>("one".to_string()),
        Ok::<String, ()>("two".to_string()),
    ]);
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(1);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx, None));

    second_mapped_rx.await.expect("second event mapped");
    tokio::task::yield_now().await;
    assert!(
        !handle.is_finished(),
        "pump must be blocked on bounded send"
    );

    let mut texts = Vec::<String>::new();
    while let Some(ev) = rx.recv().await {
        if ev.kind == AgentWrapperEventKind::TextOutput {
            if let Some(text) = ev.text {
                texts.push(text);
            }
        }
    }

    handle.await.expect("pump task completes");
    assert_eq!(texts, vec!["one".to_string(), "two".to_string()]);
}

#[tokio::test]
async fn pump_stops_forwarding_after_receiver_drop_but_drains_to_end() {
    let total = 20usize;
    let consumed = std::sync::Arc::new(AtomicUsize::new(0));
    let items: VecDeque<Result<ToyEvent, ToyBackendError>> = (0..total)
        .map(|i| Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text(format!("ev-{i}"))))
        .collect();

    let events = CountingStream {
        items,
        consumed: consumed.clone(),
    };
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(1);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx, None));

    let first = rx.recv().await.expect("at least one forwarded event");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);
    drop(rx);

    handle.await.expect("pump task completes");
    assert_eq!(
        consumed.load(Ordering::SeqCst),
        total,
        "backend stream must be fully drained after receiver drop"
    );
}

#[tokio::test]
async fn pump_enforces_bounds_before_forwarding() {
    struct BoundsAdapter;

    impl BackendHarnessAdapter for BoundsAdapter {
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

        type BackendEvent = ();
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
            panic!("spawn unused in pump tests");
        }

        fn map_event(&self, _event: Self::BackendEvent) -> Vec<crate::AgentWrapperEvent> {
            vec![crate::AgentWrapperEvent {
                agent_kind: toy_kind(),
                kind: AgentWrapperEventKind::Error,
                channel: Some("error".to_string()),
                text: None,
                message: Some("a".repeat(crate::bounds::MESSAGE_BOUND_BYTES + 10)),
                data: None,
            }]
        }

        fn map_completion(
            &self,
            _completion: Self::BackendCompletion,
        ) -> Result<crate::AgentWrapperCompletion, crate::AgentWrapperError> {
            panic!("map_completion unused in pump tests");
        }

        fn redact_error(
            &self,
            _phase: BackendHarnessErrorPhase,
            _err: &Self::BackendError,
        ) -> String {
            "unused".to_string()
        }
    }

    let events = futures_util::stream::iter([Ok::<(), ()>(())]);
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let adapter = std::sync::Arc::new(BoundsAdapter);
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx, None));

    let ev = rx.recv().await.expect("one event forwarded");
    let message = ev.message.as_deref().expect("message present");
    assert!(message.len() <= crate::bounds::MESSAGE_BOUND_BYTES);
    assert!(message.ends_with("…(truncated)"));

    while rx.recv().await.is_some() {}
    handle.await.expect("pump task completes");
}

#[tokio::test]
async fn pump_finality_sender_dropped_only_after_backend_stream_ends() {
    let (finish_tx, finish_rx) = oneshot::channel::<()>();
    let events = ControlledEndStream::<ToyEvent, ToyBackendError> {
        first: Some(Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text(
            "hello".to_string(),
        ))),
        finish_rx,
    };
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let handle = tokio::spawn(pump_backend_events(adapter, events, tx, None));

    let ev = rx.recv().await.expect("first event forwarded");
    assert_eq!(ev.kind, AgentWrapperEventKind::TextOutput);

    tokio::task::yield_now().await;
    assert!(
        matches!(
            rx.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty)
        ),
        "events stream must not be final before backend stream ends"
    );
    assert!(
        !handle.is_finished(),
        "pump must not finish before stream end"
    );

    let _ = finish_tx.send(());
    handle.await.expect("pump task completes");
    assert!(
        rx.recv().await.is_none(),
        "events stream must be final after backend stream ends"
    );
}
