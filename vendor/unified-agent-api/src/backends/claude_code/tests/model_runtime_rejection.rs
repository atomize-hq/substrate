use std::{collections::BTreeMap, time::Duration};

use futures_util::StreamExt;

use super::support::*;

#[tokio::test]
async fn claude_runtime_model_rejection_is_safely_redacted_and_parity_is_preserved() {
    let _env_lock = test_env_lock();

    let prompt = "hello world";
    let requested_model = "request-model";
    let secret = "MODEL_RUNTIME_REJECTION_SECRET_DO_NOT_LEAK";

    let env = BTreeMap::from([
        (
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "model_runtime_rejection_after_init".to_string(),
        ),
        ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string()),
        (
            "FAKE_CLAUDE_EXPECT_MODEL".to_string(),
            requested_model.to_string(),
        ),
        (
            "FAKE_CLAUDE_EXPECT_NO_FALLBACK_MODEL".to_string(),
            "true".to_string(),
        ),
        (
            "FAKE_CLAUDE_MODEL_RUNTIME_REJECTION_SECRET".to_string(),
            secret.to_string(),
        ),
    ]);

    let adapter = new_adapter_with_config(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        ..Default::default()
    });

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: prompt.to_string(),
            model_id: Some(requested_model.to_string()),
            working_dir: None,
            effective_timeout: None,
            env,
            policy: super::super::harness::ClaudeExecPolicy {
                non_interactive: true,
                external_sandbox: false,
                resume: None,
                fork: None,
                resolved_working_dir: None,
                add_dirs: Vec::new(),
            },
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

    let mapped_events: Vec<_> = backend_events
        .into_iter()
        .flat_map(|event| adapter.map_event(event))
        .collect();

    let error_messages: Vec<_> = mapped_events
        .iter()
        .filter(|event| event.kind == crate::AgentWrapperEventKind::Error)
        .filter_map(|event| event.message.as_deref())
        .collect();

    assert_eq!(error_messages.len(), 1, "events: {mapped_events:?}");
    assert_eq!(
        error_messages[0],
        super::super::util::PINNED_MODEL_RUNTIME_REJECTION
    );
    assert!(!error_messages[0].contains(secret));
    assert!(!error_messages[0].contains(requested_model));

    for event in &mapped_events {
        let Some(message) = event.message.as_deref() else {
            continue;
        };
        assert!(
            !message.contains(secret),
            "leaked secret in event: {event:?}"
        );
        assert!(
            !message.contains(requested_model),
            "leaked model id in event: {event:?}"
        );
    }

    let err = adapter
        .map_completion(completion)
        .expect_err("runtime rejection must map to Backend error");
    match err {
        crate::AgentWrapperError::Backend { message } => {
            assert_eq!(message, super::super::util::PINNED_MODEL_RUNTIME_REJECTION);
            assert!(!message.contains(secret));
            assert!(!message.contains(requested_model));
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}
