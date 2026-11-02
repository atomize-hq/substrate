use agent_api::AgentWrapperBackend;
use serde_json::json;

use super::support::{
    add_dir_expectations, add_dirs_extension, add_dirs_fixture, assert_completion_success,
    base_env, build_backend, drain_to_none, model_expectations, run_request, STREAM_TIMEOUT,
};

async fn assert_resume_add_dirs_case(
    scenario: &str,
    resume_extension: (String, serde_json::Value),
    prompt: &str,
    extra_env: impl IntoIterator<Item = (String, String)>,
    model: Option<&str>,
) {
    let fixture = add_dirs_fixture();
    let mut env = base_env();
    env.insert("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string());
    env.insert("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string());
    env.extend(add_dir_expectations(&fixture.dirs));
    env.extend(extra_env);
    if let Some(model) = model {
        env.extend(model_expectations(model));
    }

    let backend = build_backend(env, model, false);
    let handle = backend
        .run(run_request(
            prompt,
            [add_dirs_extension(&fixture.dirs), resume_extension],
        ))
        .await
        .unwrap();

    let mut events = handle.events;
    let _seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert_completion_success(handle.completion).await;
}

#[tokio::test]
async fn resume_last_preserves_add_dir_flags_in_order() {
    assert_resume_add_dirs_case(
        "resume_last_assert",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        ),
        "hello world",
        std::iter::empty(),
        None,
    )
    .await;
}

#[tokio::test]
async fn resume_id_preserves_add_dir_flags_in_order() {
    let resume_id = "thread-123";
    assert_resume_add_dirs_case(
        "resume_id_assert",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "id", "id": resume_id}),
        ),
        "hello world",
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
        None,
    )
    .await;
}

#[tokio::test]
async fn resume_last_with_model_preserves_add_dir_flags_in_order() {
    assert_resume_add_dirs_case(
        "resume_last_assert",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        ),
        "hello world",
        std::iter::empty(),
        Some("gpt-5-codex"),
    )
    .await;
}

#[tokio::test]
async fn resume_id_with_model_preserves_add_dir_flags_in_order() {
    let resume_id = "thread-123";
    assert_resume_add_dirs_case(
        "resume_id_assert",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "id", "id": resume_id}),
        ),
        "hello world",
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
        Some("gpt-5-codex"),
    )
    .await;
}
