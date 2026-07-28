use std::collections::{BTreeMap, BTreeSet};

use agent_session_compactor::{CompactionKind, CompactionRow, RowRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::checkpoint::{EvidenceRef, ObjectiveClass};
use crate::context::{evidence_from_row, focusable_directive_rows};
use crate::input::{
    classify_directive_path_hints, classify_primary_objective_path_hints, extract_path_hints,
    leading_control_invocation, line_is_summary_heading, markdown_line_is_suppressed,
    normalize_directive_path, normalize_typed_path, token_indexes_inside_quotes_with_state,
    trusted_repository_root,
};

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

const TRUSTED_REPOSITORY_ROOT_SOURCE: &str = "trusted_repository_root";

pub fn collect_truth_artifacts(
    rows: &[CompactionRow],
    objective: &super::ObjectiveSummary,
) -> Vec<CandidateTruthArtifact> {
    let mut artifacts = BTreeMap::<(String, bool), CandidateTruthArtifact>::new();
    let mut objective_paths = BTreeSet::new();
    let mut parsed_objective_paths = BTreeSet::new();
    let mut primary_goal_compatibility_paths = BTreeSet::new();
    let objective_is_not_task = objective
        .structured
        .as_ref()
        .is_some_and(|structured| structured.objective_class == ObjectiveClass::NotTaskStatement);
    let structured_objective_paths = objective
        .structured
        .as_ref()
        .and_then(|structured| structured.target.as_ref())
        .into_iter()
        .flat_map(|target| target.paths.iter())
        .filter_map(|path| normalize_directive_path(path))
        .collect::<BTreeSet<_>>();
    let structured_objective_evidence_paths = objective
        .structured
        .as_ref()
        .and_then(|structured| structured.target.as_ref())
        .into_iter()
        .flat_map(|target| target.evidence.iter())
        .flat_map(|evidence| classify_primary_objective_path_hints(&evidence.excerpt))
        .filter(|hint| hint.ordinary_authority)
        .map(|hint| hint.path)
        .collect::<BTreeSet<_>>();
    let mut saw_objective_evidence_row = false;
    let mut matched_objective_row = false;
    for row in focusable_directive_rows(rows).filter(|row| {
        objective
            .evidence
            .iter()
            .any(|evidence| evidence.row == RowRef::from_row(row))
    }) {
        saw_objective_evidence_row = true;
        if row_is_summary(row) {
            continue;
        }
        matched_objective_row = true;
        let not_task_authority_paths =
            objective_is_not_task.then(|| not_task_statement_explicit_authority_paths(&row.text));
        parsed_objective_paths.extend(
            classify_primary_objective_path_hints(&row.text)
                .into_iter()
                .filter(|hint| {
                    hint.ordinary_authority
                        && not_task_authority_paths
                            .as_ref()
                            .is_none_or(|paths| paths.contains(&hint.path))
                })
                .map(|hint| hint.path),
        );
        primary_goal_compatibility_paths.extend(
            primary_goal_compatibility_paths_from_explicit_clauses(&row.text),
        );
    }
    if matched_objective_row
        && objective
            .structured
            .as_ref()
            .is_some_and(|structured| structured.objective_class == ObjectiveClass::TaskStatement)
    {
        if structured_objective_paths.is_empty() {
            objective_paths.extend(
                structured_objective_evidence_paths
                    .intersection(&parsed_objective_paths)
                    .cloned(),
            );
            if objective
                .structured
                .as_ref()
                .is_some_and(|structured| structured.target.is_none())
                && primary_goal_compatibility_paths.len() >= 2
            {
                objective_paths.extend(primary_goal_compatibility_paths);
            }
        } else {
            objective_paths.extend(
                structured_objective_paths
                    .intersection(&parsed_objective_paths)
                    .cloned(),
            );
        }
    } else if matched_objective_row {
        objective_paths = parsed_objective_paths;
    } else if !saw_objective_evidence_row {
        let not_task_authority_paths = objective_is_not_task
            .then(|| not_task_statement_explicit_authority_paths(&objective.text));
        objective_paths.extend(
            classify_primary_objective_path_hints(&objective.text)
                .into_iter()
                .filter(|hint| {
                    hint.ordinary_authority
                        && not_task_authority_paths
                            .as_ref()
                            .is_none_or(|paths| paths.contains(&hint.path))
                })
                .map(|hint| hint.path),
        );
    }
    for path in &objective_paths {
        artifacts.insert(
            (path.clone(), false),
            CandidateTruthArtifact {
                path: path.clone(),
                source: "objective_literal".to_string(),
                evidence: objective
                    .evidence
                    .iter()
                    .cloned()
                    .map(|mut evidence| {
                        evidence.reason = format!("truth artifact hint: {path}");
                        evidence
                    })
                    .collect(),
            },
        );
    }
    let objective_parent_paths = shared_directive_parent_paths(&objective_paths);

    for row in focusable_directive_rows(rows) {
        let summary_row = row_is_summary(row);
        let objective_evidence_row = objective
            .evidence
            .iter()
            .any(|evidence| evidence.row == RowRef::from_row(row));
        let not_task_authority_paths =
            objective_is_not_task.then(|| not_task_statement_explicit_authority_paths(&row.text));
        let primary_objective_row = !summary_row
            && objective_evidence_row
            && not_task_authority_paths
                .as_ref()
                .is_none_or(|paths| !paths.is_empty());
        let replan_authority_paths = sanctioned_replan_authority_paths(row);
        let explicit_directive_authority_paths = explicit_directive_authority_paths(row);
        let hints = if primary_objective_row {
            classify_primary_objective_path_hints(&row.text)
        } else {
            classify_directive_path_hints(&row.text)
        };
        for hint in hints {
            let row_authority = primary_objective_row
                && hint.ordinary_authority
                && objective_paths.contains(&hint.path)
                || replan_authority_paths.contains(&hint.path)
                || explicit_directive_authority_paths.contains(&hint.path)
                || hint.explicit_authority && !summary_row;
            if hint.control_only || row_authority {
                let source = if row_authority && objective_paths.contains(&hint.path) {
                    "objective_literal"
                } else if hint.control_only {
                    "control_directive_literal"
                } else {
                    "directive_literal"
                };
                let artifact = artifacts
                    .entry((hint.path.clone(), false))
                    .or_insert_with(|| CandidateTruthArtifact {
                        path: hint.path.clone(),
                        source: source.to_string(),
                        evidence: Vec::new(),
                    });
                if artifact.source == "control_directive_literal"
                    && source != "control_directive_literal"
                {
                    artifact.source = source.to_string();
                }
                artifact.evidence.push(evidence_from_row(
                    row,
                    format!("truth artifact hint: {}", hint.path),
                ));
            }

            if hint.trusted_root && !summary_row {
                artifacts
                    .entry((hint.path.clone(), true))
                    .or_insert_with(|| CandidateTruthArtifact {
                        path: hint.path.clone(),
                        source: TRUSTED_REPOSITORY_ROOT_SOURCE.to_string(),
                        evidence: Vec::new(),
                    })
                    .evidence
                    .push(evidence_from_row(
                        row,
                        format!("trusted repository root: {}", hint.path),
                    ));
            }
        }
        if primary_objective_row {
            for path in &objective_parent_paths {
                artifacts
                    .entry((path.clone(), false))
                    .or_insert_with(|| CandidateTruthArtifact {
                        path: path.clone(),
                        source: "objective_literal".to_string(),
                        evidence: Vec::new(),
                    })
                    .evidence
                    .push(evidence_from_row(
                        row,
                        format!("shared objective parent: {path}"),
                    ));
            }
        }
    }

    artifacts.into_values().collect()
}

fn not_task_statement_explicit_authority_paths(text: &str) -> BTreeSet<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.lines().count() != 1 {
        return BTreeSet::new();
    }

    let tokens = trimmed.split_whitespace().collect::<Vec<_>>();
    let first = tokens.first().copied().unwrap_or_default();
    let explicit_surface = if tokens.len() == 1 {
        Some(trimmed.to_string())
    } else if first == "--path" {
        tokens.get(1).map(|value| format!("--path {value}"))
    } else if first.starts_with("--path=") {
        Some(first.to_string())
    } else {
        not_task_trusted_repository_root_value(&tokens).map(ToOwned::to_owned)
    };
    let Some(explicit_surface) = explicit_surface else {
        return BTreeSet::new();
    };

    let ordinary_paths = classify_primary_objective_path_hints(trimmed)
        .into_iter()
        .filter(|hint| hint.ordinary_authority)
        .map(|hint| hint.path)
        .collect::<BTreeSet<_>>();
    classify_primary_objective_path_hints(&explicit_surface)
        .into_iter()
        .filter(|hint| hint.ordinary_authority && ordinary_paths.contains(&hint.path))
        .map(|hint| hint.path)
        .collect()
}

