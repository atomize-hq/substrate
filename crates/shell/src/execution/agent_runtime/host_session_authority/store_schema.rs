use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use substrate_common::HostTransitionWorkCorrelationV1;

use super::super::state_store::AcceptedWorldWorkIdentityV1;
use super::schema::{
    AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, AuthorityObjectKindV1,
    AuthorityObjectRefV1, CanonicalDirectoryV1, DurableSessionAuthorityHashInputV1,
    DurableSessionAuthorityOriginV1, HostPostTurnDispositionV1, HostSessionAuthorityPreconditionV1,
    HostSessionPostureV1, HostSessionStopPayloadHashInputV1, HostSessionStopResultHashInputV1,
    HostSessionTransitionCallerV1, HostSessionTransitionModeV1,
    HostSessionTransitionTerminalRejectionV1, TimestampV1, WorkspaceBindingV1, WorldBindingV1,
};
use super::store_format::{validate_key_id, validate_ref_id, validate_store_id};
use super::validation::validate_object_commitment_rule;

const SCHEMA_VERSION: u32 = 1;

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthorityStoreCommitmentKeyV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) key_id: String,
    pub(crate) algorithm: AuthorityStoreCommitmentAlgorithmV1,
    pub(crate) created_at: TimestampV1,
    pub(crate) state: AuthorityStoreCommitmentKeyStateV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthorityStoreInitializationV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) initial_key_id: String,
    pub(crate) created_at: TimestampV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum AuthorityStoreCommitmentAlgorithmV1 {
    HmacSha256,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub(crate) enum AuthorityStoreCommitmentKeyStateV1 {
    Active,
    VerificationOnly,
    Retired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StateRootV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) root_revision: u64,
    pub(crate) active_commitment_key_id: String,
    pub(crate) commitment_key_registry: BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    pub(crate) greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1,
    pub(crate) session_namespace_map: BTreeMap<String, SessionNamespaceRecordV1>,
    pub(crate) transition_intent_map: BTreeMap<String, HostSessionTransitionIntentV1>,
    pub(crate) issuer_request_index: BTreeMap<String, IssuerRequestIndexEntryV1>,
    pub(crate) application_journal: BTreeMap<String, HostSessionTransitionApplicationJournalV1>,
    pub(crate) object_index: BTreeMap<String, AuthorityObjectIndexEntryV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StateRootV2 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) root_revision: u64,
    pub(crate) active_commitment_key_id: String,
    pub(crate) commitment_key_registry: BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    pub(crate) greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1,
    pub(crate) session_namespace_map: BTreeMap<String, SessionNamespaceRecordV1>,
    pub(crate) transition_intent_map: BTreeMap<String, HostSessionTransitionIntentV2>,
    pub(crate) issuer_request_index: BTreeMap<String, IssuerRequestIndexEntryV1>,
    pub(crate) application_journal: BTreeMap<String, HostSessionTransitionApplicationJournalV2>,
    pub(crate) retained_worker_registration_request_index:
        BTreeMap<String, RetainedWorkerAuthorityRegistrationRequestV1>,
    pub(crate) retained_worker_registration_journal:
        BTreeMap<String, RetainedWorkerAuthorityRegistrationV1>,
    pub(crate) object_index: BTreeMap<String, AuthorityObjectIndexEntryV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StateRootV3 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) root_revision: u64,
    pub(crate) active_commitment_key_id: String,
    pub(crate) commitment_key_registry: BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    pub(crate) greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1,
    pub(crate) session_namespace_map: BTreeMap<String, SessionNamespaceRecordV1>,
    pub(crate) transition_intent_map: BTreeMap<String, HostSessionTransitionIntentV2>,
    pub(crate) issuer_request_index: BTreeMap<String, IssuerRequestIndexEntryV1>,
    pub(crate) application_journal: BTreeMap<String, HostSessionTransitionApplicationJournalV2>,
    pub(crate) retained_worker_registration_request_index:
        BTreeMap<String, RetainedWorkerAuthorityRegistrationRequestV1>,
    pub(crate) retained_worker_registration_journal:
        BTreeMap<String, RetainedWorkerAuthorityRegistrationV1>,
    pub(crate) successor_transition_intent_map: BTreeMap<String, HostSessionTransitionIntentV3>,
    pub(crate) successor_issuer_request_index: BTreeMap<String, IssuerRequestIndexEntryV1>,
    pub(crate) successor_application_journal:
        BTreeMap<String, HostSessionTransitionApplicationJournalV3>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) start_transaction_map: BTreeMap<String, StartTransactionRecordV1>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) stop_transaction_map: BTreeMap<String, HostSessionStopIntentV1>,
    pub(crate) object_index: BTreeMap<String, AuthorityObjectIndexEntryV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum StartTransactionStateV1 {
    PromptNotSubmitted,
    PromptSubmissionNoReplayBarrier {
        barrier_committed_at: TimestampV1,
    },
    PromptSubmissionIndeterminate {
        barrier_committed_at: TimestampV1,
        declared_at: TimestampV1,
    },
    ContinuationRegistered {
        registration_ref: AuthorityObjectRefV1,
        authority_revision_after: u64,
        registered_at: TimestampV1,
    },
    TurnSettledAwaitingResponse {
        settlement_ref: AuthorityObjectRefV1,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        completed_at: TimestampV1,
    },
    PublicResponseDelivered {
        settlement_ref: AuthorityObjectRefV1,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        completed_at: TimestampV1,
        delivered_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartTransactionRecordV1 {
    pub(crate) schema_version: u32,
    pub(crate) transaction_id: String,
    pub(crate) request_key_sha256: String,
    pub(crate) prompt_sha256: String,
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) authoritative_participant_id: String,
    pub(crate) backend_id: String,
    pub(crate) protocol: String,
    pub(crate) workspace_root: String,
    pub(crate) world_id: Option<String>,
    pub(crate) world_generation: Option<u64>,
    pub(crate) public_backend_id: String,
    pub(crate) public_scope: String,
    pub(crate) start_intent_id: String,
    pub(crate) start_issuer_request_id: String,
    pub(crate) start_payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) start_application_result_ref: AuthorityObjectRefV1,
    pub(crate) start_run_id: String,
    pub(crate) start_authority_revision: u64,
    pub(crate) created_at: TimestampV1,
    pub(crate) updated_at: TimestampV1,
    pub(crate) state: StartTransactionStateV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionStopIntentStateV1 {
    Issued,
    DeliveryAccepted {
        acceptance_id: String,
        accepted_by_participant_id: String,
        accepted_at: TimestampV1,
    },
    Completed {
        delivery_acceptance_id: Option<String>,
        delivery_accepted_by_participant_id: Option<String>,
        delivery_accepted_at: Option<TimestampV1>,
        result_id: String,
        result_commitment: AuthorityObjectCommitmentV1,
        authority_revision_after: u64,
        authority_record_commitment_after: AuthorityObjectCommitmentV1,
        completed_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionStopIntentV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) request_id: String,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) expected_root_revision: u64,
    pub(crate) authority_before: Box<DurableSessionAuthorityV1>,
    pub(crate) authority_record_commitment_before: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_lineage_commitment_before: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_participant_id: String,
    pub(crate) issued_at: TimestampV1,
    pub(crate) updated_at: TimestampV1,
    pub(crate) state: HostSessionStopIntentStateV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VersionedStateRoot {
    V1(StateRootV1),
    V2(StateRootV2),
    V3(StateRootV3),
}

impl VersionedStateRoot {
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, StoreSchemaError> {
        let syntax: serde_json::Value = super::canonical_json::from_slice(bytes)
            .map_err(|_| StoreSchemaError("invalid canonical authority root"))?;
        let version = syntax
            .as_object()
            .and_then(|object| object.get("schema_version"))
            .and_then(serde_json::Value::as_u64)
            .ok_or(StoreSchemaError(
                "authority root schema discriminator is missing or invalid",
            ))?;
        match version {
            1 => super::canonical_json::from_slice(bytes)
                .map(Self::V1)
                .map_err(|_| StoreSchemaError("invalid strict StateRootV1")),
            2 => super::canonical_json::from_slice(bytes)
                .map(Self::V2)
                .map_err(|_| StoreSchemaError("invalid strict StateRootV2")),
            3 => super::canonical_json::from_slice(bytes)
                .map(Self::V3)
                .map_err(|_| StoreSchemaError("invalid strict StateRootV3")),
            _ => Err(StoreSchemaError(
                "unsupported authority root schema version",
            )),
        }
    }

    pub(crate) fn to_canonical_bytes(
        &self,
    ) -> Result<Vec<u8>, super::canonical_json::CanonicalJsonError> {
        match self {
            Self::V1(root) => super::canonical_json::to_vec(root),
            Self::V2(root) => super::canonical_json::to_vec(root),
            Self::V3(root) => super::canonical_json::to_vec(root),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), StoreSchemaError> {
        match self {
            Self::V1(root) => root.validate(),
            Self::V2(root) => root.validate(),
            Self::V3(root) => root.validate(),
        }
    }

    pub(crate) fn root_revision(&self) -> u64 {
        match self {
            Self::V1(root) => root.root_revision,
            Self::V2(root) => root.root_revision,
            Self::V3(root) => root.root_revision,
        }
    }

    pub(crate) fn authority_store_id(&self) -> &str {
        match self {
            Self::V1(root) => &root.authority_store_id,
            Self::V2(root) => &root.authority_store_id,
            Self::V3(root) => &root.authority_store_id,
        }
    }

    pub(crate) fn bootstrap_home(&self) -> &CanonicalDirectoryV1 {
        match self {
            Self::V1(root) => &root.bootstrap_home,
            Self::V2(root) => &root.bootstrap_home,
            Self::V3(root) => &root.bootstrap_home,
        }
    }

    pub(crate) fn greenfield_namespace_certificate(&self) -> &GreenfieldNamespaceCertificateV1 {
        match self {
            Self::V1(root) => &root.greenfield_namespace_certificate,
            Self::V2(root) => &root.greenfield_namespace_certificate,
            Self::V3(root) => &root.greenfield_namespace_certificate,
        }
    }

    pub(crate) fn active_commitment_key_id(&self) -> &str {
        match self {
            Self::V1(root) => &root.active_commitment_key_id,
            Self::V2(root) => &root.active_commitment_key_id,
            Self::V3(root) => &root.active_commitment_key_id,
        }
    }

    pub(crate) fn active_commitment_key_id_mut(&mut self) -> &mut String {
        match self {
            Self::V1(root) => &mut root.active_commitment_key_id,
            Self::V2(root) => &mut root.active_commitment_key_id,
            Self::V3(root) => &mut root.active_commitment_key_id,
        }
    }

    pub(crate) fn commitment_key_registry(
        &self,
    ) -> &BTreeMap<String, AuthorityStoreCommitmentKeyV1> {
        match self {
            Self::V1(root) => &root.commitment_key_registry,
            Self::V2(root) => &root.commitment_key_registry,
            Self::V3(root) => &root.commitment_key_registry,
        }
    }

    pub(crate) fn commitment_key_registry_mut(
        &mut self,
    ) -> &mut BTreeMap<String, AuthorityStoreCommitmentKeyV1> {
        match self {
            Self::V1(root) => &mut root.commitment_key_registry,
            Self::V2(root) => &mut root.commitment_key_registry,
            Self::V3(root) => &mut root.commitment_key_registry,
        }
    }

    pub(crate) fn set_root_revision(&mut self, revision: u64) {
        match self {
            Self::V1(root) => root.root_revision = revision,
            Self::V2(root) => root.root_revision = revision,
            Self::V3(root) => root.root_revision = revision,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GreenfieldNamespaceCertificateV1 {
    pub(crate) schema_version: u32,
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) certified_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum SessionNamespaceRecordV1 {
    Authority(Box<DurableSessionAuthorityV1>),
    StartReservation(SessionIdReservationV1),
    StartTombstone(SessionIdTombstoneV1),
}

impl SessionNamespaceRecordV1 {
    fn orchestration_session_id(&self) -> &str {
        match self {
            Self::Authority(value) => &value.orchestration_session_id,
            Self::StartReservation(value) => &value.orchestration_session_id,
            Self::StartTombstone(value) => &value.orchestration_session_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DurableSessionAuthorityV1 {
    pub(crate) schema_version: u32,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) authority_revision: u64,
    pub(crate) origin: DurableSessionAuthorityOriginV1,
    pub(crate) authoritative_participant_lineage: Vec<String>,
    pub(crate) active_authoritative_participant_id: Option<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) host_attach_contract_ref: Option<AuthorityObjectRefV1>,
    pub(crate) retained_worker_refs: Vec<AuthorityObjectRefV1>,
    pub(crate) internal_resume_handle_refs: Vec<AuthorityObjectRefV1>,
    pub(crate) lifecycle_posture: HostSessionPostureV1,
    pub(crate) current_policy_ref: Option<AuthorityObjectRefV1>,
    pub(crate) current_policy_revision: Option<String>,
    pub(crate) updated_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionIdReservationV1 {
    pub(crate) schema_version: u32,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) reserved_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum StartTombstoneStateV1 {
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
    },
    Expired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionIdTombstoneV1 {
    pub(crate) schema_version: u32,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) terminal_state: StartTombstoneStateV1,
    pub(crate) terminal_handoff_ref: AuthorityObjectRefV1,
    pub(crate) tombstoned_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IssuerRequestIndexEntryV1 {
    pub(crate) schema_version: u32,
    pub(crate) issuer_request_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) intent_id: String,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InitialTransitionApplicationJournalV1 {
    pub(crate) authority_revision_before: Option<u64>,
    pub(crate) authority_revision_after: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) application_result_ref: AuthorityObjectRefV1,
    pub(crate) applied_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PostTurnApplicationJournalV1 {
    pub(crate) completion_ref: AuthorityObjectRefV1,
    pub(crate) authority_revision_before: u64,
    pub(crate) authority_revision_after: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) application_result_ref: AuthorityObjectRefV1,
    pub(crate) applied_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionApplicationJournalV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) initial_application: InitialTransitionApplicationJournalV1,
    pub(crate) post_turn_application: Option<PostTurnApplicationJournalV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionApplicationJournalV2 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) initial_application: InitialTransitionApplicationJournalV1,
    pub(crate) startup_terminal_application: Option<StartupOwnershipTerminalApplicationJournalV1>,
    pub(crate) post_turn_application: Option<PostTurnApplicationJournalV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PostTurnApplicationJournalV2 {
    pub(crate) completion_ref: AuthorityObjectRefV1,
    pub(crate) obligation_snapshot_ref: Option<AuthorityObjectRefV1>,
    pub(crate) authority_revision_before: u64,
    pub(crate) authority_revision_after: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) application_result_ref: AuthorityObjectRefV1,
    pub(crate) applied_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionApplicationJournalV3 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) initial_application: InitialTransitionApplicationJournalV1,
    pub(crate) startup_terminal_application: Option<StartupOwnershipTerminalApplicationJournalV1>,
    pub(crate) post_turn_application: Option<PostTurnApplicationJournalV2>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartupOwnershipTerminalApplicationJournalV1 {
    pub(crate) schema_version: u32,
    pub(crate) startup_ownership_result_ref: AuthorityObjectRefV1,
    pub(crate) evidence_id: String,
    pub(crate) authority_revision_before: u64,
    pub(crate) authority_record_commitment_before: AuthorityObjectCommitmentV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) resulting_posture: HostSessionPostureV1,
    pub(crate) authority_record_commitment_after: AuthorityObjectCommitmentV1,
    pub(crate) applied_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum AuthorityObjectStorageStateV1 {
    Present,
    ReleaseEligible {
        terminal_handoff_ref: AuthorityObjectRefV1,
    },
    Released {
        terminal_handoff_ref: AuthorityObjectRefV1,
        released_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthorityObjectIndexEntryV1 {
    pub(crate) schema_version: u32,
    pub(crate) ref_id: String,
    pub(crate) object_kind: AuthorityObjectKindV1,
    pub(crate) object_schema_version: u32,
    pub(crate) byte_length: u64,
    pub(crate) storage_state: AuthorityObjectStorageStateV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionIntentV1 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) intent_revision: u64,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) authority_precondition: HostSessionAuthorityPreconditionV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) source_authoritative_participant_id: Option<String>,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) target_participant_lease_token_ref: AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) host_attach_contract_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: Option<AuthorityObjectRefV1>,
    pub(crate) transition_input_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_disposition: Option<HostPostTurnDispositionV1>,
    pub(crate) transport_payload_ref: AuthorityObjectRefV1,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) issued_at: TimestampV1,
    pub(crate) expires_at: TimestampV1,
    pub(crate) state: HostSessionTransitionIntentStateV1,
    pub(crate) input_handoff: HostSessionTransitionInputHandoffV1,
    pub(crate) transport_payload_state: HostSessionTransitionTransportPayloadStateV1,
    pub(crate) updated_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionIntentStateV1 {
    Issued,
    Claimed {
        claim_id: String,
        claimant_attempt_id: String,
        claim_revision: u64,
        claimed_at: TimestampV1,
        claim_expires_at: TimestampV1,
    },
    Applied {
        claim_id: String,
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        application_result_ref: AuthorityObjectRefV1,
        post_turn: Box<HostSessionPostTurnApplicationV1>,
        applied_at: TimestampV1,
    },
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
        terminal_handoff_ref: AuthorityObjectRefV1,
        rejected_at: TimestampV1,
    },
    Expired {
        terminal_handoff_ref: AuthorityObjectRefV1,
        expired_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionIntentV2 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) intent_revision: u64,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) authority_precondition: HostSessionAuthorityPreconditionV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) source_authoritative_participant_id: Option<String>,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) target_participant_lease_token_ref: AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) host_attach_contract_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: Option<AuthorityObjectRefV1>,
    pub(crate) transition_input_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_disposition: Option<HostPostTurnDispositionV1>,
    pub(crate) transport_payload_ref: AuthorityObjectRefV1,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) issued_at: TimestampV1,
    pub(crate) expires_at: TimestampV1,
    pub(crate) state: HostSessionTransitionIntentStateV2,
    pub(crate) input_handoff: HostSessionTransitionInputHandoffV1,
    pub(crate) transport_payload_state: HostSessionTransitionTransportPayloadStateV1,
    pub(crate) updated_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostSessionTransitionIntentV3 {
    pub(crate) schema_version: u32,
    pub(crate) intent_id: String,
    pub(crate) issuer_request_id: String,
    pub(crate) intent_revision: u64,
    pub(crate) mode: HostSessionTransitionModeV1,
    pub(crate) authority_precondition: HostSessionAuthorityPreconditionV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) shell_trace_session_id: String,
    pub(crate) caller: HostSessionTransitionCallerV1,
    pub(crate) source_authoritative_participant_id: Option<String>,
    pub(crate) target_authoritative_participant_id: String,
    pub(crate) target_participant_lease_token_ref: AuthorityObjectRefV1,
    pub(crate) run_id: String,
    pub(crate) resulting_authoritative_lineage: Vec<String>,
    pub(crate) workspace_binding: WorkspaceBindingV1,
    pub(crate) world_binding: Option<WorldBindingV1>,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) host_attach_contract_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: Option<AuthorityObjectRefV1>,
    pub(crate) transition_input_ref: Option<AuthorityObjectRefV1>,
    pub(crate) post_turn_disposition: Option<HostPostTurnDispositionV1>,
    pub(crate) transport_payload_ref: AuthorityObjectRefV1,
    pub(crate) payload_commitment: AuthorityObjectCommitmentV1,
    pub(crate) issued_at: TimestampV1,
    pub(crate) expires_at: TimestampV1,
    pub(crate) state: HostSessionTransitionIntentStateV3,
    pub(crate) input_handoff: HostSessionTransitionInputHandoffV1,
    pub(crate) transport_payload_state: HostSessionTransitionTransportPayloadStateV1,
    pub(crate) updated_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionIntentStateV2 {
    Issued,
    Claimed {
        claim_id: String,
        claimant_attempt_id: String,
        claim_revision: u64,
        claimed_at: TimestampV1,
        claim_expires_at: TimestampV1,
    },
    Applied {
        claim_id: String,
        claimant_attempt_id: String,
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        application_result_ref: AuthorityObjectRefV1,
        startup_ownership: Box<HostSessionStartupOwnershipApplicationV1>,
        post_turn: Box<HostSessionPostTurnApplicationV1>,
        applied_at: TimestampV1,
    },
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
        terminal_handoff_ref: AuthorityObjectRefV1,
        rejected_at: TimestampV1,
    },
    Expired {
        terminal_handoff_ref: AuthorityObjectRefV1,
        expired_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionIntentStateV3 {
    Issued,
    Claimed {
        claim_id: String,
        claimant_attempt_id: String,
        claim_revision: u64,
        claimed_at: TimestampV1,
        claim_expires_at: TimestampV1,
    },
    Applied {
        claim_id: String,
        claimant_attempt_id: String,
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        application_result_ref: AuthorityObjectRefV1,
        startup_ownership: Box<HostSessionStartupOwnershipApplicationV1>,
        post_turn: Box<HostSessionPostTurnApplicationV2>,
        applied_at: TimestampV1,
    },
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
        terminal_handoff_ref: AuthorityObjectRefV1,
        rejected_at: TimestampV1,
    },
    Expired {
        terminal_handoff_ref: AuthorityObjectRefV1,
        expired_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionStartupOwnershipApplicationV1 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
        expected_active_authoritative_participant_id: String,
    },
    Accepted {
        evidence_id: String,
        result_ref: AuthorityObjectRefV1,
        authority_revision: u64,
        accepted_at: TimestampV1,
    },
    TerminalReconciled {
        evidence_id: String,
        result_ref: AuthorityObjectRefV1,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        reconciled_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum RetainedWorkerAuthorityRegistrationRequestStateV1 {
    Reserved,
    Applied {
        authority_revision_after: u64,
        authority_record_commitment_after: AuthorityObjectCommitmentV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAuthorityRegistrationRequestV1 {
    pub(crate) schema_version: u32,
    pub(crate) issuer_request_id: String,
    pub(crate) registration_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authority_revision_before: u64,
    pub(crate) authority_record_commitment_before: AuthorityObjectCommitmentV1,
    pub(crate) retained_participant_id: String,
    pub(crate) descriptor_ref_id: String,
    pub(crate) descriptor_commitment: AuthorityObjectCommitmentV1,
    pub(crate) resume_handle_ref_id: String,
    pub(crate) resume_handle_commitment: AuthorityObjectCommitmentV1,
    pub(crate) retained_worker_ref_id: String,
    pub(crate) retained_worker_commitment: AuthorityObjectCommitmentV1,
    pub(crate) current_policy_ref: AuthorityObjectRefV1,
    pub(crate) world_binding: WorldBindingV1,
    pub(crate) registered_at: TimestampV1,
    pub(crate) state: RetainedWorkerAuthorityRegistrationRequestStateV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedWorkerAuthorityRegistrationV1 {
    pub(crate) schema_version: u32,
    pub(crate) issuer_request_id: String,
    pub(crate) registration_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) authority_revision_before: u64,
    pub(crate) authority_record_commitment_before: AuthorityObjectCommitmentV1,
    pub(crate) authority_revision_after: u64,
    pub(crate) authority_record_commitment_after: AuthorityObjectCommitmentV1,
    pub(crate) retained_participant_id: String,
    pub(crate) authoritative_lineage_commitment_after: AuthorityObjectCommitmentV1,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: AuthorityObjectRefV1,
    pub(crate) retained_worker_ref: AuthorityObjectRefV1,
    pub(crate) current_policy_ref: AuthorityObjectRefV1,
    pub(crate) world_binding: WorldBindingV1,
    pub(crate) registered_at: TimestampV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionPostTurnApplicationV1 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
    },
    Applied {
        completion_ref: Box<AuthorityObjectRefV1>,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        application_result_ref: Box<AuthorityObjectRefV1>,
        applied_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionPostTurnApplicationV2 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
    },
    AwaitingObligationCut {
        completion_ref: Box<AuthorityObjectRefV1>,
        expected_run_id: String,
        expected_authority_revision: u64,
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        stream_id: String,
        accepted_work_identity: AcceptedWorldWorkIdentityV1,
        host_transition_correlation: Box<HostTransitionWorkCorrelationV1>,
        required_terminal_event_id: String,
        required_terminal_event_sequence: u64,
        recorded_at: TimestampV1,
    },
    Applied {
        completion_ref: Box<AuthorityObjectRefV1>,
        obligation_snapshot_ref: Option<Box<AuthorityObjectRefV1>>,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        application_result_ref: Box<AuthorityObjectRefV1>,
        applied_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionInputHandoffV1 {
    NotApplicable,
    Pending {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
    },
    Accepted {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
        acceptance_ref: AuthorityObjectRefV1,
        accepted_at: TimestampV1,
    },
    TerminalWithoutAcceptance {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
        terminal_handoff_ref: AuthorityObjectRefV1,
        terminal_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub(crate) enum HostSessionTransitionTransportPayloadStateV1 {
    Retained,
    ReleaseEligible {
        terminal_handoff_ref: AuthorityObjectRefV1,
    },
    Released {
        terminal_handoff_ref: AuthorityObjectRefV1,
        released_at: TimestampV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoreSchemaError(&'static str);

impl fmt::Display for StoreSchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for StoreSchemaError {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReconstructedAuthorityStateV1 {
    authority: DurableSessionAuthorityV1,
    authority_record_commitment: AuthorityObjectCommitmentV1,
    authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
}

impl AuthorityStoreInitializationV1 {
    pub(crate) fn validate(&self) -> Result<(), StoreSchemaError> {
        require_version(self.schema_version)?;
        validate_store_id(&self.authority_store_id)
            .map_err(|_| StoreSchemaError("invalid initialization store ID"))?;
        validate_key_id(&self.initial_key_id)
            .map_err(|_| StoreSchemaError("invalid initialization key ID"))?;
        required(&self.bootstrap_home.physical_path)
    }
}

impl StateRootV2 {
    pub(crate) fn try_from_greenfield_v1(root: &StateRootV1) -> Result<Self, StoreSchemaError> {
        if !root.session_namespace_map.is_empty()
            || !root.transition_intent_map.is_empty()
            || !root.issuer_request_index.is_empty()
            || !root.application_journal.is_empty()
            || !root.object_index.is_empty()
        {
            return Err(StoreSchemaError("UnsupportedNonGreenfieldRootV1"));
        }
        root.validate()?;
        let upgraded = Self {
            schema_version: 2,
            authority_store_id: root.authority_store_id.clone(),
            bootstrap_home: root.bootstrap_home.clone(),
            root_revision: root
                .root_revision
                .checked_add(1)
                .ok_or(StoreSchemaError("authority root revision overflow"))?,
            active_commitment_key_id: root.active_commitment_key_id.clone(),
            commitment_key_registry: root.commitment_key_registry.clone(),
            greenfield_namespace_certificate: root.greenfield_namespace_certificate.clone(),
            session_namespace_map: BTreeMap::new(),
            transition_intent_map: BTreeMap::new(),
            issuer_request_index: BTreeMap::new(),
            application_journal: BTreeMap::new(),
            retained_worker_registration_request_index: BTreeMap::new(),
            retained_worker_registration_journal: BTreeMap::new(),
            object_index: BTreeMap::new(),
        };
        upgraded.validate_greenfield()?;
        Ok(upgraded)
    }

    pub(crate) fn validate_greenfield(&self) -> Result<(), StoreSchemaError> {
        self.validate()?;
        if !self.session_namespace_map.is_empty()
            || !self.transition_intent_map.is_empty()
            || !self.issuer_request_index.is_empty()
            || !self.application_journal.is_empty()
            || !self.retained_worker_registration_request_index.is_empty()
            || !self.retained_worker_registration_journal.is_empty()
            || !self.object_index.is_empty()
        {
            return Err(StoreSchemaError(
                "A1.2a-1 StateRootV2 must remain greenfield-empty",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<(), StoreSchemaError> {
        if self.schema_version != 2 {
            return Err(StoreSchemaError("StateRootV2 requires schema version 2"));
        }
        validate_store_id(&self.authority_store_id)
            .map_err(|_| StoreSchemaError("invalid authority store ID"))?;
        required(&self.bootstrap_home.physical_path)?;
        if self.root_revision < 2 {
            return Err(StoreSchemaError("V2 root revision must be at least two"));
        }
        if self.greenfield_namespace_certificate.schema_version != 1
            || self.greenfield_namespace_certificate.authority_store_id != self.authority_store_id
            || self.greenfield_namespace_certificate.bootstrap_home != self.bootstrap_home
        {
            return Err(StoreSchemaError(
                "greenfield certificate does not match V2 root identity",
            ));
        }
        let mut active_count = 0_usize;
        for (key, value) in &self.commitment_key_registry {
            require_version(value.schema_version)?;
            validate_key_id(&value.key_id)
                .map_err(|_| StoreSchemaError("invalid commitment key ID"))?;
            if key != &value.key_id || value.authority_store_id != self.authority_store_id {
                return Err(StoreSchemaError("commitment key registry entry mismatch"));
            }
            if value.state == AuthorityStoreCommitmentKeyStateV1::Active {
                active_count += 1;
                if value.key_id != self.active_commitment_key_id {
                    return Err(StoreSchemaError("active key ID does not match root"));
                }
            }
        }
        if active_count != 1
            || self
                .commitment_key_registry
                .get(&self.active_commitment_key_id)
                .map(|key| key.state)
                != Some(AuthorityStoreCommitmentKeyStateV1::Active)
        {
            return Err(StoreSchemaError("root must contain exactly one active key"));
        }
        if !self
            .commitment_key_registry
            .values()
            .any(|key| key.created_at == self.greenfield_namespace_certificate.certified_at)
        {
            return Err(StoreSchemaError(
                "greenfield certificate timestamp has no initial key",
            ));
        }
        for (key, record) in &self.session_namespace_map {
            if key != record.orchestration_session_id() {
                return Err(StoreSchemaError("V2 session namespace map key mismatch"));
            }
            validate_namespace_record_identity(
                record,
                &self.authority_store_id,
                &self.bootstrap_home,
                false,
            )?;
        }
        for (key, intent) in &self.transition_intent_map {
            if key != &intent.intent_id || intent.schema_version != 2 {
                return Err(StoreSchemaError("V2 transition intent map entry mismatch"));
            }
            required(&intent.intent_id)?;
            required(&intent.issuer_request_id)?;
            required(&intent.orchestration_session_id)?;
            required(&intent.shell_trace_session_id)?;
            required(&intent.target_authoritative_participant_id)?;
            required(&intent.run_id)?;
            if intent.intent_revision == 0
                || intent.workspace_binding.authority_store_id != self.authority_store_id
                || intent.workspace_binding.authority_store_root != self.bootstrap_home
                || intent.issued_at.as_str() >= intent.expires_at.as_str()
            {
                return Err(StoreSchemaError("V2 transition intent binding mismatch"));
            }
            self.validate_start_intent_relations(intent)?;
        }
        for (key, entry) in &self.issuer_request_index {
            if key != &entry.issuer_request_id {
                return Err(StoreSchemaError("V2 issuer request index key mismatch"));
            }
            require_version(entry.schema_version)?;
            let intent = self
                .transition_intent_map
                .get(&entry.intent_id)
                .ok_or(StoreSchemaError("V2 issuer request references no intent"))?;
            if intent.issuer_request_id != entry.issuer_request_id
                || intent.orchestration_session_id != entry.orchestration_session_id
                || intent.payload_commitment != entry.payload_commitment
            {
                return Err(StoreSchemaError("V2 issuer request and intent disagree"));
            }
        }
        for (key, journal) in &self.application_journal {
            if key != &journal.intent_id || journal.schema_version != 2 {
                return Err(StoreSchemaError("V2 application journal entry mismatch"));
            }
            if !self.transition_intent_map.contains_key(&journal.intent_id) {
                return Err(StoreSchemaError(
                    "V2 application journal references no intent",
                ));
            }
        }
        if self.issuer_request_index.len() != self.transition_intent_map.len() {
            return Err(StoreSchemaError(
                "every V2 intent requires one issuer index entry",
            ));
        }
        self.validate_retained_registration_relations()?;
        for record in self.session_namespace_map.values() {
            self.validate_start_namespace_relations(record)?;
        }
        for (key, entry) in &self.object_index {
            require_version(entry.schema_version)?;
            validate_ref_id(&entry.ref_id)
                .map_err(|_| StoreSchemaError("invalid V2 object index ref ID"))?;
            if key != &entry.ref_id || entry.object_schema_version != SCHEMA_VERSION {
                return Err(StoreSchemaError("V2 object index entry mismatch"));
            }
            if entry.object_kind != AuthorityObjectKindV1::TransitionTransportPayload
                && entry.storage_state != AuthorityObjectStorageStateV1::Present
            {
                return Err(StoreSchemaError(
                    "only V2 transport payload objects may be released",
                ));
            }
        }
        Ok(())
    }

    fn validate_retained_registration_relations(&self) -> Result<(), StoreSchemaError> {
        let mut registration_ids = std::collections::BTreeSet::new();
        let mut reserved_ref_ids = std::collections::BTreeSet::new();
        let mut retained_participants = std::collections::BTreeSet::new();
        for (issuer_key, request) in &self.retained_worker_registration_request_index {
            if request.schema_version != 1
                || issuer_key != &request.issuer_request_id
                || !request
                    .issuer_request_id
                    .starts_with("retained-worker-registration:")
                || request.issuer_request_id == "retained-worker-registration:"
                || self.issuer_request_index.contains_key(issuer_key)
                || request.registration_id.is_empty()
                || request.orchestration_session_id.is_empty()
                || request.authority_revision_before == 0
                || request.retained_participant_id.is_empty()
                || request.world_binding.world_id.is_empty()
            {
                return Err(StoreSchemaError(
                    "retained registration request identity is invalid",
                ));
            }
            validate_registration_commitment(&request.authority_record_commitment_before)?;
            validate_registration_ref(&request.current_policy_ref, AuthorityObjectKindV1::Policy)?;
            for (ref_id, commitment, kind) in [
                (
                    &request.descriptor_ref_id,
                    &request.descriptor_commitment,
                    AuthorityObjectKindV1::AgentDescriptor,
                ),
                (
                    &request.resume_handle_ref_id,
                    &request.resume_handle_commitment,
                    AuthorityObjectKindV1::ResumeHandle,
                ),
                (
                    &request.retained_worker_ref_id,
                    &request.retained_worker_commitment,
                    AuthorityObjectKindV1::RetainedWorker,
                ),
            ] {
                validate_ref_id(ref_id)
                    .map_err(|_| StoreSchemaError("retained registration ref ID is invalid"))?;
                validate_object_commitment_rule(kind, 1, commitment)
                    .map_err(|_| StoreSchemaError("retained registration commitment is invalid"))?;
                if !reserved_ref_ids.insert(ref_id.clone()) {
                    return Err(StoreSchemaError(
                        "retained registration object identity is reused",
                    ));
                }
            }
            if !registration_ids.insert(request.registration_id.clone()) {
                return Err(StoreSchemaError("retained registration identity is reused"));
            }
            if !retained_participants.insert(request.retained_participant_id.clone()) {
                return Err(StoreSchemaError(
                    "retained registration participant is reused",
                ));
            }
            match &request.state {
                RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved => {
                    if self
                        .retained_worker_registration_journal
                        .contains_key(&request.registration_id)
                        || [
                            &request.descriptor_ref_id,
                            &request.resume_handle_ref_id,
                            &request.retained_worker_ref_id,
                        ]
                        .iter()
                        .any(|ref_id| self.object_index.contains_key(*ref_id))
                    {
                        return Err(StoreSchemaError(
                            "reserved retained registration is already authoritative",
                        ));
                    }
                }
                RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                    authority_revision_after,
                    authority_record_commitment_after,
                } => {
                    if *authority_revision_after
                        != request
                            .authority_revision_before
                            .checked_add(1)
                            .ok_or(StoreSchemaError("retained authority revision overflow"))?
                    {
                        return Err(StoreSchemaError(
                            "retained registration authority revision is not contiguous",
                        ));
                    }
                    validate_registration_commitment(authority_record_commitment_after)?;
                    let journal = self
                        .retained_worker_registration_journal
                        .get(&request.registration_id)
                        .ok_or(StoreSchemaError(
                            "applied retained registration has no journal",
                        ))?;
                    validate_applied_registration_request(request, journal)?;
                    for (reference, expected_kind) in [
                        (
                            &journal.descriptor_ref,
                            AuthorityObjectKindV1::AgentDescriptor,
                        ),
                        (
                            &journal.resume_handle_ref,
                            AuthorityObjectKindV1::ResumeHandle,
                        ),
                        (
                            &journal.retained_worker_ref,
                            AuthorityObjectKindV1::RetainedWorker,
                        ),
                    ] {
                        let index =
                            self.object_index
                                .get(&reference.ref_id)
                                .ok_or(StoreSchemaError(
                                    "applied retained object has no index entry",
                                ))?;
                        if index.schema_version != 1
                            || index.ref_id != reference.ref_id
                            || index.object_kind != expected_kind
                            || index.object_schema_version != reference.schema_version
                            || index.byte_length == 0
                            || index.storage_state != AuthorityObjectStorageStateV1::Present
                        {
                            return Err(StoreSchemaError(
                                "applied retained object index is inexact",
                            ));
                        }
                    }
                }
            }
        }
        if self.retained_worker_registration_journal.len()
            != self
                .retained_worker_registration_request_index
                .values()
                .filter(|request| {
                    matches!(
                        request.state,
                        RetainedWorkerAuthorityRegistrationRequestStateV1::Applied { .. }
                    )
                })
                .count()
        {
            return Err(StoreSchemaError(
                "retained registration journal has no unique applied request",
            ));
        }
        for (registration_id, journal) in &self.retained_worker_registration_journal {
            if journal.schema_version != 1
                || registration_id != &journal.registration_id
                || !self
                    .retained_worker_registration_request_index
                    .values()
                    .any(|request| {
                        request.registration_id == *registration_id
                            && request.issuer_request_id == journal.issuer_request_id
                    })
            {
                return Err(StoreSchemaError(
                    "retained registration journal identity is invalid",
                ));
            }
        }
        Ok(())
    }

    fn validate_retained_authority_descendant(
        &self,
        intent: &HostSessionTransitionIntentV2,
        current: &DurableSessionAuthorityV1,
        initial: &InitialTransitionApplicationJournalV1,
    ) -> Result<(), StoreSchemaError> {
        let mut expected = current.clone();
        expected.authority_revision = initial.authority_revision_after;
        expected.authoritative_participant_lineage = intent.resulting_authoritative_lineage.clone();
        expected.retained_worker_refs.clear();
        expected.internal_resume_handle_refs.clear();
        expected.updated_at = initial.applied_at.clone();
        let mut expected_commitment = authority_record_commitment(&expected)?;
        if expected_commitment != initial.authority_record_commitment {
            return Err(StoreSchemaError(
                "initial authority is not the base of retained ancestry",
            ));
        }
        let mut consumed = 0_usize;
        while expected.authority_revision < current.authority_revision {
            let candidates = self
                .retained_worker_registration_journal
                .values()
                .filter(|registration| {
                    registration.orchestration_session_id == expected.orchestration_session_id
                        && registration.authority_revision_before == expected.authority_revision
                        && registration.authority_record_commitment_before == expected_commitment
                })
                .collect::<Vec<_>>();
            let [registration] = candidates.as_slice() else {
                return Err(StoreSchemaError(
                    "retained authority ancestry is not uniquely contiguous",
                ));
            };
            if registration.authority_revision_after
                != expected
                    .authority_revision
                    .checked_add(1)
                    .ok_or(StoreSchemaError("retained authority revision overflow"))?
                || expected.current_policy_ref.as_ref() != Some(&registration.current_policy_ref)
                || expected.world_binding.as_ref() != Some(&registration.world_binding)
                || expected
                    .authoritative_participant_lineage
                    .contains(&registration.retained_participant_id)
                || expected
                    .retained_worker_refs
                    .contains(&registration.retained_worker_ref)
            {
                return Err(StoreSchemaError(
                    "retained authority ancestry link is inconsistent",
                ));
            }
            expected.authority_revision = registration.authority_revision_after;
            expected
                .authoritative_participant_lineage
                .push(registration.retained_participant_id.clone());
            expected
                .retained_worker_refs
                .push(registration.retained_worker_ref.clone());
            expected.updated_at = registration.registered_at.clone();
            let lineage_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: super::hash::canonical_sha256(&AuthoritativeLineageHashInputV1 {
                    schema_version: 1,
                    orchestration_session_id: expected.orchestration_session_id.clone(),
                    participant_ids: expected.authoritative_participant_lineage.clone(),
                })
                .map_err(|_| StoreSchemaError("commit retained authority lineage"))?,
            };
            if lineage_commitment != registration.authoritative_lineage_commitment_after {
                return Err(StoreSchemaError(
                    "retained authority lineage commitment is inconsistent",
                ));
            }
            expected_commitment = authority_record_commitment(&expected)?;
            if expected_commitment != registration.authority_record_commitment_after {
                return Err(StoreSchemaError(
                    "retained authority record commitment is inconsistent",
                ));
            }
            consumed += 1;
        }
        let session_registration_count = self
            .retained_worker_registration_journal
            .values()
            .filter(|registration| {
                registration.orchestration_session_id == current.orchestration_session_id
            })
            .count();
        if consumed != session_registration_count || expected != *current {
            return Err(StoreSchemaError(
                "current authority is not the exact retained descendant",
            ));
        }
        Ok(())
    }

    fn validate_start_intent_relations(
        &self,
        intent: &HostSessionTransitionIntentV2,
    ) -> Result<(), StoreSchemaError> {
        if intent.mode != HostSessionTransitionModeV1::Start
            || intent.authority_precondition != HostSessionAuthorityPreconditionV1::ExpectedAbsent
            || intent.source_authoritative_participant_id.is_some()
            || intent.resume_handle_ref.is_some()
            || intent.post_turn_disposition.is_some()
            || !matches!(
                intent.caller.kind,
                super::schema::HostSessionTransitionCallerKindV1::PublicCli
                    | super::schema::HostSessionTransitionCallerKindV1::Repl
            )
            || intent.caller.caller_participant_id.is_some()
            || intent.caller.auto_attach_obligation_id.is_some()
            || intent.caller.auto_attach_claim_owner.is_some()
            || intent.resulting_authoritative_lineage.last()
                != Some(&intent.target_authoritative_participant_id)
            || intent.resulting_authoritative_lineage.len() != 1
            || intent.target_participant_lease_token_ref.object_kind
                != AuthorityObjectKindV1::LeaseToken
            || intent.descriptor_ref.object_kind != AuthorityObjectKindV1::AgentDescriptor
            || intent.host_attach_contract_ref.object_kind
                != AuthorityObjectKindV1::HostAttachContract
            || intent.transport_payload_ref.object_kind
                != AuthorityObjectKindV1::TransitionTransportPayload
            || intent.transport_payload_state
                != HostSessionTransitionTransportPayloadStateV1::Retained
        {
            return Err(StoreSchemaError("strict V2 intent is not a valid Start"));
        }
        match (&intent.transition_input_ref, &intent.input_handoff) {
            (None, HostSessionTransitionInputHandoffV1::NotApplicable) => {}
            (
                Some(reference),
                HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id },
            ) if reference.object_kind == AuthorityObjectKindV1::TransitionInput
                && input_ref == reference
                && run_id == &intent.run_id => {}
            (
                Some(reference),
                HostSessionTransitionInputHandoffV1::Accepted {
                    input_ref, run_id, ..
                }
                | HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                    input_ref,
                    run_id,
                    ..
                },
            ) if reference.object_kind == AuthorityObjectKindV1::TransitionInput
                && input_ref == reference
                && run_id == &intent.run_id => {}
            _ => return Err(StoreSchemaError("V2 Start input handoff is inconsistent")),
        }
        let issuer = self
            .issuer_request_index
            .get(&intent.issuer_request_id)
            .ok_or(StoreSchemaError("V2 Start has no issuer request entry"))?;
        if issuer.intent_id != intent.intent_id
            || issuer.orchestration_session_id != intent.orchestration_session_id
            || issuer.payload_commitment != intent.payload_commitment
        {
            return Err(StoreSchemaError("V2 Start and issuer request disagree"));
        }
        let namespace = self
            .session_namespace_map
            .get(&intent.orchestration_session_id)
            .ok_or(StoreSchemaError("V2 Start has no namespace record"))?;
        match &intent.state {
            HostSessionTransitionIntentStateV2::Issued => {
                if intent.intent_revision != 1 || !v2_start_input_is_pending(intent) {
                    return Err(StoreSchemaError(
                        "issued V2 Start revision/input is invalid",
                    ));
                }
                self.validate_non_applied_start(intent, namespace)?;
            }
            HostSessionTransitionIntentStateV2::Claimed {
                claim_id,
                claimant_attempt_id,
                claim_revision,
                claimed_at,
                claim_expires_at,
            } => {
                required(claim_id)?;
                required(claimant_attempt_id)?;
                if *claim_revision < 2
                    || *claim_revision != intent.intent_revision
                    || claimed_at.as_str() >= claim_expires_at.as_str()
                    || !v2_start_input_is_pending(intent)
                {
                    return Err(StoreSchemaError("V2 Start claim is invalid"));
                }
                self.validate_non_applied_start(intent, namespace)?;
            }
            HostSessionTransitionIntentStateV2::Applied {
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
            } => {
                required(claim_id)?;
                required(claimant_attempt_id)?;
                if intent.intent_revision < 3
                    || authority_revision_before.is_some()
                    || *authority_revision_after != 1
                    || active_authoritative_participant_id
                        != &intent.target_authoritative_participant_id
                    || *resulting_posture != HostSessionPostureV1::ActiveAttached
                    || application_result_ref.object_kind
                        != AuthorityObjectKindV1::ApplicationResult
                    || post_turn.as_ref() != &HostSessionPostTurnApplicationV1::NotApplicable
                    || !v2_start_input_is_applied_pending(intent)
                {
                    return Err(StoreSchemaError("V2 applied Start result is invalid"));
                }
                let HostSessionStartupOwnershipApplicationV1::Pending {
                    expected_run_id,
                    expected_authority_revision,
                    expected_active_authoritative_participant_id,
                } = startup_ownership.as_ref()
                else {
                    return Err(StoreSchemaError(
                        "A1.2a applied Start ownership must remain Pending",
                    ));
                };
                if expected_run_id != &intent.run_id
                    || expected_authority_revision != authority_revision_after
                    || expected_active_authoritative_participant_id
                        != active_authoritative_participant_id
                {
                    return Err(StoreSchemaError(
                        "V2 applied Start ownership expectation is invalid",
                    ));
                }
                let SessionNamespaceRecordV1::Authority(authority) = namespace else {
                    return Err(StoreSchemaError("V2 applied Start has no authority"));
                };
                if authority.authority_revision < *authority_revision_after
                    || authority.shell_trace_session_id != intent.shell_trace_session_id
                    || !authority
                        .authoritative_participant_lineage
                        .starts_with(&intent.resulting_authoritative_lineage)
                    || authority.active_authoritative_participant_id.as_ref()
                        != Some(active_authoritative_participant_id)
                    || authority.workspace_binding != intent.workspace_binding
                    || authority.world_binding != intent.world_binding
                    || authority.host_attach_contract_ref.as_ref()
                        != Some(&intent.host_attach_contract_ref)
                    || authority.lifecycle_posture != *resulting_posture
                    || !authority_origin_matches_v2(authority, intent)
                {
                    return Err(StoreSchemaError("V2 Start authority does not match intent"));
                }
                let journal = self
                    .application_journal
                    .get(&intent.intent_id)
                    .ok_or(StoreSchemaError("V2 applied Start has no journal"))?;
                let initial = &journal.initial_application;
                if journal.startup_terminal_application.is_some()
                    || journal.post_turn_application.is_some()
                    || initial.authority_revision_before != *authority_revision_before
                    || initial.authority_revision_after != *authority_revision_after
                    || initial.authority_record_commitment != *authority_record_commitment
                    || initial.application_result_ref != *application_result_ref
                    || initial.applied_at != *applied_at
                {
                    return Err(StoreSchemaError("V2 applied Start and journal disagree"));
                }
                self.validate_retained_authority_descendant(intent, authority, initial)?;
            }
            HostSessionTransitionIntentStateV2::Rejected {
                reason,
                terminal_handoff_ref,
                ..
            } => {
                if intent.intent_revision < 2 || !v2_start_input_is_terminal(intent) {
                    return Err(StoreSchemaError(
                        "rejected V2 Start revision/input is invalid",
                    ));
                }
                self.validate_terminal_start(intent, namespace)?;
                let SessionNamespaceRecordV1::StartTombstone(tombstone) = namespace else {
                    return Err(StoreSchemaError("rejected V2 Start has no tombstone"));
                };
                if !tombstone_matches_rejected_v2(tombstone, intent, *reason, terminal_handoff_ref)
                {
                    return Err(StoreSchemaError("rejected V2 Start tombstone mismatch"));
                }
            }
            HostSessionTransitionIntentStateV2::Expired {
                terminal_handoff_ref,
                ..
            } => {
                if intent.intent_revision < 2 || !v2_start_input_is_terminal(intent) {
                    return Err(StoreSchemaError(
                        "expired V2 Start revision/input is invalid",
                    ));
                }
                self.validate_terminal_start(intent, namespace)?;
                let SessionNamespaceRecordV1::StartTombstone(tombstone) = namespace else {
                    return Err(StoreSchemaError("expired V2 Start has no tombstone"));
                };
                if !tombstone_matches_expired_v2(tombstone, intent, terminal_handoff_ref) {
                    return Err(StoreSchemaError("expired V2 Start tombstone mismatch"));
                }
            }
        }
        let transport_index = self
            .object_index
            .get(&intent.transport_payload_ref.ref_id)
            .ok_or(StoreSchemaError(
                "V2 transport payload has no object index entry",
            ))?;
        if transport_index.object_kind != AuthorityObjectKindV1::TransitionTransportPayload
            || !transport_states_match(
                &intent.transport_payload_state,
                &transport_index.storage_state,
            )
        {
            return Err(StoreSchemaError(
                "V2 transport parent and object index disagree",
            ));
        }
        validate_terminal_ref_unity_v2(intent, namespace)
    }

    fn validate_non_applied_start(
        &self,
        intent: &HostSessionTransitionIntentV2,
        namespace: &SessionNamespaceRecordV1,
    ) -> Result<(), StoreSchemaError> {
        if self.application_journal.contains_key(&intent.intent_id) {
            return Err(StoreSchemaError("non-applied V2 Start has a journal"));
        }
        let SessionNamespaceRecordV1::StartReservation(reservation) = namespace else {
            return Err(StoreSchemaError("non-applied V2 Start has no reservation"));
        };
        if !reservation_matches_v2(reservation, intent) {
            return Err(StoreSchemaError("V2 Start reservation ownership mismatch"));
        }
        Ok(())
    }

    fn validate_terminal_start(
        &self,
        intent: &HostSessionTransitionIntentV2,
        namespace: &SessionNamespaceRecordV1,
    ) -> Result<(), StoreSchemaError> {
        if self.application_journal.contains_key(&intent.intent_id)
            || !matches!(namespace, SessionNamespaceRecordV1::StartTombstone(_))
        {
            return Err(StoreSchemaError("terminal V2 Start proof is inconsistent"));
        }
        Ok(())
    }

    fn validate_start_namespace_relations(
        &self,
        record: &SessionNamespaceRecordV1,
    ) -> Result<(), StoreSchemaError> {
        let intent_id = match record {
            SessionNamespaceRecordV1::Authority(authority) => match &authority.origin {
                DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } => intent_id,
            },
            SessionNamespaceRecordV1::StartReservation(reservation) => &reservation.intent_id,
            SessionNamespaceRecordV1::StartTombstone(tombstone) => &tombstone.intent_id,
        };
        let intent = self
            .transition_intent_map
            .get(intent_id)
            .ok_or(StoreSchemaError("V2 namespace record has no Start intent"))?;
        let matches = match record {
            SessionNamespaceRecordV1::Authority(authority) => {
                matches!(
                    intent.state,
                    HostSessionTransitionIntentStateV2::Applied { .. }
                ) && authority_origin_matches_v2(authority, intent)
            }
            SessionNamespaceRecordV1::StartReservation(reservation) => {
                matches!(
                    intent.state,
                    HostSessionTransitionIntentStateV2::Issued
                        | HostSessionTransitionIntentStateV2::Claimed { .. }
                ) && reservation_matches_v2(reservation, intent)
            }
            SessionNamespaceRecordV1::StartTombstone(tombstone) => match &intent.state {
                HostSessionTransitionIntentStateV2::Rejected {
                    reason,
                    terminal_handoff_ref,
                    ..
                } => {
                    tombstone_matches_rejected_v2(tombstone, intent, *reason, terminal_handoff_ref)
                }
                HostSessionTransitionIntentStateV2::Expired {
                    terminal_handoff_ref,
                    ..
                } => tombstone_matches_expired_v2(tombstone, intent, terminal_handoff_ref),
                _ => false,
            },
        };
        if matches {
            Ok(())
        } else {
            Err(StoreSchemaError("V2 namespace ownership is inconsistent"))
        }
    }
}

impl StateRootV3 {
    pub(crate) fn try_from_v2(root: &StateRootV2) -> Result<Self, StoreSchemaError> {
        root.validate()?;
        let upgraded = Self {
            schema_version: 3,
            authority_store_id: root.authority_store_id.clone(),
            bootstrap_home: root.bootstrap_home.clone(),
            root_revision: root
                .root_revision
                .checked_add(1)
                .ok_or(StoreSchemaError("authority root revision overflow"))?,
            active_commitment_key_id: root.active_commitment_key_id.clone(),
            commitment_key_registry: root.commitment_key_registry.clone(),
            greenfield_namespace_certificate: root.greenfield_namespace_certificate.clone(),
            session_namespace_map: root.session_namespace_map.clone(),
            transition_intent_map: root.transition_intent_map.clone(),
            issuer_request_index: root.issuer_request_index.clone(),
            application_journal: root.application_journal.clone(),
            retained_worker_registration_request_index: root
                .retained_worker_registration_request_index
                .clone(),
            retained_worker_registration_journal: root.retained_worker_registration_journal.clone(),
            successor_transition_intent_map: BTreeMap::new(),
            successor_issuer_request_index: BTreeMap::new(),
            successor_application_journal: BTreeMap::new(),
            start_transaction_map: BTreeMap::new(),
            stop_transaction_map: BTreeMap::new(),
            object_index: root.object_index.clone(),
        };
        upgraded.validate()?;
        Ok(upgraded)
    }

    pub(crate) fn validate_greenfield(&self) -> Result<(), StoreSchemaError> {
        self.validate()?;
        if !self.session_namespace_map.is_empty()
            || !self.transition_intent_map.is_empty()
            || !self.issuer_request_index.is_empty()
            || !self.application_journal.is_empty()
            || !self.retained_worker_registration_request_index.is_empty()
            || !self.retained_worker_registration_journal.is_empty()
            || !self.successor_transition_intent_map.is_empty()
            || !self.successor_issuer_request_index.is_empty()
            || !self.successor_application_journal.is_empty()
            || !self.start_transaction_map.is_empty()
            || !self.stop_transaction_map.is_empty()
            || !self.object_index.is_empty()
        {
            return Err(StoreSchemaError(
                "A1.2b StateRootV3 must remain greenfield-empty",
            ));
        }
        Ok(())
    }

    pub(crate) fn preserved_v2_view(&self) -> StateRootV2 {
        StateRootV2 {
            schema_version: 2,
            authority_store_id: self.authority_store_id.clone(),
            bootstrap_home: self.bootstrap_home.clone(),
            root_revision: self.root_revision,
            active_commitment_key_id: self.active_commitment_key_id.clone(),
            commitment_key_registry: self.commitment_key_registry.clone(),
            greenfield_namespace_certificate: self.greenfield_namespace_certificate.clone(),
            session_namespace_map: self.session_namespace_map.clone(),
            transition_intent_map: self.transition_intent_map.clone(),
            issuer_request_index: self.issuer_request_index.clone(),
            application_journal: self.application_journal.clone(),
            retained_worker_registration_request_index: self
                .retained_worker_registration_request_index
                .clone(),
            retained_worker_registration_journal: self.retained_worker_registration_journal.clone(),
            object_index: self.object_index.clone(),
        }
    }

    fn preserved_v2_validation_view(&self) -> Result<StateRootV2, StoreSchemaError> {
        let mut preserved = self.preserved_v2_view();
        let mut preserved_transport_refs = Vec::new();
        for intent in preserved.transition_intent_map.values_mut() {
            let HostSessionTransitionIntentStateV2::Applied {
                authority_revision_after,
                active_authoritative_participant_id,
                startup_ownership,
                post_turn,
                ..
            } = &mut intent.state
            else {
                continue;
            };
            if !matches!(
                startup_ownership.as_ref(),
                HostSessionStartupOwnershipApplicationV1::Pending { .. }
            ) {
                *startup_ownership = Box::new(HostSessionStartupOwnershipApplicationV1::Pending {
                    expected_run_id: intent.run_id.clone(),
                    expected_authority_revision: *authority_revision_after,
                    expected_active_authoritative_participant_id:
                        active_authoritative_participant_id.clone(),
                });
            }
            *post_turn = Box::new(HostSessionPostTurnApplicationV1::NotApplicable);
            if let (
                Some(reference),
                HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance { .. },
            ) = (&intent.transition_input_ref, &intent.input_handoff)
            {
                intent.input_handoff = HostSessionTransitionInputHandoffV1::Pending {
                    input_ref: reference.clone(),
                    run_id: intent.run_id.clone(),
                };
            }
            intent.transport_payload_state = HostSessionTransitionTransportPayloadStateV1::Retained;
            preserved_transport_refs.push(intent.transport_payload_ref.ref_id.clone());
        }
        for ref_id in preserved_transport_refs {
            if let Some(entry) = preserved.object_index.get_mut(&ref_id) {
                entry.storage_state = AuthorityObjectStorageStateV1::Present;
            }
        }
        for journal in preserved.application_journal.values_mut() {
            journal.startup_terminal_application = None;
            journal.post_turn_application = None;
        }
        for (session_id, record) in &mut preserved.session_namespace_map {
            if let SessionNamespaceRecordV1::Authority(authority) = record {
                let mut completed_stops = self.stop_transaction_map.values().filter(|stop| {
                    stop.orchestration_session_id == *session_id
                        && matches!(stop.state, HostSessionStopIntentStateV1::Completed { .. })
                });
                if let Some(stop) = completed_stops.next() {
                    if completed_stops.next().is_some() {
                        return Err(StoreSchemaError(
                            "multiple completed HSA Stops target one session",
                        ));
                    }
                    **authority = stop.authority_before.as_ref().clone();
                }
                if self.has_any_successor_transition(session_id)
                    || authority.authority_revision != 1
                    || authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
                {
                    let history = self.reconstruct_v2_authority_history(authority.as_ref())?;
                    let reconstructed = history
                        .last_key_value()
                        .map(|(_, state)| state.authority.clone())
                        .ok_or(StoreSchemaError(
                            "V3 preserved V2 authority history is empty",
                        ))?;
                    **authority = reconstructed;
                }
                authority.internal_resume_handle_refs.clear();
            }
        }
        preserved
            .object_index
            .retain(|_, entry| entry.object_schema_version == SCHEMA_VERSION);
        Ok(preserved)
    }

    pub(crate) fn validate(&self) -> Result<(), StoreSchemaError> {
        if self.schema_version != 3 {
            return Err(StoreSchemaError("StateRootV3 requires schema version 3"));
        }
        if self.root_revision < 3 {
            return Err(StoreSchemaError("V3 root revision must be at least three"));
        }
        let preserved = self.preserved_v2_validation_view()?;
        preserved.validate()?;
        let mut active_request_keys = std::collections::BTreeSet::new();
        for (key, transaction) in &self.start_transaction_map {
            if key != &transaction.transaction_id
                || transaction.schema_version != 1
                || transaction.authority_store_id != self.authority_store_id
                || transaction.start_authority_revision != 1
                || transaction.world_id.is_some() != transaction.world_generation.is_some()
                || !lower_hex(&transaction.request_key_sha256, 64)
                || !lower_hex(&transaction.prompt_sha256, 64)
            {
                return Err(StoreSchemaError("invalid V3 Start transaction identity"));
            }
            for value in [
                transaction.transaction_id.as_str(),
                transaction.orchestration_session_id.as_str(),
                transaction.shell_trace_session_id.as_str(),
                transaction.authoritative_participant_id.as_str(),
                transaction.backend_id.as_str(),
                transaction.protocol.as_str(),
                transaction.workspace_root.as_str(),
                transaction.public_backend_id.as_str(),
                transaction.public_scope.as_str(),
                transaction.start_intent_id.as_str(),
                transaction.start_issuer_request_id.as_str(),
                transaction.start_run_id.as_str(),
            ] {
                required(value)?;
            }
            let intent = self
                .transition_intent_map
                .get(&transaction.start_intent_id)
                .ok_or(StoreSchemaError(
                    "Start transaction references no Start intent",
                ))?;
            let HostSessionTransitionIntentStateV2::Applied {
                application_result_ref,
                authority_revision_after,
                active_authoritative_participant_id,
                ..
            } = &intent.state
            else {
                return Err(StoreSchemaError(
                    "Start transaction requires an applied Start intent",
                ));
            };
            if intent.mode != HostSessionTransitionModeV1::Start
                || intent.orchestration_session_id != transaction.orchestration_session_id
                || intent.shell_trace_session_id != transaction.shell_trace_session_id
                || intent.target_authoritative_participant_id
                    != transaction.authoritative_participant_id
                || intent.issuer_request_id != transaction.start_issuer_request_id
                || intent.payload_commitment != transaction.start_payload_commitment
                || application_result_ref != &transaction.start_application_result_ref
                || intent.run_id != transaction.start_run_id
                || active_authoritative_participant_id != &transaction.authoritative_participant_id
                || *authority_revision_after != transaction.start_authority_revision
                || intent.workspace_binding.workspace_root.physical_path
                    != transaction.workspace_root
                || intent
                    .world_binding
                    .as_ref()
                    .map(|binding| binding.world_id.as_str())
                    != transaction.world_id.as_deref()
                || intent
                    .world_binding
                    .as_ref()
                    .map(|binding| binding.world_generation)
                    != transaction.world_generation
            {
                return Err(StoreSchemaError(
                    "Start transaction conflicts with its applied Start intent",
                ));
            }
            let authority = self
                .session_namespace_map
                .get(&transaction.orchestration_session_id)
                .and_then(|record| match record {
                    SessionNamespaceRecordV1::Authority(authority) => Some(authority.as_ref()),
                    _ => None,
                })
                .ok_or(StoreSchemaError(
                    "Start transaction references no authority",
                ))?;
            let (expected_revision, expected_ref, expected_posture, settled) =
                match &transaction.state {
                    StartTransactionStateV1::PromptNotSubmitted
                    | StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
                    | StartTransactionStateV1::PromptSubmissionIndeterminate { .. } => (
                        transaction.start_authority_revision,
                        None,
                        HostSessionPostureV1::ActiveAttached,
                        false,
                    ),
                    StartTransactionStateV1::ContinuationRegistered {
                        registration_ref,
                        authority_revision_after,
                        ..
                    } => (
                        *authority_revision_after,
                        Some(registration_ref),
                        HostSessionPostureV1::ActiveAttached,
                        false,
                    ),
                    StartTransactionStateV1::TurnSettledAwaitingResponse {
                        settlement_ref,
                        authority_revision_after,
                        resulting_posture,
                        ..
                    } => (
                        *authority_revision_after,
                        Some(settlement_ref),
                        *resulting_posture,
                        true,
                    ),
                    StartTransactionStateV1::PublicResponseDelivered {
                        settlement_ref,
                        authority_revision_after,
                        resulting_posture,
                        ..
                    } => (
                        *authority_revision_after,
                        Some(settlement_ref),
                        *resulting_posture,
                        true,
                    ),
                };
            let authority_state_mismatch = if settled {
                authority.authority_revision < expected_revision
                    || (authority.authority_revision == expected_revision
                        && authority.lifecycle_posture != expected_posture)
            } else {
                authority.authority_revision != expected_revision
                    || authority.active_authoritative_participant_id.as_deref()
                        != Some(transaction.authoritative_participant_id.as_str())
                    || authority.lifecycle_posture != expected_posture
            };
            if authority_state_mismatch
                || expected_ref.is_some_and(|reference| {
                    (!settled && authority.internal_resume_handle_refs.last() != Some(reference))
                        || (settled && !authority.internal_resume_handle_refs.contains(reference))
                        || reference.object_kind != AuthorityObjectKindV1::ResumeHandle
                        || reference.schema_version != 2
                })
            {
                return Err(StoreSchemaError(
                    "Start transaction state conflicts with current authority",
                ));
            }
            if !matches!(
                transaction.state,
                StartTransactionStateV1::PublicResponseDelivered { .. }
            ) && !active_request_keys.insert(transaction.request_key_sha256.as_str())
            {
                return Err(StoreSchemaError(
                    "multiple unfinished Start transactions share a request key",
                ));
            }
        }
        let mut stop_sessions = std::collections::BTreeSet::new();
        let mut stop_requests = std::collections::BTreeSet::new();
        for (key, stop) in &self.stop_transaction_map {
            if key != &stop.intent_id
                || stop.schema_version != 1
                || stop.authority_store_id != self.authority_store_id
                || stop.bootstrap_home != self.bootstrap_home
                || stop.expected_root_revision == 0
                || stop.expected_root_revision >= self.root_revision
                || stop.authority_before.schema_version != 1
                || stop.authority_before.orchestration_session_id != stop.orchestration_session_id
                || stop.authority_before.shell_trace_session_id != stop.shell_trace_session_id
                || stop
                    .authority_before
                    .active_authoritative_participant_id
                    .as_deref()
                    != Some(stop.authoritative_participant_id.as_str())
                || stop
                    .authority_before
                    .authoritative_participant_lineage
                    .last()
                    != Some(&stop.authoritative_participant_id)
                || stop.authority_before.lifecycle_posture == HostSessionPostureV1::Terminal
                || stop.authority_before.lifecycle_posture == HostSessionPostureV1::Invalid
                || stop.updated_at.as_str() < stop.issued_at.as_str()
                || !stop_sessions.insert(stop.orchestration_session_id.as_str())
                || !stop_requests.insert(stop.request_id.as_str())
            {
                return Err(StoreSchemaError("invalid HSA Stop transaction identity"));
            }
            required(&stop.intent_id)?;
            required(&stop.request_id)?;
            required(&stop.orchestration_session_id)?;
            required(&stop.shell_trace_session_id)?;
            required(&stop.authoritative_participant_id)?;
            if stop.caller.kind != super::schema::HostSessionTransitionCallerKindV1::PublicCli
                || stop.caller.caller_participant_id.is_some()
                || stop.caller.auto_attach_obligation_id.is_some()
                || stop.caller.auto_attach_claim_owner.is_some()
            {
                return Err(StoreSchemaError("invalid HSA Stop caller identity"));
            }
            validate_registration_commitment(&stop.payload_commitment)?;
            validate_registration_commitment(&stop.authority_record_commitment_before)?;
            validate_registration_commitment(&stop.authoritative_lineage_commitment_before)?;
            let before_commitment = authority_record_commitment(stop.authority_before.as_ref())?;
            let before_lineage_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: super::hash::canonical_sha256(&AuthoritativeLineageHashInputV1 {
                    schema_version: 1,
                    orchestration_session_id: stop.orchestration_session_id.clone(),
                    participant_ids: stop
                        .authority_before
                        .authoritative_participant_lineage
                        .clone(),
                })
                .map_err(|_| StoreSchemaError("commit HSA Stop predecessor lineage"))?,
            };
            if before_commitment != stop.authority_record_commitment_before
                || before_lineage_commitment != stop.authoritative_lineage_commitment_before
            {
                return Err(StoreSchemaError("HSA Stop predecessor commitment mismatch"));
            }
            let payload = HostSessionStopPayloadHashInputV1 {
                schema_version: 1,
                intent_id: stop.intent_id.clone(),
                request_id: stop.request_id.clone(),
                authority_store_id: stop.authority_store_id.clone(),
                bootstrap_home: stop.bootstrap_home.clone(),
                orchestration_session_id: stop.orchestration_session_id.clone(),
                shell_trace_session_id: stop.shell_trace_session_id.clone(),
                caller: stop.caller.clone(),
                authority_revision: stop.authority_before.authority_revision,
                authority_record_commitment: stop.authority_record_commitment_before.clone(),
                authoritative_lineage_commitment: stop
                    .authoritative_lineage_commitment_before
                    .clone(),
                authoritative_participant_id: stop.authoritative_participant_id.clone(),
                authoritative_lineage: stop
                    .authority_before
                    .authoritative_participant_lineage
                    .clone(),
                lifecycle_posture: stop.authority_before.lifecycle_posture,
                issued_at: stop.issued_at.clone(),
            };
            let expected_payload_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: super::hash::canonical_sha256(&payload)
                    .map_err(|_| StoreSchemaError("invalid HSA Stop payload"))?,
            };
            if stop.payload_commitment != expected_payload_commitment {
                return Err(StoreSchemaError("HSA Stop payload commitment mismatch"));
            }
            let current = self
                .session_namespace_map
                .get(&stop.orchestration_session_id)
                .and_then(|record| match record {
                    SessionNamespaceRecordV1::Authority(authority) => Some(authority.as_ref()),
                    _ => None,
                })
                .ok_or(StoreSchemaError("HSA Stop references no current authority"))?;
            match &stop.state {
                HostSessionStopIntentStateV1::Issued => {
                    if current != stop.authority_before.as_ref()
                        || stop.updated_at != stop.issued_at
                    {
                        return Err(StoreSchemaError("issued HSA Stop is stale or mismatched"));
                    }
                }
                HostSessionStopIntentStateV1::DeliveryAccepted {
                    acceptance_id,
                    accepted_by_participant_id,
                    accepted_at,
                } => {
                    required(acceptance_id)?;
                    required(accepted_by_participant_id)?;
                    if stop.authority_before.lifecycle_posture
                        != HostSessionPostureV1::ActiveAttached
                        || accepted_by_participant_id != &stop.authoritative_participant_id
                        || accepted_at.as_str() < stop.issued_at.as_str()
                        || &stop.updated_at != accepted_at
                        || current != stop.authority_before.as_ref()
                    {
                        return Err(StoreSchemaError(
                            "accepted HSA Stop delivery is stale or mismatched",
                        ));
                    }
                }
                HostSessionStopIntentStateV1::Completed {
                    delivery_acceptance_id,
                    delivery_accepted_by_participant_id,
                    delivery_accepted_at,
                    result_id,
                    result_commitment,
                    authority_revision_after,
                    authority_record_commitment_after,
                    completed_at,
                } => {
                    required(result_id)?;
                    let active_delivery = stop.authority_before.lifecycle_posture
                        == HostSessionPostureV1::ActiveAttached;
                    if delivery_acceptance_id.is_some() != active_delivery
                        || delivery_accepted_by_participant_id.is_some() != active_delivery
                        || delivery_accepted_at.is_some() != active_delivery
                        || delivery_accepted_by_participant_id
                            .as_deref()
                            .is_some_and(|value| value != stop.authoritative_participant_id)
                        || delivery_accepted_at
                            .as_ref()
                            .is_some_and(|value| completed_at.as_str() < value.as_str())
                        || completed_at.as_str() < stop.issued_at.as_str()
                        || &stop.updated_at != completed_at
                    {
                        return Err(StoreSchemaError(
                            "completed HSA Stop delivery identity is invalid",
                        ));
                    }
                    let mut expected_after = stop.authority_before.as_ref().clone();
                    expected_after.authority_revision = expected_after
                        .authority_revision
                        .checked_add(1)
                        .ok_or(StoreSchemaError("HSA Stop authority revision overflow"))?;
                    expected_after.lifecycle_posture = HostSessionPostureV1::Terminal;
                    expected_after.updated_at = completed_at.clone();
                    let expected_after_commitment = authority_record_commitment(&expected_after)?;
                    if *authority_revision_after != expected_after.authority_revision
                        || authority_record_commitment_after != &expected_after_commitment
                        || current != &expected_after
                    {
                        return Err(StoreSchemaError(
                            "completed HSA Stop authority result is inconsistent",
                        ));
                    }
                    let result = HostSessionStopResultHashInputV1 {
                        schema_version: 1,
                        result_id: result_id.clone(),
                        intent_id: stop.intent_id.clone(),
                        request_id: stop.request_id.clone(),
                        payload_commitment: stop.payload_commitment.clone(),
                        authority_store_id: stop.authority_store_id.clone(),
                        orchestration_session_id: stop.orchestration_session_id.clone(),
                        authoritative_participant_id: stop.authoritative_participant_id.clone(),
                        delivery_acceptance_id: delivery_acceptance_id.clone(),
                        authority_revision_before: stop.authority_before.authority_revision,
                        authority_record_commitment_before: stop
                            .authority_record_commitment_before
                            .clone(),
                        authority_revision_after: *authority_revision_after,
                        authority_record_commitment_after: authority_record_commitment_after
                            .clone(),
                        resulting_posture: HostSessionPostureV1::Terminal,
                        completed_at: completed_at.clone(),
                    };
                    let expected_result_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                        digest_hex: super::hash::canonical_sha256(&result)
                            .map_err(|_| StoreSchemaError("invalid HSA Stop result"))?,
                    };
                    if result_commitment != &expected_result_commitment {
                        return Err(StoreSchemaError("HSA Stop result commitment mismatch"));
                    }
                }
            }
        }
        let mut successor_histories = BTreeMap::new();
        for (key, record) in &self.session_namespace_map {
            if key != record.orchestration_session_id() {
                return Err(StoreSchemaError("V3 session namespace map key mismatch"));
            }
            validate_namespace_record_identity(
                record,
                &self.authority_store_id,
                &self.bootstrap_home,
                true,
            )?;
            if let SessionNamespaceRecordV1::Authority(authority) = record {
                let completed_stop = self.stop_transaction_map.values().find(|stop| {
                    stop.orchestration_session_id == *key
                        && matches!(stop.state, HostSessionStopIntentStateV1::Completed { .. })
                });
                let history_authority = completed_stop
                    .map(|stop| stop.authority_before.as_ref())
                    .unwrap_or(authority.as_ref());
                let history = if self.has_any_successor_transition(key) {
                    let history =
                        self.reconstruct_successor_authority_history(history_authority)?;
                    let reconstructed = history
                        .last_key_value()
                        .map(|(_, state)| &state.authority)
                        .ok_or(StoreSchemaError("V3 successor authority history is empty"))?;
                    if reconstructed != history_authority {
                        return Err(StoreSchemaError(
                            "current V3 authority is not the exact reconstructed successor state",
                        ));
                    }
                    history
                } else {
                    let history =
                        self.reconstruct_v2_runtime_authority_history(history_authority)?;
                    let reconstructed = history
                        .last_key_value()
                        .map(|(_, state)| &state.authority)
                        .ok_or(StoreSchemaError("V3 runtime V2 history is empty"))?;
                    if reconstructed != history_authority {
                        return Err(StoreSchemaError(
                            "current V3 authority is not the exact reconstructed V2 runtime state",
                        ));
                    }
                    history
                };
                let mut history = history;
                if let Some(stop) = completed_stop {
                    let HostSessionStopIntentStateV1::Completed {
                        authority_revision_after,
                        authority_record_commitment_after,
                        completed_at,
                        ..
                    } = &stop.state
                    else {
                        unreachable!("completed Stop filter must retain completed state")
                    };
                    if *authority_revision_after != history_authority.authority_revision + 1
                        || authority_record_commitment_after
                            != &authority_record_commitment(authority.as_ref())?
                        || authority.authority_revision != *authority_revision_after
                        || authority.lifecycle_posture != HostSessionPostureV1::Terminal
                        || &authority.updated_at != completed_at
                    {
                        return Err(StoreSchemaError(
                            "current V3 authority is not the exact completed HSA Stop state",
                        ));
                    }
                    history.insert(
                        *authority_revision_after,
                        reconstruct_authority_state(authority.as_ref().clone())?,
                    );
                }
                successor_histories.insert(key.clone(), history);
            }
        }
        for (key, entry) in &self.object_index {
            require_version(entry.schema_version)?;
            validate_ref_id(&entry.ref_id)
                .map_err(|_| StoreSchemaError("invalid V3 object index ref ID"))?;
            if key != &entry.ref_id
                || !v3_object_schema_version_allowed(entry.object_kind, entry.object_schema_version)
            {
                return Err(StoreSchemaError("V3 object index entry mismatch"));
            }
            if entry.object_kind != AuthorityObjectKindV1::TransitionTransportPayload
                && entry.storage_state != AuthorityObjectStorageStateV1::Present
            {
                return Err(StoreSchemaError(
                    "only V3 transport payload objects may be released",
                ));
            }
        }
        for (key, intent) in &self.successor_transition_intent_map {
            if key != &intent.intent_id || intent.schema_version != 3 {
                return Err(StoreSchemaError("V3 successor intent map entry mismatch"));
            }
            if self.transition_intent_map.contains_key(key) {
                return Err(StoreSchemaError(
                    "V3 successor intent identity collides with preserved Start intent",
                ));
            }
            let history = successor_histories
                .get(&intent.orchestration_session_id)
                .ok_or(StoreSchemaError(
                    "V3 successor session has no reconstructed authority history",
                ))?;
            self.validate_successor_intent_relations(intent, history)?;
        }
        for (key, entry) in &self.successor_issuer_request_index {
            if key != &entry.issuer_request_id {
                return Err(StoreSchemaError(
                    "V3 successor issuer request index key mismatch",
                ));
            }
            if self.issuer_request_index.contains_key(key) {
                return Err(StoreSchemaError(
                    "V3 successor issuer request identity collides with preserved Start intent",
                ));
            }
            require_version(entry.schema_version)?;
            let intent = self
                .successor_transition_intent_map
                .get(&entry.intent_id)
                .ok_or(StoreSchemaError(
                    "V3 successor issuer request references no intent",
                ))?;
            if intent.issuer_request_id != entry.issuer_request_id
                || intent.orchestration_session_id != entry.orchestration_session_id
                || intent.payload_commitment != entry.payload_commitment
            {
                return Err(StoreSchemaError(
                    "V3 successor issuer request and intent disagree",
                ));
            }
        }
        for (key, journal) in &self.successor_application_journal {
            if key != &journal.intent_id || journal.schema_version != 3 {
                return Err(StoreSchemaError(
                    "V3 successor application journal entry mismatch",
                ));
            }
            let intent = self
                .successor_transition_intent_map
                .get(&journal.intent_id)
                .ok_or(StoreSchemaError(
                    "V3 successor application journal references no intent",
                ))?;
            self.validate_successor_application_journal(intent, journal)?;
        }
        if self.successor_issuer_request_index.len() != self.successor_transition_intent_map.len() {
            return Err(StoreSchemaError(
                "every V3 successor intent requires one issuer index entry",
            ));
        }
        Ok(())
    }

    fn validate_successor_intent_relations(
        &self,
        intent: &HostSessionTransitionIntentV3,
        history: &BTreeMap<u64, ReconstructedAuthorityStateV1>,
    ) -> Result<(), StoreSchemaError> {
        required(&intent.intent_id)?;
        required(&intent.issuer_request_id)?;
        required(&intent.orchestration_session_id)?;
        required(&intent.shell_trace_session_id)?;
        required(&intent.target_authoritative_participant_id)?;
        required(&intent.run_id)?;
        if intent.intent_revision == 0
            || intent.workspace_binding.authority_store_id != self.authority_store_id
            || intent.workspace_binding.authority_store_root != self.bootstrap_home
            || intent.issued_at.as_str() >= intent.expires_at.as_str()
        {
            return Err(StoreSchemaError("V3 successor intent binding mismatch"));
        }
        let (
            authority_revision,
            expected_authority_record_commitment,
            active_authoritative_participant_id,
            authoritative_lineage_commitment,
            lifecycle_posture,
        ) = match &intent.authority_precondition {
            HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision,
                authority_record_commitment,
                active_authoritative_participant_id,
                authoritative_lineage_commitment,
                lifecycle_posture,
            } => (
                *authority_revision,
                authority_record_commitment,
                active_authoritative_participant_id,
                authoritative_lineage_commitment,
                *lifecycle_posture,
            ),
            HostSessionAuthorityPreconditionV1::ExpectedAbsent => {
                return Err(StoreSchemaError(
                    "V3 successor intent requires ExpectedRevision",
                ))
            }
        };
        if authority_revision == 0 {
            return Err(StoreSchemaError(
                "V3 successor authority precondition is invalid",
            ));
        }
        validate_registration_commitment(expected_authority_record_commitment)?;
        validate_registration_commitment(authoritative_lineage_commitment)?;
        required(active_authoritative_participant_id)?;
        let precondition_state = history.get(&authority_revision).ok_or(StoreSchemaError(
            "V3 successor authority precondition has no reconstructed state",
        ))?;
        let precondition_authority = &precondition_state.authority;
        let mut expected_successor_lineage = precondition_authority
            .authoritative_participant_lineage
            .clone();
        expected_successor_lineage.push(intent.target_authoritative_participant_id.clone());
        if !matches!(
            intent.mode,
            HostSessionTransitionModeV1::Attach | HostSessionTransitionModeV1::ResumeOneTurn
        ) || !matches!(
            lifecycle_posture,
            HostSessionPostureV1::ParkedResumable
                | HostSessionPostureV1::DetachedReconciled
                | HostSessionPostureV1::AwaitingAttention
                | HostSessionPostureV1::StaleRecoverable
        ) || intent.source_authoritative_participant_id.as_deref()
            != Some(active_authoritative_participant_id.as_str())
            || intent.resulting_authoritative_lineage != expected_successor_lineage
            || intent
                .resulting_authoritative_lineage
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != intent.resulting_authoritative_lineage.len()
            || active_authoritative_participant_id == &intent.target_authoritative_participant_id
            || intent.target_participant_lease_token_ref.object_kind
                != AuthorityObjectKindV1::LeaseToken
            || intent.descriptor_ref.object_kind != AuthorityObjectKindV1::AgentDescriptor
            || intent.host_attach_contract_ref.object_kind
                != AuthorityObjectKindV1::HostAttachContract
            || intent.transport_payload_ref.object_kind
                != AuthorityObjectKindV1::TransitionTransportPayload
            || precondition_authority.orchestration_session_id != intent.orchestration_session_id
            || precondition_authority.shell_trace_session_id != intent.shell_trace_session_id
            || precondition_authority.workspace_binding != intent.workspace_binding
            || precondition_authority.world_binding != intent.world_binding
            || precondition_authority.host_attach_contract_ref.as_ref()
                != Some(&intent.host_attach_contract_ref)
        {
            return Err(StoreSchemaError("strict V3 successor intent is invalid"));
        }
        if precondition_state.authority_record_commitment != *expected_authority_record_commitment
            || precondition_state.authoritative_lineage_commitment
                != *authoritative_lineage_commitment
            || precondition_authority
                .active_authoritative_participant_id
                .as_deref()
                != Some(active_authoritative_participant_id.as_str())
            || precondition_authority.lifecycle_posture != lifecycle_posture
        {
            return Err(StoreSchemaError(
                "V3 successor authority precondition does not match reconstructed history",
            ));
        }
        match intent.mode {
            HostSessionTransitionModeV1::Attach => {
                if intent.transition_input_ref.is_some()
                    || !matches!(
                        intent.input_handoff,
                        HostSessionTransitionInputHandoffV1::NotApplicable
                    )
                    || intent.post_turn_disposition.is_some()
                {
                    return Err(StoreSchemaError(
                        "V3 Attach input or post-turn state is invalid",
                    ));
                }
            }
            HostSessionTransitionModeV1::ResumeOneTurn => {
                let Some(reference) = intent.transition_input_ref.as_ref() else {
                    return Err(StoreSchemaError(
                        "V3 Resume requires a transition input ref",
                    ));
                };
                if reference.object_kind != AuthorityObjectKindV1::TransitionInput {
                    return Err(StoreSchemaError("V3 Resume input ref kind is invalid"));
                }
                if intent.resume_handle_ref.is_none()
                    || intent.post_turn_disposition
                        != Some(HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal)
                {
                    return Err(StoreSchemaError(
                        "V3 Resume handle or post-turn disposition is invalid",
                    ));
                }
                match &intent.input_handoff {
                    HostSessionTransitionInputHandoffV1::Pending { input_ref, run_id }
                    | HostSessionTransitionInputHandoffV1::Accepted {
                        input_ref, run_id, ..
                    }
                    | HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
                        input_ref,
                        run_id,
                        ..
                    } if input_ref == reference && run_id == &intent.run_id => {}
                    _ => return Err(StoreSchemaError("V3 Resume input handoff is inconsistent")),
                }
            }
            HostSessionTransitionModeV1::Start => {
                return Err(StoreSchemaError("V3 successor mode cannot be Start"))
            }
        }
        let issuer = self
            .successor_issuer_request_index
            .get(&intent.issuer_request_id)
            .ok_or(StoreSchemaError(
                "V3 successor intent has no issuer request entry",
            ))?;
        if issuer.intent_id != intent.intent_id
            || issuer.orchestration_session_id != intent.orchestration_session_id
            || issuer.payload_commitment != intent.payload_commitment
        {
            return Err(StoreSchemaError(
                "V3 successor intent and issuer request disagree",
            ));
        }
        match &intent.state {
            HostSessionTransitionIntentStateV3::Issued => {
                if intent.intent_revision != 1
                    || self
                        .successor_application_journal
                        .contains_key(&intent.intent_id)
                {
                    return Err(StoreSchemaError("issued V3 successor state is invalid"));
                }
            }
            HostSessionTransitionIntentStateV3::Claimed {
                claim_id,
                claimant_attempt_id,
                claim_revision,
                claimed_at,
                claim_expires_at,
            } => {
                required(claim_id)?;
                required(claimant_attempt_id)?;
                if *claim_revision < 2
                    || *claim_revision != intent.intent_revision
                    || claimed_at.as_str() >= claim_expires_at.as_str()
                    || self
                        .successor_application_journal
                        .contains_key(&intent.intent_id)
                {
                    return Err(StoreSchemaError("claimed V3 successor state is invalid"));
                }
            }
            HostSessionTransitionIntentStateV3::Applied {
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
                ..
            } => {
                required(claim_id)?;
                required(claimant_attempt_id)?;
                if intent.intent_revision < 3
                    || authority_revision_before != &Some(authority_revision)
                    || *authority_revision_after
                        != authority_revision
                            .checked_add(1)
                            .ok_or(StoreSchemaError("successor authority revision overflow"))?
                    || active_authoritative_participant_id
                        != &intent.target_authoritative_participant_id
                    || *resulting_posture != HostSessionPostureV1::ActiveAttached
                    || application_result_ref.object_kind
                        != AuthorityObjectKindV1::ApplicationResult
                {
                    return Err(StoreSchemaError("applied V3 successor result is invalid"));
                }
                validate_registration_commitment(authority_record_commitment)?;
                let applied_state =
                    history
                        .get(authority_revision_after)
                        .ok_or(StoreSchemaError(
                            "applied V3 successor has no reconstructed authority state",
                        ))?;
                match intent.mode {
                    HostSessionTransitionModeV1::Attach => {
                        if matches!(
                            startup_ownership.as_ref(),
                            HostSessionStartupOwnershipApplicationV1::NotApplicable
                        ) || post_turn.as_ref()
                            != &HostSessionPostTurnApplicationV2::NotApplicable
                        {
                            return Err(StoreSchemaError(
                                "applied V3 Attach startup or post-turn state is invalid",
                            ));
                        }
                    }
                    HostSessionTransitionModeV1::ResumeOneTurn => {
                        if startup_ownership.as_ref()
                            != &HostSessionStartupOwnershipApplicationV1::NotApplicable
                            || matches!(
                                post_turn.as_ref(),
                                HostSessionPostTurnApplicationV2::NotApplicable
                            )
                        {
                            return Err(StoreSchemaError(
                                "applied V3 Resume startup or post-turn state is invalid",
                            ));
                        }
                    }
                    HostSessionTransitionModeV1::Start => unreachable!(),
                }
                let (expected_current_revision, expected_current_posture) = match intent.mode {
                    HostSessionTransitionModeV1::Attach => match startup_ownership.as_ref() {
                        HostSessionStartupOwnershipApplicationV1::Pending { .. }
                        | HostSessionStartupOwnershipApplicationV1::Accepted { .. } => (
                            *authority_revision_after,
                            HostSessionPostureV1::ActiveAttached,
                        ),
                        HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                            authority_revision_after,
                            resulting_posture,
                            ..
                        } => (*authority_revision_after, *resulting_posture),
                        HostSessionStartupOwnershipApplicationV1::NotApplicable => unreachable!(),
                    },
                    HostSessionTransitionModeV1::ResumeOneTurn => match post_turn.as_ref() {
                        HostSessionPostTurnApplicationV2::Pending { .. }
                        | HostSessionPostTurnApplicationV2::AwaitingObligationCut { .. } => (
                            *authority_revision_after,
                            HostSessionPostureV1::ActiveAttached,
                        ),
                        HostSessionPostTurnApplicationV2::Applied {
                            authority_revision_after,
                            resulting_posture,
                            ..
                        } => (*authority_revision_after, *resulting_posture),
                        HostSessionPostTurnApplicationV2::NotApplicable => unreachable!(),
                    },
                    HostSessionTransitionModeV1::Start => unreachable!(),
                };
                if applied_state.authority_record_commitment != *authority_record_commitment
                    || applied_state.authority.authority_revision != *authority_revision_after
                    || applied_state.authority.lifecycle_posture
                        != HostSessionPostureV1::ActiveAttached
                    || applied_state
                        .authority
                        .active_authoritative_participant_id
                        .as_deref()
                        != Some(intent.target_authoritative_participant_id.as_str())
                    || applied_state
                        .authority
                        .authoritative_participant_lineage
                        .last()
                        != Some(&intent.target_authoritative_participant_id)
                    || !applied_state
                        .authority
                        .authoritative_participant_lineage
                        .iter()
                        .any(|participant| participant == active_authoritative_participant_id)
                    || intent.resume_handle_ref.as_ref().is_some_and(|reference| {
                        !applied_state
                            .authority
                            .internal_resume_handle_refs
                            .iter()
                            .any(|current| current == reference)
                    })
                {
                    return Err(StoreSchemaError(
                        "applied V3 successor authority does not match intent state",
                    ));
                }
                let journal = self
                    .successor_application_journal
                    .get(&intent.intent_id)
                    .ok_or(StoreSchemaError(
                        "applied V3 successor has no application journal",
                    ))?;
                self.validate_successor_application_journal(intent, journal)?;
                let current_state =
                    history
                        .get(&expected_current_revision)
                        .ok_or(StoreSchemaError(
                            "applied V3 successor phase has no reconstructed authority state",
                        ))?;
                if current_state.authority.lifecycle_posture != expected_current_posture {
                    return Err(StoreSchemaError(
                        "applied V3 successor terminal phase conflicts with reconstructed history",
                    ));
                }
            }
            HostSessionTransitionIntentStateV3::Rejected {
                terminal_handoff_ref,
                ..
            }
            | HostSessionTransitionIntentStateV3::Expired {
                terminal_handoff_ref,
                ..
            } => {
                if terminal_handoff_ref.object_kind != AuthorityObjectKindV1::TerminalHandoff
                    || self
                        .successor_application_journal
                        .contains_key(&intent.intent_id)
                {
                    return Err(StoreSchemaError("terminal V3 successor state is invalid"));
                }
            }
        }
        let transport_index = self
            .object_index
            .get(&intent.transport_payload_ref.ref_id)
            .ok_or(StoreSchemaError(
                "V3 successor transport payload has no object index entry",
            ))?;
        if transport_index.object_kind != AuthorityObjectKindV1::TransitionTransportPayload
            || !transport_states_match(
                &intent.transport_payload_state,
                &transport_index.storage_state,
            )
        {
            return Err(StoreSchemaError(
                "V3 successor transport parent and object index disagree",
            ));
        }
        Ok(())
    }

    fn validate_successor_application_journal(
        &self,
        intent: &HostSessionTransitionIntentV3,
        journal: &HostSessionTransitionApplicationJournalV3,
    ) -> Result<(), StoreSchemaError> {
        let HostSessionTransitionIntentStateV3::Applied {
            authority_revision_before,
            authority_revision_after,
            authority_record_commitment,
            application_result_ref,
            startup_ownership,
            post_turn,
            ..
        } = &intent.state
        else {
            return Err(StoreSchemaError(
                "non-applied V3 successor has an application journal",
            ));
        };
        if journal.initial_application.authority_revision_before != *authority_revision_before
            || journal.initial_application.authority_revision_after != *authority_revision_after
            || journal.initial_application.authority_record_commitment
                != *authority_record_commitment
            || journal.initial_application.application_result_ref != *application_result_ref
        {
            return Err(StoreSchemaError(
                "V3 successor initial application and journal disagree",
            ));
        }
        match intent.mode {
            HostSessionTransitionModeV1::Attach => {
                match startup_ownership.as_ref() {
                    HostSessionStartupOwnershipApplicationV1::Pending { .. }
                    | HostSessionStartupOwnershipApplicationV1::Accepted { .. } => {
                        if journal.startup_terminal_application.is_some() {
                            return Err(StoreSchemaError(
                                "nonterminal V3 Attach startup ownership has a terminal journal",
                            ));
                        }
                    }
                    HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                        result_ref,
                        evidence_id,
                        authority_revision_before,
                        authority_revision_after,
                        resulting_posture,
                        ..
                    } => {
                        let Some(terminal) = &journal.startup_terminal_application else {
                            return Err(StoreSchemaError(
                                "terminal V3 Attach startup ownership has no journal",
                            ));
                        };
                        if terminal.startup_ownership_result_ref != *result_ref
                            || terminal.evidence_id != *evidence_id
                            || terminal.authority_revision_before != *authority_revision_before
                            || terminal.authority_revision_after != *authority_revision_after
                            || terminal.resulting_posture != *resulting_posture
                        {
                            return Err(StoreSchemaError(
                                "V3 Attach startup terminal journal disagrees with intent state",
                            ));
                        }
                    }
                    HostSessionStartupOwnershipApplicationV1::NotApplicable => {
                        return Err(StoreSchemaError(
                            "V3 Attach startup ownership cannot be NotApplicable",
                        ))
                    }
                }
                if post_turn.as_ref() != &HostSessionPostTurnApplicationV2::NotApplicable
                    || journal.post_turn_application.is_some()
                {
                    return Err(StoreSchemaError(
                        "V3 Attach post-turn journal state is invalid",
                    ));
                }
            }
            HostSessionTransitionModeV1::ResumeOneTurn => {
                if startup_ownership.as_ref()
                    != &HostSessionStartupOwnershipApplicationV1::NotApplicable
                    || journal.startup_terminal_application.is_some()
                {
                    return Err(StoreSchemaError(
                        "V3 Resume startup journal state is invalid",
                    ));
                }
                match post_turn.as_ref() {
                    HostSessionPostTurnApplicationV2::Pending { .. }
                    | HostSessionPostTurnApplicationV2::AwaitingObligationCut { .. } => {
                        if journal.post_turn_application.is_some() {
                            return Err(StoreSchemaError(
                                "nonterminal V3 Resume post-turn has an application journal",
                            ));
                        }
                    }
                    HostSessionPostTurnApplicationV2::Applied {
                        completion_ref,
                        obligation_snapshot_ref,
                        authority_revision_before,
                        authority_revision_after,
                        application_result_ref,
                        ..
                    } => {
                        let Some(post_turn_journal) = &journal.post_turn_application else {
                            return Err(StoreSchemaError(
                                "applied V3 Resume post-turn has no journal",
                            ));
                        };
                        if post_turn_journal.completion_ref != **completion_ref
                            || post_turn_journal.obligation_snapshot_ref.as_ref()
                                != obligation_snapshot_ref.as_deref()
                            || post_turn_journal.authority_revision_before
                                != *authority_revision_before
                            || post_turn_journal.authority_revision_after
                                != *authority_revision_after
                            || post_turn_journal.application_result_ref != **application_result_ref
                        {
                            return Err(StoreSchemaError(
                                "V3 Resume post-turn journal disagrees with intent state",
                            ));
                        }
                    }
                    HostSessionPostTurnApplicationV2::NotApplicable => {
                        return Err(StoreSchemaError(
                            "V3 Resume post-turn cannot be NotApplicable",
                        ))
                    }
                }
            }
            HostSessionTransitionModeV1::Start => {
                return Err(StoreSchemaError(
                    "V3 successor journal cannot belong to Start",
                ))
            }
        }
        Ok(())
    }

    fn has_any_successor_transition(&self, orchestration_session_id: &str) -> bool {
        self.successor_transition_intent_map
            .values()
            .any(|intent| intent.orchestration_session_id == orchestration_session_id)
    }

    fn reconstruct_v2_authority_history(
        &self,
        current_authority: &DurableSessionAuthorityV1,
    ) -> Result<BTreeMap<u64, ReconstructedAuthorityStateV1>, StoreSchemaError> {
        let DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            payload_commitment,
        } = &current_authority.origin;
        let intent = self
            .transition_intent_map
            .get(intent_id)
            .ok_or(StoreSchemaError(
                "V3 preserved V2 history has no origin Start intent",
            ))?;
        let HostSessionTransitionIntentStateV2::Applied {
            authority_revision_before: None,
            authority_revision_after,
            active_authoritative_participant_id,
            resulting_posture,
            authority_record_commitment,
            applied_at,
            ..
        } = &intent.state
        else {
            return Err(StoreSchemaError(
                "V3 preserved V2 history requires an applied origin Start intent",
            ));
        };
        if intent.issuer_request_id != *issuer_request_id
            || intent.payload_commitment != *payload_commitment
            || intent.orchestration_session_id != current_authority.orchestration_session_id
            || *authority_revision_after != 1
            || active_authoritative_participant_id != &intent.target_authoritative_participant_id
            || *resulting_posture != HostSessionPostureV1::ActiveAttached
        {
            return Err(StoreSchemaError(
                "V3 preserved V2 Start origin is inconsistent",
            ));
        }
        let journal = self
            .application_journal
            .get(intent_id)
            .ok_or(StoreSchemaError(
                "V3 preserved V2 history has no origin Start journal",
            ))?;
        if journal
            .initial_application
            .authority_revision_before
            .is_some()
            || journal.initial_application.authority_revision_after != 1
            || journal.initial_application.authority_record_commitment
                != *authority_record_commitment
            || journal.initial_application.applied_at != *applied_at
        {
            return Err(StoreSchemaError(
                "V3 preserved V2 Start journal disagrees with its origin intent",
            ));
        }
        let mut history = BTreeMap::new();
        let initial = reconstruct_authority_state(DurableSessionAuthorityV1 {
            schema_version: 1,
            orchestration_session_id: current_authority.orchestration_session_id.clone(),
            shell_trace_session_id: current_authority.shell_trace_session_id.clone(),
            authority_revision: 1,
            origin: current_authority.origin.clone(),
            authoritative_participant_lineage: intent.resulting_authoritative_lineage.clone(),
            active_authoritative_participant_id: Some(
                intent.target_authoritative_participant_id.clone(),
            ),
            workspace_binding: current_authority.workspace_binding.clone(),
            world_binding: current_authority.world_binding.clone(),
            host_attach_contract_ref: current_authority.host_attach_contract_ref.clone(),
            retained_worker_refs: Vec::new(),
            internal_resume_handle_refs: Vec::new(),
            lifecycle_posture: HostSessionPostureV1::ActiveAttached,
            current_policy_ref: current_authority.current_policy_ref.clone(),
            current_policy_revision: current_authority.current_policy_revision.clone(),
            updated_at: journal.initial_application.applied_at.clone(),
        })?;
        if initial.authority_record_commitment
            != journal.initial_application.authority_record_commitment
        {
            return Err(StoreSchemaError(
                "V3 preserved V2 Start authority commitment is inconsistent",
            ));
        }
        history.insert(initial.authority.authority_revision, initial.clone());
        let session_registration_count = self
            .retained_worker_registration_journal
            .values()
            .filter(|registration| {
                registration.orchestration_session_id == current_authority.orchestration_session_id
            })
            .count();
        let mut consumed = 0_usize;
        let mut latest = initial;
        while consumed < session_registration_count {
            let candidates = self
                .retained_worker_registration_journal
                .values()
                .filter(|registration| {
                    registration.orchestration_session_id
                        == current_authority.orchestration_session_id
                        && registration.authority_revision_before
                            == latest.authority.authority_revision
                        && registration.authority_record_commitment_before
                            == latest.authority_record_commitment
                })
                .collect::<Vec<_>>();
            let [registration] = candidates.as_slice() else {
                return Err(StoreSchemaError(
                    "V3 preserved V2 retained authority ancestry is not uniquely contiguous",
                ));
            };
            let request = self
                .retained_worker_registration_request_index
                .get(&registration.issuer_request_id)
                .ok_or(StoreSchemaError(
                    "V3 preserved V2 retained registration has no request record",
                ))?;
            validate_applied_registration_request(request, registration)?;
            let next = reconstruct_authority_state(DurableSessionAuthorityV1 {
                schema_version: latest.authority.schema_version,
                orchestration_session_id: latest.authority.orchestration_session_id.clone(),
                shell_trace_session_id: latest.authority.shell_trace_session_id.clone(),
                authority_revision: registration.authority_revision_after,
                origin: latest.authority.origin.clone(),
                authoritative_participant_lineage: {
                    let mut lineage = latest.authority.authoritative_participant_lineage.clone();
                    lineage.push(registration.retained_participant_id.clone());
                    lineage
                },
                active_authoritative_participant_id: latest
                    .authority
                    .active_authoritative_participant_id
                    .clone(),
                workspace_binding: latest.authority.workspace_binding.clone(),
                world_binding: latest.authority.world_binding.clone(),
                host_attach_contract_ref: latest.authority.host_attach_contract_ref.clone(),
                retained_worker_refs: {
                    let mut refs = latest.authority.retained_worker_refs.clone();
                    refs.push(registration.retained_worker_ref.clone());
                    refs
                },
                internal_resume_handle_refs: Vec::new(),
                lifecycle_posture: latest.authority.lifecycle_posture,
                current_policy_ref: latest.authority.current_policy_ref.clone(),
                current_policy_revision: latest.authority.current_policy_revision.clone(),
                updated_at: registration.registered_at.clone(),
            })?;
            if next.authority_record_commitment != registration.authority_record_commitment_after
                || next.authoritative_lineage_commitment
                    != registration.authoritative_lineage_commitment_after
            {
                return Err(StoreSchemaError(
                    "V3 preserved V2 retained authority reconstruction is inconsistent",
                ));
            }
            history.insert(next.authority.authority_revision, next.clone());
            latest = next;
            consumed += 1;
        }
        Ok(history)
    }

    fn reconstruct_successor_authority_history(
        &self,
        current_authority: &DurableSessionAuthorityV1,
    ) -> Result<BTreeMap<u64, ReconstructedAuthorityStateV1>, StoreSchemaError> {
        let mut history = self.reconstruct_v2_runtime_authority_history(current_authority)?;
        let mut consumed = std::collections::BTreeSet::new();
        loop {
            let (latest_revision, latest_state) = history
                .last_key_value()
                .ok_or(StoreSchemaError("V3 successor authority history is empty"))?;
            let candidates = self
                .successor_transition_intent_map
                .values()
                .filter(|intent| {
                    intent.orchestration_session_id == current_authority.orchestration_session_id
                        && matches!(
                            &intent.state,
                            HostSessionTransitionIntentStateV3::Applied {
                                authority_revision_before,
                                ..
                            } if authority_revision_before == &Some(*latest_revision)
                        )
                })
                .collect::<Vec<_>>();
            let intent = match candidates.as_slice() {
                [] => break,
                [intent] => *intent,
                _ => return Err(StoreSchemaError("V3 applied successor chain is ambiguous")),
            };
            let journal = self
                .successor_application_journal
                .get(&intent.intent_id)
                .ok_or(StoreSchemaError(
                    "applied V3 successor has no application journal",
                ))?;
            let HostSessionTransitionIntentStateV3::Applied {
                authority_revision_before,
                authority_revision_after,
                authority_record_commitment,
                startup_ownership,
                post_turn,
                ..
            } = &intent.state
            else {
                unreachable!()
            };
            if authority_revision_before != &Some(*latest_revision)
                || journal.initial_application.authority_revision_before
                    != *authority_revision_before
                || journal.initial_application.authority_revision_after != *authority_revision_after
                || journal.initial_application.authority_record_commitment
                    != *authority_record_commitment
            {
                return Err(StoreSchemaError(
                    "applied V3 successor journal disagrees with initial application state",
                ));
            }
            let initial_state = reconstruct_successor_initial_state(
                latest_state,
                intent,
                &journal.initial_application.applied_at,
            )?;
            if initial_state.authority_record_commitment != *authority_record_commitment {
                return Err(StoreSchemaError(
                    "applied V3 successor initial authority commitment is inconsistent",
                ));
            }
            history.insert(
                initial_state.authority.authority_revision,
                initial_state.clone(),
            );
            match (intent.mode, startup_ownership.as_ref(), post_turn.as_ref()) {
                (
                    HostSessionTransitionModeV1::Attach,
                    HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                        authority_revision_after,
                        resulting_posture,
                        ..
                    },
                    HostSessionPostTurnApplicationV2::NotApplicable,
                ) => {
                    let terminal =
                        journal
                            .startup_terminal_application
                            .as_ref()
                            .ok_or(StoreSchemaError(
                                "terminal V3 Attach startup ownership has no journal",
                            ))?;
                    let reconciled = reconstruct_reconciled_authority_state(
                        &initial_state,
                        *authority_revision_after,
                        terminal.authority_record_commitment_after.clone(),
                        *resulting_posture,
                        &terminal.applied_at,
                    )?;
                    history.insert(reconciled.authority.authority_revision, reconciled);
                }
                (
                    HostSessionTransitionModeV1::ResumeOneTurn,
                    HostSessionStartupOwnershipApplicationV1::NotApplicable,
                    HostSessionPostTurnApplicationV2::Applied {
                        authority_revision_after,
                        resulting_posture,
                        applied_at,
                        ..
                    },
                ) => {
                    let post_turn_journal =
                        journal
                            .post_turn_application
                            .as_ref()
                            .ok_or(StoreSchemaError(
                                "applied V3 Resume post-turn has no journal",
                            ))?;
                    let reconciled = reconstruct_reconciled_authority_state(
                        &initial_state,
                        *authority_revision_after,
                        post_turn_journal.authority_record_commitment.clone(),
                        *resulting_posture,
                        applied_at,
                    )?;
                    history.insert(reconciled.authority.authority_revision, reconciled);
                }
                _ => {}
            }
            consumed.insert(intent.intent_id.clone());
        }
        let applied_count = self
            .successor_transition_intent_map
            .values()
            .filter(|intent| {
                intent.orchestration_session_id == current_authority.orchestration_session_id
                    && matches!(
                        intent.state,
                        HostSessionTransitionIntentStateV3::Applied { .. }
                    )
            })
            .count();
        if consumed.len() != applied_count {
            return Err(StoreSchemaError(
                "V3 applied successor chain is disconnected from current authority",
            ));
        }
        Ok(history)
    }

    fn reconstruct_v2_runtime_authority_history(
        &self,
        current_authority: &DurableSessionAuthorityV1,
    ) -> Result<BTreeMap<u64, ReconstructedAuthorityStateV1>, StoreSchemaError> {
        let mut history = self.reconstruct_v2_authority_history(current_authority)?;
        let DurableSessionAuthorityOriginV1::StartIntent { intent_id, .. } =
            &current_authority.origin;
        let intent = self
            .transition_intent_map
            .get(intent_id)
            .ok_or(StoreSchemaError(
                "V3 runtime V2 history has no origin Start intent",
            ))?;
        let HostSessionTransitionIntentStateV2::Applied {
            startup_ownership, ..
        } = &intent.state
        else {
            return Err(StoreSchemaError(
                "V3 runtime V2 history requires an applied origin Start intent",
            ));
        };
        let latest = history
            .last_key_value()
            .map(|(_, state)| state.clone())
            .ok_or(StoreSchemaError("V3 runtime V2 history is empty"))?;
        let journal = self
            .application_journal
            .get(intent_id)
            .ok_or(StoreSchemaError(
                "V3 runtime V2 history has no origin Start journal",
            ))?;
        match (
            startup_ownership.as_ref(),
            journal.startup_terminal_application.as_ref(),
        ) {
            (HostSessionStartupOwnershipApplicationV1::Pending { .. }, None)
            | (HostSessionStartupOwnershipApplicationV1::Accepted { .. }, None) => {
                let start_continuation_refs = current_authority
                    .internal_resume_handle_refs
                    .iter()
                    .filter(|reference| {
                        reference.object_kind == AuthorityObjectKindV1::ResumeHandle
                            && reference.schema_version == 2
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if start_continuation_refs.is_empty() {
                    return Ok(history);
                }
                if start_continuation_refs.len() > 2 {
                    return Err(StoreSchemaError(
                        "V3 Start continuation history has too many handle phases",
                    ));
                }
                let expected_revision = latest
                    .authority
                    .authority_revision
                    .checked_add(start_continuation_refs.len() as u64)
                    .ok_or(StoreSchemaError(
                        "V3 Start continuation authority revision overflow",
                    ))?;
                let earliest_successor = self
                    .successor_transition_intent_map
                    .values()
                    .filter(|successor| {
                        successor.orchestration_session_id
                            == current_authority.orchestration_session_id
                    })
                    .filter_map(|successor| match &successor.authority_precondition {
                        HostSessionAuthorityPreconditionV1::ExpectedRevision {
                            authority_revision,
                            authority_record_commitment,
                            active_authoritative_participant_id,
                            authoritative_lineage_commitment,
                            lifecycle_posture,
                        } => Some((
                            *authority_revision,
                            authority_record_commitment,
                            active_authoritative_participant_id,
                            authoritative_lineage_commitment,
                            *lifecycle_posture,
                        )),
                        HostSessionAuthorityPreconditionV1::ExpectedAbsent => None,
                    })
                    .min_by_key(|(revision, ..)| *revision);
                let reconstructed = if let Some((
                    revision,
                    authority_record_commitment,
                    active_participant,
                    lineage_commitment,
                    lifecycle_posture,
                )) = earliest_successor
                {
                    if revision != expected_revision {
                        return Err(StoreSchemaError(
                            "V3 successor does not descend from settled Start continuity",
                        ));
                    }
                    let authority = if current_authority.authority_revision == revision {
                        current_authority.clone()
                    } else {
                        let mut authority = latest.authority.clone();
                        authority.authority_revision = revision;
                        authority.active_authoritative_participant_id =
                            Some(active_participant.clone());
                        authority.internal_resume_handle_refs = start_continuation_refs.clone();
                        authority.lifecycle_posture = lifecycle_posture;
                        authority
                    };
                    let reconstructed = reconstruct_authority_state(authority)?;
                    if &reconstructed.authority_record_commitment != authority_record_commitment
                        || &reconstructed.authoritative_lineage_commitment != lineage_commitment
                    {
                        return Err(StoreSchemaError(
                            "V3 Start continuation does not match successor precondition",
                        ));
                    }
                    reconstructed
                } else {
                    if current_authority.authority_revision != expected_revision
                        || current_authority.internal_resume_handle_refs != start_continuation_refs
                    {
                        return Err(StoreSchemaError(
                            "current V3 Start continuation authority is not contiguous",
                        ));
                    }
                    reconstruct_authority_state(current_authority.clone())?
                };
                if (start_continuation_refs.len() == 1
                    && reconstructed.authority.lifecycle_posture
                        != HostSessionPostureV1::ActiveAttached)
                    || (start_continuation_refs.len() == 2
                        && !matches!(
                            reconstructed.authority.lifecycle_posture,
                            HostSessionPostureV1::ParkedResumable
                                | HostSessionPostureV1::AwaitingAttention
                                | HostSessionPostureV1::Terminal
                        ))
                {
                    return Err(StoreSchemaError(
                        "V3 Start continuation posture is inconsistent with its phases",
                    ));
                }
                history.insert(reconstructed.authority.authority_revision, reconstructed);
                Ok(history)
            }
            (
                HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                    authority_revision_before,
                    authority_revision_after,
                    resulting_posture,
                    ..
                },
                Some(terminal),
            ) => {
                if terminal.authority_revision_before != *authority_revision_before
                    || terminal.authority_record_commitment_before
                        != latest.authority_record_commitment
                    || *authority_revision_before != latest.authority.authority_revision
                    || terminal.authority_revision_after != *authority_revision_after
                    || terminal.resulting_posture != *resulting_posture
                {
                    return Err(StoreSchemaError(
                        "V3 runtime V2 startup terminal proof is inconsistent",
                    ));
                }
                let reconciled = self.reconstruct_v2_runtime_terminal_state(
                    current_authority,
                    &latest,
                    *authority_revision_after,
                    terminal.authority_record_commitment_after.clone(),
                    *resulting_posture,
                    &terminal.applied_at,
                )?;
                history.insert(reconciled.authority.authority_revision, reconciled);
                Ok(history)
            }
            (HostSessionStartupOwnershipApplicationV1::NotApplicable, _)
            | (HostSessionStartupOwnershipApplicationV1::Pending { .. }, Some(_))
            | (HostSessionStartupOwnershipApplicationV1::Accepted { .. }, Some(_))
            | (HostSessionStartupOwnershipApplicationV1::TerminalReconciled { .. }, None) => Err(
                StoreSchemaError("V3 runtime V2 startup ownership state is inconsistent"),
            ),
        }
    }

    fn reconstruct_v2_runtime_terminal_state(
        &self,
        current_authority: &DurableSessionAuthorityV1,
        latest: &ReconstructedAuthorityStateV1,
        authority_revision_after: u64,
        expected_commitment: AuthorityObjectCommitmentV1,
        resulting_posture: HostSessionPostureV1,
        applied_at: &TimestampV1,
    ) -> Result<ReconstructedAuthorityStateV1, StoreSchemaError> {
        if let Ok(reconciled) = reconstruct_reconciled_authority_state(
            latest,
            authority_revision_after,
            expected_commitment.clone(),
            resulting_posture,
            applied_at,
        ) {
            return Ok(reconciled);
        }

        let mut authority = latest.authority.clone();
        authority.authority_revision = authority_revision_after;
        authority.lifecycle_posture = resulting_posture;
        authority.updated_at = applied_at.clone();
        authority.internal_resume_handle_refs =
            self.runtime_resume_handles_for_revision(current_authority, authority_revision_after);
        let reconstructed = reconstruct_authority_state(authority)?;
        if reconstructed.authority_record_commitment != expected_commitment {
            return Err(StoreSchemaError(
                "reconciled V3 authority commitment is inconsistent",
            ));
        }
        Ok(reconstructed)
    }

    fn runtime_resume_handles_for_revision(
        &self,
        current_authority: &DurableSessionAuthorityV1,
        authority_revision: u64,
    ) -> Vec<AuthorityObjectRefV1> {
        let referenced = self
            .successor_transition_intent_map
            .values()
            .filter(|intent| {
                intent.orchestration_session_id == current_authority.orchestration_session_id
                    && matches!(
                        &intent.authority_precondition,
                        HostSessionAuthorityPreconditionV1::ExpectedRevision {
                            authority_revision: revision,
                            ..
                        } if *revision == authority_revision
                    )
            })
            .filter_map(|intent| {
                intent
                    .resume_handle_ref
                    .as_ref()
                    .map(|reference| (reference.ref_id.clone(), reference.clone()))
            })
            .collect::<BTreeMap<_, _>>();

        if referenced.is_empty() {
            return current_authority.internal_resume_handle_refs.clone();
        }

        let mut ordered = current_authority
            .internal_resume_handle_refs
            .iter()
            .filter(|reference| referenced.contains_key(&reference.ref_id))
            .cloned()
            .collect::<Vec<_>>();
        for (ref_id, reference) in referenced {
            if ordered.iter().any(|current| current.ref_id == ref_id) {
                continue;
            }
            ordered.push(reference);
        }
        ordered
    }
}

fn validate_registration_commitment(
    commitment: &AuthorityObjectCommitmentV1,
) -> Result<(), StoreSchemaError> {
    validate_object_commitment_rule(AuthorityObjectKindV1::AgentDescriptor, 1, commitment)
        .map_err(|_| StoreSchemaError("retained registration requires canonical commitment"))
}

fn validate_registration_ref(
    reference: &AuthorityObjectRefV1,
    expected_kind: AuthorityObjectKindV1,
) -> Result<(), StoreSchemaError> {
    validate_ref_id(&reference.ref_id)
        .map_err(|_| StoreSchemaError("retained registration object ref ID is invalid"))?;
    if reference.schema_version != 1 || reference.object_kind != expected_kind {
        return Err(StoreSchemaError(
            "retained registration object ref kind or version is invalid",
        ));
    }
    validate_object_commitment_rule(
        reference.object_kind,
        reference.schema_version,
        &reference.commitment,
    )
    .map_err(|_| StoreSchemaError("retained registration object ref commitment is invalid"))
}

fn validate_applied_registration_request(
    request: &RetainedWorkerAuthorityRegistrationRequestV1,
    journal: &RetainedWorkerAuthorityRegistrationV1,
) -> Result<(), StoreSchemaError> {
    let RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
        authority_revision_after,
        authority_record_commitment_after,
    } = &request.state
    else {
        return Err(StoreSchemaError(
            "retained registration journal requires applied request",
        ));
    };
    if journal.issuer_request_id != request.issuer_request_id
        || journal.registration_id != request.registration_id
        || journal.orchestration_session_id != request.orchestration_session_id
        || journal.authority_revision_before != request.authority_revision_before
        || journal.authority_record_commitment_before != request.authority_record_commitment_before
        || journal.authority_revision_after != *authority_revision_after
        || journal.authority_record_commitment_after != *authority_record_commitment_after
        || journal.retained_participant_id != request.retained_participant_id
        || journal.descriptor_ref.ref_id != request.descriptor_ref_id
        || journal.descriptor_ref.commitment != request.descriptor_commitment
        || journal.resume_handle_ref.ref_id != request.resume_handle_ref_id
        || journal.resume_handle_ref.commitment != request.resume_handle_commitment
        || journal.retained_worker_ref.ref_id != request.retained_worker_ref_id
        || journal.retained_worker_ref.commitment != request.retained_worker_commitment
        || journal.current_policy_ref != request.current_policy_ref
        || journal.world_binding != request.world_binding
        || journal.registered_at != request.registered_at
    {
        return Err(StoreSchemaError(
            "retained registration request and journal disagree",
        ));
    }
    validate_registration_ref(
        &journal.descriptor_ref,
        AuthorityObjectKindV1::AgentDescriptor,
    )?;
    validate_registration_ref(
        &journal.resume_handle_ref,
        AuthorityObjectKindV1::ResumeHandle,
    )?;
    validate_registration_ref(
        &journal.retained_worker_ref,
        AuthorityObjectKindV1::RetainedWorker,
    )?;
    validate_registration_ref(&journal.current_policy_ref, AuthorityObjectKindV1::Policy)?;
    validate_registration_commitment(&journal.authority_record_commitment_before)?;
    validate_registration_commitment(&journal.authority_record_commitment_after)?;
    validate_registration_commitment(&journal.authoritative_lineage_commitment_after)
}

fn authority_record_commitment(
    authority: &DurableSessionAuthorityV1,
) -> Result<AuthorityObjectCommitmentV1, StoreSchemaError> {
    super::hash::canonical_sha256(&DurableSessionAuthorityHashInputV1 {
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
    })
    .map(|digest_hex| AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex })
    .map_err(|_| StoreSchemaError("commit durable retained authority"))
}

