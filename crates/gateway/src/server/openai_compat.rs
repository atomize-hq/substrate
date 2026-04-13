use crate::core::{GatewayRequest, GatewayResponse};
use crate::models::{
    ContentBlock, KnownContentBlock, Message, MessageContent, Tool, ToolResultContent,
};
use bytes::Bytes;
use futures::stream::Stream;
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::providers::streaming::{parse_sse_events, SseEvent};

/// OpenAI Chat Completions request format (subset per C-10 / ADR 0008).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<OpenAIStreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,

    // Known-but-unsupported fields (presence MUST be rejected per C-10 / ADR 0008).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIStreamOptions {
    #[serde(default)]
    pub include_usage: Option<bool>,
    #[allow(dead_code)]
    #[serde(default)]
    pub include_obfuscation: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<OpenAIContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub name: Option<String>,

    /// Tool calls in assistant messages (tool loop continuation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCallIn>>,

    /// Tool call id in tool-role messages (tool loop continuation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Content can be string or array of content parts.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpenAIContent {
    String(String),
    Parts(Vec<OpenAIContentPart>),
}

/// Content part (text or image_url).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: OpenAIImageUrl },
}

/// Image URL object.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIImageUrl {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAIToolCallIn {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: OpenAIFunctionCallIn,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAIFunctionCallIn {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAIToolIn {
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: OpenAIFunctionDefIn,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpenAIFunctionDefIn {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

/// OpenAI Chat Completions response format.
#[derive(Debug, Serialize)]
pub struct OpenAIResponse {
    pub id: String,
    #[serde(rename = "object")]
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIResponseMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCallOut>>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIToolCallOut {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: OpenAIFunctionCallOut,
}

#[derive(Debug, Serialize)]
pub struct OpenAIFunctionCallOut {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Transform OpenAI request to the client-agnostic gateway request.
pub fn transform_openai_to_gateway_request(
    openai_req: OpenAIRequest,
) -> Result<GatewayRequest, String> {
    reject_known_unsupported_fields(&openai_req)?;

    let (tools, tool_names) = parse_function_tools(openai_req.tools.as_ref())?;
    let tools = apply_tool_choice(tools, &tool_names, openai_req.tool_choice.as_ref())?;

    let mut messages: Vec<Message> = Vec::new();
    let mut pending_tool_call_ids: HashSet<String> = HashSet::new();

    // Process messages
    for msg in openai_req.messages {
        let OpenAIMessage {
            role,
            content,
            tool_calls,
            tool_call_id,
            ..
        } = msg;

        match role.as_str() {
            "system" | "developer" => {
                let content =
                    openai_message_content_to_message_content_for_role(content, role.as_str())?;
                messages.push(Message { role, content });
            }
            "user" => {
                let content =
                    openai_message_content_to_message_content_for_role(content, role.as_str())?;
                messages.push(Message { role, content });
            }
            "assistant" => {
                let has_tool_calls = tool_calls.is_some();

                let mut blocks = Vec::new();
                if let Some(content) = content.as_ref() {
                    match content {
                        OpenAIContent::String(text) => {
                            if !text.is_empty() {
                                blocks.push(ContentBlock::text(text.clone(), None));
                            }
                        }
                        OpenAIContent::Parts(_) => {
                            blocks.extend(openai_content_to_blocks(content)?);
                        }
                    }
                }

                if let Some(tool_calls) = tool_calls {
                    for tool_call in tool_calls {
                        if tool_call.r#type != "function" {
                            return Err("Only function tool calls are supported".to_string());
                        }

                        let arguments: serde_json::Value =
                            serde_json::from_str(&tool_call.function.arguments).map_err(|_| {
                                "tool_calls[].function.arguments must be a JSON string".to_string()
                            })?;

                        let tool_call_id = tool_call.id;
                        blocks.push(ContentBlock::tool_use(
                            tool_call_id.clone(),
                            tool_call.function.name,
                            arguments,
                        ));
                        pending_tool_call_ids.insert(tool_call_id);
                    }
                }

                let content = if has_tool_calls {
                    MessageContent::Blocks(blocks)
                } else {
                    openai_message_content_to_message_content(content)?
                };

                messages.push(Message {
                    role: "assistant".to_string(),
                    content,
                });
            }
            "tool" => {
                let Some(tool_call_id) = tool_call_id else {
                    return Err("tool messages must include tool_call_id".to_string());
                };

                if !pending_tool_call_ids.remove(&tool_call_id) {
                    return Err(
                        "tool messages must reference a prior assistant tool call".to_string()
                    );
                }

                let text = match content.as_ref() {
                    None => String::new(),
                    Some(content) => openai_content_to_text_only(content)?,
                };

                messages.push(Message {
                    role: "user".to_string(),
                    content: MessageContent::Blocks(vec![ContentBlock::Known(
                        KnownContentBlock::ToolResult {
                            tool_use_id: tool_call_id,
                            content: ToolResultContent::Text(text),
                            is_error: false,
                            cache_control: None,
                        },
                    )]),
                });
            }
            other => return Err(format!("Unsupported message role: {other}")),
        }
    }

    Ok(GatewayRequest {
        model: openai_req.model,
        messages,
        max_output_tokens: openai_req.max_tokens.unwrap_or(4096),
        reasoning: None,
        temperature: openai_req.temperature,
        top_p: openai_req.top_p,
        top_k: None,
        stop_sequences: openai_req.stop,
        stream: openai_req.stream,
        metadata: None,
        system: None,
        tools,
    })
}

/// Transform a gateway response to OpenAI format (sync only; streaming is handled in S2).
pub fn transform_gateway_response_to_openai(
    anthropic_resp: GatewayResponse,
    model: String,
) -> OpenAIResponse {
    // Extract text and tool-use content from content blocks, suppressing thinking blocks.
    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<OpenAIToolCallOut> = Vec::new();

    for block in &anthropic_resp.content {
        match block {
            ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
                if !text.is_empty() {
                    text_parts.push(text.clone());
                }
            }
            ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input }) => {
                let arguments = serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string());
                tool_calls.push(OpenAIToolCallOut {
                    id: id.clone(),
                    r#type: "function".to_string(),
                    function: OpenAIFunctionCallOut {
                        name: name.clone(),
                        arguments,
                    },
                });
            }
            ContentBlock::Known(KnownContentBlock::Thinking { .. }) => {}
            _ => {}
        }
    }

    let content_text = text_parts.join("\n");
    let content = if content_text.is_empty() {
        None
    } else {
        Some(content_text)
    };

    let finish_reason = anthropic_resp.stop_reason.as_ref().map(|reason| {
        match reason.as_str() {
            "tool_use" => "tool_calls",
            "end_turn" => "stop",
            "max_tokens" => "length",
            "stop_sequence" => "stop",
            _ => "stop",
        }
        .to_string()
    });

    OpenAIResponse {
        id: anthropic_resp.id,
        object: "chat.completion".to_string(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        model,
        choices: vec![OpenAIChoice {
            index: 0,
            message: OpenAIResponseMessage {
                role: "assistant".to_string(),
                content,
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
            },
            finish_reason,
        }],
        usage: OpenAIUsage {
            prompt_tokens: anthropic_resp.usage.input_tokens,
            completion_tokens: anthropic_resp.usage.output_tokens,
            total_tokens: anthropic_resp.usage.input_tokens + anthropic_resp.usage.output_tokens,
        },
    }
}

