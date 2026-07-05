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
        let comparison_key = comparison_key_from_structured(&structured);
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
        if summary.structured.is_none() || summary.comparison_key.is_empty() {
            summary.comparison_key = compatibility.comparison_key.clone();
        }
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

    let verification_commands = verification_commands_from_decomposition(&decomposition);
    let structured = assemble_structured_objective(&decomposition, &verification_commands);
    let compatibility = compatibility_text_from_structured(&structured)
        .or_else(|| select_compatibility_text(&decomposition))
        .or_else(|| decomposition.primary_candidate_text())
        .unwrap_or_else(|| "No objective row available".to_string());
    let evidence = objective_summary_evidence(&decomposition, &compatibility);

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

    let mut decomposition = ObjectiveDecomposition {
        candidates,
        sections,
        clauses,
    };
    inject_structural_goal_if_absent(&mut decomposition);
    Some(decomposition)
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
        || tokens.contains(&"scope")
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
            let role_candidates =
                role_candidates_for_clause(section.kind, section.source_kind, &text);
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
            role_candidates: role_candidates_for_clause(section.kind, section.source_kind, &text),
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
    source_kind: ObjectiveSourceKind,
    text: &str,
) -> Vec<RoleCandidate> {
    // Issue 3: only genuine user/goal surfaces may earn a `Goal` role. System/developer/tool rows
    // (skill catalogs, memory policy, permission blocks, AGENTS scaffolding) can carry goal-shaped
    // verbs ("review", "ensure", "validate") but are never the user's ask, so they must not be
    // promoted to `Goal` even when their phrasing matches `looks_like_goal_text`.
    let goal_eligible_source = matches!(
        source_kind,
        ObjectiveSourceKind::ThreadGoal | ObjectiveSourceKind::UserPrompt
    );
    let lowered = text.to_ascii_lowercase();
    let looks_like_constraint = looks_like_constraint_text(&lowered);
    let looks_like_verification = looks_like_verification_text(&lowered);
    let has_strong_constraint_cue = has_strong_constraint_cue(&lowered);
    let has_explicit_verification_cue = has_explicit_verification_cue(text, &lowered);
    let has_mixed_goal_and_verification_cue = looks_like_goal_text(&lowered)
        && has_explicit_verification_cue
        && (text.contains(',') || lowered.contains(" and ") || lowered.contains(" then "));
    let mut candidates = Vec::new();

    match section_kind {
        ObjectiveSectionKind::Scope | ObjectiveSectionKind::Mission => {
            if goal_eligible_source
                && !has_strong_constraint_cue
                && (!has_explicit_verification_cue || has_mixed_goal_and_verification_cue)
            {
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

    if goal_eligible_source
        && !matches!(
            section_kind,
            ObjectiveSectionKind::Checklist
                | ObjectiveSectionKind::Verification
                | ObjectiveSectionKind::Constraints
                | ObjectiveSectionKind::Deliverables
                | ObjectiveSectionKind::Context
                | ObjectiveSectionKind::ToolingInstructions
                | ObjectiveSectionKind::Boilerplate
        )
        && (lowered.starts_with("/goal ") || looks_like_goal_text(&lowered))
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

fn compatibility_text_from_structured(structured: &StructuredObjective) -> Option<String> {
    if structured.objective_class != ObjectiveClass::TaskStatement
        || structured
            .unknowns
            .iter()
            .any(|unknown| unknown.field_name == "primary_goal")
    {
        return None;
    }

    let projected = structured
        .evidence_spans
        .iter()
        .filter(|span| span.role == ObjectiveRole::Goal)
        .max_by_key(|span| structured_compatibility_score(span))
        .map(|span| structured_compatibility_text_for_goal_span(structured, span))?;

    (!projected.is_empty()).then(|| projected.to_string())
}

fn structured_compatibility_text_for_goal_span(
    structured: &StructuredObjective,
    goal_span: &ObjectiveEvidenceSpan,
) -> String {
    let mut projected = goal_span.excerpt.trim().to_string();
    if !projected.starts_with("/goal ") {
        return projected;
    }

    let Some(section_index) = goal_span.section_index else {
        return projected;
    };
    let Some(goal_clause_index) = goal_span.clause_index else {
        return projected;
    };

    let inline_verification = structured
        .evidence_spans
        .iter()
        .filter(|span| {
            span.role == ObjectiveRole::Verification
                && span.row == goal_span.row
                && span.section_index == Some(section_index)
        })
        .filter_map(|span| {
            let clause_index = span.clause_index?;
            (clause_index > goal_clause_index).then_some((clause_index, span.excerpt.trim()))
        })
        .collect::<Vec<_>>();

    if inline_verification.is_empty() {
        return projected;
    }

    for (_, excerpt) in inline_verification {
        if excerpt.is_empty() {
            continue;
        }
        projected.push(' ');
        projected.push_str(excerpt);
    }

    projected
}

fn structured_compatibility_score(span: &ObjectiveEvidenceSpan) -> i32 {
    let mut score = source_priority(span.source_kind) + section_goal_priority(span.section_kind);
    score += match span.confidence {
        Confidence::High => 200,
        Confidence::Medium => 100,
        Confidence::Low => 0,
    };

    let lowered = span.excerpt.to_ascii_lowercase();
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

    score + span.excerpt.len().min(180) as i32
}

fn comparison_key_from_structured(structured: &StructuredObjective) -> String {
    if structured.objective_class != ObjectiveClass::TaskStatement {
        return "not_task_statement".to_string();
    }

    let mut parts = vec![comparison_key_intent(structured.primary_intent).to_string()];
    if let Some(target) = &structured.target {
        push_unique_comparison_segments(&mut parts, comparison_key_segments_for_target(target));
    } else {
        push_unique_comparison_segment(&mut parts, "unknown_target".to_string());
    }

    for constraint in &structured.constraints {
        push_unique_comparison_segments(
            &mut parts,
            comparison_key_segments_for_constraint(constraint),
        );
    }

    for success_condition in &structured.success_conditions {
        push_unique_comparison_segments(
            &mut parts,
            comparison_key_segments_for_success_condition(success_condition),
        );
    }

    parts.join("|")
}

fn comparison_key_intent(intent: ObjectiveIntent) -> &'static str {
    match intent {
        ObjectiveIntent::Implement => "implement",
        ObjectiveIntent::Debug => "debug",
        ObjectiveIntent::Review => "review",
        ObjectiveIntent::Research => "research",
        ObjectiveIntent::Plan => "plan",
        ObjectiveIntent::Validate => "validate",
        ObjectiveIntent::Docs => "docs",
        ObjectiveIntent::OtherTask => "other_task",
    }
}

fn comparison_key_target_kind(kind: ObjectiveTargetKind) -> &'static str {
    match kind {
        ObjectiveTargetKind::RepoSlice => "repo_slice",
        ObjectiveTargetKind::CrateOrPackage => "crate_or_package",
        ObjectiveTargetKind::FileOrDirectory => "file_or_directory",
        ObjectiveTargetKind::SpecOrDesignDoc => "spec_or_design_doc",
        ObjectiveTargetKind::TestOrVerifier => "test_or_verifier",
        ObjectiveTargetKind::SkillOrInstructionSurface => "skill_or_instruction_surface",
        ObjectiveTargetKind::ExternalArtifact => "external_artifact",
        ObjectiveTargetKind::ConceptualTopic => "conceptual_topic",
        ObjectiveTargetKind::UnknownTarget => "unknown_target",
    }
}

fn comparison_key_segments_for_target(target: &ObjectiveTarget) -> Vec<String> {
    let mut segments = vec![comparison_key_target_kind(target.kind).to_string()];

    // R6-3.5 defense-in-depth: even if a junk specific slipped past extraction, it must not seed a
    // comparison segment. Drop `Junk`-quality specifics (log/coordinate/model noise); keep typed
    // and plausible artifacts (paths, workspace refs, instruction surfaces).
    let mut specifics = target
        .paths
        .iter()
        .chain(target.workspace_refs.iter())
        .chain(target.named_artifacts.iter())
        .filter(|value| anchor_quality(value) != TargetAnchorQuality::Junk)
        .map(|value| normalize_comparison_key_segment(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if specifics.is_empty() {
        let display = normalize_comparison_key_segment(&target.display);
        if !display.is_empty()
            && display != comparison_key_target_kind(target.kind)
            && is_stable_goal_term(&display)
        {
            specifics.push(display);
        }
    }

    specifics.sort();
    specifics.dedup();
    segments.extend(specifics);
    segments
}

fn comparison_key_segments_for_constraint(constraint: &ObjectiveConstraint) -> Vec<String> {
    if !constraint
        .evidence
        .iter()
        .any(comparison_key_allows_constraint_evidence)
    {
        return Vec::new();
    }

    match constraint.constraint_kind {
        ObjectiveConstraintKind::NoCode => vec!["no_code".to_string()],
        ObjectiveConstraintKind::DocsOnly => vec!["docs_only".to_string()],
        ObjectiveConstraintKind::ReviewOnly => vec!["review_only".to_string()],
        ObjectiveConstraintKind::ValidateOnly => vec!["validate_only".to_string()],
        ObjectiveConstraintKind::PlatformBoundary => {
            comparison_key_platform_segments(&constraint.display)
        }
        ObjectiveConstraintKind::ScopeBoundary
        | ObjectiveConstraintKind::DeliverableFormat
        | ObjectiveConstraintKind::OtherConstraint => Vec::new(),
    }
}

fn comparison_key_allows_constraint_evidence(span: &ObjectiveEvidenceSpan) -> bool {
    matches!(
        span.section_kind,
        ObjectiveSectionKind::Scope
            | ObjectiveSectionKind::Mission
            | ObjectiveSectionKind::Constraints
            | ObjectiveSectionKind::UnknownSection
    )
}

fn comparison_key_platform_segments(text: &str) -> Vec<String> {
    let lowered = text.to_ascii_lowercase();
    let mut segments = Vec::new();
    for platform in ["linux", "macos", "windows"] {
        if lowered.contains(platform) {
            segments.push(platform.to_string());
        }
    }
    segments
}

fn comparison_key_segments_for_success_condition(
    success_condition: &SuccessCondition,
) -> Vec<String> {
    let lowered = success_condition.display.to_ascii_lowercase();
    if contains_any(&lowered, &["green", "clean"]) {
        vec!["green".to_string()]
    } else if contains_any(&lowered, &["success", "succeed", "pass"]) {
        vec!["success".to_string()]
    } else {
        Vec::new()
    }
}

fn push_unique_comparison_segment(parts: &mut Vec<String>, segment: String) {
    if !segment.is_empty() && !parts.iter().any(|existing| existing == &segment) {
        parts.push(segment);
    }
}

fn push_unique_comparison_segments(parts: &mut Vec<String>, segments: Vec<String>) {
    for segment in segments {
        push_unique_comparison_segment(parts, segment);
    }
}

fn normalize_comparison_key_segment(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for ch in value.trim().chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch);
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }

    normalized.trim_matches('_').to_string()
}

fn assemble_structured_objective(
    decomposition: &ObjectiveDecomposition,
    verification_commands: &[String],
) -> StructuredObjective {
    let goal_clause = selected_goal_clause(decomposition);
    let active_index = goal_clause.map(|clause| clause.candidate_index);
    let evidence_spans = evidence_spans_from_decomposition(decomposition, active_index);
    let target = goal_clause.and_then(explicit_target_for_clause);

    let constraints = decomposition
        .clauses
        .iter()
        .filter(|clause| clause_is_on_active_goal_surface(clause, goal_clause))
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
    let deliverables = deliverables_from_decomposition(decomposition, goal_clause);
    let mut unknowns = unknowns_for_objective(goal_clause, target.as_ref(), &evidence_spans);
    append_weak_field_unknowns(
        &mut unknowns,
        decomposition,
        goal_clause,
        &success_conditions,
        &deliverables,
    );
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
    // Issues 1/2/3: the goal must come from the active mission surface, never from a pasted
    // scaffold row that happens to contain a keyword-matched goal clause. Exclude boilerplate
    // candidates (the row scorer already drives their `objective_score` negative — pasted AGENTS.md /
    // `<skill>` bodies) so that, e.g. on `019eb47f`, the system-instruction and pasted-skill rows can
    // never supply the goal, while a real "/goal update the <skill> section" ask (positive score)
    // still can. Among the surviving (non-boilerplate) candidates the best clause-level
    // `compatibility_score` picks the owning row, so a concrete steer ("add this skill to
    // @shared-cab-app") still outranks an earlier pasted skill template.
    let mut candidates = decomposition
        .clauses
        .iter()
        .filter(|clause| !candidate_is_boilerplate_surface(decomposition, clause.candidate_index))
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

/// A boilerplate surface is a candidate row the row scorer penalized below zero: pasted AGENTS.md
/// instructions, `<skill>` bodies, available-skills catalogs, and the like. They must never own the
/// goal or contribute evidence spans. Using `objective_score`'s sign (rather than text markers) keeps
/// a genuine "/goal update the AGENTS.md / `<skill>` section" ask — which scores positive — eligible.
fn candidate_is_boilerplate_surface(
    decomposition: &ObjectiveDecomposition,
    candidate_index: usize,
) -> bool {
    decomposition
        .candidates
        .get(candidate_index)
        .map(|candidate| candidate.score.0 < 0)
        .unwrap_or(false)
}

/// When no non-boilerplate row carries a keyword-matched goal clause (its phrasing misses
/// `looks_like_goal_text`, e.g. "use the $x skill to evaluate if what landed is correct"), promote the
/// top genuine user surface's primary actionable clause to `Goal`. This anchors the structured
/// objective to the real ask instead of leaving it goalless or letting a boilerplate row win, while
/// skipping system/boilerplate surfaces so pasted scaffolding can never become the goal.
fn inject_structural_goal_if_absent(decomposition: &mut ObjectiveDecomposition) {
    if selected_goal_clause(decomposition).is_some() {
        return;
    }
    let Some(active_index) = structural_goal_surface_index(decomposition) else {
        return;
    };
    let Some(position) = structural_goal_clause_position(decomposition, active_index) else {
        return;
    };
    let role_candidates = &mut decomposition.clauses[position].role_candidates;
    push_role_candidate(
        role_candidates,
        ObjectiveRole::Goal,
        Confidence::Medium,
        650,
    );
    role_candidates.sort_by(|left, right| right.score.cmp(&left.score));
    role_candidates.dedup_by(|left, right| left.role == right.role);
}

/// The surface that may carry a synthesized structural goal: the highest-scored genuine user/goal row
/// that the row scorer did not penalize below zero. Candidates are pre-sorted by `objective_score`
/// (descending), so the first match is the highest-scored eligible user surface.
fn structural_goal_surface_index(decomposition: &ObjectiveDecomposition) -> Option<usize> {
    decomposition
        .candidates
        .iter()
        .find(|candidate| {
            candidate.score.0 >= 0
                && matches!(
                    candidate.source_kind,
                    ObjectiveSourceKind::ThreadGoal | ObjectiveSourceKind::UserPrompt
                )
        })
        .map(|candidate| candidate.candidate_index)
}

/// The active surface's primary actionable clause for structural-goal promotion: the first clause
/// that is not a checklist/verification/constraint/deliverable/boilerplate/tooling line **and** that
/// states a request action. Requiring a recognized action verb keeps vague prompts ("look at the
/// stuff above and make it better") and questions ("which docs own the acceptance wall?") honest as
/// `NotTaskStatement` instead of fabricating a goal, while still anchoring a real ask whose phrasing
/// missed the goal-keyword heuristics ("use the $x skill to evaluate if what landed is correct").
fn structural_goal_clause_position(
    decomposition: &ObjectiveDecomposition,
    active_index: usize,
) -> Option<usize> {
    decomposition.clauses.iter().position(|clause| {
        clause.candidate_index == active_index
            && !matches!(
                clause.section_kind,
                ObjectiveSectionKind::Checklist
                    | ObjectiveSectionKind::Verification
                    | ObjectiveSectionKind::Constraints
                    | ObjectiveSectionKind::Deliverables
                    | ObjectiveSectionKind::Boilerplate
                    | ObjectiveSectionKind::ToolingInstructions
            )
            && !matches!(
                top_role(clause).map(|role| role.role),
                Some(ObjectiveRole::Constraint) | Some(ObjectiveRole::Verification)
            )
            && clause_states_a_request_action(&clause.text)
    })
}

/// Whether a clause contains a recognizable request-action **verb** (used as an action, not an
/// incidental noun). Deliberately excludes noun-prone tokens such as `docs`, `tests`, `plan`, and
/// `spec` so a question like "which docs own the acceptance wall?" is not mistaken for an ask. This is
/// the concreteness gate for synthesizing a structural goal when keyword heuristics missed the ask.
fn clause_states_a_request_action(text: &str) -> bool {
    const REQUEST_ACTION_VERBS: &[&str] = &[
        "implement",
        "fix",
        "debug",
        "troubleshoot",
        "repair",
        "diagnose",
        "review",
        "inspect",
        "audit",
        "evaluate",
        "assess",
        "analyze",
        "compare",
        "determine",
        "validate",
        "verify",
        "ensure",
        "confirm",
        "investigate",
        "explore",
        "research",
        "refactor",
        "migrate",
        "integrate",
        "build",
        "create",
        "add",
        "update",
        "wire",
        "land",
        "perform",
    ];
    action_word_tokens(text)
        .iter()
        .any(|token| REQUEST_ACTION_VERBS.contains(&token.as_str()))
}

fn evidence_spans_from_decomposition(
    decomposition: &ObjectiveDecomposition,
    active_index: Option<usize>,
) -> Vec<ObjectiveEvidenceSpan> {
    // Issue 2/3: ground evidence spans to the selected goal's surface instead of pooling every clause
    // in the session. This is what keeps `Goal` spans (and the rest) off system-instruction and
    // pasted-skill-body rows — the structured sidecar reflects the real ask, not the scaffolding. When
    // no goal anchored (`active_index` is `None`), no spans are emitted rather than pooling boilerplate.
    decomposition
        .clauses
        .iter()
        .filter(|clause| Some(clause.candidate_index) == active_index)
        .flat_map(evidence_spans_for_clause)
        .collect()
}

fn evidence_spans_for_clause(clause: &ObjectiveClause) -> Vec<ObjectiveEvidenceSpan> {
    clause
        .role_candidates
        .iter()
        .filter(|role| {
            role.role != ObjectiveRole::OtherRole
                || matches!(clause.section_kind, ObjectiveSectionKind::Deliverables)
        })
        .map(|role| evidence_span_for_role(clause, role))
        .collect()
}

fn evidence_span_for_clause(clause: &ObjectiveClause) -> ObjectiveEvidenceSpan {
    let role = top_role(clause).cloned().unwrap_or(RoleCandidate {
        role: ObjectiveRole::OtherRole,
        confidence: Confidence::Low,
        score: 0,
    });
    evidence_span_for_role(clause, &role)
}

fn evidence_span_for_role(clause: &ObjectiveClause, role: &RoleCandidate) -> ObjectiveEvidenceSpan {
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

fn clause_has_role_candidate(clause: &ObjectiveClause, role: ObjectiveRole) -> bool {
    clause
        .role_candidates
        .iter()
        .any(|candidate| candidate.role == role)
}

fn success_conditions_from_decomposition(
    decomposition: &ObjectiveDecomposition,
    goal_clause: Option<&ObjectiveClause>,
) -> Vec<SuccessCondition> {
    let mut conditions = Vec::new();
    for clause in &decomposition.clauses {
        if !clause_is_on_active_goal_surface(clause, goal_clause) {
            continue;
        }
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
    goal_clause: Option<&ObjectiveClause>,
) -> Vec<RequestedDeliverable> {
    decomposition
        .clauses
        .iter()
        .filter(|clause| clause_is_on_active_goal_surface(clause, goal_clause))
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

/// A clause is on the active objective surface only when it is grounded to the selected goal's own
/// directive row and is not boilerplate/tooling/system scaffolding. Encodes the architecture's
/// Grounding Rules + "What Counts As Semantic State": success/deliverable/constraint state must come
/// from the user's goal surface, not from skill/memory/safety/tooling rows that happen to be pooled
/// into the same decomposition.
fn clause_is_on_active_goal_surface(
    clause: &ObjectiveClause,
    goal_clause: Option<&ObjectiveClause>,
) -> bool {
    let Some(goal_clause) = goal_clause else {
        return false;
    };
    clause.candidate_index == goal_clause.candidate_index
        && !matches!(
            clause.section_kind,
            ObjectiveSectionKind::Boilerplate | ObjectiveSectionKind::ToolingInstructions
        )
        && !matches!(
            clause.source_kind,
            ObjectiveSourceKind::SystemInstruction | ObjectiveSourceKind::ToolOutput
        )
}

/// Off-surface clauses whose success/proof phrasing was deliberately rejected, so the field can be
/// honestly recorded as unknown rather than fabricated ("Unknowns Are Success, Not Failure").
fn rejected_success_condition_spans(
    decomposition: &ObjectiveDecomposition,
    goal_clause: Option<&ObjectiveClause>,
) -> Vec<ObjectiveEvidenceSpan> {
    decomposition
        .clauses
        .iter()
        .filter(|clause| !clause_is_on_active_goal_surface(clause, goal_clause))
        .filter(|clause| {
            let lowered = clause.text.to_ascii_lowercase();
            clause_has_role_candidate(clause, ObjectiveRole::Verification)
                || lowered.contains("green")
                || lowered.contains("success")
        })
        .map(evidence_span_for_clause)
        .collect()
}

fn rejected_deliverable_spans(
    decomposition: &ObjectiveDecomposition,
    goal_clause: Option<&ObjectiveClause>,
) -> Vec<ObjectiveEvidenceSpan> {
    decomposition
        .clauses
        .iter()
        .filter(|clause| !clause_is_on_active_goal_surface(clause, goal_clause))
        .filter(|clause| {
            matches!(clause.section_kind, ObjectiveSectionKind::Deliverables)
                || looks_like_deliverable_text(&clause.text.to_ascii_lowercase())
        })
        .map(evidence_span_for_clause)
        .collect()
}

/// Record symmetric unknowns when a field's only supporting cue came from off-surface boilerplate /
/// scaffolding (seen but rejected), instead of fabricating the field. A field with no cue at all
/// stays simply empty to avoid spurious unknowns.
fn append_weak_field_unknowns(
    unknowns: &mut Vec<ObjectiveUnknown>,
    decomposition: &ObjectiveDecomposition,
    goal_clause: Option<&ObjectiveClause>,
    success_conditions: &[SuccessCondition],
    deliverables: &[RequestedDeliverable],
) {
    if success_conditions.is_empty() {
        let rejected = rejected_success_condition_spans(decomposition, goal_clause);
        if !rejected.is_empty() {
            unknowns.push(ObjectiveUnknown {
                field_name: "success_conditions".to_string(),
                reason: "success/proof phrasing appeared only in non-goal boilerplate or scaffolding rows, not in a grounded goal-surface clause".to_string(),
                evidence: rejected.into_iter().take(2).collect(),
            });
        }
    }
    if deliverables.is_empty() {
        let rejected = rejected_deliverable_spans(decomposition, goal_clause);
        if !rejected.is_empty() {
            unknowns.push(ObjectiveUnknown {
                field_name: "deliverables".to_string(),
                reason: "deliverable phrasing appeared only in non-goal boilerplate or scaffolding rows, not in a grounded goal-surface clause".to_string(),
                evidence: rejected.into_iter().take(2).collect(),
            });
        }
    }
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
        if !clause_has_role_candidate(clause, ObjectiveRole::Verification) {
            continue;
        }

        for command in extract_verification_commands(&clause.text) {
            if !commands.iter().any(|existing| existing == &command) {
                commands.push(command);
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

    // R6-3.5 (codex review): a bare well-known rootless file (`README`, `Makefile`) or a Rust symbol
    // ref (`foo::bar`) is a durable typed anchor even without a path or a doc/crate cue. Without this
    // fallback `update README` loses its structured target, which also makes the opaque-parent
    // guardrail suppress genuine drift. Scoped to the unambiguous grammars only — bare kebab package
    // names are deliberately excluded here to avoid promoting ordinary hyphenated prose.
    if let Some((token, kind)) = cleaned_target_tokens(&goal).into_iter().find_map(|token| {
        if validates_well_known_rootless_file(&token) || validates_rust_symbol_ref(&token) {
            stable_target_anchor_kind(&token).map(|kind| (token, kind))
        } else {
            None
        }
    }) {
        return Some(ExplicitTargetAnchor {
            display: token.clone(),
            kind,
            paths: Vec::new(),
            named_artifacts: vec![token],
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
        .find(|token| is_stable_work_item_identifier(token))
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

    // R6-3.5: never treat log/coordinate noise or bare model/version tokens (`GPT-5.4`, `v2.3.1`)
    // as a named target — the `>= 2 uppercase` / `contains('-')` rules below would otherwise admit
    // them. The work-item grammar is the model-aware `is_stable_work_item_identifier`.
    if is_variable_noise_token(&cleaned) || is_model_or_version_token(&cleaned) {
        return false;
    }

    cleaned.starts_with('@')
        || cleaned.contains('/')
        || cleaned.contains('\\')
        || cleaned.contains("::")
        || cleaned.contains('_')
        || cleaned.ends_with(".md")
        || cleaned.ends_with(".rs")
        || is_stable_work_item_identifier(&cleaned)
        || cleaned.chars().filter(|ch| ch.is_ascii_uppercase()).count() >= 2
        || (cleaned.contains('-') && cleaned.chars().any(|ch| ch.is_ascii_alphabetic()))
}

/// Bare model/assistant name (`GPT-5.4`, `claude-3`, `o4-mini`) or bare version token (`v2.3.1`,
/// `1.2.0`). Runtime metadata, never a task target. Matches model names by prefix — including the
/// alphanumeric `o1`/`o3`/`o4` families that a `take_while(alpha)` prefix scan would miss — where the
/// name is followed by a version-ish continuation and the token carries a digit.
fn is_model_or_version_token(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    if lower.chars().any(|c| c.is_ascii_digit()) {
        for model in MODEL_NAME_PREFIXES {
            if let Some(rest) = lower.strip_prefix(model) {
                if rest.is_empty() {
                    return true;
                }
                // Short `o1`/`o3`/`o4` families collide with ordinary identifiers, so a bare digit
                // continuation (`o40`) or a recognized variant/version component after a separator
                // (`o4-mini`, `o4_mini`, `o4.5`) is model metadata, while a separator into an
                // arbitrary word (`o4_router`, `o4-gateway`) is a symbol. Distinctive prefixes
                // (`gpt`, `claude`, …) keep the looser separator rule (codex re-review 2 + 3).
                let separator_ok = if AMBIGUOUS_SHORT_MODEL_PREFIXES.contains(model) {
                    rest.starts_with(|c: char| c.is_ascii_digit())
                        || ambiguous_short_model_rest_is_variant(rest)
                } else {
                    rest.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '-' | '.' | '_'))
                };
                if separator_ok {
                    return true;
                }
            }
        }
    }
    // Version token: `v2.3.1` / `1.2.0`, and the underscore-normalized comparison-key spelling
    // `v2_3_1` (both `.` and `_` are version separators here) so extraction and the scorer agree on
    // the same junk (codex re-review 3).
    let body = lower.strip_prefix('v').unwrap_or(&lower);
    !body.is_empty()
        && body
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '_'))
        && body.chars().any(|c| matches!(c, '.' | '_'))
        && body.chars().any(|c| c.is_ascii_digit())
}

// ---------------------------------------------------------------------------
// R6-3.5 shared target-anchor taxonomy (see
// docs/specs/r6/R6-3.5/agent-drift-analyzer-objective-target-hygiene-spec.md).
//
// A single deterministic classifier shared by target extraction (this module) and the
// semantic-goal-drift scorer, so junk classification cannot diverge between the two layers.
// Grammar-first: a token that validates a typed anchor grammar is `Stable` and is never re-masked
// as noise; otherwise the log-template variable masks decide `Junk` vs (plausible-but-untyped)
// `Weak`. Only `Stable` tokens may seed target specifics / comparison_key / scorer term sets.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetAnchorQuality {
    Stable,
    Weak,
    Junk,
}

/// File extensions recognized as a durable file/doc anchor leaf.
const RECOGNIZED_ANCHOR_EXTENSIONS: &[&str] = &[
    ".rs", ".md", ".toml", ".json", ".yaml", ".yml", ".py", ".ts", ".tsx", ".js", ".jsx", ".sh",
    ".lock", ".html", ".txt", ".cfg", ".rlib",
];

/// Conventional repository top-level directories accepted as a path root even without a recognized
/// leaf extension. Broadened per the codex review so legit roots (`benches/`, `examples/`,
/// `.github/`, …) are not dropped. The analyzer processes bundles from many repos, so this is a
/// portable convention list, not the current repo's `ls`.
const CONVENTIONAL_REPO_ROOTS: &[&str] = &[
    "crates", "src", "lib", "libs", "app", "apps", "pkg", "pkgs", "packages", "cmd", "internal",
    "docs", "doc", "spec", "specs", "test", "tests", "bench", "benches", "example", "examples",
    "fixtures", "proto", "schema", "scripts", "tools", "config", ".github", ".claude", ".codex",
];

/// Extension-less filenames that are still durable named artifacts. Deliberately excludes stems that
/// double as ordinary repo vocabulary (`cargo`, `agents`, `claude`): the real files carry extensions
/// (`Cargo.toml`, `AGENTS.md`, `CLAUDE.md`) and are already anchored by the extension/instruction
/// paths, so listing the bare stems here only mis-anchored plain prose (codex re-review finding 3).
const WELL_KNOWN_ROOTLESS_FILES: &[&str] = &[
    "readme",
    "makefile",
    "dockerfile",
    "license",
    "changelog",
    "contributing",
    "security",
    "justfile",
    "taskfile",
];

/// Known model / assistant name prefixes. A token that is a model name followed by a version
/// (`GPT-5.4`, `claude-3`, `o4-mini`) is runtime metadata, not a task target. Domain gazetteer,
/// analogous to a log-parser variable dictionary. Deliberately excludes tokens that collide with
/// this repo's own vocabulary (`codex`, `command`).
const MODEL_NAME_PREFIXES: &[&str] = &[
    "gpt", "claude", "gemini", "llama", "mistral", "qwen", "deepseek", "grok", "sonnet", "opus",
    "haiku", "o1", "o3", "o4", "phi", "gemma",
];

/// Model families short enough to collide with ordinary identifiers (`o4_router`); a separator into
/// one of these families only reads as model metadata when the trailing component is a recognized
/// variant/version. See `is_model_or_version_token` and `ambiguous_short_model_rest_is_variant`.
const AMBIGUOUS_SHORT_MODEL_PREFIXES: &[&str] = &["o1", "o3", "o4"];

/// Recognized model variant words that disambiguate a short-family suffix (`o4-mini`, `o4_mini`)
/// from an arbitrary symbol (`o4_router`). Kept small and specific to real model variants.
const MODEL_VARIANT_SUFFIXES: &[&str] = &[
    "mini", "preview", "pro", "high", "turbo", "nano", "max", "instruct", "chat", "latest",
    "vision",
];

/// For a short-family remainder (begins with a separator), the trailing component is model metadata
/// when its first `-`/`_`/`.`-delimited segment is a known variant word or an all-numeric version /
/// date component. `-mini` / `_mini` / `-mini-high` / `-2024-05` → variant; `_router` → symbol.
fn ambiguous_short_model_rest_is_variant(rest: &str) -> bool {
    let after_sep = rest.trim_start_matches(['-', '.', '_']);
    let first = after_sep.split(['-', '_', '.']).next().unwrap_or("");
    !first.is_empty()
        && (first.chars().all(|c| c.is_ascii_digit()) || MODEL_VARIANT_SUFFIXES.contains(&first))
}

/// Grammar-first anchor quality shared by extraction and the scorer.
pub(crate) fn anchor_quality(raw_token: &str) -> TargetAnchorQuality {
    let token = clean_anchor_token(raw_token);
    if token.is_empty() {
        return TargetAnchorQuality::Junk;
    }
    if stable_target_anchor_kind(&token).is_some() {
        TargetAnchorQuality::Stable
    } else if is_variable_noise_token(&token) {
        TargetAnchorQuality::Junk
    } else {
        TargetAnchorQuality::Weak
    }
}

/// Trim wrapping punctuation/quotes and a single trailing sentence period while preserving a leading
/// `.` (dotfiles / `./` relative) and internal `:` (`host:port`, `file.rs:line`).
fn clean_anchor_token(token: &str) -> String {
    let trimmed = token.trim_matches(|c: char| {
        matches!(
            c,
            '`' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '!' | '?'
        )
    });
    trimmed.strip_suffix('.').unwrap_or(trimmed).to_string()
}

/// Strip a trailing `:line` / `:line:col` reference so `exec.rs:1537` validates as `exec.rs`.
fn strip_line_ref(token: &str) -> &str {
    match token.split_once(':') {
        Some((head, tail))
            if !head.is_empty()
                && !tail.is_empty()
                && tail.chars().all(|c| c.is_ascii_digit() || c == ':') =>
        {
            head
        }
        _ => token,
    }
}

/// True iff `leaf` is `<stem><recognized-ext>` with a non-empty alphanumeric stem.
fn leaf_has_recognized_extension(leaf: &str) -> bool {
    let base = strip_line_ref(leaf).to_ascii_lowercase();
    RECOGNIZED_ANCHOR_EXTENSIONS.iter().any(|ext| {
        base.len() > ext.len()
            && base.ends_with(ext)
            && base[..base.len() - ext.len()]
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_alphanumeric())
    })
}

/// Typed-slot grammar validation (schema-guided DST pattern). Returns a stable anchor kind iff the
/// token validates one grammar; `None` means "not a durable typed anchor".
fn stable_target_anchor_kind(token: &str) -> Option<ObjectiveTargetKind> {
    if validates_repo_relative_path(token) || validates_windows_path(token) {
        return Some(if looks_like_doc_path(token) {
            ObjectiveTargetKind::SpecOrDesignDoc
        } else {
            ObjectiveTargetKind::FileOrDirectory
        });
    }
    if validates_recognized_extension_file(token) {
        return Some(if looks_like_doc_path(token) {
            ObjectiveTargetKind::SpecOrDesignDoc
        } else {
            ObjectiveTargetKind::FileOrDirectory
        });
    }
    if validates_rust_symbol_ref(token) {
        return Some(ObjectiveTargetKind::FileOrDirectory);
    }
    if validates_well_known_rootless_file(token) {
        return Some(ObjectiveTargetKind::FileOrDirectory);
    }
    if token.starts_with('@') && token.len() > 1 {
        return Some(ObjectiveTargetKind::RepoSlice);
    }
    if validates_bare_package_name(token) {
        return Some(ObjectiveTargetKind::CrateOrPackage);
    }
    if is_stable_work_item_identifier(token) {
        return Some(ObjectiveTargetKind::RepoSlice);
    }
    None
}

/// A path/file token must not carry structural punctuation from surrounding prose or markup
/// (`[label](path)` leftovers, quotes, glob/redirect chars); those are not real path characters.
fn has_path_punctuation_noise(token: &str) -> bool {
    token.chars().any(|c| {
        matches!(
            c,
            '[' | ']'
                | '('
                | ')'
                | '`'
                | '"'
                | '\''
                | '<'
                | '>'
                | '|'
                | '*'
                | '?'
                | '{'
                | '}'
                | ' '
                | '\t'
        )
    })
}

fn validates_repo_relative_path(token: &str) -> bool {
    if token.contains("://") || has_path_punctuation_noise(token) {
        return false;
    }
    if token.starts_with("./") || token.starts_with("../") {
        return token.len() > 2;
    }
    let segments = token
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return false;
    }
    if leaf_has_recognized_extension(segments[segments.len() - 1]) {
        return true;
    }
    let first = segments[0].to_ascii_lowercase();
    CONVENTIONAL_REPO_ROOTS.contains(&first.as_str()) && segments.len() >= 2
}

fn validates_windows_path(token: &str) -> bool {
    if has_path_punctuation_noise(token) {
        return false;
    }
    let bytes = token.as_bytes();
    let drive_absolute = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/');
    if drive_absolute {
        return true;
    }
    // A backslash-separated relative path only counts when its leaf carries a recognized
    // extension; otherwise escaped-control residue (`isolated.\n-`) would masquerade as a path.
    if token.contains('\\') {
        let segments = token
            .split('\\')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        return segments.len() >= 2
            && segments
                .last()
                .is_some_and(|leaf| leaf_has_recognized_extension(leaf));
    }
    false
}

fn validates_recognized_extension_file(token: &str) -> bool {
    !has_path_punctuation_noise(token)
        && !token.contains('/')
        && !token.contains('\\')
        && leaf_has_recognized_extension(token)
}

fn validates_rust_symbol_ref(token: &str) -> bool {
    if !token.contains("::") {
        return false;
    }
    token.split("::").all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            && segment
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    })
}

fn validates_well_known_rootless_file(token: &str) -> bool {
    let stem = strip_line_ref(token);
    let stem = stem.split('.').next().unwrap_or(stem).to_ascii_lowercase();
    WELL_KNOWN_ROOTLESS_FILES.contains(&stem.as_str())
}

/// Bare workspace package/crate name: all-lowercase kebab, ≥2 alphabetic segments, no digits/dots.
/// Admits `agent-drift-analyzer`; rejects `GPT-5.4`, `closeout/review-ready`.
fn validates_bare_package_name(token: &str) -> bool {
    if token.contains('/') || token.contains('.') || token.contains(':') {
        return false;
    }
    let segments = token.split('-').collect::<Vec<_>>();
    segments.len() >= 2
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() && c.is_ascii_alphabetic())
        })
}

