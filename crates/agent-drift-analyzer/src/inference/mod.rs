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
const ORCHESTRATION_CONTEXT_SIGNALS: [&str; 1] = ["spawned agent"];

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
    let rows = delegation_rows(session);
    let marker_evidence = collect_marker_evidence(rows);
    let context_signal_evidence = if marker_evidence.markers.is_empty() {
        Vec::new()
    } else {
        collect_context_signal_evidence(rows, context)
    };
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

    DelegationContext {
        topology,
        child_work_visibility,
        confidence,
        markers: marker_evidence.markers,
        supporting_evidence,
        counter_evidence: Vec::new(),
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

fn delegation_rows(session: &BundleSession) -> &[CompactionRow] {
    if session.compact_rows.is_empty() {
        &session.archival_rows
    } else {
        &session.compact_rows
    }
}

#[derive(Debug, Default)]
struct MarkerEvidence {
    markers: Vec<String>,
    evidence: Vec<EvidenceRef>,
}

fn collect_marker_evidence(rows: &[CompactionRow]) -> MarkerEvidence {
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
    signal: &'static str,
    evidence: EvidenceRef,
}

fn collect_context_signal_evidence(
    rows: &[CompactionRow],
    _context: &ContextPack,
) -> Vec<ContextSignalEvidence> {
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::new();

    for row in rows {
        if !matches!(
            row.kind,
            CompactionKind::AssistantMessage
                | CompactionKind::Status
                | CompactionKind::ToolOutput
                | CompactionKind::ToolCall
        ) {
            continue;
        }

        let row_text = row.text.to_ascii_lowercase();
        for signal in OPAQUE_CHILD_CONTEXT_SIGNALS
            .iter()
            .chain(ORCHESTRATION_CONTEXT_SIGNALS.iter())
        {
            if !row_text.contains(signal) {
                continue;
            }

            let candidate = ContextSignalEvidence {
                signal: *signal,
                evidence: EvidenceRef {
                    row: agent_session_compactor::RowRef::from_row(row),
                    reason: format!("delegation context signal: {signal}"),
                },
            };
            let key = evidence_key(&candidate.evidence);
            if seen.insert(key) {
                evidence.push(candidate);
            }
        }
    }

    evidence
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

    if markers.iter().any(|marker| marker == "spawn_agent")
        || markers.iter().any(|marker| marker == "wait_agent")
        || markers.iter().any(|marker| marker == "close_agent")
        || markers.iter().any(|marker| marker == "multi_agent_v1")
    {
        return DelegationTopology::DelegatingParent;
    }

    DelegationTopology::MixedOrAmbiguous
}

fn infer_child_work_visibility(
    topology: DelegationTopology,
    markers: &[String],
    context_signal_evidence: &[ContextSignalEvidence],
) -> ChildWorkVisibility {
    match topology {
        DelegationTopology::SingleAgent | DelegationTopology::DelegatedChild => {
            ChildWorkVisibility::None
        }
        DelegationTopology::DelegatingParent => {
            if has_opaque_child_signal(context_signal_evidence)
                || (context_signal_evidence.is_empty()
                    && markers
                        .iter()
                        .any(|marker| matches!(marker.as_str(), "spawn_agent" | "wait_agent")))
            {
                ChildWorkVisibility::Opaque
            } else {
                ChildWorkVisibility::Partial
            }
        }
        DelegationTopology::MixedOrAmbiguous => {
            if has_opaque_child_signal(context_signal_evidence) {
                ChildWorkVisibility::Opaque
            } else {
                ChildWorkVisibility::Partial
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
        .any(|evidence| OPAQUE_CHILD_CONTEXT_SIGNALS.contains(&evidence.signal))
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
    use agent_session_compactor::SourceKind;
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
    fn delegation_treats_explicit_child_rollout_boundaries_as_opaque() {
        let session = BundleSession {
            session_id: "session-alpha".to_string(),
            archival_rows: Vec::new(),
            compact_rows: vec![
                tool_call("spawn_agent", "{\"agent_type\":\"worker\"}"),
                row(
                    CompactionKind::AssistantMessage,
                    "Child session id 019e-test lives in a separate rollout file.",
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
