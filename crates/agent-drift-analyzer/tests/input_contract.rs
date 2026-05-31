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

#[test]
fn input_contract_accepts_compact_rows_without_canonical_text() {
    let fixture = BundleFixture::sample();
    let compact_path = fixture.input_dir.join("rows.compact.jsonl");
    let rewritten = fs::read_to_string(&compact_path)
        .expect("read compact rows")
        .lines()
        .map(|line| {
            let mut row: serde_json::Value = serde_json::from_str(line).expect("compact row json");
            row.as_object_mut()
                .expect("compact row object")
                .remove("canonical_text");
            serde_json::to_string(&row).expect("serialize compact row")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(compact_path, format!("{rewritten}\n"))
        .expect("write compact rows without canonical");

    let bundle = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect("load compact rows without canonical text");
    assert_eq!(bundle.sessions.len(), 1);
    assert!(bundle.surface.literal_objective_rows);
}
