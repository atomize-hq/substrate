#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_sentinel::{
    execute, load_live_fixture, AdjudicationConfig, CheckpointCursor, LiveCheckpointEvent,
    LiveCheckpointSource, LiveInputError, LiveRuntime, LiveRuntimeError, SchedulerPolicy,
    SentinelMode, SentinelRequest, TriggerClass, WarningPolicy,
};
use camino::{Utf8Path, Utf8PathBuf};
use serde_json::{json, Value};
use tempfile::TempDir;

fn checkpoint(session_id: &str, ordinal: usize) -> agent_drift_analyzer::Checkpoint {
    support::checkpoint(
        session_id,
        ordinal,
        82,
        true,
        "continue through the checked public-live boundary",
    )
}

fn runtime() -> LiveRuntime {
    LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default())
}

struct JsonlFixture {
    _temp_dir: TempDir,
    path: Utf8PathBuf,
}

impl JsonlFixture {
    fn from_record(record: Value) -> Self {
        let temp_dir = TempDir::new().expect("temp dir");
        let path = Utf8Path::from_path(temp_dir.path())
            .expect("utf8 temp dir")
            .join("live-events.jsonl");
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string(&record).expect("serialize live record")
            ),
        )
        .expect("write live fixture");
        Self {
            _temp_dir: temp_dir,
            path,
        }
    }
}

struct VecLiveSource {
    events: std::vec::IntoIter<LiveCheckpointEvent>,
}

impl VecLiveSource {
    fn new(events: Vec<LiveCheckpointEvent>) -> Self {
        Self {
            events: events.into_iter(),
        }
    }
}

impl LiveCheckpointSource for VecLiveSource {
    fn next_event(&mut self) -> Result<Option<LiveCheckpointEvent>, LiveInputError> {
        Ok(self.events.next())
    }
}

#[test]
fn public_live_boundary_accepts_minimum_constructor_and_direct_checkpoint_ready_events() {
    let first = checkpoint("session-public-shape", 1);
    let first_cursor = CheckpointCursor::from(&first);
    let mut runtime = runtime();

    let constructor_observation = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(1, first, None))
        .expect("minimum constructor-produced checkpoint-ready event");
    assert_eq!(constructor_observation.event.cursor, first_cursor);
    assert_eq!(
        constructor_observation.event.trigger,
        TriggerClass::CheckpointReady
    );

    let second = checkpoint("session-public-shape", 2);
    let second_cursor = CheckpointCursor::from(&second);
    let direct_event = LiveCheckpointEvent {
        emission_ordinal: 2,
        cursor: second_cursor.clone(),
        trigger: TriggerClass::CheckpointReady,
        checkpoint: Some(second),
        source_label: None,
    };
    let direct_observation = runtime
        .observe(direct_event)
        .expect("direct public caller uses the same checked conversion");

    assert_eq!(direct_observation.event.cursor, second_cursor);
    assert_eq!(direct_observation.snapshot.processed_events, 2);
}

#[test]
fn public_live_boundary_accepts_all_synthetic_discriminators_after_checkpoint() {
    let initial = checkpoint("session-synthetic-shape", 1);
    let cursor = CheckpointCursor::from(&initial);
    let mut runtime = runtime();
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(1, initial, None))
        .expect("establish latest cursor");

    let events = [
        LiveCheckpointEvent::heartbeat(2, cursor.clone(), None),
        LiveCheckpointEvent::repeated_failure(3, cursor.clone(), None),
        LiveCheckpointEvent::manual_review(4, cursor, None),
    ];
    let expected = [
        TriggerClass::Heartbeat,
        TriggerClass::RepeatedFailure,
        TriggerClass::ManualReview,
    ];

    for (event, expected_trigger) in events.into_iter().zip(expected) {
        let observation = runtime
            .observe(event)
            .expect("valid synthetic event after established checkpoint");
        assert_eq!(observation.event.trigger, expected_trigger);
        assert!(observation.event.checkpoint.is_none());
    }
    assert_eq!(runtime.snapshot().processed_events, 4);
}

