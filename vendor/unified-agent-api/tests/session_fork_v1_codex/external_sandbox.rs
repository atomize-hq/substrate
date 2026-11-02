//! Codex backend tests for `agent_api.exec.external_sandbox.v1` mapping during fork flows.
//!
//! Normative sources:
//! - `docs/specs/codex-external-sandbox-mapping-contract.md` (ES-C04)
//! - `docs/specs/unified-agent-api/extensions-spec.md` (pinned warning + ordering)

use std::time::Duration;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use serde_json::{json, Value};

use crate::support::{drain_to_none, fake_codex_app_server_binary};

const PINNED_EXTERNAL_SANDBOX_WARNING: &str =
    "DANGEROUS: external sandbox exec policy enabled (agent_api.exec.external_sandbox.v1=true)";

#[tokio::test]
async fn fork_id_external_sandbox_pins_jsonrpc_params_and_emits_warning_before_handle_facet() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        allow_external_sandbox_exec: true,
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_id_success".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_PROMPT".to_string(),
                prompt.to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_SOURCE_THREAD_ID".to_string(),
                source_thread_id.to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_THREAD_FORK_SANDBOX".to_string(),
                "danger-full-access".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [
                (
                    "agent_api.session.fork.v1".to_string(),
                    json!({"selector":"id","id": source_thread_id}),
                ),
                (
                    "agent_api.exec.external_sandbox.v1".to_string(),
                    json!(true),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("run");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let warnings: Vec<(usize, &str)> = seen
        .iter()
        .enumerate()
        .filter(|(_, ev)| ev.kind == AgentWrapperEventKind::Status)
        .filter_map(|(idx, ev)| {
            let message = ev.message.as_deref()?;
            (message == PINNED_EXTERNAL_SANDBOX_WARNING && ev.data.is_none())
                .then_some((idx, message))
        })
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "expected exactly one pinned external sandbox warning Status event"
    );
    let idx_warning = warnings[0].0;

    let handle_events: Vec<(usize, &Value)> = seen
        .iter()
        .enumerate()
        .filter(|(_, ev)| ev.kind == AgentWrapperEventKind::Status)
        .filter_map(|(idx, ev)| {
            let data = ev.data.as_ref()?;
            (data
                .get("schema")
                .and_then(Value::as_str)
                .is_some_and(|schema| schema == "agent_api.session.handle.v1"))
            .then_some((idx, data))
        })
        .collect();
    assert_eq!(
        handle_events.len(),
        1,
        "expected exactly one Status event with the session handle facet"
    );
    let idx_handle = handle_events[0].0;

    assert!(
        idx_warning < idx_handle,
        "expected external sandbox warning to be emitted before session handle facet Status event (warning_idx={idx_warning}, handle_idx={idx_handle})"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());
}
