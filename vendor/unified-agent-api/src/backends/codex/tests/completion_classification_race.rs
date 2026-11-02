use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use tempfile::tempdir;

use super::support::*;

const BUFFERED_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);
const BACKPRESSURE_ASSERT_TIMEOUT: Duration = Duration::from_millis(200);
const POST_DROP_PENDING_TIMEOUT: Duration = Duration::from_millis(50);

fn fake_codex_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api")
    {
        return PathBuf::from(path);
    }

    let current_exe = std::env::current_exe().expect("resolve current test binary path");
    let target_dir = current_exe
        .parent()
        .and_then(|dir| dir.parent())
        .expect("resolve target dir from current test binary");
    let mut binary = target_dir.join("fake_codex_stream_exec_scenarios_agent_api");
    if cfg!(windows) {
        binary.set_extension("exe");
    }
    binary
}

fn buffered_env() -> [(String, String); 2] {
    [
        (
            "FAKE_CODEX_BUFFERED_EVENT_COUNT".to_string(),
            "1024".to_string(),
        ),
        (
            "FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES".to_string(),
            "1024".to_string(),
        ),
    ]
}

fn slow_drain_buffered_env() -> [(String, String); 4] {
    [
        (
            "FAKE_CODEX_BUFFERED_EVENT_COUNT".to_string(),
            "4096".to_string(),
        ),
        (
            "FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES".to_string(),
            "2048".to_string(),
        ),
        (
            "FAKE_CODEX_BUFFERED_EVENT_SLEEP_EVERY".to_string(),
            "8".to_string(),
        ),
        (
            "FAKE_CODEX_BUFFERED_EVENT_SLEEP_MILLIS".to_string(),
            "1".to_string(),
        ),
    ]
}

fn add_dir_env(dirs: &[PathBuf]) -> BTreeMap<String, String> {
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

fn add_dirs_fixture() -> (tempfile::TempDir, Vec<PathBuf>) {
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    std::fs::create_dir_all(&dir_a).expect("alpha dir");
    std::fs::create_dir_all(&dir_b).expect("beta dir");
    (temp, vec![dir_a, dir_b])
}

async fn assert_buffered_add_dirs_runtime_rejection(
    scenario: &str,
    selector: SessionSelectorV1,
    expected_message: &'static str,
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let run_start = tempdir().expect("tempdir");
    let (_temp, add_dirs) = add_dirs_fixture();
    let env = add_dir_env(&add_dirs)
        .into_iter()
        .chain([
            ("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string()),
            (
                "FAKE_CODEX_EXPECT_PROMPT".to_string(),
                "hello world".to_string(),
            ),
        ])
        .chain(buffered_env())
        .chain(extra_env)
        .collect::<BTreeMap<_, _>>();

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            binary: Some(fake_codex_binary()),
            ..Default::default()
        },
        Some(run_start.path().to_path_buf()),
    );

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello world".to_string(),
            model_id: None,
            working_dir: Some(PathBuf::from(".")),
            effective_timeout: None,
            env,
            policy: CodexExecPolicy {
                add_dirs,
                non_interactive: true,
                external_sandbox: false,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume: Some(selector),
                fork: None,
            },
        })
        .await
        .expect("spawn succeeds");

    let mut completion = spawned.completion;
    assert!(
        tokio::time::timeout(BACKPRESSURE_ASSERT_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending while buffered events are not drained"
    );

    let backend_events: Vec<_> = spawned
        .events
        .map(|result| result.expect("backend event stream is infallible for fake codex"))
        .collect()
        .await;

    let completion = tokio::time::timeout(BUFFERED_COMPLETION_TIMEOUT, completion)
        .await
        .expect("completion resolves")
        .expect("completion is Ok for fake codex");
    let err = adapter
        .map_completion(completion)
        .expect_err("add-dir runtime rejection must map to Backend error");
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, expected_message),
        other => panic!("expected Backend error, got: {other:?}"),
    }

    let mapped_events: Vec<_> = backend_events
        .into_iter()
        .flat_map(|event| adapter.map_event(event))
        .collect();

    let handle_idx = mapped_events
        .iter()
        .position(|event| {
            event.kind == AgentWrapperEventKind::Status
                && handle_schema(event) == Some(CAP_SESSION_HANDLE_V1)
        })
        .expect("expected session handle facet");
    let error_idx = mapped_events
        .iter()
        .position(|event| event.kind == AgentWrapperEventKind::Error)
        .expect("expected terminal error event");
    assert!(handle_idx < error_idx);
    assert_eq!(
        mapped_events[error_idx].message.as_deref(),
        Some(expected_message)
    );
    assert_eq!(
        mapped_events.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error)
    );
}

