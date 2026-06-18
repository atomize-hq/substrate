use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, UserMessageRole};
use serde::{Deserialize, Serialize};

use crate::checkpoint::{
    Confidence, EvidenceRef, ObjectiveClass, ObjectiveConstraint, ObjectiveConstraintKind,
    ObjectiveEvidenceSpan, ObjectiveIntent, ObjectiveRole, ObjectiveSectionKind,
    ObjectiveSourceKind, ObjectiveTarget, ObjectiveTargetKind, ObjectiveUnknown,
    RequestedDeliverable, RequestedDeliverableKind, StructuredObjective, SuccessCondition,
};
use crate::context::directive_rows;

const MAX_OBJECTIVE_CANDIDATE_ROWS: usize = 5;

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

    fn with_structured(
        text: String,
        verification_commands: Vec<String>,
        evidence: Vec<EvidenceRef>,
        structured: StructuredObjective,
    ) -> Self {
        let comparison_key = text.clone();
        Self {
            text,
            comparison_key,
            structured: Some(structured),
            verification_commands,
            evidence,
        }
    }

    pub(crate) fn with_compatibility_display_from(&self, compatibility: &ObjectiveSummary) -> Self {
        let mut summary = self.clone();
        summary.text = compatibility.text.clone();
        summary.comparison_key = compatibility.comparison_key.clone();
        for command in &compatibility.verification_commands {
            if !summary.verification_commands.contains(command) {
                summary.verification_commands.push(command.clone());
            }
        }
        for evidence in &compatibility.evidence {
            if !summary.evidence.contains(evidence) {
                summary.evidence.push(evidence.clone());
            }
        }
        summary
    }
}

pub fn extract_objective(rows: &[CompactionRow]) -> ObjectiveSummary {
    let Some(decomposition) = decompose_objective_rows(rows) else {
        return ObjectiveSummary::compatibility(
            "No objective row available".to_string(),
            Vec::new(),
            Vec::new(),
        );
    };

    let compatibility = select_compatibility_text(&decomposition)
        .or_else(|| decomposition.primary_candidate_text())
        .unwrap_or_else(|| "No objective row available".to_string());
    let verification_commands = verification_commands_from_decomposition(&decomposition);
    let evidence = objective_summary_evidence(&decomposition, &compatibility);
    let structured = assemble_structured_objective(&decomposition, &verification_commands);

    ObjectiveSummary::with_structured(compatibility, verification_commands, evidence, structured)
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

#[derive(Debug, Clone)]
struct ObjectiveDecomposition {
    candidates: Vec<DirectiveRowCandidate>,
    sections: Vec<DecomposedObjectiveSection>,
    clauses: Vec<ObjectiveClause>,
}

impl ObjectiveDecomposition {
    fn primary_candidate_text(&self) -> Option<String> {
        self.candidates
            .first()
            .map(|candidate| candidate.text.trim().to_string())
            .filter(|text| !text.is_empty())
    }

    fn section_for_clause(&self, clause: &ObjectiveClause) -> Option<&DecomposedObjectiveSection> {
        self.sections.iter().find(|section| {
            section.candidate_index == clause.candidate_index
                && section.index == clause.section_index
        })
    }
}

#[derive(Debug, Clone)]
struct DirectiveRowCandidate {
    candidate_index: usize,
    row_ref: RowRef,
    source_kind: ObjectiveSourceKind,
    text: String,
    score: (i32, usize, usize),
}

#[derive(Debug, Clone)]
struct DecomposedObjectiveSection {
    candidate_index: usize,
    row_ref: RowRef,
    source_kind: ObjectiveSourceKind,
    index: usize,
    kind: ObjectiveSectionKind,
    body: String,
    confidence: Confidence,
}

#[derive(Debug, Clone)]
struct ObjectiveClause {
    candidate_index: usize,
    row_ref: RowRef,
    source_kind: ObjectiveSourceKind,
    section_index: usize,
    clause_index: usize,
    section_kind: ObjectiveSectionKind,
    text: String,
    role_candidates: Vec<RoleCandidate>,
}

#[derive(Debug, Clone)]
struct RoleCandidate {
    role: ObjectiveRole,
    confidence: Confidence,
    score: i32,
}

fn decompose_objective_rows(rows: &[CompactionRow]) -> Option<ObjectiveDecomposition> {
    let candidates = collect_directive_row_candidates(rows);
    if candidates.is_empty() {
        return None;
    }

    let mut sections = Vec::new();
    let mut clauses = Vec::new();
    for candidate in &candidates {
        let candidate_sections = split_objective_sections(candidate);
        for section in candidate_sections {
            let section_index = sections.len();
            let mut section = section;
            section.index = section_index;
            clauses.extend(split_section_into_clauses(&section));
            sections.push(section);
        }
    }

    if sections.is_empty() {
        return candidates.first().map(|candidate| ObjectiveDecomposition {
            candidates: candidates.clone(),
            sections: vec![DecomposedObjectiveSection {
                candidate_index: candidate.candidate_index,
                row_ref: candidate.row_ref.clone(),
                source_kind: candidate.source_kind,
                index: 0,
                kind: ObjectiveSectionKind::UnknownSection,
                body: candidate.text.trim().to_string(),
                confidence: Confidence::Low,
            }],
            clauses: Vec::new(),
        });
    }

    Some(ObjectiveDecomposition {
        candidates,
        sections,
        clauses,
    })
}

fn collect_directive_row_candidates(rows: &[CompactionRow]) -> Vec<DirectiveRowCandidate> {
    let mut candidates = rows
        .iter()
        .filter(|row| {
            matches!(
                row.kind,
                CompactionKind::UserMessage | CompactionKind::DeveloperMessage
            )
        })
        .filter(|row| !row.text.trim().is_empty())
        .map(|row| DirectiveRowCandidate {
            candidate_index: 0,
            row_ref: RowRef::from_row(row),
            source_kind: source_kind_for_row(row),
            text: row.text.clone(),
            score: objective_score(row),
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        candidates = directive_rows(rows)
            .filter(|row| !row.text.trim().is_empty())
            .map(|row| DirectiveRowCandidate {
                candidate_index: 0,
                row_ref: RowRef::from_row(row),
                source_kind: source_kind_for_row(row),
                text: row.text.clone(),
                score: objective_score(row),
            })
            .collect();
    }

    candidates.sort_by(|left, right| right.score.cmp(&left.score));
    candidates.truncate(MAX_OBJECTIVE_CANDIDATE_ROWS);
    for (index, candidate) in candidates.iter_mut().enumerate() {
        candidate.candidate_index = index;
    }
    candidates
}

fn source_kind_for_row(row: &CompactionRow) -> ObjectiveSourceKind {
    match row.kind {
        CompactionKind::UserMessage => {
            if row.text.trim_start().starts_with("/goal") {
                ObjectiveSourceKind::ThreadGoal
            } else {
                ObjectiveSourceKind::UserPrompt
            }
        }
        CompactionKind::DeveloperMessage | CompactionKind::SystemMessage => {
            ObjectiveSourceKind::SystemInstruction
        }
        CompactionKind::AssistantMessage | CompactionKind::Reasoning => {
            ObjectiveSourceKind::AssistantContext
        }
        CompactionKind::ToolCall
        | CompactionKind::ToolOutput
        | CompactionKind::Status
        | CompactionKind::Error => ObjectiveSourceKind::ToolOutput,
        CompactionKind::Unknown => ObjectiveSourceKind::UnknownSource,
    }
}

fn split_objective_sections(candidate: &DirectiveRowCandidate) -> Vec<DecomposedObjectiveSection> {
    let text = candidate.text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    let mut explicit_sections = Vec::<DecomposedObjectiveSection>::new();
    let mut current_kind = ObjectiveSectionKind::UnknownSection;
    let mut current_confidence = Confidence::Low;
    let mut current_body = String::new();
    let mut saw_header = false;

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            if !current_body.trim().is_empty() && !current_body.ends_with("\n\n") {
                current_body.push_str("\n\n");
            }
            continue;
        }

        if let Some((kind, confidence, inline_body)) = parse_section_header(trimmed) {
            saw_header = true;
            push_decomposed_section(
                &mut explicit_sections,
                candidate,
                current_kind,
                current_confidence,
                &mut current_body,
            );
            current_kind = kind;
            current_confidence = confidence;
            if let Some(inline_body) = inline_body {
                current_body.push_str(inline_body);
            }
            continue;
        }

        if !current_body.is_empty() && !current_body.ends_with('\n') {
            current_body.push('\n');
        }
        current_body.push_str(trimmed);
    }

    push_decomposed_section(
        &mut explicit_sections,
        candidate,
        current_kind,
        current_confidence,
        &mut current_body,
    );

    if saw_header && !explicit_sections.is_empty() {
        explicit_sections
    } else {
        vec![DecomposedObjectiveSection {
            candidate_index: candidate.candidate_index,
            row_ref: candidate.row_ref.clone(),
            source_kind: candidate.source_kind,
            index: 0,
            kind: synthetic_section_kind(text),
            body: text.to_string(),
            confidence: Confidence::Low,
        }]
    }
}

