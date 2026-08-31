//! Atomic HSA allocation of a parked fork-successor authority.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::canonical_json;
use super::facade::{AuthorityObservationV1, HostSessionAuthority, ResolvedCurrentAuthorityV1};
use super::schema::{
    AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, AuthorityObjectKindV1,
    AuthorityObjectRefV1, DurableSessionAuthorityHashInputV1,
    ForkSuccessorAllocationRequestHashInputV1, HostAttachContractHashInputV1,
    HostSessionAuthorityPreconditionV1, HostSessionPostureV1,
};
use super::store;
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
    ForkSuccessorAllocationRecordV1, SessionNamespaceRecordV1, StateRootV3, VersionedStateRoot,
};
use super::validation::{CanonicalHashInputV1, ValidatedCanonicalV1};

pub(crate) type AllocateForkSuccessorRequestV1 = ForkSuccessorAllocationRequestHashInputV1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForkSuccessorAllocationV1 {
    pub(crate) allocation_id: String,
    pub(crate) request_id: String,
    pub(crate) request_commitment: AuthorityObjectCommitmentV1,
    pub(crate) source_authority_before: DurableSessionAuthorityV1,
    pub(crate) target_authority: DurableSessionAuthorityV1,
    pub(crate) target_authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) target_authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
    pub(crate) root_revision_after: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ForkSuccessorAllocationOutcomeV1 {
    Allocated(ForkSuccessorAllocationV1),
    Joined(ForkSuccessorAllocationV1),
}

