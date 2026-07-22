mod row;

use std::cmp::Ordering;

use codex::{RolloutEvent, RolloutEventMsg, RolloutResponseItem, RolloutUnknown};
use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::canonicalize::canonicalize_row_text;
use crate::ingest::{
    CurrentNativeContentSegment, CurrentNativeEvent, CurrentNativeEventMessage,
    CurrentNativeResponseItem, CurrentNativeToolOutput, CurrentNativeUnsupported,
    IngestedRolloutEvent, IngestedRolloutFile, IngestedRolloutRecord, RolloutParseFailure,
};

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
                IngestedRolloutEvent::Legacy(RolloutEvent::SessionMeta(meta)) => {
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
                IngestedRolloutEvent::Legacy(RolloutEvent::EventMsg(message)) => {
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
                IngestedRolloutEvent::Legacy(RolloutEvent::ResponseItem(item)) => {
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
                IngestedRolloutEvent::Legacy(RolloutEvent::Unknown(unknown)) => {
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
                IngestedRolloutEvent::CurrentNative(event) => rows.extend(
                    normalize_current_native_event(
                        rollout,
                        record,
                        event,
                        &mut current_turn_id,
                        &mut user_message_state,
                    ),
                ),
            },
            NormalizationEntry::ParseFailure(failure) => {
                rows.push(build_failure_row(rollout, failure, current_turn_id.clone()));
            }
        }
    }

    rows
}

fn normalize_current_native_event(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    event: &CurrentNativeEvent,
    current_turn_id: &mut Option<String>,
    user_message_state: &mut UserMessageState,
) -> Vec<CompactionRow> {
    match event {
        CurrentNativeEvent::SessionMeta(meta) => meta
            .base_instructions
            .as_deref()
            .and_then(non_empty)
            .map(|text| {
                vec![build_row(
                    rollout,
                    record,
                    0,
                    CompactionKind::SystemMessage,
                    None,
                    None,
                    meta.timestamp.as_deref(),
                    None,
                    text.to_string(),
                )]
            })
            .unwrap_or_default(),
        CurrentNativeEvent::TurnContext(context) => {
            observe_current_turn(
                current_turn_id,
                user_message_state,
                Some(context.turn_id.as_str()),
            );
            user_message_state.observe_turn_context(&context.turn_id);
            let mut rows = Vec::new();
            if let Some(text) = context
                .user_instructions
                .as_deref()
                .and_then(non_empty)
            {
                rows.push(build_row(
                    rollout,
                    record,
                    rows.len(),
                    CompactionKind::SystemMessage,
                    None,
                    Some(context.turn_id.clone()),
                    context.timestamp.as_deref(),
                    None,
                    text.to_string(),
                ));
            }
            rows.push(build_row(
                rollout,
                record,
                rows.len(),
                CompactionKind::Unknown,
                None,
                Some(context.turn_id.clone()),
                context.timestamp.as_deref(),
                Some("current_native:turn_context".to_string()),
                serialize_current_record("turn_context", &context.payload),
            ));
            rows
        }
        CurrentNativeEvent::EventMessage(message) => {
            observe_current_turn(
                current_turn_id,
                user_message_state,
                message.turn_id(),
            );
            normalize_current_event_message(
                rollout,
                record,
                message,
                current_turn_id.clone(),
                user_message_state,
            )
        }
        CurrentNativeEvent::ResponseItem(item) => {
            observe_current_turn(current_turn_id, user_message_state, item.turn_id());
            normalize_current_response_item(
                rollout,
                record,
                item,
                current_turn_id.clone(),
                user_message_state,
            )
        }
        CurrentNativeEvent::Unsupported(unsupported) => vec![build_current_unsupported_row(
            rollout,
            record,
            unsupported,
            current_turn_id.clone(),
        )],
    }
}

fn observe_current_turn(
    current_turn_id: &mut Option<String>,
    user_message_state: &mut UserMessageState,
    turn_id: Option<&str>,
) {
    if let Some(turn_id) = turn_id.and_then(non_empty) {
        *current_turn_id = Some(turn_id.to_string());
        user_message_state.observe_turn_id(turn_id);
    }
}

