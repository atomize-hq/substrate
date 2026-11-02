#![cfg(feature = "codex")]

use std::{
    collections::BTreeMap,
    path::PathBuf,
    pin::Pin,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use futures_core::Stream;

const FIRST_EVENT_TIMEOUT: Duration = Duration::from_secs(5);
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
const POST_EVENT_PENDING_TIMEOUT: Duration = Duration::from_millis(200);

fn fake_codex_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_codex_stream_json_agent_api"))
}

fn any_event_contains(events: &[AgentWrapperEvent], needle: &str) -> bool {
    events.iter().any(|ev| {
        ev.message
            .as_deref()
            .is_some_and(|message| message.contains(needle))
            || ev.text.as_deref().is_some_and(|text| text.contains(needle))
    })
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
            },
        }
    }

    out
}

fn unique_temp_path(stem: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push("agent_api");
    path.push("c2");
    path.push(format!("{stem}_{}_{}.txt", std::process::id(), nanos));
    path
}

#[tokio::test]
async fn events_are_observable_before_process_exit() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "live_two_events_long_delay".to_string(),
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
        .unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first_event = tokio::time::timeout(
        FIRST_EVENT_TIMEOUT,
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event arrives")
    .expect("stream yields an event");

    assert_eq!(
        first_event.kind,
        AgentWrapperEventKind::Status,
        "expected a live status event before process exit"
    );

    drop(events);

    assert!(
        tokio::time::timeout(POST_EVENT_PENDING_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "expected completion to remain pending after observing the first event"
    );

    let completion = tokio::time::timeout(COMPLETION_TIMEOUT, &mut completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn request_env_overrides_backend_env() {
    let dump_path = unique_temp_path("env_dump");
    let dump_path_str = dump_path.to_string_lossy().to_string();

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [
            (
                "FAKE_CODEX_SCENARIO".to_string(),
                "dump_env_then_exit".to_string(),
            ),
            ("CODEX_WRAPPER_TEST_DUMP_ENV".to_string(), dump_path_str),
            ("C2_TEST_KEY".to_string(), "config".to_string()),
            ("C2_ONLY_CONFIG".to_string(), "config-only".to_string()),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            env: [("C2_TEST_KEY".to_string(), "request".to_string())]
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
            ..Default::default()
        })
        .await
        .unwrap();

    drop(handle.events);

    let completion = tokio::time::timeout(COMPLETION_TIMEOUT, handle.completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());

    let dump = std::fs::read_to_string(&dump_path).expect("dump file exists");
    let mut vars = BTreeMap::<String, String>::new();
    for line in dump.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        vars.insert(key.to_string(), value.to_string());
    }

    assert_eq!(vars.get("C2_TEST_KEY").map(String::as_str), Some("request"));
    assert_eq!(
        vars.get("C2_ONLY_CONFIG").map(String::as_str),
        Some("config-only")
    );
}

#[tokio::test]
async fn redaction_does_not_leak_raw_jsonl_line() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "emit_normalize_error_with_rawline_secret".to_string(),
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
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), DRAIN_TIMEOUT).await;
    assert!(
        seen.iter()
            .any(|ev| ev.kind == AgentWrapperEventKind::Error),
        "expected at least one redacted Error event"
    );
    assert!(
        !any_event_contains(&seen, "RAWLINE_SECRET_DO_NOT_LEAK"),
        "expected redaction to avoid raw JSONL line content"
    );

    let completion = tokio::time::timeout(COMPLETION_TIMEOUT, completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}