fn reconstruct_authority_state(
    authority: DurableSessionAuthorityV1,
) -> Result<ReconstructedAuthorityStateV1, StoreSchemaError> {
    let authority_record_commitment = authority_record_commitment(&authority)?;
    let authoritative_lineage_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: super::hash::canonical_sha256(&AuthoritativeLineageHashInputV1 {
            schema_version: 1,
            orchestration_session_id: authority.orchestration_session_id.clone(),
            participant_ids: authority.authoritative_participant_lineage.clone(),
        })
        .map_err(|_| StoreSchemaError("commit durable authority lineage"))?,
    };
    Ok(ReconstructedAuthorityStateV1 {
        authority,
        authority_record_commitment,
        authoritative_lineage_commitment,
    })
}

fn reconstruct_successor_initial_state(
    latest_state: &ReconstructedAuthorityStateV1,
    intent: &HostSessionTransitionIntentV3,
    applied_at: &TimestampV1,
) -> Result<ReconstructedAuthorityStateV1, StoreSchemaError> {
    let mut authority = latest_state.authority.clone();
    authority.authority_revision = match &intent.state {
        HostSessionTransitionIntentStateV3::Applied {
            authority_revision_after,
            ..
        } => *authority_revision_after,
        _ => {
            return Err(StoreSchemaError(
                "reconstruct successor initial state requires an applied intent",
            ))
        }
    };
    authority.active_authoritative_participant_id =
        Some(intent.target_authoritative_participant_id.clone());
    authority
        .authoritative_participant_lineage
        .push(intent.target_authoritative_participant_id.clone());
    authority.host_attach_contract_ref = Some(intent.host_attach_contract_ref.clone());
    authority.world_binding = intent.world_binding.clone();
    if let Some(resume_handle_ref) = intent.resume_handle_ref.as_ref() {
        if !authority
            .internal_resume_handle_refs
            .iter()
            .any(|current| current == resume_handle_ref)
        {
            authority
                .internal_resume_handle_refs
                .push(resume_handle_ref.clone());
        }
    }
    authority.lifecycle_posture = HostSessionPostureV1::ActiveAttached;
    authority.updated_at = applied_at.clone();
    reconstruct_authority_state(authority)
}

