use super::*;

#[tokio::test]
async fn cancel_signal_is_idempotent_and_supports_late_subscribers() {
    let cancel = HarnessCancelSignal::new();

    let waiter = tokio::spawn({
        let cancel = cancel.clone();
        async move {
            cancel.cancelled().await;
        }
    });

    tokio::task::yield_now().await;

    cancel.cancel();
    cancel.cancel();
    assert!(cancel.is_cancelled());
    waiter.await.expect("waiter observes cancellation");

    cancel.cancelled().await;
}
