#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{
    adjudication::{fallback_note, success_note},
    execute, AdjudicationConfig, AdjudicationFailure, AdjudicationResponse, SchedulerPolicy,
    SentinelMode, SentinelRequest, WarningPolicy,
};

#[test]
fn adjudication_fallback_keeps_analyzer_evidence_visible() {
    let fixture = support::ReplayFixture::sample();
    let replay = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let warning = &replay.report.visible_warnings[0];
    let fallback = warning.render_console_block(Some(&fallback_note(&AdjudicationFailure {
        message: "network timeout".to_string(),
    })));

    assert!(fallback.contains("using analyzer evidence only"));
    assert!(
        fallback.contains("Evidence: session-alpha.jsonl:1#1 flagged score for session-alpha:1")
    );
    assert!(warning
        .render_console_block(Some(&success_note(&AdjudicationResponse {
            summary: "bounded model agreed with analyzer".to_string(),
        })))
        .contains("bounded model agreed with analyzer"));
}