fn reconstruct_reconciled_authority_state(
    latest_state: &ReconstructedAuthorityStateV1,
    authority_revision_after: u64,
    expected_commitment: AuthorityObjectCommitmentV1,
    resulting_posture: HostSessionPostureV1,
    applied_at: &TimestampV1,
) -> Result<ReconstructedAuthorityStateV1, StoreSchemaError> {
    let mut authority = latest_state.authority.clone();
    authority.authority_revision = authority_revision_after;
    authority.lifecycle_posture = resulting_posture;
    authority.updated_at = applied_at.clone();
    let reconstructed = reconstruct_authority_state(authority)?;
    if reconstructed.authority_record_commitment != expected_commitment {
        return Err(StoreSchemaError(
            "reconciled V3 authority commitment is inconsistent",
        ));
    }
    Ok(reconstructed)
}

fn all_unique(values: &[String]) -> bool {
    values
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == values.len()
}

fn all_unique_refs(values: &[AuthorityObjectRefV1]) -> bool {
    values
        .iter()
        .map(|reference| &reference.ref_id)
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == values.len()
}

fn validate_namespace_record_identity(
    record: &SessionNamespaceRecordV1,
    authority_store_id: &str,
    bootstrap_home: &CanonicalDirectoryV1,
    allow_internal_resume_handles: bool,
) -> Result<(), StoreSchemaError> {
    match record {
        SessionNamespaceRecordV1::Authority(authority) => {
            require_version(authority.schema_version)?;
            required(&authority.orchestration_session_id)?;
            required(&authority.shell_trace_session_id)?;
            if authority.authority_revision == 0
                || authority.workspace_binding.authority_store_id != authority_store_id
                || &authority.workspace_binding.authority_store_root != bootstrap_home
                || authority.authoritative_participant_lineage.is_empty()
                || !all_unique(&authority.authoritative_participant_lineage)
                || !all_unique_refs(&authority.retained_worker_refs)
                || !all_unique_refs(&authority.internal_resume_handle_refs)
                || authority
                    .active_authoritative_participant_id
                    .as_ref()
                    .is_some_and(|active| {
                        !authority.authoritative_participant_lineage.contains(active)
                    })
                || (!allow_internal_resume_handles
                    && !authority.internal_resume_handle_refs.is_empty())
            {
                return Err(StoreSchemaError(if allow_internal_resume_handles {
                    "V3 durable authority binding mismatch"
                } else {
                    "V2 durable authority binding mismatch"
                }));
            }
        }
        SessionNamespaceRecordV1::StartReservation(reservation) => {
            require_version(reservation.schema_version)?;
            required(&reservation.orchestration_session_id)?;
            required(&reservation.intent_id)?;
            required(&reservation.issuer_request_id)?;
        }
        SessionNamespaceRecordV1::StartTombstone(tombstone) => {
            require_version(tombstone.schema_version)?;
            required(&tombstone.orchestration_session_id)?;
            required(&tombstone.intent_id)?;
            required(&tombstone.issuer_request_id)?;
            if tombstone.terminal_handoff_ref.object_kind != AuthorityObjectKindV1::TerminalHandoff
            {
                return Err(StoreSchemaError("V2 Start tombstone ref kind is invalid"));
            }
        }
    }
    Ok(())
}

