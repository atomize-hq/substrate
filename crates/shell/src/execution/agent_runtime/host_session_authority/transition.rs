//! Durable A1.2a greenfield Start transition protocol.

use std::fmt;

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use substrate_common::OpaqueAuthorityCommitmentV1;

use super::super::obligation_ledger::{self, ObligationLedgerSnapshotReadV1};
use super::canonical_json;
use super::facade::{HostSessionAuthority, ResolvedCurrentAuthorityV1};
use super::hash::{canonical_object_bytes, canonical_sha256, CanonicalObjectHashInputV1};
use super::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, ApplicationResultHashInputV1,
    ApplicationResultPhaseV1, AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1,
    AuthorityObjectKindV1, AuthorityObjectRefV1, DurableSessionAuthorityHashInputV1,
    DurableSessionAuthorityOriginV1, HostAttachCapabilitiesV1, HostAttachContractHashInputV1,
    HostAttachContractV1, HostAttachLaunchKnobsV1, HostPostTurnDispositionV1,
    HostPostTurnProtocolActorV1, HostPostTurnProtocolEventKindV1, HostPostTurnTerminalReasonV1,
    HostSessionAuthorityPreconditionV1, HostSessionPostureV1, HostSessionTransitionCallerKindV1,
    HostSessionTransitionCallerV1, HostSessionTransitionModeV1,
    HostSessionTransitionPayloadHashInputV1, HostStartupOwnershipEvidenceV1,
    HostStartupOwnershipProtocolActorV1, HostStartupOwnershipProtocolEventV1,
    HostStartupTerminalReasonV1, InputAcceptanceHashInputV1, ObligationAttentionDispositionV1,
    ObligationSnapshotHashInputV1, PolicyObjectHashInputV1, PostTurnCompletionHashInputV1,
    PostTurnCompletionOutcomeV1, PostTurnProtocolEventHashInputV1, StartupOwnershipOutcomeV1,
    StartupOwnershipResultHashInputV1, TerminalHandoffHashInputV1, TerminalHandoffHashInputV2,
    TerminalHandoffStateV1, TimestampV1, TransitionTransportPayloadObjectV1,
    UnresolvedAttentionObligationSnapshotEntryV1, WorkspaceBindingV1, WorldBindingV1,
};
use super::store::{
    self, GeneratedObjectV1, ObjectVerificationContextV1, VersionedObjectVerificationParentIntentV1,
};
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
    HostSessionPostTurnApplicationV1, HostSessionPostTurnApplicationV2,
    HostSessionStartupOwnershipApplicationV1, HostSessionTransitionApplicationJournalV2,
    HostSessionTransitionApplicationJournalV3, HostSessionTransitionInputHandoffV1,
    HostSessionTransitionIntentStateV2, HostSessionTransitionIntentStateV3,
    HostSessionTransitionIntentV2, HostSessionTransitionIntentV3,
    HostSessionTransitionTransportPayloadStateV1, InitialTransitionApplicationJournalV1,
    IssuerRequestIndexEntryV1, RetainedWorkerAuthorityRegistrationRequestStateV1,
    SessionIdReservationV1, SessionIdTombstoneV1, SessionNamespaceRecordV1, StartTombstoneStateV1,
    StateRootV2, StateRootV3, VersionedStateRoot,
};
use super::trusted_fs::TrustedWorkspaceRoot;