#[test]
fn checkpoint_ready_missing_payload_cannot_reuse_latest_interpretation() {
    let first = checkpoint("session-missing-payload", 1);
    let second = checkpoint("session-missing-payload", 2);
    let mut runtime = runtime();
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(1, first, None))
        .expect("establish checkpoint");
    let before = runtime.snapshot();

    let mut malformed = LiveCheckpointEvent::checkpoint_ready(2, second, None);
    malformed.checkpoint = None;
    let error = runtime
        .observe(malformed)
        .expect_err("checkpoint-ready without payload must fail before checkpoint reuse");

    assert!(matches!(
        error,
        LiveRuntimeError::MissingCheckpointPayload {
            emission_ordinal: 2
        }
    ));
    assert_eq!(runtime.snapshot(), before);
}

#[test]
fn synthetic_discriminator_with_checkpoint_payload_fails_before_interpretation() {
    let checkpoint = checkpoint("session-contradictory-trigger", 1);
    let mut runtime = runtime();
    let before = runtime.snapshot();
    let mut malformed = LiveCheckpointEvent::checkpoint_ready(1, checkpoint, None);
    malformed.trigger = TriggerClass::Heartbeat;

    let error = runtime
        .observe(malformed)
        .expect_err("synthetic discriminator must reject a checkpoint payload");

    assert!(matches!(
        error,
        LiveRuntimeError::UnexpectedCheckpointPayload {
            emission_ordinal: 1,
            trigger: "heartbeat",
        }
    ));
    assert_eq!(runtime.snapshot(), before);
}

#[test]
fn checkpoint_ready_rejects_session_and_ordinal_cursor_contradictions() {
    let checkpoint = checkpoint("session-checkpoint-identity", 4);
    let cases = [
        CheckpointCursor {
            session_id: "other-session".to_string(),
            ordinal: 4,
        },
        CheckpointCursor {
            session_id: "session-checkpoint-identity".to_string(),
            ordinal: 5,
        },
    ];

    for actual in cases {
        let mut runtime = runtime();
        let mut malformed = LiveCheckpointEvent::checkpoint_ready(1, checkpoint.clone(), None);
        malformed.cursor = actual.clone();
        let error = runtime
            .observe(malformed)
            .expect_err("event cursor must exactly match checkpoint identity");

        assert!(matches!(
            error,
            LiveRuntimeError::CursorMismatch {
                trigger: "checkpoint_ready",
                ref expected_session_id,
                expected_ordinal: 4,
                ref actual_session_id,
                actual_ordinal,
            } if expected_session_id == "session-checkpoint-identity"
                && actual_session_id == &actual.session_id
                && actual_ordinal == actual.ordinal
        ));
        assert_eq!(runtime.snapshot().processed_events, 0);
    }
}

#[test]
fn direct_public_events_reject_empty_and_zero_cursor_identity() {
    let initial = checkpoint("session-invalid-cursor", 1);
    let mut runtime = runtime();
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(1, initial, None))
        .expect("establish latest cursor");
    let before = runtime.snapshot();
    let cases = [
        (
            CheckpointCursor {
                session_id: "   ".to_string(),
                ordinal: 1,
            },
            "session_id",
        ),
        (
            CheckpointCursor {
                session_id: "session-invalid-cursor".to_string(),
                ordinal: 0,
            },
            "ordinal",
        ),
    ];

    for (cursor, expected_field) in cases {
        let error = runtime
            .observe(LiveCheckpointEvent::heartbeat(2, cursor, None))
            .expect_err("invalid public cursor identity must fail at the checked boundary");

        assert!(matches!(
            error,
            LiveRuntimeError::InvalidCursorIdentity {
                emission_ordinal: 2,
                trigger: "heartbeat",
                identity: "event cursor",
                field,
                ..
            } if field == expected_field
        ));
        assert_eq!(runtime.snapshot(), before);
    }
}