#[pin_project]
pub struct OpenAIChatCompletionsChunkStream<S> {
    #[pin]
    inner: S,
    buffer: String,
    queue: VecDeque<Bytes>,
    model: String,
    include_usage: bool,
    stream_id: String,
    created: u64,
    tool_blocks: HashMap<usize, OpenAIStreamToolBlock>,
    next_tool_call_index: u32,
    finish_reason: Option<String>,
    last_usage: Option<OpenAIUsage>,
    finalized: bool,
}

#[derive(Debug, Clone)]
struct OpenAIStreamToolBlock {
    id: String,
    name: String,
    tool_call_index: u32,
}

impl<S> OpenAIChatCompletionsChunkStream<S> {
    pub fn new(stream: S, model: String, include_usage: bool) -> Self {
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            inner: stream,
            buffer: String::new(),
            queue: VecDeque::new(),
            model,
            include_usage,
            stream_id: format!("chatcmpl_stream_{}", created),
            created,
            tool_blocks: HashMap::new(),
            next_tool_call_index: 0,
            finish_reason: None,
            last_usage: None,
            finalized: false,
        }
    }

    fn enqueue_chunk(&mut self, chunk: OpenAIChatCompletionChunk) {
        if let Ok(json) = serde_json::to_string(&chunk) {
            self.queue
                .push_back(Bytes::from(format!("data: {}\n\n", json)));
        }
    }

    fn enqueue_done(&mut self) {
        self.queue.push_back(Bytes::from("data: [DONE]\n\n"));
    }

    fn drain_completed_events(&mut self, flush_all: bool) -> Vec<SseEvent> {
        let parse_upto = if flush_all {
            if self.buffer.is_empty() {
                None
            } else {
                Some(self.buffer.len())
            }
        } else {
            self.buffer.rfind("\n\n").map(|idx| idx + 2)
        };

        let Some(end) = parse_upto else {
            return Vec::new();
        };

        let complete_portion = self.buffer[..end].to_string();
        if !flush_all {
            self.buffer = self.buffer[end..].to_string();
        } else {
            self.buffer.clear();
        }

        parse_sse_events(&complete_portion)
    }

    fn emit_text_delta(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.enqueue_chunk(OpenAIChatCompletionChunk::delta(
            &self.stream_id,
            self.created,
            &self.model,
            OpenAIDelta {
                role: None,
                content: Some(text.to_string()),
                tool_calls: None,
            },
            None,
            None,
        ));
    }

    fn emit_tool_call_start(&mut self, tool: &OpenAIStreamToolBlock) {
        self.enqueue_chunk(OpenAIChatCompletionChunk::delta(
            &self.stream_id,
            self.created,
            &self.model,
            OpenAIDelta {
                role: None,
                content: None,
                tool_calls: Some(vec![OpenAIChunkToolCallDelta {
                    index: tool.tool_call_index,
                    id: tool.id.clone(),
                    r#type: "function".to_string(),
                    function: OpenAIChunkToolCallFunctionDelta {
                        name: Some(tool.name.clone()),
                        arguments: Some(String::new()),
                    },
                }]),
            },
            None,
            None,
        ));
    }

    fn emit_tool_call_arguments_delta(&mut self, tool: &OpenAIStreamToolBlock, partial: &str) {
        if partial.is_empty() {
            return;
        }

        self.enqueue_chunk(OpenAIChatCompletionChunk::delta(
            &self.stream_id,
            self.created,
            &self.model,
            OpenAIDelta {
                role: None,
                content: None,
                tool_calls: Some(vec![OpenAIChunkToolCallDelta {
                    index: tool.tool_call_index,
                    id: tool.id.clone(),
                    r#type: "function".to_string(),
                    function: OpenAIChunkToolCallFunctionDelta {
                        name: None,
                        arguments: Some(partial.to_string()),
                    },
                }]),
            },
            None,
            None,
        ));
    }

    fn emit_final_finish_reason(&mut self) {
        let finish_reason = self.finish_reason.clone();
        self.enqueue_chunk(OpenAIChatCompletionChunk::delta(
            &self.stream_id,
            self.created,
            &self.model,
            OpenAIDelta::default(),
            finish_reason,
            None,
        ));
    }

    fn emit_usage_chunk_if_requested(&mut self) {
        if !self.include_usage {
            return;
        }
        let usage = self.last_usage.clone().unwrap_or(OpenAIUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        });

        self.enqueue_chunk(OpenAIChatCompletionChunk {
            id: self.stream_id.clone(),
            object: "chat.completion.chunk",
            created: self.created,
            model: self.model.clone(),
            choices: Vec::new(),
            usage: Some(usage),
        });
    }

    fn consume_sse_event(&mut self, evt: &SseEvent) {
        let Some(name) = evt.event.as_deref() else {
            return;
        };

        match name {
            "content_block_start" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&evt.data) else {
                    return;
                };
                let Some(content_block) = json.get("content_block") else {
                    return;
                };
                let Some(cb_type) = content_block.get("type").and_then(|v| v.as_str()) else {
                    return;
                };
                if cb_type != "tool_use" {
                    return;
                }

                let Some(index) = json.get("index").and_then(|v| v.as_u64()) else {
                    return;
                };
                let Some(id) = content_block.get("id").and_then(|v| v.as_str()) else {
                    return;
                };
                let Some(tool_name) = content_block.get("name").and_then(|v| v.as_str()) else {
                    return;
                };

                let tool = OpenAIStreamToolBlock {
                    id: id.to_string(),
                    name: tool_name.to_string(),
                    tool_call_index: self.next_tool_call_index,
                };
                self.next_tool_call_index += 1;

                self.emit_tool_call_start(&tool);
                self.tool_blocks.insert(index as usize, tool);
            }
            "content_block_delta" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&evt.data) else {
                    return;
                };
                let Some(delta) = json.get("delta") else {
                    return;
                };
                let Some(delta_type) = delta.get("type").and_then(|v| v.as_str()) else {
                    return;
                };

                match delta_type {
                    "text_delta" => {
                        if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                            self.emit_text_delta(text);
                        }
                    }
                    "input_json_delta" => {
                        let Some(index) = json.get("index").and_then(|v| v.as_u64()) else {
                            return;
                        };
                        let Some(partial) = delta.get("partial_json").and_then(|v| v.as_str())
                        else {
                            return;
                        };
                        if let Some(tool) = self.tool_blocks.get(&(index as usize)).cloned() {
                            self.emit_tool_call_arguments_delta(&tool, partial);
                        }
                    }
                    _ => {}
                }
            }
            "content_block_stop" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&evt.data) else {
                    return;
                };
                let Some(index) = json.get("index").and_then(|v| v.as_u64()) else {
                    return;
                };
                self.tool_blocks.remove(&(index as usize));
            }
            "message_delta" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&evt.data) else {
                    return;
                };

                let stop_reason = json
                    .get("delta")
                    .and_then(|d| d.get("stop_reason"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if let Some(usage) = json.get("usage") {
                    let input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64());
                    let output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64());
                    if let (Some(input_tokens), Some(output_tokens)) = (input_tokens, output_tokens)
                    {
                        self.last_usage = Some(OpenAIUsage {
                            prompt_tokens: input_tokens as u32,
                            completion_tokens: output_tokens as u32,
                            total_tokens: (input_tokens + output_tokens) as u32,
                        });
                    }
                }

                if stop_reason.is_empty() {
                    return;
                }

                self.finish_reason = Some(
                    match stop_reason {
                        "tool_use" => "tool_calls",
                        "max_tokens" => "length",
                        _ => "stop",
                    }
                    .to_string(),
                );
            }
            _ => {}
        }
    }

    fn finalize_once(&mut self) {
        if self.finalized {
            return;
        }
        self.finalized = true;

        if self.finish_reason.is_none() {
            self.finish_reason = Some("stop".to_string());
        }

        self.emit_final_finish_reason();
        self.emit_usage_chunk_if_requested();
        self.enqueue_done();
    }
}