pub(crate) const DEFAULT_INTENT_TTL_SECONDS: i64 = 300;
pub(crate) const MAX_INTENT_TTL_SECONDS: i64 = 900;
pub(crate) const DEFAULT_CLAIM_LEASE_SECONDS: i64 = 30;
pub(crate) const MAX_CLAIM_LEASE_SECONDS: i64 = 60;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StartContractMaterialV1 {
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) policy: PolicyObjectHashInputV1,
    pub(crate) capabilities: HostAttachCapabilitiesV1,
    pub(crate) launch_knobs: HostAttachLaunchKnobsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IssueHostSessionTransitionRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) authority_precondition: HostSessionAuthorityPreconditionV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) source_authoritative_participant_id: Option<String>,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) target_participant_lease_token: Vec<u8>,
    pub(crate) run_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) start_contract: StartContractMaterialV1,
    pub(crate) transition_input: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IssueSuccessorTransitionRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) authority_precondition: HostSessionAuthorityPreconditionV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) source_authoritative_participant_id: Option<String>,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) target_participant_lease_token: Vec<u8>,
    pub(crate) run_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) resume_handle_ref: Option<AuthorityObjectRefV1>,
    pub(crate) transition_input: Option<Vec<u8>>,
    pub(crate) post_turn_disposition: Option<HostPostTurnDispositionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransitionIssueOutcomeV1 {
    Issued(HostSessionTransitionIntentV2),
    Joined(HostSessionTransitionIntentV2),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SuccessorTransitionIssueOutcomeV1 {
    Issued(HostSessionTransitionIntentV3),
    Joined(HostSessionTransitionIntentV3),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IssueCrashPointV1 {
    DescriptorPublished,
    PolicyPublished,
    AttachPublished,
    LeasePublished,
    InputPublished,
    TransportPublished,
    RootCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimHostSessionTransitionRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) expected_intent_revision: u64,
    pub(crate) claim_id: String,
    pub(crate) claimant_attempt_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransitionClaimOutcomeV1 {
    Claimed(HostSessionTransitionIntentV2),
    Reclaimed(HostSessionTransitionIntentV2),
    Joined(HostSessionTransitionIntentV2),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SuccessorTransitionClaimOutcomeV1 {
    Claimed(HostSessionTransitionIntentV3),
    Reclaimed(HostSessionTransitionIntentV3),
    Joined(HostSessionTransitionIntentV3),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyHostSessionTransitionRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) expected_intent_revision: u64,
    pub(crate) claim_id: String,
    pub(crate) expected_claim_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransitionApplicationOutcomeV1 {
    Applied(HostSessionTransitionIntentV2),
    Joined(HostSessionTransitionIntentV2),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SuccessorTransitionApplicationOutcomeV1 {
    Applied(HostSessionTransitionIntentV3),
    Joined(HostSessionTransitionIntentV3),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExpireHostSessionTransitionRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) expected_intent_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransitionTerminalOutcomeV1 {
    Expired(HostSessionTransitionIntentV2),
    Joined(HostSessionTransitionIntentV2),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SuccessorTransitionTerminalOutcomeV1 {
    Expired(HostSessionTransitionIntentV3),
    Joined(HostSessionTransitionIntentV3),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolveStartupOwnershipRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) protocol_actor: HostStartupOwnershipProtocolActorV1,
    pub(crate) protocol_event: HostStartupOwnershipProtocolEventV1,
    pub(crate) observed_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StartupOwnershipResolutionOutcomeV1 {
    ResolvedStart(HostSessionTransitionIntentV2),
    JoinedStart(HostSessionTransitionIntentV2),
    ResolvedSuccessor(HostSessionTransitionIntentV3),
    JoinedSuccessor(HostSessionTransitionIntentV3),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcceptTransitionInputRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) accepting_participant_id: String,
    pub(crate) accepted_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InputAcceptanceOutcomeV1 {
    Accepted(HostSessionTransitionIntentV3),
    Joined(HostSessionTransitionIntentV3),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvePostTurnRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) protocol_actor: HostPostTurnProtocolActorV1,
    pub(crate) event_id: String,
    pub(crate) event_sequence: u64,
    pub(crate) kind: HostPostTurnProtocolEventKindV1,
    pub(crate) emitted_at: TimestampV1,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) stream_id: String,
    pub(crate) accepted_work_identity: super::super::state_store::AcceptedWorldWorkIdentityV1,
    pub(crate) host_transition_correlation: substrate_common::HostTransitionWorkCorrelationV1,
    pub(crate) completed_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PostTurnResolutionOutcomeV1 {
    AwaitingObligationCut(HostSessionTransitionIntentV3),
    Applied(HostSessionTransitionIntentV3),
    Joined(HostSessionTransitionIntentV3),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConsumeObligationSnapshotRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) ledger_read: ObligationLedgerSnapshotReadV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ObligationSnapshotConsumptionOutcomeV1 {
    Pending(HostSessionTransitionIntentV3),
    Applied(HostSessionTransitionIntentV3),
    Joined(HostSessionTransitionIntentV3),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExpiryCrashPointV1 {
    TerminalPublished,
    RootCommitted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApplicationCrashPointV1 {
    ResultPublished,
    RootCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransitionProtocolError(String);

impl fmt::Display for TransitionProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TransitionProtocolError {}

impl HostSessionAuthority {
    pub(crate) fn issue_start(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_start_inner(request, now_timestamp()?, DEFAULT_INTENT_TTL_SECONDS, None)
    }

    #[cfg(test)]
    pub(crate) fn issue_start_at(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_start_inner(request, issued_at, ttl_seconds, None)
    }

    #[cfg(test)]
    pub(crate) fn issue_start_at_with_crash_point(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
        crash_point: IssueCrashPointV1,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_start_inner(request, issued_at, ttl_seconds, Some(crash_point))
    }

    fn issue_start_inner(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
        crash_point: Option<IssueCrashPointV1>,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_start_once(request, issued_at, ttl_seconds, crash_point)
    }

    fn issue_start_once(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
        crash_point: Option<IssueCrashPointV1>,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        validate_start_request_shape(request, ttl_seconds)?;
        let workspace = TrustedWorkspaceRoot::open_exact(&request.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let current = self.ensure_a12a_root(request)?;
        if let Some(joined) = exact_issuance_join(&current, request)? {
            workspace.revalidate().map_err(protocol_error)?;
            verify_joined_objects(self, &current, request, &joined)?;
            workspace.revalidate().map_err(protocol_error)?;
            return Ok(TransitionIssueOutcomeV1::Joined(joined));
        }
        validate_new_start_request(&current, request, ttl_seconds)?;
        let expires_at = add_seconds(&issued_at, ttl_seconds)?;
        let object_context = ObjectVerificationContextV1 {
            intent_id: request.intent_id.clone(),
            run_id: request.run_id.clone(),
            parent_intent: None,
        };

        let descriptor_value = AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: request.start_contract.descriptor.clone(),
        };
        let descriptor_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::AgentDescriptor,
            CanonicalObjectHashInputV1::AgentDescriptor(&descriptor_value),
        )
        .map_err(protocol_error)?;
        let descriptor = self.prepare_generated_start_object(
            &current,
            AuthorityObjectKindV1::AgentDescriptor,
            &descriptor_bytes,
            None,
        )?;
        stop_at(crash_point, IssueCrashPointV1::DescriptorPublished)?;

        let policy_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::Policy,
            CanonicalObjectHashInputV1::Policy(&request.start_contract.policy),
        )
        .map_err(protocol_error)?;
        let policy = self.prepare_generated_start_object(
            &current,
            AuthorityObjectKindV1::Policy,
            &policy_bytes,
            None,
        )?;
        stop_at(crash_point, IssueCrashPointV1::PolicyPublished)?;

        let attach_value = HostAttachContractHashInputV1 {
            schema_version: 1,
            contract: HostAttachContractV1 {
                schema_version: 1,
                backend_id: request.start_contract.descriptor.backend_id.clone(),
                execution_scope: request.start_contract.descriptor.execution_scope,
                protocol: request.start_contract.descriptor.protocol.clone(),
                descriptor_ref: descriptor.reference.clone(),
                capabilities: request.start_contract.capabilities.clone(),
                attach_launch_knobs: request.start_contract.launch_knobs.clone(),
                policy_ref: policy.reference.clone(),
                continuity_resume_handle_ref: None,
            },
        };
        let attach_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::HostAttachContract,
            CanonicalObjectHashInputV1::HostAttachContract(&attach_value),
        )
        .map_err(protocol_error)?;
        let attach = self.prepare_generated_start_object(
            &current,
            AuthorityObjectKindV1::HostAttachContract,
            &attach_bytes,
            None,
        )?;
        stop_at(crash_point, IssueCrashPointV1::AttachPublished)?;

        let lease = self.prepare_generated_start_object(
            &current,
            AuthorityObjectKindV1::LeaseToken,
            &request.target_participant_lease_token,
            Some(&object_context),
        )?;
        stop_at(crash_point, IssueCrashPointV1::LeasePublished)?;
        let transition_input = match request.transition_input.as_deref() {
            Some(bytes) => Some(self.prepare_generated_start_object(
                &current,
                AuthorityObjectKindV1::TransitionInput,
                bytes,
                Some(&object_context),
            )?),
            None => None,
        };
        if transition_input.is_some() {
            stop_at(crash_point, IssueCrashPointV1::InputPublished)?;
        }

        let transport_value = TransitionTransportPayloadObjectV1 {
            schema_version: 1,
            intent_id: request.intent_id.clone(),
            mode: request.mode,
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            caller: request.caller.clone(),
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            target_participant_lease_token_ref: lease.reference.clone(),
            run_id: request.run_id.clone(),
            resulting_authoritative_lineage: request.resulting_authoritative_lineage.clone(),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            descriptor_ref: descriptor.reference.clone(),
            host_attach_contract_ref: attach.reference.clone(),
            resume_handle_ref: None,
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: None,
        };
        let transport_bytes = canonical_json::to_vec(&transport_value).map_err(protocol_error)?;
        let transport_ref = store::allocate_sensitive_object_ref_v2_opened(
            self.trusted_root(),
            current.root_revision,
            AuthorityObjectKindV1::TransitionTransportPayload,
            &transport_bytes,
            &object_context,
        )
        .map_err(protocol_error)?;
        let payload_value = HostSessionTransitionPayloadHashInputV1 {
            schema_version: 1,
            intent_id: request.intent_id.clone(),
            issuer_request_id: request.issuer_request_id.clone(),
            mode: request.mode,
            authority_precondition: request.authority_precondition.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            caller: request.caller.clone(),
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            target_participant_lease_token_ref: lease.reference.clone(),
            run_id: request.run_id.clone(),
            resulting_authoritative_lineage: request.resulting_authoritative_lineage.clone(),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            descriptor_ref: descriptor.reference.clone(),
            host_attach_contract_ref: attach.reference.clone(),
            resume_handle_ref: None,
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: None,
            transport_payload_ref: transport_ref.clone(),
            issued_at: issued_at.clone(),
            expires_at: expires_at.clone(),
        };
        let payload_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&payload_value).map_err(protocol_error)?,
        };
        let intent = HostSessionTransitionIntentV2 {
            schema_version: 2,
            intent_id: request.intent_id.clone(),
            issuer_request_id: request.issuer_request_id.clone(),
            intent_revision: 1,
            mode: request.mode,
            authority_precondition: request.authority_precondition.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            caller: request.caller.clone(),
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            target_participant_lease_token_ref: lease.reference.clone(),
            run_id: request.run_id.clone(),
            resulting_authoritative_lineage: request.resulting_authoritative_lineage.clone(),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            descriptor_ref: descriptor.reference.clone(),
            host_attach_contract_ref: attach.reference.clone(),
            resume_handle_ref: None,
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: None,
            transport_payload_ref: transport_ref.clone(),
            payload_commitment: payload_commitment.clone(),
            issued_at: issued_at.clone(),
            expires_at,
            state: HostSessionTransitionIntentStateV2::Issued,
            input_handoff: transition_input.as_ref().map_or(
                HostSessionTransitionInputHandoffV1::NotApplicable,
                |input| HostSessionTransitionInputHandoffV1::Pending {
                    input_ref: input.reference.clone(),
                    run_id: request.run_id.clone(),
                },
            ),
            transport_payload_state: HostSessionTransitionTransportPayloadStateV1::Retained,
            updated_at: issued_at.clone(),
        };
        let transport_context = ObjectVerificationContextV1 {
            intent_id: request.intent_id.clone(),
            run_id: request.run_id.clone(),
            parent_intent: Some(VersionedObjectVerificationParentIntentV1::V2(Box::new(
                intent.clone(),
            ))),
        };
        store::prepare_typed_object_v2_opened(
            self.trusted_root(),
            current.root_revision,
            &transport_ref,
            &transport_bytes,
            Some(&transport_context),
        )
        .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::TransportPublished)?;

        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision, "authority root")?;
        proposed.session_namespace_map.insert(
            request.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::StartReservation(SessionIdReservationV1 {
                schema_version: 1,
                orchestration_session_id: request.orchestration_session_id.clone(),
                intent_id: request.intent_id.clone(),
                issuer_request_id: request.issuer_request_id.clone(),
                payload_commitment: payload_commitment.clone(),
                reserved_at: issued_at,
            }),
        );
        proposed
            .transition_intent_map
            .insert(request.intent_id.clone(), intent.clone());
        proposed.issuer_request_index.insert(
            request.issuer_request_id.clone(),
            IssuerRequestIndexEntryV1 {
                schema_version: 1,
                issuer_request_id: request.issuer_request_id.clone(),
                orchestration_session_id: request.orchestration_session_id.clone(),
                intent_id: request.intent_id.clone(),
                payload_commitment,
            },
        );
        for (object, kind, byte_length) in [
            (
                &descriptor.reference,
                AuthorityObjectKindV1::AgentDescriptor,
                descriptor.byte_length,
            ),
            (
                &policy.reference,
                AuthorityObjectKindV1::Policy,
                policy.byte_length,
            ),
            (
                &attach.reference,
                AuthorityObjectKindV1::HostAttachContract,
                attach.byte_length,
            ),
            (
                &lease.reference,
                AuthorityObjectKindV1::LeaseToken,
                lease.byte_length,
            ),
            (
                &transport_ref,
                AuthorityObjectKindV1::TransitionTransportPayload,
                transport_bytes.len() as u64,
            ),
        ] {
            insert_present_index(&mut proposed, object, kind, byte_length)?;
        }
        if let Some(input) = transition_input {
            insert_present_index(
                &mut proposed,
                &input.reference,
                AuthorityObjectKindV1::TransitionInput,
                input.byte_length,
            )?;
        }
        workspace.revalidate().map_err(protocol_error)?;
        proposed.validate().map_err(protocol_error)?;
        store::commit_v2_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::RootCommitted)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(TransitionIssueOutcomeV1::Issued(intent))
    }

    pub(crate) fn claim_start(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
    ) -> Result<TransitionClaimOutcomeV1, TransitionProtocolError> {
        self.claim_start_inner(request, now_timestamp()?, DEFAULT_CLAIM_LEASE_SECONDS)
    }

    #[cfg(test)]
    pub(crate) fn claim_start_at(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
        claimed_at: TimestampV1,
        claim_lease_seconds: i64,
    ) -> Result<TransitionClaimOutcomeV1, TransitionProtocolError> {
        self.claim_start_inner(request, claimed_at, claim_lease_seconds)
    }

    fn claim_start_inner(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
        claimed_at: TimestampV1,
        claim_lease_seconds: i64,
    ) -> Result<TransitionClaimOutcomeV1, TransitionProtocolError> {
        if claim_lease_seconds <= 0 || claim_lease_seconds > MAX_CLAIM_LEASE_SECONDS {
            return Err(error("transition claim lease is outside the V1 bound"));
        }
        let current = self.read_a12a_root()?;
        let intent = exact_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "claim",
        )?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        if let HostSessionTransitionIntentStateV2::Claimed {
            claim_id,
            claimant_attempt_id,
            claim_expires_at,
            ..
        } = &intent.state
        {
            if claim_id == &request.claim_id
                && claimant_attempt_id == &request.claimant_attempt_id
                && claimed_at.as_str() < claim_expires_at.as_str()
                && claimed_at.as_str() < intent.expires_at.as_str()
            {
                verify_start_reservation(&current, &intent)?;
                verify_start_intent_objects(self, &current, &intent)?;
                workspace.revalidate().map_err(protocol_error)?;
                return Ok(TransitionClaimOutcomeV1::Joined(intent));
            }
            if claim_id == &request.claim_id && claimant_attempt_id == &request.claimant_attempt_id
            {
                return Err(error(
                    "expired transition claim cannot join or reuse its identity",
                ));
            }
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("transition claim intent revision is stale"));
        }
        if request.claim_id.is_empty() || request.claimant_attempt_id.is_empty() {
            return Err(error("transition claim identity must be non-empty"));
        }
        if claimed_at.as_str() >= intent.expires_at.as_str() {
            return Err(error("transition intent has reached its fixed expiry"));
        }
        if claimed_at.as_str() < intent.issued_at.as_str()
            || claimed_at.as_str() < intent.updated_at.as_str()
        {
            return Err(error("transition claim time regresses durable intent time"));
        }
        let reclaimed = match &intent.state {
            HostSessionTransitionIntentStateV2::Issued => false,
            HostSessionTransitionIntentStateV2::Claimed {
                claim_expires_at, ..
            } if claimed_at.as_str() >= claim_expires_at.as_str() => true,
            HostSessionTransitionIntentStateV2::Claimed { .. } => {
                return Err(error("transition intent has a current different claim"));
            }
            HostSessionTransitionIntentStateV2::Applied { .. }
            | HostSessionTransitionIntentStateV2::Rejected { .. }
            | HostSessionTransitionIntentStateV2::Expired { .. } => {
                return Err(error("terminal transition intent cannot be claimed"));
            }
        };
        let claim_expires_at = add_seconds(&claimed_at, claim_lease_seconds)?;
        if claim_expires_at.as_str() > intent.expires_at.as_str() {
            return Err(error("transition claim would exceed fixed intent expiry"));
        }
        verify_start_reservation(&current, &intent)?;
        verify_start_intent_objects(self, &current, &intent)?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .transition_intent_map
            .get_mut(&request.intent_id)
            .ok_or_else(|| error("transition intent disappeared during claim"))?;
        next.intent_revision = next_intent_revision;
        next.state = HostSessionTransitionIntentStateV2::Claimed {
            claim_id: request.claim_id.clone(),
            claimant_attempt_id: request.claimant_attempt_id.clone(),
            claim_revision: next_intent_revision,
            claimed_at: claimed_at.clone(),
            claim_expires_at,
        };
        next.updated_at = claimed_at;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v2_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let committed = proposed.transition_intent_map[&request.intent_id].clone();
        if reclaimed {
            Ok(TransitionClaimOutcomeV1::Reclaimed(committed))
        } else {
            Ok(TransitionClaimOutcomeV1::Claimed(committed))
        }
    }

    pub(crate) fn apply_start(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_start_inner(request, now_timestamp()?, None)
    }

    #[cfg(test)]
    pub(crate) fn apply_start_at(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_start_inner(request, applied_at, None)
    }

    #[cfg(test)]
    pub(crate) fn apply_start_at_with_crash_point(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
        crash_point: ApplicationCrashPointV1,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_start_inner(request, applied_at, Some(crash_point))
    }

    fn apply_start_inner(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
        crash_point: Option<ApplicationCrashPointV1>,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12a_root()?;
        let intent = exact_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "application",
        )?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV2::Applied { .. }
        ) {
            verify_applied_start(self, &current, &intent, request)?;
            workspace.revalidate().map_err(protocol_error)?;
            return Ok(TransitionApplicationOutcomeV1::Joined(intent));
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("transition application intent revision is stale"));
        }
        let (claim_id, claimant_attempt_id, claim_revision, claim_expires_at) = match &intent.state
        {
            HostSessionTransitionIntentStateV2::Claimed {
                claim_id,
                claimant_attempt_id,
                claim_revision,
                claim_expires_at,
                ..
            } => (
                claim_id,
                claimant_attempt_id,
                *claim_revision,
                claim_expires_at,
            ),
            _ => return Err(error("transition application requires a current claim")),
        };
        if claim_id != &request.claim_id || claim_revision != request.expected_claim_revision {
            return Err(error(
                "transition application claim identity or revision mismatch",
            ));
        }
        if applied_at.as_str() >= claim_expires_at.as_str()
            || applied_at.as_str() >= intent.expires_at.as_str()
        {
            return Err(error("transition application claim or intent has expired"));
        }
        if applied_at.as_str() < intent.updated_at.as_str() {
            return Err(error(
                "transition application time regresses durable intent time",
            ));
        }
        verify_start_reservation(&current, &intent)?;
        verify_start_intent_objects(self, &current, &intent)?;

        let attach_bytes = store::read_typed_object_v2_opened(
            self.trusted_root(),
            current.root_revision,
            &intent.host_attach_contract_ref,
            None,
        )
        .map_err(protocol_error)?;
        let attach: HostAttachContractHashInputV1 =
            canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
        let policy_bytes = store::read_typed_object_v2_opened(
            self.trusted_root(),
            current.root_revision,
            &attach.contract.policy_ref,
            None,
        )
        .map_err(protocol_error)?;
        let policy: PolicyObjectHashInputV1 =
            canonical_json::from_slice(&policy_bytes).map_err(protocol_error)?;
        let authority_record = DurableSessionAuthorityV1 {
            schema_version: 1,
            orchestration_session_id: intent.orchestration_session_id.clone(),
            shell_trace_session_id: intent.shell_trace_session_id.clone(),
            authority_revision: 1,
            origin: DurableSessionAuthorityOriginV1::StartIntent {
                intent_id: intent.intent_id.clone(),
                issuer_request_id: intent.issuer_request_id.clone(),
                payload_commitment: intent.payload_commitment.clone(),
            },
            authoritative_participant_lineage: intent.resulting_authoritative_lineage.clone(),
            active_authoritative_participant_id: Some(
                intent.target_authoritative_participant_id.clone(),
            ),
            workspace_binding: intent.workspace_binding.clone(),
            world_binding: intent.world_binding.clone(),
            host_attach_contract_ref: Some(intent.host_attach_contract_ref.clone()),
            retained_worker_refs: Vec::new(),
            internal_resume_handle_refs: Vec::new(),
            lifecycle_posture: HostSessionPostureV1::ActiveAttached,
            current_policy_ref: Some(attach.contract.policy_ref.clone()),
            current_policy_revision: Some(policy.policy_revision),
            updated_at: applied_at.clone(),
        };
        let authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&authority_record))
                .map_err(protocol_error)?,
        };
        let application_value = ApplicationResultHashInputV1 {
            schema_version: 1,
            intent_id: intent.intent_id.clone(),
            mode: HostSessionTransitionModeV1::Start,
            run_id: intent.run_id.clone(),
            phase: ApplicationResultPhaseV1::InitialTransition {
                authority_revision_before: None,
                authority_revision_after: 1,
                active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
                resulting_posture: HostSessionPostureV1::ActiveAttached,
                authority_record_commitment: authority_record_commitment.clone(),
                post_turn_pending_run_id: None,
            },
            applied_at: applied_at.clone(),
        };
        let application_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::ApplicationResult,
            CanonicalObjectHashInputV1::ApplicationResult(&application_value),
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let application = self.prepare_generated_start_object(
            &current,
            AuthorityObjectKindV1::ApplicationResult,
            &application_bytes,
            None,
        )?;
        stop_application_at(crash_point, ApplicationCrashPointV1::ResultPublished)?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        proposed.session_namespace_map.insert(
            intent.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::Authority(Box::new(authority_record)),
        );
        let next = proposed
            .transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during application"))?;
        next.intent_revision = next_intent_revision;
        next.state = HostSessionTransitionIntentStateV2::Applied {
            claim_id: claim_id.clone(),
            claimant_attempt_id: claimant_attempt_id.clone(),
            authority_revision_before: None,
            authority_revision_after: 1,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: authority_record_commitment.clone(),
            application_result_ref: application.reference.clone(),
            startup_ownership: Box::new(HostSessionStartupOwnershipApplicationV1::Pending {
                expected_run_id: intent.run_id.clone(),
                expected_authority_revision: 1,
                expected_active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
            }),
            post_turn: Box::new(HostSessionPostTurnApplicationV1::NotApplicable),
            applied_at: applied_at.clone(),
        };
        next.updated_at = applied_at.clone();
        proposed.application_journal.insert(
            intent.intent_id.clone(),
            HostSessionTransitionApplicationJournalV2 {
                schema_version: 2,
                intent_id: intent.intent_id.clone(),
                initial_application: InitialTransitionApplicationJournalV1 {
                    authority_revision_before: None,
                    authority_revision_after: 1,
                    authority_record_commitment,
                    application_result_ref: application.reference.clone(),
                    applied_at,
                },
                startup_terminal_application: None,
                post_turn_application: None,
            },
        );
        insert_present_index(
            &mut proposed,
            &application.reference,
            AuthorityObjectKindV1::ApplicationResult,
            application.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v2_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        stop_application_at(crash_point, ApplicationCrashPointV1::RootCommitted)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(TransitionApplicationOutcomeV1::Applied(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn expire_start(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        self.expire_start_inner(request, now_timestamp()?, None)
    }

    #[cfg(test)]
    pub(crate) fn expire_start_at(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        self.expire_start_inner(request, expired_at, None)
    }

    #[cfg(test)]
    pub(crate) fn expire_start_at_with_crash_point(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
        crash_point: ExpiryCrashPointV1,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        self.expire_start_inner(request, expired_at, Some(crash_point))
    }

    fn expire_start_inner(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
        crash_point: Option<ExpiryCrashPointV1>,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12a_root()?;
        let intent = exact_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "expiry",
        )?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV2::Expired { .. }
        ) {
            verify_terminal_start(self, &current, &intent)?;
            workspace.revalidate().map_err(protocol_error)?;
            return Ok(TransitionTerminalOutcomeV1::Joined(intent));
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("transition expiry intent revision is stale"));
        }
        if expired_at.as_str() < intent.expires_at.as_str() {
            return Err(error("transition intent has not reached its fixed expiry"));
        }
        if expired_at.as_str() < intent.updated_at.as_str() {
            return Err(error(
                "transition expiry time regresses durable intent time",
            ));
        }
        if !matches!(
            intent.state,
            HostSessionTransitionIntentStateV2::Issued
                | HostSessionTransitionIntentStateV2::Claimed { .. }
        ) || current.application_journal.contains_key(&intent.intent_id)
        {
            return Err(error("transition intent cannot be expired"));
        }
        verify_start_reservation(&current, &intent)?;
        verify_start_intent_objects(self, &current, &intent)?;

        let terminal_value = TerminalHandoffHashInputV1 {
            schema_version: 1,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            terminal_state: TerminalHandoffStateV1::Expired,
            application_result_ref: None,
            input_acceptance_ref: None,
            post_turn_completion_ref: None,
            post_turn_application_result_ref: None,
            recorded_at: expired_at.clone(),
        };
        let terminal_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::TerminalHandoff,
            CanonicalObjectHashInputV1::TerminalHandoff(&terminal_value),
        )
        .map_err(protocol_error)?;
        let terminal = self.prepare_generated_start_object(
            &current,
            AuthorityObjectKindV1::TerminalHandoff,
            &terminal_bytes,
            None,
        )?;
        stop_expiry_at(crash_point, ExpiryCrashPointV1::TerminalPublished)?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during expiry"))?;
        next.intent_revision = next_intent_revision;
        next.state = HostSessionTransitionIntentStateV2::Expired {
            terminal_handoff_ref: terminal.reference.clone(),
            expired_at: expired_at.clone(),
        };
        if let HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id } =
            &next.input_handoff
        {
            next.input_handoff = HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                input_ref: input_ref.clone(),
                run_id: run_id.clone(),
                terminal_handoff_ref: terminal.reference.clone(),
                terminal_at: expired_at.clone(),
            };
        }
        next.updated_at = expired_at.clone();
        proposed.session_namespace_map.insert(
            intent.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::StartTombstone(SessionIdTombstoneV1 {
                schema_version: 1,
                orchestration_session_id: intent.orchestration_session_id.clone(),
                intent_id: intent.intent_id.clone(),
                issuer_request_id: intent.issuer_request_id.clone(),
                payload_commitment: intent.payload_commitment.clone(),
                terminal_state: StartTombstoneStateV1::Expired,
                terminal_handoff_ref: terminal.reference.clone(),
                tombstoned_at: expired_at,
            }),
        );
        insert_present_index(
            &mut proposed,
            &terminal.reference,
            AuthorityObjectKindV1::TerminalHandoff,
            terminal.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v2_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        stop_expiry_at(crash_point, ExpiryCrashPointV1::RootCommitted)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(TransitionTerminalOutcomeV1::Expired(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn issue_successor(
        &self,
        current: &ResolvedCurrentAuthorityV1,
        request: &IssueSuccessorTransitionRequestV1,
    ) -> Result<SuccessorTransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_successor_inner(
            request,
            current,
            now_timestamp()?,
            DEFAULT_INTENT_TTL_SECONDS,
        )
    }

    #[cfg(test)]
    pub(crate) fn issue_successor_at(
        &self,
        current: &ResolvedCurrentAuthorityV1,
        request: &IssueSuccessorTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
    ) -> Result<SuccessorTransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_successor_inner(request, current, issued_at, ttl_seconds)
    }

    fn issue_successor_inner(
        &self,
        request: &IssueSuccessorTransitionRequestV1,
        current: &ResolvedCurrentAuthorityV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
    ) -> Result<SuccessorTransitionIssueOutcomeV1, TransitionProtocolError> {
        let current_root = self.read_a12b_root()?;
        if let Some(joined) = exact_successor_issuance_join(&current_root, request)? {
            verify_successor_joined_objects(self, &current_root, request, &joined)?;
            return Ok(SuccessorTransitionIssueOutcomeV1::Joined(joined));
        }
        let durable_current =
            current_authority_from_v3_root(&current_root, &request.orchestration_session_id)?;
        validate_new_successor_request(
            &current_root,
            durable_current,
            current,
            request,
            ttl_seconds,
        )?;
        let workspace = TrustedWorkspaceRoot::open_exact(&request.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;

        let descriptor_ref = current.caller.descriptor_ref.clone();
        let attach_ref = durable_current
            .host_attach_contract_ref
            .clone()
            .ok_or_else(|| error("successor issuance requires a current attach contract"))?;
        let object_context = ObjectVerificationContextV1 {
            intent_id: request.intent_id.clone(),
            run_id: request.run_id.clone(),
            parent_intent: None,
        };
        let expires_at = add_seconds(&issued_at, ttl_seconds)?;
        let lease = self.prepare_generated_successor_object(
            &current_root,
            AuthorityObjectKindV1::LeaseToken,
            &request.target_participant_lease_token,
            Some(&object_context),
        )?;
        let transition_input = match request.transition_input.as_deref() {
            Some(bytes) => Some(self.prepare_generated_successor_object(
                &current_root,
                AuthorityObjectKindV1::TransitionInput,
                bytes,
                Some(&object_context),
            )?),
            None => None,
        };
        let transport_value = TransitionTransportPayloadObjectV1 {
            schema_version: 1,
            intent_id: request.intent_id.clone(),
            mode: request.mode,
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            caller: request.caller.clone(),
            source_authoritative_participant_id: request
                .source_authoritative_participant_id
                .clone(),
            target_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            target_participant_lease_token_ref: lease.reference.clone(),
            run_id: request.run_id.clone(),
            resulting_authoritative_lineage: request.resulting_authoritative_lineage.clone(),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            descriptor_ref: descriptor_ref.clone(),
            host_attach_contract_ref: attach_ref.clone(),
            resume_handle_ref: request.resume_handle_ref.clone(),
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: request.post_turn_disposition,
        };
        let transport_bytes = canonical_json::to_vec(&transport_value).map_err(protocol_error)?;
        let transport_ref = self.allocate_sensitive_successor_object_ref(
            &current_root,
            AuthorityObjectKindV1::TransitionTransportPayload,
            &transport_bytes,
            &object_context,
        )?;
        let payload_value = HostSessionTransitionPayloadHashInputV1 {
            schema_version: 1,
            intent_id: request.intent_id.clone(),
            issuer_request_id: request.issuer_request_id.clone(),
            mode: request.mode,
            authority_precondition: request.authority_precondition.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            caller: request.caller.clone(),
            source_authoritative_participant_id: request
                .source_authoritative_participant_id
                .clone(),
            target_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            target_participant_lease_token_ref: lease.reference.clone(),
            run_id: request.run_id.clone(),
            resulting_authoritative_lineage: request.resulting_authoritative_lineage.clone(),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            descriptor_ref: descriptor_ref.clone(),
            host_attach_contract_ref: attach_ref.clone(),
            resume_handle_ref: request.resume_handle_ref.clone(),
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: request.post_turn_disposition,
            transport_payload_ref: transport_ref.clone(),
            issued_at: issued_at.clone(),
            expires_at: expires_at.clone(),
        };
        let payload_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&payload_value).map_err(protocol_error)?,
        };
        let intent = HostSessionTransitionIntentV3 {
            schema_version: 3,
            intent_id: request.intent_id.clone(),
            issuer_request_id: request.issuer_request_id.clone(),
            intent_revision: 1,
            mode: request.mode,
            authority_precondition: request.authority_precondition.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            caller: request.caller.clone(),
            source_authoritative_participant_id: request
                .source_authoritative_participant_id
                .clone(),
            target_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            target_participant_lease_token_ref: lease.reference.clone(),
            run_id: request.run_id.clone(),
            resulting_authoritative_lineage: request.resulting_authoritative_lineage.clone(),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            descriptor_ref,
            host_attach_contract_ref: attach_ref,
            resume_handle_ref: request.resume_handle_ref.clone(),
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: request.post_turn_disposition,
            transport_payload_ref: transport_ref.clone(),
            payload_commitment: payload_commitment.clone(),
            issued_at: issued_at.clone(),
            expires_at,
            state: HostSessionTransitionIntentStateV3::Issued,
            input_handoff: transition_input.as_ref().map_or(
                HostSessionTransitionInputHandoffV1::NotApplicable,
                |input| HostSessionTransitionInputHandoffV1::Pending {
                    input_ref: input.reference.clone(),
                    run_id: request.run_id.clone(),
                },
            ),
            transport_payload_state: HostSessionTransitionTransportPayloadStateV1::Retained,
            updated_at: issued_at.clone(),
        };
        let transport_context = ObjectVerificationContextV1 {
            intent_id: request.intent_id.clone(),
            run_id: request.run_id.clone(),
            parent_intent: Some(VersionedObjectVerificationParentIntentV1::V3(Box::new(
                intent.clone(),
            ))),
        };
        self.prepare_typed_successor_object(
            &current_root,
            &transport_ref,
            &transport_bytes,
            Some(&transport_context),
        )?;

        let mut proposed = current_root.clone();
        proposed.root_revision = next_revision(proposed.root_revision, "authority root")?;
        proposed
            .successor_transition_intent_map
            .insert(request.intent_id.clone(), intent.clone());
        proposed.successor_issuer_request_index.insert(
            request.issuer_request_id.clone(),
            IssuerRequestIndexEntryV1 {
                schema_version: 1,
                issuer_request_id: request.issuer_request_id.clone(),
                orchestration_session_id: request.orchestration_session_id.clone(),
                intent_id: request.intent_id.clone(),
                payload_commitment,
            },
        );
        for (reference, kind, byte_length) in [
            (
                &lease.reference,
                AuthorityObjectKindV1::LeaseToken,
                lease.byte_length,
            ),
            (
                &transport_ref,
                AuthorityObjectKindV1::TransitionTransportPayload,
                transport_bytes.len() as u64,
            ),
        ] {
            insert_present_index_v3(&mut proposed, reference, kind, byte_length)?;
        }
        if let Some(input) = transition_input {
            insert_present_index_v3(
                &mut proposed,
                &input.reference,
                AuthorityObjectKindV1::TransitionInput,
                input.byte_length,
            )?;
        }
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current_root,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(SuccessorTransitionIssueOutcomeV1::Issued(intent))
    }

    pub(crate) fn claim_successor(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
    ) -> Result<SuccessorTransitionClaimOutcomeV1, TransitionProtocolError> {
        self.claim_successor_inner(request, now_timestamp()?, DEFAULT_CLAIM_LEASE_SECONDS)
    }

    #[cfg(test)]
    pub(crate) fn claim_successor_at(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
        claimed_at: TimestampV1,
        claim_lease_seconds: i64,
    ) -> Result<SuccessorTransitionClaimOutcomeV1, TransitionProtocolError> {
        self.claim_successor_inner(request, claimed_at, claim_lease_seconds)
    }

    fn claim_successor_inner(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
        claimed_at: TimestampV1,
        claim_lease_seconds: i64,
    ) -> Result<SuccessorTransitionClaimOutcomeV1, TransitionProtocolError> {
        if claim_lease_seconds <= 0 || claim_lease_seconds > MAX_CLAIM_LEASE_SECONDS {
            return Err(error("transition claim lease is outside the V1 bound"));
        }
        let current = self.read_a12b_root()?;
        let intent = exact_successor_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "claim",
        )?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        if let HostSessionTransitionIntentStateV3::Claimed {
            claim_id,
            claimant_attempt_id,
            claim_expires_at,
            ..
        } = &intent.state
        {
            if claim_id == &request.claim_id
                && claimant_attempt_id == &request.claimant_attempt_id
                && claimed_at.as_str() < claim_expires_at.as_str()
                && claimed_at.as_str() < intent.expires_at.as_str()
            {
                verify_successor_intent_objects(self, &current, &intent)?;
                workspace.revalidate().map_err(protocol_error)?;
                return Ok(SuccessorTransitionClaimOutcomeV1::Joined(intent));
            }
            if claim_id == &request.claim_id && claimant_attempt_id == &request.claimant_attempt_id
            {
                return Err(error(
                    "expired transition claim cannot join or reuse its identity",
                ));
            }
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("transition claim intent revision is stale"));
        }
        if request.claim_id.is_empty() || request.claimant_attempt_id.is_empty() {
            return Err(error("transition claim identity must be non-empty"));
        }
        if claimed_at.as_str() >= intent.expires_at.as_str() {
            return Err(error("transition intent has reached its fixed expiry"));
        }
        if claimed_at.as_str() < intent.issued_at.as_str()
            || claimed_at.as_str() < intent.updated_at.as_str()
        {
            return Err(error("transition claim time regresses durable intent time"));
        }
        let reclaimed = match &intent.state {
            HostSessionTransitionIntentStateV3::Issued => false,
            HostSessionTransitionIntentStateV3::Claimed {
                claim_expires_at, ..
            } if claimed_at.as_str() >= claim_expires_at.as_str() => true,
            HostSessionTransitionIntentStateV3::Claimed { .. } => {
                return Err(error("transition intent has a current different claim"));
            }
            HostSessionTransitionIntentStateV3::Applied { .. }
            | HostSessionTransitionIntentStateV3::Rejected { .. }
            | HostSessionTransitionIntentStateV3::Expired { .. } => {
                return Err(error("terminal transition intent cannot be claimed"));
            }
        };
        let claim_expires_at = add_seconds(&claimed_at, claim_lease_seconds)?;
        if claim_expires_at.as_str() > intent.expires_at.as_str() {
            return Err(error("transition claim would exceed fixed intent expiry"));
        }
        verify_successor_authority_precondition_holds(&current, &intent)?;
        verify_successor_intent_objects(self, &current, &intent)?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .successor_transition_intent_map
            .get_mut(&request.intent_id)
            .ok_or_else(|| error("transition intent disappeared during claim"))?;
        next.intent_revision = next_intent_revision;
        next.state = HostSessionTransitionIntentStateV3::Claimed {
            claim_id: request.claim_id.clone(),
            claimant_attempt_id: request.claimant_attempt_id.clone(),
            claim_revision: next_intent_revision,
            claimed_at: claimed_at.clone(),
            claim_expires_at,
        };
        next.updated_at = claimed_at;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let committed = proposed.successor_transition_intent_map[&request.intent_id].clone();
        if reclaimed {
            Ok(SuccessorTransitionClaimOutcomeV1::Reclaimed(committed))
        } else {
            Ok(SuccessorTransitionClaimOutcomeV1::Claimed(committed))
        }
    }

    pub(crate) fn apply_successor(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
    ) -> Result<SuccessorTransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_successor_inner(request, now_timestamp()?)
    }

    #[cfg(test)]
    pub(crate) fn apply_successor_at(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
    ) -> Result<SuccessorTransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_successor_inner(request, applied_at)
    }

    fn apply_successor_inner(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
    ) -> Result<SuccessorTransitionApplicationOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12b_root()?;
        let intent = exact_successor_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "application",
        )?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV3::Applied { .. }
        ) {
            verify_applied_successor(self, &current, &intent, request)?;
            workspace.revalidate().map_err(protocol_error)?;
            return Ok(SuccessorTransitionApplicationOutcomeV1::Joined(intent));
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("transition application intent revision is stale"));
        }
        let (claim_id, claimant_attempt_id, claim_revision, claim_expires_at) = match &intent.state
        {
            HostSessionTransitionIntentStateV3::Claimed {
                claim_id,
                claimant_attempt_id,
                claim_revision,
                claim_expires_at,
                ..
            } => (
                claim_id,
                claimant_attempt_id,
                *claim_revision,
                claim_expires_at,
            ),
            _ => return Err(error("transition application requires a current claim")),
        };
        if claim_id != &request.claim_id || claim_revision != request.expected_claim_revision {
            return Err(error(
                "transition application claim identity or revision mismatch",
            ));
        }
        if applied_at.as_str() >= claim_expires_at.as_str()
            || applied_at.as_str() >= intent.expires_at.as_str()
        {
            return Err(error("transition application claim or intent has expired"));
        }
        if applied_at.as_str() < intent.updated_at.as_str() {
            return Err(error(
                "transition application time regresses durable intent time",
            ));
        }
        verify_successor_authority_precondition_holds(&current, &intent)?;
        verify_successor_intent_objects(self, &current, &intent)?;

        let current_authority =
            current_authority_from_v3_root(&current, &intent.orchestration_session_id)?.clone();
        let authority_revision_after =
            next_revision(current_authority.authority_revision, "durable authority")?;
        let mut authority_record = current_authority.clone();
        authority_record.authority_revision = authority_revision_after;
        authority_record.active_authoritative_participant_id =
            Some(intent.target_authoritative_participant_id.clone());
        authority_record.authoritative_participant_lineage =
            intent.resulting_authoritative_lineage.clone();
        authority_record.host_attach_contract_ref = Some(intent.host_attach_contract_ref.clone());
        authority_record.world_binding = intent.world_binding.clone();
        if let Some(resume_handle_ref) = intent.resume_handle_ref.as_ref() {
            if !authority_record
                .internal_resume_handle_refs
                .iter()
                .any(|current| current == resume_handle_ref)
            {
                authority_record
                    .internal_resume_handle_refs
                    .push(resume_handle_ref.clone());
            }
        }
        authority_record.lifecycle_posture = HostSessionPostureV1::ActiveAttached;
        authority_record.updated_at = applied_at.clone();
        let authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&authority_record))
                .map_err(protocol_error)?,
        };
        let application_value = ApplicationResultHashInputV1 {
            schema_version: 1,
            intent_id: intent.intent_id.clone(),
            mode: intent.mode,
            run_id: intent.run_id.clone(),
            phase: ApplicationResultPhaseV1::InitialTransition {
                authority_revision_before: Some(current_authority.authority_revision),
                authority_revision_after,
                active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
                resulting_posture: HostSessionPostureV1::ActiveAttached,
                authority_record_commitment: authority_record_commitment.clone(),
                post_turn_pending_run_id: matches!(
                    intent.mode,
                    HostSessionTransitionModeV1::ResumeOneTurn
                )
                .then(|| intent.run_id.clone()),
            },
            applied_at: applied_at.clone(),
        };
        let application_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::ApplicationResult,
            CanonicalObjectHashInputV1::ApplicationResult(&application_value),
        )
        .map_err(protocol_error)?;
        let application = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::ApplicationResult,
            &application_bytes,
            None,
        )?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        proposed.session_namespace_map.insert(
            intent.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::Authority(Box::new(authority_record)),
        );
        let next = proposed
            .successor_transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during application"))?;
        next.intent_revision = next_intent_revision;
        next.state = HostSessionTransitionIntentStateV3::Applied {
            claim_id: claim_id.clone(),
            claimant_attempt_id: claimant_attempt_id.clone(),
            authority_revision_before: Some(current_authority.authority_revision),
            authority_revision_after,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: authority_record_commitment.clone(),
            application_result_ref: application.reference.clone(),
            startup_ownership: Box::new(match intent.mode {
                HostSessionTransitionModeV1::Attach => {
                    HostSessionStartupOwnershipApplicationV1::Pending {
                        expected_run_id: intent.run_id.clone(),
                        expected_authority_revision: authority_revision_after,
                        expected_active_authoritative_participant_id: intent
                            .target_authoritative_participant_id
                            .clone(),
                    }
                }
                HostSessionTransitionModeV1::ResumeOneTurn => {
                    HostSessionStartupOwnershipApplicationV1::NotApplicable
                }
                HostSessionTransitionModeV1::Start => unreachable!(),
            }),
            post_turn: Box::new(match intent.mode {
                HostSessionTransitionModeV1::Attach => {
                    HostSessionPostTurnApplicationV2::NotApplicable
                }
                HostSessionTransitionModeV1::ResumeOneTurn => {
                    HostSessionPostTurnApplicationV2::Pending {
                        expected_run_id: intent.run_id.clone(),
                        expected_authority_revision: authority_revision_after,
                    }
                }
                HostSessionTransitionModeV1::Start => unreachable!(),
            }),
            applied_at: applied_at.clone(),
        };
        next.updated_at = applied_at.clone();
        proposed.successor_application_journal.insert(
            intent.intent_id.clone(),
            HostSessionTransitionApplicationJournalV3 {
                schema_version: 3,
                intent_id: intent.intent_id.clone(),
                initial_application: InitialTransitionApplicationJournalV1 {
                    authority_revision_before: Some(current_authority.authority_revision),
                    authority_revision_after,
                    authority_record_commitment,
                    application_result_ref: application.reference.clone(),
                    applied_at,
                },
                startup_terminal_application: None,
                post_turn_application: None,
            },
        );
        insert_present_index_v3(
            &mut proposed,
            &application.reference,
            AuthorityObjectKindV1::ApplicationResult,
            application.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(SuccessorTransitionApplicationOutcomeV1::Applied(
            proposed.successor_transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn expire_successor(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
    ) -> Result<SuccessorTransitionTerminalOutcomeV1, TransitionProtocolError> {
        self.expire_successor_inner(request, now_timestamp()?)
    }

    #[cfg(test)]
    pub(crate) fn expire_successor_at(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
    ) -> Result<SuccessorTransitionTerminalOutcomeV1, TransitionProtocolError> {
        self.expire_successor_inner(request, expired_at)
    }

    fn expire_successor_inner(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
    ) -> Result<SuccessorTransitionTerminalOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12b_root()?;
        let intent = exact_successor_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "expiry",
        )?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV3::Expired { .. }
        ) {
            verify_terminal_successor(self, &current, &intent)?;
            workspace.revalidate().map_err(protocol_error)?;
            return Ok(SuccessorTransitionTerminalOutcomeV1::Joined(intent));
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("transition expiry intent revision is stale"));
        }
        if expired_at.as_str() < intent.expires_at.as_str() {
            return Err(error("transition intent has not reached its fixed expiry"));
        }
        if expired_at.as_str() < intent.updated_at.as_str() {
            return Err(error(
                "transition expiry time regresses durable intent time",
            ));
        }
        if !matches!(
            intent.state,
            HostSessionTransitionIntentStateV3::Issued
                | HostSessionTransitionIntentStateV3::Claimed { .. }
        ) || current
            .successor_application_journal
            .contains_key(&intent.intent_id)
        {
            return Err(error("transition intent cannot be expired"));
        }
        verify_successor_intent_objects(self, &current, &intent)?;

        let terminal_value = super::schema::TerminalHandoffHashInputV2 {
            schema_version: 2,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            terminal_state: TerminalHandoffStateV1::Expired,
            application_result_ref: None,
            input_acceptance_ref: None,
            startup_ownership_result_ref: None,
            post_turn_completion_ref: None,
            post_turn_application_result_ref: None,
            recorded_at: expired_at.clone(),
        };
        let terminal_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::TerminalHandoff,
            CanonicalObjectHashInputV1::TerminalHandoffV2(&terminal_value),
        )
        .map_err(protocol_error)?;
        let terminal = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::TerminalHandoff,
            &terminal_bytes,
            None,
        )?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .successor_transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during expiry"))?;
        next.intent_revision = next_intent_revision;
        next.state = HostSessionTransitionIntentStateV3::Expired {
            terminal_handoff_ref: terminal.reference.clone(),
            expired_at: expired_at.clone(),
        };
        if let HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id } =
            &next.input_handoff
        {
            next.input_handoff = HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                input_ref: input_ref.clone(),
                run_id: run_id.clone(),
                terminal_handoff_ref: terminal.reference.clone(),
                terminal_at: expired_at.clone(),
            };
        }
        next.updated_at = expired_at.clone();
        insert_present_index_v3(
            &mut proposed,
            &terminal.reference,
            AuthorityObjectKindV1::TerminalHandoff,
            terminal.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(SuccessorTransitionTerminalOutcomeV1::Expired(
            proposed.successor_transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn read_a12a_root(&self) -> Result<StateRootV2, TransitionProtocolError> {
        store::read_opened_root_v2(self.trusted_root()).map_err(protocol_error)
    }

    pub(crate) fn read_a12b_root(&self) -> Result<StateRootV3, TransitionProtocolError> {
        match store::read_opened_root_v2_or_v3(self.trusted_root()).map_err(protocol_error)? {
            VersionedStateRoot::V3(root) => Ok(root),
            VersionedStateRoot::V2(root) => store::upgrade_v2_root_to_v3_exact_current_opened(
                self.trusted_root(),
                &root,
                || Ok(()),
            )
            .map_err(protocol_error),
            VersionedStateRoot::V1(_) => Err(error(
                "A1.2b successor transitions require strict StateRootV2 or StateRootV3",
            )),
        }
    }

    pub(crate) fn resolve_startup_ownership(
        &self,
        request: &ResolveStartupOwnershipRequestV1,
    ) -> Result<StartupOwnershipResolutionOutcomeV1, TransitionProtocolError> {
        self.resolve_startup_ownership_inner(request, now_timestamp()?)
    }

    #[cfg(test)]
    pub(crate) fn resolve_startup_ownership_at(
        &self,
        request: &ResolveStartupOwnershipRequestV1,
        resolved_at: TimestampV1,
    ) -> Result<StartupOwnershipResolutionOutcomeV1, TransitionProtocolError> {
        self.resolve_startup_ownership_inner(request, resolved_at)
    }

    fn resolve_startup_ownership_inner(
        &self,
        request: &ResolveStartupOwnershipRequestV1,
        resolved_at: TimestampV1,
    ) -> Result<StartupOwnershipResolutionOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12b_root()?;
        if preserved_start_identity_matches_exact(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
        ) {
            let intent = exact_transition_attempt_v3(
                &current,
                &request.intent_id,
                &request.issuer_request_id,
                &request.payload_commitment,
                "startup ownership resolution",
            )?;
            let workspace =
                TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
                    .map_err(protocol_error)?;
            workspace.revalidate().map_err(protocol_error)?;
            let (
                claim_id,
                claimant_attempt_id,
                authority_revision_after,
                persisted_authority_record_commitment,
                application_result_ref,
                startup_ownership,
                post_turn,
                applied_at,
            ) = match &intent.state {
                HostSessionTransitionIntentStateV2::Applied {
                    claim_id,
                    claimant_attempt_id,
                    authority_revision_after,
                    authority_record_commitment,
                    application_result_ref,
                    startup_ownership,
                    post_turn,
                    applied_at,
                    ..
                } => (
                    claim_id,
                    claimant_attempt_id,
                    *authority_revision_after,
                    authority_record_commitment.clone(),
                    application_result_ref,
                    startup_ownership,
                    post_turn,
                    applied_at,
                ),
                _ => {
                    return Err(error(
                        "startup ownership resolution requires an applied Start transition",
                    ))
                }
            };
            if post_turn.as_ref() != &HostSessionPostTurnApplicationV1::NotApplicable {
                return Err(error(
                    "Start startup ownership cannot resolve alongside post-turn work",
                ));
            }
            if request.observed_at.as_str() < applied_at.as_str()
                || resolved_at.as_str() < request.observed_at.as_str()
            {
                return Err(error(
                    "startup ownership timestamps regress the applied transition lifecycle",
                ));
            }
            validate_startup_actor_event(
                &request.protocol_actor,
                &request.protocol_event,
                &intent.target_authoritative_participant_id,
                claim_id,
                claimant_attempt_id,
            )?;
            let evidence_id = startup_protocol_event_id(&request.protocol_event).to_owned();
            let current_authority =
                current_authority_from_v3_root(&current, &intent.orchestration_session_id)?.clone();
            let current_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&authority_hash_input(&current_authority))
                    .map_err(protocol_error)?,
            };
            match startup_ownership.as_ref() {
                HostSessionStartupOwnershipApplicationV1::Accepted {
                    evidence_id: existing,
                    result_ref,
                    ..
                } => {
                    if existing != &evidence_id {
                        return Err(error(
                            "startup ownership evidence conflicts with the committed result",
                        ));
                    }
                    let result_value = StartupOwnershipResultHashInputV1 {
                        schema_version: 1,
                        evidence: HostStartupOwnershipEvidenceV1 {
                            schema_version: 1,
                            evidence_id: evidence_id.clone(),
                            authority_store_id: current.authority_store_id.clone(),
                            orchestration_session_id: intent.orchestration_session_id.clone(),
                            intent_id: intent.intent_id.clone(),
                            claim_id: claim_id.clone(),
                            claimant_attempt_id: claimant_attempt_id.clone(),
                            run_id: intent.run_id.clone(),
                            application_result_ref: application_result_ref.clone(),
                            expected_authority_revision: 1,
                            active_authoritative_participant_id: intent
                                .target_authoritative_participant_id
                                .clone(),
                            protocol_actor: request.protocol_actor.clone(),
                            protocol_event: request.protocol_event.clone(),
                            observed_at: request.observed_at.clone(),
                        },
                        outcome: StartupOwnershipOutcomeV1::Accepted,
                        resolved_at: resolved_at.clone(),
                    };
                    verify_startup_ownership_join(self, &current, result_ref, &result_value)?;
                    workspace.revalidate().map_err(protocol_error)?;
                    return Ok(StartupOwnershipResolutionOutcomeV1::JoinedStart(
                        intent.clone(),
                    ));
                }
                HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                    evidence_id: existing,
                    result_ref,
                    authority_revision_after,
                    resulting_posture,
                    ..
                } => {
                    if existing != &evidence_id {
                        return Err(error(
                            "startup ownership evidence conflicts with the committed result",
                        ));
                    }
                    let journal = current
                        .application_journal
                        .get(&intent.intent_id)
                        .ok_or_else(|| error("Start application journal disappeared"))?;
                    let startup_terminal = journal
                        .startup_terminal_application
                        .as_ref()
                        .ok_or_else(|| error("Start startup terminal journal disappeared"))?;
                    let reason = startup_terminal_reason(&request.protocol_event)?
                        .ok_or_else(|| error("startup protocol event does not terminalize"))?;
                    let result_value = StartupOwnershipResultHashInputV1 {
                        schema_version: 1,
                        evidence: HostStartupOwnershipEvidenceV1 {
                            schema_version: 1,
                            evidence_id: evidence_id.clone(),
                            authority_store_id: current.authority_store_id.clone(),
                            orchestration_session_id: intent.orchestration_session_id.clone(),
                            intent_id: intent.intent_id.clone(),
                            claim_id: claim_id.clone(),
                            claimant_attempt_id: claimant_attempt_id.clone(),
                            run_id: intent.run_id.clone(),
                            application_result_ref: application_result_ref.clone(),
                            expected_authority_revision: 1,
                            active_authoritative_participant_id: intent
                                .target_authoritative_participant_id
                                .clone(),
                            protocol_actor: request.protocol_actor.clone(),
                            protocol_event: request.protocol_event.clone(),
                            observed_at: request.observed_at.clone(),
                        },
                        outcome: StartupOwnershipOutcomeV1::TerminalReconciled {
                            reason,
                            authority_revision_after: *authority_revision_after,
                            resulting_posture: *resulting_posture,
                            authority_record_commitment: startup_terminal
                                .authority_record_commitment_after
                                .clone(),
                        },
                        resolved_at: resolved_at.clone(),
                    };
                    verify_startup_ownership_join(self, &current, result_ref, &result_value)?;
                    workspace.revalidate().map_err(protocol_error)?;
                    return Ok(StartupOwnershipResolutionOutcomeV1::JoinedStart(
                        intent.clone(),
                    ));
                }
                HostSessionStartupOwnershipApplicationV1::Pending {
                    expected_run_id,
                    expected_authority_revision,
                    expected_active_authoritative_participant_id,
                } => {
                    if expected_run_id != &intent.run_id
                        || *expected_authority_revision != 1
                        || expected_active_authoritative_participant_id
                            != &intent.target_authoritative_participant_id
                    {
                        return Err(error(
                            "startup ownership resolution does not match the applied Start authority",
                        ));
                    }
                    let initial_authority = DurableSessionAuthorityV1 {
                        schema_version: 1,
                        orchestration_session_id: intent.orchestration_session_id.clone(),
                        shell_trace_session_id: intent.shell_trace_session_id.clone(),
                        authority_revision: authority_revision_after,
                        origin: DurableSessionAuthorityOriginV1::StartIntent {
                            intent_id: intent.intent_id.clone(),
                            issuer_request_id: intent.issuer_request_id.clone(),
                            payload_commitment: intent.payload_commitment.clone(),
                        },
                        authoritative_participant_lineage: intent
                            .resulting_authoritative_lineage
                            .clone(),
                        active_authoritative_participant_id: Some(
                            intent.target_authoritative_participant_id.clone(),
                        ),
                        workspace_binding: intent.workspace_binding.clone(),
                        world_binding: intent.world_binding.clone(),
                        host_attach_contract_ref: Some(intent.host_attach_contract_ref.clone()),
                        retained_worker_refs: Vec::new(),
                        internal_resume_handle_refs: Vec::new(),
                        lifecycle_posture: HostSessionPostureV1::ActiveAttached,
                        current_policy_ref: current_authority.current_policy_ref.clone(),
                        current_policy_revision: current_authority.current_policy_revision.clone(),
                        updated_at: applied_at.clone(),
                    };
                    verify_retained_registration_descendant_v3(
                        &current,
                        &initial_authority,
                        &current_authority,
                        &persisted_authority_record_commitment,
                    )?;
                }
                HostSessionStartupOwnershipApplicationV1::NotApplicable => {
                    return Err(error(
                        "startup ownership resolution cannot target NotApplicable state",
                    ))
                }
            }

            let result_value = StartupOwnershipResultHashInputV1 {
                schema_version: 1,
                evidence: HostStartupOwnershipEvidenceV1 {
                    schema_version: 1,
                    evidence_id: evidence_id.clone(),
                    authority_store_id: current.authority_store_id.clone(),
                    orchestration_session_id: intent.orchestration_session_id.clone(),
                    intent_id: intent.intent_id.clone(),
                    claim_id: claim_id.clone(),
                    claimant_attempt_id: claimant_attempt_id.clone(),
                    run_id: intent.run_id.clone(),
                    application_result_ref: application_result_ref.clone(),
                    expected_authority_revision: 1,
                    active_authoritative_participant_id: intent
                        .target_authoritative_participant_id
                        .clone(),
                    protocol_actor: request.protocol_actor.clone(),
                    protocol_event: request.protocol_event.clone(),
                    observed_at: request.observed_at.clone(),
                },
                outcome: match startup_terminal_reason(&request.protocol_event)? {
                    None => StartupOwnershipOutcomeV1::Accepted,
                    Some(reason) => {
                        let mut terminal_authority = current_authority.clone();
                        terminal_authority.authority_revision = next_revision(
                            current_authority.authority_revision,
                            "durable authority",
                        )?;
                        terminal_authority.lifecycle_posture = HostSessionPostureV1::Terminal;
                        terminal_authority.updated_at = resolved_at.clone();
                        let authority_record_commitment =
                            AuthorityObjectCommitmentV1::CanonicalSha256 {
                                digest_hex: canonical_sha256(&authority_hash_input(
                                    &terminal_authority,
                                ))
                                .map_err(protocol_error)?,
                            };
                        StartupOwnershipOutcomeV1::TerminalReconciled {
                            reason,
                            authority_revision_after: terminal_authority.authority_revision,
                            resulting_posture: HostSessionPostureV1::Terminal,
                            authority_record_commitment,
                        }
                    }
                },
                resolved_at: resolved_at.clone(),
            };
            let result_bytes = canonical_object_bytes(
                AuthorityObjectKindV1::StartupOwnershipResult,
                CanonicalObjectHashInputV1::StartupOwnershipResult(&result_value),
            )
            .map_err(protocol_error)?;
            let result = self.prepare_generated_successor_object(
                &current,
                AuthorityObjectKindV1::StartupOwnershipResult,
                &result_bytes,
                None,
            )?;

            let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
            let mut proposed = current.clone();
            proposed.root_revision = next_revision(current.root_revision, "authority root")?;
            let next = proposed
                .transition_intent_map
                .get_mut(&intent.intent_id)
                .ok_or_else(|| error("transition intent disappeared during startup ownership"))?;
            next.intent_revision = next_intent_revision;
            next.updated_at = resolved_at.clone();
            match &result_value.outcome {
                StartupOwnershipOutcomeV1::Accepted => {
                    let HostSessionTransitionIntentStateV2::Applied {
                        startup_ownership, ..
                    } = &mut next.state
                    else {
                        return Err(error(
                            "applied Start intent disappeared during startup ownership",
                        ));
                    };
                    *startup_ownership =
                        Box::new(HostSessionStartupOwnershipApplicationV1::Accepted {
                            evidence_id: evidence_id.clone(),
                            result_ref: result.reference.clone(),
                            authority_revision: current_authority.authority_revision,
                            accepted_at: resolved_at.clone(),
                        });
                }
                StartupOwnershipOutcomeV1::TerminalReconciled {
                    authority_revision_after: terminal_authority_revision_after,
                    resulting_posture,
                    authority_record_commitment,
                    ..
                } => {
                    let input_acceptance_ref = match &next.input_handoff {
                        HostSessionTransitionInputHandoffV1::NotApplicable => None,
                        HostSessionTransitionInputHandoffV1::Pending { .. } => None,
                        HostSessionTransitionInputHandoffV1::Accepted {
                            acceptance_ref, ..
                        } => Some(acceptance_ref.clone()),
                        HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                            ..
                        } => {
                            return Err(error(
                                "Start startup terminalization conflicts with terminal input state",
                            ))
                        }
                    };
                    let terminal_value = TerminalHandoffHashInputV2 {
                        schema_version: 2,
                        intent_id: intent.intent_id.clone(),
                        run_id: intent.run_id.clone(),
                        payload_commitment: intent.payload_commitment.clone(),
                        terminal_state: TerminalHandoffStateV1::Applied,
                        application_result_ref: Some(application_result_ref.clone()),
                        input_acceptance_ref,
                        startup_ownership_result_ref: Some(result.reference.clone()),
                        post_turn_completion_ref: None,
                        post_turn_application_result_ref: None,
                        recorded_at: resolved_at.clone(),
                    };
                    let terminal_bytes = canonical_object_bytes(
                        AuthorityObjectKindV1::TerminalHandoff,
                        CanonicalObjectHashInputV1::TerminalHandoffV2(&terminal_value),
                    )
                    .map_err(protocol_error)?;
                    let terminal = self.prepare_generated_successor_object(
                        &current,
                        AuthorityObjectKindV1::TerminalHandoff,
                        &terminal_bytes,
                        None,
                    )?;
                    if let HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id } =
                        &next.input_handoff
                    {
                        next.input_handoff =
                            HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                                input_ref: input_ref.clone(),
                                run_id: run_id.clone(),
                                terminal_handoff_ref: terminal.reference.clone(),
                                terminal_at: resolved_at.clone(),
                            };
                    }
                    let HostSessionTransitionIntentStateV2::Applied {
                        startup_ownership, ..
                    } = &mut next.state
                    else {
                        return Err(error(
                            "applied Start intent disappeared during startup ownership",
                        ));
                    };
                    *startup_ownership = Box::new(
                        HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                            evidence_id: evidence_id.clone(),
                            result_ref: result.reference.clone(),
                            authority_revision_before: current_authority.authority_revision,
                            authority_revision_after: *terminal_authority_revision_after,
                            resulting_posture: *resulting_posture,
                            reconciled_at: resolved_at.clone(),
                        },
                    );
                    next.transport_payload_state =
                        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                            terminal_handoff_ref: terminal.reference.clone(),
                        };

                    let mut terminal_authority = current_authority.clone();
                    terminal_authority.authority_revision = *terminal_authority_revision_after;
                    terminal_authority.lifecycle_posture = *resulting_posture;
                    terminal_authority.updated_at = resolved_at.clone();
                    proposed.session_namespace_map.insert(
                        intent.orchestration_session_id.clone(),
                        SessionNamespaceRecordV1::Authority(Box::new(terminal_authority)),
                    );
                    let journal = proposed
                        .application_journal
                        .get_mut(&intent.intent_id)
                        .ok_or_else(|| error("Start application journal disappeared"))?;
                    journal.startup_terminal_application = Some(
                        super::store_schema::StartupOwnershipTerminalApplicationJournalV1 {
                            schema_version: 1,
                            startup_ownership_result_ref: result.reference.clone(),
                            evidence_id: evidence_id.clone(),
                            authority_revision_before: current_authority.authority_revision,
                            authority_record_commitment_before: current_commitment.clone(),
                            authority_revision_after: *terminal_authority_revision_after,
                            resulting_posture: *resulting_posture,
                            authority_record_commitment_after: authority_record_commitment.clone(),
                            applied_at: resolved_at.clone(),
                        },
                    );
                    insert_present_index_v3(
                        &mut proposed,
                        &terminal.reference,
                        AuthorityObjectKindV1::TerminalHandoff,
                        terminal.byte_length,
                    )?;
                    proposed
                        .object_index
                        .get_mut(&intent.transport_payload_ref.ref_id)
                        .ok_or_else(|| error("transport payload index disappeared"))?
                        .storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
                        terminal_handoff_ref: terminal.reference.clone(),
                    };
                }
            }
            insert_present_index_v3(
                &mut proposed,
                &result.reference,
                AuthorityObjectKindV1::StartupOwnershipResult,
                result.byte_length,
            )?;
            proposed.validate().map_err(protocol_error)?;
            workspace.revalidate().map_err(protocol_error)?;
            store::commit_v3_root_exact_current_opened(
                self.trusted_root(),
                &current,
                &proposed,
                || {
                    workspace
                        .revalidate()
                        .map_err(|_| store::BootstrapError::transition_guard())
                },
            )
            .map_err(protocol_error)?;
            workspace.revalidate().map_err(protocol_error)?;
            return Ok(StartupOwnershipResolutionOutcomeV1::ResolvedStart(
                proposed.transition_intent_map[&request.intent_id].clone(),
            ));
        }
        let intent = exact_successor_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "startup ownership resolution",
        )?;
        if intent.mode != HostSessionTransitionModeV1::Attach {
            return Err(error(
                "startup ownership resolution currently requires an applied Attach successor",
            ));
        }
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let (
            claim_id,
            claimant_attempt_id,
            authority_revision_before,
            authority_revision_after,
            persisted_authority_record_commitment,
            application_result_ref,
            startup_ownership,
            post_turn,
            applied_at,
        ) = match &intent.state {
            HostSessionTransitionIntentStateV3::Applied {
                claim_id,
                claimant_attempt_id,
                authority_revision_before,
                authority_revision_after,
                authority_record_commitment,
                application_result_ref,
                startup_ownership,
                post_turn,
                applied_at,
                ..
            } => (
                claim_id,
                claimant_attempt_id,
                *authority_revision_before,
                *authority_revision_after,
                authority_record_commitment.clone(),
                application_result_ref,
                startup_ownership,
                post_turn,
                applied_at,
            ),
            _ => {
                return Err(error(
                    "startup ownership resolution requires an applied Attach successor",
                ))
            }
        };
        if post_turn.as_ref() != &HostSessionPostTurnApplicationV2::NotApplicable {
            return Err(error(
                "Attach startup ownership cannot resolve alongside post-turn work",
            ));
        }
        if request.observed_at.as_str() < applied_at.as_str()
            || resolved_at.as_str() < request.observed_at.as_str()
        {
            return Err(error(
                "startup ownership timestamps regress the applied successor lifecycle",
            ));
        }
        validate_startup_actor_event(
            &request.protocol_actor,
            &request.protocol_event,
            &intent.target_authoritative_participant_id,
            claim_id,
            claimant_attempt_id,
        )?;
        let evidence_id = startup_protocol_event_id(&request.protocol_event).to_owned();
        let Some(SessionNamespaceRecordV1::Authority(current_authority)) = current
            .session_namespace_map
            .get(&intent.orchestration_session_id)
        else {
            return Err(error(
                "startup ownership resolution has no durable authority",
            ));
        };
        let current_authority = current_authority.as_ref().clone();
        let current_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&current_authority))
                .map_err(protocol_error)?,
        };
        match startup_ownership.as_ref() {
            HostSessionStartupOwnershipApplicationV1::Accepted {
                evidence_id: existing,
                result_ref,
                ..
            } => {
                if existing != &evidence_id {
                    return Err(error(
                        "startup ownership evidence conflicts with the committed result",
                    ));
                }
                let result_value = StartupOwnershipResultHashInputV1 {
                    schema_version: 1,
                    evidence: HostStartupOwnershipEvidenceV1 {
                        schema_version: 1,
                        evidence_id: evidence_id.clone(),
                        authority_store_id: current.authority_store_id.clone(),
                        orchestration_session_id: intent.orchestration_session_id.clone(),
                        intent_id: intent.intent_id.clone(),
                        claim_id: claim_id.clone(),
                        claimant_attempt_id: claimant_attempt_id.clone(),
                        run_id: intent.run_id.clone(),
                        application_result_ref: application_result_ref.clone(),
                        expected_authority_revision: authority_revision_after,
                        active_authoritative_participant_id: intent
                            .target_authoritative_participant_id
                            .clone(),
                        protocol_actor: request.protocol_actor.clone(),
                        protocol_event: request.protocol_event.clone(),
                        observed_at: request.observed_at.clone(),
                    },
                    outcome: StartupOwnershipOutcomeV1::Accepted,
                    resolved_at: resolved_at.clone(),
                };
                verify_startup_ownership_join(self, &current, result_ref, &result_value)?;
                workspace.revalidate().map_err(protocol_error)?;
                return Ok(StartupOwnershipResolutionOutcomeV1::JoinedSuccessor(intent));
            }
            HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                evidence_id: existing,
                result_ref,
                authority_revision_after: terminal_authority_revision_after,
                resulting_posture,
                ..
            } => {
                if existing != &evidence_id {
                    return Err(error(
                        "startup ownership evidence conflicts with the committed result",
                    ));
                }
                let journal = current
                    .successor_application_journal
                    .get(&intent.intent_id)
                    .ok_or_else(|| error("successor application journal disappeared"))?;
                let startup_terminal = journal
                    .startup_terminal_application
                    .as_ref()
                    .ok_or_else(|| error("successor startup terminal journal disappeared"))?;
                let reason = startup_terminal_reason(&request.protocol_event)?
                    .ok_or_else(|| error("startup protocol event does not terminalize"))?;
                let result_value = StartupOwnershipResultHashInputV1 {
                    schema_version: 1,
                    evidence: HostStartupOwnershipEvidenceV1 {
                        schema_version: 1,
                        evidence_id: evidence_id.clone(),
                        authority_store_id: current.authority_store_id.clone(),
                        orchestration_session_id: intent.orchestration_session_id.clone(),
                        intent_id: intent.intent_id.clone(),
                        claim_id: claim_id.clone(),
                        claimant_attempt_id: claimant_attempt_id.clone(),
                        run_id: intent.run_id.clone(),
                        application_result_ref: application_result_ref.clone(),
                        expected_authority_revision: authority_revision_after,
                        active_authoritative_participant_id: intent
                            .target_authoritative_participant_id
                            .clone(),
                        protocol_actor: request.protocol_actor.clone(),
                        protocol_event: request.protocol_event.clone(),
                        observed_at: request.observed_at.clone(),
                    },
                    outcome: StartupOwnershipOutcomeV1::TerminalReconciled {
                        reason,
                        authority_revision_after: *terminal_authority_revision_after,
                        resulting_posture: *resulting_posture,
                        authority_record_commitment: startup_terminal
                            .authority_record_commitment_after
                            .clone(),
                    },
                    resolved_at: resolved_at.clone(),
                };
                verify_startup_ownership_join(self, &current, result_ref, &result_value)?;
                workspace.revalidate().map_err(protocol_error)?;
                return Ok(StartupOwnershipResolutionOutcomeV1::JoinedSuccessor(intent));
            }
            HostSessionStartupOwnershipApplicationV1::Pending {
                expected_run_id,
                expected_authority_revision,
                expected_active_authoritative_participant_id,
            } => {
                if expected_run_id != &intent.run_id
                    || *expected_authority_revision != authority_revision_after
                    || expected_active_authoritative_participant_id
                        != &intent.target_authoritative_participant_id
                    || current_authority.authority_revision != authority_revision_after
                    || current_authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
                    || current_authority
                        .active_authoritative_participant_id
                        .as_deref()
                        != Some(intent.target_authoritative_participant_id.as_str())
                {
                    return Err(error(
                        "startup ownership resolution does not match the applied Attach authority",
                    ));
                }
            }
            HostSessionStartupOwnershipApplicationV1::NotApplicable => {
                return Err(error(
                    "startup ownership resolution cannot target NotApplicable state",
                ))
            }
        }

        let result_value = StartupOwnershipResultHashInputV1 {
            schema_version: 1,
            evidence: HostStartupOwnershipEvidenceV1 {
                schema_version: 1,
                evidence_id: evidence_id.clone(),
                authority_store_id: current.authority_store_id.clone(),
                orchestration_session_id: intent.orchestration_session_id.clone(),
                intent_id: intent.intent_id.clone(),
                claim_id: claim_id.clone(),
                claimant_attempt_id: claimant_attempt_id.clone(),
                run_id: intent.run_id.clone(),
                application_result_ref: application_result_ref.clone(),
                expected_authority_revision: authority_revision_after,
                active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
                protocol_actor: request.protocol_actor.clone(),
                protocol_event: request.protocol_event.clone(),
                observed_at: request.observed_at.clone(),
            },
            outcome: match startup_terminal_reason(&request.protocol_event)? {
                None => StartupOwnershipOutcomeV1::Accepted,
                Some(reason) => {
                    let mut terminal_authority = current_authority.clone();
                    terminal_authority.authority_revision =
                        next_revision(current_authority.authority_revision, "durable authority")?;
                    terminal_authority.lifecycle_posture = HostSessionPostureV1::DetachedReconciled;
                    terminal_authority.updated_at = resolved_at.clone();
                    let authority_record_commitment =
                        AuthorityObjectCommitmentV1::CanonicalSha256 {
                            digest_hex: canonical_sha256(&authority_hash_input(
                                &terminal_authority,
                            ))
                            .map_err(protocol_error)?,
                        };
                    StartupOwnershipOutcomeV1::TerminalReconciled {
                        reason,
                        authority_revision_after: terminal_authority.authority_revision,
                        resulting_posture: HostSessionPostureV1::DetachedReconciled,
                        authority_record_commitment,
                    }
                }
            },
            resolved_at: resolved_at.clone(),
        };
        let result_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::StartupOwnershipResult,
            CanonicalObjectHashInputV1::StartupOwnershipResult(&result_value),
        )
        .map_err(protocol_error)?;
        let result = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::StartupOwnershipResult,
            &result_bytes,
            None,
        )?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .successor_transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during startup ownership"))?;
        next.intent_revision = next_intent_revision;
        next.updated_at = resolved_at.clone();
        match &result_value.outcome {
            StartupOwnershipOutcomeV1::Accepted => {
                next.state = HostSessionTransitionIntentStateV3::Applied {
                    claim_id: claim_id.clone(),
                    claimant_attempt_id: claimant_attempt_id.clone(),
                    authority_revision_before,
                    authority_revision_after,
                    active_authoritative_participant_id: intent
                        .target_authoritative_participant_id
                        .clone(),
                    resulting_posture: HostSessionPostureV1::ActiveAttached,
                    authority_record_commitment: persisted_authority_record_commitment.clone(),
                    application_result_ref: application_result_ref.clone(),
                    startup_ownership: Box::new(
                        HostSessionStartupOwnershipApplicationV1::Accepted {
                            evidence_id,
                            result_ref: result.reference.clone(),
                            authority_revision: current_authority.authority_revision,
                            accepted_at: resolved_at.clone(),
                        },
                    ),
                    post_turn: Box::new(HostSessionPostTurnApplicationV2::NotApplicable),
                    applied_at: applied_at.clone(),
                };
            }
            StartupOwnershipOutcomeV1::TerminalReconciled {
                authority_revision_after: terminal_authority_revision_after,
                resulting_posture,
                authority_record_commitment,
                ..
            } => {
                if !matches!(
                    next.input_handoff,
                    HostSessionTransitionInputHandoffV1::NotApplicable
                ) {
                    return Err(error(
                        "Attach startup ownership terminalization cannot carry transition input",
                    ));
                }
                let terminal_value = TerminalHandoffHashInputV2 {
                    schema_version: 2,
                    intent_id: intent.intent_id.clone(),
                    run_id: intent.run_id.clone(),
                    payload_commitment: intent.payload_commitment.clone(),
                    terminal_state: TerminalHandoffStateV1::Applied,
                    application_result_ref: Some(application_result_ref.clone()),
                    input_acceptance_ref: None,
                    startup_ownership_result_ref: Some(result.reference.clone()),
                    post_turn_completion_ref: None,
                    post_turn_application_result_ref: None,
                    recorded_at: resolved_at.clone(),
                };
                let terminal_bytes = canonical_object_bytes(
                    AuthorityObjectKindV1::TerminalHandoff,
                    CanonicalObjectHashInputV1::TerminalHandoffV2(&terminal_value),
                )
                .map_err(protocol_error)?;
                let terminal = self.prepare_generated_successor_object(
                    &current,
                    AuthorityObjectKindV1::TerminalHandoff,
                    &terminal_bytes,
                    None,
                )?;
                next.transport_payload_state =
                    HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                        terminal_handoff_ref: terminal.reference.clone(),
                    };

                let mut terminal_authority = current_authority.clone();
                terminal_authority.authority_revision = *terminal_authority_revision_after;
                terminal_authority.lifecycle_posture = *resulting_posture;
                terminal_authority.updated_at = resolved_at.clone();
                proposed.session_namespace_map.insert(
                    intent.orchestration_session_id.clone(),
                    SessionNamespaceRecordV1::Authority(Box::new(terminal_authority)),
                );
                let journal = proposed
                    .successor_application_journal
                    .get_mut(&intent.intent_id)
                    .ok_or_else(|| error("successor application journal disappeared"))?;
                journal.startup_terminal_application = Some(
                    super::store_schema::StartupOwnershipTerminalApplicationJournalV1 {
                        schema_version: 1,
                        startup_ownership_result_ref: result.reference.clone(),
                        evidence_id: evidence_id.clone(),
                        authority_revision_before: current_authority.authority_revision,
                        authority_record_commitment_before: current_commitment.clone(),
                        authority_revision_after: *terminal_authority_revision_after,
                        resulting_posture: *resulting_posture,
                        authority_record_commitment_after: authority_record_commitment.clone(),
                        applied_at: resolved_at.clone(),
                    },
                );
                next.state = HostSessionTransitionIntentStateV3::Applied {
                    claim_id: claim_id.clone(),
                    claimant_attempt_id: claimant_attempt_id.clone(),
                    authority_revision_before,
                    authority_revision_after,
                    active_authoritative_participant_id: intent
                        .target_authoritative_participant_id
                        .clone(),
                    resulting_posture: HostSessionPostureV1::ActiveAttached,
                    authority_record_commitment: persisted_authority_record_commitment.clone(),
                    application_result_ref: application_result_ref.clone(),
                    startup_ownership: Box::new(
                        HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                            evidence_id,
                            result_ref: result.reference.clone(),
                            authority_revision_before: current_authority.authority_revision,
                            authority_revision_after: *terminal_authority_revision_after,
                            resulting_posture: *resulting_posture,
                            reconciled_at: resolved_at.clone(),
                        },
                    ),
                    post_turn: Box::new(HostSessionPostTurnApplicationV2::NotApplicable),
                    applied_at: applied_at.clone(),
                };
                insert_present_index_v3(
                    &mut proposed,
                    &terminal.reference,
                    AuthorityObjectKindV1::TerminalHandoff,
                    terminal.byte_length,
                )?;
                proposed
                    .object_index
                    .get_mut(&intent.transport_payload_ref.ref_id)
                    .ok_or_else(|| error("transport payload index disappeared"))?
                    .storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
                    terminal_handoff_ref: terminal.reference.clone(),
                };
            }
        }
        insert_present_index_v3(
            &mut proposed,
            &result.reference,
            AuthorityObjectKindV1::StartupOwnershipResult,
            result.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(StartupOwnershipResolutionOutcomeV1::ResolvedSuccessor(
            proposed.successor_transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn accept_transition_input(
        &self,
        request: &AcceptTransitionInputRequestV1,
    ) -> Result<InputAcceptanceOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12b_root()?;
        let intent = exact_successor_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "input acceptance",
        )?;
        if intent.mode != HostSessionTransitionModeV1::ResumeOneTurn {
            return Err(error(
                "input acceptance currently requires an applied ResumeOneTurn successor",
            ));
        }
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let (authority_revision_after, startup_ownership, post_turn, applied_at) =
            match &intent.state {
                HostSessionTransitionIntentStateV3::Applied {
                    authority_revision_after,
                    startup_ownership,
                    post_turn,
                    applied_at,
                    ..
                } => (
                    *authority_revision_after,
                    startup_ownership,
                    post_turn,
                    applied_at,
                ),
                _ => {
                    return Err(error(
                        "input acceptance currently requires an applied ResumeOneTurn successor",
                    ))
                }
            };
        if startup_ownership.as_ref() != &HostSessionStartupOwnershipApplicationV1::NotApplicable {
            return Err(error(
                "ResumeOneTurn input acceptance cannot carry startup ownership state",
            ));
        }
        let current_authority =
            current_authority_from_v3_root(&current, &intent.orchestration_session_id)?.clone();
        let Some(expected_input_ref) = intent.transition_input_ref.as_ref() else {
            return Err(error(
                "ResumeOneTurn input acceptance requires a committed transition input",
            ));
        };
        let expected_acceptance = |input_ref: &AuthorityObjectRefV1| InputAcceptanceHashInputV1 {
            schema_version: 1,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            input_ref: input_ref.clone(),
            accepting_participant_id: request.accepting_participant_id.clone(),
            accepted_at: request.accepted_at.clone(),
        };
        match &intent.input_handoff {
            HostSessionTransitionInputHandoffV1::Accepted {
                input_ref,
                run_id,
                acceptance_ref,
                ..
            } => {
                if input_ref != expected_input_ref || run_id != &intent.run_id {
                    return Err(error(
                        "accepted input handoff conflicts with the applied ResumeOneTurn intent",
                    ));
                }
                let bytes = store::read_typed_object_v2_or_v3_opened(
                    self.trusted_root(),
                    current.root_revision,
                    acceptance_ref,
                    None,
                )
                .map_err(protocol_error)?;
                let committed: InputAcceptanceHashInputV1 =
                    canonical_json::from_slice(&bytes).map_err(protocol_error)?;
                if committed != expected_acceptance(input_ref) {
                    return Err(error(
                        "input acceptance evidence conflicts with the committed handoff",
                    ));
                }
                workspace.revalidate().map_err(protocol_error)?;
                return Ok(InputAcceptanceOutcomeV1::Joined(intent));
            }
            HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance { .. } => {
                return Err(error(
                    "terminalized ResumeOneTurn input cannot be accepted after terminal handoff",
                ))
            }
            HostSessionTransitionInputHandoffV1::NotApplicable => {
                return Err(error(
                    "ResumeOneTurn input acceptance requires a pending transition input",
                ))
            }
            HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id } => {
                if input_ref != expected_input_ref || run_id != &intent.run_id {
                    return Err(error(
                        "pending input handoff conflicts with the applied ResumeOneTurn intent",
                    ));
                }
            }
        }
        if request.accepting_participant_id != intent.target_authoritative_participant_id {
            return Err(error(
                "input acceptance participant does not match the applied ResumeOneTurn target",
            ));
        }
        if request.accepted_at.as_str() < applied_at.as_str()
            || request.accepted_at.as_str() < intent.updated_at.as_str()
        {
            return Err(error(
                "input acceptance time regresses the applied ResumeOneTurn lifecycle",
            ));
        }
        if current_authority.authority_revision != authority_revision_after
            || current_authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
            || current_authority
                .active_authoritative_participant_id
                .as_deref()
                != Some(intent.target_authoritative_participant_id.as_str())
            || !matches!(
                post_turn.as_ref(),
                HostSessionPostTurnApplicationV2::Pending {
                    expected_run_id,
                    expected_authority_revision,
                } if expected_run_id == &intent.run_id
                    && *expected_authority_revision == authority_revision_after
            )
        {
            return Err(error(
                "input acceptance no longer matches the current applied ResumeOneTurn authority",
            ));
        }

        let acceptance_value = expected_acceptance(expected_input_ref);
        let acceptance_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::InputAcceptance,
            CanonicalObjectHashInputV1::InputAcceptance(&acceptance_value),
        )
        .map_err(protocol_error)?;
        let acceptance = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::InputAcceptance,
            &acceptance_bytes,
            None,
        )?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .successor_transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during input acceptance"))?;
        next.intent_revision = next_intent_revision;
        next.input_handoff = HostSessionTransitionInputHandoffV1::Accepted {
            input_ref: expected_input_ref.clone(),
            run_id: intent.run_id.clone(),
            acceptance_ref: acceptance.reference.clone(),
            accepted_at: request.accepted_at.clone(),
        };
        next.updated_at = request.accepted_at.clone();
        insert_present_index_v3(
            &mut proposed,
            &acceptance.reference,
            AuthorityObjectKindV1::InputAcceptance,
            acceptance.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(InputAcceptanceOutcomeV1::Accepted(
            proposed.successor_transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn resolve_post_turn(
        &self,
        request: &ResolvePostTurnRequestV1,
    ) -> Result<PostTurnResolutionOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12b_root()?;
        let intent = exact_successor_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "post-turn resolution",
        )?;
        if intent.mode != HostSessionTransitionModeV1::ResumeOneTurn {
            return Err(error(
                "post-turn resolution currently requires an applied ResumeOneTurn successor",
            ));
        }
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let (
            claim_id,
            claimant_attempt_id,
            _authority_revision_before,
            authority_revision_after,
            _persisted_authority_record_commitment,
            application_result_ref,
            startup_ownership,
            post_turn,
            applied_at,
        ) =
            match &intent.state {
                HostSessionTransitionIntentStateV3::Applied {
                    claim_id,
                    claimant_attempt_id,
                    authority_revision_before,
                    authority_revision_after,
                    authority_record_commitment,
                    application_result_ref,
                    startup_ownership,
                    post_turn,
                    applied_at,
                    ..
                } => (
                    claim_id,
                    claimant_attempt_id,
                    *authority_revision_before,
                    *authority_revision_after,
                    authority_record_commitment.clone(),
                    application_result_ref,
                    startup_ownership,
                    post_turn,
                    applied_at,
                ),
                _ => return Err(error(
                    "post-turn resolution currently requires an applied ResumeOneTurn successor",
                )),
            };
        if startup_ownership.as_ref() != &HostSessionStartupOwnershipApplicationV1::NotApplicable {
            return Err(error(
                "ResumeOneTurn post-turn resolution cannot carry startup ownership state",
            ));
        }
        let observed_authority_revision =
            post_turn_observed_authority_revision(post_turn.as_ref())?;
        let expected_protocol_event = PostTurnProtocolEventHashInputV1 {
            schema_version: 1,
            authority_store_id: current.authority_store_id.clone(),
            orchestration_session_id: intent.orchestration_session_id.clone(),
            intent_id: intent.intent_id.clone(),
            claim_id: claim_id.clone(),
            claimant_attempt_id: claimant_attempt_id.clone(),
            run_id: intent.run_id.clone(),
            authority_revision_observed: observed_authority_revision,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            acceptance_record_id: request.acceptance_record_id.clone(),
            acceptance_record_revision: request.acceptance_record_revision,
            stream_id: request.stream_id.clone(),
            accepted_work_identity: request.accepted_work_identity.clone(),
            host_transition_correlation: request.host_transition_correlation.clone(),
            protocol_actor: request.protocol_actor.clone(),
            event_id: request.event_id.clone(),
            event_sequence: request.event_sequence,
            kind: request.kind.clone(),
            emitted_at: request.emitted_at.clone(),
        };
        let expected_completion = PostTurnCompletionHashInputV1 {
            schema_version: 1,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            authority_revision_observed: observed_authority_revision,
            acceptance_record_id: request.acceptance_record_id.clone(),
            acceptance_record_revision: request.acceptance_record_revision,
            stream_id: request.stream_id.clone(),
            accepted_work_identity: request.accepted_work_identity.clone(),
            host_transition_correlation: request.host_transition_correlation.clone(),
            terminal_event_id: request.event_id.clone(),
            terminal_event_sequence: request.event_sequence,
            protocol_event_ref: placeholder_ref_for_kind(
                AuthorityObjectKindV1::PostTurnProtocolEvent,
            ),
            outcome: post_turn_completion_outcome(&request.kind),
            completed_at: request.completed_at.clone(),
        };

        match post_turn.as_ref() {
            HostSessionPostTurnApplicationV2::AwaitingObligationCut { completion_ref, .. }
            | HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } => {
                verify_post_turn_completion_join(
                    self,
                    &current,
                    completion_ref.as_ref(),
                    &expected_protocol_event,
                    &expected_completion,
                )?;
                workspace.revalidate().map_err(protocol_error)?;
                return Ok(PostTurnResolutionOutcomeV1::Joined(intent));
            }
            HostSessionPostTurnApplicationV2::NotApplicable => {
                return Err(error(
                    "ResumeOneTurn post-turn resolution cannot target NotApplicable state",
                ))
            }
            HostSessionPostTurnApplicationV2::Pending {
                expected_run_id,
                expected_authority_revision,
            } => {
                if expected_run_id != &intent.run_id
                    || *expected_authority_revision != authority_revision_after
                {
                    return Err(error(
                        "pending ResumeOneTurn post-turn state conflicts with the applied intent",
                    ));
                }
            }
        }

        if request.acceptance_record_id.is_empty()
            || request.acceptance_record_revision == 0
            || request.stream_id.is_empty()
            || request.event_id.is_empty()
            || request.event_sequence == 0
        {
            return Err(error("post-turn evidence identity is invalid"));
        }
        if request.emitted_at.as_str() < applied_at.as_str()
            || request.emitted_at.as_str() < intent.updated_at.as_str()
            || request.completed_at.as_str() < request.emitted_at.as_str()
        {
            return Err(error(
                "post-turn timestamps regress the applied ResumeOneTurn lifecycle",
            ));
        }
        if request.host_transition_correlation.schema_version != 1
            || request.host_transition_correlation.authority_store_id != current.authority_store_id
            || request.host_transition_correlation.orchestration_session_id
                != intent.orchestration_session_id
            || request
                .host_transition_correlation
                .authoritative_participant_id
                != intent.target_authoritative_participant_id
            || request.host_transition_correlation.transition_intent_id != intent.intent_id
            || request
                .host_transition_correlation
                .transition_intent_revision_observed
                != intent.intent_revision
            || request.host_transition_correlation.transition_run_id != intent.run_id
            || request
                .host_transition_correlation
                .transition_payload_commitment
                != opaque_commitment(&intent.payload_commitment)
            || request
                .host_transition_correlation
                .authority_revision_observed
                != observed_authority_revision
        {
            return Err(error(
                "post-turn host-transition correlation does not match the applied ResumeOneTurn intent",
            ));
        }
        let current_authority =
            current_authority_from_v3_root(&current, &intent.orchestration_session_id)?.clone();
        if current_authority.authority_revision != authority_revision_after
            || current_authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
            || current_authority
                .active_authoritative_participant_id
                .as_deref()
                != Some(intent.target_authoritative_participant_id.as_str())
        {
            return Err(error(
                "post-turn resolution no longer matches the current applied ResumeOneTurn authority",
            ));
        }

        let Some(expected_input_ref) = intent.transition_input_ref.as_ref() else {
            return Err(error(
                "ResumeOneTurn post-turn resolution requires a committed transition input",
            ));
        };
        let (input_acceptance_ref, terminalize_pending_input) = match (&request.kind, &intent.input_handoff) {
            (
                HostPostTurnProtocolEventKindV1::ResumableClean
                | HostPostTurnProtocolEventKindV1::TerminalClean,
                HostSessionTransitionInputHandoffV1::Accepted {
                    input_ref,
                    run_id,
                    acceptance_ref,
                    ..
                },
            ) if input_ref == expected_input_ref && run_id == &intent.run_id => {
                if request.protocol_actor
                    != (HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                        participant_id: intent.target_authoritative_participant_id.clone(),
                    })
                {
                    return Err(error(
                        "post-turn actor does not match the applied ResumeOneTurn participant",
                    ));
                }
                (Some(acceptance_ref.clone()), false)
            }
            (
                HostPostTurnProtocolEventKindV1::TerminalFailure {
                    reason: HostPostTurnTerminalReasonV1::ResumeRuntimeCreationRejected,
                },
                HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id },
            ) if input_ref == expected_input_ref && run_id == &intent.run_id => {
                if request.protocol_actor
                    != (HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
                        claim_id: claim_id.clone(),
                        claimant_attempt_id: claimant_attempt_id.clone(),
                    })
                {
                    return Err(error(
                        "pre-acceptance runtime rejection must be reported by the exact applied claimant",
                    ));
                }
                (None, true)
            }
            (
                HostPostTurnProtocolEventKindV1::TerminalFailure {
                    reason: HostPostTurnTerminalReasonV1::TargetFailedBeforeInputAcceptance,
                },
                HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id },
            ) if input_ref == expected_input_ref && run_id == &intent.run_id => {
                if request.protocol_actor
                    != (HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                        participant_id: intent.target_authoritative_participant_id.clone(),
                    })
                {
                    return Err(error(
                        "pre-acceptance target failure must be reported by the exact applied participant",
                    ));
                }
                (None, true)
            }
            (
                HostPostTurnProtocolEventKindV1::TerminalFailure {
                    reason: HostPostTurnTerminalReasonV1::TargetFailedAfterInputAcceptance,
                },
                HostSessionTransitionInputHandoffV1::Accepted {
                    input_ref,
                    run_id,
                    acceptance_ref,
                    ..
                },
            ) if input_ref == expected_input_ref && run_id == &intent.run_id => {
                if request.protocol_actor
                    != (HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                        participant_id: intent.target_authoritative_participant_id.clone(),
                    })
                {
                    return Err(error(
                        "post-acceptance target failure must be reported by the exact applied participant",
                    ));
                }
                (Some(acceptance_ref.clone()), false)
            }
            _ => {
                return Err(error(
                    "post-turn actor, reason, or input state does not match the applied ResumeOneTurn protocol",
                ))
            }
        };

        let protocol_event_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::PostTurnProtocolEvent,
            CanonicalObjectHashInputV1::PostTurnProtocolEvent(&expected_protocol_event),
        )
        .map_err(protocol_error)?;
        let protocol_event = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::PostTurnProtocolEvent,
            &protocol_event_bytes,
            None,
        )?;
        let completion_value = PostTurnCompletionHashInputV1 {
            protocol_event_ref: protocol_event.reference.clone(),
            ..expected_completion
        };
        let completion_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::PostTurnCompletion,
            CanonicalObjectHashInputV1::PostTurnCompletion(&completion_value),
        )
        .map_err(protocol_error)?;
        let completion = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::PostTurnCompletion,
            &completion_bytes,
            None,
        )?;

        let next_intent_revision = next_revision(intent.intent_revision, "transition intent")?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .successor_transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during post-turn resolution"))?;
        next.intent_revision = next_intent_revision;
        next.updated_at = request.completed_at.clone();

        match request.kind {
            HostPostTurnProtocolEventKindV1::ResumableClean => {
                let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &mut next.state
                else {
                    return Err(error(
                        "applied ResumeOneTurn intent disappeared during post-turn resolution",
                    ));
                };
                *post_turn = Box::new(HostSessionPostTurnApplicationV2::AwaitingObligationCut {
                    completion_ref: Box::new(completion.reference.clone()),
                    expected_run_id: intent.run_id.clone(),
                    expected_authority_revision: authority_revision_after,
                    acceptance_record_id: request.acceptance_record_id.clone(),
                    acceptance_record_revision: request.acceptance_record_revision,
                    stream_id: request.stream_id.clone(),
                    accepted_work_identity: request.accepted_work_identity.clone(),
                    host_transition_correlation: Box::new(
                        request.host_transition_correlation.clone(),
                    ),
                    required_terminal_event_id: request.event_id.clone(),
                    required_terminal_event_sequence: request.event_sequence,
                    recorded_at: request.completed_at.clone(),
                });
                insert_present_index_v3(
                    &mut proposed,
                    &protocol_event.reference,
                    AuthorityObjectKindV1::PostTurnProtocolEvent,
                    protocol_event.byte_length,
                )?;
                insert_present_index_v3(
                    &mut proposed,
                    &completion.reference,
                    AuthorityObjectKindV1::PostTurnCompletion,
                    completion.byte_length,
                )?;
                proposed.validate().map_err(protocol_error)?;
                workspace.revalidate().map_err(protocol_error)?;
                store::commit_v3_root_exact_current_opened(
                    self.trusted_root(),
                    &current,
                    &proposed,
                    || {
                        workspace
                            .revalidate()
                            .map_err(|_| store::BootstrapError::transition_guard())
                    },
                )
                .map_err(protocol_error)?;
                workspace.revalidate().map_err(protocol_error)?;
                return Ok(PostTurnResolutionOutcomeV1::AwaitingObligationCut(
                    proposed.successor_transition_intent_map[&request.intent_id].clone(),
                ));
            }
            HostPostTurnProtocolEventKindV1::TerminalClean
            | HostPostTurnProtocolEventKindV1::TerminalFailure { .. } => {}
        }

        let mut terminal_authority = current_authority.clone();
        terminal_authority.authority_revision =
            next_revision(current_authority.authority_revision, "durable authority")?;
        terminal_authority.lifecycle_posture = HostSessionPostureV1::Terminal;
        terminal_authority.updated_at = request.completed_at.clone();
        let authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&terminal_authority))
                .map_err(protocol_error)?,
        };
        let post_turn_application_value = ApplicationResultHashInputV1 {
            schema_version: 1,
            intent_id: intent.intent_id.clone(),
            mode: intent.mode,
            run_id: intent.run_id.clone(),
            phase: ApplicationResultPhaseV1::PostTurn {
                completion_ref: completion.reference.clone(),
                authority_revision_before: current_authority.authority_revision,
                authority_revision_after: terminal_authority.authority_revision,
                active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
                resulting_posture: HostSessionPostureV1::Terminal,
                authority_record_commitment: authority_record_commitment.clone(),
            },
            applied_at: request.completed_at.clone(),
        };
        let post_turn_application_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::ApplicationResult,
            CanonicalObjectHashInputV1::ApplicationResult(&post_turn_application_value),
        )
        .map_err(protocol_error)?;
        let post_turn_application = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::ApplicationResult,
            &post_turn_application_bytes,
            None,
        )?;
        let terminal_value = TerminalHandoffHashInputV1 {
            schema_version: 1,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            terminal_state: TerminalHandoffStateV1::Applied,
            application_result_ref: Some(application_result_ref.clone()),
            input_acceptance_ref: input_acceptance_ref.clone(),
            post_turn_completion_ref: Some(completion.reference.clone()),
            post_turn_application_result_ref: Some(post_turn_application.reference.clone()),
            recorded_at: request.completed_at.clone(),
        };
        let terminal_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::TerminalHandoff,
            CanonicalObjectHashInputV1::TerminalHandoff(&terminal_value),
        )
        .map_err(protocol_error)?;
        let terminal = self.prepare_generated_successor_object(
            &current,
            AuthorityObjectKindV1::TerminalHandoff,
            &terminal_bytes,
            None,
        )?;

        if terminalize_pending_input {
            next.input_handoff = HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                input_ref: expected_input_ref.clone(),
                run_id: intent.run_id.clone(),
                terminal_handoff_ref: terminal.reference.clone(),
                terminal_at: request.completed_at.clone(),
            };
        }
        next.transport_payload_state =
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref: terminal.reference.clone(),
            };
        proposed.session_namespace_map.insert(
            intent.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::Authority(Box::new(terminal_authority.clone())),
        );
        let journal = proposed
            .successor_application_journal
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("successor application journal disappeared"))?;
        journal.post_turn_application = Some(super::store_schema::PostTurnApplicationJournalV2 {
            completion_ref: completion.reference.clone(),
            obligation_snapshot_ref: None,
            authority_revision_before: current_authority.authority_revision,
            authority_revision_after: terminal_authority.authority_revision,
            authority_record_commitment: authority_record_commitment.clone(),
            application_result_ref: post_turn_application.reference.clone(),
            applied_at: request.completed_at.clone(),
        });
        let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &mut next.state else {
            return Err(error(
                "applied ResumeOneTurn intent disappeared during post-turn terminalization",
            ));
        };
        *post_turn = Box::new(HostSessionPostTurnApplicationV2::Applied {
            completion_ref: Box::new(completion.reference.clone()),
            obligation_snapshot_ref: None,
            authority_revision_before: current_authority.authority_revision,
            authority_revision_after: terminal_authority.authority_revision,
            resulting_posture: HostSessionPostureV1::Terminal,
            application_result_ref: Box::new(post_turn_application.reference.clone()),
            applied_at: request.completed_at.clone(),
        });
        proposed
            .object_index
            .get_mut(&intent.transport_payload_ref.ref_id)
            .ok_or_else(|| error("transport payload index disappeared"))?
            .storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
            terminal_handoff_ref: terminal.reference.clone(),
        };
        for (reference, kind, byte_length) in [
            (
                &protocol_event.reference,
                AuthorityObjectKindV1::PostTurnProtocolEvent,
                protocol_event.byte_length,
            ),
            (
                &completion.reference,
                AuthorityObjectKindV1::PostTurnCompletion,
                completion.byte_length,
            ),
            (
                &post_turn_application.reference,
                AuthorityObjectKindV1::ApplicationResult,
                post_turn_application.byte_length,
            ),
            (
                &terminal.reference,
                AuthorityObjectKindV1::TerminalHandoff,
                terminal.byte_length,
            ),
        ] {
            insert_present_index_v3(&mut proposed, reference, kind, byte_length)?;
        }
        proposed.validate().map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                workspace
                    .revalidate()
                    .map_err(|_| store::BootstrapError::transition_guard())
            },
        )
        .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(PostTurnResolutionOutcomeV1::Applied(
            proposed.successor_transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn consume_obligation_snapshot(
        &self,
        request: &ConsumeObligationSnapshotRequestV1,
    ) -> Result<ObligationSnapshotConsumptionOutcomeV1, TransitionProtocolError> {
        let current = self.read_a12b_root()?;
        let intent = exact_successor_transition_attempt(
            &current,
            &request.intent_id,
            &request.issuer_request_id,
            &request.payload_commitment,
            "obligation snapshot consumption",
        )?;
        if intent.mode != HostSessionTransitionModeV1::ResumeOneTurn {
            return Err(error(
                "obligation snapshot consumption currently requires an applied ResumeOneTurn successor",
            ));
        }
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        let (
            authority_revision_after,
            startup_ownership,
            application_result_ref,
            post_turn,
        ) = match &intent.state {
            HostSessionTransitionIntentStateV3::Applied {
                authority_revision_after,
                startup_ownership,
                application_result_ref,
                post_turn,
                ..
            } => (
                *authority_revision_after,
                startup_ownership,
                application_result_ref,
                post_turn,
            ),
            _ => {
                return Err(error(
                    "obligation snapshot consumption currently requires an applied ResumeOneTurn successor",
                ))
            }
        };
        if startup_ownership.as_ref() != &HostSessionStartupOwnershipApplicationV1::NotApplicable {
            return Err(error(
                "ResumeOneTurn obligation snapshot consumption cannot carry startup ownership state",
            ));
        }

        if let HostSessionPostTurnApplicationV2::Applied {
            obligation_snapshot_ref,
            ..
        } = post_turn.as_ref()
        {
            let Some(snapshot_ref) = obligation_snapshot_ref.as_ref() else {
                return Err(error(
                    "applied ResumeOneTurn post-turn has no obligation snapshot to join",
                ));
            };
            let ObligationLedgerSnapshotReadV1::Complete { snapshot } = &request.ledger_read else {
                return Err(error(
                    "pending obligation reads cannot join an already applied ResumeOneTurn cut",
                ));
            };
            let expected_snapshot = ledger_snapshot_to_authority(snapshot)?;
            let bytes = store::read_typed_object_v2_or_v3_opened(
                self.trusted_root(),
                current.root_revision,
                snapshot_ref,
                None,
            )
            .map_err(protocol_error)?;
            let committed: ObligationSnapshotHashInputV1 =
                canonical_json::from_slice(&bytes).map_err(protocol_error)?;
            if committed != expected_snapshot {
                return Err(error(
                    "obligation snapshot conflicts with the committed ResumeOneTurn post-turn application",
                ));
            }
            workspace.revalidate().map_err(protocol_error)?;
            return Ok(ObligationSnapshotConsumptionOutcomeV1::Joined(intent));
        }

        let (
            completion_ref,
            expected_run_id,
            expected_authority_revision,
            acceptance_record_id,
            acceptance_record_revision,
            stream_id,
            accepted_work_identity,
            host_transition_correlation,
            required_terminal_event_id,
            required_terminal_event_sequence,
        ) = match post_turn.as_ref() {
            HostSessionPostTurnApplicationV2::AwaitingObligationCut {
                completion_ref,
                expected_run_id,
                expected_authority_revision,
                acceptance_record_id,
                acceptance_record_revision,
                stream_id,
                accepted_work_identity,
                host_transition_correlation,
                required_terminal_event_id,
                required_terminal_event_sequence,
                ..
            } => (
                completion_ref.as_ref(),
                expected_run_id,
                *expected_authority_revision,
                acceptance_record_id,
                *acceptance_record_revision,
                stream_id,
                accepted_work_identity,
                host_transition_correlation.as_ref(),
                required_terminal_event_id,
                *required_terminal_event_sequence,
            ),
            HostSessionPostTurnApplicationV2::Pending { .. } => {
                return Err(error(
                    "obligation snapshot consumption requires AwaitingObligationCut state",
                ))
            }
            HostSessionPostTurnApplicationV2::NotApplicable => return Err(error(
                "ResumeOneTurn obligation snapshot consumption cannot target NotApplicable state",
            )),
            HostSessionPostTurnApplicationV2::Applied { .. } => unreachable!(),
        };
        if expected_run_id != &intent.run_id
            || expected_authority_revision != authority_revision_after
        {
            return Err(error(
                "AwaitingObligationCut state conflicts with the applied ResumeOneTurn intent",
            ));
        }
        let current_authority =
            current_authority_from_v3_root(&current, &intent.orchestration_session_id)?.clone();
        if current_authority.authority_revision != expected_authority_revision
            || current_authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
            || current_authority
                .active_authoritative_participant_id
                .as_deref()
                != Some(intent.target_authoritative_participant_id.as_str())
        {
            return Err(error(
                "AwaitingObligationCut no longer matches the current applied ResumeOneTurn authority",
            ));
        }
        let acceptance_ref =
            match &intent.input_handoff {
                HostSessionTransitionInputHandoffV1::Accepted {
                    input_ref,
                    run_id,
                    acceptance_ref,
                    ..
                } if intent.transition_input_ref.as_ref() == Some(input_ref)
                    && run_id == &intent.run_id =>
                {
                    acceptance_ref.clone()
                }
                _ => return Err(error(
                    "AwaitingObligationCut ResumeOneTurn must retain exact accepted input handoff",
                )),
            };

        match &request.ledger_read {
            ObligationLedgerSnapshotReadV1::Pending {
                authority_store_id,
                orchestration_session_id,
                authoritative_participant_id,
                acceptance_record_id: pending_acceptance_record_id,
                acceptance_record_revision: pending_acceptance_record_revision,
                stream_id: pending_stream_id,
                accepted_work_identity: pending_work_identity,
                host_transition_correlation: pending_correlation,
                transition_intent_id,
                transition_run_id,
                authority_revision_observed,
                observed_session_ledger_revision,
                required_terminal_event_id: pending_terminal_event_id,
                required_terminal_event_sequence: pending_terminal_event_sequence,
            } => {
                if authority_store_id != &current.authority_store_id
                    || orchestration_session_id != &intent.orchestration_session_id
                    || authoritative_participant_id != &intent.target_authoritative_participant_id
                    || pending_acceptance_record_id != acceptance_record_id
                    || *pending_acceptance_record_revision != acceptance_record_revision
                    || pending_stream_id != stream_id
                    || pending_work_identity != accepted_work_identity
                    || pending_correlation != host_transition_correlation
                    || transition_intent_id != &intent.intent_id
                    || transition_run_id != &intent.run_id
                    || *authority_revision_observed != expected_authority_revision
                    || *observed_session_ledger_revision == 0
                    || pending_terminal_event_id != required_terminal_event_id
                    || *pending_terminal_event_sequence != required_terminal_event_sequence
                {
                    return Err(error(
                        "pending obligation snapshot read conflicts with AwaitingObligationCut commitments",
                    ));
                }
                workspace.revalidate().map_err(protocol_error)?;
                Ok(ObligationSnapshotConsumptionOutcomeV1::Pending(intent))
            }
            ObligationLedgerSnapshotReadV1::Complete { snapshot } => {
                let snapshot_value = ledger_snapshot_to_authority(snapshot)?;
                if snapshot_value.authority_store_id != current.authority_store_id
                    || snapshot_value.orchestration_session_id != intent.orchestration_session_id
                    || snapshot_value.authoritative_participant_id
                        != intent.target_authoritative_participant_id
                    || snapshot_value.acceptance_record_id != *acceptance_record_id
                    || snapshot_value.acceptance_record_revision != acceptance_record_revision
                    || snapshot_value.stream_id != *stream_id
                    || snapshot_value.accepted_work_identity != *accepted_work_identity
                    || &snapshot_value.host_transition_correlation != host_transition_correlation
                    || snapshot_value.transition_intent_id != intent.intent_id
                    || snapshot_value.transition_run_id != intent.run_id
                    || snapshot_value.authority_revision_observed != expected_authority_revision
                    || snapshot_value.materialization_cut.acceptance_record_id
                        != *acceptance_record_id
                    || snapshot_value
                        .materialization_cut
                        .acceptance_record_revision
                        != acceptance_record_revision
                    || snapshot_value.materialization_cut.stream_id != *stream_id
                    || snapshot_value.materialization_cut.terminal_event_id
                        != *required_terminal_event_id
                    || snapshot_value.materialization_cut.terminal_event_sequence
                        != required_terminal_event_sequence
                    || snapshot_value
                        .materialization_cut
                        .materialized_through_event_sequence
                        != required_terminal_event_sequence
                    || match snapshot_value.attention_disposition {
                        ObligationAttentionDispositionV1::NoUnresolvedAttention => {
                            !snapshot_value.unresolved_attention_obligations.is_empty()
                        }
                        ObligationAttentionDispositionV1::HasUnresolvedAttention => {
                            snapshot_value.unresolved_attention_obligations.is_empty()
                        }
                    }
                {
                    return Err(error(
                        "complete obligation snapshot conflicts with AwaitingObligationCut commitments",
                    ));
                }

                let snapshot_bytes = canonical_object_bytes(
                    AuthorityObjectKindV1::ObligationSnapshot,
                    CanonicalObjectHashInputV1::ObligationSnapshot(&snapshot_value),
                )
                .map_err(protocol_error)?;
                let snapshot_object = self.prepare_generated_successor_object(
                    &current,
                    AuthorityObjectKindV1::ObligationSnapshot,
                    &snapshot_bytes,
                    None,
                )?;
                let resulting_posture = match snapshot_value.attention_disposition {
                    ObligationAttentionDispositionV1::NoUnresolvedAttention => {
                        HostSessionPostureV1::ParkedResumable
                    }
                    ObligationAttentionDispositionV1::HasUnresolvedAttention => {
                        HostSessionPostureV1::AwaitingAttention
                    }
                };
                let mut next_authority = current_authority.clone();
                next_authority.authority_revision =
                    next_revision(current_authority.authority_revision, "durable authority")?;
                next_authority.lifecycle_posture = resulting_posture;
                next_authority.updated_at = snapshot_value.captured_at.clone();
                let authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: canonical_sha256(&authority_hash_input(&next_authority))
                        .map_err(protocol_error)?,
                };
                let post_turn_application_value = ApplicationResultHashInputV1 {
                    schema_version: 1,
                    intent_id: intent.intent_id.clone(),
                    mode: intent.mode,
                    run_id: intent.run_id.clone(),
                    phase: ApplicationResultPhaseV1::PostTurn {
                        completion_ref: completion_ref.clone(),
                        authority_revision_before: current_authority.authority_revision,
                        authority_revision_after: next_authority.authority_revision,
                        active_authoritative_participant_id: intent
                            .target_authoritative_participant_id
                            .clone(),
                        resulting_posture,
                        authority_record_commitment: authority_record_commitment.clone(),
                    },
                    applied_at: snapshot_value.captured_at.clone(),
                };
                let post_turn_application_bytes = canonical_object_bytes(
                    AuthorityObjectKindV1::ApplicationResult,
                    CanonicalObjectHashInputV1::ApplicationResult(&post_turn_application_value),
                )
                .map_err(protocol_error)?;
                let post_turn_application = self.prepare_generated_successor_object(
                    &current,
                    AuthorityObjectKindV1::ApplicationResult,
                    &post_turn_application_bytes,
                    None,
                )?;
                let terminal_value = TerminalHandoffHashInputV1 {
                    schema_version: 1,
                    intent_id: intent.intent_id.clone(),
                    run_id: intent.run_id.clone(),
                    payload_commitment: intent.payload_commitment.clone(),
                    terminal_state: TerminalHandoffStateV1::Applied,
                    application_result_ref: Some(application_result_ref.clone()),
                    input_acceptance_ref: Some(acceptance_ref.clone()),
                    post_turn_completion_ref: Some(completion_ref.clone()),
                    post_turn_application_result_ref: Some(post_turn_application.reference.clone()),
                    recorded_at: snapshot_value.captured_at.clone(),
                };
                let terminal_bytes = canonical_object_bytes(
                    AuthorityObjectKindV1::TerminalHandoff,
                    CanonicalObjectHashInputV1::TerminalHandoff(&terminal_value),
                )
                .map_err(protocol_error)?;
                let terminal = self.prepare_generated_successor_object(
                    &current,
                    AuthorityObjectKindV1::TerminalHandoff,
                    &terminal_bytes,
                    None,
                )?;

                let next_intent_revision =
                    next_revision(intent.intent_revision, "transition intent")?;
                let mut proposed = current.clone();
                proposed.root_revision = next_revision(current.root_revision, "authority root")?;
                proposed.session_namespace_map.insert(
                    intent.orchestration_session_id.clone(),
                    SessionNamespaceRecordV1::Authority(Box::new(next_authority.clone())),
                );
                let next = proposed
                    .successor_transition_intent_map
                    .get_mut(&intent.intent_id)
                    .ok_or_else(|| {
                        error(
                            "transition intent disappeared during obligation snapshot consumption",
                        )
                    })?;
                next.intent_revision = next_intent_revision;
                next.updated_at = snapshot_value.captured_at.clone();
                next.transport_payload_state =
                    HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                        terminal_handoff_ref: terminal.reference.clone(),
                    };
                let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &mut next.state
                else {
                    return Err(error(
                        "applied ResumeOneTurn intent disappeared during obligation snapshot consumption",
                    ));
                };
                *post_turn = Box::new(HostSessionPostTurnApplicationV2::Applied {
                    completion_ref: Box::new(completion_ref.clone()),
                    obligation_snapshot_ref: Some(Box::new(snapshot_object.reference.clone())),
                    authority_revision_before: current_authority.authority_revision,
                    authority_revision_after: next_authority.authority_revision,
                    resulting_posture,
                    application_result_ref: Box::new(post_turn_application.reference.clone()),
                    applied_at: snapshot_value.captured_at.clone(),
                });
                let journal = proposed
                    .successor_application_journal
                    .get_mut(&intent.intent_id)
                    .ok_or_else(|| error("successor application journal disappeared"))?;
                journal.post_turn_application =
                    Some(super::store_schema::PostTurnApplicationJournalV2 {
                        completion_ref: completion_ref.clone(),
                        obligation_snapshot_ref: Some(snapshot_object.reference.clone()),
                        authority_revision_before: current_authority.authority_revision,
                        authority_revision_after: next_authority.authority_revision,
                        authority_record_commitment: authority_record_commitment.clone(),
                        application_result_ref: post_turn_application.reference.clone(),
                        applied_at: snapshot_value.captured_at.clone(),
                    });
                proposed
                    .object_index
                    .get_mut(&intent.transport_payload_ref.ref_id)
                    .ok_or_else(|| error("transport payload index disappeared"))?
                    .storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
                    terminal_handoff_ref: terminal.reference.clone(),
                };
                for (reference, kind, byte_length) in [
                    (
                        &snapshot_object.reference,
                        AuthorityObjectKindV1::ObligationSnapshot,
                        snapshot_object.byte_length,
                    ),
                    (
                        &post_turn_application.reference,
                        AuthorityObjectKindV1::ApplicationResult,
                        post_turn_application.byte_length,
                    ),
                    (
                        &terminal.reference,
                        AuthorityObjectKindV1::TerminalHandoff,
                        terminal.byte_length,
                    ),
                ] {
                    insert_present_index_v3(&mut proposed, reference, kind, byte_length)?;
                }
                proposed.validate().map_err(protocol_error)?;
                workspace.revalidate().map_err(protocol_error)?;
                store::commit_v3_root_exact_current_opened(
                    self.trusted_root(),
                    &current,
                    &proposed,
                    || {
                        workspace
                            .revalidate()
                            .map_err(|_| store::BootstrapError::transition_guard())
                    },
                )
                .map_err(protocol_error)?;
                workspace.revalidate().map_err(protocol_error)?;
                Ok(ObligationSnapshotConsumptionOutcomeV1::Applied(
                    proposed.successor_transition_intent_map[&request.intent_id].clone(),
                ))
            }
        }
    }

    fn ensure_a12a_root(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
    ) -> Result<StateRootV2, TransitionProtocolError> {
        if let Ok(root) = self.read_a12a_root() {
            return Ok(root);
        }
        let v1 = self.read_root().map_err(protocol_error)?;
        if request.workspace_binding.authority_store_id != v1.authority_store_id
            || request.workspace_binding.authority_store_root != v1.bootstrap_home
            || v1.greenfield_namespace_certificate.authority_store_id != v1.authority_store_id
            || v1.greenfield_namespace_certificate.bootstrap_home != v1.bootstrap_home
        {
            return Err(error(
                "Start workspace or greenfield certificate binding is invalid",
            ));
        }
        store::upgrade_greenfield_root_opened(self.trusted_root()).map_err(protocol_error)?;
        self.read_a12a_root()
    }

    fn prepare_generated_start_object(
        &self,
        root: &StateRootV2,
        kind: AuthorityObjectKindV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<GeneratedObjectV1, TransitionProtocolError> {
        store::prepare_generated_object_v2_opened(
            self.trusted_root(),
            root.root_revision,
            kind,
            bytes,
            context,
        )
        .map_err(protocol_error)
    }

    fn prepare_generated_successor_object(
        &self,
        root: &StateRootV3,
        kind: AuthorityObjectKindV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<GeneratedObjectV1, TransitionProtocolError> {
        store::prepare_generated_object_v3_opened(
            self.trusted_root(),
            root.root_revision,
            kind,
            bytes,
            context,
        )
        .map_err(protocol_error)
    }

    fn allocate_sensitive_successor_object_ref(
        &self,
        root: &StateRootV3,
        kind: AuthorityObjectKindV1,
        bytes: &[u8],
        context: &ObjectVerificationContextV1,
    ) -> Result<AuthorityObjectRefV1, TransitionProtocolError> {
        store::allocate_sensitive_object_ref_v3_opened(
            self.trusted_root(),
            root.root_revision,
            kind,
            bytes,
            context,
        )
        .map_err(protocol_error)
    }

    fn prepare_typed_successor_object(
        &self,
        root: &StateRootV3,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<store::ObjectPublicationOutcomeV1, TransitionProtocolError> {
        store::prepare_typed_object_v3_opened(
            self.trusted_root(),
            root.root_revision,
            reference,
            bytes,
            context,
        )
        .map_err(protocol_error)
    }
}

fn exact_issuance_join(
    root: &StateRootV2,
    request: &IssueHostSessionTransitionRequestV1,
) -> Result<Option<HostSessionTransitionIntentV2>, TransitionProtocolError> {
    let by_request = root.issuer_request_index.get(&request.issuer_request_id);
    let by_intent = root.transition_intent_map.get(&request.intent_id);
    match (by_request, by_intent) {
        (None, None) => Ok(None),
        (Some(index), Some(intent))
            if index.intent_id == request.intent_id
                && index.orchestration_session_id == request.orchestration_session_id
                && intent.issuer_request_id == request.issuer_request_id
                && intent.orchestration_session_id == request.orchestration_session_id =>
        {
            let expected = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&payload_value_from_intent(intent))
                    .map_err(protocol_error)?,
            };
            if expected != intent.payload_commitment || index.payload_commitment != expected {
                return Err(error(
                    "existing issuance payload commitment is inconsistent",
                ));
            }
            if !request_matches_intent(request, intent) {
                return Err(error(
                    "intent or request identity does not match stored issuance",
                ));
            }
            Ok(Some(intent.clone()))
        }
        _ => Err(error(
            "intent ID or issuer request ID conflicts with stored issuance",
        )),
    }
}

