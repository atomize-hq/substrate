use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};

use camino::{Utf8Path, Utf8PathBuf};
use codex::{RolloutEvent, RolloutJsonlError, RolloutJsonlParser, RolloutUnknown};
use serde_json::Value;

use crate::discovery::DiscoveredSessionArtifact;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolloutRowProvenance {
    pub source_file: Utf8PathBuf,
    pub line_number: usize,
    pub event_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentSpawnResult {
    pub parent_session_id: String,
    pub child_session_id: String,
    pub call_id: String,
    pub spawn_call_provenance: RolloutRowProvenance,
    pub spawn_result_provenance: RolloutRowProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildSessionOrigin {
    pub child_session_id: String,
    pub parent_session_id: String,
    pub depth: u32,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
    pub provenance: RolloutRowProvenance,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RolloutLinkageMetadata {
    pub parent_spawn_results: Vec<ParentSpawnResult>,
    pub child_origin: Option<ChildSessionOrigin>,
}

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

pub fn extract_rollout_linkage_metadata(rollout: &IngestedRolloutFile) -> RolloutLinkageMetadata {
    let mut spawn_calls = BTreeMap::<&str, Vec<&IngestedRolloutRecord>>::new();
    for record in &rollout.records {
        let RolloutEvent::ResponseItem(item) = &record.event else {
            continue;
        };
        if item.payload.kind.as_deref() == Some("function_call")
            && item.payload.name.as_deref() == Some("spawn_agent")
        {
            if let Some(call_id) = item.payload.call_id.as_deref() {
                spawn_calls.entry(call_id).or_default().push(record);
            }
        }
    }

    let mut parent_spawn_results = Vec::new();
    if let Some(parent_session_id) = rollout.session_id.as_deref() {
        for record in &rollout.records {
            let RolloutEvent::ResponseItem(item) = &record.event else {
                continue;
            };
            if item.payload.kind.as_deref() != Some("function_call_output") {
                continue;
            }
            let Some(call_id) = item.payload.call_id.as_deref() else {
                continue;
            };
            let Some(child_session_id) = item
                .payload
                .output
                .as_deref()
                .and_then(extract_spawn_result_child_id)
            else {
                continue;
            };

            for spawn_call in spawn_calls.get(call_id).into_iter().flatten() {
                parent_spawn_results.push(ParentSpawnResult {
                    parent_session_id: parent_session_id.to_string(),
                    child_session_id: child_session_id.clone(),
                    call_id: call_id.to_string(),
                    spawn_call_provenance: row_provenance(spawn_call),
                    spawn_result_provenance: row_provenance(record),
                });
            }
        }
    }

    let child_origin = rollout.records.iter().find_map(|record| {
        let RolloutEvent::SessionMeta(meta) = &record.event else {
            return None;
        };
        let child_session_id = meta.payload.id.as_ref()?;
        let source = meta.payload.source.as_deref()?;
        let source: ChildOriginSource = serde_json::from_str(source).ok()?;
        let origin = source.subagent.thread_spawn;
        Some(ChildSessionOrigin {
            child_session_id: child_session_id.clone(),
            parent_session_id: origin.parent_thread_id,
            depth: origin.depth,
            agent_nickname: origin.agent_nickname,
            agent_role: origin.agent_role,
            provenance: row_provenance(record),
        })
    });

    RolloutLinkageMetadata {
        parent_spawn_results,
        child_origin,
    }
}

fn extract_spawn_result_child_id(output: &str) -> Option<String> {
    serde_json::from_str::<Value>(output)
        .ok()?
        .get("agent_id")?
        .as_str()
        .map(str::to_string)
}

fn row_provenance(record: &IngestedRolloutRecord) -> RolloutRowProvenance {
    RolloutRowProvenance {
        source_file: record.source_file.clone(),
        line_number: record.line_number,
        event_index: record.event_index,
    }
}

#[derive(serde::Deserialize)]
struct ChildOriginSource {
    subagent: ChildOriginSubagent,
}

#[derive(serde::Deserialize)]
struct ChildOriginSubagent {
    thread_spawn: ChildOriginThreadSpawn,
}

#[derive(serde::Deserialize)]
struct ChildOriginThreadSpawn {
    parent_thread_id: String,
    depth: u32,
    agent_nickname: Option<String>,
    agent_role: Option<String>,
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
