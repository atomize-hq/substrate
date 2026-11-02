use claude_code::{ClaudeStreamJsonEvent, ClaudeStreamJsonParser};
use futures_core::Stream;
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs as std_fs,
    path::{Path, PathBuf},
    pin::Pin,
    process::Command,
    sync::OnceLock,
    time::Duration,
};

pub(super) use super::super::harness::ClaudeBackendEvent;
pub(super) use super::super::*;
pub(super) use crate::{
    backend_harness::BackendHarnessAdapter,
    backends::test_support::{test_env_lock, CurrentDirGuard},
    mcp::{AgentWrapperMcpAddRequest, AgentWrapperMcpAddTransport, AgentWrapperMcpRemoveRequest},
    mcp::{
        CAPABILITY_MCP_ADD_V1, CAPABILITY_MCP_GET_V1, CAPABILITY_MCP_LIST_V1,
        CAPABILITY_MCP_REMOVE_V1,
    },
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest,
};
pub(super) use serde_json::Value as JsonValue;

pub(super) const SYSTEM_INIT: &str =
    include_str!("../../../../../claude_code/tests/fixtures/stream_json/v1/system_init.jsonl");
pub(super) const SYSTEM_OTHER: &str =
    include_str!("../../../../../claude_code/tests/fixtures/stream_json/v1/system_other.jsonl");
pub(super) const RESULT_ERROR: &str =
    include_str!("../../../../../claude_code/tests/fixtures/stream_json/v1/result_error.jsonl");
pub(super) const ASSISTANT_MESSAGE_TEXT: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/assistant_message_text.jsonl"
);
pub(super) const ASSISTANT_MESSAGE_TOOL_USE: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/assistant_message_tool_use.jsonl"
);
pub(super) const ASSISTANT_MESSAGE_TOOL_RESULT: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/assistant_message_tool_result.jsonl"
);
pub(super) const STREAM_EVENT_TEXT_DELTA: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/stream_event_text_delta.jsonl"
);
pub(super) const STREAM_EVENT_INPUT_JSON_DELTA: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/stream_event_input_json_delta.jsonl"
);
pub(super) const STREAM_EVENT_TOOL_USE_START: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/stream_event_tool_use_start.jsonl"
);
pub(super) const STREAM_EVENT_TOOL_RESULT_START: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/stream_event_tool_result_start.jsonl"
);
pub(super) const UNKNOWN_OUTER_TYPE: &str = include_str!(
    "../../../../../claude_code/tests/fixtures/stream_json/v1/unknown_outer_type.jsonl"
);

pub(super) fn parse_stream_json_fixture(text: &str) -> ClaudeStreamJsonEvent {
    let line = text
        .lines()
        .find(|line| !line.chars().all(|ch| ch.is_whitespace()))
        .expect("fixture contains a non-empty line");
    let mut parser = ClaudeStreamJsonParser::new();
    parser
        .parse_line(line)
        .expect("fixture parses")
        .expect("fixture yields a typed event")
}

pub(super) fn map_fixture(text: &str) -> AgentWrapperEvent {
    let event = parse_stream_json_fixture(text);
    let mapped = super::super::mapping::map_stream_json_event(event);
    assert_eq!(
        mapped.len(),
        1,
        "fixture should map to exactly one wrapper event"
    );
    mapped
        .into_iter()
        .next()
        .expect("fixture mapping returns at least one event")
}

pub(super) fn success_exit_status() -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
}

pub(super) fn exit_status_with_code(code: i32) -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code << 8)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code as u32)
    }
}

pub(super) fn sample_mcp_add_request() -> AgentWrapperMcpAddRequest {
    AgentWrapperMcpAddRequest {
        name: "demo".to_string(),
        transport: AgentWrapperMcpAddTransport::Stdio {
            command: vec!["node".to_string()],
            args: vec!["server.js".to_string()],
            env: std::collections::BTreeMap::from([(
                "SERVER_ONLY".to_string(),
                "server-value".to_string(),
            )]),
        },
        context: Default::default(),
    }
}

pub(super) fn sample_mcp_remove_request() -> AgentWrapperMcpRemoveRequest {
    AgentWrapperMcpRemoveRequest {
        name: "demo".to_string(),
        context: Default::default(),
    }
}

pub(super) fn new_adapter() -> ClaudeHarnessAdapter {
    new_test_adapter(ClaudeCodeBackendConfig::default())
}

