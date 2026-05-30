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
    let checkpoint = &first.sessions[0].checkpoints[0];
    assert_eq!(checkpoint.session_id, "session-alpha");
    assert_eq!(checkpoint.ordinal, 1);
}
