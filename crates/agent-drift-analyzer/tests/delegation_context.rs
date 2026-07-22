#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{
    analyze_bundle, AnalyzeRequest, AnalyzeResult, ChildWorkVisibility, Confidence,
    DelegationTopology, ProgressSignalCode,
};
use agent_session_compactor::{
    BundleManifest, CompactionKind, CompactionRow, DelegationEvidenceRef, DelegationLink,
    DelegationLinkState, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
use support::{read_checkpoints, BundleFixture};

const PARENT: &str = "session-parent";
const CHILD_A: &str = "session-child-a";
const CHILD_B: &str = "session-child-b";

#[test]
fn delegation_verified_graph_derives_ordered_parent_and_child_roles() {
    let fixture = delegation_fixture(&[PARENT, CHILD_A, CHILD_B]);
    let result = analyze_with_links(
        &fixture,
        vec![
            verified_link(PARENT, CHILD_B, 1, 0),
            verified_link(PARENT, CHILD_A, 1, 0),
        ],
    );

    let parent = final_checkpoint(&result, PARENT);
    assert_eq!(
        parent.delegation.topology,
        DelegationTopology::DelegatingParent
    );
    assert_eq!(parent.delegation.parent_session_id, None);
    assert_eq!(
        parent.delegation.child_session_ids,
        vec![CHILD_A.to_string(), CHILD_B.to_string()]
    );
    assert_eq!(
        parent.delegation.child_work_visibility,
        ChildWorkVisibility::Linked
    );
    assert_eq!(parent.delegation.confidence, Confidence::High);
    assert_typed_evidence_is_deterministic(&parent.delegation.supporting_evidence);

    for child_session_id in [CHILD_A, CHILD_B] {
        let child = final_checkpoint(&result, child_session_id);
        assert_eq!(
            child.delegation.topology,
            DelegationTopology::DelegatedChild
        );
        assert_eq!(child.delegation.parent_session_id.as_deref(), Some(PARENT));
        assert!(child.delegation.child_session_ids.is_empty());
        assert_eq!(
            child.delegation.child_work_visibility,
            ChildWorkVisibility::Linked
        );
        assert_eq!(child.delegation.confidence, Confidence::High);
        assert_typed_evidence_is_deterministic(&child.delegation.supporting_evidence);
        assert!(
            child
                .boundary
                .end
                .source_file
                .as_str()
                .contains(child_session_id),
            "child checkpoint must retain its own trajectory boundary"
        );
    }

    assert_ne!(
        final_checkpoint(&result, PARENT).task_frame.objective,
        final_checkpoint(&result, CHILD_A).task_frame.objective,
        "parent and child task frames must remain session-local"
    );
}

#[test]
fn delegation_verified_plus_one_sided_direct_observation_is_partial() {
    let fixture = delegation_fixture(&[PARENT, CHILD_A]);
    let result = analyze_with_links(
        &fixture,
        vec![
            verified_link(PARENT, CHILD_A, 1, 0),
            typed_link(
                PARENT,
                "session-missing-child",
                None,
                None,
                DelegationLinkState::ParentOnly,
                vec![delegation_evidence(PARENT, 1)],
                Vec::new(),
            ),
            typed_link(
                PARENT,
                "session-child-only",
                Some(PARENT),
                Some(1),
                DelegationLinkState::ChildOnly,
                Vec::new(),
                vec![delegation_evidence("session-child-only", 0)],
            ),
        ],
    );

    let parent = final_checkpoint(&result, PARENT);
    assert_eq!(
        parent.delegation.topology,
        DelegationTopology::DelegatingParent
    );
    assert_eq!(
        parent.delegation.child_session_ids,
        vec![CHILD_A.to_string()],
        "only verified graph edges may authorize semantic child ids"
    );
    assert_eq!(
        parent.delegation.child_work_visibility,
        ChildWorkVisibility::Partial
    );
    assert_eq!(parent.delegation.confidence, Confidence::Medium);
    assert!(parent
        .delegation
        .counter_evidence
        .iter()
        .any(|evidence| evidence.reason.contains("parent_only")));
    assert!(parent
        .delegation
        .counter_evidence
        .iter()
        .any(|evidence| evidence.reason.contains("child_only")));
    assert_typed_evidence_is_deterministic(&parent.delegation.counter_evidence);
}

#[test]
fn typed_delegation_limits_parent_progress_before_checkpoint_export() {
    let fixture = delegation_fixture(&[PARENT, CHILD_A]);
    let result = analyze_with_links(
        &fixture,
        vec![
            verified_link(PARENT, CHILD_A, 1, 0),
            typed_link(
                PARENT,
                "session-unresolved-child",
                None,
                None,
                DelegationLinkState::ParentOnly,
                vec![delegation_evidence(PARENT, 1)],
                Vec::new(),
            ),
        ],
    );

    let parent = final_checkpoint(&result, PARENT);
    assert_eq!(
        parent.delegation.topology,
        DelegationTopology::DelegatingParent
    );
    assert_eq!(
        parent.delegation.child_work_visibility,
        ChildWorkVisibility::Partial
    );
    let progress = parent
        .session_progress
        .as_ref()
        .expect("delegating-parent progress");
    assert!(matches!(
        progress.confidence,
        Confidence::Low | Confidence::Medium
    ));
    let limiting_signal = progress
        .signals
        .iter()
        .find(|signal| signal.code == ProgressSignalCode::DelegationVisibilityLimited)
        .expect("typed delegation must limit the canonical progress analysis");
    assert!(!limiting_signal.evidence.is_empty());
    assert!(parent.delegation.counter_evidence.iter().any(|evidence| {
        evidence
            .reason
            .contains("typed delegation observation remained non-semantic: parent_only")
    }));
}

#[test]
fn delegation_conflict_and_role_conflict_fail_closed_without_semantic_ids() {
    let conflict_fixture = delegation_fixture(&[PARENT, CHILD_A]);
    let conflict_result = analyze_with_links(
        &conflict_fixture,
        vec![
            verified_link(PARENT, CHILD_A, 1, 0),
            typed_link(
                PARENT,
                "session-conflicted-child",
                Some("session-other-parent"),
                Some(1),
                DelegationLinkState::ConflictingParent,
                vec![delegation_evidence(PARENT, 1)],
                vec![delegation_evidence("session-conflicted-child", 0)],
            ),
        ],
    );
    assert_opaque_ambiguous(final_checkpoint(&conflict_result, PARENT));

    for (state, child_session_id, origin_parent, depth) in [
        (DelegationLinkState::SelfLink, PARENT, Some(PARENT), Some(1)),
        (
            DelegationLinkState::Duplicate,
            "session-duplicate-child",
            Some(PARENT),
            Some(1),
        ),
        (
            DelegationLinkState::MalformedSessionId,
            "malformed child",
            Some(PARENT),
            Some(1),
        ),
        (
            DelegationLinkState::DepthMismatch,
            "session-depth-mismatch",
            Some(PARENT),
            Some(0),
        ),
    ] {
        let fixture = delegation_fixture(&[PARENT, CHILD_A]);
        let result = analyze_with_links(
            &fixture,
            vec![
                verified_link(PARENT, CHILD_A, 1, 0),
                typed_link(
                    PARENT,
                    child_session_id,
                    origin_parent,
                    depth,
                    state,
                    vec![delegation_evidence(PARENT, 1)],
                    vec![delegation_evidence(child_session_id, 0)],
                ),
            ],
        );
        assert_opaque_ambiguous(final_checkpoint(&result, PARENT));
    }

    let role_fixture = delegation_fixture(&[PARENT, CHILD_A, CHILD_B]);
    let role_result = analyze_with_links(
        &role_fixture,
        vec![
            verified_link(PARENT, CHILD_A, 1, 0),
            verified_link(CHILD_A, CHILD_B, 1, 0),
        ],
    );
    assert_opaque_ambiguous(final_checkpoint(&role_result, CHILD_A));
}

#[test]
fn delegation_heuristics_are_used_only_when_typed_graph_truth_is_absent() {
    let fallback_fixture = heuristic_parent_fixture();
    let fallback = analyze_with_links(&fallback_fixture, Vec::new());
    let fallback_parent = final_checkpoint(&fallback, PARENT);
    assert_eq!(
        fallback_parent.delegation.topology,
        DelegationTopology::DelegatingParent
    );
    assert_eq!(
        fallback_parent.delegation.child_work_visibility,
        ChildWorkVisibility::Opaque
    );
    assert_eq!(
        fallback_parent.delegation.markers,
        vec!["spawn_agent".to_string()]
    );

    let typed_fixture = verified_child_with_local_spawn_marker_fixture();
    let typed = analyze_with_links(&typed_fixture, vec![verified_link(PARENT, CHILD_A, 1, 0)]);
    let typed_child = final_checkpoint(&typed, CHILD_A);
    assert_eq!(
        typed_child.delegation.topology,
        DelegationTopology::DelegatedChild,
        "verified graph truth must outrank the child's local spawn marker"
    );
    assert_eq!(
        typed_child.delegation.child_work_visibility,
        ChildWorkVisibility::Linked
    );
    assert_eq!(
        typed_child.delegation.parent_session_id.as_deref(),
        Some(PARENT)
    );
    assert!(typed_child.delegation.child_session_ids.is_empty());
    assert_eq!(
        typed_child.delegation.markers,
        vec!["spawn_agent".to_string()],
        "markers remain observable but do not drive the role when graph truth exists"
    );
}

#[test]
fn delegation_summary_matches_verified_graph_parent_checkpoint() {
    let fixture = delegation_fixture(&[PARENT, CHILD_A]);
    let result = analyze_with_links(&fixture, vec![verified_link(PARENT, CHILD_A, 1, 0)]);

    assert_summary_matches_exported_delegation(&result, PARENT);
}

#[test]
fn delegation_summary_matches_verified_graph_child_with_local_marker_checkpoint() {
    let fixture = verified_child_with_local_spawn_marker_fixture();
    let result = analyze_with_links(&fixture, vec![verified_link(PARENT, CHILD_A, 1, 0)]);

    assert_summary_matches_exported_delegation(&result, CHILD_A);
}

#[test]
fn delegation_summary_matches_partial_closure_checkpoint() {
    let fixture = delegation_fixture(&[PARENT, CHILD_A]);
    let result = analyze_with_links(
        &fixture,
        vec![
            verified_link(PARENT, CHILD_A, 1, 0),
            typed_link(
                PARENT,
                "session-missing-child",
                None,
                None,
                DelegationLinkState::ParentOnly,
                vec![delegation_evidence(PARENT, 1)],
                Vec::new(),
            ),
        ],
    );

    assert_summary_matches_exported_delegation(&result, PARENT);
}

#[test]
fn delegation_summary_matches_conflict_opaque_checkpoint() {
    let fixture = delegation_fixture(&[PARENT, CHILD_A]);
    let result = analyze_with_links(
        &fixture,
        vec![
            verified_link(PARENT, CHILD_A, 1, 0),
            typed_link(
                PARENT,
                "session-conflicted-child",
                Some("session-other-parent"),
                Some(1),
                DelegationLinkState::ConflictingParent,
                vec![delegation_evidence(PARENT, 1)],
                vec![delegation_evidence("session-conflicted-child", 0)],
            ),
        ],
    );

    assert_summary_matches_exported_delegation(&result, PARENT);
}

fn assert_summary_matches_exported_delegation(result: &AnalyzeResult, session_id: &str) {
    let exported_checkpoints = read_checkpoints(&result.checkpoints_path);
    let exported = exported_checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.session_id == session_id)
        .max_by_key(|checkpoint| checkpoint.ordinal)
        .expect("exported session checkpoint");
    assert_eq!(
        &exported.delegation,
        &final_checkpoint(result, session_id).delegation,
        "checkpoints.jsonl must preserve the public checkpoint delegation context"
    );

    let delegation_json = serde_json::to_value(&exported.delegation)
        .expect("serialize public delegation context for summary comparison");
    let topology = delegation_json["topology"]
        .as_str()
        .expect("delegation topology wire name");
    let visibility = delegation_json["child_work_visibility"]
        .as_str()
        .expect("delegation visibility wire name");
    let confidence = delegation_json["confidence"]
        .as_str()
        .expect("delegation confidence wire name");
    let markers = delegation_json["markers"]
        .as_array()
        .expect("delegation marker list")
        .iter()
        .map(|marker| marker.as_str().expect("delegation marker"))
        .collect::<Vec<_>>();
    let markers = if markers.is_empty() {
        "none".to_string()
    } else {
        markers.join(",")
    };
    let support = format_summary_delegation_evidence(&exported.delegation.supporting_evidence);
    let counter = format_summary_delegation_evidence(&exported.delegation.counter_evidence);

    let summary = fs::read_to_string(&result.summary_path).expect("summary");
    let session_header = format!("## {session_id}\n");
    let session_section = summary
        .split_once(&session_header)
        .map(|(_, tail)| tail)
        .and_then(|tail| tail.split("\n## ").next())
        .expect("session summary section");
    let checkpoint_header = format!("- {}:", exported.checkpoint_id);
    let checkpoint_section = session_section
        .split_once(&checkpoint_header)
        .map(|(_, tail)| tail)
        .and_then(|tail| tail.split("\n- ").next())
        .expect("checkpoint summary section");
    let expected = format!(
        "  delegation: `topology={topology} visibility={visibility} confidence={confidence} markers={markers} support[{support}] counter[{counter}]`"
    );
    assert!(
        checkpoint_section.lines().any(|line| line == expected),
        "summary delegation must match checkpoints.jsonl for {}\nexpected: {expected}\ncheckpoint summary:{checkpoint_section}",
        exported.checkpoint_id
    );
}

