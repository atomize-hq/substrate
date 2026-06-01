use std::fs;
use std::io::{BufRead, BufReader};

use agent_drift_analyzer::Checkpoint;
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use crate::input::CheckpointCursor;
use crate::operator_surface::warning_fingerprint;
use crate::scheduler::TriggerClass;

const SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA: &str = "v0.2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveCheckpointEvent {
    pub emission_ordinal: usize,
    pub cursor: CheckpointCursor,
    pub trigger: TriggerClass,
    pub checkpoint: Option<Checkpoint>,
    pub source_label: Option<String>,
}

impl LiveCheckpointEvent {
    pub fn checkpoint_ready(
        emission_ordinal: usize,
        checkpoint: Checkpoint,
        source_label: Option<String>,
    ) -> Self {
        let cursor = CheckpointCursor::from(&checkpoint);
        Self {
            emission_ordinal,
            cursor,
            trigger: TriggerClass::CheckpointReady,
            checkpoint: Some(checkpoint),
            source_label,
        }
    }

    pub fn heartbeat(
        emission_ordinal: usize,
        cursor: CheckpointCursor,
        source_label: Option<String>,
    ) -> Self {
        Self::synthetic(
            emission_ordinal,
            cursor,
            TriggerClass::Heartbeat,
            source_label,
        )
    }

    pub fn repeated_failure(
        emission_ordinal: usize,
        cursor: CheckpointCursor,
        source_label: Option<String>,
    ) -> Self {
        Self::synthetic(
            emission_ordinal,
            cursor,
            TriggerClass::RepeatedFailure,
            source_label,
        )
    }

    pub fn manual_review(
        emission_ordinal: usize,
        cursor: CheckpointCursor,
        source_label: Option<String>,
    ) -> Self {
        Self::synthetic(
            emission_ordinal,
            cursor,
            TriggerClass::ManualReview,
            source_label,
        )
    }

    fn synthetic(
        emission_ordinal: usize,
        cursor: CheckpointCursor,
        trigger: TriggerClass,
        source_label: Option<String>,
    ) -> Self {
        Self {
            emission_ordinal,
            cursor,
            trigger,
            checkpoint: None,
            source_label,
        }
    }

    fn trigger_name(&self) -> &'static str {
        trigger_name(self.trigger)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveCheckpointCompatibility {
    pub cursor: CheckpointCursor,
    pub warning_fingerprint: String,
    pub max_flagged_score: Option<u8>,
    pub flagged: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum LiveInputError {
    #[error("required live checkpoint fixture is missing: {path}")]
    MissingFixture { path: Utf8PathBuf },
    #[error("failed to read live checkpoint fixture {path}: {source}")]
    ReadFixture {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse live checkpoint fixture {path} at line {line_number}: {source}")]
    ParseFixtureLine {
        path: Utf8PathBuf,
        line_number: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("live checkpoint fixture {path} does not contain any events")]
    EmptyFixture { path: Utf8PathBuf },
    #[error(
        "live checkpoint event order must be append-only; line {line_number} used emission ordinal {current} after {previous}"
    )]
    NonMonotonicEmissionOrdinal {
        line_number: usize,
        previous: usize,
        current: usize,
    },
    #[error("live checkpoint-ready event at line {line_number} is missing checkpoint payload")]
    MissingCheckpointPayload { line_number: usize },
    #[error("live {trigger} event at line {line_number} must not include a checkpoint payload")]
    UnexpectedCheckpointPayload {
        line_number: usize,
        trigger: &'static str,
    },
    #[error(
        "live checkpoint-ready event at line {line_number} used cursor {actual_session_id}:{actual_ordinal}, expected {expected_session_id}:{expected_ordinal}"
    )]
    CheckpointCursorMismatch {
        line_number: usize,
        expected_session_id: String,
        expected_ordinal: usize,
        actual_session_id: String,
        actual_ordinal: usize,
    },
    #[error(
        "live checkpoint-ready event at line {line_number} regressed or duplicated cursor {current_session_id}:{current_ordinal} after {previous_session_id}:{previous_ordinal}"
    )]
    OutOfOrderCheckpointCursor {
        line_number: usize,
        previous_session_id: String,
        previous_ordinal: usize,
        current_session_id: String,
        current_ordinal: usize,
    },
    #[error(
        "live {trigger} event at line {line_number} arrived before any checkpoint-ready event established a cursor"
    )]
    SyntheticEventBeforeCheckpoint {
        line_number: usize,
        trigger: &'static str,
    },
    #[error(
        "live {trigger} event at line {line_number} referenced cursor {actual_session_id}:{actual_ordinal}, expected the latest checkpoint cursor {expected_session_id}:{expected_ordinal}"
    )]
    SyntheticCursorMismatch {
        line_number: usize,
        trigger: &'static str,
        expected_session_id: String,
        expected_ordinal: usize,
        actual_session_id: String,
        actual_ordinal: usize,
    },
    #[error(
        "analyzer checkpoint {checkpoint_id} is not live-compatible: missing or invalid {field} ({reason})"
    )]
    CompatibilityGap {
        checkpoint_id: String,
        field: &'static str,
        reason: String,
    },
}

