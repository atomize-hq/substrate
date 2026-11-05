//! Ordinary Fork policy ownership. New population requires the actual active
//! controller; immutable exact joins carry no new-work or World-send permit.
use super::super::native_launch_context::AuthenticatedOrdinarySelectionV1;
use super::*;

fn authenticate_ordinary_fork_scope(
    selection: &AuthenticatedOrdinarySelectionV1,
    session: &str,
    caller: &str,
    caller_backend: &str,
    target_backend: &str,
    world_id: &str,
    world_generation: u64,
) -> Result<(), DispatchPolicyCommitmentError> {
    let fail = || {
        DispatchPolicyCommitmentError::new(
            "ordinary Fork lost current configured scope or permission",
        )
    };
    selection
        .require_ordinary_configured_authority()
        .map_err(|_| fail())?;
    selection.revalidate().map_err(|_| fail())?;
    let current = selection
        .held_authority()
        .resolve_current_exact(selection.original_session_id(), None)
        .map_err(|_| fail())?;
    if selection.native_selection().placement
        != transport_api_types::AgentSelectionPlacementV1::Host
        || selection.requested_selection().placement
            != transport_api_types::AgentSelectionPlacementV1::World
        || session != selection.original_session_id()
        || caller != selection.current_participant_id()
        || current.caller.participant_id != caller
        || caller_backend != selection.native_selection().backend_id
        || target_backend != selection.requested_selection().backend_id
        || selection.current_world_binding() != Some((world_id, world_generation))
        || current
            .authority
            .world_binding
            .as_ref()
            .map(|w| (w.world_id.as_str(), w.world_generation))
            != Some((world_id, world_generation))
    {
        return Err(fail());
    }
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        std::path::Path::new(selection.workspace_root()),
        &selection.held_authority().bootstrap_home(),
    )
    .map_err(|_| fail())?;
    let steering = policy.world_dispatch_policy();
    if !steering.enabled
        || !steering
            .allowed_actions
            .iter()
            .any(|a| a == "fork_world_worker")
        || !steering.allowed_modes.iter().any(|m| m == "retained")
        || !steering
            .allowed_backends
            .iter()
            .any(|b| b == target_backend)
    {
        return Err(fail());
    }
    selection
        .revalidate_requested_account(&policy)
        .map_err(|_| fail())?;
    selection.revalidate().map_err(|_| fail())
}

fn read_ordinary_fork_registry(
    selection: &AuthenticatedOrdinarySelectionV1,
    lookup: &DispatchPolicyCommitmentLookupKeyV1,
) -> Result<DispatchPolicyCommitmentRegistryV1, DispatchPolicyCommitmentError> {
    let snapshot = read_existing_accepted_work_authority_snapshot(selection.held_authority())
        .map_err(|_| {
            DispatchPolicyCommitmentError::new("read complete ordinary Fork policy source")
        })?;
    let DispatchPolicyCommitmentPhysicalReadV1::Present(physical) = snapshot.e2() else {
        return Err(DispatchPolicyCommitmentError::new(
            "ordinary Fork source cap registry absent",
        ));
    };
    decode_and_authenticate_e2_registry(physical, snapshot.authority_store_id(), lookup).map_err(
        |_| DispatchPolicyCommitmentError::new("authenticate complete ordinary Fork policy source"),
    )
}

pub(crate) fn resolve_ordinary_fork_policy_material(
    selection: &AuthenticatedOrdinarySelectionV1,
    input: ForkPolicyResolutionInputV1,
) -> Result<AuthenticatedForkPolicyMaterialV1, DispatchPolicyCommitmentError> {
    authenticate_ordinary_fork_scope(
        selection,
        &input.orchestration_session_id,
        &input.caller_participant_id,
        &input.caller_backend_id,
        &input.target_backend_id,
        &input.world_id,
        input.world_generation,
    )?;
    let current = selection
        .held_authority()
        .resolve_current_exact(selection.original_session_id(), None)
        .map_err(|_| DispatchPolicyCommitmentError::new("read ordinary Fork parent policy"))?;
    if current.authority.current_policy_ref.as_ref()
        != Some(input.current_parent_and_fork_patch.snapshot_ref())
        || current.current_policy.policy_revision != input.current_parent_and_fork_patch.revision()
        || (input.fork_patch.is_none()
            && current.current_policy.canonical_policy_snapshot_sha256
                != input.current_parent_and_fork_patch.hash())
        || (input.world_generation == 0 && input.fork_patch.is_some())
    {
        return Err(DispatchPolicyCommitmentError::new(
            "ordinary Fork current parent policy differs",
        ));
    }
    let lookup = DispatchPolicyCommitmentLookupKeyV1 {
        authority_store_id: selection.authority_store_id().into(),
        orchestration_session_id: input.orchestration_session_id.clone(),
        request_id: input.request_id.clone(),
        subject: DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerFork {
            source_participant_id: input.source_participant_id.clone(),
        },
    };
    let registry = read_ordinary_fork_registry(selection, &lookup)?;
    let reference = input.source_cap.commitment_ref();
    let record = registry
        .commitments_by_id
        .get(&reference.commitment_id)
        .ok_or_else(|| DispatchPolicyCommitmentError::new("ordinary Fork source cap missing"))?;
    validate_cap_for_participant(
        &registry,
        &input.source_participant_id,
        reference,
        &input.orchestration_session_id,
    )?;
    validate_commitment(record, Some(reference))?;
    let snapshot_bytes = record.policy_snapshot_bytes.resolve_inline()?;
    let snapshot = serde_json::from_slice(&snapshot_bytes)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode ordinary Fork source policy"))?;
    let authenticated = AuthenticatedDispatchPolicyCommitmentV1 {
        record: record.clone(),
        snapshot,
        snapshot_bytes,
    };
    let result = resolve_fork_policy_material_inner(input, authenticated);
    selection
        .revalidate()
        .map_err(|error| DispatchPolicyCommitmentError::new(error.to_string()))?;
    result
}

