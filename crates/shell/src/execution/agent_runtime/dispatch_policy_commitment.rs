//! Durable E2 policy commitments and retained-worker policy caps.
//!
//! This module owns only immutable dispatch-policy authority. It cannot admit,
//! route, resume, cancel, or settle retained workers.

use std::collections::BTreeMap;
use std::fmt;

use base64::Engine as _;
use chrono::{SecondsFormat, Utc};
use rand::RngCore;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use transport_api_types::{
    DispatchPolicyCommitmentRefCarrierV1, DispatchPolicyNarrowingPatchV1,
    DispatchPolicySnapshotCarrierV1, E2DispatchPolicyReservationRefCarrierV1,
    E2LaunchRequestCommitmentV1, E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1,
    PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsWriteV3,
    RetainedTurnPolicyCommitmentSubjectV1, WorldBindingRefV1, WorldFsDenyEnforcementV3,
};
use uuid::Uuid;

use super::dispatch_contract::ValidatedWorldDispatchRequestV1;
use super::host_session_authority::canonical_json;
use super::host_session_authority::schema::{
    AuthorityObjectCommitmentV1, AuthorityObjectRefV1, TimestampV1, WorldBindingV1,
};
use super::host_session_authority::store::{
    dispatch_policy_commitment_storage_for_authority, BootstrapError,
};
use super::host_session_authority::HostSessionAuthority;
use super::retained_worker_runtime::{
    CanonicalValidatedSpawnRequestV1, RetainedWorkerAdmissionCommitmentV1,
    RetainedWorkerAdmissionPlanV1, RetainedWorkerAdmissionRecordV1,
    RetainedWorkerAdmissionRegistrationV1, RetainedWorkerAdmissionStateV1,
};
use super::state_store::{
    AcceptedWorldWorkIdentityV1, RuntimeAcceptanceEvidenceV1, WorldWorkAcceptanceRecordV1,
};
use super::world_work_execution_supervisor::WorldWorkExecutionClaimV1;

const REQUEST_COMMITMENT_DOMAIN: &str = "substrate.e2.fresh-spawn-validated-request.v1";
const RESERVATION_HASH_DOMAIN: &str = "substrate.e2.dispatch-policy-commitment-reservation.v1";
const COMMITMENT_HASH_DOMAIN: &str = "substrate.e2.dispatch-policy-commitment.v1";
const PATCH_HASH_DOMAIN: &str = "substrate.e2.dispatch-policy-patch.v1";
const CLAIM_LINK_HASH_DOMAIN: &str = "substrate.e2.world-work-execution-claim-link.v1";
const STABLE_ADMISSION_HASH_DOMAIN: &str = "substrate.e2.b3-2a-stable-admission-link.v1";
const FORK_REQUEST_HASH_DOMAIN: &str = "substrate.e2.fork-dispatch-request.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DispatchPolicyCommitmentError(String);

impl DispatchPolicyCommitmentError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for DispatchPolicyCommitmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for DispatchPolicyCommitmentError {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispatchPolicyCommitmentReservationRefV1 {
    pub(crate) authority_store_id: String,
    pub(crate) reservation_id: String,
    pub(crate) reservation_hash: String,
}

/// Storage-authenticated, non-serializable E2 authority handed to B3.2a.
///
/// Deliberately not `Clone`: each value represents one completed registry
/// authentication. No key identity, digest, or request preimage crosses this
/// seam.
pub(crate) struct AuthenticatedFreshSpawnReservationProofV1 {
    reservation_ref: DispatchPolicyCommitmentReservationRefV1,
    request_id: String,
    idempotency_key: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    parent_policy_ref: AuthorityObjectRefV1,
    parent_policy_revision: String,
    retained_participant_id: String,
    bootstrap_run_id: String,
    narrowing_attestation: Option<OpaqueFreshSpawnNarrowingAttestationV1>,
}

struct OpaqueFreshSpawnNarrowingAttestationV1;

impl AuthenticatedFreshSpawnReservationProofV1 {
    pub(crate) fn retained_participant_id(&self) -> &str {
        &self.retained_participant_id
    }

    pub(crate) fn bootstrap_run_id(&self) -> &str {
        &self.bootstrap_run_id
    }

    pub(crate) fn permits_capability_narrowing(&self) -> bool {
        self.narrowing_attestation.is_some()
    }

