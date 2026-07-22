#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_session_compactor::{
    export_bundle, BundleManifest, ChildSessionOrigin, CompactionKind, CompactionRow, DedupeGroup,
    DelegationLink, DelegationLinkState, ExportBundleRequest, ParentSpawnResult,
    RolloutLinkageMetadata, RolloutRowProvenance, RowRef, SourceKind, UserMessageRole,
};
use camino::{Utf8Path, Utf8PathBuf};
use support::{load_sample_bundle, BundleFixture};
use tempfile::TempDir;

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

#[test]
fn input_contract_loads_verified_links_into_a_deterministic_direct_graph() {
    let rows = vec![
        session_row("session-parent", 0),
        session_row("session-child-b", 1),
        session_row("session-child-a", 2),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
    write_delegation_links(
        &fixture,
        vec![
            direct_link(
                "session-parent",
                "session-child-b",
                DelegationLinkState::Verified,
            ),
            direct_link(
                "session-parent",
                "session-child-a",
                DelegationLinkState::Verified,
            ),
        ],
    );

    let bundle = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect("verified direct links should load");

    assert_eq!(
        bundle
            .delegation_graph
            .by_session_id
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["session-child-a", "session-child-b", "session-parent"]
    );
    let parent = &bundle.delegation_graph.by_session_id["session-parent"];
    assert!(parent.parent_links.is_empty());
    assert_eq!(
        parent
            .child_links
            .iter()
            .map(|link| link.child_session_id.as_str())
            .collect::<Vec<_>>(),
        vec!["session-child-a", "session-child-b"]
    );
    assert_eq!(
        bundle.delegation_graph.by_session_id["session-child-a"]
            .parent_links
            .first()
            .map(|link| link.parent_session_id.as_str()),
        Some("session-parent")
    );
    assert_eq!(
        bundle.delegation_graph.by_session_id["session-child-b"]
            .parent_links
            .first()
            .map(|link| link.parent_session_id.as_str()),
        Some("session-parent")
    );
}

#[test]
fn input_contract_keeps_non_verified_links_out_of_the_semantic_graph() {
    let rows = vec![
        session_row("session-parent", 0),
        session_row("session-child", 1),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
    write_delegation_links(
        &fixture,
        vec![
            direct_link(
                "session-parent",
                "session-child",
                DelegationLinkState::Verified,
            ),
            direct_link(
                "session-parent",
                "session-missing",
                DelegationLinkState::ParentOnly,
            ),
            direct_link(
                "session-conflicting-parent",
                "session-child",
                DelegationLinkState::ConflictingParent,
            ),
        ],
    );

    let bundle = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect("non-verified links should remain bounded evidence");

    assert_eq!(bundle.manifest.delegation_links.len(), 3);
    assert_eq!(bundle.delegation_graph.by_session_id.len(), 2);
    assert_eq!(
        bundle.delegation_graph.by_session_id["session-parent"]
            .child_links
            .iter()
            .map(|link| link.child_session_id.as_str())
            .collect::<Vec<_>>(),
        vec!["session-child"]
    );
    assert_eq!(
        bundle.delegation_graph.by_session_id["session-child"]
            .parent_links
            .first()
            .map(|link| link.parent_session_id.as_str()),
        Some("session-parent")
    );
}

#[test]
fn input_contract_rejects_verified_links_to_sessions_missing_from_the_bundle() {
    assert_missing_verified_link_is_rejected(
        "session-parent",
        "session-parent",
        "session-missing-child",
        "session-missing-child",
    );
    assert_missing_verified_link_is_rejected(
        "session-child",
        "session-missing-parent",
        "session-child",
        "session-missing-parent",
    );
}

#[test]
fn input_contract_accepts_compactor_registered_verified_metadata_only_child() {
    let temp_dir = TempDir::new().expect("temp dir");
    let input_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp path")
        .join("bundle");
    let parent_session_id = "session-parent";
    let child_session_id = "session-child";
    let parent_path = Utf8PathBuf::from(format!("/tmp/{parent_session_id}/rollout.jsonl"));
    let child_path = Utf8PathBuf::from(format!("/tmp/{child_session_id}/rollout.jsonl"));
    let parent_row = session_row(parent_session_id, 0);
    let archival_rows = vec![parent_row.clone()];
    let compact_rows = vec![parent_row];
    let linkage_metadata = vec![
        RolloutLinkageMetadata {
            parent_spawn_results: vec![ParentSpawnResult {
                parent_session_id: parent_session_id.to_string(),
                child_session_id: child_session_id.to_string(),
                call_id: "call-spawn".to_string(),
                spawn_call_provenance: RolloutRowProvenance {
                    source_file: parent_path.clone(),
                    line_number: 2,
                    event_index: 1,
                },
                spawn_result_provenance: RolloutRowProvenance {
                    source_file: parent_path.clone(),
                    line_number: 3,
                    event_index: 2,
                },
            }],
            child_origin: None,
        },
        RolloutLinkageMetadata {
            parent_spawn_results: Vec::new(),
            child_origin: Some(ChildSessionOrigin {
                child_session_id: child_session_id.to_string(),
                parent_session_id: parent_session_id.to_string(),
                depth: 1,
                agent_nickname: None,
                agent_role: None,
                provenance: RolloutRowProvenance {
                    source_file: child_path.clone(),
                    line_number: 1,
                    event_index: 0,
                },
            }),
        },
    ];

    export_bundle(&ExportBundleRequest {
        codex_home: Utf8Path::new("/tmp/.codex"),
        output_dir: &input_dir,
        generated_at: time::OffsetDateTime::UNIX_EPOCH,
        session_ids: vec![parent_session_id.to_string(), child_session_id.to_string()],
        source_files: vec![parent_path, child_path.clone()],
        linkage_metadata: &linkage_metadata,
        archival_rows: &archival_rows,
        compact_rows: &compact_rows,
        dedupe_groups: &[],
    })
    .expect("export compactor-produced sparse child bundle");

    let bundle = agent_drift_analyzer::input::load_bundle(&input_dir)
        .expect("registered metadata-only child bundle should validate");
    let child = bundle
        .sessions
        .iter()
        .find(|session| session.session_id == child_session_id)
        .expect("metadata-only child session");
    assert!(child.archival_rows.is_empty());
    assert!(child.compact_rows.is_empty());
    assert_eq!(
        bundle.delegation_graph.by_session_id[parent_session_id]
            .child_links
            .first()
            .map(|link| link.child_session_id.as_str()),
        Some(child_session_id)
    );
    let child_file = bundle
        .manifest
        .files
        .iter()
        .find(|file| file.path == child_path)
        .expect("metadata-only child file registry entry");
    assert_eq!(child_file.session_id.as_deref(), Some(child_session_id));
    assert!(child_file.turns.is_empty());
    assert!(bundle
        .archival_rows
        .iter()
        .chain(bundle.compact_rows.iter())
        .all(|row| row.session_id.as_deref() != Some(child_session_id)));
}

#[test]
fn input_contract_loads_legacy_v0_2_manifests_with_an_empty_link_graph() {
    let fixture = BundleFixture::sample();
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture.input_dir.join("manifest.json")).expect("manifest"),
    )
    .expect("manifest json");
    assert!(manifest.get("delegation_links").is_none());

    let bundle = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect("legacy v0.2 manifest should load");

    assert!(bundle.manifest.delegation_links.is_empty());
    assert!(bundle.delegation_graph.by_session_id.is_empty());
}

