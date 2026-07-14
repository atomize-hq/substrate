//! Durable A1.2a greenfield Start transition protocol.

use std::fmt;

use chrono::{DateTime, Duration, SecondsFormat, Utc};

use super::canonical_json;
use super::facade::HostSessionAuthority;
use super::hash::{canonical_object_bytes, canonical_sha256, CanonicalObjectHashInputV1};
use super::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AuthorityObjectCommitmentV1,
    AuthorityObjectKindV1, HostAttachCapabilitiesV1, HostAttachContractHashInputV1,
    HostAttachContractV1, HostAttachLaunchKnobsV1, HostSessionAuthorityPreconditionV1,
    HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1, HostSessionTransitionModeV1,
    HostSessionTransitionPayloadHashInputV1, PolicyObjectHashInputV1, TimestampV1,
    TransitionTransportPayloadObjectV1, WorkspaceBindingV1, WorldBindingV1,
};
use super::store::{
    self, GeneratedObjectV1, ObjectVerificationContextV1, VersionedObjectVerificationParentIntentV1,
};
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1,
    HostSessionTransitionInputHandoffV1, HostSessionTransitionIntentStateV2,
    HostSessionTransitionIntentV2, HostSessionTransitionTransportPayloadStateV1,
    IssuerRequestIndexEntryV1, SessionIdReservationV1, SessionNamespaceRecordV1, StateRootV2,
};
use super::trusted_fs::TrustedWorkspaceRoot;

pub(crate) const DEFAULT_INTENT_TTL_SECONDS: i64 = 300;
pub(crate) const MAX_INTENT_TTL_SECONDS: i64 = 900;

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
        || (material.descriptor.execution_scope == super::schema::AgentExecutionScopeV1::Host
            && request.world_binding.is_some())
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

fn error(message: impl Into<String>) -> TransitionProtocolError {
    TransitionProtocolError(message.into())
}

fn protocol_error(error: impl fmt::Display) -> TransitionProtocolError {
    TransitionProtocolError(error.to_string())
}
