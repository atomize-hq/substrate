use codex::{
    thread_event_jsonl_file, thread_event_jsonl_reader, ExecStreamError, JsonlThreadEventParser,
    ThreadEvent, ThreadEventJsonlRecord,
};
use serde_json::Value;
mod support_paths;
use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

const VERSIONS: &[&str] = &["0.61.0", "0.77.0"];

fn fixture_path(version: &str, name: &str) -> PathBuf {
    support_paths::codex_examples_dir()
        .join("fixtures")
        .join("versioned")
        .join(version)
        .join(name)
}

fn collect_file_records(path: &Path) -> Vec<ThreadEventJsonlRecord> {
    let reader = thread_event_jsonl_file(path).unwrap_or_else(|err| {
        panic!(
            "expected thread_event_jsonl_file({}) to succeed; got {err:?}",
            path.display()
        )
    });
    reader.collect()
}

fn error_outcomes(records: &[ThreadEventJsonlRecord]) -> Vec<(usize, &ExecStreamError)> {
    records
        .iter()
        .filter_map(|record| {
            record
                .outcome
                .as_ref()
                .err()
                .map(|err| (record.line_number, err))
        })
        .collect()
}

fn records_have_item_event(records: &[ThreadEventJsonlRecord]) -> bool {
    records.iter().any(|record| {
        matches!(
            record.outcome.as_ref(),
            Ok(ThreadEvent::ItemStarted(_))
                | Ok(ThreadEvent::ItemDelta(_))
                | Ok(ThreadEvent::ItemCompleted(_))
                | Ok(ThreadEvent::ItemFailed(_))
        )
    })
}

#[test]
fn scenario_a_parses_streaming_fixtures_without_errors() {
    for version in VERSIONS {
        let path = fixture_path(version, "streaming.jsonl");
        let records = collect_file_records(&path);
        let errors = error_outcomes(&records);
        assert!(
            errors.is_empty(),
            "expected {version} streaming.jsonl to parse with zero error outcomes; got {errors:?}"
        );

        assert!(
            records
                .iter()
                .any(|record| matches!(record.outcome.as_ref(), Ok(ThreadEvent::ThreadStarted(_)))),
            "expected {version} streaming.jsonl to include thread.started (or normalized equivalent)"
        );
        assert!(
            records
                .iter()
                .any(|record| matches!(record.outcome.as_ref(), Ok(ThreadEvent::TurnStarted(_)))),
            "expected {version} streaming.jsonl to include turn.started"
        );
        assert!(
            records_have_item_event(&records),
            "expected {version} streaming.jsonl to include item.* events"
        );
    }
}

#[test]
fn scenario_b_parses_resume_fixtures_without_errors() {
    for version in VERSIONS {
        let path = fixture_path(version, "resume.jsonl");
        let records = collect_file_records(&path);
        let errors = error_outcomes(&records);
        assert!(
            errors.is_empty(),
            "expected {version} resume.jsonl to parse with zero error outcomes; got {errors:?}"
        );

        assert!(
            records
                .iter()
                .any(|record| matches!(record.outcome.as_ref(), Ok(ThreadEvent::ThreadStarted(_)))),
            "expected {version} resume.jsonl to include normalized ThreadEvent::ThreadStarted"
        );
        assert!(
            records
                .iter()
                .any(|record| matches!(record.outcome.as_ref(), Ok(ThreadEvent::TurnStarted(_)))),
            "expected {version} resume.jsonl to include turn.started"
        );
        assert!(
            records_have_item_event(&records),
            "expected {version} resume.jsonl to include item.* events"
        );
    }
}

#[test]
fn scenario_c_malformed_lines_yield_errors_and_continue() {
    for version in VERSIONS {
        let path = fixture_path(version, "malformed.jsonl");
        let records = collect_file_records(&path);

        let first_error_index = records.iter().position(|record| record.outcome.is_err());
        assert!(
            first_error_index.is_some(),
            "expected {version} malformed.jsonl to include at least one error outcome"
        );
        let first_error_index = first_error_index.unwrap();
        assert!(
            first_error_index + 1 < records.len(),
            "expected {version} malformed.jsonl to continue emitting records after the first error"
        );

        let has_thread_started_after_error = records[first_error_index + 1..]
            .iter()
            .any(|record| matches!(record.outcome.as_ref(), Ok(ThreadEvent::ThreadStarted(_))));
        assert!(
            has_thread_started_after_error,
            "expected {version} malformed.jsonl to include a successful ThreadEvent::ThreadStarted after a malformed line"
        );

        let has_turn_started_after_error = records[first_error_index + 1..]
            .iter()
            .any(|record| matches!(record.outcome.as_ref(), Ok(ThreadEvent::TurnStarted(_))));
        assert!(
            has_turn_started_after_error,
            "expected {version} malformed.jsonl to include a successful turn.started after a malformed line"
        );
    }
}

fn find_fixture_line(
    lines: &[String],
    predicate: impl Fn(&Value) -> bool,
) -> Option<(usize, String)> {
    for (index, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if predicate(&value) {
            return Some((index, line.clone()));
        }
    }
    None
}

