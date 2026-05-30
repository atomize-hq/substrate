#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{execute, AdjudicationConfig, SchedulerPolicy, SentinelMode, SentinelRequest, WarningPolicy};

#[test]
fn warning_policy_keeps_low_signal_or_duplicate_checkpoints_silent() {
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

    assert_eq!(result.report.visible_warnings.len(), 1);
    assert_eq!(result.report.silent_checkpoints.len(), 3);
    assert!(result
        .report
        .silent_checkpoints
        .iter()
        .any(|checkpoint| checkpoint
            .render_console_block(None)
            .contains("below visible score threshold")));
    assert!(result
        .report
        .silent_checkpoints
        .iter()
        .any(|checkpoint| checkpoint
            .render_console_block(None)
            .contains("duplicate replay warning")));
}
