//! Component-only retained-worker authority registration.
//!
//! This module deliberately has no production ingress caller. It owns immutable
//! retained-object construction while `HostSessionAuthority` owns durable
//! reservation and authority mutation.

#![allow(
    dead_code,
    reason = "R0 is a component proof and deliberately has no production ingress caller"
)]

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rand::RngCore;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use substrate_common::agent_events::{
    AgentEvent, AgentEventKind, RuntimeEventIdentityV1, RuntimeFrameIdentityV1,
    RuntimeTerminalIdentityV1,
};
use transport_api_types::{
    RetainedWorkerAdmissionCommitmentCarrierV1, RetainedWorkerAuthorityObjectCommitmentV1,
    RetainedWorkerLaunchAuthorityProofV1, RetainedWorkerLaunchWorldBindingV1,
};

use super::dispatch_policy_commitment::AuthenticatedFreshSpawnReservationProofV1;
use super::host_session_authority::canonical_json;
#[cfg(test)]
use super::host_session_authority::facade::RetainedReservationCrashPointV1;
use super::host_session_authority::facade::{
    ReservedRetainedWorkerRegistrationV1, ResolvedCurrentAuthorityV1, ResolvedSessionAuthorityV1,
    RetainedWorkerAuthorityPreconditionV1,
};
use super::host_session_authority::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AgentExecutionScopeV1,
    AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, AuthorityObjectRefV1,
    DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1, HostAttachContractV1,
    HostSessionPostureV1, PolicyObjectHashInputV1, ResumeHandleHashInputV1,
    RetainedWorkerObjectHashInputV1, TimestampV1, WorldBindingV1,
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
    pub(crate) registration_commitment: AuthorityObjectCommitmentV1,
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
#[must_use]
pub(crate) enum RetainedWorkerAdmissionCancelDeliveryStateV1 {
    Available,
    Claimed {
        delivery_claim_id: String,
        claimed_at: TimestampV1,
        claim_expires_at: TimestampV1,
    },
    Confirmed {
        delivery_claim_id: String,
        confirmed_at: TimestampV1,
    },
}

impl Default for RetainedWorkerAdmissionCancelDeliveryStateV1 {
    fn default() -> Self {
        Self::Available
    }
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_span_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stream_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_frame_sequence: Option<u64>,
        claimed_at: TimestampV1,
    },
    Routable {
        registration: RetainedWorkerAdmissionRegistrationV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_claim_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_span_id: Option<String>,
        stream_id: String,
        registered_frame_sequence: u64,
        registered_event_id: String,
        registered_event_sequence: u64,
        registered_at: TimestampV1,
    },
    InterruptedNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_claim_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_span_id: Option<String>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cancel_request_id: Option<String>,
    },
    RejectedBeforeRegistration {
        reason: String,
        rejected_at: TimestampV1,
    },
    CancelledBeforeRegistration {
        cancel_request_id: String,
        cancelled_at: TimestampV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        registration_head: Option<RetainedWorkerAdmissionRegistrationHeadV1>,
    },
    CancelledBeforeTransport {
        registration: RetainedWorkerAdmissionRegistrationV1,
        cancel_request_id: String,
        cancelled_at: TimestampV1,
    },
    CancellationAcceptedTransportCloseoutPending {
        registration: RetainedWorkerAdmissionRegistrationV1,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_claim_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_span_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stream_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_frame_sequence: Option<u64>,
        cancel_request_id: String,
        accepted_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAdmissionRegistrationHeadV1 {
    pub(crate) authority_revision_expected: u64,
    pub(crate) authority_record_commitment_expected: AuthorityObjectCommitmentV1,
    pub(crate) head_acquired_at: TimestampV1,
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
struct RetainedWorkerAdmissionCancelDeliveryRecordV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    retained_participant_id: String,
    cancel_request_id: String,
    transport_span_id: String,
    accepted_at: TimestampV1,
    delivery_state: RetainedWorkerAdmissionCancelDeliveryStateV1,
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    cancel_deliveries_by_request: BTreeMap<String, RetainedWorkerAdmissionCancelDeliveryRecordV1>,
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
pub(crate) struct RetainedWorkerAdmissionCancelRequestV1 {
    pub(crate) schema_version: u32,
    pub(crate) cancel_request_id: String,
    pub(crate) authority_store_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) caller_participant_id: String,
    pub(crate) retained_participant_id: String,
    pub(crate) bootstrap_run_id: String,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) world_binding: WorldBindingV1,
    pub(crate) current_policy_ref: AuthorityObjectRefV1,
    pub(crate) current_policy_revision: String,
    pub(crate) expected_record_revision: u64,
    pub(crate) expected_state: RetainedWorkerAdmissionStateV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RetainedWorkerAdmissionCancelOutcomeV1 {
    CancelledBeforeTransport {
        record: RetainedWorkerAdmissionRecordV1,
        cancel_request_id: String,
    },
    CancelAcceptedPendingCloseout {
        record: RetainedWorkerAdmissionRecordV1,
        cancel_request_id: String,
        transport_span_id: Option<String>,
        delivery_disposition: RetainedWorkerAdmissionCancelDeliveryDispositionV1,
    },
    AlreadyRoutable {
        record: RetainedWorkerAdmissionRecordV1,
    },
    AlreadyTerminal {
        record: RetainedWorkerAdmissionRecordV1,
        cancel_request_id: Option<String>,
    },
    RejectedBeforeRegistration {
        record: RetainedWorkerAdmissionRecordV1,
    },
}
impl RetainedWorkerAdmissionCancelOutcomeV1 {
    pub(crate) fn record(&self) -> &RetainedWorkerAdmissionRecordV1 {
        match self {
            Self::CancelledBeforeTransport { record, .. }
            | Self::CancelAcceptedPendingCloseout { record, .. }
            | Self::AlreadyRoutable { record }
            | Self::AlreadyTerminal { record, .. }
            | Self::RejectedBeforeRegistration { record } => record,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RetainedWorkerAdmissionRoutabilityDispositionV1 {
    Routable,
    CancellationWon { cancel_request_id: String },
    AlreadyTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAdmissionRoutabilityOutcomeV1 {
    pub(crate) record: RetainedWorkerAdmissionRecordV1,
    pub(crate) disposition: RetainedWorkerAdmissionRoutabilityDispositionV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAdmissionTransportCancellationV1 {
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) retained_participant_id: String,
    pub(crate) cancel_request_id: String,
    pub(crate) transport_span_id: String,
    pub(crate) delivery_claim_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmissionRejectedBeforeRegistrationInputV1 {
    pub(crate) reason: String,
    pub(crate) rejected_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AdmissionHeadPreparationV1 {
    Ready(RetainedWorkerAdmissionRecordV1),
    Complete(RetainedWorkerAdmissionRecordV1),
    Queued,
}

struct ResolvedPostR0RegisteredGraphV1 {
    result: RetainedWorkerRegistrationResultV1,
    resolved_target: ResolvedRetainedTargetV1,
}

struct ResolvedPostR0RegistryGraphV1 {
    current_exact_authority: CanonicalExactCurrentAuthorityV1,
    typed_authority_history: Option<BTreeMap<u64, ResolvedSessionAuthorityV1>>,
    registered_graph: ResolvedPostR0RegisteredGraphV1,
}

struct AdmissionTransportClaimInputV1<'a> {
    claimed_at: TimestampV1,
    claim_entropy: [u8; 16],
    post_r0_graph: Option<&'a ResolvedPostR0RegisteredGraphV1>,
}

struct AdmissionTransportPublicationInputV1 {
    claimed_at: TimestampV1,
    claim_entropy: [u8; 16],
    publication_nonce: [u8; 16],
    #[cfg(test)]
    crash_point: Option<AdmissionTransportClaimCrashPointV1>,
}

enum AdmissionRuntimeTruthInputV1<'a> {
    TransportStarted {
        frame_identity: &'a RuntimeFrameIdentityV1,
        transport_span_id: &'a str,
    },
    Registered {
        frame_identity: &'a RuntimeFrameIdentityV1,
        event: &'a AgentEvent,
        registered_at: TimestampV1,
    },
    Terminal {
        frame_identity: &'a RuntimeFrameIdentityV1,
        event_identity: &'a RuntimeEventIdentityV1,
        terminal_identity: &'a RuntimeTerminalIdentityV1,
        transport_span_id: Option<&'a str>,
        exit_code: i32,
        terminal_at: TimestampV1,
    },
    Interrupted {
        last_frame_identity: Option<&'a RuntimeFrameIdentityV1>,
        interrupted_at: TimestampV1,
    },
}

struct AdmissionRegistrationAdvanceInputV1<'a> {
    retained_participant_id: &'a str,
    secret_key: &'a [u8; 32],
    post_r0_graph: &'a ResolvedPostR0RegisteredGraphV1,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionInitializationCrashPointV1 {
    BeforeKeyTempPersistence,
    AfterKeyTempFsync,
    BeforeKeyPublication,
    AfterKeyPublication,
    BeforeRegistryTempPersistence,
    AfterRegistryTempFsync,
    BeforeRegistryNoReplacePublication,
    AfterRegistryPublication,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionReservationCrashPointV1 {
    BeforeTempPersistence,
    AfterTempFsync,
    BeforeRegistryReplacementPublication,
    AfterSlotReserved,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionRegistrationCrashPointV1 {
    BeforeRegistrationHeadPublication,
    DuringRegistrationHeadPublication,
    AfterRegistrationHeadPublicationBeforeResponse,
    WhileRegistrationHead,
    AfterR0BeforeAdmissionAdvance,
    BeforeAdmissionAdvancePublication,
    DuringAdmissionAdvancePublication,
    AfterAdmissionAdvancePublicationBeforeResponse,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmissionTransportClaimCrashPointV1 {
    BeforeTempPersistence,
    AfterTempFsync,
    BeforeRegistryReplacementPublication,
    AfterPublicationBeforeResponse,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AdmissionCancellationCrashPointV1 {
    BeforeTempPersistence,
    AfterTempFsync,
    BeforeRegistryReplacementPublication,
    AfterPublicationBeforeResponse,
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
            #[cfg(test)]
            if crash_point == Some(AdmissionInitializationCrashPointV1::BeforeKeyTempPersistence) {
                return Err(
                    super::host_session_authority::store::BootstrapError::retained_admission_crash(
                    ),
                );
            }
            transaction.stage_key_temp(&key_temp_name, &envelope_bytes)?;
            #[cfg(test)]
            if matches!(
                crash_point,
                Some(AdmissionInitializationCrashPointV1::AfterKeyTempFsync)
                    | Some(AdmissionInitializationCrashPointV1::BeforeKeyPublication)
            ) {
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
                cancel_deliveries_by_request: BTreeMap::new(),
            };
            let registry_bytes = retain_semantic_error(
                encode_canonical(&registry, "encode admission registry"),
                &mut semantic_failure,
            )?;
            #[cfg(test)]
            if crash_point
                == Some(AdmissionInitializationCrashPointV1::BeforeRegistryTempPersistence)
            {
                return Err(
                    super::host_session_authority::store::BootstrapError::retained_admission_crash(
                    ),
                );
            }
            #[cfg(test)]
            if matches!(
                crash_point,
                Some(AdmissionInitializationCrashPointV1::AfterRegistryTempFsync)
                    | Some(AdmissionInitializationCrashPointV1::BeforeRegistryNoReplacePublication)
            ) {
                transaction
                    .stage_registry_replacement_for_test(&registry_temp_name, &registry_bytes)?;
                return Err(
                    super::host_session_authority::store::BootstrapError::retained_admission_crash(
                    ),
                );
            }
            transaction.publish_registry_no_replace(&registry_temp_name, &registry_bytes)?;
            #[cfg(test)]
            if crash_point == Some(AdmissionInitializationCrashPointV1::AfterRegistryPublication) {
                return Err(
                    super::host_session_authority::store::BootstrapError::retained_admission_crash(
                    ),
                );
            }
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
                if crash_point
                    == Some(AdmissionInitializationCrashPointV1::BeforeKeyTempPersistence)
                {
                    return RetainedWorkerRuntimeError(
                        "injected crash before admission key temp persistence".into(),
                    );
                }
                if crash_point == Some(AdmissionInitializationCrashPointV1::AfterKeyTempFsync) {
                    return RetainedWorkerRuntimeError(
                        "injected crash after admission key temp fsync".into(),
                    );
                }
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
                if crash_point
                    == Some(AdmissionInitializationCrashPointV1::BeforeRegistryTempPersistence)
                {
                    return RetainedWorkerRuntimeError(
                        "injected crash before admission registry temp persistence".into(),
                    );
                }
                if crash_point == Some(AdmissionInitializationCrashPointV1::AfterRegistryTempFsync)
                {
                    return RetainedWorkerRuntimeError(
                        "injected crash after admission registry temp fsync".into(),
                    );
                }
                if crash_point
                    == Some(AdmissionInitializationCrashPointV1::BeforeRegistryNoReplacePublication)
                {
                    return RetainedWorkerRuntimeError(
                        "injected crash before admission registry no-replace publication".into(),
                    );
                }
                if crash_point
                    == Some(AdmissionInitializationCrashPointV1::AfterRegistryPublication)
                {
                    return RetainedWorkerRuntimeError(
                        "injected crash after admission registry publication before response"
                            .into(),
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
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
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
            reservation_proof,
            reserved_at,
            participant_entropy,
            bootstrap_run_entropy,
            publication_nonce,
            None,
        )
    }

    pub(crate) fn canonical_plan_for_existing_admission(
        &self,
        authority: &HostSessionAuthority,
        presented_plan: &RetainedWorkerAdmissionPlanV1,
        expected_record: &RetainedWorkerAdmissionRecordV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    ) -> Result<RetainedWorkerAdmissionPlanV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&presented_plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = presented_plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let typed_history = authority
            .resolve_exact_typed_history(&presented_plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let post_r0_graphs = self.resolve_all_post_r0_registry_graphs(authority)?;
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
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
                "resolve exact accepted-home existing-admission policy".into(),
            )
        })?;
        validate_admission_policy_against_resolved(
            &current_plan.policy_and_admission_cap,
            &policy,
        )?;
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
            let registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
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
            retain_semantic_error(
                validate_complete_post_r0_registry_graphs(
                    &registry,
                    transaction.authority_root(),
                    &post_r0_graphs,
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_plan(
                    transaction.authority_root(),
                    &current_plan,
                    reservation_proof,
                ),
                &mut semantic_failure,
            )?;
            let locator = retain_semantic_error(
                registry
                    .issuer_request_index
                    .get(&presented_plan.issuer_request_id)
                    .ok_or_else(|| {
                        RetainedWorkerRuntimeError(
                            "exact existing admission issuer is absent".into(),
                        )
                    }),
                &mut semantic_failure,
            )?;
            if locator.orchestration_session_id != expected_record.orchestration_session_id
                || locator.retained_participant_id != expected_record.retained_participant_id
            {
                retain_semantic_error(
                    Err(RetainedWorkerRuntimeError(
                        "exact existing admission locator conflicts with the joined record".into(),
                    )),
                    &mut semantic_failure,
                )?;
            }
            let record = retain_semantic_error(
                registry
                    .records_by_session
                    .get(&locator.orchestration_session_id)
                    .and_then(|records| records.get(&locator.retained_participant_id))
                    .ok_or_else(|| {
                        RetainedWorkerRuntimeError(
                            "exact existing admission record is absent".into(),
                        )
                    }),
                &mut semantic_failure,
            )?;
            if record != expected_record {
                retain_semantic_error(
                    Err(RetainedWorkerRuntimeError(
                        "exact existing admission record changed after its locked join".into(),
                    )),
                    &mut semantic_failure,
                )?;
            }
            retain_semantic_error(
                validate_reservation_proof_identities(reservation_proof, record),
                &mut semantic_failure,
            )?;
            let root = retain_semantic_error(
                preserved_start_root_view(transaction.authority_root()),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_admission_to_current_exact_authority_ancestry(
                    root.as_ref(),
                    &current_plan.exact_authority,
                    record,
                    typed_history.as_ref(),
                ),
                &mut semantic_failure,
            )?;
            let mut canonical_plan = presented_plan.clone();
            canonical_plan.exact_authority = retain_semantic_error(
                reconstruct_exact_authority_at_revision(
                    root.as_ref(),
                    &current_plan.exact_authority,
                    record.admission_authority_revision,
                    typed_history.as_ref(),
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                validate_supplied_admission_authority(
                    root.as_ref(),
                    &current_plan.exact_authority,
                    &canonical_plan.exact_authority,
                    record,
                    typed_history.as_ref(),
                ),
                &mut semantic_failure,
            )?;
            retain_semantic_error(
                verify_admission_record_fingerprint(&canonical_plan, record, &envelope.secret_key),
                &mut semantic_failure,
            )?;
            if record.authority_store_id != canonical_plan.exact_authority.authority_store_id
                || record.issuer_request_id != canonical_plan.issuer_request_id
                || record.orchestration_session_id
                    != canonical_plan.spawn_request.orchestration_session_id
                || record.admission_authority_revision
                    != canonical_plan.exact_authority.authority_revision
                || record.admission_authority_record_commitment
                    != canonical_plan.exact_authority.authority_record_commitment
                || record.backend_id
                    != canonical_plan
                        .descriptor_and_runtime_plan
                        .descriptor
                        .backend_id
                || record.protocol
                    != canonical_plan
                        .descriptor_and_runtime_plan
                        .descriptor
                        .protocol
                || record.world_binding.world_id != canonical_plan.spawn_request.world_id
                || record.world_binding.world_generation
                    != canonical_plan.spawn_request.world_generation
                || record.current_policy_ref
                    != canonical_plan.policy_and_admission_cap.current_policy_ref
                || record.current_policy_revision
                    != canonical_plan
                        .policy_and_admission_cap
                        .current_policy
                        .policy_revision
                || record.max_live_retained_workers
                    != canonical_plan
                        .policy_and_admission_cap
                        .max_live_retained_workers
            {
                retain_semantic_error(
                    Err(RetainedWorkerRuntimeError(
                        "exact existing admission conflicts with canonical re-presentation".into(),
                    )),
                    &mut semantic_failure,
                )?;
            }
            Ok(canonical_plan)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
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
            None,
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
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
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
        let typed_history = authority
            .resolve_exact_typed_history(&plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut fingerprint_plan = plan.clone();
        self.initialize_admission_registry(authority)?;
        let post_r0_graphs = self.resolve_all_post_r0_registry_graphs(authority)?;
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
            retain_semantic_error(
                validate_complete_post_r0_registry_graphs(
                    &registry,
                    transaction.authority_root(),
                    &post_r0_graphs,
                ),
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
                let root = retain_semantic_error(
                    preserved_start_root_view(transaction.authority_root()),
                    &mut semantic_failure,
                )?;
                retain_semantic_error(
                    validate_admission_to_current_exact_authority_ancestry(
                        root.as_ref(),
                        &current_plan.exact_authority,
                        record,
                        typed_history.as_ref(),
                    ),
                    &mut semantic_failure,
                )?;
                fingerprint_plan.exact_authority = retain_semantic_error(
                    reconstruct_exact_authority_at_revision(
                        root.as_ref(),
                        &current_plan.exact_authority,
                        record.admission_authority_revision,
                        typed_history.as_ref(),
                    ),
                    &mut semantic_failure,
                )?;
                retain_semantic_error(
                    validate_supplied_admission_authority(
                        root.as_ref(),
                        &current_plan.exact_authority,
                        &fingerprint_plan.exact_authority,
                        record,
                        typed_history.as_ref(),
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
                    &fingerprint_plan,
                    reservation_proof,
                    &envelope.secret_key,
                    reserved_at,
                    participant_entropy,
                    bootstrap_run_entropy,
                    typed_history.as_ref(),
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
            #[cfg(test)]
            if crash_point == Some(AdmissionReservationCrashPointV1::BeforeTempPersistence) {
                retain_semantic_error(
                    Err(RetainedWorkerRuntimeError(
                        "injected crash before admission slot temp persistence".into(),
                    )),
                    &mut semantic_failure,
                )?;
            }
            #[cfg(test)]
            if matches!(
                crash_point,
                Some(
                    AdmissionReservationCrashPointV1::AfterTempFsync
                        | AdmissionReservationCrashPointV1::BeforeRegistryReplacementPublication
                )
            ) {
                transaction.stage_registry_replacement_for_test(&temp_name, &published)?;
                let message = if crash_point
                    == Some(AdmissionReservationCrashPointV1::AfterTempFsync)
                {
                    "injected crash after admission slot temp fsync"
                } else {
                    "injected crash before admission slot registry publication"
                };
                retain_semantic_error(
                    Err(RetainedWorkerRuntimeError(message.into())),
                    &mut semantic_failure,
                )?;
            }
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
        let slot = result.map_err(|error| {
            #[cfg(test)]
            if error.to_string() == "injected retained admission initialization crash"
                && crash_point == Some(AdmissionReservationCrashPointV1::AfterSlotReserved)
            {
                return RetainedWorkerRuntimeError(
                    "injected crash after admission slot reservation".into(),
                );
            }
            RetainedWorkerRuntimeError(error.to_string())
        })?;
        if slot.joined && admission_registration(&slot.record.state).is_some() {
            self.validate_admitted_record_graph(authority, plan, &slot.record, reservation_proof)?;
        }
        Ok(slot)
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

    pub(crate) fn cancel_pending_admission(
        &self,
        authority: &HostSessionAuthority,
        request: &RetainedWorkerAdmissionCancelRequestV1,
    ) -> Result<RetainedWorkerAdmissionCancelOutcomeV1, RetainedWorkerRuntimeError> {
        let accepted_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| {
            RetainedWorkerRuntimeError("create admission cancellation timestamp".into())
        })?;
        let mut publication_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut publication_nonce);
        let mut delivery_claim_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut delivery_claim_nonce);
        self.cancel_pending_admission_with(
            authority,
            request,
            accepted_at,
            delivery_claim_nonce,
            publication_nonce,
            #[cfg(test)]
            None,
            #[cfg(not(test))]
            None,
        )
    }

    #[cfg(test)]
    pub(crate) fn cancel_pending_admission_at(
        &self,
        authority: &HostSessionAuthority,
        request: &RetainedWorkerAdmissionCancelRequestV1,
        accepted_at: TimestampV1,
        delivery_claim_nonce: [u8; 16],
        publication_nonce: [u8; 16],
        crash_point: Option<AdmissionCancellationCrashPointV1>,
    ) -> Result<RetainedWorkerAdmissionCancelOutcomeV1, RetainedWorkerRuntimeError> {
        self.cancel_pending_admission_with(
            authority,
            request,
            accepted_at,
            delivery_claim_nonce,
            publication_nonce,
            crash_point,
        )
    }

    fn cancel_pending_admission_with(
        &self,
        authority: &HostSessionAuthority,
        request: &RetainedWorkerAdmissionCancelRequestV1,
        accepted_at: TimestampV1,
        delivery_claim_nonce: [u8; 16],
        publication_nonce: [u8; 16],
        #[cfg(test)] crash_point: Option<AdmissionCancellationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<RetainedWorkerAdmissionCancelOutcomeV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let current_exact = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let typed_history = authority
            .resolve_exact_typed_history(&request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let post_r0_graphs = self.resolve_all_post_r0_registry_graphs(authority)?;
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
            retain_semantic_error(
                validate_complete_post_r0_registry_graphs(
                    &registry,
                    transaction.authority_root(),
                    &post_r0_graphs,
                ),
                &mut semantic_failure,
            )?;
            let (outcome, changed) = retain_semantic_error(
                cancel_pending_admission_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    &current_exact,
                    request,
                    &accepted_at,
                    &delivery_claim_nonce,
                    typed_history.as_ref(),
                    &post_r0_graphs,
                ),
                &mut semantic_failure,
            )?;
            if changed {
                #[cfg(test)]
                if crash_point == Some(AdmissionCancellationCrashPointV1::BeforeTempPersistence) {
                    retain_semantic_error(
                        Err(RetainedWorkerRuntimeError(
                            "injected crash before admission cancellation temp persistence".into(),
                        )),
                        &mut semantic_failure,
                    )?;
                }
                let bytes = retain_semantic_error(
                    encode_canonical(&registry, "encode admission registry"),
                    &mut semantic_failure,
                )?;
                let temp_name = format!(
                    "admission-registry--{}.tmp",
                    lower_hex(&publication_nonce)
                );
                #[cfg(test)]
                if matches!(
                    crash_point,
                    Some(
                        AdmissionCancellationCrashPointV1::AfterTempFsync
                            | AdmissionCancellationCrashPointV1::BeforeRegistryReplacementPublication
                    )
                ) {
                    transaction.stage_registry_replacement_for_test(&temp_name, &bytes)?;
                    let message = if crash_point
                        == Some(AdmissionCancellationCrashPointV1::AfterTempFsync)
                    {
                        "injected crash after admission cancellation temp fsync"
                    } else {
                        "injected crash before admission cancellation registry publication"
                    };
                    retain_semantic_error(
                        Err(RetainedWorkerRuntimeError(message.into())),
                        &mut semantic_failure,
                    )?;
                }
                transaction.replace_registry(&temp_name, &bytes)?;
                #[cfg(test)]
                if crash_point
                    == Some(AdmissionCancellationCrashPointV1::AfterPublicationBeforeResponse)
                {
                    retain_semantic_error(
                        Err(RetainedWorkerRuntimeError(
                            "injected crash after admission cancellation publication before response"
                                .into(),
                        )),
                        &mut semantic_failure,
                    )?;
                }
            }
            Ok(outcome)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    pub(crate) fn record_pending_admission_cancel_delivery(
        &self,
        authority: &HostSessionAuthority,
        completion: &RetainedWorkerAdmissionCancelDeliveryCompletionV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        let observed_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| {
            RetainedWorkerRuntimeError(
                "create admission cancellation delivery result timestamp".into(),
            )
        })?;
        self.record_pending_admission_cancel_delivery_at(authority, completion, observed_at)
    }

    fn record_pending_admission_cancel_delivery_at(
        &self,
        authority: &HostSessionAuthority,
        completion: &RetainedWorkerAdmissionCancelDeliveryCompletionV1,
        observed_at: TimestampV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        let mut publication_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut publication_nonce);
        let post_r0_graphs = self.resolve_all_post_r0_registry_graphs(authority)?;
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let applied = storage.transaction(|transaction| {
            let registry_bytes = transaction.read_registry()?.ok_or_else(
                super::host_session_authority::store::BootstrapError::retained_admission_semantic,
            )?;
            let mut registry: RetainedWorkerAdmissionRegistryV1 = retain_semantic_error(
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
            retain_semantic_error(
                validate_complete_post_r0_registry_graphs(
                    &registry,
                    transaction.authority_root(),
                    &post_r0_graphs,
                ),
                &mut semantic_failure,
            )?;
            let (record, changed) = retain_semantic_error(
                record_pending_admission_cancel_delivery_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    completion,
                    &observed_at,
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
        applied.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    pub(crate) fn reconcile_existing_admission(
        &self,
        authority: &HostSessionAuthority,
        presented_plan: &RetainedWorkerAdmissionPlanV1,
        expected_record: &RetainedWorkerAdmissionRecordV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        rejection: Option<&AdmissionRejectedBeforeRegistrationInputV1>,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&presented_plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = presented_plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let canonical_plan = self.canonical_plan_for_existing_admission(
            authority,
            presented_plan,
            expected_record,
            reservation_proof,
        )?;
        let durable = self
            .read_admission_record(
                authority,
                &expected_record.orchestration_session_id,
                &expected_record.retained_participant_id,
            )?
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("exact existing admission is absent".into())
            })?;
        if durable != *expected_record {
            return Err(RetainedWorkerRuntimeError(
                "exact existing admission record changed after its locked join".into(),
            ));
        }
        match &durable.state {
            RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => Ok(durable),
            RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. }
            | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { .. }
            | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                ..
            } => Ok(durable),
            RetainedWorkerAdmissionStateV1::SlotReserved { .. } => {
                let rejection = rejection.ok_or_else(|| {
                    RetainedWorkerRuntimeError(
                        "pre-registration admission resolution requires an explicit terminal rejection"
                            .into(),
                    )
                })?;
                self.reject_existing_admission_before_registration(
                    authority,
                    &current_plan,
                    &canonical_plan,
                    &durable,
                    reservation_proof,
                    rejection,
                )
            }
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. } => {
                let authority_root = VersionedStateRoot::V2(
                    authority
                        .read_preserved_start_root_v2()
                        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?,
                );
                if let Some(registration) =
                    admission_registration_from_hsa(&authority_root, &durable)?
                {
                    let post_r0_graph =
                        self.resolve_admission_registration_graph(authority, &registration)?;
                    let mut publication_nonce = [0_u8; 16];
                    rand::rngs::OsRng.fill_bytes(&mut publication_nonce);
                    self.advance_registration_head_after_r0(
                        authority,
                        &canonical_plan,
                        &durable.retained_participant_id,
                        &post_r0_graph,
                        reservation_proof,
                        publication_nonce,
                    )
                } else {
                    let rejection = rejection.ok_or_else(|| {
                        RetainedWorkerRuntimeError(
                            "pre-registration admission resolution requires an explicit terminal rejection"
                                .into(),
                        )
                    })?;
                    self.reject_existing_admission_before_registration(
                        authority,
                        &current_plan,
                        &canonical_plan,
                        &durable,
                        reservation_proof,
                        rejection,
                    )
                }
            }
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
            | RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
            | RetainedWorkerAdmissionStateV1::Routable { .. }
            | RetainedWorkerAdmissionStateV1::InterruptedNonterminal { .. }
            | RetainedWorkerAdmissionStateV1::Terminal { .. } => {
                self.validate_admitted_record_graph(
                    authority,
                    &canonical_plan,
                    &durable,
                    reservation_proof,
                )?;
                Ok(durable)
            }
        }
    }

    fn reject_existing_admission_before_registration(
        &self,
        authority: &HostSessionAuthority,
        current_plan: &RetainedWorkerAdmissionPlanV1,
        canonical_plan: &RetainedWorkerAdmissionPlanV1,
        expected_record: &RetainedWorkerAdmissionRecordV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        rejection: &AdmissionRejectedBeforeRegistrationInputV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        let typed_history = authority
            .resolve_exact_typed_history(&current_plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let post_r0_graphs = self.resolve_all_post_r0_registry_graphs(authority)?;
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut publication_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut publication_nonce);
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
            retain_semantic_error(
                validate_complete_post_r0_registry_graphs(
                    &registry,
                    transaction.authority_root(),
                    &post_r0_graphs,
                ),
                &mut semantic_failure,
            )?;
            let (record, changed) = retain_semantic_error(
                reject_before_registration_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    current_plan,
                    canonical_plan,
                    expected_record,
                    &envelope.secret_key,
                    rejection,
                    reservation_proof,
                    typed_history.as_ref(),
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

    pub(crate) fn validate_admission_authority_ancestry(
        &self,
        authority: &HostSessionAuthority,
        record: &RetainedWorkerAdmissionRecordV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let current = authority
            .resolve_current_exact(&record.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if current.observation.authority_store_id != record.authority_store_id
            || current.observation.orchestration_session_id != record.orchestration_session_id
        {
            return Err(RetainedWorkerRuntimeError(
                "retained admission authority store or session is inexact".into(),
            ));
        }
        let root = authority
            .read_preserved_start_root_v2()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let current_exact = CanonicalExactCurrentAuthorityV1::from_resolved(&current);
        let typed_history = authority
            .resolve_exact_typed_history(&record.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        validate_admission_to_current_exact_authority_ancestry(
            &root,
            &current_exact,
            record,
            typed_history.as_ref(),
        )
    }

    pub(crate) fn register_admitted_worker(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    ) -> Result<RetainedWorkerAdmissionRegistrationOutcomeV1, RetainedWorkerRuntimeError> {
        self.register_admitted_worker_with(authority, plan, reservation_proof, None)
    }

    #[cfg(test)]
    fn register_admitted_worker_at(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        crash_point: Option<AdmissionRegistrationCrashPointV1>,
    ) -> Result<RetainedWorkerAdmissionRegistrationOutcomeV1, RetainedWorkerRuntimeError> {
        self.register_admitted_worker_with(authority, plan, None, crash_point)
    }

    fn register_admitted_worker_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        #[cfg(test)] crash_point: Option<AdmissionRegistrationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<RetainedWorkerAdmissionRegistrationOutcomeV1, RetainedWorkerRuntimeError> {
        let slot = self.reserve_admission_slot(authority, plan, reservation_proof)?;
        let head_acquired_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| RetainedWorkerRuntimeError("create registration head timestamp".into()))?;
        let mut publication_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut publication_nonce);
        let head = self.prepare_admission_registration_head_with(
            authority,
            plan,
            &slot.record.retained_participant_id,
            reservation_proof,
            head_acquired_at,
            publication_nonce,
            #[cfg(test)]
            crash_point,
            #[cfg(not(test))]
            _crash_point,
        )?;
        if let AdmissionHeadPreparationV1::Complete(record) = head {
            self.validate_admitted_record_graph(authority, plan, &record, reservation_proof)?;
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
        let resolved_target = self.resolve_retained_target(authority, &registration)?;
        let post_r0_graph = ResolvedPostR0RegisteredGraphV1 {
            result: registration,
            resolved_target,
        };
        let mut reconcile_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut reconcile_nonce);
        let record = self.advance_registration_head_after_r0_with(
            authority,
            plan,
            &record.retained_participant_id,
            &post_r0_graph,
            reservation_proof,
            reconcile_nonce,
            #[cfg(test)]
            crash_point,
            #[cfg(not(test))]
            _crash_point,
        )?;
        Ok(RetainedWorkerAdmissionRegistrationOutcomeV1 {
            record,
            joined: slot.joined,
        })
    }

    fn prepare_admission_registration_head(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        head_acquired_at: TimestampV1,
        publication_nonce: [u8; 16],
    ) -> Result<AdmissionHeadPreparationV1, RetainedWorkerRuntimeError> {
        self.prepare_admission_registration_head_with(
            authority,
            plan,
            retained_participant_id,
            reservation_proof,
            head_acquired_at,
            publication_nonce,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_admission_registration_head_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        head_acquired_at: TimestampV1,
        publication_nonce: [u8; 16],
        #[cfg(test)] crash_point: Option<AdmissionRegistrationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<AdmissionHeadPreparationV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let typed_history = authority
            .resolve_exact_typed_history(&plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
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
                    plan,
                    retained_participant_id,
                    reservation_proof,
                    &envelope.secret_key,
                    head_acquired_at,
                    typed_history.as_ref(),
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
                #[cfg(test)]
                if crash_point
                    == Some(AdmissionRegistrationCrashPointV1::BeforeRegistrationHeadPublication)
                {
                    return Err(
                        super::host_session_authority::store::BootstrapError::retained_admission_crash(
                        ),
                    );
                }
                #[cfg(test)]
                if crash_point
                    == Some(AdmissionRegistrationCrashPointV1::DuringRegistrationHeadPublication)
                {
                    transaction.stage_registry_replacement_for_test(&temp_name, &bytes)?;
                    return Err(
                        super::host_session_authority::store::BootstrapError::retained_admission_crash(
                        ),
                    );
                }
                transaction.replace_registry(&temp_name, &bytes)?;
                #[cfg(test)]
                if crash_point
                    == Some(
                        AdmissionRegistrationCrashPointV1::AfterRegistrationHeadPublicationBeforeResponse,
                    )
                {
                    return Err(
                        super::host_session_authority::store::BootstrapError::retained_admission_crash(
                        ),
                    );
                }
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
        post_r0_graph: &ResolvedPostR0RegisteredGraphV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        publication_nonce: [u8; 16],
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        self.advance_registration_head_after_r0_with(
            authority,
            plan,
            retained_participant_id,
            post_r0_graph,
            reservation_proof,
            publication_nonce,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn advance_registration_head_after_r0_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        post_r0_graph: &ResolvedPostR0RegisteredGraphV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        publication_nonce: [u8; 16],
        #[cfg(test)] crash_point: Option<AdmissionRegistrationCrashPointV1>,
        #[cfg(not(test))] _crash_point: Option<()>,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let typed_history = authority
            .resolve_exact_typed_history(&plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
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
                    plan,
                    &AdmissionRegistrationAdvanceInputV1 {
                        retained_participant_id,
                        secret_key: &envelope.secret_key,
                        post_r0_graph,
                    },
                    reservation_proof,
                    typed_history.as_ref(),
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
                #[cfg(test)]
                if crash_point
                    == Some(AdmissionRegistrationCrashPointV1::BeforeAdmissionAdvancePublication)
                {
                    return Err(
                        super::host_session_authority::store::BootstrapError::retained_admission_crash(
                        ),
                    );
                }
                #[cfg(test)]
                if crash_point
                    == Some(AdmissionRegistrationCrashPointV1::DuringAdmissionAdvancePublication)
                {
                    transaction.stage_registry_replacement_for_test(&temp_name, &bytes)?;
                    return Err(
                        super::host_session_authority::store::BootstrapError::retained_admission_crash(
                        ),
                    );
                }
                transaction.replace_registry(&temp_name, &bytes)?;
                #[cfg(test)]
                if crash_point
                    == Some(
                        AdmissionRegistrationCrashPointV1::AfterAdmissionAdvancePublicationBeforeResponse,
                    )
                {
                    return Err(
                        super::host_session_authority::store::BootstrapError::retained_admission_crash(
                        ),
                    );
                }
            }
            Ok(record)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    fn resolve_admitted_record_graph(
        &self,
        authority: &HostSessionAuthority,
        record: &RetainedWorkerAdmissionRecordV1,
    ) -> Result<ResolvedPostR0RegisteredGraphV1, RetainedWorkerRuntimeError> {
        let admission_registration = admission_registration(&record.state).ok_or_else(|| {
            RetainedWorkerRuntimeError("post-R0 admission has no registration".into())
        })?;
        self.resolve_admission_registration_graph(authority, admission_registration)
    }

    fn resolve_admission_registration_graph(
        &self,
        authority: &HostSessionAuthority,
        admission_registration: &RetainedWorkerAdmissionRegistrationV1,
    ) -> Result<ResolvedPostR0RegisteredGraphV1, RetainedWorkerRuntimeError> {
        let root = authority
            .read_preserved_start_root_v2()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let registration = root
            .retained_worker_registration_journal
            .get(&admission_registration.registration_id)
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("post-R0 admission registration is absent".into())
            })?;
        let result = RetainedWorkerRegistrationResultV1 {
            registration_id: registration.registration_id.clone(),
            registration_commitment: canonical_registration_commitment(registration)?,
            authority_store_id: root.authority_store_id.clone(),
            orchestration_session_id: registration.orchestration_session_id.clone(),
            retained_participant_id: registration.retained_participant_id.clone(),
            retained_worker_ref: registration.retained_worker_ref.clone(),
            authority_revision_after: registration.authority_revision_after,
            authority_record_commitment_after: registration
                .authority_record_commitment_after
                .clone(),
        };
        let resolved = self.resolve_retained_target(authority, &result)?;
        Ok(ResolvedPostR0RegisteredGraphV1 {
            result,
            resolved_target: resolved,
        })
    }

    fn validate_admitted_record_graph(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        record: &RetainedWorkerAdmissionRecordV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let post_r0_graph = self.resolve_admitted_record_graph(authority, record)?;
        let resolved_authority = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority =
            CanonicalExactCurrentAuthorityV1::from_resolved(&resolved_authority);
        let typed_history = authority
            .resolve_exact_typed_history(&plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let root = authority
            .read_preserved_start_root_v2()
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let versioned_root = VersionedStateRoot::V2(root.clone());
        validate_admission_plan(&versioned_root, &current_plan, reservation_proof)?;
        validate_reservation_proof_identities(reservation_proof, record)?;
        let registration = admission_registration(&record.state).ok_or_else(|| {
            RetainedWorkerRuntimeError("post-R0 admission has no registration".into())
        })?;
        validate_post_r0_registered_graph(
            &root,
            record,
            &current_plan,
            registration,
            &post_r0_graph.result,
            &post_r0_graph.resolved_target,
            typed_history.as_ref(),
        )
    }

    fn resolve_all_post_r0_registry_graphs(
        &self,
        authority: &HostSessionAuthority,
    ) -> Result<BTreeMap<(String, String), ResolvedPostR0RegistryGraphV1>, RetainedWorkerRuntimeError>
    {
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut semantic_failure = None;
        let registrations = storage.transaction(|transaction| {
            let Some(registry_bytes) = transaction.read_registry()? else {
                return Ok(Vec::new());
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
            let mut found = Vec::new();
            for record in registry
                .records_by_session
                .values()
                .flat_map(BTreeMap::values)
            {
                let registration = if let Some(registration) = admission_registration(&record.state)
                {
                    Some(registration.clone())
                } else if matches!(
                    record.state,
                    RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
                        | RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                            registration_head: Some(_),
                            ..
                        }
                ) {
                    retain_semantic_error(
                        admission_registration_from_hsa(transaction.authority_root(), record),
                        &mut semantic_failure,
                    )?
                } else {
                    None
                };
                if let Some(registration) = registration {
                    found.push((record.clone(), registration));
                }
            }
            Ok(found)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        let registrations =
            registrations.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut graphs = BTreeMap::new();
        for (record, registration) in registrations {
            let registered_graph =
                self.resolve_admission_registration_graph(authority, &registration)?;
            let current = authority
                .resolve_current_exact(&record.orchestration_session_id, None)
                .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
            let typed_authority_history = authority
                .resolve_exact_typed_history(&record.orchestration_session_id)
                .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
            let key = (
                record.orchestration_session_id.clone(),
                record.retained_participant_id.clone(),
            );
            if graphs
                .insert(
                    key,
                    ResolvedPostR0RegistryGraphV1 {
                        current_exact_authority: CanonicalExactCurrentAuthorityV1::from_resolved(
                            &current,
                        ),
                        typed_authority_history,
                        registered_graph,
                    },
                )
                .is_some()
            {
                return Err(RetainedWorkerRuntimeError(
                    "post-R0 registry graph identity is duplicated".into(),
                ));
            }
        }
        Ok(graphs)
    }

    pub(crate) fn claim_admission_transport(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
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
            reservation_proof,
            &AdmissionTransportPublicationInputV1 {
                claimed_at,
                claim_entropy,
                publication_nonce,
                #[cfg(test)]
                crash_point: None,
            },
        )
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    fn claim_admission_transport_at(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        claimed_at: TimestampV1,
        claim_entropy: [u8; 16],
        publication_nonce: [u8; 16],
        crash_point: Option<AdmissionTransportClaimCrashPointV1>,
    ) -> Result<RetainedWorkerTransportClaimV1, RetainedWorkerRuntimeError> {
        self.claim_admission_transport_with(
            authority,
            plan,
            retained_participant_id,
            None,
            &AdmissionTransportPublicationInputV1 {
                claimed_at,
                claim_entropy,
                publication_nonce,
                crash_point,
            },
        )
    }

    fn claim_admission_transport_with(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        publication: &AdmissionTransportPublicationInputV1,
    ) -> Result<RetainedWorkerTransportClaimV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let typed_history = authority
            .resolve_exact_typed_history(&plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let post_r0_graph = self
            .read_admission_record(
                authority,
                &plan.spawn_request.orchestration_session_id,
                retained_participant_id,
            )?
            .filter(|record| admission_registration(&record.state).is_some())
            .map(|record| self.resolve_admitted_record_graph(authority, &record))
            .transpose()?;
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
                    plan,
                    retained_participant_id,
                    &envelope.secret_key,
                    &AdmissionTransportClaimInputV1 {
                        claimed_at: publication.claimed_at.clone(),
                        claim_entropy: publication.claim_entropy,
                        post_r0_graph: post_r0_graph.as_ref(),
                    },
                    reservation_proof,
                    typed_history.as_ref(),
                ),
                &mut semantic_failure,
            )?;
            if claim.newly_claimed {
                #[cfg(test)]
                if publication.crash_point
                    == Some(AdmissionTransportClaimCrashPointV1::BeforeTempPersistence)
                {
                    retain_semantic_error(
                        Err(RetainedWorkerRuntimeError(
                            "injected crash before transport-claim temp persistence".into(),
                        )),
                        &mut semantic_failure,
                    )?;
                }
                let bytes = retain_semantic_error(
                    encode_canonical(&registry, "encode admission registry"),
                    &mut semantic_failure,
                )?;
                let temp_name = format!(
                    "admission-registry--{}.tmp",
                    lower_hex(&publication.publication_nonce)
                );
                #[cfg(test)]
                if matches!(
                    publication.crash_point,
                    Some(
                        AdmissionTransportClaimCrashPointV1::AfterTempFsync
                            | AdmissionTransportClaimCrashPointV1::BeforeRegistryReplacementPublication
                    )
                )
                {
                    transaction.stage_registry_replacement_for_test(&temp_name, &bytes)?;
                    let message = if publication.crash_point
                        == Some(AdmissionTransportClaimCrashPointV1::AfterTempFsync)
                    {
                        "injected crash after transport-claim temp fsync"
                    } else {
                        "injected crash before transport-claim registry publication"
                    };
                    retain_semantic_error(
                        Err(RetainedWorkerRuntimeError(message.into())),
                        &mut semantic_failure,
                    )?;
                }
                transaction.replace_registry(&temp_name, &bytes)?;
                #[cfg(test)]
                if publication.crash_point
                    == Some(AdmissionTransportClaimCrashPointV1::AfterPublicationBeforeResponse)
                {
                    retain_semantic_error(
                        Err(RetainedWorkerRuntimeError(
                            "injected crash after transport-claim publication before response"
                                .into(),
                        )),
                        &mut semantic_failure,
                    )?;
                }
            }
            Ok(claim)
        });
        if let Some(error) = semantic_failure {
            return Err(error);
        }
        result.map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }

    pub(crate) fn launch_authority_proof_for_claim(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        claim: &RetainedWorkerTransportClaimV1,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    ) -> Result<RetainedWorkerLaunchAuthorityProofV1, RetainedWorkerRuntimeError> {
        let durable = self
            .read_admission_record(
                authority,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )?
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("transport claim admission record is absent".into())
            })?;
        if durable != claim.record {
            return Err(RetainedWorkerRuntimeError(
                "transport claim is not the exact durable admission record".into(),
            ));
        }
        let RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
            registration,
            transport_claim_id,
            ..
        } = &durable.state
        else {
            return Err(RetainedWorkerRuntimeError(
                "launch proof requires the exact durable transport claim".into(),
            ));
        };
        self.validate_admitted_record_graph(authority, plan, &durable, reservation_proof)?;
        let graph = self.resolve_admitted_record_graph(authority, &durable)?;
        if registration.registration_id != graph.result.registration_id
            || registration.retained_worker_ref != graph.result.retained_worker_ref
            || durable.issuer_request_id != plan.issuer_request_id
            || durable.orchestration_session_id != plan.spawn_request.orchestration_session_id
            || durable.retained_participant_id != graph.result.retained_participant_id
            || durable.backend_id != plan.descriptor_and_runtime_plan.descriptor.backend_id
            || durable.protocol != plan.descriptor_and_runtime_plan.descriptor.protocol
            || durable.world_binding.world_id != plan.spawn_request.world_id
            || durable.world_binding.world_generation != plan.spawn_request.world_generation
            || durable.current_policy_ref != plan.policy_and_admission_cap.current_policy_ref
            || durable.current_policy_revision
                != plan.policy_and_admission_cap.current_policy.policy_revision
        {
            return Err(RetainedWorkerRuntimeError(
                "launch proof inputs conflict with the exact admission/R0 graph".into(),
            ));
        }

        let proof = RetainedWorkerLaunchAuthorityProofV1 {
            schema_version: 1,
            authority_store_id: durable.authority_store_id.clone(),
            issuer_request_id: durable.issuer_request_id.clone(),
            canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
                schema_version: durable.canonical_spawn_fingerprint.schema_version,
                algorithm: "hmac-sha-256".to_string(),
                key_id: durable.canonical_spawn_fingerprint.key_id.clone(),
                digest_hex: durable.canonical_spawn_fingerprint.digest_hex.clone(),
            },
            registration_id: graph.result.registration_id.clone(),
            registration_commitment: project_launch_commitment(
                &graph.result.registration_commitment,
            ),
            authority_revision_after: graph.result.authority_revision_after,
            authority_record_commitment_after: project_launch_commitment(
                &graph.result.authority_record_commitment_after,
            ),
            orchestration_session_id: durable.orchestration_session_id.clone(),
            caller_participant_id: plan.spawn_request.caller_participant_id.clone(),
            retained_participant_id: durable.retained_participant_id.clone(),
            bootstrap_run_id: durable.bootstrap_run_id.clone(),
            transport_claim_id: transport_claim_id.clone(),
            backend_id: durable.backend_id.clone(),
            protocol: durable.protocol.clone(),
            world_binding: RetainedWorkerLaunchWorldBindingV1 {
                world_id: durable.world_binding.world_id.clone(),
                world_generation: durable.world_binding.world_generation,
            },
            current_policy_ref_id: durable.current_policy_ref.ref_id.clone(),
            current_policy_revision: durable.current_policy_revision.clone(),
            retained_worker_ref_id: graph.result.retained_worker_ref.ref_id.clone(),
            retained_worker_commitment: project_launch_commitment(
                &graph.result.retained_worker_ref.commitment,
            ),
        };
        proof.validate().map_err(RetainedWorkerRuntimeError)?;
        Ok(proof)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn mark_admission_routable(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        frame_identity: &RuntimeFrameIdentityV1,
        event: &AgentEvent,
        registered_at: TimestampV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        self.mark_admission_routable_outcome(
            authority,
            plan,
            retained_participant_id,
            reservation_proof,
            frame_identity,
            event,
            registered_at,
        )
        .map(|outcome| outcome.record)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn mark_admission_routable_outcome(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        frame_identity: &RuntimeFrameIdentityV1,
        event: &AgentEvent,
        registered_at: TimestampV1,
    ) -> Result<RetainedWorkerAdmissionRoutabilityOutcomeV1, RetainedWorkerRuntimeError> {
        let record = self.publish_admission_runtime_truth(
            authority,
            plan,
            retained_participant_id,
            reservation_proof,
            &AdmissionRuntimeTruthInputV1::Registered {
                frame_identity,
                event,
                registered_at,
            },
        )?;
        let disposition = match &record.state {
            RetainedWorkerAdmissionStateV1::Routable { .. } => {
                RetainedWorkerAdmissionRoutabilityDispositionV1::Routable
            }
            RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                cancel_request_id,
                ..
            }
            | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
                cancel_request_id, ..
            }
            | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                cancel_request_id,
                ..
            } => RetainedWorkerAdmissionRoutabilityDispositionV1::CancellationWon {
                cancel_request_id: cancel_request_id.clone(),
            },
            RetainedWorkerAdmissionStateV1::Terminal { .. }
            | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => {
                RetainedWorkerAdmissionRoutabilityDispositionV1::AlreadyTerminal
            }
            _ => {
                return Err(RetainedWorkerRuntimeError(
                    "Registered runtime truth did not reach a canonical routability disposition"
                        .into(),
                ));
            }
        };
        Ok(RetainedWorkerAdmissionRoutabilityOutcomeV1 {
            record,
            disposition,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn observe_admission_transport_start(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        frame_identity: &RuntimeFrameIdentityV1,
        transport_span_id: &str,
    ) -> Result<Option<RetainedWorkerAdmissionTransportCancellationV1>, RetainedWorkerRuntimeError>
    {
        let record = self.publish_admission_runtime_truth(
            authority,
            plan,
            retained_participant_id,
            reservation_proof,
            &AdmissionRuntimeTruthInputV1::TransportStarted {
                frame_identity,
                transport_span_id,
            },
        )?;
        self.claim_pending_admission_cancel_delivery(authority, plan, &record)
    }

    pub(crate) fn claim_pending_admission_cancel_delivery(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        record: &RetainedWorkerAdmissionRecordV1,
    ) -> Result<Option<RetainedWorkerAdmissionTransportCancellationV1>, RetainedWorkerRuntimeError>
    {
        let RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            cancel_request_id,
            ..
        } = &record.state
        else {
            return Ok(None);
        };
        let outcome = self.cancel_pending_admission(
            authority,
            &RetainedWorkerAdmissionCancelRequestV1 {
                schema_version: 1,
                cancel_request_id: cancel_request_id.clone(),
                authority_store_id: record.authority_store_id.clone(),
                issuer_request_id: record.issuer_request_id.clone(),
                orchestration_session_id: record.orchestration_session_id.clone(),
                caller_participant_id: plan.spawn_request.caller_participant_id.clone(),
                retained_participant_id: record.retained_participant_id.clone(),
                bootstrap_run_id: record.bootstrap_run_id.clone(),
                backend_id: record.backend_id.clone(),
                protocol: record.protocol.clone(),
                world_binding: record.world_binding.clone(),
                current_policy_ref: record.current_policy_ref.clone(),
                current_policy_revision: record.current_policy_revision.clone(),
                expected_record_revision: record.record_revision,
                expected_state: record.state.clone(),
            },
        )?;
        match outcome {
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                record,
                cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        delivery_claim_id,
                        transport_span_id,
                    },
                ..
            } => Ok(Some(RetainedWorkerAdmissionTransportCancellationV1 {
                authority_store_id: record.authority_store_id,
                orchestration_session_id: record.orchestration_session_id,
                retained_participant_id: record.retained_participant_id,
                cancel_request_id,
                transport_span_id,
                delivery_claim_id,
            })),
            _ => Ok(None),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn mark_admission_terminal(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        frame_identity: &RuntimeFrameIdentityV1,
        event_identity: &RuntimeEventIdentityV1,
        terminal_identity: &RuntimeTerminalIdentityV1,
        exit_code: i32,
        terminal_at: TimestampV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        self.publish_admission_runtime_truth(
            authority,
            plan,
            retained_participant_id,
            reservation_proof,
            &AdmissionRuntimeTruthInputV1::Terminal {
                frame_identity,
                event_identity,
                terminal_identity,
                transport_span_id: None,
                exit_code,
                terminal_at,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn mark_admission_terminal_for_transport(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        frame_identity: &RuntimeFrameIdentityV1,
        event_identity: &RuntimeEventIdentityV1,
        terminal_identity: &RuntimeTerminalIdentityV1,
        transport_span_id: &str,
        exit_code: i32,
        terminal_at: TimestampV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        self.publish_admission_runtime_truth(
            authority,
            plan,
            retained_participant_id,
            reservation_proof,
            &AdmissionRuntimeTruthInputV1::Terminal {
                frame_identity,
                event_identity,
                terminal_identity,
                transport_span_id: Some(transport_span_id),
                exit_code,
                terminal_at,
            },
        )
    }

    pub(crate) fn mark_admission_interrupted(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        last_frame_identity: Option<&RuntimeFrameIdentityV1>,
        interrupted_at: TimestampV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        self.publish_admission_runtime_truth(
            authority,
            plan,
            retained_participant_id,
            reservation_proof,
            &AdmissionRuntimeTruthInputV1::Interrupted {
                last_frame_identity,
                interrupted_at,
            },
        )
    }

    fn publish_admission_runtime_truth(
        &self,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        retained_participant_id: &str,
        reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
        truth: &AdmissionRuntimeTruthInputV1<'_>,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        let resolved = authority
            .resolve_current_exact(&plan.spawn_request.orchestration_session_id, None)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let typed_history = authority
            .resolve_exact_typed_history(&plan.spawn_request.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let post_r0_graphs = self.resolve_all_post_r0_registry_graphs(authority)?;
        let storage =
            super::host_session_authority::store::retained_worker_admission_storage_for_authority(
                authority,
            )
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let authority_store_id = storage.authority_store_id().to_owned();
        let mut publication_nonce = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut publication_nonce);
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
            retain_semantic_error(
                validate_complete_post_r0_registry_graphs(
                    &registry,
                    transaction.authority_root(),
                    &post_r0_graphs,
                ),
                &mut semantic_failure,
            )?;
            let graph = post_r0_graphs
                .get(&(
                    plan.spawn_request.orchestration_session_id.clone(),
                    retained_participant_id.to_owned(),
                ))
                .ok_or_else(
                    super::host_session_authority::store::BootstrapError::retained_admission_semantic,
                )?;
            let (record, changed) = retain_semantic_error(
                advance_admission_runtime_truth_in_registry(
                    &mut registry,
                    transaction.authority_root(),
                    &current_plan,
                    plan,
                    retained_participant_id,
                    &envelope.secret_key,
                    &graph.registered_graph,
                    truth,
                    reservation_proof,
                    typed_history.as_ref(),
                ),
                &mut semantic_failure,
            )?;
            if changed {
                let bytes = retain_semantic_error(
                    encode_canonical(&registry, "encode admission registry"),
                    &mut semantic_failure,
                )?;
                transaction.replace_registry(
                    &format!(
                        "admission-registry--{}.tmp",
                        lower_hex(&publication_nonce)
                    ),
                    &bytes,
                )?;
            }
            Ok(record)
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
        let registration_commitment = canonical_registration_commitment(&applied.registration)?;
        Ok(RetainedWorkerRegistrationResultV1 {
            registration_id: applied.registration.registration_id.clone(),
            registration_commitment,
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
            .read_preserved_start_root_v2()
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
        let registration_commitment = canonical_registration_commitment(registration)?;
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
        if registration_commitment != target.registration_commitment
            || registration.orchestration_session_id != target.orchestration_session_id
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
        let typed_history = authority
            .resolve_exact_typed_history(&target.orchestration_session_id)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        if let Some(typed_history) = typed_history {
            let authority_before = typed_history
                .get(&registration.authority_revision_before)
                .ok_or_else(|| {
                    RetainedWorkerRuntimeError(
                        "retained target registration predecessor is absent from typed ancestry"
                            .into(),
                    )
                })?;
            let authority_after = typed_history
                .get(&registration.authority_revision_after)
                .ok_or_else(|| {
                    RetainedWorkerRuntimeError(
                        "retained target registration is absent from typed ancestry".into(),
                    )
                })?;
            let exact_current = typed_history
                .get(&current.authority.authority_revision)
                .ok_or_else(|| {
                    RetainedWorkerRuntimeError(
                        "retained target typed ancestry does not reach current authority".into(),
                    )
                })?;
            if authority_before.observation().authority_record_commitment
                != registration.authority_record_commitment_before
                || authority_after.observation().authority_record_commitment
                    != registration.authority_record_commitment_after
                || authority_after
                    .authority
                    .retained_worker_refs
                    .iter()
                    .filter(|reference| *reference == &target.retained_worker_ref)
                    .count()
                    != 1
                || exact_current.authority != current.authority
                || exact_current.observation() != current.observation
            {
                return Err(RetainedWorkerRuntimeError(
                    "retained target registration is not on exact current typed ancestry".into(),
                ));
            }
        } else {
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

fn preserved_start_root_view(
    authority_root: &VersionedStateRoot,
) -> Result<Cow<'_, StateRootV2>, RetainedWorkerRuntimeError> {
    match authority_root {
        VersionedStateRoot::V2(root) => Ok(Cow::Borrowed(root)),
        VersionedStateRoot::V3(root) => Ok(Cow::Owned(root.preserved_v2_view())),
        VersionedStateRoot::V1(_) => Err(RetainedWorkerRuntimeError(
            "retained admission requires strict V2 or preserved V3 authority".into(),
        )),
    }
}

fn validate_supplied_admission_authority(
    root: &StateRootV2,
    current_exact_authority: &CanonicalExactCurrentAuthorityV1,
    supplied_exact_authority: &CanonicalExactCurrentAuthorityV1,
    record: &RetainedWorkerAdmissionRecordV1,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(), RetainedWorkerRuntimeError> {
    let admission_authority = reconstruct_exact_authority_at_revision(
        root,
        current_exact_authority,
        record.admission_authority_revision,
        typed_history,
    )?;
    if supplied_exact_authority != &admission_authority {
        return Err(RetainedWorkerRuntimeError(
            "canonical admission authority proof is not the exact admission-time bound read".into(),
        ));
    }
    Ok(())
}

fn validate_admission_to_current_exact_authority_ancestry(
    root: &StateRootV2,
    current_exact_authority: &CanonicalExactCurrentAuthorityV1,
    record: &RetainedWorkerAdmissionRecordV1,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(), RetainedWorkerRuntimeError> {
    if let Some(history) = typed_history {
        let admission_authority = history
            .get(&record.admission_authority_revision)
            .ok_or_else(|| {
                RetainedWorkerRuntimeError(
                    "admission authority revision is absent from exact typed ancestry".into(),
                )
            })?;
        let admission_observation = admission_authority.observation();
        if admission_observation.authority_store_id != current_exact_authority.authority_store_id
            || admission_observation.orchestration_session_id
                != current_exact_authority.orchestration_session_id
            || admission_observation.authority_revision != record.admission_authority_revision
            || admission_observation.authority_record_commitment
                != record.admission_authority_record_commitment
        {
            return Err(RetainedWorkerRuntimeError(
                "admission-time authority is not the exact recorded typed ancestor".into(),
            ));
        }
        let reconstructed_current = reconstruct_exact_authority_at_revision(
            root,
            current_exact_authority,
            current_exact_authority.authority_revision,
            Some(history),
        )?;
        if &reconstructed_current != current_exact_authority {
            return Err(RetainedWorkerRuntimeError(
                "admission-to-current authority interval is not complete exact typed ancestry"
                    .into(),
            ));
        }
        return Ok(());
    }

    let admission_authority = reconstruct_exact_authority_at_revision(
        root,
        current_exact_authority,
        record.admission_authority_revision,
        None,
    )?;
    if admission_authority.authority_record_commitment
        != record.admission_authority_record_commitment
    {
        return Err(RetainedWorkerRuntimeError(
            "admission-time authority is not the exact recorded typed ancestor".into(),
        ));
    }
    let reconstructed_current = reconstruct_exact_authority_at_revision(
        root,
        current_exact_authority,
        current_exact_authority.authority_revision,
        None,
    )?;
    if &reconstructed_current != current_exact_authority {
        return Err(RetainedWorkerRuntimeError(
            "admission-to-current authority interval is not complete exact typed ancestry".into(),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn prepare_registration_head_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    current_plan: &RetainedWorkerAdmissionPlanV1,
    supplied_plan: &RetainedWorkerAdmissionPlanV1,
    retained_participant_id: &str,
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    secret_key: &[u8; 32],
    head_acquired_at: TimestampV1,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(AdmissionHeadPreparationV1, bool), RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, current_plan, reservation_proof)?;
    let root = preserved_start_root_view(authority_root)?;
    let mut record = registry
        .records_by_session
        .get(&current_plan.spawn_request.orchestration_session_id)
        .and_then(|records| records.get(retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot is absent".into()))?;
    validate_reservation_proof_identities(reservation_proof, &record)?;
    validate_supplied_admission_authority(
        root.as_ref(),
        &current_plan.exact_authority,
        &supplied_plan.exact_authority,
        &record,
        typed_history,
    )?;
    verify_admission_record_fingerprint(supplied_plan, &record, secret_key)?;
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
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { .. }
        | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. } => {
            return Err(RetainedWorkerRuntimeError(
                "admission cancellation prevents registration".into(),
            ));
        }
        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. } => {
            admission_registration_from_hsa(authority_root, &record)?;
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
    validate_admission_to_current_exact_authority_ancestry(
        root.as_ref(),
        &current_plan.exact_authority,
        &record,
        typed_history,
    )?;
    record.state = RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
        authority_revision_expected: current_plan.exact_authority.authority_revision,
        authority_record_commitment_expected: current_plan
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

fn cancel_delivery_claim_expires_at(
    claimed_at: &TimestampV1,
) -> Result<TimestampV1, RetainedWorkerRuntimeError> {
    let parsed = chrono::DateTime::parse_from_rfc3339(claimed_at.as_str())
        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
    let expires_at = parsed
        .checked_add_signed(chrono::Duration::seconds(
            RETAINED_ADMISSION_CANCEL_DELIVERY_CLAIM_LEASE_SECONDS,
        ))
        .ok_or_else(|| {
            RetainedWorkerRuntimeError(
                "admission cancellation delivery claim expiry overflow".into(),
            )
        })?
        .with_timezone(&chrono::Utc)
        .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    TimestampV1::parse(expires_at).map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
}

fn claim_pending_admission_cancel_delivery_in_record(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    record: &RetainedWorkerAdmissionRecordV1,
    claimed_at: &TimestampV1,
    claim_nonce: &[u8; 16],
) -> Result<Option<String>, RetainedWorkerRuntimeError> {
    let RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
        transport_span_id,
        cancel_request_id,
        accepted_at,
        ..
    } = &record.state
    else {
        return Ok(None);
    };
    let Some(transport_span_id) = transport_span_id.as_deref() else {
        return Ok(None);
    };
    let claimed_at = if claimed_at.as_str() < accepted_at.as_str() {
        accepted_at
    } else {
        claimed_at
    };
    let existing = registry
        .cancel_deliveries_by_request
        .get(cancel_request_id)
        .cloned();
    if existing.as_ref().is_some_and(|delivery| {
        delivery.schema_version != 1
            || delivery.authority_store_id != record.authority_store_id
            || delivery.orchestration_session_id != record.orchestration_session_id
            || delivery.retained_participant_id != record.retained_participant_id
            || delivery.cancel_request_id != *cancel_request_id
            || delivery.transport_span_id != transport_span_id
            || delivery.accepted_at != *accepted_at
    }) {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation delivery does not exact-join accepted cancellation truth"
                .into(),
        ));
    }
    let claim_is_available = match existing.as_ref().map(|delivery| &delivery.delivery_state) {
        None | Some(RetainedWorkerAdmissionCancelDeliveryStateV1::Available) => true,
        Some(RetainedWorkerAdmissionCancelDeliveryStateV1::Claimed {
            claim_expires_at, ..
        }) => claimed_at.as_str() >= claim_expires_at.as_str(),
        Some(RetainedWorkerAdmissionCancelDeliveryStateV1::Confirmed { .. }) => false,
    };
    if !claim_is_available {
        return Ok(None);
    }
    if transport_span_id.trim().is_empty() {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation delivery span is empty".into(),
        ));
    }
    let delivery_claim_id = format!("rcdc_{}", lower_hex(claim_nonce));
    if registry
        .cancel_deliveries_by_request
        .values()
        .any(|delivery| {
            matches!(
                &delivery.delivery_state,
                RetainedWorkerAdmissionCancelDeliveryStateV1::Claimed {
                    delivery_claim_id: existing,
                    ..
                } | RetainedWorkerAdmissionCancelDeliveryStateV1::Confirmed {
                    delivery_claim_id: existing,
                    ..
                } if existing == &delivery_claim_id
            )
        })
    {
        return Err(RetainedWorkerRuntimeError(
            "generated admission cancellation delivery claim identity collides".into(),
        ));
    }
    registry.cancel_deliveries_by_request.insert(
        cancel_request_id.clone(),
        RetainedWorkerAdmissionCancelDeliveryRecordV1 {
            schema_version: 1,
            authority_store_id: record.authority_store_id.clone(),
            orchestration_session_id: record.orchestration_session_id.clone(),
            retained_participant_id: record.retained_participant_id.clone(),
            cancel_request_id: cancel_request_id.clone(),
            transport_span_id: transport_span_id.to_owned(),
            accepted_at: accepted_at.clone(),
            delivery_state: RetainedWorkerAdmissionCancelDeliveryStateV1::Claimed {
                delivery_claim_id: delivery_claim_id.clone(),
                claimed_at: claimed_at.clone(),
                claim_expires_at: cancel_delivery_claim_expires_at(claimed_at)?,
            },
        },
    );
    Ok(Some(delivery_claim_id))
}

fn reject_before_registration_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    current_plan: &RetainedWorkerAdmissionPlanV1,
    supplied_plan: &RetainedWorkerAdmissionPlanV1,
    expected_record: &RetainedWorkerAdmissionRecordV1,
    secret_key: &[u8; 32],
    rejection: &AdmissionRejectedBeforeRegistrationInputV1,
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(RetainedWorkerAdmissionRecordV1, bool), RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, current_plan, reservation_proof)?;
    let root = preserved_start_root_view(authority_root)?;
    let mut record = registry
        .records_by_session
        .get(&expected_record.orchestration_session_id)
        .and_then(|records| records.get(&expected_record.retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("exact existing admission is absent".into()))?;
    if &record != expected_record {
        return Err(RetainedWorkerRuntimeError(
            "exact existing admission record changed after its locked join".into(),
        ));
    }
    validate_reservation_proof_identities(reservation_proof, &record)?;
    validate_supplied_admission_authority(
        root.as_ref(),
        &current_plan.exact_authority,
        &supplied_plan.exact_authority,
        &record,
        typed_history,
    )?;
    verify_admission_record_fingerprint(supplied_plan, &record, secret_key)?;
    match &record.state {
        RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => {
            validate_admission_registry(registry, authority_root)?;
            return Ok((record, false));
        }
        RetainedWorkerAdmissionStateV1::SlotReserved { .. }
        | RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. } => {}
        RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::Routable { .. }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::Terminal { .. } => {
            return Err(RetainedWorkerRuntimeError(
                "only pre-registration admissions may be rejected before registration".into(),
            ));
        }
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { .. }
        | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. } => {
            return Err(RetainedWorkerRuntimeError(
                "cancelled admission cannot be rejected again".into(),
            ));
        }
    }
    record.state = RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration {
        reason: rejection.reason.clone(),
        rejected_at: rejection.rejected_at.clone(),
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
    Ok((record, true))
}

const RETAINED_ADMISSION_CANCEL_DELIVERY_CLAIM_LEASE_SECONDS: i64 = 60;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RetainedWorkerAdmissionCancelDeliveryDispositionV1 {
    Unavailable,
    Claimed {
        delivery_claim_id: String,
        transport_span_id: String,
    },
    InProgress {
        delivery_claim_id: String,
        transport_span_id: String,
        claim_expires_at: TimestampV1,
    },
    Confirmed {
        transport_span_id: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetainedWorkerAdmissionCancelDeliveryResultV1 {
    ConfirmedDelivered,
    ConfirmedNotDelivered,
    Ambiguous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAdmissionCancelDeliveryCompletionV1 {
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) retained_participant_id: String,
    pub(crate) cancel_request_id: String,
    pub(crate) transport_span_id: String,
    pub(crate) delivery_claim_id: String,
    pub(crate) result: RetainedWorkerAdmissionCancelDeliveryResultV1,
}

fn record_pending_admission_cancel_delivery_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    completion: &RetainedWorkerAdmissionCancelDeliveryCompletionV1,
    observed_at: &TimestampV1,
) -> Result<(RetainedWorkerAdmissionRecordV1, bool), RetainedWorkerRuntimeError> {
    if completion.authority_store_id != registry.authority_store_id
        || completion.cancel_request_id.trim().is_empty()
        || completion.transport_span_id.trim().is_empty()
        || !valid_generated_id(&completion.delivery_claim_id, "rcdc_")
    {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation delivery result identity is invalid".into(),
        ));
    }
    let mut record = registry
        .records_by_session
        .get(&completion.orchestration_session_id)
        .and_then(|records| records.get(&completion.retained_participant_id))
        .cloned()
        .ok_or_else(|| {
            RetainedWorkerRuntimeError(
                "exact admission cancellation delivery target is absent".into(),
            )
        })?;
    let record_cancel_request_id = match &record.state {
        RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            cancel_request_id,
            ..
        } => Some(cancel_request_id),
        RetainedWorkerAdmissionStateV1::Terminal {
            cancel_request_id: Some(cancel_request_id),
            ..
        } => Some(cancel_request_id),
        _ => None,
    };
    if record.authority_store_id != completion.authority_store_id
        || record.orchestration_session_id != completion.orchestration_session_id
        || record.retained_participant_id != completion.retained_participant_id
        || record_cancel_request_id != Some(&completion.cancel_request_id)
    {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation delivery result does not exact-join durable admission truth"
                .into(),
        ));
    }
    let delivery = registry
        .cancel_deliveries_by_request
        .get(&completion.cancel_request_id)
        .cloned()
        .ok_or_else(|| {
            RetainedWorkerRuntimeError(
                "admission cancellation delivery result has no durable claim".into(),
            )
        })?;
    if delivery.authority_store_id != completion.authority_store_id
        || delivery.orchestration_session_id != completion.orchestration_session_id
        || delivery.retained_participant_id != completion.retained_participant_id
        || delivery.cancel_request_id != completion.cancel_request_id
        || delivery.transport_span_id != completion.transport_span_id
    {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation delivery result changed immutable delivery identity".into(),
        ));
    }
    let next_state = match &delivery.delivery_state {
        RetainedWorkerAdmissionCancelDeliveryStateV1::Claimed {
            delivery_claim_id,
            claim_expires_at,
            ..
        } if delivery_claim_id == &completion.delivery_claim_id
            && observed_at.as_str() < claim_expires_at.as_str() =>
        {
            match completion.result {
                RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedDelivered => {
                    RetainedWorkerAdmissionCancelDeliveryStateV1::Confirmed {
                        delivery_claim_id: completion.delivery_claim_id.clone(),
                        confirmed_at: observed_at.clone(),
                    }
                }
                RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedNotDelivered => {
                    RetainedWorkerAdmissionCancelDeliveryStateV1::Available
                }
                RetainedWorkerAdmissionCancelDeliveryResultV1::Ambiguous => {
                    return Ok((record, false));
                }
            }
        }
        RetainedWorkerAdmissionCancelDeliveryStateV1::Confirmed {
            delivery_claim_id, ..
        } if delivery_claim_id == &completion.delivery_claim_id
            && completion.result
                == RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedDelivered =>
        {
            return Ok((record, false));
        }
        RetainedWorkerAdmissionCancelDeliveryStateV1::Available
            if completion.result
                == RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedNotDelivered =>
        {
            return Ok((record, false));
        }
        _ => {
            return Err(RetainedWorkerRuntimeError(
                "admission cancellation delivery result does not own the live exact claim".into(),
            ));
        }
    };
    registry.cancel_deliveries_by_request.insert(
        completion.cancel_request_id.clone(),
        RetainedWorkerAdmissionCancelDeliveryRecordV1 {
            delivery_state: next_state,
            ..delivery
        },
    );
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

#[allow(clippy::too_many_arguments)]
fn cancel_pending_admission_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    current_exact: &CanonicalExactCurrentAuthorityV1,
    request: &RetainedWorkerAdmissionCancelRequestV1,
    accepted_at: &TimestampV1,
    delivery_claim_nonce: &[u8; 16],
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
    post_r0_graphs: &BTreeMap<(String, String), ResolvedPostR0RegistryGraphV1>,
) -> Result<(RetainedWorkerAdmissionCancelOutcomeV1, bool), RetainedWorkerRuntimeError> {
    if request.schema_version != 1 || request.cancel_request_id.trim().is_empty() {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation request identity is invalid".into(),
        ));
    }
    let root = preserved_start_root_view(authority_root)?;
    let mut record = registry
        .records_by_session
        .get(&request.orchestration_session_id)
        .and_then(|records| records.get(&request.retained_participant_id))
        .cloned()
        .ok_or_else(|| {
            RetainedWorkerRuntimeError("exact admission cancellation target is absent".into())
        })?;
    let indexed = registry
        .issuer_request_index
        .get(&request.issuer_request_id)
        .ok_or_else(|| {
            RetainedWorkerRuntimeError("admission cancellation issuer is absent".into())
        })?;
    if indexed.orchestration_session_id != request.orchestration_session_id
        || indexed.retained_participant_id != request.retained_participant_id
        || request.authority_store_id != registry.authority_store_id
        || record.authority_store_id != request.authority_store_id
        || record.issuer_request_id != request.issuer_request_id
        || record.orchestration_session_id != request.orchestration_session_id
        || current_exact.authority_store_id != request.authority_store_id
        || current_exact.orchestration_session_id != request.orchestration_session_id
        || current_exact.caller_participant_id != request.caller_participant_id
        || record.retained_participant_id != request.retained_participant_id
        || record.bootstrap_run_id != request.bootstrap_run_id
        || record.backend_id != request.backend_id
        || record.protocol != request.protocol
        || record.world_binding != request.world_binding
        || record.current_policy_ref != request.current_policy_ref
        || record.current_policy_revision != request.current_policy_revision
        || current_exact.authority.current_policy_ref.as_ref() != Some(&request.current_policy_ref)
        || current_exact.current_policy.policy_revision != request.current_policy_revision
    {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation request does not exact-join authority and admission truth"
                .into(),
        ));
    }
    validate_admission_to_current_exact_authority_ancestry(
        root.as_ref(),
        current_exact,
        &record,
        typed_history,
    )?;

    let cancelled_registration_head = match &record.state {
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
            cancel_request_id,
            cancelled_at,
            registration_head: Some(_),
        } => Some((cancel_request_id.clone(), cancelled_at.clone())),
        _ => None,
    };
    if let Some((cancel_request_id, cancelled_at)) = cancelled_registration_head {
        if let Some(registration) = admission_registration_from_hsa(authority_root, &record)? {
            let graph = post_r0_graphs
                .get(&(
                    record.orchestration_session_id.clone(),
                    record.retained_participant_id.clone(),
                ))
                .ok_or_else(|| {
                    RetainedWorkerRuntimeError(
                        "reconciled R0 cancellation has no complete registration graph".into(),
                    )
                })?;
            validate_post_r0_registered_graph_against_expectation(
                root.as_ref(),
                &record,
                current_exact,
                &graph.registered_graph.resolved_target.descriptor,
                &record.current_policy_ref,
                &graph.registered_graph.resolved_target.current_policy,
                &registration,
                &graph.registered_graph.result,
                &graph.registered_graph.resolved_target,
                typed_history,
            )?;
            record = replace_admission_state(
                registry,
                authority_root,
                record,
                RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
                    registration,
                    cancel_request_id,
                    cancelled_at,
                },
            )?;
            let outcome = admission_cancel_outcome_from_record(registry, &record, None)
                .ok_or_else(|| {
                    RetainedWorkerRuntimeError(
                        "reconciled admission cancellation outcome is absent".into(),
                    )
                })?;
            return Ok((outcome, true));
        }
    }

    if matches!(
        record.state,
        RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. }
    ) {
        let delivery_claim_id = claim_pending_admission_cancel_delivery_in_record(
            registry,
            &record,
            accepted_at,
            delivery_claim_nonce,
        )?;
        let changed = delivery_claim_id.is_some();
        if changed {
            record.record_revision = record.record_revision.checked_add(1).ok_or_else(|| {
                RetainedWorkerRuntimeError("admission record revision overflow".into())
            })?;
            registry
                .records_by_session
                .get_mut(&record.orchestration_session_id)
                .ok_or_else(|| {
                    RetainedWorkerRuntimeError("admission session bucket is absent".into())
                })?
                .insert(record.retained_participant_id.clone(), record.clone());
        }
        validate_admission_registry(registry, authority_root)?;
        let outcome =
            admission_cancel_outcome_from_record(registry, &record, delivery_claim_id.as_deref())
                .ok_or_else(|| {
                RetainedWorkerRuntimeError(
                    "pending admission cancellation outcome is absent".into(),
                )
            })?;
        return Ok((outcome, changed));
    }
    if let Some(outcome) = admission_cancel_outcome_from_record(registry, &record, None) {
        validate_admission_registry(registry, authority_root)?;
        return Ok((outcome, false));
    }
    if record.record_revision != request.expected_record_revision
        || record.state != request.expected_state
    {
        return Err(RetainedWorkerRuntimeError(
            "admission cancellation expected revision or state is stale".into(),
        ));
    }

    let next_state = match &record.state {
        RetainedWorkerAdmissionStateV1::SlotReserved { .. } => {
            RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                cancel_request_id: request.cancel_request_id.clone(),
                cancelled_at: accepted_at.clone(),
                registration_head: None,
            }
        }
        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
            authority_revision_expected,
            authority_record_commitment_expected,
            head_acquired_at,
        } => {
            if let Some(registration) = admission_registration_from_hsa(authority_root, &record)? {
                let graph = post_r0_graphs
                    .get(&(
                        record.orchestration_session_id.clone(),
                        record.retained_participant_id.clone(),
                    ))
                    .ok_or_else(|| {
                        RetainedWorkerRuntimeError(
                            "applied R0 cancellation has no complete registration graph".into(),
                        )
                    })?;
                validate_post_r0_registered_graph_against_expectation(
                    root.as_ref(),
                    &record,
                    current_exact,
                    &graph.registered_graph.resolved_target.descriptor,
                    &record.current_policy_ref,
                    &graph.registered_graph.resolved_target.current_policy,
                    &registration,
                    &graph.registered_graph.result,
                    &graph.registered_graph.resolved_target,
                    typed_history,
                )?;
                RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
                    registration,
                    cancel_request_id: request.cancel_request_id.clone(),
                    cancelled_at: accepted_at.clone(),
                }
            } else {
                RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                    cancel_request_id: request.cancel_request_id.clone(),
                    cancelled_at: accepted_at.clone(),
                    registration_head: Some(RetainedWorkerAdmissionRegistrationHeadV1 {
                        authority_revision_expected: *authority_revision_expected,
                        authority_record_commitment_expected: authority_record_commitment_expected
                            .clone(),
                        head_acquired_at: head_acquired_at.clone(),
                    }),
                }
            }
        }
        RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration } => {
            RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
                registration: registration.clone(),
                cancel_request_id: request.cancel_request_id.clone(),
                cancelled_at: accepted_at.clone(),
            }
        }
        RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
            registration,
            transport_claim_id,
            transport_span_id,
            stream_id,
            last_frame_sequence,
            ..
        } => RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            registration: registration.clone(),
            transport_claim_id: Some(transport_claim_id.clone()),
            transport_span_id: transport_span_id.clone(),
            stream_id: stream_id.clone(),
            last_frame_sequence: *last_frame_sequence,
            cancel_request_id: request.cancel_request_id.clone(),
            accepted_at: accepted_at.clone(),
        },
        RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
            registration,
            transport_claim_id,
            transport_span_id,
            stream_id,
            last_frame_sequence,
            ..
        } => RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            registration: registration.clone(),
            transport_claim_id: transport_claim_id.clone(),
            transport_span_id: transport_span_id.clone(),
            stream_id: stream_id.clone(),
            last_frame_sequence: *last_frame_sequence,
            cancel_request_id: request.cancel_request_id.clone(),
            accepted_at: accepted_at.clone(),
        },
        RetainedWorkerAdmissionStateV1::Routable { .. }
        | RetainedWorkerAdmissionStateV1::Terminal { .. }
        | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { .. }
        | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. } => {
            unreachable!("terminal outcomes were classified before mutation")
        }
    };
    record.state = next_state;
    let delivery_claim_id = claim_pending_admission_cancel_delivery_in_record(
        registry,
        &record,
        accepted_at,
        delivery_claim_nonce,
    )?;
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
    let outcome =
        admission_cancel_outcome_from_record(registry, &record, delivery_claim_id.as_deref())
            .ok_or_else(|| {
                RetainedWorkerRuntimeError(
                    "durable admission cancellation outcome is absent".into(),
                )
            })?;
    Ok((outcome, true))
}
fn admission_cancel_outcome_from_record(
    registry: &RetainedWorkerAdmissionRegistryV1,
    record: &RetainedWorkerAdmissionRecordV1,
    newly_claimed_delivery_id: Option<&str>,
) -> Option<RetainedWorkerAdmissionCancelOutcomeV1> {
    match &record.state {
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
            cancel_request_id, ..
        }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
            cancel_request_id, ..
        } => Some(
            RetainedWorkerAdmissionCancelOutcomeV1::CancelledBeforeTransport {
                record: record.clone(),
                cancel_request_id: cancel_request_id.clone(),
            },
        ),
        RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            cancel_request_id,
            transport_span_id,
            ..
        } => {
            let delivery = registry.cancel_deliveries_by_request.get(cancel_request_id);
            let delivery_disposition = match (
                transport_span_id,
                delivery.map(|delivery| &delivery.delivery_state),
            ) {
                (_, None | Some(RetainedWorkerAdmissionCancelDeliveryStateV1::Available)) => {
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Unavailable
                }
                (
                    Some(transport_span_id),
                    Some(RetainedWorkerAdmissionCancelDeliveryStateV1::Claimed {
                        delivery_claim_id,
                        ..
                    }),
                ) if newly_claimed_delivery_id == Some(delivery_claim_id.as_str()) => {
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        delivery_claim_id: delivery_claim_id.clone(),
                        transport_span_id: transport_span_id.clone(),
                    }
                }
                (
                    Some(transport_span_id),
                    Some(RetainedWorkerAdmissionCancelDeliveryStateV1::Claimed {
                        delivery_claim_id,
                        claim_expires_at,
                        ..
                    }),
                ) => RetainedWorkerAdmissionCancelDeliveryDispositionV1::InProgress {
                    delivery_claim_id: delivery_claim_id.clone(),
                    transport_span_id: transport_span_id.clone(),
                    claim_expires_at: claim_expires_at.clone(),
                },
                (
                    Some(transport_span_id),
                    Some(RetainedWorkerAdmissionCancelDeliveryStateV1::Confirmed { .. }),
                ) => RetainedWorkerAdmissionCancelDeliveryDispositionV1::Confirmed {
                    transport_span_id: transport_span_id.clone(),
                },
                (None, _) => return None,
            };
            Some(
                RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                    record: record.clone(),
                    cancel_request_id: cancel_request_id.clone(),
                    transport_span_id: transport_span_id.clone(),
                    delivery_disposition,
                },
            )
        }
        RetainedWorkerAdmissionStateV1::Routable { .. } => {
            Some(RetainedWorkerAdmissionCancelOutcomeV1::AlreadyRoutable {
                record: record.clone(),
            })
        }
        RetainedWorkerAdmissionStateV1::Terminal {
            cancel_request_id, ..
        } => Some(RetainedWorkerAdmissionCancelOutcomeV1::AlreadyTerminal {
            record: record.clone(),
            cancel_request_id: cancel_request_id.clone(),
        }),
        RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. } => Some(
            RetainedWorkerAdmissionCancelOutcomeV1::RejectedBeforeRegistration {
                record: record.clone(),
            },
        ),
        RetainedWorkerAdmissionStateV1::SlotReserved { .. }
        | RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        | RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal { .. } => None,
    }
}
fn verify_admission_record_fingerprint(
    supplied_plan: &RetainedWorkerAdmissionPlanV1,
    record: &RetainedWorkerAdmissionRecordV1,
    secret_key: &[u8; 32],
) -> Result<(), RetainedWorkerRuntimeError> {
    let fingerprint = canonical_spawn_fingerprint(
        &record.canonical_spawn_fingerprint.key_id,
        secret_key,
        supplied_plan,
        &record.retained_participant_id,
        &record.bootstrap_run_id,
    )?;
    if fingerprint != record.canonical_spawn_fingerprint
        || supplied_plan.exact_authority.authority_store_id != record.authority_store_id
        || supplied_plan.exact_authority.authority_revision != record.admission_authority_revision
        || supplied_plan.exact_authority.authority_record_commitment
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
    current_plan: &RetainedWorkerAdmissionPlanV1,
    supplied_plan: &RetainedWorkerAdmissionPlanV1,
    input: &AdmissionRegistrationAdvanceInputV1<'_>,
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(RetainedWorkerAdmissionRecordV1, bool), RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, current_plan, reservation_proof)?;
    let root = preserved_start_root_view(authority_root)?;
    let mut record = registry
        .records_by_session
        .get(&current_plan.spawn_request.orchestration_session_id)
        .and_then(|records| records.get(input.retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot is absent".into()))?;
    validate_reservation_proof_identities(reservation_proof, &record)?;
    validate_supplied_admission_authority(
        root.as_ref(),
        &current_plan.exact_authority,
        &supplied_plan.exact_authority,
        &record,
        typed_history,
    )?;
    verify_admission_record_fingerprint(supplied_plan, &record, input.secret_key)?;
    if let Some(registration) = admission_registration(&record.state) {
        validate_registration_result(
            authority_root,
            &record,
            registration,
            &input.post_r0_graph.result,
        )?;
        validate_post_r0_registered_graph(
            root.as_ref(),
            &record,
            current_plan,
            registration,
            &input.post_r0_graph.result,
            &input.post_r0_graph.resolved_target,
            typed_history,
        )?;
        validate_admission_registry(registry, authority_root)?;
        return Ok((record, false));
    }
    if !matches!(
        record.state,
        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
            | RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                registration_head: Some(_),
                ..
            }
    ) {
        return Err(RetainedWorkerRuntimeError(
            "only the registration head can advance after R0".into(),
        ));
    }
    let registration =
        admission_registration_from_hsa(authority_root, &record)?.ok_or_else(|| {
            RetainedWorkerRuntimeError(
                "R0 registration has not reached durable Applied truth".into(),
            )
        })?;
    validate_registration_result(
        authority_root,
        &record,
        &registration,
        &input.post_r0_graph.result,
    )?;
    validate_post_r0_registered_graph(
        root.as_ref(),
        &record,
        current_plan,
        &registration,
        &input.post_r0_graph.result,
        &input.post_r0_graph.resolved_target,
        typed_history,
    )?;
    record.state = match &record.state {
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
            cancel_request_id,
            cancelled_at,
            ..
        } => RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
            registration,
            cancel_request_id: cancel_request_id.clone(),
            cancelled_at: cancelled_at.clone(),
        },
        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. } => {
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { registration }
        }
        _ => unreachable!("registration head state was checked before R0 reconciliation"),
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
    Ok((record, true))
}

#[allow(clippy::too_many_arguments)]
fn claim_transport_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    current_plan: &RetainedWorkerAdmissionPlanV1,
    supplied_plan: &RetainedWorkerAdmissionPlanV1,
    retained_participant_id: &str,
    secret_key: &[u8; 32],
    claim_input: &AdmissionTransportClaimInputV1<'_>,
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<RetainedWorkerTransportClaimV1, RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, current_plan, reservation_proof)?;
    let root = preserved_start_root_view(authority_root)?;
    let mut record = registry
        .records_by_session
        .get(&current_plan.spawn_request.orchestration_session_id)
        .and_then(|records| records.get(retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot is absent".into()))?;
    validate_reservation_proof_identities(reservation_proof, &record)?;
    validate_supplied_admission_authority(
        root.as_ref(),
        &current_plan.exact_authority,
        &supplied_plan.exact_authority,
        &record,
        typed_history,
    )?;
    verify_admission_record_fingerprint(supplied_plan, &record, secret_key)?;
    if let Some(registration) = admission_registration(&record.state) {
        let Some(post_r0_graph) = claim_input.post_r0_graph else {
            return Err(RetainedWorkerRuntimeError(
                "transport claim has no complete post-R0 graph".into(),
            ));
        };
        validate_post_r0_registered_graph(
            root.as_ref(),
            &record,
            current_plan,
            registration,
            &post_r0_graph.result,
            &post_r0_graph.resolved_target,
            typed_history,
        )?;
    }
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
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { .. }
        | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. } => {
            return Err(RetainedWorkerRuntimeError(
                "admission cancellation prevents transport claim".into(),
            ));
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
        .any(|candidate| match &candidate.state {
            RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                transport_claim_id: existing,
                ..
            } => existing == &transport_claim_id,
            RetainedWorkerAdmissionStateV1::Routable {
                transport_claim_id: Some(existing),
                ..
            }
            | RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                transport_claim_id: Some(existing),
                ..
            }
            | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                transport_claim_id: Some(existing),
                ..
            } => existing == &transport_claim_id,
            _ => false,
        })
    {
        return Err(RetainedWorkerRuntimeError(
            "generated transport claim identity collides".into(),
        ));
    }
    record.state = RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
        registration,
        transport_claim_id,
        transport_span_id: None,
        stream_id: None,
        last_frame_sequence: None,
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

#[allow(clippy::too_many_arguments)]
fn advance_admission_runtime_truth_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    current_plan: &RetainedWorkerAdmissionPlanV1,
    supplied_plan: &RetainedWorkerAdmissionPlanV1,
    retained_participant_id: &str,
    secret_key: &[u8; 32],
    post_r0_graph: &ResolvedPostR0RegisteredGraphV1,
    truth: &AdmissionRuntimeTruthInputV1<'_>,
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(RetainedWorkerAdmissionRecordV1, bool), RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, current_plan, reservation_proof)?;
    let root = preserved_start_root_view(authority_root)?;
    let mut record = registry
        .records_by_session
        .get(&current_plan.spawn_request.orchestration_session_id)
        .and_then(|records| records.get(retained_participant_id))
        .cloned()
        .ok_or_else(|| RetainedWorkerRuntimeError("admission slot is absent".into()))?;
    validate_reservation_proof_identities(reservation_proof, &record)?;
    validate_supplied_admission_authority(
        root.as_ref(),
        &current_plan.exact_authority,
        &supplied_plan.exact_authority,
        &record,
        typed_history,
    )?;
    verify_admission_record_fingerprint(supplied_plan, &record, secret_key)?;
    if matches!(
        record.state,
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. }
            | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { .. }
            | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. }
    ) {
        validate_admission_registry(registry, authority_root)?;
        return Ok((record, false));
    }
    let registration = admission_registration(&record.state)
        .cloned()
        .ok_or_else(|| {
            RetainedWorkerRuntimeError(
                "runtime truth cannot advance before exact R0 admission".into(),
            )
        })?;
    validate_post_r0_registered_graph(
        root.as_ref(),
        &record,
        current_plan,
        &registration,
        &post_r0_graph.result,
        &post_r0_graph.resolved_target,
        typed_history,
    )?;

    let next_state = match truth {
        AdmissionRuntimeTruthInputV1::TransportStarted {
            frame_identity,
            transport_span_id,
        } => {
            frame_identity
                .validate()
                .map_err(RetainedWorkerRuntimeError)?;
            if transport_span_id.trim().is_empty() {
                return Err(RetainedWorkerRuntimeError(
                    "transport start span identity is empty".into(),
                ));
            }
            match &record.state {
                RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                    transport_claim_id,
                    transport_span_id: existing_span_id,
                    stream_id,
                    last_frame_sequence,
                    claimed_at,
                    ..
                } => {
                    if existing_span_id
                        .as_deref()
                        .is_some_and(|existing| existing != *transport_span_id)
                        || stream_id
                            .as_ref()
                            .is_some_and(|existing| existing != &frame_identity.stream_id)
                        || last_frame_sequence
                            .is_some_and(|existing| existing != frame_identity.frame_sequence)
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "transport start conflicts with durable admission transport identity"
                                .into(),
                        ));
                    }
                    if existing_span_id.as_deref() == Some(*transport_span_id)
                        && stream_id.as_ref() == Some(&frame_identity.stream_id)
                        && *last_frame_sequence == Some(frame_identity.frame_sequence)
                    {
                        return Ok((record, false));
                    }
                    RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                        registration,
                        transport_claim_id: transport_claim_id.clone(),
                        transport_span_id: Some((*transport_span_id).to_owned()),
                        stream_id: Some(frame_identity.stream_id.clone()),
                        last_frame_sequence: Some(frame_identity.frame_sequence),
                        claimed_at: claimed_at.clone(),
                    }
                }
                RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                    transport_claim_id,
                    transport_span_id: existing_span_id,
                    stream_id,
                    last_frame_sequence,
                    cancel_request_id,
                    accepted_at,
                    ..
                } => {
                    if existing_span_id
                        .as_deref()
                        .is_some_and(|existing| existing != *transport_span_id)
                        || stream_id
                            .as_ref()
                            .is_some_and(|existing| existing != &frame_identity.stream_id)
                        || last_frame_sequence
                            .is_some_and(|existing| existing != frame_identity.frame_sequence)
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "transport start conflicts with accepted cancellation identity".into(),
                        ));
                    }
                    if existing_span_id.as_deref() == Some(*transport_span_id)
                        && stream_id.as_ref() == Some(&frame_identity.stream_id)
                        && *last_frame_sequence == Some(frame_identity.frame_sequence)
                    {
                        return Ok((record, false));
                    }
                    RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                        registration,
                        transport_claim_id: transport_claim_id.clone(),
                        transport_span_id: Some((*transport_span_id).to_owned()),
                        stream_id: Some(frame_identity.stream_id.clone()),
                        last_frame_sequence: Some(frame_identity.frame_sequence),
                        cancel_request_id: cancel_request_id.clone(),
                        accepted_at: accepted_at.clone(),
                    }
                }
                RetainedWorkerAdmissionStateV1::Routable {
                    transport_span_id: Some(existing_span_id),
                    stream_id,
                    ..
                }
                | RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                    transport_span_id: Some(existing_span_id),
                    stream_id: Some(stream_id),
                    ..
                } if existing_span_id == *transport_span_id
                    && stream_id == &frame_identity.stream_id =>
                {
                    return Ok((record, false));
                }
                RetainedWorkerAdmissionStateV1::Terminal { .. } => {
                    return Ok((record, false));
                }
                _ => {
                    return Err(RetainedWorkerRuntimeError(
                        "transport start conflicts with durable admission state".into(),
                    ));
                }
            }
        }
        AdmissionRuntimeTruthInputV1::Registered {
            frame_identity,
            event,
            registered_at,
        } => {
            frame_identity
                .validate()
                .map_err(RetainedWorkerRuntimeError)?;
            event
                .validate_identity_contract()
                .map_err(RetainedWorkerRuntimeError)?;
            let event_identity = event.event_identity.as_ref().ok_or_else(|| {
                RetainedWorkerRuntimeError(
                    "Registered runtime truth requires exact event identity".into(),
                )
            })?;
            // `AgentEvent.agent_id` identifies the execute-envelope telemetry producer on this
            // route, not the selected retained-worker descriptor. The descriptor and runtime plan
            // were already exact-joined by the admission fingerprint, R0 graph, and launch proof.
            if event.kind != AgentEventKind::Registered
                || event.orchestration_session_id != record.orchestration_session_id
                || event.run_id != record.bootstrap_run_id
                || event.participant_id.as_deref() != Some(record.retained_participant_id.as_str())
                || event.parent_participant_id.is_some()
                || event.resumed_from_participant_id.is_some()
                || event.backend_id.as_deref() != Some(record.backend_id.as_str())
                || event.world_id.as_deref() != Some(record.world_binding.world_id.as_str())
                || event.world_generation != Some(record.world_binding.world_generation)
            {
                return Err(RetainedWorkerRuntimeError(
                    "Registered runtime truth does not exactly identify the admitted worker".into(),
                ));
            }
            match &record.state {
                RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                    transport_claim_id,
                    transport_span_id,
                    stream_id,
                    last_frame_sequence,
                    ..
                } => {
                    if stream_id
                        .as_ref()
                        .is_some_and(|existing| existing != &frame_identity.stream_id)
                        || last_frame_sequence
                            .is_some_and(|existing| existing >= frame_identity.frame_sequence)
                        || transport_span_id.as_ref().is_some_and(|existing| {
                            event
                                .span_id
                                .as_ref()
                                .is_some_and(|event_span| event_span != existing)
                        })
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "Registered runtime truth conflicts with durable transport identity"
                                .into(),
                        ));
                    }
                    RetainedWorkerAdmissionStateV1::Routable {
                        registration,
                        transport_claim_id: Some(transport_claim_id.clone()),
                        transport_span_id: transport_span_id
                            .clone()
                            .or_else(|| event.span_id.clone()),
                        stream_id: frame_identity.stream_id.clone(),
                        registered_frame_sequence: frame_identity.frame_sequence,
                        registered_event_id: event_identity.event_id.clone(),
                        registered_event_sequence: event_identity.event_sequence,
                        registered_at: registered_at.clone(),
                    }
                }
                RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                    transport_claim_id,
                    transport_span_id,
                    stream_id,
                    last_frame_sequence,
                    cancel_request_id,
                    accepted_at,
                    ..
                } => {
                    if stream_id
                        .as_ref()
                        .is_some_and(|existing| existing != &frame_identity.stream_id)
                        || last_frame_sequence
                            .is_some_and(|existing| existing >= frame_identity.frame_sequence)
                        || transport_span_id.as_ref().is_some_and(|existing| {
                            event
                                .span_id
                                .as_ref()
                                .is_some_and(|event_span| event_span != existing)
                        })
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "Registered runtime truth conflicts with accepted cancellation identity"
                                .into(),
                        ));
                    }
                    RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                        registration,
                        transport_claim_id: transport_claim_id.clone(),
                        transport_span_id: transport_span_id
                            .clone()
                            .or_else(|| event.span_id.clone()),
                        stream_id: Some(frame_identity.stream_id.clone()),
                        last_frame_sequence: Some(frame_identity.frame_sequence),
                        cancel_request_id: cancel_request_id.clone(),
                        accepted_at: accepted_at.clone(),
                    }
                }
                RetainedWorkerAdmissionStateV1::Routable {
                    stream_id,
                    registered_frame_sequence,
                    registered_event_id,
                    registered_event_sequence,
                    ..
                } if stream_id == &frame_identity.stream_id
                    && *registered_frame_sequence == frame_identity.frame_sequence
                    && registered_event_id == &event_identity.event_id
                    && *registered_event_sequence == event_identity.event_sequence =>
                {
                    return Ok((record, false));
                }
                RetainedWorkerAdmissionStateV1::Terminal { .. } => {
                    return Ok((record, false));
                }
                _ => {
                    return Err(RetainedWorkerRuntimeError(
                        "Registered runtime truth conflicts with durable admission state".into(),
                    ));
                }
            }
        }
        AdmissionRuntimeTruthInputV1::Terminal {
            frame_identity,
            event_identity,
            terminal_identity,
            transport_span_id,
            exit_code,
            terminal_at,
        } => {
            frame_identity
                .validate()
                .map_err(RetainedWorkerRuntimeError)?;
            event_identity
                .validate()
                .map_err(RetainedWorkerRuntimeError)?;
            terminal_identity
                .validate()
                .map_err(RetainedWorkerRuntimeError)?;
            if !terminal_identity.matches_event(event_identity) {
                return Err(RetainedWorkerRuntimeError(
                    "terminal runtime truth does not name its exact event".into(),
                ));
            }
            if transport_span_id.is_some_and(|identity| identity.trim().is_empty()) {
                return Err(RetainedWorkerRuntimeError(
                    "terminal runtime truth transport span is empty".into(),
                ));
            }
            match &record.state {
                RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. } => {}
                RetainedWorkerAdmissionStateV1::Routable {
                    stream_id,
                    registered_frame_sequence,
                    registered_event_sequence,
                    ..
                } if stream_id == &frame_identity.stream_id
                    && *registered_frame_sequence < frame_identity.frame_sequence
                    && *registered_event_sequence < event_identity.event_sequence => {}
                RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                    stream_id,
                    last_frame_sequence,
                    ..
                } if stream_id
                    .as_ref()
                    .is_none_or(|stream_id| stream_id == &frame_identity.stream_id)
                    && last_frame_sequence
                        .is_none_or(|sequence| sequence < frame_identity.frame_sequence) => {}
                RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                    transport_span_id: expected_transport_span_id,
                    stream_id,
                    last_frame_sequence,
                    ..
                } if transport_span_id.is_some()
                    && expected_transport_span_id
                        .as_deref()
                        .is_none_or(|expected| Some(expected) == *transport_span_id)
                    && stream_id
                        .as_ref()
                        .is_none_or(|stream_id| stream_id == &frame_identity.stream_id)
                    && last_frame_sequence
                        .is_none_or(|sequence| sequence < frame_identity.frame_sequence) => {}
                RetainedWorkerAdmissionStateV1::Terminal {
                    stream_id,
                    terminal_frame_sequence,
                    terminal_event_id,
                    terminal_event_sequence,
                    exit_code: existing_exit_code,
                    ..
                } if stream_id == &frame_identity.stream_id
                    && *terminal_frame_sequence == frame_identity.frame_sequence
                    && terminal_event_id == &event_identity.event_id
                    && *terminal_event_sequence == event_identity.event_sequence
                    && existing_exit_code == exit_code =>
                {
                    return Ok((record, false));
                }
                _ => {
                    return Err(RetainedWorkerRuntimeError(
                        "terminal runtime truth conflicts with durable admission state".into(),
                    ));
                }
            }
            RetainedWorkerAdmissionStateV1::Terminal {
                registration,
                stream_id: frame_identity.stream_id.clone(),
                terminal_frame_sequence: frame_identity.frame_sequence,
                terminal_event_id: event_identity.event_id.clone(),
                terminal_event_sequence: event_identity.event_sequence,
                exit_code: *exit_code,
                terminal_at: terminal_at.clone(),
                cancel_request_id: match &record.state {
                    RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                        cancel_request_id,
                        ..
                    } => Some(cancel_request_id.clone()),
                    _ => None,
                },
            }
        }
        AdmissionRuntimeTruthInputV1::Interrupted {
            last_frame_identity,
            interrupted_at,
        } => {
            if let Some(frame_identity) = last_frame_identity {
                frame_identity
                    .validate()
                    .map_err(RetainedWorkerRuntimeError)?;
            }
            let next_stream_id = last_frame_identity.map(|identity| identity.stream_id.clone());
            let next_frame_sequence = last_frame_identity.map(|identity| identity.frame_sequence);
            let pending_next_state = match &record.state {
                RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                    transport_claim_id,
                    transport_span_id,
                    stream_id,
                    last_frame_sequence,
                    cancel_request_id,
                    accepted_at,
                    ..
                } => {
                    if stream_id
                        .as_ref()
                        .is_some_and(|existing| next_stream_id.as_ref() != Some(existing))
                        || last_frame_sequence.is_some_and(|existing| {
                            next_frame_sequence.is_some_and(|observed| observed < existing)
                        })
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "interrupted runtime truth conflicts with accepted cancellation identity"
                                .into(),
                        ));
                    }
                    let reconciled_stream_id = next_stream_id.clone().or_else(|| stream_id.clone());
                    let reconciled_frame_sequence = next_frame_sequence.or(*last_frame_sequence);
                    if &reconciled_stream_id == stream_id
                        && &reconciled_frame_sequence == last_frame_sequence
                    {
                        return Ok((record, false));
                    }
                    Some(
                        RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                            registration: registration.clone(),
                            transport_claim_id: transport_claim_id.clone(),
                            transport_span_id: transport_span_id.clone(),
                            stream_id: reconciled_stream_id,
                            last_frame_sequence: reconciled_frame_sequence,
                            cancel_request_id: cancel_request_id.clone(),
                            accepted_at: accepted_at.clone(),
                        },
                    )
                }
                _ => None,
            };
            if let Some(pending_next_state) = pending_next_state {
                let record =
                    replace_admission_state(registry, authority_root, record, pending_next_state)?;
                return Ok((record, true));
            }
            match &record.state {
                RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. } => {}
                RetainedWorkerAdmissionStateV1::Routable {
                    stream_id,
                    registered_frame_sequence,
                    ..
                } if next_stream_id
                    .as_ref()
                    .is_none_or(|observed| observed == stream_id)
                    && next_frame_sequence
                        .is_none_or(|observed| observed >= *registered_frame_sequence) => {}
                RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                    stream_id,
                    last_frame_sequence,
                    ..
                } if stream_id == &next_stream_id
                    && last_frame_sequence == &next_frame_sequence =>
                {
                    return Ok((record, false));
                }
                RetainedWorkerAdmissionStateV1::Terminal { .. } => {
                    return Ok((record, false));
                }
                _ => {
                    return Err(RetainedWorkerRuntimeError(
                        "interrupted runtime truth conflicts with durable admission state".into(),
                    ));
                }
            }
            RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                registration,
                transport_claim_id: admission_transport_claim_id(&record.state).cloned(),
                transport_span_id: admission_transport_span_id(&record.state).cloned(),
                stream_id: next_stream_id,
                last_frame_sequence: next_frame_sequence,
                interrupted_at: interrupted_at.clone(),
            }
        }
    };
    record.state = next_state;
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

fn admission_registration_from_hsa(
    authority_root: &VersionedStateRoot,
    record: &RetainedWorkerAdmissionRecordV1,
) -> Result<Option<RetainedWorkerAdmissionRegistrationV1>, RetainedWorkerRuntimeError> {
    let root = preserved_start_root_view(authority_root)?;
    let (authority_revision_expected, authority_record_commitment_expected) = match &record.state {
        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
            authority_revision_expected,
            authority_record_commitment_expected,
            ..
        } => (
            *authority_revision_expected,
            authority_record_commitment_expected,
        ),
        RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
            registration_head: Some(registration_head),
            ..
        } => (
            registration_head.authority_revision_expected,
            &registration_head.authority_record_commitment_expected,
        ),
        _ => {
            return Err(RetainedWorkerRuntimeError(
                "R0 join requires the exact admission registration head".into(),
            ));
        }
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
        || request.authority_revision_before != authority_revision_expected
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
        authority_root,
        record,
        &admission_registration,
    )?;
    Ok(Some(admission_registration))
}

fn replace_admission_state(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    mut record: RetainedWorkerAdmissionRecordV1,
    state: RetainedWorkerAdmissionStateV1,
) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
    record.state = state;
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
    Ok(record)
}

fn admission_transport_claim_id(state: &RetainedWorkerAdmissionStateV1) -> Option<&String> {
    match state {
        RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
            transport_claim_id, ..
        } => Some(transport_claim_id),
        RetainedWorkerAdmissionStateV1::Routable {
            transport_claim_id, ..
        }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
            transport_claim_id, ..
        }
        | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            transport_claim_id,
            ..
        } => transport_claim_id.as_ref(),
        _ => None,
    }
}

fn admission_transport_span_id(state: &RetainedWorkerAdmissionStateV1) -> Option<&String> {
    match state {
        RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
            transport_span_id, ..
        }
        | RetainedWorkerAdmissionStateV1::Routable {
            transport_span_id, ..
        }
        | RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
            transport_span_id, ..
        }
        | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            transport_span_id,
            ..
        } => transport_span_id.as_ref(),
        _ => None,
    }
}

fn canonical_registration_commitment(
    registration: &RetainedWorkerAuthorityRegistrationV1,
) -> Result<AuthorityObjectCommitmentV1, RetainedWorkerRuntimeError> {
    let canonical = encode_canonical(
        registration,
        "encode canonical retained-worker registration proof",
    )?;
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: lower_hex(&Sha256::digest(canonical)),
    })
}

fn project_launch_commitment(
    commitment: &AuthorityObjectCommitmentV1,
) -> RetainedWorkerAuthorityObjectCommitmentV1 {
    match commitment {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => {
            RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: digest_hex.clone(),
            }
        }
        AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id,
            domain,
            digest_hex,
        } => RetainedWorkerAuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id: key_id.clone(),
            domain: domain.clone(),
            digest_hex: digest_hex.clone(),
        },
    }
}

fn validate_registration_result(
    authority_root: &VersionedStateRoot,
    record: &RetainedWorkerAdmissionRecordV1,
    registration: &RetainedWorkerAdmissionRegistrationV1,
    result: &RetainedWorkerRegistrationResultV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    let root = preserved_start_root_view(authority_root)?;
    let durable = root
        .retained_worker_registration_journal
        .get(&registration.registration_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("R0 result has no durable journal".into()))?;
    if result.registration_id != registration.registration_id
        || result.registration_commitment != canonical_registration_commitment(durable)?
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

fn validate_post_r0_registered_graph(
    root: &StateRootV2,
    record: &RetainedWorkerAdmissionRecordV1,
    plan: &RetainedWorkerAdmissionPlanV1,
    admission_registration: &RetainedWorkerAdmissionRegistrationV1,
    result: &RetainedWorkerRegistrationResultV1,
    resolved: &ResolvedRetainedTargetV1,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(), RetainedWorkerRuntimeError> {
    validate_post_r0_registered_graph_against_expectation(
        root,
        record,
        &plan.exact_authority,
        &plan.descriptor_and_runtime_plan.descriptor,
        &plan.policy_and_admission_cap.current_policy_ref,
        &plan.policy_and_admission_cap.current_policy,
        admission_registration,
        result,
        resolved,
        typed_history,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "the complete registered graph is intentionally explicit"
)]
fn validate_post_r0_registered_graph_against_expectation(
    root: &StateRootV2,
    record: &RetainedWorkerAdmissionRecordV1,
    current_exact_authority: &CanonicalExactCurrentAuthorityV1,
    expected_descriptor: &AgentDescriptorV1,
    expected_policy_ref: &AuthorityObjectRefV1,
    expected_policy: &PolicyObjectHashInputV1,
    admission_registration: &RetainedWorkerAdmissionRegistrationV1,
    result: &RetainedWorkerRegistrationResultV1,
    resolved: &ResolvedRetainedTargetV1,
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<(), RetainedWorkerRuntimeError> {
    let SessionNamespaceRecordV1::Authority(current_authority) = root
        .session_namespace_map
        .get(&record.orchestration_session_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("post-R0 graph session is absent".into()))?
    else {
        return Err(RetainedWorkerRuntimeError(
            "post-R0 graph session has no authority".into(),
        ));
    };
    let registration = root
        .retained_worker_registration_journal
        .get(&admission_registration.registration_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("post-R0 graph has no registration".into()))?;
    let issuer_request_id = format!("retained-worker-registration:{}", record.issuer_request_id);
    let request = root
        .retained_worker_registration_request_index
        .get(&issuer_request_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("post-R0 graph has no request".into()))?;
    let resulting_authority = reconstruct_exact_authority_at_revision(
        root,
        current_exact_authority,
        registration.authority_revision_after,
        typed_history,
    )?;
    let registration_commitment = canonical_registration_commitment(registration)?;
    let expected_internal_uaa_session_id = format!("uaa_{}", record.bootstrap_run_id);
    let current_authority_commitment = canonical_authority_commitment(current_authority)?;
    let current_lineage_commitment = canonical_lineage_commitment(current_authority)?;

    if current_exact_authority.authority_store_id != root.authority_store_id
        || current_exact_authority.orchestration_session_id != record.orchestration_session_id
        || current_exact_authority.authority_revision != current_authority.authority_revision
        || current_exact_authority.authority_record_commitment != current_authority_commitment
        || current_exact_authority.authoritative_lineage_commitment != current_lineage_commitment
        || current_exact_authority.authority != **current_authority
        || current_exact_authority.current_policy != *expected_policy
        || resolved.registration != *registration
        || admission_registration.registration_id != registration.registration_id
        || admission_registration.retained_worker_ref != registration.retained_worker_ref
        || result.registration_id != registration.registration_id
        || result.registration_commitment != registration_commitment
        || result.authority_store_id != record.authority_store_id
        || result.orchestration_session_id != registration.orchestration_session_id
        || result.retained_participant_id != registration.retained_participant_id
        || result.retained_worker_ref != registration.retained_worker_ref
        || result.authority_revision_after != registration.authority_revision_after
        || result.authority_record_commitment_after
            != registration.authority_record_commitment_after
        || request.schema_version != 1
        || request.issuer_request_id != registration.issuer_request_id
        || request.registration_id != registration.registration_id
        || request.orchestration_session_id != registration.orchestration_session_id
        || request.authority_revision_before != registration.authority_revision_before
        || request.authority_record_commitment_before
            != registration.authority_record_commitment_before
        || request.retained_participant_id != registration.retained_participant_id
        || request.descriptor_ref_id != registration.descriptor_ref.ref_id
        || request.descriptor_commitment != registration.descriptor_ref.commitment
        || request.resume_handle_ref_id != registration.resume_handle_ref.ref_id
        || request.resume_handle_commitment != registration.resume_handle_ref.commitment
        || request.retained_worker_ref_id != registration.retained_worker_ref.ref_id
        || request.retained_worker_commitment != registration.retained_worker_ref.commitment
        || request.current_policy_ref != registration.current_policy_ref
        || request.world_binding != registration.world_binding
        || !matches!(
            &request.state,
            RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                authority_revision_after,
                authority_record_commitment_after,
            } if *authority_revision_after == registration.authority_revision_after
                && authority_record_commitment_after
                    == &registration.authority_record_commitment_after
        )
        || registration.orchestration_session_id != record.orchestration_session_id
        || registration.retained_participant_id != record.retained_participant_id
        || registration.current_policy_ref != record.current_policy_ref
        || registration.world_binding != record.world_binding
        || resulting_authority.authority_revision != registration.authority_revision_after
        || resulting_authority.authority_record_commitment
            != registration.authority_record_commitment_after
        || resulting_authority.authoritative_lineage_commitment
            != registration.authoritative_lineage_commitment_after
        || resulting_authority
            .authority
            .authoritative_participant_lineage
            .iter()
            .filter(|participant| *participant == &record.retained_participant_id)
            .count()
            != 1
        || resulting_authority
            .authority
            .retained_worker_refs
            .iter()
            .filter(|reference| *reference == &registration.retained_worker_ref)
            .count()
            != 1
        || resulting_authority.authority.world_binding.as_ref() != Some(&record.world_binding)
        || resulting_authority.authority.current_policy_ref.as_ref()
            != Some(&record.current_policy_ref)
        || resulting_authority
            .authority
            .current_policy_revision
            .as_deref()
            != Some(record.current_policy_revision.as_str())
        || resolved.current_authority_revision != current_exact_authority.authority_revision
        || resolved.descriptor != *expected_descriptor
        || resolved.descriptor.execution_scope != AgentExecutionScopeV1::World
        || resolved.descriptor.backend_id != record.backend_id
        || resolved.descriptor.protocol != record.protocol
        || resolved.resume_handle.orchestration_session_id != record.orchestration_session_id
        || resolved.resume_handle.participant_id != record.retained_participant_id
        || resolved.resume_handle.backend_id != record.backend_id
        || resolved.resume_handle.protocol != record.protocol
        || resolved.resume_handle.internal_uaa_session_id != expected_internal_uaa_session_id
        || resolved.retained_worker.orchestration_session_id != record.orchestration_session_id
        || resolved.retained_worker.participant_id != record.retained_participant_id
        || resolved.retained_worker.world_binding != record.world_binding
        || resolved.retained_worker.descriptor_ref != registration.descriptor_ref
        || resolved.retained_worker.resume_handle_ref != registration.resume_handle_ref
        || resolved.retained_worker.policy_ref != record.current_policy_ref
        || resolved.current_policy != *expected_policy
        || resolved.current_policy.policy_revision != record.current_policy_revision
        || expected_policy_ref != &record.current_policy_ref
    {
        return Err(RetainedWorkerRuntimeError(
            "post-R0 registered graph is inexact".into(),
        ));
    }
    Ok(())
}

fn validate_complete_post_r0_registry_graphs(
    registry: &RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    resolved_graphs: &BTreeMap<(String, String), ResolvedPostR0RegistryGraphV1>,
) -> Result<(), RetainedWorkerRuntimeError> {
    let root = preserved_start_root_view(authority_root)?;
    let mut validated_count = 0_usize;
    for record in registry
        .records_by_session
        .values()
        .flat_map(BTreeMap::values)
    {
        let registration = if let Some(registration) = admission_registration(&record.state) {
            Some(registration.clone())
        } else if matches!(
            record.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
                | RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                    registration_head: Some(_),
                    ..
                }
        ) {
            admission_registration_from_hsa(authority_root, record)?
        } else {
            None
        };
        let Some(registration) = registration else {
            continue;
        };
        let key = (
            record.orchestration_session_id.clone(),
            record.retained_participant_id.clone(),
        );
        let resolved = resolved_graphs.get(&key).ok_or_else(|| {
            RetainedWorkerRuntimeError("complete post-R0 registry graph is absent".into())
        })?;
        validate_post_r0_registered_graph_against_expectation(
            root.as_ref(),
            record,
            &resolved.current_exact_authority,
            &resolved.registered_graph.resolved_target.descriptor,
            &record.current_policy_ref,
            &resolved.registered_graph.resolved_target.current_policy,
            &registration,
            &resolved.registered_graph.result,
            &resolved.registered_graph.resolved_target,
            resolved.typed_authority_history.as_ref(),
        )?;
        validated_count = validated_count.checked_add(1).ok_or_else(|| {
            RetainedWorkerRuntimeError("complete post-R0 graph count overflow".into())
        })?;
    }
    if validated_count != resolved_graphs.len() {
        return Err(RetainedWorkerRuntimeError(
            "complete post-R0 registry graph snapshot is inexact".into(),
        ));
    }
    Ok(())
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

#[allow(clippy::too_many_arguments)]
fn reserve_slot_in_registry(
    registry: &mut RetainedWorkerAdmissionRegistryV1,
    authority_root: &VersionedStateRoot,
    current_plan: &RetainedWorkerAdmissionPlanV1,
    supplied_plan: &RetainedWorkerAdmissionPlanV1,
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    secret_key: &[u8; 32],
    reserved_at: TimestampV1,
    participant_entropy: [u8; 16],
    bootstrap_run_entropy: [u8; 16],
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
) -> Result<RetainedWorkerAdmissionSlotV1, RetainedWorkerRuntimeError> {
    validate_admission_plan(authority_root, current_plan, reservation_proof)?;
    let root = preserved_start_root_view(authority_root)?;
    if let Some(locator) = registry
        .issuer_request_index
        .get(&supplied_plan.issuer_request_id)
    {
        let record = registry
            .records_by_session
            .get(&locator.orchestration_session_id)
            .and_then(|records| records.get(&locator.retained_participant_id))
            .ok_or_else(|| {
                RetainedWorkerRuntimeError("admission issuer index is dangling".into())
            })?;
        validate_reservation_proof_identities(reservation_proof, record)?;
        validate_supplied_admission_authority(
            root.as_ref(),
            &current_plan.exact_authority,
            &supplied_plan.exact_authority,
            record,
            typed_history,
        )?;
        let fingerprint = canonical_spawn_fingerprint(
            &registry.commitment_key.key_id,
            secret_key,
            supplied_plan,
            &record.retained_participant_id,
            &record.bootstrap_run_id,
        )?;
        if record.schema_version != 1
            || record.authority_store_id != supplied_plan.exact_authority.authority_store_id
            || record.issuer_request_id != supplied_plan.issuer_request_id
            || record.canonical_spawn_fingerprint != fingerprint
            || record.orchestration_session_id
                != supplied_plan.spawn_request.orchestration_session_id
            || record.admission_authority_revision
                != supplied_plan.exact_authority.authority_revision
            || record.admission_authority_record_commitment
                != supplied_plan.exact_authority.authority_record_commitment
            || record.backend_id
                != supplied_plan
                    .descriptor_and_runtime_plan
                    .descriptor
                    .backend_id
            || record.protocol
                != supplied_plan
                    .descriptor_and_runtime_plan
                    .descriptor
                    .protocol
            || record.world_binding.world_id != supplied_plan.spawn_request.world_id
            || record.world_binding.world_generation != supplied_plan.spawn_request.world_generation
            || record.current_policy_ref
                != supplied_plan.policy_and_admission_cap.current_policy_ref
            || record.current_policy_revision
                != supplied_plan
                    .policy_and_admission_cap
                    .current_policy
                    .policy_revision
            || record.max_live_retained_workers
                != supplied_plan
                    .policy_and_admission_cap
                    .max_live_retained_workers
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

    if supplied_plan != current_plan {
        return Err(RetainedWorkerRuntimeError(
            "new admission slot requires the exact current bound read".into(),
        ));
    }
    let plan = current_plan;
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

    let (retained_participant_id, bootstrap_run_id) = reservation_proof.map_or_else(
        || {
            (
                format!("rwp_{}", lower_hex(&participant_entropy)),
                format!("rwr_{}", lower_hex(&bootstrap_run_entropy)),
            )
        },
        |proof| {
            (
                proof.retained_participant_id().to_owned(),
                proof.bootstrap_run_id().to_owned(),
            )
        },
    );
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
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
) -> Result<(), RetainedWorkerRuntimeError> {
    let request = &plan.spawn_request;
    let exact = &plan.exact_authority;
    let runtime = &plan.descriptor_and_runtime_plan;
    let policy = &plan.policy_and_admission_cap;
    if let Some(proof) = reservation_proof {
        proof
            .validate_admission_plan_binding(plan)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
    }
    let root = preserved_start_root_view(authority_root)?;
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
        || (policy.allow_capability_narrowing
            && !reservation_proof.is_some_and(
                AuthenticatedFreshSpawnReservationProofV1::permits_capability_narrowing,
            ))
    {
        return Err(RetainedWorkerRuntimeError(
            "canonical admission plan is incomplete or conflicts with locked authority".into(),
        ));
    }
    Ok(())
}

fn validate_reservation_proof_identities(
    reservation_proof: Option<&AuthenticatedFreshSpawnReservationProofV1>,
    record: &RetainedWorkerAdmissionRecordV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    if reservation_proof.is_some_and(|proof| {
        proof.retained_participant_id() != record.retained_participant_id
            || proof.bootstrap_run_id() != record.bootstrap_run_id
    }) {
        return Err(RetainedWorkerRuntimeError(
            "fresh-spawn reservation identities conflict with B3.2a admission".into(),
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
        &plan.descriptor_and_runtime_plan,
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
    typed_history: Option<&BTreeMap<u64, ResolvedSessionAuthorityV1>>,
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
    if let Some(history) = typed_history {
        let expected_revisions = usize::try_from(current.authority_revision).map_err(|_| {
            RetainedWorkerRuntimeError("typed authority history length overflows usize".into())
        })?;
        let resolved_current = history.get(&current.authority_revision).ok_or_else(|| {
            RetainedWorkerRuntimeError(
                "typed authority history does not reach the exact current authority".into(),
            )
        })?;
        let current_observation = resolved_current.observation();
        if history.len() != expected_revisions
            || history.keys().copied().ne(1..=current.authority_revision)
            || current_observation.authority_store_id != current.authority_store_id
            || current_observation.orchestration_session_id != current.orchestration_session_id
            || current_observation.authority_revision != current.authority_revision
            || current_observation.authority_record_commitment
                != current.authority_record_commitment
            || current_observation.authoritative_lineage_commitment
                != current.authoritative_lineage_commitment
            || resolved_current.authority != current.authority
        {
            return Err(RetainedWorkerRuntimeError(
                "typed authority history does not authenticate the exact current authority".into(),
            ));
        }
        let resolved = history.get(&target_revision).ok_or_else(|| {
            RetainedWorkerRuntimeError(
                "target authority revision is absent from the exact typed history".into(),
            )
        })?;
        let observation = resolved.observation();
        if observation.authority_store_id != current.authority_store_id
            || observation.orchestration_session_id != current.orchestration_session_id
            || observation.authority_revision != target_revision
            || resolved
                .authority
                .active_authoritative_participant_id
                .as_deref()
                != Some(current.caller_participant_id.as_str())
            || resolved.authority.host_attach_contract_ref
                != current.authority.host_attach_contract_ref
            || resolved.authority.current_policy_ref != current.authority.current_policy_ref
        {
            return Err(RetainedWorkerRuntimeError(
                "typed historical authority cannot be projected as the exact admission caller"
                    .into(),
            ));
        }
        let mut reconstructed = current.clone();
        reconstructed.authority_revision = target_revision;
        reconstructed.authority_record_commitment = observation.authority_record_commitment;
        reconstructed.authoritative_lineage_commitment =
            observation.authoritative_lineage_commitment;
        reconstructed.authority = resolved.authority.clone();
        return Ok(reconstructed);
    }
    let DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } = &current.authority.origin;
    let intent = root.transition_intent_map.get(intent_id).ok_or_else(|| {
        RetainedWorkerRuntimeError("admission authority ancestry has no Start intent".into())
    })?;
    let HostSessionTransitionIntentStateV2::Applied {
        authority_revision_after,
        authority_record_commitment,
        active_authoritative_participant_id,
        resulting_posture,
        applied_at,
        ..
    } = &intent.state
    else {
        return Err(RetainedWorkerRuntimeError(
            "admission authority ancestry Start is not applied".into(),
        ));
    };
    if *authority_revision_after != 1
        || active_authoritative_participant_id != &intent.target_authoritative_participant_id
        || *resulting_posture != HostSessionPostureV1::ActiveAttached
    {
        return Err(RetainedWorkerRuntimeError(
            "admission authority ancestry Start proof is inconsistent".into(),
        ));
    }
    let mut authority = DurableSessionAuthorityV1 {
        schema_version: current.authority.schema_version,
        orchestration_session_id: current.authority.orchestration_session_id.clone(),
        shell_trace_session_id: current.authority.shell_trace_session_id.clone(),
        authority_revision: *authority_revision_after,
        origin: current.authority.origin.clone(),
        authoritative_participant_lineage: intent.resulting_authoritative_lineage.clone(),
        active_authoritative_participant_id: Some(
            intent.target_authoritative_participant_id.clone(),
        ),
        workspace_binding: current.authority.workspace_binding.clone(),
        world_binding: current.authority.world_binding.clone(),
        host_attach_contract_ref: current.authority.host_attach_contract_ref.clone(),
        retained_worker_refs: Vec::new(),
        internal_resume_handle_refs: Vec::new(),
        lifecycle_posture: HostSessionPostureV1::ActiveAttached,
        current_policy_ref: current.authority.current_policy_ref.clone(),
        current_policy_revision: current.authority.current_policy_revision.clone(),
        updated_at: applied_at.clone(),
    };
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
                "legacy V2 admission authority ancestry is not uniquely contiguous R0".into(),
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
            | RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. }
            | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { .. }
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
        | RetainedWorkerAdmissionStateV1::Terminal { registration, .. }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeTransport { registration, .. }
        | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
            registration,
            ..
        } => Some(registration),
        RetainedWorkerAdmissionStateV1::SlotReserved { .. }
        | RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        | RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration { .. }
        | RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration { .. } => None,
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
    let mut cancel_request_ids = BTreeSet::new();
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
                    transport_span_id,
                    stream_id,
                    last_frame_sequence,
                    ..
                } => {
                    if !valid_generated_id(transport_claim_id, "rtc_")
                        || !transport_claim_ids.insert(transport_claim_id.clone())
                        || transport_span_id
                            .as_ref()
                            .is_some_and(|identity| identity.trim().is_empty())
                        || stream_id
                            .as_ref()
                            .is_some_and(|identity| identity.trim().is_empty())
                        || last_frame_sequence == &Some(0)
                        || (stream_id.is_none() != last_frame_sequence.is_none())
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "admission transport claim identity is invalid or duplicated".into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::Routable {
                    transport_claim_id,
                    transport_span_id,
                    stream_id,
                    registered_frame_sequence,
                    registered_event_id,
                    registered_event_sequence,
                    ..
                } => {
                    if transport_claim_id.as_ref().is_some_and(|identity| {
                        !valid_generated_id(identity, "rtc_")
                            || !transport_claim_ids.insert(identity.clone())
                    }) || transport_span_id
                        .as_ref()
                        .is_some_and(|identity| identity.trim().is_empty())
                        || stream_id.trim().is_empty()
                        || *registered_frame_sequence == 0
                        || registered_event_id.trim().is_empty()
                        || *registered_event_sequence == 0
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "admission routable identity is invalid".into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                    transport_claim_id,
                    transport_span_id,
                    stream_id,
                    last_frame_sequence,
                    ..
                } => {
                    if transport_claim_id.as_ref().is_some_and(|identity| {
                        !valid_generated_id(identity, "rtc_")
                            || !transport_claim_ids.insert(identity.clone())
                    }) || transport_span_id
                        .as_ref()
                        .is_some_and(|identity| identity.trim().is_empty())
                        || stream_id
                            .as_ref()
                            .is_some_and(|stream_id| stream_id.trim().is_empty())
                        || last_frame_sequence == &Some(0)
                        || (stream_id.is_none() && last_frame_sequence.is_some())
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "admission interruption identity is invalid".into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::Terminal {
                    stream_id,
                    terminal_frame_sequence,
                    terminal_event_id,
                    terminal_event_sequence,
                    cancel_request_id,
                    ..
                } => {
                    if stream_id.trim().is_empty()
                        || *terminal_frame_sequence == 0
                        || terminal_event_id.trim().is_empty()
                        || *terminal_event_sequence == 0
                        || cancel_request_id.as_ref().is_some_and(|identity| {
                            identity.trim().is_empty()
                                || !cancel_request_ids.insert(identity.clone())
                        })
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "admission terminal identity is invalid".into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                    cancel_request_id,
                    registration_head,
                    ..
                } => {
                    if cancel_request_id.trim().is_empty()
                        || !cancel_request_ids.insert(cancel_request_id.clone())
                        || registration_head
                            .as_ref()
                            .is_some_and(|head| head.authority_revision_expected == 0)
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "pre-registration cancellation identity is invalid or duplicated"
                                .into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
                    cancel_request_id,
                    ..
                } => {
                    if cancel_request_id.trim().is_empty()
                        || !cancel_request_ids.insert(cancel_request_id.clone())
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "pre-transport cancellation identity is invalid or duplicated".into(),
                        ));
                    }
                }
                RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                    transport_claim_id,
                    transport_span_id,
                    stream_id,
                    last_frame_sequence,
                    cancel_request_id,
                    ..
                } => {
                    if cancel_request_id.trim().is_empty()
                        || !cancel_request_ids.insert(cancel_request_id.clone())
                        || transport_claim_id.as_ref().is_some_and(|identity| {
                            !valid_generated_id(identity, "rtc_")
                                || !transport_claim_ids.insert(identity.clone())
                        })
                        || transport_span_id
                            .as_ref()
                            .is_some_and(|identity| identity.trim().is_empty())
                        || stream_id
                            .as_ref()
                            .is_some_and(|identity| identity.trim().is_empty())
                        || last_frame_sequence == &Some(0)
                        || (stream_id.is_none() != last_frame_sequence.is_none())
                    {
                        return Err(RetainedWorkerRuntimeError(
                            "pending-closeout cancellation identity is invalid or duplicated"
                                .into(),
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
    let mut delivery_claim_ids = BTreeSet::new();
    for (cancel_request_id, delivery) in &registry.cancel_deliveries_by_request {
        let record = registry
            .records_by_session
            .get(&delivery.orchestration_session_id)
            .and_then(|records| records.get(&delivery.retained_participant_id))
            .ok_or_else(|| {
                RetainedWorkerRuntimeError(
                    "admission cancellation delivery target is absent".into(),
                )
            })?;
        let exact_state = match &record.state {
            RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                transport_span_id: Some(transport_span_id),
                cancel_request_id,
                accepted_at,
                ..
            } => {
                cancel_request_id == &delivery.cancel_request_id
                    && transport_span_id == &delivery.transport_span_id
                    && accepted_at == &delivery.accepted_at
            }
            RetainedWorkerAdmissionStateV1::Terminal {
                cancel_request_id: Some(cancel_request_id),
                ..
            } => cancel_request_id == &delivery.cancel_request_id,
            _ => false,
        };
        if delivery.schema_version != 1
            || cancel_request_id != &delivery.cancel_request_id
            || delivery.authority_store_id != registry.authority_store_id
            || delivery.authority_store_id != record.authority_store_id
            || delivery.orchestration_session_id != record.orchestration_session_id
            || delivery.retained_participant_id != record.retained_participant_id
            || delivery.cancel_request_id.trim().is_empty()
            || delivery.transport_span_id.trim().is_empty()
            || !exact_state
        {
            return Err(RetainedWorkerRuntimeError(
                "admission cancellation delivery identity is inexact".into(),
            ));
        }
        match &delivery.delivery_state {
            RetainedWorkerAdmissionCancelDeliveryStateV1::Available => {}
            RetainedWorkerAdmissionCancelDeliveryStateV1::Claimed {
                delivery_claim_id,
                claimed_at,
                claim_expires_at,
            } => {
                if !valid_generated_id(delivery_claim_id, "rcdc_")
                    || !delivery_claim_ids.insert(delivery_claim_id.clone())
                    || claimed_at.as_str() < delivery.accepted_at.as_str()
                    || claim_expires_at.as_str() <= claimed_at.as_str()
                {
                    return Err(RetainedWorkerRuntimeError(
                        "admission cancellation delivery claim is invalid or duplicated".into(),
                    ));
                }
            }
            RetainedWorkerAdmissionCancelDeliveryStateV1::Confirmed {
                delivery_claim_id,
                confirmed_at,
            } => {
                if !valid_generated_id(delivery_claim_id, "rcdc_")
                    || !delivery_claim_ids.insert(delivery_claim_id.clone())
                    || confirmed_at.as_str() < delivery.accepted_at.as_str()
                {
                    return Err(RetainedWorkerRuntimeError(
                        "confirmed admission cancellation delivery is invalid or duplicated".into(),
                    ));
                }
            }
        }
    }
    let root = preserved_start_root_view(authority_root)?;
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
        let (authority_revision_expected, authority_record_commitment_expected) = match &record
            .state
        {
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
                authority_revision_expected,
                authority_record_commitment_expected,
                ..
            } => (
                *authority_revision_expected,
                authority_record_commitment_expected,
            ),
            RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                registration_head: Some(registration_head),
                ..
            } => (
                registration_head.authority_revision_expected,
                &registration_head.authority_record_commitment_expected,
            ),
            _ => {
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
            }
        };
        if request.orchestration_session_id != record.orchestration_session_id
            || request.retained_participant_id != record.retained_participant_id
            || request.authority_revision_before != authority_revision_expected
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
    let root = preserved_start_root_view(authority_root)?;
    let registration = root
        .retained_worker_registration_journal
        .get(&admission_registration.registration_id)
        .ok_or_else(|| {
            RetainedWorkerRuntimeError("post-R0 admission registration is absent".into())
        })?;
    let issuer_request_id = format!("retained-worker-registration:{}", record.issuer_request_id);
    let request = root
        .retained_worker_registration_request_index
        .get(&issuer_request_id)
        .ok_or_else(|| RetainedWorkerRuntimeError("post-R0 admission request is absent".into()))?;
    canonical_registration_commitment(registration)?;
    if registration.schema_version != 1
        || registration.registration_id != admission_registration.registration_id
        || registration.orchestration_session_id != record.orchestration_session_id
        || registration.retained_participant_id != record.retained_participant_id
        || registration.retained_worker_ref != admission_registration.retained_worker_ref
        || registration.current_policy_ref != record.current_policy_ref
        || registration.world_binding != record.world_binding
        || request.schema_version != 1
        || request.issuer_request_id != registration.issuer_request_id
        || request.registration_id != registration.registration_id
        || request.orchestration_session_id != registration.orchestration_session_id
        || request.authority_revision_before != registration.authority_revision_before
        || request.authority_record_commitment_before
            != registration.authority_record_commitment_before
        || request.retained_participant_id != registration.retained_participant_id
        || request.descriptor_ref_id != registration.descriptor_ref.ref_id
        || request.descriptor_commitment != registration.descriptor_ref.commitment
        || request.resume_handle_ref_id != registration.resume_handle_ref.ref_id
        || request.resume_handle_commitment != registration.resume_handle_ref.commitment
        || request.retained_worker_ref_id != registration.retained_worker_ref.ref_id
        || request.retained_worker_commitment != registration.retained_worker_ref.commitment
        || request.current_policy_ref != registration.current_policy_ref
        || request.world_binding != registration.world_binding
        || !matches!(
            &request.state,
            RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                authority_revision_after,
                authority_record_commitment_after,
            } if *authority_revision_after == registration.authority_revision_after
                && authority_record_commitment_after
                    == &registration.authority_record_commitment_after
        )
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

    use substrate_common::agent_events::{
        AgentEvent, MessageEventKind, RuntimeEventIdentityV1, RuntimeFrameIdentityV1,
        RuntimeTerminalIdentityV1, RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
    };

    use super::*;
    use crate::execution::agent_runtime::dispatch_policy_commitment::test_authenticated_fresh_spawn_reservation_proof;
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

    fn write_private_test_file(path: &std::path::Path, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
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

    fn start_additional_authority_session(authority: &HostSessionAuthority) {
        let root = authority.read_a12a_root().unwrap();
        let binding = WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home.clone(),
            authority_store_id: root.authority_store_id.clone(),
        };
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: "second-start-intent".into(),
            issuer_request_id: "retained-worker-registration:second-transition-start".into(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: "second-session".into(),
            shell_trace_session_id: "second-trace".into(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: "second-orchestrator".into(),
            target_participant_lease_token: b"second-start-lease".to_vec(),
            run_id: "second-start-run".into(),
            resulting_authoritative_lineage: vec!["second-orchestrator".into()],
            workspace_binding: binding,
            world_binding: Some(WorldBindingV1 {
                world_id: "second-world".into(),
                world_generation: 11,
            }),
            start_contract: StartContractMaterialV1 {
                descriptor: AgentDescriptorV1 {
                    schema_version: 1,
                    agent_id: "codex-second".into(),
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
            .issue_start_at(&request, timestamp("2026-07-14T13:00:00.000000000Z"), 300)
            .unwrap()
        else {
            panic!("second Start issuance must commit")
        };
        let claim_request = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: "second-start-claim".into(),
            claimant_attempt_id: "second-start-attempt".into(),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(
                &claim_request,
                timestamp("2026-07-14T13:01:00.000000000Z"),
                30,
            )
            .unwrap()
        else {
            panic!("second Start claim must commit")
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            panic!("second Start must remain claimed")
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
                .apply_start_at(&application, timestamp("2026-07-14T13:01:10.000000000Z"))
                .unwrap(),
            TransitionApplicationOutcomeV1::Applied(_)
        ));
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
        conflict.registration_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "aa".repeat(32),
        };
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
    fn registration_commitment_hashes_the_complete_canonical_r0_proof() {
        let (_parent, authority, observation) = started_authority();
        let result = RetainedWorkerRuntime
            .register_retained_target(&authority, &plan(observation))
            .unwrap();
        let root = authority.read_a12a_root().unwrap();
        let registration = root
            .retained_worker_registration_journal
            .get(&result.registration_id)
            .unwrap();
        let canonical = canonical_json::to_vec(registration).unwrap();
        let expected = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: lower_hex(&Sha256::digest(&canonical)),
        };
        assert_eq!(result.registration_commitment, expected);
        assert_eq!(
            canonical_registration_commitment(registration).unwrap(),
            expected
        );

        let mut variants = Vec::new();
        let mut changed = registration.clone();
        changed.registration_id = "rr_different".into();
        variants.push(changed);
        let mut changed = registration.clone();
        changed.authority_revision_after += 1;
        variants.push(changed);
        let mut changed = registration.clone();
        changed.authoritative_lineage_commitment_after =
            AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "11".repeat(32),
            };
        variants.push(changed);
        let mut changed = registration.clone();
        changed.descriptor_ref.commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "22".repeat(32),
        };
        variants.push(changed);
        let mut changed = registration.clone();
        changed.resume_handle_ref.commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "33".repeat(32),
        };
        variants.push(changed);
        let mut changed = registration.clone();
        changed.retained_worker_ref.commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "44".repeat(32),
        };
        variants.push(changed);

        for changed in variants {
            assert_ne!(
                canonical_registration_commitment(&changed).unwrap(),
                result.registration_commitment
            );
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
            registration_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "00".repeat(32),
            },
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
    const ADMISSION_RESERVATION_SUBPROCESS_DROP_PROOF: &str =
        "SUBSTRATE_B3_2A_ADMISSION_RESERVATION_SUBPROCESS_DROP_PROOF";
    const ADMISSION_HEAD_SUBPROCESS_HOME: &str = "SUBSTRATE_B3_2A_ADMISSION_HEAD_SUBPROCESS_HOME";
    const ADMISSION_HEAD_SUBPROCESS_ISSUER: &str =
        "SUBSTRATE_B3_2A_ADMISSION_HEAD_SUBPROCESS_ISSUER";
    const ADMISSION_HEAD_SUBPROCESS_PROMPT: &str =
        "SUBSTRATE_B3_2A_ADMISSION_HEAD_SUBPROCESS_PROMPT";
    const ADMISSION_HEAD_SUBPROCESS_EXACT_AUTHORITY: &str =
        "SUBSTRATE_B3_2A_ADMISSION_HEAD_SUBPROCESS_EXACT_AUTHORITY";
    const ADMISSION_HEAD_SUBPROCESS_MODE: &str = "SUBSTRATE_B3_2A_ADMISSION_HEAD_SUBPROCESS_MODE";
    const TRANSPORT_CLAIM_SUBPROCESS_HOME: &str = "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_HOME";
    const TRANSPORT_CLAIM_SUBPROCESS_ISSUER: &str =
        "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_ISSUER";
    const TRANSPORT_CLAIM_SUBPROCESS_PROMPT: &str =
        "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_PROMPT";
    const TRANSPORT_CLAIM_SUBPROCESS_PARTICIPANT: &str =
        "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_PARTICIPANT";
    const TRANSPORT_CLAIM_SUBPROCESS_ENTROPY: &str =
        "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_ENTROPY";
    const TRANSPORT_CLAIM_SUBPROCESS_EXACT_AUTHORITY: &str =
        "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_EXACT_AUTHORITY";
    const TRANSPORT_CLAIM_SUBPROCESS_MODE: &str = "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_MODE";
    const TRANSPORT_CLAIM_SUBPROCESS_RESULT: &str =
        "SUBSTRATE_B3_2A_TRANSPORT_CLAIM_SUBPROCESS_RESULT";
    const NON_AUTHORITATIVE_PROBE_HOME: &str = "SUBSTRATE_B3_2A_NON_AUTHORITATIVE_PROBE_HOME";
    const NON_AUTHORITATIVE_PROBE_SESSION: &str = "SUBSTRATE_B3_2A_NON_AUTHORITATIVE_PROBE_SESSION";
    const NON_AUTHORITATIVE_PROBE_PARTICIPANT: &str =
        "SUBSTRATE_B3_2A_NON_AUTHORITATIVE_PROBE_PARTICIPANT";

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
        let reservation = RetainedWorkerRuntime.reserve_admission_slot(&authority, &plan, None);
        drop(plan);
        if let Some(path) = std::env::var_os(ADMISSION_RESERVATION_SUBPROCESS_DROP_PROOF) {
            fs::write(path, "creating request dropped").unwrap();
        }
        drop(reservation);
    }

    #[test]
    fn admission_head_subprocess_worker() {
        let Some(home) = std::env::var_os(ADMISSION_HEAD_SUBPROCESS_HOME) else {
            return;
        };
        if let Some(trace_path) = std::env::var_os("SHIM_TRACE_LOG") {
            let trace_path = std::path::PathBuf::from(trace_path);
            let trace_root = trace_path.parent().unwrap();
            substrate_trace::set_global_trace_context(
                substrate_trace::TraceContext::explicit_product(trace_root).unwrap(),
            )
            .unwrap();
            substrate_trace::init_trace(Some(trace_path)).unwrap();
        }
        let authority = HostSessionAuthority::open(std::path::Path::new(&home)).unwrap();
        let issuer = std::env::var(ADMISSION_HEAD_SUBPROCESS_ISSUER).unwrap();
        let mut prompt = std::env::var(ADMISSION_HEAD_SUBPROCESS_PROMPT).unwrap();
        let mode = std::env::var(ADMISSION_HEAD_SUBPROCESS_MODE).unwrap();
        if mode == "conflict" {
            prompt.push_str(" changed");
        }
        let mut plan = admission_plan(&authority, &issuer, &prompt, 4);
        plan.exact_authority = decode_canonical(
            std::env::var(ADMISSION_HEAD_SUBPROCESS_EXACT_AUTHORITY)
                .unwrap()
                .as_bytes(),
            "decode re-presented admission authority",
        )
        .unwrap();
        let result = RetainedWorkerRuntime.register_admitted_worker_at(
            &authority,
            &plan,
            Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
        );
        match mode.as_str() {
            "exact" => assert_eq!(
                result.unwrap_err().to_string(),
                "injected crash while admission registration head"
            ),
            "blocked" => assert_eq!(
                result.unwrap_err().to_string(),
                "a lower-sequence admission slot owns the registration head"
            ),
            "conflict" => assert!(result.is_err()),
            _ => panic!("unknown admission-head subprocess mode"),
        }
    }

    #[test]
    fn transport_claim_subprocess_worker() {
        let Some(home) = std::env::var_os(TRANSPORT_CLAIM_SUBPROCESS_HOME) else {
            return;
        };
        let authority = HostSessionAuthority::open(std::path::Path::new(&home)).unwrap();
        let issuer = std::env::var(TRANSPORT_CLAIM_SUBPROCESS_ISSUER).unwrap();
        let prompt = std::env::var(TRANSPORT_CLAIM_SUBPROCESS_PROMPT).unwrap();
        let participant_id = std::env::var(TRANSPORT_CLAIM_SUBPROCESS_PARTICIPANT).unwrap();
        let entropy = std::env::var(TRANSPORT_CLAIM_SUBPROCESS_ENTROPY)
            .unwrap()
            .parse::<u8>()
            .unwrap();
        let mode = std::env::var(TRANSPORT_CLAIM_SUBPROCESS_MODE).unwrap();
        let result_path =
            std::env::var_os(TRANSPORT_CLAIM_SUBPROCESS_RESULT).map(std::path::PathBuf::from);
        let mut plan = admission_plan(&authority, &issuer, &prompt, 4);
        plan.exact_authority = decode_canonical(
            std::env::var(TRANSPORT_CLAIM_SUBPROCESS_EXACT_AUTHORITY)
                .unwrap()
                .as_bytes(),
            "decode re-presented admission authority",
        )
        .unwrap();
        if matches!(mode.as_str(), "routable-join" | "terminal-join") {
            let expected = RetainedWorkerRuntime
                .read_admission_record(&authority, "r0-session", &participant_id)
                .unwrap()
                .unwrap();
            invoke_no_steal_mutation(
                &authority,
                &plan,
                &expected,
                if mode == "routable-join" {
                    NoStealMutationV1::RoutableJoin
                } else {
                    NoStealMutationV1::TerminalJoin
                },
            );
            fs::write(
                result_path.expect("runtime-truth worker requires a result path"),
                "joined",
            )
            .unwrap();
            return;
        }
        let crash_point = match mode.as_str() {
            "claim" | "conflict" => None,
            "crash-after-publication" => {
                Some(AdmissionTransportClaimCrashPointV1::AfterPublicationBeforeResponse)
            }
            _ => panic!("unknown transport claim subprocess mode"),
        };
        let result = RetainedWorkerRuntime.claim_admission_transport_at(
            &authority,
            &plan,
            &participant_id,
            timestamp("2026-07-15T14:03:00.000000000Z"),
            [entropy; 16],
            [entropy.wrapping_add(20); 16],
            crash_point,
        );
        if mode == "crash-after-publication" {
            assert_eq!(
                result.unwrap_err().to_string(),
                "injected crash after transport-claim publication before response"
            );
            std::process::exit(0);
        }
        if mode == "conflict" {
            assert_eq!(
                result.unwrap_err().to_string(),
                "admission record fingerprint or authority ancestry is inexact"
            );
            std::process::exit(23);
        }
        let claim = result.unwrap();
        fs::write(
            result_path.expect("claim worker requires a result path"),
            if claim.newly_claimed { "new" } else { "joined" },
        )
        .unwrap();
    }

    #[test]
    fn non_authoritative_probe_subprocess_worker() {
        let Some(home) = std::env::var_os(NON_AUTHORITATIVE_PROBE_HOME) else {
            return;
        };
        let authority = HostSessionAuthority::open(std::path::Path::new(&home)).unwrap();
        let record = RetainedWorkerRuntime
            .read_admission_record(
                &authority,
                &std::env::var(NON_AUTHORITATIVE_PROBE_SESSION).unwrap(),
                &std::env::var(NON_AUTHORITATIVE_PROBE_PARTICIPANT).unwrap(),
            )
            .unwrap();
        assert!(record.is_some());
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
    fn admission_key_and_registry_publication_boundaries_reopen_without_false_success() {
        for (index, crash_point) in [
            AdmissionInitializationCrashPointV1::BeforeKeyTempPersistence,
            AdmissionInitializationCrashPointV1::AfterKeyTempFsync,
            AdmissionInitializationCrashPointV1::BeforeKeyPublication,
            AdmissionInitializationCrashPointV1::AfterKeyPublication,
            AdmissionInitializationCrashPointV1::BeforeRegistryTempPersistence,
            AdmissionInitializationCrashPointV1::AfterRegistryTempFsync,
            AdmissionInitializationCrashPointV1::BeforeRegistryNoReplacePublication,
            AdmissionInitializationCrashPointV1::AfterRegistryPublication,
        ]
        .into_iter()
        .enumerate()
        {
            let (parent, authority, _) = started_authority();
            let runtime = RetainedWorkerRuntime;
            let created_at = timestamp("2026-07-15T12:10:00.000000000Z");
            let key_entropy = [20_u8.wrapping_add(index as u8); 16];
            let publication_nonce = [40_u8.wrapping_add(index as u8); 16];
            let secret_key = [60_u8.wrapping_add(index as u8); 32];
            assert!(runtime
                .initialize_admission_registry_at(
                    &authority,
                    created_at.clone(),
                    key_entropy,
                    publication_nonce,
                    secret_key,
                    Some(crash_point),
                )
                .is_err());

            let admission_root = parent
                .path()
                .join("home/authority-v1/retained-worker-admission-v1");
            let keys_root = admission_root.join("keys");
            let tmp_root = admission_root.join("tmp");
            let registry_path = admission_root.join("registry-v1.json");
            match crash_point {
                AdmissionInitializationCrashPointV1::BeforeKeyTempPersistence => {
                    assert_eq!(fs::read_dir(&keys_root).unwrap().count(), 0);
                    assert!(!registry_path.exists());
                }
                AdmissionInitializationCrashPointV1::AfterKeyTempFsync
                | AdmissionInitializationCrashPointV1::BeforeKeyPublication => {
                    let names = fs::read_dir(&keys_root)
                        .unwrap()
                        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                        .collect::<Vec<_>>();
                    assert_eq!(names.len(), 1);
                    assert!(names[0].starts_with("admission-key--"));
                    assert!(!registry_path.exists());
                }
                AdmissionInitializationCrashPointV1::AfterKeyPublication
                | AdmissionInitializationCrashPointV1::BeforeRegistryTempPersistence => {
                    assert_eq!(fs::read_dir(&keys_root).unwrap().count(), 1);
                    assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 0);
                    assert!(!registry_path.exists());
                }
                AdmissionInitializationCrashPointV1::AfterRegistryTempFsync
                | AdmissionInitializationCrashPointV1::BeforeRegistryNoReplacePublication => {
                    assert_eq!(fs::read_dir(&keys_root).unwrap().count(), 1);
                    assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 1);
                    assert!(!registry_path.exists());
                }
                AdmissionInitializationCrashPointV1::AfterRegistryPublication => {
                    assert_eq!(fs::read_dir(&keys_root).unwrap().count(), 1);
                    assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 0);
                    assert!(registry_path.is_file());
                }
            }

            let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
            let initialized = runtime
                .initialize_admission_registry_at(
                    &reopened,
                    created_at,
                    key_entropy,
                    publication_nonce,
                    secret_key,
                    None,
                )
                .unwrap();
            assert_eq!(
                initialized.key_id,
                format!("adk_{}", lower_hex(&key_entropy))
            );
            assert_eq!(fs::read_dir(&keys_root).unwrap().count(), 1);
            assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 0);
            assert!(registry_path.is_file());
        }

        for registry_boundary in [false, true] {
            let (parent, authority, _) = started_authority();
            let runtime = RetainedWorkerRuntime;
            let created_at = timestamp("2026-07-15T12:11:00.000000000Z");
            let key_entropy = [91_u8.wrapping_add(u8::from(registry_boundary)); 16];
            let publication_nonce = [101_u8.wrapping_add(u8::from(registry_boundary)); 16];
            let secret_key = [111_u8.wrapping_add(u8::from(registry_boundary)); 32];
            let crash_point = if registry_boundary {
                AdmissionInitializationCrashPointV1::AfterRegistryTempFsync
            } else {
                AdmissionInitializationCrashPointV1::BeforeKeyPublication
            };
            assert!(runtime
                .initialize_admission_registry_at(
                    &authority,
                    created_at.clone(),
                    key_entropy,
                    publication_nonce,
                    secret_key,
                    Some(crash_point),
                )
                .is_err());
            let admission_root = parent
                .path()
                .join("home/authority-v1/retained-worker-admission-v1");
            let keys_root = admission_root.join("keys");
            let tmp_root = admission_root.join("tmp");
            let registry_path = admission_root.join("registry-v1.json");
            if registry_boundary {
                let staged = fs::read_dir(&tmp_root)
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                fs::rename(staged, &registry_path).unwrap();
                write_private_test_file(
                    &tmp_root.join(format!("admission-registry--{}.tmp", "ee".repeat(16))),
                    b"conflicting registry orphan",
                );
            } else {
                let staged = fs::read_dir(&keys_root)
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                let key_id = format!("adk_{}", lower_hex(&key_entropy));
                fs::rename(staged, keys_root.join(format!("{key_id}.key"))).unwrap();
                write_private_test_file(
                    &keys_root.join(format!("admission-key--{}.tmp", "dd".repeat(16))),
                    b"conflicting key orphan",
                );
            }
            let registry_before_retry = fs::read(&registry_path).ok();
            let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
            let initialized = runtime
                .initialize_admission_registry_at(
                    &reopened,
                    created_at,
                    key_entropy,
                    publication_nonce,
                    secret_key,
                    None,
                )
                .unwrap();
            assert_eq!(
                initialized.key_id,
                format!("adk_{}", lower_hex(&key_entropy))
            );
            assert_eq!(fs::read_dir(&keys_root).unwrap().count(), 1);
            assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 0);
            assert!(registry_path.is_file());
            if let Some(registry_before_retry) = registry_before_retry {
                assert_eq!(fs::read(&registry_path).unwrap(), registry_before_retry);
            }
        }
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

    fn regular_file_contents(root: &std::path::Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
        let mut contents = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let file_type = entry.file_type().unwrap();
                if file_type.is_dir() {
                    pending.push(entry.path());
                } else if file_type.is_file() {
                    contents.push((entry.path(), fs::read(entry.path()).unwrap()));
                }
            }
        }
        contents
    }

    #[derive(Clone, Copy)]
    enum NoStealMutationV1 {
        QueuedRegistration,
        RegistrationHead,
        TransportJoin,
        RoutableJoin,
        TerminalJoin,
    }

    fn exact_routable_truth(
        plan: &RetainedWorkerAdmissionPlanV1,
        record: &RetainedWorkerAdmissionRecordV1,
    ) -> (RuntimeFrameIdentityV1, AgentEvent, TimestampV1) {
        let RetainedWorkerAdmissionStateV1::Routable {
            stream_id,
            registered_frame_sequence,
            registered_event_id,
            registered_event_sequence,
            registered_at,
            ..
        } = &record.state
        else {
            panic!("expected exact Routable admission truth")
        };
        let (mut frame, mut event) = registered_runtime_truth(plan, record, stream_id);
        frame.frame_sequence = *registered_frame_sequence;
        event.event_identity = Some(RuntimeEventIdentityV1 {
            event_id: registered_event_id.clone(),
            event_sequence: *registered_event_sequence,
        });
        (frame, event, registered_at.clone())
    }

    fn exact_terminal_truth(
        record: &RetainedWorkerAdmissionRecordV1,
    ) -> (
        RuntimeFrameIdentityV1,
        RuntimeEventIdentityV1,
        RuntimeTerminalIdentityV1,
        i32,
        TimestampV1,
    ) {
        let RetainedWorkerAdmissionStateV1::Terminal {
            stream_id,
            terminal_frame_sequence,
            terminal_event_id,
            terminal_event_sequence,
            exit_code,
            terminal_at,
            ..
        } = &record.state
        else {
            panic!("expected exact Terminal admission truth")
        };
        let frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: stream_id.clone(),
            frame_sequence: *terminal_frame_sequence,
        };
        let event = RuntimeEventIdentityV1 {
            event_id: terminal_event_id.clone(),
            event_sequence: *terminal_event_sequence,
        };
        let terminal = RuntimeTerminalIdentityV1::from(&event);
        (frame, event, terminal, *exit_code, terminal_at.clone())
    }

    fn invoke_no_steal_mutation(
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        expected: &RetainedWorkerAdmissionRecordV1,
        mutation: NoStealMutationV1,
    ) {
        let runtime = RetainedWorkerRuntime;
        match mutation {
            NoStealMutationV1::QueuedRegistration => assert_eq!(
                runtime
                    .register_admitted_worker_at(
                        authority,
                        plan,
                        Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
                    )
                    .unwrap_err()
                    .to_string(),
                "a lower-sequence admission slot owns the registration head"
            ),
            NoStealMutationV1::RegistrationHead => assert_eq!(
                runtime
                    .register_admitted_worker_at(
                        authority,
                        plan,
                        Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
                    )
                    .unwrap_err()
                    .to_string(),
                "injected crash while admission registration head"
            ),
            NoStealMutationV1::TransportJoin => {
                let joined = runtime
                    .claim_admission_transport(
                        authority,
                        plan,
                        &expected.retained_participant_id,
                        None,
                    )
                    .unwrap();
                assert!(!joined.newly_claimed);
                assert_eq!(joined.record, *expected);
            }
            NoStealMutationV1::RoutableJoin => {
                let (frame, event, registered_at) = exact_routable_truth(plan, expected);
                let joined = runtime
                    .mark_admission_routable(
                        authority,
                        plan,
                        &expected.retained_participant_id,
                        None,
                        &frame,
                        &event,
                        registered_at,
                    )
                    .unwrap();
                assert_eq!(joined, *expected);
                let mut conflicting_event = event;
                conflicting_event.event_identity.as_mut().unwrap().event_id += "-changed";
                assert!(runtime
                    .mark_admission_routable(
                        authority,
                        plan,
                        &expected.retained_participant_id,
                        None,
                        &frame,
                        &conflicting_event,
                        exact_routable_truth(plan, expected).2,
                    )
                    .is_err());
            }
            NoStealMutationV1::TerminalJoin => {
                let (frame, event, terminal, exit_code, terminal_at) =
                    exact_terminal_truth(expected);
                let joined = runtime
                    .mark_admission_terminal(
                        authority,
                        plan,
                        &expected.retained_participant_id,
                        None,
                        &frame,
                        &event,
                        &terminal,
                        exit_code,
                        terminal_at,
                    )
                    .unwrap();
                assert_eq!(joined, *expected);
                let mut conflicting_event = event;
                conflicting_event.event_id += "-changed";
                let conflicting_terminal = RuntimeTerminalIdentityV1::from(&conflicting_event);
                assert!(runtime
                    .mark_admission_terminal(
                        authority,
                        plan,
                        &expected.retained_participant_id,
                        None,
                        &frame,
                        &conflicting_event,
                        &conflicting_terminal,
                        exit_code,
                        exact_terminal_truth(expected).4,
                    )
                    .is_err());
            }
        }

        let mut changed = plan.clone();
        changed.spawn_request.payload.prompt.push_str(" changed");
        let conflict = match mutation {
            NoStealMutationV1::QueuedRegistration | NoStealMutationV1::RegistrationHead => runtime
                .register_admitted_worker_at(
                    authority,
                    &changed,
                    Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
                )
                .map(drop),
            NoStealMutationV1::TransportJoin => runtime
                .claim_admission_transport(
                    authority,
                    &changed,
                    &expected.retained_participant_id,
                    None,
                )
                .map(drop),
            NoStealMutationV1::RoutableJoin => {
                let (frame, event, registered_at) = exact_routable_truth(plan, expected);
                runtime
                    .mark_admission_routable(
                        authority,
                        &changed,
                        &expected.retained_participant_id,
                        None,
                        &frame,
                        &event,
                        registered_at,
                    )
                    .map(drop)
            }
            NoStealMutationV1::TerminalJoin => {
                let (frame, event, terminal, exit_code, terminal_at) =
                    exact_terminal_truth(expected);
                runtime
                    .mark_admission_terminal(
                        authority,
                        &changed,
                        &expected.retained_participant_id,
                        None,
                        &frame,
                        &event,
                        &terminal,
                        exit_code,
                        terminal_at,
                    )
                    .map(drop)
            }
        };
        assert!(conflict.is_err());
    }

    fn invoke_no_steal_mutation_from_another_pid(
        parent: &tempfile::TempDir,
        plan: &RetainedWorkerAdmissionPlanV1,
        expected: &RetainedWorkerAdmissionRecordV1,
        mutation: NoStealMutationV1,
    ) {
        let executable = std::env::current_exe().unwrap();
        let exact_authority = String::from_utf8(
            encode_canonical(
                &plan.exact_authority,
                "encode no-steal re-presented admission authority",
            )
            .unwrap(),
        )
        .unwrap();
        match mutation {
            NoStealMutationV1::QueuedRegistration | NoStealMutationV1::RegistrationHead => {
                let test_name = "execution::agent_runtime::retained_worker_runtime::tests::admission_head_subprocess_worker";
                assert!(Command::new(&executable)
                    .arg("--exact")
                    .arg(test_name)
                    .arg("--nocapture")
                    .arg("--test-threads=1")
                    .env(ADMISSION_HEAD_SUBPROCESS_HOME, parent.path().join("home"))
                    .env(ADMISSION_HEAD_SUBPROCESS_ISSUER, &plan.issuer_request_id)
                    .env(
                        ADMISSION_HEAD_SUBPROCESS_PROMPT,
                        &plan.spawn_request.payload.prompt,
                    )
                    .env(ADMISSION_HEAD_SUBPROCESS_EXACT_AUTHORITY, exact_authority)
                    .env(
                        ADMISSION_HEAD_SUBPROCESS_MODE,
                        if matches!(mutation, NoStealMutationV1::QueuedRegistration) {
                            "blocked"
                        } else {
                            "exact"
                        },
                    )
                    .status()
                    .unwrap()
                    .success());
            }
            NoStealMutationV1::TransportJoin
            | NoStealMutationV1::RoutableJoin
            | NoStealMutationV1::TerminalJoin => {
                let test_name = "execution::agent_runtime::retained_worker_runtime::tests::transport_claim_subprocess_worker";
                let result_path = parent.path().join("no-steal-subprocess-result");
                assert!(Command::new(&executable)
                    .arg("--exact")
                    .arg(test_name)
                    .arg("--nocapture")
                    .arg("--test-threads=1")
                    .env(TRANSPORT_CLAIM_SUBPROCESS_HOME, parent.path().join("home"))
                    .env(TRANSPORT_CLAIM_SUBPROCESS_ISSUER, &plan.issuer_request_id)
                    .env(
                        TRANSPORT_CLAIM_SUBPROCESS_PROMPT,
                        &plan.spawn_request.payload.prompt,
                    )
                    .env(TRANSPORT_CLAIM_SUBPROCESS_EXACT_AUTHORITY, exact_authority)
                    .env(
                        TRANSPORT_CLAIM_SUBPROCESS_PARTICIPANT,
                        &expected.retained_participant_id,
                    )
                    .env(TRANSPORT_CLAIM_SUBPROCESS_ENTROPY, "201")
                    .env(
                        TRANSPORT_CLAIM_SUBPROCESS_MODE,
                        match mutation {
                            NoStealMutationV1::TransportJoin => "claim",
                            NoStealMutationV1::RoutableJoin => "routable-join",
                            NoStealMutationV1::TerminalJoin => "terminal-join",
                            NoStealMutationV1::QueuedRegistration
                            | NoStealMutationV1::RegistrationHead => unreachable!(),
                        },
                    )
                    .env(TRANSPORT_CLAIM_SUBPROCESS_RESULT, &result_path)
                    .status()
                    .unwrap()
                    .success());
                assert_eq!(fs::read_to_string(result_path).unwrap(), "joined");
            }
        }
    }

    fn admission_created_by_dropped_exited_caller(
        parent: &tempfile::TempDir,
        authority: &HostSessionAuthority,
        issuer_request_id: &str,
        prompt: &str,
    ) -> (RetainedWorkerAdmissionPlanV1, RetainedWorkerAdmissionSlotV1) {
        let drop_proof = parent
            .path()
            .join(format!("{issuer_request_id}-drop-proof"));
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::admission_reservation_subprocess_worker";
        assert!(Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg(test_name)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(
                ADMISSION_RESERVATION_SUBPROCESS_HOME,
                parent.path().join("home"),
            )
            .env(ADMISSION_RESERVATION_SUBPROCESS_ISSUER, issuer_request_id)
            .env(ADMISSION_RESERVATION_SUBPROCESS_PROMPT, prompt)
            .env(ADMISSION_RESERVATION_SUBPROCESS_CAP, "4")
            .env(ADMISSION_RESERVATION_SUBPROCESS_DROP_PROOF, &drop_proof,)
            .status()
            .unwrap()
            .success());
        assert_eq!(
            fs::read_to_string(&drop_proof).unwrap(),
            "creating request dropped"
        );
        fs::remove_file(drop_proof).unwrap();

        let re_presented = admission_plan(authority, issuer_request_id, prompt, 4);
        let joined = RetainedWorkerRuntime
            .reserve_admission_slot(authority, &re_presented, None)
            .unwrap();
        assert!(joined.joined);
        (re_presented, joined)
    }

    fn assert_non_authoritative_matrix_preserves_record(
        parent: &tempfile::TempDir,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        expected: &RetainedWorkerAdmissionRecordV1,
        mutation: NoStealMutationV1,
    ) {
        let runtime = RetainedWorkerRuntime;
        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let registry_before = fs::read(&registry_path).unwrap();
        let assert_unchanged = |authority: &HostSessionAuthority| {
            assert_eq!(
                runtime
                    .read_admission_record(
                        authority,
                        &expected.orchestration_session_id,
                        &expected.retained_participant_id,
                    )
                    .unwrap(),
                Some(expected.clone())
            );
            assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
        };
        let assert_attempt = |authority: &HostSessionAuthority| {
            invoke_no_steal_mutation(authority, plan, expected, mutation);
            assert_unchanged(authority);
        };

        // The helper that created this exact admission explicitly dropped its creating request
        // and then exited before `plan` was reconstructed and re-presented in this process.
        assert_attempt(authority);
        assert_attempt(authority);
        invoke_no_steal_mutation_from_another_pid(parent, plan, expected, mutation);
        assert_unchanged(authority);
        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        assert_attempt(&reopened);

        let mut helper = Command::new("sh")
            .arg("-c")
            .arg("read line")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        assert_attempt(authority);
        helper.kill().unwrap();
        helper.wait().unwrap();
        assert_attempt(authority);

        let socket_path = parent.path().join("non-authoritative.sock");
        let listener = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
        assert_attempt(authority);
        let endpoint = std::os::unix::net::UnixStream::connect(&socket_path).unwrap();
        assert_attempt(authority);
        drop(endpoint);
        drop(listener);
        fs::remove_file(&socket_path).unwrap();
        assert_attempt(authority);

        let (mut eof_reader, eof_writer) = std::os::unix::net::UnixStream::pair().unwrap();
        drop(eof_writer);
        let mut eof = Vec::new();
        std::io::Read::read_to_end(&mut eof_reader, &mut eof).unwrap();
        assert!(eof.is_empty());
        assert_attempt(authority);

        let (_timeout_sender, timeout_receiver) = std::sync::mpsc::channel::<()>();
        assert!(matches!(
            timeout_receiver.recv_timeout(std::time::Duration::from_millis(1)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        assert_attempt(authority);
        let (observer_sender, observer_receiver) = std::sync::mpsc::channel::<()>();
        drop(observer_receiver);
        assert!(observer_sender.send(()).is_err());
        assert_attempt(authority);
    }

    fn registered_runtime_truth(
        plan: &RetainedWorkerAdmissionPlanV1,
        record: &RetainedWorkerAdmissionRecordV1,
        stream_id: &str,
    ) -> (RuntimeFrameIdentityV1, AgentEvent) {
        let frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: stream_id.into(),
            frame_sequence: 1,
        };
        let mut event = AgentEvent::message(
            plan.descriptor_and_runtime_plan.descriptor.agent_id.clone(),
            record.orchestration_session_id.clone(),
            record.bootstrap_run_id.clone(),
            MessageEventKind::Registered,
            "registered",
        );
        event.participant_id = Some(record.retained_participant_id.clone());
        event.backend_id = Some(record.backend_id.clone());
        event.world_id = Some(record.world_binding.world_id.clone());
        event.world_generation = Some(record.world_binding.world_generation);
        event.event_identity = Some(RuntimeEventIdentityV1 {
            event_id: format!("event-{stream_id}-registered"),
            event_sequence: 1,
        });
        (frame, event)
    }

    #[test]
    #[serial_test::serial]
    fn hsa_retained_continue_translation_uses_exact_r0_truth_without_legacy_projection() {
        struct SubstrateHomeGuard(Option<std::ffi::OsString>);

        impl Drop for SubstrateHomeGuard {
            fn drop(&mut self) {
                match self.0.take() {
                    Some(value) => std::env::set_var("SUBSTRATE_HOME", value),
                    None => std::env::remove_var("SUBSTRATE_HOME"),
                }
            }
        }

        use crate::execution::agent_runtime::dispatch_contract::{
            WorkerContinuePayloadV1, WorldDispatchPayloadV1,
        };
        use crate::execution::agent_runtime::state_store::AgentRuntimeStateStore;
        use crate::execution::agent_runtime::tool_invocation_contract::{
            translate_follow_up_tool_to_internal_dispatch_request_v1, HostToolFollowUpHandleV1,
            HostToolNameV1, HostToolRuntimeDispatchMetadataV1, RetainedWorkerHandleV1,
        };
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "translate-continue", "prompt", 1);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .expect("register retained Continue target");
        let claim = runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .expect("claim retained Continue transport");
        let (frame, event) =
            registered_runtime_truth(&plan, &claim.record, "stream-translate-continue");
        let routable = runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &frame,
                &event,
                timestamp("2026-07-15T21:30:00.000000000Z"),
            )
            .expect("publish exact routable retained Continue truth");

        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        let _home_guard = SubstrateHomeGuard(previous_home);
        std::env::set_var("SUBSTRATE_HOME", parent.path().join("home"));
        let store = AgentRuntimeStateStore::new().expect("bind exact retained Continue store");
        assert!(store
            .load_session("r0-session")
            .expect("inspect absent legacy session projection")
            .is_none());
        assert!(store
            .load_participant(&routable.retained_participant_id)
            .expect("inspect absent legacy participant projection")
            .is_none());

        let metadata = HostToolRuntimeDispatchMetadataV1 {
            request_id: "request-translate-continue".into(),
            idempotency_key: "idempotency-translate-continue".into(),
            orchestration_session_id: "r0-session".into(),
            caller_participant_id: "r0-orchestrator".into(),
        };
        let error = translate_follow_up_tool_to_internal_dispatch_request_v1(
            &store,
            &metadata,
            HostToolNameV1::ContinueWorldWorker,
            HostToolFollowUpHandleV1::RetainedWorker(RetainedWorkerHandleV1 {
                participant_id: routable.retained_participant_id.clone(),
            }),
            WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                prompt: "continue exact retained worker".into(),
                thread_id: Some("thread-translate-continue".into()),
            }),
        )
        .expect_err("legacy HSA retained Continue must fail before dispatch preparation");
        assert!(error.to_string().contains("unsupported_legacy_state"));
        assert!(error.to_string().contains("MissingCanonicalCapBytes"));
        assert!(store
            .load_session("r0-session")
            .expect("reinspect absent legacy session projection")
            .is_none());
        assert!(store
            .load_participant(&routable.retained_participant_id)
            .expect("reinspect absent legacy participant projection")
            .is_none());
    }

    #[test]
    #[serial_test::serial]
    fn hsa_retained_continue_translation_rejects_inexact_or_non_routable_truth() {
        struct SubstrateHomeGuard(Option<std::ffi::OsString>);

        impl Drop for SubstrateHomeGuard {
            fn drop(&mut self) {
                match self.0.take() {
                    Some(value) => std::env::set_var("SUBSTRATE_HOME", value),
                    None => std::env::remove_var("SUBSTRATE_HOME"),
                }
            }
        }

        use crate::execution::agent_runtime::dispatch_contract::{
            WorkerContinuePayloadV1, WorldDispatchPayloadV1,
        };
        use crate::execution::agent_runtime::state_store::AgentRuntimeStateStore;
        use crate::execution::agent_runtime::tool_invocation_contract::{
            translate_follow_up_tool_to_internal_dispatch_request_v1, HostToolFollowUpHandleV1,
            HostToolNameV1, HostToolRuntimeDispatchMetadataV1, RetainedWorkerHandleV1,
        };

        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "translate-rejections", "prompt", 1);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .expect("register retained rejection target");
        let participant_id = admitted.record.retained_participant_id.clone();

        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        let _home_guard = SubstrateHomeGuard(previous_home);
        std::env::set_var("SUBSTRATE_HOME", parent.path().join("home"));
        let store = AgentRuntimeStateStore::new().expect("bind retained rejection store");
        let metadata = HostToolRuntimeDispatchMetadataV1 {
            request_id: "request-translate-rejections".into(),
            idempotency_key: "idempotency-translate-rejections".into(),
            orchestration_session_id: "r0-session".into(),
            caller_participant_id: "r0-orchestrator".into(),
        };
        let translate = |metadata: &HostToolRuntimeDispatchMetadataV1, participant_id: &str| {
            translate_follow_up_tool_to_internal_dispatch_request_v1(
                &store,
                metadata,
                HostToolNameV1::ContinueWorldWorker,
                HostToolFollowUpHandleV1::RetainedWorker(RetainedWorkerHandleV1 {
                    participant_id: participant_id.to_string(),
                }),
                WorldDispatchPayloadV1::WorkerContinue(WorkerContinuePayloadV1 {
                    prompt: "continue exact retained worker".into(),
                    thread_id: Some("thread-translate-rejections".into()),
                }),
            )
        };

        let absent = translate(&metadata, "rwp_00000000000000000000000000000000")
            .expect_err("absent exact target must fail closed");
        assert!(absent.to_string().contains("target_not_in_session"));
        let not_routable = translate(&metadata, &participant_id)
            .expect_err("pre-transport target must fail closed");
        assert!(not_routable.to_string().contains("not durably routable"));
        let mut wrong_caller = metadata.clone();
        wrong_caller.caller_participant_id = "substituted-caller".into();
        let caller_mismatch = translate(&wrong_caller, &participant_id)
            .expect_err("substituted caller must fail closed");
        assert!(caller_mismatch
            .to_string()
            .contains("session/caller/store scope mismatch"));
        let mut cross_session = metadata.clone();
        cross_session.orchestration_session_id = "other-session".into();
        assert!(translate(&cross_session, &participant_id).is_err());

        let claim = runtime
            .claim_admission_transport(&authority, &plan, &participant_id, None)
            .expect("claim rejection target transport");
        let (registered_frame, registered_event) =
            registered_runtime_truth(&plan, &claim.record, "stream-translate-rejections");
        runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &participant_id,
                None,
                &registered_frame,
                &registered_event,
                timestamp("2026-07-15T21:40:00.000000000Z"),
            )
            .expect("publish rejection target routable truth");
        let missing_cap = translate(&metadata, &participant_id)
            .expect_err("exact legacy routable truth must fail without immutable cap");
        assert!(missing_cap.to_string().contains("unsupported_legacy_state"));
        assert!(missing_cap.to_string().contains("MissingCanonicalCapBytes"));

        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let exact_registry_bytes = fs::read(&registry_path).expect("read exact admission registry");
        let exact_registry: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&exact_registry_bytes, "decode exact admission registry")
                .expect("decode exact admission registry");

        let mut stale_registry = exact_registry.clone();
        stale_registry
            .records_by_session
            .get_mut("r0-session")
            .expect("stale session bucket")
            .get_mut(&participant_id)
            .expect("stale participant record")
            .admission_authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "f".repeat(64),
        };
        fs::write(
            &registry_path,
            encode_canonical(&stale_registry, "encode stale admission registry")
                .expect("encode stale admission registry"),
        )
        .expect("write stale admission registry");
        let stale = translate(&metadata, &participant_id)
            .expect_err("stale admission ancestry must fail closed");
        assert!(stale
            .to_string()
            .contains("admission-time authority is not the exact recorded typed ancestor"));
        fs::write(&registry_path, &exact_registry_bytes).expect("restore exact admission registry");

        let mut backend_registry = exact_registry.clone();
        backend_registry
            .records_by_session
            .get_mut("r0-session")
            .expect("backend session bucket")
            .get_mut(&participant_id)
            .expect("backend participant record")
            .backend_id = "cli:substituted-worker".into();
        fs::write(
            &registry_path,
            encode_canonical(
                &backend_registry,
                "encode backend-mismatched admission registry",
            )
            .expect("encode backend-mismatched admission registry"),
        )
        .expect("write backend-mismatched admission registry");
        let backend = translate(&metadata, &participant_id)
            .expect_err("backend-substituted admission must fail closed");
        assert!(
            backend.to_string().contains("backend_mismatch"),
            "unexpected backend-substitution error: {backend:#}"
        );
        fs::write(&registry_path, &exact_registry_bytes).expect("restore exact admission registry");

        let mut world_registry = exact_registry;
        world_registry
            .records_by_session
            .get_mut("r0-session")
            .expect("world session bucket")
            .get_mut(&participant_id)
            .expect("world participant record")
            .world_binding
            .world_generation += 1;
        fs::write(
            &registry_path,
            encode_canonical(
                &world_registry,
                "encode world-mismatched admission registry",
            )
            .expect("encode world-mismatched admission registry"),
        )
        .expect("write world-mismatched admission registry");
        assert!(translate(&metadata, &participant_id).is_err());
        fs::write(&registry_path, &exact_registry_bytes).expect("restore exact admission registry");

        let terminal_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: registered_frame.stream_id,
            frame_sequence: 2,
        };
        let terminal_event = RuntimeEventIdentityV1 {
            event_id: "event-stream-translate-rejections-terminal".into(),
            event_sequence: 2,
        };
        let terminal_identity = RuntimeTerminalIdentityV1::from(&terminal_event);
        runtime
            .mark_admission_terminal(
                &authority,
                &plan,
                &participant_id,
                None,
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                0,
                timestamp("2026-07-15T21:41:00.000000000Z"),
            )
            .expect("publish exact terminal retained truth");
        let terminal = translate(&metadata, &participant_id)
            .expect_err("terminal retained target must fail closed");
        assert!(terminal.to_string().contains("target_already_terminal"));
        assert!(store
            .load_session("r0-session")
            .expect("inspect absent legacy session after rejections")
            .is_none());
        assert!(store
            .load_participant(&participant_id)
            .expect("inspect absent legacy participant after rejections")
            .is_none());
    }

    #[test]
    fn registered_truth_uses_exact_member_identity_not_ambient_telemetry_producer() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "registered-telemetry-producer", "prompt", 2);
        runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let claim = runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        let (frame, mut registered) =
            registered_runtime_truth(&plan, &claim.record, "stream-ambient-producer");
        registered.agent_id = "ambient-shell-telemetry-producer".to_string();

        let mut inexact_lineage = registered.clone();
        inexact_lineage.parent_participant_id = Some("unexpected-parent".to_string());
        assert!(runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &frame,
                &inexact_lineage,
                timestamp("2026-07-15T21:00:00.000000000Z"),
            )
            .is_err());
        assert_eq!(
            runtime
                .read_admission_record(
                    &authority,
                    &claim.record.orchestration_session_id,
                    &claim.record.retained_participant_id,
                )
                .unwrap(),
            Some(claim.record.clone()),
            "inexact Registered lineage must not publish Routable"
        );

        let routable = runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &frame,
                &registered,
                timestamp("2026-07-15T21:00:00.000000000Z"),
            )
            .unwrap();
        assert!(matches!(
            routable.state,
            RetainedWorkerAdmissionStateV1::Routable { .. }
        ));
        let replayed = runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &frame,
                &registered,
                timestamp("2026-07-15T21:00:01.000000000Z"),
            )
            .unwrap();
        assert_eq!(
            replayed, routable,
            "exact Registered replay must preserve its first observation timestamp"
        );

        let mut inexact_member = registered;
        inexact_member.orchestration_session_id.push_str("-changed");
        assert!(runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &frame,
                &inexact_member,
                timestamp("2026-07-15T21:00:00.000000000Z"),
            )
            .is_err());
    }

    #[test]
    fn interrupted_nonterminal_remains_live_until_exact_terminal_truth() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "interrupted-live", "prompt", 1);
        runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let claim = runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        let (registered_frame, registered_event) =
            registered_runtime_truth(&plan, &claim.record, "stream-interrupted-live");
        let routable = runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &registered_frame,
                &registered_event,
                timestamp("2026-07-15T21:09:00.000000000Z"),
            )
            .unwrap();
        assert!(matches!(
            routable.state,
            RetainedWorkerAdmissionStateV1::Routable { .. }
        ));
        let last_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "stream-interrupted-live".to_string(),
            frame_sequence: 4,
        };
        let inexact_terminal_event = RuntimeEventIdentityV1 {
            event_id: "event-inexact-terminal".to_string(),
            event_sequence: 2,
        };
        let mut inexact_terminal_identity =
            RuntimeTerminalIdentityV1::from(&inexact_terminal_event);
        inexact_terminal_identity.terminal_event_sequence += 1;
        assert!(runtime
            .mark_admission_terminal(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &last_frame,
                &inexact_terminal_event,
                &inexact_terminal_identity,
                1,
                timestamp("2026-07-15T21:09:30.000000000Z"),
            )
            .is_err());
        let interrupted = runtime
            .mark_admission_interrupted(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                Some(&last_frame),
                timestamp("2026-07-15T21:10:00.000000000Z"),
            )
            .unwrap();
        assert!(matches!(
            interrupted.state,
            RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                ref stream_id,
                last_frame_sequence: Some(4),
                ..
            } if stream_id.as_deref() == Some("stream-interrupted-live")
        ));
        let joined = runtime
            .mark_admission_interrupted(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                Some(&last_frame),
                timestamp("2026-07-15T21:11:00.000000000Z"),
            )
            .unwrap();
        assert_eq!(
            joined, interrupted,
            "repeated interruption is an exact no-op"
        );

        let blocked = admission_plan(&authority, "interrupted-blocked", "next", 1);
        assert!(runtime
            .reserve_admission_slot(&authority, &blocked, None)
            .is_err());

        let terminal_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: last_frame.stream_id,
            frame_sequence: 5,
        };
        let terminal_event = RuntimeEventIdentityV1 {
            event_id: "event-interrupted-terminal".to_string(),
            event_sequence: 1,
        };
        let terminal_identity = RuntimeTerminalIdentityV1::from(&terminal_event);
        let terminal = runtime
            .mark_admission_terminal(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                143,
                timestamp("2026-07-15T21:12:00.000000000Z"),
            )
            .unwrap();
        assert!(matches!(
            terminal.state,
            RetainedWorkerAdmissionStateV1::Terminal { exit_code: 143, .. }
        ));
        let terminal_replay = runtime
            .mark_admission_terminal(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                143,
                timestamp("2026-07-15T21:13:00.000000000Z"),
            )
            .unwrap();
        assert_eq!(
            terminal_replay, terminal,
            "exact Terminal replay must preserve its first observation timestamp"
        );
        runtime
            .reserve_admission_slot(&authority, &blocked, None)
            .unwrap();
    }

    #[test]
    fn caller_pid_socket_and_liveness_never_acquire_replace_renew_or_steal() {
        let production_source = include_str!("retained_worker_runtime.rs")
            .split("#[cfg(all(test, any(target_os = \"linux\", target_os = \"macos\")))]")
            .next()
            .unwrap()
            .to_ascii_lowercase();
        for forbidden in [
            "pid", "socket", "endpoint", "timeout", "eof", "liveness", "observer", "helper",
        ] {
            assert!(
                !production_source.contains(forbidden),
                "non-authoritative `{forbidden}` input must not exist in retained admission logic"
            );
        }

        let (slot_parent, slot_authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let head_plan = admission_plan(&slot_authority, "no-steal-slot-head", "head", 4);
        runtime
            .reserve_admission_slot(&slot_authority, &head_plan, None)
            .unwrap();
        let (slot_plan, slot) = admission_created_by_dropped_exited_caller(
            &slot_parent,
            &slot_authority,
            "no-steal-slot",
            "queued",
        );
        assert!(matches!(
            slot.record.state,
            RetainedWorkerAdmissionStateV1::SlotReserved { .. }
        ));
        assert_non_authoritative_matrix_preserves_record(
            &slot_parent,
            &slot_authority,
            &slot_plan,
            &slot.record,
            NoStealMutationV1::QueuedRegistration,
        );

        let (head_parent, head_authority, _) = started_authority();
        let (head_plan, _) = admission_created_by_dropped_exited_caller(
            &head_parent,
            &head_authority,
            "no-steal-head",
            "head",
        );
        let failure = runtime
            .register_admitted_worker_at(
                &head_authority,
                &head_plan,
                Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
            )
            .unwrap_err();
        assert_eq!(
            failure.to_string(),
            "injected crash while admission registration head"
        );
        let head = runtime
            .reserve_admission_slot(&head_authority, &head_plan, None)
            .unwrap();
        assert!(matches!(
            head.record.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        ));
        assert_non_authoritative_matrix_preserves_record(
            &head_parent,
            &head_authority,
            &head_plan,
            &head.record,
            NoStealMutationV1::RegistrationHead,
        );

        let (claim_parent, claim_authority, _) = started_authority();
        let (claim_plan, _) = admission_created_by_dropped_exited_caller(
            &claim_parent,
            &claim_authority,
            "no-steal-claim",
            "claim",
        );
        let admitted = runtime
            .register_admitted_worker(&claim_authority, &claim_plan, None)
            .unwrap();
        let claim = runtime
            .claim_admission_transport(
                &claim_authority,
                &claim_plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        assert!(matches!(
            claim.record.state,
            RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
        ));
        assert_non_authoritative_matrix_preserves_record(
            &claim_parent,
            &claim_authority,
            &claim_plan,
            &claim.record,
            NoStealMutationV1::TransportJoin,
        );

        let (routable_parent, routable_authority, _) = started_authority();
        let (routable_plan, _) = admission_created_by_dropped_exited_caller(
            &routable_parent,
            &routable_authority,
            "no-steal-routable",
            "routable",
        );
        let routable_admitted = runtime
            .register_admitted_worker(&routable_authority, &routable_plan, None)
            .unwrap();
        let routable_claim = runtime
            .claim_admission_transport(
                &routable_authority,
                &routable_plan,
                &routable_admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        let (routable_frame, registered_event) = registered_runtime_truth(
            &routable_plan,
            &routable_claim.record,
            "stream-no-steal-routable",
        );
        let routable = runtime
            .mark_admission_routable(
                &routable_authority,
                &routable_plan,
                &routable_claim.record.retained_participant_id,
                None,
                &routable_frame,
                &registered_event,
                timestamp("2026-07-15T20:00:00.000000000Z"),
            )
            .unwrap();
        assert_non_authoritative_matrix_preserves_record(
            &routable_parent,
            &routable_authority,
            &routable_plan,
            &routable,
            NoStealMutationV1::RoutableJoin,
        );

        let (terminal_parent, terminal_authority, _) = started_authority();
        let (terminal_plan, _) = admission_created_by_dropped_exited_caller(
            &terminal_parent,
            &terminal_authority,
            "no-steal-terminal",
            "terminal",
        );
        let terminal_admitted = runtime
            .register_admitted_worker(&terminal_authority, &terminal_plan, None)
            .unwrap();
        let terminal_claim = runtime
            .claim_admission_transport(
                &terminal_authority,
                &terminal_plan,
                &terminal_admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        let (terminal_registered_frame, terminal_registered_event) = registered_runtime_truth(
            &terminal_plan,
            &terminal_claim.record,
            "stream-no-steal-terminal",
        );
        runtime
            .mark_admission_routable(
                &terminal_authority,
                &terminal_plan,
                &terminal_claim.record.retained_participant_id,
                None,
                &terminal_registered_frame,
                &terminal_registered_event,
                timestamp("2026-07-15T20:00:30.000000000Z"),
            )
            .unwrap();
        let terminal_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: terminal_registered_frame.stream_id.clone(),
            frame_sequence: 2,
        };
        let terminal_event = RuntimeEventIdentityV1 {
            event_id: "event-stream-no-steal-terminal-exit".into(),
            event_sequence: 2,
        };
        let terminal_identity = RuntimeTerminalIdentityV1::from(&terminal_event);
        let terminal = runtime
            .mark_admission_terminal(
                &terminal_authority,
                &terminal_plan,
                &terminal_claim.record.retained_participant_id,
                None,
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                0,
                timestamp("2026-07-15T20:01:00.000000000Z"),
            )
            .unwrap();
        assert_non_authoritative_matrix_preserves_record(
            &terminal_parent,
            &terminal_authority,
            &terminal_plan,
            &terminal,
            NoStealMutationV1::TerminalJoin,
        );
    }

    fn forge_admission_bootstrap_identity(
        parent: &tempfile::TempDir,
        authority: &HostSessionAuthority,
        plan: &RetainedWorkerAdmissionPlanV1,
        record: &RetainedWorkerAdmissionRecordV1,
        changed_bootstrap_run_id: &str,
    ) -> (std::path::PathBuf, Vec<u8>) {
        let identity = RetainedWorkerRuntime
            .initialize_admission_registry(authority)
            .unwrap();
        let (admission_root, key_path) = initialized_admission_paths(parent, &identity);
        let registry_path = admission_root.join("registry-v1.json");
        let mut registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
            &fs::read(&registry_path).unwrap(),
            "decode fixture registry",
        )
        .unwrap();
        let envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(key_path).unwrap(), "decode fixture key").unwrap();
        let changed = registry
            .records_by_session
            .get_mut(&record.orchestration_session_id)
            .unwrap()
            .get_mut(&record.retained_participant_id)
            .unwrap();
        changed.bootstrap_run_id = changed_bootstrap_run_id.into();
        changed.canonical_spawn_fingerprint = canonical_spawn_fingerprint(
            &identity.key_id,
            &envelope.secret_key,
            plan,
            &changed.retained_participant_id,
            changed_bootstrap_run_id,
        )
        .unwrap();
        let forged = encode_canonical(&registry, "encode fixture registry").unwrap();
        fs::write(&registry_path, &forged).unwrap();
        (registry_path, forged)
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
        let policy_path = std::path::Path::new(&bootstrap_home).join("policy.yaml");
        fs::write(
            &policy_path,
            format!(
                "agents:\n  allowed_backends:\n    - \"{target_backend_id}\"\n  world_dispatch:\n    enabled: true\n    allowed_backends:\n      - \"{target_backend_id}\"\n    allowed_actions:\n      - \"spawn_world_worker\"\n    allowed_modes:\n      - \"retained\"\n    same_session_only: true\n    same_world_binding_only: true\n    allow_capability_narrowing: false\n    max_live_retained_workers: {max_live_retained_workers}\n    max_concurrent_ephemeral: 4\n"
            ),
        )
        .unwrap();
        fs::set_permissions(&policy_path, fs::Permissions::from_mode(0o600)).unwrap();
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

    fn admission_plan_for_second_session(
        authority: &HostSessionAuthority,
        issuer_request_id: &str,
        prompt: &str,
        max_live_retained_workers: u64,
    ) -> RetainedWorkerAdmissionPlanV1 {
        let mut plan = admission_plan(
            authority,
            issuer_request_id,
            prompt,
            max_live_retained_workers,
        );
        let resolved = authority
            .resolve_current_exact("second-session", None)
            .unwrap();
        let world_binding = resolved.authority.world_binding.clone().unwrap();
        plan.spawn_request.orchestration_session_id = "second-session".into();
        plan.spawn_request.caller_participant_id = "second-orchestrator".into();
        plan.spawn_request.world_id = world_binding.world_id;
        plan.spawn_request.world_generation = world_binding.world_generation;
        plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        plan.policy_and_admission_cap.current_policy_ref = resolved
            .authority
            .current_policy_ref
            .clone()
            .expect("second started authority must bind policy");
        plan.policy_and_admission_cap.current_policy = resolved.current_policy;
        plan
    }

    #[test]
    fn admission_cap_zero_rejects_without_slot_or_authority_mutation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let authority_before = authority.read_a12a_root().unwrap();
        let plan = admission_plan(&authority, "cap-zero", "prompt", 0);

        assert!(runtime
            .reserve_admission_slot(&authority, &plan, None)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), authority_before);
        let registry = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let decoded: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&fs::read(registry).unwrap(), "decode fixture registry").unwrap();
        assert!(decoded.records_by_session.is_empty());
    }

    #[test]
    fn authenticated_e2_reservation_preallocates_admission_identities() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "e2-preallocated", "prompt", 3);
        let proof = test_authenticated_fresh_spawn_reservation_proof(
            &plan,
            "rwp_11111111111111111111111111111111",
            "rwr_22222222222222222222222222222222",
            false,
        );

        let slot = runtime
            .reserve_admission_slot(&authority, &plan, Some(&proof))
            .expect("authenticated E2 reservation must allocate its stored identities");

        assert_eq!(
            slot.record.retained_participant_id,
            "rwp_11111111111111111111111111111111"
        );
        assert_eq!(
            slot.record.bootstrap_run_id,
            "rwr_22222222222222222222222222222222"
        );
    }

    #[test]
    fn authenticated_e2_proof_propagates_through_every_admission_validation_path() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let mut plan = admission_plan(&authority, "e2-proof-propagation", "prompt", 3);
        let bootstrap_home = authority
            .bootstrap_home()
            .identity()
            .unwrap()
            .physical_path
            .clone();
        let policy_path = std::path::Path::new(&bootstrap_home).join("policy.yaml");
        let policy = fs::read_to_string(&policy_path).unwrap();
        fs::write(
            &policy_path,
            policy.replace(
                "allow_capability_narrowing: false",
                "allow_capability_narrowing: true",
            ),
        )
        .unwrap();
        fs::set_permissions(&policy_path, fs::Permissions::from_mode(0o600)).unwrap();
        plan.policy_and_admission_cap.allow_capability_narrowing = true;
        let proof = test_authenticated_fresh_spawn_reservation_proof(
            &plan,
            "rwp_33333333333333333333333333333333",
            "rwr_44444444444444444444444444444444",
            true,
        );

        let admitted = runtime
            .register_admitted_worker(&authority, &plan, Some(&proof))
            .expect("E2 proof must reach reserve, head, advance, and graph validation");
        assert_eq!(
            admitted.record.retained_participant_id,
            proof.retained_participant_id()
        );
        assert_eq!(admitted.record.bootstrap_run_id, proof.bootstrap_run_id());

        let canonical = runtime
            .canonical_plan_for_existing_admission(
                &authority,
                &plan,
                &admitted.record,
                Some(&proof),
            )
            .expect("E2 proof must reach existing-admission validation");
        assert!(canonical == plan);

        let claim = runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                Some(&proof),
            )
            .expect("E2 proof must reach transport validation");
        runtime
            .launch_authority_proof_for_claim(&authority, &plan, &claim, Some(&proof))
            .expect("E2 proof must reach launch graph validation");

        let (registered_frame, registered_event) =
            registered_runtime_truth(&plan, &claim.record, "stream-e2-proof");
        runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                Some(&proof),
                &registered_frame,
                &registered_event,
                timestamp("2026-07-15T22:00:00.000000000Z"),
            )
            .expect("E2 proof must reach routability validation");

        let last_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: registered_frame.stream_id.clone(),
            frame_sequence: 2,
        };
        runtime
            .mark_admission_interrupted(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                Some(&proof),
                Some(&last_frame),
                timestamp("2026-07-15T22:01:00.000000000Z"),
            )
            .expect("E2 proof must reach interrupted validation");

        let terminal_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: registered_frame.stream_id,
            frame_sequence: 3,
        };
        let terminal_event = RuntimeEventIdentityV1 {
            event_id: "event-stream-e2-proof-terminal".into(),
            event_sequence: 2,
        };
        let terminal_identity = RuntimeTerminalIdentityV1::from(&terminal_event);
        runtime
            .mark_admission_terminal(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                Some(&proof),
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                0,
                timestamp("2026-07-15T22:02:00.000000000Z"),
            )
            .expect("E2 proof must reach terminal validation");
    }

    #[test]
    fn capability_narrowing_requires_the_authenticated_e2_attestation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let mut plan = admission_plan(&authority, "e2-narrowing-attestation", "prompt", 3);
        let bootstrap_home = authority
            .bootstrap_home()
            .identity()
            .unwrap()
            .physical_path
            .clone();
        let policy_path = std::path::Path::new(&bootstrap_home).join("policy.yaml");
        let policy = fs::read_to_string(&policy_path).unwrap();
        fs::write(
            &policy_path,
            policy.replace(
                "allow_capability_narrowing: false",
                "allow_capability_narrowing: true",
            ),
        )
        .unwrap();
        fs::set_permissions(&policy_path, fs::Permissions::from_mode(0o600)).unwrap();
        plan.policy_and_admission_cap.allow_capability_narrowing = true;

        assert!(runtime
            .reserve_admission_slot(&authority, &plan, None)
            .is_err());
        let no_attestation = test_authenticated_fresh_spawn_reservation_proof(
            &plan,
            "rwp_55555555555555555555555555555555",
            "rwr_66666666666666666666666666666666",
            false,
        );
        assert!(runtime
            .reserve_admission_slot(&authority, &plan, Some(&no_attestation))
            .is_err());
        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let registry: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&fs::read(&registry_path).unwrap(), "decode E2 registry").unwrap();
        assert!(registry.records_by_session.is_empty());

        let attested = test_authenticated_fresh_spawn_reservation_proof(
            &plan,
            "rwp_55555555555555555555555555555555",
            "rwr_66666666666666666666666666666666",
            true,
        );
        runtime
            .reserve_admission_slot(&authority, &plan, Some(&attested))
            .expect("authenticated E2 narrowing attestation must admit the same exact plan");
    }

    #[test]
    fn authenticated_e2_retry_rejects_changed_identity_and_non_policy_material_without_mutation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "e2-proof-conflict", "prompt", 3);
        let proof = test_authenticated_fresh_spawn_reservation_proof(
            &plan,
            "rwp_77777777777777777777777777777777",
            "rwr_88888888888888888888888888888888",
            false,
        );
        runtime
            .reserve_admission_slot(&authority, &plan, Some(&proof))
            .expect("publish the exact E2-bound slot");
        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let exact_registry = fs::read(&registry_path).unwrap();

        let changed_identity = test_authenticated_fresh_spawn_reservation_proof(
            &plan,
            "rwp_99999999999999999999999999999999",
            "rwr_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            false,
        );
        let identity_error = runtime
            .reserve_admission_slot(&authority, &plan, Some(&changed_identity))
            .unwrap_err();
        assert!(identity_error
            .to_string()
            .contains("reservation identities conflict"));
        assert_eq!(fs::read(&registry_path).unwrap(), exact_registry);

        let mut changed_protocol = plan.clone();
        changed_protocol
            .descriptor_and_runtime_plan
            .descriptor
            .protocol
            .push_str("-changed");
        let non_policy_error = runtime
            .reserve_admission_slot(&authority, &changed_protocol, Some(&proof))
            .unwrap_err();
        assert!(!non_policy_error
            .to_string()
            .contains("reservation proof does not match"));
        assert_eq!(fs::read(registry_path).unwrap(), exact_registry);
    }

    #[test]
    #[serial_test::serial]
    fn retained_target_returns_typed_missing_legacy_cap_after_all_non_policy_checks() {
        struct SubstrateHomeGuard(Option<std::ffi::OsString>);

        impl Drop for SubstrateHomeGuard {
            fn drop(&mut self) {
                match self.0.take() {
                    Some(value) => std::env::set_var("SUBSTRATE_HOME", value),
                    None => std::env::remove_var("SUBSTRATE_HOME"),
                }
            }
        }

        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "e2-missing-legacy-cap", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let claim = runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        let (frame, event) =
            registered_runtime_truth(&plan, &claim.record, "stream-e2-missing-cap");
        let routable = runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &frame,
                &event,
                timestamp("2026-07-15T22:10:00.000000000Z"),
            )
            .unwrap();
        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        let _home_guard = SubstrateHomeGuard(previous_home);
        std::env::set_var("SUBSTRATE_HOME", parent.path().join("home"));
        let store =
            crate::execution::agent_runtime::state_store::AgentRuntimeStateStore::new().unwrap();

        let backend_error = store
            .resolve_world_work_registry_authority(
                &routable.orchestration_session_id,
                &plan.spawn_request.caller_participant_id,
                &routable.world_binding.world_id,
                routable.world_binding.world_generation,
                Some((&routable.retained_participant_id, "cli:substituted-worker")),
            )
            .unwrap_err();
        assert!(backend_error.to_string().contains("backend_mismatch"));

        let resolved = store
            .resolve_world_work_registry_authority(
                &routable.orchestration_session_id,
                &plan.spawn_request.caller_participant_id,
                &routable.world_binding.world_id,
                routable.world_binding.world_generation,
                Some((&routable.retained_participant_id, &routable.backend_id)),
            )
            .expect("read-only retained target must expose mixed-version policy compatibility");
        let retained_target = resolved.retained_target.as_ref().unwrap();
        assert_eq!(
            retained_target.participant_id,
            routable.retained_participant_id
        );
        assert_eq!(retained_target.backend_id, routable.backend_id);
        assert!(matches!(
            &retained_target.policy_cap_compatibility,
            crate::execution::agent_runtime::dispatch_policy_commitment::ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                retained_participant_id,
                reason: crate::execution::agent_runtime::dispatch_policy_commitment::PolicyCommitmentCompatibilityReasonV1::MissingCanonicalCapBytes,
            } if retained_participant_id.as_str() == routable.retained_participant_id.as_str()
        ));
        let typed_error = store
            .resolve_hsa_retained_continue_translation_authority(
                &routable.orchestration_session_id,
                &plan.spawn_request.caller_participant_id,
                &routable.retained_participant_id,
            )
            .expect_err("submission-facing legacy resolution must fail before B1 binding");
        assert!(typed_error.to_string().contains("unsupported_legacy_state"));
        assert!(typed_error.to_string().contains("MissingCanonicalCapBytes"));
    }

    #[test]
    #[serial_test::serial]
    fn retained_target_returns_compatible_cap_then_typed_hash_invalid_legacy_cap() {
        struct SubstrateHomeGuard(Option<std::ffi::OsString>);

        impl Drop for SubstrateHomeGuard {
            fn drop(&mut self) {
                match self.0.take() {
                    Some(value) => std::env::set_var("SUBSTRATE_HOME", value),
                    None => std::env::remove_var("SUBSTRATE_HOME"),
                }
            }
        }

        use crate::execution::agent_runtime::dispatch_policy_commitment::{
            publish_fresh_spawn_commitment, reserve_fresh_spawn,
            test_corrupt_retained_worker_cap_hash, validate_policy_snapshot_material,
            AppliedDispatchPolicyPatchIdentityV1, FreshSpawnReservationInputV1,
            PolicyCommitmentCompatibilityReasonV1, ResolvedPolicyCommitmentCompatibilityV1,
        };
        use transport_api_types::{
            PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
            PolicySnapshotWorldFsWriteV3,
        };

        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let mut plan = admission_plan(&authority, "request-e2-compatible-cap", "prompt", 3);
        plan.issuer_request_id = plan.spawn_request.request_id.clone();
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3::default(),
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: None,
                write: PolicySnapshotWorldFsWriteV3::default(),
            },
        }
        .canonicalize()
        .unwrap();
        let snapshot_bytes = serde_json::to_vec(&snapshot).unwrap();
        let snapshot_hash = format!("{:x}", Sha256::digest(&snapshot_bytes));
        let snapshot_material = validate_policy_snapshot_material(
            &snapshot,
            &snapshot_bytes,
            &plan.policy_and_admission_cap.current_policy_ref,
            &snapshot_hash,
            &plan.policy_and_admission_cap.current_policy.policy_revision,
        )
        .unwrap();
        let reservation = reserve_fresh_spawn(
            &authority,
            FreshSpawnReservationInputV1 {
                spawn_request: plan.spawn_request.clone(),
                caller_backend_id: plan.exact_authority.caller_descriptor.backend_id.clone(),
                parent_policy_ref: plan.policy_and_admission_cap.current_policy_ref.clone(),
                parent_policy_revision: plan
                    .policy_and_admission_cap
                    .current_policy
                    .policy_revision
                    .clone(),
                applied_patch: AppliedDispatchPolicyPatchIdentityV1::UnchangedParent,
                policy_snapshot: snapshot_material,
                reason: None,
            },
        )
        .unwrap();
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, Some(&reservation.proof))
            .unwrap();
        publish_fresh_spawn_commitment(&authority, &reservation.proof, &admitted.record).unwrap();
        let claim = runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                Some(&reservation.proof),
            )
            .unwrap();
        let (frame, event) =
            registered_runtime_truth(&plan, &claim.record, "stream-e2-compatible-cap");
        let routable = runtime
            .mark_admission_routable(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                Some(&reservation.proof),
                &frame,
                &event,
                timestamp("2026-07-15T22:20:00.000000000Z"),
            )
            .unwrap();

        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        let _home_guard = SubstrateHomeGuard(previous_home);
        std::env::set_var("SUBSTRATE_HOME", parent.path().join("home"));
        let store =
            crate::execution::agent_runtime::state_store::AgentRuntimeStateStore::new().unwrap();
        let resolve = || {
            store
                .resolve_world_work_registry_authority(
                    &routable.orchestration_session_id,
                    &plan.spawn_request.caller_participant_id,
                    &routable.world_binding.world_id,
                    routable.world_binding.world_generation,
                    Some((&routable.retained_participant_id, &routable.backend_id)),
                )
                .unwrap()
        };
        let compatible = resolve();
        let compatible_target = compatible.retained_target.as_ref().unwrap();
        assert!(matches!(
            &compatible_target.policy_cap_compatibility,
            ResolvedPolicyCommitmentCompatibilityV1::Compatible { cap }
                if cap.retained_participant_id() == routable.retained_participant_id
                    && cap.launch_parent_policy_ref()
                        == &plan.policy_and_admission_cap.current_policy_ref
        ));
        test_corrupt_retained_worker_cap_hash(&authority, &routable.retained_participant_id)
            .unwrap();

        let incompatible = resolve();
        let incompatible_target = incompatible.retained_target.as_ref().unwrap();
        assert!(matches!(
            &incompatible_target.policy_cap_compatibility,
            ResolvedPolicyCommitmentCompatibilityV1::UnsupportedLegacyState {
                retained_participant_id,
                reason: PolicyCommitmentCompatibilityReasonV1::CapHashMismatch,
            } if retained_participant_id.as_str() == routable.retained_participant_id.as_str()
        ));
        let typed_error = store
            .resolve_hsa_retained_continue_translation_authority(
                &routable.orchestration_session_id,
                &plan.spawn_request.caller_participant_id,
                &routable.retained_participant_id,
            )
            .expect_err("submission-facing hash-invalid cap must fail closed");
        assert!(typed_error.to_string().contains("unsupported_legacy_state"));
        assert!(typed_error.to_string().contains("CapHashMismatch"));
    }

    #[test]
    fn admission_cap_boundary_counts_slot_reserved_and_rejects_second_request() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "cap-one-first", "first", 1);
        let first = runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let root_after_first = authority.read_a12a_root().unwrap();

        let second_plan = admission_plan(&authority, "cap-one-second", "second", 1);
        assert!(runtime
            .reserve_admission_slot(&authority, &second_plan, None)
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
    fn slot_reserved_resolution_rejects_before_registration_and_frees_capacity_idempotently() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "slot-resolution", "prompt", 1);
        let reserved = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        let rejection = AdmissionRejectedBeforeRegistrationInputV1 {
            reason: "cancelled before registration".to_string(),
            rejected_at: timestamp("2026-07-16T10:00:00.000000000Z"),
        };

        let rejected = runtime
            .reconcile_existing_admission(
                &authority,
                &plan,
                &reserved.record,
                None,
                Some(&rejection),
            )
            .unwrap();
        assert!(matches!(
            rejected.state,
            RetainedWorkerAdmissionStateV1::RejectedBeforeRegistration {
                ref reason,
                ref rejected_at,
            } if reason == "cancelled before registration"
                && rejected_at == &timestamp("2026-07-16T10:00:00.000000000Z")
        ));

        let rejoined = runtime
            .reconcile_existing_admission(&authority, &plan, &rejected, None, Some(&rejection))
            .unwrap();
        assert_eq!(rejoined, rejected);

        let next_plan = admission_plan(&authority, "slot-resolution-next", "prompt", 1);
        let next = runtime
            .reserve_admission_slot(&authority, &next_plan, None)
            .unwrap();
        assert!(!next.joined);
        assert!(matches!(
            next.record.state,
            RetainedWorkerAdmissionStateV1::SlotReserved {
                slot_sequence: 2,
                ..
            }
        ));

        let retry = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        assert!(retry.joined);
        assert_eq!(retry.record, rejected);
    }

    #[test]
    fn admission_exact_retry_joins_stable_identity_and_changed_prompt_conflicts() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "exact-retry", "original prompt", 1);
        let first = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        let joined = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        assert!(!first.joined);
        assert!(joined.joined);
        assert_eq!(joined.record, first.record);

        let changed = admission_plan(&authority, "exact-retry", "changed prompt", 1);
        assert!(runtime
            .reserve_admission_slot(&authority, &changed, None)
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
    fn existing_admission_plan_reconstructs_admission_time_authority_without_mutation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let original = admission_plan(
            &authority,
            "historical-plan-retry",
            "original retry prompt",
            4,
        );
        let admitted = runtime
            .register_admitted_worker(&authority, &original, None)
            .unwrap();
        let presented = admission_plan(
            &authority,
            "historical-plan-retry",
            "original retry prompt",
            4,
        );
        assert!(
            presented.exact_authority.authority_revision
                > original.exact_authority.authority_revision
        );
        let joined = runtime
            .reserve_admission_slot(&authority, &presented, None)
            .unwrap();
        assert!(joined.joined);
        assert_eq!(joined.record, admitted.record);

        let snapshot_authority_files = || {
            let authority_dir = parent.path().join("home/authority-v1");
            let mut pending = vec![authority_dir.clone()];
            let mut snapshot = Vec::<(std::path::PathBuf, Vec<u8>)>::new();
            while let Some(directory) = pending.pop() {
                for entry in fs::read_dir(&directory).unwrap() {
                    let entry = entry.unwrap();
                    let file_type = entry.file_type().unwrap();
                    if file_type.is_dir() {
                        pending.push(entry.path());
                    } else if file_type.is_file() {
                        snapshot.push((
                            entry
                                .path()
                                .strip_prefix(&authority_dir)
                                .unwrap()
                                .to_path_buf(),
                            fs::read(entry.path()).unwrap(),
                        ));
                    }
                }
            }
            snapshot.sort_by(|left, right| left.0.cmp(&right.0));
            snapshot
        };
        let bytes_before = snapshot_authority_files();
        let reconstructed = runtime
            .canonical_plan_for_existing_admission(&authority, &presented, &joined.record, None)
            .unwrap();
        assert!(reconstructed == original);
        assert_eq!(snapshot_authority_files(), bytes_before);

        let mut variants = Vec::new();
        let mut changed = presented.clone();
        changed.issuer_request_id = "wrong-issuer".into();
        variants.push((changed, joined.record.clone()));
        let mut changed = presented.clone();
        changed.spawn_request.request_id = "wrong-request".into();
        variants.push((changed, joined.record.clone()));
        let mut changed = presented.clone();
        changed.descriptor_and_runtime_plan.descriptor.agent_id = "wrong-agent".into();
        variants.push((changed, joined.record.clone()));
        let mut changed = presented.clone();
        changed.policy_and_admission_cap.max_concurrent_ephemeral += 1;
        variants.push((changed, joined.record.clone()));
        let mut substituted_transient_authority = presented.clone();
        substituted_transient_authority
            .exact_authority
            .authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "11".repeat(32),
        };
        let reconstructed_from_substituted = runtime
            .canonical_plan_for_existing_admission(
                &authority,
                &substituted_transient_authority,
                &joined.record,
                None,
            )
            .unwrap();
        assert!(reconstructed_from_substituted == original);
        assert_eq!(snapshot_authority_files(), bytes_before);
        let mut changed_record = joined.record.clone();
        changed_record.retained_participant_id = "rwp_wrong-participant".into();
        variants.push((presented.clone(), changed_record));
        let mut changed_record = joined.record.clone();
        changed_record.bootstrap_run_id = "run_wrong-bootstrap".into();
        variants.push((presented.clone(), changed_record));
        let mut changed_record = joined.record.clone();
        changed_record.canonical_spawn_fingerprint.digest_hex = "00".repeat(32);
        variants.push((presented.clone(), changed_record));
        let mut changed_record = joined.record.clone();
        changed_record.admission_authority_revision = 0;
        variants.push((presented.clone(), changed_record));

        for (changed_plan, changed_record) in variants {
            let error = runtime
                .canonical_plan_for_existing_admission(
                    &authority,
                    &changed_plan,
                    &changed_record,
                    None,
                )
                .err()
                .expect("changed existing-admission identity must fail closed");
            assert!(!error.to_string().contains("original retry prompt"));
            assert_eq!(snapshot_authority_files(), bytes_before);
        }

        let first_claim = runtime
            .claim_admission_transport(
                &authority,
                &reconstructed,
                &joined.record.retained_participant_id,
                None,
            )
            .unwrap();
        assert!(first_claim.newly_claimed);
        let bytes_after_claim = snapshot_authority_files();
        let joined_claim = runtime
            .claim_admission_transport(
                &authority,
                &reconstructed,
                &joined.record.retained_participant_id,
                None,
            )
            .unwrap();
        assert!(!joined_claim.newly_claimed);
        assert_eq!(joined_claim.record, first_claim.record);
        assert_eq!(snapshot_authority_files(), bytes_after_claim);

        let new_plan = admission_plan(
            &authority,
            "current-authority-new-slot",
            "new slot prompt",
            4,
        );
        let new_slot = runtime
            .reserve_admission_slot(&authority, &new_plan, None)
            .unwrap();
        assert!(!new_slot.joined);
        assert_eq!(
            new_slot.record.admission_authority_revision,
            new_plan.exact_authority.authority_revision
        );
        assert_eq!(
            new_slot.record.admission_authority_record_commitment,
            new_plan.exact_authority.authority_record_commitment
        );
    }

    #[test]
    fn admission_retry_rejects_every_changed_bound_input_without_mutation() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "changed-input", "original prompt", 3);
        let first = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
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
        let mut substituted_authority_commitment = plan.clone();
        substituted_authority_commitment
            .exact_authority
            .authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "00".repeat(32),
        };
        let mut substituted_lineage_commitment = plan.clone();
        substituted_lineage_commitment
            .exact_authority
            .authoritative_lineage_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "11".repeat(32),
        };
        let mut substituted_workspace = plan.clone();
        substituted_workspace
            .exact_authority
            .authority
            .workspace_binding
            .workspace_root
            .physical_path = "/substituted-workspace".into();

        for changed in variants {
            assert!(runtime
                .reserve_admission_slot(&authority, &changed, None)
                .is_err());
            assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
            assert_eq!(authority.read_a12a_root().unwrap(), authority_before);
        }
        for substituted_authority in [
            substituted_authority_commitment,
            substituted_lineage_commitment,
            substituted_workspace,
        ] {
            let joined = runtime
                .reserve_admission_slot(&authority, &substituted_authority, None)
                .unwrap();
            assert!(joined.joined);
            assert_eq!(joined.record, first.record);
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
    fn authority_registration_head_resolution_reconciles_applied_r0_idempotently() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "head-resolution", "prompt", 3);
        let reserved = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        let participant_id = reserved.record.retained_participant_id.clone();
        runtime
            .register_admitted_worker_at(
                &authority,
                &plan,
                Some(AdmissionRegistrationCrashPointV1::AfterR0BeforeAdmissionAdvance),
            )
            .unwrap_err();
        let root_after_r0 = authority.read_a12a_root().unwrap();
        let head = runtime
            .read_admission_record(&authority, "r0-session", &participant_id)
            .unwrap()
            .unwrap();
        assert!(matches!(
            head.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        ));

        let reconciled = runtime
            .reconcile_existing_admission(&authority, &plan, &head, None, None)
            .unwrap();
        assert!(matches!(
            reconciled.state,
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        ));
        assert_eq!(authority.read_a12a_root().unwrap(), root_after_r0);

        let joined = runtime
            .reconcile_existing_admission(&authority, &plan, &reconciled, None, None)
            .unwrap();
        assert_eq!(joined, reconciled);
    }

    #[test]
    fn transport_claim_resolution_joins_ambiguous_nonterminal_without_freeing_capacity() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "transport-resolution", "prompt", 1);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let participant_id = admitted.record.retained_participant_id.clone();
        runtime
            .claim_admission_transport_at(
                &authority,
                &plan,
                &participant_id,
                timestamp("2026-07-16T12:00:00.000000000Z"),
                [61_u8; 16],
                [71_u8; 16],
                Some(AdmissionTransportClaimCrashPointV1::AfterPublicationBeforeResponse),
            )
            .unwrap_err();
        let claimed = runtime
            .read_admission_record(&authority, "r0-session", &participant_id)
            .unwrap()
            .unwrap();
        assert!(matches!(
            claimed.state,
            RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
        ));

        let resolved = runtime
            .reconcile_existing_admission(&authority, &plan, &claimed, None, None)
            .unwrap();
        assert_eq!(resolved, claimed);

        let blocked_plan = admission_plan(&authority, "transport-resolution-blocked", "prompt", 1);
        assert!(runtime
            .reserve_admission_slot(&authority, &blocked_plan, None)
            .is_err());
    }

    #[test]
    fn malformed_committed_admission_identity_fails_closed_without_repair() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "identity-corruption", "prompt", 2);
        let slot = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
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
        let slot = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
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
        let slot = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
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
    fn admission_fingerprint_frames_the_canonical_runtime_plan_exactly_once() {
        #[derive(Serialize)]
        #[serde(deny_unknown_fields)]
        struct DuplicatedRuntimePlan<'a> {
            schema_version: u32,
            descriptor: &'a AgentDescriptorV1,
            runtime_role: &'a str,
            internal_uaa_session_id_domain: &'a str,
            retained_participant_id: &'a str,
            bootstrap_run_id: &'a str,
        }

        let (_parent, authority, _) = started_authority();
        let plan = admission_plan(&authority, "canonical-framing", "prompt", 2);
        let retained_participant_id = "rwp_11111111111111111111111111111111";
        let bootstrap_run_id = "rwr_22222222222222222222222222222222";
        let key_id = "adk_33333333333333333333333333333333";
        let secret_key = [44_u8; 32];

        let request = encode_canonical(
            &plan.spawn_request,
            "encode canonical validated spawn request",
        )
        .unwrap();
        let exact_authority = encode_canonical(
            &plan.exact_authority,
            "encode canonical exact current authority",
        )
        .unwrap();
        let descriptor_runtime = encode_canonical(
            &plan.descriptor_and_runtime_plan,
            "encode canonical descriptor and runtime plan",
        )
        .unwrap();
        let policy_cap = encode_canonical(
            &plan.policy_and_admission_cap,
            "encode canonical policy and admission cap",
        )
        .unwrap();
        let mut expected_input = b"substrate.retained-worker.admission.hmac-input.v1\0".to_vec();
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
            append_len64(&mut expected_input, member).unwrap();
        }
        let expected = RetainedWorkerAdmissionCommitmentV1 {
            schema_version: 1,
            algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256,
            key_id: key_id.into(),
            digest_hex: lower_hex(&hmac_sha256(&secret_key, &expected_input)),
        };

        assert_eq!(
            canonical_spawn_fingerprint(
                key_id,
                &secret_key,
                &plan,
                retained_participant_id,
                bootstrap_run_id,
            )
            .unwrap(),
            expected
        );

        let duplicated_runtime = encode_canonical(
            &DuplicatedRuntimePlan {
                schema_version: 1,
                descriptor: &plan.descriptor_and_runtime_plan.descriptor,
                runtime_role: &plan.descriptor_and_runtime_plan.runtime_role,
                internal_uaa_session_id_domain: &plan
                    .descriptor_and_runtime_plan
                    .internal_uaa_session_id_domain,
                retained_participant_id,
                bootstrap_run_id,
            },
            "encode duplicate-field runtime plan",
        )
        .unwrap();
        let mut duplicated_input = b"substrate.retained-worker.admission.hmac-input.v1\0".to_vec();
        for member in [
            b"substrate.retained-worker.admission.spawn.v1".as_slice(),
            plan.exact_authority.authority_store_id.as_bytes(),
            plan.issuer_request_id.as_bytes(),
            request.as_slice(),
            exact_authority.as_slice(),
            duplicated_runtime.as_slice(),
            policy_cap.as_slice(),
            retained_participant_id.as_bytes(),
            bootstrap_run_id.as_bytes(),
        ] {
            append_len64(&mut duplicated_input, member).unwrap();
        }
        assert_ne!(
            expected.digest_hex,
            lower_hex(&hmac_sha256(&secret_key, &duplicated_input))
        );
    }

    #[test]
    fn admission_registry_contains_neither_prompt_nor_key_material() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let prompt = "private prompt marker B3.2a";
        let plan = admission_plan(&authority, "secret-exclusion", prompt, 2);
        runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
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
    fn queued_promotion_persists_no_request_prompt_payload_or_diagnostic_preimage() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "preimage-first", "ordinary first", 4);
        let marker = "B3.2a-QP-private-prompt-payload-marker-7f3ac491";
        let queued_plan = admission_plan(&authority, "preimage-queued", marker, 4);
        runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let queued = runtime
            .reserve_admission_slot(&authority, &queued_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        let promoted = runtime
            .prepare_admission_registration_head(
                &authority,
                &queued_plan,
                &queued.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:30:00.000000000Z"),
                [131_u8; 16],
            )
            .unwrap();
        assert!(matches!(promoted, AdmissionHeadPreparationV1::Ready(_)));

        let changed = admission_plan(
            &authority,
            "preimage-queued",
            "B3.2a-QP-conflicting-private-marker-90b9bd27",
            4,
        );
        let error = runtime
            .prepare_admission_registration_head(
                &authority,
                &changed,
                &queued.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:31:00.000000000Z"),
                [132_u8; 16],
            )
            .unwrap_err();
        assert!(!error.to_string().contains(marker));
        assert!(!error
            .to_string()
            .contains("B3.2a-QP-conflicting-private-marker-90b9bd27"));

        let exact_authority = String::from_utf8(
            encode_canonical(
                &queued_plan.exact_authority,
                "encode re-presented admission authority",
            )
            .unwrap(),
        )
        .unwrap();
        let trace_path = parent.path().join("trace.jsonl");
        let output = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("execution::agent_runtime::retained_worker_runtime::tests::admission_head_subprocess_worker")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(ADMISSION_HEAD_SUBPROCESS_HOME, parent.path().join("home"))
            .env(ADMISSION_HEAD_SUBPROCESS_ISSUER, "preimage-queued")
            .env(ADMISSION_HEAD_SUBPROCESS_PROMPT, marker)
            .env(ADMISSION_HEAD_SUBPROCESS_EXACT_AUTHORITY, exact_authority)
            .env(ADMISSION_HEAD_SUBPROCESS_MODE, "exact")
            .env("SHIM_TRACE_LOG", &trace_path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "trace-enabled promotion subprocess failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout).replace(marker, "[REDACTED]"),
            String::from_utf8_lossy(&output.stderr).replace(marker, "[REDACTED]")
        );
        for (name, diagnostics) in [
            ("stdout", output.stdout.as_slice()),
            ("stderr", output.stderr.as_slice()),
        ] {
            assert!(
                !diagnostics
                    .windows(marker.len())
                    .any(|window| window == marker.as_bytes()),
                "request/prompt preimage appeared in captured {name} diagnostics"
            );
        }
        assert!(trace_path.is_file());

        for (path, bytes) in regular_file_contents(parent.path()) {
            assert!(
                !bytes
                    .windows(marker.len())
                    .any(|window| window == marker.as_bytes()),
                "request/prompt preimage persisted in {}",
                path.display()
            );
            let conflict = b"B3.2a-QP-conflicting-private-marker-90b9bd27";
            assert!(
                !bytes
                    .windows(conflict.len())
                    .any(|window| window == conflict),
                "conflicting request/prompt preimage persisted in {}",
                path.display()
            );
        }
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

        let joined = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
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
    fn slot_reservation_publication_boundaries_reopen_and_retry_exactly() {
        for (index, crash_point) in [
            AdmissionReservationCrashPointV1::BeforeTempPersistence,
            AdmissionReservationCrashPointV1::AfterTempFsync,
            AdmissionReservationCrashPointV1::BeforeRegistryReplacementPublication,
            AdmissionReservationCrashPointV1::AfterSlotReserved,
        ]
        .into_iter()
        .enumerate()
        {
            let (parent, authority, _) = started_authority();
            let runtime = RetainedWorkerRuntime;
            runtime.initialize_admission_registry(&authority).unwrap();
            let plan = admission_plan(
                &authority,
                &format!("slot-publication-crash-{index}"),
                "prompt",
                2,
            );
            let admission_root = parent
                .path()
                .join("home/authority-v1/retained-worker-admission-v1");
            let registry_path = admission_root.join("registry-v1.json");
            let tmp_root = admission_root.join("tmp");
            let registry_before = fs::read(&registry_path).unwrap();
            let participant_entropy = [11_u8.wrapping_add(index as u8); 16];
            let bootstrap_entropy = [21_u8.wrapping_add(index as u8); 16];
            let publication_nonce = [31_u8.wrapping_add(index as u8); 16];

            let failure = runtime
                .reserve_admission_slot_at(
                    &authority,
                    &plan,
                    timestamp("2026-07-15T13:00:00.000000000Z"),
                    participant_entropy,
                    bootstrap_entropy,
                    publication_nonce,
                    Some(crash_point),
                )
                .unwrap_err();
            let expected_failure = match crash_point {
                AdmissionReservationCrashPointV1::BeforeTempPersistence => {
                    "injected crash before admission slot temp persistence"
                }
                AdmissionReservationCrashPointV1::AfterTempFsync => {
                    "injected crash after admission slot temp fsync"
                }
                AdmissionReservationCrashPointV1::BeforeRegistryReplacementPublication => {
                    "injected crash before admission slot registry publication"
                }
                AdmissionReservationCrashPointV1::AfterSlotReserved => {
                    "injected crash after admission slot reservation"
                }
            };
            assert_eq!(failure.to_string(), expected_failure);
            let durable_after_failure = fs::read(&registry_path).unwrap();
            if crash_point == AdmissionReservationCrashPointV1::AfterSlotReserved {
                assert_ne!(durable_after_failure, registry_before);
                assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 0);
            } else {
                assert_eq!(durable_after_failure, registry_before);
                assert_eq!(
                    fs::read_dir(&tmp_root).unwrap().count(),
                    usize::from(matches!(
                        crash_point,
                        AdmissionReservationCrashPointV1::AfterTempFsync
                            | AdmissionReservationCrashPointV1::BeforeRegistryReplacementPublication
                    ))
                );
            }

            write_private_test_file(
                &tmp_root.join(format!("admission-registry--{}.tmp", "ed".repeat(16))),
                b"conflicting slot orphan",
            );
            let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
            let retried = runtime
                .reserve_admission_slot(&reopened, &plan, None)
                .unwrap();
            assert_eq!(
                retried.joined,
                crash_point == AdmissionReservationCrashPointV1::AfterSlotReserved
            );
            assert!(matches!(
                retried.record.state,
                RetainedWorkerAdmissionStateV1::SlotReserved { .. }
            ));
            if retried.joined {
                assert_eq!(
                    retried.record.retained_participant_id,
                    format!("rwp_{}", lower_hex(&participant_entropy))
                );
                assert_eq!(
                    retried.record.bootstrap_run_id,
                    format!("rwr_{}", lower_hex(&bootstrap_entropy))
                );
            }
            assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 0);
            let joined = runtime
                .reserve_admission_slot(&reopened, &plan, None)
                .unwrap();
            assert!(joined.joined);
            assert_eq!(joined.record, retried.record);
        }

        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        runtime.initialize_admission_registry(&authority).unwrap();
        let plan = admission_plan(&authority, "slot-rename-before-dirsync", "prompt", 2);
        let admission_root = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1");
        let registry_path = admission_root.join("registry-v1.json");
        let tmp_root = admission_root.join("tmp");
        let publication_nonce = [91_u8; 16];
        assert!(runtime
            .reserve_admission_slot_at(
                &authority,
                &plan,
                timestamp("2026-07-15T13:01:00.000000000Z"),
                [92_u8; 16],
                [93_u8; 16],
                publication_nonce,
                Some(AdmissionReservationCrashPointV1::AfterTempFsync),
            )
            .is_err());
        fs::rename(
            tmp_root.join(format!(
                "admission-registry--{}.tmp",
                lower_hex(&publication_nonce)
            )),
            &registry_path,
        )
        .unwrap();
        write_private_test_file(
            &tmp_root.join(format!("admission-registry--{}.tmp", "ec".repeat(16))),
            b"conflicting slot orphan after rename",
        );
        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        let retried = runtime
            .reserve_admission_slot(&reopened, &plan, None)
            .unwrap();
        assert!(retried.joined);
        assert_eq!(
            retried.record.retained_participant_id,
            format!("rwp_{}", lower_hex(&[92_u8; 16]))
        );
        assert_eq!(fs::read_dir(tmp_root).unwrap().count(), 0);
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
        let third_plan = admission_plan(&authority, "head-third", "third prompt", 4);
        let first_slot = runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let second_slot = runtime
            .reserve_admission_slot(&authority, &second_plan, None)
            .unwrap();
        let third_slot = runtime
            .reserve_admission_slot(&authority, &third_plan, None)
            .unwrap();

        let queued = runtime
            .register_admitted_worker(&authority, &second_plan, None)
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
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        assert!(matches!(
            first.record.state,
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        ));
        let released_without_promotion = runtime
            .read_admission_record(
                &authority,
                "r0-session",
                &second_slot.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            released_without_promotion.state,
            RetainedWorkerAdmissionStateV1::SlotReserved {
                slot_sequence: 2,
                ..
            }
        ));
        let still_queued = runtime
            .read_admission_record(
                &authority,
                "r0-session",
                &third_slot.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            still_queued.state,
            RetainedWorkerAdmissionStateV1::SlotReserved {
                slot_sequence: 3,
                ..
            }
        ));
        let second_head = runtime
            .prepare_admission_registration_head(
                &authority,
                &second_plan,
                &second_slot.record.retained_participant_id,
                None,
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
            .register_admitted_worker(&authority, &second_plan, None)
            .unwrap();
        assert!(matches!(
            second.record.state,
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        ));
        let third_still_queued_without_head = runtime
            .read_admission_record(
                &authority,
                "r0-session",
                &third_slot.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            third_still_queued_without_head.state,
            RetainedWorkerAdmissionStateV1::SlotReserved {
                slot_sequence: 3,
                ..
            }
        ));
        let third = runtime
            .register_admitted_worker(&authority, &third_plan, None)
            .unwrap();
        assert!(matches!(
            third.record.state,
            RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
        ));
        assert_eq!(
            authority
                .resolve_current_exact("r0-session", None)
                .unwrap()
                .authority
                .authority_revision,
            4
        );
        let exact_retry = runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        assert!(exact_retry.joined);
        assert_eq!(exact_retry.record, first.record);
    }

    #[test]
    fn queued_head_acquisition_requires_the_complete_exact_earliest_request() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "queued-exact-first", "first prompt", 4);
        let earliest_plan = admission_plan(
            &authority,
            "queued-exact-earliest",
            "private queued prompt",
            4,
        );
        let later_plan = admission_plan(&authority, "queued-exact-later", "later prompt", 4);
        runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let earliest = runtime
            .reserve_admission_slot(&authority, &earliest_plan, None)
            .unwrap();
        let later = runtime
            .reserve_admission_slot(&authority, &later_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();

        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (admission_root, key_path) = initialized_admission_paths(&parent, &identity);
        let registry_path = admission_root.join("registry-v1.json");
        let no_head_registry = fs::read(&registry_path).unwrap();
        let no_head: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&no_head_registry, "decode no-head registry").unwrap();
        let queued_records = no_head.records_by_session["r0-session"]
            .values()
            .filter(|record| {
                matches!(
                    record.state,
                    RetainedWorkerAdmissionStateV1::SlotReserved { .. }
                )
            })
            .count();
        let head_records = no_head.records_by_session["r0-session"]
            .values()
            .filter(|record| {
                matches!(
                    record.state,
                    RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
                )
            })
            .count();
        assert_eq!(queued_records, 2);
        assert_eq!(head_records, 0);

        let current_observation_substitution = admission_plan(
            &authority,
            "queued-exact-earliest",
            "private queued prompt",
            4,
        );
        assert_ne!(
            current_observation_substitution
                .exact_authority
                .authority_revision,
            earliest_plan.exact_authority.authority_revision
        );
        assert!(runtime
            .prepare_admission_registration_head(
                &authority,
                &current_observation_substitution,
                &earliest.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:09:00.000000000Z"),
                [80_u8; 16],
            )
            .is_err());
        assert_eq!(fs::read(&registry_path).unwrap(), no_head_registry);

        let later_retry = runtime
            .prepare_admission_registration_head(
                &authority,
                &later_plan,
                &later.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:10:00.000000000Z"),
                [81_u8; 16],
            )
            .unwrap();
        assert!(matches!(later_retry, AdmissionHeadPreparationV1::Queued));
        assert_eq!(fs::read(&registry_path).unwrap(), no_head_registry);

        let mut changed_plans = Vec::new();
        let mut changed = earliest_plan.clone();
        changed.spawn_request.payload.prompt = "changed prompt".into();
        changed_plans.push(changed);
        let mut changed = earliest_plan.clone();
        changed.spawn_request.action = "changed_action".into();
        changed_plans.push(changed);
        let mut changed = earliest_plan.clone();
        changed.spawn_request.caller_participant_id = "changed-caller".into();
        changed_plans.push(changed);
        let mut changed = earliest_plan.clone();
        changed.exact_authority.caller_participant_id = "changed-authority-caller".into();
        changed_plans.push(changed);
        let mut changed = earliest_plan.clone();
        changed.descriptor_and_runtime_plan.runtime_role = "changed-role".into();
        changed_plans.push(changed);
        let mut changed = earliest_plan.clone();
        changed.policy_and_admission_cap.max_live_retained_workers += 1;
        changed_plans.push(changed);

        for (index, changed) in changed_plans.into_iter().enumerate() {
            let error = runtime
                .prepare_admission_registration_head(
                    &authority,
                    &changed,
                    &earliest.record.retained_participant_id,
                    None,
                    timestamp("2026-07-15T14:11:00.000000000Z"),
                    [90_u8.wrapping_add(index as u8); 16],
                )
                .unwrap_err();
            assert!(!error.to_string().contains("private queued prompt"));
            assert_eq!(fs::read(&registry_path).unwrap(), no_head_registry);
        }

        let wrong_participant = runtime
            .prepare_admission_registration_head(
                &authority,
                &earliest_plan,
                &later.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:12:00.000000000Z"),
                [101_u8; 16],
            )
            .unwrap_err();
        assert!(!wrong_participant
            .to_string()
            .contains("private queued prompt"));
        assert_eq!(fs::read(&registry_path).unwrap(), no_head_registry);

        let envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(key_path).unwrap(), "decode fixture key").unwrap();
        let resolved = authority.resolve_current_exact("r0-session", None).unwrap();
        let mut current_plan = earliest_plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let authority_root = VersionedStateRoot::V2(authority.read_a12a_root().unwrap());
        let mut changed_bootstrap_registry = no_head.clone();
        changed_bootstrap_registry
            .records_by_session
            .get_mut("r0-session")
            .unwrap()
            .get_mut(&earliest.record.retained_participant_id)
            .unwrap()
            .bootstrap_run_id = "rwr_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();
        let changed_bootstrap_before = changed_bootstrap_registry.clone();
        assert!(prepare_registration_head_in_registry(
            &mut changed_bootstrap_registry,
            &authority_root,
            &current_plan,
            &earliest_plan,
            &earliest.record.retained_participant_id,
            None,
            &envelope.secret_key,
            timestamp("2026-07-15T14:13:00.000000000Z"),
            None,
        )
        .is_err());
        assert_eq!(changed_bootstrap_registry, changed_bootstrap_before);

        let acquired = runtime
            .prepare_admission_registration_head(
                &authority,
                &earliest_plan,
                &earliest.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:14:00.000000000Z"),
                [111_u8; 16],
            )
            .unwrap();
        let AdmissionHeadPreparationV1::Ready(acquired) = acquired else {
            panic!("the exact earliest retry must acquire the released head")
        };
        assert!(matches!(
            acquired.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
                authority_revision_expected: 2,
                ..
            }
        ));
        let acquired_registry = fs::read(&registry_path).unwrap();

        let lost_response_retry = runtime
            .prepare_admission_registration_head(
                &authority,
                &earliest_plan,
                &earliest.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:15:00.000000000Z"),
                [112_u8; 16],
            )
            .unwrap();
        assert_eq!(
            lost_response_retry,
            AdmissionHeadPreparationV1::Ready(acquired)
        );
        assert_eq!(fs::read(&registry_path).unwrap(), acquired_registry);

        let blocked_later = runtime
            .prepare_admission_registration_head(
                &authority,
                &later_plan,
                &later.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:16:00.000000000Z"),
                [113_u8; 16],
            )
            .unwrap();
        assert!(matches!(blocked_later, AdmissionHeadPreparationV1::Queued));
        assert_eq!(fs::read(registry_path).unwrap(), acquired_registry);
    }

    #[test]
    fn reopen_preserves_no_head_with_queued_slots_until_exact_retry() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "reopen-no-head-first", "first", 3);
        let queued_plan = admission_plan(&authority, "reopen-no-head-queued", "queued", 3);
        runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let queued = runtime
            .reserve_admission_slot(&authority, &queued_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        drop(authority);

        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        let before_retry = runtime
            .read_admission_record(
                &reopened,
                "r0-session",
                &queued.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            before_retry.state,
            RetainedWorkerAdmissionStateV1::SlotReserved {
                slot_sequence: 2,
                ..
            }
        ));
        let promoted = runtime
            .prepare_admission_registration_head(
                &reopened,
                &queued_plan,
                &queued.record.retained_participant_id,
                None,
                timestamp("2026-07-15T14:20:00.000000000Z"),
                [121_u8; 16],
            )
            .unwrap();
        assert!(matches!(promoted, AdmissionHeadPreparationV1::Ready(_)));
    }

    #[test]
    fn concurrent_identical_earliest_retries_publish_one_exact_head() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "concurrent-head-first", "first", 4);
        let queued_plan = admission_plan(
            &authority,
            "concurrent-head-earliest",
            "same queued prompt",
            4,
        );
        runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let queued = runtime
            .reserve_admission_slot(&authority, &queued_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        let exact_authority = String::from_utf8(
            encode_canonical(
                &queued_plan.exact_authority,
                "encode re-presented admission authority",
            )
            .unwrap(),
        )
        .unwrap();

        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::admission_head_subprocess_worker";
        let spawn = || {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(ADMISSION_HEAD_SUBPROCESS_HOME, parent.path().join("home"))
                .env(ADMISSION_HEAD_SUBPROCESS_ISSUER, "concurrent-head-earliest")
                .env(ADMISSION_HEAD_SUBPROCESS_PROMPT, "same queued prompt")
                .env(ADMISSION_HEAD_SUBPROCESS_EXACT_AUTHORITY, &exact_authority)
                .env(ADMISSION_HEAD_SUBPROCESS_MODE, "exact")
                .spawn()
                .unwrap()
        };
        let mut first = spawn();
        let mut second = spawn();
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());

        let head = runtime
            .read_admission_record(
                &authority,
                "r0-session",
                &queued.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            head.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
                authority_revision_expected: 2,
                ..
            }
        ));
        assert_eq!(head.record_revision, queued.record.record_revision + 1);
    }

    #[test]
    fn concurrent_conflicting_earliest_retries_leave_one_exact_head() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "conflicting-head-first", "first", 4);
        let queued_plan = admission_plan(
            &authority,
            "conflicting-head-earliest",
            "original queued prompt",
            4,
        );
        runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let queued = runtime
            .reserve_admission_slot(&authority, &queued_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        let exact_authority = String::from_utf8(
            encode_canonical(
                &queued_plan.exact_authority,
                "encode re-presented admission authority",
            )
            .unwrap(),
        )
        .unwrap();

        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::admission_head_subprocess_worker";
        let spawn = |mode: &str| {
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(ADMISSION_HEAD_SUBPROCESS_HOME, parent.path().join("home"))
                .env(
                    ADMISSION_HEAD_SUBPROCESS_ISSUER,
                    "conflicting-head-earliest",
                )
                .env(ADMISSION_HEAD_SUBPROCESS_PROMPT, "original queued prompt")
                .env(ADMISSION_HEAD_SUBPROCESS_EXACT_AUTHORITY, &exact_authority)
                .env(ADMISSION_HEAD_SUBPROCESS_MODE, mode)
                .spawn()
                .unwrap()
        };
        let mut exact = spawn("exact");
        let mut conflict = spawn("conflict");
        assert!(exact.wait().unwrap().success());
        assert!(conflict.wait().unwrap().success());

        let head = runtime
            .read_admission_record(
                &authority,
                "r0-session",
                &queued.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            head.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead {
                authority_revision_expected: 2,
                ..
            }
        ));
        assert_eq!(head.record_revision, queued.record.record_revision + 1);
    }

    #[test]
    fn independent_authority_sessions_may_each_hold_one_registration_head() {
        let (_first_parent, first_authority, _) = started_authority();
        let (_second_parent, second_authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&first_authority, "independent-head-first", "first", 2);
        let second_plan = admission_plan(&second_authority, "independent-head-second", "second", 2);

        assert_eq!(
            runtime
                .register_admitted_worker_at(
                    &first_authority,
                    &first_plan,
                    Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
                )
                .unwrap_err()
                .to_string(),
            "injected crash while admission registration head"
        );
        assert_eq!(
            runtime
                .register_admitted_worker_at(
                    &second_authority,
                    &second_plan,
                    Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
                )
                .unwrap_err()
                .to_string(),
            "injected crash while admission registration head"
        );

        let first_slot = runtime
            .reserve_admission_slot(&first_authority, &first_plan, None)
            .unwrap();
        let second_slot = runtime
            .reserve_admission_slot(&second_authority, &second_plan, None)
            .unwrap();
        assert!(matches!(
            first_slot.record.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        ));
        assert!(matches!(
            second_slot.record.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        ));
        assert_ne!(
            first_slot.record.authority_store_id,
            second_slot.record.authority_store_id
        );
    }

    #[test]
    fn one_registry_may_hold_one_registration_head_in_each_session_bucket() {
        let (parent, authority, _) = started_authority();
        start_additional_authority_session(&authority);
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "multi-session-head-first", "first", 2);
        let second_plan =
            admission_plan_for_second_session(&authority, "multi-session-head-second", "second", 2);
        assert_eq!(
            runtime
                .register_admitted_worker_at(
                    &authority,
                    &first_plan,
                    Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
                )
                .unwrap_err()
                .to_string(),
            "injected crash while admission registration head"
        );
        assert_eq!(
            runtime
                .register_admitted_worker_at(
                    &authority,
                    &second_plan,
                    Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
                )
                .unwrap_err()
                .to_string(),
            "injected crash while admission registration head"
        );

        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
            &fs::read(&registry_path).unwrap(),
            "decode production-path multi-session registry",
        )
        .unwrap();
        let first_locator = &registry.issuer_request_index["multi-session-head-first"];
        let first = &registry.records_by_session[&first_locator.orchestration_session_id]
            [&first_locator.retained_participant_id];
        let second_locator = &registry.issuer_request_index["multi-session-head-second"];
        let second = &registry.records_by_session[&second_locator.orchestration_session_id]
            [&second_locator.retained_participant_id];

        assert!(matches!(
            first.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        ));
        assert!(matches!(
            second.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        ));
        assert_eq!(first.orchestration_session_id, "r0-session");
        assert_eq!(second.orchestration_session_id, "second-session");
        assert_eq!(first.authority_store_id, second.authority_store_id);
        validate_admission_registry(
            &registry,
            &VersionedStateRoot::V2(authority.read_a12a_root().unwrap()),
        )
        .unwrap();
    }

    #[test]
    fn queued_head_promotion_requires_complete_legacy_v2_r0_ancestry() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "ancestry-first", "first prompt", 3);
        let queued_plan = admission_plan(&authority, "ancestry-queued", "queued prompt", 3);
        runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let queued = runtime
            .reserve_admission_slot(&authority, &queued_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        let root = authority.read_a12a_root().unwrap();
        let resolved = authority.resolve_current_exact("r0-session", None).unwrap();
        let current = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &current,
            &queued.record,
            None,
        )
        .is_ok());

        let mut missing_link = root.clone();
        missing_link.retained_worker_registration_journal.clear();
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &missing_link,
            &current,
            &queued.record,
            None,
        )
        .is_err());

        let mut skipped_revision = root.clone();
        skipped_revision
            .retained_worker_registration_journal
            .values_mut()
            .next()
            .unwrap()
            .authority_revision_after += 1;
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &skipped_revision,
            &current,
            &queued.record,
            None,
        )
        .is_err());

        let mut ambiguous_parent = root.clone();
        let duplicate = ambiguous_parent
            .retained_worker_registration_journal
            .values()
            .next()
            .unwrap()
            .clone();
        ambiguous_parent
            .retained_worker_registration_journal
            .insert("ambiguous-registration-parent".into(), duplicate);
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &ambiguous_parent,
            &current,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_caller = current.clone();
        changed_caller.authority.active_authoritative_participant_id =
            Some("changed-caller".into());
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &changed_caller,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_workspace = current.clone();
        changed_workspace
            .authority
            .workspace_binding
            .workspace_root
            .physical_path = "/changed-workspace".into();
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &changed_workspace,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_world = current.clone();
        changed_world
            .authority
            .world_binding
            .as_mut()
            .unwrap()
            .world_id = "changed-world".into();
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &changed_world,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_policy = current.clone();
        changed_policy
            .authority
            .current_policy_ref
            .as_mut()
            .unwrap()
            .ref_id = "changed-policy-ref".into();
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &changed_policy,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_origin = current.clone();
        let DurableSessionAuthorityOriginV1::StartIntent {
            issuer_request_id, ..
        } = &mut changed_origin.authority.origin;
        *issuer_request_id = "changed-origin".into();
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &changed_origin,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_unrelated_refs = current.clone();
        let mut unrelated_ref = current.caller_descriptor_ref.clone();
        unrelated_ref.ref_id = "changed-unrelated-ref".into();
        changed_unrelated_refs
            .authority
            .internal_resume_handle_refs
            .push(unrelated_ref);
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &changed_unrelated_refs,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_posture = current.clone();
        changed_posture.authority.lifecycle_posture =
            super::super::host_session_authority::schema::HostSessionPostureV1::AwaitingAttention;
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &changed_posture,
            &queued.record,
            None,
        )
        .is_err());

        let mut transition_or_terminal_change = current.clone();
        transition_or_terminal_change.authority_revision += 1;
        transition_or_terminal_change.authority.authority_revision += 1;
        assert!(validate_admission_to_current_exact_authority_ancestry(
            &root,
            &transition_or_terminal_change,
            &queued.record,
            None,
        )
        .is_err());

        let mut changed_wrapper_caller_plan = queued_plan;
        changed_wrapper_caller_plan.exact_authority = current;
        changed_wrapper_caller_plan
            .exact_authority
            .caller_participant_id = "changed-wrapper-caller".into();
        assert!(validate_admission_plan(
            &VersionedStateRoot::V2(root),
            &changed_wrapper_caller_plan,
            None,
        )
        .is_err());
    }

    #[test]
    fn rewinding_r0_ancestry_ignores_non_r0_current_authority_fields() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let first_plan = admission_plan(&authority, "rewind-first", "first prompt", 3);
        let queued_plan = admission_plan(&authority, "rewind-queued", "queued prompt", 3);
        runtime
            .reserve_admission_slot(&authority, &first_plan, None)
            .unwrap();
        let queued = runtime
            .reserve_admission_slot(&authority, &queued_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker(&authority, &first_plan, None)
            .unwrap();
        let root = authority.read_a12a_root().unwrap();
        let resolved = authority.resolve_current_exact("r0-session", None).unwrap();
        let mut current = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        current.authority.lifecycle_posture = HostSessionPostureV1::AwaitingAttention;
        current.authority.active_authoritative_participant_id = Some("changed-caller".into());

        let rewound = reconstruct_exact_authority_at_revision(
            &root,
            &current,
            queued.record.admission_authority_revision,
            None,
        )
        .unwrap();

        assert_eq!(
            rewound.authority_revision,
            queued.record.admission_authority_revision
        );
        assert_eq!(
            rewound.authority_record_commitment,
            queued.record.admission_authority_record_commitment
        );
        assert_eq!(
            rewound
                .authority
                .active_authoritative_participant_id
                .as_deref(),
            Some(resolved.caller.participant_id.as_str())
        );
        assert_eq!(
            rewound.authority.lifecycle_posture,
            HostSessionPostureV1::ActiveAttached
        );
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

        let reconciled = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
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
    fn post_r0_join_rejects_every_inexact_registered_graph_member() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "r0-complete-graph", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let RetainedWorkerAdmissionStateV1::PreTransportNonterminal {
            registration: admission_registration,
        } = &admitted.record.state
        else {
            panic!("registered admission must be pre-transport")
        };
        let root = authority.read_a12a_root().unwrap();
        let registration = root
            .retained_worker_registration_journal
            .get(&admission_registration.registration_id)
            .unwrap();
        let result = RetainedWorkerRegistrationResultV1 {
            registration_id: registration.registration_id.clone(),
            registration_commitment: canonical_registration_commitment(registration).unwrap(),
            authority_store_id: root.authority_store_id.clone(),
            orchestration_session_id: registration.orchestration_session_id.clone(),
            retained_participant_id: registration.retained_participant_id.clone(),
            retained_worker_ref: registration.retained_worker_ref.clone(),
            authority_revision_after: registration.authority_revision_after,
            authority_record_commitment_after: registration
                .authority_record_commitment_after
                .clone(),
        };
        let resolved = runtime
            .resolve_retained_target(&authority, &result)
            .unwrap();
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(
            &authority.resolve_current_exact("r0-session", None).unwrap(),
        );

        validate_post_r0_registered_graph(
            &root,
            &admitted.record,
            &current_plan,
            admission_registration,
            &result,
            &resolved,
            None,
        )
        .unwrap();

        let mut variants = Vec::new();
        let mut changed = resolved.clone();
        changed.descriptor.agent_id = "different-agent".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.descriptor.backend_id = "cli:different".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.descriptor.protocol = "different.protocol".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.descriptor.execution_scope = AgentExecutionScopeV1::Host;
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.resume_handle.participant_id = "rwp_different".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.resume_handle.orchestration_session_id = "different-session".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.resume_handle.backend_id = "cli:different".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.resume_handle.protocol = "different.protocol".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.resume_handle.internal_uaa_session_id = "uaa_different".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.retained_worker.orchestration_session_id = "different-session".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.retained_worker.participant_id = "rwp_different".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.retained_worker.world_binding.world_generation += 1;
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.retained_worker.descriptor_ref = resolved.registration.resume_handle_ref.clone();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.retained_worker.resume_handle_ref = resolved.registration.descriptor_ref.clone();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.retained_worker.policy_ref = resolved.registration.descriptor_ref.clone();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.current_policy.policy_revision = "different-policy".into();
        variants.push(changed);
        let mut changed = resolved.clone();
        changed.registration.registration_id = "rr_different".into();
        variants.push(changed);

        for changed in variants {
            assert!(validate_post_r0_registered_graph(
                &root,
                &admitted.record,
                &current_plan,
                admission_registration,
                &result,
                &changed,
                None,
            )
            .is_err());
        }

        let mut changed_result = result.clone();
        changed_result.registration_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "55".repeat(32),
        };
        assert!(validate_post_r0_registered_graph(
            &root,
            &admitted.record,
            &current_plan,
            admission_registration,
            &changed_result,
            &resolved,
            None,
        )
        .is_err());

        let mut changed_result = result.clone();
        changed_result.authority_revision_after += 1;
        assert!(validate_post_r0_registered_graph(
            &root,
            &admitted.record,
            &current_plan,
            admission_registration,
            &changed_result,
            &resolved,
            None,
        )
        .is_err());

        let mut changed_result = result.clone();
        changed_result.authority_record_commitment_after =
            AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "00".repeat(32),
            };
        assert!(validate_post_r0_registered_graph(
            &root,
            &admitted.record,
            &current_plan,
            admission_registration,
            &changed_result,
            &resolved,
            None,
        )
        .is_err());

        let zero_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "66".repeat(32),
        };
        let issuer_request_id = format!(
            "retained-worker-registration:{}",
            admitted.record.issuer_request_id
        );
        let mut changed_roots = Vec::new();
        let mut changed = root.clone();
        changed
            .retained_worker_registration_request_index
            .get_mut(&issuer_request_id)
            .unwrap()
            .descriptor_commitment = zero_commitment.clone();
        changed_roots.push(changed);
        let mut changed = root.clone();
        changed
            .retained_worker_registration_request_index
            .get_mut(&issuer_request_id)
            .unwrap()
            .resume_handle_commitment = zero_commitment.clone();
        changed_roots.push(changed);
        let mut changed = root.clone();
        changed
            .retained_worker_registration_request_index
            .get_mut(&issuer_request_id)
            .unwrap()
            .retained_worker_commitment = zero_commitment.clone();
        changed_roots.push(changed);
        let mut changed = root.clone();
        changed
            .retained_worker_registration_journal
            .get_mut(&admission_registration.registration_id)
            .unwrap()
            .authoritative_lineage_commitment_after = zero_commitment.clone();
        changed_roots.push(changed);
        let mut changed = root.clone();
        changed
            .retained_worker_registration_journal
            .get_mut(&admission_registration.registration_id)
            .unwrap()
            .descriptor_ref
            .commitment = zero_commitment.clone();
        changed_roots.push(changed);
        let mut changed = root.clone();
        changed
            .retained_worker_registration_journal
            .get_mut(&admission_registration.registration_id)
            .unwrap()
            .resume_handle_ref
            .commitment = zero_commitment.clone();
        changed_roots.push(changed);
        let mut changed = root.clone();
        changed
            .retained_worker_registration_journal
            .get_mut(&admission_registration.registration_id)
            .unwrap()
            .retained_worker_ref
            .commitment = zero_commitment;
        changed_roots.push(changed);

        for changed_root in changed_roots {
            assert!(validate_post_r0_registered_graph(
                &changed_root,
                &admitted.record,
                &current_plan,
                admission_registration,
                &result,
                &resolved,
                None,
            )
            .is_err());
        }

        let mut changed_record = admitted.record.clone();
        changed_record.current_policy_revision = "different-policy".into();
        assert!(validate_post_r0_registered_graph(
            &root,
            &changed_record,
            &current_plan,
            admission_registration,
            &result,
            &resolved,
            None,
        )
        .is_err());
    }

    #[test]
    fn idempotent_registration_advance_revalidates_the_complete_post_r0_graph() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "r0-idempotent-advance", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let post_r0_graph = runtime
            .resolve_admitted_record_graph(&authority, &admitted.record)
            .unwrap();
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (admission_root, key_path) = initialized_admission_paths(&parent, &identity);
        let registry_bytes = fs::read(admission_root.join("registry-v1.json")).unwrap();
        let mut registry: RetainedWorkerAdmissionRegistryV1 =
            decode_canonical(&registry_bytes, "decode fixture registry").unwrap();
        registry
            .records_by_session
            .get_mut(&admitted.record.orchestration_session_id)
            .unwrap()
            .get_mut(&admitted.record.retained_participant_id)
            .unwrap()
            .current_policy_revision = "forged-policy-revision".into();
        let envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(key_path).unwrap(), "decode fixture key").unwrap();
        let resolved = authority.resolve_current_exact("r0-session", None).unwrap();
        let mut current_plan = plan.clone();
        current_plan.exact_authority = CanonicalExactCurrentAuthorityV1::from_resolved(&resolved);
        let authority_root = VersionedStateRoot::V2(authority.read_a12a_root().unwrap());

        assert!(advance_registration_head_in_registry(
            &mut registry,
            &authority_root,
            &current_plan,
            &plan,
            &AdmissionRegistrationAdvanceInputV1 {
                retained_participant_id: &admitted.record.retained_participant_id,
                secret_key: &envelope.secret_key,
                post_r0_graph: &post_r0_graph,
            },
            None,
            None,
        )
        .is_err());
    }

    #[test]
    fn registration_head_and_advance_publication_crashes_reopen_and_retry_exactly() {
        for (index, (crash_point, publish_staged_before_directory_fsync)) in [
            (
                AdmissionRegistrationCrashPointV1::BeforeRegistrationHeadPublication,
                false,
            ),
            (
                AdmissionRegistrationCrashPointV1::DuringRegistrationHeadPublication,
                false,
            ),
            (
                AdmissionRegistrationCrashPointV1::DuringRegistrationHeadPublication,
                true,
            ),
            (
                AdmissionRegistrationCrashPointV1::AfterRegistrationHeadPublicationBeforeResponse,
                false,
            ),
            (
                AdmissionRegistrationCrashPointV1::BeforeAdmissionAdvancePublication,
                false,
            ),
            (
                AdmissionRegistrationCrashPointV1::DuringAdmissionAdvancePublication,
                false,
            ),
            (
                AdmissionRegistrationCrashPointV1::DuringAdmissionAdvancePublication,
                true,
            ),
            (
                AdmissionRegistrationCrashPointV1::AfterAdmissionAdvancePublicationBeforeResponse,
                false,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let (parent, authority, _) = started_authority();
            let runtime = RetainedWorkerRuntime;
            let first_plan = admission_plan(
                &authority,
                &format!("registration-publication-first-{index}"),
                "first request",
                3,
            );
            let issuer = format!("registration-publication-queued-{index}");
            let plan = admission_plan(&authority, &issuer, "exact queued retry prompt", 3);
            let first_slot = runtime
                .reserve_admission_slot(&authority, &first_plan, None)
                .unwrap();
            let queued = runtime
                .reserve_admission_slot(&authority, &plan, None)
                .unwrap();
            runtime
                .register_admitted_worker(&authority, &first_plan, None)
                .unwrap();
            assert!(matches!(
                queued.record.state,
                RetainedWorkerAdmissionStateV1::SlotReserved {
                    slot_sequence: 2,
                    ..
                }
            ));

            let is_head_publication = matches!(
                crash_point,
                AdmissionRegistrationCrashPointV1::BeforeRegistrationHeadPublication
                    | AdmissionRegistrationCrashPointV1::DuringRegistrationHeadPublication
                    | AdmissionRegistrationCrashPointV1::AfterRegistrationHeadPublicationBeforeResponse
            );
            let failure = if is_head_publication {
                runtime
                    .prepare_admission_registration_head_with(
                        &authority,
                        &plan,
                        &queued.record.retained_participant_id,
                        None,
                        timestamp("2026-07-15T20:00:00.000000000Z"),
                        [140_u8.wrapping_add(index as u8); 16],
                        Some(crash_point),
                    )
                    .map(drop)
            } else {
                runtime
                    .register_admitted_worker_at(&authority, &plan, Some(crash_point))
                    .map(drop)
            };
            assert!(
                failure.is_err(),
                "no success may be reported before the selected durable response boundary"
            );

            let admission_root = parent
                .path()
                .join("home/authority-v1/retained-worker-admission-v1");
            let registry_path = admission_root.join("registry-v1.json");
            let tmp_root = admission_root.join("tmp");
            if publish_staged_before_directory_fsync {
                let staged = fs::read_dir(&tmp_root)
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .collect::<Vec<_>>();
                assert_eq!(staged.len(), 1);
                fs::rename(&staged[0], &registry_path).unwrap();
                write_private_test_file(
                    &tmp_root.join(format!("admission-registry--{}.tmp", "ff".repeat(16))),
                    b"conflicting orphan",
                );
            }
            let registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
                &fs::read(&registry_path).unwrap(),
                "decode crashed registration registry",
            )
            .unwrap();
            let locator = &registry.issuer_request_index[&issuer];
            let durable = &registry.records_by_session[&locator.orchestration_session_id]
                [&locator.retained_participant_id];
            if is_head_publication {
                if publish_staged_before_directory_fsync
                    || crash_point
                        == AdmissionRegistrationCrashPointV1::AfterRegistrationHeadPublicationBeforeResponse
                {
                    assert!(matches!(
                        durable.state,
                        RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
                    ));
                } else {
                    assert!(matches!(
                        durable.state,
                        RetainedWorkerAdmissionStateV1::SlotReserved { .. }
                    ));
                }
            } else if publish_staged_before_directory_fsync
                || crash_point
                    == AdmissionRegistrationCrashPointV1::AfterAdmissionAdvancePublicationBeforeResponse
            {
                assert!(matches!(
                    durable.state,
                    RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
                ));
            } else {
                assert!(matches!(
                    durable.state,
                    RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
                ));
            }
            if publish_staged_before_directory_fsync {
                assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 1);
            } else {
                let expected_temp_count = usize::from(matches!(
                    crash_point,
                    AdmissionRegistrationCrashPointV1::DuringRegistrationHeadPublication
                        | AdmissionRegistrationCrashPointV1::DuringAdmissionAdvancePublication
                ));
                assert_eq!(
                    fs::read_dir(&tmp_root).unwrap().count(),
                    expected_temp_count
                );
            }

            let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
            let retried = runtime
                .register_admitted_worker(&reopened, &plan, None)
                .unwrap();
            assert!(matches!(
                retried.record.state,
                RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
            ));
            assert_eq!(fs::read_dir(&tmp_root).unwrap().count(), 0);
            let joined = runtime
                .register_admitted_worker(&reopened, &plan, None)
                .unwrap();
            assert!(joined.joined);
            assert_eq!(joined.record, retried.record);
            let first = runtime
                .read_admission_record(
                    &reopened,
                    "r0-session",
                    &first_slot.record.retained_participant_id,
                )
                .unwrap()
                .unwrap();
            assert!(matches!(
                first.state,
                RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
            ));
        }
    }

    #[test]
    fn transport_claim_crash_boundaries_retry_exactly_without_duplicate_send_authority() {
        for (index, crash_point) in [
            AdmissionTransportClaimCrashPointV1::BeforeTempPersistence,
            AdmissionTransportClaimCrashPointV1::AfterTempFsync,
            AdmissionTransportClaimCrashPointV1::BeforeRegistryReplacementPublication,
            AdmissionTransportClaimCrashPointV1::AfterPublicationBeforeResponse,
        ]
        .into_iter()
        .enumerate()
        {
            let (parent, authority, _) = started_authority();
            let runtime = RetainedWorkerRuntime;
            let plan = admission_plan(&authority, &format!("claim-crash-{index}"), "prompt", 3);
            let admitted = runtime
                .register_admitted_worker(&authority, &plan, None)
                .unwrap();
            let participant_id = admitted.record.retained_participant_id.clone();
            let identity = runtime.initialize_admission_registry(&authority).unwrap();
            let (admission_root, _) = initialized_admission_paths(&parent, &identity);
            let registry_path = admission_root.join("registry-v1.json");
            let registry_before = fs::read(&registry_path).unwrap();
            let failure = runtime
                .claim_admission_transport_at(
                    &authority,
                    &plan,
                    &participant_id,
                    timestamp("2026-07-15T14:02:00.000000000Z"),
                    [61_u8.wrapping_add(index as u8); 16],
                    [71_u8.wrapping_add(index as u8); 16],
                    Some(crash_point),
                )
                .unwrap_err();
            let expected_failure = match crash_point {
                AdmissionTransportClaimCrashPointV1::BeforeTempPersistence => {
                    "injected crash before transport-claim temp persistence"
                }
                AdmissionTransportClaimCrashPointV1::AfterTempFsync => {
                    "injected crash after transport-claim temp fsync"
                }
                AdmissionTransportClaimCrashPointV1::BeforeRegistryReplacementPublication => {
                    "injected crash before transport-claim registry publication"
                }
                AdmissionTransportClaimCrashPointV1::AfterPublicationBeforeResponse => {
                    "injected crash after transport-claim publication before response"
                }
            };
            assert_eq!(failure.to_string(), expected_failure);
            let staged_temp_count = fs::read_dir(admission_root.join("tmp")).unwrap().count();
            let durable_after_failure = runtime
                .read_admission_record(&authority, "r0-session", &participant_id)
                .unwrap()
                .unwrap();
            if crash_point == AdmissionTransportClaimCrashPointV1::AfterPublicationBeforeResponse {
                assert!(matches!(
                    durable_after_failure.state,
                    RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
                ));
                assert_ne!(fs::read(&registry_path).unwrap(), registry_before);
            } else {
                assert!(matches!(
                    durable_after_failure.state,
                    RetainedWorkerAdmissionStateV1::PreTransportNonterminal { .. }
                ));
                assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
                assert_eq!(
                    staged_temp_count,
                    usize::from(matches!(
                        crash_point,
                        AdmissionTransportClaimCrashPointV1::AfterTempFsync
                            | AdmissionTransportClaimCrashPointV1::BeforeRegistryReplacementPublication
                    ))
                );
            }

            write_private_test_file(
                &admission_root
                    .join("tmp")
                    .join(format!("admission-registry--{}.tmp", "eb".repeat(16))),
                b"conflicting transport-claim orphan",
            );

            let retried = runtime
                .claim_admission_transport_at(
                    &authority,
                    &plan,
                    &participant_id,
                    timestamp("2026-07-15T14:02:00.000000000Z"),
                    [61_u8.wrapping_add(index as u8); 16],
                    [81_u8.wrapping_add(index as u8); 16],
                    None,
                )
                .unwrap();
            assert_eq!(
                retried.newly_claimed,
                crash_point != AdmissionTransportClaimCrashPointV1::AfterPublicationBeforeResponse
            );
            let RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                transport_claim_id,
                claimed_at,
                ..
            } = &retried.record.state
            else {
                panic!("claim retry must resolve the durable transport claim")
            };
            assert_eq!(
                transport_claim_id,
                &format!("rtc_{}", lower_hex(&[61_u8.wrapping_add(index as u8); 16]))
            );
            assert_eq!(claimed_at, &timestamp("2026-07-15T14:02:00.000000000Z"));
            assert_eq!(fs::read_dir(admission_root.join("tmp")).unwrap().count(), 0);

            let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
            let no_steal = runtime
                .claim_admission_transport_at(
                    &reopened,
                    &plan,
                    &participant_id,
                    timestamp("2026-07-15T18:00:00.000000000Z"),
                    [91_u8; 16],
                    [92_u8; 16],
                    None,
                )
                .unwrap();
            assert!(!no_steal.newly_claimed);
            assert_eq!(no_steal.record, retried.record);
        }

        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "claim-rename-before-dirsync", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let participant_id = admitted.record.retained_participant_id.clone();
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (admission_root, _) = initialized_admission_paths(&parent, &identity);
        let registry_path = admission_root.join("registry-v1.json");
        let tmp_root = admission_root.join("tmp");
        let publication_nonce = [111_u8; 16];
        assert!(runtime
            .claim_admission_transport_at(
                &authority,
                &plan,
                &participant_id,
                timestamp("2026-07-15T14:04:00.000000000Z"),
                [112_u8; 16],
                publication_nonce,
                Some(AdmissionTransportClaimCrashPointV1::AfterTempFsync),
            )
            .is_err());
        fs::rename(
            tmp_root.join(format!(
                "admission-registry--{}.tmp",
                lower_hex(&publication_nonce)
            )),
            &registry_path,
        )
        .unwrap();
        write_private_test_file(
            &tmp_root.join(format!("admission-registry--{}.tmp", "ea".repeat(16))),
            b"conflicting transport-claim orphan after rename",
        );
        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        let retried = runtime
            .claim_admission_transport_at(
                &reopened,
                &plan,
                &participant_id,
                timestamp("2026-07-15T14:04:00.000000000Z"),
                [112_u8; 16],
                [113_u8; 16],
                None,
            )
            .unwrap();
        assert!(!retried.newly_claimed);
        assert!(matches!(
            retried.record.state,
            RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal { .. }
        ));
        assert_eq!(fs::read_dir(tmp_root).unwrap().count(), 0);
    }

    #[test]
    fn cross_process_transport_claim_contention_and_process_death_never_steal() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let executable = std::env::current_exe().unwrap();
        let test_name = "execution::agent_runtime::retained_worker_runtime::tests::transport_claim_subprocess_worker";
        let spawn = |issuer: &str,
                     prompt: &str,
                     exact_authority: &CanonicalExactCurrentAuthorityV1,
                     participant_id: &str,
                     entropy: u8,
                     mode: &str,
                     result_path: &std::path::Path| {
            let exact_authority = String::from_utf8(
                encode_canonical(exact_authority, "encode re-presented admission authority")
                    .unwrap(),
            )
            .unwrap();
            Command::new(&executable)
                .arg("--exact")
                .arg(test_name)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(TRANSPORT_CLAIM_SUBPROCESS_HOME, parent.path().join("home"))
                .env(TRANSPORT_CLAIM_SUBPROCESS_ISSUER, issuer)
                .env(TRANSPORT_CLAIM_SUBPROCESS_PROMPT, prompt)
                .env(TRANSPORT_CLAIM_SUBPROCESS_EXACT_AUTHORITY, exact_authority)
                .env(TRANSPORT_CLAIM_SUBPROCESS_PARTICIPANT, participant_id)
                .env(TRANSPORT_CLAIM_SUBPROCESS_ENTROPY, entropy.to_string())
                .env(TRANSPORT_CLAIM_SUBPROCESS_MODE, mode)
                .env(TRANSPORT_CLAIM_SUBPROCESS_RESULT, result_path)
                .spawn()
                .unwrap()
        };

        let identical_plan = admission_plan(&authority, "process-identical", "prompt", 4);
        let identical = runtime
            .register_admitted_worker(&authority, &identical_plan, None)
            .unwrap();
        let participant_id = identical.record.retained_participant_id.clone();
        let first_result = parent.path().join("claim-first-result");
        let second_result = parent.path().join("claim-second-result");
        let mut first = spawn(
            "process-identical",
            "prompt",
            &identical_plan.exact_authority,
            &participant_id,
            101,
            "claim",
            &first_result,
        );
        let mut second = spawn(
            "process-identical",
            "prompt",
            &identical_plan.exact_authority,
            &participant_id,
            102,
            "claim",
            &second_result,
        );
        assert!(first.wait().unwrap().success());
        assert!(second.wait().unwrap().success());
        let mut outcomes = vec![
            fs::read_to_string(first_result).unwrap(),
            fs::read_to_string(second_result).unwrap(),
        ];
        outcomes.sort();
        assert_eq!(outcomes, vec!["joined", "new"]);

        let death_plan = admission_plan(&authority, "process-death", "prompt", 4);
        let death_admitted = runtime
            .register_admitted_worker(&authority, &death_plan, None)
            .unwrap();
        let death_participant_id = death_admitted.record.retained_participant_id.clone();
        let mut crashed = spawn(
            "process-death",
            "prompt",
            &death_plan.exact_authority,
            &death_participant_id,
            103,
            "crash-after-publication",
            &parent.path().join("unused-crash-result"),
        );
        assert!(crashed.wait().unwrap().success());
        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        let after_process_death = runtime
            .claim_admission_transport_at(
                &reopened,
                &death_plan,
                &death_participant_id,
                timestamp("2026-07-15T19:00:00.000000000Z"),
                [104_u8; 16],
                [105_u8; 16],
                None,
            )
            .unwrap();
        assert!(!after_process_death.newly_claimed);
        let RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
            transport_claim_id, ..
        } = &after_process_death.record.state
        else {
            panic!("process death must leave one durable transport claim")
        };
        assert_eq!(
            transport_claim_id,
            &format!("rtc_{}", lower_hex(&[103_u8; 16]))
        );

        let conflict_plan = admission_plan(&authority, "process-conflict", "prompt", 4);
        let conflict_admitted = runtime
            .register_admitted_worker(&authority, &conflict_plan, None)
            .unwrap();
        let conflict_participant_id = conflict_admitted.record.retained_participant_id.clone();
        let correct_result = parent.path().join("claim-correct-result");
        let conflict_result = parent.path().join("claim-conflict-result");
        let mut correct = spawn(
            "process-conflict",
            "prompt",
            &conflict_plan.exact_authority,
            &conflict_participant_id,
            106,
            "claim",
            &correct_result,
        );
        let mut conflicting = spawn(
            "process-conflict",
            "changed prompt",
            &conflict_plan.exact_authority,
            &conflict_participant_id,
            107,
            "conflict",
            &conflict_result,
        );
        let correct_status = correct.wait().unwrap();
        let conflicting_status = conflicting.wait().unwrap();
        assert!(correct_status.success());
        assert!(!conflicting_status.success());
        assert_eq!(fs::read_to_string(correct_result).unwrap(), "new");
        assert!(!conflict_result.exists());
        let durable = runtime
            .read_admission_record(&authority, "r0-session", &conflict_participant_id)
            .unwrap()
            .unwrap();
        let RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
            transport_claim_id, ..
        } = durable.state
        else {
            panic!("conflicting process must not steal the durable claim")
        };
        assert_eq!(
            transport_claim_id,
            format!("rtc_{}", lower_hex(&[106_u8; 16]))
        );
    }

    #[test]
    fn concurrent_transport_claim_has_one_durable_winner_and_never_steals() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "claim-race", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
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
                        None,
                        &AdmissionTransportPublicationInputV1 {
                            claimed_at: timestamp("2026-07-15T14:01:00.000000000Z"),
                            claim_entropy: [entropy; 16],
                            publication_nonce: [entropy.wrapping_add(10); 16],
                            crash_point: None,
                        },
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
            .claim_admission_transport(&authority, &plan, &participant_id, None)
            .unwrap();
        assert!(!later.newly_claimed);
        assert_eq!(later.record, claims[0].record);
        assert!(transport_claim_id.starts_with("rtc_"));

        let mut conflicting_plan = plan.clone();
        conflicting_plan.spawn_request.payload.prompt = "conflicting prompt".into();
        assert!(runtime
            .claim_admission_transport(&authority, &conflicting_plan, &participant_id, None)
            .is_err());
        let durable = runtime
            .read_admission_record(&authority, "r0-session", &participant_id)
            .unwrap()
            .unwrap();
        assert_eq!(durable, claims[0].record);
    }

    #[test]
    fn transport_claim_rejects_a_post_r0_record_without_a_complete_graph_join() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "claim-complete-join", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let registry_path = parent
            .path()
            .join("home/authority-v1/retained-worker-admission-v1/registry-v1.json");
        let mut registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
            &fs::read(&registry_path).unwrap(),
            "decode fixture registry",
        )
        .unwrap();
        registry
            .records_by_session
            .get_mut(&admitted.record.orchestration_session_id)
            .unwrap()
            .get_mut(&admitted.record.retained_participant_id)
            .unwrap()
            .current_policy_revision = "forged-policy-revision".into();
        let forged = encode_canonical(&registry, "encode fixture registry").unwrap();
        fs::write(&registry_path, &forged).unwrap();

        assert!(runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .is_err());
        assert_eq!(fs::read(registry_path).unwrap(), forged);
    }

    #[test]
    fn post_r0_slot_retry_rejects_substituted_bootstrap_continuity() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let plan = admission_plan(&authority, "slot-complete-join", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let identity = runtime.initialize_admission_registry(&authority).unwrap();
        let (admission_root, key_path) = initialized_admission_paths(&parent, &identity);
        let registry_path = admission_root.join("registry-v1.json");
        let mut registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
            &fs::read(&registry_path).unwrap(),
            "decode fixture registry",
        )
        .unwrap();
        let envelope: RetainedWorkerAdmissionKeyEnvelopeV1 =
            decode_canonical(&fs::read(key_path).unwrap(), "decode fixture key").unwrap();
        let changed_bootstrap_run_id = "rwr_77777777777777777777777777777777";
        let record = registry
            .records_by_session
            .get_mut(&admitted.record.orchestration_session_id)
            .unwrap()
            .get_mut(&admitted.record.retained_participant_id)
            .unwrap();
        record.bootstrap_run_id = changed_bootstrap_run_id.into();
        record.canonical_spawn_fingerprint = canonical_spawn_fingerprint(
            &identity.key_id,
            &envelope.secret_key,
            &plan,
            &record.retained_participant_id,
            changed_bootstrap_run_id,
        )
        .unwrap();
        let forged = encode_canonical(&registry, "encode fixture registry").unwrap();
        fs::write(&registry_path, &forged).unwrap();

        assert!(runtime
            .reserve_admission_slot(&authority, &plan, None)
            .is_err());
        assert_eq!(fs::read(registry_path).unwrap(), forged);
    }

    #[test]
    fn new_slot_reservation_joins_every_existing_post_r0_graph() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let existing_plan = admission_plan(&authority, "existing-complete-join", "prompt", 3);
        let admitted = runtime
            .register_admitted_worker(&authority, &existing_plan, None)
            .unwrap();
        let (registry_path, forged) = forge_admission_bootstrap_identity(
            &parent,
            &authority,
            &existing_plan,
            &admitted.record,
            "rwr_88888888888888888888888888888888",
        );
        let new_plan = admission_plan(&authority, "new-slot-complete-join", "prompt", 3);

        assert!(runtime
            .reserve_admission_slot(&authority, &new_plan, None)
            .is_err());
        assert_eq!(fs::read(registry_path).unwrap(), forged);
    }

    #[test]
    fn new_slot_reservation_joins_an_applied_r0_head_graph() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let existing_plan = admission_plan(&authority, "applied-head-complete-join", "prompt", 3);
        let slot = runtime
            .reserve_admission_slot(&authority, &existing_plan, None)
            .unwrap();
        runtime
            .register_admitted_worker_at(
                &authority,
                &existing_plan,
                Some(AdmissionRegistrationCrashPointV1::AfterR0BeforeAdmissionAdvance),
            )
            .unwrap_err();
        let head = runtime
            .read_admission_record(
                &authority,
                &slot.record.orchestration_session_id,
                &slot.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            head.state,
            RetainedWorkerAdmissionStateV1::AuthorityRegistrationHead { .. }
        ));
        let (registry_path, forged) = forge_admission_bootstrap_identity(
            &parent,
            &authority,
            &existing_plan,
            &head,
            "rwr_99999999999999999999999999999999",
        );
        let new_plan = admission_plan(&authority, "after-applied-head", "prompt", 3);

        assert!(runtime
            .reserve_admission_slot(&authority, &new_plan, None)
            .is_err());
        assert_eq!(fs::read(registry_path).unwrap(), forged);
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

    fn b4_pending_admission_cancel_request(
        record: &RetainedWorkerAdmissionRecordV1,
        cancel_request_id: &str,
    ) -> RetainedWorkerAdmissionCancelRequestV1 {
        RetainedWorkerAdmissionCancelRequestV1 {
            schema_version: 1,
            cancel_request_id: cancel_request_id.to_string(),
            authority_store_id: record.authority_store_id.clone(),
            issuer_request_id: record.issuer_request_id.clone(),
            orchestration_session_id: record.orchestration_session_id.clone(),
            caller_participant_id: "r0-orchestrator".into(),
            retained_participant_id: record.retained_participant_id.clone(),
            bootstrap_run_id: record.bootstrap_run_id.clone(),
            backend_id: record.backend_id.clone(),
            protocol: record.protocol.clone(),
            world_binding: record.world_binding.clone(),
            current_policy_ref: record.current_policy_ref.clone(),
            current_policy_revision: record.current_policy_revision.clone(),
            expected_record_revision: record.record_revision,
            expected_state: record.state.clone(),
        }
    }

    #[test]
    fn b4_pending_admission_slot_cancellation_is_durable_and_prevents_registration() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        runtime.initialize_admission_registry(&authority).unwrap();
        let plan = admission_plan(&authority, "b4-slot-cancel", "prompt", 1);
        let slot = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        let request = b4_pending_admission_cancel_request(&slot.record, "cancel-b4-slot");

        let outcome = runtime
            .cancel_pending_admission(&authority, &request)
            .unwrap();
        assert!(matches!(
            outcome,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelledBeforeTransport {
                ref cancel_request_id,
                ..
            } if cancel_request_id == "cancel-b4-slot"
        ));
        let durable = runtime
            .read_admission_record(
                &authority,
                &slot.record.orchestration_session_id,
                &slot.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            durable.state,
            RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                ref cancel_request_id,
                ..
            } if cancel_request_id == "cancel-b4-slot"
        ));
        assert!(runtime
            .register_admitted_worker(&authority, &plan, None)
            .is_err());
    }

    fn b4_pending_admission_claimed_fixture(
        authority: &HostSessionAuthority,
        issuer: &str,
        capacity: u64,
    ) -> (
        RetainedWorkerAdmissionPlanV1,
        RetainedWorkerTransportClaimV1,
    ) {
        let runtime = RetainedWorkerRuntime;
        runtime.initialize_admission_registry(authority).unwrap();
        let plan = admission_plan(authority, issuer, "prompt", capacity);
        let admitted = runtime
            .register_admitted_worker(authority, &plan, None)
            .unwrap();
        let claim = runtime
            .claim_admission_transport(
                authority,
                &plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        (plan, claim)
    }

    #[test]
    fn b4_pending_admission_pretransport_and_registration_head_cancellation_prevent_routing() {
        for (issuer, applied_r0) in [("b4-head-before-r0", false), ("b4-head-after-r0", true)] {
            let (_parent, authority, _) = started_authority();
            let runtime = RetainedWorkerRuntime;
            runtime.initialize_admission_registry(&authority).unwrap();
            let plan = admission_plan(&authority, issuer, "prompt", 1);
            let slot = runtime
                .reserve_admission_slot(&authority, &plan, None)
                .unwrap();
            runtime
                .register_admitted_worker_at(
                    &authority,
                    &plan,
                    Some(if applied_r0 {
                        AdmissionRegistrationCrashPointV1::AfterR0BeforeAdmissionAdvance
                    } else {
                        AdmissionRegistrationCrashPointV1::WhileRegistrationHead
                    }),
                )
                .unwrap_err();
            let head = runtime
                .read_admission_record(
                    &authority,
                    &slot.record.orchestration_session_id,
                    &slot.record.retained_participant_id,
                )
                .unwrap()
                .unwrap();
            let authority_root_before_cancel = authority.read_a12a_root().unwrap();
            let request = b4_pending_admission_cancel_request(&head, &format!("cancel-{issuer}"));
            let outcome = runtime
                .cancel_pending_admission(&authority, &request)
                .unwrap();
            assert!(matches!(
                outcome,
                RetainedWorkerAdmissionCancelOutcomeV1::CancelledBeforeTransport { .. }
            ));
            let cancelled = outcome.record().clone();
            assert!(!admission_state_is_live(&cancelled.state));
            assert!(!matches!(
                cancelled.state,
                RetainedWorkerAdmissionStateV1::Routable { .. }
            ));
            assert!(runtime
                .claim_admission_transport(
                    &authority,
                    &plan,
                    &cancelled.retained_participant_id,
                    None,
                )
                .is_err());
            assert_eq!(
                authority.read_a12a_root().unwrap(),
                authority_root_before_cancel
            );
            if applied_r0 {
                assert!(admission_registration(&cancelled.state).is_some());
            }
        }

        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        runtime.initialize_admission_registry(&authority).unwrap();
        let plan = admission_plan(&authority, "b4-pretransport", "prompt", 1);
        let admitted = runtime
            .register_admitted_worker(&authority, &plan, None)
            .unwrap();
        let request = b4_pending_admission_cancel_request(&admitted.record, "cancel-pretransport");
        let outcome = runtime
            .cancel_pending_admission(&authority, &request)
            .unwrap();
        assert!(matches!(
            outcome,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelledBeforeTransport { .. }
        ));
        assert!(runtime
            .claim_admission_transport(
                &authority,
                &plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .is_err());
    }

    #[test]
    fn b4_pending_admission_cancelled_head_reconciles_late_r0_without_routing() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        runtime.initialize_admission_registry(&authority).unwrap();
        let plan = admission_plan(&authority, "b4-cancel-before-late-r0", "prompt", 1);
        let slot = runtime
            .reserve_admission_slot(&authority, &plan, None)
            .unwrap();
        runtime
            .register_admitted_worker_at(
                &authority,
                &plan,
                Some(AdmissionRegistrationCrashPointV1::WhileRegistrationHead),
            )
            .unwrap_err();
        let head = runtime
            .read_admission_record(
                &authority,
                &slot.record.orchestration_session_id,
                &slot.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let registration_plan = admission_registration_plan(&head, &plan).unwrap();
        let cancelled = runtime
            .cancel_pending_admission(
                &authority,
                &b4_pending_admission_cancel_request(&head, "cancel-before-late-r0"),
            )
            .unwrap()
            .record()
            .clone();
        assert!(matches!(
            cancelled.state,
            RetainedWorkerAdmissionStateV1::CancelledBeforeRegistration {
                ref cancel_request_id,
                registration_head: Some(_),
                ..
            } if cancel_request_id == "cancel-before-late-r0"
        ));

        let registration = runtime
            .register_retained_target(&authority, &registration_plan)
            .unwrap();
        let resolved_target = runtime
            .resolve_retained_target(&authority, &registration)
            .unwrap();
        let reconciled = runtime
            .advance_registration_head_after_r0(
                &authority,
                &plan,
                &cancelled.retained_participant_id,
                &ResolvedPostR0RegisteredGraphV1 {
                    result: registration,
                    resolved_target,
                },
                None,
                [0xB4; 16],
            )
            .unwrap();
        assert!(matches!(
            reconciled.state,
            RetainedWorkerAdmissionStateV1::CancelledBeforeTransport {
                ref cancel_request_id,
                ..
            } if cancel_request_id == "cancel-before-late-r0"
        ));
        assert!(admission_registration(&reconciled.state).is_some());
        assert!(!admission_state_is_live(&reconciled.state));
        assert!(
            runtime
                .claim_admission_transport(
                    &authority,
                    &plan,
                    &reconciled.retained_participant_id,
                    None,
                )
                .is_err()
        );
    }

    #[test]
    fn b4_pending_admission_claimed_and_interrupted_cancellation_reject_manual_routability() {
        for interrupted in [false, true] {
            let (_parent, authority, _) = started_authority();
            let runtime = RetainedWorkerRuntime;
            let (plan, claim) = b4_pending_admission_claimed_fixture(
                &authority,
                if interrupted {
                    "b4-interrupted"
                } else {
                    "b4-claimed"
                },
                1,
            );
            let eligible = if interrupted {
                runtime
                    .mark_admission_interrupted(
                        &authority,
                        &plan,
                        &claim.record.retained_participant_id,
                        None,
                        None,
                        timestamp("2026-09-07T10:00:00.000000000Z"),
                    )
                    .unwrap()
            } else {
                claim.record.clone()
            };
            let request = b4_pending_admission_cancel_request(
                &eligible,
                if interrupted {
                    "cancel-interrupted"
                } else {
                    "cancel-claimed"
                },
            );
            let outcome = runtime
                .cancel_pending_admission(&authority, &request)
                .unwrap();
            let pending_revision = outcome.record().record_revision;
            assert!(matches!(
                outcome,
                RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout { .. }
            ));
            let (frame, event) =
                registered_runtime_truth(&plan, outcome.record(), "stream-b4-cancelled");
            let routability = runtime
                .mark_admission_routable_outcome(
                    &authority,
                    &plan,
                    &eligible.retained_participant_id,
                    None,
                    &frame,
                    &event,
                    timestamp("2026-09-07T10:00:01.000000000Z"),
                )
                .unwrap();
            assert!(matches!(
                routability.disposition,
                RetainedWorkerAdmissionRoutabilityDispositionV1::CancellationWon { .. }
            ));
            assert!(matches!(
                routability.record.state,
                RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. }
            ));
            assert!(routability.record.record_revision >= pending_revision);
            assert!(!matches!(
                routability.record.state,
                RetainedWorkerAdmissionStateV1::Routable { .. }
            ));
        }
    }

    #[test]
    fn b4_pending_admission_pending_retry_preserves_original_cancel_identity_and_revision() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (_plan, claim) = b4_pending_admission_claimed_fixture(&authority, "b4-idempotent", 1);
        let first_request = b4_pending_admission_cancel_request(&claim.record, "cancel-original");
        let first = runtime
            .cancel_pending_admission(&authority, &first_request)
            .unwrap();
        let first_revision = first.record().record_revision;
        let retry = b4_pending_admission_cancel_request(first.record(), "cancel-replacement");
        let second = runtime
            .cancel_pending_admission(&authority, &retry)
            .unwrap();
        assert_eq!(second.record().record_revision, first_revision);
        assert!(matches!(
            second,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                ..
            } if cancel_request_id == "cancel-original"
        ));
    }

    #[test]
    fn b4_pending_admission_confirmed_delivery_is_suppressed_across_retry_and_restart() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (plan, claim) =
            b4_pending_admission_claimed_fixture(&authority, "b4-delivery-authority", 1);
        runtime
            .observe_admission_transport_start(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &RuntimeFrameIdentityV1 {
                    schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                    stream_id: "stream-b4-delivery-authority".into(),
                    frame_sequence: 1,
                },
                "span-b4-delivery-authority",
            )
            .unwrap();
        let started = runtime
            .read_admission_record(
                &authority,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let first_request = b4_pending_admission_cancel_request(&started, "cancel-first");
        let first = runtime
            .cancel_pending_admission(&authority, &first_request)
            .unwrap();
        let (delivery_claim_id, transport_span_id) = match &first {
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        delivery_claim_id,
                        transport_span_id,
                    },
                ..
            } => (delivery_claim_id.clone(), transport_span_id.clone()),
            other => panic!("first cancellation must claim exact delivery, got {other:?}"),
        };
        assert_eq!(transport_span_id, "span-b4-delivery-authority");
        let confirmed = runtime
            .record_pending_admission_cancel_delivery(
                &authority,
                &RetainedWorkerAdmissionCancelDeliveryCompletionV1 {
                    authority_store_id: started.authority_store_id.clone(),
                    orchestration_session_id: started.orchestration_session_id.clone(),
                    retained_participant_id: started.retained_participant_id.clone(),
                    cancel_request_id: "cancel-first".into(),
                    transport_span_id,
                    delivery_claim_id,
                    result: RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedDelivered,
                },
            )
            .unwrap();
        let confirmed_revision = confirmed.record_revision;
        assert!(matches!(
            confirmed.state,
            RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. }
        ));

        let retry_request = b4_pending_admission_cancel_request(&confirmed, "cancel-retry");
        let retry = runtime
            .cancel_pending_admission(&authority, &retry_request)
            .unwrap();
        assert_eq!(retry.record().record_revision, confirmed_revision);
        assert!(matches!(
            retry,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Confirmed {
                        ref transport_span_id,
                    },
                ..
            } if cancel_request_id == "cancel-first"
                && transport_span_id == "span-b4-delivery-authority"
        ));

        drop(authority);
        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        let durable = runtime
            .read_admission_record(
                &reopened,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let restart_request = b4_pending_admission_cancel_request(&durable, "cancel-after-restart");
        let restart = runtime
            .cancel_pending_admission(&reopened, &restart_request)
            .unwrap();
        assert_eq!(restart.record().record_revision, confirmed_revision);
        assert!(matches!(
            restart,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Confirmed { .. },
                ..
            } if cancel_request_id == "cancel-first"
        ));
    }

    #[test]
    fn b4_pending_admission_abandoned_delivery_claim_is_recoverable_after_reopen() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (plan, claim) =
            b4_pending_admission_claimed_fixture(&authority, "b4-abandoned-delivery", 1);
        runtime
            .observe_admission_transport_start(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &RuntimeFrameIdentityV1 {
                    schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                    stream_id: "stream-b4-abandoned-delivery".into(),
                    frame_sequence: 1,
                },
                "span-b4-abandoned-delivery",
            )
            .unwrap();
        let started = runtime
            .read_admission_record(
                &authority,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let request = b4_pending_admission_cancel_request(&started, "cancel-abandoned");
        let first = runtime
            .cancel_pending_admission_at(
                &authority,
                &request,
                timestamp("2026-09-08T20:00:00.000000000Z"),
                [101_u8; 16],
                [102_u8; 16],
                None,
            )
            .unwrap();
        let first_claim_id = match first {
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        delivery_claim_id,
                        ref transport_span_id,
                    },
                ..
            } if transport_span_id == "span-b4-abandoned-delivery" => delivery_claim_id,
            other => panic!("first attempt must hold the exact delivery claim, got {other:?}"),
        };

        drop(authority);
        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        let durable = runtime
            .read_admission_record(
                &reopened,
                &started.orchestration_session_id,
                &started.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let retry_request = b4_pending_admission_cancel_request(&durable, "cancel-replacement");
        let recovered = runtime
            .cancel_pending_admission_at(
                &reopened,
                &retry_request,
                timestamp("2026-09-08T20:01:01.000000000Z"),
                [103_u8; 16],
                [104_u8; 16],
                None,
            )
            .unwrap();
        assert!(matches!(
            recovered,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        ref delivery_claim_id,
                        ref transport_span_id,
                    },
                ..
            } if cancel_request_id == "cancel-abandoned"
                && delivery_claim_id != &first_claim_id
                && transport_span_id == "span-b4-abandoned-delivery"
        ));
    }

    #[test]
    fn b4_pending_admission_ambiguous_delivery_waits_for_exact_lease_recovery() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (plan, claim) =
            b4_pending_admission_claimed_fixture(&authority, "b4-ambiguous-delivery", 1);
        runtime
            .observe_admission_transport_start(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &RuntimeFrameIdentityV1 {
                    schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                    stream_id: "stream-b4-ambiguous-delivery".into(),
                    frame_sequence: 1,
                },
                "span-b4-ambiguous-delivery",
            )
            .unwrap();
        let started = runtime
            .read_admission_record(
                &authority,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let first = runtime
            .cancel_pending_admission_at(
                &authority,
                &b4_pending_admission_cancel_request(&started, "cancel-ambiguous-original"),
                timestamp("2026-09-08T20:00:00.000000000Z"),
                [111_u8; 16],
                [112_u8; 16],
                None,
            )
            .unwrap();
        let (first_claim_id, transport_span_id) = match &first {
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        delivery_claim_id,
                        transport_span_id,
                    },
                ..
            } if cancel_request_id == "cancel-ambiguous-original" => {
                (delivery_claim_id.clone(), transport_span_id.clone())
            }
            other => panic!("first attempt must own exact delivery, got {other:?}"),
        };
        assert_eq!(transport_span_id, "span-b4-ambiguous-delivery");
        let unchanged_revision = first.record().record_revision;
        let ambiguous = runtime
            .record_pending_admission_cancel_delivery_at(
                &authority,
                &RetainedWorkerAdmissionCancelDeliveryCompletionV1 {
                    authority_store_id: started.authority_store_id.clone(),
                    orchestration_session_id: started.orchestration_session_id.clone(),
                    retained_participant_id: started.retained_participant_id.clone(),
                    cancel_request_id: "cancel-ambiguous-original".into(),
                    transport_span_id: transport_span_id.clone(),
                    delivery_claim_id: first_claim_id.clone(),
                    result: RetainedWorkerAdmissionCancelDeliveryResultV1::Ambiguous,
                },
                timestamp("2026-09-08T20:00:01.000000000Z"),
            )
            .unwrap();
        assert_eq!(ambiguous.record_revision, unchanged_revision);

        let before_expiry = runtime
            .cancel_pending_admission_at(
                &authority,
                &b4_pending_admission_cancel_request(&ambiguous, "cancel-before-expiry"),
                timestamp("2026-09-08T20:00:59.000000000Z"),
                [113_u8; 16],
                [114_u8; 16],
                None,
            )
            .unwrap();
        assert!(matches!(
            before_expiry,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::InProgress {
                        ref delivery_claim_id,
                        ref transport_span_id,
                        ..
                    },
                ..
            } if cancel_request_id == "cancel-ambiguous-original"
                && delivery_claim_id == &first_claim_id
                && transport_span_id == "span-b4-ambiguous-delivery"
        ));

        let expired = runtime.record_pending_admission_cancel_delivery_at(
            &authority,
            &RetainedWorkerAdmissionCancelDeliveryCompletionV1 {
                authority_store_id: started.authority_store_id.clone(),
                orchestration_session_id: started.orchestration_session_id.clone(),
                retained_participant_id: started.retained_participant_id.clone(),
                cancel_request_id: "cancel-ambiguous-original".into(),
                transport_span_id: transport_span_id.clone(),
                delivery_claim_id: first_claim_id.clone(),
                result: RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedDelivered,
            },
            timestamp("2026-09-08T20:01:00.000000000Z"),
        );
        assert!(expired
            .expect_err("completion at claim expiry must be stale")
            .to_string()
            .contains("does not own the live exact claim"));

        let recovered = runtime
            .cancel_pending_admission_at(
                &authority,
                &b4_pending_admission_cancel_request(before_expiry.record(), "cancel-after-expiry"),
                timestamp("2026-09-08T20:01:00.000000000Z"),
                [115_u8; 16],
                [116_u8; 16],
                None,
            )
            .unwrap();
        let replacement_claim_id = match &recovered {
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        delivery_claim_id,
                        transport_span_id,
                    },
                ..
            } if cancel_request_id == "cancel-ambiguous-original"
                && transport_span_id == "span-b4-ambiguous-delivery" =>
            {
                delivery_claim_id.clone()
            }
            other => panic!("expired ambiguity must reacquire exact delivery, got {other:?}"),
        };
        assert_ne!(replacement_claim_id, first_claim_id);

        let stale = runtime.record_pending_admission_cancel_delivery_at(
            &authority,
            &RetainedWorkerAdmissionCancelDeliveryCompletionV1 {
                authority_store_id: started.authority_store_id.clone(),
                orchestration_session_id: started.orchestration_session_id.clone(),
                retained_participant_id: started.retained_participant_id.clone(),
                cancel_request_id: "cancel-ambiguous-original".into(),
                transport_span_id,
                delivery_claim_id: first_claim_id,
                result: RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedDelivered,
            },
            timestamp("2026-09-08T20:01:01.000000000Z"),
        );
        assert!(stale
            .expect_err("late completion must not overwrite replacement claim")
            .to_string()
            .contains("does not own the live exact claim"));

        let still_replaced = runtime
            .cancel_pending_admission_at(
                &authority,
                &b4_pending_admission_cancel_request(
                    recovered.record(),
                    "cancel-after-stale-completion",
                ),
                timestamp("2026-09-08T20:01:01.000000000Z"),
                [117_u8; 16],
                [118_u8; 16],
                None,
            )
            .unwrap();
        assert!(matches!(
            still_replaced,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::InProgress {
                        ref delivery_claim_id,
                        ref transport_span_id,
                        ..
                    },
                ..
            } if delivery_claim_id == &replacement_claim_id
                && transport_span_id == "span-b4-ambiguous-delivery"
        ));
    }

    #[test]
    fn b4_pending_admission_undelivered_transport_releases_exact_claim_for_retry() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (plan, claim) =
            b4_pending_admission_claimed_fixture(&authority, "b4-undelivered-retry", 1);
        runtime
            .observe_admission_transport_start(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &RuntimeFrameIdentityV1 {
                    schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                    stream_id: "stream-b4-undelivered-retry".into(),
                    frame_sequence: 1,
                },
                "span-b4-undelivered-retry",
            )
            .unwrap();
        let started = runtime
            .read_admission_record(
                &authority,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let first = runtime
            .cancel_pending_admission(
                &authority,
                &b4_pending_admission_cancel_request(&started, "cancel-undelivered"),
            )
            .unwrap();
        let (first_claim_id, transport_span_id) = match &first {
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        delivery_claim_id,
                        transport_span_id,
                    },
                ..
            } => (delivery_claim_id.clone(), transport_span_id.clone()),
            other => panic!("first attempt must claim delivery, got {other:?}"),
        };
        let released = runtime
            .record_pending_admission_cancel_delivery(
                &authority,
                &RetainedWorkerAdmissionCancelDeliveryCompletionV1 {
                    authority_store_id: started.authority_store_id.clone(),
                    orchestration_session_id: started.orchestration_session_id.clone(),
                    retained_participant_id: started.retained_participant_id.clone(),
                    cancel_request_id: "cancel-undelivered".into(),
                    transport_span_id,
                    delivery_claim_id: first_claim_id.clone(),
                    result: RetainedWorkerAdmissionCancelDeliveryResultV1::ConfirmedNotDelivered,
                },
            )
            .unwrap();
        assert!(matches!(
            released.state,
            RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending { .. }
        ));
        let retry = runtime
            .cancel_pending_admission(
                &authority,
                &b4_pending_admission_cancel_request(&released, "cancel-replacement"),
            )
            .unwrap();
        assert!(matches!(
            retry,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                delivery_disposition:
                    RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed {
                        ref delivery_claim_id,
                        ref transport_span_id,
                    },
                ..
            } if cancel_request_id == "cancel-undelivered"
                && delivery_claim_id != &first_claim_id
                && transport_span_id == "span-b4-undelivered-retry"
        ));
    }

    #[test]
    fn b4_pending_admission_concurrent_cancellations_authorize_at_most_one_delivery() {
        use std::sync::{Arc, Barrier};

        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (plan, claim) =
            b4_pending_admission_claimed_fixture(&authority, "b4-cancel-contention", 1);
        runtime
            .observe_admission_transport_start(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &RuntimeFrameIdentityV1 {
                    schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                    stream_id: "stream-b4-cancel-contention".into(),
                    frame_sequence: 1,
                },
                "span-b4-cancel-contention",
            )
            .unwrap();
        let started = runtime
            .read_admission_record(
                &authority,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let request_a = b4_pending_admission_cancel_request(&started, "cancel-a");
        let request_b = b4_pending_admission_cancel_request(&started, "cancel-b");
        let home = parent.path().join("home");
        drop(authority);
        let barrier = Arc::new(Barrier::new(2));

        let thread_a = {
            let barrier = Arc::clone(&barrier);
            let home = home.clone();
            std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(&home).unwrap();
                barrier.wait();
                RetainedWorkerRuntime
                    .cancel_pending_admission(&authority, &request_a)
                    .unwrap()
            })
        };
        let thread_b = {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(&home).unwrap();
                barrier.wait();
                RetainedWorkerRuntime
                    .cancel_pending_admission(&authority, &request_b)
                    .unwrap()
            })
        };
        let outcomes = [thread_a.join().unwrap(), thread_b.join().unwrap()];
        let delivery_authorizations = outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome,
                    RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                        delivery_disposition:
                            RetainedWorkerAdmissionCancelDeliveryDispositionV1::Claimed { .. },
                        ..
                    }
                )
            })
            .count();
        assert_eq!(delivery_authorizations, 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                        delivery_disposition:
                            RetainedWorkerAdmissionCancelDeliveryDispositionV1::InProgress { .. },
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            outcomes[0].record().record_revision,
            outcomes[1].record().record_revision
        );
        let cancellation_ids = outcomes.map(|outcome| match outcome {
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                cancel_request_id,
                ..
            } => cancel_request_id,
            other => panic!("expected pending cancellation outcome, got {other:?}"),
        });
        assert_eq!(cancellation_ids[0], cancellation_ids[1]);
        assert!(matches!(
            cancellation_ids[0].as_str(),
            "cancel-a" | "cancel-b"
        ));
    }

    #[test]
    fn b4_pending_admission_transport_closeout_remains_live_and_releases_capacity_once() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (plan, claim) = b4_pending_admission_claimed_fixture(&authority, "b4-closeout", 1);
        let start_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "stream-b4-closeout".into(),
            frame_sequence: 1,
        };
        assert!(runtime
            .observe_admission_transport_start(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &start_frame,
                "span-b4-closeout",
            )
            .unwrap()
            .is_none());
        let started = runtime
            .read_admission_record(
                &authority,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let request = b4_pending_admission_cancel_request(&started, "cancel-closeout");
        let pending = runtime
            .cancel_pending_admission(&authority, &request)
            .unwrap();
        assert!(admission_state_is_live(&pending.record().state));
        assert!(matches!(
            pending,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                transport_span_id: Some(ref span_id),
                ..
            } if cancel_request_id == "cancel-closeout" && span_id == "span-b4-closeout"
        ));
        let blocked = admission_plan(&authority, "b4-closeout-blocked", "prompt", 1);
        assert!(runtime
            .reserve_admission_slot(&authority, &blocked, None)
            .is_err());

        let terminal_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "stream-b4-closeout".into(),
            frame_sequence: 2,
        };
        let terminal_event = RuntimeEventIdentityV1 {
            event_id: "event-b4-closeout-exit".into(),
            event_sequence: 2,
        };
        let terminal_identity = RuntimeTerminalIdentityV1::from(&terminal_event);
        let closed = runtime
            .mark_admission_terminal_for_transport(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                "span-b4-closeout",
                130,
                timestamp("2026-09-07T10:10:00.000000000Z"),
            )
            .unwrap();
        assert!(!admission_state_is_live(&closed.state));
        assert!(matches!(
            closed.state,
            RetainedWorkerAdmissionStateV1::Terminal {
                cancel_request_id: Some(ref cancel_request_id),
                ..
            } if cancel_request_id == "cancel-closeout"
        ));
        let closed_revision = closed.record_revision;
        let replay = runtime
            .mark_admission_terminal_for_transport(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                "span-b4-closeout",
                130,
                timestamp("2026-09-07T10:10:00.000000000Z"),
            )
            .unwrap();
        assert_eq!(replay.record_revision, closed_revision);
        assert!(
            runtime
                .reserve_admission_slot(&authority, &blocked, None)
                .unwrap()
                .record
                .record_revision
                > 0
        );
    }

    #[test]
    fn b4_pending_admission_terminal_truth_prevents_observer_transport_claim() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (plan, claim) =
            b4_pending_admission_claimed_fixture(&authority, "b4-terminal-observer", 1);
        let terminal_frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "stream-b4-terminal-observer".into(),
            frame_sequence: 1,
        };
        let terminal_event = RuntimeEventIdentityV1 {
            event_id: "event-b4-terminal-observer".into(),
            event_sequence: 1,
        };
        let terminal_identity = RuntimeTerminalIdentityV1::from(&terminal_event);
        let terminal = runtime
            .mark_admission_terminal(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &terminal_frame,
                &terminal_event,
                &terminal_identity,
                130,
                timestamp("2026-09-08T21:00:00.000000000Z"),
            )
            .unwrap();

        let late_start = runtime
            .observe_admission_transport_start(
                &authority,
                &plan,
                &claim.record.retained_participant_id,
                None,
                &RuntimeFrameIdentityV1 {
                    schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                    stream_id: terminal_frame.stream_id,
                    frame_sequence: 2,
                },
                "span-b4-late-terminal-observer",
            )
            .unwrap();
        assert!(late_start.is_none());
        assert_eq!(
            runtime
                .read_admission_record(
                    &authority,
                    &terminal.orchestration_session_id,
                    &terminal.retained_participant_id,
                )
                .unwrap()
                .unwrap(),
            terminal
        );
    }

    #[test]
    fn b4_pending_admission_terminal_and_rejected_records_are_revision_stable() {
        let (_parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        runtime.initialize_admission_registry(&authority).unwrap();
        let rejected_plan = admission_plan(&authority, "b4-rejected", "prompt", 2);
        let slot = runtime
            .reserve_admission_slot(&authority, &rejected_plan, None)
            .unwrap();
        let rejected = runtime
            .reconcile_existing_admission(
                &authority,
                &rejected_plan,
                &slot.record,
                None,
                Some(&AdmissionRejectedBeforeRegistrationInputV1 {
                    reason: "policy_denied: test".into(),
                    rejected_at: timestamp("2026-09-07T10:20:00.000000000Z"),
                }),
            )
            .unwrap();
        let request = b4_pending_admission_cancel_request(&rejected, "cancel-rejected");
        let outcome = runtime
            .cancel_pending_admission(&authority, &request)
            .unwrap();
        assert!(matches!(
            outcome,
            RetainedWorkerAdmissionCancelOutcomeV1::RejectedBeforeRegistration { .. }
        ));
        assert_eq!(outcome.record(), &rejected);

        let terminal_plan = admission_plan(&authority, "b4-terminal", "prompt", 2);
        let admitted = runtime
            .register_admitted_worker(&authority, &terminal_plan, None)
            .unwrap();
        let claim = runtime
            .claim_admission_transport(
                &authority,
                &terminal_plan,
                &admitted.record.retained_participant_id,
                None,
            )
            .unwrap();
        let frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "stream-b4-terminal".into(),
            frame_sequence: 1,
        };
        let event = RuntimeEventIdentityV1 {
            event_id: "event-b4-terminal".into(),
            event_sequence: 1,
        };
        let terminal = RuntimeTerminalIdentityV1::from(&event);
        let terminal_record = runtime
            .mark_admission_terminal(
                &authority,
                &terminal_plan,
                &claim.record.retained_participant_id,
                None,
                &frame,
                &event,
                &terminal,
                143,
                timestamp("2026-09-07T10:21:00.000000000Z"),
            )
            .unwrap();
        let request = b4_pending_admission_cancel_request(&terminal_record, "cancel-terminal");
        let outcome = runtime
            .cancel_pending_admission(&authority, &request)
            .unwrap();
        assert!(matches!(
            outcome,
            RetainedWorkerAdmissionCancelOutcomeV1::AlreadyTerminal { .. }
        ));
        assert_eq!(outcome.record(), &terminal_record);
    }

    #[test]
    fn b4_pending_admission_restart_retry_converges_without_duplicate_intent() {
        let (parent, authority, _) = started_authority();
        let runtime = RetainedWorkerRuntime;
        let (_plan, claim) = b4_pending_admission_claimed_fixture(&authority, "b4-restart", 1);
        let request = b4_pending_admission_cancel_request(&claim.record, "cancel-restart");
        runtime
            .cancel_pending_admission_at(
                &authority,
                &request,
                timestamp("2026-09-07T10:30:00.000000000Z"),
                [90_u8; 16],
                [91_u8; 16],
                Some(AdmissionCancellationCrashPointV1::AfterPublicationBeforeResponse),
            )
            .unwrap_err();
        drop(authority);
        let reopened = HostSessionAuthority::open(&parent.path().join("home")).unwrap();
        let durable = runtime
            .read_admission_record(
                &reopened,
                &claim.record.orchestration_session_id,
                &claim.record.retained_participant_id,
            )
            .unwrap()
            .unwrap();
        let revision = durable.record_revision;
        let retry = b4_pending_admission_cancel_request(&durable, "cancel-later-retry");
        let outcome = runtime.cancel_pending_admission(&reopened, &retry).unwrap();
        assert_eq!(outcome.record().record_revision, revision);
        assert!(matches!(
            outcome,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout {
                ref cancel_request_id,
                ..
            } if cancel_request_id == "cancel-restart"
        ));
    }

    #[test]
    fn b4_pending_admission_old_persisted_records_decode_without_reinterpretation() {
        let (_parent, authority, _) = started_authority();
        let (_plan, claim) = b4_pending_admission_claimed_fixture(&authority, "b4-old-decode", 1);
        let bytes = encode_canonical(&claim.record, "encode old admission record").unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let state = value
            .get("state")
            .and_then(serde_json::Value::as_object)
            .unwrap();
        assert!(!state.contains_key("transport_span_id"));
        assert!(!state.contains_key("stream_id"));
        assert!(!state.contains_key("last_frame_sequence"));
        let decoded: RetainedWorkerAdmissionRecordV1 =
            decode_canonical(&bytes, "decode old admission record").unwrap();
        assert_eq!(decoded, claim.record);
        assert_eq!(
            encode_canonical(&decoded, "re-encode old admission record").unwrap(),
            bytes
        );
    }

    #[test]
    fn b4_pending_admission_cancellation_and_routing_contention_has_one_winner() {
        use std::sync::{Arc, Barrier};

        let (parent, authority, _) = started_authority();
        let (plan, claim) = b4_pending_admission_claimed_fixture(&authority, "b4-contention", 1);
        let request = b4_pending_admission_cancel_request(&claim.record, "cancel-contention");
        let (frame, event) = registered_runtime_truth(&plan, &claim.record, "stream-b4-contention");
        let home = parent.path().join("home");
        drop(authority);
        let barrier = Arc::new(Barrier::new(2));
        let cancel_barrier = Arc::clone(&barrier);
        let cancel_home = home.clone();
        let cancel_thread = std::thread::spawn(move || {
            let authority = HostSessionAuthority::open(&cancel_home).unwrap();
            cancel_barrier.wait();
            RetainedWorkerRuntime
                .cancel_pending_admission(&authority, &request)
                .unwrap()
        });
        let route_barrier = Arc::clone(&barrier);
        let route_thread = std::thread::spawn(move || {
            let authority = HostSessionAuthority::open(&home).unwrap();
            route_barrier.wait();
            RetainedWorkerRuntime
                .mark_admission_routable_outcome(
                    &authority,
                    &plan,
                    &claim.record.retained_participant_id,
                    None,
                    &frame,
                    &event,
                    timestamp("2026-09-07T10:40:00.000000000Z"),
                )
                .unwrap()
        });
        let cancel = cancel_thread.join().unwrap();
        let route = route_thread.join().unwrap();
        let cancellation_won = matches!(
            cancel,
            RetainedWorkerAdmissionCancelOutcomeV1::CancelAcceptedPendingCloseout { .. }
        );
        let routing_won = matches!(
            route.disposition,
            RetainedWorkerAdmissionRoutabilityDispositionV1::Routable
        );
        assert_ne!(cancellation_won, routing_won);
        if cancellation_won {
            assert!(matches!(
                route.disposition,
                RetainedWorkerAdmissionRoutabilityDispositionV1::CancellationWon { .. }
            ));
        } else {
            assert!(matches!(
                cancel,
                RetainedWorkerAdmissionCancelOutcomeV1::AlreadyRoutable { .. }
            ));
        }
    }
}
