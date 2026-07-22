use serde_json::{Map, Value};

/// Current native rollout record parsed by the explicit v2 adapter.
#[derive(Debug, Clone, PartialEq)]
pub enum CurrentNativeEvent {
    SessionMeta(CurrentNativeSessionMeta),
    TurnContext(CurrentNativeTurnContext),
    EventMessage(CurrentNativeEventMessage),
    ResponseItem(CurrentNativeResponseItem),
    Unsupported(CurrentNativeUnsupported),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentNativeSessionMeta {
    pub thread_id: String,
    pub session_id: Option<String>,
    pub timestamp: Option<String>,
    pub base_instructions: Option<String>,
    pub child_origin: Option<CurrentNativeChildOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentNativeChildOrigin {
    pub parent_thread_id: String,
    pub depth: u32,
    pub agent_nickname: Option<String>,
    pub agent_role: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurrentNativeTurnContext {
    pub turn_id: String,
    pub timestamp: Option<String>,
    pub user_instructions: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurrentNativeUnsupported {
    pub record_type: String,
    pub item_type: Option<String>,
    pub timestamp: Option<String>,
    pub turn_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurrentNativeEventMessage {
    AgentMessage {
        timestamp: Option<String>,
        turn_id: Option<String>,
        message: String,
    },
    UserMessage {
        timestamp: Option<String>,
        turn_id: String,
        message: String,
    },
    TaskStarted {
        timestamp: Option<String>,
        turn_id: String,
        payload: Value,
    },
    TaskComplete {
        timestamp: Option<String>,
        turn_id: String,
        payload: Value,
    },
    PatchApplyEnd {
        timestamp: Option<String>,
        turn_id: Option<String>,
        payload: Value,
    },
    WebSearchEnd {
        timestamp: Option<String>,
        turn_id: Option<String>,
        payload: Value,
    },
    CollabAgentSpawnBegin {
        timestamp: Option<String>,
        turn_id: Option<String>,
        call_id: String,
        sender_thread_id: String,
        prompt: String,
        payload: Value,
    },
    CollabAgentSpawnEnd {
        timestamp: Option<String>,
        turn_id: Option<String>,
        call_id: String,
        sender_thread_id: String,
        new_thread_id: Option<String>,
        payload: Value,
    },
    SubAgentActivity {
        timestamp: Option<String>,
        turn_id: Option<String>,
        event_id: String,
        agent_thread_id: String,
        activity_kind: String,
        payload: Value,
    },
    TokenCount,
    Unsupported(CurrentNativeUnsupported),
}

impl CurrentNativeEventMessage {
    pub fn turn_id(&self) -> Option<&str> {
        match self {
            Self::AgentMessage { turn_id, .. }
            | Self::PatchApplyEnd { turn_id, .. }
            | Self::WebSearchEnd { turn_id, .. }
            | Self::CollabAgentSpawnBegin { turn_id, .. }
            | Self::CollabAgentSpawnEnd { turn_id, .. }
            | Self::SubAgentActivity { turn_id, .. } => turn_id.as_deref(),
            Self::UserMessage { turn_id, .. }
            | Self::TaskStarted { turn_id, .. }
            | Self::TaskComplete { turn_id, .. } => Some(turn_id),
            Self::TokenCount => None,
            Self::Unsupported(unsupported) => unsupported.turn_id.as_deref(),
        }
    }

    pub fn timestamp(&self) -> Option<&str> {
        match self {
            Self::AgentMessage { timestamp, .. }
            | Self::UserMessage { timestamp, .. }
            | Self::TaskStarted { timestamp, .. }
            | Self::TaskComplete { timestamp, .. }
            | Self::PatchApplyEnd { timestamp, .. }
            | Self::WebSearchEnd { timestamp, .. }
            | Self::CollabAgentSpawnBegin { timestamp, .. }
            | Self::CollabAgentSpawnEnd { timestamp, .. }
            | Self::SubAgentActivity { timestamp, .. } => timestamp.as_deref(),
            Self::TokenCount => None,
            Self::Unsupported(unsupported) => unsupported.timestamp.as_deref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurrentNativeResponseItem {
    Message {
        timestamp: Option<String>,
        turn_id: Option<String>,
        role: String,
        content: Vec<CurrentNativeContentSegment>,
        encrypted_content: Option<String>,
    },
    AgentMessage {
        timestamp: Option<String>,
        turn_id: Option<String>,
        author: String,
        recipient: String,
        content: Vec<CurrentNativeContentSegment>,
    },
    Reasoning {
        timestamp: Option<String>,
        turn_id: Option<String>,
        content: Vec<CurrentNativeContentSegment>,
        encrypted_content: Option<String>,
    },
    ToolCall {
        timestamp: Option<String>,
        turn_id: Option<String>,
        item_type: String,
        name: Option<String>,
        call_id: Option<String>,
        input: String,
    },
    ToolOutput {
        timestamp: Option<String>,
        turn_id: Option<String>,
        item_type: String,
        name: Option<String>,
        call_id: Option<String>,
        output: CurrentNativeToolOutput,
    },
    Unsupported(CurrentNativeUnsupported),
}

impl CurrentNativeResponseItem {
    pub fn turn_id(&self) -> Option<&str> {
        match self {
            Self::Message { turn_id, .. }
            | Self::AgentMessage { turn_id, .. }
            | Self::Reasoning { turn_id, .. }
            | Self::ToolCall { turn_id, .. }
            | Self::ToolOutput { turn_id, .. } => turn_id.as_deref(),
            Self::Unsupported(unsupported) => unsupported.turn_id.as_deref(),
        }
    }

    pub fn timestamp(&self) -> Option<&str> {
        match self {
            Self::Message { timestamp, .. }
            | Self::AgentMessage { timestamp, .. }
            | Self::Reasoning { timestamp, .. }
            | Self::ToolCall { timestamp, .. }
            | Self::ToolOutput { timestamp, .. } => timestamp.as_deref(),
            Self::Unsupported(unsupported) => unsupported.timestamp.as_deref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurrentNativeToolOutput {
    Text(String),
    Segments(Vec<CurrentNativeContentSegment>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurrentNativeContentSegment {
    Text {
        segment_type: String,
        text: String,
    },
    Image {
        image_url: String,
        detail: Option<String>,
    },
    Audio {
        audio_url: String,
    },
    Encrypted,
}

impl CurrentNativeContentSegment {
    pub fn segment_type(&self) -> &str {
        match self {
            Self::Text { segment_type, .. } => segment_type,
            Self::Image { .. } => "input_image",
            Self::Audio { .. } => "input_audio",
            Self::Encrypted => "encrypted_content",
        }
    }

    pub fn text_projection(&self) -> String {
        match self {
            Self::Text { text, .. } => text.clone(),
            Self::Image { image_url, detail } => {
                let mut object = Map::new();
                object.insert("type".to_string(), Value::String("input_image".to_string()));
                object.insert("image_url".to_string(), Value::String(image_url.clone()));
                if let Some(detail) = detail {
                    object.insert("detail".to_string(), Value::String(detail.clone()));
                }
                Value::Object(object).to_string()
            }
            Self::Audio { audio_url } => {
                let mut object = Map::new();
                object.insert("type".to_string(), Value::String("input_audio".to_string()));
                object.insert("audio_url".to_string(), Value::String(audio_url.clone()));
                Value::Object(object).to_string()
            }
            Self::Encrypted => "[encrypted_content]".to_string(),
        }
    }
}

pub(crate) fn current_native_marker(line: &str) -> Result<Option<bool>, String> {
    let value: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    if value.get("type").and_then(Value::as_str) != Some("session_meta") {
        return Ok(None);
    }
    let payload = value
        .get("payload")
        .and_then(Value::as_object)
        .ok_or_else(|| "session_meta.payload must be an object".to_string())?;
    let Some(version) = payload.get("multi_agent_version") else {
        return Ok(Some(false));
    };
    let version = version
        .as_str()
        .ok_or_else(|| "session_meta.multi_agent_version must be a string".to_string())?;
    match version {
        "v2" => Ok(Some(true)),
        "disabled" | "v1" => Ok(Some(false)),
        other => Err(format!(
            "unsupported session_meta.multi_agent_version {other:?}"
        )),
    }
}

pub(crate) fn current_native_record_turn_id(line: &str) -> Option<String> {
    let value: Value = serde_json::from_str(line).ok()?;
    let object = value.as_object()?;
    let record_type = object.get("type")?.as_str()?;
    let payload = object.get("payload")?.as_object()?;
    current_native_payload_turn_id(record_type, payload)
}

pub(crate) fn parse_current_native_line(line: &str) -> Result<Option<CurrentNativeEvent>, String> {
    if line.trim().is_empty() {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(line)
        .map_err(|error| format!("failed to parse current-native Codex rollout JSONL: {error}"))?;
    let object = value.as_object().ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: record must be an object".to_string()
    })?;
    let record_type = required_string(object, "type", "record")?;
    let timestamp = optional_string(object, "timestamp", "record")?;
    let payload = object.get("payload").cloned().ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: record.payload is required".to_string()
    })?;
    let payload_object = payload.as_object().cloned().ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: record.payload must be an object"
            .to_string()
    })?;

    let event = match record_type.as_str() {
        "session_meta" => {
            CurrentNativeEvent::SessionMeta(parse_session_meta(&payload_object, timestamp)?)
        }
        "turn_context" => CurrentNativeEvent::TurnContext(parse_turn_context(
            payload.clone(),
            &payload_object,
            timestamp,
        )?),
        "event_msg" => CurrentNativeEvent::EventMessage(parse_event_message(
            payload.clone(),
            &payload_object,
            timestamp,
        )?),
        "response_item" => CurrentNativeEvent::ResponseItem(parse_response_item(
            payload.clone(),
            &payload_object,
            timestamp,
        )?),
        _ => {
            let turn_id = current_native_payload_turn_id(&record_type, &payload_object);
            CurrentNativeEvent::Unsupported(CurrentNativeUnsupported {
                record_type,
                item_type: None,
                timestamp,
                turn_id,
                payload,
            })
        }
    };
    Ok(Some(event))
}

fn parse_session_meta(
    payload: &Map<String, Value>,
    timestamp: Option<String>,
) -> Result<CurrentNativeSessionMeta, String> {
    let thread_id = required_string(payload, "id", "session_meta.payload")?;
    let session_id = optional_string(payload, "session_id", "session_meta.payload")?;
    let base_instructions = match payload.get("base_instructions") {
        None | Some(Value::Null) => None,
        Some(value) => {
            let object = value.as_object().ok_or_else(|| {
                "failed to parse current-native Codex rollout JSONL: session_meta.payload.base_instructions must be an object".to_string()
            })?;
            optional_string(object, "text", "session_meta.payload.base_instructions")?
        }
    };
    let child_origin = parse_child_origin(payload)?;
    Ok(CurrentNativeSessionMeta {
        thread_id,
        session_id,
        timestamp,
        base_instructions,
        child_origin,
    })
}

fn parse_child_origin(
    payload: &Map<String, Value>,
) -> Result<Option<CurrentNativeChildOrigin>, String> {
    let Some(source) = payload.get("source") else {
        return Ok(None);
    };
    if source.is_null() || source.is_string() {
        return Ok(None);
    }
    let source = source.as_object().ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: session_meta.payload.source must be a string or object".to_string()
    })?;
    let subagent = source.get("subagent").or_else(|| source.get("sub_agent"));
    let Some(subagent) = subagent else {
        return Ok(None);
    };
    if subagent.is_null() || subagent.is_string() {
        return Ok(None);
    }
    let subagent = subagent.as_object().ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: session_meta.payload.source.subagent must be a string or object".to_string()
    })?;
    let Some(thread_spawn) = subagent.get("thread_spawn") else {
        return Ok(None);
    };
    let thread_spawn = thread_spawn.as_object().ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: session_meta.payload.source.subagent.thread_spawn must be an object".to_string()
    })?;
    let parent_thread_id = required_string(
        thread_spawn,
        "parent_thread_id",
        "session_meta.payload.source.subagent.thread_spawn",
    )?;
    if let Some(top_level_parent) =
        optional_string(payload, "parent_thread_id", "session_meta.payload")?
    {
        if top_level_parent != parent_thread_id {
            return Err("failed to parse current-native Codex rollout JSONL: session_meta parent_thread_id conflicts with source.subagent.thread_spawn.parent_thread_id".to_string());
        }
    }
    let depth = required_u32(
        thread_spawn,
        "depth",
        "session_meta.payload.source.subagent.thread_spawn",
    )?;
    Ok(Some(CurrentNativeChildOrigin {
        parent_thread_id,
        depth,
        agent_nickname: optional_string(
            thread_spawn,
            "agent_nickname",
            "session_meta.payload.source.subagent.thread_spawn",
        )?,
        agent_role: optional_string(
            thread_spawn,
            "agent_role",
            "session_meta.payload.source.subagent.thread_spawn",
        )?,
    }))
}

fn parse_turn_context(
    payload: Value,
    object: &Map<String, Value>,
    timestamp: Option<String>,
) -> Result<CurrentNativeTurnContext, String> {
    Ok(CurrentNativeTurnContext {
        turn_id: required_string(object, "turn_id", "turn_context.payload")?,
        timestamp,
        user_instructions: optional_string(object, "user_instructions", "turn_context.payload")?,
        payload,
    })
}

fn parse_event_message(
    payload: Value,
    object: &Map<String, Value>,
    timestamp: Option<String>,
) -> Result<CurrentNativeEventMessage, String> {
    let kind = required_string(object, "type", "event_msg.payload")?;
    let optional_turn = optional_string(object, "turn_id", "event_msg.payload")?;
    match kind.as_str() {
        "agent_message" => Ok(CurrentNativeEventMessage::AgentMessage {
            timestamp,
            turn_id: optional_turn,
            message: required_string(object, "message", "event_msg.payload")?,
        }),
        "user_message" => Ok(CurrentNativeEventMessage::UserMessage {
            timestamp,
            turn_id: required_string(object, "turn_id", "event_msg.payload")?,
            message: required_string(object, "message", "event_msg.payload")?,
        }),
        "task_started" => Ok(CurrentNativeEventMessage::TaskStarted {
            timestamp,
            turn_id: required_string(object, "turn_id", "event_msg.payload")?,
            payload,
        }),
        "task_complete" => Ok(CurrentNativeEventMessage::TaskComplete {
            timestamp,
            turn_id: required_string(object, "turn_id", "event_msg.payload")?,
            payload,
        }),
        "patch_apply_end" => Ok(CurrentNativeEventMessage::PatchApplyEnd {
            timestamp,
            turn_id: optional_turn,
            payload,
        }),
        "web_search_end" => Ok(CurrentNativeEventMessage::WebSearchEnd {
            timestamp,
            turn_id: optional_turn,
            payload,
        }),
        "collab_agent_spawn_begin" => Ok(CurrentNativeEventMessage::CollabAgentSpawnBegin {
            timestamp,
            turn_id: optional_turn,
            call_id: required_string(object, "call_id", "event_msg.payload")?,
            sender_thread_id: required_string(object, "sender_thread_id", "event_msg.payload")?,
            prompt: required_string_allow_empty(object, "prompt", "event_msg.payload")?,
            payload,
        }),
        "collab_agent_spawn_end" => Ok(CurrentNativeEventMessage::CollabAgentSpawnEnd {
            timestamp,
            turn_id: optional_turn,
            call_id: required_string(object, "call_id", "event_msg.payload")?,
            sender_thread_id: required_string(object, "sender_thread_id", "event_msg.payload")?,
            new_thread_id: optional_string(object, "new_thread_id", "event_msg.payload")?,
            payload,
        }),
        "sub_agent_activity" => Ok(CurrentNativeEventMessage::SubAgentActivity {
            timestamp,
            turn_id: optional_turn,
            event_id: required_string(object, "event_id", "event_msg.payload")?,
            agent_thread_id: required_string(object, "agent_thread_id", "event_msg.payload")?,
            activity_kind: required_string(object, "kind", "event_msg.payload")?,
            payload,
        }),
        "token_count" => Ok(CurrentNativeEventMessage::TokenCount),
        _ => Ok(CurrentNativeEventMessage::Unsupported(
            CurrentNativeUnsupported {
                record_type: "event_msg".to_string(),
                item_type: Some(kind),
                timestamp,
                turn_id: optional_turn,
                payload,
            },
        )),
    }
}

fn parse_response_item(
    payload: Value,
    object: &Map<String, Value>,
    timestamp: Option<String>,
) -> Result<CurrentNativeResponseItem, String> {
    let item_type = required_string(object, "type", "response_item.payload")?;
    let turn_id = response_item_turn_id(object)?;
    match item_type.as_str() {
        "message" => Ok(CurrentNativeResponseItem::Message {
            timestamp,
            turn_id,
            role: required_string(object, "role", "response_item.payload")?,
            content: parse_content_array(
                object.get("content"),
                "response_item.payload.content",
                ContentPolicy::Message,
            )?,
            encrypted_content: optional_string(
                object,
                "encrypted_content",
                "response_item.payload",
            )?,
        }),
        "agent_message" => Ok(CurrentNativeResponseItem::AgentMessage {
            timestamp,
            turn_id,
            author: required_string(object, "author", "response_item.payload")?,
            recipient: required_string(object, "recipient", "response_item.payload")?,
            content: parse_content_array(
                object.get("content"),
                "response_item.payload.content",
                ContentPolicy::AgentMessage,
            )?,
        }),
        "reasoning" => {
            let mut content = parse_optional_content_array(
                object.get("summary"),
                "response_item.payload.summary",
                ContentPolicy::Reasoning,
            )?;
            content.extend(parse_optional_content_array(
                object.get("content"),
                "response_item.payload.content",
                ContentPolicy::Reasoning,
            )?);
            Ok(CurrentNativeResponseItem::Reasoning {
                timestamp,
                turn_id,
                content,
                encrypted_content: optional_string(
                    object,
                    "encrypted_content",
                    "response_item.payload",
                )?,
            })
        }
        "function_call" => Ok(CurrentNativeResponseItem::ToolCall {
            timestamp,
            turn_id,
            item_type: item_type.clone(),
            name: Some(required_string(object, "name", "response_item.payload")?),
            call_id: Some(required_string(object, "call_id", "response_item.payload")?),
            input: required_string(object, "arguments", "response_item.payload")?,
        }),
        "custom_tool_call" => Ok(CurrentNativeResponseItem::ToolCall {
            timestamp,
            turn_id,
            item_type: item_type.clone(),
            name: Some(required_string(object, "name", "response_item.payload")?),
            call_id: Some(required_string(object, "call_id", "response_item.payload")?),
            input: required_string(object, "input", "response_item.payload")?,
        }),
        "web_search_call" | "tool_search_call" => Ok(CurrentNativeResponseItem::ToolCall {
            timestamp,
            turn_id,
            item_type: item_type.clone(),
            name: Some(
                optional_string(object, "name", "response_item.payload")?
                    .unwrap_or_else(|| item_type.clone()),
            ),
            call_id: optional_string(object, "call_id", "response_item.payload")?,
            input: canonical_field_or_payload(object, "arguments"),
        }),
        "function_call_output" | "custom_tool_call_output" => {
            Ok(CurrentNativeResponseItem::ToolOutput {
                timestamp,
                turn_id,
                item_type: item_type.clone(),
                name: optional_string(object, "name", "response_item.payload")?,
                call_id: Some(required_string(object, "call_id", "response_item.payload")?),
                output: parse_tool_output(object.get("output"))?,
            })
        }
        "tool_search_output" => Ok(CurrentNativeResponseItem::ToolOutput {
            timestamp,
            turn_id,
            item_type: item_type.clone(),
            name: Some("tool_search_output".to_string()),
            call_id: optional_string(object, "call_id", "response_item.payload")?,
            output: CurrentNativeToolOutput::Text(payload.to_string()),
        }),
        _ => Ok(CurrentNativeResponseItem::Unsupported(
            CurrentNativeUnsupported {
                record_type: "response_item".to_string(),
                item_type: Some(item_type),
                timestamp,
                turn_id,
                payload,
            },
        )),
    }
}

fn current_native_payload_turn_id(
    record_type: &str,
    payload: &Map<String, Value>,
) -> Option<String> {
    if record_type == "response_item" {
        let metadata_turn = payload
            .get("internal_chat_message_metadata_passthrough")
            .and_then(Value::as_object)
            .and_then(|metadata| metadata.get("turn_id"))
            .and_then(Value::as_str)
            .and_then(non_empty)
            .map(ToOwned::to_owned);
        if metadata_turn.is_some() {
            return metadata_turn;
        }
    }

    payload
        .get("turn_id")
        .and_then(Value::as_str)
        .and_then(non_empty)
        .map(ToOwned::to_owned)
}

fn response_item_turn_id(object: &Map<String, Value>) -> Result<Option<String>, String> {
    let Some(metadata) = object.get("internal_chat_message_metadata_passthrough") else {
        return Ok(None);
    };
    if metadata.is_null() {
        return Ok(None);
    }
    let metadata = metadata.as_object().ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: response_item.payload.internal_chat_message_metadata_passthrough must be an object".to_string()
    })?;
    optional_string(
        metadata,
        "turn_id",
        "response_item.payload.internal_chat_message_metadata_passthrough",
    )
}

