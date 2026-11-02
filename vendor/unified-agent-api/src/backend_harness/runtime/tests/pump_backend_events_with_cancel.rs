use super::*;

use super::support::{CountingStream, GatedCountingStream};

#[tokio::test]
async fn pump_with_cancel_closes_universal_stream_but_still_drains_typed_stream() {
    let total = 3usize;
    let consumed = std::sync::Arc::new(AtomicUsize::new(0));
    let (gate_tx, gate_rx) = oneshot::channel::<()>();

    let stream = GatedCountingStream {
        first: Some(Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text(
            "ev-0".to_string(),
        ))),
        rest: (1..total)
            .map(|i| Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text(format!("ev-{i}"))))
            .collect(),
        gate_rx: Some(gate_rx),
        consumed: consumed.clone(),
    };
    let events: DynBackendEventStream<_, _> = Box::pin(stream);

    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let (tx, mut rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let cancel = HarnessCancelSignal::new();
    let handle = tokio::spawn(pump_backend_events_with_cancel(
        adapter,
        events,
        tx,
        cancel.clone(),
        None,
    ));

    let first = rx.recv().await.expect("at least one forwarded event");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);

    cancel.cancel();

    assert!(
        tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("recv does not hang")
            .is_none(),
        "universal stream must close after cancellation"
    );

    let _ = gate_tx.send(());
    handle.await.expect("pump task completes");
    assert_eq!(
        consumed.load(Ordering::SeqCst),
        total,
        "typed backend stream must be drained to end after cancellation"
    );
}

#[tokio::test]
async fn pump_with_cancel_preserves_drain_on_drop_posture() {
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
    let cancel = HarnessCancelSignal::new();
    let handle = tokio::spawn(pump_backend_events_with_cancel(
        adapter, events, tx, cancel, None,
    ));

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
