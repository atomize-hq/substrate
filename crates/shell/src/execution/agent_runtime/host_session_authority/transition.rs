//! Durable A1.2 host-session transition protocol.
//!
//! This module owns authority decisions. Helper plans are projections of records
//! created here and never participate in issuance, claim, or application truth.

use std::fmt;

use chrono::{DateTime, Duration, SecondsFormat, Utc};

use super::canonical_json;
use super::facade::HostSessionAuthority;
use super::hash::{canonical_object_bytes, canonical_sha256, CanonicalObjectHashInputV1};
use super::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, ApplicationResultHashInputV1,
    ApplicationResultPhaseV1, AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1,
    AuthorityObjectKindV1, DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1,
    HostAttachCapabilitiesV1, HostAttachContractHashInputV1, HostAttachContractV1,
    HostAttachLaunchKnobsV1, HostPostTurnDispositionV1, HostSessionAuthorityPreconditionV1,
    HostSessionPostureV1, HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
    HostSessionTransitionModeV1, HostSessionTransitionPayloadHashInputV1,
    HostSessionTransitionTerminalRejectionV1, InputAcceptanceHashInputV1, PolicyObjectHashInputV1,
    PostTurnCompletionHashInputV1, PostTurnCompletionOutcomeV1, TerminalHandoffHashInputV1,
    TerminalHandoffStateV1, TimestampV1, TransitionTransportPayloadObjectV1, WorkspaceBindingV1,
    WorldBindingV1,
};
use super::store::{
    ExpectedAuthorityRevisionV1, ExpectedRevisionsV1, ExpectedStartAuthorityBirthV1,
    ObjectVerificationContextV1,
};
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
    HostSessionPostTurnApplicationV1, HostSessionTransitionApplicationJournalV1,
    HostSessionTransitionInputHandoffV1, HostSessionTransitionIntentStateV1,
    HostSessionTransitionIntentV1, HostSessionTransitionTransportPayloadStateV1,
    InitialTransitionApplicationJournalV1, IssuerRequestIndexEntryV1, PostTurnApplicationJournalV1,
    SessionIdReservationV1, SessionIdTombstoneV1, SessionNamespaceRecordV1, StartTombstoneStateV1,
    StateRootV1,
};
use super::trusted_fs::TrustedWorkspaceRoot;

