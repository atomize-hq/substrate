use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::PathBuf,
    process::ExitStatus,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::{
    backend_harness::{BackendSpawn, DynBackendCompletionFuture, DynBackendEventStream},
    backends::spawn_path::resolve_effective_working_dir,
    AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperKind,
    AgentWrapperRunRequest,
};

use super::{
    mapping::{error_event, status_event},
    CodexApprovalPolicy, CodexBackendCompletion, CodexBackendConfig, CodexBackendError,
    CodexBackendEvent, CodexHandleFacetState, CodexSandboxMode, CodexTerminationHandle,
    PINNED_APPROVAL_REQUIRED, PINNED_TIMEOUT,
};

use super::super::{
    session_selectors::{parse_session_fork_v1, SessionSelectorV1, EXT_SESSION_FORK_V1},
    termination::{TerminationHandle, TerminationState},
};

#[derive(Clone, Debug)]
pub(super) struct ForkFlowRequest {
    pub(super) custody: codex::mcp::McpProcessCustody,
    pub(super) selector: SessionSelectorV1,
    pub(super) prompt: String,
    pub(super) working_dir: Option<PathBuf>,
    pub(super) effective_timeout: Option<Duration>,
    pub(super) env: BTreeMap<String, String>,
    pub(super) config: CodexBackendConfig,
    pub(super) run_start_cwd: Option<PathBuf>,
    pub(super) termination: Option<Arc<TerminationState<CodexTerminationHandle>>>,
    pub(super) non_interactive: bool,
    pub(super) external_sandbox: bool,
    pub(super) approval_policy: Option<CodexApprovalPolicy>,
    pub(super) sandbox_mode: CodexSandboxMode,
    pub(super) handle_state: Arc<std::sync::Mutex<CodexHandleFacetState>>,
}

pub(super) fn extract_fork_selector_v1(
    request: &AgentWrapperRunRequest,
) -> Result<Option<SessionSelectorV1>, AgentWrapperError> {
    request
        .extensions
        .get(EXT_SESSION_FORK_V1)
        .map(parse_session_fork_v1)
        .transpose()
}

#[derive(Clone)]
pub(super) struct AppServerTurnCancelHandle {
    server: Arc<codex::mcp::CodexAppServer>,
    request_id: codex::mcp::RequestId,
}

impl TerminationHandle for AppServerTurnCancelHandle {
    fn request_termination(&self) {
        let _ = self.server.cancel(self.request_id);
    }
}

fn synthetic_success_exit_status() -> ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(0)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(0)
    }
}

fn map_sandbox_mode(mode: &CodexSandboxMode) -> &'static str {
    match mode {
        CodexSandboxMode::ReadOnly => "read-only",
        CodexSandboxMode::WorkspaceWrite => "workspace-write",
        CodexSandboxMode::DangerFullAccess => "danger-full-access",
    }
}

fn map_approval_policy(policy: &CodexApprovalPolicy) -> &'static str {
    match policy {
        CodexApprovalPolicy::Untrusted => "untrusted",
        CodexApprovalPolicy::OnFailure => "on-failure",
        CodexApprovalPolicy::OnRequest => "on-request",
        CodexApprovalPolicy::Never => "never",
    }
}

fn effective_approval_policy(
    non_interactive: bool,
    approval_policy: Option<&CodexApprovalPolicy>,
) -> Option<String> {
    if non_interactive {
        return Some("never".to_string());
    }
    approval_policy.map(|policy| map_approval_policy(policy).to_string())
}

fn resolve_fork_effective_cwd(
    config: &CodexBackendConfig,
    run_start_cwd: Option<&PathBuf>,
    working_dir: Option<&PathBuf>,
) -> Result<PathBuf, CodexBackendError> {
    resolve_effective_working_dir(
        working_dir.map(PathBuf::as_path),
        config.default_working_dir.as_deref(),
        run_start_cwd.map(PathBuf::as_path),
    )
    .ok_or(CodexBackendError::WorkingDirectoryUnresolved)
}

