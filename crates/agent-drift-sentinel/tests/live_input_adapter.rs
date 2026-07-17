#![allow(unused_crate_dependencies)]

use std::fs;

use camino::Utf8PathBuf;
use serde_json::Value;
use tempfile::TempDir;

use agent_drift_sentinel::{
    FixtureLiveCheckpointSource, LiveCheckpointSource, LiveInputError, TriggerClass,
};

#[test]
fn live_input_adapter_reads_append_only_fixture_stream_in_order() {
    let path = fixture_path("append_only_stream.jsonl");
    let mut source =
        FixtureLiveCheckpointSource::from_path(&path).expect("load append-only live fixture");
    let mut triggers = Vec::new();
    let mut ordinals = Vec::new();

    while let Some(event) = source.next_event().expect("advance live fixture source") {
        triggers.push(event.trigger);
        ordinals.push(event.emission_ordinal);
    }

    assert_eq!(
        triggers,
        vec![
            TriggerClass::CheckpointReady,
            TriggerClass::Heartbeat,
            TriggerClass::CheckpointReady,
            TriggerClass::ManualReview,
        ]
    );
    assert_eq!(ordinals, vec![1, 2, 3, 4]);
}

#[test]
fn live_input_adapter_rejects_fixture_cursor_regression() {
    let path = fixture_path("cursor_regression_stream.jsonl");
    let error = FixtureLiveCheckpointSource::from_path(&path)
        .expect_err("cursor regression fixture must fail validation");

    assert!(matches!(
        error,
        LiveInputError::OutOfOrderCheckpointCursor { .. }
    ));
}

#[test]
fn live_input_adapter_retains_fixture_path_line_and_checkpoint_contract_detail() {
    let source =
        fs::read_to_string(fixture_path("append_only_stream.jsonl")).expect("read source fixture");
    let mut malformed: Value = serde_json::from_str(
        source
            .lines()
            .next()
            .expect("append-only fixture should contain a checkpoint"),
    )
    .expect("parse checkpoint fixture line");
    malformed["checkpoint"]["expected_next_step"] = Value::String(String::new());

    let temp_dir = TempDir::new().expect("temp dir");
    let path = Utf8PathBuf::from_path_buf(temp_dir.path().join("malformed-live.jsonl"))
        .expect("utf8 temp path");
    fs::write(
        &path,
        format!(
            "\n{}\n",
            serde_json::to_string(&malformed).expect("encode malformed fixture")
        ),
    )
    .expect("write malformed fixture");

    let error = FixtureLiveCheckpointSource::from_path(&path)
        .expect_err("malformed checkpoint fixture must fail closed");
    assert!(matches!(
        error,
        LiveInputError::FixtureContractGap {
            path: ref actual_path,
            line_number: 2,
            ref schema_version,
            ref field,
            ref reason,
        } if actual_path == &path
            && schema_version == "v0.3"
            && field == "checkpoint.expected_next_step"
            && reason.contains("session-alpha:0001")
            && reason.contains("non-empty string")
    ));
}

fn fixture_path(name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("live")
        .join(name)
}