const SCHEMA_VERSION: u32 = 1;
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
    pub(crate) start_contract: Option<StartContractMaterialV1>,
    pub(crate) resume_handle_ref: Option<super::schema::AuthorityObjectRefV1>,
    pub(crate) transition_input: Option<Vec<u8>>,
    pub(crate) post_turn_disposition: Option<HostPostTurnDispositionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransitionIssueOutcomeV1 {
    Issued(HostSessionTransitionIntentV1),
    Joined(HostSessionTransitionIntentV1),
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
    Claimed(HostSessionTransitionIntentV1),
    Reclaimed(HostSessionTransitionIntentV1),
    Joined(HostSessionTransitionIntentV1),
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
    Applied(HostSessionTransitionIntentV1),
    Rejected(HostSessionTransitionIntentV1),
    Joined(HostSessionTransitionIntentV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApplicationCrashPointV1 {
    ResultPublished,
    RootCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcceptTransitionInputRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) expected_intent_revision: u64,
    pub(crate) input_ref: super::schema::AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) accepting_participant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InputAcceptanceOutcomeV1 {
    Accepted(HostSessionTransitionIntentV1),
    Joined(HostSessionTransitionIntentV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApplyPostTurnCompletionRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) expected_intent_revision: u64,
    pub(crate) expected_authority_revision: u64,
    pub(crate) outcome: PostTurnCompletionOutcomeV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PostTurnCompletionOutcomeResultV1 {
    Applied(HostSessionTransitionIntentV1),
    Joined(HostSessionTransitionIntentV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransitionIdentityRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransitionHandoffProjectionV1 {
    pub(crate) intent_revision: u64,
    pub(crate) transport_payload: TransitionTransportPayloadObjectV1,
    pub(crate) participant_lease_token: Option<Vec<u8>>,
    pub(crate) pending_input: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TransportReleaseCrashPointV1 {
    EligibleCommitted,
    PayloadDeleted,
    ReleasedCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransportReleaseOutcomeV1 {
    Released(HostSessionTransitionIntentV1),
    Joined(HostSessionTransitionIntentV1),
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
    Expired(HostSessionTransitionIntentV1),
    Joined(HostSessionTransitionIntentV1),
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
    pub(crate) fn issue_transition(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_transition_inner(request, now_timestamp()?, DEFAULT_INTENT_TTL_SECONDS, None)
    }

    #[cfg(test)]
    pub(crate) fn issue_transition_at(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_transition_inner(request, issued_at, ttl_seconds, None)
    }

    #[cfg(test)]
    pub(crate) fn issue_transition_at_with_crash_point(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
        crash_point: IssueCrashPointV1,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        self.issue_transition_inner(request, issued_at, ttl_seconds, Some(crash_point))
    }

    fn issue_transition_inner(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
        crash_point: Option<IssueCrashPointV1>,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        let attempted = self.issue_transition_once(request, issued_at, ttl_seconds, crash_point);
        if attempted.is_ok() || crash_point.is_some() {
            return attempted;
        }
        let original_error = attempted.unwrap_err();
        let observed = self.read_root().map_err(protocol_error)?;
        let Some(joined) = exact_issuance_join(&observed, request)? else {
            return Err(original_error);
        };
        let workspace = TrustedWorkspaceRoot::open_exact(&request.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        verify_joined_objects(self, &observed, request, &joined)?;
        Ok(TransitionIssueOutcomeV1::Joined(joined))
    }

    fn issue_transition_once(
        &self,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
        crash_point: Option<IssueCrashPointV1>,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        let current = self.read_root().map_err(protocol_error)?;
        let workspace = TrustedWorkspaceRoot::open_exact(&request.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        if let Some(joined) = exact_issuance_join(&current, request)? {
            workspace.revalidate().map_err(protocol_error)?;
            verify_joined_objects(self, &current, request, &joined)?;
            return Ok(TransitionIssueOutcomeV1::Joined(joined));
        }
        if request.mode != HostSessionTransitionModeV1::Start {
            return self.issue_parked_transition_once(
                &current,
                request,
                issued_at,
                ttl_seconds,
                crash_point,
                workspace,
            );
        }
        validate_new_start_request(&current, request, ttl_seconds)?;
        let expires_at = add_seconds(&issued_at, ttl_seconds)?;
        let material = request
            .start_contract
            .as_ref()
            .ok_or_else(|| error("Start requires complete contract material"))?;

        let object_context = ObjectVerificationContextV1 {
            intent_id: request.intent_id.clone(),
            run_id: request.run_id.clone(),
            parent_intent: None,
        };
        let descriptor_value = AgentDescriptorHashInputV1 {
            schema_version: SCHEMA_VERSION,
            descriptor: material.descriptor.clone(),
        };
        let descriptor_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::AgentDescriptor,
            CanonicalObjectHashInputV1::AgentDescriptor(&descriptor_value),
        )
        .map_err(protocol_error)?;
        let descriptor = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::AgentDescriptor,
                &descriptor_bytes,
                None,
            )
            .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::DescriptorPublished)?;

        let policy_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::Policy,
            CanonicalObjectHashInputV1::Policy(&material.policy),
        )
        .map_err(protocol_error)?;
        let policy = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::Policy,
                &policy_bytes,
                None,
            )
            .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::PolicyPublished)?;

        let attach_value = HostAttachContractHashInputV1 {
            schema_version: SCHEMA_VERSION,
            contract: HostAttachContractV1 {
                schema_version: SCHEMA_VERSION,
                backend_id: material.descriptor.backend_id.clone(),
                execution_scope: material.descriptor.execution_scope,
                protocol: material.descriptor.protocol.clone(),
                descriptor_ref: descriptor.reference.clone(),
                capabilities: material.capabilities.clone(),
                attach_launch_knobs: material.launch_knobs.clone(),
                policy_ref: policy.reference.clone(),
                continuity_resume_handle_ref: None,
            },
        };
        let attach_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::HostAttachContract,
            CanonicalObjectHashInputV1::HostAttachContract(&attach_value),
        )
        .map_err(protocol_error)?;
        let attach = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::HostAttachContract,
                &attach_bytes,
                None,
            )
            .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::AttachPublished)?;

        let lease = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::LeaseToken,
                &request.target_participant_lease_token,
                Some(&object_context),
            )
            .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::LeasePublished)?;
        let transition_input = match request.transition_input.as_deref() {
            Some(bytes) => Some(
                self.prepare_generated_object(
                    current.root_revision,
                    AuthorityObjectKindV1::TransitionInput,
                    bytes,
                    Some(&object_context),
                )
                .map_err(protocol_error)?,
            ),
            None => None,
        };
        if transition_input.is_some() {
            stop_at(crash_point, IssueCrashPointV1::InputPublished)?;
        }

        let transport_value = TransitionTransportPayloadObjectV1 {
            schema_version: SCHEMA_VERSION,
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
            descriptor_ref: descriptor.reference.clone(),
            host_attach_contract_ref: attach.reference.clone(),
            resume_handle_ref: request.resume_handle_ref.clone(),
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: request.post_turn_disposition,
        };
        let transport_bytes = canonical_json::to_vec(&transport_value).map_err(protocol_error)?;
        let transport_ref = self
            .allocate_sensitive_object_ref(
                current.root_revision,
                AuthorityObjectKindV1::TransitionTransportPayload,
                &transport_bytes,
                &object_context,
            )
            .map_err(protocol_error)?;

        let payload_value = HostSessionTransitionPayloadHashInputV1 {
            schema_version: SCHEMA_VERSION,
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
            descriptor_ref: descriptor.reference.clone(),
            host_attach_contract_ref: attach.reference.clone(),
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
        let intent = HostSessionTransitionIntentV1 {
            schema_version: SCHEMA_VERSION,
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
            descriptor_ref: descriptor.reference.clone(),
            host_attach_contract_ref: attach.reference.clone(),
            resume_handle_ref: request.resume_handle_ref.clone(),
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: request.post_turn_disposition,
            transport_payload_ref: transport_ref.clone(),
            payload_commitment: payload_commitment.clone(),
            issued_at: issued_at.clone(),
            expires_at,
            state: HostSessionTransitionIntentStateV1::Issued,
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
            parent_intent: Some(Box::new(intent.clone())),
        };
        self.prepare_typed_object(
            current.root_revision,
            &transport_ref,
            &transport_bytes,
            Some(&transport_context),
        )
        .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::TransportPublished)?;

        let mut proposed = current.clone();
        proposed.root_revision = proposed
            .root_revision
            .checked_add(1)
            .ok_or_else(|| error("authority root revision overflow"))?;
        proposed.session_namespace_map.insert(
            request.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::StartReservation(SessionIdReservationV1 {
                schema_version: SCHEMA_VERSION,
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
                schema_version: SCHEMA_VERSION,
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
        let commit = self.commit_transition_root(
            &current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: None,
            },
            &proposed,
            None,
            &workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            if let Some(joined) = exact_issuance_join(&observed, request)? {
                verify_joined_objects(self, &observed, request, &joined)?;
                return Ok(TransitionIssueOutcomeV1::Joined(joined));
            }
            return Err(protocol_error(commit_error));
        }
        stop_at(crash_point, IssueCrashPointV1::RootCommitted)?;
        workspace.revalidate().map_err(protocol_error)?;
        Ok(TransitionIssueOutcomeV1::Issued(intent))
    }

    fn issue_parked_transition_once(
        &self,
        current: &StateRootV1,
        request: &IssueHostSessionTransitionRequestV1,
        issued_at: TimestampV1,
        ttl_seconds: i64,
        crash_point: Option<IssueCrashPointV1>,
        workspace: TrustedWorkspaceRoot,
    ) -> Result<TransitionIssueOutcomeV1, TransitionProtocolError> {
        let authority_record = validate_new_parked_request(current, request, ttl_seconds)?;
        let attach_ref = authority_record
            .host_attach_contract_ref
            .as_ref()
            .ok_or_else(|| error("parked authority has no attach contract"))?;
        let attach_bytes = self
            .read_typed_object(current.root_revision, attach_ref, None)
            .map_err(protocol_error)?;
        let attach: HostAttachContractHashInputV1 =
            canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
        if attach.contract.descriptor_ref.object_kind != AuthorityObjectKindV1::AgentDescriptor
            || attach.contract.policy_ref
                != authority_record
                    .current_policy_ref
                    .clone()
                    .ok_or_else(|| error("parked authority has no current policy"))?
            || Some(attach.contract.policy_ref.clone()) != authority_record.current_policy_ref
            || attach.contract.continuity_resume_handle_ref != request.resume_handle_ref
        {
            return Err(error(
                "parked authority attach, descriptor, policy, or resume projection mismatches",
            ));
        }
        self.read_typed_object(current.root_revision, &attach.contract.descriptor_ref, None)
            .map_err(protocol_error)?;
        let policy_bytes = self
            .read_typed_object(current.root_revision, &attach.contract.policy_ref, None)
            .map_err(protocol_error)?;
        let policy: PolicyObjectHashInputV1 =
            canonical_json::from_slice(&policy_bytes).map_err(protocol_error)?;
        if Some(policy.policy_revision) != authority_record.current_policy_revision {
            return Err(error("parked authority policy revision mismatches"));
        }
        if let Some(resume_handle_ref) = request.resume_handle_ref.as_ref() {
            self.read_typed_object(current.root_revision, resume_handle_ref, None)
                .map_err(protocol_error)?;
            if !authority_record
                .internal_resume_handle_refs
                .contains(resume_handle_ref)
            {
                return Err(error("resume handle is not retained by current authority"));
            }
        }

        let expires_at = add_seconds(&issued_at, ttl_seconds)?;
        let object_context = ObjectVerificationContextV1 {
            intent_id: request.intent_id.clone(),
            run_id: request.run_id.clone(),
            parent_intent: None,
        };
        let lease = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::LeaseToken,
                &request.target_participant_lease_token,
                Some(&object_context),
            )
            .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::LeasePublished)?;
        let transition_input = match request.transition_input.as_deref() {
            Some(bytes) => Some(
                self.prepare_generated_object(
                    current.root_revision,
                    AuthorityObjectKindV1::TransitionInput,
                    bytes,
                    Some(&object_context),
                )
                .map_err(protocol_error)?,
            ),
            None => None,
        };
        if transition_input.is_some() {
            stop_at(crash_point, IssueCrashPointV1::InputPublished)?;
        }
        let transport_value = TransitionTransportPayloadObjectV1 {
            schema_version: SCHEMA_VERSION,
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
            descriptor_ref: attach.contract.descriptor_ref.clone(),
            host_attach_contract_ref: attach_ref.clone(),
            resume_handle_ref: request.resume_handle_ref.clone(),
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: request.post_turn_disposition,
        };
        let transport_bytes = canonical_json::to_vec(&transport_value).map_err(protocol_error)?;
        let transport_ref = self
            .allocate_sensitive_object_ref(
                current.root_revision,
                AuthorityObjectKindV1::TransitionTransportPayload,
                &transport_bytes,
                &object_context,
            )
            .map_err(protocol_error)?;
        let payload_value = HostSessionTransitionPayloadHashInputV1 {
            schema_version: SCHEMA_VERSION,
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
            descriptor_ref: attach.contract.descriptor_ref.clone(),
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
        let intent = HostSessionTransitionIntentV1 {
            schema_version: SCHEMA_VERSION,
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
            descriptor_ref: attach.contract.descriptor_ref.clone(),
            host_attach_contract_ref: attach_ref.clone(),
            resume_handle_ref: request.resume_handle_ref.clone(),
            transition_input_ref: transition_input
                .as_ref()
                .map(|object| object.reference.clone()),
            post_turn_disposition: request.post_turn_disposition,
            transport_payload_ref: transport_ref.clone(),
            payload_commitment: payload_commitment.clone(),
            issued_at: issued_at.clone(),
            expires_at,
            state: HostSessionTransitionIntentStateV1::Issued,
            input_handoff: transition_input.as_ref().map_or(
                HostSessionTransitionInputHandoffV1::NotApplicable,
                |input| HostSessionTransitionInputHandoffV1::Pending {
                    input_ref: input.reference.clone(),
                    run_id: request.run_id.clone(),
                },
            ),
            transport_payload_state: HostSessionTransitionTransportPayloadStateV1::Retained,
            updated_at: issued_at,
        };
        let transport_context = ObjectVerificationContextV1 {
            intent_id: request.intent_id.clone(),
            run_id: request.run_id.clone(),
            parent_intent: Some(Box::new(intent.clone())),
        };
        self.prepare_typed_object(
            current.root_revision,
            &transport_ref,
            &transport_bytes,
            Some(&transport_context),
        )
        .map_err(protocol_error)?;
        stop_at(crash_point, IssueCrashPointV1::TransportPublished)?;

        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        proposed
            .transition_intent_map
            .insert(request.intent_id.clone(), intent.clone());
        proposed.issuer_request_index.insert(
            request.issuer_request_id.clone(),
            IssuerRequestIndexEntryV1 {
                schema_version: SCHEMA_VERSION,
                issuer_request_id: request.issuer_request_id.clone(),
                orchestration_session_id: request.orchestration_session_id.clone(),
                intent_id: request.intent_id.clone(),
                payload_commitment,
            },
        );
        insert_present_index(
            &mut proposed,
            &lease.reference,
            AuthorityObjectKindV1::LeaseToken,
            lease.byte_length,
        )?;
        insert_present_index(
            &mut proposed,
            &transport_ref,
            AuthorityObjectKindV1::TransitionTransportPayload,
            transport_bytes.len() as u64,
        )?;
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
        let commit = self.commit_transition_root(
            current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: None,
            },
            &proposed,
            None,
            &workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            if let Some(joined) = exact_issuance_join(&observed, request)? {
                verify_joined_objects(self, &observed, request, &joined)?;
                return Ok(TransitionIssueOutcomeV1::Joined(joined));
            }
            return Err(protocol_error(commit_error));
        }
        stop_at(crash_point, IssueCrashPointV1::RootCommitted)?;
        Ok(TransitionIssueOutcomeV1::Issued(intent))
    }

    pub(crate) fn claim_transition(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
    ) -> Result<TransitionClaimOutcomeV1, TransitionProtocolError> {
        self.claim_transition_inner(request, now_timestamp()?, DEFAULT_CLAIM_LEASE_SECONDS)
    }

    #[cfg(test)]
    pub(crate) fn claim_transition_at(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
        claimed_at: TimestampV1,
        claim_lease_seconds: i64,
    ) -> Result<TransitionClaimOutcomeV1, TransitionProtocolError> {
        self.claim_transition_inner(request, claimed_at, claim_lease_seconds)
    }

    fn claim_transition_inner(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
        claimed_at: TimestampV1,
        claim_lease_seconds: i64,
    ) -> Result<TransitionClaimOutcomeV1, TransitionProtocolError> {
        let attempted =
            self.claim_transition_once(request, claimed_at.clone(), claim_lease_seconds);
        if attempted.is_ok() {
            return attempted;
        }
        let original_error = attempted.unwrap_err();
        let observed = self.read_root().map_err(protocol_error)?;
        let joined = exact_attempt(&observed, request)?;
        let workspace = TrustedWorkspaceRoot::open_exact(&joined.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        if let HostSessionTransitionIntentStateV1::Claimed {
            claim_id,
            claimant_attempt_id,
            claim_expires_at,
            ..
        } = &joined.state
        {
            if claim_id == &request.claim_id
                && claimant_attempt_id == &request.claimant_attempt_id
                && claimed_at.as_str() < claim_expires_at.as_str()
                && claimed_at.as_str() < joined.expires_at.as_str()
            {
                workspace.revalidate().map_err(protocol_error)?;
                verify_intent_objects(self, &observed, joined)?;
                return Ok(TransitionClaimOutcomeV1::Joined(joined.clone()));
            }
        }
        Err(original_error)
    }

    fn claim_transition_once(
        &self,
        request: &ClaimHostSessionTransitionRequestV1,
        claimed_at: TimestampV1,
        claim_lease_seconds: i64,
    ) -> Result<TransitionClaimOutcomeV1, TransitionProtocolError> {
        if claim_lease_seconds <= 0 || claim_lease_seconds > MAX_CLAIM_LEASE_SECONDS {
            return Err(error("transition claim lease is outside the V1 bound"));
        }
        let current = self.read_root().map_err(protocol_error)?;
        let intent = exact_attempt(&current, request)?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        if let HostSessionTransitionIntentStateV1::Claimed {
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
                workspace.revalidate().map_err(protocol_error)?;
                verify_intent_objects(self, &current, intent)?;
                return Ok(TransitionClaimOutcomeV1::Joined(intent.clone()));
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
        let (claim_revision, reclaimed) = match &intent.state {
            HostSessionTransitionIntentStateV1::Issued => (1, false),
            HostSessionTransitionIntentStateV1::Claimed {
                claim_revision,
                claim_expires_at,
                ..
            } if claimed_at.as_str() >= claim_expires_at.as_str() => (
                claim_revision
                    .checked_add(1)
                    .ok_or_else(|| error("transition claim revision overflow"))?,
                true,
            ),
            HostSessionTransitionIntentStateV1::Claimed { .. } => {
                return Err(error("transition intent has a current different claim"))
            }
            HostSessionTransitionIntentStateV1::Applied { .. }
            | HostSessionTransitionIntentStateV1::Rejected { .. }
            | HostSessionTransitionIntentStateV1::Expired { .. } => {
                return Err(error("terminal transition intent cannot be claimed"))
            }
        };
        let claim_expires_at = add_seconds(&claimed_at, claim_lease_seconds)?;
        if claim_expires_at.as_str() > intent.expires_at.as_str() {
            return Err(error("transition claim would exceed fixed intent expiry"));
        }
        verify_intent_precondition(&current, intent)?;
        verify_intent_objects(self, &current, intent)?;

        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .transition_intent_map
            .get_mut(&request.intent_id)
            .ok_or_else(|| error("transition intent disappeared during claim"))?;
        next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
        next.state = HostSessionTransitionIntentStateV1::Claimed {
            claim_id: request.claim_id.clone(),
            claimant_attempt_id: request.claimant_attempt_id.clone(),
            claim_revision,
            claimed_at: claimed_at.clone(),
            claim_expires_at,
        };
        next.updated_at = claimed_at.clone();
        proposed.validate().map_err(protocol_error)?;
        let commit = self.commit_transition_root(
            &current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: None,
            },
            &proposed,
            None,
            &workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            let joined = exact_attempt(&observed, request)?;
            if let HostSessionTransitionIntentStateV1::Claimed {
                claim_id,
                claimant_attempt_id,
                claim_expires_at,
                ..
            } = &joined.state
            {
                if claim_id == &request.claim_id
                    && claimant_attempt_id == &request.claimant_attempt_id
                    && claimed_at.as_str() < claim_expires_at.as_str()
                    && claimed_at.as_str() < joined.expires_at.as_str()
                {
                    verify_intent_objects(self, &observed, joined)?;
                    return Ok(TransitionClaimOutcomeV1::Joined(joined.clone()));
                }
            }
            return Err(protocol_error(commit_error));
        }
        let committed = proposed.transition_intent_map[&request.intent_id].clone();
        if reclaimed {
            Ok(TransitionClaimOutcomeV1::Reclaimed(committed))
        } else {
            Ok(TransitionClaimOutcomeV1::Claimed(committed))
        }
    }

    pub(crate) fn apply_transition(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_transition_inner(request, now_timestamp()?, None)
    }

    #[cfg(test)]
    pub(crate) fn apply_transition_at(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_transition_inner(request, applied_at, None)
    }

    #[cfg(test)]
    pub(crate) fn apply_transition_at_with_crash_point(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
        crash_point: ApplicationCrashPointV1,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        self.apply_transition_inner(request, applied_at, Some(crash_point))
    }

    fn apply_transition_inner(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
        crash_point: Option<ApplicationCrashPointV1>,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        let attempted = self.apply_transition_once(request, applied_at, crash_point);
        if attempted.is_ok() || crash_point.is_some() {
            return attempted;
        }
        let original_error = attempted.unwrap_err();
        let observed = self.read_root().map_err(protocol_error)?;
        let intent = exact_application_attempt(&observed, request)?;
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV1::Applied { .. }
        ) {
            verify_applied_transition(self, &observed, intent, request)?;
            return Ok(TransitionApplicationOutcomeV1::Joined(intent.clone()));
        }
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV1::Rejected { .. }
        ) {
            verify_terminal_transition(self, &observed, intent)?;
            return Ok(TransitionApplicationOutcomeV1::Joined(intent.clone()));
        }
        Err(original_error)
    }

    fn apply_transition_once(
        &self,
        request: &ApplyHostSessionTransitionRequestV1,
        applied_at: TimestampV1,
        crash_point: Option<ApplicationCrashPointV1>,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        let current = self.read_root().map_err(protocol_error)?;
        let intent = exact_application_attempt(&current, request)?;
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV1::Applied { .. }
        ) {
            verify_applied_transition(self, &current, intent, request)?;
            return Ok(TransitionApplicationOutcomeV1::Joined(intent.clone()));
        }
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV1::Rejected { .. }
        ) {
            verify_terminal_transition(self, &current, intent)?;
            return Ok(TransitionApplicationOutcomeV1::Joined(intent.clone()));
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("transition application intent revision is stale"));
        }
        let (claim_id, claim_revision, claim_expires_at) = match &intent.state {
            HostSessionTransitionIntentStateV1::Claimed {
                claim_id,
                claim_revision,
                claim_expires_at,
                ..
            } => (claim_id, *claim_revision, claim_expires_at),
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
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        verify_intent_objects(self, &current, intent)?;
        if verify_intent_precondition(&current, intent).is_err() {
            let reason = transition_rejection_reason(&current, intent)?;
            return self.reject_claimed_transition(
                &current, intent, request, applied_at, reason, &workspace,
            );
        }
        let attach_bytes = self
            .read_typed_object(
                current.root_revision,
                &intent.host_attach_contract_ref,
                None,
            )
            .map_err(protocol_error)?;
        let attach: HostAttachContractHashInputV1 =
            canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
        let policy_bytes = self
            .read_typed_object(current.root_revision, &attach.contract.policy_ref, None)
            .map_err(protocol_error)?;
        let policy: PolicyObjectHashInputV1 =
            canonical_json::from_slice(&policy_bytes).map_err(protocol_error)?;
        let (authority_revision_before, authority_revision_after, authority_record) = if intent.mode
            == HostSessionTransitionModeV1::Start
        {
            (
                None,
                1,
                DurableSessionAuthorityV1 {
                    schema_version: SCHEMA_VERSION,
                    orchestration_session_id: intent.orchestration_session_id.clone(),
                    shell_trace_session_id: intent.shell_trace_session_id.clone(),
                    authority_revision: 1,
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
                    internal_resume_handle_refs: intent
                        .resume_handle_ref
                        .clone()
                        .into_iter()
                        .collect(),
                    lifecycle_posture: HostSessionPostureV1::ActiveAttached,
                    current_policy_ref: Some(attach.contract.policy_ref.clone()),
                    current_policy_revision: Some(policy.policy_revision.clone()),
                    updated_at: applied_at.clone(),
                },
            )
        } else {
            let current_authority = match current
                .session_namespace_map
                .get(&intent.orchestration_session_id)
            {
                Some(SessionNamespaceRecordV1::Authority(authority)) => authority.as_ref(),
                _ => return Err(error("successor application has no durable authority")),
            };
            let next_authority_revision =
                next_revision(current_authority.authority_revision, "durable authority")?;
            let mut next_authority = current_authority.clone();
            next_authority.authority_revision = next_authority_revision;
            next_authority.authoritative_participant_lineage =
                intent.resulting_authoritative_lineage.clone();
            next_authority.active_authoritative_participant_id =
                Some(intent.target_authoritative_participant_id.clone());
            next_authority.host_attach_contract_ref = Some(intent.host_attach_contract_ref.clone());
            next_authority.lifecycle_posture = HostSessionPostureV1::ActiveAttached;
            next_authority.current_policy_ref = Some(attach.contract.policy_ref.clone());
            next_authority.current_policy_revision = Some(policy.policy_revision.clone());
            next_authority.updated_at = applied_at.clone();
            (
                Some(current_authority.authority_revision),
                next_authority_revision,
                next_authority,
            )
        };
        let authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&authority_record))
                .map_err(protocol_error)?,
        };
        let application_value = ApplicationResultHashInputV1 {
            schema_version: SCHEMA_VERSION,
            intent_id: intent.intent_id.clone(),
            mode: intent.mode,
            run_id: intent.run_id.clone(),
            phase: ApplicationResultPhaseV1::InitialTransition {
                authority_revision_before,
                authority_revision_after,
                active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
                resulting_posture: HostSessionPostureV1::ActiveAttached,
                authority_record_commitment: authority_record_commitment.clone(),
                post_turn_pending_run_id: (intent.mode
                    == HostSessionTransitionModeV1::ResumeOneTurn)
                    .then(|| intent.run_id.clone()),
            },
            applied_at: applied_at.clone(),
        };
        let application_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::ApplicationResult,
            CanonicalObjectHashInputV1::ApplicationResult(&application_value),
        )
        .map_err(protocol_error)?;
        let application = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::ApplicationResult,
                &application_bytes,
                None,
            )
            .map_err(protocol_error)?;
        stop_application_at(crash_point, ApplicationCrashPointV1::ResultPublished)?;

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
        next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
        next.state = HostSessionTransitionIntentStateV1::Applied {
            claim_id: request.claim_id.clone(),
            authority_revision_before,
            authority_revision_after,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: authority_record_commitment.clone(),
            application_result_ref: application.reference.clone(),
            post_turn: Box::new(
                if intent.mode == HostSessionTransitionModeV1::ResumeOneTurn {
                    HostSessionPostTurnApplicationV1::Pending {
                        expected_run_id: intent.run_id.clone(),
                        expected_authority_revision: authority_revision_after,
                    }
                } else {
                    HostSessionPostTurnApplicationV1::NotApplicable
                },
            ),
            applied_at: applied_at.clone(),
        };
        next.updated_at = applied_at.clone();
        proposed.application_journal.insert(
            intent.intent_id.clone(),
            HostSessionTransitionApplicationJournalV1 {
                schema_version: SCHEMA_VERSION,
                intent_id: intent.intent_id.clone(),
                initial_application: InitialTransitionApplicationJournalV1 {
                    authority_revision_before,
                    authority_revision_after,
                    authority_record_commitment: authority_record_commitment.clone(),
                    application_result_ref: application.reference.clone(),
                    applied_at,
                },
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
        let start_birth = (intent.mode == HostSessionTransitionModeV1::Start).then(|| {
            ExpectedStartAuthorityBirthV1 {
                orchestration_session_id: intent.orchestration_session_id.clone(),
                intent_id: intent.intent_id.clone(),
                issuer_request_id: intent.issuer_request_id.clone(),
                payload_commitment: intent.payload_commitment.clone(),
            }
        });
        let expected_authority =
            authority_revision_before.map(|revision| ExpectedAuthorityRevisionV1 {
                orchestration_session_id: intent.orchestration_session_id.clone(),
                authority_revision: revision,
            });
        let commit = self.commit_transition_root(
            &current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: expected_authority,
            },
            &proposed,
            start_birth.as_ref(),
            &workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            let joined = exact_application_attempt(&observed, request)?;
            if matches!(
                joined.state,
                HostSessionTransitionIntentStateV1::Applied { .. }
            ) {
                verify_applied_transition(self, &observed, joined, request)?;
                return Ok(TransitionApplicationOutcomeV1::Joined(joined.clone()));
            }
            return Err(protocol_error(commit_error));
        }
        stop_application_at(crash_point, ApplicationCrashPointV1::RootCommitted)?;
        Ok(TransitionApplicationOutcomeV1::Applied(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }

    fn reject_claimed_transition(
        &self,
        current: &StateRootV1,
        intent: &HostSessionTransitionIntentV1,
        request: &ApplyHostSessionTransitionRequestV1,
        rejected_at: TimestampV1,
        reason: HostSessionTransitionTerminalRejectionV1,
        workspace: &TrustedWorkspaceRoot,
    ) -> Result<TransitionApplicationOutcomeV1, TransitionProtocolError> {
        let terminal_value = TerminalHandoffHashInputV1 {
            schema_version: SCHEMA_VERSION,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            terminal_state: TerminalHandoffStateV1::Rejected { reason },
            application_result_ref: None,
            input_acceptance_ref: None,
            post_turn_completion_ref: None,
            post_turn_application_result_ref: None,
            recorded_at: rejected_at.clone(),
        };
        let terminal_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::TerminalHandoff,
            CanonicalObjectHashInputV1::TerminalHandoff(&terminal_value),
        )
        .map_err(protocol_error)?;
        let terminal = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::TerminalHandoff,
                &terminal_bytes,
                None,
            )
            .map_err(protocol_error)?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during rejection"))?;
        next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
        next.state = HostSessionTransitionIntentStateV1::Rejected {
            reason,
            terminal_handoff_ref: terminal.reference.clone(),
            rejected_at: rejected_at.clone(),
        };
        if let HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id } =
            &next.input_handoff
        {
            next.input_handoff = HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                input_ref: input_ref.clone(),
                run_id: run_id.clone(),
                terminal_handoff_ref: terminal.reference.clone(),
                terminal_at: rejected_at.clone(),
            };
        }
        next.updated_at = rejected_at.clone();
        if next.mode == HostSessionTransitionModeV1::Start {
            proposed.session_namespace_map.insert(
                next.orchestration_session_id.clone(),
                SessionNamespaceRecordV1::StartTombstone(SessionIdTombstoneV1 {
                    schema_version: SCHEMA_VERSION,
                    orchestration_session_id: next.orchestration_session_id.clone(),
                    intent_id: next.intent_id.clone(),
                    issuer_request_id: next.issuer_request_id.clone(),
                    payload_commitment: next.payload_commitment.clone(),
                    terminal_state: StartTombstoneStateV1::Rejected { reason },
                    terminal_handoff_ref: terminal.reference.clone(),
                    tombstoned_at: rejected_at,
                }),
            );
        }
        insert_present_index(
            &mut proposed,
            &terminal.reference,
            AuthorityObjectKindV1::TerminalHandoff,
            terminal.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        let commit = self.commit_transition_root(
            current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: None,
            },
            &proposed,
            None,
            workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            let joined = exact_application_attempt(&observed, request)?;
            if matches!(
                joined.state,
                HostSessionTransitionIntentStateV1::Rejected { .. }
            ) {
                verify_terminal_transition(self, &observed, joined)?;
                return Ok(TransitionApplicationOutcomeV1::Joined(joined.clone()));
            }
            return Err(protocol_error(commit_error));
        }
        Ok(TransitionApplicationOutcomeV1::Rejected(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn accept_transition_input(
        &self,
        request: &AcceptTransitionInputRequestV1,
    ) -> Result<InputAcceptanceOutcomeV1, TransitionProtocolError> {
        self.accept_transition_input_inner(request, now_timestamp()?)
    }

    #[cfg(test)]
    pub(crate) fn accept_transition_input_at(
        &self,
        request: &AcceptTransitionInputRequestV1,
        accepted_at: TimestampV1,
    ) -> Result<InputAcceptanceOutcomeV1, TransitionProtocolError> {
        self.accept_transition_input_inner(request, accepted_at)
    }

    fn accept_transition_input_inner(
        &self,
        request: &AcceptTransitionInputRequestV1,
        accepted_at: TimestampV1,
    ) -> Result<InputAcceptanceOutcomeV1, TransitionProtocolError> {
        let current = self.read_root().map_err(protocol_error)?;
        let intent = exact_input_attempt(&current, request)?;
        if let HostSessionTransitionInputHandoffV1::Accepted {
            input_ref,
            run_id,
            acceptance_ref,
            accepted_at: committed_at,
        } = &intent.input_handoff
        {
            if input_ref == &request.input_ref
                && run_id == &request.run_id
                && verify_input_acceptance(
                    self,
                    &current,
                    intent,
                    acceptance_ref,
                    &request.accepting_participant_id,
                    committed_at,
                )
                .is_ok()
            {
                return Ok(InputAcceptanceOutcomeV1::Joined(intent.clone()));
            }
            return Err(error("committed input acceptance conflicts"));
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("input acceptance intent revision is stale"));
        }
        let HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id } =
            &intent.input_handoff
        else {
            return Err(error("transition input is not pending acceptance"));
        };
        if input_ref != &request.input_ref || run_id != &request.run_id {
            return Err(error("input acceptance ref or run mismatches"));
        }
        let (
            HostSessionTransitionIntentStateV1::Applied {
                authority_revision_after,
                active_authoritative_participant_id,
                ..
            },
            Some(SessionNamespaceRecordV1::Authority(authority_record)),
        ) = (
            &intent.state,
            current
                .session_namespace_map
                .get(&intent.orchestration_session_id),
        )
        else {
            return Err(error("input acceptance requires applied durable authority"));
        };
        if active_authoritative_participant_id != &request.accepting_participant_id
            || authority_record.authority_revision != *authority_revision_after
            || authority_record
                .active_authoritative_participant_id
                .as_ref()
                != Some(&request.accepting_participant_id)
            || accepted_at.as_str() < intent.updated_at.as_str()
        {
            return Err(error(
                "input acceptance participant, authority revision, or time mismatches",
            ));
        }
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        self.read_typed_object(current.root_revision, input_ref, Some(&context))
            .map_err(protocol_error)?;
        let acceptance_value = InputAcceptanceHashInputV1 {
            schema_version: SCHEMA_VERSION,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            input_ref: input_ref.clone(),
            accepting_participant_id: request.accepting_participant_id.clone(),
            accepted_at: accepted_at.clone(),
        };
        let acceptance_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::InputAcceptance,
            CanonicalObjectHashInputV1::InputAcceptance(&acceptance_value),
        )
        .map_err(protocol_error)?;
        let acceptance = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::InputAcceptance,
                &acceptance_bytes,
                None,
            )
            .map_err(protocol_error)?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during input acceptance"))?;
        next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
        next.input_handoff = HostSessionTransitionInputHandoffV1::Accepted {
            input_ref: input_ref.clone(),
            run_id: run_id.clone(),
            acceptance_ref: acceptance.reference.clone(),
            accepted_at: accepted_at.clone(),
        };
        next.updated_at = accepted_at;
        insert_present_index(
            &mut proposed,
            &acceptance.reference,
            AuthorityObjectKindV1::InputAcceptance,
            acceptance.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        let commit = self.commit_transition_root(
            &current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: None,
            },
            &proposed,
            None,
            &workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            let joined = exact_input_attempt(&observed, request)?;
            if let HostSessionTransitionInputHandoffV1::Accepted {
                acceptance_ref,
                accepted_at,
                ..
            } = &joined.input_handoff
            {
                verify_input_acceptance(
                    self,
                    &observed,
                    joined,
                    acceptance_ref,
                    &request.accepting_participant_id,
                    accepted_at,
                )?;
                return Ok(InputAcceptanceOutcomeV1::Joined(joined.clone()));
            }
            return Err(protocol_error(commit_error));
        }
        Ok(InputAcceptanceOutcomeV1::Accepted(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn apply_post_turn_completion(
        &self,
        request: &ApplyPostTurnCompletionRequestV1,
    ) -> Result<PostTurnCompletionOutcomeResultV1, TransitionProtocolError> {
        self.apply_post_turn_completion_inner(request, now_timestamp()?)
    }

    #[cfg(test)]
    pub(crate) fn apply_post_turn_completion_at(
        &self,
        request: &ApplyPostTurnCompletionRequestV1,
        completed_at: TimestampV1,
    ) -> Result<PostTurnCompletionOutcomeResultV1, TransitionProtocolError> {
        self.apply_post_turn_completion_inner(request, completed_at)
    }

    fn apply_post_turn_completion_inner(
        &self,
        request: &ApplyPostTurnCompletionRequestV1,
        completed_at: TimestampV1,
    ) -> Result<PostTurnCompletionOutcomeResultV1, TransitionProtocolError> {
        let current = self.read_root().map_err(protocol_error)?;
        let intent = exact_post_turn_attempt(&current, request)?;
        let HostSessionTransitionIntentStateV1::Applied { post_turn, .. } = &intent.state else {
            return Err(error("post-turn completion requires applied transition"));
        };
        if let HostSessionPostTurnApplicationV1::Applied {
            completion_ref,
            application_result_ref,
            ..
        } = post_turn.as_ref()
        {
            verify_post_turn_application(
                self,
                &current,
                intent,
                request,
                completion_ref,
                application_result_ref,
            )?;
            return Ok(PostTurnCompletionOutcomeResultV1::Joined(intent.clone()));
        }
        if request.expected_intent_revision != intent.intent_revision {
            return Err(error("post-turn intent revision is stale"));
        }
        let HostSessionPostTurnApplicationV1::Pending {
            expected_run_id,
            expected_authority_revision,
        } = post_turn.as_ref()
        else {
            return Err(error("transition has no pending post-turn application"));
        };
        if intent.mode != HostSessionTransitionModeV1::ResumeOneTurn
            || expected_run_id != &intent.run_id
            || *expected_authority_revision != request.expected_authority_revision
            || !matches!(
                intent.input_handoff,
                HostSessionTransitionInputHandoffV1::Accepted { .. }
            )
            || completed_at.as_str() < intent.updated_at.as_str()
        {
            return Err(error(
                "post-turn mode, run, input acceptance, revision, or time mismatches",
            ));
        }
        let Some(SessionNamespaceRecordV1::Authority(current_authority)) = current
            .session_namespace_map
            .get(&intent.orchestration_session_id)
        else {
            return Err(error("post-turn application has no durable authority"));
        };
        if current_authority.authority_revision != request.expected_authority_revision
            || current_authority
                .active_authoritative_participant_id
                .as_ref()
                != Some(&intent.target_authoritative_participant_id)
        {
            return Err(error("post-turn durable authority is stale or substituted"));
        }
        let resulting_posture = match request.outcome {
            PostTurnCompletionOutcomeV1::ResumableClean => HostSessionPostureV1::ParkedResumable,
            PostTurnCompletionOutcomeV1::TerminalClean
            | PostTurnCompletionOutcomeV1::TerminalFailure => HostSessionPostureV1::Terminal,
        };
        let completion_value = PostTurnCompletionHashInputV1 {
            schema_version: SCHEMA_VERSION,
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            authority_revision_observed: current_authority.authority_revision,
            outcome: request.outcome,
            completed_at: completed_at.clone(),
        };
        let completion_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::PostTurnCompletion,
            CanonicalObjectHashInputV1::PostTurnCompletion(&completion_value),
        )
        .map_err(protocol_error)?;
        let completion = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::PostTurnCompletion,
                &completion_bytes,
                None,
            )
            .map_err(protocol_error)?;
        let mut next_authority = current_authority.as_ref().clone();
        next_authority.authority_revision =
            next_revision(current_authority.authority_revision, "durable authority")?;
        next_authority.lifecycle_posture = resulting_posture;
        next_authority.updated_at = completed_at.clone();
        let authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(&next_authority))
                .map_err(protocol_error)?,
        };
        let application_value = ApplicationResultHashInputV1 {
            schema_version: SCHEMA_VERSION,
            intent_id: intent.intent_id.clone(),
            mode: intent.mode,
            run_id: intent.run_id.clone(),
            phase: ApplicationResultPhaseV1::PostTurn {
                completion_ref: completion.reference.clone(),
                authority_revision_before: current_authority.authority_revision,
                authority_revision_after: next_authority.authority_revision,
                active_authoritative_participant_id: intent
                    .target_authoritative_participant_id
                    .clone(),
                resulting_posture,
                authority_record_commitment: authority_record_commitment.clone(),
            },
            applied_at: completed_at.clone(),
        };
        let application_bytes = canonical_object_bytes(
            AuthorityObjectKindV1::ApplicationResult,
            CanonicalObjectHashInputV1::ApplicationResult(&application_value),
        )
        .map_err(protocol_error)?;
        let application = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::ApplicationResult,
                &application_bytes,
                None,
            )
            .map_err(protocol_error)?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        proposed.session_namespace_map.insert(
            intent.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::Authority(Box::new(next_authority.clone())),
        );
        let next = proposed
            .transition_intent_map
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during post-turn"))?;
        let HostSessionTransitionIntentStateV1::Applied { post_turn, .. } = &mut next.state else {
            return Err(error("transition application state disappeared"));
        };
        *post_turn = Box::new(HostSessionPostTurnApplicationV1::Applied {
            completion_ref: Box::new(completion.reference.clone()),
            authority_revision_before: current_authority.authority_revision,
            authority_revision_after: next_authority.authority_revision,
            resulting_posture,
            application_result_ref: Box::new(application.reference.clone()),
            applied_at: completed_at.clone(),
        });
        next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
        next.updated_at = completed_at.clone();
        proposed
            .application_journal
            .get_mut(&intent.intent_id)
            .ok_or_else(|| error("post-turn application has no initial journal"))?
            .post_turn_application = Some(PostTurnApplicationJournalV1 {
            completion_ref: completion.reference.clone(),
            authority_revision_before: current_authority.authority_revision,
            authority_revision_after: next_authority.authority_revision,
            authority_record_commitment,
            application_result_ref: application.reference.clone(),
            applied_at: completed_at,
        });
        for (object, kind, byte_length) in [
            (
                &completion.reference,
                AuthorityObjectKindV1::PostTurnCompletion,
                completion.byte_length,
            ),
            (
                &application.reference,
                AuthorityObjectKindV1::ApplicationResult,
                application.byte_length,
            ),
        ] {
            insert_present_index(&mut proposed, object, kind, byte_length)?;
        }
        proposed.validate().map_err(protocol_error)?;
        let commit = self.commit_transition_root(
            &current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: Some(ExpectedAuthorityRevisionV1 {
                    orchestration_session_id: intent.orchestration_session_id.clone(),
                    authority_revision: current_authority.authority_revision,
                }),
            },
            &proposed,
            None,
            &workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            let joined = exact_post_turn_attempt(&observed, request)?;
            let HostSessionTransitionIntentStateV1::Applied { post_turn, .. } = &joined.state
            else {
                return Err(protocol_error(commit_error));
            };
            if let HostSessionPostTurnApplicationV1::Applied {
                completion_ref,
                application_result_ref,
                ..
            } = post_turn.as_ref()
            {
                verify_post_turn_application(
                    self,
                    &observed,
                    joined,
                    request,
                    completion_ref,
                    application_result_ref,
                )?;
                return Ok(PostTurnCompletionOutcomeResultV1::Joined(joined.clone()));
            }
            return Err(protocol_error(commit_error));
        }
        Ok(PostTurnCompletionOutcomeResultV1::Applied(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn reproject_transition_handoff(
        &self,
        request: &TransitionIdentityRequestV1,
    ) -> Result<TransitionHandoffProjectionV1, TransitionProtocolError> {
        let root = self.read_root().map_err(protocol_error)?;
        let intent = exact_identity_attempt(&root, request)?;
        if !matches!(
            intent.transport_payload_state,
            HostSessionTransitionTransportPayloadStateV1::Retained
        ) {
            return Err(error("transition transport payload is not retained"));
        }
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        workspace.revalidate().map_err(protocol_error)?;
        verify_intent_objects(self, &root, intent)?;
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        let parent = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: Some(Box::new(intent.clone())),
        };
        let transport_bytes = self
            .read_typed_object(
                root.root_revision,
                &intent.transport_payload_ref,
                Some(&parent),
            )
            .map_err(protocol_error)?;
        let transport_payload: TransitionTransportPayloadObjectV1 =
            canonical_json::from_slice(&transport_bytes).map_err(protocol_error)?;
        let participant_lease_token = matches!(
            intent.state,
            HostSessionTransitionIntentStateV1::Issued
                | HostSessionTransitionIntentStateV1::Claimed { .. }
        )
        .then(|| {
            self.read_typed_object(
                root.root_revision,
                &intent.target_participant_lease_token_ref,
                Some(&context),
            )
            .map_err(protocol_error)
        })
        .transpose()?;
        let pending_input = match (&intent.input_handoff, intent.transition_input_ref.as_ref()) {
            (HostSessionTransitionInputHandoffV1::Pending { .. }, Some(input_ref)) => Some(
                self.read_typed_object(root.root_revision, input_ref, Some(&context))
                    .map_err(protocol_error)?,
            ),
            _ => None,
        };
        Ok(TransitionHandoffProjectionV1 {
            intent_revision: intent.intent_revision,
            transport_payload,
            participant_lease_token,
            pending_input,
        })
    }

    pub(crate) fn release_transition_transport(
        &self,
        request: &TransitionIdentityRequestV1,
    ) -> Result<TransportReleaseOutcomeV1, TransitionProtocolError> {
        self.release_transition_transport_inner(request, now_timestamp()?, None)
    }

    #[cfg(test)]
    pub(crate) fn release_transition_transport_at(
        &self,
        request: &TransitionIdentityRequestV1,
        released_at: TimestampV1,
    ) -> Result<TransportReleaseOutcomeV1, TransitionProtocolError> {
        self.release_transition_transport_inner(request, released_at, None)
    }

    #[cfg(test)]
    pub(crate) fn release_transition_transport_at_with_crash_point(
        &self,
        request: &TransitionIdentityRequestV1,
        released_at: TimestampV1,
        crash_point: TransportReleaseCrashPointV1,
    ) -> Result<TransportReleaseOutcomeV1, TransitionProtocolError> {
        self.release_transition_transport_inner(request, released_at, Some(crash_point))
    }

    fn release_transition_transport_inner(
        &self,
        request: &TransitionIdentityRequestV1,
        released_at: TimestampV1,
        crash_point: Option<TransportReleaseCrashPointV1>,
    ) -> Result<TransportReleaseOutcomeV1, TransitionProtocolError> {
        let mut current = self.read_root().map_err(protocol_error)?;
        let mut intent = exact_identity_attempt(&current, request)?.clone();
        if matches!(
            intent.transport_payload_state,
            HostSessionTransitionTransportPayloadStateV1::Released { .. }
        ) {
            return Ok(TransportReleaseOutcomeV1::Joined(intent));
        }
        if matches!(
            intent.transport_payload_state,
            HostSessionTransitionTransportPayloadStateV1::Retained
        ) {
            verify_transport_release_eligibility(&current, &intent)?;
            let terminal = terminal_handoff_for_release(self, &current, &intent, &released_at)?;
            let mut proposed = current.clone();
            proposed.root_revision = next_revision(current.root_revision, "authority root")?;
            let next = proposed
                .transition_intent_map
                .get_mut(&intent.intent_id)
                .ok_or_else(|| error("transition intent disappeared during release eligibility"))?;
            next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
            next.transport_payload_state =
                HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                    terminal_handoff_ref: terminal.reference.clone(),
                };
            next.updated_at = released_at.clone();
            let index = proposed
                .object_index
                .get_mut(&intent.transport_payload_ref.ref_id)
                .ok_or_else(|| error("transport payload index disappeared"))?;
            index.storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
                terminal_handoff_ref: terminal.reference.clone(),
            };
            insert_present_index(
                &mut proposed,
                &terminal.reference,
                AuthorityObjectKindV1::TerminalHandoff,
                terminal.byte_length,
            )?;
            proposed.validate().map_err(protocol_error)?;
            let workspace =
                TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
                    .map_err(protocol_error)?;
            self.commit_transition_root(
                &current,
                &ExpectedRevisionsV1 {
                    root_revision: current.root_revision,
                    authority: None,
                },
                &proposed,
                None,
                &workspace,
            )
            .map_err(protocol_error)?;
            stop_release_at(crash_point, TransportReleaseCrashPointV1::EligibleCommitted)?;
            current = proposed;
            intent = current.transition_intent_map[&request.intent_id].clone();
        }
        let HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
            terminal_handoff_ref,
        } = &intent.transport_payload_state
        else {
            return Err(error("transport release state is invalid"));
        };
        self.delete_release_eligible_transport(&current, &intent.intent_id)
            .map_err(protocol_error)?;
        stop_release_at(crash_point, TransportReleaseCrashPointV1::PayloadDeleted)?;
        let observed = self.read_root().map_err(protocol_error)?;
        let observed_intent = exact_identity_attempt(&observed, request)?;
        if let HostSessionTransitionTransportPayloadStateV1::Released { .. } =
            observed_intent.transport_payload_state
        {
            return Ok(TransportReleaseOutcomeV1::Joined(observed_intent.clone()));
        }
        if observed_intent.transport_payload_state != intent.transport_payload_state {
            return Err(error("transport release state changed concurrently"));
        }
        let workspace =
            TrustedWorkspaceRoot::open_exact(&observed_intent.workspace_binding.workspace_root)
                .map_err(protocol_error)?;
        let mut proposed = observed.clone();
        proposed.root_revision = next_revision(observed.root_revision, "authority root")?;
        let next = proposed
            .transition_intent_map
            .get_mut(&observed_intent.intent_id)
            .ok_or_else(|| error("transition intent disappeared during release"))?;
        next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
        next.transport_payload_state = HostSessionTransitionTransportPayloadStateV1::Released {
            terminal_handoff_ref: terminal_handoff_ref.clone(),
            released_at: released_at.clone(),
        };
        next.updated_at = released_at.clone();
        proposed
            .object_index
            .get_mut(&observed_intent.transport_payload_ref.ref_id)
            .ok_or_else(|| error("transport payload index disappeared during release"))?
            .storage_state = AuthorityObjectStorageStateV1::Released {
            terminal_handoff_ref: terminal_handoff_ref.clone(),
            released_at,
        };
        proposed.validate().map_err(protocol_error)?;
        self.commit_transition_root(
            &observed,
            &ExpectedRevisionsV1 {
                root_revision: observed.root_revision,
                authority: None,
            },
            &proposed,
            None,
            &workspace,
        )
        .map_err(protocol_error)?;
        stop_release_at(crash_point, TransportReleaseCrashPointV1::ReleasedCommitted)?;
        Ok(TransportReleaseOutcomeV1::Released(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }

    pub(crate) fn expire_transition(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        self.expire_transition_inner(request, now_timestamp()?)
    }

    #[cfg(test)]
    pub(crate) fn expire_transition_at(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        self.expire_transition_inner(request, expired_at)
    }

    fn expire_transition_inner(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        let attempted = self.expire_transition_once(request, expired_at);
        if attempted.is_ok() {
            return attempted;
        }
        let original_error = attempted.unwrap_err();
        let observed = self.read_root().map_err(protocol_error)?;
        let joined = exact_expiry_attempt(&observed, request)?;
        if matches!(
            joined.state,
            HostSessionTransitionIntentStateV1::Expired { .. }
        ) {
            let workspace =
                TrustedWorkspaceRoot::open_exact(&joined.workspace_binding.workspace_root)
                    .map_err(protocol_error)?;
            workspace.revalidate().map_err(protocol_error)?;
            verify_intent_objects(self, &observed, joined)?;
            return Ok(TransitionTerminalOutcomeV1::Joined(joined.clone()));
        }
        Err(original_error)
    }

    fn expire_transition_once(
        &self,
        request: &ExpireHostSessionTransitionRequestV1,
        expired_at: TimestampV1,
    ) -> Result<TransitionTerminalOutcomeV1, TransitionProtocolError> {
        let current = self.read_root().map_err(protocol_error)?;
        let intent = exact_expiry_attempt(&current, request)?;
        let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
            .map_err(protocol_error)?;
        if matches!(
            intent.state,
            HostSessionTransitionIntentStateV1::Expired { .. }
        ) {
            verify_intent_objects(self, &current, intent)?;
            return Ok(TransitionTerminalOutcomeV1::Joined(intent.clone()));
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
            HostSessionTransitionIntentStateV1::Issued
                | HostSessionTransitionIntentStateV1::Claimed { .. }
        ) || current.application_journal.contains_key(&intent.intent_id)
        {
            return Err(error("transition intent cannot be expired"));
        }
        verify_intent_precondition(&current, intent)?;
        verify_intent_objects(self, &current, intent)?;
        let terminal_value = TerminalHandoffHashInputV1 {
            schema_version: SCHEMA_VERSION,
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
        let terminal = self
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::TerminalHandoff,
                &terminal_bytes,
                None,
            )
            .map_err(protocol_error)?;

        let mut proposed = current.clone();
        proposed.root_revision = next_revision(current.root_revision, "authority root")?;
        let next = proposed
            .transition_intent_map
            .get_mut(&request.intent_id)
            .ok_or_else(|| error("transition intent disappeared during expiry"))?;
        next.intent_revision = next_revision(next.intent_revision, "transition intent")?;
        next.state = HostSessionTransitionIntentStateV1::Expired {
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
        if next.mode == HostSessionTransitionModeV1::Start {
            proposed.session_namespace_map.insert(
                next.orchestration_session_id.clone(),
                SessionNamespaceRecordV1::StartTombstone(SessionIdTombstoneV1 {
                    schema_version: SCHEMA_VERSION,
                    orchestration_session_id: next.orchestration_session_id.clone(),
                    intent_id: next.intent_id.clone(),
                    issuer_request_id: next.issuer_request_id.clone(),
                    payload_commitment: next.payload_commitment.clone(),
                    terminal_state: StartTombstoneStateV1::Expired,
                    terminal_handoff_ref: terminal.reference.clone(),
                    tombstoned_at: expired_at,
                }),
            );
        }
        insert_present_index(
            &mut proposed,
            &terminal.reference,
            AuthorityObjectKindV1::TerminalHandoff,
            terminal.byte_length,
        )?;
        proposed.validate().map_err(protocol_error)?;
        let commit = self.commit_transition_root(
            &current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: None,
            },
            &proposed,
            None,
            &workspace,
        );
        if let Err(commit_error) = commit {
            let observed = self.read_root().map_err(protocol_error)?;
            let joined = exact_expiry_attempt(&observed, request)?;
            if matches!(
                joined.state,
                HostSessionTransitionIntentStateV1::Expired { .. }
            ) {
                verify_intent_objects(self, &observed, joined)?;
                return Ok(TransitionTerminalOutcomeV1::Joined(joined.clone()));
            }
            return Err(protocol_error(commit_error));
        }
        Ok(TransitionTerminalOutcomeV1::Expired(
            proposed.transition_intent_map[&request.intent_id].clone(),
        ))
    }
}

