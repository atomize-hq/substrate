use std::collections::{BTreeMap, BTreeSet};

use agent_session_compactor::{CompactionKind, CompactionRow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::checkpoint::EvidenceRef;
use crate::context::{evidence_from_row, focusable_directive_rows};
use crate::input::extract_path_hints;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateTruthArtifact {
    pub path: String,
    pub source: String,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkingSetPath {
    pub path: String,
    pub source: String,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolObservation {
    pub name: String,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandObservation {
    pub family: String,
    pub raw_command: String,
    pub tool_name: String,
    pub paths: Vec<String>,
    pub read_like: bool,
    pub write_like: bool,
    pub verification_like: bool,
    pub evidence: Vec<EvidenceRef>,
}

pub fn collect_truth_artifacts(
    rows: &[CompactionRow],
    objective: &super::ObjectiveSummary,
) -> Vec<CandidateTruthArtifact> {
    let mut artifacts = BTreeMap::<String, CandidateTruthArtifact>::new();

    for row in focusable_directive_rows(rows) {
        for path in extract_path_hints(&row.text) {
            let source = if objective.text.contains(&path) {
                "objective_literal"
            } else {
                "directive_literal"
            };
            artifacts
                .entry(path.clone())
                .or_insert_with(|| CandidateTruthArtifact {
                    path: path.clone(),
                    source: source.to_string(),
                    evidence: Vec::new(),
                })
                .evidence
                .push(evidence_from_row(
                    row,
                    format!("truth artifact hint: {path}"),
                ));
        }
    }

    artifacts.into_values().collect()
}

pub fn collect_working_set_paths(
    rows: &[CompactionRow],
    truth_artifacts: &[CandidateTruthArtifact],
    command_observations: &[CommandObservation],
) -> Vec<WorkingSetPath> {
    let mut paths = BTreeMap::<String, WorkingSetPath>::new();

    for artifact in truth_artifacts {
        paths.insert(
            artifact.path.clone(),
            WorkingSetPath {
                path: artifact.path.clone(),
                source: "truth_artifact".to_string(),
                evidence: artifact.evidence.clone(),
            },
        );
    }

    for row in focusable_directive_rows(rows) {
        for path in extract_path_hints(&row.text) {
            paths
                .entry(path.clone())
                .or_insert_with(|| WorkingSetPath {
                    path: path.clone(),
                    source: "directive_literal".to_string(),
                    evidence: Vec::new(),
                })
                .evidence
                .push(evidence_from_row(row, format!("working-set hint: {path}")));
        }
    }

    for observation in command_observations {
        for path in &observation.paths {
            paths
                .entry(path.clone())
                .or_insert_with(|| WorkingSetPath {
                    path: path.clone(),
                    source: "observed_command".to_string(),
                    evidence: Vec::new(),
                })
                .evidence
                .extend(observation.evidence.clone());
        }
    }

    paths.into_values().collect()
}

pub fn collect_tools(commands: &[CommandObservation]) -> Vec<ToolObservation> {
    let mut tools = BTreeMap::<String, Vec<EvidenceRef>>::new();
    for command in commands {
        tools
            .entry(command.tool_name.clone())
            .or_default()
            .extend(command.evidence.clone());
    }
    tools
        .into_iter()
        .map(|(name, evidence)| ToolObservation { name, evidence })
        .collect()
}

pub fn collect_command_observations(rows: &[CompactionRow]) -> Vec<CommandObservation> {
    let mut commands = Vec::new();
    for row in rows
        .iter()
        .filter(|row| row.kind == CompactionKind::ToolCall)
    {
        let tool_name = tool_name(row);
        let payload = serde_json::from_str::<Value>(&row.text).ok();
        let raw_command = payload
            .as_ref()
            .and_then(|value| {
                value
                    .get("command")
                    .or_else(|| value.get("cmd"))
                    .and_then(Value::as_str)
            })
            .unwrap_or_else(|| row.text.as_str())
            .to_string();
        let family = command_family(&raw_command).unwrap_or_else(|| tool_name.clone());
        let mut paths = extract_path_hints(&raw_command);
        if tool_name.contains("apply_patch") {
            paths.extend(extract_apply_patch_paths(&row.text));
        }
        paths.sort();
        paths.dedup();

        commands.push(CommandObservation {
            family: family.clone(),
            raw_command: raw_command.clone(),
            tool_name,
            paths,
            read_like: is_read_like(&family),
            write_like: is_write_like(&family),
            verification_like: is_verification_like(&family),
            evidence: vec![evidence_from_row(row, format!("command family: {family}"))],
        });
    }
    commands
}

fn tool_name(row: &CompactionRow) -> String {
    row.dedupe_identity
        .as_deref()
        .and_then(|identity| serde_json::from_str::<Value>(identity).ok())
        .and_then(|value| {
            value
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "tool_call".to_string())
}

fn command_family(command: &str) -> Option<String> {
    command
        .split(['\n', ';', '|', '&'])
        .flat_map(str::split_whitespace)
        .find(|token| !token.contains('=') && !token.is_empty())
        .map(|token| {
            token
                .trim_matches(|ch: char| matches!(ch, '(' | ')' | '"' | '\''))
                .to_string()
        })
}

fn extract_apply_patch_paths(text: &str) -> Vec<String> {
    let mut paths = BTreeSet::new();
    for line in text.lines() {
        for prefix in ["*** Update File: ", "*** Add File: ", "*** Delete File: "] {
            if let Some(path) = line.strip_prefix(prefix) {
                paths.insert(path.trim().to_string());
            }
        }
    }
    paths.into_iter().collect()
}

fn is_read_like(family: &str) -> bool {
    matches!(
        family,
        "cat" | "sed" | "rg" | "ls" | "find" | "head" | "tail" | "jq" | "git"
    )
}

fn is_write_like(family: &str) -> bool {
    matches!(family, "apply_patch" | "mkdir" | "mv" | "cp" | "cargo")
}

fn is_verification_like(family: &str) -> bool {
    matches!(family, "cargo" | "pnpm" | "npm")
}