fn not_task_trusted_repository_root_value<'a>(tokens: &[&'a str]) -> Option<&'a str> {
    let mut index = 0;
    if tokens
        .first()
        .is_some_and(|token| token.eq_ignore_ascii_case("the"))
    {
        index += 1;
    }
    let declaration = ["trusted", "repository", "root", "is"];
    if declaration.iter().enumerate().all(|(offset, expected)| {
        tokens
            .get(index + offset)
            .is_some_and(|token| token.eq_ignore_ascii_case(expected))
    }) {
        tokens.get(index + declaration.len()).copied()
    } else {
        None
    }
}

fn primary_goal_compatibility_paths_from_explicit_clauses(text: &str) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut fenced_code = None;
    let mut quote_state = None;
    let mut goal_active = false;
    for line in text.lines() {
        if line.trim().is_empty() {
            goal_active = false;
            continue;
        }
        if markdown_line_is_suppressed(line, &mut fenced_code)
            || line.starts_with("    ")
            || line.starts_with('\t')
        {
            continue;
        }
        let line = line.trim_start();
        if line.starts_with('>') {
            continue;
        }
        if line_is_summary_heading(line) {
            break;
        }

        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let quoted_indexes = token_indexes_inside_quotes_with_state(&tokens, &mut quote_state);
        let mut clause_start = 0;
        if let Some((control_index, control)) =
            leading_control_invocation(line).filter(|(index, _)| !quoted_indexes.contains(index))
        {
            goal_active = control == "/goal";
            if goal_active {
                clause_start = control_index + 1;
            }
        }
        if !goal_active {
            continue;
        }
        while clause_start < tokens.len() {
            let clause_end = (clause_start..tokens.len())
                .find(|index| authority_clause_ends(&tokens[*index]))
                .unwrap_or(tokens.len() - 1);
            let clause = tokens[clause_start..=clause_end].join(" ");
            let clause = clause.trim_start_matches(['-', '*', '+']).trim_start();
            let lower = clause.to_ascii_lowercase();
            let explicit_instruction = lower.starts_with("read ")
                || lower.starts_with("treat ")
                || lower.starts_with("update ")
                || lower.starts_with("modify ")
                || lower.starts_with("edit ")
                || lower.starts_with("write ")
                || lower.starts_with("before final response, update ");
            if explicit_instruction
                && !(clause_start..=clause_end).any(|index| quoted_indexes.contains(&index))
                && !authority_clause_has_negative_predicate(&tokens, clause_start, clause_end)
            {
                let cutoff = authority_clause_exclusion_cutoff(&tokens, clause_start, clause_end);
                let clause = tokens[clause_start..cutoff].join(" ");
                paths.extend(
                    classify_primary_objective_path_hints(&clause)
                        .into_iter()
                        .filter(|hint| hint.ordinary_authority && is_codex_goals_path(&hint.path))
                        .map(|hint| hint.path),
                );
            }
            clause_start = clause_end + 1;
        }
    }
    paths
}