    pub(crate) fn validate_admission_plan_binding(
        &self,
        plan: &RetainedWorkerAdmissionPlanV1,
    ) -> Result<(), DispatchPolicyCommitmentError> {
        let request = &plan.spawn_request;
        if self.reservation_ref.authority_store_id != plan.exact_authority.authority_store_id
            || self.request_id != request.request_id
            || self.idempotency_key != request.idempotency_key
            || self.orchestration_session_id != request.orchestration_session_id
            || self.orchestration_session_id != plan.exact_authority.orchestration_session_id
            || self.caller_participant_id != request.caller_participant_id
            || self.target_backend_id != request.target_backend_id
            || self.world_id != request.world_id
            || self.world_generation != request.world_generation
            || self.parent_policy_ref != plan.policy_and_admission_cap.current_policy_ref
            || self.parent_policy_revision
                != plan.policy_and_admission_cap.current_policy.policy_revision
        {
            return Err(DispatchPolicyCommitmentError::new(
                "fresh-spawn reservation proof does not match the admission plan",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum DispatchPolicyCommitmentSubjectV1 {
    EphemeralWork {
        task_run_id: String,
    },
    RetainedWorkerLaunch {
        retained_participant_id: String,
        bootstrap_run_id: String,
    },
    RetainedWorkerTurn {
        retained_participant_id: String,
        active_run_id: String,
        message_id: String,
    },
    RetainedWorkerFork {
        source_participant_id: String,
        child_participant_id: String,
        bootstrap_run_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum DispatchPolicyCommitmentSubjectKeyV1 {
    EphemeralWork {
        task_run_id: String,
    },
    RetainedWorkerLaunch,
    RetainedWorkerTurn {
        active_run_id: String,
        message_id: String,
    },
    RetainedWorkerFork {
        source_participant_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispatchPolicyCommitmentLookupKeyV1 {
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) request_id: String,
    pub(crate) subject: DispatchPolicyCommitmentSubjectKeyV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImmutableBytesRefV1 {
    pub(crate) authority_store_id: String,
    pub(crate) object_ref: String,
    pub(crate) byte_length: u64,
    pub(crate) sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum ImmutableBytesMaterialV1 {
    Inline {
        bytes_base64: String,
        byte_length: u64,
    },
    Durable {
        immutable_bytes_ref: ImmutableBytesRefV1,
    },
}

impl ImmutableBytesMaterialV1 {
    fn inline(bytes: &[u8]) -> Self {
        Self::Inline {
            bytes_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            byte_length: bytes.len() as u64,
        }
    }

    fn resolve_inline(&self) -> Result<Vec<u8>, DispatchPolicyCommitmentError> {
        let Self::Inline {
            bytes_base64,
            byte_length,
        } = self
        else {
            return Err(DispatchPolicyCommitmentError::new(
                "durable immutable bytes require authority-store resolution",
            ));
        };
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(bytes_base64)
            .map_err(|_| DispatchPolicyCommitmentError::new("invalid canonical inline base64"))?;
        if bytes.len() as u64 != *byte_length
            || base64::engine::general_purpose::STANDARD.encode(&bytes) != *bytes_base64
        {
            return Err(DispatchPolicyCommitmentError::new(
                "inline immutable bytes length or canonical base64 mismatch",
            ));
        }
        Ok(bytes)
    }
}

pub(crate) fn applied_dispatch_policy_patch_identity(
    patch: Option<&DispatchPolicyNarrowingPatchV1>,
) -> Result<AppliedDispatchPolicyPatchIdentityV1, DispatchPolicyCommitmentError> {
    let Some(patch) = patch else {
        return Ok(AppliedDispatchPolicyPatchIdentityV1::UnchangedParent);
    };
    patch
        .validate()
        .map_err(|_| DispatchPolicyCommitmentError::new("invalid E1 dispatch narrowing patch"))?;
    if patch
        .restricted_policy_patch
        .world_fs
        .as_ref()
        .is_none_or(|world_fs| world_fs.is_empty())
    {
        return Err(DispatchPolicyCommitmentError::new(
            "a supplied dispatch narrowing patch must not be empty",
        ));
    }
    let bytes = encode_canonical(patch)?;
    Ok(AppliedDispatchPolicyPatchIdentityV1::RestrictedWorldFs {
        patch_schema_version: 1,
        patch_hash: patch_identity_hash(&bytes)?,
        canonical_patch: ImmutableBytesMaterialV1::inline(&bytes),
    })
}

pub(crate) fn dispatch_policy_narrowing_reason(
    patch: Option<&DispatchPolicyNarrowingPatchV1>,
) -> Option<String> {
    patch.and_then(|patch| patch.reason.clone())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum AppliedDispatchPolicyPatchIdentityV1 {
    UnchangedParent,
    RestrictedWorldFs {
        patch_schema_version: u32,
        canonical_patch: ImmutableBytesMaterialV1,
        patch_hash: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispatchPolicyCommitmentRefV1 {
    pub(crate) authority_store_id: String,
    pub(crate) commitment_id: String,
    pub(crate) exact_linkage_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum RetainedWorkerCapLinkV1 {
    ThisCommitment,
    Existing {
        cap_ref: DispatchPolicyCommitmentRefV1,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum PolicyCommitmentStatusV1 {
    Immutable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum FreshSpawnRequestCommitmentAlgorithmV1 {
    HmacSha256,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FreshSpawnValidatedRequestCommitmentV1 {
    pub(crate) schema_version: u32,
    pub(crate) algorithm: FreshSpawnRequestCommitmentAlgorithmV1,
    pub(crate) key_id: String,
    pub(crate) domain: String,
    pub(crate) digest_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicyCommitmentKeyHeaderV1 {
    schema_version: u32,
    authority_store_id: String,
    key_id: String,
    created_at: TimestampV1,
    algorithm: FreshSpawnRequestCommitmentAlgorithmV1,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicyCommitmentKeyEnvelopeV1 {
    schema_version: u32,
    authority_store_id: String,
    key_id: String,
    created_at: TimestampV1,
    algorithm: FreshSpawnRequestCommitmentAlgorithmV1,
    secret_key: [u8; 32],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldWorkExecutionClaimDurableKeyV1 {
    pub(crate) supervisor_schema_version: u32,
    pub(crate) executions_by_acceptance_record_id_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldWorkExecutionClaimLinkV1 {
    pub(crate) authority_store_id: String,
    pub(crate) durable_claim_key: WorldWorkExecutionClaimDurableKeyV1,
    pub(crate) acceptance_record_id: String,
    pub(crate) acceptance_record_revision: u64,
    pub(crate) claim_revision: u64,
    pub(crate) observer_instance_id: String,
    pub(crate) observer_epoch: u64,
    pub(crate) claim_preimage: ImmutableBytesMaterialV1,
    pub(crate) claim_linkage_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAdmissionRegistryKeyV1 {
    pub(crate) orchestration_session_id: String,
    pub(crate) retained_participant_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAdmissionStableSourceFieldsV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) admission_authority_revision: u64,
    pub(crate) admission_authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) retained_participant_id: String,
    pub(crate) bootstrap_run_id: String,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) world_binding: WorldBindingV1,
    pub(crate) current_policy_ref: AuthorityObjectRefV1,
    pub(crate) current_policy_revision: String,
    pub(crate) max_live_retained_workers: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAdmissionStableIdentityLinkV1 {
    pub(crate) registry_key: RetainedWorkerAdmissionRegistryKeyV1,
    pub(crate) stable_source_fields: RetainedWorkerAdmissionStableSourceFieldsV1,
    pub(crate) registration: RetainedWorkerAdmissionRegistrationV1,
    pub(crate) stable_identity_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum PolicyCommitmentAuthorityLinkV1 {
    B1 {
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    },
    RetainedAdmission {
        stable_admission_identity: RetainedWorkerAdmissionStableIdentityLinkV1,
    },
    ForkDispatch {
        canonical_validated_dispatch_request_sha256: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProposedRetainedWorkerCapV1 {
    pub(crate) commitment_id: String,
    pub(crate) policy_snapshot_ref: AuthorityObjectRefV1,
    pub(crate) policy_snapshot_hash: String,
    pub(crate) policy_snapshot_revision: String,
    pub(crate) retained_worker_cap_link: RetainedWorkerCapLinkV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispatchPolicyCommitmentReservationV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) reservation_id: String,
    pub(crate) lookup_key: DispatchPolicyCommitmentLookupKeyV1,
    pub(crate) subject: DispatchPolicyCommitmentSubjectV1,
    pub(crate) validated_request_commitment: FreshSpawnValidatedRequestCommitmentV1,
    pub(crate) request_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) caller_backend_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) retained_participant_id: String,
    pub(crate) bootstrap_run_id: String,
    pub(crate) parent_policy_ref: AuthorityObjectRefV1,
    pub(crate) parent_policy_revision: String,
    pub(crate) applied_patch: AppliedDispatchPolicyPatchIdentityV1,
    pub(crate) policy_snapshot_bytes: ImmutableBytesMaterialV1,
    pub(crate) policy_snapshot_ref: AuthorityObjectRefV1,
    pub(crate) policy_snapshot_hash: String,
    pub(crate) policy_snapshot_revision: String,
    pub(crate) reason: Option<String>,
    pub(crate) proposed_commitment_id: String,
    pub(crate) proposed_worker_cap: ProposedRetainedWorkerCapV1,
    pub(crate) created_revision: u64,
    pub(crate) created_at: TimestampV1,
    pub(crate) reservation_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispatchPolicyCommitmentV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) commitment_id: String,
    pub(crate) subject: DispatchPolicyCommitmentSubjectV1,
    pub(crate) request_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) caller_backend_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) authority_link: PolicyCommitmentAuthorityLinkV1,
    pub(crate) execution_claim_link: Option<WorldWorkExecutionClaimLinkV1>,
    pub(crate) fresh_spawn_reservation_ref: Option<DispatchPolicyCommitmentReservationRefV1>,
    pub(crate) fresh_spawn_validated_request_commitment:
        Option<FreshSpawnValidatedRequestCommitmentV1>,
    pub(crate) parent_policy_ref: AuthorityObjectRefV1,
    pub(crate) parent_policy_revision: String,
    pub(crate) applied_patch: AppliedDispatchPolicyPatchIdentityV1,
    pub(crate) policy_snapshot_bytes: ImmutableBytesMaterialV1,
    pub(crate) policy_snapshot_ref: AuthorityObjectRefV1,
    pub(crate) policy_snapshot_hash: String,
    pub(crate) policy_snapshot_revision: String,
    pub(crate) reason: Option<String>,
    pub(crate) retained_worker_cap_link: Option<RetainedWorkerCapLinkV1>,
    pub(crate) source_worker_cap_ref: Option<DispatchPolicyCommitmentRefV1>,
    pub(crate) created_revision: u64,
    pub(crate) application_revision: u64,
    pub(crate) status: PolicyCommitmentStatusV1,
    pub(crate) created_at: TimestampV1,
    pub(crate) exact_linkage_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum DispatchPolicyCommitmentIndexEntryV1 {
    FreshSpawnReserved {
        reservation_ref: DispatchPolicyCommitmentReservationRefV1,
    },
    Committed {
        reservation_ref: Option<DispatchPolicyCommitmentReservationRefV1>,
        fresh_spawn_validated_request_commitment: Option<FreshSpawnValidatedRequestCommitmentV1>,
        commitment_ref: DispatchPolicyCommitmentRefV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicyCommitmentIndexRecordV1 {
    lookup_key: DispatchPolicyCommitmentLookupKeyV1,
    entry: DispatchPolicyCommitmentIndexEntryV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicyCommitmentRegistryV1 {
    schema_version: u32,
    authority_store_id: String,
    request_commitment_key: DispatchPolicyCommitmentKeyHeaderV1,
    reservations_by_id: BTreeMap<String, DispatchPolicyCommitmentReservationV1>,
    request_subject_index: BTreeMap<String, DispatchPolicyCommitmentIndexRecordV1>,
    commitments_by_id: BTreeMap<String, DispatchPolicyCommitmentV1>,
    retained_worker_caps_by_participant_id: BTreeMap<String, DispatchPolicyCommitmentRefV1>,
}

#[derive(Clone, Debug)]
pub(crate) struct ValidatedPolicySnapshotMaterialV1 {
    snapshot: PolicySnapshotV3,
    bytes: Vec<u8>,
    snapshot_ref: AuthorityObjectRefV1,
    hash: String,
    revision: String,
}

impl ValidatedPolicySnapshotMaterialV1 {
    pub(crate) fn snapshot(&self) -> &PolicySnapshotV3 {
        &self.snapshot
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn snapshot_ref(&self) -> &AuthorityObjectRefV1 {
        &self.snapshot_ref
    }

    pub(crate) fn hash(&self) -> &str {
        &self.hash
    }

    pub(crate) fn revision(&self) -> &str {
        &self.revision
    }
}

pub(crate) fn validate_policy_snapshot_material(
    expected: &PolicySnapshotV3,
    policy_snapshot_bytes: &[u8],
    policy_snapshot_ref: &AuthorityObjectRefV1,
    policy_snapshot_hash: &str,
    policy_snapshot_revision: &str,
) -> Result<ValidatedPolicySnapshotMaterialV1, DispatchPolicyCommitmentError> {
    let decoded: PolicySnapshotV3 = serde_json::from_slice(policy_snapshot_bytes)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode E1 PolicySnapshotV3 bytes"))?;
    let reserialized = serde_json::to_vec(&decoded)
        .map_err(|_| DispatchPolicyCommitmentError::new("reserialize E1 PolicySnapshotV3"))?;
    let expected_bytes = serde_json::to_vec(expected)
        .map_err(|_| DispatchPolicyCommitmentError::new("serialize expected PolicySnapshotV3"))?;
    let hash = sha256_hex(policy_snapshot_bytes);
    if reserialized != policy_snapshot_bytes
        || expected_bytes != policy_snapshot_bytes
        || hash != policy_snapshot_hash
        || policy_snapshot_revision.is_empty()
    {
        return Err(DispatchPolicyCommitmentError::new(
            "E1 policy snapshot bytes, hash, revision, or decoded identity mismatch",
        ));
    }
    Ok(ValidatedPolicySnapshotMaterialV1 {
        snapshot: decoded,
        bytes: policy_snapshot_bytes.to_vec(),
        snapshot_ref: policy_snapshot_ref.clone(),
        hash,
        revision: policy_snapshot_revision.to_string(),
    })
}

#[derive(Clone)]
pub(crate) struct FreshSpawnReservationInputV1 {
    pub(crate) spawn_request: CanonicalValidatedSpawnRequestV1,
    pub(crate) caller_backend_id: String,
    pub(crate) parent_policy_ref: AuthorityObjectRefV1,
    pub(crate) parent_policy_revision: String,
    pub(crate) applied_patch: AppliedDispatchPolicyPatchIdentityV1,
    pub(crate) policy_snapshot: ValidatedPolicySnapshotMaterialV1,
    pub(crate) reason: Option<String>,
}

pub(crate) struct FreshSpawnReservationOutcomeV1 {
    pub(crate) reservation_ref: DispatchPolicyCommitmentReservationRefV1,
    #[cfg(test)]
    pub(crate) retained_participant_id: String,
    #[cfg(test)]
    pub(crate) bootstrap_run_id: String,
    #[cfg(test)]
    pub(crate) proposed_commitment_id: String,
    #[cfg(test)]
    pub(crate) proof: AuthenticatedFreshSpawnReservationProofV1,
    #[cfg(test)]
    pub(crate) joined: bool,
}

#[derive(Clone)]
pub(crate) struct AcceptedWorkPolicyCommitmentInputV1 {
    pub(crate) idempotency_key: String,
    pub(crate) acceptance: WorldWorkAcceptanceRecordV1,
    pub(crate) execution_claim: WorldWorkExecutionClaimV1,
    pub(crate) applied_patch: AppliedDispatchPolicyPatchIdentityV1,
    pub(crate) policy_snapshot: ValidatedPolicySnapshotMaterialV1,
    pub(crate) reason: Option<String>,
    pub(crate) retained_worker_cap_ref: Option<DispatchPolicyCommitmentRefV1>,
}

pub(crate) struct ForkPolicyCommitmentInputV1 {
    pub(crate) request_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) caller_backend_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) source_participant_id: String,
    pub(crate) canonical_validated_dispatch_request: Vec<u8>,
    pub(crate) parent_policy_ref: AuthorityObjectRefV1,
    pub(crate) parent_policy_revision: String,
    pub(crate) applied_patch: AppliedDispatchPolicyPatchIdentityV1,
    pub(crate) authenticated_policy: AuthenticatedForkPolicyMaterialV1,
    pub(crate) reason: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct PersistedDispatchPolicyCommitmentV1(DispatchPolicyCommitmentV1);

impl PersistedDispatchPolicyCommitmentV1 {
    #[cfg(test)]
    pub(crate) fn record(&self) -> &DispatchPolicyCommitmentV1 {
        &self.0
    }

    pub(crate) fn commitment_ref(&self) -> DispatchPolicyCommitmentRefV1 {
        DispatchPolicyCommitmentRefV1 {
            authority_store_id: self.0.authority_store_id.clone(),
            commitment_id: self.0.commitment_id.clone(),
            exact_linkage_hash: self.0.exact_linkage_hash.clone(),
        }
    }

    pub(crate) fn fork_child_participant_id(&self) -> Option<&str> {
        match &self.0.subject {
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
                child_participant_id,
                ..
            } => Some(child_participant_id),
            _ => None,
        }
    }

    pub(crate) fn fork_bootstrap_run_id(&self) -> Option<&str> {
        match &self.0.subject {
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
                bootstrap_run_id, ..
            } => Some(bootstrap_run_id),
            _ => None,
        }
    }
}

pub(crate) struct AuthenticatedDispatchPolicyCommitmentV1 {
    record: DispatchPolicyCommitmentV1,
    snapshot: PolicySnapshotV3,
    snapshot_bytes: Vec<u8>,
}

impl AuthenticatedDispatchPolicyCommitmentV1 {
    pub(crate) fn commitment_ref(&self) -> DispatchPolicyCommitmentRefV1 {
        DispatchPolicyCommitmentRefV1 {
            authority_store_id: self.record.authority_store_id.clone(),
            commitment_id: self.record.commitment_id.clone(),
            exact_linkage_hash: self.record.exact_linkage_hash.clone(),
        }
    }

    pub(crate) fn subject(&self) -> &DispatchPolicyCommitmentSubjectV1 {
        &self.record.subject
    }

    pub(crate) fn authority_store_id(&self) -> &str {
        &self.record.authority_store_id
    }

    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.record.orchestration_session_id
    }

    pub(crate) fn retained_participant_id(&self) -> Option<&str> {
        match &self.record.subject {
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
                retained_participant_id,
                ..
            }
            | DispatchPolicyCommitmentSubjectV1::RetainedWorkerTurn {
                retained_participant_id,
                ..
            } => Some(retained_participant_id),
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
                child_participant_id,
                ..
            } => Some(child_participant_id),
            DispatchPolicyCommitmentSubjectV1::EphemeralWork { .. } => None,
        }
    }

    pub(crate) fn policy_snapshot(&self) -> &PolicySnapshotV3 {
        &self.snapshot
    }

    pub(crate) fn policy_snapshot_bytes(&self) -> &[u8] {
        &self.snapshot_bytes
    }

    pub(crate) fn policy_snapshot_hash(&self) -> &str {
        &self.record.policy_snapshot_hash
    }

    pub(crate) fn retained_worker_cap_link(&self) -> Option<&RetainedWorkerCapLinkV1> {
        self.record.retained_worker_cap_link.as_ref()
    }

    pub(crate) fn target_backend_id(&self) -> &str {
        &self.record.target_backend_id
    }

    pub(crate) fn world_id(&self) -> &str {
        &self.record.world_id
    }

    pub(crate) fn world_generation(&self) -> u64 {
        self.record.world_generation
    }

    pub(crate) fn member_launch_activation_carrier(
        &self,
    ) -> Result<E2MemberLaunchActivationCarrierV1, DispatchPolicyCommitmentError> {
        if self.record.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment) {
            return Err(DispatchPolicyCommitmentError::new(
                "E2 member launch commitment is not its own immutable worker cap",
            ));
        }
        let commitment_ref = DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: self.record.authority_store_id.clone(),
            commitment_id: self.record.commitment_id.clone(),
            exact_linkage_hash: self.record.exact_linkage_hash.clone(),
        };
        let (
            launch_kind,
            retained_participant_id,
            bootstrap_run_id,
            source_participant_id,
            reservation_ref,
            request_commitment,
        ) = match &self.record.subject {
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
                retained_participant_id,
                bootstrap_run_id,
            } => {
                let reservation = self
                    .record
                    .fresh_spawn_reservation_ref
                    .as_ref()
                    .ok_or_else(|| {
                        DispatchPolicyCommitmentError::new(
                            "Fresh Spawn commitment omitted its durable E2 reservation",
                        )
                    })?;
                let request = self
                    .record
                    .fresh_spawn_validated_request_commitment
                    .as_ref()
                    .ok_or_else(|| {
                        DispatchPolicyCommitmentError::new(
                            "Fresh Spawn commitment omitted its request HMAC",
                        )
                    })?;
                (
                    E2MemberLaunchKindV1::FreshSpawn,
                    retained_participant_id.clone(),
                    bootstrap_run_id.clone(),
                    None,
                    Some(E2DispatchPolicyReservationRefCarrierV1 {
                        authority_store_id: reservation.authority_store_id.clone(),
                        reservation_id: reservation.reservation_id.clone(),
                        reservation_hash: reservation.reservation_hash.clone(),
                    }),
                    E2LaunchRequestCommitmentV1::HmacSha256 {
                        key_id: request.key_id.clone(),
                        domain: request.domain.clone(),
                        digest_hex: request.digest_hex.clone(),
                    },
                )
            }
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
                source_participant_id,
                child_participant_id,
                bootstrap_run_id,
            } => {
                let PolicyCommitmentAuthorityLinkV1::ForkDispatch {
                    canonical_validated_dispatch_request_sha256,
                } = &self.record.authority_link
                else {
                    return Err(DispatchPolicyCommitmentError::new(
                        "Fork launch commitment omitted its authenticated request binding",
                    ));
                };
                (
                    E2MemberLaunchKindV1::Fork,
                    child_participant_id.clone(),
                    bootstrap_run_id.clone(),
                    Some(source_participant_id.clone()),
                    None,
                    E2LaunchRequestCommitmentV1::CanonicalSha256 {
                        domain: FORK_REQUEST_HASH_DOMAIN.to_string(),
                        digest_hex: canonical_validated_dispatch_request_sha256.clone(),
                    },
                )
            }
            _ => {
                return Err(DispatchPolicyCommitmentError::new(
                    "policy commitment subject is not an E2 member launch",
                ))
            }
        };
        let carrier = E2MemberLaunchActivationCarrierV1 {
            schema_version: 1,
            activation_id: format!("e2a_{}", &self.record.exact_linkage_hash[..32]),
            launch_kind,
            reservation_ref,
            commitment_ref: commitment_ref.clone(),
            immutable_worker_cap_ref: commitment_ref,
            immutable_worker_cap_created_revision: self.record.created_revision,
            immutable_worker_cap_application_revision: self.record.application_revision,
            policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD
                .encode(&self.snapshot_bytes),
            policy_snapshot_byte_length: self.snapshot_bytes.len() as u64,
            policy_snapshot_ref: transport_policy_ref(&self.record.policy_snapshot_ref),
            policy_snapshot_hash: self.record.policy_snapshot_hash.clone(),
            policy_snapshot_revision: self.record.policy_snapshot_revision.clone(),
            reason: self.record.reason.clone(),
            request_id: self.record.request_id.clone(),
            idempotency_key: self.record.idempotency_key.clone(),
            orchestration_session_id: self.record.orchestration_session_id.clone(),
            caller_participant_id: self.record.caller_participant_id.clone(),
            caller_backend_id: self.record.caller_backend_id.clone(),
            target_backend_id: self.record.target_backend_id.clone(),
            retained_participant_id,
            bootstrap_run_id,
            source_participant_id,
            target_world: WorldBindingRefV1 {
                world_id: self.record.world_id.clone(),
                world_generation: self.record.world_generation,
            },
            parent_policy_ref: transport_policy_ref(&self.record.parent_policy_ref),
            parent_policy_revision: self.record.parent_policy_revision.clone(),
            request_commitment,
            registry_publication_revision: self.record.application_revision,
        };
        carrier
            .validate()
            .map_err(DispatchPolicyCommitmentError::new)?;
        Ok(carrier)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PolicyCommitmentCompatibilityReasonV1 {
    MissingCanonicalCapBytes,
    MissingImmutableCapBytesRef,
    CapHashMismatch,
    UnsupportedCapSchema,
    CapSubjectOrLinkageMismatch,
}

/// Cloneable locator only; callers must re-authenticate it against the locked
/// registry before constructing a transport carrier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DurablePolicyCommitmentDescriptorV1 {
    commitment_ref: DispatchPolicyCommitmentRefV1,
    orchestration_session_id: String,
    retained_participant_id: String,
    launch_parent_policy_ref: AuthorityObjectRefV1,
    launch_parent_policy_revision: String,
}

impl DurablePolicyCommitmentDescriptorV1 {
    pub(crate) fn commitment_ref(&self) -> &DispatchPolicyCommitmentRefV1 {
        &self.commitment_ref
    }

    pub(crate) fn authority_store_id(&self) -> &str {
        &self.commitment_ref.authority_store_id
    }

    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.orchestration_session_id
    }

    pub(crate) fn retained_participant_id(&self) -> &str {
        &self.retained_participant_id
    }

    pub(crate) fn launch_parent_policy_ref(&self) -> &AuthorityObjectRefV1 {
        &self.launch_parent_policy_ref
    }

    pub(crate) fn launch_parent_policy_revision(&self) -> &str {
        &self.launch_parent_policy_revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum ResolvedPolicyCommitmentCompatibilityV1 {
    Compatible {
        cap: DurablePolicyCommitmentDescriptorV1,
    },
    UnsupportedLegacyState {
        retained_participant_id: String,
        reason: PolicyCommitmentCompatibilityReasonV1,
    },
}

#[derive(Clone)]
pub(crate) struct RetainedTurnPolicyResolutionInputV1 {
    pub(crate) cap: DurablePolicyCommitmentDescriptorV1,
    /// Exact E1 current-parent snapshot after applying the optional turn patch.
    pub(crate) current_parent_and_turn_patch: ValidatedPolicySnapshotMaterialV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) caller_backend_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) retained_participant_id: String,
    pub(crate) active_run_id: String,
    pub(crate) message_id: Option<String>,
    pub(crate) turn_patch: Option<DispatchPolicyNarrowingPatchV1>,
}

/// Registry-authenticated, non-cloneable transport material for one retained turn.
#[derive(Debug)]
pub(crate) struct AuthenticatedRetainedTurnPolicyMaterialV1 {
    carrier: DispatchPolicySnapshotCarrierV1,
}

impl AuthenticatedRetainedTurnPolicyMaterialV1 {
    pub(crate) fn carrier(&self) -> &DispatchPolicySnapshotCarrierV1 {
        &self.carrier
    }

    pub(crate) fn into_carrier(self) -> DispatchPolicySnapshotCarrierV1 {
        self.carrier
    }
}

#[derive(Clone)]
pub(crate) struct ForkPolicyResolutionInputV1 {
    pub(crate) source_cap: DurablePolicyCommitmentDescriptorV1,
    /// Exact E1 current-parent snapshot after applying the optional fork patch.
    pub(crate) current_parent_and_fork_patch: ValidatedPolicySnapshotMaterialV1,
    pub(crate) request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) caller_backend_id: String,
    pub(crate) target_backend_id: String,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) source_participant_id: String,
    pub(crate) fork_patch: Option<DispatchPolicyNarrowingPatchV1>,
}

#[derive(Debug)]
pub(crate) struct AuthenticatedForkPolicyMaterialV1 {
    snapshot: ValidatedPolicySnapshotMaterialV1,
    source_worker_cap_ref: DispatchPolicyCommitmentRefV1,
}

impl AuthenticatedForkPolicyMaterialV1 {
    pub(crate) fn snapshot(&self) -> &ValidatedPolicySnapshotMaterialV1 {
        &self.snapshot
    }

    pub(crate) fn source_worker_cap_ref(&self) -> &DispatchPolicyCommitmentRefV1 {
        &self.source_worker_cap_ref
    }
}

pub(crate) fn initialize_dispatch_policy_commitment_registry(
    authority: &HostSessionAuthority,
) -> Result<(), DispatchPolicyCommitmentError> {
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    let authority_store_id = storage.authority_store_id().to_string();
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        if let Some(bytes) = transaction.read_registry()? {
            let registry: DispatchPolicyCommitmentRegistryV1 =
                retain_semantic(decode_canonical(&bytes), &mut semantic_failure)?;
            let keys = transaction.read_keys()?;
            retain_semantic(
                validate_registry_and_key(&registry, &authority_store_id, &keys).map(|_| ()),
                &mut semantic_failure,
            )?;
            return Ok(());
        }

        for (name, _) in transaction.read_keys()? {
            transaction.remove_key(&name)?;
        }
        let mut entropy = [0_u8; 16];
        let mut nonce = [0_u8; 16];
        let mut secret_key = [0_u8; 32];
        let mut random = rand::rngs::OsRng;
        random.fill_bytes(&mut entropy);
        random.fill_bytes(&mut nonce);
        random.fill_bytes(&mut secret_key);
        let created_at = current_timestamp().map_err(|error| {
            semantic_failure = Some(error);
            BootstrapError::dispatch_policy_commitment_semantic()
        })?;
        let key_id = format!("dpk_{}", lower_hex(&entropy));
        let header = DispatchPolicyCommitmentKeyHeaderV1 {
            schema_version: 1,
            authority_store_id: authority_store_id.clone(),
            key_id: key_id.clone(),
            created_at: created_at.clone(),
            algorithm: FreshSpawnRequestCommitmentAlgorithmV1::HmacSha256,
        };
        let envelope = DispatchPolicyCommitmentKeyEnvelopeV1 {
            schema_version: 1,
            authority_store_id: authority_store_id.clone(),
            key_id: key_id.clone(),
            created_at,
            algorithm: FreshSpawnRequestCommitmentAlgorithmV1::HmacSha256,
            secret_key,
        };
        let envelope_bytes = retain_semantic(encode_canonical(&envelope), &mut semantic_failure)?;
        let nonce = lower_hex(&nonce);
        let key_temp = format!("dispatch-policy-key--{nonce}.tmp");
        transaction.stage_key_temp(&key_temp, &envelope_bytes)?;
        transaction.publish_staged_key_no_replace(&key_temp, &format!("{key_id}.key"))?;

        let registry = DispatchPolicyCommitmentRegistryV1 {
            schema_version: 1,
            authority_store_id,
            request_commitment_key: header,
            reservations_by_id: BTreeMap::new(),
            request_subject_index: BTreeMap::new(),
            commitments_by_id: BTreeMap::new(),
            retained_worker_caps_by_participant_id: BTreeMap::new(),
        };
        let bytes = retain_semantic(encode_canonical(&registry), &mut semantic_failure)?;
        transaction.publish_registry_no_replace(
            &format!("dispatch-policy-registry--{nonce}.tmp"),
            &bytes,
        )?;
        let readback = transaction
            .read_registry()?
            .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
        if readback != bytes {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "dispatch policy commitment registry readback mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        Ok(())
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result.map_err(storage_error)
}

pub(crate) fn reserve_fresh_spawn(
    authority: &HostSessionAuthority,
    input: FreshSpawnReservationInputV1,
) -> Result<FreshSpawnReservationOutcomeV1, DispatchPolicyCommitmentError> {
    initialize_dispatch_policy_commitment_registry(authority)?;
    validate_fresh_spawn_input(&input)?;
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    let mut semantic_failure = None;
    #[cfg(test)]
    let mut joined = false;
    let result = storage.transaction(|transaction| {
        let registry_bytes = transaction
            .read_registry()?
            .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
        let mut registry: DispatchPolicyCommitmentRegistryV1 =
            retain_semantic(decode_canonical(&registry_bytes), &mut semantic_failure)?;
        let key = retain_semantic(
            validate_registry_and_key(
                &registry,
                storage.authority_store_id(),
                &transaction.read_keys()?,
            ),
            &mut semantic_failure,
        )?;
        let request_bytes = retain_semantic(
            canonical_json::to_vec(&input.spawn_request)
                .map_err(|_| DispatchPolicyCommitmentError::new("canonicalize Spawn request")),
            &mut semantic_failure,
        )?;
        let request_commitment = FreshSpawnValidatedRequestCommitmentV1 {
            schema_version: 1,
            algorithm: FreshSpawnRequestCommitmentAlgorithmV1::HmacSha256,
            key_id: registry.request_commitment_key.key_id.clone(),
            domain: REQUEST_COMMITMENT_DOMAIN.into(),
            digest_hex: hmac_fresh_spawn_request(&key.secret_key, &request_bytes),
        };
        let lookup_key = fresh_spawn_lookup_key(storage.authority_store_id(), &input);
        let lookup_digest = retain_semantic(lookup_index_key(&lookup_key), &mut semantic_failure)?;
        if let Some(index) = registry.request_subject_index.get(&lookup_digest) {
            if index.lookup_key != lookup_key {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "fresh-spawn lookup index collision",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            }
            let reservation_ref = match &index.entry {
                DispatchPolicyCommitmentIndexEntryV1::FreshSpawnReserved { reservation_ref }
                | DispatchPolicyCommitmentIndexEntryV1::Committed {
                    reservation_ref: Some(reservation_ref),
                    ..
                } => reservation_ref,
                _ => {
                    semantic_failure = Some(DispatchPolicyCommitmentError::new(
                        "fresh-spawn index state is invalid",
                    ));
                    return Err(BootstrapError::dispatch_policy_commitment_semantic());
                }
            };
            let reservation = registry
                .reservations_by_id
                .get(&reservation_ref.reservation_id)
                .ok_or_else(|| {
                    semantic_failure = Some(DispatchPolicyCommitmentError::new(
                        "fresh-spawn reservation index is torn",
                    ));
                    BootstrapError::dispatch_policy_commitment_semantic()
                })?;
            retain_semantic(
                validate_reservation(reservation, reservation_ref),
                &mut semantic_failure,
            )?;
            retain_semantic(
                ensure_reservation_matches_input(reservation, &input, &request_commitment),
                &mut semantic_failure,
            )?;
            #[cfg(test)]
            {
                joined = true;
            }
            return Ok(reservation.clone());
        }

        let mut participant_entropy = [0_u8; 16];
        let mut bootstrap_entropy = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut participant_entropy);
        rand::rngs::OsRng.fill_bytes(&mut bootstrap_entropy);
        let commitment_id = format!("dpc_{}", Uuid::now_v7());
        let reservation_id = format!("dpr_{}", Uuid::now_v7());
        let retained_participant_id = format!("rwp_{}", lower_hex(&participant_entropy));
        let bootstrap_run_id = format!("rwr_{}", lower_hex(&bootstrap_entropy));
        let subject = DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
            retained_participant_id: retained_participant_id.clone(),
            bootstrap_run_id: bootstrap_run_id.clone(),
        };
        let mut reservation = DispatchPolicyCommitmentReservationV1 {
            schema_version: 1,
            authority_store_id: storage.authority_store_id().into(),
            reservation_id: reservation_id.clone(),
            lookup_key: lookup_key.clone(),
            subject,
            validated_request_commitment: request_commitment,
            request_id: input.spawn_request.request_id.clone(),
            idempotency_key: input.spawn_request.idempotency_key.clone(),
            orchestration_session_id: input.spawn_request.orchestration_session_id.clone(),
            caller_participant_id: input.spawn_request.caller_participant_id.clone(),
            caller_backend_id: input.caller_backend_id.clone(),
            target_backend_id: input.spawn_request.target_backend_id.clone(),
            world_id: input.spawn_request.world_id.clone(),
            world_generation: input.spawn_request.world_generation,
            retained_participant_id,
            bootstrap_run_id,
            parent_policy_ref: input.parent_policy_ref.clone(),
            parent_policy_revision: input.parent_policy_revision.clone(),
            applied_patch: input.applied_patch.clone(),
            policy_snapshot_bytes: ImmutableBytesMaterialV1::inline(input.policy_snapshot.bytes()),
            policy_snapshot_ref: input.policy_snapshot.snapshot_ref().clone(),
            policy_snapshot_hash: input.policy_snapshot.hash().into(),
            policy_snapshot_revision: input.policy_snapshot.revision().into(),
            reason: input.reason.clone(),
            proposed_commitment_id: commitment_id.clone(),
            proposed_worker_cap: ProposedRetainedWorkerCapV1 {
                commitment_id,
                policy_snapshot_ref: input.policy_snapshot.snapshot_ref().clone(),
                policy_snapshot_hash: input.policy_snapshot.hash().into(),
                policy_snapshot_revision: input.policy_snapshot.revision().into(),
                retained_worker_cap_link: RetainedWorkerCapLinkV1::ThisCommitment,
            },
            created_revision: 1,
            created_at: retain_semantic(current_timestamp(), &mut semantic_failure)?,
            reservation_hash: String::new(),
        };
        reservation.reservation_hash =
            retain_semantic(reservation_hash(&reservation), &mut semantic_failure)?;
        let reservation_ref = reservation_ref(&reservation);
        registry
            .reservations_by_id
            .insert(reservation_id, reservation.clone());
        registry.request_subject_index.insert(
            lookup_digest,
            DispatchPolicyCommitmentIndexRecordV1 {
                lookup_key,
                entry: DispatchPolicyCommitmentIndexEntryV1::FreshSpawnReserved { reservation_ref },
            },
        );
        let bytes = retain_semantic(encode_canonical(&registry), &mut semantic_failure)?;
        let nonce = random_nonce();
        transaction.replace_registry(&format!("dispatch-policy-registry--{nonce}.tmp"), &bytes)?;
        if transaction.read_registry()?.as_deref() != Some(bytes.as_slice()) {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fresh-spawn reservation readback mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        Ok(reservation)
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    let reservation = result.map_err(storage_error)?;
    #[cfg(test)]
    let proof = proof_from_reservation(&reservation, false);
    Ok(FreshSpawnReservationOutcomeV1 {
        reservation_ref: reservation_ref(&reservation),
        #[cfg(test)]
        retained_participant_id: reservation.retained_participant_id.clone(),
        #[cfg(test)]
        bootstrap_run_id: reservation.bootstrap_run_id.clone(),
        #[cfg(test)]
        proposed_commitment_id: reservation.proposed_commitment_id.clone(),
        #[cfg(test)]
        proof,
        #[cfg(test)]
        joined,
    })
}

pub(crate) fn authenticate_fresh_spawn_reservation(
    authority: &HostSessionAuthority,
    reference: &DispatchPolicyCommitmentReservationRefV1,
    request: &CanonicalValidatedSpawnRequestV1,
    admission_plan: &RetainedWorkerAdmissionPlanV1,
    e1_effective_snapshot: &ValidatedPolicySnapshotMaterialV1,
) -> Result<AuthenticatedFreshSpawnReservationProofV1, DispatchPolicyCommitmentError> {
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        let registry: DispatchPolicyCommitmentRegistryV1 = retain_semantic(
            decode_canonical(
                &transaction
                    .read_registry()?
                    .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?,
            ),
            &mut semantic_failure,
        )?;
        let key = retain_semantic(
            validate_registry_and_key(
                &registry,
                storage.authority_store_id(),
                &transaction.read_keys()?,
            ),
            &mut semantic_failure,
        )?;
        let reservation = registry
            .reservations_by_id
            .get(&reference.reservation_id)
            .ok_or_else(|| {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "fresh-spawn reservation does not exist",
                ));
                BootstrapError::dispatch_policy_commitment_semantic()
            })?;
        retain_semantic(
            validate_reservation(reservation, reference),
            &mut semantic_failure,
        )?;
        let request_bytes = retain_semantic(
            canonical_json::to_vec(request)
                .map_err(|_| DispatchPolicyCommitmentError::new("canonicalize Spawn request")),
            &mut semantic_failure,
        )?;
        let expected = hmac_fresh_spawn_request(&key.secret_key, &request_bytes);
        if reservation.validated_request_commitment.key_id != key.key_id
            || reservation.validated_request_commitment.domain != REQUEST_COMMITMENT_DOMAIN
            || !constant_time_eq(
                reservation
                    .validated_request_commitment
                    .digest_hex
                    .as_bytes(),
                expected.as_bytes(),
            )
        {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fresh-spawn validated request commitment mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        let nonempty_restricted = retain_semantic(
            validate_authenticated_fresh_spawn_e1_material(
                reservation,
                request,
                admission_plan,
                e1_effective_snapshot,
            ),
            &mut semantic_failure,
        )?;
        let proof = proof_from_reservation(
            reservation,
            nonempty_restricted
                && admission_plan
                    .policy_and_admission_cap
                    .allow_capability_narrowing,
        );
        retain_semantic(
            proof.validate_admission_plan_binding(admission_plan),
            &mut semantic_failure,
        )?;
        Ok(proof)
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result.map_err(storage_error)
}

fn validate_authenticated_fresh_spawn_e1_material(
    reservation: &DispatchPolicyCommitmentReservationV1,
    request: &CanonicalValidatedSpawnRequestV1,
    admission_plan: &RetainedWorkerAdmissionPlanV1,
    e1_effective_snapshot: &ValidatedPolicySnapshotMaterialV1,
) -> Result<bool, DispatchPolicyCommitmentError> {
    if reservation.policy_snapshot_bytes.resolve_inline()? != e1_effective_snapshot.bytes()
        || reservation.policy_snapshot_ref != *e1_effective_snapshot.snapshot_ref()
        || reservation.policy_snapshot_hash != e1_effective_snapshot.hash()
        || reservation.policy_snapshot_revision != e1_effective_snapshot.revision()
        || reservation.parent_policy_ref
            != admission_plan.policy_and_admission_cap.current_policy_ref
        || reservation.parent_policy_revision
            != admission_plan
                .policy_and_admission_cap
                .current_policy
                .policy_revision
    {
        return Err(DispatchPolicyCommitmentError::new(
            "Fresh Spawn reservation does not match the authenticated E1 effective snapshot",
        ));
    }
    match &reservation.applied_patch {
        AppliedDispatchPolicyPatchIdentityV1::UnchangedParent => Ok(false),
        AppliedDispatchPolicyPatchIdentityV1::RestrictedWorldFs {
            canonical_patch, ..
        } => {
            let bytes = canonical_patch.resolve_inline()?;
            let patch: DispatchPolicyNarrowingPatchV1 = decode_canonical(&bytes)?;
            let expected_parent = transport_policy_ref(&reservation.parent_policy_ref);
            if patch.request_id != request.request_id
                || patch.orchestration_session_id != request.orchestration_session_id
                || patch.caller_participant_id != request.caller_participant_id
                || patch.target_backend_id != request.target_backend_id
                || patch.target_world.world_id != request.world_id
                || patch.target_world.world_generation != request.world_generation
                || patch.parent_policy_ref != expected_parent
                || patch.parent_policy_revision != reservation.parent_policy_revision
                || !matches!(
                    patch.applies_to,
                    transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerSpawn
                )
                || patch
                    .restricted_policy_patch
                    .world_fs
                    .as_ref()
                    .is_none_or(|world_fs| world_fs.is_empty())
            {
                return Err(DispatchPolicyCommitmentError::new(
                    "Fresh Spawn E1 patch binding or effective snapshot is not authenticated",
                ));
            }
            Ok(true)
        }
    }
}

pub(crate) fn publish_fresh_spawn_commitment(
    authority: &HostSessionAuthority,
    proof: &AuthenticatedFreshSpawnReservationProofV1,
    admission: &RetainedWorkerAdmissionRecordV1,
) -> Result<PersistedDispatchPolicyCommitmentV1, DispatchPolicyCommitmentError> {
    let stable_admission = stable_admission_link_from_record(admission)?;
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    if proof.reservation_ref.authority_store_id != storage.authority_store_id() {
        return Err(DispatchPolicyCommitmentError::new(
            "fresh-spawn publication authority store mismatch",
        ));
    }
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        let mut registry: DispatchPolicyCommitmentRegistryV1 = retain_semantic(
            decode_canonical(
                &transaction
                    .read_registry()?
                    .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?,
            ),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_registry_and_key(
                &registry,
                storage.authority_store_id(),
                &transaction.read_keys()?,
            )
            .map(|_| ()),
            &mut semantic_failure,
        )?;
        let reservation = registry
            .reservations_by_id
            .get(&proof.reservation_ref.reservation_id)
            .cloned()
            .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
        retain_semantic(
            validate_reservation(&reservation, &proof.reservation_ref),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_proof_matches_reservation(proof, &reservation),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_stable_admission_matches_reservation(&stable_admission, &reservation),
            &mut semantic_failure,
        )?;
        let lookup_digest = retain_semantic(
            lookup_index_key(&reservation.lookup_key),
            &mut semantic_failure,
        )?;
        let index = registry
            .request_subject_index
            .get(&lookup_digest)
            .cloned()
            .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
        if index.lookup_key != reservation.lookup_key {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fresh-spawn publication lookup index mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        if let DispatchPolicyCommitmentIndexEntryV1::Committed {
            reservation_ref: Some(existing_reservation_ref),
            fresh_spawn_validated_request_commitment: Some(existing_request_commitment),
            commitment_ref: existing_commitment_ref,
        } = &index.entry
        {
            if existing_reservation_ref != &proof.reservation_ref
                || existing_request_commitment != &reservation.validated_request_commitment
            {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "fresh-spawn committed retry changed reservation material",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            }
            let existing = registry
                .commitments_by_id
                .get(&existing_commitment_ref.commitment_id)
                .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
            retain_semantic(
                validate_commitment(existing, Some(existing_commitment_ref)),
                &mut semantic_failure,
            )?;
            retain_semantic(
                validate_fresh_spawn_record_matches(existing, &reservation, &stable_admission),
                &mut semantic_failure,
            )?;
            return Ok(existing.clone());
        }
        if index.entry
            != (DispatchPolicyCommitmentIndexEntryV1::FreshSpawnReserved {
                reservation_ref: proof.reservation_ref.clone(),
            })
        {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fresh-spawn publication index is not the exact reservation",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }

        let mut record = DispatchPolicyCommitmentV1 {
            schema_version: 1,
            authority_store_id: reservation.authority_store_id.clone(),
            commitment_id: reservation.proposed_commitment_id.clone(),
            subject: reservation.subject.clone(),
            request_id: reservation.request_id.clone(),
            idempotency_key: reservation.idempotency_key.clone(),
            orchestration_session_id: reservation.orchestration_session_id.clone(),
            caller_participant_id: reservation.caller_participant_id.clone(),
            caller_backend_id: reservation.caller_backend_id.clone(),
            target_backend_id: reservation.target_backend_id.clone(),
            world_id: reservation.world_id.clone(),
            world_generation: reservation.world_generation,
            authority_link: PolicyCommitmentAuthorityLinkV1::RetainedAdmission {
                stable_admission_identity: stable_admission.clone(),
            },
            execution_claim_link: None,
            fresh_spawn_reservation_ref: Some(proof.reservation_ref.clone()),
            fresh_spawn_validated_request_commitment: Some(
                reservation.validated_request_commitment.clone(),
            ),
            parent_policy_ref: reservation.parent_policy_ref.clone(),
            parent_policy_revision: reservation.parent_policy_revision.clone(),
            applied_patch: reservation.applied_patch.clone(),
            policy_snapshot_bytes: reservation.policy_snapshot_bytes.clone(),
            policy_snapshot_ref: reservation.policy_snapshot_ref.clone(),
            policy_snapshot_hash: reservation.policy_snapshot_hash.clone(),
            policy_snapshot_revision: reservation.policy_snapshot_revision.clone(),
            reason: reservation.reason.clone(),
            retained_worker_cap_link: Some(RetainedWorkerCapLinkV1::ThisCommitment),
            source_worker_cap_ref: None,
            created_revision: 1,
            application_revision: 1,
            status: PolicyCommitmentStatusV1::Immutable,
            created_at: retain_semantic(current_timestamp(), &mut semantic_failure)?,
            exact_linkage_hash: String::new(),
        };
        record.exact_linkage_hash =
            retain_semantic(commitment_hash(&record), &mut semantic_failure)?;
        retain_semantic(validate_commitment(&record, None), &mut semantic_failure)?;
        let reference = commitment_ref(&record);
        if registry
            .commitments_by_id
            .insert(record.commitment_id.clone(), record.clone())
            .is_some()
            || registry
                .retained_worker_caps_by_participant_id
                .insert(
                    reservation.retained_participant_id.clone(),
                    reference.clone(),
                )
                .is_some()
        {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fresh-spawn commitment or worker-cap identity was already occupied",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        registry.request_subject_index.insert(
            lookup_digest,
            DispatchPolicyCommitmentIndexRecordV1 {
                lookup_key: reservation.lookup_key.clone(),
                entry: DispatchPolicyCommitmentIndexEntryV1::Committed {
                    reservation_ref: Some(proof.reservation_ref.clone()),
                    fresh_spawn_validated_request_commitment: Some(
                        reservation.validated_request_commitment.clone(),
                    ),
                    commitment_ref: reference,
                },
            },
        );
        let bytes = retain_semantic(encode_canonical(&registry), &mut semantic_failure)?;
        transaction.replace_registry(
            &format!("dispatch-policy-registry--{}.tmp", random_nonce()),
            &bytes,
        )?;
        if transaction.read_registry()?.as_deref() != Some(bytes.as_slice()) {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fresh-spawn commitment readback mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        Ok(record)
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result
        .map(PersistedDispatchPolicyCommitmentV1)
        .map_err(storage_error)
}

pub(crate) fn publish_accepted_work_commitment(
    authority: &HostSessionAuthority,
    input: AcceptedWorkPolicyCommitmentInputV1,
) -> Result<PersistedDispatchPolicyCommitmentV1, DispatchPolicyCommitmentError> {
    input
        .acceptance
        .validate()
        .map_err(|_| DispatchPolicyCommitmentError::new("invalid durable B1 acceptance"))?;
    validate_component("idempotency_key", &input.idempotency_key)?;
    validate_applied_patch(&input.applied_patch)?;
    validate_claim_matches_acceptance(&input.execution_claim, &input.acceptance)?;
    if input.acceptance.current_policy_snapshot_ref != *input.policy_snapshot.snapshot_ref()
        || input.acceptance.current_policy_snapshot_hash != input.policy_snapshot.hash()
        || input.acceptance.current_policy_revision != input.policy_snapshot.revision()
    {
        return Err(DispatchPolicyCommitmentError::new(
            "B1 acceptance does not identify the exact effective E1 snapshot",
        ));
    }
    let (subject, cap_link) = match &input.acceptance.work_identity {
        AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } => {
            if input.retained_worker_cap_ref.is_some() {
                return Err(DispatchPolicyCommitmentError::new(
                    "ephemeral work cannot carry a retained-worker cap",
                ));
            }
            (
                DispatchPolicyCommitmentSubjectV1::EphemeralWork {
                    task_run_id: task_run_id.clone(),
                },
                None,
            )
        }
        AcceptedWorldWorkIdentityV1::RetainedTurn {
            active_run_id,
            message_id,
            target_participant_id,
        } => {
            let cap_ref = input.retained_worker_cap_ref.clone().ok_or_else(|| {
                DispatchPolicyCommitmentError::new(
                    "retained turn requires the exact immutable worker cap",
                )
            })?;
            (
                DispatchPolicyCommitmentSubjectV1::RetainedWorkerTurn {
                    retained_participant_id: target_participant_id.clone(),
                    active_run_id: active_run_id.clone(),
                    message_id: message_id.clone(),
                },
                Some(RetainedWorkerCapLinkV1::Existing { cap_ref }),
            )
        }
    };
    let claim_link = execution_claim_link(&input.execution_claim)?;
    let mut record = DispatchPolicyCommitmentV1 {
        schema_version: 1,
        authority_store_id: input.acceptance.authority_store_id.clone(),
        commitment_id: format!("dpc_{}", Uuid::now_v7()),
        subject,
        request_id: input.acceptance.request_id.clone(),
        idempotency_key: input.idempotency_key,
        orchestration_session_id: input.acceptance.orchestration_session_id.clone(),
        caller_participant_id: input.acceptance.caller_participant_id.clone(),
        caller_backend_id: input.acceptance.caller_backend_id.clone(),
        target_backend_id: input.acceptance.target_backend_id.clone(),
        world_id: input.acceptance.world_id.clone(),
        world_generation: input.acceptance.world_generation,
        authority_link: PolicyCommitmentAuthorityLinkV1::B1 {
            acceptance_record_id: input.acceptance.acceptance_record_id.clone(),
            acceptance_record_revision: input.acceptance.record_revision,
            runtime_acceptance: input.acceptance.runtime_acceptance.clone(),
        },
        execution_claim_link: Some(claim_link),
        fresh_spawn_reservation_ref: None,
        fresh_spawn_validated_request_commitment: None,
        parent_policy_ref: input.acceptance.current_policy_snapshot_ref,
        parent_policy_revision: input.acceptance.current_policy_revision,
        applied_patch: input.applied_patch,
        policy_snapshot_bytes: ImmutableBytesMaterialV1::inline(input.policy_snapshot.bytes()),
        policy_snapshot_ref: input.policy_snapshot.snapshot_ref().clone(),
        policy_snapshot_hash: input.policy_snapshot.hash().into(),
        policy_snapshot_revision: input.policy_snapshot.revision().into(),
        reason: input.reason,
        retained_worker_cap_link: cap_link,
        source_worker_cap_ref: None,
        created_revision: 1,
        application_revision: 1,
        status: PolicyCommitmentStatusV1::Immutable,
        created_at: current_timestamp()?,
        exact_linkage_hash: String::new(),
    };
    record.exact_linkage_hash = commitment_hash(&record)?;
    publish_complete_non_spawn_commitment(authority, record)
}

pub(crate) fn publish_fork_commitment(
    authority: &HostSessionAuthority,
    input: ForkPolicyCommitmentInputV1,
) -> Result<PersistedDispatchPolicyCommitmentV1, DispatchPolicyCommitmentError> {
    for (name, value) in [
        ("request_id", input.request_id.as_str()),
        ("idempotency_key", input.idempotency_key.as_str()),
        (
            "orchestration_session_id",
            input.orchestration_session_id.as_str(),
        ),
        (
            "caller_participant_id",
            input.caller_participant_id.as_str(),
        ),
        ("caller_backend_id", input.caller_backend_id.as_str()),
        ("target_backend_id", input.target_backend_id.as_str()),
        ("world_id", input.world_id.as_str()),
        (
            "source_participant_id",
            input.source_participant_id.as_str(),
        ),
        (
            "parent_policy_revision",
            input.parent_policy_revision.as_str(),
        ),
    ] {
        validate_component(name, value)?;
    }
    if input.world_generation == 0 {
        return Err(DispatchPolicyCommitmentError::new(
            "fork world generation or authority binding is invalid",
        ));
    }
    validate_applied_patch(&input.applied_patch)?;
    let canonical_request: Value = decode_canonical(&input.canonical_validated_dispatch_request)?;
    let fork_request_hash = canonical_domain_hash(
        FORK_REQUEST_HASH_DOMAIN,
        "validated_dispatch_request",
        canonical_request,
    )?;
    initialize_dispatch_policy_commitment_registry(authority)?;
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    if input
        .authenticated_policy
        .source_worker_cap_ref()
        .authority_store_id
        != storage.authority_store_id()
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fork source cap belongs to another authority store",
        ));
    }
    let lookup_key = DispatchPolicyCommitmentLookupKeyV1 {
        authority_store_id: storage.authority_store_id().into(),
        orchestration_session_id: input.orchestration_session_id.clone(),
        request_id: input.request_id.clone(),
        subject: DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerFork {
            source_participant_id: input.source_participant_id.clone(),
        },
    };
    let lookup_digest = lookup_index_key(&lookup_key)?;
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        let mut registry: DispatchPolicyCommitmentRegistryV1 = retain_semantic(
            decode_canonical(
                &transaction
                    .read_registry()?
                    .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?,
            ),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_registry_and_key(
                &registry,
                storage.authority_store_id(),
                &transaction.read_keys()?,
            )
            .map(|_| ()),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_cap_for_participant(
                &registry,
                &input.source_participant_id,
                input.authenticated_policy.source_worker_cap_ref(),
                &input.orchestration_session_id,
            ),
            &mut semantic_failure,
        )?;
        if let Some(index) = registry.request_subject_index.get(&lookup_digest) {
            if index.lookup_key != lookup_key {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "fork lookup index collision",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            }
            let DispatchPolicyCommitmentIndexEntryV1::Committed {
                reservation_ref: None,
                fresh_spawn_validated_request_commitment: None,
                commitment_ref: existing_ref,
            } = &index.entry
            else {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "fork lookup index has incompatible state",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            };
            let existing = registry
                .commitments_by_id
                .get(&existing_ref.commitment_id)
                .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
            retain_semantic(
                validate_commitment(existing, Some(existing_ref)),
                &mut semantic_failure,
            )?;
            retain_semantic(
                validate_fork_retry(existing, &input, &fork_request_hash),
                &mut semantic_failure,
            )?;
            let child =
                retain_semantic(fork_child_participant_id(existing), &mut semantic_failure)?;
            if registry.retained_worker_caps_by_participant_id.get(child) != Some(existing_ref) {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "fork retry child-cap index is torn",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            }
            return Ok(existing.clone());
        }

        // Fork is deliberately outside B3.2a admission. Keep its stable child
        // and bootstrap identities in the existing non-admission namespace so
        // WorldService does not mistake the E2-owned fork link for a Fresh
        // Spawn launch-authority proof.
        let child_participant_id = format!("ash_{}", Uuid::now_v7());
        let bootstrap_run_id = Uuid::now_v7().to_string();
        let mut record = fork_record_from_input(
            storage.authority_store_id(),
            &input,
            &fork_request_hash,
            child_participant_id.clone(),
            bootstrap_run_id,
            retain_semantic(current_timestamp(), &mut semantic_failure)?,
        );
        record.exact_linkage_hash =
            retain_semantic(commitment_hash(&record), &mut semantic_failure)?;
        retain_semantic(
            validate_non_spawn_subject_rules(&record),
            &mut semantic_failure,
        )?;
        let reference = commitment_ref(&record);
        if registry
            .commitments_by_id
            .contains_key(&record.commitment_id)
            || registry
                .retained_worker_caps_by_participant_id
                .contains_key(&child_participant_id)
        {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fork generated commitment or child identity collision",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        registry
            .commitments_by_id
            .insert(record.commitment_id.clone(), record.clone());
        registry
            .retained_worker_caps_by_participant_id
            .insert(child_participant_id, reference.clone());
        registry.request_subject_index.insert(
            lookup_digest.clone(),
            DispatchPolicyCommitmentIndexRecordV1 {
                lookup_key: lookup_key.clone(),
                entry: DispatchPolicyCommitmentIndexEntryV1::Committed {
                    reservation_ref: None,
                    fresh_spawn_validated_request_commitment: None,
                    commitment_ref: reference,
                },
            },
        );
        let bytes = retain_semantic(encode_canonical(&registry), &mut semantic_failure)?;
        transaction.replace_registry(
            &format!("dispatch-policy-registry--{}.tmp", random_nonce()),
            &bytes,
        )?;
        if transaction.read_registry()?.as_deref() != Some(bytes.as_slice()) {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "fork commitment readback mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        Ok(record)
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result
        .map(PersistedDispatchPolicyCommitmentV1)
        .map_err(storage_error)
}

fn publish_complete_non_spawn_commitment(
    authority: &HostSessionAuthority,
    record: DispatchPolicyCommitmentV1,
) -> Result<PersistedDispatchPolicyCommitmentV1, DispatchPolicyCommitmentError> {
    validate_non_spawn_subject_rules(&record)?;
    initialize_dispatch_policy_commitment_registry(authority)?;
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    if record.authority_store_id != storage.authority_store_id() {
        return Err(DispatchPolicyCommitmentError::new(
            "non-Spawn commitment authority store mismatch",
        ));
    }
    let lookup_key = commitment_lookup_key(&record);
    let lookup_digest = lookup_index_key(&lookup_key)?;
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        let mut registry: DispatchPolicyCommitmentRegistryV1 = retain_semantic(
            decode_canonical(
                &transaction
                    .read_registry()?
                    .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?,
            ),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_registry_and_key(
                &registry,
                storage.authority_store_id(),
                &transaction.read_keys()?,
            )
            .map(|_| ()),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_record_cap_sources(&registry, &record),
            &mut semantic_failure,
        )?;
        if let Some(existing_index) = registry.request_subject_index.get(&lookup_digest) {
            if existing_index.lookup_key != lookup_key {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "non-Spawn lookup index collision",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            }
            let DispatchPolicyCommitmentIndexEntryV1::Committed {
                reservation_ref: None,
                fresh_spawn_validated_request_commitment: None,
                commitment_ref: existing_ref,
            } = &existing_index.entry
            else {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "non-Spawn lookup index has incompatible state",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            };
            let existing = registry
                .commitments_by_id
                .get(&existing_ref.commitment_id)
                .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
            retain_semantic(
                validate_commitment(existing, Some(existing_ref)),
                &mut semantic_failure,
            )?;
            retain_semantic(
                validate_exact_non_spawn_retry(existing, &record),
                &mut semantic_failure,
            )?;
            return Ok(existing.clone());
        }
        if registry
            .commitments_by_id
            .contains_key(&record.commitment_id)
        {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "non-Spawn commitment identity is already occupied",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        let reference = commitment_ref(&record);
        registry
            .commitments_by_id
            .insert(record.commitment_id.clone(), record.clone());
        registry.request_subject_index.insert(
            lookup_digest,
            DispatchPolicyCommitmentIndexRecordV1 {
                lookup_key,
                entry: DispatchPolicyCommitmentIndexEntryV1::Committed {
                    reservation_ref: None,
                    fresh_spawn_validated_request_commitment: None,
                    commitment_ref: reference.clone(),
                },
            },
        );
        if let DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
            child_participant_id,
            ..
        } = &record.subject
        {
            if registry
                .retained_worker_caps_by_participant_id
                .insert(child_participant_id.clone(), reference)
                .is_some()
            {
                semantic_failure = Some(DispatchPolicyCommitmentError::new(
                    "fork child already owns an immutable worker cap",
                ));
                return Err(BootstrapError::dispatch_policy_commitment_semantic());
            }
        }
        let bytes = retain_semantic(encode_canonical(&registry), &mut semantic_failure)?;
        transaction.replace_registry(
            &format!("dispatch-policy-registry--{}.tmp", random_nonce()),
            &bytes,
        )?;
        if transaction.read_registry()?.as_deref() != Some(bytes.as_slice()) {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "non-Spawn commitment readback mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        Ok(record)
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result
        .map(PersistedDispatchPolicyCommitmentV1)
        .map_err(storage_error)
}

fn validate_claim_matches_acceptance(
    claim: &WorldWorkExecutionClaimV1,
    acceptance: &WorldWorkAcceptanceRecordV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    if claim.schema_version != 1
        || claim.authority_store_id != acceptance.authority_store_id
        || claim.authority_revision_observed != acceptance.authority_revision_observed
        || claim.acceptance_record_id != acceptance.acceptance_record_id
        || claim.acceptance_record_revision != acceptance.record_revision
        || claim.orchestration_session_id != acceptance.orchestration_session_id
        || claim.caller_participant_id != acceptance.caller_participant_id
        || claim.caller_backend_id != acceptance.caller_backend_id
        || claim.target_backend_id != acceptance.target_backend_id
        || claim.work_identity != acceptance.work_identity
        || claim.world_id != acceptance.world_id
        || claim.world_generation != acceptance.world_generation
        || claim.host_transition_correlation != acceptance.host_transition_correlation
        || claim.stream_id != acceptance.runtime_acceptance.stream_id
        || claim.acceptance_frame_sequence != acceptance.runtime_acceptance.frame_sequence
        || Some(claim.runtime_submission_id.as_str())
            != acceptance
                .runtime_acceptance
                .runtime_submission_id
                .as_deref()
        || claim.observer_instance_id.is_empty()
        || claim.observer_epoch == 0
        || claim.claim_revision == 0
    {
        return Err(DispatchPolicyCommitmentError::new(
            "B2.1 claim does not exact-link the durable B1 acceptance",
        ));
    }
    Ok(())
}

fn execution_claim_link(
    claim: &WorldWorkExecutionClaimV1,
) -> Result<WorldWorkExecutionClaimLinkV1, DispatchPolicyCommitmentError> {
    let claim_preimage = canonical_json::to_vec(claim)
        .map_err(|_| DispatchPolicyCommitmentError::new("canonicalize exact B2.1 claim"))?;
    let claim_value: Value = serde_json::from_slice(&claim_preimage)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode exact B2.1 claim preimage"))?;
    let claim_linkage_hash = canonical_domain_hash(CLAIM_LINK_HASH_DOMAIN, "claim", claim_value)?;
    Ok(WorldWorkExecutionClaimLinkV1 {
        authority_store_id: claim.authority_store_id.clone(),
        durable_claim_key: WorldWorkExecutionClaimDurableKeyV1 {
            supervisor_schema_version: 1,
            executions_by_acceptance_record_id_key: claim.acceptance_record_id.clone(),
        },
        acceptance_record_id: claim.acceptance_record_id.clone(),
        acceptance_record_revision: claim.acceptance_record_revision,
        claim_revision: claim.claim_revision,
        observer_instance_id: claim.observer_instance_id.clone(),
        observer_epoch: claim.observer_epoch,
        claim_preimage: ImmutableBytesMaterialV1::inline(&claim_preimage),
        claim_linkage_hash,
    })
}

fn validate_execution_claim_link(
    link: &WorldWorkExecutionClaimLinkV1,
    authority_store_id: &str,
) -> Result<WorldWorkExecutionClaimV1, DispatchPolicyCommitmentError> {
    let bytes = link.claim_preimage.resolve_inline()?;
    let claim: WorldWorkExecutionClaimV1 = serde_json::from_slice(&bytes)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode linked B2.1 claim"))?;
    let canonical = canonical_json::to_vec(&claim)
        .map_err(|_| DispatchPolicyCommitmentError::new("re-encode linked B2.1 claim"))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode linked B2.1 claim value"))?;
    if canonical != bytes
        || link.authority_store_id != authority_store_id
        || claim.authority_store_id != authority_store_id
        || link.durable_claim_key.supervisor_schema_version != 1
        || link
            .durable_claim_key
            .executions_by_acceptance_record_id_key
            != claim.acceptance_record_id
        || link.acceptance_record_id != claim.acceptance_record_id
        || link.acceptance_record_revision != claim.acceptance_record_revision
        || link.claim_revision != claim.claim_revision
        || link.observer_instance_id != claim.observer_instance_id
        || link.observer_epoch != claim.observer_epoch
        || link.claim_linkage_hash != canonical_domain_hash(CLAIM_LINK_HASH_DOMAIN, "claim", value)?
    {
        return Err(DispatchPolicyCommitmentError::new(
            "B2.1 claim preimage, durable key, or linkage hash mismatch",
        ));
    }
    Ok(claim)
}

fn validate_non_spawn_subject_rules(
    record: &DispatchPolicyCommitmentV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    if record.fresh_spawn_reservation_ref.is_some()
        || record.fresh_spawn_validated_request_commitment.is_some()
    {
        return Err(DispatchPolicyCommitmentError::new(
            "non-Spawn commitment cannot carry Fresh Spawn reservation material",
        ));
    }
    match (&record.subject, &record.authority_link) {
        (
            DispatchPolicyCommitmentSubjectV1::EphemeralWork { task_run_id },
            PolicyCommitmentAuthorityLinkV1::B1 {
                acceptance_record_id,
                acceptance_record_revision,
                runtime_acceptance,
            },
        ) => {
            let claim = validate_execution_claim_link(
                record.execution_claim_link.as_ref().ok_or_else(|| {
                    DispatchPolicyCommitmentError::new(
                        "ephemeral work requires exact B2.1 claim linkage",
                    )
                })?,
                &record.authority_store_id,
            )?;
            if record.retained_worker_cap_link.is_some()
                || record.source_worker_cap_ref.is_some()
                || claim.authority_store_id != record.authority_store_id
                || claim.orchestration_session_id != record.orchestration_session_id
                || claim.caller_participant_id != record.caller_participant_id
                || claim.caller_backend_id != record.caller_backend_id
                || claim.target_backend_id != record.target_backend_id
                || claim.world_id != record.world_id
                || claim.world_generation != record.world_generation
                || acceptance_record_id != &claim.acceptance_record_id
                || acceptance_record_revision != &claim.acceptance_record_revision
                || runtime_acceptance.acceptance_record_id != claim.acceptance_record_id
                || runtime_acceptance.task_run_id.as_deref() != Some(task_run_id)
                || runtime_acceptance.active_run_id.is_some()
                || runtime_acceptance.message_id.is_some()
                || runtime_acceptance.retained_participant_id.is_some()
            {
                return Err(DispatchPolicyCommitmentError::new(
                    "ephemeral B1/B2.1 subject linkage mismatch",
                ));
            }
        }
        (
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerTurn {
                retained_participant_id,
                active_run_id,
                message_id,
            },
            PolicyCommitmentAuthorityLinkV1::B1 {
                acceptance_record_id,
                acceptance_record_revision,
                runtime_acceptance,
            },
        ) => {
            let claim = validate_execution_claim_link(
                record.execution_claim_link.as_ref().ok_or_else(|| {
                    DispatchPolicyCommitmentError::new(
                        "retained turn requires exact B2.1 claim linkage",
                    )
                })?,
                &record.authority_store_id,
            )?;
            if !matches!(
                record.retained_worker_cap_link,
                Some(RetainedWorkerCapLinkV1::Existing { .. })
            ) || record.source_worker_cap_ref.is_some()
                || claim.authority_store_id != record.authority_store_id
                || claim.orchestration_session_id != record.orchestration_session_id
                || claim.caller_participant_id != record.caller_participant_id
                || claim.caller_backend_id != record.caller_backend_id
                || claim.target_backend_id != record.target_backend_id
                || claim.world_id != record.world_id
                || claim.world_generation != record.world_generation
                || acceptance_record_id != &claim.acceptance_record_id
                || acceptance_record_revision != &claim.acceptance_record_revision
                || runtime_acceptance.acceptance_record_id != claim.acceptance_record_id
                || runtime_acceptance.active_run_id.as_deref() != Some(active_run_id)
                || runtime_acceptance.message_id.as_deref() != Some(message_id)
                || runtime_acceptance.retained_participant_id.as_deref()
                    != Some(retained_participant_id)
                || runtime_acceptance.task_run_id.is_some()
            {
                return Err(DispatchPolicyCommitmentError::new(
                    "retained-turn B1/B2.1/cap subject linkage mismatch",
                ));
            }
        }
        (
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork { .. },
            PolicyCommitmentAuthorityLinkV1::ForkDispatch {
                canonical_validated_dispatch_request_sha256,
            },
        ) => {
            validate_digest(
                "fork validated dispatch request hash",
                canonical_validated_dispatch_request_sha256,
            )?;
            if record.execution_claim_link.is_some()
                || record.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment)
                || record.source_worker_cap_ref.is_none()
            {
                return Err(DispatchPolicyCommitmentError::new(
                    "fork must carry only E2 fork authority and source/child cap links",
                ));
            }
        }
        (DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch { .. }, _) => {
            return Err(DispatchPolicyCommitmentError::new(
                "Fresh Spawn launch requires reservation/B3.2a publication",
            ));
        }
        _ => {
            return Err(DispatchPolicyCommitmentError::new(
                "dispatch policy subject uses the wrong authority owner",
            ));
        }
    }
    validate_commitment(record, None)
}

fn commitment_lookup_key(
    record: &DispatchPolicyCommitmentV1,
) -> DispatchPolicyCommitmentLookupKeyV1 {
    let subject = match &record.subject {
        DispatchPolicyCommitmentSubjectV1::EphemeralWork { task_run_id } => {
            DispatchPolicyCommitmentSubjectKeyV1::EphemeralWork {
                task_run_id: task_run_id.clone(),
            }
        }
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch { .. } => {
            DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerLaunch
        }
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerTurn {
            active_run_id,
            message_id,
            ..
        } => DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerTurn {
            active_run_id: active_run_id.clone(),
            message_id: message_id.clone(),
        },
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
            source_participant_id,
            ..
        } => DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerFork {
            source_participant_id: source_participant_id.clone(),
        },
    };
    DispatchPolicyCommitmentLookupKeyV1 {
        authority_store_id: record.authority_store_id.clone(),
        orchestration_session_id: record.orchestration_session_id.clone(),
        request_id: record.request_id.clone(),
        subject,
    }
}

fn validate_record_cap_sources(
    registry: &DispatchPolicyCommitmentRegistryV1,
    record: &DispatchPolicyCommitmentV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    let (participant_id, source_ref) = match (&record.subject, &record.retained_worker_cap_link) {
        (
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerTurn {
                retained_participant_id,
                ..
            },
            Some(RetainedWorkerCapLinkV1::Existing { cap_ref }),
        ) => (retained_participant_id, cap_ref),
        (
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
                source_participant_id,
                ..
            },
            Some(RetainedWorkerCapLinkV1::ThisCommitment),
        ) => (
            source_participant_id,
            record.source_worker_cap_ref.as_ref().ok_or_else(|| {
                DispatchPolicyCommitmentError::new("fork source cap ref is absent")
            })?,
        ),
        (DispatchPolicyCommitmentSubjectV1::EphemeralWork { .. }, None) => return Ok(()),
        _ => {
            return Err(DispatchPolicyCommitmentError::new(
                "invalid non-Spawn cap source shape",
            ));
        }
    };
    if registry
        .retained_worker_caps_by_participant_id
        .get(participant_id)
        != Some(source_ref)
    {
        return Err(DispatchPolicyCommitmentError::new(
            "immutable worker cap does not resolve for the source participant",
        ));
    }
    let cap = registry
        .commitments_by_id
        .get(&source_ref.commitment_id)
        .ok_or_else(|| {
            DispatchPolicyCommitmentError::new("immutable worker cap record is absent")
        })?;
    validate_commitment(cap, Some(source_ref))?;
    let cap_participant_id = match &cap.subject {
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
            retained_participant_id,
            ..
        } => retained_participant_id,
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
            child_participant_id,
            ..
        } => child_participant_id,
        _ => {
            return Err(DispatchPolicyCommitmentError::new(
                "worker cap must originate from launch or fork",
            ));
        }
    };
    if cap_participant_id != participant_id
        || cap.orchestration_session_id != record.orchestration_session_id
        || cap.authority_store_id != record.authority_store_id
        || cap.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment)
    {
        return Err(DispatchPolicyCommitmentError::new(
            "source worker cap subject/session/linkage mismatch",
        ));
    }
    Ok(())
}

fn validate_cap_for_participant(
    registry: &DispatchPolicyCommitmentRegistryV1,
    participant_id: &str,
    source_ref: &DispatchPolicyCommitmentRefV1,
    orchestration_session_id: &str,
) -> Result<(), DispatchPolicyCommitmentError> {
    if registry
        .retained_worker_caps_by_participant_id
        .get(participant_id)
        != Some(source_ref)
    {
        return Err(DispatchPolicyCommitmentError::new(
            "immutable worker cap does not resolve for the source participant",
        ));
    }
    let cap = registry
        .commitments_by_id
        .get(&source_ref.commitment_id)
        .ok_or_else(|| {
            DispatchPolicyCommitmentError::new("immutable source worker cap record is absent")
        })?;
    validate_commitment(cap, Some(source_ref))?;
    let cap_participant_id = match &cap.subject {
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
            retained_participant_id,
            ..
        } => retained_participant_id,
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
            child_participant_id,
            ..
        } => child_participant_id,
        _ => {
            return Err(DispatchPolicyCommitmentError::new(
                "immutable source worker cap has a non-cap subject",
            ));
        }
    };
    if cap_participant_id != participant_id
        || cap.orchestration_session_id != orchestration_session_id
        || cap.authority_store_id != registry.authority_store_id
        || cap.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment)
    {
        return Err(DispatchPolicyCommitmentError::new(
            "source worker cap subject, session, store, or linkage mismatch",
        ));
    }
    Ok(())
}

fn fork_child_participant_id(
    record: &DispatchPolicyCommitmentV1,
) -> Result<&str, DispatchPolicyCommitmentError> {
    match &record.subject {
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
            child_participant_id,
            ..
        } => Ok(child_participant_id),
        _ => Err(DispatchPolicyCommitmentError::new(
            "fork retry resolved a non-fork commitment",
        )),
    }
}

fn validate_fork_retry(
    existing: &DispatchPolicyCommitmentV1,
    input: &ForkPolicyCommitmentInputV1,
    fork_request_hash: &str,
) -> Result<(), DispatchPolicyCommitmentError> {
    let subject_matches = matches!(
        &existing.subject,
        DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
            source_participant_id,
            child_participant_id,
            bootstrap_run_id,
        } if source_participant_id == &input.source_participant_id
            && validate_component("fork child participant id", child_participant_id).is_ok()
            && validate_component("fork bootstrap run id", bootstrap_run_id).is_ok()
    );
    let authority_matches = matches!(
        &existing.authority_link,
        PolicyCommitmentAuthorityLinkV1::ForkDispatch {
            canonical_validated_dispatch_request_sha256,
        } if canonical_validated_dispatch_request_sha256 == fork_request_hash
    );
    if !subject_matches
        || !authority_matches
        || existing.authority_store_id
            != input
                .authenticated_policy
                .source_worker_cap_ref()
                .authority_store_id
        || existing.request_id != input.request_id
        || existing.idempotency_key != input.idempotency_key
        || existing.orchestration_session_id != input.orchestration_session_id
        || existing.caller_participant_id != input.caller_participant_id
        || existing.caller_backend_id != input.caller_backend_id
        || existing.target_backend_id != input.target_backend_id
        || existing.world_id != input.world_id
        || existing.world_generation != input.world_generation
        || existing.execution_claim_link.is_some()
        || existing.fresh_spawn_reservation_ref.is_some()
        || existing.fresh_spawn_validated_request_commitment.is_some()
        || existing.parent_policy_ref != input.parent_policy_ref
        || existing.parent_policy_revision != input.parent_policy_revision
        || existing.applied_patch != input.applied_patch
        || existing.policy_snapshot_bytes.resolve_inline()?
            != input.authenticated_policy.snapshot().bytes()
        || existing.policy_snapshot_ref != *input.authenticated_policy.snapshot().snapshot_ref()
        || existing.policy_snapshot_hash != input.authenticated_policy.snapshot().hash()
        || existing.policy_snapshot_revision != input.authenticated_policy.snapshot().revision()
        || existing.reason != input.reason
        || existing.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment)
        || existing.source_worker_cap_ref.as_ref()
            != Some(input.authenticated_policy.source_worker_cap_ref())
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fork retry changed request, source cap, parent, patch, snapshot, runtime binding, or caller material",
        ));
    }
    Ok(())
}

