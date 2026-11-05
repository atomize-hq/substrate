use std::fmt;

use serde::Serialize;
use substrate_common::HostTransitionWorkCorrelationV1;

use super::super::obligation_ledger::{
    ObligationMaterializationCutV1, SupervisorJournalEventRefV1,
};
use super::super::state_store::AcceptedWorldWorkIdentityV1;
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

pub(super) fn validate_fork_successor_attach_semantics(
    source: &HostAttachContractHashInputV1,
    target: &HostAttachContractHashInputV1,
) -> Result<(), ValidationError> {
    let mut expected_target = source.clone();
    expected_target.contract.continuity_resume_handle_ref = None;
    if !source.contract.capabilities.session_fork {
        return Err(ValidationError(
            "fork successor source attach contract disables session_fork",
        ));
    }
    if target != &expected_target {
        return Err(ValidationError(
            "fork successor target attach contract is not the continuity-cleared source contract",
        ));
    }
    Ok(())
}

fn schema_v1(value: u32) -> Result<(), ValidationError> {
    if value == 1 {
        Ok(())
    } else {
        Err(ValidationError(
            "named hash input requires schema version 1",
        ))
    }
}

fn schema_v2(value: u32) -> Result<(), ValidationError> {
    if value == 2 {
        Ok(())
    } else {
        Err(ValidationError(
            "named hash input requires schema version 2",
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

fn object_schema_version(
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
) -> Result<(), ValidationError> {
    match object_kind {
        AuthorityObjectKindV1::TerminalHandoff if matches!(schema_version, 1 | 2) => Ok(()),
        AuthorityObjectKindV1::ResumeHandle if matches!(schema_version, 1..=3) => Ok(()),
        AuthorityObjectKindV1::PostTurnCompletion if matches!(schema_version, 1..=4) => Ok(()),
        AuthorityObjectKindV1::PostTurnProtocolEvent if matches!(schema_version, 1 | 2) => Ok(()),
        _ => schema_v1(schema_version),
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
    object_schema_version(object_kind, schema_version)?;
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
        | AuthorityObjectKindV1::StartupOwnershipResult
        | AuthorityObjectKindV1::ObligationSnapshot
        | AuthorityObjectKindV1::PostTurnProtocolEvent
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
    if !prefixed_id(&value.ref_id, "ao_") {
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

fn validate_work_identity(value: &AcceptedWorldWorkIdentityV1) -> Result<(), ValidationError> {
    match value {
        AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } => required(task_run_id),
        AcceptedWorldWorkIdentityV1::RetainedTurn {
            active_run_id,
            message_id,
            target_participant_id,
        } => {
            required(active_run_id)?;
            required(message_id)?;
            required(target_participant_id)
        }
    }
}

fn validate_host_transition_correlation(
    value: &HostTransitionWorkCorrelationV1,
) -> Result<(), ValidationError> {
    value
        .validate()
        .map_err(|_| ValidationError("invalid host transition correlation"))
}

fn validate_supervisor_journal_event(
    value: &SupervisorJournalEventRefV1,
) -> Result<(), ValidationError> {
    schema_v1(value.schema_version)?;
    required(&value.journal_entry_id)?;
    required(&value.acceptance_record_id)?;
    if value.acceptance_record_revision == 0 {
        return Err(ValidationError("invalid acceptance record revision"));
    }
    validate_work_identity(&value.accepted_work_identity)?;
    required(&value.stream_id)?;
    required(&value.event_id)?;
    validate_canonical_commitment(&value.transport_event_commitment)
}

fn validate_obligation_cut(value: &ObligationMaterializationCutV1) -> Result<(), ValidationError> {
    required(&value.acceptance_record_id)?;
    if value.acceptance_record_revision == 0 || value.session_ledger_revision == 0 {
        return Err(ValidationError(
            "invalid obligation materialization cut revision",
        ));
    }
    required(&value.stream_id)?;
    required(&value.terminal_event_id)
}

impl sealed::Sealed for AuthorityObjectRefV1 {}
impl ValidatedCanonicalV1 for AuthorityObjectRefV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        if !prefixed_id(&self.ref_id, "ao_") {
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

impl sealed::Sealed for ForkSuccessorAllocationRequestHashInputV1 {}
impl ValidatedCanonicalV1 for ForkSuccessorAllocationRequestHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        for value in [
            self.allocation_id.as_str(),
            self.request_id.as_str(),
            self.source_orchestration_session_id.as_str(),
            self.source_shell_trace_session_id.as_str(),
            self.target_orchestration_session_id.as_str(),
            self.target_shell_trace_session_id.as_str(),
            self.target_authoritative_participant_id.as_str(),
        ] {
            required(value)?;
        }
        if !prefixed_id(&self.authority_store_id, "as_") || self.expected_source_root_revision == 0
        {
            return Err(ValidationError(
                "fork successor allocation store or root revision is invalid",
            ));
        }
        validate_directory(&self.bootstrap_home)?;
        validate_workspace(&self.workspace_binding)?;
        if self.workspace_binding.authority_store_id != self.authority_store_id
            || self.workspace_binding.authority_store_root != self.bootstrap_home
        {
            return Err(ValidationError(
                "fork successor allocation workspace binding is inconsistent",
            ));
        }
        if let Some(world) = &self.world_binding {
            validate_world(world)?;
        }
        let HostSessionAuthorityPreconditionV1::ExpectedRevision {
            authority_revision,
            authority_record_commitment,
            active_authoritative_participant_id,
            authoritative_lineage_commitment,
            ..
        } = &self.source_authority_precondition
        else {
            return Err(ValidationError(
                "fork successor allocation requires an exact source revision",
            ));
        };
        if *authority_revision == 0 {
            return Err(ValidationError(
                "fork successor allocation source revision is invalid",
            ));
        }
        validate_canonical_commitment(authority_record_commitment)?;
        validate_canonical_commitment(authoritative_lineage_commitment)?;
        required(active_authoritative_participant_id)?;
        if self.source_authoritative_participant_lineage.is_empty()
            || self
                .source_authoritative_participant_lineage
                .iter()
                .any(|participant| participant.is_empty())
            || self
                .source_authoritative_participant_lineage
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.source_authoritative_participant_lineage.len()
            || !self
                .source_authoritative_participant_lineage
                .contains(active_authoritative_participant_id)
            || self
                .source_authoritative_participant_lineage
                .contains(&self.target_authoritative_participant_id)
        {
            return Err(ValidationError(
                "fork successor allocation source lineage is invalid",
            ));
        }
        let mut expected_lineage = self.source_authoritative_participant_lineage.clone();
        expected_lineage.push(self.target_authoritative_participant_id.clone());
        if self.resulting_authoritative_lineage != expected_lineage
            || self.source_orchestration_session_id == self.target_orchestration_session_id
            || self.source_shell_trace_session_id == self.target_shell_trace_session_id
        {
            return Err(ValidationError(
                "fork successor allocation target identity or lineage is invalid",
            ));
        }
        Ok(())
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

impl sealed::Sealed for HostSessionStopPayloadHashInputV1 {}
impl ValidatedCanonicalV1 for HostSessionStopPayloadHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        for value in [
            self.intent_id.as_str(),
            self.request_id.as_str(),
            self.authority_store_id.as_str(),
            self.orchestration_session_id.as_str(),
            self.shell_trace_session_id.as_str(),
            self.authoritative_participant_id.as_str(),
        ] {
            required(value)?;
        }
        validate_directory(&self.bootstrap_home)?;
        validate_caller(&self.caller)?;
        validate_canonical_commitment(&self.authority_record_commitment)?;
        validate_canonical_commitment(&self.authoritative_lineage_commitment)?;
        if self.authority_revision == 0
            || self.authoritative_lineage.is_empty()
            || self
                .authoritative_lineage
                .iter()
                .any(|value| value.is_empty())
            || self.authoritative_lineage.last() != Some(&self.authoritative_participant_id)
            || self
                .authoritative_lineage
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.authoritative_lineage.len()
            || matches!(
                self.lifecycle_posture,
                HostSessionPostureV1::Terminal | HostSessionPostureV1::Invalid
            )
        {
            return Err(ValidationError("invalid HSA Stop payload identity"));
        }
        Ok(())
    }
}

impl sealed::Sealed for HostSessionStopResultHashInputV1 {}
impl ValidatedCanonicalV1 for HostSessionStopResultHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        for value in [
            self.result_id.as_str(),
            self.intent_id.as_str(),
            self.request_id.as_str(),
            self.authority_store_id.as_str(),
            self.orchestration_session_id.as_str(),
            self.authoritative_participant_id.as_str(),
        ] {
            required(value)?;
        }
        if let Some(value) = self.delivery_acceptance_id.as_deref() {
            required(value)?;
        }
        validate_canonical_commitment(&self.payload_commitment)?;
        validate_canonical_commitment(&self.authority_record_commitment_before)?;
        validate_canonical_commitment(&self.authority_record_commitment_after)?;
        if self.authority_revision_before == 0
            || self.authority_revision_after
                != self
                    .authority_revision_before
                    .checked_add(1)
                    .ok_or(ValidationError("HSA Stop authority revision overflows"))?
            || self.resulting_posture != HostSessionPostureV1::Terminal
        {
            return Err(ValidationError("invalid HSA Stop result identity"));
        }
        Ok(())
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

impl sealed::Sealed for StartupOwnershipResultHashInputV1 {}
impl ValidatedCanonicalV1 for StartupOwnershipResultHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        let evidence = &self.evidence;
        schema_v1(evidence.schema_version)?;
        required(&evidence.evidence_id)?;
        required(&evidence.authority_store_id)?;
        required(&evidence.orchestration_session_id)?;
        required(&evidence.intent_id)?;
        required(&evidence.claim_id)?;
        required(&evidence.claimant_attempt_id)?;
        required(&evidence.run_id)?;
        validate_ref(
            &evidence.application_result_ref,
            AuthorityObjectKindV1::ApplicationResult,
        )?;
        if evidence.expected_authority_revision == 0 {
            return Err(ValidationError("invalid expected authority revision"));
        }
        required(&evidence.active_authoritative_participant_id)?;
        match &evidence.protocol_actor {
            HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id,
            } => {
                required(participant_id)?;
            }
            HostStartupOwnershipProtocolActorV1::LaunchApplicationClaimant {
                claim_id,
                claimant_attempt_id,
            } => {
                required(claim_id)?;
                required(claimant_attempt_id)?;
            }
        }
        match &evidence.protocol_event {
            HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
                ownership_acknowledgement_id,
            } => required(ownership_acknowledgement_id)?,
            HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected { rejection_id } => {
                required(rejection_id)?
            }
            HostStartupOwnershipProtocolEventV1::StartupFailedBeforeOwnership { failure_id } => {
                required(failure_id)?
            }
        }
        match &self.outcome {
            StartupOwnershipOutcomeV1::Accepted => {}
            StartupOwnershipOutcomeV1::TerminalReconciled {
                authority_revision_after,
                authority_record_commitment,
                ..
            } => {
                if *authority_revision_after == 0 {
                    return Err(ValidationError("invalid terminal authority revision"));
                }
                validate_canonical_commitment(authority_record_commitment)?;
            }
        }
        Ok(())
    }
}

impl sealed::Sealed for ObligationSnapshotRecordHashInputV1 {}
impl ValidatedCanonicalV1 for ObligationSnapshotRecordHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.authority_store_id)?;
        required(&self.orchestration_session_id)?;
        required(&self.authoritative_participant_id)?;
        validate_supervisor_journal_event(&self.source_journal_event)?;
        required(&self.obligation_id)?;
        if self.obligation_revision == 0 {
            return Err(ValidationError("invalid obligation revision"));
        }
        Ok(())
    }
}

