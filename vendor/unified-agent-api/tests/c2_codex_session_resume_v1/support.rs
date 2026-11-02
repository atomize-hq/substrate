use std::{collections::BTreeMap, fs, path::PathBuf, pin::Pin, time::Duration};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest, DynAgentWrapperCompletion,
};
use futures_core::Stream;
use serde_json::{json, Value};
use tempfile::{tempdir, TempDir};

pub(super) const EXTERNAL_SANDBOX_WARNING: &str =
    "DANGEROUS: external sandbox exec policy enabled (agent_api.exec.external_sandbox.v1=true)";
pub(super) const ADD_DIRS_RUNTIME_REJECTION_MESSAGE: &str = "add_dirs rejected by runtime";
pub(super) const ADD_DIR_LEAK_SENTINELS: [&str; 3] = [
    "ADD_DIR_RAW_PATH_SECRET",
    "ADD_DIR_STDOUT_SECRET",
    "ADD_DIR_STDERR_SECRET",
];
pub(super) const STREAM_TIMEOUT: Duration = Duration::from_secs(2);

pub(super) struct AddDirsFixture {
    _temp: TempDir,
    pub(super) dirs: Vec<PathBuf>,
}

#[cfg(unix)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(super) enum AddDirProbeMode {
    Unsupported,
    Unknown,
    SlowSupported,
    EnvSensitiveSupported,
}

#[cfg(unix)]
pub(super) struct ProbeOnlyCodexFixture {
    _temp: TempDir,
    pub(super) exec_log: PathBuf,
    pub(super) backend: CodexBackend,
}

pub(super) fn fake_codex_binary() -> PathBuf {
    PathBuf::from(env!(
        "CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api"
    ))
}

pub(super) fn base_env() -> BTreeMap<String, String> {
    [
        (
            "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
            "workspace-write".to_string(),
        ),
        (
            "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
            "never".to_string(),
        ),
    ]
    .into_iter()
    .collect()
}

pub(super) fn add_dir_expectations(dirs: &[PathBuf]) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([(
        "FAKE_CODEX_EXPECT_ADD_DIR_COUNT".to_string(),
        dirs.len().to_string(),
    )]);
    for (index, dir) in dirs.iter().enumerate() {
        env.insert(
            format!("FAKE_CODEX_EXPECT_ADD_DIR_{index}"),
            dir.display().to_string(),
        );
    }
    env
}

pub(super) fn model_expectations(model: &str) -> BTreeMap<String, String> {
    [("FAKE_CODEX_EXPECT_MODEL".to_string(), model.to_string())]
        .into_iter()
        .collect()
}

pub(super) fn add_dirs_fixture() -> AddDirsFixture {
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    AddDirsFixture {
        _temp: temp,
        dirs: vec![dir_a, dir_b],
    }
}

pub(super) fn add_dirs_extension(dirs: &[PathBuf]) -> (String, Value) {
    (
        "agent_api.exec.add_dirs.v1".to_string(),
        json!({
            "dirs": dirs
                .iter()
                .map(|dir| dir.display().to_string())
                .collect::<Vec<_>>(),
        }),
    )
}

pub(super) fn build_backend(
    env: BTreeMap<String, String>,
    model: Option<&str>,
    allow_external_sandbox_exec: bool,
) -> CodexBackend {
    CodexBackend::new(CodexBackendConfig {
        allow_external_sandbox_exec,
        binary: Some(fake_codex_binary()),
        env,
        model: model.map(ToOwned::to_owned),
        ..Default::default()
    })
}

#[cfg(unix)]
pub(super) fn build_probe_only_backend(
    mode: AddDirProbeMode,
    env: BTreeMap<String, String>,
    model: Option<&str>,
    allow_external_sandbox_exec: bool,
) -> ProbeOnlyCodexFixture {
    let temp = tempdir().expect("tempdir");
    let exec_log = temp.path().join("exec.log");
    let binary = write_probe_only_codex(temp.path(), mode, &exec_log);
    let backend = CodexBackend::new(CodexBackendConfig {
        allow_external_sandbox_exec,
        binary: Some(binary),
        env,
        model: model.map(ToOwned::to_owned),
        ..Default::default()
    });

    ProbeOnlyCodexFixture {
        _temp: temp,
        exec_log,
        backend,
    }
}

pub(super) fn run_request(
    prompt: &str,
    extensions: impl IntoIterator<Item = (String, Value)>,
) -> AgentWrapperRunRequest {
    AgentWrapperRunRequest {
        prompt: prompt.to_string(),
        extensions: extensions.into_iter().collect(),
        ..Default::default()
    }
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
                    Some(event) => out.push(event),
                    None => break,
                }
            }
        }
    }

    out
}