async fn assert_buffered_selection_failure(
    scenario: &str,
    selector: SessionSelectorV1,
    expected_message: &'static str,
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let run_start = tempdir().expect("tempdir");
    let env = [
        ("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string()),
        (
            "FAKE_CODEX_EXPECT_PROMPT".to_string(),
            "hello world".to_string(),
        ),
    ]
    .into_iter()
    .chain(buffered_env())
    .chain(extra_env)
    .collect::<BTreeMap<_, _>>();

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            binary: Some(fake_codex_binary()),
            ..Default::default()
        },
        Some(run_start.path().to_path_buf()),
    );

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello world".to_string(),
            model_id: None,
            working_dir: Some(PathBuf::from(".")),
            effective_timeout: None,
            env,
            policy: CodexExecPolicy {
                add_dirs: Vec::new(),
                non_interactive: true,
                external_sandbox: false,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume: Some(selector),
                fork: None,
            },
        })
        .await
        .expect("spawn succeeds");

    let backend_events: Vec<_> = spawned
        .events
        .map(|result| result.expect("backend event stream is infallible for fake codex"))
        .collect()
        .await;

    let completion = tokio::time::timeout(BUFFERED_COMPLETION_TIMEOUT, spawned.completion)
        .await
        .expect("completion resolves")
        .expect("completion is Ok for fake codex");
    let err = adapter
        .map_completion(completion)
        .expect_err("selection failure must map to Backend error");
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, expected_message),
        other => panic!("expected Backend error, got: {other:?}"),
    }

    let mapped_events: Vec<_> = backend_events
        .into_iter()
        .flat_map(|event| adapter.map_event(event))
        .collect();

    let error_messages: Vec<_> = mapped_events
        .iter()
        .filter(|event| event.kind == AgentWrapperEventKind::Error)
        .filter_map(|event| event.message.as_deref())
        .collect();
    assert_eq!(error_messages, vec![expected_message]);
    assert_eq!(
        mapped_events.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error)
    );
}

async fn assert_dropping_events_waits_for_add_dirs_classification(
    scenario: &str,
    selector: SessionSelectorV1,
    expected_message: &'static str,
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let run_start = tempdir().expect("tempdir");
    let (_temp, add_dirs) = add_dirs_fixture();
    let env = add_dir_env(&add_dirs)
        .into_iter()
        .chain([
            ("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string()),
            (
                "FAKE_CODEX_EXPECT_PROMPT".to_string(),
                "hello world".to_string(),
            ),
        ])
        .chain(slow_drain_buffered_env())
        .chain(extra_env)
        .collect::<BTreeMap<_, _>>();

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            binary: Some(fake_codex_binary()),
            ..Default::default()
        },
        Some(run_start.path().to_path_buf()),
    );

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello world".to_string(),
            model_id: None,
            working_dir: Some(PathBuf::from(".")),
            effective_timeout: None,
            env,
            policy: CodexExecPolicy {
                add_dirs,
                non_interactive: true,
                external_sandbox: false,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume: Some(selector),
                fork: None,
            },
        })
        .await
        .expect("spawn succeeds");

    let mut events = spawned.events;
    let mut completion = spawned.completion;

    let first = tokio::time::timeout(
        BUFFERED_COMPLETION_TIMEOUT,
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event arrives")
    .expect("expected a live event before drop")
    .expect("backend event stream is infallible for fake codex");
    let _ = first;

    assert!(
        tokio::time::timeout(BACKPRESSURE_ASSERT_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending while buffered events are still attached"
    );

    drop(events);

    assert!(
        tokio::time::timeout(POST_DROP_PENDING_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending until the event task drains buffered suppressed errors"
    );

    let completion = tokio::time::timeout(BUFFERED_COMPLETION_TIMEOUT, completion)
        .await
        .expect("completion resolves")
        .expect("completion is Ok for fake codex");
    let err = adapter
        .map_completion(completion)
        .expect_err("selection failure must map to Backend error");
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, expected_message),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn buffered_resume_last_add_dirs_runtime_rejection_stays_pinned_before_events_are_drained() {
    assert_buffered_add_dirs_runtime_rejection(
        "add_dirs_runtime_rejection_resume_last_buffered_tail",
        SessionSelectorV1::Last,
        PINNED_ADD_DIRS_RUNTIME_REJECTION,
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn buffered_resume_id_add_dirs_runtime_rejection_stays_pinned_before_events_are_drained() {
    let resume_id = "thread-123";
    assert_buffered_add_dirs_runtime_rejection(
        "add_dirs_runtime_rejection_resume_id_buffered_tail",
        SessionSelectorV1::Id {
            id: resume_id.to_string(),
        },
        PINNED_ADD_DIRS_RUNTIME_REJECTION,
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
    )
    .await;
}

#[tokio::test]
async fn buffered_resume_last_selection_failure_stays_pinned_before_events_are_drained() {
    assert_buffered_selection_failure(
        "resume_last_not_found_buffered_transport_errors",
        SessionSelectorV1::Last,
        super::super::pinned_selection_failure_message(&SessionSelectorV1::Last),
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn buffered_resume_id_selection_failure_stays_pinned_before_events_are_drained() {
    let resume_id = "thread-123";
    assert_buffered_selection_failure(
        "resume_id_not_found_buffered_transport_errors",
        SessionSelectorV1::Id {
            id: resume_id.to_string(),
        },
        super::super::pinned_selection_failure_message(&SessionSelectorV1::Id {
            id: resume_id.to_string(),
        }),
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
    )
    .await;
}

#[tokio::test]
async fn dropping_events_does_not_bypass_buffered_resume_add_dirs_classification() {
    assert_dropping_events_waits_for_add_dirs_classification(
        "add_dirs_runtime_rejection_resume_last_buffered_tail",
        SessionSelectorV1::Last,
        PINNED_ADD_DIRS_RUNTIME_REJECTION,
        std::iter::empty(),
    )
    .await;
}
