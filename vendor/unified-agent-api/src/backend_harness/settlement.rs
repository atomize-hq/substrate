//! Retained acknowledgment for the concrete SDK child owners, independent of cancellation value.
use crate::{AgentWrapperError, DynAgentWrapperDisposalRequested, DynAgentWrapperSettlement};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::Notify;

pub(crate) type SettlementFactory = Arc<dyn Fn() -> DynAgentWrapperSettlement + Send + Sync>;
#[derive(Clone)]
pub(crate) struct BackendAttemptCustody {
    pub(crate) request_termination: Arc<dyn Fn() + Send + Sync>,
    pub(crate) settle: SettlementFactory,
}

#[derive(Clone, Default)]
pub(crate) struct SettlementObservation {
    inner: Arc<ObservationInner>,
}
#[derive(Default)]
struct ObservationInner {
    result: Mutex<Option<Result<(), String>>>,
    changed: Notify,
    disposal_requested: AtomicBool,
    disposal_changed: Notify,
}
impl SettlementObservation {
    pub(crate) fn request_disposal(&self) {
        self.inner.disposal_requested.store(true, Ordering::Release);
        self.inner.disposal_changed.notify_waiters();
    }
    pub(crate) fn disposal_factory(
        &self,
    ) -> Arc<dyn Fn() -> DynAgentWrapperDisposalRequested + Send + Sync> {
        let observation = self.clone();
        Arc::new(move || {
            let observation = observation.clone();
            Box::pin(async move {
                loop {
                    let changed = observation.inner.disposal_changed.notified();
                    tokio::pin!(changed);
                    changed.as_mut().enable();
                    if observation.inner.disposal_requested.load(Ordering::Acquire) {
                        return;
                    }
                    changed.await;
                }
            })
        })
    }

    pub(crate) fn factory(&self) -> SettlementFactory {
        let observation = self.clone();
        Arc::new(move || {
            let observation = observation.clone();
            Box::pin(async move { observation.wait().await })
        })
    }
    pub(crate) fn publisher(&self) -> SettlementPublisher {
        SettlementPublisher(Some(self.clone()))
    }
    fn publish(&self, result: Result<(), String>) {
        let mut slot = self.inner.result.lock().unwrap_or_else(|e| e.into_inner());
        if slot.is_none() {
            *slot = Some(result);
        }
        drop(slot);
        self.inner.changed.notify_waiters();
    }
    async fn wait(&self) -> Result<(), AgentWrapperError> {
        loop {
            let changed = self.inner.changed.notified();
            if let Some(result) = self
                .inner
                .result
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone()
            {
                return result.map_err(|message| AgentWrapperError::Backend { message });
            }
            changed.await;
        }
    }
}
pub(crate) struct SettlementPublisher(Option<SettlementObservation>);
impl SettlementPublisher {
    pub(crate) fn request_disposal(&self) {
        if let Some(observation) = &self.0 {
            observation.request_disposal();
        }
    }

    pub(crate) fn finish(mut self, result: &Result<(), AgentWrapperError>) {
        if let Some(observation) = self.0.take() {
            observation.publish(
                result
                    .as_ref()
                    .map(|_| ())
                    .map_err(|_| "backend process settlement failed".to_string()),
            );
        }
    }
}
impl Drop for SettlementPublisher {
    fn drop(&mut self) {
        if let Some(observation) = self.0.take() {
            observation.request_disposal();
            observation.publish(Err(
                "backend settlement owner ended without acknowledgment".to_string()
            ));
        }
    }
}

pub(crate) async fn settle_attempt(
    custody: Option<&BackendAttemptCustody>,
) -> Result<(), AgentWrapperError> {
    match custody {
        Some(custody) => (custody.settle)().await,
        None => Ok(()),
    }
}
