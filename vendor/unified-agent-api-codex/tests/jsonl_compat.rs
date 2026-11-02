use codex::{
    CodexClient, ExecStreamError, ExecStreamRequest, ItemPayload, ResumeRequest, ThreadEvent,
};
use futures_util::StreamExt;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const V0_61_0_STREAMING: &str =
    include_str!("../examples/fixtures/versioned/0.61.0/streaming.jsonl");
const V0_61_0_RESUME: &str = include_str!("../examples/fixtures/versioned/0.61.0/resume.jsonl");
const V0_61_0_MALFORMED: &str =
    include_str!("../examples/fixtures/versioned/0.61.0/malformed.jsonl");

const V0_77_0_STREAMING: &str =
    include_str!("../examples/fixtures/versioned/0.77.0/streaming.jsonl");
const V0_77_0_RESUME: &str = include_str!("../examples/fixtures/versioned/0.77.0/resume.jsonl");
const V0_77_0_MALFORMED: &str =
    include_str!("../examples/fixtures/versioned/0.77.0/malformed.jsonl");

#[cfg(unix)]
fn write_executable(path: &Path, contents: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, contents)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(unix)]
fn write_fake_codex_binary(root: &Path, exec_jsonl: &str, resume_jsonl: &str) -> PathBuf {
    let exec_path = root.join("exec.jsonl");
    let resume_path = root.join("resume.jsonl");
    fs::write(&exec_path, exec_jsonl).expect("write exec fixture");
    fs::write(&resume_path, resume_jsonl).expect("write resume fixture");

    let script_path = root.join("codex");
    let script = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

	is_exec=0
	is_resume=0
	for arg in "$@"; do
	  if [[ "$arg" == "exec" ]]; then
	    is_exec=1
	  elif [[ "$arg" == "resume" ]]; then
	    is_resume=1
	  fi
	done

	if [[ "$is_resume" == "1" ]]; then
	  cat >/dev/null || true
	  cat "{resume}"
	  exit 0
	fi

	if [[ "$is_exec" == "1" ]]; then
	  cat >/dev/null || true
	  cat "{exec}"
	  exit 0
	fi

	echo "unexpected args: $*" >&2
	exit 2
	"#,
        exec = exec_path.display(),
        resume = resume_path.display()
    );
    write_executable(&script_path, &script).expect("write fake codex script");
    script_path
}

#[cfg(unix)]
async fn collect_exec_events(
    exec_jsonl: &str,
    resume_jsonl: &str,
) -> Vec<Result<ThreadEvent, ExecStreamError>> {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let binary = write_fake_codex_binary(root, exec_jsonl, resume_jsonl);

    let codex_home = root.join("codex-home");
    let cwd = root.join("cwd");
    fs::create_dir_all(&cwd).expect("create cwd");

    let client = CodexClient::builder()
        .binary(&binary)
        .codex_home(&codex_home)
        .cd(&cwd)
        .json(true)
        .mirror_stdout(false)
        .quiet(true)
        .timeout(Duration::from_secs(5))
        .build();

    let mut stream = client
        .stream_exec(ExecStreamRequest {
            prompt: "fixture prompt".to_string(),
            ephemeral: false,
            ignore_rules: false,
            ignore_user_config: false,
            idle_timeout: None,
            output_last_message: None,
            output_schema: None,
            json_event_log: None,
        })
        .await
        .expect("start exec stream");

    let mut events = Vec::new();
    while let Some(event) = stream.events.next().await {
        events.push(event);
    }
    stream.completion.await.expect("exec completion");
    events
}

#[cfg(unix)]
async fn collect_resume_events(
    exec_jsonl: &str,
    resume_jsonl: &str,
) -> Vec<Result<ThreadEvent, ExecStreamError>> {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();
    let binary = write_fake_codex_binary(root, exec_jsonl, resume_jsonl);

    let codex_home = root.join("codex-home");
    let cwd = root.join("cwd");
    fs::create_dir_all(&cwd).expect("create cwd");

    let client = CodexClient::builder()
        .binary(&binary)
        .codex_home(&codex_home)
        .cd(&cwd)
        .json(true)
        .mirror_stdout(false)
        .quiet(true)
        .timeout(Duration::from_secs(5))
        .build();

    let mut stream = client
        .stream_resume(ResumeRequest::last())
        .await
        .expect("start resume stream");

    let mut events = Vec::new();
    while let Some(event) = stream.events.next().await {
        events.push(event);
    }
    stream.completion.await.expect("resume completion");
    events
}

fn has_item_event(events: &[Result<ThreadEvent, ExecStreamError>]) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            Ok(ThreadEvent::ItemStarted(_))
                | Ok(ThreadEvent::ItemDelta(_))
                | Ok(ThreadEvent::ItemCompleted(_))
                | Ok(ThreadEvent::ItemFailed(_))
        )
    })
}

