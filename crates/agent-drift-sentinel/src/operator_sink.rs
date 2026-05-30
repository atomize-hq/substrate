use anyhow::Result;

use crate::input::CheckpointCursor;
use crate::live_runtime::LiveObservation;
use crate::operator_surface::{CheckpointPresentation, WarningDisposition};
use crate::scheduler::{DecisionReason, TriggerClass};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorEvent {
    VisibleWarning(VisibleWarningEvent),
    SilentCheckpoint(SilentCheckpointEvent),
    Heartbeat(HeartbeatEvent),
    Status(StatusEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleWarningEvent {
    pub cursor: CheckpointCursor,
    pub source_label: Option<String>,
    pub presentation: CheckpointPresentation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SilentCheckpointEvent {
    pub cursor: CheckpointCursor,
    pub source_label: Option<String>,
    pub checkpoint_id: String,
    pub trigger: TriggerClass,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeartbeatEvent {
    pub cursor: CheckpointCursor,
    pub source_label: Option<String>,
    pub evaluated: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEvent {
    pub cursor: CheckpointCursor,
    pub source_label: Option<String>,
    pub trigger: TriggerClass,
    pub message: String,
}

pub trait OperatorSink {
    fn emit(&mut self, event: OperatorEvent) -> Result<()>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RecordingOperatorSink {
    events: Vec<OperatorEvent>,
}

impl RecordingOperatorSink {
    pub fn events(&self) -> &[OperatorEvent] {
        &self.events
    }

    pub fn into_events(self) -> Vec<OperatorEvent> {
        self.events
    }
}

impl OperatorSink for RecordingOperatorSink {
    fn emit(&mut self, event: OperatorEvent) -> Result<()> {
        self.events.push(event);
        Ok(())
    }
}

pub fn build_operator_events(observation: &LiveObservation) -> Vec<OperatorEvent> {
    let mut events = Vec::new();
    let cursor = observation.event.cursor.clone();
    let source_label = observation.event.source_label.clone();

    match &observation.presentation.disposition {
        WarningDisposition::Visible => {
            events.push(OperatorEvent::VisibleWarning(VisibleWarningEvent {
                cursor: cursor.clone(),
                source_label: source_label.clone(),
                presentation: observation.presentation.clone(),
            }));
        }
        WarningDisposition::Silent { reason }
            if matches!(observation.event.trigger, TriggerClass::CheckpointReady) =>
        {
            events.push(OperatorEvent::SilentCheckpoint(SilentCheckpointEvent {
                cursor: cursor.clone(),
                source_label: source_label.clone(),
                checkpoint_id: observation.presentation.checkpoint.checkpoint_id.clone(),
                trigger: observation.event.trigger,
                reason: reason.clone(),
            }));
        }
        WarningDisposition::Silent { .. } => {}
    }

    match observation.event.trigger {
        TriggerClass::Heartbeat => {
            events.push(OperatorEvent::Heartbeat(HeartbeatEvent {
                cursor,
                source_label,
                evaluated: observation.decision.evaluate,
                message: heartbeat_message(observation),
            }));
        }
        TriggerClass::ManualReview | TriggerClass::RepeatedFailure => {
            events.push(OperatorEvent::Status(StatusEvent {
                cursor,
                source_label,
                trigger: observation.event.trigger,
                message: status_message(observation),
            }));
        }
        TriggerClass::CheckpointReady => {}
    }

    events
}

pub fn emit_operator_events<S>(
    sink: &mut S,
    observation: &LiveObservation,
) -> Result<Vec<OperatorEvent>>
where
    S: OperatorSink,
{
    let events = build_operator_events(observation);
    for event in events.iter().cloned() {
        sink.emit(event)?;
    }
    Ok(events)
}

fn heartbeat_message(observation: &LiveObservation) -> String {
    if observation.decision.evaluate {
        format!(
            "heartbeat evaluated {}",
            observation.presentation.checkpoint.checkpoint_id
        )
    } else {
        format!(
            "heartbeat kept {} under review ({})",
            observation.presentation.checkpoint.checkpoint_id,
            decision_reason_name(&observation.decision.reason)
        )
    }
}

fn status_message(observation: &LiveObservation) -> String {
    let trigger = trigger_name(observation.event.trigger);
    let checkpoint_id = &observation.presentation.checkpoint.checkpoint_id;

    match &observation.presentation.disposition {
        WarningDisposition::Visible => {
            format!("{trigger} surfaced a visible warning for {checkpoint_id}")
        }
        WarningDisposition::Silent { reason } => {
            format!("{trigger} completed without a visible warning for {checkpoint_id}: {reason}")
        }
    }
}

fn decision_reason_name(reason: &DecisionReason) -> &'static str {
    match reason {
        DecisionReason::InitialCheckpoint => "initial_checkpoint",
        DecisionReason::CooldownSatisfied => "cooldown_satisfied",
        DecisionReason::HeartbeatIntervalReached => "heartbeat_interval_reached",
        DecisionReason::RepeatedFailureFastPath => "repeated_failure_fast_path",
        DecisionReason::ManualReview => "manual_review",
        DecisionReason::CooldownDeferred => "cooldown_deferred",
        DecisionReason::WarningDebounced => "warning_debounced",
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
