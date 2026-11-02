use super::*;

use std::{collections::BTreeMap, time::Duration};

use futures_util::StreamExt;

#[cfg(unix)]
#[tokio::test]
async fn stream_resume_termination_closes_events_without_polling_completion() {
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
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let env_overrides = BTreeMap::new();
    let ExecStreamControl {
        mut events,
        completion,
        termination,
    } = client
        .stream_resume_with_env_overrides_control(
            ResumeRequest::last().prompt("hello"),
            &env_overrides,
        )
        .await
        .unwrap();

    let first = tokio::time::timeout(Duration::from_secs(2), events.next())
        .await
        .unwrap();
    assert!(
        matches!(first, Some(Ok(ThreadEvent::ThreadStarted(_)))),
        "expected thread.started, got: {first:?}"
    );

    termination.request_termination();

    let closed = tokio::time::timeout(Duration::from_secs(2), events.next()).await;
    match closed {
        Ok(None) => {}
        Ok(Some(item)) => panic!("expected events stream to close, got: {item:?}"),
        Err(_) => {
            drop(completion);
            panic!("timed out waiting for events stream to close after termination");
        }
    }
}
