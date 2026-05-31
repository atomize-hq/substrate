#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use support::{read_checkpoints, BundleFixture};

#[test]
fn export_bundle_writes_checkpoints_and_summary() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let summary = fs::read_to_string(&result.summary_path).expect("summary");

    assert_eq!(checkpoints.len(), 2);
    assert!(summary.contains("Agent Drift Analyzer Summary"));
    assert!(summary.contains("session-alpha"));
    assert!(summary.contains("Turns observed: `1`"));
    assert!(summary.contains("Checkpoints emitted: `2`"));
    assert!(summary.contains("- Turns observed: `1`"));
}
