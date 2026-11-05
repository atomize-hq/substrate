//! Structural ancestry for World-bound local controllers. Each immutable
//! registration and successor journal retains its exact commitment. Typed
//! object-aware history separately authenticates every Start/native edge.
use super::*;

impl StateRootV3 {
    pub(super) fn has_world_controller_start(&self, current: &DurableSessionAuthorityV1) -> bool {
        let DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } = &current.origin;
        current.world_binding.is_some()
            && self.start_transaction_map.values().any(|transaction| {
                transaction.start_intent_id == *intent_id
                    && transaction.orchestration_session_id == current.orchestration_session_id
                    && transaction.public_scope == "world"
                    && match &transaction.state {
                        StartTransactionStateV1::ContinuationRegistered {
                            registration_ref,
                            ..
                        } => registration_ref.schema_version == 2,
                        StartTransactionStateV1::TurnSettledAwaitingResponse {
                            settlement_ref,
                            ..
                        }
                        | StartTransactionStateV1::PublicResponseDelivered {
                            settlement_ref, ..
                        } => settlement_ref.schema_version == 3,
                        StartTransactionStateV1::PromptNotSubmitted
                        | StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
                        | StartTransactionStateV1::PromptSubmissionIndeterminate { .. } => false,
                    }
                    && transaction.world_id.as_deref()
                        == current.world_binding.as_ref().map(|w| w.world_id.as_str())
                    && transaction.world_generation
                        == current.world_binding.as_ref().map(|w| w.world_generation)
            })
    }

    pub(super) fn reconstruct_world_controller_authority_history(
        &self,
        current: &DurableSessionAuthorityV1,
    ) -> Result<BTreeMap<u64, ReconstructedAuthorityStateV1>, StoreSchemaError> {
        let fail =
            || StoreSchemaError("World controller authority ancestry is disconnected or ambiguous");
        let DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } = &current.origin;
        let transactions = self
            .start_transaction_map
            .values()
            .filter(|t| {
                t.start_intent_id == *intent_id
                    && t.orchestration_session_id == current.orchestration_session_id
            })
            .collect::<Vec<_>>();
        let [transaction] = transactions.as_slice() else {
            return Err(fail());
        };
        let start_refs = current
            .internal_resume_handle_refs
            .iter()
            .filter(|r| {
                r.object_kind == AuthorityObjectKindV1::ResumeHandle
                    && matches!(r.schema_version, 2 | 3)
            })
            .collect::<Vec<_>>();
        if start_refs.len() > 2
            || start_refs.first().is_some_and(|r| r.schema_version != 2)
            || (start_refs.len() == 2 && start_refs[1].schema_version != 3)
        {
            return Err(fail());
        }
        let settlement = match &transaction.state {
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
            } => Some((
                settlement_ref,
                *authority_revision_after,
                *resulting_posture,
                completed_at,
            )),
            _ => None,
        };
        if settlement.is_some() != (start_refs.len() == 2)
            || settlement.is_some_and(|(r, ..)| Some(r) != start_refs.get(1).copied())
        {
            return Err(fail());
        }
        let mut history = self.reconstruct_origin_start_authority_history(current)?;
        let mut start_phase = 0;
        let mut retained = std::collections::BTreeSet::new();
        let mut initial = std::collections::BTreeSet::new();
        let mut completed = std::collections::BTreeSet::new();
        while history.last_key_value().ok_or_else(fail)?.0 < &current.authority_revision {
            let latest = history.last_key_value().ok_or_else(fail)?.1.clone();
            let next_revision = latest
                .authority
                .authority_revision
                .checked_add(1)
                .ok_or_else(fail)?;
            let mut candidates: Vec<(&str, String, ReconstructedAuthorityStateV1)> = Vec::new();
            for registration in
                self.retained_registration_candidates(&latest, &current.orchestration_session_id)
            {
                if retained.contains(&registration.registration_id) {
                    return Err(fail());
                }
                candidates.push((
                    "retained",
                    registration.registration_id.clone(),
                    self.reconstruct_retained_registration_state(&latest, registration)?,
                ));
            }
            for intent in self
                .successor_transition_intent_map
                .values()
                .filter(|intent| {
                    intent.orchestration_session_id == current.orchestration_session_id
                })
            {
                let HostSessionTransitionIntentStateV3::Applied {
                    authority_revision_before,
                    authority_revision_after,
                    authority_record_commitment,
                    post_turn,
                    ..
                } = &intent.state
                else {
                    continue;
                };
                let journal = self
                    .successor_application_journal
                    .get(&intent.intent_id)
                    .ok_or_else(fail)?;
                if *authority_revision_before == Some(latest.authority.authority_revision)
                    && !initial.contains(&intent.intent_id)
                {
                    if *authority_revision_after != next_revision
                        || journal.initial_application.authority_revision_before
                            != *authority_revision_before
                        || journal.initial_application.authority_revision_after
                            != *authority_revision_after
                        || journal.initial_application.authority_record_commitment
                            != *authority_record_commitment
                        || intent.mode != HostSessionTransitionModeV1::ResumeOneTurn
                        || intent.world_binding != current.world_binding
                    {
                        return Err(fail());
                    }
                    let next = reconstruct_successor_initial_state(
                        &latest,
                        intent,
                        &journal.initial_application.applied_at,
                    )?;
                    if next.authority_record_commitment != *authority_record_commitment {
                        return Err(fail());
                    }
                    candidates.push(("initial", intent.intent_id.clone(), next));
                }
                if let HostSessionPostTurnApplicationV2::Applied {
                    completion_ref,
                    authority_revision_before,
                    authority_revision_after,
                    resulting_posture,
                    applied_at,
                    ..
                } = post_turn.as_ref()
                {
                    if *authority_revision_before != latest.authority.authority_revision
                        || completed.contains(&intent.intent_id)
                    {
                        continue;
                    }
                    let phase = journal.post_turn_application.as_ref().ok_or_else(fail)?;
                    if !initial.contains(&intent.intent_id)
                        || *authority_revision_after != next_revision
                        || completion_ref.schema_version != 4
                        || completion_ref.object_kind != AuthorityObjectKindV1::PostTurnCompletion
                        || phase.completion_ref != **completion_ref
                        || phase.authority_revision_before != *authority_revision_before
                        || phase.authority_revision_after != *authority_revision_after
                    {
                        return Err(fail());
                    }
                    let next = match reconstruct_reconciled_authority_state(
                        &latest,
                        next_revision,
                        phase.authority_record_commitment.clone(),
                        *resulting_posture,
                        applied_at,
                    ) {
                        Ok(next) => next,
                        Err(_) => {
                            let with_handle =
                                self.reconstruct_completion_v4_authority_state(&latest, current)?;
                            reconstruct_reconciled_authority_state(
                                &with_handle,
                                next_revision,
                                phase.authority_record_commitment.clone(),
                                *resulting_posture,
                                applied_at,
                            )?
                        }
                    };
                    candidates.push(("completion", intent.intent_id.clone(), next));
                }
            }
            if candidates.is_empty() && start_phase < start_refs.len() {
                let mut authority = latest.authority.clone();
                authority.authority_revision = next_revision;
                if start_phase == 0 {
                    if authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached {
                        return Err(fail());
                    }
                    // For settled transactions the typed registered object owns
                    // the old timestamp. It is outside the authority commitment;
                    // exact object-aware history checks its real ordering.
                    if let StartTransactionStateV1::ContinuationRegistered {
                        registration_ref,
                        authority_revision_after,
                        registered_at,
                    } = &transaction.state
                    {
                        if *authority_revision_after != next_revision
                            || registration_ref != start_refs[0]
                        {
                            return Err(fail());
                        }
                        authority.updated_at = registered_at.clone();
                    }
                } else {
                    let (_, revision, posture, completed_at) = settlement.ok_or_else(fail)?;
                    if revision != next_revision {
                        return Err(fail());
                    }
                    authority.lifecycle_posture = posture;
                    authority.updated_at = completed_at.clone();
                }
                authority
                    .internal_resume_handle_refs
                    .push(start_refs[start_phase].clone());
                start_phase += 1;
                candidates.push((
                    "start",
                    String::new(),
                    reconstruct_authority_state(authority)?,
                ));
            }
            let [(kind, id, next)] = candidates.as_slice() else {
                return Err(fail());
            };
            if next.authority.authority_revision != next_revision {
                return Err(fail());
            }
            match *kind {
                "retained" => {
                    retained.insert(id.clone());
                }
                "initial" => {
                    initial.insert(id.clone());
                }
                "completion" => {
                    completed.insert(id.clone());
                }
                "start" => {}
                _ => return Err(fail()),
            }
            if history.insert(next_revision, next.clone()).is_some() {
                return Err(fail());
            }
        }
        let expected_initial = self
            .successor_transition_intent_map
            .values()
            .filter(|i| {
                i.orchestration_session_id == current.orchestration_session_id
                    && matches!(i.state, HostSessionTransitionIntentStateV3::Applied { .. })
            })
            .count();
        let expected_completed = self
            .successor_application_journal
            .values()
            .filter(|j| {
                self.successor_transition_intent_map
                    .get(&j.intent_id)
                    .is_some_and(|intent| {
                        intent.orchestration_session_id == current.orchestration_session_id
                    })
                    && j.post_turn_application.is_some()
            })
            .count();
        let last = history.last_key_value().ok_or_else(fail)?.1;
        if retained.len() != self.retained_registration_count(&current.orchestration_session_id)
            || initial.len() != expected_initial
            || completed.len() != expected_completed
            || start_phase != start_refs.len()
            || last.authority != *current
        {
            return Err(fail());
        }
        Ok(history)
    }
}
