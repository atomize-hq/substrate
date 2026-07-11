use std::fmt;

use serde::Serialize;

use super::schema::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidationError(&'static str);

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for ValidationError {}

pub(crate) trait ValidatedCanonicalV1: sealed::Sealed + Serialize {
    fn validate(&self) -> Result<(), ValidationError>;
}

pub(crate) trait CanonicalHashInputV1: ValidatedCanonicalV1 {}

fn schema_v1(value: u32) -> Result<(), ValidationError> {
    if value == 1 {
        Ok(())
    } else {
        Err(ValidationError(
            "named hash input requires schema version 1",
        ))
    }
}

fn required(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        Err(ValidationError(
            "named hash input contains an empty identity",
        ))
    } else {
        Ok(())
    }
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn prefixed_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|body| lower_hex(body, 32))
}

fn validate_commitment(commitment: &AuthorityObjectCommitmentV1) -> Result<(), ValidationError> {
    match commitment {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex }
            if lower_hex(digest_hex, 64) =>
        {
            Ok(())
        }
        AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id,
            domain,
            digest_hex,
        } if prefixed_id(key_id, "ak_")
            && matches!(
                domain.as_str(),
                "substrate.a1.transition-input.v1"
                    | "substrate.a1.participant-lease-token.v1"
                    | "substrate.a1.raw-transport-payload.v1"
            )
            && lower_hex(digest_hex, 64) =>
        {
            Ok(())
        }
        _ => Err(ValidationError("invalid authority object commitment")),
    }
}

pub(crate) fn validate_object_commitment_rule(
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
    commitment: &AuthorityObjectCommitmentV1,
) -> Result<(), ValidationError> {
    schema_v1(schema_version)?;
    validate_commitment(commitment)?;
    let expected_domain = match object_kind {
        AuthorityObjectKindV1::TransitionTransportPayload => {
            Some("substrate.a1.raw-transport-payload.v1")
        }
        AuthorityObjectKindV1::TransitionInput => Some("substrate.a1.transition-input.v1"),
        AuthorityObjectKindV1::LeaseToken => Some("substrate.a1.participant-lease-token.v1"),
        AuthorityObjectKindV1::AgentDescriptor
        | AuthorityObjectKindV1::RetainedWorker
        | AuthorityObjectKindV1::ResumeHandle
        | AuthorityObjectKindV1::Policy
        | AuthorityObjectKindV1::HostAttachContract
        | AuthorityObjectKindV1::ApplicationResult
        | AuthorityObjectKindV1::InputAcceptance
        | AuthorityObjectKindV1::PostTurnCompletion
        | AuthorityObjectKindV1::TerminalHandoff => None,
    };
    match (expected_domain, commitment) {
        (None, AuthorityObjectCommitmentV1::CanonicalSha256 { .. }) => Ok(()),
        (Some(expected), AuthorityObjectCommitmentV1::StoreHmacSha256 { domain, .. })
            if domain == expected =>
        {
            Ok(())
        }
        _ => Err(ValidationError(
            "authority object kind and commitment rule do not match",
        )),
    }
}

fn validate_ref(
    value: &AuthorityObjectRefV1,
    expected_kind: AuthorityObjectKindV1,
) -> Result<(), ValidationError> {
    if !prefixed_id(&value.ref_id, "ao_") || value.schema_version != 1 {
        return Err(ValidationError(
            "invalid authority object reference identity",
        ));
    }
    if value.object_kind != expected_kind {
        return Err(ValidationError("authority object reference has wrong kind"));
    }
    validate_object_commitment_rule(value.object_kind, value.schema_version, &value.commitment)
}

fn validate_directory(value: &CanonicalDirectoryV1) -> Result<(), ValidationError> {
    required(&value.physical_path)?;
    match &value.physical_identity {
        DirectoryPhysicalIdentityV1::Linux { .. } => Ok(()),
        DirectoryPhysicalIdentityV1::MacOs { volume_uuid, .. } => required(volume_uuid),
    }
}

fn validate_workspace(value: &WorkspaceBindingV1) -> Result<(), ValidationError> {
    if !prefixed_id(&value.authority_store_id, "as_") {
        return Err(ValidationError("invalid authority store ID"));
    }
    validate_directory(&value.workspace_root)?;
    validate_directory(&value.authority_store_root)
}

fn validate_world(value: &WorldBindingV1) -> Result<(), ValidationError> {
    required(&value.world_id)
}

fn validate_canonical_commitment(
    value: &AuthorityObjectCommitmentV1,
) -> Result<(), ValidationError> {
    match value {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex }
            if lower_hex(digest_hex, 64) =>
        {
            Ok(())
        }
        _ => Err(ValidationError("expected a canonical SHA-256 commitment")),
    }
}

