use crate::models::{ContentBlock, Message, SystemPrompt, Tool};
use bytes::Bytes;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

/// Client-agnostic request shape used by routing and providers.
///
/// Anthropic Messages and other public API shapes should convert into this type
/// rather than driving core behavior directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    pub model: String,
    pub messages: Vec<Message>,
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
    pub tools: Option<Vec<Tool>>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
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
        Ok(GatewayRequest {
            model: value.model,
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
            tools: value.tools,
        })
    }
}