impl sealed::Sealed for ObligationSnapshotHashInputV1 {}
impl ValidatedCanonicalV1 for ObligationSnapshotHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.authority_store_id)?;
        required(&self.orchestration_session_id)?;
        required(&self.authoritative_participant_id)?;
        required(&self.acceptance_record_id)?;
        if self.acceptance_record_revision == 0 || self.authority_revision_observed == 0 {
            return Err(ValidationError("invalid obligation snapshot revision"));
        }
        required(&self.stream_id)?;
        validate_work_identity(&self.accepted_work_identity)?;
        validate_host_transition_correlation(&self.host_transition_correlation)?;
        required(&self.transition_intent_id)?;
        required(&self.transition_run_id)?;
        validate_obligation_cut(&self.materialization_cut)?;
        let mut last_materialized = None;
        for event in &self.materialized_journal_events {
            validate_supervisor_journal_event(event)?;
            let key = (
                event.event_sequence,
                event.frame_sequence,
                event.journal_entry_id.as_str(),
                event.event_id.as_str(),
            );
            if last_materialized.is_some_and(|last| last >= key) {
                return Err(ValidationError(
                    "materialized journal events must be strictly ordered",
                ));
            }
            last_materialized = Some(key);
        }
        let mut last_unresolved = None;
        for obligation in &self.unresolved_attention_obligations {
            required(&obligation.obligation_id)?;
            if obligation.obligation_revision == 0 {
                return Err(ValidationError("invalid unresolved obligation revision"));
            }
            validate_canonical_commitment(&obligation.canonical_record_commitment)?;
            let digest_hex = match &obligation.canonical_record_commitment {
                AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => digest_hex.as_str(),
                AuthorityObjectCommitmentV1::StoreHmacSha256 { .. } => {
                    return Err(ValidationError("expected a canonical SHA-256 commitment"))
                }
            };
            let key = (
                obligation.obligation_id.as_str(),
                obligation.obligation_revision,
                digest_hex,
            );
            if last_unresolved.is_some_and(|last| last >= key) {
                return Err(ValidationError(
                    "unresolved attention obligations must be strictly ordered",
                ));
            }
            last_unresolved = Some(key);
        }
        Ok(())
    }
}

