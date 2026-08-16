use std::path::{Path, PathBuf};

use super::controls::SecurityAgentArmEvidenceV2;
#[cfg(test)]
use super::controls::SecurityAgentRawReportEvidenceV2;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    document_sha256_v2, ExecutableIdentityV2, FinalizationRequestV2, LaunchIdentityV2,
    ProcessIdentityV2, ProtectedCasBindingV2, PublisherPreRemovalReceiptV2, TargetSetKindV2,
    TerminalAcknowledgementV2, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
    MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2, MAC_R3_FINALIZER_REQUEST_PATH_V2,
};

use super::process::SupplementaryGroupAttestationV2;
use super::{
    RepetitionV2, DISPOSABLE_PUBLISHER_ROOT_V2, EXPERIMENT_ROOT_V2, EXPERIMENT_VERSION_V2,
};
use crate::contract::TERMINAL_BINDING_PATH;

pub const PUBLISHER_PROTOCOL_OWNER_V2: &str =
    "substrate.r3-macos-disposable-root-publisher-handshake";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherPreparedInputV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub publisher_identity: ExecutableIdentityV2,
    pub publisher_identity_packet_sha256: String,
    pub securityagent_baseline_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherProcessAttestationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub effective_uid: u32,
    pub supplementary_groups: SupplementaryGroupAttestationV2,
    pub canonical_account: String,
    pub pid: i32,
    pub process_start_identity_sha256: String,
    pub executable_path: String,
    pub executable_physical_identity_sha256: String,
    pub executable_sha256: String,
    pub designated_requirement: String,
    pub cdhash: String,
    pub interaction_denial_established_first: bool,
}

impl PublisherProcessAttestationV2 {
    pub fn validate(&self, prepared: &PublisherPreparedInputV2) -> Result<()> {
        prepared.validate()?;
        self.supplementary_groups.validate_empty()?;
        if self.schema_owner != PUBLISHER_PROTOCOL_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != super::EXPERIMENT_ID_V2
            || self.effective_uid != 0
            || self.canonical_account != "root"
            || self.pid <= 1
            || self.executable_path != MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2
            || self.executable_physical_identity_sha256
                != prepared.publisher_identity.physical_identity_sha256
            || self.executable_sha256 != prepared.publisher_identity.executable_sha256
            || self.designated_requirement != prepared.publisher_identity.designated_requirement
            || self.cdhash != prepared.publisher_identity.cdhash
            || !self.interaction_denial_established_first
        {
            bail!("running publisher process does not exact-match its frozen no-UI identity")
        }
        require_digest(
            &self.process_start_identity_sha256,
            "publisher process start identity",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateIdentityPacketV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub finalizer_identity: ExecutableIdentityV2,
    pub finalizer_signing_posture: super::CodeSigningPostureV2,
    pub coordinator_identity: ExecutableIdentityV2,
    pub coordinator_signing_posture: super::CodeSigningPostureV2,
    pub launch_identity: LaunchIdentityV2,
    pub capability_digest: String,
}

impl CandidateIdentityPacketV2 {
    pub fn validate(&self) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        if self.experiment_id != super::EXPERIMENT_ID_V2 {
            bail!("candidate identity packet experiment identity changed")
        }
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            &self.finalizer_identity,
            substrate_common::macos_retirement_v2::MAC_R3_FINALIZER_PATH_V2,
            substrate_common::macos_retirement_v2::MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
        )?;
        self.finalizer_signing_posture
            .validate_for(&self.finalizer_identity)?;
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            &self.coordinator_identity,
            substrate_common::macos_retirement_v2::MAC_R3_COORDINATOR_PATH_V2,
            substrate_common::macos_retirement_v2::MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
        )?;
        self.coordinator_signing_posture
            .validate_for(&self.coordinator_identity)?;
        substrate_common::macos_retirement_v2::validate_launch_identity_v2(&self.launch_identity)?;
        require_digest(&self.capability_digest, "candidate capability")
    }
}

