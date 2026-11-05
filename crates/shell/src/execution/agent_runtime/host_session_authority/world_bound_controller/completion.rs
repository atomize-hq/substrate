//! Immutable World controller completion joins. History leaves consume an
//! authenticated prefix; they never recurse into full history reconstruction.

use anyhow::{ensure, Result};
use std::collections::BTreeMap;

use super::super::{
    canonical_json,
    facade::{exact_v3_authority_history_with_reader, ResolvedSessionAuthorityV1},
    hash::canonical_sha256,
    schema::*,
    store::BootstrapError,
    store_schema::*,
    validation::{CanonicalHashInputV1, ValidatedCanonicalV1},
};
use super::{AuthenticatedWorldBoundControllerRunV1, WorldBoundControllerEntryV1};
use crate::execution::agent_runtime::obligation_ledger::world_bound_controller::AuthenticatedWorldControllerWorkAttentionCutV1;

fn prepare_world_controller_object(
    kind: AuthorityObjectKindV1,
    version: u32,
    input: super::super::hash::CanonicalObjectHashInputV1<'_>,
) -> Result<(AuthorityObjectRefV1, Vec<u8>)> {
    use sha2::{Digest, Sha256};
    let bytes = super::super::hash::canonical_object_bytes(kind, input)?;
    let digest_hex = format!("{:x}", Sha256::digest(&bytes));
    Ok((
        AuthorityObjectRefV1 {
            ref_id: format!("ao_{}", &digest_hex[..32]),
            object_kind: kind,
            schema_version: version,
            commitment: AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex },
        },
        bytes,
    ))
}

fn insert_world_controller_object(
    root: &mut StateRootV3,
    object: &(AuthorityObjectRefV1, Vec<u8>),
) -> Result<()> {
    let (reference, bytes) = object;
    ensure!(
        !root.object_index.contains_key(&reference.ref_id),
        "World completion object index collision"
    );
    root.object_index.insert(
        reference.ref_id.clone(),
        AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: reference.ref_id.clone(),
            object_kind: reference.object_kind,
            object_schema_version: reference.schema_version,
            byte_length: bytes.len() as u64,
            storage_state: AuthorityObjectStorageStateV1::Present,
        },
    );
    Ok(())
}

fn world_controller_authority_commitment(
    authority: &DurableSessionAuthorityV1,
) -> Result<AuthorityObjectCommitmentV1> {
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&DurableSessionAuthorityHashInputV1 {
            schema_version: authority.schema_version,
            orchestration_session_id: authority.orchestration_session_id.clone(),
            shell_trace_session_id: authority.shell_trace_session_id.clone(),
            authority_revision: authority.authority_revision,
            origin: authority.origin.clone(),
            authoritative_participant_lineage: authority.authoritative_participant_lineage.clone(),
            active_authoritative_participant_id: authority
                .active_authoritative_participant_id
                .clone(),
            workspace_binding: authority.workspace_binding.clone(),
            world_binding: authority.world_binding.clone(),
            host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
            retained_worker_refs: authority.retained_worker_refs.clone(),
            internal_resume_handle_refs: authority.internal_resume_handle_refs.clone(),
            lifecycle_posture: authority.lifecycle_posture,
            current_policy_ref: authority.current_policy_ref.clone(),
            current_policy_revision: authority.current_policy_revision.clone(),
        })?,
    })
}

