use super::{
    AuthorityObjectKindV1, AuthorityObjectRefV1, HostSessionPostTurnApplicationV1,
    HostSessionStartupOwnershipApplicationV1, HostSessionTransitionInputHandoffV1,
    HostSessionTransitionIntentStateV1, HostSessionTransitionIntentStateV2,
    HostSessionTransitionTransportPayloadStateV1, ObjectVerificationContextV1,
    SessionNamespaceRecordV1, StateRootV1, StateRootV2, StoreError,
    VersionedObjectVerificationParentIntentV1,
};
use crate::execution::agent_runtime::host_session_authority::store_schema::{
    HostSessionPostTurnApplicationV2, HostSessionTransitionIntentStateV3, StateRootV3,
};
use crate::execution::agent_runtime::host_session_authority::validation::ValidatedCanonicalV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ReachableObjectV1 {
    pub(super) reference: AuthorityObjectRefV1,
    pub(super) context: Option<ObjectVerificationContextV1>,
}

pub(super) fn collect_reachable_objects(
    root: &StateRootV1,
) -> Result<std::collections::BTreeMap<String, ReachableObjectV1>, StoreError> {
    let mut reachable = std::collections::BTreeMap::new();
    for record in root.session_namespace_map.values() {
        match record {
            SessionNamespaceRecordV1::Authority(authority) => {
                add_optional_ref(
                    &mut reachable,
                    authority.host_attach_contract_ref.as_ref(),
                    AuthorityObjectKindV1::HostAttachContract,
                    None,
                )?;
                for reference in &authority.retained_worker_refs {
                    add_expected_ref(
                        &mut reachable,
                        reference,
                        AuthorityObjectKindV1::RetainedWorker,
                        None,
                    )?;
                }
                for reference in &authority.internal_resume_handle_refs {
                    add_expected_ref(
                        &mut reachable,
                        reference,
                        AuthorityObjectKindV1::ResumeHandle,
                        None,
                    )?;
                }
                add_optional_ref(
                    &mut reachable,
                    authority.current_policy_ref.as_ref(),
                    AuthorityObjectKindV1::Policy,
                    None,
                )?;
            }
            SessionNamespaceRecordV1::StartReservation(_) => {}
            SessionNamespaceRecordV1::StartTombstone(tombstone) => {
                add_expected_ref(
                    &mut reachable,
                    &tombstone.terminal_handoff_ref,
                    AuthorityObjectKindV1::TerminalHandoff,
                    None,
                )?;
            }
        }
    }

    for intent in root.transition_intent_map.values() {
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        add_expected_ref(
            &mut reachable,
            &intent.target_participant_lease_token_ref,
            AuthorityObjectKindV1::LeaseToken,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.descriptor_ref,
            AuthorityObjectKindV1::AgentDescriptor,
            None,
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.host_attach_contract_ref,
            AuthorityObjectKindV1::HostAttachContract,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.resume_handle_ref.as_ref(),
            AuthorityObjectKindV1::ResumeHandle,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.transition_input_ref.as_ref(),
            AuthorityObjectKindV1::TransitionInput,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.transport_payload_ref,
            AuthorityObjectKindV1::TransitionTransportPayload,
            Some(ObjectVerificationContextV1 {
                intent_id: intent.intent_id.clone(),
                run_id: intent.run_id.clone(),
                parent_intent: Some(VersionedObjectVerificationParentIntentV1::V1(Box::new(
                    intent.clone(),
                ))),
            }),
        )?;
        collect_intent_state_refs(&mut reachable, &intent.state)?;
        collect_input_handoff_refs(&mut reachable, &intent.input_handoff, &context)?;
        match &intent.transport_payload_state {
            HostSessionTransitionTransportPayloadStateV1::Retained => {}
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref,
            }
            | HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref,
                ..
            } => add_expected_ref(
                &mut reachable,
                terminal_handoff_ref,
                AuthorityObjectKindV1::TerminalHandoff,
                None,
            )?,
        }
    }

    for journal in root.application_journal.values() {
        add_expected_ref(
            &mut reachable,
            &journal.initial_application.application_result_ref,
            AuthorityObjectKindV1::ApplicationResult,
            None,
        )?;
        if let Some(post_turn) = &journal.post_turn_application {
            add_expected_ref(
                &mut reachable,
                &post_turn.completion_ref,
                AuthorityObjectKindV1::PostTurnCompletion,
                None,
            )?;
            add_expected_ref(
                &mut reachable,
                &post_turn.application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
                None,
            )?;
        }
    }
    Ok(reachable)
}