fn parse_section_header(line: &str) -> Option<(ObjectiveSectionKind, Confidence, Option<&str>)> {
    if line.starts_with("/goal ") {
        return Some((ObjectiveSectionKind::Mission, Confidence::High, Some(line)));
    }

    if line.starts_with('#') {
        let title = line.trim_start_matches('#').trim();
        if !title.is_empty() {
            let classified = classify_section_label(title);
            return Some((classified.kind, classified.confidence, None));
        }
    }

    let (label, remainder) = line.split_once(':')?;
    let label = label.trim();
    if label.is_empty() || label.split_whitespace().count() > 9 {
        return None;
    }

    let classified = classify_section_label(label);
    if matches!(classified.kind, ObjectiveSectionKind::UnknownSection)
        && !remainder.trim().is_empty()
    {
        return None;
    }

    let inline_body = (!remainder.trim().is_empty()).then_some(remainder.trim());
    Some((classified.kind, classified.confidence, inline_body))
}

#[derive(Debug, Clone, Copy)]
struct SectionClassification {
    kind: ObjectiveSectionKind,
    confidence: Confidence,
}

fn classify_section_label(label: &str) -> SectionClassification {
    let normalized = normalize_label(label);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    let classified = if matches_any_phrase(&normalized, &["scope", "task scope", "review scope"])
        || tokens.iter().any(|token| *token == "scope")
    {
        SectionClassification {
            kind: ObjectiveSectionKind::Scope,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "checklist",
            "steps",
            "plan of attack",
            "procedure",
            "sequence",
            "runbook",
            "implementation steps",
            "after that",
        ],
    ) {
        SectionClassification {
            kind: ObjectiveSectionKind::Checklist,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "verify",
            "verification",
            "validation",
            "smoke",
            "test",
            "proof",
            "acceptance",
            "verification wall",
            "commands",
            "checks",
            "expected green",
            "success criteria",
            "verification task",
        ],
    ) {
        SectionClassification {
            kind: ObjectiveSectionKind::Verification,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "constraint",
            "constraints",
            "boundary",
            "boundaries",
            "guardrail",
            "guardrails",
            "out of scope",
            "non goals",
            "non goal",
            "rules",
            "restrictions",
            "requirements",
            "must",
            "must not",
            "assumptions",
            "execution rules",
            "required review scope",
            "task constraints",
        ],
    ) {
        SectionClassification {
            kind: ObjectiveSectionKind::Constraints,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "return with",
            "output",
            "deliverable",
            "deliverables",
            "report with",
            "final response",
            "include",
            "produce",
            "write",
            "create docs",
            "summary",
            "output requirements",
            "output request",
        ],
    ) {
        SectionClassification {
            kind: ObjectiveSectionKind::Deliverables,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "mission",
            "objective",
            "goal",
            "main task",
            "primary task",
            "requested work",
            "what i need",
            "task summary",
            "work request",
            "desired outcome",
            "purpose",
            "goal for this packet",
            "concrete task ask",
            "concrete workspace action request",
            "workspace action request",
            "task ask",
            "task request",
        ],
    ) || tokens
        .iter()
        .any(|token| matches!(*token, "mission" | "objective" | "goal"))
    {
        SectionClassification {
            kind: ObjectiveSectionKind::Mission,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "context",
            "background",
            "read first",
            "relevant docs",
            "authoritative docs",
            "files to inspect",
            "current state",
            "repo reality",
            "project guidance",
        ],
    ) {
        SectionClassification {
            kind: ObjectiveSectionKind::Context,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "tooling",
            "gitnexus",
            "skill",
            "plugin",
            "connector",
            "connectors",
            "use tool",
            "use hf cli",
            "execution environment",
        ],
    ) {
        SectionClassification {
            kind: ObjectiveSectionKind::ToolingInstructions,
            confidence: Confidence::High,
        }
    } else if matches_any_phrase(
        &normalized,
        &[
            "agents md",
            "permissions",
            "memory",
            "safety",
            "policy",
            "boilerplate",
            "system",
            "developer instruction",
            "sandbox",
            "available skills",
            "capability",
            "tool namespace",
        ],
    ) {
        SectionClassification {
            kind: ObjectiveSectionKind::Boilerplate,
            confidence: Confidence::High,
        }
    } else {
        SectionClassification {
            kind: ObjectiveSectionKind::UnknownSection,
            confidence: Confidence::Low,
        }
    };

    classified
}

fn normalize_label(label: &str) -> String {
    label
        .trim()
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && !c.is_whitespace())
        .to_ascii_lowercase()
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matches_any_phrase(text: &str, phrases: &[&str]) -> bool {
    phrases
        .iter()
        .any(|phrase| text == *phrase || text.contains(phrase))
}

fn synthetic_section_kind(text: &str) -> ObjectiveSectionKind {
    let lowered = text.to_ascii_lowercase();
    if lowered.starts_with("/goal ") || looks_like_goal_text(&lowered) {
        ObjectiveSectionKind::Mission
    } else if looks_like_verification_text(&lowered) {
        ObjectiveSectionKind::Verification
    } else if looks_like_constraint_text(&lowered) {
        ObjectiveSectionKind::Constraints
    } else {
        ObjectiveSectionKind::UnknownSection
    }
}

fn push_decomposed_section(
    sections: &mut Vec<DecomposedObjectiveSection>,
    candidate: &DirectiveRowCandidate,
    kind: ObjectiveSectionKind,
    confidence: Confidence,
    body: &mut String,
) {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        body.clear();
        return;
    }

    sections.push(DecomposedObjectiveSection {
        candidate_index: candidate.candidate_index,
        row_ref: candidate.row_ref.clone(),
        source_kind: candidate.source_kind,
        index: 0,
        kind,
        body: trimmed.to_string(),
        confidence,
    });
    body.clear();
}

