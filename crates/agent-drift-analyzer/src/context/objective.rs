use agent_session_compactor::{CompactionKind, CompactionRow, UserMessageRole};
use serde::{Deserialize, Serialize};

use crate::checkpoint::{EvidenceRef, StructuredObjective};
use crate::context::{directive_rows, evidence_from_row};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectiveSummary {
    pub text: String,
    #[serde(default)]
    pub comparison_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured: Option<StructuredObjective>,
    pub verification_commands: Vec<String>,
    pub evidence: Vec<EvidenceRef>,
}

impl ObjectiveSummary {
    pub fn compatibility(
        text: String,
        verification_commands: Vec<String>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        let comparison_key = text.clone();
        Self {
            text,
            comparison_key,
            structured: None,
            verification_commands,
            evidence,
        }
    }
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
        Some(row) => {
            let objective_text =
                extract_section_aware_objective_text(&row.text).unwrap_or_else(|| row.text.clone());
            ObjectiveSummary::compatibility(
                objective_text,
                extract_verification_commands(&row.text),
                vec![evidence_from_row(row, "literal objective row")],
            )
        }
        None => ObjectiveSummary::compatibility(
            "No objective row available".to_string(),
            Vec::new(),
            Vec::new(),
        ),
    }
}

fn objective_score(row: &CompactionRow) -> (i32, usize, usize) {
    let text = row.text.as_str();
    let mut score = match row.kind {
        CompactionKind::UserMessage => 100,
        CompactionKind::DeveloperMessage => 75,
        _ => 0,
    };
    score += match row.user_message_role.unwrap_or(UserMessageRole::Unknown) {
        UserMessageRole::Prompt => 600,
        UserMessageRole::Unknown => 200,
        UserMessageRole::Steer => 0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ObjectiveSectionKind {
    Mission,
    Constraints,
    Deliverables,
    Context,
    Verification,
    Checklist,
    Boilerplate,
    ToolingInstruction,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObjectiveSection {
    kind: ObjectiveSectionKind,
    body: String,
}

fn extract_section_aware_objective_text(text: &str) -> Option<String> {
    if !text.contains('\n') {
        return None;
    }

    let sections = split_objective_sections(text);
    if sections.len() < 2 {
        return None;
    }

    let mut ranked_sections = sections
        .iter()
        .filter_map(|section| {
            let display = display_text_for_section(section)?;
            let score = section_priority(section, &display);
            Some((score, display, section.kind))
        })
        .collect::<Vec<_>>();

    ranked_sections.sort_by(|left, right| right.0.cmp(&left.0));

    let (best_score, best_text, best_kind) = ranked_sections.first()?.clone();
    let second_score = ranked_sections
        .get(1)
        .map(|entry| entry.0)
        .unwrap_or(i32::MIN);

    if matches!(
        best_kind,
        ObjectiveSectionKind::Checklist
            | ObjectiveSectionKind::Verification
            | ObjectiveSectionKind::Boilerplate
            | ObjectiveSectionKind::ToolingInstruction
    ) {
        return None;
    }

    let decisive_mission = best_kind == ObjectiveSectionKind::Mission
        && looks_like_goal_text(&best_text.to_ascii_lowercase());

    if decisive_mission {
        if best_score < 700 {
            return None;
        }
    } else if best_score < 650 || best_score - second_score < 100 {
        return None;
    }

    Some(best_text)
}

fn split_objective_sections(text: &str) -> Vec<ObjectiveSection> {
    let mut sections = Vec::new();
    let mut current_kind = ObjectiveSectionKind::Unknown;
    let mut current_body = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((kind, inline_body)) = parse_section_header(trimmed) {
            push_section(&mut sections, current_kind, &mut current_body);
            current_kind = kind;
            if let Some(inline_body) = inline_body {
                current_body.push_str(inline_body);
            }
            continue;
        }

        if !current_body.is_empty() {
            current_body.push('\n');
        }
        current_body.push_str(trimmed);
    }

    push_section(&mut sections, current_kind, &mut current_body);
    sections
}

fn parse_section_header(line: &str) -> Option<(ObjectiveSectionKind, Option<&str>)> {
    if line.starts_with("/goal ") {
        return Some((ObjectiveSectionKind::Mission, Some(line)));
    }

    if line.starts_with('#') {
        let title = line.trim_start_matches('#').trim();
        if !title.is_empty() {
            return Some((classify_section_label(title), None));
        }
    }

    let (label, remainder) = line.split_once(':')?;
    let label = label.trim();
    if label.is_empty() || label.split_whitespace().count() > 7 {
        return None;
    }

    let kind = classify_section_label(label);
    if matches!(kind, ObjectiveSectionKind::Unknown) && !remainder.trim().is_empty() {
        return None;
    }

    let inline_body = (!remainder.trim().is_empty()).then_some(remainder.trim());
    Some((kind, inline_body))
}

fn classify_section_label(label: &str) -> ObjectiveSectionKind {
    let lowered = label.to_ascii_lowercase();

    if matches!(
        lowered.as_str(),
        "scope"
            | "mission"
            | "objective"
            | "goal"
            | "concrete task ask"
            | "concrete workspace action request"
            | "workspace action request"
            | "task ask"
            | "task request"
    ) {
        return ObjectiveSectionKind::Mission;
    }

    if lowered.contains("checklist")
        || lowered.contains("steps")
        || lowered.contains("plan of attack")
    {
        return ObjectiveSectionKind::Checklist;
    }

    if lowered.contains("verify") || lowered.contains("validation") || lowered.contains("smoke") {
        return ObjectiveSectionKind::Verification;
    }

    if lowered.contains("constraint")
        || lowered.contains("boundary")
        || lowered.contains("guardrail")
        || lowered.contains("out of scope")
        || lowered.contains("execution rules")
        || lowered.contains("required review scope")
    {
        return ObjectiveSectionKind::Constraints;
    }

    if lowered.contains("return with")
        || lowered.contains("output")
        || lowered.contains("deliverable")
        || lowered.contains("report with")
    {
        return ObjectiveSectionKind::Deliverables;
    }

    if lowered.contains("context")
        || lowered.contains("read first")
        || lowered.contains("authoritative docs")
        || lowered.contains("files to inspect")
        || lowered.contains("project guidance")
    {
        return ObjectiveSectionKind::Context;
    }

    if lowered.contains("tooling")
        || lowered.contains("gitnexus")
        || lowered.contains("skill")
        || lowered.contains("plugin")
        || lowered.contains("connector")
    {
        return ObjectiveSectionKind::ToolingInstruction;
    }

    if lowered.contains("agents.md")
        || lowered.contains("permissions")
        || lowered.contains("memory")
        || lowered.contains("safety")
        || lowered.contains("policy")
        || lowered.contains("boilerplate")
    {
        return ObjectiveSectionKind::Boilerplate;
    }

    ObjectiveSectionKind::Unknown
}

fn push_section(
    sections: &mut Vec<ObjectiveSection>,
    kind: ObjectiveSectionKind,
    body: &mut String,
) {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        body.clear();
        return;
    }

    sections.push(ObjectiveSection {
        kind,
        body: trimmed.to_string(),
    });
    body.clear();
}

fn display_text_for_section(section: &ObjectiveSection) -> Option<String> {
    let body = section.body.trim();
    if body.is_empty() {
        return None;
    }

    if body.starts_with("/goal ") {
        return Some(body.to_string());
    }

    let best_line = body
        .lines()
        .map(normalize_clause_line)
        .filter(|line| !line.is_empty())
        .max_by_key(|line| clause_goal_score(line));

    match best_line {
        Some(line) if clause_goal_score(&line) >= 80 => Some(line),
        Some(_) if body.lines().count() == 1 => Some(body.to_string()),
        Some(_) => Some(body.lines().next()?.trim().to_string()),
        None => None,
    }
}

fn normalize_clause_line(line: &str) -> String {
    line.trim()
        .trim_start_matches(['-', '*', '•'])
        .trim_start_matches(|c: char| c.is_ascii_digit() || matches!(c, '.' | ')' | '('))
        .trim()
        .to_string()
}

fn clause_goal_score(line: &str) -> i32 {
    let lowered = line.to_ascii_lowercase();
    let mut score = 0;

    if lowered.starts_with("/goal ") {
        score += 1_000;
    }

    if looks_like_goal_text(&lowered) {
        score += 250;
    }

    if looks_like_constraint_text(&lowered) {
        score += 80;
    }

    if looks_like_deliverable_text(&lowered) {
        score += 60;
    }

    if looks_like_verification_text(&lowered) {
        score -= 200;
    }

    if looks_like_checklist_text(&lowered) {
        score -= 250;
    }

    if looks_like_boilerplate_text(&lowered) {
        score -= 350;
    }

    score + line.len().min(240) as i32
}

fn section_priority(section: &ObjectiveSection, display_text: &str) -> i32 {
    let lowered = display_text.to_ascii_lowercase();
    let mut score = match section.kind {
        ObjectiveSectionKind::Mission => 800,
        ObjectiveSectionKind::Constraints => 575,
        ObjectiveSectionKind::Deliverables => 525,
        ObjectiveSectionKind::Context => 250,
        ObjectiveSectionKind::Verification => 125,
        ObjectiveSectionKind::Checklist => 75,
        ObjectiveSectionKind::Boilerplate => -200,
        ObjectiveSectionKind::ToolingInstruction => -150,
        ObjectiveSectionKind::Unknown => 350,
    };

    if looks_like_goal_text(&lowered) {
        score += 250;
    }
    if looks_like_constraint_text(&lowered) {
        score += 80;
    }
    if looks_like_deliverable_text(&lowered) {
        score += 60;
    }
    if looks_like_verification_text(&lowered) {
        score -= 200;
    }
    if looks_like_checklist_text(&lowered) {
        score -= 250;
    }
    if looks_like_boilerplate_text(&lowered) {
        score -= 350;
    }

    score + display_text.len().min(240) as i32
}

fn looks_like_goal_text(text: &str) -> bool {
    [
        "/goal ",
        "implement ",
        "fix ",
        "add ",
        "update ",
        "review ",
        "debug ",
        "troubleshoot ",
        "determine whether",
        "compare ",
        "inspect ",
        "explain ",
        "analyze ",
        "plan ",
        "teach ",
        "keep ",
        "stop ",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_constraint_text(text: &str) -> bool {
    [
        "stay strictly",
        "do not",
        "without widening",
        "out of scope",
        "only",
        "must",
        "keep the work centered",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_deliverable_text(text: &str) -> bool {
    [
        "return with",
        "changed files",
        "tests run",
        "verification commands run",
        "residual risk",
        "recommended commit message",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_verification_text(text: &str) -> bool {
    text.contains("verify")
        || text.contains("verification")
        || text.contains("cargo test")
        || text.contains("cargo build")
}

fn looks_like_checklist_text(text: &str) -> bool {
    [
        "step ",
        "checklist",
        "run this task on",
        "after implementing",
        "start with",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_boilerplate_text(text: &str) -> bool {
    [
        "filesystem sandboxing defines",
        "approval policy is currently",
        "use memory by default",
        "tools are grouped by namespace",
        "apps (connectors)",
        "plugin instructions",
        "codex desktop context",
        "<skill>",
        "available skills",
        "agents.md instructions",
    ]
    .iter()
    .any(|needle| text.contains(needle))
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