fn exact_transition_attempt(
    root: &StateRootV2,
    intent_id: &str,
    issuer_request_id: &str,
    payload_commitment: &AuthorityObjectCommitmentV1,
    operation: &'static str,
) -> Result<HostSessionTransitionIntentV2, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != issuer_request_id
        || index.intent_id != intent_id
        || index.orchestration_session_id != intent.orchestration_session_id
        || index.payload_commitment != *payload_commitment
        || intent.payload_commitment != *payload_commitment
    {
        return Err(error(format!(
            "transition {operation} identity or payload commitment mismatch"
        )));
    }
    Ok(intent.clone())
}

fn exact_transition_attempt_v3(
    root: &StateRootV3,
    intent_id: &str,
    issuer_request_id: &str,
    payload_commitment: &AuthorityObjectCommitmentV1,
    operation: &'static str,
) -> Result<HostSessionTransitionIntentV2, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != issuer_request_id
        || index.intent_id != intent_id
        || index.orchestration_session_id != intent.orchestration_session_id
        || index.payload_commitment != *payload_commitment
        || intent.payload_commitment != *payload_commitment
    {
        return Err(error(format!(
            "transition {operation} identity or payload commitment mismatch"
        )));
    }
    Ok(intent.clone())
}

fn preserved_start_identity_matches_exact(
    root: &StateRootV3,
    intent_id: &str,
    issuer_request_id: &str,
) -> bool {
    matches!(
        (
            root.transition_intent_map.get(intent_id),
            root.issuer_request_index.get(issuer_request_id),
        ),
        (Some(intent), Some(index))
            if intent.issuer_request_id == issuer_request_id
                && index.intent_id == intent_id
                && index.orchestration_session_id == intent.orchestration_session_id
    )
}