fn exact_issuance_join(
    root: &StateRootV1,
    request: &IssueHostSessionTransitionRequestV1,
) -> Result<Option<HostSessionTransitionIntentV1>, TransitionProtocolError> {
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
            let payload = payload_value_from_intent(intent);
            let expected = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&payload).map_err(protocol_error)?,
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

fn exact_attempt<'root>(
    root: &'root StateRootV1,
    request: &ClaimHostSessionTransitionRequestV1,
) -> Result<&'root HostSessionTransitionIntentV1, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(&request.issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != request.issuer_request_id
        || index.intent_id != request.intent_id
        || index.payload_commitment != request.payload_commitment
        || intent.payload_commitment != request.payload_commitment
    {
        return Err(error(
            "transition claim identity or payload commitment mismatch",
        ));
    }
    Ok(intent)
}

fn exact_expiry_attempt<'root>(
    root: &'root StateRootV1,
    request: &ExpireHostSessionTransitionRequestV1,
) -> Result<&'root HostSessionTransitionIntentV1, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(&request.issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != request.issuer_request_id
        || index.intent_id != request.intent_id
        || index.payload_commitment != request.payload_commitment
        || intent.payload_commitment != request.payload_commitment
    {
        return Err(error(
            "transition expiry identity or payload commitment mismatch",
        ));
    }
    Ok(intent)
}

