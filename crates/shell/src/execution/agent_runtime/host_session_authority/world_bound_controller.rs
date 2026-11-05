//! Source authentication for an ordinary local Host controller with a requested
//! World selection. Native completion never certifies separate World work.

use anyhow::{ensure, Result};
use std::sync::Arc;
use transport_api_types::AgentSelectionPlacementV1;

use crate::execution::agent_runtime::{
    control::HiddenOwnerHelperLaunchPlan, native_launch_context::AuthenticatedOrdinarySelectionV1,
};

use super::{
    facade::ResolvedCurrentAuthorityV1,
    schema::{AgentExecutionScopeV1, HostSessionTransitionModeV1, WorldBindingV1},
    store_schema::HostSessionTransitionIntentV3,
    trusted_fs::TrustedWorkspaceRoot,
};

mod completion;
mod publication;
pub(super) use completion::{
    verify_world_controller_completed_join, verify_world_controller_completion_v4_edge,
};
pub(crate) use publication::{
    authenticate_ordinary_world_publication, AuthenticatedOrdinaryWorldPublicationV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldBoundControllerEntryV1 {
    Start,
    ResumeOneTurn,
}

/// Private construction joins the genuine original selection to an actual
/// Applied helper application. No serde, Clone or caller-supplied factory can
/// turn a World label/binding into this source result. It grants no delivery,
/// work/attention closure, World effect, or HSA settlement permission.
pub(crate) struct AuthenticatedWorldBoundControllerRunV1 {
    selection: Arc<AuthenticatedOrdinarySelectionV1>,
    origin: ResolvedCurrentAuthorityV1,
    entry: WorldBoundControllerEntryV1,
    intent_id: String,
    run_id: String,
    world_binding: WorldBindingV1,
    workspace: TrustedWorkspaceRoot,
}

impl AuthenticatedWorldBoundControllerRunV1 {
    pub(crate) fn authenticate_received_helper(
        selection: Arc<AuthenticatedOrdinarySelectionV1>,
        helper: &HiddenOwnerHelperLaunchPlan,
        applied_successor: Option<&HostSessionTransitionIntentV3>,
    ) -> Result<Self> {
        selection.require_ordinary_configured_authority()?;
        selection.revalidate()?;
        ensure!(
            selection.native_selection().placement == AgentSelectionPlacementV1::Host
                && selection.requested_selection().placement == AgentSelectionPlacementV1::World,
            "World-bound controller requires distinct original native Host and requested World roles"
        );
        let (world_id, world_generation) = selection
            .original_world_binding()
            .ok_or_else(|| anyhow::anyhow!("original World controller placement is absent"))?;
        let world_binding = WorldBindingV1 {
            world_id: world_id.into(),
            world_generation,
        };
        let observation = selection
            .held_authority()
            .observe_current_host_gateway_run_v1(
                helper.orchestration_session_id(),
                helper.participant_id(),
                &helper.descriptor.backend_id,
                &helper.participant.run_id,
            )?;
        let result = (|| {
            let proof = observation.current_run();
            proof.verify_selected_helper_launch_v1(helper, applied_successor, &selection)?;
            let entry = match proof.transition_mode() {
                HostSessionTransitionModeV1::Start => WorldBoundControllerEntryV1::Start,
                HostSessionTransitionModeV1::ResumeOneTurn => {
                    WorldBoundControllerEntryV1::ResumeOneTurn
                }
                _ => anyhow::bail!("World-bound controller is outside issued Start/Resume"),
            };
            let current = selection
                .held_authority()
                .resolve_current_exact(helper.orchestration_session_id(), None)?;
            ensure!(
                observation.original_start().world_binding.as_ref() == Some(&world_binding)
                    && current.authority.world_binding.as_ref() == Some(&world_binding)
                    && current.caller.descriptor.execution_scope == AgentExecutionScopeV1::Host
                    && current.caller.participant_id == proof.participant_id()
                    && current.caller.descriptor.backend_id == proof.backend_id()
                    && proof.authority_store_id() == selection.authority_store_id()
                    && proof.original_start_intent_id() == selection.original_start_intent_id()
                    && proof.original_start_issuer_request_id()
                        == selection.original_start_issuer_request_id()?,
                "World controller original/current application or placement changed"
            );
            let workspace = TrustedWorkspaceRoot::open_exact(
                &current.authority.workspace_binding.workspace_root,
            )?;
            Ok(Self {
                selection: Arc::clone(&selection),
                origin: current,
                entry,
                intent_id: proof.intent_id().into(),
                run_id: proof.run_id().into(),
                world_binding,
                workspace,
            })
        })();
        // Physical/source exit checks also run on semantic refusal. No root
        // lock survives these reads or spans a later service call.
        let current_exit = observation.revalidate();
        let original_exit = selection.revalidate();
        current_exit?;
        original_exit?;
        result
    }

    pub(crate) fn verify_workspace(&self) -> Result<()> {
        self.workspace.revalidate()?;
        Ok(())
    }

    pub(crate) fn selection(&self) -> &Arc<AuthenticatedOrdinarySelectionV1> {
        &self.selection
    }

    pub(crate) fn origin(&self) -> &ResolvedCurrentAuthorityV1 {
        &self.origin
    }

    pub(crate) fn entry(&self) -> WorldBoundControllerEntryV1 {
        self.entry
    }

    pub(crate) fn intent_id(&self) -> &str {
        &self.intent_id
    }

    pub(crate) fn run_id(&self) -> &str {
        &self.run_id
    }

    pub(crate) fn world_binding(&self) -> &WorldBindingV1 {
        &self.world_binding
    }
}

/// Wire-shape tests use real complete selected roles and a genuine captured
/// empty session cut. Synthetic terminal/reference values here are never
/// submitted, published to HSA, or counted as runtime completion evidence.
#[cfg(test)]
pub(crate) fn exercise_world_controller_completion_codecs_for_test(
    controller: &AuthenticatedWorldBoundControllerRunV1,
    cut: &super::schema::WorldControllerWorkAttentionCutV1,
) -> Result<()> {
    use super::{canonical_json, hash, schema::*, validation::ValidatedCanonicalV1 as _};
    let reference = |kind| AuthorityObjectRefV1 {
        ref_id: format!("ao_{}", "a".repeat(32)),
        object_kind: kind,
        schema_version: 1,
        commitment: if kind == AuthorityObjectKindV1::TransitionInput {
            AuthorityObjectCommitmentV1::StoreHmacSha256 {
                key_id: format!("ak_{}", "a".repeat(32)),
                domain: "substrate.a1.transition-input.v1".into(),
                digest_hex: "a".repeat(64),
            }
        } else {
            AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "a".repeat(64),
            }
        },
    };
    let current = &controller.origin().authority;
    let descriptor = &controller.origin().caller.descriptor;
    let mut registered = reference(AuthorityObjectKindV1::ResumeHandle);
    registered.schema_version = 2;
    let completed_at = TimestampV1::parse("2026-11-02T00:00:00.000000000Z")?;
    let core = StartContinuationHandleHashInputV2 {
        schema_version: 2,
        authority_store_id: cut.authority_store_id.clone(),
        orchestration_session_id: cut.orchestration_session_id.clone(),
        participant_id: controller.origin().caller.participant_id.clone(),
        backend_id: descriptor.backend_id.clone(),
        protocol: descriptor.protocol.clone(),
        internal_uaa_session_id: "codec-only-thread".into(),
        start_intent_id: controller.intent_id().into(),
        start_issuer_request_id: "codec-only-issuer".into(),
        start_payload_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "a".repeat(64),
        },
        start_application_result_ref: reference(AuthorityObjectKindV1::ApplicationResult),
        start_run_id: controller.run_id().into(),
        authority_revision_before: 1,
        authority_record_commitment_before: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "b".repeat(64),
        },
        authority_revision_after: 2,
        state: StartContinuationHandleStateV2::Settled {
            registered_resume_handle_ref: Box::new(registered.clone()),
            protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: controller.origin().caller.participant_id.clone(),
            },
            event_id: "codec-only-event".into(),
            event_sequence: 2,
            provider_event_kind: "turn.completed".into(),
            thread_id: "codec-only-thread".into(),
            turn_id: "codec-only-turn".into(),
            evidence_sha256: "c".repeat(64),
            completion_kind: StartTurnCompletionKindV1::ResumableClean,
            obligation_snapshot: None,
            completed_at: completed_at.clone(),
        },
    };
    let old_bytes = hash::canonical_object_bytes(
        AuthorityObjectKindV1::ResumeHandle,
        hash::CanonicalObjectHashInputV1::StartContinuationHandle(&core),
    )?;
    let start = StartContinuationHandleHashInputV3 {
        schema_version: 3,
        continuation: core.clone(),
        world_binding: controller.world_binding().clone(),
        native_selection: controller.selection().native_selection().clone(),
        requested_selection: controller.selection().requested_selection().clone(),
        work_attention_cut: cut.clone(),
    };
    let bytes = hash::canonical_object_bytes(
        AuthorityObjectKindV1::ResumeHandle,
        hash::CanonicalObjectHashInputV1::WorldStartContinuationHandle(&start),
    )?;
    assert_eq!(
        canonical_json::from_slice::<StartContinuationHandleHashInputV3>(&bytes)?,
        start
    );
    assert!(canonical_json::from_slice::<StartContinuationHandleHashInputV2>(&bytes).is_err());
    assert!(canonical_json::from_slice::<StartContinuationHandleHashInputV3>(&old_bytes).is_err());
    assert_eq!(
        old_bytes,
        hash::canonical_object_bytes(
            AuthorityObjectKindV1::ResumeHandle,
            hash::CanonicalObjectHashInputV1::StartContinuationHandle(&core)
        )?
    );
    assert_eq!(
        start.resulting_posture(),
        HostSessionPostureV1::ParkedResumable
    );
    let mut world_reference = reference(AuthorityObjectKindV1::ResumeHandle);
    world_reference.schema_version = 3;
    let decoded =
        super::start_continuity::decode_start_continuation_handle(&world_reference, &bytes)?;
    assert!(decoded.world_controller().is_some());
    assert_eq!(
        decoded.committed_posture()?,
        HostSessionPostureV1::ParkedResumable
    );
    let mut wrong_outcome = start.clone();
    if let StartContinuationHandleStateV2::Settled {
        provider_event_kind,
        ..
    } = &mut wrong_outcome.continuation.state
    {
        *provider_event_kind = "turn.failed".into();
    }
    assert!(
        wrong_outcome.validate().is_err(),
        "native failure cannot certify resumable success"
    );
    let mut pending = start.clone();
    pending
        .work_attention_cut
        .proposal_request_ids
        .push("pending-real-source-shape".into());
    pending
        .work_attention_cut
        .pending_request_ids
        .push("pending-real-source-shape".into());
    pending.validate()?;
    assert_eq!(
        pending.resulting_posture(),
        HostSessionPostureV1::AwaitingAttention
    );
    pending.work_attention_cut.pending_request_ids.clear();
    assert!(
        pending.validate().is_err(),
        "dropping pending coverage cannot encode a clean source cut"
    );
    let mut swapped = start.clone();
    std::mem::swap(
        &mut swapped.native_selection,
        &mut swapped.requested_selection,
    );
    assert!(swapped.validate().is_err());
    let mut unknown = serde_json::to_value(&start)?;
    unknown["future"] = serde_json::json!(true);
    assert!(serde_json::from_value::<StartContinuationHandleHashInputV3>(unknown).is_err());
    let binding = WorldControllerResumeBindingV1 {
        schema_version: 1,
        authority_store_id: cut.authority_store_id.clone(),
        bootstrap_home: controller.origin().observation.bootstrap_home.clone(),
        workspace_binding: current.workspace_binding.clone(),
        world_binding: controller.world_binding().clone(),
        orchestration_session_id: cut.orchestration_session_id.clone(),
        shell_trace_session_id: current.shell_trace_session_id.clone(),
        authoritative_participant_id: controller.origin().caller.participant_id.clone(),
        authoritative_lineage: current.authoritative_participant_lineage.clone(),
        original_start_intent_id: controller.selection().original_start_intent_id().into(),
        original_start_run_id: controller.run_id().into(),
        native_selection: start.native_selection.clone(),
        requested_selection: start.requested_selection.clone(),
        authority_revision_observed: current.authority_revision,
        authority_record_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "d".repeat(64),
        },
        policy_ref: current
            .current_policy_ref
            .clone()
            .ok_or_else(|| anyhow::anyhow!("fixture policy absent"))?,
        policy_revision: current
            .current_policy_revision
            .clone()
            .ok_or_else(|| anyhow::anyhow!("fixture policy revision absent"))?,
        descriptor_ref: reference(AuthorityObjectKindV1::AgentDescriptor),
        intent_id: "codec-only-resume".into(),
        issuer_request_id: "codec-only-issuer".into(),
        payload_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "e".repeat(64),
        },
        intent_revision_observed: 1,
        claim_id: "codec-only-claim".into(),
        claimant_attempt_id: "codec-only-attempt".into(),
        run_id: "codec-only-run".into(),
        initial_application_result_ref: reference(AuthorityObjectKindV1::ApplicationResult),
        transition_input_ref: reference(AuthorityObjectKindV1::TransitionInput),
        input_acceptance_ref: reference(AuthorityObjectKindV1::InputAcceptance),
        consumed_resume_handle_ref: registered,
    };
    let completion = PostTurnCompletionHashInputV4 {
        schema_version: 4,
        binding,
        protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: controller.origin().caller.participant_id.clone(),
        },
        event_id: "codec-only-resume-terminal".into(),
        kind: HostPostTurnProtocolEventKindV1::ResumableClean,
        native_terminal: DirectHostNativeTerminalEvidenceV1 {
            schema_version: 1,
            backend_id: descriptor.backend_id.clone(),
            protocol: descriptor.protocol.clone(),
            native_session_id: "codec-only-thread".into(),
            thread_id: "codec-only-thread".into(),
            turn_id: "codec-only-turn".into(),
            sequence: DirectHostNativeSequenceV1::Codex {
                resumed_thread_id: "codec-only-thread".into(),
                started_turn_id: "codec-only-turn".into(),
            },
            terminal_event_kind: "turn.completed".into(),
            raw_event_sha256: "f".repeat(64),
            terminal_ordinal: 1,
        },
        work_attention_cut: cut.clone(),
        completed_at,
        produced_resume_handle_ref: Some(reference(AuthorityObjectKindV1::ResumeHandle)),
    };
    let bytes = hash::canonical_object_bytes(
        AuthorityObjectKindV1::PostTurnCompletion,
        hash::CanonicalObjectHashInputV1::PostTurnCompletionV4(&completion),
    )?;
    assert_eq!(
        canonical_json::from_slice::<PostTurnCompletionHashInputV4>(&bytes)?,
        completion
    );
    assert!(canonical_json::from_slice::<PostTurnCompletionHashInputV3>(&bytes).is_err());
    assert!(canonical_json::from_slice::<PostTurnCompletionHashInputV2>(&bytes).is_err());
    assert_eq!(
        completion.resulting_posture(),
        HostSessionPostureV1::ParkedResumable
    );
    let mut missing = completion.clone();
    missing.produced_resume_handle_ref = None;
    assert!(
        missing.validate().is_err(),
        "resumable World completion must own a continuation"
    );
    let mut wrong_native = completion;
    wrong_native.native_terminal.turn_id = "substituted".into();
    assert!(wrong_native.validate().is_err());
    Ok(())
}
