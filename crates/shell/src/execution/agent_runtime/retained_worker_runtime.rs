//! Component-only retained-worker authority registration.
//!
//! This module deliberately has no production ingress caller. It owns immutable
//! retained-object construction while `HostSessionAuthority` owns durable
//! reservation and authority mutation.

#![allow(
    dead_code,
    reason = "R0 is a component proof and deliberately has no production ingress caller"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rand::RngCore;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use super::host_session_authority::canonical_json;
#[cfg(test)]
use super::host_session_authority::facade::RetainedReservationCrashPointV1;
use super::host_session_authority::facade::{
    ReservedRetainedWorkerRegistrationV1, ResolvedCurrentAuthorityV1,
    RetainedWorkerAuthorityPreconditionV1,
};
use super::host_session_authority::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AgentExecutionScopeV1,
    AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, AuthorityObjectRefV1,
    DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1, HostAttachContractV1,
    PolicyObjectHashInputV1, ResumeHandleHashInputV1, RetainedWorkerObjectHashInputV1, TimestampV1,
    WorldBindingV1,
};
use super::host_session_authority::store_schema::{
    DurableSessionAuthorityV1, HostSessionTransitionIntentStateV2,
    RetainedWorkerAuthorityRegistrationRequestStateV1, RetainedWorkerAuthorityRegistrationV1,
    SessionNamespaceRecordV1, StateRootV2, VersionedStateRoot,
};
use super::host_session_authority::validation::ValidatedCanonicalV1;
use super::host_session_authority::HostSessionAuthority;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerRegistrationPlanV1 {
    pub(crate) registration_request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) expected_authority: RetainedWorkerAuthorityPreconditionV1,
    pub(crate) retained_participant_id: String,
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) internal_uaa_session_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerRegistrationResultV1 {
    pub(crate) registration_id: String,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) retained_participant_id: String,
    pub(crate) retained_worker_ref: super::host_session_authority::schema::AuthorityObjectRefV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) authority_record_commitment_after:
        super::host_session_authority::schema::AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedRetainedTargetV1 {
    pub(crate) registration: RetainedWorkerAuthorityRegistrationV1,
    pub(crate) current_authority_revision: u64,
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) resume_handle: ResumeHandleHashInputV1,
    pub(crate) retained_worker: RetainedWorkerObjectHashInputV1,
    pub(crate) current_policy: PolicyObjectHashInputV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetainedWorkerAdmissionCommitmentAlgorithmV1 {
    HmacSha256,
}

impl Serialize for RetainedWorkerAdmissionCommitmentAlgorithmV1 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("hmac_sha256")
    }
}

