//! Retained acknowledgment for the concrete SDK child owners, independent of cancellation value.
use crate::{
    AgentWrapperError, AgentWrapperProcessSettlementObservation,
    AgentWrapperProcessTerminationObservation, DynAgentWrapperDisposalRequested,
    DynAgentWrapperProcessSettlement, DynAgentWrapperSettlement,
};
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
    // Temporary driver-owned getter. Never captured by the retained public handle.
    pub(crate) observe_processes:
        Option<Arc<dyn Fn() -> Vec<AgentWrapperProcessTerminationObservation> + Send + Sync>>,
    pub(crate) observe_closed_before_any_spawn: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
}

#[derive(Clone, Default)]
pub(crate) struct SettlementObservation {
    inner: Arc<ObservationInner>,
}
#[derive(Default)]
struct ObservationInner {
    result: Mutex<Option<PublishedSettlement>>,
    changed: Notify,
    disposal_requested: AtomicBool,
    disposal_changed: Notify,
}
#[derive(Clone)]
struct PublishedSettlement {
    result: Result<(), String>,
    observations: Option<Vec<AgentWrapperProcessTerminationObservation>>,
    closed_before_any_spawn: Option<bool>,
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
    pub(crate) fn process_factory(
        &self,
    ) -> Arc<dyn Fn() -> DynAgentWrapperProcessSettlement + Send + Sync> {
        let observation = self.clone();
        Arc::new(move || {
            let observation = observation.clone();
            Box::pin(async move {
                let published = observation.wait_published().await;
                AgentWrapperProcessSettlementObservation {
                    acknowledged: published.result.is_ok(),
                    observations: published.observations,
                    closed_before_any_spawn: published.closed_before_any_spawn,
                }
            })
        })
    }
    pub(crate) fn publisher(&self) -> SettlementPublisher {
        SettlementPublisher(Some(self.clone()))
    }
    fn publish(
        &self,
        result: Result<(), String>,
        observations: Option<Vec<AgentWrapperProcessTerminationObservation>>,
        closed_before_any_spawn: Option<bool>,
    ) {
        let mut slot = self.inner.result.lock().unwrap_or_else(|e| e.into_inner());
        if slot.is_none() {
            *slot = Some(PublishedSettlement {
                result,
                observations,
                closed_before_any_spawn,
            });
        }
        drop(slot);
        self.inner.changed.notify_waiters();
    }
    async fn wait(&self) -> Result<(), AgentWrapperError> {
        self.wait_published()
            .await
            .result
            .map_err(|message| AgentWrapperError::Backend { message })
    }
    async fn wait_published(&self) -> PublishedSettlement {
        loop {
            let changed = self.inner.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if let Some(result) = self
                .inner
                .result
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .clone()
            {
                return result;
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

    pub(crate) fn finish(self, result: &Result<(), AgentWrapperError>) {
        self.finish_with_process_observations(result, None, None)
    }
    pub(crate) fn finish_with_process_observations(
        mut self,
        result: &Result<(), AgentWrapperError>,
        observations: Option<Vec<AgentWrapperProcessTerminationObservation>>,
        closed_before_any_spawn: Option<bool>,
    ) {
        if let Some(observation) = self.0.take() {
            observation.publish(
                result
                    .as_ref()
                    .map(|_| ())
                    .map_err(|_| "backend process settlement failed".to_string()),
                observations,
                closed_before_any_spawn,
            );
        }
    }
}
impl Drop for SettlementPublisher {
    fn drop(&mut self) {
        if let Some(observation) = self.0.take() {
            observation.request_disposal();
            observation.publish(
                Err("backend settlement owner ended without acknowledgment".to_string()),
                None,
                None,
            );
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