impl super::super::HostSessionAuthority {
    /// Exact HSA completion for the genuine local controller. The retained cut
    /// describes separate World work; it never becomes a worker receipt or an
    /// execution grant. The same physical guards survive final root publication.
    pub(crate) fn settle_ordinary_world_controller_resume(
        &self,
        controller: &AuthenticatedWorldBoundControllerRunV1,
        native: &DirectHostNativeTerminalEvidenceV1,
        kind: HostPostTurnProtocolEventKindV1,
        emitted_at: &TimestampV1,
        cut: &AuthenticatedWorldControllerWorkAttentionCutV1<'_, '_>,
    ) -> Result<super::super::transition::PostTurnResolutionOutcomeV1> {
        use super::super::transition::PostTurnResolutionOutcomeV1 as Outcome;
        use super::super::{hash::CanonicalObjectHashInputV1 as Input, store};
        controller.verify_workspace()?;
        ensure!(
            controller.entry() == WorldBoundControllerEntryV1::ResumeOneTurn
                && self.trusted_root().identity()
                    == controller
                        .selection()
                        .held_authority()
                        .trusted_root()
                        .identity(),
            "World Resume requires its genuine issued controller/home"
        );
        let current = self.read_a12b_root()?;
        let intent = current
            .successor_transition_intent_map
            .get(controller.intent_id())
            .ok_or_else(|| anyhow::anyhow!("World Resume issued intent absent"))?;
        ensure!(
            intent.run_id == controller.run_id(),
            "World Resume run changed"
        );
        let HostSessionTransitionIntentStateV3::Applied {
            claim_id,
            claimant_attempt_id,
            application_result_ref,
            post_turn,
            ..
        } = &intent.state
        else {
            anyhow::bail!("World Resume is not Applied");
        };
        if let HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } = post_turn.as_ref()
        {
            let completed = verify_world_controller_completed_join(
                &current,
                intent,
                completion_ref,
                &mut |reference| {
                    self.read_authority_object_v2_at(current.root_revision, reference)
                        .map_err(|_| BootstrapError::world_controller_source_guard())
                },
            )?;
            ensure!(
                &completed.native_terminal == native
                    && completed.kind == kind
                    && completed.binding.native_selection
                        == *controller.selection().native_selection()
                    && completed.binding.requested_selection
                        == *controller.selection().requested_selection()
                    && completed.binding.world_binding == *controller.world_binding()
                    && &completed.work_attention_cut == cut.cut(),
                "World Resume retry conflicts with its immutable completion"
            );
            return Ok(Outcome::Joined(intent.clone()));
        }
        let physical = cut.physical_authority_snapshot();
        ensure!(
            physical.hsa_state_root_bytes() == canonical_json::to_vec(&current)?
                && physical.authority_root_identity() == self.trusted_root().identity(),
            "World Resume cut does not hold the exact committing authority"
        );
        cut.verify_sources()?;
        let selection = controller.selection().refresh_original_selection()?;
        ensure!(
            std::sync::Arc::ptr_eq(
                selection.held_authority(),
                controller.selection().held_authority()
            ) && selection.native_selection() == controller.selection().native_selection()
                && selection.requested_selection() == controller.selection().requested_selection(),
            "World Resume original role custody changed"
        );
        selection.revalidate()?;
        let resolved = selection
            .held_authority()
            .resolve_current_exact(&intent.orchestration_session_id, None)?;
        ensure!(
            resolved.observation.root_revision == current.root_revision
                && resolved.caller == controller.origin().caller
                && resolved.authority.world_binding.as_ref() == Some(controller.world_binding())
                && matches!(post_turn.as_ref(), HostSessionPostTurnApplicationV2::Pending { expected_run_id, expected_authority_revision }
                if expected_run_id == controller.run_id() && *expected_authority_revision <= resolved.authority.authority_revision),
            "World Resume current target or pending application changed"
        );
        let HostSessionTransitionInputHandoffV1::Accepted {
            input_ref,
            run_id,
            acceptance_ref,
            ..
        } = &intent.input_handoff
        else {
            anyhow::bail!("World Resume cannot settle without exact Accepted input");
        };
        ensure!(
            run_id == controller.run_id(),
            "World Resume Accepted run differs"
        );
        let b = WorldControllerResumeBindingV1 {
            schema_version: 1,
            authority_store_id: current.authority_store_id.clone(),
            bootstrap_home: current.bootstrap_home.clone(),
            workspace_binding: resolved.authority.workspace_binding.clone(),
            world_binding: controller.world_binding().clone(),
            orchestration_session_id: intent.orchestration_session_id.clone(),
            shell_trace_session_id: intent.shell_trace_session_id.clone(),
            authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            authoritative_lineage: resolved.authority.authoritative_participant_lineage.clone(),
            original_start_intent_id: selection.original_start_intent_id().into(),
            original_start_run_id: selection.context().original_run_id.clone(),
            native_selection: selection.native_selection().clone(),
            requested_selection: selection.requested_selection().clone(),
            authority_revision_observed: resolved.authority.authority_revision,
            authority_record_commitment: resolved.observation.authority_record_commitment.clone(),
            policy_ref: resolved
                .authority
                .current_policy_ref
                .clone()
                .ok_or_else(|| anyhow::anyhow!("World current policy reference absent"))?,
            policy_revision: resolved.current_policy.policy_revision.clone(),
            descriptor_ref: intent.descriptor_ref.clone(),
            intent_id: intent.intent_id.clone(),
            issuer_request_id: intent.issuer_request_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            intent_revision_observed: intent.intent_revision,
            claim_id: claim_id.clone(),
            claimant_attempt_id: claimant_attempt_id.clone(),
            run_id: intent.run_id.clone(),
            initial_application_result_ref: application_result_ref.clone(),
            transition_input_ref: input_ref.clone(),
            input_acceptance_ref: acceptance_ref.clone(),
            consumed_resume_handle_ref: intent
                .resume_handle_ref
                .clone()
                .ok_or_else(|| anyhow::anyhow!("World Resume consumed no continuation"))?,
        };
        let prefix = exact_v3_authority_history_with_reader(
            &current,
            &b.orchestration_session_id,
            |reference| {
                self.read_authority_object_v2_at(current.root_revision, reference)
                    .map_err(|error| error.to_string())
            },
        )?;
        let (descriptor, native_session) =
            verify_world_completion_binding(&current, intent, &b, &prefix, &mut |reference| {
                self.read_authority_object_v2_at(current.root_revision, reference)
                    .map_err(|_| BootstrapError::world_controller_source_guard())
            })?;
        verify_world_completion_native(native, &kind, &descriptor, &native_session)?;
        let produced = if kind == HostPostTurnProtocolEventKindV1::ResumableClean {
            Some(prepare_world_controller_object(
                AuthorityObjectKindV1::ResumeHandle,
                1,
                Input::ResumeHandle(&ResumeHandleHashInputV1 {
                    schema_version: 1,
                    orchestration_session_id: b.orchestration_session_id.clone(),
                    participant_id: b.authoritative_participant_id.clone(),
                    backend_id: native.backend_id.clone(),
                    protocol: native.protocol.clone(),
                    internal_uaa_session_id: native.native_session_id.clone(),
                }),
            )?)
        } else {
            None
        };
        let completion_value = PostTurnCompletionHashInputV4 {
            schema_version: 4,
            binding: b.clone(),
            protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: b.authoritative_participant_id.clone(),
            },
            event_id: world_controller_terminal_event_id(&b, native)?,
            kind,
            native_terminal: native.clone(),
            work_attention_cut: cut.cut().clone(),
            completed_at: emitted_at.clone(),
            produced_resume_handle_ref: produced.as_ref().map(|object| object.0.clone()),
        };
        let completion = prepare_world_controller_object(
            AuthorityObjectKindV1::PostTurnCompletion,
            4,
            Input::PostTurnCompletionV4(&completion_value),
        )?;
        let mut next_authority = resolved.authority.clone();
        next_authority.authority_revision = next_authority
            .authority_revision
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("World authority revision overflow"))?;
        next_authority.lifecycle_posture = completion_value.resulting_posture();
        next_authority.updated_at = emitted_at.clone();
        if let Some((reference, _)) = &produced {
            next_authority
                .internal_resume_handle_refs
                .push(reference.clone());
        }
        let commitment = world_controller_authority_commitment(&next_authority)?;
        let application = prepare_world_controller_object(
            AuthorityObjectKindV1::ApplicationResult,
            1,
            Input::ApplicationResult(&ApplicationResultHashInputV1 {
                schema_version: 1,
                intent_id: b.intent_id.clone(),
                mode: HostSessionTransitionModeV1::ResumeOneTurn,
                run_id: b.run_id.clone(),
                phase: ApplicationResultPhaseV1::PostTurn {
                    completion_ref: completion.0.clone(),
                    authority_revision_before: b.authority_revision_observed,
                    authority_revision_after: next_authority.authority_revision,
                    active_authoritative_participant_id: b.authoritative_participant_id.clone(),
                    resulting_posture: next_authority.lifecycle_posture,
                    authority_record_commitment: commitment.clone(),
                },
                applied_at: emitted_at.clone(),
            }),
        )?;
        let terminal = prepare_world_controller_object(
            AuthorityObjectKindV1::TerminalHandoff,
            1,
            Input::TerminalHandoff(&TerminalHandoffHashInputV1 {
                schema_version: 1,
                intent_id: b.intent_id.clone(),
                run_id: b.run_id.clone(),
                payload_commitment: b.payload_commitment.clone(),
                terminal_state: TerminalHandoffStateV1::Applied,
                application_result_ref: Some(b.initial_application_result_ref.clone()),
                input_acceptance_ref: Some(b.input_acceptance_ref.clone()),
                post_turn_completion_ref: Some(completion.0.clone()),
                post_turn_application_result_ref: Some(application.0.clone()),
                recorded_at: emitted_at.clone(),
            }),
        )?;
        let mut proposed = current.clone();
        proposed.root_revision = proposed
            .root_revision
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("World root revision overflow"))?;
        proposed.session_namespace_map.insert(
            b.orchestration_session_id.clone(),
            SessionNamespaceRecordV1::Authority(Box::new(next_authority.clone())),
        );
        let next = proposed
            .successor_transition_intent_map
            .get_mut(&b.intent_id)
            .ok_or_else(|| anyhow::anyhow!("World Resume intent disappeared"))?;
        next.intent_revision = next
            .intent_revision
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("World intent revision overflow"))?;
        next.updated_at = emitted_at.clone();
        next.transport_payload_state =
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref: terminal.0.clone(),
            };
        let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &mut next.state else {
            anyhow::bail!("World Resume lost Applied state");
        };
        *post_turn = Box::new(HostSessionPostTurnApplicationV2::Applied {
            completion_ref: Box::new(completion.0.clone()),
            obligation_snapshot_ref: None,
            authority_revision_before: b.authority_revision_observed,
            authority_revision_after: next_authority.authority_revision,
            resulting_posture: next_authority.lifecycle_posture,
            application_result_ref: Box::new(application.0.clone()),
            applied_at: emitted_at.clone(),
        });
        proposed
            .successor_application_journal
            .get_mut(&b.intent_id)
            .ok_or_else(|| anyhow::anyhow!("World Resume journal disappeared"))?
            .post_turn_application = Some(PostTurnApplicationJournalV2 {
            completion_ref: completion.0.clone(),
            obligation_snapshot_ref: None,
            authority_revision_before: b.authority_revision_observed,
            authority_revision_after: next_authority.authority_revision,
            authority_record_commitment: commitment,
            application_result_ref: application.0.clone(),
            applied_at: emitted_at.clone(),
        });
        proposed
            .object_index
            .get_mut(&intent.transport_payload_ref.ref_id)
            .ok_or_else(|| anyhow::anyhow!("World Resume transport index disappeared"))?
            .storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
            terminal_handoff_ref: terminal.0.clone(),
        };
        let objects = [&completion, &application, &terminal]
            .into_iter()
            .chain(produced.iter())
            .collect::<Vec<_>>();
        for object in &objects {
            insert_world_controller_object(&mut proposed, object)?;
        }
        proposed.validate()?;
        verify_world_controller_completion_v4_edge(
            &proposed,
            &proposed.successor_transition_intent_map[&b.intent_id],
            &completion.0,
            &prefix,
            &mut |reference| {
                if let Some(object) = objects.iter().find(|object| &object.0 == reference) {
                    return Ok(object.1.clone());
                }
                self.read_authority_object_v2_at(current.root_revision, reference)
                    .map_err(|_| BootstrapError::world_controller_source_guard())
            },
        )?;
        controller.verify_workspace()?;
        cut.verify_sources()?;
        for (reference, bytes) in &objects {
            store::prepare_typed_object_v3_opened(
                self.trusted_root(),
                current.root_revision,
                reference,
                bytes,
                None,
            )?;
        }
        store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &current,
            &proposed,
            || {
                controller
                    .verify_workspace()
                    .map_err(|_| BootstrapError::world_controller_source_guard())?;
                cut.verify_sources()
                    .map_err(|_| BootstrapError::world_controller_source_guard())
            },
        )?;
        Ok(Outcome::Applied(
            proposed.successor_transition_intent_map[&b.intent_id].clone(),
        ))
    }
}

