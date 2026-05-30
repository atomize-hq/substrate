mod row;

use std::cmp::Ordering;

use codex::{RolloutEvent, RolloutEventMsg, RolloutResponseItem, RolloutUnknown};
use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::canonicalize::canonicalize_row_text;
use crate::ingest::{IngestedRolloutFile, IngestedRolloutRecord, RolloutParseFailure};

pub use row::{CompactionKind, CompactionRow, SourceKind};

pub fn normalize_rollout_file(rollout: &IngestedRolloutFile) -> Vec<CompactionRow> {
    let mut rows = Vec::new();
    let mut current_turn_id = None;
    let mut entries = normalization_entries(rollout);

    entries.sort_by(|left, right| match left.event_index().cmp(&right.event_index()) {
        Ordering::Equal => left.line_number().cmp(&right.line_number()),
        other => other,
    });

    for entry in entries {
        match entry {
            NormalizationEntry::Record(record) => match &record.event {
                RolloutEvent::SessionMeta(meta) => {
                    if let Some(text) = meta
                        .payload
                        .base_instructions
                        .as_ref()
                        .and_then(|base| base.text.as_deref())
                        .and_then(non_empty)
                    {
                        rows.push(build_row(
                            rollout,
                            record,
                            CompactionKind::SystemMessage,
                            None,
                            meta.timestamp.as_deref(),
                            text.to_string(),
                        ));
                    }
                }
                RolloutEvent::EventMsg(message) => {
                    if let Some(turn_id) = extract_turn_id_from_value_map(&message.payload.extra) {
                        current_turn_id = Some(turn_id);
                    }
                    if let Some(row) = normalize_event_message(
                        rollout,
                        record,
                        message,
                        current_turn_id.clone(),
                    ) {
                        rows.push(row);
                    }
                }
                RolloutEvent::ResponseItem(item) => {
                    if let Some(row) =
                        normalize_response_item(rollout, record, item, current_turn_id.clone())
                    {
                        rows.push(row);
                    }
                }
                RolloutEvent::Unknown(unknown) => {
                    if let Some(turn_id) = extract_turn_id_from_value(&unknown.payload) {
                        current_turn_id = Some(turn_id);
                    }
                    if let Some(row) =
                        normalize_unknown_record(rollout, record, unknown, current_turn_id.clone())
                    {
                        rows.push(row);
                    }
                }
            },
            NormalizationEntry::ParseFailure(failure) => {
                rows.push(build_failure_row(rollout, failure, current_turn_id.clone()));
            }
        }
    }

    rows
}

fn normalize_event_message(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    message: &RolloutEventMsg,
    turn_id: Option<String>,
) -> Option<CompactionRow> {
    let kind = message.payload.kind.as_deref()?;
    match kind {
        "agent_message" => extract_message_text(&message.payload.extra).map(|text| {
            build_row(
                rollout,
                record,
                CompactionKind::AssistantMessage,
                turn_id,
                message.timestamp.as_deref(),
                text,
            )
        }),
        "user_message" => extract_message_text(&message.payload.extra).map(|text| {
            build_row(
                rollout,
                record,
                CompactionKind::UserMessage,
                turn_id,
                message.timestamp.as_deref(),
                text,
            )
        }),
        "patch_apply_end" => Some(build_row(
            rollout,
            record,
            CompactionKind::ToolOutput,
            turn_id,
            message.timestamp.as_deref(),
            serialize_kind_and_extra(kind, &message.payload.extra),
        )),
        "task_started" | "web_search_end" => Some(build_row(
            rollout,
            record,
            CompactionKind::Status,
            turn_id,
            message.timestamp.as_deref(),
            serialize_kind_and_extra(kind, &message.payload.extra),
        )),
        "token_count" => None,
        _ => Some(build_row(
            rollout,
            record,
            CompactionKind::Unknown,
            turn_id,
            message.timestamp.as_deref(),
            serialize_kind_and_extra(kind, &message.payload.extra),
        )),
    }
}

