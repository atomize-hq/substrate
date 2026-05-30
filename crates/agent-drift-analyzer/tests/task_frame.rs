#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::Confidence;
use support::load_sample_bundle;

#[test]
fn task_frame_infers_a_high_confidence_session_goal() {
    let bundle = load_sample_bundle();
    let context = agent_drift_analyzer::context::assemble_context(&bundle.sessions[0]);
    let task_frame = agent_drift_analyzer::inference::infer_task_frame(&context);

    assert!(task_frame.objective.contains("Packet 6"));
    assert_eq!(task_frame.confidence, Confidence::High);
    assert!(task_frame
        .verification_commands
        .iter()
        .any(|command| command == "cargo build -p agent-drift-analyzer"));
}