pub(super) struct PreparedForkSuccessorAttachV1 {
    pub(super) reference: AuthorityObjectRefV1,
    pub(super) bytes: Vec<u8>,
    pub(super) requires_index: bool,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ForkSuccessorAllocationCrashPointV1 {
    BeforeRootPublication,
    AfterRootPublication,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ForkSuccessorAllocationError(String);

impl fmt::Display for ForkSuccessorAllocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ForkSuccessorAllocationError {}

impl HostSessionAuthority {
    pub(crate) fn allocate_fork_successor(
        &self,
        request: &AllocateForkSuccessorRequestV1,
    ) -> Result<ForkSuccessorAllocationOutcomeV1, ForkSuccessorAllocationError> {
        self.allocate_fork_successor_inner(request, None)
    }

    #[cfg(test)]
    pub(crate) fn allocate_fork_successor_with_crash_point(
        &self,
        request: &AllocateForkSuccessorRequestV1,
        crash_point: ForkSuccessorAllocationCrashPointV1,
    ) -> Result<ForkSuccessorAllocationOutcomeV1, ForkSuccessorAllocationError> {
        self.allocate_fork_successor_inner(request, Some(crash_point))
    }

    fn allocate_fork_successor_inner(
        &self,
        request: &AllocateForkSuccessorRequestV1,
        #[cfg(test)] crash_point: Option<ForkSuccessorAllocationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<ForkSuccessorAllocationOutcomeV1, ForkSuccessorAllocationError> {
        request.validate().map_err(|error| {
            ForkSuccessorAllocationError(format!(
                "invalid fork successor allocation request: {error}"
            ))
        })?;
        let request_commitment = canonical_commitment(request)?;
        let initial_root = self.read_fork_successor_root()?;
        if let Some(joined) = exact_join_or_conflict(&initial_root, request, &request_commitment)? {
            return Ok(ForkSuccessorAllocationOutcomeV1::Joined(joined));
        }
        if initial_root.root_revision != request.expected_source_root_revision {
            return Err(ForkSuccessorAllocationError(
                "fork successor source root revision is stale".into(),
            ));
        }

        let source_observation = source_observation(request)?;
        let source = match self.resolve_current_exact(
            &request.source_orchestration_session_id,
            Some(&source_observation),
        ) {
            Ok(source) => source,
            Err(error) => {
                return join_after_store_failure(
                    self,
                    request,
                    &request_commitment,
                    format!("resolve exact fork successor source authority: {error}"),
                )
            }
        };
        validate_exact_source(&source, request)?;
        if request.allocated_at.as_str() < source.authority.updated_at.as_str() {
            return Err(ForkSuccessorAllocationError(
                "fork successor allocation predates its exact source authority".into(),
            ));
        }
        validate_target_absent(&initial_root, request)?;

        let prepared_attach = match prepare_fork_successor_attach(self, &initial_root, &source) {
            Ok(prepared) => prepared,
            Err(error) => {
                return join_after_store_failure(
                    self,
                    request,
                    &request_commitment,
                    error.to_string(),
                )
            }
        };
        let target_attach_ref = prepared_attach.reference;
        let target_attach_bytes = prepared_attach.bytes;

        let target_authority = DurableSessionAuthorityV1 {
            schema_version: 1,
            orchestration_session_id: request.target_orchestration_session_id.clone(),
            shell_trace_session_id: request.target_shell_trace_session_id.clone(),
            authority_revision: 1,
            origin: source.authority.origin.clone(),
            authoritative_participant_lineage: request.resulting_authoritative_lineage.clone(),
            active_authoritative_participant_id: Some(
                request.target_authoritative_participant_id.clone(),
            ),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            host_attach_contract_ref: Some(target_attach_ref.clone()),
            retained_worker_refs: Vec::new(),
            internal_resume_handle_refs: Vec::new(),
            lifecycle_posture: HostSessionPostureV1::ParkedResumable,
            current_policy_ref: source.authority.current_policy_ref.clone(),
            current_policy_revision: source.authority.current_policy_revision.clone(),
            updated_at: request.allocated_at.clone(),
        };
        let target_authority_record_commitment =
            canonical_commitment(&authority_hash_input(&target_authority))?;
        let target_authoritative_lineage_commitment =
            canonical_commitment(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: request.target_orchestration_session_id.clone(),
                participant_ids: request.resulting_authoritative_lineage.clone(),
            })?;
        let root_revision_after = initial_root.root_revision.checked_add(1).ok_or_else(|| {
            ForkSuccessorAllocationError("fork successor root revision overflow".into())
        })?;
        let record = ForkSuccessorAllocationRecordV1 {
            schema_version: 1,
            request: request.clone(),
            request_commitment: request_commitment.clone(),
            source_authority_before: Box::new(source.authority.clone()),
            target_authority: Box::new(target_authority.clone()),
            target_authority_record_commitment: target_authority_record_commitment.clone(),
            target_authoritative_lineage_commitment: target_authoritative_lineage_commitment
                .clone(),
            root_revision_after,
        };
        let result = allocation_result(&record);
        let mut proposed = initial_root.clone();
        proposed.root_revision = root_revision_after;
        if prepared_attach.requires_index {
            proposed.object_index.insert(
                target_attach_ref.ref_id.clone(),
                AuthorityObjectIndexEntryV1 {
                    schema_version: 1,
                    ref_id: target_attach_ref.ref_id.clone(),
                    object_kind: target_attach_ref.object_kind,
                    object_schema_version: target_attach_ref.schema_version,
                    byte_length: target_attach_bytes.len() as u64,
                    storage_state: AuthorityObjectStorageStateV1::Present,
                },
            );
        }
        proposed.session_namespace_map.insert(
            request.target_orchestration_session_id.clone(),
            SessionNamespaceRecordV1::Authority(Box::new(target_authority)),
        );
        proposed
            .fork_successor_allocation_map
            .insert(request.allocation_id.clone(), record);
        proposed.validate().map_err(|error| {
            ForkSuccessorAllocationError(format!(
                "validate fork successor allocation candidate: {error}"
            ))
        })?;

        #[cfg(test)]
        if crash_point == Some(ForkSuccessorAllocationCrashPointV1::BeforeRootPublication) {
            return Err(ForkSuccessorAllocationError(
                "injected crash before fork successor root publication".into(),
            ));
        }
        if let Err(error) = store::commit_v3_root_exact_current_opened(
            self.trusted_root(),
            &initial_root,
            &proposed,
            || Ok(()),
        ) {
            return join_after_store_failure(
                self,
                request,
                &request_commitment,
                format!("commit fork successor allocation: {error}"),
            );
        }
        #[cfg(test)]
        if crash_point == Some(ForkSuccessorAllocationCrashPointV1::AfterRootPublication) {
            return Err(ForkSuccessorAllocationError(
                "injected crash after fork successor root publication".into(),
            ));
        }
        Ok(ForkSuccessorAllocationOutcomeV1::Allocated(result))
    }

    fn read_fork_successor_root(&self) -> Result<StateRootV3, ForkSuccessorAllocationError> {
        match store::read_opened_root_v2_or_v3(self.trusted_root()).map_err(|error| {
            ForkSuccessorAllocationError(format!("read fork successor authority root: {error}"))
        })? {
            VersionedStateRoot::V3(root) => Ok(root),
            VersionedStateRoot::V1(_) | VersionedStateRoot::V2(_) => {
                Err(ForkSuccessorAllocationError(
                    "fork successor allocation requires strict StateRootV3".into(),
                ))
            }
        }
    }
}

pub(super) fn prepare_fork_successor_attach(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    source: &ResolvedCurrentAuthorityV1,
) -> Result<PreparedForkSuccessorAttachV1, ForkSuccessorAllocationError> {
    let mut target_attach_contract = source.host_attach_contract.clone();
    target_attach_contract.continuity_resume_handle_ref = None;
    let target_attach_hash_input = HostAttachContractHashInputV1 {
        schema_version: 1,
        contract: target_attach_contract,
    };
    let bytes = canonical_json::to_vec(&target_attach_hash_input).map_err(|error| {
        ForkSuccessorAllocationError(format!("encode fork successor attach contract: {error}"))
    })?;
    let source_attach_ref = source
        .authority
        .host_attach_contract_ref
        .as_ref()
        .ok_or_else(|| {
            ForkSuccessorAllocationError(
                "fork successor source authority has no attach contract".into(),
            )
        })?;
    let reference = if source
        .host_attach_contract
        .continuity_resume_handle_ref
        .is_none()
    {
        source_attach_ref.clone()
    } else {
        deterministic_object_ref(
            AuthorityObjectKindV1::HostAttachContract,
            &target_attach_hash_input,
        )?
    };
    let requires_index =
        source_attach_ref != &reference && !root.object_index.contains_key(&reference.ref_id);
    if source_attach_ref != &reference {
        let result = if requires_index {
            store::prepare_typed_object_v3_opened(
                authority.trusted_root(),
                root.root_revision,
                &reference,
                &bytes,
                None,
            )
            .map(|_| ())
            .map_err(|error| error.to_string())
        } else {
            store::read_typed_object_v2_or_v3_opened(
                authority.trusted_root(),
                root.root_revision,
                &reference,
                None,
            )
            .map_err(|error| error.to_string())
            .and_then(|persisted| {
                if persisted == bytes {
                    Ok(())
                } else {
                    Err("authoritative fork successor attach contract bytes differ".into())
                }
            })
        };
        result.map_err(|error| {
            ForkSuccessorAllocationError(format!(
                "prepare or reuse fork successor attach contract: {error}"
            ))
        })?;
    }
    Ok(PreparedForkSuccessorAttachV1 {
        reference,
        bytes,
        requires_index,
    })
}

fn source_observation(
    request: &AllocateForkSuccessorRequestV1,
) -> Result<AuthorityObservationV1, ForkSuccessorAllocationError> {
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision,
        authority_record_commitment,
        authoritative_lineage_commitment,
        ..
    } = &request.source_authority_precondition
    else {
        return Err(ForkSuccessorAllocationError(
            "fork successor allocation requires ExpectedRevision".into(),
        ));
    };
    Ok(AuthorityObservationV1 {
        authority_store_id: request.authority_store_id.clone(),
        bootstrap_home: request.bootstrap_home.clone(),
        orchestration_session_id: request.source_orchestration_session_id.clone(),
        root_revision: request.expected_source_root_revision,
        authority_revision: *authority_revision,
        authority_record_commitment: authority_record_commitment.clone(),
        authoritative_lineage_commitment: authoritative_lineage_commitment.clone(),
    })
}