pub(super) fn collect_reachable_objects_v2(
    root: &StateRootV2,
) -> Result<std::collections::BTreeMap<String, ReachableObjectV1>, StoreError> {
    let mut reachable = std::collections::BTreeMap::new();
    collect_namespace_refs(&mut reachable, root.session_namespace_map.values())?;
    for intent in root.transition_intent_map.values() {
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        add_expected_ref(
            &mut reachable,
            &intent.target_participant_lease_token_ref,
            AuthorityObjectKindV1::LeaseToken,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.descriptor_ref,
            AuthorityObjectKindV1::AgentDescriptor,
            None,
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.host_attach_contract_ref,
            AuthorityObjectKindV1::HostAttachContract,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.resume_handle_ref.as_ref(),
            AuthorityObjectKindV1::ResumeHandle,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.transition_input_ref.as_ref(),
            AuthorityObjectKindV1::TransitionInput,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.transport_payload_ref,
            AuthorityObjectKindV1::TransitionTransportPayload,
            Some(ObjectVerificationContextV1 {
                intent_id: intent.intent_id.clone(),
                run_id: intent.run_id.clone(),
                parent_intent: Some(VersionedObjectVerificationParentIntentV1::V2(Box::new(
                    intent.clone(),
                ))),
            }),
        )?;
        collect_intent_state_refs_v2(&mut reachable, &intent.state)?;
        collect_input_handoff_refs(&mut reachable, &intent.input_handoff, &context)?;
        match &intent.transport_payload_state {
            HostSessionTransitionTransportPayloadStateV1::Retained => {}
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref,
            }
            | HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref,
                ..
            } => add_expected_ref(
                &mut reachable,
                terminal_handoff_ref,
                AuthorityObjectKindV1::TerminalHandoff,
                None,
            )?,
        }
    }
    for journal in root.application_journal.values() {
        add_expected_ref(
            &mut reachable,
            &journal.initial_application.application_result_ref,
            AuthorityObjectKindV1::ApplicationResult,
            None,
        )?;
        if journal.startup_terminal_application.is_some() {
            return Err(StoreError(
                "A1.2a V2 cannot contain a startup terminal application",
            ));
        }
        if let Some(post_turn) = &journal.post_turn_application {
            add_expected_ref(
                &mut reachable,
                &post_turn.completion_ref,
                AuthorityObjectKindV1::PostTurnCompletion,
                None,
            )?;
            add_expected_ref(
                &mut reachable,
                &post_turn.application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
                None,
            )?;
        }
    }
    Ok(reachable)
}

