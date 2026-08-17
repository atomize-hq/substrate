use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    document_sha256_v2, parse_canonical_bounded_v2, sha256_hex_v2,
    MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
};

use super::evidence_export::GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2;
use super::freeze_manifest::{
    CandidateFreezeCoordinatorProvenanceInputV2, CandidateFreezeGlobalProvenanceInputV2,
    CandidateFreezeManifestInputV2, CandidateFreezeManifestV2,
    CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
    CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2, CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2,
    CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2, CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2,
    CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2, CANDIDATE_FREEZE_MANIFEST_INPUT_PATH_V2,
    CANDIDATE_FREEZE_MANIFEST_PATH_V2, CANDIDATE_FREEZE_REVIEWED_ADMIN_BLOCK_PATH_V2,
};
use super::freeze_provenance::{
    CandidateFreezeBuildInputManifestV2, CandidateFreezeSourceHashesManifestV2,
    CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2,
};
use super::publisher_protocol::{PublisherArtifactV2, PublisherStageV2};
use super::{
    HarnessStageV2, RepetitionV2, DISPOSABLE_HARNESS_UID_V2, EXPERIMENT_ROOT_V2,
    GLOBAL_PUBLISHER_EXCHANGE_GID_V2, GLOBAL_PUBLISHER_EXCHANGE_MODE_V2,
    GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2, GLOBAL_PUBLISHER_EXCHANGE_UID_V2,
};

const DIRECTORY_MODE: u32 = 0o700;
const IMMUTABLE_FILE_MODE: u32 = 0o400;
const PUBLISHER_OUTPUT_MODE: u32 = 0o444;