impl PublisherPreparedInputV2 {
    pub fn validate(&self) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        if self.experiment_id != super::EXPERIMENT_ID_V2 {
            bail!("publisher prepared input experiment identity changed")
        }
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            &self.publisher_identity,
            MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
            MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2,
        )?;
        require_digest(
            &self.publisher_identity_packet_sha256,
            "publisher identity packet",
        )?;
        if self.publisher_identity_packet_sha256 != document_sha256_v2(&self.publisher_identity)? {
            bail!("publisher identity packet digest does not bind the exact executable identity")
        }
        require_digest(
            &self.securityagent_baseline_sha256,
            "prepared SecurityAgent baseline",
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PublisherStageV2 {
    Prepared,
    SurrogatesCreated,
    ReceiptSigned,
    ProtectedCasSigned,
    FinalizationRequestPublished,
    ResidualRemoved,
    TerminalBindingPublished,
    RestorationComplete,
}

pub const PUBLISHER_STAGE_SEQUENCE_V2: [PublisherStageV2; 8] = [
    PublisherStageV2::Prepared,
    PublisherStageV2::SurrogatesCreated,
    PublisherStageV2::ReceiptSigned,
    PublisherStageV2::ProtectedCasSigned,
    PublisherStageV2::FinalizationRequestPublished,
    PublisherStageV2::ResidualRemoved,
    PublisherStageV2::TerminalBindingPublished,
    PublisherStageV2::RestorationComplete,
];

impl PublisherStageV2 {
    pub fn require_successor(self, next: Self) -> Result<()> {
        let current = PUBLISHER_STAGE_SEQUENCE_V2
            .iter()
            .position(|candidate| *candidate == self)
            .expect("closed publisher stage is in its sequence");
        let next = PUBLISHER_STAGE_SEQUENCE_V2
            .iter()
            .position(|candidate| *candidate == next)
            .expect("closed publisher stage is in its sequence");
        if next != current + 1 {
            bail!("root publisher invocation is not the exact next closed state")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisposableSignerIdentityV2 {
    pub application_tag_sha256: String,
    pub label: String,
    pub application_label: String,
    pub application_label_sha256: String,
    pub persistent_reference_sha256: String,
    pub spki_der: String,
    pub spki_der_sha256: String,
    pub access_control_sha256: String,
    pub identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SurrogateCreationReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub target_set_kind: TargetSetKindV2,
    pub publisher_identity: ExecutableIdentityV2,
    pub publisher_process_attestation_sha256: String,
    pub target: DisposableSignerIdentityV2,
    pub wrong: DisposableSignerIdentityV2,
    pub capability_pre_observation_sha256: String,
    pub protected_wrapper_identity_sha256: String,
    pub current_lock_identity_sha256: String,
    pub terminal_latch_identity_sha256: String,
    pub before_observation_sha256: String,
    pub quiesced_observation_sha256: String,
    pub protected_cas_generation: u64,
    pub protected_cas_head_sha256: String,
    pub securityagent_baseline_sha256: String,
}

impl SurrogateCreationReceiptV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.target_set_kind != TargetSetKindV2::DisposableCapability
            || self.protected_cas_generation == 0
        {
            bail!("surrogate creation receipt is not the fixed disposable experiment")
        }
        for (digest, label) in [
            (
                &self.target.application_tag_sha256,
                "target application tag",
            ),
            (
                &self.target.application_label_sha256,
                "target application label",
            ),
            (
                &self.target.persistent_reference_sha256,
                "target persistent reference",
            ),
            (&self.target.spki_der_sha256, "target SPKI"),
            (&self.target.access_control_sha256, "target access"),
            (&self.target.identity_sha256, "target identity"),
            (&self.wrong.application_tag_sha256, "wrong application tag"),
            (
                &self.wrong.application_label_sha256,
                "wrong application label",
            ),
            (
                &self.wrong.persistent_reference_sha256,
                "wrong persistent reference",
            ),
            (&self.wrong.spki_der_sha256, "wrong SPKI"),
            (&self.wrong.access_control_sha256, "wrong access"),
            (&self.wrong.identity_sha256, "wrong identity"),
            (
                &self.capability_pre_observation_sha256,
                "capability pre-observation",
            ),
            (
                &self.publisher_process_attestation_sha256,
                "publisher process attestation",
            ),
            (
                &self.protected_wrapper_identity_sha256,
                "protected wrapper identity",
            ),
            (&self.current_lock_identity_sha256, "current lock identity"),
            (
                &self.terminal_latch_identity_sha256,
                "terminal latch identity",
            ),
            (&self.before_observation_sha256, "before observation"),
            (&self.quiesced_observation_sha256, "quiesced observation"),
            (&self.protected_cas_head_sha256, "protected CAS head"),
            (
                &self.securityagent_baseline_sha256,
                "SecurityAgent baseline",
            ),
        ] {
            require_digest(digest, label)?;
        }
        if self.target.label != format!("{}:signing-key", self.scope_id)
            || self.wrong.label != format!("{}:wrong-surrogate-signing-key", self.scope_id)
            || self.target.application_tag_sha256 == self.wrong.application_tag_sha256
            || self.target.identity_sha256 == self.wrong.identity_sha256
        {
            bail!("surrogate target and wrong-key identities are not exactly separated")
        }
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            &self.publisher_identity,
            MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
            MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2,
        )?;
        require_base64url(&self.target.application_label, "target application label")?;
        require_base64url(&self.target.spki_der, "target SPKI")?;
        require_base64url(&self.wrong.application_label, "wrong application label")?;
        require_base64url(&self.wrong.spki_der, "wrong SPKI")?;
        Ok(())
    }

    pub fn validate_prepared_input(&self, prepared: &PublisherPreparedInputV2) -> Result<()> {
        prepared.validate()?;
        if self.publisher_identity != prepared.publisher_identity
            || self.securityagent_baseline_sha256 != prepared.securityagent_baseline_sha256
        {
            bail!("creation receipt did not echo the frozen publisher identity and UI baseline")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IdentityBindingPacketV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub creation_receipt_sha256: String,
    pub candidate_identity_packet_sha256: String,
    pub finalizer_identity: ExecutableIdentityV2,
    pub coordinator_identity: ExecutableIdentityV2,
    pub coordinator_process: ProcessIdentityV2,
    pub launch_identity: LaunchIdentityV2,
    pub capability_digest: String,
    pub current_lock_identity_sha256: String,
    pub signer_access_control_sha256: String,
}

impl IdentityBindingPacketV2 {
    pub fn validate_against_creation(
        &self,
        repetition: RepetitionV2,
        creation: &SurrogateCreationReceiptV2,
    ) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        creation.validate(repetition)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.creation_receipt_sha256 != document_sha256_v2(creation)?
            || self.current_lock_identity_sha256 != creation.current_lock_identity_sha256
            || self.signer_access_control_sha256 != creation.target.access_control_sha256
        {
            bail!("identity-binding packet does not exact-bind created surrogate state")
        }
        for (digest, label) in [
            (&self.creation_receipt_sha256, "creation receipt"),
            (
                &self.candidate_identity_packet_sha256,
                "candidate identity packet",
            ),
            (&self.capability_digest, "capability"),
            (&self.current_lock_identity_sha256, "current lock"),
            (&self.signer_access_control_sha256, "signer access"),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }

    pub fn validate_candidate(&self, candidate: &CandidateIdentityPacketV2) -> Result<()> {
        candidate.validate()?;
        if self.candidate_identity_packet_sha256 != document_sha256_v2(candidate)?
            || self.finalizer_identity != candidate.finalizer_identity
            || self.coordinator_identity != candidate.coordinator_identity
            || self.launch_identity != candidate.launch_identity
            || self.capability_digest != candidate.capability_digest
        {
            bail!("identity binding differs from the frozen candidate identity packet")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResidualCleanupReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub effects_response_sha256: String,
    pub target_absent: bool,
    pub wrong_absent: bool,
    pub exact_after_observation_sha256: String,
    pub securityagent_observation_sha256: String,
}

impl ResidualCleanupReceiptV2 {
    pub fn validate(&self, repetition: RepetitionV2, effects_sha256: &str) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.effects_response_sha256 != effects_sha256
            || !self.target_absent
            || !self.wrong_absent
        {
            bail!("residual cleanup did not preserve target absence and remove only the wrong key")
        }
        require_digest(&self.effects_response_sha256, "effects response")?;
        require_digest(
            &self.exact_after_observation_sha256,
            "exact residual after observation",
        )?;
        require_digest(
            &self.securityagent_observation_sha256,
            "residual SecurityAgent observation",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RestorationReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub complete_response_sha256: String,
    pub target_absent: bool,
    pub wrong_absent: bool,
    pub scope_resources_absent: bool,
    pub securityagent_observation_sha256: String,
    pub restoration_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmergencyRollbackMarkerV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub last_durable_stage: PublisherStageV2,
    pub failure_classification: EmergencyFailureClassificationV2,
    pub failure_observation_sha256: String,
    pub target_identity_sha256: String,
    pub wrong_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreCreationEmergencyRollbackMarkerV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub target_label: String,
    pub wrong_label: String,
    pub failure_classification: EmergencyFailureClassificationV2,
    pub failure_observation_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmergencyFailureClassificationV2 {
    ClosedFailure,
    SecurityAgentAlertExit86,
}

impl PreCreationEmergencyRollbackMarkerV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.target_label != format!("{}:signing-key", repetition.scope_id())
            || self.wrong_label != format!("{}:wrong-surrogate-signing-key", repetition.scope_id())
        {
            bail!("pre-creation rollback is not the exact compiled surrogate scope")
        }
        require_digest(
            &self.failure_observation_sha256,
            "pre-creation rollback failure",
        )
    }
}

impl EmergencyRollbackMarkerV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        creation: &SurrogateCreationReceiptV2,
    ) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        creation.validate(repetition)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.last_durable_stage == PublisherStageV2::RestorationComplete
            || self.target_identity_sha256 != creation.target.identity_sha256
            || self.wrong_identity_sha256 != creation.wrong.identity_sha256
        {
            bail!("emergency rollback marker is not exact scope-bound surrogate rollback")
        }
        require_digest(
            &self.failure_observation_sha256,
            "rollback failure observation",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmergencyRollbackReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub rollback_marker_sha256: String,
    pub target_absent: bool,
    pub wrong_absent: bool,
    pub scope_resources_absent: bool,
    pub securityagent_observation_sha256: String,
    pub restoration_sha256: String,
}

impl EmergencyRollbackReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        marker: &EmergencyRollbackMarkerV2,
    ) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.rollback_marker_sha256 != document_sha256_v2(marker)?
            || !self.target_absent
            || !self.wrong_absent
            || !self.scope_resources_absent
        {
            bail!("emergency rollback receipt does not prove exact surrogate restoration")
        }
        for (digest, label) in [
            (
                &self.securityagent_observation_sha256,
                "rollback SecurityAgent observation",
            ),
            (&self.restoration_sha256, "rollback restoration"),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }

    pub fn validate_precreation(
        &self,
        repetition: RepetitionV2,
        marker: &PreCreationEmergencyRollbackMarkerV2,
    ) -> Result<()> {
        marker.validate(repetition)?;
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.rollback_marker_sha256 != document_sha256_v2(marker)?
            || !self.target_absent
            || !self.wrong_absent
            || !self.scope_resources_absent
        {
            bail!("pre-creation rollback receipt does not prove exact surrogate restoration")
        }
        for (digest, label) in [
            (
                &self.securityagent_observation_sha256,
                "rollback SecurityAgent observation",
            ),
            (&self.restoration_sha256, "rollback restoration"),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }
}

impl RestorationReceiptV2 {
    pub fn validate(&self, repetition: RepetitionV2, complete_sha256: &str) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.complete_response_sha256 != complete_sha256
            || !self.target_absent
            || !self.wrong_absent
            || !self.scope_resources_absent
        {
            bail!("publisher restoration receipt does not prove exact disposable absence")
        }
        for (digest, label) in [
            (&self.complete_response_sha256, "complete response"),
            (
                &self.securityagent_observation_sha256,
                "restoration SecurityAgent observation",
            ),
            (&self.restoration_sha256, "publisher restoration"),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherAdvanceMarkerV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub prior_stage: PublisherStageV2,
    pub next_stage: PublisherStageV2,
    pub fixed_input_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherProgressV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub generation: u64,
    pub stage: PublisherStageV2,
    pub predecessor_progress_sha256: String,
    pub accepted_input_bundle_sha256: String,
    pub step_receipt_sha256: String,
}

impl PublisherProgressV2 {
    pub fn genesis(repetition: RepetitionV2) -> Self {
        Self {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: super::EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            generation: 1,
            stage: PublisherStageV2::Prepared,
            predecessor_progress_sha256: publisher_progress_genesis_sha256_v2(
                repetition,
                "progress-genesis",
            ),
            accepted_input_bundle_sha256: publisher_progress_genesis_sha256_v2(
                repetition,
                "no-input-yet",
            ),
            step_receipt_sha256: publisher_progress_genesis_sha256_v2(repetition, "no-step-yet"),
        }
    }

    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        let expected_generation = publisher_stage_generation_v2(self.stage);
        if self.experiment_id != super::EXPERIMENT_ID_V2 || self.generation != expected_generation {
            bail!("publisher progress generation does not exact-match its closed stage")
        }
        for (digest, label) in [
            (
                &self.predecessor_progress_sha256,
                "publisher predecessor progress",
            ),
            (
                &self.accepted_input_bundle_sha256,
                "publisher accepted input bundle",
            ),
            (&self.step_receipt_sha256, "publisher step receipt"),
        ] {
            require_digest(digest, label)?;
        }
        if self.stage == PublisherStageV2::Prepared && self != &Self::genesis(repetition) {
            bail!("publisher Prepared progress is not the exact scope-bound genesis")
        }
        Ok(())
    }

    pub fn validate_successor(
        &self,
        repetition: RepetitionV2,
        predecessor: &Self,
        marker: &PublisherAdvanceMarkerV2,
        bundle: &PublisherInputBundleV2,
        step: &PublisherStepReceiptV2,
    ) -> Result<()> {
        self.validate(repetition)?;
        predecessor.validate(repetition)?;
        predecessor.stage.require_successor(self.stage)?;
        marker.validate_input_bundle(repetition, bundle)?;
        step.validate(repetition, marker, &step.output_sha256)?;
        if self.generation != predecessor.generation + 1
            || marker.prior_stage != predecessor.stage
            || marker.next_stage != self.stage
            || self.predecessor_progress_sha256 != document_sha256_v2(predecessor)?
            || self.accepted_input_bundle_sha256 != document_sha256_v2(bundle)?
            || self.step_receipt_sha256 != document_sha256_v2(step)?
        {
            bail!("publisher progress replacement does not exact-bind predecessor, input, and step")
        }
        Ok(())
    }
}

pub fn publisher_stage_generation_v2(stage: PublisherStageV2) -> u64 {
    u64::try_from(
        PUBLISHER_STAGE_SEQUENCE_V2
            .iter()
            .position(|candidate| *candidate == stage)
            .expect("closed publisher stage is in its sequence"),
    )
    .expect("closed publisher stage count fits u64")
        + 1
}

pub fn publisher_progress_genesis_sha256_v2(repetition: RepetitionV2, label: &str) -> String {
    substrate_common::macos_retirement_v2::sha256_hex_v2(
        format!(
            "substrate.r3-macos-disposable-publisher.v2\0{}\0{label}",
            repetition.scope_id()
        )
        .as_bytes(),
    )
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PublisherInputArtifactV2 {
    PreparedInput,
    PublisherProcessAttestation,
    CreationReceipt,
    IdentityBinding,
    UnsignedReceipt,
    SignedReceipt,
    HarnessAcknowledgement,
    UnsignedProtectedCas,
    SignedProtectedCas,
    FinalizationRequest,
    EffectsCompleteResponse,
    TerminalAcknowledgement,
    TerminalBinding,
    CompleteResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherInputArtifactDigestV2 {
    pub artifact: PublisherInputArtifactV2,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherInputBundleV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub prior_stage: PublisherStageV2,
    pub next_stage: PublisherStageV2,
    pub artifacts: Vec<PublisherInputArtifactDigestV2>,
}

impl PublisherInputBundleV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2 {
            bail!("publisher input bundle experiment identity changed")
        }
        self.prior_stage.require_successor(self.next_stage)?;
        let expected = expected_input_artifacts_v2(self.next_stage);
        if self.artifacts.len() != expected.len()
            || !self
                .artifacts
                .iter()
                .zip(expected)
                .all(|(actual, expected)| actual.artifact == *expected)
        {
            bail!("publisher input bundle is not the exact ordered transition input")
        }
        for artifact in &self.artifacts {
            require_digest(&artifact.sha256, "publisher input artifact")?;
        }
        Ok(())
    }
}

pub const fn expected_input_artifacts_v2(
    next: PublisherStageV2,
) -> &'static [PublisherInputArtifactV2] {
    use PublisherInputArtifactV2 as A;
    match next {
        PublisherStageV2::Prepared => &[],
        PublisherStageV2::SurrogatesCreated => &[A::PreparedInput, A::PublisherProcessAttestation],
        PublisherStageV2::ReceiptSigned => {
            &[A::CreationReceipt, A::IdentityBinding, A::UnsignedReceipt]
        }
        PublisherStageV2::ProtectedCasSigned => &[
            A::SignedReceipt,
            A::HarnessAcknowledgement,
            A::UnsignedProtectedCas,
        ],
        PublisherStageV2::FinalizationRequestPublished => &[
            A::SignedReceipt,
            A::HarnessAcknowledgement,
            A::SignedProtectedCas,
            A::FinalizationRequest,
        ],
        PublisherStageV2::ResidualRemoved => &[A::EffectsCompleteResponse],
        PublisherStageV2::TerminalBindingPublished => {
            &[A::TerminalAcknowledgement, A::TerminalBinding]
        }
        PublisherStageV2::RestorationComplete => &[A::CompleteResponse],
    }
}

impl PublisherAdvanceMarkerV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2 {
            bail!("publisher marker experiment identity changed")
        }
        self.prior_stage.require_successor(self.next_stage)?;
        require_digest(&self.fixed_input_sha256, "publisher fixed input")
    }

    pub fn validate_input_bundle(
        &self,
        repetition: RepetitionV2,
        bundle: &PublisherInputBundleV2,
    ) -> Result<()> {
        self.validate(repetition)?;
        bundle.validate(repetition)?;
        if self.prior_stage != bundle.prior_stage
            || self.next_stage != bundle.next_stage
            || self.fixed_input_sha256 != document_sha256_v2(bundle)?
        {
            bail!("publisher marker does not bind its exact canonical input bundle")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherStepReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub prior_stage: PublisherStageV2,
    pub new_stage: PublisherStageV2,
    pub marker_sha256: String,
    pub output_sha256: String,
    pub securityagent_observation_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherSecurityAgentObservationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub stage: PublisherStageV2,
    pub prearm_baseline_sha256: String,
    pub arm_evidence: SecurityAgentArmEvidenceV2,
    pub arm_evidence_sha256: String,
    pub observation_started_monotonic_ns: u64,
    pub observation_finished_monotonic_ns: u64,
    pub unexpected_ui_observed: bool,
}

impl PublisherSecurityAgentObservationV2 {
    pub fn validate(&self, repetition: RepetitionV2, stage: PublisherStageV2) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        self.arm_evidence.validate()?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.stage != stage
            || self.observation_started_monotonic_ns >= self.observation_finished_monotonic_ns
            || self.unexpected_ui_observed
            || self.arm_evidence.unexpected_ui_observed
            || self.arm_evidence_sha256 != document_sha256_v2(&self.arm_evidence)?
        {
            bail!("publisher SecurityAgent observation is not one successful bounded arm span")
        }
        require_digest(&self.prearm_baseline_sha256, "publisher UI prearm baseline")?;
        require_digest(&self.arm_evidence_sha256, "publisher UI raw arm evidence")
    }
}

impl PublisherStepReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        marker: &PublisherAdvanceMarkerV2,
        expected_output_sha256: &str,
    ) -> Result<()> {
        require_header(
            &self.schema_owner,
            self.schema_version,
            PUBLISHER_PROTOCOL_OWNER_V2,
        )?;
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        marker.validate(repetition)?;
        if self.experiment_id != super::EXPERIMENT_ID_V2
            || self.prior_stage != marker.prior_stage
            || self.new_stage != marker.next_stage
            || self.marker_sha256 != document_sha256_v2(marker)?
            || self.output_sha256 != expected_output_sha256
        {
            bail!("publisher step receipt does not bind its one closed transition")
        }
        require_digest(
            &self.securityagent_observation_sha256,
            "publisher SecurityAgent observation",
        )
    }

    pub fn validate_securityagent_observation(
        &self,
        repetition: RepetitionV2,
        observation: &PublisherSecurityAgentObservationV2,
    ) -> Result<()> {
        observation.validate(repetition, self.new_stage)?;
        if self.securityagent_observation_sha256 != document_sha256_v2(observation)? {
            bail!("publisher step receipt does not bind its actual SecurityAgent arm span")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublisherArtifactV2 {
    GlobalPreEffectPacket,
    CreatorRouteReceiptSet,
    NativeEvidenceExport,
    NativeEvidenceExportAcknowledgement,
    NativeEvidenceCleanupReceipt,
    PreparedInput,
    PublisherProcessAttestation,
    CandidateIdentityPacket,
    CreationReceipt,
    IdentityBinding,
    UnsignedReceipt,
    SignedReceipt,
    UnsignedProtectedCas,
    SignedProtectedCas,
    FinalizationRequest,
    HarnessAcknowledgement,
    EffectsCompleteResponse,
    TerminalAcknowledgement,
    TerminalBinding,
    CompleteResponse,
    ResidualCleanupReceipt,
    RestorationReceipt,
    EmergencyRollbackMarker,
    PreCreationEmergencyRollbackMarker,
    EmergencyRollbackReceipt,
    TransportControlsCompleteMarker,
    TransportControlsObservation,
    PeerControlSetReceipt,
    PeerControlsReadyMarker,
    PeerControlRunnerSetReceipt,
    PeerControlRestorationReceipt,
    SecurityAgentObservation(PublisherStageV2),
    Progress,
    InputBundle(PublisherStageV2),
    StepReceipt(PublisherStageV2),
    AdvanceMarker(PublisherStageV2),
}

impl PublisherArtifactV2 {
    pub const fn filename(self) -> &'static str {
        match self {
            Self::GlobalPreEffectPacket => "global-pre-effect-packet.v2.json",
            Self::CreatorRouteReceiptSet => "creator-route-receipts.v2.json",
            Self::NativeEvidenceExport => "native-evidence-export.v2.json",
            Self::NativeEvidenceExportAcknowledgement => {
                "native-evidence-export-acknowledgement.v2.json"
            }
            Self::NativeEvidenceCleanupReceipt => "native-evidence-cleanup-receipt.v2.json",
            Self::PreparedInput => "publisher-prepared-input.v2.json",
            Self::PublisherProcessAttestation => "publisher-process-attestation.v2.json",
            Self::CandidateIdentityPacket => "candidate-identity-packet.v2.json",
            Self::CreationReceipt => "surrogate-creation-receipt.v2.json",
            Self::IdentityBinding => "identity-binding.v2.json",
            Self::UnsignedReceipt => "unsigned-receipt.v2.json",
            Self::SignedReceipt => "signed-receipt.v2.json",
            Self::UnsignedProtectedCas => "unsigned-protected-cas.v2.json",
            Self::SignedProtectedCas => "signed-protected-cas.v2.json",
            Self::FinalizationRequest => "finalization-request.v2.json",
            Self::HarnessAcknowledgement => "harness-acknowledgement.v2.json",
            Self::EffectsCompleteResponse => "effects-complete-response.v2.json",
            Self::TerminalAcknowledgement => "terminal-acknowledgement.v2.json",
            Self::TerminalBinding => "terminal-binding-request.v2.json",
            Self::CompleteResponse => "complete-response.v2.json",
            Self::ResidualCleanupReceipt => "residual-cleanup-receipt.v2.json",
            Self::RestorationReceipt => "restoration-receipt.v2.json",
            Self::EmergencyRollbackMarker => "emergency-rollback.marker.v2.json",
            Self::PreCreationEmergencyRollbackMarker => {
                "precreation-emergency-rollback.marker.v2.json"
            }
            Self::EmergencyRollbackReceipt => "emergency-rollback.receipt.v2.json",
            Self::TransportControlsCompleteMarker => "transport-controls-complete.marker.v2.json",
            Self::TransportControlsObservation => "transport-controls-observation.receipt.v2.json",
            Self::PeerControlSetReceipt => "peer-substitution-control-set.v2.json",
            Self::PeerControlsReadyMarker => "peer-controls-ready.marker.v2.json",
            Self::PeerControlRunnerSetReceipt => "peer-controls-runner-set.receipt.v2.json",
            Self::PeerControlRestorationReceipt => "peer-controls-restoration.receipt.v2.json",
            Self::SecurityAgentObservation(PublisherStageV2::Prepared) => {
                "ui-00-prepared.observation.v1.json"
            }
            Self::SecurityAgentObservation(PublisherStageV2::SurrogatesCreated) => {
                "ui-01-surrogates-created.observation.v1.json"
            }
            Self::SecurityAgentObservation(PublisherStageV2::ReceiptSigned) => {
                "ui-02-receipt-signed.observation.v1.json"
            }
            Self::SecurityAgentObservation(PublisherStageV2::ProtectedCasSigned) => {
                "ui-03-protected-cas-signed.observation.v1.json"
            }
            Self::SecurityAgentObservation(PublisherStageV2::FinalizationRequestPublished) => {
                "ui-04-finalization-request-published.observation.v1.json"
            }
            Self::SecurityAgentObservation(PublisherStageV2::ResidualRemoved) => {
                "ui-05-residual-removed.observation.v1.json"
            }
            Self::SecurityAgentObservation(PublisherStageV2::TerminalBindingPublished) => {
                "ui-06-terminal-binding-published.observation.v1.json"
            }
            Self::SecurityAgentObservation(PublisherStageV2::RestorationComplete) => {
                "ui-07-restoration-complete.observation.v1.json"
            }
            Self::Progress => "progress.v2.json",
            Self::InputBundle(PublisherStageV2::Prepared) => "input-00-prepared.bundle.v2.json",
            Self::InputBundle(PublisherStageV2::SurrogatesCreated) => {
                "input-01-surrogates-created.bundle.v2.json"
            }
            Self::InputBundle(PublisherStageV2::ReceiptSigned) => {
                "input-02-receipt-signed.bundle.v2.json"
            }
            Self::InputBundle(PublisherStageV2::ProtectedCasSigned) => {
                "input-03-protected-cas-signed.bundle.v2.json"
            }
            Self::InputBundle(PublisherStageV2::FinalizationRequestPublished) => {
                "input-04-finalization-request-published.bundle.v2.json"
            }
            Self::InputBundle(PublisherStageV2::ResidualRemoved) => {
                "input-05-residual-removed.bundle.v2.json"
            }
            Self::InputBundle(PublisherStageV2::TerminalBindingPublished) => {
                "input-06-terminal-binding-published.bundle.v2.json"
            }
            Self::InputBundle(PublisherStageV2::RestorationComplete) => {
                "input-07-restoration-complete.bundle.v2.json"
            }
            Self::StepReceipt(PublisherStageV2::Prepared) => "step-00-prepared.receipt.v2.json",
            Self::StepReceipt(PublisherStageV2::SurrogatesCreated) => {
                "step-01-surrogates-created.receipt.v2.json"
            }
            Self::StepReceipt(PublisherStageV2::ReceiptSigned) => {
                "step-02-receipt-signed.receipt.v2.json"
            }
            Self::StepReceipt(PublisherStageV2::ProtectedCasSigned) => {
                "step-03-protected-cas-signed.receipt.v2.json"
            }
            Self::StepReceipt(PublisherStageV2::FinalizationRequestPublished) => {
                "step-04-finalization-request-published.receipt.v2.json"
            }
            Self::StepReceipt(PublisherStageV2::ResidualRemoved) => {
                "step-05-residual-removed.receipt.v2.json"
            }
            Self::StepReceipt(PublisherStageV2::TerminalBindingPublished) => {
                "step-06-terminal-binding-published.receipt.v2.json"
            }
            Self::StepReceipt(PublisherStageV2::RestorationComplete) => {
                "step-07-restoration-complete.receipt.v2.json"
            }
            Self::AdvanceMarker(PublisherStageV2::Prepared) => "advance-00-prepared.marker.v2.json",
            Self::AdvanceMarker(PublisherStageV2::SurrogatesCreated) => {
                "advance-01-surrogates-created.marker.v2.json"
            }
            Self::AdvanceMarker(PublisherStageV2::ReceiptSigned) => {
                "advance-02-receipt-signed.marker.v2.json"
            }
            Self::AdvanceMarker(PublisherStageV2::ProtectedCasSigned) => {
                "advance-03-protected-cas-signed.marker.v2.json"
            }
            Self::AdvanceMarker(PublisherStageV2::FinalizationRequestPublished) => {
                "advance-04-finalization-request-published.marker.v2.json"
            }
            Self::AdvanceMarker(PublisherStageV2::ResidualRemoved) => {
                "advance-05-residual-removed.marker.v2.json"
            }
            Self::AdvanceMarker(PublisherStageV2::TerminalBindingPublished) => {
                "advance-06-terminal-binding-published.marker.v2.json"
            }
            Self::AdvanceMarker(PublisherStageV2::RestorationComplete) => {
                "advance-07-restoration-complete.marker.v2.json"
            }
        }
    }
}

pub fn installed_finalizer_inbox_path_v2(artifact: PublisherArtifactV2) -> Result<&'static Path> {
    match artifact {
        PublisherArtifactV2::FinalizationRequest => Ok(Path::new(MAC_R3_FINALIZER_REQUEST_PATH_V2)),
        PublisherArtifactV2::TerminalBinding => Ok(Path::new(TERMINAL_BINDING_PATH)),
        _ => bail!("publisher artifact has no finalizer inbox destination"),
    }
}