pub(super) fn collect_reachable_objects_v3(
    root: &StateRootV3,
) -> Result<std::collections::BTreeMap<String, ReachableObjectV1>, StoreError> {
    let mut reachable = std::collections::BTreeMap::new();
    collect_namespace_refs(&mut reachable, root.session_namespace_map.values())?;
    for intent in root.transition_intent_map.values() {
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        add_expected_ref(
            &mut reachable,
            &intent.target_participant_lease_token_ref,
            AuthorityObjectKindV1::LeaseToken,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.descriptor_ref,
            AuthorityObjectKindV1::AgentDescriptor,
            None,
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.host_attach_contract_ref,
            AuthorityObjectKindV1::HostAttachContract,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.resume_handle_ref.as_ref(),
            AuthorityObjectKindV1::ResumeHandle,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.transition_input_ref.as_ref(),
            AuthorityObjectKindV1::TransitionInput,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.transport_payload_ref,
            AuthorityObjectKindV1::TransitionTransportPayload,
            Some(ObjectVerificationContextV1 {
                intent_id: intent.intent_id.clone(),
                run_id: intent.run_id.clone(),
                parent_intent: Some(VersionedObjectVerificationParentIntentV1::V2(Box::new(
                    intent.clone(),
                ))),
            }),
        )?;
        collect_intent_state_refs_v2(&mut reachable, &intent.state)?;
        collect_input_handoff_refs(&mut reachable, &intent.input_handoff, &context)?;
        match &intent.transport_payload_state {
            HostSessionTransitionTransportPayloadStateV1::Retained => {}
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref,
            }
            | HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref,
                ..
            } => add_expected_ref(
                &mut reachable,
                terminal_handoff_ref,
                AuthorityObjectKindV1::TerminalHandoff,
                None,
            )?,
        }
    }
    for journal in root.application_journal.values() {
        add_expected_ref(
            &mut reachable,
            &journal.initial_application.application_result_ref,
            AuthorityObjectKindV1::ApplicationResult,
            None,
        )?;
        if let Some(startup_terminal_application) = &journal.startup_terminal_application {
            add_expected_ref(
                &mut reachable,
                &startup_terminal_application.startup_ownership_result_ref,
                AuthorityObjectKindV1::StartupOwnershipResult,
                None,
            )?;
        }
        if let Some(post_turn) = &journal.post_turn_application {
            add_expected_ref(
                &mut reachable,
                &post_turn.completion_ref,
                AuthorityObjectKindV1::PostTurnCompletion,
                None,
            )?;
            add_expected_ref(
                &mut reachable,
                &post_turn.application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
                None,
            )?;
        }
    }

    for intent in root.successor_transition_intent_map.values() {
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        add_expected_ref(
            &mut reachable,
            &intent.target_participant_lease_token_ref,
            AuthorityObjectKindV1::LeaseToken,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.descriptor_ref,
            AuthorityObjectKindV1::AgentDescriptor,
            None,
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.host_attach_contract_ref,
            AuthorityObjectKindV1::HostAttachContract,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.resume_handle_ref.as_ref(),
            AuthorityObjectKindV1::ResumeHandle,
            None,
        )?;
        add_optional_ref(
            &mut reachable,
            intent.transition_input_ref.as_ref(),
            AuthorityObjectKindV1::TransitionInput,
            Some(context.clone()),
        )?;
        add_expected_ref(
            &mut reachable,
            &intent.transport_payload_ref,
            AuthorityObjectKindV1::TransitionTransportPayload,
            Some(ObjectVerificationContextV1 {
                intent_id: intent.intent_id.clone(),
                run_id: intent.run_id.clone(),
                parent_intent: Some(VersionedObjectVerificationParentIntentV1::V3(Box::new(
                    intent.clone(),
                ))),
            }),
        )?;
        collect_intent_state_refs_v3(&mut reachable, &intent.state)?;
        collect_input_handoff_refs(&mut reachable, &intent.input_handoff, &context)?;
        match &intent.transport_payload_state {
            HostSessionTransitionTransportPayloadStateV1::Retained => {}
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref,
            }
            | HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref,
                ..
            } => add_expected_ref(
                &mut reachable,
                terminal_handoff_ref,
                AuthorityObjectKindV1::TerminalHandoff,
                None,
            )?,
        }
    }

    for journal in root.successor_application_journal.values() {
        add_expected_ref(
            &mut reachable,
            &journal.initial_application.application_result_ref,
            AuthorityObjectKindV1::ApplicationResult,
            None,
        )?;
        if let Some(startup_terminal_application) = &journal.startup_terminal_application {
            add_expected_ref(
                &mut reachable,
                &startup_terminal_application.startup_ownership_result_ref,
                AuthorityObjectKindV1::StartupOwnershipResult,
                None,
            )?;
        }
        if let Some(post_turn) = &journal.post_turn_application {
            add_expected_ref(
                &mut reachable,
                &post_turn.completion_ref,
                AuthorityObjectKindV1::PostTurnCompletion,
                None,
            )?;
            add_optional_ref(
                &mut reachable,
                post_turn.obligation_snapshot_ref.as_ref(),
                AuthorityObjectKindV1::ObligationSnapshot,
                None,
            )?;
            add_expected_ref(
                &mut reachable,
                &post_turn.application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
                None,
            )?;
        }
    }
    Ok(reachable)
}

fn collect_namespace_refs<'a>(
    reachable: &mut std::collections::BTreeMap<String, ReachableObjectV1>,
    records: impl Iterator<Item = &'a SessionNamespaceRecordV1>,
) -> Result<(), StoreError> {
    for record in records {
        match record {
            SessionNamespaceRecordV1::Authority(authority) => {
                add_optional_ref(
                    reachable,
                    authority.host_attach_contract_ref.as_ref(),
                    AuthorityObjectKindV1::HostAttachContract,
                    None,
                )?;
                for reference in &authority.retained_worker_refs {
                    add_expected_ref(
                        reachable,
                        reference,
                        AuthorityObjectKindV1::RetainedWorker,
                        None,
                    )?;
                }
                for reference in &authority.internal_resume_handle_refs {
                    add_expected_ref(
                        reachable,
                        reference,
                        AuthorityObjectKindV1::ResumeHandle,
                        None,
                    )?;
                }
                add_optional_ref(
                    reachable,
                    authority.current_policy_ref.as_ref(),
                    AuthorityObjectKindV1::Policy,
                    None,
                )?;
            }
            SessionNamespaceRecordV1::StartReservation(_) => {}
            SessionNamespaceRecordV1::StartTombstone(tombstone) => add_expected_ref(
                reachable,
                &tombstone.terminal_handoff_ref,
                AuthorityObjectKindV1::TerminalHandoff,
                None,
            )?,
        }
    }
    Ok(())
}