fn format_summary_delegation_evidence(evidence: &[agent_drift_analyzer::EvidenceRef]) -> String {
    if evidence.is_empty() {
        return "none".to_string();
    }

    evidence
        .iter()
        .map(|item| {
            let source = item
                .row
                .source_file
                .file_name()
                .unwrap_or(item.row.source_file.as_str());
            format!(
                "{source}@{}:{} {}",
                item.row.event_index, item.row.row_ordinal, item.reason
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn assert_opaque_ambiguous(checkpoint: &agent_drift_analyzer::Checkpoint) {
    assert_eq!(
        checkpoint.delegation.topology,
        DelegationTopology::MixedOrAmbiguous
    );
    assert_eq!(
        checkpoint.delegation.child_work_visibility,
        ChildWorkVisibility::Opaque
    );
    assert_eq!(checkpoint.delegation.confidence, Confidence::Low);
    assert_eq!(checkpoint.delegation.parent_session_id, None);
    assert!(checkpoint.delegation.child_session_ids.is_empty());
    assert!(!checkpoint.delegation.counter_evidence.is_empty());
}

fn assert_typed_evidence_is_deterministic(evidence: &[agent_drift_analyzer::EvidenceRef]) {
    assert!(!evidence.is_empty());
    let keys = evidence
        .iter()
        .map(|evidence| {
            (
                evidence.row.source_file.clone(),
                evidence.row.event_index,
                evidence.row.row_ordinal,
                evidence.reason.clone(),
            )
        })
        .collect::<Vec<_>>();
    let mut sorted = keys.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        keys, sorted,
        "delegation evidence must be stable and deduplicated"
    );
}

fn delegation_fixture(session_ids: &[&str]) -> BundleFixture {
    let rows = session_ids
        .iter()
        .flat_map(|session_id| session_rows(session_id, false))
        .collect();
    BundleFixture::from_compact_rows(rows)
}

fn heuristic_parent_fixture() -> BundleFixture {
    BundleFixture::from_compact_rows(session_rows(PARENT, true))
}

fn verified_child_with_local_spawn_marker_fixture() -> BundleFixture {
    let rows = session_rows(PARENT, false)
        .into_iter()
        .chain(session_rows(CHILD_A, true))
        .collect();
    BundleFixture::from_compact_rows(rows)
}

fn session_rows(session_id: &str, with_spawn_marker: bool) -> Vec<CompactionRow> {
    let source_file = Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl"));
    let objective = format!("/goal Analyze only the {session_id} trajectory.");
    let mut rows = vec![CompactionRow {
        source_file: source_file.clone(),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some(session_id.to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 0,
        line_number: 1,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::UserMessage,
        user_message_role: Some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: objective.clone(),
        canonical_text: objective.clone(),
        text_hash_hex: format!("hash-{session_id}-objective"),
    }];

    if with_spawn_marker {
        rows.push(CompactionRow {
            source_file,
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some(session_id.to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 1,
            line_number: 2,
            row_ordinal: 0,
            timestamp: None,
            kind: CompactionKind::ToolCall,
            user_message_role: None,
            dedupe_identity: Some(
                r#"{"call_id":"call-spawn","name":"spawn_agent","type":"function_call"}"#
                    .to_string(),
            ),
            text: r#"{"goal":"bounded child work"}"#.to_string(),
            canonical_text: "spawn_agent bounded child work".to_string(),
            text_hash_hex: format!("hash-{session_id}-spawn"),
        });
    } else {
        rows.push(CompactionRow {
            source_file,
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some(session_id.to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 1,
            line_number: 2,
            row_ordinal: 0,
            timestamp: None,
            kind: CompactionKind::AssistantMessage,
            user_message_role: None,
            dedupe_identity: None,
            text: format!("Session-local analysis for {session_id}."),
            canonical_text: format!("Session-local analysis for {session_id}."),
            text_hash_hex: format!("hash-{session_id}-assistant"),
        });
    }

    rows
}

fn analyze_with_links(fixture: &BundleFixture, links: Vec<DelegationLink>) -> AnalyzeResult {
    let manifest_path = fixture.input_dir.join("manifest.json");
    let mut manifest: BundleManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read fixture manifest"))
            .expect("parse fixture manifest");
    manifest.delegation_links = links;
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize fixture manifest"),
    )
    .expect("rewrite fixture manifest");

    analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze delegation fixture")
}

fn final_checkpoint<'a>(
    result: &'a AnalyzeResult,
    session_id: &str,
) -> &'a agent_drift_analyzer::Checkpoint {
    result
        .sessions
        .iter()
        .find(|session| session.session_id == session_id)
        .and_then(|session| session.checkpoints.last())
        .expect("session checkpoint")
}

fn verified_link(
    parent_session_id: &str,
    child_session_id: &str,
    parent_event_index: usize,
    child_event_index: usize,
) -> DelegationLink {
    typed_link(
        parent_session_id,
        child_session_id,
        Some(parent_session_id),
        Some(1),
        DelegationLinkState::Verified,
        vec![delegation_evidence(parent_session_id, parent_event_index)],
        vec![delegation_evidence(child_session_id, child_event_index)],
    )
}

fn typed_link(
    parent_session_id: &str,
    child_session_id: &str,
    child_origin_parent_session_id: Option<&str>,
    depth: Option<u32>,
    state: DelegationLinkState,
    parent_evidence: Vec<DelegationEvidenceRef>,
    child_evidence: Vec<DelegationEvidenceRef>,
) -> DelegationLink {
    DelegationLink {
        parent_session_id: parent_session_id.to_string(),
        child_session_id: child_session_id.to_string(),
        child_origin_parent_session_id: child_origin_parent_session_id.map(str::to_string),
        depth,
        state,
        parent_evidence,
        child_evidence,
    }
}

fn delegation_evidence(session_id: &str, event_index: usize) -> DelegationEvidenceRef {
    DelegationEvidenceRef {
        source_file: Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl")),
        line_number: event_index + 1,
        event_index,
    }
}
