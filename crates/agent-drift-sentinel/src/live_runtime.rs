use agent_drift_analyzer::Checkpoint;

use crate::input::CheckpointCursor;
use crate::live_input::{
    verify_live_checkpoint_compatibility, LiveCheckpointCompatibility, LiveCheckpointEvent,
    LiveCheckpointSource, LiveInputError,
};
use crate::operator_surface::{present_checkpoint, CheckpointPresentation, WarningPolicy};
use crate::scheduler::{
    EvaluationDecision, ReplayScheduler, SchedulerPolicy, SchedulerState, TriggerClass,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRuntimeSnapshot {
    pub latest_cursor: Option<CheckpointCursor>,
    pub latest_checkpoint_id: Option<String>,
    pub last_trigger: Option<TriggerClass>,
    pub processed_events: usize,
    pub scheduler_state: SchedulerState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveObservation {
    pub event: LiveCheckpointEvent,
    pub compatibility: LiveCheckpointCompatibility,
    pub decision: EvaluationDecision,
    pub presentation: CheckpointPresentation,
    pub snapshot: LiveRuntimeSnapshot,
}

#[derive(Debug, thiserror::Error)]
pub enum LiveRuntimeError {
    #[error(transparent)]
    Input(#[from] LiveInputError),
    #[error(
        "live {trigger} event referenced cursor {actual_session_id}:{actual_ordinal} before the runtime observed a checkpoint"
    )]
    MissingCheckpoint {
        trigger: &'static str,
        actual_session_id: String,
        actual_ordinal: usize,
    },
    #[error(
        "live {trigger} event referenced cursor {actual_session_id}:{actual_ordinal}, expected latest checkpoint cursor {expected_session_id}:{expected_ordinal}"
    )]
    CursorMismatch {
        trigger: &'static str,
        expected_session_id: String,
        expected_ordinal: usize,
        actual_session_id: String,
        actual_ordinal: usize,
    },
}

#[derive(Debug, Clone)]
pub struct LiveRuntime {
    scheduler: ReplayScheduler,
    warning_policy: WarningPolicy,
    latest_checkpoint: Option<Checkpoint>,
    latest_compatibility: Option<LiveCheckpointCompatibility>,
    processed_events: usize,
    last_trigger: Option<TriggerClass>,
}

impl LiveRuntime {
    pub fn new(policy: SchedulerPolicy, warning_policy: WarningPolicy) -> Self {
        Self {
            scheduler: ReplayScheduler::new(policy),
            warning_policy,
            latest_checkpoint: None,
            latest_compatibility: None,
            processed_events: 0,
            last_trigger: None,
        }
    }

    pub fn snapshot(&self) -> LiveRuntimeSnapshot {
        LiveRuntimeSnapshot {
            latest_cursor: self
                .latest_compatibility
                .as_ref()
                .map(|value| value.cursor.clone()),
            latest_checkpoint_id: self
                .latest_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_id.clone()),
            last_trigger: self.last_trigger,
            processed_events: self.processed_events,
            scheduler_state: self.scheduler.state().clone(),
        }
    }

    pub fn observe(
        &mut self,
        event: LiveCheckpointEvent,
    ) -> Result<LiveObservation, LiveRuntimeError> {
        let (checkpoint, compatibility) = if let Some(checkpoint) = event.checkpoint.as_ref() {
            let compatibility = verify_live_checkpoint_compatibility(checkpoint)?;
            if compatibility.cursor != event.cursor {
                return Err(LiveRuntimeError::CursorMismatch {
                    trigger: trigger_name(event.trigger),
                    expected_session_id: compatibility.cursor.session_id,
                    expected_ordinal: compatibility.cursor.ordinal,
                    actual_session_id: event.cursor.session_id.clone(),
                    actual_ordinal: event.cursor.ordinal,
                });
            }
            self.latest_checkpoint = Some(checkpoint.clone());
            self.latest_compatibility = Some(compatibility.clone());
            (checkpoint.clone(), compatibility)
        } else {
            let checkpoint = self.latest_checkpoint.as_ref().cloned().ok_or_else(|| {
                LiveRuntimeError::MissingCheckpoint {
                    trigger: trigger_name(event.trigger),
                    actual_session_id: event.cursor.session_id.clone(),
                    actual_ordinal: event.cursor.ordinal,
                }
            })?;
            let compatibility = self
                .latest_compatibility
                .as_ref()
                .cloned()
                .expect("latest compatibility tracks latest checkpoint");
            if compatibility.cursor != event.cursor {
                return Err(LiveRuntimeError::CursorMismatch {
                    trigger: trigger_name(event.trigger),
                    expected_session_id: compatibility.cursor.session_id,
                    expected_ordinal: compatibility.cursor.ordinal,
                    actual_session_id: event.cursor.session_id.clone(),
                    actual_ordinal: event.cursor.ordinal,
                });
            }
            (checkpoint, compatibility)
        };

        let decision = self.scheduler.observe(
            event.cursor.clone(),
            event.trigger,
            checkpoint.flagged,
            Some(&compatibility.warning_fingerprint),
        );
        let presentation =
            present_checkpoint(&checkpoint, event.trigger, &decision, &self.warning_policy);

        self.processed_events += 1;
        self.last_trigger = Some(event.trigger);

        Ok(LiveObservation {
            event,
            compatibility,
            decision,
            presentation,
            snapshot: self.snapshot(),
        })
    }

    pub fn drain<S>(&mut self, source: &mut S) -> Result<Vec<LiveObservation>, LiveRuntimeError>
    where
        S: LiveCheckpointSource,
    {
        let mut observations = Vec::new();
        while let Some(event) = source.next_event()? {
            observations.push(self.observe(event)?);
        }
        Ok(observations)
    }
}

fn trigger_name(trigger: TriggerClass) -> &'static str {
    match trigger {
        TriggerClass::CheckpointReady => "checkpoint_ready",
        TriggerClass::Heartbeat => "heartbeat",
        TriggerClass::RepeatedFailure => "repeated_failure",
        TriggerClass::ManualReview => "manual_review",
    }
}