fn validate_exact_source(
    source: &ResolvedCurrentAuthorityV1,
    request: &AllocateForkSuccessorRequestV1,
) -> Result<(), ForkSuccessorAllocationError> {
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        active_authoritative_participant_id,
        lifecycle_posture,
        ..
    } = &request.source_authority_precondition
    else {
        return Err(ForkSuccessorAllocationError(
            "fork successor allocation requires ExpectedRevision".into(),
        ));
    };
    if source.authority.orchestration_session_id != request.source_orchestration_session_id
        || source.authority.shell_trace_session_id != request.source_shell_trace_session_id
        || source
            .authority
            .active_authoritative_participant_id
            .as_ref()
            != Some(active_authoritative_participant_id)
        || source.authority.authoritative_participant_lineage
            != request.source_authoritative_participant_lineage
        || source.authority.lifecycle_posture != *lifecycle_posture
        || source.authority.workspace_binding != request.workspace_binding
        || source.authority.world_binding != request.world_binding
        || matches!(
            source.authority.lifecycle_posture,
            HostSessionPostureV1::Terminal | HostSessionPostureV1::Invalid
        )
        || !source.host_attach_contract.capabilities.session_fork
    {
        return Err(ForkSuccessorAllocationError(
            "fork successor source identity, binding, lineage, posture, or capability mismatch"
                .into(),
        ));
    }
    Ok(())
}

fn validate_target_absent(
    root: &StateRootV3,
    request: &AllocateForkSuccessorRequestV1,
) -> Result<(), ForkSuccessorAllocationError> {
    if root
        .session_namespace_map
        .contains_key(&request.target_orchestration_session_id)
        || root.fork_successor_target_identity_is_occupied(request)
    {
        return Err(ForkSuccessorAllocationError(
            "fork successor target namespace or reserved identity already exists".into(),
        ));
    }
    Ok(())
}

