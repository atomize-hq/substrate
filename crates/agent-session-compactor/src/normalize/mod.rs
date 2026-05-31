mod row;

use std::cmp::Ordering;

use codex::{RolloutEvent, RolloutEventMsg, RolloutResponseItem, RolloutUnknown};
use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::canonicalize::canonicalize_row_text;
use crate::ingest::{IngestedRolloutFile, IngestedRolloutRecord, RolloutParseFailure};

pub use row::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};

pub fn normalize_rollout_file(rollout: &IngestedRolloutFile) -> Vec<CompactionRow> {
    let mut rows = Vec::new();
    let mut current_turn_id = None;
    let mut user_message_state = UserMessageState::default();
    let mut entries = normalization_entries(rollout);

    entries.sort_by(
        |left, right| match left.event_index().cmp(&right.event_index()) {
            Ordering::Equal => left.line_number().cmp(&right.line_number()),
            other => other,
        },
    );

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
                            0,
                            CompactionKind::SystemMessage,
                            None,
                            None,
                            meta.timestamp.as_deref(),
                            None,
                            text.to_string(),
                        ));
                    }
                }
                RolloutEvent::EventMsg(message) => {
                    if let Some(turn_id) = extract_turn_id_from_value_map(&message.payload.extra) {
                        current_turn_id = Some(turn_id.clone());
                        user_message_state.observe_turn_id(&turn_id);
                    }
                    if let Some(row) = normalize_event_message(
                        rollout,
                        record,
                        message,
                        current_turn_id.clone(),
                        &mut user_message_state,
                    ) {
                        rows.push(row);
                    }
                }
                RolloutEvent::ResponseItem(item) => {
                    if let Some(row) = normalize_response_item(
                        rollout,
                        record,
                        item,
                        current_turn_id.clone(),
                        &mut user_message_state,
                    ) {
                        rows.push(row);
                    }
                }
                RolloutEvent::Unknown(unknown) => {
                    if let Some(turn_id) = extract_turn_id_from_value(&unknown.payload) {
                        current_turn_id = Some(turn_id.clone());
                        user_message_state.observe_turn_id(&turn_id);
                    }
                    rows.extend(normalize_unknown_record(
                        rollout,
                        record,
                        unknown,
                        current_turn_id.clone(),
                        &mut user_message_state,
                    ));
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
    user_message_state: &mut UserMessageState,
) -> Option<CompactionRow> {
    let kind = message.payload.kind.as_deref()?;
    match kind {
        "agent_message" => extract_message_text(&message.payload.extra).map(|text| {
            build_row(
                rollout,
                record,
                0,
                CompactionKind::AssistantMessage,
                None,
                turn_id,
                message.timestamp.as_deref(),
                None,
                text,
            )
        }),
        "user_message" => extract_message_text(&message.payload.extra).and_then(|text| {
            if user_message_state.suppress_mirrored_user_message(
                UserMessageSource::EventMessage,
                record.event_index,
                turn_id.as_deref(),
                message.timestamp.as_deref(),
                &text,
            ) {
                return None;
            }
            let user_message_role = Some(user_message_state.classify(turn_id.as_deref(), &text));
            user_message_state.observe_emitted_user_message(
                UserMessageSource::EventMessage,
                record.event_index,
                turn_id.as_deref(),
                message.timestamp.as_deref(),
                &text,
            );
            Some(build_row(
                rollout,
                record,
                0,
                CompactionKind::UserMessage,
                user_message_role,
                turn_id,
                message.timestamp.as_deref(),
                None,
                text,
            ))
        }),
        "task_complete" => {
            if let Some(turn_id) = turn_id.as_deref() {
                user_message_state.observe_task_complete(turn_id);
            } else {
                user_message_state.observe_task_complete_without_turn();
            }
            Some(build_row(
                rollout,
                record,
                0,
                CompactionKind::Status,
                None,
                turn_id,
                message.timestamp.as_deref(),
                None,
                serialize_kind_and_extra(kind, &message.payload.extra),
            ))
        }
        "patch_apply_end" => Some(build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolOutput,
            None,
            turn_id,
            message.timestamp.as_deref(),
            None,
            serialize_kind_and_extra(kind, &message.payload.extra),
        )),
        "task_started" | "web_search_end" => {
            if kind == "task_started" {
                if let Some(turn_id) = turn_id.as_deref() {
                    user_message_state.observe_task_started(turn_id);
                } else {
                    user_message_state.observe_task_started_without_turn();
                }
            }
            Some(build_row(
                rollout,
                record,
                0,
                CompactionKind::Status,
                None,
                turn_id,
                message.timestamp.as_deref(),
                None,
                serialize_kind_and_extra(kind, &message.payload.extra),
            ))
        }
        "token_count" => None,
        _ => Some(build_row(
            rollout,
            record,
            0,
            CompactionKind::Unknown,
            None,
            turn_id,
            message.timestamp.as_deref(),
            None,
            serialize_kind_and_extra(kind, &message.payload.extra),
        )),
    }
}

