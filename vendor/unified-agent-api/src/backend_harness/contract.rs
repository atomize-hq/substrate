use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use futures_core::Stream;
use tokio::sync::Notify;

use crate::{
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperKind,
    AgentWrapperRunRequest,
};

pub(crate) type DynBackendEventStream<E, BE> =
    Pin<Box<dyn Stream<Item = Result<E, BE>> + Send + 'static>>;

pub(crate) type DynBackendCompletionFuture<C, BE> =
    Pin<Box<dyn Future<Output = Result<C, BE>> + Send + 'static>>;

#[derive(Clone, Debug, Default)]
pub(crate) struct EventObservabilitySignal {
    inner: Arc<EventObservabilitySignalInner>,
}

#[derive(Debug, Default)]
struct EventObservabilitySignalInner {
    done: AtomicBool,
    notify: Notify,
}

impl EventObservabilitySignal {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn signal(&self) {
        if self.inner.done.swap(true, Ordering::SeqCst) {
            return;
        }

        self.inner.notify.notify_waiters();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BackendHarnessErrorPhase {
    Spawn,
    Stream,
    Completion,
}

pub(crate) struct BackendSpawn<E, C, BE> {
    pub events: DynBackendEventStream<E, BE>,
    pub completion: DynBackendCompletionFuture<C, BE>,
    pub events_observability: Option<EventObservabilitySignal>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BackendDefaults {
    pub env: BTreeMap<String, String>,
    pub default_timeout: Option<Duration>,
}

pub(crate) struct NormalizedRequest<P> {
    /// Stable identity for error reporting and event stamping.
    pub agent_kind: AgentWrapperKind,

    /// Preserved from `AgentWrapperRunRequest` (must be non-empty after trimming).
    pub prompt: String,

    /// Typed handoff for the shared model-selection field after normalization.
    pub model_id: Option<String>,

    /// Preserved from `AgentWrapperRunRequest` (no harness defaulting in v1).
    pub working_dir: Option<std::path::PathBuf>,

    /// Derived per BH-C03. `Some(Duration::ZERO)` is an explicit “no timeout” request.
    pub effective_timeout: Option<Duration>,

    /// Derived per BH-C03: `defaults.env` overridden by `request.env`.
    pub env: BTreeMap<String, String>,

    /// Backend-owned extracted policy derived from `request.extensions` after the allowlist check.
    pub policy: P,
}

pub(crate) trait BackendHarnessAdapter: Send + Sync + 'static {
    /// MUST return a stable, lower_snake_case id (see `AgentWrapperKind` rules).
    fn kind(&self) -> AgentWrapperKind;

    /// Supported extension keys for this backend (exact string match; case-sensitive).
    ///
    /// This list MUST include both:
    /// - core keys under `agent_api.*` that the backend supports, and
    /// - backend keys under `backend.<agent_kind>.*` owned by the backend.
    fn supported_extension_keys(&self) -> &'static [&'static str];

    /// Backend-owned policy extracted from known extension keys only.
    ///
    /// This hook MUST NOT implement “unknown key” rejection (that is BH-C02, harness-owned).
    type Policy: Send + 'static;

    fn validate_and_extract_policy(
        &self,
        request: &AgentWrapperRunRequest,
    ) -> Result<Self::Policy, AgentWrapperError>;

    /// Typed backend event and completion types emitted by the wrapper runtime.
    type BackendEvent: Send + 'static;
    type BackendCompletion: Send + 'static;

    /// Backend error type used at spawn/stream/completion boundaries.
    type BackendError: Send + Sync + 'static;

    /// Acquires actual SDK attempt custody before startup is constructed or polled.
    /// The default supplies no acknowledgment; unrelated backend ownership is unchanged.
    fn attempt_custody(
        &self,
        _req: &NormalizedRequest<Self::Policy>,
    ) -> Option<super::BackendAttemptCustody> {
        None
    }

    /// Spawns the backend run using only the normalized request.
    ///
    /// The returned stream MUST be drained to completion by the harness pump (BH-C04).
    #[allow(clippy::type_complexity)]
    fn spawn(
        &self,
        req: NormalizedRequest<Self::Policy>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        BackendSpawn<
                            Self::BackendEvent,
                            Self::BackendCompletion,
                            Self::BackendError,
                        >,
                        Self::BackendError,
                    >,
                > + Send
                + 'static,
        >,
    >;

    /// Maps one typed backend event into 0..N universal events.
    ///
    /// Mapping is **infallible** by contract: backends MUST convert parse errors into
    /// `BackendError` at the stream boundary, not here.
    fn map_event(&self, event: Self::BackendEvent) -> Vec<AgentWrapperEvent>;

    /// Maps a typed backend completion value to the universal completion payload.
    fn map_completion(
        &self,
        completion: Self::BackendCompletion,
    ) -> Result<AgentWrapperCompletion, AgentWrapperError>;

    /// Produces a safe/redacted message for a backend error at a given phase.
    ///
    /// This message MUST NOT contain raw backend stdout/stderr lines or raw JSONL lines.
    /// It MAY include bounded metadata such as `line_bytes=<n>` or a stable error kind tag.
    fn redact_error(&self, phase: BackendHarnessErrorPhase, err: &Self::BackendError) -> String;
}

#[cfg(test)]
mod tests;