fn exact_application_attempt<'root>(
    root: &'root StateRootV1,
    request: &ApplyHostSessionTransitionRequestV1,
) -> Result<&'root HostSessionTransitionIntentV1, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(&request.issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != request.issuer_request_id
        || index.intent_id != request.intent_id
        || index.payload_commitment != request.payload_commitment
        || intent.payload_commitment != request.payload_commitment
    {
        return Err(error(
            "transition application identity or payload commitment mismatch",
        ));
    }
    Ok(intent)
}

fn exact_identity_attempt<'root>(
    root: &'root StateRootV1,
    request: &TransitionIdentityRequestV1,
) -> Result<&'root HostSessionTransitionIntentV1, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(&request.issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != request.issuer_request_id
        || index.intent_id != request.intent_id
        || intent.payload_commitment != request.payload_commitment
        || index.payload_commitment != request.payload_commitment
    {
        return Err(error(
            "transition identity or payload commitment mismatches",
        ));
    }
    Ok(intent)
}

fn verify_transport_release_eligibility(
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
) -> Result<(), TransitionProtocolError> {
    let input_complete = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::NotApplicable
        | HostSessionTransitionInputHandoffV1::Accepted { .. } => true,
        HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance { .. } => matches!(
            intent.state,
            HostSessionTransitionIntentStateV1::Rejected { .. }
                | HostSessionTransitionIntentStateV1::Expired { .. }
        ),
        HostSessionTransitionInputHandoffV1::Pending { .. } => false,
    };
    if !input_complete {
        return Err(error(
            "transition input handoff is not terminal for release",
        ));
    }
    match &intent.state {
        HostSessionTransitionIntentStateV1::Rejected { .. }
        | HostSessionTransitionIntentStateV1::Expired { .. } => {
            if root.application_journal.contains_key(&intent.intent_id) {
                return Err(error(
                    "terminal transition unexpectedly has application proof",
                ));
            }
        }
        HostSessionTransitionIntentStateV1::Applied {
            application_result_ref,
            post_turn,
            ..
        } => {
            if root
                .application_journal
                .get(&intent.intent_id)
                .map(|journal| &journal.initial_application.application_result_ref)
                != Some(application_result_ref)
            {
                return Err(error("applied transition has no exact application proof"));
            }
            match intent.mode {
                HostSessionTransitionModeV1::Start | HostSessionTransitionModeV1::Attach
                    if matches!(
                        post_turn.as_ref(),
                        HostSessionPostTurnApplicationV1::NotApplicable
                    ) => {}
                HostSessionTransitionModeV1::ResumeOneTurn
                    if matches!(
                        post_turn.as_ref(),
                        HostSessionPostTurnApplicationV1::Applied { .. }
                    ) => {}
                _ => return Err(error("transition post-turn is not terminal for release")),
            }
        }
        HostSessionTransitionIntentStateV1::Issued
        | HostSessionTransitionIntentStateV1::Claimed { .. } => {
            return Err(error("nonterminal transition cannot release transport"))
        }
    }
    Ok(())
}

