use super::*;

#[tokio::test]
async fn completion_sender_selects_cancelled_error_but_waits_for_backend_exit() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let cancel = HarnessCancelSignal::new();

    let term_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let request_termination: RequestTerminationHook = {
        let term_calls = term_calls.clone();
        std::sync::Arc::new(move || {
            term_calls.fetch_add(1, Ordering::SeqCst);
        })
    };

    let (backend_done_tx, backend_done_rx) = oneshot::channel::<()>();
    let completion: DynBackendCompletionFuture<_, _> = Box::pin(async move {
        backend_done_rx.await.expect("backend completion released");
        Ok::<ToyCompletion, ToyBackendError>(ToyCompletion)
    });

    let (completion_tx, mut completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let task = tokio::spawn(send_completion_with_cancel(
        adapter,
        completion,
        cancel.clone(),
        Some(request_termination),
        completion_tx,
    ));

    cancel.cancel();

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    assert!(matches!(
        Pin::new(&mut completion_rx).poll(&mut cx),
        Poll::Pending
    ));

    let _ = backend_done_tx.send(());
    let outcome = completion_rx.await.expect("completion sent");
    assert!(
        matches!(
            outcome,
            Err(AgentWrapperError::Backend { ref message }) if message == "cancelled"
        ),
        "completion must resolve to the pinned cancelled error"
    );
    assert_eq!(term_calls.load(Ordering::SeqCst), 1);

    task.await.expect("completion task completes");
}

#[tokio::test]
async fn completion_sender_preserves_backend_outcome_when_completion_wins() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: false });
    let cancel = HarnessCancelSignal::new();

    let term_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let request_termination: RequestTerminationHook = {
        let term_calls = term_calls.clone();
        std::sync::Arc::new(move || {
            term_calls.fetch_add(1, Ordering::SeqCst);
        })
    };

    let completion: DynBackendCompletionFuture<_, _> =
        Box::pin(async move { Ok::<ToyCompletion, ToyBackendError>(ToyCompletion) });

    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();

    let task = tokio::spawn(send_completion_with_cancel(
        adapter,
        completion,
        cancel.clone(),
        Some(request_termination),
        completion_tx,
    ));

    let outcome = completion_rx.await.expect("completion sent");
    assert!(
        outcome.is_ok(),
        "backend completion should win when ready first"
    );

    cancel.cancel();
    assert_eq!(term_calls.load(Ordering::SeqCst), 0);

    task.await.expect("completion task completes");
}
