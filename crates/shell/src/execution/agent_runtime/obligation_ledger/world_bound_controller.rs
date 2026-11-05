//! Exhaustive World work/attention meaning from retained physical sources.
//! This never reconciles a journal or treats controller completion as work completion.

use super::*;
use std::collections::{BTreeMap, BTreeSet};

use crate::execution::agent_runtime::{
    dispatch_policy_commitment::{
        decode_world_controller_e2_session_sources,
        resolve_accepted_work_receipt_material_from_snapshot,
        AcceptedWorkReceiptMaterialResolutionV1, DispatchPolicyCommitmentLookupKeyV1,
        DispatchPolicyCommitmentSubjectKeyV1, DispatchPolicyCommitmentSubjectV1,
    },
    host_session_authority::{
        schema::WorldBindingV1,
        store::{
            DispatchPolicyCommitmentPhysicalReadV1, HeldWorldControllerCompletionPhysicalSourcesV1,
            WorldWorkReceiptRegistryPhysicalReadV1,
        },
    },
    retained_worker_runtime::decode_world_controller_admission_sources,
    state_store::authenticate_ordinary_world_work_session_cut_v1,
    world_work_execution_supervisor::decode_world_controller_supervisor_observations,
};

pub(crate) use crate::execution::agent_runtime::host_session_authority::schema::{
    WorldControllerCompletedWorkV1, WorldControllerWorkAttentionCutV1,
};

/// The private constructor preserves the actual source borrow through the
/// consuming commit. Cloned serializable facts cannot recreate this capability.
pub(crate) struct AuthenticatedWorldControllerWorkAttentionCutV1<'capture, 'source> {
    sources: &'capture HeldWorldControllerCompletionPhysicalSourcesV1<'source>,
    cut: WorldControllerWorkAttentionCutV1,
}

impl AuthenticatedWorldControllerWorkAttentionCutV1<'_, '_> {
    pub(crate) fn cut(&self) -> &WorldControllerWorkAttentionCutV1 {
        &self.cut
    }

    pub(crate) fn physical_authority_snapshot(&self) -> &crate::execution::agent_runtime::host_session_authority::store::AcceptedWorkAuthorityPhysicalSnapshotV1{
        self.sources.accepted_work_source().snapshot()
    }

    pub(crate) fn verify_sources(&self) -> Result<()> {
        self.sources
            .verify_sources()
            .map_err(|error| anyhow::anyhow!("World work/attention custody changed: {error:?}"))
    }
}

