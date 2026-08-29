use anyhow::{Context as _, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use substrate_common::{
    agent_events::{AgentEvent, WorldWorkerEventClassV1, WorldWorkerEventV1},
    HostTransitionWorkCorrelationV1,
};
use transport_api_types::ExecuteStreamFrame;

use super::{
    host_session_authority::{canonical_json, schema::AuthorityObjectCommitmentV1},
    state_store::{
        is_stale_or_conflicting_c1_obligation_ledger_materialization_plan_error,
        AcceptedWorldWorkIdentityV1, AgentRuntimeStateStore, ResolvedWorldWorkRegistryAuthorityV1,
        WorldWorkAcceptanceRecordV1, WorldWorkReceiptRegistry,
    },
    world_work_execution_supervisor::{
        WorldWorkExecutionClaimV1, WorldWorkExecutionObservationV1, WorldWorkExecutionSupervisor,
    },
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SupervisorJournalEventRefV1 {
    pub(crate) schema_version: u32,
    pub(crate) journal_entry_id: String,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) accepted_work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) stream_id: String,
    pub(crate) frame_sequence: u64,
    pub(crate) event_id: String,
    pub(crate) event_sequence: u64,
    pub(crate) transport_event_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObligationSnapshotRecordStateV1 {
    UnresolvedAttention,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationSnapshotRecordHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) source_journal_event: SupervisorJournalEventRefV1,
    pub(crate) obligation_id: String,
    pub(crate) obligation_revision: u64,
    pub(crate) state: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedAttentionObligationSnapshotEntryV1 {
    pub(crate) obligation_id: String,
    pub(crate) obligation_revision: u64,
    pub(crate) canonical_record_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObligationAttentionDispositionV1 {
    NoUnresolvedAttention,
    HasUnresolvedAttention,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationMaterializationCutV1 {
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) stream_id: String,
    pub(crate) session_ledger_revision: u64,
    pub(crate) terminal_event_id: String,
    pub(crate) terminal_event_sequence: u64,
    pub(crate) materialized_through_event_sequence: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationSnapshotHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) stream_id: String,
    pub(crate) accepted_work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) host_transition_correlation: HostTransitionWorkCorrelationV1,
    pub(crate) transition_intent_id: String,
    pub(crate) transition_run_id: String,
    pub(crate) authority_revision_observed: u64,
    pub(crate) materialization_cut: ObligationMaterializationCutV1,
    pub(crate) materialized_journal_events: Vec<SupervisorJournalEventRefV1>,
    pub(crate) attention_disposition: ObligationAttentionDispositionV1,
    pub(crate) unresolved_attention_obligations: Vec<UnresolvedAttentionObligationSnapshotEntryV1>,
    pub(crate) captured_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(crate) enum ObligationLedgerSnapshotReadV1 {
    Pending {
        authority_store_id: String,
        orchestration_session_id: String,
        authoritative_participant_id: String,
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        stream_id: String,
        accepted_work_identity: AcceptedWorldWorkIdentityV1,
        host_transition_correlation: HostTransitionWorkCorrelationV1,
        transition_intent_id: String,
        transition_run_id: String,
        authority_revision_observed: u64,
        observed_session_ledger_revision: u64,
        required_terminal_event_id: String,
        required_terminal_event_sequence: u64,
    },
    Complete {
        snapshot: ObligationSnapshotHashInputV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationLedgerSnapshotReadRequestV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) stream_id: String,
    pub(crate) accepted_work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) host_transition_correlation: HostTransitionWorkCorrelationV1,
    pub(crate) transition_intent_id: String,
    pub(crate) transition_run_id: String,
    pub(crate) authority_revision_observed: u64,
    pub(crate) required_terminal_event_id: String,
    pub(crate) required_terminal_event_sequence: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationLedgerSessionStateV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) accepted_work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) stream_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    pub(crate) authority_revision_observed: u64,
    pub(crate) session_ledger_revision: u64,
    pub(crate) materialized_through_event_sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) terminal_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) terminal_event_sequence: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationLedgerRevisionCursorV1 {
    pub(crate) schema_version: u32,
    pub(crate) current_session_ledger_revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MaterializedObligationLedgerEventV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) accepted_work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) stream_id: String,
    pub(crate) source_journal_event: SupervisorJournalEventRefV1,
    pub(crate) event_class: WorldWorkerEventClassV1,
    pub(crate) attention_required: bool,
    pub(crate) emitted_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) obligation_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationLedgerMaterializationPlanV1 {
    pub(crate) expected_revision_cursor: ObligationLedgerRevisionCursorV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) expected_state: Option<ObligationLedgerSessionStateV1>,
    pub(crate) next_revision_cursor: ObligationLedgerRevisionCursorV1,
    pub(crate) next_state: ObligationLedgerSessionStateV1,
    pub(crate) materialized_events: Vec<MaterializedObligationLedgerEventV1>,
    pub(crate) projected_obligations: Vec<OrchestrationObligationRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerEventTransportHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) acceptance_record_id: String,
    pub(crate) stream_id: String,
    pub(crate) frame_sequence: u64,
    pub(crate) event_id: String,
    pub(crate) event_sequence: u64,
    pub(crate) request_id: String,
    pub(crate) active_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    pub(crate) causation_message_id: String,
    pub(crate) causation_request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) source_participant_id: String,
    pub(crate) target_participant_id: String,
    pub(crate) source_backend_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) thread_id: String,
    pub(crate) event_class: String,
    pub(crate) attention_required: bool,
    pub(crate) payload: Value,
    pub(crate) emitted_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObligationPayloadHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) payload: Value,
}

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

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApprovalObligationCloseoutDisposition {
    Resolve,
    Dismiss,
}

