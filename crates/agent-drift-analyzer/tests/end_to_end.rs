#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use support::{read_checkpoints, BundleFixture};

#[test]
fn end_to_end_analysis_is_stable_across_reruns() {
    let fixture = BundleFixture::sample();
    let request = agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    };

    let first = agent_drift_analyzer::analyze_bundle(&request).expect("first analysis");
    let first_checkpoints = fs::read_to_string(&first.checkpoints_path).expect("first checkpoints");
    let first_summary = fs::read_to_string(&first.summary_path).expect("first summary");

    let second = agent_drift_analyzer::analyze_bundle(&request).expect("second analysis");
    assert_eq!(
        first_checkpoints,
        fs::read_to_string(&second.checkpoints_path).expect("second checkpoints")
    );
    assert_eq!(
        first_summary,
        fs::read_to_string(&second.summary_path).expect("second summary")
    );

    let checkpoints = read_checkpoints(&second.checkpoints_path);
    assert_eq!(checkpoints[0].schema_version, "v0.1");
    assert!(!checkpoints[0].expected_next_step.is_empty());
}