impl<S, E> Stream for OpenAIChatCompletionsChunkStream<S>
where
    S: Stream<Item = Result<Bytes, E>>,
{
    type Item = Result<Bytes, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // SAFETY: this method never moves pinned fields. `inner` is polled via a pinned
        // reference and other fields are mutated in place.
        let this = unsafe { self.get_unchecked_mut() };

        if let Some(item) = this.queue.pop_front() {
            return Poll::Ready(Some(Ok(item)));
        }

        let mut inner = unsafe { Pin::new_unchecked(&mut this.inner) };
        match inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                if let Ok(text) = std::str::from_utf8(bytes.as_ref()) {
                    this.buffer.push_str(text);
                    let events = this.drain_completed_events(false);
                    for evt in events {
                        this.consume_sse_event(&evt);
                    }
                }

                if let Some(item) = this.queue.pop_front() {
                    Poll::Ready(Some(Ok(item)))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                let events = this.drain_completed_events(true);
                for evt in events {
                    this.consume_sse_event(&evt);
                }
                this.finalize_once();

                if let Some(item) = this.queue.pop_front() {
                    Poll::Ready(Some(Ok(item)))
                } else {
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAIChatCompletionChunk {
    id: String,
    #[serde(rename = "object")]
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<OpenAIChunkChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<OpenAIUsage>,
}

impl OpenAIChatCompletionChunk {
    fn delta(
        id: &str,
        created: u64,
        model: &str,
        delta: OpenAIDelta,
        finish_reason: Option<String>,
        usage: Option<OpenAIUsage>,
    ) -> Self {
        Self {
            id: id.to_string(),
            object: "chat.completion.chunk",
            created,
            model: model.to_string(),
            choices: vec![OpenAIChunkChoice {
                index: 0,
                delta,
                finish_reason,
            }],
            usage,
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAIChunkChoice {
    index: u32,
    delta: OpenAIDelta,
    // MUST serialize as null or string for every chunk.
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct OpenAIDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIChunkToolCallDelta>>,
}

#[derive(Debug, Serialize)]
struct OpenAIChunkToolCallDelta {
    index: u32,
    id: String,
    #[serde(rename = "type")]
    r#type: String,
    function: OpenAIChunkToolCallFunctionDelta,
}

#[derive(Debug, Serialize)]
struct OpenAIChunkToolCallFunctionDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<String>,
}

fn reject_known_unsupported_fields(openai_req: &OpenAIRequest) -> Result<(), String> {
    if openai_req.n.is_some() {
        return Err("Unsupported field: n".to_string());
    }
    if openai_req.logprobs.is_some() {
        return Err("Unsupported field: logprobs".to_string());
    }
    if openai_req.audio.is_some() {
        return Err("Unsupported field: audio".to_string());
    }
    if openai_req.modalities.is_some() {
        return Err("Unsupported field: modalities".to_string());
    }
    Ok(())
}

fn openai_content_to_text_only(content: &OpenAIContent) -> Result<String, String> {
    match content {
        OpenAIContent::String(s) => Ok(s.clone()),
        OpenAIContent::Parts(parts) => {
            let mut out = Vec::new();
            for part in parts {
                match part {
                    OpenAIContentPart::Text { text } => out.push(text.clone()),
                    OpenAIContentPart::ImageUrl { .. } => {
                        return Err(
                            "tool messages only support string content or text parts".to_string()
                        );
                    }
                }
            }
            Ok(out.join("\n"))
        }
    }
}

fn openai_message_content_to_message_content(
    content: Option<OpenAIContent>,
) -> Result<MessageContent, String> {
    openai_message_content_to_message_content_for_role(content, "user")
}

fn openai_message_content_to_message_content_for_role(
    content: Option<OpenAIContent>,
    role: &str,
) -> Result<MessageContent, String> {
    let Some(content) = content else {
        return Ok(MessageContent::Text(String::new()));
    };

    match content {
        OpenAIContent::String(text) => Ok(MessageContent::Text(text)),
        OpenAIContent::Parts(parts) => {
            let blocks = openai_parts_to_blocks(parts, role != "system" && role != "developer")?;
            if blocks.is_empty() {
                Ok(MessageContent::Text(String::new()))
            } else {
                Ok(MessageContent::Blocks(blocks))
            }
        }
    }
}

fn openai_content_to_blocks(content: &OpenAIContent) -> Result<Vec<ContentBlock>, String> {
    let OpenAIContent::Parts(parts) = content else {
        return Ok(Vec::new());
    };
    openai_parts_to_blocks(parts.clone(), true)
}

fn openai_parts_to_blocks(
    parts: Vec<OpenAIContentPart>,
    allow_images: bool,
) -> Result<Vec<ContentBlock>, String> {
    let mut blocks: Vec<ContentBlock> = Vec::new();
    for part in parts {
        match part {
            OpenAIContentPart::Text { text } => {
                blocks.push(ContentBlock::text(text, None));
            }
            OpenAIContentPart::ImageUrl { image_url } => {
                if !allow_images {
                    return Err(
                        "system and developer messages do not support image_url parts".to_string(),
                    );
                }
                // Parse data URL or external URL
                if image_url.url.starts_with("data:") {
                    // data:image/png;base64,iVBORw0KG...
                    if let Some(comma_idx) = image_url.url.find(',') {
                        let header = &image_url.url[..comma_idx];
                        let data = &image_url.url[comma_idx + 1..];

                        // Determine media type from header
                        let media_type = if header.contains("image/jpeg") {
                            "image/jpeg"
                        } else if header.contains("image/png") {
                            "image/png"
                        } else if header.contains("image/gif") {
                            "image/gif"
                        } else if header.contains("image/webp") {
                            "image/webp"
                        } else {
                            "image/png" // default
                        };

                        blocks.push(ContentBlock::image(crate::models::ImageSource {
                            r#type: "base64".to_string(),
                            media_type: Some(media_type.to_string()),
                            data: Some(data.to_string()),
                            url: None,
                        }));
                    }
                } else {
                    blocks.push(ContentBlock::image(crate::models::ImageSource {
                        r#type: "url".to_string(),
                        media_type: None,
                        data: None,
                        url: Some(image_url.url),
                    }));
                }
            }
        }
    }

    Ok(blocks)
}

fn parse_function_tools(
    tools: Option<&Vec<serde_json::Value>>,
) -> Result<(Option<Vec<Tool>>, HashSet<String>), String> {
    let Some(tools) = tools else {
        return Ok((None, HashSet::new()));
    };

    let mut out = Vec::new();
    let mut names = HashSet::new();

    for tool in tools {
        let tool_in: OpenAIToolIn = serde_json::from_value(tool.clone())
            .map_err(|_| "Invalid tool definition".to_string())?;
        if tool_in.r#type != "function" {
            return Err("Only function tools are supported".to_string());
        }
        names.insert(tool_in.function.name.clone());
        out.push(Tool {
            r#type: Some("function".to_string()),
            name: Some(tool_in.function.name),
            description: tool_in.function.description,
            input_schema: tool_in.function.parameters,
        });
    }

    Ok((Some(out), names))
}

fn apply_tool_choice(
    tools: Option<Vec<Tool>>,
    tool_names: &HashSet<String>,
    tool_choice: Option<&serde_json::Value>,
) -> Result<Option<Vec<Tool>>, String> {
    let Some(tool_choice) = tool_choice else {
        return Ok(tools);
    };

    match tool_choice {
        serde_json::Value::String(choice) => match choice.as_str() {
            "none" => Ok(None),
            "auto" => Ok(tools),
            "required" => {
                if tools.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                    Err("tool_choice=\"required\" requires tools to be non-empty".to_string())
                } else {
                    Ok(tools)
                }
            }
            _ => Err("Unsupported tool_choice string".to_string()),
        },
        serde_json::Value::Object(obj) => {
            let Some(kind) = obj.get("type").and_then(|v| v.as_str()) else {
                return Err("Unsupported tool_choice object".to_string());
            };
            if kind != "function" {
                return Err("Only function tool_choice is supported".to_string());
            }
            let Some(name) = obj
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
            else {
                return Err("tool_choice.function.name is required".to_string());
            };
            if !tool_names.contains(name) {
                return Err("tool_choice requested unknown function".to_string());
            }
            let filtered_tools = tools.map(|tools| {
                tools
                    .into_iter()
                    .filter(|tool| tool.name.as_deref() == Some(name))
                    .collect::<Vec<_>>()
            });

            Ok(filtered_tools)
        }
        _ => Err("Unsupported tool_choice value".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::GatewayUsage;
    use futures::StreamExt;
    use serde_json::json;

    fn base_req(extra: serde_json::Value) -> serde_json::Value {
        let mut base = json!({
            "model": "gpt-test",
            "messages": [{"role":"user","content":"hi"}]
        });
        if let (Some(base_obj), Some(extra_obj)) = (base.as_object_mut(), extra.as_object()) {
            for (k, v) in extra_obj {
                base_obj.insert(k.clone(), v.clone());
            }
        }
        base
    }

    #[test]
    fn rejects_known_but_unsupported_top_level_fields() {
        for field in ["n", "logprobs", "audio", "modalities"] {
            let mut extra = serde_json::Map::new();
            extra.insert(field.to_string(), json!(true));
            let req: OpenAIRequest =
                serde_json::from_value(base_req(serde_json::Value::Object(extra))).unwrap();
            assert!(
                transform_openai_to_gateway_request(req).is_err(),
                "field {field} should be rejected"
            );
        }
    }

    #[test]
    fn unknown_top_level_fields_are_ignored() {
        let req: OpenAIRequest =
            serde_json::from_value(base_req(json!({ "unknown_field": 123 }))).unwrap();
        assert!(transform_openai_to_gateway_request(req).is_ok());
    }

    #[test]
    fn ordered_instruction_roles_preserve_text_only_content_in_messages() {
        let req: OpenAIRequest = serde_json::from_value(json!({
            "model":"gpt-test",
            "messages":[
                {
                    "role":"system",
                    "content":[
                        {"type":"text","text":"sys"},
                        {"type":"text","text":"sys-2"}
                    ]
                },
                {
                    "role":"developer",
                    "content":[
                        {"type":"text","text":"dev"},
                        {"type":"text","text":"dev-2"}
                    ]
                },
                {
                    "role":"user",
                    "content":[
                        {"type":"text","text":"hello"},
                        {"type":"image_url","image_url":{"url":"https://example.com/user.png"}}
                    ]
                }
            ]
        }))
        .unwrap();

        let gw = transform_openai_to_gateway_request(req).unwrap();
        assert!(gw.system.is_none());
        assert_eq!(gw.messages.len(), 3);

        for (idx, role) in ["system", "developer", "user"].iter().enumerate() {
            assert_eq!(&gw.messages[idx].role, role);
            match &gw.messages[idx].content {
                MessageContent::Blocks(blocks) => {
                    assert_eq!(blocks.len(), 2);
                    assert_eq!(
                        blocks[0].as_text(),
                        Some(if idx == 0 {
                            "sys"
                        } else if idx == 1 {
                            "dev"
                        } else {
                            "hello"
                        })
                    );
                    if idx == 2 {
                        assert!(matches!(
                            &blocks[1],
                            ContentBlock::Known(KnownContentBlock::Image { source })
                                if source.url.as_deref() == Some("https://example.com/user.png")
                        ));
                    } else {
                        assert_eq!(
                            blocks[1].as_text(),
                            Some(if idx == 0 { "sys-2" } else { "dev-2" })
                        );
                    }
                }
                other => panic!("expected blocks, got {other:?}"),
            }
        }
    }

    #[test]
    fn system_and_developer_messages_reject_image_url_parts() {
        let req: OpenAIRequest = serde_json::from_value(json!({
            "model":"gpt-test",
            "messages":[
                {
                    "role":"system",
                    "content":[
                        {"type":"text","text":"sys"},
                        {"type":"image_url","image_url":{"url":"https://example.com/system.png"}}
                    ]
                }
            ]
        }))
        .unwrap();

        let err = transform_openai_to_gateway_request(req).unwrap_err();
        assert!(err.contains("do not support image_url parts"));
    }

    #[test]
    fn rejects_non_function_tools() {
        let req: OpenAIRequest = serde_json::from_value(base_req(json!({
            "tools": [{"type":"web_search"}]
        })))
        .unwrap();
        assert!(transform_openai_to_gateway_request(req).is_err());
    }

    #[test]
    fn tool_choice_none_drops_tools() {
        let req: OpenAIRequest = serde_json::from_value(base_req(json!({
            "tools": [{"type":"function","function":{"name":"f","parameters":{"type":"object"}}}],
            "tool_choice": "none"
        })))
        .unwrap();
        let gw = transform_openai_to_gateway_request(req).unwrap();
        assert!(gw.tools.is_none());
    }

    #[test]
    fn tool_choice_required_requires_tools() {
        let req: OpenAIRequest =
            serde_json::from_value(base_req(json!({ "tool_choice": "required" }))).unwrap();
        assert!(transform_openai_to_gateway_request(req).is_err());
    }

    #[test]
    fn tool_choice_explicit_function_validates_name_exists() {
        let req_missing: OpenAIRequest = serde_json::from_value(base_req(json!({
            "tools": [{"type":"function","function":{"name":"f1","parameters":{"type":"object"}}}],
            "tool_choice": {"type":"function","function":{"name":"nope"}}
        })))
        .unwrap();
        assert!(transform_openai_to_gateway_request(req_missing).is_err());

        let req_ok: OpenAIRequest = serde_json::from_value(base_req(json!({
            "tools": [{"type":"function","function":{"name":"f1","parameters":{"type":"object"}}}],
            "tool_choice": {"type":"function","function":{"name":"f1"}}
        })))
        .unwrap();
        let gw = transform_openai_to_gateway_request(req_ok).unwrap();
        let tools = gw.tools.expect("expected filtered tools");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name.as_deref(), Some("f1"));
    }

    #[test]
    fn tool_messages_require_tool_call_id_and_map_to_tool_result() {
        let req_missing_id: OpenAIRequest = serde_json::from_value(json!({
            "model":"gpt-test",
            "messages":[{"role":"tool","content":"x"}]
        }))
        .unwrap();
        assert!(transform_openai_to_gateway_request(req_missing_id).is_err());

        let req: OpenAIRequest = serde_json::from_value(json!({
            "model":"gpt-test",
            "messages":[
                {"role":"assistant","tool_calls":[{"id":"call_1","type":"function","function":{"name":"F","arguments":"{\"x\":1}"}}]},
                {"role":"tool","tool_call_id":"call_1","content":"ok"}
            ]
        }))
        .unwrap();
        let gw = transform_openai_to_gateway_request(req).unwrap();
        assert_eq!(gw.messages.len(), 2);
        assert_eq!(gw.messages[1].role, "user");
        match &gw.messages[1].content {
            MessageContent::Blocks(blocks) => match &blocks[0] {
                ContentBlock::Known(KnownContentBlock::ToolResult {
                    tool_use_id,
                    content,
                    ..
                }) => {
                    assert_eq!(tool_use_id, "call_1");
                    assert!(matches!(content, ToolResultContent::Text(t) if t == "ok"));
                }
                _ => panic!("expected tool_result block"),
            },
            _ => panic!("expected blocks"),
        }
    }

    #[test]
    fn tool_messages_reject_mismatched_prior_tool_call_id() {
        let req: OpenAIRequest = serde_json::from_value(json!({
            "model":"gpt-test",
            "messages":[
                {"role":"assistant","tool_calls":[{"id":"call_1","type":"function","function":{"name":"F","arguments":"{\"x\":1}"}}]},
                {"role":"tool","tool_call_id":"call_2","content":"ok"}
            ]
        }))
        .unwrap();

        assert!(transform_openai_to_gateway_request(req).is_err());
    }

    #[test]
    fn tool_messages_reject_image_url_parts() {
        let req: OpenAIRequest = serde_json::from_value(json!({
            "model":"gpt-test",
            "messages":[
                {
                    "role":"tool",
                    "tool_call_id":"call_1",
                    "content":[{"type":"image_url","image_url":{"url":"https://example.com/x.png"}}]
                }
            ]
        }))
        .unwrap();

        assert!(transform_openai_to_gateway_request(req).is_err());
    }

    #[test]
    fn assistant_tool_calls_map_to_internal_tool_use_blocks() {
        let req: OpenAIRequest = serde_json::from_value(json!({
            "model":"gpt-test",
            "messages":[
                {"role":"assistant","tool_calls":[{"id":"call_1","type":"function","function":{"name":"F","arguments":"{\"x\":1}"}}]},
                {"role":"tool","tool_call_id":"call_1","content":"done"}
            ]
        }))
        .unwrap();

        let gw = transform_openai_to_gateway_request(req).unwrap();
        assert_eq!(gw.messages.len(), 2);
        assert_eq!(gw.messages[0].role, "assistant");
        match &gw.messages[0].content {
            MessageContent::Blocks(blocks) => {
                assert!(blocks.iter().any(|b| matches!(b, ContentBlock::Known(KnownContentBlock::ToolUse { id, name, .. }) if id == "call_1" && name == "F")));
            }
            _ => panic!("expected blocks"),
        }
    }

    #[test]
    fn sync_response_suppresses_thinking_and_maps_tool_use_to_tool_calls_and_finish_reason() {
        let resp = GatewayResponse {
            id: "resp_1".to_string(),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![
                ContentBlock::thinking(json!({"type":"thinking","text":"secret"})),
                ContentBlock::tool_use("call_1".to_string(), "F".to_string(), json!({"x":1})),
                ContentBlock::text("hi".to_string(), None),
            ],
            model: "internal".to_string(),
            stop_reason: Some("tool_use".to_string()),
            stop_sequence: None,
            usage: GatewayUsage {
                input_tokens: 1,
                output_tokens: 2,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        };

        let out = transform_gateway_response_to_openai(resp, "public-model".to_string());
        assert_eq!(out.model, "public-model");
        assert_eq!(out.choices.len(), 1);
        assert_eq!(out.choices[0].message.role, "assistant");
        assert_eq!(out.choices[0].finish_reason.as_deref(), Some("tool_calls"));

        let msg = &out.choices[0].message;
        assert_eq!(msg.content.as_deref(), Some("hi"));
        assert!(msg
            .tool_calls
            .as_ref()
            .unwrap()
            .iter()
            .any(|tc| tc.id == "call_1"
                && tc.function.name == "F"
                && tc.function.arguments == "{\"x\":1}"));
    }

    fn anthropic_sse(event: &str, payload: serde_json::Value) -> bytes::Bytes {
        bytes::Bytes::from(format!(
            "event: {event}\ndata: {}\n\n",
            serde_json::to_string(&payload).unwrap()
        ))
    }

    async fn collect_openai_sse<S, E>(mut stream: OpenAIChatCompletionsChunkStream<S>) -> String
    where
        S: futures::stream::Stream<Item = Result<bytes::Bytes, E>> + Unpin,
    {
        let mut out = Vec::new();
        while let Some(item) = stream.next().await {
            let chunk = match item {
                Ok(chunk) => chunk,
                Err(_) => panic!("unexpected stream error"),
            };
            out.extend_from_slice(chunk.as_ref());
        }
        String::from_utf8(out).unwrap()
    }

    #[tokio::test]
    async fn chunk_stream_outputs_openai_data_lines_only() {
        let input = futures::stream::iter(vec![
            Ok::<_, std::convert::Infallible>(anthropic_sse(
                "content_block_delta",
                json!({
                    "type":"content_block_delta",
                    "index": 0,
                    "delta": { "type":"text_delta", "text":"hello" }
                }),
            )),
            Ok::<_, std::convert::Infallible>(anthropic_sse(
                "content_block_delta",
                json!({
                    "type":"content_block_delta",
                    "index": 0,
                    "delta": { "type":"text_delta", "text":" world" }
                }),
            )),
            Ok::<_, std::convert::Infallible>(anthropic_sse(
                "message_delta",
                json!({
                    "type":"message_delta",
                    "delta": { "stop_reason":"end_turn", "stop_sequence": null },
                    "usage": { "input_tokens": 10, "output_tokens": 3 }
                }),
            )),
        ]);

        let text = collect_openai_sse(OpenAIChatCompletionsChunkStream::new(
            input,
            "public-model".to_string(),
            false,
        ))
        .await;

        assert!(
            !text.contains("\nevent: "),
            "should not emit SSE event framing"
        );
        assert!(
            !text.contains("content_block_start"),
            "should not leak Anthropic event names"
        );
        assert!(text.contains("\"content\":\"hello\""));
        assert!(text.contains("\"content\":\" world\""));
        assert!(text.ends_with("data: [DONE]\n\n"));
    }

    #[tokio::test]
    async fn chunk_stream_emits_usage_chunk_when_requested() {
        let input = futures::stream::iter(vec![Ok::<_, std::convert::Infallible>(anthropic_sse(
            "message_delta",
            json!({
                "type":"message_delta",
                "delta": { "stop_reason":"end_turn", "stop_sequence": null },
                "usage": { "input_tokens": 4, "output_tokens": 6 }
            }),
        ))]);

        let text = collect_openai_sse(OpenAIChatCompletionsChunkStream::new(
            input,
            "public-model".to_string(),
            true,
        ))
        .await;

        let mut saw_usage_chunk = false;
        for block in text.split("\n\n").filter(|b| !b.trim().is_empty()) {
            let Some(payload) = block.strip_prefix("data: ") else {
                continue;
            };
            if payload == "[DONE]" {
                continue;
            }
            let value: serde_json::Value = serde_json::from_str(payload).unwrap();
            if value
                .get("choices")
                .and_then(|c| c.as_array())
                .is_some_and(|a| a.is_empty())
                && value.get("usage").is_some()
            {
                saw_usage_chunk = true;
            }
        }
        assert!(
            saw_usage_chunk,
            "include_usage=true must emit a usage chunk"
        );
    }
}
