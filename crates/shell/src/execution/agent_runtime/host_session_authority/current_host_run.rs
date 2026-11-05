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

use crate::execution::{
    agent_runtime::control::{
        HiddenOwnerHelperLaunchPlan, OwnerHelperMode, ResolvedRuntimeBackendKind,
        ResolvedRuntimeDescriptor,
    },
    config_model::AgentExecutionScope,
};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

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

    /// Compare a received public Start/Resume helper with this actual Applied run.
    ///
    /// Start requires its genuine, still-unsubmitted public transaction. Resume
    /// requires the supplied Applied successor envelope; only the stored intent
    /// supplies authentication contexts. Host-rooted World placement is retained.
    ///
    /// A match grants no input delivery, replay, lifecycle or process-liveness
    /// authority. In particular Accepted input stays Accepted. Both snapshot exit
    /// checks run on match/refusal; no lock is held across later caller effects.
    pub(crate) fn verify_helper_launch_v1(
        &self,
        helper: &HiddenOwnerHelperLaunchPlan,
        applied_successor: Option<&HostSessionTransitionIntentV3>,
    ) -> Result<()> {
        self.workspace
            .revalidate()
            .map_err(|_| anyhow!("helper launch workspace custody refused"))?;
        let snapshot_result = store::with_existing_current_host_run_snapshot_opened(
            self.authority.trusted_root(),
            |snapshot| {
                // Keep semantic failure inside Ok so neither snapshot finalizer
                // can be bypassed. The retained workspace check is unconditional.
                let semantic = (|| {
                    let (fresh, workspace) = authenticate_current_host_run(
                        self.authority,
                        snapshot,
                        self.orchestration_session_id(),
                        self.participant_id(),
                        self.backend_id(),
                        self.run_id(),
                        &self.facts.observation,
                    )?;
                    let comparison = (|| {
                        ensure!(fresh == self.facts, "current helper run changed");
                        compare_helper_launch_snapshot(snapshot, &fresh, helper, applied_successor)
                    })();
                    workspace
                        .revalidate()
                        .map_err(|_| anyhow!("fresh helper workspace custody refused"))?;
                    comparison
                })();
                let workspace_exit = self.workspace.revalidate();
                Ok(if workspace_exit.is_err() {
                    Err(anyhow!("retained helper workspace custody refused"))
                } else {
                    semantic
                })
            },
        );
        // These checks also run after a snapshot refusal. Only fixed, nonsecret
        // errors leave this boundary, with physical integrity taking precedence.
        let workspace_exit = self.workspace.revalidate();
        let root_exit = self.authority.trusted_root().revalidate();
        if workspace_exit.is_err() || root_exit.is_err() {
            bail!("helper launch physical custody refused");
        }
        match snapshot_result {
            Err(_) => bail!("helper launch snapshot integrity refused"),
            Ok(Err(_)) => bail!("helper launch comparison refused"),
            Ok(Ok(())) => Ok(()),
        }
    }

    /// Compare only the captured selected public Start/Resume producer. No delivery permit.
    #[cfg(target_os = "linux")]
    pub(crate) fn verify_selected_helper_launch_v1(
        &self,
        helper: &HiddenOwnerHelperLaunchPlan,
        applied_successor: Option<&HostSessionTransitionIntentV3>,
        original_selection: &crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1,
    ) -> Result<()> {
        self.workspace
            .revalidate()
            .map_err(|_| anyhow!("helper launch workspace custody refused"))?;
        let snapshot_result = store::with_existing_current_host_run_snapshot_opened(
            self.authority.trusted_root(),
            |snapshot| {
                // Keep semantic failure inside Ok so neither snapshot finalizer
                // can be bypassed. The retained workspace check is unconditional.
                let semantic = original_selection
                    .with_original_start_context_for_helper_comparison_v1(|context| {
                        let (fresh, workspace) = authenticate_current_host_run(
                            self.authority,
                            snapshot,
                            self.orchestration_session_id(),
                            self.participant_id(),
                            self.backend_id(),
                            self.run_id(),
                            &self.facts.observation,
                        )?;
                        let comparison = (|| {
                            ensure!(fresh == self.facts, "current helper run changed");
                            ensure!(
                                context.workspace_binding.authority_store_root
                                    == *self.authority.trusted_root().identity(),
                                "selected helper physical root differs"
                            );
                            compare_selected_helper_launch_snapshot(
                                snapshot,
                                &fresh,
                                helper,
                                applied_successor,
                                context,
                            )
                        })();
                        workspace
                            .revalidate()
                            .map_err(|_| anyhow!("fresh helper workspace custody refused"))?;
                        comparison
                    });
                let workspace_exit = self.workspace.revalidate();
                Ok(if workspace_exit.is_err() {
                    Err(anyhow!("retained helper workspace custody refused"))
                } else {
                    semantic
                })
            },
        );
        // These checks also run after a snapshot refusal. Only fixed, nonsecret
        // errors leave this boundary, with physical integrity taking precedence.
        let workspace_exit = self.workspace.revalidate();
        let root_exit = self.authority.trusted_root().revalidate();
        if workspace_exit.is_err() || root_exit.is_err() {
            bail!("helper launch physical custody refused");
        }
        match snapshot_result {
            Err(_) => bail!("helper launch snapshot integrity refused"),
            Ok(Err(error)) => Err(error),
            Ok(Ok(())) => Ok(()),
        }
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

fn compare_helper_launch_snapshot(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    facts: &CurrentHostRunFactsV1,
    helper: &HiddenOwnerHelperLaunchPlan,
    supplied: Option<&HostSessionTransitionIntentV3>,
) -> Result<()> {
    let VersionedStateRoot::V3(root) = snapshot.captured_root() else {
        bail!("public helper requires an existing public transaction root")
    };
    let (mode, session, trace, binding, world, participant, run, lease, intent_id) = match facts
        .mode
    {
        HostSessionTransitionModeV1::Start => {
            let intent = root
                .transition_intent_map
                .get(&facts.intent_id)
                .ok_or_else(|| anyhow!("original helper Start absent"))?;
            ensure!(
                supplied.is_none(),
                "Start cannot carry a successor assertion"
            );
            compare_helper_start(root, intent, helper)?;
            (
                OwnerHelperMode::Start,
                &intent.orchestration_session_id,
                &intent.shell_trace_session_id,
                &intent.workspace_binding,
                intent.world_binding.as_ref(),
                &intent.target_authoritative_participant_id,
                &intent.run_id,
                &intent.target_participant_lease_token_ref,
                &intent.intent_id,
            )
        }
        HostSessionTransitionModeV1::ResumeOneTurn => {
            let intent = root
                .successor_transition_intent_map
                .get(&facts.intent_id)
                .ok_or_else(|| anyhow!("current helper Resume absent"))?;
            let supplied = supplied.ok_or_else(|| anyhow!("Applied helper assertion absent"))?;
            compare_helper_successor_intent(intent, supplied)?;
            compare_helper_resume(snapshot, intent, helper)?;
            (
                OwnerHelperMode::ResumeOneTurn,
                &intent.orchestration_session_id,
                &intent.shell_trace_session_id,
                &intent.workspace_binding,
                intent.world_binding.as_ref(),
                &intent.target_authoritative_participant_id,
                &intent.run_id,
                &intent.target_participant_lease_token_ref,
                &intent.intent_id,
            )
        }
        HostSessionTransitionModeV1::Attach => bail!("Attach comparison is outside this boundary"),
    };
    let descriptor: AgentDescriptorHashInputV1 =
        read_run_object(snapshot, &facts.descriptor_ref, None)?;
    compare_helper_descriptor(&descriptor.descriptor, &helper.descriptor)?;
    ensure!(
        helper.mode == mode
            && helper.session.orchestration_session_id == *session
            && helper.session.shell_trace_session_id == *trace
            && helper.session.workspace_root == binding.workspace_root.physical_path
            && helper.session.world_id.as_deref() == world.map(|w| w.world_id.as_str())
            && helper.session.world_generation == world.map(|w| w.world_generation)
            && helper.participant.participant_id == *participant
            && helper.participant.run_id == *run
            && helper.source_orchestration_session_id.is_none(),
        "helper current participant, placement or mode mismatch"
    );
    let context = ObjectVerificationContextV1 {
        intent_id: intent_id.clone(),
        run_id: run.clone(),
        parent_intent: None,
    };
    let lease_bytes = snapshot
        .read_typed_object(
            root.authority_store_id.as_str(),
            root.root_revision,
            lease,
            Some(&context),
        )
        .map_err(|_| anyhow!("helper lease authentication refused"))?;
    ensure!(
        lease_bytes == helper.participant.lease_token.as_bytes(),
        "helper lease mismatch"
    );
    let prompt = helper
        .startup_prompt
        .as_ref()
        .ok_or_else(|| anyhow!("public helper prompt absent"))?;
    ensure!(
        prompt.stream_path
            == helper_launch_stream_path(&root.bootstrap_home.physical_path, session, participant,),
        "helper listener endpoint mismatch"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn compare_selected_helper_launch_snapshot(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    facts: &CurrentHostRunFactsV1,
    helper: &HiddenOwnerHelperLaunchPlan,
    supplied: Option<&HostSessionTransitionIntentV3>,
    context: &crate::execution::agent_runtime::native_launch_context::NativeLaunchContextV1,
) -> Result<()> {
    let VersionedStateRoot::V3(root) = snapshot.captured_root() else {
        bail!("public helper requires an existing public transaction root")
    };
    let original_transaction = join_selected_original_start_v1(snapshot, root, facts, context)?;
    let (mode, session, trace, binding, world, participant, run, lease, intent_id) = match facts
        .mode
    {
        HostSessionTransitionModeV1::Start => {
            let intent = root
                .transition_intent_map
                .get(&facts.intent_id)
                .ok_or_else(|| anyhow!("original helper Start absent"))?;
            ensure!(
                supplied.is_none(),
                "Start cannot carry a successor assertion"
            );
            ensure!(
                intent.mode == HostSessionTransitionModeV1::Start
                    && intent.authority_precondition
                        == HostSessionAuthorityPreconditionV1::ExpectedAbsent
                    && intent.source_authoritative_participant_id.is_none()
                    && intent.resume_handle_ref.is_none()
                    && intent.transition_input_ref.is_none()
                    && intent.post_turn_disposition.is_none()
                    && helper.participant.resumed_from_participant_id.is_none()
                    && helper.participant.internal_uaa_session_id.is_none()
                    && helper.host_attach_contract.is_some(),
                "helper original Start shape mismatch"
            );

            compare_selected_helper_start_prompt(original_transaction, helper, context)?;
            (
                OwnerHelperMode::Start,
                &intent.orchestration_session_id,
                &intent.shell_trace_session_id,
                &intent.workspace_binding,
                intent.world_binding.as_ref(),
                &intent.target_authoritative_participant_id,
                &intent.run_id,
                &intent.target_participant_lease_token_ref,
                &intent.intent_id,
            )
        }
        HostSessionTransitionModeV1::ResumeOneTurn => {
            let intent = root
                .successor_transition_intent_map
                .get(&facts.intent_id)
                .ok_or_else(|| anyhow!("current helper Resume absent"))?;
            let supplied = supplied.ok_or_else(|| anyhow!("Applied helper assertion absent"))?;
            compare_helper_successor_intent(intent, supplied)?;
            compare_helper_resume(snapshot, intent, helper)?;
            (
                OwnerHelperMode::ResumeOneTurn,
                &intent.orchestration_session_id,
                &intent.shell_trace_session_id,
                &intent.workspace_binding,
                intent.world_binding.as_ref(),
                &intent.target_authoritative_participant_id,
                &intent.run_id,
                &intent.target_participant_lease_token_ref,
                &intent.intent_id,
            )
        }
        HostSessionTransitionModeV1::Attach => bail!("Attach comparison is outside this boundary"),
    };
    let descriptor: AgentDescriptorHashInputV1 =
        read_run_object(snapshot, &facts.descriptor_ref, None)?;
    compare_helper_descriptor(&descriptor.descriptor, &helper.descriptor)?;
    ensure!(
        helper.mode == mode
            && helper.session.orchestration_session_id == *session
            && helper.session.shell_trace_session_id == *trace
            && helper.session.workspace_root == binding.workspace_root.physical_path
            && helper.session.world_id.as_deref() == world.map(|w| w.world_id.as_str())
            && helper.session.world_generation == world.map(|w| w.world_generation)
            && helper.participant.participant_id == *participant
            && helper.participant.run_id == *run
            && helper.source_orchestration_session_id.is_none(),
        "helper current participant, placement or mode mismatch"
    );
    let context = ObjectVerificationContextV1 {
        intent_id: intent_id.clone(),
        run_id: run.clone(),
        parent_intent: None,
    };
    let lease_bytes = snapshot
        .read_typed_object(
            root.authority_store_id.as_str(),
            root.root_revision,
            lease,
            Some(&context),
        )
        .map_err(|_| anyhow!("helper lease authentication refused"))?;
    ensure!(
        lease_bytes == helper.participant.lease_token.as_bytes(),
        "helper lease mismatch"
    );
    let prompt = helper
        .startup_prompt
        .as_ref()
        .ok_or_else(|| anyhow!("public helper prompt absent"))?;
    ensure!(
        prompt.stream_path
            == helper_launch_stream_path(&root.bootstrap_home.physical_path, session, participant,),
        "helper listener endpoint mismatch"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn join_selected_original_start_v1<'root>(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    root: &'root StateRootV3,
    facts: &CurrentHostRunFactsV1,
    context: &crate::execution::agent_runtime::native_launch_context::NativeLaunchContextV1,
) -> Result<&'root StartTransactionRecordV1> {
    let original = root
        .transition_intent_map
        .get(&facts.original_start_intent_id)
        .ok_or_else(|| anyhow!("selected original Start absent"))?;
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref,
        authority_revision_after,
        ..
    } = &original.state
    else {
        bail!("selected original Start not Applied")
    };
    ensure!(
        context.workspace_binding == facts.workspace_binding
            && context.workspace_binding.authority_store_id == root.authority_store_id
            && context.workspace_binding.authority_store_root == root.bootstrap_home
            && context.intent_id == original.intent_id
            && context.intent_id == facts.original_start_intent_id
            && context.orchestration_session_id == original.orchestration_session_id
            && context.orchestration_session_id == facts.observation.orchestration_session_id
            && context.shell_trace_session_id == original.shell_trace_session_id
            && context.original_participant_id == original.target_authoritative_participant_id
            && context.original_run_id == original.run_id
            && context.original_run_id == facts.original_start_run_id
            && context.workspace_binding == original.workspace_binding
            && context.original_world_binding == original.world_binding
            && original.mode == HostSessionTransitionModeV1::Start
            && context.issuer_request_id()? == original.issuer_request_id
            && original.issuer_request_id == facts.original_start_issuer_request_id
            && original.payload_commitment == facts.original_start_payload_commitment
            && application_result_ref == &facts.original_start_application_result_ref,
        "selected original context/application differs"
    );
    let descriptor: AgentDescriptorHashInputV1 =
        read_run_object(snapshot, &original.descriptor_ref, None)?;
    ensure!(
        descriptor.descriptor == context.descriptor,
        "selected original descriptor differs"
    );
    context.selections.validate()?;
    let mut transactions = root.start_transaction_map.values().filter(|transaction| {
        transaction.orchestration_session_id == original.orchestration_session_id
    });
    let transaction = transactions
        .next()
        .ok_or_else(|| anyhow!("selected original public transaction absent"))?;
    ensure!(
        transactions.next().is_none()
            && transaction.authority_store_id == root.authority_store_id
            && transaction.start_intent_id == original.intent_id
            && transaction.start_issuer_request_id == original.issuer_request_id
            && transaction.start_payload_commitment == original.payload_commitment
            && transaction.start_application_result_ref == *application_result_ref
            && transaction.start_run_id == original.run_id
            && transaction.start_authority_revision == *authority_revision_after
            && transaction.authoritative_participant_id
                == original.target_authoritative_participant_id
            && transaction.shell_trace_session_id == original.shell_trace_session_id
            && transaction.workspace_root == context.workspace_binding.workspace_root.physical_path
            && transaction.world_id.as_deref()
                == original
                    .world_binding
                    .as_ref()
                    .map(|world| world.world_id.as_str())
            && transaction.world_generation
                == original
                    .world_binding
                    .as_ref()
                    .map(|world| world.world_generation)
            && transaction.backend_id == context.descriptor.backend_id
            && transaction.protocol == context.descriptor.protocol
            && transaction.public_backend_id == context.selections.requested.backend_id
            && transaction.public_scope
                == match context.selections.requested.placement {
                    transport_api_types::AgentSelectionPlacementV1::Host => "host",
                    transport_api_types::AgentSelectionPlacementV1::World => "world",
                }
            && context.selections.native.backend_id == context.descriptor.backend_id
            && context.selections.native.realized_agent_id == context.descriptor.agent_id
            && original.world_binding.is_some()
                == (context.selections.requested.placement
                    == transport_api_types::AgentSelectionPlacementV1::World),
        "selected original transaction differs"
    );
    Ok(transaction)
}

#[cfg(target_os = "linux")]
fn compare_selected_helper_start_prompt(
    transaction: &StartTransactionRecordV1,
    helper: &HiddenOwnerHelperLaunchPlan,
    context: &crate::execution::agent_runtime::native_launch_context::NativeLaunchContextV1,
) -> Result<()> {
    let prompt = helper
        .startup_prompt
        .as_ref()
        .ok_or_else(|| anyhow!("selected Start prompt absent"))?;
    let public_scope = match prompt.public_scope {
        Some(AgentExecutionScope::Host) => "host",
        Some(AgentExecutionScope::World) => "world",
        None => bail!("selected Start public scope absent"),
    };
    let prompt_digest = helper_start_digest(
        "substrate.a1.3-p1.public-start-prompt.v1",
        &[prompt.prompt_text.as_bytes()],
    );
    ensure!(
        matches!(
            transaction.state,
            StartTransactionStateV1::PromptNotSubmitted
        ) && prompt.request_key_sha256.as_ref() == Some(&transaction.request_key_sha256)
            && prompt.prompt_sha256.as_ref() == Some(&transaction.prompt_sha256)
            && prompt.public_backend_id.as_ref() == Some(&transaction.public_backend_id)
            && public_scope == transaction.public_scope
            && prompt_digest == transaction.prompt_sha256,
        "selected Start prompt/public identity differs"
    );
    let request = serde_json::json!({
        "schema_version": 2,
        "native_selection": &context.selections.native,
        "requested_selection": &context.selections.requested,
        "descriptor": &helper.descriptor,
        "workspace_root": &helper.session.workspace_root,
        "host_attach_contract": &helper.host_attach_contract,
        "public_backend_id": &transaction.public_backend_id,
        "public_scope": public_scope,
        "prompt_sha256": &prompt_digest,
    });
    let bytes = canonical_json::to_vec(&request)?;
    ensure!(
        helper_start_digest("substrate.selected-public-start-request.v1", &[&bytes])
            == transaction.request_key_sha256,
        "selected full original request differs"
    );
    Ok(())
}

fn compare_helper_start(
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV2,
    helper: &HiddenOwnerHelperLaunchPlan,
) -> Result<()> {
    let HostSessionTransitionIntentStateV2::Applied {
        application_result_ref,
        authority_revision_after,
        ..
    } = &intent.state
    else {
        bail!("original helper Start was not Applied")
    };
    ensure!(
        intent.mode == HostSessionTransitionModeV1::Start
            && intent.authority_precondition == HostSessionAuthorityPreconditionV1::ExpectedAbsent
            && intent.source_authoritative_participant_id.is_none()
            && intent.resume_handle_ref.is_none()
            && intent.transition_input_ref.is_none()
            && intent.post_turn_disposition.is_none()
            && helper.participant.resumed_from_participant_id.is_none()
            && helper.participant.internal_uaa_session_id.is_none()
            && helper.host_attach_contract.is_some(),
        "helper original Start shape mismatch"
    );
    let mut matches = root
        .start_transaction_map
        .values()
        .filter(|t| t.orchestration_session_id == intent.orchestration_session_id);
    let transaction = matches
        .next()
        .ok_or_else(|| anyhow!("public Start transaction absent"))?;
    ensure!(
        matches.next().is_none(),
        "public Start transaction ambiguous"
    );
    ensure!(
        transaction.authority_store_id == root.authority_store_id
            && transaction.start_intent_id == intent.intent_id
            && transaction.start_issuer_request_id == intent.issuer_request_id
            && transaction.start_payload_commitment == intent.payload_commitment
            && transaction.start_application_result_ref == *application_result_ref
            && transaction.start_run_id == intent.run_id
            && transaction.start_authority_revision == *authority_revision_after
            && transaction.authoritative_participant_id
                == intent.target_authoritative_participant_id
            && transaction.shell_trace_session_id == intent.shell_trace_session_id
            && transaction.workspace_root == intent.workspace_binding.workspace_root.physical_path
            && transaction.world_id.as_deref()
                == intent.world_binding.as_ref().map(|w| w.world_id.as_str())
            && transaction.world_generation
                == intent.world_binding.as_ref().map(|w| w.world_generation)
            && transaction.backend_id == helper.descriptor.backend_id
            && transaction.protocol == helper.descriptor.protocol
            && transaction.public_scope
                == if intent.world_binding.is_some() {
                    "world"
                } else {
                    "host"
                }
            && matches!(
                transaction.state,
                StartTransactionStateV1::PromptNotSubmitted
            ),
        "helper public transaction does not join unsubmitted original application"
    );
    compare_helper_start_prompt(transaction, helper)
}

fn compare_helper_start_prompt(
    transaction: &StartTransactionRecordV1,
    helper: &HiddenOwnerHelperLaunchPlan,
) -> Result<()> {
    let prompt = helper
        .startup_prompt
        .as_ref()
        .ok_or_else(|| anyhow!("Start prompt absent"))?;
    let public_scope = match prompt.public_scope {
        Some(AgentExecutionScope::Host) => "host",
        Some(AgentExecutionScope::World) => "world",
        None => bail!("Start public scope absent"),
    };
    let prompt_digest = helper_start_digest(
        "substrate.a1.3-p1.public-start-prompt.v1",
        &[prompt.prompt_text.as_bytes()],
    );
    ensure!(
        prompt.request_key_sha256.as_ref() == Some(&transaction.request_key_sha256)
            && prompt.prompt_sha256.as_ref() == Some(&transaction.prompt_sha256)
            && prompt.public_backend_id.as_ref() == Some(&transaction.public_backend_id)
            && public_scope == transaction.public_scope
            && prompt_digest == transaction.prompt_sha256,
        "helper prompt or public identity mismatch"
    );
    // Exact JSON and framing from agents_cmd::public_start_request_key. This
    // also binds the complete presented attach contract, including its policy.
    let request = serde_json::json!({
        "schema_version": 1,
        "descriptor": &helper.descriptor,
        "workspace_root": &helper.session.workspace_root,
        "host_attach_contract": &helper.host_attach_contract,
        "public_backend_id": &transaction.public_backend_id,
        "public_scope": public_scope,
        "prompt_sha256": &prompt_digest,
    });
    let bytes = serde_json::to_vec(&request)?;
    ensure!(
        helper_start_digest("substrate.a1.3-p1.public-start-request.v1", &[&bytes])
            == transaction.request_key_sha256,
        "helper full original request mismatch"
    );
    Ok(())
}

fn compare_helper_resume(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    intent: &HostSessionTransitionIntentV3,
    helper: &HiddenOwnerHelperLaunchPlan,
) -> Result<()> {
    ensure!(
        intent.mode == HostSessionTransitionModeV1::ResumeOneTurn
            && helper.host_attach_contract.is_none()
            && intent.source_authoritative_participant_id.is_some()
            && helper.participant.resumed_from_participant_id
                == intent.source_authoritative_participant_id,
        "helper Resume source or contract mismatch"
    );
    let prompt = helper
        .startup_prompt
        .as_ref()
        .ok_or_else(|| anyhow!("Resume input absent"))?;
    ensure!(
        prompt.request_key_sha256.is_none()
            && prompt.prompt_sha256.is_none()
            && prompt.public_backend_id.is_none()
            && prompt.public_scope.is_none(),
        "Resume contains unissued public Start metadata"
    );
    let reference = intent
        .transition_input_ref
        .as_ref()
        .ok_or_else(|| anyhow!("genuine Resume input absent"))?;
    let context = ObjectVerificationContextV1 {
        intent_id: intent.intent_id.clone(),
        run_id: intent.run_id.clone(),
        parent_intent: None,
    };
    let root = snapshot.captured_root();
    let bytes = snapshot
        .read_typed_object(
            root.authority_store_id(),
            root.root_revision(),
            reference,
            Some(&context),
        )
        .map_err(|_| anyhow!("helper Resume input authentication refused"))?;
    ensure!(
        bytes == prompt.prompt_text.as_bytes(),
        "helper Resume input mismatch"
    );
    let native_session = helper_resume_native_session(snapshot, intent)?;
    ensure!(
        helper.participant.internal_uaa_session_id.as_ref() == Some(&native_session),
        "helper consumed native session mismatch"
    );
    Ok(())
}

fn compare_helper_successor_intent(
    actual: &HostSessionTransitionIntentV3,
    supplied: &HostSessionTransitionIntentV3,
) -> Result<()> {
    // Every immutable declaration member is compared. The asserted envelope is
    // never used as a sensitive object's authenticated parent or read context.
    ensure!(
        actual.schema_version == supplied.schema_version
            && actual.intent_id == supplied.intent_id
            && actual.issuer_request_id == supplied.issuer_request_id
            && actual.mode == supplied.mode
            && actual.authority_precondition == supplied.authority_precondition
            && actual.orchestration_session_id == supplied.orchestration_session_id
            && actual.shell_trace_session_id == supplied.shell_trace_session_id
            && actual.caller == supplied.caller
            && actual.source_authoritative_participant_id
                == supplied.source_authoritative_participant_id
            && actual.target_authoritative_participant_id
                == supplied.target_authoritative_participant_id
            && actual.target_participant_lease_token_ref
                == supplied.target_participant_lease_token_ref
            && actual.run_id == supplied.run_id
            && actual.resulting_authoritative_lineage == supplied.resulting_authoritative_lineage
            && actual.workspace_binding == supplied.workspace_binding
            && actual.world_binding == supplied.world_binding
            && actual.descriptor_ref == supplied.descriptor_ref
            && actual.host_attach_contract_ref == supplied.host_attach_contract_ref
            && actual.resume_handle_ref == supplied.resume_handle_ref
            && actual.transition_input_ref == supplied.transition_input_ref
            && actual.post_turn_disposition == supplied.post_turn_disposition
            && actual.transport_payload_ref == supplied.transport_payload_ref
            && actual.payload_commitment == supplied.payload_commitment
            && actual.issued_at == supplied.issued_at
            && actual.expires_at == supplied.expires_at,
        "supplied immutable successor differs from genuine stored intent"
    );
    let (
        HostSessionTransitionIntentStateV3::Applied {
            claim_id,
            claimant_attempt_id,
            authority_revision_before,
            authority_revision_after,
            active_authoritative_participant_id,
            resulting_posture,
            authority_record_commitment,
            application_result_ref,
            applied_at,
            ..
        },
        HostSessionTransitionIntentStateV3::Applied {
            claim_id: other_claim,
            claimant_attempt_id: other_attempt,
            authority_revision_before: other_before,
            authority_revision_after: other_after,
            active_authoritative_participant_id: other_actor,
            resulting_posture: other_posture,
            authority_record_commitment: other_commitment,
            application_result_ref: other_ref,
            applied_at: other_at,
            ..
        },
    ) = (&actual.state, &supplied.state)
    else {
        bail!("supplied successor application absent")
    };
    ensure!(
        claim_id == other_claim
            && claimant_attempt_id == other_attempt
            && authority_revision_before == other_before
            && authority_revision_after == other_after
            && active_authoritative_participant_id == other_actor
            && resulting_posture == other_posture
            && authority_record_commitment == other_commitment
            && application_result_ref == other_ref
            && applied_at == other_at,
        "supplied successor application differs"
    );
    // intent_revision, updated_at, handoff/release and startup/post-turn state
    // evolve after the launcher captured its Applied envelope. Only the genuine
    // current snapshot establishes those states, including Accepted input.
    Ok(())
}

fn compare_helper_descriptor(
    actual: &AgentDescriptorV1,
    helper: &ResolvedRuntimeDescriptor,
) -> Result<()> {
    let backend_kind = match helper.backend_kind {
        ResolvedRuntimeBackendKind::Codex => RuntimeBackendKindV1::Codex,
        ResolvedRuntimeBackendKind::ClaudeCode => RuntimeBackendKindV1::ClaudeCode,
    };
    ensure!(
        actual.agent_id == helper.agent_id
            && actual.backend_id == helper.backend_id
            && actual.backend_kind == backend_kind
            && actual.protocol == helper.protocol
            && actual.execution_scope == AgentExecutionScopeV1::Host
            && helper.execution_scope == AgentExecutionScope::Host
            && actual.binary_path == helper.binary_path,
        "helper descriptor differs"
    );
    Ok(())
}

fn helper_resume_native_session(
    snapshot: &CurrentHostRunReadViewV1<'_, '_>,
    intent: &HostSessionTransitionIntentV3,
) -> Result<String> {
    // authenticate_current_host_run already verified the exact source-owned
    // consumed handle, its producer/native join and current Pending application.
    let reference = intent
        .resume_handle_ref
        .as_ref()
        .ok_or_else(|| anyhow!("consumed helper handle absent"))?;
    let native = match reference.schema_version {
        1 => {
            read_run_object::<ResumeHandleHashInputV1>(snapshot, reference, None)?
                .internal_uaa_session_id
        }
        2 | 3 => {
            let handle = if reference.schema_version == 2 {
                super::start_continuity::DecodedStartContinuationHandleV1::V2(Box::new(
                    read_run_object::<StartContinuationHandleHashInputV2>(
                        snapshot, reference, None,
                    )?,
                ))
            } else {
                super::start_continuity::DecodedStartContinuationHandleV1::WorldV3(Box::new(
                    read_run_object::<StartContinuationHandleHashInputV3>(
                        snapshot, reference, None,
                    )?,
                ))
            };
            handle.internal_uaa_session_id.clone()
        }
        _ => bail!("consumed helper handle version unsupported"),
    };
    ensure!(!native.is_empty(), "consumed helper native session empty");
    Ok(native)
}

// These pure algorithms preserve the existing public Start producer's domains,
// part framing and deterministic listener pathname. They perform no I/O.
fn helper_start_digest(domain: &str, parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn helper_launch_fragment(id: &str) -> String {
    let normalized = id
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if normalized.len() <= 12 {
        return normalized;
    }
    format!(
        "{}{}",
        &normalized[..6],
        &normalized[normalized.len() - 6..]
    )
}

fn helper_launch_stream_path(home: &str, session: &str, participant: &str) -> PathBuf {
    let session_fragment = helper_launch_fragment(session);
    let participant_fragment = helper_launch_fragment(participant);
    let socket_name = format!("{session_fragment}-{participant_fragment}.startup.sock");
    let preferred = Path::new(home)
        .join("runtime-control/durable-start")
        .join("startup")
        .join(&socket_name);
    if preferred.as_os_str().len() > 100 {
        return PathBuf::from("/tmp")
            .join("substrate-durable-start")
            .join("startup")
            .join(socket_name);
    }
    preferred
}

/// Narrow, borrowed composition of an authenticated current run and its exact
/// original Start. This view is neither transferable nor a Host effect permit.
pub(crate) struct CurrentHostGatewayRunObservationV1<'authority> {
    current_run: AuthenticatedCurrentHostRunV1<'authority>,
    original_start: HostSessionTransitionIntentV2,
    original_descriptor: AgentDescriptorV1,
    current_policy: PolicyObjectHashInputV1,
}

impl CurrentHostGatewayRunObservationV1<'_> {
    pub(crate) fn current_run(&self) -> &AuthenticatedCurrentHostRunV1<'_> {
        &self.current_run
    }

    pub(crate) fn original_start(&self) -> &HostSessionTransitionIntentV2 {
        &self.original_start
    }

    pub(crate) fn original_descriptor(&self) -> &AgentDescriptorV1 {
        &self.original_descriptor
    }

    pub(crate) fn current_policy(&self) -> &PolicyObjectHashInputV1 {
        &self.current_policy
    }

    pub(crate) fn revalidate(&self) -> Result<()> {
        self.current_run.revalidate()
    }
}

