//! Minimal MCP example: start `codex mcp-server`, stream `codex/event` updates,
//! optionally cancel, and send a follow-up `codex/codex-reply`.
//! Usage:
//! `cargo run -p unified-agent-api-codex --example mcp_codex_flow -- "<prompt>" ["<follow up prompt>"]`
//! Gate with `crates/codex/examples/feature_detection.rs` if your binary might not expose MCP endpoints.
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): CODEX_HOME to pass through.
//! - `CANCEL_AFTER_MS` (optional): delay before sending `$ /cancelRequest`.

use std::{collections::BTreeMap, env, path::PathBuf, time::Duration};

use codex::mcp::{
    ClientInfo, CodexCallParams, CodexEvent, CodexMcpServer, CodexReplyParams, EventStream,
    McpError, StdioServerConfig,
};
use tokio::time;

#[derive(Default, Clone, Debug)]
struct EventSummary {
    conversation_id: Option<String>,
    last_task_complete: Option<serde_json::Value>,
    last_raw_event: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let use_sample = take_flag(&mut args, "--sample");
    let mut args = args.into_iter();
    let prompt = args
        .next()
        .unwrap_or_else(|| "Sample MCP codex prompt".to_string());
    let follow_up = args.next();

    if use_sample {
        replay_sample(&prompt, follow_up.as_deref());
        return Ok(());
    }

    let config = config_from_env();
    let client = ClientInfo {
        name: "codex-mcp-example".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    };
    let server = CodexMcpServer::start(config, client)
        .await
        .map_err(boxed_err)?;

    let cancel_after_ms = env::var("CANCEL_AFTER_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());

    let mut handle = server
        .codex(CodexCallParams {
            prompt: prompt.clone(),
            model: None,
            cwd: None,
            sandbox: None,
            approval_policy: None,
            profile: None,
            config: BTreeMap::new(),
        })
        .await
        .map_err(boxed_err)?;

    if let Some(delay) = cancel_after_ms {
        time::sleep(Duration::from_millis(delay)).await;
        let _ = server.cancel(handle.request_id);
    }

    let summary = stream_codex_events("codex/codex", &mut handle.events).await;
    let first_response = match handle.response.await {
        Ok(resp) => resp,
        Err(err) => return Err(boxed_err(err)),
    };
    let mut response_conversation = None;
    match first_response {
        Ok(resp) => {
            response_conversation = resp.conversation_id.clone();
            println!(
                "codex response (full): {}",
                serde_json::to_string_pretty(&resp).unwrap_or_else(|_| resp.output.to_string())
            );
            if let Some(task) = summary.last_task_complete.as_ref() {
                println!(
                    "last task_complete payload: {}",
                    serde_json::to_string_pretty(task).unwrap_or_else(|_| task.to_string())
                );
            }
            if let Some(raw) = summary.last_raw_event.as_ref() {
                println!(
                    "last raw codex/event: {}",
                    serde_json::to_string_pretty(raw).unwrap_or_else(|_| raw.to_string())
                );
            }

            let conv = resp
                .conversation_id
                .or_else(|| summary.conversation_id.clone());
            match conv {
                Some(conv) => {
                    println!("conversation: {conv}");
                    println!("(use this conversationId with mcp_codex_reply)");
                }
                None => println!("conversation: <missing>"),
            }
        }
        Err(McpError::Cancelled) => eprintln!("codex call cancelled"),
        Err(other) => return Err(boxed_err(other)),
    }

    let conversation_id = summary.conversation_id.or(response_conversation);
    if let (Some(follow_up_prompt), Some(conversation_id)) = (follow_up, conversation_id) {
        let mut follow_up = server
            .codex_reply(CodexReplyParams {
                conversation_id: conversation_id.clone(),
                prompt: follow_up_prompt,
            })
            .await
            .map_err(boxed_err)?;

        let _ = stream_codex_events("codex/codex-reply", &mut follow_up.events).await;
        let follow_up_response = match follow_up.response.await {
            Ok(resp) => resp,
            Err(err) => return Err(boxed_err(err)),
        };
        match follow_up_response {
            Ok(resp) => {
                println!(
                    "codex-reply response (full): {}",
                    serde_json::to_string_pretty(&resp).unwrap_or_else(|_| resp.output.to_string())
                );
                let conv = resp
                    .conversation_id
                    .unwrap_or_else(|| conversation_id.clone());
                println!("codex-reply {} => {}", conv, resp.output);
            }
            Err(err) => eprintln!("codex-reply failed: {err}"),
        }
    }

    let _ = server.shutdown().await;
    Ok(())
}

fn boxed_err<E: std::error::Error + 'static>(err: E) -> Box<dyn std::error::Error> {
    Box::new(err)
}

fn config_from_env() -> StdioServerConfig {
    let binary = env::var_os("CODEX_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"));
    let code_home = env::var_os("CODEX_HOME").map(PathBuf::from);

    StdioServerConfig {
        binary,
        code_home,
        current_dir: None,
        env: Vec::new(),
        app_server_analytics_default_enabled: false,
        mirror_stdio: true,
        startup_timeout: Duration::from_secs(10),
    }
}

async fn stream_codex_events(label: &str, events: &mut EventStream<CodexEvent>) -> EventSummary {
    let mut summary = EventSummary::default();
    while let Some(event) = events.recv().await {
        match &event {
            CodexEvent::TaskComplete {
                conversation_id: conv,
                result,
            } => {
                println!(
                    "[{label}] task_complete {conv}: {}",
                    serde_json::to_string_pretty(result).unwrap_or_else(|_| result.to_string())
                );
                if !conv.is_empty() {
                    summary.conversation_id = Some(conv.clone());
                }
                summary.last_task_complete = Some(result.clone());
                break;
            }
            CodexEvent::ApprovalRequired(req) => {
                println!("[{label}] approval {:?}: {:?}", req.approval_id, req.kind);
            }
            CodexEvent::Cancelled {
                conversation_id: conv,
                reason,
            } => {
                println!("[{label}] cancelled {:?}: {:?}", conv, reason);
                if let Some(conv) = conv {
                    summary.conversation_id = Some(conv.clone());
                }
                break;
            }
            CodexEvent::Error { message, data } => {
                println!("[{label}] error {message} {data:?}");
            }
            CodexEvent::Raw { method, params } => {
                println!("[{label}] raw {method}: {params}");
                summary.last_raw_event = Some(params.clone());
                if summary.conversation_id.is_none() {
                    if let Some(msg) = params.get("msg") {
                        let candidate = msg
                            .get("session_id")
                            .or_else(|| msg.get("thread_id"))
                            .or_else(|| msg.get("conversation_id"))
                            .or_else(|| msg.get("conversationId"))
                            .and_then(|value| value.as_str())
                            .map(|conv| conv.to_string());
                        summary.conversation_id = candidate.or(summary.conversation_id.clone());
                    }
                }
            }
        }
    }

    summary
}

fn replay_sample(prompt: &str, follow_up: Option<&str>) {
    println!("[sample] codex prompt: {prompt}");
    println!("[sample] task_complete conv=sample-conv: streamed sample events");
    if let Some(follow) = follow_up {
        println!("[sample] codex-reply {follow} => sample follow-up response");
    }
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|arg| arg != flag);
    before != args.len()
}