fn read_world_completion_object<T>(
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
    reference: &AuthorityObjectRefV1,
) -> Result<T>
where
    T: serde::de::DeserializeOwned + CanonicalHashInputV1,
{
    let value: T = canonical_json::from_slice(&reader(reference)?)?;
    value.validate()?;
    ensure!(
        reference.commitment
            == AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&value)?,
            },
        "World completion public object commitment differs"
    );
    Ok(value)
}

fn verify_world_completion_binding(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    b: &WorldControllerResumeBindingV1,
    prefix: &BTreeMap<u64, ResolvedSessionAuthorityV1>,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<(AgentDescriptorV1, String)> {
    b.validate()?;
    ensure!(
        root.successor_transition_intent_map.get(&b.intent_id) == Some(intent)
            && root.authority_store_id == b.authority_store_id
            && root.bootstrap_home == b.bootstrap_home
            && intent.mode == HostSessionTransitionModeV1::ResumeOneTurn
            && intent.intent_id == b.intent_id
            && intent.issuer_request_id == b.issuer_request_id
            && intent.payload_commitment == b.payload_commitment
            && intent.run_id == b.run_id
            && intent.orchestration_session_id == b.orchestration_session_id
            && intent.shell_trace_session_id == b.shell_trace_session_id
            && intent.workspace_binding == b.workspace_binding
            && intent.world_binding.as_ref() == Some(&b.world_binding)
            && intent.target_authoritative_participant_id == b.authoritative_participant_id
            && b.authoritative_lineage
                .starts_with(&intent.resulting_authoritative_lineage)
            && intent.descriptor_ref == b.descriptor_ref
            && intent.transition_input_ref.as_ref() == Some(&b.transition_input_ref)
            && intent.resume_handle_ref.as_ref() == Some(&b.consumed_resume_handle_ref),
        "World completion intent/source binding differs"
    );
    let HostSessionTransitionIntentStateV3::Applied {
        claim_id,
        claimant_attempt_id,
        authority_revision_before,
        authority_revision_after,
        active_authoritative_participant_id,
        authority_record_commitment,
        application_result_ref,
        ..
    } = &intent.state
    else {
        anyhow::bail!("World completion parent is not Applied");
    };
    ensure!(
        claim_id == &b.claim_id
            && claimant_attempt_id == &b.claimant_attempt_id
            && *authority_revision_after <= b.authority_revision_observed
            && active_authoritative_participant_id == &b.authoritative_participant_id
            && application_result_ref == &b.initial_application_result_ref,
        "World completion claim/attempt/application differs"
    );
    let applied = prefix
        .get(authority_revision_after)
        .ok_or_else(|| anyhow::anyhow!("World applied authority prefix missing"))?;
    ensure!(
        &applied.authority_record_commitment == authority_record_commitment
            && applied.authority.authoritative_participant_lineage
                == intent.resulting_authoritative_lineage,
        "World completion initial application commitment differs"
    );
    let latest = prefix
        .get(&b.authority_revision_observed)
        .ok_or_else(|| anyhow::anyhow!("World completion current authority prefix missing"))?;
    ensure!(
        prefix.last_key_value().map(|(revision, _)| *revision)
            == Some(b.authority_revision_observed)
            && latest.authority_record_commitment == b.authority_record_commitment
            && latest
                .authority
                .active_authoritative_participant_id
                .as_ref()
                == Some(&b.authoritative_participant_id)
            && latest.authority.authoritative_participant_lineage == b.authoritative_lineage
            && latest.authority.lifecycle_posture == HostSessionPostureV1::ActiveAttached
            && latest.authority.current_policy_ref.as_ref() == Some(&b.policy_ref)
            && latest.authority.current_policy_revision.as_ref() == Some(&b.policy_revision)
            && prefix
                .values()
                .all(
                    |state| state.authority.world_binding.as_ref() == Some(&b.world_binding)
                        && state.authority.workspace_binding == b.workspace_binding
                        && state.authority.shell_trace_session_id == b.shell_trace_session_id
                ),
        "World completion exact authority/World prefix differs"
    );
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision,
        authority_record_commitment: expected_source,
        active_authoritative_participant_id: source_actor,
        authoritative_lineage_commitment,
        lifecycle_posture,
    } = &intent.authority_precondition
    else {
        anyhow::bail!("World Resume lacks exact source precondition");
    };
    let source = prefix
        .get(authority_revision)
        .ok_or_else(|| anyhow::anyhow!("World Resume source history missing"))?;
    ensure!(
        authority_revision_before.as_ref() == Some(authority_revision)
            && source.authority_record_commitment == *expected_source
            && source.authoritative_lineage_commitment == *authoritative_lineage_commitment
            && source.authority.lifecycle_posture == *lifecycle_posture
            && source
                .authority
                .active_authoritative_participant_id
                .as_ref()
                == Some(source_actor)
            && intent.source_authoritative_participant_id.as_ref() == Some(source_actor)
            && source
                .authority
                .internal_resume_handle_refs
                .contains(&b.consumed_resume_handle_ref),
        "World completion consumed handle/source precondition differs"
    );
    let (handle_session, handle_owner, handle_backend, handle_protocol, native_session_id) =
        match b.consumed_resume_handle_ref.schema_version {
            1 => {
                let handle: ResumeHandleHashInputV1 =
                    read_world_completion_object(reader, &b.consumed_resume_handle_ref)?;
                (
                    handle.orchestration_session_id,
                    handle.participant_id,
                    handle.backend_id,
                    handle.protocol,
                    handle.internal_uaa_session_id,
                )
            }
            3 => {
                let handle: StartContinuationHandleHashInputV3 =
                    read_world_completion_object(reader, &b.consumed_resume_handle_ref)?;
                ensure!(
                    handle.world_binding == b.world_binding
                        && handle.native_selection == b.native_selection
                        && handle.requested_selection == b.requested_selection,
                    "consumed World Start role binding differs"
                );
                let handle = handle.continuation;
                (
                    handle.orchestration_session_id,
                    handle.participant_id,
                    handle.backend_id,
                    handle.protocol,
                    handle.internal_uaa_session_id,
                )
            }
            _ => anyhow::bail!("World controller consumed an unsupported continuation family"),
        };
    ensure!(
        handle_session == b.orchestration_session_id
            && &handle_owner == source_actor
            && handle_backend == b.native_selection.backend_id
            && !native_session_id.is_empty(),
        "World consumed continuation is not owned by exact active source"
    );
    let origin = root
        .transition_intent_map
        .get(&b.original_start_intent_id)
        .ok_or_else(|| anyhow::anyhow!("World original Start missing"))?;
    ensure!(
        origin.run_id == b.original_start_run_id
            && origin.orchestration_session_id == b.orchestration_session_id
            && origin.world_binding.as_ref() == Some(&b.world_binding)
            && origin.workspace_binding == b.workspace_binding,
        "World completion original Start differs"
    );
    let transaction = root
        .start_transaction_map
        .values()
        .filter(|transaction| transaction.start_intent_id == b.original_start_intent_id)
        .collect::<Vec<_>>();
    let [transaction] = transaction.as_slice() else {
        anyhow::bail!("World Start transaction absent or ambiguous");
    };
    let start_ref = match &transaction.state {
        StartTransactionStateV1::TurnSettledAwaitingResponse { settlement_ref, .. }
        | StartTransactionStateV1::PublicResponseDelivered { settlement_ref, .. } => settlement_ref,
        _ => anyhow::bail!("World original Start has not settled"),
    };
    ensure!(
        start_ref.object_kind == AuthorityObjectKindV1::ResumeHandle
            && start_ref.schema_version == 3,
        "World completion requires its actual World Start owner artifact"
    );
    let start: StartContinuationHandleHashInputV3 =
        read_world_completion_object(reader, start_ref)?;
    ensure!(
        start.native_selection == b.native_selection
            && start.requested_selection == b.requested_selection
            && start.world_binding == b.world_binding
            && start.continuation.start_run_id == b.original_start_run_id
            && start.continuation.start_intent_id == b.original_start_intent_id,
        "World completion original complete selection differs"
    );
    let descriptor: AgentDescriptorHashInputV1 =
        read_world_completion_object(reader, &b.descriptor_ref)?;
    let attach: HostAttachContractHashInputV1 =
        read_world_completion_object(reader, &intent.host_attach_contract_ref)?;
    let policy: PolicyObjectHashInputV1 = read_world_completion_object(reader, &b.policy_ref)?;
    ensure!(
        descriptor.descriptor.execution_scope == AgentExecutionScopeV1::Host
            && descriptor.descriptor.backend_id == b.native_selection.backend_id
            && descriptor.descriptor.protocol == handle_protocol
            && attach.contract.execution_scope == AgentExecutionScopeV1::Host
            && attach.contract.backend_id == descriptor.descriptor.backend_id
            && attach.contract.protocol == descriptor.descriptor.protocol
            && attach.contract.descriptor_ref == b.descriptor_ref
            && attach.contract.policy_ref == b.policy_ref
            && policy.policy_revision == b.policy_revision
            && latest.authority.host_attach_contract_ref.as_ref()
                == Some(&intent.host_attach_contract_ref),
        "World native descriptor/attach/policy differs"
    );
    let issuer = root
        .successor_issuer_request_index
        .get(&b.issuer_request_id)
        .ok_or_else(|| anyhow::anyhow!("World successor issuer entry missing"))?;
    let journal = root
        .successor_application_journal
        .get(&b.intent_id)
        .ok_or_else(|| anyhow::anyhow!("World successor initial journal missing"))?;
    ensure!(
        issuer.intent_id == b.intent_id
            && issuer.issuer_request_id == b.issuer_request_id
            && issuer.orchestration_session_id == b.orchestration_session_id
            && issuer.payload_commitment == b.payload_commitment
            && journal.initial_application.application_result_ref
                == b.initial_application_result_ref
            && journal.initial_application.authority_record_commitment
                == *authority_record_commitment
            && journal.initial_application.authority_revision_after == *authority_revision_after
            && journal.initial_application.authority_revision_before == *authority_revision_before
            && journal.startup_terminal_application.is_none(),
        "World successor issuer/initial journal differs"
    );
    let HostSessionTransitionInputHandoffV1::Accepted {
        input_ref,
        run_id,
        acceptance_ref,
        accepted_at,
    } = &intent.input_handoff
    else {
        anyhow::bail!("World completion requires exact Accepted input");
    };
    ensure!(
        input_ref == &b.transition_input_ref
            && acceptance_ref == &b.input_acceptance_ref
            && run_id == &b.run_id,
        "World completion Accepted input differs"
    );
    let acceptance: InputAcceptanceHashInputV1 =
        read_world_completion_object(reader, acceptance_ref)?;
    ensure!(
        acceptance.intent_id == b.intent_id
            && acceptance.run_id == b.run_id
            && acceptance.input_ref == b.transition_input_ref
            && acceptance.accepting_participant_id == b.authoritative_participant_id
            && &acceptance.accepted_at == accepted_at,
        "World input acceptance owner artifact differs"
    );
    Ok((descriptor.descriptor, native_session_id))
}

