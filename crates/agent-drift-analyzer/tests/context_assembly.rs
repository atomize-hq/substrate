#![allow(unused_crate_dependencies)]

mod support;

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
    assert!(context.command_families.iter().any(|family| family == "cargo"));
}
