//! Independently authenticated inputs for the stock Codex 0.156.1 release pair.
use serde::{Deserialize, Serialize};

use crate::{E3ElfExecutionModelV1, RuntimeArtifactProvenanceV1, RuntimeSupportFileV1};

/// Additional source identity for the156 loader split and native catalogue/cache.
/// These hashes bind official source; they do not attest execution of the release.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Codex0156LoaderSourcesV1 {
    pub upstream_commit: String,
    pub upstream_tree: String,
    pub files: std::collections::BTreeMap<String, String>,
    pub builtin_catalogue_sha256: String,
    pub models_cache_filename: String,
    pub cloud_cache_filename: String,
}

/// Stock156 rendering uses the common nonsecret document/publication mechanics.
pub struct Codex0156ProjectionV1;

/// Opaque156 plan, kept distinct from the historical125 public renderer input.
pub struct Codex0156ProjectionPlanV1(pub(crate) crate::Codex0125ProjectionPlanV1);

impl Codex0156ProjectionV1 {
    /// Validates the156 source identity, exact startup locators and fingerprint.
    /// Filesystem identity/readback remains the registry's separate responsibility.
    pub fn validate_loader_inputs(
        closure: &crate::CodexAmbientConfigClosureV1,
    ) -> Result<(), crate::ConfigProjectionFailureV1> {
        use crate::{CodexLoaderInputDispositionV1 as D, ConfigProjectionFailureV1 as F};
        if closure.loader_source != pinned_loader_source_v1() {
            return Err(F::UnsupportedRuntimeVersion);
        }
        if closure.allowed_enabled_layers != ["System", "User"]
            || closure.forbidden_cli_overrides
                != [
                    "--config",
                    "--ignore-rules",
                    "--ignore-user-config",
                    "--model",
                    "--oss",
                    "--profile",
                    "-c",
                ]
        {
            return Err(F::UnsupportedPolicySurface);
        }
        let mut expected = Vec::new();
        for layer in ["System", "User"] {
            for path in [
                "config.toml",
                "managed_config.toml",
                "requirements.toml",
                "rules",
                "skills",
            ] {
                expected.push((
                    layer,
                    path,
                    if layer == "User" && path == "config.toml" {
                        D::Projected
                    } else {
                        D::ProvenAbsent
                    },
                ));
            }
        }
        for path in [".codex/config.toml", ".codex/rules", ".codex/skills"] {
            expected.push(("Project", path, D::DisabledByTrust));
        }
        expected.extend([
            (
                "McpCredentials",
                ".credentials.json",
                D::DisabledByEmptyMcpSetAndProvenAbsent,
            ),
            (
                "Auth",
                "auth.json",
                D::DisabledByEphemeralCredentialStoreAndProvenAbsent,
            ),
            (
                "CloudRequirements",
                "cloud-config-bundle-cache.json",
                D::DisabledByNoEphemeralAuthAndProvenAbsent,
            ),
            ("ModelsCache", "models_cache.json", D::ProvenAbsent),
        ]);
        if closure.inputs.len() != expected.len() {
            return Err(F::WrongBinding);
        }
        for (input, (layer, path, disposition)) in closure.inputs.iter().zip(expected) {
            if input.layer != layer
                || input.relative_path != path
                || input.disposition != disposition
                || input.locator
                    != format!(
                        "{}/{}",
                        input.directory.physical_path.trim_end_matches('/'),
                        path
                    )
                || !matches!(
                    input.directory.physical_identity,
                    crate::DirectoryPhysicalIdentityV1::Linux {
                        device_id: 1..,
                        inode: 1..
                    }
                )
            {
                return Err(F::WrongBinding);
            }
            let present = [
                input.device_id.is_some(),
                input.inode.is_some(),
                input.byte_length.is_some(),
                input.sha256.is_some(),
            ];
            if (disposition == D::Projected && present != [true; 4])
                || (disposition == D::DisabledByTrust
                    && present != [true; 4]
                    && present != [false; 4])
                || (!matches!(disposition, D::Projected | D::DisabledByTrust)
                    && present != [false; 4])
                || input.device_id == Some(0)
                || input.inode == Some(0)
            {
                return Err(F::WrongBinding);
            }
        }
        if crate::ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.codex-0.156-loader-inputs.v1",
            &serde_json::json!({"inputs":closure.inputs,"loader_source":closure.loader_source}),
        )? != closure.validated_loader_input_fingerprint
        {
            return Err(F::HashInvalid);
        }
        Ok(())
    }

    pub fn render(
        identity: &crate::ConfigProjectionIdentityV1,
        effective: &crate::EffectiveAgentConfigProjectionV1,
        gateway: &crate::ManagedGatewayProjectionV1,
        root: crate::NativeProjectionRootV1,
        fence_id: &str,
        project_inputs: Vec<crate::CodexLoaderInputAttestationV1>,
    ) -> Result<Codex0156ProjectionPlanV1, crate::ConfigProjectionFailureV1> {
        let main = &identity.runtime_artifacts.codex;
        let pin = Codex0156ReleaseBundleV1::pinned().main;
        if main.role != crate::ConfigProjectionArtifactRoleV1::Codex0156
            || main.configured_absolute_path != CODEX_0156_MAIN_PATH
            || main.provenance != pin.provenance()
            || main.sha256 != pin.extracted_executable_sha256
            || main.byte_length != pin.extracted_byte_length
        {
            return Err(crate::ConfigProjectionFailureV1::UnsupportedRuntimeVersion);
        }
        validate_support_version_v1(&main.runtime_support)?;
        if main.runtime_support.code_mode_host.is_none() {
            return Err(crate::ConfigProjectionFailureV1::WrongBinding);
        }
        crate::Codex0125ProjectionV1::render_for_release_v1(
            identity,
            effective,
            gateway,
            root,
            fence_id,
            project_inputs,
            true,
        )
        .map(Codex0156ProjectionPlanV1)
    }
}