pub(super) fn world_controller_terminal_event_id(
    binding: &WorldControllerResumeBindingV1,
    native: &DirectHostNativeTerminalEvidenceV1,
) -> Result<String> {
    Ok(format!(
        "world-controller-terminal:{}:{}",
        canonical_sha256(binding)?,
        native.raw_event_sha256
    ))
}

fn verify_world_completion_native(
    native: &DirectHostNativeTerminalEvidenceV1,
    kind: &HostPostTurnProtocolEventKindV1,
    descriptor: &AgentDescriptorV1,
    native_session: &str,
) -> Result<()> {
    native.validate()?;
    let sequence_matches = match (&descriptor.backend_kind, &native.sequence) {
        (
            RuntimeBackendKindV1::Codex,
            DirectHostNativeSequenceV1::Codex {
                resumed_thread_id,
                started_turn_id,
            },
        ) => resumed_thread_id == &native.thread_id && started_turn_id == &native.turn_id,
        (
            RuntimeBackendKindV1::ClaudeCode,
            DirectHostNativeSequenceV1::ClaudeCode {
                initialized_session_id,
            },
        ) => initialized_session_id == &native.thread_id && native.turn_id == native.thread_id,
        _ => false,
    };
    let success = matches!(
        native.terminal_event_kind.as_str(),
        "turn.completed" | "result.success"
    );
    let outcome_matches = match kind {
        HostPostTurnProtocolEventKindV1::ResumableClean
        | HostPostTurnProtocolEventKindV1::TerminalClean => success,
        HostPostTurnProtocolEventKindV1::TerminalFailure {
            reason: HostPostTurnTerminalReasonV1::TargetFailedAfterInputAcceptance,
        } => !success,
        _ => false,
    };
    ensure!(
        descriptor.execution_scope == AgentExecutionScopeV1::Host
            && native.backend_id == descriptor.backend_id
            && native.protocol == descriptor.protocol
            && native.native_session_id == native_session
            && native.thread_id == native_session
            && sequence_matches
            && outcome_matches,
        "World native terminal does not join its actual descriptor/session/outcome"
    );
    Ok(())
}