impl<'de> Deserialize<'de> for RetainedWorkerAdmissionCommitmentAlgorithmV1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        if value == "hmac_sha256" {
            Ok(Self::HmacSha256)
        } else {
            Err(serde::de::Error::custom(
                "unsupported retained admission commitment algorithm",
            ))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAdmissionCommitmentV1 {
    pub(crate) schema_version: u32,
    pub(crate) algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1,
    pub(crate) key_id: String,
    pub(crate) digest_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAdmissionRegistrationV1 {
    pub(crate) registration_id: String,
    pub(crate) retained_worker_ref: AuthorityObjectRefV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RetainedWorkerAdmissionStateV1 {
    SlotReserved {
        slot_sequence: u64,
        reserved_at: TimestampV1,
    },
    AuthorityRegistrationHead {
        authority_revision_expected: u64,
        authority_record_commitment_expected: AuthorityObjectCommitmentV1,
        head_acquired_at: TimestampV1,
    },
    PreTransportNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
    },
    TransportClaimedNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        transport_claim_id: String,
        claimed_at: TimestampV1,
    },
    Routable {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: String,
        registered_frame_sequence: u64,
        registered_event_id: String,
        registered_event_sequence: u64,
        registered_at: TimestampV1,
    },
    InterruptedNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: Option<String>,
        last_frame_sequence: Option<u64>,
        interrupted_at: TimestampV1,
    },
    Terminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: String,
        terminal_frame_sequence: u64,
        terminal_event_id: String,
        terminal_event_sequence: u64,
        exit_code: i32,
        terminal_at: TimestampV1,
    },
    RejectedBeforeRegistration {
        reason: String,
        rejected_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAdmissionRecordV1 {
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
    pub(crate) state: RetainedWorkerAdmissionStateV1,
    pub(crate) record_revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerAdmissionKeyHeaderV1 {
    schema_version: u32,
    authority_store_id: String,
    key_id: String,
    created_at: TimestampV1,
    algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerAdmissionKeyEnvelopeV1 {
    schema_version: u32,
    authority_store_id: String,
    key_id: String,
    created_at: TimestampV1,
    algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1,
    secret_key: [u8; 32],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerAdmissionRecordLocatorV1 {
    orchestration_session_id: String,
    retained_participant_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerAdmissionRegistryV1 {
    schema_version: u32,
    authority_store_id: String,
    commitment_key: RetainedWorkerAdmissionKeyHeaderV1,
    records_by_session: BTreeMap<String, BTreeMap<String, RetainedWorkerAdmissionRecordV1>>,
    issuer_request_index: BTreeMap<String, RetainedWorkerAdmissionRecordLocatorV1>,
    next_slot_sequence_by_session: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAdmissionKeyIdentityV1 {
    pub(crate) authority_store_id: String,
    pub(crate) key_id: String,
    pub(crate) algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalWorkerSpawnPayloadV1 {
    pub(crate) prompt: String,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalValidatedSpawnRequestV1 {
    pub(crate) schema_version: u32,
    pub(crate) request_id: String,
    pub(crate) idempotency_key: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) action: String,
    pub(crate) mode: String,
    pub(crate) target_backend_id: String,
    pub(crate) task_run_id: Option<String>,
    pub(crate) target_participant_id: Option<String>,
    pub(crate) world_id: String,
    pub(crate) world_generation: u64,
    pub(crate) payload: CanonicalWorkerSpawnPayloadV1,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalExactCurrentAuthorityV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authority: DurableSessionAuthorityV1,
    pub(crate) caller_participant_id: String,
    pub(crate) caller_role: String,
    pub(crate) caller_descriptor_ref: AuthorityObjectRefV1,
    pub(crate) caller_descriptor: AgentDescriptorV1,
    pub(crate) host_attach_contract: HostAttachContractV1,
    pub(crate) current_policy: PolicyObjectHashInputV1,
}

impl CanonicalExactCurrentAuthorityV1 {
    pub(crate) fn from_resolved(resolved: &ResolvedCurrentAuthorityV1) -> Self {
        Self {
            schema_version: 1,
            authority_store_id: resolved.observation.authority_store_id.clone(),
            orchestration_session_id: resolved.observation.orchestration_session_id.clone(),
            authority_revision: resolved.observation.authority_revision,
            authority_record_commitment: resolved.observation.authority_record_commitment.clone(),
            authoritative_lineage_commitment: resolved
                .observation
                .authoritative_lineage_commitment
                .clone(),
            authority: resolved.authority.clone(),
            caller_participant_id: resolved.caller.participant_id.clone(),
            caller_role: "orchestrator".into(),
            caller_descriptor_ref: resolved.caller.descriptor_ref.clone(),
            caller_descriptor: resolved.caller.descriptor.clone(),
            host_attach_contract: resolved.host_attach_contract.clone(),
            current_policy: resolved.current_policy.clone(),
        }
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalDescriptorAndRuntimePlanV1 {
    pub(crate) schema_version: u32,
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) runtime_role: String,
    pub(crate) internal_uaa_session_id_domain: String,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalPolicyAndAdmissionCapV1 {
    pub(crate) schema_version: u32,
    pub(crate) current_policy_ref: AuthorityObjectRefV1,
    pub(crate) current_policy: PolicyObjectHashInputV1,
    pub(crate) dispatch_enabled: bool,
    pub(crate) allowed_backends: Vec<String>,
    pub(crate) allowed_actions: Vec<String>,
    pub(crate) allowed_modes: Vec<String>,
    pub(crate) same_session_only: bool,
    pub(crate) same_world_binding_only: bool,
    pub(crate) allow_capability_narrowing: bool,
    pub(crate) max_live_retained_workers: u64,
    pub(crate) max_concurrent_ephemeral: u64,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAdmissionPlanV1 {
    pub(crate) issuer_request_id: String,
    pub(crate) spawn_request: CanonicalValidatedSpawnRequestV1,
    pub(crate) exact_authority: CanonicalExactCurrentAuthorityV1,
    pub(crate) descriptor_and_runtime_plan: CanonicalDescriptorAndRuntimePlanV1,
    pub(crate) policy_and_admission_cap: CanonicalPolicyAndAdmissionCapV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAdmissionSlotV1 {
    pub(crate) record: RetainedWorkerAdmissionRecordV1,
    pub(crate) joined: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAdmissionRegistrationOutcomeV1 {
    pub(crate) record: RetainedWorkerAdmissionRecordV1,
    pub(crate) joined: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerTransportClaimV1 {
    pub(crate) record: RetainedWorkerAdmissionRecordV1,
    pub(crate) newly_claimed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AdmissionHeadPreparationV1 {
    Ready(RetainedWorkerAdmissionRecordV1),
    Complete(RetainedWorkerAdmissionRecordV1),
    Queued,
}

struct AdmissionTransportClaimInputV1 {
    claimed_at: TimestampV1,
    claim_entropy: [u8; 16],
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionInitializationCrashPointV1 {
    BeforeKeyPublication,
    AfterKeyPublication,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionReservationCrashPointV1 {
    AfterSlotReserved,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionRegistrationCrashPointV1 {
    WhileRegistrationHead,
    AfterR0BeforeAdmissionAdvance,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RetainedObjectPublicationCrashPointV1 {
    Descriptor,
    ResumeHandle,
    RetainedWorker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerRuntimeError(String);

impl fmt::Display for RetainedWorkerRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RetainedWorkerRuntimeError {}

#[derive(Debug, Default)]
pub(crate) struct RetainedWorkerRuntime;

impl RetainedWorkerRuntime {
    pub(crate) fn initialize_admission_registry(
        &self,
        authority: &HostSessionAuthority,
    ) -> Result<RetainedWorkerAdmissionKeyIdentityV1, RetainedWorkerRuntimeError> {
        let mut key_entropy = [0_u8; 16];
        let mut publication_nonce = [0_u8; 16];
        let mut secret_key = [0_u8; 32];
        let mut random = rand::rngs::OsRng;
        random.fill_bytes(&mut key_entropy);
        random.fill_bytes(&mut publication_nonce);
        random.fill_bytes(&mut secret_key);
        let created_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| RetainedWorkerRuntimeError("create admission key timestamp".into()))?;
        self.initialize_admission_registry_with(
            authority,
            created_at,
            key_entropy,
            publication_nonce,
            secret_key,
            None,
        )
    }

    #[cfg(test)]
    fn initialize_admission_registry_at(
        &self,
        authority: &HostSessionAuthority,
        created_at: TimestampV1,
        key_entropy: [u8; 16],
        publication_nonce: [u8; 16],
        secret_key: [u8; 32],
        crash_point: Option<AdmissionInitializationCrashPointV1>,
    ) -> Result<RetainedWorkerAdmissionKeyIdentityV1, RetainedWorkerRuntimeError> {
        self.initialize_admission_registry_with(
            authority,
            created_at,
            key_entropy,
            publication_nonce,
            secret_key,
            crash_point,
        )
    }

    fn initialize_admission_registry_with(
        &self,
        authority: &HostSessionAuthority,
        created_at: TimestampV1,
        key_entropy: [u8; 16],
        publication_nonce: [u8; 16],
        secret_key: [u8; 32],
        #[cfg(test)] crash_point: Option<AdmissionInitializationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<RetainedWorkerAdmissionKeyIdentityV1, RetainedWorkerRuntimeError> {
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let result = storage.transaction(|transaction| {
            if let Some(registry_bytes) = transaction.read_registry()? {
                let registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
                    decode_canonical(&registry_bytes, "decode canonical admission registry"),
                    &mut semantic_failure,
                )?;
                let keys = transaction.read_keys()?;
                let identity = retain_semantic_error(
                    validate_committed_admission_key(&authority_store_id, &registry, &keys),
                    &mut semantic_failure,
                )?;
                retain_semantic_error(
                    validate_admission_registry(&registry, transaction.authority_root()),
                    &mut semantic_failure,
                )?;
                return Ok(identity);
            }

            for (orphan_name, _) in transaction.read_keys()? {
                transaction.remove_key(&orphan_name)?;
            }

            let key_id = format!("adk_{}", lower_hex(&key_entropy));
            let key_name = format!("{key_id}.key");
            let nonce = lower_hex(&publication_nonce);
            let key_temp_name = format!("admission-key--{nonce}.tmp");
            let registry_temp_name = format!("admission-registry--{nonce}.tmp");
            let envelope = RetainedWorkerAdmissionKeyEnvelopeV1 {
                schema_version: 1,
                authority_store_id: authority_store_id.clone(),
                key_id: key_id.clone(),
                created_at: created_at.clone(),
                algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256,
                secret_key,
            };
            let envelope_bytes = retain_semantic_error(
                encode_canonical(&envelope, "encode admission key envelope"),
                &mut semantic_failure,
            )?;
            transaction.stage_key_temp(&key_temp_name, &envelope_bytes)?;
            #[cfg(test)]
            if crash_point == Some(AdmissionInitializationCrashPointV1::BeforeKeyPublication) {
                return Err(
                    super::host_session_authority::store::BootstrapError::retained_admission_crash(
                    ),
                );
            }
            transaction.publish_staged_key_no_replace(&key_temp_name, &key_name)?;
            #[cfg(test)]
            if crash_point == Some(AdmissionInitializationCrashPointV1::AfterKeyPublication) {
                return Err(
                    super::host_session_authority::store::BootstrapError::retained_admission_crash(
                    ),
                );
            }
            let header = RetainedWorkerAdmissionKeyHeaderV1 {
                schema_version: 1,
                authority_store_id: authority_store_id.clone(),
                key_id: key_id.clone(),
                created_at,
                algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256,
            };
            let registry = RetainedWorkerAdmissionRegistryV1 {
                schema_version: 1,
                authority_store_id: authority_store_id.clone(),
                commitment_key: header,
                records_by_session: BTreeMap::new(),
                issuer_request_index: BTreeMap::new(),
                next_slot_sequence_by_session: BTreeMap::new(),
            };
            let registry_bytes = retain_semantic_error(
                encode_canonical(&registry, "encode admission registry"),
                &mut semantic_failure,
            )?;
            transaction.publish_registry_no_replace(&registry_temp_name, &registry_bytes)?;
            let identity = retain_semantic_error(
                validate_committed_admission_key(
                    &authority_store_id,
                    &registry,
                    &transaction.read_keys()?,
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_registry(&registry, transaction.authority_root()),
                &mut semantic_failure,
            )?;
            Ok(identity)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| {
            #[cfg(test)]
            if error.to_string() == "injected retained admission initialization crash" {
                if crash_point == Some(AdmissionInitializationCrashPointV1::BeforeKeyPublication) {
                    return RetainedWorkerRuntimeError(
                        "injected crash before admission key publication".into(),
                    );
                }
                if crash_point == Some(AdmissionInitializationCrashPointV1::AfterKeyPublication) {
                    return RetainedWorkerRuntimeError(
                        "injected crash after admission key publication".into(),
                    );
                }
            }
            RetainedWorkerRuntimeError(error.to_string())
        })
    }

    pub(crate) fn reserve_admission_slot(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
    ) -> Result<RetainedWorkerAdmissionSlotV1, RetainedWorkerRuntimeError> {
        let mut participant_entropy = [0_u8; 16];
        let mut bootstrap_run_entropy = [0_u8; 16];
        let mut publication_nonce = [0_u8; 16];
        let mut random = rand::rngs::OsRng;
        random.fill_bytes(&mut participant_entropy);
        random.fill_bytes(&mut bootstrap_run_entropy);
        random.fill_bytes(&mut publication_nonce);
        let reserved_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| RetainedWorkerRuntimeError("create admission slot timestamp".into()))?;
        self.reserve_admission_slot_with(
            authority,
            plan,
            reserved_at,
            participant_entropy,
            bootstrap_run_entropy,
            publication_nonce,
            None,
        )
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    fn reserve_admission_slot_at(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        reserved_at: TimestampV1,
        participant_entropy: [u8; 16],
        bootstrap_run_entropy: [u8; 16],
        publication_nonce: [u8; 16],
        crash_point: Option<AdmissionReservationCrashPointV1>,
    ) -> Result<RetainedWorkerAdmissionSlotV1, RetainedWorkerRuntimeError> {
        self.reserve_admission_slot_with(
            authority,
            plan,
            reserved_at,
            participant_entropy,
            bootstrap_run_entropy,
            publication_nonce,
            crash_point,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reserve_admission_slot_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        reserved_at: TimestampV1,
        participant_entropy: [u8; 16],
        bootstrap_run_entropy: [u8; 16],
        publication_nonce: [u8; 16],
        #[cfg(test)] crash_point: Option<AdmissionReservationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<RetainedWorkerAdmissionSlotV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        self.initialize_admission_registry(authority)?;
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let result = storage.transaction(|transaction| {
            let registry_bytes = transaction.read_registry()?.ok_or_else(
                super::host_session_authority::store::BootstrapError::retained_admission_semantic,
            )?;
            let mut registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
                decode_canonical(&registry_bytes, "decode canonical admission registry"),
                &mut semantic_failure,
            )?;
            let (_, envelope) = retain_semantic_error(
                load_committed_admission_key(
                    &authority_store_id,
                    &registry,
                    &transaction.read_keys()?,
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_registry(&registry, transaction.authority_root()),
                &mut semantic_failure,
            )?;
            if let Some(locator) = registry.issuer_request_index.get(&plan.issuer_request_id) {
                let record = registry
                    .records_by_session
                    .get(&locator.orchestration_session_id)
                    .and_then(|records| records.get(&locator.retained_participant_id))
                    .ok_or_else(
                        super::host_session_authority::store::BootstrapError::retained_admission_semantic,
                    )?;
                let VersionedStateRoot::V2(root) = transaction.authority_root() else {
                    return Err(
                        super::host_session_authority::store::BootstrapError::retained_admission_semantic(),
                    );
                };
                retain_semantic_error(
                    validate_supplied_admission_authority(
                        root,
                        &current_plan.exact_authority,
                        &plan.exact_authority,
                        record,
                    ),
                    &mut semantic_failure,
                )?;
            } else if plan.exact_authority != current_plan.exact_authority {
                retain_semantic_error(
                    Err(RetainedWorkerRuntimeError(
                        "canonical admission authority proof is not the exact bound read".into(),
                    )),
                    &mut semantic_failure,
                )?;
            }
            let policy = retain_semantic_error(
                crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                    std::path::Path::new(
                        &current_plan
                            .exact_authority
                            .authority
                            .workspace_binding
                            .workspace_root
                            .physical_path,
                    ),
                    &authority.bootstrap_home(),
                )
                .map_err(|_| {
                    RetainedWorkerRuntimeError(
                        "resolve exact accepted-home admission policy".into(),
                    )
                }),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_policy_against_resolved(
                    &current_plan.policy_and_admission_cap,
                    &policy,
                ),
                &mut semantic_failure,
            )?;
            let slot = retain_semantic_error(
                reserve_slot_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    &current_plan,
                    &envelope.secret_key,
                    reserved_at,
                    participant_entropy,
                    bootstrap_run_entropy,
                ),
                &mut semantic_failure,
            )?;
            if slot.joined {
                return Ok(slot);
            }
            let published = retain_semantic_error(
                encode_canonical(&registry, "encode admission registry"),
                &mut semantic_failure,
            )?;
            let temp_name = format!("admission-registry--{}.tmp", lower_hex(&publication_nonce));
            transaction.replace_registry(&temp_name, &published)?;
            #[cfg(test)]
            if crash_point == Some(AdmissionReservationCrashPointV1::AfterSlotReserved) {
                return Err(
                    super::host_session_authority::store::BootstrapError::retained_admission_crash(
                    ),
                );
            }
            Ok(slot)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| {
            #[cfg(test)]
            if error.to_string() == "injected retained admission initialization crash"
                && crash_point == Some(AdmissionReservationCrashPointV1::AfterSlotReserved)
            {
                return RetainedWorkerRuntimeError(
                    "injected crash after admission slot reservation".into(),
                );
            }
            RetainedWorkerRuntimeError(error.to_string())
        })
    }

    pub(crate) fn read_admission_record(
        &self,
        authority: &HostSessionAuthority,
        orchestration_session_id: &str,
        retained_participant_id: &str,
    ) -> Result<Option<RetainedWorkerAdmissionRecordV1>, RetainedWorkerRuntimeError> {
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let result = storage.transaction(|transaction| {
            let Some(registry_bytes) = transaction.read_registry()? else {
                return Ok(None);
            };
            let registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
                decode_canonical(&registry_bytes, "decode canonical admission registry"),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                load_committed_admission_key(
                    &authority_store_id,
                    &registry,
                    &transaction.read_keys()?,
                )
                .map(drop),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_registry(&registry, transaction.authority_root()),
                &mut semantic_failure,
            )?;
            Ok(registry
                .records_by_session
                .get(orchestration_session_id)
                .and_then(|records| records.get(retained_participant_id))
                .cloned())
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    pub(crate) fn register_admitted_worker(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
    ) -> Result<RetainedWorkerAdmissionRegistrationOutcomeV1, RetainedWorkerRuntimeError> {
        self.register_admitted_worker_with(authority, plan, None)
    }

    #[cfg(test)]
    fn register_admitted_worker_at(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        crash_point: Option<AdmissionRegistrationCrashPointV1>,
    ) -> Result<RetainedWorkerAdmissionRegistrationOutcomeV1, RetainedWorkerRuntimeError> {
        self.register_admitted_worker_with(authority, plan, crash_point)
    }

    fn register_admitted_worker_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        #[cfg(test)] crash_point: Option<AdmissionRegistrationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<RetainedWorkerAdmissionRegistrationOutcomeV1, RetainedWorkerRuntimeError> {
        let slot = self.reserve_admission_slot(authority, plan)?;
        let head_acquired_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| RetainedWorkerRuntimeError("create registration head timestamp".into()))?;
        let mut publication_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut publication_nonce);
        let head = self.prepare_admission_registration_head(
            authority,
            plan,
            &slot.record.retained_participant_id,
            head_acquired_at,
            publication_nonce,
        )?;
        if let AdmissionHeadPreparationV1::Complete(record) = head {
            return Ok(RetainedWorkerAdmissionRegistrationOutcomeV1 {
                record,
                joined: true,
            });
        }
        let AdmissionHeadPreparationV1::Ready(record) = head else {
            return Err(RetainedWorkerRuntimeError(
                "a lower-sequence admission slot owns the registration head".into(),
            ));
        };
        #[cfg(test)]
        if crash_point == Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead) {
            return Err(RetainedWorkerRuntimeError(
                "injected crash while admission registration head".into(),
            ));
        }
        let registration_plan = admission_registration_plan(&record, plan)?;
        let registration = self.register_retained_target(authority, &registration_plan)?;
        #[cfg(test)]
        if crash_point == Some(AdmissionRegistrationCrashPointV1::AfterR0BeforeAdmissionAdvance) {
            return Err(RetainedWorkerRuntimeError(
                "injected crash after R0 before admission advancement".into(),
            ));
        }
        let mut reconcile_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut reconcile_nonce);
        let record = self.advance_registration_head_after_r0(
            authority,
            plan,
            &record.retained_participant_id,
            &registration,
            reconcile_nonce,
        )?;
        Ok(RetainedWorkerAdmissionRegistrationOutcomeV1 {
            record,
            joined: false,
        })
    }

    fn prepare_admission_registration_head(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        head_acquired_at: TimestampV1,
        publication_nonce: [u8; 16],
    ) -> Result<AdmissionHeadPreparationV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let result = storage.transaction(|transaction| {
            let registry_bytes = transaction.read_registry()?.ok_or_else(
                super::host_session_authority::store::BootstrapError::retained_admission_semantic,
            )?;
            let mut registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
                decode_canonical(&registry_bytes, "decode canonical admission registry"),
                &mut semantic_failure,
            )?;
            let (_, envelope) = retain_semantic_error(
                load_committed_admission_key(
                    &authority_store_id,
                    &registry,
                    &transaction.read_keys()?,
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_registry(&registry, transaction.authority_root()),
                &mut semantic_failure,
            )?;
            let (preparation, changed) = retain_semantic_error(
                prepare_registration_head_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    &current_plan,
                    &plan.exact_authority,
                    retained_participant_id,
                    &envelope.secret_key,
                    head_acquired_at,
                ),
                &mut semantic_failure,
            )?;
            if changed {
                let bytes = retain_semantic_error(
                    encode_canonical(&registry, "encode admission registry"),
                    &mut semantic_failure,
                )?;
                let temp_name =
                    format!("admission-registry--{}.tmp", lower_hex(&publication_nonce));
                transaction.replace_registry(&temp_name, &bytes)?;
            }
            Ok(preparation)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    fn advance_registration_head_after_r0(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        registration: &RetainedWorkerRegistrationResultV1,
        publication_nonce: [u8; 16],
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let result = storage.transaction(|transaction| {
            let registry_bytes = transaction.read_registry()?.ok_or_else(
                super::host_session_authority::store::BootstrapError::retained_admission_semantic,
            )?;
            let mut registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
                decode_canonical(&registry_bytes, "decode canonical admission registry"),
                &mut semantic_failure,
            )?;
            let (_, envelope) = retain_semantic_error(
                load_committed_admission_key(
                    &authority_store_id,
                    &registry,
                    &transaction.read_keys()?,
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_registry(&registry, transaction.authority_root()),
                &mut semantic_failure,
            )?;
            let (record, changed) = retain_semantic_error(
                advance_registration_head_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    &current_plan,
                    &plan.exact_authority,
                    retained_participant_id,
                    &envelope.secret_key,
                    registration,
                ),
                &mut semantic_failure,
            )?;
            if changed {
                let bytes = retain_semantic_error(
                    encode_canonical(&registry, "encode admission registry"),
                    &mut semantic_failure,
                )?;
                let temp_name =
                    format!("admission-registry--{}.tmp", lower_hex(&publication_nonce));
                transaction.replace_registry(&temp_name, &bytes)?;
            }
            Ok(record)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    pub(crate) fn claim_admission_transport(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
    ) -> Result<RetainedWorkerTransportClaimV1, RetainedWorkerRuntimeError> {
        let claimed_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| RetainedWorkerRuntimeError("create transport claim timestamp".into()))?;
        let mut claim_entropy = [0_u8; 16];
        let mut publication_nonce = [0_u8; 16];
        let mut random = rand::rngs::OsRng;
        random.fill_bytes(&mut claim_entropy);
        random.fill_bytes(&mut publication_nonce);
        self.claim_admission_transport_with(
            authority,
            plan,
            retained_participant_id,
            claimed_at,
            claim_entropy,
            publication_nonce,
        )
    }

    fn claim_admission_transport_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        claimed_at: TimestampV1,
        claim_entropy: [u8; 16],
        publication_nonce: [u8; 16],
    ) -> Result<RetainedWorkerTransportClaimV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let result = storage.transaction(|transaction| {
            let registry_bytes = transaction.read_registry()?.ok_or_else(
                super::host_session_authority::store::BootstrapError::retained_admission_semantic,
            )?;
            let mut registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
                decode_canonical(&registry_bytes, "decode canonical admission registry"),
                &mut semantic_failure,
            )?;
            let (_, envelope) = retain_semantic_error(
                load_committed_admission_key(
                    &authority_store_id,
                    &registry,
                    &transaction.read_keys()?,
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_registry(&registry, transaction.authority_root()),
                &mut semantic_failure,
            )?;
            let claim = retain_semantic_error(
                claim_transport_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    &current_plan,
                    &plan.exact_authority,
                    retained_participant_id,
                    &envelope.secret_key,
                    &AdmissionTransportClaimInputV1 {
                        claimed_at,
                        claim_entropy,
                    },
                ),
                &mut semantic_failure,
            )?;
            if claim.newly_claimed {
                let bytes = retain_semantic_error(
                    encode_canonical(&registry, "encode admission registry"),
                    &mut semantic_failure,
                )?;
                let temp_name =
                    format!("admission-registry--{}.tmp", lower_hex(&publication_nonce));
                transaction.replace_registry(&temp_name, &bytes)?;
            }
            Ok(claim)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    pub(crate) fn reserve_registration(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, RetainedWorkerRuntimeError> {
        let (descriptor_bytes, resume_handle_bytes) = Self::immutable_inputs(plan)?;
        authority
            .reserve_retained_worker_registration(
                &plan.registration_request_id,
                &plan.orchestration_session_id,
                &plan.expected_authority,
                &plan.retained_participant_id,
                descriptor_bytes,
                resume_handle_bytes,
                move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                    Self::retained_worker_bytes(
                        plan,
                        descriptor_ref,
                        resume_handle_ref,
                        policy_ref,
                        world_binding,
                    )
                },
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    #[cfg(test)]
    fn reserve_registration_at(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
        registered_at: TimestampV1,
        crash_point: Option<RetainedReservationCrashPointV1>,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, RetainedWorkerRuntimeError> {
        let (descriptor_bytes, resume_handle_bytes) = Self::immutable_inputs(plan)?;
        authority
            .reserve_retained_worker_registration_at(
                &plan.registration_request_id,
                &plan.orchestration_session_id,
                &plan.expected_authority,
                &plan.retained_participant_id,
                descriptor_bytes,
                resume_handle_bytes,
                registered_at,
                crash_point,
                move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                    Self::retained_worker_bytes(
                        plan,
                        descriptor_ref,
                        resume_handle_ref,
                        policy_ref,
                        world_binding,
                    )
                },
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    fn immutable_inputs(
        plan: &RetainedWorkerRegistrationPlanV1,
    ) -> Result<(Vec<u8>, Vec<u8>), RetainedWorkerRuntimeError> {
        let descriptor = AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: plan.descriptor.clone(),
        };
        descriptor
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if descriptor.descriptor.execution_scope != AgentExecutionScopeV1::World {
            return Err(RetainedWorkerRuntimeError(
                "retained descriptor must be world-scoped".into(),
            ));
        }
        let descriptor_bytes = canonical_json::to_vec(&descriptor)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_handle = ResumeHandleHashInputV1 {
            schema_version: 1,
            orchestration_session_id: plan.orchestration_session_id.clone(),
            participant_id: plan.retained_participant_id.clone(),
            backend_id: plan.descriptor.backend_id.clone(),
            protocol: plan.descriptor.protocol.clone(),
            internal_uaa_session_id: plan.internal_uaa_session_id.clone(),
        };
        resume_handle
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_handle_bytes = canonical_json::to_vec(&resume_handle)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        Ok((descriptor_bytes, resume_handle_bytes))
    }

    fn retained_worker_bytes(
        plan: &RetainedWorkerRegistrationPlanV1,
        descriptor_ref: &super::host_session_authority::schema::AuthorityObjectRefV1,
        resume_handle_ref: &super::host_session_authority::schema::AuthorityObjectRefV1,
        policy_ref: &super::host_session_authority::schema::AuthorityObjectRefV1,
        world_binding: &super::host_session_authority::schema::WorldBindingV1,
    ) -> Result<Vec<u8>, &'static str> {
        let worker = RetainedWorkerObjectHashInputV1 {
            schema_version: 1,
            orchestration_session_id: plan.orchestration_session_id.clone(),
            participant_id: plan.retained_participant_id.clone(),
            world_binding: world_binding.clone(),
            descriptor_ref: descriptor_ref.clone(),
            resume_handle_ref: resume_handle_ref.clone(),
            policy_ref: policy_ref.clone(),
        };
        worker
            .validate()
            .map_err(|_| "validate retained-worker object")?;
        canonical_json::to_vec(&worker).map_err(|_| "encode retained-worker object")
    }

    pub(crate) fn publish_reserved_object_graph(
        &self,
        authority: &HostSessionAuthority,
        reserved: &ReservedRetainedWorkerRegistrationV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        self.publish_reserved_object_graph_with(authority, reserved, |_| Ok(()))
    }

    fn publish_reserved_object_graph_with(
        &self,
        authority: &HostSessionAuthority,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        mut after_publication: impl FnMut(usize) -> Result<(), RetainedWorkerRuntimeError>,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        for (index, (reference, bytes)) in [
            (
                &reserved.descriptor_ref,
                reserved.descriptor_bytes.as_slice(),
            ),
            (
                &reserved.resume_handle_ref,
                reserved.resume_handle_bytes.as_slice(),
            ),
            (
                &reserved.retained_worker_ref,
                reserved.retained_worker_bytes.as_slice(),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            authority
                .publish_reserved_retained_object(reserved, reference, bytes)
                .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
            after_publication(index)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn publish_reserved_object_graph_with_crash_point(
        &self,
        authority: &HostSessionAuthority,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        crash_point: RetainedObjectPublicationCrashPointV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let stop_after = match crash_point {
            RetainedObjectPublicationCrashPointV1::Descriptor => 0,
            RetainedObjectPublicationCrashPointV1::ResumeHandle => 1,
            RetainedObjectPublicationCrashPointV1::RetainedWorker => 2,
        };
        self.publish_reserved_object_graph_with(authority, reserved, |index| {
            if index == stop_after {
                Err(RetainedWorkerRuntimeError(
                    "injected crash after retained object publication".into(),
                ))
            } else {
                Ok(())
            }
        })
    }

    pub(crate) fn register_retained_target(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
    ) -> Result<RetainedWorkerRegistrationResultV1, RetainedWorkerRuntimeError> {
        self.register_retained_target_with(authority, plan, |_| Ok(()))
    }

    fn register_retained_target_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerRegistrationPlanV1,
        after_reservation: impl FnOnce(
            &ReservedRetainedWorkerRegistrationV1,
        ) -> Result<(), RetainedWorkerRuntimeError>,
    ) -> Result<RetainedWorkerRegistrationResultV1, RetainedWorkerRuntimeError> {
        let mut reserved = self.reserve_registration(authority, plan)?;
        after_reservation(&reserved)?;
        if matches!(
            reserved.request.state,
            RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
        ) {
            if let Err(publication_error) = self.publish_reserved_object_graph(authority, &reserved)
            {
                let refreshed = self.reserve_registration(authority, plan)?;
                if matches!(
                    refreshed.request.state,
                    RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
                ) {
                    return Err(publication_error);
                }
                reserved = refreshed;
            }
        }
        let applied = authority
            .apply_reserved_retained_worker_registration(&reserved)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        Ok(RetainedWorkerRegistrationResultV1 {
            registration_id: applied.registration.registration_id.clone(),
            authority_store_id: plan.expected_authority.authority_store_id.clone(),
            orchestration_session_id: applied.registration.orchestration_session_id.clone(),
            retained_participant_id: applied.registration.retained_participant_id.clone(),
            retained_worker_ref: applied.registration.retained_worker_ref.clone(),
            authority_revision_after: applied.registration.authority_revision_after,
            authority_record_commitment_after: applied
                .registration
                .authority_record_commitment_after,
        })
    }

    pub(crate) fn resolve_retained_target(
        &self,
        authority: &HostSessionAuthority,
        target: &RetainedWorkerRegistrationResultV1,
    ) -> Result<ResolvedRetainedTargetV1, RetainedWorkerRuntimeError> {
        let current = authority
            .resolve_current_exact(&target.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if current.observation.authority_store_id != target.authority_store_id
            || current.observation.orchestration_session_id != target.orchestration_session_id
            || current.observation.authority_revision < target.authority_revision_after
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target authority store, session, or revision is inexact".into(),
            ));
        }
        let root = authority
            .read_a12a_root()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let SessionNamespaceRecordV1::Authority(root_authority) = root
            .session_namespace_map
            .get(&target.orchestration_session_id)
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("retained target session is absent".into())
            })?
        else {
            return Err(RetainedWorkerRuntimeError(
                "retained target session has no durable authority".into(),
            ));
        };
        if root.root_revision != current.observation.root_revision
            || root.authority_store_id != target.authority_store_id
            || root_authority.as_ref() != &current.authority
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target authority snapshot changed during resolution".into(),
            ));
        }
        let registrations = root
            .retained_worker_registration_journal
            .iter()
            .filter(|(registration_id, registration)| {
                *registration_id == &target.registration_id
                    && registration.registration_id == target.registration_id
            })
            .collect::<Vec<_>>();
        let [(_, registration)] = registrations.as_slice() else {
            return Err(RetainedWorkerRuntimeError(
                "retained target has no unique registration proof".into(),
            ));
        };
        let requests = root
            .retained_worker_registration_request_index
            .values()
            .filter(|request| request.registration_id == target.registration_id)
            .collect::<Vec<_>>();
        let [request] = requests.as_slice() else {
            return Err(RetainedWorkerRuntimeError(
                "retained target has no unique registration request".into(),
            ));
        };
        if registration.orchestration_session_id != target.orchestration_session_id
            || registration.retained_participant_id != target.retained_participant_id
            || registration.retained_worker_ref != target.retained_worker_ref
            || registration.authority_revision_after != target.authority_revision_after
            || registration.authority_record_commitment_after
                != target.authority_record_commitment_after
            || request.issuer_request_id != registration.issuer_request_id
            || request.registration_id != registration.registration_id
            || !matches!(
                &request.state,
                RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                    authority_revision_after,
                    authority_record_commitment_after,
                } if *authority_revision_after == registration.authority_revision_after
                    && authority_record_commitment_after
                        == &registration.authority_record_commitment_after
            )
            || current
                .authority
                .authoritative_participant_lineage
                .iter()
                .filter(|participant| *participant == &target.retained_participant_id)
                .count()
                != 1
            || current
                .authority
                .retained_worker_refs
                .iter()
                .filter(|reference| *reference == &target.retained_worker_ref)
                .count()
                != 1
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target request, proof, or authority membership is inexact".into(),
            ));
        }
        let highest_proof_revision = root
            .retained_worker_registration_journal
            .values()
            .filter(|candidate| {
                candidate.orchestration_session_id == target.orchestration_session_id
            })
            .map(|candidate| candidate.authority_revision_after)
            .max()
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("retained target authority proof is absent".into())
            })?;
        if highest_proof_revision != current.authority.authority_revision {
            return Err(RetainedWorkerRuntimeError(
                "retained target proof revision is behind current authority".into(),
            ));
        }

        let descriptor_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.descriptor_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.resume_handle_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let worker_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.retained_worker_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let policy_bytes = authority
            .read_authority_object_v2_at(root.root_revision, &registration.current_policy_ref)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let descriptor: AgentDescriptorHashInputV1 = canonical_json::from_slice(&descriptor_bytes)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let resume_handle: ResumeHandleHashInputV1 = canonical_json::from_slice(&resume_bytes)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let retained_worker: RetainedWorkerObjectHashInputV1 =
            canonical_json::from_slice(&worker_bytes)
                .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let current_policy: PolicyObjectHashInputV1 = canonical_json::from_slice(&policy_bytes)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        descriptor
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        resume_handle
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        retained_worker
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        current_policy
            .validate()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if descriptor.descriptor.execution_scope != AgentExecutionScopeV1::World
            || resume_handle.orchestration_session_id != target.orchestration_session_id
            || resume_handle.participant_id != target.retained_participant_id
            || resume_handle.backend_id != descriptor.descriptor.backend_id
            || resume_handle.protocol != descriptor.descriptor.protocol
            || retained_worker.orchestration_session_id != target.orchestration_session_id
            || retained_worker.participant_id != target.retained_participant_id
            || retained_worker.world_binding != registration.world_binding
            || retained_worker.descriptor_ref != registration.descriptor_ref
            || retained_worker.resume_handle_ref != registration.resume_handle_ref
            || retained_worker.policy_ref != registration.current_policy_ref
            || current.authority.world_binding.as_ref() != Some(&retained_worker.world_binding)
            || current.authority.current_policy_ref.as_ref() != Some(&retained_worker.policy_ref)
        {
            return Err(RetainedWorkerRuntimeError(
                "retained target immutable object graph is inexact".into(),
            ));
        }
        Ok(ResolvedRetainedTargetV1 {
            registration: (*registration).clone(),
            current_authority_revision: current.authority.authority_revision,
            descriptor: descriptor.descriptor,
            resume_handle,
            retained_worker,
            current_policy,
        })
    }
}

fn validate_admission_policy_against_resolved(
    expected: &CanonicalPolicyAndAdmissionCapV1,
    policy: &substrate_broker::Policy,
) -> Result<(), RetainedWorkerRuntimeError> {
    let actual = policy.world_dispatch_policy();
    if expected.dispatch_enabled != actual.enabled
        || expected.allowed_backends != actual.allowed_backends
        || expected.allowed_actions != actual.allowed_actions
        || expected.allowed_modes != actual.allowed_modes
        || expected.same_session_only != actual.same_session_only
        || expected.same_world_binding_only != actual.same_world_binding_only
        || expected.allow_capability_narrowing != actual.allow_capability_narrowing
        || expected.max_live_retained_workers != u64::from(actual.max_live_retained_workers)
        || expected.max_concurrent_ephemeral != u64::from(actual.max_concurrent_ephemeral)
    {
        return Err(RetainedWorkerRuntimeError(
            "admission policy and cap do not match the accepted-home policy".into(),
        ));
    }
    Ok(())
}

fn encode_canonical<T: Serialize>(
    value: &T,
    reason: &'static str,
) -> Result<Vec<u8>, RetainedWorkerRuntimeError> {
    canonical_json::to_vec(value).map_err(|_| RetainedWorkerRuntimeError(reason.into()))
}

fn decode_canonical<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    reason: &'static str,
) -> Result<T, RetainedWorkerRuntimeError> {
    let value =
        canonical_json::from_slice(bytes).map_err(|_| RetainedWorkerRuntimeError(reason.into()))?;
    if encode_canonical(&value, reason)? != bytes {
        return Err(RetainedWorkerRuntimeError(reason.into()));
    }
    Ok(value)
}

fn retain_semantic_error<T>(
    result: Result<T, RetainedWorkerRuntimeError>,
    failure: &mut Option<RetainedWorkerRuntimeError>,
) -> Result<T, super::host_session_authority::store::BootstrapError> {
    result.map_err(|error| {
        *failure = Some(error);
        super::host_session_authority::store::BootstrapError::retained_admission_semantic()
    })
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn validate_committed_admission_key(
    authority_store_id: &str,
    registry: &RetainedWorkerAdmissionRegistryV1,
    keys: &[(String, Vec<u8>)],
) -> Result<RetainedWorkerAdmissionKeyIdentityV1, RetainedWorkerRuntimeError> {
    load_committed_admission_key(authority_store_id, registry, keys)
        .map(|(identity, _envelope)| identity)
}

fn load_committed_admission_key(
    authority_store_id: &str,
    registry: &RetainedWorkerAdmissionRegistryV1,
    keys: &[(String, Vec<u8>)],
) -> Result<
    (
        RetainedWorkerAdmissionKeyIdentityV1,
        RetainedWorkerAdmissionKeyEnvelopeV1,
    ),
    RetainedWorkerRuntimeError,
> {
    let header = &registry.commitment_key;
    if registry.schema_version != 1
        || registry.authority_store_id != authority_store_id
        || header.schema_version != 1
        || header.authority_store_id != authority_store_id
        || header.algorithm != RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256
        || !valid_key_id(&header.key_id)
    {
        return Err(RetainedWorkerRuntimeError(
            "committed admission registry header is invalid".into(),
        ));
    }
    let expected_name = format!("{}.key", header.key_id);
    let [(key_name, key_bytes)] = keys else {
        return Err(RetainedWorkerRuntimeError(
            "committed admission registry has no unique key".into(),
        ));
    };
    if key_name != &expected_name {
        return Err(RetainedWorkerRuntimeError(
            "committed admission registry key identity is wrong".into(),
        ));
    }
    let envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
        decode_canonical(key_bytes, "committed admission key envelope is malformed")?;
    if envelope.schema_version != 1
        || envelope.authority_store_id != authority_store_id
        || envelope.key_id != header.key_id
        || envelope.created_at != header.created_at
        || envelope.algorithm != header.algorithm
    {
        return Err(RetainedWorkerRuntimeError(
            "committed admission key envelope identity is inexact".into(),
        ));
    }
    Ok((
        RetainedWorkerAdmissionKeyIdentityV1 {
            authority_store_id: authority_store_id.to_owned(),
            key_id: header.key_id.clone(),
            algorithm: header.algorithm,
        },
        envelope,
    ))
}

fn valid_key_id(value: &str) -> bool {
    value.strip_prefix("adk_").is_some_and(|hex| {
        hex.len() == 32
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct CanonicalDescriptorAndRuntimeFingerprintV1<'a> {
    schema_version: u32,
    descriptor: &'a AgentDescriptorV1,
    runtime_role: &'a str,
    internal_uaa_session_id_domain: &'a str,
    retained_participant_id: &'a str,
    bootstrap_run_id: &'a str,
    internal_uaa_session_id: String,
}

fn validate_supplied_admission_authority(
    root: &StateRootV2,
    current_exact_authority: &CanonicalExactCurrentAuthorityV1,
    supplied_exact_authority: &CanonicalExactCurrentAuthorityV1,
    record: &RetainedWorkerAdmissionRecordV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    let admission_authority = reconstruct_exact_authority_at_revision(
        root,
        current_exact_authority,
        record.admission_authority_revision,
    )?;
    if supplied_exact_authority != current_exact_authority
        && supplied_exact_authority != &admission_authority
    {
        return Err(RetainedWorkerRuntimeError(
            "canonical admission authority proof is not the exact bound read or admission ancestor"
                .into(),
        ));
    }
    Ok(())
}

fn prepare_registration_head_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    plan: &RetainedWorkerAdmissionPlanV1,
    supplied_exact_authority: &CanonicalExactCurrentAuthorityV1,
    retained_participant_id: &str,
    secret_key: &[u8; 32],
    head_acquired_at: TimestampV1,
) -> Result<(AdmissionHeadPreparationV1, bool), RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, plan)?;
    let VersionedStateRoot::V2(root) = authority_root else {
        return Err(RetainedWorkerRuntimeError(
            "registration head requires strict V2 authority".into(),
        ));
    };
    let mut record = registry
        .records_by_session
        .get(&plan.spawn_request.orchestration_session_id)
        .and_then(|records| records.get(retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot is absent".into()))?;
    validate_supplied_admission_authority(
        root,
        &plan.exact_authority,
        supplied_exact_authority,
        &record,
    )?;
    verify_admission_record_fingerprint(root, plan, &record, secret_key)?;
    match &record.state {
        RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::Routable { .. }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::Terminal { .. } => {
            return Ok((AdmissionHeadPreparationV1::Complete(record), false));
        }
        RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => {
            return Err(RetainedWorkerRuntimeError(
                "admission slot was rejected before registration".into(),
            ));
        }
        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. } => {
            if let Some(registration) = admission_registration_from_hsa(root, &record)? {
                record.state =
                    RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration };
                record.record_revision =
                    record.record_revision.checked_add(1).ok_or_else(|| {
                        RetainedWorkerRuntimeError("admission record revision overflow".into())
                    })?;
                registry
                    .records_by_session
                    .get_mut(&record.orchestration_session_id)
                    .ok_or_else(|| {
                        RetainedWorkerRuntimeError("admission session bucket is absent".into())
                    })?
                    .insert(record.retained_participant_id.clone(), record.clone());
                validate_admission_registry(registry, authority_root)?;
                return Ok((AdmissionHeadPreparationV1::Complete(record), true));
            }
            return Ok((AdmissionHeadPreparationV1::Ready(record), false));
        }
        RetainedWorkerAdmissionStateV1::SlotReserved { .. } => {}
    }
    if registry
        .records_by_session
        .get(&record.orchestration_session_id)
        .into_iter()
        .flat_map(BTreeMap::values)
        .any(|candidate| {
            matches!(
                candidate.state,
                RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
            )
        })
    {
        return Ok((AdmissionHeadPreparationV1::Queued, false));
    }
    let lowest = registry
        .records_by_session
        .get(&record.orchestration_session_id)
        .into_iter()
        .flat_map(BTreeMap::values)
        .filter_map(|candidate| match candidate.state {
            RetainedWorkerAdmissionStateV1::SlotReserved { slot_sequence, .. } => {
                Some((slot_sequence, candidate.retained_participant_id.as_str()))
            }
            _ => None,
        })
        .min();
    if lowest.is_none_or(|(_, participant_id)| participant_id != retained_participant_id) {
        return Ok((AdmissionHeadPreparationV1::Queued, false));
    }
    let reconstructed = reconstruct_exact_authority_at_revision(
        root,
        &plan.exact_authority,
        record.admission_authority_revision,
    )?;
    if reconstructed.authority_record_commitment != record.admission_authority_record_commitment {
        return Err(RetainedWorkerRuntimeError(
            "admission-time authority is not an exact R0-only ancestor".into(),
        ));
    }
    record.state = RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
        authority_revision_expected: plan.exact_authority.authority_revision,
        authority_record_commitment_expected: plan
            .exact_authority
            .authority_record_commitment
            .clone(),
        head_acquired_at,
    };
    record.record_revision = record
        .record_revision
        .checked_add(1)
        .ok_or_else(|| RetainedWorkerRuntimeError("admission record revision overflow".into()))?;
    registry
        .records_by_session
        .get_mut(&record.orchestration_session_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("admission session bucket is absent".into()))?
        .insert(record.retained_participant_id.clone(), record.clone());
    validate_admission_registry(registry, authority_root)?;
    Ok((AdmissionHeadPreparationV1::Ready(record), true))
}

fn verify_admission_record_fingerprint(
    root: &StateRootV2,
    current_plan: &RetainedWorkerAdmissionPlanV1,
    record: &RetainedWorkerAdmissionRecordV1,
    secret_key: &[u8; 32],
) -> Result<(), RetainedWorkerRuntimeError> {
    let mut fingerprint_plan = current_plan.clone();
    fingerprint_plan.exact_authority = reconstruct_exact_authority_at_revision(
        root,
        &current_plan.exact_authority,
        record.admission_authority_revision,
    )?;
    let fingerprint = canonical_spawn_fingerprint(
        &record.canonical_spawn_fingerprint.key_id,
        secret_key,
        &fingerprint_plan,
        &record.retained_participant_id,
        &record.bootstrap_run_id,
    )?;
    if fingerprint != record.canonical_spawn_fingerprint
        || fingerprint_plan.exact_authority.authority_store_id != record.authority_store_id
        || fingerprint_plan.exact_authority.authority_record_commitment
            != record.admission_authority_record_commitment
    {
        return Err(RetainedWorkerRuntimeError(
            "admission record fingerprint or authority ancestry is inexact".into(),
        ));
    }
    Ok(())
}

fn advance_registration_head_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    plan: &RetainedWorkerAdmissionPlanV1,
    supplied_exact_authority: &CanonicalExactCurrentAuthorityV1,
    retained_participant_id: &str,
    secret_key: &[u8; 32],
    registration_result: &RetainedWorkerRegistrationResultV1,
) -> Result<(RetainedWorkerAdmissionRecordV1, bool), RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, plan)?;
    let VersionedStateRoot::V2(root) = authority_root else {
        return Err(RetainedWorkerRuntimeError(
            "registration advancement requires strict V2 authority".into(),
        ));
    };
    let mut record = registry
        .records_by_session
        .get(&plan.spawn_request.orchestration_session_id)
        .and_then(|records| records.get(retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot is absent".into()))?;
    validate_supplied_admission_authority(
        root,
        &plan.exact_authority,
        supplied_exact_authority,
        &record,
    )?;
    verify_admission_record_fingerprint(root, plan, &record, secret_key)?;
    if let Some(registration) = admission_registration(&record.state) {
        validate_registration_result(authority_root, &record, registration, registration_result)?;
        return Ok((record, false));
    }
    if !matches!(
        record.state,
        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
    ) {
        return Err(RetainedWorkerRuntimeError(
            "only the registration head can advance after R0".into(),
        ));
    }
    let registration = admission_registration_from_hsa(root, &record)?.ok_or_else(|| {
        RetainedWorkerRuntimeError("R0 registration has not reached durable Applied truth".into())
    })?;
    validate_registration_result(authority_root, &record, &registration, registration_result)?;
    record.state = RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration };
    record.record_revision = record
        .record_revision
        .checked_add(1)
        .ok_or_else(|| RetainedWorkerRuntimeError("admission record revision overflow".into()))?;
    registry
        .records_by_session
        .get_mut(&record.orchestration_session_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("admission session bucket is absent".into()))?
        .insert(record.retained_participant_id.clone(), record.clone());
    validate_admission_registry(registry, authority_root)?;
    Ok((record, true))
}

fn claim_transport_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    plan: &RetainedWorkerAdmissionPlanV1,
    supplied_exact_authority: &CanonicalExactCurrentAuthorityV1,
    retained_participant_id: &str,
    secret_key: &[u8; 32],
    claim_input: &AdmissionTransportClaimInputV1,
) -> Result<RetainedWorkerTransportClaimV1, RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, plan)?;
    let VersionedStateRoot::V2(root) = authority_root else {
        return Err(RetainedWorkerRuntimeError(
            "transport claim requires strict V2 authority".into(),
        ));
    };
    let mut record = registry
        .records_by_session
        .get(&plan.spawn_request.orchestration_session_id)
        .and_then(|records| records.get(retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot is absent".into()))?;
    validate_supplied_admission_authority(
        root,
        &plan.exact_authority,
        supplied_exact_authority,
        &record,
    )?;
    verify_admission_record_fingerprint(root, plan, &record, secret_key)?;
    let registration = match &record.state {
        RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration } => {
            registration.clone()
        }
        RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::Routable { .. }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::Terminal { .. } => {
            return Ok(RetainedWorkerTransportClaimV1 {
                record,
                newly_claimed: false,
            });
        }
        RetainedWorkerAdmissionStateV1::SlotReserved { .. }
        | RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => {
            return Err(RetainedWorkerRuntimeError(
                "transport cannot be claimed before exact R0 admission".into(),
            ));
        }
    };
    let transport_claim_id = format!("rtc_{}", lower_hex(&claim_input.claim_entropy));
    if registry
        .records_by_session
        .values()
        .flat_map(BTreeMap::values)
        .any(|candidate| {
            matches!(
                &candidate.state,
                RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                    transport_claim_id: existing,
                    ..
                } if existing == &transport_claim_id
            )
        })
    {
        return Err(RetainedWorkerRuntimeError(
            "generated transport claim identity collides".into(),
        ));
    }
    record.state = RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
        registration,
        transport_claim_id,
        claimed_at: claim_input.claimed_at.clone(),
    };
    record.record_revision = record
        .record_revision
        .checked_add(1)
        .ok_or_else(|| RetainedWorkerRuntimeError("admission record revision overflow".into()))?;
    registry
        .records_by_session
        .get_mut(&record.orchestration_session_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("admission session bucket is absent".into()))?
        .insert(record.retained_participant_id.clone(), record.clone());
    validate_admission_registry(registry, authority_root)?;
    Ok(RetainedWorkerTransportClaimV1 {
        record,
        newly_claimed: true,
    })
}

fn admission_registration_from_hsa(
    root: &StateRootV2,
    record: &RetainedWorkerAdmissionRecordV1,
) -> Result<Option<RetainedWorkerAdmissionRegistrationV1>, RetainedWorkerRuntimeError> {
    let RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
        authority_revision_expected,
        authority_record_commitment_expected,
        ..
    } = &record.state
    else {
        return Err(RetainedWorkerRuntimeError(
            "R0 join requires the exact admission registration head".into(),
        ));
    };
    let issuer_request_id = format!("retained-worker-registration:{}", record.issuer_request_id);
    let Some(request) = root
        .retained_worker_registration_request_index
        .get(&issuer_request_id)
    else {
        return Ok(None);
    };
    if request.schema_version != 1
        || request.issuer_request_id != issuer_request_id
        || request.orchestration_session_id != record.orchestration_session_id
        || request.authority_revision_before != *authority_revision_expected
        || request.authority_record_commitment_before != *authority_record_commitment_expected
        || request.retained_participant_id != record.retained_participant_id
        || request.current_policy_ref != record.current_policy_ref
        || request.world_binding != record.world_binding
    {
        return Err(RetainedWorkerRuntimeError(
            "R0 request conflicts with its admission registration head".into(),
        ));
    }
    let RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
        authority_revision_after,
        authority_record_commitment_after,
    } = &request.state
    else {
        return Ok(None);
    };
    let registration = root
        .retained_worker_registration_journal
        .get(&request.registration_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("applied R0 journal is absent".into()))?;
    if registration.schema_version != 1
        || registration.issuer_request_id != request.issuer_request_id
        || registration.registration_id != request.registration_id
        || registration.orchestration_session_id != request.orchestration_session_id
        || registration.authority_revision_before != request.authority_revision_before
        || registration.authority_record_commitment_before
            != request.authority_record_commitment_before
        || registration.authority_revision_after != *authority_revision_after
        || registration.authority_record_commitment_after != *authority_record_commitment_after
        || registration.retained_participant_id != request.retained_participant_id
        || registration.current_policy_ref != request.current_policy_ref
        || registration.world_binding != request.world_binding
    {
        return Err(RetainedWorkerRuntimeError(
            "applied R0 request and journal are inexact".into(),
        ));
    }
    let admission_registration = RetainedWorkerAdmissionRegistrationV1 {
        registration_id: registration.registration_id.clone(),
        retained_worker_ref: registration.retained_worker_ref.clone(),
    };
    validate_admission_registration_against_authority(
        &VersionedStateRoot::V2(root.clone()),
        record,
        &admission_registration,
    )?;
    Ok(Some(admission_registration))
}

