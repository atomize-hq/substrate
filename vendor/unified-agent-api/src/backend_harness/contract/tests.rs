use std::collections::BTreeMap;

use futures_util::StreamExt;

use super::*;
use crate::{AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperRunRequest};

use super::super::test_support::{success_exit_status, ToyAdapter, ToyPolicy};

#[tokio::test]
async fn toy_adapter_success_smoke() {
    let adapter = ToyAdapter { fail_spawn: false };

    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    let policy = adapter
        .validate_and_extract_policy(&request)
        .expect("policy extraction succeeds");

    let req = NormalizedRequest {
        agent_kind: adapter.kind(),
        prompt: "hello".to_string(),
        model_id: None,
        working_dir: None,
        effective_timeout: None,
        env: BTreeMap::new(),
        policy,
    };

    let spawned = adapter.spawn(req).await.expect("spawn succeeds");

    let mut universal = Vec::<AgentWrapperEvent>::new();
    let mut events = spawned.events;
    while let Some(item) = events.next().await {
        let event = item.expect("toy stream yields Ok");
        universal.extend(adapter.map_event(event));
    }

    assert_eq!(universal.len(), 2);
    assert_eq!(universal[0].agent_kind.as_str(), "toy");
    assert_eq!(universal[0].kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(universal[0].text.as_deref(), Some("one"));
    assert_eq!(universal[1].agent_kind.as_str(), "toy");
    assert_eq!(universal[1].kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(universal[1].text.as_deref(), Some("two"));

    let completion = spawned.completion.await.expect("typed completion ok");
    let mapped = adapter
        .map_completion(completion)
        .expect("completion mapping ok");
    assert_eq!(mapped.status, success_exit_status());
    assert_eq!(mapped.final_text.as_deref(), Some("done"));
}

#[tokio::test]
async fn toy_adapter_spawn_failure_is_redacted() {
    let adapter = ToyAdapter { fail_spawn: true };
    let req = NormalizedRequest {
        agent_kind: adapter.kind(),
        prompt: "hello".to_string(),
        model_id: None,
        working_dir: None,
        effective_timeout: None,
        env: BTreeMap::new(),
        policy: ToyPolicy,
    };

    let err = match adapter.spawn(req).await {
        Ok(_) => panic!("spawn expected to fail"),
        Err(err) => err,
    };
    assert_eq!(err.secret, "SECRET_SPAWN");
    let redacted = adapter.redact_error(BackendHarnessErrorPhase::Spawn, &err);
    assert!(!redacted.contains("SECRET_SPAWN"));
    assert!(redacted.contains("spawn"));
}