impl sealed::Sealed for PostTurnProtocolEventHashInputV1 {}
impl ValidatedCanonicalV1 for PostTurnProtocolEventHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.authority_store_id)?;
        required(&self.orchestration_session_id)?;
        required(&self.intent_id)?;
        required(&self.claim_id)?;
        required(&self.claimant_attempt_id)?;
        required(&self.run_id)?;
        if self.authority_revision_observed == 0 || self.acceptance_record_revision == 0 {
            return Err(ValidationError("invalid post-turn protocol revision"));
        }
        required(&self.active_authoritative_participant_id)?;
        required(&self.acceptance_record_id)?;
        required(&self.stream_id)?;
        validate_work_identity(&self.accepted_work_identity)?;
        validate_host_transition_correlation(&self.host_transition_correlation)?;
        match &self.protocol_actor {
            HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant { participant_id } => {
                required(participant_id)?
            }
            HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
                claim_id,
                claimant_attempt_id,
            } => {
                required(claim_id)?;
                required(claimant_attempt_id)?;
            }
        }
        required(&self.event_id)
    }
}

impl sealed::Sealed for PostTurnCompletionHashInputV1 {}
impl ValidatedCanonicalV1 for PostTurnCompletionHashInputV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.intent_id)?;
        required(&self.run_id)?;
        if self.authority_revision_observed == 0 || self.acceptance_record_revision == 0 {
            return Err(ValidationError("invalid observed authority revision"));
        }
        required(&self.acceptance_record_id)?;
        required(&self.stream_id)?;
        validate_work_identity(&self.accepted_work_identity)?;
        validate_host_transition_correlation(&self.host_transition_correlation)?;
        required(&self.terminal_event_id)?;
        if self.protocol_event_ref.schema_version != 1 {
            return Err(ValidationError(
                "V1 post-turn completion requires a V1 protocol event",
            ));
        }
        validate_ref(
            &self.protocol_event_ref,
            AuthorityObjectKindV1::PostTurnProtocolEvent,
        )
    }
}

impl sealed::Sealed for DirectHostNativeTerminalEvidenceV1 {}
impl ValidatedCanonicalV1 for DirectHostNativeTerminalEvidenceV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        for value in [
            &self.backend_id,
            &self.protocol,
            &self.native_session_id,
            &self.thread_id,
            &self.turn_id,
        ] {
            required(value)?;
        }
        if self.native_session_id != self.thread_id
            || self.terminal_ordinal != 1
            || !lower_hex(&self.raw_event_sha256, 64)
        {
            return Err(ValidationError(
                "invalid direct-host native terminal identity",
            ));
        }
        let valid = match &self.sequence {
            DirectHostNativeSequenceV1::Codex {
                resumed_thread_id,
                started_turn_id,
            } => {
                resumed_thread_id == &self.thread_id
                    && started_turn_id == &self.turn_id
                    && matches!(
                        self.terminal_event_kind.as_str(),
                        "turn.completed" | "turn.failed"
                    )
            }
            DirectHostNativeSequenceV1::ClaudeCode {
                initialized_session_id,
            } => {
                initialized_session_id == &self.thread_id
                    && self.turn_id == self.thread_id
                    && matches!(
                        self.terminal_event_kind.as_str(),
                        "result.success" | "result.error"
                    )
            }
        };
        if !valid {
            return Err(ValidationError(
                "direct-host native terminal sequence does not join",
            ));
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for DirectHostNativeTerminalEvidenceV1 {}

impl sealed::Sealed for DirectHostResumeBindingV1 {}
impl ValidatedCanonicalV1 for DirectHostResumeBindingV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        for value in [
            &self.authority_store_id,
            &self.orchestration_session_id,
            &self.shell_trace_session_id,
            &self.authoritative_participant_id,
            &self.intent_id,
            &self.issuer_request_id,
            &self.claim_id,
            &self.claimant_attempt_id,
            &self.run_id,
        ] {
            required(value)?;
        }
        validate_directory(&self.bootstrap_home)?;
        validate_workspace(&self.workspace_binding)?;
        if self.workspace_binding.authority_store_id != self.authority_store_id
            || self.workspace_binding.authority_store_root != self.bootstrap_home
            || self.authority_revision_observed == 0
            || self.intent_revision_observed == 0
            || self.authoritative_lineage.last() != Some(&self.authoritative_participant_id)
            || self.authoritative_lineage.iter().any(String::is_empty)
            || self
                .authoritative_lineage
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.authoritative_lineage.len()
            || self.policy_ref.is_some() != self.policy_revision.is_some()
        {
            return Err(ValidationError("invalid direct-host persisted binding"));
        }
        validate_canonical_commitment(&self.authority_history_commitment)?;
        validate_canonical_commitment(&self.authority_record_commitment)?;
        validate_canonical_commitment(&self.payload_commitment)?;
        if let Some(reference) = &self.policy_ref {
            validate_ref(reference, AuthorityObjectKindV1::Policy)?;
        }
        if let Some(revision) = &self.policy_revision {
            required(revision)?;
        }
        validate_ref(&self.descriptor_ref, AuthorityObjectKindV1::AgentDescriptor)?;
        validate_ref(
            &self.host_attach_contract_ref,
            AuthorityObjectKindV1::HostAttachContract,
        )?;
        validate_ref(
            &self.initial_application_result_ref,
            AuthorityObjectKindV1::ApplicationResult,
        )?;
        validate_ref(
            &self.transition_input_ref,
            AuthorityObjectKindV1::TransitionInput,
        )?;
        if let Some(reference) = &self.input_acceptance_ref {
            validate_ref(reference, AuthorityObjectKindV1::InputAcceptance)?;
        }
        validate_ref(
            &self.consumed_resume_handle_ref,
            AuthorityObjectKindV1::ResumeHandle,
        )
    }
}
impl CanonicalHashInputV1 for DirectHostResumeBindingV1 {}