fn v3_object_schema_version_allowed(
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
) -> bool {
    schema_version == SCHEMA_VERSION
        || (matches!(
            object_kind,
            AuthorityObjectKindV1::TerminalHandoff | AuthorityObjectKindV1::ResumeHandle
        ) && schema_version == 2)
}

fn reservation_matches_v2(
    reservation: &SessionIdReservationV1,
    intent: &HostSessionTransitionIntentV2,
) -> bool {
    reservation.orchestration_session_id == intent.orchestration_session_id
        && reservation.intent_id == intent.intent_id
        && reservation.issuer_request_id == intent.issuer_request_id
        && reservation.payload_commitment == intent.payload_commitment
}

fn v2_start_input_is_pending(intent: &HostSessionTransitionIntentV2) -> bool {
    matches!(
        (&intent.transition_input_ref, &intent.input_handoff),
        (None, HostSessionTransitionInputHandoffV1::NotApplicable)
            | (Some(_), HostSessionTransitionInputHandoffV1::Pending { .. })
    )
}

fn v2_start_input_is_applied_pending(intent: &HostSessionTransitionIntentV2) -> bool {
    matches!(
        (&intent.transition_input_ref, &intent.input_handoff),
        (None, HostSessionTransitionInputHandoffV1::NotApplicable)
            | (
                Some(_),
                HostSessionTransitionInputHandoffV1::Pending { .. }
                    | HostSessionTransitionInputHandoffV1::Accepted { .. }
            )
    )
}

