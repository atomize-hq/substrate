//! Canonical, pre-install candidate freeze packet built before any native experiment effect.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, parse_canonical_v2, sha256_hex_v2,
    MAC_R3_COORDINATOR_INBOX_ROOT_V2, MAC_R3_COORDINATOR_PATH_V2,
    MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2, MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
    MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2, MAC_R3_FINALIZER_ENDPOINT_V2,
    MAC_R3_FINALIZER_JOURNAL_ROOT_V2, MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_PLIST_PATH_V2, MAC_R3_FINALIZER_REQUEST_PATH_V2,
    MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2, MAC_R3_RETIREMENT_LATCH_ROOT_V2,
    MAC_R3_TERMINAL_BINDING_PATH_V2,
};

use super::freeze_provenance::{
    build_candidate_freeze_build_input_manifest_v2,
    build_candidate_freeze_source_hashes_manifest_v2, CandidateFreezeBuildInputManifestInputV2,
    CandidateFreezeBuildInputManifestV2, CandidateFreezeBuildLaneV2,
    CandidateFreezeSourceHashesInputV2, CandidateFreezeSourceHashesManifestV2,
    CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2,
};
use super::{
    RepetitionV2, AD_HOC_HARDENED_RUNTIME_FLAGS_V2, ALTERNATE_COORDINATOR_PATH_V2,
    BENIGN_INJECTION_LIBRARY_PATH_V2, BENIGN_INJECTION_LIBRARY_SIGNING_IDENTIFIER_V2,
    CANDIDATE_IDENTITY_PACKET_PATH_V2, CREATOR_EXECUTABLE_PATH_V2,
    CREATOR_EXECUTABLE_SIGNING_IDENTIFIER_V2, CREATOR_MARKER_ROOT_V2,
    DISPOSABLE_EXPERIMENT_RUNNER_PATH_V2, DISPOSABLE_EXPERIMENT_RUNNER_ROOT_V2,
    DISPOSABLE_EXPERIMENT_RUNNER_SIGNING_IDENTIFIER_V2, DISPOSABLE_HARNESS_PATH_V2,
    DISPOSABLE_HARNESS_SIGNING_IDENTIFIER_V2, DISPOSABLE_PREPARED_INPUT_PATH_V2,
    DISPOSABLE_PUBLISHER_ROOT_V2, EMPTY_ENTITLEMENTS_SHA256_V2, EXPERIMENT_ID_V2,
    EXPERIMENT_VERSION_V2, GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2, NOBODY_OWNER_PROBE_PATH_V2,
    NOBODY_OWNER_PROBE_SIGNING_IDENTIFIER_V2, PEER_CODE_PROBE_PATH_V2,
    PEER_CODE_PROBE_SIGNING_IDENTIFIER_V2, PEER_CONTROL_IDENTITY_PACKET_PATH_V2,
    SECURITYAGENT_OBSERVER_PATH_V2, SECURITYAGENT_OBSERVER_SIGNING_IDENTIFIER_V2,
    WRONG_IDENTITY_EXECUTABLE_PATH_V2, WRONG_IDENTITY_EXECUTABLE_SIGNING_IDENTIFIER_V2,
};

pub const CANDIDATE_FREEZE_MANIFEST_OWNER_V2: &str = "substrate.r3-macos-candidate-freeze-manifest";
pub const CANDIDATE_FREEZE_EXTERNAL_FILE_UID_V2: u32 = 501;
pub const CANDIDATE_FREEZE_EXTERNAL_FILE_GID_V2: u32 = 20;
pub const CANDIDATE_FREEZE_EXTERNAL_FILE_MODE_V2: u32 = 0o400;
pub const CANDIDATE_FREEZE_ARTIFACT_ROOT_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze";
pub const CANDIDATE_FREEZE_MANIFEST_INPUT_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/candidate-manifest-input.v2.json";
pub const CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/coordinator-provenance-input.v2.json";
pub const CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/global-provenance-input.v2.json";
pub const CANDIDATE_FREEZE_MANIFEST_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/candidate-artifact-manifest.v2.json";
pub const CANDIDATE_FREEZE_REVIEWED_ADMIN_BLOCK_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/reviewed-admin-command-block.v2.txt";
pub const CANDIDATE_FREEZE_TEMP_ROOT_V2: &str =
    "/private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2";
pub const CANDIDATE_FREEZE_REPOSITORY_PATH_V2: &str =
    "/Users/spensermcconnell/.codex/worktrees/r3-macos-finalizer-rcv-stack/substrate";
pub const CANDIDATE_FREEZE_BRANCH_V2: &str = "feat/r3-macos-finalizer-rcv-stack";
pub const CANDIDATE_FREEZE_PROCESS_SNAPSHOT_IDENTITY_V2: &str =
    "candidate_preinstall_process_snapshot_v2";
pub const CANDIDATE_FREEZE_ATOMIZE_SUPPORT_ROOT_V2: &str = "/Library/Application Support/Atomize";
pub const CANDIDATE_FREEZE_PRODUCT_SUPPORT_ROOT_V2: &str =
    "/Library/Application Support/Atomize/R3MacEvidenceFinalizer";
pub const CANDIDATE_FREEZE_INSTALLED_SUPPORT_ROOT_V2: &str =
    "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2";
pub const CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/candidate-artifact-manifest.v2.json";
/// Maximum byte length of one candidate artifact that may be installed and reattested.
///
/// This is deliberately distinct from the one-MiB control-document and child-output bounds. It
/// admits the closed multi-megabyte Mach-O executables while keeping manifest-driven allocation
/// bounded at every producer and consumer.
pub const CANDIDATE_FREEZE_INSTALLED_ARTIFACT_MAX_BYTES_V2: u64 = 16 * 1024 * 1024;
pub const CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/coordinator-build-inputs.v2.json";
pub const CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/global-build-inputs.v2.json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeArtifactRoleV2 {
    FinalizerExecutable,
    CoordinatorExecutable,
    DisposableHarnessExecutable,
    PeerCodeProbeExecutable,
    AlternateCoordinatorExecutable,
    CreatorExecutable,
    WrongIdentityExecutable,
    DisposablePublisherExecutable,
    DisposableExperimentRunnerExecutable,
    NobodyOwnerProbeExecutable,
    SecurityAgentObserverExecutable,
    BenignInjectionLibrary,
    LaunchdPlist,
    CapabilityManifest,
}

pub const CANDIDATE_FREEZE_ARTIFACT_SEQUENCE_V2: [CandidateFreezeArtifactRoleV2; 14] = [
    CandidateFreezeArtifactRoleV2::FinalizerExecutable,
    CandidateFreezeArtifactRoleV2::CoordinatorExecutable,
    CandidateFreezeArtifactRoleV2::DisposableHarnessExecutable,
    CandidateFreezeArtifactRoleV2::PeerCodeProbeExecutable,
    CandidateFreezeArtifactRoleV2::AlternateCoordinatorExecutable,
    CandidateFreezeArtifactRoleV2::CreatorExecutable,
    CandidateFreezeArtifactRoleV2::WrongIdentityExecutable,
    CandidateFreezeArtifactRoleV2::DisposablePublisherExecutable,
    CandidateFreezeArtifactRoleV2::DisposableExperimentRunnerExecutable,
    CandidateFreezeArtifactRoleV2::NobodyOwnerProbeExecutable,
    CandidateFreezeArtifactRoleV2::SecurityAgentObserverExecutable,
    CandidateFreezeArtifactRoleV2::BenignInjectionLibrary,
    CandidateFreezeArtifactRoleV2::LaunchdPlist,
    CandidateFreezeArtifactRoleV2::CapabilityManifest,
];