fn split_section_into_clauses(section: &DecomposedObjectiveSection) -> Vec<ObjectiveClause> {
    let mut clauses = Vec::new();

    for line in section.body.lines() {
        let normalized = normalize_clause_line(line);
        if normalized.is_empty() {
            continue;
        }

        for piece in split_clause_line(&normalized) {
            let text = piece.trim().to_string();
            if text.is_empty() {
                continue;
            }
            let role_candidates = role_candidates_for_clause(section.kind, &text);
            let clause_index = clauses.len();
            clauses.push(ObjectiveClause {
                candidate_index: section.candidate_index,
                row_ref: section.row_ref.clone(),
                source_kind: section.source_kind,
                section_index: section.index,
                clause_index,
                section_kind: section.kind,
                text,
                role_candidates,
            });
        }
    }

    if clauses.is_empty() && !section.body.trim().is_empty() {
        let text = section.body.trim().to_string();
        clauses.push(ObjectiveClause {
            candidate_index: section.candidate_index,
            row_ref: section.row_ref.clone(),
            source_kind: section.source_kind,
            section_index: section.index,
            clause_index: 0,
            section_kind: section.kind,
            text: text.clone(),
            role_candidates: role_candidates_for_clause(section.kind, &text),
        });
    }

    clauses
}

fn split_clause_line(line: &str) -> Vec<String> {
    let protected = line
        .replace(", do not ", ". Do not ")
        .replace(", must not ", ". Must not ")
        .replace(", and return ", ". Return ")
        .replace(", then return ", ". Return ")
        .replace(" and then ", ". Then ")
        .replace(" after that ", ". After that ");

    let mut pieces = Vec::new();
    let mut start = 0usize;
    let chars = protected.char_indices().collect::<Vec<_>>();
    for (index, (char_index, ch)) in chars.iter().enumerate() {
        let char_index = *char_index;
        let ch = *ch;
        let is_boundary = matches!(ch, '.' | '?' | '!' | ';')
            && chars
                .get(index + 1)
                .map(|(_, next)| (*next).is_whitespace())
                .unwrap_or(true);
        if is_boundary {
            let end = char_index + ch.len_utf8();
            let piece = protected[start..end].trim();
            if !piece.is_empty() {
                pieces.push(piece.to_string());
            }
            start = end;
        }
    }

    let rest = protected[start..].trim();
    if !rest.is_empty() {
        pieces.push(rest.to_string());
    }

    if pieces.is_empty() {
        vec![line.to_string()]
    } else {
        pieces
    }
}

fn role_candidates_for_clause(
    section_kind: ObjectiveSectionKind,
    text: &str,
) -> Vec<RoleCandidate> {
    let lowered = text.to_ascii_lowercase();
    let looks_like_constraint = looks_like_constraint_text(&lowered);
    let looks_like_verification = looks_like_verification_text(&lowered);
    let has_strong_constraint_cue = has_strong_constraint_cue(&lowered);
    let has_explicit_verification_cue = has_explicit_verification_cue(text, &lowered);
    let mut candidates = Vec::new();

    match section_kind {
        ObjectiveSectionKind::Scope | ObjectiveSectionKind::Mission => {
            if !has_strong_constraint_cue && !has_explicit_verification_cue {
                push_role_candidate(&mut candidates, ObjectiveRole::Goal, Confidence::High, 900);
            }
        }
        ObjectiveSectionKind::Constraints => {
            push_role_candidate(
                &mut candidates,
                ObjectiveRole::Constraint,
                Confidence::High,
                850,
            );
        }
        ObjectiveSectionKind::Verification => {
            push_role_candidate(
                &mut candidates,
                ObjectiveRole::Verification,
                Confidence::High,
                850,
            );
        }
        ObjectiveSectionKind::Context => {
            push_role_candidate(
                &mut candidates,
                ObjectiveRole::Context,
                Confidence::High,
                700,
            );
        }
        ObjectiveSectionKind::Deliverables => {
            push_role_candidate(
                &mut candidates,
                ObjectiveRole::OtherRole,
                Confidence::Medium,
                650,
            );
        }
        ObjectiveSectionKind::Checklist => {
            push_role_candidate(
                &mut candidates,
                ObjectiveRole::OtherRole,
                Confidence::Medium,
                400,
            );
        }
        ObjectiveSectionKind::Boilerplate | ObjectiveSectionKind::ToolingInstructions => {
            push_role_candidate(
                &mut candidates,
                ObjectiveRole::Context,
                Confidence::Medium,
                300,
            );
        }
        ObjectiveSectionKind::UnknownSection => {}
    }

    if has_explicit_verification_cue
        && matches!(
            section_kind,
            ObjectiveSectionKind::Scope
                | ObjectiveSectionKind::Mission
                | ObjectiveSectionKind::UnknownSection
        )
    {
        push_role_candidate(
            &mut candidates,
            ObjectiveRole::Verification,
            Confidence::High,
            825,
        );
    }

    if !matches!(
        section_kind,
        ObjectiveSectionKind::Checklist
            | ObjectiveSectionKind::Verification
            | ObjectiveSectionKind::Constraints
            | ObjectiveSectionKind::Deliverables
            | ObjectiveSectionKind::Context
            | ObjectiveSectionKind::ToolingInstructions
            | ObjectiveSectionKind::Boilerplate
    ) && (lowered.starts_with("/goal ") || looks_like_goal_text(&lowered))
    {
        push_role_candidate(
            &mut candidates,
            ObjectiveRole::Goal,
            Confidence::Medium,
            650,
        );
    }
    if looks_like_constraint {
        push_role_candidate(
            &mut candidates,
            ObjectiveRole::Constraint,
            Confidence::High,
            800,
        );
    }
    if looks_like_verification {
        push_role_candidate(
            &mut candidates,
            ObjectiveRole::Verification,
            Confidence::High,
            825,
        );
    }
    if looks_like_deliverable_text(&lowered) {
        push_role_candidate(
            &mut candidates,
            ObjectiveRole::OtherRole,
            Confidence::Medium,
            700,
        );
    }
    if looks_like_context_text(&lowered) || looks_like_boilerplate_text(&lowered) {
        push_role_candidate(
            &mut candidates,
            ObjectiveRole::Context,
            Confidence::Medium,
            525,
        );
    }

    if candidates.is_empty() {
        push_role_candidate(
            &mut candidates,
            ObjectiveRole::OtherRole,
            Confidence::Low,
            100,
        );
    }

    candidates.sort_by(|left, right| right.score.cmp(&left.score));
    candidates.dedup_by(|left, right| left.role == right.role);
    candidates
}

fn push_role_candidate(
    candidates: &mut Vec<RoleCandidate>,
    role: ObjectiveRole,
    confidence: Confidence,
    score: i32,
) {
    candidates.push(RoleCandidate {
        role,
        confidence,
        score,
    });
}