fn v2_start_input_is_terminal(intent: &HostSessionTransitionIntentV2) -> bool {
    matches!(
        (&intent.transition_input_ref, &intent.input_handoff),
        (None, HostSessionTransitionInputHandoffV1::NotApplicable)
            | (
                Some(_),
                HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance { .. }
            )
    )
}

fn authority_origin_matches_v2(
    authority: &DurableSessionAuthorityV1,
    intent: &HostSessionTransitionIntentV2,
) -> bool {
    matches!(
        &authority.origin,
        DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            payload_commitment,
        } if intent_id == &intent.intent_id
            && issuer_request_id == &intent.issuer_request_id
            && payload_commitment == &intent.payload_commitment
            && authority.orchestration_session_id == intent.orchestration_session_id
    )
}

fn tombstone_matches_rejected_v2(
    tombstone: &SessionIdTombstoneV1,
    intent: &HostSessionTransitionIntentV2,
    reason: HostSessionTransitionTerminalRejectionV1,
    terminal_handoff_ref: &AuthorityObjectRefV1,
) -> bool {
    reservation_fields_match_tombstone_v2(tombstone, intent)
        && tombstone.terminal_handoff_ref == *terminal_handoff_ref
        && tombstone.terminal_state == StartTombstoneStateV1::Rejected { reason }
}

