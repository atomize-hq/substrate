use agent_session_compactor::{CompactionKind, CompactionRow};
use serde::{Deserialize, Serialize};

use crate::checkpoint::EvidenceRef;
use crate::context::{directive_rows, evidence_from_row};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectiveSummary {
    pub text: String,
    pub verification_commands: Vec<String>,
    pub evidence: Vec<EvidenceRef>,
}

pub fn extract_objective(rows: &[CompactionRow]) -> ObjectiveSummary {
    let objective_row = rows
        .iter()
        .filter(|row| {
            matches!(
                row.kind,
                CompactionKind::UserMessage | CompactionKind::DeveloperMessage
            )
        })
        .filter(|row| !row.text.trim().is_empty())
        .max_by_key(|row| objective_score(row))
        .or_else(|| directive_rows(rows).find(|row| !row.text.trim().is_empty()));

    match objective_row {
        Some(row) => ObjectiveSummary {
            text: row.text.clone(),
            verification_commands: extract_verification_commands(&row.text),
            evidence: vec![evidence_from_row(row, "literal objective row")],
        },
        None => ObjectiveSummary {
            text: "No objective row available".to_string(),
            verification_commands: Vec::new(),
            evidence: Vec::new(),
        },
    }
}

fn objective_score(row: &CompactionRow) -> (i32, usize, usize) {
    let text = row.text.as_str();
    let mut score = match row.kind {
        CompactionKind::UserMessage => 100,
        CompactionKind::DeveloperMessage => 75,
        _ => 0,
    };
    if text.contains("/goal") {
        score += 1_000;
    }
    if text.contains("Definition of done") {
        score += 400;
    }
    if text.contains("Verify:") || text.contains("Verify with") {
        score += 200;
    }
    if [
        "Complete ",
        "Implement ",
        "Fix ",
        "Add ",
        "Update ",
        "Wire ",
    ]
    .iter()
    .any(|needle| text.contains(needle))
    {
        score += 150;
    }
    if text.contains("AGENTS.md instructions") || text.contains("<skill>") {
        score -= 1_000;
    }
    if text.starts_with("# AGENTS.md") {
        score -= 500;
    }

    (score, text.len(), row.event_index)
}

pub fn extract_verification_commands(text: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut cursor = text;
    while let Some(index) = cursor.find("cargo ") {
        let start = index;
        let remainder = &cursor[start..];
        let end = remainder.find(['`', '\n']).unwrap_or(remainder.len());
        let candidate = remainder[..end].trim().trim_end_matches('.');
        if !candidate.is_empty() && !commands.iter().any(|command| command == candidate) {
            commands.push(candidate.to_string());
        }
        cursor = &remainder[end..];
    }
    commands
}