impl ApprovalObligationCloseoutDisposition {
    fn review_state(self) -> OrchestrationObligationReviewState {
        match self {
            Self::Resolve => OrchestrationObligationReviewState::Resolved,
            Self::Dismiss => OrchestrationObligationReviewState::Dismissed,
        }
    }
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

impl OrchestrationObligationAttachState {
    #[allow(dead_code)]
    pub(crate) fn forward_design_state(self) -> &'static str {
        match self {
            Self::NotEligible => "not_requested",
            Self::Eligible => "queued",
            Self::Claimed => "claimed",
            Self::Satisfied => "completed",
            Self::FailedClosed => "dead_letter",
            Self::Superseded => "cancelled",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LocalHostObligationTargetingDisposition {
    Untargeted,
    TargetedToLocalHost,
    WrongHost { target_host_id: String },
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
    pub authority_store_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authoritative_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obligation_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_journal_event: Option<SupervisorJournalEventRefV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_commitment: Option<AuthorityObjectCommitmentV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_record_commitment: Option<AuthorityObjectCommitmentV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingress_source_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingress_source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ingress_received_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_host_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_host_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_request_id: Option<String>,
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
            authority_store_id: None,
            authoritative_participant_id: None,
            obligation_revision: None,
            source_journal_event: None,
            payload_commitment: None,
            canonical_record_commitment: None,
            resolution_note: None,
            source_participant_id: None,
            ingress_source_kind: None,
            ingress_source_id: None,
            ingress_received_at: None,
            origin_host_id: None,
            target_host_id: None,
            causation_event_id: None,
            causation_message_id: None,
            causation_request_id: None,
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
        validate_optional_exact_identity(self.authority_store_id.as_deref(), "authority_store_id")?;
        validate_optional_exact_identity(
            self.authoritative_participant_id.as_deref(),
            "authoritative_participant_id",
        )?;
        validate_optional_exact_identity(
            self.ingress_source_kind.as_deref(),
            "ingress_source_kind",
        )?;
        validate_optional_exact_identity(self.ingress_source_id.as_deref(), "ingress_source_id")?;
        validate_optional_host_id(self.origin_host_id.as_deref(), "origin_host_id")?;
        validate_optional_host_id(self.target_host_id.as_deref(), "target_host_id")?;
        validate_optional_exact_identity(self.causation_event_id.as_deref(), "causation_event_id")?;
        validate_optional_exact_identity(
            self.causation_message_id.as_deref(),
            "causation_message_id",
        )?;
        validate_optional_exact_identity(
            self.causation_request_id.as_deref(),
            "causation_request_id",
        )?;
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
        if self.source_journal_event.is_some()
            || self.authority_store_id.is_some()
            || self.authoritative_participant_id.is_some()
            || self.obligation_revision.is_some()
            || self.payload_commitment.is_some()
            || self.canonical_record_commitment.is_some()
        {
            self.validate_c1_materialization_fields()?;
        }
        if self.state == OrchestrationObligationState::Pending
            && matches!(
                self.review_state,
                OrchestrationObligationReviewState::Dismissed
                    | OrchestrationObligationReviewState::Resolved
            )
        {
            anyhow::bail!(
                "pending orchestration obligations cannot advertise terminal review_state"
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

    #[allow(dead_code)]
    pub(crate) fn classify_local_host_targeting(
        &self,
        local_host_id: &str,
    ) -> Result<LocalHostObligationTargetingDisposition> {
        validate_host_id(local_host_id, "local_host_id")?;

        match self.target_host_id.as_deref() {
            None => Ok(LocalHostObligationTargetingDisposition::Untargeted),
            Some(target_host_id) if target_host_id == local_host_id => {
                Ok(LocalHostObligationTargetingDisposition::TargetedToLocalHost)
            }
            Some(target_host_id) => Ok(LocalHostObligationTargetingDisposition::WrongHost {
                target_host_id: target_host_id.to_string(),
            }),
        }
    }

    pub(crate) fn is_pending(&self) -> bool {
        self.state.is_pending()
    }

    pub(crate) fn projects_detached_attention(&self) -> bool {
        self.is_pending() && self.attention_required
    }

    pub(crate) fn has_c1_materialization_identity(&self) -> bool {
        self.authority_store_id.is_some()
            && self.authoritative_participant_id.is_some()
            && self.obligation_revision.is_some()
            && self.source_journal_event.is_some()
            && self.payload_commitment.is_some()
            && self.canonical_record_commitment.is_some()
    }

    pub(crate) fn matches_c1_materialization_identity(
        &self,
        other: &OrchestrationObligationRecord,
    ) -> bool {
        self.obligation_id == other.obligation_id
            && self.orchestration_session_id == other.orchestration_session_id
            && self.authority_store_id == other.authority_store_id
            && self.authoritative_participant_id == other.authoritative_participant_id
            && self.obligation_revision == other.obligation_revision
            && self.source_journal_event == other.source_journal_event
            && self.payload_commitment == other.payload_commitment
            && self.canonical_record_commitment == other.canonical_record_commitment
    }

    pub(crate) fn matches_c1_materialization_projection(
        &self,
        other: &OrchestrationObligationRecord,
    ) -> bool {
        self.matches_c1_materialization_identity(other)
            && self.kind == other.kind
            && self.severity == other.severity
            && self.created_at == other.created_at
            && self.summary == other.summary
            && self.source_participant_id == other.source_participant_id
            && self.ingress_source_kind == other.ingress_source_kind
            && self.ingress_source_id == other.ingress_source_id
            && self.ingress_received_at == other.ingress_received_at
            && self.origin_host_id == other.origin_host_id
            && self.target_host_id == other.target_host_id
            && self.causation_event_id == other.causation_event_id
            && self.causation_message_id == other.causation_message_id
            && self.causation_request_id == other.causation_request_id
            && self.target_backend_id == other.target_backend_id
            && self.world_id == other.world_id
            && self.world_generation == other.world_generation
            && self.payload == other.payload
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

    pub(crate) fn release_attach_claim(&mut self, released_at: DateTime<Utc>) {
        self.attach_state = OrchestrationObligationAttachState::Eligible;
        self.attach_claim_owner = None;
        self.attach_completion_reason = None;
        self.updated_at = released_at;
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

    #[allow(dead_code)]
    pub(crate) fn mark_approval_response_closed(
        &mut self,
        disposition: ApprovalObligationCloseoutDisposition,
        resolution_note: Option<String>,
        resolved_at: DateTime<Utc>,
    ) {
        self.state = OrchestrationObligationState::Resolved;
        self.review_state = disposition.review_state();
        self.attention_required = false;
        self.resolution_note = resolution_note;
        self.resolved_at = Some(resolved_at);
        self.updated_at = resolved_at;
    }

    #[allow(dead_code)]
    pub(crate) fn mark_clarification_response_closed(
        &mut self,
        resolution_note: Option<String>,
        resolved_at: DateTime<Utc>,
    ) {
        self.state = OrchestrationObligationState::Resolved;
        self.review_state = OrchestrationObligationReviewState::Resolved;
        self.attention_required = false;
        self.resolution_note = resolution_note;
        self.resolved_at = Some(resolved_at);
        self.updated_at = resolved_at;
    }

    fn validate_c1_materialization_fields(&self) -> Result<()> {
        let authority_store_id = self.authority_store_id.as_deref().ok_or_else(|| {
            anyhow::anyhow!("C1 materialized obligations must include authority_store_id")
        })?;
        let authoritative_participant_id = self
            .authoritative_participant_id
            .as_deref()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "C1 materialized obligations must include authoritative_participant_id"
                )
            })?;
        let obligation_revision = self.obligation_revision.ok_or_else(|| {
            anyhow::anyhow!("C1 materialized obligations must include obligation_revision")
        })?;
        let source_journal_event = self.source_journal_event.as_ref().ok_or_else(|| {
            anyhow::anyhow!("C1 materialized obligations must include source_journal_event")
        })?;
        let payload_commitment = self.payload_commitment.as_ref().ok_or_else(|| {
            anyhow::anyhow!("C1 materialized obligations must include payload_commitment")
        })?;
        let canonical_record_commitment =
            self.canonical_record_commitment.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "C1 materialized obligations must include canonical_record_commitment"
                )
            })?;

        validate_exact_identity(authority_store_id, "authority_store_id")?;
        validate_exact_identity(authoritative_participant_id, "authoritative_participant_id")?;
        if obligation_revision == 0 {
            anyhow::bail!("C1 materialized obligations must use positive obligation_revision");
        }
        if self.is_pending() && !self.attention_required {
            anyhow::bail!("pending C1 materialized obligations must require attention");
        }
        if !self.is_pending() && self.attention_required {
            anyhow::bail!("resolved C1 materialized obligations must not require attention");
        }
        source_journal_event.validate()?;
        validate_canonical_commitment(payload_commitment, "payload_commitment")?;
        validate_canonical_commitment(canonical_record_commitment, "canonical_record_commitment")?;
        if self.causation_event_id.as_deref() != Some(source_journal_event.event_id.as_str()) {
            anyhow::bail!(
                "C1 materialized obligations must bind causation_event_id to source_journal_event.event_id"
            );
        }
        if let Some(world_id) = self.world_id.as_deref() {
            if world_id.trim().is_empty() {
                anyhow::bail!("C1 materialized obligations must not persist an empty world_id");
            }
        }
        if let Some(transport_commitment) = Some(&source_journal_event.transport_event_commitment) {
            validate_canonical_commitment(
                transport_commitment,
                "source_journal_event.transport_event_commitment",
            )?;
        }
        let expected_payload_commitment =
            obligation_payload_commitment(self.payload.as_ref().ok_or_else(|| {
                anyhow::anyhow!("C1 materialized obligations must include payload")
            })?)?;
        if &expected_payload_commitment != payload_commitment {
            anyhow::bail!("C1 materialized obligation payload commitment drifted");
        }
        let expected_record_commitment = obligation_snapshot_record_commitment(
            authority_store_id,
            &self.orchestration_session_id,
            authoritative_participant_id,
            source_journal_event,
            &self.obligation_id,
            obligation_revision,
        )?;
        if &expected_record_commitment != canonical_record_commitment {
            anyhow::bail!("C1 materialized obligation record commitment drifted");
        }
        Ok(())
    }
}

fn validate_optional_host_id(host_id: Option<&str>, field_name: &str) -> Result<()> {
    if let Some(host_id) = host_id {
        validate_host_id(host_id, field_name)?;
    }
    Ok(())
}

fn validate_host_id(host_id: &str, field_name: &str) -> Result<()> {
    if host_id.trim().is_empty() {
        anyhow::bail!("orchestration obligations must not persist an empty {field_name}");
    }
    Ok(())
}

fn validate_optional_exact_identity(field_value: Option<&str>, field_name: &str) -> Result<()> {
    if field_value.is_some_and(|value| value.trim().is_empty()) {
        anyhow::bail!("orchestration obligations must not persist an empty {field_name}");
    }
    Ok(())
}

fn validate_exact_identity(field_value: &str, field_name: &str) -> Result<()> {
    if field_value.trim().is_empty() {
        anyhow::bail!("orchestration obligations must not persist an empty {field_name}");
    }
    Ok(())
}

fn validate_prefixed_uuid_v7(value: &str, prefix: &str, field_name: &'static str) -> Result<()> {
    let Some(body) = value.strip_prefix(prefix) else {
        anyhow::bail!("{field_name}");
    };
    if body.len() != 36
        || !body
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
    {
        anyhow::bail!("{field_name}");
    }
    Ok(())
}

fn validate_accepted_work_identity(value: &AcceptedWorldWorkIdentityV1) -> Result<()> {
    match value {
        AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } => {
            if task_run_id.trim().is_empty() {
                anyhow::bail!("accepted work identity must include task_run_id");
            }
        }
        AcceptedWorldWorkIdentityV1::RetainedTurn {
            active_run_id,
            message_id,
            target_participant_id,
        } => {
            if active_run_id.trim().is_empty()
                || message_id.trim().is_empty()
                || target_participant_id.trim().is_empty()
            {
                anyhow::bail!(
                    "accepted work identity must include active_run_id/message_id/target_participant_id"
                );
            }
            validate_prefixed_uuid_v7(
                message_id,
                "wwm_",
                "retained work identity message_id is invalid",
            )?;
        }
    }
    Ok(())
}

fn canonical_sha256<T: Serialize>(value: &T) -> Result<String> {
    let canonical =
        canonical_json::to_vec(value).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(lower_hex(&Sha256::digest(canonical)))
}

fn validate_canonical_commitment(
    commitment: &AuthorityObjectCommitmentV1,
    field_name: &'static str,
) -> Result<()> {
    match commitment {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex }
            if digest_hex.len() == 64
                && digest_hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) =>
        {
            Ok(())
        }
        _ => anyhow::bail!("{field_name} must be a canonical SHA-256 commitment"),
    }
}

pub(crate) fn obligation_payload_commitment(
    payload: &Value,
) -> Result<AuthorityObjectCommitmentV1> {
    ObligationPayloadHashInputV1 {
        schema_version: 1,
        payload: payload.clone(),
    }
    .validate()?;
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&ObligationPayloadHashInputV1 {
            schema_version: 1,
            payload: payload.clone(),
        })
        .context("compute C1 obligation payload commitment")?,
    })
}

pub(crate) fn worker_event_transport_commitment(
    worker_event: &WorldWorkerEventV1,
) -> Result<AuthorityObjectCommitmentV1> {
    let hash_input = WorkerEventTransportHashInputV1::from_worker_event(worker_event);
    hash_input.validate()?;
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&hash_input)
            .context("compute C1 worker event transport commitment")?,
    })
}

pub(crate) fn obligation_snapshot_record_commitment(
    authority_store_id: &str,
    orchestration_session_id: &str,
    authoritative_participant_id: &str,
    source_journal_event: &SupervisorJournalEventRefV1,
    obligation_id: &str,
    obligation_revision: u64,
) -> Result<AuthorityObjectCommitmentV1> {
    ObligationSnapshotRecordHashInputV1 {
        schema_version: 1,
        authority_store_id: authority_store_id.to_string(),
        orchestration_session_id: orchestration_session_id.to_string(),
        authoritative_participant_id: authoritative_participant_id.to_string(),
        source_journal_event: source_journal_event.clone(),
        obligation_id: obligation_id.to_string(),
        obligation_revision,
        state: c1_obligation_snapshot_record_state_label(
            ObligationSnapshotRecordStateV1::UnresolvedAttention,
        )
        .to_string(),
    }
    .validate()?;
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&ObligationSnapshotRecordHashInputV1 {
            schema_version: 1,
            authority_store_id: authority_store_id.to_string(),
            orchestration_session_id: orchestration_session_id.to_string(),
            authoritative_participant_id: authoritative_participant_id.to_string(),
            source_journal_event: source_journal_event.clone(),
            obligation_id: obligation_id.to_string(),
            obligation_revision,
            state: c1_obligation_snapshot_record_state_label(
                ObligationSnapshotRecordStateV1::UnresolvedAttention,
            )
            .to_string(),
        })
        .context("compute C1 obligation snapshot record commitment")?,
    })
}