pub const CODEX_0156_VERSION: &str = "0.156.1";
pub const CODEX_0156_TARGET: &str = "x86_64-unknown-linux-musl";
pub const CODEX_0156_MAIN_PATH: &str = "/var/lib/substrate/world-deps/codex-runtime/bin/codex";
pub const CODEX_0156_HOST_PATH: &str =
    "/var/lib/substrate/world-deps/codex-runtime/bin/codex-code-mode-host";

/// One independently authenticated official archive and its sole regular member.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Codex0156OfficialArchiveV1 {
    pub archive_name: String,
    pub archive_url: String,
    pub archive_sha256: String,
    pub archive_byte_length: u64,
    pub archive_entry_path: String,
    pub extracted_executable_sha256: String,
    pub extracted_byte_length: u64,
}

impl Codex0156OfficialArchiveV1 {
    pub fn provenance(&self) -> RuntimeArtifactProvenanceV1 {
        RuntimeArtifactProvenanceV1::OfficialCodexRelease {
            version: CODEX_0156_VERSION.to_string(),
            target_triple: CODEX_0156_TARGET.to_string(),
            archive_name: self.archive_name.clone(),
            archive_url: self.archive_url.clone(),
            archive_sha256: self.archive_sha256.clone(),
            archive_entry_path: self.archive_entry_path.clone(),
            extracted_executable_sha256: self.extracted_executable_sha256.clone(),
        }
    }
}

/// The pair is published in one existing installer-source record, never as two heads.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Codex0156ReleaseBundleV1 {
    pub version: String,
    pub target_triple: String,
    pub main: Codex0156OfficialArchiveV1,
    pub code_mode_host: Codex0156OfficialArchiveV1,
}

impl Codex0156ReleaseBundleV1 {
    /// Exact official Linux pins. These authenticate release bytes, not runtime execution.
    /// ```
    /// let pair = config_projection::Codex0156ReleaseBundleV1::pinned();
    /// assert_eq!(pair.version, "0.156.1");
    /// assert_ne!(pair.main.archive_sha256, pair.code_mode_host.archive_sha256);
    /// assert_eq!(pair.code_mode_host.archive_entry_path, "codex-code-mode-host-x86_64-unknown-linux-musl");
    /// ```
    pub fn pinned() -> Self {
        let archive = |member: &str,
                       archive_sha256: &str,
                       archive_byte_length,
                       extracted_executable_sha256: &str,
                       extracted_byte_length| {
            let archive_name = format!("{member}.tar.gz");
            Codex0156OfficialArchiveV1 {
                archive_url: format!("https://github.com/openai/codex/releases/download/rust-v0.156.1/{archive_name}"),
                archive_name,
                archive_sha256: archive_sha256.to_string(),
                archive_byte_length,
                archive_entry_path: member.to_string(),
                extracted_executable_sha256: extracted_executable_sha256.to_string(),
                extracted_byte_length,
            }
        };
        Self {
            version: CODEX_0156_VERSION.to_string(),
            target_triple: CODEX_0156_TARGET.to_string(),
            main: archive(
                "codex-x86_64-unknown-linux-musl",
                "aff46539a83aff86e3c62c592bce2c50d95391f9df289afaf03a50c01d14533d",
                107380357,
                "0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f",
                284479848,
            ),
            code_mode_host: archive(
                "codex-code-mode-host-x86_64-unknown-linux-musl",
                "a929daa9f6a0bddc00c0c9e6402df117b125acd96f9d554f6c99c32c7e66c608",
                26176187,
                "45ba654b0e145406f3316be4729b886980ce811aa2e3b6ed4e033dd41ab524f1",
                70603664,
            ),
        }
    }
}

