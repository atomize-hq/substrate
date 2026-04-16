use crate::core::{GatewayRequest, GatewayResponse};
use crate::models::{
    ContentBlock, ImageSource, KnownContentBlock, Message, MessageContent, RouteType, Tool,
    ToolResultContent,
};
use crate::providers::streaming::{parse_sse_events, SseEvent};
use axum::{
    body::Body,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use futures::stream::{Stream, TryStreamExt};
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use uuid::Uuid;

use super::{
    classify_provider_error, inject_continuation_text, prefer_failure_class,
    should_inject_continuation, write_routing_info, AppError, AppState, FailureClass,
    StructuredEventTracingStream,
};

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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIResponsesRequest {
    model: String,
    input: OpenAIResponsesInput,
    #[serde(default)]
    tools: Option<Vec<OpenAIResponsesTool>>,
    #[serde(default)]
    tool_choice: Option<OpenAIResponsesToolChoice>,
    #[serde(default)]
    parallel_tool_calls: Option<bool>,
    #[serde(default)]
    text: Option<OpenAIResponsesText>,
    #[serde(default)]
    include: Option<Vec<String>>,
    #[serde(default)]
    reasoning: Option<OpenAIResponsesReasoning>,
    #[serde(default)]
    max_output_tokens: Option<u32>,
    #[serde(default)]
    metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    stop: Option<Vec<String>>,
    #[serde(default)]
    truncation: Option<serde_json::Value>,
    #[serde(default)]
    previous_response_id: Option<String>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    stream_options: Option<OpenAIResponsesStreamOptions>,
    #[serde(default)]
    service_tier: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OpenAIResponsesInput {
    Text(String),
    Items(Vec<OpenAIResponsesInputItem>),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum OpenAIResponsesInputItem {
    #[serde(rename = "message")]
    Message(OpenAIResponsesMessageItem),
    #[serde(rename = "function_call")]
    FunctionCall(OpenAIResponsesFunctionCallItem),
    #[serde(rename = "function_call_output")]
    FunctionCallOutput(OpenAIResponsesFunctionCallOutputItem),
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesMessageItem {
    role: String,
    content: Vec<OpenAIResponsesMessageContentPart>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum OpenAIResponsesMessageContentPart {
    #[serde(rename = "input_text")]
    InputText { text: String },
    #[serde(rename = "input_image")]
    InputImage {
        image_url: String,
        #[serde(default)]
        detail: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesFunctionCallOutputItem {
    call_id: String,
    output: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesFunctionCallItem {
    call_id: String,
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesTool {
    #[serde(rename = "type")]
    r#type: String,
    function: OpenAIResponsesToolFunction,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesToolFunction {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum OpenAIResponsesToolChoice {
    String(String),
    Object(OpenAIResponsesToolChoiceObject),
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIResponsesToolChoiceObject {
    #[serde(rename = "type")]
    r#type: String,
    function: OpenAIResponsesToolChoiceFunction,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIResponsesToolChoiceFunction {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesText {
    #[serde(default)]
    format: Option<OpenAIResponsesTextFormat>,
    #[serde(default)]
    verbosity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesTextFormat {
    #[serde(rename = "type")]
    r#type: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponsesReasoning {
    effort: String,
    #[serde(default)]
    summary: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIResponsesStreamOptions {
    #[allow(dead_code)]
    #[serde(default)]
    include_obfuscation: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
struct OpenAIResponsesResponse {
    id: String,
    #[serde(rename = "object")]
    object_type: String,
    status: String,
    model: String,
    output: Vec<OpenAIResponsesOutputItem>,
    usage: OpenAIResponsesUsage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIResponsesUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
enum OpenAIResponsesOutputItem {
    #[serde(rename = "message")]
    Message {
        role: String,
        content: Vec<OpenAIResponsesOutputContentPart>,
    },
    #[serde(rename = "function_call")]
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
}

#[derive(Debug, Serialize, Clone)]
struct OpenAIResponsesOutputContentPart {
    #[serde(rename = "type")]
    r#type: String,
    text: String,
    annotations: Vec<serde_json::Value>,
}

fn transform_openai_to_gateway_request(
    openai_req: OpenAIResponsesRequest,
) -> Result<GatewayRequest, String> {
    reject_known_unsupported_request_state(&openai_req)?;
    let allows_previous_response_continuation = openai_req.previous_response_id.is_some();

    let (tools, tool_names) = parse_function_tools(openai_req.tools.as_ref())?;
    let tool_choice = openai_req.tool_choice.as_ref();
    let tools = apply_tool_choice(tools, &tool_names, tool_choice)?;

    let mut messages = Vec::new();

    match openai_req.input {
        OpenAIResponsesInput::Text(text) => {
            messages.push(Message {
                role: "user".to_string(),
                content: MessageContent::Text(text),
            });
        }
        OpenAIResponsesInput::Items(items) => {
            let mut prior_function_call_ids = HashSet::new();

            for item in items {
                match item {
                    OpenAIResponsesInputItem::Message(message) => {
                        let content = responses_message_content_to_message_content(&message)?;
                        match message.role.as_str() {
                            "system" | "developer" | "user" | "assistant" => {
                                messages.push(Message {
                                    role: message.role,
                                    content,
                                });
                            }
                            other => return Err(format!("Unsupported message role: {other}")),
                        }
                    }
                    OpenAIResponsesInputItem::FunctionCall(function_call) => {
                        let call_id = function_call.call_id.trim();
                        if call_id.is_empty() {
                            return Err("function_call.call_id must not be empty".to_string());
                        }
                        let name = function_call.name.trim();
                        if name.is_empty() {
                            return Err("function_call.name must not be empty".to_string());
                        }

                        let arguments =
                            parse_responses_function_call_arguments(&function_call.arguments)?;
                        prior_function_call_ids.insert(call_id.to_string());

                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: MessageContent::Blocks(vec![ContentBlock::Known(
                                KnownContentBlock::ToolUse {
                                    id: call_id.to_string(),
                                    name: name.to_string(),
                                    input: arguments,
                                },
                            )]),
                        });
                    }
                    OpenAIResponsesInputItem::FunctionCallOutput(function_call_output) => {
                        if function_call_output.call_id.trim().is_empty() {
                            return Err(
                                "function_call_output.call_id must not be empty".to_string()
                            );
                        }
                        if !prior_function_call_ids.contains(function_call_output.call_id.trim())
                            && !allows_previous_response_continuation
                        {
                            return Err(
                                "function_call_output.call_id must reference a prior function_call item"
                                    .to_string(),
                            );
                        }

                        messages.push(Message {
                            role: "user".to_string(),
                            content: MessageContent::Blocks(vec![ContentBlock::Known(
                                KnownContentBlock::ToolResult {
                                    tool_use_id: function_call_output.call_id,
                                    content: ToolResultContent::Text(function_call_output.output),
                                    is_error: false,
                                    cache_control: None,
                                },
                            )]),
                        });
                    }
                }
            }
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert(
        OPENAI_PUBLIC_RESPONSES_METADATA_KEY.to_string(),
        serde_json::Value::Bool(true),
    );
    if let Some(parallel_tool_calls) = openai_req.parallel_tool_calls {
        metadata.insert(
            "parallel_tool_calls".to_string(),
            serde_json::Value::Bool(parallel_tool_calls),
        );
    }
    if let Some(tool_choice) = tool_choice {
        metadata.insert(
            OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY.to_string(),
            serde_json::to_value(tool_choice).map_err(|err| err.to_string())?,
        );
    }
    if let Some(max_output_tokens) = openai_req.max_output_tokens {
        metadata.insert(
            OPENAI_RESPONSES_EXPLICIT_MAX_OUTPUT_TOKENS_METADATA_KEY.to_string(),
            serde_json::Value::from(max_output_tokens),
        );
    }
    if let Some(request_metadata) = &openai_req.metadata {
        metadata.insert(
            OPENAI_RESPONSES_INPUT_METADATA_METADATA_KEY.to_string(),
            serde_json::to_value(request_metadata).map_err(|err| err.to_string())?,
        );
    }
    if let Some(truncation) = &openai_req.truncation {
        metadata.insert(
            OPENAI_RESPONSES_TRUNCATION_METADATA_KEY.to_string(),
            truncation.clone(),
        );
    }
    if let Some(previous_response_id) = &openai_req.previous_response_id {
        metadata.insert(
            OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY.to_string(),
            serde_json::Value::String(previous_response_id.clone()),
        );
    }
    if let Some(user) = &openai_req.user {
        metadata.insert(
            OPENAI_RESPONSES_USER_METADATA_KEY.to_string(),
            serde_json::Value::String(user.clone()),
        );
    }
    if let Some(stream_options) = &openai_req.stream_options {
        metadata.insert(
            OPENAI_RESPONSES_STREAM_OPTIONS_METADATA_KEY.to_string(),
            serde_json::to_value(stream_options).map_err(|err| err.to_string())?,
        );
    }
    if let Some(service_tier) = &openai_req.service_tier {
        metadata.insert(
            OPENAI_RESPONSES_SERVICE_TIER_METADATA_KEY.to_string(),
            service_tier.clone(),
        );
    }
    if let Some(reasoning) = &openai_req.reasoning {
        metadata.insert(
            OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY.to_string(),
            serde_json::Value::String(reasoning.effort.clone()),
        );
        if let Some(summary) = &reasoning.summary {
            metadata.insert(
                OPENAI_RESPONSES_REASONING_SUMMARY_METADATA_KEY.to_string(),
                serde_json::Value::String(summary.clone()),
            );
        }
    }
    if let Some(include) = &openai_req.include {
        metadata.insert(
            OPENAI_RESPONSES_INCLUDE_METADATA_KEY.to_string(),
            serde_json::Value::Array(
                include
                    .iter()
                    .cloned()
                    .map(serde_json::Value::String)
                    .collect(),
            ),
        );
    }
    if let Some(text) = &openai_req.text {
        if let Some(verbosity) = &text.verbosity {
            metadata.insert(
                OPENAI_RESPONSES_TEXT_VERBOSITY_METADATA_KEY.to_string(),
                serde_json::Value::String(verbosity.clone()),
            );
        }
    }

    Ok(GatewayRequest {
        model: openai_req.model,
        messages,
        max_output_tokens: openai_req.max_output_tokens.unwrap_or(4096),
        reasoning: openai_req
            .reasoning
            .as_ref()
            .map(|reasoning| crate::core::ReasoningConfig {
                r#type: if reasoning.effort == "none" {
                    "disabled".to_string()
                } else {
                    "enabled".to_string()
                },
                budget_tokens: None,
            }),
        temperature: openai_req.temperature,
        top_p: openai_req.top_p,
        top_k: None,
        stop_sequences: openai_req.stop,
        stream: openai_req.stream,
        metadata: if metadata.is_empty() {
            None
        } else {
            Some(metadata)
        },
        system: None,
        tools,
    })
}

fn transform_gateway_response_to_openai_response(
    gateway_response: GatewayResponse,
    model: String,
) -> OpenAIResponsesResponse {
    let mut output = Vec::new();
    let mut pending_text_parts = Vec::new();

    let flush_pending_text =
        |output: &mut Vec<OpenAIResponsesOutputItem>,
         pending_text_parts: &mut Vec<OpenAIResponsesOutputContentPart>| {
            if pending_text_parts.is_empty() {
                return;
            }

            output.push(OpenAIResponsesOutputItem::Message {
                role: "assistant".to_string(),
                content: std::mem::take(pending_text_parts),
            });
        };

    for block in &gateway_response.content {
        match block {
            ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
                if !text.is_empty() {
                    pending_text_parts.push(OpenAIResponsesOutputContentPart {
                        r#type: "output_text".to_string(),
                        text: text.clone(),
                        annotations: Vec::new(),
                    });
                }
            }
            ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input }) => {
                flush_pending_text(&mut output, &mut pending_text_parts);
                output.push(OpenAIResponsesOutputItem::FunctionCall {
                    call_id: id.clone(),
                    name: name.clone(),
                    arguments: serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string()),
                });
            }
            ContentBlock::Known(KnownContentBlock::Thinking { .. }) => {}
            _ => {}
        }
    }

    flush_pending_text(&mut output, &mut pending_text_parts);

    OpenAIResponsesResponse {
        id: gateway_response.id,
        object_type: "response".to_string(),
        status: match gateway_response.stop_reason.as_deref() {
            Some("max_tokens") => "incomplete",
            Some("error") => "failed",
            _ => "completed",
        }
        .to_string(),
        model,
        output,
        usage: OpenAIResponsesUsage {
            input_tokens: gateway_response.usage.input_tokens,
            output_tokens: gateway_response.usage.output_tokens,
            total_tokens: gateway_response.usage.input_tokens
                + gateway_response.usage.output_tokens,
        },
    }
}

#[pin_project]
struct OpenAIResponsesSseStream<S> {
    #[pin]
    inner: S,
    buffer: String,
    queue: VecDeque<Bytes>,
    model: String,
    response_id: String,
    output_items: BTreeMap<usize, OpenAIResponsesOutputItem>,
    open_items: HashMap<usize, StreamingOutputItem>,
    usage: OpenAIResponsesUsage,
    status: String,
    finalized: bool,
}

#[derive(Debug, Clone)]
enum StreamingOutputItem {
    Text {
        text: String,
    },
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
}

impl<S> OpenAIResponsesSseStream<S> {
    fn new(stream: S, model: String) -> Self {
        let response_id = format!("resp_{}", Uuid::new_v4());
        let mut queue = VecDeque::new();

        let created_response = OpenAIResponsesResponse {
            id: response_id.clone(),
            object_type: "response".to_string(),
            status: "in_progress".to_string(),
            model: model.clone(),
            output: Vec::new(),
            usage: OpenAIResponsesUsage {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            },
        };
        queue.push_back(build_sse_event(
            "response.created",
            serde_json::json!({
                "response": created_response,
            }),
        ));

        Self {
            inner: stream,
            buffer: String::new(),
            queue,
            model,
            response_id,
            output_items: BTreeMap::new(),
            open_items: HashMap::new(),
            usage: OpenAIResponsesUsage {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            },
            status: "completed".to_string(),
            finalized: false,
        }
    }

    fn enqueue_event(&mut self, event_type: &str, payload: serde_json::Value) {
        self.queue.push_back(build_sse_event(event_type, payload));
    }

    fn response_snapshot(&self, status: &str) -> OpenAIResponsesResponse {
        OpenAIResponsesResponse {
            id: self.response_id.clone(),
            object_type: "response".to_string(),
            status: status.to_string(),
            model: self.model.clone(),
            output: self.output_items.values().cloned().collect(),
            usage: self.usage.clone(),
        }
    }

    fn text_item(text: String) -> OpenAIResponsesOutputItem {
        OpenAIResponsesOutputItem::Message {
            role: "assistant".to_string(),
            content: vec![OpenAIResponsesOutputContentPart {
                r#type: "output_text".to_string(),
                text,
                annotations: Vec::new(),
            }],
        }
    }

    fn function_call_item(
        call_id: String,
        name: String,
        arguments: String,
    ) -> OpenAIResponsesOutputItem {
        OpenAIResponsesOutputItem::FunctionCall {
            call_id,
            name,
            arguments,
        }
    }

    fn emit_text_start(&mut self, output_index: usize) {
        let item = Self::text_item(String::new());
        let item_value = serde_json::to_value(&item).unwrap_or_else(|_| serde_json::json!({}));

        self.enqueue_event(
            "response.output_item.added",
            serde_json::json!({
                "response_id": self.response_id,
                "output_index": output_index,
                "item": item_value,
            }),
        );
        self.enqueue_event(
            "response.content_part.added",
            serde_json::json!({
                "response_id": self.response_id,
                "output_index": output_index,
                "content_index": 0,
                "part": {
                    "type": "output_text",
                    "text": "",
                    "annotations": []
                },
            }),
        );
    }

    fn emit_tool_start(&mut self, output_index: usize, call_id: &str, name: &str) {
        let item = Self::function_call_item(call_id.to_string(), name.to_string(), String::new());
        let item_value = serde_json::to_value(&item).unwrap_or_else(|_| serde_json::json!({}));

        self.enqueue_event(
            "response.output_item.added",
            serde_json::json!({
                "response_id": self.response_id,
                "output_index": output_index,
                "item": item_value,
            }),
        );
    }

    fn consume_sse_event(&mut self, event: &SseEvent) {
        let Some(name) = event.event.as_deref() else {
            return;
        };

        match name {
            "content_block_start" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.data) else {
                    return;
                };
                let Some(index) = json.get("index").and_then(|value| value.as_u64()) else {
                    return;
                };
                let Some(content_block) = json.get("content_block") else {
                    return;
                };
                let Some(block_type) = content_block.get("type").and_then(|value| value.as_str())
                else {
                    return;
                };
                if block_type != "tool_use" {
                    return;
                }
                let Some(call_id) = content_block.get("id").and_then(|value| value.as_str()) else {
                    return;
                };
                let Some(tool_name) = content_block.get("name").and_then(|value| value.as_str())
                else {
                    return;
                };

                let index = index as usize;
                self.open_items.insert(
                    index,
                    StreamingOutputItem::FunctionCall {
                        call_id: call_id.to_string(),
                        name: tool_name.to_string(),
                        arguments: String::new(),
                    },
                );
                self.emit_tool_start(index, call_id, tool_name);
            }
            "content_block_delta" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.data) else {
                    return;
                };
                let Some(index) = json.get("index").and_then(|value| value.as_u64()) else {
                    return;
                };
                let Some(delta) = json.get("delta") else {
                    return;
                };
                let Some(delta_type) = delta.get("type").and_then(|value| value.as_str()) else {
                    return;
                };
                let index = index as usize;

                match delta_type {
                    "text_delta" => {
                        let Some(text_delta) = delta.get("text").and_then(|value| value.as_str())
                        else {
                            return;
                        };
                        let was_absent = !self.open_items.contains_key(&index);
                        if was_absent {
                            self.open_items.insert(
                                index,
                                StreamingOutputItem::Text {
                                    text: String::new(),
                                },
                            );
                            self.emit_text_start(index);
                        }
                        if let Some(StreamingOutputItem::Text { text }) =
                            self.open_items.get_mut(&index)
                        {
                            text.push_str(text_delta);
                        }
                        self.enqueue_event(
                            "response.output_text.delta",
                            serde_json::json!({
                                "response_id": self.response_id,
                                "output_index": index,
                                "content_index": 0,
                                "delta": text_delta,
                            }),
                        );
                    }
                    "input_json_delta" => {
                        let Some(arguments_delta) =
                            delta.get("partial_json").and_then(|value| value.as_str())
                        else {
                            return;
                        };
                        let Some(StreamingOutputItem::FunctionCall {
                            call_id, arguments, ..
                        }) = self.open_items.get_mut(&index)
                        else {
                            return;
                        };
                        arguments.push_str(arguments_delta);
                        let call_id = call_id.clone();
                        self.enqueue_event(
                            "response.function_call_arguments.delta",
                            serde_json::json!({
                                "response_id": self.response_id,
                                "output_index": index,
                                "call_id": call_id,
                                "delta": arguments_delta,
                            }),
                        );
                    }
                    _ => {}
                }
            }
            "content_block_stop" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.data) else {
                    return;
                };
                let Some(index) = json.get("index").and_then(|value| value.as_u64()) else {
                    return;
                };
                self.finalize_item(index as usize);
            }
            "message_delta" => {
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&event.data) else {
                    return;
                };

                if let Some(usage) = json.get("usage") {
                    let input_tokens = usage.get("input_tokens").and_then(|value| value.as_u64());
                    let output_tokens = usage.get("output_tokens").and_then(|value| value.as_u64());
                    if let (Some(input_tokens), Some(output_tokens)) = (input_tokens, output_tokens)
                    {
                        self.usage = OpenAIResponsesUsage {
                            input_tokens: input_tokens as u32,
                            output_tokens: output_tokens as u32,
                            total_tokens: (input_tokens + output_tokens) as u32,
                        };
                    }
                }

                if let Some(stop_reason) = json
                    .get("delta")
                    .and_then(|value| value.get("stop_reason"))
                    .and_then(|value| value.as_str())
                {
                    self.status = match stop_reason {
                        "max_tokens" => "incomplete",
                        _ => "completed",
                    }
                    .to_string();
                }
            }
            "message_stop" => self.finalize_once(),
            _ => {}
        }
    }

    fn finalize_item(&mut self, output_index: usize) {
        let Some(item) = self.open_items.remove(&output_index) else {
            return;
        };

        match item {
            StreamingOutputItem::Text { text } => {
                self.enqueue_event(
                    "response.output_text.done",
                    serde_json::json!({
                        "response_id": self.response_id,
                        "output_index": output_index,
                        "content_index": 0,
                        "text": text,
                    }),
                );
                self.enqueue_event(
                    "response.content_part.done",
                    serde_json::json!({
                        "response_id": self.response_id,
                        "output_index": output_index,
                        "content_index": 0,
                        "part": {
                            "type": "output_text",
                            "text": text,
                            "annotations": []
                        },
                    }),
                );
                let item = Self::text_item(text);
                let item_value =
                    serde_json::to_value(&item).unwrap_or_else(|_| serde_json::json!({}));
                self.enqueue_event(
                    "response.output_item.done",
                    serde_json::json!({
                        "response_id": self.response_id,
                        "output_index": output_index,
                        "item": item_value,
                    }),
                );
                self.output_items.insert(output_index, item);
            }
            StreamingOutputItem::FunctionCall {
                call_id,
                name,
                arguments,
            } => {
                self.enqueue_event(
                    "response.function_call_arguments.done",
                    serde_json::json!({
                        "response_id": self.response_id,
                        "output_index": output_index,
                        "call_id": call_id,
                        "arguments": arguments,
                    }),
                );
                let item = Self::function_call_item(call_id, name, arguments);
                let item_value =
                    serde_json::to_value(&item).unwrap_or_else(|_| serde_json::json!({}));
                self.enqueue_event(
                    "response.output_item.done",
                    serde_json::json!({
                        "response_id": self.response_id,
                        "output_index": output_index,
                        "item": item_value,
                    }),
                );
                self.output_items.insert(output_index, item);
            }
        }
    }

    fn finalize_once(&mut self) {
        if self.finalized {
            return;
        }
        self.finalized = true;

        let pending_indexes = self.open_items.keys().cloned().collect::<Vec<_>>();
        for output_index in pending_indexes {
            self.finalize_item(output_index);
        }

        let completed = self.response_snapshot(&self.status);
        self.enqueue_event(
            "response.completed",
            serde_json::json!({
                "response": completed,
            }),
        );
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
        if flush_all {
            self.buffer.clear();
        } else {
            self.buffer = self.buffer[end..].to_string();
        }

        parse_sse_events(&complete_portion)
    }
}

impl<S, E> Stream for OpenAIResponsesSseStream<S>
where
    S: Stream<Item = Result<Bytes, E>>,
{
    type Item = Result<Bytes, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
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
                    for event in events {
                        this.consume_sse_event(&event);
                    }
                }

                if let Some(item) = this.queue.pop_front() {
                    Poll::Ready(Some(Ok(item)))
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
            Poll::Ready(None) => {
                let events = this.drain_completed_events(true);
                for event in events {
                    this.consume_sse_event(&event);
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

fn build_sse_event(event_type: &str, mut payload: serde_json::Value) -> Bytes {
    if let serde_json::Value::Object(map) = &mut payload {
        map.insert(
            "type".to_string(),
            serde_json::Value::String(event_type.to_string()),
        );
    }
    Bytes::from(format!(
        "event: {}\ndata: {}\n\n",
        event_type,
        serde_json::to_string(&payload)
            .unwrap_or_else(|_| "{\"type\":\"response.error\"}".to_string())
    ))
}

pub async fn handle_openai_responses(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_json): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    let openai_request: OpenAIResponsesRequest = serde_json::from_value(request_json)
        .map_err(|e| AppError::Routing(format!("Invalid request format: {}", e)))?;

    let model = openai_request.model.clone();
    let start_time = std::time::Instant::now();
    let inner = state.snapshot();
    let trace_id = state.message_tracer.new_trace_id();

    let mut gateway_request =
        transform_openai_to_gateway_request(openai_request).map_err(AppError::Routing)?;

    let decision = inner
        .router
        .route(&mut gateway_request)
        .map_err(|e| AppError::Routing(e.to_string()))?;

    let forced_provider = headers
        .get("x-provider")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if let Some(model_config) = inner
        .config
        .models
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(&decision.model_name))
    {
        let mut sorted_mappings = model_config.mappings.clone();

        if let Some(ref provider_name) = forced_provider {
            sorted_mappings.retain(|m| m.provider == *provider_name);
            if sorted_mappings.is_empty() {
                return Err(AppError::Routing(format!(
                    "Provider '{}' not found in mappings for model '{}'",
                    provider_name, decision.model_name
                )));
            }
        } else {
            sorted_mappings.sort_by_key(|m| m.priority);
        }

        let mut last_failure_class = None;

        for (idx, mapping) in sorted_mappings.iter().enumerate() {
            if let Some(provider) = inner.provider_registry.get_provider(&mapping.provider) {
                let mut routed_request = gateway_request.clone();
                routed_request.model = mapping.actual_model.clone();

                if mapping.inject_continuation_prompt
                    && decision.route_type != RouteType::Background
                {
                    if let Some(last_msg) = routed_request.messages.last_mut() {
                        if should_inject_continuation(last_msg) {
                            inject_continuation_text(last_msg);
                        }
                    }
                }

                if idx == 0 {
                    write_routing_info(
                        &mapping.actual_model,
                        &mapping.provider,
                        &decision.route_type,
                    );
                }

                if routed_request.stream == Some(true) {
                    match provider.send_message_stream(routed_request).await {
                        Ok(stream_response) => {
                            let traced_provider_stream = if trace_id.is_empty() {
                                stream_response.stream
                            } else {
                                Box::pin(StructuredEventTracingStream::new(
                                    stream_response.stream,
                                    state.message_tracer.clone(),
                                    trace_id.clone(),
                                ))
                            };

                            let responses_stream = OpenAIResponsesSseStream::new(
                                traced_provider_stream,
                                model.clone(),
                            );
                            let body_stream =
                                responses_stream.map_err(|e| std::io::Error::other(e.to_string()));
                            let body = Body::from_stream(body_stream);
                            let mut response_builder = Response::builder()
                                .status(200)
                                .header("Content-Type", "text/event-stream")
                                .header("Cache-Control", "no-cache")
                                .header("Connection", "keep-alive");

                            for (name, value) in stream_response.headers {
                                response_builder = response_builder.header(name, value);
                            }

                            return Ok(response_builder.body(body).unwrap());
                        }
                        Err(err) => {
                            last_failure_class = prefer_failure_class(
                                last_failure_class,
                                classify_provider_error(&err),
                            );
                            continue;
                        }
                    }
                } else {
                    match provider.send_message(routed_request).await {
                        Ok(response) => {
                            let latency_ms = start_time.elapsed().as_millis() as u64;
                            let tok_s =
                                (response.usage.output_tokens as f32 * 1000.0) / latency_ms as f32;
                            tracing::info!(
                                "📊 {}@{} {}ms {:.0}t/s {}tok",
                                mapping.actual_model,
                                mapping.provider,
                                latency_ms,
                                tok_s,
                                response.usage.output_tokens
                            );

                            if idx > 0 {
                                write_routing_info(
                                    &mapping.actual_model,
                                    &mapping.provider,
                                    &decision.route_type,
                                );
                            }

                            let openai_response = transform_gateway_response_to_openai_response(
                                response,
                                model.clone(),
                            );
                            return Ok(Json(openai_response).into_response());
                        }
                        Err(err) => {
                            last_failure_class = prefer_failure_class(
                                last_failure_class,
                                classify_provider_error(&err),
                            );
                            continue;
                        }
                    }
                }
            } else {
                last_failure_class =
                    prefer_failure_class(last_failure_class, FailureClass::Deployment);
            }
        }

        Err(AppError::provider_class(
            last_failure_class.unwrap_or(FailureClass::Deployment),
            format!(
                "All {} provider mappings failed for the routed model",
                sorted_mappings.len()
            ),
        ))
    } else {
        let direct_provider = if let Some(ref provider_name) = forced_provider {
            let provider = inner
                .provider_registry
                .get_provider(provider_name)
                .ok_or_else(|| {
                    AppError::Routing(format!(
                        "Provider '{}' not found for model '{}'",
                        provider_name, decision.model_name
                    ))
                })?;

            if !provider.supports_model(&decision.model_name) {
                return Err(AppError::Routing(format!(
                    "Provider '{}' does not support model '{}'",
                    provider_name, decision.model_name
                )));
            }

            Some((provider_name.as_str(), provider))
        } else {
            inner
                .provider_registry
                .get_provider_for_model(&decision.model_name)
                .ok()
                .map(|provider| ("direct", provider))
        };

        if let Some((provider_name, provider)) = direct_provider {
            tracing::info!(
                "📦 Using provider from registry (direct lookup): {}/{}",
                provider_name,
                decision.model_name
            );

            let mut routed_request = gateway_request.clone();
            routed_request.model = decision.model_name.clone();

            if routed_request.stream == Some(true) {
                let stream_response =
                    provider
                        .send_message_stream(routed_request)
                        .await
                        .map_err(|e| {
                            AppError::provider_class(
                                classify_provider_error(&e),
                                "Provider streaming request failed",
                            )
                        })?;

                let traced_provider_stream = if trace_id.is_empty() {
                    stream_response.stream
                } else {
                    Box::pin(StructuredEventTracingStream::new(
                        stream_response.stream,
                        state.message_tracer.clone(),
                        trace_id,
                    ))
                };
                let responses_stream = OpenAIResponsesSseStream::new(traced_provider_stream, model);
                let body_stream =
                    responses_stream.map_err(|e| std::io::Error::other(e.to_string()));
                let body = Body::from_stream(body_stream);
                let mut response_builder = Response::builder()
                    .status(200)
                    .header("Content-Type", "text/event-stream")
                    .header("Cache-Control", "no-cache")
                    .header("Connection", "keep-alive");

                for (name, value) in stream_response.headers {
                    response_builder = response_builder.header(name, value);
                }

                Ok(response_builder.body(body).unwrap())
            } else {
                let response = provider.send_message(routed_request).await.map_err(|e| {
                    AppError::provider_class(classify_provider_error(&e), "Provider request failed")
                })?;

                let openai_response =
                    transform_gateway_response_to_openai_response(response, model);
                Ok(Json(openai_response).into_response())
            }
        } else {
            Err(AppError::provider_class(
                FailureClass::Route,
                "Gateway route selection failed",
            ))
        }
    }
}

fn reject_known_unsupported_request_state(
    openai_req: &OpenAIResponsesRequest,
) -> Result<(), String> {
    if let Some(text) = &openai_req.text {
        if let Some(format) = &text.format {
            if format.r#type != "text" {
                return Err("Unsupported text.format.type".to_string());
            }
        }
    }

    Ok(())
}

fn parse_function_tools(
    tools: Option<&Vec<OpenAIResponsesTool>>,
) -> Result<(Option<Vec<Tool>>, HashSet<String>), String> {
    let Some(tools) = tools else {
        return Ok((None, HashSet::new()));
    };

    let mut out = Vec::new();
    let mut names = HashSet::new();

    for tool in tools {
        if tool.r#type != "function" {
            return Err("Only function tools are supported".to_string());
        }

        names.insert(tool.function.name.clone());
        out.push(Tool {
            r#type: Some("function".to_string()),
            name: Some(tool.function.name.clone()),
            description: tool.function.description.clone(),
            input_schema: tool.function.parameters.clone(),
        });
    }

    Ok((Some(out), names))
}

fn apply_tool_choice(
    tools: Option<Vec<Tool>>,
    tool_names: &HashSet<String>,
    tool_choice: Option<&OpenAIResponsesToolChoice>,
) -> Result<Option<Vec<Tool>>, String> {
    let Some(tool_choice) = tool_choice else {
        return Ok(tools);
    };

    match tool_choice {
        OpenAIResponsesToolChoice::String(choice) => match choice.as_str() {
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
        OpenAIResponsesToolChoice::Object(obj) => {
            if obj.r#type != "function" {
                return Err("Only function tool_choice is supported".to_string());
            }

            if !tool_names.contains(&obj.function.name) {
                return Err("tool_choice requested unknown function".to_string());
            }

            let filtered_tools = tools.map(|tools| {
                tools
                    .into_iter()
                    .filter(|tool| tool.name.as_deref() == Some(obj.function.name.as_str()))
                    .collect::<Vec<_>>()
            });

            Ok(filtered_tools)
        }
    }
}

fn responses_message_content_to_message_content(
    message: &OpenAIResponsesMessageItem,
) -> Result<MessageContent, String> {
    let mut blocks = Vec::new();

    for part in &message.content {
        match part {
            OpenAIResponsesMessageContentPart::InputText { text } => {
                blocks.push(ContentBlock::text(text.clone(), None));
            }
            OpenAIResponsesMessageContentPart::InputImage { image_url, detail } => {
                if let Some(detail) = detail {
                    if !matches!(detail.as_str(), "low" | "high" | "auto") {
                        return Err("Unsupported input_image.detail".to_string());
                    }
                }
                blocks.push(ContentBlock::image(image_source_from_url(image_url)?));
            }
        }
    }

    if blocks.is_empty() {
        Ok(MessageContent::Text(String::new()))
    } else {
        Ok(MessageContent::Blocks(blocks))
    }
}

fn parse_responses_function_call_arguments(arguments: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(arguments)
        .map_err(|_| "function_call.arguments must be a JSON string".to_string())
}

fn image_source_from_url(image_url: &str) -> Result<ImageSource, String> {
    if image_url.starts_with("data:") {
        if let Some(comma_idx) = image_url.find(',') {
            let header = &image_url[..comma_idx];
            let data = &image_url[comma_idx + 1..];
            let media_type = if header.contains("image/jpeg") {
                "image/jpeg"
            } else if header.contains("image/png") {
                "image/png"
            } else if header.contains("image/gif") {
                "image/gif"
            } else if header.contains("image/webp") {
                "image/webp"
            } else {
                "image/png"
            };

            Ok(ImageSource {
                r#type: "base64".to_string(),
                media_type: Some(media_type.to_string()),
                data: Some(data.to_string()),
                url: None,
            })
        } else {
            Err("Invalid data URL".to_string())
        }
    } else {
        Ok(ImageSource {
            r#type: "url".to_string(),
            media_type: None,
            data: None,
            url: Some(image_url.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ModelMapping;
    use crate::providers::ProviderRegistry;
    use crate::providers::{OpenAIProvider, OpenAIProviderConfig, OpenAITransport};
    use crate::server::openai_conformance_test_support::{
        read_json_fixture, response_text_response as response_text,
        response_with_text_and_tool as response_with_tool_and_thinking, ConformanceHarness,
        FixtureNamespace, StubProvider,
    };
    use axum::body::to_bytes;
    use axum::http::{HeaderMap, HeaderValue};
    use serde_json::json;

    #[test]
    fn string_input_shorthand_maps_to_a_user_message() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": "hello",
            "parallel_tool_calls": false
        }))
        .unwrap();

        let gateway_request = transform_openai_to_gateway_request(request).unwrap();
        assert_eq!(gateway_request.model, "gpt-test");
        assert_eq!(gateway_request.messages.len(), 1);
        assert_eq!(gateway_request.messages[0].role, "user");
        assert!(matches!(
            gateway_request.messages[0].content,
            MessageContent::Text(ref text) if text == "hello"
        ));
        assert_eq!(
            gateway_request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("parallel_tool_calls")),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            gateway_request
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("openai_public_responses")),
            Some(&serde_json::Value::Bool(true))
        );
    }

    #[test]
    fn message_items_function_calls_and_outputs_map_into_the_normalized_core() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": [
                {
                    "type": "message",
                    "role": "system",
                    "content": [
                        { "type": "input_text", "text": "one" },
                        { "type": "input_image", "image_url": "https://example.com/system.png", "detail": "auto" },
                        { "type": "input_text", "text": "two" }
                    ]
                },
                {
                    "type": "message",
                    "role": "developer",
                    "content": [
                        { "type": "input_text", "text": "dev" },
                        { "type": "input_image", "image_url": "https://example.com/dev.png", "detail": "high" }
                    ]
                },
                {
                    "type": "message",
                    "role": "user",
                    "content": [
                        { "type": "input_text", "text": "question" },
                        { "type": "input_image", "image_url": "https://example.com/image.png", "detail": "auto" }
                    ]
                },
                {
                    "type": "function_call",
                    "call_id": "call_123",
                    "name": "lookup",
                    "arguments": "{\"query\":\"question\"}"
                },
                {
                    "type": "function_call_output",
                    "call_id": "call_123",
                    "output": "tool-output"
                }
            ]
        }))
        .unwrap();

        let gateway_request = transform_openai_to_gateway_request(request).unwrap();
        assert!(gateway_request.system.is_none());
        assert_eq!(gateway_request.messages.len(), 5);

        assert_eq!(gateway_request.messages[0].role, "system");
        match &gateway_request.messages[0].content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 3);
                assert_eq!(blocks[0].as_text(), Some("one"));
                assert!(matches!(
                    &blocks[1],
                    ContentBlock::Known(KnownContentBlock::Image { source })
                        if source.url.as_deref() == Some("https://example.com/system.png")
                ));
                assert_eq!(blocks[2].as_text(), Some("two"));
            }
            _ => panic!("expected blocks"),
        }

        assert_eq!(gateway_request.messages[1].role, "developer");
        match &gateway_request.messages[1].content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(blocks[0].as_text(), Some("dev"));
                assert!(matches!(
                    &blocks[1],
                    ContentBlock::Known(KnownContentBlock::Image { source })
                        if source.url.as_deref() == Some("https://example.com/dev.png")
                ));
            }
            _ => panic!("expected blocks"),
        }

        assert_eq!(gateway_request.messages[2].role, "user");
        match &gateway_request.messages[2].content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(blocks[0].as_text(), Some("question"));
                assert!(matches!(
                    &blocks[1],
                    ContentBlock::Known(KnownContentBlock::Image { source }) if source.url.as_deref() == Some("https://example.com/image.png")
                ));
            }
            _ => panic!("expected blocks"),
        }

        assert_eq!(gateway_request.messages[3].role, "assistant");
        assert!(matches!(
            gateway_request.messages[3].content,
            MessageContent::Blocks(ref blocks)
                if matches!(
                    &blocks[0],
                    ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input })
                    if id == "call_123"
                        && name == "lookup"
                        && input == &serde_json::json!({"query":"question"})
                )
        ));

        assert_eq!(gateway_request.messages[4].role, "user");
        assert!(matches!(
            gateway_request.messages[4].content,
            MessageContent::Blocks(ref blocks)
                if matches!(
                    &blocks[0],
                    ContentBlock::Known(KnownContentBlock::ToolResult { tool_use_id, content, .. })
                    if tool_use_id == "call_123" && matches!(content, ToolResultContent::Text(text) if text == "tool-output")
                )
        ));
    }

    #[test]
    fn invalid_function_call_call_id_is_rejected() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": [
                {
                    "type": "function_call",
                    "call_id": "",
                    "name": "lookup",
                    "arguments": "{}"
                }
            ]
        }))
        .unwrap();

        let err = transform_openai_to_gateway_request(request).unwrap_err();
        assert_eq!(err, "function_call.call_id must not be empty");
    }

    #[test]
    fn invalid_function_call_name_is_rejected() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": [
                {
                    "type": "function_call",
                    "call_id": "call_123",
                    "name": "",
                    "arguments": "{}"
                }
            ]
        }))
        .unwrap();

        let err = transform_openai_to_gateway_request(request).unwrap_err();
        assert_eq!(err, "function_call.name must not be empty");
    }

    #[test]
    fn invalid_function_call_arguments_are_rejected() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": [
                {
                    "type": "function_call",
                    "call_id": "call_123",
                    "name": "lookup",
                    "arguments": "not-json"
                }
            ]
        }))
        .unwrap();

        let err = transform_openai_to_gateway_request(request).unwrap_err();
        assert_eq!(err, "function_call.arguments must be a JSON string");
    }

    #[test]
    fn previous_response_id_allows_function_call_output_without_same_request_function_call() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "previous_response_id": "resp_prev_123",
            "input": [
                {
                    "type": "function_call_output",
                    "call_id": "call_123",
                    "output": "tool-output"
                }
            ]
        }))
        .unwrap();

        let gateway_request = transform_openai_to_gateway_request(request).unwrap();
        assert_eq!(gateway_request.messages.len(), 1);
        assert_eq!(gateway_request.messages[0].role, "user");
        assert!(matches!(
            gateway_request.messages[0].content,
            MessageContent::Blocks(ref blocks)
                if matches!(
                    &blocks[0],
                    ContentBlock::Known(KnownContentBlock::ToolResult { tool_use_id, content, .. })
                    if tool_use_id == "call_123"
                        && matches!(content, ToolResultContent::Text(text) if text == "tool-output")
                )
        ));
        assert_eq!(
            gateway_request.metadata.as_ref().and_then(
                |metadata| metadata.get(OPENAI_RESPONSES_PREVIOUS_RESPONSE_ID_METADATA_KEY)
            ),
            Some(&json!("resp_prev_123"))
        );
    }

    #[test]
    fn tool_choice_validation_filters_and_rejects_unknown_functions() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": "hello",
            "tools": [
                { "type": "function", "function": { "name": "alpha", "parameters": {"type":"object"} } },
                { "type": "function", "function": { "name": "beta", "parameters": {"type":"object"} } }
            ],
            "tool_choice": {"type":"function","function":{"name":"beta"}}
        }))
        .unwrap();

        let gateway_request = transform_openai_to_gateway_request(request).unwrap();
        assert_eq!(gateway_request.tools.as_ref().unwrap().len(), 1);
        assert_eq!(
            gateway_request.tools.as_ref().unwrap()[0].name.as_deref(),
            Some("beta")
        );

        let rejected: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": "hello",
            "tools": [
                { "type": "function", "function": { "name": "alpha", "parameters": {"type":"object"} } }
            ],
            "tool_choice": {"type":"function","function":{"name":"missing"}}
        }))
        .unwrap();
        assert!(transform_openai_to_gateway_request(rejected).is_err());
    }

    #[test]
    fn generic_public_responses_controls_remain_accepted_at_ingress() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": "hello",
            "max_output_tokens": 128,
            "temperature": 0.2,
            "top_p": 0.8,
            "stop": ["halt"],
            "stream_options": { "include_obfuscation": true },
            "tools": [
                { "type": "function", "function": { "name": "lookup", "parameters": {"type":"object"} } }
            ],
            "tool_choice": {"type":"function","function":{"name":"lookup"}}
        }))
        .unwrap();

        let gateway_request = transform_openai_to_gateway_request(request).unwrap();
        assert_eq!(gateway_request.max_output_tokens, 128);
        assert_eq!(gateway_request.temperature, Some(0.2));
        assert_eq!(gateway_request.top_p, Some(0.8));
        assert_eq!(
            gateway_request.stop_sequences,
            Some(vec!["halt".to_string()])
        );
        assert_eq!(gateway_request.tools.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn public_responses_passthrough_controls_survive_ingress_and_provider_serialization() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": "hello",
            "metadata": {
                "client": "sdk",
                "request_id": "req_123"
            },
            "truncation": "auto",
            "previous_response_id": "resp_prev_123",
            "user": "user_123",
            "stream_options": { "include_obfuscation": true },
            "service_tier": "priority"
        }))
        .unwrap();

        let gateway_request = transform_openai_to_gateway_request(request).unwrap();
        let provider = OpenAIProvider::with_transport(
            OpenAITransport::OpenAI,
            OpenAIProviderConfig {
                name: "test-openai".to_string(),
                api_key: String::new(),
                base_url: "https://example.invalid/v1".to_string(),
                models: Vec::new(),
                custom_headers: Vec::new(),
                oauth_provider: None,
                token_store: None,
                codex_auth_source: None,
            },
        );

        let serialized = serde_json::to_value(
            provider
                .transform_to_responses_request(&gateway_request)
                .unwrap(),
        )
        .unwrap();

        assert_eq!(
            serialized["metadata"],
            json!({
                "client": "sdk",
                "request_id": "req_123"
            })
        );
        assert_eq!(serialized["truncation"], json!("auto"));
        assert_eq!(serialized["previous_response_id"], json!("resp_prev_123"));
        assert_eq!(serialized["user"], json!("user_123"));
        assert_eq!(
            serialized["stream_options"],
            json!({ "include_obfuscation": true })
        );
        assert_eq!(serialized["service_tier"], json!("priority"));
    }

    #[test]
    fn rejects_builtin_tools_and_non_text_outputs() {
        let builtin_tool: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": "hello",
            "tools": [{ "type": "web_search", "function": { "name": "ignored" } }]
        }))
        .unwrap();
        assert!(transform_openai_to_gateway_request(builtin_tool).is_err());

        let unsupported_text: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": "hello",
            "text": { "format": { "type": "json_schema" } }
        }))
        .unwrap();
        assert!(transform_openai_to_gateway_request(unsupported_text).is_err());
    }

    #[test]
    fn accepts_non_text_system_parts_in_system_and_developer_messages() {
        let request: OpenAIResponsesRequest = serde_json::from_value(json!({
            "model": "gpt-test",
            "input": [
                {
                    "type": "message",
                    "role": "system",
                    "content": [
                        { "type": "input_image", "image_url": "https://example.com/image.png" }
                    ]
                },
                {
                    "type": "message",
                    "role": "developer",
                    "content": [
                        { "type": "input_text", "text": "dev" },
                        { "type": "input_image", "image_url": "https://example.com/dev.png", "detail": "high" }
                    ]
                }
            ]
        }))
        .unwrap();

        let gateway_request = transform_openai_to_gateway_request(request).unwrap();
        assert!(gateway_request.system.is_none());
        assert_eq!(gateway_request.messages.len(), 2);
        assert_eq!(gateway_request.messages[0].role, "system");
        match &gateway_request.messages[0].content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert!(matches!(
                    &blocks[0],
                    ContentBlock::Known(KnownContentBlock::Image { source })
                        if source.url.as_deref() == Some("https://example.com/image.png")
                ));
            }
            other => panic!("expected system blocks, got {:?}", other),
        }

        assert_eq!(gateway_request.messages[1].role, "developer");
        match &gateway_request.messages[1].content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(blocks[0].as_text(), Some("dev"));
                assert!(matches!(
                    &blocks[1],
                    ContentBlock::Known(KnownContentBlock::Image { source })
                        if source.url.as_deref() == Some("https://example.com/dev.png")
                ));
            }
            other => panic!("expected developer blocks, got {:?}", other),
        }
    }

    #[test]
    fn sync_response_mapping_preserves_model_usage_and_tool_call_order() {
        let gateway_response = response_with_tool_and_thinking("actual-model");
        let response = transform_gateway_response_to_openai_response(
            gateway_response,
            "public-model".to_string(),
        );

        assert_eq!(response.object_type, "response");
        assert_eq!(response.status, "completed");
        assert_eq!(response.model, "public-model");
        assert_eq!(response.usage.input_tokens, 9);
        assert_eq!(response.usage.output_tokens, 3);
        assert_eq!(response.usage.total_tokens, 12);
        assert_eq!(response.output.len(), 3);

        assert!(matches!(
            &response.output[0],
            OpenAIResponsesOutputItem::Message { content, .. }
                if content.len() == 1 && content[0].text == "before"
        ));
        assert!(matches!(
            &response.output[1],
            OpenAIResponsesOutputItem::FunctionCall { call_id, name, arguments }
                if call_id == "call_1" && name == "lookup" && arguments == "{\"query\":\"x\"}"
        ));
        assert!(matches!(
            &response.output[2],
            OpenAIResponsesOutputItem::Message { content, .. }
                if content.len() == 1 && content[0].text == "after"
        ));
    }

    #[derive(Debug, Deserialize)]
    struct ToolLoopRequestFixture {
        request: serde_json::Value,
        expected_tool_use_id: String,
        expected_tool_name: String,
        expected_arguments: serde_json::Value,
        expected_output: String,
    }

    #[derive(Debug, Deserialize)]
    struct SyncFixture {
        request_model: String,
        provider_response: GatewayResponse,
        expected: SyncFixtureExpected,
    }

    #[derive(Debug, Deserialize)]
    struct SyncFixtureExpected {
        status: String,
        texts: Vec<String>,
        call_ids: Vec<String>,
        usage: OpenAIResponsesUsage,
    }

    #[derive(Debug, Deserialize)]
    struct NegativeFixture {
        request: serde_json::Value,
        expected_error_contains: String,
    }

    #[derive(Debug, Deserialize)]
    struct StreamFixture {
        request: serde_json::Value,
        provider_stream_chunks: Vec<String>,
        required_events: Vec<String>,
        expected_fragments: Vec<String>,
        forbidden_fragments: Vec<String>,
    }

    #[tokio::test]
    async fn handler_respects_x_provider_and_echoes_the_public_model() {
        let primary = StubProvider::new(response_text("primary", "primary-actual"), vec![]);
        let secondary = StubProvider::new(response_text("secondary", "secondary-actual"), vec![]);
        let primary_requests = primary.captured_requests();
        let secondary_requests = secondary.captured_requests();

        let mut provider_registry = ProviderRegistry::new();
        provider_registry.insert_provider_for_tests("primary", Box::new(primary));
        provider_registry.insert_provider_for_tests("secondary", Box::new(secondary));

        let harness = ConformanceHarness::with_registry(
            "gateway-default",
            provider_registry,
            vec![
                ModelMapping {
                    priority: 1,
                    provider: "primary".to_string(),
                    actual_model: "primary-actual".to_string(),
                    inject_continuation_prompt: false,
                },
                ModelMapping {
                    priority: 2,
                    provider: "secondary".to_string(),
                    actual_model: "secondary-actual".to_string(),
                    inject_continuation_prompt: false,
                },
            ],
        );
        let state = harness.state();

        let mut headers = HeaderMap::new();
        headers.insert("x-provider", HeaderValue::from_static("secondary"));

        let request = json!({
            "model": "gateway-default",
            "input": "hello",
            "stream": false
        });

        let response = handle_openai_responses(State(state), headers, Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["model"], "gateway-default");
        assert_eq!(json["object"], "response");
        assert_eq!(json["status"], "completed");
        assert_eq!(json["output"][0]["content"][0]["text"], "secondary");

        assert!(primary_requests.lock().unwrap().is_empty());
        assert_eq!(secondary_requests.lock().unwrap().len(), 1);
        assert_eq!(
            secondary_requests.lock().unwrap()[0].model,
            "secondary-actual"
        );
    }

    #[tokio::test]
    async fn handler_streams_contracted_events_and_hides_provider_framing() {
        let provider = StubProvider::new(
            response_text("unused", "primary-actual"),
            vec![
                "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stream\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"primary-actual\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":0,\"output_tokens\":0}}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n".to_string(),
                "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n".to_string(),
                "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"call_1\",\"name\":\"lookup\",\"input\":{}}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\":\\\"README.md\\\"}\"}}\n\n".to_string(),
                "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":1}\n\n".to_string(),
                "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":10,\"output_tokens\":3}}\n\n".to_string(),
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
            ],
        );
        let captured_requests = provider.captured_requests();

        let mut provider_registry = ProviderRegistry::new();
        provider_registry.insert_provider_for_tests("primary", Box::new(provider));

        let harness = ConformanceHarness::with_registry(
            "gateway-default",
            provider_registry,
            vec![ModelMapping {
                priority: 1,
                provider: "primary".to_string(),
                actual_model: "primary-actual".to_string(),
                inject_continuation_prompt: false,
            }],
        );
        let state = harness.state();

        let request = json!({
            "model": "gateway-default",
            "input": "hello",
            "stream": true
        });

        let response = handle_openai_responses(State(state), HeaderMap::new(), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        for event_name in [
            "response.created",
            "response.output_item.added",
            "response.content_part.added",
            "response.output_text.delta",
            "response.output_text.done",
            "response.content_part.done",
            "response.function_call_arguments.delta",
            "response.function_call_arguments.done",
            "response.completed",
        ] {
            assert!(text.contains(&format!("event: {event_name}")));
            assert!(text.contains(&format!("\"type\":\"{event_name}\"")));
        }

        assert!(text.contains("\"call_id\":\"call_1\""));
        assert!(text.contains("\"model\":\"gateway-default\""));
        assert!(!text.contains("event: content_block_"));
        assert!(!text.contains("\"type\":\"message_delta\""));

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].model, "primary-actual");
        assert_eq!(requests[0].stream, Some(true));
    }

    #[test]
    fn fixture_tool_loop_request_preserves_call_id() {
        let fixture: ToolLoopRequestFixture = read_json_fixture(
            FixtureNamespace::OpenAiResponses,
            "request-tool-loop-function-call-output.json",
        );
        let request: OpenAIResponsesRequest = serde_json::from_value(fixture.request).unwrap();
        let gateway_request = transform_openai_to_gateway_request(request).unwrap();

        assert!(matches!(
            gateway_request.messages[1].content,
            MessageContent::Blocks(ref blocks)
                if matches!(
                    &blocks[0],
                    ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input })
                    if id == &fixture.expected_tool_use_id
                        && name == &fixture.expected_tool_name
                        && input == &fixture.expected_arguments
                )
        ));
        assert!(matches!(
            gateway_request.messages.last().unwrap().content,
            MessageContent::Blocks(ref blocks)
                if matches!(
                    &blocks[0],
                    ContentBlock::Known(KnownContentBlock::ToolResult { tool_use_id, content, .. })
                    if tool_use_id == &fixture.expected_tool_use_id
                        && matches!(content, ToolResultContent::Text(text) if text == &fixture.expected_output)
                )
        ));
    }

    #[test]
    fn sync_response_fixtures_cover_text_tool_and_mixed_cases() {
        for fixture_name in ["sync-text.json", "sync-tool-call.json", "sync-mixed.json"] {
            let fixture: SyncFixture =
                read_json_fixture(FixtureNamespace::OpenAiResponses, fixture_name);
            let response = transform_gateway_response_to_openai_response(
                fixture.provider_response,
                fixture.request_model.clone(),
            );

            let texts = response
                .output
                .iter()
                .filter_map(|item| match item {
                    OpenAIResponsesOutputItem::Message { content, .. } => Some(
                        content
                            .iter()
                            .map(|part| part.text.clone())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    ),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let call_ids = response
                .output
                .iter()
                .filter_map(|item| match item {
                    OpenAIResponsesOutputItem::FunctionCall { call_id, .. } => {
                        Some(call_id.clone())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            assert_eq!(response.status, fixture.expected.status, "{fixture_name}");
            assert_eq!(response.model, fixture.request_model, "{fixture_name}");
            assert_eq!(texts, fixture.expected.texts, "{fixture_name}");
            assert_eq!(call_ids, fixture.expected.call_ids, "{fixture_name}");
            assert_eq!(
                response.usage.input_tokens, fixture.expected.usage.input_tokens,
                "{fixture_name}"
            );
            assert_eq!(
                response.usage.output_tokens, fixture.expected.usage.output_tokens,
                "{fixture_name}"
            );
            assert_eq!(
                response.usage.total_tokens, fixture.expected.usage.total_tokens,
                "{fixture_name}"
            );
        }
    }

    #[test]
    fn negative_request_fixtures_reject_deterministically() {
        for fixture_name in [
            "negative-built-in-tool.json",
            "negative-unsupported-text-format.json",
        ] {
            let fixture: NegativeFixture =
                read_json_fixture(FixtureNamespace::OpenAiResponses, fixture_name);
            let request: OpenAIResponsesRequest = serde_json::from_value(fixture.request).unwrap();
            let error = transform_openai_to_gateway_request(request).unwrap_err();
            assert!(
                error.contains(&fixture.expected_error_contains),
                "{fixture_name}: {error}"
            );
        }
    }

    #[tokio::test]
    async fn stream_fixture_locks_required_event_surface() {
        let fixture: StreamFixture =
            read_json_fixture(FixtureNamespace::OpenAiResponses, "stream-mixed.json");
        let provider = StubProvider::new(
            response_text("unused", "primary-actual"),
            fixture.provider_stream_chunks.clone(),
        );
        let captured_requests = provider.captured_requests();

        let mut provider_registry = ProviderRegistry::new();
        provider_registry.insert_provider_for_tests("primary", Box::new(provider));

        let harness = ConformanceHarness::with_registry(
            "gateway-default",
            provider_registry,
            vec![ModelMapping {
                priority: 1,
                provider: "primary".to_string(),
                actual_model: "primary-actual".to_string(),
                inject_continuation_prompt: false,
            }],
        );
        let state = harness.state();

        let response = handle_openai_responses(
            State(state),
            HeaderMap::new(),
            Json(fixture.request.clone()),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        for event_name in fixture.required_events {
            assert!(
                text.contains(&format!("event: {event_name}")),
                "{event_name}"
            );
            assert!(
                text.contains(&format!("\"type\":\"{event_name}\"")),
                "{event_name}"
            );
        }

        for fragment in fixture.expected_fragments {
            assert!(text.contains(&fragment), "{fragment}");
        }

        for fragment in fixture.forbidden_fragments {
            assert!(!text.contains(&fragment), "{fragment}");
        }

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].model, "primary-actual");
        assert_eq!(requests[0].stream, Some(true));
    }
}