fn verify_world_completion_produced_handle(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    reference: &AuthorityObjectRefV1,
    completion: &PostTurnCompletionHashInputV4,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<()> {
    let Some(handle_ref) = &completion.produced_resume_handle_ref else {
        return Ok(());
    };
    let handle: ResumeHandleHashInputV1 = read_world_completion_object(reader, handle_ref)?;
    ensure!(
        handle.orchestration_session_id == completion.binding.orchestration_session_id
            && handle.participant_id == intent.target_authoritative_participant_id
            && handle.backend_id == completion.native_terminal.backend_id
            && handle.protocol == completion.native_terminal.protocol
            && handle.internal_uaa_session_id == completion.native_terminal.native_session_id,
        "World produced continuation does not belong to current target/native terminal"
    );
    let mut producers = 0;
    for parent in root.successor_transition_intent_map.values() {
        let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &parent.state else {
            continue;
        };
        let HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } = post_turn.as_ref()
        else {
            continue;
        };
        let produced = match completion_ref.schema_version {
            4 => {
                read_world_completion_object::<PostTurnCompletionHashInputV4>(
                    reader,
                    completion_ref,
                )?
                .produced_resume_handle_ref
            }
            3 => {
                read_world_completion_object::<PostTurnCompletionHashInputV3>(
                    reader,
                    completion_ref,
                )?
                .produced_resume_handle_ref
            }
            _ => None,
        };
        if produced.as_ref() == Some(handle_ref) {
            ensure!(
                completion_ref.as_ref() == reference && parent == intent,
                "World continuation has conflicting producing owners"
            );
            producers += 1;
        }
    }
    ensure!(
        producers == 1,
        "World continuation producer absent or ambiguous"
    );
    Ok(())
}

