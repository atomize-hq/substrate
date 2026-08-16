//! Typed source and build-input manifests for the pre-install candidate freeze.

use std::path::{Component, Path};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{document_sha256_v2, sha256_hex_v2};

use super::freeze_manifest::CandidateFreezeArtifactRoleV2;
use super::EXPERIMENT_VERSION_V2;

pub const CANDIDATE_FREEZE_SOURCE_HASHES_OWNER_V2: &str =
    "substrate.r3-macos-candidate-source-hashes";
pub const CANDIDATE_FREEZE_BUILD_INPUTS_OWNER_V2: &str =
    "substrate.r3-macos-candidate-build-inputs";
pub const CANDIDATE_FREEZE_RUSTFLAGS_V2: &[&str] = &[
    "-C link-arg=-fuse-ld=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld",
    "-C linker=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang",
];
pub const CANDIDATE_FREEZE_LINKER_FLAGS_V2: &[&str] = &[
    "-fuse-ld=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld",
];
pub const CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a00b11-d9d4-75d2-ad3f-8ea379698723/candidate-freeze/source-hashes.v2.json";

pub const CANDIDATE_FREEZE_SOURCE_PATHS_V2: &[&str] = &[
    "Cargo.lock",
    "Cargo.toml",
    "crates/common/Cargo.toml",
    "crates/common/src/agent_events.rs",
    "crates/common/src/agent_identity.rs",
    "crates/common/src/authority_commitment.rs",
    "crates/common/src/fs_diff.rs",
    "crates/common/src/gateway_auth_bundle.rs",
    "crates/common/src/identity.rs",
    "crates/common/src/lib.rs",
    "crates/common/src/macos_retirement_v2.rs",
    "crates/common/src/managed_artifact.rs",
    "crates/common/src/manager_manifest/mod.rs",
    "crates/common/src/manager_manifest/resolver.rs",
    "crates/common/src/manager_manifest/schema.rs",
    "crates/common/src/manager_manifest/tests.rs",
    "crates/common/src/manager_manifest/validator.rs",
    "crates/common/src/paths.rs",
    "crates/common/src/seccomp.rs",
    "crates/common/src/settings.rs",
    "crates/common/src/world_exec_guard.rs",
    "rust-toolchain.toml",
    "scripts/mac/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist",
    "scripts/mac/freeze-r3-macos-finalizer-candidate.sh",
    "scripts/mac/r3-macos-finalizer-root-install.py",
    "tools/r3-macos-finalizer/Cargo.toml",
    "tools/r3-macos-finalizer/build.rs",
    "tools/r3-macos-finalizer/capability-v2.json",
    "tools/r3-macos-finalizer/native/securityagent_observer.swift",
    "tools/r3-macos-finalizer/src/ambient.rs",
    "tools/r3-macos-finalizer/src/authority.rs",
    "tools/r3-macos-finalizer/src/bin/candidate_freeze_global_provenance_builder.rs",
    "tools/r3-macos-finalizer/src/bin/candidate_freeze_manifest_builder.rs",
    "tools/r3-macos-finalizer/src/bin/candidate_freeze_provenance_builder.rs",
    "tools/r3-macos-finalizer/src/bin/coordinator.rs",
    "tools/r3-macos-finalizer/src/bin/experiment_harness.rs",
    "tools/r3-macos-finalizer/src/bin/experiment_peer_probe.rs",
    "tools/r3-macos-finalizer/src/bin/finalizer.rs",
    "tools/r3-macos-finalizer/src/contract.rs",
    "tools/r3-macos-finalizer/src/darwin.rs",
    "tools/r3-macos-finalizer/src/disposable_capability.rs",
    "tools/r3-macos-finalizer/src/engine.rs",
    "tools/r3-macos-finalizer/src/experiment/controls.rs",
    "tools/r3-macos-finalizer/src/experiment/documents.rs",
    "tools/r3-macos-finalizer/src/experiment/durable.rs",
    "tools/r3-macos-finalizer/src/experiment/evidence_export.rs",
    "tools/r3-macos-finalizer/src/experiment/freeze_manifest.rs",
    "tools/r3-macos-finalizer/src/experiment/freeze_provenance.rs",
    "tools/r3-macos-finalizer/src/experiment/harness_protocol.rs",
    "tools/r3-macos-finalizer/src/experiment/mod.rs",
    "tools/r3-macos-finalizer/src/experiment/peer_probe.rs",
    "tools/r3-macos-finalizer/src/experiment/pre_effect.rs",
    "tools/r3-macos-finalizer/src/experiment/process.rs",
    "tools/r3-macos-finalizer/src/experiment/publisher_protocol.rs",
    "tools/r3-macos-finalizer/src/fixed_inbox.rs",
    "tools/r3-macos-finalizer/src/frame.rs",
    "tools/r3-macos-finalizer/src/journal.rs",
    "tools/r3-macos-finalizer/src/lib.rs",
    "tools/r3-macos-finalizer/src/native_effects.rs",
    "tools/r3-macos-finalizer/src/targets.rs",
    "tools/r3-macos-signer-acl/Cargo.lock",
    "tools/r3-macos-signer-acl/Cargo.toml",
    "tools/r3-macos-signer-acl/build.rs",
    "tools/r3-macos-signer-acl/native/benign_injection_probe.c",
    "tools/r3-macos-signer-acl/src/attestation.rs",
    "tools/r3-macos-signer-acl/src/bin/creator_route.rs",
    "tools/r3-macos-signer-acl/src/bin/disposable_publisher.rs",
    "tools/r3-macos-signer-acl/src/bin/experiment_runner.rs",
    "tools/r3-macos-signer-acl/src/bin/nobody_owner_probe.rs",
    "tools/r3-macos-signer-acl/src/bin/wrong_identity.rs",
    "tools/r3-macos-signer-acl/src/experiment.rs",
    "tools/r3-macos-signer-acl/src/ffi.rs",
    "tools/r3-macos-signer-acl/src/immutable_publish.rs",
    "tools/r3-macos-signer-acl/src/lib.rs",
    "tools/r3-macos-signer-acl/src/owner_probe.rs",
    "tools/r3-macos-signer-acl/src/publisher.rs",
    "tools/r3-macos-signer-acl/src/publisher_surface.rs",
    "tools/r3-macos-signer-acl/src/runner.rs",
    "tools/r3-macos-signer-acl/src/runner_identity.rs",
    "tools/r3-macos-signer-acl/src/securityagent.rs",
    "tools/r3-macos-signer-acl/src/supervisor.rs",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeSourceHashEntryV2 {
    pub repository_relative_path: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeSourceHashesInputV2 {
    pub entries: Vec<CandidateFreezeSourceHashEntryV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeSourceHashesManifestV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub repository_path: String,
    pub source_commit: String,
    pub source_tree: String,
    pub entries: Vec<CandidateFreezeSourceHashEntryV2>,
    pub entry_set_sha256: String,
}

impl CandidateFreezeSourceHashesManifestV2 {
    pub fn validate(
        &self,
        repository_path: &str,
        source_commit: &str,
        source_tree: &str,
    ) -> Result<()> {
        if self.schema_owner != CANDIDATE_FREEZE_SOURCE_HASHES_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.repository_path != repository_path
            || self.source_commit != source_commit
            || self.source_tree != source_tree
            || self.entries.len() != CANDIDATE_FREEZE_SOURCE_PATHS_V2.len()
            || self.entry_set_sha256 != source_entry_set_sha256_v2(&self.entries)?
        {
            bail!("candidate source-hashes manifest changed its fixed identity or aggregate")
        }
        for (entry, expected_path) in self.entries.iter().zip(CANDIDATE_FREEZE_SOURCE_PATHS_V2) {
            validate_relative_path(&entry.repository_relative_path)?;
            if entry.repository_relative_path != *expected_path
                || entry.byte_length == 0
                || !is_sha256(&entry.sha256)
            {
                bail!("candidate source-hashes manifest omitted or changed a frozen source")
            }
        }
        Ok(())
    }
}

pub fn build_candidate_freeze_source_hashes_manifest_v2(
    input: CandidateFreezeSourceHashesInputV2,
    repository_path: &str,
    source_commit: &str,
    source_tree: &str,
) -> Result<CandidateFreezeSourceHashesManifestV2> {
    let manifest = CandidateFreezeSourceHashesManifestV2 {
        schema_owner: CANDIDATE_FREEZE_SOURCE_HASHES_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        repository_path: repository_path.to_string(),
        source_commit: source_commit.to_string(),
        source_tree: source_tree.to_string(),
        entry_set_sha256: source_entry_set_sha256_v2(&input.entries)?,
        entries: input.entries,
    };
    manifest.validate(repository_path, source_commit, source_tree)?;
    Ok(manifest)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeBuildLaneV2 {
    Coordinator,
    Global,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeBuildToolV2 {
    Rustc,
    Cargo,
    Clang,
    Ld,
    Swiftc,
    Codesign,
    Xcrun,
}

impl CandidateFreezeBuildToolV2 {
    const fn executable_path(self) -> &'static str {
        match self {
            Self::Rustc => "/Users/spensermcconnell/.cargo/bin/rustc",
            Self::Cargo => "/Users/spensermcconnell/.cargo/bin/cargo",
            Self::Clang => "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang",
            Self::Ld => "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/ld",
            Self::Swiftc => "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc",
            Self::Codesign => "/usr/bin/codesign",
            Self::Xcrun => "/usr/bin/xcrun",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeBuildToolIdentityV2 {
    pub tool: CandidateFreezeBuildToolV2,
    pub executable_path: String,
    pub executable_sha256: String,
    pub version_output_base64url: String,
    pub version_output_sha256: String,
    pub version_output_byte_length: u64,
}

impl CandidateFreezeBuildToolIdentityV2 {
    fn validate(&self, expected: CandidateFreezeBuildToolV2) -> Result<()> {
        let version = URL_SAFE_NO_PAD
            .decode(&self.version_output_base64url)
            .context("decode frozen build-tool version output")?;
        if self.tool != expected
            || self.executable_path != expected.executable_path()
            || self.executable_path.contains(['\0', '\n', '\r'])
            || !is_sha256(&self.executable_sha256)
            || version.is_empty()
            || version.len() > 64 * 1024
            || self.version_output_byte_length
                != u64::try_from(version.len()).context("tool version length exceeds u64")?
            || self.version_output_sha256 != sha256_hex_v2(&version)
        {
            bail!("candidate build tool changed its exact path, bytes, version, or order")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeBuildEnvironmentEntryV2 {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeCoordinatorPrerequisiteV2 {
    pub source_commit: String,
    pub source_tree: String,
    pub source_hashes_sha256: String,
    pub build_inputs_sha256: String,
    pub executable_sha256: String,
    pub executable_size: u64,
    pub external_path: String,
    pub intended_path: String,
    pub signing_identifier: String,
    pub designated_requirement: String,
    pub cdhash: String,
    pub code_flags: u32,
    pub team_id: Option<String>,
    pub entitlements_size: u64,
    pub entitlements_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateFreezeBuildRecipeToolV2 {
    Cargo,
    Swiftc,
    Clang,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeBuildRecipeV2 {
    pub sequence_ordinal: u8,
    pub tool: CandidateFreezeBuildRecipeToolV2,
    pub package_manifest: Option<String>,
    pub cargo_features: Vec<String>,
    pub binary_products: Vec<String>,
    pub artifact_roles: Vec<CandidateFreezeArtifactRoleV2>,
}

pub fn expected_candidate_freeze_build_recipes_v2(
    lane: CandidateFreezeBuildLaneV2,
) -> Vec<CandidateFreezeBuildRecipeV2> {
    let recipe =
        |sequence_ordinal,
         tool,
         package_manifest: Option<&str>,
         cargo_features: &[&str],
         binary_products: &[&str],
         artifact_roles: &[CandidateFreezeArtifactRoleV2]| CandidateFreezeBuildRecipeV2 {
            sequence_ordinal,
            tool,
            package_manifest: package_manifest.map(str::to_string),
            cargo_features: cargo_features
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            binary_products: binary_products
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            artifact_roles: artifact_roles.to_vec(),
        };
    match lane {
        CandidateFreezeBuildLaneV2::Coordinator => vec![recipe(
            1,
            CandidateFreezeBuildRecipeToolV2::Cargo,
            Some("tools/r3-macos-finalizer/Cargo.toml"),
            &[],
            &["substrate-r3-macos-evidence-coordinator"],
            &[
                CandidateFreezeArtifactRoleV2::CoordinatorExecutable,
                CandidateFreezeArtifactRoleV2::AlternateCoordinatorExecutable,
            ],
        )],
        CandidateFreezeBuildLaneV2::Global => vec![
            recipe(
                1,
                CandidateFreezeBuildRecipeToolV2::Cargo,
                Some("tools/r3-macos-finalizer/Cargo.toml"),
                &[],
                &[
                    "substrate-r3-macos-evidence-finalizer",
                    "substrate-r3-macos-peer-code-probe",
                ],
                &[
                    CandidateFreezeArtifactRoleV2::FinalizerExecutable,
                    CandidateFreezeArtifactRoleV2::PeerCodeProbeExecutable,
                ],
            ),
            recipe(
                2,
                CandidateFreezeBuildRecipeToolV2::Cargo,
                Some("tools/r3-macos-finalizer/Cargo.toml"),
                &["experiment-harness"],
                &["substrate-r3-macos-disposable-harness"],
                &[CandidateFreezeArtifactRoleV2::DisposableHarnessExecutable],
            ),
            recipe(
                3,
                CandidateFreezeBuildRecipeToolV2::Cargo,
                Some("tools/r3-macos-finalizer/Cargo.toml"),
                &["candidate-freeze-builder"],
                &[
                    "substrate-r3-macos-candidate-freeze-provenance-builder",
                    "substrate-r3-macos-candidate-freeze-global-provenance-builder",
                    "substrate-r3-macos-candidate-freeze-manifest-builder",
                ],
                &[],
            ),
            recipe(
                4,
                CandidateFreezeBuildRecipeToolV2::Cargo,
                Some("tools/r3-macos-signer-acl/Cargo.toml"),
                &[],
                &[
                    "substrate-r3-macos-signer-acl-creator",
                    "substrate-r3-macos-signer-acl-wrong-identity",
                    "substrate-r3-macos-disposable-publisher",
                    "substrate-r3-macos-disposable-experiment-runner",
                    "substrate-r3-macos-nobody-owner-probe",
                ],
                &[
                    CandidateFreezeArtifactRoleV2::CreatorExecutable,
                    CandidateFreezeArtifactRoleV2::WrongIdentityExecutable,
                    CandidateFreezeArtifactRoleV2::DisposablePublisherExecutable,
                    CandidateFreezeArtifactRoleV2::DisposableExperimentRunnerExecutable,
                    CandidateFreezeArtifactRoleV2::NobodyOwnerProbeExecutable,
                ],
            ),
            recipe(
                5,
                CandidateFreezeBuildRecipeToolV2::Swiftc,
                None,
                &[],
                &["substrate-r3-macos-securityagent-observer"],
                &[CandidateFreezeArtifactRoleV2::SecurityAgentObserverExecutable],
            ),
            recipe(
                6,
                CandidateFreezeBuildRecipeToolV2::Clang,
                None,
                &[],
                &["substrate-r3-macos-benign-injection-probe.dylib"],
                &[CandidateFreezeArtifactRoleV2::BenignInjectionLibrary],
            ),
        ],
    }
}

impl CandidateFreezeCoordinatorPrerequisiteV2 {
    pub fn validate_for(
        &self,
        source_manifest: &CandidateFreezeSourceHashesManifestV2,
        coordinator_build: &CandidateFreezeBuildInputManifestV2,
    ) -> Result<()> {
        let expected_external = format!(
            "{}/artifacts/{}",
            super::freeze_manifest::CANDIDATE_FREEZE_ARTIFACT_ROOT_V2,
            CandidateFreezeArtifactRoleV2::CoordinatorExecutable.external_filename()
        );
        if self.source_commit != source_manifest.source_commit
            || self.source_tree != source_manifest.source_tree
            || self.source_hashes_sha256 != document_sha256_v2(source_manifest)?
            || self.build_inputs_sha256 != coordinator_build.input_set_sha256
            || !is_sha256(&self.executable_sha256)
            || self.executable_size == 0
            || self.external_path != expected_external
            || self.intended_path
                != CandidateFreezeArtifactRoleV2::CoordinatorExecutable.intended_path()
            || self.signing_identifier
                != CandidateFreezeArtifactRoleV2::CoordinatorExecutable
                    .signing_identifier()
                    .expect("coordinator is code")
            || self.designated_requirement.is_empty()
            || !is_cdhash(&self.cdhash)
            || self.code_flags != super::AD_HOC_HARDENED_RUNTIME_FLAGS_V2
            || self.team_id.is_some()
            || self.entitlements_size != 0
            || self.entitlements_sha256 != super::EMPTY_ENTITLEMENTS_SHA256_V2
        {
            bail!("global build prerequisite is not the exact frozen coordinator artifact")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeBuildInputManifestInputV2 {
    pub lane: CandidateFreezeBuildLaneV2,
    pub coordinator_prerequisite: Option<CandidateFreezeCoordinatorPrerequisiteV2>,
    pub tools: Vec<CandidateFreezeBuildToolIdentityV2>,
    pub macos_sdk_path: String,
    pub macos_sdk_version: String,
    pub macos_sdk_settings_sha256: String,
    pub target_triple: String,
    pub macos_deployment_target: String,
    pub cargo_profile: String,
    pub cargo_locked: bool,
    pub recipes: Vec<CandidateFreezeBuildRecipeV2>,
    pub rustflags: Vec<String>,
    pub linker_flags: Vec<String>,
    pub frozen_environment: Vec<CandidateFreezeBuildEnvironmentEntryV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateFreezeBuildInputManifestV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub lane: CandidateFreezeBuildLaneV2,
    pub repository_path: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_hashes_manifest_path: String,
    pub source_hashes_manifest_sha256: String,
    pub capability_manifest_sha256: String,
    pub launchd_plist_sha256: String,
    pub coordinator_prerequisite: Option<CandidateFreezeCoordinatorPrerequisiteV2>,
    pub tools: Vec<CandidateFreezeBuildToolIdentityV2>,
    pub macos_sdk_path: String,
    pub macos_sdk_version: String,
    pub macos_sdk_settings_sha256: String,
    pub target_triple: String,
    pub macos_deployment_target: String,
    pub cargo_profile: String,
    pub cargo_locked: bool,
    pub recipes: Vec<CandidateFreezeBuildRecipeV2>,
    pub rustflags: Vec<String>,
    pub linker_flags: Vec<String>,
    pub frozen_environment: Vec<CandidateFreezeBuildEnvironmentEntryV2>,
    pub input_set_sha256: String,
    pub derived_build_inputs_environment: CandidateFreezeBuildEnvironmentEntryV2,
}

impl CandidateFreezeBuildInputManifestV2 {
    pub fn validate(&self) -> Result<()> {
        if self.schema_owner != CANDIDATE_FREEZE_BUILD_INPUTS_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || !is_sha256(&self.source_hashes_manifest_sha256)
            || !is_sha256(&self.capability_manifest_sha256)
            || !is_sha256(&self.launchd_plist_sha256)
            || !is_sha256(&self.macos_sdk_settings_sha256)
            || self.source_hashes_manifest_path != CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2
            || !Path::new(&self.macos_sdk_path).is_absolute()
            || !self.cargo_locked
            || self.cargo_profile != "release"
            || self.input_set_sha256 != build_input_set_sha256_v2(self)?
            || self.derived_build_inputs_environment.name != "R3_BUILD_INPUTS_SHA256"
            || self.derived_build_inputs_environment.value != self.input_set_sha256
        {
            bail!("candidate build-input manifest changed its fixed identity or build posture")
        }
        let expected_tools = match self.lane {
            CandidateFreezeBuildLaneV2::Coordinator => &[
                CandidateFreezeBuildToolV2::Rustc,
                CandidateFreezeBuildToolV2::Cargo,
                CandidateFreezeBuildToolV2::Clang,
                CandidateFreezeBuildToolV2::Ld,
                CandidateFreezeBuildToolV2::Codesign,
                CandidateFreezeBuildToolV2::Xcrun,
            ][..],
            CandidateFreezeBuildLaneV2::Global => &[
                CandidateFreezeBuildToolV2::Rustc,
                CandidateFreezeBuildToolV2::Cargo,
                CandidateFreezeBuildToolV2::Clang,
                CandidateFreezeBuildToolV2::Ld,
                CandidateFreezeBuildToolV2::Swiftc,
                CandidateFreezeBuildToolV2::Codesign,
                CandidateFreezeBuildToolV2::Xcrun,
            ][..],
        };
        if (self.lane == CandidateFreezeBuildLaneV2::Coordinator
            && self.coordinator_prerequisite.is_some())
            || (self.lane == CandidateFreezeBuildLaneV2::Global
                && self.coordinator_prerequisite.is_none())
        {
            bail!("candidate build lane changed its typed coordinator prerequisite")
        }
        if self.tools.len() != expected_tools.len() {
            bail!("candidate build-input manifest omitted a frozen build tool")
        }
        for (tool, expected) in self.tools.iter().zip(expected_tools) {
            tool.validate(*expected)?;
        }
        if self.recipes != expected_candidate_freeze_build_recipes_v2(self.lane) {
            bail!("candidate build-input manifest changed its ordered feature-isolated recipes")
        }
        for value in [
            &self.repository_path,
            &self.source_commit,
            &self.source_tree,
            &self.macos_sdk_version,
            &self.target_triple,
            &self.macos_deployment_target,
        ] {
            require_bounded_plain(value)?;
        }
        validate_sorted_unique_plain(&self.rustflags)?;
        validate_sorted_unique_plain(&self.linker_flags)?;
        if self.rustflags
            != CANDIDATE_FREEZE_RUSTFLAGS_V2
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
            || self.linker_flags
                != CANDIDATE_FREEZE_LINKER_FLAGS_V2
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>()
        {
            bail!("candidate build input changed the exact Rust linker or Darwin ld flags")
        }
        let mut prior = None;
        for entry in &self.frozen_environment {
            require_bounded_plain(&entry.name)?;
            if !entry
                .name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
                || entry.value.len() > 16 * 1024
                || entry.value.contains('\0')
                || prior.is_some_and(|value: &str| value >= entry.name.as_str())
            {
                bail!("candidate frozen build environment is not sorted, unique, and bounded")
            }
            prior = Some(entry.name.as_str());
        }
        let expected_rustflags = CANDIDATE_FREEZE_RUSTFLAGS_V2.join(" ");
        if self
            .frozen_environment
            .iter()
            .find(|entry| entry.name == "RUSTFLAGS")
            .map(|entry| entry.value.as_str())
            != Some(expected_rustflags.as_str())
        {
            bail!("candidate frozen environment did not apply its exact Rust linker flags")
        }
        Ok(())
    }
}

pub fn build_candidate_freeze_build_input_manifest_v2(
    input: CandidateFreezeBuildInputManifestInputV2,
    repository_path: &str,
    source_commit: &str,
    source_tree: &str,
    source_hashes_manifest_sha256: &str,
    capability_manifest_sha256: &str,
    launchd_plist_sha256: &str,
) -> Result<CandidateFreezeBuildInputManifestV2> {
    let mut manifest = CandidateFreezeBuildInputManifestV2 {
        schema_owner: CANDIDATE_FREEZE_BUILD_INPUTS_OWNER_V2.to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        lane: input.lane,
        repository_path: repository_path.to_string(),
        source_commit: source_commit.to_string(),
        source_tree: source_tree.to_string(),
        source_hashes_manifest_path: CANDIDATE_FREEZE_SOURCE_HASHES_PATH_V2.to_string(),
        source_hashes_manifest_sha256: source_hashes_manifest_sha256.to_string(),
        capability_manifest_sha256: capability_manifest_sha256.to_string(),
        launchd_plist_sha256: launchd_plist_sha256.to_string(),
        coordinator_prerequisite: input.coordinator_prerequisite,
        tools: input.tools,
        macos_sdk_path: input.macos_sdk_path,
        macos_sdk_version: input.macos_sdk_version,
        macos_sdk_settings_sha256: input.macos_sdk_settings_sha256,
        target_triple: input.target_triple,
        macos_deployment_target: input.macos_deployment_target,
        cargo_profile: input.cargo_profile,
        cargo_locked: input.cargo_locked,
        recipes: input.recipes,
        rustflags: input.rustflags,
        linker_flags: input.linker_flags,
        frozen_environment: input.frozen_environment,
        input_set_sha256: String::new(),
        derived_build_inputs_environment: CandidateFreezeBuildEnvironmentEntryV2 {
            name: "R3_BUILD_INPUTS_SHA256".to_string(),
            value: String::new(),
        },
    };
    manifest.input_set_sha256 = build_input_set_sha256_v2(&manifest)?;
    manifest.derived_build_inputs_environment.value = manifest.input_set_sha256.clone();
    manifest.validate()?;
    Ok(manifest)
}

fn source_entry_set_sha256_v2(entries: &[CandidateFreezeSourceHashEntryV2]) -> Result<String> {
    #[derive(Serialize)]
    struct Binding<'a> {
        domain: &'static str,
        entries: &'a [CandidateFreezeSourceHashEntryV2],
    }
    document_sha256_v2(&Binding {
        domain: "substrate.r3-macos-candidate-source-hash-set.v2",
        entries,
    })
}

fn build_input_set_sha256_v2(manifest: &CandidateFreezeBuildInputManifestV2) -> Result<String> {
    #[derive(Serialize)]
    struct Binding<'a> {
        domain: &'static str,
        lane: CandidateFreezeBuildLaneV2,
        repository_path: &'a str,
        source_commit: &'a str,
        source_tree: &'a str,
        source_hashes_manifest_path: &'a str,
        source_hashes_manifest_sha256: &'a str,
        capability_manifest_sha256: &'a str,
        launchd_plist_sha256: &'a str,
        coordinator_prerequisite: &'a Option<CandidateFreezeCoordinatorPrerequisiteV2>,
        tools: &'a [CandidateFreezeBuildToolIdentityV2],
        macos_sdk_path: &'a str,
        macos_sdk_version: &'a str,
        macos_sdk_settings_sha256: &'a str,
        target_triple: &'a str,
        macos_deployment_target: &'a str,
        cargo_profile: &'a str,
        cargo_locked: bool,
        recipes: &'a [CandidateFreezeBuildRecipeV2],
        rustflags: &'a [String],
        linker_flags: &'a [String],
        frozen_environment: &'a [CandidateFreezeBuildEnvironmentEntryV2],
    }
    document_sha256_v2(&Binding {
        domain: "substrate.r3-macos-candidate-build-input-set.v2",
        lane: manifest.lane,
        repository_path: &manifest.repository_path,
        source_commit: &manifest.source_commit,
        source_tree: &manifest.source_tree,
        source_hashes_manifest_path: &manifest.source_hashes_manifest_path,
        source_hashes_manifest_sha256: &manifest.source_hashes_manifest_sha256,
        capability_manifest_sha256: &manifest.capability_manifest_sha256,
        launchd_plist_sha256: &manifest.launchd_plist_sha256,
        coordinator_prerequisite: &manifest.coordinator_prerequisite,
        tools: &manifest.tools,
        macos_sdk_path: &manifest.macos_sdk_path,
        macos_sdk_version: &manifest.macos_sdk_version,
        macos_sdk_settings_sha256: &manifest.macos_sdk_settings_sha256,
        target_triple: &manifest.target_triple,
        macos_deployment_target: &manifest.macos_deployment_target,
        cargo_profile: &manifest.cargo_profile,
        cargo_locked: manifest.cargo_locked,
        recipes: &manifest.recipes,
        rustflags: &manifest.rustflags,
        linker_flags: &manifest.linker_flags,
        frozen_environment: &manifest.frozen_environment,
    })
}

fn validate_relative_path(value: &str) -> Result<()> {
    if value.is_empty()
        || Path::new(value).is_absolute()
        || Path::new(value)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("candidate source path is not one normalized repository-relative path")
    }
    Ok(())
}

fn validate_sorted_unique_plain(values: &[String]) -> Result<()> {
    let mut prior = None;
    for value in values {
        require_bounded_plain(value)?;
        if prior.is_some_and(|previous: &str| previous >= value.as_str()) {
            bail!("candidate build option vector is not sorted and unique")
        }
        prior = Some(value.as_str());
    }
    Ok(())
}

fn require_bounded_plain(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 16 * 1024 || value.contains(['\0', '\n', '\r']) {
        bail!("candidate build input is empty, unbounded, or non-plain")
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
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

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn tool(tool: CandidateFreezeBuildToolV2, seed: u8) -> CandidateFreezeBuildToolIdentityV2 {
        let version = format!("frozen-tool-{seed}").into_bytes();
        CandidateFreezeBuildToolIdentityV2 {
            tool,
            executable_path: tool.executable_path().to_string(),
            executable_sha256: digest(seed),
            version_output_base64url: URL_SAFE_NO_PAD.encode(&version),
            version_output_sha256: sha256_hex_v2(&version),
            version_output_byte_length: u64::try_from(version.len()).unwrap(),
        }
    }

    fn build_input(lane: CandidateFreezeBuildLaneV2) -> CandidateFreezeBuildInputManifestInputV2 {
        let tools = match lane {
            CandidateFreezeBuildLaneV2::Coordinator => {
                vec![
                    tool(CandidateFreezeBuildToolV2::Rustc, 1),
                    tool(CandidateFreezeBuildToolV2::Cargo, 2),
                    tool(CandidateFreezeBuildToolV2::Clang, 3),
                    tool(CandidateFreezeBuildToolV2::Ld, 4),
                    tool(CandidateFreezeBuildToolV2::Codesign, 5),
                    tool(CandidateFreezeBuildToolV2::Xcrun, 6),
                ]
            }
            CandidateFreezeBuildLaneV2::Global => {
                vec![
                    tool(CandidateFreezeBuildToolV2::Rustc, 1),
                    tool(CandidateFreezeBuildToolV2::Cargo, 2),
                    tool(CandidateFreezeBuildToolV2::Clang, 3),
                    tool(CandidateFreezeBuildToolV2::Ld, 4),
                    tool(CandidateFreezeBuildToolV2::Swiftc, 5),
                    tool(CandidateFreezeBuildToolV2::Codesign, 6),
                    tool(CandidateFreezeBuildToolV2::Xcrun, 7),
                ]
            }
        };
        CandidateFreezeBuildInputManifestInputV2 {
            lane,
            coordinator_prerequisite: (lane == CandidateFreezeBuildLaneV2::Global).then(|| {
                CandidateFreezeCoordinatorPrerequisiteV2 {
                    source_commit: "1".repeat(40),
                    source_tree: "2".repeat(40),
                    source_hashes_sha256: digest(70),
                    build_inputs_sha256: digest(71),
                    executable_sha256: digest(72),
                    executable_size: 1,
                    external_path: format!(
                        "{}/artifacts/{}",
                        super::super::freeze_manifest::CANDIDATE_FREEZE_ARTIFACT_ROOT_V2,
                        CandidateFreezeArtifactRoleV2::CoordinatorExecutable.external_filename()
                    ),
                    intended_path: CandidateFreezeArtifactRoleV2::CoordinatorExecutable
                        .intended_path()
                        .to_string(),
                    signing_identifier: CandidateFreezeArtifactRoleV2::CoordinatorExecutable
                        .signing_identifier()
                        .unwrap()
                        .to_string(),
                    designated_requirement:
                        "identifier frozen and cdhash H\"3333333333333333333333333333333333333333\""
                            .to_string(),
                    cdhash: "3".repeat(40),
                    code_flags: super::super::AD_HOC_HARDENED_RUNTIME_FLAGS_V2,
                    team_id: None,
                    entitlements_size: 0,
                    entitlements_sha256: super::super::EMPTY_ENTITLEMENTS_SHA256_V2.to_string(),
                }
            }),
            tools,
            macos_sdk_path: "/frozen/MacOSX.sdk".to_string(),
            macos_sdk_version: "15.5".to_string(),
            macos_sdk_settings_sha256: digest(7),
            target_triple: "aarch64-apple-darwin".to_string(),
            macos_deployment_target: "15.0".to_string(),
            cargo_profile: "release".to_string(),
            cargo_locked: true,
            recipes: expected_candidate_freeze_build_recipes_v2(lane),
            rustflags: CANDIDATE_FREEZE_RUSTFLAGS_V2
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            linker_flags: CANDIDATE_FREEZE_LINKER_FLAGS_V2
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            frozen_environment: vec![
                CandidateFreezeBuildEnvironmentEntryV2 {
                    name: "RUSTFLAGS".to_string(),
                    value: CANDIDATE_FREEZE_RUSTFLAGS_V2.join(" "),
                },
                CandidateFreezeBuildEnvironmentEntryV2 {
                    name: "SOURCE_DATE_EPOCH".to_string(),
                    value: "1".to_string(),
                },
            ],
        }
    }

    #[test]
    fn source_and_build_manifests_are_closed_canonical_digest_authorities() {
        let repository = super::super::freeze_manifest::CANDIDATE_FREEZE_REPOSITORY_PATH_V2;
        let commit = "1".repeat(40);
        let tree = "2".repeat(40);
        let source = build_candidate_freeze_source_hashes_manifest_v2(
            CandidateFreezeSourceHashesInputV2 {
                entries: CANDIDATE_FREEZE_SOURCE_PATHS_V2
                    .iter()
                    .enumerate()
                    .map(|(index, path)| CandidateFreezeSourceHashEntryV2 {
                        repository_relative_path: (*path).to_string(),
                        byte_length: u64::try_from(index + 1).unwrap(),
                        sha256: digest(u8::try_from(index + 1).unwrap()),
                    })
                    .collect(),
            },
            repository,
            &commit,
            &tree,
        )
        .unwrap();
        let source_digest = document_sha256_v2(&source).unwrap();
        for lane in [
            CandidateFreezeBuildLaneV2::Coordinator,
            CandidateFreezeBuildLaneV2::Global,
        ] {
            let manifest = build_candidate_freeze_build_input_manifest_v2(
                build_input(lane),
                repository,
                &commit,
                &tree,
                &source_digest,
                &digest(80),
                &digest(81),
            )
            .unwrap();
            manifest.validate().unwrap();
            assert_eq!(
                manifest.derived_build_inputs_environment.value,
                manifest.input_set_sha256
            );
            assert_ne!(
                manifest.input_set_sha256,
                document_sha256_v2(&manifest).unwrap()
            );
            assert_eq!(
                sha256_hex_v2(
                    &substrate_common::macos_retirement_v2::canonical_bytes_v2(&manifest).unwrap()
                ),
                document_sha256_v2(&manifest).unwrap()
            );
        }

        let mut omitted = source;
        omitted.entries.pop();
        assert!(omitted.validate(repository, &commit, &tree).is_err());
    }

    #[test]
    fn frozen_source_paths_equal_the_local_compiled_source_closure() {
        fn collect_rs(root: &Path, directory: &Path, values: &mut Vec<String>) {
            for entry in std::fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect_rs(root, &path, values);
                } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                    values.push(
                        path.strip_prefix(root)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                    );
                }
            }
        }

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repository = manifest_dir.parent().unwrap().parent().unwrap();
        let mut actual = vec![
            "Cargo.lock".to_string(),
            "Cargo.toml".to_string(),
            "crates/common/Cargo.toml".to_string(),
            "rust-toolchain.toml".to_string(),
            "scripts/mac/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist".to_string(),
            "scripts/mac/freeze-r3-macos-finalizer-candidate.sh".to_string(),
            "scripts/mac/r3-macos-finalizer-root-install.py".to_string(),
            "tools/r3-macos-finalizer/Cargo.toml".to_string(),
            "tools/r3-macos-finalizer/build.rs".to_string(),
            "tools/r3-macos-finalizer/capability-v2.json".to_string(),
            "tools/r3-macos-finalizer/native/securityagent_observer.swift".to_string(),
            "tools/r3-macos-signer-acl/Cargo.lock".to_string(),
            "tools/r3-macos-signer-acl/Cargo.toml".to_string(),
            "tools/r3-macos-signer-acl/build.rs".to_string(),
            "tools/r3-macos-signer-acl/native/benign_injection_probe.c".to_string(),
        ];
        for directory in [
            "crates/common/src",
            "tools/r3-macos-finalizer/src",
            "tools/r3-macos-signer-acl/src",
        ] {
            collect_rs(repository, &repository.join(directory), &mut actual);
        }
        actual.sort();
        actual.dedup();
        assert_eq!(
            actual,
            CANDIDATE_FREEZE_SOURCE_PATHS_V2
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
        );
    }
}