fn terminal_handoff_for_release(
    authority: &HostSessionAuthority,
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
    recorded_at: &TimestampV1,
) -> Result<super::store::GeneratedObjectV1, TransitionProtocolError> {
    let existing_terminal = match &intent.state {
        HostSessionTransitionIntentStateV1::Rejected {
            terminal_handoff_ref,
            ..
        }
        | HostSessionTransitionIntentStateV1::Expired {
            terminal_handoff_ref,
            ..
        } => Some(terminal_handoff_ref),
        _ => None,
    };
    if let Some(reference) = existing_terminal {
        verify_terminal_transition(authority, root, intent)?;
        let index = root
            .object_index
            .get(&reference.ref_id)
            .ok_or_else(|| error("terminal handoff index is missing"))?;
        return Ok(super::store::GeneratedObjectV1 {
            reference: reference.clone(),
            byte_length: index.byte_length,
        });
    }
    let HostSessionTransitionIntentStateV1::Applied {
        application_result_ref,
        post_turn,
        ..
    } = &intent.state
    else {
        return Err(error("release has no terminal application state"));
    };
    let input_acceptance_ref = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::Accepted { acceptance_ref, .. } => {
            Some(acceptance_ref.clone())
        }
        _ => None,
    };
    let (post_turn_completion_ref, post_turn_application_result_ref) = match post_turn.as_ref() {
        HostSessionPostTurnApplicationV1::Applied {
            completion_ref,
            application_result_ref,
            ..
        } => (
            Some(completion_ref.as_ref().clone()),
            Some(application_result_ref.as_ref().clone()),
        ),
        _ => (None, None),
    };
    let terminal_value = TerminalHandoffHashInputV1 {
        schema_version: SCHEMA_VERSION,
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        terminal_state: TerminalHandoffStateV1::Applied,
        application_result_ref: Some(application_result_ref.clone()),
        input_acceptance_ref,
        post_turn_completion_ref,
        post_turn_application_result_ref,
        recorded_at: recorded_at.clone(),
    };
    let bytes = canonical_object_bytes(
        AuthorityObjectKindV1::TerminalHandoff,
        CanonicalObjectHashInputV1::TerminalHandoff(&terminal_value),
    )
    .map_err(protocol_error)?;
    authority
        .prepare_generated_object(
            root.root_revision,
            AuthorityObjectKindV1::TerminalHandoff,
            &bytes,
            None,
        )
        .map_err(protocol_error)
}

