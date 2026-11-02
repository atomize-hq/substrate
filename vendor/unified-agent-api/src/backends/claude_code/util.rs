use std::{
    future::Future,
    path::{Path, PathBuf},
};

use claude_code::{ClaudeOutputFormat, ClaudePrintRequest};
use futures_util::stream;
use tokio::sync::OnceCell;

use crate::{
    backend_harness::{BackendSpawn, DynBackendEventStream},
    backends::spawn_path::resolve_effective_working_dir,
    AgentWrapperError,
};

use super::super::session_selectors::SessionSelectorV1;
use super::{
    harness::{ClaudeBackendCompletion, ClaudeBackendError, ClaudeBackendEvent},
    ClaudeCodeBackendConfig,
};

pub(super) const ADD_DIRS_RUNTIME_REJECTION_MESSAGE: &str = "add_dirs rejected by runtime";
pub(super) const PINNED_MODEL_RUNTIME_REJECTION: &str =
    "claude_code backend error: model rejected by runtime (details redacted)";
pub(super) const PINNED_WORKING_DIR_RESOLUTION_FAILURE: &str =
    "claude backend failed to resolve working directory";

pub(super) fn parse_bool(
    value: &serde_json::Value,
    key: &str,
) -> Result<bool, crate::AgentWrapperError> {
    value
        .as_bool()
        .ok_or_else(|| crate::AgentWrapperError::InvalidRequest {
            message: format!("{key} must be a boolean"),
        })
}

fn is_session_not_found_signal(text: &str) -> bool {
    let text = text.to_ascii_lowercase();

    (text.contains("not found")
        && (text.contains("session") || text.contains("thread") || text.contains("conversation")))
        || text.contains("no session")
        || text.contains("unknown session")
        || text.contains("no thread")
        || text.contains("unknown thread")
        || text.contains("no conversation")
        || text.contains("unknown conversation")
}

pub(super) fn json_contains_not_found_signal(raw: &serde_json::Value) -> bool {
    const MAX_DEPTH: usize = 6;
    const MAX_STRING_LEAVES: usize = 64;

    fn visit(raw: &serde_json::Value, depth: usize, strings_seen: &mut usize) -> bool {
        if depth > MAX_DEPTH || *strings_seen >= MAX_STRING_LEAVES {
            return false;
        }

        match raw {
            serde_json::Value::String(s) => {
                *strings_seen += 1;
                is_session_not_found_signal(s)
            }
            serde_json::Value::Array(arr) => arr
                .iter()
                .any(|child| visit(child, depth + 1, strings_seen)),
            serde_json::Value::Object(obj) => obj
                .values()
                .any(|child| visit(child, depth + 1, strings_seen)),
            _ => false,
        }
    }

    let mut strings_seen = 0usize;
    visit(raw, 0, &mut strings_seen)
}

pub(super) fn json_contains_add_dirs_runtime_rejection_signal(raw: &serde_json::Value) -> bool {
    raw.get("message").and_then(serde_json::Value::as_str)
        == Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
}

pub(super) fn json_contains_model_runtime_rejection_signal(raw: &serde_json::Value) -> bool {
    raw.get("message").and_then(serde_json::Value::as_str) == Some("model rejected by runtime")
}

pub(super) fn generic_non_zero_exit_message(status: &std::process::ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("claude_code exited non-zero: code={code} (output redacted)"),
        None => "claude_code exited non-zero (output redacted)".to_string(),
    }
}

pub(super) fn resolve_completion_messages(
    status: &std::process::ExitStatus,
    selection_selector: Option<&SessionSelectorV1>,
    saw_stream_error: bool,
    saw_not_found_signal: bool,
    runtime_backend_error_message: Option<String>,
) -> (Option<String>, Option<String>) {
    if saw_stream_error {
        return (None, None);
    }

    let backend_error_message =
        if selection_selector.is_some() && !status.success() && saw_not_found_signal {
            match selection_selector {
                Some(SessionSelectorV1::Last) => Some("no session found".to_string()),
                Some(SessionSelectorV1::Id { .. }) => Some("session not found".to_string()),
                None => None,
            }
        } else {
            runtime_backend_error_message
        };

    let terminal_error_event_message = backend_error_message.clone().or_else(|| {
        if !status.success() {
            selection_selector
                .as_ref()
                .map(|_| generic_non_zero_exit_message(status))
        } else {
            None
        }
    });

    (backend_error_message, terminal_error_event_message)
}