/// Work-item identifier grammar (existing shape) minus model/version tokens.
fn is_stable_work_item_identifier(token: &str) -> bool {
    if !looks_like_work_item_identifier(token) {
        return false;
    }
    let prefix = token
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_lowercase();
    !MODEL_NAME_PREFIXES.contains(&prefix.as_str())
}

/// Log-template variable masks (Drain/LogPai preprocessing). Whole-token classification: only a
/// token whose entire value is a dynamic parameter is noise.
fn is_variable_noise_token(token: &str) -> bool {
    if token.contains("://") {
        return true; // URL scheme
    }
    let authority = token.split('/').next().unwrap_or(token);
    if is_host_port(authority) {
        return true; // ip:port / host:port, optionally followed by a route
    }
    if token.starts_with('/')
        && token.split('/').filter(|s| !s.is_empty()).count() == 1
        && !leaf_has_recognized_extension(token.trim_start_matches('/'))
    {
        return true; // bare `/graphql`-style endpoint
    }
    if is_numeric_run(token) {
        return true; // coordinate / port / number run
    }
    if is_hex_blob(token) {
        return true;
    }
    if is_duration_or_timestamp(token) {
        return true;
    }
    if is_control_residue(token) {
        return true;
    }
    false
}

fn is_host_port(token: &str) -> bool {
    let Some((host, port)) = token.rsplit_once(':') else {
        return false;
    };
    if port.is_empty() || !port.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    host == "localhost"
        || (!host.is_empty()
            && host
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c.is_ascii_alphabetic())
            && host.chars().any(|c| c == '.' || c.is_ascii_digit()))
}

