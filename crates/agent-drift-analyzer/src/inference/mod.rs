use std::collections::BTreeSet;

use agent_session_compactor::{CompactionKind, CompactionRow};
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
        delegation_rows(session),
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

fn collect_context_signal_evidence<'a>(
    rows: impl IntoIterator<Item = &'a CompactionRow>,
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
        .chain(collect_command_signal_evidence(context))
        .chain(collect_supporting_signal_evidence(context))
        .chain(collect_raw_context_signal_evidence(rows))
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

fn collect_command_signal_evidence(context: &ContextPack) -> Vec<ContextSignalEvidence> {
    let mut evidence = Vec::new();
    for command in &context.command_observations {
        for path in &command.paths {
            if !looks_like_child_rollout_artifact(path) {
                continue;
            }

            evidence.extend(command.evidence.iter().map(|item| ContextSignalEvidence {
                category: ContextSignalCategory::Visibility,
                visibility: Some(ChildWorkVisibility::Partial),
                evidence: EvidenceRef {
                    row: item.row.clone(),
                    reason: format!(
                        "delegation child rollout command via {}: {path}",
                        command.family
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

fn collect_supporting_signal_evidence(context: &ContextPack) -> Vec<ContextSignalEvidence> {
    context
        .supporting_evidence
        .iter()
        .filter_map(|evidence| {
            let reason = evidence.reason.to_ascii_lowercase();
            if let Some(path) = reason
                .split_once(": ")
                .map(|(_, value)| value)
                .filter(|path| looks_like_child_rollout_artifact(path))
            {
                return Some(ContextSignalEvidence {
                    category: ContextSignalCategory::Visibility,
                    visibility: Some(ChildWorkVisibility::Opaque),
                    evidence: EvidenceRef {
                        row: evidence.row.clone(),
                        reason: format!(
                            "delegation supporting hint references child rollout: {path}"
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

fn collect_raw_context_signal_evidence<'a>(
    rows: impl IntoIterator<Item = &'a CompactionRow>,
) -> Vec<ContextSignalEvidence> {
    rows.into_iter()
        .filter_map(|row| {
            let signal = anchored_child_context_signal(row)?;
            Some(ContextSignalEvidence {
                category: ContextSignalCategory::Visibility,
                visibility: Some(ChildWorkVisibility::Opaque),
                evidence: EvidenceRef {
                    row: agent_session_compactor::RowRef::from_row(row),
                    reason: format!("delegation context row anchors {signal}"),
                },
            })
        })
        .collect()
}

fn anchored_child_context_signal(row: &CompactionRow) -> Option<&'static str> {
    if !matches!(
        row.kind,
        CompactionKind::AssistantMessage
            | CompactionKind::UserMessage
            | CompactionKind::DeveloperMessage
            | CompactionKind::SystemMessage
    ) {
        return None;
    }

    let text = row.text.trim_start();
    if text.is_empty() || text.len() > 2_000 {
        return None;
    }

    let anchored = if let Some(stripped) = text.strip_prefix("- ") {
        stripped
    } else if let Some(stripped) = text.strip_prefix("* ") {
        stripped
    } else {
        text
    };
    let normalized = anchored.to_ascii_lowercase();

    if normalized.starts_with("child session id") && normalized.contains("separate rollout") {
        Some("child session id")
    } else if normalized.starts_with("child rollout") {
        Some("child rollout")
    } else if normalized.starts_with("separate rollout") {
        Some("separate rollout")
    } else {
        None
    }
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

fn looks_like_child_rollout_artifact(path: &str) -> bool {
    path.contains(".codex/sessions/") || (path.contains("rollout-") && path.ends_with(".jsonl"))
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
            .any(|evidence| evidence.reason.contains("delegation context signal")));
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
                    "{\"command\":\"sed -n '1,40p' /tmp/child/rollout-019e-test.jsonl\",\"workdir\":\"/repo\"}",
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
                "{\"command\":\"sed -n '1,40p' /tmp/child/rollout-019e-test.jsonl\",\"workdir\":\"/repo\"}",
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
    fn delegation_harvests_anchored_child_boundary_rows() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::AssistantMessage,
                    "Child session id 019e-test lives in a separate rollout file after the spawned agent completed work.",
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
            .any(|evidence| evidence
                .reason
                .contains("delegation context row anchors child session id")));
        assert_eq!(delegation.counter_evidence.len(), 1);
        assert!(delegation.counter_evidence[0]
            .reason
            .contains("remained child-opaque"));
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
