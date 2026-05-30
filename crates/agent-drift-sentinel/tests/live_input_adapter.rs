#![allow(unused_crate_dependencies)]

use camino::Utf8PathBuf;

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

fn fixture_path(name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("live")
        .join(name)
}