fn is_numeric_run(token: &str) -> bool {
    let stripped = strip_line_ref(token);
    let mut saw_digit = false;
    for c in stripped.chars() {
        if c.is_ascii_digit() {
            saw_digit = true;
        } else if !matches!(c, '.' | '_' | '-' | ':' | 'x' | 'X') {
            return false;
        }
    }
    saw_digit
}

fn is_hex_blob(token: &str) -> bool {
    let body = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"));
    match body {
        Some(rest) => rest.len() >= 4 && rest.chars().all(|c| c.is_ascii_hexdigit()),
        None => {
            token.len() >= 8
                && token.chars().all(|c| c.is_ascii_hexdigit())
                && token.chars().any(|c| c.is_ascii_alphabetic())
                && token.chars().any(|c| c.is_ascii_digit())
        }
    }
}

fn is_duration_or_timestamp(token: &str) -> bool {
    // duration: <digits><unit>, e.g. 5s, 120ms, 2h
    let lowered = token.to_ascii_lowercase();
    for unit in ["ms", "us", "ns", "s", "m", "h", "sec", "min"] {
        if let Some(head) = lowered.strip_suffix(unit) {
            if !head.is_empty() && head.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    // ISO-ish timestamp or clock: contains 'T' between digit runs, or HH:MM:SS
    let clockish = token.split(':').count() == 3
        && token
            .split(':')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
    clockish
}

fn is_control_residue(token: &str) -> bool {
    if token.contains("\\n") || token.contains("\\t") || token.contains("\\r") {
        return true;
    }
    // stray control letters left by escaped-newline splitting: only n/t/r and digits/separators
    let mut saw_control = false;
    for c in token.chars() {
        match c {
            'n' | 't' | 'r' => saw_control = true,
            c if c.is_ascii_digit() || matches!(c, '_' | '-' | '.') => {}
            _ => return false,
        }
    }
    saw_control
}

/// Scorer-side guard operating on the normalized (`[a-z0-9_]+`) term form. Shares the taxonomy:
/// rejects numeric/coordinate runs, control residue, and model-version tokens, while accepting
/// alphanumeric work-item / packet IDs (`r6_3_5`, `so_2_3b`) that extraction treats as stable
/// anchors — the two sides must agree or packet-only goals silently lose eligibility.
pub(crate) fn is_stable_goal_term(term: &str) -> bool {
    if term.len() < 2 {
        return false;
    }
    let segments = term
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return false;
    }
    // A term is content-bearing if some segment is either a real alphabetic word (`objective`) or a
    // letter-led alphanumeric identifier (`r6`, `so2`). This accepts work-item / packet IDs
    // (`r6_3_5`, `so_2_3b`) while rejecting pure number/coordinate runs (`0_0_0_0_4000`,
    // `1920x1080`), control residue (`5_n_n`), and digit-led duration units (`5s`, `120ms`) — those
    // never start a segment with a letter.
    let has_content = segments.iter().any(|s| {
        let alpha_word =
            s.len() >= 2 && s.chars().all(|c| c.is_ascii_alphabetic()) && !is_control_word(s);
        let id_segment = s.len() >= 2
            && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && s.chars().any(|c| c.is_ascii_digit())
            && s.chars().all(|c| c.is_ascii_alphanumeric());
        alpha_word || id_segment
    });
    if !has_content {
        return false;
    }
    // Reject normalized model/version junk by round-tripping the segments through the shared
    // model/version classifier. `normalize_comparison_key_segment` flattens the `.`/`-` version and
    // variant separators to `_`, so rejoin on `.` to reunite them: `o4-mini`->`o4_mini`->`o4.mini`
    // (model branch) and `v2.3.1`->`v2_3_1`->`v2.3.1` (version branch) both fire, while work-item IDs
    // (`r6.3.5`, `so.2.3b`) match neither grammar (codex re-review finding 1).
    if is_model_or_version_token(&segments.join(".")) {
        return false;
    }
    true
}

fn is_control_word(segment: &str) -> bool {
    !segment.is_empty() && segment.chars().all(|c| matches!(c, 'n' | 't' | 'r'))
}

/// Derive `primary_intent` from the selected goal clause's request **action**, not incidental
/// substrings (architecture "Minimum Semantic Coverage #2" + Stage 4 intent inference). We match
/// whole action words so the noun "implementation" never trips the verb "implement" and "Planning
/// Pack" never trips "plan". The clause's leading imperative verb wins first; otherwise we fall back
/// to a whole-word family scan.
fn intent_for_text(text: &str) -> ObjectiveIntent {
    let goal = target_display_for_goal(text);
    let scan = if goal.is_empty() { text } else { goal.as_str() };
    let tokens = action_word_tokens(scan);

    if let Some(intent) = tokens.first().and_then(|verb| intent_for_action_verb(verb)) {
        return intent;
    }
    intent_from_action_words(&tokens)
}

fn action_word_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|c: char| !c.is_ascii_alphanumeric())
                .to_ascii_lowercase()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn intent_for_action_verb(verb: &str) -> Option<ObjectiveIntent> {
    Some(match verb {
        "implement" | "add" | "update" | "wire" | "land" | "build" | "create" | "refactor"
        | "migrate" | "integrate" => ObjectiveIntent::Implement,
        "debug" | "fix" | "troubleshoot" | "repair" | "diagnose" => ObjectiveIntent::Debug,
        "review" | "inspect" | "audit" | "compare" | "analyze" | "evaluate" | "assess"
        | "determine" => ObjectiveIntent::Review,
        "research" | "investigate" | "explore" | "survey" | "study" => ObjectiveIntent::Research,
        "plan" | "design" | "spec" | "scope" | "draft" => ObjectiveIntent::Plan,
        "validate" | "verify" | "ensure" | "confirm" => ObjectiveIntent::Validate,
        "document" | "docs" | "readme" => ObjectiveIntent::Docs,
        _ => return None,
    })
}