fn exact_input_attempt<'root>(
    root: &'root StateRootV1,
    request: &AcceptTransitionInputRequestV1,
) -> Result<&'root HostSessionTransitionIntentV1, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(&request.issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != request.issuer_request_id
        || index.intent_id != request.intent_id
        || intent.payload_commitment != request.payload_commitment
        || index.payload_commitment != request.payload_commitment
    {
        return Err(error("input acceptance transition identity mismatches"));
    }
    Ok(intent)
}

fn exact_post_turn_attempt<'root>(
    root: &'root StateRootV1,
    request: &ApplyPostTurnCompletionRequestV1,
) -> Result<&'root HostSessionTransitionIntentV1, TransitionProtocolError> {
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("unknown transition intent"))?;
    let index = root
        .issuer_request_index
        .get(&request.issuer_request_id)
        .ok_or_else(|| error("unknown transition issuer request"))?;
    if intent.issuer_request_id != request.issuer_request_id
        || index.intent_id != request.intent_id
        || intent.payload_commitment != request.payload_commitment
        || index.payload_commitment != request.payload_commitment
    {
        return Err(error("post-turn transition identity mismatches"));
    }
    Ok(intent)
}

fn verify_post_turn_application(
    authority: &HostSessionAuthority,
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
    request: &ApplyPostTurnCompletionRequestV1,
    completion_ref: &super::schema::AuthorityObjectRefV1,
    application_result_ref: &super::schema::AuthorityObjectRefV1,
) -> Result<(), TransitionProtocolError> {
    let HostSessionTransitionIntentStateV1::Applied { post_turn, .. } = &intent.state else {
        return Err(error("post-turn transition is not applied"));
    };
    let HostSessionPostTurnApplicationV1::Applied {
        authority_revision_before,
        authority_revision_after,
        resulting_posture,
        applied_at,
        ..
    } = post_turn.as_ref()
    else {
        return Err(error("post-turn result is not committed"));
    };
    if *authority_revision_before != request.expected_authority_revision {
        return Err(error("post-turn committed revision conflicts"));
    }
    let completion_bytes = authority
        .read_typed_object(root.root_revision, completion_ref, None)
        .map_err(protocol_error)?;
    let completion: PostTurnCompletionHashInputV1 =
        canonical_json::from_slice(&completion_bytes).map_err(protocol_error)?;
    let expected_completion = PostTurnCompletionHashInputV1 {
        schema_version: SCHEMA_VERSION,
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        authority_revision_observed: request.expected_authority_revision,
        outcome: request.outcome,
        completed_at: completion.completed_at.clone(),
    };
    if completion != expected_completion || completion.completed_at != *applied_at {
        return Err(error("post-turn completion object conflicts"));
    }
    let Some(SessionNamespaceRecordV1::Authority(current_authority)) = root
        .session_namespace_map
        .get(&intent.orchestration_session_id)
    else {
        return Err(error("post-turn result has no durable authority"));
    };
    if current_authority.authority_revision < *authority_revision_after {
        return Err(error("post-turn authority revision regressed"));
    }
    let journal = root
        .application_journal
        .get(&intent.intent_id)
        .and_then(|journal| journal.post_turn_application.as_ref())
        .ok_or_else(|| error("post-turn result has no application journal"))?;
    let application_bytes = authority
        .read_typed_object(root.root_revision, application_result_ref, None)
        .map_err(protocol_error)?;
    let application: ApplicationResultHashInputV1 =
        canonical_json::from_slice(&application_bytes).map_err(protocol_error)?;
    let expected_application = ApplicationResultHashInputV1 {
        schema_version: SCHEMA_VERSION,
        intent_id: intent.intent_id.clone(),
        mode: intent.mode,
        run_id: intent.run_id.clone(),
        phase: ApplicationResultPhaseV1::PostTurn {
            completion_ref: completion_ref.clone(),
            authority_revision_before: *authority_revision_before,
            authority_revision_after: *authority_revision_after,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: *resulting_posture,
            authority_record_commitment: journal.authority_record_commitment.clone(),
        },
        applied_at: applied_at.clone(),
    };
    if application != expected_application {
        return Err(error("post-turn application result object conflicts"));
    }
    if current_authority.authority_revision == *authority_revision_after {
        let commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(current_authority))
                .map_err(protocol_error)?,
        };
        if commitment != journal.authority_record_commitment {
            return Err(error("post-turn current authority commitment mismatches"));
        }
    }
    Ok(())
}

