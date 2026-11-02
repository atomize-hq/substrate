#![cfg(feature = "claude_code")]

use std::{collections::BTreeMap, fs, pin::Pin, time::Duration};

#[allow(dead_code, unused_imports)]
#[path = "c1_claude_exec_policy/support.rs"]
mod support;

use serde_json::{json, Value};
use support::*;
use tempfile::tempdir;

const ADD_DIRS_RUNTIME_REJECTION_MESSAGE: &str = "add_dirs rejected by runtime";
const ADD_DIR_LEAK_SENTINELS: [&str; 3] = [
    "ADD_DIR_RAW_PATH_SECRET",
    "ADD_DIR_STDOUT_SECRET",
    "ADD_DIR_STDERR_SECRET",
];
const STREAM_TIMEOUT: Duration = Duration::from_secs(2);

fn add_dir_expectations(dirs: &[std::path::PathBuf]) -> BTreeMap<String, String> {
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

fn any_user_visible_surface_contains(events: &[AgentWrapperEvent], needle: &str) -> bool {
    events.iter().any(|event| {
        event
            .message
            .as_deref()
            .is_some_and(|message| message.contains(needle))
            || event
                .text
                .as_deref()
                .is_some_and(|text| text.contains(needle))
    })
}

fn assert_no_add_dir_sentinel_leaks_in_events(events: &[AgentWrapperEvent]) {
    for sentinel in ADD_DIR_LEAK_SENTINELS {
        assert!(
            !any_user_visible_surface_contains(events, sentinel),
            "expected add-dir runtime rejection sentinel {sentinel} to stay out of user-visible event surfaces"
        );
    }
}

async fn collect_events_to_none(
    mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>,
    timeout: Duration,
) -> Vec<AgentWrapperEvent> {
    tokio::time::timeout(timeout, async move {
        let mut events = Vec::new();
        while let Some(event) = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await {
            events.push(event);
        }
        events
    })
    .await
    .expect("events stream should reach None before timeout")
}

async fn assert_runtime_rejection_parity(
    test_name: &str,
    prompt: &str,
    scenario: &str,
    session_extension: Option<(&str, Value)>,
    resume_id: Option<&str>,
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let mut env = BTreeMap::from([
        ("FAKE_CLAUDE_SCENARIO".to_string(), scenario.to_string()),
        ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
    ]);
    env.extend(add_dir_expectations(&add_dirs));
    if let Some(resume_id) = resume_id {
        env.insert(
            "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        );
    }
    env.extend(extra_env);

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env,
        ..Default::default()
    });

    let mut extensions = BTreeMap::from([(
        "agent_api.exec.add_dirs.v1".to_string(),
        json!({
            "dirs": add_dirs
                .iter()
                .map(|dir| dir.display().to_string())
                .collect::<Vec<_>>(),
        }),
    )]);
    if let Some((key, value)) = session_extension {
        extensions.insert(key.to_string(), value);
    }

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions,
            ..Default::default()
        })
        .await
        .unwrap_or_else(|err| {
            panic!("{test_name}: expected handle before runtime rejection: {err:?}")
        });

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = collect_events_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    let handle_idx = session_handle_facet_index(&seen)
        .unwrap_or_else(|| panic!("{test_name}: expected session handle facet"));
    let error_indices: Vec<_> = seen
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (event.kind == AgentWrapperEventKind::Error).then_some(idx))
        .collect();
    assert_eq!(
        error_indices.len(),
        1,
        "{test_name}: expected exactly one terminal Error event"
    );

    let error_idx = error_indices[0];
    assert_eq!(
        first_error_index(&seen),
        Some(error_idx),
        "{test_name}: expected the first Error event to be the only Error event"
    );
    assert!(
        handle_idx < error_idx,
        "{test_name}: expected session handle facet before runtime rejection"
    );
    assert_eq!(
        seen.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "{test_name}: expected Error event to be terminal before stream close"
    );
    assert_eq!(
        seen[error_idx].message.as_deref(),
        Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE),
        "{test_name}: expected the safe runtime rejection message on the terminal Error event"
    );
    assert_no_add_dir_sentinel_leaks_in_events(&seen);

    let err = tokio::time::timeout(STREAM_TIMEOUT, completion)
        .await
        .unwrap_or_else(|_| panic!("{test_name}: completion should resolve after stream finality"))
        .expect_err("completion should surface a backend error");
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(
                message, ADD_DIRS_RUNTIME_REJECTION_MESSAGE,
                "{test_name}: completion error should match the terminal Error event message"
            );
            for sentinel in ADD_DIR_LEAK_SENTINELS {
                assert!(
                    !message.contains(sentinel),
                    "{test_name}: expected add-dir runtime rejection sentinel {sentinel} to stay out of completion error"
                );
            }
        }
        other => panic!("{test_name}: expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn fresh_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    assert_runtime_rejection_parity(
        "fresh_add_dirs_runtime_rejection_emits_handle_before_backend_error",
        "hello world",
        "add_dirs_runtime_rejection_fresh",
        None,
        None,
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn resume_last_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    assert_runtime_rejection_parity(
        "resume_last_add_dirs_runtime_rejection_emits_handle_before_backend_error",
        "hello world",
        "add_dirs_runtime_rejection_resume_last",
        Some(("agent_api.session.resume.v1", json!({"selector": "last"}))),
        None,
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn resume_id_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    let resume_id = "sess-123";
    assert_runtime_rejection_parity(
        "resume_id_add_dirs_runtime_rejection_emits_handle_before_backend_error",
        "hello world",
        "add_dirs_runtime_rejection_resume_id",
        Some((
            "agent_api.session.resume.v1",
            json!({"selector": "id", "id": resume_id}),
        )),
        Some(resume_id),
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn fork_last_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    assert_runtime_rejection_parity(
        "fork_last_add_dirs_runtime_rejection_emits_handle_before_backend_error",
        "hello world",
        "add_dirs_runtime_rejection_fork_last",
        Some(("agent_api.session.fork.v1", json!({"selector": "last"}))),
        None,
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn fork_id_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    let fork_id = "sess-123";
    assert_runtime_rejection_parity(
        "fork_id_add_dirs_runtime_rejection_emits_handle_before_backend_error",
        "hello world",
        "add_dirs_runtime_rejection_fork_id",
        Some((
            "agent_api.session.fork.v1",
            json!({"selector": "id", "id": fork_id}),
        )),
        Some(fork_id),
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn fresh_add_dirs_runtime_rejection_is_fatal_even_on_zero_exit() {
    assert_runtime_rejection_parity(
        "fresh_add_dirs_runtime_rejection_is_fatal_even_on_zero_exit",
        "hello world",
        "add_dirs_runtime_rejection_fresh",
        None,
        None,
        [(
            "FAKE_CLAUDE_RUNTIME_REJECTION_EXIT_CODE".to_string(),
            "0".to_string(),
        )],
    )
    .await;
}

#[tokio::test]
async fn nested_add_dirs_rejection_detail_does_not_override_result_error() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let mut env = BTreeMap::from([
        (
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "add_dirs_runtime_rejection_nested_detail_trap".to_string(),
        ),
        ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
    ]);
    env.extend(add_dir_expectations(&add_dirs));

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env,
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.exec.add_dirs.v1".to_string(),
                json!({
                    "dirs": add_dirs
                        .iter()
                        .map(|dir| dir.display().to_string())
                        .collect::<Vec<_>>(),
                }),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("nested detail trap should still return a handle");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = collect_events_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert!(
        session_handle_facet_index(&seen).is_some(),
        "expected session handle facet before failure"
    );

    let error_messages: Vec<_> = seen
        .iter()
        .filter(|event| event.kind == AgentWrapperEventKind::Error)
        .map(|event| event.message.as_deref())
        .collect();
    assert_eq!(
        error_messages,
        vec![Some("result error")],
        "nested detail trap should preserve the original ResultError path"
    );
    assert!(
        error_messages
            .iter()
            .all(|message| *message != Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE)),
        "nested detail trap must not be rewritten to the pinned add-dir rejection"
    );

    let completion = tokio::time::timeout(STREAM_TIMEOUT, completion)
        .await
        .expect("completion should resolve after stream finality")
        .expect("completion should not become a backend error");
    assert!(
        !completion.status.success(),
        "nested detail trap should still preserve the non-zero exit status"
    );
}

#[tokio::test]
async fn top_level_add_dirs_rejection_beats_nested_resume_not_found_detail() {
    let resume_id = "sess-123";
    assert_runtime_rejection_parity(
        "top_level_add_dirs_rejection_beats_nested_resume_not_found_detail",
        "hello world",
        "add_dirs_runtime_rejection_resume_id_not_found_detail_trap",
        Some((
            "agent_api.session.resume.v1",
            json!({"selector": "id", "id": resume_id}),
        )),
        Some(resume_id),
        std::iter::empty(),
    )
    .await;
}