fn normalize_current_event_message(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    message: &CurrentNativeEventMessage,
    turn_id: Option<String>,
    user_message_state: &mut UserMessageState,
) -> Vec<CompactionRow> {
    match message {
        CurrentNativeEventMessage::AgentMessage {
            timestamp, message, ..
        } => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::AssistantMessage,
            None,
            turn_id,
            timestamp.as_deref(),
            None,
            message.clone(),
        )],
        CurrentNativeEventMessage::UserMessage {
            timestamp, message, ..
        } => {
            if user_message_state.suppress_mirrored_user_message(
                UserMessageSource::EventMessage,
                record.event_index,
                turn_id.as_deref(),
                timestamp.as_deref(),
                message,
            ) {
                return Vec::new();
            }
            let user_message_role = Some(user_message_state.classify(turn_id.as_deref(), message));
            user_message_state.observe_emitted_user_message(
                UserMessageSource::EventMessage,
                record.event_index,
                turn_id.as_deref(),
                timestamp.as_deref(),
                message,
            );
            vec![build_row(
                rollout,
                record,
                0,
                CompactionKind::UserMessage,
                user_message_role,
                turn_id,
                timestamp.as_deref(),
                None,
                message.clone(),
            )]
        }
        CurrentNativeEventMessage::TaskStarted {
            timestamp, payload, ..
        } => {
            if let Some(turn_id) = turn_id.as_deref() {
                user_message_state.observe_task_started(turn_id);
            } else {
                user_message_state.observe_task_started_without_turn();
            }
            vec![build_row(
                rollout,
                record,
                0,
                CompactionKind::Status,
                None,
                turn_id,
                timestamp.as_deref(),
                Some("current_native:event_msg:task_started".to_string()),
                serialize_current_record("event_msg", payload),
            )]
        }
        CurrentNativeEventMessage::TaskComplete {
            timestamp, payload, ..
        } => {
            if let Some(turn_id) = turn_id.as_deref() {
                user_message_state.observe_task_complete(turn_id);
            } else {
                user_message_state.observe_task_complete_without_turn();
            }
            vec![build_row(
                rollout,
                record,
                0,
                CompactionKind::Status,
                None,
                turn_id,
                timestamp.as_deref(),
                Some("current_native:event_msg:task_complete".to_string()),
                serialize_current_record("event_msg", payload),
            )]
        }
        CurrentNativeEventMessage::PatchApplyEnd {
            timestamp, payload, ..
        } => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolOutput,
            None,
            turn_id,
            timestamp.as_deref(),
            Some("current_native:event_msg:patch_apply_end".to_string()),
            serialize_current_record("event_msg", payload),
        )],
        CurrentNativeEventMessage::WebSearchEnd {
            timestamp, payload, ..
        } => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::Status,
            None,
            turn_id,
            timestamp.as_deref(),
            Some("current_native:event_msg:web_search_end".to_string()),
            serialize_current_record("event_msg", payload),
        )],
        CurrentNativeEventMessage::CollabAgentSpawnBegin {
            timestamp,
            call_id,
            sender_thread_id,
            prompt,
            payload,
            ..
        } => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolCall,
            None,
            turn_id,
            timestamp.as_deref(),
            Some(current_tool_identity(
                "collab_agent_spawn_begin",
                Some("spawn_agent"),
                Some(call_id.as_str()),
                None,
            )),
            if prompt.trim().is_empty() {
                serialize_current_record("event_msg", payload)
            } else {
                let mut object = Map::new();
                object.insert("prompt".to_string(), Value::String(prompt.clone()));
                object.insert(
                    "sender_thread_id".to_string(),
                    Value::String(sender_thread_id.clone()),
                );
                Value::Object(object).to_string()
            },
        )],
        CurrentNativeEventMessage::CollabAgentSpawnEnd {
            timestamp,
            call_id,
            payload,
            ..
        } => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolOutput,
            None,
            turn_id,
            timestamp.as_deref(),
            Some(current_tool_identity(
                "collab_agent_spawn_end",
                Some("spawn_agent"),
                Some(call_id.as_str()),
                None,
            )),
            serialize_current_record("event_msg", payload),
        )],
        CurrentNativeEventMessage::SubAgentActivity {
            timestamp,
            event_id,
            agent_thread_id,
            activity_kind,
            payload,
            ..
        } => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::Status,
            None,
            turn_id,
            timestamp.as_deref(),
            Some(current_activity_identity(
                event_id,
                agent_thread_id,
                activity_kind,
            )),
            serialize_current_record("event_msg", payload),
        )],
        CurrentNativeEventMessage::TokenCount => Vec::new(),
        CurrentNativeEventMessage::Unsupported(unsupported) => vec![
            build_current_unsupported_row(rollout, record, unsupported, turn_id),
        ],
    }
}

