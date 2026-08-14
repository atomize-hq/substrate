//! Canonical native-evidence export retained before root journal cleanup.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, parse_canonical_v2, sha256_hex_v2,
    FinalizerResponseStateV2, FinalizerResponseV2, HostTargetRoleV2,
    MAC_R3_COORDINATOR_INBOX_ROOT_V2, MAC_R3_FINALIZER_ENDPOINT_V2,
    MAC_R3_FINALIZER_JOURNAL_ROOT_V2, MAC_R3_FINALIZER_LAUNCHD_LABEL_V2,
    MAC_R3_FINALIZER_REQUEST_PATH_V2, MAC_R3_RETIREMENT_LATCH_ROOT_V2,
    MAC_R3_TERMINAL_BINDING_PATH_V2,
};

use crate::contract::JournalEventKind;
use crate::engine::EffectObservationArtifactV2;
use crate::journal::JournalGeneration;

use super::controls::{
    BoundedRawStreamEvidenceV2, NobodyOwnerAuthorityOperationV2, PeerSubstitutionControlV2,
    SecurityAgentArmEvidenceV2, SecurityAgentRawReportEvidenceV2, TransportControlV2,
    NOBODY_OWNER_AUTHORITY_SEQUENCE_V2, PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2,
    TRANSPORT_CONTROL_SEQUENCE_V2,
};
use super::pre_effect::{CreatorNativeArmV2, CREATOR_NATIVE_ARM_SEQUENCE_V2};
use super::pre_effect::{
    GlobalPreEffectPacketV2, RootInstallCompletionV2, RootInstallPreclaimV2,
    ROOT_INSTALL_CLAIM_ROOT_V2, ROOT_INSTALL_COMPLETION_PATH_V2, ROOT_INSTALL_PRECLAIM_PATH_V2,
};
use super::publisher_protocol::{PublisherStageV2, PUBLISHER_STAGE_SEQUENCE_V2};
use super::{RepetitionV2, EXPERIMENT_ID_V2, EXPERIMENT_VERSION_V2};

pub const NATIVE_EVIDENCE_EXPORT_OWNER_V2: &str =
    "substrate.r3-macos-disposable-native-evidence-export";
const MAX_RAW_JOURNAL_BYTES_V2: usize = 4 * 1024 * 1024;
pub const RUNNER_PRIVATE_ARCHIVE_MAX_BYTES_V2: u64 = 67_108_864;
pub const RUNNER_PRIVATE_ROOT_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2";
const RUNNER_PRIVATE_ARCHIVE_ENTRY_MAX_BYTES_V2: usize = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalJournalGenerationExportV2 {
    pub generation: u64,
    pub canonical_base64url: String,
    pub canonical_sha256: String,
    pub record: JournalGeneration,
}

