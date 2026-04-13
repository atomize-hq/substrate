use crate::core::GatewayResponse;
use crate::models::{ContentBlock, KnownContentBlock};
use crate::providers::streaming::{parse_sse_events, SseEvent};
use bytes::Bytes;
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizedEventKind {
    ToolIntent,
    Action,
    Final,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOrigin {
    ToolRequest,
    AssistantProgress,
    AssistantFinal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedEvent {
    pub event_kind: NormalizedEventKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_arguments: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_text: Option<String>,
    pub source_origin: SourceOrigin,
}

impl NormalizedEvent {
    pub fn action(text: String) -> Self {
        Self {
            event_kind: NormalizedEventKind::Action,
            tool_name: None,
            tool_id: None,
            tool_arguments: None,
            summary_text: Some(text),
            final_text: None,
            source_origin: SourceOrigin::AssistantProgress,
        }
    }

    pub fn tool_intent(id: String, name: String, input: Option<Value>) -> Self {
        Self {
            event_kind: NormalizedEventKind::ToolIntent,
            tool_name: Some(name),
            tool_id: Some(id),
            tool_arguments: input,
            summary_text: None,
            final_text: None,
            source_origin: SourceOrigin::ToolRequest,
        }
    }

    pub fn final_text(text: String) -> Self {
        Self {
            event_kind: NormalizedEventKind::Final,
            tool_name: None,
            tool_id: None,
            tool_arguments: None,
            summary_text: None,
            final_text: Some(text),
            source_origin: SourceOrigin::AssistantFinal,
        }
    }
}

pub fn normalized_events_from_provider_response(
    response: &GatewayResponse,
) -> Vec<NormalizedEvent> {
    let mut events: Vec<NormalizedEvent> = Vec::new();

    let mut visible_text: Vec<String> = Vec::new();

    let flush_as_action = |events: &mut Vec<NormalizedEvent>, visible_text: &mut Vec<String>| {
        let text = visible_text.join("");
        visible_text.clear();
        if !text.trim().is_empty() {
            events.push(NormalizedEvent::action(text));
        }
    };

    let flush_as_final = |events: &mut Vec<NormalizedEvent>, visible_text: &mut Vec<String>| {
        let text = visible_text.join("");
        visible_text.clear();
        if !text.trim().is_empty() {
            events.push(NormalizedEvent::final_text(text));
        }
    };

    for block in &response.content {
        match block {
            ContentBlock::Known(KnownContentBlock::Text { text, .. }) => {
                visible_text.push(text.clone());
            }
            ContentBlock::Known(KnownContentBlock::ToolUse { id, name, input }) => {
                flush_as_action(&mut events, &mut visible_text);
                events.push(NormalizedEvent::tool_intent(
                    id.clone(),
                    name.clone(),
                    Some(input.clone()),
                ));
            }
            _ => {}
        }
    }

    // End-of-response handling: only emit final when the response is terminal.
    let stop_reason = response.stop_reason.as_deref().unwrap_or("");
    if stop_reason == "end_turn" {
        flush_as_final(&mut events, &mut visible_text);
    }

    events
}

#[derive(Debug, Clone)]
struct StreamToolBlock {
    id: String,
    name: String,
    partial_json: String,
}

#[derive(Default)]
pub struct AnthropicSseNormalizedEventExtractor {
    sse_buffer: String,
    visible_text: String,
    tool_blocks: HashMap<usize, StreamToolBlock>,
}

impl AnthropicSseNormalizedEventExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_bytes(&mut self, bytes: &Bytes) -> Vec<NormalizedEvent> {
        let Ok(text) = std::str::from_utf8(bytes.as_ref()) else {
            return Vec::new();
        };

        self.sse_buffer.push_str(text);
        self.drain_completed_events(false)
    }

    pub fn finalize(&mut self) -> Vec<NormalizedEvent> {
        self.drain_completed_events(true)
    }

    fn drain_completed_events(&mut self, flush_all: bool) -> Vec<NormalizedEvent> {
        let mut out: Vec<NormalizedEvent> = Vec::new();

        let parse_upto = if flush_all {
            if self.sse_buffer.is_empty() {
                None
            } else {
                Some(self.sse_buffer.len())
            }
        } else {
            self.sse_buffer.rfind("\n\n").map(|idx| idx + 2)
        };

        let Some(end) = parse_upto else {
            return out;
        };

        let complete_portion = self.sse_buffer[..end].to_string();
        if !flush_all {
            self.sse_buffer = self.sse_buffer[end..].to_string();
        } else {
            self.sse_buffer.clear();
        }

        let events: Vec<SseEvent> = parse_sse_events(&complete_portion);
        for evt in events {
            self.consume_sse_event(&evt, &mut out);
        }

        out
    }

    fn flush_visible_text_as_action(&mut self, out: &mut Vec<NormalizedEvent>) {
        if self.visible_text.trim().is_empty() {
            self.visible_text.clear();
            return;
        }
        let text = std::mem::take(&mut self.visible_text);
        out.push(NormalizedEvent::action(text));
    }

    fn flush_visible_text_as_final(&mut self, out: &mut Vec<NormalizedEvent>) {
        if self.visible_text.trim().is_empty() {
            self.visible_text.clear();
            return;
        }
        let text = std::mem::take(&mut self.visible_text);
        out.push(NormalizedEvent::final_text(text));
    }

    fn consume_sse_event(&mut self, evt: &SseEvent, out: &mut Vec<NormalizedEvent>) {
        let Some(name) = evt.event.as_deref() else {
            return;
        };

        match name {
            "content_block_start" => {
                let Ok(json) = serde_json::from_str::<Value>(&evt.data) else {
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

                // Any visible text accumulated before tool_use is an "action" event.
                self.flush_visible_text_as_action(out);

                self.tool_blocks.insert(
                    index as usize,
                    StreamToolBlock {
                        id: id.to_string(),
                        name: tool_name.to_string(),
                        partial_json: String::new(),
                    },
                );
            }
            "content_block_delta" => {
                let Ok(json) = serde_json::from_str::<Value>(&evt.data) else {
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
                            self.visible_text.push_str(text);
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
                        if let Some(block) = self.tool_blocks.get_mut(&(index as usize)) {
                            block.partial_json.push_str(partial);
                        }
                    }
                    _ => {}
                }
            }
            "content_block_stop" => {
                let Ok(json) = serde_json::from_str::<Value>(&evt.data) else {
                    return;
                };
                let Some(index) = json.get("index").and_then(|v| v.as_u64()) else {
                    return;
                };
                let Some(block) = self.tool_blocks.remove(&(index as usize)) else {
                    return;
                };

                let input = if block.partial_json.trim().is_empty() {
                    None
                } else {
                    serde_json::from_str::<Value>(&block.partial_json).ok()
                };

                out.push(NormalizedEvent::tool_intent(block.id, block.name, input));
            }
            "message_delta" => {
                let Ok(json) = serde_json::from_str::<Value>(&evt.data) else {
                    return;
                };
                let stop_reason = json
                    .get("delta")
                    .and_then(|d| d.get("stop_reason"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if stop_reason.is_empty() {
                    return;
                }

                if stop_reason != "tool_use" {
                    self.flush_visible_text_as_final(out);
                }
            }
            _ => {}
        }
    }
}

#[pin_project]
pub struct NormalizedEventSseStream<S> {
    #[pin]
    inner: S,
    extractor: AnthropicSseNormalizedEventExtractor,
    queue: VecDeque<Bytes>,
}

impl<S> NormalizedEventSseStream<S> {
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            extractor: AnthropicSseNormalizedEventExtractor::new(),
            queue: VecDeque::new(),
        }
    }

    fn enqueue_events(queue: &mut VecDeque<Bytes>, events: Vec<NormalizedEvent>) {
        for event in events {
            let Some(bytes) = normalized_event_to_sse_bytes(&event) else {
                continue;
            };
            queue.push_back(bytes);
        }
    }
}

fn normalized_event_to_sse_bytes(event: &NormalizedEvent) -> Option<Bytes> {
    let event_name = match event.event_kind {
        NormalizedEventKind::ToolIntent => "tool_intent",
        NormalizedEventKind::Action => "action",
        NormalizedEventKind::Final => "final",
    };

    let data = serde_json::to_string(event).ok()?;
    Some(Bytes::from(format!(
        "event: {}\ndata: {}\n\n",
        event_name, data
    )))
}

impl<S, E> futures::stream::Stream for NormalizedEventSseStream<S>
where
    S: futures::stream::Stream<Item = Result<Bytes, E>>,
{
    type Item = Result<Bytes, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if let Some(item) = this.queue.pop_front() {
            return Poll::Ready(Some(Ok(item)));
        }

        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let events = this.extractor.push_bytes(&bytes);
                if !events.is_empty() {
                    Self::enqueue_events(this.queue, events);
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
                let events = this.extractor.finalize();
                if !events.is_empty() {
                    Self::enqueue_events(this.queue, events);
                }

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

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::core::{GatewayResponse, GatewayUsage};
    use serde::Deserialize;

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
        tool_arguments: Option<Value>,
        #[serde(default)]
        source_origin: Option<String>,
    }

    fn normalized_from_fixture(event: &FixtureEvent) -> NormalizedEvent {
        match event.event_kind.as_str() {
            "action" => NormalizedEvent::action(
                event
                    .summary_text
                    .clone()
                    .expect("fixture action summary_text"),
            ),
            "final" => NormalizedEvent::final_text(
                event.final_text.clone().expect("fixture final final_text"),
            ),
            "tool_intent" => NormalizedEvent::tool_intent(
                event.tool_id.clone().expect("fixture tool_intent tool_id"),
                event
                    .tool_name
                    .clone()
                    .expect("fixture tool_intent tool_name"),
                event.tool_arguments.clone(),
            ),
            other => panic!("unknown fixture event_kind: {}", other),
        }
    }

    fn provider_response_from_events(events: &[NormalizedEvent]) -> GatewayResponse {
        let mut blocks: Vec<ContentBlock> = Vec::new();
        let mut saw_tool_intent = false;
        let mut saw_final = false;

        for event in events {
            match event.event_kind {
                NormalizedEventKind::Action => {
                    blocks.push(ContentBlock::text(
                        event.summary_text.clone().unwrap_or_default(),
                        None,
                    ));
                }
                NormalizedEventKind::ToolIntent => {
                    saw_tool_intent = true;
                    blocks.push(ContentBlock::tool_use(
                        event
                            .tool_id
                            .clone()
                            .unwrap_or_else(|| "tool_0".to_string()),
                        event
                            .tool_name
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                        event
                            .tool_arguments
                            .clone()
                            .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
                    ));
                }
                NormalizedEventKind::Final => {
                    saw_final = true;
                    blocks.push(ContentBlock::text(
                        event.final_text.clone().unwrap_or_default(),
                        None,
                    ));
                }
            }
        }

        let stop_reason = if saw_final {
            "end_turn"
        } else if saw_tool_intent {
            "tool_use"
        } else {
            "end_turn"
        };

        GatewayResponse {
            id: "msg_fixture".to_string(),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: blocks,
            model: "fixture-model".to_string(),
            stop_reason: Some(stop_reason.to_string()),
            stop_sequence: None,
            usage: GatewayUsage {
                input_tokens: 0,
                output_tokens: 0,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        }
    }

    #[test]
    fn normalized_events_from_provider_response_matches_fixture_shape() {
        let cases = [
            "explicit-tool-calls-k2-thinking-stream",
            "hidden-markers-k2-thinking-nonstream",
            "hidden-markers-k2-thinking-stream",
            "mixed-reasoning-and-tool-calls-k2-thinking",
            "no-tool-control-k2-5-stream",
        ];

        for case in cases {
            let path = format!(
                "{}/tests/fixtures/azure_kimi/{}.json",
                env!("CARGO_MANIFEST_DIR"),
                case
            );
            let raw = std::fs::read_to_string(&path).expect("read fixture");
            let fixture: FixtureFile = serde_json::from_str(&raw).expect("parse fixture");

            let expected: Vec<NormalizedEvent> = fixture
                .normalized_events
                .iter()
                .map(normalized_from_fixture)
                .collect();

            let response = provider_response_from_events(&expected);
            let actual = normalized_events_from_provider_response(&response);

            assert_eq!(actual, expected, "fixture mismatch for {}", case);
        }
    }

    #[test]
    fn extractor_emits_action_then_tool_intent_for_tool_use_stream() {
        let sse = concat!(
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Let me check.\"}}\n\n",
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tool_1\",\"name\":\"Read\",\"input\":{}}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"file_path\\\":\\\"/tmp\\\"}\"}}\n\n",
            "event: content_block_stop\n",
            "data: {\"type\":\"content_block_stop\",\"index\":1}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null}}\n\n",
        );

        let mut extractor = AnthropicSseNormalizedEventExtractor::new();
        let events = extractor.push_bytes(&Bytes::from(sse));

        assert_eq!(
            events,
            vec![
                NormalizedEvent::action("Let me check.".to_string()),
                NormalizedEvent::tool_intent(
                    "tool_1".to_string(),
                    "Read".to_string(),
                    Some(serde_json::json!({ "file_path": "/tmp" })),
                ),
            ]
        );
    }

    #[test]
    fn extractor_emits_final_on_end_turn() {
        let sse = concat!(
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"All done.\"}}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null}}\n\n",
        );

        let mut extractor = AnthropicSseNormalizedEventExtractor::new();
        let events = extractor.push_bytes(&Bytes::from(sse));

        assert_eq!(
            events,
            vec![NormalizedEvent::final_text("All done.".to_string())]
        );
    }
}