pub(super) async fn assert_completion_success(
    completion: DynAgentWrapperCompletion,
) -> AgentWrapperCompletion {
    let completion = tokio::time::timeout(STREAM_TIMEOUT, completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
    completion
}

pub(super) async fn assert_backend_error_message(
    completion: DynAgentWrapperCompletion,
    expected_message: &str,
) {
    let err = tokio::time::timeout(STREAM_TIMEOUT, completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, expected_message),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

pub(super) fn any_event_contains(events: &[AgentWrapperEvent], needle: &str) -> bool {
    events.iter().any(|event| {
        event
            .message
            .as_deref()
            .is_some_and(|message| message.contains(needle))
            || event
                .text
                .as_deref()
                .is_some_and(|text| text.contains(needle))
            || event
                .data
                .as_ref()
                .and_then(|data| serde_json::to_string(data).ok())
                .is_some_and(|data| data.contains(needle))
    })
}

pub(super) fn handle_facet_index(events: &[AgentWrapperEvent]) -> Option<usize> {
    events.iter().position(|event| {
        event.kind == AgentWrapperEventKind::Status
            && event
                .data
                .as_ref()
                .and_then(|data| data.get("schema"))
                .and_then(serde_json::Value::as_str)
                == Some("agent_api.session.handle.v1")
    })
}

pub(super) fn assert_external_sandbox_warning_before_session_handle_facet(
    events: &[AgentWrapperEvent],
) {
    let mut warning_idx = None;
    for (idx, event) in events.iter().enumerate() {
        if event.kind == AgentWrapperEventKind::Status
            && event.message.as_deref() == Some(EXTERNAL_SANDBOX_WARNING)
        {
            assert!(
                warning_idx.is_none(),
                "expected exactly one external sandbox warning Status event"
            );
            warning_idx = Some(idx);
            assert_eq!(event.channel.as_deref(), Some("status"));
            assert_eq!(event.data, None);
        }
    }

    let warning_idx = warning_idx.expect("expected external sandbox warning Status event");
    let handle_idx =
        handle_facet_index(events).expect("expected session handle facet Status event");
    assert!(
        warning_idx < handle_idx,
        "expected warning to be emitted before the session handle facet Status event"
    );
}

pub(super) fn assert_no_add_dir_sentinel_leaks_in_events(events: &[AgentWrapperEvent]) {
    for sentinel in ADD_DIR_LEAK_SENTINELS {
        assert!(
            !any_event_contains(events, sentinel),
            "expected add-dir runtime rejection sentinel {sentinel} to stay backend-private"
        );
    }
}

#[cfg(unix)]
fn write_probe_only_codex(
    dir: &std::path::Path,
    mode: AddDirProbeMode,
    exec_log_path: &std::path::Path,
) -> PathBuf {
    let (json_probe, text_probe, help_output) = match mode {
        AddDirProbeMode::Unsupported => (
            r#"echo '{"features":["output_schema"]}'"#,
            r#"echo "output_schema""#,
            r#"echo "Usage: codex exec""#,
        ),
        AddDirProbeMode::Unknown => (
            r#"echo "feature list unavailable" >&2; exit 1"#,
            r#"echo "feature list unavailable" >&2; exit 1"#,
            r#"echo "Usage: codex exec""#,
        ),
        AddDirProbeMode::SlowSupported => (
            r#"sleep 0.2; echo '{"features":["add_dir"]}'"#,
            r#"sleep 0.2; echo "add_dir""#,
            r#"sleep 0.2; echo "Usage: codex --add-dir""#,
        ),
        AddDirProbeMode::EnvSensitiveSupported => (
            r#"if [[ "${FAKE_CODEX_ENABLE_ADD_DIR_PROBE:-}" == "1" ]]; then echo '{"features":["add_dir"]}'; else echo '{"features":["output_schema"]}'; fi"#,
            r#"if [[ "${FAKE_CODEX_ENABLE_ADD_DIR_PROBE:-}" == "1" ]]; then echo "add_dir"; else echo "output_schema"; fi"#,
            r#"if [[ "${FAKE_CODEX_ENABLE_ADD_DIR_PROBE:-}" == "1" ]]; then echo "Usage: codex --add-dir"; else echo "Usage: codex exec"; fi"#,
        ),
    };
    let script = format!(
        r#"#!/bin/bash
log="{log}"
if [[ "$1" == "--version" ]]; then
  echo "codex 0.9.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  {json_probe}
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  {text_probe}
elif [[ "$1" == "--help" ]]; then
  {help_output}
else
  for arg in "$@"; do
    if [[ "$arg" == "exec" ]]; then
      echo "$@ probe_env=${{FAKE_CODEX_ENABLE_ADD_DIR_PROBE:-unset}}" >> "$log"
      exit 99
    fi
  done
fi
"#,
        log = exec_log_path.display(),
        json_probe = json_probe,
        text_probe = text_probe,
        help_output = help_output,
    );
    let binary = dir.join("fake-probe-only-codex.sh");
    fs::write(&binary, script).expect("write fake codex script");
    let mut perms = fs::metadata(&binary).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&binary, perms).expect("set permissions");
    binary
}
