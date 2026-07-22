use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};

use camino::{Utf8Path, Utf8PathBuf};
use codex::{RolloutEvent, RolloutJsonlError, RolloutJsonlParser, RolloutUnknown};
use serde_json::Value;

use crate::discovery::DiscoveredSessionArtifact;
use crate::ingest::current_native::{
    current_native_marker, parse_current_native_line, CurrentNativeEvent,
    CurrentNativeEventMessage, CurrentNativeResponseItem,
};

/// Exact source location of a rollout record used to derive linkage metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolloutRowProvenance {
    /// Rollout JSONL file containing the record.
    pub source_file: Utf8PathBuf,
    /// One-based physical line number in `source_file`.
    pub line_number: usize,
    /// Zero-based ingested event ordinal; parse failures consume an ordinal.
    pub event_index: usize,
}

/// Parent-side delegation observed by matching a `spawn_agent` call to its output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentSpawnResult {
    /// Session identifier of the rollout containing the matched call and output.
    pub parent_session_id: String,
    /// Child session identifier carried by the selected rollout adapter.
    pub child_session_id: String,
    /// Call identifier shared by the `spawn_agent` call and its output.
    pub call_id: String,
    /// Source location of the `spawn_agent` function-call record.
    pub spawn_call_provenance: RolloutRowProvenance,
    /// Source location of the matching function-call-output record.
    pub spawn_result_provenance: RolloutRowProvenance,
}

/// Child-side delegation origin recorded in a rollout's session metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildSessionOrigin {
    /// Session identifier declared by the child rollout's session metadata.
    pub child_session_id: String,
    /// Parent thread identifier declared by `source.subagent.thread_spawn`.
    pub parent_session_id: String,
    /// Delegation depth declared by `source.subagent.thread_spawn`.
    pub depth: u32,
    /// Optional agent nickname recorded by the spawning runtime.
    pub agent_nickname: Option<String>,
    /// Optional agent role recorded by the spawning runtime.
    pub agent_role: Option<String>,
    /// Source location of the session-metadata record declaring the origin.
    pub provenance: RolloutRowProvenance,
}

/// Delegation linkage observations derived from one ingested rollout.
///
/// A parent rollout can contain multiple matched spawn results. A child rollout
/// contributes at most one origin: the first valid origin-bearing session
/// metadata record in ingestion order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RolloutLinkageMetadata {
    /// Matched parent-side spawn results, in output-record ingestion order.
    pub parent_spawn_results: Vec<ParentSpawnResult>,
    /// First valid child-side origin, when the rollout declares one.
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
    #[error("unsupported rollout format in {path}: {reason}")]
    UnsupportedFormat {
        path: Utf8PathBuf,
        reason: String,
    },
}

/// Explicit parser boundary selected for one rollout stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolloutFormat {
    Legacy,
    CurrentNativeV2,
}

/// Parsed record emitted by exactly one trusted rollout adapter.
#[derive(Debug, Clone)]
pub enum IngestedRolloutEvent {
    Legacy(RolloutEvent),
    CurrentNative(CurrentNativeEvent),
}

#[derive(Debug, Clone)]
pub struct IngestedRolloutRecord {
    pub source_file: Utf8PathBuf,
    pub line_number: usize,
    pub event_index: usize,
    pub event: IngestedRolloutEvent,
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
    pub format: RolloutFormat,
    pub session_id: Option<String>,
    pub records: Vec<IngestedRolloutRecord>,
    pub unknown_records: Vec<IngestedRolloutUnknown>,
    pub parse_failures: Vec<RolloutParseFailure>,
}

