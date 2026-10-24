//! Closed Responses transport types. Transport serialization is deliberately
//! separate from diagnostic projections; native remains the execution owner.
use super::{ContentBlock, KnownContentBlock, Message, MessageContent, Tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt, io};

pub const PAYLOAD_LIMIT: usize = 2 * 1024 * 1024;
pub const WIRE_LIMIT: usize = 8 * PAYLOAD_LIMIT;
// Simultaneous logical payload partitions (not allocator capacity or RSS):
// retained upstream chunk 8B; provider state/prefixes/index metadata/captured model 2B;
// normalized chunk/buffer/single-frame/event 8B; trace retention 2B;
// public state/candidate 4B;
// public render/snapshot temporaries 4B; bounded queued frames 4B.
pub const WORKING_LIMIT: usize = 32 * PAYLOAD_LIMIT;
pub const NORMALIZED_LIMIT: usize = 3 * PAYLOAD_LIMIT;
pub const QUEUE_LIMIT: usize = 4 * PAYLOAD_LIMIT;

/// Checked admission before retaining or extending a payload copy. Each caller
/// has a partition of the aggregate copy ceiling; overflow never wraps.
pub fn bounded_add(used: usize, additional: usize, partition: usize) -> Result<usize, String> {
    used.checked_add(additional)
        .filter(|n| *n <= partition && *n <= WORKING_LIMIT)
        .ok_or_else(|| "Responses payload budget exceeded".into())
}