pub(crate) fn publish_ordinary_fork_commitment(
    selection: &AuthenticatedOrdinarySelectionV1,
    input: ForkPolicyCommitmentInputV1,
) -> Result<
    (
        PersistedDispatchPolicyCommitmentV1,
        Option<AuthenticatedOrdinaryForkIssuanceV1>,
    ),
    DispatchPolicyCommitmentError,
> {
    authenticate_ordinary_fork_scope(
        selection,
        &input.orchestration_session_id,
        &input.caller_participant_id,
        &input.caller_backend_id,
        &input.target_backend_id,
        &input.world_id,
        input.world_generation,
    )?;
    let current = selection
        .held_authority()
        .resolve_current_exact(selection.original_session_id(), None)
        .map_err(|_| DispatchPolicyCommitmentError::new("read ordinary Fork publication parent"))?;
    if current.authority.current_policy_ref.as_ref() != Some(&input.parent_policy_ref)
        || current.current_policy.policy_revision != input.parent_policy_revision
        || (input.world_generation == 0
            && input.applied_patch != AppliedDispatchPolicyPatchIdentityV1::UnchangedParent)
    {
        return Err(DispatchPolicyCommitmentError::new(
            "ordinary Fork publication parent differs",
        ));
    }
    let hash = fork_input_request_hash(&input)?;
    if let Some(existing) = join_existing_ordinary_fork_commitment(selection, &input, &hash)? {
        return Ok((existing, None));
    }
    let publication = super::super::host_session_authority::world_bound_controller::authenticate_ordinary_world_publication(selection)
        .map_err(|error| DispatchPolicyCommitmentError::new(error.to_string()))?;
    let storage = super::super::host_session_authority::store::dispatch_policy_commitment_storage_for_ordinary_publication(selection.held_authority(), publication)
        .map_err(storage_error)?;
    // The source cap requires an existing complete registry; no empty init.
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        std::path::Path::new(selection.workspace_root()),
        &selection.held_authority().bootstrap_home(),
    )
    .map_err(|_| {
        DispatchPolicyCommitmentError::new("read current ordinary Fork population policy")
    })?;
    let maximum = u64::from(policy.world_dispatch_policy().max_live_retained_workers);
    selection
        .revalidate()
        .map_err(|error| DispatchPolicyCommitmentError::new(error.to_string()))?;
    publish_fork_commitment_in_storage_with_issuance(&storage, input, hash, Some(maximum))
}

fn join_existing_ordinary_fork_commitment(
    selection: &AuthenticatedOrdinarySelectionV1,
    input: &ForkPolicyCommitmentInputV1,
    hash: &str,
) -> Result<Option<PersistedDispatchPolicyCommitmentV1>, DispatchPolicyCommitmentError> {
    let lookup = DispatchPolicyCommitmentLookupKeyV1 {
        authority_store_id: selection.authority_store_id().into(),
        orchestration_session_id: input.orchestration_session_id.clone(),
        request_id: input.request_id.clone(),
        subject: DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerFork {
            source_participant_id: input.source_participant_id.clone(),
        },
    };
    let registry = read_ordinary_fork_registry(selection, &lookup)?;
    validate_cap_for_participant(
        &registry,
        &input.source_participant_id,
        input.authenticated_policy.source_worker_cap_ref(),
        &input.orchestration_session_id,
    )?;
    let Some(index) = registry
        .request_subject_index
        .get(&lookup_index_key(&lookup)?)
    else {
        return Ok(None);
    };
    if index.lookup_key != lookup {
        return Err(DispatchPolicyCommitmentError::new(
            "ordinary Fork retry index collision",
        ));
    }
    let DispatchPolicyCommitmentIndexEntryV1::Committed {
        reservation_ref: None,
        fresh_spawn_validated_request_commitment: None,
        commitment_ref: reference,
    } = &index.entry
    else {
        return Err(DispatchPolicyCommitmentError::new(
            "ordinary Fork retry changed entry class",
        ));
    };
    let record = registry
        .commitments_by_id
        .get(&reference.commitment_id)
        .ok_or_else(|| DispatchPolicyCommitmentError::new("ordinary Fork retry record missing"))?;
    validate_commitment(record, Some(reference))?;
    validate_fork_retry(record, input, hash)?;
    let child = fork_child_participant_id(record)?;
    if registry.retained_worker_caps_by_participant_id.get(child) != Some(reference) {
        return Err(DispatchPolicyCommitmentError::new(
            "ordinary Fork child-cap index differs",
        ));
    }
    selection
        .revalidate()
        .map_err(|error| DispatchPolicyCommitmentError::new(error.to_string()))?;
    Ok(Some(PersistedDispatchPolicyCommitmentV1(record.clone())))
}