fn successor_identity_collides_with_preserved_start(
    root: &StateRootV3,
    intent_id: &str,
    issuer_request_id: &str,
) -> bool {
    root.transition_intent_map.contains_key(intent_id)
        || root.issuer_request_index.contains_key(issuer_request_id)
}

fn exact_successor_issuance_join(
    root: &StateRootV3,
    request: &IssueSuccessorTransitionRequestV1,
) -> Result<Option<HostSessionTransitionIntentV3>, TransitionProtocolError> {
    if successor_identity_collides_with_preserved_start(
        root,
        &request.intent_id,
        &request.issuer_request_id,
    ) {
        return Err(error(
            "successor issuance identity collides with committed Start intent",
        ));
    }
    let by_request = root
        .successor_issuer_request_index
        .get(&request.issuer_request_id);
    let by_intent = root.successor_transition_intent_map.get(&request.intent_id);
    match (by_request, by_intent) {
        (None, None) => Ok(None),
        (Some(index), Some(intent))
            if index.intent_id == request.intent_id
                && index.orchestration_session_id == request.orchestration_session_id
                && intent.issuer_request_id == request.issuer_request_id
                && intent.orchestration_session_id == request.orchestration_session_id =>
        {
            let expected = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&successor_payload_value_from_intent(intent))
                    .map_err(protocol_error)?,
            };
            if expected != intent.payload_commitment || index.payload_commitment != expected {
                return Err(error(
                    "existing issuance payload commitment is inconsistent",
                ));
            }
            if !successor_request_matches_intent(request, intent) {
                return Err(error(
                    "intent or request identity does not match stored issuance",
                ));
            }
            Ok(Some(intent.clone()))
        }
        _ => Err(error(
            "issuer request identity collides with a different transition intent",
        )),
    }
}