impl sealed::Sealed for AuthorityObjectRefV1 {}
impl ValidatedCanonicalV1 for AuthorityObjectRefV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        if !prefixed_id(&self.ref_id, "ao_") || self.schema_version != 1 {
            return Err(ValidationError(
                "invalid authority object reference identity",
            ));
        }
        validate_object_commitment_rule(self.object_kind, self.schema_version, &self.commitment)
    }
}

impl sealed::Sealed for AgentDescriptorHashInputV1 {}
impl ValidatedCanonicalV1 for AgentDescriptorHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        schema_v1(self.descriptor.schema_version)?;
        required(&self.descriptor.agent_id)?;
        required(&self.descriptor.backend_id)?;
        required(&self.descriptor.protocol)?;
        required(&self.descriptor.binary_path)
    }
}

impl sealed::Sealed for HostAttachContractHashInputV1 {}
impl ValidatedCanonicalV1 for HostAttachContractHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        schema_v1(self.contract.schema_version)?;
        required(&self.contract.backend_id)?;
        required(&self.contract.protocol)?;
        validate_ref(
            &self.contract.descriptor_ref,
            AuthorityObjectKindV1::AgentDescriptor,
        )?;
        validate_ref(&self.contract.policy_ref, AuthorityObjectKindV1::Policy)?;
        if let Some(value) = &self.contract.continuity_resume_handle_ref {
            validate_ref(value, AuthorityObjectKindV1::ResumeHandle)?;
        }
        Ok(())
    }
}

impl sealed::Sealed for ResumeHandleHashInputV1 {}
impl ValidatedCanonicalV1 for ResumeHandleHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.orchestration_session_id)?;
        required(&self.participant_id)?;
        required(&self.backend_id)?;
        required(&self.protocol)?;
        required(&self.internal_uaa_session_id)
    }
}

impl sealed::Sealed for PolicyObjectHashInputV1 {}
impl ValidatedCanonicalV1 for PolicyObjectHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.policy_revision)?;
        if lower_hex(&self.canonical_policy_snapshot_sha256, 64) {
            Ok(())
        } else {
            Err(ValidationError("invalid canonical policy snapshot digest"))
        }
    }
}

impl sealed::Sealed for RetainedWorkerObjectHashInputV1 {}
impl ValidatedCanonicalV1 for RetainedWorkerObjectHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.orchestration_session_id)?;
        required(&self.participant_id)?;
        validate_world(&self.world_binding)?;
        validate_ref(&self.descriptor_ref, AuthorityObjectKindV1::AgentDescriptor)?;
        validate_ref(&self.resume_handle_ref, AuthorityObjectKindV1::ResumeHandle)?;
        validate_ref(&self.policy_ref, AuthorityObjectKindV1::Policy)
    }
}

impl sealed::Sealed for AuthoritativeLineageHashInputV1 {}
impl ValidatedCanonicalV1 for AuthoritativeLineageHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.orchestration_session_id)?;
        if self.participant_ids.is_empty() || self.participant_ids.iter().any(|id| id.is_empty()) {
            Err(ValidationError(
                "authoritative lineage must contain identities",
            ))
        } else {
            Ok(())
        }
    }
}

impl sealed::Sealed for DurableSessionAuthorityHashInputV1 {}
impl ValidatedCanonicalV1 for DurableSessionAuthorityHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.orchestration_session_id)?;
        required(&self.shell_trace_session_id)?;
        if self.authority_revision == 0
            || self.authoritative_participant_lineage.is_empty()
            || self
                .authoritative_participant_lineage
                .iter()
                .any(|id| id.is_empty())
        {
            return Err(ValidationError("invalid authority revision or lineage"));
        }
        if self
            .active_authoritative_participant_id
            .as_ref()
            .is_some_and(|id| id.is_empty() || !self.authoritative_participant_lineage.contains(id))
        {
            return Err(ValidationError("active participant is not in the lineage"));
        }
        match &self.origin {
            DurableSessionAuthorityOriginV1::StartIntent {
                intent_id,
                issuer_request_id,
                payload_commitment,
            } => {
                required(intent_id)?;
                required(issuer_request_id)?;
                validate_canonical_commitment(payload_commitment)?;
            }
        }
        validate_workspace(&self.workspace_binding)?;
        if let Some(value) = &self.world_binding {
            validate_world(value)?;
        }
        if let Some(value) = &self.host_attach_contract_ref {
            validate_ref(value, AuthorityObjectKindV1::HostAttachContract)?;
        }
        for value in &self.retained_worker_refs {
            validate_ref(value, AuthorityObjectKindV1::RetainedWorker)?;
        }
        for value in &self.internal_resume_handle_refs {
            validate_ref(value, AuthorityObjectKindV1::ResumeHandle)?;
        }
        match (&self.current_policy_ref, &self.current_policy_revision) {
            (Some(value), Some(revision)) => {
                validate_ref(value, AuthorityObjectKindV1::Policy)?;
                required(revision)
            }
            (None, None) => Ok(()),
            _ => Err(ValidationError("policy ref and revision must agree")),
        }
    }
}

