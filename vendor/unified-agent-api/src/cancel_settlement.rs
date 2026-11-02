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

/// Nonterminal request to revoke an enclosing per-run route. Resolving this
/// observation does not acknowledge child/stdio settlement or report an outcome.
pub type DynAgentWrapperDisposalRequested = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub(super) struct AgentWrapperCancelInner {
    pub(super) called: AtomicBool,
    pub(super) cancel: Box<dyn Fn() + Send + Sync + 'static>,
    pub(super) settlement: Option<Arc<dyn Fn() -> DynAgentWrapperSettlement + Send + Sync>>,
    pub(super) disposal: Option<Arc<dyn Fn() -> DynAgentWrapperDisposalRequested + Send + Sync>>,
}

impl AgentWrapperCancelHandle {
    #[allow(dead_code)]
    pub(crate) fn new_with_settlement(
        cancel: impl Fn() + Send + Sync + 'static,
        settlement: Arc<dyn Fn() -> DynAgentWrapperSettlement + Send + Sync>,
        disposal: Arc<dyn Fn() -> DynAgentWrapperDisposalRequested + Send + Sync>,
    ) -> Self {
        Self {
            inner: Arc::new(AgentWrapperCancelInner {
                called: AtomicBool::new(false),
                cancel: Box::new(cancel),
                settlement: Some(settlement),
                disposal: Some(disposal),
            }),
        }
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