#[test]
fn synthetic_event_must_reference_the_runtime_latest_cursor() {
    let initial = checkpoint("session-latest-cursor", 1);
    let mut runtime = runtime();
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(1, initial, None))
        .expect("establish latest cursor");
    let before = runtime.snapshot();

    let error = runtime
        .observe(LiveCheckpointEvent::manual_review(
            2,
            CheckpointCursor {
                session_id: "session-latest-cursor".to_string(),
                ordinal: 2,
            },
            None,
        ))
        .expect_err("synthetic event cannot select another cursor");

    assert!(matches!(
        error,
        LiveRuntimeError::CursorMismatch {
            trigger: "manual_review",
            expected_ordinal: 1,
            actual_ordinal: 2,
            ..
        }
    ));
    assert_eq!(runtime.snapshot(), before);
}

#[test]
fn custom_live_source_drain_cannot_bypass_the_checked_boundary() {
    let checkpoint = checkpoint("session-drain-shape", 1);
    let cursor = CheckpointCursor::from(&checkpoint);
    let mut malformed = LiveCheckpointEvent::heartbeat(2, cursor, None);
    malformed.checkpoint = Some(checkpoint.clone());
    let mut source = VecLiveSource::new(vec![
        LiveCheckpointEvent::checkpoint_ready(1, checkpoint, None),
        malformed,
    ]);
    let mut runtime = runtime();

    let error = runtime
        .drain(&mut source)
        .expect_err("drain must route every source event through observe validation");

    assert!(matches!(
        error,
        LiveRuntimeError::UnexpectedCheckpointPayload {
            emission_ordinal: 2,
            trigger: "heartbeat",
        }
    ));
    assert_eq!(runtime.snapshot().processed_events, 1);
}

#[test]
fn serialized_synthetic_checkpoint_and_contradictory_discriminator_fail_before_conversion() {
    let checkpoint = checkpoint("session-serialized-shape", 1);
    let cursor = CheckpointCursor::from(&checkpoint);
    let synthetic_with_checkpoint = JsonlFixture::from_record(json!({
        "event_type": "heartbeat",
        "emission_ordinal": 2,
        "cursor": cursor,
        "checkpoint": checkpoint,
    }));
    let error = load_live_fixture(&synthetic_with_checkpoint.path)
        .expect_err("serialized synthetic checkpoint payload must fail");
    assert!(matches!(
        error,
        LiveInputError::SerializedEventShape {
            line_number: 1,
            ref event_type,
            ref reason,
            ..
        } if event_type == "heartbeat" && reason.contains("must not include a checkpoint field")
    ));

    let contradictory_trigger = JsonlFixture::from_record(json!({
        "event_type": "heartbeat",
        "trigger": "manual_review",
        "emission_ordinal": 2,
        "cursor": {
            "session_id": "session-serialized-shape",
            "ordinal": 1
        }
    }));
    let error = load_live_fixture(&contradictory_trigger.path)
        .expect_err("contradictory serialized discriminator must fail");
    assert!(matches!(
        error,
        LiveInputError::SerializedEventShape {
            line_number: 1,
            ref event_type,
            ref reason,
            ..
        } if event_type == "heartbeat" && reason.contains("manual_review")
    ));
}

#[test]
fn serialized_checkpoint_cursor_contradiction_and_unsupported_variant_fail_closed() {
    let checkpoint = checkpoint("session-serialized-cursor", 1);
    let mismatched_cursor = JsonlFixture::from_record(json!({
        "event_type": "checkpoint_ready",
        "emission_ordinal": 1,
        "cursor": {
            "session_id": "session-serialized-cursor",
            "ordinal": 2
        },
        "checkpoint": checkpoint,
    }));
    let error = load_live_fixture(&mismatched_cursor.path)
        .expect_err("serialized cursor contradiction must fail before conversion");
    assert!(matches!(
        error,
        LiveInputError::CheckpointCursorMismatch {
            line_number: 1,
            ref expected_session_id,
            expected_ordinal: 1,
            ref actual_session_id,
            actual_ordinal: 2,
        } if expected_session_id == "session-serialized-cursor"
            && actual_session_id == "session-serialized-cursor"
    ));

    let unsupported = JsonlFixture::from_record(json!({
        "event_type": "future_scheduler_event",
        "emission_ordinal": 1
    }));
    let error = load_live_fixture(&unsupported.path)
        .expect_err("unsupported tagged variant must retain serde rejection");
    assert!(matches!(
        error,
        LiveInputError::ParseFixtureLine {
            line_number: 1,
            ref source,
            ..
        } if source.to_string().contains("unknown variant")
            && source.to_string().contains("future_scheduler_event")
    ));
}

