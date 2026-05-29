use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationObligationSeverity {
    #[default]
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationObligationKind {
    FollowUpRequired,
    ApprovalRequired,
    Blocked,
    TaskCompleted,
    TaskFailed,
    RuntimeAlert,
    ForkRequest,
    ForkRecommendation,
    EscalationRecommended,
    ResultAvailable,
}

impl OrchestrationObligationKind {
    #[allow(dead_code)]
    pub(crate) fn supports_router_auto_attach(self) -> bool {
        matches!(
            self,
            Self::FollowUpRequired | Self::ApprovalRequired | Self::Blocked | Self::ForkRequest
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationObligationState {
    #[default]
    Pending,
    Resolved,
}

impl OrchestrationObligationState {
    fn is_pending(self) -> bool {
        matches!(self, Self::Pending)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationObligationReviewState {
    #[default]
    Unread,
    Acknowledged,
    Dismissed,
    Resolved,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationObligationAttachState {
    #[default]
    NotEligible,
    Eligible,
    Claimed,
    Satisfied,
    FailedClosed,
    Superseded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct OrchestrationObligationRecord {
    pub orchestration_session_id: String,
    pub obligation_id: String,
    pub kind: OrchestrationObligationKind,
    #[serde(default)]
    pub severity: OrchestrationObligationSeverity,
    #[serde(default)]
    pub attention_required: bool,
    pub state: OrchestrationObligationState,
    #[serde(default)]
    pub review_state: OrchestrationObligationReviewState,
    #[serde(default)]
    pub attach_state: OrchestrationObligationAttachState,
    #[serde(default)]
    pub attach_attempt_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_claim_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_last_attempt_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_completion_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_backend_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
}

impl OrchestrationObligationRecord {
    #[allow(dead_code)]
    pub(crate) fn new(
        orchestration_session_id: impl Into<String>,
        obligation_id: impl Into<String>,
        kind: OrchestrationObligationKind,
        summary: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            orchestration_session_id: orchestration_session_id.into(),
            obligation_id: obligation_id.into(),
            kind,
            severity: OrchestrationObligationSeverity::Info,
            attention_required: false,
            state: OrchestrationObligationState::Pending,
            review_state: OrchestrationObligationReviewState::Unread,
            attach_state: OrchestrationObligationAttachState::NotEligible,
            attach_attempt_count: 0,
            attach_claim_owner: None,
            attach_last_attempt_at: None,
            attach_completion_reason: None,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            summary: summary.into(),
            resolution_note: None,
            source_participant_id: None,
            target_backend_id: None,
            world_id: None,
            world_generation: None,
            payload: None,
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.orchestration_session_id.trim().is_empty() {
            anyhow::bail!("orchestration obligation must include orchestration_session_id");
        }
        if self.obligation_id.trim().is_empty() {
            anyhow::bail!("orchestration obligation must include obligation_id");
        }
        if self.summary.trim().is_empty() {
            anyhow::bail!("orchestration obligation must include summary");
        }
        if self.state.is_pending() && self.resolved_at.is_some() {
            anyhow::bail!("pending orchestration obligations must not include resolved_at");
        }
        if !self.state.is_pending() && self.resolved_at.is_none() {
            anyhow::bail!("resolved orchestration obligations must include resolved_at");
        }
        if self.world_id.is_some() != self.world_generation.is_some() {
            anyhow::bail!(
                "orchestration obligation world binding must provide both world_id and world_generation"
            );
        }
        if self.state == OrchestrationObligationState::Pending
            && self.review_state == OrchestrationObligationReviewState::Resolved
        {
            anyhow::bail!(
                "pending orchestration obligations cannot advertise resolved review_state"
            );
        }
        if self.attach_attempt_count > 0 && self.attach_last_attempt_at.is_none() {
            anyhow::bail!(
                "orchestration obligations with attach attempts must include attach_last_attempt_at"
            );
        }
        if self
            .attach_claim_owner
            .as_deref()
            .is_some_and(|claim_owner| claim_owner.trim().is_empty())
        {
            anyhow::bail!("orchestration obligations must not persist an empty attach_claim_owner");
        }
        if self
            .attach_completion_reason
            .as_deref()
            .is_some_and(|reason| reason.trim().is_empty())
        {
            anyhow::bail!(
                "orchestration obligations must not persist an empty attach_completion_reason"
            );
        }
        match self.attach_state {
            OrchestrationObligationAttachState::Claimed => {
                if self.attach_attempt_count == 0 {
                    anyhow::bail!(
                        "claimed orchestration obligations must record attach_attempt_count"
                    );
                }
                if self.attach_claim_owner.is_none() {
                    anyhow::bail!(
                        "claimed orchestration obligations must include attach_claim_owner"
                    );
                }
                if self.attach_completion_reason.is_some() {
                    anyhow::bail!(
                        "claimed orchestration obligations must not include attach_completion_reason"
                    );
                }
            }
            OrchestrationObligationAttachState::Satisfied
            | OrchestrationObligationAttachState::FailedClosed
            | OrchestrationObligationAttachState::Superseded => {
                if self.attach_completion_reason.is_none() {
                    anyhow::bail!(
                        "terminal orchestration attach states must include attach_completion_reason"
                    );
                }
            }
            OrchestrationObligationAttachState::NotEligible
            | OrchestrationObligationAttachState::Eligible => {}
        }

        Ok(())
    }

    pub(crate) fn is_pending(&self) -> bool {
        self.state.is_pending()
    }

    pub(crate) fn projects_detached_attention(&self) -> bool {
        self.is_pending() && self.attention_required
    }

    #[allow(dead_code)]
    pub(crate) fn is_auto_attach_eligible(&self) -> bool {
        self.is_pending()
            && self.attach_state == OrchestrationObligationAttachState::Eligible
            && self.kind.supports_router_auto_attach()
    }

    #[allow(dead_code)]
    pub(crate) fn is_auto_attach_claimed(&self) -> bool {
        self.is_pending() && self.attach_state == OrchestrationObligationAttachState::Claimed
    }

    #[allow(dead_code)]
    pub(crate) fn mark_attach_claimed(
        &mut self,
        attach_claim_owner: impl Into<String>,
        claimed_at: DateTime<Utc>,
    ) {
        self.attach_state = OrchestrationObligationAttachState::Claimed;
        self.attach_attempt_count = self
            .attach_attempt_count
            .checked_add(1)
            .expect("attach_attempt_count overflow");
        self.attach_claim_owner = Some(attach_claim_owner.into());
        self.attach_last_attempt_at = Some(claimed_at);
        self.attach_completion_reason = None;
        self.updated_at = claimed_at;
    }

    pub(crate) fn mark_attach_satisfied(
        &mut self,
        attach_completion_reason: impl Into<String>,
        settled_at: DateTime<Utc>,
    ) {
        self.mark_attach_terminal_state(
            OrchestrationObligationAttachState::Satisfied,
            attach_completion_reason,
            settled_at,
        );
    }

    #[allow(dead_code)]
    pub(crate) fn mark_attach_failed_closed(
        &mut self,
        attach_completion_reason: impl Into<String>,
        settled_at: DateTime<Utc>,
    ) {
        self.mark_attach_terminal_state(
            OrchestrationObligationAttachState::FailedClosed,
            attach_completion_reason,
            settled_at,
        );
    }

    pub(crate) fn mark_attach_superseded(
        &mut self,
        attach_completion_reason: impl Into<String>,
        settled_at: DateTime<Utc>,
    ) {
        self.mark_attach_terminal_state(
            OrchestrationObligationAttachState::Superseded,
            attach_completion_reason,
            settled_at,
        );
    }

    fn mark_attach_terminal_state(
        &mut self,
        attach_state: OrchestrationObligationAttachState,
        attach_completion_reason: impl Into<String>,
        settled_at: DateTime<Utc>,
    ) {
        self.attach_state = attach_state;
        self.attach_completion_reason = Some(attach_completion_reason.into());
        self.updated_at = settled_at;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_obligation_validates_minimum_shape() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::FollowUpRequired,
            "Need host follow-up",
        );
        obligation.attention_required = true;
        obligation.attach_state = OrchestrationObligationAttachState::Eligible;

        obligation.validate().expect("pending obligation validates");
    }

    #[test]
    fn resolved_obligation_requires_resolved_at() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::Blocked,
            "Waiting on host action",
        );
        obligation.state = OrchestrationObligationState::Resolved;
        obligation.review_state = OrchestrationObligationReviewState::Resolved;

        let err = obligation
            .validate()
            .expect_err("resolved obligations must carry resolved_at");
        assert!(err
            .to_string()
            .contains("resolved orchestration obligations must include resolved_at"));
    }

    #[test]
    fn world_binding_requires_both_id_and_generation() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::RuntimeAlert,
            "World reported a runtime alert",
        );
        obligation.world_id = Some("world-17".to_string());

        let err = obligation
            .validate()
            .expect_err("partial world binding must fail validation");
        assert!(err.to_string().contains("world binding"));
    }

    #[test]
    fn claimed_attach_state_requires_claim_metadata() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::ApprovalRequired,
            "Need host approval",
        );
        obligation.attach_state = OrchestrationObligationAttachState::Claimed;

        let err = obligation
            .validate()
            .expect_err("claimed attach state without metadata must fail validation");
        assert!(err
            .to_string()
            .contains("claimed orchestration obligations must record attach_attempt_count"));
    }

    #[test]
    fn terminal_attach_state_requires_completion_reason() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::FollowUpRequired,
            "Need host follow-up",
        );
        obligation.attach_state = OrchestrationObligationAttachState::Superseded;

        let err = obligation
            .validate()
            .expect_err("terminal attach state must include a completion reason");
        assert!(err.to_string().contains(
            "terminal orchestration attach states must include attach_completion_reason"
        ));
    }
}