pub trait LiveCheckpointSource {
    fn next_event(&mut self) -> Result<Option<LiveCheckpointEvent>, LiveInputError>;
}

#[derive(Debug, Clone)]
pub struct FixtureLiveCheckpointSource {
    events: Vec<LiveCheckpointEvent>,
    next_index: usize,
}

impl FixtureLiveCheckpointSource {
    pub fn from_path(path: &Utf8Path) -> Result<Self, LiveInputError> {
        let events = load_live_fixture(path)?;
        Ok(Self {
            events,
            next_index: 0,
        })
    }

    pub fn events(&self) -> &[LiveCheckpointEvent] {
        &self.events
    }
}

impl LiveCheckpointSource for FixtureLiveCheckpointSource {
    fn next_event(&mut self) -> Result<Option<LiveCheckpointEvent>, LiveInputError> {
        if self.next_index >= self.events.len() {
            return Ok(None);
        }
        let event = self.events[self.next_index].clone();
        self.next_index += 1;
        Ok(Some(event))
    }
}

pub fn load_live_fixture(path: &Utf8Path) -> Result<Vec<LiveCheckpointEvent>, LiveInputError> {
    if !path.exists() {
        return Err(LiveInputError::MissingFixture {
            path: path.to_owned(),
        });
    }

    let file = fs::File::open(path).map_err(|source| LiveInputError::ReadFixture {
        path: path.to_owned(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line.map_err(|source| LiveInputError::ReadFixture {
            path: path.to_owned(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let record: LiveCheckpointFixtureRecord =
            serde_json::from_str(&line).map_err(|source| LiveInputError::ParseFixtureLine {
                path: path.to_owned(),
                line_number,
                source,
            })?;
        events.push(record.into_event());
    }

    if events.is_empty() {
        return Err(LiveInputError::EmptyFixture {
            path: path.to_owned(),
        });
    }

    validate_live_event_sequence(&events)?;
    Ok(events)
}

pub fn validate_live_event_sequence(events: &[LiveCheckpointEvent]) -> Result<(), LiveInputError> {
    let mut last_emission_ordinal = None;
    let mut last_checkpoint_cursor: Option<CheckpointCursor> = None;

    for (index, event) in events.iter().enumerate() {
        let line_number = index + 1;

        if let Some(previous) = last_emission_ordinal {
            if event.emission_ordinal <= previous {
                return Err(LiveInputError::NonMonotonicEmissionOrdinal {
                    line_number,
                    previous,
                    current: event.emission_ordinal,
                });
            }
        }

        match event.trigger {
            TriggerClass::CheckpointReady => {
                let Some(checkpoint) = event.checkpoint.as_ref() else {
                    return Err(LiveInputError::MissingCheckpointPayload { line_number });
                };
                let expected_cursor = CheckpointCursor::from(checkpoint);
                if event.cursor != expected_cursor {
                    return Err(LiveInputError::CheckpointCursorMismatch {
                        line_number,
                        expected_session_id: expected_cursor.session_id,
                        expected_ordinal: expected_cursor.ordinal,
                        actual_session_id: event.cursor.session_id.clone(),
                        actual_ordinal: event.cursor.ordinal,
                    });
                }
                if let Some(previous) = last_checkpoint_cursor.as_ref() {
                    if expected_cursor <= *previous {
                        return Err(LiveInputError::OutOfOrderCheckpointCursor {
                            line_number,
                            previous_session_id: previous.session_id.clone(),
                            previous_ordinal: previous.ordinal,
                            current_session_id: expected_cursor.session_id,
                            current_ordinal: expected_cursor.ordinal,
                        });
                    }
                }
                last_checkpoint_cursor = Some(event.cursor.clone());
            }
            trigger => {
                if event.checkpoint.is_some() {
                    return Err(LiveInputError::UnexpectedCheckpointPayload {
                        line_number,
                        trigger: trigger_name(trigger),
                    });
                }
                let Some(expected_cursor) = last_checkpoint_cursor.as_ref() else {
                    return Err(LiveInputError::SyntheticEventBeforeCheckpoint {
                        line_number,
                        trigger: event.trigger_name(),
                    });
                };
                if event.cursor != *expected_cursor {
                    return Err(LiveInputError::SyntheticCursorMismatch {
                        line_number,
                        trigger: event.trigger_name(),
                        expected_session_id: expected_cursor.session_id.clone(),
                        expected_ordinal: expected_cursor.ordinal,
                        actual_session_id: event.cursor.session_id.clone(),
                        actual_ordinal: event.cursor.ordinal,
                    });
                }
            }
        }

        last_emission_ordinal = Some(event.emission_ordinal);
    }

    Ok(())
}

pub fn verify_live_checkpoint_compatibility(
    checkpoint: &Checkpoint,
) -> Result<LiveCheckpointCompatibility, LiveInputError> {
    if checkpoint.schema_version != SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA {
        return Err(compatibility_gap(
            checkpoint,
            "schema_version",
            format!(
                "expected {} but found {}",
                SUPPORTED_ANALYZER_CHECKPOINT_SCHEMA, checkpoint.schema_version
            ),
        ));
    }
    if checkpoint.session_id.trim().is_empty() {
        return Err(compatibility_gap(
            checkpoint,
            "session_id",
            "empty session identifier".to_string(),
        ));
    }
    if checkpoint.checkpoint_id.trim().is_empty() {
        return Err(compatibility_gap(
            checkpoint,
            "checkpoint_id",
            "empty checkpoint identifier".to_string(),
        ));
    }
    if checkpoint.task_frame.objective.trim().is_empty() {
        return Err(compatibility_gap(
            checkpoint,
            "task_frame.objective",
            "empty objective prevents live warning summaries".to_string(),
        ));
    }
    if checkpoint.expected_next_step.trim().is_empty() {
        return Err(compatibility_gap(
            checkpoint,
            "expected_next_step",
            "empty expected next step prevents operator guidance".to_string(),
        ));
    }

    Ok(LiveCheckpointCompatibility {
        cursor: CheckpointCursor::from(checkpoint),
        warning_fingerprint: warning_fingerprint(checkpoint),
        max_flagged_score: checkpoint
            .drift_scores
            .iter()
            .filter(|score| score.flagged)
            .map(|score| score.raw_score)
            .max(),
        flagged: checkpoint.flagged,
    })
}

fn compatibility_gap(
    checkpoint: &Checkpoint,
    field: &'static str,
    reason: String,
) -> LiveInputError {
    LiveInputError::CompatibilityGap {
        checkpoint_id: checkpoint.checkpoint_id.clone(),
        field,
        reason,
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

#[derive(Debug, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
enum LiveCheckpointFixtureRecord {
    CheckpointReady {
        emission_ordinal: usize,
        checkpoint: Checkpoint,
        source_label: Option<String>,
    },
    Heartbeat {
        emission_ordinal: usize,
        cursor: CheckpointCursor,
        source_label: Option<String>,
    },
    RepeatedFailure {
        emission_ordinal: usize,
        cursor: CheckpointCursor,
        source_label: Option<String>,
    },
    ManualReview {
        emission_ordinal: usize,
        cursor: CheckpointCursor,
        source_label: Option<String>,
    },
}

impl LiveCheckpointFixtureRecord {
    fn into_event(self) -> LiveCheckpointEvent {
        match self {
            Self::CheckpointReady {
                emission_ordinal,
                checkpoint,
                source_label,
            } => LiveCheckpointEvent::checkpoint_ready(emission_ordinal, checkpoint, source_label),
            Self::Heartbeat {
                emission_ordinal,
                cursor,
                source_label,
            } => LiveCheckpointEvent::heartbeat(emission_ordinal, cursor, source_label),
            Self::RepeatedFailure {
                emission_ordinal,
                cursor,
                source_label,
            } => LiveCheckpointEvent::repeated_failure(emission_ordinal, cursor, source_label),
            Self::ManualReview {
                emission_ordinal,
                cursor,
                source_label,
            } => LiveCheckpointEvent::manual_review(emission_ordinal, cursor, source_label),
        }
    }
}
