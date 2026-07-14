//! Durable A1.2a greenfield Start transition protocol.

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
    HostAttachLaunchKnobsV1, HostSessionAuthorityPreconditionV1, HostSessionPostureV1,
    HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1, HostSessionTransitionModeV1,
    HostSessionTransitionPayloadHashInputV1, PolicyObjectHashInputV1, TerminalHandoffHashInputV1,
    TerminalHandoffStateV1, TimestampV1, TransitionTransportPayloadObjectV1, WorkspaceBindingV1,
    WorldBindingV1,
};
use super::store::{
    self, GeneratedObjectV1, ObjectVerificationContextV1, VersionedObjectVerificationParentIntentV1,
};
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
    HostSessionPostTurnApplicationV1, HostSessionStartupOwnershipApplicationV1,
    HostSessionTransitionApplicationJournalV2, HostSessionTransitionInputHandoffV1,
    HostSessionTransitionIntentStateV2, HostSessionTransitionIntentV2,
    HostSessionTransitionTransportPayloadStateV1, InitialTransitionApplicationJournalV1,
    IssuerRequestIndexEntryV1, RetainedWorkerAuthorityRegistrationRequestStateV1,
    SessionIdReservationV1, SessionIdTombstoneV1, SessionNamespaceRecordV1, StartTombstoneStateV1,
    StateRootV2,
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
pub(crate) enum TransitionIssueOutcomeV1 {
    Issued(HostSessionTransitionIntentV2),
    Joined(HostSessionTransitionIntentV2),
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

    pub(crate) fn read_a12a_root(&self) -> Result<StateRootV2, TransitionProtocolError> {
        store::read_opened_root_v2(self.trusted_root()).map_err(protocol_error)
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
    let descriptor_bytes = store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.descriptor_ref,
        None,
    )
    .map_err(protocol_error)?;
    let descriptor: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(&descriptor_bytes).map_err(protocol_error)?;
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
    store::read_typed_object_v2_opened(
        authority.trusted_root(),
        root.root_revision,
        &intent.target_participant_lease_token_ref,
        Some(&context),
    )
    .map_err(protocol_error)?;
    if let Some(reference) = intent.transition_input_ref.as_ref() {
        store::read_typed_object_v2_opened(
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
    let transport_bytes = store::read_typed_object_v2_opened(
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
    let application_bytes = store::read_typed_object_v2_opened(
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