fn verify_input_acceptance(
    authority: &HostSessionAuthority,
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
    acceptance_ref: &super::schema::AuthorityObjectRefV1,
    accepting_participant_id: &str,
    accepted_at: &TimestampV1,
) -> Result<(), TransitionProtocolError> {
    let input_ref = intent
        .transition_input_ref
        .as_ref()
        .ok_or_else(|| error("accepted transition has no input ref"))?;
    let bytes = authority
        .read_typed_object(root.root_revision, acceptance_ref, None)
        .map_err(protocol_error)?;
    let acceptance: InputAcceptanceHashInputV1 =
        canonical_json::from_slice(&bytes).map_err(protocol_error)?;
    let expected = InputAcceptanceHashInputV1 {
        schema_version: SCHEMA_VERSION,
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        input_ref: input_ref.clone(),
        accepting_participant_id: accepting_participant_id.to_owned(),
        accepted_at: accepted_at.clone(),
    };
    if acceptance != expected {
        return Err(error("input acceptance object conflicts"));
    }
    Ok(())
}

fn transition_rejection_reason(
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
) -> Result<HostSessionTransitionTerminalRejectionV1, TransitionProtocolError> {
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision,
        authority_record_commitment,
        ..
    } = &intent.authority_precondition
    else {
        return Ok(if intent.mode == HostSessionTransitionModeV1::Start {
            HostSessionTransitionTerminalRejectionV1::AuthorityPreconditionNoLongerHolds
        } else {
            HostSessionTransitionTerminalRejectionV1::InvalidCommittedModePrecondition
        });
    };
    let Some(SessionNamespaceRecordV1::Authority(authority)) = root
        .session_namespace_map
        .get(&intent.orchestration_session_id)
    else {
        return Ok(HostSessionTransitionTerminalRejectionV1::AuthorityPreconditionNoLongerHolds);
    };
    if authority.authority_revision != *authority_revision {
        return Ok(HostSessionTransitionTerminalRejectionV1::StaleAuthorityRevision);
    }
    let commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&authority_hash_input(authority)).map_err(protocol_error)?,
    };
    Ok(if commitment != *authority_record_commitment {
        HostSessionTransitionTerminalRejectionV1::AuthorityRecordCommitmentMismatch
    } else {
        HostSessionTransitionTerminalRejectionV1::AuthorityPreconditionNoLongerHolds
    })
}

fn verify_terminal_transition(
    authority: &HostSessionAuthority,
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
) -> Result<(), TransitionProtocolError> {
    let (reason, terminal_handoff_ref, recorded_at) = match &intent.state {
        HostSessionTransitionIntentStateV1::Rejected {
            reason,
            terminal_handoff_ref,
            rejected_at,
        } => (Some(*reason), terminal_handoff_ref, rejected_at),
        HostSessionTransitionIntentStateV1::Expired {
            terminal_handoff_ref,
            expired_at,
        } => (None, terminal_handoff_ref, expired_at),
        _ => return Err(error("transition is not terminal")),
    };
    let bytes = authority
        .read_typed_object(root.root_revision, terminal_handoff_ref, None)
        .map_err(protocol_error)?;
    let terminal: TerminalHandoffHashInputV1 =
        canonical_json::from_slice(&bytes).map_err(protocol_error)?;
    let expected_state = reason.map_or(TerminalHandoffStateV1::Expired, |reason| {
        TerminalHandoffStateV1::Rejected { reason }
    });
    if terminal.intent_id != intent.intent_id
        || terminal.run_id != intent.run_id
        || terminal.payload_commitment != intent.payload_commitment
        || terminal.terminal_state != expected_state
        || terminal.recorded_at != *recorded_at
    {
        return Err(error("terminal transition handoff conflicts"));
    }
    let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
        .map_err(protocol_error)?;
    workspace.revalidate().map_err(protocol_error)?;
    verify_intent_objects(authority, root, intent)
}

fn verify_applied_transition(
    authority: &HostSessionAuthority,
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
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
        post_turn,
        applied_at,
    ) = match &intent.state {
        HostSessionTransitionIntentStateV1::Applied {
            claim_id,
            authority_revision_before,
            authority_revision_after,
            active_authoritative_participant_id,
            resulting_posture,
            authority_record_commitment,
            application_result_ref,
            post_turn,
            applied_at,
        } => (
            claim_id,
            authority_revision_before,
            *authority_revision_after,
            active_authoritative_participant_id,
            *resulting_posture,
            authority_record_commitment,
            application_result_ref,
            post_turn,
            applied_at,
        ),
        _ => return Err(error("transition application has not committed")),
    };
    let revisions_match = match (&intent.mode, &intent.authority_precondition) {
        (
            HostSessionTransitionModeV1::Start,
            HostSessionAuthorityPreconditionV1::ExpectedAbsent,
        ) => authority_revision_before.is_none() && authority_revision_after == 1,
        (
            HostSessionTransitionModeV1::Attach | HostSessionTransitionModeV1::ResumeOneTurn,
            HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision, ..
            },
        ) => {
            authority_revision_before == &Some(*authority_revision)
                && authority_revision
                    .checked_add(1)
                    .is_some_and(|next| next == authority_revision_after)
        }
        _ => false,
    };
    let post_turn_matches = match intent.mode {
        HostSessionTransitionModeV1::Start | HostSessionTransitionModeV1::Attach => {
            matches!(
                post_turn.as_ref(),
                HostSessionPostTurnApplicationV1::NotApplicable
            )
        }
        HostSessionTransitionModeV1::ResumeOneTurn => matches!(
            post_turn.as_ref(),
            HostSessionPostTurnApplicationV1::Pending {
                expected_run_id,
                expected_authority_revision,
            } if expected_run_id == &intent.run_id
                && *expected_authority_revision == authority_revision_after
        ),
    };
    if claim_id != &request.claim_id
        || !revisions_match
        || active_participant != &intent.target_authoritative_participant_id
        || resulting_posture != HostSessionPostureV1::ActiveAttached
        || !post_turn_matches
    {
        return Err(error("committed transition application result conflicts"));
    }
    let SessionNamespaceRecordV1::Authority(authority_record) = root
        .session_namespace_map
        .get(&intent.orchestration_session_id)
        .ok_or_else(|| error("committed transition has no authority record"))?
    else {
        return Err(error("committed transition has no durable authority"));
    };
    if authority_record.authority_revision < authority_revision_after {
        return Err(error(
            "committed durable authority regressed below application",
        ));
    }
    if authority_record.authority_revision == authority_revision_after {
        let current_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(authority_record))
                .map_err(protocol_error)?,
        };
        if &current_commitment != authority_record_commitment {
            return Err(error("committed durable authority commitment mismatch"));
        }
    }
    let application_bytes = authority
        .read_typed_object(root.root_revision, application_result_ref, None)
        .map_err(protocol_error)?;
    let application: ApplicationResultHashInputV1 =
        canonical_json::from_slice(&application_bytes).map_err(protocol_error)?;
    let expected_application = ApplicationResultHashInputV1 {
        schema_version: SCHEMA_VERSION,
        intent_id: intent.intent_id.clone(),
        mode: intent.mode,
        run_id: intent.run_id.clone(),
        phase: ApplicationResultPhaseV1::InitialTransition {
            authority_revision_before: *authority_revision_before,
            authority_revision_after,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: authority_record_commitment.clone(),
            post_turn_pending_run_id: (intent.mode == HostSessionTransitionModeV1::ResumeOneTurn)
                .then(|| intent.run_id.clone()),
        },
        applied_at: applied_at.clone(),
    };
    if application != expected_application {
        return Err(error("committed application result object conflicts"));
    }
    let workspace = TrustedWorkspaceRoot::open_exact(&intent.workspace_binding.workspace_root)
        .map_err(protocol_error)?;
    workspace.revalidate().map_err(protocol_error)?;
    verify_intent_objects(authority, root, intent)
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

fn verify_intent_precondition(
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
) -> Result<(), TransitionProtocolError> {
    match (intent.mode, &intent.authority_precondition) {
        (
            HostSessionTransitionModeV1::Start,
            HostSessionAuthorityPreconditionV1::ExpectedAbsent,
        ) => match root
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
        },
        (
            HostSessionTransitionModeV1::Attach | HostSessionTransitionModeV1::ResumeOneTurn,
            HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision,
                authority_record_commitment,
                active_authoritative_participant_id,
                authoritative_lineage_commitment,
                lifecycle_posture,
            },
        ) => {
            let Some(SessionNamespaceRecordV1::Authority(authority)) = root
                .session_namespace_map
                .get(&intent.orchestration_session_id)
            else {
                return Err(error("successor intent has no durable authority"));
            };
            let expected_authority = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&authority_hash_input(authority))
                    .map_err(protocol_error)?,
            };
            let expected_lineage = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
                    schema_version: SCHEMA_VERSION,
                    orchestration_session_id: authority.orchestration_session_id.clone(),
                    participant_ids: authority.authoritative_participant_lineage.clone(),
                })
                .map_err(protocol_error)?,
            };
            if authority.authority_revision != *authority_revision
                || expected_authority != *authority_record_commitment
                || authority.active_authoritative_participant_id.as_ref()
                    != Some(active_authoritative_participant_id)
                || expected_lineage != *authoritative_lineage_commitment
                || authority.lifecycle_posture != *lifecycle_posture
                || intent.source_authoritative_participant_id.as_ref()
                    != Some(active_authoritative_participant_id)
                || intent.shell_trace_session_id != authority.shell_trace_session_id
                || intent.workspace_binding != authority.workspace_binding
                || intent.world_binding != authority.world_binding
                || authority.authoritative_participant_lineage.last()
                    != Some(active_authoritative_participant_id)
            {
                return Err(error("successor authority precondition no longer holds"));
            }
            Ok(())
        }
        _ => Err(error(
            "transition authority precondition is not implemented for this mode",
        )),
    }
}