pub fn external_exchange_path_v2(
    repetition: RepetitionV2,
    artifact: PublisherArtifactV2,
) -> PathBuf {
    Path::new(EXPERIMENT_ROOT_V2)
        .join("repetitions")
        .join(repetition.directory_name())
        .join("publisher-exchange")
        .join(artifact.filename())
}

pub fn global_external_exchange_path_v2(artifact: PublisherArtifactV2) -> Result<PathBuf> {
    if !matches!(
        artifact,
        PublisherArtifactV2::GlobalPreEffectPacket
            | PublisherArtifactV2::CreatorRouteReceiptSet
            | PublisherArtifactV2::NativeEvidenceExport
            | PublisherArtifactV2::NativeEvidenceExportAcknowledgement
            | PublisherArtifactV2::NativeEvidenceCleanupReceipt
            | PublisherArtifactV2::PeerControlRestorationReceipt
    ) {
        bail!("global publisher exchange accepts only the closed pre-effect, evidence-export, acknowledgement, cleanup, and restoration artifacts")
    }
    Ok(Path::new(super::GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2).join(artifact.filename()))
}

pub fn root_publisher_path_v2(repetition: RepetitionV2, artifact: PublisherArtifactV2) -> PathBuf {
    Path::new(DISPOSABLE_PUBLISHER_ROOT_V2)
        .join("repetitions")
        .join(repetition.directory_name())
        .join(artifact.filename())
}

