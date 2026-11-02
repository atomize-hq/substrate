use super::*;

use std::time::Duration;

use futures_util::StreamExt;

#[cfg(unix)]
#[tokio::test]
async fn stream_resume_timeout_closes_events_without_polling_completion() {
    let temp = tempfile::tempdir().unwrap();
    let script_path = write_fake_codex(
        temp.path(),
        r#"#!/usr/bin/env bash
set -euo pipefail

echo '{"type":"thread.started","thread_id":"t"}'
exec sleep 1000000
"#,
    );

    let client = CodexClient::builder()
        .binary(&script_path)
        .timeout(Duration::from_secs(1))
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let stream = client.stream_resume(ResumeRequest::last()).await.unwrap();
    let mut events = stream.events;
    let completion = stream.completion;

    let first = tokio::time::timeout(Duration::from_secs(2), events.next())
        .await
        .unwrap();
    assert!(
        matches!(first, Some(Ok(ThreadEvent::ThreadStarted(_)))),
        "expected thread.started, got: {first:?}"
    );

    let closed = tokio::time::timeout(Duration::from_secs(2), events.next()).await;
    match closed {
        Ok(None) => {}
        Ok(Some(item)) => {
            drop(completion);
            panic!("expected events stream to close, got: {item:?}");
        }
        Err(_) => {
            drop(completion);
            panic!("timed out waiting for events stream to close after timeout");
        }
    }

    let result = completion.await;
    assert!(
        matches!(
            result,
            Err(ExecStreamError::Codex(CodexError::Timeout { .. }))
        ),
        "expected completion to fail with timeout, got: {result:?}"
    );
}