fn validate_registration_result(
    authority_root: &VersionedStateRoot,
    record: &RetainedWorkerAdmissionRecordV1,
    registration: &RetainedWorkerAdmissionRegistrationV1,
    result: &RetainedWorkerRegistrationResultV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    let VersionedStateRoot::V2(root) = authority_root else {
        return Err(RetainedWorkerRuntimeError(
            "R0 result validation requires strict V2 authority".into(),
        ));
    };
    let durable = root
        .retained_worker_registration_journal
        .get(&registration.registration_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("R0 result has no durable journal".into()))?;
    if result.registration_id != registration.registration_id
        || result.authority_store_id != record.authority_store_id
        || result.orchestration_session_id != record.orchestration_session_id
        || result.retained_participant_id != record.retained_participant_id
        || result.retained_worker_ref != registration.retained_worker_ref
        || result.authority_revision_after != durable.authority_revision_after
        || result.authority_record_commitment_after != durable.authority_record_commitment_after
    {
        return Err(RetainedWorkerRuntimeError(
            "R0 registration result conflicts with durable admission truth".into(),
        ));
    }
    validate_admission_registration_against_authority(authority_root, record, registration)
}

fn admission_registration_plan(
    record: &RetainedWorkerAdmissionRecordV1,
    plan: &RetainedWorkerAdmissionPlanV1,
) -> Result<RetainedWorkerRegistrationPlanV1, RetainedWorkerRuntimeError> {
    let RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
        authority_revision_expected,
        authority_record_commitment_expected,
        ..
    } = &record.state
    else {
        return Err(RetainedWorkerRuntimeError(
            "R0 registration plan requires the exact admission head".into(),
        ));
    };
    if record.orchestration_session_id != plan.spawn_request.orchestration_session_id
        || record.backend_id != plan.descriptor_and_runtime_plan.descriptor.backend_id
        || record.protocol != plan.descriptor_and_runtime_plan.descriptor.protocol
    {
        return Err(RetainedWorkerRuntimeError(
            "R0 registration plan conflicts with canonical admission inputs".into(),
        ));
    }
    Ok(RetainedWorkerRegistrationPlanV1 {
        registration_request_id: record.issuer_request_id.clone(),
        orchestration_session_id: record.orchestration_session_id.clone(),
        expected_authority: RetainedWorkerAuthorityPreconditionV1 {
            authority_store_id: record.authority_store_id.clone(),
            authority_revision: *authority_revision_expected,
            authority_record_commitment: authority_record_commitment_expected.clone(),
        },
        retained_participant_id: record.retained_participant_id.clone(),
        descriptor: plan.descriptor_and_runtime_plan.descriptor.clone(),
        internal_uaa_session_id: format!("uaa_{}", record.bootstrap_run_id),
    })
}

