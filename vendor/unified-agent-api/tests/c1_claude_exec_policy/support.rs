use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
    pin::Pin,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub(super) use agent_api::{
    backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest,
};
pub(super) use claude_code::ClaudeHomeLayout;
pub(super) use futures_core::Stream;
pub(super) use serde_json::json;

pub(super) const PINNED_EXTERNAL_SANDBOX_WARNING: &str =
    "DANGEROUS: external sandbox exec policy enabled (agent_api.exec.external_sandbox.v1=true)";

pub(super) fn fake_claude_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_claude_stream_json_agent_api"))
}

pub(super) async fn drain_to_none(
    mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>,
    timeout: Duration,
) -> Vec<AgentWrapperEvent> {
    let mut out = Vec::new();
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => break,
            item = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)) => {
                match item {
                    Some(ev) => out.push(ev),
                    None => break,
                }
            }
        }
    }

    out
}

pub(super) fn unique_temp_log_path(test_name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{test_name}_{pid}_{nanos}.log"))
}

pub(super) fn unique_missing_dir_path(test_name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{test_name}_{pid}_{nanos}_missing"))
}

pub(super) fn read_invocations(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).expect("read FAKE_CLAUDE_INVOCATION_LOG");
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

pub(super) fn read_env_snapshot(path: &Path) -> BTreeMap<String, String> {
    let text = std::fs::read_to_string(path).expect("read FAKE_CLAUDE_ENV_SNAPSHOT_PATH");
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        out.insert(key.to_string(), value.to_string());
    }
    out
}

pub(super) fn warning_indices(events: &[AgentWrapperEvent]) -> Vec<usize> {
    events
        .iter()
        .enumerate()
        .filter(|(_, ev)| ev.kind == AgentWrapperEventKind::Status)
        .filter(|(_, ev)| ev.channel.as_deref() == Some("status"))
        .filter(|(_, ev)| ev.message.as_deref() == Some(PINNED_EXTERNAL_SANDBOX_WARNING))
        .filter(|(_, ev)| ev.data.is_none())
        .map(|(idx, _)| idx)
        .collect()
}

pub(super) fn session_handle_facet_index(events: &[AgentWrapperEvent]) -> Option<usize> {
    events
        .iter()
        .enumerate()
        .find(|(_, ev)| {
            ev.kind == AgentWrapperEventKind::Status
                && ev
                    .data
                    .as_ref()
                    .and_then(|data| data.get("schema"))
                    .and_then(serde_json::Value::as_str)
                    == Some("agent_api.session.handle.v1")
        })
        .map(|(idx, _)| idx)
}

pub(super) fn first_user_visible_index(events: &[AgentWrapperEvent]) -> Option<usize> {
    events
        .iter()
        .enumerate()
        .find(|(_, ev)| {
            matches!(
                ev.kind,
                AgentWrapperEventKind::TextOutput
                    | AgentWrapperEventKind::ToolCall
                    | AgentWrapperEventKind::ToolResult
            )
        })
        .map(|(idx, _)| idx)
}

pub(super) fn first_error_index(events: &[AgentWrapperEvent]) -> Option<usize> {
    events
        .iter()
        .enumerate()
        .find(|(_, ev)| ev.kind == AgentWrapperEventKind::Error)
        .map(|(idx, _)| idx)
}

pub(super) fn count(lines: &[String], needle: &str) -> usize {
    lines.iter().filter(|line| line.as_str() == needle).count()
}

pub(super) fn first_index(lines: &[String], needle: &str) -> Option<usize> {
    lines.iter().position(|line| line.as_str() == needle)
}

pub(super) struct EnvGuard {
    pub(super) key: &'static str,
    pub(super) previous: Option<OsString>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.previous.take() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}