fn global_publisher_output_maximum_bytes_v2(artifact: PublisherArtifactV2) -> usize {
    if artifact == PublisherArtifactV2::GlobalPreEffectPacket {
        GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2
    } else {
        MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentArtifactV2 {
    HarnessProgress(HarnessStageV2),
    HarnessInputBundle(HarnessStageV2),
    HarnessStepReceipt(HarnessStageV2),
    HarnessSigningSeed,
    HarnessSigningSeedRemoval,
    HarnessProcessAttestation,
    CoordinatorProcessAttestation,
    DisposableDenylistProof,
    Baseline,
    IdentityBinding,
    SignedPublisherReceipt,
    HarnessAcknowledgement,
    SignedProtectedCas,
    FrozenFinalizationRequest,
    TransportControlReceipts,
    EffectsResponse,
    EffectsNativeArmReceipt,
    ParityProof,
    TerminalAcknowledgement,
    TerminalBinding,
    CompleteResponse,
    CompleteNativeArmReceipt,
    RestorationManifest,
}

impl ExperimentArtifactV2 {
    fn relative_path(self) -> String {
        match self {
            Self::HarnessProgress(stage) => format!(
                "journal/progress-{:02}-{}.v2.json",
                harness_stage_ordinal(stage),
                harness_stage_slug(stage)
            ),
            Self::HarnessInputBundle(stage) => format!(
                "journal/input-{:02}-{}.bundle.v2.json",
                harness_stage_ordinal(stage),
                harness_stage_slug(stage)
            ),
            Self::HarnessStepReceipt(stage) => format!(
                "journal/step-{:02}-{}.receipt.v2.json",
                harness_stage_ordinal(stage),
                harness_stage_slug(stage)
            ),
            Self::HarnessSigningSeed => "protected/harness-ed25519-seed.v2.bin".to_string(),
            Self::HarnessSigningSeedRemoval => {
                "journal/harness-ed25519-seed-removal.v2.json".to_string()
            }
            Self::HarnessProcessAttestation => {
                "external/harness-process-attestation.v2.json".to_string()
            }
            Self::CoordinatorProcessAttestation => {
                "external/coordinator-process-attestation.v2.json".to_string()
            }
            Self::DisposableDenylistProof => {
                "external/disposable-denylist-proof.v2.json".to_string()
            }
            Self::Baseline => "external/baseline.v2.json".to_string(),
            Self::IdentityBinding => "external/identity-binding.v2.json".to_string(),
            Self::SignedPublisherReceipt => "external/publisher-receipt.v2.json".to_string(),
            Self::HarnessAcknowledgement => "external/harness-acknowledgement.v2.json".to_string(),
            Self::SignedProtectedCas => "protected/protected-cas-binding.v2.json".to_string(),
            Self::FrozenFinalizationRequest => "inbox/finalization-request.v2.json".to_string(),
            Self::TransportControlReceipts => {
                "external/transport-control-receipts.v2.json".to_string()
            }
            Self::EffectsResponse => "external/effects-response.v2.json".to_string(),
            Self::EffectsNativeArmReceipt => {
                "external/effects-native-arm-receipt.v2.json".to_string()
            }
            Self::ParityProof => "external/parity-proof.v2.json".to_string(),
            Self::TerminalAcknowledgement => {
                "external/terminal-acknowledgement.v2.json".to_string()
            }
            Self::TerminalBinding => "inbox/terminal-binding-request.v2.json".to_string(),
            Self::CompleteResponse => "external/complete-response.v2.json".to_string(),
            Self::CompleteNativeArmReceipt => {
                "external/complete-native-arm-receipt.v2.json".to_string()
            }
            Self::RestorationManifest => "external/restoration-manifest.v2.json".to_string(),
        }
    }
}

fn harness_stage_ordinal(stage: HarnessStageV2) -> usize {
    super::HARNESS_STAGE_SEQUENCE_V2
        .iter()
        .position(|candidate| *candidate == stage)
        .expect("closed harness stage is in its sequence")
}

const fn harness_stage_slug(stage: HarnessStageV2) -> &'static str {
    match stage {
        HarnessStageV2::Prepared => "prepared",
        HarnessStageV2::CoordinatorAttested => "coordinator-attested",
        HarnessStageV2::SurrogatesCreated => "surrogates-created",
        HarnessStageV2::ReceiptExternallyDurable => "receipt-externally-durable",
        HarnessStageV2::AcknowledgementExternallyDurable => "acknowledgement-externally-durable",
        HarnessStageV2::AcknowledgementCasBound => "acknowledgement-cas-bound",
        HarnessStageV2::FinalizationRequestFrozen => "finalization-request-frozen",
        HarnessStageV2::FinalizationRequestPublished => "finalization-request-published",
        HarnessStageV2::FinalizerAccepted => "finalizer-accepted",
        HarnessStageV2::EffectsComplete => "effects-complete",
        HarnessStageV2::HarnessResidualRemoving => "harness-residual-removing",
        HarnessStageV2::ParityExternallyDurable => "parity-externally-durable",
        HarnessStageV2::TerminalAcknowledgementBound => "terminal-acknowledgement-bound",
        HarnessStageV2::TerminalBindingPublished => "terminal-binding-published",
        HarnessStageV2::Complete => "complete",
        HarnessStageV2::RestorationComplete => "restoration-complete",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DurableFileObservationV2 {
    pub absolute_path: String,
    pub sha256: String,
    pub device: u64,
    pub inode: u64,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub mode: u32,
    pub link_count: u64,
    pub size: u64,
}

impl DurableFileObservationV2 {
    pub fn identity_sha256(&self) -> Result<String> {
        document_sha256_v2(self)
    }
}

pub struct ExperimentStoreV2 {
    root: PathBuf,
    expected_uid: u32,
}

impl ExperimentStoreV2 {
    pub fn open_fixed() -> Result<Self> {
        Self::open_at(Path::new(EXPERIMENT_ROOT_V2), DISPOSABLE_HARNESS_UID_V2)
    }

    fn open_at(root: &Path, expected_uid: u32) -> Result<Self> {
        if !root.is_absolute() {
            bail!("disposable experiment root is not absolute")
        }
        let metadata = fs::symlink_metadata(root).context("inspect fixed experiment root")?;
        validate_directory(&metadata, expected_uid, "fixed experiment root")?;
        let canonical = fs::canonicalize(root).context("canonicalize fixed experiment root")?;
        if canonical != root {
            bail!("fixed experiment root has a nonphysical or alternate spelling")
        }
        Ok(Self {
            root: canonical,
            expected_uid,
        })
    }

    pub fn root_identity_sha256(&self) -> Result<String> {
        let metadata = fs::symlink_metadata(&self.root).context("reinspect experiment root")?;
        validate_directory(&metadata, self.expected_uid, "fixed experiment root")?;
        #[derive(Serialize)]
        #[serde(deny_unknown_fields)]
        struct RootIdentity<'a> {
            path: &'a str,
            device: u64,
            inode: u64,
            owner_uid: u32,
            owner_gid: u32,
            mode: u32,
        }
        document_sha256_v2(&RootIdentity {
            path: self
                .root
                .to_str()
                .context("fixed experiment root is not UTF-8")?,
            device: metadata.dev(),
            inode: metadata.ino(),
            owner_uid: metadata.uid(),
            owner_gid: metadata.gid(),
            mode: metadata.mode(),
        })
    }

    pub fn read_candidate_freeze_manifest(
        &self,
    ) -> Result<(CandidateFreezeManifestV2, DurableFileObservationV2)> {
        let path = Path::new(CANDIDATE_FREEZE_MANIFEST_PATH_V2);
        if !path.starts_with(&self.root) {
            bail!("candidate freeze manifest escaped the fixed experiment root")
        }
        let (observation, bytes) = read_exact_file(path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        validate_candidate_freeze_external_observation(&observation)?;
        let manifest = substrate_common::macos_retirement_v2::parse_canonical_v2(&bytes)?;
        Ok((manifest, observation))
    }

    pub fn read_candidate_freeze_manifest_input(
        &self,
    ) -> Result<(CandidateFreezeManifestInputV2, DurableFileObservationV2)> {
        let path = Path::new(CANDIDATE_FREEZE_MANIFEST_INPUT_PATH_V2);
        if !path.starts_with(&self.root) {
            bail!("candidate freeze manifest input escaped the fixed experiment root")
        }
        let (observation, bytes) = read_exact_file(path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        validate_candidate_freeze_external_observation(&observation)?;
        let input =
            serde_json::from_slice(&bytes).context("parse typed candidate manifest input")?;
        Ok((input, observation))
    }

    pub fn read_candidate_freeze_source_hashes(
        &self,
    ) -> Result<(
        CandidateFreezeSourceHashesManifestV2,
        DurableFileObservationV2,
    )> {
        self.read_candidate_freeze_canonical_file(Path::new(CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2))
    }

    pub fn read_candidate_freeze_coordinator_provenance_input(
        &self,
    ) -> Result<(
        CandidateFreezeCoordinatorProvenanceInputV2,
        DurableFileObservationV2,
    )> {
        let path = Path::new(CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2);
        if !path.starts_with(&self.root) {
            bail!("candidate provenance input escaped the fixed experiment root")
        }
        let (observation, bytes) = read_exact_file(path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        validate_candidate_freeze_external_observation(&observation)?;
        let input =
            serde_json::from_slice(&bytes).context("parse typed candidate provenance input")?;
        Ok((input, observation))
    }

    pub fn read_candidate_freeze_global_provenance_input(
        &self,
    ) -> Result<(
        CandidateFreezeGlobalProvenanceInputV2,
        DurableFileObservationV2,
    )> {
        let path = Path::new(CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2);
        if !path.starts_with(&self.root) {
            bail!("global provenance input escaped the fixed experiment root")
        }
        let (observation, bytes) = read_exact_file(path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        validate_candidate_freeze_external_observation(&observation)?;
        let input =
            serde_json::from_slice(&bytes).context("parse typed global provenance input")?;
        Ok((input, observation))
    }

    pub fn read_candidate_freeze_coordinator_build_inputs(
        &self,
    ) -> Result<(
        CandidateFreezeBuildInputManifestV2,
        DurableFileObservationV2,
    )> {
        self.read_candidate_freeze_canonical_file(Path::new(
            CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
        ))
    }

    pub fn read_candidate_freeze_global_build_inputs(
        &self,
    ) -> Result<(
        CandidateFreezeBuildInputManifestV2,
        DurableFileObservationV2,
    )> {
        self.read_candidate_freeze_canonical_file(Path::new(
            CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2,
        ))
    }

    fn read_candidate_freeze_canonical_file<T>(
        &self,
        path: &Path,
    ) -> Result<(T, DurableFileObservationV2)>
    where
        T: serde::de::DeserializeOwned + Serialize,
    {
        if !path.starts_with(&self.root) {
            bail!("candidate freeze supporting manifest escaped the fixed experiment root")
        }
        let (observation, bytes) = read_exact_file(path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        validate_candidate_freeze_external_observation(&observation)?;
        let value = substrate_common::macos_retirement_v2::parse_canonical_v2(&bytes)?;
        Ok((value, observation))
    }

    pub fn read_candidate_freeze_admin_block(&self) -> Result<(Vec<u8>, DurableFileObservationV2)> {
        let path = Path::new(CANDIDATE_FREEZE_REVIEWED_ADMIN_BLOCK_PATH_V2);
        if !path.starts_with(&self.root) {
            bail!("reviewed admin block escaped the fixed experiment root")
        }
        let (observation, bytes) = read_exact_file(path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        validate_candidate_freeze_external_observation(&observation)?;
        if bytes.is_empty() || bytes.len() > 1024 * 1024 {
            bail!("reviewed admin block is empty or exceeds its fixed one-MiB bound")
        }
        Ok((bytes, observation))
    }

    pub fn prepare_repetition(&self, repetition: RepetitionV2) -> Result<()> {
        let root = self
            .root
            .join("repetitions")
            .join(repetition.directory_name());
        self.ensure_owned_directory(&self.root.join("repetitions"))?;
        self.ensure_owned_directory(&root)?;
        for child in [
            "external",
            "protected",
            "inbox",
            "journal",
            "publisher-exchange",
        ] {
            self.ensure_owned_directory(&root.join(child))?;
        }
        sync_directory(&root)
    }

    pub fn artifact_path(
        &self,
        repetition: RepetitionV2,
        artifact: ExperimentArtifactV2,
    ) -> PathBuf {
        self.root
            .join("repetitions")
            .join(repetition.directory_name())
            .join(artifact.relative_path())
    }

    pub fn persist_canonical<T: Serialize>(
        &self,
        repetition: RepetitionV2,
        artifact: ExperimentArtifactV2,
        value: &T,
    ) -> Result<DurableFileObservationV2> {
        let bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(value)?;
        self.persist_bytes(repetition, artifact, &bytes)
    }

    pub fn persist_bytes(
        &self,
        repetition: RepetitionV2,
        artifact: ExperimentArtifactV2,
        bytes: &[u8],
    ) -> Result<DurableFileObservationV2> {
        if bytes.is_empty() {
            bail!("durable experiment artifact cannot be empty")
        }
        self.prepare_repetition(repetition)?;
        let path = self.artifact_path(repetition, artifact);
        let parent = path.parent().context("experiment artifact has no parent")?;
        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .mode(IMMUTABLE_FILE_MODE)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        match options.open(&path) {
            Ok(mut file) => {
                file.write_all(bytes)
                    .context("write durable experiment artifact")?;
                file.sync_all()
                    .context("fsync durable experiment artifact")?;
                drop(file);
                sync_directory(parent)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // Idempotency permits only the exact same immutable bytes.
            }
            Err(error) => return Err(error).context("create durable experiment artifact"),
        }
        let (observed, observed_bytes) =
            read_exact_file(&path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        if observed_bytes != bytes || observed.sha256 != sha256_hex_v2(bytes) {
            bail!("durable experiment artifact differs after reopen")
        }
        sync_directory(parent)?;
        Ok(observed)
    }

    pub fn read_canonical<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        repetition: RepetitionV2,
        artifact: ExperimentArtifactV2,
    ) -> Result<(T, DurableFileObservationV2)> {
        let path = self.artifact_path(repetition, artifact);
        let (observation, bytes) = read_exact_file(&path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        let value = substrate_common::macos_retirement_v2::parse_canonical_v2(&bytes)?;
        Ok((value, observation))
    }

    pub fn load_or_create_signing_seed(
        &self,
        repetition: RepetitionV2,
        generated_seed: [u8; 32],
    ) -> Result<([u8; 32], DurableFileObservationV2)> {
        self.prepare_repetition(repetition)?;
        let path = self.artifact_path(repetition, ExperimentArtifactV2::HarnessSigningSeed);
        match read_exact_file(&path, self.expected_uid, IMMUTABLE_FILE_MODE) {
            Ok((observation, bytes)) => {
                let seed: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("durable harness signing seed is not 32 bytes"))?;
                Ok((seed, observation))
            }
            Err(error)
                if error
                    .chain()
                    .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
                    .any(|cause| cause.kind() == std::io::ErrorKind::NotFound) =>
            {
                let observation = self.persist_bytes(
                    repetition,
                    ExperimentArtifactV2::HarnessSigningSeed,
                    &generated_seed,
                )?;
                Ok((generated_seed, observation))
            }
            Err(error) => Err(error),
        }
    }

    pub fn remove_exact_signing_seed(
        &self,
        repetition: RepetitionV2,
        expected_observation: &DurableFileObservationV2,
    ) -> Result<()> {
        let path = self.artifact_path(repetition, ExperimentArtifactV2::HarnessSigningSeed);
        let (observed, bytes) = read_exact_file(&path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        if &observed != expected_observation || bytes.len() != 32 {
            bail!("harness signing seed changed before exact removal")
        }
        fs::remove_file(&path).context("remove exact completed harness signing seed")?;
        sync_directory(path.parent().context("signing seed lacks a parent")?)?;
        if fs::symlink_metadata(&path).is_ok() {
            bail!("harness signing seed remains after exact completed removal")
        }
        Ok(())
    }

    pub fn read_publisher_output<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        repetition: RepetitionV2,
        artifact: PublisherArtifactV2,
    ) -> Result<(T, DurableFileObservationV2)> {
        if !matches!(
            artifact,
            PublisherArtifactV2::PreparedInput
                | PublisherArtifactV2::PublisherProcessAttestation
                | PublisherArtifactV2::CandidateIdentityPacket
                | PublisherArtifactV2::CreationReceipt
                | PublisherArtifactV2::SignedReceipt
                | PublisherArtifactV2::SignedProtectedCas
                | PublisherArtifactV2::StepReceipt(_)
                | PublisherArtifactV2::Progress
                | PublisherArtifactV2::ResidualCleanupReceipt
                | PublisherArtifactV2::RestorationReceipt
                | PublisherArtifactV2::EmergencyRollbackReceipt
                | PublisherArtifactV2::TransportControlsObservation
                | PublisherArtifactV2::PeerControlRunnerSetReceipt
                | PublisherArtifactV2::PeerControlRestorationReceipt
                | PublisherArtifactV2::SecurityAgentObservation(_)
        ) {
            bail!("publisher output is not one fixed protocol artifact")
        }
        let path = self
            .root
            .join("repetitions")
            .join(repetition.directory_name())
            .join("publisher-exchange")
            .join(artifact.filename());
        let (observation, bytes) = read_exact_file(&path, 0, PUBLISHER_OUTPUT_MODE)?;
        let value = substrate_common::macos_retirement_v2::parse_canonical_v2(&bytes)?;
        Ok((value, observation))
    }

    pub fn read_global_publisher_output<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        artifact: PublisherArtifactV2,
    ) -> Result<(T, DurableFileObservationV2)> {
        if !matches!(
            artifact,
            PublisherArtifactV2::GlobalPreEffectPacket
                | PublisherArtifactV2::CreatorRouteReceiptSet
                | PublisherArtifactV2::NativeEvidenceExport
                | PublisherArtifactV2::NativeEvidenceCleanupReceipt
                | PublisherArtifactV2::PeerControlRestorationReceipt
        ) {
            bail!("global publisher output is not one closed protocol artifact")
        }
        let directory = PathBuf::from(GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2);
        require_global_exchange_directory(&directory)?;
        let path = directory.join(artifact.filename());
        let (observation, bytes) = read_exact_file(&path, 0, PUBLISHER_OUTPUT_MODE)?;
        let value =
            parse_canonical_bounded_v2(&bytes, global_publisher_output_maximum_bytes_v2(artifact))?;
        Ok((value, observation))
    }

    pub fn persist_global_publisher_input<T: Serialize>(
        &self,
        artifact: PublisherArtifactV2,
        value: &T,
    ) -> Result<DurableFileObservationV2> {
        if artifact != PublisherArtifactV2::NativeEvidenceExportAcknowledgement {
            bail!("global harness input is not the one closed native-evidence acknowledgement")
        }
        let bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(value)?;
        let directory = PathBuf::from(GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2);
        require_global_exchange_directory(&directory)?;
        persist_exact_exchange_input(
            &directory.join(artifact.filename()),
            &bytes,
            self.expected_uid,
        )
    }

    pub fn persist_publisher_input<T: Serialize>(
        &self,
        repetition: RepetitionV2,
        artifact: PublisherArtifactV2,
        value: &T,
    ) -> Result<DurableFileObservationV2> {
        if !matches!(
            artifact,
            PublisherArtifactV2::PreparedInput
                | PublisherArtifactV2::IdentityBinding
                | PublisherArtifactV2::UnsignedReceipt
                | PublisherArtifactV2::HarnessAcknowledgement
                | PublisherArtifactV2::UnsignedProtectedCas
                | PublisherArtifactV2::FinalizationRequest
                | PublisherArtifactV2::EffectsCompleteResponse
                | PublisherArtifactV2::TerminalAcknowledgement
                | PublisherArtifactV2::TerminalBinding
                | PublisherArtifactV2::CompleteResponse
                | PublisherArtifactV2::InputBundle(_)
                | PublisherArtifactV2::AdvanceMarker(PublisherStageV2::Prepared)
                | PublisherArtifactV2::AdvanceMarker(PublisherStageV2::SurrogatesCreated)
                | PublisherArtifactV2::AdvanceMarker(PublisherStageV2::ReceiptSigned)
                | PublisherArtifactV2::AdvanceMarker(PublisherStageV2::ProtectedCasSigned)
                | PublisherArtifactV2::AdvanceMarker(
                    PublisherStageV2::FinalizationRequestPublished
                )
                | PublisherArtifactV2::AdvanceMarker(PublisherStageV2::ResidualRemoved)
                | PublisherArtifactV2::AdvanceMarker(PublisherStageV2::TerminalBindingPublished)
                | PublisherArtifactV2::EmergencyRollbackMarker
                | PublisherArtifactV2::PreCreationEmergencyRollbackMarker
                | PublisherArtifactV2::TransportControlsCompleteMarker
                | PublisherArtifactV2::PeerControlSetReceipt
                | PublisherArtifactV2::PeerControlsReadyMarker
        ) {
            bail!("harness publisher input is not one closed transition artifact")
        }
        let bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(value)?;
        self.prepare_repetition(repetition)?;
        let path = self
            .root
            .join("repetitions")
            .join(repetition.directory_name())
            .join("publisher-exchange")
            .join(artifact.filename());
        persist_exact_exchange_input(&path, &bytes, self.expected_uid)
    }

    pub fn read_publisher_input<T: serde::de::DeserializeOwned + Serialize>(
        &self,
        repetition: RepetitionV2,
        artifact: PublisherArtifactV2,
    ) -> Result<(T, DurableFileObservationV2)> {
        if !matches!(
            artifact,
            PublisherArtifactV2::PeerControlSetReceipt
                | PublisherArtifactV2::TransportControlsCompleteMarker
        ) {
            bail!("publisher exchange read is not one fixed coordinator input artifact")
        }
        let path = self
            .root
            .join("repetitions")
            .join(repetition.directory_name())
            .join("publisher-exchange")
            .join(artifact.filename());
        let (observation, bytes) = read_exact_file(&path, self.expected_uid, IMMUTABLE_FILE_MODE)?;
        let value = substrate_common::macos_retirement_v2::parse_canonical_v2(&bytes)?;
        Ok((value, observation))
    }

    fn ensure_owned_directory(&self, path: &Path) -> Result<()> {
        if !path.starts_with(&self.root) {
            bail!("experiment directory escaped the fixed root")
        }
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                validate_directory(&metadata, self.expected_uid, "experiment directory")
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(path).context("create fixed experiment directory")?;
                fs::set_permissions(path, fs::Permissions::from_mode(DIRECTORY_MODE))
                    .context("set fixed experiment directory mode")?;
                sync_directory(
                    path.parent()
                        .context("experiment directory lacks a parent")?,
                )?;
                let metadata = fs::symlink_metadata(path).context("reopen experiment directory")?;
                validate_directory(&metadata, self.expected_uid, "experiment directory")
            }
            Err(error) => Err(error).context("inspect experiment directory"),
        }
    }

    #[cfg(test)]
    fn open_test(root: &Path) -> Result<Self> {
        Self::open_at(root, unsafe { libc::geteuid() })
    }
}

fn read_exact_file(
    path: &Path,
    expected_uid: u32,
    expected_mode: u32,
) -> Result<(DurableFileObservationV2, Vec<u8>)> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .with_context(|| format!("open exact experiment file {}", path.display()))?;
    let before = file.metadata().context("fstat exact experiment file")?;
    validate_file(&before, expected_uid, expected_mode)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .context("read exact experiment file")?;
    let after = file
        .metadata()
        .context("refstat exact experiment file after read")?;
    if physical_file_fields(&before) != physical_file_fields(&after) {
        bail!("exact experiment file changed while reading")
    }
    Ok((observation(path, &before, &bytes)?, bytes))
}

fn persist_exact_exchange_input(
    path: &Path,
    bytes: &[u8],
    expected_uid: u32,
) -> Result<DurableFileObservationV2> {
    let parent = path.parent().context("publisher input lacks a parent")?;
    match read_exact_file(path, expected_uid, IMMUTABLE_FILE_MODE) {
        Ok((observation, existing)) if existing == bytes => return Ok(observation),
        Ok(_) => bail!("publisher input already exists with different immutable bytes"),
        Err(error)
            if error
                .chain()
                .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
                .any(|cause| cause.kind() == std::io::ErrorKind::NotFound) => {}
        Err(error) => return Err(error),
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(IMMUTABLE_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .with_context(|| format!("create exact publisher input {}", path.display()))?;
    file.write_all(bytes)
        .context("write exact publisher input")?;
    file.sync_all().context("fsync exact publisher input")?;
    drop(file);
    sync_directory(parent)?;
    let (observation, reopened) = read_exact_file(path, expected_uid, IMMUTABLE_FILE_MODE)?;
    if reopened != bytes {
        bail!("publisher input differs after durable reopen")
    }
    Ok(observation)
}

fn validate_directory(metadata: &fs::Metadata, expected_uid: u32, label: &str) -> Result<()> {
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != expected_uid
        || metadata.mode() & 0o7777 != DIRECTORY_MODE
    {
        bail!("{label} owner, mode, or type differs")
    }
    Ok(())
}

fn validate_candidate_freeze_external_observation(
    observation: &DurableFileObservationV2,
) -> Result<()> {
    if observation.owner_uid != CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2
        || observation.owner_gid != CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2
        || observation.mode & 0o7777 != IMMUTABLE_FILE_MODE
        || observation.link_count != 1
    {
        bail!("candidate freeze external evidence leaf is not UID501:staff 0400 nlink1")
    }
    Ok(())
}

fn require_global_exchange_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("inspect fixed global publisher exchange")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != GLOBAL_PUBLISHER_EXCHANGE_UID_V2
        || metadata.gid() != GLOBAL_PUBLISHER_EXCHANGE_GID_V2
        || metadata.mode() & 0o7777 != GLOBAL_PUBLISHER_EXCHANGE_MODE_V2
    {
        bail!("global publisher exchange is not exact UID501:wheel 0700")
    }
    Ok(())
}

fn validate_file(metadata: &fs::Metadata, expected_uid: u32, expected_mode: u32) -> Result<()> {
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.uid() != expected_uid
        || metadata.mode() & 0o7777 != expected_mode
        || metadata.nlink() != 1
    {
        bail!("exact experiment file owner, mode, type, or link count differs")
    }
    Ok(())
}

fn physical_file_fields(metadata: &fs::Metadata) -> (u64, u64, u32, u32, u32, u64, u64, i64, i64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.uid(),
        metadata.gid(),
        metadata.mode(),
        metadata.nlink(),
        metadata.size(),
        metadata.mtime(),
        metadata.mtime_nsec(),
    )
}

fn observation(
    path: &Path,
    metadata: &fs::Metadata,
    bytes: &[u8],
) -> Result<DurableFileObservationV2> {
    Ok(DurableFileObservationV2 {
        absolute_path: path
            .to_str()
            .context("experiment artifact path is not UTF-8")?
            .to_string(),
        sha256: sha256_hex_v2(bytes),
        device: metadata.dev(),
        inode: metadata.ino(),
        owner_uid: metadata.uid(),
        owner_gid: metadata.gid(),
        mode: metadata.mode(),
        link_count: metadata.nlink(),
        size: metadata.size(),
    })
}

fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .with_context(|| format!("open directory {} for fsync", path.display()))?
        .sync_all()
        .with_context(|| format!("fsync directory {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_pre_effect_packet_uses_its_dedicated_read_bound() {
        const OBSERVED_PACKET_BYTES: usize = 1_175_768;
        let empty_document = br#"{"payload":""}"#;
        let value = serde_json::json!({
            "payload": "x".repeat(OBSERVED_PACKET_BYTES - empty_document.len()),
        });
        let bytes = substrate_common::macos_retirement_v2::canonical_bytes_v2(&value).unwrap();
        assert_eq!(bytes.len(), OBSERVED_PACKET_BYTES);

        assert!(
            substrate_common::macos_retirement_v2::parse_canonical_v2::<serde_json::Value>(&bytes)
                .is_err()
        );
        assert_eq!(
            global_publisher_output_maximum_bytes_v2(PublisherArtifactV2::GlobalPreEffectPacket),
            GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2
        );
        assert_eq!(
            parse_canonical_bounded_v2::<serde_json::Value>(
                &bytes,
                global_publisher_output_maximum_bytes_v2(
                    PublisherArtifactV2::GlobalPreEffectPacket
                ),
            )
            .unwrap(),
            value
        );

        assert_eq!(
            global_publisher_output_maximum_bytes_v2(PublisherArtifactV2::CreatorRouteReceiptSet),
            MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2
        );
        assert!(parse_canonical_bounded_v2::<serde_json::Value>(
            &bytes,
            global_publisher_output_maximum_bytes_v2(PublisherArtifactV2::CreatorRouteReceiptSet),
        )
        .is_err());
    }

    #[test]
    fn durable_store_is_no_follow_immutable_and_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("experiment");
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(DIRECTORY_MODE)).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let store = ExperimentStoreV2::open_test(&root).unwrap();
        let first = store
            .persist_bytes(RepetitionV2::One, ExperimentArtifactV2::Baseline, b"fixed")
            .unwrap();
        let second = store
            .persist_bytes(RepetitionV2::One, ExperimentArtifactV2::Baseline, b"fixed")
            .unwrap();
        assert_eq!(first, second);
        assert!(store
            .persist_bytes(
                RepetitionV2::One,
                ExperimentArtifactV2::Baseline,
                b"changed"
            )
            .is_err());

        let target = store.artifact_path(RepetitionV2::Two, ExperimentArtifactV2::Baseline);
        store.prepare_repetition(RepetitionV2::Two).unwrap();
        std::os::unix::fs::symlink("/dev/null", &target).unwrap();
        assert!(store
            .persist_bytes(RepetitionV2::Two, ExperimentArtifactV2::Baseline, b"fixed")
            .is_err());
    }

    #[test]
    fn durable_signing_seed_survives_reopen_and_ignores_replacement_entropy() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("experiment");
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(DIRECTORY_MODE)).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let store = ExperimentStoreV2::open_test(&root).unwrap();
        let (first, first_observation) = store
            .load_or_create_signing_seed(RepetitionV2::One, [0x11; 32])
            .unwrap();
        drop(store);

        let reopened = ExperimentStoreV2::open_test(&root).unwrap();
        let (recovered, recovered_observation) = reopened
            .load_or_create_signing_seed(RepetitionV2::One, [0x22; 32])
            .unwrap();
        assert_eq!(first, [0x11; 32]);
        assert_eq!(recovered, first);
        assert_eq!(recovered_observation, first_observation);
        reopened
            .remove_exact_signing_seed(RepetitionV2::One, &recovered_observation)
            .unwrap();
        assert!(!reopened
            .artifact_path(RepetitionV2::One, ExperimentArtifactV2::HarnessSigningSeed)
            .exists());
    }

    #[test]
    fn every_closed_publisher_transition_accepts_only_its_prior_stage_marker_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("experiment");
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(DIRECTORY_MODE)).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let store = ExperimentStoreV2::open_test(&root).unwrap();
        let priors = [
            PublisherStageV2::Prepared,
            PublisherStageV2::SurrogatesCreated,
            PublisherStageV2::ReceiptSigned,
            PublisherStageV2::ProtectedCasSigned,
            PublisherStageV2::FinalizationRequestPublished,
            PublisherStageV2::ResidualRemoved,
            PublisherStageV2::TerminalBindingPublished,
        ];
        for prior in priors {
            store
                .persist_publisher_input(
                    RepetitionV2::One,
                    PublisherArtifactV2::AdvanceMarker(prior),
                    &serde_json::json!({"prior": format!("{prior:?}")}),
                )
                .unwrap();
        }
        assert!(store
            .persist_publisher_input(
                RepetitionV2::One,
                PublisherArtifactV2::AdvanceMarker(PublisherStageV2::RestorationComplete),
                &serde_json::json!({"invalid": true}),
            )
            .is_err());
    }
}