fn select_compatibility_text(decomposition: &ObjectiveDecomposition) -> Option<String> {
    let mut candidates = decomposition
        .clauses
        .iter()
        .filter_map(|clause| {
            let role = top_role(clause)?;
            if role.role != ObjectiveRole::Goal {
                return None;
            }
            let score = compatibility_score(decomposition, clause, role);
            Some((score, clause))
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    let selected = candidates.first().map(|(_, clause)| *clause)?;
    Some(compatibility_text_for_clause(decomposition, selected))
}

fn compatibility_score(
    decomposition: &ObjectiveDecomposition,
    clause: &ObjectiveClause,
    role: &RoleCandidate,
) -> i32 {
    let mut score = role.score;
    score += source_priority(clause.source_kind);
    score += section_goal_priority(clause.section_kind);

    let lowered = clause.text.to_ascii_lowercase();
    if lowered.starts_with("/goal ") {
        score += 500;
    }
    if looks_like_goal_text(&lowered) {
        score += 150;
    }
    if looks_like_checklist_text(&lowered) {
        score -= 300;
    }
    if looks_like_verification_text(&lowered) {
        score -= 350;
    }
    if looks_like_boilerplate_text(&lowered) && !explicitly_targets_instruction_surface(&lowered) {
        score -= 250;
    }
    if decomposition
        .section_for_clause(clause)
        .map(|section| section.confidence == Confidence::High)
        .unwrap_or(false)
    {
        score += 25;
    }

    score + clause.text.len().min(180) as i32
}

fn source_priority(source_kind: ObjectiveSourceKind) -> i32 {
    match source_kind {
        ObjectiveSourceKind::ThreadGoal => 500,
        ObjectiveSourceKind::UserPrompt => 450,
        ObjectiveSourceKind::AssistantContext => 75,
        ObjectiveSourceKind::UnknownSource => 0,
        ObjectiveSourceKind::SystemInstruction => -250,
        ObjectiveSourceKind::ToolOutput => -500,
    }
}

fn section_goal_priority(section_kind: ObjectiveSectionKind) -> i32 {
    match section_kind {
        ObjectiveSectionKind::Scope | ObjectiveSectionKind::Mission => 500,
        ObjectiveSectionKind::UnknownSection => 125,
        ObjectiveSectionKind::Deliverables => 25,
        ObjectiveSectionKind::Constraints => 0,
        ObjectiveSectionKind::Context => -50,
        ObjectiveSectionKind::Checklist => -250,
        ObjectiveSectionKind::Verification => -400,
        ObjectiveSectionKind::ToolingInstructions => -450,
        ObjectiveSectionKind::Boilerplate => -500,
    }
}

fn compatibility_text_for_clause(
    decomposition: &ObjectiveDecomposition,
    clause: &ObjectiveClause,
) -> String {
    if let Some(section) = decomposition.section_for_clause(clause) {
        let body = section.body.trim();
        if body.starts_with("/goal ") {
            return body.split("\n\n").next().unwrap_or(body).trim().to_string();
        }
    }
    clause.text.trim().to_string()
}

fn assemble_structured_objective(
    decomposition: &ObjectiveDecomposition,
    verification_commands: &[String],
) -> StructuredObjective {
    let goal_clause = selected_goal_clause(decomposition);
    let evidence_spans = evidence_spans_from_decomposition(decomposition);
    let target = goal_clause.and_then(explicit_target_for_clause);

    let constraints = decomposition
        .clauses
        .iter()
        .filter(|clause| top_role(clause).map(|role| role.role) == Some(ObjectiveRole::Constraint))
        .map(|clause| ObjectiveConstraint {
            display: clause.text.clone(),
            constraint_kind: constraint_kind_for_text(&clause.text),
            evidence: vec![evidence_span_for_clause(clause)],
            confidence: top_role(clause)
                .map(|role| role.confidence)
                .unwrap_or(Confidence::Low),
        })
        .collect::<Vec<_>>();

    let success_conditions = success_conditions_from_decomposition(decomposition, goal_clause);
    let deliverables = deliverables_from_decomposition(decomposition);
    let unknowns = unknowns_for_objective(goal_clause, target.as_ref(), &evidence_spans);
    let primary_intent = goal_clause
        .map(|clause| intent_for_text(&clause.text))
        .unwrap_or(ObjectiveIntent::OtherTask);
    let confidence = objective_confidence(goal_clause, &evidence_spans);

    StructuredObjective {
        objective_class: if goal_clause.is_some() {
            ObjectiveClass::TaskStatement
        } else {
            ObjectiveClass::NotTaskStatement
        },
        primary_intent,
        target,
        constraints,
        success_conditions,
        deliverables,
        verification_commands: verification_commands.to_vec(),
        evidence_spans,
        confidence,
        unknowns,
    }
}

fn selected_goal_clause(decomposition: &ObjectiveDecomposition) -> Option<&ObjectiveClause> {
    let mut candidates = decomposition
        .clauses
        .iter()
        .filter_map(|clause| {
            let role = top_role(clause)?;
            if role.role == ObjectiveRole::Goal {
                Some((compatibility_score(decomposition, clause, role), clause))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    candidates.first().map(|(_, clause)| *clause)
}

fn evidence_spans_from_decomposition(
    decomposition: &ObjectiveDecomposition,
) -> Vec<ObjectiveEvidenceSpan> {
    let mut spans = Vec::new();
    for clause in &decomposition.clauses {
        if let Some(role) = top_role(clause) {
            if role.role != ObjectiveRole::OtherRole
                || matches!(clause.section_kind, ObjectiveSectionKind::Deliverables)
            {
                spans.push(evidence_span_for_clause(clause));
            }
        }
    }
    spans
}

fn evidence_span_for_clause(clause: &ObjectiveClause) -> ObjectiveEvidenceSpan {
    let role = top_role(clause).cloned().unwrap_or(RoleCandidate {
        role: ObjectiveRole::OtherRole,
        confidence: Confidence::Low,
        score: 0,
    });
    ObjectiveEvidenceSpan {
        row: clause.row_ref.clone(),
        source_kind: clause.source_kind,
        section_kind: clause.section_kind,
        role: role.role,
        excerpt: clause.text.clone(),
        section_index: Some(clause.section_index),
        clause_index: Some(clause.clause_index),
        start_char: None,
        end_char: None,
        confidence: role.confidence,
    }
}

fn top_role(clause: &ObjectiveClause) -> Option<&RoleCandidate> {
    clause.role_candidates.first()
}

fn success_conditions_from_decomposition(
    decomposition: &ObjectiveDecomposition,
    goal_clause: Option<&ObjectiveClause>,
) -> Vec<SuccessCondition> {
    let mut conditions = Vec::new();
    for clause in &decomposition.clauses {
        let lowered = clause.text.to_ascii_lowercase();
        let verification_role =
            top_role(clause).map(|role| role.role) == Some(ObjectiveRole::Verification);
        if verification_role || lowered.contains("green") || lowered.contains("success") {
            conditions.push(SuccessCondition {
                display: clause.text.clone(),
                evidence: vec![evidence_span_for_clause(clause)],
                confidence: top_role(clause)
                    .map(|role| role.confidence)
                    .unwrap_or(Confidence::Low),
            });
        }
    }

    if conditions.is_empty() {
        if let Some(goal_clause) = goal_clause {
            let lowered = goal_clause.text.to_ascii_lowercase();
            if lowered.contains("ensure") || lowered.contains("validate") {
                conditions.push(SuccessCondition {
                    display: goal_clause.text.clone(),
                    evidence: vec![evidence_span_for_clause(goal_clause)],
                    confidence: Confidence::Medium,
                });
            }
        }
    }

    conditions
}

fn deliverables_from_decomposition(
    decomposition: &ObjectiveDecomposition,
) -> Vec<RequestedDeliverable> {
    decomposition
        .clauses
        .iter()
        .filter(|clause| {
            matches!(clause.section_kind, ObjectiveSectionKind::Deliverables)
                || looks_like_deliverable_text(&clause.text.to_ascii_lowercase())
        })
        .map(|clause| RequestedDeliverable {
            display: clause.text.clone(),
            deliverable_kind: deliverable_kind_for_text(&clause.text),
            evidence: vec![evidence_span_for_clause(clause)],
            confidence: Confidence::Medium,
        })
        .collect()
}

fn unknowns_for_objective(
    goal_clause: Option<&ObjectiveClause>,
    target: Option<&ObjectiveTarget>,
    evidence_spans: &[ObjectiveEvidenceSpan],
) -> Vec<ObjectiveUnknown> {
    let mut unknowns = Vec::new();
    if goal_clause.is_none() {
        unknowns.push(ObjectiveUnknown {
            field_name: "primary_goal".to_string(),
            reason: "no grounded goal clause was found".to_string(),
            evidence: evidence_spans.iter().take(1).cloned().collect(),
        });
    }
    if target.is_none() {
        unknowns.push(ObjectiveUnknown {
            field_name: "target".to_string(),
            reason: "target could not be inferred from explicit target evidence in a grounded goal clause".to_string(),
            evidence: evidence_spans.iter().take(1).cloned().collect(),
        });
    }
    unknowns
}

fn objective_confidence(
    goal_clause: Option<&ObjectiveClause>,
    evidence_spans: &[ObjectiveEvidenceSpan],
) -> Confidence {
    if let Some(goal_clause) = goal_clause {
        if matches!(
            goal_clause.section_kind,
            ObjectiveSectionKind::Scope | ObjectiveSectionKind::Mission
        ) && evidence_spans
            .iter()
            .any(|span| span.role == ObjectiveRole::Goal)
        {
            return Confidence::High;
        }
        return Confidence::Medium;
    }
    Confidence::Low
}

fn verification_commands_from_decomposition(decomposition: &ObjectiveDecomposition) -> Vec<String> {
    let mut commands = Vec::new();
    for clause in &decomposition.clauses {
        if top_role(clause).map(|role| role.role) == Some(ObjectiveRole::Verification) {
            for command in extract_verification_commands(&clause.text) {
                if !commands.iter().any(|existing| existing == &command) {
                    commands.push(command);
                }
            }
        }
    }

    if commands.is_empty() {
        for candidate in &decomposition.candidates {
            for command in extract_verification_commands(&candidate.text) {
                if !commands.iter().any(|existing| existing == &command) {
                    commands.push(command);
                }
            }
        }
    }

    commands
}

fn objective_summary_evidence(
    decomposition: &ObjectiveDecomposition,
    compatibility: &str,
) -> Vec<EvidenceRef> {
    let Some(selected_clause) = decomposition
        .clauses
        .iter()
        .find(|clause| clause.text == compatibility || compatibility.contains(&clause.text))
        .or_else(|| selected_goal_clause(decomposition))
    else {
        return decomposition
            .candidates
            .first()
            .map(|candidate| EvidenceRef {
                row: candidate.row_ref.clone(),
                reason: "literal objective row".to_string(),
            })
            .into_iter()
            .collect();
    };

    vec![EvidenceRef {
        row: selected_clause.row_ref.clone(),
        reason: format!(
            "structured objective {:?} clause in {:?} section",
            top_role(selected_clause)
                .map(|role| role.role)
                .unwrap_or(ObjectiveRole::OtherRole),
            selected_clause.section_kind
        ),
    }]
}

fn target_display_for_goal(goal: &str) -> String {
    goal.trim()
        .trim_start_matches("/goal")
        .trim_start_matches(|c: char| c == ':' || c.is_whitespace())
        .trim()
        .to_string()
}

#[derive(Debug, Clone)]
struct ExplicitTargetAnchor {
    display: String,
    kind: ObjectiveTargetKind,
    paths: Vec<String>,
    named_artifacts: Vec<String>,
    workspace_refs: Vec<String>,
}

fn explicit_target_for_clause(clause: &ObjectiveClause) -> Option<ObjectiveTarget> {
    let anchor = explicit_target_anchor_for_text(&clause.text)?;
    Some(ObjectiveTarget {
        display: anchor.display,
        kind: anchor.kind,
        paths: anchor.paths,
        symbols: Vec::new(),
        named_artifacts: anchor.named_artifacts,
        workspace_refs: anchor.workspace_refs,
        evidence: vec![evidence_span_for_clause(clause)],
        confidence: top_role(clause)
            .map(|role| role.confidence)
            .unwrap_or(Confidence::Low),
    })
}

fn explicit_target_anchor_for_text(text: &str) -> Option<ExplicitTargetAnchor> {
    let goal = target_display_for_goal(text);
    if goal.is_empty() {
        return None;
    }

    let lowered = goal.to_ascii_lowercase();
    if is_insufficient_target_reference(&lowered) {
        return None;
    }

    let named_artifacts = extract_named_artifacts(&goal);
    if !named_artifacts.is_empty() {
        return Some(ExplicitTargetAnchor {
            display: named_artifacts.join(", "),
            kind: ObjectiveTargetKind::SkillOrInstructionSurface,
            paths: extract_inline_paths(&goal),
            named_artifacts,
            workspace_refs: extract_workspace_refs(&goal),
        });
    }

    let paths = extract_inline_paths(&goal);
    if let Some(path) = paths.first() {
        let kind = if looks_like_doc_path(path) {
            ObjectiveTargetKind::SpecOrDesignDoc
        } else {
            ObjectiveTargetKind::FileOrDirectory
        };
        return Some(ExplicitTargetAnchor {
            display: path.clone(),
            kind,
            paths,
            named_artifacts: Vec::new(),
            workspace_refs: extract_workspace_refs(&goal),
        });
    }

    let workspace_refs = extract_workspace_refs(&goal);
    if let Some(workspace_ref) = workspace_refs.first() {
        return Some(ExplicitTargetAnchor {
            display: workspace_ref.clone(),
            kind: ObjectiveTargetKind::RepoSlice,
            paths: Vec::new(),
            named_artifacts: Vec::new(),
            workspace_refs,
        });
    }

    if let Some(identifier) = extract_work_item_identifier(&goal) {
        return Some(ExplicitTargetAnchor {
            display: identifier.clone(),
            kind: ObjectiveTargetKind::RepoSlice,
            paths: Vec::new(),
            named_artifacts: vec![identifier],
            workspace_refs: Vec::new(),
        });
    }

    if let Some(crate_or_package) = extract_crate_or_package_target(&goal) {
        return Some(ExplicitTargetAnchor {
            display: crate_or_package.clone(),
            kind: ObjectiveTargetKind::CrateOrPackage,
            paths: Vec::new(),
            named_artifacts: vec![crate_or_package],
            workspace_refs: Vec::new(),
        });
    }

    if let Some(doc_target) = extract_named_doc_target(&goal) {
        return Some(ExplicitTargetAnchor {
            display: doc_target.clone(),
            kind: ObjectiveTargetKind::SpecOrDesignDoc,
            paths: Vec::new(),
            named_artifacts: vec![doc_target],
            workspace_refs: Vec::new(),
        });
    }

    if let Some(test_target) = extract_named_test_or_verifier_target(&goal) {
        return Some(ExplicitTargetAnchor {
            display: test_target.clone(),
            kind: ObjectiveTargetKind::TestOrVerifier,
            paths: Vec::new(),
            named_artifacts: vec![test_target],
            workspace_refs: Vec::new(),
        });
    }

    let conceptual_target = extract_named_conceptual_target(&goal)?;
    Some(ExplicitTargetAnchor {
        display: conceptual_target.clone(),
        kind: target_kind_for_text(&conceptual_target),
        paths: Vec::new(),
        named_artifacts: vec![conceptual_target],
        workspace_refs: Vec::new(),
    })
}

fn target_kind_for_text(text: &str) -> ObjectiveTargetKind {
    let lowered = text.to_ascii_lowercase();
    if explicitly_targets_instruction_surface(&lowered) {
        ObjectiveTargetKind::SkillOrInstructionSurface
    } else if lowered.contains(".rs")
        || lowered.contains(".md")
        || lowered.contains("/")
        || lowered.contains("\\")
    {
        ObjectiveTargetKind::FileOrDirectory
    } else if lowered.contains("crate") || lowered.contains("package") {
        ObjectiveTargetKind::CrateOrPackage
    } else if lowered.contains("spec") || lowered.contains("design") || lowered.contains("doc") {
        ObjectiveTargetKind::SpecOrDesignDoc
    } else if lowered.contains("test")
        || lowered.contains("verifier")
        || lowered.contains("validation")
    {
        ObjectiveTargetKind::TestOrVerifier
    } else {
        ObjectiveTargetKind::ConceptualTopic
    }
}

fn looks_like_doc_path(path: &str) -> bool {
    let lowered = path.to_ascii_lowercase();
    lowered.ends_with(".md")
        || lowered.contains("/docs/")
        || lowered.starts_with("docs/")
        || lowered.contains("/specs/")
        || lowered.starts_with("specs/")
}

fn is_insufficient_target_reference(text: &str) -> bool {
    matches!(
        text.trim(),
        "this" | "it" | "the above" | "what landed" | "the current issue"
    )
}

fn extract_work_item_identifier(text: &str) -> Option<String> {
    cleaned_target_tokens(text)
        .into_iter()
        .find(|token| looks_like_work_item_identifier(token))
}

fn looks_like_work_item_identifier(token: &str) -> bool {
    let cleaned = clean_target_token(token);
    let prefix = cleaned
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .collect::<String>();
    !cleaned.is_empty()
        && !cleaned.contains('/')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_uppercase())
        && prefix != "V"
        && cleaned.chars().any(|ch| ch.is_ascii_alphabetic())
        && cleaned.chars().any(|ch| ch.is_ascii_digit())
        && (cleaned.contains('-') || cleaned.contains('.'))
}

fn extract_crate_or_package_target(text: &str) -> Option<String> {
    extract_neighbor_target_for_cues(text, &["crate", "package"])
}

fn extract_named_doc_target(text: &str) -> Option<String> {
    let tokens = cleaned_target_tokens(text);
    let lowered = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();

    for (index, token) in lowered.iter().enumerate() {
        if !matches!(
            token.as_str(),
            "spec" | "specs" | "design" | "doc" | "docs" | "plan" | "plans" | "tasks"
        ) {
            continue;
        }

        if let Some(previous) = index
            .checked_sub(1)
            .and_then(|prev| tokens.get(prev))
            .filter(|candidate| looks_like_explicit_named_target(candidate))
        {
            return Some(format!("{previous} {}", normalize_target_kind_label(token)));
        }

        if let Some(next) = tokens
            .get(index + 1)
            .filter(|candidate| looks_like_explicit_named_target(candidate))
        {
            return Some(format!("{} {next}", normalize_target_kind_label(token)));
        }
    }

    None
}

fn extract_named_test_or_verifier_target(text: &str) -> Option<String> {
    let tokens = cleaned_target_tokens(text);
    let lowered = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();

    for (index, token) in lowered.iter().enumerate() {
        if !matches!(
            token.as_str(),
            "test" | "tests" | "verifier" | "verification" | "harness" | "suite"
        ) {
            continue;
        }

        if let Some(previous) = index
            .checked_sub(1)
            .and_then(|prev| tokens.get(prev))
            .filter(|candidate| looks_like_explicit_named_target(candidate))
        {
            return Some(format!("{previous} {}", normalize_target_kind_label(token)));
        }

        if let Some(next) = tokens
            .get(index + 1)
            .filter(|candidate| looks_like_explicit_named_target(candidate))
        {
            return Some(format!("{} {next}", normalize_target_kind_label(token)));
        }
    }

    None
}

fn normalize_target_kind_label(token: &str) -> &str {
    match token {
        "specs" => "spec",
        "docs" => "doc",
        "plans" => "plan",
        "tests" => "test",
        _ => token,
    }
}

fn extract_neighbor_target_for_cues(text: &str, cues: &[&str]) -> Option<String> {
    let tokens = cleaned_target_tokens(text);
    let lowered = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();

    for (index, token) in lowered.iter().enumerate() {
        if !cues.iter().any(|cue| cue == token) {
            continue;
        }

        if let Some(next) = tokens
            .get(index + 1)
            .filter(|candidate| looks_like_explicit_named_target(candidate))
        {
            return Some(next.clone());
        }

        if let Some(previous) = index
            .checked_sub(1)
            .and_then(|prev| tokens.get(prev))
            .filter(|candidate| looks_like_explicit_named_target(candidate))
        {
            return Some(previous.clone());
        }
    }

    None
}

fn extract_named_conceptual_target(text: &str) -> Option<String> {
    let tokens = cleaned_target_tokens(text);
    let lowered = tokens
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();

    for (index, token) in lowered.iter().enumerate() {
        if !matches!(
            token.as_str(),
            "sidecar"
                | "extractor"
                | "system"
                | "architecture"
                | "migration"
                | "harness"
                | "schema"
                | "pipeline"
                | "workflow"
        ) {
            continue;
        }

        let mut start = index.saturating_sub(3);
        while start < index
            && matches!(
                lowered[start].as_str(),
                "the"
                    | "a"
                    | "an"
                    | "this"
                    | "that"
                    | "review"
                    | "analyze"
                    | "fix"
                    | "validate"
                    | "implement"
                    | "determine"
                    | "inspect"
                    | "compare"
                    | "explain"
                    | "plan"
                    | "debug"
                    | "troubleshoot"
                    | "perform"
                    | "ensure"
                    | "keep"
                    | "stop"
                    | "add"
                    | "update"
            )
        {
            start += 1;
        }

        if start >= index {
            continue;
        }

        let phrase = tokens[start..=index].join(" ");
        if phrase.split_whitespace().count() >= 2
            && !is_insufficient_target_reference(&phrase.to_ascii_lowercase())
        {
            return Some(phrase);
        }
    }

    None
}

fn cleaned_target_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(clean_target_token)
        .filter(|token| !token.is_empty())
        .collect()
}

fn clean_target_token(token: &str) -> String {
    token
        .trim_matches(|c: char| {
            matches!(
                c,
                ',' | '.' | ';' | ':' | '`' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}'
            )
        })
        .to_string()
}

fn looks_like_explicit_named_target(token: &str) -> bool {
    let cleaned = clean_target_token(token);
    if cleaned.is_empty() {
        return false;
    }

    let lowered = cleaned.to_ascii_lowercase();
    if is_insufficient_target_reference(&lowered) {
        return false;
    }

    cleaned.starts_with('@')
        || cleaned.contains('/')
        || cleaned.contains('\\')
        || cleaned.contains("::")
        || cleaned.contains('_')
        || cleaned.ends_with(".md")
        || cleaned.ends_with(".rs")
        || looks_like_work_item_identifier(&cleaned)
        || cleaned.chars().filter(|ch| ch.is_ascii_uppercase()).count() >= 2
        || (cleaned.contains('-') && cleaned.chars().any(|ch| ch.is_ascii_alphabetic()))
}

fn intent_for_text(text: &str) -> ObjectiveIntent {
    let lowered = text.to_ascii_lowercase();
    if contains_any(
        &lowered,
        &["implement", "add ", "update", "wire", "land ", "build"],
    ) {
        ObjectiveIntent::Implement
    } else if contains_any(&lowered, &["debug", "fix", "troubleshoot"]) {
        ObjectiveIntent::Debug
    } else if contains_any(
        &lowered,
        &[
            "review",
            "inspect",
            "determine whether",
            "compare",
            "analyze",
        ],
    ) {
        ObjectiveIntent::Review
    } else if contains_any(&lowered, &["research", "look up", "survey"]) {
        ObjectiveIntent::Research
    } else if contains_any(&lowered, &["plan", "design", "spec"]) {
        ObjectiveIntent::Plan
    } else if contains_any(&lowered, &["validate", "verify", "ensure", "green", "test"]) {
        ObjectiveIntent::Validate
    } else if contains_any(&lowered, &["docs", "document", "readme"]) {
        ObjectiveIntent::Docs
    } else {
        ObjectiveIntent::OtherTask
    }
}

fn constraint_kind_for_text(text: &str) -> ObjectiveConstraintKind {
    let lowered = text.to_ascii_lowercase();
    if contains_any(&lowered, &["no code", "do not change code", "no-code"]) {
        ObjectiveConstraintKind::NoCode
    } else if contains_any(&lowered, &["docs only", "docs-only"]) {
        ObjectiveConstraintKind::DocsOnly
    } else if contains_any(&lowered, &["review only", "review-only"]) {
        ObjectiveConstraintKind::ReviewOnly
    } else if contains_any(&lowered, &["validate only", "validate-only"]) {
        ObjectiveConstraintKind::ValidateOnly
    } else if contains_any(&lowered, &["linux", "macos", "windows"]) {
        ObjectiveConstraintKind::PlatformBoundary
    } else if contains_any(&lowered, &["return with", "output", "format"]) {
        ObjectiveConstraintKind::DeliverableFormat
    } else if contains_any(&lowered, &["only", "scope", "boundary", "out of scope"]) {
        ObjectiveConstraintKind::ScopeBoundary
    } else {
        ObjectiveConstraintKind::OtherConstraint
    }
}

fn deliverable_kind_for_text(text: &str) -> RequestedDeliverableKind {
    let lowered = text.to_ascii_lowercase();
    if contains_any(&lowered, &["design doc", "design"]) {
        RequestedDeliverableKind::DesignDoc
    } else if contains_any(&lowered, &["plan", "tasks"]) {
        RequestedDeliverableKind::Plan
    } else if contains_any(&lowered, &["review", "findings"]) {
        RequestedDeliverableKind::Review
    } else if contains_any(&lowered, &["validation", "verify", "tests run"]) {
        RequestedDeliverableKind::ValidationReport
    } else if contains_any(&lowered, &["research", "summary"]) {
        RequestedDeliverableKind::ResearchSummary
    } else if contains_any(&lowered, &["code", "patch", "changed files"]) {
        RequestedDeliverableKind::CodeChange
    } else {
        RequestedDeliverableKind::OtherDeliverable
    }
}

fn extract_inline_paths(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token.trim_matches(|c: char| matches!(c, ',' | '.' | ';' | ':' | '`' | '"' | '\''))
        })
        .filter(|token| looks_like_repo_path_token(token))
        .map(ToString::to_string)
        .collect()
}