fn fork_record_from_input(
    authority_store_id: &str,
    input: &ForkPolicyCommitmentInputV1,
    fork_request_hash: &str,
    child_participant_id: String,
    bootstrap_run_id: String,
    created_at: TimestampV1,
) -> DispatchPolicyCommitmentV1 {
    DispatchPolicyCommitmentV1 {
        schema_version: 1,
        authority_store_id: authority_store_id.into(),
        commitment_id: format!("dpc_{}", Uuid::now_v7()),
        subject: DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
            source_participant_id: input.source_participant_id.clone(),
            child_participant_id,
            bootstrap_run_id,
        },
        request_id: input.request_id.clone(),
        idempotency_key: input.idempotency_key.clone(),
        orchestration_session_id: input.orchestration_session_id.clone(),
        caller_participant_id: input.caller_participant_id.clone(),
        caller_backend_id: input.caller_backend_id.clone(),
        target_backend_id: input.target_backend_id.clone(),
        world_id: input.world_id.clone(),
        world_generation: input.world_generation,
        authority_link: PolicyCommitmentAuthorityLinkV1::ForkDispatch {
            canonical_validated_dispatch_request_sha256: fork_request_hash.into(),
        },
        execution_claim_link: None,
        fresh_spawn_reservation_ref: None,
        fresh_spawn_validated_request_commitment: None,
        parent_policy_ref: input.parent_policy_ref.clone(),
        parent_policy_revision: input.parent_policy_revision.clone(),
        applied_patch: input.applied_patch.clone(),
        policy_snapshot_bytes: ImmutableBytesMaterialV1::inline(
            input.authenticated_policy.snapshot().bytes(),
        ),
        policy_snapshot_ref: input.authenticated_policy.snapshot().snapshot_ref().clone(),
        policy_snapshot_hash: input.authenticated_policy.snapshot().hash().into(),
        policy_snapshot_revision: input.authenticated_policy.snapshot().revision().into(),
        reason: input.reason.clone(),
        retained_worker_cap_link: Some(RetainedWorkerCapLinkV1::ThisCommitment),
        source_worker_cap_ref: Some(input.authenticated_policy.source_worker_cap_ref().clone()),
        created_revision: 1,
        application_revision: 1,
        status: PolicyCommitmentStatusV1::Immutable,
        created_at,
        exact_linkage_hash: String::new(),
    }
}

