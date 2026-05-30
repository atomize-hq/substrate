#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{execute, AdjudicationConfig, SchedulerPolicy, SentinelMode, SentinelRequest, WarningPolicy};

#[test]
fn operator_surface_renders_evidence_backed_visible_warning_blocks() {
    let fixture = support::ReplayFixture::sample();
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains("Agent Drift Sentinel Replay"));
    assert!(rendered.contains("Visible warnings:"));
    assert!(rendered.contains("Expected next step: align plan to repo truth"));
    assert!(rendered.contains("Evidence: session-alpha.jsonl:1#1 flagged score for session-alpha:1"));
}