fn validate_caller(value: &HostSessionTransitionCallerV1) -> Result<(), ValidationError> {
    if value
        .caller_participant_id
        .as_ref()
        .is_some_and(|value| value.is_empty())
        || value
            .auto_attach_obligation_id
            .as_ref()
            .is_some_and(|value| value.is_empty())
        || value
            .auto_attach_claim_owner
            .as_ref()
            .is_some_and(|value| value.is_empty())
    {
        Err(ValidationError("caller contains an empty identity"))
    } else {
        Ok(())
    }
}

struct TransitionValidation<'a> {
    schema_version: u32,
    intent_id: &'a str,
    orchestration_session_id: &'a str,
    shell_trace_session_id: &'a str,
    caller: &'a HostSessionTransitionCallerV1,
    target_participant_id: &'a str,
    lease_ref: &'a AuthorityObjectRefV1,
    run_id: &'a str,
    lineage: &'a [String],
    workspace: &'a WorkspaceBindingV1,
    world: Option<&'a WorldBindingV1>,
    descriptor_ref: &'a AuthorityObjectRefV1,
    attach_ref: &'a AuthorityObjectRefV1,
    resume_ref: Option<&'a AuthorityObjectRefV1>,
    input_ref: Option<&'a AuthorityObjectRefV1>,
}

fn validate_transition_common(value: TransitionValidation<'_>) -> Result<(), ValidationError> {
    schema_v1(value.schema_version)?;
    required(value.intent_id)?;
    required(value.orchestration_session_id)?;
    required(value.shell_trace_session_id)?;
    validate_caller(value.caller)?;
    required(value.target_participant_id)?;
    required(value.run_id)?;
    if value.lineage.is_empty() || value.lineage.iter().any(|id| id.is_empty()) {
        return Err(ValidationError(
            "transition lineage must contain identities",
        ));
    }
    validate_ref(value.lease_ref, AuthorityObjectKindV1::LeaseToken)?;
    validate_workspace(value.workspace)?;
    if let Some(world) = value.world {
        validate_world(world)?;
    }
    validate_ref(value.descriptor_ref, AuthorityObjectKindV1::AgentDescriptor)?;
    validate_ref(value.attach_ref, AuthorityObjectKindV1::HostAttachContract)?;
    if let Some(resume_ref) = value.resume_ref {
        validate_ref(resume_ref, AuthorityObjectKindV1::ResumeHandle)?;
    }
    if let Some(input_ref) = value.input_ref {
        validate_ref(input_ref, AuthorityObjectKindV1::TransitionInput)?;
    }
    Ok(())
}

impl sealed::Sealed for HostSessionTransitionPayloadHashInputV1 {}
impl ValidatedCanonicalV1 for HostSessionTransitionPayloadHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        required(&self.issuer_request_id)?;
        validate_transition_common(TransitionValidation {
            schema_version: self.schema_version,
            intent_id: &self.intent_id,
            orchestration_session_id: &self.orchestration_session_id,
            shell_trace_session_id: &self.shell_trace_session_id,
            caller: &self.caller,
            target_participant_id: &self.target_authoritative_participant_id,
            lease_ref: &self.target_participant_lease_token_ref,
            run_id: &self.run_id,
            lineage: &self.resulting_authoritative_lineage,
            workspace: &self.workspace_binding,
            world: self.world_binding.as_ref(),
            descriptor_ref: &self.descriptor_ref,
            attach_ref: &self.host_attach_contract_ref,
            resume_ref: self.resume_handle_ref.as_ref(),
            input_ref: self.transition_input_ref.as_ref(),
        })?;
        validate_ref(
            &self.transport_payload_ref,
            AuthorityObjectKindV1::TransitionTransportPayload,
        )
    }
}

