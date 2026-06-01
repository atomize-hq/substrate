use std::fs;
use std::io::{BufRead, BufReader};

use camino::{Utf8Path, Utf8PathBuf};
use codex::{RolloutEvent, RolloutJsonlError, RolloutJsonlParser, RolloutUnknown};
use serde_json::Value;

use crate::discovery::DiscoveredSessionArtifact;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("failed to open rollout JSONL file {path}: {source}")]
    Open {
        path: Utf8PathBuf,
        #[source]
        source: RolloutJsonlError,
    },
}

#[derive(Debug, Clone)]
pub struct IngestedRolloutRecord {
    pub source_file: Utf8PathBuf,
    pub line_number: usize,
    pub event_index: usize,
    pub event: RolloutEvent,
}

#[derive(Debug, Clone)]
pub struct IngestedRolloutUnknown {
    pub source_file: Utf8PathBuf,
    pub line_number: usize,
    pub event_index: usize,
    pub event: RolloutUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolloutParseFailure {
    pub source_file: Utf8PathBuf,
    pub line_number: usize,
    pub event_index: usize,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct IngestedRolloutFile {
    pub source_file: Utf8PathBuf,
    pub session_id: Option<String>,
    pub records: Vec<IngestedRolloutRecord>,
    pub unknown_records: Vec<IngestedRolloutUnknown>,
    pub parse_failures: Vec<RolloutParseFailure>,
}

pub fn ingest_rollout_artifacts(
    artifacts: &[DiscoveredSessionArtifact],
) -> Result<Vec<IngestedRolloutFile>, IngestError> {
    artifacts
        .iter()
        .filter(|artifact| is_rollout_jsonl(&artifact.path))
        .map(|artifact| ingest_rollout_file(&artifact.path))
        .collect()
}

pub fn ingest_rollout_file(path: &Utf8Path) -> Result<IngestedRolloutFile, IngestError> {
    let file = fs::File::open(path).map_err(|source| IngestError::Open {
        path: path.to_owned(),
        source: RolloutJsonlError::Io { source },
    })?;
    let reader = BufReader::new(file);
    let mut parser = RolloutJsonlParser::new();

    let mut session_id = None;
    let mut records = Vec::new();
    let mut unknown_records = Vec::new();
    let mut parse_failures = Vec::new();

    let mut event_index = 0;
    for (line_index, line) in reader.lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.map_err(|source| IngestError::Open {
            path: path.to_owned(),
            source: RolloutJsonlError::Io { source },
        })?;
        match parse_rollout_line(&mut parser, &line) {
            Ok(None) => continue,
            Ok(Some(event)) => {
                if let RolloutEvent::SessionMeta(meta) = &event {
                    if session_id.is_none() {
                        session_id = meta.payload.id.clone();
                    }
                }
                if let RolloutEvent::Unknown(unknown) = &event {
                    unknown_records.push(IngestedRolloutUnknown {
                        source_file: path.to_owned(),
                        line_number,
                        event_index,
                        event: unknown.clone(),
                    });
                }
                records.push(IngestedRolloutRecord {
                    source_file: path.to_owned(),
                    line_number,
                    event_index,
                    event,
                });
                event_index += 1;
            }
            Err(error) => {
                parse_failures.push(RolloutParseFailure {
                    source_file: path.to_owned(),
                    line_number,
                    event_index,
                    error: error.to_string(),
                });
                event_index += 1;
            }
        }
    }

    Ok(IngestedRolloutFile {
        source_file: path.to_owned(),
        session_id,
        records,
        unknown_records,
        parse_failures,
    })
}

fn parse_rollout_line(
    parser: &mut RolloutJsonlParser,
    line: &str,
) -> Result<Option<RolloutEvent>, RolloutJsonlError> {
    match parser.parse_line(line) {
        Ok(event) => Ok(event),
        Err(error) => {
            let Some(sanitized) = sanitize_known_live_shapes(line) else {
                return Err(error);
            };
            parser.parse_line(&sanitized)
        }
    }
}

fn sanitize_known_live_shapes(line: &str) -> Option<String> {
    let mut value: Value = serde_json::from_str(line).ok()?;
    let record_type = value.get("type")?.as_str()?.to_string();
    let payload = value.get_mut("payload")?.as_object_mut()?;
    let mut changed = false;

    if record_type == "session_meta" {
        changed |= coerce_value_to_json_string(payload, "source");
    }
    if record_type == "response_item" {
        changed |= coerce_value_to_json_string(payload, "arguments");
    }

    changed.then(|| serde_json::to_string(&value).expect("sanitized rollout line is JSON"))
}

fn coerce_value_to_json_string(
    object: &mut serde_json::Map<String, Value>,
    field_name: &str,
) -> bool {
    let Some(value) = object.get_mut(field_name) else {
        return false;
    };
    if value.is_null() || value.is_string() {
        return false;
    }
    *value = Value::String(serde_json::to_string(value).expect("field value serializes"));
    true
}

fn is_rollout_jsonl(path: &Utf8Path) -> bool {
    matches!(
        path.file_name(),
        Some(file_name) if file_name.starts_with("rollout-") && file_name.ends_with(".jsonl")
    )
}