impl HostSessionAuthority {
    /// Obtain a genuine held observation from the existing typed snapshot, then
    /// invoke the mandatory verifier. Semantic refusal is finalized as carefully
    /// as success. No executing resolver, reconciliation or upgrade is performed.
    pub(crate) fn observe_current_host_gateway_run_v1<'authority>(
        &'authority self,
        session: &str,
        participant: &str,
        backend: &str,
        run: &str,
    ) -> Result<CurrentHostGatewayRunObservationV1<'authority>> {
        ensure!(
            [session, participant, backend, run]
                .iter()
                .all(|value| !value.is_empty()),
            "current Host gateway claim must be exact and nonempty"
        );
        let (facts, original_start, original_descriptor, current_policy) =
            store::with_existing_current_host_run_snapshot_opened(
                self.trusted_root(),
                |snapshot| {
                    Ok((|| -> Result<_> {
                        let captured = snapshot.captured_root();
                        let read = |reference: &AuthorityObjectRefV1,
                                context: Option<&ObjectVerificationContextV1>| {
                        snapshot
                            .read_typed_object(
                                captured.authority_store_id(),
                                captured.root_revision(),
                                reference,
                                context,
                            )
                            .map_err(|error| format!("{error:?}"))
                    };
                        let observation = match captured {
                            VersionedStateRoot::V1(_) => {
                                bail!("current Host gateway requires existing V2 or V3")
                            }
                            VersionedStateRoot::V2(root) => {
                                self.resolve_current_exact_from_preserved_start_root_with_reader(
                                    root, session, None, &read,
                                )
                                .map_err(|error| anyhow!("{error}"))?
                                .observation
                            }
                            VersionedStateRoot::V3(root) => {
                                let history = exact_v3_authority_history_with_reader(
                                    root,
                                    session,
                                    |reference| read(reference, None),
                                )
                                .map_err(|error| anyhow!("{error}"))?;
                                history
                                    .last_key_value()
                                    .ok_or_else(|| anyhow!("current Host gateway history absent"))?
                                    .1
                                    .observation()
                            }
                        };
                        let (facts, workspace) = authenticate_current_host_run(
                            self,
                            snapshot,
                            session,
                            participant,
                            backend,
                            run,
                            &observation,
                        )?;
                        let original = match captured {
                            VersionedStateRoot::V2(root) => root
                                .transition_intent_map
                                .get(&facts.original_start_intent_id),
                            VersionedStateRoot::V3(root) => root
                                .transition_intent_map
                                .get(&facts.original_start_intent_id),
                            VersionedStateRoot::V1(_) => None,
                        }
                        .ok_or_else(|| anyhow!("authenticated original Start absent"))?
                        .clone();
                        ensure!(
                            original.workspace_binding == facts.workspace_binding
                                && original.descriptor_ref == facts.descriptor_ref,
                            "ordinary gateway original/current workspace or descriptor differs"
                        );
                        let descriptor: AgentDescriptorHashInputV1 =
                            read_run_object(snapshot, &original.descriptor_ref, None)?;
                        let policy: PolicyObjectHashInputV1 =
                            read_run_object(snapshot, &facts.policy_ref, None)?;
                        workspace.revalidate().map_err(|error| anyhow!("{error}"))?;
                        Ok((facts, original, descriptor.descriptor, policy))
                    })())
                },
            )
            .map_err(|error| anyhow!("current Host gateway snapshot refused: {error:?}"))??;

        // This is deliberately the reviewed verifier, supplied with the genuine
        // observation obtained above. No caller-asserted observation is accepted.
        let current_run = self.verify_current_host_run_v1(
            session,
            participant,
            backend,
            run,
            &facts.observation,
        )?;
        ensure!(
            current_run.facts == facts,
            "current Host gateway changed across observation and verification"
        );
        let result = CurrentHostGatewayRunObservationV1 {
            current_run,
            original_start,
            original_descriptor,
            current_policy,
        };
        result.revalidate()?;
        Ok(result)
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
                let (produced, actor, producer_intent, producer_run, native) =
                    match completion_ref.schema_version {
                        3 => {
                            let completion: PostTurnCompletionHashInputV3 =
                                read_run_object(snapshot, completion_ref, None)?;
                            let event: PostTurnProtocolEventHashInputV2 =
                                read_run_object(snapshot, &completion.protocol_event_ref, None)?;
                            let native = event
                                .native_terminal
                                .ok_or_else(|| anyhow!("source handle native terminal absent"))?;
                            (
                                completion.produced_resume_handle_ref,
                                completion.binding.authoritative_participant_id,
                                completion.binding.intent_id,
                                completion.binding.run_id,
                                native,
                            )
                        }
                        4 => {
                            let completion: PostTurnCompletionHashInputV4 =
                                read_run_object(snapshot, completion_ref, None)?;
                            ensure!(
                                source.world_binding.as_ref()
                                    == Some(&completion.binding.world_binding),
                                "World source handle generation differs"
                            );
                            (
                                completion.produced_resume_handle_ref,
                                completion.binding.authoritative_participant_id,
                                completion.binding.intent_id,
                                completion.binding.run_id,
                                completion.native_terminal,
                            )
                        }
                        _ => continue,
                    };
                if produced.as_ref() != Some(reference) {
                    continue;
                }
                ensure!(
                    *authority_revision_after <= source.authority_revision
                        && actor == *owner
                        && producer_intent == parent.intent_id
                        && producer_run == parent.run_id
                        && h.internal_uaa_session_id == native.native_session_id
                        && h.backend_id == native.backend_id
                        && h.protocol == native.protocol,
                    "source handle producer or native identity mismatch"
                );
                producers += 1;
            }
            ensure!(producers == 1, "source handle producer absent or ambiguous");
        }
        2 | 3 => {
            let h = if reference.schema_version == 2 {
                super::start_continuity::DecodedStartContinuationHandleV1::V2(Box::new(
                    read_run_object::<StartContinuationHandleHashInputV2>(
                        snapshot, reference, None,
                    )?,
                ))
            } else {
                super::start_continuity::DecodedStartContinuationHandleV1::WorldV3(Box::new(
                    read_run_object::<StartContinuationHandleHashInputV3>(
                        snapshot, reference, None,
                    )?,
                ))
            };
            ensure!(h.world_controller().is_none_or(|world| source.world_binding.as_ref() == Some(&world.world_binding)),
                "World continuation source placement differs");
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
pub(crate) fn verify_selected_helper_snapshot_finalizers_for_test(
    proof: &AuthenticatedCurrentHostRunV1<'_>,
    helper: &HiddenOwnerHelperLaunchPlan,
    selected: &crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1,
) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
    let home = Path::new(
        &proof
            .facts
            .workspace_binding
            .authority_store_root
            .physical_path,
    );
    let path = home.join("authority-v1/state-root-v1.json");
    let original = std::fs::read(&path)?;
    for refusal in [false, true] {
        for phase in [Phase::BeforeCustodyFinalize, Phase::AfterCustodyFinalize] {
            let mut presented = helper.clone();
            if refusal {
                presented.participant.lease_token.push_str("tampered");
            }
            let changed = path.clone();
            let fired = std::rc::Rc::new(std::cell::Cell::new(false));
            let observed = std::rc::Rc::clone(&fired);
            hook(phase, move || {
                let mut value: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(&changed).unwrap()).unwrap();
                value["root_revision"] =
                    serde_json::json!(value["root_revision"].as_u64().unwrap() + 1);
                std::fs::write(changed, canonical_json::to_vec(&value).unwrap()).unwrap();
                observed.set(true);
            });
            let result = proof.verify_selected_helper_launch_v1(&presented, None, selected);
            assert!(fired.get(), "the requested snapshot finalizer must execute");
            assert_eq!(
                result.unwrap_err().to_string(),
                "helper launch snapshot integrity refused"
            );
            std::fs::write(&path, &original)?;
        }
    }

    // Actual current snapshot owns original issuer/application/transaction joins.
    // Mutating a persisted binding is a refusal fixture, never a new authority.
    {
        let pointer = "/authority_store_id";
        let mut value: serde_json::Value = serde_json::from_slice(&original)?;
        *value.pointer_mut(pointer).unwrap() = serde_json::json!("as_wrong_selected_store");
        std::fs::write(&path, canonical_json::to_vec(&value)?)?;
        assert!(proof
            .verify_selected_helper_launch_v1(helper, None, selected)
            .is_err());
        std::fs::write(&path, &original)?;
    }
    for field in ["start_application_result_ref", "start_payload_commitment"] {
        let mut value: serde_json::Value = serde_json::from_slice(&original)?;
        let transaction = value["start_transaction_map"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap();
        if field == "start_application_result_ref" {
            transaction[field]["ref_id"] = serde_json::json!("ao_wrong_original_application");
        } else {
            // Reuse a structurally valid different stored commitment; no guess at its encoding.
            transaction[field] = transaction["start_application_result_ref"]["commitment"].clone();
        }
        std::fs::write(&path, canonical_json::to_vec(&value)?)?;
        assert!(
            proof
                .verify_selected_helper_launch_v1(helper, None, selected)
                .is_err(),
            "{field}"
        );
        std::fs::write(&path, &original)?;
    }
    let root_value: serde_json::Value = serde_json::from_slice(&original)?;
    let transaction_id = root_value["start_transaction_map"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    for (field, replacement) in [
        ("start_issuer_request_id", serde_json::json!("req_other")),
        ("start_run_id", serde_json::json!("run_other")),
        ("start_intent_id", serde_json::json!("hti_other")),
        (
            "authoritative_participant_id",
            serde_json::json!("ash_other"),
        ),
        ("public_backend_id", serde_json::json!("cli:other")),
        ("shell_trace_session_id", serde_json::json!("trace_other")),
        ("request_key_sha256", serde_json::json!("0".repeat(64))),
    ] {
        let mut value = root_value.clone();
        value["start_transaction_map"][&transaction_id][field] = replacement;
        std::fs::write(&path, canonical_json::to_vec(&value)?)?;
        assert!(
            proof
                .verify_selected_helper_launch_v1(helper, None, selected)
                .is_err(),
            "{field}"
        );
        std::fs::write(&path, &original)?;
    }

    // A valid root temporary is eligible executing-reader reconciliation input.
    // The retained read-only scope tolerates eligible material without consuming,
    // publishing or upgrading it; semantic tamper must still refuse.
    let root: serde_json::Value = serde_json::from_slice(&original)?;
    let temporary = home.join("authority-v1/tmp").join(format!(
        "root--r{}--{}.tmp",
        root["root_revision"].as_u64().unwrap(),
        "a".repeat(32)
    ));
    std::fs::write(&temporary, &original)?;
    std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
    let metadata = std::fs::metadata(&temporary)?;
    for refusal in [false, true] {
        let mut presented = helper.clone();
        if refusal {
            presented.participant.lease_token.push_str("tampered");
        }
        let comparison = proof.verify_selected_helper_launch_v1(&presented, None, selected);
        assert_eq!(comparison.is_err(), refusal);
        assert_eq!(std::fs::read(&path)?, original);
        assert_eq!(std::fs::read(&temporary)?, original);
        use std::os::unix::fs::MetadataExt;
        assert_eq!(std::fs::metadata(&temporary)?.ino(), metadata.ino());
    }
    std::fs::remove_file(temporary)?;
    proof.verify_selected_helper_launch_v1(helper, None, selected)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn exercise_selected_helper_resumes_for_test(
    authority: &HostSessionAuthority,
    selected: &crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1,
    helper: &HiddenOwnerHelperLaunchPlan,
    interleaved_r0: bool,
) -> Result<()> {
    tests::exercise_selected_helper_resumes_for_test(authority, selected, helper, interleaved_r0)
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

    pub(super) fn exercise_selected_helper_resumes_for_test(
        authority: &HostSessionAuthority,
        selected: &crate::execution::agent_runtime::native_launch_context::AuthenticatedOrdinarySelectionV1,
        start_helper: &HiddenOwnerHelperLaunchPlan,
        interleaved_r0: bool,
    ) -> Result<()> {
        use crate::execution::agent_runtime::control::register_durable_start_startup_prompt_listener;
        use anyhow::Context as _;
        let key = start_helper
            .startup_prompt
            .as_ref()
            .unwrap()
            .request_key_sha256
            .as_ref()
            .unwrap();
        let transaction = authority.retryable_start_transaction(key)?.unwrap();
        let root = authority.read_preserved_start_root_v2()?;
        let original = root.transition_intent_map[&transaction.start_intent_id].clone();
        let world_controller = if original.world_binding.is_some() {
            Some(super::super::world_bound_controller::AuthenticatedWorldBoundControllerRunV1::authenticate_received_helper(
                std::sync::Arc::new(selected.refresh_original_selection()?), start_helper, None,
            )?)
        } else {
            None
        };
        let registration = EstablishStartContinuationRequestV1 {
            start_transaction_id: transaction.transaction_id.clone(),
            request_key_sha256: key.clone(),
            authority_store_id: transaction.authority_store_id.clone(),
            orchestration_session_id: transaction.orchestration_session_id.clone(),
            intent_id: original.intent_id.clone(),
            issuer_request_id: original.issuer_request_id.clone(),
            payload_commitment: original.payload_commitment.clone(),
            application_result_ref: transaction.start_application_result_ref.clone(),
            run_id: original.run_id.clone(),
            expected_authority_revision: transaction.start_authority_revision,
            authoritative_participant_id: original.target_authoritative_participant_id.clone(),
            backend_id: start_helper.descriptor.backend_id.clone(),
            protocol: start_helper.descriptor.protocol.clone(),
            exchange_id: "exchange-start-1".into(),
            exchange_sequence: 1,
            provider_event_kind: "turn.exchange_opened".into(),
            exchange_evidence_sha256: "a".repeat(64),
            internal_uaa_session_id: "uaa-session-start-1".into(),
            observed_at: at(1, 20),
        };
        if original.world_binding.is_none() {
            let current =
                authority.resolve_current_exact(&original.orchestration_session_id, None)?;
            authority.enroll_direct_host_start(&current)?;
        }
        authority.mark_start_prompt_submission_no_replay_barrier(
            &transaction.transaction_id,
            key,
            at(1, 15),
        )?;
        let held = authority.observe_current_host_gateway_run_v1(
            &original.orchestration_session_id,
            &original.target_authoritative_participant_id,
            &start_helper.descriptor.backend_id,
            &original.run_id,
        )?;
        assert!(
            held.current_run()
                .verify_selected_helper_launch_v1(start_helper, None, selected)
                .is_err(),
            "deliberate barrier is outside Start launch comparison"
        );
        let StartContinuationOutcomeV1::Applied(registered) =
            authority.establish_start_continuation(&registration)?
        else {
            anyhow::bail!("real continuation required");
        };
        if world_controller.is_some() && interleaved_r0 {
            register_world_fixture_retained_at(
                authority,
                &original.orchestration_session_id,
                "during-start",
                at(1, 25),
            )?;
        }
        let settlement_request = clean_start_settlement_request(
            &registration,
            registered.resume_handle_ref,
            authority
                .resolve_current_exact(&original.orchestration_session_id, None)?
                .observation
                .authority_revision,
        );
        let outcome = if let Some(controller) = world_controller.as_ref() {
            let active_selection = controller.selection().refresh_original_selection()?;
            let publication =
                super::super::world_bound_controller::authenticate_ordinary_world_publication(
                    &active_selection,
                )?;
            let e2_publication =
                super::super::world_bound_controller::authenticate_ordinary_world_publication(
                    &active_selection,
                )?;
            let held =
                super::super::store::hold_existing_accepted_work_authority_snapshot(authority)
                    .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let sources = held
                .capture_world_controller_completion_sources(&original.orchestration_session_id)
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let cut = crate::execution::agent_runtime::obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut(
                &sources, controller.world_binding(),
            )?;
            let outcome = authority
                .settle_ordinary_world_controller_start(&settlement_request, controller, &cut)
                .context("apply issued World Start with held physical sources")?;
            assert!(
                matches!(
                    authority
                        .settle_ordinary_world_controller_start(
                            &settlement_request,
                            controller,
                            &cut
                        )
                        .context("join exact issued World Start retry")?,
                    StartTurnSettlementOutcomeV1::Joined(_)
                ),
                "exact World Start retry must join its immutable completion"
            );
            let home = Path::new(
                &original
                    .workspace_binding
                    .authority_store_root
                    .physical_path,
            );
            let b1 = super::super::store::WorldWorkReceiptRegistryStorageV1::bind(
                home,
                authority.trusted_root().identity(),
                controller.selection().authority_store_id(),
            )?;
            unchanged(home, || {
                assert!(
                    b1.begin_transaction_with_ordinary_publication(&publication)
                        .is_err(),
                    "a pre-settlement B1 producer cannot publish after controller settlement"
                );
            });
            unchanged(home, || {
                assert!(super::super::store::dispatch_policy_commitment_storage_for_ordinary_publication(authority, e2_publication).is_err(),
                    "a pre-settlement E2 producer cannot initialize or reconcile after controller settlement");
            });
            unchanged(home, || {
                let closed = controller.selection().refresh_original_selection().unwrap();
                assert!(
                    super::super::world_bound_controller::authenticate_ordinary_world_publication(
                        &closed
                    )
                    .is_err(),
                    "original-selection refresh cannot regrant new work after settlement"
                );
            });
            outcome
        } else {
            authority.settle_start_turn(&settlement_request)?
        };
        let StartTurnSettlementOutcomeV1::Applied(settled) = outcome else {
            anyhow::bail!("real settlement required");
        };
        let home = Path::new(
            &original
                .workspace_binding
                .authority_store_root
                .physical_path,
        );
        let public_posture = unchanged(home, || {
            crate::execution::agent_runtime::control::verify_public_start_continuity_settled(
                &original.orchestration_session_id,
            )
        })
        .context("actual public Start continuity verifier must admit the committed schema")?;
        assert_eq!(public_posture, settled.resulting_posture);
        let mut handle = settled.continuation_resume_handle_ref;
        for turn in 1..=2 {
            let current =
                authority.resolve_current_exact(&original.orchestration_session_id, None)?;
            let mut request = resume_successor_request(
                &current,
                handle.clone(),
                &format!("selected-resume-{turn}"),
            );
            request.intent_id = format!("selected-resume-intent-{turn}");
            request.issuer_request_id = format!("selected-resume-request-{turn}");
            request.run_id = format!("selected-resume-run-{turn}");
            let applied = apply_successor(authority, &request, 2 * turn);
            let listener = register_durable_start_startup_prompt_listener(
                &applied.orchestration_session_id,
                &applied.target_authoritative_participant_id,
            )?;
            let mut helper = start_helper.clone();
            helper.mode = OwnerHelperMode::ResumeOneTurn;
            helper.host_attach_contract = None;
            helper.participant = HiddenOwnerHelperParticipantPlan {
                participant_id: applied.target_authoritative_participant_id.clone(),
                lease_token: String::from_utf8(request.target_participant_lease_token.clone())?,
                run_id: applied.run_id.clone(),
                resumed_from_participant_id: applied.source_authoritative_participant_id.clone(),
                internal_uaa_session_id: Some("uaa-session-start-1".into()),
            };
            helper.startup_prompt = Some(HiddenOwnerHelperStartupPromptPlan {
                prompt_text: String::from_utf8(request.transition_input.clone().unwrap())?,
                stream_path: listener.path().to_path_buf(),
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: None,
                public_scope: None,
            });
            let observed = authority.observe_current_host_gateway_run_v1(
                &applied.orchestration_session_id,
                &applied.target_authoritative_participant_id,
                &helper.descriptor.backend_id,
                &applied.run_id,
            )?;
            let home = Path::new(&applied.workspace_binding.authority_store_root.physical_path);
            unchanged(home, || {
                observed.current_run().verify_selected_helper_launch_v1(
                    &helper,
                    Some(&applied),
                    selected,
                )
            })?;
            let mut wrong = helper.clone();
            wrong.participant.internal_uaa_session_id = Some("wrong-native".into());
            unchanged(home, || {
                assert!(observed
                    .current_run()
                    .verify_selected_helper_launch_v1(&wrong, Some(&applied), selected)
                    .is_err())
            });
            let mut wrong = applied.clone();
            wrong.issuer_request_id.push_str("-tampered");
            unchanged(home, || {
                assert!(observed
                    .current_run()
                    .verify_selected_helper_launch_v1(&helper, Some(&wrong), selected)
                    .is_err())
            });
            let resume_controller = if original.world_binding.is_some() {
                Some(super::super::world_bound_controller::AuthenticatedWorldBoundControllerRunV1::authenticate_received_helper(
                    std::sync::Arc::new(selected.refresh_original_selection()?), &helper, Some(&applied),
                ).context("authenticate genuine World Resume fixture")?)
            } else {
                None
            };
            authority.accept_transition_input(&AcceptTransitionInputRequestV1 {
                intent_id: applied.intent_id.clone(),
                issuer_request_id: applied.issuer_request_id.clone(),
                payload_commitment: applied.payload_commitment.clone(),
                accepting_participant_id: applied.target_authoritative_participant_id.clone(),
                accepted_at: at(2 * turn, 30),
            })?;
            let accepted = authority.observe_current_host_gateway_run_v1(
                &applied.orchestration_session_id,
                &applied.target_authoritative_participant_id,
                &helper.descriptor.backend_id,
                &applied.run_id,
            )?;
            unchanged(home, || {
                accepted.current_run().verify_selected_helper_launch_v1(
                    &helper,
                    Some(&applied),
                    selected,
                )
            })?;
            // Comparison above is read-only; Accepted remains Accepted and
            // supplies no second delivery/transport permit.
            handle = if let Some(controller) = &resume_controller {
                if interleaved_r0 {
                    register_world_fixture_retained_at(
                        authority,
                        &original.orchestration_session_id,
                        &format!("during-resume-{turn}"),
                        at(2 * turn, 35),
                    )?;
                }
                settle_selected_fixture_world_resume(
                    authority,
                    controller,
                    &applied,
                    2 * turn,
                    &helper,
                    interleaved_r0,
                )
                .context("settle genuine World Resume fixture")?
            } else {
                settle_selected_fixture_resume(authority, &applied, 2 * turn, &helper)
            };
            assert_eq!(
                handle.schema_version, 1,
                "Resume2 consumes real current-owner V1 handle"
            );
            let path = listener.path().to_path_buf();
            let inode = std::fs::symlink_metadata(&path)?;
            drop(listener);
            let retained = std::fs::symlink_metadata(&path)?;
            use std::os::unix::fs::MetadataExt;
            ensure!(
                retained.dev() == inode.dev() && retained.ino() == inode.ino(),
                "listener substituted"
            );
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    fn register_world_fixture_retained_at(
        authority: &HostSessionAuthority,
        session: &str,
        suffix: &str,
        registered_at: TimestampV1,
    ) -> Result<()> {
        let before = authority.resolve_current_exact(session, None)?.observation;
        let plan = RetainedWorkerRegistrationPlanV1 {
            registration_request_id: format!("world-mixed-registration-{suffix}"),
            orchestration_session_id: session.into(),
            expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                authority_store_id: before.authority_store_id,
                authority_revision: before.authority_revision,
                authority_record_commitment: before.authority_record_commitment,
            },
            retained_participant_id: format!("world-mixed-worker-{suffix}"),
            descriptor: AgentDescriptorV1 {
                schema_version: 1,
                agent_id: format!("world-mixed-agent-{suffix}"),
                backend_id: "cli:codex-world".into(),
                backend_kind: RuntimeBackendKindV1::Codex,
                protocol: "substrate.agent.session".into(),
                execution_scope: AgentExecutionScopeV1::World,
                binary_path: "/usr/bin/codex".into(),
            },
            internal_uaa_session_id: format!("world-mixed-native-{suffix}"),
        };
        let reserved =
            RetainedWorkerRuntime.reserve_registration_at(authority, &plan, registered_at, None)?;
        RetainedWorkerRuntime.publish_reserved_object_graph(authority, &reserved)?;
        authority.apply_reserved_retained_worker_registration(&reserved)?;
        let after = authority.resolve_current_exact(session, None)?;
        ensure!(
            after.authority.authority_revision == plan.expected_authority.authority_revision + 1
        );
        ensure!(after
            .authority
            .authoritative_participant_lineage
            .contains(&plan.retained_participant_id));
        // This is real R0 publication only. No execution, Registered
        // acknowledgement, admission or Routable state is manufactured.
        Ok(())
    }

    fn settle_selected_fixture_world_resume(
        authority: &HostSessionAuthority,
        controller: &super::super::world_bound_controller::AuthenticatedWorldBoundControllerRunV1,
        applied: &HostSessionTransitionIntentV3,
        minute: u32,
        helper: &HiddenOwnerHelperLaunchPlan,
        interleaved_r0: bool,
    ) -> Result<AuthorityObjectRefV1> {
        // Genuine issued HSA/helper/Accepted producers; native observations are
        // explicit component fixtures, with no provider or execution claim.
        let terminal = DirectHostNativeTerminalEvidenceV1 {
            schema_version: 1,
            backend_id: helper.descriptor.backend_id.clone(),
            protocol: helper.descriptor.protocol.clone(),
            native_session_id: "uaa-session-start-1".into(),
            thread_id: "uaa-session-start-1".into(),
            turn_id: format!("fixture-world-turn-{minute}"),
            sequence: DirectHostNativeSequenceV1::Codex {
                resumed_thread_id: "uaa-session-start-1".into(),
                started_turn_id: format!("fixture-world-turn-{minute}"),
            },
            terminal_event_kind: "turn.completed".into(),
            raw_event_sha256: "e".repeat(64),
            terminal_ordinal: 1,
        };
        let held = super::super::store::hold_existing_accepted_work_authority_snapshot(authority)
            .map_err(|error| anyhow!("capture issued World Resume source: {error:?}"))?;
        let sources = held
            .capture_world_controller_completion_sources(&applied.orchestration_session_id)
            .map_err(|error| anyhow!("capture issued World Resume attention: {error:?}"))?;
        let cut = crate::execution::agent_runtime::obligation_ledger::world_bound_controller::authenticate_world_controller_work_attention_cut(&sources, controller.world_binding())?;
        assert_eq!(
            cut.cut().unclassified_hsa_retained_participant_ids.len(),
            if interleaved_r0 {
                1 + minute as usize / 2
            } else {
                0
            }
        );
        assert_eq!(
            cut.cut().has_unresolved(),
            interleaved_r0,
            "unclassified R0 never supplies bootstrap closure"
        );
        let outcome = authority.settle_ordinary_world_controller_resume(
            controller,
            &terminal,
            HostPostTurnProtocolEventKindV1::ResumableClean,
            &at(minute, 40),
            &cut,
        )?;
        assert!(matches!(outcome, PostTurnResolutionOutcomeV1::Applied(_)));
        let home = Path::new(&applied.workspace_binding.authority_store_root.physical_path);
        unchanged(home, || {
            assert!(matches!(
                authority
                    .settle_ordinary_world_controller_resume(
                        controller,
                        &terminal,
                        HostPostTurnProtocolEventKindV1::ResumableClean,
                        &at(minute, 40),
                        &cut
                    )
                    .unwrap(),
                PostTurnResolutionOutcomeV1::Joined(_)
            ));
        });
        let mut wrong = terminal.clone();
        wrong.raw_event_sha256 = "f".repeat(64);
        unchanged(home, || {
            assert!(authority
                .settle_ordinary_world_controller_resume(
                    controller,
                    &wrong,
                    HostPostTurnProtocolEventKindV1::ResumableClean,
                    &at(minute, 40),
                    &cut
                )
                .is_err())
        });
        cut.verify_sources()?;
        let root = authority.read_a12b_root()?;
        let intent = &root.successor_transition_intent_map[&applied.intent_id];
        assert!(
            matches!(
                &intent.input_handoff,
                HostSessionTransitionInputHandoffV1::Accepted { .. }
            ),
            "settlement preserves exact Accepted input"
        );
        let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &intent.state else {
            bail!("World Resume lost Applied state");
        };
        let HostSessionPostTurnApplicationV2::Applied { completion_ref, .. } = post_turn.as_ref()
        else {
            bail!("World Resume completion absent");
        };
        assert_eq!(completion_ref.schema_version, 4);
        let completion: PostTurnCompletionHashInputV4 = canonical_json::from_slice(
            &authority.read_authority_object_v2_at(root.root_revision, completion_ref)?,
        )?;
        assert_eq!(
            completion.binding.authoritative_participant_id,
            applied.target_authoritative_participant_id
        );
        assert_eq!(
            completion.resulting_posture(),
            if interleaved_r0 {
                HostSessionPostureV1::AwaitingAttention
            } else {
                HostSessionPostureV1::ParkedResumable
            }
        );
        assert_eq!(
            completion
                .work_attention_cut
                .unclassified_hsa_retained_participant_ids,
            cut.cut().unclassified_hsa_retained_participant_ids
        );
        Ok(completion.produced_resume_handle_ref.unwrap())
    }

    fn settle_selected_fixture_resume(
        authority: &HostSessionAuthority,
        applied: &HostSessionTransitionIntentV3,
        minute: u32,
        helper: &HiddenOwnerHelperLaunchPlan,
    ) -> AuthorityObjectRefV1 {
        // Genuine HSA and inbox producers, with a real exclusively flocked helper
        // description. Native fields below are typed protocol-fixture observations,
        // not evidence of a real native/provider process.
        let helper_plan = helper.clone();
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
            backend_id: helper.descriptor.backend_id.clone(),
            protocol: helper.descriptor.protocol.clone(),
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
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("retained partial capture must fail integrity"),
        };
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

    #[test]
    fn test_host_gateway_observer_registration_refreshes_custody() {
        let (_parent, authority, request, _) = started(true, AgentExecutionScopeV1::Host);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let before = observe(&authority, &request.orchestration_session_id);
        let old = unchanged(home, || {
            authority
                .observe_current_host_gateway_run_v1(
                    &request.orchestration_session_id,
                    &request.target_authoritative_participant_id,
                    "cli:codex",
                    &request.run_id,
                )
                .unwrap()
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
        let fresh = unchanged(home, || {
            authority
                .observe_current_host_gateway_run_v1(
                    &request.orchestration_session_id,
                    &request.target_authoritative_participant_id,
                    "cli:codex",
                    &request.run_id,
                )
                .unwrap()
        });
        assert_eq!(fresh.current_run().run_id(), old.current_run().run_id());
        assert_eq!(
            fresh.current_run().intent_id(),
            old.current_run().intent_id()
        );
        assert_eq!(
            fresh.current_run().transition_mode(),
            HostSessionTransitionModeV1::Start
        );
        unchanged(home, || fresh.revalidate().unwrap());
        unchanged(home, || {
            assert!(authority
                .observe_current_host_gateway_run_v1(
                    &request.orchestration_session_id,
                    &plan.retained_participant_id,
                    "cli:codex-world",
                    &request.run_id,
                )
                .is_err())
        });
    }

    #[test]
    fn test_host_gateway_observer_genuine_attach() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        park_start(&authority, &request, &original);
        let parked = authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap();
        let attach = attach_successor_request(&parked, "verifier-attach");
        let applied = apply_successor(&authority, &attach, 3);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let proof = unchanged(home, || {
            authority
                .observe_current_host_gateway_run_v1(
                    &applied.orchestration_session_id,
                    &applied.target_authoritative_participant_id,
                    "cli:codex",
                    &applied.run_id,
                )
                .unwrap()
        });
        assert_eq!(
            proof.current_run().transition_mode(),
            HostSessionTransitionModeV1::Attach
        );
        assert_eq!(proof.current_run().run_id(), attach.run_id);
        assert_eq!(proof.current_run().original_start_run_id(), request.run_id);
        assert_ne!(
            proof.current_run().application_result_ref(),
            proof.current_run().original_start_application_result_ref()
        );
        unchanged(home, || proof.revalidate().unwrap());
        unchanged(home, || {
            assert!(authority
                .observe_current_host_gateway_run_v1(
                    &request.orchestration_session_id,
                    &request.target_authoritative_participant_id,
                    "cli:codex",
                    &request.run_id
                )
                .is_err())
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
        unchanged(home, || assert!(proof.revalidate().is_err()));
        unchanged(home, || {
            authority
                .observe_current_host_gateway_run_v1(
                    &applied.orchestration_session_id,
                    &applied.target_authoritative_participant_id,
                    "cli:codex",
                    &applied.run_id,
                )
                .unwrap()
                .revalidate()
                .unwrap()
        });
    }

    #[test]
    #[serial_test::serial]
    fn test_host_gateway_observer_two_genuine_resumes() {
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
            let proof = unchanged(home, || {
                authority
                    .observe_current_host_gateway_run_v1(
                        &applied.orchestration_session_id,
                        &applied.target_authoritative_participant_id,
                        "cli:codex",
                        &applied.run_id,
                    )
                    .unwrap()
            });
            assert_eq!(proof.current_run().run_id(), successor.run_id);
            assert_eq!(proof.current_run().original_start_run_id(), request.run_id);
            assert_eq!(
                proof.current_run().original_start_intent_id(),
                request.intent_id
            );
            assert_eq!(
                proof.current_run().transition_mode(),
                HostSessionTransitionModeV1::ResumeOneTurn
            );
            unchanged(home, || proof.revalidate().unwrap());
            unchanged(home, || {
                assert!(authority
                    .observe_current_host_gateway_run_v1(
                        &request.orchestration_session_id,
                        &request.target_authoritative_participant_id,
                        "cli:codex",
                        &request.run_id
                    )
                    .is_err())
            });
            if let Some((participant, run)) = &prior {
                unchanged(home, || {
                    assert!(authority
                        .observe_current_host_gateway_run_v1(
                            &request.orchestration_session_id,
                            participant,
                            "cli:codex",
                            run,
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
            unchanged(home, || {
                authority
                    .observe_current_host_gateway_run_v1(
                        &applied.orchestration_session_id,
                        &applied.target_authoritative_participant_id,
                        "cli:codex",
                        &applied.run_id,
                    )
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
    fn test_host_gateway_observer_root_race_and_refusal_finalize() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        for semantic_error in [false, true] {
            for custody_error in [false, true] {
                let (parent, authority, request, original) =
                    started(false, AgentExecutionScopeV1::Host);
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
                    .observe_current_host_gateway_run_v1(
                        &request.orchestration_session_id,
                        &request.target_authoritative_participant_id,
                        if semantic_error {
                            "wrong-backend"
                        } else {
                            "cli:codex"
                        },
                        &request.run_id,
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

        use std::sync::{Arc, Mutex};
        for replace_home in [false, true] {
            let (parent, authority, request, _) = started(false, AgentExecutionScopeV1::Host);
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
            let result = authority.observe_current_host_gateway_run_v1(
                &request.orchestration_session_id,
                &request.target_authoritative_participant_id,
                "cli:codex",
                &request.run_id,
            );
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
    fn test_host_gateway_observer_start_nonupgrading_and_exact_claims() {
        for world in [false, true] {
            let (_parent, authority, request, original) =
                started(world, AgentExecutionScopeV1::Host);
            if world {
                // A World binding alone retains V2. Prepare a genuine public
                // Start transaction before the non-upgrading observation cut.
                prepare_start_continuity_transaction(&authority, &request, &original);
            }
            let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
            let expected_version = if world { 3 } else { 2 };
            let observed = unchanged(home, || {
                authority
                    .observe_current_host_gateway_run_v1(
                        &request.orchestration_session_id,
                        &request.target_authoritative_participant_id,
                        "cli:codex",
                        &request.run_id,
                    )
                    .unwrap()
            });
            assert_eq!(observed.original_start(), &original);
            assert_eq!(
                observed.original_descriptor(),
                &request.start_contract.descriptor
            );
            assert_eq!(observed.current_policy(), &request.start_contract.policy);
            assert_eq!(observed.current_run().run_id(), request.run_id);
            unchanged(home, || observed.revalidate().unwrap());
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
                        .observe_current_host_gateway_run_v1(
                            &values[0], &values[1], &values[2], &values[3],
                        )
                        .is_err())
                });
            }
            let root: serde_json::Value = serde_json::from_slice(
                &fs::read(home.join("authority-v1/state-root-v1.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(root["schema_version"], expected_version);
            let application = observed
                .current_run()
                .original_start_application_result_ref();
            let object = home.join(format!(
                "authority-v1/objects/application-result/v{}/{}.obj",
                application.schema_version, application.ref_id,
            ));
            assert!(object.is_file());
            fs::remove_file(object).unwrap();
            unchanged(home, || {
                assert!(authority
                    .observe_current_host_gateway_run_v1(
                        &request.orchestration_session_id,
                        &request.target_authoritative_participant_id,
                        "cli:codex",
                        &request.run_id,
                    )
                    .is_err())
            });
        }
    }

    #[test]
    fn test_host_gateway_observer_world_and_other_store_refuse() {
        let (_parent, world, request, _) = started(true, AgentExecutionScopeV1::World);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        unchanged(home, || {
            assert!(world
                .observe_current_host_gateway_run_v1(
                    &request.orchestration_session_id,
                    &request.target_authoritative_participant_id,
                    "cli:codex",
                    &request.run_id,
                )
                .is_err())
        });
        let (_parent, other, _) = authority();
        let other_home = other.bootstrap_home();
        let other_path = Path::new(&other_home.identity().unwrap().physical_path);
        unchanged(other_path, || {
            assert!(other
                .observe_current_host_gateway_run_v1(
                    &request.orchestration_session_id,
                    &request.target_authoritative_participant_id,
                    "cli:codex",
                    &request.run_id,
                )
                .is_err())
        });
    }

    #[allow(clippy::type_complexity)]
    fn helper_start_fixture(
        world: bool,
    ) -> (
        tempfile::TempDir,
        HostSessionAuthority,
        IssueHostSessionTransitionRequestV1,
        HostSessionTransitionIntentV2,
        HiddenOwnerHelperLaunchPlan,
    ) {
        use crate::execution::agent_runtime::orchestration_session::{
            HostAttachCapabilities, HostAttachContract, HostAttachExecutionClientStart,
            HostAttachLaunchKnobs, HostAttachModePreference,
        };
        use sha2::{Digest, Sha256};
        // Independent fixture encoding of the existing public producer algorithms.
        // HSA issue/claim/apply/begin operations publish every authority object.
        let (parent, authority, mut binding) = authority();
        let workspace = parent.path().join("workspace");
        fs::create_dir(&workspace).unwrap();
        fs::set_permissions(&workspace, fs::Permissions::from_mode(0o700)).unwrap();
        binding.workspace_root = super::super::trusted_fs::TrustedAuthorityRoot::open(&workspace)
            .unwrap()
            .identity()
            .clone();
        let mut request = start_request(binding);
        if world {
            request.world_binding = Some(WorldBindingV1 {
                world_id: "verifier-world".into(),
                world_generation: 1,
            });
        }
        let apply = issue_and_claim_start(&authority, &request);
        let TransitionApplicationOutcomeV1::Applied(applied) =
            authority.apply_start_at(&apply, at(1, 10)).unwrap()
        else {
            panic!("genuine Start application")
        };
        let descriptor = ResolvedRuntimeDescriptor {
            agent_id: request.start_contract.descriptor.agent_id.clone(),
            backend_id: request.start_contract.descriptor.backend_id.clone(),
            backend_kind: ResolvedRuntimeBackendKind::Codex,
            protocol: request.start_contract.descriptor.protocol.clone(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: request.start_contract.descriptor.binary_path.clone(),
        };
        let scope = if world {
            AgentExecutionScope::World
        } else {
            AgentExecutionScope::Host
        };
        let scope_text = if world { "world" } else { "host" };
        let public_backend = if world {
            "cli:world-request"
        } else {
            "cli:codex"
        };
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let preferred = home
            .join("runtime-control/durable-start/startup/sessiostart1-particstart1.startup.sock");
        let stream_path = if preferred.as_os_str().len() > 100 {
            PathBuf::from(
                "/tmp/substrate-durable-start/startup/sessiostart1-particstart1.startup.sock",
            )
        } else {
            preferred
        };
        let mut helper = HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::Start,
            descriptor: descriptor.clone(),
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id: request.orchestration_session_id.clone(),
                shell_trace_session_id: request.shell_trace_session_id.clone(),
                workspace_root: request
                    .workspace_binding
                    .workspace_root
                    .physical_path
                    .clone(),
                world_id: request.world_binding.as_ref().map(|w| w.world_id.clone()),
                world_generation: request.world_binding.as_ref().map(|w| w.world_generation),
            },
            participant: HiddenOwnerHelperParticipantPlan {
                participant_id: request.target_authoritative_participant_id.clone(),
                lease_token: String::from_utf8(request.target_participant_lease_token.clone())
                    .unwrap(),
                run_id: request.run_id.clone(),
                resumed_from_participant_id: None,
                internal_uaa_session_id: None,
            },
            host_attach_contract: Some(HostAttachContract {
                backend_id: descriptor.backend_id.clone(),
                execution_scope: AgentExecutionScope::Host,
                protocol: descriptor.protocol.clone(),
                launch_descriptor: descriptor,
                capabilities: HostAttachCapabilities::default(),
                attach_launch_knobs: HostAttachLaunchKnobs {
                    requested_execution_scope: AgentExecutionScope::Host,
                    host_execution_client_start: HostAttachExecutionClientStart::StartNow,
                    attach_mode_preference: HostAttachModePreference::ContinuityPreferred,
                },
                effective_policy: serde_json::json!({"fixture_policy": "accepted-original"}),
                continuity_uaa_session_id: None,
            }),
            startup_prompt: Some(HiddenOwnerHelperStartupPromptPlan {
                prompt_text: "Exact public inaugural prompt\nwith UTF-8: λ".into(),
                stream_path,
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: Some(public_backend.into()),
                public_scope: Some(scope),
            }),
            source_orchestration_session_id: None,
        };
        let digest = |domain: &str, bytes: &[u8]| {
            let mut h = Sha256::new();
            h.update(domain.as_bytes());
            h.update((bytes.len() as u64).to_be_bytes());
            h.update(bytes);
            format!("{:x}", h.finalize())
        };
        let prompt_sha = digest(
            "substrate.a1.3-p1.public-start-prompt.v1",
            helper
                .startup_prompt
                .as_ref()
                .unwrap()
                .prompt_text
                .as_bytes(),
        );
        let request_bytes = serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "descriptor": &helper.descriptor,
            "workspace_root": &helper.session.workspace_root,
            "host_attach_contract": &helper.host_attach_contract,
            "public_backend_id": public_backend,
            "public_scope": scope_text,
            "prompt_sha256": &prompt_sha,
        }))
        .unwrap();
        let request_key = digest("substrate.a1.3-p1.public-start-request.v1", &request_bytes);
        let startup = helper.startup_prompt.as_mut().unwrap();
        startup.prompt_sha256 = Some(prompt_sha.clone());
        startup.request_key_sha256 = Some(request_key.clone());
        let HostSessionTransitionIntentStateV2::Applied {
            application_result_ref,
            authority_revision_after,
            ..
        } = &applied.state
        else {
            panic!("genuine Start application")
        };
        let mut id_hash = Sha256::new();
        id_hash.update(b"substrate.a1.3-p1.start-transaction-id.v1");
        id_hash.update([0]);
        id_hash.update(request_key.as_bytes());
        id_hash.update([0]);
        id_hash.update(request.orchestration_session_id.as_bytes());
        let transaction = StartTransactionRecordV1 {
            schema_version: 1,
            transaction_id: format!("stx_{:x}", id_hash.finalize()),
            request_key_sha256: request_key,
            prompt_sha256: prompt_sha,
            authority_store_id: request.workspace_binding.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            authoritative_participant_id: request.target_authoritative_participant_id.clone(),
            backend_id: request.start_contract.descriptor.backend_id.clone(),
            protocol: request.start_contract.descriptor.protocol.clone(),
            workspace_root: request
                .workspace_binding
                .workspace_root
                .physical_path
                .clone(),
            world_id: helper.session.world_id.clone(),
            world_generation: helper.session.world_generation,
            public_backend_id: public_backend.into(),
            public_scope: scope_text.into(),
            start_intent_id: request.intent_id.clone(),
            start_issuer_request_id: request.issuer_request_id.clone(),
            start_payload_commitment: applied.payload_commitment.clone(),
            start_application_result_ref: application_result_ref.clone(),
            start_run_id: request.run_id.clone(),
            start_authority_revision: *authority_revision_after,
            created_at: at(1, 10),
            updated_at: at(1, 10),
            state: StartTransactionStateV1::PromptNotSubmitted,
        };
        assert!(matches!(
            authority.begin_start_transaction(&transaction).unwrap(),
            StartTransactionBeginOutcomeV1::Applied(_)
        ));
        (parent, authority, request, applied, helper)
    }

    #[allow(clippy::type_complexity)]
    fn helper_resume_fixture(
        world: bool,
    ) -> (
        tempfile::TempDir,
        HostSessionAuthority,
        IssueHostSessionTransitionRequestV1,
        IssueSuccessorTransitionRequestV1,
        HostSessionTransitionIntentV3,
        HiddenOwnerHelperLaunchPlan,
    ) {
        let (parent, authority, original_request, original) =
            started(world, AgentExecutionScopeV1::Host);
        let handle = park_start(&authority, &original_request, &original);
        let source = authority
            .resolve_current_exact(&original_request.orchestration_session_id, None)
            .unwrap();
        let request = resume_successor_request(&source, handle, "helper-resume-target");
        let applied = apply_successor(&authority, &request, 2);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let preferred = home
            .join("runtime-control/durable-start/startup/sessiostart1-helpertarget.startup.sock");
        let stream_path = if preferred.as_os_str().len() > 100 {
            PathBuf::from(
                "/tmp/substrate-durable-start/startup/sessiostart1-helpertarget.startup.sock",
            )
        } else {
            preferred
        };
        let helper = HiddenOwnerHelperLaunchPlan {
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
                world_id: applied.world_binding.as_ref().map(|w| w.world_id.clone()),
                world_generation: applied.world_binding.as_ref().map(|w| w.world_generation),
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
                stream_path,
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: None,
                public_scope: None,
            }),
            source_orchestration_session_id: None,
        };
        (
            parent,
            authority,
            original_request,
            request,
            applied,
            helper,
        )
    }

    fn assert_helper_refusal_unchanged(
        proof: &AuthenticatedCurrentHostRunV1<'_>,
        helper: &HiddenOwnerHelperLaunchPlan,
        supplied: Option<&HostSessionTransitionIntentV3>,
    ) {
        let home = Path::new(&proof.facts.observation.bootstrap_home.physical_path);
        let error = unchanged(home.parent().unwrap(), || {
            proof.verify_helper_launch_v1(helper, supplied)
        })
        .unwrap_err();
        assert_eq!(error.to_string(), "helper launch comparison refused");
        assert!(!error.to_string().contains(&helper.participant.lease_token));
    }

    #[test]
    fn test_helper_launch_start_host_world_and_all_fields() {
        use serde_json::json;
        for world in [false, true] {
            let (_parent, authority, request, _, helper) = helper_start_fixture(world);
            let held = observe(&authority, &request.orchestration_session_id);
            let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
            let proof = unchanged(home, || authenticate(&authority, &request, &held).unwrap());
            unchanged(home.parent().unwrap(), || {
                proof.verify_helper_launch_v1(&helper, None).unwrap()
            });
            let mutations = [
                ("/mode", json!("attach")),
                ("/descriptor/agent_id", json!("other-agent")),
                ("/descriptor/backend_id", json!("cli:other")),
                ("/descriptor/backend_kind", json!("claude_code")),
                ("/descriptor/protocol", json!("other.protocol")),
                ("/descriptor/execution_scope", json!("world")),
                ("/descriptor/binary_path", json!("/other/binary")),
                ("/session/orchestration_session_id", json!("other-session")),
                ("/session/shell_trace_session_id", json!("other-trace")),
                ("/session/workspace_root", json!("/other/workspace")),
                ("/session/world_id", json!("other-world")),
                ("/session/world_generation", json!(999)),
                ("/participant/participant_id", json!("other-participant")),
                ("/participant/lease_token", json!("wrong-private-lease")),
                ("/participant/run_id", json!("other-run")),
                (
                    "/participant/resumed_from_participant_id",
                    json!("invented-source"),
                ),
                (
                    "/participant/internal_uaa_session_id",
                    json!("invented-native"),
                ),
                ("/host_attach_contract", json!(null)),
                ("/host_attach_contract/backend_id", json!("cli:other")),
                ("/host_attach_contract/execution_scope", json!("world")),
                ("/host_attach_contract/protocol", json!("other.protocol")),
                (
                    "/host_attach_contract/launch_descriptor/agent_id",
                    json!("other-agent"),
                ),
                (
                    "/host_attach_contract/launch_descriptor/backend_id",
                    json!("cli:other"),
                ),
                (
                    "/host_attach_contract/launch_descriptor/backend_kind",
                    json!("claude_code"),
                ),
                (
                    "/host_attach_contract/launch_descriptor/protocol",
                    json!("other.protocol"),
                ),
                (
                    "/host_attach_contract/launch_descriptor/execution_scope",
                    json!("world"),
                ),
                (
                    "/host_attach_contract/launch_descriptor/binary_path",
                    json!("/other/binary"),
                ),
                (
                    "/host_attach_contract/capabilities/session_resume",
                    json!(false),
                ),
                (
                    "/host_attach_contract/capabilities/session_fork",
                    json!(false),
                ),
                (
                    "/host_attach_contract/capabilities/session_stop",
                    json!(false),
                ),
                (
                    "/host_attach_contract/capabilities/status_snapshot",
                    json!(false),
                ),
                (
                    "/host_attach_contract/capabilities/event_stream",
                    json!(false),
                ),
                (
                    "/host_attach_contract/attach_launch_knobs/requested_execution_scope",
                    json!("world"),
                ),
                (
                    "/host_attach_contract/attach_launch_knobs/host_execution_client_start",
                    json!("defer"),
                ),
                (
                    "/host_attach_contract/attach_launch_knobs/attach_mode_preference",
                    json!("fresh_allowed"),
                ),
                (
                    "/host_attach_contract/effective_policy",
                    json!({"fixture_policy": "substituted"}),
                ),
                (
                    "/host_attach_contract/continuity_uaa_session_id",
                    json!("invented-native"),
                ),
                ("/startup_prompt", json!(null)),
                ("/startup_prompt/prompt_text", json!("substituted prompt")),
                (
                    "/startup_prompt/stream_path",
                    json!("/tmp/substituted.sock"),
                ),
                ("/startup_prompt/request_key_sha256", json!("a".repeat(64))),
                ("/startup_prompt/prompt_sha256", json!("b".repeat(64))),
                (
                    "/startup_prompt/public_backend_id",
                    json!("cli:substituted"),
                ),
                (
                    "/startup_prompt/public_scope",
                    json!(if world { "host" } else { "world" }),
                ),
                ("/source_orchestration_session_id", json!("fork-source")),
            ];
            for (path, replacement) in mutations {
                let mut value = serde_json::to_value(&helper).unwrap();
                let (parent, key) = path.rsplit_once('/').unwrap();
                value
                    .pointer_mut(parent)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert(key.into(), replacement);
                let changed = serde_json::from_value(value).unwrap();
                assert_helper_refusal_unchanged(&proof, &changed, None);
            }
            for key in [
                "request_key_sha256",
                "prompt_sha256",
                "public_backend_id",
                "public_scope",
            ] {
                let mut value = serde_json::to_value(&helper).unwrap();
                value["startup_prompt"].as_object_mut().unwrap().remove(key);
                let changed = serde_json::from_value(value).unwrap();
                assert_helper_refusal_unchanged(&proof, &changed, None);
            }
            if world {
                let mut changed = helper.clone();
                changed.session.world_id = None;
                assert_helper_refusal_unchanged(&proof, &changed, None);
                let mut changed = helper.clone();
                changed.session.world_generation = None;
                assert_helper_refusal_unchanged(&proof, &changed, None);
            }
            // A self-consistent substitute prompt and substitute asserted digest
            // still cannot authenticate against the stored original transaction.
            let mut changed = helper.clone();
            let prompt = changed.startup_prompt.as_mut().unwrap();
            prompt.prompt_text = "self-consistent but unissued prompt".into();
            prompt.prompt_sha256 = Some(helper_start_digest(
                "substrate.a1.3-p1.public-start-prompt.v1",
                &[prompt.prompt_text.as_bytes()],
            ));
            assert_helper_refusal_unchanged(&proof, &changed, None);
        }
        assert_eq!(
            helper_launch_stream_path("/h", "a!", "b?"),
            PathBuf::from("/h/runtime-control/durable-start/startup/a-b.startup.sock")
        );
        let suffix = "/runtime-control/durable-start/startup/a-b.startup.sock";
        let at_threshold = format!("/{}", "x".repeat(100 - suffix.len() - 1));
        assert_eq!(
            helper_launch_stream_path(&at_threshold, "a!", "b?"),
            PathBuf::from(format!("{at_threshold}{suffix}"))
        );
        assert_eq!(
            helper_launch_stream_path(&format!("{at_threshold}x"), "a!", "b?"),
            PathBuf::from("/tmp/substrate-durable-start/startup/a-b.startup.sock")
        );
    }

    #[test]
    fn test_helper_launch_resume_host_world_and_all_fields() {
        use serde_json::json;
        let (_p, _a, _r, _, start_helper) = helper_start_fixture(false);
        let attach_value = serde_json::to_value(&start_helper.host_attach_contract).unwrap();
        for world in [false, true] {
            let (_parent, authority, original, _, applied, helper) = helper_resume_fixture(world);
            let home = Path::new(
                &original
                    .workspace_binding
                    .authority_store_root
                    .physical_path,
            );
            let held = observe(&authority, &original.orchestration_session_id);
            let proof = unchanged(home, || {
                successor_proof(&authority, &applied, &held).unwrap()
            });
            unchanged(home.parent().unwrap(), || {
                proof
                    .verify_helper_launch_v1(&helper, Some(&applied))
                    .unwrap()
            });
            let mutations = [
                ("/mode", json!("start")),
                ("/descriptor/agent_id", json!("other-agent")),
                ("/descriptor/backend_id", json!("cli:other")),
                ("/descriptor/backend_kind", json!("claude_code")),
                ("/descriptor/protocol", json!("other.protocol")),
                ("/descriptor/execution_scope", json!("world")),
                ("/descriptor/binary_path", json!("/other/binary")),
                ("/session/orchestration_session_id", json!("other-session")),
                ("/session/shell_trace_session_id", json!("other-trace")),
                ("/session/workspace_root", json!("/other/workspace")),
                ("/session/world_id", json!("other-world")),
                ("/session/world_generation", json!(999)),
                ("/participant/participant_id", json!("other-participant")),
                ("/participant/lease_token", json!("wrong-private-lease")),
                ("/participant/run_id", json!("other-run")),
                (
                    "/participant/resumed_from_participant_id",
                    json!("wrong-source"),
                ),
                ("/participant/resumed_from_participant_id", json!(null)),
                (
                    "/participant/internal_uaa_session_id",
                    json!("wrong-native"),
                ),
                ("/participant/internal_uaa_session_id", json!(null)),
                ("/host_attach_contract", attach_value.clone()),
                ("/startup_prompt", json!(null)),
                ("/startup_prompt/prompt_text", json!("substituted input")),
                (
                    "/startup_prompt/stream_path",
                    json!("/tmp/substituted.sock"),
                ),
                ("/startup_prompt/request_key_sha256", json!("a".repeat(64))),
                ("/startup_prompt/prompt_sha256", json!("b".repeat(64))),
                (
                    "/startup_prompt/public_backend_id",
                    json!("cli:substituted"),
                ),
                ("/startup_prompt/public_scope", json!("host")),
                ("/source_orchestration_session_id", json!("fork-source")),
            ];
            for (path, replacement) in mutations {
                let mut value = serde_json::to_value(&helper).unwrap();
                let (parent, key) = path.rsplit_once('/').unwrap();
                value
                    .pointer_mut(parent)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert(key.into(), replacement);
                let changed = serde_json::from_value(value).unwrap();
                assert_helper_refusal_unchanged(&proof, &changed, Some(&applied));
            }
            if world {
                let mut changed = helper.clone();
                changed.session.world_id = None;
                assert_helper_refusal_unchanged(&proof, &changed, Some(&applied));
                let mut changed = helper.clone();
                changed.session.world_generation = None;
                assert_helper_refusal_unchanged(&proof, &changed, Some(&applied));
            }
            assert_helper_refusal_unchanged(&proof, &helper, None);
            // An Accepted input remains compareable, but a match conveys no
            // delivery permission and performs no acceptance or prompt replay.
            authority
                .accept_transition_input(&AcceptTransitionInputRequestV1 {
                    intent_id: applied.intent_id.clone(),
                    issuer_request_id: applied.issuer_request_id.clone(),
                    payload_commitment: applied.payload_commitment.clone(),
                    accepting_participant_id: applied.target_authoritative_participant_id.clone(),
                    accepted_at: at(2, 30),
                })
                .unwrap();
            assert_helper_refusal_unchanged(&proof, &helper, Some(&applied));
            let accepted_held = observe(&authority, &original.orchestration_session_id);
            let fresh = unchanged(home, || {
                successor_proof(&authority, &applied, &accepted_held).unwrap()
            });
            unchanged(home.parent().unwrap(), || {
                fresh
                    .verify_helper_launch_v1(&helper, Some(&applied))
                    .unwrap()
            });
            unchanged(home, || {
                store::with_existing_current_host_run_snapshot_opened(
                    authority.trusted_root(),
                    |snapshot| {
                        let VersionedStateRoot::V3(root) = snapshot.captured_root() else {
                            panic!("V3")
                        };
                        assert!(matches!(
                            root.successor_transition_intent_map[&applied.intent_id].input_handoff,
                            HostSessionTransitionInputHandoffV1::Accepted { .. }
                        ));
                        Ok(())
                    },
                )
                .unwrap()
            });
        }
    }

    #[test]
    fn test_helper_launch_resume_full_successor_payload() {
        use serde_json::json;
        for world in [false, true] {
            let (_parent, authority, original, _, applied, helper) = helper_resume_fixture(world);
            let home = Path::new(
                &original
                    .workspace_binding
                    .authority_store_root
                    .physical_path,
            );
            let held = observe(&authority, &original.orchestration_session_id);
            let proof = unchanged(home, || {
                successor_proof(&authority, &applied, &held).unwrap()
            });
            let mutations = [
                ("/schema_version", json!(99)),
                ("/intent_id", json!("other-intent")),
                ("/issuer_request_id", json!("other-issuer")),
                ("/mode", json!({"kind": "Attach"})),
                ("/authority_precondition", json!({"kind": "ExpectedAbsent"})),
                ("/orchestration_session_id", json!("other-session")),
                ("/shell_trace_session_id", json!("other-trace")),
                ("/caller/kind", json!({"kind": "PublicCli"})),
                ("/caller/caller_participant_id", json!("other-caller")),
                (
                    "/caller/auto_attach_obligation_id",
                    json!("invented-obligation"),
                ),
                (
                    "/caller/auto_attach_claim_owner",
                    json!("invented-claimant"),
                ),
                (
                    "/source_authoritative_participant_id",
                    json!("other-source"),
                ),
                (
                    "/target_authoritative_participant_id",
                    json!("other-target"),
                ),
                (
                    "/target_participant_lease_token_ref/ref_id",
                    json!("other-lease"),
                ),
                ("/run_id", json!("other-run")),
                ("/resulting_authoritative_lineage", json!(["other-lineage"])),
                (
                    "/workspace_binding/workspace_root/physical_path",
                    json!("/other-workspace"),
                ),
                (
                    "/workspace_binding/authority_store_root/physical_path",
                    json!("/other-home"),
                ),
                (
                    "/workspace_binding/authority_store_id",
                    json!("other-store"),
                ),
                (
                    "/workspace_binding/workspace_root/physical_identity/value/inode",
                    json!(999),
                ),
                (
                    "/workspace_binding/authority_store_root/physical_identity/value/inode",
                    json!(999),
                ),
                (
                    "/world_binding",
                    if world {
                        json!(null)
                    } else {
                        json!({"world_id":"invented-world","world_generation":1})
                    },
                ),
                ("/descriptor_ref/ref_id", json!("other-descriptor")),
                ("/host_attach_contract_ref/ref_id", json!("other-contract")),
                ("/resume_handle_ref/ref_id", json!("other-handle")),
                ("/transition_input_ref/ref_id", json!("other-input")),
                ("/post_turn_disposition", json!(null)),
                ("/transport_payload_ref/ref_id", json!("other-payload")),
                (
                    "/payload_commitment",
                    json!({"kind":"CanonicalSha256","value":{"digest_hex":"a".repeat(64)}}),
                ),
                ("/issued_at", json!("2026-07-14T12:02:01.000000000Z")),
                ("/expires_at", json!("2026-07-14T12:09:00.000000000Z")),
                ("/state/value/claim_id", json!("other-claim")),
                ("/state/value/claimant_attempt_id", json!("other-attempt")),
                ("/state/value/authority_revision_before", json!(999)),
                ("/state/value/authority_revision_after", json!(999)),
                (
                    "/state/value/active_authoritative_participant_id",
                    json!("other-target"),
                ),
                (
                    "/state/value/resulting_posture",
                    json!({"kind":"ParkedResumable"}),
                ),
                (
                    "/state/value/authority_record_commitment",
                    json!({"kind":"CanonicalSha256","value":{"digest_hex":"b".repeat(64)}}),
                ),
                (
                    "/state/value/application_result_ref/ref_id",
                    json!("other-application"),
                ),
                (
                    "/state/value/applied_at",
                    json!("2026-07-14T12:02:21.000000000Z"),
                ),
            ];
            for (path, replacement) in mutations {
                let mut value = serde_json::to_value(&applied).unwrap();
                let (parent, key) = path.rsplit_once('/').unwrap();
                value
                    .pointer_mut(parent)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert(key.into(), replacement);
                let supplied = serde_json::from_value(value).unwrap();
                assert_helper_refusal_unchanged(&proof, &helper, Some(&supplied));
            }
            // A valid immutable envelope for another current authority is also
            // assertion input, never authority for sensitive-object contexts.
            let (_p, _a, _o, _, other, _) = helper_resume_fixture(false);
            assert_helper_refusal_unchanged(&proof, &helper, Some(&other));
        }
    }

    #[test]
    fn test_helper_launch_rejects_stale_run_and_observation() {
        let (_parent, authority, request, _, helper) = helper_start_fixture(false);
        let held = observe(&authority, &request.orchestration_session_id);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let proof = unchanged(home, || authenticate(&authority, &request, &held).unwrap());
        let mut wrong = helper.clone();
        wrong.participant.run_id = "not-the-applied-run".into();
        assert_helper_refusal_unchanged(&proof, &wrong, None);
        let mut wrong_held = held.clone();
        wrong_held.root_revision += 1;
        unchanged(home, || {
            assert!(authenticate(&authority, &request, &wrong_held).is_err())
        });
        // The real producer moves the public transaction without permitting replay.
        let transaction = authority
            .retryable_start_transaction(
                helper
                    .startup_prompt
                    .as_ref()
                    .unwrap()
                    .request_key_sha256
                    .as_ref()
                    .unwrap(),
            )
            .unwrap()
            .unwrap();
        authority
            .mark_start_prompt_submission_no_replay_barrier(
                &transaction.transaction_id,
                &transaction.request_key_sha256,
                at(1, 15),
            )
            .unwrap();
        assert_helper_refusal_unchanged(&proof, &helper, None);
        let fresh_held = observe(&authority, &request.orchestration_session_id);
        let fresh = unchanged(home, || {
            authenticate(&authority, &request, &fresh_held).unwrap()
        });
        assert_helper_refusal_unchanged(&fresh, &helper, None);
    }

    #[test]
    fn test_helper_launch_exit_and_custody_faults() {
        use std::{cell::RefCell, rc::Rc};
        use store::{set_current_host_run_read_hook as hook, CurrentHostRunReadPhaseV1 as Phase};
        for refusal in [false, true] {
            for phase in [Phase::BeforeCustodyFinalize, Phase::AfterCustodyFinalize] {
                let (parent, authority, request, _, mut helper) = helper_start_fixture(false);
                let held = observe(&authority, &request.orchestration_session_id);
                let home =
                    PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
                let proof = unchanged(&home, || authenticate(&authority, &request, &held).unwrap());
                if refusal {
                    helper
                        .startup_prompt
                        .as_mut()
                        .unwrap()
                        .prompt_text
                        .push_str("tampered");
                }
                let root = home.join("authority-v1/state-root-v1.json");
                let captured = Rc::new(RefCell::new(None));
                let after_producer = Rc::clone(&captured);
                let inventory_root = parent.path().to_path_buf();
                hook(phase, move || {
                    let mut value: serde_json::Value =
                        serde_json::from_slice(&fs::read(&root).unwrap()).unwrap();
                    value["root_revision"] =
                        serde_json::json!(value["root_revision"].as_u64().unwrap() + 1);
                    fs::write(&root, canonical_json::to_vec(&value).unwrap()).unwrap();
                    *after_producer.borrow_mut() = Some(inventory(&inventory_root));
                });
                let error = proof.verify_helper_launch_v1(&helper, None).unwrap_err();
                assert_eq!(
                    error.to_string(),
                    "helper launch snapshot integrity refused"
                );
                assert_eq!(
                    inventory(parent.path()),
                    captured.borrow_mut().take().unwrap()
                );
            }
            for workspace in [false, true] {
                let (parent, authority, request, _, mut helper) = helper_start_fixture(false);
                let held = observe(&authority, &request.orchestration_session_id);
                let home =
                    PathBuf::from(&request.workspace_binding.authority_store_root.physical_path);
                let proof = unchanged(&home, || authenticate(&authority, &request, &held).unwrap());
                if refusal {
                    helper.participant.lease_token.push_str("tampered");
                }
                let retained = parent.path().join(if workspace {
                    "retained-workspace"
                } else {
                    "retained-home"
                });
                let replaced = if workspace {
                    PathBuf::from(&request.workspace_binding.workspace_root.physical_path)
                } else {
                    home.clone()
                };
                let after_producer = Rc::new(RefCell::new(None));
                let captured = Rc::clone(&after_producer);
                let inventory_root = parent.path().to_path_buf();
                hook(Phase::BeforeCustodyFinalize, move || {
                    fs::rename(&replaced, retained).unwrap();
                    fs::create_dir(&replaced).unwrap();
                    fs::set_permissions(&replaced, fs::Permissions::from_mode(0o700)).unwrap();
                    *captured.borrow_mut() = Some(inventory(&inventory_root));
                });
                let error = proof.verify_helper_launch_v1(&helper, None).unwrap_err();
                assert_eq!(error.to_string(), "helper launch physical custody refused");
                assert_eq!(
                    inventory(parent.path()),
                    after_producer.borrow_mut().take().unwrap()
                );
            }
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_helper_launch_scope_and_lifecycle_boundaries() {
        let (_parent, authority, request, original) = started(false, AgentExecutionScopeV1::Host);
        let held = observe(&authority, &request.orchestration_session_id);
        let home = Path::new(&request.workspace_binding.authority_store_root.physical_path);
        let proof = unchanged(home, || authenticate(&authority, &request, &held).unwrap());
        let (_p, _a, _r, _, helper) = helper_start_fixture(false);
        // V2 has no genuine public transaction. Comparison cannot create one.
        assert_helper_refusal_unchanged(&proof, &helper, None);
        let _handle = park_start(&authority, &request, &original);
        unchanged(home, || {
            assert!(proof.verify_helper_launch_v1(&helper, None).is_err())
        });
        let parked = observe(&authority, &request.orchestration_session_id);
        unchanged(home, || {
            assert!(authenticate(&authority, &request, &parked).is_err())
        });
        let source = authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap();
        let attach = attach_successor_request(&source, "helper-attach-target");
        let attached = apply_successor(&authority, &attach, 2);
        let attached_held = observe(&authority, &request.orchestration_session_id);
        let attach_proof = unchanged(home, || {
            successor_proof(&authority, &attached, &attached_held).unwrap()
        });
        assert_helper_refusal_unchanged(&attach_proof, &helper, None);
        let (_wp, world_role, world_request, _) = started(true, AgentExecutionScopeV1::World);
        let held = observe(&world_role, &world_request.orchestration_session_id);
        unchanged(
            Path::new(
                &world_request
                    .workspace_binding
                    .authority_store_root
                    .physical_path,
            ),
            || assert!(authenticate(&world_role, &world_request, &held).is_err()),
        );
        let (_rp, resumed, original, resume_request, applied, helper) =
            helper_resume_fixture(false);
        let home = Path::new(
            &original
                .workspace_binding
                .authority_store_root
                .physical_path,
        );
        let held = observe(&resumed, &original.orchestration_session_id);
        let proof = unchanged(home, || successor_proof(&resumed, &applied, &held).unwrap());
        resumed
            .accept_transition_input(&AcceptTransitionInputRequestV1 {
                intent_id: applied.intent_id.clone(),
                issuer_request_id: applied.issuer_request_id.clone(),
                payload_commitment: applied.payload_commitment.clone(),
                accepting_participant_id: applied.target_authoritative_participant_id.clone(),
                accepted_at: at(2, 30),
            })
            .unwrap();
        let next_handle = settle_fixture_resume(&resumed, &resume_request, &applied, 2);
        assert_eq!(next_handle.schema_version, 1);
        assert_helper_refusal_unchanged(&proof, &helper, Some(&applied));
        let completed = observe(&resumed, &original.orchestration_session_id);
        unchanged(home, || {
            assert!(successor_proof(&resumed, &applied, &completed).is_err())
        });
        // The second genuine Resume consumes the first Resume's produced V1
        // handle, complementing the public Start V2 handle cases above.
        let source = resumed
            .resolve_current_exact(&original.orchestration_session_id, None)
            .unwrap();
        let mut second_request = resume_successor_request(&source, next_handle, "second");
        second_request.intent_id = "helper-resume-second-intent".into();
        second_request.issuer_request_id = "helper-resume-second-request".into();
        second_request.run_id = "helper-resume-second-run".into();
        let second = apply_successor(&resumed, &second_request, 4);
        let mut second_helper = helper.clone();
        second_helper.participant.participant_id =
            second.target_authoritative_participant_id.clone();
        second_helper.participant.run_id = second.run_id.clone();
        second_helper.participant.resumed_from_participant_id =
            second.source_authoritative_participant_id.clone();
        second_helper.participant.lease_token =
            String::from_utf8(second_request.target_participant_lease_token.clone()).unwrap();
        let preferred =
            home.join("runtime-control/durable-start/startup/sessiostart1-second.startup.sock");
        second_helper.startup_prompt.as_mut().unwrap().stream_path = if preferred.as_os_str().len()
            > 100
        {
            PathBuf::from("/tmp/substrate-durable-start/startup/sessiostart1-second.startup.sock")
        } else {
            preferred
        };
        let held = observe(&resumed, &original.orchestration_session_id);
        let second_proof = unchanged(home, || successor_proof(&resumed, &second, &held).unwrap());
        unchanged(home.parent().unwrap(), || {
            second_proof
                .verify_helper_launch_v1(&second_helper, Some(&second))
                .unwrap()
        });
        second_helper.participant.internal_uaa_session_id = Some("wrong-produced-native".into());
        assert_helper_refusal_unchanged(&second_proof, &second_helper, Some(&second));
    }
}
