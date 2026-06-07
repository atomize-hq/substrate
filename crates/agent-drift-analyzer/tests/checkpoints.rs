#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::AnalyzeRequest;
use support::{analyze_sample_bundle, load_sample_bundle, BundleFixture};
use time::macros::datetime;

#[test]
fn checkpoints_are_deterministic_and_session_scoped() {
    let first = analyze_sample_bundle();
    let second = analyze_sample_bundle();

    assert_eq!(
        first.sessions[0].checkpoints,
        second.sessions[0].checkpoints
    );
    let checkpoints = &first.sessions[0].checkpoints;
    assert_eq!(checkpoints.len(), 2);
    assert_eq!(checkpoints[0].session_id, "session-alpha");
    assert_eq!(checkpoints[0].schema_version, "v0.4");
    assert_eq!(checkpoints[0].ordinal, 1);
    let first_turn = checkpoints[0]
        .turn_context
        .as_ref()
        .expect("first turn context");
    assert_eq!(first_turn.turn_id.as_deref(), Some("turn-001"));
    assert_eq!(first_turn.turn_ordinal, 1);
    assert_eq!(first_turn.rows_since_turn_start, 9);
    assert_eq!(first_turn.seconds_since_turn_start, None);
    assert_eq!(first_turn.checkpoints_in_turn, 1);
    assert_eq!(first_turn.prompts_observed_in_session, 1);
    assert_eq!(checkpoints[1].ordinal, 2);
    assert_eq!(checkpoints[1].schema_version, "v0.4");
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");
    assert_eq!(second_turn.turn_id.as_deref(), Some("turn-001"));
    assert_eq!(second_turn.turn_ordinal, 1);
    assert_eq!(second_turn.rows_since_turn_start, 13);
    assert_eq!(second_turn.seconds_since_turn_start, None);
    assert_eq!(second_turn.checkpoints_in_turn, 2);
    assert_eq!(second_turn.prompts_observed_in_session, 1);
    assert_eq!(checkpoints[0].boundary.end.event_index, 8);
    assert_eq!(checkpoints[1].boundary.end.event_index, 12);
    assert!(checkpoints[0].boundary.end.event_index < checkpoints[1].boundary.end.event_index);
}

#[test]
fn checkpoints_compute_turn_timing_from_turn_slice_boundaries() {
    let mut bundle = load_sample_bundle();
    for row in &mut bundle.archival_rows {
        row.timestamp = Some(
            datetime!(2026-05-29 12:00:00 UTC)
                + time::Duration::seconds(row.event_index as i64 * 60),
        );
    }
    for row in &mut bundle.compact_rows {
        row.timestamp = Some(
            datetime!(2026-05-29 12:00:00 UTC)
                + time::Duration::seconds(row.event_index as i64 * 60),
        );
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle with timestamps");
    let checkpoints = &result.sessions[0].checkpoints;

    assert_eq!(
        checkpoints[0]
            .turn_context
            .as_ref()
            .and_then(|turn| turn.seconds_since_turn_start),
        Some(480)
    );
    assert_eq!(
        checkpoints[1]
            .turn_context
            .as_ref()
            .and_then(|turn| turn.seconds_since_turn_start),
        Some(720)
    );
}

#[test]
fn checkpoints_start_a_new_turn_when_the_boundary_moves_to_a_new_turn_id() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle with two turns");
    let checkpoints = &result.sessions[0].checkpoints;
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(second_turn.turn_id.as_deref(), Some("turn-002"));
    assert_eq!(second_turn.turn_ordinal, 2);
    assert_eq!(second_turn.rows_since_turn_start, 4);
    assert_eq!(second_turn.checkpoints_in_turn, 1);
    assert_eq!(second_turn.prompts_observed_in_session, 1);
}

#[test]
fn checkpoints_degrade_conservatively_when_no_turn_id_is_available() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
    {
        row.turn_id = None;
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle without turn ids");
    let checkpoints = &result.sessions[0].checkpoints;
    let first_turn = checkpoints[0]
        .turn_context
        .as_ref()
        .expect("first turn context");
    let second_turn = checkpoints[1]
        .turn_context
        .as_ref()
        .expect("second turn context");

    assert_eq!(first_turn.turn_id, None);
    assert_eq!(first_turn.turn_ordinal, 0);
    assert_eq!(first_turn.rows_since_turn_start, 9);
    assert_eq!(first_turn.checkpoints_in_turn, 1);
    assert_eq!(first_turn.prompts_observed_in_session, 1);

    assert_eq!(second_turn.turn_id, None);
    assert_eq!(second_turn.turn_ordinal, 0);
    assert_eq!(second_turn.rows_since_turn_start, 4);
    assert_eq!(second_turn.checkpoints_in_turn, 1);
    assert_eq!(second_turn.prompts_observed_in_session, 1);
}