fn reserve_slot_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    plan: &RetainedWorkerAdmissionPlanV1,
    secret_key: &[u8; 32],
    reserved_at: TimestampV1,
    participant_entropy: [u8; 16],
    bootstrap_run_entropy: [u8; 16],
) -> Result<RetainedWorkerAdmissionSlotV1, RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, plan)?;
    if let Some(locator) = registry.issuer_request_index.get(&plan.issuer_request_id) {
        let record = registry
            .records_by_session
            .get(&locator.orchestration_session_id)
            .and_then(|records| records.get(&locator.retained_participant_id))
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("admission issuer index is dangling".into())
            })?;
        let VersionedStateRoot::V2(root) = authority_root else {
            return Err(RetainedWorkerRuntimeError(
                "admission retry requires strict V2 authority".into(),
            ));
        };
        let mut fingerprint_plan = plan.clone();
        fingerprint_plan.exact_authority = reconstruct_exact_authority_at_revision(
            root,
            &plan.exact_authority,
            record.admission_authority_revision,
        )?;
        let fingerprint = canonical_spawn_fingerprint(
            &registry.commitment_key.key_id,
            secret_key,
            &fingerprint_plan,
            &record.retained_participant_id,
            &record.bootstrap_run_id,
        )?;
        if record.schema_version != 1
            || record.authority_store_id != fingerprint_plan.exact_authority.authority_store_id
            || record.issuer_request_id != plan.issuer_request_id
            || record.canonical_spawn_fingerprint != fingerprint
            || record.orchestration_session_id != plan.spawn_request.orchestration_session_id
            || record.admission_authority_revision
                != fingerprint_plan.exact_authority.authority_revision
            || record.admission_authority_record_commitment
                != fingerprint_plan.exact_authority.authority_record_commitment
            || record.backend_id != plan.descriptor_and_runtime_plan.descriptor.backend_id
            || record.protocol != plan.descriptor_and_runtime_plan.descriptor.protocol
            || record.world_binding.world_id != plan.spawn_request.world_id
            || record.world_binding.world_generation != plan.spawn_request.world_generation
            || record.current_policy_ref != plan.policy_and_admission_cap.current_policy_ref
            || record.current_policy_revision
                != plan.policy_and_admission_cap.current_policy.policy_revision
            || record.max_live_retained_workers
                != plan.policy_and_admission_cap.max_live_retained_workers
        {
            return Err(RetainedWorkerRuntimeError(
                "admission issuer retry changed canonical spawn bytes or bound scope".into(),
            ));
        }
        return Ok(RetainedWorkerAdmissionSlotV1 {
            record: record.clone(),
            joined: true,
        });
    }

    let live_count = registry
        .records_by_session
        .values()
        .flat_map(BTreeMap::values)
        .filter(|record| admission_state_is_live(&record.state))
        .count() as u64;
    if live_count >= plan.policy_and_admission_cap.max_live_retained_workers {
        return Err(RetainedWorkerRuntimeError(format!(
            "retained_worker_cap_reached: live retained worker count {live_count} reaches configured maximum {}",
            plan.policy_and_admission_cap.max_live_retained_workers
        )));
    }

    let retained_participant_id = format!("rwp_{}", lower_hex(&participant_entropy));
    let bootstrap_run_id = format!("rwr_{}", lower_hex(&bootstrap_run_entropy));
    if registry
        .records_by_session
        .values()
        .flat_map(BTreeMap::values)
        .any(|record| {
            record.retained_participant_id == retained_participant_id
                || record.bootstrap_run_id == bootstrap_run_id
        })
    {
        return Err(RetainedWorkerRuntimeError(
            "generated admission participant or bootstrap-run identity collides".into(),
        ));
    }
    let slot_sequence = registry
        .next_slot_sequence_by_session
        .get(&plan.spawn_request.orchestration_session_id)
        .copied()
        .unwrap_or(1);
    let next_slot_sequence = slot_sequence
        .checked_add(1)
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot sequence overflow".into()))?;
    let fingerprint = canonical_spawn_fingerprint(
        &registry.commitment_key.key_id,
        secret_key,
        plan,
        &retained_participant_id,
        &bootstrap_run_id,
    )?;
    let world_binding = WorldBindingV1 {
        world_id: plan.spawn_request.world_id.clone(),
        world_generation: plan.spawn_request.world_generation,
    };
    let record = RetainedWorkerAdmissionRecordV1 {
        schema_version: 1,
        authority_store_id: plan.exact_authority.authority_store_id.clone(),
        issuer_request_id: plan.issuer_request_id.clone(),
        canonical_spawn_fingerprint: fingerprint,
        orchestration_session_id: plan.spawn_request.orchestration_session_id.clone(),
        admission_authority_revision: plan.exact_authority.authority_revision,
        admission_authority_record_commitment: plan
            .exact_authority
            .authority_record_commitment
            .clone(),
        retained_participant_id: retained_participant_id.clone(),
        bootstrap_run_id,
        backend_id: plan
            .descriptor_and_runtime_plan
            .descriptor
            .backend_id
            .clone(),
        protocol: plan.descriptor_and_runtime_plan.descriptor.protocol.clone(),
        world_binding,
        current_policy_ref: plan.policy_and_admission_cap.current_policy_ref.clone(),
        current_policy_revision: plan
            .policy_and_admission_cap
            .current_policy
            .policy_revision
            .clone(),
        max_live_retained_workers: plan.policy_and_admission_cap.max_live_retained_workers,
        state: RetainedWorkerAdmissionStateV1::SlotReserved {
            slot_sequence,
            reserved_at,
        },
        record_revision: 1,
    };
    registry.issuer_request_index.insert(
        plan.issuer_request_id.clone(),
        RetainedWorkerAdmissionRecordLocatorV1 {
            orchestration_session_id: record.orchestration_session_id.clone(),
            retained_participant_id: retained_participant_id.clone(),
        },
    );
    registry
        .records_by_session
        .entry(record.orchestration_session_id.clone())
        .or_default()
        .insert(retained_participant_id, record.clone());
    registry.next_slot_sequence_by_session.insert(
        plan.spawn_request.orchestration_session_id.clone(),
        next_slot_sequence,
    );
    validate_admission_registry(registry, authority_root)?;
    Ok(RetainedWorkerAdmissionSlotV1 {
        record,
        joined: false,
    })
}

fn validate_admission_plan(
    authority_root: &VersionedStateRoot,
    plan: &RetainedWorkerAdmissionPlanV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    let request = &plan.spawn_request;
    let exact = &plan.exact_authority;
    let runtime = &plan.descriptor_and_runtime_plan;
    let policy = &plan.policy_and_admission_cap;
    let VersionedStateRoot::V2(root) = authority_root else {
        return Err(RetainedWorkerRuntimeError(
            "retained admission requires strict V2 authority".into(),
        ));
    };
    let SessionNamespaceRecordV1::Authority(locked_authority) = root
        .session_namespace_map
        .get(&request.orchestration_session_id)
        .ok_or_else(|| {
            RetainedWorkerRuntimeError("admission session authority is absent".into())
        })?
    else {
        return Err(RetainedWorkerRuntimeError(
            "admission session namespace is not authority".into(),
        ));
    };
    AgentDescriptorHashInputV1 {
        schema_version: 1,
        descriptor: runtime.descriptor.clone(),
    }
    .validate()
    .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
    policy
        .current_policy
        .validate()
        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
    policy
        .current_policy_ref
        .validate()
        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
    if plan.issuer_request_id.is_empty()
        || request.schema_version != 1
        || request.request_id.is_empty()
        || request.idempotency_key.is_empty()
        || request.action != "spawn_world_worker"
        || request.mode != "retained"
        || request.task_run_id.is_some()
        || request.target_participant_id.is_some()
        || request.payload.prompt.is_empty()
        || exact.schema_version != 1
        || exact.authority_store_id != root.authority_store_id
        || exact.orchestration_session_id != request.orchestration_session_id
        || exact.authority_revision != locked_authority.authority_revision
        || exact.authority != **locked_authority
        || exact.authority.orchestration_session_id != request.orchestration_session_id
        || exact
            .authority
            .active_authoritative_participant_id
            .as_deref()
            != Some(request.caller_participant_id.as_str())
        || exact.caller_participant_id != request.caller_participant_id
        || exact.caller_role != "orchestrator"
        || exact.caller_descriptor_ref != exact.host_attach_contract.descriptor_ref
        || exact.caller_descriptor.backend_id != exact.host_attach_contract.backend_id
        || exact.caller_descriptor.protocol != exact.host_attach_contract.protocol
        || exact.current_policy != policy.current_policy
        || exact.authority.current_policy_ref.as_ref() != Some(&policy.current_policy_ref)
        || exact.authority.current_policy_revision.as_deref()
            != Some(policy.current_policy.policy_revision.as_str())
        || exact.authority.world_binding.as_ref()
            != Some(&WorldBindingV1 {
                world_id: request.world_id.clone(),
                world_generation: request.world_generation,
            })
        || runtime.schema_version != 1
        || runtime.descriptor.execution_scope != AgentExecutionScopeV1::World
        || runtime.descriptor.backend_id != request.target_backend_id
        || runtime.runtime_role != "member"
        || runtime.internal_uaa_session_id_domain
            != "substrate.retained-worker.internal-uaa-session.v1"
        || policy.schema_version != 1
        || !policy.dispatch_enabled
        || !policy
            .allowed_backends
            .iter()
            .any(|backend| backend == &request.target_backend_id)
        || !policy
            .allowed_actions
            .iter()
            .any(|action| action == &request.action)
        || !policy
            .allowed_modes
            .iter()
            .any(|mode| mode == &request.mode)
        || !policy.same_session_only
        || !policy.same_world_binding_only
        || policy.allow_capability_narrowing
    {
        return Err(RetainedWorkerRuntimeError(
            "canonical admission plan is incomplete or conflicts with locked authority".into(),
        ));
    }
    Ok(())
}

fn canonical_spawn_fingerprint(
    key_id: &str,
    secret_key: &[u8; 32],
    plan: &RetainedWorkerAdmissionPlanV1,
    retained_participant_id: &str,
    bootstrap_run_id: &str,
) -> Result<RetainedWorkerAdmissionCommitmentV1, RetainedWorkerRuntimeError> {
    let request = encode_canonical(
        &plan.spawn_request,
        "encode canonical validated spawn request",
    )?;
    let exact_authority = encode_canonical(
        &plan.exact_authority,
        "encode canonical exact current authority",
    )?;
    let descriptor_runtime = encode_canonical(
        &CanonicalDescriptorAndRuntimeFingerprintV1 {
            schema_version: 1,
            descriptor: &plan.descriptor_and_runtime_plan.descriptor,
            runtime_role: &plan.descriptor_and_runtime_plan.runtime_role,
            internal_uaa_session_id_domain: &plan
                .descriptor_and_runtime_plan
                .internal_uaa_session_id_domain,
            retained_participant_id,
            bootstrap_run_id,
            internal_uaa_session_id: format!("uaa_{bootstrap_run_id}"),
        },
        "encode canonical descriptor and runtime plan",
    )?;
    let policy_cap = encode_canonical(
        &plan.policy_and_admission_cap,
        "encode canonical policy and admission cap",
    )?;
    let mut input = b"substrate.retained-worker.admission.hmac-input.v1\0".to_vec();
    for member in [
        b"substrate.retained-worker.admission.spawn.v1".as_slice(),
        plan.exact_authority.authority_store_id.as_bytes(),
        plan.issuer_request_id.as_bytes(),
        request.as_slice(),
        exact_authority.as_slice(),
        descriptor_runtime.as_slice(),
        policy_cap.as_slice(),
        retained_participant_id.as_bytes(),
        bootstrap_run_id.as_bytes(),
    ] {
        append_len64(&mut input, member)?;
    }
    let digest = hmac_sha256(secret_key, &input);
    Ok(RetainedWorkerAdmissionCommitmentV1 {
        schema_version: 1,
        algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256,
        key_id: key_id.to_owned(),
        digest_hex: lower_hex(&digest),
    })
}

fn reconstruct_exact_authority_at_revision(
    root: &StateRootV2,
    current: &CanonicalExactCurrentAuthorityV1,
    target_revision: u64,
) -> Result<CanonicalExactCurrentAuthorityV1, RetainedWorkerRuntimeError> {
    if current.authority_store_id != root.authority_store_id
        || current.authority.authority_revision != current.authority_revision
        || target_revision == 0
        || target_revision > current.authority_revision
    {
        return Err(RetainedWorkerRuntimeError(
            "admission authority ancestry bounds are invalid".into(),
        ));
    }
    let DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } = &current.authority.origin;
    let intent = root.transition_intent_map.get(intent_id).ok_or_else(|| {
        RetainedWorkerRuntimeError("admission authority ancestry has no Start intent".into())
    })?;
    let HostSessionTransitionIntentStateV2::Applied {
        authority_revision_after,
        authority_record_commitment,
        applied_at,
        ..
    } = &intent.state
    else {
        return Err(RetainedWorkerRuntimeError(
            "admission authority ancestry Start is not applied".into(),
        ));
    };
    let mut authority = current.authority.clone();
    authority.authority_revision = *authority_revision_after;
    authority.authoritative_participant_lineage = intent.resulting_authoritative_lineage.clone();
    authority.retained_worker_refs.clear();
    authority.internal_resume_handle_refs.clear();
    authority.updated_at = applied_at.clone();
    let mut commitment = canonical_authority_commitment(&authority)?;
    if commitment != *authority_record_commitment {
        return Err(RetainedWorkerRuntimeError(
            "admission authority ancestry initial commitment is inexact".into(),
        ));
    }
    while authority.authority_revision < target_revision {
        let candidates = root
            .retained_worker_registration_journal
            .values()
            .filter(|registration| {
                registration.orchestration_session_id == authority.orchestration_session_id
                    && registration.authority_revision_before == authority.authority_revision
                    && registration.authority_record_commitment_before == commitment
            })
            .collect::<Vec<_>>();
        let [registration] = candidates.as_slice() else {
            return Err(RetainedWorkerRuntimeError(
                "admission authority ancestry is not uniquely contiguous R0".into(),
            ));
        };
        if registration.authority_revision_after
            != authority.authority_revision.checked_add(1).ok_or_else(|| {
                RetainedWorkerRuntimeError("admission authority revision overflow".into())
            })?
            || authority.current_policy_ref.as_ref() != Some(&registration.current_policy_ref)
            || authority.world_binding.as_ref() != Some(&registration.world_binding)
            || authority
                .authoritative_participant_lineage
                .contains(&registration.retained_participant_id)
            || authority
                .retained_worker_refs
                .contains(&registration.retained_worker_ref)
        {
            return Err(RetainedWorkerRuntimeError(
                "admission authority R0 ancestry link is inconsistent".into(),
            ));
        }
        authority.authority_revision = registration.authority_revision_after;
        authority
            .authoritative_participant_lineage
            .push(registration.retained_participant_id.clone());
        authority
            .retained_worker_refs
            .push(registration.retained_worker_ref.clone());
        authority.updated_at = registration.registered_at.clone();
        let lineage = canonical_lineage_commitment(&authority)?;
        if lineage != registration.authoritative_lineage_commitment_after {
            return Err(RetainedWorkerRuntimeError(
                "admission authority R0 lineage commitment is inconsistent".into(),
            ));
        }
        commitment = canonical_authority_commitment(&authority)?;
        if commitment != registration.authority_record_commitment_after {
            return Err(RetainedWorkerRuntimeError(
                "admission authority R0 record commitment is inconsistent".into(),
            ));
        }
    }
    let mut reconstructed = current.clone();
    reconstructed.authority_revision = target_revision;
    reconstructed.authority_record_commitment = commitment;
    reconstructed.authoritative_lineage_commitment = canonical_lineage_commitment(&authority)?;
    reconstructed.authority = authority;
    Ok(reconstructed)
}

fn canonical_authority_commitment(
    authority: &DurableSessionAuthorityV1,
) -> Result<AuthorityObjectCommitmentV1, RetainedWorkerRuntimeError> {
    let input = DurableSessionAuthorityHashInputV1 {
        schema_version: authority.schema_version,
        orchestration_session_id: authority.orchestration_session_id.clone(),
        shell_trace_session_id: authority.shell_trace_session_id.clone(),
        authority_revision: authority.authority_revision,
        origin: authority.origin.clone(),
        authoritative_participant_lineage: authority.authoritative_participant_lineage.clone(),
        active_authoritative_participant_id: authority.active_authoritative_participant_id.clone(),
        workspace_binding: authority.workspace_binding.clone(),
        world_binding: authority.world_binding.clone(),
        host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
        retained_worker_refs: authority.retained_worker_refs.clone(),
        internal_resume_handle_refs: authority.internal_resume_handle_refs.clone(),
        lifecycle_posture: authority.lifecycle_posture,
        current_policy_ref: authority.current_policy_ref.clone(),
        current_policy_revision: authority.current_policy_revision.clone(),
    };
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: super::host_session_authority::hash::canonical_sha256(&input)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?,
    })
}

fn canonical_lineage_commitment(
    authority: &DurableSessionAuthorityV1,
) -> Result<AuthorityObjectCommitmentV1, RetainedWorkerRuntimeError> {
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: super::host_session_authority::hash::canonical_sha256(
            &AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: authority.orchestration_session_id.clone(),
                participant_ids: authority.authoritative_participant_lineage.clone(),
            },
        )
        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?,
    })
}

fn append_len64(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), RetainedWorkerRuntimeError> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| RetainedWorkerRuntimeError("admission HMAC member is too large".into()))?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(bytes);
    Ok(())
}

