use super::*;

use super::support::ControlledEndStream;

#[tokio::test]
async fn cancellation_closes_events_but_does_not_accelerate_completion() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let cancel = HarnessCancelSignal::new();

    let (backend_done_tx, backend_done_rx) = oneshot::channel::<()>();
    let completion: DynBackendCompletionFuture<_, _> = Box::pin(async move {
        backend_done_rx.await.expect("backend completion released");
        Ok::<ToyCompletion, ToyBackendError>(ToyCompletion)
    });

    let (events_finish_tx, events_finish_rx) = oneshot::channel::<()>();
    let events = ControlledEndStream::<ToyEvent, ToyBackendError> {
        first: Some(Ok::<ToyEvent, ToyBackendError>(ToyEvent::Text(
            "one".to_string(),
        ))),
        finish_rx: events_finish_rx,
    };
    let events: DynBackendEventStream<_, _> = Box::pin(events);

    let (tx, rx) = mpsc::channel::<crate::AgentWrapperEvent>(8);
    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let pump_task = tokio::spawn(pump_backend_events_with_cancel(
        adapter.clone(),
        events,
        tx,
        cancel.clone(),
        None,
    ));
    let completion_task = tokio::spawn(send_completion_with_cancel(
        adapter,
        completion,
        cancel.clone(),
        None,
        completion_tx,
    ));

    let mut handle = crate::run_handle_gate::build_gated_run_handle(rx, completion_rx);

    let first = handle.events.next().await.expect("first event forwarded");
    assert_eq!(first.kind, AgentWrapperEventKind::TextOutput);

    cancel.cancel();

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
    pump_task.await.expect("pump task completes");
    completion_task.await.expect("completion task completes");
}
