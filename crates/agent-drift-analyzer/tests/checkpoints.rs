#![allow(unused_crate_dependencies)]

mod support;

use support::analyze_sample_bundle;

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
    assert_eq!(checkpoints[0].ordinal, 1);
    assert_eq!(checkpoints[1].ordinal, 2);
    assert!(checkpoints[0].boundary.end.event_index < checkpoints[1].boundary.end.event_index);
}