#[test]
fn malformed_serialized_identity_and_discriminator_fail_closed() {
    for (cursor, expected_field) in [
        (
            json!({
                "session_id": "",
                "ordinal": 1
            }),
            "session_id",
        ),
        (
            json!({
                "session_id": "session-serialized-identity",
                "ordinal": 0
            }),
            "ordinal",
        ),
    ] {
        let fixture = JsonlFixture::from_record(json!({
            "event_type": "heartbeat",
            "emission_ordinal": 1,
            "cursor": cursor,
        }));
        let error = load_live_fixture(&fixture.path)
            .expect_err("empty or zero serialized cursor identity must fail");
        assert!(matches!(
            error,
            LiveInputError::InvalidCursorIdentity {
                line_number: 1,
                trigger: "heartbeat",
                identity: "event cursor",
                field,
                ..
            } if field == expected_field
        ));
    }

    let empty_redundant_cursor = JsonlFixture::from_record(json!({
        "event_type": "checkpoint_ready",
        "emission_ordinal": 1,
        "cursor": {
            "session_id": "",
            "ordinal": 1
        },
        "checkpoint": checkpoint("session-malformed-cursor", 1),
    }));
    let error = load_live_fixture(&empty_redundant_cursor.path)
        .expect_err("empty redundant checkpoint cursor identity must fail");
    assert!(matches!(
        error,
        LiveInputError::InvalidCursorIdentity {
            line_number: 1,
            trigger: "checkpoint_ready",
            identity: "event cursor",
            field: "session_id",
            ..
        }
    ));

    let malformed_cursor = JsonlFixture::from_record(json!({
        "event_type": "checkpoint_ready",
        "emission_ordinal": 1,
        "cursor": "not-a-checkpoint-cursor",
        "checkpoint": checkpoint("session-malformed-cursor", 1),
    }));
    let error = load_live_fixture(&malformed_cursor.path)
        .expect_err("malformed redundant checkpoint cursor must fail");
    assert!(matches!(
        error,
        LiveInputError::SerializedEventShape {
            line_number: 1,
            ref event_type,
            ref reason,
            ..
        } if event_type == "checkpoint_ready"
            && reason.contains("invalid redundant checkpoint-ready cursor")
    ));

    let malformed_discriminator = JsonlFixture::from_record(json!({
        "event_type": "manual_review",
        "trigger": {"unexpected": true},
        "emission_ordinal": 1,
        "cursor": {
            "session_id": "session-malformed-trigger",
            "ordinal": 1
        }
    }));
    let error = load_live_fixture(&malformed_discriminator.path)
        .expect_err("malformed redundant discriminator must fail");
    assert!(matches!(
        error,
        LiveInputError::SerializedEventShape {
            line_number: 1,
            ref event_type,
            ref reason,
            ..
        } if event_type == "manual_review" && reason.contains("trigger must equal")
    ));
}

#[test]
fn validated_replay_and_public_live_checkpoint_have_identical_visible_outcomes() {
    let checkpoint = checkpoint("session-replay-live-shape", 1);
    let fixture = support::ReplayFixture::from_checkpoints(
        vec![checkpoint.clone()],
        support::sample_summary(),
    );
    let scheduler_policy = SchedulerPolicy::default();
    let warning_policy = WarningPolicy::default();
    let replay = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy,
        warning_policy,
        adjudication: AdjudicationConfig::default(),
    })
    .expect("validated replay checkpoint");

    let mut runtime = LiveRuntime::new(scheduler_policy, warning_policy);
    let live = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(1, checkpoint, None))
        .expect("validated public-live checkpoint");

    assert_eq!(replay.report.visible_warnings.len(), 1);
    assert_eq!(replay.report.visible_warnings[0], live.presentation);
    assert_eq!(replay.report.next_cursor, Some(live.compatibility.cursor));
}