fn looks_like_repo_path_token(token: &str) -> bool {
    if token.ends_with(".rs")
        || token.ends_with(".md")
        || token.ends_with(".toml")
        || token.ends_with(".json")
    {
        return true;
    }

    if token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
        || token.contains('\\')
    {
        return true;
    }

    if !token.contains('/') {
        return false;
    }

    let segments = token
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    segments.len() >= 3
        || segments.iter().any(|segment| {
            segment.contains('.')
                || segment.contains('_')
                || segment.contains('-')
                || segment.chars().any(|ch| ch.is_ascii_digit())
        })
}

fn extract_named_artifacts(text: &str) -> Vec<String> {
    let mut artifacts = Vec::new();
    let lowered = text.to_ascii_lowercase();
    for (needle, display) in [
        ("agents.md", "AGENTS.md"),
        ("<skill>", "<skill>"),
        ("available skills", "Available skills"),
        ("plugin instructions", "plugin instructions"),
        ("apps (connectors)", "Apps (Connectors)"),
        ("codex desktop context", "Codex desktop context"),
        ("safety guardrails", "safety guardrails"),
        ("tooling boilerplate", "tooling boilerplate"),
        ("instruction block", "instruction block"),
        ("instructions block", "instructions block"),
    ] {
        if lowered.contains(needle) && !artifacts.iter().any(|existing| existing == display) {
            artifacts.push(display.to_string());
        }
    }
    artifacts
}

