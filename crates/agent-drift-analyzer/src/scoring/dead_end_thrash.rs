use std::collections::BTreeMap;

use agent_session_compactor::{CompactionKind, RowRef};

use crate::checkpoint::{Confidence, DriftClass, DriftScore, EvidenceRef};
use crate::context::{collect_command_observations, ContextPack};
use crate::input::BundleSession;

pub fn score_dead_end_thrash(session: &BundleSession, context: &ContextPack) -> DriftScore {
    let mut repeated_commands = BTreeMap::<String, Vec<EvidenceRef>>::new();
    let mut repeated_failures = BTreeMap::<String, Vec<EvidenceRef>>::new();

    let archival_commands = collect_command_observations(&session.archival_rows);
    for command in archival_commands
        .iter()
        .filter(|command| command.verification_like)
    {
        repeated_commands
            .entry(command.raw_command.clone())
            .or_default()
            .extend(command.evidence.clone());
    }

    for row in session.archival_rows.iter().filter(|row| {
        matches!(row.kind, CompactionKind::Error | CompactionKind::ToolOutput)
            && !row.text.trim().is_empty()
    }) {
        repeated_failures
            .entry(row.text_hash_hex.clone())
            .or_default()
            .push(EvidenceRef {
                row: RowRef::from_row(row),
                reason: "repeated failure evidence".to_string(),
            });
    }

    let command_loops = repeated_commands
        .into_values()
        .filter(|evidence| evidence.len() >= 3)
        .collect::<Vec<_>>();
    let failure_loops = repeated_failures
        .into_values()
        .filter(|evidence| evidence.len() >= 2)
        .collect::<Vec<_>>();

    let raw_score = ((command_loops.len() * 40) + (failure_loops.len() * 30)).min(100) as u8;
    let mut evidence = Vec::new();
    for loop_evidence in &command_loops {
        evidence.extend(loop_evidence.clone());
    }
    for loop_evidence in &failure_loops {
        evidence.extend(loop_evidence.clone());
    }

    DriftScore {
        class: DriftClass::DeadEndThrash,
        raw_score,
        confidence: if !command_loops.is_empty() {
            Confidence::High
        } else if !failure_loops.is_empty() || !context.command_observations.is_empty() {
            Confidence::Medium
        } else {
            Confidence::Low
        },
        flagged: raw_score >= 60,
        evidence,
    }
}
