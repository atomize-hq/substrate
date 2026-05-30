use anyhow as _;
use blake3 as _;
use std::fs;

use agent_session_compactor::{compact_codex_sessions, RunConfig};
use camino::Utf8Path;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time::macros::datetime;
use time as _;
use walkdir as _;

#[test]
fn end_to_end_compaction_produces_stable_bundle_outputs() {
    let codex_home = seeded_codex_home();
    let output_dir = TempDir::new().expect("output temp dir");
    let output_path = Utf8Path::from_path(output_dir.path()).expect("utf8 output path");

    let config = RunConfig {
        codex_home: Some(
            Utf8Path::from_path(codex_home.path())
                .expect("utf8 codex home")
                .to_owned(),
        ),
        session_id: None,
        output_dir: output_path.to_owned(),
        generated_at: Some(datetime!(2026-05-29 12:00:00 UTC)),
    };

    let first = compact_codex_sessions(&config).expect("first compaction");
    let first_manifest = fs::read_to_string(output_path.join("manifest.json")).expect("manifest");
    let first_archival =
        fs::read_to_string(output_path.join("rows.archival.jsonl")).expect("archival rows");
    let first_compact =
        fs::read_to_string(output_path.join("rows.compact.jsonl")).expect("compact rows");
    let first_audit =
        fs::read_to_string(output_path.join("dedupe-audit.jsonl")).expect("dedupe audit");
    let first_summary = fs::read_to_string(output_path.join("summary.md")).expect("summary");

    let second = compact_codex_sessions(&config).expect("second compaction");

    assert_eq!(first.manifest, second.manifest);
    assert_eq!(
        first_manifest,
        fs::read_to_string(output_path.join("manifest.json")).expect("manifest after rerun")
    );
    assert_eq!(
        first_archival,
        fs::read_to_string(output_path.join("rows.archival.jsonl"))
            .expect("archival rows after rerun")
    );
    assert_eq!(
        first_compact,
        fs::read_to_string(output_path.join("rows.compact.jsonl"))
            .expect("compact rows after rerun")
    );
    assert_eq!(
        first_audit,
        fs::read_to_string(output_path.join("dedupe-audit.jsonl"))
            .expect("dedupe audit after rerun")
    );
    assert_eq!(
        first_summary,
        fs::read_to_string(output_path.join("summary.md")).expect("summary after rerun")
    );

    assert!(first.manifest.archival_row_count >= first.manifest.compact_row_count);
    assert_eq!(first.manifest.session_ids, vec!["session-123".to_string()]);
}

fn seeded_codex_home() -> TempDir {
    let temp_dir = TempDir::new().expect("codex home temp dir");
    let rollout_dir = temp_dir.path().join("sessions/2026/05/29");
    fs::create_dir_all(&rollout_dir).expect("create rollout dir");
    fs::write(
        rollout_dir.join("rollout-session-123.jsonl"),
        concat!(
            "{\"timestamp\":\"2026-05-29T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-123\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:01Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-abc\",\"user_instructions\":\"Repo-local rules\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:02Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"Ship the packet\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Packet complete\"}]}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:04Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Packet complete\"}]}}\n"
        ),
    )
    .expect("write rollout fixture");
    temp_dir
}