impl SupervisorJournalEventRefV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || self.acceptance_record_revision == 0 {
            anyhow::bail!("C1 supervisor journal event ref must use schema/revision 1");
        }
        validate_exact_identity(&self.journal_entry_id, "journal_entry_id")?;
        validate_exact_identity(&self.acceptance_record_id, "acceptance_record_id")?;
        validate_exact_identity(&self.stream_id, "stream_id")?;
        validate_exact_identity(&self.event_id, "event_id")?;
        if self.frame_sequence == 0 || self.event_sequence == 0 {
            anyhow::bail!("C1 supervisor journal event ref sequences must be positive");
        }
        validate_accepted_work_identity(&self.accepted_work_identity)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        validate_canonical_commitment(
            &self.transport_event_commitment,
            "transport_event_commitment",
        )?;
        Ok(())
    }
}

impl ObligationLedgerSnapshotReadRequestV1 {
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "C1 defines the closed snapshot read contract before A1.2b adopts it"
        )
    )]
    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            anyhow::bail!("C1 obligation snapshot read request must use schema version 1");
        }
        validate_exact_identity(&self.authority_store_id, "authority_store_id")?;
        validate_exact_identity(&self.orchestration_session_id, "orchestration_session_id")?;
        validate_exact_identity(
            &self.authoritative_participant_id,
            "authoritative_participant_id",
        )?;
        validate_exact_identity(&self.acceptance_record_id, "acceptance_record_id")?;
        validate_exact_identity(&self.stream_id, "stream_id")?;
        validate_exact_identity(
            &self.required_terminal_event_id,
            "required_terminal_event_id",
        )?;
        if self.acceptance_record_revision == 0
            || self.authority_revision_observed == 0
            || self.required_terminal_event_sequence == 0
        {
            anyhow::bail!(
                "C1 obligation snapshot read request revisions/sequences must be positive"
            );
        }
        validate_accepted_work_identity(&self.accepted_work_identity)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        self.host_transition_correlation
            .validate()
            .map_err(anyhow::Error::msg)?;
        if self.transition_intent_id != self.host_transition_correlation.transition_intent_id
            || self.transition_run_id != self.host_transition_correlation.transition_run_id
            || self.authority_revision_observed
                != self.host_transition_correlation.authority_revision_observed
        {
            anyhow::bail!("C1 obligation snapshot read request transition scope drifted");
        }
        Ok(())
    }
}

impl ObligationLedgerSessionStateV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || self.session_ledger_revision == 0
            || self.acceptance_record_revision == 0
        {
            anyhow::bail!(
                "C1 obligation ledger state must use schema version 1 and positive revisions"
            );
        }
        validate_exact_identity(&self.authority_store_id, "authority_store_id")?;
        validate_exact_identity(&self.orchestration_session_id, "orchestration_session_id")?;
        validate_exact_identity(
            &self.authoritative_participant_id,
            "authoritative_participant_id",
        )?;
        validate_exact_identity(&self.acceptance_record_id, "acceptance_record_id")?;
        validate_exact_identity(&self.stream_id, "stream_id")?;
        if self.authority_revision_observed == 0 {
            anyhow::bail!(
                "C1 obligation ledger state must include positive authority_revision_observed"
            );
        }
        validate_accepted_work_identity(&self.accepted_work_identity)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        if let Some(correlation) = self.host_transition_correlation.as_ref() {
            correlation.validate().map_err(anyhow::Error::msg)?;
            if correlation.authority_store_id != self.authority_store_id
                || correlation.orchestration_session_id != self.orchestration_session_id
                || correlation.authoritative_participant_id != self.authoritative_participant_id
                || correlation.authority_revision_observed != self.authority_revision_observed
            {
                anyhow::bail!("C1 obligation ledger state transition scope drifted");
            }
        }
        match (
            self.terminal_event_id.as_deref(),
            self.terminal_event_sequence,
        ) {
            (Some(event_id), Some(event_sequence)) => {
                validate_exact_identity(event_id, "terminal_event_id")?;
                if event_sequence == 0
                    || self.materialized_through_event_sequence < event_sequence
                {
                    anyhow::bail!(
                        "C1 obligation ledger terminal cut must not exceed the materialized watermark"
                    );
                }
            }
            (None, None) => {}
            _ => anyhow::bail!(
                "C1 obligation ledger terminal cut must provide both terminal_event_id and terminal_event_sequence"
            ),
        }
        Ok(())
    }
}

impl ObligationLedgerRevisionCursorV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            anyhow::bail!("C1 obligation ledger revision cursor must use schema version 1");
        }
        Ok(())
    }
}

impl MaterializedObligationLedgerEventV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || self.acceptance_record_revision == 0 {
            anyhow::bail!("C1 materialized event must use schema/revision 1");
        }
        validate_exact_identity(&self.authority_store_id, "authority_store_id")?;
        validate_exact_identity(&self.orchestration_session_id, "orchestration_session_id")?;
        validate_exact_identity(
            &self.authoritative_participant_id,
            "authoritative_participant_id",
        )?;
        validate_exact_identity(&self.acceptance_record_id, "acceptance_record_id")?;
        validate_exact_identity(&self.stream_id, "stream_id")?;
        validate_accepted_work_identity(&self.accepted_work_identity)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        self.source_journal_event.validate()?;
        if self.source_journal_event.acceptance_record_id != self.acceptance_record_id
            || self.source_journal_event.acceptance_record_revision
                != self.acceptance_record_revision
            || self.source_journal_event.accepted_work_identity != self.accepted_work_identity
            || self.source_journal_event.stream_id != self.stream_id
        {
            anyhow::bail!("C1 materialized event scope does not match its source journal ref");
        }
        if self.event_class.attention_required_by_default() && !self.attention_required {
            anyhow::bail!("C1 materialized event attention flag drifted");
        }
        Ok(())
    }
}

impl ObligationLedgerMaterializationPlanV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        self.expected_revision_cursor.validate()?;
        if let Some(state) = self.expected_state.as_ref() {
            state.validate()?;
        }
        self.next_revision_cursor.validate()?;
        self.next_state.validate()?;
        if self.next_revision_cursor.current_session_ledger_revision
            < self
                .expected_revision_cursor
                .current_session_ledger_revision
        {
            anyhow::bail!("C1 materialization plan session ledger revision regressed");
        }
        for event in &self.materialized_events {
            event.validate()?;
            if event.orchestration_session_id != self.next_state.orchestration_session_id
                || event.acceptance_record_id != self.next_state.acceptance_record_id
            {
                anyhow::bail!("C1 materialization plan event/session scope mismatch");
            }
        }
        for obligation in &self.projected_obligations {
            obligation.validate()?;
            if !obligation.has_c1_materialization_identity() {
                anyhow::bail!(
                    "C1 materialization plan projected obligation omitted canonical identity"
                );
            }
            if obligation.orchestration_session_id != self.next_state.orchestration_session_id
                || obligation.authority_store_id.as_deref()
                    != Some(self.next_state.authority_store_id.as_str())
                || obligation.authoritative_participant_id.as_deref()
                    != Some(self.next_state.authoritative_participant_id.as_str())
            {
                anyhow::bail!("C1 materialization plan projected obligation scope mismatch");
            }
        }
        Ok(())
    }
}

impl ObligationSnapshotRecordHashInputV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || self.obligation_revision == 0 {
            anyhow::bail!("invalid C1 obligation snapshot record revision");
        }
        if self.authority_store_id.trim().is_empty()
            || self.orchestration_session_id.trim().is_empty()
            || self.authoritative_participant_id.trim().is_empty()
            || self.obligation_id.trim().is_empty()
        {
            anyhow::bail!("invalid C1 obligation snapshot record identity");
        }
        parse_c1_obligation_snapshot_record_state_label(&self.state)?;
        self.source_journal_event.validate()?;
        Ok(())
    }
}

impl WorkerEventTransportHashInputV1 {
    pub(crate) fn from_worker_event(worker_event: &WorldWorkerEventV1) -> Self {
        Self {
            schema_version: 1,
            acceptance_record_id: worker_event.acceptance_record_id.clone(),
            stream_id: worker_event.stream_id.clone(),
            frame_sequence: worker_event.frame_sequence,
            event_id: worker_event.event_id.clone(),
            event_sequence: worker_event.event_sequence,
            request_id: worker_event.request_id.clone(),
            active_run_id: worker_event.active_run_id.clone(),
            host_transition_correlation: worker_event.host_transition_correlation.clone(),
            causation_message_id: worker_event.causation_message_id.clone(),
            causation_request_id: worker_event.causation_request_id.clone(),
            orchestration_session_id: worker_event.orchestration_session_id.clone(),
            source_participant_id: worker_event.source_participant_id.clone(),
            target_participant_id: worker_event.target_participant_id.clone(),
            source_backend_id: worker_event.source_backend_id.clone(),
            target_backend_id: worker_event.target_backend_id.clone(),
            world_id: worker_event.world_id.clone(),
            world_generation: worker_event.world_generation,
            thread_id: worker_event.thread_id.clone(),
            event_class: c1_event_class_label(worker_event.event_class).to_string(),
            attention_required: worker_event.attention_required,
            payload: worker_event.payload.clone(),
            emitted_at: worker_event.emitted_at,
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            anyhow::bail!("invalid C1 worker event transport schema version");
        }
        WorldWorkerEventV1 {
            schema_version: 1,
            acceptance_record_id: self.acceptance_record_id.clone(),
            stream_id: self.stream_id.clone(),
            frame_sequence: self.frame_sequence,
            event_id: self.event_id.clone(),
            event_sequence: self.event_sequence,
            request_id: self.request_id.clone(),
            active_run_id: self.active_run_id.clone(),
            host_transition_correlation: self.host_transition_correlation.clone(),
            causation_message_id: self.causation_message_id.clone(),
            causation_request_id: self.causation_request_id.clone(),
            orchestration_session_id: self.orchestration_session_id.clone(),
            source_participant_id: self.source_participant_id.clone(),
            target_participant_id: self.target_participant_id.clone(),
            source_backend_id: self.source_backend_id.clone(),
            target_backend_id: self.target_backend_id.clone(),
            world_id: self.world_id.clone(),
            world_generation: self.world_generation,
            thread_id: self.thread_id.clone(),
            event_class: parse_c1_event_class_label(&self.event_class)?,
            attention_required: self.attention_required,
            payload: self.payload.clone(),
            emitted_at: self.emitted_at,
        }
        .validate()
        .map_err(anyhow::Error::msg)
    }
}