fn read_fixture_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    reader.lines().collect()
}

#[test]
fn scenario_e_crlf_tolerance_trailing_carriage_return() {
    for version in VERSIONS {
        let streaming_path = fixture_path(version, "streaming.jsonl");
        let lines = read_fixture_lines(&streaming_path).expect("read fixture");

        let (_thread_index, thread_line) = find_fixture_line(&lines, |value| {
            value
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|t| t == "thread.started")
        })
        .unwrap_or_else(|| {
            panic!("expected {version} streaming fixture to include a thread.started line")
        });

        let (item_index, item_line) = find_fixture_line(&lines, |value| {
            value
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|t| t.starts_with("item."))
        })
        .unwrap_or_else(|| {
            panic!("expected {version} streaming fixture to include an item.* line")
        });

        let mut parser = JsonlThreadEventParser::new();
        let event = parser
            .parse_line(&thread_line)
            .expect("parse thread.started")
            .expect("thread.started yields an event");
        let mut parser_crlf = JsonlThreadEventParser::new();
        let event_crlf = parser_crlf
            .parse_line(&format!("{thread_line}\r"))
            .expect("parse thread.started with \\r")
            .expect("thread.started yields an event");
        assert_eq!(
            serde_json::to_value(event).unwrap(),
            serde_json::to_value(event_crlf).unwrap(),
            "expected {version} thread.started + \\r to parse equivalently"
        );

        let mut parser = JsonlThreadEventParser::new();
        let mut parser_crlf = JsonlThreadEventParser::new();
        for line in &lines[..item_index] {
            let _ = parser.parse_line(line).expect("parse fixture prefix");
            let _ = parser_crlf.parse_line(line).expect("parse fixture prefix");
        }

        let event = parser
            .parse_line(&item_line)
            .expect("parse item.*")
            .expect("item.* yields an event");
        let event_crlf = parser_crlf
            .parse_line(&format!("{item_line}\r"))
            .expect("parse item.* with \\r")
            .expect("item.* yields an event");
        assert_eq!(
            serde_json::to_value(event).unwrap(),
            serde_json::to_value(event_crlf).unwrap(),
            "expected {version} item.* + \\r to parse equivalently"
        );
    }
}

#[test]
fn scenario_f_unknown_type_yields_error_and_continues() {
    let streaming_path = fixture_path("0.77.0", "streaming.jsonl");
    let lines = read_fixture_lines(&streaming_path).expect("read fixture");

    let (_thread_index, thread_line) = find_fixture_line(&lines, |value| {
        value
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|t| t == "thread.started")
    })
    .expect("fixture contains thread.started");

    let (_turn_index, turn_line) = find_fixture_line(&lines, |value| {
        value
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|t| t == "turn.started")
    })
    .expect("fixture contains turn.started");

    let unknown = r#"{"type":"some.new.event","thread_id":"t-unknown","turn_id":"turn-unknown"}"#;
    let input = format!("{thread_line}\n{unknown}\n{turn_line}\n");
    let records: Vec<ThreadEventJsonlRecord> =
        thread_event_jsonl_reader(io::Cursor::new(input)).collect();

    assert_eq!(records.len(), 3, "expected exactly three emitted records");
    assert!(
        matches!(
            records[0].outcome.as_ref(),
            Ok(ThreadEvent::ThreadStarted(_))
        ),
        "expected success for thread.started line"
    );
    assert!(
        records[1].outcome.is_err(),
        "expected error outcome for unknown type line"
    );
    assert!(
        matches!(records[2].outcome.as_ref(), Ok(ThreadEvent::TurnStarted(_))),
        "expected success for turn.started line"
    );
}

#[cfg(unix)]
fn write_executable(path: &Path, contents: &str) -> io::Result<()> {
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
async fn collect_streaming_events(
    exec_jsonl: &str,
    resume_jsonl: &str,
) -> Vec<Result<ThreadEvent, ExecStreamError>> {
    use codex::{CodexClient, ExecStreamRequest};
    use futures_util::StreamExt;
    use std::time::Duration;

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
#[tokio::test]
async fn scenario_d_offline_matches_streaming_normalization() {
    for version in VERSIONS {
        let streaming_path = fixture_path(version, "streaming.jsonl");
        let offline_records = collect_file_records(&streaming_path);
        let offline_values: Vec<Value> = offline_records
            .into_iter()
            .filter_map(|record| record.outcome.ok())
            .map(|event| serde_json::to_value(event).unwrap())
            .collect();

        let exec_fixture =
            fs::read_to_string(&streaming_path).expect("read streaming.jsonl fixture");
        let resume_fixture =
            fs::read_to_string(fixture_path(version, "resume.jsonl")).expect("read resume.jsonl");
        let streaming_events = collect_streaming_events(&exec_fixture, &resume_fixture).await;
        let streaming_values: Vec<Value> = streaming_events
            .into_iter()
            .filter_map(|event| event.ok())
            .map(|event| serde_json::to_value(event).unwrap())
            .collect();

        assert_eq!(
            offline_values, streaming_values,
            "expected offline API to match streaming normalization for {version} streaming.jsonl"
        );
    }
}
