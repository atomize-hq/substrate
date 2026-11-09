//! Non-upgrading authentication of an actual current native Host run.
//!
//! The configured authority, held observation and applied run are separate inputs.
//! No lifecycle operation, reconciliation or external admission classification is
//! performed here. A returned proof must be revalidated immediately before effects.
use anyhow::{anyhow, bail, ensure, Result};

use super::{
    canonical_json,
    facade::{
        exact_v3_authority_history_with_reader, AuthorityObservationV1, HostSessionAuthority,
    },
    schema::*,
    store::{
        self, CurrentHostRunReadViewV1, ObjectVerificationContextV1,
        VersionedObjectVerificationParentIntentV1,
    },
    store_schema::*,
    trusted_fs::TrustedWorkspaceRoot,
};

/// Authentication under one independently opened authority, with retained physical
/// workspace custody. No public fields, construction, serde or transferable proof.
pub(crate) struct AuthenticatedCurrentHostRunV1<'authority> {
    authority: &'authority HostSessionAuthority,
    facts: CurrentHostRunFactsV1,
    workspace: TrustedWorkspaceRoot,
}

#[derive(Eq, PartialEq)]
struct CurrentHostRunFactsV1 {
    observation: AuthorityObservationV1,
    participant_id: String,
    backend_id: String,
    run_id: String,
    mode: HostSessionTransitionModeV1,
    intent_id: String,
    application_result_ref: AuthorityObjectRefV1,
    original_start_intent_id: String,
    original_start_issuer_request_id: String,
    original_start_payload_commitment: AuthorityObjectCommitmentV1,
    original_start_application_result_ref: AuthorityObjectRefV1,
    original_start_run_id: String,
    descriptor_ref: AuthorityObjectRefV1,
    attach_contract_ref: AuthorityObjectRefV1,
    policy_ref: AuthorityObjectRefV1,
    policy_revision: String,
    workspace_binding: WorkspaceBindingV1,
}

impl AuthenticatedCurrentHostRunV1<'_> {
    pub(crate) fn authority_store_id(&self) -> &str {
        &self.facts.observation.authority_store_id
    }

    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.facts.observation.orchestration_session_id
    }

    pub(crate) fn participant_id(&self) -> &str {
        &self.facts.participant_id
    }

    pub(crate) fn backend_id(&self) -> &str {
        &self.facts.backend_id
    }

    /// The authenticated current Start, Attach or ResumeOneTurn run.
    pub(crate) fn run_id(&self) -> &str {
        &self.facts.run_id
    }

    pub(crate) fn transition_mode(&self) -> HostSessionTransitionModeV1 {
        self.facts.mode
    }

    pub(crate) fn intent_id(&self) -> &str {
        &self.facts.intent_id
    }

    pub(crate) fn application_result_ref(&self) -> &AuthorityObjectRefV1 {
        &self.facts.application_result_ref
    }

    pub(crate) fn original_start_intent_id(&self) -> &str {
        &self.facts.original_start_intent_id
    }

    pub(crate) fn original_start_issuer_request_id(&self) -> &str {
        &self.facts.original_start_issuer_request_id
    }

    pub(crate) fn original_start_payload_commitment(&self) -> &AuthorityObjectCommitmentV1 {
        &self.facts.original_start_payload_commitment
    }

    pub(crate) fn original_start_application_result_ref(&self) -> &AuthorityObjectRefV1 {
        &self.facts.original_start_application_result_ref
    }

    /// Birth identity only; never substituted for a successor's run.
    pub(crate) fn original_start_run_id(&self) -> &str {
        &self.facts.original_start_run_id
    }

    /// Reauthenticate the same run/application and exact held observation.
    ///
    /// A new registration can preserve the run while invalidating this proof's
    /// observation. Acquire a new proof with a fresh observation in that case.
    /// This method does not hold a lock across subsequent caller effects.
    pub(crate) fn revalidate(&self) -> Result<()> {
        self.workspace.revalidate().map_err(|e| anyhow!("{e}"))?;
        let fresh = self.authority.verify_current_host_run_v1(
            self.orchestration_session_id(),
            self.participant_id(),
            self.backend_id(),
            self.run_id(),
            &self.facts.observation,
        )?;
        ensure!(
            fresh.facts == self.facts,
            "current Host run application changed"
        );
        self.workspace.revalidate().map_err(|e| anyhow!("{e}"))
    }
}

impl HostSessionAuthority {
    /// Authenticate a claimed native Host orchestrator run under this independently
    /// configured accepted home/principal, without upgrading or repairing the store.
    ///
    /// The observation is mandatory and checked against genuine current authority.
    /// Matching strings alone never establish an Applied transition/application.
    /// This result grants no ordinary/protected admission or process-liveness truth.
    pub(crate) fn verify_current_host_run_v1<'authority>(
        &'authority self,
        orchestration_session_id: &str,
        participant_id: &str,
        backend_id: &str,
        run_id: &str,
        held_observation: &AuthorityObservationV1,
    ) -> Result<AuthenticatedCurrentHostRunV1<'authority>> {
        ensure!(
            [orchestration_session_id, participant_id, backend_id, run_id]
                .iter()
                .all(|value| !value.is_empty()),
            "current Host run claim must be exact and nonempty"
        );
        ensure!(
            held_observation.orchestration_session_id == orchestration_session_id
                && &held_observation.bootstrap_home == self.trusted_root().identity(),
            "current Host run observation is bound to a different session or home"
        );
        // Semantic failure remains a callback value: unconditional snapshot exit
        // verification takes precedence on both success and refusal.
        let (facts, workspace) = store::with_existing_current_host_run_snapshot_opened(
            self.trusted_root(),
            |snapshot| {
                Ok(authenticate_current_host_run(
                    self,
                    snapshot,
                    orchestration_session_id,
                    participant_id,
                    backend_id,
                    run_id,
                    held_observation,
                ))
            },
        )
        .map_err(|e| anyhow!("current Host run snapshot refused: {e:?}"))??;
        workspace.revalidate().map_err(|e| anyhow!("{e}"))?;
        self.trusted_root()
            .revalidate()
            .map_err(|e| anyhow!("{e}"))?;
        Ok(AuthenticatedCurrentHostRunV1 {
            authority: self,
            facts,
            workspace,
        })
    }
}

fn read_run_object<T: serde::de::DeserializeOwned + serde::Serialize>(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    reference: &AuthorityObjectRefV1,
    context: Option<&ObjectVerificationContextV1>,
) -> Result<T> {
    let root = snapshot.captured_root();
    let bytes = snapshot
        .read_typed_object(
            root.authority_store_id(),
            root.root_revision(),
            reference,
            context,
        )
        .map_err(|e| anyhow!("current Host run object refused: {e:?}"))?;
    canonical_json::from_slice(&bytes).map_err(|e| anyhow!("{e}"))
}