fn normalize_response_item(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    item: &RolloutResponseItem,
    turn_id: Option<String>,
) -> Option<CompactionRow> {
    let kind = item.payload.kind.as_deref()?;
    match kind {
        "message" => {
            let text = extract_content_text(item.payload.content.as_ref())
                .or_else(|| encrypted_placeholder("encrypted_message_content", item.payload.encrypted_content.as_deref()))?;
            Some(build_row(
                rollout,
                record,
                message_role_kind(item.payload.role.as_deref()),
                turn_id,
                item.timestamp.as_deref(),
                text,
            ))
        }
        "reasoning" => {
            let text = extract_content_text(item.payload.summary.as_ref())
                .or_else(|| extract_content_text(item.payload.content.as_ref()))
                .or_else(|| encrypted_placeholder("encrypted_reasoning", item.payload.encrypted_content.as_deref()))?;
            Some(build_row(
                rollout,
                record,
                CompactionKind::Reasoning,
                turn_id,
                item.timestamp.as_deref(),
                text,
            ))
        }
        "function_call" | "custom_tool_call" | "web_search_call" => Some(build_row(
            rollout,
            record,
            CompactionKind::ToolCall,
            turn_id,
            item.timestamp.as_deref(),
            item.payload
                .arguments
                .clone()
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| serialize_payload(kind, &item.payload.extra, item.payload.name.as_deref(), item.payload.call_id.as_deref(), None)),
        )),
        "function_call_output" | "custom_tool_call_output" => Some(build_row(
            rollout,
            record,
            CompactionKind::ToolOutput,
            turn_id,
            item.timestamp.as_deref(),
            item.payload
                .output
                .clone()
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| serialize_payload(kind, &item.payload.extra, item.payload.name.as_deref(), item.payload.call_id.as_deref(), None)),
        )),
        _ => Some(build_row(
            rollout,
            record,
            CompactionKind::Unknown,
            turn_id,
            item.timestamp.as_deref(),
            serialize_payload(kind, &item.payload.extra, item.payload.name.as_deref(), item.payload.call_id.as_deref(), item.payload.role.as_deref()),
        )),
    }
}

fn normalize_unknown_record(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    unknown: &RolloutUnknown,
    turn_id: Option<String>,
) -> Option<CompactionRow> {
    match unknown.record_type.as_str() {
        "turn_context" => {
            let text = unknown
                .payload
                .get("user_instructions")
                .and_then(Value::as_str)
                .and_then(non_empty)?;
            Some(build_row(
                rollout,
                record,
                CompactionKind::SystemMessage,
                turn_id,
                unknown.timestamp.as_deref(),
                text.to_string(),
            ))
        }
        _ => None,
    }
}

fn build_failure_row(
    rollout: &IngestedRolloutFile,
    failure: &RolloutParseFailure,
    turn_id: Option<String>,
) -> CompactionRow {
    let (canonical_text, text_hash_hex) = canonicalize_row_text(&failure.error);
    CompactionRow {
        source_file: failure.source_file.clone(),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: rollout.session_id.clone(),
        turn_id,
        event_index: failure.event_index,
        line_number: failure.line_number,
        timestamp: None,
        kind: CompactionKind::Error,
        text: failure.error.clone(),
        canonical_text,
        text_hash_hex,
    }
}

fn build_row(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    kind: CompactionKind,
    turn_id: Option<String>,
    timestamp: Option<&str>,
    text: String,
) -> CompactionRow {
    let (canonical_text, text_hash_hex) = canonicalize_row_text(&text);
    CompactionRow {
        source_file: record.source_file.clone(),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: rollout.session_id.clone(),
        turn_id,
        event_index: record.event_index,
        line_number: record.line_number,
        timestamp: parse_timestamp(timestamp),
        kind,
        text,
        canonical_text,
        text_hash_hex,
    }
}