pub fn validate_signed_receipt_output_v2(
    repetition: RepetitionV2,
    creation: &SurrogateCreationReceiptV2,
    binding: &IdentityBindingPacketV2,
    receipt: &PublisherPreRemovalReceiptV2,
) -> Result<()> {
    binding.validate_against_creation(repetition, creation)?;
    if receipt.scope_id != repetition.scope_id()
        || receipt.target_set_kind != TargetSetKindV2::DisposableCapability
        || receipt.current_lock_identity_sha256 != binding.current_lock_identity_sha256
        || receipt.signer_access_control_sha256 != binding.signer_access_control_sha256
        || receipt.publisher_signer_spki_der != creation.target.spki_der
        || receipt.finalizer_identity != binding.finalizer_identity
        || receipt.coordinator_identity != binding.coordinator_identity
        || receipt.coordinator_process != binding.coordinator_process
        || receipt.launch_identity != binding.launch_identity
        || receipt.capability_digest != binding.capability_digest
    {
        bail!("signed receipt differs from the exact publisher identity binding")
    }
    substrate_common::macos_retirement_v2::validate_publisher_pre_removal_receipt_v2(receipt)
        .context("validate exact publisher-signed disposable receipt")
}

pub fn validate_signed_cas_output_v2(
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &substrate_common::macos_retirement_v2::HarnessDurabilityAcknowledgementV2,
    binding: &ProtectedCasBindingV2,
) -> Result<()> {
    substrate_common::macos_retirement_v2::validate_protected_cas_binding_v2(
        binding,
        receipt,
        acknowledgement,
    )
}