pub(super) fn render_backend_error_message(err: &ClaudeBackendError) -> String {
    match err {
        ClaudeBackendError::Spawn(err) | ClaudeBackendError::Completion(err) => {
            format!("claude_code error: {err}")
        }
        ClaudeBackendError::ExternalSandboxPreflight { message } => message.clone(),
        ClaudeBackendError::StreamParse(err) => err.message.clone(),
    }
}

pub(super) fn startup_failure_spawn(
    err: ClaudeBackendError,
    emit_external_sandbox_warning: bool,
) -> BackendSpawn<ClaudeBackendEvent, ClaudeBackendCompletion, ClaudeBackendError> {
    let message = render_backend_error_message(&err);
    let events: DynBackendEventStream<ClaudeBackendEvent, ClaudeBackendError> =
        if emit_external_sandbox_warning {
            Box::pin(stream::iter(vec![
                Ok(ClaudeBackendEvent::ExternalSandboxWarning),
                Ok(ClaudeBackendEvent::TerminalError { message }),
            ]))
        } else {
            Box::pin(stream::once(async move {
                Ok(ClaudeBackendEvent::TerminalError { message })
            }))
        };
    let completion = Box::pin(async move { Err(err) });
    BackendSpawn {
        events,
        completion,
        events_observability: None,
    }
}

pub(super) fn resolve_claude_effective_working_dir(
    config: &ClaudeCodeBackendConfig,
    run_start_cwd: Option<&Path>,
    request_working_dir: Option<&Path>,
) -> Result<Option<PathBuf>, AgentWrapperError> {
    let selected_working_dir = request_working_dir.or(config.default_working_dir.as_deref());
    let resolved_working_dir = resolve_effective_working_dir(
        request_working_dir,
        config.default_working_dir.as_deref(),
        run_start_cwd,
    );

    if selected_working_dir.is_some_and(Path::is_relative) && resolved_working_dir.is_none() {
        return Err(AgentWrapperError::Backend {
            message: PINNED_WORKING_DIR_RESOLUTION_FAILURE.to_string(),
        });
    }

    Ok(resolved_working_dir)
}

pub(super) fn build_fresh_run_print_request(
    prompt: String,
    non_interactive: bool,
    external_sandbox: bool,
    allow_dangerously_skip_permissions: bool,
    add_dirs: &[PathBuf],
) -> ClaudePrintRequest {
    let mut print_req = ClaudePrintRequest::new(prompt)
        .output_format(ClaudeOutputFormat::StreamJson)
        .include_partial_messages(true);
    if non_interactive {
        print_req = print_req.permission_mode("bypassPermissions");
    }
    if external_sandbox {
        print_req = print_req.dangerously_skip_permissions(true);
        if allow_dangerously_skip_permissions {
            print_req = print_req.allow_dangerously_skip_permissions(true);
        }
    }
    if !add_dirs.is_empty() {
        print_req = print_req.add_dirs(
            add_dirs
                .iter()
                .map(|dir| dir.as_os_str().to_string_lossy().into_owned()),
        );
    }

    print_req
}

fn help_supports_allow_flag(stdout: &str) -> bool {
    stdout.contains("--allow-dangerously-skip-permissions")
}

pub(super) async fn preflight_allow_flag_support<F, Fut>(
    cell: &OnceCell<bool>,
    help: F,
) -> Result<bool, String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<claude_code::CommandOutput, claude_code::ClaudeCodeError>>,
{
    let supported = cell
        .get_or_try_init(move || async move {
            let output = match help().await {
                Ok(output) => output,
                Err(err) => return Err(format!("claude --help preflight failed: {err}")),
            };

            if !output.status.success() {
                let message = match output.status.code() {
                    Some(code) => {
                        format!("claude --help exited non-zero: code={code} (output redacted)")
                    }
                    None => "claude --help exited non-zero (output redacted)".to_string(),
                };
                return Err(message);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(help_supports_allow_flag(&stdout))
        })
        .await?;

    Ok(*supported)
}