/// Pure history leaf. The caller supplies the exact, already-authenticated
/// authority prefix immediately before this completion; no source reread occurs.
pub(in crate::execution::agent_runtime::host_session_authority) fn verify_world_controller_completion_v4_edge(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    reference: &AuthorityObjectRefV1,
    prefix: &BTreeMap<u64, ResolvedSessionAuthorityV1>,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<Option<AuthorityObjectRefV1>> {
    ensure!(
        reference.object_kind == AuthorityObjectKindV1::PostTurnCompletion
            && reference.schema_version == 4,
        "World history requires completion schema4"
    );
    let c: PostTurnCompletionHashInputV4 = read_world_completion_object(reader, reference)?;
    let (descriptor, native_session_id) =
        verify_world_completion_binding(root, intent, &c.binding, prefix, reader)?;
    let native = &c.native_terminal;
    verify_world_completion_native(native, &c.kind, &descriptor, &native_session_id)?;
    let accepted_at = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::Accepted { accepted_at, .. } => accepted_at,
        _ => anyhow::bail!("World completion native terminal has no Accepted input"),
    };
    ensure!(
        native.backend_id == descriptor.backend_id
            && native.protocol == descriptor.protocol
            && native.native_session_id == native_session_id
            && native.thread_id == native_session_id
            && c.event_id == world_controller_terminal_event_id(&c.binding, native)?
            && c.completed_at.as_str() >= accepted_at.as_str()
            && c.completed_at.as_str()
                >= prefix[&c.binding.authority_revision_observed]
                    .authority
                    .updated_at
                    .as_str(),
        "World native terminal sequence/event/time differs from actual consumed continuation"
    );
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &intent.state else {
        anyhow::bail!("World completion parent not Applied");
    };
    let HostSessionPostTurnApplicationV2::Applied {
        completion_ref,
        obligation_snapshot_ref,
        authority_revision_before,
        authority_revision_after,
        resulting_posture,
        applied_at,
        ..
    } = post_turn.as_ref()
    else {
        anyhow::bail!("World completion post-turn not Applied");
    };
    ensure!(
        completion_ref.as_ref() == reference
            && obligation_snapshot_ref.is_none()
            && *authority_revision_before == c.binding.authority_revision_observed
            && authority_revision_before.checked_add(1) == Some(*authority_revision_after)
            && *resulting_posture == c.resulting_posture()
            && applied_at == &c.completed_at
            && c.binding.intent_revision_observed.checked_add(1) == Some(intent.intent_revision)
            && intent.updated_at == c.completed_at,
        "World completion immutable edge/posture differs"
    );
    verify_world_completion_release(root, intent, reference, &c, reader)?;
    verify_world_completion_produced_handle(root, intent, reference, &c, reader)?;
    Ok(c.produced_resume_handle_ref)
}

