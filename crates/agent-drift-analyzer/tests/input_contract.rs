#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_session_compactor::{
    CompactionKind, CompactionRow, DedupeGroup, RowRef, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
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
fn input_contract_allows_sparse_tool_payload_surface_when_objective_rows_and_path_hints_survive() {
    let fixture = BundleFixture::from_compact_rows(vec![
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-alpha".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 0,
            line_number: 1,
            row_ordinal: 0,
            timestamp: None,
            kind: CompactionKind::UserMessage,
            user_message_role: Some(UserMessageRole::Prompt),
            dedupe_identity: None,
            text: "/goal Audit crates/agent-drift-analyzer/src/input.rs using docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md".to_string(),
            canonical_text: "/goal Audit crates/agent-drift-analyzer/src/input.rs using docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md".to_string(),
            text_hash_hex: "hash-goal".to_string(),
        },
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-alpha".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 1,
            line_number: 2,
            row_ordinal: 0,
            timestamp: None,
            kind: CompactionKind::SystemMessage,
            user_message_role: None,
            dedupe_identity: None,
            text: "Use crates/agent-drift-analyzer/src/lib.rs as the read-only checkpoint surface.".to_string(),
            canonical_text: "Use crates/agent-drift-analyzer/src/lib.rs as the read-only checkpoint surface.".to_string(),
            text_hash_hex: "hash-system".to_string(),
        },
    ]);

    let bundle = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect("sparse tool payload bundle should load");
    assert!(bundle.surface.literal_objective_rows);
    assert!(bundle.surface.truth_artifact_hints);
    assert!(!bundle.surface.working_set_hints);
    assert!(!bundle.surface.tool_argument_json);
    assert!(bundle.surface.repetition_preserved);
    assert!(bundle.surface.stable_row_refs);
}

#[test]
fn input_contract_allows_sparse_path_hint_surface_when_objective_rows_survive_without_tool_calls() {
    let fixture = BundleFixture::from_compact_rows(vec![CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 0,
        line_number: 1,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::UserMessage,
        user_message_role: Some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: "Explain how the retry behavior works and summarize the tradeoffs.".to_string(),
        canonical_text: "Explain how the retry behavior works and summarize the tradeoffs."
            .to_string(),
        text_hash_hex: "hash-conceptual-ask".to_string(),
    }]);

    let bundle = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect("sparse path-hint bundle should load");
    assert!(bundle.surface.literal_objective_rows);
    assert!(!bundle.surface.truth_artifact_hints);
    assert!(!bundle.surface.working_set_hints);
    assert!(!bundle.surface.tool_argument_json);
    assert!(bundle.surface.repetition_preserved);
    assert!(bundle.surface.stable_row_refs);
}

#[test]
fn input_contract_allows_clean_no_duplicate_bundle_when_archival_covers_compact_rows() {
    let compact_rows = vec![CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 0,
        line_number: 1,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::UserMessage,
        user_message_role: Some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: "/goal Review crates/agent-drift-analyzer/src/input.rs and summarize the contract split."
            .to_string(),
        canonical_text:
            "/goal Review crates/agent-drift-analyzer/src/input.rs and summarize the contract split."
                .to_string(),
        text_hash_hex: "hash-clean-no-duplicate".to_string(),
    }];
    let fixture = BundleFixture::from_rows(compact_rows.clone(), compact_rows, Vec::new());

    let bundle =
        agent_drift_analyzer::input::load_bundle(&fixture.input_dir).expect("clean bundle loads");
    assert!(bundle.surface.literal_objective_rows);
    assert!(bundle.surface.repetition_preserved);
    assert!(bundle.surface.stable_row_refs);
    assert!(bundle.dedupe_groups.is_empty());
}

#[test]
fn input_contract_fails_on_unstable_archival_row_refs_with_exact_variant() {
    let row = CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 0,
        line_number: 1,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::UserMessage,
        user_message_role: Some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: "/goal Review the sparse readable contract.".to_string(),
        canonical_text: "/goal Review the sparse readable contract.".to_string(),
        text_hash_hex: "hash-unstable-ref".to_string(),
    };
    let fixture = BundleFixture::from_rows(vec![row.clone(), row.clone()], vec![row], Vec::new());

    let error = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect_err("unstable row refs should fail");
    assert!(matches!(
        error,
        agent_drift_analyzer::input::InputError::InsufficientContract { ref reason }
        if reason == "row references are not unique and stable"
    ));
}

#[test]
fn input_contract_fails_on_missing_dedupe_representative_with_exact_variant() {
    let row = CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 0,
        line_number: 1,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::UserMessage,
        user_message_role: Some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: "/goal Review the sparse readable contract.".to_string(),
        canonical_text: "/goal Review the sparse readable contract.".to_string(),
        text_hash_hex: "hash-missing-dedupe-rep".to_string(),
    };
    let missing_row = RowRef {
        source_file: row.source_file.clone(),
        event_index: 99,
        row_ordinal: 0,
    };
    let fixture = BundleFixture::from_rows(
        vec![row.clone()],
        vec![row.clone()],
        vec![DedupeGroup {
            kind: row.kind,
            canonical_text_hash_hex: "missing-row".to_string(),
            representative: missing_row.clone(),
            duplicates: Vec::new(),
        }],
    );

    let error = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect_err("missing dedupe representative should fail");
    assert!(matches!(
        error,
        agent_drift_analyzer::input::InputError::MissingDedupeRepresentative { ref row }
        if row == &missing_row
    ));
}

#[test]
fn input_contract_fails_when_archival_rows_do_not_cover_compact_rows_with_exact_variant() {
    let first_row = CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 0,
        line_number: 1,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::UserMessage,
        user_message_role: Some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: "/goal Review the sparse readable contract.".to_string(),
        canonical_text: "/goal Review the sparse readable contract.".to_string(),
        text_hash_hex: "hash-archival-cover-0".to_string(),
    };
    let second_row = CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 1,
        line_number: 2,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::SystemMessage,
        user_message_role: None,
        dedupe_identity: None,
        text: "Keep the regression scoped to tests only.".to_string(),
        canonical_text: "Keep the regression scoped to tests only.".to_string(),
        text_hash_hex: "hash-archival-cover-1".to_string(),
    };
    let fixture = BundleFixture::from_rows(
        vec![first_row.clone()],
        vec![first_row, second_row],
        Vec::new(),
    );

    let error = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect_err("archival rows must cover compact rows");
    assert!(matches!(
        error,
        agent_drift_analyzer::input::InputError::InsufficientContract { ref reason }
        if reason == "archival rows do not preserve repetition beyond the compacted view"
    ));
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
