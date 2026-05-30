use anyhow as _;
use blake3 as _;
use std::fs;

use agent_session_compactor::discovery::DiscoveredSessionArtifact;
use agent_session_compactor::ingest::{ingest_rollout_artifacts, ingest_rollout_file};
use camino::Utf8Path;
use clap as _;
use codex::RolloutEvent;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time as _;
use walkdir as _;

#[test]
fn rollout_ingest_parses_records_and_captures_unknown_and_invalid_lines() {
    let temp_dir = TempDir::new().expect("temp dir");
    let rollout_path = temp_dir.path().join("rollout-session-123.jsonl");
    fs::write(
        &rollout_path,
        concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-123\",\"cli_version\":\"0.1.0\"}}\n",
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"hi\"}]}}\n",
            "{\"type\":\"mystery\",\"payload\":{\"x\":1}}\n",
            "{\"type\":\"response_item\",\"payload\":\n"
        ),
    )
    .expect("write rollout fixture");

    let rollout = ingest_rollout_file(
        Utf8Path::from_path(&rollout_path).expect("rollout path should be valid UTF-8"),
    )
    .expect("ingest rollout");

    assert_eq!(rollout.session_id.as_deref(), Some("session-123"));
    assert_eq!(rollout.records.len(), 3);
    assert_eq!(rollout.unknown_records.len(), 1);
    assert_eq!(rollout.parse_failures.len(), 1);
    assert_eq!(rollout.records[0].event_index, 0);
    assert_eq!(rollout.records[1].event_index, 1);
    assert_eq!(rollout.records[2].event_index, 2);
    assert_eq!(rollout.parse_failures[0].event_index, 3);
    assert_eq!(rollout.unknown_records[0].line_number, 3);
    assert!(rollout.parse_failures[0]
        .error
        .contains("failed to parse codex rollout JSONL"));

    match &rollout.records[1].event {
        RolloutEvent::ResponseItem(item) => {
            assert_eq!(item.payload.kind.as_deref(), Some("message"));
            assert_eq!(item.payload.role.as_deref(), Some("assistant"));
        }
        other => panic!("expected response item, got {other:?}"),
    }
}

#[test]
fn rollout_ingest_filters_to_rollout_jsonl_artifacts_and_preserves_file_order() {
    let temp_dir = seeded_artifact_root();
    let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path");

    let artifacts = vec![
        DiscoveredSessionArtifact {
            path: root.join("sessions/2026/05/29/notes.txt"),
        },
        DiscoveredSessionArtifact {
            path: root.join("sessions/2026/05/29/rollout-b.jsonl"),
        },
        DiscoveredSessionArtifact {
            path: root.join("sessions/2026/05/29/rollout-c.jsonl"),
        },
    ];

    let ingested = ingest_rollout_artifacts(&artifacts).expect("ingest rollout files");
    let paths: Vec<_> = ingested.into_iter().map(|file| file.source_file).collect();

    assert_eq!(
        paths,
        vec![
            root.join("sessions/2026/05/29/rollout-b.jsonl"),
            root.join("sessions/2026/05/29/rollout-c.jsonl"),
        ]
    );
}

fn seeded_artifact_root() -> TempDir {
    let temp_dir = TempDir::new().expect("temp dir");
    let base = temp_dir.path().join("sessions/2026/05/29");
    fs::create_dir_all(&base).expect("create session dir");
    fs::write(
        base.join("rollout-b.jsonl"),
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-b\"}}\n",
    )
    .expect("write rollout-b");
    fs::write(
        base.join("rollout-c.jsonl"),
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-c\"}}\n",
    )
    .expect("write rollout-c");
    fs::write(base.join("notes.txt"), "ignore me\n").expect("write notes");
    temp_dir
}