pub(super) fn new_adapter_with_config(config: ClaudeCodeBackendConfig) -> ClaudeHarnessAdapter {
    new_test_adapter(config)
}

pub(super) fn new_adapter_with_run_start_cwd(
    run_start_cwd: Option<PathBuf>,
) -> ClaudeHarnessAdapter {
    new_test_adapter_with_run_start_cwd(ClaudeCodeBackendConfig::default(), run_start_cwd)
}

pub(super) fn new_adapter_with_config_and_run_start_cwd(
    config: ClaudeCodeBackendConfig,
    run_start_cwd: Option<PathBuf>,
) -> ClaudeHarnessAdapter {
    new_test_adapter_with_run_start_cwd(config, run_start_cwd)
}

fn build_fake_claude_binary(repo_root: &Path, _target_dir: &Path) -> Result<(), String> {
    let output = Command::new("cargo")
        .args([
            "build",
            "-p",
            "unified-agent-api",
            "--bin",
            "fake_claude_stream_json_agent_api",
            "--all-features",
        ])
        .current_dir(repo_root)
        .output()
        .map_err(|err| format!("spawn cargo build: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo build failed: status={:?}, stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn direct_fake_claude_binary(target_dir: &Path) -> Option<PathBuf> {
    let binary_name = if cfg!(windows) {
        "fake_claude_stream_json_agent_api.exe"
    } else {
        "fake_claude_stream_json_agent_api"
    };
    let direct_binary = target_dir.join(binary_name);
    direct_binary.exists().then_some(direct_binary)
}

fn find_existing_fake_claude_binary(target_dir: &Path) -> Option<PathBuf> {
    let deps_dir = target_dir.join("deps");
    let prefix = "fake_claude_stream_json_agent_api-";
    let deps_binary = std_fs::read_dir(&deps_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                return false;
            };
            if cfg!(windows) {
                file_name.starts_with(prefix) && file_name.ends_with(".exe")
            } else {
                file_name.starts_with(prefix) && !file_name.contains('.')
            }
        });
    deps_binary
}

fn resolve_fake_claude_binary_now_with<F>(
    target_dir: &Path,
    repo_root: &Path,
    build_if_missing: F,
) -> Result<PathBuf, String>
where
    F: Fn(&Path, &Path) -> Result<(), String>,
{
    if let Some(binary) = direct_fake_claude_binary(target_dir) {
        return Ok(binary);
    }

    let build_error = build_if_missing(repo_root, target_dir).err();

    if let Some(binary) = direct_fake_claude_binary(target_dir) {
        return Ok(binary);
    }

    if let Some(binary) = find_existing_fake_claude_binary(target_dir) {
        return Ok(binary);
    }

    if let Some(err) = build_error {
        return Err(err);
    }

    Err(format!(
        "cargo build succeeded but fake_claude_stream_json_agent_api was not found under {target_dir:?}"
    ))
}

fn resolve_fake_claude_binary_now(target_dir: &Path, repo_root: &Path) -> Result<PathBuf, String> {
    resolve_fake_claude_binary_now_with(target_dir, repo_root, build_fake_claude_binary)
}

pub(super) fn fake_claude_binary() -> PathBuf {
    static BUILD_GATE: OnceLock<std::sync::Mutex<()>> = OnceLock::new();

    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_fake_claude_stream_json_agent_api") {
        let path = PathBuf::from(path);
        if path.exists() {
            return path;
        }
    }

    let current_exe = std::env::current_exe().expect("resolve current test binary path");
    let target_dir = current_exe
        .parent()
        .and_then(|dir| dir.parent())
        .expect("resolve target dir from current test binary");
    let repo_root = target_dir
        .parent()
        .and_then(|dir| dir.parent())
        .expect("resolve repo root from current test binary");

    if let Some(existing) = direct_fake_claude_binary(target_dir) {
        return existing;
    }

    let gate = BUILD_GATE.get_or_init(|| std::sync::Mutex::new(()));
    let _guard = gate.lock().expect("lock fake Claude build gate");

    resolve_fake_claude_binary_now(target_dir, repo_root)
        .unwrap_or_else(|err| panic!("resolve fake Claude binary from {target_dir:?}: {err}"))
}

