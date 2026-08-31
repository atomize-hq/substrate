use std::fmt;

use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use substrate_common::HostTransitionWorkCorrelationV1;

use super::super::obligation_ledger::{
    ObligationMaterializationCutV1, SupervisorJournalEventRefV1,
};
use super::super::state_store::AcceptedWorldWorkIdentityV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TimestampV1(String);

impl TimestampV1 {
    pub(crate) fn parse(value: impl Into<String>) -> Result<Self, TimestampV1Error> {
        let value = value.into();
        let bytes = value.as_bytes();
        let punctuation_is_exact = bytes.len() == 30
            && bytes.get(4) == Some(&b'-')
            && bytes.get(7) == Some(&b'-')
            && bytes.get(10) == Some(&b'T')
            && bytes.get(13) == Some(&b':')
            && bytes.get(16) == Some(&b':')
            && bytes.get(19) == Some(&b'.')
            && bytes.get(29) == Some(&b'Z');
        let digits_are_exact = bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19 | 29) || byte.is_ascii_digit()
        });
        let seconds_are_not_leap = bytes
            .get(17..19)
            .and_then(|seconds| std::str::from_utf8(seconds).ok())
            .and_then(|seconds| seconds.parse::<u8>().ok())
            .is_some_and(|seconds| seconds <= 59);
        if !punctuation_is_exact
            || !digits_are_exact
            || !seconds_are_not_leap
            || NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S%.9fZ").is_err()
        {
            return Err(TimestampV1Error);
        }
        Ok(Self(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for TimestampV1 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TimestampV1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TimestampV1Error;

impl fmt::Display for TimestampV1Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "TimestampV1 requires a valid UTC RFC3339 timestamp with nine fractional digits and uppercase Z",
        )
    }
}