impl sealed::Sealed for TransitionTransportPayloadObjectV1 {}
impl ValidatedCanonicalV1 for TransitionTransportPayloadObjectV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        validate_transition_common(TransitionValidation {
            schema_version: self.schema_version,
            intent_id: &self.intent_id,
            orchestration_session_id: &self.orchestration_session_id,
            shell_trace_session_id: &self.shell_trace_session_id,
            caller: &self.caller,
            target_participant_id: &self.target_authoritative_participant_id,
            lease_ref: &self.target_participant_lease_token_ref,
            run_id: &self.run_id,
            lineage: &self.resulting_authoritative_lineage,
            workspace: &self.workspace_binding,
            world: self.world_binding.as_ref(),
            descriptor_ref: &self.descriptor_ref,
            attach_ref: &self.host_attach_contract_ref,
            resume_ref: self.resume_handle_ref.as_ref(),
            input_ref: self.transition_input_ref.as_ref(),
        })
    }
}

impl sealed::Sealed for ApplicationResultHashInputV1 {}
impl ValidatedCanonicalV1 for ApplicationResultHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.intent_id)?;
        required(&self.run_id)?;
        match &self.phase {
            ApplicationResultPhaseV1::InitialTransition {
                authority_revision_after,
                active_authoritative_participant_id,
                authority_record_commitment,
                post_turn_pending_run_id,
                ..
            } => {
                if *authority_revision_after == 0 {
                    return Err(ValidationError("invalid authority revision"));
                }
                required(active_authoritative_participant_id)?;
                validate_canonical_commitment(authority_record_commitment)?;
                if let Some(value) = post_turn_pending_run_id {
                    required(value)?;
                }
            }
            ApplicationResultPhaseV1::PostTurn {
                completion_ref,
                authority_revision_before,
                authority_revision_after,
                active_authoritative_participant_id,
                authority_record_commitment,
                ..
            } => {
                if *authority_revision_after <= *authority_revision_before {
                    return Err(ValidationError("post-turn revision did not advance"));
                }
                validate_ref(completion_ref, AuthorityObjectKindV1::PostTurnCompletion)?;
                required(active_authoritative_participant_id)?;
                validate_canonical_commitment(authority_record_commitment)?;
            }
        }
        Ok(())
    }
}

impl sealed::Sealed for InputAcceptanceHashInputV1 {}
impl ValidatedCanonicalV1 for InputAcceptanceHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.intent_id)?;
        required(&self.run_id)?;
        required(&self.accepting_participant_id)?;
        validate_ref(&self.input_ref, AuthorityObjectKindV1::TransitionInput)
    }
}

impl sealed::Sealed for PostTurnCompletionHashInputV1 {}
impl ValidatedCanonicalV1 for PostTurnCompletionHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.intent_id)?;
        required(&self.run_id)?;
        if self.authority_revision_observed == 0 {
            Err(ValidationError("invalid observed authority revision"))
        } else {
            Ok(())
        }
    }
}

impl sealed::Sealed for TerminalHandoffHashInputV1 {}
impl ValidatedCanonicalV1 for TerminalHandoffHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.intent_id)?;
        required(&self.run_id)?;
        validate_canonical_commitment(&self.payload_commitment)?;
        if let Some(value) = &self.application_result_ref {
            validate_ref(value, AuthorityObjectKindV1::ApplicationResult)?;
        }
        if let Some(value) = &self.input_acceptance_ref {
            validate_ref(value, AuthorityObjectKindV1::InputAcceptance)?;
        }
        if let Some(value) = &self.post_turn_completion_ref {
            validate_ref(value, AuthorityObjectKindV1::PostTurnCompletion)?;
        }
        if let Some(value) = &self.post_turn_application_result_ref {
            validate_ref(value, AuthorityObjectKindV1::ApplicationResult)?;
        }
        Ok(())
    }
}

impl CanonicalHashInputV1 for AuthorityObjectRefV1 {}
impl CanonicalHashInputV1 for AgentDescriptorHashInputV1 {}
impl CanonicalHashInputV1 for HostAttachContractHashInputV1 {}
impl CanonicalHashInputV1 for ResumeHandleHashInputV1 {}
impl CanonicalHashInputV1 for PolicyObjectHashInputV1 {}
impl CanonicalHashInputV1 for RetainedWorkerObjectHashInputV1 {}
impl CanonicalHashInputV1 for AuthoritativeLineageHashInputV1 {}
impl CanonicalHashInputV1 for DurableSessionAuthorityHashInputV1 {}
impl CanonicalHashInputV1 for HostSessionTransitionPayloadHashInputV1 {}
impl CanonicalHashInputV1 for ApplicationResultHashInputV1 {}
impl CanonicalHashInputV1 for InputAcceptanceHashInputV1 {}
impl CanonicalHashInputV1 for PostTurnCompletionHashInputV1 {}
impl CanonicalHashInputV1 for TerminalHandoffHashInputV1 {}

pub(crate) mod sealed {
    pub trait Sealed {}
}
