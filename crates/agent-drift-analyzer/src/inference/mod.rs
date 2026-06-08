use std::collections::BTreeSet;

use agent_session_compactor::CompactionRow;
use camino::Utf8Path;
use serde_json::Value;

use crate::checkpoint::{Confidence, EvidenceRef, TaskFrame};
use crate::context::ContextPack;
use crate::input::BundleSession;

const EXPLICIT_DELEGATION_MARKERS: [&str; 4] =
    ["multi_agent_v1", "spawn_agent", "wait_agent", "close_agent"];
const OPAQUE_CHILD_CONTEXT_SIGNALS: [&str; 3] =
    ["child session id", "child rollout", "separate rollout"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DelegationContext {
    pub topology: DelegationTopology,
    pub child_work_visibility: ChildWorkVisibility,
    pub confidence: Confidence,
    pub markers: Vec<String>,
    pub supporting_evidence: Vec<EvidenceRef>,
    pub counter_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DelegationTopology {
    SingleAgent,
    DelegatingParent,
    #[allow(dead_code)]
    DelegatedChild,
    MixedOrAmbiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChildWorkVisibility {
    None,
    Partial,
    Opaque,
}

pub fn infer_task_frame(context: &ContextPack) -> TaskFrame {
    let truth_artifacts = context
        .truth_artifacts
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    let working_set_paths = context
        .working_set_paths
        .iter()
        .map(|path| path.path.clone())
        .collect::<Vec<_>>();
    let tools = context.tools.iter().map(|tool| tool.name.clone()).collect();
    let counter_evidence = infer_counter_evidence(context, &truth_artifacts);
    let confidence = infer_confidence(context, &counter_evidence);

    TaskFrame {
        objective: context.objective.text.clone(),
        confidence,
        truth_artifacts,
        working_set_paths,
        tools,
        command_families: context.command_families.clone(),
        verification_commands: context.objective.verification_commands.clone(),
        supporting_evidence: context.supporting_evidence.clone(),
        counter_evidence,
    }
}

pub(crate) fn infer_delegation_context(
    session: &BundleSession,
    context: &ContextPack,
) -> DelegationContext {
    let marker_evidence = collect_marker_evidence(delegation_rows(session));
    let context_signal_evidence = collect_context_signal_evidence(
        &session.session_id,
        context,
        !marker_evidence.markers.is_empty(),
    );
    let supporting_evidence = dedupe_evidence(
        marker_evidence.evidence.iter().chain(
            context_signal_evidence
                .iter()
                .map(|evidence| &evidence.evidence),
        ),
    );
    let topology = infer_delegation_topology(&marker_evidence.markers, &context_signal_evidence);
    let child_work_visibility =
        infer_child_work_visibility(topology, &marker_evidence.markers, &context_signal_evidence);
    let confidence =
        infer_delegation_confidence(topology, &marker_evidence.markers, &context_signal_evidence);
    let counter_evidence =
        infer_delegation_counter_evidence(context, &marker_evidence, &context_signal_evidence);

    DelegationContext {
        topology,
        child_work_visibility,
        confidence,
        markers: marker_evidence.markers,
        supporting_evidence,
        counter_evidence,
    }
}

fn infer_counter_evidence(context: &ContextPack, truth_artifacts: &[String]) -> Vec<EvidenceRef> {
    let truth_set = truth_artifacts.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::new();
    for command in &context.command_observations {
        for path in &command.paths {
            if !truth_set.contains(path) {
                for item in &command.evidence {
                    let candidate = EvidenceRef {
                        row: item.row.clone(),
                        reason: format!(
                            "observed path outside explicit truth artifact set: {path}"
                        ),
                    };
                    let key = format!(
                        "{}:{}:{}:{}",
                        candidate.row.source_file,
                        candidate.row.event_index,
                        candidate.row.row_ordinal,
                        candidate.reason
                    );
                    if seen.insert(key) {
                        evidence.push(candidate);
                    }
                }
            }
        }
    }
    evidence
}

fn infer_confidence(context: &ContextPack, counter_evidence: &[EvidenceRef]) -> Confidence {
    let has_objective = !context.objective.text.trim().is_empty()
        && context.objective.text != "No objective row available";
    let has_truth = !context.truth_artifacts.is_empty();
    let has_working_set = !context.working_set_paths.is_empty();
    let has_commands = !context.command_observations.is_empty();

    match (
        has_objective,
        has_truth || has_working_set,
        has_commands,
        counter_evidence.len(),
    ) {
        (true, true, true, 0..=2) => Confidence::High,
        (true, true, _, _) | (true, _, true, _) => Confidence::Medium,
        _ => Confidence::Low,
    }
}

fn delegation_rows(session: &BundleSession) -> impl Iterator<Item = &CompactionRow> + '_ {
    session
        .archival_rows
        .iter()
        .chain(session.compact_rows.iter())
}

#[derive(Debug, Default)]
struct MarkerEvidence {
    markers: Vec<String>,
    evidence: Vec<EvidenceRef>,
}

fn collect_marker_evidence<'a>(
    rows: impl IntoIterator<Item = &'a CompactionRow>,
) -> MarkerEvidence {
    let mut seen_markers = BTreeSet::new();
    let mut seen_evidence = BTreeSet::new();
    let mut markers = Vec::new();
    let mut evidence = Vec::new();

    for row in rows {
        let Some(marker) = explicit_marker_from_row(row) else {
            continue;
        };

        if seen_markers.insert(marker) {
            markers.push(marker.to_string());
        }

        let candidate = EvidenceRef {
            row: agent_session_compactor::RowRef::from_row(row),
            reason: format!("delegation marker: {marker}"),
        };
        let key = evidence_key(&candidate);
        if seen_evidence.insert(key) {
            evidence.push(candidate);
        }
    }

    MarkerEvidence { markers, evidence }
}

#[derive(Debug, Clone)]
struct ContextSignalEvidence {
    category: ContextSignalCategory,
    visibility: Option<ChildWorkVisibility>,
    evidence: EvidenceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextSignalCategory {
    Orchestration,
    Visibility,
}

fn collect_context_signal_evidence(
    session_id: &str,
    context: &ContextPack,
    has_explicit_markers: bool,
) -> Vec<ContextSignalEvidence> {
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::new();

    if !has_explicit_markers {
        return evidence;
    }

    for candidate in collect_tool_signal_evidence(context)
        .into_iter()
        .chain(collect_command_signal_evidence(session_id, context))
        .chain(collect_supporting_signal_evidence(session_id, context))
    {
        let key = evidence_key(&candidate.evidence);
        if seen.insert(key) {
            evidence.push(candidate);
        }
    }

    evidence
}

fn collect_tool_signal_evidence(context: &ContextPack) -> Vec<ContextSignalEvidence> {
    context
        .tools
        .iter()
        .filter(|tool| EXPLICIT_DELEGATION_MARKERS.contains(&tool.name.as_str()))
        .flat_map(|tool| {
            tool.evidence
                .iter()
                .map(move |evidence| ContextSignalEvidence {
                    category: ContextSignalCategory::Orchestration,
                    visibility: None,
                    evidence: EvidenceRef {
                        row: evidence.row.clone(),
                        reason: format!("delegation tool observed via context pack: {}", tool.name),
                    },
                })
        })
        .collect()
}

fn collect_command_signal_evidence(
    session_id: &str,
    context: &ContextPack,
) -> Vec<ContextSignalEvidence> {
    let mut evidence = Vec::new();
    for command in &context.command_observations {
        for path in &command.paths {
            let Some(child_session_id) = separate_child_rollout_session_id(path, session_id) else {
                continue;
            };

            evidence.extend(command.evidence.iter().map(|item| ContextSignalEvidence {
                category: ContextSignalCategory::Visibility,
                visibility: Some(ChildWorkVisibility::Partial),
                evidence: EvidenceRef {
                    row: item.row.clone(),
                    reason: format!(
                        "delegation child rollout command via {}: {path} (session {child_session_id})",
                        command.family,
                    ),
                },
            }));
        }

        let raw_command = command.raw_command.to_ascii_lowercase();
        for signal in OPAQUE_CHILD_CONTEXT_SIGNALS {
            if !raw_command.contains(signal) {
                continue;
            }
            evidence.extend(command.evidence.iter().map(|item| ContextSignalEvidence {
                category: ContextSignalCategory::Visibility,
                visibility: Some(ChildWorkVisibility::Opaque),
                evidence: EvidenceRef {
                    row: item.row.clone(),
                    reason: format!("delegation command surface mentions {signal}"),
                },
            }));
        }
    }
    evidence
}

fn collect_supporting_signal_evidence(
    session_id: &str,
    context: &ContextPack,
) -> Vec<ContextSignalEvidence> {
    context
        .supporting_evidence
        .iter()
        .filter_map(|evidence| {
            let reason = evidence.reason.to_ascii_lowercase();
            if let Some((path, child_session_id)) = reason
                .split_once(": ")
                .map(|(_, value)| value)
                .and_then(|path| {
                    separate_child_rollout_session_id(path, session_id)
                        .map(|child_session_id| (path, child_session_id))
                })
            {
                return Some(ContextSignalEvidence {
                    category: ContextSignalCategory::Visibility,
                    visibility: Some(ChildWorkVisibility::Opaque),
                    evidence: EvidenceRef {
                        row: evidence.row.clone(),
                        reason: format!(
                            "delegation supporting hint references child rollout: {path} (session {child_session_id})"
                        ),
                    },
                });
            }

            for signal in OPAQUE_CHILD_CONTEXT_SIGNALS {
                if reason.contains(signal) {
                    return Some(ContextSignalEvidence {
                        category: ContextSignalCategory::Visibility,
                        visibility: Some(ChildWorkVisibility::Opaque),
                        evidence: EvidenceRef {
                            row: evidence.row.clone(),
                            reason: format!("delegation supporting evidence mentions {signal}"),
                        },
                    });
                }
            }

            None
        })
        .collect()
}

fn dedupe_evidence<'a>(items: impl Iterator<Item = &'a EvidenceRef>) -> Vec<EvidenceRef> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for evidence in items {
        let key = evidence_key(evidence);
        if seen.insert(key) {
            deduped.push(evidence.clone());
        }
    }
    deduped
}