fn exact_successor_transition_attempt(
    root: &StateRootV3,
    intent_id: &str,
    issuer_request_id: &str,
    payload_commitment: &AuthorityObjectCommitmentV1,
    action: &str,
) -> Result<HostSessionTransitionIntentV3, TransitionProtocolError> {
    let intent = root
        .successor_transition_intent_map
        .get(intent_id)
        .ok_or_else(|| error(format!("transition intent does not exist for {action}")))?;
    if intent.issuer_request_id != issuer_request_id {
        return Err(error("transition issuer request identity does not match"));
    }
    let issuer = root
        .successor_issuer_request_index
        .get(issuer_request_id)
        .ok_or_else(|| error("transition issuer request index is missing"))?;
    if issuer.intent_id != intent.intent_id
        || issuer.orchestration_session_id != intent.orchestration_session_id
        || &issuer.payload_commitment != payload_commitment
    {
        return Err(error("transition issuer request index does not match"));
    }
    if &intent.payload_commitment != payload_commitment {
        return Err(error("transition payload commitment does not match"));
    }
    let expected = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&successor_payload_value_from_intent(intent))
            .map_err(protocol_error)?,
    };
    if expected != *payload_commitment {
        return Err(error("transition payload commitment does not verify"));
    }
    Ok(intent.clone())
}

