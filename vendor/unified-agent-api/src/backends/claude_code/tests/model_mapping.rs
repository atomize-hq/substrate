use std::{collections::BTreeMap, time::Duration};

use futures_util::StreamExt;
use tempfile::tempdir;

use super::super::super::session_selectors::SessionSelectorV1;
use super::support::*;

async fn spawn_and_drain(
    model_id: Option<String>,
    policy: super::super::harness::ClaudeExecPolicy,
    env: BTreeMap<String, String>,
    prompt: &str,
) {
    let _env_lock = test_env_lock();

    let adapter = new_adapter_with_config(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        ..Default::default()
    });

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: prompt.to_string(),
            model_id,
            working_dir: None,
            effective_timeout: None,
            env,
            policy,
        })
        .await
        .expect("spawn succeeds");

    let (backend_events, completion) = tokio::time::timeout(Duration::from_secs(2), async move {
        let events_fut = async move {
            spawned
                .events
                .map(|result| result.expect("backend event stream is infallible for fake Claude"))
                .collect::<Vec<_>>()
                .await
        };
        let completion_fut = async move {
            spawned
                .completion
                .await
                .expect("completion is Ok for fake Claude")
        };
        tokio::join!(events_fut, completion_fut)
    })
    .await
    .expect("spawned events and completion resolve");
    let _events = backend_events;

    assert!(
        completion.status.success(),
        "expected successful fake Claude run, completion: {completion:?}"
    );
}

fn base_env(scenario: &str, prompt: &str, expect_model: Option<&str>) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([
        ("FAKE_CLAUDE_SCENARIO".to_string(), scenario.to_string()),
        ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
        (
            "FAKE_CLAUDE_EXPECT_NO_FALLBACK_MODEL".to_string(),
            "true".to_string(),
        ),
    ]);
    if let Some(model) = expect_model {
        env.insert("FAKE_CLAUDE_EXPECT_MODEL".to_string(), model.to_string());
    } else {
        env.insert(
            "FAKE_CLAUDE_EXPECT_NO_MODEL".to_string(),
            "true".to_string(),
        );
    }
    env
}

#[tokio::test]
async fn claude_model_id_is_mapped_to_print_request_model_flag() {
    let prompt = "hello world";
    let env = base_env("fresh_assert", prompt, Some("request-model"));

    spawn_and_drain(
        Some("request-model".to_string()),
        super::super::harness::ClaudeExecPolicy {
            non_interactive: true,
            external_sandbox: false,
            resume: None,
            fork: None,
            resolved_working_dir: None,
            add_dirs: Vec::new(),
        },
        env,
        prompt,
    )
    .await;
}

#[tokio::test]
async fn claude_absent_model_id_emits_no_model_flag() {
    let prompt = "hello world";
    let env = base_env("fresh_assert", prompt, None);

    spawn_and_drain(
        None,
        super::super::harness::ClaudeExecPolicy {
            non_interactive: true,
            external_sandbox: false,
            resume: None,
            fork: None,
            resolved_working_dir: None,
            add_dirs: Vec::new(),
        },
        env,
        prompt,
    )
    .await;
}

#[tokio::test]
async fn claude_fresh_run_model_orders_before_add_dir_group() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let add_dir = temp.path().join("context");
    std::fs::create_dir_all(&add_dir).expect("create add-dir");

    let env = base_env("fresh_assert", prompt, Some("request-model"))
        .into_iter()
        .chain(expected_add_dirs_env(std::slice::from_ref(&add_dir)))
        .collect::<BTreeMap<_, _>>();

    spawn_and_drain(
        Some("request-model".to_string()),
        super::super::harness::ClaudeExecPolicy {
            non_interactive: true,
            external_sandbox: false,
            resume: None,
            fork: None,
            resolved_working_dir: None,
            add_dirs: vec![add_dir],
        },
        env,
        prompt,
    )
    .await;
}

#[tokio::test]
async fn claude_resume_last_emits_model_before_continue() {
    let prompt = "hello world";
    let env = base_env("resume_last_assert", prompt, Some("request-model"));

    spawn_and_drain(
        Some("request-model".to_string()),
        super::super::harness::ClaudeExecPolicy {
            non_interactive: true,
            external_sandbox: false,
            resume: Some(SessionSelectorV1::Last),
            fork: None,
            resolved_working_dir: None,
            add_dirs: Vec::new(),
        },
        env,
        prompt,
    )
    .await;
}

#[tokio::test]
async fn claude_resume_id_emits_model_before_resume_flag() {
    let prompt = "hello world";
    let resume_id = "sess-123";
    let env = base_env("resume_id_assert", prompt, Some("request-model"))
        .into_iter()
        .chain([(
            "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )])
        .collect::<BTreeMap<_, _>>();

    spawn_and_drain(
        Some("request-model".to_string()),
        super::super::harness::ClaudeExecPolicy {
            non_interactive: true,
            external_sandbox: false,
            resume: Some(SessionSelectorV1::Id {
                id: resume_id.to_string(),
            }),
            fork: None,
            resolved_working_dir: None,
            add_dirs: Vec::new(),
        },
        env,
        prompt,
    )
    .await;
}

#[tokio::test]
async fn claude_fork_last_emits_model_before_continue_and_fork_session() {
    let prompt = "hello world";
    let env = base_env("fork_last_assert", prompt, Some("request-model"));

    spawn_and_drain(
        Some("request-model".to_string()),
        super::super::harness::ClaudeExecPolicy {
            non_interactive: true,
            external_sandbox: false,
            resume: None,
            fork: Some(SessionSelectorV1::Last),
            resolved_working_dir: None,
            add_dirs: Vec::new(),
        },
        env,
        prompt,
    )
    .await;
}

#[tokio::test]
async fn claude_fork_id_emits_model_before_fork_session_and_resume_flag() {
    let prompt = "hello world";
    let fork_id = "sess-123";
    let env = base_env("fork_id_assert", prompt, Some("request-model"))
        .into_iter()
        .chain([(
            "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
            fork_id.to_string(),
        )])
        .collect::<BTreeMap<_, _>>();

    spawn_and_drain(
        Some("request-model".to_string()),
        super::super::harness::ClaudeExecPolicy {
            non_interactive: true,
            external_sandbox: false,
            resume: None,
            fork: Some(SessionSelectorV1::Id {
                id: fork_id.to_string(),
            }),
            resolved_working_dir: None,
            add_dirs: Vec::new(),
        },
        env,
        prompt,
    )
    .await;
}
