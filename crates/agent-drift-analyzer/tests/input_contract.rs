#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use support::{load_sample_bundle, BundleFixture};

#[test]
fn input_contract_loads_manifest_rows_and_dedupe_audit() {
    let bundle = load_sample_bundle();
    assert_eq!(bundle.sessions.len(), 1);
    assert!(bundle.surface.literal_objective_rows);
    assert!(bundle.surface.working_set_hints);
    assert_eq!(bundle.dedupe_groups.len(), 1);
}

#[test]
fn input_contract_fails_when_required_files_are_missing() {
    let fixture = BundleFixture::sample();
    fs::remove_file(fixture.input_dir.join("rows.compact.jsonl")).expect("remove compact rows");
    let error =
        agent_drift_analyzer::input::load_bundle(&fixture.input_dir).expect_err("missing artifact");
    assert!(error.to_string().contains("rows.compact.jsonl"));
}