impl std::error::Error for TimestampV1Error {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum DirectoryPhysicalIdentityV1 {
    Linux {
        device_id: u64,
        inode: u64,
    },
    MacOs {
        volume_uuid: String,
        file_id: u64,
        case_sensitive: bool,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalDirectoryV1 {
    pub(crate) physical_path: String,
    pub(crate) physical_identity: DirectoryPhysicalIdentityV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkspaceBindingV1 {
    pub(crate) workspace_root: CanonicalDirectoryV1,
    pub(crate) authority_store_root: CanonicalDirectoryV1,
    pub(crate) authority_store_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldBindingV1 {
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum AuthorityObjectKindV1 {
    AgentDescriptor,
    RetainedWorker,
    ResumeHandle,
    Policy,
    HostAttachContract,
    TransitionTransportPayload,
    TransitionInput,
    LeaseToken,
    ApplicationResult,
    InputAcceptance,
    StartupOwnershipResult,
    ObligationSnapshot,
    PostTurnProtocolEvent,
    PostTurnCompletion,
    TerminalHandoff,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum AuthorityObjectCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthorityObjectRefV1 {
    pub(crate) ref_id: String,
    pub(crate) object_kind: AuthorityObjectKindV1,
    pub(crate) schema_version: u32,
    pub(crate) commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostSessionPostureV1 {
    ActiveAttached,
    ParkedResumable,
    DetachedReconciled,
    AwaitingAttention,
    Terminal,
    StaleRecoverable,
    Invalid,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum DurableSessionAuthorityOriginV1 {
    StartIntent {
        intent_id: String,
        issuer_request_id: String,
        payload_commitment: AuthorityObjectCommitmentV1,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionModeV1 {
    Start,
    Attach,
    ResumeOneTurn,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionCallerKindV1 {
    PublicCli,
    Repl,
    RouterAutoAttach,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionCallerV1 {
    pub(crate) kind: HostSessionTransitionCallerKindV1,
    pub(crate) caller_participant_id: Option<String>,
    pub(crate) auto_attach_obligation_id: Option<String>,
    pub(crate) auto_attach_claim_owner: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionAuthorityPreconditionV1 {
    ExpectedAbsent,
    ExpectedRevision {
        authority_revision: u64,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        active_authoritative_participant_id: String,
        authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
        lifecycle_posture: HostSessionPostureV1,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostPostTurnDispositionV1 {
    ReconcileToAttentionParkOrTerminal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionTerminalRejectionV1 {
    InvalidCommittedModePrecondition,
    AuthorityPreconditionNoLongerHolds,
    StaleAuthorityRevision,
    AuthorityRecordCommitmentMismatch,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum AgentExecutionScopeV1 {
    Host,
    World,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum RuntimeBackendKindV1 {
    Codex,
    ClaudeCode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentDescriptorV1 {
    pub(crate) schema_version: u32,
    pub(crate) agent_id: String,
    pub(crate) backend_id: String,
    pub(crate) backend_kind: RuntimeBackendKindV1,
    pub(crate) protocol: String,
    pub(crate) execution_scope: AgentExecutionScopeV1,
    pub(crate) binary_path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostAttachCapabilitiesV1 {
    pub(crate) session_resume: bool,
    pub(crate) session_fork: bool,
    pub(crate) session_stop: bool,
    pub(crate) status_snapshot: bool,
    pub(crate) event_stream: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostAttachExecutionClientStartV1 {
    StartNow,
    Defer,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostAttachModePreferenceV1 {
    ContinuityRequired,
    ContinuityPreferred,
    FreshAllowed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostAttachLaunchKnobsV1 {
    pub(crate) requested_execution_scope: AgentExecutionScopeV1,
    pub(crate) host_execution_client_start: HostAttachExecutionClientStartV1,
    pub(crate) attach_mode_preference: HostAttachModePreferenceV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostAttachContractV1 {
    pub(crate) schema_version: u32,
    pub(crate) backend_id: String,
    pub(crate) execution_scope: AgentExecutionScopeV1,
    pub(crate) protocol: String,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) capabilities: HostAttachCapabilitiesV1,
    pub(crate) attach_launch_knobs: HostAttachLaunchKnobsV1,
    pub(crate) policy_ref: AuthorityObjectRefV1,
    pub(crate) continuity_resume_handle_ref: Option<AuthorityObjectRefV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DurableSessionAuthorityHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) authority_revision: u64,
    pub(crate) origin: DurableSessionAuthorityOriginV1,
    pub(crate) authoritative_participant_lineage: Vec<String>,
    pub(crate) active_authoritative_participant_id: Option<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) host_attach_contract_ref: Option<AuthorityObjectRefV1>,
    pub(crate) retained_worker_refs: Vec<AuthorityObjectRefV1>,
    pub(crate) internal_resume_handle_refs: Vec<AuthorityObjectRefV1>,
    pub(crate) lifecycle_posture: HostSessionPostureV1,
    pub(crate) current_policy_ref: Option<AuthorityObjectRefV1>,
    pub(crate) current_policy_revision: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForkSuccessorAllocationRequestHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) allocation_id: String,
    pub(crate) request_id: String,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) expected_source_root_revision: u64,
    pub(crate) source_orchestration_session_id: String,
    pub(crate) source_shell_trace_session_id: String,
    pub(crate) source_authority_precondition: HostSessionAuthorityPreconditionV1,
    pub(crate) source_authoritative_participant_lineage: Vec<String>,
    pub(crate) target_orchestration_session_id: String,
    pub(crate) target_shell_trace_session_id: String,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) allocated_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthoritativeLineageHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) orchestration_session_id: String,
    pub(crate) participant_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionPayloadHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) authority_precondition: HostSessionAuthorityPreconditionV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) source_authoritative_participant_id: Option<String>,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) target_participant_lease_token_ref: AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) host_attach_contract_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: Option<AuthorityObjectRefV1>,
    pub(crate) transition_input_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_disposition: Option<HostPostTurnDispositionV1>,
    pub(crate) transport_payload_ref: AuthorityObjectRefV1,
    pub(crate) issued_at: TimestampV1,
    pub(crate) expires_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionStopPayloadHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) request_id: String,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_participant_id: String,
    pub(crate) authoritative_lineage: Vec<String>,
    pub(crate) lifecycle_posture: HostSessionPostureV1,
    pub(crate) issued_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionStopResultHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) result_id: String,
    pub(crate) intent_id: String,
    pub(crate) request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) delivery_acceptance_id: Option<String>,
    pub(crate) authority_revision_before: u64,
    pub(crate) authority_record_commitment_before: AuthorityObjectCommitmentV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) authority_record_commitment_after: AuthorityObjectCommitmentV1,
    pub(crate) resulting_posture: HostSessionPostureV1,
    pub(crate) completed_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransitionTransportPayloadObjectV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) source_authoritative_participant_id: Option<String>,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) target_participant_lease_token_ref: AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) host_attach_contract_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: Option<AuthorityObjectRefV1>,
    pub(crate) transition_input_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_disposition: Option<HostPostTurnDispositionV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentDescriptorHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) descriptor: AgentDescriptorV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostAttachContractHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) contract: HostAttachContractV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResumeHandleHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) orchestration_session_id: String,
    pub(crate) participant_id: String,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) internal_uaa_session_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum StartTurnCompletionKindV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure { reason: String },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum StartContinuationHandleStateV2 {
    Registered {
        exchange_id: String,
        exchange_sequence: u64,
        provider_event_kind: String,
        evidence_sha256: String,
        observed_at: TimestampV1,
    },
    Settled {
        registered_resume_handle_ref: Box<AuthorityObjectRefV1>,
        protocol_actor: HostPostTurnProtocolActorV1,
        event_id: String,
        event_sequence: u64,
        provider_event_kind: String,
        thread_id: String,
        turn_id: String,
        evidence_sha256: String,
        completion_kind: StartTurnCompletionKindV1,
        obligation_snapshot: Option<Box<ObligationSnapshotHashInputV1>>,
        completed_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartContinuationHandleHashInputV2 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) participant_id: String,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) internal_uaa_session_id: String,
    pub(crate) start_intent_id: String,
    pub(crate) start_issuer_request_id: String,
    pub(crate) start_payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) start_application_result_ref: AuthorityObjectRefV1,
    pub(crate) start_run_id: String,
    pub(crate) authority_revision_before: u64,
    pub(crate) authority_record_commitment_before: AuthorityObjectCommitmentV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) state: StartContinuationHandleStateV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolicyObjectHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) policy_revision: String,
    pub(crate) canonical_policy_snapshot_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerObjectHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) orchestration_session_id: String,
    pub(crate) participant_id: String,
    pub(crate) world_binding: WorldBindingV1,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: AuthorityObjectRefV1,
    pub(crate) policy_ref: AuthorityObjectRefV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum ApplicationResultPhaseV1 {
    InitialTransition {
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        post_turn_pending_run_id: Option<String>,
    },
    PostTurn {
        completion_ref: AuthorityObjectRefV1,
        authority_revision_before: u64,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApplicationResultHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) run_id: String,
    pub(crate) phase: ApplicationResultPhaseV1,
    pub(crate) applied_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputAcceptanceHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) run_id: String,
    pub(crate) input_ref: AuthorityObjectRefV1,
    pub(crate) accepting_participant_id: String,
    pub(crate) accepted_at: TimestampV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostStartupTerminalReasonV1 {
    RuntimeCreationRejected,
    StartupFailedBeforeOwnership,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostStartupOwnershipProtocolEventV1 {
    OwnershipAccepted {
        ownership_acknowledgement_id: String,
    },
    RuntimeCreationRejected {
        rejection_id: String,
    },
    StartupFailedBeforeOwnership {
        failure_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostStartupOwnershipProtocolActorV1 {
    TargetAuthoritativeParticipant {
        participant_id: String,
    },
    LaunchApplicationClaimant {
        claim_id: String,
        claimant_attempt_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostStartupOwnershipEvidenceV1 {
    pub(crate) schema_version: u32,
    pub(crate) evidence_id: String,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) claim_id: String,
    pub(crate) claimant_attempt_id: String,
    pub(crate) run_id: String,
    pub(crate) application_result_ref: AuthorityObjectRefV1,
    pub(crate) expected_authority_revision: u64,
    pub(crate) active_authoritative_participant_id: String,
    pub(crate) protocol_actor: HostStartupOwnershipProtocolActorV1,
    pub(crate) protocol_event: HostStartupOwnershipProtocolEventV1,
    pub(crate) observed_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum StartupOwnershipOutcomeV1 {
    Accepted,
    TerminalReconciled {
        reason: HostStartupTerminalReasonV1,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartupOwnershipResultHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) evidence: HostStartupOwnershipEvidenceV1,
    pub(crate) outcome: StartupOwnershipOutcomeV1,
    pub(crate) resolved_at: TimestampV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
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
    pub(crate) state: ObligationSnapshotRecordStateV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnresolvedAttentionObligationSnapshotEntryV1 {
    pub(crate) obligation_id: String,
    pub(crate) obligation_revision: u64,
    pub(crate) canonical_record_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum ObligationAttentionDispositionV1 {
    NoUnresolvedAttention,
    HasUnresolvedAttention,
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
    pub(crate) captured_at: TimestampV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum PostTurnCompletionOutcomeV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum HostPostTurnTerminalReasonV1 {
    ResumeRuntimeCreationRejected,
    TargetFailedBeforeInputAcceptance,
    TargetFailedAfterInputAcceptance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostPostTurnProtocolEventKindV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure {
        reason: HostPostTurnTerminalReasonV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostPostTurnProtocolActorV1 {
    TargetAuthoritativeParticipant {
        participant_id: String,
    },
    LaunchApplicationClaimant {
        claim_id: String,
        claimant_attempt_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PostTurnProtocolEventHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) claim_id: String,
    pub(crate) claimant_attempt_id: String,
    pub(crate) run_id: String,
    pub(crate) authority_revision_observed: u64,
    pub(crate) active_authoritative_participant_id: String,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) stream_id: String,
    pub(crate) accepted_work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) host_transition_correlation: HostTransitionWorkCorrelationV1,
    pub(crate) protocol_actor: HostPostTurnProtocolActorV1,
    pub(crate) event_id: String,
    pub(crate) event_sequence: u64,
    pub(crate) kind: HostPostTurnProtocolEventKindV1,
    pub(crate) emitted_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PostTurnCompletionHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) run_id: String,
    pub(crate) authority_revision_observed: u64,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) stream_id: String,
    pub(crate) accepted_work_identity: AcceptedWorldWorkIdentityV1,
    pub(crate) host_transition_correlation: HostTransitionWorkCorrelationV1,
    pub(crate) terminal_event_id: String,
    pub(crate) terminal_event_sequence: u64,
    pub(crate) protocol_event_ref: AuthorityObjectRefV1,
    pub(crate) outcome: PostTurnCompletionOutcomeV1,
    pub(crate) completed_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum TerminalHandoffStateV1 {
    Applied,
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
    },
    Expired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TerminalHandoffHashInputV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) run_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) terminal_state: TerminalHandoffStateV1,
    pub(crate) application_result_ref: Option<AuthorityObjectRefV1>,
    pub(crate) input_acceptance_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_completion_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_application_result_ref: Option<AuthorityObjectRefV1>,
    pub(crate) recorded_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TerminalHandoffHashInputV2 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) run_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) terminal_state: TerminalHandoffStateV1,
    pub(crate) application_result_ref: Option<AuthorityObjectRefV1>,
    pub(crate) input_acceptance_ref: Option<AuthorityObjectRefV1>,
    pub(crate) startup_ownership_result_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_completion_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_application_result_ref: Option<AuthorityObjectRefV1>,
    pub(crate) recorded_at: TimestampV1,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_runtime::host_session_authority::canonical_json;

    #[test]
    fn timestamp_v1_accepts_only_exact_utc_nine_fraction_shape() {
        let valid = TimestampV1::parse("2026-07-11T12:49:11.000000000Z").unwrap();
        assert_eq!(valid.as_str(), "2026-07-11T12:49:11.000000000Z");
        assert_eq!(
            canonical_json::to_vec(&valid).unwrap(),
            br#""2026-07-11T12:49:11.000000000Z""#
        );

        for invalid in [
            "2026-07-11T12:49:11Z",
            "2026-07-11T12:49:11.000Z",
            "2026-07-11T12:49:11.000000000z",
            "2026-07-11T12:49:11.000000000+00:00",
            "2026-07-11T12:49:60.000000000Z",
            "2026-02-29T12:49:11.000000000Z",
        ] {
            assert!(TimestampV1::parse(invalid).is_err(), "accepted {invalid}");
        }
    }

    #[test]
    fn timestamp_v1_decoder_rejects_noncanonical_timestamp_strings() {
        assert!(canonical_json::from_slice::<TimestampV1>(
            br#""2026-07-11T12:49:11.000000000+00:00""#,
        )
        .is_err());
    }
}