/// Exact sibling support object. Main Codex still executes through its held descriptor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Codex0156CodeModeHostV1 {
    pub file: RuntimeSupportFileV1,
    pub owner_uid: u64,
    pub elf_execution_model: E3ElfExecutionModelV1,
    pub provenance: RuntimeArtifactProvenanceV1,
}

impl Codex0156CodeModeHostV1 {
    /// Rejects a helper whose path, root ownership, size, bytes or provenance differs from the stock pin.
    pub fn validate(&self) -> Result<(), crate::ConfigProjectionFailureV1> {
        let pin = Codex0156ReleaseBundleV1::pinned().code_mode_host;
        if self.file.absolute_path != CODEX_0156_HOST_PATH
            || self.file.device_id == 0
            || self.file.inode == 0
            || self.file.mode != 0o755
            || self.owner_uid != 0
            || self.file.byte_length != pin.extracted_byte_length
            || self.file.sha256 != pin.extracted_executable_sha256
            || self.provenance != pin.provenance()
        {
            return Err(crate::ConfigProjectionFailureV1::WrongBinding);
        }
        Ok(())
    }
}

pub(crate) fn validate_support_version_v1(
    support: &crate::E3RuntimeSupportManifestV1,
) -> Result<(), crate::ConfigProjectionFailureV1> {
    match (support.support_policy_version, &support.code_mode_host) {
        (1, None) => Ok(()),
        (2, Some(host)) => host.validate(),
        _ => Err(crate::ConfigProjectionFailureV1::WrongBinding),
    }
}

pub(crate) fn validate_release_source_v1(
    record: &crate::InstallerArtifactSourceRecordV1,
) -> Result<(), crate::ConfigProjectionFailureV1> {
    use crate::{
        ConfigProjectionFailureV1::WrongBinding, InstallerArtifactBuildInputV1,
        InstallerArtifactSourceStreamV1,
    };
    let pins = Codex0156ReleaseBundleV1::pinned();
    if record.source_stream != InstallerArtifactSourceStreamV1::Codex0156OfficialArchives
        || record.build_input
            != InstallerArtifactBuildInputV1::Codex0156OfficialArchives(pins.clone())
        || record.entries.len() != 2
    {
        return Err(WrongBinding);
    }
    for (entry, component, path, pin) in [
        (
            &record.entries[0],
            "codex",
            CODEX_0156_MAIN_PATH,
            &pins.main,
        ),
        (
            &record.entries[1],
            "codex-code-mode-host",
            CODEX_0156_HOST_PATH,
            &pins.code_mode_host,
        ),
    ] {
        if entry.component != component
            || entry.installed_absolute_path != path
            || entry.byte_length != pin.extracted_byte_length
            || entry.sha256 != pin.extracted_executable_sha256
        {
            return Err(WrongBinding);
        }
    }
    let host = record.entries[0]
        .runtime_support
        .code_mode_host
        .as_ref()
        .ok_or(WrongBinding)?;
    host.validate()?;
    let entry = &record.entries[1];
    if host.file.device_id != entry.device_id
        || host.file.inode != entry.inode
        || host.file.mode != entry.mode
        || host.owner_uid != entry.owner_uid
        || host.elf_execution_model != entry.runtime_support.elf_execution_model
        || entry.runtime_support.code_mode_host.is_some()
    {
        return Err(WrongBinding);
    }
    Ok(())
}