impl sealed::Sealed for DirectHostCanonicalAttentionCutV1 {}
impl ValidatedCanonicalV1 for DirectHostCanonicalAttentionCutV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        required(&self.authority_store_id)?;
        required(&self.orchestration_session_id)?;
        if self.source_protocol != "substrate.direct-host-inbox.resume-only.v1"
            || self.unresolved_attention.len() > 4096
        {
            return Err(ValidationError(
                "unsupported direct-host attention source or size",
            ));
        }
        for commitment in [
            &self.genesis_commitment,
            &self.source_head_commitment,
            &self.ledger_head_commitment,
        ] {
            validate_canonical_commitment(commitment)?;
        }
        let mut previous: Option<&str> = None;
        for entry in &self.unresolved_attention {
            required(&entry.obligation_id)?;
            if entry.obligation_revision == 0
                || previous.is_some_and(|id| id >= entry.obligation_id.as_str())
            {
                return Err(ValidationError(
                    "direct-host attention revisions are not strictly ordered",
                ));
            }
            validate_canonical_commitment(&entry.canonical_record_commitment)?;
            validate_canonical_commitment(&entry.original_receipt_commitment)?;
            validate_canonical_commitment(&entry.original_publication_commitment)?;
            previous = Some(&entry.obligation_id);
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for DirectHostCanonicalAttentionCutV1 {}

impl sealed::Sealed for PostTurnProtocolEventHashInputV2 {}
impl ValidatedCanonicalV1 for PostTurnProtocolEventHashInputV2 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v2(self.schema_version)?;
        self.binding.validate()?;
        required(&self.event_id)?;
        let accepted = self.binding.input_acceptance_ref.is_some();
        let runtime_rejected = matches!(
            self.kind,
            HostPostTurnProtocolEventKindV1::TerminalFailure {
                reason: HostPostTurnTerminalReasonV1::ResumeRuntimeCreationRejected,
            }
        );
        match (&self.protocol_actor, runtime_rejected) {
            (
                HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
                    claim_id,
                    claimant_attempt_id,
                },
                true,
            ) if claim_id == &self.binding.claim_id
                && claimant_attempt_id == &self.binding.claimant_attempt_id => {}
            (
                HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant { participant_id },
                false,
            ) if participant_id == &self.binding.authoritative_participant_id => {}
            _ => return Err(ValidationError("direct-host post-turn actor does not join")),
        }
        let expects_accepted = !matches!(
            self.kind,
            HostPostTurnProtocolEventKindV1::TerminalFailure {
                reason: HostPostTurnTerminalReasonV1::ResumeRuntimeCreationRejected
                    | HostPostTurnTerminalReasonV1::TargetFailedBeforeInputAcceptance,
            }
        );
        if accepted != expects_accepted {
            return Err(ValidationError(
                "direct-host post-turn input boundary does not join",
            ));
        }
        if runtime_rejected {
            if self.native_terminal.is_some() {
                return Err(ValidationError(
                    "runtime rejection cannot claim a native terminal",
                ));
            }
        } else {
            let native = self
                .native_terminal
                .as_ref()
                .ok_or(ValidationError("direct-host native terminal is absent"))?;
            native.validate()?;
            let native_failed = matches!(
                native.terminal_event_kind.as_str(),
                "turn.failed" | "result.error"
            );
            if native_failed
                != matches!(
                    self.kind,
                    HostPostTurnProtocolEventKindV1::TerminalFailure { .. }
                )
            {
                return Err(ValidationError("direct-host native outcome does not join"));
            }
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for PostTurnProtocolEventHashInputV2 {}

impl sealed::Sealed for PostTurnCompletionHashInputV2 {}
impl ValidatedCanonicalV1 for PostTurnCompletionHashInputV2 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v2(self.schema_version)?;
        self.binding.validate()?;
        validate_ref(
            &self.protocol_event_ref,
            AuthorityObjectKindV1::PostTurnProtocolEvent,
        )?;
        if self.protocol_event_ref.schema_version != 2 {
            return Err(ValidationError(
                "direct-host completion requires a V2 protocol event",
            ));
        }
        match (&self.outcome, &self.attention_cut) {
            (PostTurnCompletionOutcomeV1::ResumableClean, Some(cut)) => {
                cut.validate()?;
                if self.binding.input_acceptance_ref.is_none()
                    || cut.authority_store_id != self.binding.authority_store_id
                    || cut.orchestration_session_id != self.binding.orchestration_session_id
                {
                    return Err(ValidationError(
                        "direct-host completion attention scope does not join",
                    ));
                }
            }
            (
                PostTurnCompletionOutcomeV1::TerminalClean
                | PostTurnCompletionOutcomeV1::TerminalFailure,
                None,
            ) => {}
            _ => {
                return Err(ValidationError(
                    "direct-host completion has no exact attention disposition",
                ))
            }
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for PostTurnCompletionHashInputV2 {}

impl sealed::Sealed for PostTurnCompletionHashInputV3 {}
impl ValidatedCanonicalV1 for PostTurnCompletionHashInputV3 {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != 3 {
            return Err(ValidationError(
                "direct-host completion requires schema version 3",
            ));
        }
        // Validate the shared fields under the unchanged V2 rules. This view is
        // never serialized or hashed as a replacement for the V3 object.
        PostTurnCompletionHashInputV2 {
            schema_version: 2,
            binding: self.binding.clone(),
            protocol_event_ref: self.protocol_event_ref.clone(),
            outcome: self.outcome,
            attention_cut: self.attention_cut.clone(),
            completed_at: self.completed_at.clone(),
        }
        .validate()?;
        match (self.outcome, &self.produced_resume_handle_ref) {
            (PostTurnCompletionOutcomeV1::ResumableClean, Some(reference)) => {
                validate_ref(reference, AuthorityObjectKindV1::ResumeHandle)?;
                if reference.schema_version != 1 {
                    return Err(ValidationError(
                        "produced continuation requires ResumeHandle schema1",
                    ));
                }
            }
            (
                PostTurnCompletionOutcomeV1::TerminalClean
                | PostTurnCompletionOutcomeV1::TerminalFailure,
                None,
            ) => {}
            _ => {
                return Err(ValidationError(
                    "completion continuation disagrees with terminal disposition",
                ))
            }
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for PostTurnCompletionHashInputV3 {}

fn validate_world_controller_selection(
    native: &transport_api_types::ResolvedAgentSelectionV1,
    requested: &transport_api_types::ResolvedAgentSelectionV1,
    backend_id: &str,
) -> Result<(), ValidationError> {
    native
        .validate()
        .map_err(|_| ValidationError("invalid native controller selection"))?;
    requested
        .validate()
        .map_err(|_| ValidationError("invalid requested World selection"))?;
    if native.placement != transport_api_types::AgentSelectionPlacementV1::Host
        || requested.placement != transport_api_types::AgentSelectionPlacementV1::World
        || native.backend_id != backend_id
    {
        return Err(ValidationError("World controller selection roles differ"));
    }
    Ok(())
}

impl sealed::Sealed for WorldControllerWorkAttentionCutV1 {}
impl ValidatedCanonicalV1 for WorldControllerWorkAttentionCutV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        use std::collections::BTreeSet;
        schema_v1(self.schema_version)?;
        required(&self.authority_store_id)?;
        required(&self.orchestration_session_id)?;
        validate_world(&self.world_binding)?;
        if self.hsa_root_revision == 0
            || !lower_hex(&self.b1_registry_sha256, 64)
            || self
                .e2_registry_sha256
                .as_ref()
                .is_some_and(|hash| !lower_hex(hash, 64))
            || self
                .supervisor_sha256
                .as_ref()
                .is_some_and(|hash| !lower_hex(hash, 64))
            || self
                .admission_registry_sha256
                .as_ref()
                .is_some_and(|hash| !lower_hex(hash, 64))
            || self.ledger_files_sha256.iter().any(|(path, hash)| {
                path.is_empty()
                    || path.split('/').any(|part| {
                        part.is_empty() || matches!(part, "." | "..") || part.contains('\\')
                    })
                    || !lower_hex(hash, 64)
            })
        {
            return Err(ValidationError(
                "invalid World controller physical source cut",
            ));
        }
        for values in [
            &self.proposal_request_ids,
            &self.acceptance_record_ids,
            &self.pending_request_ids,
            &self.unresolved_work_acceptance_ids,
            &self.unresolved_attention_acceptance_ids,
            &self.e2_session_source_ids,
            &self.unresolved_e2_source_ids,
            &self.unresolved_admission_participant_ids,
            &self.unclassified_hsa_retained_participant_ids,
        ] {
            if values.iter().any(String::is_empty)
                || values.windows(2).any(|pair| pair[0] >= pair[1])
            {
                return Err(ValidationError(
                    "World source identities are not unique and ordered",
                ));
            }
        }
        if self.proposal_request_ids.len()
            != self.acceptance_record_ids.len() + self.pending_request_ids.len()
            || !self
                .pending_request_ids
                .iter()
                .all(|id| self.proposal_request_ids.contains(id))
        {
            return Err(ValidationError("World proposal coverage is incomplete"));
        }
        let mut covered = BTreeSet::new();
        if (!self.e2_session_source_ids.is_empty() && self.e2_registry_sha256.is_none())
            || !self
                .unresolved_e2_source_ids
                .iter()
                .all(|id| self.e2_session_source_ids.contains(id))
            || (!self.unresolved_admission_participant_ids.is_empty()
                && self.admission_registry_sha256.is_none())
            || !self
                .unresolved_attention_acceptance_ids
                .iter()
                .all(|id| self.acceptance_record_ids.contains(id))
        {
            return Err(ValidationError(
                "World pending source coverage is incomplete",
            ));
        }
        for id in &self.unresolved_work_acceptance_ids {
            covered.insert(id.as_str());
        }
        for work in &self.completed_work {
            for value in [
                &work.acceptance_record_id,
                &work.stream_id,
                &work.terminal_event_id,
                &work.e2_commitment_id,
            ] {
                required(value)?;
            }
            validate_work_identity(&work.accepted_work_identity)?;
            if work.acceptance_record_revision == 0
                || work.terminal_event_sequence == 0
                || !lower_hex(&work.e2_commitment_hash, 64)
                || !covered.insert(&work.acceptance_record_id)
                || self.e2_registry_sha256.is_none()
                || self.supervisor_sha256.is_none()
            {
                return Err(ValidationError(
                    "invalid or duplicate completed World work source",
                ));
            }
        }
        if covered
            != self
                .acceptance_record_ids
                .iter()
                .map(String::as_str)
                .collect()
        {
            return Err(ValidationError("World acceptance coverage is incomplete"));
        }
        let mut attention = BTreeSet::new();
        for entry in &self.unresolved_attention {
            required(&entry.obligation_id)?;
            validate_canonical_commitment(&entry.canonical_record_commitment)?;
            if entry.obligation_revision == 0 || !attention.insert(&entry.obligation_id) {
                return Err(ValidationError(
                    "invalid or duplicate World attention source",
                ));
            }
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for WorldControllerWorkAttentionCutV1 {}

impl sealed::Sealed for StartContinuationHandleHashInputV3 {}
impl ValidatedCanonicalV1 for StartContinuationHandleHashInputV3 {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != 3 {
            return Err(ValidationError("World Start continuation requires schema3"));
        }
        let core = &self.continuation;
        core.validate()?;
        validate_world(&self.world_binding)?;
        self.work_attention_cut.validate()?;
        validate_world_controller_selection(
            &self.native_selection,
            &self.requested_selection,
            &core.backend_id,
        )?;
        let StartContinuationHandleStateV2::Settled {
            protocol_actor,
            thread_id,
            turn_id,
            provider_event_kind,
            completion_kind,
            obligation_snapshot,
            ..
        } = &core.state
        else {
            return Err(ValidationError(
                "World Start continuation must contain exact settled Start data",
            ));
        };
        let native_outcome = match completion_kind {
            StartTurnCompletionKindV1::ResumableClean
            | StartTurnCompletionKindV1::TerminalClean => {
                matches!(
                    provider_event_kind.as_str(),
                    "turn.completed" | "result.success"
                )
            }
            StartTurnCompletionKindV1::TerminalFailure { .. } => {
                matches!(provider_event_kind.as_str(), "turn.failed" | "result.error")
            }
        };
        if self.work_attention_cut.authority_store_id != core.authority_store_id
            || self.work_attention_cut.orchestration_session_id != core.orchestration_session_id
            || self.work_attention_cut.world_binding != self.world_binding
            || thread_id != &core.internal_uaa_session_id
            || obligation_snapshot.is_some()
            || !native_outcome
            || (provider_event_kind.starts_with("result.") && thread_id != turn_id)
            || *protocol_actor
                != (HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                    participant_id: core.participant_id.clone(),
                })
        {
            return Err(ValidationError(
                "World Start completion source/actor/native join differs",
            ));
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for StartContinuationHandleHashInputV3 {}

impl sealed::Sealed for WorldControllerResumeBindingV1 {}
impl ValidatedCanonicalV1 for WorldControllerResumeBindingV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v1(self.schema_version)?;
        for value in [
            &self.authority_store_id,
            &self.orchestration_session_id,
            &self.shell_trace_session_id,
            &self.authoritative_participant_id,
            &self.original_start_intent_id,
            &self.original_start_run_id,
            &self.intent_id,
            &self.issuer_request_id,
            &self.claim_id,
            &self.claimant_attempt_id,
            &self.run_id,
            &self.policy_revision,
        ] {
            required(value)?;
        }
        validate_directory(&self.bootstrap_home)?;
        validate_workspace(&self.workspace_binding)?;
        validate_world(&self.world_binding)?;
        validate_world_controller_selection(
            &self.native_selection,
            &self.requested_selection,
            &self.native_selection.backend_id,
        )?;
        if self.workspace_binding.authority_store_id != self.authority_store_id
            || self.workspace_binding.authority_store_root != self.bootstrap_home
            || self.authority_revision_observed == 0
            || self.intent_revision_observed == 0
            || !self
                .authoritative_lineage
                .contains(&self.authoritative_participant_id)
            || self.authoritative_lineage.iter().any(String::is_empty)
            || self
                .authoritative_lineage
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.authoritative_lineage.len()
        {
            return Err(ValidationError(
                "World controller persisted binding differs",
            ));
        }
        validate_canonical_commitment(&self.authority_record_commitment)?;
        validate_canonical_commitment(&self.payload_commitment)?;
        for (reference, kind) in [
            (&self.policy_ref, AuthorityObjectKindV1::Policy),
            (&self.descriptor_ref, AuthorityObjectKindV1::AgentDescriptor),
            (
                &self.initial_application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
            ),
            (
                &self.transition_input_ref,
                AuthorityObjectKindV1::TransitionInput,
            ),
            (
                &self.input_acceptance_ref,
                AuthorityObjectKindV1::InputAcceptance,
            ),
            (
                &self.consumed_resume_handle_ref,
                AuthorityObjectKindV1::ResumeHandle,
            ),
        ] {
            validate_ref(reference, kind)?;
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for WorldControllerResumeBindingV1 {}

impl sealed::Sealed for PostTurnCompletionHashInputV4 {}
impl ValidatedCanonicalV1 for PostTurnCompletionHashInputV4 {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != 4 {
            return Err(ValidationError(
                "World controller completion requires schema4",
            ));
        }
        self.binding.validate()?;
        self.native_terminal.validate()?;
        self.work_attention_cut.validate()?;
        required(&self.event_id)?;
        if self.native_terminal.backend_id != self.binding.native_selection.backend_id
            || self.work_attention_cut.authority_store_id != self.binding.authority_store_id
            || self.work_attention_cut.orchestration_session_id
                != self.binding.orchestration_session_id
            || self.work_attention_cut.world_binding != self.binding.world_binding
            || self.protocol_actor
                != (HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                    participant_id: self.binding.authoritative_participant_id.clone(),
                })
        {
            return Err(ValidationError(
                "World controller completion actor/native/source differs",
            ));
        }
        let success = matches!(
            self.native_terminal.terminal_event_kind.as_str(),
            "turn.completed" | "result.success"
        );
        match (&self.kind, &self.produced_resume_handle_ref) {
            (HostPostTurnProtocolEventKindV1::ResumableClean, Some(reference)) if success => {
                validate_ref(reference, AuthorityObjectKindV1::ResumeHandle)?;
                if reference.schema_version != 1 {
                    return Err(ValidationError(
                        "World continuation must be target-owned schema1",
                    ));
                }
            }
            (HostPostTurnProtocolEventKindV1::TerminalClean, None) if success => {}
            (
                HostPostTurnProtocolEventKindV1::TerminalFailure {
                    reason: HostPostTurnTerminalReasonV1::TargetFailedAfterInputAcceptance,
                },
                None,
            ) if !success => {}
            _ => {
                return Err(ValidationError(
                    "World controller native outcome/continuation differs",
                ))
            }
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for PostTurnCompletionHashInputV4 {}

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

impl sealed::Sealed for TerminalHandoffHashInputV2 {}
impl ValidatedCanonicalV1 for TerminalHandoffHashInputV2 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v2(self.schema_version)?;
        required(&self.intent_id)?;
        required(&self.run_id)?;
        validate_canonical_commitment(&self.payload_commitment)?;
        if let Some(value) = &self.application_result_ref {
            validate_ref(value, AuthorityObjectKindV1::ApplicationResult)?;
        }
        if let Some(value) = &self.input_acceptance_ref {
            validate_ref(value, AuthorityObjectKindV1::InputAcceptance)?;
        }
        if let Some(value) = &self.startup_ownership_result_ref {
            validate_ref(value, AuthorityObjectKindV1::StartupOwnershipResult)?;
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
impl sealed::Sealed for StartContinuationHandleHashInputV2 {}
impl ValidatedCanonicalV1 for StartContinuationHandleHashInputV2 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema_v2(self.schema_version)?;
        for value in [
            &self.authority_store_id,
            &self.orchestration_session_id,
            &self.participant_id,
            &self.backend_id,
            &self.protocol,
            &self.internal_uaa_session_id,
            &self.start_intent_id,
            &self.start_issuer_request_id,
            &self.start_run_id,
        ] {
            required(value)?;
        }
        validate_commitment(&self.start_payload_commitment)?;
        validate_ref(
            &self.start_application_result_ref,
            AuthorityObjectKindV1::ApplicationResult,
        )?;
        validate_commitment(&self.authority_record_commitment_before)?;
        let expected_revision_after =
            self.authority_revision_before
                .checked_add(1)
                .ok_or(ValidationError(
                    "Start continuation authority revision overflows",
                ))?;
        if self.authority_revision_before == 0
            || self.authority_revision_after != expected_revision_after
        {
            return Err(ValidationError(
                "Start continuation authority revisions are not contiguous",
            ));
        }
        match &self.state {
            StartContinuationHandleStateV2::Registered {
                exchange_id,
                exchange_sequence,
                provider_event_kind,
                evidence_sha256,
                ..
            } => {
                required(exchange_id)?;
                required(provider_event_kind)?;
                if *exchange_sequence == 0 {
                    return Err(ValidationError(
                        "Start continuation exchange sequence is invalid",
                    ));
                }
                if !lower_hex(evidence_sha256, 64) {
                    return Err(ValidationError(
                        "Start continuation exchange evidence is invalid",
                    ));
                }
            }
            StartContinuationHandleStateV2::Settled {
                registered_resume_handle_ref,
                protocol_actor,
                event_id,
                event_sequence,
                provider_event_kind,
                thread_id,
                turn_id,
                evidence_sha256,
                completion_kind,
                obligation_snapshot,
                ..
            } => {
                validate_ref(
                    registered_resume_handle_ref,
                    AuthorityObjectKindV1::ResumeHandle,
                )?;
                if registered_resume_handle_ref.schema_version != 2 {
                    return Err(ValidationError(
                        "Start settlement registration ref requires schema version 2",
                    ));
                }
                match protocol_actor {
                    HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                        participant_id,
                    } => required(participant_id)?,
                    HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
                        claim_id,
                        claimant_attempt_id,
                    } => {
                        required(claim_id)?;
                        required(claimant_attempt_id)?;
                    }
                }
                required(event_id)?;
                required(provider_event_kind)?;
                required(thread_id)?;
                required(turn_id)?;
                if *event_sequence == 0 {
                    return Err(ValidationError(
                        "Start settlement event sequence is invalid",
                    ));
                }
                if !lower_hex(evidence_sha256, 64) {
                    return Err(ValidationError(
                        "Start settlement completion evidence is invalid",
                    ));
                }
                if let StartTurnCompletionKindV1::TerminalFailure { reason } = completion_kind {
                    required(reason)?;
                }
                if obligation_snapshot.is_some()
                    && !matches!(completion_kind, StartTurnCompletionKindV1::ResumableClean)
                {
                    return Err(ValidationError(
                        "terminal Start settlement cannot carry an obligation snapshot",
                    ));
                }
                if let Some(snapshot) = obligation_snapshot.as_deref() {
                    snapshot.validate()?;
                }
            }
        }
        Ok(())
    }
}
impl CanonicalHashInputV1 for StartContinuationHandleHashInputV2 {}
impl CanonicalHashInputV1 for PolicyObjectHashInputV1 {}
impl CanonicalHashInputV1 for RetainedWorkerObjectHashInputV1 {}
impl CanonicalHashInputV1 for AuthoritativeLineageHashInputV1 {}
impl CanonicalHashInputV1 for DurableSessionAuthorityHashInputV1 {}
impl CanonicalHashInputV1 for ForkSuccessorAllocationRequestHashInputV1 {}
impl CanonicalHashInputV1 for HostSessionTransitionPayloadHashInputV1 {}
impl CanonicalHashInputV1 for HostSessionStopPayloadHashInputV1 {}
impl CanonicalHashInputV1 for HostSessionStopResultHashInputV1 {}
impl CanonicalHashInputV1 for ApplicationResultHashInputV1 {}
impl CanonicalHashInputV1 for InputAcceptanceHashInputV1 {}
impl CanonicalHashInputV1 for StartupOwnershipResultHashInputV1 {}
impl CanonicalHashInputV1 for ObligationSnapshotRecordHashInputV1 {}
impl CanonicalHashInputV1 for ObligationSnapshotHashInputV1 {}
impl CanonicalHashInputV1 for PostTurnProtocolEventHashInputV1 {}
impl CanonicalHashInputV1 for PostTurnCompletionHashInputV1 {}
impl CanonicalHashInputV1 for TerminalHandoffHashInputV1 {}
impl CanonicalHashInputV1 for TerminalHandoffHashInputV2 {}

pub(crate) mod sealed {
    pub trait Sealed {}
}

#[cfg(test)]
mod direct_host_codec_tests {
    use super::*;
    use crate::execution::agent_runtime::host_session_authority::{canonical_json, hash};

    fn commitment() -> AuthorityObjectCommitmentV1 {
        AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "1".repeat(64),
        }
    }

    fn reference(kind: AuthorityObjectKindV1) -> AuthorityObjectRefV1 {
        AuthorityObjectRefV1 {
            ref_id: format!("ao_{}", "1".repeat(32)),
            object_kind: kind,
            schema_version: 1,
            commitment: if kind == AuthorityObjectKindV1::TransitionInput {
                AuthorityObjectCommitmentV1::StoreHmacSha256 {
                    key_id: format!("ak_{}", "1".repeat(32)),
                    domain: "substrate.a1.transition-input.v1".into(),
                    digest_hex: "1".repeat(64),
                }
            } else {
                commitment()
            },
        }
    }

    fn binding() -> DirectHostResumeBindingV1 {
        let home = CanonicalDirectoryV1 {
            physical_path: "/test/home".into(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: 1,
                inode: 2,
            },
        };
        DirectHostResumeBindingV1 {
            schema_version: 1,
            authority_store_id: format!("as_{}", "1".repeat(32)),
            bootstrap_home: home.clone(),
            workspace_binding: WorkspaceBindingV1 {
                workspace_root: CanonicalDirectoryV1 {
                    physical_path: "/test/workspace".into(),
                    ..home.clone()
                },
                authority_store_root: home,
                authority_store_id: format!("as_{}", "1".repeat(32)),
            },
            orchestration_session_id: "session".into(),
            shell_trace_session_id: "trace".into(),
            authoritative_participant_id: "participant".into(),
            authoritative_lineage: vec!["participant".into()],
            authority_history_commitment: commitment(),
            authority_revision_observed: 4,
            authority_record_commitment: commitment(),
            policy_ref: None,
            policy_revision: None,
            descriptor_ref: reference(AuthorityObjectKindV1::AgentDescriptor),
            host_attach_contract_ref: reference(AuthorityObjectKindV1::HostAttachContract),
            intent_id: "intent".into(),
            issuer_request_id: "request".into(),
            payload_commitment: commitment(),
            intent_revision_observed: 3,
            claim_id: "claim".into(),
            claimant_attempt_id: "attempt".into(),
            run_id: "run".into(),
            initial_application_result_ref: reference(AuthorityObjectKindV1::ApplicationResult),
            transition_input_ref: reference(AuthorityObjectKindV1::TransitionInput),
            input_acceptance_ref: Some(reference(AuthorityObjectKindV1::InputAcceptance)),
            consumed_resume_handle_ref: reference(AuthorityObjectKindV1::ResumeHandle),
        }
    }

    fn native() -> DirectHostNativeTerminalEvidenceV1 {
        DirectHostNativeTerminalEvidenceV1 {
            schema_version: 1,
            backend_id: "backend".into(),
            protocol: "protocol".into(),
            native_session_id: "thread".into(),
            thread_id: "thread".into(),
            turn_id: "turn".into(),
            sequence: DirectHostNativeSequenceV1::Codex {
                resumed_thread_id: "thread".into(),
                started_turn_id: "turn".into(),
            },
            terminal_event_kind: "turn.completed".into(),
            raw_event_sha256: "2".repeat(64),
            terminal_ordinal: 1,
        }
    }

    fn event() -> PostTurnProtocolEventHashInputV2 {
        PostTurnProtocolEventHashInputV2 {
            schema_version: 2,
            binding: binding(),
            protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: "participant".into(),
            },
            event_id: "run-terminal".into(),
            kind: HostPostTurnProtocolEventKindV1::ResumableClean,
            native_terminal: Some(native()),
            emitted_at: TimestampV1::parse("2026-10-29T00:00:00.000000000Z").unwrap(),
        }
    }

    #[test]
    fn test_direct_host_resume_codec_requires_exact_native_sequence() {
        native().validate().unwrap();
        for change in 0..6 {
            let mut value = native();
            match change {
                0 => value.thread_id = "other-thread".into(),
                1 => value.turn_id = "other-turn".into(),
                2 => value.terminal_event_kind = "result.success".into(),
                3 => value.raw_event_sha256 = "X".repeat(64),
                4 => value.terminal_ordinal = 2,
                _ => value.native_session_id.clear(),
            }
            assert!(value.validate().is_err(), "accepted substitution {change}");
        }
        let mut claude = native();
        claude.sequence = DirectHostNativeSequenceV1::ClaudeCode {
            initialized_session_id: "thread".into(),
        };
        claude.turn_id = "thread".into();
        claude.terminal_event_kind = "result.success".into();
        claude.validate().unwrap();
        claude.turn_id = "turn".into();
        assert!(claude.validate().is_err());
    }

    #[test]
    fn test_direct_host_resume_codec_preserves_actor_and_input_failure_order() {
        event().validate().unwrap();
        let mut value = event();
        value.binding.input_acceptance_ref = None;
        assert!(value.validate().is_err());
        value.kind = HostPostTurnProtocolEventKindV1::TerminalFailure {
            reason: HostPostTurnTerminalReasonV1::ResumeRuntimeCreationRejected,
        };
        value.protocol_actor = HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
            claim_id: "claim".into(),
            claimant_attempt_id: "attempt".into(),
        };
        assert!(value.validate().is_err()); // A launch rejection cannot carry native evidence.
        value.native_terminal = None;
        value.validate().unwrap();
        value.binding.input_acceptance_ref = binding().input_acceptance_ref;
        assert!(value.validate().is_err()); // Post-acceptance launch failure is never pre-acceptance rejection.
        value = event();
        value.protocol_actor = HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: "other".into(),
        };
        assert!(value.validate().is_err());
        value = event();
        value.native_terminal.as_mut().unwrap().terminal_event_kind = "turn.failed".into();
        assert!(value.validate().is_err());
        value.kind = HostPostTurnProtocolEventKindV1::TerminalFailure {
            reason: HostPostTurnTerminalReasonV1::TargetFailedAfterInputAcceptance,
        };
        value.validate().unwrap();
    }

    #[test]
    fn test_direct_host_resume_codec_does_not_reinterpret_world_v1() {
        let value = event();
        let bytes = hash::canonical_object_bytes(
            AuthorityObjectKindV1::PostTurnProtocolEvent,
            hash::CanonicalObjectHashInputV1::PostTurnProtocolEventV2(&value),
        )
        .unwrap();
        assert_eq!(
            canonical_json::from_slice::<PostTurnProtocolEventHashInputV2>(&bytes).unwrap(),
            value
        );
        assert!(canonical_json::from_slice::<PostTurnProtocolEventHashInputV1>(&bytes).is_err());
        let mut object = serde_json::to_value(&value).unwrap();
        object["world_binding"] = serde_json::json!({"world_id": "forged", "world_generation": 1});
        assert!(serde_json::from_value::<PostTurnProtocolEventHashInputV2>(object).is_err());
        let mut wrong_version = value;
        wrong_version.schema_version = 1;
        assert!(wrong_version.validate().is_err());
        assert!(object_schema_version(AuthorityObjectKindV1::PostTurnProtocolEvent, 3).is_err());
        assert!(object_schema_version(AuthorityObjectKindV1::ObligationSnapshot, 2).is_err());
    }

    #[test]
    fn test_completion_v3_requires_exact_continuation_and_preserves_v2_codec() {
        let mut event_ref = reference(AuthorityObjectKindV1::PostTurnProtocolEvent);
        event_ref.schema_version = 2;
        let v2 = PostTurnCompletionHashInputV2 {
            schema_version: 2,
            binding: binding(),
            protocol_event_ref: event_ref,
            outcome: PostTurnCompletionOutcomeV1::TerminalClean,
            attention_cut: None,
            completed_at: TimestampV1::parse("2026-10-29T00:00:00.000000000Z").unwrap(),
        };
        let old = hash::canonical_object_bytes(
            AuthorityObjectKindV1::PostTurnCompletion,
            hash::CanonicalObjectHashInputV1::PostTurnCompletionV2(&v2),
        )
        .unwrap();
        let mut v3 = PostTurnCompletionHashInputV3 {
            schema_version: 3,
            binding: v2.binding.clone(),
            protocol_event_ref: v2.protocol_event_ref.clone(),
            outcome: v2.outcome,
            attention_cut: None,
            completed_at: v2.completed_at.clone(),
            produced_resume_handle_ref: None,
        };
        v3.validate().unwrap();
        let bytes = hash::canonical_object_bytes(
            AuthorityObjectKindV1::PostTurnCompletion,
            hash::CanonicalObjectHashInputV1::PostTurnCompletionV3(&v3),
        )
        .unwrap();
        assert_eq!(
            canonical_json::from_slice::<PostTurnCompletionHashInputV3>(&bytes).unwrap(),
            v3
        );
        assert!(canonical_json::from_slice::<PostTurnCompletionHashInputV2>(&bytes).is_err());
        assert!(canonical_json::from_slice::<PostTurnCompletionHashInputV3>(&old).is_err());
        assert_eq!(
            hash::canonical_object_bytes(
                AuthorityObjectKindV1::PostTurnCompletion,
                hash::CanonicalObjectHashInputV1::PostTurnCompletionV2(&v2)
            )
            .unwrap(),
            old
        );
        let mut unknown = serde_json::to_value(&v3).unwrap();
        unknown["future"] = serde_json::json!(1);
        assert!(serde_json::from_value::<PostTurnCompletionHashInputV3>(unknown).is_err());
        v3.produced_resume_handle_ref = Some(reference(AuthorityObjectKindV1::ResumeHandle));
        assert!(
            v3.validate().is_err(),
            "terminal clean cannot produce a continuation"
        );
        v3.outcome = PostTurnCompletionOutcomeV1::TerminalFailure;
        assert!(
            v3.validate().is_err(),
            "terminal failure cannot produce a continuation"
        );
        v3.outcome = PostTurnCompletionOutcomeV1::ResumableClean;
        v3.attention_cut = Some(DirectHostCanonicalAttentionCutV1 {
            schema_version: 1,
            authority_store_id: v3.binding.authority_store_id.clone(),
            orchestration_session_id: v3.binding.orchestration_session_id.clone(),
            genesis_commitment: commitment(),
            source_head_commitment: commitment(),
            ledger_head_commitment: commitment(),
            receipt_high_water: 0,
            ledger_revision: 0,
            source_protocol: "substrate.direct-host-inbox.resume-only.v1".into(),
            unresolved_attention: vec![],
        });
        v3.validate().unwrap();
        v3.produced_resume_handle_ref = None;
        assert!(
            v3.validate().is_err(),
            "resumable completion must produce its continuation"
        );
        for (kind, version) in [
            (AuthorityObjectKindV1::ResumeHandle, 2),
            (AuthorityObjectKindV1::ResumeHandle, 3),
            (AuthorityObjectKindV1::PostTurnCompletion, 1),
        ] {
            let mut r = reference(kind);
            r.schema_version = version;
            v3.produced_resume_handle_ref = Some(r);
            assert!(v3.validate().is_err());
        }
        for kind in [
            AuthorityObjectKindV1::PostTurnProtocolEvent,
            AuthorityObjectKindV1::TerminalHandoff,
            AuthorityObjectKindV1::ApplicationResult,
        ] {
            assert!(object_schema_version(kind, 3).is_err());
        }
        object_schema_version(AuthorityObjectKindV1::ResumeHandle, 3).unwrap();
        assert!(object_schema_version(AuthorityObjectKindV1::ResumeHandle, 4).is_err());
        object_schema_version(AuthorityObjectKindV1::PostTurnCompletion, 3).unwrap();
    }

    #[test]
    fn test_direct_host_resume_codec_requires_explicit_complete_attention_cut() {
        let mut event_ref = reference(AuthorityObjectKindV1::PostTurnProtocolEvent);
        event_ref.schema_version = 2;
        let mut value = PostTurnCompletionHashInputV2 {
            schema_version: 2,
            binding: binding(),
            protocol_event_ref: event_ref,
            outcome: PostTurnCompletionOutcomeV1::ResumableClean,
            attention_cut: None,
            completed_at: TimestampV1::parse("2026-10-29T00:00:00.000000000Z").unwrap(),
        };
        assert!(value.validate().is_err());
        value.attention_cut = Some(DirectHostCanonicalAttentionCutV1 {
            schema_version: 1,
            authority_store_id: value.binding.authority_store_id.clone(),
            orchestration_session_id: value.binding.orchestration_session_id.clone(),
            genesis_commitment: commitment(),
            source_head_commitment: commitment(),
            ledger_head_commitment: commitment(),
            receipt_high_water: 0,
            ledger_revision: 0,
            source_protocol: "substrate.direct-host-inbox.resume-only.v1".into(),
            unresolved_attention: vec![],
        });
        value.validate().unwrap(); // Structural validation alone does not authenticate this cut.
        value
            .attention_cut
            .as_mut()
            .unwrap()
            .orchestration_session_id = "other".into();
        assert!(value.validate().is_err());
        value.attention_cut = None;
        value.outcome = PostTurnCompletionOutcomeV1::TerminalFailure;
        value.validate().unwrap();
        value.protocol_event_ref.schema_version = 1;
        assert!(value.validate().is_err());
    }
}

#[cfg(test)]
mod direct_host_world_v1_preservation_tests {
    use super::*;
    use crate::execution::agent_runtime::host_session_authority::{canonical_json, hash};

    #[test]
    fn test_direct_host_resume_codec_keeps_world_v1_event_reference_strict() {
        let mut completion: PostTurnCompletionHashInputV1 =
            canonical_json::from_slice(include_bytes!("testdata/post-turn-completion.json"))
                .unwrap();
        assert_eq!(
            hash::canonical_sha256(&completion).unwrap(),
            "5f09e17c0e0822f97e04d13977fb4b009b62a49ad92703b629bb953d82ec704c"
        );
        for version in [2, 3] {
            completion.protocol_event_ref.schema_version = version;
            assert!(
                hash::canonical_sha256(&completion).is_err(),
                "world V1 completion admitted a V{version} event reference"
            );
        }
    }
}