/// Extracts explicit parent- and child-side delegation linkage from a rollout.
///
/// Legacy parent-side linkage requires a `spawn_agent` function call and a
/// function-call output with the same `call_id`; the output must be JSON with a
/// string `agent_id`. Current-native parent-side linkage uses the typed
/// collaboration spawn records and their direct `new_thread_id`. Child-side
/// linkage requires a session-metadata source matching `subagent.thread_spawn`.
/// Missing or malformed linkage fields are omitted rather than inferred. Every
/// returned observation retains the source file, physical line, and ingested
/// event ordinal of the records that proved it.
///
/// # Examples
///
/// ```
/// use std::fs;
///
/// use agent_session_compactor::{extract_rollout_linkage_metadata, ingest_rollout_file};
/// use camino::Utf8Path;
/// use tempfile::tempdir;
///
/// let temp_dir = tempdir()?;
/// let path = temp_dir.path().join("rollout-parent.jsonl");
/// fs::write(
///     &path,
///     concat!(
///         "{\"type\":\"session_meta\",\"payload\":{\"id\":\"parent\"}}\n",
///         "{\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"spawn_agent\",\"call_id\":\"call-1\"}}\n",
///         "{\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-1\",\"output\":\"{\\\"agent_id\\\":\\\"child\\\"}\"}}\n",
///     ),
/// )?;
/// let path = Utf8Path::from_path(&path).expect("temporary path is valid UTF-8");
/// let rollout = ingest_rollout_file(path)?;
///
/// let metadata = extract_rollout_linkage_metadata(&rollout);
/// let spawn = metadata.parent_spawn_results.first().expect("matched spawn");
/// assert_eq!(spawn.child_session_id, "child");
/// assert_eq!(spawn.spawn_call_provenance.line_number, 2);
/// assert_eq!(spawn.spawn_call_provenance.event_index, 1);
/// assert_eq!(spawn.spawn_result_provenance.line_number, 3);
/// assert_eq!(spawn.spawn_result_provenance.source_file, rollout.source_file);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_rollout_linkage_metadata(rollout: &IngestedRolloutFile) -> RolloutLinkageMetadata {
    let mut spawn_calls = BTreeMap::<String, Vec<&IngestedRolloutRecord>>::new();
    for record in &rollout.records {
        let call_id = match &record.event {
            IngestedRolloutEvent::Legacy(RolloutEvent::ResponseItem(item))
                if item.payload.kind.as_deref() == Some("function_call")
                    && item.payload.name.as_deref() == Some("spawn_agent") =>
            {
                item.payload.call_id.as_deref()
            }
            IngestedRolloutEvent::CurrentNative(CurrentNativeEvent::ResponseItem(
                CurrentNativeResponseItem::ToolCall {
                    name: Some(name),
                    call_id,
                    ..
                },
            )) if name == "spawn_agent" => call_id.as_deref(),
            IngestedRolloutEvent::CurrentNative(CurrentNativeEvent::EventMessage(
                CurrentNativeEventMessage::CollabAgentSpawnBegin { call_id, .. },
            )) => Some(call_id.as_str()),
            _ => None,
        };
        if let Some(call_id) = call_id {
            spawn_calls
                .entry(call_id.to_string())
                .or_default()
                .push(record);
        }
    }

    let mut parent_spawn_results = Vec::new();
    if let Some(parent_session_id) = rollout.session_id.as_deref() {
        for record in &rollout.records {
            match &record.event {
                IngestedRolloutEvent::Legacy(RolloutEvent::ResponseItem(item))
                    if item.payload.kind.as_deref() == Some("function_call_output") =>
                {
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
                IngestedRolloutEvent::CurrentNative(CurrentNativeEvent::EventMessage(
                    CurrentNativeEventMessage::CollabAgentSpawnEnd {
                        call_id,
                        sender_thread_id,
                        new_thread_id: Some(child_session_id),
                        ..
                    },
                )) if sender_thread_id == parent_session_id => {
                    let spawn_call = current_spawn_call_record(
                        spawn_calls.get(call_id).map(Vec::as_slice),
                        record,
                    );
                    parent_spawn_results.push(ParentSpawnResult {
                        parent_session_id: parent_session_id.to_string(),
                        child_session_id: child_session_id.clone(),
                        call_id: call_id.clone(),
                        spawn_call_provenance: row_provenance(spawn_call),
                        spawn_result_provenance: row_provenance(record),
                    });
                }
                _ => {}
            }
        }
    }

    let child_origin = rollout.records.iter().find_map(|record| match &record.event {
        IngestedRolloutEvent::Legacy(RolloutEvent::SessionMeta(meta)) => {
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
        }
        IngestedRolloutEvent::CurrentNative(CurrentNativeEvent::SessionMeta(meta)) => {
            let origin = meta.child_origin.as_ref()?;
            Some(ChildSessionOrigin {
                child_session_id: meta.thread_id.clone(),
                parent_session_id: origin.parent_thread_id.clone(),
                depth: origin.depth,
                agent_nickname: origin.agent_nickname.clone(),
                agent_role: origin.agent_role.clone(),
                provenance: row_provenance(record),
            })
        }
        _ => None,
    });

    RolloutLinkageMetadata {
        parent_spawn_results,
        child_origin,
    }
}

