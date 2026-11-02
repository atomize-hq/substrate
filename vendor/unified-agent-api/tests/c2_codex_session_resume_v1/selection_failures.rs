use std::path::PathBuf;

use agent_api::{
    backends::codex::CodexBackendConfig, AgentWrapperBackend, AgentWrapperError,
    AgentWrapperEventKind,
};
use serde_json::json;

use super::support::{
    assert_backend_error_message, base_env, build_backend, drain_to_none, run_request,
    STREAM_TIMEOUT,
};

async fn assert_selection_failure_case(
    scenario: &str,
    resume_extension: (String, serde_json::Value),
    prompt: &str,
    expected_message: &str,
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let mut env = base_env();
    env.insert("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string());
    env.insert("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string());
    env.extend(extra_env);

    let backend = build_backend(env, None, false);
    let handle = backend
        .run(run_request(prompt, [resume_extension]))
        .await
        .unwrap();
    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert_selection_failure_events(&seen, expected_message);
    assert_backend_error_message(handle.completion, expected_message).await;
}

fn assert_selection_failure_events(seen: &[agent_api::AgentWrapperEvent], expected_message: &str) {
    let error_events: Vec<_> = seen
        .iter()
        .filter(|event| event.kind == AgentWrapperEventKind::Error)
        .collect();
    assert_eq!(error_events.len(), 1, "expected exactly one Error event");
    assert_eq!(
        seen.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen.last().and_then(|event| event.message.as_deref()),
        Some(expected_message)
    );
}

#[tokio::test]
async fn invalid_resume_schema_is_rejected_pre_spawn() {
    let backend = agent_api::backends::codex::CodexBackend::new(CodexBackendConfig {
        binary: Some(PathBuf::from("definitely-not-a-real-codex-binary")),
        ..Default::default()
    });

    let err = backend
        .run(run_request(
            "hello",
            [(
                "agent_api.session.resume.v1".to_string(),
                json!("not an object"),
            )],
        ))
        .await
        .unwrap_err();

    assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
}

#[tokio::test]
async fn resume_last_selection_failure_is_translated_and_emits_one_terminal_error_event() {
    assert_selection_failure_case(
        "resume_last_not_found",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        ),
        "hello world",
        "no session found",
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn resume_id_selection_failure_is_translated_and_emits_one_terminal_error_event() {
    let resume_id = "thread-123";
    assert_selection_failure_case(
        "resume_id_not_found",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "id", "id": resume_id}),
        ),
        "hello world",
        "session not found",
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
    )
    .await;
}
