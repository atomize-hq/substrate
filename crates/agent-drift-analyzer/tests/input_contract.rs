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
    assert_eq!(bundle.manifest.schema_version, "v0.2");
    assert_eq!(
        bundle.manifest.files[0].session_id.as_deref(),
        Some("session-alpha")
    );
    assert_eq!(bundle.manifest.files[0].turns, vec!["turn-001".to_string()]);
    assert_eq!(bundle.archival_rows[0].turn_id.as_deref(), Some("turn-001"));
    assert_eq!(
        bundle.archival_rows[0].source_file.as_str(),
        "/tmp/session-alpha/rollout.jsonl"
    );
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
fn input_contract_uses_source_file_id_instead_of_inline_source_file() {
    let fixture = BundleFixture::sample();
    let compact_path = fixture.input_dir.join("rows.compact.jsonl");
    let compact_rows = fs::read_to_string(&compact_path).expect("read compact rows");
    assert!(compact_rows
        .lines()
        .all(|line| line.contains("\"source_file_id\"")));
    assert!(compact_rows
        .lines()
        .all(|line| !line.contains("\"session_id\"")));
    assert!(compact_rows
        .lines()
        .all(|line| !line.contains("\"line_number\"")));
    assert!(compact_rows
        .lines()
        .all(|line| !line.contains("\"source_file\"")));
    assert!(compact_rows
        .lines()
        .all(|line| !line.contains("\"turn_id\"")));
    assert!(compact_rows
        .lines()
        .all(|line| line.contains("\"turn_id_ref\"")));

    let bundle = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect("load v0.2 compact rows");
    assert_eq!(bundle.sessions.len(), 1);
    assert!(bundle.surface.literal_objective_rows);
    assert!(bundle
        .compact_rows
        .iter()
        .all(|row| row.turn_id.as_deref() == Some("turn-001")));
}

#[test]
fn input_contract_fails_on_unknown_source_file_id() {
    let fixture = BundleFixture::sample();
    let archival_path = fixture.input_dir.join("rows.archival.jsonl");
    let rewritten = fs::read_to_string(&archival_path)
        .expect("read archival rows")
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let mut row: serde_json::Value = serde_json::from_str(line).expect("archival row json");
            if index == 0 {
                row.as_object_mut()
                    .expect("archival row object")
                    .insert("source_file_id".to_string(), serde_json::json!(99));
            }
            serde_json::to_string(&row).expect("serialize archival row")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&archival_path, format!("{rewritten}\n")).expect("write archival rows");

    let error =
        agent_drift_analyzer::input::load_bundle(&fixture.input_dir).expect_err("unknown file id");
    assert!(error.to_string().contains("unknown source_file_id 99"));
}

#[test]
fn input_contract_fails_on_unknown_turn_id_ref() {
    let fixture = BundleFixture::sample();
    let archival_path = fixture.input_dir.join("rows.archival.jsonl");
    let rewritten = fs::read_to_string(&archival_path)
        .expect("read archival rows")
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let mut row: serde_json::Value = serde_json::from_str(line).expect("archival row json");
            if index == 0 {
                row.as_object_mut()
                    .expect("archival row object")
                    .insert("turn_id_ref".to_string(), serde_json::json!(9));
            }
            serde_json::to_string(&row).expect("serialize archival row")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&archival_path, format!("{rewritten}\n")).expect("write archival rows");

    let error =
        agent_drift_analyzer::input::load_bundle(&fixture.input_dir).expect_err("unknown turn id");
    assert!(error.to_string().contains("unknown turn_id_ref 9"));
}

#[test]
fn input_contract_fails_on_duplicate_manifest_file_ids() {
    let fixture = BundleFixture::sample();
    let manifest_path = fixture.input_dir.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest"))
            .expect("manifest json");
    let files = manifest["files"].as_array_mut().expect("manifest files");
    let duplicate = files.first().cloned().expect("first manifest file");
    files.push(duplicate);
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("manifest json"),
    )
    .expect("write manifest");

    let error = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect_err("duplicate manifest ids");
    assert!(error.to_string().contains("reuses source_file_id 0"));
}
