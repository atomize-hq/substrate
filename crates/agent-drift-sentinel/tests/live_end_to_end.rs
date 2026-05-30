#![allow(unused_crate_dependencies)]

use camino::Utf8PathBuf;

use agent_drift_sentinel::{
    emit_operator_events, FixtureLiveCheckpointSource, LiveRuntime, OperatorEvent,
    RecordingOperatorSink, SchedulerPolicy, WarningPolicy,
};

#[test]
fn live_end_to_end_fixture_stream_produces_stable_operator_events() {
    let path = fixture_path("append_only_stream.jsonl");
    let mut source =
        FixtureLiveCheckpointSource::from_path(&path).expect("load append-only live fixture");
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let mut sink = RecordingOperatorSink::default();

    let observations = runtime
        .drain(&mut source)
        .expect("drain fixture-backed live runtime");
    assert_eq!(observations.len(), 4);

    for observation in &observations {
        emit_operator_events(&mut sink, observation).expect("emit operator events");
    }

    let events = sink.into_events();
    assert_eq!(events.len(), 4);

    let visible = match &events[0] {
        OperatorEvent::VisibleWarning(event) => event,
        other => panic!("expected visible warning event, got {other:?}"),
    };
    assert_eq!(visible.cursor.session_id, "session-alpha");
    assert_eq!(visible.cursor.ordinal, 1);
    assert!(visible.presentation.headline.contains("session-alpha:0001"));
    assert!(visible
        .presentation
        .render_console_block(None)
        .contains("align plan to repo truth"));

    let heartbeat = match &events[1] {
        OperatorEvent::Heartbeat(event) => event,
        other => panic!("expected heartbeat event, got {other:?}"),
    };
    assert!(!heartbeat.evaluated);
    assert!(heartbeat.message.contains("warning_debounced"));
    assert_eq!(heartbeat.cursor.ordinal, 1);

    let silent = match &events[2] {
        OperatorEvent::SilentCheckpoint(event) => event,
        other => panic!("expected silent checkpoint event, got {other:?}"),
    };
    assert_eq!(silent.checkpoint_id, "session-alpha:0002");
    assert!(silent
        .reason
        .contains("checkpoint recorded without a visible warning"));

    let status = match &events[3] {
        OperatorEvent::Status(event) => event,
        other => panic!("expected status event, got {other:?}"),
    };
    assert_eq!(status.cursor.ordinal, 2);
    assert!(status.message.contains("manual_review"));
    assert!(status.message.contains("without a visible warning"));

    let final_snapshot = &observations[3].snapshot;
    assert_eq!(final_snapshot.processed_events, 4);
    assert_eq!(
        final_snapshot.latest_checkpoint_id.as_deref(),
        Some("session-alpha:0002")
    );
    assert_eq!(final_snapshot.latest_cursor.as_ref(), Some(&status.cursor));
}

fn fixture_path(name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("live")
        .join(name)
}
