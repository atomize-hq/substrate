use agent_api::AgentWrapperBackend;
use std::collections::BTreeMap;

use serde_json::json;

use super::support::{
    assert_completion_success, assert_external_sandbox_warning_before_session_handle_facet,
    build_backend, drain_to_none, run_request, STREAM_TIMEOUT,
};

async fn assert_external_sandbox_resume_case(
    scenario: &str,
    resume_extension: (String, serde_json::Value),
    extra_env: impl IntoIterator<Item = (String, String)>,
    prompt: &str,
) {
    let env: BTreeMap<String, String> = [
        ("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string()),
        ("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string()),
        (
            "FAKE_CODEX_EXPECT_DANGEROUS_BYPASS".to_string(),
            "1".to_string(),
        ),
    ]
    .into_iter()
    .chain(extra_env)
    .collect();

    let backend = build_backend(env, None, true);
    let handle = backend
        .run(run_request(
            prompt,
            [
                (
                    "agent_api.exec.external_sandbox.v1".to_string(),
                    json!(true),
                ),
                resume_extension,
            ],
        ))
        .await
        .unwrap();

    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert_external_sandbox_warning_before_session_handle_facet(&seen);
    assert_completion_success(handle.completion).await;
}

#[tokio::test]
async fn codex_resume_last_external_sandbox_maps_to_dangerous_bypass_argv_and_emits_warning_before_handle_facet(
) {
    assert_external_sandbox_resume_case(
        "resume_last_assert",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        ),
        std::iter::empty(),
        "hello world",
    )
    .await;
}

#[tokio::test]
async fn codex_resume_id_external_sandbox_maps_to_dangerous_bypass_argv_and_emits_warning_before_handle_facet(
) {
    let resume_id = "thread-123";
    assert_external_sandbox_resume_case(
        "resume_id_assert",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "id", "id": resume_id}),
        ),
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
        "hello world",
    )
    .await;
}
