#![forbid(unsafe_code)]
#![allow(unused_crate_dependencies)]

#[cfg(test)]
use tempfile as _;

use camino::Utf8PathBuf;

pub mod adjudication;
pub mod cli;
pub mod input;
pub mod live_input;
pub mod live_runtime;
pub mod operator_sink;
pub mod operator_surface;
pub mod scheduler;

pub use adjudication::{
    AdjudicationConfig, AdjudicationFailure, AdjudicationRequest, AdjudicationResponse,
    ReasoningEffort,
};
pub use input::{CheckpointCursor, InputError, ReplayCheckpointBundle};
pub use live_input::{
    load_live_fixture, validate_live_event_sequence, verify_live_checkpoint_compatibility,
    FixtureLiveCheckpointSource, LiveCheckpointCompatibility, LiveCheckpointEvent,
    LiveCheckpointSource, LiveInputError,
};
pub use live_runtime::{LiveObservation, LiveRuntime, LiveRuntimeError, LiveRuntimeSnapshot};
pub use operator_sink::{
    build_operator_events, emit_operator_events, HeartbeatEvent, OperatorEvent, OperatorSink,
    RecordingOperatorSink, SilentCheckpointEvent, StatusEvent, VisibleWarningEvent,
};
pub use operator_surface::{
    CheckpointPresentation, ReplayReport, WarningDisposition, WarningPolicy,
};
pub use scheduler::{
    DecisionReason, EvaluationDecision, SchedulerPolicy, SchedulerState, TriggerClass,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentinelMode {
    Replay,
    Live,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelRequest {
    pub checkpoint_dir: Utf8PathBuf,
    pub mode: SentinelMode,
    pub cursor: Option<CheckpointCursor>,
    pub scheduler_policy: SchedulerPolicy,
    pub warning_policy: WarningPolicy,
    pub adjudication: AdjudicationConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelResult {
    pub bundle: ReplayCheckpointBundle,
    pub report: ReplayReport,
    pub adjudication_requests: Vec<AdjudicationRequest>,
}

#[derive(Debug, thiserror::Error)]
pub enum SentinelError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error("live mode remains gated by S10 until replay usefulness is reviewed and approved")]
    LiveModeDeferred,
}

pub fn run() -> anyhow::Result<()> {
    cli::run()
}

pub fn execute(request: &SentinelRequest) -> Result<SentinelResult, SentinelError> {
    if matches!(request.mode, SentinelMode::Live) {
        return Err(SentinelError::LiveModeDeferred);
    }

    let bundle = input::load_replay_bundle(&request.checkpoint_dir)?;
    let checkpoints = bundle.checkpoints_after(request.cursor.as_ref());
    let report = operator_surface::render_replay_report(
        &bundle,
        &checkpoints,
        &request.scheduler_policy,
        &request.warning_policy,
    );
    let adjudication_requests = report
        .visible_warnings
        .iter()
        .filter_map(|presentation| adjudication::shape_request(presentation, &request.adjudication))
        .collect();

    Ok(SentinelResult {
        bundle,
        report,
        adjudication_requests,
    })
}
