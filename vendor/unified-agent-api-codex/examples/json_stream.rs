//! Streams Codex JSONL output in real time and reports where the last message is saved.
//! Usage:
//! ```powershell
//! cargo run -p unified-agent-api-codex --example json_stream -- --output-last-message ./last_message.txt --log-events ./events.log -- "Summarize repo status"
//! ```

use codex::{CodexClient, ExecStreamRequest, ItemDeltaPayload, ItemPayload, ThreadEvent};
use futures_util::StreamExt;
use std::{env, error::Error, path::PathBuf, time::Duration};

type ParsedArgs = (String, Option<PathBuf>, Option<PathBuf>);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (prompt, output_last_message, json_event_log) = parse_args()?;
    let client = CodexClient::builder()
        .json(true)
        .quiet(true)
        .mirror_stdout(false)
        .build();

    let mut stream = client
        .stream_exec(ExecStreamRequest {
            prompt,
            ephemeral: false,
            ignore_rules: false,
            ignore_user_config: false,
            idle_timeout: Some(Duration::from_secs(30)),
            output_last_message,
            output_schema: None,
            json_event_log,
        })
        .await?;

    println!("Streaming Codex events...");
    while let Some(event) = stream.events.next().await {
        match event {
            Ok(event) => println!("→ {}", summarize_event(&event)),
            Err(err) => {
                eprintln!("stream error: {err}");
                break;
            }
        }
    }

    let completion = stream.completion.await?;
    if let Some(path) = completion.last_message_path.as_ref() {
        println!("Last message written to {}", path.display());
    }
    if let Some(last_message) = completion.last_message.as_ref() {
        println!("\n--- Last message ---\n{last_message}");
    }

    Ok(())
}

fn parse_args() -> Result<ParsedArgs, Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();
    if matches!(args.peek().map(|s| s.as_str()), Some("--")) {
        args.next();
    }

    let mut output_last_message = None;
    let mut json_event_log = None;
    let mut prompt_parts = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "--output-last-message" {
            let path = args
                .next()
                .ok_or("Provide a path after --output-last-message")?;
            output_last_message = Some(PathBuf::from(path));
        } else if arg == "--log-events" {
            let path = args.next().ok_or("Provide a path after --log-events")?;
            json_event_log = Some(PathBuf::from(path));
        } else {
            prompt_parts.push(arg);
        }
    }

    if prompt_parts.is_empty() {
        return Err("Provide a prompt".into());
    }

    Ok((prompt_parts.join(" "), output_last_message, json_event_log))
}

fn summarize_event(event: &ThreadEvent) -> String {
    match event {
        ThreadEvent::ThreadStarted(event) => format!("thread.started {}", event.thread_id),
        ThreadEvent::TurnStarted(event) => format!("turn.started {}", event.turn_id),
        ThreadEvent::TurnCompleted(event) => format!(
            "turn.completed last_item={}",
            event.last_item_id.as_deref().unwrap_or("-")
        ),
        ThreadEvent::TurnFailed(event) => format!("turn.failed {}", event.error.message),
        ThreadEvent::ItemStarted(item) => format!(
            "item.started {} [{}]",
            item.item.item_id,
            item_payload_label(&item.item.payload)
        ),
        ThreadEvent::ItemDelta(delta) => format!(
            "item.delta {} [{}]",
            delta.item_id,
            item_delta_label(&delta.delta)
        ),
        ThreadEvent::ItemCompleted(item) => format!(
            "item.completed {} [{}]",
            item.item.item_id,
            item_payload_label(&item.item.payload)
        ),
        ThreadEvent::ItemFailed(item) => {
            format!(
                "item.failed {} ({})",
                item.item.item_id, item.item.error.message
            )
        }
        ThreadEvent::Error(err) => format!("stream error: {}", err.message),
    }
}

fn item_payload_label(payload: &ItemPayload) -> &'static str {
    match payload {
        ItemPayload::AgentMessage(_) => "agent_message",
        ItemPayload::Reasoning(_) => "reasoning",
        ItemPayload::CommandExecution(_) => "command_execution",
        ItemPayload::FileChange(_) => "file_change",
        ItemPayload::McpToolCall(_) => "mcp_tool_call",
        ItemPayload::WebSearch(_) => "web_search",
        ItemPayload::TodoList(_) => "todo_list",
        ItemPayload::Error(_) => "error",
    }
}

fn item_delta_label(delta: &ItemDeltaPayload) -> &'static str {
    match delta {
        ItemDeltaPayload::AgentMessage(_) => "agent_message",
        ItemDeltaPayload::Reasoning(_) => "reasoning",
        ItemDeltaPayload::CommandExecution(_) => "command_execution",
        ItemDeltaPayload::FileChange(_) => "file_change",
        ItemDeltaPayload::McpToolCall(_) => "mcp_tool_call",
        ItemDeltaPayload::WebSearch(_) => "web_search",
        ItemDeltaPayload::TodoList(_) => "todo_list",
        ItemDeltaPayload::Error(_) => "error",
    }
}