fn verify_intent_objects(
    authority: &HostSessionAuthority,
    root: &StateRootV1,
    intent: &HostSessionTransitionIntentV1,
) -> Result<(), TransitionProtocolError> {
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    authority
        .read_typed_object(root.root_revision, &intent.descriptor_ref, None)
        .map_err(protocol_error)?;
    let attach_bytes = authority
        .read_typed_object(root.root_revision, &intent.host_attach_contract_ref, None)
        .map_err(protocol_error)?;
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
    if attach.contract.descriptor_ref != intent.descriptor_ref
        || attach.contract.continuity_resume_handle_ref != intent.resume_handle_ref
    {
        return Err(error(
            "transition attach contract projection is inconsistent",
        ));
    }
    authority
        .read_typed_object(root.root_revision, &attach.contract.policy_ref, None)
        .map_err(protocol_error)?;
    authority
        .read_typed_object(
            root.root_revision,
            &intent.target_participant_lease_token_ref,
            Some(&context),
        )
        .map_err(protocol_error)?;
    if let Some(reference) = intent.resume_handle_ref.as_ref() {
        authority
            .read_typed_object(root.root_revision, reference, None)
            .map_err(protocol_error)?;
    }
    if let Some(reference) = intent.transition_input_ref.as_ref() {
        authority
            .read_typed_object(root.root_revision, reference, Some(&context))
            .map_err(protocol_error)?;
    }
    let parent = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: Some(Box::new(intent.clone())),
    };
    authority
        .read_typed_object(
            root.root_revision,
            &intent.transport_payload_ref,
            Some(&parent),
        )
        .map_err(protocol_error)?;
    let expected = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&payload_value_from_intent(intent)).map_err(protocol_error)?,
    };
    if expected != intent.payload_commitment {
        return Err(error("transition payload commitment does not verify"));
    }
    Ok(())
}

fn request_matches_intent(
    request: &IssueHostSessionTransitionRequestV1,
    intent: &HostSessionTransitionIntentV1,
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

fn verify_joined_objects(
    authority: &HostSessionAuthority,
    root: &StateRootV1,
    request: &IssueHostSessionTransitionRequestV1,
    intent: &HostSessionTransitionIntentV1,
) -> Result<(), TransitionProtocolError> {
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    let lease = authority
        .read_typed_object(
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
            let actual = authority
                .read_typed_object(root.root_revision, reference, Some(&context))
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
        parent_intent: Some(Box::new(intent.clone())),
    };
    authority
        .read_typed_object(
            root.root_revision,
            &intent.transport_payload_ref,
            Some(&parent),
        )
        .map_err(protocol_error)?;
    if intent.mode != HostSessionTransitionModeV1::Start {
        if request.start_contract.is_some() {
            return Err(error("successor retry contains Start contract material"));
        }
        verify_intent_objects(authority, root, intent)?;
        return Ok(());
    }
    let material = request
        .start_contract
        .as_ref()
        .ok_or_else(|| error("stored Start retry requires contract material"))?;
    let descriptor_bytes = authority
        .read_typed_object(root.root_revision, &intent.descriptor_ref, None)
        .map_err(protocol_error)?;
    let descriptor: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(&descriptor_bytes).map_err(protocol_error)?;
    if descriptor.descriptor != material.descriptor {
        return Err(error(
            "descriptor projection does not match stored issuance",
        ));
    }
    let attach_bytes = authority
        .read_typed_object(root.root_revision, &intent.host_attach_contract_ref, None)
        .map_err(protocol_error)?;
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&attach_bytes).map_err(protocol_error)?;
    let policy_bytes = authority
        .read_typed_object(root.root_revision, &attach.contract.policy_ref, None)
        .map_err(protocol_error)?;
    let policy: PolicyObjectHashInputV1 =
        canonical_json::from_slice(&policy_bytes).map_err(protocol_error)?;
    if policy != material.policy
        || attach.contract.capabilities != material.capabilities
        || attach.contract.attach_launch_knobs != material.launch_knobs
    {
        return Err(error(
            "attach or policy projection does not match stored issuance",
        ));
    }
    Ok(())
}

fn validate_new_start_request(
    root: &StateRootV1,
    request: &IssueHostSessionTransitionRequestV1,
    ttl_seconds: i64,
) -> Result<(), TransitionProtocolError> {
    if request.mode != HostSessionTransitionModeV1::Start
        || request.authority_precondition != HostSessionAuthorityPreconditionV1::ExpectedAbsent
    {
        return Err(error(
            "only Start with ExpectedAbsent is accepted by this issuance path",
        ));
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
        || request.resume_handle_ref.is_some()
        || request.post_turn_disposition.is_some()
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
    }) {
        return Err(error(
            "Start target participant identity is already semantically occupied",
        ));
    }
    if root
        .transition_intent_map
        .values()
        .any(|intent| intent.run_id == request.run_id)
    {
        return Err(error("Start run identity is already semantically occupied"));
    }
    if root
        .transition_intent_map
        .values()
        .any(|intent| intent.shell_trace_session_id == request.shell_trace_session_id)
    {
        return Err(error(
            "Start shell trace identity is already semantically occupied",
        ));
    }
    let material = request
        .start_contract
        .as_ref()
        .ok_or_else(|| error("Start contract material is required"))?;
    if material.descriptor.schema_version != SCHEMA_VERSION
        || material.policy.schema_version != SCHEMA_VERSION
        || material.descriptor.execution_scope != material.launch_knobs.requested_execution_scope
        || (material.descriptor.execution_scope == super::schema::AgentExecutionScopeV1::Host
            && request.world_binding.is_some())
        || (material.descriptor.execution_scope == super::schema::AgentExecutionScopeV1::World
            && request.world_binding.is_none())
    {
        return Err(error(
            "Start descriptor, policy, launch, or world binding is invalid",
        ));
    }
    Ok(())
}

fn validate_new_parked_request<'root>(
    root: &'root StateRootV1,
    request: &IssueHostSessionTransitionRequestV1,
    ttl_seconds: i64,
) -> Result<&'root DurableSessionAuthorityV1, TransitionProtocolError> {
    if !matches!(
        request.mode,
        HostSessionTransitionModeV1::Attach | HostSessionTransitionModeV1::ResumeOneTurn
    ) || ttl_seconds <= 0
        || ttl_seconds > MAX_INTENT_TTL_SECONDS
        || request.start_contract.is_some()
    {
        return Err(error("parked transition mode, TTL, or material is invalid"));
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
    let authority = match root
        .session_namespace_map
        .get(&request.orchestration_session_id)
    {
        Some(SessionNamespaceRecordV1::Authority(authority)) => authority.as_ref(),
        _ => return Err(error("parked transition requires durable authority")),
    };
    if !matches!(
        authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
            | HostSessionPostureV1::DetachedReconciled
            | HostSessionPostureV1::AwaitingAttention
            | HostSessionPostureV1::StaleRecoverable
    ) {
        return Err(error("current authority posture is not successor-eligible"));
    }
    let source = authority
        .active_authoritative_participant_id
        .as_ref()
        .ok_or_else(|| error("current authority has no exact source participant"))?;
    if authority.authoritative_participant_lineage.last() != Some(source)
        || request.source_authoritative_participant_id.as_ref() != Some(source)
        || request.target_authoritative_participant_id == *source
        || authority
            .authoritative_participant_lineage
            .contains(&request.target_authoritative_participant_id)
    {
        return Err(error(
            "successor source, target, or current lineage is invalid",
        ));
    }
    let mut expected_lineage = authority.authoritative_participant_lineage.clone();
    expected_lineage.push(request.target_authoritative_participant_id.clone());
    if request.resulting_authoritative_lineage != expected_lineage {
        return Err(error("successor lineage is not the exact authority append"));
    }
    let expected_precondition = HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision: authority.authority_revision,
        authority_record_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_hash_input(authority))
                .map_err(protocol_error)?,
        },
        active_authoritative_participant_id: source.clone(),
        authoritative_lineage_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
                schema_version: SCHEMA_VERSION,
                orchestration_session_id: authority.orchestration_session_id.clone(),
                participant_ids: authority.authoritative_participant_lineage.clone(),
            })
            .map_err(protocol_error)?,
        },
        lifecycle_posture: authority.lifecycle_posture,
    };
    if request.authority_precondition != expected_precondition
        || request.shell_trace_session_id != authority.shell_trace_session_id
        || request.workspace_binding != authority.workspace_binding
        || request.world_binding != authority.world_binding
        || request.target_participant_lease_token.is_empty()
    {
        return Err(error(
            "successor authority revision, identity, binding, or lease mismatches",
        ));
    }
    let public_caller = matches!(
        request.caller.kind,
        HostSessionTransitionCallerKindV1::PublicCli | HostSessionTransitionCallerKindV1::Repl
    ) && request.caller.caller_participant_id.as_ref() == Some(source)
        && request.caller.auto_attach_obligation_id.is_none()
        && request.caller.auto_attach_claim_owner.is_none();
    let router_caller = request.mode == HostSessionTransitionModeV1::Attach
        && request.caller.kind == HostSessionTransitionCallerKindV1::RouterAutoAttach
        && request.caller.caller_participant_id.as_ref() == Some(source)
        && request.caller.auto_attach_obligation_id.is_some()
        && request.caller.auto_attach_claim_owner.is_some();
    if !public_caller && !router_caller {
        return Err(error("successor transition caller identity is invalid"));
    }
    match request.mode {
        HostSessionTransitionModeV1::Attach
            if request.transition_input.is_none() && request.post_turn_disposition.is_none() => {}
        HostSessionTransitionModeV1::ResumeOneTurn
            if request.resume_handle_ref.is_some()
                && request.transition_input.is_some()
                && request.post_turn_disposition
                    == Some(HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal) => {}
        _ => return Err(error("successor mode handoff fields are invalid")),
    }
    if root.transition_intent_map.contains_key(&request.intent_id)
        || root
            .issuer_request_index
            .contains_key(&request.issuer_request_id)
        || root.transition_intent_map.values().any(|intent| {
            intent.orchestration_session_id == request.orchestration_session_id
                && matches!(
                    intent.state,
                    HostSessionTransitionIntentStateV1::Issued
                        | HostSessionTransitionIntentStateV1::Claimed { .. }
                )
        })
        || root
            .transition_intent_map
            .values()
            .any(|intent| intent.run_id == request.run_id)
        || root.transition_intent_map.values().any(|intent| {
            intent.target_authoritative_participant_id
                == request.target_authoritative_participant_id
                || intent
                    .resulting_authoritative_lineage
                    .contains(&request.target_authoritative_participant_id)
        })
    {
        return Err(error(
            "successor intent, issuer, run, or participant identity is occupied",
        ));
    }
    Ok(authority)
}

fn payload_value_from_intent(
    intent: &HostSessionTransitionIntentV1,
) -> HostSessionTransitionPayloadHashInputV1 {
    HostSessionTransitionPayloadHashInputV1 {
        schema_version: SCHEMA_VERSION,
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
    root: &mut StateRootV1,
    reference: &super::schema::AuthorityObjectRefV1,
    kind: AuthorityObjectKindV1,
    byte_length: u64,
) -> Result<(), TransitionProtocolError> {
    if reference.object_kind != kind || reference.schema_version != SCHEMA_VERSION {
        return Err(error("typed object kind or schema is inconsistent"));
    }
    let entry = AuthorityObjectIndexEntryV1 {
        schema_version: SCHEMA_VERSION,
        ref_id: reference.ref_id.clone(),
        object_kind: kind,
        object_schema_version: SCHEMA_VERSION,
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
        Err(error("injected transition issuance crash point"))
    } else {
        Ok(())
    }
}

fn stop_application_at(
    actual: Option<ApplicationCrashPointV1>,
    expected: ApplicationCrashPointV1,
) -> Result<(), TransitionProtocolError> {
    if actual == Some(expected) {
        Err(error("injected transition application crash point"))
    } else {
        Ok(())
    }
}

fn stop_release_at(
    actual: Option<TransportReleaseCrashPointV1>,
    expected: TransportReleaseCrashPointV1,
) -> Result<(), TransitionProtocolError> {
    if actual == Some(expected) {
        Err(error("injected transition transport release crash point"))
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