fn normalization_entries<'a>(rollout: &'a IngestedRolloutFile) -> Vec<NormalizationEntry<'a>> {
    let mut entries = Vec::with_capacity(rollout.records.len() + rollout.parse_failures.len());
    entries.extend(rollout.records.iter().map(NormalizationEntry::Record));
    entries.extend(
        rollout
            .parse_failures
            .iter()
            .map(NormalizationEntry::ParseFailure),
    );
    entries
}

fn parse_timestamp(timestamp: Option<&str>) -> Option<OffsetDateTime> {
    timestamp.and_then(|value| OffsetDateTime::parse(value, &Rfc3339).ok())
}

fn extract_message_text(values: &std::collections::BTreeMap<String, Value>) -> Option<String> {
    values
        .get("message")
        .and_then(Value::as_str)
        .and_then(non_empty)
        .map(ToOwned::to_owned)
}

fn extract_content_text(parts: Option<&Vec<codex::RolloutContentPart>>) -> Option<String> {
    let text_parts = parts?
        .iter()
        .filter_map(|part| part.text.as_deref())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    (!text_parts.is_empty()).then(|| text_parts.join("\n\n"))
}

fn extract_turn_id_from_value_map(
    values: &std::collections::BTreeMap<String, Value>,
) -> Option<String> {
    values
        .get("turn_id")
        .and_then(Value::as_str)
        .and_then(non_empty)
        .map(ToOwned::to_owned)
}

fn extract_turn_id_from_value(value: &Value) -> Option<String> {
    value
        .get("turn_id")
        .and_then(Value::as_str)
        .and_then(non_empty)
        .map(ToOwned::to_owned)
}

fn message_role_kind(role: Option<&str>) -> CompactionKind {
    match role {
        Some("user") => CompactionKind::UserMessage,
        Some("developer") => CompactionKind::DeveloperMessage,
        Some("system") => CompactionKind::SystemMessage,
        _ => CompactionKind::AssistantMessage,
    }
}

fn encrypted_placeholder(label: &str, encrypted_content: Option<&str>) -> Option<String> {
    encrypted_content
        .and_then(non_empty)
        .map(|_| format!("[{label}]"))
}

fn serialize_kind_and_extra(
    kind: &str,
    extra: &std::collections::BTreeMap<String, Value>,
) -> String {
    let mut object = Map::new();
    object.insert("type".to_string(), Value::String(kind.to_string()));
    for (key, value) in extra {
        object.insert(key.clone(), value.clone());
    }
    Value::Object(object).to_string()
}

fn serialize_payload(
    kind: &str,
    extra: &std::collections::BTreeMap<String, Value>,
    name: Option<&str>,
    call_id: Option<&str>,
    role: Option<&str>,
) -> String {
    let mut object = Map::new();
    object.insert("type".to_string(), Value::String(kind.to_string()));
    if let Some(name) = name.and_then(non_empty) {
        object.insert("name".to_string(), Value::String(name.to_string()));
    }
    if let Some(call_id) = call_id.and_then(non_empty) {
        object.insert("call_id".to_string(), Value::String(call_id.to_string()));
    }
    if let Some(role) = role.and_then(non_empty) {
        object.insert("role".to_string(), Value::String(role.to_string()));
    }
    for (key, value) in extra {
        object.insert(key.clone(), value.clone());
    }
    Value::Object(object).to_string()
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

enum NormalizationEntry<'a> {
    Record(&'a IngestedRolloutRecord),
    ParseFailure(&'a RolloutParseFailure),
}

impl<'a> NormalizationEntry<'a> {
    fn event_index(&self) -> usize {
        match self {
            Self::Record(record) => record.event_index,
            Self::ParseFailure(failure) => failure.event_index,
        }
    }

    fn line_number(&self) -> usize {
        match self {
            Self::Record(record) => record.line_number,
            Self::ParseFailure(failure) => failure.line_number,
        }
    }
}
