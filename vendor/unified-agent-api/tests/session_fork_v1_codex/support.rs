use std::{fs, path::PathBuf, pin::Pin, time::Duration};

use agent_api::AgentWrapperEvent;
use futures_core::Stream;
use serde_json::{json, Value};
use tempfile::NamedTempFile;

pub(super) fn fake_codex_app_server_binary() -> PathBuf {
    PathBuf::from(env!(
        "CARGO_BIN_EXE_fake_codex_app_server_jsonrpc_agent_api"
    ))
}

pub(super) fn add_dirs_payload(dirs: &[impl AsRef<str>]) -> Value {
    json!({
        "dirs": dirs.iter().map(|dir| dir.as_ref()).collect::<Vec<_>>()
    })
}

pub(super) fn request_log_file() -> NamedTempFile {
    NamedTempFile::new().expect("create request log")
}

pub(super) fn read_logged_request_methods(log: &NamedTempFile) -> Vec<String> {
    let raw = fs::read_to_string(log.path()).expect("read request log");
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
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

pub(super) fn any_event_contains(events: &[AgentWrapperEvent], needle: &str) -> bool {
    events.iter().any(|ev| {
        ev.message
            .as_deref()
            .is_some_and(|message| message.contains(needle))
            || ev.text.as_deref().is_some_and(|text| text.contains(needle))
            || ev
                .data
                .as_ref()
                .and_then(|data| serde_json::to_string(data).ok())
                .is_some_and(|data| data.contains(needle))
    })
}