#[allow(clippy::too_many_arguments)]
fn authenticate_current_host_run(
    facade: &HostSessionAuthority,
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    session: &str,
    participant: &str,
    backend: &str,
    run: &str,
    held: &AuthorityObservationV1,
) -> Result<(CurrentHostRunFactsV1, TrustedWorkspaceRoot)> {
    let captured = snapshot.captured_root();
    ensure!(
        captured.authority_store_id() == held.authority_store_id
            && captured.root_revision() == held.root_revision,
        "current Host run store or root observation changed"
    );
    let read = |reference: &AuthorityObjectRefV1, context: Option<&ObjectVerificationContextV1>| {
        snapshot
            .read_typed_object(
                captured.authority_store_id(),
                captured.root_revision(),
                reference,
                context,
            )
            .map_err(|e| format!("{e:?}"))
    };
    let (origin_view, current, history) = match captured {
        VersionedStateRoot::V1(_) => bail!("current Host run requires existing V2 or V3"),
        VersionedStateRoot::V2(root) => {
            let resolved = facade
                .resolve_current_exact_from_preserved_start_root_with_reader(
                    root,
                    session,
                    Some(held),
                    &read,
                )
                .map_err(|e| anyhow!("{e}"))?;
            (root.clone(), resolved.authority, None)
        }
        VersionedStateRoot::V3(root) => {
            ensure!(
                !root
                    .fork_successor_allocation_map
                    .values()
                    .any(
                        |allocation| allocation.request.target_orchestration_session_id == session
                    ),
                "current Host run requires a genuine original Start"
            );
            let history = exact_v3_authority_history_with_reader(root, session, |reference| {
                read(reference, None)
            })
            .map_err(|e| anyhow!("{e}"))?;
            let current = history
                .last_key_value()
                .ok_or_else(|| anyhow!("current history absent"))?
                .1;
            ensure!(
                &current.observation() == held,
                "current Host run authority observation changed"
            );
            // Full original maps/history remain intact. Initial application
            // authentication below does not pretend this is current V2 authority.
            (
                root.preserved_v2_view(),
                current.authority.clone(),
                Some(history),
            )
        }
    };
    ensure!(
        current.lifecycle_posture == HostSessionPostureV1::ActiveAttached
            && current.active_authoritative_participant_id.as_deref() == Some(participant)
            && current
                .authoritative_participant_lineage
                .iter()
                .filter(|id| id.as_str() == participant)
                .count()
                == 1,
        "claim is not the current active Host orchestrator"
    );
    ensure!(
        current.workspace_binding.authority_store_id == held.authority_store_id
            && current.workspace_binding.authority_store_root == held.bootstrap_home,
        "current Host run workspace authority binding changed"
    );
    let DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } = &current.origin;
    let original = origin_view
        .transition_intent_map
        .get(intent_id)
        .ok_or_else(|| anyhow!("current Host run has no original Start"))?
        .clone();
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref: original_application,
        claim_id: original_claim,
        claimant_attempt_id: original_attempt,
        authority_revision_after: original_revision,
        startup_ownership,
        ..
    } = &original.state
    else {
        bail!("original Start is not Applied")
    };
    // Authenticate actual persisted ownership before immutable application checks.
    // No original ownership phase is projected or replaced.
    verify_run_startup(
        snapshot,
        &original.intent_id,
        &original.run_id,
        &original.target_authoritative_participant_id,
        original_application,
        session,
        *original_revision,
        original_claim,
        original_attempt,
        startup_ownership,
    )?;
    // R02-F02: a V2 current-descendant validator cannot describe an initial
    // application while retaining later registration journals. Authenticate the
    // original immutable objects directly after full real history and ownership.
    if let Some(history) = &history {
        let initial = history
            .get(&1)
            .ok_or_else(|| anyhow!("original Start authority absent"))?;
        verify_original_start_application_v3_with_reader(
            &original,
            &initial.authority,
            &initial.authority_record_commitment,
            &mut |reference, context| read(reference, context).map_err(|error| anyhow!("{error}")),
        )?;
    }
    let mut facts = CurrentHostRunFactsV1 {
        observation: held.clone(),
        participant_id: participant.to_owned(),
        backend_id: backend.to_owned(),
        run_id: original.run_id.clone(),
        mode: HostSessionTransitionModeV1::Start,
        intent_id: original.intent_id.clone(),
        application_result_ref: original_application.clone(),
        original_start_intent_id: original.intent_id.clone(),
        original_start_issuer_request_id: original.issuer_request_id.clone(),
        original_start_payload_commitment: original.payload_commitment.clone(),
        original_start_application_result_ref: original_application.clone(),
        original_start_run_id: original.run_id.clone(),
        descriptor_ref: original.descriptor_ref.clone(),
        attach_contract_ref: original.host_attach_contract_ref.clone(),
        policy_ref: current
            .current_policy_ref
            .clone()
            .ok_or_else(|| anyhow!("current policy absent"))?,
        policy_revision: current
            .current_policy_revision
            .clone()
            .ok_or_else(|| anyhow!("current policy revision absent"))?,
        workspace_binding: current.workspace_binding.clone(),
    };
    let successors = match captured {
        VersionedStateRoot::V3(root) => root
            .successor_transition_intent_map
            .values()
            .filter(|intent| {
                intent.orchestration_session_id == session
                    && intent.target_authoritative_participant_id == participant
                    && matches!(
                        intent.state,
                        HostSessionTransitionIntentStateV3::Applied { .. }
                    )
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    match (
        original.target_authoritative_participant_id == participant,
        successors.as_slice(),
    ) {
        (true, []) => {
            ensure!(
                original.run_id == run,
                "claimed run is not the current Start run"
            );
        }
        (false, [successor]) => {
            let history = history
                .as_ref()
                .ok_or_else(|| anyhow!("successor history absent"))?;
            verify_current_successor(snapshot, successor, &current, history, &mut facts)?;
            ensure!(
                facts.run_id == run,
                "claimed run is not the current successor run"
            );
        }
        _ => bail!("current Host run application is absent or ambiguous"),
    }
    let descriptor: AgentDescriptorHashInputV1 =
        read_run_object(snapshot, &facts.descriptor_ref, None)?;
    let attach: HostAttachContractHashInputV1 =
        read_run_object(snapshot, &facts.attach_contract_ref, None)?;
    let policy: PolicyObjectHashInputV1 = read_run_object(snapshot, &facts.policy_ref, None)?;
    ensure!(
        descriptor.schema_version == 1
            && descriptor.descriptor.schema_version == 1
            && attach.schema_version == 1
            && attach.contract.schema_version == 1
            && policy.schema_version == 1
            && descriptor.descriptor.execution_scope == AgentExecutionScopeV1::Host
            && attach.contract.execution_scope == AgentExecutionScopeV1::Host
            && descriptor.descriptor.backend_id == backend
            && attach.contract.backend_id == backend
            && descriptor.descriptor.protocol == attach.contract.protocol
            && attach.contract.descriptor_ref == facts.descriptor_ref
            && attach.contract.policy_ref == facts.policy_ref
            && policy.policy_revision == facts.policy_revision
            && current.host_attach_contract_ref.as_ref() == Some(&facts.attach_contract_ref),
        "current Host descriptor, role, backend, attach contract or policy mismatch"
    );
    ensure!(
        match facts.mode {
            HostSessionTransitionModeV1::Start => true,
            HostSessionTransitionModeV1::Attach => attach.contract.capabilities.session_resume,
            HostSessionTransitionModeV1::ResumeOneTurn =>
                attach.contract.capabilities.session_resume,
        },
        "current Host contract does not permit continuity"
    );
    let workspace = TrustedWorkspaceRoot::open_exact(&facts.workspace_binding.workspace_root)
        .map_err(|e| anyhow!("{e}"))?;
    workspace.revalidate().map_err(|e| anyhow!("{e}"))?;
    Ok((facts, workspace))
}

fn verify_original_start_application_v3_with_reader<F>(
    intent: &HostSessionTransitionIntentV2,
    initial: &DurableSessionAuthorityV1,
    initial_commitment: &AuthorityObjectCommitmentV1,
    read: &mut F,
) -> Result<()>
where
    F: FnMut(&AuthorityObjectRefV1, Option<&ObjectVerificationContextV1>) -> Result<Vec<u8>>,
{
    // Same immutable object, parent-context and payload checks as the existing
    // Start validator. No mutable V2 current-descendant claim is made here.
    verify_original_start_objects_v3_with_reader(intent, read)?;
    let HostSessionTransitionIntentStateV2::Applied {
        claim_id,
        claimant_attempt_id,
        authority_revision_before,
        authority_revision_after,
        active_authoritative_participant_id,
        resulting_posture,
        authority_record_commitment,
        application_result_ref,
        post_turn,
        applied_at,
        ..
    } = &intent.state
    else {
        bail!("original Start is not Applied")
    };
    ensure!(
        intent.mode == HostSessionTransitionModeV1::Start
            && intent.authority_precondition == HostSessionAuthorityPreconditionV1::ExpectedAbsent
            && intent.source_authoritative_participant_id.is_none()
            && intent.resume_handle_ref.is_none()
            && intent.post_turn_disposition.is_none()
            && !claim_id.is_empty()
            && !claimant_attempt_id.is_empty()
            && intent.intent_revision >= 2
            && authority_revision_before.is_none()
            && *authority_revision_after == 1
            && active_authoritative_participant_id == &intent.target_authoritative_participant_id
            && *resulting_posture == HostSessionPostureV1::ActiveAttached
            && authority_record_commitment == initial_commitment
            && post_turn.as_ref() == &HostSessionPostTurnApplicationV1::NotApplicable,
        "original Start application identity conflicts"
    );
    let application: ApplicationResultHashInputV1 =
        canonical_json::from_slice(&read(application_result_ref, None)?)
            .map_err(|error| anyhow!("{error}"))?;
    ensure!(
        application
            == ApplicationResultHashInputV1 {
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
            },
        "original Start immutable application object conflicts"
    );
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&read(&intent.host_attach_contract_ref, None)?)
            .map_err(|error| anyhow!("{error}"))?;
    let policy: PolicyObjectHashInputV1 =
        canonical_json::from_slice(&read(&attach.contract.policy_ref, None)?)
            .map_err(|error| anyhow!("{error}"))?;
    ensure!(
        initial.schema_version == 1
            && initial.orchestration_session_id == intent.orchestration_session_id
            && initial.shell_trace_session_id == intent.shell_trace_session_id
            && initial.authority_revision == 1
            && initial.origin
                == DurableSessionAuthorityOriginV1::StartIntent {
                    intent_id: intent.intent_id.clone(),
                    issuer_request_id: intent.issuer_request_id.clone(),
                    payload_commitment: intent.payload_commitment.clone()
                }
            && initial.authoritative_participant_lineage == intent.resulting_authoritative_lineage
            && initial.active_authoritative_participant_id.as_ref()
                == Some(&intent.target_authoritative_participant_id)
            && initial.workspace_binding == intent.workspace_binding
            && initial.world_binding == intent.world_binding
            && initial.host_attach_contract_ref.as_ref() == Some(&intent.host_attach_contract_ref)
            && initial.retained_worker_refs.is_empty()
            && initial.internal_resume_handle_refs.is_empty()
            && initial.lifecycle_posture == HostSessionPostureV1::ActiveAttached
            && initial.current_policy_ref.as_ref() == Some(&attach.contract.policy_ref)
            && initial.current_policy_revision.as_ref() == Some(&policy.policy_revision)
            && initial.updated_at == *applied_at,
        "original Start initial authority joins conflict"
    );
    Ok(())
}

fn verify_original_start_objects_v3_with_reader<F>(
    intent: &HostSessionTransitionIntentV2,
    read_object: &mut F,
) -> Result<()>
where
    F: FnMut(&AuthorityObjectRefV1, Option<&ObjectVerificationContextV1>) -> Result<Vec<u8>>,
{
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    let descriptor_bytes = read_object(&intent.descriptor_ref, None)?;
    let descriptor: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(&descriptor_bytes).map_err(|error| anyhow!("{error}"))?;
    let attach_bytes = read_object(&intent.host_attach_contract_ref, None)?;
    let attach: HostAttachContractHashInputV1 =
        canonical_json::from_slice(&attach_bytes).map_err(|error| anyhow!("{error}"))?;
    let policy_bytes = read_object(&attach.contract.policy_ref, None)?;
    let policy: PolicyObjectHashInputV1 =
        canonical_json::from_slice(&policy_bytes).map_err(|error| anyhow!("{error}"))?;
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
        return Err(anyhow!(
            "Start descriptor, attach contract, or policy projection is inconsistent",
        ));
    }
    read_object(&intent.target_participant_lease_token_ref, Some(&context))?;
    if let Some(reference) = intent.transition_input_ref.as_ref() {
        read_object(reference, Some(&context))?;
    }
    let parent = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V2(Box::new(
            intent.clone(),
        ))),
    };
    let transport_bytes = read_object(&intent.transport_payload_ref, Some(&parent))?;
    let transport: TransitionTransportPayloadObjectV1 =
        canonical_json::from_slice(&transport_bytes).map_err(|error| anyhow!("{error}"))?;
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
        return Err(anyhow!(
            "Start transport payload projection is inconsistent"
        ));
    }
    let expected_payload = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: super::hash::canonical_sha256(&original_start_payload(intent))
            .map_err(|error| anyhow!("{error}"))?,
    };
    if expected_payload != intent.payload_commitment {
        return Err(anyhow!("transition payload commitment does not verify"));
    }
    Ok(())
}