fn install_thread_id(handle_state: &Arc<std::sync::Mutex<CodexHandleFacetState>>, thread_id: &str) {
    if thread_id.trim().is_empty() {
        return;
    }
    let Ok(mut state) = handle_state.lock() else {
        return;
    };
    if state.thread_id.is_none() {
        let len = thread_id.len();
        if len <= super::SESSION_HANDLE_ID_BOUND_BYTES {
            state.thread_id = Some(thread_id.to_string());
        } else if !state.oversize_warning_emitted {
            state.oversize_warning_emitted = true;
            state.oversize_warning_len_bytes = Some(len);
        }
    }
}

fn tools_facet_data(
    backend_item_id: Option<&str>,
    thread_id: Option<&str>,
    turn_id: Option<&str>,
    kind: &str,
    phase: &str,
    status: &str,
) -> Value {
    serde_json::json!({
        "schema": super::TOOLS_FACET_SCHEMA,
        "tool": {
            "backend_item_id": backend_item_id,
            "thread_id": thread_id,
            "turn_id": turn_id,
            "kind": kind,
            "phase": phase,
            "status": status,
            "exit_code": null,
            "bytes": { "stdout": 0, "stderr": 0, "diff": 0, "result": 0 },
            "tool_name": null,
            "tool_use_id": null
        }
    })
}

pub(super) fn map_app_server_notification(
    method: &str,
    params: &Value,
) -> Option<AgentWrapperEvent> {
    match method {
        "agentMessage/delta" => {
            let delta = params.as_str()?;
            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(delta.to_string()),
                message: None,
                data: None,
            })
        }
        "reasoning/text/delta" => {
            let text = params
                .get("content")
                .and_then(|content| content.get("text"))
                .and_then(Value::as_str)
                .or_else(|| params.get("text").and_then(Value::as_str))
                .or_else(|| params.as_str())?;
            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some(text.to_string()),
                message: None,
                data: None,
            })
        }
        "turn/started" | "turn/completed" => Some(status_event(None)),
        "item/started" => {
            let item_id = params.get("item_id").and_then(Value::as_str);
            let thread_id = params.get("thread_id").and_then(Value::as_str);
            let turn_id = params.get("turn_id").and_then(Value::as_str);
            let kind = params
                .get("item_type")
                .and_then(Value::as_str)
                .unwrap_or("unknown");

            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolCall,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    item_id, thread_id, turn_id, kind, "start", "running",
                )),
            })
        }
        "item/completed" => {
            let item_id = params.get("item_id").and_then(Value::as_str);
            let thread_id = params.get("thread_id").and_then(Value::as_str);
            let turn_id = params.get("turn_id").and_then(Value::as_str);
            let kind = params
                .get("item_type")
                .and_then(Value::as_str)
                .unwrap_or("unknown");

            Some(AgentWrapperEvent {
                agent_kind: AgentWrapperKind("codex".to_string()),
                kind: AgentWrapperEventKind::ToolResult,
                channel: Some("tool".to_string()),
                text: None,
                message: None,
                data: Some(tools_facet_data(
                    item_id,
                    thread_id,
                    turn_id,
                    kind,
                    "complete",
                    "completed",
                )),
            })
        }
        "error" => {
            let message = params
                .get("error")
                .and_then(|err| err.get("message"))
                .and_then(Value::as_str)
                .or_else(|| params.get("message").and_then(Value::as_str))
                .unwrap_or_default();
            if message.trim().is_empty() {
                return None;
            }
            Some(error_event(message.to_string()))
        }
        _ => None,
    }
}

pub(super) fn is_approval_request_notification(method: &str, params: &Value) -> bool {
    if method != "codex/event" {
        return false;
    }

    let payload = params.get("msg").unwrap_or(params);
    let payload = match payload.as_object() {
        Some(payload) => payload,
        None => return false,
    };

    let event_type = payload.get("type").and_then(Value::as_str);
    if !matches!(event_type, Some("approval_required" | "approval")) {
        return false;
    }

    let approval_id = payload
        .get("approval_id")
        .or_else(|| payload.get("id"))
        .and_then(Value::as_str);
    approval_id.is_some_and(|id| !id.trim().is_empty())
}

