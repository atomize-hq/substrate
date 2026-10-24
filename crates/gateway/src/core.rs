use crate::models::response_transport::ToolDefinition;
use crate::models::{ContentBlock, Message, SystemPrompt};
use bytes::Bytes;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

/// Client-agnostic request shape used by routing and providers.
///
/// Anthropic Messages and other public API shapes should convert into this type
/// rather than driving core behavior directly.
#[derive(Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    pub model: String,
    /// Set by the public adapter, never by caller metadata or deserialization.
    #[serde(skip)]
    pub responses_caller: bool,
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_metadata: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_context: Option<String>,
    pub max_output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

/// Reasoning configuration used by the gateway core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub r#type: String, // "enabled" or "disabled"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_tokens: Option<u32>,
}

/// Gateway response that maintains Anthropic content-block semantics internally,
/// but is not tied to any specific client API surface.
#[derive(Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_turn: Option<bool>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: GatewayUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

/// Streaming response used by the gateway core. The bytes are currently Anthropic SSE.
pub struct GatewayStreamResponse {
    pub stream:
        Pin<Box<dyn Stream<Item = Result<Bytes, crate::providers::error::ProviderError>> + Send>>,
    pub headers: HashMap<String, String>,
}

impl TryFrom<crate::models::AnthropicMessagesRequest> for GatewayRequest {
    type Error = String;

    fn try_from(value: crate::models::AnthropicMessagesRequest) -> Result<Self, Self::Error> {
        for message in &value.messages {
            if let crate::models::MessageContent::Blocks(blocks) = &message.content {
                for block in blocks {
                    if let ContentBlock::Unknown(v) = block {
                        if v.get("transport").is_some()
                            || v.get("type").and_then(|v| v.as_str()).is_some_and(|name| {
                                matches!(
                                    name,
                                    "reasoning"
                                        | "function_call"
                                        | "function_call_output"
                                        | "custom_tool_call"
                                        | "custom_tool_call_output"
                                        | "tool_search_call"
                                        | "tool_search_output"
                                        | "compaction"
                                        | "context_compaction"
                                        | "compaction_summary"
                                        | "configuration_update"
                                        | "additional_tools"
                                        | "agent_message"
                                        | "gateway_response_item"
                                        | "web_search_call"
                                        | "reasoning_text"
                                        | "encrypted_content"
                                )
                            })
                        {
                            return Err("Unsupported advanced content on Messages ingress".into());
                        }
                    }
                    if matches!(
                        block,
                        ContentBlock::Known(crate::models::KnownContentBlock::ToolResult {
                            content: crate::models::ToolResultContent::ResponseItems { .. }
                                | crate::models::ToolResultContent::DiscoveredTools { .. },
                            ..
                        }) | ContentBlock::Known(
                            crate::models::KnownContentBlock::ResponseItem { .. }
                        ) | ContentBlock::Known(crate::models::KnownContentBlock::ToolUse {
                            transport: Some(_),
                            ..
                        }) | ContentBlock::Known(crate::models::KnownContentBlock::ToolResult {
                            transport: Some(_),
                            ..
                        })
                    ) {
                        return Err("Responses transport fields are not Messages fields".into());
                    }
                }
            }
        }
        if value.tools.as_ref().is_some_and(|tools| {
            tools.iter().any(|t| {
                t.r#type.as_deref().is_some_and(|kind| {
                    matches!(kind, "namespace" | "tool_search" | "web_search")
                        || (kind == "custom" && t.input_schema.is_none())
                })
            })
        }) {
            return Err("Responses tools are not Messages declarations".into());
        }
        for message in &value.messages {
            if let crate::models::MessageContent::Blocks(blocks) = &message.content {
                for block in blocks {
                    if let ContentBlock::Known(crate::models::KnownContentBlock::ToolResult {
                        content: crate::models::ToolResultContent::Blocks(parts),
                        ..
                    }) = block
                    {
                        for part in parts {
                            if let crate::models::ToolResultBlock::Unknown(v) = part {
                                if v.get("type").and_then(|v| v.as_str()).is_some_and(|t| {
                                    matches!(
                                        t,
                                        "input_text"
                                            | "output_text"
                                            | "input_image"
                                            | "encrypted_content"
                                            | "input_audio"
                                    )
                                }) {
                                    return Err(
                                        "Responses result fields are not Messages fields".into()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(GatewayRequest {
            model: value.model,
            responses_caller: false,
            instructions: None,
            prompt_cache_key: None,
            client_metadata: None,
            reasoning_context: None,
            messages: value.messages,
            max_output_tokens: value.max_tokens,
            reasoning: value.thinking.map(|t| ReasoningConfig {
                r#type: t.r#type,
                budget_tokens: t.budget_tokens,
            }),
            temperature: value.temperature,
            top_p: value.top_p,
            top_k: value.top_k,
            stop_sequences: value.stop_sequences,
            stream: value.stream,
            metadata: value.metadata,
            system: value.system,
            tools: value
                .tools
                .map(|ts| ts.into_iter().map(Into::into).collect()),
        })
    }
}

impl GatewayRequest {
    /// Whether a request needs the lossless Responses transport and destination pin.
    pub fn requires_responses(&self) -> bool {
        self.metadata.as_ref().is_some_and(|m| {
            m.contains_key("openai_responses_store")
                || m.contains_key("openai_responses_include")
                || m.contains_key("openai_responses_reasoning_summary")
        }) || self.instructions.is_some()
            || self.prompt_cache_key.is_some()
            || self.client_metadata.is_some()
            || self.reasoning_context.is_some()
            || self
                .tools
                .as_ref()
                .is_some_and(|v| v.iter().any(|t| t.legacy().is_err()))
            || self.messages.iter().any(|m| match &m.content {
                crate::models::MessageContent::Blocks(v) => v.iter().any(|b| {
                    matches!(
                        b,
                        ContentBlock::Known(crate::models::KnownContentBlock::Image {
                            source: crate::models::ImageSource {
                                detail: Some(crate::models::ImageDetail::Original),
                                ..
                            }
                        }) | ContentBlock::Known(
                            crate::models::KnownContentBlock::ResponseItem { .. }
                        ) | ContentBlock::Known(crate::models::KnownContentBlock::ToolUse {
                            transport: Some(_),
                            ..
                        }) | ContentBlock::Known(crate::models::KnownContentBlock::ToolResult {
                            transport: Some(_),
                            ..
                        }) | ContentBlock::Known(crate::models::KnownContentBlock::ToolResult {
                            content: crate::models::ToolResultContent::ResponseItems { .. }
                                | crate::models::ToolResultContent::DiscoveredTools { .. },
                            ..
                        })
                    )
                }),
                _ => false,
            })
    }
}

impl std::fmt::Debug for GatewayRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GatewayRequest([payload omitted])")
    }
}

impl std::fmt::Debug for GatewayResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GatewayResponse([payload omitted])")
    }
}
