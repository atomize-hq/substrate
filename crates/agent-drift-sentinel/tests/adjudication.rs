#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{
    adjudication::shape_request, execute, AdjudicationConfig, ReasoningEffort, SchedulerPolicy,
    SentinelMode, SentinelRequest, WarningPolicy,
};

#[test]
fn adjudication_request_shaping_is_disabled_by_default_and_bounded_when_enabled() {
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

    assert!(replay.adjudication_requests.is_empty());

    let request = shape_request(
        &replay.report.visible_warnings[0],
        &AdjudicationConfig {
            enabled: true,
            model: "gpt-5.4-mini".to_string(),
            reasoning_effort: ReasoningEffort::Medium,
            max_evidence_items: 1,
            max_context_chars: 80,
        },
    )
    .expect("shaped request");

    assert_eq!(request.model, "gpt-5.4-mini");
    assert_eq!(request.reasoning_effort, "medium");
    assert_eq!(request.evidence.len(), 1);
    assert!(request.operator_summary.len() <= 83);
}
