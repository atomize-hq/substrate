//! Dedicated HSA-owned Stop issuance, delivery, and terminal closeout.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::facade::{AuthorityObservationV1, HostSessionAuthority, ResolvedCurrentAuthorityV1};
use super::hash::canonical_sha256;
use super::schema::{
    AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, CanonicalDirectoryV1,
    DurableSessionAuthorityHashInputV1, HostSessionPostureV1, HostSessionStopPayloadHashInputV1,
    HostSessionStopResultHashInputV1, HostSessionTransitionCallerKindV1,
    HostSessionTransitionCallerV1, TimestampV1,
};
use super::store;
use super::store_schema::{
    DurableSessionAuthorityV1, HostSessionStopIntentStateV1, HostSessionStopIntentV1,
    SessionNamespaceRecordV1, StateRootV3,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IssueHostSessionStopRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) expected_authority: AuthorityObservationV1,
    pub(crate) authoritative_participant_id: String,
    pub(crate) authoritative_lineage: Vec<String>,
    pub(crate) issued_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HostSessionStopIssueOutcomeV1 {
    Issued(HostSessionStopIntentV1),
    Joined(HostSessionStopIntentV1),
    AlreadyTerminal(HostSessionStopResultV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcceptHostSessionStopDeliveryRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) acceptance_id: String,
    pub(crate) accepted_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HostSessionStopDeliveryOutcomeV1 {
    Accepted(HostSessionStopIntentV1),
    Joined(HostSessionStopIntentV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompleteHostSessionStopRequestV1 {
    pub(crate) intent_id: String,
    pub(crate) request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) delivery_acceptance_id: Option<String>,
    pub(crate) result_id: String,
    pub(crate) completed_at: TimestampV1,
}

pub(crate) type HostSessionStopResultV1 = HostSessionStopResultHashInputV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HostSessionStopCompletionOutcomeV1 {
    Completed(HostSessionStopResultV1),
    Joined(HostSessionStopResultV1),
}

/// Exact private-delivery identity. The owner revalidates every field against
/// the durable HSA Stop intent before acknowledging delivery.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionStopDeliveryV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_participant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostSessionStopProtocolError(String);

impl fmt::Display for HostSessionStopProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for HostSessionStopProtocolError {}

impl HostSessionAuthority {
    pub(crate) fn issue_stop(
        &self,
        resolved: &ResolvedCurrentAuthorityV1,
        request: &IssueHostSessionStopRequestV1,
    ) -> Result<HostSessionStopIssueOutcomeV1, HostSessionStopProtocolError> {
        validate_issue_request(resolved, request)?;
        let current = self.read_a12b_root().map_err(stop_error)?;
        if let Some(existing) = current.stop_transaction_map.get(&request.intent_id) {
            validate_issue_join(existing, resolved, request)?;
            return match &existing.state {
                HostSessionStopIntentStateV1::Completed { .. } => Ok(
                    HostSessionStopIssueOutcomeV1::AlreadyTerminal(stop_result(existing)?),
                ),
                _ => Ok(HostSessionStopIssueOutcomeV1::Joined(existing.clone())),
            };
        }
        if current.stop_transaction_map.values().any(|existing| {
            existing.orchestration_session_id == request.orchestration_session_id
                || existing.request_id == request.request_id
        }) {
            return Err(error("conflicting HSA Stop transaction already exists"));
        }
        let current_authority = exact_authority(&current, &request.orchestration_session_id)?;
        if current.root_revision != request.expected_authority.root_revision
            || current.authority_store_id != request.expected_authority.authority_store_id
            || current.bootstrap_home != request.expected_authority.bootstrap_home
            || current_authority != &resolved.authority
            || authority_commitment(current_authority)?
                != request.expected_authority.authority_record_commitment
            || lineage_commitment(current_authority)?
                != request.expected_authority.authoritative_lineage_commitment
        {
            return Err(error(
                "HSA Stop authority precondition is stale or mismatched",
            ));
        }
        let payload = stop_payload(&current, current_authority, request);
        let intent = HostSessionStopIntentV1 {
            schema_version: 1,
            intent_id: request.intent_id.clone(),
            request_id: request.request_id.clone(),
            authority_store_id: current.authority_store_id.clone(),
            bootstrap_home: current.bootstrap_home.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: current_authority.shell_trace_session_id.clone(),
            caller: request.caller.clone(),
            payload_commitment: commitment(&payload)?,
            expected_root_revision: current.root_revision,
            authority_before: Box::new(current_authority.clone()),
            authority_record_commitment_before: authority_commitment(current_authority)?,
            authoritative_lineage_commitment_before: lineage_commitment(current_authority)?,
            authoritative_participant_id: request.authoritative_participant_id.clone(),
            issued_at: request.issued_at.clone(),
            updated_at: request.issued_at.clone(),
            state: HostSessionStopIntentStateV1::Issued,
        };
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        proposed
            .stop_transaction_map
            .insert(intent.intent_id.clone(), intent.clone());
        super::facade::validate_v3_schema_with_external_exact_authority_history(&proposed)
            .map_err(stop_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(stop_error)?;
        Ok(HostSessionStopIssueOutcomeV1::Issued(intent))
    }

    pub(crate) fn stop_transaction_for_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<HostSessionStopIntentV1>, HostSessionStopProtocolError> {
        let current = self.read_a12b_root().map_err(stop_error)?;
        let mut matches = current
            .stop_transaction_map
            .values()
            .filter(|intent| intent.orchestration_session_id == orchestration_session_id);
        let result = matches.next().cloned();
        if matches.next().is_some() {
            return Err(error("multiple HSA Stop transactions match one session"));
        }
        Ok(result)
    }

    pub(crate) fn accept_stop_delivery(
        &self,
        request: &AcceptHostSessionStopDeliveryRequestV1,
    ) -> Result<HostSessionStopDeliveryOutcomeV1, HostSessionStopProtocolError> {
        validate_delivery_request(request)?;
        let current = self.read_a12b_root().map_err(stop_error)?;
        let intent = current
            .stop_transaction_map
            .get(&request.intent_id)
            .ok_or_else(|| error("exact HSA Stop intent was not found"))?;
        validate_delivery_identity(intent, request)?;
        match &intent.state {
            HostSessionStopIntentStateV1::DeliveryAccepted {
                acceptance_id,
                accepted_by_participant_id,
                accepted_at,
            } if acceptance_id == &request.acceptance_id
                && accepted_by_participant_id == &request.authoritative_participant_id
                && accepted_at == &request.accepted_at =>
            {
                return Ok(HostSessionStopDeliveryOutcomeV1::Joined(intent.clone()));
            }
            HostSessionStopIntentStateV1::Issued => {}
            _ => return Err(error("HSA Stop delivery conflicts with committed state")),
        }
        if intent.authority_before.lifecycle_posture != HostSessionPostureV1::ActiveAttached
            || exact_authority(&current, &request.orchestration_session_id)?
                != intent.authority_before.as_ref()
            || request.accepted_at.as_str() < intent.issued_at.as_str()
        {
            return Err(error(
                "HSA Stop delivery precondition is stale or mismatched",
            ));
        }
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        let proposed_intent = proposed
            .stop_transaction_map
            .get_mut(&request.intent_id)
            .ok_or_else(|| error("exact HSA Stop intent disappeared"))?;
        proposed_intent.updated_at = request.accepted_at.clone();
        proposed_intent.state = HostSessionStopIntentStateV1::DeliveryAccepted {
            acceptance_id: request.acceptance_id.clone(),
            accepted_by_participant_id: request.authoritative_participant_id.clone(),
            accepted_at: request.accepted_at.clone(),
        };
        let accepted = proposed_intent.clone();
        super::facade::validate_v3_schema_with_external_exact_authority_history(&proposed)
            .map_err(stop_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(stop_error)?;
        Ok(HostSessionStopDeliveryOutcomeV1::Accepted(accepted))
    }

    pub(crate) fn complete_stop(
        &self,
        request: &CompleteHostSessionStopRequestV1,
    ) -> Result<HostSessionStopCompletionOutcomeV1, HostSessionStopProtocolError> {
        validate_completion_request(request)?;
        let current = self.read_a12b_root().map_err(stop_error)?;
        let intent = current
            .stop_transaction_map
            .get(&request.intent_id)
            .ok_or_else(|| error("exact HSA Stop intent was not found"))?;
        validate_completion_identity(intent, request)?;
        if matches!(intent.state, HostSessionStopIntentStateV1::Completed { .. }) {
            let result = stop_result(intent)?;
            if result.result_id != request.result_id
                || result.delivery_acceptance_id != request.delivery_acceptance_id
                || result.completed_at != request.completed_at
            {
                return Err(error(
                    "HSA Stop completion retry conflicts with committed result",
                ));
            }
            return Ok(HostSessionStopCompletionOutcomeV1::Joined(result));
        }
        let active =
            intent.authority_before.lifecycle_posture == HostSessionPostureV1::ActiveAttached;
        let delivery = match &intent.state {
            HostSessionStopIntentStateV1::DeliveryAccepted {
                acceptance_id,
                accepted_by_participant_id,
                accepted_at,
            } if active
                && request.delivery_acceptance_id.as_ref() == Some(acceptance_id)
                && request.caller.kind == HostSessionTransitionCallerKindV1::Repl
                && request.caller.caller_participant_id.as_ref()
                    == Some(accepted_by_participant_id) =>
            {
                (
                    Some(acceptance_id.clone()),
                    Some(accepted_by_participant_id.clone()),
                    Some(accepted_at.clone()),
                )
            }
            HostSessionStopIntentStateV1::Issued
                if !active
                    && request.delivery_acceptance_id.is_none()
                    && request.caller.kind == HostSessionTransitionCallerKindV1::PublicCli
                    && request.caller.caller_participant_id.is_none() =>
            {
                (None, None, None)
            }
            _ => {
                return Err(error(
                    "HSA Stop completion lacks exact authenticated delivery",
                ))
            }
        };
        if exact_authority(&current, &request.orchestration_session_id)?
            != intent.authority_before.as_ref()
            || request.completed_at.as_str() < intent.updated_at.as_str()
        {
            return Err(error(
                "HSA Stop completion authority is stale or mismatched",
            ));
        }
        let mut authority_after = intent.authority_before.as_ref().clone();
        authority_after.authority_revision = next_revision(authority_after.authority_revision)?;
        authority_after.lifecycle_posture = HostSessionPostureV1::Terminal;
        authority_after.updated_at = request.completed_at.clone();
        let authority_after_commitment = authority_commitment(&authority_after)?;
        let result = HostSessionStopResultV1 {
            schema_version: 1,
            result_id: request.result_id.clone(),
            intent_id: intent.intent_id.clone(),
            request_id: intent.request_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            authority_store_id: intent.authority_store_id.clone(),
            orchestration_session_id: intent.orchestration_session_id.clone(),
            authoritative_participant_id: intent.authoritative_participant_id.clone(),
            delivery_acceptance_id: delivery.0.clone(),
            authority_revision_before: intent.authority_before.authority_revision,
            authority_record_commitment_before: intent.authority_record_commitment_before.clone(),
            authority_revision_after: authority_after.authority_revision,
            authority_record_commitment_after: authority_after_commitment.clone(),
            resulting_posture: HostSessionPostureV1::Terminal,
            completed_at: request.completed_at.clone(),
        };
        let result_commitment = commitment(&result)?;
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        proposed.session_namespace_map.insert(
            request.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::Authority(Box::new(authority_after)),
        );
        let proposed_intent = proposed
            .stop_transaction_map
            .get_mut(&request.intent_id)
            .ok_or_else(|| error("exact HSA Stop intent disappeared"))?;
        proposed_intent.updated_at = request.completed_at.clone();
        proposed_intent.state = HostSessionStopIntentStateV1::Completed {
            delivery_acceptance_id: delivery.0,
            delivery_accepted_by_participant_id: delivery.1,
            delivery_accepted_at: delivery.2,
            result_id: request.result_id.clone(),
            result_commitment,
            authority_revision_after: result.authority_revision_after,
            authority_record_commitment_after: authority_after_commitment,
            completed_at: request.completed_at.clone(),
        };
        super::facade::validate_v3_schema_with_external_exact_authority_history(&proposed)
            .map_err(stop_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(stop_error)?;
        Ok(HostSessionStopCompletionOutcomeV1::Completed(result))
    }
}

pub(crate) fn stop_delivery(intent: &HostSessionStopIntentV1) -> HostSessionStopDeliveryV1 {
    HostSessionStopDeliveryV1 {
        schema_version: 1,
        authority_store_id: intent.authority_store_id.clone(),
        bootstrap_home: intent.bootstrap_home.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        intent_id: intent.intent_id.clone(),
        request_id: intent.request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        authority_revision: intent.authority_before.authority_revision,
        authority_record_commitment: intent.authority_record_commitment_before.clone(),
        authoritative_participant_id: intent.authoritative_participant_id.clone(),
    }
}

fn validate_issue_request(
    resolved: &ResolvedCurrentAuthorityV1,
    request: &IssueHostSessionStopRequestV1,
) -> Result<(), HostSessionStopProtocolError> {
    if request.intent_id.is_empty()
        || request.request_id.is_empty()
        || request.orchestration_session_id.is_empty()
        || request.authoritative_participant_id.is_empty()
        || request.caller.kind != HostSessionTransitionCallerKindV1::PublicCli
        || request.caller.caller_participant_id.is_some()
        || request.caller.auto_attach_obligation_id.is_some()
        || request.caller.auto_attach_claim_owner.is_some()
        || request.expected_authority.orchestration_session_id != request.orchestration_session_id
        || request.expected_authority != resolved.observation
        || request.orchestration_session_id != resolved.authority.orchestration_session_id
        || request.authoritative_participant_id != resolved.caller.participant_id
        || request.authoritative_lineage != resolved.authority.authoritative_participant_lineage
        || request.authoritative_lineage.last() != Some(&request.authoritative_participant_id)
        || !resolved.host_attach_contract.capabilities.session_stop
        || matches!(
            resolved.authority.lifecycle_posture,
            HostSessionPostureV1::Terminal | HostSessionPostureV1::Invalid
        )
    {
        return Err(error("invalid or unauthorized HSA Stop issuance"));
    }
    Ok(())
}

fn validate_issue_join(
    existing: &HostSessionStopIntentV1,
    resolved: &ResolvedCurrentAuthorityV1,
    request: &IssueHostSessionStopRequestV1,
) -> Result<(), HostSessionStopProtocolError> {
    let payload = HostSessionStopPayloadHashInputV1 {
        schema_version: 1,
        intent_id: request.intent_id.clone(),
        request_id: request.request_id.clone(),
        authority_store_id: request.expected_authority.authority_store_id.clone(),
        bootstrap_home: request.expected_authority.bootstrap_home.clone(),
        orchestration_session_id: request.orchestration_session_id.clone(),
        shell_trace_session_id: resolved.authority.shell_trace_session_id.clone(),
        caller: request.caller.clone(),
        authority_revision: resolved.authority.authority_revision,
        authority_record_commitment: request
            .expected_authority
            .authority_record_commitment
            .clone(),
        authoritative_lineage_commitment: request
            .expected_authority
            .authoritative_lineage_commitment
            .clone(),
        authoritative_participant_id: request.authoritative_participant_id.clone(),
        authoritative_lineage: request.authoritative_lineage.clone(),
        lifecycle_posture: resolved.authority.lifecycle_posture,
        issued_at: request.issued_at.clone(),
    };
    if existing.request_id != request.request_id
        || existing.authority_before.as_ref() != &resolved.authority
        || existing.payload_commitment != commitment(&payload)?
    {
        return Err(error("HSA Stop retry conflicts with committed intent"));
    }
    Ok(())
}

fn validate_delivery_request(
    request: &AcceptHostSessionStopDeliveryRequestV1,
) -> Result<(), HostSessionStopProtocolError> {
    if request.intent_id.is_empty()
        || request.request_id.is_empty()
        || request.orchestration_session_id.is_empty()
        || request.authoritative_participant_id.is_empty()
        || request.authority_revision == 0
        || request.acceptance_id.is_empty()
    {
        return Err(error("incomplete HSA Stop delivery request"));
    }
    Ok(())
}

fn validate_delivery_identity(
    intent: &HostSessionStopIntentV1,
    request: &AcceptHostSessionStopDeliveryRequestV1,
) -> Result<(), HostSessionStopProtocolError> {
    if intent.request_id != request.request_id
        || intent.payload_commitment != request.payload_commitment
        || intent.orchestration_session_id != request.orchestration_session_id
        || intent.authoritative_participant_id != request.authoritative_participant_id
        || intent.authority_before.authority_revision != request.authority_revision
        || intent.authority_record_commitment_before != request.authority_record_commitment
    {
        return Err(error(
            "HSA Stop delivery identity is substituted or mismatched",
        ));
    }
    Ok(())
}

fn validate_completion_request(
    request: &CompleteHostSessionStopRequestV1,
) -> Result<(), HostSessionStopProtocolError> {
    if request.intent_id.is_empty()
        || request.request_id.is_empty()
        || request.orchestration_session_id.is_empty()
        || request.authoritative_participant_id.is_empty()
        || request.result_id.is_empty()
        || request
            .delivery_acceptance_id
            .as_ref()
            .is_some_and(String::is_empty)
        || request.caller.auto_attach_obligation_id.is_some()
        || request.caller.auto_attach_claim_owner.is_some()
    {
        return Err(error("incomplete HSA Stop completion request"));
    }
    Ok(())
}

fn validate_completion_identity(
    intent: &HostSessionStopIntentV1,
    request: &CompleteHostSessionStopRequestV1,
) -> Result<(), HostSessionStopProtocolError> {
    if intent.request_id != request.request_id
        || intent.payload_commitment != request.payload_commitment
        || intent.orchestration_session_id != request.orchestration_session_id
        || intent.authoritative_participant_id != request.authoritative_participant_id
    {
        return Err(error(
            "HSA Stop completion identity is substituted or mismatched",
        ));
    }
    Ok(())
}

fn stop_payload(
    root: &StateRootV3,
    authority: &DurableSessionAuthorityV1,
    request: &IssueHostSessionStopRequestV1,
) -> HostSessionStopPayloadHashInputV1 {
    HostSessionStopPayloadHashInputV1 {
        schema_version: 1,
        intent_id: request.intent_id.clone(),
        request_id: request.request_id.clone(),
        authority_store_id: root.authority_store_id.clone(),
        bootstrap_home: root.bootstrap_home.clone(),
        orchestration_session_id: request.orchestration_session_id.clone(),
        shell_trace_session_id: authority.shell_trace_session_id.clone(),
        caller: request.caller.clone(),
        authority_revision: authority.authority_revision,
        authority_record_commitment: request
            .expected_authority
            .authority_record_commitment
            .clone(),
        authoritative_lineage_commitment: request
            .expected_authority
            .authoritative_lineage_commitment
            .clone(),
        authoritative_participant_id: request.authoritative_participant_id.clone(),
        authoritative_lineage: request.authoritative_lineage.clone(),
        lifecycle_posture: authority.lifecycle_posture,
        issued_at: request.issued_at.clone(),
    }
}

fn stop_result(
    intent: &HostSessionStopIntentV1,
) -> Result<HostSessionStopResultV1, HostSessionStopProtocolError> {
    let HostSessionStopIntentStateV1::Completed {
        delivery_acceptance_id,
        result_id,
        result_commitment,
        authority_revision_after,
        authority_record_commitment_after,
        completed_at,
        ..
    } = &intent.state
    else {
        return Err(error("HSA Stop has no committed terminal result"));
    };
    let result = HostSessionStopResultV1 {
        schema_version: 1,
        result_id: result_id.clone(),
        intent_id: intent.intent_id.clone(),
        request_id: intent.request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        authority_store_id: intent.authority_store_id.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        authoritative_participant_id: intent.authoritative_participant_id.clone(),
        delivery_acceptance_id: delivery_acceptance_id.clone(),
        authority_revision_before: intent.authority_before.authority_revision,
        authority_record_commitment_before: intent.authority_record_commitment_before.clone(),
        authority_revision_after: *authority_revision_after,
        authority_record_commitment_after: authority_record_commitment_after.clone(),
        resulting_posture: HostSessionPostureV1::Terminal,
        completed_at: completed_at.clone(),
    };
    if commitment(&result)? != *result_commitment {
        return Err(error("committed HSA Stop result does not authenticate"));
    }
    Ok(result)
}

fn exact_authority<'root>(
    root: &'root StateRootV3,
    orchestration_session_id: &str,
) -> Result<&'root DurableSessionAuthorityV1, HostSessionStopProtocolError> {
    match root.session_namespace_map.get(orchestration_session_id) {
        Some(SessionNamespaceRecordV1::Authority(authority)) => Ok(authority.as_ref()),
        _ => Err(error("exact HSA session authority was not found")),
    }
}

fn authority_commitment(
    authority: &DurableSessionAuthorityV1,
) -> Result<AuthorityObjectCommitmentV1, HostSessionStopProtocolError> {
    commitment(&DurableSessionAuthorityHashInputV1 {
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
    })
}

fn lineage_commitment(
    authority: &DurableSessionAuthorityV1,
) -> Result<AuthorityObjectCommitmentV1, HostSessionStopProtocolError> {
    commitment(&AuthoritativeLineageHashInputV1 {
        schema_version: 1,
        orchestration_session_id: authority.orchestration_session_id.clone(),
        participant_ids: authority.authoritative_participant_lineage.clone(),
    })
}

fn commitment<T: super::validation::CanonicalHashInputV1>(
    value: &T,
) -> Result<AuthorityObjectCommitmentV1, HostSessionStopProtocolError> {
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(value).map_err(stop_error)?,
    })
}

fn next_revision(value: u64) -> Result<u64, HostSessionStopProtocolError> {
    value
        .checked_add(1)
        .ok_or_else(|| error("HSA Stop revision overflows"))
}

fn stop_error(value: impl fmt::Display) -> HostSessionStopProtocolError {
    error(&value.to_string())
}

fn error(message: &str) -> HostSessionStopProtocolError {
    HostSessionStopProtocolError(message.into())
}