fn session_row(session_id: &str, event_index: usize) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl")),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some(session_id.to_string()),
        turn_id: None,
        event_index,
        line_number: event_index + 1,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::UserMessage,
        user_message_role: Some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: format!("/goal Analyze {session_id}."),
        canonical_text: format!("/goal Analyze {session_id}."),
        text_hash_hex: format!("hash-{session_id}"),
    }
}

fn direct_link(
    parent_session_id: &str,
    child_session_id: &str,
    state: DelegationLinkState,
) -> DelegationLink {
    DelegationLink {
        parent_session_id: parent_session_id.to_string(),
        child_session_id: child_session_id.to_string(),
        child_origin_parent_session_id: Some(parent_session_id.to_string()),
        depth: Some(1),
        state,
        parent_evidence: Vec::new(),
        child_evidence: Vec::new(),
    }
}

fn write_delegation_links(fixture: &BundleFixture, delegation_links: Vec<DelegationLink>) {
    let manifest_path = fixture.input_dir.join("manifest.json");
    let mut manifest: BundleManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read bundle manifest"))
            .expect("parse bundle manifest");
    manifest.delegation_links = delegation_links;
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize bundle manifest"),
    )
    .expect("write bundle manifest");
}

fn assert_missing_verified_link_is_rejected(
    included_session_id: &str,
    parent_session_id: &str,
    child_session_id: &str,
    expected_missing_session_id: &str,
) {
    let rows = vec![session_row(included_session_id, 0)];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
    write_delegation_links(
        &fixture,
        vec![direct_link(
            parent_session_id,
            child_session_id,
            DelegationLinkState::Verified,
        )],
    );

    let error = agent_drift_analyzer::input::load_bundle(&fixture.input_dir)
        .expect_err("verified links require both sessions in the bundle");

    assert!(matches!(
        error,
        agent_drift_analyzer::input::InputError::VerifiedDelegationSessionMissing {
            parent_session_id: ref actual_parent_session_id,
            child_session_id: ref actual_child_session_id,
            missing_session_id: ref actual_missing_session_id,
        } if actual_parent_session_id == parent_session_id
            && actual_child_session_id == child_session_id
            && actual_missing_session_id == expected_missing_session_id
    ));
}
