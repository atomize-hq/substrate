use std::collections::BTreeSet;

use agent_session_compactor::{CompactionKind, CompactionRow};

use crate::checkpoint::{Confidence, EvidenceRef, TaskFrame};
use crate::context::ContextPack;
use crate::input::BundleSession;

const EXPLICIT_DELEGATION_MARKERS: [&str; 4] =
    ["multi_agent_v1", "spawn_agent", "wait_agent", "close_agent"];
const CONSERVATIVE_CONTEXT_SIGNALS: [&str; 4] = [
    "child session id",
    "child rollout",
    "separate rollout",
    "spawned agent",
];

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
    let context_signal_evidence = collect_context_signal_evidence(rows, context);
    let supporting_evidence = dedupe_evidence(
        marker_evidence
            .evidence
            .iter()
            .chain(context_signal_evidence.iter()),
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
    if session.archival_rows.is_empty() {
        &session.compact_rows
    } else {
        &session.archival_rows
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
        for marker in EXPLICIT_DELEGATION_MARKERS {
            if !row.text.contains(marker) {
                continue;
            }

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
    }

    MarkerEvidence { markers, evidence }
}

fn collect_context_signal_evidence(
    rows: &[CompactionRow],
    context: &ContextPack,
) -> Vec<EvidenceRef> {
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

        for signal in CONSERVATIVE_CONTEXT_SIGNALS {
            if !row.text.contains(signal) {
                continue;
            }
            let candidate = EvidenceRef {
                row: agent_session_compactor::RowRef::from_row(row),
                reason: format!("delegation context signal: {signal}"),
            };
            let key = evidence_key(&candidate);
            if seen.insert(key) {
                evidence.push(candidate);
            }
        }
    }

    for command in &context.command_observations {
        if !command.raw_command.contains("rollout-") || !command.raw_command.contains("session") {
            continue;
        }
        for candidate in &command.evidence {
            let evidence_ref = EvidenceRef {
                row: candidate.row.clone(),
                reason: "delegation command signal: rollout/session coordination".to_string(),
            };
            let key = evidence_key(&evidence_ref);
            if seen.insert(key) {
                evidence.push(evidence_ref);
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
    context_signal_evidence: &[EvidenceRef],
) -> DelegationTopology {
    if markers.is_empty() {
        if context_signal_evidence.is_empty() {
            return DelegationTopology::SingleAgent;
        }
        return DelegationTopology::MixedOrAmbiguous;
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
    context_signal_evidence: &[EvidenceRef],
) -> ChildWorkVisibility {
    match topology {
        DelegationTopology::SingleAgent | DelegationTopology::DelegatedChild => {
            ChildWorkVisibility::None
        }
        DelegationTopology::DelegatingParent => {
            if context_signal_evidence.is_empty()
                && markers
                    .iter()
                    .any(|marker| matches!(marker.as_str(), "spawn_agent" | "wait_agent"))
            {
                ChildWorkVisibility::Opaque
            } else {
                ChildWorkVisibility::Partial
            }
        }
        DelegationTopology::MixedOrAmbiguous => ChildWorkVisibility::Partial,
    }
}

fn infer_delegation_confidence(
    topology: DelegationTopology,
    markers: &[String],
    context_signal_evidence: &[EvidenceRef],
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

fn evidence_key(evidence: &EvidenceRef) -> String {
    format!(
        "{}:{}:{}:{}",
        evidence.row.source_file,
        evidence.row.event_index,
        evidence.row.row_ordinal,
        evidence.reason
    )
}