/// Read every session proposal, acceptance and supervisor journal, together with
/// the complete canonical ledger subtree. Pending/unknown coverage remains
/// unresolved; absent B1 enrollment refuses before any empty-work inference.
pub(crate) fn authenticate_world_controller_work_attention_cut<'capture, 'source>(
    sources: &'capture HeldWorldControllerCompletionPhysicalSourcesV1<'source>,
    world: &WorldBindingV1,
) -> Result<AuthenticatedWorldControllerWorkAttentionCutV1<'capture, 'source>> {
    sources
        .verify_sources()
        .map_err(|error| anyhow::anyhow!("{error:?}"))?;
    let outcome = (|| {
        let snapshot = sources.accepted_work_source().snapshot();
        let session =
            authenticate_ordinary_world_work_session_cut_v1(snapshot, sources.session_id(), world)?;
        let (reservations, commitments) =
            decode_world_controller_e2_session_sources(snapshot, session.session_id(), world)?;
        let bootstrap_sources = decode_world_controller_admission_sources(
            sources.accepted_work_source(),
            sources.admission_registry_bytes(),
            &sources.admission_key_files(),
            session.session_id(),
            world,
        )?;
        let mut e2_session_source_ids = Vec::new();
        let mut unresolved_e2_source_ids = Vec::new();
        for reservation in &reservations {
            let id = format!("reservation:{}", reservation.reservation_id);
            e2_session_source_ids.push(id.clone());
            let completed = bootstrap_sources
                .completed_bootstraps
                .iter()
                .chain(bootstrap_sources.retired_bootstraps.iter())
                .any(|activation| {
                    activation
                        .reservation_ref
                        .as_ref()
                        .is_some_and(|reference| {
                            reference.authority_store_id == reservation.authority_store_id
                                && reference.reservation_id == reservation.reservation_id
                                && reference.reservation_hash == reservation.reservation_hash
                        })
                        && activation.commitment_ref.commitment_id
                            == reservation.proposed_commitment_id
                        && activation.retained_participant_id == reservation.retained_participant_id
                        && activation.bootstrap_run_id == reservation.bootstrap_run_id
                });
            if !completed {
                unresolved_e2_source_ids.push(id);
            }
        }
        for commitment in &commitments {
            let id = format!("commitment:{}", commitment.commitment_id);
            e2_session_source_ids.push(id.clone());
            let accepted = session.acceptances().iter().any(|acceptance| {
                acceptance.request_id == commitment.request_id
                    && match (&acceptance.work_identity, &commitment.subject) {
                        (
                            AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id: left },
                            DispatchPolicyCommitmentSubjectV1::EphemeralWork { task_run_id: right },
                        ) => left == right,
                        (
                            AcceptedWorldWorkIdentityV1::RetainedTurn {
                                active_run_id: left_run,
                                message_id: left_message,
                                target_participant_id: left_participant,
                            },
                            DispatchPolicyCommitmentSubjectV1::RetainedWorkerTurn {
                                active_run_id: right_run,
                                message_id: right_message,
                                retained_participant_id: right_participant,
                            },
                        ) => {
                            left_run == right_run
                                && left_message == right_message
                                && left_participant == right_participant
                        }
                        _ => false,
                    }
            });
            let bootstrap_completed =
                bootstrap_sources
                    .completed_bootstraps
                    .iter()
                    .any(|activation| {
                        activation.commitment_ref.authority_store_id
                            == commitment.authority_store_id
                            && activation.commitment_ref.commitment_id == commitment.commitment_id
                            && activation.commitment_ref.exact_linkage_hash
                                == commitment.exact_linkage_hash
                    });
            let bootstrap_retired = bootstrap_sources
                .retired_bootstraps
                .iter()
                .any(|activation| {
                    activation.commitment_ref.authority_store_id == commitment.authority_store_id
                        && activation.commitment_ref.commitment_id == commitment.commitment_id
                        && activation.commitment_ref.exact_linkage_hash
                            == commitment.exact_linkage_hash
                });
            if !accepted && !bootstrap_completed && !bootstrap_retired {
                unresolved_e2_source_ids.push(id);
            }
        }
        e2_session_source_ids.sort();
        unresolved_e2_source_ids.sort();
        // Native bootstrap completion is distinct from logical worker life.
        // Admission state alone, including Terminal, never supplies this fact.
        let unresolved_admission_participant_ids =
            bootstrap_sources.unresolved_admission_participant_ids;
        let unclassified_hsa_retained_participant_ids =
            bootstrap_sources.unclassified_hsa_retained_participant_ids;
        let observations = decode_world_controller_supervisor_observations(
            sources.supervisor_bytes(),
            snapshot.authority_store_id(),
            session.session_id(),
        )?;
        let observations: BTreeMap<_, _> = observations
            .into_iter()
            .map(|observation| (observation.claim.acceptance_record_id.clone(), observation))
            .collect();
        anyhow::ensure!(
            observations.keys().all(|id| session
                .acceptances()
                .iter()
                .any(|record| &record.acceptance_record_id == id)),
            "World supervisor contains session work outside complete B1 coverage"
        );

        let mut states = BTreeMap::new();
        let mut events: BTreeMap<String, Vec<MaterializedObligationLedgerEventV1>> =
            BTreeMap::new();
        let mut obligations: BTreeMap<String, Vec<OrchestrationObligationRecord>> = BTreeMap::new();
        let mut cursor = None;
        for (path, bytes) in sources.ledger_files() {
            let parts: Vec<_> = path.split('/').collect();
            match parts.as_slice() {
                ["revision-cursor.json"] => {
                    let value: ObligationLedgerRevisionCursorV1 = serde_json::from_slice(bytes)?;
                    value.validate()?;
                    cursor = Some(value);
                }
                ["acceptances", id, "state.json"] => {
                    let value: ObligationLedgerSessionStateV1 = serde_json::from_slice(bytes)?;
                    value.validate()?;
                    anyhow::ensure!(
                        value.acceptance_record_id == *id,
                        "World ledger state filename differs"
                    );
                    states.insert((*id).to_owned(), value);
                }
                ["acceptances", id, "events", name] => {
                    let value: MaterializedObligationLedgerEventV1 = serde_json::from_slice(bytes)?;
                    value.validate()?;
                    anyhow::ensure!(
                        value.acceptance_record_id == *id
                            && *name
                                == format!(
                                    "{:020}.json",
                                    value.source_journal_event.event_sequence
                                ),
                        "World ledger event filename differs"
                    );
                    events.entry((*id).to_owned()).or_default().push(value);
                }
                ["acceptances", id, "obligations", name] => {
                    let value: OrchestrationObligationRecord = serde_json::from_slice(bytes)?;
                    value.validate()?;
                    anyhow::ensure!(
                        *name == format!("{}.json", value.obligation_id)
                            && value
                                .source_journal_event
                                .as_ref()
                                .is_some_and(|source| source.acceptance_record_id == *id),
                        "World ledger obligation source/filename differs"
                    );
                    obligations.entry((*id).to_owned()).or_default().push(value);
                }
                _ => anyhow::bail!("World ledger contains an uncovered artifact {path}"),
            }
        }
        for id in states.keys().chain(events.keys()).chain(obligations.keys()) {
            anyhow::ensure!(
                session
                    .acceptances()
                    .iter()
                    .any(|record| &record.acceptance_record_id == id),
                "World ledger contains work outside complete session B1 coverage"
            );
        }
        if !sources.ledger_files().is_empty() {
            let cursor = cursor
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("World ledger revision source is missing"))?;
            anyhow::ensure!(
                states
                    .values()
                    .all(|state| state.session_ledger_revision
                        <= cursor.current_session_ledger_revision),
                "World ledger cursor is behind canonical state"
            );
        }

        let mut pending_request_ids = Vec::new();
        for proposal in session.proposals() {
            if !session
                .acceptances()
                .iter()
                .any(|acceptance| acceptance.request_id == proposal.request_id())
            {
                pending_request_ids.push(proposal.request_id().to_owned());
            }
        }
        let mut unresolved_work_acceptance_ids = Vec::new();
        let mut completed_work = Vec::new();
        let mut unresolved_attention = Vec::new();
        let mut unresolved_attention_acceptance_ids = Vec::new();
        for acceptance in session.acceptances() {
            let id = &acceptance.acceptance_record_id;
            let Some(observation) = observations.get(id) else {
                unresolved_work_acceptance_ids.push(id.clone());
                continue;
            };
            anyhow::ensure!(
                observation.claim.matches_acceptance(acceptance),
                "World supervisor claim differs from B1"
            );
            let Some(terminal) = observation
                .terminal
                .as_ref()
                .filter(|_| observation.interruption.is_none())
            else {
                unresolved_work_acceptance_ids.push(id.clone());
                continue;
            };
            let subject = match &acceptance.work_identity {
                AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } => {
                    DispatchPolicyCommitmentSubjectKeyV1::EphemeralWork {
                        task_run_id: task_run_id.clone(),
                    }
                }
                AcceptedWorldWorkIdentityV1::RetainedTurn {
                    active_run_id,
                    message_id,
                    ..
                } => DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerTurn {
                    active_run_id: active_run_id.clone(),
                    message_id: message_id.clone(),
                },
            };
            let key = DispatchPolicyCommitmentLookupKeyV1 {
                authority_store_id: snapshot.authority_store_id().into(),
                orchestration_session_id: session.session_id().into(),
                request_id: acceptance.request_id.clone(),
                subject,
            };
            let receipt =
                resolve_accepted_work_receipt_material_from_snapshot(snapshot, &key, acceptance)
                    .map_err(|error| {
                        anyhow::anyhow!("World work policy source refused: {error:?}")
                    })?;
            let AcceptedWorkReceiptMaterialResolutionV1::Resolved(receipt) = receipt else {
                unresolved_work_acceptance_ids.push(id.clone());
                continue;
            };
            let claim = receipt.execution_claim();
            anyhow::ensure!(
                claim.canonical_preimage() == canonical_json::to_vec(&observation.claim)?
                    && claim.claim_revision() == observation.claim.claim_revision
                    && claim.observer_epoch() == observation.claim.observer_epoch,
                "World journal claim differs from authenticated E2 receipt"
            );
            match &acceptance.work_identity {
                AcceptedWorldWorkIdentityV1::RetainedTurn { .. } => {
                    let expected = expected_retained_obligation_materialization_from_observation(
                        acceptance,
                        observation,
                    )?;
                    let Some(state) = states.get(id) else {
                        unresolved_work_acceptance_ids.push(id.clone());
                        continue;
                    };
                    validate_state_matches_acceptance(state, acceptance)?;
                    let stored_events = events.get(id).map(Vec::as_slice).unwrap_or(&[]);
                    if state.terminal_event_id != expected.terminal_event_id
                        || state.terminal_event_sequence != expected.terminal_event_sequence
                        || state.materialized_through_event_sequence
                            != expected.materialized_through_event_sequence
                        || stored_events != expected.materialized_events
                    {
                        unresolved_work_acceptance_ids.push(id.clone());
                        continue;
                    }
                    let stored_obligations = obligations.get(id).map(Vec::as_slice).unwrap_or(&[]);
                    if stored_obligations.len() != expected.projected_obligations.len()
                        || expected.projected_obligations.iter().any(|expected| {
                            !stored_obligations.iter().any(|stored| {
                                stored.matches_c1_materialization_projection(expected)
                            })
                        })
                    {
                        unresolved_work_acceptance_ids.push(id.clone());
                        continue;
                    }
                    unresolved_attention.extend(
                        unresolved_attention_entries_for_materialized_obligations(
                            stored_obligations,
                        )?,
                    );
                }
                AcceptedWorldWorkIdentityV1::EphemeralTask { .. } => {
                    unresolved_attention_acceptance_ids.push(id.clone());
                    // C1's retained-event materializer cannot certify a different
                    // event family. Preserve uncertainty instead of discarding it.
                    if observation.journal.iter().any(|entry| {
                        entry.event_identity.is_some()
                            && entry.event_identity.as_ref() != Some(&terminal.event_identity)
                    }) || states.contains_key(id)
                        || events.contains_key(id)
                        || obligations.contains_key(id)
                    {
                        unresolved_work_acceptance_ids.push(id.clone());
                        continue;
                    }
                }
            }
            completed_work.push(WorldControllerCompletedWorkV1 {
                acceptance_record_id: id.clone(),
                acceptance_record_revision: acceptance.record_revision,
                accepted_work_identity: acceptance.work_identity.clone(),
                stream_id: acceptance.runtime_acceptance.stream_id.clone(),
                terminal_event_id: terminal.event_identity.event_id.clone(),
                terminal_event_sequence: terminal.event_identity.event_sequence,
                exit_code: terminal.exit_code,
                e2_commitment_id: receipt.commitment_ref().commitment_id.clone(),
                e2_commitment_hash: receipt.commitment_ref().exact_linkage_hash.clone(),
            });
        }
        sort_unresolved_attention_entries(&mut unresolved_attention);
        anyhow::ensure!(
            unresolved_attention
                .iter()
                .map(|entry| &entry.obligation_id)
                .collect::<BTreeSet<_>>()
                .len()
                == unresolved_attention.len(),
            "World attention source contains duplicate obligation identities"
        );
        let digest = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
        let WorldWorkReceiptRegistryPhysicalReadV1::Present(b1) = session.snapshot().b1() else {
            anyhow::bail!("B1 source disappeared");
        };
        let cut = WorldControllerWorkAttentionCutV1 {
            schema_version: 1,
            authority_store_id: snapshot.authority_store_id().into(),
            orchestration_session_id: session.session_id().into(),
            world_binding: session.world_binding().clone(),
            hsa_root_revision: snapshot.hsa_root_revision_at_read(),
            e2_registry_sha256: match snapshot.e2() {
                DispatchPolicyCommitmentPhysicalReadV1::Absent => None,
                DispatchPolicyCommitmentPhysicalReadV1::Present(e2) => {
                    Some(digest(e2.registry_bytes()))
                }
            },
            admission_registry_sha256: sources.admission_registry_bytes().map(digest),
            e2_session_source_ids,
            unresolved_e2_source_ids,
            unresolved_admission_participant_ids,
            unclassified_hsa_retained_participant_ids,
            b1_registry_sha256: digest(b1.registry_bytes()),
            supervisor_sha256: sources.supervisor_bytes().map(digest),
            ledger_files_sha256: sources
                .ledger_files()
                .iter()
                .map(|(path, bytes)| (path.clone(), digest(bytes)))
                .collect(),
            proposal_request_ids: session
                .proposals()
                .iter()
                .map(|proposal| proposal.request_id().to_owned())
                .collect(),
            acceptance_record_ids: session
                .acceptances()
                .iter()
                .map(|record| record.acceptance_record_id.clone())
                .collect(),
            pending_request_ids,
            unresolved_work_acceptance_ids,
            unresolved_attention_acceptance_ids,
            completed_work,
            unresolved_attention,
        };
        use crate::execution::agent_runtime::host_session_authority::validation::ValidatedCanonicalV1 as _;
        cut.validate()?;
        Ok(AuthenticatedWorldControllerWorkAttentionCutV1 { sources, cut })
    })();
    // Refusal still finalizes the physical sources that were actually read.
    sources
        .verify_sources()
        .map_err(|error| anyhow::anyhow!("World work/attention custody changed: {error:?}"))?;
    outcome
}