fn validate_exact_non_spawn_retry(
    existing: &DispatchPolicyCommitmentV1,
    requested: &DispatchPolicyCommitmentV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    let mut normalized = requested.clone();
    normalized.commitment_id = existing.commitment_id.clone();
    normalized.created_at = existing.created_at.clone();
    normalized.exact_linkage_hash = existing.exact_linkage_hash.clone();
    if &normalized != existing {
        return Err(DispatchPolicyCommitmentError::new(
            "non-Spawn retry changed immutable request, subject, policy, cap, or linkage material",
        ));
    }
    Ok(())
}

fn stable_admission_link_from_record(
    record: &RetainedWorkerAdmissionRecordV1,
) -> Result<RetainedWorkerAdmissionStableIdentityLinkV1, DispatchPolicyCommitmentError> {
    let registration = match &record.state {
        RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration }
        | RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { registration, .. }
        | RetainedWorkerAdmissionStateV1::Routable { registration, .. }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal { registration, .. }
        | RetainedWorkerAdmissionStateV1::Terminal { registration, .. } => registration.clone(),
        RetainedWorkerAdmissionStateV1::SlotReserved { .. }
        | RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => {
            return Err(DispatchPolicyCommitmentError::new(
                "B3.2a record is not registration-bearing",
            ));
        }
    };
    let stable_source_fields = RetainedWorkerAdmissionStableSourceFieldsV1 {
        schema_version: record.schema_version,
        authority_store_id: record.authority_store_id.clone(),
        issuer_request_id: record.issuer_request_id.clone(),
        canonical_spawn_fingerprint: record.canonical_spawn_fingerprint.clone(),
        orchestration_session_id: record.orchestration_session_id.clone(),
        admission_authority_revision: record.admission_authority_revision,
        admission_authority_record_commitment: record.admission_authority_record_commitment.clone(),
        retained_participant_id: record.retained_participant_id.clone(),
        bootstrap_run_id: record.bootstrap_run_id.clone(),
        backend_id: record.backend_id.clone(),
        protocol: record.protocol.clone(),
        world_binding: record.world_binding.clone(),
        current_policy_ref: record.current_policy_ref.clone(),
        current_policy_revision: record.current_policy_revision.clone(),
        max_live_retained_workers: record.max_live_retained_workers,
    };
    let mut hash_input = serde_json::Map::new();
    hash_input.insert(
        "domain".into(),
        Value::String(STABLE_ADMISSION_HASH_DOMAIN.into()),
    );
    hash_input.insert(
        "stable_source_fields".into(),
        serde_json::to_value(&stable_source_fields)
            .map_err(|_| DispatchPolicyCommitmentError::new("serialize B3.2a stable fields"))?,
    );
    hash_input.insert(
        "registration".into(),
        serde_json::to_value(&registration)
            .map_err(|_| DispatchPolicyCommitmentError::new("serialize B3.2a registration"))?,
    );
    Ok(RetainedWorkerAdmissionStableIdentityLinkV1 {
        registry_key: RetainedWorkerAdmissionRegistryKeyV1 {
            orchestration_session_id: record.orchestration_session_id.clone(),
            retained_participant_id: record.retained_participant_id.clone(),
        },
        stable_source_fields,
        registration,
        stable_identity_hash: sha256_hex(&encode_canonical(&Value::Object(hash_input))?),
    })
}

fn validate_proof_matches_reservation(
    proof: &AuthenticatedFreshSpawnReservationProofV1,
    reservation: &DispatchPolicyCommitmentReservationV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    if proof.reservation_ref != reservation_ref(reservation)
        || proof.request_id != reservation.request_id
        || proof.idempotency_key != reservation.idempotency_key
        || proof.orchestration_session_id != reservation.orchestration_session_id
        || proof.caller_participant_id != reservation.caller_participant_id
        || proof.target_backend_id != reservation.target_backend_id
        || proof.world_id != reservation.world_id
        || proof.world_generation != reservation.world_generation
        || proof.parent_policy_ref != reservation.parent_policy_ref
        || proof.parent_policy_revision != reservation.parent_policy_revision
        || proof.retained_participant_id != reservation.retained_participant_id
        || proof.bootstrap_run_id != reservation.bootstrap_run_id
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fresh-spawn authenticated proof changed before publication",
        ));
    }
    Ok(())
}