fn infer_delegation_topology(
    markers: &[String],
    _context_signal_evidence: &[ContextSignalEvidence],
) -> DelegationTopology {
    if markers.is_empty() {
        return DelegationTopology::SingleAgent;
    }

    if markers.iter().any(|marker| marker == "spawn_agent") {
        return DelegationTopology::DelegatingParent;
    }

    DelegationTopology::MixedOrAmbiguous
}

fn infer_child_work_visibility(
    topology: DelegationTopology,
    _markers: &[String],
    context_signal_evidence: &[ContextSignalEvidence],
) -> ChildWorkVisibility {
    let has_partial_visibility = has_partial_child_signal(context_signal_evidence);
    let has_opaque_visibility = has_opaque_child_signal(context_signal_evidence);

    match topology {
        DelegationTopology::SingleAgent | DelegationTopology::DelegatedChild => {
            ChildWorkVisibility::None
        }
        DelegationTopology::DelegatingParent => {
            if has_partial_visibility {
                ChildWorkVisibility::Partial
            } else if has_opaque_visibility {
                ChildWorkVisibility::Opaque
            } else {
                ChildWorkVisibility::Opaque
            }
        }
        DelegationTopology::MixedOrAmbiguous => {
            if has_opaque_visibility {
                ChildWorkVisibility::Opaque
            } else if has_partial_visibility {
                ChildWorkVisibility::Partial
            } else {
                ChildWorkVisibility::None
            }
        }
    }
}

