use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::schema::{
    AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1, CanonicalDirectoryV1,
    DurableSessionAuthorityOriginV1, HostPostTurnDispositionV1, HostSessionAuthorityPreconditionV1,
    HostSessionPostureV1, HostSessionTransitionCallerV1, HostSessionTransitionModeV1,
    HostSessionTransitionTerminalRejectionV1, TimestampV1, WorkspaceBindingV1, WorldBindingV1,
};
use super::store_format::{validate_key_id, validate_ref_id, validate_store_id};
use super::validation::validate_object_commitment_rule;

const SCHEMA_VERSION: u32 = 1;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VersionedStateRoot {
    V1(StateRootV1),
    V2(StateRootV2),
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
        }
    }

    pub(crate) fn validate(&self) -> Result<(), StoreSchemaError> {
        match self {
            Self::V1(root) => root.validate(),
            Self::V2(root) => root.validate(),
        }
    }

    pub(crate) fn root_revision(&self) -> u64 {
        match self {
            Self::V1(root) => root.root_revision,
            Self::V2(root) => root.root_revision,
        }
    }

    pub(crate) fn authority_store_id(&self) -> &str {
        match self {
            Self::V1(root) => &root.authority_store_id,
            Self::V2(root) => &root.authority_store_id,
        }
    }

    pub(crate) fn bootstrap_home(&self) -> &CanonicalDirectoryV1 {
        match self {
            Self::V1(root) => &root.bootstrap_home,
            Self::V2(root) => &root.bootstrap_home,
        }
    }

    pub(crate) fn greenfield_namespace_certificate(&self) -> &GreenfieldNamespaceCertificateV1 {
        match self {
            Self::V1(root) => &root.greenfield_namespace_certificate,
            Self::V2(root) => &root.greenfield_namespace_certificate,
        }
    }

    pub(crate) fn active_commitment_key_id(&self) -> &str {
        match self {
            Self::V1(root) => &root.active_commitment_key_id,
            Self::V2(root) => &root.active_commitment_key_id,
        }
    }

    pub(crate) fn active_commitment_key_id_mut(&mut self) -> &mut String {
        match self {
            Self::V1(root) => &mut root.active_commitment_key_id,
            Self::V2(root) => &mut root.active_commitment_key_id,
        }
    }

    pub(crate) fn commitment_key_registry(
        &self,
    ) -> &BTreeMap<String, AuthorityStoreCommitmentKeyV1> {
        match self {
            Self::V1(root) => &root.commitment_key_registry,
            Self::V2(root) => &root.commitment_key_registry,
        }
    }

    pub(crate) fn commitment_key_registry_mut(
        &mut self,
    ) -> &mut BTreeMap<String, AuthorityStoreCommitmentKeyV1> {
        match self {
            Self::V1(root) => &mut root.commitment_key_registry,
            Self::V2(root) => &mut root.commitment_key_registry,
        }
    }

    pub(crate) fn set_root_revision(&mut self, revision: u64) {
        match self {
            Self::V1(root) => root.root_revision = revision,
            Self::V2(root) => root.root_revision = revision,
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
                if authority.authority_revision != *authority_revision_after
                    || authority.shell_trace_session_id != intent.shell_trace_session_id
                    || authority.authoritative_participant_lineage
                        != intent.resulting_authoritative_lineage
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

fn validate_namespace_record_identity(
    record: &SessionNamespaceRecordV1,
    authority_store_id: &str,
    bootstrap_home: &CanonicalDirectoryV1,
) -> Result<(), StoreSchemaError> {
    match record {
        SessionNamespaceRecordV1::Authority(authority) => {
            require_version(authority.schema_version)?;
            required(&authority.orchestration_session_id)?;
            required(&authority.shell_trace_session_id)?;
            if authority.authority_revision == 0
                || authority.workspace_binding.authority_store_id != authority_store_id
                || &authority.workspace_binding.authority_store_root != bootstrap_home
                || !authority.retained_worker_refs.is_empty()
                || !authority.internal_resume_handle_refs.is_empty()
                || authority
                    .active_authoritative_participant_id
                    .as_ref()
                    .is_some_and(|active| {
                        !authority.authoritative_participant_lineage.contains(active)
                    })
            {
                return Err(StoreSchemaError("V2 durable authority binding mismatch"));
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