impl ObligationPayloadHashInputV1 {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            anyhow::bail!("invalid C1 obligation payload schema version");
        }
        Ok(())
    }
}

pub(crate) fn reconcile_retained_event_obligations(
    store: &AgentRuntimeStateStore,
    receipt_registry: &WorldWorkReceiptRegistry,
    execution_supervisor: &WorldWorkExecutionSupervisor,
    acceptance_record_id: &str,
) -> Result<ObligationLedgerSessionStateV1> {
    const MAX_RETRIES: usize = 8;

    for _attempt in 0..MAX_RETRIES {
        let observation = execution_supervisor
            .inspect_observation_by_acceptance_id(acceptance_record_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "C1 obligation materialization requires exact B2.1 observation {}",
                    acceptance_record_id
                )
            })?;
        let acceptance = receipt_registry
            .inspect_world_work_acceptance_by_id(
                &observation.claim.authority_store_id,
                acceptance_record_id,
            )?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "C1 obligation materialization requires exact B1 acceptance {}",
                    acceptance_record_id
                )
            })?;
        validate_retained_acceptance_and_claim(&acceptance, &observation.claim)?;

        let current_cursor =
            store.load_obligation_ledger_revision_cursor(&acceptance.orchestration_session_id)?;
        let current_state = store.load_obligation_ledger_state(
            &acceptance.orchestration_session_id,
            &acceptance.acceptance_record_id,
        )?;
        if let Some(state) = current_state.as_ref() {
            validate_state_matches_acceptance(state, &acceptance)?;
        }
        let expected = expected_retained_obligation_materialization_from_observation(
            &acceptance,
            &observation,
        )?;
        if current_state.as_ref().is_some_and(|state| {
            state.materialized_through_event_sequence > expected.materialized_through_event_sequence
        }) {
            anyhow::bail!(
                "C1 obligation ledger watermark advanced beyond the exact observed event cut"
            );
        }
        if current_state.as_ref().is_some_and(|state| {
            state.terminal_event_id.is_some()
                && (state.terminal_event_id != expected.terminal_event_id
                    || state.terminal_event_sequence != expected.terminal_event_sequence)
        }) {
            anyhow::bail!(
                "C1 obligation ledger terminal cut drifted from the exact supervisor observation"
            );
        }

        let mut materialized_events = Vec::new();
        let mut projected_obligations = Vec::new();
        let stored_events = store.list_materialized_obligation_ledger_events(
            &acceptance.orchestration_session_id,
            &acceptance.acceptance_record_id,
        )?;
        let mut stored_events_by_sequence = std::collections::BTreeMap::new();
        for event in stored_events {
            let sequence = event.source_journal_event.event_sequence;
            let Some(expected_event) = expected
                .materialized_events
                .iter()
                .find(|candidate| candidate.source_journal_event.event_sequence == sequence)
            else {
                anyhow::bail!(
                    "unexpected C1 materialized event {} outside the exact retained event cut",
                    event.source_journal_event.event_id
                );
            };
            if &event != expected_event {
                anyhow::bail!(
                    "conflicting duplicate C1 materialized event {}",
                    event.source_journal_event.event_id
                );
            }
            stored_events_by_sequence.insert(sequence, event);
        }
        for expected_event in &expected.materialized_events {
            if !stored_events_by_sequence
                .contains_key(&expected_event.source_journal_event.event_sequence)
            {
                materialized_events.push(expected_event.clone());
            }
        }

        let stored_obligations = store.list_materialized_obligation_ledger_obligations(
            &acceptance.orchestration_session_id,
            &acceptance.acceptance_record_id,
        )?;
        let mut stored_obligations_by_id = std::collections::BTreeMap::new();
        for obligation in stored_obligations {
            let Some(expected_obligation) = expected
                .projected_obligations
                .iter()
                .find(|candidate| candidate.obligation_id == obligation.obligation_id)
            else {
                anyhow::bail!(
                    "unexpected C1 materialized obligation {} outside the exact retained event cut",
                    obligation.obligation_id
                );
            };
            if !obligation.matches_c1_materialization_projection(expected_obligation) {
                anyhow::bail!(
                    "conflicting duplicate C1 obligation {}",
                    obligation.obligation_id
                );
            }
            stored_obligations_by_id.insert(obligation.obligation_id.clone(), obligation);
        }
        for expected_obligation in &expected.projected_obligations {
            if !stored_obligations_by_id.contains_key(&expected_obligation.obligation_id) {
                projected_obligations.push(expected_obligation.clone());
            }
        }

        let mut state_changed = current_state.is_none()
            || current_state.as_ref().is_some_and(|state| {
                state.materialized_through_event_sequence
                    != expected.materialized_through_event_sequence
            });
        if current_state
            .as_ref()
            .and_then(|state| state.terminal_event_id.as_deref())
            != expected.terminal_event_id.as_deref()
            || current_state
                .as_ref()
                .and_then(|state| state.terminal_event_sequence)
                != expected.terminal_event_sequence
        {
            state_changed = true;
        }

        let next_revision_cursor = if state_changed {
            ObligationLedgerRevisionCursorV1 {
                schema_version: 1,
                current_session_ledger_revision: current_cursor
                    .current_session_ledger_revision
                    .saturating_add(1),
            }
        } else {
            current_cursor.clone()
        };
        let next_state = ObligationLedgerSessionStateV1 {
            schema_version: 1,
            authority_store_id: acceptance.authority_store_id.clone(),
            orchestration_session_id: acceptance.orchestration_session_id.clone(),
            authoritative_participant_id: acceptance.caller_participant_id.clone(),
            acceptance_record_id: acceptance.acceptance_record_id.clone(),
            acceptance_record_revision: acceptance.record_revision,
            accepted_work_identity: acceptance.work_identity.clone(),
            stream_id: acceptance.runtime_acceptance.stream_id.clone(),
            host_transition_correlation: acceptance.host_transition_correlation.clone(),
            authority_revision_observed: acceptance.authority_revision_observed,
            session_ledger_revision: next_revision_cursor.current_session_ledger_revision,
            materialized_through_event_sequence: expected.materialized_through_event_sequence,
            terminal_event_id: expected.terminal_event_id.clone(),
            terminal_event_sequence: expected.terminal_event_sequence,
        };
        next_state.validate()?;

        if state_changed || !materialized_events.is_empty() || !projected_obligations.is_empty() {
            match store.apply_obligation_ledger_materialization_plan(
                &ObligationLedgerMaterializationPlanV1 {
                    expected_revision_cursor: current_cursor,
                    expected_state: current_state,
                    next_revision_cursor,
                    next_state: next_state.clone(),
                    materialized_events,
                    projected_obligations,
                },
            ) {
                Ok(()) => return Ok(next_state),
                Err(err)
                    if is_stale_or_conflicting_c1_obligation_ledger_materialization_plan_error(
                        &err,
                    ) =>
                {
                    continue;
                }
                Err(err) => return Err(err),
            }
        }

        return Ok(next_state);
    }

    anyhow::bail!(
        "C1 obligation materialization failed to converge after concurrent session updates"
    )
}

