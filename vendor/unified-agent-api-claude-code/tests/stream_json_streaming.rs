use std::time::Duration;

mod support_paths;

use claude_code::{ClaudeClient, ClaudePrintRequest, ClaudeStreamJsonEvent};
use futures_util::StreamExt;

fn make_fake_client(scenario: &str) -> ClaudeClient {
    ClaudeClient::builder()
        .binary(support_paths::target_debug_binary(
            "fake_claude_stream_json",
        ))
        .env("FAKE_CLAUDE_SCENARIO", scenario)
        .build()
}

fn make_fake_client_mirroring_stdout(scenario: &str) -> ClaudeClient {
    ClaudeClient::builder()
        .binary(support_paths::target_debug_binary(
            "fake_claude_stream_json",
        ))
        .env("FAKE_CLAUDE_SCENARIO", scenario)
        .mirror_stdout(true)
        .build()
}

#[tokio::test]
async fn print_stream_json_yields_events_incrementally_before_process_exit() {
    let client = make_fake_client("two_events_delayed");
    let request = ClaudePrintRequest::new("hello");
    let handle = client.print_stream_json(request).await.unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::select! {
        biased;
        res = &mut completion => panic!("completion resolved before first event: {res:?}"),
        item = events.next() => item.expect("stream open").expect("event parses"),
    };
    assert!(matches!(first, ClaudeStreamJsonEvent::SystemInit { .. }));

    let second = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .expect("expected second event")
        .expect("stream open")
        .expect("event parses");
    assert!(matches!(second, ClaudeStreamJsonEvent::UserMessage { .. }));

    completion.await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn print_stream_json_mirror_stdout_works_on_current_thread_runtime() {
    let client = make_fake_client_mirroring_stdout("two_events_delayed");
    let request = ClaudePrintRequest::new("hello");
    let handle = client.print_stream_json(request).await.unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::select! {
        biased;
        res = &mut completion => panic!("completion resolved before first event: {res:?}"),
        item = tokio::time::timeout(Duration::from_secs(1), events.next()) => {
            item.expect("timeout waiting for first event")
                .expect("stream open")
                .expect("event parses")
        }
    };
    assert!(matches!(first, ClaudeStreamJsonEvent::SystemInit { .. }));

    let second = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .expect("expected second event")
        .expect("stream open")
        .expect("event parses");
    assert!(matches!(second, ClaudeStreamJsonEvent::UserMessage { .. }));

    completion.await.unwrap();
}

#[tokio::test]
async fn print_stream_json_ignores_crlf_and_blank_lines() {
    let client = make_fake_client("crlf_blank_lines");
    let request = ClaudePrintRequest::new("hello");
    let handle = client.print_stream_json(request).await.unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let a = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let b = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(a, ClaudeStreamJsonEvent::SystemInit { .. }));
    assert!(matches!(b, ClaudeStreamJsonEvent::UserMessage { .. }));

    assert!(events.next().await.is_none(), "no extra events expected");
    completion.await.unwrap();
}

#[tokio::test]
async fn print_stream_json_redacts_parse_errors() {
    let client = make_fake_client("parse_error_redaction");
    let request = ClaudePrintRequest::new("hello");
    let handle = client.print_stream_json(request).await.unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let first = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .unwrap()
        .unwrap()
        .expect_err("expected parse error first");
    let secret = "VERY_SECRET_SHOULD_NOT_APPEAR";
    assert!(!first.message.contains(secret));
    assert!(!first.details.contains(secret));

    let second = tokio::time::timeout(Duration::from_secs(1), events.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(second, ClaudeStreamJsonEvent::SystemInit { .. }));

    assert!(events.next().await.is_none(), "no extra events expected");
    completion.await.unwrap();
}