fn parse_tool_output(value: Option<&Value>) -> Result<CurrentNativeToolOutput, String> {
    let value = value.ok_or_else(|| {
        "failed to parse current-native Codex rollout JSONL: response_item.payload.output is required"
            .to_string()
    })?;
    match value {
        Value::String(text) => Ok(CurrentNativeToolOutput::Text(text.clone())),
        Value::Array(_) => Ok(CurrentNativeToolOutput::Segments(parse_content_array(
            Some(value),
            "response_item.payload.output",
            ContentPolicy::ToolOutput,
        )?)),
        _ => Err("failed to parse current-native Codex rollout JSONL: response_item.payload.output must be a string or typed array".to_string()),
    }
}

#[derive(Debug, Clone, Copy)]
enum ContentPolicy {
    Message,
    AgentMessage,
    Reasoning,
    ToolOutput,
}

fn parse_optional_content_array(
    value: Option<&Value>,
    context: &str,
    policy: ContentPolicy,
) -> Result<Vec<CurrentNativeContentSegment>, String> {
    match value {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(value) => parse_content_array(Some(value), context, policy),
    }
}

fn parse_content_array(
    value: Option<&Value>,
    context: &str,
    policy: ContentPolicy,
) -> Result<Vec<CurrentNativeContentSegment>, String> {
    let array = value.and_then(Value::as_array).ok_or_else(|| {
        format!("failed to parse current-native Codex rollout JSONL: {context} must be an array")
    })?;
    array
        .iter()
        .enumerate()
        .map(|(index, segment)| parse_content_segment(segment, context, index, policy))
        .collect()
}

