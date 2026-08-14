//! Root-runner-authored packet that is durable before either native experiment group starts.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, sha256_hex_v2, ExecutableIdentityV2,
    MAC_R3_COORDINATOR_INBOX_ROOT_V2, MAC_R3_COORDINATOR_PATH_V2,
    MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2, MAC_R3_FINALIZER_ENDPOINT_V2,
    MAC_R3_FINALIZER_JOURNAL_ROOT_V2, MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_REQUEST_PATH_V2, MAC_R3_RETIREMENT_LATCH_ROOT_V2,
    MAC_R3_TERMINAL_BINDING_PATH_V2,
};

use super::controls::{
    NobodyOwnerAuthorityOperationV2, PeerSubstitutionControlV2, SecurityAgentRawReportEvidenceV2,
    TransportControlV2, ERR_SEC_AUTH_FAILED_V2, ERR_SEC_INTERACTION_NOT_ALLOWED_V2,
    NOBODY_OWNER_AUTHORITY_SEQUENCE_V2, PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2,
    TRANSPORT_CONTROL_SEQUENCE_V2,
};
use super::freeze_manifest::{
    expected_candidate_freeze_absence_plan_v2, CandidateFreezeAbsenceClassificationV2,
    CandidateFreezeAbsenceKindV2, CandidateFreezeArtifactRoleV2, CandidateFreezeInstallDirectoryV2,
    CandidateFreezeManifestV2, CandidateFreezeProcessSnapshotV2,
    CandidateFreezeRawAbsenceObservationV2, CANDIDATE_FREEZE_ARTIFACT_ROOT_V2,
    CANDIDATE_FREEZE_ATOMIZE_SUPPORT_ROOT_V2, CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
    CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2,
    CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2, CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2,
    CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2, CANDIDATE_FREEZE_INSTALLED_SUPPORT_ROOT_V2,
    CANDIDATE_FREEZE_MANIFEST_INPUT_PATH_V2, CANDIDATE_FREEZE_MANIFEST_PATH_V2,
    CANDIDATE_FREEZE_PRODUCT_SUPPORT_ROOT_V2, CANDIDATE_FREEZE_REVIEWED_ADMIN_BLOCK_PATH_V2,
};
use super::freeze_provenance::CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2;
use super::harness_protocol::{
    compiled_preserved_denylist_v2, exhaustive_disjoint_comparison_v2, frozen_inventory_v2,
    CompiledPreservedDenylistRuleV2, DisposableInventoryEntryV2, DisposableInventoryNamespaceV2,
};
use super::publisher_protocol::{CandidateIdentityPacketV2, PublisherPreparedInputV2};
use super::{
    CodeSigningPostureV2, RepetitionV2, ALTERNATE_COORDINATOR_PATH_V2, CREATOR_EXECUTABLE_PATH_V2,
    CREATOR_MARKER_PATH_V2, CREATOR_MARKER_ROOT_V2, CREATOR_SCOPES_V2, DISPOSABLE_HARNESS_PATH_V2,
    DISPOSABLE_HARNESS_UID_V2, EXPERIMENT_ID_V2, EXPERIMENT_ROOT_V2, EXPERIMENT_VERSION_V2,
    GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2, NOBODY_OWNER_PROBE_PATH_V2, PEER_CODE_PROBE_PATH_V2,
    WRONG_IDENTITY_EXECUTABLE_PATH_V2,
};

pub const GLOBAL_PRE_EFFECT_PACKET_OWNER_V2: &str =
    "substrate.r3-macos-disposable-global-pre-effect-packet";
pub const ROOT_INSTALL_CLAIM_ROOT_V2: &str =
    "/private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2";
pub const ROOT_INSTALL_PRECLAIM_PATH_V2: &str =
    "/private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2/root-install-preclaim.v2.json";
pub const ROOT_INSTALL_COMPLETION_PATH_V2: &str =
    "/private/tmp/com.atomize.substrate.r3-macos-finalizer-freeze.v2/root-install-completion.v2.json";
pub const ROOT_INSTALL_PRECLAIM_OWNER_V2: &str =
    "substrate.r3-macos-finalizer-root-install-preclaim";
pub const ROOT_INSTALL_COMPLETION_OWNER_V2: &str =
    "substrate.r3-macos-finalizer-root-install-completion";
pub const MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/journal-root.lock";
pub const TERMINAL_ADMIN_CLEANUP_CLAIM_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/019ffec6-95f6-7d30-80bc-8003ce27d5ba/global-publisher-exchange/terminal-admin-cleanup-claim.v2.json";
pub const TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/019ffec6-95f6-7d30-80bc-8003ce27d5ba/global-publisher-exchange/terminal-admin-restoration-receipt.v2.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RootInstallPhysicalIdentityV2 {
    pub path: String,
    pub device: u64,
    pub inode: u64,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub link_count: u64,
    pub size: u64,
    pub modified_seconds: i64,
    pub modified_nanoseconds: i64,
}

impl RootInstallPhysicalIdentityV2 {
    fn validate_directory(&self, path: &str, uid: u32, gid: u32, mode: u32) -> Result<()> {
        if self.path != path
            || self.uid != uid
            || self.gid != gid
            || self.mode != u32::from(libc::S_IFDIR) | mode
            || self.link_count < 2
            || self.size == 0
        {
            bail!("root-install directory physical identity changed")
        }
        Ok(())
    }