fn startup_protocol_event_id(event: &HostStartupOwnershipProtocolEventV1) -> &str {
    match event {
        HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id,
        } => ownership_acknowledgement_id.as_str(),
        HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected { rejection_id } => {
            rejection_id.as_str()
        }
        HostStartupOwnershipProtocolEventV1::StartupFailedBeforeOwnership { failure_id } => {
            failure_id.as_str()
        }
    }
}

fn startup_terminal_reason(
    event: &HostStartupOwnershipProtocolEventV1,
) -> Result<Option<HostStartupTerminalReasonV1>, TransitionProtocolError> {
    match event {
        HostStartupOwnershipProtocolEventV1::OwnershipAccepted { .. } => Ok(None),
        HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected { .. } => {
            Ok(Some(HostStartupTerminalReasonV1::RuntimeCreationRejected))
        }
        HostStartupOwnershipProtocolEventV1::StartupFailedBeforeOwnership { .. } => Ok(Some(
            HostStartupTerminalReasonV1::StartupFailedBeforeOwnership,
        )),
    }
}

fn validate_startup_actor_event(
    actor: &HostStartupOwnershipProtocolActorV1,
    event: &HostStartupOwnershipProtocolEventV1,
    active_participant_id: &str,
    claim_id: &str,
    claimant_attempt_id: &str,
) -> Result<(), TransitionProtocolError> {
    match (actor, event) {
        (
            HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant { participant_id },
            HostStartupOwnershipProtocolEventV1::OwnershipAccepted { .. }
            | HostStartupOwnershipProtocolEventV1::StartupFailedBeforeOwnership { .. },
        ) if participant_id == active_participant_id => Ok(()),
        (
            HostStartupOwnershipProtocolActorV1::LaunchApplicationClaimant {
                claim_id: event_claim_id,
                claimant_attempt_id: event_attempt_id,
            },
            HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected { .. },
        ) if event_claim_id == claim_id && event_attempt_id == claimant_attempt_id => Ok(()),
        _ => Err(error(
            "startup protocol actor or event does not match the applied transition",
        )),
    }
}

fn post_turn_observed_authority_revision(
    post_turn: &HostSessionPostTurnApplicationV2,
) -> Result<u64, TransitionProtocolError> {
    match post_turn {
        HostSessionPostTurnApplicationV2::Pending {
            expected_authority_revision,
            ..
        }
        | HostSessionPostTurnApplicationV2::AwaitingObligationCut {
            expected_authority_revision,
            ..
        } => Ok(*expected_authority_revision),
        HostSessionPostTurnApplicationV2::Applied {
            authority_revision_before,
            ..
        } => Ok(*authority_revision_before),
        HostSessionPostTurnApplicationV2::NotApplicable => Err(error(
            "ResumeOneTurn post-turn state cannot be NotApplicable",
        )),
    }
}

fn post_turn_completion_outcome(
    kind: &HostPostTurnProtocolEventKindV1,
) -> PostTurnCompletionOutcomeV1 {
    match kind {
        HostPostTurnProtocolEventKindV1::ResumableClean => {
            PostTurnCompletionOutcomeV1::ResumableClean
        }
        HostPostTurnProtocolEventKindV1::TerminalClean => {
            PostTurnCompletionOutcomeV1::TerminalClean
        }
        HostPostTurnProtocolEventKindV1::TerminalFailure { .. } => {
            PostTurnCompletionOutcomeV1::TerminalFailure
        }
    }
}

fn opaque_commitment(commitment: &AuthorityObjectCommitmentV1) -> OpaqueAuthorityCommitmentV1 {
    match commitment {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => {
            OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: digest_hex.clone(),
            }
        }
        AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id,
            domain,
            digest_hex,
        } => OpaqueAuthorityCommitmentV1::StoreHmacSha256 {
            key_id: key_id.clone(),
            domain: domain.clone(),
            digest_hex: digest_hex.clone(),
        },
    }
}

fn placeholder_ref_for_kind(kind: AuthorityObjectKindV1) -> AuthorityObjectRefV1 {
    AuthorityObjectRefV1 {
        ref_id: format!("placeholder-{kind:?}").to_lowercase(),
        object_kind: kind,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "0".repeat(64),
        },
    }
}

fn verify_post_turn_completion_join(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    completion_ref: &AuthorityObjectRefV1,
    expected_protocol_event: &PostTurnProtocolEventHashInputV1,
    expected_completion: &PostTurnCompletionHashInputV1,
) -> Result<(), TransitionProtocolError> {
    let completion_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        completion_ref,
        None,
    )
    .map_err(protocol_error)?;
    let committed_completion: PostTurnCompletionHashInputV1 =
        canonical_json::from_slice(&completion_bytes).map_err(protocol_error)?;
    let expected = PostTurnCompletionHashInputV1 {
        protocol_event_ref: committed_completion.protocol_event_ref.clone(),
        ..expected_completion.clone()
    };
    if committed_completion != expected {
        return Err(error(
            "post-turn completion conflicts with the committed ResumeOneTurn result",
        ));
    }
    let protocol_event_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &committed_completion.protocol_event_ref,
        None,
    )
    .map_err(protocol_error)?;
    let committed_protocol_event: PostTurnProtocolEventHashInputV1 =
        canonical_json::from_slice(&protocol_event_bytes).map_err(protocol_error)?;
    if &committed_protocol_event != expected_protocol_event {
        return Err(error(
            "post-turn protocol event conflicts with the committed ResumeOneTurn result",
        ));
    }
    Ok(())
}

fn verify_startup_ownership_join(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    result_ref: &AuthorityObjectRefV1,
    expected: &StartupOwnershipResultHashInputV1,
) -> Result<(), TransitionProtocolError> {
    let result_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        result_ref,
        None,
    )
    .map_err(protocol_error)?;
    let committed: StartupOwnershipResultHashInputV1 =
        canonical_json::from_slice(&result_bytes).map_err(protocol_error)?;
    if &committed != expected {
        return Err(error(
            "startup ownership result conflicts with the committed transition result",
        ));
    }
    Ok(())
}

fn ledger_snapshot_to_authority(
    snapshot: &obligation_ledger::ObligationSnapshotHashInputV1,
) -> Result<ObligationSnapshotHashInputV1, TransitionProtocolError> {
    Ok(ObligationSnapshotHashInputV1 {
        schema_version: 1,
        authority_store_id: snapshot.authority_store_id.clone(),
        orchestration_session_id: snapshot.orchestration_session_id.clone(),
        authoritative_participant_id: snapshot.authoritative_participant_id.clone(),
        acceptance_record_id: snapshot.acceptance_record_id.clone(),
        acceptance_record_revision: snapshot.acceptance_record_revision,
        stream_id: snapshot.stream_id.clone(),
        accepted_work_identity: snapshot.accepted_work_identity.clone(),
        host_transition_correlation: snapshot.host_transition_correlation.clone(),
        transition_intent_id: snapshot.transition_intent_id.clone(),
        transition_run_id: snapshot.transition_run_id.clone(),
        authority_revision_observed: snapshot.authority_revision_observed,
        materialization_cut: snapshot.materialization_cut.clone(),
        materialized_journal_events: snapshot.materialized_journal_events.clone(),
        attention_disposition: match snapshot.attention_disposition {
            obligation_ledger::ObligationAttentionDispositionV1::NoUnresolvedAttention => {
                ObligationAttentionDispositionV1::NoUnresolvedAttention
            }
            obligation_ledger::ObligationAttentionDispositionV1::HasUnresolvedAttention => {
                ObligationAttentionDispositionV1::HasUnresolvedAttention
            }
        },
        unresolved_attention_obligations: snapshot
            .unresolved_attention_obligations
            .iter()
            .map(|obligation| UnresolvedAttentionObligationSnapshotEntryV1 {
                obligation_id: obligation.obligation_id.clone(),
                obligation_revision: obligation.obligation_revision,
                canonical_record_commitment: obligation.canonical_record_commitment.clone(),
            })
            .collect(),
        captured_at: TimestampV1::parse(
            snapshot
                .captured_at
                .to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .map_err(protocol_error)?,
    })
}

fn current_authority_from_v3_root<'a>(
    root: &'a StateRootV3,
    orchestration_session_id: &str,
) -> Result<&'a DurableSessionAuthorityV1, TransitionProtocolError> {
    let Some(SessionNamespaceRecordV1::Authority(authority)) =
        root.session_namespace_map.get(orchestration_session_id)
    else {
        return Err(error("session has no current durable authority"));
    };
    Ok(authority.as_ref())
}

fn validate_new_successor_request(
    root: &StateRootV3,
    durable_current: &DurableSessionAuthorityV1,
    current: &ResolvedCurrentAuthorityV1,
    request: &IssueSuccessorTransitionRequestV1,
    ttl_seconds: i64,
) -> Result<(), TransitionProtocolError> {
    validate_successor_request_shape(request, ttl_seconds)?;
    if successor_identity_collides_with_preserved_start(
        root,
        &request.intent_id,
        &request.issuer_request_id,
    ) {
        return Err(error(
            "successor issuance identity collides with committed Start intent",
        ));
    }
    if &current.authority != durable_current {
        return Err(error(
            "exact current authority snapshot no longer matches durable state",
        ));
    }
    let expected_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&authority_hash_input(durable_current))
            .map_err(protocol_error)?,
    };
    let expected_lineage = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
            schema_version: 1,
            orchestration_session_id: durable_current.orchestration_session_id.clone(),
            participant_ids: durable_current.authoritative_participant_lineage.clone(),
        })
        .map_err(protocol_error)?,
    };
    if current.observation.authority_record_commitment != expected_commitment
        || current.observation.authoritative_lineage_commitment != expected_lineage
    {
        return Err(error(
            "exact current authority commitments no longer match durable state",
        ));
    }
    if request.orchestration_session_id != durable_current.orchestration_session_id
        || request.shell_trace_session_id != durable_current.shell_trace_session_id
        || request.workspace_binding != durable_current.workspace_binding
        || request.world_binding != durable_current.world_binding
        || durable_current.host_attach_contract_ref.is_none()
    {
        return Err(error(
            "successor issuance binding does not match current authority",
        ));
    }
    let Some(active_participant_id) = durable_current
        .active_authoritative_participant_id
        .as_deref()
    else {
        return Err(error(
            "successor issuance requires an active current authoritative participant",
        ));
    };
    let expected_precondition = HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision: durable_current.authority_revision,
        authority_record_commitment: expected_commitment,
        active_authoritative_participant_id: active_participant_id.to_owned(),
        authoritative_lineage_commitment: expected_lineage,
        lifecycle_posture: durable_current.lifecycle_posture,
    };
    if request.authority_precondition != expected_precondition
        || request.source_authoritative_participant_id.as_deref() != Some(active_participant_id)
        || !matches!(
            durable_current.lifecycle_posture,
            HostSessionPostureV1::ParkedResumable
                | HostSessionPostureV1::DetachedReconciled
                | HostSessionPostureV1::AwaitingAttention
                | HostSessionPostureV1::StaleRecoverable
        )
    {
        return Err(error(
            "successor issuance precondition does not match current authority",
        ));
    }
    let mut expected_lineage = durable_current.authoritative_participant_lineage.clone();
    expected_lineage.push(request.target_authoritative_participant_id.clone());
    if request.resulting_authoritative_lineage != expected_lineage {
        return Err(error(
            "successor lineage must preserve the full current lineage and append the target",
        ));
    }
    match request.mode {
        HostSessionTransitionModeV1::Attach => {
            if request.transition_input.is_some()
                || request.resume_handle_ref.is_some()
                || request.post_turn_disposition.is_some()
            {
                return Err(error(
                    "Attach successor input or post-turn state is invalid",
                ));
            }
        }
        HostSessionTransitionModeV1::ResumeOneTurn => {
            let Some(resume_handle_ref) = request.resume_handle_ref.as_ref() else {
                return Err(error("Resume successor requires a durable resume handle"));
            };
            if request.transition_input.is_none()
                || request.post_turn_disposition
                    != Some(HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal)
                || !durable_current
                    .internal_resume_handle_refs
                    .iter()
                    .any(|current| current == resume_handle_ref)
            {
                return Err(error(
                    "Resume successor handle, input, or post-turn disposition is invalid",
                ));
            }
        }
        HostSessionTransitionModeV1::Start => {
            return Err(error("successor issuance cannot use Start mode"))
        }
    }
    Ok(())
}

