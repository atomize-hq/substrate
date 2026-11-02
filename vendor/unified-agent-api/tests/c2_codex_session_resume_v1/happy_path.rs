use agent_api::AgentWrapperBackend;
use serde_json::json;

use super::support::{
    assert_completion_success, base_env, build_backend, drain_to_none, run_request, STREAM_TIMEOUT,
};

#[tokio::test]
async fn resume_last_maps_to_exec_json_resume_last_dash_and_stdin_prompt() {
    let prompt = "hello world";
    let mut env = base_env();
    env.insert(
        "FAKE_CODEX_SCENARIO".to_string(),
        "resume_last_assert".to_string(),
    );
    env.insert("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string());

    let backend = build_backend(env, None, false);
    let handle = backend
        .run(run_request(
            prompt,
            [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "last"}),
            )],
        ))
        .await
        .unwrap();

    let mut events = handle.events;
    let _seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert_completion_success(handle.completion).await;
}

#[tokio::test]
async fn resume_id_maps_to_exec_json_resume_id_dash_and_stdin_prompt() {
    let prompt = "hello world";
    let resume_id = "thread-123";

    let mut env = base_env();
    env.insert(
        "FAKE_CODEX_SCENARIO".to_string(),
        "resume_id_assert".to_string(),
    );
    env.insert("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string());
    env.insert(
        "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
        resume_id.to_string(),
    );

    let backend = build_backend(env, None, false);
    let handle = backend
        .run(run_request(
            prompt,
            [(
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "id", "id": resume_id}),
            )],
        ))
        .await
        .unwrap();

    let mut events = handle.events;
    let _seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert_completion_success(handle.completion).await;
}