fn explicit_directive_authority_paths(row: &CompactionRow) -> BTreeSet<String> {
    if !matches!(
        row.kind,
        CompactionKind::DeveloperMessage | CompactionKind::SystemMessage
    ) {
        return BTreeSet::new();
    }
    if row_is_summary(row) {
        return BTreeSet::new();
    }
    authority_paths_from_positive_clauses(&row.text, AuthorityClauseKind::Directive)
}

fn sanctioned_replan_authority_paths(row: &CompactionRow) -> BTreeSet<String> {
    if !matches!(row.kind, CompactionKind::UserMessage)
        || row.user_message_role != Some(agent_session_compactor::UserMessageRole::Steer)
        || row_is_summary(row)
    {
        return BTreeSet::new();
    }
    authority_paths_from_positive_clauses(&row.text, AuthorityClauseKind::Replan)
}

#[derive(Clone, Copy)]
enum AuthorityClauseKind {
    Directive,
    Replan,
}

fn authority_paths_from_positive_clauses(
    text: &str,
    kind: AuthorityClauseKind,
) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut fenced_code = None;
    let mut quote_state = None;
    let mut control_active = false;
    for line in text.lines() {
        if line.trim().is_empty() {
            control_active = false;
            continue;
        }
        if markdown_line_is_suppressed(line, &mut fenced_code)
            || line.starts_with("    ")
            || line.starts_with('\t')
        {
            continue;
        }
        let line = line.trim_start();
        if line.starts_with('>') {
            continue;
        }
        if line_is_summary_heading(line) {
            break;
        }

        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let quoted_indexes = token_indexes_inside_quotes_with_state(&tokens, &mut quote_state);
        if leading_control_invocation(line)
            .is_some_and(|(index, _)| !quoted_indexes.contains(&index))
        {
            control_active = true;
        }
        if control_active {
            continue;
        }
        let mut clause_start = 0;
        while clause_start < tokens.len() {
            let clause_end = (clause_start..tokens.len())
                .find(|index| authority_clause_ends(&tokens[*index]))
                .unwrap_or(tokens.len() - 1);
            let clause = tokens[clause_start..=clause_end].join(" ");
            let clause = clause.trim_start_matches(['-', '*', '+']).trim_start();
            let lower = clause.to_ascii_lowercase();
            let lower = lower.strip_prefix("please ").unwrap_or(&lower);
            let positive = match kind {
                AuthorityClauseKind::Directive => {
                    (lower.starts_with("use ") && !lower.starts_with("use of "))
                        || lower.starts_with("stay inside ")
                        || lower.starts_with("work only in ")
                        || lower.starts_with("limit changes to ")
                }
                AuthorityClauseKind::Replan => {
                    lower.starts_with("replan")
                        || lower.starts_with("pivot")
                        || lower.starts_with("new objective")
                        || lower.starts_with("different objective")
                        || lower.starts_with("change objective")
                        || lower.starts_with("change the objective")
                        || lower.starts_with("change scope")
                        || lower.starts_with("change the scope")
                }
            };
            if positive
                && !(clause_start..=clause_end).any(|index| quoted_indexes.contains(&index))
                && !authority_clause_has_negative_predicate(&tokens, clause_start, clause_end)
            {
                let cutoff = authority_clause_exclusion_cutoff(&tokens, clause_start, clause_end);
                let clause = tokens[clause_start..cutoff].join(" ");
                paths.extend(
                    classify_directive_path_hints(&clause)
                        .into_iter()
                        .filter(|hint| hint.ordinary_authority)
                        .map(|hint| hint.path),
                );
            }
            clause_start = clause_end + 1;
        }
    }
    paths
}