impl CanonicalJournalGenerationExportV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        let bytes = decode_bounded(&self.canonical_base64url, MAX_RAW_JOURNAL_BYTES_V2)?;
        let parsed: JournalGeneration = parse_canonical_v2(&bytes)?;
        if self.generation == 0
            || self.generation != self.record.generation
            || self.record != parsed
            || self.canonical_sha256 != sha256_hex_v2(&bytes)
            || bytes != canonical_bytes_v2(&self.record)?
            || self.record.scope_id != repetition.scope_id()
        {
            bail!("exported journal generation changed its canonical bytes or scope")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalEffectObservationExportV2 {
    pub effect_ordinal: u16,
    pub canonical_base64url: String,
    pub canonical_sha256: String,
    pub artifact: EffectObservationArtifactV2,
}

impl CanonicalEffectObservationExportV2 {
    pub fn validate(&self) -> Result<()> {
        let bytes = decode_bounded(&self.canonical_base64url, MAX_RAW_JOURNAL_BYTES_V2)?;
        let parsed: EffectObservationArtifactV2 = parse_canonical_v2(&bytes)?;
        if self.effect_ordinal == 0
            || self.effect_ordinal != self.artifact.effect_ordinal
            || self.artifact != parsed
            || self.canonical_sha256 != sha256_hex_v2(&bytes)
            || bytes != canonical_bytes_v2(&self.artifact)?
            || self.artifact.schema_owner != "substrate.mac-r3-finalizer-effect-observation"
            || self.artifact.schema_version != 2
            || self.artifact.invocation_evidence_sha256.is_some()
                != self.artifact.invocation_evidence_canonical_json.is_some()
            || ((matches!(
                self.artifact.effect_role,
                HostTargetRoleV2::Signer
                    | HostTargetRoleV2::CapabilitySignControl
                    | HostTargetRoleV2::CapabilityExportControl
                    | HostTargetRoleV2::CapabilityAclMutationControl
                    | HostTargetRoleV2::CapabilityWrongKeyDeleteControl
            ) || (self.artifact.effect_invocation_attempt > 0
                && !matches!(
                    self.artifact.effect_role,
                    HostTargetRoleV2::PublisherService | HostTargetRoleV2::PublisherEndpoint
                )))
                && self.artifact.invocation_evidence_sha256.is_none())
        {
            bail!("exported EffectObservationArtifactV2 changed its exact canonical record")
        }
        require_digest(&self.artifact.effect_identity_sha256)?;
        require_digest(&self.artifact.state_observation_sha256)?;
        if let (Some(digest), Some(json)) = (
            self.artifact.invocation_evidence_sha256.as_deref(),
            self.artifact.invocation_evidence_canonical_json.as_deref(),
        ) {
            require_digest(digest)?;
            if sha256_hex_v2(json.as_bytes()) != digest {
                bail!("exported native invocation evidence digest changed")
            }
            let _: Value = parse_canonical_v2(json.as_bytes())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RawOsStatusRecordV2 {
    pub effect_ordinal: u16,
    pub json_pointer: String,
    pub raw_os_status: i64,
    pub classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RepetitionNativeEvidenceExportV2 {
    pub repetition: u8,
    pub scope_id: String,
    pub request_digest: String,
    pub complete_response_sha256: String,
    pub journal_head_generation: u64,
    pub journal_head_sha256: String,
    pub generations: Vec<CanonicalJournalGenerationExportV2>,
    pub effect_observations: Vec<CanonicalEffectObservationExportV2>,
    pub raw_os_status_records: Vec<RawOsStatusRecordV2>,
    pub artifact_set_sha256: String,
}

impl RepetitionNativeEvidenceExportV2 {
    pub fn validate(&self, repetition: RepetitionV2, complete: &FinalizerResponseV2) -> Result<()> {
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(complete)?;
        if complete.state != FinalizerResponseStateV2::HostComplete
            || self.request_digest != complete.request_digest
            || self.complete_response_sha256 != document_sha256_v2(complete)?
            || self.generations.is_empty()
            || self.journal_head_generation
                != u64::try_from(self.generations.len()).context("journal length exceeds u64")?
            || self.generations.last().map(|value| &value.canonical_sha256)
                != Some(&self.journal_head_sha256)
            || complete.journal_head_sha256 != self.journal_head_sha256
        {
            bail!("native evidence export is not the exact completed accepted journal")
        }
        let empty_head = "0".repeat(64);
        let mut prior = empty_head.as_str();
        for (index, generation) in self.generations.iter().enumerate() {
            generation.validate(repetition)?;
            if generation.generation
                != u64::try_from(index + 1).expect("bounded generation count fits u64")
                || generation.record.predecessor_head_sha256 != prior
                || generation.record.request_digest != self.request_digest
            {
                bail!("exported journal has a gap, fork, or alternate request")
            }
            prior = &generation.canonical_sha256;
        }
        if self
            .generations
            .first()
            .map(|value| value.record.event.kind)
            != Some(JournalEventKind::FinalizerAccepted)
            || self.generations.last().map(|value| value.record.event.kind)
                != Some(JournalEventKind::Complete)
        {
            bail!("exported journal does not span FinalizerAccepted through Complete")
        }
        for artifact in &self.effect_observations {
            artifact.validate()?;
            let event = self.generations.iter().find(|generation| {
                generation.record.event.kind == JournalEventKind::EffectObserved
                    && generation.record.event.effect_ordinal == Some(artifact.effect_ordinal)
            });
            let Some(event) = event else {
                bail!("exported effect observation lacks its exact journal generation")
            };
            if event.record.event.observation_sha256.as_deref() != Some(&artifact.canonical_sha256)
                || event.record.event.effect_role != Some(artifact.artifact.effect_role)
                || event.record.event.effect_identity_sha256.as_deref()
                    != Some(&artifact.artifact.effect_identity_sha256)
            {
                bail!("exported effect artifact differs from its EffectObserved generation")
            }
        }
        let observed_count = self
            .generations
            .iter()
            .filter(|generation| generation.record.event.kind == JournalEventKind::EffectObserved)
            .count();
        if observed_count != self.effect_observations.len()
            || self.raw_os_status_records
                != derive_raw_os_status_records(&self.effect_observations)?
            || self.artifact_set_sha256
                != repetition_artifact_set_sha256_v2(
                    &self.generations,
                    &self.effect_observations,
                    &self.raw_os_status_records,
                )?
        {
            bail!("native evidence export omitted an effect artifact or raw OSStatus record")
        }
        require_digest(&self.journal_head_sha256)?;
        require_digest(&self.artifact_set_sha256)
    }
}

pub fn build_repetition_native_evidence_export_v2(
    repetition: RepetitionV2,
    complete: &FinalizerResponseV2,
    canonical_generations: Vec<Vec<u8>>,
    canonical_effect_observations: Vec<Vec<u8>>,
) -> Result<RepetitionNativeEvidenceExportV2> {
    let generations = canonical_generations
        .into_iter()
        .map(|bytes| {
            if bytes.is_empty() || bytes.len() > MAX_RAW_JOURNAL_BYTES_V2 {
                bail!("raw journal generation is outside its fixed bound")
            }
            let record: JournalGeneration = parse_canonical_v2(&bytes)?;
            if canonical_bytes_v2(&record)? != bytes {
                bail!("journal generation export input is not exact canonical bytes")
            }
            Ok(CanonicalJournalGenerationExportV2 {
                generation: record.generation,
                canonical_sha256: sha256_hex_v2(&bytes),
                canonical_base64url: URL_SAFE_NO_PAD.encode(bytes),
                record,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let effect_observations = canonical_effect_observations
        .into_iter()
        .map(|bytes| {
            if bytes.is_empty() || bytes.len() > MAX_RAW_JOURNAL_BYTES_V2 {
                bail!("raw effect observation is outside its fixed bound")
            }
            let artifact: EffectObservationArtifactV2 = parse_canonical_v2(&bytes)?;
            if canonical_bytes_v2(&artifact)? != bytes {
                bail!("effect observation export input is not exact canonical bytes")
            }
            Ok(CanonicalEffectObservationExportV2 {
                effect_ordinal: artifact.effect_ordinal,
                canonical_sha256: sha256_hex_v2(&bytes),
                canonical_base64url: URL_SAFE_NO_PAD.encode(bytes),
                artifact,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if effect_observations
        .windows(2)
        .any(|pair| pair[0].effect_ordinal >= pair[1].effect_ordinal)
    {
        bail!("effect observation export inputs are not in strict effect order")
    }
    let raw_os_status_records = derive_raw_os_status_records(&effect_observations)?;
    let artifact_set_sha256 = repetition_artifact_set_sha256_v2(
        &generations,
        &effect_observations,
        &raw_os_status_records,
    )?;
    let head = generations
        .last()
        .context("native evidence export lacks a journal head")?;
    let export = RepetitionNativeEvidenceExportV2 {
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_string(),
        request_digest: complete.request_digest.clone(),
        complete_response_sha256: document_sha256_v2(complete)?,
        journal_head_generation: u64::try_from(generations.len())
            .context("journal generation count exceeds u64")?,
        journal_head_sha256: head.canonical_sha256.clone(),
        generations,
        effect_observations,
        raw_os_status_records,
        artifact_set_sha256,
    };
    export.validate(repetition, complete)?;
    Ok(export)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "arm", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecurityAgentEvidenceArmV2 {
    GlobalNonceBaseline,
    Creator {
        control: CreatorNativeArmV2,
    },
    Publisher {
        stage: PublisherStageV2,
    },
    Transport {
        control: TransportControlV2,
    },
    Peer {
        control: PeerSubstitutionControlV2,
    },
    BenignInjection,
    NobodyOwner {
        operation: NobodyOwnerAuthorityOperationV2,
    },
    FinalizationEffects,
    TerminalBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecurityAgentEvidenceBindingV2 {
    pub repetition: u8,
    pub sequence_ordinal: u16,
    pub arm: SecurityAgentEvidenceArmV2,
    pub source_artifact_sha256: String,
    pub arm_evidence: SecurityAgentArmEvidenceV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecurityAgentExpectedSourceV2 {
    pub repetition: u8,
    pub arm: SecurityAgentEvidenceArmV2,
    pub source_artifact_sha256: String,
    pub arm_evidence_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecurityAgentReportArchiveV2 {
    pub bindings: Vec<SecurityAgentEvidenceBindingV2>,
    pub binding_set_sha256: String,
}

impl SecurityAgentReportArchiveV2 {
    pub fn validate(&self) -> Result<()> {
        let plan = securityagent_evidence_plan_v2();
        if self.bindings.len() != plan.len()
            || self.binding_set_sha256 != document_sha256_v2(&self.bindings)?
        {
            bail!("SecurityAgent archive does not retain the exact complete arm set")
        }
        for (index, (binding, (repetition, arm))) in self.bindings.iter().zip(plan).enumerate() {
            if binding.repetition != repetition
                || binding.sequence_ordinal
                    != u16::try_from(index + 1).expect("bounded UI arm count fits u16")
                || binding.arm != arm
            {
                bail!("SecurityAgent report archive skipped or reordered one native arm")
            }
            require_digest(&binding.source_artifact_sha256)?;
            binding.arm_evidence.validate()?;
        }
        Ok(())
    }
}

pub fn build_securityagent_report_archive_v2(
    mut bindings: Vec<SecurityAgentEvidenceBindingV2>,
) -> Result<SecurityAgentReportArchiveV2> {
    for (index, binding) in bindings.iter_mut().enumerate() {
        binding.sequence_ordinal =
            u16::try_from(index + 1).context("SecurityAgent binding count exceeds u16")?;
    }
    let archive = SecurityAgentReportArchiveV2 {
        binding_set_sha256: document_sha256_v2(&bindings)?,
        bindings,
    };
    archive.validate()?;
    Ok(archive)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunnerPrivateArchiveEntryV2 {
    pub name: String,
    pub canonical_byte_length: u64,
    pub canonical_sha256: String,
    pub physical_identity_sha256: String,
    pub canonical_base64url: String,
}

impl RunnerPrivateArchiveEntryV2 {
    pub fn validate(&self) -> Result<Vec<u8>> {
        if self.name.is_empty()
            || self.name.starts_with('.')
            || self.name.contains(['/', '\0', '\n', '\r'])
            || self.canonical_byte_length == 0
            || self.canonical_byte_length
                > u64::try_from(RUNNER_PRIVATE_ARCHIVE_ENTRY_MAX_BYTES_V2)?
        {
            bail!("runner private archive entry changed its closed name or size")
        }
        require_digest(&self.canonical_sha256)?;
        require_digest(&self.physical_identity_sha256)?;
        let bytes = decode_bounded(
            &self.canonical_base64url,
            RUNNER_PRIVATE_ARCHIVE_ENTRY_MAX_BYTES_V2,
        )?;
        if u64::try_from(bytes.len())? != self.canonical_byte_length
            || sha256_hex_v2(&bytes) != self.canonical_sha256
        {
            bail!("runner private archive entry bytes changed")
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunnerPrivateArchiveV2 {
    pub entries: Vec<RunnerPrivateArchiveEntryV2>,
    pub entry_set_sha256: String,
    pub total_bytes: u64,
    pub maximum_bytes: u64,
}

impl RunnerPrivateArchiveV2 {
    pub fn validate(&self) -> Result<()> {
        if self.entries.is_empty()
            || self.entry_set_sha256 != document_sha256_v2(&self.entries)?
            || self.maximum_bytes != RUNNER_PRIVATE_ARCHIVE_MAX_BYTES_V2
        {
            bail!("runner private archive changed its exact set authority")
        }
        let mut total = 0_u64;
        let mut prior = None;
        for entry in &self.entries {
            if prior.is_some_and(|prior: &str| prior >= entry.name.as_str()) {
                bail!("runner private archive entries are not strictly sorted")
            }
            total = total
                .checked_add(u64::try_from(entry.validate()?.len())?)
                .context("runner private archive byte total overflowed")?;
            prior = Some(entry.name.as_str());
        }
        if total != self.total_bytes || total > self.maximum_bytes {
            bail!("runner private archive exceeded or changed its exact byte total")
        }
        Ok(())
    }
}

pub fn build_runner_private_archive_v2(
    mut entries: Vec<RunnerPrivateArchiveEntryV2>,
) -> Result<RunnerPrivateArchiveV2> {
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    let total_bytes = entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.canonical_byte_length)
            .context("runner private archive byte total overflowed")
    })?;
    let value = RunnerPrivateArchiveV2 {
        entry_set_sha256: document_sha256_v2(&entries)?,
        entries,
        total_bytes,
        maximum_bytes: RUNNER_PRIVATE_ARCHIVE_MAX_BYTES_V2,
    };
    value.validate()?;
    Ok(value)
}

pub fn runner_private_archive_union_v2(
    first: &RunnerPrivateArchiveV2,
    second: &RunnerPrivateArchiveV2,
) -> Result<(String, u32, u64)> {
    first.validate()?;
    second.validate()?;
    let mut entries = first.entries.clone();
    entries.extend(second.entries.clone());
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    if entries
        .windows(2)
        .any(|window| window[0].name >= window[1].name)
    {
        bail!("runner private archives overlap or are not a disjoint exact union")
    }
    let total = first
        .total_bytes
        .checked_add(second.total_bytes)
        .context("runner private archive union byte total overflowed")?;
    if total > RUNNER_PRIVATE_ARCHIVE_MAX_BYTES_V2 {
        bail!("runner private archive union exceeds its exact 64 MiB bound")
    }
    Ok((
        document_sha256_v2(&entries)?,
        u32::try_from(entries.len())?,
        total,
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeEvidenceExportV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetitions: Vec<RepetitionNativeEvidenceExportV2>,
    pub securityagent_archive: SecurityAgentReportArchiveV2,
    pub repetition_artifact_set_sha256: Vec<String>,
    pub raw_securityagent_archive_sha256: String,
    pub global_pre_effect_sha256: String,
    pub journal_root_lock_identity_sha256: String,
    pub root_install_claims_binding_sha256: String,
    pub root_install_preclaim_sha256: String,
    pub root_install_preclaim_canonical_base64url: String,
    pub root_install_completion_sha256: String,
    pub root_install_completion_canonical_base64url: String,
    pub runner_private_archive: RunnerPrivateArchiveV2,
    pub runner_private_archive_sha256: String,
    pub artifact_set_sha256: String,
}

impl NativeEvidenceExportV2 {
    pub fn validate(&self, completes: &[FinalizerResponseV2]) -> Result<()> {
        if self.schema_owner != NATIVE_EVIDENCE_EXPORT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.repetitions.len() != RepetitionV2::ALL.len()
            || completes.len() != RepetitionV2::ALL.len()
        {
            bail!("native evidence export identity or repetition count changed")
        }
        for ((repetition, export), complete) in RepetitionV2::ALL
            .into_iter()
            .zip(&self.repetitions)
            .zip(completes)
        {
            export.validate(repetition, complete)?;
        }
        self.securityagent_archive.validate()?;
        let preclaim_bytes = decode_bounded(
            &self.root_install_preclaim_canonical_base64url,
            MAX_RAW_JOURNAL_BYTES_V2,
        )?;
        let preclaim: RootInstallPreclaimV2 = parse_canonical_v2(&preclaim_bytes)?;
        let completion_bytes = decode_bounded(
            &self.root_install_completion_canonical_base64url,
            MAX_RAW_JOURNAL_BYTES_V2,
        )?;
        let completion: RootInstallCompletionV2 = parse_canonical_v2(&completion_bytes)?;
        if canonical_bytes_v2(&preclaim)? != preclaim_bytes
            || canonical_bytes_v2(&completion)? != completion_bytes
            || self.root_install_preclaim_sha256 != sha256_hex_v2(&preclaim_bytes)
            || self.root_install_completion_sha256 != sha256_hex_v2(&completion_bytes)
        {
            bail!("native evidence export changed the raw root-install claims")
        }
        let repetition_hashes = self
            .repetitions
            .iter()
            .map(|value| value.artifact_set_sha256.clone())
            .collect::<Vec<_>>();
        if self.repetition_artifact_set_sha256 != repetition_hashes
            || self.raw_securityagent_archive_sha256
                != document_sha256_v2(&self.securityagent_archive)?
            || self.artifact_set_sha256
                != document_sha256_v2(&(
                    &self.repetition_artifact_set_sha256,
                    &self.raw_securityagent_archive_sha256,
                    &self.global_pre_effect_sha256,
                    &self.journal_root_lock_identity_sha256,
                    &self.root_install_claims_binding_sha256,
                    &self.root_install_preclaim_sha256,
                    &self.root_install_completion_sha256,
                    &self.runner_private_archive_sha256,
                ))?
        {
            bail!("native evidence export artifact-set binding changed")
        }
        require_digest(&self.journal_root_lock_identity_sha256)?;
        self.runner_private_archive.validate()?;
        if self.runner_private_archive_sha256 != document_sha256_v2(&self.runner_private_archive)? {
            bail!("native evidence export changed its runner private archive")
        }
        Ok(())
    }

    pub fn validate_root_install_claims(
        &self,
        global_pre_effect: &GlobalPreEffectPacketV2,
    ) -> Result<()> {
        let preclaim = canonical_bytes_v2(&global_pre_effect.root_install_claims.preclaim)?;
        let completion = canonical_bytes_v2(&global_pre_effect.root_install_claims.completion)?;
        if self.global_pre_effect_sha256 != document_sha256_v2(global_pre_effect)?
            || self.journal_root_lock_identity_sha256
                != global_pre_effect.journal_root_lock_identity_sha256
            || self.root_install_claims_binding_sha256
                != document_sha256_v2(&global_pre_effect.root_install_claims)?
            || self.root_install_preclaim_sha256 != sha256_hex_v2(&preclaim)
            || self.root_install_completion_sha256 != sha256_hex_v2(&completion)
            || self.root_install_preclaim_canonical_base64url != URL_SAFE_NO_PAD.encode(preclaim)
            || self.root_install_completion_canonical_base64url
                != URL_SAFE_NO_PAD.encode(completion)
        {
            bail!("native evidence export does not exactly bind the acknowledged root install")
        }
        Ok(())
    }

    pub fn validate_securityagent_sources(
        &self,
        expected: &[SecurityAgentExpectedSourceV2],
    ) -> Result<()> {
        if expected.len() != self.securityagent_archive.bindings.len()
            || !self
                .securityagent_archive
                .bindings
                .iter()
                .zip(expected)
                .all(|(binding, source)| {
                    binding.repetition == source.repetition
                        && binding.arm == source.arm
                        && binding.source_artifact_sha256 == source.source_artifact_sha256
                        && document_sha256_v2(&binding.arm_evidence)
                            .is_ok_and(|digest| digest == source.arm_evidence_sha256)
                })
        {
            bail!("raw SecurityAgent archive does not bind the harness-reopened source artifacts")
        }
        for source in expected {
            require_digest(&source.source_artifact_sha256)?;
            require_digest(&source.arm_evidence_sha256)?;
        }
        Ok(())
    }
}

pub fn build_native_evidence_export_v2(
    repetitions: Vec<RepetitionNativeEvidenceExportV2>,
    securityagent_archive: SecurityAgentReportArchiveV2,
    global_pre_effect: &GlobalPreEffectPacketV2,
    runner_private_archive: RunnerPrivateArchiveV2,
) -> Result<NativeEvidenceExportV2> {
    if repetitions.len() != RepetitionV2::ALL.len() {
        bail!("native evidence export builder requires exactly two repetitions")
    }
    for (expected, repetition) in RepetitionV2::ALL.into_iter().zip(&repetitions) {
        expected.validate_binding(repetition.repetition, &repetition.scope_id)?;
        require_digest(&repetition.artifact_set_sha256)?;
    }
    securityagent_archive.validate()?;
    runner_private_archive.validate()?;
    let repetition_artifact_set_sha256 = repetitions
        .iter()
        .map(|value| value.artifact_set_sha256.clone())
        .collect::<Vec<_>>();
    let raw_securityagent_archive_sha256 = document_sha256_v2(&securityagent_archive)?;
    let preclaim = canonical_bytes_v2(&global_pre_effect.root_install_claims.preclaim)?;
    let completion = canonical_bytes_v2(&global_pre_effect.root_install_claims.completion)?;
    let global_pre_effect_sha256 = document_sha256_v2(global_pre_effect)?;
    let journal_root_lock_identity_sha256 =
        global_pre_effect.journal_root_lock_identity_sha256.clone();
    let root_install_claims_binding_sha256 =
        document_sha256_v2(&global_pre_effect.root_install_claims)?;
    let root_install_preclaim_sha256 = sha256_hex_v2(&preclaim);
    let root_install_completion_sha256 = sha256_hex_v2(&completion);
    let runner_private_archive_sha256 = document_sha256_v2(&runner_private_archive)?;
    let artifact_set_sha256 = document_sha256_v2(&(
        &repetition_artifact_set_sha256,
        &raw_securityagent_archive_sha256,
        &global_pre_effect_sha256,
        &journal_root_lock_identity_sha256,
        &root_install_claims_binding_sha256,
        &root_install_preclaim_sha256,
        &root_install_completion_sha256,
        &runner_private_archive_sha256,
    ))?;
    let export = NativeEvidenceExportV2 {
        schema_owner: NATIVE_EVIDENCE_EXPORT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        repetitions,
        securityagent_archive,
        repetition_artifact_set_sha256,
        raw_securityagent_archive_sha256,
        global_pre_effect_sha256,
        journal_root_lock_identity_sha256,
        root_install_claims_binding_sha256,
        root_install_preclaim_sha256,
        root_install_preclaim_canonical_base64url: URL_SAFE_NO_PAD.encode(preclaim),
        root_install_completion_sha256,
        root_install_completion_canonical_base64url: URL_SAFE_NO_PAD.encode(completion),
        runner_private_archive,
        runner_private_archive_sha256,
        artifact_set_sha256,
    };
    export.validate_root_install_claims(global_pre_effect)?;
    Ok(export)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeEvidenceExportAcknowledgementV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub native_evidence_export_sha256: String,
    pub artifact_set_sha256: String,
    pub raw_securityagent_archive_sha256: String,
    pub harness_process_attestation_sha256: String,
    pub canonical_validation_complete: bool,
}

impl NativeEvidenceExportAcknowledgementV2 {
    pub fn validate(
        &self,
        export: &NativeEvidenceExportV2,
        harness_process_attestation_sha256: &str,
    ) -> Result<()> {
        if self.schema_owner != NATIVE_EVIDENCE_EXPORT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.native_evidence_export_sha256 != document_sha256_v2(export)?
            || self.artifact_set_sha256 != export.artifact_set_sha256
            || self.raw_securityagent_archive_sha256 != export.raw_securityagent_archive_sha256
            || self.harness_process_attestation_sha256 != harness_process_attestation_sha256
            || !self.canonical_validation_complete
        {
            bail!("native evidence acknowledgement did not bind exact validated raw artifacts")
        }
        require_digest(&self.harness_process_attestation_sha256)
    }
}

pub fn build_native_evidence_export_acknowledgement_v2(
    export: &NativeEvidenceExportV2,
    harness_process_attestation_sha256: String,
) -> Result<NativeEvidenceExportAcknowledgementV2> {
    let acknowledgement = NativeEvidenceExportAcknowledgementV2 {
        schema_owner: NATIVE_EVIDENCE_EXPORT_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        native_evidence_export_sha256: document_sha256_v2(export)?,
        artifact_set_sha256: export.artifact_set_sha256.clone(),
        raw_securityagent_archive_sha256: export.raw_securityagent_archive_sha256.clone(),
        harness_process_attestation_sha256,
        canonical_validation_complete: true,
    };
    acknowledgement.validate(export, &acknowledgement.harness_process_attestation_sha256)?;
    Ok(acknowledgement)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeEvidenceCleanupReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub native_evidence_export_sha256: String,
    pub acknowledgement_sha256: String,
    pub launchd_label: String,
    pub launchctl_bootout_exit_status: i32,
    pub launchctl_bootout_stdout: BoundedRawStreamEvidenceV2,
    pub launchctl_bootout_stderr: BoundedRawStreamEvidenceV2,
    pub launchctl_print_not_found_exit_status: i32,
    pub launchctl_print_stdout: BoundedRawStreamEvidenceV2,
    pub launchctl_print_stderr: BoundedRawStreamEvidenceV2,
    pub endpoint_path: String,
    pub endpoint_absent_without_cleanup_unlink: bool,
    pub endpoint_absence_observation_sha256: String,
    pub coordinator_inbox_root: String,
    pub request_inbox_path: String,
    pub terminal_inbox_path: String,
    pub request_inbox_absent: bool,
    pub terminal_inbox_absent: bool,
    pub coordinator_inbox_empty: bool,
    pub coordinator_inbox_observation_sha256: String,
    pub scope_ids: Vec<String>,
    pub journal_scope_paths: Vec<String>,
    pub journal_scope_absent: Vec<bool>,
    pub capability_scope_paths: Vec<String>,
    pub capability_scope_absent: Vec<bool>,
    pub latch_paths: Vec<String>,
    pub latch_absent: Vec<bool>,
    pub journal_absence_observation_sha256: Vec<String>,
    pub capability_absence_observation_sha256: Vec<String>,
    pub latch_absence_observation_sha256: Vec<String>,
    pub journal_root_path: String,
    pub journal_root_activation_identity_sha256: String,
    pub journal_root_lock_path: String,
    pub journal_root_lock_identity_sha256: String,
    pub journal_root_lock_absent: bool,
    pub journal_root_lock_absence_observation_sha256: String,
    pub journal_root_empty_before_removal: bool,
    pub journal_root_removed_via_held_parent_dirfd: bool,
    pub journal_root_parent_fsynced: bool,
    pub journal_root_absent: bool,
    pub journal_root_absence_observation_sha256: String,
    pub root_install_claim_root_path: String,
    pub root_install_preclaim_path: String,
    pub root_install_completion_path: String,
    pub root_install_claims_binding_sha256: String,
    pub root_install_preclaim_sha256: String,
    pub root_install_completion_sha256: String,
    pub root_install_claim_root_identity_sha256: String,
    pub root_install_claims_retained: bool,
    pub admin_cleanup_authorized: bool,
    pub root_install_claims_retention_observation_sha256: String,
    pub cleanup_private_archive: RunnerPrivateArchiveV2,
    pub cleanup_private_archive_sha256: String,
    pub runner_private_archive_union_sha256: String,
    pub runner_private_archive_union_cardinality: u32,
    pub runner_private_archive_total_bytes: u64,
    pub runner_private_archive_max_bytes: u64,
    pub native_cleanup_plan_sha256: String,
    pub native_cleanup_step_sha256: Vec<String>,
    pub runner_root_path: String,
    pub runner_root_stable_identity_sha256: String,
    pub runner_root_empty_before_removal: bool,
    pub runner_root_removed_via_held_parent_dirfd: bool,
    pub runner_root_parent_fsynced: bool,
    pub runner_root_absent: bool,
    pub runner_root_absence_observation_sha256: String,
    pub securityagent_report: SecurityAgentRawReportEvidenceV2,
    pub cleanup_artifact_set_sha256: String,
}

impl NativeEvidenceCleanupReceiptV2 {
    pub fn validate(
        &self,
        export: &NativeEvidenceExportV2,
        acknowledgement: &NativeEvidenceExportAcknowledgementV2,
        global_pre_effect: &GlobalPreEffectPacketV2,
    ) -> Result<()> {
        let scopes = RepetitionV2::ALL
            .into_iter()
            .map(|value| value.scope_id().to_string())
            .collect::<Vec<_>>();
        let journal_paths = scopes
            .iter()
            .map(|scope| format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/{scope}"))
            .collect::<Vec<_>>();
        let capability_paths = scopes
            .iter()
            .map(|scope| format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/capability/{scope}"))
            .collect::<Vec<_>>();
        let latch_paths = scopes
            .iter()
            .map(|scope| {
                format!("{MAC_R3_RETIREMENT_LATCH_ROOT_V2}/{scope}.retirement-terminal.v2.latch")
            })
            .collect::<Vec<_>>();
        self.launchctl_bootout_stdout.validate()?;
        self.launchctl_bootout_stderr.validate()?;
        self.launchctl_print_stdout.validate()?;
        self.launchctl_print_stderr.validate()?;
        self.cleanup_private_archive.validate()?;
        let (union_sha256, union_cardinality, union_total_bytes) = runner_private_archive_union_v2(
            &export.runner_private_archive,
            &self.cleanup_private_archive,
        )?;
        if self.schema_owner != NATIVE_EVIDENCE_EXPORT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.native_evidence_export_sha256 != document_sha256_v2(export)?
            || self.acknowledgement_sha256 != document_sha256_v2(acknowledgement)?
            || self.launchd_label != MAC_R3_FINALIZER_LAUNCHD_LABEL_V2
            || self.launchctl_bootout_exit_status != 0
            || self.launchctl_print_not_found_exit_status != 113
            || self.endpoint_path != MAC_R3_FINALIZER_ENDPOINT_V2
            || !self.endpoint_absent_without_cleanup_unlink
            || self.coordinator_inbox_root != MAC_R3_COORDINATOR_INBOX_ROOT_V2
            || self.request_inbox_path != MAC_R3_FINALIZER_REQUEST_PATH_V2
            || self.terminal_inbox_path != MAC_R3_TERMINAL_BINDING_PATH_V2
            || !self.request_inbox_absent
            || !self.terminal_inbox_absent
            || !self.coordinator_inbox_empty
            || self.scope_ids != scopes
            || self.journal_scope_paths != journal_paths
            || self.journal_scope_absent != vec![true; RepetitionV2::ALL.len()]
            || self.capability_scope_paths != capability_paths
            || self.capability_scope_absent != vec![true; RepetitionV2::ALL.len()]
            || self.latch_paths != latch_paths
            || self.latch_absent != vec![true; RepetitionV2::ALL.len()]
            || self.journal_absence_observation_sha256.len() != RepetitionV2::ALL.len()
            || self.capability_absence_observation_sha256.len() != RepetitionV2::ALL.len()
            || self.latch_absence_observation_sha256.len() != RepetitionV2::ALL.len()
            || self.journal_root_path != MAC_R3_FINALIZER_JOURNAL_ROOT_V2
            || self.journal_root_activation_identity_sha256
                != global_pre_effect.activation_membrane_root_identity_sha256
            || self.journal_root_lock_path
                != super::pre_effect::MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2
            || self.journal_root_lock_identity_sha256
                != global_pre_effect.journal_root_lock_identity_sha256
            || export.journal_root_lock_identity_sha256
                != global_pre_effect.journal_root_lock_identity_sha256
            || !self.journal_root_lock_absent
            || !self.journal_root_empty_before_removal
            || !self.journal_root_removed_via_held_parent_dirfd
            || !self.journal_root_parent_fsynced
            || !self.journal_root_absent
            || self.root_install_claim_root_path != ROOT_INSTALL_CLAIM_ROOT_V2
            || self.root_install_preclaim_path != ROOT_INSTALL_PRECLAIM_PATH_V2
            || self.root_install_completion_path != ROOT_INSTALL_COMPLETION_PATH_V2
            || self.root_install_claims_binding_sha256 != export.root_install_claims_binding_sha256
            || self.root_install_preclaim_sha256 != export.root_install_preclaim_sha256
            || self.root_install_completion_sha256 != export.root_install_completion_sha256
            || self.root_install_claim_root_identity_sha256
                != document_sha256_v2(
                    &global_pre_effect
                        .root_install_claims
                        .claim_root_current_identity,
                )?
            || !self.root_install_claims_retained
            || !self.admin_cleanup_authorized
            || self.cleanup_private_archive_sha256
                != document_sha256_v2(&self.cleanup_private_archive)?
            || self.runner_private_archive_union_sha256 != union_sha256
            || self.runner_private_archive_union_cardinality != union_cardinality
            || self.runner_private_archive_total_bytes != union_total_bytes
            || self.runner_private_archive_max_bytes != RUNNER_PRIVATE_ARCHIVE_MAX_BYTES_V2
            || self.native_cleanup_step_sha256.is_empty()
            || self.runner_root_path != RUNNER_PRIVATE_ROOT_V2
            || !self.runner_root_empty_before_removal
            || !self.runner_root_removed_via_held_parent_dirfd
            || !self.runner_root_parent_fsynced
            || !self.runner_root_absent
            || self.cleanup_artifact_set_sha256
                != native_evidence_cleanup_artifact_set_sha256_v2(self)?
        {
            bail!("root journal cleanup did not follow acknowledged raw evidence export")
        }
        for digest in self
            .journal_absence_observation_sha256
            .iter()
            .chain(&self.capability_absence_observation_sha256)
            .chain(&self.latch_absence_observation_sha256)
        {
            require_digest(digest)?;
        }
        for digest in [
            &self.endpoint_absence_observation_sha256,
            &self.coordinator_inbox_observation_sha256,
            &self.journal_root_activation_identity_sha256,
            &self.journal_root_lock_identity_sha256,
            &self.journal_root_lock_absence_observation_sha256,
            &self.journal_root_absence_observation_sha256,
            &self.root_install_claim_root_identity_sha256,
            &self.root_install_claims_retention_observation_sha256,
            &self.cleanup_private_archive_sha256,
            &self.runner_private_archive_union_sha256,
            &self.native_cleanup_plan_sha256,
            &self.runner_root_stable_identity_sha256,
            &self.runner_root_absence_observation_sha256,
        ] {
            require_digest(digest)?;
        }
        for digest in &self.native_cleanup_step_sha256 {
            require_digest(digest)?;
        }
        self.securityagent_report.validate()
    }
}

pub fn native_evidence_cleanup_artifact_set_sha256_v2(
    receipt: &NativeEvidenceCleanupReceiptV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Binding<'a> {
        domain: &'static str,
        native_evidence_export_sha256: &'a str,
        acknowledgement_sha256: &'a str,
        launchd_label: &'a str,
        launchctl_bootout_exit_status: i32,
        launchctl_bootout_stdout_sha256: &'a str,
        launchctl_bootout_stderr_sha256: &'a str,
        launchctl_print_not_found_exit_status: i32,
        launchctl_print_stdout_sha256: &'a str,
        launchctl_print_stderr_sha256: &'a str,
        endpoint_path: &'a str,
        endpoint_absent_without_cleanup_unlink: bool,
        endpoint_absence_observation_sha256: &'a str,
        coordinator_inbox_root: &'a str,
        request_inbox_path: &'a str,
        terminal_inbox_path: &'a str,
        request_inbox_absent: bool,
        terminal_inbox_absent: bool,
        coordinator_inbox_empty: bool,
        coordinator_inbox_observation_sha256: &'a str,
        scope_ids: &'a [String],
        journal_scope_paths: &'a [String],
        journal_scope_absent: &'a [bool],
        journal_absence_observation_sha256: &'a [String],
        capability_scope_paths: &'a [String],
        capability_scope_absent: &'a [bool],
        capability_absence_observation_sha256: &'a [String],
        latch_paths: &'a [String],
        latch_absent: &'a [bool],
        latch_absence_observation_sha256: &'a [String],
        journal_root_path: &'a str,
        journal_root_activation_identity_sha256: &'a str,
        journal_root_lock_path: &'a str,
        journal_root_lock_identity_sha256: &'a str,
        journal_root_lock_absent: bool,
        journal_root_lock_absence_observation_sha256: &'a str,
        journal_root_empty_before_removal: bool,
        journal_root_removed_via_held_parent_dirfd: bool,
        journal_root_parent_fsynced: bool,
        journal_root_absent: bool,
        journal_root_absence_observation_sha256: &'a str,
        root_install_claim_root_path: &'a str,
        root_install_claims_binding_sha256: &'a str,
        root_install_preclaim_sha256: &'a str,
        root_install_completion_sha256: &'a str,
        root_install_claim_root_identity_sha256: &'a str,
        root_install_claims_retained: bool,
        admin_cleanup_authorized: bool,
        root_install_claims_retention_observation_sha256: &'a str,
        cleanup_private_archive_sha256: &'a str,
        runner_private_archive_union_sha256: &'a str,
        runner_private_archive_union_cardinality: u32,
        runner_private_archive_total_bytes: u64,
        runner_private_archive_max_bytes: u64,
        native_cleanup_plan_sha256: &'a str,
        native_cleanup_step_sha256: &'a [String],
        runner_root_path: &'a str,
        runner_root_stable_identity_sha256: &'a str,
        runner_root_empty_before_removal: bool,
        runner_root_removed_via_held_parent_dirfd: bool,
        runner_root_parent_fsynced: bool,
        runner_root_absent: bool,
        runner_root_absence_observation_sha256: &'a str,
        securityagent_report_sha256: &'a str,
    }
    document_sha256_v2(&Binding {
        domain: "substrate.r3-macos-disposable-native-evidence-cleanup-artifact-set.v2",
        native_evidence_export_sha256: &receipt.native_evidence_export_sha256,
        acknowledgement_sha256: &receipt.acknowledgement_sha256,
        launchd_label: &receipt.launchd_label,
        launchctl_bootout_exit_status: receipt.launchctl_bootout_exit_status,
        launchctl_bootout_stdout_sha256: &receipt.launchctl_bootout_stdout.raw_sha256,
        launchctl_bootout_stderr_sha256: &receipt.launchctl_bootout_stderr.raw_sha256,
        launchctl_print_not_found_exit_status: receipt.launchctl_print_not_found_exit_status,
        launchctl_print_stdout_sha256: &receipt.launchctl_print_stdout.raw_sha256,
        launchctl_print_stderr_sha256: &receipt.launchctl_print_stderr.raw_sha256,
        endpoint_path: &receipt.endpoint_path,
        endpoint_absent_without_cleanup_unlink: receipt.endpoint_absent_without_cleanup_unlink,
        endpoint_absence_observation_sha256: &receipt.endpoint_absence_observation_sha256,
        coordinator_inbox_root: &receipt.coordinator_inbox_root,
        request_inbox_path: &receipt.request_inbox_path,
        terminal_inbox_path: &receipt.terminal_inbox_path,
        request_inbox_absent: receipt.request_inbox_absent,
        terminal_inbox_absent: receipt.terminal_inbox_absent,
        coordinator_inbox_empty: receipt.coordinator_inbox_empty,
        coordinator_inbox_observation_sha256: &receipt.coordinator_inbox_observation_sha256,
        scope_ids: &receipt.scope_ids,
        journal_scope_paths: &receipt.journal_scope_paths,
        journal_scope_absent: &receipt.journal_scope_absent,
        journal_absence_observation_sha256: &receipt.journal_absence_observation_sha256,
        capability_scope_paths: &receipt.capability_scope_paths,
        capability_scope_absent: &receipt.capability_scope_absent,
        capability_absence_observation_sha256: &receipt.capability_absence_observation_sha256,
        latch_paths: &receipt.latch_paths,
        latch_absent: &receipt.latch_absent,
        latch_absence_observation_sha256: &receipt.latch_absence_observation_sha256,
        journal_root_path: &receipt.journal_root_path,
        journal_root_activation_identity_sha256: &receipt.journal_root_activation_identity_sha256,
        journal_root_lock_path: &receipt.journal_root_lock_path,
        journal_root_lock_identity_sha256: &receipt.journal_root_lock_identity_sha256,
        journal_root_lock_absent: receipt.journal_root_lock_absent,
        journal_root_lock_absence_observation_sha256: &receipt
            .journal_root_lock_absence_observation_sha256,
        journal_root_empty_before_removal: receipt.journal_root_empty_before_removal,
        journal_root_removed_via_held_parent_dirfd: receipt
            .journal_root_removed_via_held_parent_dirfd,
        journal_root_parent_fsynced: receipt.journal_root_parent_fsynced,
        journal_root_absent: receipt.journal_root_absent,
        journal_root_absence_observation_sha256: &receipt.journal_root_absence_observation_sha256,
        root_install_claim_root_path: &receipt.root_install_claim_root_path,
        root_install_claims_binding_sha256: &receipt.root_install_claims_binding_sha256,
        root_install_preclaim_sha256: &receipt.root_install_preclaim_sha256,
        root_install_completion_sha256: &receipt.root_install_completion_sha256,
        root_install_claim_root_identity_sha256: &receipt.root_install_claim_root_identity_sha256,
        root_install_claims_retained: receipt.root_install_claims_retained,
        admin_cleanup_authorized: receipt.admin_cleanup_authorized,
        root_install_claims_retention_observation_sha256: &receipt
            .root_install_claims_retention_observation_sha256,
        cleanup_private_archive_sha256: &receipt.cleanup_private_archive_sha256,
        runner_private_archive_union_sha256: &receipt.runner_private_archive_union_sha256,
        runner_private_archive_union_cardinality: receipt.runner_private_archive_union_cardinality,
        runner_private_archive_total_bytes: receipt.runner_private_archive_total_bytes,
        runner_private_archive_max_bytes: receipt.runner_private_archive_max_bytes,
        native_cleanup_plan_sha256: &receipt.native_cleanup_plan_sha256,
        native_cleanup_step_sha256: &receipt.native_cleanup_step_sha256,
        runner_root_path: &receipt.runner_root_path,
        runner_root_stable_identity_sha256: &receipt.runner_root_stable_identity_sha256,
        runner_root_empty_before_removal: receipt.runner_root_empty_before_removal,
        runner_root_removed_via_held_parent_dirfd: receipt
            .runner_root_removed_via_held_parent_dirfd,
        runner_root_parent_fsynced: receipt.runner_root_parent_fsynced,
        runner_root_absent: receipt.runner_root_absent,
        runner_root_absence_observation_sha256: &receipt.runner_root_absence_observation_sha256,
        securityagent_report_sha256: &receipt.securityagent_report.raw_report_sha256,
    })
}

pub fn securityagent_evidence_plan_v2() -> Vec<(u8, SecurityAgentEvidenceArmV2)> {
    let mut plan = vec![(0, SecurityAgentEvidenceArmV2::GlobalNonceBaseline)];
    for repetition in 1_u8..=2 {
        for control in CREATOR_NATIVE_ARM_SEQUENCE_V2 {
            plan.push((repetition, SecurityAgentEvidenceArmV2::Creator { control }));
        }
    }
    for repetition in RepetitionV2::ALL {
        for stage in PUBLISHER_STAGE_SEQUENCE_V2.into_iter().skip(1) {
            plan.push((
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::Publisher { stage },
            ));
        }
        for control in TRANSPORT_CONTROL_SEQUENCE_V2 {
            plan.push((
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::Transport { control },
            ));
        }
        for control in PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2 {
            plan.push((
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::Peer { control },
            ));
        }
        plan.push((
            repetition.ordinal(),
            SecurityAgentEvidenceArmV2::BenignInjection,
        ));
        for operation in NOBODY_OWNER_AUTHORITY_SEQUENCE_V2 {
            plan.push((
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::NobodyOwner { operation },
            ));
        }
        plan.push((
            repetition.ordinal(),
            SecurityAgentEvidenceArmV2::FinalizationEffects,
        ));
        plan.push((
            repetition.ordinal(),
            SecurityAgentEvidenceArmV2::TerminalBinding,
        ));
    }
    plan
}

fn repetition_artifact_set_sha256_v2(
    generations: &[CanonicalJournalGenerationExportV2],
    observations: &[CanonicalEffectObservationExportV2],
    os_statuses: &[RawOsStatusRecordV2],
) -> Result<String> {
    document_sha256_v2(&(generations, observations, os_statuses))
}

fn derive_raw_os_status_records(
    observations: &[CanonicalEffectObservationExportV2],
) -> Result<Vec<RawOsStatusRecordV2>> {
    let mut records = Vec::new();
    for observation in observations {
        let Some(json) = observation
            .artifact
            .invocation_evidence_canonical_json
            .as_deref()
        else {
            continue;
        };
        let value: Value = parse_canonical_v2(json.as_bytes())?;
        collect_os_statuses(observation.effect_ordinal, "", &value, &mut records);
    }
    records.sort();
    Ok(records)
}

fn collect_os_statuses(
    effect_ordinal: u16,
    pointer: &str,
    value: &Value,
    records: &mut Vec<RawOsStatusRecordV2>,
) {
    match value {
        Value::Object(object) => {
            if let Some(raw_os_status) = object.get("raw_os_status").and_then(Value::as_i64) {
                records.push(RawOsStatusRecordV2 {
                    effect_ordinal,
                    json_pointer: format!("{pointer}/raw_os_status"),
                    raw_os_status,
                    classification: object
                        .get("classification")
                        .and_then(Value::as_str)
                        .map(ToString::to_string),
                });
            }
            for (key, child) in object {
                collect_os_statuses(effect_ordinal, &format!("{pointer}/{key}"), child, records);
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                collect_os_statuses(
                    effect_ordinal,
                    &format!("{pointer}/{index}"),
                    child,
                    records,
                );
            }
        }
        _ => {}
    }
}

fn decode_bounded(value: &str, maximum: usize) -> Result<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .context("decode raw evidence")?;
    if bytes.is_empty() || bytes.len() > maximum {
        bail!("raw evidence length is outside its fixed bound")
    }
    Ok(bytes)
}

fn require_digest(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("native evidence binding is not one lowercase SHA-256 digest")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn securityagent_archive_plan_is_closed_and_exhaustive() {
        let plan = securityagent_evidence_plan_v2();
        assert_eq!(plan.len(), 69);
        assert_eq!(
            plan.first(),
            Some(&(0, SecurityAgentEvidenceArmV2::GlobalNonceBaseline))
        );
        assert_eq!(
            plan.last(),
            Some(&(2, SecurityAgentEvidenceArmV2::TerminalBinding))
        );
    }

    #[test]
    fn exported_signer_and_capability_observations_require_invocation_evidence() {
        for role in [
            HostTargetRoleV2::Signer,
            HostTargetRoleV2::CapabilitySignControl,
            HostTargetRoleV2::CapabilityExportControl,
            HostTargetRoleV2::CapabilityAclMutationControl,
            HostTargetRoleV2::CapabilityWrongKeyDeleteControl,
        ] {
            let artifact = EffectObservationArtifactV2 {
                schema_owner: "substrate.mac-r3-finalizer-effect-observation".to_owned(),
                schema_version: 2,
                effect_ordinal: 1,
                effect_role: role,
                effect_identity_sha256: "a".repeat(64),
                effect_invocation_attempt: 1,
                state_observation_sha256: "b".repeat(64),
                invocation_evidence_sha256: None,
                invocation_evidence_canonical_json: None,
            };
            let bytes = canonical_bytes_v2(&artifact).unwrap();
            let exported = CanonicalEffectObservationExportV2 {
                effect_ordinal: 1,
                canonical_base64url: URL_SAFE_NO_PAD.encode(&bytes),
                canonical_sha256: sha256_hex_v2(&bytes),
                artifact,
            };
            assert!(exported.validate().is_err());
        }
    }

    #[test]
    fn exported_invoked_filesystem_observation_requires_durability_evidence() {
        let artifact = EffectObservationArtifactV2 {
            schema_owner: "substrate.mac-r3-finalizer-effect-observation".to_owned(),
            schema_version: 2,
            effect_ordinal: 1,
            effect_role: HostTargetRoleV2::ProtectedWrapper,
            effect_identity_sha256: "a".repeat(64),
            effect_invocation_attempt: 1,
            state_observation_sha256: "b".repeat(64),
            invocation_evidence_sha256: None,
            invocation_evidence_canonical_json: None,
        };
        let bytes = canonical_bytes_v2(&artifact).unwrap();
        let exported = CanonicalEffectObservationExportV2 {
            effect_ordinal: 1,
            canonical_base64url: URL_SAFE_NO_PAD.encode(&bytes),
            canonical_sha256: sha256_hex_v2(&bytes),
            artifact,
        };
        assert!(exported.validate().is_err());
    }

    fn private_entry(name: &str, bytes: &[u8]) -> RunnerPrivateArchiveEntryV2 {
        RunnerPrivateArchiveEntryV2 {
            name: name.to_owned(),
            canonical_byte_length: u64::try_from(bytes.len()).unwrap(),
            canonical_sha256: sha256_hex_v2(bytes),
            physical_identity_sha256: "c".repeat(64),
            canonical_base64url: URL_SAFE_NO_PAD.encode(bytes),
        }
    }

    #[test]
    fn runner_private_archives_are_disjoint_sorted_and_bounded_to_64_mib() {
        let first = build_runner_private_archive_v2(vec![private_entry("a.json", b"a")]).unwrap();
        let second = build_runner_private_archive_v2(vec![private_entry("b.json", b"bb")]).unwrap();
        let (digest, cardinality, total) =
            runner_private_archive_union_v2(&first, &second).unwrap();
        assert_eq!(cardinality, 2);
        assert_eq!(total, 3);
        assert_eq!(digest.len(), 64);
        assert_eq!(first.maximum_bytes, 67_108_864);

        let overlap =
            build_runner_private_archive_v2(vec![private_entry("a.json", b"other")]).unwrap();
        assert!(runner_private_archive_union_v2(&first, &overlap).is_err());
        assert!(build_runner_private_archive_v2(Vec::new()).is_err());
        assert!(build_runner_private_archive_v2(vec![private_entry("zero", b"")]).is_err());
    }
}