pub fn validate_published_request_v2(
    request: &FinalizationRequestV2,
    scope_id: &str,
) -> Result<()> {
    let (receipt, _, _) =
        substrate_common::macos_retirement_v2::validate_finalization_request_v2(request, None)?;
    if receipt.scope_id != scope_id
        || receipt.target_set_kind != TargetSetKindV2::DisposableCapability
    {
        bail!("publisher request publication crossed the disposable scope")
    }
    Ok(())
}

pub fn validate_terminal_publication_v2(
    acknowledgement: &TerminalAcknowledgementV2,
    scope_id: &str,
) -> Result<()> {
    if acknowledgement.scope_id != scope_id {
        bail!("terminal publication crossed the disposable scope")
    }
    Ok(())
}

fn require_header(owner: &str, version: u32, expected_owner: &str) -> Result<()> {
    if owner != expected_owner || version != EXPERIMENT_VERSION_V2 {
        bail!("disposable publisher schema owner or version changed")
    }
    Ok(())
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact lowercase SHA-256 digest")
    }
    Ok(())
}

fn require_base64url(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        bail!("{label} is not unpadded base64url")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;

    use super::*;

    fn digest(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn ui_arm() -> SecurityAgentArmEvidenceV2 {
        let bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(&serde_json::json!({
            "activeTransitionAfterBaseline": false,
            "baseline": {},
            "distinctProcesses": [],
            "distinctWindows": [],
            "newProcessAfterBaseline": false,
            "sampleCount": 101,
            "sampleIntervalMilliseconds": 50,
            "schemaOwner": "substrate.r3-macos-securityagent-observation",
            "schemaVersion": 1,
            "unexpectedUiObserved": false,
            "windowAfterBaseline": false
        }))
        .unwrap();
        let report = super::SecurityAgentRawReportEvidenceV2 {
            raw_report_base64url: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes),
            raw_report_sha256: substrate_common::macos_retirement_v2::sha256_hex_v2(&bytes),
            raw_report_byte_length: u64::try_from(bytes.len()).unwrap(),
        };
        let reports = vec![report];
        SecurityAgentArmEvidenceV2 {
            schema_owner: "substrate.r3-macos-securityagent-arm-observation".to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            observer_path: super::super::SECURITYAGENT_OBSERVER_PATH_V2.to_string(),
            report_set_sha256: document_sha256_v2(&reports).unwrap(),
            reports,
            overlap_rearm_count: 0,
            gap_free_rearm_coverage: true,
            unexpected_ui_observed: false,
        }
    }

    #[test]
    fn publisher_transitions_and_paths_are_frozen() {
        for pair in PUBLISHER_STAGE_SEQUENCE_V2.windows(2) {
            pair[0].require_successor(pair[1]).unwrap();
        }
        assert!(PublisherStageV2::Prepared
            .require_successor(PublisherStageV2::ReceiptSigned)
            .is_err());
        let path =
            external_exchange_path_v2(RepetitionV2::Two, PublisherArtifactV2::IdentityBinding);
        assert_eq!(
            path,
            Path::new(EXPERIMENT_ROOT_V2)
                .join("repetitions")
                .join("02-01a00aa6-5593-7227-8432-87bf380684f1")
                .join("publisher-exchange/identity-binding.v2.json")
        );
        assert!(root_publisher_path_v2(
            RepetitionV2::One,
            PublisherArtifactV2::AdvanceMarker(PublisherStageV2::ProtectedCasSigned)
        )
        .starts_with(DISPOSABLE_PUBLISHER_ROOT_V2));
    }

    #[test]
    fn fixed_protocol_does_not_expose_action_path_or_predicate_fields() {
        let marker = PublisherAdvanceMarkerV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: super::super::EXPERIMENT_ID_V2.to_string(),
            repetition: 1,
            scope_id: RepetitionV2::One.scope_id().to_string(),
            prior_stage: PublisherStageV2::Prepared,
            next_stage: PublisherStageV2::SurrogatesCreated,
            fixed_input_sha256: "11".repeat(32),
        };
        marker.validate(RepetitionV2::One).unwrap();
        let encoded = serde_json::to_value(marker).unwrap();
        let object = encoded.as_object().unwrap();
        for forbidden in ["action", "path", "tag", "service", "account", "predicate"] {
            assert!(!object.contains_key(forbidden));
        }
    }

    #[test]
    fn every_publisher_stage_fixture_exact_binds_progress_input_step_and_ui() {
        let repetition = RepetitionV2::One;
        let mut progress = PublisherProgressV2::genesis(repetition);
        progress.validate(repetition).unwrap();

        for (index, next) in PUBLISHER_STAGE_SEQUENCE_V2
            .iter()
            .copied()
            .skip(1)
            .enumerate()
        {
            let artifacts = expected_input_artifacts_v2(next)
                .iter()
                .copied()
                .enumerate()
                .map(
                    |(artifact_index, artifact)| PublisherInputArtifactDigestV2 {
                        artifact,
                        sha256: digest(u8::try_from(artifact_index + index + 1).unwrap()),
                    },
                )
                .collect();
            let bundle = PublisherInputBundleV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: super::super::EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                prior_stage: progress.stage,
                next_stage: next,
                artifacts,
            };
            bundle.validate(repetition).unwrap();
            let marker = PublisherAdvanceMarkerV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: super::super::EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                prior_stage: progress.stage,
                next_stage: next,
                fixed_input_sha256: document_sha256_v2(&bundle).unwrap(),
            };
            marker.validate_input_bundle(repetition, &bundle).unwrap();
            let arm_evidence = ui_arm();
            let ui = PublisherSecurityAgentObservationV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: super::super::EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                stage: next,
                prearm_baseline_sha256: digest(90),
                arm_evidence_sha256: document_sha256_v2(&arm_evidence).unwrap(),
                arm_evidence,
                observation_started_monotonic_ns: 1,
                observation_finished_monotonic_ns: 2,
                unexpected_ui_observed: false,
            };
            let step = PublisherStepReceiptV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: super::super::EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                prior_stage: progress.stage,
                new_stage: next,
                marker_sha256: document_sha256_v2(&marker).unwrap(),
                output_sha256: digest(u8::try_from(120 + index).unwrap()),
                securityagent_observation_sha256: document_sha256_v2(&ui).unwrap(),
            };
            step.validate(repetition, &marker, &step.output_sha256)
                .unwrap();
            step.validate_securityagent_observation(repetition, &ui)
                .unwrap();

            let successor = PublisherProgressV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: super::super::EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                generation: publisher_stage_generation_v2(next),
                stage: next,
                predecessor_progress_sha256: document_sha256_v2(&progress).unwrap(),
                accepted_input_bundle_sha256: document_sha256_v2(&bundle).unwrap(),
                step_receipt_sha256: document_sha256_v2(&step).unwrap(),
            };
            successor
                .validate_successor(repetition, &progress, &marker, &bundle, &step)
                .unwrap();
            let mut fabricated_successor = successor.clone();
            fabricated_successor.predecessor_progress_sha256 = digest(250);
            assert!(fabricated_successor
                .validate_successor(repetition, &progress, &marker, &bundle, &step)
                .is_err());

            assert!(
                external_exchange_path_v2(repetition, PublisherArtifactV2::InputBundle(next))
                    .ends_with(PublisherArtifactV2::InputBundle(next).filename())
            );
            assert!(
                external_exchange_path_v2(repetition, PublisherArtifactV2::StepReceipt(next))
                    .ends_with(PublisherArtifactV2::StepReceipt(next).filename())
            );
            assert!(external_exchange_path_v2(
                repetition,
                PublisherArtifactV2::SecurityAgentObservation(next)
            )
            .ends_with(PublisherArtifactV2::SecurityAgentObservation(next).filename()));
            progress = successor;
        }
        assert_eq!(progress.stage, PublisherStageV2::RestorationComplete);
        assert_eq!(progress.generation, 8);

        let mut fabricated = progress.clone();
        fabricated.generation = 7;
        assert!(fabricated.validate(repetition).is_err());
    }
}