fn extract_workspace_refs(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|token| token.starts_with('@'))
        .map(|token| {
            token
                .trim_matches(|c: char| matches!(c, ',' | '.' | ';' | ':'))
                .to_string()
        })
        .collect()
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn normalize_clause_line(line: &str) -> String {
    line.trim()
        .trim_start_matches(['-', '*', '•'])
        .trim_start_matches(|c: char| c.is_ascii_digit() || matches!(c, '.' | ')' | '('))
        .trim()
        .to_string()
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
        "ensure ",
        "validate ",
        "keep ",
        "stop ",
        "perform ",
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
        "must not",
        "keep the work centered",
        "no code",
        "review-only",
        "docs-only",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_deliverable_text(text: &str) -> bool {
    [
        "return with",
        "return ",
        "provide ",
        "changed files",
        "tests run",
        "verification commands run",
        "residual risk",
        "recommended commit message",
        "final response",
        "output requirements",
        "report with",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_context_text(text: &str) -> bool {
    [
        "read first",
        "authoritative docs",
        "files to inspect",
        "context",
        "background",
        "current state",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn looks_like_verification_text(text: &str) -> bool {
    text.contains("verify")
        || text.contains("verification")
        || text.contains("cargo test")
        || text.contains("cargo build")
        || text.contains("pytest")
        || text.contains("pnpm test")
        || text.contains("npm test")
        || text.contains("vitest")
}

fn has_explicit_verification_cue(text: &str, lowered: &str) -> bool {
    lowered.starts_with("verify ")
        || lowered.starts_with("verify with ")
        || lowered.starts_with("verification ")
        || lowered.starts_with("validation ")
        || lowered.starts_with("smoke ")
        || lowered.starts_with("test ")
        || !extract_verification_commands(text).is_empty()
}

fn has_strong_constraint_cue(text: &str) -> bool {
    [
        "stay strictly",
        "do not",
        "must",
        "must not",
        "keep the work centered",
        "no code",
        "review-only",
        "docs-only",
        "out of scope",
    ]
    .iter()
    .any(|needle| text.starts_with(needle))
}

fn looks_like_checklist_text(text: &str) -> bool {
    [
        "step ",
        "checklist",
        "run this task on",
        "after implementing",
        "start with",
        "first ",
        "then ",
        "after that",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn explicitly_targets_instruction_surface(text: &str) -> bool {
    contains_any(
        text,
        &[
            "agents.md",
            "<skill>",
            "available skills",
            "instruction block",
            "instructions block",
            "plugin instructions",
            "apps (connectors)",
            "codex desktop context",
            "safety guardrails",
            "tooling boilerplate",
        ],
    )
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
    for command in extract_backticked_commands(text) {
        if is_verification_command(&command)
            && !commands.iter().any(|existing| existing == &command)
        {
            commands.push(command);
        }
    }

    for line in text.lines() {
        for command in extract_verification_commands_from_line(line) {
            if !commands.iter().any(|existing| existing == &command) {
                commands.push(command);
            }
        }
    }

    commands
}

fn extract_backticked_commands(text: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining.find('`') {
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        let candidate = clean_command_candidate(&after_start[..end]);
        if !candidate.is_empty() {
            commands.push(candidate);
        }
        remaining = &after_start[end + 1..];
    }
    commands
}

fn extract_verification_commands_from_line(line: &str) -> Vec<String> {
    let cleaned_line = normalize_clause_line(line)
        .trim_start_matches('`')
        .trim_end_matches('`')
        .trim()
        .to_string();
    if cleaned_line.is_empty() {
        return Vec::new();
    }

    let lowered = cleaned_line.to_ascii_lowercase();
    let mut commands = Vec::new();
    let mut used_ranges = Vec::<(usize, usize)>::new();
    for &prefix in VERIFICATION_COMMAND_PREFIXES {
        let mut search_start = 0usize;
        while search_start < lowered.len() {
            let Some(relative_index) = lowered[search_start..].find(prefix) else {
                break;
            };
            let start = search_start + relative_index;
            if used_ranges
                .iter()
                .any(|(range_start, range_end)| start >= *range_start && start < *range_end)
            {
                search_start = start + prefix.len();
                continue;
            }
            if !is_command_boundary(&lowered, start) {
                search_start = start + prefix.len();
                continue;
            }
            let raw_candidate = &cleaned_line[start..];
            let command_slice = take_one_command(raw_candidate);
            let candidate = clean_command_candidate(command_slice);
            if is_verification_command(&candidate)
                && !commands.iter().any(|existing| existing == &candidate)
            {
                used_ranges.push((start, start + command_slice.len()));
                commands.push(candidate);
            }
            search_start = start + prefix.len();
        }
    }
    commands
}

fn take_one_command(candidate: &str) -> &str {
    let mut end = candidate.len();
    for delimiter in [",", ";", "`", " && ", " || ", " and "] {
        if let Some(index) = candidate.find(delimiter) {
            end = end.min(index);
        }
    }
    &candidate[..end]
}

fn clean_command_candidate(candidate: &str) -> String {
    let mut cleaned = candidate
        .trim()
        .trim_start_matches("verify with ")
        .trim_start_matches("Verify with ")
        .trim_start_matches("verify: ")
        .trim_start_matches("Verify: ")
        .trim_start_matches("run ")
        .trim_start_matches("Run ")
        .trim_matches(|c: char| matches!(c, '`' | '"' | '\''))
        .trim_end_matches(|c: char| matches!(c, ':' | ';' | ','))
        .trim()
        .to_string();
    if cleaned.ends_with('.') && !cleaned.ends_with("/...") {
        cleaned.pop();
    }
    cleaned
}

fn is_command_boundary(text: &str, start: usize) -> bool {
    if start == 0 {
        return true;
    }
    text[..start]
        .chars()
        .next_back()
        .map(|ch| ch.is_whitespace() || matches!(ch, '`' | ':' | '-' | '*' | '(' | '['))
        .unwrap_or(true)
}

fn is_verification_command(command: &str) -> bool {
    let lowered = command.trim().to_ascii_lowercase();
    VERIFICATION_COMMAND_PREFIXES
        .iter()
        .any(|prefix| lowered == *prefix || lowered.starts_with(&format!("{prefix} ")))
}

const VERIFICATION_COMMAND_PREFIXES: &[&str] = &[
    "python -m pytest",
    "cargo fmt --check",
    "cargo test",
    "cargo check",
    "cargo build",
    "cargo clippy",
    "npm run typecheck",
    "npm run lint",
    "npm run test",
    "npm test",
    "pnpm exec vitest",
    "pnpm run lint",
    "pnpm run test",
    "pnpm test",
    "yarn run lint",
    "yarn run test",
    "yarn lint",
    "yarn test",
    "npx vitest",
    "bun test",
    "go test",
    "make test",
    "just test",
    "pytest",
    "vitest",
];

#[cfg(test)]
mod tests {
    use super::*;
    use agent_session_compactor::SourceKind;
    use camino::Utf8PathBuf;

    fn test_row(text: &str) -> CompactionRow {
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/objective-test/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 0,
            line_number: 1,
            row_ordinal: 0,
            timestamp: None,
            kind: CompactionKind::UserMessage,
            user_message_role: Some(UserMessageRole::Prompt),
            dedupe_identity: None,
            text: text.to_string(),
            canonical_text: text.to_string(),
            text_hash_hex: format!("hash-{}", text.split_whitespace().collect::<String>()),
        }
    }

    fn status_row(text: &str) -> CompactionRow {
        CompactionRow {
            kind: CompactionKind::Status,
            user_message_role: None,
            ..test_row(text)
        }
    }

    #[test]
    fn decomposes_dense_single_paragraph_into_goal_constraint_and_deliverable_clauses() {
        let rows = vec![test_row(
            "Review SO-2.1, do not change code, identify brittle gaps, and return concrete packet fixes.",
        )];
        let summary = extract_objective(&rows);
        let Some(structured) = summary.structured else {
            panic!("structured sidecar")
        };
        assert_eq!(structured.objective_class, ObjectiveClass::TaskStatement);
        assert!(
            structured
                .evidence_spans
                .iter()
                .any(|span| span.role == ObjectiveRole::Goal
                    && span.excerpt.contains("Review SO-2.1"))
        );
        assert!(structured
            .constraints
            .iter()
            .any(|constraint| constraint.display.contains("Do not change code")));
        assert!(structured
            .deliverables
            .iter()
            .any(|deliverable| deliverable.display.contains("Return concrete packet fixes")));
    }

    #[test]
    fn recognizes_unlisted_mission_headings_without_promoting_checklists() {
        let rows = vec![test_row(
            "## What I need\nEnsure the analyzer objective system extracts structured objective fields from long prompts.\n\n## Steps\n- Run this task on a linux machine.",
        )];
        let summary = extract_objective(&rows);
        assert_eq!(
            summary.text,
            "Ensure the analyzer objective system extracts structured objective fields from long prompts."
        );
        let Some(structured) = summary.structured else {
            panic!("structured sidecar")
        };
        assert!(structured.evidence_spans.iter().any(|span| {
            span.role == ObjectiveRole::Goal
                && matches!(span.section_kind, ObjectiveSectionKind::Mission)
                && span.section_index == Some(0)
                && span.clause_index == Some(0)
        }));
        assert!(!summary.text.contains("Run this task on a linux machine"));
    }

    #[test]
    fn keeps_verification_command_as_verification_not_goal() {
        let rows = vec![test_row(
            "## Scope\nValidate the structured objective sidecar.\n\n## Verification\n- cargo test -p agent-drift-analyzer checkpoints -- --nocapture",
        )];
        let summary = extract_objective(&rows);
        assert_eq!(summary.text, "Validate the structured objective sidecar.");
        assert_eq!(
            summary.verification_commands,
            vec!["cargo test -p agent-drift-analyzer checkpoints -- --nocapture"]
        );
        let Some(structured) = summary.structured else {
            panic!("structured sidecar")
        };
        assert!(structured.evidence_spans.iter().any(|span| {
            span.role == ObjectiveRole::Verification
                && span.excerpt.contains("cargo test -p agent-drift-analyzer")
                && span.section_index == Some(1)
                && span.clause_index == Some(0)
        }));
        assert!(!structured
            .target
            .as_ref()
            .map(|target| target.display.contains("cargo test"))
            .unwrap_or(false));
    }

    #[test]
    fn preserves_user_requested_instruction_surface_targets() {
        let rows = vec![test_row(
            "Review this AGENTS.md instruction block and tell me whether it should change.\n\n# AGENTS.md\nUse the incremental implementation skill.",
        )];
        let summary = extract_objective(&rows);
        let Some(structured) = summary.structured else {
            panic!("structured sidecar")
        };
        let Some(target) = structured.target else {
            panic!("target")
        };
        assert_eq!(target.kind, ObjectiveTargetKind::SkillOrInstructionSurface);
        assert!(summary
            .text
            .contains("Review this AGENTS.md instruction block"));
    }

    #[test]
    fn classifies_status_rows_as_non_objective_tool_output_context() {
        let row = status_row("task_complete: success=true");
        assert_eq!(source_kind_for_row(&row), ObjectiveSourceKind::ToolOutput);
    }
}