struct ForkStartupGuard(Option<codex::mcp::McpProcessCustody>);
impl Drop for ForkStartupGuard {
    fn drop(&mut self) {
        if let Some(custody) = self.0.take() {
            custody.request_termination();
        }
    }
}

fn with_cleanup_error(
    original: CodexBackendError,
    cleanup: Result<(), codex::mcp::McpError>,
) -> CodexBackendError {
    match cleanup {
        Ok(()) => original,
        Err(cleanup) => CodexBackendError::AppServerCleanup {
            original: Box::new(original),
            cleanup,
        },
    }
}

pub(super) async fn spawn_fork_v1_flow(
    req: ForkFlowRequest,
) -> Result<
    BackendSpawn<CodexBackendEvent, CodexBackendCompletion, CodexBackendError>,
    CodexBackendError,
> {
    let custody = req.custody.clone();
    let mut guard = ForkStartupGuard(Some(custody.clone()));
    let result = spawn_fork_v1_flow_inner(req).await;
    drop(guard.0.take());
    match result {
        Ok(spawned) => Ok(spawned),
        Err(original) => {
            custody.request_termination();
            let cleanup = custody
                .settle()
                .await
                .map_err(|error| codex::mcp::McpError::Transport(error.to_string()));
            Err(with_cleanup_error(original, cleanup))
        }
    }
}

async fn spawn_fork_v1_flow_inner(
    req: ForkFlowRequest,
) -> Result<
    BackendSpawn<CodexBackendEvent, CodexBackendCompletion, CodexBackendError>,
    CodexBackendError,
