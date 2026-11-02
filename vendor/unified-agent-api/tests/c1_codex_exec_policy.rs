#![cfg(feature = "codex")]

use std::{collections::BTreeMap, fs, path::PathBuf, pin::Pin, time::Duration};

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest,
};
use futures_core::Stream;
use serde_json::json;
use tempfile::tempdir;

async fn drain_to_none(
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

fn fake_codex_binary() -> PathBuf {
    PathBuf::from(env!(
        "CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api"
    ))
}

fn base_env() -> BTreeMap<String, String> {
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

fn add_dir_expectations(dirs: &[PathBuf]) -> BTreeMap<String, String> {
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

fn model_expectations(model: &str) -> BTreeMap<String, String> {
    [("FAKE_CODEX_EXPECT_MODEL".to_string(), model.to_string())]
        .into_iter()
        .collect()
}

fn handle_facet_index(events: &[AgentWrapperEvent]) -> Option<usize> {
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

fn any_event_contains(events: &[AgentWrapperEvent], needle: &str) -> bool {
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

const ADD_DIRS_RUNTIME_REJECTION_MESSAGE: &str = "add_dirs rejected by runtime";
const ADD_DIR_LEAK_SENTINELS: [&str; 3] = [
    "ADD_DIR_RAW_PATH_SECRET",
    "ADD_DIR_STDOUT_SECRET",
    "ADD_DIR_STDERR_SECRET",
];

fn assert_no_add_dir_sentinel_leaks_in_events(events: &[AgentWrapperEvent]) {
    for sentinel in ADD_DIR_LEAK_SENTINELS {
        assert!(
            !any_event_contains(events, sentinel),
            "expected add-dir runtime rejection sentinel {sentinel} to stay backend-private"
        );
    }
}

async fn assert_exec_add_dirs_runtime_rejection(
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(&add_dirs))
            .chain([(
                "FAKE_CODEX_SCENARIO".to_string(),
                "add_dirs_runtime_rejection_exec".to_string(),
            )])
            .chain(extra_env)
            .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "agent_api.exec.add_dirs.v1".to_string(),
                json!({"dirs": add_dirs.iter().map(|dir| dir.display().to_string()).collect::<Vec<_>>() }),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    let handle_idx = handle_facet_index(&seen).expect("expected session handle facet");
    let error_indices: Vec<_> = seen
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (event.kind == AgentWrapperEventKind::Error).then_some(idx))
        .collect();
    assert_eq!(error_indices.len(), 1, "expected exactly one Error event");
    let error_idx = error_indices[0];
    assert!(
        handle_idx < error_idx,
        "expected handle facet status event before backend error"
    );
    assert_eq!(
        seen.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen[error_idx].message.as_deref(),
        Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
    );
    assert_no_add_dir_sentinel_leaks_in_events(&seen);

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, ADD_DIRS_RUNTIME_REJECTION_MESSAGE);
            for sentinel in ADD_DIR_LEAK_SENTINELS {
                assert!(
                    !message.contains(sentinel),
                    "expected add-dir runtime rejection sentinel {sentinel} to stay out of completion error"
                );
            }
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

const EXTERNAL_SANDBOX_WARNING: &str =
    "DANGEROUS: external sandbox exec policy enabled (agent_api.exec.external_sandbox.v1=true)";

#[tokio::test]
async fn codex_backend_defaults_to_non_interactive_and_workspace_write_sandbox() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        seen.iter()
            .any(|ev| ev.kind == AgentWrapperEventKind::Status),
        "expected at least one Status event"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn sandbox_mode_extension_overrides_codex_sandbox() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [
            (
                "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
                "danger-full-access".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
                "never".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let mut extensions = BTreeMap::new();
    extensions.insert(
        "backend.codex.exec.sandbox_mode".to_string(),
        serde_json::Value::String("danger-full-access".to_string()),
    );

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions,
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _ = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn non_interactive_false_does_not_force_ask_for_approval() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [
            (
                "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
                "workspace-write".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
                "<absent>".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let mut extensions = BTreeMap::new();
    extensions.insert(
        "agent_api.exec.non_interactive".to_string(),
        serde_json::Value::Bool(false),
    );

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions,
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        seen.iter()
            .any(|ev| ev.kind == AgentWrapperEventKind::Status),
        "expected status events even when interactive"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn codex_external_sandbox_exec_maps_to_dangerous_bypass_argv_and_emits_warning_before_handle_facet(
) {
    let backend = CodexBackend::new(CodexBackendConfig {
        allow_external_sandbox_exec: true,
        binary: Some(fake_codex_binary()),
        env: [
            ("FAKE_CODEX_SCENARIO".to_string(), "ok".to_string()),
            (
                "FAKE_CODEX_EXPECT_DANGEROUS_BYPASS".to_string(),
                "1".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let mut extensions = BTreeMap::new();
    extensions.insert(
        "agent_api.exec.external_sandbox.v1".to_string(),
        serde_json::Value::Bool(true),
    );

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions,
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let mut warning_idx: Option<usize> = None;
    for (idx, event) in seen.iter().enumerate() {
        if event.kind == AgentWrapperEventKind::Status
            && event.message.as_deref() == Some(EXTERNAL_SANDBOX_WARNING)
        {
            assert!(
                warning_idx.is_none(),
                "expected exactly one warning Status event"
            );
            warning_idx = Some(idx);
            assert_eq!(event.channel.as_deref(), Some("status"));
            assert_eq!(event.data, None);
        }
    }
    let warning_idx = warning_idx.expect("expected warning Status event");

    let handle_idx = seen
        .iter()
        .enumerate()
        .find(|(_, event)| {
            event.kind == AgentWrapperEventKind::Status
                && event
                    .data
                    .as_ref()
                    .and_then(|data| data.get("schema"))
                    .and_then(serde_json::Value::as_str)
                    == Some("agent_api.session.handle.v1")
        })
        .map(|(idx, _)| idx)
        .expect("expected session handle facet Status event");

    assert!(
        warning_idx < handle_idx,
        "expected warning to be emitted before the session handle facet Status event"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn codex_exec_without_add_dirs_emits_no_add_dir_flags() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(&[]))
            .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _ = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn codex_exec_with_add_dirs_emits_repeated_flags_in_order() {
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(&add_dirs))
            .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "agent_api.exec.add_dirs.v1".to_string(),
                json!({"dirs": add_dirs.iter().map(|dir| dir.display().to_string()).collect::<Vec<_>>() }),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _ = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn codex_exec_with_model_emits_model_before_add_dirs() {
    let model = "gpt-5-codex";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        model: Some(model.to_string()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(&add_dirs))
            .chain(model_expectations(model))
            .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "agent_api.exec.add_dirs.v1".to_string(),
                json!({"dirs": add_dirs.iter().map(|dir| dir.display().to_string()).collect::<Vec<_>>() }),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _ = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn codex_exec_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    assert_exec_add_dirs_runtime_rejection(std::iter::empty()).await;
}

#[tokio::test]
async fn codex_exec_add_dirs_runtime_rejection_is_fatal_even_on_zero_exit() {
    assert_exec_add_dirs_runtime_rejection([(
        "FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE".to_string(),
        "0".to_string(),
    )])
    .await;
}
