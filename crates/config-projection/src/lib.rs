//! Durable, accepted-home authority for E3 agent configuration projections.

mod codec;
mod codex_0125;
mod linux_artifacts;
mod registry;
mod service;

pub use codec::ConfigProjectionCodecV1;
pub use codex_0125::*;
pub use linux_artifacts::*;
pub use registry::*;
pub use service::*;

pub use transport_api_types::{
    ConfigProjectionAuthoringInputRefV1, ConfigProjectionRefV1,
    DispatchPolicyCommitmentRefCarrierV1, E2MemberLaunchKindV1, InWorldGatewayRefV1,
    ManagedGatewayActivationIntentRefV1, PolicyRefV1, RetainedTurnPolicyCommitmentSubjectV1,
    WorldBindingRefV1,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Timestamp(pub String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDirectoryV1 {
    pub physical_path: String,
    pub physical_identity: DirectoryPhysicalIdentityV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum DirectoryPhysicalIdentityV1 {
    Linux { device_id: u64, inode: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionIdentityV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub accepted_home: CanonicalDirectoryV1,
    pub workspace_root: CanonicalDirectoryV1,
    pub orchestration_session_id: String,
    pub retained_participant_id: String,
    pub bootstrap_run_id: String,
    pub backend_id: String,
    pub runtime_family: String,
    pub world_id: String,
    pub world_generation: u64,
    pub immutable_launch_cap: ConfigProjectionE2CapLinkV1,
    pub runtime_artifacts: ConfigProjectionRuntimeArtifactsV1,
    pub identity_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionE2CapLinkV1 {
    pub e2_activation_id: String,
    pub e2_launch_kind: E2MemberLaunchKindV1,
    pub commitment_ref: DispatchPolicyCommitmentRefCarrierV1,
    pub commitment_subject: ConfigProjectionE2SubjectV1,
    pub immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    pub immutable_worker_cap_created_revision: u64,
    pub immutable_worker_cap_application_revision: u64,
    pub policy_snapshot_ref: PolicyRefV1,
    pub policy_snapshot_hash: String,
    pub policy_snapshot_revision: String,
    pub request_id: String,
    pub idempotency_key: String,
    pub caller_participant_id: String,
    pub caller_backend_id: String,
    pub target_backend_id: String,
    pub target_world: WorldBindingRefV1,
    pub registry_publication_revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ConfigProjectionE2SubjectV1 {
    RetainedWorkerLaunch {
        retained_participant_id: String,
        bootstrap_run_id: String,
    },
    RetainedWorkerFork {
        source_participant_id: String,
        child_participant_id: String,
        bootstrap_run_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionRuntimeArtifactsV1 {
    pub codex: DescriptorPinnedArtifactV1,
    pub world_entry_wrapper: DescriptorPinnedArtifactV1,
    pub managed_gateway: DescriptorPinnedArtifactV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptorPinnedArtifactV1 {
    pub role: ConfigProjectionArtifactRoleV1,
    pub configured_absolute_path: String,
    pub device_id: u64,
    pub inode: u64,
    pub file_type: String,
    pub mode: u32,
    pub owner_uid: u64,
    pub byte_length: u64,
    pub sha256: String,
    pub authority_ref: RuntimeArtifactAuthorityRefV1,
    pub provenance: RuntimeArtifactProvenanceV1,
    pub runtime_support: E3RuntimeSupportManifestV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3RuntimeSupportManifestV1 {
    pub schema_version: u32,
    pub support_policy_version: u32,
    pub elf_execution_model: E3ElfExecutionModelV1,
    pub elf_interpreter: Option<RuntimeSupportFileV1>,
    pub dynamic_loader_cache: Option<RuntimeSupportFileV1>,
    pub ordered_elf_dependencies: Vec<RuntimeSupportFileV1>,
    pub ordered_present_common_files: Vec<RuntimeSupportFileV1>,
    pub system_config_mount_target: RuntimeSupportDirectoryV1,
    pub manifest_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum E3ElfExecutionModelV1 {
    StaticExec,
    StaticPie(E3StaticPieRelocationMetadataV1),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3StaticPieRelocationMetadataV1 {
    pub dynamic_segment_file_offset: u64,
    pub dynamic_segment_byte_length: u64,
    pub rela_virtual_address: u64,
    pub rela_byte_length: u64,
    pub rela_entry_byte_length: u64,
    pub relative_relocation_count: u64,
    pub ordered_dynamic_entries_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSupportFileV1 {
    pub absolute_path: String,
    pub device_id: u64,
    pub inode: u64,
    pub mode: u32,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSupportDirectoryV1 {
    pub absolute_path: String,
    pub device_id: u64,
    pub inode: u64,
    pub mode: u32,
    pub owner_uid: u64,
    pub owner_gid: u64,
    pub ordered_entry_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ConfigProjectionArtifactRoleV1 {
    Codex0125,
    WorldEntryWrapper,
    ManagedGateway,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeArtifactAuthorityRefV1 {
    pub authority_store_id: String,
    pub manifest_id: String,
    pub manifest_revision: u64,
    pub manifest_entry_id: String,
    pub manifest_hash: String,
    pub entry_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum RuntimeArtifactProvenanceV1 {
    OfficialCodexRelease {
        version: String,
        target_triple: String,
        archive_name: String,
        archive_url: String,
        archive_sha256: String,
        archive_entry_path: String,
        extracted_executable_sha256: String,
    },
    SubstrateSourceBuild {
        component: String,
        source_commit: String,
        source_tree: String,
        cargo_lock_sha256: String,
        target_triple: String,
        profile: String,
        executable_sha256: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveSubstrateConfigSourceV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub accepted_home: CanonicalDirectoryV1,
    pub workspace_root: CanonicalDirectoryV1,
    pub values: E3EffectiveConfigInputV1,
    pub ordered_explain_origins: Vec<E3ConfigExplainOriginV1>,
    pub source_revision: String,
    pub source_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3EffectiveConfigInputV1 {
    pub llm_enabled: bool,
    pub agents_enabled: bool,
    pub world_enabled: bool,
    pub default_execution_scope: String,
    pub default_cli_mode: String,
    pub managed_gateway_enabled: bool,
    pub managed_gateway_mode: String,
    pub default_backend_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3ConfigExplainOriginV1 {
    pub key: String,
    pub source_kind: E3ConfigExplainOriginKindV1,
    pub source_location: Option<E3ConfigExplainFileV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum E3ConfigExplainOriginKindV1 {
    Default,
    GlobalPatch,
    WorkspacePatch,
    OverrideEnv,
    CliFlag,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3ConfigExplainFileV1 {
    pub source_root: CanonicalDirectoryV1,
    pub source_relative_path: String,
    pub source_bytes_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentInventorySourceMaterialV1 {
    pub inventory_scope: String,
    pub accepted_root: CanonicalDirectoryV1,
    pub relative_path: String,
    pub file_device_id: u64,
    pub file_inode: u64,
    pub byte_length: u64,
    pub raw_bytes_sha256: String,
    pub source_revision: String,
    pub source_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustedRuntimeArtifactManifestV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub manifest_id: String,
    pub revision: u64,
    pub entries: Vec<RuntimeArtifactManifestEntryV1>,
    pub created_at: Timestamp,
    pub manifest_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeArtifactManifestEntryV1 {
    pub manifest_entry_id: String,
    pub authority_role: RuntimeArtifactAuthorityRoleV1,
    pub configured_absolute_path: String,
    pub device_id: u64,
    pub inode: u64,
    pub mode: u32,
    pub owner_uid: u64,
    pub byte_length: u64,
    pub sha256: String,
    pub installer_source_ref: InstallerArtifactSourceRefV1,
    pub provenance: RuntimeArtifactProvenanceV1,
    pub runtime_support: E3RuntimeSupportManifestV1,
    pub entry_hash: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RuntimeArtifactAuthorityRoleV1 {
    Codex0125,
    ManagedGateway,
    WorldEntryWrapper,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallerArtifactSourceRefV1 {
    pub source_store_id: String,
    pub source_record_id: String,
    pub revision: u64,
    pub record_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallerArtifactSourceStoreV1 {
    pub schema_version: u32,
    pub source_store_id: String,
    pub root: CanonicalDirectoryV1,
    pub created_at: Timestamp,
    pub store_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallerArtifactSourceRecordV1 {
    pub schema_version: u32,
    pub source_store_id: String,
    pub source_record_id: String,
    pub source_stream: InstallerArtifactSourceStreamV1,
    pub revision: u64,
    pub predecessor_ref: Option<InstallerArtifactSourceRefV1>,
    pub build_input: InstallerArtifactBuildInputV1,
    pub entries: Vec<InstallerArtifactSourceEntryV1>,
    pub created_at: Timestamp,
    pub record_hash: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum InstallerArtifactSourceStreamV1 {
    SubstrateSourceBuild,
    Codex0125OfficialArchive,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InstallerArtifactBuildInputV1 {
    SubstrateSourceBuild {
        source_commit: String,
        source_tree: String,
        cargo_lock_sha256: String,
        rustc_version: String,
        target_triple: String,
        profile: String,
    },
    CodexOfficialArchive {
        version: String,
        target_triple: String,
        archive_name: String,
        archive_url: String,
        archive_sha256: String,
        archive_entry_path: String,
        extracted_executable_sha256: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallerArtifactSourceEntryV1 {
    pub component: String,
    pub installed_absolute_path: String,
    pub device_id: u64,
    pub inode: u64,
    pub file_type: String,
    pub mode: u32,
    pub owner_uid: u64,
    pub byte_length: u64,
    pub sha256: String,
    pub runtime_support: E3RuntimeSupportManifestV1,
    pub entry_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallerArtifactSourceHeadV1 {
    pub schema_version: u32,
    pub source_store_id: String,
    pub source_stream: InstallerArtifactSourceStreamV1,
    pub head_ref: InstallerArtifactSourceRefV1,
    pub head_revision: u64,
    pub predecessor_head_hash: Option<String>,
    pub updated_at: Timestamp,
    pub head_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalAgentConfigProjectionV1 {
    pub projection_hash: String,
    pub sources: Vec<LogicalConfigSourceRefV1>,
    pub logical_agent_id: String,
    pub placement: String,
    pub realized_agent_id: String,
    pub backend_id: String,
    pub kind: String,
    pub protocol: String,
    pub execution_scope: String,
    pub cli_mode: String,
    pub runtime_family: String,
    pub capabilities: Vec<String>,
    pub requested_model: String,
    pub requested_mcp_servers: Vec<LogicalMcpServerV1>,
    pub requested_features: Vec<LogicalFeatureV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum LogicalConfigSourceRefV1 {
    EffectiveSubstrateConfig {
        authority_store_id: String,
        source_revision: String,
        source_hash: String,
    },
    AgentInventory {
        authority_store_id: String,
        inventory_scope: String,
        source_revision: String,
        source_hash: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalMcpServerV1 {
    pub server_id: String,
    pub transport: LogicalMcpTransportV1,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum LogicalMcpTransportV1 {
    Stdio {
        command: String,
        args: Vec<String>,
        nonsecret_env: Vec<NamedValueV1>,
    },
    StreamableHttp {
        url: String,
        nonsecret_headers: Vec<NamedValueV1>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalFeatureV1 {
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NamedValueV1 {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveAgentConfigProjectionV1 {
    pub projection_hash: String,
    pub logical_projection_hash: String,
    pub accepted_policy: ConfigProjectionE2CapLinkV1,
    pub capabilities: Vec<String>,
    pub model: String,
    pub provider: EffectiveManagedProviderV1,
    pub mcp_servers: Vec<EffectiveMcpServerV1>,
    pub features: Vec<LogicalFeatureV1>,
    pub environment: EffectiveEnvironmentV1,
    pub workspace_overlay: WorkspaceOverlayPostureV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveManagedProviderV1 {
    pub provider_id: String,
    pub wire_api: String,
    pub requires_openai_auth: bool,
    pub supports_websockets: bool,
    pub gateway_intent_ref: ManagedGatewayActivationIntentRefV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveMcpServerV1 {
    pub server_id: String,
    pub transport: LogicalMcpTransportV1,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveEnvironmentV1 {
    pub inherited_names: Vec<String>,
    pub set: Vec<NamedValueV1>,
    pub remove: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum WorkspaceOverlayPostureV1 {
    Disabled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeAgentConfigProjectionV1 {
    pub projection_hash: String,
    pub renderer: NativeRendererIdentityV1,
    pub root: NativeProjectionRootV1,
    pub files: Vec<NativeProjectedFileV1>,
    pub directories: Vec<NativeProjectedDirectoryV1>,
    pub environment: EffectiveEnvironmentV1,
    pub invocation: CodexNativeInvocationV1,
    pub cwd: CanonicalDirectoryV1,
    pub ambient_closure: CodexAmbientConfigClosureV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRendererIdentityV1 {
    pub renderer_id: String,
    pub renderer_schema_version: u32,
    pub codex_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjectionRootV1 {
    pub root_id: String,
    pub authority_relative_path: String,
    pub guest_absolute_path: String,
    pub owner_uid: u64,
    pub owner_gid: u64,
    pub directory_mode: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjectedDirectoryV1 {
    pub relative_path: String,
    pub mode: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjectedFileV1 {
    pub role: NativeProjectedFileRoleV1,
    pub relative_path: String,
    pub mode: u32,
    pub bytes_base64: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum NativeProjectedFileRoleV1 {
    CodexConfigToml,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexNativeInvocationV1 {
    pub wrapper_argv: Vec<String>,
    pub initial_codex_argv: Vec<String>,
    pub resume_codex_argv_prefix: Vec<String>,
    pub prompt_delivery: String,
    pub output_last_message_directory: String,
    pub output_last_message_name_domain: String,
    pub forbidden_arguments: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexAmbientConfigClosureV1 {
    pub loader_source: CodexLoaderSourceIdentityV1,
    pub allowed_enabled_layers: Vec<String>,
    pub inputs: Vec<CodexLoaderInputAttestationV1>,
    pub forbidden_cli_overrides: Vec<String>,
    pub validated_loader_input_fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexLoaderSourceIdentityV1 {
    pub codex_version: String,
    pub upstream_tag: String,
    pub config_loader_source_sha256: String,
    pub layer_io_source_sha256: String,
    pub loader_model_source_sha256: String,
    pub exec_source_sha256: String,
    pub cloud_requirements_source_sha256: String,
    pub auth_storage_source_sha256: String,
    pub config_types_source_sha256: String,
    pub validator_schema_version: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexLoaderInputAttestationV1 {
    pub layer: String,
    pub locator: String,
    pub disposition: CodexLoaderInputDispositionV1,
    pub directory: CanonicalDirectoryV1,
    pub relative_path: String,
    pub device_id: Option<u64>,
    pub inode: Option<u64>,
    pub byte_length: Option<u64>,
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum CodexLoaderInputDispositionV1 {
    Projected,
    ApprovedMandatory,
    ProvenAbsent,
    DisabledByTrust,
    DisabledByEphemeralCredentialStoreAndProvenAbsent,
    DisabledByEmptyMcpSetAndProvenAbsent,
    DisabledByNoEphemeralAuthAndProvenAbsent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CodexSetupReadyAttestationV1 {
    pub schema_version: u32,
    pub series_id: String,
    pub fence_id: String,
    pub wrapper_pid: u32,
    pub wrapper_pid_start_time_ticks: u64,
    pub mount_namespace_inode: u64,
    pub masked_system_directory: CanonicalDirectoryV1,
    pub native_root: CanonicalDirectoryV1,
    pub codex_launch_plan_hash: String,
    pub validated_loader_input_fingerprint: String,
    pub child_security: E3ChildSecurityAttestationV1,
    pub attestation_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3WorldFsEnforcementInputV1 {
    pub schema_version: u32,
    pub child_role: E3IsolatedChildRoleV1,
    pub projection_identity_hash: String,
    pub policy_authority: E3PolicyAuthoritySourceV1,
    pub immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    pub policy_snapshot_bytes_base64: String,
    pub policy_snapshot_byte_length: u64,
    pub policy_snapshot_ref: PolicyRefV1,
    pub policy_snapshot_hash: String,
    pub policy_snapshot_revision: String,
    pub expected_process_cgroup: CanonicalCgroupIdentityV1,
    pub kernel_boot_id: String,
    pub user_namespace_requirement: E3UserNamespaceRequirementV1,
    pub target_uid: u64,
    pub target_gid: u64,
    pub immutable_config_source: Option<CanonicalDirectoryV1>,
    pub private_realization: Option<CanonicalDirectoryV1>,
    pub codex_launch_plan_hash: Option<String>,
    pub executable_artifact: DescriptorPinnedArtifactV1,
    pub denied_control_probe_targets: Vec<E3DeniedControlProbeTargetV1>,
    pub support_policy_version: u32,
    pub enforcement_input_hash: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum E3IsolatedChildRoleV1 {
    Codex,
    ManagedGateway,
    ManagedGatewayReadinessProbe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3LinuxIdMapExtentV1 {
    pub inside_id: u64,
    pub outside_id: u64,
    pub length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3UserNamespaceRequirementV1 {
    pub trusted_service_uid: u64,
    pub parent_namespace_device_id: u64,
    pub parent_namespace_inode: u64,
    pub uid_map: E3LinuxIdMapExtentV1,
    pub gid_map: E3LinuxIdMapExtentV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3UserNamespaceAttestationV1 {
    pub namespace_device_id: u64,
    pub namespace_inode: u64,
    pub owner_uid: u64,
    pub parent_namespace_device_id: u64,
    pub parent_namespace_inode: u64,
    pub uid_map: E3LinuxIdMapExtentV1,
    pub gid_map: E3LinuxIdMapExtentV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum E3PolicyAuthoritySourceV1 {
    InitialLaunch {
        e2_activation_id: String,
        commitment_ref: DispatchPolicyCommitmentRefCarrierV1,
    },
    RetainedTurn {
        subject: RetainedTurnPolicyCommitmentSubjectV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3DeniedControlProbeTargetV1 {
    pub binding: E3DeniedControlProbeTargetBindingV1,
    pub target_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum E3DeniedControlProbeTargetBindingV1 {
    AcceptedHomeRegistry {
        directory: CanonicalDirectoryV1,
    },
    SiblingNativeRealization {
        directory: CanonicalDirectoryV1,
    },
    CgroupControl {
        cgroup: CanonicalCgroupIdentityV1,
        control_file: String,
    },
    NftablesControl {
        network_namespace_inode: u64,
    },
    WorldServiceState {
        directory: CanonicalDirectoryV1,
    },
    OtherRolePrivateRoot {
        directory: CanonicalDirectoryV1,
    },
    OtherRoleProcessState {
        pid: u32,
        pid_start_time_ticks: u64,
        procfs_mount_device_id: u64,
        procfs_mount_inode: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3DerivedWorldFsEnforcementPlanV1 {
    pub e2_plan_hash: String,
    pub e2_discover_paths: Vec<String>,
    pub e2_execute_paths: Vec<String>,
    pub e2_read_paths: Vec<String>,
    pub e2_write_paths: Vec<String>,
    pub e3_support_discover_paths: Vec<String>,
    pub e3_support_execute_paths: Vec<String>,
    pub e3_support_read_paths: Vec<String>,
    pub e3_support_write_paths: Vec<String>,
    pub derived_support_ruleset_hash: String,
    pub role_narrowing_ruleset_hash: String,
    pub effective_landlock_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3ChildSecurityAttestationV1 {
    pub schema_version: u32,
    pub child_role: E3IsolatedChildRoleV1,
    pub projection_identity_hash: String,
    pub pid: u32,
    pub pid_start_time_ticks: u64,
    pub real_uid: u64,
    pub effective_uid: u64,
    pub saved_uid: u64,
    pub filesystem_uid: u64,
    pub real_gid: u64,
    pub effective_gid: u64,
    pub saved_gid: u64,
    pub filesystem_gid: u64,
    pub supplementary_group_count: u32,
    pub cap_inheritable: String,
    pub cap_permitted: String,
    pub cap_effective: String,
    pub cap_bounding: String,
    pub cap_ambient: String,
    pub cap_last_cap: u32,
    pub no_new_privs: bool,
    pub dumpable: u32,
    pub tracer_pid: u32,
    pub kernel_boot_id: String,
    pub user_namespace: E3UserNamespaceAttestationV1,
    pub seccomp_mode: u32,
    pub landlock_abi: u32,
    pub e2_enforcement_plan_hash: String,
    pub derived_support_ruleset_hash: String,
    pub role_narrowing_ruleset_hash: String,
    pub effective_landlock_hash: String,
    pub policy_snapshot_ref: PolicyRefV1,
    pub policy_snapshot_hash: String,
    pub policy_snapshot_revision: String,
    pub enforcement_input_hash: String,
    pub denied_control_probe_hash: String,
    pub attestation_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3DeniedControlProbeV1 {
    pub target: E3DeniedControlProbeTargetV1,
    pub operation: String,
    pub result_errno: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3GatewaySecretReadyAttestationV1 {
    pub schema_version: u32,
    pub gateway_instance_id: String,
    pub launch_input_hash: String,
    pub gateway_pid: u32,
    pub gateway_pid_start_time_ticks: u64,
    pub user_namespace_device_id: u64,
    pub user_namespace_inode: u64,
    pub dumpable: u32,
    pub rlimit_core_soft: u64,
    pub rlimit_core_hard: u64,
    pub tracer_pid: u32,
    pub attestation_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjectionSourceManifestV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub fence_id: String,
    pub source_root: CanonicalDirectoryV1,
    pub native_projection_hash: String,
    pub ordered_file_hashes: Vec<String>,
    pub created_at: Timestamp,
    pub manifest_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayAccessBoundaryRefV1 {
    pub authority_store_id: String,
    pub access_boundary_id: String,
    pub revision: u64,
    pub boundary_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3KernelEffectIntentRefV1 {
    pub authority_store_id: String,
    pub effect_intent_id: String,
    pub intent_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3KernelEffectIntentV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub effect_intent_id: String,
    pub preparation_id: String,
    pub fence_id: String,
    pub effect: E3KernelEffectKindV1,
    pub created_at: Timestamp,
    pub intent_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum E3KernelEffectKindV1 {
    CreateChildCgroup {
        cgroup_registration_id: String,
        role: E3TerminalProcessRoleV1,
        parent_cgroup: CanonicalCgroupIdentityV1,
        child_component: String,
        expected_relative_path: String,
    },
    InstallGatewayBoundary {
        access_boundary_id: String,
        network_namespace_inode: u64,
        table_name: String,
        chain_name: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3KernelEffectResolutionV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub resolution_id: String,
    pub effect_intent_ref: E3KernelEffectIntentRefV1,
    pub disposition: E3KernelEffectResolutionDispositionV1,
    pub observed_cgroup: Option<CanonicalCgroupIdentityV1>,
    pub observed_nftables_table_handle: Option<u64>,
    pub resolved_at: Timestamp,
    pub resolution_hash: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum E3KernelEffectResolutionDispositionV1 {
    NoEffectObserved,
    RevertedAndQuiescent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3ChildProcessRegistrationV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub registration_id: String,
    pub cgroup_registration_id: String,
    pub cgroup_registration_hash: String,
    pub fence_id: String,
    pub role: E3TerminalProcessRoleV1,
    pub pid: u32,
    pub pid_start_time_ticks: u64,
    pub process_cgroup: CanonicalCgroupIdentityV1,
    pub kernel_boot_id: String,
    pub parent_service_instance_id: String,
    pub registered_at: Timestamp,
    pub registration_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3ChildCgroupRegistrationV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub cgroup_registration_id: String,
    pub kernel_effect_intent_ref: E3KernelEffectIntentRefV1,
    pub fence_id: String,
    pub turn_id: Option<String>,
    pub role: E3TerminalProcessRoleV1,
    pub cgroup: CanonicalCgroupIdentityV1,
    pub kernel_boot_id: String,
    pub registered_at: Timestamp,
    pub cgroup_registration_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayListenerIdentityV1 {
    pub transport: String,
    pub network_namespace_inode: u64,
    pub address: String,
    pub port: u16,
    pub socket_inode: u64,
    pub listen_backlog: u32,
    pub deny_boundary_effective_before_listen: bool,
    pub responses_base_path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NftablesChainIdentityV1 {
    pub family: String,
    pub table: String,
    pub chain: String,
    pub chain_handle: u64,
    pub chain_type: String,
    pub hook: String,
    pub priority: i32,
    pub policy: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum NftablesRuleRoleV1 {
    ReadinessProbeAccept,
    ExactMemberAccept,
    RejectRemainder,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NftablesRuleIdentityV1 {
    pub role: NftablesRuleRoleV1,
    pub family: String,
    pub table: String,
    pub chain: String,
    pub rule_handle: u64,
    pub canonical_expression: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum GatewayAccessPostureV1 {
    DenyAllDormant,
    AllowExactMember,
    Revoked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayAccessBoundaryV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub kernel_effect_intent_ref: E3KernelEffectIntentRefV1,
    pub access_boundary_id: String,
    pub gateway_instance_id: String,
    pub config_projection_identity_hash: String,
    pub orchestration_session_id: String,
    pub retained_participant_id: String,
    pub backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub gateway_listener: GatewayListenerIdentityV1,
    pub readiness_probe_cgroup: CanonicalCgroupIdentityV1,
    pub allowed_member_cgroup: CanonicalCgroupIdentityV1,
    pub nftables_chain: NftablesChainIdentityV1,
    pub nftables_rules: Vec<NftablesRuleIdentityV1>,
    pub posture: GatewayAccessPostureV1,
    pub revision: u64,
    pub predecessor_ref: Option<GatewayAccessBoundaryRefV1>,
    pub boundary_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedGatewayActivationAckRefV1 {
    pub authority_store_id: String,
    pub activation_ack_id: String,
    pub ack_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedGatewayProjectionV1 {
    pub projection_hash: String,
    pub activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    pub expected_gateway_ref: InWorldGatewayRefV1,
    pub codex_base_url: String,
    pub access_boundary_ref: GatewayAccessBoundaryRefV1,
    pub activation_ack_ref: Option<ManagedGatewayActivationAckRefV1>,
    pub posture: ManagedGatewayProjectionPostureV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ManagedGatewayProjectionPostureV1 {
    Dormant,
    ReadyClosed,
    Active,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecretHandoffRefV1 {
    pub authority_store_id: String,
    pub handoff_id: String,
    pub orchestration_session_id: String,
    pub retained_participant_id: String,
    pub runtime_family: String,
    pub world_id: String,
    pub world_generation: u64,
    pub receiving_gateway_identity_hash: String,
    pub handoff_state_revision: u64,
    pub handoff_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialSourceRefV1 {
    pub schema_version: u32,
    pub credential_source_id: String,
    pub preparation_id: String,
    pub selected_backend_id: String,
    pub bundle_backend_id: String,
    pub ordered_field_names: Vec<String>,
    pub optional_account_id_present: bool,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
    pub ref_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum SecretDeliveryMechanismV1 {
    SecureFd {
        fd_name: String,
        one_time: bool,
        gateway_receiver_only: bool,
        deny_child_inheritance: bool,
        close_after_consume: bool,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SecretHandoffStateV1 {
    Prepared,
    Delivered,
    Consumed,
    Failed,
    Expired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NonsecretHandoffProjectionV1 {
    pub projection_hash: String,
    pub secret_handoff_ref: SecretHandoffRefV1,
    pub credential_source_ref: CredentialSourceRefV1,
    pub receiving_gateway_ref: InWorldGatewayRefV1,
    pub delivery: SecretDeliveryMechanismV1,
    pub observed_state: SecretHandoffStateV1,
    pub activation_ack_ref: Option<ManagedGatewayActivationAckRefV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionActivationV1 {
    pub activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    pub publication_fence: ConfigProjectionPublicationFenceV1,
    pub gateway_activation_ack_ref: Option<ManagedGatewayActivationAckRefV1>,
    pub released_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ConfigProjectionPublicationFenceV1 {
    ZeroLiveClosed {
        fence_id: String,
    },
    Released {
        fence_id: String,
        closed_record_ref: ConfigProjectionRefV1,
        activation_ack_ref: ManagedGatewayActivationAckRefV1,
        release_hash: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfigProjectionRecordV1 {
    pub schema_version: u32,
    pub identity: ConfigProjectionIdentityV1,
    pub record_id: String,
    pub revision: u64,
    pub predecessor_ref: Option<ConfigProjectionRefV1>,
    pub logical: LogicalAgentConfigProjectionV1,
    pub effective: EffectiveAgentConfigProjectionV1,
    pub native: NativeAgentConfigProjectionV1,
    pub managed_gateway: ManagedGatewayProjectionV1,
    pub nonsecret_handoff: NonsecretHandoffProjectionV1,
    pub activation: ConfigProjectionActivationV1,
    pub created_at: Timestamp,
    pub record_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionStoreV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub accepted_home: CanonicalDirectoryV1,
    pub created_at: Timestamp,
    pub store_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledAcceptedHomeBootstrapRecordV1 {
    pub schema_version: u32,
    pub install_bootstrap_carrier: String,
    pub host_context_commitment: String,
    pub intended_account: String,
    pub intended_uid: u64,
    pub intended_gid: u64,
    pub accepted_home: CanonicalDirectoryV1,
    pub installed_at: Timestamp,
    pub record_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledAcceptedHomeBootstrapHeadV1 {
    pub schema_version: u32,
    pub head_record_hash: String,
    pub predecessor_head_hash: Option<String>,
    pub updated_at: Timestamp,
    pub head_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionSubjectBindingV1 {
    pub schema_version: u32,
    pub subject_hash: String,
    pub identity: ConfigProjectionIdentityV1,
    pub series_id: String,
    pub created_at: Timestamp,
    pub binding_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionHeadV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub head_ref: ConfigProjectionRefV1,
    pub head_revision: u64,
    pub predecessor_head_hash: Option<String>,
    pub updated_at: Timestamp,
    pub head_hash: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConfigProjectionConsumerKindV1 {
    MemberDispatchV2,
    WorldRuntimeAdapterV3,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConfigProjectionConsumerLeasePostureV1 {
    Held,
    Released,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionConsumerLeaseV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub consumer_id: String,
    pub consumer_kind: ConfigProjectionConsumerKindV1,
    pub revision: u64,
    pub predecessor_lease_hash: Option<String>,
    pub acquired_projection_ref: ConfigProjectionRefV1,
    pub posture: ConfigProjectionConsumerLeasePostureV1,
    pub acquired_at: Timestamp,
    pub released_at: Option<Timestamp>,
    pub lease_hash: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConfigProjectionEvidenceKindV1 {
    SecretHandoff,
    ConsumerLease,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionEvidenceHeadV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub evidence_kind: ConfigProjectionEvidenceKindV1,
    pub series_id: Option<String>,
    pub evidence_owner_id: String,
    pub head_revision: u64,
    pub head_object_hash: String,
    pub predecessor_head_hash: Option<String>,
    pub updated_at: Timestamp,
    pub evidence_head_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3TerminalChildQuiescenceEvidenceRefV1 {
    pub authority_store_id: String,
    pub series_id: String,
    pub evidence_id: String,
    pub evidence_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3TerminalChildQuiescenceEvidenceV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub evidence_id: String,
    pub final_projection_ref: ConfigProjectionRefV1,
    pub world_id: String,
    pub world_generation: u64,
    pub ordered_terminal_processes: Vec<E3TerminalProcessObservationV1>,
    pub ordered_empty_cgroups: Vec<E3TerminalCgroupQuiescenceV1>,
    pub observed_at: Timestamp,
    pub evidence_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3TerminalProcessObservationV1 {
    pub registration_id: String,
    pub registration_hash: String,
    pub role: E3TerminalProcessRoleV1,
    pub pid: u32,
    pub pid_start_time_ticks: u64,
    pub observation: E3TerminalProcessObservationKindV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum E3TerminalProcessRoleV1 {
    ManagedGateway,
    Codex,
    ReadinessProbe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum E3TerminalProcessObservationKindV1 {
    ParentWaitid {
        terminal_wait_status: i32,
    },
    RecoveryObservedTerminal {
        original_service_instance_id: String,
        recovery_service_instance_id: String,
        pidfd_open_errno: Option<i32>,
        pidfd_kill_errno: Option<i32>,
        pidfd_became_readable: Option<bool>,
        waitid_errno: Option<i32>,
        proc_identity: E3RecoveryProcIdentityV1,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum E3RecoveryProcIdentityV1 {
    Absent,
    PidReused { observed_pid_start_time_ticks: u64 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalCgroupIdentityV1 {
    pub cgroup_v2_mount_device_id: u64,
    pub cgroup_v2_mount_inode: u64,
    pub cgroup_directory_inode: u64,
    pub cgroup_relative_path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct E3TerminalCgroupQuiescenceV1 {
    pub cgroup_registration_id: String,
    pub cgroup_registration_hash: String,
    pub role: E3TerminalProcessRoleV1,
    pub cgroup: CanonicalCgroupIdentityV1,
    pub cgroup_events_sha256: String,
    pub cgroup_procs_sha256: String,
    pub populated: bool,
    pub ordered_live_pids: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigProjectionRetirementV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub series_id: String,
    pub final_head_ref: ConfigProjectionRefV1,
    pub terminal_child_evidence_ref: E3TerminalChildQuiescenceEvidenceRefV1,
    pub revoked_boundary_ref: GatewayAccessBoundaryRefV1,
    pub consumer_lease_count: u64,
    pub retired_at: Timestamp,
    pub retirement_hash: String,
}

/// Closed E3-B failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigProjectionFailureV1 {
    Malformed,
    MissingPreparation,
    ExpiredPreparation,
    CancelledPreparation,
    WrongBinding,
    HashInvalid,
    StaleRevision,
    PartialPublication,
    Conflict,
    UnsupportedConfiguration,
    UnsupportedPolicySurface,
    UnsupportedRuntimeVersion,
    UnsupportedPlatform,
    UnsupportedSecurityPosture,
    RetiredSeries,
}

impl std::fmt::Display for ConfigProjectionFailureV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ConfigProjectionFailureV1 {}
