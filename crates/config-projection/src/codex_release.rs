//! Repository-embedded release data. Parsing data never confers release authority.
use crate::{
    ConfigProjectionCodecV1 as Codec, ConfigProjectionFailureV1 as F, E3ElfExecutionModelV1,
    RuntimeArtifactProvenanceV1,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CODEX_STOCK_SIBLING_HOST_PROFILE_V1: &str = "codex-exec-stock-sibling-host-v1";
pub const CODEX_RELEASE_TARGET_V1: &str = "x86_64-unknown-linux-musl";
pub const CODEX_MAIN_PATH_V1: &str = "/var/lib/substrate/world-deps/codex-runtime/bin/codex";
pub const CODEX_HOST_PATH_V1: &str =
    "/var/lib/substrate/world-deps/codex-runtime/bin/codex-code-mode-host";
pub const CODEX_NATIVE_DOMAIN_V1: &str = "substrate.e3.codex-native-config-projection.v1";
pub const CODEX_LOADER_DOMAIN_V1: &str = "substrate.e3.codex-loader-inputs.v1";
const EMBEDDED_CATALOG: &[u8] = include_bytes!("../releases/codex.json");

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseCatalogV1 {
    pub schema_version: u32,
    pub selected: BTreeMap<String, String>,
    pub releases: Vec<CodexReleaseRecordV1>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseRecordV1 {
    pub schema_version: u32,
    pub version: String,
    pub target: String,
    pub behavior_profile: String,
    pub source: CodexReleaseSourceV1,
    pub artifacts: Vec<CodexReleaseArtifactV1>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseSourceV1 {
    pub tag: String,
    pub commit: String,
    pub tree: String,
    pub files: Vec<CodexReleaseSourceFileV1>,
    pub builtin_catalogue: CodexReleaseSourceDigestV1,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseSourceFileV1 {
    pub slot: String,
    pub path: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseSourceDigestV1 {
    pub path: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodexReleaseArtifactRoleV1 {
    Main,
    CodeModeHost,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseArtifactV1 {
    pub role: CodexReleaseArtifactRoleV1,
    pub archive: CodexReleaseArchiveV1,
    pub extracted: CodexReleaseExtractedV1,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseArchiveV1 {
    pub name: String,
    pub url: String,
    pub member: String,
    pub byte_length: u64,
    pub sha256: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseExtractedV1 {
    pub byte_length: u64,
    pub sha256: String,
    pub elf: E3ElfExecutionModelV1,
}
/// Stable per-record identity, unaffected by catalog selection or unrelated additions.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseBindingV1 {
    pub manifest_sha256: String,
    pub behavior_profile: String,
    pub version: String,
    pub target: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexReleaseInputV1 {
    pub binding: CodexReleaseBindingV1,
    pub manifest: CodexReleaseRecordV1,
}
/// Only compiled repository data can construct this authority-bearing selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedCodexReleaseV1(CodexReleaseInputV1);

const SOURCE_SLOTS: [&str; 19] = [
    "remote_session",
    "remote_session_connection",
    "core_config",
    "tool_mode",
    "feature_defaults",
    "install_context",
    "models_manager",
    "cloud_bundle_loader",
    "cloud_cache",
    "cloud_config",
    "config_schema",
    "layer_io",
    "local_loader",
    "managed_requirements",
    "config_loader",
    "config_state",
    "config_types",
    "exec_cli",
    "auth_storage",
];
fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn release_version(value: &str) -> bool {
    let (numbers, suffix) = value
        .split_once('-')
        .map_or((value, None), |(n, s)| (n, Some(s)));
    numbers.split('.').count() == 3
        && numbers.split('.').all(|n| {
            !n.is_empty()
                && n.bytes().all(|b| b.is_ascii_digit())
                && (n == "0" || !n.starts_with('0'))
        })
        && suffix.is_none_or(|s| {
            !s.is_empty()
                && s.split('.').all(|v| {
                    !v.is_empty() && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                })
        })
}
fn safe_path(value: &str) -> bool {
    !value.is_empty()
        && value.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
fn safe_basename(value: &str) -> bool {
    value.len() <= 255 && !value.contains('/') && safe_path(value)
}

impl CodexReleaseRecordV1 {
    /// Validates the closed profile's data shape; this does not admit external data.
    pub fn validate_structure(&self) -> Result<(), F> {
        if self.schema_version != 1
            || self.target != CODEX_RELEASE_TARGET_V1
            || self.behavior_profile != CODEX_STOCK_SIBLING_HOST_PROFILE_V1
            || self.version.len() > 64
            || !release_version(&self.version)
            || self.source.tag != format!("rust-v{}", self.version)
            || !hex(&self.source.commit, 40)
            || !hex(&self.source.tree, 40)
            || self.artifacts.len() != 2
            || self.source.files.len() != SOURCE_SLOTS.len()
            || !safe_path(&self.source.builtin_catalogue.path)
            || !hex(&self.source.builtin_catalogue.sha256, 64)
        {
            return Err(F::UnsupportedRuntimeVersion);
        }
        let mut paths = BTreeSet::new();
        for (file, slot) in self.source.files.iter().zip(SOURCE_SLOTS) {
            if file.slot != slot
                || !safe_path(&file.path)
                || !paths.insert(&file.path)
                || !hex(&file.sha256, 64)
            {
                return Err(F::WrongBinding);
            }
        }
        let mut archives = BTreeSet::new();
        let mut members = BTreeSet::new();
        for (artifact, role) in self.artifacts.iter().zip([
            CodexReleaseArtifactRoleV1::Main,
            CodexReleaseArtifactRoleV1::CodeModeHost,
        ]) {
            let archive = &artifact.archive;
            if artifact.role != role
                || !safe_basename(&archive.member)
                || !safe_basename(&archive.name)
                || archive
                    .name
                    .strip_suffix(".tar.gz")
                    .is_none_or(|stem| stem.is_empty())
                || !archives.insert(&archive.name)
                || !members.insert(&archive.member)
                || archive.url
                    != format!(
                        "https://github.com/openai/codex/releases/download/{}/{}",
                        self.source.tag, archive.name
                    )
                || artifact.archive.byte_length == 0
                || artifact.extracted.byte_length == 0
                || !hex(&artifact.archive.sha256, 64)
                || !hex(&artifact.extracted.sha256, 64)
            {
                return Err(F::WrongBinding);
            }
            if let E3ElfExecutionModelV1::StaticPie(pie) = &artifact.extracted.elf {
                if pie.dynamic_segment_byte_length == 0
                    || pie.rela_entry_byte_length != 24
                    || pie.rela_byte_length == 0
                    || pie.relative_relocation_count.checked_mul(24) != Some(pie.rela_byte_length)
                    || !hex(&pie.ordered_dynamic_entries_sha256, 64)
                {
                    return Err(F::WrongBinding);
                }
            }
        }
        Ok(())
    }
    pub fn binding(&self) -> Result<CodexReleaseBindingV1, F> {
        self.validate_structure()?;
        Ok(CodexReleaseBindingV1 {
            manifest_sha256: Codec::domain_sha256("substrate.codex.release-manifest.v1", self)?,
            behavior_profile: self.behavior_profile.clone(),
            version: self.version.clone(),
            target: self.target.clone(),
        })
    }
}
impl CodexReleaseCatalogV1 {
    /// Parse strict canonical repository data. An external parsed catalog is not authority.
    pub fn parse(bytes: &[u8]) -> Result<Self, F> {
        let result: Self = Codec::decode_canonical_json(bytes)?;
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), F> {
        if self.schema_version != 1 || self.selected.is_empty() || self.releases.is_empty() {
            return Err(F::Malformed);
        }
        let mut keys = BTreeSet::new();
        let mut digests = BTreeSet::new();
        for release in &self.releases {
            let binding = release.binding()?;
            if !keys.insert((&release.target, &release.version))
                || !digests.insert(binding.manifest_sha256)
            {
                return Err(F::WrongBinding);
            }
        }
        for (target, version) in &self.selected {
            if target != CODEX_RELEASE_TARGET_V1 || !keys.contains(&(target, version)) {
                return Err(F::WrongBinding);
            }
        }
        Ok(())
    }
    pub fn embedded() -> Result<Self, F> {
        Self::parse(EMBEDDED_CATALOG)
    }
}
impl TrustedCodexReleaseV1 {
    /// Selects the compiled target, with no latest or environment override.
    /// ```
    /// let release = config_projection::TrustedCodexReleaseV1::selected("x86_64-unknown-linux-musl").unwrap();
    /// assert_eq!(release.binding().behavior_profile, "codex-exec-stock-sibling-host-v1");
    /// assert_ne!(release.main().extracted.sha256, release.host().extracted.sha256);
    /// ```
    pub fn selected(target: &str) -> Result<Self, F> {
        let catalog = CodexReleaseCatalogV1::embedded()?;
        let version = catalog
            .selected
            .get(target)
            .ok_or(F::UnsupportedRuntimeVersion)?;
        let record = catalog
            .releases
            .iter()
            .find(|r| r.target == target && &r.version == version)
            .ok_or(F::WrongBinding)?
            .clone();
        Self::from_record(record)
    }
    pub fn resolve(binding: &CodexReleaseBindingV1) -> Result<Self, F> {
        let catalog = CodexReleaseCatalogV1::embedded()?;
        let record = catalog
            .releases
            .into_iter()
            .find(|r| r.binding().as_ref() == Ok(binding))
            .ok_or(F::UnsupportedRuntimeVersion)?;
        Self::from_record(record)
    }
    fn from_record(manifest: CodexReleaseRecordV1) -> Result<Self, F> {
        let binding = manifest.binding()?;
        Ok(Self(CodexReleaseInputV1 { binding, manifest }))
    }
    pub fn authenticate(input: &CodexReleaseInputV1) -> Result<Self, F> {
        let selected = Self::resolve(&input.binding)?;
        if selected.0 != *input {
            return Err(F::WrongBinding);
        }
        Ok(selected)
    }
    pub fn binding(&self) -> &CodexReleaseBindingV1 {
        &self.0.binding
    }
    pub fn input(&self) -> &CodexReleaseInputV1 {
        &self.0
    }
    pub fn record(&self) -> &CodexReleaseRecordV1 {
        &self.0.manifest
    }
    pub fn main(&self) -> &CodexReleaseArtifactV1 {
        &self.0.manifest.artifacts[0]
    }
    pub fn host(&self) -> &CodexReleaseArtifactV1 {
        &self.0.manifest.artifacts[1]
    }
    pub fn provenance(&self, component: CodexReleaseArtifactRoleV1) -> RuntimeArtifactProvenanceV1 {
        RuntimeArtifactProvenanceV1::TrustedCodexRelease {
            binding: self.binding().clone(),
            component,
        }
    }
    pub fn validate_helper(&self, host: &crate::Codex0156CodeModeHostV1) -> Result<(), F> {
        if host.file.absolute_path != CODEX_HOST_PATH_V1
            || host.file.device_id == 0
            || host.file.inode == 0
            || host.file.mode != 0o755
            || host.owner_uid != 0
            || host.file.byte_length != self.host().extracted.byte_length
            || host.file.sha256 != self.host().extracted.sha256
            || host.elf_execution_model != self.host().extracted.elf
            || host.provenance != self.provenance(CodexReleaseArtifactRoleV1::CodeModeHost)
        {
            return Err(F::WrongBinding);
        }
        Ok(())
    }
    pub fn validate_main(&self, artifact: &crate::DescriptorPinnedArtifactV1) -> Result<(), F> {
        if artifact.role != crate::ConfigProjectionArtifactRoleV1::CodexMain
            || artifact.configured_absolute_path != CODEX_MAIN_PATH_V1
            || artifact.provenance != self.provenance(CodexReleaseArtifactRoleV1::Main)
            || artifact.byte_length != self.main().extracted.byte_length
            || artifact.sha256 != self.main().extracted.sha256
            || artifact.runtime_support.support_policy_version != 3
            || artifact.runtime_support.elf_execution_model != self.main().extracted.elf
        {
            return Err(F::WrongBinding);
        }
        self.validate_helper(
            artifact
                .runtime_support
                .code_mode_host
                .as_ref()
                .ok_or(F::WrongBinding)?,
        )
    }
    pub fn loader_source(&self) -> crate::CodexLoaderSourceIdentityV1 {
        let source = &self.record().source;
        let hash = |slot: &str| {
            source
                .files
                .iter()
                .find(|f| f.slot == slot)
                .expect("validated profile source slot")
                .sha256
                .clone()
        };
        crate::CodexLoaderSourceIdentityV1 {
            release_binding: Some(self.binding().clone()),
            codex_0156_sources: None,
            codex_version: self.record().version.clone(),
            upstream_tag: source.tag.clone(),
            validator_schema_version: 3,
            config_loader_source_sha256: hash("config_loader"),
            layer_io_source_sha256: hash("layer_io"),
            loader_model_source_sha256: hash("config_state"),
            exec_source_sha256: hash("exec_cli"),
            cloud_requirements_source_sha256: hash("cloud_bundle_loader"),
            auth_storage_source_sha256: hash("auth_storage"),
            config_types_source_sha256: hash("config_schema"),
        }
    }
}
pub(crate) fn validate_generic_source(
    record: &crate::InstallerArtifactSourceRecordV1,
) -> Result<(), F> {
    use crate::{InstallerArtifactBuildInputV1 as B, InstallerArtifactSourceStreamV1 as S};
    let B::CodexOfficialRelease(input) = &record.build_input else {
        return Err(F::WrongBinding);
    };
    let release = TrustedCodexReleaseV1::authenticate(input)?;
    if record.source_stream != S::CodexOfficialRelease || record.entries.len() != 2 {
        return Err(F::WrongBinding);
    }
    for (entry, (artifact, component, path)) in record.entries.iter().zip([
        (release.main(), "codex", CODEX_MAIN_PATH_V1),
        (release.host(), "codex-code-mode-host", CODEX_HOST_PATH_V1),
    ]) {
        if entry.component != component
            || entry.installed_absolute_path != path
            || entry.byte_length != artifact.extracted.byte_length
            || entry.sha256 != artifact.extracted.sha256
            || entry.runtime_support.elf_execution_model != artifact.extracted.elf
        {
            return Err(F::WrongBinding);
        }
    }
    let host = record.entries[0]
        .runtime_support
        .code_mode_host
        .as_ref()
        .ok_or(F::WrongBinding)?;
    release.validate_helper(host)?;
    let entry = &record.entries[1];
    if record.entries[0].runtime_support.support_policy_version != 3
        || entry.runtime_support.support_policy_version != 1
        || entry.runtime_support.code_mode_host.is_some()
        || host.file.device_id != entry.device_id
        || host.file.inode != entry.inode
        || host.file.mode != entry.mode
        || host.owner_uid != entry.owner_uid
        || host.elf_execution_model != entry.runtime_support.elf_execution_model
    {
        return Err(F::WrongBinding);
    }
    Ok(())
}
pub fn validate_codex_support_v1(support: &crate::E3RuntimeSupportManifestV1) -> Result<(), F> {
    if support.support_policy_version == 3 {
        let host = support.code_mode_host.as_ref().ok_or(F::WrongBinding)?;
        let RuntimeArtifactProvenanceV1::TrustedCodexRelease {
            binding,
            component: CodexReleaseArtifactRoleV1::CodeModeHost,
        } = &host.provenance
        else {
            return Err(F::WrongBinding);
        };
        TrustedCodexReleaseV1::resolve(binding)?.validate_helper(host)
    } else {
        crate::codex_0156::validate_support_version_v1(support)
    }
}
pub fn codex_native_hash_domain_v1(renderer: &crate::NativeRendererIdentityV1) -> &'static str {
    if renderer.release_binding.is_some() {
        CODEX_NATIVE_DOMAIN_V1
    } else if renderer.codex_version == "0.156.1" {
        "substrate.e3.codex-0.156-native-config-projection.v1"
    } else {
        "substrate.e3.native-config-projection.v1"
    }
}
/// Authenticates one component under its own compiled release record.
pub fn codex_release_from_provenance_v1(
    provenance: &RuntimeArtifactProvenanceV1,
    component: CodexReleaseArtifactRoleV1,
) -> Result<TrustedCodexReleaseV1, F> {
    let RuntimeArtifactProvenanceV1::TrustedCodexRelease {
        binding,
        component: actual,
    } = provenance
    else {
        return Err(F::WrongBinding);
    };
    if *actual != component {
        return Err(F::WrongBinding);
    }
    TrustedCodexReleaseV1::resolve(binding)
}
pub fn validate_codex_release_component_v1(
    provenance: &RuntimeArtifactProvenanceV1,
    component: CodexReleaseArtifactRoleV1,
    sha256: &str,
    length: u64,
    elf: &E3ElfExecutionModelV1,
) -> Result<(), F> {
    let release = codex_release_from_provenance_v1(provenance, component.clone())?;
    let artifact = match component {
        CodexReleaseArtifactRoleV1::Main => release.main(),
        CodexReleaseArtifactRoleV1::CodeModeHost => release.host(),
    };
    if artifact.extracted.sha256 != sha256
        || artifact.extracted.byte_length != length
        || artifact.extracted.elf != *elf
    {
        return Err(F::WrongBinding);
    }
    Ok(())
}
/// Validates a checked stock sibling using its immutable provenance authority.
pub fn validate_codex_host_identity_v1(host: &crate::Codex0156CodeModeHostV1) -> Result<(), F> {
    match &host.provenance {
        RuntimeArtifactProvenanceV1::TrustedCodexRelease {
            binding,
            component: CodexReleaseArtifactRoleV1::CodeModeHost,
        } => TrustedCodexReleaseV1::resolve(binding)?.validate_helper(host),
        _ => host.validate(),
    }
}