pub(crate) fn pinned_loader_source_v1() -> crate::CodexLoaderSourceIdentityV1 {
    crate::CodexLoaderSourceIdentityV1 {
        codex_version: CODEX_0156_VERSION.to_string(),
        upstream_tag: "rust-v0.156.1".to_string(),
        validator_schema_version: 2,
        config_loader_source_sha256:
            "de2afb184ca5e6807ab753f82ba18ba651aec7a8f75bc53722e5aab5b799d5b2".to_string(),
        layer_io_source_sha256: "a090443bc10317e086cd552c29ec4558917bdade0b397abd5229f81f93042022"
            .to_string(),
        loader_model_source_sha256:
            "4d3e0fd6be54861035c902565244ed1239f39f81b5b0cad9641b7e5d075a467d".to_string(),
        exec_source_sha256: "bbacb1efa0741f7e039e48742e1dc4db927c21cd353dc02c0b811293525fd1e1"
            .to_string(),
        cloud_requirements_source_sha256:
            "3cb2bf30b0a2ed4bc766cf2d2172fc2fb6dbd7e2ff36b24281820f831a43cc1a".to_string(),
        auth_storage_source_sha256:
            "2e95a8998687e926675d41b5d941c3c3546a652c03d9680e6c7385e59bf459af".to_string(),
        config_types_source_sha256:
            "8ca8f3d3013f4b69eb2fe6da91ec783914ea3e5c2cc3633d737d43d5a09a40eb".to_string(),
        codex_0156_sources: Some(Codex0156LoaderSourcesV1 {
            upstream_commit: "b412ff32c417f855c2b2d1581b77058eed87c84b".to_string(),
            upstream_tree: "0a0b0bf9acf2f45e9239e7a52ef79f7702aa5799".to_string(),
            files: [
                (
                    "codex-rs/code-mode/src/remote_session.rs".to_string(),
                    "987ff29ed4b3978c482cd2616aeeef75ac1cec561912a49a4220f63efba8d244".to_string(),
                ),
                (
                    "codex-rs/code-mode/src/remote_session/connection.rs".to_string(),
                    "3e7eba38ab29a1b5b0e006da81378f9e96d4c0d630f6da8c46710d13b58a9236".to_string(),
                ),
                (
                    "codex-rs/core/src/config/mod.rs".to_string(),
                    "9f44253d22309f25fa4084f6f15b26dd313f3757e5b133e27cb380498e3d05ba".to_string(),
                ),
                (
                    "codex-rs/core/src/tools/mod.rs".to_string(),
                    "5e89a9b4d122f4c43e599ecf349673a2478264b4b95024bda8660be5c22da0fe".to_string(),
                ),
                (
                    "codex-rs/features/src/lib.rs".to_string(),
                    "d5f1bf1e16c3053df8a739d48e57d743ee76822e532219cee46cff5a9b1d8d78".to_string(),
                ),
                (
                    "codex-rs/install-context/src/lib.rs".to_string(),
                    "b7305f28586f61f7a3cc7d3c53c95dacabca05529b1ba3aa4b5e42d306db2ae5".to_string(),
                ),
                (
                    "codex-rs/models-manager/src/manager.rs".to_string(),
                    "7cce0601fcb3abdf1d944412356ae541d0a1c30014a911ddda61a879b89e8358".to_string(),
                ),
                (
                    "codex-rs/cloud-config/src/bundle_loader.rs".to_string(),
                    "3cb2bf30b0a2ed4bc766cf2d2172fc2fb6dbd7e2ff36b24281820f831a43cc1a".to_string(),
                ),
                (
                    "codex-rs/cloud-config/src/cache.rs".to_string(),
                    "7a63eea1affc82c19c670ea90259ccdae2cc9f2497fb66d1fcae8dc7e79a4eb9".to_string(),
                ),
                (
                    "codex-rs/cloud-config/src/lib.rs".to_string(),
                    "560ebd727ca57c9e0b7d0ae1f80a670b1a4bfeb8579dc9d507629b3503632175".to_string(),
                ),
                (
                    "codex-rs/config/src/config_toml.rs".to_string(),
                    "8ca8f3d3013f4b69eb2fe6da91ec783914ea3e5c2cc3633d737d43d5a09a40eb".to_string(),
                ),
                (
                    "codex-rs/config/src/loader/layer_io.rs".to_string(),
                    "a090443bc10317e086cd552c29ec4558917bdade0b397abd5229f81f93042022".to_string(),
                ),
                (
                    "codex-rs/config/src/loader/local.rs".to_string(),
                    "67ea1e125708d88286cfe2c4787c4e02225a8a90c27b8764fee7135f24570d52".to_string(),
                ),
                (
                    "codex-rs/config/src/loader/managed_requirements.rs".to_string(),
                    "d59e2bd253451c16fa8054c9466445aff1d537a4b4916b1b3a7b67ad554810de".to_string(),
                ),
                (
                    "codex-rs/config/src/loader/mod.rs".to_string(),
                    "de2afb184ca5e6807ab753f82ba18ba651aec7a8f75bc53722e5aab5b799d5b2".to_string(),
                ),
                (
                    "codex-rs/config/src/state.rs".to_string(),
                    "4d3e0fd6be54861035c902565244ed1239f39f81b5b0cad9641b7e5d075a467d".to_string(),
                ),
                (
                    "codex-rs/config/src/types.rs".to_string(),
                    "5eb45c22b58c0b022ef61e3527c90c10080267a165c733c5b5885115b1073727".to_string(),
                ),
                (
                    "codex-rs/exec/src/lib.rs".to_string(),
                    "bbacb1efa0741f7e039e48742e1dc4db927c21cd353dc02c0b811293525fd1e1".to_string(),
                ),
                (
                    "codex-rs/login/src/auth/storage.rs".to_string(),
                    "2e95a8998687e926675d41b5d941c3c3546a652c03d9680e6c7385e59bf459af".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            builtin_catalogue_sha256:
                "1892a933a420e79a30779ef5f1ce5b7dbc145464aa6fb1432106a9e9859981d3".to_string(),
            models_cache_filename: "models_cache.json".to_string(),
            cloud_cache_filename: "cloud-config-bundle-cache.json".to_string(),
        }),
    }
}
