#![allow(unused_crate_dependencies)]

mod support;

use agent_session_compactor::UserMessageRole;
use support::load_sample_bundle;

#[test]
fn context_assembly_prefers_literal_user_objective_and_truth_paths() {
    let bundle = load_sample_bundle();
    let context = agent_drift_analyzer::context::assemble_context(&bundle.sessions[0]);

    assert!(context.objective.text.contains("Packet 6"));
    assert!(context
        .truth_artifacts
        .iter()
        .any(|artifact| artifact.path == "docs/specs/agent-drift-analyzer-v0.1-plan.md"));
    assert!(context
        .working_set_paths
        .iter()
        .any(|path| path.path == "crates/agent-drift-analyzer/src/lib.rs"));
    assert!(context
        .command_families
        .iter()
        .any(|family| family == "cargo"));
}

#[test]
fn context_assembly_prefers_prompt_objective_over_later_steer_rows() {
    let mut bundle = load_sample_bundle();
    let prompt_text = bundle.sessions[0].compact_rows[0].text.clone();
    let mut steer_row = bundle.sessions[0].compact_rows[0].clone();
    steer_row.event_index = 99;
    steer_row.line_number = 99;
    steer_row.text = "/goal Complete only Packet 9. Definition of done: rewrite the scope. Verify with `cargo build -p wrong-crate`.".to_string();
    steer_row.user_message_role = Some(UserMessageRole::Steer);
    bundle.sessions[0].compact_rows.push(steer_row);

    let context = agent_drift_analyzer::context::assemble_context(&bundle.sessions[0]);

    assert_eq!(context.objective.text, prompt_text);
}