#[test]
fn fake_claude_binary_finds_deps_executable_when_top_level_binary_is_absent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let target_dir = temp.path().join("debug");
    let deps_dir = target_dir.join("deps");
    std_fs::create_dir_all(&deps_dir).expect("create deps dir");

    let deps_binary = deps_dir.join(if cfg!(windows) {
        "fake_claude_stream_json_agent_api-deadbeef.exe"
    } else {
        "fake_claude_stream_json_agent_api-deadbeef"
    });
    std_fs::write(&deps_binary, b"test").expect("write deps binary");

    let discovered =
        find_existing_fake_claude_binary(&target_dir).expect("deps binary should be discovered");
    assert_eq!(discovered, deps_binary);
}

#[test]
fn fake_claude_binary_prefers_top_level_binary_over_deps_executable() {
    let temp = tempfile::tempdir().expect("tempdir");
    let target_dir = temp.path().join("debug");
    let deps_dir = target_dir.join("deps");
    std_fs::create_dir_all(&deps_dir).expect("create deps dir");

    let top_level_binary = target_dir.join(if cfg!(windows) {
        "fake_claude_stream_json_agent_api.exe"
    } else {
        "fake_claude_stream_json_agent_api"
    });
    std_fs::write(&top_level_binary, b"top-level").expect("write top-level binary");

    let deps_binary = deps_dir.join(if cfg!(windows) {
        "fake_claude_stream_json_agent_api-deadbeef.exe"
    } else {
        "fake_claude_stream_json_agent_api-deadbeef"
    });
    std_fs::write(&deps_binary, b"deps").expect("write deps binary");

    let discovered =
        direct_fake_claude_binary(&target_dir).expect("top-level binary should be discovered");
    assert_eq!(discovered, top_level_binary);
}

#[test]
fn fake_claude_binary_resolution_does_not_return_stale_top_level_path_after_removal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let target_dir = temp.path().join("debug");
    let deps_dir = target_dir.join("deps");
    std_fs::create_dir_all(&deps_dir).expect("create deps dir");

    let top_level_binary = target_dir.join(if cfg!(windows) {
        "fake_claude_stream_json_agent_api.exe"
    } else {
        "fake_claude_stream_json_agent_api"
    });
    std_fs::write(&top_level_binary, b"top-level").expect("write top-level binary");

    let first = resolve_fake_claude_binary_now_with(&target_dir, temp.path(), |_, _| Ok(()))
        .expect("top-level binary should resolve");
    assert_eq!(first, top_level_binary);

    std_fs::remove_file(&top_level_binary).expect("remove top-level binary");

    let deps_binary = deps_dir.join(if cfg!(windows) {
        "fake_claude_stream_json_agent_api-deadbeef.exe"
    } else {
        "fake_claude_stream_json_agent_api-deadbeef"
    });
    std_fs::write(&deps_binary, b"deps").expect("write deps binary");

    let second = resolve_fake_claude_binary_now_with(&target_dir, temp.path(), |_, _| Ok(()))
        .expect("deps binary should resolve after top-level removal");
    assert_eq!(second, deps_binary);
    assert_ne!(second, top_level_binary);
}

pub(super) fn expected_add_dirs_env(dirs: &[PathBuf]) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([(
        "FAKE_CLAUDE_EXPECT_ADD_DIR_COUNT".to_string(),
        dirs.len().to_string(),
    )]);
    for (index, dir) in dirs.iter().enumerate() {
        env.insert(
            format!("FAKE_CLAUDE_EXPECT_ADD_DIR_{index}"),
            dir.display().to_string(),
        );
    }
    env
}

pub(super) fn expect_no_add_dir_env() -> BTreeMap<String, String> {
    BTreeMap::from([("FAKE_CLAUDE_EXPECT_NO_ADD_DIR".to_string(), "1".to_string())])
}

pub(super) fn add_dirs_payload(dirs: &[impl AsRef<str>]) -> JsonValue {
    json!({
        "dirs": dirs.iter().map(|dir| dir.as_ref()).collect::<Vec<_>>()
    })
}

pub(super) fn parse_single_line(line: &str) -> ClaudeStreamJsonEvent {
    let mut parser = ClaudeStreamJsonParser::new();
    parser
        .parse_line(line)
        .expect("line parses")
        .expect("line yields a typed event")
}

pub(super) fn handle_facet_schema(event: &crate::AgentWrapperEvent) -> Option<&str> {
    event
        .data
        .as_ref()
        .and_then(|v| v.get("schema"))
        .and_then(|v| v.as_str())
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
