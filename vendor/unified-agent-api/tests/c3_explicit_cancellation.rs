#![cfg(feature = "codex")]

use std::{path::PathBuf, pin::Pin, sync::Arc, time::Duration};

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperGateway,
    AgentWrapperKind, AgentWrapperRunRequest,
};
use futures_core::Stream;

const FIRST_EVENT_TIMEOUT: Duration = Duration::from_secs(1);
const CANCEL_TERMINATION_TIMEOUT: Duration = Duration::from_secs(3);
const DROP_COMPLETION_TIMEOUT: Duration = Duration::from_secs(3);
const STDERR_SECRET_SENTINEL: &str = "RAW-STDERR-SECRET-CANCEL";

fn fake_codex_binary() -> PathBuf {
    PathBuf::from(env!(
        "CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api"
    ))
}

fn any_event_contains(events: &[AgentWrapperEvent], needle: &str) -> bool {
    events.iter().any(|ev| {
        ev.message
            .as_deref()
            .is_some_and(|message| message.contains(needle))
            || ev.text.as_deref().is_some_and(|text| text.contains(needle))
            || ev
                .data
                .as_ref()
                .and_then(|data| serde_json::to_string(data).ok())
                .is_some_and(|data| data.contains(needle))
    })
}

async fn drain_to_none(
    mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>,
) -> Vec<AgentWrapperEvent> {
    let mut out = Vec::new();
    while let Some(ev) = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await {
        out.push(ev);
    }
    out
}

fn codex_gateway_block_until_killed() -> (AgentWrapperGateway, AgentWrapperKind) {
    let backend = Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "block_until_killed".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    }));

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).expect("register codex backend");

    let kind = AgentWrapperKind::new("codex").expect("valid agent kind");
    assert!(
        gateway
            .backend(&kind)
            .expect("codex backend registered")
            .capabilities()
            .contains("agent_api.control.cancel.v1"),
        "expected codex backend to advertise explicit cancellation"
    );

    (gateway, kind)
}

#[tokio::test]
async fn explicit_cancel_terminates_blocking_backend_and_completion_is_cancelled() {
    let (gateway, kind) = codex_gateway_block_until_killed();

    let run = gateway
        .run_control(
            &kind,
            AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                ..Default::default()
            },
        )
        .await
        .expect("run_control");

    let agent_api::AgentWrapperRunControl { handle, cancel } = run;
    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = handle;

    let first = tokio::time::timeout(
        FIRST_EVENT_TIMEOUT,
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event should arrive");
    let Some(first) = first else {
        panic!("events stream ended before the first event");
    };

    cancel.cancel();
    cancel.cancel();

    let mut seen = vec![first];

    let (rest, completion_outcome) = tokio::time::timeout(CANCEL_TERMINATION_TIMEOUT, async {
        tokio::join!(drain_to_none(events.as_mut()), completion)
    })
    .await
    .expect("cancellation should terminate the backend and close the event stream");

    seen.extend(rest);

    assert!(
        !any_event_contains(&seen, STDERR_SECRET_SENTINEL),
        "expected backend stderr to never leak into message/text/data"
    );

    match completion_outcome {
        Err(AgentWrapperError::Backend { ref message }) if message == "cancelled" => {}
        other => panic!("expected cancelled completion error, got {other:?}"),
    }
}

#[tokio::test]
async fn dropping_events_does_not_prevent_explicit_cancel() {
    let (gateway, kind) = codex_gateway_block_until_killed();

    let run = gateway
        .run_control(
            &kind,
            AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                ..Default::default()
            },
        )
        .await
        .expect("run_control");

    let agent_api::AgentWrapperRunControl { handle, cancel } = run;
    let agent_api::AgentWrapperRunHandle { events, completion } = handle;
    drop(events);
    tokio::task::yield_now().await;

    cancel.cancel();
    cancel.cancel();

    let completion_outcome = tokio::time::timeout(CANCEL_TERMINATION_TIMEOUT, completion)
        .await
        .expect("completion resolves after cancel");
    match completion_outcome {
        Err(AgentWrapperError::Backend { ref message }) if message == "cancelled" => {}
        other => panic!("expected cancelled completion error, got {other:?}"),
    }
}

#[tokio::test]
async fn dropping_events_does_not_deadlock_completion() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "many_events_then_exit".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .expect("run");

    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = handle;

    let first = tokio::time::timeout(
        FIRST_EVENT_TIMEOUT,
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event should arrive");
    let Some(_first) = first else {
        panic!("events stream ended before the first event");
    };

    drop(events);
    tokio::task::yield_now().await;

    let completion_outcome = tokio::time::timeout(DROP_COMPLETION_TIMEOUT, completion)
        .await
        .expect("completion should resolve after dropping events");
    let completion = completion_outcome.expect("completion resolves successfully");
    assert!(completion.status.success());
}
