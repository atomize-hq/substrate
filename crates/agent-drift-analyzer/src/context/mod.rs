pub mod objective;
pub mod working_set;

use std::collections::{BTreeMap, BTreeSet};

use agent_session_compactor::{CompactionKind, RowRef};
use serde::{Deserialize, Serialize};

use crate::checkpoint::EvidenceRef;
use crate::input::BundleSession;

pub use objective::{extract_objective, extract_verification_commands, ObjectiveSummary};
pub use working_set::{
    collect_command_observations, collect_tools, collect_truth_artifacts,
    collect_working_set_paths, CandidateTruthArtifact, CommandObservation, ToolObservation,
    WorkingSetPath,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextPack {
    pub session_id: String,
    pub objective: ObjectiveSummary,
    pub truth_artifacts: Vec<CandidateTruthArtifact>,
    pub working_set_paths: Vec<WorkingSetPath>,
    pub tools: Vec<ToolObservation>,
    pub command_families: Vec<String>,
    pub command_observations: Vec<CommandObservation>,
    pub supporting_evidence: Vec<EvidenceRef>,
}

pub fn assemble_context(session: &BundleSession) -> ContextPack {
    let objective = extract_objective(&session.compact_rows);
    let truth_artifacts = collect_truth_artifacts(&session.compact_rows, &objective);
    let command_observations = collect_command_observations(&session.compact_rows);
    let working_set_paths = collect_working_set_paths(
        &session.compact_rows,
        &truth_artifacts,
        &command_observations,
    );
    let tools = collect_tools(&command_observations);
    let command_families = unique_command_families(&command_observations);
    let supporting_evidence = collect_supporting_evidence(
        &objective.evidence,
        &truth_artifacts,
        &working_set_paths,
        &tools,
    );

    ContextPack {
        session_id: session.session_id.clone(),
        objective,
        truth_artifacts,
        working_set_paths,
        tools,
        command_families,
        command_observations,
        supporting_evidence,
    }
}

fn collect_supporting_evidence(
    objective: &[EvidenceRef],
    truth_artifacts: &[CandidateTruthArtifact],
    working_set_paths: &[WorkingSetPath],
    tools: &[ToolObservation],
) -> Vec<EvidenceRef> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for evidence in objective
        .iter()
        .chain(
            truth_artifacts
                .iter()
                .flat_map(|artifact| artifact.evidence.iter()),
        )
        .chain(
            working_set_paths
                .iter()
                .flat_map(|path| path.evidence.iter()),
        )
        .chain(tools.iter().flat_map(|tool| tool.evidence.iter()))
    {
        let key = evidence_key(evidence);
        if seen.insert(key) {
            deduped.push(evidence.clone());
        }
    }
    deduped
}

fn unique_command_families(commands: &[CommandObservation]) -> Vec<String> {
    let mut families = commands
        .iter()
        .fold(BTreeMap::new(), |mut acc, command| {
            *acc.entry(command.family.clone()).or_insert(0usize) += 1;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();
    families.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    families.into_iter().map(|(family, _)| family).collect()
}

pub(crate) fn directive_rows(
    rows: &[agent_session_compactor::CompactionRow],
) -> impl Iterator<Item = &agent_session_compactor::CompactionRow> {
    rows.iter().filter(|row| {
        matches!(
            row.kind,
            CompactionKind::UserMessage
                | CompactionKind::DeveloperMessage
                | CompactionKind::SystemMessage
        )
    })
}

pub(crate) fn focusable_directive_rows(
    rows: &[agent_session_compactor::CompactionRow],
) -> impl Iterator<Item = &agent_session_compactor::CompactionRow> {
    directive_rows(rows).filter(|row| row_text_is_focusable(row))
}

pub(crate) fn evidence_from_row(
    row: &agent_session_compactor::CompactionRow,
    reason: impl Into<String>,
) -> EvidenceRef {
    EvidenceRef {
        row: RowRef::from_row(row),
        reason: reason.into(),
    }
}

pub(crate) fn row_text_is_focusable(row: &agent_session_compactor::CompactionRow) -> bool {
    row.text.len() <= 2_000
        && !row.text.contains("AGENTS.md instructions")
        && !row.text.contains("<skill>")
        && !row.text.contains("Available skills")
}

fn evidence_key(evidence: &EvidenceRef) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        evidence.row.source_file,
        evidence.row.line_number,
        evidence.row.event_index,
        evidence.row.row_ordinal,
        evidence.reason
    )
}