#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "C1 defines the closed snapshot read contract before A1.2b adopts it"
    )
)]
pub(crate) fn read_obligation_ledger_snapshot(
    store: &AgentRuntimeStateStore,
    receipt_registry: &WorldWorkReceiptRegistry,
    execution_supervisor: &WorldWorkExecutionSupervisor,
    request: &ObligationLedgerSnapshotReadRequestV1,
) -> Result<ObligationLedgerSnapshotReadV1> {
    request.validate()?;
    let state = reconcile_retained_event_obligations(
        store,
        receipt_registry,
        execution_supervisor,
        &request.acceptance_record_id,
    )?;
    let acceptance = receipt_registry
        .inspect_world_work_acceptance_by_id(
            &request.authority_store_id,
            &request.acceptance_record_id,
        )?
        .ok_or_else(|| anyhow::anyhow!("C1 snapshot acceptance disappeared"))?;
    let observation = execution_supervisor
        .inspect_observation_by_acceptance_id(&request.acceptance_record_id)?
        .ok_or_else(|| anyhow::anyhow!("C1 snapshot observation disappeared"))?;
    validate_retained_acceptance_and_claim(&acceptance, &observation.claim)?;
    validate_snapshot_request_against_acceptance(request, &acceptance, &observation.claim)?;
    validate_state_matches_acceptance(&state, &acceptance)?;

    let expected =
        expected_retained_obligation_materialization_from_observation(&acceptance, &observation)?;
    if let Some(terminal) = observation.terminal.as_ref() {
        if terminal.event_identity.event_id != request.required_terminal_event_id
            || terminal.event_identity.event_sequence != request.required_terminal_event_sequence
        {
            anyhow::bail!(
                "C1 snapshot request terminal cut mismatched the exact supervisor observation"
            );
        }
    }
    let stored_events = store.list_materialized_obligation_ledger_events(
        &request.orchestration_session_id,
        &request.acceptance_record_id,
    )?;
    let complete_cut = observation.terminal.as_ref().is_some_and(|terminal| {
        terminal.event_identity.event_id == request.required_terminal_event_id
            && terminal.event_identity.event_sequence == request.required_terminal_event_sequence
            && state.terminal_event_id.as_deref()
                == Some(request.required_terminal_event_id.as_str())
            && state.terminal_event_sequence == Some(request.required_terminal_event_sequence)
            && state.materialized_through_event_sequence == request.required_terminal_event_sequence
            && stored_events == expected.materialized_events
    });

    if !complete_cut {
        return Ok(ObligationLedgerSnapshotReadV1::Pending {
            authority_store_id: request.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            authoritative_participant_id: request.authoritative_participant_id.clone(),
            acceptance_record_id: request.acceptance_record_id.clone(),
            acceptance_record_revision: request.acceptance_record_revision,
            stream_id: request.stream_id.clone(),
            accepted_work_identity: request.accepted_work_identity.clone(),
            host_transition_correlation: request.host_transition_correlation.clone(),
            transition_intent_id: request.transition_intent_id.clone(),
            transition_run_id: request.transition_run_id.clone(),
            authority_revision_observed: request.authority_revision_observed,
            observed_session_ledger_revision: state.session_ledger_revision,
            required_terminal_event_id: request.required_terminal_event_id.clone(),
            required_terminal_event_sequence: request.required_terminal_event_sequence,
        });
    }

    let stored_obligations = materialized_obligations_for_exact_event_cut(
        store,
        &request.orchestration_session_id,
        &request.acceptance_record_id,
        &expected.projected_obligations,
    )?;
    let mut unresolved =
        unresolved_attention_entries_for_materialized_obligations(&stored_obligations)?;
    sort_unresolved_attention_entries(&mut unresolved);
    let attention_disposition = if unresolved.is_empty() {
        ObligationAttentionDispositionV1::NoUnresolvedAttention
    } else {
        ObligationAttentionDispositionV1::HasUnresolvedAttention
    };
    let captured_at = expected
        .materialized_events
        .last()
        .map(|event| event.emitted_at)
        .unwrap_or(acceptance.runtime_acceptance.observed_at);
    Ok(ObligationLedgerSnapshotReadV1::Complete {
        snapshot: ObligationSnapshotHashInputV1 {
            schema_version: 1,
            authority_store_id: request.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            authoritative_participant_id: request.authoritative_participant_id.clone(),
            acceptance_record_id: request.acceptance_record_id.clone(),
            acceptance_record_revision: request.acceptance_record_revision,
            stream_id: request.stream_id.clone(),
            accepted_work_identity: request.accepted_work_identity.clone(),
            host_transition_correlation: request.host_transition_correlation.clone(),
            transition_intent_id: request.transition_intent_id.clone(),
            transition_run_id: request.transition_run_id.clone(),
            authority_revision_observed: request.authority_revision_observed,
            materialization_cut: ObligationMaterializationCutV1 {
                acceptance_record_id: request.acceptance_record_id.clone(),
                acceptance_record_revision: request.acceptance_record_revision,
                stream_id: request.stream_id.clone(),
                session_ledger_revision: state.session_ledger_revision,
                terminal_event_id: request.required_terminal_event_id.clone(),
                terminal_event_sequence: request.required_terminal_event_sequence,
                materialized_through_event_sequence: state.materialized_through_event_sequence,
            },
            materialized_journal_events: expected
                .materialized_events
                .into_iter()
                .map(|event| event.source_journal_event)
                .collect(),
            attention_disposition,
            unresolved_attention_obligations: unresolved,
            captured_at,
        },
    })
}

pub(crate) fn read_exact_start_obligation_ledger_snapshot(
    store: &AgentRuntimeStateStore,
    authority: &ResolvedWorldWorkRegistryAuthorityV1,
    expected_correlation: &HostTransitionWorkCorrelationV1,
) -> Result<Option<ObligationLedgerSnapshotReadV1>> {
    expected_correlation
        .validate()
        .map_err(anyhow::Error::msg)?;
    if authority.authority_store_id != expected_correlation.authority_store_id
        || authority.orchestration_session_id != expected_correlation.orchestration_session_id
        || authority.caller_participant_id != expected_correlation.authoritative_participant_id
        || authority.authority_revision_observed != expected_correlation.authority_revision_observed
    {
        anyhow::bail!("Start obligation consumer authority does not authenticate");
    }

    let persisted = authority
        .receipt_registry
        .persisted_acceptances_for_recovery()?;
    let mut exact = None;
    for acceptance in persisted {
        let record = acceptance.record();
        let Some(correlation) = record.host_transition_correlation.as_ref() else {
            continue;
        };
        if correlation.transition_intent_id != expected_correlation.transition_intent_id
            || correlation.transition_run_id != expected_correlation.transition_run_id
        {
            continue;
        }
        if correlation != expected_correlation
            || record.authority_store_id != authority.authority_store_id
            || record.orchestration_session_id != authority.orchestration_session_id
            || record.caller_participant_id != authority.caller_participant_id
            || record.authority_revision_observed != authority.authority_revision_observed
        {
            anyhow::bail!(
                "applicable Start obligation acceptance has conflicting correlation evidence"
            );
        }
        if !matches!(
            record.work_identity,
            AcceptedWorldWorkIdentityV1::RetainedTurn { .. }
        ) {
            continue;
        }
        if exact.replace(record.clone()).is_some() {
            anyhow::bail!("multiple canonical obligation results match the inaugural Start turn");
        }
    }
    let Some(acceptance) = exact else {
        return Ok(None);
    };
    let observation = authority
        .execution_supervisor
        .inspect_observation_by_acceptance_id(&acceptance.acceptance_record_id)?
        .ok_or_else(|| anyhow::anyhow!("applicable Start obligation observation is absent"))?;
    let terminal = observation.terminal.as_ref().ok_or_else(|| {
        anyhow::anyhow!("applicable Start obligation result is pending its exact terminal cut")
    })?;
    let request = ObligationLedgerSnapshotReadRequestV1 {
        schema_version: 1,
        authority_store_id: acceptance.authority_store_id.clone(),
        orchestration_session_id: acceptance.orchestration_session_id.clone(),
        authoritative_participant_id: acceptance.caller_participant_id.clone(),
        acceptance_record_id: acceptance.acceptance_record_id.clone(),
        acceptance_record_revision: acceptance.record_revision,
        stream_id: acceptance.runtime_acceptance.stream_id.clone(),
        accepted_work_identity: acceptance.work_identity.clone(),
        host_transition_correlation: expected_correlation.clone(),
        transition_intent_id: expected_correlation.transition_intent_id.clone(),
        transition_run_id: expected_correlation.transition_run_id.clone(),
        authority_revision_observed: expected_correlation.authority_revision_observed,
        required_terminal_event_id: terminal.event_identity.event_id.clone(),
        required_terminal_event_sequence: terminal.event_identity.event_sequence,
    };
    read_obligation_ledger_snapshot(
        store,
        &authority.receipt_registry,
        &authority.execution_supervisor,
        &request,
    )
    .map(Some)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedRetainedObligationMaterialization {
    materialized_events: Vec<MaterializedObligationLedgerEventV1>,
    projected_obligations: Vec<OrchestrationObligationRecord>,
    terminal_event_id: Option<String>,
    terminal_event_sequence: Option<u64>,
    materialized_through_event_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MaterializedRetainedWorkerEvent {
    event_record: MaterializedObligationLedgerEventV1,
    projected_obligation: Option<OrchestrationObligationRecord>,
}

fn validate_retained_acceptance_and_claim(
    acceptance: &WorldWorkAcceptanceRecordV1,
    claim: &WorldWorkExecutionClaimV1,
) -> Result<()> {
    acceptance.validate()?;
    if !claim.matches_acceptance(acceptance) {
        anyhow::bail!("C1 retained acceptance and claim scope drifted");
    }
    if !matches!(
        acceptance.work_identity,
        AcceptedWorldWorkIdentityV1::RetainedTurn { .. }
    ) {
        anyhow::bail!("C1 retained obligation materializer requires RetainedTurn acceptance");
    }
    Ok(())
}

fn validate_state_matches_acceptance(
    state: &ObligationLedgerSessionStateV1,
    acceptance: &WorldWorkAcceptanceRecordV1,
) -> Result<()> {
    if state.authority_store_id != acceptance.authority_store_id
        || state.orchestration_session_id != acceptance.orchestration_session_id
        || state.authoritative_participant_id != acceptance.caller_participant_id
        || state.acceptance_record_id != acceptance.acceptance_record_id
        || state.acceptance_record_revision != acceptance.record_revision
        || state.accepted_work_identity != acceptance.work_identity
        || state.stream_id != acceptance.runtime_acceptance.stream_id
        || state.authority_revision_observed != acceptance.authority_revision_observed
        || state.host_transition_correlation != acceptance.host_transition_correlation
    {
        anyhow::bail!("C1 obligation ledger state drifted from accepted retained scope");
    }
    Ok(())
}

#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "C1 defines the closed snapshot read contract before A1.2b adopts it"
    )
)]
fn validate_snapshot_request_against_acceptance(
    request: &ObligationLedgerSnapshotReadRequestV1,
    acceptance: &WorldWorkAcceptanceRecordV1,
    claim: &WorldWorkExecutionClaimV1,
) -> Result<()> {
    if request.authority_store_id != acceptance.authority_store_id
        || request.orchestration_session_id != acceptance.orchestration_session_id
        || request.authoritative_participant_id != acceptance.caller_participant_id
        || request.acceptance_record_id != acceptance.acceptance_record_id
        || request.acceptance_record_revision != acceptance.record_revision
        || request.stream_id != acceptance.runtime_acceptance.stream_id
        || request.accepted_work_identity != acceptance.work_identity
        || request.host_transition_correlation
            != acceptance
                .host_transition_correlation
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!("C1 snapshot requires host_transition_correlation")
                })?
        || request.authority_revision_observed != acceptance.authority_revision_observed
        || request.transition_intent_id != request.host_transition_correlation.transition_intent_id
        || request.transition_run_id != request.host_transition_correlation.transition_run_id
        || claim.authority_store_id != request.authority_store_id
    {
        anyhow::bail!("C1 snapshot request scope drifted from accepted retained scope");
    }
    Ok(())
}