fn parse_content_segment(
    value: &Value,
    context: &str,
    index: usize,
    policy: ContentPolicy,
) -> Result<CurrentNativeContentSegment, String> {
    let object = value.as_object().ok_or_else(|| {
        format!(
            "failed to parse current-native Codex rollout JSONL: {context}[{index}] must be an object"
        )
    })?;
    let segment_context = format!("{context}[{index}]");
    let segment_type = required_string(object, "type", &segment_context)?;
    let text_type_allowed = match policy {
        ContentPolicy::Message => matches!(segment_type.as_str(), "input_text" | "output_text"),
        ContentPolicy::AgentMessage | ContentPolicy::ToolOutput => segment_type == "input_text",
        ContentPolicy::Reasoning => {
            matches!(segment_type.as_str(), "summary_text" | "reasoning_text")
        }
    };
    if text_type_allowed {
        return Ok(CurrentNativeContentSegment::Text {
            segment_type,
            text: required_string(object, "text", &segment_context)?,
        });
    }

    match segment_type.as_str() {
        "input_image" if matches!(policy, ContentPolicy::Message | ContentPolicy::ToolOutput) => {
            Ok(CurrentNativeContentSegment::Image {
                image_url: required_string(object, "image_url", &segment_context)?,
                detail: optional_string(object, "detail", &segment_context)?,
            })
        }
        "input_audio" if matches!(policy, ContentPolicy::Message | ContentPolicy::ToolOutput) => {
            Ok(CurrentNativeContentSegment::Audio {
                audio_url: required_string(object, "audio_url", &segment_context)?,
            })
        }
        "encrypted_content" => {
            required_string(object, "encrypted_content", &segment_context)?;
            Ok(CurrentNativeContentSegment::Encrypted)
        }
        _ => Err(format!(
            "failed to parse current-native Codex rollout JSONL: {context}[{index}] has unsupported segment type {segment_type:?}"
        )),
    }
}