fn validate_successor_request_shape(
    request: &IssueSuccessorTransitionRequestV1,
    ttl_seconds: i64,
) -> Result<(), TransitionProtocolError> {
    if ttl_seconds <= 0 || ttl_seconds > MAX_INTENT_TTL_SECONDS {
        return Err(error("transition TTL is outside the strict V1 bound"));
    }
    if request.intent_id.is_empty()
        || request.issuer_request_id.is_empty()
        || request.orchestration_session_id.is_empty()
        || request.shell_trace_session_id.is_empty()
        || request.target_authoritative_participant_id.is_empty()
        || request.run_id.is_empty()
        || request
            .workspace_binding
            .workspace_root
            .physical_path
            .is_empty()
        || request
            .workspace_binding
            .authority_store_root
            .physical_path
            .is_empty()
        || request.workspace_binding.authority_store_id.is_empty()
        || request.resulting_authoritative_lineage.is_empty()
        || !all_unique_participants(&request.resulting_authoritative_lineage)
        || request.resulting_authoritative_lineage.last()
            != Some(&request.target_authoritative_participant_id)
    {
        return Err(error("successor request shape is invalid"));
    }
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision,
        active_authoritative_participant_id,
        ..
    } = &request.authority_precondition
    else {
        return Err(error(
            "successor request requires an exact current authority revision precondition",
        ));
    };
    if *authority_revision == 0
        || active_authoritative_participant_id.is_empty()
        || request.source_authoritative_participant_id.as_deref()
            != Some(active_authoritative_participant_id.as_str())
        || request
            .resulting_authoritative_lineage
            .iter()
            .filter(|participant| *participant == active_authoritative_participant_id)
            .count()
            != 1
        || active_authoritative_participant_id == &request.target_authoritative_participant_id
    {
        return Err(error(
            "successor request authority or lineage identity is invalid",
        ));
    }
    Ok(())
}

fn verify_successor_authority_precondition_holds(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
) -> Result<(), TransitionProtocolError> {
    let current_authority = current_authority_from_v3_root(root, &intent.orchestration_session_id)?;
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision,
        authority_record_commitment,
        active_authoritative_participant_id,
        authoritative_lineage_commitment,
        lifecycle_posture,
    } = &intent.authority_precondition
    else {
        return Err(error(
            "successor intent precondition must bind to an exact current authority revision",
        ));
    };
    let current_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&authority_hash_input(current_authority))
            .map_err(protocol_error)?,
    };
    let current_lineage = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
            schema_version: 1,
            orchestration_session_id: current_authority.orchestration_session_id.clone(),
            participant_ids: current_authority.authoritative_participant_lineage.clone(),
        })
        .map_err(protocol_error)?,
    };
    if current_authority.authority_revision != *authority_revision
        || &current_commitment != authority_record_commitment
        || current_authority
            .active_authoritative_participant_id
            .as_deref()
            != Some(active_authoritative_participant_id.as_str())
        || &current_lineage != authoritative_lineage_commitment
        || current_authority.lifecycle_posture != *lifecycle_posture
        || current_authority.orchestration_session_id != intent.orchestration_session_id
        || current_authority.shell_trace_session_id != intent.shell_trace_session_id
        || current_authority.workspace_binding != intent.workspace_binding
        || current_authority.world_binding != intent.world_binding
        || current_authority.host_attach_contract_ref.as_ref()
            != Some(&intent.host_attach_contract_ref)
    {
        return Err(error(
            "transition authority precondition no longer holds against current durable authority",
        ));
    }
    Ok(())
}

fn verify_start_reservation(
    root: &StateRootV2,
    intent: &HostSessionTransitionIntentV2,
) -> Result<(), TransitionProtocolError> {
    match root
        .session_namespace_map
        .get(&intent.orchestration_session_id)
    {
        Some(SessionNamespaceRecordV1::StartReservation(reservation))
            if reservation.intent_id == intent.intent_id
                && reservation.issuer_request_id == intent.issuer_request_id
                && reservation.payload_commitment == intent.payload_commitment =>
        {
            Ok(())
        }
        _ => Err(error("Start intent does not own its exact reservation")),
    }
}

fn verify_start_intent_objects(
    authority: &HostSessionAuthority,
    root: &StateRootV2,
    intent: &HostSessionTransitionIntentV2,
) -> Result<(), TransitionProtocolError> {
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    let descriptor_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.descriptor_ref,
        None,
    )
    .map_err(protocol_error)?;
    let descriptor: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(&descriptor_bytes).map_err(protocol_error)?;
    let attach_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.host_attach_contract_ref,
        None,
    )
    .map_err(protocol_error)?;
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
    let policy_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &attach.contract.policy_ref,
        None,
    )
    .map_err(protocol_error)?;
    let policy: PolicyObjectHashInputV1 =
        canonical_json::from_slice(&policy_bytes).map_err(protocol_error)?;
    if descriptor.schema_version != 1
        || descriptor.descriptor.schema_version != 1
        || attach.schema_version != 1
        || attach.contract.schema_version != 1
        || policy.schema_version != 1
        || attach.contract.descriptor_ref != intent.descriptor_ref
        || attach.contract.backend_id != descriptor.descriptor.backend_id
        || attach.contract.execution_scope != descriptor.descriptor.execution_scope
        || attach.contract.protocol != descriptor.descriptor.protocol
        || attach.contract.continuity_resume_handle_ref.is_some()
    {
        return Err(error(
            "Start descriptor, attach contract, or policy projection is inconsistent",
        ));
    }
    store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.target_participant_lease_token_ref,
        Some(&context),
    )
    .map_err(protocol_error)?;
    if let Some(reference) = intent.transition_input_ref.as_ref() {
        store::read_typed_object_v2_or_v3_opened(
            authority.trusted_root(),
            root.root_revision,
            reference,
            Some(&context),
        )
        .map_err(protocol_error)?;
    }
    let parent = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V2(Box::new(
            intent.clone(),
        ))),
    };
    let transport_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.transport_payload_ref,
        Some(&parent),
    )
    .map_err(protocol_error)?;
    let transport: TransitionTransportPayloadObjectV1 =
        canonical_json::from_slice(&transport_bytes).map_err(protocol_error)?;
    let expected_transport = TransitionTransportPayloadObjectV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        mode: intent.mode,
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        caller: intent.caller.clone(),
        source_authoritative_participant_id: None,
        target_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        target_participant_lease_token_ref: intent.target_participant_lease_token_ref.clone(),
        run_id: intent.run_id.clone(),
        resulting_authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        descriptor_ref: intent.descriptor_ref.clone(),
        host_attach_contract_ref: intent.host_attach_contract_ref.clone(),
        resume_handle_ref: None,
        transition_input_ref: intent.transition_input_ref.clone(),
        post_turn_disposition: None,
    };
    if transport != expected_transport {
        return Err(error("Start transport payload projection is inconsistent"));
    }
    let expected_payload = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&payload_value_from_intent(intent)).map_err(protocol_error)?,
    };
    if expected_payload != intent.payload_commitment {
        return Err(error("transition payload commitment does not verify"));
    }
    Ok(())
}

pub(super) fn verify_applied_start(
    authority: &HostSessionAuthority,
    root: &StateRootV2,
    intent: &HostSessionTransitionIntentV2,
    request: &ApplyHostSessionTransitionRequestV1,
) -> Result<(), TransitionProtocolError> {
    let (
        claim_id,
        authority_revision_before,
        authority_revision_after,
        active_participant,
        resulting_posture,
        authority_record_commitment,
        application_result_ref,
        startup_ownership,
        post_turn,
        applied_at,
    ) = match &intent.state {
        HostSessionTransitionIntentStateV2::Applied {
            claim_id,
            authority_revision_before,
            authority_revision_after,
            active_authoritative_participant_id,
            resulting_posture,
            authority_record_commitment,
            application_result_ref,
            startup_ownership,
            post_turn,
            applied_at,
            ..
        } => (
            claim_id,
            authority_revision_before,
            *authority_revision_after,
            active_authoritative_participant_id,
            *resulting_posture,
            authority_record_commitment,
            application_result_ref,
            startup_ownership,
            post_turn,
            applied_at,
        ),
        _ => return Err(error("Start application has not committed")),
    };
    if claim_id != &request.claim_id
        || request.expected_intent_revision != request.expected_claim_revision
        || request
            .expected_claim_revision
            .checked_add(1)
            .is_none_or(|revision| revision != intent.intent_revision)
        || authority_revision_before.is_some()
        || authority_revision_after != 1
        || active_participant != &intent.target_authoritative_participant_id
        || resulting_posture != HostSessionPostureV1::ActiveAttached
        || !matches!(
            startup_ownership.as_ref(),
            HostSessionStartupOwnershipApplicationV1::Pending {
                expected_run_id,
                expected_authority_revision: 1,
                expected_active_authoritative_participant_id,
            } if expected_run_id == &intent.run_id
                && expected_active_authoritative_participant_id == active_participant
        )
        || post_turn.as_ref() != &HostSessionPostTurnApplicationV1::NotApplicable
    {
        return Err(error("committed Start application result conflicts"));
    }
    let Some(SessionNamespaceRecordV1::Authority(authority_record)) = root
        .session_namespace_map
        .get(&intent.orchestration_session_id)
    else {
        return Err(error("committed Start has no durable authority"));
    };
    let application_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        application_result_ref,
        None,
    )
    .map_err(protocol_error)?;
    let application: ApplicationResultHashInputV1 =
        canonical_json::from_slice(&application_bytes).map_err(protocol_error)?;
    let expected_application = ApplicationResultHashInputV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        mode: HostSessionTransitionModeV1::Start,
        run_id: intent.run_id.clone(),
        phase: ApplicationResultPhaseV1::InitialTransition {
            authority_revision_before: None,
            authority_revision_after,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: authority_record_commitment.clone(),
            post_turn_pending_run_id: None,
        },
        applied_at: applied_at.clone(),
    };
    if application != expected_application {
        return Err(error("committed application result object conflicts"));
    }
    verify_start_intent_objects(authority, root, intent)?;
    let attach_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.host_attach_contract_ref,
        None,
    )
    .map_err(protocol_error)?;
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
    let policy_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &attach.contract.policy_ref,
        None,
    )
    .map_err(protocol_error)?;
    let policy: PolicyObjectHashInputV1 =
        canonical_json::from_slice(&policy_bytes).map_err(protocol_error)?;
    let initial_authority = DurableSessionAuthorityV1 {
        schema_version: 1,
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        authority_revision: authority_revision_after,
        origin: DurableSessionAuthorityOriginV1::StartIntent {
            intent_id: intent.intent_id.clone(),
            issuer_request_id: intent.issuer_request_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
        },
        authoritative_participant_lineage: intent.resulting_authoritative_lineage.clone(),
        active_authoritative_participant_id: Some(
            intent.target_authoritative_participant_id.clone(),
        ),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        host_attach_contract_ref: Some(intent.host_attach_contract_ref.clone()),
        retained_worker_refs: Vec::new(),
        internal_resume_handle_refs: Vec::new(),
        lifecycle_posture: HostSessionPostureV1::ActiveAttached,
        current_policy_ref: Some(attach.contract.policy_ref),
        current_policy_revision: Some(policy.policy_revision),
        updated_at: applied_at.clone(),
    };
    verify_retained_registration_descendant(
        root,
        &initial_authority,
        authority_record,
        authority_record_commitment,
    )
}

fn verify_applied_successor(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    request: &ApplyHostSessionTransitionRequestV1,
) -> Result<(), TransitionProtocolError> {
    let (
        claim_id,
        authority_revision_before,
        authority_revision_after,
        active_participant,
        resulting_posture,
        authority_record_commitment,
        application_result_ref,
        startup_ownership,
        post_turn,
        applied_at,
    ) = match &intent.state {
        HostSessionTransitionIntentStateV3::Applied {
            claim_id,
            authority_revision_before,
            authority_revision_after,
            active_authoritative_participant_id,
            resulting_posture,
            authority_record_commitment,
            application_result_ref,
            startup_ownership,
            post_turn,
            applied_at,
            ..
        } => (
            claim_id,
            authority_revision_before,
            *authority_revision_after,
            active_authoritative_participant_id,
            *resulting_posture,
            authority_record_commitment,
            application_result_ref,
            startup_ownership,
            post_turn,
            applied_at,
        ),
        _ => return Err(error("successor application has not committed")),
    };
    let precondition_revision = match &intent.authority_precondition {
        HostSessionAuthorityPreconditionV1::ExpectedRevision {
            authority_revision, ..
        } => *authority_revision,
        HostSessionAuthorityPreconditionV1::ExpectedAbsent => {
            return Err(error("successor application precondition is invalid"))
        }
    };
    if claim_id != &request.claim_id
        || authority_revision_before != &Some(precondition_revision)
        || authority_revision_after != next_revision(precondition_revision, "durable authority")?
        || active_participant != &intent.target_authoritative_participant_id
        || resulting_posture != HostSessionPostureV1::ActiveAttached
    {
        return Err(error("committed successor application result conflicts"));
    }
    match intent.mode {
        HostSessionTransitionModeV1::Attach => {
            if !matches!(
                startup_ownership.as_ref(),
                HostSessionStartupOwnershipApplicationV1::Pending {
                    expected_run_id,
                    expected_authority_revision,
                    expected_active_authoritative_participant_id,
                } if expected_run_id == &intent.run_id
                    && *expected_authority_revision == authority_revision_after
                    && expected_active_authoritative_participant_id == active_participant
            ) || post_turn.as_ref() != &HostSessionPostTurnApplicationV2::NotApplicable
            {
                return Err(error("committed Attach successor state conflicts"));
            }
        }
        HostSessionTransitionModeV1::ResumeOneTurn => {
            if startup_ownership.as_ref()
                != &HostSessionStartupOwnershipApplicationV1::NotApplicable
                || !matches!(
                    post_turn.as_ref(),
                    HostSessionPostTurnApplicationV2::Pending {
                        expected_run_id,
                        expected_authority_revision,
                    } if expected_run_id == &intent.run_id
                        && *expected_authority_revision == authority_revision_after
                )
            {
                return Err(error("committed Resume successor state conflicts"));
            }
        }
        HostSessionTransitionModeV1::Start => {
            return Err(error("successor application cannot use Start mode"))
        }
    }
    let application_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        application_result_ref,
        None,
    )
    .map_err(protocol_error)?;
    let application: ApplicationResultHashInputV1 =
        canonical_json::from_slice(&application_bytes).map_err(protocol_error)?;
    let expected_application = ApplicationResultHashInputV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        mode: intent.mode,
        run_id: intent.run_id.clone(),
        phase: ApplicationResultPhaseV1::InitialTransition {
            authority_revision_before: Some(precondition_revision),
            authority_revision_after,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: authority_record_commitment.clone(),
            post_turn_pending_run_id: matches!(
                intent.mode,
                HostSessionTransitionModeV1::ResumeOneTurn
            )
            .then(|| intent.run_id.clone()),
        },
        applied_at: applied_at.clone(),
    };
    if application != expected_application {
        return Err(error("committed application result object conflicts"));
    }
    verify_successor_intent_objects(authority, root, intent)
}

pub(super) fn verify_retained_registration_descendant(
    root: &StateRootV2,
    initial_authority: &DurableSessionAuthorityV1,
    current_authority: &DurableSessionAuthorityV1,
    initial_commitment: &AuthorityObjectCommitmentV1,
) -> Result<(), TransitionProtocolError> {
    let mut expected = initial_authority.clone();
    let mut expected_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&authority_hash_input(&expected)).map_err(protocol_error)?,
    };
    if &expected_commitment != initial_commitment
        || current_authority.authority_revision < expected.authority_revision
    {
        return Err(error("Start application authority origin is inconsistent"));
    }
    let mut consumed = Vec::new();
    while expected.authority_revision < current_authority.authority_revision {
        let candidates = root
            .retained_worker_registration_journal
            .iter()
            .filter(|(_, registration)| {
                registration.orchestration_session_id == expected.orchestration_session_id
                    && registration.authority_revision_before == expected.authority_revision
                    && registration.authority_record_commitment_before == expected_commitment
            })
            .collect::<Vec<_>>();
        let [(registration_key, registration)] = candidates.as_slice() else {
            return Err(error(
                "current authority has no unique contiguous R0 registration ancestry",
            ));
        };
        if *registration_key != &registration.registration_id
            || registration.schema_version != 1
            || registration.authority_revision_after
                != next_revision(expected.authority_revision, "durable authority")?
            || expected.current_policy_ref.as_ref() != Some(&registration.current_policy_ref)
            || expected.world_binding.as_ref() != Some(&registration.world_binding)
            || expected
                .authoritative_participant_lineage
                .contains(&registration.retained_participant_id)
            || expected
                .retained_worker_refs
                .contains(&registration.retained_worker_ref)
        {
            return Err(error("R0 registration ancestry link is inconsistent"));
        }
        let request = root
            .retained_worker_registration_request_index
            .get(&registration.issuer_request_id)
            .ok_or_else(|| error("R0 registration ancestry has no request record"))?;
        if request.schema_version != 1
            || request.issuer_request_id != registration.issuer_request_id
            || request.registration_id != registration.registration_id
            || request.orchestration_session_id != registration.orchestration_session_id
            || request.authority_revision_before != registration.authority_revision_before
            || request.authority_record_commitment_before
                != registration.authority_record_commitment_before
            || request.retained_participant_id != registration.retained_participant_id
            || request.descriptor_ref_id != registration.descriptor_ref.ref_id
            || request.descriptor_commitment != registration.descriptor_ref.commitment
            || request.resume_handle_ref_id != registration.resume_handle_ref.ref_id
            || request.resume_handle_commitment != registration.resume_handle_ref.commitment
            || request.retained_worker_ref_id != registration.retained_worker_ref.ref_id
            || request.retained_worker_commitment != registration.retained_worker_ref.commitment
            || request.current_policy_ref != registration.current_policy_ref
            || request.world_binding != registration.world_binding
            || request.registered_at != registration.registered_at
            || !matches!(
                &request.state,
                RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                    authority_revision_after,
                    authority_record_commitment_after,
                } if *authority_revision_after == registration.authority_revision_after
                    && authority_record_commitment_after
                        == &registration.authority_record_commitment_after
            )
        {
            return Err(error("R0 registration request and ancestry link disagree"));
        }

        expected.authority_revision = registration.authority_revision_after;
        expected
            .authoritative_participant_lineage
            .push(registration.retained_participant_id.clone());
        expected
            .retained_worker_refs
            .push(registration.retained_worker_ref.clone());
        expected.updated_at = registration.registered_at.clone();
        let lineage_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: expected.orchestration_session_id.clone(),
                participant_ids: expected.authoritative_participant_lineage.clone(),
            })
            .map_err(protocol_error)?,
        };
        if lineage_commitment != registration.authoritative_lineage_commitment_after {
            return Err(error("R0 registration lineage commitment is inconsistent"));
        }
        expected_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&expected))
                .map_err(protocol_error)?,
        };
        if expected_commitment != registration.authority_record_commitment_after {
            return Err(error(
                "R0 registration authority commitment is inconsistent",
            ));
        }
        consumed.push(registration.registration_id.clone());
    }
    let session_registration_count = root
        .retained_worker_registration_journal
        .values()
        .filter(|registration| {
            registration.orchestration_session_id == expected.orchestration_session_id
        })
        .count();
    if consumed.len() != session_registration_count || expected != *current_authority {
        return Err(error(
            "current authority is not the exact contiguous R0 registration descendant",
        ));
    }
    Ok(())
}

fn verify_retained_registration_descendant_v3(
    root: &StateRootV3,
    initial_authority: &DurableSessionAuthorityV1,
    current_authority: &DurableSessionAuthorityV1,
    initial_commitment: &AuthorityObjectCommitmentV1,
) -> Result<(), TransitionProtocolError> {
    let mut expected = initial_authority.clone();
    let mut expected_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&authority_hash_input(&expected)).map_err(protocol_error)?,
    };
    if &expected_commitment != initial_commitment
        || current_authority.authority_revision < expected.authority_revision
    {
        return Err(error("Start application authority origin is inconsistent"));
    }
    let mut consumed = Vec::new();
    while expected.authority_revision < current_authority.authority_revision {
        let candidates = root
            .retained_worker_registration_journal
            .iter()
            .filter(|(_, registration)| {
                registration.orchestration_session_id == expected.orchestration_session_id
                    && registration.authority_revision_before == expected.authority_revision
                    && registration.authority_record_commitment_before == expected_commitment
            })
            .collect::<Vec<_>>();
        let [(registration_key, registration)] = candidates.as_slice() else {
            return Err(error(
                "current authority has no unique contiguous R0 registration ancestry",
            ));
        };
        if *registration_key != &registration.registration_id
            || registration.schema_version != 1
            || registration.authority_revision_after
                != next_revision(expected.authority_revision, "durable authority")?
            || expected.current_policy_ref.as_ref() != Some(&registration.current_policy_ref)
            || expected.world_binding.as_ref() != Some(&registration.world_binding)
            || expected
                .authoritative_participant_lineage
                .contains(&registration.retained_participant_id)
            || expected
                .retained_worker_refs
                .contains(&registration.retained_worker_ref)
        {
            return Err(error("R0 registration ancestry link is inconsistent"));
        }
        let request = root
            .retained_worker_registration_request_index
            .get(&registration.issuer_request_id)
            .ok_or_else(|| error("R0 registration ancestry has no request record"))?;
        if request.schema_version != 1
            || request.issuer_request_id != registration.issuer_request_id
            || request.registration_id != registration.registration_id
            || request.orchestration_session_id != registration.orchestration_session_id
            || request.authority_revision_before != registration.authority_revision_before
            || request.authority_record_commitment_before
                != registration.authority_record_commitment_before
            || request.retained_participant_id != registration.retained_participant_id
            || request.descriptor_ref_id != registration.descriptor_ref.ref_id
            || request.descriptor_commitment != registration.descriptor_ref.commitment
            || request.resume_handle_ref_id != registration.resume_handle_ref.ref_id
            || request.resume_handle_commitment != registration.resume_handle_ref.commitment
            || request.retained_worker_ref_id != registration.retained_worker_ref.ref_id
            || request.retained_worker_commitment != registration.retained_worker_ref.commitment
            || request.current_policy_ref != registration.current_policy_ref
            || request.world_binding != registration.world_binding
            || request.registered_at != registration.registered_at
            || !matches!(
                &request.state,
                RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                    authority_revision_after,
                    authority_record_commitment_after,
                } if *authority_revision_after == registration.authority_revision_after
                    && authority_record_commitment_after
                        == &registration.authority_record_commitment_after
            )
        {
            return Err(error("R0 registration request and ancestry link disagree"));
        }

        expected.authority_revision = registration.authority_revision_after;
        expected
            .authoritative_participant_lineage
            .push(registration.retained_participant_id.clone());
        expected
            .retained_worker_refs
            .push(registration.retained_worker_ref.clone());
        expected.updated_at = registration.registered_at.clone();
        let lineage_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: expected.orchestration_session_id.clone(),
                participant_ids: expected.authoritative_participant_lineage.clone(),
            })
            .map_err(protocol_error)?,
        };
        if lineage_commitment != registration.authoritative_lineage_commitment_after {
            return Err(error("R0 registration lineage commitment is inconsistent"));
        }
        expected_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&expected))
                .map_err(protocol_error)?,
        };
        if expected_commitment != registration.authority_record_commitment_after {
            return Err(error(
                "R0 registration authority commitment is inconsistent",
            ));
        }
        consumed.push(registration.registration_id.clone());
    }
    let session_registration_count = root
        .retained_worker_registration_journal
        .values()
        .filter(|registration| {
            registration.orchestration_session_id == expected.orchestration_session_id
        })
        .count();
    if consumed.len() != session_registration_count || expected != *current_authority {
        return Err(error(
            "current authority is not the exact contiguous R0 registration descendant",
        ));
    }
    Ok(())
}