fn collect_intent_state_refs_v2(
    reachable: &mut std::collections::BTreeMap<String, ReachableObjectV1>,
    state: &HostSessionTransitionIntentStateV2,
) -> Result<(), StoreError> {
    match state {
        HostSessionTransitionIntentStateV2::Issued
        | HostSessionTransitionIntentStateV2::Claimed { .. } => Ok(()),
        HostSessionTransitionIntentStateV2::Applied {
            application_result_ref,
            startup_ownership,
            post_turn,
            ..
        } => {
            add_expected_ref(
                reachable,
                application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
                None,
            )?;
            match startup_ownership.as_ref() {
                HostSessionStartupOwnershipApplicationV1::Pending { .. } => {}
                HostSessionStartupOwnershipApplicationV1::Accepted { result_ref, .. }
                | HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                    result_ref, ..
                } => add_expected_ref(
                    reachable,
                    result_ref,
                    AuthorityObjectKindV1::StartupOwnershipResult,
                    None,
                )?,
                HostSessionStartupOwnershipApplicationV1::NotApplicable => {
                    return Err(StoreError("V2 Start startup ownership is invalid"))
                }
            }
            if post_turn.as_ref() != &HostSessionPostTurnApplicationV1::NotApplicable {
                return Err(StoreError("V2 Start post-turn substate is invalid"));
            }
            Ok(())
        }
        HostSessionTransitionIntentStateV2::Rejected {
            terminal_handoff_ref,
            ..
        }
        | HostSessionTransitionIntentStateV2::Expired {
            terminal_handoff_ref,
            ..
        } => add_expected_ref(
            reachable,
            terminal_handoff_ref,
            AuthorityObjectKindV1::TerminalHandoff,
            None,
        ),
    }
}

fn collect_intent_state_refs_v3(
    reachable: &mut std::collections::BTreeMap<String, ReachableObjectV1>,
    state: &HostSessionTransitionIntentStateV3,
) -> Result<(), StoreError> {
    match state {
        HostSessionTransitionIntentStateV3::Issued
        | HostSessionTransitionIntentStateV3::Claimed { .. } => Ok(()),
        HostSessionTransitionIntentStateV3::Applied {
            application_result_ref,
            startup_ownership,
            post_turn,
            ..
        } => {
            add_expected_ref(
                reachable,
                application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
                None,
            )?;
            match startup_ownership.as_ref() {
                HostSessionStartupOwnershipApplicationV1::NotApplicable
                | HostSessionStartupOwnershipApplicationV1::Pending { .. } => {}
                HostSessionStartupOwnershipApplicationV1::Accepted { result_ref, .. }
                | HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                    result_ref, ..
                } => add_expected_ref(
                    reachable,
                    result_ref,
                    AuthorityObjectKindV1::StartupOwnershipResult,
                    None,
                )?,
            }
            match post_turn.as_ref() {
                HostSessionPostTurnApplicationV2::NotApplicable
                | HostSessionPostTurnApplicationV2::Pending { .. } => Ok(()),
                HostSessionPostTurnApplicationV2::AwaitingObligationCut {
                    completion_ref, ..
                } => add_expected_ref(
                    reachable,
                    completion_ref,
                    AuthorityObjectKindV1::PostTurnCompletion,
                    None,
                ),
                HostSessionPostTurnApplicationV2::Applied {
                    completion_ref,
                    obligation_snapshot_ref,
                    application_result_ref,
                    ..
                } => {
                    add_expected_ref(
                        reachable,
                        completion_ref,
                        AuthorityObjectKindV1::PostTurnCompletion,
                        None,
                    )?;
                    add_optional_ref(
                        reachable,
                        obligation_snapshot_ref.as_deref(),
                        AuthorityObjectKindV1::ObligationSnapshot,
                        None,
                    )?;
                    add_expected_ref(
                        reachable,
                        application_result_ref,
                        AuthorityObjectKindV1::ApplicationResult,
                        None,
                    )
                }
            }
        }
        HostSessionTransitionIntentStateV3::Rejected {
            terminal_handoff_ref,
            ..
        }
        | HostSessionTransitionIntentStateV3::Expired {
            terminal_handoff_ref,
            ..
        } => add_expected_ref(
            reachable,
            terminal_handoff_ref,
            AuthorityObjectKindV1::TerminalHandoff,
            None,
        ),
    }
}

