#![cfg(feature = "claude_code")]

use std::{collections::BTreeMap, fs, path::PathBuf, pin::Pin, time::Duration};

use agent_api::{
    backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest,
};
use futures_core::Stream;
use serde_json::json;
use tempfile::tempdir;

fn fake_claude_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_claude_stream_json_agent_api"))
}

fn add_dir_expectations(dirs: &[PathBuf]) -> BTreeMap<String, String> {
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

#[tokio::test]
async fn resume_last_maps_to_continue_and_prompt_is_final_token() {
    let prompt = "hello world";
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "resume_last_assert".to_string(),
            ),
            ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "last"}),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn resume_id_maps_to_resume_flag_and_prompt_is_final_token() {
    let prompt = "hello world";
    let resume_id = "sess-123";

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "resume_id_assert".to_string(),
            ),
            ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
            (
                "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
                resume_id.to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "id", "id": resume_id}),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn resume_last_preserves_add_dir_flags_in_order() {
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
            "resume_last_assert".to_string(),
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
            extensions: [
                (
                    "agent_api.exec.add_dirs.v1".to_string(),
                    json!({"dirs": add_dirs.iter().map(|dir| dir.display().to_string()).collect::<Vec<_>>() }),
                ),
                (
                    "agent_api.session.resume.v1".to_string(),
                    json!({"selector": "last"}),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn resume_id_preserves_add_dir_flags_in_order() {
    let prompt = "hello world";
    let resume_id = "sess-123";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let mut env = BTreeMap::from([
        (
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "resume_id_assert".to_string(),
        ),
        ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
        (
            "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        ),
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
            extensions: [
                (
                    "agent_api.exec.add_dirs.v1".to_string(),
                    json!({"dirs": add_dirs.iter().map(|dir| dir.display().to_string()).collect::<Vec<_>>() }),
                ),
                (
                    "agent_api.session.resume.v1".to_string(),
                    json!({"selector": "id", "id": resume_id}),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn invalid_resume_schema_is_rejected_pre_spawn() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(PathBuf::from("definitely-not-a-real-claude-binary")),
        ..Default::default()
    });

    let err = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!("not an object"),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
}

#[tokio::test]
async fn resume_last_selection_failure_is_translated_and_emits_one_terminal_error_event() {
    let prompt = "hello world";
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "resume_last_not_found".to_string(),
            ),
            ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "last"}),
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
    let error_events: Vec<_> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Error)
        .collect();
    assert_eq!(error_events.len(), 1, "expected exactly one Error event");
    assert_eq!(
        seen.last().map(|ev| ev.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("no session found")
    );

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, "no session found"),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn resume_id_selection_failure_is_translated_and_emits_one_terminal_error_event() {
    let prompt = "hello world";
    let resume_id = "sess-123";

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "resume_id_not_found".to_string(),
            ),
            ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
            (
                "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
                resume_id.to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "id", "id": resume_id}),
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
    let error_events: Vec<_> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Error)
        .collect();
    assert_eq!(error_events.len(), 1, "expected exactly one Error event");
    assert_eq!(
        seen.last().map(|ev| ev.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("session not found")
    );

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, "session not found"),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn resume_last_generic_failure_emits_tail_terminal_error_and_completion_is_ok_non_success() {
    let prompt = "hello world";
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "resume_last_generic_error".to_string(),
            ),
            ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "last"}),
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
    let error_events: Vec<_> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Error)
        .collect();
    assert_eq!(error_events.len(), 1, "expected exactly one Error event");
    assert_eq!(
        seen.last().map(|ev| ev.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("claude_code exited non-zero: code=1 (output redacted)")
    );
    assert_ne!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("no session found")
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(!completion.status.success());
}

#[tokio::test]
async fn resume_id_generic_failure_emits_tail_terminal_error_and_completion_is_ok_non_success() {
    let prompt = "hello world";
    let resume_id = "sess-123";

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "resume_id_generic_error".to_string(),
            ),
            ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
            (
                "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
                resume_id.to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "id", "id": resume_id}),
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
    let error_events: Vec<_> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Error)
        .collect();
    assert_eq!(error_events.len(), 1, "expected exactly one Error event");
    assert_eq!(
        seen.last().map(|ev| ev.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("claude_code exited non-zero: code=1 (output redacted)")
    );
    assert_ne!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("session not found")
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(!completion.status.success());
}

#[tokio::test]
async fn resume_id_file_not_found_does_not_translate_to_session_not_found() {
    let prompt = "hello world";
    let resume_id = "sess-123";

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "resume_id_file_not_found_trap".to_string(),
            ),
            ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
            (
                "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
                resume_id.to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "id", "id": resume_id}),
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
    let error_events: Vec<_> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Error)
        .collect();
    assert_eq!(error_events.len(), 1, "expected exactly one Error event");
    assert_eq!(
        seen.last().map(|ev| ev.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("claude_code exited non-zero: code=1 (output redacted)")
    );
    assert_ne!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("session not found")
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(!completion.status.success());
}