fn verify_terminal_successor(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
) -> Result<(), TransitionProtocolError> {
    let (terminal_handoff_ref, recorded_at) = match &intent.state {
        HostSessionTransitionIntentStateV3::Expired {
            terminal_handoff_ref,
            expired_at,
        } => (terminal_handoff_ref, expired_at),
        _ => return Err(error("successor transition is not expired")),
    };
    let bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        terminal_handoff_ref,
        None,
    )
    .map_err(protocol_error)?;
    let terminal: super::schema::TerminalHandoffHashInputV2 =
        canonical_json::from_slice(&bytes).map_err(protocol_error)?;
    let expected = super::schema::TerminalHandoffHashInputV2 {
        schema_version: 2,
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        terminal_state: TerminalHandoffStateV1::Expired,
        application_result_ref: None,
        input_acceptance_ref: None,
        startup_ownership_result_ref: None,
        post_turn_completion_ref: None,
        post_turn_application_result_ref: None,
        recorded_at: recorded_at.clone(),
    };
    if terminal != expected {
        return Err(error("expired successor terminal handoff conflicts"));
    }
    verify_successor_intent_objects(authority, root, intent)
}

fn authority_hash_input(
    authority: &DurableSessionAuthorityV1,
) -> DurableSessionAuthorityHashInputV1 {
    DurableSessionAuthorityHashInputV1 {
        schema_version: authority.schema_version,
        orchestration_session_id: authority.orchestration_session_id.clone(),
        shell_trace_session_id: authority.shell_trace_session_id.clone(),
        authority_revision: authority.authority_revision,
        origin: authority.origin.clone(),
        authoritative_participant_lineage: authority.authoritative_participant_lineage.clone(),
        active_authoritative_participant_id: authority.active_authoritative_participant_id.clone(),
        workspace_binding: authority.workspace_binding.clone(),
        world_binding: authority.world_binding.clone(),
        host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
        retained_worker_refs: authority.retained_worker_refs.clone(),
        internal_resume_handle_refs: authority.internal_resume_handle_refs.clone(),
        lifecycle_posture: authority.lifecycle_posture,
        current_policy_ref: authority.current_policy_ref.clone(),
        current_policy_revision: authority.current_policy_revision.clone(),
    }
}

fn successor_payload_value_from_intent(
    intent: &HostSessionTransitionIntentV3,
) -> HostSessionTransitionPayloadHashInputV1 {
    HostSessionTransitionPayloadHashInputV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        issuer_request_id: intent.issuer_request_id.clone(),
        mode: intent.mode,
        authority_precondition: intent.authority_precondition.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        caller: intent.caller.clone(),
        source_authoritative_participant_id: intent.source_authoritative_participant_id.clone(),
        target_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        target_participant_lease_token_ref: intent.target_participant_lease_token_ref.clone(),
        run_id: intent.run_id.clone(),
        resulting_authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        descriptor_ref: intent.descriptor_ref.clone(),
        host_attach_contract_ref: intent.host_attach_contract_ref.clone(),
        resume_handle_ref: intent.resume_handle_ref.clone(),
        transition_input_ref: intent.transition_input_ref.clone(),
        post_turn_disposition: intent.post_turn_disposition,
        transport_payload_ref: intent.transport_payload_ref.clone(),
        issued_at: intent.issued_at.clone(),
        expires_at: intent.expires_at.clone(),
    }
}

fn request_matches_intent(
    request: &IssueHostSessionTransitionRequestV1,
    intent: &HostSessionTransitionIntentV2,
) -> bool {
    request.mode == intent.mode
        && request.authority_precondition == intent.authority_precondition
        && request.shell_trace_session_id == intent.shell_trace_session_id
        && request.caller == intent.caller
        && intent.source_authoritative_participant_id.is_none()
        && request.target_authoritative_participant_id == intent.target_authoritative_participant_id
        && request.run_id == intent.run_id
        && request.resulting_authoritative_lineage == intent.resulting_authoritative_lineage
        && request.workspace_binding == intent.workspace_binding
        && request.world_binding == intent.world_binding
        && intent.resume_handle_ref.is_none()
        && intent.post_turn_disposition.is_none()
        && request.transition_input.is_some() == intent.transition_input_ref.is_some()
}

fn successor_request_matches_intent(
    request: &IssueSuccessorTransitionRequestV1,
    intent: &HostSessionTransitionIntentV3,
) -> bool {
    request.mode == intent.mode
        && request.authority_precondition == intent.authority_precondition
        && request.shell_trace_session_id == intent.shell_trace_session_id
        && request.caller == intent.caller
        && request.source_authoritative_participant_id == intent.source_authoritative_participant_id
        && request.target_authoritative_participant_id == intent.target_authoritative_participant_id
        && request.run_id == intent.run_id
        && request.resulting_authoritative_lineage == intent.resulting_authoritative_lineage
        && request.workspace_binding == intent.workspace_binding
        && request.world_binding == intent.world_binding
        && request.resume_handle_ref == intent.resume_handle_ref
        && request.post_turn_disposition == intent.post_turn_disposition
        && request.transition_input.is_some() == intent.transition_input_ref.is_some()
}

fn transport_value_from_successor_intent(
    intent: &HostSessionTransitionIntentV3,
) -> TransitionTransportPayloadObjectV1 {
    TransitionTransportPayloadObjectV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        mode: intent.mode,
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        caller: intent.caller.clone(),
        source_authoritative_participant_id: intent.source_authoritative_participant_id.clone(),
        target_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        target_participant_lease_token_ref: intent.target_participant_lease_token_ref.clone(),
        run_id: intent.run_id.clone(),
        resulting_authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        descriptor_ref: intent.descriptor_ref.clone(),
        host_attach_contract_ref: intent.host_attach_contract_ref.clone(),
        resume_handle_ref: intent.resume_handle_ref.clone(),
        transition_input_ref: intent.transition_input_ref.clone(),
        post_turn_disposition: intent.post_turn_disposition,
    }
}

fn verify_joined_objects(
    authority: &HostSessionAuthority,
    root: &StateRootV2,
    request: &IssueHostSessionTransitionRequestV1,
    intent: &HostSessionTransitionIntentV2,
) -> Result<(), TransitionProtocolError> {
    if matches!(
        intent.state,
        HostSessionTransitionIntentStateV2::Expired { .. }
    ) {
        verify_terminal_start(authority, root, intent)?;
    }
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    let lease = store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.target_participant_lease_token_ref,
        Some(&context),
    )
    .map_err(protocol_error)?;
    if lease != request.target_participant_lease_token {
        return Err(error(
            "participant lease token does not match stored issuance",
        ));
    }
    match (&request.transition_input, &intent.transition_input_ref) {
        (Some(expected), Some(reference)) => {
            let actual = store::read_typed_object_v2_opened(
                authority.trusted_root(),
                root.root_revision,
                reference,
                Some(&context),
            )
            .map_err(protocol_error)?;
            if actual != *expected {
                return Err(error("transition input does not match stored issuance"));
            }
        }
        (None, None) => {}
        _ => {
            return Err(error(
                "transition input presence does not match stored issuance",
            ))
        }
    }
    let parent = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V2(Box::new(
            intent.clone(),
        ))),
    };
    store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.transport_payload_ref,
        Some(&parent),
    )
    .map_err(protocol_error)?;
    let descriptor_bytes = store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.descriptor_ref,
        None,
    )
    .map_err(protocol_error)?;
    let descriptor: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(&descriptor_bytes).map_err(protocol_error)?;
    if descriptor.descriptor != request.start_contract.descriptor {
        return Err(error(
            "descriptor projection does not match stored issuance",
        ));
    }
    let attach_bytes = store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.host_attach_contract_ref,
        None,
    )
    .map_err(protocol_error)?;
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
    let policy_bytes = store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        &attach.contract.policy_ref,
        None,
    )
    .map_err(protocol_error)?;
    let policy: PolicyObjectHashInputV1 =
        canonical_json::from_slice(&policy_bytes).map_err(protocol_error)?;
    if attach.contract.backend_id != request.start_contract.descriptor.backend_id
        || attach.contract.execution_scope != request.start_contract.descriptor.execution_scope
        || attach.contract.protocol != request.start_contract.descriptor.protocol
        || attach.contract.descriptor_ref != intent.descriptor_ref
        || attach.contract.continuity_resume_handle_ref.is_some()
        || policy != request.start_contract.policy
        || attach.contract.capabilities != request.start_contract.capabilities
        || attach.contract.attach_launch_knobs != request.start_contract.launch_knobs
    {
        return Err(error(
            "attach or policy projection does not match stored issuance",
        ));
    }
    Ok(())
}

fn verify_successor_joined_objects(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    request: &IssueSuccessorTransitionRequestV1,
    intent: &HostSessionTransitionIntentV3,
) -> Result<(), TransitionProtocolError> {
    if matches!(
        intent.state,
        HostSessionTransitionIntentStateV3::Expired { .. }
    ) {
        verify_terminal_successor(authority, root, intent)?;
    }
    verify_successor_intent_objects(authority, root, intent)?;
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    let lease = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.target_participant_lease_token_ref,
        Some(&context),
    )
    .map_err(protocol_error)?;
    if lease != request.target_participant_lease_token {
        return Err(error(
            "participant lease token does not match stored issuance",
        ));
    }
    match (&request.transition_input, &intent.transition_input_ref) {
        (Some(expected), Some(reference)) => {
            let actual = store::read_typed_object_v2_or_v3_opened(
                authority.trusted_root(),
                root.root_revision,
                reference,
                Some(&context),
            )
            .map_err(protocol_error)?;
            if actual != *expected {
                return Err(error("transition input does not match stored issuance"));
            }
        }
        (None, None) => {}
        _ => {
            return Err(error(
                "transition input presence does not match stored issuance",
            ))
        }
    }
    Ok(())
}

fn verify_successor_intent_objects(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
) -> Result<(), TransitionProtocolError> {
    let descriptor_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.descriptor_ref,
        None,
    )
    .map_err(protocol_error)?;
    let _: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(&descriptor_bytes).map_err(protocol_error)?;
    let attach_bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.host_attach_contract_ref,
        None,
    )
    .map_err(protocol_error)?;
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
    store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &attach.contract.policy_ref,
        None,
    )
    .map_err(protocol_error)?;
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.target_participant_lease_token_ref,
        Some(&context),
    )
    .map_err(protocol_error)?;
    if let Some(reference) = intent.transition_input_ref.as_ref() {
        store::read_typed_object_v2_or_v3_opened(
            authority.trusted_root(),
            root.root_revision,
            reference,
            Some(&context),
        )
        .map_err(protocol_error)?;
    }
    if let Some(reference) = intent.resume_handle_ref.as_ref() {
        store::read_typed_object_v2_or_v3_opened(
            authority.trusted_root(),
            root.root_revision,
            reference,
            None,
        )
        .map_err(protocol_error)?;
    }
    let parent = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V3(Box::new(
            intent.clone(),
        ))),
    };
    let expected_payload = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&successor_payload_value_from_intent(intent))
            .map_err(protocol_error)?,
    };
    if expected_payload != intent.payload_commitment {
        return Err(error("transition payload commitment does not verify"));
    }
    if matches!(
        intent.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::Retained
    ) {
        let transport_bytes = store::read_typed_object_v2_or_v3_opened(
            authority.trusted_root(),
            root.root_revision,
            &intent.transport_payload_ref,
            Some(&parent),
        )
        .map_err(protocol_error)?;
        let transport: TransitionTransportPayloadObjectV1 =
            canonical_json::from_slice(&transport_bytes).map_err(protocol_error)?;
        if transport != transport_value_from_successor_intent(intent) {
            return Err(error(
                "successor transport payload projection is inconsistent",
            ));
        }
    }
    Ok(())
}

fn verify_terminal_start(
    authority: &HostSessionAuthority,
    root: &StateRootV2,
    intent: &HostSessionTransitionIntentV2,
) -> Result<(), TransitionProtocolError> {
    let (terminal_handoff_ref, recorded_at) = match &intent.state {
        HostSessionTransitionIntentStateV2::Expired {
            terminal_handoff_ref,
            expired_at,
        } => (terminal_handoff_ref, expired_at),
        _ => return Err(error("Start transition is not expired")),
    };
    let bytes = store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        terminal_handoff_ref,
        None,
    )
    .map_err(protocol_error)?;
    let terminal: TerminalHandoffHashInputV1 =
        canonical_json::from_slice(&bytes).map_err(protocol_error)?;
    let expected = TerminalHandoffHashInputV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        terminal_state: TerminalHandoffStateV1::Expired,
        application_result_ref: None,
        input_acceptance_ref: None,
        post_turn_completion_ref: None,
        post_turn_application_result_ref: None,
        recorded_at: recorded_at.clone(),
    };
    if terminal != expected {
        return Err(error("expired Start terminal handoff conflicts"));
    }
    verify_start_intent_objects(authority, root, intent)
}

fn validate_new_start_request(
    root: &StateRootV2,
    request: &IssueHostSessionTransitionRequestV1,
    ttl_seconds: i64,
) -> Result<(), TransitionProtocolError> {
    validate_start_request_shape(request, ttl_seconds)?;
    if request.mode != HostSessionTransitionModeV1::Start
        || request.authority_precondition != HostSessionAuthorityPreconditionV1::ExpectedAbsent
    {
        return Err(error("only Start with ExpectedAbsent is accepted by A1.2a"));
    }
    if ttl_seconds <= 0 || ttl_seconds > MAX_INTENT_TTL_SECONDS {
        return Err(error("transition intent TTL is outside the V1 bound"));
    }
    for value in [
        request.intent_id.as_str(),
        request.issuer_request_id.as_str(),
        request.orchestration_session_id.as_str(),
        request.shell_trace_session_id.as_str(),
        request.target_authoritative_participant_id.as_str(),
        request.run_id.as_str(),
    ] {
        if value.is_empty() {
            return Err(error("transition identity fields must be non-empty"));
        }
    }
    if request.source_authoritative_participant_id.is_some()
        || request.resulting_authoritative_lineage
            != [request.target_authoritative_participant_id.clone()]
        || request.target_participant_lease_token.is_empty()
    {
        return Err(error(
            "Start identity, lineage, or handoff fields are invalid",
        ));
    }
    if !matches!(
        request.caller.kind,
        HostSessionTransitionCallerKindV1::PublicCli | HostSessionTransitionCallerKindV1::Repl
    ) || request.caller.caller_participant_id.is_some()
        || request.caller.auto_attach_obligation_id.is_some()
        || request.caller.auto_attach_claim_owner.is_some()
    {
        return Err(error("Start caller identity is invalid"));
    }
    if request.workspace_binding.authority_store_id != root.authority_store_id
        || request.workspace_binding.authority_store_root != root.bootstrap_home
        || request
            .workspace_binding
            .workspace_root
            .physical_path
            .is_empty()
        || root.greenfield_namespace_certificate.authority_store_id != root.authority_store_id
        || root.greenfield_namespace_certificate.bootstrap_home != root.bootstrap_home
    {
        return Err(error(
            "Start workspace or greenfield certificate binding is invalid",
        ));
    }
    if root
        .session_namespace_map
        .contains_key(&request.orchestration_session_id)
        || root.transition_intent_map.contains_key(&request.intent_id)
        || root
            .issuer_request_index
            .contains_key(&request.issuer_request_id)
    {
        return Err(error(
            "Start namespace or issuance identity is already committed",
        ));
    }
    if root.transition_intent_map.values().any(|intent| {
        intent.target_authoritative_participant_id == request.target_authoritative_participant_id
            || intent
                .resulting_authoritative_lineage
                .contains(&request.target_authoritative_participant_id)
            || intent.run_id == request.run_id
            || intent.shell_trace_session_id == request.shell_trace_session_id
    }) {
        return Err(error("Start identity is already semantically occupied"));
    }
    let material = &request.start_contract;
    if material.descriptor.schema_version != 1
        || material.policy.schema_version != 1
        || material.descriptor.execution_scope != material.launch_knobs.requested_execution_scope
        || (material.descriptor.execution_scope == super::schema::AgentExecutionScopeV1::World
            && request.world_binding.is_none())
    {
        return Err(error(
            "Start descriptor, policy, launch, or world binding is invalid",
        ));
    }
    Ok(())
}

fn validate_start_request_shape(
    request: &IssueHostSessionTransitionRequestV1,
    ttl_seconds: i64,
) -> Result<(), TransitionProtocolError> {
    if request.mode != HostSessionTransitionModeV1::Start
        || request.authority_precondition != HostSessionAuthorityPreconditionV1::ExpectedAbsent
    {
        return Err(error("only Start with ExpectedAbsent is accepted by A1.2a"));
    }
    if ttl_seconds <= 0 || ttl_seconds > MAX_INTENT_TTL_SECONDS {
        return Err(error("transition intent TTL is outside the V1 bound"));
    }
    for value in [
        request.intent_id.as_str(),
        request.issuer_request_id.as_str(),
        request.orchestration_session_id.as_str(),
        request.shell_trace_session_id.as_str(),
        request.target_authoritative_participant_id.as_str(),
        request.run_id.as_str(),
    ] {
        if value.is_empty() {
            return Err(error("transition identity fields must be non-empty"));
        }
    }
    if request.source_authoritative_participant_id.is_some()
        || request.resulting_authoritative_lineage
            != [request.target_authoritative_participant_id.clone()]
        || request.target_participant_lease_token.is_empty()
    {
        return Err(error(
            "Start identity, lineage, or handoff fields are invalid",
        ));
    }
    if !matches!(
        request.caller.kind,
        HostSessionTransitionCallerKindV1::PublicCli | HostSessionTransitionCallerKindV1::Repl
    ) || request.caller.caller_participant_id.is_some()
        || request.caller.auto_attach_obligation_id.is_some()
        || request.caller.auto_attach_claim_owner.is_some()
    {
        return Err(error("Start caller identity is invalid"));
    }
    let material = &request.start_contract;
    if material.descriptor.schema_version != 1
        || material.policy.schema_version != 1
        || material.descriptor.execution_scope != material.launch_knobs.requested_execution_scope
        || (material.descriptor.execution_scope == super::schema::AgentExecutionScopeV1::World
            && request.world_binding.is_none())
    {
        return Err(error(
            "Start descriptor, policy, launch, or world binding is invalid",
        ));
    }
    if request
        .world_binding
        .as_ref()
        .is_some_and(|world| world.world_id.is_empty())
    {
        return Err(error("Start world binding is invalid"));
    }
    let descriptor = AgentDescriptorHashInputV1 {
        schema_version: 1,
        descriptor: material.descriptor.clone(),
    };
    canonical_sha256(&descriptor).map_err(protocol_error)?;
    canonical_sha256(&material.policy).map_err(protocol_error)?;
    Ok(())
}

fn payload_value_from_intent(
    intent: &HostSessionTransitionIntentV2,
) -> HostSessionTransitionPayloadHashInputV1 {
    HostSessionTransitionPayloadHashInputV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        issuer_request_id: intent.issuer_request_id.clone(),
        mode: intent.mode,
        authority_precondition: intent.authority_precondition.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        caller: intent.caller.clone(),
        source_authoritative_participant_id: intent.source_authoritative_participant_id.clone(),
        target_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        target_participant_lease_token_ref: intent.target_participant_lease_token_ref.clone(),
        run_id: intent.run_id.clone(),
        resulting_authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        descriptor_ref: intent.descriptor_ref.clone(),
        host_attach_contract_ref: intent.host_attach_contract_ref.clone(),
        resume_handle_ref: intent.resume_handle_ref.clone(),
        transition_input_ref: intent.transition_input_ref.clone(),
        post_turn_disposition: intent.post_turn_disposition,
        transport_payload_ref: intent.transport_payload_ref.clone(),
        issued_at: intent.issued_at.clone(),
        expires_at: intent.expires_at.clone(),
    }
}

fn insert_present_index(
    root: &mut StateRootV2,
    reference: &super::schema::AuthorityObjectRefV1,
    kind: AuthorityObjectKindV1,
    byte_length: u64,
) -> Result<(), TransitionProtocolError> {
    if reference.object_kind != kind || reference.schema_version != 1 {
        return Err(error("typed object kind or schema is inconsistent"));
    }
    let entry = AuthorityObjectIndexEntryV1 {
        schema_version: 1,
        ref_id: reference.ref_id.clone(),
        object_kind: kind,
        object_schema_version: 1,
        byte_length,
        storage_state: AuthorityObjectStorageStateV1::Present,
    };
    match root
        .object_index
        .insert(reference.ref_id.clone(), entry.clone())
    {
        None => Ok(()),
        Some(existing) if existing == entry => Ok(()),
        Some(_) => Err(error(
            "typed object index conflicts with an existing object",
        )),
    }
}

fn insert_present_index_v3(
    root: &mut StateRootV3,
    reference: &super::schema::AuthorityObjectRefV1,
    kind: AuthorityObjectKindV1,
    byte_length: u64,
) -> Result<(), TransitionProtocolError> {
    if reference.object_kind != kind || reference.schema_version != 1 {
        return Err(error("typed object kind or schema is inconsistent"));
    }
    let entry = AuthorityObjectIndexEntryV1 {
        schema_version: 1,
        ref_id: reference.ref_id.clone(),
        object_kind: kind,
        object_schema_version: 1,
        byte_length,
        storage_state: AuthorityObjectStorageStateV1::Present,
    };
    match root
        .object_index
        .insert(reference.ref_id.clone(), entry.clone())
    {
        None => Ok(()),
        Some(existing) if existing == entry => Ok(()),
        Some(_) => Err(error(
            "typed object index conflicts with an existing object",
        )),
    }
}

fn all_unique_participants(values: &[String]) -> bool {
    values
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == values.len()
}

fn add_seconds(
    timestamp: &TimestampV1,
    seconds: i64,
) -> Result<TimestampV1, TransitionProtocolError> {
    let parsed = DateTime::parse_from_rfc3339(timestamp.as_str()).map_err(protocol_error)?;
    let expires = parsed
        .checked_add_signed(Duration::seconds(seconds))
        .ok_or_else(|| error("transition expiry timestamp overflow"))?
        .with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Nanos, true);
    TimestampV1::parse(expires).map_err(protocol_error)
}

fn now_timestamp() -> Result<TimestampV1, TransitionProtocolError> {
    TimestampV1::parse(Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true))
        .map_err(protocol_error)
}

fn next_revision(current: u64, name: &'static str) -> Result<u64, TransitionProtocolError> {
    current
        .checked_add(1)
        .ok_or_else(|| error(format!("{name} revision overflow")))
}

fn stop_at(
    actual: Option<IssueCrashPointV1>,
    expected: IssueCrashPointV1,
) -> Result<(), TransitionProtocolError> {
    if actual == Some(expected) {
        Err(error("injected Start issuance interruption"))
    } else {
        Ok(())
    }
}

fn stop_application_at(
    actual: Option<ApplicationCrashPointV1>,
    expected: ApplicationCrashPointV1,
) -> Result<(), TransitionProtocolError> {
    if actual == Some(expected) {
        Err(error("injected Start application interruption"))
    } else {
        Ok(())
    }
}

fn stop_expiry_at(
    actual: Option<ExpiryCrashPointV1>,
    expected: ExpiryCrashPointV1,
) -> Result<(), TransitionProtocolError> {
    if actual == Some(expected) {
        Err(error("injected Start expiry interruption"))
    } else {
        Ok(())
    }
}

fn error(message: impl Into<String>) -> TransitionProtocolError {
    TransitionProtocolError(message.into())
}

fn protocol_error(error: impl fmt::Display) -> TransitionProtocolError {
    TransitionProtocolError(error.to_string())
}