fn canonical_field_or_payload(object: &Map<String, Value>, field: &str) -> String {
    object
        .get(field)
        .map(Value::to_string)
        .unwrap_or_else(|| Value::Object(object.clone()).to_string())
}

fn required_string(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<String, String> {
    let value = object.get(field).ok_or_else(|| {
        format!("failed to parse current-native Codex rollout JSONL: {context}.{field} is required")
    })?;
    let value = value.as_str().ok_or_else(|| {
        format!(
            "failed to parse current-native Codex rollout JSONL: {context}.{field} must be a string"
        )
    })?;
    if value.trim().is_empty() {
        return Err(format!(
            "failed to parse current-native Codex rollout JSONL: {context}.{field} must not be empty"
        ));
    }
    Ok(value.to_string())
}

fn required_string_allow_empty(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<String, String> {
    let value = object.get(field).ok_or_else(|| {
        format!("failed to parse current-native Codex rollout JSONL: {context}.{field} is required")
    })?;
    value.as_str().map(str::to_string).ok_or_else(|| {
        format!(
            "failed to parse current-native Codex rollout JSONL: {context}.{field} must be a string"
        )
    })
}

fn optional_string(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<Option<String>, String> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let value = value.as_str().ok_or_else(|| {
        format!(
            "failed to parse current-native Codex rollout JSONL: {context}.{field} must be a string or null"
        )
    })?;
    if value.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(value.to_string()))
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

fn required_u32(object: &Map<String, Value>, field: &str, context: &str) -> Result<u32, String> {
    let value = object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            format!(
                "failed to parse current-native Codex rollout JSONL: {context}.{field} must be an unsigned integer"
            )
        })?;
    u32::try_from(value).map_err(|_| {
        format!("failed to parse current-native Codex rollout JSONL: {context}.{field} exceeds u32")
    })
}
