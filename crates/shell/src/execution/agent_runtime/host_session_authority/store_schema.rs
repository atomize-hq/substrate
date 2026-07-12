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