fn normalize_response_item(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    item: &RolloutResponseItem,
    turn_id: Option<String>,
    user_message_state: &mut UserMessageState,
) -> Option<CompactionRow> {
    let kind = item.payload.kind.as_deref()?;
    match kind {
        "message" => {
            let text = extract_content_text(item.payload.content.as_ref()).or_else(|| {
                encrypted_placeholder(
                    "encrypted_message_content",
                    item.payload.encrypted_content.as_deref(),
                )
            })?;
            let message_kind = message_role_kind(item.payload.role.as_deref());
            if message_kind == CompactionKind::UserMessage
                && user_message_state.suppress_mirrored_user_message(
                    UserMessageSource::ResponseItem,
                    record.event_index,
                    turn_id.as_deref(),
                    item.timestamp.as_deref(),
                    &text,
                )
            {
                return None;
            }
            let user_message_role = (message_kind == CompactionKind::UserMessage)
                .then(|| user_message_state.classify(turn_id.as_deref(), &text));
            if message_kind == CompactionKind::UserMessage {
                user_message_state.observe_emitted_user_message(
                    UserMessageSource::ResponseItem,
                    record.event_index,
                    turn_id.as_deref(),
                    item.timestamp.as_deref(),
                    &text,
                );
            }
            Some(build_row(
                rollout,
                record,
                0,
                message_kind,
                user_message_role,
                turn_id,
                item.timestamp.as_deref(),
                None,
                text,
            ))
        }
        "reasoning" => {
            let text = extract_content_text(item.payload.summary.as_ref())
                .or_else(|| extract_content_text(item.payload.content.as_ref()))
                .or_else(|| {
                    encrypted_placeholder(
                        "encrypted_reasoning",
                        item.payload.encrypted_content.as_deref(),
                    )
                })?;
            Some(build_row(
                rollout,
                record,
                0,
                CompactionKind::Reasoning,
                None,
                turn_id,
                item.timestamp.as_deref(),
                None,
                text,
            ))
        }
        "function_call" | "custom_tool_call" | "web_search_call" => Some(build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolCall,
            None,
            turn_id,
            item.timestamp.as_deref(),
            Some(tool_dedupe_identity(
                kind,
                item.payload.name.as_deref(),
                item.payload.call_id.as_deref(),
            )),
            item.payload
                .arguments
                .clone()
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| {
                    serialize_payload(
                        kind,
                        &item.payload.extra,
                        item.payload.name.as_deref(),
                        item.payload.call_id.as_deref(),
                        None,
                    )
                }),
        )),
        "function_call_output" | "custom_tool_call_output" => Some(build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolOutput,
            None,
            turn_id,
            item.timestamp.as_deref(),
            Some(tool_dedupe_identity(
                kind,
                item.payload.name.as_deref(),
                item.payload.call_id.as_deref(),
            )),
            item.payload
                .output
                .clone()
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| {
                    serialize_payload(
                        kind,
                        &item.payload.extra,
                        item.payload.name.as_deref(),
                        item.payload.call_id.as_deref(),
                        None,
                    )
                }),
        )),
        _ => Some(build_row(
            rollout,
            record,
            0,
            CompactionKind::Unknown,
            None,
            turn_id,
            item.timestamp.as_deref(),
            None,
            serialize_payload(
                kind,
                &item.payload.extra,
                item.payload.name.as_deref(),
                item.payload.call_id.as_deref(),
                item.payload.role.as_deref(),
            ),
        )),
    }
}

fn normalize_unknown_record(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    unknown: &RolloutUnknown,
    turn_id: Option<String>,
    user_message_state: &mut UserMessageState,
) -> Vec<CompactionRow> {
    let mut rows = Vec::new();

    if unknown.record_type == "turn_context" {
        if let Some(turn_id) = turn_id.as_deref() {
            user_message_state.observe_turn_context(turn_id);
        }
        if let Some(text) = unknown
            .payload
            .get("user_instructions")
            .and_then(Value::as_str)
            .and_then(non_empty)
        {
            rows.push(build_row(
                rollout,
                record,
                0,
                CompactionKind::SystemMessage,
                None,
                turn_id.clone(),
                unknown.timestamp.as_deref(),
                None,
                text.to_string(),
            ));
        }
    }

    rows.push(build_row(
        rollout,
        record,
        rows.len(),
        CompactionKind::Unknown,
        None,
        turn_id,
        unknown.timestamp.as_deref(),
        Some(format!("unknown:{}", unknown.record_type)),
        serialize_unknown_record(unknown),
    ));

    rows
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
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::Error,
        user_message_role: None,
        dedupe_identity: None,
        text: failure.error.clone(),
        canonical_text,
        text_hash_hex,
    }
}

