use camino::{Utf8Path, Utf8PathBuf};
use codex::{rollout_jsonl_file, RolloutEvent, RolloutJsonlError, RolloutUnknown};

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
    let mut reader = rollout_jsonl_file(path).map_err(|source| IngestError::Open {
        path: path.to_owned(),
        source,
    })?;

    let mut session_id = None;
    let mut records = Vec::new();
    let mut unknown_records = Vec::new();
    let mut parse_failures = Vec::new();

    for (event_index, record) in (&mut reader).enumerate() {
        match record.outcome {
            Ok(event) => {
                if let RolloutEvent::SessionMeta(meta) = &event {
                    if session_id.is_none() {
                        session_id = meta.payload.id.clone();
                    }
                }
                if let RolloutEvent::Unknown(unknown) = &event {
                    unknown_records.push(IngestedRolloutUnknown {
                        source_file: path.to_owned(),
                        line_number: record.line_number,
                        event_index,
                        event: unknown.clone(),
                    });
                }
                records.push(IngestedRolloutRecord {
                    source_file: path.to_owned(),
                    line_number: record.line_number,
                    event_index,
                    event,
                });
            }
            Err(error) => {
                parse_failures.push(RolloutParseFailure {
                    source_file: path.to_owned(),
                    line_number: record.line_number,
                    event_index,
                    error: error.to_string(),
                });
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

fn is_rollout_jsonl(path: &Utf8Path) -> bool {
    matches!(
        path.file_name(),
        Some(file_name) if file_name.starts_with("rollout-") && file_name.ends_with(".jsonl")
    )
}
