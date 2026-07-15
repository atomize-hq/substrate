use agent_session_compactor::{
    export_bundle, BundleManifest, ChildSessionOrigin, DelegationLinkState, ExportBundleRequest,
    ParentSpawnResult, RolloutLinkageMetadata, RolloutRowProvenance,
};
use anyhow as _;
use blake3 as _;
use camino::{Utf8Path, Utf8PathBuf};
use clap as _;
use codex as _;
use serde as _;
use tempfile::TempDir;
use thiserror as _;
use time::macros::datetime;
use walkdir as _;

#[test]
fn delegation_link_exact_reciprocal_depth_one_is_verified() {
    let links = export_links(&[
        parent_claim("parent-a", "child-a", "call-a"),
        child_origin("child-a", "parent-a", 1, "/tmp/child-a.jsonl"),
    ]);

    assert_eq!(links.len(), 1);
    assert_eq!(links[0].parent_session_id, "parent-a");
    assert_eq!(links[0].child_session_id, "child-a");
    assert_eq!(
        links[0].child_origin_parent_session_id.as_deref(),
        Some("parent-a")
    );
    assert_eq!(links[0].depth, Some(1));
    assert_eq!(links[0].state, DelegationLinkState::Verified);
    assert_eq!(links[0].parent_evidence.len(), 2);
    assert_eq!(links[0].parent_evidence[0].source_file, "/tmp/parent.jsonl");
    assert_eq!(links[0].parent_evidence[0].line_number, 2);
    assert_eq!(links[0].parent_evidence[0].event_index, 1);
    assert_eq!(links[0].parent_evidence[1].line_number, 3);
    assert_eq!(links[0].parent_evidence[1].event_index, 2);
    assert_eq!(links[0].child_evidence.len(), 1);
    assert_eq!(links[0].child_evidence[0].source_file, "/tmp/child-a.jsonl");
    assert_eq!(links[0].child_evidence[0].line_number, 1);
    assert_eq!(links[0].child_evidence[0].event_index, 0);

    let serialized = serde_json::to_value(&links[0]).expect("serialized delegation link");
    assert_eq!(
        serialized.pointer("/parent_evidence/0/source_file"),
        Some(&serde_json::json!("/tmp/parent.jsonl"))
    );
    assert_eq!(
        serialized.pointer("/child_evidence/0/event_index"),
        Some(&serde_json::json!(0))
    );
}

#[test]
fn delegation_link_non_reciprocal_inputs_remain_typed_non_semantic_states() {
    let cases = [
        (
            vec![parent_claim(
                "parent-only",
                "child-parent-only",
                "call-parent-only",
            )],
            DelegationLinkState::ParentOnly,
        ),
        (
            vec![child_origin(
                "child-only",
                "parent-child-only",
                1,
                "/tmp/child-only.jsonl",
            )],
            DelegationLinkState::ChildOnly,
        ),
        (
            vec![
                parent_claim("parent-claimed", "child-conflict", "call-conflict"),
                child_origin(
                    "child-conflict",
                    "parent-declared",
                    1,
                    "/tmp/child-conflict.jsonl",
                ),
            ],
            DelegationLinkState::ConflictingParent,
        ),
        (
            vec![
                parent_claim("self", "self", "call-self"),
                child_origin("self", "self", 1, "/tmp/self.jsonl"),
            ],
            DelegationLinkState::SelfLink,
        ),
        (
            vec![
                parent_claim("parent-duplicate", "child-duplicate", "call-duplicate-a"),
                parent_claim("parent-duplicate", "child-duplicate", "call-duplicate-b"),
                child_origin(
                    "child-duplicate",
                    "parent-duplicate",
                    1,
                    "/tmp/child-duplicate.jsonl",
                ),
            ],
            DelegationLinkState::Duplicate,
        ),
        (
            vec![
                parent_claim(
                    "parent-duplicate-child",
                    "child-duplicate-child",
                    "call-child",
                ),
                child_origin(
                    "child-duplicate-child",
                    "parent-duplicate-child",
                    1,
                    "/tmp/child-duplicate-child-a.jsonl",
                ),
                child_origin(
                    "child-duplicate-child",
                    "parent-duplicate-child",
                    1,
                    "/tmp/child-duplicate-child-b.jsonl",
                ),
            ],
            DelegationLinkState::Duplicate,
        ),
        (
            vec![parent_claim("parent-malformed", " ", "call-malformed")],
            DelegationLinkState::MalformedSessionId,
        ),
        (
            vec![
                parent_claim("parent-depth-zero", "child-depth-zero", "call-depth-zero"),
                child_origin(
                    "child-depth-zero",
                    "parent-depth-zero",
                    0,
                    "/tmp/child-depth-zero.jsonl",
                ),
            ],
            DelegationLinkState::DepthMismatch,
        ),
        (
            vec![
                parent_claim("parent-deeper", "child-deeper", "call-deeper"),
                child_origin(
                    "child-deeper",
                    "parent-deeper",
                    2,
                    "/tmp/child-deeper.jsonl",
                ),
            ],
            DelegationLinkState::DeeperResidue,
        ),
    ];

    for (metadata, expected_state) in cases {
        let links = export_links(&metadata);
        assert_eq!(links.len(), 1, "state {expected_state:?}");
        assert_eq!(links[0].state, expected_state);
        assert_ne!(links[0].state, DelegationLinkState::Verified);
    }
}