fn build_row(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    row_ordinal: usize,
    kind: CompactionKind,
    user_message_role: Option<UserMessageRole>,
    turn_id: Option<String>,
    timestamp: Option<&str>,
    dedupe_identity: Option<String>,
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
        row_ordinal,
        timestamp: parse_timestamp(timestamp),
        kind,
        user_message_role,
        dedupe_identity,
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

#[derive(Debug, Clone, Default)]
struct UserMessageState {
    current_turn_id: Option<String>,
    boundary_seen: bool,
    task_active: bool,
    task_prompt_emitted: bool,
    last_emitted_user_message: Option<EmittedUserMessage>,
}

impl UserMessageState {
    fn observe_turn_id(&mut self, turn_id: &str) {
        if self.current_turn_id.as_deref() != Some(turn_id) {
            self.current_turn_id = Some(turn_id.to_string());
            self.boundary_seen = false;
            self.last_emitted_user_message = None;
        }
    }

    fn observe_turn_context(&mut self, turn_id: &str) {
        self.observe_turn_id(turn_id);
        self.boundary_seen = true;
    }

    fn observe_task_started(&mut self, turn_id: &str) {
        self.observe_turn_id(turn_id);
        self.task_active = true;
        self.task_prompt_emitted = false;
    }

    fn observe_task_started_without_turn(&mut self) {
        self.task_active = true;
        self.task_prompt_emitted = false;
    }

    fn observe_task_complete(&mut self, turn_id: &str) {
        self.observe_turn_id(turn_id);
        self.task_active = false;
        self.task_prompt_emitted = false;
    }

    fn observe_task_complete_without_turn(&mut self) {
        self.task_active = false;
        self.task_prompt_emitted = false;
    }

    fn classify(&mut self, turn_id: Option<&str>, text: &str) -> UserMessageRole {
        let Some(turn_id) = turn_id else {
            return UserMessageRole::Unknown;
        };
        self.observe_turn_id(turn_id);
        if !self.boundary_seen {
            return UserMessageRole::Unknown;
        }
        if is_synthetic_user_message(text) {
            return UserMessageRole::Unknown;
        }
        if !self.task_active {
            self.task_active = true;
            self.task_prompt_emitted = true;
            return UserMessageRole::Prompt;
        }
        if !self.task_prompt_emitted {
            self.task_prompt_emitted = true;
            return UserMessageRole::Prompt;
        }
        UserMessageRole::Steer
    }

    fn observe_emitted_user_message(
        &mut self,
        source: UserMessageSource,
        event_index: usize,
        turn_id: Option<&str>,
        timestamp: Option<&str>,
        text: &str,
    ) {
        self.last_emitted_user_message = Some(EmittedUserMessage {
            source,
            event_index,
            turn_id: turn_id.map(ToOwned::to_owned),
            timestamp: timestamp.map(ToOwned::to_owned),
            text: text.to_string(),
        });
    }

    fn suppress_mirrored_user_message(
        &self,
        source: UserMessageSource,
        event_index: usize,
        turn_id: Option<&str>,
        timestamp: Option<&str>,
        text: &str,
    ) -> bool {
        let Some(last) = &self.last_emitted_user_message else {
            return false;
        };
        last.source != source
            && last.event_index.abs_diff(event_index) <= 1
            && last.turn_id.as_deref() == turn_id
            && timestamps_close(last.timestamp.as_deref(), timestamp)
            && last.text == text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UserMessageSource {
    EventMessage,
    ResponseItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EmittedUserMessage {
    source: UserMessageSource,
    event_index: usize,
    turn_id: Option<String>,
    timestamp: Option<String>,
    text: String,
}

fn is_synthetic_user_message(text: &str) -> bool {
    text.contains("AGENTS.md instructions")
        || text.contains("<skill>")
        || text.contains("Available skills")
}

fn timestamps_close(left: Option<&str>, right: Option<&str>) -> bool {
    match (parse_timestamp(left), parse_timestamp(right)) {
        (Some(left), Some(right)) => (left - right).whole_milliseconds().abs() <= 1_000,
        _ => left == right,
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

fn serialize_unknown_record(unknown: &RolloutUnknown) -> String {
    let mut object = Map::new();
    object.insert(
        "type".to_string(),
        Value::String(unknown.record_type.clone()),
    );
    object.insert("payload".to_string(), unknown.payload.clone());
    Value::Object(object).to_string()
}

fn tool_dedupe_identity(kind: &str, name: Option<&str>, call_id: Option<&str>) -> String {
    let mut object = Map::new();
    object.insert("type".to_string(), Value::String(kind.to_string()));
    if let Some(name) = name.and_then(non_empty) {
        object.insert("name".to_string(), Value::String(name.to_string()));
    }
    if let Some(call_id) = call_id.and_then(non_empty) {
        object.insert("call_id".to_string(), Value::String(call_id.to_string()));
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