fn normalize_current_response_item(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    item: &CurrentNativeResponseItem,
    turn_id: Option<String>,
    user_message_state: &mut UserMessageState,
) -> Vec<CompactionRow> {
    match item {
        CurrentNativeResponseItem::Message {
            timestamp,
            role,
            content,
            encrypted_content,
            ..
        } => {
            let text = render_current_content(content).or_else(|| {
                encrypted_placeholder(
                    "encrypted_message_content",
                    encrypted_content.as_deref(),
                )
            });
            let Some(text) = text else {
                return Vec::new();
            };
            let message_kind = message_role_kind(Some(role.as_str()));
            if message_kind == CompactionKind::UserMessage
                && user_message_state.suppress_mirrored_user_message(
                    UserMessageSource::ResponseItem,
                    record.event_index,
                    turn_id.as_deref(),
                    timestamp.as_deref(),
                    &text,
                )
            {
                return Vec::new();
            }
            let user_message_role = (message_kind == CompactionKind::UserMessage)
                .then(|| user_message_state.classify(turn_id.as_deref(), &text));
            if message_kind == CompactionKind::UserMessage {
                user_message_state.observe_emitted_user_message(
                    UserMessageSource::ResponseItem,
                    record.event_index,
                    turn_id.as_deref(),
                    timestamp.as_deref(),
                    &text,
                );
            }
            vec![build_row(
                rollout,
                record,
                0,
                message_kind,
                user_message_role,
                turn_id,
                timestamp.as_deref(),
                None,
                text,
            )]
        }
        CurrentNativeResponseItem::AgentMessage {
            timestamp,
            author,
            recipient,
            content,
            ..
        } => {
            let text = render_current_content(content)
                .unwrap_or_else(|| "[empty_agent_message]".to_string());
            vec![build_row(
                rollout,
                record,
                0,
                CompactionKind::AssistantMessage,
                None,
                turn_id,
                timestamp.as_deref(),
                Some(current_agent_message_identity(author, recipient, content)),
                text,
            )]
        }
        CurrentNativeResponseItem::Reasoning {
            timestamp,
            content,
            encrypted_content,
            ..
        } => {
            let text = render_current_content(content).or_else(|| {
                encrypted_placeholder("encrypted_reasoning", encrypted_content.as_deref())
            });
            text.map(|text| {
                vec![build_row(
                    rollout,
                    record,
                    0,
                    CompactionKind::Reasoning,
                    None,
                    turn_id,
                    timestamp.as_deref(),
                    None,
                    text,
                )]
            })
            .unwrap_or_default()
        }
        CurrentNativeResponseItem::ToolCall {
            timestamp,
            item_type,
            name,
            call_id,
            input,
            ..
        } => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolCall,
            None,
            turn_id,
            timestamp.as_deref(),
            Some(current_tool_identity(
                item_type,
                name.as_deref(),
                call_id.as_deref(),
                None,
            )),
            input.clone(),
        )],
        CurrentNativeResponseItem::ToolOutput {
            timestamp,
            item_type,
            name,
            call_id,
            output,
            ..
        } => normalize_current_tool_output(
            rollout,
            record,
            timestamp.as_deref(),
            turn_id,
            item_type,
            name.as_deref(),
            call_id.as_deref(),
            output,
        ),
        CurrentNativeResponseItem::Unsupported(unsupported) => vec![
            build_current_unsupported_row(rollout, record, unsupported, turn_id),
        ],
    }
}

