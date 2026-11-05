//! Additive observation on the existing cancellation handle; no runtime dependency when disabled.
use crate::{AgentWrapperCancelHandle, AgentWrapperError};
use std::{
    future::Future,
    pin::Pin,
    sync::{atomic::AtomicBool, Arc},
};

/// Checked settlement of the backend's actual child and stdio owners.
/// This is independent of the pinned cancellation completion value.
pub type DynAgentWrapperSettlement =
    Pin<Box<dyn Future<Output = Result<(), AgentWrapperError>> + Send + 'static>>;

/// Copied actual child wait data. Missing values are explicitly unknown and
/// supply neither process-absence proof nor authority. Primary and recovery
/// statuses remain separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentWrapperProcessTerminationObservation {
    pub process_id: Option<u32>,
    pub status: Option<std::process::ExitStatus>,
    pub recovery_status: Option<std::process::ExitStatus>,
}

/// The existing unit settlement outcome plus its copied actual wait data.
/// `None` observations means the owner did not publish the snapshot. An empty
/// list is not proof that no process ever existed. Consumers still need exact
/// run/source custody and their independent terminal/cleanup checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentWrapperProcessSettlementObservation {
    pub acknowledged: bool,
    pub observations: Option<Vec<AgentWrapperProcessTerminationObservation>>,
    /// Positive copied admission closeout before any spawn attempt. `None` is
    /// unsupported/missing; `Some(false)` does not prove a child existed. Even
    /// `Some(true)` needs checked settlement and exact enclosing custody.
    pub closed_before_any_spawn: Option<bool>,
}

/// Observation only: polling does not cancel, seal admission or launch.
pub type DynAgentWrapperProcessSettlement =
    Pin<Box<dyn Future<Output = AgentWrapperProcessSettlementObservation> + Send + 'static>>;

/// Nonterminal request to revoke an enclosing per-run route. Resolving this
/// observation does not acknowledge child/stdio settlement or report an outcome.
pub type DynAgentWrapperDisposalRequested = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub(super) struct AgentWrapperCancelInner {
    pub(super) called: AtomicBool,
    pub(super) cancel: Box<dyn Fn() + Send + Sync + 'static>,
    pub(super) settlement: Option<Arc<dyn Fn() -> DynAgentWrapperSettlement + Send + Sync>>,
    pub(super) disposal: Option<Arc<dyn Fn() -> DynAgentWrapperDisposalRequested + Send + Sync>>,
    pub(super) process_settlement:
        Option<Arc<dyn Fn() -> DynAgentWrapperProcessSettlement + Send + Sync>>,
}

impl AgentWrapperCancelHandle {
    #[allow(dead_code)]
    pub(crate) fn new_with_settlement(
        cancel: impl Fn() + Send + Sync + 'static,
        settlement: Arc<dyn Fn() -> DynAgentWrapperSettlement + Send + Sync>,
        disposal: Arc<dyn Fn() -> DynAgentWrapperDisposalRequested + Send + Sync>,
    ) -> Self {
        Self::new_with_settlement_and_process_observation(cancel, settlement, disposal, None)
    }

    pub(crate) fn new_with_settlement_and_process_observation(
        cancel: impl Fn() + Send + Sync + 'static,
        settlement: Arc<dyn Fn() -> DynAgentWrapperSettlement + Send + Sync>,
        disposal: Arc<dyn Fn() -> DynAgentWrapperDisposalRequested + Send + Sync>,
        process_settlement: Option<Arc<dyn Fn() -> DynAgentWrapperProcessSettlement + Send + Sync>>,
    ) -> Self {
        Self {
            inner: Arc::new(AgentWrapperCancelInner {
                called: AtomicBool::new(false),
                cancel: Box::new(cancel),
                settlement: Some(settlement),
                disposal: Some(disposal),
                process_settlement,
            }),
        }
    }

    /// Observe the copied actual child wait results after the existing backend
    /// settlement publisher finishes. This future retains data/watch state only,
    /// not native custody or spawn-admission callbacks. Unsupported observation
    /// is `None`; failed/missing publication remains explicit in the result.
    /// The raw completion and `settlement()` contract are unchanged.
    /// ```no_run
    /// # async fn inspect(cancel: &agent_api::AgentWrapperCancelHandle) {
    /// if let Some(observation) = cancel.process_settlement() {
    ///     let actual_waits = observation.await;
    ///     // Check settlement(), exact run custody and the enclosing cleanup independently.
    ///     let _ = actual_waits;
    /// }
    /// # }
    /// ```
    pub fn process_settlement(&self) -> Option<DynAgentWrapperProcessSettlement> {
        self.inner
            .process_settlement
            .as_ref()
            .map(|factory| factory())
    }

    /// Returns a retained checked child/stdio settlement observation when supported by this run.
    /// `None` means no checked acknowledgment is available; it MUST NOT be treated as successful
    /// cleanup. Observing does not cancel the run. An enclosing terminal publisher requiring
    /// custody MUST await and check this result, including after a `"cancelled"` completion.
    ///
    /// ```no_run
    /// # async fn terminal(cancel: &agent_api::AgentWrapperCancelHandle) -> Result<(), Box<dyn std::error::Error>> {
    /// let settlement = cancel.settlement().ok_or("checked settlement unavailable")?;
    /// cancel.cancel();
    /// settlement.await?;
    /// # Ok(()) }
    /// ```
    pub fn settlement(&self) -> Option<DynAgentWrapperSettlement> {
        self.inner.settlement.as_ref().map(|factory| factory())
    }
}

impl AgentWrapperCancelHandle {
    /// Observes the request to dispose this run's enclosing route independently
    /// of completion and checked settlement. Cancellation, raw startup failure,
    /// raw completion and abandoned custody publish this latched request before
    /// waiting for settlement. Observation itself does not cancel, seal native
    /// admission, grant authority or acknowledge child/stdio cleanup.
    ///
    /// `None` means this run does not support the observation. It is never an ACK.
    ///
    /// ```no_run
    /// # async fn dispose(cancel: &agent_api::AgentWrapperCancelHandle) {
    /// if let Some(requested) = cancel.disposal_requested() {
    ///     requested.await;
    ///     // Revoke the enclosing route; separately await/check settlement().
    /// }
    /// # }
    /// ```
    pub fn disposal_requested(&self) -> Option<DynAgentWrapperDisposalRequested> {
        self.inner.disposal.as_ref().map(|factory| factory())
    }
}
