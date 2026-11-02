//! DR-0012 / BH-C05 — completion gating semantics.
//!
//! This module constructs an [`AgentWrapperRunHandle`] such that `completion` is not observable
//! until (1) the backend completion outcome is ready, and (2) the universal events stream is
//! final — unless the consumer explicitly opts out by dropping the `events` stream.
//!
//! Definitions (pinned):
//! - Stream finality: the events [`mpsc::Receiver<AgentWrapperEvent>`] yields `None` (the upstream
//!   [`mpsc::Sender`] was dropped). The pump/drainer (BH-C04 / SEAM-3) is responsible for dropping
//!   the sender only at true stream end (receiver drop is not finality).
//! - Consumer drop: dropping the `events` stream instance unblocks the finality gate (consumer
//!   opt-out), while upstream draining may continue in the background per BH-C04.
//! - Completion outcome: sourced from a [`oneshot`]; if the completion channel is dropped, the
//!   completion resolves as [`AgentWrapperError::Backend`] with message `"completion channel dropped"`.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use tokio::sync::{mpsc, oneshot};

use crate::{
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperRunHandle,
    DynAgentWrapperCompletion, DynAgentWrapperEventStream,
};

pub(crate) fn build_gated_run_handle(
    rx: mpsc::Receiver<AgentWrapperEvent>,
    completion_rx: oneshot::Receiver<Result<AgentWrapperCompletion, AgentWrapperError>>,
) -> AgentWrapperRunHandle {
    let (events_done_tx, events_done_rx) = oneshot::channel::<()>();

    let events: DynAgentWrapperEventStream = Box::pin(FinalityEventStream {
        rx,
        events_done_tx: Some(events_done_tx),
    });

    let completion: DynAgentWrapperCompletion = Box::pin(async move {
        let result = completion_rx.await.unwrap_or_else(|_| {
            Err(AgentWrapperError::Backend {
                message: "completion channel dropped".to_string(),
            })
        });

        let _ = events_done_rx.await;
        result
    });

    AgentWrapperRunHandle { events, completion }
}

struct FinalityEventStream {
    rx: mpsc::Receiver<AgentWrapperEvent>,
    events_done_tx: Option<oneshot::Sender<()>>,
}

impl FinalityEventStream {
    fn signal_done(&mut self) {
        if let Some(tx) = self.events_done_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Stream for FinalityEventStream {
    type Item = AgentWrapperEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let poll = Pin::new(&mut self.rx).poll_recv(cx);
        if let Poll::Ready(None) = poll {
            self.signal_done();
        }
        poll
    }
}

impl Drop for FinalityEventStream {
    fn drop(&mut self) {
        self.signal_done();
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures_core::Stream;
    use futures_util::task::noop_waker;
    use tokio::sync::{mpsc, oneshot};

    use crate::{
        AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
        AgentWrapperKind,
    };

    fn success_exit_status() -> std::process::ExitStatus {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(0)
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(0)
        }
    }

    fn block_on_ready<T>(mut future: Pin<Box<dyn Future<Output = T>>>) -> T {
        let waker = noop_waker();
        let mut context = Context::from_waker(&waker);

        for _ in 0..64 {
            if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
                return output;
            }
            std::thread::yield_now();
        }

        panic!("future did not resolve quickly (expected Ready)");
    }

    fn drain_to_none(mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>) {
        let waker = noop_waker();
        let mut context = Context::from_waker(&waker);

        loop {
            match stream.as_mut().poll_next(&mut context) {
                Poll::Ready(Some(_)) => continue,
                Poll::Ready(None) => break,
                Poll::Pending => {
                    std::thread::yield_now();
                }
            }
        }
    }

    #[test]
    fn completion_is_pending_until_events_stream_is_drained_to_none() {
        let (tx, rx) = mpsc::channel::<AgentWrapperEvent>(32);
        tx.try_send(AgentWrapperEvent {
            agent_kind: AgentWrapperKind::new("dummy").unwrap(),
            kind: AgentWrapperEventKind::Status,
            channel: None,
            text: None,
            message: Some("hello".to_string()),
            data: None,
        })
        .unwrap();
        drop(tx);

        let (completion_tx, completion_rx) =
            oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();
        completion_tx
            .send(Ok(AgentWrapperCompletion {
                status: success_exit_status(),
                final_text: None,
                data: None,
            }))
            .unwrap();

        let mut handle = super::build_gated_run_handle(rx, completion_rx);

        {
            let waker = noop_waker();
            let mut context = Context::from_waker(&waker);
            assert!(matches!(
                handle.completion.as_mut().poll(&mut context),
                Poll::Pending
            ));
        }

        drain_to_none(handle.events.as_mut());

        let completion_result = block_on_ready(handle.completion);
        assert!(completion_result.is_ok());
    }

    #[test]
    fn dropping_events_stream_unblocks_completion() {
        let (tx, rx) = mpsc::channel::<AgentWrapperEvent>(32);
        tx.try_send(AgentWrapperEvent {
            agent_kind: AgentWrapperKind::new("dummy").unwrap(),
            kind: AgentWrapperEventKind::Status,
            channel: None,
            text: None,
            message: Some("hello".to_string()),
            data: None,
        })
        .unwrap();

        let (completion_tx, completion_rx) =
            oneshot::channel::<Result<AgentWrapperCompletion, AgentWrapperError>>();
        completion_tx
            .send(Ok(AgentWrapperCompletion {
                status: success_exit_status(),
                final_text: None,
                data: None,
            }))
            .unwrap();

        let handle = super::build_gated_run_handle(rx, completion_rx);
        let crate::AgentWrapperRunHandle { events, completion } = handle;

        {
            let waker = noop_waker();
            let mut context = Context::from_waker(&waker);
            let mut completion = completion;
            assert!(matches!(
                completion.as_mut().poll(&mut context),
                Poll::Pending
            ));

            drop(events);

            let completion_result = block_on_ready(completion);
            assert!(completion_result.is_ok());
        }

        drop(tx);
    }
}