fn first_thread_started_extra(
    events: &[Result<ThreadEvent, ExecStreamError>],
) -> Option<BTreeMap<String, Value>> {
    events.iter().find_map(|event| match event {
        Ok(ThreadEvent::ThreadStarted(started)) => Some(started.extra.clone()),
        _ => None,
    })
}

#[cfg(unix)]
#[tokio::test]
async fn parses_versioned_exec_fixtures() {
    for (version, exec_fixture, resume_fixture) in [
        ("0.61.0", V0_61_0_STREAMING, V0_61_0_RESUME),
        ("0.77.0", V0_77_0_STREAMING, V0_77_0_RESUME),
    ] {
        let events = collect_exec_events(exec_fixture, resume_fixture).await;
        let errors: Vec<&ExecStreamError> = events
            .iter()
            .filter_map(|event| event.as_ref().err())
            .collect();
        assert!(
            errors.is_empty(),
            "expected {version} exec fixture to parse without errors; got {errors:?}"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Ok(ThreadEvent::ThreadStarted(_)))),
            "expected {version} exec fixture to include thread.started"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Ok(ThreadEvent::TurnStarted(_)))),
            "expected {version} exec fixture to include turn.started"
        );
        assert!(
            has_item_event(&events),
            "expected {version} exec fixture to include item events"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn parses_versioned_resume_fixtures() {
    for (version, exec_fixture, resume_fixture) in [
        ("0.61.0", V0_61_0_STREAMING, V0_61_0_RESUME),
        ("0.77.0", V0_77_0_STREAMING, V0_77_0_RESUME),
    ] {
        let events = collect_resume_events(exec_fixture, resume_fixture).await;
        let errors: Vec<&ExecStreamError> = events
            .iter()
            .filter_map(|event| event.as_ref().err())
            .collect();
        assert!(
            errors.is_empty(),
            "expected {version} resume fixture to parse without errors; got {errors:?}"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Ok(ThreadEvent::ThreadStarted(_)))),
            "expected {version} resume fixture to include normalized thread.started"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, Ok(ThreadEvent::TurnStarted(_)))),
            "expected {version} resume fixture to include turn.started"
        );
        assert!(
            has_item_event(&events),
            "expected {version} resume fixture to include item events"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn retains_unknown_fields_in_extra_maps() {
    let events = collect_exec_events(V0_77_0_STREAMING, V0_77_0_RESUME).await;
    let extra = first_thread_started_extra(&events).expect("thread.started present");
    assert_eq!(
        extra.get("future_flag"),
        Some(&Value::from(1)),
        "thread.started should preserve unknown fields"
    );

    let item_extra = events.iter().find_map(|event| match event {
        Ok(ThreadEvent::ItemStarted(envelope)) => Some(envelope.item.extra.clone()),
        Ok(ThreadEvent::ItemCompleted(envelope)) => Some(envelope.item.extra.clone()),
        _ => None,
    });
    let item_extra = item_extra.expect("item event present");
    assert!(
        item_extra.contains_key("new_top") || item_extra.contains_key("extra_meta"),
        "item events should preserve unknown fields in extra maps"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn malformed_lines_are_non_fatal() {
    for (version, malformed_fixture, expected_thread_id) in [
        ("0.61.0", V0_61_0_MALFORMED, "t061-malformed"),
        ("0.77.0", V0_77_0_MALFORMED, "t077-malformed"),
    ] {
        let events = collect_exec_events(malformed_fixture, V0_77_0_RESUME).await;

        let first_is_error = matches!(events.first(), Some(Err(_)))
            || events
                .iter()
                .any(|event| matches!(event, Ok(ThreadEvent::Error(_))));
        assert!(
            first_is_error,
            "expected {version} malformed fixture to surface an error for invalid JSON"
        );

        let valid_index = events.iter().position(|event| {
            matches!(
                event,
                Ok(ThreadEvent::ThreadStarted(started)) if started.thread_id == expected_thread_id
            )
        });
        assert!(
            valid_index.is_some_and(|idx| idx > 0),
            "expected {version} stream to continue after malformed line and parse subsequent events"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn known_good_fixtures_include_text_payloads() {
    let events = collect_exec_events(V0_61_0_STREAMING, V0_61_0_RESUME).await;
    let first_agent_message = events.iter().find_map(|event| match event {
        Ok(ThreadEvent::ItemStarted(envelope)) => match &envelope.item.payload {
            ItemPayload::AgentMessage(content) => Some(content.text.clone()),
            _ => None,
        },
        Ok(ThreadEvent::ItemCompleted(envelope)) => match &envelope.item.payload {
            ItemPayload::AgentMessage(content) => Some(content.text.clone()),
            _ => None,
        },
        _ => None,
    });
    assert_eq!(
        first_agent_message.as_deref(),
        Some("hello from 0.61.0"),
        "legacy fixtures should normalize string content into typed text payloads"
    );
}
