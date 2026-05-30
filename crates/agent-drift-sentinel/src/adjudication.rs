use crate::operator_surface::CheckpointPresentation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjudicationConfig {
    pub enabled: bool,
    pub model: String,
    pub reasoning_effort: ReasoningEffort,
    pub max_evidence_items: usize,
    pub max_context_chars: usize,
}

impl Default for AdjudicationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            model: "gpt-5.4-mini".to_string(),
            reasoning_effort: ReasoningEffort::Medium,
            max_evidence_items: 3,
            max_context_chars: 160,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
}

impl ReasoningEffort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjudicationRequest {
    pub model: String,
    pub reasoning_effort: String,
    pub checkpoint_id: String,
    pub session_id: String,
    pub operator_summary: String,
    pub expected_next_step: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjudicationResponse {
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjudicationFailure {
    pub message: String,
}

pub fn shape_request(
    presentation: &CheckpointPresentation,
    config: &AdjudicationConfig,
) -> Option<AdjudicationRequest> {
    if !config.enabled {
        return None;
    }

    Some(AdjudicationRequest {
        model: config.model.clone(),
        reasoning_effort: config.reasoning_effort.as_str().to_string(),
        checkpoint_id: presentation.checkpoint.checkpoint_id.clone(),
        session_id: presentation.checkpoint.session_id.clone(),
        operator_summary: truncate(
            &presentation.render_console_block(None),
            config.max_context_chars,
        ),
        expected_next_step: truncate(&presentation.expected_next_step, config.max_context_chars),
        evidence: presentation
            .evidence_lines
            .iter()
            .take(config.max_evidence_items)
            .cloned()
            .collect(),
    })
}

pub fn success_note(response: &AdjudicationResponse) -> String {
    format!("model supplement: {}", response.summary)
}

pub fn fallback_note(failure: &AdjudicationFailure) -> String {
    format!(
        "model adjudication unavailable; using analyzer evidence only ({})",
        failure.message
    )
}

fn truncate(text: &str, max_chars: usize) -> String {
    let truncated = text.chars().take(max_chars).collect::<String>();
    if text.chars().count() > max_chars {
        format!("{truncated}...")
    } else {
        truncated
    }
}
