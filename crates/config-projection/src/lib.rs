//! Durable, accepted-home authority for E3 agent configuration projections.

mod codec;
mod registry;

pub use codec::ConfigProjectionCodecV1;
pub use registry::*;

pub use transport_api_types::{
    ConfigProjectionRefV1, DispatchPolicyCommitmentRefCarrierV1, E2MemberLaunchKindV1,
    InWorldGatewayRefV1, ManagedGatewayActivationIntentRefV1, PolicyRefV1, WorldBindingRefV1,
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