> {
    let ForkFlowRequest {
        custody,
        selector,
        prompt,
        working_dir,
        effective_timeout,
        env,
        config,
        run_start_cwd,
        termination,
        non_interactive,
        external_sandbox,
        approval_policy,
        sandbox_mode,
        handle_state,
    } = req;

    let timeout_for_turn: Option<Duration> = match effective_timeout {
        Some(timeout) if timeout == Duration::ZERO => None,
        other => other,
    };

    let effective_cwd =
        resolve_fork_effective_cwd(&config, run_start_cwd.as_ref(), working_dir.as_ref())?;

    let server = codex::mcp::CodexAppServer::with_capabilities_and_custody(
        codex::mcp::StdioServerConfig {
            binary: config
                .binary
                .clone()
                .unwrap_or_else(|| PathBuf::from("codex")),
            code_home: config.codex_home.clone(),
            current_dir: Some(effective_cwd.clone()),
            env: env
                .iter()
                .map(|(k, v)| (OsString::from(k), OsString::from(v)))
                .collect(),
            app_server_analytics_default_enabled: false,
            mirror_stdio: false,
            // Launcher queue and checked home admission precede native initialize.
            // Keep a finite receiver window; timeout_for_turn is unchanged.
            startup_timeout: Duration::from_secs(60),
        },
        codex::mcp::ClientInfo {
            name: "agent_api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        serde_json::json!({"experimentalApi": true}),
        custody,
    )
    .await
    .map_err(CodexBackendError::AppServer)?;
    let server = Arc::new(server);

    let source_thread_id = match selector {
        SessionSelectorV1::Id { id } => id,
        SessionSelectorV1::Last => server
            .select_last_thread_id(effective_cwd.clone())
            .await
            .map_err(CodexBackendError::AppServer)?
            .ok_or(CodexBackendError::ForkSelectionEmpty)?,
    };

    let (approval_policy, sandbox_mode) = if external_sandbox {
        (
            Some(CodexApprovalPolicy::Never),
            CodexSandboxMode::DangerFullAccess,
        )
    } else {
        (approval_policy, sandbox_mode)
    };

    let approval_policy = effective_approval_policy(non_interactive, approval_policy.as_ref());
    let sandbox = Some(map_sandbox_mode(&sandbox_mode).to_string());

    let forked = server
        .thread_fork(codex::mcp::ThreadForkParams {
            thread_id: source_thread_id.clone(),
            cwd: Some(effective_cwd.clone()),
            approval_policy: approval_policy.clone(),
            sandbox,
            persist_extended_history: Some(true),
        })
        .await;

    let forked = match forked {
        Ok(forked) => forked,
        Err(codex::mcp::McpError::Rpc { message, .. }) if super::is_not_found_signal(&message) => {
            return Err(CodexBackendError::ForkSessionNotFound);
        }
        Err(err) => return Err(CodexBackendError::AppServer(err)),
    };

    if forked.thread.id.trim().is_empty()
        || forked.thread.id == source_thread_id
        || forked.thread.id.len() > super::SESSION_HANDLE_ID_BOUND_BYTES
    {
        return Err(CodexBackendError::AppServer(codex::mcp::McpError::Server(
            "thread/fork returned no distinct bounded child identity".into(),
        )));
    }
    install_thread_id(&handle_state, forked.thread.id.as_str());

    let turn = server
        .turn_start_v2(codex::mcp::TurnStartParamsV2 {
            thread_id: forked.thread.id.clone(),
            input: vec![codex::mcp::UserInputV2::text(prompt)],
            approval_policy,
            cwd: Some(effective_cwd),
        })
        .await
        .map_err(CodexBackendError::AppServer)?;

    if let Some(state) = termination.as_ref() {
        state.set_handle(CodexTerminationHandle::AppServerTurn(
            AppServerTurnCancelHandle {
                server: Arc::clone(&server),
                request_id: turn.request_id,
            },
        ));
    }

    let approval_required = Arc::new(AtomicBool::new(false));
    let stop_forwarding = Arc::new(AtomicBool::new(false));
    let (approval_signal_tx, mut approval_signal_rx) = oneshot::channel::<()>();

    let (event_tx, event_rx) = mpsc::unbounded_channel::<CodexBackendEvent>();
    let (completion_tx, completion_rx) =
        oneshot::channel::<Result<CodexBackendCompletion, CodexBackendError>>();

    let codex::mcp::AppCallHandle {
        request_id: turn_request_id,
        events: mut turn_events,
        response: turn_response,
    } = turn;

    let event_tx_for_notifications = event_tx.clone();
    let approval_required_for_notifications = Arc::clone(&approval_required);
    let stop_forwarding_for_notifications = Arc::clone(&stop_forwarding);

    tokio::spawn(async move {
        let mut approval_signal_tx = Some(approval_signal_tx);
        while let Some(notification) = turn_events.recv().await {
            if let codex::mcp::AppNotification::Raw { method, params } = &notification {
                if is_approval_request_notification(method, params)
                    && !approval_required_for_notifications.swap(true, Ordering::SeqCst)
                {
                    if let Some(tx) = approval_signal_tx.take() {
                        let _ = tx.send(());
                    }
                    // Stop forwarding so the `"approval required"` tail event is terminal.
                    return;
                }
            }

            if stop_forwarding_for_notifications.load(Ordering::SeqCst) {
                return;
            }

            let _ = event_tx_for_notifications
                .send(CodexBackendEvent::AppServerNotification(notification));
        }
    });

    tokio::spawn({
        let server = Arc::clone(&server);
        let approval_required = Arc::clone(&approval_required);
        let stop_forwarding = Arc::clone(&stop_forwarding);
        async move {
            let mut cancel_sent = false;
            tokio::pin!(turn_response);

            match timeout_for_turn {
                None => loop {
                    tokio::select! {
                        biased;
                        _ = &mut approval_signal_rx, if !cancel_sent => {
                            cancel_sent = true;
                            if approval_required.load(Ordering::SeqCst) {
                                let _ = server.cancel(turn_request_id);
                            }
                        }
                        response_outcome = &mut turn_response => {
                            let response_outcome = match response_outcome {
                                Ok(outcome) => outcome,
                                Err(_) => Err(codex::mcp::McpError::ChannelClosed),
                            };

                            let mut selection_failure_message = None;
                            if approval_required.load(Ordering::SeqCst) {
                                stop_forwarding.store(true, Ordering::SeqCst);
                                selection_failure_message = Some(PINNED_APPROVAL_REQUIRED.to_string());
                                let _ = event_tx.send(CodexBackendEvent::TerminalError {
                                    message: PINNED_APPROVAL_REQUIRED.to_string(),
                                });
                            } else if let Err(err) = response_outcome {
                                let cleanup = server.shutdown().await;
                                let _ = completion_tx.send(Err(with_cleanup_error(CodexBackendError::AppServer(err), cleanup)));
                                return;
                            }

                            if let Err(error) = server.shutdown().await {
                                let _ = completion_tx.send(Err(CodexBackendError::AppServer(error)));
                                return;
                            }
                            let _ = completion_tx.send(Ok(CodexBackendCompletion {
                                status: synthetic_success_exit_status(),
                                final_text: None,
                                backend_error_message: None,
                                selection_failure_message,
                            }));
                            return;
                        }
                    }
                },
                Some(timeout) => {
                    let deadline = tokio::time::sleep(timeout);
                    tokio::pin!(deadline);

                    loop {
                        tokio::select! {
                            biased;
                            _ = &mut approval_signal_rx, if !cancel_sent => {
                                cancel_sent = true;
                                if approval_required.load(Ordering::SeqCst) {
                                    let _ = server.cancel(turn_request_id);
                                }
                            }
                            _ = &mut deadline => {
                                stop_forwarding.store(true, Ordering::SeqCst);
                                let _ = event_tx.send(CodexBackendEvent::TerminalError {
                                    message: PINNED_TIMEOUT.to_string(),
                                });
                                let _ = server.cancel(turn_request_id);
                                let cleanup = server.shutdown().await;
                                let _ = completion_tx.send(Err(with_cleanup_error(CodexBackendError::Timeout { timeout }, cleanup)));
                                return;
                            }
                            response_outcome = &mut turn_response => {
                                let response_outcome = match response_outcome {
                                    Ok(outcome) => outcome,
                                    Err(_) => Err(codex::mcp::McpError::ChannelClosed),
                                };

                                let mut selection_failure_message = None;
                                if approval_required.load(Ordering::SeqCst) {
                                    selection_failure_message = Some(PINNED_APPROVAL_REQUIRED.to_string());
                                    let _ = event_tx.send(CodexBackendEvent::TerminalError {
                                        message: PINNED_APPROVAL_REQUIRED.to_string(),
                                    });
                                } else if let Err(err) = response_outcome {
                                    let cleanup = server.shutdown().await;
                                    let _ = completion_tx.send(Err(with_cleanup_error(CodexBackendError::AppServer(err), cleanup)));
                                    return;
                                }

                                if let Err(error) = server.shutdown().await {
                                    let _ = completion_tx.send(Err(CodexBackendError::AppServer(error)));
                                    return;
                                }
                                let _ = completion_tx.send(Ok(CodexBackendCompletion {
                                    status: synthetic_success_exit_status(),
                                    final_text: None,
                                    backend_error_message: None,
                                    selection_failure_message,
                                }));
                                return;
                            }
                        }
                    }
                }
            }
        }
    });

    let events: DynBackendEventStream<CodexBackendEvent, CodexBackendError> = Box::pin(
        futures_util::stream::unfold(event_rx, |mut rx| async move {
            rx.recv().await.map(|event| (Ok(event), rx))
        }),
    );

    let completion: DynBackendCompletionFuture<CodexBackendCompletion, CodexBackendError> =
        Box::pin(async move {
            completion_rx
                .await
                .unwrap_or(Err(CodexBackendError::CompletionTaskDropped))
        });

    Ok(BackendSpawn {
        events,
        completion,
        events_observability: None,
    })
}

#[cfg(all(test, unix))]
mod custody_tests {
    use crate::backends::codex::{CodexBackend, CodexBackendConfig};
    use crate::{AgentWrapperBackend, AgentWrapperRunRequest};
    use std::{fs, os::unix::fs::PermissionsExt, time::Duration};

    // MCP lifecycle fixture only. This makes no native-history/production-Fork claim.
    #[tokio::test]
    async fn test_fork_public_control_waits_for_real_app_server_custody_on_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("mcp-lifecycle-fixture");
        let entered = dir.path().join("turn-entered");
        fs::write(
            &binary,
            r#"#!/usr/bin/env python3
import json,os,sys
assert sys.argv[1:] == ['app-server']
for line in sys.stdin:
 m=json.loads(line); method=m.get('method')
 if method=='initialize': result={'ready':True}
 elif method=='thread/fork': result={'thread':{'id':'component-child'}}
 elif method=='turn/start':
  open(os.environ['MCP_TURN_MARKER'],'w').close()
  continue
 elif method=='shutdown': result={}
 else: continue
 print(json.dumps({'jsonrpc':'2.0','id':m['id'],'result':result}),flush=True)
"#,
        )
        .unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let backend = CodexBackend::new(CodexBackendConfig {
            binary: Some(binary),
            ..Default::default()
        });
        let control = backend
            .run_control(AgentWrapperRunRequest {
                prompt: "component lifecycle fixture".into(),
                env: [("MCP_TURN_MARKER".into(), entered.display().to_string())]
                    .into_iter()
                    .collect(),
                extensions: [(
                    "agent_api.session.fork.v1".into(),
                    serde_json::json!({"selector":"id","id":"component-source"}),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            })
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            while !entered.exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("actual SDK-spawned app-server reached turn/start");
        let settlement = control
            .cancel
            .settlement()
            .expect("real MCP owner is exposed");
        tokio::pin!(settlement);
        assert!(
            tokio::time::timeout(Duration::from_millis(30), &mut settlement)
                .await
                .is_err()
        );
        control.cancel.cancel();
        drop(control.handle.events);
        tokio::time::timeout(Duration::from_secs(3), &mut settlement)
            .await
            .unwrap()
            .unwrap();
        let raw = tokio::time::timeout(Duration::from_secs(3), control.handle.completion)
            .await
            .unwrap();
        assert!(
            matches!(raw, Err(crate::AgentWrapperError::Backend { message }) if message == "cancelled"),
            "checked MCP cleanup must preserve the exact cancellation result"
        );
    }
    // Real SDK child/stdio regression only; no native-history or authority claim.
    #[tokio::test]
    async fn test_initial_fork_initialize_budget_and_pending_startup_cancel() {
        use futures_util::StreamExt;
        for cancel_during_initialize in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let binary = dir.path().join("held-initialize-fork");
            let entered = dir.path().join("initialize-entered");
            let release = dir.path().join("release-initialize");
            fs::write(&binary, r#"#!/usr/bin/env python3
import json,os,sys,time
assert sys.argv[1:] == ['app-server']
for line in sys.stdin:
 m=json.loads(line); method=m.get('method'); params=m.get('params',{})
 if method=='initialize':
  open(os.environ['FORK_INITIALIZE_MARKER'],'w').close()
  while not os.path.exists(os.environ['FORK_INITIALIZE_RELEASE']): time.sleep(0.005)
  result={'ready':True}
 elif method=='thread/fork':
  assert params['threadId']=='component-source'
  result={'thread':{'id':'component-child'}}
 elif method=='turn/start':
  assert params['threadId']=='component-child'
  print(json.dumps({'jsonrpc':'2.0','method':'turn/completed',
   'params':{'threadId':'component-child','turn':{'id':'component-turn','status':'completed'}}}),flush=True)
  result={'turn':{'id':'component-turn','status':'completed'}}
 elif method=='shutdown': result={}
 elif method=='exit': break
 else: continue
 print(json.dumps({'jsonrpc':'2.0','id':m['id'],'result':result}),flush=True)
"#).unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
            let backend = CodexBackend::new(CodexBackendConfig {
                binary: Some(binary),
                ..Default::default()
            });
            let control = backend
                .run_control(AgentWrapperRunRequest {
                    prompt: "held initialize protocol fixture".into(),
                    working_dir: Some(dir.path().to_path_buf()),
                    env: [
                        (
                            "FORK_INITIALIZE_MARKER".into(),
                            entered.display().to_string(),
                        ),
                        (
                            "FORK_INITIALIZE_RELEASE".into(),
                            release.display().to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    extensions: [(
                        "agent_api.session.fork.v1".into(),
                        serde_json::json!({"selector":"id","id":"component-source"}),
                    )]
                    .into_iter()
                    .collect(),
                    ..Default::default()
                })
                .await
                .unwrap();
            let cancel = control.cancel;
            let settlement = cancel.settlement().expect("actual Fork MCP custody");
            tokio::pin!(settlement);
            let mut events = control.handle.events;
            let drain = tokio::spawn(async move { while events.next().await.is_some() {} });
            let completion = tokio::spawn(control.handle.completion);
            tokio::time::timeout(Duration::from_secs(3), async {
                while !entered.is_file() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("actual child received initialize");
            assert!(
                tokio::time::timeout(Duration::from_millis(20), &mut settlement)
                    .await
                    .is_err(),
                "held initialize cannot acknowledge child/pipe settlement"
            );
            if cancel_during_initialize {
                cancel.cancel();
            } else {
                // Beyond the former 30-second window, after actual request receipt.
                tokio::time::sleep(Duration::from_secs(31)).await;
                fs::write(&release, b"release").unwrap();
            }
            let result = tokio::time::timeout(Duration::from_secs(5), completion)
                .await
                .unwrap()
                .unwrap();
            tokio::time::timeout(Duration::from_secs(3), &mut settlement)
                .await
                .unwrap()
                .unwrap();
            tokio::time::timeout(Duration::from_secs(3), drain)
                .await
                .unwrap()
                .unwrap();
            if cancel_during_initialize {
                assert!(
                    matches!(result, Err(crate::AgentWrapperError::Backend { message })
                    if message == "cancelled"),
                    "held-startup cancellation remains exact and checked"
                );
            } else {
                let result = result.expect("bounded startup accepts a delayed initialize reply");
                assert_eq!(result.data.unwrap()["session"]["id"], "component-child");
            }
        }
    }

    #[test]
    fn test_cleanup_error_retains_original_backend_variant() {
        let result = super::with_cleanup_error(
            super::CodexBackendError::Timeout {
                timeout: Duration::from_secs(2),
            },
            Err(codex::mcp::McpError::Transport(
                "fixture settlement failure".into(),
            )),
        );
        match result {
            super::CodexBackendError::AppServerCleanup { original, cleanup } => {
                assert!(
                    matches!(*original, super::CodexBackendError::Timeout { timeout } if timeout == Duration::from_secs(2))
                );
                assert!(
                    matches!(cleanup, codex::mcp::McpError::Transport(message) if message == "fixture settlement failure")
                );
            }
            _ => panic!("cleanup replaced the original raw error"),
        }
    }
}

#[cfg(all(test, unix))]
mod initial_fork_identity_tests {
    use crate::backends::codex::{CodexBackend, CodexBackendConfig};
    use crate::{AgentWrapperBackend, AgentWrapperRunRequest};
    use futures_util::StreamExt;
    use std::{fs, os::unix::fs::PermissionsExt, time::Duration};

    // Actual SDK app-server process/RPC ownership, using a protocol fixture.
    // This does not claim Codex native history availability or real provider work.
    #[tokio::test]
    async fn test_initial_fork_inherits_selection_and_uses_only_distinct_returned_child() {
        for child in ["component-child", "component-source", "", "   "] {
            let dir = tempfile::tempdir().unwrap();
            let binary = dir.path().join("initial-fork-protocol-fixture");
            let entered = dir.path().join("turn-entered");
            let shutdown = dir.path().join("shutdown-entered");
            let release = dir.path().join("release-shutdown");
            fs::write(
                &binary,
                r#"#!/usr/bin/env python3
import json,os,sys,time
assert sys.argv[1:] == ['app-server']
for line in sys.stdin:
 m=json.loads(line); method=m.get('method'); params=m.get('params',{})
 if method=='initialize': result={'ready':True}
 elif method=='thread/fork':
  assert params['threadId']=='component-source'
  assert params['persistExtendedHistory'] is True
  assert 'model' not in params and 'addDirs' not in params
  assert params['cwd']==os.environ['FORK_CWD']
  result={'thread':{'id':os.environ['FORK_CHILD']}}
 elif method=='turn/start':
  assert os.environ['FORK_CHILD']=='component-child'
  assert params['threadId']=='component-child'
  assert 'model' not in params and 'addDirs' not in params
  assert params['cwd']==os.environ['FORK_CWD']
  open(os.environ['FORK_TURN_MARKER'],'w').close()
  print(json.dumps({'jsonrpc':'2.0','method':'turn/completed',
   'params':{'threadId':'component-child','turn':{'id':'component-turn','status':'completed'}}}),flush=True)
  result={'turn':{'id':'component-turn','status':'completed'}}
 elif method=='shutdown':
  open(os.environ['FORK_SHUTDOWN_MARKER'],'w').close()
  deadline=time.monotonic()+2
  while not os.path.exists(os.environ['FORK_RELEASE_SHUTDOWN']):
   assert time.monotonic()<deadline, 'test owner did not release protocol shutdown'
   time.sleep(0.005)
  result={}
 else: continue
 print(json.dumps({'jsonrpc':'2.0','id':m['id'],'result':result}),flush=True)
"#,
            )
            .unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
            let backend = CodexBackend::new(CodexBackendConfig {
                binary: Some(binary),
                ..Default::default()
            });
            let control = backend
                .run_control(AgentWrapperRunRequest {
                    prompt: "component protocol proof".into(),
                    working_dir: Some(dir.path().to_path_buf()),
                    env: [
                        ("FORK_CHILD".into(), child.into()),
                        ("FORK_CWD".into(), dir.path().display().to_string()),
                        ("FORK_TURN_MARKER".into(), entered.display().to_string()),
                        (
                            "FORK_SHUTDOWN_MARKER".into(),
                            shutdown.display().to_string(),
                        ),
                        (
                            "FORK_RELEASE_SHUTDOWN".into(),
                            release.display().to_string(),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    extensions: [(
                        "agent_api.session.fork.v1".into(),
                        serde_json::json!({"selector":"id","id":"component-source"}),
                    )]
                    .into_iter()
                    .collect(),
                    ..Default::default()
                })
                .await;
            let control = control.unwrap();
            let settlement = control.cancel.settlement().expect("actual MCP settlement");
            tokio::pin!(settlement);
            let mut events = control.handle.events;
            let reader = tokio::spawn(async move {
                let mut observed = Vec::new();
                while let Some(event) = events.next().await {
                    observed.push(event);
                }
                observed
            });
            let completion = control.handle.completion;
            tokio::pin!(completion);
            if child == "component-child" {
                tokio::time::timeout(Duration::from_secs(3), async {
                    while !shutdown.is_file() {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                assert!(
                    tokio::time::timeout(Duration::from_millis(20), &mut completion)
                        .await
                        .is_err()
                );
                assert!(tokio::time::timeout(Duration::from_millis(20), &mut settlement).await.is_err(),
                    "turn completion notification/response does not acknowledge live app-server custody");
                fs::write(&release, b"release").unwrap();
            }
            let result = tokio::time::timeout(Duration::from_secs(3), &mut completion)
                .await
                .unwrap();
            let observed = reader.await.unwrap();
            settlement.await.unwrap();
            if child == "component-child" {
                let result = result.unwrap();
                assert!(entered.is_file());
                assert!(
                    observed.iter().any(|event| event
                        .data
                        .as_ref()
                        .is_some_and(|data| data["session"]["id"] == "component-child")),
                    "actual turn/completed notification must traverse the UAA event path"
                );
                assert_eq!(result.data.unwrap()["session"]["id"], "component-child");
            } else {
                assert!(
                    result.is_err(),
                    "invalid native child identity must fail startup"
                );
                assert!(
                    !entered.exists(),
                    "invalid child must never reach turn/start"
                );
            }
        }
    }
}