fn original_start_payload(
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

#[allow(clippy::too_many_arguments)]
fn verify_run_startup(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    intent: &str,
    run: &str,
    participant: &str,
    application: &AuthorityObjectRefV1,
    session: &str,
    revision: u64,
    claim: &str,
    attempt: &str,
    startup: &HostSessionStartupOwnershipApplicationV1,
) -> Result<()> {
    match startup {
        HostSessionStartupOwnershipApplicationV1::Pending {
            expected_run_id,
            expected_active_authoritative_participant_id,
            expected_authority_revision,
        } => {
            ensure!(
                expected_run_id == run
                    && expected_active_authoritative_participant_id == participant
                    && *expected_authority_revision == revision,
                "current Host startup ownership mismatch"
            );
        }
        HostSessionStartupOwnershipApplicationV1::Accepted {
            evidence_id,
            result_ref,
            authority_revision,
            accepted_at,
        } => {
            let result: StartupOwnershipResultHashInputV1 =
                read_run_object(snapshot, result_ref, None)?;
            let e = &result.evidence;
            ensure!(
                result.schema_version == 1
                    && e.schema_version == 1
                    && e.evidence_id == *evidence_id
                    && e.orchestration_session_id == session
                    && e.claim_id == claim
                    && e.claimant_attempt_id == attempt
                    && e.intent_id == intent
                    && e.run_id == run
                    && e.active_authoritative_participant_id == participant
                    && &e.application_result_ref == application
                    && e.authority_store_id == snapshot.captured_root().authority_store_id()
                    && e.expected_authority_revision == *authority_revision
                    && *authority_revision == revision
                    && matches!(&e.protocol_actor, HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant { participant_id } if participant_id == participant)
                    && matches!(&e.protocol_event, HostStartupOwnershipProtocolEventV1::OwnershipAccepted { ownership_acknowledgement_id } if ownership_acknowledgement_id == evidence_id)
                    && result.outcome == StartupOwnershipOutcomeV1::Accepted
                    && result.resolved_at == *accepted_at,
                "current Host accepted startup application mismatch"
            );
        }
        _ => bail!("current Host startup is terminal or unsupported"),
    }
    Ok(())
}

fn verify_current_successor(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    intent: &HostSessionTransitionIntentV3,
    current: &DurableSessionAuthorityV1,
    history: &std::collections::BTreeMap<u64, super::facade::ResolvedSessionAuthorityV1>,
    facts: &mut CurrentHostRunFactsV1,
) -> Result<()> {
    let HostSessionTransitionIntentStateV3::Applied {
        claim_id,
        claimant_attempt_id,
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
    } = &intent.state
    else {
        bail!("current successor is not Applied")
    };
    let applied = history
        .get(authority_revision_after)
        .ok_or_else(|| anyhow!("successor application authority absent"))?;
    ensure!(
        applied.authority_record_commitment == *authority_record_commitment
            && applied
                .authority
                .active_authoritative_participant_id
                .as_ref()
                == Some(&intent.target_authoritative_participant_id)
            && active_authoritative_participant_id == &intent.target_authoritative_participant_id
            && *resulting_posture == HostSessionPostureV1::ActiveAttached
            && intent.workspace_binding == current.workspace_binding
            && intent.world_binding == current.world_binding,
        "current successor application, actor or placement mismatch"
    );
    let VersionedStateRoot::V3(root) = snapshot.captured_root() else {
        bail!("successor snapshot is not V3")
    };
    let journal = root
        .successor_application_journal
        .get(&intent.intent_id)
        .ok_or_else(|| anyhow!("successor application journal absent"))?;
    ensure!(
        journal.initial_application.application_result_ref == *application_result_ref,
        "successor application journal ref mismatch"
    );
    let expected = ApplicationResultHashInputV1 {
        schema_version: 1,
        intent_id: intent.intent_id.clone(),
        mode: intent.mode,
        run_id: intent.run_id.clone(),
        phase: ApplicationResultPhaseV1::InitialTransition {
            authority_revision_before: *authority_revision_before,
            authority_revision_after: *authority_revision_after,
            active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
            resulting_posture: *resulting_posture,
            authority_record_commitment: authority_record_commitment.clone(),
            post_turn_pending_run_id: (intent.mode == HostSessionTransitionModeV1::ResumeOneTurn)
                .then(|| intent.run_id.clone()),
        },
        applied_at: applied_at.clone(),
    };
    ensure!(
        read_run_object::<ApplicationResultHashInputV1>(snapshot, application_result_ref, None)?
            == expected,
        "current successor application object mismatch"
    );
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    let root = snapshot.captured_root();
    for reference in std::iter::once(&intent.target_participant_lease_token_ref)
        .chain(intent.transition_input_ref.iter())
    {
        snapshot
            .read_typed_object(
                root.authority_store_id(),
                root.root_revision(),
                reference,
                Some(&context),
            )
            .map_err(|e| anyhow!("current Host lease/input refused: {e:?}"))?;
    }
    ensure!(
        matches!(
            intent.transport_payload_state,
            HostSessionTransitionTransportPayloadStateV1::Retained
        ),
        "active successor transport is unavailable"
    );
    let transport_context = ObjectVerificationContextV1 {
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V3(Box::new(
            intent.clone(),
        ))),
        ..context
    };
    snapshot
        .read_typed_object(
            root.authority_store_id(),
            root.root_revision(),
            &intent.transport_payload_ref,
            Some(&transport_context),
        )
        .map_err(|e| anyhow!("current Host immutable transport refused: {e:?}"))?;
    match intent.mode {
        HostSessionTransitionModeV1::Attach => {
            ensure!(
                matches!(
                    post_turn.as_ref(),
                    HostSessionPostTurnApplicationV2::NotApplicable
                ) && intent.transition_input_ref.is_none(),
                "Attach contains post-turn input"
            );
            verify_run_startup(
                snapshot,
                &intent.intent_id,
                &intent.run_id,
                &intent.target_authoritative_participant_id,
                application_result_ref,
                &intent.orchestration_session_id,
                *authority_revision_after,
                claim_id,
                claimant_attempt_id,
                startup_ownership,
            )?;
        }
        HostSessionTransitionModeV1::ResumeOneTurn => {
            ensure!(
                matches!(
                    startup_ownership.as_ref(),
                    HostSessionStartupOwnershipApplicationV1::NotApplicable
                ) && matches!(post_turn.as_ref(), HostSessionPostTurnApplicationV2::Pending { expected_run_id, expected_authority_revision }
                    if expected_run_id == &intent.run_id && expected_authority_revision == authority_revision_after),
                "Resume is completed or has incompatible current observation"
            );
            let input_ref = intent
                .transition_input_ref
                .as_ref()
                .ok_or_else(|| anyhow!("Resume input absent"))?;
            match &intent.input_handoff {
                HostSessionTransitionInputHandoffV1::Pending {
                    input_ref: actual,
                    run_id,
                } if actual == input_ref && run_id == &intent.run_id => {}
                HostSessionTransitionInputHandoffV1::Accepted {
                    input_ref: actual,
                    run_id,
                    acceptance_ref,
                    accepted_at,
                } if actual == input_ref && run_id == &intent.run_id => {
                    let expected = InputAcceptanceHashInputV1 {
                        schema_version: 1,
                        intent_id: intent.intent_id.clone(),
                        run_id: intent.run_id.clone(),
                        input_ref: input_ref.clone(),
                        accepting_participant_id: intent
                            .target_authoritative_participant_id
                            .clone(),
                        accepted_at: accepted_at.clone(),
                    };
                    ensure!(
                        read_run_object::<InputAcceptanceHashInputV1>(
                            snapshot,
                            acceptance_ref,
                            None
                        )? == expected,
                        "Resume input acceptance application mismatch"
                    );
                }
                _ => bail!("Resume input is terminal or substituted"),
            }
            let source = authority_revision_before
                .and_then(|r| history.get(&r))
                .ok_or_else(|| anyhow!("Resume source authority absent"))?;
            let reference = intent
                .resume_handle_ref
                .as_ref()
                .ok_or_else(|| anyhow!("Resume consumed handle absent"))?;
            let descriptor: AgentDescriptorHashInputV1 =
                read_run_object(snapshot, &intent.descriptor_ref, None)?;
            ensure!(
                intent.source_authoritative_participant_id.as_ref()
                    == source
                        .authority
                        .active_authoritative_participant_id
                        .as_ref(),
                "Resume source participant mismatches exact authority"
            );
            verify_consumed_host_handle(
                snapshot,
                reference,
                &source.authority,
                &descriptor.descriptor,
            )?;
        }
        HostSessionTransitionModeV1::Start => bail!("successor cannot be Start"),
    }
    facts.run_id = intent.run_id.clone();
    facts.mode = intent.mode;
    facts.intent_id = intent.intent_id.clone();
    facts.application_result_ref = application_result_ref.clone();
    facts.descriptor_ref = intent.descriptor_ref.clone();
    facts.attach_contract_ref = intent.host_attach_contract_ref.clone();
    Ok(())
}