fn hmac_sha256(secret_key: &[u8; 32], input: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;
    let mut inner_pad = [0x36_u8; BLOCK_SIZE];
    let mut outer_pad = [0x5c_u8; BLOCK_SIZE];
    for (index, byte) in secret_key.iter().enumerate() {
        inner_pad[index] ^= byte;
        outer_pad[index] ^= byte;
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(input);
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}

fn admission_state_is_live(state: &RetainedWorkerAdmissionStateV1) -> bool {
    !matches!(
        state,
        RetainedWorkerAdmissionStateV1::Terminal { .. }
            | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. }
    )
}

fn admission_registration(
    state: &RetainedWorkerAdmissionStateV1,
) -> Option<&RetainedWorkerAdmissionRegistrationV1> {
    match state {
        RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration }
        | RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { registration, .. }
        | RetainedWorkerAdmissionStateV1::Routable { registration, .. }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal { registration, .. }
        | RetainedWorkerAdmissionStateV1::Terminal { registration, .. } => Some(registration),
        RetainedWorkerAdmissionStateV1::SlotReserved { .. }
        | RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => None,
    }
}

fn validate_admission_registry(
    registry: &RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
) -> Result<(), RetainedWorkerRuntimeError> {
    if registry.schema_version != 1
        || registry.authority_store_id != authority_root.authority_store_id()
        || registry.commitment_key.schema_version != 1
        || registry.commitment_key.authority_store_id != registry.authority_store_id
        || registry.commitment_key.algorithm
            != RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256
        || !valid_key_id(&registry.commitment_key.key_id)
    {
        return Err(RetainedWorkerRuntimeError(
            "admission registry header is invalid".into(),
        ));
    }
    let mut expected_issuers = BTreeMap::new();
    let mut participant_ids = BTreeSet::new();
    let mut bootstrap_run_ids = BTreeSet::new();
    let mut registration_ids = BTreeSet::new();
    let mut transport_claim_ids = BTreeSet::new();
    for (session_id, records) in &registry.records_by_session {
        if session_id.is_empty() || records.is_empty() {
            return Err(RetainedWorkerRuntimeError(
                "admission registry session bucket is invalid".into(),
            ));
        }
        let mut head_count = 0_usize;
        let mut slot_sequences = BTreeSet::new();
        for (participant_id, record) in records {
            record
                .current_policy_ref
                .validate()
                .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
            if record.schema_version != 1
                || record.authority_store_id != registry.authority_store_id
                || record.issuer_request_id.is_empty()
                || record.orchestration_session_id != *session_id
                || record.retained_participant_id != *participant_id
                || !valid_generated_id(&record.retained_participant_id, "rwp_")
                || !valid_generated_id(&record.bootstrap_run_id, "rwr_")
                || record.backend_id.is_empty()
                || record.protocol.is_empty()
                || record.world_binding.world_id.is_empty()
                || record.current_policy_revision.is_empty()
                || record.admission_authority_revision == 0
                || record.max_live_retained_workers == 0
                || record.record_revision == 0
                || record.canonical_spawn_fingerprint.schema_version != 1
                || record.canonical_spawn_fingerprint.algorithm
                    != RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256
                || record.canonical_spawn_fingerprint.key_id != registry.commitment_key.key_id
                || !is_lower_hex(&record.canonical_spawn_fingerprint.digest_hex, 64)
                || !participant_ids.insert(record.retained_participant_id.clone())
                || !bootstrap_run_ids.insert(record.bootstrap_run_id.clone())
            {
                return Err(RetainedWorkerRuntimeError(
                    "admission registry record identity is invalid".into(),
                ));
            }
            match &record.state {
                RetainedWorkerAdmissionStateV1::SlotReserved { slot_sequence, .. } => {
                    if *slot_sequence == 0 || !slot_sequences.insert(*slot_sequence) {
                        return Err(RetainedWorkerRuntimeError(
                            "admission slot sequence is invalid".into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
                    authority_revision_expected,
                    ..
                } => {
                    head_count += 1;
                    if *authority_revision_expected == 0 {
                        return Err(RetainedWorkerRuntimeError(
                            "admission registration head revision is invalid".into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                    transport_claim_id,
                    ..
                } => {
                    if !valid_generated_id(transport_claim_id, "rtc_")
                        || !transport_claim_ids.insert(transport_claim_id.clone())
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "admission transport claim identity is invalid or duplicated".into(),
                        ));
                    }
                }
                _ => {}
            }
            if head_count > 1 {
                return Err(RetainedWorkerRuntimeError(
                    "admission registry has more than one session registration head".into(),
                ));
            }
            if let Some(registration) = admission_registration(&record.state) {
                if registration.registration_id.is_empty()
                    || !registration_ids.insert(registration.registration_id.clone())
                {
                    return Err(RetainedWorkerRuntimeError(
                        "admission registration identity is invalid or duplicated".into(),
                    ));
                }
                validate_admission_registration_against_authority(
                    authority_root,
                    record,
                    registration,
                )?;
            }
            if expected_issuers
                .insert(
                    record.issuer_request_id.clone(),
                    RetainedWorkerAdmissionRecordLocatorV1 {
                        orchestration_session_id: session_id.clone(),
                        retained_participant_id: participant_id.clone(),
                    },
                )
                .is_some()
            {
                return Err(RetainedWorkerRuntimeError(
                    "admission issuer identity is duplicated".into(),
                ));
            }
        }
        let next = registry
            .next_slot_sequence_by_session
            .get(session_id)
            .copied()
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("admission next slot sequence is absent".into())
            })?;
        if next == 0
            || slot_sequences
                .iter()
                .next_back()
                .is_some_and(|value| *value >= next)
        {
            return Err(RetainedWorkerRuntimeError(
                "admission next slot sequence is not monotonic".into(),
            ));
        }
    }
    if registry.issuer_request_index != expected_issuers
        || registry
            .next_slot_sequence_by_session
            .keys()
            .any(|session| !registry.records_by_session.contains_key(session))
    {
        return Err(RetainedWorkerRuntimeError(
            "admission registry indices are inexact".into(),
        ));
    }
    let VersionedStateRoot::V2(root) = authority_root else {
        return Err(RetainedWorkerRuntimeError(
            "retained admission requires strict V2 authority".into(),
        ));
    };
    for request in root.retained_worker_registration_request_index.values() {
        let admission_issuer = request
            .issuer_request_id
            .strip_prefix("retained-worker-registration:")
            .ok_or_else(|| {
                RetainedWorkerRuntimeError(
                    "HSA retained registration issuer is not domain separated".into(),
                )
            })?;
        let matching_records = registry
            .records_by_session
            .values()
            .flat_map(BTreeMap::values)
            .filter(|record| record.issuer_request_id == admission_issuer)
            .collect::<Vec<_>>();
        let [record] = matching_records.as_slice() else {
            return Err(RetainedWorkerRuntimeError(
                "HSA retained registration has no unique admission record".into(),
            ));
        };
        let RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
            authority_revision_expected,
            authority_record_commitment_expected,
            ..
        } = &record.state
        else {
            let Some(registration) = admission_registration(&record.state) else {
                return Err(RetainedWorkerRuntimeError(
                    "HSA retained registration precedes its admission head".into(),
                ));
            };
            if registration.registration_id != request.registration_id {
                return Err(RetainedWorkerRuntimeError(
                    "HSA retained registration request conflicts with admission state".into(),
                ));
            }
            continue;
        };
        if request.orchestration_session_id != record.orchestration_session_id
            || request.retained_participant_id != record.retained_participant_id
            || request.authority_revision_before != *authority_revision_expected
            || request.authority_record_commitment_before != *authority_record_commitment_expected
            || request.current_policy_ref != record.current_policy_ref
            || request.world_binding != record.world_binding
        {
            return Err(RetainedWorkerRuntimeError(
                "HSA retained registration request conflicts with admission head".into(),
            ));
        }
        if matches!(
            request.state,
            RetainedWorkerAuthorityRegistrationRequestStateV1::Applied { .. }
        ) {
            let registration = root
                .retained_worker_registration_journal
                .get(&request.registration_id)
                .ok_or_else(|| {
                    RetainedWorkerRuntimeError(
                        "applied HSA retained registration journal is absent".into(),
                    )
                })?;
            validate_admission_registration_against_authority(
                authority_root,
                record,
                &RetainedWorkerAdmissionRegistrationV1 {
                    registration_id: registration.registration_id.clone(),
                    retained_worker_ref: registration.retained_worker_ref.clone(),
                },
            )?;
        }
    }
    Ok(())
}