/// Opaque transport bytes survive serde and cloning, with a redacted Debug view.
///
/// ```
/// use substrate_gateway::models::response_transport::Opaque;
/// let value = Opaque("opaque fixture".into());
/// assert_eq!(serde_json::to_string(&value.clone())?, "\"opaque fixture\"");
/// assert!(!format!("{value:?}").contains("opaque fixture"));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Opaque(pub String);
impl fmt::Debug for Opaque {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[private transport]")
    }
}
impl fmt::Display for Opaque {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[private transport]")
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ToolDefinition {
    /// Endpoint-owned Anthropic declaration. Never admitted as Responses wire input.
    Anthropic { definition: Tool },
    Function {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parameters: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
    },
    Custom {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        format: CustomFormat,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        defer_loading: Option<bool>,
    },
    Namespace {
        name: String,
        description: String,
        tools: Vec<ToolDefinition>,
    },
    ToolSearch {
        execution: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parameters: Option<Value>,
    },
    WebSearch {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        external_web_access: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        indexed_web_access: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filters: Option<SearchFilters>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        user_location: Option<SearchLocation>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        search_content_types: Option<Vec<String>>,
    },
}
impl fmt::Debug for ToolDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ToolDefinition([payload omitted])")
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CustomFormat {
    Text,
    Grammar { syntax: String, definition: String },
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchLocation {
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}
impl From<Tool> for ToolDefinition {
    fn from(definition: Tool) -> Self {
        Self::Anthropic { definition }
    }
}
impl ToolDefinition {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Function { name, .. }
            | Self::Custom { name, .. }
            | Self::Namespace { name, .. } => Some(name),
            Self::Anthropic { definition } => definition.name.as_deref(),
            _ => None,
        }
    }
    pub fn legacy(&self) -> Result<Tool, String> {
        match self {
            Self::Anthropic { definition } => Ok(definition.clone()),
            Self::Function {
                name,
                description,
                parameters,
                strict: None,
                defer_loading: None,
            } => Ok(Tool {
                r#type: None,
                name: Some(name.clone()),
                description: description.clone(),
                input_schema: parameters.clone(),
            }),
            _ => Err("Selected provider cannot represent Responses tools".into()),
        }
    }
    pub fn native(&self) -> Result<Self, String> {
        match self {
            Self::Anthropic { definition } => {
                if definition
                    .r#type
                    .as_deref()
                    .is_some_and(|t| !matches!(t, "function" | "custom"))
                {
                    return Err(
                        "Selected Responses provider cannot represent this endpoint tool".into(),
                    );
                }
                let name = definition
                    .name
                    .clone()
                    .filter(|n| !n.is_empty())
                    .ok_or("Tool name must not be empty")?;
                Ok(Self::Function {
                    name,
                    description: definition.description.clone(),
                    parameters: definition.input_schema.clone(),
                    strict: None,
                    defer_loading: None,
                })
            }
            tool => {
                tool.validate()?;
                Ok(tool.clone())
            }
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Anthropic { .. } => return Err("Unsupported Responses tool definition".into()),
            Self::Function { name, .. }
            | Self::Custom { name, .. }
            | Self::Namespace { name, .. }
                if name.is_empty() =>
            {
                return Err("Tool name must not be empty".into())
            }
            Self::Custom {
                format: CustomFormat::Grammar { syntax, .. },
                ..
            } if syntax != "lark" => return Err("Unsupported custom grammar".into()),
            Self::Namespace { tools, .. } => {
                for t in tools {
                    if !matches!(t, Self::Function { .. } | Self::Custom { .. }) {
                        return Err("Unsupported namespace member".into());
                    }
                    t.validate()?;
                }
            }
            Self::ToolSearch { execution, .. } if execution != "client" => {
                return Err("Unsupported tool search execution".into())
            }
            Self::WebSearch {
                search_context_size: Some(s),
                ..
            } if !matches!(s.as_str(), "low" | "medium" | "high") => {
                return Err("Unsupported search context".into())
            }
            Self::WebSearch {
                user_location: Some(s),
                ..
            } if s.r#type != "approximate" => return Err("Unsupported search location".into()),
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallKind {
    Function,
    Custom,
    ClientSearch,
}
/// Additional identity of the existing tool-use/result block, not a second call.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolTransport {
    pub kind: CallKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default)]
    pub absent_call_id: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_name: Option<String>,
}
impl fmt::Debug for ToolTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ToolTransport([identity omitted])")
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum InputContent {
    InputText {
        text: String,
    },
    OutputText {
        text: String,
    },
    InputImage {
        image_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    EncryptedContent {
        encrypted_content: Opaque,
    },
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResultBody {
    Text(String),
    Items(Vec<InputContent>),
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Summary {
    SummaryText { text: String },
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SearchAction {
    Search {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        query: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        queries: Option<Vec<String>>,
    },
    OpenPage {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
    FindInPage {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
    },
}

/// The admitted complete wire item. This DTO is converted to ordered core blocks
/// before routing. It is never retained alongside an authoritative core copy.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResponseItem {
    Message {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        role: String,
        content: Vec<InputContent>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        phase: Option<String>,
    },
    FunctionCall {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        name: String,
        arguments: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
    },
    CustomToolCall {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        name: String,
        input: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
    },
    ToolSearchCall {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        call_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        execution: String,
        arguments: Value,
    },
    FunctionCallOutput {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        output: ResultBody,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
    },
    CustomToolCallOutput {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        output: ResultBody,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    ToolSearchOutput {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        call_id: String,
        status: String,
        execution: String,
        tools: Vec<ToolDefinition>,
    },
    AdditionalTools {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        role: String,
        tools: Vec<ToolDefinition>,
    },
    Reasoning {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        summary: Vec<Summary>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        encrypted_content: Option<Opaque>,
    },
    Compaction {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        encrypted_content: Opaque,
    },
    ContextCompaction {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(
            default,
            deserialize_with = "strict_some",
            skip_serializing_if = "Option::is_none"
        )]
        encrypted_content: Option<Opaque>,
    },
    WebSearchCall {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<SearchAction>,
    },
}
impl fmt::Debug for ResponseItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ResponseItem([payload omitted])")
    }
}
impl ResponseItem {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Message {
                role,
                content,
                phase,
                ..
            } => {
                if !matches!(role.as_str(), "system" | "developer" | "user" | "assistant") {
                    return Err("Unsupported message role".into());
                }
                if phase
                    .as_ref()
                    .is_some_and(|p| !matches!(p.as_str(), "commentary" | "final_answer"))
                {
                    return Err("Unsupported message phase".into());
                }
                for c in content {
                    c.validate(false)?;
                }
            }
            Self::FunctionCall {
                call_id,
                name,
                arguments,
                ..
            } => {
                if call_id.trim().is_empty() {
                    return Err("function_call.call_id must not be empty".into());
                }
                if name.trim().is_empty() {
                    return Err("function_call.name must not be empty".into());
                }
                serde_json::from_str::<Value>(arguments)
                    .map_err(|_| "function_call.arguments must be a JSON string")?;
            }
            Self::CustomToolCall { call_id, name, .. } => {
                nonempty(call_id)?;
                nonempty(name)?;
            }
            Self::ToolSearchCall {
                execution, call_id, ..
            } => {
                if execution != "client" {
                    return Err("Unsupported search execution".into());
                }
                if let Some(id) = call_id {
                    nonempty(id)?
                }
            }
            Self::FunctionCallOutput {
                call_id, output, ..
            }
            | Self::CustomToolCallOutput {
                call_id, output, ..
            } => {
                nonempty(call_id)?;
                if let ResultBody::Items(items) = output {
                    for c in items {
                        c.validate(true)?;
                    }
                }
            }
            Self::ToolSearchOutput {
                call_id,
                execution,
                tools,
                ..
            } => {
                nonempty(call_id)?;
                if execution != "client" {
                    return Err("Unsupported search execution".into());
                }
                for t in tools {
                    t.validate()?;
                }
            }
            Self::AdditionalTools { role, tools, .. } => {
                if role != "developer" {
                    return Err("additional_tools requires developer role".into());
                }
                for t in tools {
                    t.validate()?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub fn into_message(self) -> Message {
        let role = match &self {
            Self::Message { role, .. } | Self::AdditionalTools { role, .. } => role.clone(),
            Self::FunctionCallOutput { .. }
            | Self::CustomToolCallOutput { .. }
            | Self::ToolSearchOutput { .. } => "user".into(),
            _ => "assistant".into(),
        };
        let block = match self {
            Self::Message {
                id: None,
                phase: None,
                content,
                ..
            } => {
                let blocks = content
                    .into_iter()
                    .map(|c| match c {
                        InputContent::InputText { text } | InputContent::OutputText { text } => {
                            ContentBlock::text(text, None)
                        }
                        InputContent::InputImage { image_url, detail } => {
                            let detail = match detail.as_deref() {
                                Some("auto") => Some(super::ImageDetail::Auto),
                                Some("low") => Some(super::ImageDetail::Low),
                                Some("high") => Some(super::ImageDetail::High),
                                Some("original") => Some(super::ImageDetail::Original),
                                _ => None,
                            };
                            let (kind, media, data, url) = if let Some(data) =
                                image_url.strip_prefix("data:")
                            {
                                let (header, body) =
                                    data.split_once(',').expect("validated image URL");
                                (
                                    "base64",
                                    Some(
                                        header.split(';').next().unwrap_or("image/png").to_string(),
                                    ),
                                    Some(body.to_string()),
                                    None,
                                )
                            } else {
                                ("url", None, None, Some(image_url))
                            };
                            ContentBlock::image(super::ImageSource {
                                r#type: kind.into(),
                                media_type: media,
                                data,
                                url,
                                detail,
                            })
                        }
                        InputContent::EncryptedContent { .. } => unreachable!("validated message"),
                    })
                    .collect();
                return Message {
                    role,
                    content: MessageContent::Blocks(blocks),
                };
            }
            Self::FunctionCall {
                id,
                call_id,
                name,
                namespace,
                arguments,
            } => ContentBlock::Known(KnownContentBlock::ToolUse {
                id: call_id,
                name,
                input: Value::String(arguments),
                transport: Some(ToolTransport {
                    kind: CallKind::Function,
                    item_id: id,
                    namespace,
                    status: None,
                    absent_call_id: false,
                    result_name: None,
                }),
            }),
            Self::CustomToolCall {
                id,
                call_id,
                name,
                namespace,
                input,
                status,
            } => ContentBlock::Known(KnownContentBlock::ToolUse {
                id: call_id,
                name,
                input: Value::String(input),
                transport: Some(ToolTransport {
                    kind: CallKind::Custom,
                    item_id: id,
                    namespace,
                    status,
                    absent_call_id: false,
                    result_name: None,
                }),
            }),
            Self::ToolSearchCall {
                id,
                call_id,
                status,
                arguments,
                ..
            } => ContentBlock::Known(KnownContentBlock::ToolUse {
                id: call_id.clone().unwrap_or_default(),
                name: String::new(),
                input: arguments,
                transport: Some(ToolTransport {
                    kind: CallKind::ClientSearch,
                    item_id: id,
                    namespace: None,
                    status,
                    absent_call_id: call_id.is_none(),
                    result_name: None,
                }),
            }),
            Self::FunctionCallOutput {
                id,
                call_id,
                name,
                namespace,
                output,
            } => result_block(
                CallKind::Function,
                id,
                call_id,
                name,
                namespace,
                None,
                output,
            ),
            Self::CustomToolCallOutput {
                id,
                call_id,
                name,
                output,
            } => result_block(CallKind::Custom, id, call_id, name, None, None, output),
            Self::ToolSearchOutput {
                id,
                call_id,
                status,
                tools,
                ..
            } => ContentBlock::Known(KnownContentBlock::ToolResult {
                tool_use_id: call_id,
                content: super::ToolResultContent::DiscoveredTools { tools },
                is_error: false,
                cache_control: None,
                transport: Some(ToolTransport {
                    kind: CallKind::ClientSearch,
                    item_id: id,
                    namespace: None,
                    status: Some(status),
                    absent_call_id: false,
                    result_name: None,
                }),
            }),
            item => ContentBlock::Known(KnownContentBlock::ResponseItem {
                item: Box::new(item),
            }),
        };
        Message {
            role,
            content: MessageContent::Blocks(vec![block]),
        }
    }
    pub fn from_block(block: &ContentBlock) -> Option<Self> {
        match block {
            ContentBlock::Known(KnownContentBlock::ResponseItem { item }) => Some(*item.clone()),
            ContentBlock::Known(KnownContentBlock::ToolResult {
                tool_use_id,
                content,
                transport: Some(t),
                ..
            }) => {
                let output = match content {
                    super::ToolResultContent::Text(s) => ResultBody::Text(s.clone()),
                    super::ToolResultContent::ResponseItems { items } => {
                        ResultBody::Items(items.clone())
                    }
                    super::ToolResultContent::DiscoveredTools { tools } => {
                        return Some(Self::ToolSearchOutput {
                            id: t.item_id.clone(),
                            call_id: tool_use_id.clone(),
                            execution: "client".into(),
                            status: t.status.clone()?,
                            tools: tools.clone(),
                        })
                    }
                    _ => return None,
                };
                Some(match t.kind {
                    CallKind::Function => Self::FunctionCallOutput {
                        id: t.item_id.clone(),
                        call_id: tool_use_id.clone(),
                        name: t.result_name.clone(),
                        namespace: t.namespace.clone(),
                        output,
                    },
                    CallKind::Custom => Self::CustomToolCallOutput {
                        id: t.item_id.clone(),
                        call_id: tool_use_id.clone(),
                        name: t.result_name.clone(),
                        output,
                    },
                    CallKind::ClientSearch => return None,
                })
            }
            ContentBlock::Known(KnownContentBlock::ToolUse {
                id,
                name,
                input,
                transport: Some(t),
            }) => Some(match t.kind {
                CallKind::Function => Self::FunctionCall {
                    id: t.item_id.clone(),
                    call_id: id.clone(),
                    name: name.clone(),
                    namespace: t.namespace.clone(),
                    arguments: input.as_str()?.into(),
                },
                CallKind::Custom => Self::CustomToolCall {
                    id: t.item_id.clone(),
                    call_id: id.clone(),
                    name: name.clone(),
                    namespace: t.namespace.clone(),
                    input: input.as_str()?.into(),
                    status: t.status.clone(),
                },
                CallKind::ClientSearch => Self::ToolSearchCall {
                    id: t.item_id.clone(),
                    call_id: (!t.absent_call_id).then(|| id.clone()),
                    status: t.status.clone(),
                    execution: "client".into(),
                    arguments: input.clone(),
                },
            }),
            _ => None,
        }
    }
    pub fn call(&self) -> Option<(CallKind, &str, Option<&str>, Option<&str>)> {
        match self {
            Self::FunctionCall {
                call_id,
                name,
                namespace,
                ..
            } => Some((
                CallKind::Function,
                call_id,
                Some(name),
                namespace.as_deref(),
            )),
            Self::CustomToolCall {
                call_id,
                name,
                namespace,
                ..
            } => Some((CallKind::Custom, call_id, Some(name), namespace.as_deref())),
            Self::ToolSearchCall {
                call_id: Some(id), ..
            } => Some((CallKind::ClientSearch, id, None, None)),
            _ => None,
        }
    }
    pub fn result(&self) -> Option<(CallKind, &str, Option<&str>, Option<&str>)> {
        match self {
            Self::FunctionCallOutput {
                call_id,
                name,
                namespace,
                ..
            } => Some((
                CallKind::Function,
                call_id,
                name.as_deref(),
                namespace.as_deref(),
            )),
            Self::CustomToolCallOutput { call_id, name, .. } => {
                Some((CallKind::Custom, call_id, name.as_deref(), None))
            }
            Self::ToolSearchOutput { call_id, .. } => {
                Some((CallKind::ClientSearch, call_id, None, None))
            }
            _ => None,
        }
    }
}
impl InputContent {
    fn validate(&self, result: bool) -> Result<(), String> {
        match self {
            Self::EncryptedContent { .. } if !result => {
                Err("Encrypted message content unsupported".into())
            }
            Self::InputImage {
                detail: Some(d), ..
            } if !matches!(d.as_str(), "auto" | "low" | "high" | "original") => {
                Err("Unsupported input_image.detail".into())
            }
            Self::InputImage { image_url, .. }
                if image_url.starts_with("data:") && !image_url.contains(',') =>
            {
                Err("Invalid data URL".into())
            }
            Self::InputImage { image_url, .. }
                if image_url.starts_with("data:")
                    && image_url.split_once(',').is_some_and(|(header, _)| {
                        !header.ends_with(";base64") || header.matches(';').count() != 1
                    }) =>
            {
                Err("Unsupported inline image data encoding".into())
            }
            Self::InputImage { image_url, .. }
                if !(image_url.starts_with("https://")
                    || image_url.starts_with("http://")
                    || image_url.starts_with("data:image/")) =>
            {
                Err("Unsupported inline image URL".into())
            }
            _ => Ok(()),
        }
    }
}
fn nonempty(s: &str) -> Result<(), String> {
    if s.is_empty() {
        Err("Empty call identity".into())
    } else {
        Ok(())
    }
}

/// Enforces earlier same-kind provenance without rewriting identity or payload.
pub fn validate_history(items: &[ResponseItem], allow_previous: bool) -> Result<(), String> {
    let mut calls: HashMap<&str, &ResponseItem> = HashMap::new();
    for item in items {
        item.validate()?;
        if let Some((_, id, ..)) = item.call() {
            if let Some(old) = calls.get(id) {
                if *old != item {
                    return Err("Conflicting duplicate call".into());
                }
            } else {
                calls.insert(id, item);
            }
        }
        if let Some((kind, id, name, ns)) = item.result() {
            match calls.get(id).and_then(|call| call.call()) {
                Some((k, _, n, s))
                    if k == kind
                        && name.is_none_or(|v| Some(v) == n)
                        && ns.is_none_or(|v| Some(v) == s) => {}
                None if allow_previous && kind == CallKind::Function => {}
                _ => return Err("Tool result lacks matching earlier same-kind call".into()),
            }
        }
    }
    Ok(())
}

/// Counts serialized UTF-8 without allocating a serialized output buffer.
pub fn payload_size<T: Serialize + ?Sized>(value: &T, limit: usize) -> Result<usize, String> {
    struct Count {
        n: usize,
        limit: usize,
    }
    impl io::Write for Count {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| io::Error::other("payload limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count { n: 0, limit };
    serde_json::to_writer(&mut count, value)
        .map_err(|_| "Responses payload limit exceeded".to_string())?;
    Ok(count.n)
}

fn result_block(
    kind: CallKind,
    id: Option<String>,
    call_id: String,
    name: Option<String>,
    namespace: Option<String>,
    status: Option<String>,
    output: ResultBody,
) -> ContentBlock {
    let content = match output {
        ResultBody::Text(s) => super::ToolResultContent::Text(s),
        ResultBody::Items(v) => super::ToolResultContent::ResponseItems { items: v },
    };
    ContentBlock::Known(KnownContentBlock::ToolResult {
        tool_use_id: call_id,
        content,
        is_error: false,
        cache_control: None,
        transport: Some(ToolTransport {
            kind,
            item_id: id,
            namespace,
            status,
            absent_call_id: false,
            result_name: name,
        }),
    })
}

impl fmt::Debug for InputContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InputContent([payload omitted])")
    }
}

fn strict_some<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