fn exact_join_or_conflict(
    root: &StateRootV3,
    request: &AllocateForkSuccessorRequestV1,
    request_commitment: &AuthorityObjectCommitmentV1,
) -> Result<Option<ForkSuccessorAllocationV1>, ForkSuccessorAllocationError> {
    if let Some(record) = root
        .fork_successor_allocation_map
        .get(&request.allocation_id)
    {
        if record.request == *request && record.request_commitment == *request_commitment {
            return Ok(Some(allocation_result(record)));
        }
        return Err(ForkSuccessorAllocationError(
            "fork successor allocation identity already commits a different request".into(),
        ));
    }
    if root.fork_successor_allocation_map.values().any(|record| {
        record.request.request_id == request.request_id
            || record.request.target_orchestration_session_id
                == request.target_orchestration_session_id
            || record.request.target_shell_trace_session_id == request.target_shell_trace_session_id
            || record.request.target_authoritative_participant_id
                == request.target_authoritative_participant_id
    }) {
        return Err(ForkSuccessorAllocationError(
            "fork successor request or target identity conflicts with a completed allocation"
                .into(),
        ));
    }
    Ok(None)
}

fn join_after_store_failure(
    authority: &HostSessionAuthority,
    request: &AllocateForkSuccessorRequestV1,
    request_commitment: &AuthorityObjectCommitmentV1,
    original_error: String,
) -> Result<ForkSuccessorAllocationOutcomeV1, ForkSuccessorAllocationError> {
    if let Ok(root) = authority.read_fork_successor_root() {
        if let Some(joined) = exact_join_or_conflict(&root, request, request_commitment)? {
            return Ok(ForkSuccessorAllocationOutcomeV1::Joined(joined));
        }
    }
    Err(ForkSuccessorAllocationError(original_error))
}

fn allocation_result(record: &ForkSuccessorAllocationRecordV1) -> ForkSuccessorAllocationV1 {
    ForkSuccessorAllocationV1 {
        allocation_id: record.request.allocation_id.clone(),
        request_id: record.request.request_id.clone(),
        request_commitment: record.request_commitment.clone(),
        source_authority_before: record.source_authority_before.as_ref().clone(),
        target_authority: record.target_authority.as_ref().clone(),
        target_authority_record_commitment: record.target_authority_record_commitment.clone(),
        target_authoritative_lineage_commitment: record
            .target_authoritative_lineage_commitment
            .clone(),
        root_revision_after: record.root_revision_after,
    }
}

fn deterministic_object_ref<T: CanonicalHashInputV1>(
    object_kind: AuthorityObjectKindV1,
    value: &T,
) -> Result<AuthorityObjectRefV1, ForkSuccessorAllocationError> {
    let commitment = canonical_commitment(value)?;
    let AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } = &commitment else {
        unreachable!("canonical hash input produces canonical SHA-256")
    };
    Ok(AuthorityObjectRefV1 {
        schema_version: 1,
        ref_id: format!("ao_{}", &digest_hex[..32]),
        object_kind,
        commitment,
    })
}

fn canonical_commitment<T: CanonicalHashInputV1>(
    value: &T,
) -> Result<AuthorityObjectCommitmentV1, ForkSuccessorAllocationError> {
    super::hash::canonical_sha256(value)
        .map(|digest_hex| AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex })
        .map_err(|error| {
            ForkSuccessorAllocationError(format!("commit fork successor canonical input: {error}"))
        })
}

fn authority_hash_input(value: &DurableSessionAuthorityV1) -> DurableSessionAuthorityHashInputV1 {
    DurableSessionAuthorityHashInputV1 {
        schema_version: value.schema_version,
        orchestration_session_id: value.orchestration_session_id.clone(),
        shell_trace_session_id: value.shell_trace_session_id.clone(),
        authority_revision: value.authority_revision,
        origin: value.origin.clone(),
        authoritative_participant_lineage: value.authoritative_participant_lineage.clone(),
        active_authoritative_participant_id: value.active_authoritative_participant_id.clone(),
        workspace_binding: value.workspace_binding.clone(),
        world_binding: value.world_binding.clone(),
        host_attach_contract_ref: value.host_attach_contract_ref.clone(),
        retained_worker_refs: value.retained_worker_refs.clone(),
        internal_resume_handle_refs: value.internal_resume_handle_refs.clone(),
        lifecycle_posture: value.lifecycle_posture,
        current_policy_ref: value.current_policy_ref.clone(),
        current_policy_revision: value.current_policy_revision.clone(),
    }
}