    fn validate_file(&self, path: &str, uid: u32, gid: u32, mode: u32, size: u64) -> Result<()> {
        if self.path != path
            || self.uid != uid
            || self.gid != gid
            || self.mode != u32::from(libc::S_IFREG) | mode
            || self.link_count != 1
            || self.size != size
        {
            bail!("root-install file physical identity changed")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RootInstallLiveAbsenceObservationV2 {
    pub predicate_sha256: String,
    pub raw_observation: CandidateFreezeRawAbsenceObservationV2,
    pub raw_observation_sha256: String,
    pub observation_sha256: String,
}

impl RootInstallLiveAbsenceObservationV2 {
    fn validate(
        &self,
        expected: &super::freeze_manifest::CandidateFreezeAbsenceProbeV2,
        shared_process_snapshot: &mut Option<CandidateFreezeProcessSnapshotV2>,
    ) -> Result<()> {
        let raw = canonical_bytes_v2(&self.raw_observation)?;
        if self.predicate_sha256 != document_sha256_v2(expected)?
            || self.raw_observation_sha256 != sha256_hex_v2(&raw)
            || self.observation_sha256
                != document_sha256_v2(&(
                    &self.predicate_sha256,
                    &self.raw_observation_sha256,
                    expected.classification,
                ))?
        {
            bail!("root-install live absence evidence changed its raw binding")
        }
        self.raw_observation
            .validate(expected, shared_process_snapshot)?;
        match (
            &self.raw_observation,
            expected.kind,
            expected.classification,
        ) {
            (
                CandidateFreezeRawAbsenceObservationV2::FilesystemPath {
                    identity,
                    lstat_return,
                    raw_errno,
                    stat_result,
                },
                CandidateFreezeAbsenceKindV2::FilesystemPath,
                CandidateFreezeAbsenceClassificationV2::PathAbsent,
            )
            | (
                CandidateFreezeRawAbsenceObservationV2::UnixEndpoint {
                    identity,
                    lstat_return,
                    raw_errno,
                    stat_result,
                },
                CandidateFreezeAbsenceKindV2::UnixEndpoint,
                CandidateFreezeAbsenceClassificationV2::EndpointAbsent,
            ) if identity == &expected.identity
                && *lstat_return == -1
                && *raw_errno == libc::ENOENT
                && stat_result.is_none() => {}
            (
                CandidateFreezeRawAbsenceObservationV2::LaunchdLabel {
                    identity,
                    raw_exit_status,
                    ..
                },
                CandidateFreezeAbsenceKindV2::LaunchdLabel,
                CandidateFreezeAbsenceClassificationV2::LaunchdLabelAbsent {
                    raw_exit_status: expected_status,
                },
            ) if identity == &expected.identity && *raw_exit_status == expected_status => {}
            (
                CandidateFreezeRawAbsenceObservationV2::ProcessSnapshot { identity, .. },
                CandidateFreezeAbsenceKindV2::ProcessSnapshot,
                CandidateFreezeAbsenceClassificationV2::ProcessSnapshotCaptured,
            ) if identity == &expected.identity => {}
            (
                CandidateFreezeRawAbsenceObservationV2::ProcessExecutablePath {
                    identity,
                    enumeration,
                },
                CandidateFreezeAbsenceKindV2::ProcessExecutablePath,
                CandidateFreezeAbsenceClassificationV2::ProcessAbsent,
            ) if identity == &expected.identity && enumeration.matching_pids.is_empty() => {}
            _ => bail!("root-install live absence evidence changed its exact classification"),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RootInstallPreclaimV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub sequence: u8,
    pub claim_root_path: String,
    pub preclaim_path: String,
    pub completion_path: String,
    pub private_ancestor_identity: RootInstallPhysicalIdentityV2,
    pub private_tmp_ancestor_identity: RootInstallPhysicalIdentityV2,
    pub claim_root_identity: RootInstallPhysicalIdentityV2,
    pub candidate_freeze_manifest_sha256: String,
    pub reviewed_admin_block_sha256: String,
    pub candidate_artifact_set_sha256: String,
    pub install_parent_directory_set_sha256: String,
    pub preinstall_absence_observation_set_sha256: String,
    pub rollback_plan_sha256: String,
    pub supporting_manifest_set_sha256: String,
    pub live_absence_observations: Vec<RootInstallLiveAbsenceObservationV2>,
    pub live_absence_observation_set_sha256: String,
    pub preclaim_leaf_lstat_return: i32,
    pub preclaim_leaf_raw_errno: i32,
    pub completion_leaf_lstat_return: i32,
    pub completion_leaf_raw_errno: i32,
    pub pre_effects_authorized: bool,
}

impl RootInstallPreclaimV2 {
    pub fn validate(&self, manifest: &CandidateFreezeManifestV2) -> Result<()> {
        manifest.validate()?;
        let expected = root_install_live_absence_plan_v2();
        if self.schema_owner != ROOT_INSTALL_PRECLAIM_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.sequence != 1
            || self.claim_root_path != ROOT_INSTALL_CLAIM_ROOT_V2
            || self.preclaim_path != ROOT_INSTALL_PRECLAIM_PATH_V2
            || self.completion_path != ROOT_INSTALL_COMPLETION_PATH_V2
            || self.candidate_freeze_manifest_sha256 != document_sha256_v2(manifest)?
            || self.reviewed_admin_block_sha256 != manifest.reviewed_admin_block_sha256
            || self.candidate_artifact_set_sha256 != manifest.artifact_set_sha256
            || self.install_parent_directory_set_sha256
                != manifest.install_parent_directory_set_sha256
            || self.preinstall_absence_observation_set_sha256
                != manifest.preinstall_absence_observation_set_sha256
            || self.rollback_plan_sha256 != manifest.rollback_plan_sha256
            || self.supporting_manifest_set_sha256
                != root_install_supporting_manifest_set_sha256_v2(manifest)?
            || self.live_absence_observations.len() != expected.len()
            || self.live_absence_observation_set_sha256
                != document_sha256_v2(&self.live_absence_observations)?
            || self.preclaim_leaf_lstat_return != -1
            || self.preclaim_leaf_raw_errno != libc::ENOENT
            || self.completion_leaf_lstat_return != -1
            || self.completion_leaf_raw_errno != libc::ENOENT
            || self.pre_effects_authorized
        {
            bail!("root-install preclaim changed its exact pre-mutation authority")
        }
        self.private_ancestor_identity
            .validate_directory("/private", 0, 0, 0o755)?;
        self.private_tmp_ancestor_identity
            .validate_directory("/private/tmp", 0, 0, 0o1777)?;
        self.claim_root_identity
            .validate_directory(ROOT_INSTALL_CLAIM_ROOT_V2, 0, 0, 0o700)?;
        let mut shared_process_snapshot = None;
        for (observation, expected) in self.live_absence_observations.iter().zip(&expected) {
            observation.validate(expected, &mut shared_process_snapshot)?;
        }
        if shared_process_snapshot.is_none() {
            bail!("root-install preclaim omitted its shared process snapshot")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RootInstallArtifactIdentityV2 {
    pub role: CandidateFreezeArtifactRoleV2,
    pub path: String,
    pub physical_identity: RootInstallPhysicalIdentityV2,
    pub sha256: String,
    pub executable_identity: Option<ExecutableIdentityV2>,
    pub signing_posture: Option<CodeSigningPostureV2>,
}

impl RootInstallArtifactIdentityV2 {
    fn validate(
        &self,
        expected: &super::freeze_manifest::CandidateFreezeArtifactEntryV2,
        manifest: &CandidateFreezeManifestV2,
    ) -> Result<()> {
        self.physical_identity.validate_file(
            &expected.intended_path,
            expected.uid,
            expected.gid,
            expected.mode,
            expected.size,
        )?;
        if self.role != expected.role
            || self.path != expected.intended_path
            || self.sha256 != expected.sha256
        {
            bail!("root-install artifact identity differs from the candidate manifest")
        }
        match expected.signing_identifier.as_deref() {
            Some(identifier) => {
                let executable = self
                    .executable_identity
                    .as_ref()
                    .context("signed root-install artifact lacks executable identity")?;
                let posture = self
                    .signing_posture
                    .as_ref()
                    .context("signed root-install artifact lacks signing posture")?;
                if executable.intended_path != expected.intended_path
                    || executable.executable_sha256 != expected.sha256
                    || executable.executable_size != expected.size
                    || executable.signing_identifier != identifier
                    || executable.source_commit != manifest.source_commit
                    || executable.source_tree != manifest.source_tree
                    || executable.source_hashes_sha256 != manifest.source_hashes_manifest_sha256
                    || executable.build_inputs_sha256
                        != if matches!(
                            expected.role,
                            CandidateFreezeArtifactRoleV2::CoordinatorExecutable
                                | CandidateFreezeArtifactRoleV2::AlternateCoordinatorExecutable
                        ) {
                            manifest.coordinator_build_digest.clone()
                        } else {
                            manifest.global_build_digest.clone()
                        }
                    || executable.designated_requirement
                        != expected.designated_requirement.clone().unwrap_or_default()
                    || executable.cdhash != expected.cdhash.clone().unwrap_or_default()
                    || executable.physical_identity_sha256
                        != root_install_executable_physical_identity_sha256_v2(
                            &self.physical_identity,
                        )?
                    || posture.executable_identity_sha256 != document_sha256_v2(executable)?
                {
                    bail!("root-install signed artifact changed its exact code identity")
                }
                posture.validate_for(executable)?;
            }
            None if self.executable_identity.is_none() && self.signing_posture.is_none() => {}
            None => bail!("unsigned root-install artifact unexpectedly has code identity"),
        }
        Ok(())
    }
}

pub fn root_install_executable_physical_identity_sha256_v2(
    identity: &RootInstallPhysicalIdentityV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct ExecutableFileIdentity {
        device: u64,
        inode: u64,
        owner_uid: u32,
        owner_gid: u32,
        mode: u32,
        link_count: u64,
        size: u64,
        modified_seconds: i64,
        modified_nanoseconds: i64,
    }
    document_sha256_v2(&ExecutableFileIdentity {
        device: identity.device,
        inode: identity.inode,
        owner_uid: identity.uid,
        owner_gid: identity.gid,
        mode: identity.mode,
        link_count: identity.link_count,
        size: identity.size,
        modified_seconds: identity.modified_seconds,
        modified_nanoseconds: identity.modified_nanoseconds,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RootInstallCompletionV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub sequence: u8,
    pub preclaim_sha256: String,
    pub candidate_freeze_manifest_sha256: String,
    pub reviewed_admin_block_sha256: String,
    pub candidate_artifact_set_sha256: String,
    pub install_parent_directory_set_sha256: String,
    pub preinstall_absence_observation_set_sha256: String,
    pub rollback_plan_sha256: String,
    pub supporting_manifest_set_sha256: String,
    pub installed_directories: Vec<RootInstallPhysicalIdentityV2>,
    pub installed_directory_set_sha256: String,
    pub installed_artifacts: Vec<RootInstallArtifactIdentityV2>,
    pub installed_artifact_identity_set_sha256: String,
    pub preclaim_leaf_identity: RootInstallPhysicalIdentityV2,
    pub completion_leaf_lstat_return: i32,
    pub completion_leaf_raw_errno: i32,
    pub launchd_label_absence: RootInstallLiveAbsenceObservationV2,
    pub endpoint_absence: RootInstallLiveAbsenceObservationV2,
    pub install_complete: bool,
    pub launchd_bootstrap_authorized: bool,
}

impl RootInstallCompletionV2 {
    pub fn validate(
        &self,
        manifest: &CandidateFreezeManifestV2,
        preclaim: &RootInstallPreclaimV2,
    ) -> Result<()> {
        preclaim.validate(manifest)?;
        let expected_absence = expected_candidate_freeze_absence_plan_v2();
        let expected_directories = expected_root_install_directories_v2(manifest);
        let launchd = expected_absence
            .iter()
            .find(|value| value.kind == CandidateFreezeAbsenceKindV2::LaunchdLabel)
            .context("candidate absence plan lacks launchd label")?;
        let endpoint = expected_absence
            .iter()
            .find(|value| value.kind == CandidateFreezeAbsenceKindV2::UnixEndpoint)
            .context("candidate absence plan lacks finalizer endpoint")?;
        if self.schema_owner != ROOT_INSTALL_COMPLETION_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.sequence != 2
            || self.preclaim_sha256 != document_sha256_v2(preclaim)?
            || self.candidate_freeze_manifest_sha256 != document_sha256_v2(manifest)?
            || self.reviewed_admin_block_sha256 != manifest.reviewed_admin_block_sha256
            || self.candidate_artifact_set_sha256 != manifest.artifact_set_sha256
            || self.install_parent_directory_set_sha256
                != manifest.install_parent_directory_set_sha256
            || self.preinstall_absence_observation_set_sha256
                != manifest.preinstall_absence_observation_set_sha256
            || self.rollback_plan_sha256 != manifest.rollback_plan_sha256
            || self.supporting_manifest_set_sha256
                != root_install_supporting_manifest_set_sha256_v2(manifest)?
            || self.installed_directories.len() != expected_directories.len()
            || self.installed_directory_set_sha256
                != document_sha256_v2(&self.installed_directories)?
            || self.installed_artifacts.len() != manifest.artifacts.len()
            || self.installed_artifact_identity_set_sha256
                != document_sha256_v2(&self.installed_artifacts)?
            || self.completion_leaf_lstat_return != -1
            || self.completion_leaf_raw_errno != libc::ENOENT
            || !self.install_complete
            || self.launchd_bootstrap_authorized
        {
            bail!("root-install completion changed its exact pre-run installation")
        }
        self.preclaim_leaf_identity.validate_file(
            ROOT_INSTALL_PRECLAIM_PATH_V2,
            0,
            0,
            0o400,
            u64::try_from(canonical_bytes_v2(preclaim)?.len())?,
        )?;
        for (identity, expected) in self.installed_directories.iter().zip(&expected_directories) {
            identity.validate_directory(
                &expected.path,
                expected.uid,
                expected.gid,
                expected.mode,
            )?;
        }
        for (identity, expected) in self.installed_artifacts.iter().zip(&manifest.artifacts) {
            identity.validate(expected, manifest)?;
        }
        self.launchd_label_absence.validate(launchd, &mut None)?;
        self.endpoint_absence.validate(endpoint, &mut None)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RootInstallClaimsBindingV2 {
    pub preclaim: RootInstallPreclaimV2,
    pub completion: RootInstallCompletionV2,
    pub preclaim_sha256: String,
    pub completion_sha256: String,
    pub claim_root_current_identity: RootInstallPhysicalIdentityV2,
    pub claim_root_pathname_before_sha256: String,
    pub claim_root_descriptor_sha256: String,
    pub claim_root_pathname_after_sha256: String,
    pub completion_leaf_identity: RootInstallPhysicalIdentityV2,
}

impl RootInstallClaimsBindingV2 {
    pub fn validate(&self, manifest: &CandidateFreezeManifestV2) -> Result<()> {
        self.preclaim.validate(manifest)?;
        self.completion.validate(manifest, &self.preclaim)?;
        self.claim_root_current_identity.validate_directory(
            ROOT_INSTALL_CLAIM_ROOT_V2,
            0,
            0,
            0o700,
        )?;
        if self.preclaim_sha256 != document_sha256_v2(&self.preclaim)?
            || self.completion_sha256 != document_sha256_v2(&self.completion)?
            || self.claim_root_pathname_before_sha256
                != document_sha256_v2(&self.claim_root_current_identity)?
            || self.claim_root_descriptor_sha256 != self.claim_root_pathname_before_sha256
            || self.claim_root_pathname_after_sha256 != self.claim_root_pathname_before_sha256
        {
            bail!("root-install claim binding changed its pathname/descriptor identity")
        }
        if self.claim_root_current_identity.device != self.preclaim.claim_root_identity.device
            || self.claim_root_current_identity.inode != self.preclaim.claim_root_identity.inode
            || self.claim_root_current_identity.uid != self.preclaim.claim_root_identity.uid
            || self.claim_root_current_identity.gid != self.preclaim.claim_root_identity.gid
            || self.claim_root_current_identity.mode != self.preclaim.claim_root_identity.mode
        {
            bail!("root-install claim directory was substituted after preclaim")
        }
        self.completion_leaf_identity.validate_file(
            ROOT_INSTALL_COMPLETION_PATH_V2,
            0,
            0,
            0o400,
            u64::try_from(canonical_bytes_v2(&self.completion)?.len())?,
        )
    }
}

pub fn root_install_supporting_manifest_set_sha256_v2(
    manifest: &CandidateFreezeManifestV2,
) -> Result<String> {
    document_sha256_v2(&[
        &manifest.source_hashes_manifest_sha256,
        &manifest.coordinator_build_input_manifest_sha256,
        &manifest.global_build_input_manifest_sha256,
        &manifest.coordinator_provenance_input_sha256,
        &manifest.global_provenance_input_sha256,
        &manifest.manifest_input_sha256,
    ])
}

pub fn root_install_live_absence_plan_v2(
) -> Vec<super::freeze_manifest::CandidateFreezeAbsenceProbeV2> {
    expected_candidate_freeze_absence_plan_v2()
        .into_iter()
        .filter(|value| value.identity != ROOT_INSTALL_CLAIM_ROOT_V2)
        .collect()
}

pub fn expected_root_install_directories_v2(
    manifest: &CandidateFreezeManifestV2,
) -> Vec<CandidateFreezeInstallDirectoryV2> {
    let mut values = manifest.install_parent_directories.clone();
    values.extend([
        CandidateFreezeInstallDirectoryV2 {
            path: MAC_R3_COORDINATOR_INBOX_ROOT_V2.to_owned(),
            uid: 0,
            gid: 20,
            mode: 0o750,
        },
        CandidateFreezeInstallDirectoryV2 {
            path: MAC_R3_FINALIZER_JOURNAL_ROOT_V2.to_owned(),
            uid: 0,
            gid: 0,
            mode: 0o700,
        },
        CandidateFreezeInstallDirectoryV2 {
            path: GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2.to_owned(),
            uid: DISPOSABLE_HARNESS_UID_V2,
            gid: 0,
            mode: 0o700,
        },
    ]);
    values
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeExperimentGroupV2 {
    CreatorRoute,
    ExactFinalizer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RollbackDispositionV2 {
    NamespaceBindingOnly,
    BoundPredicateOnly,
    ExactKeychainDeleteThenAbsence,
    BootoutExactLaunchdThenAbsence,
    RemoveExactEndpointThenAbsence,
    RemoveExactTemporaryPathThenAbsence,
    RetainDurableExternalEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GlobalCreatedObjectV2 {
    pub group: NativeExperimentGroupV2,
    pub repetition: u8,
    pub namespace: DisposableInventoryNamespaceV2,
    pub identity: String,
    pub rollback: RollbackDispositionV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "arm", rename_all = "snake_case", deny_unknown_fields)]
pub enum GlobalNativeArmV2 {
    CreatorQueryUiFailCreateDelete,
    CreatorFreshProcessCreate,
    CreatorWrongIdentityDelete,
    CreatorFirstSecurityCallDisableThenDelete,
    CreatorAlreadyAbsentRetry,
    FinalizerTransport {
        control: TransportControlV2,
    },
    FinalizerPeerSubstitution {
        control: PeerSubstitutionControlV2,
    },
    BenignDynamicLibraryInjection,
    NobodyOwnerAuthority {
        operation: NobodyOwnerAuthorityOperationV2,
    },
    FinalizationEffects,
    TerminalBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum PrecommittedNativeOutcomeV2 {
    OsStatusZero,
    ErrSecItemNotFound { raw_os_status: i32 },
    ErrSecInteractionNotAllowed { raw_os_status: i32 },
    ClosedAuthorizationDenial { allowed_raw_os_status: Vec<i32> },
    CanonicalSafePreAcceptanceStop,
    ConnectionClosedBeforeResponse,
    BenignInjectionIgnored,
    EffectsComplete,
    HostComplete,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityAgentExpectationV2 {
    NoProcessActivationWindowPromptOrCredentialRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GlobalNativeArmPlanEntryV2 {
    pub group: NativeExperimentGroupV2,
    pub repetition: u8,
    pub sequence_ordinal: u8,
    pub arm: GlobalNativeArmV2,
    pub expected_outcome: PrecommittedNativeOutcomeV2,
    pub securityagent_expectation: SecurityAgentExpectationV2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlobalNonceObjectKindV2 {
    KeychainApplicationTag,
    KeychainLabel,
    FilesystemPath,
    LaunchdLabel,
    UnixEndpoint,
    ProcessExecutablePath,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "classification", rename_all = "snake_case", deny_unknown_fields)]
pub enum GlobalExactAbsenceClassificationV2 {
    SecItemNotFound { raw_os_status: i32 },
    PathAbsent,
    LaunchdLabelAbsent,
    EndpointAbsent,
    ProcessAbsent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GlobalNonceAbsenceObservationV2 {
    pub kind: GlobalNonceObjectKindV2,
    pub repetition: u8,
    pub identity: String,
    pub exact_predicate_sha256: String,
    pub classification: GlobalExactAbsenceClassificationV2,
    pub observation_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GlobalNonceAbsenceBaselineV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub observations: Vec<GlobalNonceAbsenceObservationV2>,
    pub observation_set_sha256: String,
    pub supporting_baseline_sha256: String,
    pub securityagent_report: SecurityAgentRawReportEvidenceV2,
}

impl GlobalNonceAbsenceBaselineV2 {
    pub fn validate(&self) -> Result<()> {
        let expected = global_nonce_absence_plan_v2();
        if self.schema_owner != GLOBAL_PRE_EFFECT_PACKET_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.observations.len() != expected.len()
            || self.observation_set_sha256 != document_sha256_v2(&self.observations)?
        {
            bail!("global nonce baseline did not prove every exact frozen absence")
        }
        for (observation, key) in self.observations.iter().zip(&expected) {
            if observation.kind != key.kind
                || observation.repetition != key.repetition
                || observation.identity != key.identity
                || observation.classification != key.classification
                || observation.exact_predicate_sha256 != document_sha256_v2(key)?
            {
                bail!("global nonce baseline changed an exact absence predicate or classification")
            }
            require_digest(&observation.observation_sha256)?;
        }
        self.securityagent_report.validate()?;
        require_digest(&self.supporting_baseline_sha256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreatorNativeArmV2 {
    QueryUiFailCreateThenDelete,
    FreshProcessCreateThenExit,
    WrongIdentityDelete,
    FreshProcessFirstCallDisableThenDelete,
    AlreadyAbsentRetry,
}

pub const CREATOR_NATIVE_ARM_SEQUENCE_V2: [CreatorNativeArmV2; 5] = [
    CreatorNativeArmV2::QueryUiFailCreateThenDelete,
    CreatorNativeArmV2::FreshProcessCreateThenExit,
    CreatorNativeArmV2::WrongIdentityDelete,
    CreatorNativeArmV2::FreshProcessFirstCallDisableThenDelete,
    CreatorNativeArmV2::AlreadyAbsentRetry,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreatorNativeOperationV2 {
    DisableProcessInteractionFirst,
    CreateProductEquivalentSigner,
    DeleteExactSigner,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreatorNativeClassificationV2 {
    InteractionDisabled,
    CreatedAndPresent,
    DeletedAndAbsent,
    InteractionNotAllowedAndPresent,
    AlreadyAbsent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CreatorNativeOperationReceiptV2 {
    pub sequence_ordinal: u8,
    pub operation: CreatorNativeOperationV2,
    pub raw_status: i64,
    pub classification: CreatorNativeClassificationV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CreatorNativeArmReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub creator_scope_id: String,
    pub sequence_ordinal: u8,
    pub arm: CreatorNativeArmV2,
    pub executable_identity_sha256: String,
    pub process_attestation_sha256: String,
    pub marker_before: String,
    pub marker_after: String,
    pub target_present_before: bool,
    pub target_present_after: bool,
    pub operations: Vec<CreatorNativeOperationReceiptV2>,
    pub native_receipt_base64url: String,
    pub native_receipt_sha256: String,
    pub securityagent_report: SecurityAgentRawReportEvidenceV2,
}

impl CreatorNativeArmReceiptV2 {
    pub fn validate(
        &self,
        repetition: u8,
        identities: &super::controls::PeerControlIdentityPacketV2,
    ) -> Result<()> {
        if self.schema_owner != GLOBAL_PRE_EFFECT_PACKET_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || repetition == 0
            || repetition > 2
            || self.repetition != repetition
            || self.creator_scope_id != CREATOR_SCOPES_V2[usize::from(repetition - 1)]
        {
            bail!("creator native-arm receipt identity changed")
        }
        let arm_index = usize::from(self.sequence_ordinal.saturating_sub(1));
        let expected_arm = CREATOR_NATIVE_ARM_SEQUENCE_V2
            .get(arm_index)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("creator arm ordinal is outside the closed sequence"))?;
        let expected_identity = if expected_arm == CreatorNativeArmV2::WrongIdentityDelete {
            &identities.wrong_identity
        } else {
            &identities.creator_identity
        };
        if self.sequence_ordinal == 0
            || self.arm != expected_arm
            || self.executable_identity_sha256 != document_sha256_v2(expected_identity)?
            || self.marker_before != creator_marker_before_v2(repetition, expected_arm)
            || self.marker_after != creator_marker_after_v2(repetition, expected_arm)
            || (self.target_present_before, self.target_present_after)
                != creator_presence_v2(expected_arm)
            || self.operations != creator_operations_v2(expected_arm)
        {
            bail!(
                "creator native-arm receipt changed its exact route, status, or marker transition"
            )
        }
        let native = URL_SAFE_NO_PAD
            .decode(&self.native_receipt_base64url)
            .map_err(|error| anyhow::anyhow!("decode creator native receipt: {error}"))?;
        if native.is_empty()
            || native.len() > 1024 * 1024
            || self.native_receipt_sha256 != sha256_hex_v2(&native)
        {
            bail!("creator native-arm receipt did not retain its bounded raw native receipt")
        }
        self.securityagent_report.validate()?;
        require_digest(&self.process_attestation_sha256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CreatorRouteReceiptSetV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub global_pre_effect_packet_sha256: String,
    pub runner_process_attestation_sha256: String,
    pub receipts: Vec<CreatorNativeArmReceiptV2>,
    pub receipt_set_sha256: String,
}

impl CreatorRouteReceiptSetV2 {
    pub fn validate(
        &self,
        packet: &GlobalPreEffectPacketV2,
        identities: &super::controls::PeerControlIdentityPacketV2,
    ) -> Result<()> {
        if self.schema_owner != GLOBAL_PRE_EFFECT_PACKET_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.global_pre_effect_packet_sha256 != document_sha256_v2(packet)?
            || self.runner_process_attestation_sha256 != packet.runner_process_attestation_sha256
            || self.receipts.len() != CREATOR_NATIVE_ARM_SEQUENCE_V2.len() * 2
            || self.receipt_set_sha256 != document_sha256_v2(&self.receipts)?
        {
            bail!("creator receipt set does not bind the frozen pre-effect packet and ten arms")
        }
        for (index, receipt) in self.receipts.iter().enumerate() {
            let repetition = u8::try_from(index / CREATOR_NATIVE_ARM_SEQUENCE_V2.len() + 1)
                .expect("two repetitions fit u8");
            let ordinal = u8::try_from(index % CREATOR_NATIVE_ARM_SEQUENCE_V2.len() + 1)
                .expect("five creator arms fit u8");
            if receipt.sequence_ordinal != ordinal {
                bail!("creator receipt set reordered one native arm")
            }
            receipt.validate(repetition, identities)?;
        }
        require_digest(&self.runner_process_attestation_sha256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GlobalPreEffectDynamicBindingsV2 {
    pub candidate_freeze_manifest: CandidateFreezeManifestV2,
    pub installed_candidate_manifest: InstalledCandidateManifestBindingV2,
    pub root_install_claims: RootInstallClaimsBindingV2,
    pub runner_process_attestation_sha256: String,
    pub activation_membrane_lock_identity_sha256: String,
    pub activation_membrane_root_identity_sha256: String,
    pub journal_root_lock_identity: RootInstallPhysicalIdentityV2,
    pub nonce_absence_baseline: GlobalNonceAbsenceBaselineV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstalledCandidateManifestBindingV2 {
    pub external_manifest_path: String,
    pub installed_manifest_path: String,
    pub external_manifest_sha256: String,
    pub installed_manifest_sha256: String,
    pub reviewed_admin_block_sha256: String,
    pub ancestor_chain_before_sha256: String,
    pub ancestor_chain_after_sha256: String,
    pub pathname_before_identity_sha256: String,
    pub descriptor_identity_sha256: String,
    pub pathname_after_identity_sha256: String,
    pub device: u64,
    pub inode: u64,
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub mode: u32,
    pub link_count: u64,
    pub size: u64,
}

impl InstalledCandidateManifestBindingV2 {
    pub fn validate(&self, manifest: &CandidateFreezeManifestV2) -> Result<()> {
        let manifest_sha256 = document_sha256_v2(manifest)?;
        let expected_size = u64::try_from(
            substrate_common::macos_retirement_v2::canonical_bytes_v2(manifest)?.len(),
        )
        .context("candidate manifest size exceeds u64")?;
        #[derive(Serialize)]
        struct PhysicalIdentity<'a> {
            path: &'a str,
            sha256: &'a str,
            device: u64,
            inode: u64,
            owner_uid: u32,
            owner_gid: u32,
            mode: u32,
            link_count: u64,
            size: u64,
        }
        let physical = document_sha256_v2(&PhysicalIdentity {
            path: &self.installed_manifest_path,
            sha256: &self.installed_manifest_sha256,
            device: self.device,
            inode: self.inode,
            owner_uid: self.owner_uid,
            owner_gid: self.owner_gid,
            mode: self.mode,
            link_count: self.link_count,
            size: self.size,
        })?;
        if self.external_manifest_path != CANDIDATE_FREEZE_MANIFEST_PATH_V2
            || self.installed_manifest_path != CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2
            || manifest.installed_manifest_path != self.installed_manifest_path
            || self.external_manifest_sha256 != manifest_sha256
            || self.installed_manifest_sha256 != manifest_sha256
            || self.reviewed_admin_block_sha256 != manifest.reviewed_admin_block_sha256
            || self.ancestor_chain_before_sha256 != self.ancestor_chain_after_sha256
            || self.pathname_before_identity_sha256 != physical
            || self.descriptor_identity_sha256 != physical
            || self.pathname_after_identity_sha256 != physical
            || self.owner_uid != 0
            || self.owner_gid != 0
            || self.mode != 0o400
            || self.link_count != 1
            || self.size != expected_size
        {
            bail!("installed candidate manifest is not the exact root-owned admin-installed external manifest copy")
        }
        require_digest(&self.ancestor_chain_before_sha256)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GlobalPreEffectPacketV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub prepared_input_sha256: String,
    pub candidate_identity_packet_sha256: String,
    pub peer_control_identity_packet_sha256: String,
    pub candidate_freeze_manifest: CandidateFreezeManifestV2,
    pub candidate_freeze_manifest_sha256: String,
    pub installed_candidate_manifest: InstalledCandidateManifestBindingV2,
    pub root_install_claims: RootInstallClaimsBindingV2,
    pub root_install_preclaim_sha256: String,
    pub root_install_completion_sha256: String,
    pub root_install_claims_binding_sha256: String,
    pub candidate_artifact_root: String,
    pub candidate_artifact_set_sha256: String,
    pub reviewed_admin_block_sha256: String,
    pub runner_identity_sha256: String,
    pub runner_process_attestation_sha256: String,
    pub activation_membrane_lock_identity_sha256: String,
    pub activation_membrane_root_identity_sha256: String,
    pub journal_root_lock_identity: RootInstallPhysicalIdentityV2,
    pub journal_root_lock_identity_sha256: String,
    pub securityagent_baseline_sha256: String,
    pub nonce_absence_baseline: GlobalNonceAbsenceBaselineV2,
    pub disposable_baseline_sha256: String,
    pub created_object_inventory: Vec<GlobalCreatedObjectV2>,
    pub native_arm_plan: Vec<GlobalNativeArmPlanEntryV2>,
    pub compiled_preserved_denylist: Vec<CompiledPreservedDenylistRuleV2>,
    pub created_object_inventory_sha256: String,
    pub native_arm_plan_sha256: String,
    pub compiled_preserved_denylist_sha256: String,
    pub exhaustive_comparison_count: u64,
    pub exhaustive_disjoint_comparison_sha256: String,
}

impl GlobalPreEffectPacketV2 {
    pub fn validate(
        &self,
        prepared: &PublisherPreparedInputV2,
        candidate: &CandidateIdentityPacketV2,
        peer: &super::controls::PeerControlIdentityPacketV2,
    ) -> Result<()> {
        prepared.validate()?;
        candidate.validate()?;
        peer.validate(candidate)?;
        self.candidate_freeze_manifest.validate()?;
        self.installed_candidate_manifest
            .validate(&self.candidate_freeze_manifest)?;
        self.root_install_claims
            .validate(&self.candidate_freeze_manifest)?;
        validate_candidate_freeze_runtime_bindings_v2(
            &self.candidate_freeze_manifest,
            prepared,
            candidate,
            peer,
        )?;
        if self.schema_owner != GLOBAL_PRE_EFFECT_PACKET_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.prepared_input_sha256 != document_sha256_v2(prepared)?
            || self.candidate_identity_packet_sha256 != document_sha256_v2(candidate)?
            || self.peer_control_identity_packet_sha256 != document_sha256_v2(peer)?
            || self.candidate_freeze_manifest_sha256
                != document_sha256_v2(&self.candidate_freeze_manifest)?
            || self.candidate_artifact_root != CANDIDATE_FREEZE_ARTIFACT_ROOT_V2
            || self.candidate_artifact_set_sha256
                != self.candidate_freeze_manifest.artifact_set_sha256
            || self.reviewed_admin_block_sha256
                != self.candidate_freeze_manifest.reviewed_admin_block_sha256
            || self.root_install_preclaim_sha256
                != document_sha256_v2(&self.root_install_claims.preclaim)?
            || self.root_install_completion_sha256
                != document_sha256_v2(&self.root_install_claims.completion)?
            || self.root_install_claims_binding_sha256
                != document_sha256_v2(&self.root_install_claims)?
            || self.runner_identity_sha256 != document_sha256_v2(&peer.runner_identity)?
            || self.securityagent_baseline_sha256 != prepared.securityagent_baseline_sha256
        {
            bail!("global pre-effect packet changed its frozen code or baseline identities")
        }
        for value in [
            &self.runner_process_attestation_sha256,
            &self.activation_membrane_lock_identity_sha256,
            &self.activation_membrane_root_identity_sha256,
            &self.journal_root_lock_identity_sha256,
        ] {
            require_digest(value)?;
        }
        self.journal_root_lock_identity.validate_file(
            MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2,
            0,
            0,
            0o600,
            0,
        )?;
        if self.journal_root_lock_identity_sha256
            != document_sha256_v2(&self.journal_root_lock_identity)?
        {
            bail!("global pre-effect packet changed the exact journal-root lock identity")
        }
        self.nonce_absence_baseline.validate()?;
        if self.disposable_baseline_sha256 != document_sha256_v2(&self.nonce_absence_baseline)? {
            bail!("global pre-effect packet did not bind the full typed nonce-absence baseline")
        }
        let expected_inventory = expected_created_object_inventory_v2();
        let expected_arms = expected_native_arm_plan_v2();
        let expected_denylist = compiled_preserved_denylist_v2();
        if self.created_object_inventory != expected_inventory
            || self.native_arm_plan != expected_arms
            || self.compiled_preserved_denylist != expected_denylist
            || self.created_object_inventory_sha256 != document_sha256_v2(&expected_inventory)?
            || self.native_arm_plan_sha256 != document_sha256_v2(&expected_arms)?
            || self.compiled_preserved_denylist_sha256 != document_sha256_v2(&expected_denylist)?
        {
            bail!("global pre-effect packet changed its exact inventory, rollback, or arm plan")
        }
        let comparison_inventory = expected_inventory
            .iter()
            .map(|entry| DisposableInventoryEntryV2 {
                namespace: entry.namespace,
                identity: entry.identity.clone(),
            })
            .collect::<Vec<_>>();
        let (count, digest) =
            exhaustive_disjoint_comparison_v2(&comparison_inventory, &expected_denylist)?;
        if self.exhaustive_comparison_count != count
            || self.exhaustive_disjoint_comparison_sha256 != digest
        {
            bail!("global pre-effect packet does not prove every preserved-denylist comparison")
        }
        Ok(())
    }
}

fn validate_candidate_freeze_runtime_bindings_v2(
    manifest: &CandidateFreezeManifestV2,
    prepared: &PublisherPreparedInputV2,
    candidate: &CandidateIdentityPacketV2,
    peer: &super::controls::PeerControlIdentityPacketV2,
) -> Result<()> {
    use super::freeze_manifest::CandidateFreezeArtifactRoleV2 as R;
    let bindings = [
        (R::FinalizerExecutable, &candidate.finalizer_identity),
        (R::CoordinatorExecutable, &candidate.coordinator_identity),
        (R::CoordinatorExecutable, &peer.alternate_caller_identity),
        (R::DisposableHarnessExecutable, &peer.harness_identity),
        (R::PeerCodeProbeExecutable, &peer.alternate_code_identity),
        (
            R::AlternateCoordinatorExecutable,
            &peer.alternate_path_identity,
        ),
        (R::CreatorExecutable, &peer.creator_identity),
        (R::WrongIdentityExecutable, &peer.wrong_identity),
        (
            R::DisposablePublisherExecutable,
            &prepared.publisher_identity,
        ),
        (
            R::DisposableExperimentRunnerExecutable,
            &peer.runner_identity,
        ),
        (
            R::NobodyOwnerProbeExecutable,
            &peer.nobody_owner_probe_identity,
        ),
        (
            R::SecurityAgentObserverExecutable,
            &peer.securityagent_observer_identity,
        ),
        (
            R::BenignInjectionLibrary,
            &peer.benign_injection_library_identity,
        ),
    ];
    for (role, identity) in bindings {
        let entry = manifest
            .artifacts
            .iter()
            .find(|entry| entry.role == role)
            .context("candidate freeze manifest lacks a runtime artifact role")?;
        let expected_build_digest = if matches!(
            role,
            R::CoordinatorExecutable | R::AlternateCoordinatorExecutable
        ) {
            &manifest.coordinator_build_digest
        } else {
            &manifest.global_build_digest
        };
        if entry.sha256 != identity.executable_sha256
            || entry.size != identity.executable_size
            || entry.intended_path != identity.intended_path
            || entry.signing_identifier.as_deref() != Some(&identity.signing_identifier)
            || entry.designated_requirement.as_deref() != Some(&identity.designated_requirement)
            || entry.cdhash.as_deref() != Some(&identity.cdhash)
            || identity.source_commit != manifest.source_commit
            || identity.source_tree != manifest.source_tree
            || identity.source_hashes_sha256 != manifest.source_hashes_manifest_sha256
            || &identity.build_inputs_sha256 != expected_build_digest
        {
            bail!("installed runtime identity does not equal its frozen preinstall artifact")
        }
    }
    if candidate.capability_digest != manifest.capability_manifest_sha256
        || candidate.launch_identity.launchd_plist_sha256 != manifest.launchd_plist_sha256
    {
        bail!("runtime capability or launchd plist differs from the frozen candidate artifacts")
    }
    Ok(())
}

pub fn build_global_pre_effect_packet_v2(
    prepared: &PublisherPreparedInputV2,
    candidate: &CandidateIdentityPacketV2,
    peer: &super::controls::PeerControlIdentityPacketV2,
    bindings: GlobalPreEffectDynamicBindingsV2,
) -> Result<GlobalPreEffectPacketV2> {
    let created_object_inventory = expected_created_object_inventory_v2();
    let native_arm_plan = expected_native_arm_plan_v2();
    let compiled_preserved_denylist = compiled_preserved_denylist_v2();
    let comparison_inventory = created_object_inventory
        .iter()
        .map(|entry| DisposableInventoryEntryV2 {
            namespace: entry.namespace,
            identity: entry.identity.clone(),
        })
        .collect::<Vec<_>>();
    let (exhaustive_comparison_count, exhaustive_disjoint_comparison_sha256) =
        exhaustive_disjoint_comparison_v2(&comparison_inventory, &compiled_preserved_denylist)?;
    bindings
        .root_install_claims
        .validate(&bindings.candidate_freeze_manifest)?;
    let root_install_claims_binding_sha256 = document_sha256_v2(&bindings.root_install_claims)?;
    let packet = GlobalPreEffectPacketV2 {
        schema_owner: GLOBAL_PRE_EFFECT_PACKET_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_string(),
        prepared_input_sha256: document_sha256_v2(prepared)?,
        candidate_identity_packet_sha256: document_sha256_v2(candidate)?,
        peer_control_identity_packet_sha256: document_sha256_v2(peer)?,
        candidate_freeze_manifest_sha256: document_sha256_v2(&bindings.candidate_freeze_manifest)?,
        installed_candidate_manifest: bindings.installed_candidate_manifest,
        root_install_preclaim_sha256: bindings.root_install_claims.preclaim_sha256.clone(),
        root_install_completion_sha256: bindings.root_install_claims.completion_sha256.clone(),
        root_install_claims_binding_sha256,
        root_install_claims: bindings.root_install_claims,
        candidate_artifact_root: CANDIDATE_FREEZE_ARTIFACT_ROOT_V2.to_string(),
        candidate_artifact_set_sha256: bindings
            .candidate_freeze_manifest
            .artifact_set_sha256
            .clone(),
        reviewed_admin_block_sha256: bindings
            .candidate_freeze_manifest
            .reviewed_admin_block_sha256
            .clone(),
        candidate_freeze_manifest: bindings.candidate_freeze_manifest,
        runner_identity_sha256: document_sha256_v2(&peer.runner_identity)?,
        runner_process_attestation_sha256: bindings.runner_process_attestation_sha256,
        activation_membrane_lock_identity_sha256: bindings.activation_membrane_lock_identity_sha256,
        activation_membrane_root_identity_sha256: bindings.activation_membrane_root_identity_sha256,
        journal_root_lock_identity_sha256: document_sha256_v2(
            &bindings.journal_root_lock_identity,
        )?,
        journal_root_lock_identity: bindings.journal_root_lock_identity,
        securityagent_baseline_sha256: prepared.securityagent_baseline_sha256.clone(),
        disposable_baseline_sha256: document_sha256_v2(&bindings.nonce_absence_baseline)?,
        nonce_absence_baseline: bindings.nonce_absence_baseline,
        created_object_inventory_sha256: document_sha256_v2(&created_object_inventory)?,
        native_arm_plan_sha256: document_sha256_v2(&native_arm_plan)?,
        compiled_preserved_denylist_sha256: document_sha256_v2(&compiled_preserved_denylist)?,
        created_object_inventory,
        native_arm_plan,
        compiled_preserved_denylist,
        exhaustive_comparison_count,
        exhaustive_disjoint_comparison_sha256,
    };
    packet.validate(prepared, candidate, peer)?;
    Ok(packet)
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalNonceAbsenceProbeV2 {
    pub kind: GlobalNonceObjectKindV2,
    pub repetition: u8,
    pub identity: String,
    pub classification: GlobalExactAbsenceClassificationV2,
}

pub fn global_nonce_absence_plan_v2() -> Vec<GlobalNonceAbsenceProbeV2> {
    use GlobalExactAbsenceClassificationV2 as C;
    use GlobalNonceObjectKindV2 as K;
    let mut values = Vec::new();
    let mut push = |kind, repetition, identity: String, classification| {
        values.push(GlobalNonceAbsenceProbeV2 {
            kind,
            repetition,
            identity,
            classification,
        });
    };
    for (index, scope) in CREATOR_SCOPES_V2.into_iter().enumerate() {
        let repetition = u8::try_from(index + 1).expect("two repetitions fit u8");
        push(
            K::KeychainApplicationTag,
            repetition,
            format!("{scope}:product-equivalent-p256"),
            C::SecItemNotFound {
                raw_os_status: -25_300,
            },
        );
        push(
            K::KeychainLabel,
            repetition,
            format!("{scope}.product-equivalent-p256"),
            C::SecItemNotFound {
                raw_os_status: -25_300,
            },
        );
    }
    for repetition in RepetitionV2::ALL {
        let scope = repetition.scope_id();
        for (kind, identity) in [
            (K::KeychainApplicationTag, format!("{scope}:signing-key")),
            (K::KeychainLabel, format!("{scope}:signing-key")),
            (
                K::KeychainApplicationTag,
                format!("{scope}:wrong-surrogate-signing-key"),
            ),
            (
                K::KeychainLabel,
                format!("{scope}:wrong-surrogate-signing-key"),
            ),
        ] {
            push(
                kind,
                repetition.ordinal(),
                identity,
                C::SecItemNotFound {
                    raw_os_status: -25_300,
                },
            );
        }
        let capability = format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/capability/{scope}");
        for path in [
            format!(
                "{EXPERIMENT_ROOT_V2}/repetitions/{}",
                repetition.directory_name()
            ),
            format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/{scope}"),
            capability.clone(),
            format!("{capability}/surrogate-wrapper.v2"),
            format!("{capability}/surrogate.lock"),
            format!("{MAC_R3_RETIREMENT_LATCH_ROOT_V2}/{scope}.retirement-terminal.v2.latch"),
        ] {
            push(K::FilesystemPath, repetition.ordinal(), path, C::PathAbsent);
        }
    }
    for path in [
        MAC_R3_FINALIZER_REQUEST_PATH_V2,
        MAC_R3_TERMINAL_BINDING_PATH_V2,
    ] {
        push(K::FilesystemPath, 0, path.to_string(), C::PathAbsent);
    }
    push(
        K::LaunchdLabel,
        0,
        MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_string(),
        C::LaunchdLabelAbsent,
    );
    push(
        K::UnixEndpoint,
        0,
        MAC_R3_FINALIZER_ENDPOINT_V2.to_string(),
        C::EndpointAbsent,
    );
    for path in [
        CREATOR_EXECUTABLE_PATH_V2,
        WRONG_IDENTITY_EXECUTABLE_PATH_V2,
        MAC_R3_FINALIZER_PATH_V2,
        MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2,
        DISPOSABLE_HARNESS_PATH_V2,
        PEER_CODE_PROBE_PATH_V2,
        ALTERNATE_COORDINATOR_PATH_V2,
        NOBODY_OWNER_PROBE_PATH_V2,
    ] {
        push(
            K::ProcessExecutablePath,
            0,
            path.to_string(),
            C::ProcessAbsent,
        );
    }
    values
}

fn creator_marker_before_v2(repetition: u8, arm: CreatorNativeArmV2) -> String {
    let prefix = if repetition == 1 { "first" } else { "second" };
    let state = match arm {
        CreatorNativeArmV2::QueryUiFailCreateThenDelete => "query-prepared",
        CreatorNativeArmV2::FreshProcessCreateThenExit => "fresh-create-prepared",
        CreatorNativeArmV2::WrongIdentityDelete => "wrong-prepared",
        CreatorNativeArmV2::FreshProcessFirstCallDisableThenDelete => "fresh-delete-prepared",
        CreatorNativeArmV2::AlreadyAbsentRetry => "absent-retry-prepared",
    };
    format!("creator-route-v2:{prefix}-{state}")
}

fn creator_marker_after_v2(repetition: u8, arm: CreatorNativeArmV2) -> String {
    let prefix = if repetition == 1 { "first" } else { "second" };
    match arm {
        CreatorNativeArmV2::QueryUiFailCreateThenDelete => {
            format!("creator-route-v2:{prefix}-fresh-create-prepared")
        }
        CreatorNativeArmV2::FreshProcessCreateThenExit => {
            format!("creator-route-v2:{prefix}-wrong-prepared")
        }
        CreatorNativeArmV2::WrongIdentityDelete => {
            format!("creator-route-v2:{prefix}-fresh-delete-prepared")
        }
        CreatorNativeArmV2::FreshProcessFirstCallDisableThenDelete => {
            format!("creator-route-v2:{prefix}-absent-retry-prepared")
        }
        CreatorNativeArmV2::AlreadyAbsentRetry if repetition == 1 => {
            "creator-route-v2:second-query-prepared".to_string()
        }
        CreatorNativeArmV2::AlreadyAbsentRetry => "creator-route-v2:complete".to_string(),
    }
}

const fn creator_presence_v2(arm: CreatorNativeArmV2) -> (bool, bool) {
    match arm {
        CreatorNativeArmV2::QueryUiFailCreateThenDelete => (false, false),
        CreatorNativeArmV2::FreshProcessCreateThenExit => (false, true),
        CreatorNativeArmV2::WrongIdentityDelete => (true, true),
        CreatorNativeArmV2::FreshProcessFirstCallDisableThenDelete => (true, false),
        CreatorNativeArmV2::AlreadyAbsentRetry => (false, false),
    }
}

fn creator_operations_v2(arm: CreatorNativeArmV2) -> Vec<CreatorNativeOperationReceiptV2> {
    use CreatorNativeClassificationV2 as C;
    use CreatorNativeOperationV2 as O;
    let values = match arm {
        CreatorNativeArmV2::QueryUiFailCreateThenDelete => vec![
            (O::CreateProductEquivalentSigner, 0, C::CreatedAndPresent),
            (O::DeleteExactSigner, 0, C::DeletedAndAbsent),
        ],
        CreatorNativeArmV2::FreshProcessCreateThenExit => {
            vec![(O::CreateProductEquivalentSigner, 0, C::CreatedAndPresent)]
        }
        CreatorNativeArmV2::WrongIdentityDelete => vec![(
            O::DeleteExactSigner,
            i64::from(ERR_SEC_INTERACTION_NOT_ALLOWED_V2),
            C::InteractionNotAllowedAndPresent,
        )],
        CreatorNativeArmV2::FreshProcessFirstCallDisableThenDelete => vec![
            (O::DisableProcessInteractionFirst, 0, C::InteractionDisabled),
            (O::DeleteExactSigner, 0, C::DeletedAndAbsent),
        ],
        CreatorNativeArmV2::AlreadyAbsentRetry => vec![
            (O::DisableProcessInteractionFirst, 0, C::InteractionDisabled),
            (O::DeleteExactSigner, -25_300, C::AlreadyAbsent),
        ],
    };
    values
        .into_iter()
        .enumerate()
        .map(
            |(index, (operation, raw_status, classification))| CreatorNativeOperationReceiptV2 {
                sequence_ordinal: u8::try_from(index + 1).expect("creator operations fit u8"),
                operation,
                raw_status,
                classification,
            },
        )
        .collect()
}

fn expected_created_object_inventory_v2() -> Vec<GlobalCreatedObjectV2> {
    use DisposableInventoryNamespaceV2 as N;
    use RollbackDispositionV2 as R;
    let mut objects = Vec::new();
    for identity in [
        CANDIDATE_FREEZE_ARTIFACT_ROOT_V2,
        CANDIDATE_FREEZE_MANIFEST_INPUT_PATH_V2,
        CANDIDATE_FREEZE_COORDINATOR_PROVENANCE_INPUT_PATH_V2,
        CANDIDATE_FREEZE_GLOBAL_PROVENANCE_INPUT_PATH_V2,
        CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2,
        CANDIDATE_FREEZE_COORDINATOR_BUILD_INPUTS_PATH_V2,
        CANDIDATE_FREEZE_GLOBAL_BUILD_INPUTS_PATH_V2,
        CANDIDATE_FREEZE_MANIFEST_PATH_V2,
        CANDIDATE_FREEZE_REVIEWED_ADMIN_BLOCK_PATH_V2,
        GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2,
        TERMINAL_ADMIN_CLEANUP_CLAIM_PATH_V2,
        TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH_V2,
    ] {
        objects.push(GlobalCreatedObjectV2 {
            group: NativeExperimentGroupV2::ExactFinalizer,
            repetition: 0,
            namespace: N::FilesystemPath,
            identity: identity.to_string(),
            rollback: R::RetainDurableExternalEvidence,
        });
    }
    for identity in [
        CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2,
        CANDIDATE_FREEZE_INSTALLED_SUPPORT_ROOT_V2,
        CANDIDATE_FREEZE_PRODUCT_SUPPORT_ROOT_V2,
        CANDIDATE_FREEZE_ATOMIZE_SUPPORT_ROOT_V2,
        MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2,
    ] {
        objects.push(GlobalCreatedObjectV2 {
            group: NativeExperimentGroupV2::ExactFinalizer,
            repetition: 0,
            namespace: N::FilesystemPath,
            identity: identity.to_string(),
            rollback: R::RemoveExactTemporaryPathThenAbsence,
        });
    }
    for (index, scope) in CREATOR_SCOPES_V2.into_iter().enumerate() {
        let repetition = u8::try_from(index + 1).expect("two creator repetitions fit u8");
        let tag = format!("{scope}:product-equivalent-p256");
        let label = format!("{scope}.product-equivalent-p256");
        for (namespace, identity, rollback) in [
            (N::Scope, scope.to_string(), R::NamespaceBindingOnly),
            (
                N::KeychainApplicationTag,
                tag,
                R::ExactKeychainDeleteThenAbsence,
            ),
            (N::KeychainLabel, label, R::BoundPredicateOnly),
            (
                N::ExecutablePath,
                CREATOR_EXECUTABLE_PATH_V2.to_string(),
                R::RemoveExactTemporaryPathThenAbsence,
            ),
            (
                N::ExecutablePath,
                WRONG_IDENTITY_EXECUTABLE_PATH_V2.to_string(),
                R::RemoveExactTemporaryPathThenAbsence,
            ),
            (
                N::FilesystemPath,
                CREATOR_MARKER_ROOT_V2.to_string(),
                R::RemoveExactTemporaryPathThenAbsence,
            ),
            (
                N::FilesystemPath,
                CREATOR_MARKER_PATH_V2.to_string(),
                R::RemoveExactTemporaryPathThenAbsence,
            ),
        ] {
            objects.push(GlobalCreatedObjectV2 {
                group: NativeExperimentGroupV2::CreatorRoute,
                repetition,
                namespace,
                identity,
                rollback,
            });
        }
    }
    for repetition in RepetitionV2::ALL {
        for entry in frozen_inventory_v2(repetition) {
            let rollback = rollback_for_finalizer_entry_v2(&entry);
            objects.push(GlobalCreatedObjectV2 {
                group: NativeExperimentGroupV2::ExactFinalizer,
                repetition: repetition.ordinal(),
                namespace: entry.namespace,
                identity: entry.identity,
                rollback,
            });
        }
    }
    objects
}

fn rollback_for_finalizer_entry_v2(entry: &DisposableInventoryEntryV2) -> RollbackDispositionV2 {
    use DisposableInventoryNamespaceV2 as N;
    use RollbackDispositionV2 as R;
    match entry.namespace {
        N::Scope => R::NamespaceBindingOnly,
        N::KeychainApplicationTag => R::ExactKeychainDeleteThenAbsence,
        N::KeychainLabel => R::BoundPredicateOnly,
        N::LaunchdLabel => R::BootoutExactLaunchdThenAbsence,
        N::UnixEndpoint => R::RemoveExactEndpointThenAbsence,
        N::ExecutablePath => R::RemoveExactTemporaryPathThenAbsence,
        N::FilesystemPath if entry.identity.starts_with(EXPERIMENT_ROOT_V2) => {
            R::RetainDurableExternalEvidence
        }
        N::FilesystemPath => R::RemoveExactTemporaryPathThenAbsence,
        N::KeychainService | N::KeychainAccount | N::LimaInstance | N::EvidenceMirrorIdentity => {
            R::BoundPredicateOnly
        }
    }
}

fn expected_native_arm_plan_v2() -> Vec<GlobalNativeArmPlanEntryV2> {
    use GlobalNativeArmV2 as A;
    use PrecommittedNativeOutcomeV2 as O;
    let ui = SecurityAgentExpectationV2::NoProcessActivationWindowPromptOrCredentialRequest;
    let mut plan = Vec::new();
    for repetition in 1_u8..=2 {
        let creator = [
            (A::CreatorQueryUiFailCreateDelete, O::OsStatusZero),
            (A::CreatorFreshProcessCreate, O::OsStatusZero),
            (
                A::CreatorWrongIdentityDelete,
                O::ErrSecInteractionNotAllowed {
                    raw_os_status: ERR_SEC_INTERACTION_NOT_ALLOWED_V2,
                },
            ),
            (
                A::CreatorFirstSecurityCallDisableThenDelete,
                O::OsStatusZero,
            ),
            (
                A::CreatorAlreadyAbsentRetry,
                O::ErrSecItemNotFound {
                    raw_os_status: -25_300,
                },
            ),
        ];
        for (index, (arm, expected_outcome)) in creator.into_iter().enumerate() {
            plan.push(GlobalNativeArmPlanEntryV2 {
                group: NativeExperimentGroupV2::CreatorRoute,
                repetition,
                sequence_ordinal: u8::try_from(index + 1).expect("creator arm count fits u8"),
                arm,
                expected_outcome,
                securityagent_expectation: ui,
            });
        }
    }
    for repetition in RepetitionV2::ALL {
        let mut ordinal = 0_u8;
        for control in TRANSPORT_CONTROL_SEQUENCE_V2 {
            ordinal += 1;
            plan.push(GlobalNativeArmPlanEntryV2 {
                group: NativeExperimentGroupV2::ExactFinalizer,
                repetition: repetition.ordinal(),
                sequence_ordinal: ordinal,
                arm: A::FinalizerTransport { control },
                expected_outcome: O::CanonicalSafePreAcceptanceStop,
                securityagent_expectation: ui,
            });
        }
        for control in PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2 {
            ordinal += 1;
            plan.push(GlobalNativeArmPlanEntryV2 {
                group: NativeExperimentGroupV2::ExactFinalizer,
                repetition: repetition.ordinal(),
                sequence_ordinal: ordinal,
                arm: A::FinalizerPeerSubstitution { control },
                expected_outcome: O::ConnectionClosedBeforeResponse,
                securityagent_expectation: ui,
            });
        }
        ordinal += 1;
        plan.push(GlobalNativeArmPlanEntryV2 {
            group: NativeExperimentGroupV2::ExactFinalizer,
            repetition: repetition.ordinal(),
            sequence_ordinal: ordinal,
            arm: A::BenignDynamicLibraryInjection,
            expected_outcome: O::BenignInjectionIgnored,
            securityagent_expectation: ui,
        });
        for operation in NOBODY_OWNER_AUTHORITY_SEQUENCE_V2 {
            ordinal += 1;
            plan.push(GlobalNativeArmPlanEntryV2 {
                group: NativeExperimentGroupV2::ExactFinalizer,
                repetition: repetition.ordinal(),
                sequence_ordinal: ordinal,
                arm: A::NobodyOwnerAuthority { operation },
                expected_outcome: O::ClosedAuthorizationDenial {
                    allowed_raw_os_status: vec![
                        ERR_SEC_AUTH_FAILED_V2,
                        ERR_SEC_INTERACTION_NOT_ALLOWED_V2,
                    ],
                },
                securityagent_expectation: ui,
            });
        }
        ordinal += 1;
        plan.push(GlobalNativeArmPlanEntryV2 {
            group: NativeExperimentGroupV2::ExactFinalizer,
            repetition: repetition.ordinal(),
            sequence_ordinal: ordinal,
            arm: A::FinalizationEffects,
            expected_outcome: O::EffectsComplete,
            securityagent_expectation: ui,
        });
        ordinal += 1;
        plan.push(GlobalNativeArmPlanEntryV2 {
            group: NativeExperimentGroupV2::ExactFinalizer,
            repetition: repetition.ordinal(),
            sequence_ordinal: ordinal,
            arm: A::TerminalBinding,
            expected_outcome: O::HostComplete,
            securityagent_expectation: ui,
        });
    }
    plan
}

fn require_digest(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("global pre-effect binding is not one lowercase SHA-256 digest")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_plan_precommits_both_groups_repetitions_rollback_and_ui() {
        let objects = expected_created_object_inventory_v2();
        let arms = expected_native_arm_plan_v2();
        assert!(objects.iter().any(|object| {
            object.group == NativeExperimentGroupV2::CreatorRoute
                && object.rollback == RollbackDispositionV2::ExactKeychainDeleteThenAbsence
        }));
        assert!(objects.iter().any(|object| {
            object.group == NativeExperimentGroupV2::ExactFinalizer
                && object.rollback == RollbackDispositionV2::BootoutExactLaunchdThenAbsence
        }));
        assert!(objects.iter().any(|object| {
            object.identity == MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2
                && object.rollback == RollbackDispositionV2::RemoveExactTemporaryPathThenAbsence
        }));
        for retained in [
            TERMINAL_ADMIN_CLEANUP_CLAIM_PATH_V2,
            TERMINAL_ADMIN_RESTORATION_RECEIPT_PATH_V2,
        ] {
            assert!(objects.iter().any(|object| {
                object.identity == retained
                    && object.rollback == RollbackDispositionV2::RetainDurableExternalEvidence
            }));
        }
        for repetition in 1..=2 {
            assert_eq!(
                arms.iter()
                    .filter(|arm| {
                        arm.group == NativeExperimentGroupV2::CreatorRoute
                            && arm.repetition == repetition
                    })
                    .count(),
                5
            );
            assert_eq!(
                arms.iter()
                    .filter(|arm| {
                        arm.group == NativeExperimentGroupV2::ExactFinalizer
                            && arm.repetition == repetition
                    })
                    .count(),
                22
            );
        }
        assert!(arms.iter().all(|arm| {
            arm.securityagent_expectation
                == SecurityAgentExpectationV2::NoProcessActivationWindowPromptOrCredentialRequest
        }));
    }
}