fn current_spawn_call_record<'a>(
    calls: Option<&[&'a IngestedRolloutRecord]>,
    result: &'a IngestedRolloutRecord,
) -> &'a IngestedRolloutRecord {
    calls
        .and_then(|calls| {
            calls.iter().copied().find(|record| {
                matches!(
                    &record.event,
                    IngestedRolloutEvent::CurrentNative(CurrentNativeEvent::EventMessage(
                        CurrentNativeEventMessage::CollabAgentSpawnBegin { .. }
                    ))
                )
            })
        })
        .or_else(|| calls.and_then(|calls| calls.first().copied()))
        .unwrap_or(result)
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
    let lines = reader
        .lines()
        .map(|line| {
            line.map_err(|source| IngestError::Open {
                path: path.to_owned(),
                source: RolloutJsonlError::Io { source },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let format = select_rollout_format(path, &lines)?;
    match format {
        RolloutFormat::Legacy => ingest_legacy_lines(path, lines),
        RolloutFormat::CurrentNativeV2 => ingest_current_native_lines(path, lines),
    }
}

fn select_rollout_format(
    path: &Utf8Path,
    lines: &[String],
) -> Result<RolloutFormat, IngestError> {
    for line in lines {
        match current_native_marker(line) {
            Ok(Some(true)) => return Ok(RolloutFormat::CurrentNativeV2),
            Ok(Some(false)) => return Ok(RolloutFormat::Legacy),
            Ok(None) => {}
            Err(reason) => {
                return Err(IngestError::UnsupportedFormat {
                    path: path.to_owned(),
                    reason,
                });
            }
        }
    }
    Ok(RolloutFormat::Legacy)
}

fn ingest_legacy_lines(
    path: &Utf8Path,
    lines: Vec<String>,
) -> Result<IngestedRolloutFile, IngestError> {
    let mut parser = RolloutJsonlParser::new();
    let mut session_id = None;
    let mut records = Vec::new();
    let mut unknown_records = Vec::new();
    let mut parse_failures = Vec::new();
    let mut event_index = 0;

    for (line_index, line) in lines.into_iter().enumerate() {
        let line_number = line_index + 1;
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
                    event: IngestedRolloutEvent::Legacy(event),
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
        format: RolloutFormat::Legacy,
        session_id,
        records,
        unknown_records,
        parse_failures,
    })
}

fn ingest_current_native_lines(
    path: &Utf8Path,
    lines: Vec<String>,
) -> Result<IngestedRolloutFile, IngestError> {
    let mut session_id = None;
    let mut records = Vec::new();
    let mut parse_failures = Vec::new();
    let mut event_index = 0;

    for (line_index, line) in lines.into_iter().enumerate() {
        let line_number = line_index + 1;
        match parse_current_native_line(&line) {
            Ok(None) => continue,
            Ok(Some(event)) => {
                if let CurrentNativeEvent::SessionMeta(meta) = &event {
                    if session_id.is_none() {
                        session_id = Some(meta.thread_id.clone());
                    }
                }
                records.push(IngestedRolloutRecord {
                    source_file: path.to_owned(),
                    line_number,
                    event_index,
                    event: IngestedRolloutEvent::CurrentNative(event),
                });
                event_index += 1;
            }
            Err(error) => {
                parse_failures.push(RolloutParseFailure {
                    source_file: path.to_owned(),
                    line_number,
                    event_index,
                    error,
                });
                event_index += 1;
            }
        }
    }

    Ok(IngestedRolloutFile {
        source_file: path.to_owned(),
        format: RolloutFormat::CurrentNativeV2,
        session_id,
        records,
        unknown_records: Vec::new(),
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
