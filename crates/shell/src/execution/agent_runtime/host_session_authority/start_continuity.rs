//! Durable registration and settlement for the real inaugural Start exchange.

use std::fmt;

use chrono::SecondsFormat;
use substrate_common::OpaqueAuthorityCommitmentV1;

use super::canonical_json;
use super::facade::HostSessionAuthority;
use super::hash::{canonical_object_bytes, canonical_sha256, CanonicalObjectHashInputV1};
use super::schema::{
    AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1,
    DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1,
    HostPostTurnProtocolActorV1, HostSessionPostureV1, HostSessionTransitionModeV1,
    StartContinuationHandleHashInputV2, StartContinuationHandleStateV2, TimestampV1,
};
use super::store;
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, HostSessionTransitionIntentStateV2,
    SessionNamespaceRecordV1, StartTransactionRecordV1, StartTransactionStateV1, StateRootV2,
    StateRootV3,
};

pub(crate) use super::schema::StartTurnCompletionKindV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EstablishStartContinuationRequestV1 {
    pub(crate) start_transaction_id: String,
    pub(crate) request_key_sha256: String,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) application_result_ref: AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) expected_authority_revision: u64,
    pub(crate) authoritative_participant_id: String,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) exchange_id: String,
    pub(crate) exchange_sequence: u64,
    pub(crate) provider_event_kind: String,
    pub(crate) exchange_evidence_sha256: String,
    pub(crate) internal_uaa_session_id: String,
    pub(crate) observed_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SettleStartTurnRequestV1 {
    pub(crate) start_transaction_id: String,
    pub(crate) request_key_sha256: String,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) application_result_ref: AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) registered_resume_handle_ref: AuthorityObjectRefV1,
    pub(crate) expected_authority_revision: u64,
    pub(crate) protocol_actor: HostPostTurnProtocolActorV1,
    pub(crate) event_id: String,
    pub(crate) event_sequence: u64,
    pub(crate) provider_event_kind: String,
    pub(crate) thread_id: String,
    pub(crate) turn_id: String,
    pub(crate) completion_evidence_sha256: String,
    pub(crate) kind: StartTurnCompletionKindV1,
    pub(crate) obligation_ledger_read:
        Option<super::super::obligation_ledger::ObligationLedgerSnapshotReadV1>,
    pub(crate) completed_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StartContinuationRegistrationV1 {
    pub(crate) resume_handle_ref: AuthorityObjectRefV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) internal_uaa_session_id: String,
    pub(crate) registered_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StartContinuationOutcomeV1 {
    Applied(StartContinuationRegistrationV1),
    Joined(StartContinuationRegistrationV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StartTurnSettlementV1 {
    pub(crate) continuation_resume_handle_ref: AuthorityObjectRefV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) resulting_posture: HostSessionPostureV1,
    pub(crate) completed_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommittedStartPublicResultV1 {
    pub(crate) completion_kind: StartTurnCompletionKindV1,
    pub(crate) resulting_posture: HostSessionPostureV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StartTurnSettlementOutcomeV1 {
    Applied(StartTurnSettlementV1),
    Joined(StartTurnSettlementV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StartTransactionBeginOutcomeV1 {
    Applied(StartTransactionRecordV1),
    Joined(StartTransactionRecordV1),
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EstablishStartContinuationCrashPointV1 {
    HandlePublished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SettleStartTurnCrashPointV1 {
    HandlePublished,
    RootCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StartContinuityProtocolError(String);

impl fmt::Display for StartContinuityProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for StartContinuityProtocolError {}

impl HostSessionAuthority {
    pub(crate) fn begin_start_transaction(
        &self,
        record: &StartTransactionRecordV1,
    ) -> Result<StartTransactionBeginOutcomeV1, StartContinuityProtocolError> {
        let current = self.read_a12b_root().map_err(start_error)?;
        if let Some(existing) = current.start_transaction_map.get(&record.transaction_id) {
            return if existing == record {
                Ok(StartTransactionBeginOutcomeV1::Joined(existing.clone()))
            } else {
                Err(error("conflicting Start transaction already exists"))
            };
        }
        if current.start_transaction_map.values().any(|existing| {
            existing.request_key_sha256 == record.request_key_sha256
                && !matches!(
                    existing.state,
                    StartTransactionStateV1::PublicResponseDelivered { .. }
                )
        }) {
            return Err(error(
                "another unfinished Start transaction has the same request key",
            ));
        }
        if !matches!(record.state, StartTransactionStateV1::PromptNotSubmitted) {
            return Err(error(
                "new Start transaction must begin before prompt submission",
            ));
        }
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        proposed
            .start_transaction_map
            .insert(record.transaction_id.clone(), record.clone());
        proposed.validate().map_err(start_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(start_error)?;
        Ok(StartTransactionBeginOutcomeV1::Applied(record.clone()))
    }

    pub(crate) fn retryable_start_transaction(
        &self,
        request_key_sha256: &str,
    ) -> Result<Option<StartTransactionRecordV1>, StartContinuityProtocolError> {
        if let Ok(root) = self.read_root() {
            return retryable_start_transaction_from_v1(&root);
        }
        let current = self.read_a12b_root().map_err(start_error)?;
        let mut matches = current.start_transaction_map.values().filter(|record| {
            record.request_key_sha256 == request_key_sha256
                && !matches!(
                    record.state,
                    StartTransactionStateV1::PublicResponseDelivered { .. }
                )
        });
        let result = matches.next().cloned();
        if matches.next().is_some() {
            return Err(error(
                "multiple unfinished Start transactions share a request key",
            ));
        }
        Ok(result)
    }

    pub(crate) fn mark_start_prompt_submission_no_replay_barrier(
        &self,
        transaction_id: &str,
        request_key_sha256: &str,
        barrier_committed_at: TimestampV1,
    ) -> Result<StartTransactionRecordV1, StartContinuityProtocolError> {
        let current = self.read_a12b_root().map_err(start_error)?;
        let record = current
            .start_transaction_map
            .get(transaction_id)
            .ok_or_else(|| error("Start transaction was not found"))?;
        if record.request_key_sha256 != request_key_sha256 {
            return Err(error("Start transaction request key does not authenticate"));
        }
        match &record.state {
            StartTransactionStateV1::PromptNotSubmitted => {}
            StartTransactionStateV1::PromptSubmissionNoReplayBarrier {
                barrier_committed_at: existing,
            } if existing == &barrier_committed_at => return Ok(record.clone()),
            _ => {
                return Err(error(
                    "Start prompt submission barrier cannot be entered from the committed transaction phase",
                ))
            }
        }
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        let next = proposed
            .start_transaction_map
            .get_mut(transaction_id)
            .ok_or_else(|| error("Start transaction disappeared"))?;
        next.updated_at = barrier_committed_at.clone();
        next.state = StartTransactionStateV1::PromptSubmissionNoReplayBarrier {
            barrier_committed_at,
        };
        proposed.validate().map_err(start_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(start_error)?;
        Ok(proposed.start_transaction_map[transaction_id].clone())
    }

    pub(crate) fn mark_start_submission_outcome_indeterminate(
        &self,
        transaction_id: &str,
        request_key_sha256: &str,
        declared_at: TimestampV1,
    ) -> Result<StartTransactionRecordV1, StartContinuityProtocolError> {
        let current = self.read_a12b_root().map_err(start_error)?;
        let record = current
            .start_transaction_map
            .get(transaction_id)
            .ok_or_else(|| error("Start transaction was not found"))?;
        if record.request_key_sha256 != request_key_sha256 {
            return Err(error("Start transaction request key does not authenticate"));
        }
        let barrier_committed_at = match &record.state {
            StartTransactionStateV1::PromptSubmissionNoReplayBarrier {
                barrier_committed_at,
            } => barrier_committed_at.clone(),
            StartTransactionStateV1::PromptSubmissionIndeterminate {
                declared_at: existing,
                ..
            } if existing == &declared_at => return Ok(record.clone()),
            _ => {
                return Err(error(
                    "indeterminate Start submission cannot be declared from the committed transaction phase",
                ))
            }
        };
        if declared_at.as_str() < barrier_committed_at.as_str() {
            return Err(error(
                "indeterminate Start submission declaration precedes its no-replay barrier",
            ));
        }
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        let next = proposed
            .start_transaction_map
            .get_mut(transaction_id)
            .ok_or_else(|| error("Start transaction disappeared"))?;
        next.updated_at = declared_at.clone();
        next.state = StartTransactionStateV1::PromptSubmissionIndeterminate {
            barrier_committed_at,
            declared_at,
        };
        proposed.validate().map_err(start_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(start_error)?;
        Ok(proposed.start_transaction_map[transaction_id].clone())
    }

    pub(crate) fn mark_start_response_delivered(
        &self,
        transaction_id: &str,
        request_key_sha256: &str,
        delivered_at: TimestampV1,
    ) -> Result<StartTransactionRecordV1, StartContinuityProtocolError> {
        let current = self.read_a12b_root().map_err(start_error)?;
        let record = current
            .start_transaction_map
            .get(transaction_id)
            .ok_or_else(|| error("Start transaction was not found"))?;
        if record.request_key_sha256 != request_key_sha256 {
            return Err(error("Start transaction request key does not authenticate"));
        }
        let StartTransactionStateV1::TurnSettledAwaitingResponse {
            settlement_ref,
            authority_revision_after,
            resulting_posture,
            completed_at,
        } = &record.state
        else {
            if matches!(
                record.state,
                StartTransactionStateV1::PublicResponseDelivered { .. }
            ) {
                return Ok(record.clone());
            }
            return Err(error(
                "public Start response cannot be delivered before settlement",
            ));
        };
        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        let next = proposed
            .start_transaction_map
            .get_mut(transaction_id)
            .ok_or_else(|| error("Start transaction disappeared"))?;
        next.updated_at = delivered_at.clone();
        next.state = StartTransactionStateV1::PublicResponseDelivered {
            settlement_ref: settlement_ref.clone(),
            authority_revision_after: *authority_revision_after,
            resulting_posture: *resulting_posture,
            completed_at: completed_at.clone(),
            delivered_at,
        };
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(start_error)?;
        Ok(proposed.start_transaction_map[transaction_id].clone())
    }

    pub(crate) fn committed_start_public_result(
        &self,
        expected_transaction: &StartTransactionRecordV1,
    ) -> Result<CommittedStartPublicResultV1, StartContinuityProtocolError> {
        let root = self.read_a12b_root().map_err(start_error)?;
        let transaction = root
            .start_transaction_map
            .get(&expected_transaction.transaction_id)
            .ok_or_else(|| error("committed Start transaction was not found"))?;
        if transaction != expected_transaction
            || transaction.authority_store_id != root.authority_store_id
        {
            return Err(error(
                "committed Start transaction does not exact-join authority storage",
            ));
        }
        let (settlement_ref, authority_revision_after, resulting_posture, completed_at) =
            match &transaction.state {
                StartTransactionStateV1::TurnSettledAwaitingResponse {
                    settlement_ref,
                    authority_revision_after,
                    resulting_posture,
                    completed_at,
                }
                | StartTransactionStateV1::PublicResponseDelivered {
                    settlement_ref,
                    authority_revision_after,
                    resulting_posture,
                    completed_at,
                    ..
                } => (
                    settlement_ref,
                    *authority_revision_after,
                    *resulting_posture,
                    completed_at,
                ),
                _ => return Err(error("Start transaction has no committed public result")),
            };
        let session = authority_record(&root, &transaction.orchestration_session_id)?;
        if !session.internal_resume_handle_refs.contains(settlement_ref) {
            return Err(error(
                "committed Start settlement is not retained by session authority",
            ));
        }
        let settled = read_start_handle(self, &root, settlement_ref)?;
        if settled.authority_store_id != transaction.authority_store_id
            || settled.orchestration_session_id != transaction.orchestration_session_id
            || settled.participant_id != transaction.authoritative_participant_id
            || settled.backend_id != transaction.backend_id
            || settled.protocol != transaction.protocol
            || settled.start_intent_id != transaction.start_intent_id
            || settled.start_issuer_request_id != transaction.start_issuer_request_id
            || settled.start_payload_commitment != transaction.start_payload_commitment
            || settled.start_application_result_ref != transaction.start_application_result_ref
            || settled.start_run_id != transaction.start_run_id
            || settled.authority_revision_after != authority_revision_after
        {
            return Err(error(
                "committed Start settlement identity does not authenticate",
            ));
        }
        let StartContinuationHandleStateV2::Settled {
            registered_resume_handle_ref,
            completion_kind,
            obligation_snapshot,
            completed_at: settled_at,
            ..
        } = &settled.state
        else {
            return Err(error("committed Start settlement reference is not settled"));
        };
        if settled_at != completed_at
            || !session
                .internal_resume_handle_refs
                .contains(registered_resume_handle_ref.as_ref())
        {
            return Err(error(
                "committed Start settlement ancestry does not authenticate",
            ));
        }
        let registered = read_start_handle(self, &root, registered_resume_handle_ref)?;
        if registered.authority_store_id != transaction.authority_store_id
            || registered.orchestration_session_id != transaction.orchestration_session_id
            || registered.participant_id != transaction.authoritative_participant_id
            || registered.backend_id != transaction.backend_id
            || registered.protocol != transaction.protocol
            || registered.start_intent_id != transaction.start_intent_id
            || registered.start_issuer_request_id != transaction.start_issuer_request_id
            || registered.start_payload_commitment != transaction.start_payload_commitment
            || registered.start_application_result_ref != transaction.start_application_result_ref
            || registered.start_run_id != transaction.start_run_id
            || registered.authority_revision_before != transaction.start_authority_revision
            || registered.authority_revision_after > settled.authority_revision_before
            || !matches!(
                registered.state,
                StartContinuationHandleStateV2::Registered { .. }
            )
        {
            return Err(error(
                "committed Start registration ancestry does not authenticate",
            ));
        }
        let registered_authority = self
            .resolve_exact_at_revision(
                &transaction.orchestration_session_id,
                registered.authority_revision_after,
            )
            .map_err(|_| error("committed Start registration revision is not authenticated"))?;
        let settlement_predecessor = self
            .resolve_exact_at_revision(
                &transaction.orchestration_session_id,
                settled.authority_revision_before,
            )
            .map_err(|_| error("committed Start settlement predecessor is not authenticated"))?;
        if registered_authority.root_revision != root.root_revision
            || settlement_predecessor.root_revision != root.root_revision
            || registered_authority
                .authority
                .internal_resume_handle_refs
                .last()
                != Some(registered_resume_handle_ref.as_ref())
            || !settlement_predecessor
                .authority
                .internal_resume_handle_refs
                .contains(registered_resume_handle_ref.as_ref())
        {
            return Err(error(
                "committed Start mixed authority ancestry does not authenticate",
            ));
        }
        let authenticated_posture =
            committed_settlement_posture(completion_kind, obligation_snapshot)?;
        if authenticated_posture != resulting_posture {
            return Err(error(
                "committed Start public result conflicts with settlement posture",
            ));
        }
        Ok(CommittedStartPublicResultV1 {
            completion_kind: completion_kind.clone(),
            resulting_posture,
        })
    }

    pub(crate) fn establish_start_continuation(
        &self,
        request: &EstablishStartContinuationRequestV1,
    ) -> Result<StartContinuationOutcomeV1, StartContinuityProtocolError> {
        self.establish_start_continuation_impl(request, false)
    }

    #[cfg(test)]
    pub(crate) fn establish_start_continuation_at_with_crash_point(
        &self,
        request: &EstablishStartContinuationRequestV1,
        crash_point: EstablishStartContinuationCrashPointV1,
    ) -> Result<StartContinuationOutcomeV1, StartContinuityProtocolError> {
        self.establish_start_continuation_impl(
            request,
            matches!(
                crash_point,
                EstablishStartContinuationCrashPointV1::HandlePublished
            ),
        )
    }

    fn establish_start_continuation_impl(
        &self,
        request: &EstablishStartContinuationRequestV1,
        crash_after_publication: bool,
    ) -> Result<StartContinuationOutcomeV1, StartContinuityProtocolError> {
        validate_registration_request(request)?;
        let current = self.read_a12b_root().map_err(start_error)?;
        let (intent, authority) = exact_applied_start(&current, request)?;
        let resolved = self
            .resolve_current_exact(&request.orchestration_session_id, None)
            .map_err(start_error)?;
        if resolved.observation.authority_store_id != request.authority_store_id
            || resolved.caller.participant_id != request.authoritative_participant_id
            || resolved.caller.descriptor.backend_id != request.backend_id
            || resolved.caller.descriptor.protocol != request.protocol
        {
            return Err(error(
                "Start continuation backend authority does not authenticate",
            ));
        }
        if let Some(joined) = find_registered_handle(self, &current, request)? {
            return Ok(StartContinuationOutcomeV1::Joined(joined));
        }
        if authority.authority_revision != request.expected_authority_revision
            || authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
            || authority.active_authoritative_participant_id.as_deref()
                != Some(&request.authoritative_participant_id)
            || authority
                .internal_resume_handle_refs
                .iter()
                .any(|reference| reference.schema_version == 2)
        {
            return Err(error("Start continuation registration precondition failed"));
        }
        let handle = StartContinuationHandleHashInputV2 {
            schema_version: 2,
            authority_store_id: request.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            participant_id: request.authoritative_participant_id.clone(),
            backend_id: request.backend_id.clone(),
            protocol: request.protocol.clone(),
            internal_uaa_session_id: request.internal_uaa_session_id.clone(),
            start_intent_id: request.intent_id.clone(),
            start_issuer_request_id: request.issuer_request_id.clone(),
            start_payload_commitment: request.payload_commitment.clone(),
            start_application_result_ref: request.application_result_ref.clone(),
            start_run_id: request.run_id.clone(),
            authority_revision_before: request.expected_authority_revision,
            authority_record_commitment_before: authority_commitment(authority)?,
            authority_revision_after: next_revision(request.expected_authority_revision)?,
            state: StartContinuationHandleStateV2::Registered {
                exchange_id: request.exchange_id.clone(),
                exchange_sequence: request.exchange_sequence,
                provider_event_kind: request.provider_event_kind.clone(),
                evidence_sha256: request.exchange_evidence_sha256.clone(),
                observed_at: request.observed_at.clone(),
            },
        };
        let (reference, bytes) = continuation_object(&handle)?;
        store::prepare_typed_object_v3_opened(
            self.trusted_root(),
            current.root_revision,
            &reference,
            &bytes,
            None,
        )
        .map_err(start_error)?;
        if crash_after_publication {
            return Err(error("injected crash after Start handle publication"));
        }

        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        let proposed_authority = authority_mut(&mut proposed, &request.orchestration_session_id)?;
        proposed_authority.authority_revision = handle.authority_revision_after;
        proposed_authority
            .internal_resume_handle_refs
            .push(reference.clone());
        proposed_authority.updated_at = request.observed_at.clone();
        let transaction = proposed
            .start_transaction_map
            .get_mut(&request.start_transaction_id)
            .ok_or_else(|| error("Start registration transaction was not found"))?;
        validate_transaction_for_registration(transaction, request)?;
        transaction.updated_at = request.observed_at.clone();
        transaction.state = StartTransactionStateV1::ContinuationRegistered {
            registration_ref: reference.clone(),
            authority_revision_after: handle.authority_revision_after,
            registered_at: request.observed_at.clone(),
        };
        insert_present(&mut proposed, &reference, bytes.len())?;
        proposed.validate().map_err(start_error)?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(start_error)?;
        let _ = intent;
        Ok(StartContinuationOutcomeV1::Applied(
            StartContinuationRegistrationV1 {
                resume_handle_ref: reference,
                authority_revision_after: handle.authority_revision_after,
                internal_uaa_session_id: request.internal_uaa_session_id.clone(),
                registered_at: request.observed_at.clone(),
            },
        ))
    }

    pub(crate) fn settle_start_turn(
        &self,
        request: &SettleStartTurnRequestV1,
    ) -> Result<StartTurnSettlementOutcomeV1, StartContinuityProtocolError> {
        self.settle_start_turn_impl(request, None)
    }

    #[cfg(test)]
    pub(crate) fn settle_start_turn_at_with_crash_point(
        &self,
        request: &SettleStartTurnRequestV1,
        crash_point: SettleStartTurnCrashPointV1,
    ) -> Result<StartTurnSettlementOutcomeV1, StartContinuityProtocolError> {
        self.settle_start_turn_impl(request, Some(crash_point))
    }

    fn settle_start_turn_impl(
        &self,
        request: &SettleStartTurnRequestV1,
        #[cfg_attr(not(test), allow(unused_variables))] crash_point: Option<
            SettleStartTurnCrashPointV1,
        >,
    ) -> Result<StartTurnSettlementOutcomeV1, StartContinuityProtocolError> {
        validate_settlement_request(request)?;
        let current = self.read_a12b_root().map_err(start_error)?;
        let (intent, authority) = exact_applied_start_for_settlement(&current, request)?;
        if let Some(joined) = find_settled_handle(self, &current, request)? {
            return Ok(StartTurnSettlementOutcomeV1::Joined(joined));
        }
        if authority.authority_revision != request.expected_authority_revision
            || authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
            || authority.internal_resume_handle_refs.last()
                != Some(&request.registered_resume_handle_ref)
        {
            return Err(error("Start turn settlement precondition failed"));
        }
        let registered = read_start_handle(self, &current, &request.registered_resume_handle_ref)?;
        validate_registered_for_settlement(&registered, request)?;
        let registered_authority = self
            .resolve_exact_at_revision(
                &request.orchestration_session_id,
                registered.authority_revision_after,
            )
            .map_err(|_| error("Start settlement registration ancestry does not authenticate"))?;
        if registered_authority.root_revision != current.root_revision
            || registered_authority
                .authority
                .internal_resume_handle_refs
                .last()
                != Some(&request.registered_resume_handle_ref)
            || registered_authority.authority.lifecycle_posture
                != HostSessionPostureV1::ActiveAttached
        {
            return Err(error(
                "Start settlement registration ancestry is stale or mismatched",
            ));
        }
        validate_protocol_actor(intent, request)?;
        let resulting_posture = settlement_posture(request, intent)?;
        let handle = StartContinuationHandleHashInputV2 {
            schema_version: 2,
            authority_store_id: request.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            participant_id: request.authoritative_participant_id.clone(),
            backend_id: request.backend_id.clone(),
            protocol: request.protocol.clone(),
            internal_uaa_session_id: registered.internal_uaa_session_id.clone(),
            start_intent_id: request.intent_id.clone(),
            start_issuer_request_id: request.issuer_request_id.clone(),
            start_payload_commitment: request.payload_commitment.clone(),
            start_application_result_ref: request.application_result_ref.clone(),
            start_run_id: request.run_id.clone(),
            authority_revision_before: request.expected_authority_revision,
            authority_record_commitment_before: authority_commitment(authority)?,
            authority_revision_after: next_revision(request.expected_authority_revision)?,
            state: StartContinuationHandleStateV2::Settled {
                registered_resume_handle_ref: Box::new(
                    request.registered_resume_handle_ref.clone(),
                ),
                protocol_actor: request.protocol_actor.clone(),
                event_id: request.event_id.clone(),
                event_sequence: request.event_sequence,
                provider_event_kind: request.provider_event_kind.clone(),
                thread_id: request.thread_id.clone(),
                turn_id: request.turn_id.clone(),
                evidence_sha256: request.completion_evidence_sha256.clone(),
                completion_kind: request.kind.clone(),
                obligation_snapshot: authenticated_complete_start_snapshot(request, intent)?
                    .map(Box::new),
                completed_at: request.completed_at.clone(),
            },
        };
        let (reference, bytes) = continuation_object(&handle)?;
        store::prepare_typed_object_v3_opened(
            self.trusted_root(),
            current.root_revision,
            &reference,
            &bytes,
            None,
        )
        .map_err(start_error)?;
        #[cfg(test)]
        if matches!(
            crash_point,
            Some(SettleStartTurnCrashPointV1::HandlePublished)
        ) {
            return Err(error("injected crash after Start settlement publication"));
        }

        let mut proposed = current.clone();
        proposed.root_revision = next_revision(proposed.root_revision)?;
        let proposed_authority = authority_mut(&mut proposed, &request.orchestration_session_id)?;
        proposed_authority.authority_revision = handle.authority_revision_after;
        proposed_authority.lifecycle_posture = resulting_posture;
        proposed_authority
            .internal_resume_handle_refs
            .push(reference.clone());
        proposed_authority.updated_at = request.completed_at.clone();
        let transaction = proposed
            .start_transaction_map
            .get_mut(&request.start_transaction_id)
            .ok_or_else(|| error("Start settlement transaction was not found"))?;
        validate_transaction_for_settlement(transaction, request, &registered)?;
        transaction.updated_at = request.completed_at.clone();
        transaction.state = StartTransactionStateV1::TurnSettledAwaitingResponse {
            settlement_ref: reference.clone(),
            authority_revision_after: handle.authority_revision_after,
            resulting_posture,
            completed_at: request.completed_at.clone(),
        };
        insert_present(&mut proposed, &reference, bytes.len())?;
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || Ok(()),
        )
        .map_err(start_error)?;
        let receipt = StartTurnSettlementV1 {
            continuation_resume_handle_ref: reference,
            authority_revision_after: handle.authority_revision_after,
            resulting_posture,
            completed_at: request.completed_at.clone(),
        };
        #[cfg(test)]
        if matches!(
            crash_point,
            Some(SettleStartTurnCrashPointV1::RootCommitted)
        ) {
            return Err(error("injected crash after Start settlement commit"));
        }
        Ok(StartTurnSettlementOutcomeV1::Applied(receipt))
    }
}

pub(crate) fn retryable_start_transaction_from_v1(
    root: &super::store_schema::StateRootV1,
) -> Result<Option<StartTransactionRecordV1>, StartContinuityProtocolError> {
    StateRootV2::try_from_greenfield_v1(root).map_err(start_error)?;
    Ok(None)
}

fn validate_registration_request(
    request: &EstablishStartContinuationRequestV1,
) -> Result<(), StartContinuityProtocolError> {
    if [
        request.authority_store_id.as_str(),
        request.start_transaction_id.as_str(),
        request.request_key_sha256.as_str(),
        request.orchestration_session_id.as_str(),
        request.intent_id.as_str(),
        request.issuer_request_id.as_str(),
        request.run_id.as_str(),
        request.authoritative_participant_id.as_str(),
        request.backend_id.as_str(),
        request.protocol.as_str(),
        request.exchange_id.as_str(),
        request.provider_event_kind.as_str(),
        request.exchange_evidence_sha256.as_str(),
        request.internal_uaa_session_id.as_str(),
    ]
    .iter()
    .any(|value| value.is_empty())
        || request.expected_authority_revision == 0
        || request.exchange_sequence == 0
        || request.application_result_ref.object_kind != AuthorityObjectKindV1::ApplicationResult
        || !lower_hex(&request.request_key_sha256)
        || !lower_hex(&request.exchange_evidence_sha256)
    {
        return Err(error("Start continuation request is incomplete"));
    }
    Ok(())
}

fn validate_settlement_request(
    request: &SettleStartTurnRequestV1,
) -> Result<(), StartContinuityProtocolError> {
    if [
        request.authority_store_id.as_str(),
        request.start_transaction_id.as_str(),
        request.request_key_sha256.as_str(),
        request.orchestration_session_id.as_str(),
        request.intent_id.as_str(),
        request.issuer_request_id.as_str(),
        request.run_id.as_str(),
        request.authoritative_participant_id.as_str(),
        request.backend_id.as_str(),
        request.protocol.as_str(),
        request.event_id.as_str(),
        request.provider_event_kind.as_str(),
        request.thread_id.as_str(),
        request.turn_id.as_str(),
        request.completion_evidence_sha256.as_str(),
    ]
    .iter()
    .any(|value| value.is_empty())
        || request.expected_authority_revision == 0
        || request.event_sequence == 0
        || request.registered_resume_handle_ref.object_kind != AuthorityObjectKindV1::ResumeHandle
        || request.registered_resume_handle_ref.schema_version != 2
        || !lower_hex(&request.request_key_sha256)
        || !lower_hex(&request.completion_evidence_sha256)
    {
        return Err(error("Start settlement request is incomplete"));
    }
    Ok(())
}

fn exact_applied_start<'a>(
    root: &'a StateRootV3,
    request: &EstablishStartContinuationRequestV1,
) -> Result<
    (
        &'a super::store_schema::HostSessionTransitionIntentV2,
        &'a super::store_schema::DurableSessionAuthorityV1,
    ),
    StartContinuityProtocolError,
> {
    if root.authority_store_id != request.authority_store_id {
        return Err(error("Start continuation authority store mismatch"));
    }
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("Start continuation intent was not found"))?;
    let authority = root
        .session_namespace_map
        .get(&request.orchestration_session_id)
        .and_then(|record| match record {
            SessionNamespaceRecordV1::Authority(authority) => Some(authority.as_ref()),
            _ => None,
        })
        .ok_or_else(|| error("Start continuation authority was not found"))?;
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref,
        authority_revision_after,
        active_authoritative_participant_id,
        ..
    } = &intent.state
    else {
        return Err(error("Start continuation requires an applied Start intent"));
    };
    let attach = root
        .object_index
        .get(&intent.host_attach_contract_ref.ref_id)
        .ok_or_else(|| error("Start continuation attach contract was not found"))?;
    if intent.mode != HostSessionTransitionModeV1::Start
        || intent.orchestration_session_id != request.orchestration_session_id
        || intent.issuer_request_id != request.issuer_request_id
        || intent.payload_commitment != request.payload_commitment
        || application_result_ref != &request.application_result_ref
        || intent.run_id != request.run_id
        || intent.target_authoritative_participant_id != request.authoritative_participant_id
        || active_authoritative_participant_id != &request.authoritative_participant_id
        || *authority_revision_after != 1
        || attach.object_kind != AuthorityObjectKindV1::HostAttachContract
        || authority.origin
            != (DurableSessionAuthorityOriginV1::StartIntent {
                intent_id: request.intent_id.clone(),
                issuer_request_id: request.issuer_request_id.clone(),
                payload_commitment: request.payload_commitment.clone(),
            })
    {
        return Err(error("Start continuation evidence does not authenticate"));
    }
    Ok((intent, authority))
}

fn exact_applied_start_for_settlement<'a>(
    root: &'a StateRootV3,
    request: &SettleStartTurnRequestV1,
) -> Result<
    (
        &'a super::store_schema::HostSessionTransitionIntentV2,
        &'a super::store_schema::DurableSessionAuthorityV1,
    ),
    StartContinuityProtocolError,
> {
    let registration = EstablishStartContinuationRequestV1 {
        start_transaction_id: request.start_transaction_id.clone(),
        request_key_sha256: request.request_key_sha256.clone(),
        authority_store_id: request.authority_store_id.clone(),
        orchestration_session_id: request.orchestration_session_id.clone(),
        intent_id: request.intent_id.clone(),
        issuer_request_id: request.issuer_request_id.clone(),
        payload_commitment: request.payload_commitment.clone(),
        application_result_ref: request.application_result_ref.clone(),
        run_id: request.run_id.clone(),
        expected_authority_revision: 1,
        authoritative_participant_id: request.authoritative_participant_id.clone(),
        backend_id: request.backend_id.clone(),
        protocol: request.protocol.clone(),
        exchange_id: "validated-from-registered-handle".into(),
        exchange_sequence: 1,
        provider_event_kind: "validated-from-registered-handle".into(),
        exchange_evidence_sha256: "0".repeat(64),
        internal_uaa_session_id: "validated-from-registered-handle".into(),
        observed_at: request.completed_at.clone(),
    };
    exact_applied_start(root, &registration)
}

fn validate_registered_for_settlement(
    registered: &StartContinuationHandleHashInputV2,
    request: &SettleStartTurnRequestV1,
) -> Result<(), StartContinuityProtocolError> {
    let StartContinuationHandleStateV2::Registered {
        exchange_sequence, ..
    } = registered.state
    else {
        return Err(error(
            "Start settlement does not reference registration evidence",
        ));
    };
    if registered.authority_store_id != request.authority_store_id
        || registered.orchestration_session_id != request.orchestration_session_id
        || registered.participant_id != request.authoritative_participant_id
        || registered.backend_id != request.backend_id
        || registered.protocol != request.protocol
        || registered.start_intent_id != request.intent_id
        || registered.start_issuer_request_id != request.issuer_request_id
        || registered.start_payload_commitment != request.payload_commitment
        || registered.start_application_result_ref != request.application_result_ref
        || registered.start_run_id != request.run_id
        || registered.authority_revision_after > request.expected_authority_revision
        || request.event_sequence <= exchange_sequence
    {
        return Err(error(
            "Start settlement registration evidence does not authenticate",
        ));
    }
    Ok(())
}

fn validate_transaction_for_registration(
    transaction: &StartTransactionRecordV1,
    request: &EstablishStartContinuationRequestV1,
) -> Result<(), StartContinuityProtocolError> {
    if transaction.transaction_id != request.start_transaction_id
        || transaction.request_key_sha256 != request.request_key_sha256
        || transaction.authority_store_id != request.authority_store_id
        || transaction.orchestration_session_id != request.orchestration_session_id
        || transaction.authoritative_participant_id != request.authoritative_participant_id
        || transaction.backend_id != request.backend_id
        || transaction.protocol != request.protocol
        || transaction.start_intent_id != request.intent_id
        || transaction.start_issuer_request_id != request.issuer_request_id
        || transaction.start_payload_commitment != request.payload_commitment
        || transaction.start_application_result_ref != request.application_result_ref
        || transaction.start_run_id != request.run_id
        || transaction.start_authority_revision != request.expected_authority_revision
        || !matches!(
            transaction.state,
            StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
        )
    {
        return Err(error(
            "Start registration does not exact-join its durable transaction",
        ));
    }
    Ok(())
}

fn validate_transaction_for_settlement(
    transaction: &StartTransactionRecordV1,
    request: &SettleStartTurnRequestV1,
    registered: &StartContinuationHandleHashInputV2,
) -> Result<(), StartContinuityProtocolError> {
    let StartTransactionStateV1::ContinuationRegistered {
        registration_ref,
        authority_revision_after,
        ..
    } = &transaction.state
    else {
        return Err(error(
            "Start settlement transaction has no committed registration",
        ));
    };
    if transaction.transaction_id != request.start_transaction_id
        || transaction.request_key_sha256 != request.request_key_sha256
        || transaction.authority_store_id != request.authority_store_id
        || transaction.orchestration_session_id != request.orchestration_session_id
        || transaction.authoritative_participant_id != request.authoritative_participant_id
        || transaction.backend_id != request.backend_id
        || transaction.protocol != request.protocol
        || transaction.start_intent_id != request.intent_id
        || transaction.start_issuer_request_id != request.issuer_request_id
        || transaction.start_payload_commitment != request.payload_commitment
        || transaction.start_application_result_ref != request.application_result_ref
        || transaction.start_run_id != request.run_id
        || registration_ref != &request.registered_resume_handle_ref
        || *authority_revision_after != registered.authority_revision_after
    {
        return Err(error(
            "Start settlement does not exact-join its durable transaction",
        ));
    }
    Ok(())
}

fn validate_protocol_actor(
    intent: &super::store_schema::HostSessionTransitionIntentV2,
    request: &SettleStartTurnRequestV1,
) -> Result<(), StartContinuityProtocolError> {
    let valid = match (&request.protocol_actor, &intent.state) {
        (HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant { participant_id }, _) => {
            participant_id == &request.authoritative_participant_id
        }
        (
            HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
                claim_id,
                claimant_attempt_id,
            },
            HostSessionTransitionIntentStateV2::Applied {
                claim_id: expected_claim,
                claimant_attempt_id: expected_attempt,
                ..
            },
        ) => claim_id == expected_claim && claimant_attempt_id == expected_attempt,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(error(
            "Start settlement protocol actor does not authenticate",
        ))
    }
}

fn settlement_posture(
    request: &SettleStartTurnRequestV1,
    intent: &super::store_schema::HostSessionTransitionIntentV2,
) -> Result<HostSessionPostureV1, StartContinuityProtocolError> {
    match &request.kind {
        StartTurnCompletionKindV1::TerminalClean
        | StartTurnCompletionKindV1::TerminalFailure { .. } => {
            if request.obligation_ledger_read.is_some() {
                return Err(error("terminal Start settlement cannot carry obligations"));
            }
            Ok(HostSessionPostureV1::Terminal)
        }
        StartTurnCompletionKindV1::ResumableClean => {
            let Some(snapshot) = authenticated_complete_start_snapshot(request, intent)? else {
                return Ok(HostSessionPostureV1::ParkedResumable);
            };
            match snapshot.attention_disposition {
                super::schema::ObligationAttentionDispositionV1::HasUnresolvedAttention
                    if !snapshot.unresolved_attention_obligations.is_empty() =>
                {
                    Ok(HostSessionPostureV1::AwaitingAttention)
                }
                super::schema::ObligationAttentionDispositionV1::NoUnresolvedAttention
                    if snapshot.unresolved_attention_obligations.is_empty() =>
                {
                    Ok(HostSessionPostureV1::ParkedResumable)
                }
                _ => Err(error(
                    "Start obligation snapshot disposition is inconsistent",
                )),
            }
        }
    }
}

pub(super) fn committed_settlement_posture(
    completion_kind: &StartTurnCompletionKindV1,
    obligation_snapshot: &Option<Box<super::schema::ObligationSnapshotHashInputV1>>,
) -> Result<HostSessionPostureV1, StartContinuityProtocolError> {
    match completion_kind {
        StartTurnCompletionKindV1::TerminalClean
        | StartTurnCompletionKindV1::TerminalFailure { .. } => {
            if obligation_snapshot.is_some() {
                return Err(error(
                    "committed terminal Start settlement carries obligations",
                ));
            }
            Ok(HostSessionPostureV1::Terminal)
        }
        StartTurnCompletionKindV1::ResumableClean => match obligation_snapshot.as_deref() {
            None => Ok(HostSessionPostureV1::ParkedResumable),
            Some(snapshot) => match snapshot.attention_disposition {
                super::schema::ObligationAttentionDispositionV1::HasUnresolvedAttention
                    if !snapshot.unresolved_attention_obligations.is_empty() =>
                {
                    Ok(HostSessionPostureV1::AwaitingAttention)
                }
                super::schema::ObligationAttentionDispositionV1::NoUnresolvedAttention
                    if snapshot.unresolved_attention_obligations.is_empty() =>
                {
                    Ok(HostSessionPostureV1::ParkedResumable)
                }
                _ => Err(error(
                    "committed Start obligation snapshot disposition is inconsistent",
                )),
            },
        },
    }
}

fn find_registered_handle(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    request: &EstablishStartContinuationRequestV1,
) -> Result<Option<StartContinuationRegistrationV1>, StartContinuityProtocolError> {
    let session = authority_record(root, &request.orchestration_session_id)?;
    for reference in session
        .internal_resume_handle_refs
        .iter()
        .filter(|reference| reference.schema_version == 2)
    {
        let handle = read_start_handle(authority, root, reference)?;
        if handle.start_intent_id != request.intent_id {
            continue;
        }
        if handle.authority_store_id != request.authority_store_id
            || handle.orchestration_session_id != request.orchestration_session_id
            || handle.participant_id != request.authoritative_participant_id
            || handle.backend_id != request.backend_id
            || handle.protocol != request.protocol
            || handle.internal_uaa_session_id != request.internal_uaa_session_id
            || handle.start_issuer_request_id != request.issuer_request_id
            || handle.start_payload_commitment != request.payload_commitment
            || handle.start_application_result_ref != request.application_result_ref
            || handle.start_run_id != request.run_id
            || handle.authority_revision_before != request.expected_authority_revision
        {
            return Err(error("conflicting Start continuation already exists"));
        }
        if let StartContinuationHandleStateV2::Registered {
            exchange_id,
            exchange_sequence,
            provider_event_kind,
            evidence_sha256,
            observed_at,
        } = handle.state
        {
            if exchange_id != request.exchange_id
                || exchange_sequence != request.exchange_sequence
                || provider_event_kind != request.provider_event_kind
                || evidence_sha256 != request.exchange_evidence_sha256
                || observed_at != request.observed_at
            {
                return Err(error("conflicting Start exchange already exists"));
            }
            let registration = StartContinuationRegistrationV1 {
                resume_handle_ref: reference.clone(),
                authority_revision_after: handle.authority_revision_after,
                internal_uaa_session_id: handle.internal_uaa_session_id.clone(),
                registered_at: observed_at,
            };
            let transaction = root
                .start_transaction_map
                .get(&request.start_transaction_id)
                .ok_or_else(|| error("joined Start registration has no durable transaction"))?;
            if transaction.request_key_sha256 != request.request_key_sha256
                || !transaction_authenticates_registration(
                    authority,
                    root,
                    transaction,
                    &registration,
                )?
            {
                return Err(error(
                    "joined Start registration conflicts with its durable transaction",
                ));
            }
            return Ok(Some(registration));
        }
    }
    Ok(None)
}

fn find_settled_handle(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    request: &SettleStartTurnRequestV1,
) -> Result<Option<StartTurnSettlementV1>, StartContinuityProtocolError> {
    let session = authority_record(root, &request.orchestration_session_id)?;
    let intent = root
        .transition_intent_map
        .get(&request.intent_id)
        .ok_or_else(|| error("Start settlement intent was not found"))?;
    let expected_obligation_snapshot = authenticated_complete_start_snapshot(request, intent)?;
    for reference in session
        .internal_resume_handle_refs
        .iter()
        .filter(|reference| reference.schema_version == 2)
    {
        let handle = read_start_handle(authority, root, reference)?;
        let StartContinuationHandleStateV2::Settled {
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
            completed_at,
        } = &handle.state
        else {
            continue;
        };
        if handle.authority_store_id == request.authority_store_id
            && handle.orchestration_session_id == request.orchestration_session_id
            && handle.participant_id == request.authoritative_participant_id
            && handle.backend_id == request.backend_id
            && handle.protocol == request.protocol
            && handle.start_intent_id == request.intent_id
            && handle.start_issuer_request_id == request.issuer_request_id
            && handle.start_payload_commitment == request.payload_commitment
            && handle.start_application_result_ref == request.application_result_ref
            && handle.start_run_id == request.run_id
            && handle.authority_revision_before == request.expected_authority_revision
            && registered_resume_handle_ref.as_ref() == &request.registered_resume_handle_ref
            && protocol_actor == &request.protocol_actor
            && event_id == &request.event_id
            && *event_sequence == request.event_sequence
            && provider_event_kind == &request.provider_event_kind
            && thread_id == &request.thread_id
            && turn_id == &request.turn_id
            && evidence_sha256 == &request.completion_evidence_sha256
            && completion_kind == &request.kind
            && obligation_snapshot.as_deref() == expected_obligation_snapshot.as_ref()
            && completed_at == &request.completed_at
        {
            let settlement = StartTurnSettlementV1 {
                continuation_resume_handle_ref: reference.clone(),
                authority_revision_after: handle.authority_revision_after,
                resulting_posture: settlement_posture(request, intent)?,
                completed_at: completed_at.clone(),
            };
            let transaction = root
                .start_transaction_map
                .get(&request.start_transaction_id)
                .ok_or_else(|| error("joined Start settlement has no durable transaction"))?;
            if transaction.request_key_sha256 != request.request_key_sha256
                || !transaction_authenticates_settlement(transaction, &settlement)
            {
                return Err(error(
                    "joined Start settlement conflicts with its durable transaction",
                ));
            }
            return Ok(Some(settlement));
        }
        if handle.start_intent_id == request.intent_id {
            return Err(error("conflicting Start settlement already exists"));
        }
    }
    Ok(None)
}

fn transaction_authenticates_registration(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    transaction: &StartTransactionRecordV1,
    registration: &StartContinuationRegistrationV1,
) -> Result<bool, StartContinuityProtocolError> {
    let exact_registered = StartTransactionStateV1::ContinuationRegistered {
        registration_ref: registration.resume_handle_ref.clone(),
        authority_revision_after: registration.authority_revision_after,
        registered_at: registration.registered_at.clone(),
    };
    if transaction.state == exact_registered {
        return Ok(true);
    }
    let settlement_ref = match &transaction.state {
        StartTransactionStateV1::TurnSettledAwaitingResponse { settlement_ref, .. }
        | StartTransactionStateV1::PublicResponseDelivered { settlement_ref, .. } => settlement_ref,
        _ => return Ok(false),
    };
    let settled = read_start_handle(authority, root, settlement_ref)?;
    let StartContinuationHandleStateV2::Settled {
        registered_resume_handle_ref,
        ..
    } = &settled.state
    else {
        return Ok(false);
    };
    if registered_resume_handle_ref.as_ref() != &registration.resume_handle_ref {
        return Ok(false);
    }
    let registered_authority = authority
        .resolve_exact_at_revision(
            &transaction.orchestration_session_id,
            registration.authority_revision_after,
        )
        .map_err(|_| error("retry registration ancestry does not authenticate"))?;
    let settlement_predecessor = authority
        .resolve_exact_at_revision(
            &transaction.orchestration_session_id,
            settled.authority_revision_before,
        )
        .map_err(|_| error("retry settlement predecessor does not authenticate"))?;
    if registered_authority.root_revision != root.root_revision
        || settlement_predecessor.root_revision != root.root_revision
        || registered_authority
            .authority
            .internal_resume_handle_refs
            .last()
            != Some(&registration.resume_handle_ref)
        || !settlement_predecessor
            .authority
            .internal_resume_handle_refs
            .contains(&registration.resume_handle_ref)
    {
        return Ok(false);
    }
    authority.committed_start_public_result(transaction)?;
    Ok(true)
}

fn transaction_authenticates_settlement(
    transaction: &StartTransactionRecordV1,
    settlement: &StartTurnSettlementV1,
) -> bool {
    match &transaction.state {
        StartTransactionStateV1::TurnSettledAwaitingResponse {
            settlement_ref,
            authority_revision_after,
            resulting_posture,
            completed_at,
        }
        | StartTransactionStateV1::PublicResponseDelivered {
            settlement_ref,
            authority_revision_after,
            resulting_posture,
            completed_at,
            ..
        } => {
            settlement_ref == &settlement.continuation_resume_handle_ref
                && *authority_revision_after == settlement.authority_revision_after
                && *resulting_posture == settlement.resulting_posture
                && completed_at == &settlement.completed_at
        }
        _ => false,
    }
}

fn authenticated_complete_start_snapshot(
    request: &SettleStartTurnRequestV1,
    intent: &super::store_schema::HostSessionTransitionIntentV2,
) -> Result<Option<super::schema::ObligationSnapshotHashInputV1>, StartContinuityProtocolError> {
    let Some(read) = request.obligation_ledger_read.as_ref() else {
        return Ok(None);
    };
    let super::super::obligation_ledger::ObligationLedgerSnapshotReadV1::Complete { snapshot } =
        read
    else {
        return Err(error(
            "pending canonical obligation evidence cannot settle Start",
        ));
    };
    let snapshot = ledger_snapshot_to_authority(snapshot)?;
    let correlation = &snapshot.host_transition_correlation;
    let expected_payload_commitment = opaque_commitment(&request.payload_commitment);
    let cut = &snapshot.materialization_cut;
    let exact_journal = !snapshot.materialized_journal_events.is_empty()
        && snapshot.materialized_journal_events.iter().all(|event| {
            event.acceptance_record_id == snapshot.acceptance_record_id
                && event.acceptance_record_revision == snapshot.acceptance_record_revision
                && event.stream_id == snapshot.stream_id
                && event.accepted_work_identity == snapshot.accepted_work_identity
                && event.event_sequence <= cut.materialized_through_event_sequence
        })
        && snapshot
            .materialized_journal_events
            .last()
            .is_some_and(|event| {
                event.event_id == cut.terminal_event_id
                    && event.event_sequence == cut.terminal_event_sequence
            });
    if snapshot.authority_store_id != request.authority_store_id
        || snapshot.orchestration_session_id != request.orchestration_session_id
        || snapshot.authoritative_participant_id != request.authoritative_participant_id
        || snapshot.transition_intent_id != request.intent_id
        || snapshot.transition_run_id != request.run_id
        || snapshot.authority_revision_observed != request.expected_authority_revision
        || correlation.authority_store_id != request.authority_store_id
        || correlation.orchestration_session_id != request.orchestration_session_id
        || correlation.authoritative_participant_id != request.authoritative_participant_id
        || correlation.transition_intent_id != request.intent_id
        || correlation.transition_intent_revision_observed != intent.intent_revision
        || correlation.transition_run_id != request.run_id
        || correlation.transition_payload_commitment != expected_payload_commitment
        || correlation.authority_revision_observed != request.expected_authority_revision
        || cut.acceptance_record_id != snapshot.acceptance_record_id
        || cut.acceptance_record_revision != snapshot.acceptance_record_revision
        || cut.stream_id != snapshot.stream_id
        || cut.materialized_through_event_sequence != cut.terminal_event_sequence
        || !exact_journal
    {
        return Err(error(
            "canonical Start obligation ledger result does not authenticate",
        ));
    }
    match snapshot.attention_disposition {
        super::schema::ObligationAttentionDispositionV1::HasUnresolvedAttention
            if snapshot.unresolved_attention_obligations.is_empty() =>
        {
            Err(error(
                "canonical Start obligation ledger disposition is inconsistent",
            ))
        }
        super::schema::ObligationAttentionDispositionV1::NoUnresolvedAttention
            if !snapshot.unresolved_attention_obligations.is_empty() =>
        {
            Err(error(
                "canonical Start obligation ledger disposition is inconsistent",
            ))
        }
        _ => Ok(Some(snapshot)),
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

fn read_start_handle(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    reference: &AuthorityObjectRefV1,
) -> Result<StartContinuationHandleHashInputV2, StartContinuityProtocolError> {
    if reference.object_kind != AuthorityObjectKindV1::ResumeHandle || reference.schema_version != 2
    {
        return Err(error(
            "Start continuation handle reference is not schema version 2",
        ));
    }
    let bytes = store::read_typed_object_v2_or_v3_opened(
        authority.trusted_root(),
        root.root_revision,
        reference,
        None,
    )
    .map_err(start_error)?;
    canonical_json::from_slice(&bytes).map_err(start_error)
}

fn authority_record<'a>(
    root: &'a StateRootV3,
    orchestration_session_id: &str,
) -> Result<&'a super::store_schema::DurableSessionAuthorityV1, StartContinuityProtocolError> {
    root.session_namespace_map
        .get(orchestration_session_id)
        .and_then(|record| match record {
            SessionNamespaceRecordV1::Authority(authority) => Some(authority.as_ref()),
            _ => None,
        })
        .ok_or_else(|| error("Start authority was not found"))
}

fn ledger_snapshot_to_authority(
    snapshot: &super::super::obligation_ledger::ObligationSnapshotHashInputV1,
) -> Result<super::schema::ObligationSnapshotHashInputV1, StartContinuityProtocolError> {
    Ok(super::schema::ObligationSnapshotHashInputV1 {
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
            super::super::obligation_ledger::ObligationAttentionDispositionV1::NoUnresolvedAttention => {
                super::schema::ObligationAttentionDispositionV1::NoUnresolvedAttention
            }
            super::super::obligation_ledger::ObligationAttentionDispositionV1::HasUnresolvedAttention => {
                super::schema::ObligationAttentionDispositionV1::HasUnresolvedAttention
            }
        },
        unresolved_attention_obligations: snapshot
            .unresolved_attention_obligations
            .iter()
            .map(|obligation| super::schema::UnresolvedAttentionObligationSnapshotEntryV1 {
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
        .map_err(start_error)?,
    })
}

fn authority_mut<'a>(
    root: &'a mut StateRootV3,
    orchestration_session_id: &str,
) -> Result<&'a mut super::store_schema::DurableSessionAuthorityV1, StartContinuityProtocolError> {
    root.session_namespace_map
        .get_mut(orchestration_session_id)
        .and_then(|record| match record {
            SessionNamespaceRecordV1::Authority(authority) => Some(authority.as_mut()),
            _ => None,
        })
        .ok_or_else(|| error("Start authority was not found"))
}

fn continuation_object(
    handle: &StartContinuationHandleHashInputV2,
) -> Result<(AuthorityObjectRefV1, Vec<u8>), StartContinuityProtocolError> {
    let bytes = canonical_object_bytes(
        AuthorityObjectKindV1::ResumeHandle,
        CanonicalObjectHashInputV1::StartContinuationHandle(handle),
    )
    .map_err(start_error)?;
    let digest_hex = canonical_sha256(handle).map_err(start_error)?;
    Ok((
        AuthorityObjectRefV1 {
            ref_id: format!("ao_{}", &digest_hex[..32]),
            object_kind: AuthorityObjectKindV1::ResumeHandle,
            schema_version: 2,
            commitment: AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex },
        },
        bytes,
    ))
}

fn authority_commitment(
    authority: &super::store_schema::DurableSessionAuthorityV1,
) -> Result<AuthorityObjectCommitmentV1, StartContinuityProtocolError> {
    let digest_hex = canonical_sha256(&DurableSessionAuthorityHashInputV1 {
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
    .map_err(start_error)?;
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex })
}

fn insert_present(
    root: &mut StateRootV3,
    reference: &AuthorityObjectRefV1,
    byte_length: usize,
) -> Result<(), StartContinuityProtocolError> {
    if root
        .object_index
        .insert(
            reference.ref_id.clone(),
            AuthorityObjectIndexEntryV1 {
                schema_version: 1,
                ref_id: reference.ref_id.clone(),
                object_kind: reference.object_kind,
                object_schema_version: reference.schema_version,
                byte_length: byte_length as u64,
                storage_state: AuthorityObjectStorageStateV1::Present,
            },
        )
        .is_some()
    {
        return Err(error("Start continuation object index collision"));
    }
    Ok(())
}

fn next_revision(revision: u64) -> Result<u64, StartContinuityProtocolError> {
    revision
        .checked_add(1)
        .ok_or_else(|| error("Start continuation revision overflow"))
}

fn lower_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn error(message: impl Into<String>) -> StartContinuityProtocolError {
    StartContinuityProtocolError(message.into())
}

fn start_error(value: impl fmt::Display) -> StartContinuityProtocolError {
    error(value.to_string())
}