fn verify_consumed_host_handle(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    reference: &AuthorityObjectRefV1,
    source: &DurableSessionAuthorityV1,
    descriptor: &AgentDescriptorV1,
) -> Result<()> {
    let owner = source
        .active_authoritative_participant_id
        .as_ref()
        .ok_or_else(|| anyhow!("Resume source has no active owner"))?;
    ensure!(
        reference.object_kind == AuthorityObjectKindV1::ResumeHandle
            && source
                .internal_resume_handle_refs
                .iter()
                .filter(|r| *r == reference)
                .count()
                == 1,
        "Resume handle is not uniquely owned by source authority"
    );
    match reference.schema_version {
        1 => {
            let h: ResumeHandleHashInputV1 = read_run_object(snapshot, reference, None)?;
            ensure!(
                h.orchestration_session_id == source.orchestration_session_id
                    && &h.participant_id == owner
                    && h.backend_id == descriptor.backend_id
                    && h.protocol == descriptor.protocol
                    && !h.internal_uaa_session_id.is_empty(),
                "Resume handle does not belong to the exact active source"
            );
            let VersionedStateRoot::V3(root) = snapshot.captured_root() else {
                bail!("produced Host handle requires V3")
            };
            // Exact history has already authenticated each V3 completion edge and
            // its producer/native join. Select that exact source's produced handle;
            // an old lineage member's valid handle cannot authorize this source.
            let mut producers = 0;
            for parent in root
                .successor_transition_intent_map
                .values()
                .filter(|parent| {
                    parent.orchestration_session_id == source.orchestration_session_id
                        && &parent.target_authoritative_participant_id == owner
                })
            {
                let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &parent.state
                else {
                    continue;
                };
                let HostSessionPostTurnApplicationV2::Applied {
                    completion_ref,
                    authority_revision_after,
                    ..
                } = post_turn.as_ref()
                else {
                    continue;
                };
                if completion_ref.schema_version != 3 {
                    continue;
                }
                let completion: PostTurnCompletionHashInputV3 =
                    read_run_object(snapshot, completion_ref, None)?;
                if completion.produced_resume_handle_ref.as_ref() != Some(reference) {
                    continue;
                }
                let event: PostTurnProtocolEventHashInputV2 =
                    read_run_object(snapshot, &completion.protocol_event_ref, None)?;
                let native = event
                    .native_terminal
                    .as_ref()
                    .ok_or_else(|| anyhow!("source handle native terminal absent"))?;
                ensure!(
                    *authority_revision_after <= source.authority_revision
                        && completion.binding.authoritative_participant_id == *owner
                        && completion.binding.intent_id == parent.intent_id
                        && completion.binding.run_id == parent.run_id
                        && h.internal_uaa_session_id == native.native_session_id
                        && h.backend_id == native.backend_id
                        && h.protocol == native.protocol,
                    "source handle producer or native identity mismatch"
                );
                producers += 1;
            }
            ensure!(producers == 1, "source handle producer absent or ambiguous");
        }
        2 => {
            let h: StartContinuationHandleHashInputV2 = read_run_object(snapshot, reference, None)?;
            let StartContinuationHandleStateV2::Settled {
                registered_resume_handle_ref,
                thread_id,
                completion_kind: StartTurnCompletionKindV1::ResumableClean,
                ..
            } = &h.state
            else {
                bail!("Resume requires a settled resumable Start handle")
            };
            let registered: StartContinuationHandleHashInputV2 =
                read_run_object(snapshot, registered_resume_handle_ref, None)?;
            ensure!(
                h.authority_store_id == snapshot.captured_root().authority_store_id()
                    && h.orchestration_session_id == source.orchestration_session_id
                    && &h.participant_id == owner
                    && h.backend_id == descriptor.backend_id
                    && h.protocol == descriptor.protocol
                    && !h.internal_uaa_session_id.is_empty()
                    && thread_id == &h.internal_uaa_session_id
                    && registered.internal_uaa_session_id == h.internal_uaa_session_id
                    && registered.participant_id == h.participant_id
                    && registered.backend_id == h.backend_id
                    && registered.protocol == h.protocol
                    && registered.orchestration_session_id == h.orchestration_session_id
                    && matches!(
                        registered.state,
                        StartContinuationHandleStateV2::Registered { .. }
                    ),
                "Start handle does not join exact source owner and native registration"
            );
        }
        _ => bail!("Resume handle schema unsupported"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_runtime::host_session_authority::{
        facade::{ResolvedCurrentAuthorityV1, RetainedWorkerAuthorityPreconditionV1},
        start_continuity::{
            EstablishStartContinuationRequestV1, SettleStartTurnRequestV1,
            StartContinuationOutcomeV1, StartTransactionBeginOutcomeV1, StartTurnCompletionKindV1,
            StartTurnSettlementOutcomeV1,
        },
        transition::*,
    };
    use crate::execution::agent_runtime::{
        control::{
            AuthorityManagedSuccessorLaunchPlanV1, HiddenOwnerHelperLaunchPlan,
            HiddenOwnerHelperParticipantPlan, HiddenOwnerHelperSessionPlan,
            HiddenOwnerHelperStartupPromptPlan, OwnerHelperMode, ResolvedRuntimeBackendKind,
            ResolvedRuntimeDescriptor,
        },
        retained_worker_runtime::{RetainedWorkerRegistrationPlanV1, RetainedWorkerRuntime},
    };
    use crate::execution::config_model::AgentExecutionScope;
    use std::{
        collections::BTreeMap,
        fs,
        os::{
            fd::AsRawFd,
            unix::fs::{MetadataExt, PermissionsExt},
        },
        path::{Path, PathBuf},
    };
    fn timestamp(value: &str) -> TimestampV1 {
        TimestampV1::parse(value).unwrap()
    }

    fn authority() -> (tempfile::TempDir, HostSessionAuthority, WorkspaceBindingV1) {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        #[cfg(unix)]
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        #[cfg(unix)]
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        let authority = HostSessionAuthority::open(&home).unwrap();
        let root = authority.bootstrap().unwrap();
        let binding = WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home.clone(),
            authority_store_id: root.authority_store_id,
        };
        (parent, authority, binding)
    }

    fn start_request(binding: WorkspaceBindingV1) -> IssueHostSessionTransitionRequestV1 {
        IssueHostSessionTransitionRequestV1 {
            intent_id: "intent-start-1".into(),
            issuer_request_id: "request-start-1".into(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: "session-start-1".into(),
            shell_trace_session_id: "trace-start-1".into(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: "participant-start-1".into(),
            target_participant_lease_token: b"lease-start-1".to_vec(),
            run_id: "run-start-1".into(),
            resulting_authoritative_lineage: vec!["participant-start-1".into()],
            workspace_binding: binding,
            world_binding: None,
            start_contract: StartContractMaterialV1 {
                descriptor: AgentDescriptorV1 {
                    schema_version: 1,
                    agent_id: "codex".into(),
                    backend_id: "cli:codex".into(),
                    backend_kind: RuntimeBackendKindV1::Codex,
                    protocol: "substrate.agent.session".into(),
                    execution_scope: AgentExecutionScopeV1::Host,
                    binary_path: "/usr/bin/codex".into(),
                },
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: "policy-1".into(),
                    canonical_policy_snapshot_sha256:
                        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                },
                capabilities: HostAttachCapabilitiesV1 {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                launch_knobs: HostAttachLaunchKnobsV1 {
                    requested_execution_scope: AgentExecutionScopeV1::Host,
                    host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityPreferred,
                },
            },
            transition_input: None,
        }
    }

    fn issue_and_claim_start(
        authority: &HostSessionAuthority,
        request: &IssueHostSessionTransitionRequestV1,
    ) -> ApplyHostSessionTransitionRequestV1 {
        let TransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_start_at(request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
            .unwrap()
        else {
            panic!("first issuance must commit")
        };
        let claim = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: "claim-apply-1".into(),
            claimant_attempt_id: "attempt-apply-1".into(),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(&claim, timestamp("2026-07-14T12:01:00.000000000Z"), 30)
            .unwrap()
        else {
            panic!("first claim must commit")
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            panic!("claimed outcome must retain a claim")
        };
        ApplyHostSessionTransitionRequestV1 {
            intent_id: claimed.intent_id,
            issuer_request_id: claimed.issuer_request_id,
            payload_commitment: claimed.payload_commitment,
            expected_intent_revision: claimed.intent_revision,
            claim_id: claim.claim_id,
            expected_claim_revision: claim_revision,
        }
    }

    fn prepare_start_continuity_transaction(
        authority: &HostSessionAuthority,
        request: &IssueHostSessionTransitionRequestV1,
        intent: &HostSessionTransitionIntentV2,
    ) -> EstablishStartContinuationRequestV1 {
        let HostSessionTransitionIntentStateV2::Applied {
            application_result_ref,
            authority_revision_after,
            ..
        } = &intent.state
        else {
            panic!("Start must remain applied")
        };
        let root = authority.read_a12a_root().unwrap();
        let registration = EstablishStartContinuationRequestV1 {
            start_transaction_id: "stx_start_1".into(),
            request_key_sha256: "c".repeat(64),
            authority_store_id: root.authority_store_id,
            orchestration_session_id: request.orchestration_session_id.clone(),
            intent_id: request.intent_id.clone(),
            issuer_request_id: request.issuer_request_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            application_result_ref: application_result_ref.clone(),
            run_id: request.run_id.clone(),
            expected_authority_revision: *authority_revision_after,
            authoritative_participant_id: request.target_authoritative_participant_id.clone(),
            backend_id: request.start_contract.descriptor.backend_id.clone(),
            protocol: request.start_contract.descriptor.protocol.clone(),
            exchange_id: "exchange-start-1".into(),
            exchange_sequence: 1,
            provider_event_kind: "turn.exchange_opened".into(),
            exchange_evidence_sha256: "a".repeat(64),
            internal_uaa_session_id: "uaa-session-start-1".into(),
            observed_at: timestamp("2026-07-14T12:01:20.000000000Z"),
        };
        let transaction = StartTransactionRecordV1 {
            schema_version: 1,
            transaction_id: registration.start_transaction_id.clone(),
            request_key_sha256: registration.request_key_sha256.clone(),
            prompt_sha256: "d".repeat(64),
            authority_store_id: registration.authority_store_id.clone(),
            orchestration_session_id: registration.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            authoritative_participant_id: registration.authoritative_participant_id.clone(),
            backend_id: registration.backend_id.clone(),
            protocol: registration.protocol.clone(),
            workspace_root: request
                .workspace_binding
                .workspace_root
                .physical_path
                .clone(),
            world_id: request
                .world_binding
                .as_ref()
                .map(|binding| binding.world_id.clone()),
            world_generation: request
                .world_binding
                .as_ref()
                .map(|binding| binding.world_generation),
            public_backend_id: registration.backend_id.clone(),
            public_scope: if request.world_binding.is_some() {
                "world".into()
            } else {
                "host".into()
            },
            start_intent_id: registration.intent_id.clone(),
            start_issuer_request_id: registration.issuer_request_id.clone(),
            start_payload_commitment: registration.payload_commitment.clone(),
            start_application_result_ref: registration.application_result_ref.clone(),
            start_run_id: registration.run_id.clone(),
            start_authority_revision: registration.expected_authority_revision,
            created_at: timestamp("2026-07-14T12:01:10.000000000Z"),
            updated_at: timestamp("2026-07-14T12:01:10.000000000Z"),
            state: StartTransactionStateV1::PromptNotSubmitted,
        };
        assert!(matches!(
            authority.begin_start_transaction(&transaction).unwrap(),
            StartTransactionBeginOutcomeV1::Applied(_)
        ));
        // Direct-host inbox enrollment is only the no-world producer route.
        // A Host actor with optional World placement still has a genuine public
        // Start transaction, without being relabeled as direct-host ingress.
        if request.world_binding.is_none() {
            authority
                .enroll_direct_host_start(
                    &authority
                        .resolve_current_exact(&request.orchestration_session_id, None)
                        .unwrap(),
                )
                .unwrap();
        }
        authority
            .mark_start_prompt_submission_no_replay_barrier(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                timestamp("2026-07-14T12:01:15.000000000Z"),
            )
            .unwrap();
        registration
    }

    fn clean_start_settlement_request(
        registration: &EstablishStartContinuationRequestV1,
        resume_handle_ref: AuthorityObjectRefV1,
        expected_authority_revision: u64,
    ) -> SettleStartTurnRequestV1 {
        SettleStartTurnRequestV1 {
            start_transaction_id: registration.start_transaction_id.clone(),
            request_key_sha256: registration.request_key_sha256.clone(),
            authority_store_id: registration.authority_store_id.clone(),
            orchestration_session_id: registration.orchestration_session_id.clone(),
            intent_id: registration.intent_id.clone(),
            issuer_request_id: registration.issuer_request_id.clone(),
            payload_commitment: registration.payload_commitment.clone(),
            application_result_ref: registration.application_result_ref.clone(),
            run_id: registration.run_id.clone(),
            authoritative_participant_id: registration.authoritative_participant_id.clone(),
            backend_id: registration.backend_id.clone(),
            protocol: registration.protocol.clone(),
            registered_resume_handle_ref: resume_handle_ref,
            expected_authority_revision,
            protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: registration.authoritative_participant_id.clone(),
            },
            event_id: "event-start-completed-1".into(),
            event_sequence: 2,
            provider_event_kind: "turn.completed".into(),
            thread_id: registration.internal_uaa_session_id.clone(),
            turn_id: "turn-start-1".into(),
            completion_evidence_sha256: "b".repeat(64),
            kind: StartTurnCompletionKindV1::ResumableClean,
            obligation_ledger_read: None,
            completed_at: timestamp("2026-07-14T12:01:30.000000000Z"),
        }
    }

    fn attach_successor_request(
        current: &ResolvedCurrentAuthorityV1,
        target_participant_id: &str,
    ) -> IssueSuccessorTransitionRequestV1 {
        let mut lineage = current.authority.authoritative_participant_lineage.clone();
        lineage.push(target_participant_id.to_string());
        IssueSuccessorTransitionRequestV1 {
            intent_id: "intent-attach-1".into(),
            issuer_request_id: "request-attach-1".into(),
            mode: HostSessionTransitionModeV1::Attach,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision: current.observation.authority_revision,
                authority_record_commitment: current
                    .observation
                    .authority_record_commitment
                    .clone(),
                active_authoritative_participant_id: current
                    .authority
                    .active_authoritative_participant_id
                    .clone()
                    .unwrap(),
                authoritative_lineage_commitment: current
                    .observation
                    .authoritative_lineage_commitment
                    .clone(),
                lifecycle_posture: current.authority.lifecycle_posture,
            },
            orchestration_session_id: current.authority.orchestration_session_id.clone(),
            shell_trace_session_id: current.authority.shell_trace_session_id.clone(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::Repl,
                caller_participant_id: current
                    .authority
                    .active_authoritative_participant_id
                    .clone(),
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: current
                .authority
                .active_authoritative_participant_id
                .clone(),
            target_authoritative_participant_id: target_participant_id.into(),
            target_participant_lease_token: b"lease-attach-1".to_vec(),
            run_id: "run-attach-1".into(),
            resulting_authoritative_lineage: lineage,
            workspace_binding: current.authority.workspace_binding.clone(),
            world_binding: current.authority.world_binding.clone(),
            resume_handle_ref: None,
            transition_input: None,
            post_turn_disposition: None,
        }
    }

    fn resume_successor_request(
        current: &ResolvedCurrentAuthorityV1,
        resume_handle_ref: AuthorityObjectRefV1,
        target_participant_id: &str,
    ) -> IssueSuccessorTransitionRequestV1 {
        let mut lineage = current.authority.authoritative_participant_lineage.clone();
        lineage.push(target_participant_id.to_string());
        IssueSuccessorTransitionRequestV1 {
            intent_id: "intent-resume-1".into(),
            issuer_request_id: "request-resume-1".into(),
            mode: HostSessionTransitionModeV1::ResumeOneTurn,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision: current.observation.authority_revision,
                authority_record_commitment: current
                    .observation
                    .authority_record_commitment
                    .clone(),
                active_authoritative_participant_id: current
                    .authority
                    .active_authoritative_participant_id
                    .clone()
                    .unwrap(),
                authoritative_lineage_commitment: current
                    .observation
                    .authoritative_lineage_commitment
                    .clone(),
                lifecycle_posture: current.authority.lifecycle_posture,
            },
            orchestration_session_id: current.authority.orchestration_session_id.clone(),
            shell_trace_session_id: current.authority.shell_trace_session_id.clone(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::Repl,
                caller_participant_id: current
                    .authority
                    .active_authoritative_participant_id
                    .clone(),
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: current
                .authority
                .active_authoritative_participant_id
                .clone(),
            target_authoritative_participant_id: target_participant_id.into(),
            target_participant_lease_token: b"lease-resume-1".to_vec(),
            run_id: "run-resume-1".into(),
            resulting_authoritative_lineage: lineage,
            workspace_binding: current.authority.workspace_binding.clone(),
            world_binding: current.authority.world_binding.clone(),
            resume_handle_ref: Some(resume_handle_ref),
            transition_input: Some(br#"{"input":"resume"}"#.to_vec()),
            post_turn_disposition: Some(
                HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal,
            ),
        }
    }

    fn started(
        world: bool,
        scope: AgentExecutionScopeV1,
    ) -> (
        tempfile::TempDir,
        HostSessionAuthority,
        IssueHostSessionTransitionRequestV1,
        HostSessionTransitionIntentV2,
    ) {
        let (parent, authority, binding) = authority();
        let mut request = start_request(binding);
        request.start_contract.descriptor.execution_scope = scope;
        request
            .start_contract
            .launch_knobs
            .requested_execution_scope = scope;
        if world {
            request.world_binding = Some(WorldBindingV1 {
                world_id: "verifier-world".into(),
                world_generation: 1,
            });
        }
        let apply = issue_and_claim_start(&authority, &request);
        let TransitionApplicationOutcomeV1::Applied(intent) = authority
            .apply_start_at(&apply, timestamp("2026-07-14T12:01:10.000000000Z"))
            .unwrap()
        else {
            panic!("genuine Start must apply")
        };
        (parent, authority, request, intent)
    }

    // Inventory every entry, including directory and file identity, permissions,
    // length, mtime, ctime, links and content digest. Access time is intentionally
    // absent because these are reads. No authority subtree is excluded.
    #[derive(Debug, Eq, PartialEq)]
    struct InventoryEntry {
        device: u64,
        inode: u64,
        mode: u32,
        uid: u32,
        gid: u32,
        links: u64,
        length: u64,
        mtime: (i64, i64),
        ctime: (i64, i64),
        content: Option<String>,
    }

    fn inventory(root: &Path) -> BTreeMap<PathBuf, InventoryEntry> {
        use sha2::Digest;
        let mut found = BTreeMap::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            let m = fs::symlink_metadata(&path).unwrap();
            let content = if m.is_file() {
                Some(format!(
                    "{:x}",
                    sha2::Sha256::digest(fs::read(&path).unwrap())
                ))
            } else if m.file_type().is_symlink() {
                Some(fs::read_link(&path).unwrap().display().to_string())
            } else {
                None
            };
            if m.is_dir() {
                pending.extend(fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
            }
            found.insert(
                path.strip_prefix(root).unwrap().to_path_buf(),
                InventoryEntry {
                    device: m.dev(),
                    inode: m.ino(),
                    mode: m.mode(),
                    uid: m.uid(),
                    gid: m.gid(),
                    links: m.nlink(),
                    length: m.len(),
                    mtime: (m.mtime(), m.mtime_nsec()),
                    ctime: (m.ctime(), m.ctime_nsec()),
                    content,
                },
            );
        }
        found
    }

    fn unchanged<T>(home: &Path, operation: impl FnOnce() -> T) -> T {
        let before = inventory(home);
        let result = operation();
        assert_eq!(
            inventory(home),
            before,
            "verifier must not write, reconcile or upgrade"
        );
        eprintln!(
            "HSA_CURRENT_HOST_RUN_INVENTORY entries={} unchanged=true",
            before.len()
        );
        result
    }

    fn observe(authority: &HostSessionAuthority, session: &str) -> AuthorityObservationV1 {
        // Fixture/preflight effect outside the subsequent verifier inventory interval.
        authority
            .resolve_current_exact(session, None)
            .unwrap()
            .observation
    }

    fn authenticate<'a>(
        authority: &'a HostSessionAuthority,
        request: &IssueHostSessionTransitionRequestV1,
        observation: &AuthorityObservationV1,
    ) -> Result<AuthenticatedCurrentHostRunV1<'a>> {
        authority.verify_current_host_run_v1(
            &request.orchestration_session_id,
            &request.target_authoritative_participant_id,
            &request.start_contract.descriptor.backend_id,
            &request.run_id,
            observation,
        )
    }

    fn park_start(
        authority: &HostSessionAuthority,
        request: &IssueHostSessionTransitionRequestV1,
        intent: &HostSessionTransitionIntentV2,
    ) -> AuthorityObjectRefV1 {
        let registration = prepare_start_continuity_transaction(authority, request, intent);
        let StartContinuationOutcomeV1::Applied(registered) = authority
            .establish_start_continuation(&registration)
            .unwrap()
        else {
            panic!("genuine continuation registration must apply")
        };
        let settlement = clean_start_settlement_request(
            &registration,
            registered.resume_handle_ref,
            registered.authority_revision_after,
        );
        let StartTurnSettlementOutcomeV1::Applied(settled) =
            authority.settle_start_turn(&settlement).unwrap()
        else {
            panic!("genuine Start settlement must apply")
        };
        settled.continuation_resume_handle_ref
    }

    fn at(minute: u32, second: u32) -> TimestampV1 {
        timestamp(&format!("2026-07-14T12:{minute:02}:{second:02}.000000000Z"))
    }

    fn apply_successor(
        authority: &HostSessionAuthority,
        request: &IssueSuccessorTransitionRequestV1,
        minute: u32,
    ) -> HostSessionTransitionIntentV3 {
        let current = authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap();
        let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_successor_at(&current, request, at(minute, 0), 300)
            .unwrap()
        else {
            panic!("genuine successor must issue")
        };
        let claim = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: format!("claim-{}", issued.intent_id),
            claimant_attempt_id: format!("attempt-{}", issued.intent_id),
        };
        let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_successor_at(&claim, at(minute, 10), 30)
            .unwrap()
        else {
            panic!("genuine successor must claim")
        };
        let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = &claimed.state
        else {
            panic!("genuine claim state required")
        };
        let apply = ApplyHostSessionTransitionRequestV1 {
            intent_id: claimed.intent_id.clone(),
            issuer_request_id: claimed.issuer_request_id.clone(),
            payload_commitment: claimed.payload_commitment.clone(),
            expected_intent_revision: claimed.intent_revision,
            claim_id: claim.claim_id,
            expected_claim_revision: *claim_revision,
        };
        let SuccessorTransitionApplicationOutcomeV1::Applied(applied) = authority
            .apply_successor_at(&apply, at(minute, 20))
            .unwrap()
        else {
            panic!("genuine successor application required")
        };
        applied
    }

    fn successor_proof<'a>(
        authority: &'a HostSessionAuthority,
        intent: &HostSessionTransitionIntentV3,
        observation: &AuthorityObservationV1,
    ) -> Result<AuthenticatedCurrentHostRunV1<'a>> {
        authority.verify_current_host_run_v1(
            &intent.orchestration_session_id,
            &intent.target_authoritative_participant_id,
            "cli:codex",
            &intent.run_id,
            observation,
        )
    }

    #[test]
    fn test_current_host_run_start_v2_and_claim_refusals_do_not_write_or_upgrade() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let observation = observe(&authority, &request.orchestration_session_id);
        let proof = unchanged(home, || {
            authenticate(&authority, &request, &observation).unwrap()
        });
        assert_eq!(proof.run_id(), request.run_id);
        assert_eq!(proof.original_start_run_id(), request.run_id);
        assert_eq!(proof.original_start_intent_id(), request.intent_id);
        assert_eq!(
            proof.original_start_issuer_request_id(),
            request.issuer_request_id
        );
        assert_eq!(
            proof.original_start_payload_commitment(),
            &original.payload_commitment
        );
        assert_eq!(proof.transition_mode(), HostSessionTransitionModeV1::Start);
        assert_eq!(
            proof.application_result_ref(),
            proof.original_start_application_result_ref()
        );
        unchanged(home, || proof.revalidate().unwrap());
        for field in 0..4 {
            let mut values = [
                request.orchestration_session_id.clone(),
                request.target_authoritative_participant_id.clone(),
                "cli:codex".into(),
                request.run_id.clone(),
            ];
            values[field].push_str("-substituted");
            unchanged(home, || {
                assert!(authority
                    .verify_current_host_run_v1(
                        &values[0],
                        &values[1],
                        &values[2],
                        &values[3],
                        &observation
                    )
                    .is_err())
            });
        }
        for field in 0..7 {
            let mut changed = observation.clone();
            match field {
                0 => changed.authority_store_id.push_str("-wrong"),
                1 => changed.orchestration_session_id.push_str("-wrong"),
                2 => changed.root_revision += 1,
                3 => changed.authority_revision += 1,
                4 => {
                    changed.authority_record_commitment =
                        AuthorityObjectCommitmentV1::CanonicalSha256 {
                            digest_hex: "f".repeat(64),
                        }
                }
                5 => {
                    changed.authoritative_lineage_commitment =
                        AuthorityObjectCommitmentV1::CanonicalSha256 {
                            digest_hex: "e".repeat(64),
                        }
                }
                _ => changed.bootstrap_home.physical_path.push_str("-wrong"),
            }
            unchanged(home, || {
                assert!(authenticate(&authority, &request, &changed).is_err())
            });
        }
        assert_eq!(authority.read_a12a_root().unwrap().schema_version, 2);
        assert!(
            super::super::trusted_fs::TrustedAuthorityRoot::open_for_owner(
                home,
                unsafe { libc::geteuid() }.saturating_add(1)
            )
            .is_err()
        );
    }

    #[test]
    fn test_current_host_run_start_after_genuine_startup_acceptance_is_read_only() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let result = ResolveStartupOwnershipRequestV1 {
            intent_id: original.intent_id.clone(),
            issuer_request_id: original.issuer_request_id.clone(),
            payload_commitment: original.payload_commitment.clone(),
            protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: request.target_authoritative_participant_id.clone(),
            },
            protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
                ownership_acknowledgement_id: "verifier-start-accepted".into(),
            },
            observed_at: at(1, 20),
        };
        authority
            .resolve_startup_ownership_at(&result, at(1, 21))
            .unwrap();
        let observation = observe(&authority, &request.orchestration_session_id);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let proof = unchanged(home, || {
            authenticate(&authority, &request, &observation).unwrap()
        });
        assert_eq!(proof.run_id(), request.run_id);
        assert_eq!(proof.transition_mode(), HostSessionTransitionModeV1::Start);
        unchanged(home, || proof.revalidate().unwrap());
        let root = authority.read_a12b_root().unwrap();
        let HostSessionTransitionIntentStateV2::Applied {
            startup_ownership, ..
        } = &root.transition_intent_map[&request.intent_id].state
        else {
            panic!("actual original Start must remain Applied")
        };
        let HostSessionStartupOwnershipApplicationV1::Accepted { result_ref, .. } =
            startup_ownership.as_ref()
        else {
            panic!("genuine ownership acceptance must be retained")
        };
        let object = home.join(format!(
            "authority-v1/objects/startup-ownership-result/v{}/{}.obj",
            result_ref.schema_version, result_ref.ref_id
        ));
        assert!(object.is_file());
        fs::remove_file(object).unwrap();
        unchanged(home, || {
            assert!(
                authenticate(&authority, &request, &observation).is_err(),
                "initial-application projection cannot mask missing real Accepted ownership"
            )
        });
    }

    #[test]
    fn test_current_host_run_registration_keeps_start_with_fresh_observation() {
        let (_parent, authority, request, _) = started(true, AgentExecutionScopeV1::Host);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let before = observe(&authority, &request.orchestration_session_id);
        let old = unchanged(home, || {
            authenticate(&authority, &request, &before).unwrap()
        });
        let plan = RetainedWorkerRegistrationPlanV1 {
            registration_request_id: "verifier-retained-registration".into(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                authority_store_id: before.authority_store_id.clone(),
                authority_revision: before.authority_revision,
                authority_record_commitment: before.authority_record_commitment.clone(),
            },
            retained_participant_id: "verifier-retained-worker".into(),
            descriptor: AgentDescriptorV1 {
                schema_version: 1,
                agent_id: "verifier-worker".into(),
                backend_id: "cli:codex-world".into(),
                backend_kind: RuntimeBackendKindV1::Codex,
                protocol: "substrate.agent.session".into(),
                execution_scope: AgentExecutionScopeV1::World,
                binary_path: "/usr/bin/codex".into(),
            },
            internal_uaa_session_id: "verifier-retained-native".into(),
        };
        RetainedWorkerRuntime
            .register_retained_target(&authority, &plan)
            .unwrap();
        let after = observe(&authority, &request.orchestration_session_id);
        assert!(after.authority_revision > before.authority_revision);
        assert!(after.root_revision > before.root_revision);
        unchanged(home, || assert!(old.revalidate().is_err()));
        let fresh = unchanged(home, || authenticate(&authority, &request, &after).unwrap());
        assert_eq!(fresh.run_id(), old.run_id());
        assert_eq!(fresh.intent_id(), old.intent_id());
        assert_eq!(fresh.transition_mode(), HostSessionTransitionModeV1::Start);
        unchanged(home, || fresh.revalidate().unwrap());
        unchanged(home, || {
            assert!(authority
                .verify_current_host_run_v1(
                    &request.orchestration_session_id,
                    &plan.retained_participant_id,
                    "cli:codex-world",
                    &request.run_id,
                    &after
                )
                .is_err())
        });
    }

    #[test]
    fn test_current_host_run_world_role_and_other_store_are_refused_without_writes() {
        let (_p, world, request, _) = started(true, AgentExecutionScopeV1::World);
        let observation = observe(&world, &request.orchestration_session_id);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        unchanged(home, || {
            assert!(authenticate(&world, &request, &observation).is_err())
        });
        let (_p2, other, other_request, _) = started(false, AgentExecutionScopeV1::Host);
        unchanged(
            Path::new(
                &other_request
                    .workspace_binding
                    .authority_store_root
                    .physical_path,
            ),
            || {
                assert!(authenticate(&other, &request, &observation).is_err());
                let mut same_home_wrong_store = observation.clone();
                same_home_wrong_store.bootstrap_home =
                    other_request.workspace_binding.authority_store_root.clone();
                assert!(authenticate(&other, &request, &same_home_wrong_store).is_err());
            },
        );
    }

    #[test]
    fn test_current_host_run_genuine_attach_supersedes_original_start() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        park_start(&authority, &request, &original);
        let parked = authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap();
        let attach = attach_successor_request(&parked, "verifier-attach");
        let applied = apply_successor(&authority, &attach, 3);
        let observation = observe(&authority, &request.orchestration_session_id);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let proof = unchanged(home, || {
            successor_proof(&authority, &applied, &observation).unwrap()
        });
        assert_eq!(proof.transition_mode(), HostSessionTransitionModeV1::Attach);
        assert_eq!(proof.run_id(), attach.run_id);
        assert_eq!(proof.original_start_run_id(), request.run_id);
        assert_ne!(
            proof.application_result_ref(),
            proof.original_start_application_result_ref()
        );
        unchanged(home, || proof.revalidate().unwrap());
        unchanged(home, || {
            assert!(authenticate(&authority, &request, &observation).is_err())
        });
        authority
            .resolve_startup_ownership_at(
                &ResolveStartupOwnershipRequestV1 {
                    intent_id: applied.intent_id.clone(),
                    issuer_request_id: applied.issuer_request_id.clone(),
                    payload_commitment: applied.payload_commitment.clone(),
                    protocol_actor:
                        HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
                            participant_id: applied.target_authoritative_participant_id.clone(),
                        },
                    protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
                        ownership_acknowledgement_id: "verifier-attach-accepted".into(),
                    },
                    observed_at: at(3, 30),
                },
                at(3, 31),
            )
            .unwrap();
        let accepted = observe(&authority, &request.orchestration_session_id);
        unchanged(home, || assert!(proof.revalidate().is_err()));
        unchanged(home, || {
            successor_proof(&authority, &applied, &accepted)
                .unwrap()
                .revalidate()
                .unwrap()
        });
    }

    struct InheritedGuardEnvironment {
        previous: Option<std::ffi::OsString>,
        _file: fs::File,
    }

    impl Drop for InheritedGuardEnvironment {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => {
                    std::env::set_var("SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD", value)
                }
                None => std::env::remove_var("SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD"),
            }
        }
    }

    fn settle_fixture_resume(
        authority: &HostSessionAuthority,
        request: &IssueSuccessorTransitionRequestV1,
        applied: &HostSessionTransitionIntentV3,
        minute: u32,
    ) -> AuthorityObjectRefV1 {
        // Genuine HSA and inbox producers, with a real exclusively flocked helper
        // description. Native fields below are typed protocol-fixture observations,
        // not evidence of a real native/provider process.
        let helper_plan = HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::ResumeOneTurn,
            descriptor: ResolvedRuntimeDescriptor {
                agent_id: "codex".into(),
                backend_id: "cli:codex".into(),
                backend_kind: ResolvedRuntimeBackendKind::Codex,
                protocol: "substrate.agent.session".into(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: "/usr/bin/codex".into(),
            },
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id: applied.orchestration_session_id.clone(),
                shell_trace_session_id: applied.shell_trace_session_id.clone(),
                workspace_root: applied
                    .workspace_binding
                    .workspace_root
                    .physical_path
                    .clone(),
                world_id: None,
                world_generation: None,
            },
            participant: HiddenOwnerHelperParticipantPlan {
                participant_id: applied.target_authoritative_participant_id.clone(),
                lease_token: String::from_utf8(request.target_participant_lease_token.clone())
                    .unwrap(),
                run_id: applied.run_id.clone(),
                resumed_from_participant_id: applied.source_authoritative_participant_id.clone(),
                internal_uaa_session_id: Some("uaa-session-start-1".into()),
            },
            host_attach_contract: None,
            startup_prompt: Some(HiddenOwnerHelperStartupPromptPlan {
                prompt_text: String::from_utf8(request.transition_input.clone().unwrap()).unwrap(),
                stream_path: Path::new(&applied.workspace_binding.workspace_root.physical_path)
                    .join("unused-fixture-stream"),
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: None,
                public_scope: None,
            }),
            source_orchestration_session_id: None,
        };
        let plan = AuthorityManagedSuccessorLaunchPlanV1 {
            schema_version: 1,
            authority_store_id: applied.workspace_binding.authority_store_id.clone(),
            helper_plan,
            applied_transition: applied.clone(),
        };
        let path = Path::new(&applied.workspace_binding.authority_store_root.physical_path)
            .join("runtime-control/durable-start/successor-launch-guards");
        fs::create_dir_all(&path).unwrap();
        // R02-F01: production guard identity checks reopen each component as
        // a private directory. These are owned fixture directories, not repairs.
        let mut component =
            Path::new(&applied.workspace_binding.authority_store_root.physical_path).to_path_buf();
        for name in [
            "runtime-control",
            "durable-start",
            "successor-launch-guards",
        ] {
            component.push(name);
            fs::set_permissions(&component, fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(fs::metadata(&component).unwrap().mode() & 0o777, 0o700);
        }
        let file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path.join(format!("{}.lock", applied.intent_id)))
            .unwrap();
        fs::set_permissions(
            path.join(format!("{}.lock", applied.intent_id)),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        fs2::FileExt::lock_exclusive(&file).unwrap();
        let previous = std::env::var_os("SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD");
        std::env::set_var(
            "SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD",
            file.as_raw_fd().to_string(),
        );
        let _guard = InheritedGuardEnvironment {
            previous,
            _file: file,
        };
        let terminal = DirectHostNativeTerminalEvidenceV1 {
            schema_version: 1,
            backend_id: "cli:codex".into(),
            protocol: "substrate.agent.session".into(),
            native_session_id: "uaa-session-start-1".into(),
            thread_id: "uaa-session-start-1".into(),
            turn_id: format!("fixture-turn-{minute}"),
            sequence: DirectHostNativeSequenceV1::Codex {
                resumed_thread_id: "uaa-session-start-1".into(),
                started_turn_id: format!("fixture-turn-{minute}"),
            },
            terminal_event_kind: "turn.completed".into(),
            raw_event_sha256: "e".repeat(64),
            terminal_ordinal: 1,
        };
        authority
            .settle_direct_host_resume(
                &plan,
                &terminal,
                HostPostTurnProtocolEventKindV1::ResumableClean,
                &at(minute, 40),
            )
            .unwrap();
        let root = authority.read_a12b_root().unwrap();
        let intent = &root.successor_transition_intent_map[&applied.intent_id];
        let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &intent.state else {
            panic!("settled Resume must remain Applied")
        };
        let HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } = post_turn.as_ref()
        else {
            panic!("genuine completion must commit")
        };
        let completion: PostTurnCompletionHashInputV3 = canonical_json::from_slice(
            &authority
                .read_authority_object_v2_at(root.root_revision, completion_ref)
                .unwrap(),
        )
        .unwrap();
        completion.produced_resume_handle_ref.unwrap()
    }

    #[test]
    #[serial_test::serial]
    fn test_current_host_run_two_genuine_resumes_keep_origin_and_reject_prior_runs() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let mut handle = park_start(&authority, &request, &original);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let mut prior: Option<(String, String)> = None;
        for turn in 1..=2 {
            let source = authority
                .resolve_current_exact(&request.orchestration_session_id, None)
                .unwrap();
            let mut successor = resume_successor_request(
                &source,
                handle.clone(),
                &format!("verifier-resume-{turn}"),
            );
            successor.intent_id = format!("verifier-resume-intent-{turn}");
            successor.issuer_request_id = format!("verifier-resume-request-{turn}");
            successor.run_id = format!("verifier-resume-run-{turn}");
            let minute = 2 + 2 * turn;
            let applied = apply_successor(&authority, &successor, minute);
            let observation = observe(&authority, &request.orchestration_session_id);
            let proof = unchanged(home, || {
                successor_proof(&authority, &applied, &observation).unwrap()
            });
            assert_eq!(proof.run_id(), successor.run_id);
            assert_eq!(proof.original_start_run_id(), request.run_id);
            assert_eq!(proof.original_start_intent_id(), request.intent_id);
            assert_eq!(
                proof.transition_mode(),
                HostSessionTransitionModeV1::ResumeOneTurn
            );
            unchanged(home, || proof.revalidate().unwrap());
            unchanged(home, || {
                assert!(authenticate(&authority, &request, &observation).is_err())
            });
            if let Some((participant, run)) = &prior {
                unchanged(home, || {
                    assert!(authority
                        .verify_current_host_run_v1(
                            &request.orchestration_session_id,
                            participant,
                            "cli:codex",
                            run,
                            &observation
                        )
                        .is_err())
                });
            }
            authority
                .accept_transition_input(&AcceptTransitionInputRequestV1 {
                    intent_id: applied.intent_id.clone(),
                    issuer_request_id: applied.issuer_request_id.clone(),
                    payload_commitment: applied.payload_commitment.clone(),
                    accepting_participant_id: applied.target_authoritative_participant_id.clone(),
                    accepted_at: at(minute, 30),
                })
                .unwrap();
            unchanged(home, || assert!(proof.revalidate().is_err()));
            let accepted_observation = observe(&authority, &request.orchestration_session_id);
            unchanged(home, || {
                successor_proof(&authority, &applied, &accepted_observation)
                    .unwrap()
                    .revalidate()
                    .unwrap()
            });
            let next = settle_fixture_resume(&authority, &successor, &applied, minute);
            assert_ne!(
                next, handle,
                "genuine settlement produces the target-owned next handle"
            );
            handle = next;
            prior = Some((
                successor.target_authoritative_participant_id.clone(),
                successor.run_id.clone(),
            ));
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_current_host_run_leaf_rejects_ancestor_handle_from_genuine_resume_source() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let original_handle = park_start(&authority, &request, &original);
        let initial = authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap();
        let mut request1 =
            resume_successor_request(&initial, original_handle.clone(), "verifier-source-resume");
        request1.intent_id = "verifier-source-resume-intent".into();
        request1.issuer_request_id = "verifier-source-resume-request".into();
        request1.run_id = "verifier-source-resume-run".into();
        let applied = apply_successor(&authority, &request1, 4);
        authority
            .accept_transition_input(&AcceptTransitionInputRequestV1 {
                intent_id: applied.intent_id.clone(),
                issuer_request_id: applied.issuer_request_id.clone(),
                payload_commitment: applied.payload_commitment.clone(),
                accepting_participant_id: applied.target_authoritative_participant_id.clone(),
                accepted_at: at(4, 30),
            })
            .unwrap();
        let produced = settle_fixture_resume(&authority, &request1, &applied, 4);
        let source = authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap();
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        assert!(source
            .authority
            .internal_resume_handle_refs
            .contains(&original_handle));
        assert!(source
            .authority
            .authoritative_participant_lineage
            .contains(&request.target_authoritative_participant_id));
        assert_eq!(
            source
                .authority
                .active_authoritative_participant_id
                .as_ref(),
            Some(&applied.target_authoritative_participant_id)
        );
        // Bounded leaf refusal, not a claim that an invalid successor was issued
        // or Applied. Both source and both handles were genuinely produced.
        unchanged(home, || {
            store::with_existing_current_host_run_snapshot_opened(
                authority.trusted_root(),
                |snapshot| {
                    verify_consumed_host_handle(
                        snapshot,
                        &produced,
                        &source.authority,
                        &source.caller.descriptor,
                    )
                    .unwrap();
                    let refusal = verify_consumed_host_handle(
                        snapshot,
                        &original_handle,
                        &source.authority,
                        &source.caller.descriptor,
                    )
                    .err()
                    .unwrap();
                    assert_eq!(
                        refusal.to_string(),
                        "Start handle does not join exact source owner and native registration"
                    );
                    Ok(())
                },
            )
            .unwrap();
        });
        // Reuse the existing focused R04 native/producer substitution proof on
        // this genuinely produced completion; no new history is inserted.
        let root = authority.read_a12b_root().unwrap();
        super::super::direct_host_resume::assert_direct_host_completion_v3_refusals_for_test(
            &authority,
            &root,
            &root.successor_transition_intent_map[&applied.intent_id],
        );
    }

    #[test]
    fn test_current_host_run_v1_and_unsupported_version_refuse_without_upgrade() {
        let (_parent, authority, binding) = authority();
        let root = authority.read_root().unwrap();
        let placeholder = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "f".repeat(64),
        };
        let held = AuthorityObservationV1 {
            authority_store_id: root.authority_store_id.clone(),
            bootstrap_home: root.bootstrap_home.clone(),
            orchestration_session_id: "missing-session".into(),
            root_revision: root.root_revision,
            authority_revision: 1,
            authority_record_commitment: placeholder.clone(),
            authoritative_lineage_commitment: placeholder,
        };
        let home = Path::new(&binding.authority_store_root.physical_path);
        unchanged(home, || {
            assert!(authority
                .verify_current_host_run_v1("missing-session", "p", "cli:codex", "r", &held)
                .is_err())
        });
        // Negative fixture corruption only. No positive history/application insertion.
        let path = home.join("authority-v1/state-root-v1.json");
        let mut unsupported: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        unsupported["schema_version"] = serde_json::json!(99);
        fs::write(&path, canonical_json::to_vec(&unsupported).unwrap()).unwrap();
        unchanged(home, || {
            assert!(authority
                .verify_current_host_run_v1("missing-session", "p", "cli:codex", "r", &held)
                .is_err())
        });
    }

    #[test]
    fn test_current_host_run_missing_application_refuses_without_repair() {
        let (_parent, authority, request, _) = started(false, AgentExecutionScopeV1::Host);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let observation = observe(&authority, &request.orchestration_session_id);
        let proof = unchanged(home, || {
            authenticate(&authority, &request, &observation).unwrap()
        });
        let reference = proof.application_result_ref();
        let object = home.join(format!(
            "authority-v1/objects/application-result/v{}/{}.obj",
            reference.schema_version, reference.ref_id
        ));
        assert!(object.is_file());
        fs::remove_file(object).unwrap();
        unchanged(home, || {
            assert!(authenticate(&authority, &request, &observation).is_err())
        });
        unchanged(home, || assert!(proof.revalidate().is_err()));
    }

    #[test]
    fn test_current_host_run_retained_proof_rejects_workspace_replacement() {
        let (parent, authority, binding) = authority();
        let workspace_path = parent.path().join("workspace");
        fs::create_dir(&workspace_path).unwrap();
        let mut request = start_request(binding);
        let metadata = fs::metadata(&workspace_path).unwrap();
        request.workspace_binding.workspace_root = CanonicalDirectoryV1 {
            physical_path: workspace_path.to_str().unwrap().into(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
        };
        let apply = issue_and_claim_start(&authority, &request);
        authority.apply_start_at(&apply, at(1, 10)).unwrap();
        let observation = observe(&authority, &request.orchestration_session_id);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let proof = unchanged(home, || {
            authenticate(&authority, &request, &observation).unwrap()
        });
        fs::rename(&workspace_path, parent.path().join("retained-workspace")).unwrap();
        fs::create_dir(&workspace_path).unwrap();
        unchanged(parent.path(), || {
            assert!(proof.revalidate().is_err());
            assert!(authenticate(&authority, &request, &observation).is_err());
        });
    }

    #[test]
    fn test_current_host_run_retained_proof_rejects_configured_home_replacement() {
        let (parent, authority, request, _) = started(false, AgentExecutionScopeV1::Host);
        let observation = observe(&authority, &request.orchestration_session_id);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let proof = unchanged(home, || {
            authenticate(&authority, &request, &observation).unwrap()
        });
        fs::rename(home, parent.path().join("retained-home")).unwrap();
        fs::create_dir(home).unwrap();
        fs::set_permissions(home, fs::Permissions::from_mode(0o700)).unwrap();
        unchanged(parent.path(), || assert!(proof.revalidate().is_err()));
    }

    #[test]
    fn test_current_host_run_snapshot_exit_detects_root_and_observation_races() {
        use std::sync::{Arc, Mutex};
        for replace_home in [false, true] {
            let (parent, authority, request, _) = started(false, AgentExecutionScopeV1::Host);
            let observation = observe(&authority, &request.orchestration_session_id);
            let home = PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
            let parent_path = parent.path().to_path_buf();
            let after_producer = Arc::new(Mutex::new(None));
            let capture = Arc::clone(&after_producer);
            let inventory_root = parent_path.clone();
            super::super::store::set_current_host_run_read_hook(
                super::super::store::CurrentHostRunReadPhaseV1::BeforeCustodyFinalize,
                move || {
                    if replace_home {
                        fs::rename(&home, parent_path.join("retained-home")).unwrap();
                        fs::create_dir(&home).unwrap();
                        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
                    } else {
                        let path = home.join("authority-v1/state-root-v1.json");
                        let mut value: serde_json::Value =
                            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                        value["root_revision"] =
                            serde_json::json!(value["root_revision"].as_u64().unwrap() + 1);
                        fs::write(path, canonical_json::to_vec(&value).unwrap()).unwrap();
                    }
                    *capture.lock().unwrap() = Some(inventory(&inventory_root));
                },
            );
            let result = authenticate(&authority, &request, &observation);
            assert!(
                result.is_err(),
                "snapshot exit must dominate successful semantic authentication"
            );
            let expected = after_producer
                .lock()
                .unwrap()
                .take()
                .expect("race hook must run");
            assert_eq!(
                inventory(parent.path()),
                expected,
                "only the exact race producer may mutate this fixture"
            );
        }
    }

    #[test]
    fn test_current_host_run_v3_public_start_registration_keeps_same_run() {
        let (_parent, authority, request, original) = started(true, AgentExecutionScopeV1::Host);
        // Genuine public Start enrollment upgrades to V3 and retains the actual
        // unsettled transaction; no positive root/history is inserted manually.
        prepare_start_continuity_transaction(&authority, &request, &original);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let before = observe(&authority, &request.orchestration_session_id);
        let old = unchanged(home, || {
            authenticate(&authority, &request, &before).unwrap()
        });
        let plan = RetainedWorkerRegistrationPlanV1 {
            registration_request_id: "verifier-v3-retained-registration".into(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                authority_store_id: before.authority_store_id.clone(),
                authority_revision: before.authority_revision,
                authority_record_commitment: before.authority_record_commitment.clone(),
            },
            retained_participant_id: "verifier-v3-retained-worker".into(),
            descriptor: AgentDescriptorV1 {
                schema_version: 1,
                agent_id: "verifier-worker".into(),
                backend_id: "cli:codex-world".into(),
                backend_kind: RuntimeBackendKindV1::Codex,
                protocol: "substrate.agent.session".into(),
                execution_scope: AgentExecutionScopeV1::World,
                binary_path: "/usr/bin/codex".into(),
            },
            internal_uaa_session_id: "verifier-v3-retained-native".into(),
        };
        RetainedWorkerRuntime
            .register_retained_target(&authority, &plan)
            .unwrap();
        let after = observe(&authority, &request.orchestration_session_id);
        assert!(after.authority_revision > before.authority_revision);
        assert!(after.root_revision > before.root_revision);
        unchanged(home, || assert!(old.revalidate().is_err()));
        let fresh = unchanged(home, || authenticate(&authority, &request, &after).unwrap());
        assert_eq!(fresh.run_id(), request.run_id);
        assert_eq!(fresh.original_start_run_id(), old.original_start_run_id());
        assert_eq!(
            fresh.original_start_intent_id(),
            old.original_start_intent_id()
        );
        assert_eq!(
            fresh.original_start_application_result_ref(),
            old.original_start_application_result_ref()
        );
        assert_eq!(fresh.transition_mode(), HostSessionTransitionModeV1::Start);
        unchanged(home, || fresh.revalidate().unwrap());
        // Preserve the generic reader's strict default and original full root.
        unchanged(home, || {
            assert!(store::with_existing_typed_object_snapshot_opened(
                authority.trusted_root(),
                |_| Ok(())
            )
            .is_err())
        });
        unchanged(home, || {
            store::with_existing_current_host_run_snapshot_opened(
                authority.trusted_root(),
                |view| {
                    let VersionedStateRoot::V3(root) = view.captured_root() else {
                        panic!("genuine public Start must remain V3")
                    };
                    assert!(!root.start_transaction_map.is_empty());
                    Ok(())
                },
            )
            .unwrap()
        });
    }

    #[test]
    fn test_current_host_run_precaptured_entry_rejects_semantically_equal_replacement() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        for replace in [false, true] {
            let (parent, authority, request, _) = started(false, AgentExecutionScopeV1::Host);
            let observation = observe(&authority, &request.orchestration_session_id);
            let home = PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
            let root = home.join("authority-v1/state-root-v1.json");
            let after_producer = Rc::new(RefCell::new(None));
            let captured = Rc::clone(&after_producer);
            let inventory_root = parent.path().to_path_buf();
            hook(Phase::BeforeScope, move || {
                let bytes = fs::read(&root).unwrap();
                if replace {
                    let replacement = root.with_extension("replacement");
                    fs::write(&replacement, &bytes).unwrap();
                    fs::set_permissions(&replacement, fs::Permissions::from_mode(0o600)).unwrap();
                    fs::rename(&replacement, &root).unwrap();
                } else {
                    fs::write(&root, &bytes).unwrap();
                }
                *captured.borrow_mut() = Some(inventory(&inventory_root));
            });
            let events = Rc::new(RefCell::new(Vec::new()));
            let first = Rc::clone(&events);
            hook(Phase::BeforeCustodyFinalize, move || {
                first.borrow_mut().push("adapter-finalizer-entered")
            });
            let second = Rc::clone(&events);
            hook(Phase::AfterCustodyFinalize, move || {
                second.borrow_mut().push("adapter-finalizer-completed")
            });
            let error = authenticate(&authority, &request, &observation)
                .err()
                .expect("pre-scope entry changed");
            assert!(error.to_string().contains("custody finalization failed"));
            assert_eq!(
                *events.borrow(),
                vec!["adapter-finalizer-entered", "adapter-finalizer-completed"]
            );
            assert_eq!(
                inventory(parent.path()),
                after_producer.borrow_mut().take().unwrap()
            );
        }
    }

    #[test]
    fn test_current_host_run_captured_object_key_and_namespace_faults_refuse_without_writes() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        for (key, alter, namespace) in [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, true, false),
            (false, false, true),
        ] {
            let (parent, authority, request, original) =
                started(false, AgentExecutionScopeV1::Host);
            let observation = observe(&authority, &request.orchestration_session_id);
            let home = PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
            let path = if key {
                let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } =
                    &original.target_participant_lease_token_ref.commitment
                else {
                    panic!("genuine token uses store key")
                };
                home.join(format!("authority-v1/keys/{key_id}.key"))
            } else {
                home.join(format!(
                    "authority-v1/objects/agent-descriptor/v1/{}.obj",
                    original.descriptor_ref.ref_id
                ))
            };
            assert!(path.is_file());
            let after_producer = Rc::new(RefCell::new(None));
            let captured = Rc::clone(&after_producer);
            let inventory_root = parent.path().to_path_buf();
            hook(Phase::BeforeCustodyFinalize, move || {
                if namespace {
                    let directory = path.parent().unwrap();
                    let retained = directory.with_extension("retained");
                    fs::rename(directory, &retained).unwrap();
                    fs::create_dir(directory).unwrap();
                    fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
                    fs::copy(retained.join(path.file_name().unwrap()), &path).unwrap();
                } else {
                    let mut bytes = fs::read(&path).unwrap();
                    if alter {
                        bytes[0] ^= 1;
                        fs::write(&path, bytes).unwrap();
                    } else {
                        let next = path.with_extension("replacement");
                        fs::write(&next, bytes).unwrap();
                        fs::set_permissions(&next, fs::Permissions::from_mode(0o600)).unwrap();
                        fs::rename(next, &path).unwrap();
                    }
                }
                *captured.borrow_mut() = Some(inventory(&inventory_root));
            });
            let finalized = Rc::new(RefCell::new(false));
            let completed = Rc::clone(&finalized);
            hook(Phase::AfterCustodyFinalize, move || {
                *completed.borrow_mut() = true
            });
            assert!(authenticate(&authority, &request, &observation).is_err());
            assert!(
                *finalized.borrow(),
                "adapter finalizer must run even when integrity fails"
            );
            assert_eq!(
                inventory(parent.path()),
                after_producer.borrow_mut().take().unwrap(),
                "verifier adds no mutation to fault producer"
            );
        }
    }

    #[test]
    fn test_current_host_run_outer_finalizer_dominates_success_semantic_and_custody_errors() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        for semantic_error in [false, true] {
            for custody_error in [false, true] {
                let (parent, authority, request, original) =
                    started(false, AgentExecutionScopeV1::Host);
                let observation = observe(&authority, &request.orchestration_session_id);
                let home =
                    PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
                if custody_error {
                    let object = home.join(format!(
                        "authority-v1/objects/agent-descriptor/v1/{}.obj",
                        original.descriptor_ref.ref_id
                    ));
                    hook(Phase::BeforeCustodyFinalize, move || {
                        let bytes = fs::read(&object).unwrap();
                        fs::write(object, bytes).unwrap();
                    });
                }
                let after_producer = Rc::new(RefCell::new(None));
                let captured = Rc::clone(&after_producer);
                let inventory_root = parent.path().to_path_buf();
                hook(Phase::AfterCustodyFinalize, move || {
                    // This runs AFTER adapter verification, so only the actual
                    // unchanged outer scope verifier can reject this movement.
                    let path = home.join("authority-v1/state-root-v1.json");
                    let mut root: serde_json::Value =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    root["root_revision"] =
                        serde_json::json!(root["root_revision"].as_u64().unwrap() + 1);
                    fs::write(path, canonical_json::to_vec(&root).unwrap()).unwrap();
                    *captured.borrow_mut() = Some(inventory(&inventory_root));
                });
                let error = authority
                    .verify_current_host_run_v1(
                        &request.orchestration_session_id,
                        &request.target_authoritative_participant_id,
                        if semantic_error {
                            "wrong-backend"
                        } else {
                            "cli:codex"
                        },
                        &request.run_id,
                        &observation,
                    )
                    .err()
                    .expect("outer finalization must dominate every callback outcome");
                assert!(
                    error
                        .to_string()
                        .contains("direct-host inbox authority root changed"),
                    "actual outer verification error must dominate: {error}"
                );
                assert_eq!(
                    inventory(parent.path()),
                    after_producer.borrow_mut().take().unwrap()
                );
            }
        }
    }

    #[test]
    fn test_current_host_run_partial_key_capture_auth_refusal_keeps_integrity_priority() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        let (parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let home = PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
        let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } =
            &original.target_participant_lease_token_ref.commitment
        else {
            panic!("genuine token uses store key")
        };
        let path = home.join(format!("authority-v1/keys/{key_id}.key"));
        let after_producer = Rc::new(RefCell::new(None));
        let captured = Rc::clone(&after_producer);
        let inventory_root = parent.path().to_path_buf();
        hook(Phase::BeforeObjectAuthentication, move || {
            let bytes = fs::read(&path).unwrap();
            fs::write(path, bytes).unwrap();
            *captured.borrow_mut() = Some(inventory(&inventory_root));
        });
        let result = store::with_existing_current_host_run_snapshot_opened(
            authority.trusted_root(),
            |view| {
                let root = view.captured_root();
                let wrong = ObjectVerificationContextV1 {
                    intent_id: original.intent_id.clone(),
                    run_id: "wrong-run".into(),
                    parent_intent: None,
                };
                let refusal = view.read_typed_object(
                    root.authority_store_id(),
                    root.root_revision(),
                    &original.target_participant_lease_token_ref,
                    Some(&wrong),
                );
                assert!(
                    refusal.is_err(),
                    "the captured token must fail its wrong HMAC context"
                );
                refusal.map(|_| ())
            },
        );
        let error = result
            .err()
            .expect("retained partial capture must fail integrity");
        assert!(
            error.to_string().contains("custody finalization failed"),
            "integrity cannot disappear behind contextual auth refusal: {error}"
        );
        assert_eq!(
            inventory(parent.path()),
            after_producer.borrow_mut().take().unwrap()
        );
    }

    #[test]
    fn test_current_host_run_semantic_refusal_after_contextual_reads_is_finalized() {
        use crate::execution::agent_runtime::dispatch_policy_commitment::ReadOnlyAuthoritySnapshotErrorV1;
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let events = Rc::new(RefCell::new(Vec::new()));
        let first = Rc::clone(&events);
        hook(Phase::BeforeCustodyFinalize, move || {
            first.borrow_mut().push(1)
        });
        let second = Rc::clone(&events);
        hook(Phase::AfterCustodyFinalize, move || {
            second.borrow_mut().push(2)
        });
        let result: std::result::Result<(), _> = unchanged(home, || {
            store::with_existing_current_host_run_snapshot_opened(
                authority.trusted_root(),
                |view| {
                    let root = view.captured_root();
                    let context = ObjectVerificationContextV1 {
                        intent_id: original.intent_id.clone(),
                        run_id: original.run_id.clone(),
                        parent_intent: None,
                    };
                    view.read_typed_object(
                        root.authority_store_id(),
                        root.root_revision(),
                        &original.target_participant_lease_token_ref,
                        Some(&context),
                    )?;
                    Err(ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "deliberate semantic refusal after contextual reads",
                    })
                },
            )
        });
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("contextual read refused"));
        assert_eq!(*events.borrow(), vec![1, 2]);
    }

    #[test]
    fn test_current_host_run_workspace_replaced_after_captured_reads_refuses() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        let (parent, authority, binding) = authority();
        let workspace = parent.path().join("workspace");
        fs::create_dir(&workspace).unwrap();
        let mut request = start_request(binding);
        let metadata = fs::metadata(&workspace).unwrap();
        request.workspace_binding.workspace_root = CanonicalDirectoryV1 {
            physical_path: workspace.to_str().unwrap().into(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
        };
        let apply = issue_and_claim_start(&authority, &request);
        authority.apply_start_at(&apply, at(1, 10)).unwrap();
        let observation = observe(&authority, &request.orchestration_session_id);
        let after_producer = Rc::new(RefCell::new(None));
        let capture = Rc::clone(&after_producer);
        let inventory_root = parent.path().to_path_buf();
        hook(Phase::BeforeCustodyFinalize, move || {
            fs::rename(&workspace, inventory_root.join("retained-workspace")).unwrap();
            fs::create_dir(&workspace).unwrap();
            *capture.borrow_mut() = Some(inventory(&inventory_root));
        });
        assert!(
            authenticate(&authority, &request, &observation).is_err(),
            "retained opened workspace must be revalidated before returning verified success"
        );
        assert_eq!(
            inventory(parent.path()),
            after_producer.borrow_mut().take().unwrap()
        );
    }

    #[test]
    fn test_current_host_run_partial_object_capture_survives_missing_key_auth_error() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        let (parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let home = PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
        let mut reference = original.target_participant_lease_token_ref.clone();
        let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } = &mut reference.commitment
        else {
            panic!("real lease token uses store key")
        };
        *key_id = "missing-key-auth-negative".into();
        let path = home.join(format!(
            "authority-v1/objects/lease-token/v1/{}.obj",
            reference.ref_id
        ));
        assert!(path.is_file());
        let after_producer = Rc::new(RefCell::new(None));
        let capture = Rc::clone(&after_producer);
        let inventory_root = parent.path().to_path_buf();
        hook(Phase::BeforeCustodyFinalize, move || {
            let bytes = fs::read(&path).unwrap();
            fs::write(path, bytes).unwrap();
            *capture.borrow_mut() = Some(inventory(&inventory_root));
        });
        let result = store::with_existing_current_host_run_snapshot_opened(
            authority.trusted_root(),
            |view| {
                let root = view.captured_root();
                let context = ObjectVerificationContextV1 {
                    intent_id: original.intent_id.clone(),
                    run_id: original.run_id.clone(),
                    parent_intent: None,
                };
                let error = view
                    .read_typed_object(
                        root.authority_store_id(),
                        root.root_revision(),
                        &reference,
                        Some(&context),
                    )
                    .unwrap_err();
                assert!(
                    format!("{error:?}").contains("key is unavailable"),
                    "must fail after object capture and before key capture"
                );
                Err::<(), _>(error)
            },
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("custody finalization failed"),
            "partial object capture must remain registered despite key authentication failure"
        );
        assert_eq!(
            inventory(parent.path()),
            after_producer.borrow_mut().take().unwrap()
        );
    }

    #[test]
    fn test_current_host_run_v3_original_immutable_and_real_ownership_checks() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        prepare_start_continuity_transaction(&authority, &request, &original);
        authority
            .resolve_startup_ownership_at(
                &ResolveStartupOwnershipRequestV1 {
                    intent_id: original.intent_id.clone(),
                    issuer_request_id: original.issuer_request_id.clone(),
                    payload_commitment: original.payload_commitment.clone(),
                    protocol_actor:
                        HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
                            participant_id: request.target_authoritative_participant_id.clone(),
                        },
                    protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
                        ownership_acknowledgement_id: "v3-original-owner-accepted".into(),
                    },
                    observed_at: at(1, 25),
                },
                at(1, 26),
            )
            .unwrap();
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let observation = observe(&authority, &request.orchestration_session_id);
        let proof = unchanged(home, || {
            authenticate(&authority, &request, &observation).unwrap()
        });
        unchanged(home, || {
            store::with_existing_current_host_run_snapshot_opened(
                authority.trusted_root(),
                |view| {
                    let VersionedStateRoot::V3(root) = view.captured_root() else {
                        panic!("genuine public Start is V3")
                    };
                    let original = &root.transition_intent_map[&request.intent_id];
                    let read =
                        |reference: &AuthorityObjectRefV1,
                         context: Option<&ObjectVerificationContextV1>| {
                            view.read_typed_object(
                                &root.authority_store_id,
                                root.root_revision,
                                reference,
                                context,
                            )
                            .map_err(|error| anyhow!("{error:?}"))
                        };
                    let history = exact_v3_authority_history_with_reader(
                        root,
                        &request.orchestration_session_id,
                        |reference| read(reference, None).map_err(|error| error.to_string()),
                    )
                    .unwrap();
                    let initial = &history[&1];
                    verify_original_start_application_v3_with_reader(
                        original,
                        &initial.authority,
                        &initial.authority_record_commitment,
                        &mut |reference, context| read(reference, context),
                    )
                    .unwrap();
                    // Negative-only reader injections AFTER actual captured typed reads
                    // isolate immutable semantic comparisons. No fixture root/history,
                    // positive authority or returned proof is fabricated by these bytes.
                    for fault in 0..4 {
                        let result = verify_original_start_application_v3_with_reader(
                            original,
                            &initial.authority,
                            &initial.authority_record_commitment,
                            &mut |reference, context| {
                                let bytes = read(reference, context)?;
                                let selected = match fault {
                                    0 => {
                                        reference.object_kind
                                            == AuthorityObjectKindV1::ApplicationResult
                                    }
                                    1 => reference == &original.descriptor_ref,
                                    2 => reference.object_kind == AuthorityObjectKindV1::Policy,
                                    _ => reference == &original.transport_payload_ref,
                                };
                                if !selected {
                                    return Ok(bytes);
                                }
                                let mut value: serde_json::Value =
                                    serde_json::from_slice(&bytes).unwrap();
                                match fault {
                                    0 | 3 => {
                                        value["run_id"] = serde_json::json!("wrong-original-run")
                                    }
                                    1 => {
                                        value["descriptor"]["backend_id"] =
                                            serde_json::json!("wrong-original-backend")
                                    }
                                    _ => {
                                        value["policy_revision"] =
                                            serde_json::json!("wrong-original-policy")
                                    }
                                }
                                canonical_json::to_vec(&value).map_err(|error| anyhow!("{error}"))
                            },
                        );
                        assert!(
                            result.is_err(),
                            "original immutable semantic substitution {fault} must refuse"
                        );
                    }
                    let mut changed = original.clone();
                    changed.payload_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                        digest_hex: "e".repeat(64),
                    };
                    let error = verify_original_start_application_v3_with_reader(
                        &changed,
                        &initial.authority,
                        &initial.authority_record_commitment,
                        &mut |reference, context| read(reference, context),
                    )
                    .unwrap_err();
                    assert!(error
                        .to_string()
                        .contains("payload commitment does not verify"));
                    let mut changed = original.clone();
                    let HostSessionTransitionIntentStateV2::Applied { claim_id, .. } =
                        &mut changed.state
                    else {
                        unreachable!()
                    };
                    claim_id.clear();
                    assert!(verify_original_start_application_v3_with_reader(
                        &changed,
                        &initial.authority,
                        &initial.authority_record_commitment,
                        &mut |reference, context| read(reference, context)
                    )
                    .is_err());
                    let mut changed = initial.authority.clone();
                    changed.current_policy_revision = Some("substituted-policy".into());
                    assert!(verify_original_start_application_v3_with_reader(
                        original,
                        &changed,
                        &initial.authority_record_commitment,
                        &mut |reference, context| read(reference, context)
                    )
                    .is_err());
                    Ok(())
                },
            )
            .unwrap()
        });
        let root = authority.read_a12b_root().unwrap();
        let HostSessionTransitionIntentStateV2::Applied {
            startup_ownership, ..
        } = &root.transition_intent_map[&request.intent_id].state
        else {
            unreachable!()
        };
        let HostSessionStartupOwnershipApplicationV1::Accepted { result_ref, .. } =
            startup_ownership.as_ref()
        else {
            panic!("real original ownership must be Accepted")
        };
        let result = home.join(format!(
            "authority-v1/objects/startup-ownership-result/v{}/{}.obj",
            result_ref.schema_version, result_ref.ref_id
        ));
        fs::remove_file(result).unwrap();
        unchanged(home, || {
            assert!(proof.revalidate().is_err());
            assert!(authenticate(&authority, &request, &observation).is_err());
        });
    }
}