#[test]
fn delegation_link_manifest_order_is_deterministic() {
    let forward = vec![
        child_origin("child-b", "parent-b", 1, "/tmp/child-b.jsonl"),
        parent_claim("parent-a", "child-a", "call-a"),
        child_origin("child-a", "parent-a", 1, "/tmp/child-a.jsonl"),
        parent_claim("parent-b", "child-b", "call-b"),
    ];
    let reverse = forward.iter().cloned().rev().collect::<Vec<_>>();

    let first = export_links(&forward);
    let second = export_links(&reverse);

    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .map(|link| (
                link.parent_session_id.as_str(),
                link.child_session_id.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![("parent-a", "child-a"), ("parent-b", "child-b")]
    );

    let duplicate_observations = export_links(&[
        parent_claim("parent-d", "child-d", "call-d"),
        parent_claim("parent-d", "child-d", "call-d"),
        child_origin("child-d", "parent-d", 1, "/tmp/child-d.jsonl"),
        child_origin("child-d", "parent-d", 1, "/tmp/child-d.jsonl"),
    ]);
    assert_eq!(duplicate_observations.len(), 1);
    assert_eq!(
        duplicate_observations[0].state,
        DelegationLinkState::Duplicate
    );
    assert_eq!(duplicate_observations[0].parent_evidence.len(), 2);
    assert_eq!(duplicate_observations[0].child_evidence.len(), 1);
}

#[test]
fn delegation_link_legacy_v0_2_manifest_defaults_to_no_links() {
    let legacy = serde_json::json!({
        "schema_version": "v0.2",
        "generated_at": "2026-05-29T12:00:00Z",
        "codex_home": "/tmp/.codex",
        "output_dir": "/tmp/output",
        "discovered_file_count": 0,
        "archival_row_count": 0,
        "compact_row_count": 0,
        "dedupe_group_count": 0,
        "session_ids": [],
        "files": []
    });

    let manifest: BundleManifest = serde_json::from_value(legacy).expect("legacy v0.2 manifest");
    assert_eq!(manifest.schema_version, "v0.2");
    assert!(manifest.delegation_links.is_empty());
}

fn export_links(
    metadata: &[RolloutLinkageMetadata],
) -> Vec<agent_session_compactor::DelegationLink> {
    let temp_dir = TempDir::new().expect("temp dir");
    let output_dir = Utf8Path::from_path(temp_dir.path())
        .expect("UTF-8 temp path")
        .join("bundle");
    let manifest = export_bundle(&ExportBundleRequest {
        codex_home: Utf8Path::new("/tmp/.codex"),
        output_dir: &output_dir,
        generated_at: datetime!(2026-05-29 12:00:00 UTC),
        session_ids: Vec::new(),
        source_files: Vec::new(),
        linkage_metadata: metadata,
        archival_rows: &[],
        compact_rows: &[],
        dedupe_groups: &[],
    })
    .expect("export bundle");
    assert_eq!(manifest.schema_version, "v0.2");
    manifest.delegation_links
}

fn parent_claim(parent: &str, child: &str, call_id: &str) -> RolloutLinkageMetadata {
    RolloutLinkageMetadata {
        parent_spawn_results: vec![ParentSpawnResult {
            parent_session_id: parent.to_string(),
            child_session_id: child.to_string(),
            call_id: call_id.to_string(),
            spawn_call_provenance: provenance("/tmp/parent.jsonl", 2, 1),
            spawn_result_provenance: provenance("/tmp/parent.jsonl", 3, 2),
        }],
        child_origin: None,
    }
}

fn child_origin(
    child: &str,
    parent: &str,
    depth: u32,
    source_file: &str,
) -> RolloutLinkageMetadata {
    RolloutLinkageMetadata {
        parent_spawn_results: Vec::new(),
        child_origin: Some(ChildSessionOrigin {
            child_session_id: child.to_string(),
            parent_session_id: parent.to_string(),
            depth,
            agent_nickname: None,
            agent_role: None,
            provenance: provenance(source_file, 1, 0),
        }),
    }
}

fn provenance(path: &str, line_number: usize, event_index: usize) -> RolloutRowProvenance {
    RolloutRowProvenance {
        source_file: Utf8PathBuf::from(path),
        line_number,
        event_index,
    }
}