fn materialize_retained_worker_event(
    acceptance: &WorldWorkAcceptanceRecordV1,
    claim: &WorldWorkExecutionClaimV1,
    frame_identity: &substrate_common::agent_events::RuntimeFrameIdentityV1,
    event_identity: &substrate_common::agent_events::RuntimeEventIdentityV1,
    event: &AgentEvent,
) -> Result<MaterializedRetainedWorkerEvent> {
    event
        .validate_identity_contract()
        .map_err(anyhow::Error::msg)?;
    let worker_event = event.worker_event.as_ref().ok_or_else(|| {
        anyhow::anyhow!("C1 retained materializer requires typed worker_event on every Event")
    })?;
    validate_worker_event_against_acceptance(
        acceptance,
        claim,
        frame_identity,
        event_identity,
        worker_event,
    )?;
    let source_journal_event = SupervisorJournalEventRefV1 {
        schema_version: 1,
        journal_entry_id: format!(
            "wwje_{}",
            canonical_sha256(&serde_json::json!({
                "schema_version": 1,
                "acceptance_record_id": acceptance.acceptance_record_id,
                "frame_sequence": frame_identity.frame_sequence,
                "event_id": event_identity.event_id,
                "event_sequence": event_identity.event_sequence,
            }))?
        ),
        acceptance_record_id: acceptance.acceptance_record_id.clone(),
        acceptance_record_revision: acceptance.record_revision,
        accepted_work_identity: acceptance.work_identity.clone(),
        stream_id: acceptance.runtime_acceptance.stream_id.clone(),
        frame_sequence: frame_identity.frame_sequence,
        event_id: event_identity.event_id.clone(),
        event_sequence: event_identity.event_sequence,
        transport_event_commitment: worker_event_transport_commitment(worker_event)?,
    };
    let (projected_obligation, obligation_id) =
        projected_obligation_from_worker_event(acceptance, worker_event, &source_journal_event)?;
    Ok(MaterializedRetainedWorkerEvent {
        event_record: MaterializedObligationLedgerEventV1 {
            schema_version: 1,
            authority_store_id: acceptance.authority_store_id.clone(),
            orchestration_session_id: acceptance.orchestration_session_id.clone(),
            authoritative_participant_id: acceptance.caller_participant_id.clone(),
            acceptance_record_id: acceptance.acceptance_record_id.clone(),
            acceptance_record_revision: acceptance.record_revision,
            accepted_work_identity: acceptance.work_identity.clone(),
            stream_id: acceptance.runtime_acceptance.stream_id.clone(),
            source_journal_event,
            event_class: worker_event.event_class,
            attention_required: worker_event.attention_required,
            emitted_at: worker_event.emitted_at,
            obligation_id,
        },
        projected_obligation,
    })
}

fn validate_worker_event_against_acceptance(
    acceptance: &WorldWorkAcceptanceRecordV1,
    claim: &WorldWorkExecutionClaimV1,
    frame_identity: &substrate_common::agent_events::RuntimeFrameIdentityV1,
    event_identity: &substrate_common::agent_events::RuntimeEventIdentityV1,
    worker_event: &WorldWorkerEventV1,
) -> Result<()> {
    let AcceptedWorldWorkIdentityV1::RetainedTurn {
        active_run_id,
        message_id,
        target_participant_id,
    } = &acceptance.work_identity
    else {
        anyhow::bail!("C1 retained materializer requires a retained accepted work identity");
    };
    if worker_event.acceptance_record_id != acceptance.acceptance_record_id
        || worker_event.stream_id != acceptance.runtime_acceptance.stream_id
        || worker_event.frame_sequence != frame_identity.frame_sequence
        || worker_event.event_id != event_identity.event_id
        || worker_event.event_sequence != event_identity.event_sequence
        || worker_event.request_id != acceptance.request_id
        || worker_event.active_run_id != *active_run_id
        || worker_event.causation_request_id != acceptance.request_id
        || worker_event.causation_message_id != *message_id
        || worker_event.orchestration_session_id != acceptance.orchestration_session_id
        || worker_event.source_participant_id != *target_participant_id
        || worker_event.target_participant_id != acceptance.caller_participant_id
        || worker_event.source_backend_id != acceptance.target_backend_id
        || worker_event.target_backend_id != acceptance.caller_backend_id
        || worker_event.world_id != acceptance.world_id
        || worker_event.world_generation != acceptance.world_generation
        || worker_event.host_transition_correlation != acceptance.host_transition_correlation
        || claim.stream_id != worker_event.stream_id
    {
        anyhow::bail!("C1 retained materializer detected worker_event scope drift");
    }
    Ok(())
}

fn projected_obligation_from_worker_event(
    acceptance: &WorldWorkAcceptanceRecordV1,
    worker_event: &WorldWorkerEventV1,
    source_journal_event: &SupervisorJournalEventRefV1,
) -> Result<(Option<OrchestrationObligationRecord>, Option<String>)> {
    let maybe_kind = match (worker_event.event_class, worker_event.attention_required) {
        (WorldWorkerEventClassV1::AttentionRequired, true) => Some((
            OrchestrationObligationKind::FollowUpRequired,
            format!(
                "retained worker {} requires host attention during continue_world_worker",
                worker_event.source_participant_id
            ),
        )),
        (WorldWorkerEventClassV1::FollowUpQuestion, true) => Some((
            OrchestrationObligationKind::FollowUpRequired,
            format!(
                "retained worker {} requested host follow-up during continue_world_worker",
                worker_event.source_participant_id
            ),
        )),
        (WorldWorkerEventClassV1::ApprovalRequest, true) => Some((
            OrchestrationObligationKind::ApprovalRequired,
            format!(
                "retained worker {} requested approval during continue_world_worker",
                worker_event.source_participant_id
            ),
        )),
        (WorldWorkerEventClassV1::Blocked, true) => Some((
            OrchestrationObligationKind::Blocked,
            format!(
                "retained worker {} reported blocked state during continue_world_worker",
                worker_event.source_participant_id
            ),
        )),
        (WorldWorkerEventClassV1::ForkRequest, true) => Some((
            OrchestrationObligationKind::ForkRequest,
            format!(
                "retained worker {} requested a child worker during continue_world_worker",
                worker_event.source_participant_id
            ),
        )),
        (
            WorldWorkerEventClassV1::Reply
            | WorldWorkerEventClassV1::ProgressUpdate
            | WorldWorkerEventClassV1::ControlAck
            | WorldWorkerEventClassV1::ForkRecommendation
            | WorldWorkerEventClassV1::Result
            | WorldWorkerEventClassV1::Failure,
            false,
        ) => None,
        _ => {
            anyhow::bail!(
                "unsupported C1 worker event class/attention combination {:?}/{}",
                worker_event.event_class,
                worker_event.attention_required
            );
        }
    };

    let Some((kind, summary)) = maybe_kind else {
        return Ok((None, None));
    };

    let obligation_id = format!(
        "obl_c1_{}",
        canonical_sha256(&serde_json::json!({
            "schema_version": 1,
            "authority_store_id": acceptance.authority_store_id,
            "orchestration_session_id": acceptance.orchestration_session_id,
            "authoritative_participant_id": acceptance.caller_participant_id,
            "acceptance_record_id": acceptance.acceptance_record_id,
            "acceptance_record_revision": acceptance.record_revision,
            "stream_id": acceptance.runtime_acceptance.stream_id,
            "event_id": worker_event.event_id,
            "event_sequence": worker_event.event_sequence,
            "event_class": c1_event_class_label(worker_event.event_class),
        }))?
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "event_class": c1_event_class_label(worker_event.event_class),
        "request_id": worker_event.request_id,
        "active_run_id": worker_event.active_run_id,
        "target_participant_id": worker_event.target_participant_id,
        "source_backend_id": worker_event.source_backend_id,
        "thread_id": worker_event.thread_id,
        "stream_id": worker_event.stream_id,
        "frame_sequence": worker_event.frame_sequence,
        "event_id": worker_event.event_id,
        "event_sequence": worker_event.event_sequence,
        "host_transition_correlation": worker_event.host_transition_correlation,
        "payload": worker_event.payload,
    });
    let payload_commitment = obligation_payload_commitment(&payload)?;
    let canonical_record_commitment = obligation_snapshot_record_commitment(
        &acceptance.authority_store_id,
        &acceptance.orchestration_session_id,
        &acceptance.caller_participant_id,
        source_journal_event,
        &obligation_id,
        1,
    )?;
    let mut obligation = OrchestrationObligationRecord::new(
        acceptance.orchestration_session_id.clone(),
        obligation_id.clone(),
        kind,
        summary,
    );
    obligation.created_at = worker_event.emitted_at;
    obligation.updated_at = worker_event.emitted_at;
    obligation.attention_required = true;
    obligation.severity = match kind {
        OrchestrationObligationKind::Blocked => OrchestrationObligationSeverity::Error,
        OrchestrationObligationKind::ApprovalRequired
        | OrchestrationObligationKind::FollowUpRequired
        | OrchestrationObligationKind::ForkRequest => OrchestrationObligationSeverity::Warning,
        _ => OrchestrationObligationSeverity::Info,
    };
    obligation.attach_state = match kind {
        OrchestrationObligationKind::FollowUpRequired
        | OrchestrationObligationKind::ApprovalRequired
        | OrchestrationObligationKind::Blocked
        | OrchestrationObligationKind::ForkRequest => OrchestrationObligationAttachState::Eligible,
        _ => OrchestrationObligationAttachState::NotEligible,
    };
    obligation.authority_store_id = Some(acceptance.authority_store_id.clone());
    obligation.authoritative_participant_id = Some(acceptance.caller_participant_id.clone());
    obligation.obligation_revision = Some(1);
    obligation.source_journal_event = Some(source_journal_event.clone());
    obligation.payload_commitment = Some(payload_commitment);
    obligation.canonical_record_commitment = Some(canonical_record_commitment);
    obligation.source_participant_id = Some(worker_event.source_participant_id.clone());
    obligation.ingress_source_kind = Some("world_work_execution_supervisor".to_string());
    obligation.ingress_source_id = Some(source_journal_event.journal_entry_id.clone());
    obligation.ingress_received_at = Some(worker_event.emitted_at);
    obligation.causation_event_id = Some(worker_event.event_id.clone());
    obligation.causation_message_id = Some(worker_event.causation_message_id.clone());
    obligation.causation_request_id = Some(worker_event.causation_request_id.clone());
    obligation.target_backend_id = Some(worker_event.target_backend_id.clone());
    obligation.world_id = Some(worker_event.world_id.clone());
    obligation.world_generation = Some(worker_event.world_generation);
    obligation.payload = Some(payload);
    Ok((Some(obligation), Some(obligation_id)))
}