fn authority_clause_ends(token: &&str) -> bool {
    token
        .trim_end_matches(['"', '\'', '`', ')', ']', '}'])
        .ends_with(['.', ';', '!', '?'])
}

fn normalized_authority_marker(token: &&str) -> String {
    token
        .trim_matches(|ch: char| {
            matches!(
                ch,
                ',' | ':'
                    | ';'
                    | '.'
                    | '!'
                    | '?'
                    | '"'
                    | '\''
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '`'
            )
        })
        .to_ascii_lowercase()
}

fn authority_clause_is_negative_predicate(token: &&str) -> bool {
    matches!(
        normalized_authority_marker(token).as_str(),
        "not"
            | "no"
            | "never"
            | "rejected"
            | "reject"
            | "denied"
            | "deny"
            | "forbidden"
            | "prohibited"
            | "disallowed"
            | "excluded"
            | "outside"
            | "review-only"
    )
}

fn authority_clause_is_exclusion(token: &&str) -> bool {
    matches!(
        normalized_authority_marker(token).as_str(),
        "without" | "except" | "excluding" | "exclude"
    )
}

fn authority_clause_has_negative_predicate(tokens: &[&str], start: usize, end: usize) -> bool {
    (start..=end).any(|index| authority_clause_is_negative_predicate(&tokens[index]))
}