fn validate_stable_admission_matches_reservation(
    link: &RetainedWorkerAdmissionStableIdentityLinkV1,
    reservation: &DispatchPolicyCommitmentReservationV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    let stable = &link.stable_source_fields;
    if stable.authority_store_id != reservation.authority_store_id
        || stable.issuer_request_id != reservation.request_id
        || stable.orchestration_session_id != reservation.orchestration_session_id
        || stable.retained_participant_id != reservation.retained_participant_id
        || stable.bootstrap_run_id != reservation.bootstrap_run_id
        || stable.backend_id != reservation.target_backend_id
        || stable.world_binding.world_id != reservation.world_id
        || stable.world_binding.world_generation != reservation.world_generation
        || stable.current_policy_ref != reservation.parent_policy_ref
        || stable.current_policy_revision != reservation.parent_policy_revision
        || link.registry_key.orchestration_session_id != reservation.orchestration_session_id
        || link.registry_key.retained_participant_id != reservation.retained_participant_id
    {
        return Err(DispatchPolicyCommitmentError::new(
            "B3.2a stable admission identity does not exact-join the E2 reservation",
        ));
    }
    validate_digest("stable admission hash", &link.stable_identity_hash)
}

fn validate_fresh_spawn_record_matches(
    record: &DispatchPolicyCommitmentV1,
    reservation: &DispatchPolicyCommitmentReservationV1,
    stable_admission: &RetainedWorkerAdmissionStableIdentityLinkV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    if record.subject != reservation.subject
        || record.commitment_id != reservation.proposed_commitment_id
        || record.fresh_spawn_reservation_ref.as_ref() != Some(&reservation_ref(reservation))
        || record.fresh_spawn_validated_request_commitment.as_ref()
            != Some(&reservation.validated_request_commitment)
        || record.authority_link
            != (PolicyCommitmentAuthorityLinkV1::RetainedAdmission {
                stable_admission_identity: stable_admission.clone(),
            })
        || record.execution_claim_link.is_some()
        || record.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment)
        || record.source_worker_cap_ref.is_some()
        || record.policy_snapshot_bytes != reservation.policy_snapshot_bytes
        || record.policy_snapshot_hash != reservation.policy_snapshot_hash
        || record.parent_policy_ref != reservation.parent_policy_ref
        || record.parent_policy_revision != reservation.parent_policy_revision
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fresh-spawn committed retry changed immutable linkage",
        ));
    }
    Ok(())
}

pub(crate) fn resolve_retained_worker_cap(
    authority: &HostSessionAuthority,
    orchestration_session_id: &str,
    retained_participant_id: &str,
) -> Result<ResolvedPolicyCommitmentCompatibilityV1, DispatchPolicyCommitmentError> {
    validate_component("orchestration_session_id", orchestration_session_id)?;
    validate_component("retained_participant_id", retained_participant_id)?;
    let registry_exists = crate::execution::agent_runtime::host_session_authority::store::dispatch_policy_commitment_registry_exists_for_authority(authority)
        .map_err(storage_error)?;
    if !registry_exists {
        return Ok(
            ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                retained_participant_id: retained_participant_id.into(),
                reason: PolicyCommitmentCompatibilityReasonV1::MissingCanonicalCapBytes,
            },
        );
    }
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        let Some(registry_bytes) = transaction.read_registry()? else {
            return Ok(
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                    retained_participant_id: retained_participant_id.into(),
                    reason: PolicyCommitmentCompatibilityReasonV1::MissingCanonicalCapBytes,
                },
            );
        };
        let registry: DispatchPolicyCommitmentRegistryV1 =
            retain_semantic(decode_canonical(&registry_bytes), &mut semantic_failure)?;
        retain_semantic(
            validate_registry_header_and_key(
                &registry,
                storage.authority_store_id(),
                &transaction.read_keys()?,
            )
            .map(|_| ()),
            &mut semantic_failure,
        )?;
        let Some(reference) = registry
            .retained_worker_caps_by_participant_id
            .get(retained_participant_id)
        else {
            return Ok(
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                    retained_participant_id: retained_participant_id.into(),
                    reason: PolicyCommitmentCompatibilityReasonV1::MissingCanonicalCapBytes,
                },
            );
        };
        let Some(record) = registry.commitments_by_id.get(&reference.commitment_id) else {
            return Ok(
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                    retained_participant_id: retained_participant_id.into(),
                    reason: PolicyCommitmentCompatibilityReasonV1::MissingImmutableCapBytesRef,
                },
            );
        };
        if record.schema_version != 1 {
            return Ok(
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                    retained_participant_id: retained_participant_id.into(),
                    reason: PolicyCommitmentCompatibilityReasonV1::UnsupportedCapSchema,
                },
            );
        }
        match &record.policy_snapshot_bytes {
            ImmutableBytesMaterialV1::Durable { .. } => {
                return Ok(
                    ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                        retained_participant_id: retained_participant_id.into(),
                        reason: PolicyCommitmentCompatibilityReasonV1::MissingImmutableCapBytesRef,
                    },
                );
            }
            ImmutableBytesMaterialV1::Inline { byte_length, .. } if *byte_length == 0 => {
                return Ok(
                    ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                        retained_participant_id: retained_participant_id.into(),
                        reason: PolicyCommitmentCompatibilityReasonV1::MissingCanonicalCapBytes,
                    },
                );
            }
            ImmutableBytesMaterialV1::Inline { .. } => {}
        }
        if validate_commitment(record, Some(reference)).is_err() {
            return Ok(
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                    retained_participant_id: retained_participant_id.into(),
                    reason: PolicyCommitmentCompatibilityReasonV1::CapHashMismatch,
                },
            );
        }
        let subject_matches = match &record.subject {
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
                retained_participant_id: participant_id,
                ..
            } => participant_id == retained_participant_id,
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
                child_participant_id,
                ..
            } => child_participant_id == retained_participant_id,
            DispatchPolicyCommitmentSubjectV1::EphemeralWork { .. }
            | DispatchPolicyCommitmentSubjectV1::RetainedWorkerTurn { .. } => false,
        };
        if !subject_matches
            || record.orchestration_session_id != orchestration_session_id
            || record.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment)
        {
            return Ok(
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                    retained_participant_id: retained_participant_id.into(),
                    reason: PolicyCommitmentCompatibilityReasonV1::CapSubjectOrLinkageMismatch,
                },
            );
        }
        Ok(ResolvedPolicyCommitmentCompatibilityV1::Compatible {
            cap: DurablePolicyCommitmentDescriptorV1 {
                commitment_ref: reference.clone(),
                orchestration_session_id: record.orchestration_session_id.clone(),
                retained_participant_id: retained_participant_id.into(),
                launch_parent_policy_ref: record.parent_policy_ref.clone(),
                launch_parent_policy_revision: record.parent_policy_revision.clone(),
            },
        })
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result.map_err(storage_error)
}

pub(crate) fn authenticate_dispatch_policy_commitment(
    authority: &HostSessionAuthority,
    reference: &DispatchPolicyCommitmentRefV1,
) -> Result<AuthenticatedDispatchPolicyCommitmentV1, DispatchPolicyCommitmentError> {
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    if reference.authority_store_id != storage.authority_store_id() {
        return Err(DispatchPolicyCommitmentError::new(
            "dispatch policy commitment authority store mismatch",
        ));
    }
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        let registry: DispatchPolicyCommitmentRegistryV1 = retain_semantic(
            decode_canonical(
                &transaction
                    .read_registry()?
                    .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?,
            ),
            &mut semantic_failure,
        )?;
        retain_semantic(
            validate_registry_and_key(
                &registry,
                storage.authority_store_id(),
                &transaction.read_keys()?,
            )
            .map(|_| ()),
            &mut semantic_failure,
        )?;
        let record = registry
            .commitments_by_id
            .get(&reference.commitment_id)
            .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
        retain_semantic(
            validate_commitment(record, Some(reference)),
            &mut semantic_failure,
        )?;
        let snapshot_bytes = retain_semantic(
            record.policy_snapshot_bytes.resolve_inline(),
            &mut semantic_failure,
        )?;
        let snapshot: PolicySnapshotV3 = retain_semantic(
            serde_json::from_slice(&snapshot_bytes).map_err(|_| {
                DispatchPolicyCommitmentError::new("decode committed PolicySnapshotV3")
            }),
            &mut semantic_failure,
        )?;
        let reserialized = retain_semantic(
            serde_json::to_vec(&snapshot).map_err(|_| {
                DispatchPolicyCommitmentError::new("reserialize committed PolicySnapshotV3")
            }),
            &mut semantic_failure,
        )?;
        if reserialized != snapshot_bytes
            || sha256_hex(&snapshot_bytes) != record.policy_snapshot_hash
        {
            semantic_failure = Some(DispatchPolicyCommitmentError::new(
                "committed policy snapshot byte identity mismatch",
            ));
            return Err(BootstrapError::dispatch_policy_commitment_semantic());
        }
        Ok(AuthenticatedDispatchPolicyCommitmentV1 {
            record: record.clone(),
            snapshot,
            snapshot_bytes,
        })
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result.map_err(storage_error)
}

pub(crate) fn resolve_retained_turn_policy_material(
    authority: &HostSessionAuthority,
    input: RetainedTurnPolicyResolutionInputV1,
) -> Result<AuthenticatedRetainedTurnPolicyMaterialV1, DispatchPolicyCommitmentError> {
    let authenticated =
        authenticate_dispatch_policy_commitment(authority, input.cap.commitment_ref())?;
    if input.cap.commitment_ref() != &authenticated.commitment_ref()
        || input.cap.authority_store_id() != authenticated.authority_store_id()
        || input.cap.orchestration_session_id() != authenticated.orchestration_session_id()
        || input.cap.retained_participant_id() != input.retained_participant_id
        || authenticated.retained_participant_id() != Some(input.retained_participant_id.as_str())
        || authenticated.retained_worker_cap_link()
            != Some(&RetainedWorkerCapLinkV1::ThisCommitment)
        || authenticated.orchestration_session_id() != input.orchestration_session_id
        || authenticated.record.target_backend_id != input.target_backend_id
        || authenticated.record.world_id != input.world_id
        || authenticated.record.world_generation != input.world_generation
    {
        return Err(DispatchPolicyCommitmentError::new(
            "retained-turn cap descriptor or runtime binding mismatch",
        ));
    }
    validate_retained_turn_patch_bindings(&input)?;
    let snapshot = conjoin_policy_snapshots(
        input.current_parent_and_turn_patch.snapshot(),
        authenticated.policy_snapshot(),
    )?;
    let bytes = serde_json::to_vec(&snapshot).map_err(|_| {
        DispatchPolicyCommitmentError::new("serialize retained-turn PolicySnapshotV3")
    })?;
    let hash = sha256_hex(&bytes);
    let carrier = DispatchPolicySnapshotCarrierV1 {
        schema_version: 1,
        immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: authenticated.record.authority_store_id.clone(),
            commitment_id: authenticated.record.commitment_id.clone(),
            exact_linkage_hash: authenticated.record.exact_linkage_hash.clone(),
        },
        immutable_worker_cap_created_revision: authenticated.record.created_revision,
        immutable_worker_cap_application_revision: authenticated.record.application_revision,
        subject: RetainedTurnPolicyCommitmentSubjectV1 {
            retained_participant_id: input.retained_participant_id,
            active_run_id: input.active_run_id,
            message_id: input.message_id,
        },
        orchestration_session_id: input.orchestration_session_id,
        caller_participant_id: input.caller_participant_id,
        caller_backend_id: input.caller_backend_id,
        target_backend_id: input.target_backend_id,
        target_world: WorldBindingRefV1 {
            world_id: input.world_id,
            world_generation: input.world_generation,
        },
        policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
        policy_snapshot_byte_length: bytes.len() as u64,
        policy_snapshot_ref: transport_policy_ref(
            input.current_parent_and_turn_patch.snapshot_ref(),
        ),
        policy_snapshot_hash: hash,
        policy_snapshot_revision: input.current_parent_and_turn_patch.revision().into(),
        reason: input
            .turn_patch
            .as_ref()
            .and_then(|patch| patch.reason.clone()),
    };
    carrier.validate().map_err(|_| {
        DispatchPolicyCommitmentError::new("constructed retained-turn carrier is invalid")
    })?;
    Ok(AuthenticatedRetainedTurnPolicyMaterialV1 { carrier })
}

pub(crate) fn resolve_fork_policy_material(
    authority: &HostSessionAuthority,
    input: ForkPolicyResolutionInputV1,
) -> Result<AuthenticatedForkPolicyMaterialV1, DispatchPolicyCommitmentError> {
    for (name, value) in [
        ("request_id", input.request_id.as_str()),
        (
            "orchestration_session_id",
            input.orchestration_session_id.as_str(),
        ),
        (
            "caller_participant_id",
            input.caller_participant_id.as_str(),
        ),
        ("caller_backend_id", input.caller_backend_id.as_str()),
        ("target_backend_id", input.target_backend_id.as_str()),
        ("world_id", input.world_id.as_str()),
        (
            "source_participant_id",
            input.source_participant_id.as_str(),
        ),
    ] {
        validate_component(name, value)?;
    }
    if input.world_generation == 0 {
        return Err(DispatchPolicyCommitmentError::new(
            "fork world generation is invalid",
        ));
    }
    let authenticated =
        authenticate_dispatch_policy_commitment(authority, input.source_cap.commitment_ref())?;
    if input.source_cap.commitment_ref() != &authenticated.commitment_ref()
        || input.source_cap.authority_store_id() != authenticated.authority_store_id()
        || input.source_cap.orchestration_session_id() != authenticated.orchestration_session_id()
        || input.source_cap.retained_participant_id() != input.source_participant_id
        || authenticated.retained_participant_id() != Some(input.source_participant_id.as_str())
        || authenticated.retained_worker_cap_link()
            != Some(&RetainedWorkerCapLinkV1::ThisCommitment)
        || authenticated.orchestration_session_id() != input.orchestration_session_id
        || authenticated.record.target_backend_id != input.target_backend_id
        || authenticated.record.world_id != input.world_id
        || authenticated.record.world_generation != input.world_generation
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fork source-cap descriptor or runtime binding mismatch",
        ));
    }
    validate_fork_patch_bindings(&input)?;
    let snapshot = conjoin_policy_snapshots(
        input.current_parent_and_fork_patch.snapshot(),
        authenticated.policy_snapshot(),
    )?;
    let bytes = serde_json::to_vec(&snapshot)
        .map_err(|_| DispatchPolicyCommitmentError::new("serialize fork PolicySnapshotV3"))?;
    let hash = sha256_hex(&bytes);
    Ok(AuthenticatedForkPolicyMaterialV1 {
        snapshot: ValidatedPolicySnapshotMaterialV1 {
            snapshot,
            bytes,
            snapshot_ref: input.current_parent_and_fork_patch.snapshot_ref().clone(),
            hash,
            revision: input.current_parent_and_fork_patch.revision().into(),
        },
        source_worker_cap_ref: authenticated.commitment_ref(),
    })
}

fn validate_fork_patch_bindings(
    input: &ForkPolicyResolutionInputV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    let Some(patch) = &input.fork_patch else {
        return Ok(());
    };
    patch
        .validate()
        .map_err(|_| DispatchPolicyCommitmentError::new("invalid fork E1 patch"))?;
    let expected_parent = transport_policy_ref(input.current_parent_and_fork_patch.snapshot_ref());
    if patch.request_id != input.request_id
        || patch.orchestration_session_id != input.orchestration_session_id
        || patch.caller_participant_id != input.caller_participant_id
        || patch.target_backend_id != input.target_backend_id
        || patch.target_world.world_id != input.world_id
        || patch.target_world.world_generation != input.world_generation
        || patch.parent_policy_ref != expected_parent
        || patch.parent_policy_revision != input.current_parent_and_fork_patch.revision()
        || !matches!(
            &patch.applies_to,
            transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerFork {
                source_participant_id
            } if source_participant_id == &input.source_participant_id
        )
        || patch
            .restricted_policy_patch
            .world_fs
            .as_ref()
            .is_none_or(|world_fs| world_fs.is_empty())
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fork patch does not match current authority, world, or source subject",
        ));
    }
    Ok(())
}

fn validate_retained_turn_patch_bindings(
    input: &RetainedTurnPolicyResolutionInputV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    let Some(patch) = &input.turn_patch else {
        return Ok(());
    };
    patch
        .validate()
        .map_err(|_| DispatchPolicyCommitmentError::new("invalid retained-turn E1 patch"))?;
    let expected_parent = transport_policy_ref(input.current_parent_and_turn_patch.snapshot_ref());
    if patch.orchestration_session_id != input.orchestration_session_id
        || patch.caller_participant_id != input.caller_participant_id
        || patch.target_backend_id != input.target_backend_id
        || patch.target_world.world_id != input.world_id
        || patch.target_world.world_generation != input.world_generation
        || patch.parent_policy_ref != expected_parent
        || patch.parent_policy_revision != input.current_parent_and_turn_patch.revision()
        || !matches!(
            &patch.applies_to,
            transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerTurn {
                retained_participant_id
            } if retained_participant_id == &input.retained_participant_id
        )
        || patch
            .restricted_policy_patch
            .world_fs
            .as_ref()
            .is_none_or(|world_fs| world_fs.is_empty())
    {
        return Err(DispatchPolicyCommitmentError::new(
            "retained-turn patch does not match current authority, world, or subject",
        ));
    }
    Ok(())
}

