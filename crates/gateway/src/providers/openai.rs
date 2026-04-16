use super::{error::ProviderError, OpenAITransport};
use crate::auth::{
    CodexAuthSource, OAuthClient, OAuthConfig, ResolvedCodexAuthContext, TokenStore,
};
use crate::core::{GatewayRequest, GatewayResponse, GatewayStreamResponse, GatewayUsage};
use crate::models::{
    ContentBlock, CountTokensRequest, CountTokensResponse, ImageSource, KnownContentBlock,
    MessageContent,
};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::providers::streaming::{parse_sse_events, SseEvent, SseStream};

const OPENAI_PARALLEL_TOOL_CALLS_METADATA_KEY: &str = "parallel_tool_calls";
const OPENAI_PUBLIC_RESPONSES_METADATA_KEY: &str = "openai_public_responses";
const OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY: &str = "openai_responses_tool_choice";
const OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY: &str = "openai_responses_reasoning_effort";
const OPENAI_RESPONSES_REASONING_SUMMARY_METADATA_KEY: &str = "openai_responses_reasoning_summary";
const OPENAI_RESPONSES_INCLUDE_METADATA_KEY: &str = "openai_responses_include";
const OPENAI_RESPONSES_TEXT_VERBOSITY_METADATA_KEY: &str = "openai_responses_text_verbosity";
const OPENAI_RESPONSES_EXPLICIT_MAX_OUTPUT_TOKENS_METADATA_KEY: &str =
    "openai_responses_explicit_max_output_tokens";
const OPENAI_RESPONSES_INPUT_METADATA_METADATA_KEY: &str = "openai_responses_input_metadata";
const OPENAI_RESPONSES_TRUNCATION_METADATA_KEY: &str = "openai_responses_truncation";
const OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY: &str =
    "openai_responses_previous_response_id";
const OPENAI_RESPONSES_USER_METADATA_KEY: &str = "openai_responses_user";
const OPENAI_RESPONSES_STREAM_OPTIONS_METADATA_KEY: &str = "openai_responses_stream_options";
const OPENAI_RESPONSES_SERVICE_TIER_METADATA_KEY: &str = "openai_responses_service_tier";

/// Official Codex instructions from OpenAI
/// Source: https://github.com/openai/codex (rust-v0.58.0)
const CODEX_INSTRUCTIONS: &str = include_str!("codex_instructions.md");

/// OpenAI stream_options for requesting usage in streaming responses
#[derive(Debug, Serialize)]
struct OpenAIStreamOptions {
    include_usage: bool,
}

/// OpenAI Chat Completions request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<OpenAIStreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
}

/// OpenAI Responses API request format (for Codex models)
#[derive(Debug, Serialize)]
pub(crate) struct OpenAIResponsesRequest {
    model: String,
    input: OpenAIResponsesInput,
    /// System instructions for the model (required for ChatGPT Codex)
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
    /// Whether to store the conversation (must be false for ChatGPT backend)
    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,
    /// Enable streaming responses
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncation: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_tier: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
    // Note: ChatGPT Codex does NOT support max_output_tokens, max_tokens, temperature, top_p, stop
}

/// Input for Responses API can be string or array of messages
#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum OpenAIResponsesInput {
    Text(String),
    Items(Vec<OpenAIResponsesInputItem>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OpenAIResponsesInputItem {
    #[serde(rename = "message")]
    Message {
        role: String,
        content: Vec<OpenAIResponsesInputContentPart>,
    },
    #[serde(rename = "function_call")]
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    #[serde(rename = "function_call_output")]
    FunctionCallOutput { call_id: String, output: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum OpenAIResponsesInputContentPart {
    #[serde(rename = "input_text")]
    InputText { text: String },
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(rename = "input_image")]
    InputImage {
        image_url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

/// Content can be string or array of content parts
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OpenAIContent {
    String(String),
    Parts(Vec<OpenAIContentPart>),
}

/// Content part (text or image_url)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAIImageUrl },
}

/// Image URL object
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIImageUrl {
    url: String,
}

fn image_url_from_source(source: &ImageSource) -> Option<OpenAIImageUrl> {
    if source.r#type == "base64" {
        let media_type = source.media_type.as_deref().unwrap_or("image/png");
        let data = source.data.as_deref().unwrap_or("");
        Some(OpenAIImageUrl {
            url: format!("data:{};base64,{}", media_type, data),
        })
    } else {
        source
            .url
            .as_ref()
            .map(|url| OpenAIImageUrl { url: url.clone() })
    }
}

fn responses_image_url_from_source(source: &ImageSource) -> Option<String> {
    image_url_from_source(source).map(|image_url| image_url.url)
}

fn system_prompt_to_openai_content(system: &crate::models::SystemPrompt) -> Option<OpenAIContent> {
    match system {
        crate::models::SystemPrompt::Text(text) => Some(OpenAIContent::String(text.clone())),
        crate::models::SystemPrompt::Blocks(blocks) => {
            let mut parts = Vec::new();
            let mut only_text = true;

            for block in blocks {
                match block {
                    ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
                        parts.push(OpenAIContentPart::Text { text: text.clone() });
                    }
                    ContentBlock::Known(KnownContentBlock::Image { source }) => {
                        if let Some(image_url) = image_url_from_source(source) {
                            only_text = false;
                            parts.push(OpenAIContentPart::ImageUrl { image_url });
                        }
                    }
                    _ => {}
                }
            }

            if parts.is_empty() {
                None
            } else if only_text && parts.len() == 1 {
                match parts.into_iter().next() {
                    Some(OpenAIContentPart::Text { text }) => Some(OpenAIContent::String(text)),
                    _ => None,
                }
            } else {
                Some(OpenAIContent::Parts(parts))
            }
        }
    }
}

fn responses_part_from_block(
    block: &ContentBlock,
    role: &str,
) -> Option<OpenAIResponsesInputContentPart> {
    match block {
        ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
            if text.is_empty() {
                None
            } else if role == "assistant" {
                Some(OpenAIResponsesInputContentPart::OutputText { text: text.clone() })
            } else {
                Some(OpenAIResponsesInputContentPart::InputText { text: text.clone() })
            }
        }
        ContentBlock::Known(KnownContentBlock::Image { source }) => {
            responses_image_url_from_source(source).map(|image_url| {
                OpenAIResponsesInputContentPart::InputImage {
                    image_url,
                    detail: None,
                }
            })
        }
        _ => None,
    }
}

fn responses_parts_from_blocks(
    blocks: &[ContentBlock],
    role: &str,
) -> Vec<OpenAIResponsesInputContentPart> {
    let mut parts = Vec::new();

    for block in blocks {
        if let Some(part) = responses_part_from_block(block, role) {
            parts.push(part);
        }
    }

    parts
}

fn system_prompt_to_responses_parts(
    system: &crate::models::SystemPrompt,
) -> Vec<OpenAIResponsesInputContentPart> {
    match system {
        crate::models::SystemPrompt::Text(text) if !text.is_empty() => {
            vec![OpenAIResponsesInputContentPart::InputText { text: text.clone() }]
        }
        crate::models::SystemPrompt::Blocks(blocks) => {
            responses_parts_from_blocks(blocks, "system")
        }
        _ => Vec::new(),
    }
}

/// Tool call in assistant message
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    r#type: String, // "function"
    function: OpenAIFunctionCall,
}

/// Function call details
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String, // JSON string
}

/// Tool definition
#[derive(Debug, Serialize, Deserialize)]
struct OpenAITool {
    r#type: String, // "function"
    function: OpenAIFunctionDef,
}

/// Function definition
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionDef {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<OpenAIContent>,
    #[serde(alias = "reasoning_content")]
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

/// OpenAI Chat Completions response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    #[serde(default, rename = "object")]
    _object: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

/// OpenAI Responses API response format (for Codex models)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIResponsesResponse {
    id: String,
    model: String,
    output: Vec<ResponsesOutput>,
    usage: ResponsesUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResponsesOutput {
    #[serde(rename = "type")]
    output_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Vec<ResponsesContentBlock>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResponsesContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResponsesUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// OpenAI Streaming Chunk (for SSE transformation)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIStreamChunk {
    id: String,
    #[serde(default)]
    model: String,
    choices: Vec<OpenAIStreamChoice>,
    #[serde(default)]
    created: u64,
    /// Usage data (only present in final chunk when stream_options.include_usage=true)
    #[serde(default)]
    usage: Option<OpenAIStreamUsage>,
}

/// Usage data from OpenAI streaming response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIStreamUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    #[serde(default)]
    index: usize,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIStreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    #[serde(alias = "reasoning_content")]
    reasoning: Option<String>, // For GLM/Cerebras models
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<serde_json::Value>>,
}

/// OpenAI-compatible error response (returned by some providers in stream body)
/// Example: {"status_code":500,"error":{"message":"Server error","type":"server_error",...}}
#[derive(Debug, Deserialize)]
struct OpenAIStreamError {
    #[serde(default)]
    status_code: Option<u16>,
    error: OpenAIErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAIErrorDetail {
    message: String,
    #[serde(default)]
    r#type: Option<String>,
}

/// State for OpenAI → Anthropic SSE transformation
///
/// Tracks streaming state across multiple chunks to properly transform
/// OpenAI's incremental tool call format to Anthropic's content block format.
#[derive(Debug, Default)]
struct StreamTransformState {
    /// Has message_start been emitted?
    message_started: bool,
    /// Is a thinking content block currently open?
    thinking_block_open: bool,
    /// The block index assigned to the thinking block (if opened)
    thinking_block_index: u32,
    /// Is a text content block currently open?
    text_block_open: bool,
    /// The block index assigned to the text block (if opened)
    text_block_index: u32,
    /// Tool call indices that have had content_block_start emitted
    /// Maps OpenAI tool_call index → Anthropic content_block index
    tool_blocks: std::collections::HashMap<u32, u32>,
    /// Next available content block index
    next_block_index: u32,
    /// Has finish_reason been received?
    stream_ended: bool,
    /// Did this response include any tool calls? (for correct stop_reason)
    had_tool_calls: bool,
    /// Did the provider emit explicit streamed tool_calls for this response?
    saw_explicit_tool_calls: bool,
    /// Buffered Kimi reasoning_content for hidden-marker reconstruction.
    kimi_reasoning_buffer: String,
    /// Prevent duplicate hidden-marker emission once the buffered section is flushed.
    kimi_hidden_markers_emitted: bool,
}

#[derive(Debug, Clone)]
struct KimiHiddenToolCall {
    id: String,
    name: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Clone)]
struct KimiHiddenMarkerParse {
    prefix: Option<String>,
    tool_calls: Vec<KimiHiddenToolCall>,
}

#[derive(Debug)]
struct KimiNormalization {
    content_blocks: Vec<ContentBlock>,
    had_tool_use: bool,
}

static KIMI_TOOL_SECTION_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<\|tool_calls_section_begin\|>(.*?)<\|tool_calls_section_end\|>")
        .expect("valid Kimi tool section regex")
});

static KIMI_TOOL_CALL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?s)<\|tool_call_begin\|>\s*([^\s<]+)\s*<\|tool_call_argument_begin\|>\s*(.*?)\s*<\|tool_call_end\|>",
    )
    .expect("valid Kimi tool call regex")
});

/// OpenAI provider implementation
pub struct OpenAIProvider {
    name: String,
    api_key: String,
    base_url: String,
    transport: OpenAITransport,
    client: Client,
    models: Vec<String>,
    custom_headers: Vec<(String, String)>,
    /// OAuth provider ID (if using OAuth instead of API key)
    oauth_provider: Option<String>,
    /// Token store for OAuth authentication
    token_store: Option<TokenStore>,
    /// Explicit Codex auth source selected at bootstrap for OAuth-backed Codex routes
    codex_auth_source: Option<CodexAuthSource>,
}

pub(crate) struct OpenAIProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub models: Vec<String>,
    pub custom_headers: Vec<(String, String)>,
    pub oauth_provider: Option<String>,
    pub token_store: Option<TokenStore>,
    pub codex_auth_source: Option<CodexAuthSource>,
}

#[derive(Debug, Clone)]
enum CodexSemanticItem {
    Message {
        text: String,
    },
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
}

#[derive(Debug, Default)]
struct CodexSemanticAssemblyState {
    response_id: Option<String>,
    response_status: Option<String>,
    usage: Option<GatewayUsage>,
    open_items: HashMap<usize, CodexSemanticItem>,
    finalized_items: BTreeMap<usize, CodexSemanticItem>,
    saw_completed: bool,
}

#[derive(Debug, Default)]
struct CodexSemanticStreamState {
    response_id: Option<String>,
    response_status: Option<String>,
    usage: Option<GatewayUsage>,
    message_started: bool,
    saw_completed: bool,
    text_block_started: HashSet<usize>,
    tool_block_started: HashSet<usize>,
    open_items: HashMap<usize, CodexSemanticItem>,
}

fn codex_transport_drift(message: impl Into<String>) -> ProviderError {
    ProviderError::ApiError {
        status: 502,
        message: message.into(),
    }
}

fn flatten_responses_tool_choice(tool_choice: &serde_json::Value) -> serde_json::Value {
    let Some(obj) = tool_choice.as_object() else {
        return tool_choice.clone();
    };
    if obj.get("type").and_then(|value| value.as_str()) != Some("function") {
        return tool_choice.clone();
    }
    let Some(name) = obj
        .get("function")
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
    else {
        return tool_choice.clone();
    };

    serde_json::json!({
        "type": "function",
        "name": name
    })
}

fn codex_semantic_output_index(json: &serde_json::Value) -> Result<usize, ProviderError> {
    json.get("output_index")
        .or_else(|| json.get("index"))
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
        .ok_or_else(|| codex_transport_drift("Codex semantic event missing output index"))
}

fn codex_semantic_item_text(item: &serde_json::Value) -> String {
    item.get("content")
        .and_then(|value| value.as_array())
        .into_iter()
        .flat_map(|parts| parts.iter())
        .filter(|part| part.get("type").and_then(|value| value.as_str()) == Some("output_text"))
        .filter_map(|part| part.get("text").and_then(|value| value.as_str()))
        .collect::<Vec<_>>()
        .join("")
}

fn codex_semantic_item_function_call(item: &serde_json::Value) -> Option<(String, String, String)> {
    let call_id = item.get("call_id").and_then(|value| value.as_str())?;
    let name = item.get("name").and_then(|value| value.as_str())?;
    let arguments = item
        .get("arguments")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    Some((call_id.to_string(), name.to_string(), arguments.to_string()))
}