fn authority_clause_exclusion_cutoff(tokens: &[&str], start: usize, end: usize) -> usize {
    (start..=end)
        .find(|index| authority_clause_is_exclusion(&tokens[*index]))
        .unwrap_or(end + 1)
}

fn row_is_summary(row: &CompactionRow) -> bool {
    row.text
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(line_is_summary_heading)
}

fn shared_directive_parent_paths(paths: &BTreeSet<String>) -> BTreeSet<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for path in paths {
        let Some(separator) = path.rfind(['/', '\\']) else {
            continue;
        };
        let parent = &path[..separator];
        let portable_parent = parent.replace('\\', "/");
        if parent.is_empty()
            || parent == "/"
            || parent.ends_with(':')
            || parent.ends_with(":\\")
            || parent.ends_with(":/")
            || !is_codex_goals_path(&portable_parent)
        {
            continue;
        }
        *counts.entry(parent.to_string()).or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(path, count)| (count >= 2).then_some(path))
        .collect()
}

fn is_codex_goals_path(path: &str) -> bool {
    let portable_path = path.replace('\\', "/");
    portable_path == ".codex/goals" || portable_path.starts_with(".codex/goals/")
}

pub fn collect_working_set_paths(
    rows: &[CompactionRow],
    truth_artifacts: &[CandidateTruthArtifact],
    command_observations: &[CommandObservation],
) -> Vec<WorkingSetPath> {
    let mut paths = BTreeMap::<String, WorkingSetPath>::new();

    for artifact in truth_artifacts {
        if artifact.source == TRUSTED_REPOSITORY_ROOT_SOURCE {
            continue;
        }
        paths.insert(
            artifact.path.clone(),
            WorkingSetPath {
                path: artifact.path.clone(),
                source: if artifact.source == "control_directive_literal" {
                    "control_directive_literal".to_string()
                } else {
                    "truth_artifact".to_string()
                },
                evidence: artifact.evidence.clone(),
            },
        );
    }

    for row in focusable_directive_rows(rows) {
        for hint in classify_directive_path_hints(&row.text) {
            if !hint.control_only && !hint.ordinary_authority {
                continue;
            }
            let source = if hint.control_only {
                "control_directive_literal"
            } else {
                "directive_literal"
            };
            let working_path = paths
                .entry(hint.path.clone())
                .or_insert_with(|| WorkingSetPath {
                    path: hint.path.clone(),
                    source: source.to_string(),
                    evidence: Vec::new(),
                });
            if working_path.source == "control_directive_literal"
                && source != "control_directive_literal"
            {
                working_path.source = source.to_string();
            }
            working_path.evidence.push(evidence_from_row(
                row,
                format!("working-set hint: {}", hint.path),
            ));
        }
    }

    for observation in command_observations {
        for path in &observation.paths {
            let working_path = paths.entry(path.clone()).or_insert_with(|| WorkingSetPath {
                path: path.clone(),
                source: "observed_command".to_string(),
                evidence: Vec::new(),
            });
            if working_path.source == "control_directive_literal" {
                working_path.source = "observed_command".to_string();
            }
            working_path.evidence.extend(observation.evidence.clone());
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
    let trusted_roots = collect_trusted_repository_roots(rows);
    collect_command_observations_with_trusted_roots(rows, &trusted_roots)
}

/// Collects command observations using already-established trusted-root authority.
///
/// Checkpoint intervals may contain actions but omit the earlier root declaration that still
/// governs the session. Callers with a complete context pass those roots explicitly so relative
/// typed paths remain resolvable without letting a tool-provided cwd create authority.
pub(crate) fn collect_command_observations_with_trusted_roots(
    rows: &[CompactionRow],
    trusted_roots: &[String],
) -> Vec<CommandObservation> {
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
            .unwrap_or(row.text.as_str())
            .to_string();
        let typed_filesystem = typed_filesystem_tool(&tool_name);
        let family = if typed_filesystem {
            typed_filesystem_family(&tool_name).to_string()
        } else {
            command_family(&raw_command).unwrap_or_else(|| tool_name.clone())
        };
        let (mut paths, unresolved_paths) = if typed_filesystem {
            typed_filesystem_paths(payload.as_ref(), trusted_roots)
        } else {
            (extract_path_hints(&raw_command), Vec::new())
        };
        if tool_name.contains("apply_patch") {
            paths.extend(extract_apply_patch_paths(&row.text));
        }
        paths.sort();
        paths.dedup();
        let evidence_reason = if unresolved_paths.is_empty() {
            format!("command family: {family}")
        } else {
            format!(
                "command family: {family}; unresolved typed paths: {}",
                unresolved_paths.join(", ")
            )
        };

        commands.push(CommandObservation {
            family: family.clone(),
            raw_command: raw_command.clone(),
            tool_name,
            paths,
            read_like: is_read_like(&family),
            write_like: is_write_like(&family),
            verification_like: is_verification_like(&family),
            evidence: vec![evidence_from_row(row, evidence_reason)],
        });
    }
    commands
}

pub(crate) fn command_has_unresolved_paths(command: &CommandObservation) -> bool {
    command
        .evidence
        .iter()
        .any(|evidence| evidence.reason.contains("; unresolved typed paths: "))
}

fn typed_filesystem_tool(tool_name: &str) -> bool {
    matches!(
        typed_filesystem_family(tool_name),
        "read_file" | "write_file" | "edit_file" | "replace_file" | "create_file" | "delete_file"
    )
}

fn typed_filesystem_family(tool_name: &str) -> &str {
    tool_name
        .rsplit(['.', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(tool_name)
}

fn collect_trusted_repository_roots(rows: &[CompactionRow]) -> Vec<String> {
    focusable_directive_rows(rows)
        .filter(|row| !row_is_summary(row))
        .flat_map(|row| classify_directive_path_hints(&row.text))
        .filter(|hint| hint.trusted_root)
        .filter_map(|hint| trusted_repository_root(&hint.path))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn typed_filesystem_paths(
    payload: Option<&Value>,
    trusted_roots: &[String],
) -> (Vec<String>, Vec<String>) {
    let Some(payload) = payload else {
        return (Vec::new(), vec!["<missing typed payload>".to_string()]);
    };
    let workdir = payload
        .get("workdir")
        .or_else(|| payload.get("cwd"))
        .and_then(Value::as_str);
    let mut paths = BTreeSet::new();
    let mut unresolved = BTreeSet::new();
    let mut saw_path_field = false;
    for field in [
        "path",
        "paths",
        "file",
        "files",
        "file_path",
        "source_path",
        "destination_path",
        "target_path",
        "directory",
        "directories",
    ] {
        let Some(value) = payload.get(field) else {
            continue;
        };
        saw_path_field = true;
        let values = match value {
            Value::String(path) => Some(vec![path.as_str()]),
            Value::Array(values) => values.iter().map(Value::as_str).collect::<Option<Vec<_>>>(),
            _ => None,
        };
        let Some(values) = values else {
            unresolved.insert(format!("{field}=<invalid>"));
            continue;
        };
        if values.is_empty() {
            unresolved.insert(format!("{field}=<empty>"));
            continue;
        }
        for path in values {
            if let Some(resolved) = normalize_typed_path(path, workdir, trusted_roots) {
                paths.insert(resolved);
            } else {
                unresolved.insert(path.to_string());
            }
        }
    }
    if !saw_path_field {
        unresolved.insert("<missing typed path>".to_string());
    }
    (
        paths.into_iter().collect(),
        unresolved.into_iter().collect(),
    )
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
        "cat" | "sed" | "rg" | "ls" | "find" | "head" | "tail" | "jq" | "git" | "read_file"
    )
}

fn is_write_like(family: &str) -> bool {
    matches!(
        family,
        "apply_patch"
            | "mkdir"
            | "mv"
            | "cp"
            | "cargo"
            | "write_file"
            | "edit_file"
            | "replace_file"
            | "create_file"
            | "delete_file"
    )
}

fn is_verification_like(family: &str) -> bool {
    matches!(family, "cargo" | "pnpm" | "npm")
}