fn verify_world_completion_release(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    reference: &AuthorityObjectRefV1,
    completion: &PostTurnCompletionHashInputV4,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<()> {
    let b = &completion.binding;
    let journal = root
        .successor_application_journal
        .get(&b.intent_id)
        .and_then(|journal| journal.post_turn_application.as_ref())
        .ok_or_else(|| anyhow::anyhow!("World completion application journal absent"))?;
    ensure!(
        journal.completion_ref == *reference
            && journal.obligation_snapshot_ref.is_none()
            && journal.authority_revision_before == b.authority_revision_observed
            && b.authority_revision_observed.checked_add(1)
                == Some(journal.authority_revision_after)
            && journal.applied_at == completion.completed_at,
        "World completion journal differs"
    );
    let application: ApplicationResultHashInputV1 =
        read_world_completion_object(reader, &journal.application_result_ref)?;
    ensure!(
        application.intent_id == b.intent_id
            && application.run_id == b.run_id
            && application.mode == HostSessionTransitionModeV1::ResumeOneTurn
            && application.applied_at == completion.completed_at
            && application.phase
                == ApplicationResultPhaseV1::PostTurn {
                    completion_ref: reference.clone(),
                    authority_revision_before: journal.authority_revision_before,
                    authority_revision_after: journal.authority_revision_after,
                    active_authoritative_participant_id: b.authoritative_participant_id.clone(),
                    resulting_posture: completion.resulting_posture(),
                    authority_record_commitment: journal.authority_record_commitment.clone(),
                },
        "World completion application artifact differs"
    );
    let terminal_ref = match &intent.transport_payload_state {
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
            terminal_handoff_ref,
        }
        | HostSessionTransitionTransportPayloadStateV1::Released {
            terminal_handoff_ref,
            ..
        } => terminal_handoff_ref,
        _ => anyhow::bail!("World completion has no physical transport handoff"),
    };
    let terminal: TerminalHandoffHashInputV1 = read_world_completion_object(reader, terminal_ref)?;
    ensure!(
        terminal.intent_id == b.intent_id
            && terminal.run_id == b.run_id
            && terminal.payload_commitment == b.payload_commitment
            && terminal.terminal_state == TerminalHandoffStateV1::Applied
            && terminal.application_result_ref.as_ref() == Some(&b.initial_application_result_ref)
            && terminal.input_acceptance_ref.as_ref() == Some(&b.input_acceptance_ref)
            && terminal.post_turn_completion_ref.as_ref() == Some(reference)
            && terminal.post_turn_application_result_ref.as_ref()
                == Some(&journal.application_result_ref)
            && terminal.recorded_at == completion.completed_at,
        "World completion terminal handoff differs"
    );
    let transport = root
        .object_index
        .get(&intent.transport_payload_ref.ref_id)
        .ok_or_else(|| anyhow::anyhow!("World completion transport index absent"))?;
    let matches = match (&intent.transport_payload_state, &transport.storage_state) {
        (
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref: a,
            },
            AuthorityObjectStorageStateV1::ReleaseEligible {
                terminal_handoff_ref: b,
            },
        ) => a == b,
        (
            HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref: a,
                released_at: at,
            },
            AuthorityObjectStorageStateV1::Released {
                terminal_handoff_ref: b,
                released_at: bt,
            },
        ) => a == b && at == bt,
        _ => false,
    };
    ensure!(
        matches
            && transport.ref_id == intent.transport_payload_ref.ref_id
            && transport.object_kind == AuthorityObjectKindV1::TransitionTransportPayload
            && transport.object_schema_version == intent.transport_payload_ref.schema_version,
        "World completion transport release/index differs"
    );
    Ok(())
}

pub(in crate::execution::agent_runtime::host_session_authority) fn verify_world_controller_completed_join(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
    reference: &AuthorityObjectRefV1,
    reader: &mut impl FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, BootstrapError>,
) -> Result<PostTurnCompletionHashInputV4> {
    let completion: PostTurnCompletionHashInputV4 =
        read_world_completion_object(reader, reference)?;
    let history = exact_v3_authority_history_with_reader(
        root,
        &completion.binding.orchestration_session_id,
        |reference| reader(reference).map_err(|error| error.to_string()),
    )?;
    let prefix = history
        .into_iter()
        .filter(|(revision, _)| *revision <= completion.binding.authority_revision_observed)
        .collect();
    verify_world_controller_completion_v4_edge(root, intent, reference, &prefix, reader)?;
    Ok(completion)
}