fn intent_from_action_words(tokens: &[String]) -> ObjectiveIntent {
    let has = |needles: &[&str]| tokens.iter().any(|token| needles.contains(&token.as_str()));
    if has(&["implement", "add", "update", "wire", "land", "build"]) {
        ObjectiveIntent::Implement
    } else if has(&["debug", "fix", "troubleshoot"]) {
        ObjectiveIntent::Debug
    } else if has(&[
        "review",
        "inspect",
        "determine",
        "compare",
        "analyze",
        "evaluate",
        "assess",
        "audit",
    ]) {
        ObjectiveIntent::Review
    } else if has(&["research", "survey"]) {
        ObjectiveIntent::Research
    } else if has(&["plan", "design", "spec"]) {
        ObjectiveIntent::Plan
    } else if has(&["validate", "verify", "ensure", "green", "test"]) {
        ObjectiveIntent::Validate
    } else if has(&["docs", "document", "readme"]) {
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
    let mut paths: Vec<String> = Vec::new();
    let push_if_path = |candidate: &str, paths: &mut Vec<String>| {
        let cleaned = clean_anchor_token(candidate);
        if is_stable_path_anchor(&cleaned) && !paths.contains(&cleaned) {
            paths.push(cleaned);
        }
    };
    // Parse-before-split (codex review §6): markdown-link / backtick / quoted path targets survive
    // whitespace tokenization that would otherwise mangle `[label](path)` and `` `path` ``.
    for candidate in extract_delimited_path_candidates(text) {
        push_if_path(&candidate, &mut paths);
    }
    for token in text.split_whitespace() {
        push_if_path(token, &mut paths);
    }
    paths
}

/// Path-family anchors only (repo-relative / recognized-extension / Windows), excluding symbol,
/// crate, and work-item anchors that `stable_target_anchor_kind` also accepts.
fn is_stable_path_anchor(token: &str) -> bool {
    validates_repo_relative_path(token)
        || validates_windows_path(token)
        || validates_recognized_extension_file(token)
}

/// Extract path candidates wrapped in a markdown link `](target)`, backticks, or quotes, before
/// whitespace splitting can break them apart.
fn extract_delimited_path_candidates(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut search = text;
    while let Some(open) = search.find("](") {
        let after = &search[open + 2..];
        match after.find(')') {
            Some(close) => {
                let target = after[..close].trim();
                if !target.is_empty() {
                    out.push(target.to_string());
                }
                search = &after[close + 1..];
            }
            None => break,
        }
    }
    for delim in ['`', '"', '\''] {
        let mut rest = text;
        while let Some(start) = rest.find(delim) {
            let after = &rest[start + 1..];
            match after.find(delim) {
                Some(end) => {
                    let inner = after[..end].trim();
                    if !inner.is_empty() && !inner.contains(char::is_whitespace) {
                        out.push(inner.to_string());
                    }
                    rest = &after[end + 1..];
                }
                None => break,
            }
        }
    }
    out
}

fn extract_named_artifacts(text: &str) -> Vec<String> {
    let mut artifacts = Vec::new();
    let lowered = text.to_ascii_lowercase();
    for (needle, display) in [
        ("agents.md", "AGENTS.md"),
        ("claude.md", "CLAUDE.md"),
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
    // Bare uppercase instruction-file shorthands (`update AGENTS`, `review CLAUDE`) point at the
    // AGENTS.md / CLAUDE.md instruction surfaces. Case-sensitive whole-word match over the original
    // text so ordinary prose words ("the agents", "Claude") are not promoted — dropping these stems
    // from WELL_KNOWN_ROOTLESS_FILES otherwise left bare shorthands with no target (codex re-review 3).
    for raw in text.split_whitespace() {
        let display = match raw.trim_matches(|c: char| !c.is_ascii_alphanumeric()) {
            "AGENTS" => "AGENTS.md",
            "CLAUDE" => "CLAUDE.md",
            _ => continue,
        };
        if !artifacts.iter().any(|existing| existing == display) {
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
        .trim_end_matches([':', ';', ','])
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
    fn projects_multiline_goal_sections_from_structured_goal_evidence() {
        let rows = vec![test_row(
            "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only.\nUse the $incremental-implementation skill.\nReturn with changed files and residual risk.\n\n## Verification\n- cargo test -p agent-drift-analyzer checkpoints -- --nocapture",
        )];

        let summary = extract_objective(&rows);
        assert_eq!(
            summary.text,
            "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
        );
        assert!(!summary.text.contains("$incremental-implementation"));

        let Some(structured) = summary.structured else {
            panic!("structured sidecar")
        };
        let goal_span = structured
            .evidence_spans
            .iter()
            .find(|span| {
                span.role == ObjectiveRole::Goal
                    && span.excerpt
                        == "/goal Review crates/agent-drift-analyzer/src/context/objective.rs only."
            })
            .expect("goal evidence span");
        assert_eq!(goal_span.excerpt, summary.text);
        assert_eq!(
            structured
                .target
                .as_ref()
                .map(|target| target.display.as_str()),
            Some("crates/agent-drift-analyzer/src/context/objective.rs")
        );
        assert_eq!(
            summary.verification_commands,
            vec!["cargo test -p agent-drift-analyzer checkpoints -- --nocapture"]
        );
    }

    #[test]
    fn collects_verification_commands_from_goal_led_mixed_role_clauses() {
        let rows = vec![test_row(
            "## Scope\nReview the objective extractor, run make test, and return concrete fixes.",
        )];
        let decomposition = decompose_objective_rows(&rows).expect("objective decomposition");
        let mixed_clause = decomposition
            .clauses
            .iter()
            .find(|clause| clause.text.contains("run make test"))
            .expect("mixed role clause");

        assert_eq!(
            top_role(mixed_clause).map(|role| role.role),
            Some(ObjectiveRole::Goal)
        );
        assert!(clause_has_role_candidate(
            mixed_clause,
            ObjectiveRole::Verification
        ));
        assert_eq!(
            verification_commands_from_decomposition(&decomposition),
            vec!["make test"]
        );

        let summary = extract_objective(&rows);
        assert_eq!(
            summary.text,
            "Review the objective extractor, run make test."
        );
        let Some(structured) = summary.structured else {
            panic!("structured sidecar")
        };
        assert_eq!(structured.verification_commands, vec!["make test"]);
        assert!(structured.evidence_spans.iter().any(|span| {
            span.role == ObjectiveRole::Goal
                && span
                    .excerpt
                    .contains("Review the objective extractor, run make test.")
                && span.section_index == Some(0)
                && span.clause_index == Some(0)
        }));
        assert!(structured.evidence_spans.iter().any(|span| {
            span.role == ObjectiveRole::Verification
                && span
                    .excerpt
                    .contains("Review the objective extractor, run make test.")
                && span.section_index == Some(0)
                && span.clause_index == Some(0)
        }));
        assert!(structured
            .deliverables
            .iter()
            .any(|deliverable| deliverable.display.contains("Return concrete fixes")));
    }

    #[test]
    fn verification_commands_prefer_clause_grounding_over_whole_candidate_fallback() {
        let row = test_row(
            "Review the objective extractor, run make test, and return concrete fixes. Record the literal string `cargo fmt --check` in the handoff.",
        );
        let decomposition = ObjectiveDecomposition {
            candidates: vec![DirectiveRowCandidate {
                candidate_index: 0,
                row_ref: RowRef::from_row(&row),
                source_kind: source_kind_for_row(&row),
                text: row.text.clone(),
                score: objective_score(&row),
            }],
            sections: Vec::new(),
            clauses: vec![
                ObjectiveClause {
                    candidate_index: 0,
                    row_ref: RowRef::from_row(&row),
                    source_kind: source_kind_for_row(&row),
                    section_index: 0,
                    clause_index: 0,
                    section_kind: ObjectiveSectionKind::Scope,
                    text: "Review the objective extractor, run make test.".to_string(),
                    role_candidates: vec![
                        RoleCandidate {
                            role: ObjectiveRole::Goal,
                            confidence: Confidence::High,
                            score: 900,
                        },
                        RoleCandidate {
                            role: ObjectiveRole::Verification,
                            confidence: Confidence::High,
                            score: 825,
                        },
                    ],
                },
                ObjectiveClause {
                    candidate_index: 0,
                    row_ref: RowRef::from_row(&row),
                    source_kind: source_kind_for_row(&row),
                    section_index: 0,
                    clause_index: 1,
                    section_kind: ObjectiveSectionKind::Scope,
                    text: "Return concrete fixes.".to_string(),
                    role_candidates: vec![RoleCandidate {
                        role: ObjectiveRole::OtherRole,
                        confidence: Confidence::Medium,
                        score: 700,
                    }],
                },
                ObjectiveClause {
                    candidate_index: 0,
                    row_ref: RowRef::from_row(&row),
                    source_kind: source_kind_for_row(&row),
                    section_index: 0,
                    clause_index: 2,
                    section_kind: ObjectiveSectionKind::Scope,
                    text: "Record the literal string `cargo fmt --check` in the handoff."
                        .to_string(),
                    role_candidates: vec![RoleCandidate {
                        role: ObjectiveRole::OtherRole,
                        confidence: Confidence::Medium,
                        score: 400,
                    }],
                },
            ],
        };

        assert_eq!(
            verification_commands_from_decomposition(&decomposition),
            vec!["make test"]
        );
    }

    #[test]
    fn falls_back_to_primary_candidate_text_when_structured_primary_goal_is_unknown() {
        let prompt =
            "## Questions to ask\n- Which packet follows SO-G2?\n- Which docs own the acceptance wall?";
        let rows = vec![test_row(prompt)];

        let summary = extract_objective(&rows);
        assert_eq!(summary.text, prompt);

        let Some(structured) = summary.structured else {
            panic!("structured sidecar")
        };
        assert!(structured
            .unknowns
            .iter()
            .any(|unknown| unknown.field_name == "primary_goal"));
    }

    #[test]
    fn comparison_key_uses_structured_state_instead_of_checklist_wording() {
        let scope = "Validate Packet SO-3.2 only until the verification wall is green.";
        let prompt_a = format!(
            "## Scope\n{scope}\n\n## Checklist\n- Run this task on a linux machine.\n- Inspect objective.rs before editing."
        );
        let prompt_b = format!(
            "## Scope\n{scope}\n\n## Checklist\n- Keep validation on linux only.\n- Read the packet notes before touching code."
        );

        let summary_a = extract_objective(&[test_row(&prompt_a)]);
        let summary_b = extract_objective(&[test_row(&prompt_b)]);

        assert_eq!(summary_a.text, scope);
        assert_eq!(summary_b.text, scope);
        assert_eq!(summary_a.comparison_key, "validate|repo_slice|so_3_2|green");
        assert_eq!(summary_a.comparison_key, summary_b.comparison_key);
    }

    #[test]
    fn compatibility_overlay_preserves_structured_comparison_key() {
        let summary = extract_objective(&[test_row(
            "/goal Determine whether the AGENTS.md instruction block and <skill> section should change.",
        )]);
        let compatibility = ObjectiveSummary::compatibility(
            "Filesystem sandboxing defines which files can be read or written.".to_string(),
            Vec::new(),
            Vec::new(),
        );

        let merged = summary.with_compatibility_display_from(&compatibility);

        assert_eq!(
            merged.text,
            "Filesystem sandboxing defines which files can be read or written."
        );
        assert_eq!(merged.comparison_key, summary.comparison_key);
        assert_eq!(
            merged.comparison_key,
            "review|skill_or_instruction_surface|agents_md|instruction_block|skill"
        );
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

    // --- R6-3.5 shared target-anchor taxonomy ---

    #[test]
    fn anchor_quality_marks_typed_grammars_stable() {
        for stable in [
            "crates/agent-drift-analyzer/src/context/objective.rs",
            "docs/architecture_overview.md",
            ".github/workflows/ci.yml",
            "Cargo.lock",
            "README.md",
            "README",
            "Makefile",
            "v2.3.1/notes.md",
            "2024-report.md",
            "docs/graphql/overview.md",
            "0xdeadbeef.rs",
            "./relative/path.rs",
            "exec.rs:1537",
            "foo::bar",
            "score_semantic_goal_drift::inner",
            "agent-drift-analyzer",
            "@workspace-ref",
            "R6-3.5",
            "SO-2.3B",
        ] {
            assert_eq!(
                anchor_quality(stable),
                TargetAnchorQuality::Stable,
                "expected `{stable}` to be a stable anchor"
            );
        }
    }

    #[test]
    fn anchor_quality_marks_variable_noise_junk() {
        for junk in [
            "http://0.0.0.0:4000/graphql",
            "0.0.0.0:4000",
            "localhost:4000",
            "127.0.0.1:8080",
            "/graphql",
            "0_0_0_0_4000",
            "1920x1080",
            "5s",
            "120ms",
            "11:58:13",
            "isolated.\\n-",
            "5\\n\\n",
        ] {
            assert_eq!(
                anchor_quality(junk),
                TargetAnchorQuality::Junk,
                "expected `{junk}` to be junk"
            );
        }
    }

    #[test]
    fn anchor_quality_marks_untyped_prose_and_truncated_paths_weak() {
        for weak in [
            "/Users/spenser/Library/Application",
            "closeout",
            "narrowing",
        ] {
            assert_eq!(
                anchor_quality(weak),
                TargetAnchorQuality::Weak,
                "expected `{weak}` to be weak (non-scoring), not stable or junk"
            );
        }
    }

    #[test]
    fn anchor_quality_rejects_bare_model_tokens() {
        assert_ne!(anchor_quality("GPT-5.4"), TargetAnchorQuality::Stable);
        assert_ne!(anchor_quality("gpt-4o"), TargetAnchorQuality::Stable);
    }

    #[test]
    fn extract_inline_paths_rejects_graphql_startup_log_targets() {
        let paths = extract_inline_paths(
            "Server listening on http://0.0.0.0:4000/graphql (5s, supergraph)",
        );
        assert!(
            paths.is_empty(),
            "graphql startup log must yield no path targets, got {paths:?}"
        );
    }

    #[test]
    fn extract_inline_paths_preserves_repo_paths_and_markdown_links() {
        assert_eq!(
            extract_inline_paths(
                "Review crates/agent-drift-analyzer/src/context/objective.rs only."
            ),
            vec!["crates/agent-drift-analyzer/src/context/objective.rs".to_string()]
        );
        assert_eq!(
            extract_inline_paths(
                "See [architecture overview](docs/architecture_overview.md) first."
            ),
            vec!["docs/architecture_overview.md".to_string()]
        );
        let mixed = extract_inline_paths("touch .github/workflows/ci.yml and Cargo.lock");
        assert!(mixed.iter().any(|p| p == ".github/workflows/ci.yml"));
        assert!(mixed.iter().any(|p| p == "Cargo.lock"));
    }

    #[test]
    fn extract_inline_paths_drops_truncated_external_path_fragments() {
        let paths = extract_inline_paths("saw /Users/spenser/Library/Application in the log");
        assert!(
            paths.is_empty(),
            "truncated external path fragment is not a stable path, got {paths:?}"
        );
    }

    #[test]
    fn named_target_extraction_rejects_model_tokens_but_keeps_work_items() {
        assert!(!looks_like_explicit_named_target("GPT-5.4"));
        assert!(!looks_like_explicit_named_target("gpt-4o"));
        assert!(is_model_or_version_token("GPT-5.4"));
        assert!(is_model_or_version_token("v2.3.1"));
        assert!(!is_model_or_version_token("SO-2.3B"));
        assert_eq!(
            extract_work_item_identifier("Land SO-2.3B packet"),
            Some("SO-2.3B".to_string())
        );
        assert_eq!(extract_work_item_identifier("Use GPT-5.4 to review"), None);
    }

    #[test]
    fn comparison_key_drops_junk_target_specifics_keeps_stable() {
        let target = ObjectiveTarget {
            display: "0.0.0.0:4000".to_string(),
            kind: ObjectiveTargetKind::FileOrDirectory,
            paths: vec!["0.0.0.0:4000".to_string(), "crates/foo/bar.rs".to_string()],
            symbols: Vec::new(),
            named_artifacts: Vec::new(),
            workspace_refs: Vec::new(),
            evidence: Vec::new(),
            confidence: Confidence::High,
        };
        let segments = comparison_key_segments_for_target(&target);
        assert!(
            segments.iter().any(|s| s.contains("bar")),
            "stable path must be retained: {segments:?}"
        );
        assert!(
            !segments.iter().any(|s| s.contains("4000")),
            "junk host:port specific must be dropped: {segments:?}"
        );
    }

    #[test]
    fn is_stable_goal_term_rejects_normalized_junk() {
        for junk in [
            "0_0_0_0_4000",
            "5_n_n",
            "0_0_0_0_4000_n",
            "n",
            "gpt_5_4",
            "5s",
            "1920x1080",
            // codex re-review finding 1: normalized model/version junk must not survive the
            // comparison-key gate (`o4-mini`->`o4_mini`, `v2.3.1`->`v2_3_1`).
            "o4_mini",
            "v2_3_1",
        ] {
            assert!(
                !is_stable_goal_term(junk),
                "normalized term `{junk}` must not be a stable goal term"
            );
        }
        for stable in [
            "objective_rs",
            "architecture_overview_md",
            "readme_md",
            "agent_drift_analyzer",
            // work-item / packet IDs must agree with the extraction side (codex review finding 1)
            "r6_3_5",
            "so_2_3b",
            "r5_75_6_4",
        ] {
            assert!(
                is_stable_goal_term(stable),
                "normalized term `{stable}` must remain a stable goal term"
            );
        }
    }

    #[test]
    fn model_mask_catches_alphanumeric_o_families() {
        // codex review finding 2: `take_while(alpha)` reduced `o4-mini` to prefix `o` and missed it.
        for model in [
            "o4-mini",
            "o3",
            "o1-preview",
            "GPT-5.4",
            "claude-3",
            "gemma-2",
            "v2.3.1",
            // codex re-review 3: underscore-normalized model/variant and version spellings must be
            // caught on the extraction side too, not only in the scorer's `.`-joined form.
            "o4_mini",
            "v2_3_1",
        ] {
            assert!(
                is_model_or_version_token(model),
                "`{model}` must be recognized as model/version metadata"
            );
            assert_ne!(
                anchor_quality(model),
                TargetAnchorQuality::Stable,
                "`{model}` must not be a stable anchor"
            );
        }
        // repo/domain tokens that must NOT be swept up as model metadata. `o4_router` is the codex
        // re-review finding 2 case: a short-family prefix over an underscore is a symbol, not a model.
        for keep in [
            "SO-2.3B",
            "R6-3.5",
            "codex-wrapper",
            "agent-drift-analyzer",
            "o4_router",
        ] {
            assert!(
                !is_model_or_version_token(keep),
                "`{keep}` must not be treated as model metadata"
            );
        }
    }

    #[test]
    fn generic_repo_vocabulary_is_not_a_rootless_file_anchor() {
        // codex re-review finding 3: `cargo`/`agents`/`claude` are ordinary prose words; the real
        // files carry extensions and are anchored by the extension/instruction paths instead.
        for word in ["cargo", "agents", "claude"] {
            assert!(
                !validates_well_known_rootless_file(word),
                "`{word}` must not validate as a rootless-file anchor"
            );
            assert_eq!(
                stable_target_anchor_kind(word),
                None,
                "`{word}` must not be a stable target anchor"
            );
        }
        // A goal that merely mentions the build tool must not anchor `cargo` as a file target.
        if let Some(anchor) = explicit_target_anchor_for_text("run cargo test for the suite") {
            assert!(
                anchor.named_artifacts.iter().all(|a| a != "cargo") && anchor.display != "cargo",
                "bare `cargo` must not become a file target, got {anchor:?}"
            );
        }
        // The extension-bearing real files must still anchor.
        assert_eq!(
            stable_target_anchor_kind("Cargo.toml"),
            Some(ObjectiveTargetKind::FileOrDirectory),
            "`Cargo.toml` must remain a file anchor via the extension path"
        );
    }

    #[test]
    fn extraction_recovers_bare_rootless_files_and_rust_symbols() {
        // codex review finding 3: without a path or cue, `update README` / `refactor foo::bar` must
        // still yield a structured target (otherwise the opaque-parent guardrail suppresses drift).
        let readme = explicit_target_anchor_for_text("update README and tidy it")
            .expect("README should be a stable target");
        assert!(readme.named_artifacts.iter().any(|a| a == "README"));

        let symbol = explicit_target_anchor_for_text("refactor foo::bar for clarity")
            .expect("a rust symbol ref should be a stable target");
        assert!(symbol.named_artifacts.iter().any(|a| a == "foo::bar"));
    }

    #[test]
    fn named_target_extraction_stays_consistent_for_underscore_model_tokens() {
        // codex re-review 3: the extraction guard must agree with the scorer. Underscore-form model
        // and version tokens are metadata (not targets); an underscore into an arbitrary symbol word
        // is a real target — the `o4_router` case round 2 required us to preserve.
        assert!(
            !looks_like_explicit_named_target("o4_mini"),
            "`o4_mini` is a model variant, not a named target"
        );
        assert!(
            !looks_like_explicit_named_target("v2_3_1"),
            "`v2_3_1` is a version token, not a named target"
        );
        assert!(
            looks_like_explicit_named_target("o4_router"),
            "`o4_router` is a symbol and must remain a named target"
        );
    }

    #[test]
    fn bare_instruction_file_shorthands_anchor_as_instruction_surface() {
        // codex re-review 3: dropping agents/claude from the rootless set left `update AGENTS` /
        // `review CLAUDE` with no target; route the uppercase shorthands to the instruction surface.
        let artifacts = extract_named_artifacts("update AGENTS and review CLAUDE");
        assert!(artifacts.iter().any(|a| a == "AGENTS.md"));
        assert!(artifacts.iter().any(|a| a == "CLAUDE.md"));

        // Ordinary prose words must not be promoted (case-sensitive whole-word discriminator).
        let prose = extract_named_artifacts("the agents fixed it and Claude approved");
        assert!(
            prose.is_empty(),
            "lowercase prose must not anchor, got {prose:?}"
        );

        let anchor = explicit_target_anchor_for_text("update AGENTS with the new policy")
            .expect("bare AGENTS should anchor an instruction surface");
        assert_eq!(anchor.kind, ObjectiveTargetKind::SkillOrInstructionSurface);
        assert!(anchor.named_artifacts.iter().any(|a| a == "AGENTS.md"));
    }
}