#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "C1 defines the closed snapshot read contract before A1.2b adopts it"
    )
)]
fn expected_retained_obligation_materialization_from_observation(
    acceptance: &WorldWorkAcceptanceRecordV1,
    observation: &WorldWorkExecutionObservationV1,
) -> Result<ExpectedRetainedObligationMaterialization> {
    let mut materialized_events = Vec::new();
    let mut projected_obligations = Vec::new();
    let mut materialized_through_event_sequence: u64 = 0;
    let mut seen_terminal = None::<(String, u64)>;

    for entry in &observation.journal {
        let frame = entry.decode_for_c1(&observation.claim)?;
        match frame {
            ExecuteStreamFrame::Event {
                frame_identity,
                event,
            } => {
                let event_identity = entry.event_identity.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "C1 expected durable event identity while materializing snapshot"
                    )
                })?;
                if seen_terminal.is_some() {
                    anyhow::bail!(
                        "C1 obligation materializer observed an Event after the exact terminal cut"
                    );
                }
                if event_identity.event_sequence
                    != materialized_through_event_sequence.saturating_add(1)
                {
                    anyhow::bail!("C1 obligation materializer observed an event gap or reorder");
                }
                let materialized = materialize_retained_worker_event(
                    acceptance,
                    &observation.claim,
                    &frame_identity,
                    event_identity,
                    &event,
                )?;
                if let Some(obligation) = materialized.projected_obligation {
                    projected_obligations.push(obligation);
                }
                materialized_events.push(materialized.event_record);
                materialized_through_event_sequence = event_identity.event_sequence;
            }
            ExecuteStreamFrame::Exit { event_identity, .. } => {
                if seen_terminal.is_some() {
                    anyhow::bail!("C1 obligation materializer observed multiple terminal frames");
                }
                if event_identity.event_sequence
                    != materialized_through_event_sequence.saturating_add(1)
                {
                    anyhow::bail!("C1 obligation materializer observed a terminal event gap");
                }
                materialized_through_event_sequence = event_identity.event_sequence;
                seen_terminal = Some((
                    event_identity.event_id.clone(),
                    event_identity.event_sequence,
                ));
            }
            ExecuteStreamFrame::Start { .. }
            | ExecuteStreamFrame::Stdout { .. }
            | ExecuteStreamFrame::Stderr { .. }
            | ExecuteStreamFrame::Error { .. } => {}
        }
    }

    if observation.terminal.as_ref().map(|terminal| {
        (
            terminal.event_identity.event_id.clone(),
            terminal.event_identity.event_sequence,
        )
    }) != seen_terminal
    {
        anyhow::bail!("C1 supervisor terminal cut drifted from the durable journal replay");
    }

    Ok(ExpectedRetainedObligationMaterialization {
        materialized_events,
        projected_obligations,
        terminal_event_id: observation
            .terminal
            .as_ref()
            .map(|terminal| terminal.event_identity.event_id.clone()),
        terminal_event_sequence: observation
            .terminal
            .as_ref()
            .map(|terminal| terminal.event_identity.event_sequence),
        materialized_through_event_sequence,
    })
}

fn materialized_obligations_for_exact_event_cut(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    acceptance_record_id: &str,
    expected_projected_obligations: &[OrchestrationObligationRecord],
) -> Result<Vec<OrchestrationObligationRecord>> {
    let stored_obligations = store.list_materialized_obligation_ledger_obligations(
        orchestration_session_id,
        acceptance_record_id,
    )?;
    let mut stored_by_id = std::collections::BTreeMap::new();
    for obligation in stored_obligations {
        let Some(expected_obligation) = expected_projected_obligations
            .iter()
            .find(|expected| expected.obligation_id == obligation.obligation_id)
        else {
            anyhow::bail!(
                "unexpected C1 materialized obligation {} outside the exact retained event cut",
                obligation.obligation_id
            );
        };
        if !obligation.matches_c1_materialization_projection(expected_obligation) {
            anyhow::bail!(
                "conflicting duplicate C1 obligation {}",
                obligation.obligation_id
            );
        }
        stored_by_id.insert(obligation.obligation_id.clone(), obligation);
    }

    let mut exact_cut = Vec::new();
    for expected_obligation in expected_projected_obligations {
        exact_cut.push(
            stored_by_id
                .remove(&expected_obligation.obligation_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "missing C1 materialized obligation {} required by the exact retained event cut",
                        expected_obligation.obligation_id
                    )
                })?,
        );
    }

    if let Some(unexpected) = stored_by_id.into_keys().next() {
        anyhow::bail!(
            "unexpected C1 materialized obligation {} outside the exact retained event cut",
            unexpected
        );
    }
    Ok(exact_cut)
}

fn unresolved_attention_entries_for_materialized_obligations(
    obligations: &[OrchestrationObligationRecord],
) -> Result<Vec<UnresolvedAttentionObligationSnapshotEntryV1>> {
    let mut unresolved = Vec::new();
    for obligation in obligations {
        if !obligation.is_pending()
            || !obligation.attention_required
            || !obligation.has_c1_materialization_identity()
        {
            continue;
        }
        unresolved.push(UnresolvedAttentionObligationSnapshotEntryV1 {
            obligation_id: obligation.obligation_id.clone(),
            obligation_revision: obligation.obligation_revision.ok_or_else(|| {
                anyhow::anyhow!("C1 pending obligation omitted obligation_revision")
            })?,
            canonical_record_commitment: obligation
                .canonical_record_commitment
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!("C1 pending obligation omitted canonical_record_commitment")
                })?,
        });
    }
    Ok(unresolved)
}

fn canonical_commitment_sort_key(
    commitment: &AuthorityObjectCommitmentV1,
) -> std::borrow::Cow<'_, str> {
    std::borrow::Cow::Owned(
        serde_json::to_string(commitment)
            .expect("authority commitments serialize for deterministic C1 sorting"),
    )
}

fn sort_unresolved_attention_entries(
    unresolved: &mut [UnresolvedAttentionObligationSnapshotEntryV1],
) {
    unresolved.sort_by(|left, right| {
        canonical_commitment_sort_key(&left.canonical_record_commitment)
            .cmp(&canonical_commitment_sort_key(
                &right.canonical_record_commitment,
            ))
            .then(
                left.obligation_id
                    .as_bytes()
                    .cmp(right.obligation_id.as_bytes()),
            )
    });
}

fn c1_event_class_label(event_class: WorldWorkerEventClassV1) -> &'static str {
    match event_class {
        WorldWorkerEventClassV1::Reply => "reply",
        WorldWorkerEventClassV1::ProgressUpdate => "progress_update",
        WorldWorkerEventClassV1::ControlAck => "control_ack",
        WorldWorkerEventClassV1::FollowUpQuestion => "follow_up_question",
        WorldWorkerEventClassV1::ApprovalRequest => "approval_request",
        WorldWorkerEventClassV1::Blocked => "blocked",
        WorldWorkerEventClassV1::AttentionRequired => "attention_required",
        WorldWorkerEventClassV1::ForkRequest => "fork_request",
        WorldWorkerEventClassV1::ForkRecommendation => "fork_recommendation",
        WorldWorkerEventClassV1::Result => "result",
        WorldWorkerEventClassV1::Failure => "failure",
    }
}

fn parse_c1_event_class_label(value: &str) -> Result<WorldWorkerEventClassV1> {
    match value {
        "reply" => Ok(WorldWorkerEventClassV1::Reply),
        "progress_update" => Ok(WorldWorkerEventClassV1::ProgressUpdate),
        "control_ack" => Ok(WorldWorkerEventClassV1::ControlAck),
        "follow_up_question" => Ok(WorldWorkerEventClassV1::FollowUpQuestion),
        "approval_request" => Ok(WorldWorkerEventClassV1::ApprovalRequest),
        "blocked" => Ok(WorldWorkerEventClassV1::Blocked),
        "attention_required" => Ok(WorldWorkerEventClassV1::AttentionRequired),
        "fork_request" => Ok(WorldWorkerEventClassV1::ForkRequest),
        "fork_recommendation" => Ok(WorldWorkerEventClassV1::ForkRecommendation),
        "result" => Ok(WorldWorkerEventClassV1::Result),
        "failure" => Ok(WorldWorkerEventClassV1::Failure),
        _ => anyhow::bail!("invalid C1 worker event class label"),
    }
}

fn c1_obligation_snapshot_record_state_label(
    state: ObligationSnapshotRecordStateV1,
) -> &'static str {
    match state {
        ObligationSnapshotRecordStateV1::UnresolvedAttention => "unresolved_attention",
    }
}