fn validate_admission_registration_against_authority(
    authority_root: &VersionedStateRoot,
    record: &RetainedWorkerAdmissionRecordV1,
    admission_registration: &RetainedWorkerAdmissionRegistrationV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    let VersionedStateRoot::V2(root) = authority_root else {
        return Err(RetainedWorkerRuntimeError(
            "post-R0 admission record requires V2 authority".into(),
        ));
    };
    let registration = root
        .retained_worker_registration_journal
        .get(&admission_registration.registration_id)
        .ok_or_else(|| {
            RetainedWorkerRuntimeError("post-R0 admission registration is absent".into())
        })?;
    if registration.registration_id != admission_registration.registration_id
        || registration.orchestration_session_id != record.orchestration_session_id
        || registration.retained_participant_id != record.retained_participant_id
        || registration.retained_worker_ref != admission_registration.retained_worker_ref
        || registration.current_policy_ref != record.current_policy_ref
        || registration.world_binding != record.world_binding
    {
        return Err(RetainedWorkerRuntimeError(
            "post-R0 admission registration conflicts with authority truth".into(),
        ));
    }
    Ok(())
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_generated_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(|hex| is_lower_hex(hex, 32))
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    use super::*;
    use crate::execution::agent_runtime::host_session_authority::facade::{
        AuthorityObservationV1, HostSessionAuthority, RetainedApplicationCrashPointV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AgentExecutionScopeV1, HostAttachCapabilitiesV1, HostAttachExecutionClientStartV1,
        HostAttachLaunchKnobsV1, HostAttachModePreferenceV1, HostSessionAuthorityPreconditionV1,
        HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
        HostSessionTransitionModeV1, PolicyObjectHashInputV1, RuntimeBackendKindV1, TimestampV1,
        WorkspaceBindingV1, WorldBindingV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentStateV2;
    use crate::execution::agent_runtime::host_session_authority::transition::{
        ApplyHostSessionTransitionRequestV1, ClaimHostSessionTransitionRequestV1,
        IssueHostSessionTransitionRequestV1, StartContractMaterialV1,
        TransitionApplicationOutcomeV1, TransitionClaimOutcomeV1, TransitionIssueOutcomeV1,
    };

    fn timestamp(value: &str) -> TimestampV1 {
        TimestampV1::parse(value).unwrap()
    }

    fn started_authority() -> (
        tempfile::TempDir,
        HostSessionAuthority,
        AuthorityObservationV1,
    ) {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        let authority = HostSessionAuthority::open(&home).unwrap();
        let root = authority.bootstrap().unwrap();
        let binding = WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home,
            authority_store_id: root.authority_store_id,
        };
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: "r0-start-intent".into(),
            issuer_request_id: "retained-worker-registration:transition-start".into(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: "r0-session".into(),
            shell_trace_session_id: "r0-trace".into(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: "r0-orchestrator".into(),
            target_participant_lease_token: b"r0-start-lease".to_vec(),
            run_id: "r0-start-run".into(),
            resulting_authoritative_lineage: vec!["r0-orchestrator".into()],
            workspace_binding: binding,
            world_binding: Some(WorldBindingV1 {
                world_id: "r0-world".into(),
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
                    policy_revision: "r0-policy".into(),
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
        };
        let TransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
            .unwrap()
        else {
            panic!("Start issuance must commit")
        };
        let claim_request = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: "r0-start-claim".into(),
            claimant_attempt_id: "r0-start-attempt".into(),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(
                &claim_request,
                timestamp("2026-07-14T12:01:00.000000000Z"),
                30,
            )
            .unwrap()
        else {
            panic!("Start claim must commit")
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            panic!("Start must remain claimed")
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
                .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
                .unwrap(),
            TransitionApplicationOutcomeV1::Applied(_)
        ));
        let observation = authority
            .resolve_current_exact("r0-session", None)
            .unwrap()
            .observation;
        (parent, authority, observation)
    }

    fn plan(observation: AuthorityObservationV1) -> RetainedWorkerRegistrationPlanV1 {
        RetainedWorkerRegistrationPlanV1 {
            registration_request_id: "spawn-request-1".into(),
            orchestration_session_id: "r0-session".into(),
            expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                authority_store_id: observation.authority_store_id,
                authority_revision: observation.authority_revision,
                authority_record_commitment: observation.authority_record_commitment,
            },
            retained_participant_id: "r0-retained-1".into(),
            descriptor: AgentDescriptorV1 {
                schema_version: 1,
                agent_id: "codex-worker".into(),
                backend_id: "cli:codex-worker".into(),
                backend_kind: RuntimeBackendKindV1::Codex,
                protocol: "substrate.agent.session".into(),
                execution_scope: AgentExecutionScopeV1::World,
                binary_path: "/usr/bin/codex".into(),
            },
            internal_uaa_session_id: "uaa-retained-1".into(),
        }
    }

    fn object_files(path: &std::path::Path) -> Vec<String> {
        fn visit(root: &std::path::Path, path: &std::path::Path, found: &mut Vec<String>) {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    visit(root, &entry.path(), found);
                } else {
                    found.push(
                        entry
                            .path()
                            .strip_prefix(root)
                            .unwrap()
                            .display()
                            .to_string(),
                    );
                }
            }
        }
        let mut found = Vec::new();
        visit(path, path, &mut found);
        found.sort();
        found
    }

    fn durable_authority_mut(
        root: &mut crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV2,
    ) -> &mut crate::execution::agent_runtime::host_session_authority::store_schema::DurableSessionAuthorityV1{
        match root.session_namespace_map.get_mut("r0-session").unwrap() {
            crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) => authority.as_mut(),
            _ => panic!("fixture must retain durable authority"),
        }
    }

    #[test]
    fn reservation_uses_production_start_authority_and_publishes_no_object() {
        let (_parent, authority, observation) = started_authority();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let reserved = RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .expect("valid retained registration must reserve");

        assert_eq!(reserved.request.retained_participant_id, "r0-retained-1");
        assert!(matches!(
            reserved.request.state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
        ));
        let root = authority.read_a12a_root().unwrap();
        assert_eq!(
            root.retained_worker_registration_request_index
                .get(&reserved.request.issuer_request_id),
            Some(&reserved.request)
        );
        assert!(root.retained_worker_registration_journal.is_empty());
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn exact_reservation_retry_joins_and_changed_plan_fails_without_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let registered_at = timestamp("2026-07-14T12:02:00.000000000Z");
        let first = runtime
            .reserve_registration_at(&authority, &plan, registered_at.clone(), None)
            .unwrap();
        assert!(!first.joined);
        let fixed_root = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let fixed_objects = object_files(&object_root);

        let joined = runtime
            .reserve_registration_at(&authority, &plan, registered_at.clone(), None)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(
            joined,
            ReservedRetainedWorkerRegistrationV1 {
                joined: true,
                ..first.clone()
            }
        );
        assert_eq!(authority.read_a12a_root().unwrap(), fixed_root);

        let mut conflicts = Vec::new();
        let mut changed = plan.clone();
        changed.retained_participant_id = "r0-retained-changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.orchestration_session_id = "r0-session-changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.expected_authority.authority_revision += 1;
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.expected_authority.authority_record_commitment =
            crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
            };
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.descriptor.backend_id = "cli:changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.descriptor.protocol = "changed.protocol".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.descriptor.binary_path = "/usr/bin/changed".into();
        conflicts.push((changed, registered_at.clone()));
        let mut changed = plan.clone();
        changed.internal_uaa_session_id = "uaa-changed".into();
        conflicts.push((changed, registered_at.clone()));
        conflicts.push((plan.clone(), timestamp("2026-07-14T12:02:01.000000000Z")));
        let mut changed = plan.clone();
        changed.registration_request_id = "spawn-request-2".into();
        conflicts.push((changed, registered_at));

        for (conflict, conflict_time) in conflicts {
            assert!(runtime
                .reserve_registration_at(&authority, &conflict, conflict_time, None)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), fixed_root);
            assert_eq!(object_files(&object_root), fixed_objects);
        }
    }

    #[test]
    fn reservation_crash_windows_restart_with_zero_or_exact_reserved_state() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let registered_at = timestamp("2026-07-14T12:02:00.000000000Z");
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(runtime
            .reserve_registration_at(
                &authority,
                &plan,
                registered_at.clone(),
                Some(RetainedReservationCrashPointV1::BeforeRootPublication),
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        assert!(runtime
            .reserve_registration_at(
                &authority,
                &plan,
                registered_at.clone(),
                Some(RetainedReservationCrashPointV1::AfterRootPublication),
            )
            .is_err());
        let durable = authority.read_a12a_root().unwrap();
        let request = durable
            .retained_worker_registration_request_index
            .values()
            .next()
            .unwrap()
            .clone();
        assert!(matches!(
            request.state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
        ));
        assert_eq!(object_files(&object_root), objects_before);

        let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let joined = runtime
            .reserve_registration_at(&restarted, &plan, registered_at, None)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(joined.request, request);
        assert_eq!(restarted.read_a12a_root().unwrap(), durable);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn transition_issuer_cannot_be_reused_for_retained_registration() {
        let (_parent, authority, observation) = started_authority();
        let mut conflict = plan(observation);
        conflict.registration_request_id = "transition-start".into();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &conflict,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn strict_root_rejects_duplicate_participant_across_reserved_requests() {
        let (_parent, authority, observation) = started_authority();
        let reserved = RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let mut root = authority.read_a12a_root().unwrap();
        let mut duplicate = reserved.request;
        duplicate.issuer_request_id = "retained-worker-registration:duplicate".into();
        duplicate.registration_id = "rr_11111111111111111111111111111111".into();
        duplicate.descriptor_ref_id = "ao_11111111111111111111111111111111".into();
        duplicate.resume_handle_ref_id = "ao_22222222222222222222222222222222".into();
        duplicate.retained_worker_ref_id = "ao_33333333333333333333333333333333".into();
        root.retained_worker_registration_request_index
            .insert(duplicate.issuer_request_id.clone(), duplicate);

        assert!(root.validate().is_err());
    }

    #[test]
    fn reserved_object_identity_collision_check_is_global_across_kinds() {
        let (_parent, authority, _observation) = started_authority();
        let root = authority.read_a12a_root().unwrap();
        let policy = PolicyObjectHashInputV1 {
            schema_version: 1,
            policy_revision: "orphan-policy".into(),
            canonical_policy_snapshot_sha256:
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
        };
        let bytes = canonical_json::to_vec(&policy).unwrap();
        let reference = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
            ref_id: "ao_99999999999999999999999999999999".into(),
            object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: crate::execution::agent_runtime::host_session_authority::hash::canonical_sha256(&policy).unwrap(),
            },
        };
        let home = _parent.path().join("home");
        crate::execution::agent_runtime::host_session_authority::store::prepare_typed_object_v2_test(
            &home,
            root.root_revision,
            &reference,
            &bytes,
        )
        .unwrap();

        assert!(!crate::execution::agent_runtime::host_session_authority::store::reserved_object_ref_id_is_globally_absent_test(
            &home,
            &reference.ref_id,
        )
        .unwrap());
    }

    #[test]
    fn reserved_retry_rejects_changed_current_policy_or_world_binding() {
        let (_parent, authority, observation) = started_authority();
        let plan = plan(observation);
        RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &plan,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let root = authority.read_a12a_root().unwrap();
        let validate = |candidate: &crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV2| {
            crate::execution::agent_runtime::host_session_authority::store::retained_reservation_authority_matches_test(
                candidate,
                &plan.orchestration_session_id,
                &plan.expected_authority.authority_store_id,
                plan.expected_authority.authority_revision,
                &plan.expected_authority.authority_record_commitment,
            )
        };
        validate(&root).unwrap();

        let mut changed_world = root.clone();
        let crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) = changed_world
            .session_namespace_map
            .get_mut(&plan.orchestration_session_id)
            .unwrap()
        else {
            panic!("production Start must establish authority")
        };
        authority.world_binding.as_mut().unwrap().world_generation += 1;
        assert!(validate(&changed_world).is_err());

        let mut changed_policy = root;
        let crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) = changed_policy
            .session_namespace_map
            .get_mut(&plan.orchestration_session_id)
            .unwrap()
        else {
            panic!("production Start must establish authority")
        };
        authority.current_policy_ref.as_mut().unwrap().ref_id =
            "ao_88888888888888888888888888888888".into();
        assert!(validate(&changed_policy).is_err());
    }

    #[test]
    fn runtime_owned_object_validation_rejects_before_hsa_mutation() {
        let (_parent, authority, observation) = started_authority();
        let valid = plan(observation);
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        let mut malformed_descriptor = valid.clone();
        malformed_descriptor.descriptor.agent_id.clear();
        assert!(RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &malformed_descriptor,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        let mut malformed_resume = valid.clone();
        malformed_resume.internal_uaa_session_id.clear();
        assert!(RetainedWorkerRuntime
            .reserve_registration_at(
                &authority,
                &malformed_resume,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        let canonical = |ref_id: &str,
                         object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1| {
            crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
                ref_id: ref_id.into(),
                object_kind,
                schema_version: 1,
                commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
                },
            }
        };
        assert!(RetainedWorkerRuntime::retained_worker_bytes(
            &valid,
            &canonical(
                "ao_11111111111111111111111111111111",
                crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::AgentDescriptor,
            ),
            &canonical(
                "ao_22222222222222222222222222222222",
                crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::ResumeHandle,
            ),
            &canonical(
                "ao_33333333333333333333333333333333",
                crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy,
            ),
            &WorldBindingV1 {
                world_id: String::new(),
                world_generation: 7,
            },
        )
        .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn exact_reserved_graph_publication_writes_three_orphans_without_root_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        runtime
            .publish_reserved_object_graph(&authority, &reserved)
            .expect("exact reserved graph must publish");

        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        let objects_after = object_files(&object_root);
        assert_eq!(objects_after.len(), objects_before.len() + 3);
        assert!(objects_after
            .iter()
            .any(|path| path.ends_with(&format!("{}.obj", reserved.descriptor_ref.ref_id))));
        assert!(objects_after
            .iter()
            .any(|path| path.ends_with(&format!("{}.obj", reserved.resume_handle_ref.ref_id))));
        assert!(objects_after
            .iter()
            .any(|path| path.ends_with(&format!("{}.obj", reserved.retained_worker_ref.ref_id))));
    }

    #[test]
    fn object_publication_crash_points_restart_and_exact_join() {
        for crash_point in [
            RetainedObjectPublicationCrashPointV1::Descriptor,
            RetainedObjectPublicationCrashPointV1::ResumeHandle,
            RetainedObjectPublicationCrashPointV1::RetainedWorker,
        ] {
            let (_parent, authority, observation) = started_authority();
            let runtime = RetainedWorkerRuntime;
            let reserved = runtime
                .reserve_registration_at(
                    &authority,
                    &plan(observation),
                    timestamp("2026-07-14T12:02:00.000000000Z"),
                    None,
                )
                .unwrap();
            let root_before = authority.read_a12a_root().unwrap();
            let object_root = _parent.path().join("home/authority-v1/objects");
            let objects_before = object_files(&object_root);

            assert!(runtime
                .publish_reserved_object_graph_with_crash_point(&authority, &reserved, crash_point,)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), root_before);
            let durable_count = match crash_point {
                RetainedObjectPublicationCrashPointV1::Descriptor => 1,
                RetainedObjectPublicationCrashPointV1::ResumeHandle => 2,
                RetainedObjectPublicationCrashPointV1::RetainedWorker => 3,
            };
            assert_eq!(
                object_files(&object_root).len(),
                objects_before.len() + durable_count
            );

            let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
            runtime
                .publish_reserved_object_graph(&restarted, &reserved)
                .unwrap();
            assert_eq!(restarted.read_a12a_root().unwrap(), root_before);
            let completed = object_files(&object_root);
            assert_eq!(completed.len(), objects_before.len() + 3);
            runtime
                .publish_reserved_object_graph(&restarted, &reserved)
                .unwrap();
            assert_eq!(object_files(&object_root), completed);
            assert_eq!(restarted.read_a12a_root().unwrap(), root_before);
        }
    }

    #[test]
    fn substituted_reserved_graph_rejects_before_any_object_or_root_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let mut conflicts = Vec::new();

        let mut wrong_kind = reserved.clone();
        wrong_kind.descriptor_ref.object_kind = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy;
        conflicts.push(wrong_kind);
        let mut wrong_schema = reserved.clone();
        wrong_schema.resume_handle_ref.schema_version = 2;
        conflicts.push(wrong_schema);
        let mut wrong_bytes = reserved.clone();
        wrong_bytes.descriptor_bytes.push(b' ');
        conflicts.push(wrong_bytes);
        let mut wrong_commitment = reserved.clone();
        wrong_commitment.retained_worker_ref.commitment =
            crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
            };
        conflicts.push(wrong_commitment);

        let mut wrong_descriptor = reserved.clone();
        let mut descriptor: AgentDescriptorHashInputV1 =
            canonical_json::from_slice(&wrong_descriptor.descriptor_bytes).unwrap();
        descriptor.descriptor.backend_id = "cli:substituted".into();
        descriptor.descriptor.protocol = "substituted.protocol".into();
        wrong_descriptor.descriptor_bytes = canonical_json::to_vec(&descriptor).unwrap();
        conflicts.push(wrong_descriptor);

        let mut wrong_resume = reserved.clone();
        let mut resume: ResumeHandleHashInputV1 =
            canonical_json::from_slice(&wrong_resume.resume_handle_bytes).unwrap();
        resume.orchestration_session_id = "other-session".into();
        resume.participant_id = "other-participant".into();
        wrong_resume.resume_handle_bytes = canonical_json::to_vec(&resume).unwrap();
        conflicts.push(wrong_resume);

        let mut wrong_worker = reserved.clone();
        let mut worker: RetainedWorkerObjectHashInputV1 =
            canonical_json::from_slice(&wrong_worker.retained_worker_bytes).unwrap();
        worker.world_binding.world_id = "other-world".into();
        worker.world_binding.world_generation += 1;
        worker.policy_ref.ref_id = "ao_77777777777777777777777777777777".into();
        worker.descriptor_ref.ref_id = "ao_66666666666666666666666666666666".into();
        worker.resume_handle_ref.ref_id = "ao_55555555555555555555555555555555".into();
        wrong_worker.retained_worker_bytes = canonical_json::to_vec(&worker).unwrap();
        conflicts.push(wrong_worker);

        let mut wrong_request_scope = reserved.clone();
        wrong_request_scope.request.world_binding.world_id = "other-world".into();
        wrong_request_scope.request.current_policy_ref.ref_id =
            "ao_44444444444444444444444444444444".into();
        conflicts.push(wrong_request_scope);

        for conflict in conflicts {
            assert!(runtime
                .publish_reserved_object_graph(&authority, &conflict)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), root_before);
            assert_eq!(object_files(&object_root), objects_before);
        }
    }

    #[test]
    fn atomic_registration_appends_only_the_reserved_authority_link() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let before = authority.read_a12a_root().unwrap();
        let before_authority = match before.session_namespace_map.get("r0-session").unwrap() {
            crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) => authority.as_ref().clone(),
            _ => panic!("production Start must establish durable authority"),
        };
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        let result = runtime
            .register_retained_target(&authority, &plan)
            .expect("exact retained registration must apply atomically");

        let after = authority.read_a12a_root().unwrap();
        let request = after
            .retained_worker_registration_request_index
            .get("retained-worker-registration:spawn-request-1")
            .unwrap();
        let journal = after
            .retained_worker_registration_journal
            .get(&request.registration_id)
            .unwrap();
        let after_authority = match after.session_namespace_map.get("r0-session").unwrap() {
            crate::execution::agent_runtime::host_session_authority::store_schema::SessionNamespaceRecordV1::Authority(authority) => authority.as_ref(),
            _ => panic!("retained registration must preserve durable authority"),
        };

        assert_eq!(after.root_revision, before.root_revision + 2);
        assert_eq!(after.transition_intent_map, before.transition_intent_map);
        assert_eq!(after.issuer_request_index, before.issuer_request_index);
        assert_eq!(after.application_journal, before.application_journal);
        assert_eq!(
            after_authority.authority_revision,
            before_authority.authority_revision + 1
        );
        assert_eq!(
            after_authority.authoritative_participant_lineage,
            [
                before_authority
                    .authoritative_participant_lineage
                    .as_slice(),
                std::slice::from_ref(&plan.retained_participant_id),
            ]
            .concat()
        );
        assert_eq!(
            after_authority.retained_worker_refs,
            [
                before_authority.retained_worker_refs.as_slice(),
                std::slice::from_ref(&journal.retained_worker_ref),
            ]
            .concat()
        );
        let mut expected_authority = before_authority.clone();
        expected_authority.authority_revision += 1;
        expected_authority
            .authoritative_participant_lineage
            .push(plan.retained_participant_id.clone());
        expected_authority
            .retained_worker_refs
            .push(journal.retained_worker_ref.clone());
        expected_authority.updated_at = request.registered_at.clone();
        assert_eq!(*after_authority, expected_authority);
        assert!(matches!(
            &request.state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                authority_revision_after,
                authority_record_commitment_after,
            } if *authority_revision_after == after_authority.authority_revision
                && authority_record_commitment_after == &journal.authority_record_commitment_after
        ));
        assert_eq!(
            journal.authority_revision_before,
            before_authority.authority_revision
        );
        assert_eq!(
            journal.authority_revision_after,
            after_authority.authority_revision
        );
        assert_eq!(journal.registered_at, request.registered_at);
        assert_eq!(after.object_index.len(), before.object_index.len() + 3);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
        assert_eq!(result.registration_id, request.registration_id);
        assert_eq!(result.retained_participant_id, plan.retained_participant_id);
        assert_eq!(
            result.authority_revision_after,
            after_authority.authority_revision
        );
    }

    #[test]
    fn final_root_crash_windows_restart_and_join_one_application() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &reserved)
            .unwrap();
        let reserved_root = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let durable_objects = object_files(&object_root);

        assert!(authority
            .apply_reserved_retained_worker_registration_with_crash_point(
                &reserved,
                RetainedApplicationCrashPointV1::BeforeRootPublication,
            )
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), reserved_root);
        assert_eq!(object_files(&object_root), durable_objects);

        let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let applied = restarted
            .apply_reserved_retained_worker_registration(&reserved)
            .unwrap();
        assert!(!applied.joined);
        let applied_root = restarted.read_a12a_root().unwrap();
        let joined = restarted
            .apply_reserved_retained_worker_registration(&reserved)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(joined.registration, applied.registration);
        assert_eq!(restarted.read_a12a_root().unwrap(), applied_root);
        assert_eq!(object_files(&object_root), durable_objects);

        let (_parent, authority, observation) = started_authority();
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &reserved)
            .unwrap();
        assert!(authority
            .apply_reserved_retained_worker_registration_with_crash_point(
                &reserved,
                RetainedApplicationCrashPointV1::AfterRootPublication,
            )
            .is_err());
        let applied_root = authority.read_a12a_root().unwrap();
        assert!(matches!(
            applied_root
                .retained_worker_registration_request_index
                .get(&reserved.request.issuer_request_id)
                .unwrap()
                .state,
            crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Applied { .. }
        ));
        let restarted = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let joined = restarted
            .apply_reserved_retained_worker_registration(&reserved)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(restarted.read_a12a_root().unwrap(), applied_root);
    }

    #[test]
    fn application_rejects_missing_reserved_object_without_root_or_object_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let reserved = runtime
            .reserve_registration_at(
                &authority,
                &plan(observation),
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        authority
            .publish_reserved_retained_object(
                &reserved,
                &reserved.descriptor_ref,
                &reserved.descriptor_bytes,
            )
            .unwrap();
        authority
            .publish_reserved_retained_object(
                &reserved,
                &reserved.resume_handle_ref,
                &reserved.resume_handle_bytes,
            )
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(authority
            .apply_reserved_retained_worker_registration(&reserved)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn stale_parallel_reservation_rejects_application_with_zero_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = plan(observation.clone());
        let mut second_plan = plan(observation);
        second_plan.registration_request_id = "spawn-request-2".into();
        second_plan.retained_participant_id = "r0-retained-2".into();
        second_plan.descriptor.agent_id = "codex-worker-2".into();
        second_plan.internal_uaa_session_id = "uaa-retained-2".into();
        let first = runtime
            .reserve_registration_at(
                &authority,
                &first_plan,
                timestamp("2026-07-14T12:02:00.000000000Z"),
                None,
            )
            .unwrap();
        let stale = runtime
            .reserve_registration_at(
                &authority,
                &second_plan,
                timestamp("2026-07-14T12:02:01.000000000Z"),
                None,
            )
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &first)
            .unwrap();
        runtime
            .publish_reserved_object_graph(&authority, &stale)
            .unwrap();
        authority
            .apply_reserved_retained_worker_registration(&first)
            .unwrap();
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);

        assert!(authority
            .apply_reserved_retained_worker_registration(&stale)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);
    }

    #[test]
    fn strict_root_rejects_incomplete_or_noncontiguous_applied_registration() {
        let (_parent, authority, observation) = started_authority();
        RetainedWorkerRuntime
            .register_retained_target(&authority, &plan(observation))
            .unwrap();
        let applied = authority.read_a12a_root().unwrap();
        applied.validate().unwrap();
        let request = applied
            .retained_worker_registration_request_index
            .values()
            .next()
            .unwrap()
            .clone();

        let mut missing_journal = applied.clone();
        missing_journal.retained_worker_registration_journal.clear();
        assert!(missing_journal.validate().is_err());

        let mut reserved_with_journal = applied.clone();
        reserved_with_journal
            .retained_worker_registration_request_index
            .get_mut(&request.issuer_request_id)
            .unwrap()
            .state = crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved;
        assert!(reserved_with_journal.validate().is_err());

        let mut journal_without_request = applied.clone();
        journal_without_request
            .retained_worker_registration_request_index
            .clear();
        assert!(journal_without_request.validate().is_err());

        let mut missing_index = applied.clone();
        missing_index
            .object_index
            .remove(&request.retained_worker_ref_id);
        assert!(missing_index.validate().is_err());

        let mut duplicate_lineage = applied.clone();
        durable_authority_mut(&mut duplicate_lineage)
            .authoritative_participant_lineage
            .push(request.retained_participant_id.clone());
        assert!(duplicate_lineage.validate().is_err());

        let mut duplicate_worker_ref = applied.clone();
        let worker_ref =
            durable_authority_mut(&mut duplicate_worker_ref).retained_worker_refs[0].clone();
        durable_authority_mut(&mut duplicate_worker_ref)
            .retained_worker_refs
            .push(worker_ref);
        assert!(duplicate_worker_ref.validate().is_err());

        let mut skipped_revision = applied.clone();
        durable_authority_mut(&mut skipped_revision).authority_revision += 1;
        assert!(skipped_revision.validate().is_err());

        let mut proof_ahead_of_authority = applied.clone();
        durable_authority_mut(&mut proof_ahead_of_authority).authority_revision = 1;
        assert!(proof_ahead_of_authority.validate().is_err());

        let mut forged_lineage_commitment = applied;
        forged_lineage_commitment
            .retained_worker_registration_journal
            .get_mut(&request.registration_id)
            .unwrap()
            .authoritative_lineage_commitment_after = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
            };
        assert!(forged_lineage_commitment.validate().is_err());
    }

    #[test]
    fn lost_response_whole_operation_retry_joins_without_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let first = runtime.register_retained_target(&authority, &plan).unwrap();
        let applied_root = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let applied_objects = object_files(&object_root);

        let joined = runtime
            .register_retained_target(&authority, &plan)
            .expect("lost response retry must join the Applied registration");

        assert_eq!(joined, first);
        assert_eq!(authority.read_a12a_root().unwrap(), applied_root);
        assert_eq!(object_files(&object_root), applied_objects);
    }

    #[test]
    fn applied_retry_rejects_changed_authority_store_without_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        runtime.register_retained_target(&authority, &plan).unwrap();
        let applied_root = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let applied_objects = object_files(&object_root);
        let mut changed_store = plan;
        changed_store.expected_authority.authority_store_id =
            "as_11111111111111111111111111111111".into();

        assert!(runtime
            .register_retained_target(&authority, &changed_store)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), applied_root);
        assert_eq!(object_files(&object_root), applied_objects);
    }

    #[test]
    fn reserved_observation_joins_peer_application_during_publication() {
        let (_parent, authority, observation) = started_authority();
        let peer_authority = HostSessionAuthority::open(&_parent.path().join("home")).unwrap();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let mut peer_result = None;

        let joined = runtime
            .register_retained_target_with(&authority, &plan, |_| {
                peer_result = Some(runtime.register_retained_target(&peer_authority, &plan)?);
                Ok(())
            })
            .expect("a Reserved observation must join a peer that reaches Applied first");

        assert_eq!(Some(joined), peer_result);
        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_request_index.len(), 1);
        assert_eq!(root.retained_worker_registration_journal.len(), 1);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
    }

    #[test]
    fn exact_resolution_rejects_substituted_target_without_mutation() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = plan(observation);
        let target = runtime.register_retained_target(&authority, &plan).unwrap();
        let resolved = runtime
            .resolve_retained_target(&authority, &target)
            .unwrap();
        assert_eq!(
            resolved.registration.registration_id,
            target.registration_id
        );
        assert_eq!(resolved.descriptor, plan.descriptor);
        assert_eq!(
            resolved.resume_handle.participant_id,
            target.retained_participant_id
        );
        assert_eq!(
            resolved.retained_worker.participant_id,
            target.retained_participant_id
        );
        assert_eq!(resolved.retained_worker.world_binding.world_id, "r0-world");
        assert_eq!(resolved.current_policy.policy_revision, "r0-policy");
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let mut conflicts = Vec::new();

        let mut conflict = target.clone();
        conflict.authority_store_id = "as_11111111111111111111111111111111".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.orchestration_session_id = "other-session".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.registration_id = "rr_11111111111111111111111111111111".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.retained_participant_id = "other-participant".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.retained_worker_ref.ref_id = "ao_11111111111111111111111111111111".into();
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.authority_revision_after += 1;
        conflicts.push(conflict);
        let mut conflict = target.clone();
        conflict.authority_record_commitment_after = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
        };
        conflicts.push(conflict);

        for conflict in conflicts {
            assert!(runtime
                .resolve_retained_target(&authority, &conflict)
                .is_err());
            assert_eq!(authority.read_a12a_root().unwrap(), root_before);
            assert_eq!(object_files(&object_root), objects_before);
        }
    }

    #[test]
    fn unreserved_orphan_object_never_grants_retained_authority() {
        let (_parent, authority, observation) = started_authority();
        let descriptor = AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: plan(observation.clone()).descriptor,
        };
        let bytes = canonical_json::to_vec(&descriptor).unwrap();
        let reference = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
            ref_id: "ao_88888888888888888888888888888888".into(),
            object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::AgentDescriptor,
            schema_version: 1,
            commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: crate::execution::agent_runtime::host_session_authority::hash::canonical_sha256(&descriptor).unwrap(),
            },
        };
        let root_before = authority.read_a12a_root().unwrap();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        crate::execution::agent_runtime::host_session_authority::store::prepare_typed_object_v2_test(
            &_parent.path().join("home"),
            root_before.root_revision,
            &reference,
            &bytes,
        )
        .unwrap();
        let root_after = authority.read_a12a_root().unwrap();
        assert_eq!(root_after, root_before);
        assert!(root_after
            .retained_worker_registration_request_index
            .is_empty());
        assert!(root_after.retained_worker_registration_journal.is_empty());
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 1);
        let fake_target = RetainedWorkerRegistrationResultV1 {
            registration_id: "rr_88888888888888888888888888888888".into(),
            authority_store_id: observation.authority_store_id,
            orchestration_session_id: observation.orchestration_session_id,
            retained_participant_id: "orphan-participant".into(),
            retained_worker_ref: reference,
            authority_revision_after: observation.authority_revision,
            authority_record_commitment_after: observation.authority_record_commitment,
        };
        assert!(RetainedWorkerRuntime
            .resolve_retained_target(&authority, &fake_target)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 1);
    }

    #[test]
    fn two_retained_links_preserve_pending_start_and_resolve_exactly() {
        let (_parent, authority, observation) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = plan(observation);
        let first = runtime
            .register_retained_target(&authority, &first_plan)
            .unwrap();
        let observation = authority
            .resolve_current_exact("r0-session", None)
            .unwrap()
            .observation;
        let mut second_plan = plan(observation);
        second_plan.registration_request_id = "spawn-request-2".into();
        second_plan.retained_participant_id = "r0-retained-2".into();
        second_plan.descriptor.agent_id = "codex-worker-2".into();
        second_plan.internal_uaa_session_id = "uaa-retained-2".into();
        let second = runtime
            .register_retained_target(&authority, &second_plan)
            .unwrap();

        let first_resolved = runtime.resolve_retained_target(&authority, &first).unwrap();
        let second_resolved = runtime
            .resolve_retained_target(&authority, &second)
            .unwrap();
        assert_eq!(first_resolved.current_authority_revision, 3);
        assert_eq!(second_resolved.current_authority_revision, 3);
        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_journal.len(), 2);
        let HostSessionTransitionIntentStateV2::Applied {
            startup_ownership, ..
        } = &root.transition_intent_map["r0-start-intent"].state
        else {
            panic!("production Start must remain Applied")
        };
        assert!(matches!(
            startup_ownership.as_ref(),
            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionStartupOwnershipApplicationV1::Pending {
                expected_authority_revision: 1,
                ..
            }
        ));
        let object_root = _parent.path().join("home/authority-v1/objects");
        let root_before = root.clone();
        let objects_before = object_files(&object_root);
        let observation = authority
            .resolve_current_exact("r0-session", None)
            .unwrap()
            .observation;
        let mut duplicate = plan(observation);
        duplicate.registration_request_id = "spawn-request-duplicate".into();
        assert!(runtime
            .register_retained_target(&authority, &duplicate)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_before);
        assert_eq!(object_files(&object_root), objects_before);

        let mut reused_ref = root_before;
        let registrations = reused_ref
            .retained_worker_registration_journal
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let first_ref = registrations[0].retained_worker_ref.clone();
        let second_id = registrations[1].registration_id.clone();
        reused_ref
            .retained_worker_registration_journal
            .get_mut(&second_id)
            .unwrap()
            .retained_worker_ref = first_ref;
        assert!(reused_ref.validate().is_err());
    }

    const RETAINED_SUBPROCESS_HOME: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_HOME";
    const RETAINED_SUBPROCESS_STORE: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_STORE";
    const RETAINED_SUBPROCESS_COMMITMENT: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_COMMITMENT";
    const RETAINED_SUBPROCESS_VARIANT: &str = "SUBSTRATE_R0_RETAINED_SUBPROCESS_VARIANT";
    const ADMISSION_INIT_SUBPROCESS_HOME: &str = "SUBSTRATE_B3_2A_ADMISSION_INIT_SUBPROCESS_HOME";
    const ADMISSION_RESERVATION_SUBPROCESS_HOME: &str =
        "SUBSTRATE_B3_2A_ADMISSION_RESERVATION_SUBPROCESS_HOME";
    const ADMISSION_RESERVATION_SUBPROCESS_ISSUER: &str =
        "SUBSTRATE_B3_2A_ADMISSION_RESERVATION_SUBPROCESS_ISSUER";
    const ADMISSION_RESERVATION_SUBPROCESS_PROMPT: &str =
        "SUBSTRATE_B3_2A_ADMISSION_RESERVATION_SUBPROCESS_PROMPT";
    const ADMISSION_RESERVATION_SUBPROCESS_CAP: &str =
        "SUBSTRATE_B3_2A_ADMISSION_RESERVATION_SUBPROCESS_CAP";

    #[test]
    fn retained_registration_subprocess_worker() {
        let Some(home) = std::env::var_os(RETAINED_SUBPROCESS_HOME) else {
            return;
        };
        let authority = HostSessionAuthority::open(std::path::Path::new(&home)).unwrap();
        let current = (0..32)
            .find_map(|_| {
                let resolved = authority.resolve_current_exact("r0-session", None).ok();
                if resolved.is_none() {
                    std::thread::yield_now();
                }
                resolved
            })
            .expect("subprocess must acquire a stable authority snapshot")
            .observation;
        let mut plan = plan(current);
        plan.expected_authority.authority_store_id =
            std::env::var(RETAINED_SUBPROCESS_STORE).unwrap();
        plan.expected_authority.authority_revision = 1;
        plan.expected_authority.authority_record_commitment = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: std::env::var(RETAINED_SUBPROCESS_COMMITMENT).unwrap(),
        };
        if std::env::var_os(RETAINED_SUBPROCESS_VARIANT).as_deref()
            == Some(std::ffi::OsStr::new("conflict"))
        {
            plan.retained_participant_id = "r0-retained-conflict".into();
            plan.descriptor.agent_id = "codex-worker-conflict".into();
            plan.internal_uaa_session_id = "uaa-retained-conflict".into();
        }
        RetainedWorkerRuntime
            .register_retained_target(&authority, &plan)
            .unwrap();
    }

    #[test]
    fn admission_initialization_subprocess_worker() {
        let Some(home) = std::env::var_os(ADMISSION_INIT_SUBPROCESS_HOME) else {
            return;
        };
        let authority = HostSessionAuthority::open(std::path::Path::new(&home)).unwrap();
        RetainedWorkerRuntime
            .initialize_admission_registry(&authority)
            .unwrap();
    }

    #[test]
    fn admission_reservation_subprocess_worker() {
        let Some(home) = std::env::var_os(ADMISSION_RESERVATION_SUBPROCESS_HOME) else {
            return;
        };
        let authority = HostSessionAuthority::open(std::path::Path::new(&home)).unwrap();
        let issuer = std::env::var(ADMISSION_RESERVATION_SUBPROCESS_ISSUER).unwrap();
        let prompt = std::env::var(ADMISSION_RESERVATION_SUBPROCESS_PROMPT).unwrap();
        let cap = std::env::var(ADMISSION_RESERVATION_SUBPROCESS_CAP)
            .unwrap()
            .parse()
            .unwrap();
        let plan = admission_plan(&authority, &issuer, &prompt, cap);
        let _ = RetainedWorkerRuntime.reserve_admission_slot(&authority, &plan);
    }

    #[test]
    fn two_process_identical_registration_converges_to_one_authority_link() {
        let (_parent, authority, observation) = started_authority();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex,
        } = &observation.authority_record_commitment
        else {
            panic!("durable authority commitment must be canonical")
        };
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::retained_registration_subprocess_worker";
        let spawn = || {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(RETAINED_SUBPROCESS_HOME, _parent.path().join("home"))
                .env(RETAINED_SUBPROCESS_STORE, &observation.authority_store_id)
                .env(RETAINED_SUBPROCESS_COMMITMENT, digest_hex)
                .spawn()
                .unwrap()
        };
        let mut first = spawn();
        let mut second = spawn();
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());

        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_request_index.len(), 1);
        assert_eq!(root.retained_worker_registration_journal.len(), 1);
        let SessionNamespaceRecordV1::Authority(authority_record) =
            &root.session_namespace_map["r0-session"]
        else {
            panic!("retained registration must preserve authority")
        };
        assert_eq!(authority_record.authority_revision, 2);
        assert_eq!(authority_record.retained_worker_refs.len(), 1);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
        assert_eq!(
            authority_record.authoritative_participant_lineage,
            vec!["r0-orchestrator".to_owned(), "r0-retained-1".to_owned()]
        );
    }

    #[test]
    fn two_process_conflicting_registration_has_one_winner_and_zero_partial_authority() {
        let (_parent, authority, observation) = started_authority();
        let object_root = _parent.path().join("home/authority-v1/objects");
        let objects_before = object_files(&object_root);
        let crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex,
        } = &observation.authority_record_commitment
        else {
            panic!("durable authority commitment must be canonical")
        };
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::retained_registration_subprocess_worker";
        let spawn = |variant: &str| {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(RETAINED_SUBPROCESS_HOME, _parent.path().join("home"))
                .env(RETAINED_SUBPROCESS_STORE, &observation.authority_store_id)
                .env(RETAINED_SUBPROCESS_COMMITMENT, digest_hex)
                .env(RETAINED_SUBPROCESS_VARIANT, variant)
                .spawn()
                .unwrap()
        };
        let mut first = spawn("exact");
        let mut second = spawn("conflict");
        let first_success = first.wait().unwrap().success();
        let second_success = second.wait().unwrap().success();
        assert_ne!(first_success, second_success);

        let root = authority.read_a12a_root().unwrap();
        assert_eq!(root.retained_worker_registration_request_index.len(), 1);
        assert_eq!(root.retained_worker_registration_journal.len(), 1);
        let SessionNamespaceRecordV1::Authority(authority_record) =
            &root.session_namespace_map["r0-session"]
        else {
            panic!("retained registration must preserve authority")
        };
        assert_eq!(authority_record.authority_revision, 2);
        assert_eq!(authority_record.retained_worker_refs.len(), 1);
        assert_eq!(authority_record.authoritative_participant_lineage.len(), 2);
        assert_eq!(object_files(&object_root).len(), objects_before.len() + 3);
    }

    #[test]
    fn admission_key_initialization_is_private_and_exactly_joined() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;

        let first = runtime.initialize_admission_registry(&authority).unwrap();
        let joined = runtime.initialize_admission_registry(&authority).unwrap();

        assert_eq!(joined, first);
        let admission_root = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1");
        let key_path = admission_root
            .join("keys")
            .join(format!("{}.key", first.key_id));
        let key_mode = fs::metadata(key_path).unwrap().permissions().mode() & 0o777;
        let registry_mode = fs::metadata(admission_root.join("registry-v1.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(key_mode, 0o600);
        assert_eq!(registry_mode, 0o600);
    }

    #[test]
    fn admission_key_orphan_after_key_publication_is_reconciled_before_reinitialization() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let created_at = timestamp("2026-07-15T12:00:00.000000000Z");
        let failure = runtime
            .initialize_admission_registry_at(
                &authority,
                created_at.clone(),
                [1_u8; 16],
                [2_u8; 16],
                [3_u8; 32],
                Some(AdmissionInitializationCrashPointV1::AfterKeyPublication),
            )
            .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "injected crash after admission key publication"
        );

        let admission_root = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1");
        assert!(!admission_root.join("registry-v1.json").exists());
        assert_eq!(
            fs::read_dir(admission_root.join("keys")).unwrap().count(),
            1
        );

        let joined = runtime
            .initialize_admission_registry_at(
                &authority, created_at, [4_u8; 16], [5_u8; 16], [6_u8; 32], None,
            )
            .unwrap();
        let key_names = fs::read_dir(admission_root.join("keys"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(key_names, vec![format!("{}.key", joined.key_id)]);
    }

    #[test]
    fn admission_key_temp_before_publication_is_reconciled_before_reinitialization() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let failure = runtime
            .initialize_admission_registry_at(
                &authority,
                timestamp("2026-07-15T12:01:00.000000000Z"),
                [7_u8; 16],
                [8_u8; 16],
                [9_u8; 32],
                Some(AdmissionInitializationCrashPointV1::BeforeKeyPublication),
            )
            .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "injected crash before admission key publication"
        );
        let admission_root = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1");
        assert_eq!(
            fs::read_dir(admission_root.join("keys")).unwrap().count(),
            1
        );
        let staged_name = fs::read_dir(admission_root.join("keys"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name()
            .into_string()
            .unwrap();
        assert!(staged_name.starts_with("admission-key--"));
        assert!(staged_name.ends_with(".tmp"));
        assert_eq!(fs::read_dir(admission_root.join("tmp")).unwrap().count(), 0);
        assert!(!admission_root.join("registry-v1.json").exists());

        let initialized = runtime.initialize_admission_registry(&authority).unwrap();
        assert_eq!(fs::read_dir(admission_root.join("tmp")).unwrap().count(), 0);
        assert_eq!(
            fs::read_dir(admission_root.join("keys")).unwrap().count(),
            1
        );
        assert!(admission_root
            .join("keys")
            .join(format!("{}.key", initialized.key_id))
            .exists());
    }

    #[test]
    fn concurrent_process_admission_key_initialization_exactly_joins_one_key() {
        let (parent, authority, _) = started_authority();
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::admission_initialization_subprocess_worker";
        let spawn = || {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(ADMISSION_INIT_SUBPROCESS_HOME, parent.path().join("home"))
                .spawn()
                .unwrap()
        };
        let mut first = spawn();
        let mut second = spawn();
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());

        let joined = RetainedWorkerRuntime
            .initialize_admission_registry(&authority)
            .unwrap();
        let admission_root = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1");
        let key_names = fs::read_dir(admission_root.join("keys"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(key_names, vec![format!("{}.key", joined.key_id)]);
        assert_eq!(fs::read_dir(admission_root.join("tmp")).unwrap().count(), 0);
    }

    fn initialized_admission_paths(
        parent: &tempfile::TempDir,
        identity: &RetainedWorkerAdmissionKeyIdentityV1,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let admission_root = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1");
        let key_path = admission_root
            .join("keys")
            .join(format!("{}.key", identity.key_id));
        (admission_root, key_path)
    }

    #[test]
    fn committed_admission_key_missing_fails_closed_without_registry_mutation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (admission_root, key_path) = initialized_admission_paths(&parent, &identity);
        let registry_before = fs::read(admission_root.join("registry-v1.json")).unwrap();
        fs::remove_file(&key_path).unwrap();

        assert!(runtime.initialize_admission_registry(&authority).is_err());
        assert!(!key_path.exists());
        assert_eq!(
            fs::read(admission_root.join("registry-v1.json")).unwrap(),
            registry_before
        );
    }

    #[test]
    fn committed_admission_key_malformed_fails_closed_without_repair() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (_, key_path) = initialized_admission_paths(&parent, &identity);
        fs::write(&key_path, b"malformed-admission-key").unwrap();
        let malformed = fs::read(&key_path).unwrap();

        assert!(runtime.initialize_admission_registry(&authority).is_err());
        assert_eq!(fs::read(key_path).unwrap(), malformed);
    }

    #[test]
    fn committed_admission_key_wrong_store_fails_closed_without_repair() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (_, key_path) = initialized_admission_paths(&parent, &identity);
        let mut envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(&key_path).unwrap(), "decode fixture key").unwrap();
        envelope.authority_store_id = "as_00000000000000000000000000000000".into();
        let substituted = encode_canonical(&envelope, "encode fixture key").unwrap();
        fs::write(&key_path, &substituted).unwrap();

        assert!(runtime.initialize_admission_registry(&authority).is_err());
        assert_eq!(fs::read(key_path).unwrap(), substituted);
    }

    #[test]
    fn committed_admission_key_wrong_key_identity_fails_closed_without_repair() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (_, key_path) = initialized_admission_paths(&parent, &identity);
        let mut envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(&key_path).unwrap(), "decode fixture key").unwrap();
        envelope.key_id = "adk_00000000000000000000000000000000".into();
        let substituted = encode_canonical(&envelope, "encode fixture key").unwrap();
        fs::write(&key_path, &substituted).unwrap();

        assert!(runtime.initialize_admission_registry(&authority).is_err());
        assert_eq!(fs::read(key_path).unwrap(), substituted);
    }

    #[test]
    fn committed_admission_key_unreadable_mode_fails_closed_without_repair() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (_, key_path) = initialized_admission_paths(&parent, &identity);
        fs::set_permissions(&key_path, fs::Permissions::from_mode(0o000)).unwrap();

        assert!(runtime.initialize_admission_registry(&authority).is_err());
        assert_eq!(
            fs::metadata(key_path).unwrap().permissions().mode() & 0o777,
            0
        );
    }

    #[test]
    fn committed_admission_key_symlink_is_rejected_without_following_target() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (_, key_path) = initialized_admission_paths(&parent, &identity);
        let target = parent.path().join("outside-key-target");
        fs::write(&target, b"outside").unwrap();
        fs::remove_file(&key_path).unwrap();
        std::os::unix::fs::symlink(&target, &key_path).unwrap();

        assert!(runtime.initialize_admission_registry(&authority).is_err());
        assert_eq!(fs::read(target).unwrap(), b"outside");
        assert!(fs::symlink_metadata(key_path)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    fn admission_plan(
        authority: &HostSessionAuthority,
        issuer_request_id: &str,
        prompt: &str,
        max_live_retained_workers: u64,
    ) -> RetainedWorkerAdmissionPlanV1 {
        let resolved = authority.resolve_current_exact("r0-session", None).unwrap();
        let descriptor = plan(resolved.observation.clone()).descriptor;
        let target_backend_id = descriptor.backend_id.clone();
        let bootstrap_home = authority
            .bootstrap_home()
            .identity()
            .unwrap()
            .physical_path
            .clone();
        fs::write(
            std::path::Path::new(&bootstrap_home).join("policy.yaml"),
            format!(
                "agents:\n  allowed_backends:\n    - \"{target_backend_id}\"\n  world_dispatch:\n    enabled: true\n    allowed_backends:\n      - \"{target_backend_id}\"\n    allowed_actions:\n      - \"spawn_world_worker\"\n    allowed_modes:\n      - \"retained\"\n    same_session_only: true\n    same_world_binding_only: true\n    allow_capability_narrowing: false\n    max_live_retained_workers: {max_live_retained_workers}\n    max_concurrent_ephemeral: 4\n"
            ),
        )
        .unwrap();
        RetainedWorkerAdmissionPlanV1 {
            issuer_request_id: issuer_request_id.into(),
            spawn_request: CanonicalValidatedSpawnRequestV1 {
                schema_version: 1,
                request_id: format!("request-{issuer_request_id}"),
                idempotency_key: format!("idempotency-{issuer_request_id}"),
                orchestration_session_id: "r0-session".into(),
                caller_participant_id: "r0-orchestrator".into(),
                action: "spawn_world_worker".into(),
                mode: "retained".into(),
                target_backend_id: target_backend_id.clone(),
                task_run_id: None,
                target_participant_id: None,
                world_id: "r0-world".into(),
                world_generation: 7,
                payload: CanonicalWorkerSpawnPayloadV1 {
                    prompt: prompt.into(),
                },
            },
            exact_authority: CanonicalExactCurrentAuthorityV1::from_resolved(&resolved),
            descriptor_and_runtime_plan: CanonicalDescriptorAndRuntimePlanV1 {
                schema_version: 1,
                descriptor,
                runtime_role: "member".into(),
                internal_uaa_session_id_domain: "substrate.retained-worker.internal-uaa-session.v1"
                    .into(),
            },
            policy_and_admission_cap: CanonicalPolicyAndAdmissionCapV1 {
                schema_version: 1,
                current_policy_ref: resolved
                    .authority
                    .current_policy_ref
                    .clone()
                    .expect("started authority must bind policy"),
                current_policy: resolved.current_policy.clone(),
                dispatch_enabled: true,
                allowed_backends: vec![target_backend_id],
                allowed_actions: vec!["spawn_world_worker".into()],
                allowed_modes: vec!["retained".into()],
                same_session_only: true,
                same_world_binding_only: true,
                allow_capability_narrowing: false,
                max_live_retained_workers,
                max_concurrent_ephemeral: 4,
            },
        }
    }

    #[test]
    fn admission_cap_zero_rejects_without_slot_or_authority_mutation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let authority_before = authority.read_a12a_root().unwrap();
        let plan = admission_plan(&authority, "cap-zero", "prompt", 0);

        assert!(runtime.reserve_admission_slot(&authority, &plan).is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), authority_before);
        let registry = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let decoded: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&fs::read(registry).unwrap(), "decode fixture registry").unwrap();
        assert!(decoded.records_by_session.is_empty());
    }

    #[test]
    fn admission_cap_boundary_counts_slot_reserved_and_rejects_second_request() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "cap-one-first", "first", 1);
        let first = runtime
            .reserve_admission_slot(&authority, &first_plan)
            .unwrap();
        let root_after_first = authority.read_a12a_root().unwrap();

        let second_plan = admission_plan(&authority, "cap-one-second", "second", 1);
        assert!(runtime
            .reserve_admission_slot(&authority, &second_plan)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), root_after_first);
        assert!(matches!(
            first.record.state,
            RetainedWorkerAdmissionStateV1::SlotReserved {
                slot_sequence: 1,
                ..
            }
        ));
    }

    #[test]
    fn admission_exact_retry_joins_stable_identity_and_changed_prompt_conflicts() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "exact-retry", "original prompt", 1);
        let first = runtime.reserve_admission_slot(&authority, &plan).unwrap();
        let joined = runtime.reserve_admission_slot(&authority, &plan).unwrap();
        assert!(!first.joined);
        assert!(joined.joined);
        assert_eq!(joined.record, first.record);

        let changed = admission_plan(&authority, "exact-retry", "changed prompt", 1);
        assert!(runtime
            .reserve_admission_slot(&authority, &changed)
            .is_err());
        let inspection = runtime
            .read_admission_record(
                &authority,
                &first.record.orchestration_session_id,
                &first.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert_eq!(inspection, first.record);
    }

    #[test]
    fn admission_retry_rejects_every_changed_bound_input_without_mutation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "changed-input", "original prompt", 3);
        let first = runtime.reserve_admission_slot(&authority, &plan).unwrap();
        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let registry_before = fs::read(&registry_path).unwrap();
        let authority_before = authority.read_a12a_root().unwrap();

        let mut variants = Vec::new();
        let mut changed = plan.clone();
        changed.spawn_request.payload.prompt = "changed prompt".into();
        variants.push(changed);
        let mut changed = plan.clone();
        changed.spawn_request.request_id = "changed-request".into();
        variants.push(changed);
        let mut changed = plan.clone();
        changed.spawn_request.idempotency_key = "changed-idempotency".into();
        variants.push(changed);
        let mut changed = plan.clone();
        changed.spawn_request.caller_participant_id = "changed-caller".into();
        variants.push(changed);
        let mut changed = plan.clone();
        changed.spawn_request.orchestration_session_id = "changed-session".into();
        variants.push(changed);
        let mut changed = plan.clone();
        changed.spawn_request.world_id = "changed-world".into();
        variants.push(changed);
        let mut changed = plan.clone();
        changed.spawn_request.world_generation += 1;
        variants.push(changed);
        let mut changed = plan.clone();
        changed.descriptor_and_runtime_plan.descriptor.backend_id = "cli:changed-worker".into();
        changed.spawn_request.target_backend_id = "cli:changed-worker".into();
        changed.policy_and_admission_cap.allowed_backends = vec!["cli:changed-worker".into()];
        variants.push(changed);
        let mut changed = plan.clone();
        changed.descriptor_and_runtime_plan.descriptor.protocol = "substrate.changed".into();
        variants.push(changed);
        let mut changed = plan.clone();
        changed.policy_and_admission_cap.max_live_retained_workers = 4;
        variants.push(changed);
        let mut changed = plan.clone();
        changed.policy_and_admission_cap.max_concurrent_ephemeral += 1;
        variants.push(changed);
        let mut changed = plan.clone();
        changed.policy_and_admission_cap.dispatch_enabled = false;
        variants.push(changed);
        let mut changed = plan.clone();
        changed.exact_authority.authority_record_commitment =
            AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "00".repeat(32),
            };
        variants.push(changed);
        let mut changed = plan.clone();
        changed.exact_authority.authoritative_lineage_commitment =
            AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "11".repeat(32),
            };
        variants.push(changed);
        let mut changed = plan.clone();
        changed
            .exact_authority
            .authority
            .workspace_binding
            .workspace_root
            .physical_path = "/substituted-workspace".into();
        variants.push(changed);

        for changed in variants {
            assert!(runtime
                .reserve_admission_slot(&authority, &changed)
                .is_err());
            assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
            assert_eq!(authority.read_a12a_root().unwrap(), authority_before);
        }
        assert_eq!(
            runtime
                .read_admission_record(
                    &authority,
                    &first.record.orchestration_session_id,
                    &first.record.retained_participant_id,
                )
                .unwrap(),
            Some(first.record)
        );
    }

    #[test]
    fn malformed_committed_admission_identity_fails_closed_without_repair() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "identity-corruption", "prompt", 2);
        let slot = runtime.reserve_admission_slot(&authority, &plan).unwrap();
        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let mut registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
            &fs::read(&registry_path).unwrap(),
            "decode fixture registry",
        )
        .unwrap();
        let records = registry
            .records_by_session
            .get_mut(&slot.record.orchestration_session_id)
            .unwrap();
        let mut malformed = records
            .remove(&slot.record.retained_participant_id)
            .unwrap();
        malformed.retained_participant_id = "malformed-participant".into();
        records.insert("malformed-participant".into(), malformed);
        registry
            .issuer_request_index
            .get_mut(&slot.record.issuer_request_id)
            .unwrap()
            .retained_participant_id = "malformed-participant".into();
        let malformed_bytes = encode_canonical(&registry, "encode fixture registry").unwrap();
        fs::write(&registry_path, &malformed_bytes).unwrap();

        assert!(runtime
            .read_admission_record(
                &authority,
                &slot.record.orchestration_session_id,
                "malformed-participant",
            )
            .is_err());
        assert_eq!(fs::read(registry_path).unwrap(), malformed_bytes);
    }

    #[test]
    fn admission_initialization_rejects_inexact_committed_registry_indices_and_records() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "registry-corruption", "prompt", 2);
        let slot = runtime.reserve_admission_slot(&authority, &plan).unwrap();
        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let original: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
            &fs::read(&registry_path).unwrap(),
            "decode fixture registry",
        )
        .unwrap();

        let mut variants = Vec::new();
        let mut changed = original.clone();
        changed
            .issuer_request_index
            .get_mut(&slot.record.issuer_request_id)
            .unwrap()
            .retained_participant_id = "rwp_ffffffffffffffffffffffffffffffff".into();
        variants.push(changed);
        let mut changed = original.clone();
        changed
            .next_slot_sequence_by_session
            .insert(slot.record.orchestration_session_id.clone(), 1);
        variants.push(changed);
        let mut changed = original.clone();
        changed
            .records_by_session
            .get_mut(&slot.record.orchestration_session_id)
            .unwrap()
            .get_mut(&slot.record.retained_participant_id)
            .unwrap()
            .authority_store_id = "as_00000000000000000000000000000000".into();
        variants.push(changed);
        let mut changed = original;
        changed
            .records_by_session
            .get_mut(&slot.record.orchestration_session_id)
            .unwrap()
            .get_mut(&slot.record.retained_participant_id)
            .unwrap()
            .canonical_spawn_fingerprint
            .key_id = "adk_ffffffffffffffffffffffffffffffff".into();
        variants.push(changed);

        for changed in variants {
            let changed_bytes = encode_canonical(&changed, "encode fixture registry").unwrap();
            fs::write(&registry_path, &changed_bytes).unwrap();
            assert!(runtime.initialize_admission_registry(&authority).is_err());
            assert_eq!(fs::read(&registry_path).unwrap(), changed_bytes);
        }
    }

    #[test]
    fn admission_fingerprint_binds_stable_participant_and_bootstrap_run() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "identity-binding", "private prompt", 2);
        let slot = runtime.reserve_admission_slot(&authority, &plan).unwrap();
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (_, key_path) = initialized_admission_paths(&parent, &identity);
        let envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(key_path).unwrap(), "decode fixture key").unwrap();
        let changed_participant = canonical_spawn_fingerprint(
            &identity.key_id,
            &envelope.secret_key,
            &plan,
            "rwp_00000000000000000000000000000000",
            &slot.record.bootstrap_run_id,
        )
        .unwrap();
        let changed_run = canonical_spawn_fingerprint(
            &identity.key_id,
            &envelope.secret_key,
            &plan,
            &slot.record.retained_participant_id,
            "rwr_00000000000000000000000000000000",
        )
        .unwrap();

        assert_ne!(changed_participant, slot.record.canonical_spawn_fingerprint);
        assert_ne!(changed_run, slot.record.canonical_spawn_fingerprint);
        assert_ne!(changed_participant, changed_run);
    }

    #[test]
    fn admission_registry_contains_neither_prompt_nor_key_material() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let prompt = "private prompt marker B3.2a";
        let plan = admission_plan(&authority, "secret-exclusion", prompt, 2);
        runtime.reserve_admission_slot(&authority, &plan).unwrap();
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (admission_root, key_path) = initialized_admission_paths(&parent, &identity);
        let envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(key_path).unwrap(), "decode fixture key").unwrap();
        let registry_bytes = fs::read(admission_root.join("registry-v1.json")).unwrap();
        let registry_text = std::str::from_utf8(&registry_bytes).unwrap();
        let canonical_secret = serde_json::to_string(&envelope.secret_key).unwrap();

        assert!(!registry_text.contains(prompt));
        assert!(!registry_text.contains(&canonical_secret));
    }

    #[test]
    fn crash_after_slot_reserved_retries_to_the_committed_identity() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "slot-crash", "prompt", 2);
        let failure = runtime
            .reserve_admission_slot_at(
                &authority,
                &plan,
                timestamp("2026-07-15T13:00:00.000000000Z"),
                [11_u8; 16],
                [12_u8; 16],
                [13_u8; 16],
                Some(AdmissionReservationCrashPointV1::AfterSlotReserved),
            )
            .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "injected crash after admission slot reservation"
        );

        let joined = runtime.reserve_admission_slot(&authority, &plan).unwrap();
        assert!(joined.joined);
        assert_eq!(
            joined.record.retained_participant_id,
            format!("rwp_{}", lower_hex(&[11_u8; 16]))
        );
        assert_eq!(
            joined.record.bootstrap_run_id,
            format!("rwr_{}", lower_hex(&[12_u8; 16]))
        );
    }

    #[test]
    fn concurrent_process_cap_reservation_commits_exactly_one_live_slot() {
        let (parent, authority, _) = started_authority();
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::admission_reservation_subprocess_worker";
        let spawn = |issuer: &str, prompt: &str| {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(
                    ADMISSION_RESERVATION_SUBPROCESS_HOME,
                    parent.path().join("home"),
                )
                .env(ADMISSION_RESERVATION_SUBPROCESS_ISSUER, issuer)
                .env(ADMISSION_RESERVATION_SUBPROCESS_PROMPT, prompt)
                .env(ADMISSION_RESERVATION_SUBPROCESS_CAP, "1")
                .spawn()
                .unwrap()
        };
        let mut first = spawn("cap-process-a", "prompt a");
        let mut second = spawn("cap-process-b", "prompt b");
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());

        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let registry: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&fs::read(registry_path).unwrap(), "decode fixture registry").unwrap();
        assert_eq!(
            registry
                .records_by_session
                .values()
                .flat_map(BTreeMap::values)
                .count(),
            1
        );
        assert_eq!(registry.issuer_request_index.len(), 1);
        assert_eq!(
            authority.read_a12a_root().unwrap().authority_store_id,
            registry.authority_store_id
        );
    }

    #[test]
    fn concurrent_process_exact_retry_joins_one_admission_identity() {
        let (parent, _authority, _) = started_authority();
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::admission_reservation_subprocess_worker";
        let spawn = || {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(
                    ADMISSION_RESERVATION_SUBPROCESS_HOME,
                    parent.path().join("home"),
                )
                .env(ADMISSION_RESERVATION_SUBPROCESS_ISSUER, "same-issuer")
                .env(ADMISSION_RESERVATION_SUBPROCESS_PROMPT, "same prompt")
                .env(ADMISSION_RESERVATION_SUBPROCESS_CAP, "2")
                .spawn()
                .unwrap()
        };
        let mut first = spawn();
        let mut second = spawn();
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());

        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let registry: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&fs::read(registry_path).unwrap(), "decode fixture registry").unwrap();
        assert_eq!(registry.issuer_request_index.len(), 1);
        assert_eq!(
            registry
                .records_by_session
                .values()
                .flat_map(BTreeMap::values)
                .count(),
            1
        );
    }

    #[test]
    fn admission_registration_head_crash_retries_exactly_and_serializes_queued_slots() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "head-first", "first prompt", 4);
        let second_plan = admission_plan(&authority, "head-second", "second prompt", 4);
        let first_slot = runtime
            .reserve_admission_slot(&authority, &first_plan)
            .unwrap();
        let second_slot = runtime
            .reserve_admission_slot(&authority, &second_plan)
            .unwrap();

        let queued = runtime
            .register_admitted_worker(&authority, &second_plan)
            .unwrap_err();
        assert_eq!(
            queued.to_string(),
            "a lower-sequence admission slot owns the registration head"
        );
        assert_eq!(
            authority
                .resolve_current_exact("r0-session", None)
                .unwrap()
                .authority
                .authority_revision,
            1
        );

        let crashed = runtime
            .register_admitted_worker_at(
                &authority,
                &first_plan,
                Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
            )
            .unwrap_err();
        assert_eq!(
            crashed.to_string(),
            "injected crash while admission registration head"
        );
        let durable_head = runtime
            .read_admission_record(
                &authority,
                "r0-session",
                &first_slot.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            durable_head.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
                authority_revision_expected: 1,
                ..
            }
        ));

        let first = runtime
            .register_admitted_worker(&authority, &first_plan)
            .unwrap();
        assert!(matches!(
            first.record.state,
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        ));
        let second_head = runtime
            .prepare_admission_registration_head(
                &authority,
                &second_plan,
                &second_slot.record.retained_participant_id,
                timestamp("2026-07-15T14:00:00.000000000Z"),
                [41_u8; 16],
            )
            .unwrap();
        let AdmissionHeadPreparationV1::Ready(second_head) = second_head else {
            panic!("second slot must become the serialized head")
        };
        assert!(matches!(
            second_head.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
                authority_revision_expected: 2,
                ..
            }
        ));
        assert_eq!(second_head.admission_authority_revision, 1);

        let second = runtime
            .register_admitted_worker(&authority, &second_plan)
            .unwrap();
        assert!(matches!(
            second.record.state,
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        ));
        assert_eq!(
            authority
                .resolve_current_exact("r0-session", None)
                .unwrap()
                .authority
                .authority_revision,
            3
        );
        let exact_retry = runtime
            .register_admitted_worker(&authority, &first_plan)
            .unwrap();
        assert!(exact_retry.joined);
        assert_eq!(exact_retry.record, first.record);
    }

    #[test]
    fn applied_r0_crash_reconciles_admission_without_duplicate_authority_mutation() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "r0-crash", "prompt", 3);
        let failure = runtime
            .register_admitted_worker_at(
                &authority,
                &plan,
                Some(AdmissionRegistrationCrashPointV1::AfterR0BeforeAdmissionAdvance),
            )
            .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "injected crash after R0 before admission advancement"
        );
        let root_after_r0 = authority.read_a12a_root().unwrap();
        assert_eq!(root_after_r0.retained_worker_registration_journal.len(), 1);
        let registration_id = root_after_r0
            .retained_worker_registration_journal
            .keys()
            .next()
            .unwrap()
            .clone();

        let reconciled = runtime.register_admitted_worker(&authority, &plan).unwrap();
        assert!(reconciled.joined);
        let RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration } =
            &reconciled.record.state
        else {
            panic!("applied R0 must reconcile to pre-transport truth")
        };
        assert_eq!(registration.registration_id, registration_id);
        let root_after_retry = authority.read_a12a_root().unwrap();
        assert_eq!(root_after_retry, root_after_r0);
        assert_eq!(
            root_after_retry.retained_worker_registration_journal.len(),
            1
        );
    }

    #[test]
    fn concurrent_transport_claim_has_one_durable_winner_and_never_steals() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "claim-race", "prompt", 3);
        let admitted = runtime.register_admitted_worker(&authority, &plan).unwrap();
        let participant_id = admitted.record.retained_participant_id.clone();
        let home = parent.path().join("home");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let mut workers = Vec::new();
        for entropy in [51_u8, 52_u8] {
            let home = home.clone();
            let plan = plan.clone();
            let participant_id = participant_id.clone();
            let barrier = barrier.clone();
            workers.push(std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(&home).unwrap();
                barrier.wait();
                RetainedWorkerRuntime
                    .claim_admission_transport_with(
                        &authority,
                        &plan,
                        &participant_id,
                        timestamp("2026-07-15T14:01:00.000000000Z"),
                        [entropy; 16],
                        [entropy.wrapping_add(10); 16],
                    )
                    .unwrap()
            }));
        }
        let claims = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(claims.iter().filter(|claim| claim.newly_claimed).count(), 1);
        assert_eq!(claims[0].record, claims[1].record);
        let RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
            transport_claim_id, ..
        } = &claims[0].record.state
        else {
            panic!("claim race must persist one transport claim")
        };
        let later = runtime
            .claim_admission_transport(&authority, &plan, &participant_id)
            .unwrap();
        assert!(!later.newly_claimed);
        assert_eq!(later.record, claims[0].record);
        assert!(transport_claim_id.starts_with("rtc_"));
    }

    #[test]
    fn admission_hmac_matches_independent_sha256_vector() {
        let digest = hmac_sha256(&[0x0b_u8; 32], b"Hi There");
        assert_eq!(
            lower_hex(&digest),
            "198a607eb44bfbc69903a0f1cf2bbdc5ba0aa3f3d9ae3c1c7a3b1696a0b68cf7"
        );
    }

    #[test]
    fn canonical_spawn_fingerprint_len64_framing_is_big_endian_and_unambiguous() {
        let mut framed = Vec::new();
        append_len64(&mut framed, b"a").unwrap();
        append_len64(&mut framed, b"bc").unwrap();
        assert_eq!(
            framed,
            vec![0, 0, 0, 0, 0, 0, 0, 1, b'a', 0, 0, 0, 0, 0, 0, 0, 2, b'b', b'c']
        );
    }
}