fn collect_intent_state_refs(
    reachable: &mut std::collections::BTreeMap<String, ReachableObjectV1>,
    state: &HostSessionTransitionIntentStateV1,
) -> Result<(), StoreError> {
    match state {
        HostSessionTransitionIntentStateV1::Issued
        | HostSessionTransitionIntentStateV1::Claimed { .. } => Ok(()),
        HostSessionTransitionIntentStateV1::Applied {
            application_result_ref,
            post_turn,
            ..
        } => {
            add_expected_ref(
                reachable,
                application_result_ref,
                AuthorityObjectKindV1::ApplicationResult,
                None,
            )?;
            match post_turn.as_ref() {
                HostSessionPostTurnApplicationV1::NotApplicable
                | HostSessionPostTurnApplicationV1::Pending { .. } => Ok(()),
                HostSessionPostTurnApplicationV1::Applied {
                    completion_ref,
                    application_result_ref,
                    ..
                } => {
                    add_expected_ref(
                        reachable,
                        completion_ref,
                        AuthorityObjectKindV1::PostTurnCompletion,
                        None,
                    )?;
                    add_expected_ref(
                        reachable,
                        application_result_ref,
                        AuthorityObjectKindV1::ApplicationResult,
                        None,
                    )
                }
            }
        }
        HostSessionTransitionIntentStateV1::Rejected {
            terminal_handoff_ref,
            ..
        }
        | HostSessionTransitionIntentStateV1::Expired {
            terminal_handoff_ref,
            ..
        } => add_expected_ref(
            reachable,
            terminal_handoff_ref,
            AuthorityObjectKindV1::TerminalHandoff,
            None,
        ),
    }
}

fn collect_input_handoff_refs(
    reachable: &mut std::collections::BTreeMap<String, ReachableObjectV1>,
    handoff: &HostSessionTransitionInputHandoffV1,
    context: &ObjectVerificationContextV1,
) -> Result<(), StoreError> {
    match handoff {
        HostSessionTransitionInputHandoffV1::NotApplicable => Ok(()),
        HostSessionTransitionInputHandoffV1::Pending { input_ref, .. } => add_expected_ref(
            reachable,
            input_ref,
            AuthorityObjectKindV1::TransitionInput,
            Some(context.clone()),
        ),
        HostSessionTransitionInputHandoffV1::Accepted {
            input_ref,
            acceptance_ref,
            ..
        } => {
            add_expected_ref(
                reachable,
                input_ref,
                AuthorityObjectKindV1::TransitionInput,
                Some(context.clone()),
            )?;
            add_expected_ref(
                reachable,
                acceptance_ref,
                AuthorityObjectKindV1::InputAcceptance,
                None,
            )
        }
        HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
            input_ref,
            terminal_handoff_ref,
            ..
        } => {
            add_expected_ref(
                reachable,
                input_ref,
                AuthorityObjectKindV1::TransitionInput,
                Some(context.clone()),
            )?;
            add_expected_ref(
                reachable,
                terminal_handoff_ref,
                AuthorityObjectKindV1::TerminalHandoff,
                None,
            )
        }
    }
}

fn add_optional_ref(
    reachable: &mut std::collections::BTreeMap<String, ReachableObjectV1>,
    reference: Option<&AuthorityObjectRefV1>,
    expected_kind: AuthorityObjectKindV1,
    context: Option<ObjectVerificationContextV1>,
) -> Result<(), StoreError> {
    if let Some(reference) = reference {
        add_expected_ref(reachable, reference, expected_kind, context)?;
    }
    Ok(())
}

pub(super) fn add_expected_ref(
    reachable: &mut std::collections::BTreeMap<String, ReachableObjectV1>,
    reference: &AuthorityObjectRefV1,
    expected_kind: AuthorityObjectKindV1,
    context: Option<ObjectVerificationContextV1>,
) -> Result<(), StoreError> {
    if reference.object_kind != expected_kind || reference.validate().is_err() {
        return Err(StoreError(
            "parent-owned object ref has the wrong kind or version",
        ));
    }
    let value = ReachableObjectV1 {
        reference: reference.clone(),
        context,
    };
    if reachable
        .insert(reference.ref_id.clone(), value.clone())
        .is_some_and(|existing| existing != value)
    {
        return Err(StoreError(
            "object ref is reused with conflicting authority",
        ));
    }
    Ok(())
}