fn parse_c1_obligation_snapshot_record_state_label(
    value: &str,
) -> Result<ObligationSnapshotRecordStateV1> {
    match value {
        "unresolved_attention" => Ok(ObligationSnapshotRecordStateV1::UnresolvedAttention),
        _ => anyhow::bail!("invalid C1 obligation snapshot record state label"),
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    fn pending_obligation_rejects_terminal_review_state() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::ApprovalRequired,
            "Need host approval",
        );
        obligation.review_state = OrchestrationObligationReviewState::Dismissed;

        let err = obligation
            .validate()
            .expect_err("pending obligations must not carry terminal review state");
        assert!(err
            .to_string()
            .contains("pending orchestration obligations cannot advertise terminal review_state"));
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
    fn host_targeting_fields_reject_empty_ids() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::RuntimeAlert,
            "World reported a runtime alert",
        );
        obligation.origin_host_id = Some("   ".to_string());

        let err = obligation
            .validate()
            .expect_err("blank origin_host_id must fail validation");
        assert!(err.to_string().contains("empty origin_host_id"));

        obligation.origin_host_id = Some("host-origin".to_string());
        obligation.target_host_id = Some("\t".to_string());

        let err = obligation
            .validate()
            .expect_err("blank target_host_id must fail validation");
        assert!(err.to_string().contains("empty target_host_id"));
    }

    #[test]
    fn ingress_and_causation_identity_fields_reject_empty_ids() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::RuntimeAlert,
            "World reported a runtime alert",
        );
        obligation.ingress_source_kind = Some(" ".to_string());

        let err = obligation
            .validate()
            .expect_err("blank ingress_source_kind must fail validation");
        assert!(err.to_string().contains("empty ingress_source_kind"));

        obligation.ingress_source_kind = Some("local_runtime".to_string());
        obligation.ingress_source_id = Some("\t".to_string());
        let err = obligation
            .validate()
            .expect_err("blank ingress_source_id must fail validation");
        assert!(err.to_string().contains("empty ingress_source_id"));

        obligation.ingress_source_id = Some("run-123".to_string());
        obligation.causation_event_id = Some(" ".to_string());
        let err = obligation
            .validate()
            .expect_err("blank causation_event_id must fail validation");
        assert!(err.to_string().contains("empty causation_event_id"));

        obligation.causation_event_id = Some("event-123".to_string());
        obligation.causation_message_id = Some(" ".to_string());
        let err = obligation
            .validate()
            .expect_err("blank causation_message_id must fail validation");
        assert!(err.to_string().contains("empty causation_message_id"));

        obligation.causation_message_id = Some("message-123".to_string());
        obligation.causation_request_id = Some(" ".to_string());
        let err = obligation
            .validate()
            .expect_err("blank causation_request_id must fail validation");
        assert!(err.to_string().contains("empty causation_request_id"));
    }

    #[test]
    fn payload_thread_id_is_not_promoted_into_canonical_causation_fields() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::FollowUpRequired,
            "Need host follow-up",
        );
        obligation.payload = Some(json!({
            "thread_id": "thread-123",
        }));

        obligation
            .validate()
            .expect("payload thread_id remains payload-only metadata");
        assert_eq!(obligation.causation_message_id, None);
        assert_eq!(obligation.causation_event_id, None);
        assert_eq!(obligation.causation_request_id, None);
    }

    #[test]
    fn classify_local_host_targeting_freezes_packet_one_wrong_host_boundary() {
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::FollowUpRequired,
            "Need host follow-up",
        );

        assert_eq!(
            obligation
                .classify_local_host_targeting("host-local")
                .expect("untargeted obligations preserve local-only semantics"),
            LocalHostObligationTargetingDisposition::Untargeted,
        );

        obligation.target_host_id = Some("host-local".to_string());
        assert_eq!(
            obligation
                .classify_local_host_targeting("host-local")
                .expect("same-host targeting stays locally eligible"),
            LocalHostObligationTargetingDisposition::TargetedToLocalHost,
        );

        obligation.target_host_id = Some("host-remote".to_string());
        assert_eq!(
            obligation
                .classify_local_host_targeting("host-local")
                .expect("foreign targeting is classified fail-closed"),
            LocalHostObligationTargetingDisposition::WrongHost {
                target_host_id: "host-remote".to_string(),
            },
        );
    }

    #[test]
    fn classify_local_host_targeting_rejects_blank_local_host_id() {
        let obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::FollowUpRequired,
            "Need host follow-up",
        );

        let err = obligation
            .classify_local_host_targeting("   ")
            .expect_err("blank local_host_id must fail classification");
        assert!(err.to_string().contains("empty local_host_id"));
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

    #[test]
    fn attach_state_semantics_remain_mapped_to_forward_design_states_without_rename() {
        let cases = [
            (
                OrchestrationObligationAttachState::NotEligible,
                "not_requested",
            ),
            (OrchestrationObligationAttachState::Eligible, "queued"),
            (OrchestrationObligationAttachState::Claimed, "claimed"),
            (OrchestrationObligationAttachState::Satisfied, "completed"),
            (
                OrchestrationObligationAttachState::FailedClosed,
                "dead_letter",
            ),
            (OrchestrationObligationAttachState::Superseded, "cancelled"),
        ];

        for (attach_state, expected_forward_state) in cases {
            assert_eq!(
                attach_state.forward_design_state(),
                expected_forward_state,
                "{attach_state:?} should keep its forward-design semantic mapping",
            );
        }
    }

    #[test]
    fn packet_two_obligation_kinds_keep_attach_defaults_explicit() {
        let cases = [
            (OrchestrationObligationKind::FollowUpRequired, true),
            (OrchestrationObligationKind::ApprovalRequired, true),
            (OrchestrationObligationKind::Blocked, true),
            (OrchestrationObligationKind::ForkRequest, true),
            (OrchestrationObligationKind::ForkRecommendation, false),
        ];

        for (kind, expected_auto_attach_support) in cases {
            let mut obligation = OrchestrationObligationRecord::new(
                "sess_001",
                format!("obl_{kind:?}").to_lowercase(),
                kind,
                format!("summary for {kind:?}"),
            );
            obligation.attention_required = true;
            obligation.attach_state = OrchestrationObligationAttachState::Eligible;

            assert_eq!(
                kind.supports_router_auto_attach(),
                expected_auto_attach_support,
                "{kind:?} must keep its packet-two router auto-attach default",
            );
            assert_eq!(
                obligation.is_auto_attach_eligible(),
                expected_auto_attach_support,
                "{kind:?} must keep its packet-two obligation auto-attach projection",
            );
        }
    }

    #[test]
    fn satisfied_attach_state_does_not_resolve_pending_obligation() {
        let settled_at = Utc::now();
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::ApprovalRequired,
            "Need host approval",
        );
        obligation.attention_required = true;
        obligation.attach_state = OrchestrationObligationAttachState::Claimed;
        obligation.mark_attach_claimed("router::local", settled_at);

        obligation.mark_attach_satisfied("session_attach_restored_by_test", settled_at);

        assert_eq!(obligation.state, OrchestrationObligationState::Pending);
        assert_eq!(
            obligation.review_state,
            OrchestrationObligationReviewState::Unread
        );
        assert!(obligation.resolved_at.is_none());
        assert_eq!(
            obligation.attach_state,
            OrchestrationObligationAttachState::Satisfied
        );
        assert_eq!(
            obligation.attach_completion_reason.as_deref(),
            Some("session_attach_restored_by_test")
        );
    }

    #[test]
    fn approval_response_closeout_marks_resolved_review_state_for_approve() {
        let resolved_at = Utc::now();
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::ApprovalRequired,
            "Need host approval",
        );
        obligation.attention_required = true;

        obligation.mark_approval_response_closed(
            ApprovalObligationCloseoutDisposition::Resolve,
            Some("approved by host".to_string()),
            resolved_at,
        );

        assert_eq!(obligation.state, OrchestrationObligationState::Resolved);
        assert_eq!(
            obligation.review_state,
            OrchestrationObligationReviewState::Resolved
        );
        assert!(!obligation.attention_required);
        assert_eq!(
            obligation.resolution_note.as_deref(),
            Some("approved by host")
        );
        assert_eq!(obligation.resolved_at, Some(resolved_at));
        assert_eq!(obligation.updated_at, resolved_at);
    }

    #[test]
    fn approval_response_closeout_marks_dismissed_review_state_for_deny() {
        let resolved_at = Utc::now();
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::ApprovalRequired,
            "Need host approval",
        );
        obligation.attention_required = true;

        obligation.mark_approval_response_closed(
            ApprovalObligationCloseoutDisposition::Dismiss,
            Some("denied by host".to_string()),
            resolved_at,
        );

        assert_eq!(obligation.state, OrchestrationObligationState::Resolved);
        assert_eq!(
            obligation.review_state,
            OrchestrationObligationReviewState::Dismissed
        );
        assert!(!obligation.attention_required);
        assert_eq!(
            obligation.resolution_note.as_deref(),
            Some("denied by host")
        );
        assert_eq!(obligation.resolved_at, Some(resolved_at));
        assert_eq!(obligation.updated_at, resolved_at);
    }

    #[test]
    fn clarification_response_closeout_marks_follow_up_obligation_resolved() {
        let resolved_at = Utc::now();
        let mut obligation = OrchestrationObligationRecord::new(
            "sess_001",
            "obl_001",
            OrchestrationObligationKind::FollowUpRequired,
            "Need host follow-up",
        );
        obligation.attention_required = true;

        obligation.mark_clarification_response_closed(
            Some("clarification delivered by host".to_string()),
            resolved_at,
        );

        assert_eq!(obligation.state, OrchestrationObligationState::Resolved);
        assert_eq!(
            obligation.review_state,
            OrchestrationObligationReviewState::Resolved
        );
        assert!(!obligation.attention_required);
        assert_eq!(
            obligation.resolution_note.as_deref(),
            Some("clarification delivered by host")
        );
        assert_eq!(obligation.resolved_at, Some(resolved_at));
        assert_eq!(obligation.updated_at, resolved_at);
    }

    #[test]
    fn unresolved_attention_entries_sort_by_canonical_commitment_before_obligation_id() {
        let mut unresolved = vec![
            UnresolvedAttentionObligationSnapshotEntryV1 {
                obligation_id: "obl_a".to_string(),
                obligation_revision: 1,
                canonical_record_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "f".repeat(64),
                },
            },
            UnresolvedAttentionObligationSnapshotEntryV1 {
                obligation_id: "obl_z".to_string(),
                obligation_revision: 1,
                canonical_record_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "0".repeat(64),
                },
            },
        ];

        sort_unresolved_attention_entries(&mut unresolved);

        assert_eq!(unresolved[0].obligation_id, "obl_z");
        assert_eq!(unresolved[1].obligation_id, "obl_a");
    }
}