#[allow(clippy::too_many_arguments)]
fn normalize_current_tool_output(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    timestamp: Option<&str>,
    turn_id: Option<String>,
    item_type: &str,
    name: Option<&str>,
    call_id: Option<&str>,
    output: &CurrentNativeToolOutput,
) -> Vec<CompactionRow> {
    match output {
        CurrentNativeToolOutput::Text(text) => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolOutput,
            None,
            turn_id,
            timestamp,
            Some(current_tool_identity(item_type, name, call_id, None)),
            text.clone(),
        )],
        CurrentNativeToolOutput::Segments(segments) if segments.is_empty() => vec![build_row(
            rollout,
            record,
            0,
            CompactionKind::ToolOutput,
            None,
            turn_id,
            timestamp,
            Some(current_tool_identity(
                item_type,
                name,
                call_id,
                Some((0, "empty")),
            )),
            "[]".to_string(),
        )],
        CurrentNativeToolOutput::Segments(segments) => segments
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                build_row(
                    rollout,
                    record,
                    index,
                    CompactionKind::ToolOutput,
                    None,
                    turn_id.clone(),
                    timestamp,
                    Some(current_tool_identity(
                        item_type,
                        name,
                        call_id,
                        Some((index, segment.segment_type())),
                    )),
                    segment.text_projection(),
                )
            })
            .collect(),
    }
}

fn render_current_content(content: &[CurrentNativeContentSegment]) -> Option<String> {
    let parts = content
        .iter()
        .map(CurrentNativeContentSegment::text_projection)
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

fn current_agent_message_identity(
    author: &str,
    recipient: &str,
    content: &[CurrentNativeContentSegment],
) -> String {
    let mut object = Map::new();
    object.insert("author".to_string(), Value::String(author.to_string()));
    object.insert(
        "content_types".to_string(),
        Value::Array(
            content
                .iter()
                .map(|segment| Value::String(segment.segment_type().to_string()))
                .collect(),
        ),
    );
    object.insert(
        "recipient".to_string(),
        Value::String(recipient.to_string()),
    );
    object.insert(
        "type".to_string(),
        Value::String("agent_message".to_string()),
    );
    Value::Object(object).to_string()
}

fn current_tool_identity(
    item_type: &str,
    name: Option<&str>,
    call_id: Option<&str>,
    segment: Option<(usize, &str)>,
) -> String {
    let mut object = Map::new();
    object.insert("type".to_string(), Value::String(item_type.to_string()));
    if let Some(name) = name.and_then(non_empty) {
        object.insert("name".to_string(), Value::String(name.to_string()));
    }
    if let Some(call_id) = call_id.and_then(non_empty) {
        object.insert("call_id".to_string(), Value::String(call_id.to_string()));
    }
    if let Some((index, segment_type)) = segment {
        object.insert("segment_index".to_string(), Value::from(index));
        object.insert(
            "segment_type".to_string(),
            Value::String(segment_type.to_string()),
        );
    }
    Value::Object(object).to_string()
}

fn current_activity_identity(event_id: &str, agent_thread_id: &str, kind: &str) -> String {
    let mut object = Map::new();
    object.insert(
        "agent_thread_id".to_string(),
        Value::String(agent_thread_id.to_string()),
    );
    object.insert("event_id".to_string(), Value::String(event_id.to_string()));
    object.insert("kind".to_string(), Value::String(kind.to_string()));
    object.insert(
        "type".to_string(),
        Value::String("sub_agent_activity".to_string()),
    );
    Value::Object(object).to_string()
}

fn build_current_unsupported_row(
    rollout: &IngestedRolloutFile,
    record: &IngestedRolloutRecord,
    unsupported: &CurrentNativeUnsupported,
    turn_id: Option<String>,
) -> CompactionRow {
    let identity = unsupported.item_type.as_deref().map_or_else(
        || format!("current_native:unsupported:{}", unsupported.record_type),
        |item_type| {
            format!(
                "current_native:unsupported:{}:{item_type}",
                unsupported.record_type
            )
        },
    );
    build_row(
        rollout,
        record,
        0,
        CompactionKind::Unknown,
        None,
        turn_id,
        unsupported.timestamp.as_deref(),
        Some(identity),
        serialize_current_record(&unsupported.record_type, &unsupported.payload),
    )
}

fn serialize_current_record(record_type: &str, payload: &Value) -> String {
    let mut object = Map::new();
    object.insert("payload".to_string(), payload.clone());
    object.insert(
        "type".to_string(),
        Value::String(record_type.to_string()),
    );
    Value::Object(object).to_string()
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

#[allow(clippy::too_many_arguments)]
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