fn transport_policy_ref(reference: &AuthorityObjectRefV1) -> transport_api_types::PolicyRefV1 {
    let object_kind = match reference.object_kind {
        super::host_session_authority::schema::AuthorityObjectKindV1::AgentDescriptor => {
            transport_api_types::AuthorityObjectKindV1::AgentDescriptor
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::RetainedWorker => {
            transport_api_types::AuthorityObjectKindV1::RetainedWorker
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::ResumeHandle => {
            transport_api_types::AuthorityObjectKindV1::ResumeHandle
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::Policy => {
            transport_api_types::AuthorityObjectKindV1::Policy
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::HostAttachContract => {
            transport_api_types::AuthorityObjectKindV1::HostAttachContract
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::TransitionTransportPayload => {
            transport_api_types::AuthorityObjectKindV1::TransitionTransportPayload
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::TransitionInput => {
            transport_api_types::AuthorityObjectKindV1::TransitionInput
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::LeaseToken => {
            transport_api_types::AuthorityObjectKindV1::LeaseToken
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::ApplicationResult => {
            transport_api_types::AuthorityObjectKindV1::ApplicationResult
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::InputAcceptance => {
            transport_api_types::AuthorityObjectKindV1::InputAcceptance
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::StartupOwnershipResult => {
            transport_api_types::AuthorityObjectKindV1::StartupOwnershipResult
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::ObligationSnapshot => {
            transport_api_types::AuthorityObjectKindV1::ObligationSnapshot
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::PostTurnProtocolEvent => {
            transport_api_types::AuthorityObjectKindV1::PostTurnProtocolEvent
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::PostTurnCompletion => {
            transport_api_types::AuthorityObjectKindV1::PostTurnCompletion
        }
        super::host_session_authority::schema::AuthorityObjectKindV1::TerminalHandoff => {
            transport_api_types::AuthorityObjectKindV1::TerminalHandoff
        }
    };
    let commitment = match &reference.commitment {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => {
            transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: digest_hex.clone(),
            }
        }
        AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id,
            domain,
            digest_hex,
        } => transport_api_types::OpaqueAuthorityCommitmentV1::StoreHmacSha256 {
            key_id: key_id.clone(),
            domain: domain.clone(),
            digest_hex: digest_hex.clone(),
        },
    };
    transport_api_types::PolicyRefV1 {
        ref_id: reference.ref_id.clone(),
        object_kind,
        schema_version: reference.schema_version,
        commitment,
    }
}

fn conjoin_policy_snapshots(
    current: &PolicySnapshotV3,
    cap: &PolicySnapshotV3,
) -> Result<PolicySnapshotV3, DispatchPolicyCommitmentError> {
    let current = current
        .canonicalize()
        .map_err(|_| DispatchPolicyCommitmentError::new("invalid current PolicySnapshotV3"))?;
    let cap = cap
        .canonicalize()
        .map_err(|_| DispatchPolicyCommitmentError::new("invalid cap PolicySnapshotV3"))?;
    let current_discover = current.world_fs.discover.as_ref().ok_or_else(|| {
        DispatchPolicyCommitmentError::new("canonical current snapshot omits discover")
    })?;
    let cap_discover = cap.world_fs.discover.as_ref().ok_or_else(|| {
        DispatchPolicyCommitmentError::new("canonical cap snapshot omits discover")
    })?;
    let current_read = current.world_fs.read.as_ref().ok_or_else(|| {
        DispatchPolicyCommitmentError::new("canonical current snapshot omits read")
    })?;
    let cap_read =
        cap.world_fs.read.as_ref().ok_or_else(|| {
            DispatchPolicyCommitmentError::new("canonical cap snapshot omits read")
        })?;
    let discover = conjoin_dimension(current_discover, cap_discover);
    let read = conjoin_dimension(current_read, cap_read);
    let write = conjoin_write(&current.world_fs.write, &cap.world_fs.write);
    let synthesized_deny_all =
        dimension_denies_all(&discover) || dimension_denies_all(&read) || write_denies_all(&write);
    let any_deny =
        !discover.deny_list.is_empty() || !read.deny_list.is_empty() || !write.deny_list.is_empty();
    let deny_enforcement = if synthesized_deny_all {
        Some(WorldFsDenyEnforcementV3::Strict)
    } else {
        strictest_deny_enforcement(
            current.world_fs.deny_enforcement,
            cap.world_fs.deny_enforcement,
        )
        .or_else(|| any_deny.then_some(WorldFsDenyEnforcementV3::Strict))
    };
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: conjoin_net_allowed(&current.net_allowed, &cap.net_allowed),
        world_fs: transport_api_types::PolicySnapshotWorldFsV3 {
            host_visible: current.world_fs.host_visible && cap.world_fs.host_visible && !any_deny,
            fail_closed: transport_api_types::PolicySnapshotWorldFsFailClosedV3 {
                routing: current.world_fs.fail_closed.routing
                    || cap.world_fs.fail_closed.routing
                    || !write.enabled,
            },
            deny_enforcement,
            caged_required: current.world_fs.caged_required || cap.world_fs.caged_required,
            discover: Some(discover),
            read: Some(read),
            write,
        },
    }
    .canonicalize()
    .map_err(|_| DispatchPolicyCommitmentError::new("invalid retained-turn policy conjunction"))
}

fn dimension_denies_all(dimension: &PolicySnapshotWorldFsDimensionV3) -> bool {
    dimension.allow_list == ["."] && dimension.deny_list.iter().any(|value| value == "**")
}

fn write_denies_all(write: &PolicySnapshotWorldFsWriteV3) -> bool {
    write.allow_list == ["."] && write.deny_list.iter().any(|value| value == "**")
}

fn conjoin_net_allowed(left: &[String], right: &[String]) -> Vec<String> {
    if left == ["*"] {
        return right.to_vec();
    }
    if right == ["*"] {
        return left.to_vec();
    }
    let right = right.iter().collect::<std::collections::BTreeSet<_>>();
    left.iter()
        .filter(|value| right.contains(value))
        .cloned()
        .collect()
}

fn conjoin_dimension(
    left: &PolicySnapshotWorldFsDimensionV3,
    right: &PolicySnapshotWorldFsDimensionV3,
) -> PolicySnapshotWorldFsDimensionV3 {
    let mut allow_list = intersect_path_allows(&left.allow_list, &right.allow_list);
    let mut deny_list = left
        .deny_list
        .iter()
        .chain(&right.deny_list)
        .cloned()
        .collect::<Vec<_>>();
    if allow_list.is_empty() {
        allow_list.push(".".into());
        deny_list.push("**".into());
    }
    deny_list.sort();
    deny_list.dedup();
    PolicySnapshotWorldFsDimensionV3 {
        allow_list,
        deny_list,
    }
}

fn conjoin_write(
    left: &PolicySnapshotWorldFsWriteV3,
    right: &PolicySnapshotWorldFsWriteV3,
) -> PolicySnapshotWorldFsWriteV3 {
    let dimension = conjoin_dimension(
        &PolicySnapshotWorldFsDimensionV3 {
            allow_list: left.allow_list.clone(),
            deny_list: left.deny_list.clone(),
        },
        &PolicySnapshotWorldFsDimensionV3 {
            allow_list: right.allow_list.clone(),
            deny_list: right.deny_list.clone(),
        },
    );
    PolicySnapshotWorldFsWriteV3 {
        enabled: left.enabled && right.enabled,
        allow_list: dimension.allow_list,
        deny_list: dimension.deny_list,
    }
}

fn intersect_path_allows(left: &[String], right: &[String]) -> Vec<String> {
    let mut intersection = Vec::new();
    for left_path in left {
        for right_path in right {
            if path_scope_contains(left_path, right_path) {
                intersection.push(right_path.clone());
            } else if path_scope_contains(right_path, left_path) {
                intersection.push(left_path.clone());
            }
        }
    }
    intersection.sort();
    intersection.dedup();
    intersection
}

fn path_scope_contains(parent: &str, child: &str) -> bool {
    parent == "."
        || parent == child
        || child
            .strip_prefix(parent)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn strictest_deny_enforcement(
    left: Option<WorldFsDenyEnforcementV3>,
    right: Option<WorldFsDenyEnforcementV3>,
) -> Option<WorldFsDenyEnforcementV3> {
    fn rank(value: Option<WorldFsDenyEnforcementV3>) -> u8 {
        match value {
            None => 0,
            Some(WorldFsDenyEnforcementV3::Weak) => 1,
            Some(WorldFsDenyEnforcementV3::PreferStrict) => 2,
            Some(WorldFsDenyEnforcementV3::Strict) => 3,
        }
    }
    if rank(left) >= rank(right) {
        left
    } else {
        right
    }
}

#[cfg(test)]
pub(crate) fn test_corrupt_retained_worker_cap_hash(
    authority: &HostSessionAuthority,
    retained_participant_id: &str,
) -> Result<(), DispatchPolicyCommitmentError> {
    let storage =
        dispatch_policy_commitment_storage_for_authority(authority).map_err(storage_error)?;
    let mut semantic_failure = None;
    let result = storage.transaction(|transaction| {
        let mut registry: DispatchPolicyCommitmentRegistryV1 = retain_semantic(
            decode_canonical(
                &transaction
                    .read_registry()?
                    .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?,
            ),
            &mut semantic_failure,
        )?;
        let reference = registry
            .retained_worker_caps_by_participant_id
            .get_mut(retained_participant_id)
            .ok_or_else(BootstrapError::dispatch_policy_commitment_semantic)?;
        reference.exact_linkage_hash = "00".repeat(32);
        let bytes = retain_semantic(encode_canonical(&registry), &mut semantic_failure)?;
        transaction.replace_registry(
            &format!("dispatch-policy-registry--{}.tmp", random_nonce()),
            &bytes,
        )?;
        Ok(())
    });
    if let Some(error) = semantic_failure {
        return Err(error);
    }
    result.map_err(storage_error)
}

fn storage_error(error: BootstrapError) -> DispatchPolicyCommitmentError {
    DispatchPolicyCommitmentError::new(error.to_string())
}

fn retain_semantic<T>(
    result: Result<T, DispatchPolicyCommitmentError>,
    semantic_failure: &mut Option<DispatchPolicyCommitmentError>,
) -> Result<T, BootstrapError> {
    result.map_err(|error| {
        *semantic_failure = Some(error);
        BootstrapError::dispatch_policy_commitment_semantic()
    })
}

fn current_timestamp() -> Result<TimestampV1, DispatchPolicyCommitmentError> {
    TimestampV1::parse(Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true))
        .map_err(|_| DispatchPolicyCommitmentError::new("construct E2 timestamp"))
}

fn random_nonce() -> String {
    let mut entropy = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut entropy);
    lower_hex(&entropy)
}

fn encode_canonical<T: Serialize>(value: &T) -> Result<Vec<u8>, DispatchPolicyCommitmentError> {
    let value = serde_json::to_value(value)
        .map_err(|_| DispatchPolicyCommitmentError::new("serialize E2 canonical value"))?;
    let mut bytes = Vec::new();
    encode_canonical_value(&value, &mut bytes)?;
    Ok(bytes)
}

pub(crate) fn canonical_validated_fork_request_bytes(
    request: &ValidatedWorldDispatchRequestV1,
) -> Result<Vec<u8>, DispatchPolicyCommitmentError> {
    encode_canonical(request)
}

fn encode_canonical_value(
    value: &Value,
    output: &mut Vec<u8>,
) -> Result<(), DispatchPolicyCommitmentError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_writer(output, value)
                .map_err(|_| DispatchPolicyCommitmentError::new("encode E2 canonical scalar"))?;
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                encode_canonical_value(value, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            output.push(b'{');
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key)
                    .map_err(|_| DispatchPolicyCommitmentError::new("encode E2 canonical key"))?;
                output.push(b':');
                encode_canonical_value(&values[key], output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn decode_canonical<T>(bytes: &[u8]) -> Result<T, DispatchPolicyCommitmentError>
where
    T: DeserializeOwned + Serialize,
{
    let value: T = serde_json::from_slice(bytes)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode E2 canonical value"))?;
    if encode_canonical(&value)? != bytes {
        return Err(DispatchPolicyCommitmentError::new(
            "E2 bytes are not in canonical JSON form",
        ));
    }
    Ok(value)
}

fn validate_component(name: &str, value: &str) -> Result<(), DispatchPolicyCommitmentError> {
    if value.is_empty()
        || value.len() > 512
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b'/' || byte == b'\\')
    {
        return Err(DispatchPolicyCommitmentError::new(format!(
            "invalid E2 {name}"
        )));
    }
    Ok(())
}

fn validate_digest(name: &str, value: &str) -> Result<(), DispatchPolicyCommitmentError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(DispatchPolicyCommitmentError::new(format!(
            "invalid E2 {name}"
        )));
    }
    Ok(())
}

fn validate_registry_and_key(
    registry: &DispatchPolicyCommitmentRegistryV1,
    authority_store_id: &str,
    keys: &[(String, Vec<u8>)],
) -> Result<DispatchPolicyCommitmentKeyEnvelopeV1, DispatchPolicyCommitmentError> {
    let envelope = validate_registry_header_and_key(registry, authority_store_id, keys)?;
    for (id, reservation) in &registry.reservations_by_id {
        if id != &reservation.reservation_id {
            return Err(DispatchPolicyCommitmentError::new(
                "E2 reservation map key mismatch",
            ));
        }
        validate_reservation(reservation, &reservation_ref(reservation))?;
        if reservation.validated_request_commitment.key_id != registry.request_commitment_key.key_id
            || reservation.validated_request_commitment.algorithm
                != registry.request_commitment_key.algorithm
        {
            return Err(DispatchPolicyCommitmentError::new(
                "E2 reservation request commitment key identity mismatch",
            ));
        }
    }
    for (id, record) in &registry.commitments_by_id {
        if id != &record.commitment_id {
            return Err(DispatchPolicyCommitmentError::new(
                "E2 commitment map key mismatch",
            ));
        }
        validate_commitment(record, None)?;
    }
    validate_registry_graph(registry)?;
    Ok(envelope)
}

fn validate_registry_graph(
    registry: &DispatchPolicyCommitmentRegistryV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    let mut indexed_reservations = std::collections::BTreeSet::new();
    let mut indexed_commitments = std::collections::BTreeSet::new();
    for (digest, index) in &registry.request_subject_index {
        if digest != &lookup_index_key(&index.lookup_key)?
            || index.lookup_key.authority_store_id != registry.authority_store_id
        {
            return Err(DispatchPolicyCommitmentError::new(
                "E2 request/subject index key mismatch",
            ));
        }
        match &index.entry {
            DispatchPolicyCommitmentIndexEntryV1::FreshSpawnReserved { reservation_ref } => {
                let reservation = registry
                    .reservations_by_id
                    .get(&reservation_ref.reservation_id)
                    .ok_or_else(|| {
                        DispatchPolicyCommitmentError::new("E2 reservation index is torn")
                    })?;
                validate_reservation(reservation, reservation_ref)?;
                if reservation.lookup_key != index.lookup_key
                    || !indexed_reservations.insert(reservation_ref.reservation_id.clone())
                {
                    return Err(DispatchPolicyCommitmentError::new(
                        "E2 reservation index is duplicated or mismatched",
                    ));
                }
            }
            DispatchPolicyCommitmentIndexEntryV1::Committed {
                reservation_ref: committed_reservation_ref,
                fresh_spawn_validated_request_commitment,
                commitment_ref,
            } => {
                let record = registry
                    .commitments_by_id
                    .get(&commitment_ref.commitment_id)
                    .ok_or_else(|| {
                        DispatchPolicyCommitmentError::new("E2 committed index is torn")
                    })?;
                validate_commitment(record, Some(commitment_ref))?;
                if commitment_lookup_key(record) != index.lookup_key
                    || !indexed_commitments.insert(commitment_ref.commitment_id.clone())
                {
                    return Err(DispatchPolicyCommitmentError::new(
                        "E2 committed index is duplicated or mismatched",
                    ));
                }
                match (
                    committed_reservation_ref,
                    fresh_spawn_validated_request_commitment,
                ) {
                    (Some(indexed_reservation_ref), Some(request_commitment)) => {
                        let reservation = registry
                            .reservations_by_id
                            .get(&indexed_reservation_ref.reservation_id)
                            .ok_or_else(|| {
                                DispatchPolicyCommitmentError::new(
                                    "E2 committed Fresh Spawn reservation is torn",
                                )
                            })?;
                        if reservation_ref(reservation) != *indexed_reservation_ref
                            || reservation.validated_request_commitment != *request_commitment
                            || record.fresh_spawn_reservation_ref.as_ref()
                                != Some(indexed_reservation_ref)
                            || record.fresh_spawn_validated_request_commitment.as_ref()
                                != Some(request_commitment)
                        {
                            return Err(DispatchPolicyCommitmentError::new(
                                "E2 committed Fresh Spawn linkage mismatch",
                            ));
                        }
                        indexed_reservations.insert(indexed_reservation_ref.reservation_id.clone());
                    }
                    (None, None)
                        if record.fresh_spawn_reservation_ref.is_none()
                            && record.fresh_spawn_validated_request_commitment.is_none() => {}
                    _ => {
                        return Err(DispatchPolicyCommitmentError::new(
                            "E2 committed index mixes Fresh Spawn and non-Spawn linkage",
                        ));
                    }
                }
            }
        }
    }
    if indexed_reservations.len() != registry.reservations_by_id.len()
        || indexed_commitments.len() != registry.commitments_by_id.len()
    {
        return Err(DispatchPolicyCommitmentError::new(
            "E2 registry contains unindexed reservation or commitment",
        ));
    }
    for (participant_id, reference) in &registry.retained_worker_caps_by_participant_id {
        let record = registry
            .commitments_by_id
            .get(&reference.commitment_id)
            .ok_or_else(|| DispatchPolicyCommitmentError::new("E2 worker-cap index is torn"))?;
        validate_commitment(record, Some(reference))?;
        let record_participant = match &record.subject {
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerLaunch {
                retained_participant_id,
                ..
            } => retained_participant_id,
            DispatchPolicyCommitmentSubjectV1::RetainedWorkerFork {
                child_participant_id,
                ..
            } => child_participant_id,
            _ => {
                return Err(DispatchPolicyCommitmentError::new(
                    "E2 worker cap points at a non-cap subject",
                ));
            }
        };
        if participant_id != record_participant
            || record.retained_worker_cap_link != Some(RetainedWorkerCapLinkV1::ThisCommitment)
        {
            return Err(DispatchPolicyCommitmentError::new(
                "E2 worker-cap participant or linkage mismatch",
            ));
        }
    }
    Ok(())
}

fn validate_registry_header_and_key(
    registry: &DispatchPolicyCommitmentRegistryV1,
    authority_store_id: &str,
    keys: &[(String, Vec<u8>)],
) -> Result<DispatchPolicyCommitmentKeyEnvelopeV1, DispatchPolicyCommitmentError> {
    if registry.schema_version != 1
        || registry.authority_store_id != authority_store_id
        || registry.request_commitment_key.schema_version != 1
        || registry.request_commitment_key.authority_store_id != authority_store_id
        || !matches!(
            registry.request_commitment_key.algorithm,
            FreshSpawnRequestCommitmentAlgorithmV1::HmacSha256
        )
    {
        return Err(DispatchPolicyCommitmentError::new(
            "unsupported or cross-store E2 registry",
        ));
    }
    validate_component(
        "request commitment key id",
        &registry.request_commitment_key.key_id,
    )?;
    let expected_name = format!("{}.key", registry.request_commitment_key.key_id);
    if keys.len() != 1 || keys[0].0 != expected_name {
        return Err(DispatchPolicyCommitmentError::new(
            "E2 registry must have exactly one matching private key envelope",
        ));
    }
    let envelope: DispatchPolicyCommitmentKeyEnvelopeV1 = decode_canonical(&keys[0].1)?;
    if envelope.schema_version != 1
        || envelope.authority_store_id != authority_store_id
        || envelope.key_id != registry.request_commitment_key.key_id
        || envelope.created_at != registry.request_commitment_key.created_at
        || envelope.algorithm != registry.request_commitment_key.algorithm
    {
        return Err(DispatchPolicyCommitmentError::new(
            "E2 private key envelope identity mismatch",
        ));
    }
    Ok(envelope)
}

fn validate_fresh_spawn_input(
    input: &FreshSpawnReservationInputV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    for (name, value) in [
        ("request_id", input.spawn_request.request_id.as_str()),
        (
            "idempotency_key",
            input.spawn_request.idempotency_key.as_str(),
        ),
        (
            "orchestration_session_id",
            input.spawn_request.orchestration_session_id.as_str(),
        ),
        (
            "caller_participant_id",
            input.spawn_request.caller_participant_id.as_str(),
        ),
        ("caller_backend_id", input.caller_backend_id.as_str()),
        (
            "target_backend_id",
            input.spawn_request.target_backend_id.as_str(),
        ),
        ("world_id", input.spawn_request.world_id.as_str()),
        (
            "parent_policy_revision",
            input.parent_policy_revision.as_str(),
        ),
    ] {
        validate_component(name, value)?;
    }
    if input.spawn_request.schema_version != 1
        || input.spawn_request.action != "spawn_world_worker"
        || input.spawn_request.mode != "retained"
        || input.spawn_request.task_run_id.is_some()
        || input.spawn_request.target_participant_id.is_some()
        || input.spawn_request.world_generation == 0
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fresh-spawn input has invalid identity or policy composition",
        ));
    }
    validate_applied_patch(&input.applied_patch)
}

fn fresh_spawn_lookup_key(
    authority_store_id: &str,
    input: &FreshSpawnReservationInputV1,
) -> DispatchPolicyCommitmentLookupKeyV1 {
    DispatchPolicyCommitmentLookupKeyV1 {
        authority_store_id: authority_store_id.into(),
        orchestration_session_id: input.spawn_request.orchestration_session_id.clone(),
        request_id: input.spawn_request.request_id.clone(),
        subject: DispatchPolicyCommitmentSubjectKeyV1::RetainedWorkerLaunch,
    }
}

fn lookup_index_key(
    lookup_key: &DispatchPolicyCommitmentLookupKeyV1,
) -> Result<String, DispatchPolicyCommitmentError> {
    Ok(sha256_hex(&encode_canonical(lookup_key)?))
}

fn validate_applied_patch(
    patch: &AppliedDispatchPolicyPatchIdentityV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    match patch {
        AppliedDispatchPolicyPatchIdentityV1::UnchangedParent => Ok(()),
        AppliedDispatchPolicyPatchIdentityV1::RestrictedWorldFs {
            patch_schema_version,
            canonical_patch,
            patch_hash,
        } => {
            if *patch_schema_version != 1 {
                return Err(DispatchPolicyCommitmentError::new(
                    "unsupported E2 patch schema",
                ));
            }
            let bytes = canonical_patch.resolve_inline()?;
            if bytes.is_empty() || patch_identity_hash(&bytes)? != *patch_hash {
                return Err(DispatchPolicyCommitmentError::new(
                    "restricted policy patch bytes or hash mismatch",
                ));
            }
            let _: Value = decode_canonical(&bytes)?;
            Ok(())
        }
    }
}

fn patch_identity_hash(bytes: &[u8]) -> Result<String, DispatchPolicyCommitmentError> {
    let patch: Value = decode_canonical(bytes)?;
    canonical_domain_hash(PATCH_HASH_DOMAIN, "patch", patch)
}

fn reservation_hash(
    reservation: &DispatchPolicyCommitmentReservationV1,
) -> Result<String, DispatchPolicyCommitmentError> {
    let mut value = serde_json::to_value(reservation)
        .map_err(|_| DispatchPolicyCommitmentError::new("serialize E2 reservation"))?;
    let Value::Object(fields) = &mut value else {
        return Err(DispatchPolicyCommitmentError::new(
            "invalid E2 reservation shape",
        ));
    };
    fields.remove("reservation_hash");
    canonical_domain_hash(RESERVATION_HASH_DOMAIN, "reservation", value)
}

fn commitment_hash(
    record: &DispatchPolicyCommitmentV1,
) -> Result<String, DispatchPolicyCommitmentError> {
    let mut value = serde_json::to_value(record)
        .map_err(|_| DispatchPolicyCommitmentError::new("serialize E2 commitment"))?;
    let Value::Object(fields) = &mut value else {
        return Err(DispatchPolicyCommitmentError::new(
            "invalid E2 commitment shape",
        ));
    };
    fields.remove("exact_linkage_hash");
    canonical_domain_hash(COMMITMENT_HASH_DOMAIN, "record", value)
}

fn canonical_domain_hash(
    domain: &str,
    field_name: &str,
    value: Value,
) -> Result<String, DispatchPolicyCommitmentError> {
    let mut object = serde_json::Map::new();
    object.insert("domain".into(), Value::String(domain.into()));
    object.insert(field_name.into(), value);
    Ok(sha256_hex(&encode_canonical(&Value::Object(object))?))
}

fn reservation_ref(
    reservation: &DispatchPolicyCommitmentReservationV1,
) -> DispatchPolicyCommitmentReservationRefV1 {
    DispatchPolicyCommitmentReservationRefV1 {
        authority_store_id: reservation.authority_store_id.clone(),
        reservation_id: reservation.reservation_id.clone(),
        reservation_hash: reservation.reservation_hash.clone(),
    }
}

fn commitment_ref(record: &DispatchPolicyCommitmentV1) -> DispatchPolicyCommitmentRefV1 {
    DispatchPolicyCommitmentRefV1 {
        authority_store_id: record.authority_store_id.clone(),
        commitment_id: record.commitment_id.clone(),
        exact_linkage_hash: record.exact_linkage_hash.clone(),
    }
}

fn validate_reservation(
    reservation: &DispatchPolicyCommitmentReservationV1,
    reference: &DispatchPolicyCommitmentReservationRefV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    if reservation.schema_version != 1
        || reservation.created_revision != 1
        || reservation.authority_store_id != reference.authority_store_id
        || reservation.reservation_id != reference.reservation_id
        || reservation.reservation_hash != reference.reservation_hash
        || reservation.reservation_hash != reservation_hash(reservation)?
        || reservation.lookup_key.authority_store_id != reservation.authority_store_id
        || reservation.lookup_key.request_id != reservation.request_id
        || reservation.lookup_key.orchestration_session_id != reservation.orchestration_session_id
        || reservation.validated_request_commitment.schema_version != 1
        || reservation.validated_request_commitment.domain != REQUEST_COMMITMENT_DOMAIN
        || reservation.proposed_commitment_id != reservation.proposed_worker_cap.commitment_id
        || reservation.proposed_worker_cap.policy_snapshot_ref != reservation.policy_snapshot_ref
        || reservation.proposed_worker_cap.policy_snapshot_hash != reservation.policy_snapshot_hash
        || reservation.proposed_worker_cap.policy_snapshot_revision
            != reservation.policy_snapshot_revision
        || reservation.proposed_worker_cap.retained_worker_cap_link
            != RetainedWorkerCapLinkV1::ThisCommitment
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fresh-spawn reservation identity or hash mismatch",
        ));
    }
    validate_digest("reservation hash", &reservation.reservation_hash)?;
    validate_digest(
        "validated request commitment",
        &reservation.validated_request_commitment.digest_hex,
    )?;
    validate_digest("policy snapshot hash", &reservation.policy_snapshot_hash)?;
    validate_applied_patch(&reservation.applied_patch)?;
    let bytes = reservation.policy_snapshot_bytes.resolve_inline()?;
    let snapshot: PolicySnapshotV3 = serde_json::from_slice(&bytes)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode reserved PolicySnapshotV3"))?;
    if serde_json::to_vec(&snapshot).ok().as_deref() != Some(bytes.as_slice())
        || sha256_hex(&bytes) != reservation.policy_snapshot_hash
    {
        return Err(DispatchPolicyCommitmentError::new(
            "reserved PolicySnapshotV3 byte identity mismatch",
        ));
    }
    Ok(())
}

fn validate_commitment(
    record: &DispatchPolicyCommitmentV1,
    reference: Option<&DispatchPolicyCommitmentRefV1>,
) -> Result<(), DispatchPolicyCommitmentError> {
    if record.schema_version != 1
        || record.created_revision != 1
        || record.application_revision == 0
        || record.status != PolicyCommitmentStatusV1::Immutable
        || record.exact_linkage_hash != commitment_hash(record)?
        || reference.is_some_and(|reference| commitment_ref(record) != *reference)
    {
        return Err(DispatchPolicyCommitmentError::new(
            "dispatch policy commitment identity or hash mismatch",
        ));
    }
    validate_digest("commitment exact linkage hash", &record.exact_linkage_hash)?;
    validate_digest("policy snapshot hash", &record.policy_snapshot_hash)?;
    let bytes = record.policy_snapshot_bytes.resolve_inline()?;
    let snapshot: PolicySnapshotV3 = serde_json::from_slice(&bytes)
        .map_err(|_| DispatchPolicyCommitmentError::new("decode committed PolicySnapshotV3"))?;
    if serde_json::to_vec(&snapshot).ok().as_deref() != Some(bytes.as_slice())
        || sha256_hex(&bytes) != record.policy_snapshot_hash
    {
        return Err(DispatchPolicyCommitmentError::new(
            "committed PolicySnapshotV3 byte identity mismatch",
        ));
    }
    validate_applied_patch(&record.applied_patch)
}

fn ensure_reservation_matches_input(
    reservation: &DispatchPolicyCommitmentReservationV1,
    input: &FreshSpawnReservationInputV1,
    request_commitment: &FreshSpawnValidatedRequestCommitmentV1,
) -> Result<(), DispatchPolicyCommitmentError> {
    if reservation.validated_request_commitment != *request_commitment
        || reservation.request_id != input.spawn_request.request_id
        || reservation.idempotency_key != input.spawn_request.idempotency_key
        || reservation.orchestration_session_id != input.spawn_request.orchestration_session_id
        || reservation.caller_participant_id != input.spawn_request.caller_participant_id
        || reservation.caller_backend_id != input.caller_backend_id
        || reservation.target_backend_id != input.spawn_request.target_backend_id
        || reservation.world_id != input.spawn_request.world_id
        || reservation.world_generation != input.spawn_request.world_generation
        || reservation.parent_policy_ref != input.parent_policy_ref
        || reservation.parent_policy_revision != input.parent_policy_revision
        || reservation.applied_patch != input.applied_patch
        || reservation.policy_snapshot_bytes.resolve_inline()? != input.policy_snapshot.bytes()
        || reservation.policy_snapshot_hash != input.policy_snapshot.hash()
        || reservation.policy_snapshot_revision != input.policy_snapshot.revision()
        || reservation.reason != input.reason
    {
        return Err(DispatchPolicyCommitmentError::new(
            "fresh-spawn retry changed authenticated material",
        ));
    }
    Ok(())
}

fn proof_from_reservation(
    reservation: &DispatchPolicyCommitmentReservationV1,
    permits_capability_narrowing: bool,
) -> AuthenticatedFreshSpawnReservationProofV1 {
    AuthenticatedFreshSpawnReservationProofV1 {
        reservation_ref: reservation_ref(reservation),
        request_id: reservation.request_id.clone(),
        idempotency_key: reservation.idempotency_key.clone(),
        orchestration_session_id: reservation.orchestration_session_id.clone(),
        caller_participant_id: reservation.caller_participant_id.clone(),
        target_backend_id: reservation.target_backend_id.clone(),
        world_id: reservation.world_id.clone(),
        world_generation: reservation.world_generation,
        parent_policy_ref: reservation.parent_policy_ref.clone(),
        parent_policy_revision: reservation.parent_policy_revision.clone(),
        retained_participant_id: reservation.retained_participant_id.clone(),
        bootstrap_run_id: reservation.bootstrap_run_id.clone(),
        narrowing_attestation: permits_capability_narrowing
            .then_some(OpaqueFreshSpawnNarrowingAttestationV1),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn hmac_fresh_spawn_request(key: &[u8], request: &[u8]) -> String {
    let domain = REQUEST_COMMITMENT_DOMAIN.as_bytes();
    let mut preimage = Vec::with_capacity(16 + domain.len() + request.len());
    preimage.extend_from_slice(&(domain.len() as u64).to_be_bytes());
    preimage.extend_from_slice(domain);
    preimage.extend_from_slice(&(request.len() as u64).to_be_bytes());
    preimage.extend_from_slice(request);
    lower_hex(&hmac_sha256(key, &preimage))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_LEN: usize = 64;
    let mut normalized = [0_u8; BLOCK_LEN];
    if key.len() > BLOCK_LEN {
        normalized[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        normalized[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36_u8; BLOCK_LEN];
    let mut outer_pad = [0x5c_u8; BLOCK_LEN];
    for index in 0..BLOCK_LEN {
        inner_pad[index] ^= normalized[index];
        outer_pad[index] ^= normalized[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let common = left.len().min(right.len());
    for index in 0..common {
        difference |= usize::from(left[index] ^ right[index]);
    }
    for byte in &left[common..] {
        difference |= usize::from(*byte);
    }
    for byte in &right[common..] {
        difference |= usize::from(*byte);
    }
    difference == 0
}

fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
pub(crate) fn test_authenticated_fresh_spawn_reservation_proof(
    plan: &RetainedWorkerAdmissionPlanV1,
    retained_participant_id: impl Into<String>,
    bootstrap_run_id: impl Into<String>,
    permits_capability_narrowing: bool,
) -> AuthenticatedFreshSpawnReservationProofV1 {
    AuthenticatedFreshSpawnReservationProofV1 {
        reservation_ref: DispatchPolicyCommitmentReservationRefV1 {
            authority_store_id: plan.exact_authority.authority_store_id.clone(),
            reservation_id: "dpr_00000000-0000-7000-8000-000000000001".into(),
            reservation_hash: "11".repeat(32),
        },
        request_id: plan.spawn_request.request_id.clone(),
        idempotency_key: plan.spawn_request.idempotency_key.clone(),
        orchestration_session_id: plan.spawn_request.orchestration_session_id.clone(),
        caller_participant_id: plan.spawn_request.caller_participant_id.clone(),
        target_backend_id: plan.spawn_request.target_backend_id.clone(),
        world_id: plan.spawn_request.world_id.clone(),
        world_generation: plan.spawn_request.world_generation,
        parent_policy_ref: plan.policy_and_admission_cap.current_policy_ref.clone(),
        parent_policy_revision: plan
            .policy_and_admission_cap
            .current_policy
            .policy_revision
            .clone(),
        retained_participant_id: retained_participant_id.into(),
        bootstrap_run_id: bootstrap_run_id.into(),
        narrowing_attestation: permits_capability_narrowing
            .then_some(OpaqueFreshSpawnNarrowingAttestationV1),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use super::*;
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AgentDescriptorV1, AgentExecutionScopeV1, AuthorityObjectKindV1, HostAttachCapabilitiesV1,
        HostAttachExecutionClientStartV1, HostAttachLaunchKnobsV1, HostAttachModePreferenceV1,
        HostSessionAuthorityPreconditionV1, HostSessionTransitionCallerKindV1,
        HostSessionTransitionCallerV1, HostSessionTransitionModeV1, PolicyObjectHashInputV1,
        RuntimeBackendKindV1, WorkspaceBindingV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentStateV2;
    use crate::execution::agent_runtime::host_session_authority::transition::{
        ApplyHostSessionTransitionRequestV1, ClaimHostSessionTransitionRequestV1,
        IssueHostSessionTransitionRequestV1, StartContractMaterialV1,
        TransitionApplicationOutcomeV1, TransitionClaimOutcomeV1, TransitionIssueOutcomeV1,
    };

    fn timestamp(value: &str) -> TimestampV1 {
        TimestampV1::parse(value).expect("test timestamp")
    }

    fn started_authority() -> (tempfile::TempDir, HostSessionAuthority) {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(
                    std::env::var_os("HOME").expect("tests require a private parent"),
                )
                .join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create test parent");
        let parent = tempfile::tempdir_in(safe_parent).expect("private tempdir");
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700))
            .expect("private test parent permissions");
        let home = parent.path().join("home");
        fs::create_dir(&home).expect("create authority home");
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700))
            .expect("private authority home permissions");
        let authority = HostSessionAuthority::open(&home).expect("open authority");
        let root = authority.bootstrap().expect("bootstrap authority");
        let binding = WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home,
            authority_store_id: root.authority_store_id,
        };
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: "e2-start-intent".into(),
            issuer_request_id: "dispatch-policy-commitment:transition-start".into(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: "e2-session".into(),
            shell_trace_session_id: "e2-trace".into(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: "e2-orchestrator".into(),
            target_participant_lease_token: b"e2-start-lease".to_vec(),
            run_id: "e2-start-run".into(),
            resulting_authoritative_lineage: vec!["e2-orchestrator".into()],
            workspace_binding: binding,
            world_binding: Some(WorldBindingV1 {
                world_id: "e2-world".into(),
                world_generation: 7,
            }),
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
                    policy_revision: "e2-policy".into(),
                    canonical_policy_snapshot_sha256: "aa".repeat(32),
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
        };
        let TransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_start_at(&request, timestamp("2026-08-31T12:00:00.000000000Z"), 300)
            .expect("issue start")
        else {
            panic!("start issuance must commit")
        };
        let claim_request = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: "e2-start-claim".into(),
            claimant_attempt_id: "e2-start-attempt".into(),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(
                &claim_request,
                timestamp("2026-08-31T12:01:00.000000000Z"),
                30,
            )
            .expect("claim start")
        else {
            panic!("start claim must commit")
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            panic!("start must remain claimed")
        };
        let application = ApplyHostSessionTransitionRequestV1 {
            intent_id: claimed.intent_id,
            issuer_request_id: claimed.issuer_request_id,
            payload_commitment: claimed.payload_commitment,
            expected_intent_revision: claimed.intent_revision,
            claim_id: claim_request.claim_id,
            expected_claim_revision: claim_revision,
        };
        assert!(matches!(
            authority
                .apply_start_at(&application, timestamp("2026-08-31T12:01:10.000000000Z"))
                .expect("apply start"),
            TransitionApplicationOutcomeV1::Applied(_)
        ));
        (parent, authority)
    }

    fn policy_snapshot(
        read_allow: &[&str],
        write_allow: &[&str],
        net_allowed: &[&str],
    ) -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: net_allowed.iter().map(|value| (*value).into()).collect(),
            world_fs: transport_api_types::PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: transport_api_types::PolicySnapshotWorldFsFailClosedV3 {
                    routing: false,
                },
                deny_enforcement: Some(WorldFsDenyEnforcementV3::Weak),
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: read_allow.iter().map(|value| (*value).into()).collect(),
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: read_allow.iter().map(|value| (*value).into()).collect(),
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: write_allow.iter().map(|value| (*value).into()).collect(),
                    deny_list: Vec::new(),
                },
            },
        }
        .canonicalize()
        .expect("canonical test policy")
    }

    fn policy_ref() -> AuthorityObjectRefV1 {
        AuthorityObjectRefV1 {
            ref_id: "ao_0123456789abcdef0123456789abcdef".into(),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "ab".repeat(32),
            },
        }
    }

    fn fresh_spawn_input(prompt: &str) -> FreshSpawnReservationInputV1 {
        let snapshot = policy_snapshot(&["src/lib.rs"], &["src/lib.rs"], &["api.example"]);
        let bytes = serde_json::to_vec(&snapshot).expect("snapshot bytes");
        let reference = policy_ref();
        FreshSpawnReservationInputV1 {
            spawn_request: CanonicalValidatedSpawnRequestV1 {
                schema_version: 1,
                request_id: "request-e2-spawn".into(),
                idempotency_key: "idempotency-e2-spawn".into(),
                orchestration_session_id: "e2-session".into(),
                caller_participant_id: "e2-orchestrator".into(),
                action: "spawn_world_worker".into(),
                mode: "retained".into(),
                target_backend_id: "cli:codex".into(),
                task_run_id: None,
                target_participant_id: None,
                world_id: "e2-world".into(),
                world_generation: 7,
                payload: super::super::retained_worker_runtime::CanonicalWorkerSpawnPayloadV1 {
                    prompt: prompt.into(),
                },
            },
            caller_backend_id: "cli:codex".into(),
            parent_policy_ref: reference.clone(),
            parent_policy_revision: "e2-policy".into(),
            applied_patch: AppliedDispatchPolicyPatchIdentityV1::UnchangedParent,
            policy_snapshot: validate_policy_snapshot_material(
                &snapshot,
                &bytes,
                &reference,
                &sha256_hex(&bytes),
                "e2-policy",
            )
            .expect("validated snapshot"),
            reason: None,
        }
    }

    fn publish_test_source_worker_cap(
        authority: &HostSessionAuthority,
    ) -> (String, PersistedDispatchPolicyCommitmentV1) {
        let reservation = reserve_fresh_spawn(authority, fresh_spawn_input("source worker"))
            .expect("reserve source worker");
        let participant_id = reservation.retained_participant_id.clone();
        let admission = test_admission_for_reservation(&reservation);
        let commitment = publish_fresh_spawn_commitment(authority, &reservation.proof, &admission)
            .expect("publish source worker cap");
        (participant_id, commitment)
    }

    fn test_admission_for_reservation(
        reservation: &FreshSpawnReservationOutcomeV1,
    ) -> RetainedWorkerAdmissionRecordV1 {
        RetainedWorkerAdmissionRecordV1 {
            schema_version: 1,
            authority_store_id: reservation.reservation_ref.authority_store_id.clone(),
            issuer_request_id: "request-e2-spawn".into(),
            canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentV1 {
                schema_version: 1,
                algorithm: super::super::retained_worker_runtime::RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256,
                key_id: "test-admission-key".into(),
                digest_hex: "33".repeat(32),
            },
            orchestration_session_id: "e2-session".into(),
            admission_authority_revision: 1,
            admission_authority_record_commitment:
                AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "44".repeat(32),
                },
            retained_participant_id: reservation.retained_participant_id.clone(),
            bootstrap_run_id: reservation.bootstrap_run_id.clone(),
            backend_id: "cli:codex".into(),
            protocol: "substrate.agent.session".into(),
            world_binding: WorldBindingV1 {
                world_id: "e2-world".into(),
                world_generation: 7,
            },
            current_policy_ref: policy_ref(),
            current_policy_revision: "e2-policy".into(),
            max_live_retained_workers: 8,
            state: RetainedWorkerAdmissionStateV1::PreTransportNonterminal {
                registration: RetainedWorkerAdmissionRegistrationV1 {
                    registration_id: "registration-e2-source".into(),
                    retained_worker_ref: AuthorityObjectRefV1 {
                        ref_id: "retained-worker-e2-source".into(),
                        object_kind: AuthorityObjectKindV1::RetainedWorker,
                        schema_version: 1,
                        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                            digest_hex: "55".repeat(32),
                        },
                    },
                },
            },
            record_revision: 1,
        }
    }

    fn fork_commitment_input(
        authority: &HostSessionAuthority,
        source_participant_id: &str,
        prompt: &str,
    ) -> ForkPolicyCommitmentInputV1 {
        let patch = fork_patch_for(source_participant_id);
        let snapshot = policy_snapshot(&["src/lib.rs"], &["src/lib.rs"], &["api.example"]);
        let source_cap =
            match resolve_retained_worker_cap(authority, "e2-session", source_participant_id)
                .expect("resolve source worker cap")
            {
                ResolvedPolicyCommitmentCompatibilityV1::Compatible { cap } => cap,
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState { .. } => {
                    panic!("source worker cap must be E2-compatible")
                }
            };
        let authenticated_policy = resolve_fork_policy_material(
            authority,
            ForkPolicyResolutionInputV1 {
                source_cap,
                current_parent_and_fork_patch: validated_material(snapshot),
                request_id: "request-e2-fork".into(),
                orchestration_session_id: "e2-session".into(),
                caller_participant_id: "e2-orchestrator".into(),
                caller_backend_id: "cli:codex".into(),
                target_backend_id: "cli:codex".into(),
                world_id: "e2-world".into(),
                world_generation: 7,
                source_participant_id: source_participant_id.into(),
                fork_patch: Some(patch.clone()),
            },
        )
        .expect("authenticate fork policy material");
        ForkPolicyCommitmentInputV1 {
            request_id: "request-e2-fork".into(),
            idempotency_key: "idempotency-e2-fork".into(),
            orchestration_session_id: "e2-session".into(),
            caller_participant_id: "e2-orchestrator".into(),
            caller_backend_id: "cli:codex".into(),
            target_backend_id: "cli:codex".into(),
            world_id: "e2-world".into(),
            world_generation: 7,
            source_participant_id: source_participant_id.into(),
            canonical_validated_dispatch_request: encode_canonical(&serde_json::json!({
                "payload": {"prompt": prompt},
                "request_id": "request-e2-fork"
            }))
            .expect("canonical fork request"),
            parent_policy_ref: policy_ref(),
            parent_policy_revision: "e2-policy".into(),
            applied_patch: applied_dispatch_policy_patch_identity(Some(&patch))
                .expect("fork patch identity"),
            authenticated_policy,
            reason: patch.reason,
        }
    }

    fn validated_material(snapshot: PolicySnapshotV3) -> ValidatedPolicySnapshotMaterialV1 {
        let bytes = serde_json::to_vec(&snapshot).expect("snapshot bytes");
        let reference = policy_ref();
        validate_policy_snapshot_material(
            &snapshot,
            &bytes,
            &reference,
            &sha256_hex(&bytes),
            "e2-policy",
        )
        .expect("validated snapshot")
    }

    fn fork_patch() -> DispatchPolicyNarrowingPatchV1 {
        fork_patch_for("worker-source")
    }

    fn fork_patch_for(source_participant_id: &str) -> DispatchPolicyNarrowingPatchV1 {
        DispatchPolicyNarrowingPatchV1 {
            schema_version: 1,
            request_id: "request-e2-fork".into(),
            orchestration_session_id: "e2-session".into(),
            caller_participant_id: "e2-orchestrator".into(),
            target_backend_id: "cli:codex".into(),
            target_world: WorldBindingRefV1 {
                world_id: "e2-world".into(),
                world_generation: 7,
            },
            applies_to: transport_api_types::DispatchCapabilitySubjectV1::RetainedWorkerFork {
                source_participant_id: source_participant_id.into(),
            },
            parent_policy_ref: transport_policy_ref(&policy_ref()),
            parent_policy_revision: "e2-policy".into(),
            restricted_policy_patch: transport_api_types::RestrictedPolicyPatchV1 {
                world_fs: Some(transport_api_types::RestrictedWorldFsPatchV1 {
                    read: Some(transport_api_types::RestrictedWorldFsDimensionPatchV1 {
                        allow_list: Some(vec!["src/lib.rs".into()]),
                        deny_list: None,
                    }),
                    ..Default::default()
                }),
            },
            reason: Some("fork policy narrowing".into()),
        }
    }

    fn fork_resolution_input(patch: DispatchPolicyNarrowingPatchV1) -> ForkPolicyResolutionInputV1 {
        ForkPolicyResolutionInputV1 {
            source_cap: DurablePolicyCommitmentDescriptorV1 {
                commitment_ref: DispatchPolicyCommitmentRefV1 {
                    authority_store_id: "store-e2".into(),
                    commitment_id: "dpc_00000000-0000-7000-8000-000000000001".into(),
                    exact_linkage_hash: "11".repeat(32),
                },
                orchestration_session_id: "e2-session".into(),
                retained_participant_id: "worker-source".into(),
                launch_parent_policy_ref: policy_ref(),
                launch_parent_policy_revision: "e2-policy".into(),
            },
            current_parent_and_fork_patch: validated_material(policy_snapshot(
                &["src/lib.rs"],
                &["src/lib.rs"],
                &["api.example"],
            )),
            request_id: "request-e2-fork".into(),
            orchestration_session_id: "e2-session".into(),
            caller_participant_id: "e2-orchestrator".into(),
            caller_backend_id: "cli:codex".into(),
            target_backend_id: "cli:codex".into(),
            world_id: "e2-world".into(),
            world_generation: 7,
            source_participant_id: "worker-source".into(),
            fork_patch: Some(patch),
        }
    }

    fn accepted_work_input(
        authority_store_id: &str,
        acceptance_record_id: &str,
        request_id: &str,
        work_identity: AcceptedWorldWorkIdentityV1,
        policy_snapshot: ValidatedPolicySnapshotMaterialV1,
        retained_worker_cap_ref: Option<DispatchPolicyCommitmentRefV1>,
    ) -> AcceptedWorkPolicyCommitmentInputV1 {
        let (
            runtime_submission_id,
            task_run_id,
            active_run_id,
            message_id,
            retained_participant_id,
        ) = match &work_identity {
            AcceptedWorldWorkIdentityV1::EphemeralTask { task_run_id } => (
                task_run_id.clone(),
                Some(task_run_id.clone()),
                None,
                None,
                None,
            ),
            AcceptedWorldWorkIdentityV1::RetainedTurn {
                active_run_id,
                message_id,
                target_participant_id,
            } => (
                format!("runtime-submission-{active_run_id}"),
                None,
                Some(active_run_id.clone()),
                Some(message_id.clone()),
                Some(target_participant_id.clone()),
            ),
        };
        let runtime_acceptance = RuntimeAcceptanceEvidenceV1 {
            acknowledgement_kind:
                super::super::state_store::RuntimeAcceptanceAcknowledgementKindV1::StartFrame,
            acceptance_record_id: acceptance_record_id.into(),
            stream_id: format!("stream-{request_id}"),
            frame_sequence: 1,
            runtime_submission_id: Some(runtime_submission_id.clone()),
            task_run_id,
            active_run_id,
            message_id,
            retained_participant_id,
            observed_at: Utc::now(),
        };
        let acceptance = WorldWorkAcceptanceRecordV1 {
            schema_version: 1,
            acceptance_record_id: acceptance_record_id.into(),
            request_id: request_id.into(),
            authority_store_id: authority_store_id.into(),
            authority_revision_observed: 7,
            orchestration_session_id: "e2-session".into(),
            caller_participant_id: "e2-orchestrator".into(),
            caller_backend_id: "cli:codex".into(),
            target_backend_id: "cli:codex".into(),
            world_id: "e2-world".into(),
            world_generation: 7,
            work_identity: work_identity.clone(),
            host_transition_correlation: None,
            current_policy_snapshot_ref: policy_snapshot.snapshot_ref().clone(),
            current_policy_snapshot_hash: policy_snapshot.hash().into(),
            current_policy_revision: policy_snapshot.revision().into(),
            runtime_acceptance: runtime_acceptance.clone(),
            accepted_at: Utc::now(),
            record_revision: 1,
        };
        let execution_claim = WorldWorkExecutionClaimV1 {
            schema_version: 1,
            authority_store_id: authority_store_id.into(),
            authority_revision_observed: 7,
            acceptance_record_id: acceptance_record_id.into(),
            acceptance_record_revision: 1,
            orchestration_session_id: "e2-session".into(),
            caller_participant_id: "e2-orchestrator".into(),
            caller_backend_id: "cli:codex".into(),
            target_backend_id: "cli:codex".into(),
            work_identity,
            world_id: "e2-world".into(),
            world_generation: 7,
            host_transition_correlation: None,
            stream_id: runtime_acceptance.stream_id,
            acceptance_frame_sequence: 1,
            runtime_submission_id,
            observer_instance_id: format!("observer-{request_id}"),
            observer_epoch: 1,
            claim_revision: 1,
            claimed_at: Utc::now(),
        };
        AcceptedWorkPolicyCommitmentInputV1 {
            idempotency_key: format!("idempotency-{request_id}"),
            acceptance,
            execution_claim,
            applied_patch: AppliedDispatchPolicyPatchIdentityV1::UnchangedParent,
            policy_snapshot,
            reason: None,
            retained_worker_cap_ref,
        }
    }

    #[test]
    fn accepted_ephemeral_and_retained_commitments_exact_link_b1_b21_and_cap() {
        let (_parent, authority) = started_authority();
        initialize_dispatch_policy_commitment_registry(&authority).expect("initialize registry");
        let storage =
            dispatch_policy_commitment_storage_for_authority(&authority).expect("dispatch storage");
        let authority_store_id = storage.authority_store_id().to_string();
        let snapshot = validated_material(policy_snapshot(
            &["src/lib.rs"],
            &["src/lib.rs"],
            &["api.example"],
        ));

        let ephemeral_input = accepted_work_input(
            &authority_store_id,
            "wwa_018f0f2e-7b4c-7aa1-8c22-123456789abc",
            "request-e2-ephemeral",
            AcceptedWorldWorkIdentityV1::EphemeralTask {
                task_run_id: "task-run-e2-ephemeral".into(),
            },
            snapshot.clone(),
            None,
        );
        let ephemeral = publish_accepted_work_commitment(&authority, ephemeral_input.clone())
            .expect("publish ephemeral B1/B2.1 commitment");
        let PolicyCommitmentAuthorityLinkV1::B1 {
            acceptance_record_id,
            acceptance_record_revision,
            runtime_acceptance,
        } = &ephemeral.record().authority_link
        else {
            panic!("ephemeral work must exact-link B1")
        };
        assert_eq!(
            acceptance_record_id,
            &ephemeral_input.acceptance.acceptance_record_id
        );
        assert_eq!(*acceptance_record_revision, 1);
        assert_eq!(
            runtime_acceptance,
            &ephemeral_input.acceptance.runtime_acceptance
        );
        let ephemeral_claim = ephemeral
            .record()
            .execution_claim_link
            .as_ref()
            .expect("ephemeral work exact-links B2.1");
        assert_eq!(
            ephemeral_claim
                .durable_claim_key
                .executions_by_acceptance_record_id_key,
            ephemeral_input.acceptance.acceptance_record_id
        );
        assert!(ephemeral.record().retained_worker_cap_link.is_none());
        let ephemeral_retry = publish_accepted_work_commitment(&authority, ephemeral_input.clone())
            .expect("exact ephemeral retry joins");
        assert_eq!(ephemeral_retry.commitment_ref(), ephemeral.commitment_ref());

        let mut changed_ephemeral_claim = ephemeral_input;
        changed_ephemeral_claim.execution_claim.observer_epoch = 2;
        assert!(publish_accepted_work_commitment(&authority, changed_ephemeral_claim).is_err());

        let (retained_participant_id, source_worker_cap) =
            publish_test_source_worker_cap(&authority);
        let source_cap_ref = source_worker_cap.commitment_ref().clone();
        let retained_input = accepted_work_input(
            &authority_store_id,
            "wwa_018f0f2e-7b4c-7aa1-8c22-123456789abd",
            "request-e2-retained-turn",
            AcceptedWorldWorkIdentityV1::RetainedTurn {
                active_run_id: "active-run-e2-retained".into(),
                message_id: "wwm_018f0f2e-7b4c-7aa1-8c22-123456789abe".into(),
                target_participant_id: retained_participant_id.clone(),
            },
            snapshot,
            Some(source_cap_ref.clone()),
        );
        let retained = publish_accepted_work_commitment(&authority, retained_input.clone())
            .expect("publish retained B1/B2.1/cap commitment");
        assert!(matches!(
            retained.record().retained_worker_cap_link.as_ref(),
            Some(RetainedWorkerCapLinkV1::Existing { cap_ref }) if cap_ref == &source_cap_ref
        ));
        assert_eq!(
            retained
                .record()
                .execution_claim_link
                .as_ref()
                .expect("retained turn exact-links B2.1")
                .durable_claim_key
                .executions_by_acceptance_record_id_key,
            retained_input.acceptance.acceptance_record_id
        );
        let retained_retry = publish_accepted_work_commitment(&authority, retained_input)
            .expect("exact retained retry joins");
        assert_eq!(retained_retry.commitment_ref(), retained.commitment_ref());
    }

    #[test]
    fn fresh_spawn_request_commitment_uses_the_exact_length_delimited_preimage() {
        let key = [0x0b; 20];
        let request =
            br#"{"request_id":"request-sensitive","payload":{"prompt":"do not persist"}}"#;
        let digest = hmac_fresh_spawn_request(&key, request);

        assert_eq!(
            digest,
            "8de6af3f6581567447bc0934dbef9ed0e30ad5b592fb8a636c3c6071673dab28"
        );
    }

    #[test]
    fn retained_turn_conjunction_never_broadens_an_immutable_worker_cap() {
        let current = policy_snapshot(&["."], &["."], &["*"]);
        let cap = policy_snapshot(&["src/lib.rs"], &["src/lib.rs"], &["api.example"]);

        let effective = conjoin_policy_snapshots(&current, &cap).expect("conjunction");

        assert_eq!(
            effective.world_fs.read.expect("read").allow_list,
            ["src/lib.rs"]
        );
        assert_eq!(effective.world_fs.write.allow_list, ["src/lib.rs"]);
        assert_eq!(effective.net_allowed, ["api.example"]);
    }

    #[test]
    fn retained_turn_conjunction_preserves_a_current_parent_narrowing() {
        let current = policy_snapshot(&["src/lib.rs"], &["src/lib.rs"], &["api.example"]);
        let cap = policy_snapshot(&["."], &["."], &["*"]);

        let effective = conjoin_policy_snapshots(&current, &cap).expect("conjunction");

        assert_eq!(
            effective.world_fs.read.expect("read").allow_list,
            ["src/lib.rs"]
        );
        assert_eq!(effective.world_fs.write.allow_list, ["src/lib.rs"]);
        assert_eq!(effective.net_allowed, ["api.example"]);
    }

    #[test]
    fn retained_turn_conjunction_allows_exact_file_but_not_siblings_or_outside() {
        let current = policy_snapshot(&["src"], &["src"], &["*"]);
        let cap = policy_snapshot(&["src/lib.rs"], &["src/lib.rs"], &["*"]);

        let effective = conjoin_policy_snapshots(&current, &cap).expect("conjunction");
        let read = effective.world_fs.read.expect("read");
        assert_eq!(read.allow_list, ["src/lib.rs"]);
        assert!(path_scope_contains(&read.allow_list[0], "src/lib.rs"));
        assert!(!path_scope_contains(&read.allow_list[0], "src/main.rs"));
        assert!(!path_scope_contains(&read.allow_list[0], "README.md"));
    }

    #[test]
    fn disjoint_world_fs_allows_conjoin_to_fail_closed_deny_all() {
        let current = policy_snapshot(&["src/lib.rs"], &["src/lib.rs"], &["*"]);
        let cap = policy_snapshot(&["docs/README.md"], &["docs/README.md"], &["*"]);

        let effective = conjoin_policy_snapshots(&current, &cap).expect("conjunction");
        let read = effective.world_fs.read.expect("read");
        assert_eq!(read.allow_list, ["."]);
        assert!(read.deny_list.contains(&"**".to_string()));
    }

    #[test]
    fn fork_patch_bindings_reject_wrong_backend_world_parent_or_source() {
        let valid = fork_resolution_input(fork_patch());
        validate_fork_patch_bindings(&valid).expect("exact fork patch binding");

        let mut wrong_backend = valid.clone();
        wrong_backend.target_backend_id = "cli:other".into();
        assert!(validate_fork_patch_bindings(&wrong_backend).is_err());

        let mut wrong_world = valid.clone();
        wrong_world.world_generation += 1;
        assert!(validate_fork_patch_bindings(&wrong_world).is_err());

        let mut wrong_source = valid.clone();
        wrong_source.source_participant_id = "worker-other".into();
        assert!(validate_fork_patch_bindings(&wrong_source).is_err());

        let mut wrong_parent = valid;
        wrong_parent.current_parent_and_fork_patch.revision = "changed-parent".into();
        assert!(validate_fork_patch_bindings(&wrong_parent).is_err());
    }

    #[test]
    fn fresh_spawn_reservation_cas_exact_joins_and_changed_payload_conflicts() {
        let (parent, authority) = started_authority();
        let first = reserve_fresh_spawn(&authority, fresh_spawn_input("sensitive-first-prompt"))
            .expect("first reservation");
        assert!(!first.joined);
        let restarted = HostSessionAuthority::open(&parent.path().join("home"))
            .expect("reopen authority after reservation");
        let retried = reserve_fresh_spawn(&restarted, fresh_spawn_input("sensitive-first-prompt"))
            .expect("exact retry after restart");
        assert!(retried.joined);
        assert_eq!(retried.reservation_ref, first.reservation_ref);
        assert_eq!(
            retried.retained_participant_id,
            first.retained_participant_id
        );
        assert_eq!(retried.bootstrap_run_id, first.bootstrap_run_id);
        assert_eq!(retried.proposed_commitment_id, first.proposed_commitment_id);
        assert!(
            reserve_fresh_spawn(&authority, fresh_spawn_input("changed-sensitive-prompt")).is_err()
        );

        let registry = fs::read(
            parent
                .path()
                .join("home/authority-v1/dispatch-policy-commitment-v1/registry-v1.json"),
        )
        .expect("registry bytes");
        let text = std::str::from_utf8(&registry).expect("registry utf8");
        assert!(!text.contains("sensitive-first-prompt"));
        assert!(!text.contains("changed-sensitive-prompt"));
    }

    #[test]
    fn fresh_spawn_restart_after_b3_before_e2_publication_exact_joins_once() {
        let (parent, authority) = started_authority();
        let reservation = reserve_fresh_spawn(&authority, fresh_spawn_input("post-b3 restart"))
            .expect("reserve before B3");
        let admission = test_admission_for_reservation(&reservation);

        let restarted_before_publication = HostSessionAuthority::open(&parent.path().join("home"))
            .expect("restart after B3 before E2 publication");
        let first = publish_fresh_spawn_commitment(
            &restarted_before_publication,
            &reservation.proof,
            &admission,
        )
        .expect("publish durable E2 after restart");

        let restarted_after_publication = HostSessionAuthority::open(&parent.path().join("home"))
            .expect("restart after committed publication");
        let joined = publish_fresh_spawn_commitment(
            &restarted_after_publication,
            &reservation.proof,
            &admission,
        )
        .expect("exact committed retry");
        assert_eq!(joined.commitment_ref(), first.commitment_ref());

        let mut changed_admission = admission;
        if let RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration } =
            &mut changed_admission.state
        {
            registration.registration_id = "registration-e2-changed".into();
        }
        assert!(publish_fresh_spawn_commitment(
            &restarted_after_publication,
            &reservation.proof,
            &changed_admission,
        )
        .is_err());
    }

    #[test]
    fn fork_first_writer_allocates_stable_identities_and_exact_retry_reuses_them() {
        let (parent, authority) = started_authority();
        let (source_participant_id, _source_cap) = publish_test_source_worker_cap(&authority);
        let input =
            fork_commitment_input(&authority, &source_participant_id, "fork-sensitive-prompt");

        let first = publish_fork_commitment(&authority, input).expect("publish fork");
        let first_child = first
            .fork_child_participant_id()
            .expect("fork child identity")
            .to_string();
        let first_bootstrap = first
            .fork_bootstrap_run_id()
            .expect("fork bootstrap identity")
            .to_string();
        assert!(first_child.starts_with("ash_"));
        assert!(Uuid::parse_str(&first_bootstrap).is_ok());
        assert_ne!(first_child, source_participant_id);
        let PolicyCommitmentAuthorityLinkV1::ForkDispatch {
            canonical_validated_dispatch_request_sha256,
        } = &first.record().authority_link
        else {
            panic!("fork commitment must use only the E2 ForkDispatch link");
        };
        let exact_request = serde_json::json!({
            "payload": {"prompt": "fork-sensitive-prompt"},
            "request_id": "request-e2-fork"
        });
        assert_eq!(
            canonical_validated_dispatch_request_sha256,
            &canonical_domain_hash(
                FORK_REQUEST_HASH_DOMAIN,
                "validated_dispatch_request",
                exact_request,
            )
            .expect("exact fork request hash")
        );
        let mut record_value = serde_json::to_value(first.record()).expect("fork record value");
        record_value
            .as_object_mut()
            .expect("fork record object")
            .remove("exact_linkage_hash");
        assert_eq!(
            first.record().exact_linkage_hash,
            canonical_domain_hash(COMMITMENT_HASH_DOMAIN, "record", record_value)
                .expect("exact commitment hash")
        );
        assert!(first.record().execution_claim_link.is_none());
        assert!(first.record().fresh_spawn_reservation_ref.is_none());
        assert_eq!(
            first.record().retained_worker_cap_link,
            Some(RetainedWorkerCapLinkV1::ThisCommitment)
        );

        let restarted = HostSessionAuthority::open(&parent.path().join("home"))
            .expect("reopen authority after fork publication");
        let retried = publish_fork_commitment(
            &restarted,
            fork_commitment_input(&restarted, &source_participant_id, "fork-sensitive-prompt"),
        )
        .expect("retry fork");
        assert_eq!(retried.commitment_ref(), first.commitment_ref());
        assert_eq!(
            retried.fork_child_participant_id(),
            Some(first_child.as_str())
        );
        assert_eq!(
            retried.fork_bootstrap_run_id(),
            Some(first_bootstrap.as_str())
        );
        assert!(matches!(
            resolve_retained_worker_cap(&restarted, "e2-session", &first_child)
                .expect("resolve child cap"),
            ResolvedPolicyCommitmentCompatibilityV1::Compatible { .. }
        ));

        let changed_request = fork_commitment_input(
            &restarted,
            &source_participant_id,
            "changed-fork-sensitive-prompt",
        );
        assert!(publish_fork_commitment(&restarted, changed_request).is_err());

        let mut changed_cap =
            match resolve_retained_worker_cap(&restarted, "e2-session", &source_participant_id)
                .expect("resolve cap for negative proof")
            {
                ResolvedPolicyCommitmentCompatibilityV1::Compatible { cap } => cap,
                ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState { .. } => {
                    panic!("source worker cap must be E2-compatible")
                }
            };
        changed_cap.commitment_ref.exact_linkage_hash = "66".repeat(32);
        assert!(resolve_fork_policy_material(
            &restarted,
            ForkPolicyResolutionInputV1 {
                source_cap: changed_cap,
                current_parent_and_fork_patch: validated_material(policy_snapshot(
                    &["src/lib.rs"],
                    &["src/lib.rs"],
                    &["api.example"],
                )),
                request_id: "request-e2-fork".into(),
                orchestration_session_id: "e2-session".into(),
                caller_participant_id: "e2-orchestrator".into(),
                caller_backend_id: "cli:codex".into(),
                target_backend_id: "cli:codex".into(),
                world_id: "e2-world".into(),
                world_generation: 7,
                source_participant_id: source_participant_id.clone(),
                fork_patch: Some(fork_patch_for(&source_participant_id)),
            },
        )
        .is_err());

        let registry = fs::read(
            parent
                .path()
                .join("home/authority-v1/dispatch-policy-commitment-v1/registry-v1.json"),
        )
        .expect("registry bytes");
        let text = std::str::from_utf8(&registry).expect("registry utf8");
        assert!(!text.contains("fork-sensitive-prompt"));
        assert!(!text.contains("changed-fork-sensitive-prompt"));
    }

    #[test]
    fn missing_and_hash_invalid_legacy_caps_fail_typed_without_parent_reconstruction() {
        let (_parent, authority) = started_authority();
        assert_eq!(
            resolve_retained_worker_cap(&authority, "e2-session", "legacy-worker")
                .expect("resolve missing legacy cap"),
            ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                retained_participant_id: "legacy-worker".into(),
                reason: PolicyCommitmentCompatibilityReasonV1::MissingCanonicalCapBytes,
            }
        );

        let (participant_id, _source_cap) = publish_test_source_worker_cap(&authority);
        test_corrupt_retained_worker_cap_hash(&authority, &participant_id)
            .expect("corrupt isolated cap index hash");
        assert_eq!(
            resolve_retained_worker_cap(&authority, "e2-session", &participant_id)
                .expect("resolve hash-invalid legacy cap"),
            ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                retained_participant_id: participant_id,
                reason: PolicyCommitmentCompatibilityReasonV1::CapHashMismatch,
            }
        );
    }

    #[test]
    fn staged_registry_temp_before_reservation_publication_recovers_on_retry() {
        let (_parent, authority) = started_authority();
        initialize_dispatch_policy_commitment_registry(&authority).expect("initialize registry");
        let storage =
            dispatch_policy_commitment_storage_for_authority(&authority).expect("dispatch storage");
        let crash = storage.transaction(|transaction| {
            transaction.stage_registry_replacement_for_test(
                "dispatch-policy-registry--22222222222222222222222222222222.tmp",
                b"incomplete reservation candidate",
            )?;
            Err::<(), _>(BootstrapError::dispatch_policy_commitment_crash())
        });
        assert!(crash.is_err());

        let recovered = reserve_fresh_spawn(&authority, fresh_spawn_input("restart prompt"))
            .expect("reservation retry after staged temp");
        assert!(!recovered.joined);
    }

    #[test]
    fn registry_initialization_is_private_exact_and_secret_non_disclosing() {
        let (parent, authority) = started_authority();
        initialize_dispatch_policy_commitment_registry(&authority).expect("initialize registry");
        initialize_dispatch_policy_commitment_registry(&authority).expect("exact retry");

        let root = parent
            .path()
            .join("home/authority-v1/dispatch-policy-commitment-v1");
        let registry = root.join("registry-v1.json");
        let keys = root.join("keys");
        let tmp = root.join("tmp");
        assert_eq!(
            fs::metadata(&root).expect("root metadata").mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&keys).expect("keys metadata").mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&tmp).expect("tmp metadata").mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&registry).expect("registry metadata").mode() & 0o777,
            0o600
        );
        let registry_bytes = fs::read(&registry).expect("registry bytes");
        let registry_text = std::str::from_utf8(&registry_bytes).expect("registry utf8");
        assert!(!registry_text.contains("secret_key"));
        assert!(!registry_text.contains("prompt"));
        assert!(!registry_text.contains("payload"));
        let key_entries = fs::read_dir(keys)
            .expect("key entries")
            .collect::<Result<Vec<_>, _>>()
            .expect("key listing");
        assert_eq!(key_entries.len(), 1);
        assert_eq!(
            key_entries[0].metadata().expect("key metadata").mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn orphan_key_after_key_rename_before_registry_is_reconciled() {
        let (parent, authority) = started_authority();
        let storage =
            dispatch_policy_commitment_storage_for_authority(&authority).expect("dispatch storage");
        let envelope = DispatchPolicyCommitmentKeyEnvelopeV1 {
            schema_version: 1,
            authority_store_id: storage.authority_store_id().into(),
            key_id: "dpk_33333333333333333333333333333333".into(),
            created_at: timestamp("2026-08-31T13:00:00.000000000Z"),
            algorithm: FreshSpawnRequestCommitmentAlgorithmV1::HmacSha256,
            secret_key: [0x33; 32],
        };
        let envelope_bytes = encode_canonical(&envelope).expect("canonical orphan envelope");
        storage
            .transaction(|transaction| {
                transaction.stage_key_temp(
                    "dispatch-policy-key--33333333333333333333333333333333.tmp",
                    &envelope_bytes,
                )?;
                transaction.publish_staged_key_no_replace(
                    "dispatch-policy-key--33333333333333333333333333333333.tmp",
                    "dpk_33333333333333333333333333333333.key",
                )
            })
            .expect("publish orphan key");

        initialize_dispatch_policy_commitment_registry(&authority)
            .expect("recover orphan key and initialize");

        let keys = fs::read_dir(
            parent
                .path()
                .join("home/authority-v1/dispatch-policy-commitment-v1/keys"),
        )
        .expect("key directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("key listing");
        assert_eq!(keys.len(), 1);
        assert_ne!(
            keys[0].file_name().to_string_lossy(),
            "dpk_33333333333333333333333333333333.key"
        );
    }

    #[test]
    fn recognized_registry_temp_is_reconciled_after_restart() {
        let (parent, authority) = started_authority();
        initialize_dispatch_policy_commitment_registry(&authority).expect("initialize registry");
        let temp = parent.path().join(
            "home/authority-v1/dispatch-policy-commitment-v1/tmp/dispatch-policy-registry--11111111111111111111111111111111.tmp",
        );
        fs::write(&temp, b"incomplete").expect("stage recognized temp");
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o600))
            .expect("private temp permissions");

        initialize_dispatch_policy_commitment_registry(&authority).expect("restart reconciliation");

        assert!(!temp.exists());
    }

    #[test]
    fn unrecognized_temp_and_registry_symlink_fail_closed() {
        let (parent, authority) = started_authority();
        initialize_dispatch_policy_commitment_registry(&authority).expect("initialize registry");
        let root = parent
            .path()
            .join("home/authority-v1/dispatch-policy-commitment-v1");
        let unrecognized = root.join("tmp/not-recognized.tmp");
        fs::write(&unrecognized, b"unsafe").expect("write unrecognized temp");
        fs::set_permissions(&unrecognized, fs::Permissions::from_mode(0o600))
            .expect("private unsafe temp permissions");
        assert!(initialize_dispatch_policy_commitment_registry(&authority).is_err());

        fs::remove_file(&unrecognized).expect("remove isolated test temp");
        let registry = root.join("registry-v1.json");
        let original = parent.path().join("registry-original.json");
        fs::rename(&registry, &original).expect("move isolated registry");
        std::os::unix::fs::symlink(&original, &registry).expect("install registry symlink");
        assert!(initialize_dispatch_policy_commitment_registry(&authority).is_err());
    }

    #[test]
    fn replacement_authority_root_is_rejected() {
        let (parent, authority) = started_authority();
        let home = parent.path().join("home");
        let moved = parent.path().join("moved-home");
        fs::rename(&home, &moved).expect("move isolated authority root");
        fs::create_dir(&home).expect("create replacement authority root");
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700))
            .expect("private replacement root permissions");

        assert!(initialize_dispatch_policy_commitment_registry(&authority).is_err());
    }
}