fn codex_semantic_usage(response: &serde_json::Value) -> GatewayUsage {
    let usage = response.get("usage");
    let input_tokens = usage
        .and_then(|value| value.get("input_tokens"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as u32;
    let output_tokens = usage
        .and_then(|value| value.get("output_tokens"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as u32;

    GatewayUsage {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
    }
}

fn codex_semantic_item_from_value(
    item: &serde_json::Value,
) -> Result<Option<CodexSemanticItem>, ProviderError> {
    let Some(item_type) = item.get("type").and_then(|value| value.as_str()) else {
        return Ok(None);
    };

    match item_type {
        "message" => Ok(Some(CodexSemanticItem::Message {
            text: codex_semantic_item_text(item),
        })),
        "function_call" => {
            let Some((call_id, name, arguments)) = codex_semantic_item_function_call(item) else {
                return Err(codex_transport_drift(
                    "Codex semantic function_call item missing call_id, name, or arguments",
                ));
            };

            Ok(Some(CodexSemanticItem::FunctionCall {
                call_id,
                name,
                arguments,
            }))
        }
        "reasoning" => Ok(None),
        _ => Ok(None),
    }
}

fn codex_semantic_item_to_content_block(
    item: &CodexSemanticItem,
) -> Result<Option<ContentBlock>, ProviderError> {
    match item {
        CodexSemanticItem::Message { text } => Ok(Some(ContentBlock::text(text.clone(), None))),
        CodexSemanticItem::FunctionCall {
            call_id,
            name,
            arguments,
        } => {
            let input = if arguments.trim().is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str(arguments).map_err(|_| {
                    codex_transport_drift(
                        "Codex semantic function_call arguments were not valid JSON",
                    )
                })?
            };

            Ok(Some(ContentBlock::tool_use(
                call_id.clone(),
                name.clone(),
                input,
            )))
        }
    }
}

impl CodexSemanticAssemblyState {
    fn upsert_open_item(
        &mut self,
        output_index: usize,
        item: Option<CodexSemanticItem>,
    ) -> Result<(), ProviderError> {
        if let Some(item) = item {
            self.open_items.insert(output_index, item);
        }
        Ok(())
    }

    fn set_response_metadata(&mut self, response: &serde_json::Value) {
        self.response_id = response
            .get("id")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
            .or_else(|| self.response_id.take());
        self.response_status = response
            .get("status")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());
        self.usage = Some(codex_semantic_usage(response));
    }

    fn finalize_output_index(
        &mut self,
        output_index: usize,
        item_value: Option<&serde_json::Value>,
    ) -> Result<(), ProviderError> {
        if let Some(item) = self.open_items.remove(&output_index) {
            self.finalized_items.insert(output_index, item);
            return Ok(());
        }

        if let Some(item_value) = item_value {
            if let Some(item) = codex_semantic_item_from_value(item_value)? {
                self.finalized_items.insert(output_index, item);
            }
        }

        Ok(())
    }

    fn finalize_pending_items(&mut self) -> Result<(), ProviderError> {
        let pending_indexes = self.open_items.keys().copied().collect::<Vec<_>>();
        for output_index in pending_indexes {
            self.finalize_output_index(output_index, None)?;
        }
        Ok(())
    }

    fn consume_event(&mut self, event: &SseEvent) -> Result<(), ProviderError> {
        let Some(name) = event.event.as_deref() else {
            return Ok(());
        };

        let json: serde_json::Value = serde_json::from_str(&event.data).map_err(|_| {
            codex_transport_drift(format!("Malformed Codex semantic SSE event: {name}"))
        })?;

        match name {
            "response.output_item.added" => {
                let output_index = codex_semantic_output_index(&json)?;
                if let Some(item) = json.get("item") {
                    if let Some(item) = codex_semantic_item_from_value(item)? {
                        self.upsert_open_item(output_index, Some(item))?;
                    }
                }
            }
            "response.content_part.added" => {
                let output_index = codex_semantic_output_index(&json)?;
                if let Some(part) = json.get("part") {
                    if part.get("type").and_then(|value| value.as_str()) == Some("output_text") {
                        let text = part
                            .get("text")
                            .and_then(|value| value.as_str())
                            .unwrap_or_default();
                        match self.open_items.get_mut(&output_index) {
                            Some(CodexSemanticItem::Message { text: current }) => {
                                current.push_str(text);
                            }
                            Some(CodexSemanticItem::FunctionCall { .. }) => {
                                return Err(codex_transport_drift(
                                    "Codex semantic text arrived on a function_call item",
                                ));
                            }
                            None => {
                                self.open_items.insert(
                                    output_index,
                                    CodexSemanticItem::Message {
                                        text: text.to_string(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
            "response.output_text.delta" => {
                let output_index = codex_semantic_output_index(&json)?;
                let delta = json
                    .get("delta")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                match self.open_items.get_mut(&output_index) {
                    Some(CodexSemanticItem::Message { text }) => text.push_str(delta),
                    Some(CodexSemanticItem::FunctionCall { .. }) => {
                        return Err(codex_transport_drift(
                            "Codex semantic output_text.delta arrived on a function_call item",
                        ));
                    }
                    None => {
                        self.open_items.insert(
                            output_index,
                            CodexSemanticItem::Message {
                                text: delta.to_string(),
                            },
                        );
                    }
                }
            }
            "response.output_text.done" => {
                let output_index = codex_semantic_output_index(&json)?;
                let text = json
                    .get("text")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                match self.open_items.get_mut(&output_index) {
                    Some(CodexSemanticItem::Message { text: current }) => {
                        if !text.is_empty() {
                            *current = text.to_string();
                        }
                    }
                    Some(CodexSemanticItem::FunctionCall { .. }) => {
                        return Err(codex_transport_drift(
                            "Codex semantic output_text.done arrived on a function_call item",
                        ));
                    }
                    None => {
                        self.open_items.insert(
                            output_index,
                            CodexSemanticItem::Message {
                                text: text.to_string(),
                            },
                        );
                    }
                }
            }
            "response.function_call_arguments.delta" => {
                let output_index = codex_semantic_output_index(&json)?;
                let delta = json
                    .get("delta")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                match self.open_items.get_mut(&output_index) {
                    Some(CodexSemanticItem::FunctionCall { arguments, .. }) => {
                        arguments.push_str(delta);
                    }
                    Some(CodexSemanticItem::Message { .. }) => {
                        return Err(codex_transport_drift(
                            "Codex semantic function_call_arguments.delta arrived on a message item",
                        ));
                    }
                    None => {
                        return Err(codex_transport_drift(
                            "Codex semantic function_call_arguments.delta arrived before the matching function_call item",
                        ));
                    }
                }
            }
            "response.function_call_arguments.done" => {
                let output_index = codex_semantic_output_index(&json)?;
                let arguments = json
                    .get("arguments")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                match self.open_items.get_mut(&output_index) {
                    Some(CodexSemanticItem::FunctionCall {
                        arguments: current, ..
                    }) => {
                        if !arguments.is_empty() {
                            *current = arguments.to_string();
                        }
                    }
                    Some(CodexSemanticItem::Message { .. }) => {
                        return Err(codex_transport_drift(
                            "Codex semantic function_call_arguments.done arrived on a message item",
                        ));
                    }
                    None => {
                        return Err(codex_transport_drift(
                            "Codex semantic function_call_arguments.done arrived before the matching function_call item",
                        ));
                    }
                }
            }
            "response.output_item.done" => {
                let output_index = codex_semantic_output_index(&json)?;
                self.finalize_output_index(output_index, json.get("item"))?;
            }
            "response.completed" => {
                if let Some(response) = json.get("response") {
                    self.set_response_metadata(response);
                    self.saw_completed = true;
                    self.finalize_pending_items()?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn into_gateway_response(self, model: String) -> Result<GatewayResponse, ProviderError> {
        if !self.saw_completed {
            return Err(codex_transport_drift(
                "Codex semantic SSE stream ended without response.completed",
            ));
        }

        let mut content = Vec::new();
        for item in self.finalized_items.values() {
            if let Some(block) = codex_semantic_item_to_content_block(item)? {
                content.push(block);
            }
        }

        let usage = self.usage.unwrap_or(GatewayUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        });

        Ok(GatewayResponse {
            id: self
                .response_id
                .unwrap_or_else(|| "codex-semantic-response".to_string()),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content,
            model,
            stop_reason: Some(match self.response_status.as_deref() {
                Some("incomplete") => "max_tokens".to_string(),
                _ => "end_turn".to_string(),
            }),
            stop_sequence: None,
            usage,
        })
    }
}

impl CodexSemanticStreamState {
    fn ensure_message_start(&mut self, output: &mut String, model: &str) {
        if self.message_started {
            return;
        }

        self.message_started = true;
        let response_id = self
            .response_id
            .clone()
            .unwrap_or_else(|| format!("codex_{}", uuid::Uuid::new_v4()));
        let message_start = serde_json::json!({
            "type": "message_start",
            "message": {
                "id": response_id,
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": model,
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {
                    "input_tokens": 0,
                    "output_tokens": 0
                }
            }
        });
        OpenAIProvider::push_sse_event(output, "message_start", message_start);
    }

    fn ensure_text_block(&mut self, output: &mut String, output_index: usize) {
        if self.text_block_started.contains(&output_index) {
            return;
        }

        self.text_block_started.insert(output_index);
        OpenAIProvider::push_sse_event(
            output,
            "content_block_start",
            serde_json::json!({
                "type": "content_block_start",
                "index": output_index,
                "content_block": {
                    "type": "text",
                    "text": ""
                }
            }),
        );
    }

    fn ensure_tool_block(
        &mut self,
        output: &mut String,
        output_index: usize,
        call_id: &str,
        name: &str,
    ) {
        if self.tool_block_started.contains(&output_index) {
            return;
        }

        self.tool_block_started.insert(output_index);
        OpenAIProvider::push_sse_event(
            output,
            "content_block_start",
            serde_json::json!({
                "type": "content_block_start",
                "index": output_index,
                "content_block": {
                    "type": "tool_use",
                    "id": call_id,
                    "name": name,
                    "input": {}
                }
            }),
        );
    }

    fn close_text_block(&mut self, output: &mut String, output_index: usize) {
        if self.text_block_started.remove(&output_index) {
            OpenAIProvider::push_sse_event(
                output,
                "content_block_stop",
                serde_json::json!({
                    "type": "content_block_stop",
                    "index": output_index
                }),
            );
        }
    }

    fn close_tool_block(&mut self, output: &mut String, output_index: usize) {
        if self.tool_block_started.remove(&output_index) {
            OpenAIProvider::push_sse_event(
                output,
                "content_block_stop",
                serde_json::json!({
                    "type": "content_block_stop",
                    "index": output_index
                }),
            );
        }
    }

    fn emit_message_stop(&mut self, output: &mut String) {
        let usage = self.usage.clone().unwrap_or(GatewayUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        });
        let message_delta = serde_json::json!({
            "type": "message_delta",
            "delta": {
                "stop_reason": match self.response_status.as_deref() {
                    Some("incomplete") => "max_tokens",
                    _ => "end_turn",
                },
                "stop_sequence": null
            },
            "usage": {
                "input_tokens": usage.input_tokens,
                "output_tokens": usage.output_tokens
            }
        });
        OpenAIProvider::push_sse_event(output, "message_delta", message_delta);
        OpenAIProvider::push_sse_event(
            output,
            "message_stop",
            serde_json::json!({
                "type": "message_stop"
            }),
        );
    }

    fn consume_event(&mut self, event: &SseEvent, model: &str) -> Result<String, ProviderError> {
        let mut output = String::new();
        let Some(name) = event.event.as_deref() else {
            return Ok(output);
        };

        let json: serde_json::Value = serde_json::from_str(&event.data).map_err(|_| {
            codex_transport_drift(format!("Malformed Codex semantic SSE event: {name}"))
        })?;

        match name {
            "response.output_item.added" => {
                let output_index = codex_semantic_output_index(&json)?;
                if let Some(item) = json.get("item") {
                    if let Some(item) = codex_semantic_item_from_value(item)? {
                        self.open_items.insert(output_index, item.clone());
                        match item {
                            CodexSemanticItem::Message { text } => {
                                self.ensure_message_start(&mut output, model);
                                if !text.is_empty() {
                                    self.ensure_text_block(&mut output, output_index);
                                    OpenAIProvider::push_sse_event(
                                        &mut output,
                                        "content_block_delta",
                                        serde_json::json!({
                                            "type": "content_block_delta",
                                            "index": output_index,
                                            "delta": {
                                                "type": "text_delta",
                                                "text": text
                                            }
                                        }),
                                    );
                                }
                            }
                            CodexSemanticItem::FunctionCall {
                                call_id,
                                name,
                                arguments,
                            } => {
                                self.ensure_message_start(&mut output, model);
                                self.ensure_tool_block(&mut output, output_index, &call_id, &name);
                                if !arguments.is_empty() {
                                    OpenAIProvider::push_sse_event(
                                        &mut output,
                                        "content_block_delta",
                                        serde_json::json!({
                                            "type": "content_block_delta",
                                            "index": output_index,
                                            "delta": {
                                                "type": "input_json_delta",
                                                "partial_json": arguments
                                            }
                                        }),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            "response.content_part.added" => {
                let output_index = codex_semantic_output_index(&json)?;
                let Some(part) = json.get("part") else {
                    return Ok(output);
                };
                if part.get("type").and_then(|value| value.as_str()) == Some("output_text") {
                    let text = part
                        .get("text")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default();
                    self.ensure_message_start(&mut output, model);
                    self.ensure_text_block(&mut output, output_index);
                    if !text.is_empty() {
                        OpenAIProvider::push_sse_event(
                            &mut output,
                            "content_block_delta",
                            serde_json::json!({
                                "type": "content_block_delta",
                                "index": output_index,
                                "delta": {
                                    "type": "text_delta",
                                    "text": text
                                }
                            }),
                        );
                    }
                }
            }
            "response.output_text.delta" => {
                let output_index = codex_semantic_output_index(&json)?;
                let delta = json
                    .get("delta")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                self.ensure_message_start(&mut output, model);
                self.ensure_text_block(&mut output, output_index);
                if let Some(CodexSemanticItem::Message { text }) =
                    self.open_items.get_mut(&output_index)
                {
                    text.push_str(delta);
                } else {
                    self.open_items.entry(output_index).or_insert_with(|| {
                        CodexSemanticItem::Message {
                            text: delta.to_string(),
                        }
                    });
                }
                if !delta.is_empty() {
                    OpenAIProvider::push_sse_event(
                        &mut output,
                        "content_block_delta",
                        serde_json::json!({
                            "type": "content_block_delta",
                            "index": output_index,
                            "delta": {
                                "type": "text_delta",
                                "text": delta
                            }
                        }),
                    );
                }
            }
            "response.output_text.done" => {
                let output_index = codex_semantic_output_index(&json)?;
                self.close_text_block(&mut output, output_index);
                if let Some(CodexSemanticItem::Message { text }) =
                    self.open_items.get_mut(&output_index)
                {
                    let done_text = json
                        .get("text")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default();
                    if !done_text.is_empty() {
                        *text = done_text.to_string();
                    }
                }
            }
            "response.function_call_arguments.delta" => {
                let output_index = codex_semantic_output_index(&json)?;
                let delta = json
                    .get("delta")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                let Some(CodexSemanticItem::FunctionCall {
                    call_id,
                    name,
                    arguments,
                }) = self.open_items.get_mut(&output_index)
                else {
                    return Err(codex_transport_drift(
                        "Codex semantic function_call_arguments.delta arrived before the matching function_call item",
                    ));
                };
                let call_id = call_id.clone();
                let name = name.clone();
                arguments.push_str(delta);
                self.ensure_message_start(&mut output, model);
                self.ensure_tool_block(&mut output, output_index, &call_id, &name);
                if !delta.is_empty() {
                    OpenAIProvider::push_sse_event(
                        &mut output,
                        "content_block_delta",
                        serde_json::json!({
                            "type": "content_block_delta",
                            "index": output_index,
                            "delta": {
                                "type": "input_json_delta",
                                "partial_json": delta
                            }
                        }),
                    );
                }
            }
            "response.function_call_arguments.done" => {
                let output_index = codex_semantic_output_index(&json)?;
                let Some(CodexSemanticItem::FunctionCall {
                    call_id,
                    name,
                    arguments,
                }) = self.open_items.get_mut(&output_index)
                else {
                    return Err(codex_transport_drift(
                        "Codex semantic function_call_arguments.done arrived before the matching function_call item",
                    ));
                };
                let done_arguments = json
                    .get("arguments")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                if !done_arguments.is_empty() {
                    *arguments = done_arguments.to_string();
                }
                let call_id = call_id.clone();
                let name = name.clone();
                self.ensure_message_start(&mut output, model);
                self.ensure_tool_block(&mut output, output_index, &call_id, &name);
            }
            "response.output_item.done" => {
                let output_index = codex_semantic_output_index(&json)?;
                if let Some(item) = self.open_items.remove(&output_index) {
                    match item {
                        CodexSemanticItem::Message { .. } => {
                            self.ensure_message_start(&mut output, model);
                            self.close_text_block(&mut output, output_index);
                        }
                        CodexSemanticItem::FunctionCall { .. } => {
                            self.ensure_message_start(&mut output, model);
                            self.close_tool_block(&mut output, output_index);
                        }
                    }
                } else if let Some(item_value) = json.get("item") {
                    if let Some(item) = codex_semantic_item_from_value(item_value)? {
                        self.ensure_message_start(&mut output, model);
                        match item {
                            CodexSemanticItem::Message { .. } => {
                                self.close_text_block(&mut output, output_index);
                            }
                            CodexSemanticItem::FunctionCall {
                                call_id,
                                name,
                                arguments,
                            } => {
                                self.ensure_tool_block(&mut output, output_index, &call_id, &name);
                                if !arguments.is_empty() {
                                    OpenAIProvider::push_sse_event(
                                        &mut output,
                                        "content_block_delta",
                                        serde_json::json!({
                                            "type": "content_block_delta",
                                            "index": output_index,
                                            "delta": {
                                                "type": "input_json_delta",
                                                "partial_json": arguments
                                            }
                                        }),
                                    );
                                }
                                self.close_tool_block(&mut output, output_index);
                            }
                        }
                    }
                }
            }
            "response.completed" => {
                let Some(response) = json.get("response") else {
                    return Err(codex_transport_drift(
                        "Codex semantic response.completed event missing response envelope",
                    ));
                };
                self.ensure_message_start(&mut output, model);
                self.response_id = response
                    .get("id")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string())
                    .or_else(|| self.response_id.take());
                self.response_status = response
                    .get("status")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                self.usage = Some(codex_semantic_usage(response));
                self.saw_completed = true;

                let pending_text = self.text_block_started.iter().copied().collect::<Vec<_>>();
                for output_index in pending_text {
                    self.close_text_block(&mut output, output_index);
                }

                let pending_tools = self.tool_block_started.iter().copied().collect::<Vec<_>>();
                for output_index in pending_tools {
                    self.close_tool_block(&mut output, output_index);
                }

                self.emit_message_stop(&mut output);
            }
            _ => {}
        }

        Ok(output)
    }

    fn finalize(&self, _model: &str) -> Result<String, ProviderError> {
        if !self.saw_completed {
            return Err(codex_transport_drift(
                "Codex semantic SSE stream ended without response.completed",
            ));
        }

        Ok(String::new())
    }
}

impl OpenAIProvider {
    /// Check if the model is a Codex model that requires /v1/responses endpoint
    fn is_codex_model(model: &str) -> bool {
        model.to_lowercase().contains("codex")
    }

    fn is_kimi_model(model: &str) -> bool {
        model.to_lowercase().contains("kimi")
    }

    fn uses_public_responses_api(request: &GatewayRequest) -> bool {
        request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_PUBLIC_RESPONSES_METADATA_KEY))
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    }

    fn request_contains_tool_results(request: &GatewayRequest) -> bool {
        request
            .messages
            .iter()
            .any(|message| match &message.content {
                MessageContent::Blocks(blocks) => blocks.iter().any(|block| {
                    matches!(
                        block,
                        ContentBlock::Known(KnownContentBlock::ToolResult { .. })
                    )
                }),
                MessageContent::Text(_) => false,
            })
    }

    fn should_use_responses_api(&self, request: &GatewayRequest) -> bool {
        if self.is_oauth() {
            return true;
        }

        Self::is_codex_model(&request.model)
            || (Self::uses_public_responses_api(request)
                && Self::request_contains_tool_results(request))
    }

    fn uses_max_completion_tokens(&self, model: &str) -> bool {
        matches!(self.transport, OpenAITransport::AzureOpenAI)
            && model.to_ascii_lowercase().starts_with("gpt-5")
    }

    fn responses_text_part_for_role(role: &str, text: String) -> OpenAIResponsesInputContentPart {
        if role == "assistant" {
            OpenAIResponsesInputContentPart::OutputText { text }
        } else {
            OpenAIResponsesInputContentPart::InputText { text }
        }
    }

    fn flush_responses_message_item(
        items: &mut Vec<OpenAIResponsesInputItem>,
        role: &str,
        content: &mut Vec<OpenAIResponsesInputContentPart>,
    ) {
        if content.is_empty() {
            return;
        }

        items.push(OpenAIResponsesInputItem::Message {
            role: role.to_string(),
            content: std::mem::take(content),
        });
    }

    fn effective_base_url(&self) -> &str {
        if self.is_oauth() && matches!(self.transport, OpenAITransport::OpenAI) {
            "https://chatgpt.com/backend-api"
        } else {
            &self.base_url
        }
    }

    fn normalize_base_url(transport: OpenAITransport, base_url: String) -> String {
        if !matches!(transport, OpenAITransport::AzureOpenAI) {
            return base_url;
        }

        let trimmed = base_url.trim_end_matches('/');
        trimmed
            .strip_suffix("/chat/completions")
            .unwrap_or(trimmed)
            .to_string()
    }

    fn endpoint_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.effective_base_url().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn codex_responses_endpoint(&self) -> &'static str {
        "/codex/responses"
    }

    fn auth_header_parts(&self, auth_value: &str, is_oauth: bool) -> (&'static str, String) {
        if is_oauth {
            ("Authorization", format!("Bearer {}", auth_value))
        } else if matches!(self.transport, OpenAITransport::AzureOpenAI) {
            ("api-key", auth_value.to_string())
        } else {
            ("Authorization", format!("Bearer {}", auth_value))
        }
    }

    fn codex_oauth_request_builder(
        &self,
        req_builder: reqwest::RequestBuilder,
        auth_context: &ResolvedCodexAuthContext,
    ) -> Result<reqwest::RequestBuilder, ProviderError> {
        Ok(req_builder.header("ChatGPT-Account-ID", auth_context.account_id.clone()))
    }

    fn resolve_codex_auth_context(&self) -> Result<ResolvedCodexAuthContext, ProviderError> {
        let source = self.codex_auth_source.as_ref().ok_or_else(|| {
            ProviderError::AuthError(
                "Codex auth source is not configured for the selected OAuth route".to_string(),
            )
        })?;

        source.resolve().map_err(|e| {
            ProviderError::AuthError(format!("Failed to resolve Codex auth context: {}", e))
        })
    }

    fn extract_text_content(content: Option<&OpenAIContent>) -> String {
        match content {
            Some(OpenAIContent::String(text)) => text.clone(),
            Some(OpenAIContent::Parts(parts)) => parts
                .iter()
                .filter_map(|part| match part {
                    OpenAIContentPart::Text { text } => Some(text.clone()),
                    OpenAIContentPart::ImageUrl { .. } => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            None => String::new(),
        }
    }

    fn parse_kimi_hidden_markers(reasoning: &str) -> Option<KimiHiddenMarkerParse> {
        let marker_start = reasoning.find("<|tool_calls_section_begin|>")?;
        let prefix = reasoning[..marker_start].trim();
        let prefix = if prefix.is_empty() {
            None
        } else {
            Some(prefix.to_string())
        };

        let Some(section_caps) = KIMI_TOOL_SECTION_RE.captures(reasoning) else {
            return Some(KimiHiddenMarkerParse {
                prefix,
                tool_calls: Vec::new(),
            });
        };

        let Some(section_match) = section_caps.get(1) else {
            return Some(KimiHiddenMarkerParse {
                prefix,
                tool_calls: Vec::new(),
            });
        };

        let section = section_match.as_str();
        let mut tool_calls = Vec::new();

        for caps in KIMI_TOOL_CALL_RE.captures_iter(section) {
            let raw_id = caps
                .get(1)
                .map(|m| m.as_str().trim())
                .unwrap_or_default()
                .to_string();
            let raw_arguments = caps.get(2).map(|m| m.as_str().trim()).unwrap_or_default();

            let Some(tool_name) = raw_id
                .rsplit_once('.')
                .map(|(_, tail)| tail)
                .unwrap_or(raw_id.as_str())
                .split(':')
                .next()
                .filter(|name| !name.is_empty())
                .map(|name| name.to_string())
            else {
                continue;
            };

            let Ok(arguments) = serde_json::from_str::<serde_json::Value>(raw_arguments) else {
                continue;
            };

            tool_calls.push(KimiHiddenToolCall {
                id: raw_id,
                name: tool_name,
                arguments,
            });
        }

        Some(KimiHiddenMarkerParse { prefix, tool_calls })
    }

    fn normalize_kimi_message(message: &OpenAIMessage) -> Option<KimiNormalization> {
        let mut content_blocks = Vec::new();
        let visible_text = Self::extract_text_content(message.content.as_ref());

        if let Some(tool_calls) = message.tool_calls.as_ref() {
            if !visible_text.trim().is_empty() {
                content_blocks.push(ContentBlock::text(visible_text.trim().to_string(), None));
            }

            for tool_call in tool_calls {
                let input = serde_json::from_str(&tool_call.function.arguments)
                    .unwrap_or_else(|_| serde_json::json!({}));

                content_blocks.push(ContentBlock::tool_use(
                    tool_call.id.clone(),
                    tool_call.function.name.clone(),
                    input,
                ));
            }

            return Some(KimiNormalization {
                had_tool_use: !tool_calls.is_empty(),
                content_blocks,
            });
        }

        if let Some(reasoning) = message.reasoning.as_deref() {
            if let Some(parsed) = Self::parse_kimi_hidden_markers(reasoning) {
                let action_text = if !visible_text.trim().is_empty() {
                    Some(visible_text.trim().to_string())
                } else {
                    parsed.prefix.clone()
                };

                if let Some(action_text) = action_text {
                    if !action_text.is_empty() {
                        content_blocks.push(ContentBlock::text(action_text, None));
                    }
                }

                for tool_call in parsed.tool_calls {
                    content_blocks.push(ContentBlock::tool_use(
                        tool_call.id,
                        tool_call.name,
                        tool_call.arguments,
                    ));
                }

                return Some(KimiNormalization {
                    had_tool_use: content_blocks.iter().any(|block| {
                        matches!(
                            block,
                            ContentBlock::Known(KnownContentBlock::ToolUse { .. })
                        )
                    }),
                    content_blocks,
                });
            }
        }

        if !visible_text.is_empty() {
            content_blocks.push(ContentBlock::text(visible_text, None));
            return Some(KimiNormalization {
                had_tool_use: false,
                content_blocks,
            });
        }

        None
    }

    fn push_sse_event(output: &mut String, event: &str, data: serde_json::Value) {
        output.push_str(&format!("event: {event}\ndata: {data}\n\n"));
    }

    fn close_thinking_block(output: &mut String, state: &mut StreamTransformState) {
        if state.thinking_block_open {
            Self::push_sse_event(
                output,
                "content_block_stop",
                serde_json::json!({
                    "type": "content_block_stop",
                    "index": state.thinking_block_index
                }),
            );
            state.thinking_block_open = false;
        }
    }

    fn close_text_block(output: &mut String, state: &mut StreamTransformState) {
        if state.text_block_open {
            Self::push_sse_event(
                output,
                "content_block_stop",
                serde_json::json!({
                    "type": "content_block_stop",
                    "index": state.text_block_index
                }),
            );
            state.text_block_open = false;
        }
    }

    fn close_tool_blocks(output: &mut String, state: &StreamTransformState) {
        for block_index in state.tool_blocks.values() {
            Self::push_sse_event(
                output,
                "content_block_stop",
                serde_json::json!({
                    "type": "content_block_stop",
                    "index": block_index
                }),
            );
        }
    }

    fn emit_text_delta(output: &mut String, state: &mut StreamTransformState, text: &str) {
        if text.is_empty() {
            return;
        }

        if !state.text_block_open {
            state.text_block_open = true;
            state.text_block_index = state.next_block_index;
            state.next_block_index += 1;
            Self::push_sse_event(
                output,
                "content_block_start",
                serde_json::json!({
                    "type": "content_block_start",
                    "index": state.text_block_index,
                    "content_block": {
                        "type": "text",
                        "text": ""
                    }
                }),
            );
        }

        Self::push_sse_event(
            output,
            "content_block_delta",
            serde_json::json!({
                "type": "content_block_delta",
                "index": state.text_block_index,
                "delta": {
                    "type": "text_delta",
                    "text": text
                }
            }),
        );
    }

    fn emit_hidden_kimi_markers(
        output: &mut String,
        state: &mut StreamTransformState,
        parse: &KimiHiddenMarkerParse,
    ) {
        if state.kimi_hidden_markers_emitted {
            return;
        }

        Self::close_thinking_block(output, state);

        if let Some(prefix) = parse
            .prefix
            .as_deref()
            .map(str::trim)
            .filter(|prefix| !prefix.is_empty())
        {
            Self::emit_text_delta(output, state, prefix);
        }

        if !parse.tool_calls.is_empty() {
            Self::close_text_block(output, state);
        }

        for tool_call in &parse.tool_calls {
            let block_index = state.next_block_index;
            state.next_block_index += 1;
            state.tool_blocks.insert(block_index, block_index);
            state.had_tool_calls = true;

            Self::push_sse_event(
                output,
                "content_block_start",
                serde_json::json!({
                    "type": "content_block_start",
                    "index": block_index,
                    "content_block": {
                        "type": "tool_use",
                        "id": tool_call.id,
                        "name": tool_call.name,
                        "input": tool_call.arguments
                    }
                }),
            );
        }

        state.kimi_hidden_markers_emitted = true;
    }

    /// Parse SSE (Server-Sent Events) response from ChatGPT Codex
    fn parse_sse_response(sse_text: &str) -> Result<Vec<ContentBlock>, ProviderError> {
        // Find the response.completed event and extract both reasoning and message
        let lines: Vec<&str> = sse_text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("event: response.completed") {
                // Next line should be data: {...}
                if i + 1 < lines.len() {
                    let data_line = lines[i + 1];
                    if let Some(json_str) = data_line.strip_prefix("data: ") {
                        // Skip "data: "
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                            // Extract both reasoning and message from response.output array
                            // Note: Codex models have reasoning at output[0], message at output[1]
                            if let Some(response) = json.get("response") {
                                if let Some(output) =
                                    response.get("output").and_then(|v| v.as_array())
                                {
                                    let mut content_blocks = Vec::new();

                                    // Extract reasoning and message in order
                                    for output_item in output {
                                        if let Some(output_type) =
                                            output_item.get("type").and_then(|v| v.as_str())
                                        {
                                            if let Some(content) = output_item
                                                .get("content")
                                                .and_then(|v| v.as_array())
                                            {
                                                if let Some(first_content) = content.first() {
                                                    if let Some(text) = first_content
                                                        .get("text")
                                                        .and_then(|v| v.as_str())
                                                    {
                                                        match output_type {
                                                            "reasoning" => {
                                                                // Unsigned thinking block (no signature field)
                                                                content_blocks.push(
                                                                    ContentBlock::thinking(
                                                                        serde_json::json!({
                                                                            "thinking": text
                                                                        }),
                                                                    ),
                                                                );
                                                            }
                                                            "message" => {
                                                                content_blocks.push(
                                                                    ContentBlock::text(
                                                                        text.to_string(),
                                                                        None,
                                                                    ),
                                                                );
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if !content_blocks.is_empty() {
                                        return Ok(content_blocks);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(ProviderError::ApiError {
            status: 500,
            message: "Failed to parse SSE response: no content found".to_string(),
        })
    }

    /// Transform Anthropic request to OpenAI Responses API format
    pub(crate) fn transform_to_responses_request(
        &self,
        request: &GatewayRequest,
    ) -> Result<OpenAIResponsesRequest, ProviderError> {
        if self.is_oauth() {
            if request.temperature.is_some() {
                return Err(ProviderError::ConfigError(
                    "temperature is not supported on the Codex route".to_string(),
                ));
            }
            if request.top_p.is_some() {
                return Err(ProviderError::ConfigError(
                    "top_p is not supported on the Codex route".to_string(),
                ));
            }
            if request.stop_sequences.is_some() {
                return Err(ProviderError::ConfigError(
                    "stop_sequences is not supported on the Codex route".to_string(),
                ));
            }
            if request.metadata.as_ref().is_some_and(|metadata| {
                metadata.contains_key(OPENAI_RESPONSES_EXPLICIT_MAX_OUTPUT_TOKENS_METADATA_KEY)
            }) {
                return Err(ProviderError::ConfigError(
                    "max_output_tokens is not supported on the Codex route".to_string(),
                ));
            }
            if request.metadata.as_ref().is_some_and(|metadata| {
                metadata.contains_key(OPENAI_RESPONSES_INPUT_METADATA_METADATA_KEY)
            }) {
                return Err(ProviderError::ConfigError(
                    "metadata is not supported on the Codex route".to_string(),
                ));
            }
            if request.metadata.as_ref().is_some_and(|metadata| {
                metadata.contains_key(OPENAI_RESPONSES_TRUNCATION_METADATA_KEY)
            }) {
                return Err(ProviderError::ConfigError(
                    "truncation is not supported on the Codex route".to_string(),
                ));
            }
            if request.metadata.as_ref().is_some_and(|metadata| {
                metadata.contains_key(OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY)
            }) {
                return Err(ProviderError::ConfigError(
                    "previous_response_id is not supported on the Codex route".to_string(),
                ));
            }
            if request
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.contains_key(OPENAI_RESPONSES_USER_METADATA_KEY))
            {
                return Err(ProviderError::ConfigError(
                    "user is not supported on the Codex route".to_string(),
                ));
            }
            if request.metadata.as_ref().is_some_and(|metadata| {
                metadata.contains_key(OPENAI_RESPONSES_STREAM_OPTIONS_METADATA_KEY)
            }) {
                return Err(ProviderError::ConfigError(
                    "stream_options is not supported on the Codex route".to_string(),
                ));
            }
            if request.metadata.as_ref().is_some_and(|metadata| {
                metadata.contains_key(OPENAI_RESPONSES_SERVICE_TIER_METADATA_KEY)
            }) {
                return Err(ProviderError::ConfigError(
                    "service_tier is not supported on the Codex route".to_string(),
                ));
            }
            if let Some(tool_choice) = request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY))
                .and_then(|value| value.as_str())
            {
                if tool_choice == "required" {
                    return Err(ProviderError::ConfigError(
                        "tool_choice=\"required\" is not supported on the Codex route".to_string(),
                    ));
                }
            }
            if let Some(reasoning_effort) = request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY))
                .and_then(|value| value.as_str())
            {
                if !matches!(
                    reasoning_effort,
                    "none" | "minimal" | "low" | "medium" | "high" | "xhigh"
                ) {
                    return Err(ProviderError::ConfigError(
                        "Unsupported reasoning.effort on the Codex route".to_string(),
                    ));
                }
            }
            if let Some(reasoning_summary) = request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(OPENAI_RESPONSES_REASONING_SUMMARY_METADATA_KEY))
                .and_then(|value| value.as_str())
            {
                if !matches!(reasoning_summary, "auto" | "concise" | "detailed" | "none") {
                    return Err(ProviderError::ConfigError(
                        "Unsupported reasoning.summary on the Codex route".to_string(),
                    ));
                }
                let reasoning_enabled = request
                    .metadata
                    .as_ref()
                    .and_then(|metadata| {
                        metadata.get(OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY)
                    })
                    .and_then(|value| value.as_str())
                    .is_some_and(|effort| effort != "none");
                if reasoning_summary != "none" && !reasoning_enabled {
                    return Err(ProviderError::ConfigError(
                        "reasoning.summary requires reasoning.effort to be enabled on the Codex route"
                            .to_string(),
                    ));
                }
            }
            if let Some(include) = request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(OPENAI_RESPONSES_INCLUDE_METADATA_KEY))
                .and_then(|value| value.as_array())
            {
                let include_values: Vec<&str> =
                    include.iter().filter_map(|value| value.as_str()).collect();
                if !(include_values.is_empty()
                    || (include_values.len() == 1
                        && include_values[0] == "reasoning.encrypted_content"))
                {
                    return Err(ProviderError::ConfigError(
                        "Unsupported include value on the Codex route".to_string(),
                    ));
                }
                let reasoning_enabled = request
                    .metadata
                    .as_ref()
                    .and_then(|metadata| {
                        metadata.get(OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY)
                    })
                    .and_then(|value| value.as_str())
                    .is_some_and(|effort| effort != "none");
                if !include_values.is_empty() && !reasoning_enabled {
                    return Err(ProviderError::ConfigError(
                        "include reasoning.encrypted_content requires reasoning.effort to be enabled on the Codex route"
                            .to_string(),
                    ));
                }
            }
            if let Some(verbosity) = request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TEXT_VERBOSITY_METADATA_KEY))
                .and_then(|value| value.as_str())
            {
                if !matches!(verbosity, "low" | "medium" | "high") {
                    return Err(ProviderError::ConfigError(
                        "Unsupported text.verbosity on the Codex route".to_string(),
                    ));
                }
            }
        }

        let allows_previous_response_continuation = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY))
            .and_then(|value| value.as_str())
            .is_some();
        let mut items = Vec::new();
        let mut emitted_call_ids = HashSet::new();
        let mut authoritative_provenance = HashMap::new();

        if let Some(ref system) = request.system {
            let mut content = system_prompt_to_responses_parts(system);
            Self::flush_responses_message_item(&mut items, "system", &mut content);
        }

        for msg in &request.messages {
            match &msg.content {
                MessageContent::Text(text) => {
                    if !text.is_empty() {
                        items.push(OpenAIResponsesInputItem::Message {
                            role: msg.role.clone(),
                            content: vec![Self::responses_text_part_for_role(
                                &msg.role,
                                text.clone(),
                            )],
                        });
                    }
                }
                MessageContent::Blocks(blocks) => {
                    let mut content_parts = Vec::new();

                    for block in blocks {
                        match block {
                            ContentBlock::Known(KnownContentBlock::Text { .. })
                            | ContentBlock::Known(KnownContentBlock::Image { .. }) => {
                                if let Some(part) = responses_part_from_block(block, &msg.role) {
                                    content_parts.push(part);
                                }
                            }
                            ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input }) => {
                                Self::flush_responses_message_item(
                                    &mut items,
                                    &msg.role,
                                    &mut content_parts,
                                );
                                let arguments = serde_json::to_string(input)
                                    .unwrap_or_else(|_| "{}".to_string());
                                authoritative_provenance
                                    .insert(id.clone(), (name.clone(), arguments.clone()));
                                emitted_call_ids.insert(id.clone());
                                items.push(OpenAIResponsesInputItem::FunctionCall {
                                    call_id: id.clone(),
                                    name: name.clone(),
                                    arguments,
                                });
                            }
                            ContentBlock::Known(KnownContentBlock::ToolResult {
                                tool_use_id,
                                content,
                                is_error,
                                ..
                            }) => {
                                Self::flush_responses_message_item(
                                    &mut items,
                                    &msg.role,
                                    &mut content_parts,
                                );
                                if !emitted_call_ids.contains(tool_use_id) {
                                    if let Some((name, arguments)) =
                                        authoritative_provenance.get(tool_use_id).cloned()
                                    {
                                        emitted_call_ids.insert(tool_use_id.clone());
                                        items.push(OpenAIResponsesInputItem::FunctionCall {
                                            call_id: tool_use_id.clone(),
                                            name,
                                            arguments,
                                        });
                                    } else if !allows_previous_response_continuation {
                                        return Err(ProviderError::ConfigError(
                                            "Responses continuation requires authoritative provenance for each function_call_output"
                                                .to_string(),
                                        ));
                                    }
                                }
                                let output = if *is_error {
                                    tracing::debug!(
                                        "🚨 Tool result is_error=true for {}, prefixing content",
                                        tool_use_id
                                    );
                                    format!(
                                        "[SYSTEM: Tools are disabled during warmup. Do NOT call any tools. Wait for the next user message before attempting any tool use.]\n{}",
                                        content
                                    )
                                } else {
                                    content.to_string()
                                };
                                items.push(OpenAIResponsesInputItem::FunctionCallOutput {
                                    call_id: tool_use_id.clone(),
                                    output,
                                });
                            }
                            ContentBlock::Known(KnownContentBlock::Thinking { .. })
                            | ContentBlock::Unknown(_) => {}
                        }
                    }

                    Self::flush_responses_message_item(&mut items, &msg.role, &mut content_parts);
                }
            }
        }

        let tools = request.tools.as_ref().map(|anthropic_tools| {
            anthropic_tools
                .iter()
                .filter_map(|tool| {
                    Some(OpenAITool {
                        r#type: "function".to_string(),
                        function: OpenAIFunctionDef {
                            name: tool.name.as_ref()?.clone(),
                            description: tool.description.clone(),
                            parameters: tool.input_schema.clone(),
                        },
                    })
                })
                .collect()
        });

        let tool_choice = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY))
            .map(flatten_responses_tool_choice);

        let reasoning_effort = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY))
            .and_then(|value| value.as_str());
        let reasoning_summary = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_REASONING_SUMMARY_METADATA_KEY))
            .and_then(|value| value.as_str());
        let reasoning = reasoning_effort.map(|effort| {
            let mut reasoning = serde_json::Map::new();
            reasoning.insert(
                "effort".to_string(),
                serde_json::Value::String(effort.to_string()),
            );
            if let Some(summary) = reasoning_summary {
                reasoning.insert(
                    "summary".to_string(),
                    serde_json::Value::String(summary.to_string()),
                );
            }
            serde_json::Value::Object(reasoning)
        });

        let include = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_INCLUDE_METADATA_KEY))
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|value| value.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
            });

        let text = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TEXT_VERBOSITY_METADATA_KEY))
            .and_then(|value| value.as_str())
            .map(|verbosity| {
                serde_json::json!({
                    "format": { "type": "text" },
                    "verbosity": verbosity
                })
            })
            .or_else(|| {
                self.is_oauth()
                    .then_some(serde_json::json!({ "format": { "type": "text" } }))
            });

        let input_metadata = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_INPUT_METADATA_METADATA_KEY))
            .cloned();

        let truncation = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TRUNCATION_METADATA_KEY))
            .cloned();

        let previous_response_id = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY))
            .and_then(|value| value.as_str())
            .map(ToString::to_string);

        let user = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_USER_METADATA_KEY))
            .and_then(|value| value.as_str())
            .map(ToString::to_string);

        let stream_options = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_STREAM_OPTIONS_METADATA_KEY))
            .cloned();

        let service_tier = request
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_SERVICE_TIER_METADATA_KEY))
            .cloned();

        Ok(OpenAIResponsesRequest {
            model: request.model.clone(),
            input: OpenAIResponsesInput::Items(items),
            instructions: self.is_oauth().then_some(CODEX_INSTRUCTIONS.to_string()),
            store: self.is_oauth().then_some(false),
            stream: true,
            parallel_tool_calls: request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get(OPENAI_PARALLEL_TOOL_CALLS_METADATA_KEY))
                .and_then(|value| value.as_bool()),
            max_output_tokens: (!self.is_oauth()).then_some(request.max_output_tokens),
            metadata: (!self.is_oauth()).then_some(input_metadata).flatten(),
            truncation: (!self.is_oauth()).then_some(truncation).flatten(),
            previous_response_id: (!self.is_oauth()).then_some(previous_response_id).flatten(),
            user: (!self.is_oauth()).then_some(user).flatten(),
            stream_options: (!self.is_oauth()).then_some(stream_options).flatten(),
            service_tier: (!self.is_oauth()).then_some(service_tier).flatten(),
            reasoning,
            include,
            text,
            tools,
            tool_choice,
        })
    }

    pub(crate) fn with_headers(config: OpenAIProviderConfig) -> Self {
        Self::with_transport(OpenAITransport::OpenAI, config)
    }

    pub(crate) fn with_transport(transport: OpenAITransport, config: OpenAIProviderConfig) -> Self {
        let normalized_base_url = Self::normalize_base_url(transport, config.base_url);
        Self {
            name: config.name,
            api_key: config.api_key,
            base_url: normalized_base_url,
            transport,
            client: Client::new(),
            models: config.models,
            custom_headers: config.custom_headers,
            oauth_provider: config.oauth_provider,
            token_store: config.token_store,
            codex_auth_source: config.codex_auth_source,
        }
    }

    /// Get authentication header value (API key or OAuth Bearer token)
    async fn get_auth_header(&self) -> Result<String, ProviderError> {
        // If OAuth provider is configured, use Bearer token
        if let Some(ref oauth_provider_id) = self.oauth_provider {
            if let Some(ref token_store) = self.token_store {
                // Try to get token from store
                if let Some(token) = token_store.get(oauth_provider_id) {
                    // Check if token needs refresh
                    if token.needs_refresh() {
                        tracing::info!(
                            "🔄 Token for '{}' needs refresh, refreshing...",
                            oauth_provider_id
                        );

                        // Refresh token
                        let config = OAuthConfig::openai_codex();
                        let oauth_client = OAuthClient::new(config, token_store.clone());

                        match oauth_client.refresh_token(oauth_provider_id).await {
                            Ok(new_token) => {
                                tracing::info!("✅ Token refreshed successfully");
                                return Ok(new_token.access_token.expose_secret().to_string());
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to refresh token: {}", e);
                                return Err(ProviderError::AuthError(format!(
                                    "Failed to refresh OAuth token: {}",
                                    e
                                )));
                            }
                        }
                    } else {
                        // Token is still valid
                        return Ok(token.access_token.expose_secret().to_string());
                    }
                } else {
                    return Err(ProviderError::AuthError(format!(
                        "OAuth provider '{}' configured but no token found in store",
                        oauth_provider_id
                    )));
                }
            } else {
                return Err(ProviderError::AuthError(
                    "OAuth provider configured but TokenStore not available".to_string(),
                ));
            }
        }

        // Fall back to API key
        Ok(self.api_key.clone())
    }

    /// Check if using OAuth authentication
    fn is_oauth(&self) -> bool {
        self.oauth_provider.is_some()
    }

    /// Extract ChatGPT account ID from JWT access token
    fn extract_account_id(access_token: &str) -> Option<String> {
        // JWT format: header.payload.signature
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        // Decode the payload (base64url)
        let payload = parts[1];
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(payload).ok()?;
        let json_str = String::from_utf8(decoded).ok()?;

        // Parse JSON and extract chatgpt_account_id from the correct claim path
        let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;
        json.get("https://api.openai.com/auth")?
            .get("chatgpt_account_id")?
            .as_str()
            .map(|s| s.to_string())
    }

    /// Transform Anthropic request format to OpenAI Chat Completions format.
    ///
    /// This handles the structural differences between the two APIs:
    ///
    /// # Message Content Transformation
    /// - Anthropic: `content` can be string or array of typed blocks (text, image, tool_use, tool_result)
    /// - OpenAI: `content` can be string or array of parts (text, image_url), with tools in separate fields
    ///
    /// # Key Transformations
    /// - `tool_use` blocks → `tool_calls` array on assistant messages
    /// - `tool_result` blocks → separate `tool` role messages (must come BEFORE user content)
    /// - `image` blocks → `image_url` content parts with data URI encoding
    /// - `thinking` blocks → dropped (OpenAI doesn't support this)
    ///
    /// # Tool Definition Mapping
    /// - Anthropic: `{ name, description, input_schema }`
    /// - OpenAI: `{ type: "function", function: { name, description, parameters } }`
    fn transform_request(&self, request: &GatewayRequest) -> Result<OpenAIRequest, ProviderError> {
        let mut openai_messages = Vec::new();

        // Add system message if present
        if let Some(ref system) = request.system {
            openai_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt_to_openai_content(system),
                reasoning: None,
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Transform messages
        for msg in &request.messages {
            match &msg.content {
                MessageContent::Text(text) => {
                    // Simple text message
                    openai_messages.push(OpenAIMessage {
                        role: msg.role.clone(),
                        content: Some(OpenAIContent::String(text.clone())),
                        reasoning: None,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
                MessageContent::Blocks(blocks) => {
                    // Check if we have any tool results - they need separate messages
                    let tool_results: Vec<_> = blocks.iter()
                        .filter_map(|block| {
                            if let ContentBlock::Known(KnownContentBlock::ToolResult { tool_use_id, content, is_error, .. }) = block {
                                let result_content = if *is_error {
                                    // Prefix error content so models know not to retry
                                    tracing::debug!("🚨 Tool result is_error=true for {}, prefixing content", tool_use_id);
                                    format!("[SYSTEM: Tools are disabled during warmup. Do NOT call any tools. Wait for the next user message before attempting any tool use.]\n{}", content)
                                } else {
                                    content.to_string()
                                };
                                Some((tool_use_id.clone(), result_content))
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Extract tool_calls from ToolUse blocks
                    let tool_calls: Vec<_> = blocks
                        .iter()
                        .filter_map(|block| {
                            if let ContentBlock::Known(KnownContentBlock::ToolUse {
                                id,
                                name,
                                input,
                            }) = block
                            {
                                Some(OpenAIToolCall {
                                    id: id.clone(),
                                    r#type: "function".to_string(),
                                    function: OpenAIFunctionCall {
                                        name: name.clone(),
                                        arguments: serde_json::to_string(input).unwrap_or_default(),
                                    },
                                })
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Build content parts (text and images, excluding tool use/result)
                    let mut content_parts = Vec::new();
                    for block in blocks {
                        match block {
                            ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
                                content_parts.push(OpenAIContentPart::Text { text: text.clone() });
                            }
                            ContentBlock::Known(KnownContentBlock::Image { source }) => {
                                // Convert Anthropic image format to OpenAI format
                                let url = if source.r#type == "base64" {
                                    // data:image/{media_type};base64,{data}
                                    let media_type =
                                        source.media_type.as_deref().unwrap_or("image/png");
                                    let data = source.data.as_deref().unwrap_or("");
                                    format!("data:{};base64,{}", media_type, data)
                                } else if let Some(url) = &source.url {
                                    url.clone()
                                } else {
                                    continue; // Skip invalid image sources
                                };

                                content_parts.push(OpenAIContentPart::ImageUrl {
                                    image_url: OpenAIImageUrl { url },
                                });
                            }
                            ContentBlock::Known(KnownContentBlock::ToolUse { .. }) => {
                                // Already handled in tool_calls
                            }
                            ContentBlock::Known(KnownContentBlock::ToolResult { .. }) => {
                                // Will be handled as separate messages below
                            }
                            ContentBlock::Known(KnownContentBlock::Thinking { .. }) => {
                                // OpenAI doesn't have thinking blocks, skip
                            }
                            ContentBlock::Unknown(_) => {
                                // Unknown content types - skip when converting to OpenAI
                            }
                        }
                    }

                    // OpenAI Message Ordering for Tool Results
                    // ==========================================
                    // OpenAI requires tool response messages to appear BEFORE user content
                    // when a user message contains both tool_results and text content.
                    //
                    // In Anthropic's format, a single user message can contain mixed content:
                    //   { role: "user", content: [tool_result, tool_result, text] }
                    //
                    // OpenAI requires separate messages in this order:
                    //   1. { role: "tool", tool_call_id: "...", content: "..." }  // for each result
                    //   2. { role: "user", content: "..." }  // user's text content
                    //
                    // This is critical for parallel tool calls where the user provides multiple
                    // tool results and then adds additional context or instructions.

                    // Add separate tool result messages FIRST
                    for (tool_use_id, result_content) in tool_results {
                        openai_messages.push(OpenAIMessage {
                            role: "tool".to_string(),
                            content: Some(OpenAIContent::String(result_content)),
                            reasoning: None,
                            tool_calls: None,
                            tool_call_id: Some(tool_use_id),
                        });
                    }

                    // Then add main message with content and/or tool_calls
                    if !content_parts.is_empty() || !tool_calls.is_empty() {
                        let content = if content_parts.is_empty() {
                            None
                        } else if content_parts.len() == 1 {
                            // Single text part - use string format for compatibility
                            if let OpenAIContentPart::Text { text } = &content_parts[0] {
                                Some(OpenAIContent::String(text.clone()))
                            } else {
                                Some(OpenAIContent::Parts(content_parts.clone()))
                            }
                        } else {
                            Some(OpenAIContent::Parts(content_parts))
                        };

                        openai_messages.push(OpenAIMessage {
                            role: msg.role.clone(),
                            content,
                            reasoning: None,
                            tool_calls: if tool_calls.is_empty() {
                                None
                            } else {
                                Some(tool_calls)
                            },
                            tool_call_id: None,
                        });
                    }
                }
            }
        }

        // Transform tools if present
        let tools = request.tools.as_ref().map(|anthropic_tools| {
            anthropic_tools
                .iter()
                .filter_map(|tool| {
                    // Anthropic tools have name, description, input_schema
                    Some(OpenAITool {
                        r#type: "function".to_string(),
                        function: OpenAIFunctionDef {
                            name: tool.name.as_ref()?.clone(),
                            description: tool.description.clone(),
                            parameters: tool.input_schema.clone(),
                        },
                    })
                })
                .collect()
        });

        // Request usage data in streaming responses
        let stream_options = if request.stream == Some(true) {
            Some(OpenAIStreamOptions {
                include_usage: true,
            })
        } else {
            None
        };

        Ok(OpenAIRequest {
            model: request.model.clone(),
            messages: openai_messages,
            max_tokens: (!self.uses_max_completion_tokens(&request.model))
                .then_some(request.max_output_tokens),
            max_completion_tokens: self
                .uses_max_completion_tokens(&request.model)
                .then_some(request.max_output_tokens),
            temperature: request.temperature,
            top_p: request.top_p,
            stop: request.stop_sequences.clone(),
            stream: request.stream,
            stream_options,
            tools,
            tool_choice: None, // TODO: Add tool_choice support if needed
        })
    }

    /// Transform OpenAI Chat Completions response to Anthropic Messages format.
    ///
    /// # Response Structure Mapping
    /// - OpenAI: `{ id, model, choices: [{ message: { content, reasoning, tool_calls }, finish_reason }], usage }`
    /// - Anthropic: `{ id, model, content: [...blocks], stop_reason, usage }`
    ///
    /// # Content Block Mapping
    /// - `message.reasoning` → `thinking` content block (chain-of-thought)
    /// - `message.content` → `text` content block
    /// - `message.tool_calls` → `tool_use` content blocks
    fn transform_response(&self, response: OpenAIResponse) -> GatewayResponse {
        let choice = response
            .choices
            .into_iter()
            .next()
            .expect("OpenAI response must have at least one choice");

        let kimi_normalization = if Self::is_kimi_model(&response.model) {
            Self::normalize_kimi_message(&choice.message)
        } else {
            None
        };
        let mut content_blocks = kimi_normalization
            .as_ref()
            .map(|normalized| normalized.content_blocks.clone())
            .unwrap_or_default();

        if kimi_normalization.is_none() {
            // Add reasoning as thinking block (unsigned — no signature field).
            if let Some(reasoning) = choice.message.reasoning.as_ref() {
                if !reasoning.is_empty() {
                    content_blocks.push(ContentBlock::thinking(serde_json::json!({
                        "thinking": reasoning
                    })));
                }
            }

            let text = Self::extract_text_content(choice.message.content.as_ref());

            if !text.is_empty() {
                content_blocks.push(ContentBlock::text(text, None));
            }

            // Non-streaming Tool Calls Transformation
            // ========================================
            // OpenAI returns tool_calls as an array in the message:
            //   { id: "call_xxx", type: "function", function: { name: "...", arguments: "{...}" } }
            //
            // We transform each to Anthropic's tool_use content block:
            //   { type: "tool_use", id: "...", name: "...", input: {...} }
            //
            // Note: OpenAI's `arguments` is a JSON string that we parse into `input` object.
            if let Some(tool_calls) = choice.message.tool_calls.as_ref() {
                for tool_call in tool_calls {
                    let input = serde_json::from_str(&tool_call.function.arguments)
                        .unwrap_or_else(|_| serde_json::json!({}));

                    content_blocks.push(ContentBlock::tool_use(
                        tool_call.id.clone(),
                        tool_call.function.name.clone(),
                        input,
                    ));
                }
            }
        }

        let had_tool_use = kimi_normalization
            .as_ref()
            .map(|normalized| normalized.had_tool_use)
            .unwrap_or_else(|| {
                content_blocks.iter().any(|block| {
                    matches!(
                        block,
                        ContentBlock::Known(KnownContentBlock::ToolUse { .. })
                    )
                })
            });

        // Map OpenAI finish_reason to Anthropic stop_reason
        let stop_reason = choice.finish_reason.as_deref().map(|reason| {
            if had_tool_use {
                return "tool_use".to_string();
            }

            match reason {
                "stop" => "end_turn".to_string(),
                "length" => "max_tokens".to_string(),
                "tool_calls" => "tool_use".to_string(),
                _ => "end_turn".to_string(),
            }
        });

        GatewayResponse {
            id: response.id,
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: content_blocks,
            model: response.model,
            stop_reason,
            stop_sequence: None,
            usage: GatewayUsage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        }
    }

    /// Transform Responses API response to Anthropic format
    #[allow(dead_code)]
    fn transform_responses_response(&self, response: OpenAIResponsesResponse) -> GatewayResponse {
        // Extract text from output messages
        let text = response
            .output
            .iter()
            .filter(|output| output.output_type == "message")
            .filter_map(|output| output.content.as_ref())
            .flat_map(|content_blocks| {
                content_blocks
                    .iter()
                    .filter(|block| block.block_type == "output_text")
                    .filter_map(|block| block.text.clone())
            })
            .collect::<Vec<_>>()
            .join("\n");

        GatewayResponse {
            id: response.id,
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![ContentBlock::text(text, None)],
            model: response.model,
            stop_reason: Some("end_turn".to_string()),
            stop_sequence: None,
            usage: GatewayUsage {
                input_tokens: response.usage.input_tokens,
                output_tokens: response.usage.output_tokens,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        }
    }

    /// Transform OpenAI streaming chunk to Anthropic SSE format.
    ///
    /// This function converts OpenAI's Chat Completions streaming format to Anthropic's
    /// Messages API streaming format. The transformation is stateful and handles:
    ///
    /// # Event Mapping (OpenAI → Anthropic)
    /// - First chunk → `message_start` (initializes the message envelope)
    /// - `delta.reasoning` → `thinking` content block (separate from text)
    /// - `delta.content` → `text` content block
    /// - `delta.tool_calls` → `content_block_start` (tool_use) + `input_json_delta` (incremental)
    /// - `finish_reason` → `content_block_stop` (for all open blocks) + `message_delta` + `message_stop`
    ///
    /// # Tool Call Streaming
    /// OpenAI sends tool calls incrementally:
    /// - First chunk: `{ index: 0, id: "call_xxx", function: { name: "get_weather", arguments: "" } }`
    /// - Next chunks: `{ index: 0, function: { arguments: "{\"loc" } }`
    /// - More chunks: `{ index: 0, function: { arguments: "ation\":" } }`
    ///
    /// We transform this to Anthropic format:
    /// - On first chunk (has id+name): emit `content_block_start` with type=tool_use
    /// - On argument chunks: emit `content_block_delta` with partial_json
    /// - On finish_reason: emit `content_block_stop` for all open tool blocks
    ///
    /// # Provider Quirks
    /// - Some models send `reasoning` field for chain-of-thought (emitted as thinking block)
    /// - Cerebras may close the stream without sending `finish_reason` (handled by caller)
    fn transform_openai_chunk_to_anthropic_sse(
        chunk: &OpenAIStreamChunk,
        message_id: &str,
        state: &mut StreamTransformState,
    ) -> String {
        let mut output = String::new();

        // First chunk: emit message_start
        if !state.message_started {
            state.message_started = true;
            let message_start = serde_json::json!({
                "type": "message_start",
                "message": {
                    "id": message_id,
                    "type": "message",
                    "role": "assistant",
                    "content": [],
                    "model": chunk.model,
                    "stop_reason": null,
                    "stop_sequence": null,
                    "usage": {
                        "input_tokens": 0,
                        "output_tokens": 0
                    }
                }
            });
            Self::push_sse_event(&mut output, "message_start", message_start);
        }

        // Process delta content
        for choice in &chunk.choices {
            // Handle reasoning content as thinking blocks (separate from text content)
            if let Some(reasoning) = choice.delta.reasoning.as_ref() {
                if !reasoning.is_empty() {
                    if Self::is_kimi_model(&chunk.model) {
                        state.kimi_reasoning_buffer.push_str(reasoning);
                        continue;
                    }

                    // Emit thinking block start if not already open
                    if !state.thinking_block_open {
                        state.thinking_block_open = true;
                        state.thinking_block_index = state.next_block_index;
                        state.next_block_index += 1;
                        let block_start = serde_json::json!({
                            "type": "content_block_start",
                            "index": state.thinking_block_index,
                            "content_block": {
                                "type": "thinking",
                                "thinking": ""
                            }
                        });
                        Self::push_sse_event(&mut output, "content_block_start", block_start);
                    }

                    // Emit thinking delta
                    let delta = serde_json::json!({
                        "type": "content_block_delta",
                        "index": state.thinking_block_index,
                        "delta": {
                            "type": "thinking_delta",
                            "thinking": reasoning
                        }
                    });
                    Self::push_sse_event(&mut output, "content_block_delta", delta);
                }
            }

            // Handle text content
            if let Some(text) = choice.delta.content.as_ref() {
                if !text.is_empty() {
                    Self::close_thinking_block(&mut output, state);
                    Self::emit_text_delta(&mut output, state, text);
                }
            }

            // Tool Calls Transformation (OpenAI function calling → Anthropic tool_use)
            // ==========================================================================
            // OpenAI sends tool calls incrementally:
            //   First chunk: { index: 0, id: "call_xxx", function: { name: "...", arguments: "" } }
            //   Next chunks: { index: 0, function: { arguments: "{\"loc" } }
            //
            // Anthropic expects:
            //   content_block_start: { type: "tool_use", id: "...", name: "...", input: {} }
            //   content_block_delta: { type: "input_json_delta", partial_json: "..." }
            //   content_block_stop: (only at finish_reason)
            if let Some(ref tool_calls) = choice.delta.tool_calls {
                state.saw_explicit_tool_calls = true;
                Self::close_thinking_block(&mut output, state);
                Self::close_text_block(&mut output, state);

                for tool_call in tool_calls {
                    // Get the tool call index from OpenAI
                    let tool_index =
                        tool_call.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

                    // Check if this is the first chunk for this tool (has id and name)
                    let has_id = tool_call.get("id").and_then(|v| v.as_str()).is_some();
                    let has_name = tool_call
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .is_some();

                    if has_id && has_name && !state.tool_blocks.contains_key(&tool_index) {
                        // First chunk for this tool: emit content_block_start
                        let tool_id = tool_call
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tool_0");
                        let tool_name = tool_call
                            .get("function")
                            .and_then(|f| f.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");

                        let block_index = state.next_block_index;
                        state.tool_blocks.insert(tool_index, block_index);
                        state.next_block_index += 1;
                        state.had_tool_calls = true; // Track that this response included tool calls

                        tracing::debug!(
                            "🔧 Tool start: {} (id: {}) at block index {}",
                            tool_name,
                            tool_id,
                            block_index
                        );

                        let block_start = serde_json::json!({
                            "type": "content_block_start",
                            "index": block_index,
                            "content_block": {
                                "type": "tool_use",
                                "id": tool_id,
                                "name": tool_name,
                                "input": {}
                            }
                        });
                        Self::push_sse_event(&mut output, "content_block_start", block_start);
                    }

                    // Emit argument chunks as input_json_delta
                    if let Some(args) = tool_call
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|a| a.as_str())
                    {
                        if !args.is_empty() {
                            // Get the block index for this tool
                            let block_index = state
                                .tool_blocks
                                .get(&tool_index)
                                .copied()
                                .unwrap_or_else(|| {
                                    // Tool block wasn't started yet (shouldn't happen, but handle gracefully)
                                    let idx = state.next_block_index;
                                    state.tool_blocks.insert(tool_index, idx);
                                    state.next_block_index += 1;
                                    idx
                                });

                            let input_delta = serde_json::json!({
                                "type": "content_block_delta",
                                "index": block_index,
                                "delta": {
                                    "type": "input_json_delta",
                                    "partial_json": args
                                }
                            });
                            Self::push_sse_event(&mut output, "content_block_delta", input_delta);
                        }
                    }
                }
            }

            // Stream Termination (finish_reason handling)
            // =============================================
            // When OpenAI sends a chunk with finish_reason, we need to emit the
            // Anthropic stream termination sequence:
            //   1. content_block_stop (for thinking block if open)
            //   2. content_block_stop (for text block if open)
            //   3. content_block_stop (for each open tool block)
            //   4. message_delta (with stop_reason mapped from finish_reason)
            //   5. message_stop (signals end of message)
            if let Some(reason) = &choice.finish_reason {
                state.stream_ended = true;

                if !state.saw_explicit_tool_calls
                    && !state.kimi_hidden_markers_emitted
                    && Self::is_kimi_model(&chunk.model)
                {
                    if let Some(parse) =
                        Self::parse_kimi_hidden_markers(&state.kimi_reasoning_buffer)
                    {
                        Self::emit_hidden_kimi_markers(&mut output, state, &parse);
                    }
                }

                Self::close_thinking_block(&mut output, state);
                Self::close_text_block(&mut output, state);
                Self::close_tool_blocks(&mut output, state);

                // Emit message_delta with stop reason
                // Mapping: OpenAI finish_reason → Anthropic stop_reason
                // IMPORTANT: If this response included any tool calls, force stop_reason="tool_use"
                // even if provider sent finish_reason="stop" (some providers do this incorrectly)
                let stop_reason = if state.had_tool_calls {
                    if reason.as_str() != "tool_calls" {
                        tracing::info!("🔧 Correcting stop_reason: provider sent finish_reason='{}' but response had tool calls, using stop_reason='tool_use'", reason);
                    }
                    "tool_use"
                } else {
                    match reason.as_str() {
                        "stop" => "end_turn",
                        "length" => "max_tokens",
                        "tool_calls" => "tool_use", // Model wants to execute tools
                        _ => "end_turn",
                    }
                };
                // Extract token counts from usage if available (requires stream_options.include_usage)
                let (input_tokens, output_tokens) = chunk
                    .usage
                    .as_ref()
                    .map(|u| (u.prompt_tokens, u.completion_tokens))
                    .unwrap_or((0, 0));
                let message_delta = serde_json::json!({
                    "type": "message_delta",
                    "delta": {
                        "stop_reason": stop_reason,
                        "stop_sequence": null
                    },
                    "usage": {
                        "input_tokens": input_tokens,
                        "output_tokens": output_tokens
                    }
                });
                Self::push_sse_event(&mut output, "message_delta", message_delta);

                // Emit message_stop
                let message_stop = serde_json::json!({
                    "type": "message_stop"
                });
                Self::push_sse_event(&mut output, "message_stop", message_stop);
                tracing::debug!(
                    "✅ Sent message_stop event, stream_ended=true, output_tokens={}",
                    output_tokens
                );
                tracing::debug!("📤 Termination sequence:\n{}", output);
            }
        }

        // If no events were emitted but we processed a chunk, send a ping
        if output.is_empty() {
            output.push_str(": ping\n\n");
        }

        output
    }
}

#[async_trait]
impl super::GatewayProvider for OpenAIProvider {
    async fn send_message(
        &self,
        request: GatewayRequest,
    ) -> Result<GatewayResponse, ProviderError> {
        // Check if we should use Responses API endpoint:
        // - OAuth: Always use /codex/responses for all models
        // - API Key: Only use /responses for models containing "codex"
        let use_responses_api = self.should_use_responses_api(&request);
        let codex_auth_context = if self.is_oauth() && use_responses_api {
            Some(self.resolve_codex_auth_context()?)
        } else {
            None
        };

        if use_responses_api {
            // Use /v1/responses endpoint for Codex models
            let responses_request = self.transform_to_responses_request(&request)?;

            // OAuth (ChatGPT Codex) uses /codex/responses, API Key uses /responses
            let endpoint = if self.is_oauth() {
                self.codex_responses_endpoint()
            } else {
                "/responses"
            };
            let url = self.endpoint_url(endpoint);

            tracing::debug!("Using {} endpoint for model: {}", endpoint, request.model);

            let auth_value = match codex_auth_context.as_ref() {
                Some(auth_context) => auth_context.access_token.expose_secret().to_string(),
                None => self.get_auth_header().await?,
            };
            let (auth_header_name, auth_header_value) =
                self.auth_header_parts(&auth_value, self.is_oauth());
            let mut req_builder = self
                .client
                .post(&url)
                .header(auth_header_name, auth_header_value)
                .header("Content-Type", "application/json");

            // For OAuth (ChatGPT Codex), add Codex-specific headers
            if let Some(auth_context) = codex_auth_context.as_ref() {
                req_builder = self.codex_oauth_request_builder(req_builder, auth_context)?;
                tracing::debug!(
                    "🔐 Using OAuth Bearer token for ChatGPT Codex on {}",
                    self.name
                );
            }

            if !(self.is_oauth() && use_responses_api) {
                // Add custom headers for non-Codex routes only.
                for (key, value) in &self.custom_headers {
                    req_builder = req_builder.header(key, value);
                }
            }

            let response = req_builder.json(&responses_request).send().await?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                tracing::error!("Responses API error ({}): {}", status, error_text);
                return Err(ProviderError::ApiError {
                    status,
                    message: error_text,
                });
            }

            let response_text = response.text().await?;
            tracing::debug!("Responses API response body: {}", response_text);

            if self.is_oauth() {
                let events = parse_sse_events(&response_text);
                let mut state = CodexSemanticAssemblyState::default();

                for event in &events {
                    state.consume_event(event)?;
                }

                return state.into_gateway_response(request.model.clone());
            }

            // Parse SSE (Server-Sent Events) format
            // Format: event: xxx\ndata: {...}\n\n
            // This extracts both reasoning (converted to thinking) and message blocks
            let content_blocks = Self::parse_sse_response(&response_text)?;

            // Return direct response (SSE doesn't need transform)
            Ok(GatewayResponse {
                id: "sse-response".to_string(),
                r#type: "message".to_string(),
                role: "assistant".to_string(),
                content: content_blocks,
                model: request.model.clone(),
                stop_reason: Some("end_turn".to_string()),
                stop_sequence: None,
                usage: GatewayUsage {
                    input_tokens: 0, // SSE doesn't provide token counts
                    output_tokens: 0,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                },
            })
        } else {
            // Use standard /v1/chat/completions endpoint for non-Codex models
            let auth_value = self.get_auth_header().await?;
            let openai_request = self.transform_request(&request)?;
            let url = self.endpoint_url("/chat/completions");

            let (auth_header_name, auth_header_value) =
                self.auth_header_parts(&auth_value, self.is_oauth());
            let mut req_builder = self
                .client
                .post(&url)
                .header(auth_header_name, auth_header_value)
                .header("Content-Type", "application/json");

            // For OAuth (ChatGPT), add account-specific headers
            if self.is_oauth() {
                if let Some(account_id) = Self::extract_account_id(&auth_value) {
                    req_builder = req_builder
                        .header("chatgpt-account-id", account_id)
                        // Browser-like headers to avoid Cloudflare bot detection
                        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
                        .header("Origin", "https://chatgpt.com")
                        .header("Referer", "https://chatgpt.com/")
                        .header("sec-ch-ua", "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"")
                        .header("sec-ch-ua-mobile", "?0")
                        .header("sec-ch-ua-platform", "\"macOS\"")
                        .header("sec-fetch-dest", "empty")
                        .header("sec-fetch-mode", "cors")
                        .header("sec-fetch-site", "same-origin");
                    tracing::debug!("🔐 Using OAuth Bearer token for ChatGPT on {}", self.name);
                }
            }

            // Add custom headers (for OpenRouter, NovitaAI, etc.)
            for (key, value) in &self.custom_headers {
                req_builder = req_builder.header(key, value);
            }

            let response = req_builder.json(&openai_request).send().await?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(ProviderError::ApiError {
                    status,
                    message: error_text,
                });
            }

            // Get response body as text for debugging
            let response_text = response.text().await?;
            tracing::debug!("OpenAI provider response body: {}", response_text);

            // Try to parse the response
            let openai_response: OpenAIResponse =
                serde_json::from_str(&response_text).map_err(|e| {
                    tracing::error!("Failed to parse OpenAI response: {}", e);
                    tracing::error!("Response body was: {}", response_text);
                    e
                })?;

            Ok(self.transform_response(openai_response))
        }
    }

    async fn count_tokens(
        &self,
        request: CountTokensRequest,
    ) -> Result<CountTokensResponse, ProviderError> {
        // For OpenAI, we'll use tiktoken-rs for local token counting
        // This is a placeholder - actual implementation would use tiktoken

        // Rough estimate: ~4 chars per token
        let mut total_chars = 0;

        if let Some(ref system) = request.system {
            let system_text = system.text_content();
            total_chars += system_text.len();
        }

        for msg in &request.messages {
            let content = match &msg.content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Blocks(blocks) => blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
                            Some(text.clone())
                        }
                        ContentBlock::Known(KnownContentBlock::ToolResult { content, .. }) => {
                            Some(content.to_string())
                        }
                        ContentBlock::Known(KnownContentBlock::Thinking { raw }) => raw
                            .get("thinking")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            };
            total_chars += content.len();
        }

        let estimated_tokens = (total_chars / 4) as u32;

        Ok(CountTokensResponse {
            input_tokens: estimated_tokens,
        })
    }

    async fn send_message_stream(
        &self,
        request: GatewayRequest,
    ) -> Result<GatewayStreamResponse, ProviderError> {
        use futures::stream::TryStreamExt;

        // Check if this is a Codex model
        let use_responses_api = self.should_use_responses_api(&request);
        let codex_auth_context = if self.is_oauth() && use_responses_api {
            Some(self.resolve_codex_auth_context()?)
        } else {
            None
        };

        let (url, request_body) = if use_responses_api {
            // Use /v1/responses endpoint for Codex models
            tracing::debug!(
                "Using /v1/responses endpoint for model (streaming): {}",
                request.model
            );
            let responses_request = self.transform_to_responses_request(&request)?;
            let body = serde_json::to_value(&responses_request)
                .map_err(ProviderError::SerializationError)?;
            let endpoint = if self.is_oauth() {
                self.codex_responses_endpoint()
            } else {
                "/responses"
            };
            (self.endpoint_url(endpoint), body)
        } else {
            // Use standard /v1/chat/completions endpoint
            let openai_request = self.transform_request(&request)?;
            let body =
                serde_json::to_value(&openai_request).map_err(ProviderError::SerializationError)?;
            (self.endpoint_url("/chat/completions"), body)
        };

        let auth_value = match codex_auth_context.as_ref() {
            Some(auth_context) => auth_context.access_token.expose_secret().to_string(),
            None => self.get_auth_header().await?,
        };

        // Send streaming request
        let (auth_header_name, auth_header_value) =
            self.auth_header_parts(&auth_value, self.is_oauth());
        let mut req_builder = self
            .client
            .post(&url)
            .header(auth_header_name, auth_header_value)
            .header("Content-Type", "application/json");

        // For OAuth (ChatGPT Codex), add Codex-specific headers
        if self.is_oauth() && use_responses_api {
            let auth_context = codex_auth_context
                .as_ref()
                .expect("Codex auth context missing");
            req_builder = self.codex_oauth_request_builder(req_builder, auth_context)?;
            tracing::debug!(
                "🔐 Using OAuth Bearer token for ChatGPT Codex streaming on {}",
                self.name
            );
        } else if self.is_oauth() {
            // For non-Codex OAuth (if needed in the future)
            if let Some(account_id) = Self::extract_account_id(&auth_value) {
                req_builder = req_builder.header("chatgpt-account-id", account_id);
                tracing::debug!("🔐 Using OAuth Bearer token for streaming on {}", self.name);
            }
        }

        if !(self.is_oauth() && use_responses_api) {
            // Add custom headers for non-Codex routes only.
            for (key, value) in &self.custom_headers {
                req_builder = req_builder.header(key, value);
            }
        }

        let response = req_builder.json(&request_body).send().await?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProviderError::ApiError {
                status,
                message: error_text,
            });
        }

        use futures::stream::StreamExt;
        use std::sync::{Arc, Mutex};

        let model_name = request.model.clone();
        let stream_model_name = model_name.clone();
        let cleanup_model_name = model_name.clone();

        let finalized_stream: std::pin::Pin<
            Box<dyn futures::stream::Stream<Item = Result<Bytes, ProviderError>> + Send>,
        > = if self.is_oauth() {
            let state = Arc::new(Mutex::new(CodexSemanticStreamState::default()));
            let state_for_cleanup = state.clone();
            let provider_name = self.name.clone();

            let transformed_stream = SseStream::new(response.bytes_stream())
                .then(move |result| {
                    let state = state.clone();
                    let model_name = stream_model_name.clone();
                    let provider_name = provider_name.clone();

                    async move {
                        match result {
                            Ok(sse_event) => {
                                let mut state = state.lock().unwrap();
                                match state.consume_event(&sse_event, &model_name) {
                                    Ok(output) => Ok(Bytes::from(output)),
                                    Err(err) => {
                                        tracing::error!(
                                            "❌ {} failed to assemble Codex semantic SSE: {}",
                                            provider_name,
                                            err
                                        );
                                        Err(err)
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("💥 Stream error: {}", e);
                                Err(ProviderError::HttpError(e))
                            }
                        }
                    }
                })
                .try_filter(|bytes| futures::future::ready(!bytes.is_empty()));

            Box::pin(
                transformed_stream
                    .chain(futures::stream::once(async move {
                        let state = state_for_cleanup.lock().unwrap();
                        match state.finalize(&cleanup_model_name) {
                            Ok(output) if !output.is_empty() => Ok(Bytes::from(output)),
                            Ok(_) => Ok(Bytes::new()),
                            Err(err) => Err(err),
                        }
                    }))
                    .try_filter(|bytes| futures::future::ready(!bytes.is_empty())),
            )
        } else {
            // Transform OpenAI SSE format to Anthropic SSE format
            let message_id = format!("msg_{}", uuid::Uuid::new_v4());

            // Streaming State Management
            // ===========================
            // Using Arc<Mutex<StreamTransformState>> to track state across async chunks.
            // The state tracks: message_started, text_block_open, tool_blocks, stream_ended
            let state = Arc::new(Mutex::new(StreamTransformState::default()));
            let state_for_cleanup = state.clone();

            // Convert response bytes stream to SSE events
            let sse_stream = SseStream::new(response.bytes_stream());

            // Capture provider/model names for logging
            let provider_name = self.name.clone();

            // Transform OpenAI SSE events to Anthropic format
            let transformed_stream = sse_stream
                .then(move |result| {
                    let message_id = message_id.clone();
                    let state = state.clone();
                    let provider_name = provider_name.clone();

                    async move {
                        match result {
                            Ok(sse_event) => {
                                // If stream already ended, don't process any more chunks
                                if state.lock().unwrap().stream_ended {
                                    tracing::debug!("⏹️ Stream already ended, skipping chunk");
                                    return Ok(Bytes::new());
                                }

                                tracing::debug!("📦 Received SSE chunk: {}", sse_event.data);

                                // Skip empty data
                                if sse_event.data.trim().is_empty() {
                                    tracing::debug!("⏭️ Skipping empty SSE event");
                                    return Ok(Bytes::new());
                                }

                                if sse_event.data.trim() == "[DONE]" {
                                    tracing::debug!("✅ Stream finished with [DONE]");
                                    return Ok(Bytes::new());
                                }

                                // Check for error response first (some providers return HTTP 200 with error in body)
                                if let Ok(error_response) =
                                    serde_json::from_str::<OpenAIStreamError>(&sse_event.data)
                                {
                                    let status = error_response.status_code.unwrap_or(500);
                                    let error_type =
                                        error_response.error.r#type.as_deref().unwrap_or("unknown");
                                    tracing::error!(
                                        "❌ {} upstream error ({}): {} [type={}]",
                                        provider_name,
                                        status,
                                        error_response.error.message,
                                        error_type
                                    );
                                    return Err(ProviderError::ApiError {
                                        status,
                                        message: format!(
                                            "{}: {}",
                                            provider_name, error_response.error.message
                                        ),
                                    });
                                }

                                // Parse OpenAI chunk
                                match serde_json::from_str::<OpenAIStreamChunk>(&sse_event.data) {
                                    Ok(chunk) => {
                                        tracing::debug!(
                                            "✨ Transforming chunk with {} choices",
                                            chunk.choices.len()
                                        );

                                        // Transform to Anthropic format (raw SSE bytes)
                                        let sse_output =
                                            Self::transform_openai_chunk_to_anthropic_sse(
                                                &chunk,
                                                &message_id,
                                                &mut state.lock().unwrap(),
                                            );

                                        if !sse_output.is_empty() {
                                            tracing::debug!("SSE: {} bytes", sse_output.len());
                                        } else {
                                            tracing::debug!("SSE: empty output (will be filtered)");
                                        }

                                        // Return as raw bytes (already SSE-formatted)
                                        Ok(Bytes::from(sse_output))
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "❌ {} failed to parse chunk: {} - Data: {}",
                                            provider_name,
                                            e,
                                            sse_event.data
                                        );
                                        Ok(Bytes::new())
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("💥 Stream error: {}", e);
                                Err(ProviderError::HttpError(e))
                            }
                        }
                    }
                })
                .try_filter(|bytes| futures::future::ready(!bytes.is_empty()));

            // Add stream finalization to ensure proper termination
            // Some providers close streams without sending finish_reason
            Box::pin(
                transformed_stream
                    .chain(futures::stream::once(async move {
                        let state = state_for_cleanup.lock().unwrap();
                        tracing::debug!(
                            "🏁 Stream finalization: message_started={}, stream_ended={}",
                            state.message_started,
                            state.stream_ended
                        );

                        // Only send end events if stream didn't end properly
                        if state.message_started && !state.stream_ended {
                            tracing::warn!(
                                "⚠️ Stream ended without finish_reason - sending end events"
                            );

                            let mut output = String::new();

                            // Close text block if open
                            if state.text_block_open {
                                let block_stop = serde_json::json!({
                                    "type": "content_block_stop",
                                    "index": state.text_block_index
                                });
                                output.push_str(&format!(
                                    "event: content_block_stop\ndata: {}\n\n",
                                    block_stop
                                ));
                            }

                            // Close all tool blocks
                            for block_index in state.tool_blocks.values() {
                                let block_stop = serde_json::json!({
                                    "type": "content_block_stop",
                                    "index": block_index
                                });
                                output.push_str(&format!(
                                    "event: content_block_stop\ndata: {}\n\n",
                                    block_stop
                                ));
                            }

                            // Send message_delta with end_turn (we don't know the real stop_reason)
                            let message_delta = serde_json::json!({
                                "type": "message_delta",
                                "delta": {
                                    "stop_reason": "end_turn",
                                    "stop_sequence": null
                                },
                                "usage": {
                                    "output_tokens": 0
                                }
                            });
                            output.push_str(&format!(
                                "event: message_delta\ndata: {}\n\n",
                                message_delta
                            ));

                            // Send message_stop
                            let message_stop = serde_json::json!({
                                "type": "message_stop"
                            });
                            output.push_str(&format!(
                                "event: message_stop\ndata: {}\n\n",
                                message_stop
                            ));

                            Ok(Bytes::from(output))
                        } else {
                            tracing::debug!("🏁 Stream properly ended, no finalization needed");
                            Ok(Bytes::new())
                        }
                    }))
                    .try_filter(|bytes| futures::future::ready(!bytes.is_empty())),
            )
        };

        // Wrap with logging stream to capture token stats
        use crate::providers::streaming::LoggingSseStream;
        let logging_stream = LoggingSseStream::new(finalized_stream, self.name.clone(), model_name);

        Ok(GatewayStreamResponse {
            stream: Box::pin(logging_stream),
            headers: HashMap::new(), // OpenAI doesn't have rate limit headers to forward
        })
    }

    fn supports_model(&self, model: &str) -> bool {
        self.models.iter().any(|m| m.eq_ignore_ascii_case(model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::codex_auth_context::{CodexAccountIdSource, CodexAuthMode};
    use crate::auth::CodexAuthSource;
    use crate::models::SystemPrompt;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use secrecy::SecretString;
    use serde::Deserialize;
    use std::{env, fs, sync::Mutex};
    use tempfile::TempDir;

    static ENV_LOCK: once_cell::sync::Lazy<Mutex<()>> =
        once_cell::sync::Lazy::new(|| Mutex::new(()));

    #[derive(Debug, Deserialize)]
    struct FixtureFile {
        normalized_events: Vec<FixtureEvent>,
    }

    #[derive(Debug, Deserialize)]
    struct FixtureEvent {
        event_kind: String,
        #[serde(default)]
        summary_text: Option<String>,
        #[serde(default)]
        final_text: Option<String>,
        #[serde(default)]
        tool_name: Option<String>,
        #[serde(default)]
        tool_id: Option<String>,
        #[serde(default)]
        tool_arguments: Option<serde_json::Value>,
    }

    #[derive(Debug, PartialEq)]
    enum SimplifiedEvent {
        Text(String),
        Tool {
            id: String,
            name: String,
            input: serde_json::Value,
        },
    }

    #[derive(Debug)]
    struct StreamToolBlock {
        id: String,
        name: String,
        input: serde_json::Value,
        partial_json: String,
    }

    #[test]
    fn test_parse_stream_error_response() {
        // This is the actual error format returned by Cerebras (and similar providers)
        // when they return HTTP 200 but have an error in the stream body
        let error_json = r#"{"status_code":500,"error":{"message":"Encountered a server error, please try again.","type":"server_error","param":"","code":"","id":""}}"#;

        let error: OpenAIStreamError = serde_json::from_str(error_json).unwrap();

        assert_eq!(error.status_code, Some(500));
        assert_eq!(
            error.error.message,
            "Encountered a server error, please try again."
        );
        assert_eq!(error.error.r#type, Some("server_error".to_string()));
    }

    #[test]
    fn test_stream_error_does_not_match_valid_chunk() {
        // Valid OpenAI streaming chunk should NOT parse as OpenAIStreamError
        let valid_chunk = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;

        // Should fail to parse as error (no 'error' field)
        let result = serde_json::from_str::<OpenAIStreamError>(valid_chunk);
        assert!(
            result.is_err(),
            "Valid chunk should not parse as error response"
        );
    }

    #[test]
    fn test_parse_error_without_status_code() {
        // Some providers may omit status_code
        let error_json = r#"{"error":{"message":"Rate limit exceeded","type":"rate_limit_error"}}"#;

        let error: OpenAIStreamError = serde_json::from_str(error_json).unwrap();

        assert_eq!(error.status_code, None);
        assert_eq!(error.error.message, "Rate limit exceeded");
        assert_eq!(error.error.r#type, Some("rate_limit_error".to_string()));
    }

    /// Helper: parse a JSON string as an OpenAIStreamChunk and transform it
    fn transform_chunk(json: &str, msg_id: &str, state: &mut StreamTransformState) -> String {
        let chunk: OpenAIStreamChunk = serde_json::from_str(json).unwrap();
        OpenAIProvider::transform_openai_chunk_to_anthropic_sse(&chunk, msg_id, state)
    }

    fn test_provider(transport: OpenAITransport) -> OpenAIProvider {
        OpenAIProvider {
            name: "test".to_string(),
            api_key: String::new(),
            base_url: "https://example.invalid".to_string(),
            transport,
            client: Client::new(),
            models: Vec::new(),
            custom_headers: Vec::new(),
            oauth_provider: None,
            token_store: None,
            codex_auth_source: None,
        }
    }

    fn test_oauth_provider() -> OpenAIProvider {
        let token_store = TokenStore::new(std::env::temp_dir().join(format!(
            "substrate-gateway-openai-test-{}",
            uuid::Uuid::new_v4()
        )))
        .unwrap();

        OpenAIProvider {
            name: "test-oauth".to_string(),
            api_key: String::new(),
            base_url: "https://example.invalid".to_string(),
            transport: OpenAITransport::OpenAI,
            client: Client::new(),
            models: Vec::new(),
            custom_headers: Vec::new(),
            oauth_provider: Some("test-oauth-provider".to_string()),
            token_store: Some(token_store),
            codex_auth_source: Some(CodexAuthSource::Integrated),
        }
    }

    fn codex_access_token(account_id: &str) -> String {
        let header = base64::Engine::encode(&URL_SAFE_NO_PAD, r#"{"alg":"none","typ":"JWT"}"#);
        let payload = base64::Engine::encode(
            &URL_SAFE_NO_PAD,
            format!(
                r#"{{"https://api.openai.com/auth":{{"chatgpt_account_id":"{}"}}}}"#,
                account_id
            ),
        );
        format!("{header}.{payload}.signature")
    }

    fn codex_resolved_context(
        mode: CodexAuthMode,
        explicit_account_id: Option<&str>,
        access_token: SecretString,
    ) -> Result<ResolvedCodexAuthContext, anyhow::Error> {
        if let Some(account_id) = explicit_account_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Ok(ResolvedCodexAuthContext {
                mode,
                account_id: account_id.to_string(),
                account_id_source: CodexAccountIdSource::Explicit,
                access_token,
            });
        }

        let account_id = OpenAIProvider::extract_account_id(access_token.expose_secret())
            .ok_or_else(|| anyhow::anyhow!("Codex auth context could not resolve account_id"))?;

        Ok(ResolvedCodexAuthContext {
            mode,
            account_id,
            account_id_source: CodexAccountIdSource::JwtFallback,
            access_token,
        })
    }

    #[test]
    fn codex_auth_resolution_prefers_integrated_env_handoff_before_local_path() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let bogus_home = temp_dir.path().join("home-as-file");
        fs::write(&bogus_home, "not a directory").unwrap();

        let original_home = env::var_os("HOME");
        let original_account_id = env::var_os(
            crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
        );
        let original_access_token = env::var_os(
            crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
        );

        env::set_var("HOME", &bogus_home);
        env::set_var(
            crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "acct_env_explicit",
        );
        env::set_var(
            crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            codex_access_token("acct_env_jwt"),
        );

        let provider = test_oauth_provider();
        let resolved = provider.resolve_codex_auth_context().unwrap();

        assert_eq!(resolved.account_id, "acct_env_explicit");
        assert_eq!(resolved.account_id_source, CodexAccountIdSource::Explicit);

        match original_home {
            Some(value) => env::set_var("HOME", value),
            None => env::remove_var("HOME"),
        }
        match original_account_id {
            Some(value) => env::set_var(
                crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
                value,
            ),
            None => env::remove_var(
                crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            ),
        }
        match original_access_token {
            Some(value) => env::set_var(
                crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
                value,
            ),
            None => env::remove_var(
                crate::auth::codex_auth_context::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            ),
        }
    }

    fn codex_request_with_tool_choice(tool_choice: serde_json::Value) -> GatewayRequest {
        let mut metadata = HashMap::new();
        metadata.insert(
            OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY.to_string(),
            tool_choice,
        );

        GatewayRequest {
            model: "gpt-test".to_string(),
            messages: vec![crate::models::Message {
                role: "user".to_string(),
                content: crate::models::MessageContent::Text("hello".to_string()),
            }],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: Some(metadata),
            system: None,
            tools: Some(vec![crate::models::Tool {
                r#type: Some("function".to_string()),
                name: Some("lookup".to_string()),
                description: None,
                input_schema: None,
            }]),
        }
    }

    fn gateway_request_with_parallel_tool_calls(value: Option<bool>) -> GatewayRequest {
        let metadata = value.map(|parallel_tool_calls| {
            let mut metadata = HashMap::new();
            metadata.insert(
                OPENAI_PARALLEL_TOOL_CALLS_METADATA_KEY.to_string(),
                serde_json::Value::Bool(parallel_tool_calls),
            );
            metadata
        });

        GatewayRequest {
            model: "gpt-test".to_string(),
            messages: vec![crate::models::Message {
                role: "user".to_string(),
                content: crate::models::MessageContent::Text("hello".to_string()),
            }],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata,
            system: None,
            tools: Some(vec![crate::models::Tool {
                r#type: Some("function".to_string()),
                name: Some("lookup".to_string()),
                description: None,
                input_schema: None,
            }]),
        }
    }

    fn gateway_request_with_multimodal_system_prompt() -> GatewayRequest {
        GatewayRequest {
            model: "gpt-test".to_string(),
            messages: vec![crate::models::Message {
                role: "user".to_string(),
                content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                    KnownContentBlock::Text {
                        text: "hello".to_string(),
                        cache_control: None,
                    },
                )]),
            }],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: None,
            system: Some(SystemPrompt::Blocks(vec![
                ContentBlock::Known(KnownContentBlock::Text {
                    text: "system text".to_string(),
                    cache_control: None,
                }),
                ContentBlock::Known(KnownContentBlock::Image {
                    source: crate::models::ImageSource {
                        r#type: "url".to_string(),
                        media_type: None,
                        data: None,
                        url: Some("https://example.com/system.png".to_string()),
                    },
                }),
            ])),
            tools: None,
        }
    }

    fn codex_request_with_route_matrix_controls() -> GatewayRequest {
        let mut metadata = HashMap::new();
        metadata.insert(
            OPENAI_PARALLEL_TOOL_CALLS_METADATA_KEY.to_string(),
            serde_json::Value::Bool(false),
        );
        metadata.insert(
            OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY.to_string(),
            serde_json::json!({
                "type": "function",
                "function": { "name": "lookup" }
            }),
        );
        metadata.insert(
            OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY.to_string(),
            serde_json::Value::String("low".to_string()),
        );
        metadata.insert(
            OPENAI_RESPONSES_REASONING_SUMMARY_METADATA_KEY.to_string(),
            serde_json::Value::String("concise".to_string()),
        );
        metadata.insert(
            OPENAI_RESPONSES_INCLUDE_METADATA_KEY.to_string(),
            serde_json::json!(["reasoning.encrypted_content"]),
        );
        metadata.insert(
            OPENAI_RESPONSES_TEXT_VERBOSITY_METADATA_KEY.to_string(),
            serde_json::Value::String("low".to_string()),
        );

        GatewayRequest {
            model: "gpt-4.1-mini".to_string(),
            messages: vec![crate::models::Message {
                role: "user".to_string(),
                content: crate::models::MessageContent::Blocks(vec![
                    ContentBlock::text("Describe this image".to_string(), None),
                    ContentBlock::Known(KnownContentBlock::Image {
                        source: crate::models::ImageSource {
                            r#type: "url".to_string(),
                            media_type: None,
                            data: None,
                            url: Some("https://example.com/image.png".to_string()),
                        },
                    }),
                ]),
            }],
            max_output_tokens: 32,
            reasoning: Some(crate::core::ReasoningConfig {
                r#type: "enabled".to_string(),
                budget_tokens: None,
            }),
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: Some(metadata),
            system: None,
            tools: Some(vec![crate::models::Tool {
                r#type: Some("function".to_string()),
                name: Some("lookup".to_string()),
                description: Some("Look something up".to_string()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": { "query": { "type": "string" } },
                    "required": ["query"]
                })),
            }]),
        }
    }

    fn public_responses_continuation_request() -> GatewayRequest {
        let mut metadata = HashMap::new();
        metadata.insert(
            OPENAI_PUBLIC_RESPONSES_METADATA_KEY.to_string(),
            serde_json::Value::Bool(true),
        );

        GatewayRequest {
            model: "gpt-4.1-mini".to_string(),
            messages: vec![
                crate::models::Message {
                    role: "assistant".to_string(),
                    content: crate::models::MessageContent::Text("Need tool output.".to_string()),
                },
                crate::models::Message {
                    role: "assistant".to_string(),
                    content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                        KnownContentBlock::ToolUse {
                            id: "call_fixture_1".to_string(),
                            name: "lookup".to_string(),
                            input: serde_json::json!({
                                "query": "x"
                            }),
                        },
                    )]),
                },
                crate::models::Message {
                    role: "user".to_string(),
                    content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                        KnownContentBlock::ToolResult {
                            tool_use_id: "call_fixture_1".to_string(),
                            content: crate::models::ToolResultContent::Text(
                                "{\"ok\":true}".to_string(),
                            ),
                            is_error: false,
                            cache_control: None,
                        },
                    )]),
                },
            ],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: Some(metadata),
            system: None,
            tools: None,
        }
    }

    fn public_responses_request_with_passthrough_controls() -> GatewayRequest {
        let mut metadata = HashMap::new();
        metadata.insert(
            OPENAI_PUBLIC_RESPONSES_METADATA_KEY.to_string(),
            serde_json::Value::Bool(true),
        );
        metadata.insert(
            OPENAI_RESPONSES_INPUT_METADATA_METADATA_KEY.to_string(),
            serde_json::json!({
                "client": "sdk",
                "request_id": "req_123"
            }),
        );
        metadata.insert(
            OPENAI_RESPONSES_TRUNCATION_METADATA_KEY.to_string(),
            serde_json::json!("auto"),
        );
        metadata.insert(
            OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY.to_string(),
            serde_json::Value::String("resp_prev_123".to_string()),
        );
        metadata.insert(
            OPENAI_RESPONSES_USER_METADATA_KEY.to_string(),
            serde_json::Value::String("user_123".to_string()),
        );
        metadata.insert(
            OPENAI_RESPONSES_STREAM_OPTIONS_METADATA_KEY.to_string(),
            serde_json::json!({
                "include_obfuscation": true
            }),
        );
        metadata.insert(
            OPENAI_RESPONSES_SERVICE_TIER_METADATA_KEY.to_string(),
            serde_json::json!("priority"),
        );

        GatewayRequest {
            model: "gpt-4.1-mini".to_string(),
            messages: vec![crate::models::Message {
                role: "user".to_string(),
                content: crate::models::MessageContent::Text("hello".to_string()),
            }],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: Some(metadata),
            system: None,
            tools: None,
        }
    }

    fn previous_response_id_public_responses_continuation_request() -> GatewayRequest {
        let mut metadata = HashMap::new();
        metadata.insert(
            OPENAI_PUBLIC_RESPONSES_METADATA_KEY.to_string(),
            serde_json::Value::Bool(true),
        );
        metadata.insert(
            OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY.to_string(),
            serde_json::Value::String("resp_prev_123".to_string()),
        );

        GatewayRequest {
            model: "gpt-4.1-mini".to_string(),
            messages: vec![crate::models::Message {
                role: "user".to_string(),
                content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                    KnownContentBlock::ToolResult {
                        tool_use_id: "call_fixture_1".to_string(),
                        content: crate::models::ToolResultContent::Text(
                            "{\"ok\":true}".to_string(),
                        ),
                        is_error: false,
                        cache_control: None,
                    },
                )]),
            }],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: Some(metadata),
            system: None,
            tools: None,
        }
    }

    fn orphaned_public_responses_continuation_request() -> GatewayRequest {
        let mut metadata = HashMap::new();
        metadata.insert(
            OPENAI_PUBLIC_RESPONSES_METADATA_KEY.to_string(),
            serde_json::Value::Bool(true),
        );

        GatewayRequest {
            model: "gpt-4.1-mini".to_string(),
            messages: vec![crate::models::Message {
                role: "user".to_string(),
                content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                    KnownContentBlock::ToolResult {
                        tool_use_id: "call_fixture_1".to_string(),
                        content: crate::models::ToolResultContent::Text(
                            "{\"ok\":true}".to_string(),
                        ),
                        is_error: false,
                        cache_control: None,
                    },
                )]),
            }],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: Some(metadata),
            system: None,
            tools: None,
        }
    }

    fn test_azure_provider() -> OpenAIProvider {
        OpenAIProvider {
            name: "test-azure".to_string(),
            api_key: String::new(),
            base_url: "https://example.azure.com/openai/v1".to_string(),
            transport: OpenAITransport::AzureOpenAI,
            client: Client::new(),
            models: Vec::new(),
            custom_headers: Vec::new(),
            oauth_provider: None,
            token_store: None,
            codex_auth_source: None,
        }
    }

    fn transform_response_json(json: &str) -> GatewayResponse {
        let response: OpenAIResponse = serde_json::from_str(json).unwrap();
        test_provider(OpenAITransport::OpenAI).transform_response(response)
    }

    #[test]
    fn test_azure_api_key_uses_api_key_header() {
        let provider = test_azure_provider();
        let (header_name, header_value) = provider.auth_header_parts("azure-secret", false);

        assert_eq!(header_name, "api-key");
        assert_eq!(header_value, "azure-secret");
    }

    #[test]
    fn test_azure_oauth_keeps_bearer_header() {
        let provider = test_azure_provider();
        let (header_name, header_value) = provider.auth_header_parts("entra-token", true);

        assert_eq!(header_name, "Authorization");
        assert_eq!(header_value, "Bearer entra-token");
    }

    #[test]
    fn test_generic_api_key_keeps_bearer_header() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let (header_name, header_value) = provider.auth_header_parts("openai-secret", false);

        assert_eq!(header_name, "Authorization");
        assert_eq!(header_value, "Bearer openai-secret");
    }

    #[test]
    fn test_generic_oauth_uses_chatgpt_backend_base_url() {
        let token_store = TokenStore::new(std::env::temp_dir().join(format!(
            "substrate-gateway-oauth-test-{}",
            uuid::Uuid::new_v4()
        )))
        .unwrap();

        let provider = OpenAIProvider {
            name: "test-oauth".to_string(),
            api_key: String::new(),
            base_url: "https://example.invalid".to_string(),
            transport: OpenAITransport::OpenAI,
            client: Client::new(),
            models: Vec::new(),
            custom_headers: Vec::new(),
            oauth_provider: Some("test-oauth-provider".to_string()),
            token_store: Some(token_store),
            codex_auth_source: Some(CodexAuthSource::Integrated),
        };

        assert_eq!(
            provider.endpoint_url("/chat/completions"),
            "https://chatgpt.com/backend-api/chat/completions"
        );
    }

    #[test]
    fn codex_oauth_routes_share_the_codex_responses_endpoint() {
        let provider = test_oauth_provider();

        assert_eq!(provider.codex_responses_endpoint(), "/codex/responses");
        assert_eq!(
            provider.endpoint_url(provider.codex_responses_endpoint()),
            "https://chatgpt.com/backend-api/codex/responses"
        );
    }

    #[test]
    fn test_azure_endpoint_uses_full_v1_root() {
        let provider = test_azure_provider();

        assert_eq!(
            provider.endpoint_url("/chat/completions"),
            "https://example.azure.com/openai/v1/chat/completions"
        );
        assert_eq!(
            provider.endpoint_url("responses"),
            "https://example.azure.com/openai/v1/responses"
        );
    }

    #[test]
    fn test_azure_endpoint_normalizes_full_chat_completions_url() {
        let provider = OpenAIProvider::with_transport(
            OpenAITransport::AzureOpenAI,
            OpenAIProviderConfig {
                name: "test-azure".to_string(),
                api_key: String::new(),
                base_url: "https://example.azure.com/openai/v1/chat/completions".to_string(),
                models: Vec::new(),
                custom_headers: Vec::new(),
                oauth_provider: None,
                token_store: None,
                codex_auth_source: None,
            },
        );

        assert_eq!(
            provider.endpoint_url("/chat/completions"),
            "https://example.azure.com/openai/v1/chat/completions"
        );
        assert_eq!(
            provider.endpoint_url("responses"),
            "https://example.azure.com/openai/v1/responses"
        );
    }

    #[test]
    fn transform_request_forwards_parallel_tool_calls_when_explicitly_set() {
        let provider = test_provider(OpenAITransport::OpenAI);
        for value in [Some(false), Some(true)] {
            let request = gateway_request_with_parallel_tool_calls(value);
            let openai_request = provider.transform_to_responses_request(&request).unwrap();
            assert_eq!(openai_request.parallel_tool_calls, value);

            let serialized = serde_json::to_value(openai_request).unwrap();
            assert_eq!(
                serialized
                    .get("parallel_tool_calls")
                    .and_then(|v| v.as_bool()),
                value,
            );
        }
    }

    #[test]
    fn transform_request_omits_parallel_tool_calls_when_not_set() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = gateway_request_with_parallel_tool_calls(None);

        let openai_request = provider.transform_to_responses_request(&request).unwrap();
        assert_eq!(openai_request.parallel_tool_calls, None);

        let serialized = serde_json::to_value(openai_request).unwrap();
        assert!(serialized.get("parallel_tool_calls").is_none());
    }

    #[test]
    fn transform_to_responses_request_preserves_multimodal_system_prompt_in_items_mode() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = gateway_request_with_multimodal_system_prompt();

        let openai_request = provider.transform_to_responses_request(&request).unwrap();
        let serialized = serde_json::to_value(openai_request).unwrap();
        let input = serialized
            .get("input")
            .and_then(|value| value.as_array())
            .expect("responses input array");

        let system_message = input
            .iter()
            .find(|item| {
                item.get("type").and_then(|value| value.as_str()) == Some("message")
                    && item.get("role").and_then(|value| value.as_str()) == Some("system")
            })
            .expect("system message");
        let content = system_message
            .get("content")
            .and_then(|value| value.as_array())
            .expect("system content array");

        assert_eq!(
            content[0].get("type").and_then(|value| value.as_str()),
            Some("input_text")
        );
        assert_eq!(
            content[0].get("text").and_then(|value| value.as_str()),
            Some("system text")
        );
        assert_eq!(
            content[1].get("type").and_then(|value| value.as_str()),
            Some("input_image")
        );
        assert_eq!(
            content[1].get("image_url").and_then(|value| value.as_str()),
            Some("https://example.com/system.png")
        );
    }

    #[test]
    fn transform_to_responses_request_preserves_multimodal_system_prompt_in_oauth_mode() {
        let provider = test_oauth_provider();
        let request = gateway_request_with_multimodal_system_prompt();

        let openai_request = provider.transform_to_responses_request(&request).unwrap();
        let serialized = serde_json::to_value(openai_request).unwrap();
        let input = serialized
            .get("input")
            .and_then(|value| value.as_array())
            .expect("responses input array");

        let system_message = input
            .iter()
            .find(|item| {
                item.get("type").and_then(|value| value.as_str()) == Some("message")
                    && item.get("role").and_then(|value| value.as_str()) == Some("system")
            })
            .expect("system message");
        let content = system_message
            .get("content")
            .and_then(|value| value.as_array())
            .expect("system content array");

        assert_eq!(
            serialized["instructions"],
            serde_json::json!(CODEX_INSTRUCTIONS)
        );
        assert_eq!(serialized["store"], serde_json::json!(false));
        assert!(content.iter().any(|part| {
            part.get("type").and_then(|value| value.as_str()) == Some("input_image")
                && part.get("image_url").and_then(|value| value.as_str())
                    == Some("https://example.com/system.png")
        }));
    }

    #[test]
    fn transform_to_responses_request_preserves_ordered_instruction_roles_in_message_items() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = GatewayRequest {
            model: "gpt-4.1-mini".to_string(),
            messages: vec![
                crate::models::Message {
                    role: "system".to_string(),
                    content: MessageContent::Blocks(vec![
                        ContentBlock::text("sys".to_string(), None),
                        ContentBlock::Known(KnownContentBlock::Image {
                            source: crate::models::ImageSource {
                                r#type: "url".to_string(),
                                media_type: None,
                                data: None,
                                url: Some("https://example.com/system.png".to_string()),
                            },
                        }),
                    ]),
                },
                crate::models::Message {
                    role: "developer".to_string(),
                    content: MessageContent::Blocks(vec![
                        ContentBlock::text("dev".to_string(), None),
                        ContentBlock::Known(KnownContentBlock::Image {
                            source: crate::models::ImageSource {
                                r#type: "url".to_string(),
                                media_type: None,
                                data: None,
                                url: Some("https://example.com/developer.png".to_string()),
                            },
                        }),
                    ]),
                },
                crate::models::Message {
                    role: "user".to_string(),
                    content: MessageContent::Text("hello".to_string()),
                },
            ],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: None,
            system: None,
            tools: None,
        };

        let serialized =
            serde_json::to_value(provider.transform_to_responses_request(&request).unwrap())
                .unwrap();
        let input = serialized["input"]
            .as_array()
            .expect("responses input array");

        assert_eq!(input.len(), 3);
        assert_eq!(input[0]["type"], serde_json::json!("message"));
        assert_eq!(input[0]["role"], serde_json::json!("system"));
        assert_eq!(input[1]["type"], serde_json::json!("message"));
        assert_eq!(input[1]["role"], serde_json::json!("developer"));
        assert_eq!(input[2]["type"], serde_json::json!("message"));
        assert_eq!(input[2]["role"], serde_json::json!("user"));
        assert_eq!(
            input[0]["content"][1]["type"],
            serde_json::json!("input_image")
        );
        assert_eq!(
            input[0]["content"][1]["image_url"],
            serde_json::json!("https://example.com/system.png")
        );
        assert_eq!(
            input[1]["content"][1]["type"],
            serde_json::json!("input_image")
        );
        assert_eq!(
            input[1]["content"][1]["image_url"],
            serde_json::json!("https://example.com/developer.png")
        );
    }

    #[test]
    fn public_responses_continuations_use_the_responses_api_path() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = public_responses_continuation_request();

        assert!(provider.should_use_responses_api(&request));
    }

    #[test]
    fn transform_to_responses_request_lowers_tool_results_to_function_call_output_items() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = public_responses_continuation_request();

        let openai_request = provider.transform_to_responses_request(&request).unwrap();
        let serialized = serde_json::to_value(openai_request).unwrap();
        let input = serialized
            .get("input")
            .and_then(|value| value.as_array())
            .expect("responses input array");

        assert_eq!(input.len(), 3);
        assert_eq!(input[0]["type"], serde_json::json!("message"));
        assert_eq!(input[0]["role"], serde_json::json!("assistant"));
        assert_eq!(
            input[0]["content"][0]["type"],
            serde_json::json!("output_text")
        );
        assert_eq!(input[1]["type"], serde_json::json!("function_call"));
        assert_eq!(input[1]["call_id"], serde_json::json!("call_fixture_1"));
        assert_eq!(input[1]["name"], serde_json::json!("lookup"));
        assert_eq!(
            input[1]["arguments"],
            serde_json::json!("{\"query\":\"x\"}")
        );
        assert_eq!(input[2]["type"], serde_json::json!("function_call_output"));
        assert_eq!(input[2]["call_id"], serde_json::json!("call_fixture_1"));
        assert_eq!(input[2]["output"], serde_json::json!("{\"ok\":true}"));
        assert!(serialized.get("instructions").is_none());
        assert!(serialized.get("store").is_none());
    }

    #[test]
    fn transform_to_responses_request_rejects_orphaned_tool_results() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = orphaned_public_responses_continuation_request();

        let err = provider
            .transform_to_responses_request(&request)
            .unwrap_err();
        match err {
            ProviderError::ConfigError(message) => {
                assert!(message.contains("authoritative provenance"));
            }
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[test]
    fn transform_to_responses_request_preserves_previous_response_id_output_only_continuations() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = previous_response_id_public_responses_continuation_request();

        let serialized =
            serde_json::to_value(provider.transform_to_responses_request(&request).unwrap())
                .unwrap();
        let input = serialized["input"]
            .as_array()
            .expect("responses input array");

        assert_eq!(
            serialized["previous_response_id"],
            serde_json::json!("resp_prev_123")
        );
        assert_eq!(input.len(), 1);
        assert_eq!(input[0]["type"], serde_json::json!("function_call_output"));
        assert_eq!(input[0]["call_id"], serde_json::json!("call_fixture_1"));
        assert_eq!(input[0]["output"], serde_json::json!("{\"ok\":true}"));
    }

    #[test]
    fn transform_to_responses_request_preserves_mixed_continuation_order() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let mut metadata = HashMap::new();
        metadata.insert(
            OPENAI_PUBLIC_RESPONSES_METADATA_KEY.to_string(),
            serde_json::Value::Bool(true),
        );

        let request = GatewayRequest {
            model: "gpt-4.1-mini".to_string(),
            messages: vec![
                crate::models::Message {
                    role: "assistant".to_string(),
                    content: crate::models::MessageContent::Text("first".to_string()),
                },
                crate::models::Message {
                    role: "assistant".to_string(),
                    content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                        KnownContentBlock::ToolUse {
                            id: "call_1".to_string(),
                            name: "lookup".to_string(),
                            input: serde_json::json!({"query":"a"}),
                        },
                    )]),
                },
                crate::models::Message {
                    role: "user".to_string(),
                    content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                        KnownContentBlock::ToolResult {
                            tool_use_id: "call_1".to_string(),
                            content: crate::models::ToolResultContent::Text(
                                "{\"ok\":1}".to_string(),
                            ),
                            is_error: false,
                            cache_control: None,
                        },
                    )]),
                },
                crate::models::Message {
                    role: "assistant".to_string(),
                    content: crate::models::MessageContent::Text("second".to_string()),
                },
                crate::models::Message {
                    role: "assistant".to_string(),
                    content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                        KnownContentBlock::ToolUse {
                            id: "call_2".to_string(),
                            name: "lookup".to_string(),
                            input: serde_json::json!({"query":"b"}),
                        },
                    )]),
                },
                crate::models::Message {
                    role: "user".to_string(),
                    content: crate::models::MessageContent::Blocks(vec![ContentBlock::Known(
                        KnownContentBlock::ToolResult {
                            tool_use_id: "call_2".to_string(),
                            content: crate::models::ToolResultContent::Text(
                                "{\"ok\":2}".to_string(),
                            ),
                            is_error: false,
                            cache_control: None,
                        },
                    )]),
                },
            ],
            max_output_tokens: 32,
            reasoning: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: Some(false),
            metadata: Some(metadata),
            system: None,
            tools: None,
        };

        let serialized =
            serde_json::to_value(provider.transform_to_responses_request(&request).unwrap())
                .unwrap();
        let input = serialized["input"]
            .as_array()
            .expect("responses input array");

        assert_eq!(input.len(), 6);
        assert_eq!(input[0]["type"], serde_json::json!("message"));
        assert_eq!(input[0]["content"][0]["text"], serde_json::json!("first"));
        assert_eq!(input[1]["type"], serde_json::json!("function_call"));
        assert_eq!(input[1]["call_id"], serde_json::json!("call_1"));
        assert_eq!(input[2]["type"], serde_json::json!("function_call_output"));
        assert_eq!(input[2]["call_id"], serde_json::json!("call_1"));
        assert_eq!(input[3]["type"], serde_json::json!("message"));
        assert_eq!(input[3]["content"][0]["text"], serde_json::json!("second"));
        assert_eq!(input[4]["type"], serde_json::json!("function_call"));
        assert_eq!(input[4]["call_id"], serde_json::json!("call_2"));
        assert_eq!(input[5]["type"], serde_json::json!("function_call_output"));
        assert_eq!(input[5]["call_id"], serde_json::json!("call_2"));
    }

    #[test]
    fn transform_to_responses_request_carries_explicit_tool_choice_from_metadata() {
        let provider = test_oauth_provider();
        let request = codex_request_with_tool_choice(serde_json::json!({
            "type": "function",
            "function": {
                "name": "lookup"
            }
        }));

        let openai_request = provider.transform_to_responses_request(&request).unwrap();
        let serialized = serde_json::to_value(openai_request).unwrap();

        assert_eq!(
            serialized["tool_choice"],
            serde_json::json!({
                "type": "function",
                "name": "lookup"
            })
        );
    }

    #[test]
    fn transform_to_responses_request_preserves_codex_route_matrix_controls() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = codex_request_with_route_matrix_controls();

        let serialized =
            serde_json::to_value(provider.transform_to_responses_request(&request).unwrap())
                .unwrap();

        assert_eq!(serialized["stream"], serde_json::json!(true));
        assert_eq!(serialized["parallel_tool_calls"], serde_json::json!(false));
        assert_eq!(
            serialized["tool_choice"],
            serde_json::json!({
                "type": "function",
                "name": "lookup"
            })
        );
        assert_eq!(
            serialized["reasoning"],
            serde_json::json!({
                "effort": "low",
                "summary": "concise"
            })
        );
        assert_eq!(
            serialized["include"],
            serde_json::json!(["reasoning.encrypted_content"])
        );
        assert_eq!(
            serialized["text"],
            serde_json::json!({
                "format": { "type": "text" },
                "verbosity": "low"
            })
        );
        assert_eq!(
            serialized["input"][0]["content"][0],
            serde_json::json!({
                "type": "input_text",
                "text": "Describe this image"
            })
        );
        assert_eq!(
            serialized["input"][0]["content"][1],
            serde_json::json!({
                "type": "input_image",
                "image_url": "https://example.com/image.png"
            })
        );
        let tools = serialized["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["function"]["name"], serde_json::json!("lookup"));
    }

    #[test]
    fn transform_to_responses_request_preserves_public_passthrough_controls() {
        let provider = test_provider(OpenAITransport::OpenAI);
        let request = public_responses_request_with_passthrough_controls();

        let serialized =
            serde_json::to_value(provider.transform_to_responses_request(&request).unwrap())
                .unwrap();

        assert_eq!(
            serialized["metadata"],
            serde_json::json!({
                "client": "sdk",
                "request_id": "req_123"
            })
        );
        assert_eq!(serialized["truncation"], serde_json::json!("auto"));
        assert_eq!(
            serialized["previous_response_id"],
            serde_json::json!("resp_prev_123")
        );
        assert_eq!(serialized["user"], serde_json::json!("user_123"));
        assert_eq!(
            serialized["stream_options"],
            serde_json::json!({
                "include_obfuscation": true
            })
        );
        assert_eq!(serialized["service_tier"], serde_json::json!("priority"));
    }

    #[test]
    fn transform_to_responses_request_rejects_public_passthrough_controls_on_codex_route() {
        let provider = test_oauth_provider();

        for (field, metadata_key, metadata_value) in [
            (
                "metadata",
                OPENAI_RESPONSES_INPUT_METADATA_METADATA_KEY,
                serde_json::json!({"client": "sdk"}),
            ),
            (
                "truncation",
                OPENAI_RESPONSES_TRUNCATION_METADATA_KEY,
                serde_json::json!("auto"),
            ),
            (
                "previous_response_id",
                OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY,
                serde_json::json!("resp_prev_123"),
            ),
            (
                "user",
                OPENAI_RESPONSES_USER_METADATA_KEY,
                serde_json::json!("user_123"),
            ),
            (
                "stream_options",
                OPENAI_RESPONSES_STREAM_OPTIONS_METADATA_KEY,
                serde_json::json!({"include_obfuscation": true}),
            ),
            (
                "service_tier",
                OPENAI_RESPONSES_SERVICE_TIER_METADATA_KEY,
                serde_json::json!("priority"),
            ),
        ] {
            let mut request = gateway_request_with_parallel_tool_calls(None);
            request.metadata = Some(HashMap::from([(metadata_key.to_string(), metadata_value)]));

            let err = provider
                .transform_to_responses_request(&request)
                .unwrap_err();
            match err {
                ProviderError::ConfigError(message) => {
                    assert!(
                        message.contains(field),
                        "expected reject message to mention {field}, got {message}"
                    );
                }
                other => panic!("expected config error for {field}, got {other:?}"),
            }
        }
    }

    #[test]
    fn codex_oauth_request_builder_emits_only_the_minimal_header_contract() {
        let provider = test_oauth_provider();
        let auth_context = codex_resolved_context(
            CodexAuthMode::Integrated,
            Some("acct_123"),
            SecretString::new(codex_access_token("acct_123")),
        )
        .unwrap();
        let expected_auth = format!("Bearer {}", auth_context.access_token.expose_secret());
        let (auth_header_name, auth_header_value) =
            provider.auth_header_parts(auth_context.access_token.expose_secret(), true);
        let request = provider
            .codex_oauth_request_builder(
                provider
                    .client
                    .post("https://example.invalid/v1/responses")
                    .header(auth_header_name, auth_header_value)
                    .header("Content-Type", "application/json"),
                &auth_context,
            )
            .unwrap()
            .build()
            .unwrap();

        let headers = request.headers();
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some(expected_auth.as_str())
        );
        assert_eq!(
            headers
                .get("chatgpt-account-id")
                .and_then(|value| value.to_str().ok()),
            Some("acct_123")
        );
        assert_eq!(
            headers
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
        assert!(headers.get("accept").is_none());
        assert!(headers.get("openai-beta").is_none());
        assert!(headers.get("originator").is_none());
        assert!(headers.get("user-agent").is_none());
        assert!(headers.get("origin").is_none());
        assert!(headers.get("referer").is_none());
        assert!(headers.get("sec-ch-ua").is_none());
        assert!(headers.get("sec-ch-ua-mobile").is_none());
        assert!(headers.get("sec-ch-ua-platform").is_none());
        assert!(headers.get("sec-fetch-dest").is_none());
        assert!(headers.get("sec-fetch-mode").is_none());
        assert!(headers.get("sec-fetch-site").is_none());
    }

    #[test]
    fn codex_oauth_request_builder_prefers_explicit_account_id_over_jwt_fallback() {
        let provider = test_oauth_provider();
        let auth_context = codex_resolved_context(
            CodexAuthMode::Integrated,
            Some("acct_explicit"),
            SecretString::new(codex_access_token("acct_jwt")),
        )
        .unwrap();
        assert_eq!(auth_context.account_id, "acct_explicit");
        assert_eq!(
            auth_context.account_id_source,
            CodexAccountIdSource::Explicit
        );

        let expected_auth = format!("Bearer {}", auth_context.access_token.expose_secret());
        let (auth_header_name, auth_header_value) =
            provider.auth_header_parts(auth_context.access_token.expose_secret(), true);
        let request = provider
            .codex_oauth_request_builder(
                provider
                    .client
                    .post("https://example.invalid/v1/responses")
                    .header(auth_header_name, auth_header_value)
                    .header("Content-Type", "application/json"),
                &auth_context,
            )
            .unwrap()
            .build()
            .unwrap();

        let headers = request.headers();
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some(expected_auth.as_str())
        );
        assert_eq!(
            headers
                .get("chatgpt-account-id")
                .and_then(|value| value.to_str().ok()),
            Some("acct_explicit")
        );
    }

    #[test]
    fn codex_oauth_request_builder_uses_jwt_fallback_only_when_explicit_account_id_is_absent() {
        let provider = test_oauth_provider();
        let auth_context = codex_resolved_context(
            CodexAuthMode::Standalone,
            None,
            SecretString::new(codex_access_token("acct_jwt")),
        )
        .unwrap();
        assert_eq!(auth_context.account_id, "acct_jwt");
        assert_eq!(
            auth_context.account_id_source,
            CodexAccountIdSource::JwtFallback
        );

        let expected_auth = format!("Bearer {}", auth_context.access_token.expose_secret());
        let (auth_header_name, auth_header_value) =
            provider.auth_header_parts(auth_context.access_token.expose_secret(), true);
        let request = provider
            .codex_oauth_request_builder(
                provider
                    .client
                    .post("https://example.invalid/v1/responses")
                    .header(auth_header_name, auth_header_value)
                    .header("Content-Type", "application/json"),
                &auth_context,
            )
            .unwrap()
            .build()
            .unwrap();

        let headers = request.headers();
        assert_eq!(
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok()),
            Some(expected_auth.as_str())
        );
        assert_eq!(
            headers
                .get("chatgpt-account-id")
                .and_then(|value| value.to_str().ok()),
            Some("acct_jwt")
        );
    }

    #[test]
    fn codex_oauth_request_builder_fails_before_upstream_when_account_id_is_unresolvable() {
        let err = codex_resolved_context(
            CodexAuthMode::Standalone,
            None,
            SecretString::new("not-a-jwt".to_string()),
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("Codex auth context could not resolve account_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn transform_to_responses_request_rejects_unsupported_codex_controls() {
        let provider = test_oauth_provider();
        for (field, request) in [
            (
                "temperature",
                GatewayRequest {
                    temperature: Some(0.2),
                    ..codex_request_with_tool_choice(serde_json::json!({
                        "type": "function",
                        "function": { "name": "lookup" }
                    }))
                },
            ),
            (
                "top_p",
                GatewayRequest {
                    top_p: Some(0.5),
                    ..codex_request_with_tool_choice(serde_json::json!({
                        "type": "function",
                        "function": { "name": "lookup" }
                    }))
                },
            ),
            (
                "stop_sequences",
                GatewayRequest {
                    stop_sequences: Some(vec!["done".to_string()]),
                    ..codex_request_with_tool_choice(serde_json::json!({
                        "type": "function",
                        "function": { "name": "lookup" }
                    }))
                },
            ),
        ] {
            let err = provider
                .transform_to_responses_request(&request)
                .unwrap_err();
            match err {
                ProviderError::ConfigError(message) => {
                    assert!(
                        message.contains(field),
                        "expected reject message to mention {field}, got {message}"
                    );
                }
                other => panic!("expected config error for {field}, got {other:?}"),
            }
        }
    }

    #[test]
    fn azure_gpt5_chat_requests_use_max_completion_tokens() {
        let provider = test_azure_provider();
        let request = GatewayRequest {
            model: "gpt-5.4-mini".to_string(),
            ..gateway_request_with_parallel_tool_calls(None)
        };

        let openai_request = provider.transform_request(&request).unwrap();
        let serialized = serde_json::to_value(openai_request).unwrap();

        assert_eq!(
            serialized
                .get("max_completion_tokens")
                .and_then(|value| value.as_u64()),
            Some(32)
        );
        assert!(serialized.get("max_tokens").is_none());
    }

    fn load_fixture(name: &str) -> FixtureFile {
        let json = match name {
            "explicit-tool-calls-k2-thinking-stream" => {
                include_str!(
                    "../../tests/fixtures/azure_kimi/explicit-tool-calls-k2-thinking-stream.json"
                )
            }
            "hidden-markers-k2-thinking-stream" => {
                include_str!(
                    "../../tests/fixtures/azure_kimi/hidden-markers-k2-thinking-stream.json"
                )
            }
            "hidden-markers-k2-thinking-nonstream" => {
                include_str!(
                    "../../tests/fixtures/azure_kimi/hidden-markers-k2-thinking-nonstream.json"
                )
            }
            "mixed-reasoning-and-tool-calls-k2-thinking" => {
                include_str!("../../tests/fixtures/azure_kimi/mixed-reasoning-and-tool-calls-k2-thinking.json")
            }
            "no-tool-control-k2-5-stream" => {
                include_str!("../../tests/fixtures/azure_kimi/no-tool-control-k2-5-stream.json")
            }
            other => panic!("unknown fixture: {other}"),
        };
        serde_json::from_str(json).unwrap()
    }

    fn actual_events_from_sse_outputs(outputs: &[String]) -> Vec<SimplifiedEvent> {
        use std::collections::BTreeMap;

        let mut order = Vec::new();
        let mut text_blocks: BTreeMap<u32, String> = BTreeMap::new();
        let mut tool_blocks: BTreeMap<u32, StreamToolBlock> = BTreeMap::new();

        for output in outputs {
            for raw_event in output
                .split("\n\n")
                .filter(|entry| !entry.trim().is_empty())
            {
                let mut event_name = None;
                let mut data = None;

                for line in raw_event.lines() {
                    if let Some(value) = line.strip_prefix("event: ") {
                        event_name = Some(value);
                    } else if let Some(value) = line.strip_prefix("data: ") {
                        data = Some(value);
                    }
                }

                let Some(event_name) = event_name else {
                    continue;
                };
                let Some(data) = data else {
                    continue;
                };
                let Ok(json) = serde_json::from_str::<serde_json::Value>(data) else {
                    continue;
                };

                match event_name {
                    "content_block_start" => {
                        let Some(index) = json
                            .get("index")
                            .and_then(|value| value.as_u64())
                            .map(|value| value as u32)
                        else {
                            continue;
                        };
                        let Some(content_block) = json.get("content_block") else {
                            continue;
                        };

                        match content_block.get("type").and_then(|value| value.as_str()) {
                            Some("text") => {
                                order.push(index);
                                text_blocks.entry(index).or_default();
                            }
                            Some("tool_use") => {
                                order.push(index);
                                tool_blocks.insert(
                                    index,
                                    StreamToolBlock {
                                        id: content_block
                                            .get("id")
                                            .and_then(|value| value.as_str())
                                            .unwrap_or_default()
                                            .to_string(),
                                        name: content_block
                                            .get("name")
                                            .and_then(|value| value.as_str())
                                            .unwrap_or_default()
                                            .to_string(),
                                        input: content_block
                                            .get("input")
                                            .cloned()
                                            .unwrap_or_else(|| serde_json::json!({})),
                                        partial_json: String::new(),
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
                    "content_block_delta" => {
                        let Some(index) = json
                            .get("index")
                            .and_then(|value| value.as_u64())
                            .map(|value| value as u32)
                        else {
                            continue;
                        };
                        let Some(delta) = json.get("delta") else {
                            continue;
                        };

                        match delta.get("type").and_then(|value| value.as_str()) {
                            Some("text_delta") => {
                                if let Some(text) =
                                    delta.get("text").and_then(|value| value.as_str())
                                {
                                    text_blocks.entry(index).or_default().push_str(text);
                                }
                            }
                            Some("input_json_delta") => {
                                if let Some(partial_json) =
                                    delta.get("partial_json").and_then(|value| value.as_str())
                                {
                                    if let Some(tool_block) = tool_blocks.get_mut(&index) {
                                        tool_block.partial_json.push_str(partial_json);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        order
            .into_iter()
            .filter_map(|index| {
                if let Some(text) = text_blocks.get(&index) {
                    return Some(SimplifiedEvent::Text(text.clone()));
                }

                tool_blocks.get(&index).map(|tool_block| {
                    let input = if tool_block.partial_json.is_empty() {
                        tool_block.input.clone()
                    } else {
                        serde_json::from_str(&tool_block.partial_json).unwrap()
                    };

                    SimplifiedEvent::Tool {
                        id: tool_block.id.clone(),
                        name: tool_block.name.clone(),
                        input,
                    }
                })
            })
            .collect()
    }

    fn stop_reason_from_sse_outputs(outputs: &[String]) -> Option<String> {
        for output in outputs {
            for raw_event in output
                .split("\n\n")
                .filter(|entry| !entry.trim().is_empty())
            {
                let mut event_name = None;
                let mut data = None;

                for line in raw_event.lines() {
                    if let Some(value) = line.strip_prefix("event: ") {
                        event_name = Some(value);
                    } else if let Some(value) = line.strip_prefix("data: ") {
                        data = Some(value);
                    }
                }

                if event_name != Some("message_delta") {
                    continue;
                }

                let Some(data) = data else {
                    continue;
                };
                let Ok(json) = serde_json::from_str::<serde_json::Value>(data) else {
                    continue;
                };

                if let Some(stop_reason) = json
                    .get("delta")
                    .and_then(|delta| delta.get("stop_reason"))
                    .and_then(|stop_reason| stop_reason.as_str())
                {
                    return Some(stop_reason.to_string());
                }
            }
        }

        None
    }

    fn expected_events_from_fixture(name: &str) -> Vec<SimplifiedEvent> {
        load_fixture(name)
            .normalized_events
            .into_iter()
            .map(|event| match event.event_kind.as_str() {
                "action" => SimplifiedEvent::Text(event.summary_text.expect("action summary_text")),
                "final" => SimplifiedEvent::Text(event.final_text.expect("final final_text")),
                "tool_intent" => SimplifiedEvent::Tool {
                    id: event.tool_id.expect("tool_intent tool_id"),
                    name: event.tool_name.expect("tool_intent tool_name"),
                    input: event.tool_arguments.expect("tool_intent tool_arguments"),
                },
                other => panic!("unexpected event kind: {other}"),
            })
            .collect()
    }

    fn actual_events_from_response(response: &GatewayResponse) -> Vec<SimplifiedEvent> {
        response
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
                    Some(SimplifiedEvent::Text(text.clone()))
                }
                ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input }) => {
                    Some(SimplifiedEvent::Tool {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// Regression test: kimi-k2.5 sends tool_calls before text, then a trailing
    /// content " " after the tool call. The text block must not overwrite the
    /// tool_use block (they need distinct indices).
    #[test]
    fn test_tool_call_before_text_gets_distinct_indices() {
        let mut state = StreamTransformState::default();
        let id = "msg_test";

        // 1. First chunk: tool_call with name (kimi's first tool chunk)
        let out = transform_chunk(
            r#"{
            "id":"gen-1","model":"kimi","choices":[{"index":0,"delta":{
                "role":"assistant","content":null,
                "tool_calls":[{"index":0,"id":"functions.Bash:0","type":"function",
                    "function":{"name":"Bash","arguments":null}}]
            },"finish_reason":null}]
        }"#,
            id,
            &mut state,
        );
        assert!(
            out.contains("tool_use"),
            "should emit content_block_start for tool"
        );
        assert!(out.contains(r#""name":"Bash"#), "tool name should be Bash");

        // 2. Argument chunks
        let out = transform_chunk(
            r#"{
            "id":"gen-1","model":"kimi","choices":[{"index":0,"delta":{
                "content":null,
                "tool_calls":[{"index":0,"id":"functions.Bash:0","type":"function",
                    "function":{"name":null,"arguments":"{\"command\":\"git log\"}"}}]
            },"finish_reason":null}]
        }"#,
            id,
            &mut state,
        );
        assert!(
            out.contains("input_json_delta"),
            "should emit argument delta"
        );

        // 3. Trailing content " " (kimi quirk: sent after tool_calls)
        let out = transform_chunk(
            r#"{
            "id":"gen-1","model":"kimi","choices":[{"index":0,"delta":{
                "content":" ","reasoning":null
            },"finish_reason":null}]
        }"#,
            id,
            &mut state,
        );
        // Text block should get index 1, not 0 (which is the tool block)
        assert!(
            out.contains(r#""index":1"#),
            "text block should be at index 1, not 0"
        );
        assert!(
            !out.contains(r#""index":0"#),
            "must not emit anything at index 0 (tool block)"
        );

        // 4. finish_reason: tool_calls
        let out = transform_chunk(
            r#"{
            "id":"gen-1","model":"kimi","choices":[{"index":0,"delta":{
                "content":""
            },"finish_reason":"tool_calls"}]
        }"#,
            id,
            &mut state,
        );
        assert!(out.contains("tool_use"), "stop_reason should be tool_use");
        assert!(out.contains("message_stop"), "should end the stream");
    }

    /// When text comes first (normal case), text gets index 0 and tool gets index 1.
    #[test]
    fn test_text_before_tool_call_normal_ordering() {
        let mut state = StreamTransformState::default();
        let id = "msg_test";

        // 1. Text content first
        let out = transform_chunk(
            r#"{
            "id":"gen-1","model":"test","choices":[{"index":0,"delta":{
                "content":"Let me check"
            },"finish_reason":null}]
        }"#,
            id,
            &mut state,
        );
        assert!(out.contains(r#""index":0"#), "text block at index 0");
        assert!(out.contains("text_delta"));

        // 2. Tool call arrives (should close text, open tool at index 1)
        let out = transform_chunk(
            r#"{
            "id":"gen-1","model":"test","choices":[{"index":0,"delta":{
                "content":null,
                "tool_calls":[{"index":0,"id":"call_123","type":"function",
                    "function":{"name":"Bash","arguments":"{}"}}]
            },"finish_reason":null}]
        }"#,
            id,
            &mut state,
        );
        // Should close text block at index 0
        assert!(out.contains("content_block_stop"));
        // Should open tool block at index 1
        assert!(out.contains(r#""index":1"#));
        assert!(out.contains("tool_use"));
    }

    #[test]
    fn test_explicit_kimi_tool_calls_match_fixture_shape() {
        let response = transform_response_json(
            r#"{
                "id":"resp-explicit",
                "object":"chat.completion",
                "model":"Kimi-K2-Thinking",
                "choices":[{
                    "index":0,
                    "message":{
                        "role":"assistant",
                        "content":"I'll explore the repository structure and then use the startup skill to analyze it.",
                        "reasoning_content":"internal planning that stays provider-local",
                        "tool_calls":[
                            {"id":"functions.Bash:0","type":"function","function":{"name":"Bash","arguments":"{\"command\":\"ls -la\",\"description\":\"List repository contents\"}"}},
                            {"id":"functions.Read:1","type":"function","function":{"name":"Read","arguments":"{\"file_path\":\"/Users/spensermcconnell/__Active_Code/forked/README.md\"}"}}
                        ]
                    },
                    "finish_reason":"tool_calls"
                }],
                "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}
            }"#,
        );

        assert_eq!(
            actual_events_from_response(&response),
            expected_events_from_fixture("explicit-tool-calls-k2-thinking-stream")
        );
        assert_eq!(response.stop_reason.as_deref(), Some("tool_use"));
    }

    #[test]
    fn test_hidden_kimi_markers_match_fixture_shape() {
        let response = transform_response_json(
            r#"{
                "id":"resp-hidden",
                "object":"chat.completion",
                "model":"Kimi-K2-Thinking",
                "choices":[{
                    "index":0,
                    "message":{
                        "role":"assistant",
                        "content":null,
                        "reasoning_content":"Let me look for the Python package configuration to understand the project structure better. <|tool_calls_section_begin|> <|tool_call_begin|> functions.Read:3 <|tool_call_argument_begin|> {\"file_path\":\"/Users/spensermcconnell/__Active_Code/forked/pyproject.toml\"} <|tool_call_end|> <|tool_call_begin|> functions.Read:4 <|tool_call_argument_begin|> {\"file_path\":\"/Users/spensermcconnell/__Active_Code/forked/src\"} <|tool_call_end|> <|tool_calls_section_end|>",
                        "tool_calls":null
                    },
                    "finish_reason":"stop"
                }],
                "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}
            }"#,
        );

        assert_eq!(
            actual_events_from_response(&response),
            expected_events_from_fixture("hidden-markers-k2-thinking-nonstream")
        );
        assert_eq!(response.stop_reason.as_deref(), Some("tool_use"));
    }

    #[test]
    fn test_mixed_kimi_response_prefers_explicit_tool_calls() {
        let response = transform_response_json(
            r#"{
                "id":"resp-mixed",
                "object":"chat.completion",
                "model":"Kimi-K2-Thinking",
                "choices":[{
                    "index":0,
                    "message":{
                        "role":"assistant",
                        "content":"I'll first explore this repository to understand what it does, then evaluate it using the startup skill.",
                        "reasoning_content":"The user wants me to inspect the repo first, then evaluate it.",
                        "tool_calls":[
                            {"id":"functions.Bash:0","type":"function","function":{"name":"Bash","arguments":"{\"command\":\"ls -la\",\"description\":\"List repository contents\"}"}},
                            {"id":"functions.Read:1","type":"function","function":{"name":"Read","arguments":"{\"file_path\":\"/Users/spensermcconnell/__Active_Code/forked/README.md\"}"}},
                            {"id":"functions.Read:2","type":"function","function":{"name":"Read","arguments":"{\"file_path\":\"/Users/spensermcconnell/__Active_Code/forked/package.json\"}"}}
                        ]
                    },
                    "finish_reason":"tool_calls"
                }],
                "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}
            }"#,
        );

        assert_eq!(
            actual_events_from_response(&response),
            expected_events_from_fixture("mixed-reasoning-and-tool-calls-k2-thinking")
        );
        assert_eq!(response.stop_reason.as_deref(), Some("tool_use"));
    }

    #[test]
    fn test_kimi_no_tool_response_matches_fixture_shape() {
        let response = transform_response_json(
            r#"{
                "id":"resp-final",
                "object":"chat.completion",
                "model":"Kimi-K2.5",
                "choices":[{
                    "index":0,
                    "message":{
                        "role":"assistant",
                        "content":"{\"title\": \"Start new coding session\"}",
                        "reasoning_content":"The user has simply greeted me and has not provided any coding task yet.",
                        "tool_calls":null
                    },
                    "finish_reason":"stop"
                }],
                "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}
            }"#,
        );

        assert_eq!(
            actual_events_from_response(&response),
            expected_events_from_fixture("no-tool-control-k2-5-stream")
        );
        assert_eq!(response.stop_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn test_malformed_kimi_hidden_markers_do_not_leak_sentinel_text() {
        let response = transform_response_json(
            r#"{
                "id":"resp-malformed",
                "object":"chat.completion",
                "model":"Kimi-K2-Thinking",
                "choices":[{
                    "index":0,
                    "message":{
                        "role":"assistant",
                        "content":null,
                        "reasoning_content":"Let me inspect the package layout first. <|tool_calls_section_begin|> <|tool_call_begin|> functions.Read:3",
                        "tool_calls":null
                    },
                    "finish_reason":"stop"
                }],
                "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}
            }"#,
        );

        let actual = actual_events_from_response(&response);
        assert_eq!(
            actual,
            vec![SimplifiedEvent::Text(
                "Let me inspect the package layout first.".to_string()
            )]
        );
        assert!(
            !response
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Known(KnownContentBlock::Text { text, .. }) => Some(text),
                    _ => None,
                })
                .any(|text| text.contains("<|tool_")),
            "malformed sentinel text should stay internal"
        );
        assert_eq!(response.stop_reason.as_deref(), Some("end_turn"));
    }

    /// Reasoning should be emitted as a thinking block (not merged with text content) for generic models.
    #[test]
    fn test_reasoning_becomes_thinking_block() {
        let mut state = StreamTransformState::default();
        let id = "msg_test";

        let out = transform_chunk(
            r#"{
            "id":"gen-1","model":"gpt-4","choices":[{"index":0,"delta":{
                "content":"","reasoning":"thinking about it"
            },"finish_reason":null}]
        }"#,
            id,
            &mut state,
        );
        assert!(
            out.contains("thinking about it"),
            "should include reasoning content"
        );
        assert!(
            out.contains("\"type\":\"thinking\""),
            "should be a thinking content block"
        );
        assert!(
            out.contains("thinking_delta"),
            "should use thinking_delta type"
        );
    }

    #[test]
    fn test_kimi_reasoning_content_stays_internal_in_streaming_mode() {
        let mut state = StreamTransformState::default();
        let id = "msg_test";

        let outputs = vec![
            transform_chunk(
                r#"{
            "id":"gen-1","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                "content":"",
                "reasoning_content":"internal hidden planning"
            },"finish_reason":null}]
        }"#,
                id,
                &mut state,
            ),
            transform_chunk(
                r#"{
            "id":"gen-1","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                "content":""
            },"finish_reason":"stop"}]
        }"#,
                id,
                &mut state,
            ),
        ];

        let combined = outputs.join("");
        assert!(
            combined.contains("message_start"),
            "first chunk should still start the message"
        );
        assert!(
            !combined.contains("thinking_delta"),
            "Kimi reasoning_content should stay internal"
        );
        assert!(
            !combined.contains("\"type\":\"thinking\""),
            "Kimi reasoning should not become a public thinking block"
        );
        assert_eq!(
            actual_events_from_sse_outputs(&outputs),
            Vec::<SimplifiedEvent>::new()
        );
        assert_eq!(
            stop_reason_from_sse_outputs(&outputs).as_deref(),
            Some("end_turn")
        );
    }

    #[test]
    fn test_streamed_hidden_kimi_markers_match_fixture_shape() {
        let mut state = StreamTransformState::default();
        let id = "msg_hidden_stream";
        let chunks = [
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "content":"",
                    "role":"assistant"
                },"finish_reason":null}]
            }"#,
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "reasoning_content":"  Good, now I have a much better understanding. Let me explore a bit more to understand the market and competitive landscape before running the startup evaluation."
                },"finish_reason":null}]
            }"#,
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "reasoning_content":" <|tool_calls_section_begin|> <|tool_call_begin|> functions.Read:21 <|tool_call_argument_begin|> {\"file_path\": \"/Users/spensermcconnell/__Active_Code/forked/docs/README.md\"} <|tool_call_end|>"
                },"finish_reason":null}]
            }"#,
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "reasoning_content":" <|tool_call_begin|> functions.Skill:22 <|tool_call_argument_begin|> {\"skill\": \"startup:startup-competitors\", \"args\": \"git workflow automation tools fork management\"} <|tool_call_end|> <|tool_calls_section_end|>"
                },"finish_reason":null}]
            }"#,
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "content":""
                },"finish_reason":"stop"}],
                "usage":{"prompt_tokens":56258,"completion_tokens":92,"total_tokens":56350}
            }"#,
        ];

        let outputs = chunks
            .into_iter()
            .map(|chunk| transform_chunk(chunk, id, &mut state))
            .collect::<Vec<_>>();

        assert_eq!(
            actual_events_from_sse_outputs(&outputs),
            expected_events_from_fixture("hidden-markers-k2-thinking-stream")
        );

        let combined = outputs.join("");
        assert!(
            !combined.contains("thinking_delta"),
            "hidden streamed markers should not leak Kimi reasoning as thinking"
        );
        assert!(
            !combined.contains("\"type\":\"thinking\""),
            "hidden streamed markers should not open a public thinking block"
        );
    }

    #[test]
    fn test_streamed_hidden_kimi_markers_force_tool_use_stop_reason() {
        let mut state = StreamTransformState::default();
        let id = "msg_hidden_stream";
        let chunks = [
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "content":"",
                    "role":"assistant"
                },"finish_reason":null}]
            }"#,
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "reasoning_content":"  Good, now I have a much better understanding. Let me explore a bit more to understand the market and competitive landscape before running the startup evaluation. <|tool_calls_section_begin|> <|tool_call_begin|> functions.Read:21 <|tool_call_argument_begin|> {\"file_path\": \"/Users/spensermcconnell/__Active_Code/forked/docs/README.md\"} <|tool_call_end|> <|tool_call_begin|> functions.Skill:22 <|tool_call_argument_begin|> {\"skill\": \"startup:startup-competitors\", \"args\": \"git workflow automation tools fork management\"} <|tool_call_end|> <|tool_calls_section_end|>"
                },"finish_reason":null}]
            }"#,
            r#"{
                "id":"gen-stream-hidden","model":"Kimi-K2-Thinking","choices":[{"index":0,"delta":{
                    "content":""
                },"finish_reason":"stop"}]
            }"#,
        ];

        let outputs = chunks
            .into_iter()
            .map(|chunk| transform_chunk(chunk, id, &mut state))
            .collect::<Vec<_>>();

        assert_eq!(
            stop_reason_from_sse_outputs(&outputs).as_deref(),
            Some("tool_use")
        );
    }

    #[test]
    fn codex_semantic_sync_assembles_text_and_tools_from_event_family() {
        let sse_text = [
            r#"event: response.output_item.added
data: {"type":"response.output_item.added","output_index":0,"item":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Visible"}]}}

"#,
            r#"event: response.content_part.added
data: {"type":"response.content_part.added","output_index":0,"content_index":0,"part":{"type":"output_text","text":" answer"}}

"#,
            r#"event: response.output_text.delta
data: {"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":"!"}

"#,
            r#"event: response.output_text.done
data: {"type":"response.output_text.done","output_index":0,"content_index":0,"text":"Visible answer!"}

"#,
            r#"event: response.output_item.done
data: {"type":"response.output_item.done","output_index":0,"item":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Visible answer!"}]}}

"#,
            r#"event: response.output_item.added
data: {"type":"response.output_item.added","output_index":1,"item":{"type":"function_call","call_id":"call_1","name":"lookup","arguments":"{}"}}

"#,
            r#"event: response.function_call_arguments.delta
data: {"type":"response.function_call_arguments.delta","output_index":1,"call_id":"call_1","delta":"{\"path\":\"README.md\"}"}

"#,
            r#"event: response.function_call_arguments.done
data: {"type":"response.function_call_arguments.done","output_index":1,"call_id":"call_1","arguments":"{\"path\":\"README.md\"}"}

"#,
            r#"event: response.output_item.done
data: {"type":"response.output_item.done","output_index":1,"item":{"type":"function_call","call_id":"call_1","name":"lookup","arguments":"{\"path\":\"README.md\"}"}}

"#,
            r#"event: response.completed
data: {"type":"response.completed","response":{"id":"resp_codex","status":"completed","output":[],"usage":{"input_tokens":7,"output_tokens":2}}}

"#,
        ]
        .join("");

        let events = parse_sse_events(&sse_text);
        let mut state = CodexSemanticAssemblyState::default();

        for event in &events {
            state.consume_event(event).unwrap();
        }

        let response = state
            .into_gateway_response("gateway-default".to_string())
            .unwrap();

        assert_eq!(response.id, "resp_codex");
        assert_eq!(response.model, "gateway-default");
        assert_eq!(response.stop_reason.as_deref(), Some("end_turn"));
        assert_eq!(response.usage.input_tokens, 7);
        assert_eq!(response.usage.output_tokens, 2);
        assert_eq!(response.content.len(), 2);
        assert!(matches!(
            &response.content[0],
            ContentBlock::Known(KnownContentBlock::Text { text, .. }) if text == "Visible answer!"
        ));
        assert!(matches!(
            &response.content[1],
            ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input, .. })
            if id == "call_1" && name == "lookup" && input == &serde_json::json!({"path":"README.md"})
        ));
    }

    #[test]
    fn codex_semantic_sync_requires_terminal_completed_event() {
        let sse_text = r#"event: response.output_text.delta
data: {"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":"Visible"}

"#;
        let events = parse_sse_events(sse_text);
        let mut state = CodexSemanticAssemblyState::default();

        for event in &events {
            state.consume_event(event).unwrap();
        }

        let error = state
            .into_gateway_response("gateway-default".to_string())
            .unwrap_err();
        assert!(matches!(error, ProviderError::ApiError { status: 502, .. }));
    }

    #[test]
    fn codex_semantic_stream_emits_anthropic_sse_and_hides_reasoning() {
        let sse_text = [
            r#"event: response.output_item.added
data: {"type":"response.output_item.added","output_index":0,"item":{"type":"reasoning","content":[{"type":"output_text","text":"secret reasoning"}]}}

"#,
            r#"event: response.output_item.added
data: {"type":"response.output_item.added","output_index":1,"item":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Visible"}]}}

"#,
            r#"event: response.output_text.delta
data: {"type":"response.output_text.delta","output_index":1,"content_index":0,"delta":" answer"}

"#,
            r#"event: response.output_item.added
data: {"type":"response.output_item.added","output_index":2,"item":{"type":"function_call","call_id":"call_2","name":"lookup","arguments":"{}"}}

"#,
            r#"event: response.function_call_arguments.delta
data: {"type":"response.function_call_arguments.delta","output_index":2,"call_id":"call_2","delta":"{\"path\":\"README.md\"}"}

"#,
            r#"event: response.completed
data: {"type":"response.completed","response":{"id":"resp_stream","status":"completed","output":[],"usage":{"input_tokens":4,"output_tokens":2}}}

"#,
        ]
        .join("");

        let events = parse_sse_events(&sse_text);
        let mut state = CodexSemanticStreamState::default();
        let mut output = String::new();

        for event in &events {
            output.push_str(&state.consume_event(event, "gateway-default").unwrap());
        }

        assert!(!output.contains("secret reasoning"));
        let parsed_events = parse_sse_events(&output);
        let names = parsed_events
            .iter()
            .filter_map(|event| event.event.as_deref())
            .collect::<Vec<_>>();

        assert!(names.contains(&"message_start"));
        assert!(names.contains(&"content_block_delta"));
        assert!(names.contains(&"message_delta"));
        assert!(names.contains(&"message_stop"));
    }
}