fn tombstone_matches_expired_v2(
    tombstone: &SessionIdTombstoneV1,
    intent: &HostSessionTransitionIntentV2,
    terminal_handoff_ref: &AuthorityObjectRefV1,
) -> bool {
    reservation_fields_match_tombstone_v2(tombstone, intent)
        && tombstone.terminal_handoff_ref == *terminal_handoff_ref
        && tombstone.terminal_state == StartTombstoneStateV1::Expired
}

fn reservation_fields_match_tombstone_v2(
    tombstone: &SessionIdTombstoneV1,
    intent: &HostSessionTransitionIntentV2,
) -> bool {
    tombstone.orchestration_session_id == intent.orchestration_session_id
        && tombstone.intent_id == intent.intent_id
        && tombstone.issuer_request_id == intent.issuer_request_id
        && tombstone.payload_commitment == intent.payload_commitment
}

fn validate_terminal_ref_unity_v2(
    intent: &HostSessionTransitionIntentV2,
    namespace: &SessionNamespaceRecordV1,
) -> Result<(), StoreSchemaError> {
    let mut canonical: Option<AuthorityObjectRefV1> = None;
    let mut require_same = |reference: &AuthorityObjectRefV1| {
        if reference.object_kind != AuthorityObjectKindV1::TerminalHandoff
            || canonical
                .as_ref()
                .is_some_and(|existing| existing != reference)
        {
            Err(StoreSchemaError("V2 Start terminal handoff refs disagree"))
        } else {
            canonical = Some(reference.clone());
            Ok(())
        }
    };
    match &intent.state {
        HostSessionTransitionIntentStateV2::Rejected {
            terminal_handoff_ref,
            ..
        }
        | HostSessionTransitionIntentStateV2::Expired {
            terminal_handoff_ref,
            ..
        } => require_same(terminal_handoff_ref)?,
        _ => {}
    }
    match &intent.transport_payload_state {
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
            terminal_handoff_ref,
        }
        | HostSessionTransitionTransportPayloadStateV1::Released {
            terminal_handoff_ref,
            ..
        } => require_same(terminal_handoff_ref)?,
        HostSessionTransitionTransportPayloadStateV1::Retained => {}
    }
    if let HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
        terminal_handoff_ref,
        ..
    } = &intent.input_handoff
    {
        require_same(terminal_handoff_ref)?;
    }
    if let SessionNamespaceRecordV1::StartTombstone(tombstone) = namespace {
        require_same(&tombstone.terminal_handoff_ref)?;
    }
    Ok(())
}

