//! Example for `codex app-server`: start or resume a thread, stream items and
//! task_complete notifications, and optionally interrupt the active turn.
//! Usage:
//! `cargo run -p unified-agent-api-codex --example app_server_turns -- "<prompt>" [thread-id]`
//! Gate with `crates/codex/examples/feature_detection.rs` if your binary might not expose app-server endpoints.
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): CODEX_HOME to pass through.
//! - `INTERRUPT_AFTER_MS` (optional): delay before sending `turn/interrupt`
//!   after the first item arrives.

use std::{collections::BTreeMap, env, path::PathBuf, time::Duration};

use codex::mcp::{
    AppNotification, ClientInfo, CodexAppServer, McpError, StdioServerConfig, ThreadResumeParams,
    ThreadStartParams, TurnInput, TurnInterruptParams, TurnStartParams,
};
use serde_json::Value;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let prompt = args
        .next()
        .expect("usage: app_server_turns <prompt> [thread-id]");
    let resume_thread = args.next();

    let config = config_from_env();
    let client = ClientInfo {
        name: "codex-app-example".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    };
    let server = CodexAppServer::start(config, client)
        .await
        .map_err(boxed_err)?;

    let thread_id = if let Some(existing) = resume_thread {
        let handle = server
            .thread_resume(ThreadResumeParams {
                thread_id: existing.clone(),
            })
            .await
            .map_err(boxed_err)?;
        let response = match handle.response.await {
            Ok(resp) => resp,
            Err(err) => return Err(boxed_err(err)),
        }?;
        let resolved_id = response
            .get("thread_id")
            .or_else(|| response.get("threadId"))
            .or_else(|| response.get("thread").and_then(|t| t.get("id")))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!(
            "resumed thread {} (resumed={})",
            resolved_id,
            response
                .get("resumed")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
        existing
    } else {
        let handle = server
            .thread_start(ThreadStartParams {
                thread_id: None,
                metadata: Value::Null,
            })
            .await
            .map_err(boxed_err)?;
        let response = match handle.response.await {
            Ok(resp) => resp,
            Err(err) => return Err(boxed_err(err)),
        }?;
        let id = response
            .get("thread_id")
            .or_else(|| response.get("threadId"))
            .or_else(|| response.get("thread").and_then(|t| t.get("id")))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        println!("started thread {id}");
        id
    };

    let mut turn = server
        .turn_start(TurnStartParams {
            thread_id: thread_id.clone(),
            input: vec![TurnInput {
                kind: "text".into(),
                text: Some(prompt),
            }],
            model: None,
            config: BTreeMap::new(),
        })
        .await
        .map_err(boxed_err)?;

    let interrupt_after_ms = env::var("INTERRUPT_AFTER_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok());

    let mut turn_id_for_interrupt: Option<String> = None;
    while let Some(event) = turn.events.recv().await {
        match &event {
            AppNotification::Item {
                thread_id: tid,
                turn_id,
                item,
            } => {
                println!("[turn] item {tid}/{turn_id:?}: {item}");
                if let Some(turn_id) = turn_id {
                    if turn_id_for_interrupt.is_none() {
                        turn_id_for_interrupt = Some(turn_id.clone());
                        if let Some(delay) = interrupt_after_ms {
                            time::sleep(Duration::from_millis(delay)).await;
                            let interrupt = server
                                .turn_interrupt(TurnInterruptParams {
                                    thread_id: Some(thread_id.clone()),
                                    turn_id: turn_id.clone(),
                                })
                                .await
                                .map_err(boxed_err)?;
                            let interrupt_response = match interrupt.response.await {
                                Ok(resp) => resp,
                                Err(err) => return Err(boxed_err(err)),
                            };
                            let _ = interrupt_response?;
                        }
                    }
                }
            }
            AppNotification::TaskComplete {
                thread_id: tid,
                turn_id,
                result,
            } => {
                println!("[turn] complete {tid}/{turn_id:?}: {result}");
                break;
            }
            AppNotification::Error { message, data } => {
                println!("[turn] error {message} {data:?}");
                break;
            }
            AppNotification::Raw { method, params } => {
                println!("[turn] raw {method}: {params}");
            }
        }
    }

    let turn_response = match turn.response.await {
        Ok(resp) => resp,
        Err(err) => return Err(boxed_err(err)),
    };
    match turn_response {
        Ok(value) => println!("turn response: {value}"),
        Err(McpError::Cancelled) => println!("turn was cancelled"),
        Err(other) => return Err(boxed_err(other)),
    }

    let _ = server.shutdown().await;
    Ok(())
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

fn boxed_err<E: std::error::Error + 'static>(err: E) -> Box<dyn std::error::Error> {
    Box::new(err)
}