fn infer_delegation_confidence(
    topology: DelegationTopology,
    markers: &[String],
    context_signal_evidence: &[ContextSignalEvidence],
) -> Confidence {
    match topology {
        DelegationTopology::SingleAgent => Confidence::High,
        DelegationTopology::DelegatingParent => {
            if markers.iter().any(|marker| marker == "multi_agent_v1") && markers.len() >= 2 {
                Confidence::High
            } else {
                Confidence::Medium
            }
        }
        DelegationTopology::DelegatedChild => Confidence::Low,
        DelegationTopology::MixedOrAmbiguous => {
            if context_signal_evidence.is_empty() {
                Confidence::Low
            } else {
                Confidence::Medium
            }
        }
    }
}

fn explicit_marker_from_row(row: &CompactionRow) -> Option<&'static str> {
    let tool_name = tool_name_from_row(row)?;
    EXPLICIT_DELEGATION_MARKERS
        .iter()
        .copied()
        .find(|marker| *marker == tool_name)
}

fn tool_name_from_row(row: &CompactionRow) -> Option<String> {
    row.dedupe_identity
        .as_deref()
        .and_then(|identity| serde_json::from_str::<Value>(identity).ok())
        .and_then(|value| {
            value
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn has_opaque_child_signal(context_signal_evidence: &[ContextSignalEvidence]) -> bool {
    context_signal_evidence
        .iter()
        .any(|evidence| evidence.visibility == Some(ChildWorkVisibility::Opaque))
}

fn has_partial_child_signal(context_signal_evidence: &[ContextSignalEvidence]) -> bool {
    context_signal_evidence
        .iter()
        .any(|evidence| evidence.visibility == Some(ChildWorkVisibility::Partial))
}

fn has_visibility_signal(context_signal_evidence: &[ContextSignalEvidence]) -> bool {
    context_signal_evidence
        .iter()
        .any(|evidence| evidence.category == ContextSignalCategory::Visibility)
}

fn separate_child_rollout_session_id<'a>(path: &'a str, session_id: &str) -> Option<&'a str> {
    if !path.contains(".codex/sessions/") {
        return None;
    }

    let file_name = Utf8Path::new(path).file_name()?;
    let stem = file_name.strip_suffix(".jsonl")?;
    let rollout_name = stem.strip_prefix("rollout-")?;
    let child_session_id = rollout_session_id_suffix(rollout_name)?;

    (child_session_id != session_id).then_some(child_session_id)
}

fn rollout_session_id_suffix(rollout_name: &str) -> Option<&str> {
    let hyphen_offsets = rollout_name
        .match_indices('-')
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    if hyphen_offsets.len() < 5 {
        return None;
    }

    let split_index = hyphen_offsets[hyphen_offsets.len() - 5] + 1;
    let candidate = rollout_name.get(split_index..)?;
    let candidate = candidate.split('-').collect::<Vec<_>>();
    let matches_uuid = matches!(
        candidate.as_slice(),
        [a, b, c, d, e]
            if a.len() == 8
                && b.len() == 4
                && c.len() == 4
                && d.len() == 4
                && e.len() == 12
                && candidate.iter().all(|segment| segment.chars().all(is_ascii_hex))
    );
    if !matches_uuid {
        return None;
    }

    rollout_name.get(split_index..)
}

fn is_ascii_hex(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}

fn infer_delegation_counter_evidence(
    context: &ContextPack,
    marker_evidence: &MarkerEvidence,
    context_signal_evidence: &[ContextSignalEvidence],
) -> Vec<EvidenceRef> {
    if marker_evidence.markers.is_empty() || has_partial_child_signal(context_signal_evidence) {
        return Vec::new();
    }

    let reason = if has_visibility_signal(context_signal_evidence) {
        format!(
            "explicit delegation markers remained child-opaque; no child-visible command evidence appeared across command families: {}",
            summarize_command_families(&context.command_families)
        )
    } else {
        format!(
            "explicit delegation markers lacked child visibility/context evidence across command families: {}",
            summarize_command_families(&context.command_families)
        )
    };

    let candidates = marker_evidence
        .evidence
        .iter()
        .map(|evidence| EvidenceRef {
            row: evidence.row.clone(),
            reason: reason.clone(),
        })
        .collect::<Vec<_>>();
    dedupe_evidence(candidates.iter())
}

fn summarize_command_families(command_families: &[String]) -> String {
    if command_families.is_empty() {
        "none observed".to_string()
    } else {
        command_families
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn evidence_key(evidence: &EvidenceRef) -> String {
    format!(
        "{}:{}:{}:{}",
        evidence.row.source_file,
        evidence.row.event_index,
        evidence.row.row_ordinal,
        evidence.reason
    )
}

#[cfg(test)]
mod tests {
    use agent_session_compactor::{CompactionKind, SourceKind};
    use camino::Utf8PathBuf;

    use crate::context::assemble_context;

    use super::*;

    #[test]
    fn delegation_ignores_archival_marker_docs_and_generic_rollout_session_commands() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![row(
                CompactionKind::Unknown,
                "{\"name\":\"multi_agent_v1\"} {\"name\":\"spawn_agent\"}",
            )],
            compact_rows: vec![tool_call(
                "functions.shell_command",
                "{\"command\":\"rg -n 'rollout-.*session' docs/specs\",\"workdir\":\"/repo\"}",
            )],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::SingleAgent);
        assert_eq!(delegation.child_work_visibility, ChildWorkVisibility::None);
        assert!(delegation.markers.is_empty());
        assert!(delegation.supporting_evidence.is_empty());
    }

    #[test]
    fn delegation_requires_explicit_markers_before_harvesting_context_signals() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![row(
                CompactionKind::AssistantMessage,
                "Child session id 019e-test lives in a separate rollout file after the spawned agent completed work.",
            )],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::SingleAgent);
        assert_eq!(delegation.child_work_visibility, ChildWorkVisibility::None);
        assert!(delegation.markers.is_empty());
        assert!(delegation.supporting_evidence.is_empty());
    }

    #[test]
    fn delegation_does_not_use_quoted_prose_as_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::AssistantMessage,
                    "Quoted review text: \"Child session id 019e-test lives in a separate rollout file.\"",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Opaque
        );
        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(!delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence.reason.contains("context row anchors")));
    }

    #[test]
    fn delegation_harvests_child_rollout_visibility_from_command_surfaces() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Partial
        );
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence.reason.contains("child rollout command")));
        assert!(delegation.counter_evidence.is_empty());
    }

    #[test]
    fn delegation_combines_archival_markers_with_compact_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: vec![tool_call("spawn_agent", "{\"agent_type\":\"worker\"}")],
            compact_rows: vec![tool_call(
                "functions.shell_command",
                "{\"command\":\"sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
            )],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Partial
        );
        assert_eq!(delegation.markers, vec!["spawn_agent".to_string()]);
        assert!(delegation
            .supporting_evidence
            .iter()
            .any(|evidence| evidence.reason.contains("child rollout command")));
        assert!(delegation.counter_evidence.is_empty());
    }

    #[test]
    fn delegation_does_not_promote_anchored_prompt_bullets_to_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::UserMessage,
                    "- Child session id 019e-test lives in a separate rollout file after the spawned agent completed work.",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Opaque
        );
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("context row anchors")));
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(delegation.counter_evidence[0]
            .reason
            .contains("lacked child visibility/context evidence"));
    }

    #[test]
    fn delegation_does_not_promote_copied_spec_text_to_child_visibility() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::AssistantMessage,
                    "Child rollout files / child session ids only justify partial or opaque visibility in R3.75; they do not authorize parent/child stitching in this packet.",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Opaque
        );
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("context row anchors")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_does_not_treat_current_parent_rollout_as_child_visible() {
        let session = BundleSession {
            session_id: "019ea000-0000-7000-8000-000000000000".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea000-0000-7000-8000-000000000000.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Opaque
        );
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("child rollout command")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_does_not_treat_fixture_rollout_as_child_visible() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                tool_call(
                    "functions.shell_command",
                    "{\"command\":\"sed -n '1,40p' crates/agent-drift-analyzer/tests/fixtures/acceptance/sample/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
                ),
            ],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Opaque
        );
        assert!(delegation
            .supporting_evidence
            .iter()
            .all(|evidence| !evidence.reason.contains("child rollout command")));
        assert_eq!(delegation.counter_evidence.len(), 1);
    }

    #[test]
    fn delegation_marks_missing_child_visibility_as_counter_evidence() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![tool_call("spawn_agent", "{\"agent_type\":\"worker\"}")],
        };
        let context = assemble_context(&session);

        let delegation = infer_delegation_context(&session, &context);

        assert_eq!(delegation.topology, DelegationTopology::DelegatingParent);
        assert_eq!(
            delegation.child_work_visibility,
            ChildWorkVisibility::Opaque
        );
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(delegation.counter_evidence[0]
            .reason
            .contains("lacked child visibility/context evidence"));
    }

    #[test]
    fn delegation_degrades_non_spawn_markers_to_mixed_or_ambiguous() {
        for marker in ["multi_agent_v1", "wait_agent", "close_agent"] {
            let session = BundleSession {
                session_id: format!("session-{marker}"),
                archival_rows: Vec::new(),
                compact_rows: vec![tool_call(marker, "{\"agent_type\":\"worker\"}")],
            };
            let context = assemble_context(&session);

            let delegation = infer_delegation_context(&session, &context);

            assert_eq!(delegation.topology, DelegationTopology::MixedOrAmbiguous);
            assert_eq!(delegation.child_work_visibility, ChildWorkVisibility::None);
            assert_eq!(delegation.markers, vec![marker.to_string()]);
            assert_eq!(delegation.counter_evidence.len(), 1);
        }
    }

    fn row(kind: CompactionKind, text: &str) -> CompactionRow {
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-alpha".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 0,
            line_number: 1,
            row_ordinal: 0,
            timestamp: None,
            kind,
            user_message_role: None,
            dedupe_identity: None,
            text: text.to_string(),
            canonical_text: text.to_string(),
            text_hash_hex: format!("{kind:?}-{}", text.len()),
        }
    }

    fn tool_call(tool_name: &str, text: &str) -> CompactionRow {
        let mut row = row(CompactionKind::ToolCall, text);
        row.dedupe_identity = Some(format!(
            "{{\"call_id\":\"call-1\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
        ));
        row
    }
}