impl StateRootV1 {
    pub(crate) fn validate(&self) -> Result<(), StoreSchemaError> {
        require_version(self.schema_version)?;
        validate_store_id(&self.authority_store_id)
            .map_err(|_| StoreSchemaError("invalid authority store ID"))?;
        required(&self.bootstrap_home.physical_path)?;
        if self.root_revision == 0 {
            return Err(StoreSchemaError("root revision must be positive"));
        }
        let certificate = &self.greenfield_namespace_certificate;
        require_version(certificate.schema_version)?;
        if certificate.authority_store_id != self.authority_store_id
            || certificate.bootstrap_home != self.bootstrap_home
        {
            return Err(StoreSchemaError(
                "greenfield certificate does not match root identity",
            ));
        }

        let mut active_count = 0_usize;
        for (key, value) in &self.commitment_key_registry {
            require_version(value.schema_version)?;
            validate_key_id(&value.key_id)
                .map_err(|_| StoreSchemaError("invalid commitment key ID"))?;
            if key != &value.key_id || value.authority_store_id != self.authority_store_id {
                return Err(StoreSchemaError("commitment key registry entry mismatch"));
            }
            if value.state == AuthorityStoreCommitmentKeyStateV1::Active {
                active_count += 1;
                if value.key_id != self.active_commitment_key_id {
                    return Err(StoreSchemaError("active key ID does not match root"));
                }
            }
        }
        if active_count != 1
            || self
                .commitment_key_registry
                .get(&self.active_commitment_key_id)
                .map(|key| key.state)
                != Some(AuthorityStoreCommitmentKeyStateV1::Active)
        {
            return Err(StoreSchemaError("root must contain exactly one active key"));
        }
        if !self
            .commitment_key_registry
            .values()
            .any(|key| key.created_at == certificate.certified_at)
        {
            return Err(StoreSchemaError(
                "greenfield certificate timestamp has no initial key",
            ));
        }

        for (key, value) in &self.session_namespace_map {
            if key != value.orchestration_session_id() {
                return Err(StoreSchemaError("session namespace map key mismatch"));
            }
            match value {
                SessionNamespaceRecordV1::Authority(authority) => {
                    require_version(authority.schema_version)?;
                    required(&authority.orchestration_session_id)?;
                    required(&authority.shell_trace_session_id)?;
                    if authority.authority_revision == 0
                        || authority.workspace_binding.authority_store_id != self.authority_store_id
                        || authority.workspace_binding.authority_store_root != self.bootstrap_home
                    {
                        return Err(StoreSchemaError("durable authority binding mismatch"));
                    }
                    if let Some(active) = &authority.active_authoritative_participant_id {
                        if !authority.authoritative_participant_lineage.contains(active) {
                            return Err(StoreSchemaError(
                                "active participant is absent from authority lineage",
                            ));
                        }
                    }
                }
                SessionNamespaceRecordV1::StartReservation(reservation) => {
                    require_version(reservation.schema_version)?;
                    required(&reservation.intent_id)?;
                    required(&reservation.issuer_request_id)?;
                }
                SessionNamespaceRecordV1::StartTombstone(tombstone) => {
                    require_version(tombstone.schema_version)?;
                    required(&tombstone.intent_id)?;
                    required(&tombstone.issuer_request_id)?;
                }
            }
        }
        for (key, value) in &self.transition_intent_map {
            if key != &value.intent_id {
                return Err(StoreSchemaError("transition intent map key mismatch"));
            }
            require_version(value.schema_version)?;
            required(&value.issuer_request_id)?;
            required(&value.orchestration_session_id)?;
            required(&value.shell_trace_session_id)?;
            required(&value.target_authoritative_participant_id)?;
            required(&value.run_id)?;
            if value.intent_revision == 0
                || value.workspace_binding.authority_store_id != self.authority_store_id
                || value.workspace_binding.authority_store_root != self.bootstrap_home
                || value.issued_at.as_str() >= value.expires_at.as_str()
            {
                return Err(StoreSchemaError("transition intent binding mismatch"));
            }
            self.validate_intent_relations(value)?;
        }
        for (key, value) in &self.issuer_request_index {
            if key != &value.issuer_request_id {
                return Err(StoreSchemaError("issuer request index key mismatch"));
            }
            require_version(value.schema_version)?;
            let intent = self
                .transition_intent_map
                .get(&value.intent_id)
                .ok_or(StoreSchemaError("issuer request references no intent"))?;
            if intent.issuer_request_id != value.issuer_request_id
                || intent.orchestration_session_id != value.orchestration_session_id
                || intent.payload_commitment != value.payload_commitment
            {
                return Err(StoreSchemaError("issuer request and intent disagree"));
            }
        }
        for (key, value) in &self.application_journal {
            if key != &value.intent_id {
                return Err(StoreSchemaError("application journal key mismatch"));
            }
            require_version(value.schema_version)?;
            if !self.transition_intent_map.contains_key(&value.intent_id) {
                return Err(StoreSchemaError("application journal references no intent"));
            }
        }
        if self.issuer_request_index.len() != self.transition_intent_map.len() {
            return Err(StoreSchemaError(
                "every intent requires one issuer index entry",
            ));
        }
        for value in self.session_namespace_map.values() {
            self.validate_namespace_relations(value)?;
        }
        for (key, value) in &self.object_index {
            require_version(value.schema_version)?;
            validate_ref_id(&value.ref_id)
                .map_err(|_| StoreSchemaError("invalid object index ref ID"))?;
            if key != &value.ref_id || value.object_schema_version != SCHEMA_VERSION {
                return Err(StoreSchemaError("object index entry mismatch"));
            }
            if value.object_kind != AuthorityObjectKindV1::TransitionTransportPayload
                && value.storage_state != AuthorityObjectStorageStateV1::Present
            {
                return Err(StoreSchemaError(
                    "only transport payload objects may be released",
                ));
            }
        }
        Ok(())
    }