impl CandidateFreezeArtifactRoleV2 {
    pub const fn external_filename(self) -> &'static str {
        match self {
            Self::FinalizerExecutable => "substrate-r3-macos-evidence-finalizer",
            Self::CoordinatorExecutable => "substrate-r3-macos-evidence-coordinator",
            Self::DisposableHarnessExecutable => "substrate-r3-macos-disposable-harness",
            Self::PeerCodeProbeExecutable => "substrate-r3-macos-peer-code-probe",
            Self::AlternateCoordinatorExecutable => {
                "substrate-r3-macos-evidence-coordinator-alternate-path"
            }
            Self::CreatorExecutable => "substrate-r3-macos-signer-acl-creator",
            Self::WrongIdentityExecutable => "substrate-r3-macos-signer-acl-wrong-identity",
            Self::DisposablePublisherExecutable => "substrate-r3-macos-disposable-publisher",
            Self::DisposableExperimentRunnerExecutable => {
                "substrate-r3-macos-disposable-experiment-runner"
            }
            Self::NobodyOwnerProbeExecutable => "substrate-r3-macos-nobody-owner-probe",
            Self::SecurityAgentObserverExecutable => "substrate-r3-macos-securityagent-observer",
            Self::BenignInjectionLibrary => "substrate-r3-macos-benign-injection-probe.dylib",
            Self::LaunchdPlist => "com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist",
            Self::CapabilityManifest => "capability-v2.json",
        }
    }

    pub const fn intended_path(self) -> &'static str {
        match self {
            Self::FinalizerExecutable => MAC_R3_FINALIZER_PATH_V2,
            Self::CoordinatorExecutable => MAC_R3_COORDINATOR_PATH_V2,
            Self::DisposableHarnessExecutable => DISPOSABLE_HARNESS_PATH_V2,
            Self::PeerCodeProbeExecutable => PEER_CODE_PROBE_PATH_V2,
            Self::AlternateCoordinatorExecutable => ALTERNATE_COORDINATOR_PATH_V2,
            Self::CreatorExecutable => CREATOR_EXECUTABLE_PATH_V2,
            Self::WrongIdentityExecutable => WRONG_IDENTITY_EXECUTABLE_PATH_V2,
            Self::DisposablePublisherExecutable => MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
            Self::DisposableExperimentRunnerExecutable => DISPOSABLE_EXPERIMENT_RUNNER_PATH_V2,
            Self::NobodyOwnerProbeExecutable => NOBODY_OWNER_PROBE_PATH_V2,
            Self::SecurityAgentObserverExecutable => SECURITYAGENT_OBSERVER_PATH_V2,
            Self::BenignInjectionLibrary => BENIGN_INJECTION_LIBRARY_PATH_V2,
            Self::LaunchdPlist => MAC_R3_FINALIZER_PLIST_PATH_V2,
            Self::CapabilityManifest => {
                "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/capability-v2.json"
            }
        }
    }

    pub const fn signing_identifier(self) -> Option<&'static str> {
        match self {
            Self::FinalizerExecutable => Some(MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2),
            Self::CoordinatorExecutable | Self::AlternateCoordinatorExecutable => {
                Some(MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2)
            }
            Self::DisposableHarnessExecutable => Some(DISPOSABLE_HARNESS_SIGNING_IDENTIFIER_V2),
            Self::PeerCodeProbeExecutable => Some(PEER_CODE_PROBE_SIGNING_IDENTIFIER_V2),
            Self::CreatorExecutable => Some(CREATOR_EXECUTABLE_SIGNING_IDENTIFIER_V2),
            Self::WrongIdentityExecutable => Some(WRONG_IDENTITY_EXECUTABLE_SIGNING_IDENTIFIER_V2),
            Self::DisposablePublisherExecutable => {
                Some(MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2)
            }
            Self::DisposableExperimentRunnerExecutable => {
                Some(DISPOSABLE_EXPERIMENT_RUNNER_SIGNING_IDENTIFIER_V2)
            }
            Self::NobodyOwnerProbeExecutable => Some(NOBODY_OWNER_PROBE_SIGNING_IDENTIFIER_V2),
            Self::SecurityAgentObserverExecutable => {
                Some(SECURITYAGENT_OBSERVER_SIGNING_IDENTIFIER_V2)
            }
            Self::BenignInjectionLibrary => Some(BENIGN_INJECTION_LIBRARY_SIGNING_IDENTIFIER_V2),
            Self::LaunchdPlist | Self::CapabilityManifest => None,
        }
    }

    pub const fn intended_mode(self) -> u32 {
        match self {
            Self::LaunchdPlist => 0o644,
            Self::CapabilityManifest => 0o400,
            _ => 0o555,
        }
    }

    pub const fn intended_uid(self) -> u32 {
        match self {
            Self::CapabilityManifest => 501,
            _ => 0,
        }
    }

    pub const fn intended_gid(self) -> u32 {
        match self {
            Self::CapabilityManifest => 20,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeArtifactEntryV2 {
    pub role: CandidateFreezeArtifactRoleV2,
    pub external_path: String,
    pub intended_path: String,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub size: u64,
    pub sha256: String,
    pub signing_identifier: Option<String>,
    pub designated_requirement: Option<String>,
    pub cdhash: Option<String>,
    pub code_flags: Option<u32>,
    pub team_id: Option<String>,
    pub entitlements_size: Option<u64>,
    pub entitlements_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeInstallDirectoryV2 {
    pub path: String,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
}

pub fn expected_candidate_freeze_install_directories_v2() -> Vec<CandidateFreezeInstallDirectoryV2>
{
    [
        CANDIDATE_FREEZE_ATOMIZE_SUPPORT_ROOT_V2,
        CANDIDATE_FREEZE_PRODUCT_SUPPORT_ROOT_V2,
        CANDIDATE_FREEZE_INSTALLED_SUPPORT_ROOT_V2,
    ]
    .into_iter()
    .map(|path| CandidateFreezeInstallDirectoryV2 {
        path: path.to_string(),
        uid: 0,
        gid: 0,
        mode: 0o755,
    })
    .collect()
}

impl CandidateFreezeArtifactEntryV2 {
    fn validate(&self, expected_role: CandidateFreezeArtifactRoleV2) -> Result<()> {
        let external_path = format!(
            "{CANDIDATE_FREEZE_ARTIFACT_ROOT_V2}/artifacts/{}",
            expected_role.external_filename()
        );
        if self.role != expected_role
            || self.external_path != external_path
            || self.intended_path != expected_role.intended_path()
            || self.uid != expected_role.intended_uid()
            || self.gid != expected_role.intended_gid()
            || self.mode != expected_role.intended_mode()
            || self.size == 0
            || self.size > CANDIDATE_FREEZE_INSTALLED_ARTIFACT_MAX_BYTES_V2
        {
            bail!("candidate artifact changed its fixed role, path, ownership, mode, or size")
        }
        require_sha256(&self.sha256, "candidate artifact bytes")?;
        match expected_role.signing_identifier() {
            Some(identifier) => {
                if self.signing_identifier.as_deref() != Some(identifier)
                    || self
                        .designated_requirement
                        .as_deref()
                        .is_none_or(str::is_empty)
                    || self.cdhash.as_deref().is_none_or(|value| !is_cdhash(value))
                    || self.code_flags != Some(AD_HOC_HARDENED_RUNTIME_FLAGS_V2)
                    || self.team_id.is_some()
                    || self.entitlements_size != Some(0)
                    || self.entitlements_sha256.as_deref() != Some(EMPTY_ENTITLEMENTS_SHA256_V2)
                {
                    bail!("candidate code artifact changed its exact hardened ad-hoc posture")
                }
            }
            None => {
                if self.signing_identifier.is_some()
                    || self.designated_requirement.is_some()
                    || self.cdhash.is_some()
                    || self.code_flags.is_some()
                    || self.team_id.is_some()
                    || self.entitlements_size.is_some()
                    || self.entitlements_sha256.is_some()
                {
                    bail!("non-code candidate artifact acquired a code-identity surface")
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeAbsenceKindV2 {
    FilesystemPath,
    LaunchdLabel,
    UnixEndpoint,
    ProcessSnapshot,
    ProcessExecutablePath,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "classification", rename_all = "snake_case", deny_unknown_fields)]
pub enum CandidateFreezeAbsenceClassificationV2 {
    PathAbsent,
    LaunchdLabelAbsent { raw_exit_status: i32 },
    EndpointAbsent,
    ProcessSnapshotCaptured,
    ProcessAbsent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeAbsenceProbeV2 {
    pub kind: CandidateFreezeAbsenceKindV2,
    pub identity: String,
    pub classification: CandidateFreezeAbsenceClassificationV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeAbsenceObservationV2 {
    pub kind: CandidateFreezeAbsenceKindV2,
    pub identity: String,
    pub exact_predicate_sha256: String,
    pub classification: CandidateFreezeAbsenceClassificationV2,
    pub observation_sha256: String,
    pub raw_observation_base64url: String,
    pub raw_observation_sha256: String,
    pub raw_observation_byte_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeRawBytesV2 {
    pub base64url: String,
    pub sha256: String,
    pub byte_length: u64,
}

impl CandidateFreezeRawBytesV2 {
    fn validate(&self) -> Result<Vec<u8>> {
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.base64url)
            .context("decode raw candidate-freeze command stream")?;
        if bytes.len() > 256 * 1024
            || self.byte_length
                != u64::try_from(bytes.len()).context("raw command stream length exceeds u64")?
            || self.sha256 != sha256_hex_v2(&bytes)
        {
            bail!("raw candidate-freeze command stream changed its bytes or digest")
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeProcessRecordV2 {
    pub pid: i32,
    pub path: Option<String>,
    pub classification: CandidateFreezeProcessRecordClassificationV2,
    pub raw_errno: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeProcessRecordClassificationV2 {
    PathObserved,
    DisappearedDuringSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeProcessSnapshotV2 {
    pub total_process_count: u64,
    pub process_records: Vec<CandidateFreezeProcessRecordV2>,
    pub process_snapshot: CandidateFreezeRawBytesV2,
    pub process_snapshot_sha256: String,
}

impl CandidateFreezeProcessSnapshotV2 {
    pub(crate) fn validate(&self) -> Result<()> {
        let raw = self.process_snapshot.validate()?;
        let records: Vec<CandidateFreezeProcessRecordV2> = parse_canonical_v2(&raw)?;
        if records.is_empty()
            || canonical_bytes_v2(&records)? != raw
            || records != self.process_records
            || self.total_process_count
                != u64::try_from(records.len()).context("process record count exceeds u64")?
            || self.process_snapshot_sha256 != self.process_snapshot.sha256
        {
            bail!("candidate process snapshot changed its typed canonical records")
        }
        let mut prior_pid = None;
        for record in &records {
            if record.pid <= 0 || prior_pid.is_some_and(|prior| prior >= record.pid) {
                bail!("candidate process snapshot is not strictly PID ordered")
            }
            prior_pid = Some(record.pid);
            match record.classification {
                CandidateFreezeProcessRecordClassificationV2::PathObserved => {
                    let path = record
                        .path
                        .as_deref()
                        .context("observed process record lacks its executable path")?;
                    if record.raw_errno != 0
                        || !std::path::Path::new(path).is_absolute()
                        || path.contains(['\0', '\n', '\r'])
                    {
                        bail!("observed process record changed its exact path classification")
                    }
                }
                CandidateFreezeProcessRecordClassificationV2::DisappearedDuringSnapshot => {
                    if record.path.is_some()
                        || !matches!(record.raw_errno, libc::ESRCH | libc::ENOENT)
                    {
                        bail!("disappeared process record changed its exact errno classification")
                    }
                }
            }
        }
        Ok(())
    }

    fn matching_pids(&self, expected_path: &str) -> Vec<i32> {
        self.process_records
            .iter()
            .filter(|record| record.path.as_deref() == Some(expected_path))
            .map(|record| record.pid)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeProcessEnumerationV2 {
    pub matching_pids: Vec<i32>,
    pub process_snapshot_sha256: String,
}

impl CandidateFreezeProcessEnumerationV2 {
    pub(crate) fn validate(
        &self,
        expected_path: &str,
        snapshot: &CandidateFreezeProcessSnapshotV2,
    ) -> Result<()> {
        if self.process_snapshot_sha256 != snapshot.process_snapshot_sha256 {
            bail!("candidate process enumeration changed its shared snapshot binding")
        }
        let matching = snapshot.matching_pids(expected_path);
        if self.matching_pids != matching || !matching.is_empty() {
            bail!("candidate process snapshot found an exact-path process")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CandidateFreezeRawAbsenceObservationV2 {
    FilesystemPath {
        identity: String,
        lstat_return: i32,
        raw_errno: i32,
        stat_result: Option<String>,
    },
    LaunchdLabel {
        identity: String,
        raw_exit_status: i32,
        stdout: CandidateFreezeRawBytesV2,
        stderr: CandidateFreezeRawBytesV2,
    },
    UnixEndpoint {
        identity: String,
        lstat_return: i32,
        raw_errno: i32,
        stat_result: Option<String>,
    },
    ProcessSnapshot {
        identity: String,
        snapshot: CandidateFreezeProcessSnapshotV2,
    },
    ProcessExecutablePath {
        identity: String,
        enumeration: CandidateFreezeProcessEnumerationV2,
    },
}

impl CandidateFreezeRawAbsenceObservationV2 {
    pub(crate) fn validate(
        &self,
        expected: &CandidateFreezeAbsenceProbeV2,
        shared_process_snapshot: &mut Option<CandidateFreezeProcessSnapshotV2>,
    ) -> Result<()> {
        match (self, expected.kind, expected.classification) {
            (
                Self::FilesystemPath {
                    identity,
                    lstat_return,
                    raw_errno,
                    stat_result,
                },
                CandidateFreezeAbsenceKindV2::FilesystemPath,
                CandidateFreezeAbsenceClassificationV2::PathAbsent,
            )
            | (
                Self::UnixEndpoint {
                    identity,
                    lstat_return,
                    raw_errno,
                    stat_result,
                },
                CandidateFreezeAbsenceKindV2::UnixEndpoint,
                CandidateFreezeAbsenceClassificationV2::EndpointAbsent,
            ) => {
                if identity != &expected.identity
                    || *lstat_return != -1
                    || *raw_errno != libc::ENOENT
                    || stat_result.is_some()
                {
                    bail!("candidate preinstall lstat evidence is not exact ENOENT")
                }
            }
            (
                Self::LaunchdLabel {
                    identity,
                    raw_exit_status,
                    stdout,
                    stderr,
                },
                CandidateFreezeAbsenceKindV2::LaunchdLabel,
                CandidateFreezeAbsenceClassificationV2::LaunchdLabelAbsent {
                    raw_exit_status: expected_status,
                },
            ) => {
                let stdout = stdout.validate()?;
                let stderr = stderr.validate()?;
                let expected_stderr = format!(
                    "Bad request.\nCould not find service \"{identity}\" in domain for system\n"
                );
                if identity != &expected.identity
                    || *raw_exit_status != expected_status
                    || !stdout.is_empty()
                    || stderr != expected_stderr.as_bytes()
                {
                    bail!("candidate preinstall launchd evidence changed its label or raw status")
                }
            }
            (
                Self::ProcessSnapshot { identity, snapshot },
                CandidateFreezeAbsenceKindV2::ProcessSnapshot,
                CandidateFreezeAbsenceClassificationV2::ProcessSnapshotCaptured,
            ) => {
                if identity != &expected.identity || shared_process_snapshot.is_some() {
                    bail!("candidate preinstall process snapshot changed its unique identity")
                }
                snapshot.validate()?;
                *shared_process_snapshot = Some(snapshot.clone());
            }
            (
                Self::ProcessExecutablePath {
                    identity,
                    enumeration,
                },
                CandidateFreezeAbsenceKindV2::ProcessExecutablePath,
                CandidateFreezeAbsenceClassificationV2::ProcessAbsent,
            ) => {
                if identity != &expected.identity {
                    bail!("candidate preinstall process enumeration found an exact-path process")
                }
                enumeration.validate(
                    identity,
                    shared_process_snapshot
                        .as_ref()
                        .context("candidate process enumeration precedes its shared snapshot")?,
                )?;
            }
            _ => bail!("candidate raw absence evidence changed its typed predicate class"),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeRollbackTargetV2 {
    LaunchdService,
    FinalizerEndpoint,
    FinalizerJournalRoot,
    CoordinatorInboxRoot,
    PublisherRoot,
    RunnerRoot,
    PreparedInputPacket,
    CandidateIdentityPacket,
    PeerControlIdentityPacket,
    CreatorMarkerRoot,
    FreezeTemporaryRoot,
    InstalledCandidateManifest,
    InstalledSupportV2Directory,
    InstalledSupportProductDirectory,
    InstalledSupportAtomizeDirectory,
    InstalledArtifact(CandidateFreezeArtifactRoleV2),
    ExternalEvidenceRoot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeRollbackDispositionV2 {
    BootoutThenVerifyAbsent,
    ObserveExactAbsenceOnly,
    RemoveExactThenVerifyAbsent,
    RemoveIfSameEmptyThenVerifyAbsent,
    RetainDurableExternalEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeRollbackPlanEntryV2 {
    pub sequence_ordinal: u16,
    pub target: CandidateFreezeRollbackTargetV2,
    pub disposition: CandidateFreezeRollbackDispositionV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeManifestInputV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub artifact_root: String,
    pub repository_path: String,
    pub repository_branch: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_hashes: CandidateFreezeSourceHashesInputV2,
    pub coordinator_build_inputs: CandidateFreezeBuildInputManifestInputV2,
    pub global_build_inputs: CandidateFreezeBuildInputManifestInputV2,
    pub capability_manifest_sha256: String,
    pub launchd_plist_sha256: String,
    pub install_parent_directories: Vec<CandidateFreezeInstallDirectoryV2>,
    pub artifacts: Vec<CandidateFreezeArtifactEntryV2>,
    pub preinstall_raw_absence_observations: Vec<CandidateFreezeRawAbsenceObservationV2>,
    pub rollback_plan: Vec<CandidateFreezeRollbackPlanEntryV2>,
    pub reviewed_admin_block_path: String,
    pub reviewed_admin_block_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeCoordinatorProvenanceInputV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repository_path: String,
    pub repository_branch: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_hashes: CandidateFreezeSourceHashesInputV2,
    pub coordinator_build_inputs: CandidateFreezeBuildInputManifestInputV2,
    pub capability_manifest_sha256: String,
    pub launchd_plist_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeGlobalProvenanceInputV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub coordinator_provenance_input_sha256: String,
    pub global_build_inputs: CandidateFreezeBuildInputManifestInputV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeManifestV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub artifact_root: String,
    pub repository_path: String,
    pub repository_branch: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_hashes_manifest_path: String,
    pub source_hashes_manifest_sha256: String,
    pub coordinator_build_input_manifest_path: String,
    pub coordinator_build_input_manifest_sha256: String,
    pub coordinator_build_digest: String,
    pub global_build_input_manifest_path: String,
    pub global_build_input_manifest_sha256: String,
    pub global_build_digest: String,
    pub coordinator_provenance_input_path: String,
    pub coordinator_provenance_input_sha256: String,
    pub global_provenance_input_path: String,
    pub global_provenance_input_sha256: String,
    pub installed_manifest_path: String,
    pub capability_manifest_sha256: String,
    pub launchd_plist_sha256: String,
    pub install_parent_directories: Vec<CandidateFreezeInstallDirectoryV2>,
    pub install_parent_directory_set_sha256: String,
    pub artifacts: Vec<CandidateFreezeArtifactEntryV2>,
    pub artifact_set_sha256: String,
    pub preinstall_absence_observations: Vec<CandidateFreezeAbsenceObservationV2>,
    pub preinstall_absence_observation_set_sha256: String,
    pub rollback_plan: Vec<CandidateFreezeRollbackPlanEntryV2>,
    pub rollback_plan_sha256: String,
    pub reviewed_admin_block_path: String,
    pub reviewed_admin_block_sha256: String,
    pub manifest_input_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeSupportingManifestsV2 {
    pub source_hashes: CandidateFreezeSourceHashesManifestV2,
    pub coordinator_build_inputs: CandidateFreezeBuildInputManifestV2,
    pub global_build_inputs: CandidateFreezeBuildInputManifestV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeCoordinatorSupportingManifestsV2 {
    pub source_hashes: CandidateFreezeSourceHashesManifestV2,
    pub coordinator_build_inputs: CandidateFreezeBuildInputManifestV2,
}

impl CandidateFreezeManifestV2 {
    pub fn validate(&self) -> Result<()> {
        if self.schema_owner != CANDIDATE_FREEZE_MANIFEST_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.artifact_root != CANDIDATE_FREEZE_ARTIFACT_ROOT_V2
            || self.repository_path != CANDIDATE_FREEZE_REPOSITORY_PATH_V2
            || self.repository_branch != CANDIDATE_FREEZE_BRANCH_V2
            || self.source_hashes_manifest_path != CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2
            || self.coordinator_build_input_manifest_path
                != CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2
            || self.global_build_input_manifest_path != CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2
            || self.coordinator_provenance_input_path
                != CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2
            || self.global_provenance_input_path != CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2
            || self.installed_manifest_path != CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2
            || self.reviewed_admin_block_path != CANDIDATE_FREEZE_REVIEWED_ADMIN_BLOCK_PATH_V2
            || !is_git_object_id(&self.source_commit)
            || !is_git_object_id(&self.source_tree)
            || self.artifacts.len() != CANDIDATE_FREEZE_ARTIFACT_SEQUENCE_V2.len()
        {
            bail!("candidate freeze manifest changed its fixed repository, paths, or identity")
        }
        for (entry, role) in self
            .artifacts
            .iter()
            .zip(CANDIDATE_FREEZE_ARTIFACT_SEQUENCE_V2)
        {
            entry.validate(role)?;
        }
        let capability = self
            .artifacts
            .iter()
            .find(|entry| entry.role == CandidateFreezeArtifactRoleV2::CapabilityManifest)
            .context("candidate freeze lacks capability manifest artifact")?;
        let plist = self
            .artifacts
            .iter()
            .find(|entry| entry.role == CandidateFreezeArtifactRoleV2::LaunchdPlist)
            .context("candidate freeze lacks launchd plist artifact")?;
        if self.capability_manifest_sha256 != capability.sha256
            || self.launchd_plist_sha256 != plist.sha256
            || self.artifact_set_sha256 != candidate_freeze_artifact_set_sha256_v2(&self.artifacts)?
            || self.install_parent_directories != expected_candidate_freeze_install_directories_v2()
            || self.install_parent_directory_set_sha256
                != document_sha256_v2(&self.install_parent_directories)?
        {
            bail!("candidate artifact set changed its capability, plist, or aggregate digest")
        }
        let expected_absence = expected_candidate_freeze_absence_plan_v2();
        if self.preinstall_absence_observations.len() != expected_absence.len()
            || self.preinstall_absence_observation_set_sha256
                != document_sha256_v2(&self.preinstall_absence_observations)?
        {
            bail!("candidate freeze omitted an exact preinstall absence observation")
        }
        let mut shared_process_snapshot = None;
        for (observation, expected) in self
            .preinstall_absence_observations
            .iter()
            .zip(&expected_absence)
        {
            if observation.kind != expected.kind
                || observation.identity != expected.identity
                || observation.classification != expected.classification
                || observation.exact_predicate_sha256 != document_sha256_v2(expected)?
            {
                bail!("candidate freeze changed or reordered a preinstall absence predicate")
            }
            require_sha256(
                &observation.observation_sha256,
                "preinstall absence observation",
            )?;
            let raw = URL_SAFE_NO_PAD
                .decode(&observation.raw_observation_base64url)
                .context("decode raw preinstall absence observation")?;
            if raw.is_empty()
                || raw.len() > 1024 * 1024
                || observation.raw_observation_byte_length
                    != u64::try_from(raw.len())
                        .context("raw preinstall observation length exceeds u64")?
                || observation.raw_observation_sha256 != sha256_hex_v2(&raw)
            {
                bail!("raw preinstall absence observation changed its bounded bytes or digest")
            }
            let typed: CandidateFreezeRawAbsenceObservationV2 = parse_canonical_v2(&raw)?;
            if canonical_bytes_v2(&typed)? != raw {
                bail!("raw preinstall absence observation is not canonical")
            }
            typed.validate(expected, &mut shared_process_snapshot)?;
            if observation.observation_sha256
                != document_sha256_v2(&(
                    &observation.exact_predicate_sha256,
                    &observation.raw_observation_sha256,
                    observation.classification,
                ))?
            {
                bail!("preinstall observation digest does not bind its raw typed evidence")
            }
        }
        if shared_process_snapshot.is_none() {
            bail!("candidate freeze omitted its shared preinstall process snapshot")
        }
        let expected_rollback = expected_candidate_freeze_rollback_plan_v2();
        if self.rollback_plan != expected_rollback
            || self.rollback_plan_sha256 != document_sha256_v2(&expected_rollback)?
        {
            bail!("candidate freeze changed its closed rollback plan")
        }
        for (value, label) in [
            (
                &self.source_hashes_manifest_sha256,
                "source hashes manifest",
            ),
            (
                &self.coordinator_build_input_manifest_sha256,
                "coordinator build inputs",
            ),
            (&self.coordinator_build_digest, "coordinator build digest"),
            (
                &self.global_build_input_manifest_sha256,
                "global build inputs",
            ),
            (&self.global_build_digest, "global build digest"),
            (
                &self.coordinator_provenance_input_sha256,
                "coordinator provenance input",
            ),
            (
                &self.global_provenance_input_sha256,
                "global provenance input",
            ),
            (&self.capability_manifest_sha256, "capability manifest"),
            (&self.launchd_plist_sha256, "launchd plist"),
            (&self.artifact_set_sha256, "candidate artifact set"),
            (
                &self.install_parent_directory_set_sha256,
                "install parent directory set",
            ),
            (
                &self.preinstall_absence_observation_set_sha256,
                "preinstall absence set",
            ),
            (&self.rollback_plan_sha256, "rollback plan"),
            (&self.reviewed_admin_block_sha256, "reviewed admin block"),
            (&self.manifest_input_sha256, "candidate manifest input"),
        ] {
            require_sha256(value, label)?;
        }
        Ok(())
    }
}

pub fn build_candidate_freeze_manifest_v2(
    input: CandidateFreezeManifestInputV2,
) -> Result<CandidateFreezeManifestV2> {
    if input.schema_owner != CANDIDATE_FREEZE_MANIFEST_OWNER_V2
        || input.schema_version != EXPERIMENT_VERSION_V2
        || input.experiment_id != EXPERIMENT_ID_V2
    {
        bail!("candidate manifest input owner, version, or experiment changed")
    }
    let supporting = build_candidate_freeze_supporting_manifests_v2(&input)?;
    let coordinator_provenance = candidate_freeze_coordinator_provenance_input_v2(&input);
    let global_provenance = candidate_freeze_global_provenance_input_v2(&input)?;
    let source_hashes_manifest_sha256 = document_sha256_v2(&supporting.source_hashes)?;
    let coordinator_build_input_manifest_sha256 =
        document_sha256_v2(&supporting.coordinator_build_inputs)?;
    let global_build_input_manifest_sha256 = document_sha256_v2(&supporting.global_build_inputs)?;
    let manifest_input_sha256 = document_sha256_v2(&input)?;
    let artifact_set_sha256 = candidate_freeze_artifact_set_sha256_v2(&input.artifacts)?;
    let expected_absences = expected_candidate_freeze_absence_plan_v2();
    if input.preinstall_raw_absence_observations.len() != expected_absences.len() {
        bail!("candidate manifest input omitted a raw preinstall absence result")
    }
    let mut shared_process_snapshot = None;
    let preinstall_absence_observations = input
        .preinstall_raw_absence_observations
        .iter()
        .zip(&expected_absences)
        .map(|(raw, expected)| {
            raw.validate(expected, &mut shared_process_snapshot)?;
            let raw_bytes = canonical_bytes_v2(raw)?;
            let raw_observation_sha256 = sha256_hex_v2(&raw_bytes);
            let exact_predicate_sha256 = document_sha256_v2(expected)?;
            let observation_sha256 = document_sha256_v2(&(
                &exact_predicate_sha256,
                &raw_observation_sha256,
                expected.classification,
            ))?;
            Ok(CandidateFreezeAbsenceObservationV2 {
                kind: expected.kind,
                identity: expected.identity.clone(),
                exact_predicate_sha256,
                classification: expected.classification,
                observation_sha256,
                raw_observation_base64url: URL_SAFE_NO_PAD.encode(&raw_bytes),
                raw_observation_sha256,
                raw_observation_byte_length: u64::try_from(raw_bytes.len())
                    .context("raw preinstall observation length exceeds u64")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if shared_process_snapshot.is_none() {
        bail!("candidate manifest input omitted its shared process snapshot")
    }
    let preinstall_absence_observation_set_sha256 =
        document_sha256_v2(&preinstall_absence_observations)?;
    let rollback_plan_sha256 = document_sha256_v2(&input.rollback_plan)?;
    let manifest = CandidateFreezeManifestV2 {
        schema_owner: input.schema_owner,
        schema_version: input.schema_version,
        experiment_id: input.experiment_id,
        artifact_root: input.artifact_root,
        repository_path: input.repository_path,
        repository_branch: input.repository_branch,
        source_commit: input.source_commit,
        source_tree: input.source_tree,
        source_hashes_manifest_path: CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2.to_string(),
        source_hashes_manifest_sha256,
        coordinator_build_input_manifest_path: CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2
            .to_string(),
        coordinator_build_input_manifest_sha256: coordinator_build_input_manifest_sha256.clone(),
        coordinator_build_digest: supporting.coordinator_build_inputs.input_set_sha256.clone(),
        global_build_input_manifest_path: CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2.to_string(),
        global_build_input_manifest_sha256: global_build_input_manifest_sha256.clone(),
        global_build_digest: supporting.global_build_inputs.input_set_sha256.clone(),
        coordinator_provenance_input_path: CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2
            .to_string(),
        coordinator_provenance_input_sha256: document_sha256_v2(&coordinator_provenance)?,
        global_provenance_input_path: CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2.to_string(),
        global_provenance_input_sha256: document_sha256_v2(&global_provenance)?,
        installed_manifest_path: CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2.to_string(),
        capability_manifest_sha256: input.capability_manifest_sha256,
        launchd_plist_sha256: input.launchd_plist_sha256,
        install_parent_directory_set_sha256: document_sha256_v2(&input.install_parent_directories)?,
        install_parent_directories: input.install_parent_directories,
        artifacts: input.artifacts,
        artifact_set_sha256,
        preinstall_absence_observations,
        preinstall_absence_observation_set_sha256,
        rollback_plan: input.rollback_plan,
        rollback_plan_sha256,
        reviewed_admin_block_path: input.reviewed_admin_block_path,
        reviewed_admin_block_sha256: input.reviewed_admin_block_sha256,
        manifest_input_sha256,
    };
    manifest.validate()?;
    Ok(manifest)
}

pub fn build_candidate_freeze_supporting_manifests_v2(
    input: &CandidateFreezeManifestInputV2,
) -> Result<CandidateFreezeSupportingManifestsV2> {
    let coordinator_provenance = candidate_freeze_coordinator_provenance_input_v2(input);
    let global_provenance = candidate_freeze_global_provenance_input_v2(input)?;
    let coordinator =
        build_candidate_freeze_coordinator_supporting_manifests_v2(&coordinator_provenance)?;
    let global_build_inputs = build_candidate_freeze_global_build_input_manifest_v2(
        &coordinator_provenance,
        &global_provenance,
        &coordinator,
    )?;
    let prerequisite = global_build_inputs
        .coordinator_prerequisite
        .as_ref()
        .context("global build manifest lacks its coordinator prerequisite")?;
    let artifact = input
        .artifacts
        .iter()
        .find(|value| value.role == CandidateFreezeArtifactRoleV2::CoordinatorExecutable)
        .context("candidate artifacts lack the frozen coordinator prerequisite")?;
    if prerequisite.executable_sha256 != artifact.sha256
        || prerequisite.executable_size != artifact.size
        || prerequisite.external_path != artifact.external_path
        || prerequisite.intended_path != artifact.intended_path
        || Some(prerequisite.signing_identifier.as_str()) != artifact.signing_identifier.as_deref()
        || Some(prerequisite.designated_requirement.as_str())
            != artifact.designated_requirement.as_deref()
        || Some(prerequisite.cdhash.as_str()) != artifact.cdhash.as_deref()
        || Some(prerequisite.code_flags) != artifact.code_flags
        || prerequisite.team_id != artifact.team_id
        || Some(prerequisite.entitlements_size) != artifact.entitlements_size
        || Some(prerequisite.entitlements_sha256.as_str())
            != artifact.entitlements_sha256.as_deref()
    {
        bail!("global build coordinator prerequisite differs from the final frozen artifact")
    }
    Ok(CandidateFreezeSupportingManifestsV2 {
        source_hashes: coordinator.source_hashes,
        coordinator_build_inputs: coordinator.coordinator_build_inputs,
        global_build_inputs,
    })
}

pub fn candidate_freeze_coordinator_provenance_input_v2(
    input: &CandidateFreezeManifestInputV2,
) -> CandidateFreezeCoordinatorProvenanceInputV2 {
    CandidateFreezeCoordinatorProvenanceInputV2 {
        schema_owner: CANDIDATE_FREEZE_MANIFEST_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        repository_path: input.repository_path.clone(),
        repository_branch: input.repository_branch.clone(),
        source_commit: input.source_commit.clone(),
        source_tree: input.source_tree.clone(),
        source_hashes: input.source_hashes.clone(),
        coordinator_build_inputs: input.coordinator_build_inputs.clone(),
        capability_manifest_sha256: input.capability_manifest_sha256.clone(),
        launchd_plist_sha256: input.launchd_plist_sha256.clone(),
    }
}

pub fn candidate_freeze_global_provenance_input_v2(
    input: &CandidateFreezeManifestInputV2,
) -> Result<CandidateFreezeGlobalProvenanceInputV2> {
    let coordinator = candidate_freeze_coordinator_provenance_input_v2(input);
    Ok(CandidateFreezeGlobalProvenanceInputV2 {
        schema_owner: CANDIDATE_FREEZE_MANIFEST_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        coordinator_provenance_input_sha256: document_sha256_v2(&coordinator)?,
        global_build_inputs: input.global_build_inputs.clone(),
    })
}

pub fn build_candidate_freeze_coordinator_supporting_manifests_v2(
    input: &CandidateFreezeCoordinatorProvenanceInputV2,
) -> Result<CandidateFreezeCoordinatorSupportingManifestsV2> {
    if input.schema_owner != CANDIDATE_FREEZE_MANIFEST_OWNER_V2
        || input.schema_version != EXPERIMENT_VERSION_V2
        || input.experiment_id != EXPERIMENT_ID_V2
        || input.repository_path != CANDIDATE_FREEZE_REPOSITORY_PATH_V2
        || input.repository_branch != CANDIDATE_FREEZE_BRANCH_V2
        || !is_git_object_id(&input.source_commit)
        || !is_git_object_id(&input.source_tree)
    {
        bail!("candidate provenance input changed its fixed repository identity")
    }
    let source_hashes = build_candidate_freeze_source_hashes_manifest_v2(
        input.source_hashes.clone(),
        &input.repository_path,
        &input.source_commit,
        &input.source_tree,
    )?;
    let source_hashes_sha256 = document_sha256_v2(&source_hashes)?;
    let coordinator_build_inputs = build_candidate_freeze_build_input_manifest_v2(
        input.coordinator_build_inputs.clone(),
        &input.repository_path,
        &input.source_commit,
        &input.source_tree,
        &source_hashes_sha256,
        &input.capability_manifest_sha256,
        &input.launchd_plist_sha256,
    )?;
    if coordinator_build_inputs.lane != CandidateFreezeBuildLaneV2::Coordinator {
        bail!("candidate coordinator provenance changed its closed build lane")
    }
    Ok(CandidateFreezeCoordinatorSupportingManifestsV2 {
        source_hashes,
        coordinator_build_inputs,
    })
}

pub fn build_candidate_freeze_global_build_input_manifest_v2(
    coordinator_input: &CandidateFreezeCoordinatorProvenanceInputV2,
    global_input: &CandidateFreezeGlobalProvenanceInputV2,
    coordinator: &CandidateFreezeCoordinatorSupportingManifestsV2,
) -> Result<CandidateFreezeBuildInputManifestV2> {
    if global_input.schema_owner != CANDIDATE_FREEZE_MANIFEST_OWNER_V2
        || global_input.schema_version != EXPERIMENT_VERSION_V2
        || global_input.experiment_id != EXPERIMENT_ID_V2
        || global_input.coordinator_provenance_input_sha256
            != document_sha256_v2(coordinator_input)?
    {
        bail!("global provenance input does not bind the exact coordinator stage")
    }
    let source_hashes_sha256 = document_sha256_v2(&coordinator.source_hashes)?;
    let global = build_candidate_freeze_build_input_manifest_v2(
        global_input.global_build_inputs.clone(),
        &coordinator_input.repository_path,
        &coordinator_input.source_commit,
        &coordinator_input.source_tree,
        &source_hashes_sha256,
        &coordinator_input.capability_manifest_sha256,
        &coordinator_input.launchd_plist_sha256,
    )?;
    if global.lane != CandidateFreezeBuildLaneV2::Global {
        bail!("global provenance input changed its closed build lane")
    }
    global
        .coordinator_prerequisite
        .as_ref()
        .context("global build lacks the typed coordinator prerequisite")?
        .validate_for(
            &coordinator.source_hashes,
            &coordinator.coordinator_build_inputs,
        )?;
    Ok(global)
}

pub fn candidate_freeze_artifact_set_sha256_v2(
    artifacts: &[CandidateFreezeArtifactEntryV2],
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Binding<'a> {
        domain: &'static str,
        artifacts: &'a [CandidateFreezeArtifactEntryV2],
    }
    document_sha256_v2(&Binding {
        domain: "substrate.r3-macos-candidate-freeze-artifact-set.v2",
        artifacts,
    })
}

pub fn expected_candidate_freeze_absence_plan_v2() -> Vec<CandidateFreezeAbsenceProbeV2> {
    use CandidateFreezeAbsenceClassificationV2 as C;
    use CandidateFreezeAbsenceKindV2 as K;
    let mut values = CANDIDATE_FREEZE_ARTIFACT_SEQUENCE_V2
        .into_iter()
        .filter(|role| *role != CandidateFreezeArtifactRoleV2::CapabilityManifest)
        .map(|role| CandidateFreezeAbsenceProbeV2 {
            kind: K::FilesystemPath,
            identity: role.intended_path().to_string(),
            classification: C::PathAbsent,
        })
        .collect::<Vec<_>>();
    for path in [
        CANDIDATE_FREEZE_ATOMIZE_SUPPORT_ROOT_V2,
        CANDIDATE_FREEZE_PRODUCT_SUPPORT_ROOT_V2,
        CANDIDATE_FREEZE_INSTALLED_SUPPORT_ROOT_V2,
        CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2,
        DISPOSABLE_PREPARED_INPUT_PATH_V2,
        CANDIDATE_IDENTITY_PACKET_PATH_V2,
        PEER_CONTROL_IDENTITY_PACKET_PATH_V2,
        MAC_R3_COORDINATOR_INBOX_ROOT_V2,
        MAC_R3_FINALIZER_REQUEST_PATH_V2,
        MAC_R3_TERMINAL_BINDING_PATH_V2,
        MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
        MAC_R3_RETIREMENT_LATCH_ROOT_V2,
        DISPOSABLE_PUBLISHER_ROOT_V2,
        DISPOSABLE_EXPERIMENT_RUNNER_ROOT_V2,
        CREATOR_MARKER_ROOT_V2,
        CANDIDATE_FREEZE_TEMP_ROOT_V2,
        GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2,
    ] {
        values.push(CandidateFreezeAbsenceProbeV2 {
            kind: K::FilesystemPath,
            identity: path.to_string(),
            classification: C::PathAbsent,
        });
    }
    for repetition in RepetitionV2::ALL {
        values.push(CandidateFreezeAbsenceProbeV2 {
            kind: K::FilesystemPath,
            identity: format!(
                "{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/{}",
                repetition.scope_id()
            ),
            classification: C::PathAbsent,
        });
        values.push(CandidateFreezeAbsenceProbeV2 {
            kind: K::FilesystemPath,
            identity: format!(
                "{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/capability/{}",
                repetition.scope_id()
            ),
            classification: C::PathAbsent,
        });
        values.push(CandidateFreezeAbsenceProbeV2 {
            kind: K::FilesystemPath,
            identity: format!(
                "{MAC_R3_RETIREMENT_LATCH_ROOT_V2}/{}.retirement-terminal.v2.latch",
                repetition.scope_id()
            ),
            classification: C::PathAbsent,
        });
    }
    values.push(CandidateFreezeAbsenceProbeV2 {
        kind: K::LaunchdLabel,
        identity: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_string(),
        classification: C::LaunchdLabelAbsent {
            raw_exit_status: 113,
        },
    });
    values.push(CandidateFreezeAbsenceProbeV2 {
        kind: K::UnixEndpoint,
        identity: MAC_R3_FINALIZER_ENDPOINT_V2.to_string(),
        classification: C::EndpointAbsent,
    });
    values.push(CandidateFreezeAbsenceProbeV2 {
        kind: K::ProcessSnapshot,
        identity: CANDIDATE_FREEZE_PROCESS_SNAPSHOT_IDENTITY_V2.to_string(),
        classification: C::ProcessSnapshotCaptured,
    });
    for role in CANDIDATE_FREEZE_ARTIFACT_SEQUENCE_V2 {
        if role.signing_identifier().is_some() {
            values.push(CandidateFreezeAbsenceProbeV2 {
                kind: K::ProcessExecutablePath,
                identity: role.intended_path().to_string(),
                classification: C::ProcessAbsent,
            });
        }
    }
    values
}

pub fn expected_candidate_freeze_rollback_plan_v2() -> Vec<CandidateFreezeRollbackPlanEntryV2> {
    use CandidateFreezeRollbackDispositionV2 as D;
    use CandidateFreezeRollbackTargetV2 as T;
    let mut targets = vec![
        (T::LaunchdService, D::BootoutThenVerifyAbsent),
        (T::FinalizerEndpoint, D::ObserveExactAbsenceOnly),
        (T::FinalizerJournalRoot, D::ObserveExactAbsenceOnly),
        (T::CoordinatorInboxRoot, D::RemoveExactThenVerifyAbsent),
        (T::PublisherRoot, D::RemoveExactThenVerifyAbsent),
        (T::RunnerRoot, D::RemoveExactThenVerifyAbsent),
        (T::PreparedInputPacket, D::RemoveExactThenVerifyAbsent),
        (T::CandidateIdentityPacket, D::RemoveExactThenVerifyAbsent),
        (T::PeerControlIdentityPacket, D::RemoveExactThenVerifyAbsent),
        (T::CreatorMarkerRoot, D::RemoveExactThenVerifyAbsent),
        (T::FreezeTemporaryRoot, D::RemoveExactThenVerifyAbsent),
        (
            T::InstalledCandidateManifest,
            D::RemoveExactThenVerifyAbsent,
        ),
    ];
    targets.extend(
        CANDIDATE_FREEZE_ARTIFACT_SEQUENCE_V2
            .into_iter()
            .filter(|role| *role != CandidateFreezeArtifactRoleV2::CapabilityManifest)
            .rev()
            .map(|role| (T::InstalledArtifact(role), D::RemoveExactThenVerifyAbsent)),
    );
    targets.extend([
        (
            T::InstalledSupportV2Directory,
            D::RemoveIfSameEmptyThenVerifyAbsent,
        ),
        (
            T::InstalledSupportProductDirectory,
            D::RemoveIfSameEmptyThenVerifyAbsent,
        ),
        (
            T::InstalledSupportAtomizeDirectory,
            D::RemoveIfSameEmptyThenVerifyAbsent,
        ),
    ]);
    targets.push((T::ExternalEvidenceRoot, D::RetainDurableExternalEvidence));
    targets
        .into_iter()
        .enumerate()
        .map(
            |(index, (target, disposition))| CandidateFreezeRollbackPlanEntryV2 {
                sequence_ordinal: u16::try_from(index + 1)
                    .expect("candidate rollback plan length fits u16"),
                target,
                disposition,
            },
        )
        .collect()
}

fn is_git_object_id(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_cdhash(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not one lowercase SHA-256 digest")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unsigned_artifact(
        role: CandidateFreezeArtifactRoleV2,
        size: u64,
    ) -> CandidateFreezeArtifactEntryV2 {
        CandidateFreezeArtifactEntryV2 {
            role,
            external_path: format!(
                "{CANDIDATE_FREEZE_ARTIFACT_ROOT_V2}/artifacts/{}",
                role.external_filename()
            ),
            intended_path: role.intended_path().to_owned(),
            uid: role.intended_uid(),
            gid: role.intended_gid(),
            mode: role.intended_mode(),
            size,
            sha256: "00".repeat(32),
            signing_identifier: None,
            designated_requirement: None,
            cdhash: None,
            code_flags: None,
            team_id: None,
            entitlements_size: None,
            entitlements_sha256: None,
        }
    }

    #[test]
    fn installed_artifact_schema_has_a_distinct_compiled_ceiling() {
        assert!(
            std::hint::black_box(CANDIDATE_FREEZE_INSTALLED_ARTIFACT_MAX_BYTES_V2) > 1024 * 1024
        );
        assert!(unsigned_artifact(
            CandidateFreezeArtifactRoleV2::LaunchdPlist,
            CANDIDATE_FREEZE_INSTALLED_ARTIFACT_MAX_BYTES_V2,
        )
        .validate(CandidateFreezeArtifactRoleV2::LaunchdPlist)
        .is_ok());
        assert!(unsigned_artifact(
            CandidateFreezeArtifactRoleV2::LaunchdPlist,
            CANDIDATE_FREEZE_INSTALLED_ARTIFACT_MAX_BYTES_V2 + 1,
        )
        .validate(CandidateFreezeArtifactRoleV2::LaunchdPlist)
        .is_err());
    }

    #[test]
    fn freeze_plan_is_closed_and_retains_only_external_evidence() {
        let rollback = expected_candidate_freeze_rollback_plan_v2();
        assert_eq!(
            rollback.last().map(|entry| entry.disposition),
            Some(CandidateFreezeRollbackDispositionV2::RetainDurableExternalEvidence)
        );
        assert_eq!(
            rollback
                .iter()
                .filter(|entry| {
                    entry.disposition
                        == CandidateFreezeRollbackDispositionV2::RetainDurableExternalEvidence
                })
                .count(),
            1
        );
        let absences = expected_candidate_freeze_absence_plan_v2();
        assert!(absences.iter().any(|entry| {
            entry.kind == CandidateFreezeAbsenceKindV2::LaunchdLabel
                && entry.identity == MAC_R3_FINALIZER_LAUNCHD_LABEL_V2
                && entry.classification
                    == CandidateFreezeAbsenceClassificationV2::LaunchdLabelAbsent {
                        raw_exit_status: 113,
                    }
        }));
        assert!(absences.iter().all(|entry| !entry
            .identity
            .contains("019ff983-39ca-7182-9db8-b86aa66fa443")));
    }

    #[test]
    fn process_absence_retains_and_recomputes_every_typed_raw_record() {
        let records = vec![
            CandidateFreezeProcessRecordV2 {
                pid: 1,
                path: Some("/sbin/launchd".to_owned()),
                classification: CandidateFreezeProcessRecordClassificationV2::PathObserved,
                raw_errno: 0,
            },
            CandidateFreezeProcessRecordV2 {
                pid: 41,
                path: None,
                classification:
                    CandidateFreezeProcessRecordClassificationV2::DisappearedDuringSnapshot,
                raw_errno: libc::ESRCH,
            },
        ];
        let raw = canonical_bytes_v2(&records).unwrap();
        let mut snapshot = CandidateFreezeProcessSnapshotV2 {
            total_process_count: 2,
            process_records: records,
            process_snapshot: CandidateFreezeRawBytesV2 {
                base64url: URL_SAFE_NO_PAD.encode(&raw),
                sha256: sha256_hex_v2(&raw),
                byte_length: u64::try_from(raw.len()).unwrap(),
            },
            process_snapshot_sha256: sha256_hex_v2(&raw),
        };
        let mut enumeration = CandidateFreezeProcessEnumerationV2 {
            matching_pids: Vec::new(),
            process_snapshot_sha256: sha256_hex_v2(&raw),
        };
        snapshot.validate().unwrap();
        enumeration
            .validate("/fixed/experiment-binary", &snapshot)
            .unwrap();

        snapshot.process_records[1].raw_errno = libc::EPERM;
        assert!(snapshot
            .validate()
            .unwrap_err()
            .to_string()
            .contains("typed canonical records"));

        snapshot.process_records[1].raw_errno = libc::ESRCH;
        enumeration.process_snapshot_sha256 = sha256_hex_v2(b"substitute snapshot");
        assert!(enumeration
            .validate("/fixed/experiment-binary", &snapshot)
            .unwrap_err()
            .to_string()
            .contains("shared snapshot binding"));

        enumeration.process_snapshot_sha256 = snapshot.process_snapshot_sha256.clone();
        snapshot.process_records[0].path = Some("/fixed/experiment-binary".to_owned());
        let changed_raw = canonical_bytes_v2(&snapshot.process_records).unwrap();
        snapshot.process_snapshot = CandidateFreezeRawBytesV2 {
            base64url: URL_SAFE_NO_PAD.encode(&changed_raw),
            sha256: sha256_hex_v2(&changed_raw),
            byte_length: u64::try_from(changed_raw.len()).unwrap(),
        };
        snapshot.process_snapshot_sha256 = sha256_hex_v2(&changed_raw);
        enumeration.process_snapshot_sha256 = snapshot.process_snapshot_sha256.clone();
        assert!(enumeration
            .validate("/fixed/experiment-binary", &snapshot)
            .unwrap_err()
            .to_string()
            .contains("exact-path process"));

        let raw_reference = canonical_bytes_v2(
            &CandidateFreezeRawAbsenceObservationV2::ProcessExecutablePath {
                identity: "/fixed/experiment-binary".to_owned(),
                enumeration,
            },
        )
        .unwrap();
        let reference_text = std::str::from_utf8(&raw_reference).unwrap();
        assert!(!reference_text.contains("process_records"));
        assert!(!reference_text.contains("process_snapshot\""));
    }

    #[test]
    fn freeze_and_root_snapshots_reject_saturation_and_retain_path_failures() {
        let freeze = include_str!("../../../../scripts/mac/freeze-r3-macos-finalizer-candidate.sh");
        let installer = include_str!("../../../../scripts/mac/r3-macos-finalizer-root-install.py");
        for source in [freeze, installer] {
            assert!(source.contains("actual >= capacity"));
            assert!(source.contains("disappeared_during_snapshot"));
            assert!(source.contains("raw_errno"));
            assert!(source.contains("process_records"));
            assert!(source.contains("process_snapshot"));
            assert!(source.contains("candidate_preinstall_process_snapshot_v2"));
        }
        assert!(!freeze.contains("\"enumeration\": {\"total_process_count\""));
        assert!(!installer.contains("\"enumeration\": {\n                \"total_process_count\""));
        let live_validator = installer
            .split("def validate_live_observation(")
            .nth(1)
            .expect("sealed installer contains live absence validation")
            .split("\ndef ")
            .next()
            .expect("live absence validation has a closed function body");
        assert!(live_validator.contains("set(raw) != {\"kind\", \"identity\", \"enumeration\"}"));
    }

    #[test]
    fn extracted_freeze_fence_binds_only_the_exact_committed_candidate() {
        let freeze = include_str!("../../../../scripts/mac/freeze-r3-macos-finalizer-candidate.sh");
        let installer = include_str!("../../../../scripts/mac/r3-macos-finalizer-root-install.py");
        for source in [freeze, installer] {
            assert!(source.contains(CANDIDATE_FREEZE_REPOSITORY_PATH_V2));
            assert!(source.contains(CANDIDATE_FREEZE_BRANCH_V2));
            assert!(!source.contains("feat/internal-host-orchestrator-world-dispatch-bootstrap"));
        }
        assert_eq!(
            freeze
                .matches("substrate.r3-macos-candidate-committed-tree-inventory")
                .count(),
            1
        );
        assert_eq!(freeze.matches("HEAD^{tree}").count(), 2);
        assert_eq!(
            freeze.matches("status\", \"--porcelain=v2\", \"-z").count(),
            3
        );
        for forbidden in [
            "GIT_INDEX_FILE",
            "private candidate index",
            "r3-macos-finalizer-proof-candidate",
            "feat/r3-macos-finalizer-proof-candidate",
        ] {
            assert!(!freeze.contains(forbidden));
        }
        for forbidden in [
            "'src/bin/substrate-lifecycle-linux.rs'",
            "'src/bin/substrate-lifecycle-macos.rs'",
        ] {
            assert!(!freeze.contains(forbidden));
            assert!(
                !super::super::freeze_provenance::CANDIDATE_FREEZE_SOURCE_PATHS_V2
                    .contains(&forbidden.trim_matches('\''))
            );
        }
    }

    #[test]
    fn launchd_absence_requires_the_frozen_raw_not_found_classification() {
        let label = MAC_R3_FINALIZER_LAUNCHD_LABEL_V2;
        let expected = CandidateFreezeAbsenceProbeV2 {
            kind: CandidateFreezeAbsenceKindV2::LaunchdLabel,
            identity: label.to_owned(),
            classification: CandidateFreezeAbsenceClassificationV2::LaunchdLabelAbsent {
                raw_exit_status: 113,
            },
        };
        let stream = |bytes: &[u8]| CandidateFreezeRawBytesV2 {
            base64url: URL_SAFE_NO_PAD.encode(bytes),
            sha256: sha256_hex_v2(bytes),
            byte_length: u64::try_from(bytes.len()).unwrap(),
        };
        let mut observed = CandidateFreezeRawAbsenceObservationV2::LaunchdLabel {
            identity: label.to_owned(),
            raw_exit_status: 113,
            stdout: stream(b""),
            stderr: stream(
                format!("Bad request.\nCould not find service \"{label}\" in domain for system\n")
                    .as_bytes(),
            ),
        };
        observed.validate(&expected, &mut None).unwrap();
        if let CandidateFreezeRawAbsenceObservationV2::LaunchdLabel { stderr, .. } = &mut observed {
            *stderr = stream(b"arbitrary status-113 output");
        }
        assert!(observed.validate(&expected, &mut None).is_err());

        let installer = include_str!("../../../../scripts/mac/r3-macos-finalizer-root-install.py");
        let exact_service = installer
            .split("def exact_service_absent(")
            .nth(1)
            .expect("sealed installer contains exact-service preflight")
            .split("\ndef ")
            .next()
            .expect("exact-service preflight has a closed function body");
        assert!(exact_service.contains("validate_launchctl_not_found_streams("));
    }
}