    fn validate_intent_relations(
        &self,
        intent: &HostSessionTransitionIntentV1,
    ) -> Result<(), StoreSchemaError> {
        let issuer = self
            .issuer_request_index
            .get(&intent.issuer_request_id)
            .ok_or(StoreSchemaError("intent has no issuer request index entry"))?;
        if issuer.intent_id != intent.intent_id
            || issuer.orchestration_session_id != intent.orchestration_session_id
            || issuer.payload_commitment != intent.payload_commitment
        {
            return Err(StoreSchemaError("intent and issuer request index disagree"));
        }

        match (intent.mode, &intent.authority_precondition) {
            (
                HostSessionTransitionModeV1::Start,
                HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            )
            | (
                HostSessionTransitionModeV1::Attach | HostSessionTransitionModeV1::ResumeOneTurn,
                HostSessionAuthorityPreconditionV1::ExpectedRevision { .. },
            ) => {}
            _ => return Err(StoreSchemaError("transition mode precondition is invalid")),
        }

        let namespace = self
            .session_namespace_map
            .get(&intent.orchestration_session_id)
            .ok_or(StoreSchemaError("intent has no namespace record"))?;
        match &intent.state {
            HostSessionTransitionIntentStateV1::Issued
            | HostSessionTransitionIntentStateV1::Claimed { .. } => {
                if self.application_journal.contains_key(&intent.intent_id) {
                    return Err(StoreSchemaError(
                        "non-applied intent has an application journal",
                    ));
                }
                match (intent.mode, namespace) {
                    (
                        HostSessionTransitionModeV1::Start,
                        SessionNamespaceRecordV1::StartReservation(value),
                    ) if reservation_matches(value, intent) => {}
                    (
                        HostSessionTransitionModeV1::Attach
                        | HostSessionTransitionModeV1::ResumeOneTurn,
                        SessionNamespaceRecordV1::Authority(_),
                    ) => {}
                    _ => return Err(StoreSchemaError("intent namespace state is inconsistent")),
                }
            }
            HostSessionTransitionIntentStateV1::Applied {
                authority_revision_before,
                authority_revision_after,
                authority_record_commitment,
                application_result_ref,
                post_turn,
                applied_at,
                ..
            } => {
                let SessionNamespaceRecordV1::Authority(authority) = namespace else {
                    return Err(StoreSchemaError("applied intent has no authority"));
                };
                if authority.authority_revision < *authority_revision_after {
                    return Err(StoreSchemaError(
                        "applied intent exceeds authority revision",
                    ));
                }
                let journal =
                    self.application_journal
                        .get(&intent.intent_id)
                        .ok_or(StoreSchemaError(
                            "applied intent has no application journal",
                        ))?;
                let initial = &journal.initial_application;
                if initial.authority_revision_before != *authority_revision_before
                    || initial.authority_revision_after != *authority_revision_after
                    || initial.authority_record_commitment != *authority_record_commitment
                    || initial.application_result_ref != *application_result_ref
                    || initial.applied_at != *applied_at
                {
                    return Err(StoreSchemaError("applied intent and journal disagree"));
                }
                validate_post_turn_journal(post_turn, journal.post_turn_application.as_ref())?;
                if intent.mode == HostSessionTransitionModeV1::Start
                    && !authority_origin_matches(authority, intent)
                {
                    return Err(StoreSchemaError(
                        "start authority origin does not match intent",
                    ));
                }
            }
            HostSessionTransitionIntentStateV1::Rejected {
                reason,
                terminal_handoff_ref,
                ..
            } => {
                if self.application_journal.contains_key(&intent.intent_id) {
                    return Err(StoreSchemaError(
                        "rejected intent has an application journal",
                    ));
                }
                match (intent.mode, namespace) {
                    (
                        HostSessionTransitionModeV1::Start,
                        SessionNamespaceRecordV1::StartTombstone(value),
                    ) if tombstone_matches_rejected(
                        value,
                        intent,
                        *reason,
                        terminal_handoff_ref,
                    ) => {}
                    (
                        HostSessionTransitionModeV1::Attach
                        | HostSessionTransitionModeV1::ResumeOneTurn,
                        SessionNamespaceRecordV1::Authority(_),
                    ) => {}
                    _ => {
                        return Err(StoreSchemaError(
                            "rejected intent namespace is inconsistent",
                        ))
                    }
                }
            }
            HostSessionTransitionIntentStateV1::Expired {
                terminal_handoff_ref,
                ..
            } => {
                if self.application_journal.contains_key(&intent.intent_id) {
                    return Err(StoreSchemaError(
                        "expired intent has an application journal",
                    ));
                }
                match (intent.mode, namespace) {
                    (
                        HostSessionTransitionModeV1::Start,
                        SessionNamespaceRecordV1::StartTombstone(value),
                    ) if tombstone_matches_expired(value, intent, terminal_handoff_ref) => {}
                    (
                        HostSessionTransitionModeV1::Attach
                        | HostSessionTransitionModeV1::ResumeOneTurn,
                        SessionNamespaceRecordV1::Authority(_),
                    ) => {}
                    _ => return Err(StoreSchemaError("expired intent namespace is inconsistent")),
                }
            }
        }

        let transport_index = self
            .object_index
            .get(&intent.transport_payload_ref.ref_id)
            .ok_or(StoreSchemaError(
                "transport payload has no object index entry",
            ))?;
        if transport_index.object_kind != AuthorityObjectKindV1::TransitionTransportPayload
            || !transport_states_match(
                &intent.transport_payload_state,
                &transport_index.storage_state,
            )
        {
            return Err(StoreSchemaError(
                "transport parent and object index disagree",
            ));
        }
        validate_terminal_ref_unity(intent, namespace)?;
        Ok(())
    }

    fn validate_namespace_relations(
        &self,
        record: &SessionNamespaceRecordV1,
    ) -> Result<(), StoreSchemaError> {
        match record {
            SessionNamespaceRecordV1::Authority(authority) => {
                let DurableSessionAuthorityOriginV1::StartIntent {
                    intent_id,
                    issuer_request_id,
                    payload_commitment,
                } = &authority.origin;
                let intent = self
                    .transition_intent_map
                    .get(intent_id)
                    .ok_or(StoreSchemaError("authority origin has no intent"))?;
                if intent.mode != HostSessionTransitionModeV1::Start
                    || intent.issuer_request_id != *issuer_request_id
                    || intent.payload_commitment != *payload_commitment
                    || intent.orchestration_session_id != authority.orchestration_session_id
                    || !matches!(
                        intent.state,
                        HostSessionTransitionIntentStateV1::Applied { .. }
                    )
                    || !self.application_journal.contains_key(intent_id)
                    || !authority_origin_matches(authority, intent)
                {
                    return Err(StoreSchemaError(
                        "authority has no matching application proof",
                    ));
                }
            }
            SessionNamespaceRecordV1::StartReservation(value) => {
                let intent = self
                    .transition_intent_map
                    .get(&value.intent_id)
                    .ok_or(StoreSchemaError("reservation has no intent"))?;
                if intent.mode != HostSessionTransitionModeV1::Start
                    || !matches!(
                        intent.state,
                        HostSessionTransitionIntentStateV1::Issued
                            | HostSessionTransitionIntentStateV1::Claimed { .. }
                    )
                    || !reservation_matches(value, intent)
                {
                    return Err(StoreSchemaError("reservation ownership is inconsistent"));
                }
            }
            SessionNamespaceRecordV1::StartTombstone(value) => {
                let intent = self
                    .transition_intent_map
                    .get(&value.intent_id)
                    .ok_or(StoreSchemaError("tombstone has no intent"))?;
                let matches = match &intent.state {
                    HostSessionTransitionIntentStateV1::Rejected {
                        reason,
                        terminal_handoff_ref,
                        ..
                    } => tombstone_matches_rejected(value, intent, *reason, terminal_handoff_ref),
                    HostSessionTransitionIntentStateV1::Expired {
                        terminal_handoff_ref,
                        ..
                    } => tombstone_matches_expired(value, intent, terminal_handoff_ref),
                    _ => false,
                };
                if intent.mode != HostSessionTransitionModeV1::Start || !matches {
                    return Err(StoreSchemaError("tombstone ownership is inconsistent"));
                }
            }
        }
        Ok(())
    }
}

fn validate_terminal_ref_unity(
    intent: &HostSessionTransitionIntentV1,
    namespace: &SessionNamespaceRecordV1,
) -> Result<(), StoreSchemaError> {
    let mut canonical: Option<AuthorityObjectRefV1> = None;
    let mut require_same = |reference: &AuthorityObjectRefV1| {
        if canonical
            .as_ref()
            .is_some_and(|existing| existing != reference)
        {
            Err(StoreSchemaError("intent terminal handoff refs disagree"))
        } else {
            canonical = Some(reference.clone());
            Ok(())
        }
    };
    match &intent.state {
        HostSessionTransitionIntentStateV1::Rejected {
            terminal_handoff_ref,
            ..
        }
        | HostSessionTransitionIntentStateV1::Expired {
            terminal_handoff_ref,
            ..
        } => require_same(terminal_handoff_ref)?,
        _ => {}
    }
    match &intent.transport_payload_state {
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
            terminal_handoff_ref,
        }
        | HostSessionTransitionTransportPayloadStateV1::Released {
            terminal_handoff_ref,
            ..
        } => require_same(terminal_handoff_ref)?,
        HostSessionTransitionTransportPayloadStateV1::Retained => {}
    }
    if let HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
        terminal_handoff_ref,
        ..
    } = &intent.input_handoff
    {
        require_same(terminal_handoff_ref)?;
    }
    if let SessionNamespaceRecordV1::StartTombstone(tombstone) = namespace {
        require_same(&tombstone.terminal_handoff_ref)?;
    }
    Ok(())
}

fn reservation_matches(
    reservation: &SessionIdReservationV1,
    intent: &HostSessionTransitionIntentV1,
) -> bool {
    reservation.orchestration_session_id == intent.orchestration_session_id
        && reservation.intent_id == intent.intent_id
        && reservation.issuer_request_id == intent.issuer_request_id
        && reservation.payload_commitment == intent.payload_commitment
}

fn authority_origin_matches(
    authority: &DurableSessionAuthorityV1,
    intent: &HostSessionTransitionIntentV1,
) -> bool {
    matches!(
        &authority.origin,
        DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            payload_commitment,
        } if intent_id == &intent.intent_id
            && issuer_request_id == &intent.issuer_request_id
            && payload_commitment == &intent.payload_commitment
    )
}

fn tombstone_matches_rejected(
    tombstone: &SessionIdTombstoneV1,
    intent: &HostSessionTransitionIntentV1,
    reason: HostSessionTransitionTerminalRejectionV1,
    terminal_handoff_ref: &AuthorityObjectRefV1,
) -> bool {
    reservation_fields_match_tombstone(tombstone, intent)
        && tombstone.terminal_handoff_ref == *terminal_handoff_ref
        && tombstone.terminal_state == StartTombstoneStateV1::Rejected { reason }
}

fn tombstone_matches_expired(
    tombstone: &SessionIdTombstoneV1,
    intent: &HostSessionTransitionIntentV1,
    terminal_handoff_ref: &AuthorityObjectRefV1,
) -> bool {
    reservation_fields_match_tombstone(tombstone, intent)
        && tombstone.terminal_handoff_ref == *terminal_handoff_ref
        && tombstone.terminal_state == StartTombstoneStateV1::Expired
}

fn reservation_fields_match_tombstone(
    tombstone: &SessionIdTombstoneV1,
    intent: &HostSessionTransitionIntentV1,
) -> bool {
    tombstone.orchestration_session_id == intent.orchestration_session_id
        && tombstone.intent_id == intent.intent_id
        && tombstone.issuer_request_id == intent.issuer_request_id
        && tombstone.payload_commitment == intent.payload_commitment
}

fn validate_post_turn_journal(
    post_turn: &HostSessionPostTurnApplicationV1,
    journal: Option<&PostTurnApplicationJournalV1>,
) -> Result<(), StoreSchemaError> {
    match (post_turn, journal) {
        (HostSessionPostTurnApplicationV1::NotApplicable, None)
        | (HostSessionPostTurnApplicationV1::Pending { .. }, None) => Ok(()),
        (
            HostSessionPostTurnApplicationV1::Applied {
                completion_ref,
                authority_revision_before,
                authority_revision_after,
                application_result_ref,
                applied_at,
                ..
            },
            Some(journal),
        ) if journal.completion_ref == **completion_ref
            && journal.authority_revision_before == *authority_revision_before
            && journal.authority_revision_after == *authority_revision_after
            && journal.application_result_ref == **application_result_ref
            && journal.applied_at == *applied_at =>
        {
            Ok(())
        }
        _ => Err(StoreSchemaError("post-turn state and journal disagree")),
    }
}

fn transport_states_match(
    parent: &HostSessionTransitionTransportPayloadStateV1,
    index: &AuthorityObjectStorageStateV1,
) -> bool {
    match (parent, index) {
        (
            HostSessionTransitionTransportPayloadStateV1::Retained,
            AuthorityObjectStorageStateV1::Present,
        ) => true,
        (
            HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref: left,
            },
            AuthorityObjectStorageStateV1::ReleaseEligible {
                terminal_handoff_ref: right,
            },
        ) => left == right,
        (
            HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref: left_ref,
                released_at: left_at,
            },
            AuthorityObjectStorageStateV1::Released {
                terminal_handoff_ref: right_ref,
                released_at: right_at,
            },
        ) => left_ref == right_ref && left_at == right_at,
        _ => false,
    }
}

fn require_version(version: u32) -> Result<(), StoreSchemaError> {
    if version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(StoreSchemaError(
            "unsupported authority store schema version",
        ))
    }
}

fn required(value: &str) -> Result<(), StoreSchemaError> {
    if value.is_empty() {
        Err(StoreSchemaError(
            "required authority store identity is empty",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_runtime::host_session_authority::canonical_json;
    use crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1;

    fn timestamp() -> TimestampV1 {
        TimestampV1::parse("2026-07-11T12:49:11.000000000Z").unwrap()
    }

    fn home() -> CanonicalDirectoryV1 {
        CanonicalDirectoryV1 {
            physical_path: "/tmp/substrate-a1-home".into(),
            physical_identity: DirectoryPhysicalIdentityV1::Linux {
                device_id: 7,
                inode: 11,
            },
        }
    }

    fn root() -> StateRootV1 {
        let key = AuthorityStoreCommitmentKeyV1 {
            schema_version: 1,
            authority_store_id: "as_11111111111111111111111111111111".into(),
            key_id: "ak_22222222222222222222222222222222".into(),
            algorithm: AuthorityStoreCommitmentAlgorithmV1::HmacSha256,
            created_at: timestamp(),
            state: AuthorityStoreCommitmentKeyStateV1::Active,
        };
        StateRootV1 {
            schema_version: 1,
            authority_store_id: key.authority_store_id.clone(),
            bootstrap_home: home(),
            root_revision: 1,
            active_commitment_key_id: key.key_id.clone(),
            commitment_key_registry: BTreeMap::from([(key.key_id.clone(), key)]),
            greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1 {
                schema_version: 1,
                authority_store_id: "as_11111111111111111111111111111111".into(),
                bootstrap_home: home(),
                certified_at: timestamp(),
            },
            session_namespace_map: BTreeMap::new(),
            transition_intent_map: BTreeMap::new(),
            issuer_request_index: BTreeMap::new(),
            application_journal: BTreeMap::new(),
            object_index: BTreeMap::new(),
        }
    }

    #[test]
    fn revision_one_root_round_trips_and_rejects_inconsistent_metadata() {
        let root = root();
        root.validate().unwrap();
        let bytes = canonical_json::to_vec(&root).unwrap();
        let decoded: StateRootV1 = canonical_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, root);

        let mut wrong_certificate = root.clone();
        wrong_certificate
            .greenfield_namespace_certificate
            .authority_store_id = "as_33333333333333333333333333333333".into();
        assert!(wrong_certificate.validate().is_err());

        let mut wrong_registry_key = root.clone();
        let key = wrong_registry_key
            .commitment_key_registry
            .remove("ak_22222222222222222222222222222222")
            .unwrap();
        wrong_registry_key
            .commitment_key_registry
            .insert("ak_44444444444444444444444444444444".into(), key);
        assert!(wrong_registry_key.validate().is_err());

        let mut two_active = root;
        let mut second = two_active
            .commitment_key_registry
            .values()
            .next()
            .unwrap()
            .clone();
        second.key_id = "ak_55555555555555555555555555555555".into();
        two_active
            .commitment_key_registry
            .insert(second.key_id.clone(), second);
        assert!(two_active.validate().is_err());
    }

    #[test]
    fn transport_parent_and_index_lifecycle_must_match_exactly() {
        let terminal = AuthorityObjectRefV1 {
            ref_id: "ao_99999999999999999999999999999999".into(),
            object_kind: AuthorityObjectKindV1::TerminalHandoff,
            schema_version: 1,
            commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
            },
        };
        assert!(transport_states_match(
            &HostSessionTransitionTransportPayloadStateV1::Retained,
            &AuthorityObjectStorageStateV1::Present,
        ));
        assert!(!transport_states_match(
            &HostSessionTransitionTransportPayloadStateV1::Retained,
            &AuthorityObjectStorageStateV1::Released {
                terminal_handoff_ref: terminal.clone(),
                released_at: timestamp(),
            },
        ));
        assert!(!transport_states_match(
            &HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                terminal_handoff_ref: terminal.clone(),
            },
            &AuthorityObjectStorageStateV1::Present,
        ));
        assert!(!transport_states_match(
            &HostSessionTransitionTransportPayloadStateV1::Released {
                terminal_handoff_ref: terminal.clone(),
                released_at: timestamp(),
            },
            &AuthorityObjectStorageStateV1::ReleaseEligible {
                terminal_handoff_ref: terminal,
            },
        ));
    }
}
