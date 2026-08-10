use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{
    Signature as Ed25519Signature, Signer as _, SigningKey as Ed25519SigningKey, Verifier as _,
    VerifyingKey as Ed25519PublicKey,
};
use p256::ecdsa::{Signature as P256Signature, VerifyingKey as P256PublicKey};
use p256::pkcs8::{DecodePublicKey, EncodePublicKey};
use p256::PublicKey as P256PublicKeyDocument;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

const MANIFEST_SCHEMA_OWNER_V1: &str = "substrate.managed-artifact-manifest";
const MANIFEST_SCHEMA_VERSION_V1: u32 = 1;
const ACTION_PREPARED_RECORD_OWNER_V1: &str = "substrate.managed-action-prepared-record";
const ACTION_RECEIPT_INDEX_OWNER_V1: &str = "substrate.managed-action-receipt-index";
const LIFECYCLE_PUBLISHER_PROTECTED_STATE_OWNER_V1: &str =
    "substrate.lifecycle-publisher-protected-state";
const LIFECYCLE_SIGNATURE_PREFIX_V1: &[u8] = b"SUBSTRATE-LIFECYCLE-SIGNATURE-V1\0";
const LIMA_STAGE_ONE_SCHEMA_OWNER_V1: &str = "substrate.lima-stage-one-authorization";
const MAC_PUBLISHER_CONTROL_AUTHORITY_OWNER_V1: &str = "substrate.mac-publisher-control-authority";
const MAC_PUBLISHER_INSTALL_PROVENANCE_OWNER_V1: &str =
    "substrate.mac-publisher-install-provenance";
const MAC_PUBLISHER_CONTROL_ADMISSION_OWNER_V1: &str = "substrate.mac-publisher-control-admission";
pub const GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_OWNER_V1: &str =
    "substrate.guest-publisher-pairing-session-binding";
pub const GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_VERSION_V1: u32 = 1;
pub const GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_OWNER_V1: &str =
    "substrate.guest-publisher-pairing-data-session";
pub const GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_VERSION_V1: u32 = 1;
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_OWNER_V1: &str =
    "substrate.guest-publisher-pairing-operator-tty-session";
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_VERSION_V1: u32 = 1;
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_LAUNCH_SCHEMA_OWNER_V1: &str =
    "substrate.guest-publisher-pairing-operator-launch";
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_LAUNCH_SCHEMA_VERSION_V1: u32 = 1;
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_SCHEMA_OWNER_V1: &str =
    "substrate.guest-publisher-pairing-operator-proof";
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_SCHEMA_VERSION_V1: u32 = 1;
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_GUEST_EXECUTABLE_PATH_V1: &str =
    "/usr/libexec/substrate/substrate-lifecycle-linux";
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_FIXED_COMMAND_V1: &str =
    "guest-pairing-operator-tty-session-v1";
pub const GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_TERMINAL_OBSERVATION_V1: &str =
    "guest-controlling-tty-confirmed-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedArtifactDispositionV1 {
    Created,
    PreExistingPreserved,
    Adopted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedLifecycleStateV1 {
    RecordedBefore,
    ManifestPrepared,
    ManifestDurable,
    ActionStarted,
    ActionObserved,
    ReceiptDurable,
    Committed,
    Uninstalled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedActionV1 {
    Create,
    Replace,
    Remove,
    Restore,
    Enable,
    Disable,
    Start,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ManagedArtifactRoleV1(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PublisherBootstrapComponentRoleV1(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedArtifactIdentityV1 {
    pub scope_id: String,
    pub parent_identity: String,
    pub name_identity: String,
    pub physical_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedArtifactEntryV1 {
    pub object_id: String,
    pub logical_role: ManagedArtifactRoleV1,
    pub object_type: String,
    pub identity: ManagedArtifactIdentityV1,
    pub disposition: ManagedArtifactDispositionV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_or_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acl_or_security: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_or_platform_state: Option<Value>,
    pub before_state: Value,
    pub intended_after_state: Value,
    pub restoration_state: Value,
    #[serde(default)]
    pub dependency_object_ids: Vec<String>,
    #[serde(default)]
    pub subtree_members: Vec<String>,
    pub lifecycle_state: ManagedLifecycleStateV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_durable_transition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedExecutorIdentityV1 {
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub target_triple: String,
    pub artifact_sha256: String,
    pub artifact_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_identity: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LifecycleSignatureV1 {
    pub algorithm: String,
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedActionPreparedRecordV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub authority_domain: String,
    pub scope_id: String,
    pub installation_id: String,
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    pub receipt_id: String,
    pub receipt_relative_path: String,
    pub entry_id: String,
    pub action: ManagedActionV1,
    pub attempt_id: String,
    pub request_sha256: String,
    pub before_observation: Value,
    pub executor_identity: ManagedExecutorIdentityV1,
    pub allocated_counter: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_record_sha256: Option<String>,
    pub state: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedActionReceiptV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub authority_domain: String,
    pub scope_id: String,
    pub installation_id: String,
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    pub receipt_id: String,
    pub receipt_relative_path: String,
    pub entry_id: String,
    pub action: ManagedActionV1,
    pub attempt_id: String,
    pub prepared_record_sha256: String,
    pub allocated_counter: u64,
    pub request_sha256: String,
    pub pre_observation: Value,
    pub effect_observation: Value,
    pub post_observation: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restoration_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
    pub executor_identity: ManagedExecutorIdentityV1,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedActionReceiptIndexEntryV1 {
    pub receipt_id: String,
    pub entry_id: String,
    pub action: ManagedActionV1,
    pub attempt_id: String,
    pub receipt_relative_path: String,
    pub canonical_byte_length: u64,
    pub receipt_artifact_sha256: String,
    pub retained_receipt_file_identity: String,
    pub retained_receipt_parent_identity: String,
    pub prepared_record_sha256: String,
    pub allocated_counter: u64,
    pub durable_observation: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedActionReceiptIndexV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub authority_domain: String,
    pub scope_id: String,
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    pub index_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_index_sha256: Option<String>,
    #[serde(default)]
    pub entries: Vec<ManagedActionReceiptIndexEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedManifestHeadV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub scope_id: String,
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    pub lifecycle_state: ManagedLifecycleStateV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_head_sha256: Option<String>,
    pub action_receipt_index_revision: u64,
    pub action_receipt_index_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedSharedClaimV1 {
    pub claim_id: String,
    pub logical_role: ManagedArtifactRoleV1,
    pub compatible_desired_state_sha256: String,
    pub before_state: Value,
    pub installation_id: String,
    pub host_context_commitment: String,
    pub manifest_sha256: String,
    pub lifecycle_state: ManagedLifecycleStateV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedSharedClaimsV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub manifest_generation: u64,
    pub manifest_head_sha256: String,
    #[serde(default)]
    pub claims: Vec<ManagedSharedClaimV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LifecyclePublisherAnchorV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub authority_domain: String,
    pub host_context_commitment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_mapping_commitment: Option<String>,
    pub scope_id: String,
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    pub action_receipt_index_revision: u64,
    pub action_receipt_index_sha256: String,
    pub head_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_anchor_sha256: Option<String>,
    pub request_sha256: String,
    pub requester_principal: String,
    pub attempt_nonce: String,
    pub executor_identity: ManagedExecutorIdentityV1,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LifecyclePublisherProtectedStateV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub current_anchor: LifecyclePublisherAnchorV1,
    pub counter: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_record: Option<ManagedActionPreparedRecordV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_protected_state_sha256: Option<String>,
    pub state_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedLifecyclePublisherRequestV1 {
    pub host_context_commitment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_mapping_commitment: Option<String>,
    pub scope_id: String,
    pub current_anchor_counter: u64,
    pub current_anchor_sha256: String,
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    pub role: ManagedArtifactRoleV1,
    pub action: ManagedActionV1,
    pub object_identity: ManagedArtifactIdentityV1,
    pub requester_principal: String,
    pub attempt_nonce: String,
    pub expected_executor_build: ManagedExecutorIdentityV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherBootstrapComponentV1 {
    pub component_id: String,
    pub role: PublisherBootstrapComponentRoleV1,
    pub target_identity: String,
    pub object_type: String,
    pub expected_before: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_artifact_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_signature_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acl_or_security: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_requirement: Option<String>,
    #[serde(default)]
    pub dependency_component_ids: Vec<String>,
    pub durability_method: String,
}

/// Immutable authorization for the one macOS host-control image that may connect to the fixed
/// lifecycle publisher Mach service.
///
/// The authority is installed before the listener starts. It is never inferred from an accepted
/// XPC peer, which would turn peer admission into a trust-on-first-use decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacPublisherControlAuthorityV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub control_binary: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub target_triple: String,
    pub artifact_sha256: String,
    pub designated_requirement: String,
}

/// Descriptor-measured identity for one fixed executable retained by the macOS bootstrap
/// provenance record.  This deliberately records the identity of the opened regular file rather
/// than a caller-selected pathname.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacPublisherBootstrapImageProvenanceV1 {
    pub target_triple: String,
    pub artifact_sha256: String,
    pub physical_identity: String,
    /// Measured CodeDirectory identity (currently the canonical `cdhash:<lower-hex>` form).
    /// This is distinct from the designated requirement below: the former binds the measured
    /// opened image while the latter is the policy used for peer admission.
    pub code_identity: String,
    pub code_requirement: String,
}

/// Identity of the one installer-recorded `limactl` executable.  The Stage-1 executor opens this
/// exact absolute path no-follow before it builds its fixed, selector-free command line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacLimaToolProvenanceV1 {
    pub absolute_path: String,
    pub image: MacPublisherBootstrapImageProvenanceV1,
    pub version: String,
}

/// Install-time provenance for one of the four fixed Linux artifacts consumed by the macOS
/// Lima publisher.  The installer is the sole producer: neither a caller nor a warm-up wrapper
/// can choose the role, source path, package, binary, or build invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacLimaRetainedArtifactProvenanceV1 {
    pub logical_role: String,
    pub retained_relative_path: String,
    pub cargo_package: String,
    pub cargo_binary: String,
    pub target_triple: String,
    pub cargo_lock_sha256: String,
    pub toolchain: String,
    pub build_command: String,
    pub artifact_sha256: String,
    pub physical_identity: String,
    pub mode: String,
}

/// Root-owned install-time source of truth for direct macOS publisher bootstrap. It contains
/// *only* facts durable when the installer finishes: no scope, attempt, manifest, or PM value is
/// allowed here. The direct bootstrap creates those per-scope facts later in its FD3 transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacPublisherInstallProvenanceV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub review_record_sha256: String,
    pub host_context_commitment: String,
    pub selected_host_prefix: String,
    pub control_authority: MacPublisherControlAuthorityV1,
    pub control_image: MacPublisherBootstrapImageProvenanceV1,
    pub executor_image: MacPublisherBootstrapImageProvenanceV1,
    pub launch_daemon_plist_sha256: String,
    pub lima_tool: MacLimaToolProvenanceV1,
    pub retained_linux_artifacts: Vec<MacLimaRetainedArtifactProvenanceV1>,
    pub profile_template_algorithm: String,
    pub profile_template_version: u32,
    pub profile_template_sha256: String,
}

/// Fixed global Keychain admission record.  It binds a measured peer to the current signed
/// anchor and protected-state revision before any XPC request bytes can be decoded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacPublisherControlAdmissionV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub scope_id: String,
    pub control_authority: MacPublisherControlAuthorityV1,
    pub bootstrap_authorization_sha256: String,
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    pub current_anchor_sha256: String,
    pub state_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherRetirementReservationV1 {
    pub reservation_id: String,
    pub challenge_id: String,
    pub host_pairing_component_id: String,
    #[serde(default)]
    pub guest_component_ids: Vec<String>,
    pub expected_before_sha256: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherTestRetirementCommitmentV1 {
    pub harness_public_key: String,
    pub harness_algorithm: String,
    pub authorization_digest: String,
    pub external_parent_identity: String,
    #[serde(default)]
    pub guest_reservations: Vec<GuestPublisherRetirementReservationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherBootstrapAuthorizationV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub authority_domain: String,
    pub scope_id: String,
    pub host_context_commitment: String,
    /// The exact canonical IH carrier used only within the retained direct-bootstrap FD3
    /// authorization. It is never accepted by the ordinary mapped request decoder.
    pub install_bootstrap_context_v1: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_mapping_commitment: Option<String>,
    pub requester_principal: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    /// The exact canonical manifest identity copied into the first signed anchor. These fields
    /// travel only in the direct FD3 authorization; the seed/manifest bytes never do.
    pub manifest_generation: u64,
    pub manifest_sha256: String,
    /// The full canonical generation-one pre-PM manifest. This exists only in the direct FD3
    /// authorization and binds the initial anchor to durable capsule bytes without asking a
    /// caller to provide a manifest seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_pm_manifest: Option<ManagedArtifactManifestV1>,
    pub issued_at_unix_ns: u64,
    pub expires_at_unix_ns: u64,
    pub attempt_nonce: String,
    pub publisher_expected_absent: bool,
    pub executor_build_evidence: ExecutorBuildEvidenceV1,
    /// Required only for the macOS host publisher. It binds the immutable control image and
    /// designated requirement before the fixed XPC service may admit any peer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mac_control_authority: Option<MacPublisherControlAuthorityV1>,
    #[serde(default)]
    pub components: Vec<PublisherBootstrapComponentV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_retirement_commitment: Option<PublisherTestRetirementCommitmentV1>,
    pub confirmation: String,
}

/// The hidden direct-bootstrap stdin request. Its sole field is the exact IH carrier; every
/// source/build/profile/manifest/action authority is constructed from retained install evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacPublisherBootstrapRequestV1 {
    pub install_bootstrap_context_v1: String,
}

/// A fixed-key, scope-independent System-Keychain locator for the one direct macOS bootstrap
/// attempt selected by an IH-only request.  This deliberately retains only stable joins and the
/// bounded public response: it never stores the complete FD3 authorization or creates a caller
/// supplied retry locator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacPublisherBootstrapAttemptLocatorV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub attempt_key_sha256: String,
    pub authority_domain: String,
    pub host_context_commitment: String,
    pub requester_principal: String,
    pub selected_host_prefix: String,
    pub install_provenance_sha256: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub scope_id: String,
    pub attempt_id: String,
    pub issued_at_unix_ns: u64,
    pub expires_at_unix_ns: u64,
    pub bootstrap_request_sha256: String,
    pub bootstrap_authorization_sha256: String,
    pub pre_pm_manifest_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_anchor_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_state_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capsule_sha256: Option<String>,
    /// `Allocated` is recoverable before the first key/capsule effect. `Completed` carries one
    /// canonical public response. `PreservingBlocked` intentionally cannot be retried blindly.
    pub terminal_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_response_b64: Option<String>,
    pub locator_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherTestRetirementAuthorizationV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub baseline_absence_sha256: String,
    pub evidence_id: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub platform_scope: String,
    pub bootstrap_core_digest: String,
    #[serde(default)]
    pub component_ids: Vec<String>,
    pub maximum_counter: u64,
    pub external_receipt_directory_identity: String,
    pub retirement_nonce: String,
    #[serde(default)]
    pub guest_reservations: Vec<GuestPublisherRetirementReservationV1>,
    pub harness_public_key: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherTestRetirementReceiptV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub final_anchor_sha256: String,
    pub final_protected_state_sha256: String,
    pub public_key: String,
    pub receipt_nonce: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherTestRetirementAcknowledgementV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub receipt_artifact_sha256: String,
    pub external_receipt_directory_identity: String,
    pub acknowledgement_nonce: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherReservationUnusedProofV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub reservation_id: String,
    pub challenge_id: String,
    pub scope_id: String,
    pub state: String,
    pub ticket_sha256: String,
    pub reservation_sha256: String,
    pub proof_nonce: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherReservationUnusedAcknowledgementV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub reservation_id: String,
    pub proof_artifact_sha256: String,
    pub acknowledgement_nonce: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherTestRetirementCommitmentV1 {
    pub reservation_id: String,
    pub authorization_digest: String,
    pub host_pairing_record_digest: String,
    #[serde(default)]
    pub guest_component_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherTestRetirementAuthorizationV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub reservation_id: String,
    pub ticket_sha256: String,
    pub commitment: GuestPublisherTestRetirementCommitmentV1,
    pub harness_public_key: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherTestRetirementReceiptV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub reservation_id: String,
    pub final_anchor_sha256: String,
    pub final_protected_state_sha256: String,
    pub receipt_nonce: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherTestRetirementAcknowledgementV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub reservation_id: String,
    pub receipt_artifact_sha256: String,
    pub acknowledgement_nonce: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherTransportFrameV1 {
    pub protocol_version: u32,
    pub request_bytes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_bytes: Option<String>,
    pub transport_nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingChallengeV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub challenge_id: String,
    pub challenge: String,
    pub expires_at_unix_ns: u64,
    pub host_key_fingerprint_sha256: String,
    pub current_anchor_sha256: String,
    pub host_context_commitment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_mapping_commitment: Option<String>,
    pub guest_machine_identity: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub executor_build_evidence_sha256: String,
    pub guest_component_commitment_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingTicketV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub challenge: GuestPublisherPairingChallengeV1,
    pub signer_spki_der: String,
    pub current_anchor: LifecyclePublisherAnchorV1,
    pub current_anchor_sha256: String,
    pub challenge_sha256: String,
    pub host_generation: u64,
    pub host_counter: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_test_retirement_commitment: Option<GuestPublisherTestRetirementCommitmentV1>,
    pub signature: LifecycleSignatureV1,
}

/// Immutable authority shared by the exact two R6 guest-pairing sessions.
///
/// This deliberately names no transport, command, endpoint, or path. The host records it before
/// either child observes guest data so a resumed child cannot substitute a ticket or executor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingSessionBindingV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub scope_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_mapping_commitment: Option<String>,
    pub guest_machine_identity: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub staged_executor_sha256: String,
    pub stage_one_record_sha256: String,
    pub ticket_challenge_id: String,
    pub host_record_generation: u64,
    pub pairing_session_nonce: String,
}

/// Signed, selector-free authority for exactly one direct guest operator launch.
///
/// The signature is an opaque P-256 P1363 value.  The retained ticket supplies the sole
/// verification key, so this envelope carries no ticket or key material.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingOperatorLaunchV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub binding: GuestPublisherPairingSessionBindingV1,
    pub operator_session_id: String,
    pub admitted_instance_name: String,
    pub expires_at_unix_ns: u64,
    pub limactl_absolute_path: String,
    pub limactl_sha256: String,
    pub guest_executable_path: String,
    pub guest_executable_sha256: String,
    pub fixed_operator_command: String,
    pub signature: String,
}

/// One immutable, non-reusable guest-TTY proof.  It retains only a commitment and fixed binding
/// metadata; reusable values, tickets, transcripts, keys, paths, and selectors are forbidden.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingOperatorProofV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub binding: GuestPublisherPairingSessionBindingV1,
    pub operator_session_id: String,
    /// Canonical SHA-256 of the exact signed operator launch admitted by the host record.
    pub operator_launch_sha256: String,
    pub confirmation_commitment: String,
    pub created_at_unix_ns: u64,
    pub expires_at_unix_ns: u64,
    pub terminal_observation: String,
}

/// The one PM-bound session that may carry the signed ticket and typed guest frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingDataSessionV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub binding: GuestPublisherPairingSessionBindingV1,
    pub session_id: String,
    pub ticket: GuestPublisherPairingTicketV1,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hello: Option<GuestPublisherBootstrapHelloV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<GuestPublisherBootstrapTranscriptV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_anchor_sha256: Option<String>,
    pub started_observation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_observation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_observation: Option<String>,
}

/// The independent PM-bound guest controlling-TTY session.
///
/// It retains only lifecycle observations.  The immutable proof commitment is carried solely by
/// [`GuestPublisherPairingOperatorProofV1`] and its canonical digest in the host record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingOperatorTtySessionV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub binding: GuestPublisherPairingSessionBindingV1,
    pub session_id: String,
    pub controlling_tty_identity: String,
    pub state: String,
    pub started_observation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_observation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_observation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingHostRecordV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub ticket: GuestPublisherPairingTicketV1,
    pub current_anchor_counter: u64,
    pub current_anchor_sha256: String,
    pub binding: GuestPublisherPairingSessionBindingV1,
    pub data_session: GuestPublisherPairingDataSessionV1,
    pub operator_tty_session: GuestPublisherPairingOperatorTtySessionV1,
    /// Canonical SHA-256 of the exact signed operator launch admitted for this pairing.
    pub operator_launch_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_proof_sha256: Option<String>,
    pub record_generation: u64,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_record_sha256: Option<String>,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherPairingGuestIntentV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub scope_id: String,
    pub challenge_id: String,
    pub ticket_sha256: String,
    pub guest_machine_identity: String,
    pub guest_artifact_sha256: String,
    pub seed: String,
    pub public_key: String,
    pub public_key_sha256: String,
    pub nonce: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_test_retirement_commitment: Option<GuestPublisherTestRetirementCommitmentV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<GuestPublisherBootstrapTranscriptV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hello: Option<GuestPublisherBootstrapHelloV1>,
    pub state: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherBootstrapHelloV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub ticket_sha256: String,
    pub intent_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_test_retirement_commitment: Option<GuestPublisherTestRetirementCommitmentV1>,
    pub guest_machine_identity: String,
    pub guest_artifact_sha256: String,
    pub intent_published_at_unix_ns: u64,
    pub intent_parent_fsync_observed: bool,
    pub nonce: String,
    pub guest_public_key: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPublisherBootstrapTranscriptV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub ticket_sha256: String,
    pub hello_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guest_test_retirement_commitment: Option<GuestPublisherTestRetirementCommitmentV1>,
    pub guest_machine_identity: String,
    pub staged_executor_sha256: String,
    pub guest_nonce: String,
    pub guest_public_key_sha256: String,
    pub signature: LifecycleSignatureV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecutorBuildEvidenceV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub artifact_sha256: String,
    pub artifact_identity: String,
    pub target_triple: String,
    #[serde(default)]
    pub tool_versions: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LimaStageOneAuthorizationV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub host_context_commitment: String,
    pub lima_control_root_identity: String,
    pub instance_name: String,
    pub profile_sha256: String,
    pub expected_absent: bool,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub executor_receipt_sha256: String,
    pub requester_principal: String,
    pub attempt_id: String,
    pub nonce: String,
    pub expires_at_unix_ns: u64,
    pub rendered_profile_b64: String,
    pub rendered_profile_sha256: String,
    /// The one signed pre-effect source for a post-create PM manifest.  It deliberately contains
    /// no fabricated mapping commitment, machine identifier, or complete post-PM manifest.
    pub successor_template: MacLimaStageOneSuccessorTemplateV1,
    pub signature: LifecycleSignatureV1,
}

/// The exact post-effect observation admitted to the Stage-1 transition. No caller-provided
/// mapping or machine value can substitute for this executor-owned tuple.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LimaStageOneObservationV1 {
    pub instance_name: String,
    pub instance_status: String,
    pub guest_machine_id: String,
    pub marker_attempt_id: String,
    pub marker_capsule_sha256: String,
    pub guest_account: String,
    pub guest_uid: u32,
    pub guest_home: String,
}

/// Durable per-scope transition state for the one Stage-1 effect. The System-Keychain account is
/// fixed from the scope and the record is absent-or-exact/CAS only; it is not a reusable request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacLimaStageOneCapsuleV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub scope_id: String,
    pub state: String,
    pub capsule_revision: u64,
    pub install_provenance_sha256: String,
    pub bootstrap_authorization_sha256: String,
    pub pre_pm_manifest: ManagedArtifactManifestV1,
    pub initial_anchor_sha256: String,
    pub initial_anchor_counter: u64,
    /// Digest of the exact prepared protected-state snapshot which was sealed before the
    /// executor could begin the one allowed Lima effect.  It makes a crash after the successor
    /// state CAS distinguishable from a still-pending effect without re-running the effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_protected_state_sha256: Option<String>,
    pub stage_one_authorization: LimaStageOneAuthorizationV1,
    pub rendered_profile_file_identity: Option<String>,
    pub observation: Option<LimaStageOneObservationV1>,
}

/// Signed, pre-effect-only inputs to the Stage-1 successor derivation.  The privileged executor
/// adds precisely one observed value (`guest_machine_id`) after the selected absent-instance
/// effect and uses the repository's canonical Lima mapping constructor to materialize the full
/// PM-bound manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacLimaStageOneSuccessorTemplateV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub derivation_algorithm: String,
    pub current_pre_pm_manifest_generation: u64,
    pub current_pre_pm_manifest_sha256: String,
    pub current_anchor_sha256: String,
    pub current_anchor_counter: u64,
    pub next_manifest_generation: u64,
    pub previous_manifest_sha256: String,
    pub host_context_commitment: String,
    pub scope_id: String,
    pub installation_id: String,
    pub intended_principal: String,
    pub selected_host_prefix: String,
    pub host_platform_control_root: String,
    pub instance_name: String,
    pub profile_sha256: String,
    pub source_commit: String,
    pub source_tree: String,
    pub source_ref: String,
    pub executor_build_evidence: ExecutorBuildEvidenceV1,
    pub attempt_id: String,
    pub nonce: String,
    pub expires_at_unix_ns: u64,
    pub manifest_created_at_unix_ns: u64,
    pub manifest_lifecycle_state: ManagedLifecycleStateV1,
    pub profile_template_algorithm: String,
    pub profile_template_version: u32,
    pub profile_template_sha256: String,
    #[serde(default)]
    pub ordered_non_machine_entries: Vec<ManagedArtifactEntryV1>,
    pub planned_receipt_id: String,
    pub planned_receipt_relative_path: String,
    #[serde(default)]
    pub post_effect_observation_slots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedArtifactManifestV1 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub manifest_id: String,
    pub manifest_sha256: String,
    pub host_context_commitment: String,
    pub selected_host_prefix: String,
    pub intended_principal: String,
    pub platform_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_mapping_commitment: Option<String>,
    pub authority_domain: String,
    pub installation_id: String,
    pub attempt_id: String,
    pub manifest_generation: u64,
    pub created_at_unix_ns: u64,
    pub lifecycle_state: ManagedLifecycleStateV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_manifest_sha256: Option<String>,
    #[serde(default)]
    pub entries: Vec<ManagedArtifactEntryV1>,
    #[serde(default)]
    pub planned_action_receipts: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalManifestBytesV1 {
    pub bytes: Vec<u8>,
    pub manifest_sha256: String,
}

pub fn parse_and_validate_manifest_v1(bytes: &[u8]) -> Result<ManagedArtifactManifestV1> {
    let manifest: ManagedArtifactManifestV1 = canonical_json_from_slice(bytes)
        .context("managed artifact manifest is not canonical v1 JSON")?;
    validate_manifest_v1(&manifest)?;
    Ok(manifest)
}

pub fn canonical_manifest_bytes_v1(
    manifest: &ManagedArtifactManifestV1,
) -> Result<CanonicalManifestBytesV1> {
    validate_manifest_v1(manifest)?;
    let bytes = canonical_json_to_vec(manifest).context("encode manifest canonical bytes")?;
    Ok(CanonicalManifestBytesV1 {
        bytes,
        manifest_sha256: manifest.manifest_sha256.clone(),
    })
}

/// Compute the canonical manifest identity without accepting it as caller authority.  Builders
/// use this only while materializing a manifest from already-signed inputs, then immediately
/// validate the completed manifest through [`canonical_manifest_bytes_v1`].
pub fn managed_artifact_manifest_sha256_v1(manifest: &ManagedArtifactManifestV1) -> Result<String> {
    manifest_digest_v1(manifest)
}

pub fn validate_managed_lifecycle_publisher_request_v1(
    request: &ManagedLifecyclePublisherRequestV1,
) -> Result<()> {
    require_hex_digest(
        &request.host_context_commitment,
        "publisher request host_context_commitment",
    )?;
    if let Some(commitment) = &request.platform_mapping_commitment {
        require_hex_digest(commitment, "publisher request platform_mapping_commitment")?;
    }
    require_uuid_v7(&request.scope_id, "publisher request scope_id")?;
    require_hex_digest(
        &request.current_anchor_sha256,
        "publisher request current_anchor_sha256",
    )?;
    if request.manifest_generation == 0 {
        bail!("publisher request manifest_generation must be positive");
    }
    require_hex_digest(
        &request.manifest_sha256,
        "publisher request manifest_sha256",
    )?;
    let parsed_role = parse_managed_role(&request.role.0)?;
    validate_action_allowed_for_role_v1(&parsed_role, request.action)?;
    validate_identity_v1(&request.object_identity)?;
    if request.object_identity.scope_id != request.scope_id {
        bail!("publisher request object_identity scope_id does not match scope_id");
    }
    require_nonempty_no_nul(
        &request.requester_principal,
        "publisher request requester_principal",
    )?;
    require_nonempty_no_nul(&request.attempt_nonce, "publisher request attempt_nonce")?;
    validate_executor_identity_v1(&request.expected_executor_build)
}

pub fn canonical_publisher_bootstrap_authorization_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<Vec<u8>> {
    validate_publisher_bootstrap_authorization_v1(authorization)?;
    canonical_json_to_vec(authorization).context("encode bootstrap authorization")
}

pub fn mac_publisher_bootstrap_request_sha256_v1(
    request: &MacPublisherBootstrapRequestV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_mac_publisher_bootstrap_request_v1(request)?,
    )))
}

pub fn canonical_mac_publisher_bootstrap_attempt_locator_v1(
    locator: &MacPublisherBootstrapAttemptLocatorV1,
) -> Result<Vec<u8>> {
    validate_mac_publisher_bootstrap_attempt_locator_v1(locator)?;
    canonical_json_to_vec(locator).context("encode canonical macOS bootstrap attempt locator")
}

pub fn mac_publisher_bootstrap_attempt_locator_sha256_v1(
    locator: &MacPublisherBootstrapAttemptLocatorV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_mac_publisher_bootstrap_attempt_locator_v1(locator)?,
    )))
}

/// Hash the validated canonical bootstrap authorization for the existing signed-anchor retry
/// binding. The digest never becomes a caller-supplied bootstrap carrier.
pub fn publisher_bootstrap_authorization_sha256_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_publisher_bootstrap_authorization_v1(authorization)?,
    )))
}

pub fn canonical_publisher_bootstrap_core_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<Vec<u8>> {
    let mut value =
        serde_json::to_value(authorization).context("serialize bootstrap authorization")?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("bootstrap authorization must serialize to an object"))?;
    object.insert("test_retirement_commitment".to_string(), Value::Null);
    canonical_json_value_to_vec(&value).context("encode bootstrap authorization core")
}

pub fn canonical_managed_action_prepared_record_v1(
    record: &ManagedActionPreparedRecordV1,
) -> Result<Vec<u8>> {
    validate_prepared_record_v1(record)?;
    canonical_json_to_vec(record).context("encode action prepared record")
}

pub fn managed_action_prepared_record_sha256_v1(
    record: &ManagedActionPreparedRecordV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_managed_action_prepared_record_v1(record)?,
    )))
}

pub fn canonical_lifecycle_publisher_protected_state_v1(
    state: &LifecyclePublisherProtectedStateV1,
) -> Result<Vec<u8>> {
    validate_lifecycle_publisher_protected_state_v1(state)?;
    canonical_json_to_vec(state).context("encode lifecycle protected state")
}

pub fn lifecycle_anchor_sha256_v1(anchor: &LifecyclePublisherAnchorV1) -> Result<String> {
    validate_lifecycle_anchor_v1(anchor)?;
    Ok(lower_hex(&Sha256::digest(
        canonical_json_to_vec(anchor).context("encode lifecycle anchor")?,
    )))
}

pub fn validate_lifecycle_publisher_protected_state_v1(
    state: &LifecyclePublisherProtectedStateV1,
) -> Result<()> {
    require_schema(
        &state.schema_owner,
        state.schema_version,
        LIFECYCLE_PUBLISHER_PROTECTED_STATE_OWNER_V1,
        1,
    )?;
    validate_lifecycle_anchor_v1(&state.current_anchor)?;
    if let Some(record) = &state.prepared_record {
        validate_prepared_record_v1(record)?;
        if record.authority_domain != state.current_anchor.authority_domain {
            bail!("prepared record authority_domain must match the protected-state anchor");
        }
        if record.scope_id != state.current_anchor.scope_id {
            bail!("prepared record scope_id must match the protected-state anchor");
        }
        if record.manifest_generation != state.current_anchor.manifest_generation {
            bail!("prepared record manifest_generation must match the protected-state anchor");
        }
        if record.manifest_sha256 != state.current_anchor.manifest_sha256 {
            bail!("prepared record manifest_sha256 must match the protected-state anchor");
        }
        if record.signature.algorithm != state.current_anchor.signature.algorithm
            || record.signature.public_key != state.current_anchor.signature.public_key
        {
            bail!("prepared record signer must match the protected-state anchor signer");
        }
        if record.allocated_counter != state.counter {
            bail!("prepared record counter must match protected state counter");
        }
    }
    if let Some(previous) = &state.previous_protected_state_sha256 {
        require_hex_digest(previous, "previous_protected_state_sha256")?;
    }
    Ok(())
}

pub fn canonical_action_receipt_bytes_v1(receipt: &ManagedActionReceiptV1) -> Result<Vec<u8>> {
    validate_action_receipt_v1(receipt)?;
    canonical_json_to_vec(receipt).context("encode action receipt")
}

pub fn canonical_managed_action_receipt_signature_payload_v1(
    receipt: &ManagedActionReceiptV1,
) -> Result<Vec<u8>> {
    canonical_lifecycle_signature_payload_v1(&receipt.schema_owner, receipt)
}

pub fn validate_managed_action_receipt_signature_v1(
    receipt: &ManagedActionReceiptV1,
) -> Result<()> {
    validate_action_receipt_v1(receipt)?;
    verify_lifecycle_signature_v1(&receipt.schema_owner, receipt, &receipt.signature)
}

pub fn sign_managed_action_receipt_for_test_v1(receipt: &mut ManagedActionReceiptV1) -> Result<()> {
    let signing_key = Ed25519SigningKey::from_bytes(&[9_u8; 32]);
    receipt.signature.algorithm = "ed25519-v1".to_string();
    receipt.signature.public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes());
    receipt.signature.signature.clear();
    let payload = canonical_managed_action_receipt_signature_payload_v1(receipt)?;
    receipt.signature.signature = URL_SAFE_NO_PAD.encode(signing_key.sign(&payload).to_bytes());
    Ok(())
}

pub fn sign_managed_action_prepared_record_for_test_v1(
    record: &mut ManagedActionPreparedRecordV1,
) -> Result<()> {
    let signing_key = Ed25519SigningKey::from_bytes(&[9_u8; 32]);
    record.signature.algorithm = "ed25519-v1".to_string();
    record.signature.public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes());
    record.signature.signature.clear();
    let payload = canonical_lifecycle_signature_payload_v1(&record.schema_owner, record)
        .with_context(|| "encode prepared-record signature payload")?;
    record.signature.signature = URL_SAFE_NO_PAD.encode(signing_key.sign(&payload).to_bytes());
    Ok(())
}

pub fn sign_lifecycle_anchor_for_test_v1(anchor: &mut LifecyclePublisherAnchorV1) -> Result<()> {
    let signing_key = Ed25519SigningKey::from_bytes(&[9_u8; 32]);
    anchor.signature.algorithm = "ed25519-v1".to_string();
    anchor.signature.public_key = URL_SAFE_NO_PAD.encode(signing_key.verifying_key().as_bytes());
    anchor.signature.signature.clear();
    let payload = canonical_lifecycle_signature_payload_v1(&anchor.schema_owner, anchor)
        .with_context(|| "encode lifecycle-anchor signature payload")?;
    anchor.signature.signature = URL_SAFE_NO_PAD.encode(signing_key.sign(&payload).to_bytes());
    Ok(())
}

pub fn action_receipt_artifact_sha256_v1(receipt_bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(receipt_bytes))
}

pub fn canonical_action_receipt_index_bytes_v1(
    index: &ManagedActionReceiptIndexV1,
) -> Result<Vec<u8>> {
    validate_action_receipt_index_v1(index)?;
    canonical_json_to_vec(index).context("encode action receipt index")
}

/// Encode the durable manifest head used by the signed publisher anchor.
pub fn canonical_managed_manifest_head_v1(head: &ManagedManifestHeadV1) -> Result<Vec<u8>> {
    validate_manifest_head_v1(head)?;
    canonical_json_to_vec(head).context("encode managed manifest head")
}

pub fn compare_and_swap_action_receipt_index_v1(
    current: &ManagedActionReceiptIndexV1,
    next: &ManagedActionReceiptIndexV1,
) -> Result<()> {
    validate_action_receipt_index_v1(current)?;
    validate_action_receipt_index_v1(next)?;
    if current.authority_domain != next.authority_domain
        || current.scope_id != next.scope_id
        || current.manifest_generation != next.manifest_generation
        || current.manifest_sha256 != next.manifest_sha256
    {
        bail!("receipt index compare-and-swap scope mismatch");
    }
    if next.index_revision != current.index_revision + 1 {
        bail!("receipt index revision must advance by exactly one");
    }
    let current_sha = lower_hex(&Sha256::digest(canonical_action_receipt_index_bytes_v1(
        current,
    )?));
    if next.previous_index_sha256.as_deref() != Some(current_sha.as_str()) {
        bail!("receipt index compare-and-swap previous digest mismatch");
    }
    if next.entries.len() != current.entries.len() + 1 {
        bail!("receipt index updates must be append-only");
    }
    if next.entries[..current.entries.len()] != current.entries {
        bail!("receipt index updates must preserve prior entries exactly");
    }
    let mut seen = BTreeSet::new();
    for entry in &next.entries {
        if !seen.insert(entry.receipt_id.clone()) {
            bail!("receipt index contains duplicate receipt IDs");
        }
    }
    Ok(())
}

pub fn commit_action_receipt_index_to_head_v1(
    current_head: &ManagedManifestHeadV1,
    index: &ManagedActionReceiptIndexV1,
) -> Result<ManagedManifestHeadV1> {
    validate_manifest_head_v1(current_head)?;
    validate_action_receipt_index_v1(index)?;
    if current_head.scope_id != index.scope_id
        || current_head.manifest_generation != index.manifest_generation
        || current_head.manifest_sha256 != index.manifest_sha256
    {
        bail!("manifest head and action receipt index scope mismatch");
    }
    let index_sha = lower_hex(&Sha256::digest(canonical_action_receipt_index_bytes_v1(
        index,
    )?));
    Ok(ManagedManifestHeadV1 {
        schema_owner: current_head.schema_owner.clone(),
        schema_version: current_head.schema_version,
        scope_id: current_head.scope_id.clone(),
        manifest_generation: current_head.manifest_generation,
        manifest_sha256: current_head.manifest_sha256.clone(),
        lifecycle_state: current_head.lifecycle_state,
        previous_head_sha256: Some(lower_hex(&Sha256::digest(
            canonical_json_to_vec(current_head).context("encode current head")?,
        ))),
        action_receipt_index_revision: index.index_revision,
        action_receipt_index_sha256: index_sha,
    })
}

pub fn retirement_receipt_artifact_sha256_v1(receipt_bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(receipt_bytes))
}

pub fn validate_publisher_test_retirement_acknowledgement_v1(
    acknowledgement: &PublisherTestRetirementAcknowledgementV1,
) -> Result<()> {
    require_hex_digest(
        &acknowledgement.receipt_artifact_sha256,
        "publisher retirement acknowledgement receipt_artifact_sha256",
    )?;
    verify_lifecycle_signature_v1(
        &acknowledgement.schema_owner,
        acknowledgement,
        &acknowledgement.signature,
    )
}

pub fn canonical_guest_publisher_test_retirement_ticket_core_v1(
    ticket: &GuestPublisherPairingTicketV1,
) -> Result<Vec<u8>> {
    let mut value = serde_json::to_value(ticket).context("serialize pairing ticket")?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("pairing ticket must serialize to an object"))?;
    object.insert("signature".to_string(), Value::Null);
    canonical_json_value_to_vec(&value).context("encode pairing ticket core")
}

pub fn canonical_guest_publisher_reservation_unused_proof_v1(
    proof: &GuestPublisherReservationUnusedProofV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_reservation_unused_proof_v1(proof)?;
    canonical_json_to_vec(proof).context("encode unused proof")
}

pub fn validate_guest_publisher_reservation_unused_proof_v1(
    proof: &GuestPublisherReservationUnusedProofV1,
) -> Result<()> {
    require_hex_digest(&proof.ticket_sha256, "unused proof ticket_sha256")?;
    require_hex_digest(&proof.reservation_sha256, "unused proof reservation_sha256")?;
    verify_lifecycle_signature_v1(&proof.schema_owner, proof, &proof.signature)
}

pub fn guest_publisher_reservation_unused_proof_artifact_sha256_v1(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}

pub fn canonical_guest_publisher_reservation_unused_acknowledgement_v1(
    acknowledgement: &GuestPublisherReservationUnusedAcknowledgementV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_reservation_unused_acknowledgement_v1(acknowledgement)?;
    canonical_json_to_vec(acknowledgement).context("encode unused acknowledgement")
}

pub fn validate_guest_publisher_reservation_unused_acknowledgement_v1(
    acknowledgement: &GuestPublisherReservationUnusedAcknowledgementV1,
) -> Result<()> {
    require_hex_digest(
        &acknowledgement.proof_artifact_sha256,
        "unused acknowledgement proof_artifact_sha256",
    )?;
    verify_lifecycle_signature_v1(
        &acknowledgement.schema_owner,
        acknowledgement,
        &acknowledgement.signature,
    )
}

pub fn guest_publisher_reservation_unused_acknowledgement_artifact_sha256_v1(
    bytes: &[u8],
) -> String {
    lower_hex(&Sha256::digest(bytes))
}

pub fn validate_guest_publisher_test_retirement_authorization_v1(
    authorization: &GuestPublisherTestRetirementAuthorizationV1,
) -> Result<()> {
    require_hex_digest(
        &authorization.ticket_sha256,
        "guest retirement authorization ticket_sha256",
    )?;
    require_hex_digest(
        &authorization.commitment.authorization_digest,
        "guest retirement authorization digest",
    )?;
    verify_lifecycle_signature_v1(
        &authorization.schema_owner,
        authorization,
        &authorization.signature,
    )
}

pub fn validate_guest_publisher_test_retirement_acknowledgement_v1(
    acknowledgement: &GuestPublisherTestRetirementAcknowledgementV1,
) -> Result<()> {
    require_hex_digest(
        &acknowledgement.receipt_artifact_sha256,
        "guest retirement acknowledgement receipt_artifact_sha256",
    )?;
    verify_lifecycle_signature_v1(
        &acknowledgement.schema_owner,
        acknowledgement,
        &acknowledgement.signature,
    )
}

pub fn canonical_guest_publisher_pairing_ticket_v1(
    ticket: &GuestPublisherPairingTicketV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_pairing_ticket_v1(ticket)?;
    canonical_json_to_vec(ticket).context("encode guest pairing ticket")
}

/// Return the CodeDirectory hash embedded in one of the two closed macOS control requirements.
///
/// An ad-hoc evidence control has no Apple signing anchor, so it is admitted only by its measured
/// CodeDirectory hash.  A production control must additionally carry the fixed publisher
/// identifier and the Apple generic anchor.  Callers that have the opened-image provenance must
/// join the returned hash to its separately measured `code_identity`; this parser deliberately
/// does not accept a generic identifier, a caller supplied clause, or any other codesign text.
pub fn mac_publisher_control_requirement_cdhash_v1(requirement: &str) -> Result<&str> {
    const AD_HOC_PREFIX: &str = "cdhash H\"";
    const APPLE_PUBLISHER_PREFIX: &str =
        "anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\" and cdhash H\"";

    let cdhash = requirement
        .strip_prefix(AD_HOC_PREFIX)
        .or_else(|| requirement.strip_prefix(APPLE_PUBLISHER_PREFIX))
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| {
            anyhow!(
                "macOS control authority must use the exact CDHash or Apple-publisher-and-CDHash requirement form"
            )
        })?;
    if cdhash.len() != 40
        || !cdhash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("macOS control authority has an invalid fixed CodeDirectory hash");
    }
    Ok(cdhash)
}

/// Validate the one fixed macOS control identity committed before XPC service startup.
pub fn validate_mac_publisher_control_authority_v1(
    authority: &MacPublisherControlAuthorityV1,
) -> Result<()> {
    require_schema(
        &authority.schema_owner,
        authority.schema_version,
        MAC_PUBLISHER_CONTROL_AUTHORITY_OWNER_V1,
        1,
    )?;
    if authority.control_binary != "substrate-lifecycle-control" {
        bail!("macOS control authority must name the fixed substrate-lifecycle-control binary");
    }
    require_git_oid(
        &authority.source_commit,
        "macOS control authority source_commit",
    )?;
    require_git_oid(
        &authority.source_tree,
        "macOS control authority source_tree",
    )?;
    require_nonempty_no_nul(&authority.source_ref, "macOS control authority source_ref")?;
    if authority.target_triple != "aarch64-apple-darwin" {
        bail!("macOS control authority target_triple must be aarch64-apple-darwin");
    }
    require_hex_digest(
        &authority.artifact_sha256,
        "macOS control authority artifact_sha256",
    )?;
    require_nonempty_no_nul(
        &authority.designated_requirement,
        "macOS control authority designated_requirement",
    )?;
    if authority.designated_requirement.len() > 4096 {
        bail!("macOS control authority designated_requirement is too long");
    }
    let _ = mac_publisher_control_requirement_cdhash_v1(&authority.designated_requirement)?;
    Ok(())
}

/// Encode the fixed macOS XPC-control admission record as canonical JSON.
pub fn canonical_mac_publisher_control_authority_v1(
    authority: &MacPublisherControlAuthorityV1,
) -> Result<Vec<u8>> {
    validate_mac_publisher_control_authority_v1(authority)?;
    canonical_json_to_vec(authority).context("encode macOS publisher control authority")
}

/// Hash canonical macOS XPC-control admission bytes for exact-retry comparison.
pub fn mac_publisher_control_authority_sha256_v1(
    authority: &MacPublisherControlAuthorityV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_mac_publisher_control_authority_v1(authority)?,
    )))
}

/// Validate an opened-image provenance projection without accepting a path or a caller-selected
/// transport.  The control/executor independently remeasure these values through no-follow file
/// descriptors before they use the record.
pub fn validate_mac_publisher_bootstrap_image_provenance_v1(
    image: &MacPublisherBootstrapImageProvenanceV1,
) -> Result<()> {
    require_nonempty_no_nul(&image.target_triple, "macOS bootstrap image target_triple")?;
    require_hex_digest(
        &image.artifact_sha256,
        "macOS bootstrap image artifact_sha256",
    )?;
    require_nonempty_no_nul(
        &image.physical_identity,
        "macOS bootstrap image physical_identity",
    )?;
    if !image
        .code_identity
        .strip_prefix("cdhash:")
        .is_some_and(|value| {
            value.len() == 40
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    {
        bail!("macOS bootstrap image code_identity must be a canonical CDHash");
    }
    require_nonempty_no_nul(
        &image.code_requirement,
        "macOS bootstrap image code_requirement",
    )?;
    Ok(())
}

/// Validate the one closed, installer-owned retained Linux bundle.  Keep this table literal:
/// accepting a prefix, a caller-provided list, or a package discovery result here would recreate
/// the artifact-selection surface this record is intended to close.
pub fn validate_mac_lima_retained_artifact_provenance_v1(
    artifact: &MacLimaRetainedArtifactProvenanceV1,
    expected_role: &str,
    expected_relative_path: &str,
    expected_package: &str,
    expected_binary: &str,
) -> Result<()> {
    if artifact.logical_role != expected_role
        || artifact.retained_relative_path != expected_relative_path
        || artifact.cargo_package != expected_package
        || artifact.cargo_binary != expected_binary
        || artifact.target_triple != "aarch64-unknown-linux-gnu"
        || artifact.mode != "0755"
    {
        bail!("macOS retained Linux artifact does not match the fixed role table");
    }
    require_hex_digest(
        &artifact.cargo_lock_sha256,
        "macOS retained Linux artifact Cargo.lock SHA-256",
    )?;
    require_hex_digest(
        &artifact.artifact_sha256,
        "macOS retained Linux artifact SHA-256",
    )?;
    if !artifact.toolchain.starts_with("rustc 1.89.0 ")
        || artifact.build_command != "cargo build --locked --offline --target aarch64-unknown-linux-gnu --release -p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service -p substrate-gateway --bin substrate-gateway -p substrate --bin substrate"
    {
        bail!("macOS retained Linux artifact does not record the fixed cross-build invocation");
    }
    require_nonempty_no_nul(
        &artifact.physical_identity,
        "macOS retained Linux artifact physical identity",
    )?;
    if artifact.retained_relative_path.starts_with('/')
        || artifact.retained_relative_path.contains(['\0', '\n', '\r'])
        || artifact
            .retained_relative_path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        bail!("macOS retained Linux artifact path is not a normalized relative path");
    }
    Ok(())
}

/// Validate the fixed retained MAC bootstrap provenance record.
pub fn validate_mac_publisher_install_provenance_v1(
    provenance: &MacPublisherInstallProvenanceV1,
) -> Result<()> {
    require_schema(
        &provenance.schema_owner,
        provenance.schema_version,
        MAC_PUBLISHER_INSTALL_PROVENANCE_OWNER_V1,
        2,
    )?;
    require_git_oid(
        &provenance.source_commit,
        "macOS bootstrap provenance source_commit",
    )?;
    require_git_oid(
        &provenance.source_tree,
        "macOS bootstrap provenance source_tree",
    )?;
    require_nonempty_no_nul(
        &provenance.source_ref,
        "macOS bootstrap provenance source_ref",
    )?;
    require_hex_digest(
        &provenance.review_record_sha256,
        "macOS bootstrap provenance review_record_sha256",
    )?;
    require_hex_digest(
        &provenance.host_context_commitment,
        "macOS bootstrap provenance host_context_commitment",
    )?;
    require_nonempty_no_nul(
        &provenance.selected_host_prefix,
        "macOS install provenance selected_host_prefix",
    )?;
    if !provenance.selected_host_prefix.starts_with('/') {
        bail!("macOS install provenance selected_host_prefix must be absolute");
    }
    validate_mac_publisher_control_authority_v1(&provenance.control_authority)?;
    if provenance.control_authority.source_commit != provenance.source_commit
        || provenance.control_authority.source_tree != provenance.source_tree
        || provenance.control_authority.source_ref != provenance.source_ref
    {
        bail!("macOS bootstrap provenance control authority does not join source identity");
    }
    validate_mac_publisher_bootstrap_image_provenance_v1(&provenance.control_image)?;
    validate_mac_publisher_bootstrap_image_provenance_v1(&provenance.executor_image)?;
    let control_requirement_cdhash = mac_publisher_control_requirement_cdhash_v1(
        &provenance.control_authority.designated_requirement,
    )?;
    let control_image_cdhash = provenance
        .control_image
        .code_identity
        .strip_prefix("cdhash:")
        .ok_or_else(|| anyhow!("macOS control image has no canonical CodeDirectory hash"))?;
    if provenance.control_image.target_triple != provenance.control_authority.target_triple
        || provenance.control_image.artifact_sha256 != provenance.control_authority.artifact_sha256
        || provenance.control_image.code_requirement
            != provenance.control_authority.designated_requirement
        || control_requirement_cdhash != control_image_cdhash
        || provenance.executor_image.target_triple != provenance.control_authority.target_triple
        || provenance.lima_tool.image.target_triple != provenance.control_authority.target_triple
    {
        bail!("macOS bootstrap provenance image identities do not join control authority");
    }
    if provenance.control_image.artifact_sha256 == provenance.executor_image.artifact_sha256
        || provenance.control_image.physical_identity == provenance.executor_image.physical_identity
        || provenance.control_image.code_requirement == provenance.executor_image.code_requirement
    {
        bail!("macOS bootstrap provenance must retain independently measured control and executor images");
    }
    require_hex_digest(
        &provenance.launch_daemon_plist_sha256,
        "macOS install provenance launch_daemon_plist_sha256",
    )?;
    require_nonempty_no_nul(
        &provenance.lima_tool.absolute_path,
        "macOS install provenance limactl absolute_path",
    )?;
    if !provenance.lima_tool.absolute_path.starts_with('/')
        || provenance
            .lima_tool
            .absolute_path
            .contains(['\0', '\n', '\r'])
    {
        bail!("macOS install provenance limactl path must be absolute");
    }
    validate_mac_publisher_bootstrap_image_provenance_v1(&provenance.lima_tool.image)?;
    require_nonempty_no_nul(
        &provenance.lima_tool.version,
        "macOS install provenance limactl version",
    )?;
    let expected_artifacts = [
        (
            "mac.lima.publisher-executor",
            "bin/linux/substrate-lifecycle-linux",
            "substrate",
            "substrate-lifecycle-linux",
        ),
        (
            "mac.lima.guest-binary(substrate-world-service)",
            "bin/linux/world-service",
            "world-service",
            "world-service",
        ),
        (
            "mac.lima.guest-binary(substrate-gateway)",
            "bin/linux/substrate-gateway",
            "substrate-gateway",
            "substrate-gateway",
        ),
        (
            "mac.lima.guest-binary(substrate)",
            "bin/linux/substrate",
            "substrate",
            "substrate",
        ),
    ];
    if provenance.retained_linux_artifacts.len() != expected_artifacts.len() {
        bail!("macOS install provenance must retain exactly four Linux artifacts");
    }
    for (artifact, (role, relative_path, package, binary)) in provenance
        .retained_linux_artifacts
        .iter()
        .zip(expected_artifacts)
    {
        validate_mac_lima_retained_artifact_provenance_v1(
            artifact,
            role,
            relative_path,
            package,
            binary,
        )?;
    }
    if provenance.profile_template_algorithm != "substrate.mac-lima-stage-one-profile-template"
        || provenance.profile_template_version != 1
    {
        bail!("macOS install provenance profile template algorithm/version mismatch");
    }
    require_hex_digest(
        &provenance.profile_template_sha256,
        "macOS install provenance profile_template_sha256",
    )?;
    Ok(())
}

pub fn canonical_mac_publisher_install_provenance_v1(
    provenance: &MacPublisherInstallProvenanceV1,
) -> Result<Vec<u8>> {
    validate_mac_publisher_install_provenance_v1(provenance)?;
    canonical_json_to_vec(provenance).context("encode macOS install provenance")
}

pub fn validate_mac_publisher_control_admission_v1(
    admission: &MacPublisherControlAdmissionV1,
) -> Result<()> {
    require_schema(
        &admission.schema_owner,
        admission.schema_version,
        MAC_PUBLISHER_CONTROL_ADMISSION_OWNER_V1,
        1,
    )?;
    require_uuid_v7(&admission.scope_id, "macOS control admission scope_id")?;
    validate_mac_publisher_control_authority_v1(&admission.control_authority)?;
    require_hex_digest(
        &admission.bootstrap_authorization_sha256,
        "macOS control admission bootstrap_authorization_sha256",
    )?;
    if admission.manifest_generation == 0 {
        bail!("macOS control admission manifest_generation must be positive");
    }
    require_hex_digest(
        &admission.manifest_sha256,
        "macOS control admission manifest_sha256",
    )?;
    require_hex_digest(
        &admission.current_anchor_sha256,
        "macOS control admission current_anchor_sha256",
    )?;
    Ok(())
}

pub fn canonical_mac_publisher_control_admission_v1(
    admission: &MacPublisherControlAdmissionV1,
) -> Result<Vec<u8>> {
    validate_mac_publisher_control_admission_v1(admission)?;
    canonical_json_to_vec(admission).context("encode macOS control admission")
}

pub fn validate_guest_publisher_pairing_ticket_v1(
    ticket: &GuestPublisherPairingTicketV1,
) -> Result<()> {
    require_schema(
        &ticket.schema_owner,
        ticket.schema_version,
        "substrate.guest-publisher-pairing-ticket",
        1,
    )?;
    validate_pairing_challenge_v1(&ticket.challenge)?;
    validate_lifecycle_anchor_v1(&ticket.current_anchor)?;
    let signer_der = decode_base64url(&ticket.signer_spki_der, "pairing ticket signer_spki_der")?;
    let parsed_der = parse_p256_spki_der_v1(&signer_der)?;
    let signer_hash = lower_hex(&Sha256::digest(&parsed_der));
    if signer_hash != ticket.challenge.host_key_fingerprint_sha256 {
        bail!("pairing ticket signer fingerprint does not match challenge");
    }
    let anchor_bytes =
        canonical_json_to_vec(&ticket.current_anchor).context("encode current anchor")?;
    let anchor_sha = lower_hex(&Sha256::digest(anchor_bytes));
    if anchor_sha != ticket.current_anchor_sha256
        || anchor_sha != ticket.challenge.current_anchor_sha256
    {
        bail!("pairing ticket current anchor digest mismatch");
    }
    let challenge_sha = lower_hex(&Sha256::digest(
        canonical_json_to_vec(&ticket.challenge).context("encode pairing challenge")?,
    ));
    if challenge_sha != ticket.challenge_sha256 {
        bail!("pairing ticket challenge digest mismatch");
    }
    if ticket.challenge.source_commit != ticket.current_anchor.executor_identity.source_commit
        || ticket.challenge.source_tree != ticket.current_anchor.executor_identity.source_tree
        || ticket.challenge.source_ref != ticket.current_anchor.executor_identity.source_ref
        || ticket.challenge.executor_build_evidence_sha256
            != ticket.current_anchor.executor_identity.artifact_sha256
    {
        bail!("pairing ticket source or artifact binding does not match current anchor");
    }
    if ticket.challenge.host_context_commitment != ticket.current_anchor.host_context_commitment
        || ticket.challenge.platform_mapping_commitment
            != ticket.current_anchor.platform_mapping_commitment
    {
        bail!("pairing ticket host binding does not match current anchor");
    }
    if ticket.current_anchor.authority_domain == "mac_host_shared" {
        if ticket.host_generation != ticket.current_anchor.manifest_generation {
            bail!("macOS pairing ticket host generation does not match current anchor");
        }
        if ticket.current_anchor.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
            || ticket.current_anchor.signature.public_key != ticket.signer_spki_der
            || ticket.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
            || ticket.signature.public_key != ticket.signer_spki_der
        {
            bail!("macOS pairing ticket signatures must use the canonical ticket P-256 SPKI");
        }
    }
    verify_lifecycle_signature_v1(&ticket.schema_owner, ticket, &ticket.signature)
}

/// Validate a pairing ticket against an explicit clock so callers must check expiry before use.
pub fn validate_guest_publisher_pairing_ticket_at_v1(
    ticket: &GuestPublisherPairingTicketV1,
    now_unix_ns: u64,
) -> Result<()> {
    validate_guest_publisher_pairing_ticket_v1(ticket)?;
    if now_unix_ns >= ticket.challenge.expires_at_unix_ns {
        bail!("pairing ticket has expired");
    }
    Ok(())
}

/// Validate the immutable authority shared by the two PM-bound R6 sessions.
pub fn validate_guest_publisher_pairing_session_binding_v1(
    binding: &GuestPublisherPairingSessionBindingV1,
) -> Result<()> {
    require_schema(
        &binding.schema_owner,
        binding.schema_version,
        GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_OWNER_V1,
        GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_VERSION_V1,
    )?;
    require_uuid_v7(&binding.scope_id, "pairing session binding scope_id")?;
    if let Some(commitment) = &binding.platform_mapping_commitment {
        require_hex_digest(
            commitment,
            "pairing session binding platform_mapping_commitment",
        )?;
    }
    require_nonempty_no_nul(
        &binding.guest_machine_identity,
        "pairing session binding guest_machine_identity",
    )?;
    require_git_oid(
        &binding.source_commit,
        "pairing session binding source_commit",
    )?;
    require_git_oid(&binding.source_tree, "pairing session binding source_tree")?;
    require_nonempty_no_nul(&binding.source_ref, "pairing session binding source_ref")?;
    require_hex_digest(
        &binding.staged_executor_sha256,
        "pairing session binding staged_executor_sha256",
    )?;
    require_hex_digest(
        &binding.stage_one_record_sha256,
        "pairing session binding stage_one_record_sha256",
    )?;
    require_uuid_v7(
        &binding.ticket_challenge_id,
        "pairing session binding ticket_challenge_id",
    )?;
    if binding.host_record_generation == 0 {
        bail!("pairing session binding host_record_generation must be positive");
    }
    require_hex_digest(
        &binding.pairing_session_nonce,
        "pairing session binding pairing_session_nonce",
    )
}

/// Canonical immutable shared authority bytes for R6 session admission.
pub fn canonical_guest_publisher_pairing_session_binding_v1(
    binding: &GuestPublisherPairingSessionBindingV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_pairing_session_binding_v1(binding)?;
    canonical_json_to_vec(binding).context("encode guest pairing session binding")
}

fn validate_pairing_session_binding_against_ticket_v1(
    binding: &GuestPublisherPairingSessionBindingV1,
    ticket: &GuestPublisherPairingTicketV1,
) -> Result<()> {
    validate_guest_publisher_pairing_session_binding_v1(binding)?;
    if binding.scope_id != ticket.current_anchor.scope_id
        || binding.platform_mapping_commitment != ticket.challenge.platform_mapping_commitment
        || binding.guest_machine_identity != ticket.challenge.guest_machine_identity
        || binding.source_commit != ticket.challenge.source_commit
        || binding.source_tree != ticket.challenge.source_tree
        || binding.source_ref != ticket.challenge.source_ref
        || binding.staged_executor_sha256 != ticket.challenge.executor_build_evidence_sha256
        || binding.ticket_challenge_id != ticket.challenge.challenge_id
        || binding.host_record_generation != ticket.host_generation
    {
        bail!("pairing session binding does not exactly match the signed ticket");
    }
    Ok(())
}

fn require_pairing_operator_session_id_v1(value: &str, field: &str) -> Result<()> {
    require_hex_digest(value, field)
}

fn require_pairing_absolute_path_v1(value: &str, field: &str) -> Result<()> {
    require_nonempty_no_nul(value, field)?;
    if !value.starts_with('/') || value.contains(['\n', '\r']) {
        bail!("{field} must be an absolute path without line breaks");
    }
    Ok(())
}

/// Validate every selector-free launch field before the opaque signature is assigned.
///
/// This private split lets the producer derive the dedicated signature payload with an empty
/// `signature` field, while public acceptance still requires a valid P-256 P1363 envelope.
fn validate_guest_publisher_pairing_operator_launch_fields_v1(
    launch: &GuestPublisherPairingOperatorLaunchV1,
) -> Result<()> {
    require_schema(
        &launch.schema_owner,
        launch.schema_version,
        GUEST_PUBLISHER_PAIRING_OPERATOR_LAUNCH_SCHEMA_OWNER_V1,
        GUEST_PUBLISHER_PAIRING_OPERATOR_LAUNCH_SCHEMA_VERSION_V1,
    )?;
    validate_guest_publisher_pairing_session_binding_v1(&launch.binding)?;
    require_pairing_operator_session_id_v1(
        &launch.operator_session_id,
        "pairing operator launch operator_session_id",
    )?;
    require_nonempty_no_nul(
        &launch.admitted_instance_name,
        "pairing operator launch admitted_instance_name",
    )?;
    if launch.admitted_instance_name.contains(['\n', '\r']) {
        bail!("pairing operator launch admitted_instance_name must not contain line breaks");
    }
    if launch.expires_at_unix_ns == 0 {
        bail!("pairing operator launch expiry must be positive");
    }
    require_pairing_absolute_path_v1(
        &launch.limactl_absolute_path,
        "pairing operator launch limactl_absolute_path",
    )?;
    require_hex_digest(
        &launch.limactl_sha256,
        "pairing operator launch limactl_sha256",
    )?;
    if launch.guest_executable_path != GUEST_PUBLISHER_PAIRING_OPERATOR_GUEST_EXECUTABLE_PATH_V1 {
        bail!("pairing operator launch guest executable path is not fixed");
    }
    require_hex_digest(
        &launch.guest_executable_sha256,
        "pairing operator launch guest_executable_sha256",
    )?;
    if launch.guest_executable_sha256 != launch.binding.staged_executor_sha256 {
        bail!("pairing operator launch guest executable digest does not match binding");
    }
    if launch.fixed_operator_command != GUEST_PUBLISHER_PAIRING_OPERATOR_FIXED_COMMAND_V1 {
        bail!("pairing operator launch command is not fixed");
    }
    Ok(())
}

/// Validate the structural, selector-free content of a signed direct-operator launch.
///
/// This cannot establish authority by itself: callers must use
/// [`validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1`] with the separately
/// retained signed ticket.
pub fn validate_guest_publisher_pairing_operator_launch_v1(
    launch: &GuestPublisherPairingOperatorLaunchV1,
) -> Result<()> {
    validate_guest_publisher_pairing_operator_launch_fields_v1(launch)?;
    let signature = decode_base64url(&launch.signature, "pairing operator launch signature")?;
    if signature.len() != 64 {
        bail!("pairing operator launch signature must be P-256 P1363 bytes");
    }
    Ok(())
}

/// Canonical bytes for the signed direct-operator launch envelope.
pub fn canonical_guest_publisher_pairing_operator_launch_v1(
    launch: &GuestPublisherPairingOperatorLaunchV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_pairing_operator_launch_v1(launch)?;
    canonical_json_to_vec(launch).context("encode guest pairing operator launch")
}

/// Digest the exact canonical signed operator launch admitted by the protected host record.
///
/// This is digest-only binding data: it intentionally retains no launch path, ticket, selector,
/// or authority material outside the signed launch itself.
pub fn guest_publisher_pairing_operator_launch_sha256_v1(
    launch: &GuestPublisherPairingOperatorLaunchV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_guest_publisher_pairing_operator_launch_v1(launch)?,
    )))
}

/// Canonical signature payload for the direct-operator launch.  The opaque `signature` field is
/// excluded by the common lifecycle-signature framing, while all binding metadata remains covered.
pub fn canonical_guest_publisher_pairing_operator_launch_signature_payload_v1(
    launch: &GuestPublisherPairingOperatorLaunchV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_pairing_operator_launch_fields_v1(launch)?;
    canonical_lifecycle_signature_payload_v1(&launch.schema_owner, launch)
}

/// Decode only exact canonical JSON for a direct-operator launch.
pub fn parse_and_validate_guest_publisher_pairing_operator_launch_v1(
    bytes: &[u8],
) -> Result<GuestPublisherPairingOperatorLaunchV1> {
    let launch: GuestPublisherPairingOperatorLaunchV1 = canonical_json_from_slice(bytes)
        .context("guest pairing operator launch is not canonical v1 JSON")?;
    validate_guest_publisher_pairing_operator_launch_v1(&launch)?;
    Ok(launch)
}

/// Verify a direct-operator launch solely from the retained signed ticket.
///
/// This checks canonical structure, immutable ticket binding, expiry, and the ticket P-256 key.
/// The protected-state owner separately joins the immutable binding generation to its current
/// generation and digest before performing its host-record CAS.
pub fn validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1(
    ticket: &GuestPublisherPairingTicketV1,
    launch: &GuestPublisherPairingOperatorLaunchV1,
    now_unix_ns: u64,
) -> Result<()> {
    validate_guest_publisher_pairing_ticket_at_v1(ticket, now_unix_ns)?;
    validate_guest_publisher_pairing_operator_launch_v1(launch)?;
    validate_pairing_session_binding_against_ticket_v1(&launch.binding, ticket)?;
    if launch.expires_at_unix_ns != ticket.challenge.expires_at_unix_ns {
        bail!("pairing operator launch expiry does not match signed ticket");
    }
    let signer_der = decode_base64url(&ticket.signer_spki_der, "pairing ticket signer_spki_der")?;
    let signature = decode_base64url(&launch.signature, "pairing operator launch signature")?;
    verify_p256_p1363_low_s_v1(
        &signer_der,
        &canonical_guest_publisher_pairing_operator_launch_signature_payload_v1(launch)?,
        &signature,
    )
    .context("verify signed guest pairing operator launch")
}

/// Verify a launch against the signed ticket plus the exact protected record which admitted it.
///
/// New control-plane code that has only ticket and response bindings must use
/// [`validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1`] and then perform
/// its own protected-record generation-CAS acknowledgement.
pub fn validate_guest_publisher_pairing_operator_launch_against_ticket_and_host_record_at_v1(
    ticket: &GuestPublisherPairingTicketV1,
    record: &GuestPublisherPairingHostRecordV1,
    launch: &GuestPublisherPairingOperatorLaunchV1,
    now_unix_ns: u64,
) -> Result<()> {
    validate_guest_publisher_pairing_host_record_v1(record)?;
    validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1(
        ticket,
        launch,
        now_unix_ns,
    )?;
    if record.ticket != *ticket
        || record.binding != launch.binding
        || launch.operator_session_id != record.operator_tty_session.session_id
        || launch.operator_session_id == record.data_session.session_id
        || guest_publisher_pairing_operator_launch_sha256_v1(launch)?
            != record.operator_launch_sha256
        || record.record_generation < launch.binding.host_record_generation
    {
        bail!("pairing operator launch does not exact-join ticket and protected record");
    }
    Ok(())
}

/// Validate one immutable proof before its canonical digest is admitted to the host record.
pub fn validate_guest_publisher_pairing_operator_proof_v1(
    proof: &GuestPublisherPairingOperatorProofV1,
) -> Result<()> {
    require_schema(
        &proof.schema_owner,
        proof.schema_version,
        GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_SCHEMA_OWNER_V1,
        GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_SCHEMA_VERSION_V1,
    )?;
    validate_guest_publisher_pairing_session_binding_v1(&proof.binding)?;
    require_pairing_operator_session_id_v1(
        &proof.operator_session_id,
        "pairing operator proof operator_session_id",
    )?;
    require_hex_digest(
        &proof.operator_launch_sha256,
        "pairing operator proof operator_launch_sha256",
    )?;
    require_hex_digest(
        &proof.confirmation_commitment,
        "pairing operator proof confirmation_commitment",
    )?;
    if proof.created_at_unix_ns == 0 || proof.created_at_unix_ns >= proof.expires_at_unix_ns {
        bail!("pairing operator proof timestamps are invalid");
    }
    if proof.terminal_observation != GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_TERMINAL_OBSERVATION_V1
    {
        bail!("pairing operator proof terminal observation is not fixed");
    }
    Ok(())
}

/// Canonical bytes for the immutable guest operator proof.
pub fn canonical_guest_publisher_pairing_operator_proof_v1(
    proof: &GuestPublisherPairingOperatorProofV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_pairing_operator_proof_v1(proof)?;
    canonical_json_to_vec(proof).context("encode guest pairing operator proof")
}

/// Decode only exact canonical JSON for a guest operator proof.  The caller must obtain `bytes`
/// from its fixed no-follow descriptor before invoking this typed parser.
pub fn parse_and_validate_guest_publisher_pairing_operator_proof_v1(
    bytes: &[u8],
) -> Result<GuestPublisherPairingOperatorProofV1> {
    let proof: GuestPublisherPairingOperatorProofV1 = canonical_json_from_slice(bytes)
        .context("guest pairing operator proof is not canonical v1 JSON")?;
    validate_guest_publisher_pairing_operator_proof_v1(&proof)?;
    Ok(proof)
}

/// Digest the canonical operator proof for one host-record generation-CAS admission.
pub fn guest_publisher_pairing_operator_proof_sha256_v1(
    proof: &GuestPublisherPairingOperatorProofV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_guest_publisher_pairing_operator_proof_v1(proof)?,
    )))
}

/// Validate a no-follow-read proof against the exact ticket and host record before intent can
/// advance.  `expected_commitment` is a digest only; callers must not pass or retain input values.
pub fn validate_guest_publisher_pairing_operator_proof_against_ticket_at_v1(
    ticket: &GuestPublisherPairingTicketV1,
    record: &GuestPublisherPairingHostRecordV1,
    proof: &GuestPublisherPairingOperatorProofV1,
    expected_commitment: &str,
    now_unix_ns: u64,
) -> Result<()> {
    validate_guest_publisher_pairing_ticket_at_v1(ticket, now_unix_ns)?;
    validate_guest_publisher_pairing_host_record_v1(record)?;
    validate_guest_publisher_pairing_operator_proof_v1(proof)?;
    validate_pairing_session_binding_against_ticket_v1(&proof.binding, ticket)?;
    require_hex_digest(expected_commitment, "pairing operator expected commitment")?;
    if record.ticket != *ticket
        || record.binding != proof.binding
        || proof.operator_session_id != record.operator_tty_session.session_id
        || proof.operator_session_id == record.data_session.session_id
        || proof.operator_launch_sha256 != record.operator_launch_sha256
        || proof.expires_at_unix_ns != ticket.challenge.expires_at_unix_ns
        || proof.confirmation_commitment != expected_commitment
        || record.record_generation < proof.binding.host_record_generation
    {
        bail!("pairing operator proof does not exact-join ticket and protected record");
    }
    let proof_sha256 = guest_publisher_pairing_operator_proof_sha256_v1(proof)?;
    match &record.operator_proof_sha256 {
        Some(recorded) if recorded == &proof_sha256 => Ok(()),
        Some(_) => bail!("pairing operator proof conflicts with the recorded proof digest"),
        None if record.state == "sessions_opened" => Ok(()),
        None => bail!("pairing operator proof is not admissible in this record state"),
    }
}

fn validate_pairing_observation_v1(value: &str, field: &str) -> Result<()> {
    require_nonempty_no_nul(value, field)
}

fn validate_pairing_optional_observation_v1(value: &Option<String>, field: &str) -> Result<()> {
    if let Some(value) = value {
        validate_pairing_observation_v1(value, field)?;
    }
    Ok(())
}

/// Canonical digest of the signed guest Hello used by the R6 transcript binding.
pub fn guest_publisher_bootstrap_hello_sha256_v1(
    hello: &GuestPublisherBootstrapHelloV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_json_to_vec(hello).context("encode guest pairing Hello")?,
    )))
}

/// Digest the canonical raw guest public key carried by a signed R6 Hello.
pub fn guest_publisher_bootstrap_hello_public_key_sha256_v1(
    hello: &GuestPublisherBootstrapHelloV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(decode_base64url(
        &hello.guest_public_key,
        "guest pairing Hello key",
    )?)))
}

/// Verify that a guest Hello is a signed statement by the guest key admitted by the exact
/// PM-bound ticket and immutable dual-session binding.  The host calls this before it signs any
/// durable record transition, so a child cannot turn arbitrary JSON into host attestation.
pub fn validate_guest_publisher_bootstrap_hello_v1(
    ticket: &GuestPublisherPairingTicketV1,
    binding: &GuestPublisherPairingSessionBindingV1,
    hello: &GuestPublisherBootstrapHelloV1,
) -> Result<()> {
    validate_pairing_session_binding_against_ticket_v1(binding, ticket)?;
    require_schema(
        &hello.schema_owner,
        hello.schema_version,
        "substrate.guest-publisher-bootstrap-hello",
        1,
    )?;
    if hello.ticket_sha256
        != lower_hex(&Sha256::digest(
            canonical_guest_publisher_pairing_ticket_v1(ticket)?,
        ))
        || hello.guest_machine_identity != binding.guest_machine_identity
        || hello.guest_artifact_sha256 != binding.staged_executor_sha256
        || hello.nonce != binding.pairing_session_nonce
        || hello.intent_published_at_unix_ns >= ticket.challenge.expires_at_unix_ns
        || !hello.intent_parent_fsync_observed
    {
        bail!("guest pairing Hello does not exact-join the ticket and immutable binding");
    }
    require_hex_digest(&hello.intent_sha256, "guest pairing Hello intent_sha256")?;
    require_nonempty_no_nul(
        &hello.guest_public_key,
        "guest pairing Hello guest_public_key",
    )?;
    if hello.signature.algorithm != "ed25519-v1"
        || hello.signature.public_key != hello.guest_public_key
    {
        bail!("guest pairing Hello must use its admitted guest Ed25519 public key");
    }
    verify_lifecycle_signature_v1(&hello.schema_owner, hello, &hello.signature)
        .context("verify signed guest pairing Hello")
}

/// Verify the signed host transcript before it is persisted or used to advance the R6 guest
/// publisher state.  It binds the exact Hello, guest key, PM identity, and staged artifact.
pub fn validate_guest_publisher_bootstrap_transcript_v1(
    ticket: &GuestPublisherPairingTicketV1,
    binding: &GuestPublisherPairingSessionBindingV1,
    hello: &GuestPublisherBootstrapHelloV1,
    transcript: &GuestPublisherBootstrapTranscriptV1,
) -> Result<()> {
    validate_guest_publisher_bootstrap_hello_v1(ticket, binding, hello)?;
    require_schema(
        &transcript.schema_owner,
        transcript.schema_version,
        "substrate.guest-publisher-bootstrap-transcript",
        1,
    )?;
    if transcript.ticket_sha256
        != lower_hex(&Sha256::digest(
            canonical_guest_publisher_pairing_ticket_v1(ticket)?,
        ))
        || transcript.hello_sha256 != guest_publisher_bootstrap_hello_sha256_v1(hello)?
        || transcript.guest_machine_identity != binding.guest_machine_identity
        || transcript.staged_executor_sha256 != binding.staged_executor_sha256
        || transcript.guest_nonce != binding.pairing_session_nonce
        || transcript.guest_test_retirement_commitment != ticket.guest_test_retirement_commitment
    {
        bail!("guest pairing transcript does not exact-join ticket, Hello, and binding");
    }
    if transcript.guest_public_key_sha256
        != guest_publisher_bootstrap_hello_public_key_sha256_v1(hello)?
    {
        bail!("guest pairing transcript guest key digest does not match Hello");
    }
    if transcript.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || transcript.signature.public_key != ticket.signer_spki_der
    {
        bail!("guest pairing transcript must be signed by the retained host ticket key");
    }
    verify_lifecycle_signature_v1(&transcript.schema_owner, transcript, &transcript.signature)
        .context("verify signed guest pairing transcript")
}

/// Validate the only R6 session that may carry a signed ticket and typed guest frames.
pub fn validate_guest_publisher_pairing_data_session_v1(
    session: &GuestPublisherPairingDataSessionV1,
) -> Result<()> {
    require_schema(
        &session.schema_owner,
        session.schema_version,
        GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_OWNER_V1,
        GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_VERSION_V1,
    )?;
    validate_guest_publisher_pairing_ticket_v1(&session.ticket)?;
    validate_pairing_session_binding_against_ticket_v1(&session.binding, &session.ticket)?;
    require_nonempty_no_nul(&session.session_id, "pairing data session_id")?;
    validate_pairing_observation_v1(
        &session.started_observation,
        "pairing data started_observation",
    )?;
    validate_pairing_optional_observation_v1(
        &session.ended_observation,
        "pairing data ended_observation",
    )?;
    validate_pairing_optional_observation_v1(
        &session.failure_observation,
        "pairing data failure_observation",
    )?;
    match session.state.as_str() {
        "opened" | "guest_state_root_durable" => {
            if session.hello.is_some()
                || session.transcript.is_some()
                || session.guest_anchor_sha256.is_some()
                || session.ended_observation.is_some()
            {
                bail!("pairing data session state carries frames before its durable transition");
            }
        }
        "hello_durable" => {
            if session.hello.is_none()
                || session.transcript.is_some()
                || session.guest_anchor_sha256.is_some()
                || session.ended_observation.is_some()
            {
                bail!("pairing data hello_durable state has an invalid frame set");
            }
        }
        "transcript_durable" => {
            if session.hello.is_none()
                || session.transcript.is_none()
                || session.guest_anchor_sha256.is_some()
                || session.ended_observation.is_some()
            {
                bail!("pairing data transcript_durable state has an invalid frame set");
            }
        }
        "ticket_consumed" => {
            if session.hello.is_none()
                || session.transcript.is_none()
                || session.guest_anchor_sha256.is_none()
                || session.ended_observation.is_none()
            {
                bail!("pairing data ticket_consumed state requires all durable frames and close");
            }
            require_hex_digest(
                session
                    .guest_anchor_sha256
                    .as_deref()
                    .expect("checked is_some"),
                "pairing data guest_anchor_sha256",
            )?;
        }
        _ => bail!("unknown R6 pairing data session state"),
    }
    if let Some(hello) = &session.hello {
        validate_guest_publisher_bootstrap_hello_v1(&session.ticket, &session.binding, hello)?;
    }
    if let Some(transcript) = &session.transcript {
        let hello = session
            .hello
            .as_ref()
            .ok_or_else(|| anyhow!("pairing transcript lacks its signed Hello"))?;
        validate_guest_publisher_bootstrap_transcript_v1(
            &session.ticket,
            &session.binding,
            hello,
            transcript,
        )?;
    }
    Ok(())
}

/// Validate the independent guest controlling-TTY R6 session.
pub fn validate_guest_publisher_pairing_operator_tty_session_v1(
    session: &GuestPublisherPairingOperatorTtySessionV1,
) -> Result<()> {
    require_schema(
        &session.schema_owner,
        session.schema_version,
        GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_OWNER_V1,
        GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_VERSION_V1,
    )?;
    validate_guest_publisher_pairing_session_binding_v1(&session.binding)?;
    require_nonempty_no_nul(&session.session_id, "pairing operator TTY session_id")?;
    require_nonempty_no_nul(
        &session.controlling_tty_identity,
        "pairing operator TTY controlling_tty_identity",
    )?;
    validate_pairing_observation_v1(
        &session.started_observation,
        "pairing operator TTY started_observation",
    )?;
    validate_pairing_optional_observation_v1(
        &session.ended_observation,
        "pairing operator TTY ended_observation",
    )?;
    validate_pairing_optional_observation_v1(
        &session.failure_observation,
        "pairing operator TTY failure_observation",
    )?;
    match session.state.as_str() {
        "opened" => {
            if session.ended_observation.is_some() {
                bail!("opened operator TTY session must not carry close evidence");
            }
        }
        "closed" => {
            if session.ended_observation.is_none() {
                bail!("closed operator TTY session requires close evidence");
            }
        }
        _ => bail!("unknown R6 pairing operator TTY session state"),
    }
    Ok(())
}

/// Validate the protected R6 host record for the exact PM-bound data and operator-TTY sessions.
pub fn validate_guest_publisher_pairing_host_record_v1(
    record: &GuestPublisherPairingHostRecordV1,
) -> Result<()> {
    require_schema(
        &record.schema_owner,
        record.schema_version,
        "substrate.guest-publisher-pairing-host-record",
        1,
    )?;
    validate_guest_publisher_pairing_ticket_v1(&record.ticket)?;
    if record.ticket.current_anchor.authority_domain != "mac_host_shared" {
        bail!("R4 host pairing records require mac_host_shared authority");
    }
    if record.current_anchor_counter != record.ticket.host_counter
        || record.current_anchor_sha256 != record.ticket.current_anchor_sha256
    {
        bail!("host pairing record anchor binding does not match ticket");
    }
    if record.record_generation == 0 {
        bail!("host pairing record generation must be positive");
    }
    validate_pairing_session_binding_against_ticket_v1(&record.binding, &record.ticket)?;
    validate_guest_publisher_pairing_data_session_v1(&record.data_session)?;
    validate_guest_publisher_pairing_operator_tty_session_v1(&record.operator_tty_session)?;
    if record.data_session.binding != record.binding
        || record.operator_tty_session.binding != record.binding
        || record.data_session.ticket != record.ticket
    {
        bail!("host pairing record session binding or ticket mismatch");
    }
    if record.data_session.session_id == record.operator_tty_session.session_id {
        bail!("host pairing data and operator TTY session identities must be distinct");
    }
    require_hex_digest(
        &record.operator_launch_sha256,
        "host pairing record operator_launch_sha256",
    )?;
    if let Some(proof_sha256) = &record.operator_proof_sha256 {
        require_hex_digest(proof_sha256, "host pairing record operator_proof_sha256")?;
    }
    match record.state.as_str() {
        "sessions_opened" => {
            if record.data_session.state != "opened"
                || record.operator_tty_session.state != "opened"
                || record.operator_proof_sha256.is_some()
            {
                bail!("sessions_opened record has invalid session states");
            }
            let has_loss_observation = record.data_session.failure_observation.is_some()
                || record.operator_tty_session.failure_observation.is_some();
            if !has_loss_observation
                && (record.record_generation != record.binding.host_record_generation
                    || record.previous_record_sha256.is_some())
            {
                bail!("initial R6 host pairing record must use its bound generation without predecessor");
            }
            if has_loss_observation {
                if record.record_generation <= record.binding.host_record_generation {
                    bail!("loss-observed opened R6 record must advance its generation");
                }
                let previous = record.previous_record_sha256.as_deref().ok_or_else(|| {
                    anyhow!("loss-observed opened R6 record requires predecessor hash")
                })?;
                require_hex_digest(previous, "host pairing record previous_record_sha256")?;
            }
        }
        "operator_proof_verified" => {
            if record.record_generation <= record.binding.host_record_generation
                || record.data_session.state != "opened"
                || record.operator_tty_session.state != "closed"
                || record.operator_proof_sha256.is_none()
            {
                bail!("operator_proof_verified record has invalid generation or session states");
            }
            let previous = record.previous_record_sha256.as_deref().ok_or_else(|| {
                anyhow!("operator_proof_verified record requires predecessor hash")
            })?;
            require_hex_digest(previous, "host pairing record previous_record_sha256")?;
        }
        "guest_state_root_durable" => {
            if record.record_generation <= record.binding.host_record_generation
                || record.data_session.state != "guest_state_root_durable"
                || record.operator_tty_session.state != "closed"
                || record.operator_proof_sha256.is_none()
            {
                bail!("guest_state_root_durable record has invalid generation or session states");
            }
            let previous = record.previous_record_sha256.as_deref().ok_or_else(|| {
                anyhow!("guest_state_root_durable record requires predecessor hash")
            })?;
            require_hex_digest(previous, "host pairing record previous_record_sha256")?;
        }
        "hello_durable" => {
            if record.record_generation <= record.binding.host_record_generation
                || record.data_session.state != "hello_durable"
                || record.operator_tty_session.state != "closed"
                || record.operator_proof_sha256.is_none()
            {
                bail!("hello_durable record has invalid generation or session states");
            }
            let previous = record
                .previous_record_sha256
                .as_deref()
                .ok_or_else(|| anyhow!("hello_durable record requires predecessor hash"))?;
            require_hex_digest(previous, "host pairing record previous_record_sha256")?;
        }
        "transcript_durable" => {
            if record.record_generation <= record.binding.host_record_generation
                || record.data_session.state != "transcript_durable"
                || record.operator_tty_session.state != "closed"
                || record.operator_proof_sha256.is_none()
            {
                bail!("transcript_durable record has invalid generation or session states");
            }
            let previous = record
                .previous_record_sha256
                .as_deref()
                .ok_or_else(|| anyhow!("transcript_durable record requires predecessor hash"))?;
            require_hex_digest(previous, "host pairing record previous_record_sha256")?;
        }
        "ticket_consumed" => {
            if record.record_generation <= record.binding.host_record_generation {
                bail!("consumed host pairing record must advance its generation");
            }
            if record.data_session.state != "ticket_consumed"
                || record.operator_tty_session.state != "closed"
                || record.operator_proof_sha256.is_none()
            {
                bail!("ticket_consumed record has invalid session states");
            }
            let previous = record
                .previous_record_sha256
                .as_deref()
                .ok_or_else(|| anyhow!("consumed host pairing record requires predecessor hash"))?;
            require_hex_digest(previous, "host pairing record previous_record_sha256")?;
        }
        "pre_intent_closed" => {
            if record.record_generation <= record.binding.host_record_generation
                || record.data_session.state != "opened"
                || !matches!(
                    record.operator_tty_session.state.as_str(),
                    "opened" | "closed"
                )
                || (record.data_session.failure_observation.is_none()
                    && record.operator_tty_session.failure_observation.is_none())
            {
                bail!("pre_intent_closed record has invalid generation or session states");
            }
            let previous = record
                .previous_record_sha256
                .as_deref()
                .ok_or_else(|| anyhow!("pre_intent_closed record requires predecessor hash"))?;
            require_hex_digest(previous, "host pairing record previous_record_sha256")?;
        }
        _ => bail!("unknown R6 host pairing record state"),
    }
    if record.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || record.signature.public_key != record.ticket.signer_spki_der
    {
        bail!("host pairing record signer must be the canonical ticket P-256 SPKI");
    }
    verify_lifecycle_signature_v1(&record.schema_owner, record, &record.signature)
}

/// Canonical bytes for a protected macOS host pairing record.
pub fn canonical_guest_publisher_pairing_host_record_v1(
    record: &GuestPublisherPairingHostRecordV1,
) -> Result<Vec<u8>> {
    validate_guest_publisher_pairing_host_record_v1(record)?;
    canonical_json_to_vec(record).context("encode host pairing record")
}

/// Digest canonical protected host pairing state for generation-CAS evidence.
pub fn guest_publisher_pairing_host_record_sha256_v1(
    record: &GuestPublisherPairingHostRecordV1,
) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_guest_publisher_pairing_host_record_v1(record)?,
    )))
}

/// Apply one exact R6 host-record transition without accepting alternate session identities.
pub fn compare_and_swap_guest_publisher_pairing_host_record_v1(
    current: &GuestPublisherPairingHostRecordV1,
    next: &GuestPublisherPairingHostRecordV1,
) -> Result<()> {
    let now_unix_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("read clock for guest pairing host-record transition")?;
    let now_unix_ns = u64::try_from(now_unix_ns.as_nanos())
        .context("guest pairing host-record clock overflow")?;
    compare_and_swap_guest_publisher_pairing_host_record_at_v1(current, next, now_unix_ns)
}

/// Apply an R6 host-record transition at an explicit time. The durable owner invokes this before
/// its protected-store CAS so expiry, replay, and substituted session state cannot mutate it.
pub fn compare_and_swap_guest_publisher_pairing_host_record_at_v1(
    current: &GuestPublisherPairingHostRecordV1,
    next: &GuestPublisherPairingHostRecordV1,
    now_unix_ns: u64,
) -> Result<()> {
    validate_guest_publisher_pairing_host_record_v1(current)?;
    validate_guest_publisher_pairing_host_record_v1(next)?;
    validate_guest_publisher_pairing_ticket_at_v1(&current.ticket, now_unix_ns)?;
    validate_guest_publisher_pairing_ticket_at_v1(&next.ticket, now_unix_ns)?;
    if current == next {
        return Ok(());
    }
    if current.ticket != next.ticket
        || current.current_anchor_counter != next.current_anchor_counter
        || current.current_anchor_sha256 != next.current_anchor_sha256
        || current.binding != next.binding
        || current.data_session.binding != next.data_session.binding
        || current.operator_tty_session.binding != next.operator_tty_session.binding
        || current.data_session.session_id != next.data_session.session_id
        || current.operator_tty_session.session_id != next.operator_tty_session.session_id
        || current.operator_tty_session.controlling_tty_identity
            != next.operator_tty_session.controlling_tty_identity
        || current.operator_launch_sha256 != next.operator_launch_sha256
        || current.data_session.started_observation != next.data_session.started_observation
        || current.operator_tty_session.started_observation
            != next.operator_tty_session.started_observation
    {
        bail!("host pairing record transition changes immutable ticket bindings");
    }
    let forward_transition = matches!(
        (current.state.as_str(), next.state.as_str()),
        ("sessions_opened", "operator_proof_verified")
            | ("operator_proof_verified", "guest_state_root_durable")
            | ("guest_state_root_durable", "hello_durable")
            | ("hello_durable", "transcript_durable")
            | ("transcript_durable", "ticket_consumed")
            | ("sessions_opened", "pre_intent_closed")
            | ("operator_proof_verified", "pre_intent_closed")
    );
    // A loss observation is evidence, not a new protocol state.  It is allowed to advance the
    // protected generation exactly once for either fixed session while retaining every durable
    // frame and identity.  This is deliberately narrower than a generic same-state update: it
    // cannot alter a commitment, transcript, anchor, close observation, or existing failure.
    let failure_observation_transition = current.state == next.state
        && current.data_session.state == next.data_session.state
        && current.operator_tty_session.state == next.operator_tty_session.state
        && current.data_session.hello == next.data_session.hello
        && current.data_session.transcript == next.data_session.transcript
        && current.data_session.guest_anchor_sha256 == next.data_session.guest_anchor_sha256
        && current.data_session.ended_observation == next.data_session.ended_observation
        && current.operator_launch_sha256 == next.operator_launch_sha256
        && current.operator_proof_sha256 == next.operator_proof_sha256
        && current.operator_tty_session.ended_observation
            == next.operator_tty_session.ended_observation
        && (current.data_session.failure_observation.is_none()
            && next.data_session.failure_observation.is_some()
            && current.operator_tty_session.failure_observation
                == next.operator_tty_session.failure_observation
            || current.operator_tty_session.failure_observation.is_none()
                && next.operator_tty_session.failure_observation.is_some()
                && current.data_session.failure_observation
                    == next.data_session.failure_observation);
    if !forward_transition && !failure_observation_transition {
        bail!("invalid R6 host pairing durable transition");
    }
    let current_has_loss_observation = current.data_session.failure_observation.is_some()
        || current.operator_tty_session.failure_observation.is_some();
    if current_has_loss_observation && next.state != "pre_intent_closed" {
        bail!("loss-observed R6 pairing record may only close the preserved pre-intent attempt");
    }
    if current.operator_proof_sha256.is_some()
        && current.operator_proof_sha256 != next.operator_proof_sha256
    {
        bail!("host pairing record transition changes its admitted operator proof digest");
    }
    let expected_generation = current
        .record_generation
        .checked_add(1)
        .ok_or_else(|| anyhow!("host pairing record generation overflow"))?;
    if next.record_generation != expected_generation {
        bail!("host pairing record generation-CAS mismatch");
    }
    let current_digest = guest_publisher_pairing_host_record_sha256_v1(current)?;
    if next.previous_record_sha256.as_deref() != Some(current_digest.as_str()) {
        bail!("host pairing record predecessor digest mismatch");
    }
    Ok(())
}

pub fn canonical_lifecycle_signature_payload_v1<T: Serialize>(
    owner: &str,
    record: &T,
) -> Result<Vec<u8>> {
    let mut value =
        serde_json::to_value(record).context("serialize record for signature payload")?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("signed record must serialize to an object"))?;
    object.remove("signature");
    let mut payload = Vec::with_capacity(LIFECYCLE_SIGNATURE_PREFIX_V1.len() + owner.len() + 2);
    payload.extend_from_slice(LIFECYCLE_SIGNATURE_PREFIX_V1);
    payload.extend_from_slice(owner.as_bytes());
    payload.push(0);
    payload.extend_from_slice(
        &canonical_json_value_to_vec(&value).context("encode record signature payload")?,
    );
    Ok(payload)
}

pub fn verify_lifecycle_signature_v1<T: Serialize>(
    owner: &str,
    record: &T,
    signature: &LifecycleSignatureV1,
) -> Result<()> {
    let payload = canonical_lifecycle_signature_payload_v1(owner, record)?;
    let public_key = decode_base64url(&signature.public_key, "signature public_key")?;
    let signature_bytes = decode_base64url(&signature.signature, "signature bytes")?;
    match signature.algorithm.as_str() {
        "ed25519-v1" => verify_ed25519_fixed_v1(&public_key, &payload, &signature_bytes),
        "ecdsa-p256-sha256-p1363-low-s-v1" => {
            verify_p256_p1363_low_s_v1(&public_key, &payload, &signature_bytes)
        }
        other => bail!("unknown lifecycle signature algorithm {other}"),
    }
}

pub fn parse_p256_spki_der_v1(spki_der: &[u8]) -> Result<Vec<u8>> {
    let public_key =
        P256PublicKeyDocument::from_public_key_der(spki_der).context("invalid P-256 SPKI DER")?;
    let normalized = public_key
        .to_public_key_der()
        .context("re-encode P-256 SPKI DER")?;
    let normalized = normalized.as_ref().to_vec();
    if normalized != spki_der {
        bail!("P-256 SPKI DER is not canonical");
    }
    Ok(normalized)
}

pub fn verify_p256_p1363_low_s_v1(
    spki_der: &[u8],
    payload: &[u8],
    signature_bytes: &[u8],
) -> Result<()> {
    let verifying_key =
        P256PublicKey::from_public_key_der(spki_der).context("invalid P-256 public key")?;
    let signature =
        P256Signature::from_slice(signature_bytes).context("invalid P-256 P1363 signature")?;
    if signature.normalize_s().is_some() {
        bail!("P-256 signature is not low-S canonical");
    }
    verifying_key
        .verify(payload, &signature)
        .context("P-256 signature verification failed")
}

pub fn verify_ed25519_fixed_v1(
    public_key: &[u8],
    payload: &[u8],
    signature_bytes: &[u8],
) -> Result<()> {
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| anyhow!("Ed25519 public key must be 32 bytes"))?;
    let signature_bytes: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| anyhow!("Ed25519 signature must be 64 bytes"))?;
    let verifying_key =
        Ed25519PublicKey::from_bytes(&public_key).context("invalid Ed25519 public key")?;
    let signature = Ed25519Signature::from_bytes(&signature_bytes);
    verifying_key
        .verify(payload, &signature)
        .context("Ed25519 signature verification failed")
}

pub fn parse_publisher_bootstrap_authorization_v1(
    bytes: &[u8],
) -> Result<PublisherBootstrapAuthorizationV1> {
    let authorization: PublisherBootstrapAuthorizationV1 = canonical_json_from_slice(bytes)
        .context("bootstrap authorization is not canonical v1 JSON")?;
    validate_publisher_bootstrap_authorization_v1(&authorization)?;
    Ok(authorization)
}

pub fn canonical_mac_publisher_bootstrap_request_v1(
    request: &MacPublisherBootstrapRequestV1,
) -> Result<Vec<u8>> {
    if request.install_bootstrap_context_v1.is_empty()
        || request
            .install_bootstrap_context_v1
            .contains(['\0', '\n', '\r'])
    {
        bail!("macOS publisher bootstrap request has no exact IH carrier");
    }
    canonical_json_to_vec(request).context("encode macOS publisher bootstrap request")
}

pub fn parse_mac_publisher_bootstrap_request_v1(
    bytes: &[u8],
) -> Result<MacPublisherBootstrapRequestV1> {
    let request: MacPublisherBootstrapRequestV1 = canonical_json_from_slice(bytes)
        .context("macOS publisher bootstrap request is not canonical v1 JSON")?;
    canonical_mac_publisher_bootstrap_request_v1(&request)?;
    Ok(request)
}

/// Return the canonical bytes for the fixed per-scope Stage-1 capsule before its Keychain CAS.
pub fn canonical_mac_lima_stage_one_capsule_v1(
    capsule: &MacLimaStageOneCapsuleV1,
) -> Result<Vec<u8>> {
    validate_mac_lima_stage_one_capsule_v1(capsule)?;
    canonical_json_to_vec(capsule).context("encode canonical macOS Lima Stage-1 capsule")
}

pub fn validate_mac_publisher_bootstrap_attempt_locator_v1(
    locator: &MacPublisherBootstrapAttemptLocatorV1,
) -> Result<()> {
    require_schema(
        &locator.schema_owner,
        locator.schema_version,
        "substrate.mac-publisher-bootstrap-attempt-locator",
        1,
    )?;
    require_hex_digest(
        &locator.attempt_key_sha256,
        "bootstrap attempt locator attempt_key_sha256",
    )?;
    if locator.authority_domain != "mac_host_shared" {
        bail!("bootstrap attempt locator authority_domain must be mac_host_shared");
    }
    require_hex_digest(
        &locator.host_context_commitment,
        "bootstrap attempt locator host_context_commitment",
    )?;
    require_nonempty_no_nul(
        &locator.requester_principal,
        "bootstrap attempt locator requester_principal",
    )?;
    require_nonempty_no_nul(
        &locator.selected_host_prefix,
        "bootstrap attempt locator selected_host_prefix",
    )?;
    if !locator.selected_host_prefix.starts_with('/') {
        bail!("bootstrap attempt locator selected_host_prefix must be absolute");
    }
    require_hex_digest(
        &locator.install_provenance_sha256,
        "bootstrap attempt locator install_provenance_sha256",
    )?;
    require_git_oid(
        &locator.source_commit,
        "bootstrap attempt locator source_commit",
    )?;
    require_git_oid(
        &locator.source_tree,
        "bootstrap attempt locator source_tree",
    )?;
    require_nonempty_no_nul(&locator.source_ref, "bootstrap attempt locator source_ref")?;
    require_uuid_v7(&locator.scope_id, "bootstrap attempt locator scope_id")?;
    require_uuid_v7(&locator.attempt_id, "bootstrap attempt locator attempt_id")?;
    if locator.expires_at_unix_ns <= locator.issued_at_unix_ns {
        bail!("bootstrap attempt locator expiry must follow issue time");
    }
    for (value, field) in [
        (
            &locator.bootstrap_request_sha256,
            "bootstrap_request_sha256",
        ),
        (
            &locator.bootstrap_authorization_sha256,
            "bootstrap_authorization_sha256",
        ),
        (&locator.pre_pm_manifest_sha256, "pre_pm_manifest_sha256"),
    ] {
        require_hex_digest(value, field)?;
    }
    if let Some(value) = &locator.initial_anchor_sha256 {
        require_hex_digest(value, "initial_anchor_sha256")?;
    }
    for (value, field) in [
        (&locator.protected_state_sha256, "protected_state_sha256"),
        (&locator.capsule_sha256, "capsule_sha256"),
    ] {
        if let Some(value) = value {
            require_hex_digest(value, field)?;
        }
    }
    if locator.locator_revision == 0 {
        bail!("bootstrap attempt locator revision must be positive");
    }
    match locator.terminal_state.as_str() {
        "Allocated" => {
            if locator.canonical_response_b64.is_some()
                || locator.initial_anchor_sha256.is_some()
                || locator.protected_state_sha256.is_some()
                || locator.capsule_sha256.is_some()
            {
                bail!("allocated bootstrap attempt locator must not retain a response");
            }
        }
        "Completed" => {
            let response = locator.canonical_response_b64.as_deref().ok_or_else(|| {
                anyhow!("completed bootstrap attempt locator lacks canonical response")
            })?;
            let bytes = decode_base64url(response, "bootstrap attempt locator response")?;
            let value: Value = serde_json::from_slice(&bytes)
                .context("decode bootstrap attempt locator response")?;
            if !value.is_object() || canonical_json_to_vec(&value)? != bytes {
                bail!("bootstrap attempt locator response is not canonical object JSON");
            }
            fn contains_forbidden_authorization_carrier(value: &Value) -> bool {
                match value {
                    Value::Object(object) => object.iter().any(|(key, value)| {
                        matches!(
                            key.as_str(),
                            "publisher_bootstrap_authorization"
                                | "publisher_bootstrap_authorization_v1"
                                | "complete_authorization"
                        ) || contains_forbidden_authorization_carrier(value)
                    }),
                    Value::Array(values) => {
                        values.iter().any(contains_forbidden_authorization_carrier)
                    }
                    _ => false,
                }
            }
            if contains_forbidden_authorization_carrier(&value) {
                bail!("bootstrap attempt locator response must not serialize authorization");
            }
            if locator.initial_anchor_sha256.is_none()
                || locator.protected_state_sha256.is_none()
                || locator.capsule_sha256.is_none()
            {
                bail!(
                    "completed bootstrap attempt locator lacks durable anchor/state/capsule joins"
                );
            }
        }
        "PreservingBlocked" => {
            if locator.canonical_response_b64.is_some() {
                bail!("blocked bootstrap attempt locator must not retain a success response");
            }
        }
        _ => bail!("bootstrap attempt locator has an unknown terminal state"),
    }
    Ok(())
}

pub fn validate_publisher_bootstrap_authorization_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<()> {
    require_schema(
        &authorization.schema_owner,
        authorization.schema_version,
        "substrate.publisher-bootstrap-authorization",
        1,
    )?;
    require_known_authority_domain(&authorization.authority_domain)?;
    require_uuid_v7(&authorization.scope_id, "publisher bootstrap scope_id")?;
    require_hex_digest(
        &authorization.host_context_commitment,
        "publisher bootstrap host_context_commitment",
    )?;
    require_nonempty_no_nul(
        &authorization.install_bootstrap_context_v1,
        "publisher bootstrap exact IH carrier",
    )?;
    if let Some(commitment) = &authorization.platform_mapping_commitment {
        require_hex_digest(
            commitment,
            "publisher bootstrap platform_mapping_commitment",
        )?;
    }
    require_git_oid(
        &authorization.source_commit,
        "publisher bootstrap source_commit",
    )?;
    require_git_oid(
        &authorization.source_tree,
        "publisher bootstrap source_tree",
    )?;
    require_nonempty_no_nul(&authorization.source_ref, "publisher bootstrap source_ref")?;
    if authorization.manifest_generation == 0 {
        bail!("publisher bootstrap manifest_generation must be positive");
    }
    require_hex_digest(
        &authorization.manifest_sha256,
        "publisher bootstrap manifest_sha256",
    )?;
    require_nonempty_no_nul(
        &authorization.requester_principal,
        "publisher bootstrap requester_principal",
    )?;
    require_uuid_v7(
        &authorization.attempt_nonce,
        "publisher bootstrap attempt_nonce",
    )?;
    if authorization.expires_at_unix_ns <= authorization.issued_at_unix_ns {
        bail!("publisher bootstrap authorization expiry must follow issue time");
    }
    validate_executor_build_evidence_v1(&authorization.executor_build_evidence)?;
    if authorization.executor_build_evidence.source_commit != authorization.source_commit
        || authorization.executor_build_evidence.source_tree != authorization.source_tree
        || authorization.executor_build_evidence.source_ref != authorization.source_ref
    {
        bail!("publisher bootstrap executor build evidence does not join authorization source identity");
    }
    match (
        authorization.authority_domain.as_str(),
        authorization.mac_control_authority.as_ref(),
    ) {
        ("mac_host_shared", Some(control_authority)) => {
            validate_mac_publisher_control_authority_v1(control_authority)?;
            if control_authority.source_commit != authorization.source_commit
                || control_authority.source_tree != authorization.source_tree
                || control_authority.source_ref != authorization.source_ref
            {
                bail!("mac_control_authority does not join authorization source identity");
            }
            if control_authority.target_triple
                != authorization.executor_build_evidence.target_triple
            {
                bail!("mac_control_authority does not join authorization executor build");
            }
        }
        ("mac_host_shared", None) => {
            bail!("mac_host_shared bootstrap authorization requires mac_control_authority");
        }
        (_, Some(_)) => {
            bail!(
                "mac_control_authority is valid only for mac_host_shared bootstrap authorization"
            );
        }
        (_, None) => {}
    }
    if authorization.authority_domain == "mac_host_shared"
        && authorization.platform_mapping_commitment.is_none()
        && (!authorization.publisher_expected_absent || authorization.manifest_generation != 1)
    {
        bail!(
            "pre-PM mac_host_shared bootstrap authorization requires absent publisher and generation one"
        );
    }
    match (
        authorization.authority_domain.as_str(),
        authorization.publisher_expected_absent,
        authorization.pre_pm_manifest.as_ref(),
    ) {
        ("mac_host_shared", true, Some(manifest)) => {
            canonical_manifest_bytes_v1(manifest)?;
            if manifest.authority_domain != "mac_host_shared"
                || manifest.platform_kind != "mac_lima"
                || manifest.platform_mapping_commitment.is_some()
                || manifest.manifest_generation != 1
                || manifest.manifest_generation != authorization.manifest_generation
                || manifest.manifest_sha256 != authorization.manifest_sha256
                || manifest.host_context_commitment != authorization.host_context_commitment
                || manifest.installation_id != authorization.scope_id
                || manifest.intended_principal != authorization.requester_principal
                || manifest.attempt_id != authorization.attempt_nonce
            {
                bail!("mac_host_shared bootstrap authorization pre-PM manifest does not exact-join authorization");
            }
        }
        ("mac_host_shared", true, None) => {
            bail!("mac_host_shared absent bootstrap authorization requires its canonical pre-PM manifest");
        }
        (_, _, Some(_)) => {
            bail!("pre_pm_manifest is valid only for an absent mac_host_shared bootstrap authorization");
        }
        _ => {}
    }
    let confirmation = authorization.confirmation.as_str();
    if confirmation != "CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER" {
        bail!("publisher bootstrap confirmation literal mismatch");
    }
    let mut component_ids = BTreeSet::new();
    for component in &authorization.components {
        validate_publisher_bootstrap_component_v1(component)?;
        if !component_ids.insert(component.component_id.clone()) {
            bail!("publisher bootstrap component IDs must be unique");
        }
    }
    validate_complete_publisher_bootstrap_component_set_v1(&authorization.components)?;
    if let Some(commitment) = &authorization.test_retirement_commitment {
        validate_test_retirement_commitment_v1(commitment)?;
    }
    Ok(())
}

pub fn derive_publisher_bootstrap_component_target_v1(
    role: &PublisherBootstrapComponentRoleV1,
    scope_id: &str,
) -> Result<String> {
    let parsed = parse_component_role(&role.0)?;
    Ok(match parsed {
        ParsedComponentRole::LinuxStateRoot => "/var/lib/substrate".to_string(),
        ParsedComponentRole::LinuxLifecycleContainer => {
            "/var/lib/substrate/.substrate-lifecycle-v1".to_string()
        }
        ParsedComponentRole::LinuxPublisherDirectory => {
            "/var/lib/substrate/.substrate-lifecycle-v1/publisher".to_string()
        }
        ParsedComponentRole::LinuxExecutorParent => "/usr/libexec/substrate".to_string(),
        ParsedComponentRole::LinuxBootstrapIntent => {
            format!("/var/lib/substrate/.substrate-lifecycle-bootstrap-intent-v1.{scope_id}.json")
        }
        ParsedComponentRole::LinuxExecutor => {
            "/usr/libexec/substrate/substrate-lifecycle-linux".to_string()
        }
        ParsedComponentRole::LinuxServiceUnit(kind) => {
            format!("/etc/systemd/system/substrate-lifecycle-publisher-v1.{kind}")
        }
        ParsedComponentRole::LinuxEndpoint => {
            "/run/substrate-lifecycle-publisher-v1.sock".to_string()
        }
        ParsedComponentRole::LinuxSigningKey => {
            "/var/lib/substrate/.substrate-lifecycle-v1/publisher/signing-key.v1".to_string()
        }
        ParsedComponentRole::LinuxProtectedState => {
            "/var/lib/substrate/.substrate-lifecycle-v1/publisher/current-anchor.v1.json"
                .to_string()
        }
        ParsedComponentRole::LinuxServiceState(kind) => {
            format!("substrate-lifecycle-publisher-v1.{kind}")
        }
        ParsedComponentRole::MacExecutor => {
            "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1".to_string()
        }
        ParsedComponentRole::MacPlist => {
            "/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist".to_string()
        }
        ParsedComponentRole::MacMachService => "com.substrate.lifecycle.publisher.v1".to_string(),
        ParsedComponentRole::MacSigningKey => format!("{scope_id}:signing-key"),
        ParsedComponentRole::MacProtectedState => format!("{scope_id}:current-anchor"),
        ParsedComponentRole::MacBootstrapIntent => format!("{scope_id}:bootstrap-intent"),
        ParsedComponentRole::MacServiceState => "com.substrate.lifecycle.publisher.v1".to_string(),
        ParsedComponentRole::MacGuestPairingRecord(challenge_id) => {
            format!("{scope_id}:guest-pairing:{challenge_id}")
        }
        ParsedComponentRole::WindowsProductDirectory => r"%ProgramFiles%\Substrate".to_string(),
        ParsedComponentRole::WindowsInstallDirectory => {
            r"%ProgramFiles%\Substrate\Lifecycle".to_string()
        }
        ParsedComponentRole::WindowsExecutable => {
            r"%ProgramFiles%\Substrate\Lifecycle\substrate-lifecycle-windows.exe".to_string()
        }
        ParsedComponentRole::WindowsServiceRegistration => {
            "SubstrateLifecyclePublisherV1".to_string()
        }
        ParsedComponentRole::WindowsSigningKey => "SubstrateLifecyclePublisherV1".to_string(),
        ParsedComponentRole::WindowsProtectedState => {
            format!(r"HKLM\SOFTWARE\Substrate\LifecycleV1\Anchors\{scope_id}")
        }
        ParsedComponentRole::WindowsBootstrapIntent => {
            format!(r"HKLM\SOFTWARE\Substrate\LifecycleV1\BootstrapIntents\{scope_id}")
        }
        ParsedComponentRole::WindowsEndpoint => {
            r"\\.\pipe\SubstrateLifecyclePublisherV1".to_string()
        }
        ParsedComponentRole::WindowsServiceState => "SubstrateLifecyclePublisherV1".to_string(),
        ParsedComponentRole::WindowsRegistryContainer(kind) => match kind.as_str() {
            "substrate" => r"HKLM\SOFTWARE\Substrate".to_string(),
            "lifecycle-v1" => r"HKLM\SOFTWARE\Substrate\LifecycleV1".to_string(),
            "anchors" => r"HKLM\SOFTWARE\Substrate\LifecycleV1\Anchors".to_string(),
            "bootstrap-intents" => {
                r"HKLM\SOFTWARE\Substrate\LifecycleV1\BootstrapIntents".to_string()
            }
            "pairings" => r"HKLM\SOFTWARE\Substrate\LifecycleV1\Pairings".to_string(),
            "pairing-scope" => {
                format!(r"HKLM\SOFTWARE\Substrate\LifecycleV1\Pairings\{scope_id}")
            }
            _ => unreachable!(),
        },
        ParsedComponentRole::WindowsGuestPairingRecord(challenge_id) => {
            format!(r"HKLM\SOFTWARE\Substrate\LifecycleV1\Pairings\{scope_id}\{challenge_id}")
        }
        ParsedComponentRole::GuestStateRoot(_) => "/var/lib/substrate".to_string(),
        ParsedComponentRole::GuestLifecycleContainer(_) => {
            "/var/lib/substrate/.substrate-lifecycle-v1".to_string()
        }
        ParsedComponentRole::GuestPublisherDirectory(_) => {
            "/var/lib/substrate/.substrate-lifecycle-v1/publisher".to_string()
        }
        ParsedComponentRole::GuestExecutorParent(_) => "/usr/libexec/substrate".to_string(),
        ParsedComponentRole::GuestBootstrapIntent(_, challenge_id) => {
            format!(
                "/var/lib/substrate/.substrate-lifecycle-pairing-intent-v1.{scope_id}.{challenge_id}.json"
            )
        }
        ParsedComponentRole::GuestExecutor(platform) => {
            guest_publisher_target(&platform, "executor")
        }
        ParsedComponentRole::GuestServiceUnit(platform, kind) => {
            guest_publisher_service_unit_target(&platform, &kind)
        }
        ParsedComponentRole::GuestEndpoint(platform) => {
            guest_publisher_target(&platform, "endpoint")
        }
        ParsedComponentRole::GuestSigningKey(platform) => {
            guest_publisher_target(&platform, "signing-key")
        }
        ParsedComponentRole::GuestProtectedState(platform) => {
            guest_publisher_target(&platform, "current-anchor")
        }
        ParsedComponentRole::GuestServiceState(platform, kind) => {
            format!("guest:{platform}:substrate-lifecycle-publisher-v1.{kind}")
        }
    })
}

pub fn validate_complete_publisher_bootstrap_component_set_v1(
    components: &[PublisherBootstrapComponentV1],
) -> Result<()> {
    if components.is_empty() {
        bail!("publisher bootstrap component set must not be empty");
    }
    let scope_id = infer_scope_id_from_components(components)?;
    let mut component_ids = BTreeSet::new();
    let mut targets = BTreeMap::new();
    for component in components {
        if !component_ids.insert(component.component_id.clone()) {
            bail!("publisher bootstrap component IDs must be unique");
        }
        let derived_target =
            derive_publisher_bootstrap_component_target_v1(&component.role, &scope_id)?;
        if derived_target != component.target_identity {
            bail!("publisher bootstrap component target does not match derived target");
        }
        if let Some(existing_role) =
            targets.insert(derived_target.clone(), component.role.0.as_str())
        {
            let shared_mac_service_target = matches!(
                (existing_role, component.role.0.as_str()),
                ("MacMachService", "MacServiceState") | ("MacServiceState", "MacMachService")
            );
            if !shared_mac_service_target {
                bail!("publisher bootstrap component targets must be unique");
            }
        }
        if !component
            .dependency_component_ids
            .iter()
            .all(|id| id != &component.component_id)
        {
            bail!("publisher bootstrap component cannot depend on itself");
        }
    }
    for component in components {
        for dependency in &component.dependency_component_ids {
            if !component_ids.contains(dependency) {
                bail!("publisher bootstrap component dependency is missing from the set");
            }
        }
    }
    Ok(())
}

pub fn canonical_lima_stage_one_authorization_v1(
    authorization: &LimaStageOneAuthorizationV1,
) -> Result<Vec<u8>> {
    validate_lima_stage_one_authorization_v1(authorization)?;
    canonical_json_to_vec(authorization).context("encode Lima stage-one authorization")
}

pub fn validate_lima_stage_one_authorization_v1(
    authorization: &LimaStageOneAuthorizationV1,
) -> Result<()> {
    require_schema(
        &authorization.schema_owner,
        authorization.schema_version,
        LIMA_STAGE_ONE_SCHEMA_OWNER_V1,
        1,
    )?;
    require_hex_digest(
        &authorization.host_context_commitment,
        "Lima stage-one host_context_commitment",
    )?;
    require_hex_digest(
        &authorization.profile_sha256,
        "Lima stage-one profile_sha256",
    )?;
    require_git_oid(&authorization.source_commit, "Lima stage-one source_commit")?;
    require_git_oid(&authorization.source_tree, "Lima stage-one source_tree")?;
    require_hex_digest(
        &authorization.executor_receipt_sha256,
        "Lima stage-one executor_receipt_sha256",
    )?;
    require_uuid_v7(&authorization.attempt_id, "Lima stage-one attempt_id")?;
    require_uuid_v7(&authorization.nonce, "Lima stage-one nonce")?;
    let rendered_profile = decode_base64url(
        &authorization.rendered_profile_b64,
        "Lima stage-one rendered_profile_b64",
    )?;
    if rendered_profile.is_empty()
        || lower_hex(&Sha256::digest(&rendered_profile)) != authorization.rendered_profile_sha256
    {
        bail!("Lima stage-one rendered profile does not match canonical digest");
    }
    require_hex_digest(
        &authorization.rendered_profile_sha256,
        "Lima stage-one rendered_profile_sha256",
    )?;
    validate_mac_lima_stage_one_successor_template_v1(
        &authorization.successor_template,
        authorization,
    )?;
    verify_lifecycle_signature_v1(
        &authorization.schema_owner,
        authorization,
        &authorization.signature,
    )
}

/// Validate the closed pre-effect Stage-1 template.  This is intentionally separate from
/// `ManagedArtifactManifestV1`: constructing a PM mapping/manifest before the selected create
/// would fabricate the only machine-derived observation the template permits after the effect.
pub fn validate_mac_lima_stage_one_successor_template_v1(
    template: &MacLimaStageOneSuccessorTemplateV1,
    authorization: &LimaStageOneAuthorizationV1,
) -> Result<()> {
    require_schema(
        &template.schema_owner,
        template.schema_version,
        "substrate.mac-lima-stage-one-successor-template",
        1,
    )?;
    if template.derivation_algorithm
        != "mac_lima_stage_one_successor_template_v1_platform_mapping_then_manifest"
    {
        bail!("Lima Stage-1 successor template has an unknown derivation algorithm");
    }
    if template.profile_template_algorithm != "substrate.mac-lima-stage-one-profile-template"
        || template.profile_template_version != 1
    {
        bail!("Lima Stage-1 successor template profile algorithm/version mismatch");
    }
    require_hex_digest(
        &template.profile_template_sha256,
        "Lima Stage-1 successor template profile_template_sha256",
    )?;
    if template.current_pre_pm_manifest_generation == 0
        || template.next_manifest_generation
            != template
                .current_pre_pm_manifest_generation
                .checked_add(1)
                .ok_or_else(|| anyhow!("Lima Stage-1 successor template generation overflow"))?
        || template.manifest_lifecycle_state != ManagedLifecycleStateV1::ManifestDurable
        || template.manifest_created_at_unix_ns == 0
    {
        bail!("Lima Stage-1 successor template has an invalid manifest generation/time policy");
    }
    for (value, field) in [
        (
            &template.current_pre_pm_manifest_sha256,
            "Lima Stage-1 current_pre_pm_manifest_sha256",
        ),
        (
            &template.current_anchor_sha256,
            "Lima Stage-1 current_anchor_sha256",
        ),
        (
            &template.previous_manifest_sha256,
            "Lima Stage-1 previous_manifest_sha256",
        ),
        (
            &template.host_context_commitment,
            "Lima Stage-1 template host_context_commitment",
        ),
        (
            &template.profile_sha256,
            "Lima Stage-1 template profile_sha256",
        ),
    ] {
        require_hex_digest(value, field)?;
    }
    if template.previous_manifest_sha256 != template.current_pre_pm_manifest_sha256
        || template.host_context_commitment != authorization.host_context_commitment
        || template.scope_id != template.installation_id
        || template.instance_name != authorization.instance_name
        || template.profile_sha256 != authorization.profile_sha256
        || template.source_commit != authorization.source_commit
        || template.source_tree != authorization.source_tree
        || template.source_ref != authorization.source_ref
        || template.attempt_id != authorization.attempt_id
        || template.nonce != authorization.nonce
        || template.expires_at_unix_ns != authorization.expires_at_unix_ns
        || template.intended_principal != authorization.requester_principal
    {
        bail!("Lima Stage-1 successor template does not exact-join its signed authorization");
    }
    require_uuid_v7(&template.scope_id, "Lima Stage-1 template scope_id")?;
    require_uuid_v7(
        &template.installation_id,
        "Lima Stage-1 template installation_id",
    )?;
    require_nonempty_no_nul(
        &template.intended_principal,
        "Lima Stage-1 template intended_principal",
    )?;
    require_nonempty_no_nul(
        &template.selected_host_prefix,
        "Lima Stage-1 template selected_host_prefix",
    )?;
    require_nonempty_no_nul(
        &template.host_platform_control_root,
        "Lima Stage-1 template host_platform_control_root",
    )?;
    require_nonempty_no_nul(
        &template.instance_name,
        "Lima Stage-1 template instance_name",
    )?;
    validate_executor_build_evidence_v1(&template.executor_build_evidence)?;
    if template.executor_build_evidence.source_commit != template.source_commit
        || template.executor_build_evidence.source_tree != template.source_tree
        || template.executor_build_evidence.source_ref != template.source_ref
    {
        bail!("Lima Stage-1 template executor build evidence does not join source identity");
    }
    if template.planned_receipt_id.is_empty()
        || template.planned_receipt_id.contains(['\0', '\n', '\r'])
        || !template
            .planned_receipt_relative_path
            .starts_with("receipts/")
        || template
            .planned_receipt_relative_path
            .contains(['\0', '\n', '\r'])
        || template.post_effect_observation_slots.len() != 1
        || template
            .post_effect_observation_slots
            .first()
            .map(String::as_str)
            != Some("guest_machine_id")
    {
        bail!("Lima Stage-1 template must name one receipt and only guest_machine_id observation");
    }
    let mut entry_ids = BTreeSet::new();
    for entry in &template.ordered_non_machine_entries {
        // Full role/identity validation runs after the single machine observation is joined into
        // the derived manifest.  Before the effect there is intentionally no PM mapping against
        // which the MAC role identity can be validated; still reject empty/duplicate ordering
        // identifiers at the signed template boundary.
        require_nonempty_no_nul(&entry.object_id, "Lima Stage-1 template entry object_id")?;
        if !entry_ids.insert(entry.object_id.clone()) {
            bail!("Lima Stage-1 template entries must be ordered with unique object IDs");
        }
    }
    Ok(())
}

pub fn validate_lima_stage_one_observation_v1(
    observation: &LimaStageOneObservationV1,
) -> Result<()> {
    require_nonempty_no_nul(
        &observation.instance_name,
        "Lima Stage-1 observation instance_name",
    )?;
    if observation.instance_status != "Running" {
        bail!("Lima Stage-1 observation instance_status must be Running");
    }
    if observation.guest_machine_id.len() != 32
        || !observation
            .guest_machine_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("Lima Stage-1 observation guest_machine_id must be 128-bit lowercase hex");
    }
    require_uuid_v7(
        &observation.marker_attempt_id,
        "Lima Stage-1 observation marker_attempt_id",
    )?;
    require_hex_digest(
        &observation.marker_capsule_sha256,
        "Lima Stage-1 observation marker_capsule_sha256",
    )?;
    require_nonempty_no_nul(
        &observation.guest_account,
        "Lima Stage-1 observation guest_account",
    )?;
    if observation.guest_uid == 0 {
        bail!("Lima Stage-1 observation guest_uid must be positive");
    }
    require_nonempty_no_nul(
        &observation.guest_home,
        "Lima Stage-1 observation guest_home",
    )?;
    if !observation.guest_home.starts_with('/') {
        bail!("Lima Stage-1 observation guest_home must be absolute");
    }
    Ok(())
}

pub fn validate_mac_lima_stage_one_capsule_v1(capsule: &MacLimaStageOneCapsuleV1) -> Result<()> {
    require_schema(
        &capsule.schema_owner,
        capsule.schema_version,
        "substrate.mac-lima-stage-one-capsule",
        1,
    )?;
    require_uuid_v7(&capsule.scope_id, "Lima Stage-1 capsule scope_id")?;
    if !matches!(
        capsule.state.as_str(),
        "Issued"
            | "Prepared"
            | "EffectStarted"
            | "InstanceObserved"
            | "Completed"
            | "PreservingBlocked"
    ) {
        bail!("Lima Stage-1 capsule state is not closed");
    }
    require_hex_digest(
        &capsule.install_provenance_sha256,
        "Lima Stage-1 capsule install_provenance_sha256",
    )?;
    require_hex_digest(
        &capsule.bootstrap_authorization_sha256,
        "Lima Stage-1 capsule bootstrap_authorization_sha256",
    )?;
    canonical_manifest_bytes_v1(&capsule.pre_pm_manifest)?;
    validate_lima_stage_one_authorization_v1(&capsule.stage_one_authorization)?;
    let stage = &capsule.stage_one_authorization;
    if capsule.scope_id != stage.successor_template.scope_id
        || capsule.pre_pm_manifest.manifest_generation != 1
        || capsule.pre_pm_manifest.manifest_sha256
            != stage.successor_template.current_pre_pm_manifest_sha256
        || capsule.pre_pm_manifest.host_context_commitment != stage.host_context_commitment
        || capsule.pre_pm_manifest.installation_id != capsule.scope_id
        || stage.executor_receipt_sha256 != capsule.bootstrap_authorization_sha256
        || capsule.initial_anchor_sha256 != stage.successor_template.current_anchor_sha256
        || capsule.initial_anchor_counter != stage.successor_template.current_anchor_counter
    {
        bail!("Lima Stage-1 capsule does not exact-join authorization/template/initial anchor");
    }
    if let Some(file_identity) = &capsule.rendered_profile_file_identity {
        require_nonempty_no_nul(
            file_identity,
            "Lima Stage-1 capsule rendered_profile_file_identity",
        )?;
    }
    if let Some(prepared_state_sha256) = &capsule.prepared_protected_state_sha256 {
        require_hex_digest(
            prepared_state_sha256,
            "Lima Stage-1 capsule prepared_protected_state_sha256",
        )?;
    }
    if matches!(capsule.state.as_str(), "Issued" | "Prepared")
        && capsule.prepared_protected_state_sha256.is_some()
    {
        bail!("pre-effect Lima Stage-1 capsule cannot carry a prepared-state digest");
    }
    if matches!(
        capsule.state.as_str(),
        "EffectStarted" | "InstanceObserved" | "Completed"
    ) && capsule.prepared_protected_state_sha256.is_none()
    {
        bail!("effect-capable Lima Stage-1 capsule lacks its prepared-state digest");
    }
    if let Some(observation) = &capsule.observation {
        validate_lima_stage_one_observation_v1(observation)?;
        if observation.instance_name != stage.instance_name
            || observation.marker_attempt_id != stage.attempt_id
        {
            bail!("Lima Stage-1 capsule observation does not exact-join signed authorization");
        }
    }
    if matches!(capsule.state.as_str(), "InstanceObserved" | "Completed")
        && capsule.observation.is_none()
    {
        bail!("observed/completed Lima Stage-1 capsule requires its exact observation");
    }
    Ok(())
}

fn validate_manifest_v1(manifest: &ManagedArtifactManifestV1) -> Result<()> {
    require_schema(
        &manifest.schema_owner,
        manifest.schema_version,
        MANIFEST_SCHEMA_OWNER_V1,
        MANIFEST_SCHEMA_VERSION_V1,
    )?;
    require_hex_digest(
        &manifest.host_context_commitment,
        "manifest host_context_commitment",
    )?;
    if let Some(commitment) = &manifest.platform_mapping_commitment {
        require_hex_digest(commitment, "manifest platform_mapping_commitment")?;
    }
    require_known_authority_domain(&manifest.authority_domain)?;
    require_uuid_v7(&manifest.installation_id, "manifest installation_id")?;
    require_uuid_v7(&manifest.attempt_id, "manifest attempt_id")?;
    if manifest.manifest_generation == 0 {
        bail!("manifest_generation must be positive");
    }
    let expected_manifest_id = format!(
        "m1:{}:{}",
        manifest.installation_id, manifest.manifest_generation
    );
    if manifest.manifest_id != expected_manifest_id {
        bail!("manifest_id must match installation_id and manifest_generation");
    }
    if let Some(previous) = &manifest.previous_manifest_sha256 {
        require_hex_digest(previous, "previous_manifest_sha256")?;
    }
    require_nonempty_no_nul(
        &manifest.selected_host_prefix,
        "manifest selected_host_prefix",
    )?;
    require_nonempty_no_nul(&manifest.intended_principal, "manifest intended_principal")?;
    require_known_platform_kind(&manifest.platform_kind)?;
    match (
        manifest.authority_domain.as_str(),
        manifest.platform_kind.as_str(),
    ) {
        ("unix_a_local", "unix")
        | ("windows_a_local", "windows")
        | ("linux_system", "linux")
        | ("mac_host_shared", "mac_lima")
        | ("mac_lima_guest", "mac_lima")
        | ("windows_host_shared", "windows_wsl")
        | ("windows_wsl_guest", "windows_wsl") => {}
        _ => bail!("manifest authority_domain does not match platform_kind"),
    }
    if manifest.authority_domain == "mac_host_shared"
        && manifest.platform_mapping_commitment.is_none()
    {
        let exact_pre_pm_bootstrap_manifest = manifest.platform_kind == "mac_lima"
            && manifest.manifest_generation == 1
            && manifest.previous_manifest_sha256.is_none()
            && manifest.lifecycle_state == ManagedLifecycleStateV1::ManifestDurable
            && manifest.entries.is_empty()
            && manifest.planned_action_receipts.is_empty();
        if !exact_pre_pm_bootstrap_manifest {
            bail!(
                "mac_host_shared manifests require platform_mapping_commitment except the exact empty generation-one pre-PM bootstrap manifest"
            );
        }
    }
    let digest = manifest_digest_v1(manifest)?;
    if digest != manifest.manifest_sha256 {
        bail!("manifest_sha256 does not match canonical manifest digest");
    }
    let mut object_ids = BTreeSet::new();
    let mut role_by_object_id = BTreeMap::new();
    for entry in &manifest.entries {
        let parsed_role = validate_manifest_entry_v1(manifest, entry)?;
        if !object_ids.insert(entry.object_id.clone()) {
            bail!("manifest entry object IDs must be unique");
        }
        role_by_object_id.insert(entry.object_id.clone(), parsed_role);
    }
    for entry in &manifest.entries {
        validate_manifest_entry_closure_v1(entry, &object_ids)?;
    }
    let mut receipt_ids = BTreeSet::new();
    let mut receipt_paths = BTreeSet::new();
    for planned in &manifest.planned_action_receipts {
        let planned = validate_planned_action_receipt_v1(
            planned,
            manifest.manifest_generation,
            &role_by_object_id,
        )?;
        if !receipt_ids.insert(planned.receipt_id.clone()) {
            bail!("planned action receipt IDs must be unique");
        }
        if !receipt_paths.insert(planned.receipt_relative_path.clone()) {
            bail!("planned action receipt paths must be unique");
        }
    }
    Ok(())
}

fn manifest_digest_v1(manifest: &ManagedArtifactManifestV1) -> Result<String> {
    let mut value = serde_json::to_value(manifest).context("serialize manifest for digest")?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("manifest must serialize to an object"))?;
    object.remove("manifest_sha256");
    Ok(lower_hex(&Sha256::digest(
        canonical_json_value_to_vec(&value).context("encode manifest digest bytes")?,
    )))
}

fn validate_manifest_entry_v1(
    manifest: &ManagedArtifactManifestV1,
    entry: &ManagedArtifactEntryV1,
) -> Result<ParsedManagedRole> {
    require_nonempty_no_nul(&entry.object_id, "manifest entry object_id")?;
    let parsed_role = parse_managed_role(&entry.logical_role.0)?;
    validate_identity_v1(&entry.identity)?;
    if entry.identity.scope_id != manifest.installation_id {
        bail!("manifest entry identity scope_id does not match manifest installation_id");
    }
    validate_role_authority_domain_v1(manifest, &parsed_role)?;
    validate_object_type_for_role_v1(&parsed_role, &entry.object_type)?;
    validate_manifest_entry_identity_for_role_v1(manifest, entry, &parsed_role)?;
    validate_manifest_entry_dependency_shape_v1(entry)?;
    if entry.disposition == ManagedArtifactDispositionV1::Adopted {
        bail!("manifest entries may not use adopted disposition in R3");
    }
    if let Some(hash_or_target) = &entry.bytes_or_target {
        require_nonempty_no_nul(hash_or_target, "manifest entry bytes_or_target")?;
    }
    if let Some(mode) = &entry.mode {
        validate_mode_text(mode, "manifest entry mode")?;
    }
    validate_json_value(&entry.before_state, "manifest entry before_state")?;
    validate_json_value(
        &entry.intended_after_state,
        "manifest entry intended_after_state",
    )?;
    validate_json_value(&entry.restoration_state, "manifest entry restoration_state")?;
    if let Some(state) = &entry.service_or_platform_state {
        validate_json_value(state, "manifest entry service_or_platform_state")?;
    }
    Ok(parsed_role)
}

fn validate_manifest_entry_closure_v1(
    entry: &ManagedArtifactEntryV1,
    object_ids: &BTreeSet<String>,
) -> Result<()> {
    for dependency in &entry.dependency_object_ids {
        if !object_ids.contains(dependency) {
            bail!("manifest entry dependency_object_ids must reference manifest object IDs");
        }
    }
    for member in &entry.subtree_members {
        if !object_ids.contains(member) {
            bail!("manifest entry subtree_members must reference manifest object IDs");
        }
    }
    Ok(())
}

fn validate_manifest_entry_dependency_shape_v1(entry: &ManagedArtifactEntryV1) -> Result<()> {
    let mut dependency_ids = BTreeSet::new();
    for dependency in &entry.dependency_object_ids {
        require_nonempty_no_nul(dependency, "manifest entry dependency_object_id")?;
        if dependency == &entry.object_id {
            bail!("manifest entry dependency_object_ids may not include the entry object_id");
        }
        if !dependency_ids.insert(dependency.clone()) {
            bail!("manifest entry dependency_object_ids must be unique");
        }
    }
    let mut subtree_members = BTreeSet::new();
    for member in &entry.subtree_members {
        require_nonempty_no_nul(member, "manifest entry subtree_member")?;
        if member == &entry.object_id {
            bail!("manifest entry subtree_members may not include the entry object_id");
        }
        if !subtree_members.insert(member.clone()) {
            bail!("manifest entry subtree_members must be unique");
        }
    }
    if !entry.subtree_members.is_empty() && entry.object_type != "directory" {
        bail!("manifest entry subtree_members require object_type directory");
    }
    Ok(())
}

fn validate_role_authority_domain_v1(
    manifest: &ManagedArtifactManifestV1,
    parsed_role: &ParsedManagedRole,
) -> Result<()> {
    let expected_domain = match parsed_role {
        ParsedManagedRole::UnixPrefixPayloadVersion(_)
        | ParsedManagedRole::UnixPrefixBinEntry(_)
        | ParsedManagedRole::UnixPrefixShimTree
        | ParsedManagedRole::UnixPrefixLinuxGuestCache(_)
        | ParsedManagedRole::UnixPrefixDevEnv
        | ParsedManagedRole::UnixPrefixProjection(_)
        | ParsedManagedRole::UnixPrefixRuntimeScript(_)
        | ParsedManagedRole::UnixPrefixRunDirectory
        | ParsedManagedRole::UnixPrincipalProfileSnippet(_) => "unix_a_local",
        ParsedManagedRole::WindowsPrefixPayloadVersion(_)
        | ParsedManagedRole::WindowsPrefixBinEntry(_)
        | ParsedManagedRole::WindowsPrefixShimTree
        | ParsedManagedRole::WindowsPrefixProfileHelper(_)
        | ParsedManagedRole::WindowsPrefixProjection(_)
        | ParsedManagedRole::WindowsPrefixForwarderLogDirectory
        | ParsedManagedRole::WindowsPrincipalProfileSnippet(_) => "windows_a_local",
        ParsedManagedRole::LinuxHostBinary(_)
        | ParsedManagedRole::LinuxHostAclHelper
        | ParsedManagedRole::LinuxHostUnit(_)
        | ParsedManagedRole::LinuxHostServiceState(_)
        | ParsedManagedRole::LinuxHostDirectory(_)
        | ParsedManagedRole::LinuxHostSocket
        | ParsedManagedRole::LinuxHostGroup
        | ParsedManagedRole::LinuxHostMembership(_)
        | ParsedManagedRole::LinuxHostLinger(_)
        | ParsedManagedRole::LinuxHostAclBridge(_, _)
        | ParsedManagedRole::LinuxHostSyntheticAuth
        | ParsedManagedRole::LinuxPublisherExecutor
        | ParsedManagedRole::LinuxPublisherExecutorParent
        | ParsedManagedRole::LinuxPublisherLifecycleContainer
        | ParsedManagedRole::LinuxPublisherStateDirectory
        | ParsedManagedRole::LinuxPublisherServiceUnit
        | ParsedManagedRole::LinuxPublisherSocketUnit
        | ParsedManagedRole::LinuxPublisherEndpoint
        | ParsedManagedRole::LinuxPublisherSigningKey
        | ParsedManagedRole::LinuxPublisherCurrentAnchor
        | ParsedManagedRole::LinuxPublisherBootstrapIntent
        | ParsedManagedRole::LinuxPublisherServiceState(_) => "linux_system",
        ParsedManagedRole::MacPublisherExecutor
        | ParsedManagedRole::MacPublisherPlist
        | ParsedManagedRole::MacPublisherSigningKey
        | ParsedManagedRole::MacPublisherCurrentAnchor
        | ParsedManagedRole::MacPublisherBootstrapIntent
        | ParsedManagedRole::MacPublisherGuestPairingRecord(_)
        | ParsedManagedRole::MacPublisherMachService
        | ParsedManagedRole::MacPublisherServiceState
        | ParsedManagedRole::MacLimaInstance
        | ParsedManagedRole::MacHostKnownHostsEntry
        | ParsedManagedRole::MacAttemptRenderedUnitsTree(_)
        | ParsedManagedRole::MacHostForwardSocket
        | ParsedManagedRole::MacHostSshForwarder => "mac_host_shared",
        ParsedManagedRole::MacLimaStagedWorkspace
        | ParsedManagedRole::MacLimaGuestBinary(_)
        | ParsedManagedRole::MacLimaGuestUnit(_)
        | ParsedManagedRole::MacLimaGuestServiceState(_)
        | ParsedManagedRole::MacLimaGuestDirectory(_)
        | ParsedManagedRole::MacLimaGuestGroup
        | ParsedManagedRole::MacLimaGuestMembership(_)
        | ParsedManagedRole::MacLimaGuestPrivateHome
        | ParsedManagedRole::MacLimaLayoutSentinel
        | ParsedManagedRole::MacLimaPublisherExecutor
        | ParsedManagedRole::MacLimaPublisherStateDirectory
        | ParsedManagedRole::MacLimaPublisherServiceUnit
        | ParsedManagedRole::MacLimaPublisherSocketUnit
        | ParsedManagedRole::MacLimaPublisherEndpoint
        | ParsedManagedRole::MacLimaPublisherSigningKey
        | ParsedManagedRole::MacLimaPublisherCurrentAnchor
        | ParsedManagedRole::MacLimaPublisherBootstrapIntent
        | ParsedManagedRole::MacLimaPublisherServiceState(_)
        | ParsedManagedRole::MacLimaGuestSocket => "mac_lima_guest",
        ParsedManagedRole::WindowsPublisherExecutable
        | ParsedManagedRole::WindowsPublisherProductDirectory
        | ParsedManagedRole::WindowsPublisherInstallDirectory
        | ParsedManagedRole::WindowsPublisherServiceRegistration
        | ParsedManagedRole::WindowsPublisherSigningKey
        | ParsedManagedRole::WindowsPublisherCurrentAnchor
        | ParsedManagedRole::WindowsPublisherBootstrapIntent
        | ParsedManagedRole::WindowsPublisherRegistryContainer(_)
        | ParsedManagedRole::WindowsPublisherGuestPairingRecord(_)
        | ParsedManagedRole::WindowsPublisherEndpoint
        | ParsedManagedRole::WindowsPublisherServiceState
        | ParsedManagedRole::WindowsHostForwarderProcess
        | ParsedManagedRole::WindowsHostForwarderPidRecord
        | ParsedManagedRole::WindowsHostForwarderPipe => "windows_host_shared",
        ParsedManagedRole::WindowsWslInstance
        | ParsedManagedRole::WindowsWslGuestBinary(_)
        | ParsedManagedRole::WindowsWslGuestUnit(_)
        | ParsedManagedRole::WindowsWslGuestServiceState(_)
        | ParsedManagedRole::WindowsWslGuestDirectory(_)
        | ParsedManagedRole::WindowsWslGuestGroup
        | ParsedManagedRole::WindowsWslGuestMembership(_)
        | ParsedManagedRole::WindowsWslGuestSocket
        | ParsedManagedRole::WindowsWslPublisherExecutor
        | ParsedManagedRole::WindowsWslPublisherStateDirectory
        | ParsedManagedRole::WindowsWslPublisherServiceUnit
        | ParsedManagedRole::WindowsWslPublisherSocketUnit
        | ParsedManagedRole::WindowsWslPublisherEndpoint
        | ParsedManagedRole::WindowsWslPublisherSigningKey
        | ParsedManagedRole::WindowsWslPublisherCurrentAnchor
        | ParsedManagedRole::WindowsWslPublisherBootstrapIntent
        | ParsedManagedRole::WindowsWslPublisherServiceState(_) => "windows_wsl_guest",
    };
    // R5's signed generation-N+1 `mac_host_shared/mac_lima` manifest is the sole host anchor
    // for the immediately following guest projections.  Those projected entries retain their
    // `mac_lima_guest` logical-role identity, but never become a separate authority domain or
    // request channel.  No other cross-domain manifest composition is permitted.
    let mac_lima_host_anchor_projection = expected_domain == "mac_lima_guest"
        && manifest.authority_domain == "mac_host_shared"
        && manifest.platform_kind == "mac_lima"
        && manifest.platform_mapping_commitment.is_some();
    if manifest.authority_domain != expected_domain && !mac_lima_host_anchor_projection {
        bail!("manifest entry authority_domain does not match logical_role");
    }
    Ok(())
}

fn validate_object_type_for_role_v1(
    parsed_role: &ParsedManagedRole,
    object_type: &str,
) -> Result<()> {
    let expected = match parsed_role {
        ParsedManagedRole::UnixPrefixPayloadVersion(_)
        | ParsedManagedRole::UnixPrefixShimTree
        | ParsedManagedRole::UnixPrefixRunDirectory
        | ParsedManagedRole::WindowsPrefixPayloadVersion(_)
        | ParsedManagedRole::WindowsPrefixShimTree
        | ParsedManagedRole::WindowsPrefixForwarderLogDirectory
        | ParsedManagedRole::LinuxHostDirectory(_)
        | ParsedManagedRole::LinuxPublisherExecutorParent
        | ParsedManagedRole::LinuxPublisherLifecycleContainer
        | ParsedManagedRole::LinuxPublisherStateDirectory
        | ParsedManagedRole::MacLimaStagedWorkspace
        | ParsedManagedRole::MacLimaGuestDirectory(_)
        | ParsedManagedRole::MacLimaGuestPrivateHome
        | ParsedManagedRole::MacLimaPublisherStateDirectory
        | ParsedManagedRole::MacAttemptRenderedUnitsTree(_)
        | ParsedManagedRole::WindowsPublisherProductDirectory
        | ParsedManagedRole::WindowsPublisherInstallDirectory
        | ParsedManagedRole::WindowsWslGuestDirectory(_)
        | ParsedManagedRole::WindowsWslPublisherStateDirectory => "directory",
        ParsedManagedRole::UnixPrefixBinEntry(_)
        | ParsedManagedRole::UnixPrefixLinuxGuestCache(_)
        | ParsedManagedRole::UnixPrefixDevEnv
        | ParsedManagedRole::UnixPrefixProjection(_)
        | ParsedManagedRole::UnixPrefixRuntimeScript(_)
        | ParsedManagedRole::WindowsPrefixBinEntry(_)
        | ParsedManagedRole::WindowsPrefixProfileHelper(_)
        | ParsedManagedRole::WindowsPrefixProjection(_)
        | ParsedManagedRole::LinuxHostBinary(_)
        | ParsedManagedRole::LinuxHostAclHelper
        | ParsedManagedRole::LinuxHostUnit(_)
        | ParsedManagedRole::LinuxHostSyntheticAuth
        | ParsedManagedRole::LinuxPublisherExecutor
        | ParsedManagedRole::LinuxPublisherServiceUnit
        | ParsedManagedRole::LinuxPublisherSocketUnit
        | ParsedManagedRole::LinuxPublisherSigningKey
        | ParsedManagedRole::LinuxPublisherCurrentAnchor
        | ParsedManagedRole::LinuxPublisherBootstrapIntent
        | ParsedManagedRole::MacPublisherExecutor
        | ParsedManagedRole::MacPublisherPlist
        | ParsedManagedRole::MacLimaGuestBinary(_)
        | ParsedManagedRole::MacLimaGuestUnit(_)
        | ParsedManagedRole::MacLimaLayoutSentinel
        | ParsedManagedRole::MacLimaPublisherExecutor
        | ParsedManagedRole::MacLimaPublisherServiceUnit
        | ParsedManagedRole::MacLimaPublisherSocketUnit
        | ParsedManagedRole::MacLimaPublisherSigningKey
        | ParsedManagedRole::MacLimaPublisherCurrentAnchor
        | ParsedManagedRole::MacLimaPublisherBootstrapIntent
        | ParsedManagedRole::WindowsPublisherExecutable
        | ParsedManagedRole::WindowsHostForwarderPidRecord
        | ParsedManagedRole::WindowsWslGuestBinary(_)
        | ParsedManagedRole::WindowsWslGuestUnit(_)
        | ParsedManagedRole::WindowsWslPublisherExecutor
        | ParsedManagedRole::WindowsWslPublisherServiceUnit
        | ParsedManagedRole::WindowsWslPublisherSocketUnit
        | ParsedManagedRole::WindowsWslPublisherSigningKey
        | ParsedManagedRole::WindowsWslPublisherCurrentAnchor
        | ParsedManagedRole::WindowsWslPublisherBootstrapIntent => "regular-file",
        ParsedManagedRole::UnixPrincipalProfileSnippet(_)
        | ParsedManagedRole::WindowsPrincipalProfileSnippet(_) => "marker-block",
        ParsedManagedRole::LinuxHostServiceState(_)
        | ParsedManagedRole::LinuxPublisherServiceState(_)
        | ParsedManagedRole::MacPublisherServiceState
        | ParsedManagedRole::MacLimaGuestServiceState(_)
        | ParsedManagedRole::MacLimaPublisherServiceState(_)
        | ParsedManagedRole::WindowsPublisherServiceState
        | ParsedManagedRole::WindowsWslGuestServiceState(_)
        | ParsedManagedRole::WindowsWslPublisherServiceState(_) => "service-state",
        ParsedManagedRole::LinuxHostSocket
        | ParsedManagedRole::LinuxPublisherEndpoint
        | ParsedManagedRole::MacLimaPublisherEndpoint
        | ParsedManagedRole::MacLimaGuestSocket
        | ParsedManagedRole::MacHostForwardSocket
        | ParsedManagedRole::WindowsWslGuestSocket
        | ParsedManagedRole::WindowsWslPublisherEndpoint => "unix-socket",
        ParsedManagedRole::LinuxHostGroup
        | ParsedManagedRole::MacLimaGuestGroup
        | ParsedManagedRole::WindowsWslGuestGroup => "group",
        ParsedManagedRole::LinuxHostMembership(_)
        | ParsedManagedRole::MacLimaGuestMembership(_)
        | ParsedManagedRole::WindowsWslGuestMembership(_) => "group-membership",
        ParsedManagedRole::LinuxHostLinger(_) => "linger-state",
        ParsedManagedRole::LinuxHostAclBridge(_, _) => "acl-entry",
        ParsedManagedRole::MacPublisherSigningKey => "keychain-key",
        ParsedManagedRole::MacPublisherCurrentAnchor
        | ParsedManagedRole::MacPublisherBootstrapIntent
        | ParsedManagedRole::MacPublisherGuestPairingRecord(_) => "keychain-record",
        ParsedManagedRole::MacPublisherMachService => "mach-service",
        ParsedManagedRole::MacLimaInstance => "lima-instance",
        ParsedManagedRole::MacHostKnownHostsEntry => "known-hosts-entry",
        ParsedManagedRole::MacHostSshForwarder | ParsedManagedRole::WindowsHostForwarderProcess => {
            "process"
        }
        ParsedManagedRole::WindowsPublisherServiceRegistration => "windows-service",
        ParsedManagedRole::WindowsPublisherSigningKey => "cng-key",
        ParsedManagedRole::WindowsPublisherCurrentAnchor
        | ParsedManagedRole::WindowsPublisherBootstrapIntent
        | ParsedManagedRole::WindowsPublisherGuestPairingRecord(_) => "registry-value",
        ParsedManagedRole::WindowsPublisherRegistryContainer(_) => "registry-key",
        ParsedManagedRole::WindowsPublisherEndpoint
        | ParsedManagedRole::WindowsHostForwarderPipe => "named-pipe",
        ParsedManagedRole::WindowsWslInstance => "wsl-instance",
    };
    if object_type != expected {
        bail!("manifest entry object_type does not match logical_role");
    }
    Ok(())
}

fn validate_action_allowed_for_role_v1(
    parsed_role: &ParsedManagedRole,
    action: ManagedActionV1,
) -> Result<()> {
    let allowed = match parsed_role {
        ParsedManagedRole::UnixPrefixPayloadVersion(_)
        | ParsedManagedRole::UnixPrefixShimTree
        | ParsedManagedRole::UnixPrefixLinuxGuestCache(_)
        | ParsedManagedRole::WindowsPrefixPayloadVersion(_)
        | ParsedManagedRole::WindowsPrefixShimTree => {
            matches!(
                action,
                ManagedActionV1::Create | ManagedActionV1::Replace | ManagedActionV1::Remove
            )
        }
        ParsedManagedRole::UnixPrefixBinEntry(_)
        | ParsedManagedRole::UnixPrefixDevEnv
        | ParsedManagedRole::UnixPrefixProjection(_)
        | ParsedManagedRole::UnixPrefixRuntimeScript(_)
        | ParsedManagedRole::UnixPrincipalProfileSnippet(_)
        | ParsedManagedRole::WindowsPrefixBinEntry(_)
        | ParsedManagedRole::WindowsPrefixProfileHelper(_)
        | ParsedManagedRole::WindowsPrefixProjection(_)
        | ParsedManagedRole::WindowsPrincipalProfileSnippet(_)
        | ParsedManagedRole::LinuxHostBinary(_)
        | ParsedManagedRole::LinuxHostAclHelper
        | ParsedManagedRole::LinuxHostUnit(_)
        | ParsedManagedRole::MacPublisherExecutor
        | ParsedManagedRole::MacLimaStagedWorkspace
        | ParsedManagedRole::MacLimaGuestBinary(_)
        | ParsedManagedRole::MacLimaGuestUnit(_)
        | ParsedManagedRole::MacLimaLayoutSentinel
        | ParsedManagedRole::MacLimaPublisherExecutor
        | ParsedManagedRole::MacHostKnownHostsEntry
        | ParsedManagedRole::WindowsPublisherExecutable
        | ParsedManagedRole::WindowsWslGuestBinary(_)
        | ParsedManagedRole::WindowsWslGuestUnit(_)
        | ParsedManagedRole::WindowsWslPublisherExecutor => matches!(
            action,
            ManagedActionV1::Create
                | ManagedActionV1::Replace
                | ManagedActionV1::Remove
                | ManagedActionV1::Restore
        ),
        ParsedManagedRole::UnixPrefixRunDirectory
        | ParsedManagedRole::WindowsPrefixForwarderLogDirectory
        | ParsedManagedRole::LinuxHostDirectory(_)
        | ParsedManagedRole::LinuxHostGroup
        | ParsedManagedRole::LinuxHostMembership(_)
        | ParsedManagedRole::LinuxHostLinger(_)
        | ParsedManagedRole::LinuxHostAclBridge(_, _)
        | ParsedManagedRole::LinuxHostSyntheticAuth
        | ParsedManagedRole::MacLimaGuestDirectory(_)
        | ParsedManagedRole::MacLimaGuestGroup
        | ParsedManagedRole::MacLimaGuestMembership(_)
        | ParsedManagedRole::MacLimaGuestPrivateHome
        | ParsedManagedRole::MacHostForwardSocket
        | ParsedManagedRole::WindowsHostForwarderPipe
        | ParsedManagedRole::WindowsWslGuestDirectory(_)
        | ParsedManagedRole::WindowsWslGuestGroup
        | ParsedManagedRole::WindowsWslGuestMembership(_) => {
            matches!(
                action,
                ManagedActionV1::Create | ManagedActionV1::Remove | ManagedActionV1::Restore
            )
        }
        ParsedManagedRole::LinuxPublisherExecutorParent
        | ParsedManagedRole::LinuxPublisherLifecycleContainer
        | ParsedManagedRole::LinuxPublisherStateDirectory
        | ParsedManagedRole::MacLimaPublisherStateDirectory
        | ParsedManagedRole::MacAttemptRenderedUnitsTree(_)
        | ParsedManagedRole::WindowsPublisherProductDirectory
        | ParsedManagedRole::WindowsPublisherInstallDirectory
        | ParsedManagedRole::WindowsPublisherRegistryContainer(_)
        | ParsedManagedRole::WindowsWslPublisherStateDirectory => {
            matches!(action, ManagedActionV1::Create | ManagedActionV1::Remove)
        }
        ParsedManagedRole::LinuxHostServiceState(_)
        | ParsedManagedRole::LinuxPublisherServiceState(_)
        | ParsedManagedRole::MacPublisherServiceState
        | ParsedManagedRole::MacLimaGuestServiceState(_)
        | ParsedManagedRole::MacLimaPublisherServiceState(_)
        | ParsedManagedRole::WindowsPublisherServiceState
        | ParsedManagedRole::WindowsWslGuestServiceState(_)
        | ParsedManagedRole::WindowsWslPublisherServiceState(_) => matches!(
            action,
            ManagedActionV1::Enable
                | ManagedActionV1::Disable
                | ManagedActionV1::Start
                | ManagedActionV1::Stop
                | ManagedActionV1::Restore
        ),
        ParsedManagedRole::LinuxHostSocket
        | ParsedManagedRole::LinuxPublisherEndpoint
        | ParsedManagedRole::MacPublisherMachService
        | ParsedManagedRole::MacLimaPublisherEndpoint
        | ParsedManagedRole::MacLimaGuestSocket
        | ParsedManagedRole::WindowsPublisherEndpoint
        | ParsedManagedRole::WindowsWslGuestSocket
        | ParsedManagedRole::WindowsWslPublisherEndpoint => false,
        ParsedManagedRole::LinuxPublisherExecutor
        | ParsedManagedRole::LinuxPublisherServiceUnit
        | ParsedManagedRole::LinuxPublisherSocketUnit
        | ParsedManagedRole::MacPublisherPlist
        | ParsedManagedRole::MacLimaPublisherServiceUnit
        | ParsedManagedRole::MacLimaPublisherSocketUnit
        | ParsedManagedRole::WindowsPublisherServiceRegistration
        | ParsedManagedRole::WindowsWslPublisherServiceUnit
        | ParsedManagedRole::WindowsWslPublisherSocketUnit => {
            matches!(action, ManagedActionV1::Create | ManagedActionV1::Restore)
        }
        ParsedManagedRole::LinuxPublisherSigningKey
        | ParsedManagedRole::MacPublisherSigningKey
        | ParsedManagedRole::MacLimaPublisherSigningKey
        | ParsedManagedRole::WindowsPublisherSigningKey
        | ParsedManagedRole::WindowsWslPublisherSigningKey => {
            matches!(action, ManagedActionV1::Create)
        }
        ParsedManagedRole::LinuxPublisherCurrentAnchor
        | ParsedManagedRole::LinuxPublisherBootstrapIntent
        | ParsedManagedRole::MacPublisherCurrentAnchor
        | ParsedManagedRole::MacPublisherBootstrapIntent
        | ParsedManagedRole::MacPublisherGuestPairingRecord(_)
        | ParsedManagedRole::MacLimaPublisherCurrentAnchor
        | ParsedManagedRole::MacLimaPublisherBootstrapIntent
        | ParsedManagedRole::WindowsPublisherCurrentAnchor
        | ParsedManagedRole::WindowsPublisherBootstrapIntent
        | ParsedManagedRole::WindowsPublisherGuestPairingRecord(_)
        | ParsedManagedRole::WindowsWslPublisherCurrentAnchor
        | ParsedManagedRole::WindowsWslPublisherBootstrapIntent => {
            matches!(action, ManagedActionV1::Create | ManagedActionV1::Replace)
        }
        ParsedManagedRole::WindowsHostForwarderPidRecord => matches!(
            action,
            ManagedActionV1::Create | ManagedActionV1::Replace | ManagedActionV1::Remove
        ),
        ParsedManagedRole::MacLimaInstance => matches!(
            action,
            ManagedActionV1::Create
                | ManagedActionV1::Start
                | ManagedActionV1::Stop
                | ManagedActionV1::Remove
                | ManagedActionV1::Restore
        ),
        ParsedManagedRole::MacHostSshForwarder | ParsedManagedRole::WindowsHostForwarderProcess => {
            matches!(action, ManagedActionV1::Start | ManagedActionV1::Stop)
        }
        ParsedManagedRole::WindowsWslInstance => {
            matches!(
                action,
                ManagedActionV1::Start | ManagedActionV1::Stop | ManagedActionV1::Restore
            )
        }
    };
    if !allowed {
        bail!(
            "logical_role does not allow action {}",
            managed_action_name_v1(action)
        );
    }
    Ok(())
}

fn managed_action_name_v1(action: ManagedActionV1) -> &'static str {
    match action {
        ManagedActionV1::Create => "create",
        ManagedActionV1::Replace => "replace",
        ManagedActionV1::Remove => "remove",
        ManagedActionV1::Restore => "restore",
        ManagedActionV1::Enable => "enable",
        ManagedActionV1::Disable => "disable",
        ManagedActionV1::Start => "start",
        ManagedActionV1::Stop => "stop",
    }
}

fn validate_manifest_entry_identity_for_role_v1(
    manifest: &ManagedArtifactManifestV1,
    entry: &ManagedArtifactEntryV1,
    parsed_role: &ParsedManagedRole,
) -> Result<()> {
    if let Some(expected) = expected_manifest_entry_target_v1(manifest, parsed_role) {
        validate_exact_identity_target_v1(&entry.identity, &expected)?;
        return Ok(());
    }
    match parsed_role {
        ParsedManagedRole::UnixPrincipalProfileSnippet(kind) => {
            validate_suffix_identity_target_v1(&entry.identity, &format!("/{kind}"), kind, None)
        }
        ParsedManagedRole::LinuxHostSyntheticAuth => validate_suffix_identity_target_v1(
            &entry.identity,
            "/.codex/auth.json",
            "auth.json",
            Some("/.codex"),
        ),
        _ => Ok(()),
    }
}

fn expected_manifest_entry_target_v1(
    manifest: &ManagedArtifactManifestV1,
    parsed_role: &ParsedManagedRole,
) -> Option<String> {
    match parsed_role {
        ParsedManagedRole::UnixPrefixPayloadVersion(version) => Some(unix_join_v1(
            &manifest.selected_host_prefix,
            &format!("versions/{version}"),
        )),
        ParsedManagedRole::UnixPrefixBinEntry(binary) => Some(unix_join_v1(
            &manifest.selected_host_prefix,
            &format!("bin/{binary}"),
        )),
        ParsedManagedRole::UnixPrefixShimTree => {
            Some(unix_join_v1(&manifest.selected_host_prefix, "shims"))
        }
        ParsedManagedRole::UnixPrefixLinuxGuestCache(binary) => Some(unix_join_v1(
            &manifest.selected_host_prefix,
            &format!("bin/linux/{binary}"),
        )),
        ParsedManagedRole::UnixPrefixDevEnv => Some(unix_join_v1(
            &manifest.selected_host_prefix,
            "dev-shim-env.sh",
        )),
        ParsedManagedRole::UnixPrefixProjection(kind)
        | ParsedManagedRole::UnixPrefixRuntimeScript(kind) => {
            Some(unix_join_v1(&manifest.selected_host_prefix, kind))
        }
        ParsedManagedRole::UnixPrefixRunDirectory => {
            Some(unix_join_v1(&manifest.selected_host_prefix, "run"))
        }
        ParsedManagedRole::WindowsPrefixPayloadVersion(version) => Some(windows_join_v1(
            &manifest.selected_host_prefix,
            &format!(r"versions\{version}"),
        )),
        ParsedManagedRole::WindowsPrefixBinEntry(binary) => Some(windows_join_v1(
            &manifest.selected_host_prefix,
            &format!(r"bin\{binary}.exe"),
        )),
        ParsedManagedRole::WindowsPrefixShimTree => {
            Some(windows_join_v1(&manifest.selected_host_prefix, r"shims"))
        }
        ParsedManagedRole::WindowsPrefixProfileHelper(kind) => {
            Some(windows_join_v1(&manifest.selected_host_prefix, kind))
        }
        ParsedManagedRole::WindowsPrefixProjection(kind) => Some(windows_join_v1(
            &manifest.selected_host_prefix,
            if kind == "config.yaml" {
                "config.yaml"
            } else {
                r"forwarder\forwarder.toml"
            },
        )),
        ParsedManagedRole::WindowsPrefixForwarderLogDirectory => Some(windows_join_v1(
            &manifest.selected_host_prefix,
            r"forwarder\logs",
        )),
        ParsedManagedRole::LinuxHostBinary(kind) => Some(format!("/usr/local/bin/{kind}")),
        ParsedManagedRole::LinuxHostAclHelper => {
            Some("/usr/libexec/substrate/substrate-apply-socket-acl".to_string())
        }
        ParsedManagedRole::LinuxHostUnit(kind) => Some(match kind.as_str() {
            "service" => "/etc/systemd/system/substrate-world-service.service".to_string(),
            "socket" => "/etc/systemd/system/substrate-world-service.socket".to_string(),
            "socket-drop-in" => {
                "/etc/systemd/system/substrate-world-service.socket.d/20-substrate-group-acl.conf"
                    .to_string()
            }
            _ => unreachable!(),
        }),
        ParsedManagedRole::LinuxHostServiceState(kind) => {
            Some(format!("substrate-world-service.{kind}"))
        }
        ParsedManagedRole::LinuxHostDirectory(kind) => Some(kind.clone()),
        ParsedManagedRole::LinuxHostSocket => Some("/run/substrate.sock".to_string()),
        ParsedManagedRole::LinuxHostGroup => Some("substrate".to_string()),
        ParsedManagedRole::LinuxPublisherExecutor => {
            Some("/usr/libexec/substrate/substrate-lifecycle-linux".to_string())
        }
        ParsedManagedRole::LinuxPublisherExecutorParent => {
            Some("/usr/libexec/substrate".to_string())
        }
        ParsedManagedRole::LinuxPublisherLifecycleContainer => {
            Some("/var/lib/substrate/.substrate-lifecycle-v1".to_string())
        }
        ParsedManagedRole::LinuxPublisherStateDirectory => {
            Some("/var/lib/substrate/.substrate-lifecycle-v1/publisher".to_string())
        }
        ParsedManagedRole::LinuxPublisherServiceUnit => {
            Some("/etc/systemd/system/substrate-lifecycle-publisher-v1.service".to_string())
        }
        ParsedManagedRole::LinuxPublisherSocketUnit => {
            Some("/etc/systemd/system/substrate-lifecycle-publisher-v1.socket".to_string())
        }
        ParsedManagedRole::LinuxPublisherEndpoint => {
            Some("/run/substrate-lifecycle-publisher-v1.sock".to_string())
        }
        ParsedManagedRole::LinuxPublisherSigningKey => {
            Some("/var/lib/substrate/.substrate-lifecycle-v1/publisher/signing-key.v1".to_string())
        }
        ParsedManagedRole::LinuxPublisherCurrentAnchor => Some(
            "/var/lib/substrate/.substrate-lifecycle-v1/publisher/current-anchor.v1.json"
                .to_string(),
        ),
        ParsedManagedRole::LinuxPublisherBootstrapIntent => Some(format!(
            "/var/lib/substrate/.substrate-lifecycle-bootstrap-intent-v1.{}.json",
            manifest.installation_id
        )),
        ParsedManagedRole::LinuxPublisherServiceState(kind) => {
            Some(format!("substrate-lifecycle-publisher-v1.{kind}"))
        }
        ParsedManagedRole::MacPublisherExecutor => {
            Some("/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1".to_string())
        }
        ParsedManagedRole::MacPublisherPlist => {
            Some("/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist".to_string())
        }
        ParsedManagedRole::MacPublisherSigningKey => {
            Some(format!("{}:signing-key", manifest.installation_id))
        }
        ParsedManagedRole::MacPublisherCurrentAnchor => {
            Some(format!("{}:current-anchor", manifest.installation_id))
        }
        ParsedManagedRole::MacPublisherBootstrapIntent => {
            Some(format!("{}:bootstrap-intent", manifest.installation_id))
        }
        ParsedManagedRole::MacPublisherGuestPairingRecord(challenge_id) => Some(format!(
            "{}:guest-pairing:{challenge_id}",
            manifest.installation_id
        )),
        ParsedManagedRole::MacPublisherMachService
        | ParsedManagedRole::MacPublisherServiceState => {
            Some("com.substrate.lifecycle.publisher.v1".to_string())
        }
        ParsedManagedRole::MacLimaStagedWorkspace => {
            Some("/var/lib/substrate/staged-workspace/current".to_string())
        }
        ParsedManagedRole::MacLimaGuestBinary(kind) => Some(match kind.as_str() {
            "substrate-world-service" => "/usr/local/bin/substrate-world-service".to_string(),
            "substrate-gateway" => "/usr/local/bin/substrate-gateway".to_string(),
            "substrate" => "/usr/local/bin/substrate".to_string(),
            _ => unreachable!(),
        }),
        ParsedManagedRole::MacLimaGuestUnit(kind) => Some(format!(
            "/etc/systemd/system/substrate-world-service.{kind}"
        )),
        ParsedManagedRole::MacLimaGuestServiceState(kind) => {
            Some(format!("substrate-world-service.{kind}"))
        }
        ParsedManagedRole::MacLimaGuestDirectory(kind) => Some(kind.clone()),
        ParsedManagedRole::MacLimaGuestGroup => Some("substrate".to_string()),
        ParsedManagedRole::MacLimaLayoutSentinel => Some("/etc/substrate-lima-layout".to_string()),
        ParsedManagedRole::MacLimaPublisherExecutor => {
            Some(guest_publisher_target("mac_lima", "executor"))
        }
        ParsedManagedRole::MacLimaPublisherStateDirectory => {
            Some("/var/lib/substrate/.substrate-lifecycle-v1/publisher".to_string())
        }
        ParsedManagedRole::MacLimaPublisherServiceUnit => {
            Some(guest_publisher_service_unit_target("mac_lima", "service"))
        }
        ParsedManagedRole::MacLimaPublisherSocketUnit => {
            Some(guest_publisher_service_unit_target("mac_lima", "socket"))
        }
        ParsedManagedRole::MacLimaPublisherEndpoint => {
            Some(guest_publisher_target("mac_lima", "endpoint"))
        }
        ParsedManagedRole::MacLimaPublisherSigningKey => {
            Some(guest_publisher_target("mac_lima", "signing-key"))
        }
        ParsedManagedRole::MacLimaPublisherCurrentAnchor => {
            Some(guest_publisher_target("mac_lima", "current-anchor"))
        }
        ParsedManagedRole::MacLimaPublisherServiceState(kind) => {
            Some(format!("substrate-lifecycle-publisher-v1.{kind}"))
        }
        ParsedManagedRole::MacLimaGuestSocket => Some("/run/substrate.sock".to_string()),
        ParsedManagedRole::MacHostKnownHostsEntry => Some(unix_join_v1(
            &manifest.selected_host_prefix,
            "lima_known_hosts",
        )),
        ParsedManagedRole::WindowsPublisherExecutable => {
            Some(r"%ProgramFiles%\Substrate\Lifecycle\substrate-lifecycle-windows.exe".to_string())
        }
        ParsedManagedRole::WindowsPublisherProductDirectory => {
            Some(r"%ProgramFiles%\Substrate".to_string())
        }
        ParsedManagedRole::WindowsPublisherInstallDirectory => {
            Some(r"%ProgramFiles%\Substrate\Lifecycle".to_string())
        }
        ParsedManagedRole::WindowsPublisherServiceRegistration
        | ParsedManagedRole::WindowsPublisherSigningKey
        | ParsedManagedRole::WindowsPublisherServiceState => {
            Some("SubstrateLifecyclePublisherV1".to_string())
        }
        ParsedManagedRole::WindowsPublisherCurrentAnchor => Some(format!(
            r"HKLM\SOFTWARE\Substrate\LifecycleV1\Anchors\{}",
            manifest.installation_id
        )),
        ParsedManagedRole::WindowsPublisherBootstrapIntent => Some(format!(
            r"HKLM\SOFTWARE\Substrate\LifecycleV1\BootstrapIntents\{}",
            manifest.installation_id
        )),
        ParsedManagedRole::WindowsPublisherRegistryContainer(kind) => Some(match kind.as_str() {
            "substrate" => r"HKLM\SOFTWARE\Substrate".to_string(),
            "lifecycle-v1" => r"HKLM\SOFTWARE\Substrate\LifecycleV1".to_string(),
            "anchors" => r"HKLM\SOFTWARE\Substrate\LifecycleV1\Anchors".to_string(),
            "bootstrap-intents" => {
                r"HKLM\SOFTWARE\Substrate\LifecycleV1\BootstrapIntents".to_string()
            }
            "pairings" => r"HKLM\SOFTWARE\Substrate\LifecycleV1\Pairings".to_string(),
            "pairing-scope" => format!(
                r"HKLM\SOFTWARE\Substrate\LifecycleV1\Pairings\{}",
                manifest.installation_id
            ),
            _ => unreachable!(),
        }),
        ParsedManagedRole::WindowsPublisherGuestPairingRecord(challenge_id) => Some(format!(
            r"HKLM\SOFTWARE\Substrate\LifecycleV1\Pairings\{}\{}",
            manifest.installation_id, challenge_id
        )),
        ParsedManagedRole::WindowsPublisherEndpoint => {
            Some(r"\\.\pipe\SubstrateLifecyclePublisherV1".to_string())
        }
        ParsedManagedRole::WindowsWslGuestBinary(kind) => Some(format!("/usr/local/bin/{kind}")),
        ParsedManagedRole::WindowsWslGuestUnit(kind) => Some(format!(
            "/etc/systemd/system/substrate-world-service.{kind}"
        )),
        ParsedManagedRole::WindowsWslGuestServiceState(kind) => {
            Some(format!("substrate-world-service.{kind}"))
        }
        ParsedManagedRole::WindowsWslGuestDirectory(kind) => Some(kind.clone()),
        ParsedManagedRole::WindowsWslGuestGroup => Some("substrate".to_string()),
        ParsedManagedRole::WindowsWslGuestSocket => Some("/run/substrate.sock".to_string()),
        ParsedManagedRole::WindowsWslPublisherExecutor => {
            Some(guest_publisher_target("windows_wsl", "executor"))
        }
        ParsedManagedRole::WindowsWslPublisherStateDirectory => {
            Some("/var/lib/substrate/.substrate-lifecycle-v1/publisher".to_string())
        }
        ParsedManagedRole::WindowsWslPublisherServiceUnit => Some(
            guest_publisher_service_unit_target("windows_wsl", "service"),
        ),
        ParsedManagedRole::WindowsWslPublisherSocketUnit => {
            Some(guest_publisher_service_unit_target("windows_wsl", "socket"))
        }
        ParsedManagedRole::WindowsWslPublisherEndpoint => {
            Some(guest_publisher_target("windows_wsl", "endpoint"))
        }
        ParsedManagedRole::WindowsWslPublisherSigningKey => {
            Some(guest_publisher_target("windows_wsl", "signing-key"))
        }
        ParsedManagedRole::WindowsWslPublisherCurrentAnchor => {
            Some(guest_publisher_target("windows_wsl", "current-anchor"))
        }
        ParsedManagedRole::WindowsWslPublisherServiceState(kind) => {
            Some(format!("substrate-lifecycle-publisher-v1.{kind}"))
        }
        ParsedManagedRole::UnixPrincipalProfileSnippet(_)
        | ParsedManagedRole::WindowsPrincipalProfileSnippet(_)
        | ParsedManagedRole::LinuxHostMembership(_)
        | ParsedManagedRole::LinuxHostLinger(_)
        | ParsedManagedRole::LinuxHostAclBridge(_, _)
        | ParsedManagedRole::LinuxHostSyntheticAuth
        | ParsedManagedRole::MacLimaInstance
        | ParsedManagedRole::MacLimaGuestMembership(_)
        | ParsedManagedRole::MacLimaGuestPrivateHome
        | ParsedManagedRole::MacLimaPublisherBootstrapIntent
        | ParsedManagedRole::MacAttemptRenderedUnitsTree(_)
        | ParsedManagedRole::MacHostForwardSocket
        | ParsedManagedRole::MacHostSshForwarder
        | ParsedManagedRole::WindowsHostForwarderProcess
        | ParsedManagedRole::WindowsHostForwarderPidRecord
        | ParsedManagedRole::WindowsHostForwarderPipe
        | ParsedManagedRole::WindowsWslInstance
        | ParsedManagedRole::WindowsWslGuestMembership(_)
        | ParsedManagedRole::WindowsWslPublisherBootstrapIntent => None,
    }
}

fn validate_exact_identity_target_v1(
    identity: &ManagedArtifactIdentityV1,
    expected: &str,
) -> Result<()> {
    if identity.physical_identity != expected {
        bail!("manifest entry physical_identity does not match the derived target");
    }
    Ok(())
}

fn validate_suffix_identity_target_v1(
    identity: &ManagedArtifactIdentityV1,
    expected_suffix: &str,
    expected_name: &str,
    expected_parent_suffix: Option<&str>,
) -> Result<()> {
    if identity.name_identity != expected_name {
        bail!("manifest entry name_identity does not match the derived target");
    }
    if !identity.physical_identity.ends_with(expected_suffix) {
        bail!("manifest entry physical_identity does not match the derived target");
    }
    if let Some(parent_suffix) = expected_parent_suffix {
        if !identity.parent_identity.ends_with(parent_suffix) {
            bail!("manifest entry parent_identity does not match the derived target");
        }
    }
    Ok(())
}

fn unix_join_v1(prefix: &str, suffix: &str) -> String {
    let suffix = suffix.trim_start_matches('/');
    let trimmed = prefix.trim_end_matches('/');
    if trimmed.is_empty() {
        format!("/{suffix}")
    } else {
        format!("{trimmed}/{suffix}")
    }
}

fn windows_join_v1(prefix: &str, suffix: &str) -> String {
    let suffix = suffix.trim_start_matches('\\');
    let trimmed = prefix.trim_end_matches('\\');
    if trimmed.is_empty() {
        suffix.to_string()
    } else {
        format!(r"{trimmed}\{suffix}")
    }
}

fn validate_identity_v1(identity: &ManagedArtifactIdentityV1) -> Result<()> {
    require_nonempty_no_nul(&identity.scope_id, "identity scope_id")?;
    require_nonempty_no_nul(&identity.parent_identity, "identity parent_identity")?;
    require_nonempty_no_nul(&identity.name_identity, "identity name_identity")?;
    require_nonempty_no_nul(&identity.physical_identity, "identity physical_identity")?;
    if let Some(metadata) = &identity.metadata {
        validate_json_value(metadata, "identity metadata")?;
    }
    Ok(())
}

fn validate_prepared_record_v1(record: &ManagedActionPreparedRecordV1) -> Result<()> {
    require_schema(
        &record.schema_owner,
        record.schema_version,
        ACTION_PREPARED_RECORD_OWNER_V1,
        1,
    )?;
    require_known_authority_domain(&record.authority_domain)?;
    require_uuid_v7(&record.scope_id, "prepared record scope_id")?;
    require_uuid_v7(&record.installation_id, "prepared record installation_id")?;
    if record.scope_id != record.installation_id {
        bail!("prepared record scope_id must match installation_id");
    }
    require_uuid_v7(&record.receipt_id, "prepared record receipt_id")?;
    require_uuid_v7(&record.attempt_id, "prepared record attempt_id")?;
    require_hex_digest(&record.manifest_sha256, "prepared record manifest_sha256")?;
    require_hex_digest(&record.request_sha256, "prepared record request_sha256")?;
    validate_receipt_relative_path(
        &record.receipt_relative_path,
        &record.receipt_id,
        record.manifest_generation,
    )?;
    validate_executor_identity_v1(&record.executor_identity)?;
    if let Some(previous) = &record.previous_record_sha256 {
        require_hex_digest(previous, "prepared record previous_record_sha256")?;
    }
    validate_json_value(
        &record.before_observation,
        "prepared record before_observation",
    )?;
    if record.state != "Prepared" {
        bail!("prepared record state must be Prepared");
    }
    verify_lifecycle_signature_v1(&record.schema_owner, record, &record.signature)
}

fn validate_action_receipt_v1(receipt: &ManagedActionReceiptV1) -> Result<()> {
    require_schema(
        &receipt.schema_owner,
        receipt.schema_version,
        "substrate.managed-action-receipt",
        1,
    )?;
    require_known_authority_domain(&receipt.authority_domain)?;
    require_uuid_v7(&receipt.scope_id, "action receipt scope_id")?;
    require_uuid_v7(&receipt.installation_id, "action receipt installation_id")?;
    if receipt.scope_id != receipt.installation_id {
        bail!("action receipt scope_id must match installation_id");
    }
    require_uuid_v7(&receipt.receipt_id, "action receipt receipt_id")?;
    require_uuid_v7(&receipt.attempt_id, "action receipt attempt_id")?;
    require_hex_digest(&receipt.manifest_sha256, "action receipt manifest_sha256")?;
    require_hex_digest(
        &receipt.prepared_record_sha256,
        "action receipt prepared_record_sha256",
    )?;
    require_hex_digest(&receipt.request_sha256, "action receipt request_sha256")?;
    validate_receipt_relative_path(
        &receipt.receipt_relative_path,
        &receipt.receipt_id,
        receipt.manifest_generation,
    )?;
    validate_json_value(&receipt.pre_observation, "action receipt pre_observation")?;
    validate_json_value(
        &receipt.effect_observation,
        "action receipt effect_observation",
    )?;
    validate_json_value(&receipt.post_observation, "action receipt post_observation")?;
    validate_executor_identity_v1(&receipt.executor_identity)?;
    Ok(())
}

fn validate_action_receipt_index_v1(index: &ManagedActionReceiptIndexV1) -> Result<()> {
    require_schema(
        &index.schema_owner,
        index.schema_version,
        ACTION_RECEIPT_INDEX_OWNER_V1,
        1,
    )?;
    require_known_authority_domain(&index.authority_domain)?;
    require_hex_digest(&index.manifest_sha256, "receipt index manifest_sha256")?;
    if let Some(previous) = &index.previous_index_sha256 {
        require_hex_digest(previous, "receipt index previous_index_sha256")?;
    } else if index.index_revision != 0 {
        bail!("receipt index revision > 0 requires previous_index_sha256");
    }
    let mut seen = BTreeSet::new();
    for entry in &index.entries {
        require_uuid_v7(&entry.receipt_id, "receipt index entry receipt_id")?;
        require_uuid_v7(&entry.attempt_id, "receipt index entry attempt_id")?;
        require_hex_digest(
            &entry.receipt_artifact_sha256,
            "receipt index entry receipt_artifact_sha256",
        )?;
        require_hex_digest(
            &entry.prepared_record_sha256,
            "receipt index entry prepared_record_sha256",
        )?;
        validate_receipt_relative_path(
            &entry.receipt_relative_path,
            &entry.receipt_id,
            index.manifest_generation,
        )?;
        validate_json_value(
            &entry.durable_observation,
            "receipt index entry durable_observation",
        )?;
        if !seen.insert(entry.receipt_id.clone()) {
            bail!("receipt index entry IDs must be unique");
        }
    }
    Ok(())
}

fn validate_manifest_head_v1(head: &ManagedManifestHeadV1) -> Result<()> {
    require_schema(
        &head.schema_owner,
        head.schema_version,
        "substrate.managed-manifest-head",
        1,
    )?;
    require_hex_digest(&head.manifest_sha256, "manifest head manifest_sha256")?;
    require_hex_digest(
        &head.action_receipt_index_sha256,
        "manifest head action_receipt_index_sha256",
    )?;
    if let Some(previous) = &head.previous_head_sha256 {
        require_hex_digest(previous, "manifest head previous_head_sha256")?;
    }
    Ok(())
}

fn validate_lifecycle_anchor_v1(anchor: &LifecyclePublisherAnchorV1) -> Result<()> {
    require_schema(
        &anchor.schema_owner,
        anchor.schema_version,
        "substrate.lifecycle-publisher-anchor",
        1,
    )?;
    require_known_authority_domain(&anchor.authority_domain)?;
    require_uuid_v7(&anchor.scope_id, "publisher anchor scope_id")?;
    require_hex_digest(
        &anchor.host_context_commitment,
        "publisher anchor host_context_commitment",
    )?;
    if let Some(commitment) = &anchor.platform_mapping_commitment {
        require_hex_digest(commitment, "publisher anchor platform_mapping_commitment")?;
    }
    require_hex_digest(&anchor.manifest_sha256, "publisher anchor manifest_sha256")?;
    require_hex_digest(
        &anchor.action_receipt_index_sha256,
        "publisher anchor action_receipt_index_sha256",
    )?;
    require_hex_digest(&anchor.head_sha256, "publisher anchor head_sha256")?;
    require_hex_digest(&anchor.request_sha256, "publisher anchor request_sha256")?;
    if let Some(previous) = &anchor.previous_anchor_sha256 {
        require_hex_digest(previous, "publisher anchor previous_anchor_sha256")?;
    }
    validate_executor_identity_v1(&anchor.executor_identity)?;
    verify_lifecycle_signature_v1(&anchor.schema_owner, anchor, &anchor.signature)
}

fn validate_pairing_challenge_v1(challenge: &GuestPublisherPairingChallengeV1) -> Result<()> {
    require_schema(
        &challenge.schema_owner,
        challenge.schema_version,
        "substrate.guest-publisher-pairing-challenge",
        1,
    )?;
    require_uuid_v7(&challenge.challenge_id, "pairing challenge challenge_id")?;
    decode_base64url(&challenge.challenge, "pairing challenge value")?;
    if challenge.expires_at_unix_ns == 0 {
        bail!("pairing challenge expiry must be positive");
    }
    require_hex_digest(
        &challenge.host_key_fingerprint_sha256,
        "pairing challenge host_key_fingerprint_sha256",
    )?;
    require_hex_digest(
        &challenge.current_anchor_sha256,
        "pairing challenge current_anchor_sha256",
    )?;
    require_hex_digest(
        &challenge.executor_build_evidence_sha256,
        "pairing challenge executor_build_evidence_sha256",
    )?;
    require_hex_digest(
        &challenge.guest_component_commitment_sha256,
        "pairing challenge guest_component_commitment_sha256",
    )?;
    require_hex_digest(
        &challenge.host_context_commitment,
        "pairing challenge host_context_commitment",
    )?;
    if let Some(commitment) = &challenge.platform_mapping_commitment {
        require_hex_digest(commitment, "pairing challenge platform_mapping_commitment")?;
    }
    require_nonempty_no_nul(
        &challenge.guest_machine_identity,
        "pairing challenge guest_machine_identity",
    )?;
    require_git_oid(&challenge.source_commit, "pairing challenge source_commit")?;
    require_git_oid(&challenge.source_tree, "pairing challenge source_tree")?;
    require_nonempty_no_nul(&challenge.source_ref, "pairing challenge source_ref")
}

fn validate_executor_identity_v1(executor: &ManagedExecutorIdentityV1) -> Result<()> {
    require_git_oid(&executor.source_commit, "executor source_commit")?;
    require_git_oid(&executor.source_tree, "executor source_tree")?;
    require_nonempty_no_nul(&executor.source_ref, "executor source_ref")?;
    require_hex_digest(&executor.artifact_sha256, "executor artifact_sha256")?;
    require_nonempty_no_nul(&executor.artifact_path, "executor artifact_path")?;
    require_nonempty_no_nul(&executor.target_triple, "executor target_triple")?;
    if let Some(toolchain) = &executor.toolchain {
        require_nonempty_no_nul(toolchain, "executor toolchain")?;
    }
    if let Some(code_identity) = &executor.code_identity {
        validate_json_value(code_identity, "executor code_identity")?;
    }
    Ok(())
}

fn validate_executor_build_evidence_v1(evidence: &ExecutorBuildEvidenceV1) -> Result<()> {
    require_schema(
        &evidence.schema_owner,
        evidence.schema_version,
        "substrate.executor-build-evidence",
        1,
    )?;
    require_git_oid(&evidence.source_commit, "build evidence source_commit")?;
    require_git_oid(&evidence.source_tree, "build evidence source_tree")?;
    require_nonempty_no_nul(&evidence.source_ref, "build evidence source_ref")?;
    require_hex_digest(&evidence.artifact_sha256, "build evidence artifact_sha256")?;
    require_nonempty_no_nul(
        &evidence.artifact_identity,
        "build evidence artifact_identity",
    )?;
    require_nonempty_no_nul(&evidence.target_triple, "build evidence target_triple")?;
    Ok(())
}

fn validate_test_retirement_commitment_v1(
    commitment: &PublisherTestRetirementCommitmentV1,
) -> Result<()> {
    let public_key = decode_base64url(
        &commitment.harness_public_key,
        "test retirement commitment harness_public_key",
    )?;
    if public_key.len() != 32 {
        bail!("test retirement commitment harness_public_key must be 32 bytes");
    }
    if commitment.harness_algorithm != "ed25519-v1" {
        bail!("test retirement commitment harness_algorithm must be ed25519-v1");
    }
    require_hex_digest(
        &commitment.authorization_digest,
        "test retirement commitment authorization_digest",
    )?;
    require_nonempty_no_nul(
        &commitment.external_parent_identity,
        "test retirement commitment external_parent_identity",
    )?;
    for reservation in &commitment.guest_reservations {
        validate_guest_retirement_reservation_v1(reservation)?;
    }
    Ok(())
}

fn validate_guest_retirement_reservation_v1(
    reservation: &GuestPublisherRetirementReservationV1,
) -> Result<()> {
    require_uuid_v7(
        &reservation.reservation_id,
        "guest retirement reservation_id",
    )?;
    require_uuid_v7(&reservation.challenge_id, "guest retirement challenge_id")?;
    require_nonempty_no_nul(
        &reservation.host_pairing_component_id,
        "guest retirement host_pairing_component_id",
    )?;
    require_hex_digest(
        &reservation.expected_before_sha256,
        "guest retirement expected_before_sha256",
    )?;
    require_nonempty_no_nul(&reservation.state, "guest retirement state")?;
    Ok(())
}

fn validate_publisher_bootstrap_component_v1(
    component: &PublisherBootstrapComponentV1,
) -> Result<()> {
    require_uuid_v7(&component.component_id, "bootstrap component component_id")?;
    parse_component_role(&component.role.0)?;
    require_nonempty_no_nul(
        &component.target_identity,
        "bootstrap component target_identity",
    )?;
    require_nonempty_no_nul(&component.object_type, "bootstrap component object_type")?;
    validate_json_value(
        &component.expected_before,
        "bootstrap component expected_before",
    )?;
    if let Some(hash) = &component.source_artifact_sha256 {
        require_hex_digest(hash, "bootstrap component source_artifact_sha256")?;
    }
    if let Some(hash) = &component.source_signature_sha256 {
        require_hex_digest(hash, "bootstrap component source_signature_sha256")?;
    }
    if let Some(mode) = &component.mode {
        validate_mode_text(mode, "bootstrap component mode")?;
    }
    require_nonempty_no_nul(
        &component.durability_method,
        "bootstrap component durability_method",
    )?;
    Ok(())
}

fn validate_planned_action_receipt_v1(
    value: &Value,
    manifest_generation: u64,
    entry_roles: &BTreeMap<String, ParsedManagedRole>,
) -> Result<PlannedActionReceiptV1> {
    let planned: PlannedActionReceiptV1 = serde_json::from_value(value.clone())
        .context("planned action receipt has invalid shape")?;
    require_uuid_v7(&planned.receipt_id, "planned action receipt receipt_id")?;
    require_uuid_v7(&planned.attempt_id, "planned action receipt attempt_id")?;
    let parsed_role = entry_roles.get(&planned.entry_id).ok_or_else(|| {
        anyhow!("planned action receipt entry_id is not present in manifest entries")
    })?;
    validate_action_allowed_for_role_v1(parsed_role, planned.action)?;
    validate_receipt_relative_path(
        &planned.receipt_relative_path,
        &planned.receipt_id,
        manifest_generation,
    )?;
    Ok(planned)
}

fn validate_receipt_relative_path(
    receipt_relative_path: &str,
    receipt_id: &str,
    manifest_generation: u64,
) -> Result<()> {
    let expected = format!("receipts/{manifest_generation}/receipt.{receipt_id}.json");
    if receipt_relative_path != expected {
        bail!("receipt_relative_path must match the canonical receipt filename");
    }
    Ok(())
}

fn infer_scope_id_from_components(components: &[PublisherBootstrapComponentV1]) -> Result<String> {
    let mut scope_id: Option<String> = None;
    for component in components {
        for dependency in &component.dependency_component_ids {
            if scope_id.is_none() && dependency.contains(':') {
                scope_id = Some(dependency.clone());
            }
        }
        if scope_id.is_none() {
            if let Some(prefix) = component
                .target_identity
                .strip_prefix(r"HKLM\SOFTWARE\Substrate\LifecycleV1\Anchors\")
            {
                scope_id = Some(prefix.to_string());
            } else if let Some(prefix) = component
                .target_identity
                .strip_prefix("/var/lib/substrate/.substrate-lifecycle-bootstrap-intent-v1.")
            {
                scope_id = Some(prefix.trim_end_matches(".json").to_string());
            } else if let Some(prefix) = component.target_identity.strip_suffix(":current-anchor") {
                scope_id = Some(prefix.to_string());
            }
        }
        if scope_id.is_some() {
            break;
        }
    }
    scope_id.ok_or_else(|| anyhow!("unable to infer bootstrap component scope ID"))
}

fn guest_publisher_target(platform: &str, leaf: &str) -> String {
    match (platform, leaf) {
        ("mac_lima", "executor") | ("windows_wsl", "executor") => {
            "/usr/libexec/substrate/substrate-lifecycle-linux".to_string()
        }
        ("mac_lima", "endpoint") | ("windows_wsl", "endpoint") => {
            "/run/substrate-lifecycle-publisher-v1.sock".to_string()
        }
        ("mac_lima", "signing-key") | ("windows_wsl", "signing-key") => {
            "/var/lib/substrate/.substrate-lifecycle-v1/publisher/signing-key.v1".to_string()
        }
        ("mac_lima", "current-anchor") | ("windows_wsl", "current-anchor") => {
            "/var/lib/substrate/.substrate-lifecycle-v1/publisher/current-anchor.v1.json"
                .to_string()
        }
        _ => unreachable!(),
    }
}

fn guest_publisher_service_unit_target(platform: &str, kind: &str) -> String {
    let _ = platform;
    format!("/etc/systemd/system/substrate-lifecycle-publisher-v1.{kind}")
}

fn require_schema(
    actual_owner: &str,
    actual_version: u32,
    expected_owner: &str,
    expected_version: u32,
) -> Result<()> {
    if actual_owner != expected_owner {
        bail!("unexpected schema_owner {actual_owner}");
    }
    if actual_version != expected_version {
        bail!("unexpected schema_version {actual_version}");
    }
    Ok(())
}

fn require_known_authority_domain(domain: &str) -> Result<()> {
    match domain {
        "unix_a_local"
        | "windows_a_local"
        | "linux_system"
        | "mac_host_shared"
        | "windows_host_shared"
        | "mac_lima_guest"
        | "windows_wsl_guest" => Ok(()),
        _ => bail!("unknown authority domain {domain}"),
    }
}

fn require_known_platform_kind(kind: &str) -> Result<()> {
    match kind {
        "unix" | "windows" | "linux" | "mac_lima" | "windows_wsl" => Ok(()),
        _ => bail!("unknown platform kind {kind}"),
    }
}

fn require_uuid_v7(value: &str, field: &str) -> Result<()> {
    let uuid = Uuid::parse_str(value).with_context(|| format!("{field} is not a UUID"))?;
    if uuid.get_version_num() != 7 {
        bail!("{field} must be a UUIDv7");
    }
    Ok(())
}

fn require_hex_digest(value: &str, field: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("{field} must be a lowercase 64-hex digest");
    }
    Ok(())
}

fn require_git_oid(value: &str, field: &str) -> Result<()> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("{field} must be a lowercase 40-hex git object ID");
    }
    Ok(())
}

fn require_nonempty_no_nul(value: &str, field: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{field} must be non-empty");
    }
    if value.as_bytes().contains(&0) {
        bail!("{field} must not contain NUL");
    }
    Ok(())
}

fn validate_mode_text(value: &str, field: &str) -> Result<()> {
    if value.len() != 4 || !value.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) {
        bail!("{field} must be a four-digit octal mode");
    }
    Ok(())
}

fn decode_base64url(value: &str, field: &str) -> Result<Vec<u8>> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .with_context(|| format!("{field} is not valid base64url"))?;
    if URL_SAFE_NO_PAD.encode(&bytes) != value {
        bail!("{field} is not canonical base64url");
    }
    Ok(bytes)
}

fn validate_json_value(value: &Value, field: &str) -> Result<()> {
    canonical_json_value_to_vec(value)
        .with_context(|| format!("{field} is not canonicalizable"))
        .map(|_| ())
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn parse_managed_role(value: &str) -> Result<ParsedManagedRole> {
    let parsed = if let Some(version) = parse_single_argument(value, "unix.prefix.payload-version")?
    {
        validate_version_param(&version)?;
        ParsedManagedRole::UnixPrefixPayloadVersion(version)
    } else if let Some(binary) = parse_single_argument(value, "unix.prefix.bin-entry")? {
        validate_host_binary(&binary)?;
        ParsedManagedRole::UnixPrefixBinEntry(binary)
    } else if value == "unix.prefix.shim-tree" {
        ParsedManagedRole::UnixPrefixShimTree
    } else if let Some(binary) = parse_single_argument(value, "unix.prefix.linux-guest-cache")? {
        validate_linux_guest_cache_binary(&binary)?;
        ParsedManagedRole::UnixPrefixLinuxGuestCache(binary)
    } else if value == "unix.prefix.dev-env" {
        ParsedManagedRole::UnixPrefixDevEnv
    } else if let Some(kind) = parse_single_argument(value, "unix.prefix.projection")? {
        validate_one_of(
            &kind,
            &[
                "config.yaml",
                "env.sh",
                "manager_init.sh",
                "manager_env.sh",
                "manager_hooks.yaml",
            ],
            "unix.prefix.projection kind",
        )?;
        ParsedManagedRole::UnixPrefixProjection(kind)
    } else if let Some(kind) = parse_single_argument(value, "unix.prefix.runtime-script")? {
        validate_one_of(
            &kind,
            &[
                "scripts/substrate/world-enable.sh",
                "scripts/substrate/install-substrate.sh",
                "scripts/substrate/world-deps.yaml",
                "scripts/mac/lima-warm.sh",
                "scripts/mac/lima/substrate.yaml",
                "scripts/mac/lima/substrate-dev.yaml",
            ],
            "unix.prefix.runtime-script kind",
        )?;
        ParsedManagedRole::UnixPrefixRuntimeScript(kind)
    } else if value == "unix.prefix.run-directory" {
        ParsedManagedRole::UnixPrefixRunDirectory
    } else if let Some(kind) = parse_single_argument(value, "unix.principal.profile-snippet")? {
        validate_one_of(
            &kind,
            &[
                ".bashrc",
                ".bash_profile",
                ".profile",
                ".zshrc",
                ".zprofile",
                ".config/fish/config.fish",
            ],
            "unix.principal.profile-snippet kind",
        )?;
        ParsedManagedRole::UnixPrincipalProfileSnippet(kind)
    } else if let Some(version) = parse_single_argument(value, "windows.prefix.payload-version")? {
        validate_version_param(&version)?;
        ParsedManagedRole::WindowsPrefixPayloadVersion(version)
    } else if let Some(binary) = parse_single_argument(value, "windows.prefix.bin-entry")? {
        validate_host_binary(&binary)?;
        ParsedManagedRole::WindowsPrefixBinEntry(binary)
    } else if value == "windows.prefix.shim-tree" {
        ParsedManagedRole::WindowsPrefixShimTree
    } else if let Some(kind) = parse_single_argument(value, "windows.prefix.profile-helper")? {
        validate_one_of(
            &kind,
            &["substrate-profile.ps1", "dev-substrate-profile.ps1"],
            "windows.prefix.profile-helper kind",
        )?;
        ParsedManagedRole::WindowsPrefixProfileHelper(kind)
    } else if let Some(kind) = parse_single_argument(value, "windows.prefix.projection")? {
        validate_one_of(
            &kind,
            &["config.yaml", "forwarder.toml"],
            "windows.prefix.projection kind",
        )?;
        ParsedManagedRole::WindowsPrefixProjection(kind)
    } else if value == "windows.prefix.forwarder-log-directory" {
        ParsedManagedRole::WindowsPrefixForwarderLogDirectory
    } else if let Some(kind) = parse_single_argument(value, "windows.principal.profile-snippet")? {
        validate_one_of(
            &kind,
            &["CurrentUserAllHosts", "CurrentUserCurrentHost"],
            "windows.principal.profile-snippet kind",
        )?;
        ParsedManagedRole::WindowsPrincipalProfileSnippet(kind)
    } else if let Some(kind) = parse_single_argument(value, "linux.host.binary")? {
        validate_one_of(
            &kind,
            &["substrate-world-service", "substrate-gateway"],
            "linux.host.binary kind",
        )?;
        ParsedManagedRole::LinuxHostBinary(kind)
    } else if value == "linux.host.acl-helper" {
        ParsedManagedRole::LinuxHostAclHelper
    } else if let Some(kind) = parse_single_argument(value, "linux.host.unit")? {
        validate_one_of(
            &kind,
            &["service", "socket", "socket-drop-in"],
            "linux.host.unit kind",
        )?;
        ParsedManagedRole::LinuxHostUnit(kind)
    } else if let Some(kind) = parse_single_argument(value, "linux.host.service-state")? {
        validate_one_of(
            &kind,
            &["service", "socket"],
            "linux.host.service-state kind",
        )?;
        ParsedManagedRole::LinuxHostServiceState(kind)
    } else if let Some(kind) = parse_single_argument(value, "linux.host.directory")? {
        validate_one_of(
            &kind,
            &[
                "/run/substrate",
                "/var/lib/substrate",
                "/var/lib/substrate/world-deps",
                "/var/lib/substrate/world-deps/bin",
            ],
            "linux.host.directory kind",
        )?;
        ParsedManagedRole::LinuxHostDirectory(kind)
    } else if value == "linux.host.socket" {
        ParsedManagedRole::LinuxHostSocket
    } else if value == "linux.host.group" {
        ParsedManagedRole::LinuxHostGroup
    } else if let Some(principal) = parse_single_argument(value, "linux.host.membership")? {
        require_nonempty_no_nul(&principal, "linux.host.membership principal")?;
        ParsedManagedRole::LinuxHostMembership(principal)
    } else if let Some(principal) = parse_single_argument(value, "linux.host.linger")? {
        require_nonempty_no_nul(&principal, "linux.host.linger principal")?;
        ParsedManagedRole::LinuxHostLinger(principal)
    } else if let Some((kind, principal)) = parse_two_arguments(value, "linux.host.acl-bridge")? {
        validate_one_of(
            &kind,
            &["socket", "state-tree-traversal", "world-deps-read-only"],
            "linux.host.acl-bridge kind",
        )?;
        require_nonempty_no_nul(&principal, "linux.host.acl-bridge principal")?;
        ParsedManagedRole::LinuxHostAclBridge(kind, principal)
    } else if value == "linux.host.synthetic-auth" {
        ParsedManagedRole::LinuxHostSyntheticAuth
    } else if value == "linux.publisher.executor" {
        ParsedManagedRole::LinuxPublisherExecutor
    } else if value == "linux.publisher.executor-parent" {
        ParsedManagedRole::LinuxPublisherExecutorParent
    } else if value == "linux.publisher.lifecycle-container" {
        ParsedManagedRole::LinuxPublisherLifecycleContainer
    } else if value == "linux.publisher.state-directory" {
        ParsedManagedRole::LinuxPublisherStateDirectory
    } else if value == "linux.publisher.service-unit" {
        ParsedManagedRole::LinuxPublisherServiceUnit
    } else if value == "linux.publisher.socket-unit" {
        ParsedManagedRole::LinuxPublisherSocketUnit
    } else if value == "linux.publisher.endpoint" {
        ParsedManagedRole::LinuxPublisherEndpoint
    } else if value == "linux.publisher.signing-key" {
        ParsedManagedRole::LinuxPublisherSigningKey
    } else if value == "linux.publisher.current-anchor" {
        ParsedManagedRole::LinuxPublisherCurrentAnchor
    } else if value == "linux.publisher.bootstrap-intent" {
        ParsedManagedRole::LinuxPublisherBootstrapIntent
    } else if let Some(kind) = parse_single_argument(value, "linux.publisher.service-state")? {
        validate_one_of(
            &kind,
            &["service", "socket"],
            "linux.publisher.service-state kind",
        )?;
        ParsedManagedRole::LinuxPublisherServiceState(kind)
    } else if value == "mac.publisher.executor" {
        ParsedManagedRole::MacPublisherExecutor
    } else if value == "mac.publisher.plist" {
        ParsedManagedRole::MacPublisherPlist
    } else if value == "mac.publisher.signing-key" {
        ParsedManagedRole::MacPublisherSigningKey
    } else if value == "mac.publisher.current-anchor" {
        ParsedManagedRole::MacPublisherCurrentAnchor
    } else if value == "mac.publisher.bootstrap-intent" {
        ParsedManagedRole::MacPublisherBootstrapIntent
    } else if let Some(challenge_id) =
        parse_single_argument(value, "mac.publisher.guest-pairing-record")?
    {
        require_uuid_v7(
            &challenge_id,
            "mac.publisher.guest-pairing-record challenge_id",
        )?;
        ParsedManagedRole::MacPublisherGuestPairingRecord(challenge_id)
    } else if value == "mac.publisher.mach-service" {
        ParsedManagedRole::MacPublisherMachService
    } else if value == "mac.publisher.service-state" {
        ParsedManagedRole::MacPublisherServiceState
    } else if value == "mac.lima.instance" {
        ParsedManagedRole::MacLimaInstance
    } else if value == "mac.lima.staged-workspace" {
        ParsedManagedRole::MacLimaStagedWorkspace
    } else if let Some(kind) = parse_single_argument(value, "mac.lima.guest-binary")? {
        validate_one_of(
            &kind,
            &["substrate-world-service", "substrate-gateway", "substrate"],
            "mac.lima.guest-binary kind",
        )?;
        ParsedManagedRole::MacLimaGuestBinary(kind)
    } else if let Some(kind) = parse_single_argument(value, "mac.lima.guest-unit")? {
        validate_one_of(&kind, &["service", "socket"], "mac.lima.guest-unit kind")?;
        ParsedManagedRole::MacLimaGuestUnit(kind)
    } else if let Some(kind) = parse_single_argument(value, "mac.lima.guest-service-state")? {
        validate_one_of(
            &kind,
            &["service", "socket"],
            "mac.lima.guest-service-state kind",
        )?;
        ParsedManagedRole::MacLimaGuestServiceState(kind)
    } else if let Some(kind) = parse_single_argument(value, "mac.lima.guest-directory")? {
        validate_one_of(
            &kind,
            &[
                "/run/substrate",
                "/run/substrate/substrate-gateway-runtime",
                "/var/lib/substrate",
                "/var/lib/substrate/.substrate-lifecycle-v1",
                "/var/lib/substrate/staged-workspace",
                "/usr/libexec/substrate",
            ],
            "mac.lima.guest-directory kind",
        )?;
        ParsedManagedRole::MacLimaGuestDirectory(kind)
    } else if value == "mac.lima.guest-group" {
        ParsedManagedRole::MacLimaGuestGroup
    } else if let Some(principal) = parse_single_argument(value, "mac.lima.guest-membership")? {
        require_nonempty_no_nul(&principal, "mac.lima.guest-membership principal")?;
        ParsedManagedRole::MacLimaGuestMembership(principal)
    } else if value == "mac.lima.guest-private-home" {
        ParsedManagedRole::MacLimaGuestPrivateHome
    } else if value == "mac.lima.layout-sentinel" {
        ParsedManagedRole::MacLimaLayoutSentinel
    } else if value == "mac.lima.publisher-executor" {
        ParsedManagedRole::MacLimaPublisherExecutor
    } else if value == "mac.lima.publisher-state-directory" {
        ParsedManagedRole::MacLimaPublisherStateDirectory
    } else if value == "mac.lima.publisher-service-unit" {
        ParsedManagedRole::MacLimaPublisherServiceUnit
    } else if value == "mac.lima.publisher-socket-unit" {
        ParsedManagedRole::MacLimaPublisherSocketUnit
    } else if value == "mac.lima.publisher-endpoint" {
        ParsedManagedRole::MacLimaPublisherEndpoint
    } else if value == "mac.lima.publisher-signing-key" {
        ParsedManagedRole::MacLimaPublisherSigningKey
    } else if value == "mac.lima.publisher-current-anchor" {
        ParsedManagedRole::MacLimaPublisherCurrentAnchor
    } else if value == "mac.lima.publisher-bootstrap-intent" {
        ParsedManagedRole::MacLimaPublisherBootstrapIntent
    } else if let Some(kind) = parse_single_argument(value, "mac.lima.publisher-service-state")? {
        validate_one_of(
            &kind,
            &["service", "socket"],
            "mac.lima.publisher-service-state kind",
        )?;
        ParsedManagedRole::MacLimaPublisherServiceState(kind)
    } else if value == "mac.lima.guest-socket" {
        ParsedManagedRole::MacLimaGuestSocket
    } else if value == "mac.host.known-hosts-entry" {
        ParsedManagedRole::MacHostKnownHostsEntry
    } else if let Some(attempt_id) =
        parse_single_argument(value, "mac.attempt.rendered-units-tree")?
    {
        require_uuid_v7(&attempt_id, "mac.attempt.rendered-units-tree attempt_id")?;
        ParsedManagedRole::MacAttemptRenderedUnitsTree(attempt_id)
    } else if value == "mac.host.forward-socket" {
        ParsedManagedRole::MacHostForwardSocket
    } else if value == "mac.host.ssh-forwarder" {
        ParsedManagedRole::MacHostSshForwarder
    } else if value == "windows.publisher.executable" {
        ParsedManagedRole::WindowsPublisherExecutable
    } else if value == "windows.publisher.product-directory" {
        ParsedManagedRole::WindowsPublisherProductDirectory
    } else if value == "windows.publisher.install-directory" {
        ParsedManagedRole::WindowsPublisherInstallDirectory
    } else if value == "windows.publisher.service-registration" {
        ParsedManagedRole::WindowsPublisherServiceRegistration
    } else if value == "windows.publisher.signing-key" {
        ParsedManagedRole::WindowsPublisherSigningKey
    } else if value == "windows.publisher.current-anchor" {
        ParsedManagedRole::WindowsPublisherCurrentAnchor
    } else if value == "windows.publisher.bootstrap-intent" {
        ParsedManagedRole::WindowsPublisherBootstrapIntent
    } else if let Some(kind) = parse_single_argument(value, "windows.publisher.registry-container")?
    {
        validate_one_of(
            &kind,
            &[
                "substrate",
                "lifecycle-v1",
                "anchors",
                "bootstrap-intents",
                "pairings",
                "pairing-scope",
            ],
            "windows.publisher.registry-container kind",
        )?;
        ParsedManagedRole::WindowsPublisherRegistryContainer(kind)
    } else if let Some(challenge_id) =
        parse_single_argument(value, "windows.publisher.guest-pairing-record")?
    {
        require_uuid_v7(
            &challenge_id,
            "windows.publisher.guest-pairing-record challenge_id",
        )?;
        ParsedManagedRole::WindowsPublisherGuestPairingRecord(challenge_id)
    } else if value == "windows.publisher.endpoint" {
        ParsedManagedRole::WindowsPublisherEndpoint
    } else if value == "windows.publisher.service-state" {
        ParsedManagedRole::WindowsPublisherServiceState
    } else if value == "windows.host.forwarder-process" {
        ParsedManagedRole::WindowsHostForwarderProcess
    } else if value == "windows.host.forwarder-pid-record" {
        ParsedManagedRole::WindowsHostForwarderPidRecord
    } else if value == "windows.host.forwarder-pipe" {
        ParsedManagedRole::WindowsHostForwarderPipe
    } else if value == "windows.wsl.instance" {
        ParsedManagedRole::WindowsWslInstance
    } else if let Some(kind) = parse_single_argument(value, "windows.wsl.guest-binary")? {
        validate_one_of(
            &kind,
            &["substrate-world-service", "substrate-gateway"],
            "windows.wsl.guest-binary kind",
        )?;
        ParsedManagedRole::WindowsWslGuestBinary(kind)
    } else if let Some(kind) = parse_single_argument(value, "windows.wsl.guest-unit")? {
        validate_one_of(&kind, &["service", "socket"], "windows.wsl.guest-unit kind")?;
        ParsedManagedRole::WindowsWslGuestUnit(kind)
    } else if let Some(kind) = parse_single_argument(value, "windows.wsl.guest-service-state")? {
        validate_one_of(
            &kind,
            &["service", "socket"],
            "windows.wsl.guest-service-state kind",
        )?;
        ParsedManagedRole::WindowsWslGuestServiceState(kind)
    } else if let Some(kind) = parse_single_argument(value, "windows.wsl.guest-directory")? {
        validate_one_of(
            &kind,
            &[
                "/run/substrate",
                "/var/lib/substrate",
                "/var/lib/substrate/.substrate-lifecycle-v1",
                "/usr/libexec/substrate",
            ],
            "windows.wsl.guest-directory kind",
        )?;
        ParsedManagedRole::WindowsWslGuestDirectory(kind)
    } else if value == "windows.wsl.guest-group" {
        ParsedManagedRole::WindowsWslGuestGroup
    } else if let Some(principal) = parse_single_argument(value, "windows.wsl.guest-membership")? {
        require_nonempty_no_nul(&principal, "windows.wsl.guest-membership principal")?;
        ParsedManagedRole::WindowsWslGuestMembership(principal)
    } else if value == "windows.wsl.guest-socket" {
        ParsedManagedRole::WindowsWslGuestSocket
    } else if value == "windows.wsl.publisher-executor" {
        ParsedManagedRole::WindowsWslPublisherExecutor
    } else if value == "windows.wsl.publisher-state-directory" {
        ParsedManagedRole::WindowsWslPublisherStateDirectory
    } else if value == "windows.wsl.publisher-service-unit" {
        ParsedManagedRole::WindowsWslPublisherServiceUnit
    } else if value == "windows.wsl.publisher-socket-unit" {
        ParsedManagedRole::WindowsWslPublisherSocketUnit
    } else if value == "windows.wsl.publisher-endpoint" {
        ParsedManagedRole::WindowsWslPublisherEndpoint
    } else if value == "windows.wsl.publisher-signing-key" {
        ParsedManagedRole::WindowsWslPublisherSigningKey
    } else if value == "windows.wsl.publisher-current-anchor" {
        ParsedManagedRole::WindowsWslPublisherCurrentAnchor
    } else if value == "windows.wsl.publisher-bootstrap-intent" {
        ParsedManagedRole::WindowsWslPublisherBootstrapIntent
    } else if let Some(kind) = parse_single_argument(value, "windows.wsl.publisher-service-state")?
    {
        validate_one_of(
            &kind,
            &["service", "socket"],
            "windows.wsl.publisher-service-state kind",
        )?;
        ParsedManagedRole::WindowsWslPublisherServiceState(kind)
    } else {
        bail!("unknown managed artifact role {value}");
    };
    Ok(parsed)
}

fn parse_component_role(value: &str) -> Result<ParsedComponentRole> {
    Ok(match value {
        "LinuxStateRoot" => ParsedComponentRole::LinuxStateRoot,
        "LinuxLifecycleContainer" => ParsedComponentRole::LinuxLifecycleContainer,
        "LinuxPublisherDirectory" => ParsedComponentRole::LinuxPublisherDirectory,
        "LinuxExecutorParent" => ParsedComponentRole::LinuxExecutorParent,
        "LinuxBootstrapIntent" => ParsedComponentRole::LinuxBootstrapIntent,
        "LinuxExecutor" => ParsedComponentRole::LinuxExecutor,
        "LinuxEndpoint" => ParsedComponentRole::LinuxEndpoint,
        "LinuxSigningKey" => ParsedComponentRole::LinuxSigningKey,
        "LinuxProtectedState" => ParsedComponentRole::LinuxProtectedState,
        "MacExecutor" => ParsedComponentRole::MacExecutor,
        "MacPlist" => ParsedComponentRole::MacPlist,
        "MacMachService" => ParsedComponentRole::MacMachService,
        "MacSigningKey" => ParsedComponentRole::MacSigningKey,
        "MacProtectedState" => ParsedComponentRole::MacProtectedState,
        "MacBootstrapIntent" => ParsedComponentRole::MacBootstrapIntent,
        "MacServiceState" => ParsedComponentRole::MacServiceState,
        "WindowsProductDirectory" => ParsedComponentRole::WindowsProductDirectory,
        "WindowsInstallDirectory" => ParsedComponentRole::WindowsInstallDirectory,
        "WindowsExecutable" => ParsedComponentRole::WindowsExecutable,
        "WindowsServiceRegistration" => ParsedComponentRole::WindowsServiceRegistration,
        "WindowsSigningKey" => ParsedComponentRole::WindowsSigningKey,
        "WindowsProtectedState" => ParsedComponentRole::WindowsProtectedState,
        "WindowsBootstrapIntent" => ParsedComponentRole::WindowsBootstrapIntent,
        "WindowsEndpoint" => ParsedComponentRole::WindowsEndpoint,
        "WindowsServiceState" => ParsedComponentRole::WindowsServiceState,
        _ => {
            if let Some(kind) = parse_single_argument(value, "LinuxServiceUnit")? {
                validate_one_of(&kind, &["service", "socket"], "LinuxServiceUnit kind")?;
                ParsedComponentRole::LinuxServiceUnit(kind)
            } else if let Some(kind) = parse_single_argument(value, "LinuxServiceState")? {
                validate_one_of(&kind, &["service", "socket"], "LinuxServiceState kind")?;
                ParsedComponentRole::LinuxServiceState(kind)
            } else if let Some(challenge_id) =
                parse_single_argument(value, "MacGuestPairingRecord")?
            {
                require_uuid_v7(&challenge_id, "MacGuestPairingRecord challenge_id")?;
                ParsedComponentRole::MacGuestPairingRecord(challenge_id)
            } else if let Some(kind) = parse_single_argument(value, "WindowsRegistryContainer")? {
                validate_one_of(
                    &kind,
                    &[
                        "substrate",
                        "lifecycle-v1",
                        "anchors",
                        "bootstrap-intents",
                        "pairings",
                        "pairing-scope",
                    ],
                    "WindowsRegistryContainer kind",
                )?;
                ParsedComponentRole::WindowsRegistryContainer(kind)
            } else if let Some(challenge_id) =
                parse_single_argument(value, "WindowsGuestPairingRecord")?
            {
                require_uuid_v7(&challenge_id, "WindowsGuestPairingRecord challenge_id")?;
                ParsedComponentRole::WindowsGuestPairingRecord(challenge_id)
            } else if let Some(platform) = parse_single_argument(value, "GuestStateRoot")? {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestStateRoot(platform)
            } else if let Some(platform) = parse_single_argument(value, "GuestLifecycleContainer")?
            {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestLifecycleContainer(platform)
            } else if let Some(platform) = parse_single_argument(value, "GuestPublisherDirectory")?
            {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestPublisherDirectory(platform)
            } else if let Some(platform) = parse_single_argument(value, "GuestExecutorParent")? {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestExecutorParent(platform)
            } else if let Some((platform, challenge_id)) =
                parse_two_arguments(value, "GuestBootstrapIntent")?
            {
                validate_guest_platform(&platform)?;
                require_uuid_v7(&challenge_id, "GuestBootstrapIntent challenge_id")?;
                ParsedComponentRole::GuestBootstrapIntent(platform, challenge_id)
            } else if let Some(platform) = parse_single_argument(value, "GuestExecutor")? {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestExecutor(platform)
            } else if let Some((platform, kind)) = parse_two_arguments(value, "GuestServiceUnit")? {
                validate_guest_platform(&platform)?;
                validate_one_of(&kind, &["service", "socket"], "GuestServiceUnit kind")?;
                ParsedComponentRole::GuestServiceUnit(platform, kind)
            } else if let Some(platform) = parse_single_argument(value, "GuestEndpoint")? {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestEndpoint(platform)
            } else if let Some(platform) = parse_single_argument(value, "GuestSigningKey")? {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestSigningKey(platform)
            } else if let Some(platform) = parse_single_argument(value, "GuestProtectedState")? {
                validate_guest_platform(&platform)?;
                ParsedComponentRole::GuestProtectedState(platform)
            } else if let Some((platform, kind)) = parse_two_arguments(value, "GuestServiceState")?
            {
                validate_guest_platform(&platform)?;
                validate_one_of(&kind, &["service", "socket"], "GuestServiceState kind")?;
                ParsedComponentRole::GuestServiceState(platform, kind)
            } else {
                bail!("unknown publisher bootstrap component role {value}");
            }
        }
    })
}

fn validate_guest_platform(value: &str) -> Result<()> {
    validate_one_of(value, &["mac_lima", "windows_wsl"], "guest platform")
}

fn validate_one_of(value: &str, expected: &[&str], field: &str) -> Result<()> {
    if expected.iter().any(|item| item == &value) {
        Ok(())
    } else {
        bail!("{field} must be one of {}", expected.join(", "))
    }
}

fn validate_host_binary(value: &str) -> Result<()> {
    validate_one_of(
        value,
        &[
            "substrate",
            "substrate-shim",
            "substrate-forwarder",
            "host-proxy",
            "world-service",
            "substrate-world-service",
            "substrate-gateway",
            "substrate-lifecycle-control",
            "substrate-lifecycle-linux",
            "substrate-lifecycle-macos",
            "substrate-lifecycle-windows",
            "substrate-lifecycle-unix",
        ],
        "host binary",
    )
}

fn validate_linux_guest_cache_binary(value: &str) -> Result<()> {
    validate_one_of(
        value,
        &["substrate", "world-service", "substrate-gateway"],
        "linux guest cache binary",
    )
}

fn validate_version_param(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 64
        || value == "."
        || value == ".."
        || value
            .bytes()
            .any(|byte| byte == b'/' || byte == b'\\' || byte == b':' || byte == 0 || byte == b' ')
        || value.ends_with('.')
    {
        bail!("version parameter is not canonical");
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        bail!("version parameter is empty");
    };
    if !first.is_ascii_alphanumeric() {
        bail!("version parameter must begin with an ASCII alphanumeric");
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-')) {
        bail!("version parameter contains an invalid character");
    }
    Ok(())
}

fn parse_single_argument(value: &str, prefix: &str) -> Result<Option<String>> {
    let prefix = format!("{prefix}(");
    if !value.starts_with(&prefix) || !value.ends_with(')') {
        return Ok(None);
    }
    let inner = &value[prefix.len()..value.len() - 1];
    if inner.is_empty() || inner.contains(',') {
        bail!("role argument must contain exactly one parameter");
    }
    Ok(Some(inner.to_string()))
}

fn parse_two_arguments(value: &str, prefix: &str) -> Result<Option<(String, String)>> {
    let prefix = format!("{prefix}(");
    if !value.starts_with(&prefix) || !value.ends_with(')') {
        return Ok(None);
    }
    let inner = &value[prefix.len()..value.len() - 1];
    let mut parts = inner.split(',');
    let Some(first) = parts.next() else {
        bail!("role argument list must contain two parameters");
    };
    let Some(second) = parts.next() else {
        bail!("role argument list must contain two parameters");
    };
    if parts.next().is_some() || first.is_empty() || second.is_empty() {
        bail!("role argument list must contain exactly two parameters");
    }
    Ok(Some((first.to_string(), second.to_string())))
}

fn canonical_json_to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value).context("serialize canonical JSON value")?;
    canonical_json_value_to_vec(&value)
}

fn canonical_json_from_slice<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T> {
    let (value, offset) = parse_json_value(bytes, 0)?;
    if offset != bytes.len() {
        bail!("canonical JSON rejects trailing bytes");
    }
    let decoded: T = serde_json::from_value(value).context("decode canonical JSON value")?;
    if canonical_json_to_vec(&decoded)? != bytes {
        bail!("canonical JSON requires exact typed encoding");
    }
    Ok(decoded)
}

fn canonical_json_value_to_vec(value: &Value) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    encode_json_value(value, &mut output)?;
    Ok(output)
}

fn encode_json_value(value: &Value, output: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => {
            if number.is_f64() {
                bail!("canonical JSON rejects floating-point values");
            }
            output.extend_from_slice(number.to_string().as_bytes());
        }
        Value::String(string) => {
            output.extend_from_slice(
                serde_json::to_string(string)
                    .context("encode canonical JSON string")?
                    .as_bytes(),
            );
        }
        Value::Array(array) => {
            output.push(b'[');
            for (index, item) in array.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                encode_json_value(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(object) => {
            output.push(b'{');
            let mut first = true;
            let sorted: BTreeMap<_, _> = object.iter().collect();
            for (key, item) in sorted {
                if !first {
                    output.push(b',');
                }
                first = false;
                output.extend_from_slice(
                    serde_json::to_string(key)
                        .context("encode canonical JSON object key")?
                        .as_bytes(),
                );
                output.push(b':');
                encode_json_value(item, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn parse_json_value(input: &[u8], mut offset: usize) -> Result<(Value, usize)> {
    if offset >= input.len() {
        bail!("canonical JSON input is empty");
    }
    match input[offset] {
        b'n' => {
            expect_bytes(input, offset, b"null")?;
            Ok((Value::Null, offset + 4))
        }
        b't' => {
            expect_bytes(input, offset, b"true")?;
            Ok((Value::Bool(true), offset + 4))
        }
        b'f' => {
            expect_bytes(input, offset, b"false")?;
            Ok((Value::Bool(false), offset + 5))
        }
        b'"' => parse_json_string(input, offset).map(|(value, next)| (Value::String(value), next)),
        b'[' => {
            offset += 1;
            let mut items = Vec::new();
            if input.get(offset) == Some(&b']') {
                return Ok((Value::Array(items), offset + 1));
            }
            loop {
                let (item, next) = parse_json_value(input, offset)?;
                items.push(item);
                offset = next;
                match input.get(offset).copied() {
                    Some(b',') => offset += 1,
                    Some(b']') => return Ok((Value::Array(items), offset + 1)),
                    _ => bail!("canonical JSON array is malformed"),
                }
            }
        }
        b'{' => {
            offset += 1;
            let mut object = Map::new();
            if input.get(offset) == Some(&b'}') {
                return Ok((Value::Object(object), offset + 1));
            }
            loop {
                let (key, next) = parse_json_string(input, offset)?;
                offset = next;
                if input.get(offset) != Some(&b':') {
                    bail!("canonical JSON object key must be followed by ':'");
                }
                offset += 1;
                let (value, next) = parse_json_value(input, offset)?;
                if object.insert(key.clone(), value).is_some() {
                    bail!("canonical JSON rejects duplicate object keys");
                }
                offset = next;
                match input.get(offset).copied() {
                    Some(b',') => offset += 1,
                    Some(b'}') => return Ok((Value::Object(object), offset + 1)),
                    _ => bail!("canonical JSON object is malformed"),
                }
            }
        }
        b'-' | b'0'..=b'9' => {
            parse_json_number(input, offset).map(|(value, next)| (Value::Number(value), next))
        }
        _ => bail!("canonical JSON contains an unexpected byte"),
    }
}

fn parse_json_string(input: &[u8], offset: usize) -> Result<(String, usize)> {
    let mut index = offset + 1;
    let mut escaped = false;
    while let Some(&byte) = input.get(index) {
        if escaped {
            escaped = false;
            index += 1;
            continue;
        }
        match byte {
            b'\\' => {
                escaped = true;
                index += 1;
            }
            b'"' => {
                let raw = std::str::from_utf8(&input[offset..=index])
                    .context("canonical JSON string is not valid UTF-8")?;
                let decoded: String =
                    serde_json::from_str(raw).context("canonical JSON string is invalid")?;
                return Ok((decoded, index + 1));
            }
            0x00..=0x1f => bail!("canonical JSON rejects unescaped control characters"),
            _ => index += 1,
        }
    }
    bail!("canonical JSON string is unterminated")
}

fn parse_json_number(input: &[u8], offset: usize) -> Result<(serde_json::Number, usize)> {
    let mut index = offset;
    if input[index] == b'-' {
        index += 1;
        if index >= input.len() {
            bail!("canonical JSON number is truncated");
        }
    }
    match input[index] {
        b'0' => {
            index += 1;
            if matches!(input.get(index), Some(b'0'..=b'9')) {
                bail!("canonical JSON rejects leading zeroes");
            }
        }
        b'1'..=b'9' => {
            index += 1;
            while matches!(input.get(index), Some(b'0'..=b'9')) {
                index += 1;
            }
        }
        _ => bail!("canonical JSON number is invalid"),
    }
    if matches!(input.get(index), Some(b'.' | b'e' | b'E' | b'+')) {
        bail!("canonical JSON rejects floating-point values");
    }
    let slice =
        std::str::from_utf8(&input[offset..index]).context("canonical JSON number is not UTF-8")?;
    let number = if slice.starts_with('-') {
        let value = slice
            .parse::<i64>()
            .context("canonical JSON integer is out of range")?;
        serde_json::Number::from(value)
    } else {
        let value = slice
            .parse::<u64>()
            .context("canonical JSON integer is out of range")?;
        serde_json::Number::from(value)
    };
    Ok((number, index))
}

fn expect_bytes(input: &[u8], offset: usize, expected: &[u8]) -> Result<()> {
    if input.get(offset..offset + expected.len()) == Some(expected) {
        Ok(())
    } else {
        bail!("canonical JSON token is malformed")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PlannedActionReceiptV1 {
    receipt_id: String,
    entry_id: String,
    action: ManagedActionV1,
    attempt_id: String,
    receipt_relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedManagedRole {
    UnixPrefixPayloadVersion(String),
    UnixPrefixBinEntry(String),
    UnixPrefixShimTree,
    UnixPrefixLinuxGuestCache(String),
    UnixPrefixDevEnv,
    UnixPrefixProjection(String),
    UnixPrefixRuntimeScript(String),
    UnixPrefixRunDirectory,
    UnixPrincipalProfileSnippet(String),
    WindowsPrefixPayloadVersion(String),
    WindowsPrefixBinEntry(String),
    WindowsPrefixShimTree,
    WindowsPrefixProfileHelper(String),
    WindowsPrefixProjection(String),
    WindowsPrefixForwarderLogDirectory,
    WindowsPrincipalProfileSnippet(String),
    LinuxHostBinary(String),
    LinuxHostAclHelper,
    LinuxHostUnit(String),
    LinuxHostServiceState(String),
    LinuxHostDirectory(String),
    LinuxHostSocket,
    LinuxHostGroup,
    LinuxHostMembership(String),
    LinuxHostLinger(String),
    LinuxHostAclBridge(String, String),
    LinuxHostSyntheticAuth,
    LinuxPublisherExecutor,
    LinuxPublisherExecutorParent,
    LinuxPublisherLifecycleContainer,
    LinuxPublisherStateDirectory,
    LinuxPublisherServiceUnit,
    LinuxPublisherSocketUnit,
    LinuxPublisherEndpoint,
    LinuxPublisherSigningKey,
    LinuxPublisherCurrentAnchor,
    LinuxPublisherBootstrapIntent,
    LinuxPublisherServiceState(String),
    MacPublisherExecutor,
    MacPublisherPlist,
    MacPublisherSigningKey,
    MacPublisherCurrentAnchor,
    MacPublisherBootstrapIntent,
    MacPublisherGuestPairingRecord(String),
    MacPublisherMachService,
    MacPublisherServiceState,
    MacLimaInstance,
    MacLimaStagedWorkspace,
    MacLimaGuestBinary(String),
    MacLimaGuestUnit(String),
    MacLimaGuestServiceState(String),
    MacLimaGuestDirectory(String),
    MacLimaGuestGroup,
    MacLimaGuestMembership(String),
    MacLimaGuestPrivateHome,
    MacLimaLayoutSentinel,
    MacLimaPublisherExecutor,
    MacLimaPublisherStateDirectory,
    MacLimaPublisherServiceUnit,
    MacLimaPublisherSocketUnit,
    MacLimaPublisherEndpoint,
    MacLimaPublisherSigningKey,
    MacLimaPublisherCurrentAnchor,
    MacLimaPublisherBootstrapIntent,
    MacLimaPublisherServiceState(String),
    MacLimaGuestSocket,
    MacHostKnownHostsEntry,
    MacAttemptRenderedUnitsTree(String),
    MacHostForwardSocket,
    MacHostSshForwarder,
    WindowsPublisherExecutable,
    WindowsPublisherProductDirectory,
    WindowsPublisherInstallDirectory,
    WindowsPublisherServiceRegistration,
    WindowsPublisherSigningKey,
    WindowsPublisherCurrentAnchor,
    WindowsPublisherBootstrapIntent,
    WindowsPublisherRegistryContainer(String),
    WindowsPublisherGuestPairingRecord(String),
    WindowsPublisherEndpoint,
    WindowsPublisherServiceState,
    WindowsHostForwarderProcess,
    WindowsHostForwarderPidRecord,
    WindowsHostForwarderPipe,
    WindowsWslInstance,
    WindowsWslGuestBinary(String),
    WindowsWslGuestUnit(String),
    WindowsWslGuestServiceState(String),
    WindowsWslGuestDirectory(String),
    WindowsWslGuestGroup,
    WindowsWslGuestMembership(String),
    WindowsWslGuestSocket,
    WindowsWslPublisherExecutor,
    WindowsWslPublisherStateDirectory,
    WindowsWslPublisherServiceUnit,
    WindowsWslPublisherSocketUnit,
    WindowsWslPublisherEndpoint,
    WindowsWslPublisherSigningKey,
    WindowsWslPublisherCurrentAnchor,
    WindowsWslPublisherBootstrapIntent,
    WindowsWslPublisherServiceState(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedComponentRole {
    LinuxStateRoot,
    LinuxLifecycleContainer,
    LinuxPublisherDirectory,
    LinuxExecutorParent,
    LinuxBootstrapIntent,
    LinuxExecutor,
    LinuxServiceUnit(String),
    LinuxEndpoint,
    LinuxSigningKey,
    LinuxProtectedState,
    LinuxServiceState(String),
    MacExecutor,
    MacPlist,
    MacMachService,
    MacSigningKey,
    MacProtectedState,
    MacBootstrapIntent,
    MacServiceState,
    MacGuestPairingRecord(String),
    WindowsProductDirectory,
    WindowsInstallDirectory,
    WindowsExecutable,
    WindowsServiceRegistration,
    WindowsSigningKey,
    WindowsProtectedState,
    WindowsBootstrapIntent,
    WindowsEndpoint,
    WindowsServiceState,
    WindowsRegistryContainer(String),
    WindowsGuestPairingRecord(String),
    GuestStateRoot(String),
    GuestLifecycleContainer(String),
    GuestPublisherDirectory(String),
    GuestExecutorParent(String),
    GuestBootstrapIntent(String, String),
    GuestExecutor(String),
    GuestServiceUnit(String, String),
    GuestEndpoint(String),
    GuestSigningKey(String),
    GuestProtectedState(String),
    GuestServiceState(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey as Ed25519SigningKey;
    use p256::ecdsa::SigningKey as P256SigningKey;
    use std::collections::BTreeMap;

    fn sample_executor_identity() -> ManagedExecutorIdentityV1 {
        ManagedExecutorIdentityV1 {
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            artifact_sha256: "c".repeat(64),
            artifact_path: "/tmp/substrate-lifecycle-linux".to_string(),
            toolchain: Some("rustc 1.89.0".to_string()),
            code_identity: None,
        }
    }

    fn sample_signature() -> LifecycleSignatureV1 {
        LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: URL_SAFE_NO_PAD.encode([7_u8; 32]),
            signature: URL_SAFE_NO_PAD.encode([8_u8; 64]),
        }
    }

    fn sign_p256_record_v1<T: Serialize>(
        signing_key: &P256SigningKey,
        owner: &str,
        record: &T,
    ) -> String {
        let payload = canonical_lifecycle_signature_payload_v1(owner, record).unwrap();
        let signature: P256Signature = signing_key.sign(&payload);
        let signature = signature.normalize_s().unwrap_or(signature);
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    }

    fn mac_pairing_signing_key_v1() -> P256SigningKey {
        P256SigningKey::from_bytes((&[8_u8; 32]).into()).unwrap()
    }

    fn mac_pairing_spki_der_v1(signing_key: &P256SigningKey) -> Vec<u8> {
        let public_key = P256PublicKeyDocument::from_sec1_bytes(
            signing_key
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes(),
        )
        .unwrap();
        public_key.to_public_key_der().unwrap().as_ref().to_vec()
    }

    fn valid_mac_pairing_ticket_v1() -> (P256SigningKey, GuestPublisherPairingTicketV1) {
        let signing_key = mac_pairing_signing_key_v1();
        let spki_der = mac_pairing_spki_der_v1(&signing_key);
        let signer_spki_der = URL_SAFE_NO_PAD.encode(&spki_der);
        let mut anchor = LifecyclePublisherAnchorV1 {
            schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
            schema_version: 1,
            authority_domain: "mac_host_shared".to_string(),
            host_context_commitment: "1".repeat(64),
            platform_mapping_commitment: Some("2".repeat(64)),
            scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
            manifest_generation: 1,
            manifest_sha256: "3".repeat(64),
            action_receipt_index_revision: 1,
            action_receipt_index_sha256: "4".repeat(64),
            head_sha256: "5".repeat(64),
            previous_anchor_sha256: None,
            request_sha256: "6".repeat(64),
            requester_principal: "substrate-control".to_string(),
            attempt_nonce: "attempt-1".to_string(),
            executor_identity: ManagedExecutorIdentityV1 {
                source_commit: "a".repeat(40),
                source_tree: "b".repeat(40),
                source_ref: "refs/heads/test".to_string(),
                target_triple: "aarch64-apple-darwin".to_string(),
                artifact_sha256: "c".repeat(64),
                artifact_path: "/Library/PrivilegedHelperTools/substrate-lifecycle-macos"
                    .to_string(),
                toolchain: None,
                code_identity: None,
            },
            signature: LifecycleSignatureV1 {
                algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
                public_key: signer_spki_der.clone(),
                signature: String::new(),
            },
        };
        anchor.signature.signature =
            sign_p256_record_v1(&signing_key, &anchor.schema_owner, &anchor);
        let anchor_sha256 = lifecycle_anchor_sha256_v1(&anchor).unwrap();
        let challenge = GuestPublisherPairingChallengeV1 {
            schema_owner: "substrate.guest-publisher-pairing-challenge".to_string(),
            schema_version: 1,
            challenge_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string(),
            challenge: URL_SAFE_NO_PAD.encode([0x11_u8; 32]),
            expires_at_unix_ns: 9_000_000_000_000_000_000,
            host_key_fingerprint_sha256: lower_hex(&Sha256::digest(&spki_der)),
            current_anchor_sha256: anchor_sha256.clone(),
            host_context_commitment: anchor.host_context_commitment.clone(),
            platform_mapping_commitment: anchor.platform_mapping_commitment.clone(),
            guest_machine_identity: "guest-machine-1".to_string(),
            source_commit: anchor.executor_identity.source_commit.clone(),
            source_tree: anchor.executor_identity.source_tree.clone(),
            source_ref: anchor.executor_identity.source_ref.clone(),
            executor_build_evidence_sha256: anchor.executor_identity.artifact_sha256.clone(),
            guest_component_commitment_sha256: "d".repeat(64),
        };
        let challenge_sha256 =
            lower_hex(&Sha256::digest(canonical_json_to_vec(&challenge).unwrap()));
        let mut ticket = GuestPublisherPairingTicketV1 {
            schema_owner: "substrate.guest-publisher-pairing-ticket".to_string(),
            schema_version: 1,
            challenge,
            signer_spki_der: signer_spki_der.clone(),
            current_anchor: anchor,
            current_anchor_sha256: anchor_sha256,
            challenge_sha256,
            host_generation: 1,
            host_counter: 1,
            guest_test_retirement_commitment: None,
            signature: LifecycleSignatureV1 {
                algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
                public_key: signer_spki_der,
                signature: String::new(),
            },
        };
        ticket.signature.signature =
            sign_p256_record_v1(&signing_key, &ticket.schema_owner, &ticket);
        (signing_key, ticket)
    }

    fn signed_mac_pairing_host_record_v1(
        signing_key: &P256SigningKey,
        ticket: GuestPublisherPairingTicketV1,
        record_generation: u64,
        state: &str,
        previous_record_sha256: Option<String>,
    ) -> GuestPublisherPairingHostRecordV1 {
        let binding = GuestPublisherPairingSessionBindingV1 {
            schema_owner: GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_OWNER_V1.to_string(),
            schema_version: GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_VERSION_V1,
            scope_id: ticket.current_anchor.scope_id.clone(),
            platform_mapping_commitment: ticket.challenge.platform_mapping_commitment.clone(),
            guest_machine_identity: ticket.challenge.guest_machine_identity.clone(),
            source_commit: ticket.challenge.source_commit.clone(),
            source_tree: ticket.challenge.source_tree.clone(),
            source_ref: ticket.challenge.source_ref.clone(),
            staged_executor_sha256: ticket.challenge.executor_build_evidence_sha256.clone(),
            stage_one_record_sha256: "d".repeat(64),
            ticket_challenge_id: ticket.challenge.challenge_id.clone(),
            host_record_generation: ticket.host_generation,
            pairing_session_nonce: "e".repeat(64),
        };
        let (data_state, operator_state, operator_proof_sha256, operator_ended_observation) =
            match state {
                "sessions_opened" => ("opened", "opened", None, None),
                "operator_proof_verified" => (
                    "opened",
                    "closed",
                    Some("f".repeat(64)),
                    Some("operator-child-exited-successfully".to_string()),
                ),
                _ => ("opened", "opened", None, None),
            };
        let mut record = GuestPublisherPairingHostRecordV1 {
            schema_owner: "substrate.guest-publisher-pairing-host-record".to_string(),
            schema_version: 1,
            current_anchor_counter: ticket.host_counter,
            current_anchor_sha256: ticket.current_anchor_sha256.clone(),
            data_session: GuestPublisherPairingDataSessionV1 {
                schema_owner: GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_OWNER_V1.to_string(),
                schema_version: GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_VERSION_V1,
                binding: binding.clone(),
                session_id: "1".repeat(64),
                ticket: ticket.clone(),
                state: data_state.to_string(),
                hello: None,
                transcript: None,
                guest_anchor_sha256: None,
                started_observation: "data-started".to_string(),
                ended_observation: None,
                failure_observation: None,
            },
            operator_tty_session: GuestPublisherPairingOperatorTtySessionV1 {
                schema_owner: GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_OWNER_V1
                    .to_string(),
                schema_version: GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_VERSION_V1,
                binding: binding.clone(),
                session_id: "2".repeat(64),
                controlling_tty_identity: "guest-tty-test".to_string(),
                state: operator_state.to_string(),
                started_observation: "operator-started".to_string(),
                ended_observation: operator_ended_observation,
                failure_observation: None,
            },
            ticket,
            binding,
            operator_launch_sha256: "a".repeat(64),
            operator_proof_sha256,
            record_generation,
            state: state.to_string(),
            previous_record_sha256,
            signature: LifecycleSignatureV1 {
                algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
                public_key: String::new(),
                signature: String::new(),
            },
        };
        record.signature.public_key = record.ticket.signer_spki_der.clone();
        record.signature.signature =
            sign_p256_record_v1(signing_key, &record.schema_owner, &record);
        record
    }

    fn signed_pairing_operator_launch_v1(
        signing_key: &P256SigningKey,
        record: &GuestPublisherPairingHostRecordV1,
    ) -> GuestPublisherPairingOperatorLaunchV1 {
        let mut launch = GuestPublisherPairingOperatorLaunchV1 {
            schema_owner: GUEST_PUBLISHER_PAIRING_OPERATOR_LAUNCH_SCHEMA_OWNER_V1.to_string(),
            schema_version: GUEST_PUBLISHER_PAIRING_OPERATOR_LAUNCH_SCHEMA_VERSION_V1,
            binding: record.binding.clone(),
            operator_session_id: record.operator_tty_session.session_id.clone(),
            admitted_instance_name: "substrate-r6-fixture".to_string(),
            expires_at_unix_ns: record.ticket.challenge.expires_at_unix_ns,
            limactl_absolute_path: "/opt/substrate/bin/limactl".to_string(),
            limactl_sha256: "1".repeat(64),
            guest_executable_path: GUEST_PUBLISHER_PAIRING_OPERATOR_GUEST_EXECUTABLE_PATH_V1
                .to_string(),
            guest_executable_sha256: record.binding.staged_executor_sha256.clone(),
            fixed_operator_command: GUEST_PUBLISHER_PAIRING_OPERATOR_FIXED_COMMAND_V1.to_string(),
            signature: String::new(),
        };
        let payload =
            canonical_guest_publisher_pairing_operator_launch_signature_payload_v1(&launch)
                .expect("canonical unsigned operator launch payload");
        let signature: P256Signature = signing_key.sign(&payload);
        let signature = signature.normalize_s().unwrap_or(signature);
        launch.signature = URL_SAFE_NO_PAD.encode(signature.to_bytes());
        launch
    }

    fn pairing_operator_proof_v1(
        record: &GuestPublisherPairingHostRecordV1,
        commitment: &str,
    ) -> GuestPublisherPairingOperatorProofV1 {
        GuestPublisherPairingOperatorProofV1 {
            schema_owner: GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_SCHEMA_OWNER_V1.to_string(),
            schema_version: GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_SCHEMA_VERSION_V1,
            binding: record.binding.clone(),
            operator_session_id: record.operator_tty_session.session_id.clone(),
            operator_launch_sha256: record.operator_launch_sha256.clone(),
            confirmation_commitment: commitment.to_string(),
            created_at_unix_ns: 1,
            expires_at_unix_ns: record.ticket.challenge.expires_at_unix_ns,
            terminal_observation: GUEST_PUBLISHER_PAIRING_OPERATOR_PROOF_TERMINAL_OBSERVATION_V1
                .to_string(),
        }
    }

    #[test]
    fn r6_operator_launch_is_canonical_and_ticket_key_bound() {
        let (signing_key, ticket) = valid_mac_pairing_ticket_v1();
        let mut opened =
            signed_mac_pairing_host_record_v1(&signing_key, ticket, 1, "sessions_opened", None);
        let launch = signed_pairing_operator_launch_v1(&signing_key, &opened);
        opened.operator_launch_sha256 =
            guest_publisher_pairing_operator_launch_sha256_v1(&launch).expect("launch digest");
        opened.signature.signature =
            sign_p256_record_v1(&signing_key, &opened.schema_owner, &opened);

        let canonical = canonical_guest_publisher_pairing_operator_launch_v1(&launch)
            .expect("canonical operator launch");
        assert_eq!(
            parse_and_validate_guest_publisher_pairing_operator_launch_v1(&canonical)
                .expect("parse canonical operator launch"),
            launch
        );
        validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1(
            &opened.ticket,
            &launch,
            1,
        )
        .expect("ticket signer authorizes operator launch");
        validate_guest_publisher_pairing_operator_launch_against_ticket_and_host_record_at_v1(
            &opened.ticket,
            &opened,
            &launch,
            1,
        )
        .expect("protected record binds the admitted operator session");

        let mut alternate_signed_launch = launch.clone();
        alternate_signed_launch.admitted_instance_name = "substrate-r6-alternate".to_string();
        let payload = canonical_guest_publisher_pairing_operator_launch_signature_payload_v1(
            &alternate_signed_launch,
        )
        .expect("alternate launch payload");
        let signature: P256Signature = signing_key.sign(&payload);
        let signature = signature.normalize_s().unwrap_or(signature);
        alternate_signed_launch.signature = URL_SAFE_NO_PAD.encode(signature.to_bytes());
        validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1(
            &opened.ticket,
            &alternate_signed_launch,
            1,
        )
        .expect("alternate launch remains ticket-valid");
        assert!(
            validate_guest_publisher_pairing_operator_launch_against_ticket_and_host_record_at_v1(
                &opened.ticket,
                &opened,
                &alternate_signed_launch,
                1,
            )
            .is_err()
        );

        let mut forged = launch.clone();
        forged.operator_session_id = "0".repeat(64);
        assert!(
            validate_guest_publisher_pairing_operator_launch_against_ticket_and_host_record_at_v1(
                &opened.ticket,
                &opened,
                &forged,
                1,
            )
            .is_err()
        );

        let mut expired = launch;
        expired.expires_at_unix_ns = opened.ticket.challenge.expires_at_unix_ns - 1;
        expired.signature = sign_p256_record_v1(&signing_key, &expired.schema_owner, &expired);
        assert!(
            validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1(
                &opened.ticket,
                &expired,
                1,
            )
            .is_err()
        );
    }

    #[test]
    fn r6_operator_proof_is_canonical_and_cas_bound_without_replay() {
        let (signing_key, ticket) = valid_mac_pairing_ticket_v1();
        let mut opened =
            signed_mac_pairing_host_record_v1(&signing_key, ticket, 1, "sessions_opened", None);
        let launch = signed_pairing_operator_launch_v1(&signing_key, &opened);
        opened.operator_launch_sha256 =
            guest_publisher_pairing_operator_launch_sha256_v1(&launch).expect("launch digest");
        opened.signature.signature =
            sign_p256_record_v1(&signing_key, &opened.schema_owner, &opened);
        let proof = pairing_operator_proof_v1(&opened, &"f".repeat(64));
        let canonical = canonical_guest_publisher_pairing_operator_proof_v1(&proof)
            .expect("canonical operator proof");
        assert_eq!(
            parse_and_validate_guest_publisher_pairing_operator_proof_v1(&canonical)
                .expect("parse canonical operator proof"),
            proof
        );
        validate_guest_publisher_pairing_operator_proof_against_ticket_at_v1(
            &opened.ticket,
            &opened,
            &proof,
            &"f".repeat(64),
            1,
        )
        .expect("proof joins the opened ticket and binding before expiry");

        let mut wrong_launch = proof.clone();
        wrong_launch.operator_launch_sha256 = "e".repeat(64);
        validate_guest_publisher_pairing_operator_proof_v1(&wrong_launch)
            .expect("wrong-launch proof remains structurally valid");
        assert!(
            validate_guest_publisher_pairing_operator_proof_against_ticket_at_v1(
                &opened.ticket,
                &opened,
                &wrong_launch,
                &"f".repeat(64),
                1,
            )
            .is_err()
        );

        let proof_sha256 =
            guest_publisher_pairing_operator_proof_sha256_v1(&proof).expect("proof digest");
        let mut verified = opened.clone();
        verified.record_generation = opened.record_generation + 1;
        verified.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&opened).expect("opened digest"));
        verified.state = "operator_proof_verified".to_string();
        verified.operator_tty_session.state = "closed".to_string();
        verified.operator_tty_session.ended_observation =
            Some("operator-child-exited-successfully".to_string());
        verified.operator_proof_sha256 = Some(proof_sha256.clone());
        verified.signature.signature =
            sign_p256_record_v1(&signing_key, &verified.schema_owner, &verified);
        compare_and_swap_guest_publisher_pairing_host_record_at_v1(&opened, &verified, 1)
            .expect("proof digest advances only the current generation");
        assert_eq!(verified.binding.host_record_generation, 1);
        assert_eq!(verified.record_generation, 2);

        let mut mutated_launch_binding = verified.clone();
        mutated_launch_binding.record_generation += 1;
        mutated_launch_binding.previous_record_sha256 = Some(
            guest_publisher_pairing_host_record_sha256_v1(&verified)
                .expect("verified record digest"),
        );
        mutated_launch_binding.state = "guest_state_root_durable".to_string();
        mutated_launch_binding.data_session.state = "guest_state_root_durable".to_string();
        mutated_launch_binding.operator_launch_sha256 = "e".repeat(64);
        mutated_launch_binding.signature.signature = sign_p256_record_v1(
            &signing_key,
            &mutated_launch_binding.schema_owner,
            &mutated_launch_binding,
        );
        validate_guest_publisher_pairing_host_record_v1(&mutated_launch_binding)
            .expect("mutated launch binding remains structurally valid");
        assert!(compare_and_swap_guest_publisher_pairing_host_record_at_v1(
            &verified,
            &mutated_launch_binding,
            1,
        )
        .is_err());

        validate_guest_publisher_pairing_operator_proof_against_ticket_at_v1(
            &verified.ticket,
            &verified,
            &proof,
            &"f".repeat(64),
            1,
        )
        .expect("byte-identical proof is the only replay admitted after CAS");

        let mut alternate = proof;
        alternate.confirmation_commitment = "e".repeat(64);
        assert!(
            validate_guest_publisher_pairing_operator_proof_against_ticket_at_v1(
                &verified.ticket,
                &verified,
                &alternate,
                &"e".repeat(64),
                1,
            )
            .is_err()
        );
        assert!(
            validate_guest_publisher_pairing_operator_proof_against_ticket_at_v1(
                &verified.ticket,
                &verified,
                &parse_and_validate_guest_publisher_pairing_operator_proof_v1(&canonical)
                    .expect("canonical proof remains parseable"),
                &"f".repeat(64),
                verified.ticket.challenge.expires_at_unix_ns,
            )
            .is_err()
        );
    }

    fn sample_manifest() -> ManagedArtifactManifestV1 {
        ManagedArtifactManifestV1 {
            schema_owner: MANIFEST_SCHEMA_OWNER_V1.to_string(),
            schema_version: MANIFEST_SCHEMA_VERSION_V1,
            manifest_id: "m1:018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90:7".to_string(),
            manifest_sha256: "971e968f3a35a201b04fd55650b43b74c81fd1f7aba0bfca4a2eee4fcf8ad55f"
                .to_string(),
            host_context_commitment: "1".repeat(64),
            selected_host_prefix: "/opt/substrate".to_string(),
            intended_principal: "alice:1000".to_string(),
            platform_kind: "unix".to_string(),
            platform_mapping_commitment: None,
            authority_domain: "unix_a_local".to_string(),
            installation_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
            attempt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string(),
            manifest_generation: 7,
            created_at_unix_ns: 1,
            lifecycle_state: ManagedLifecycleStateV1::ManifestDurable,
            previous_manifest_sha256: None,
            entries: Vec::new(),
            planned_action_receipts: Vec::new(),
        }
    }

    fn sample_manifest_with_prefix_entry() -> ManagedArtifactManifestV1 {
        let mut manifest = sample_manifest();
        manifest.manifest_sha256.clear();
        manifest.entries = vec![ManagedArtifactEntryV1 {
            object_id: "entry-1".to_string(),
            logical_role: ManagedArtifactRoleV1("unix.prefix.projection(config.yaml)".to_string()),
            object_type: "regular-file".to_string(),
            identity: ManagedArtifactIdentityV1 {
                scope_id: manifest.installation_id.clone(),
                parent_identity: manifest.selected_host_prefix.clone(),
                name_identity: "config.yaml".to_string(),
                physical_identity: format!("{}/config.yaml", manifest.selected_host_prefix),
                metadata: None,
            },
            disposition: ManagedArtifactDispositionV1::Created,
            bytes_or_target: Some("f".repeat(64)),
            owner: Some("alice".to_string()),
            group_name: None,
            mode: Some("0600".to_string()),
            acl_or_security: None,
            service_or_platform_state: None,
            before_state: Value::Null,
            intended_after_state: Value::Null,
            restoration_state: Value::Null,
            dependency_object_ids: Vec::new(),
            subtree_members: Vec::new(),
            lifecycle_state: ManagedLifecycleStateV1::ManifestDurable,
            last_durable_transition: Some("ManifestDurable".to_string()),
            error_class: None,
        }];
        manifest.planned_action_receipts = vec![serde_json::json!({
            "receipt_id": "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92",
            "entry_id": "entry-1",
            "action": "create",
            "attempt_id": "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a93",
            "receipt_relative_path": "receipts/7/receipt.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92.json"
        })];
        manifest.manifest_sha256 = manifest_digest_v1(&manifest).unwrap();
        manifest
    }

    #[test]
    fn manifest_digest_matches_contract_golden_vector() {
        let value = serde_json::json!({
            "installation_id": "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90",
            "manifest_generation": 7,
            "manifest_id": "m1:018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90:7",
            "schema_owner": "substrate.managed-artifact-manifest",
            "schema_version": 1
        });
        let bytes = canonical_json_value_to_vec(&value).expect("canonical bytes");
        let digest = lower_hex(&Sha256::digest(bytes));
        assert_eq!(
            digest,
            "971e968f3a35a201b04fd55650b43b74c81fd1f7aba0bfca4a2eee4fcf8ad55f"
        );
    }

    #[test]
    fn parse_and_validate_manifest_rejects_duplicate_keys() {
        let bytes = br#"{"attempt_id":"018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91","attempt_id":"018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91"}"#;
        assert!(parse_and_validate_manifest_v1(bytes).is_err());
    }

    #[test]
    fn parse_and_validate_manifest_rejects_unknown_field() {
        let manifest = sample_manifest();
        let mut value = serde_json::to_value(manifest).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_string(), Value::Bool(true));
        let bytes = canonical_json_value_to_vec(&value).unwrap();
        assert!(parse_and_validate_manifest_v1(&bytes).is_err());
    }

    #[test]
    fn parse_and_validate_manifest_rejects_wrong_type() {
        let manifest = sample_manifest();
        let mut value = serde_json::to_value(manifest).unwrap();
        value.as_object_mut().unwrap().insert(
            "manifest_generation".to_string(),
            Value::String("7".to_string()),
        );
        let bytes = canonical_json_value_to_vec(&value).unwrap();
        assert!(parse_and_validate_manifest_v1(&bytes).is_err());
    }

    #[test]
    fn parse_and_validate_manifest_rejects_trailing_bytes() {
        let bytes = canonical_json_to_vec(&sample_manifest()).unwrap();
        let mut extra = bytes.clone();
        extra.extend_from_slice(b"\n");
        assert!(parse_and_validate_manifest_v1(&extra).is_err());
    }

    #[test]
    fn parse_and_validate_manifest_rejects_cross_prefix_entry_identity() {
        let mut manifest = sample_manifest_with_prefix_entry();
        manifest.entries[0].identity.physical_identity = "/tmp/other/config.yaml".to_string();
        manifest.manifest_sha256 = manifest_digest_v1(&manifest).unwrap();
        let bytes = canonical_json_to_vec(&manifest).unwrap();
        assert!(parse_and_validate_manifest_v1(&bytes).is_err());
    }

    #[test]
    fn parse_and_validate_manifest_rejects_disallowed_action_for_role() {
        let mut manifest = sample_manifest_with_prefix_entry();
        manifest.planned_action_receipts[0]
            .as_object_mut()
            .unwrap()
            .insert("action".to_string(), Value::String("start".to_string()));
        manifest.manifest_sha256 = manifest_digest_v1(&manifest).unwrap();
        let bytes = canonical_json_to_vec(&manifest).unwrap();
        assert!(parse_and_validate_manifest_v1(&bytes).is_err());
    }

    #[test]
    fn managed_role_accepts_closed_variants_and_rejects_escape() {
        assert!(parse_managed_role("linux.host.synthetic-auth").is_ok());
        assert!(parse_managed_role("mac.lima.publisher-service-state(socket)").is_ok());
        assert!(parse_managed_role("windows.prefix.projection(../../etc/passwd)").is_err());
        assert!(parse_managed_role("unix.prefix.payload-version(v1/../x)").is_err());
    }

    #[test]
    fn publisher_request_validator_rejects_disallowed_role_action_pair() {
        let request = ManagedLifecyclePublisherRequestV1 {
            host_context_commitment: "1".repeat(64),
            platform_mapping_commitment: None,
            scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
            current_anchor_counter: 0,
            current_anchor_sha256: "2".repeat(64),
            manifest_generation: 7,
            manifest_sha256: "3".repeat(64),
            role: ManagedArtifactRoleV1("linux.host.socket".to_string()),
            action: ManagedActionV1::Create,
            object_identity: ManagedArtifactIdentityV1 {
                scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
                parent_identity: "/run".to_string(),
                name_identity: "substrate.sock".to_string(),
                physical_identity: "/run/substrate.sock".to_string(),
                metadata: None,
            },
            requester_principal: "alice:1000".to_string(),
            attempt_nonce: "nonce-1".to_string(),
            expected_executor_build: sample_executor_identity(),
        };
        assert!(validate_managed_lifecycle_publisher_request_v1(&request).is_err());
    }

    #[test]
    fn receipt_index_compare_and_swap_is_append_only() {
        let current = ManagedActionReceiptIndexV1 {
            schema_owner: ACTION_RECEIPT_INDEX_OWNER_V1.to_string(),
            schema_version: 1,
            authority_domain: "unix_a_local".to_string(),
            scope_id: "scope-1".to_string(),
            manifest_generation: 7,
            manifest_sha256: "d".repeat(64),
            index_revision: 0,
            previous_index_sha256: None,
            entries: Vec::new(),
        };
        let next = ManagedActionReceiptIndexV1 {
            schema_owner: ACTION_RECEIPT_INDEX_OWNER_V1.to_string(),
            schema_version: 1,
            authority_domain: "unix_a_local".to_string(),
            scope_id: "scope-1".to_string(),
            manifest_generation: 7,
            manifest_sha256: "d".repeat(64),
            index_revision: 1,
            previous_index_sha256: Some(lower_hex(&Sha256::digest(
                canonical_action_receipt_index_bytes_v1(&current).unwrap(),
            ))),
            entries: vec![ManagedActionReceiptIndexEntryV1 {
                receipt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92".to_string(),
                entry_id: "entry-1".to_string(),
                action: ManagedActionV1::Create,
                attempt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a93".to_string(),
                receipt_relative_path:
                    "receipts/7/receipt.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92.json".to_string(),
                canonical_byte_length: 17,
                receipt_artifact_sha256: "e".repeat(64),
                retained_receipt_file_identity: "file-1".to_string(),
                retained_receipt_parent_identity: "parent-1".to_string(),
                prepared_record_sha256: "f".repeat(64),
                allocated_counter: 1,
                durable_observation: Value::Object(Map::new()),
            }],
        };
        compare_and_swap_action_receipt_index_v1(&current, &next).expect("append-only index");
    }

    #[test]
    fn lifecycle_signature_verification_supports_ed25519_and_rejects_forgery() {
        #[derive(Clone, Serialize)]
        struct SignedRecord<'a> {
            schema_owner: &'a str,
            schema_version: u32,
            value: &'a str,
            signature: LifecycleSignatureV1,
        }

        let signing_key = Ed25519SigningKey::from_bytes(&[9_u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let mut record = SignedRecord {
            schema_owner: "substrate.test-record",
            schema_version: 1,
            value: "hello",
            signature: LifecycleSignatureV1 {
                algorithm: "ed25519-v1".to_string(),
                public_key: URL_SAFE_NO_PAD.encode(verifying_key.to_bytes()),
                signature: String::new(),
            },
        };
        let payload = canonical_lifecycle_signature_payload_v1(record.schema_owner, &record)
            .expect("payload");
        let signature = signing_key.sign(&payload);
        record.signature.signature = URL_SAFE_NO_PAD.encode(signature.to_bytes());
        verify_lifecycle_signature_v1(record.schema_owner, &record, &record.signature)
            .expect("ed25519 verify");
        record.value = "forged";
        assert!(
            verify_lifecycle_signature_v1(record.schema_owner, &record, &record.signature).is_err()
        );
    }

    #[test]
    fn lifecycle_signature_verification_supports_p256_low_s() {
        #[derive(Clone, Serialize)]
        struct SignedRecord<'a> {
            schema_owner: &'a str,
            schema_version: u32,
            value: &'a str,
            signature: LifecycleSignatureV1,
        }

        let signing_key = P256SigningKey::from_bytes((&[5_u8; 32]).into()).expect("signing key");
        let verifying_key = signing_key.verifying_key();
        let public_key = P256PublicKeyDocument::from_sec1_bytes(
            verifying_key.to_encoded_point(false).as_bytes(),
        )
        .unwrap();
        let der = public_key.to_public_key_der().unwrap();
        let mut record = SignedRecord {
            schema_owner: "substrate.test-record",
            schema_version: 1,
            value: "hello",
            signature: LifecycleSignatureV1 {
                algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
                public_key: URL_SAFE_NO_PAD.encode(der.as_ref()),
                signature: String::new(),
            },
        };
        let payload = canonical_lifecycle_signature_payload_v1(record.schema_owner, &record)
            .expect("payload");
        let signature: P256Signature = signing_key.sign(&payload);
        assert!(
            signature.normalize_s().is_none(),
            "fixture must already be low-S"
        );
        record.signature.signature = URL_SAFE_NO_PAD.encode(signature.to_bytes());
        verify_lifecycle_signature_v1(record.schema_owner, &record, &record.signature)
            .expect("p256 verify");
    }

    #[test]
    fn pairing_ticket_validation_rejects_spki_anchor_mismatch() {
        let signing_key = P256SigningKey::from_bytes((&[6_u8; 32]).into()).unwrap();
        let verifying_key = signing_key.verifying_key();
        let public_key = P256PublicKeyDocument::from_sec1_bytes(
            verifying_key.to_encoded_point(false).as_bytes(),
        )
        .unwrap();
        let der = public_key.to_public_key_der().unwrap();
        let anchor = LifecyclePublisherAnchorV1 {
            schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
            schema_version: 1,
            authority_domain: "mac_host_shared".to_string(),
            host_context_commitment: "1".repeat(64),
            platform_mapping_commitment: Some("2".repeat(64)),
            scope_id: "scope-1".to_string(),
            manifest_generation: 1,
            manifest_sha256: "3".repeat(64),
            action_receipt_index_revision: 0,
            action_receipt_index_sha256: "4".repeat(64),
            head_sha256: "5".repeat(64),
            previous_anchor_sha256: None,
            request_sha256: "6".repeat(64),
            requester_principal: "alice".to_string(),
            attempt_nonce: "nonce-1".to_string(),
            executor_identity: sample_executor_identity(),
            signature: sample_signature(),
        };
        let challenge = GuestPublisherPairingChallengeV1 {
            schema_owner: "substrate.guest-publisher-pairing-challenge".to_string(),
            schema_version: 1,
            challenge_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99".to_string(),
            challenge: URL_SAFE_NO_PAD.encode([0x11_u8; 32]),
            expires_at_unix_ns: 42,
            host_key_fingerprint_sha256: "7".repeat(64),
            current_anchor_sha256: "8".repeat(64),
            host_context_commitment: "9".repeat(64),
            platform_mapping_commitment: Some("a".repeat(64)),
            guest_machine_identity: "machine-1".to_string(),
            source_commit: "b".repeat(40),
            source_tree: "c".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            executor_build_evidence_sha256: "d".repeat(64),
            guest_component_commitment_sha256: "e".repeat(64),
        };
        let mut ticket = GuestPublisherPairingTicketV1 {
            schema_owner: "substrate.guest-publisher-pairing-ticket".to_string(),
            schema_version: 1,
            challenge,
            signer_spki_der: URL_SAFE_NO_PAD.encode(der.as_ref()),
            current_anchor: anchor,
            current_anchor_sha256: "f".repeat(64),
            challenge_sha256: "0".repeat(64),
            host_generation: 1,
            host_counter: 1,
            guest_test_retirement_commitment: None,
            signature: LifecycleSignatureV1 {
                algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
                public_key: URL_SAFE_NO_PAD.encode(der.as_ref()),
                signature: String::new(),
            },
        };
        let payload =
            canonical_lifecycle_signature_payload_v1(&ticket.schema_owner, &ticket).unwrap();
        let signature: P256Signature = signing_key.sign(&payload);
        ticket.signature.signature = URL_SAFE_NO_PAD.encode(signature.to_bytes());
        assert!(validate_guest_publisher_pairing_ticket_v1(&ticket).is_err());
    }

    #[test]
    fn mac_control_authority_accepts_only_two_canonical_requirement_forms() {
        let cdhash = "0123456789abcdef0123456789abcdef01234567";
        let mut authority = MacPublisherControlAuthorityV1 {
            schema_owner: MAC_PUBLISHER_CONTROL_AUTHORITY_OWNER_V1.to_string(),
            schema_version: 1,
            control_binary: "substrate-lifecycle-control".to_string(),
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
            artifact_sha256: "c".repeat(64),
            designated_requirement: format!(
                "anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\" and cdhash H\"{cdhash}\""
            ),
        };
        let canonical = canonical_mac_publisher_control_authority_v1(&authority)
            .expect("canonical fixed control authority");
        assert_eq!(
            mac_publisher_control_authority_sha256_v1(&authority).unwrap(),
            lower_hex(&Sha256::digest(canonical))
        );
        assert_eq!(
            mac_publisher_control_requirement_cdhash_v1(&authority.designated_requirement)
                .expect("production requirement CDHash"),
            cdhash
        );

        authority.designated_requirement = format!("cdhash H\"{cdhash}\"");
        validate_mac_publisher_control_authority_v1(&authority)
            .expect("ad-hoc CDHash requirement remains canonical");
        assert_eq!(
            mac_publisher_control_requirement_cdhash_v1(&authority.designated_requirement)
                .expect("ad-hoc requirement CDHash"),
            cdhash
        );

        for invalid_requirement in [
            "identifier \"attacker\"".to_string(),
            format!("cdhash H\"{}\"", cdhash.to_ascii_uppercase()),
            format!("cdhash H\"{cdhash}\" and identifier \"attacker\""),
            format!("anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\" and cdhash H\"{cdhash}\" and certificate leaf[subject.CN] = \"attacker\""),
            "cdhash H\"not-a-cdhash\"".to_string(),
        ] {
            authority.designated_requirement = invalid_requirement;
            assert!(validate_mac_publisher_control_authority_v1(&authority).is_err());
        }
    }

    #[test]
    fn mac_pairing_ticket_rejects_malformed_spki_high_s_expiry_and_generation_mismatch() {
        let (signing_key, ticket) = valid_mac_pairing_ticket_v1();
        validate_guest_publisher_pairing_ticket_v1(&ticket).expect("valid P-256 ticket");
        validate_guest_publisher_pairing_ticket_at_v1(&ticket, 41).expect("ticket unexpired");
        assert!(validate_guest_publisher_pairing_ticket_at_v1(
            &ticket,
            ticket.challenge.expires_at_unix_ns
        )
        .is_err());

        let mut malformed_spki = ticket.clone();
        malformed_spki.signer_spki_der = URL_SAFE_NO_PAD.encode([0x30_u8, 0x00]);
        assert!(validate_guest_publisher_pairing_ticket_v1(&malformed_spki).is_err());

        let mut high_s = ticket.clone();
        let payload =
            canonical_lifecycle_signature_payload_v1(&high_s.schema_owner, &high_s).unwrap();
        let signature: P256Signature = signing_key.sign(&payload);
        let low_s = signature.normalize_s().unwrap_or(signature);
        let mut high_s_bytes = low_s.to_bytes();
        let scalar = &high_s_bytes[32..];
        let order = [
            0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xbc, 0xe6, 0xfa, 0xad, 0xa7, 0x17, 0x9e, 0x84, 0xf3, 0xb9, 0xca, 0xc2,
            0xfc, 0x63, 0x25, 0x51,
        ];
        let mut opposite_s = [0_u8; 32];
        let mut borrow = 0_i16;
        for index in (0..32).rev() {
            let difference = order[index] as i16 - scalar[index] as i16 - borrow;
            if difference < 0 {
                opposite_s[index] = (difference + 256) as u8;
                borrow = 1;
            } else {
                opposite_s[index] = difference as u8;
                borrow = 0;
            }
        }
        assert_eq!(borrow, 0);
        high_s_bytes[32..].copy_from_slice(&opposite_s);
        P256Signature::from_slice(&high_s_bytes).expect("valid high-S representation");
        high_s.signature.signature = URL_SAFE_NO_PAD.encode(high_s_bytes);
        assert!(validate_guest_publisher_pairing_ticket_v1(&high_s).is_err());

        let mut generation_mismatch = ticket.clone();
        generation_mismatch.host_generation += 1;
        assert!(validate_guest_publisher_pairing_ticket_v1(&generation_mismatch).is_err());

        let mut machine_mismatch = ticket;
        machine_mismatch.challenge.guest_machine_identity.clear();
        assert!(validate_guest_publisher_pairing_ticket_v1(&machine_mismatch).is_err());
    }

    #[test]
    fn r6_pairing_host_record_rejects_stale_generation_replay_and_expiry() {
        let (signing_key, ticket) = valid_mac_pairing_ticket_v1();
        let opened = signed_mac_pairing_host_record_v1(
            &signing_key,
            ticket.clone(),
            1,
            "sessions_opened",
            None,
        );
        validate_guest_publisher_pairing_host_record_v1(&opened).expect("initial host record");
        let opened_sha256 = guest_publisher_pairing_host_record_sha256_v1(&opened).unwrap();
        let proof_verified = signed_mac_pairing_host_record_v1(
            &signing_key,
            ticket.clone(),
            2,
            "operator_proof_verified",
            Some(opened_sha256),
        );
        compare_and_swap_guest_publisher_pairing_host_record_v1(&opened, &proof_verified)
            .expect("exact proof-verification transition");

        let mut stale_generation = proof_verified.clone();
        stale_generation.record_generation = 3;
        stale_generation.signature.signature = sign_p256_record_v1(
            &signing_key,
            &stale_generation.schema_owner,
            &stale_generation,
        );
        assert!(compare_and_swap_guest_publisher_pairing_host_record_v1(
            &opened,
            &stale_generation
        )
        .is_err());

        let mut replay = proof_verified.clone();
        replay.data_session.ticket.challenge.challenge = URL_SAFE_NO_PAD.encode([0x99_u8; 32]);
        replay.data_session.ticket.challenge_sha256 = lower_hex(&Sha256::digest(
            canonical_json_to_vec(&replay.data_session.ticket.challenge).unwrap(),
        ));
        replay.data_session.ticket.signature.signature = sign_p256_record_v1(
            &signing_key,
            &replay.data_session.ticket.schema_owner,
            &replay.data_session.ticket,
        );
        replay.signature.signature =
            sign_p256_record_v1(&signing_key, &replay.schema_owner, &replay);
        assert!(compare_and_swap_guest_publisher_pairing_host_record_v1(&opened, &replay).is_err());

        let mut expired_ticket = ticket;
        expired_ticket.challenge.expires_at_unix_ns = 1;
        expired_ticket.challenge_sha256 = lower_hex(&Sha256::digest(
            canonical_json_to_vec(&expired_ticket.challenge).unwrap(),
        ));
        expired_ticket.signature.signature =
            sign_p256_record_v1(&signing_key, &expired_ticket.schema_owner, &expired_ticket);
        let expired_opened = signed_mac_pairing_host_record_v1(
            &signing_key,
            expired_ticket.clone(),
            1,
            "sessions_opened",
            None,
        );
        let expired_opened_sha256 =
            guest_publisher_pairing_host_record_sha256_v1(&expired_opened).unwrap();
        let expired_proof_verified = signed_mac_pairing_host_record_v1(
            &signing_key,
            expired_ticket,
            2,
            "operator_proof_verified",
            Some(expired_opened_sha256),
        );
        assert!(compare_and_swap_guest_publisher_pairing_host_record_at_v1(
            &expired_opened,
            &expired_proof_verified,
            1,
        )
        .is_err());
    }

    #[test]
    fn r6_pairing_transitions_require_signed_hello_transcript_and_terminal_close() {
        let (host_key, ticket) = valid_mac_pairing_ticket_v1();
        let opened = signed_mac_pairing_host_record_v1(
            &host_key,
            ticket.clone(),
            1,
            "sessions_opened",
            None,
        );
        let proof_verified = signed_mac_pairing_host_record_v1(
            &host_key,
            ticket.clone(),
            2,
            "operator_proof_verified",
            Some(guest_publisher_pairing_host_record_sha256_v1(&opened).unwrap()),
        );
        compare_and_swap_guest_publisher_pairing_host_record_v1(&opened, &proof_verified)
            .expect("operator proof verification transition");

        let mut guest_root = proof_verified.clone();
        guest_root.record_generation = 3;
        guest_root.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&proof_verified).unwrap());
        guest_root.state = "guest_state_root_durable".to_string();
        guest_root.data_session.state = "guest_state_root_durable".to_string();
        guest_root.signature.signature =
            sign_p256_record_v1(&host_key, &guest_root.schema_owner, &guest_root);
        compare_and_swap_guest_publisher_pairing_host_record_v1(&proof_verified, &guest_root)
            .expect("guest root durability transition follows operator-proof verification");

        let guest_key = Ed25519SigningKey::from_bytes(&[0x42_u8; 32]);
        let guest_public_key = URL_SAFE_NO_PAD.encode(guest_key.verifying_key().as_bytes());
        let mut hello = GuestPublisherBootstrapHelloV1 {
            schema_owner: "substrate.guest-publisher-bootstrap-hello".to_string(),
            schema_version: 1,
            ticket_sha256: lower_hex(&Sha256::digest(
                canonical_guest_publisher_pairing_ticket_v1(&ticket).unwrap(),
            )),
            intent_sha256: "a".repeat(64),
            guest_test_retirement_commitment: None,
            guest_machine_identity: ticket.challenge.guest_machine_identity.clone(),
            guest_artifact_sha256: ticket.challenge.executor_build_evidence_sha256.clone(),
            intent_published_at_unix_ns: 1,
            intent_parent_fsync_observed: true,
            nonce: guest_root.binding.pairing_session_nonce.clone(),
            guest_public_key: guest_public_key.clone(),
            signature: LifecycleSignatureV1 {
                algorithm: "ed25519-v1".to_string(),
                public_key: guest_public_key,
                signature: String::new(),
            },
        };
        hello.signature.signature = URL_SAFE_NO_PAD.encode(
            guest_key
                .sign(
                    &canonical_lifecycle_signature_payload_v1(&hello.schema_owner, &hello).unwrap(),
                )
                .to_bytes(),
        );
        validate_guest_publisher_bootstrap_hello_v1(&ticket, &guest_root.binding, &hello)
            .expect("host verifies signed exact guest Hello before persistence");

        let mut hello_durable = guest_root.clone();
        hello_durable.record_generation = 4;
        hello_durable.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&guest_root).unwrap());
        hello_durable.state = "hello_durable".to_string();
        hello_durable.data_session.state = "hello_durable".to_string();
        hello_durable.data_session.hello = Some(hello.clone());
        hello_durable.signature.signature =
            sign_p256_record_v1(&host_key, &hello_durable.schema_owner, &hello_durable);
        compare_and_swap_guest_publisher_pairing_host_record_v1(&guest_root, &hello_durable)
            .expect("Hello durable transition");

        let mut transcript = GuestPublisherBootstrapTranscriptV1 {
            schema_owner: "substrate.guest-publisher-bootstrap-transcript".to_string(),
            schema_version: 1,
            ticket_sha256: hello.ticket_sha256.clone(),
            hello_sha256: guest_publisher_bootstrap_hello_sha256_v1(&hello).unwrap(),
            guest_test_retirement_commitment: None,
            guest_machine_identity: ticket.challenge.guest_machine_identity.clone(),
            staged_executor_sha256: ticket.challenge.executor_build_evidence_sha256.clone(),
            guest_nonce: hello_durable.binding.pairing_session_nonce.clone(),
            guest_public_key_sha256: guest_publisher_bootstrap_hello_public_key_sha256_v1(&hello)
                .unwrap(),
            signature: LifecycleSignatureV1 {
                algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
                public_key: ticket.signer_spki_der.clone(),
                signature: String::new(),
            },
        };
        transcript.signature.signature =
            sign_p256_record_v1(&host_key, &transcript.schema_owner, &transcript);
        validate_guest_publisher_bootstrap_transcript_v1(
            &ticket,
            &hello_durable.binding,
            &hello,
            &transcript,
        )
        .expect("host verifies signed exact transcript before persistence");

        let mut transcript_durable = hello_durable.clone();
        transcript_durable.record_generation = 5;
        transcript_durable.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&hello_durable).unwrap());
        transcript_durable.state = "transcript_durable".to_string();
        transcript_durable.data_session.state = "transcript_durable".to_string();
        transcript_durable.data_session.transcript = Some(transcript.clone());
        transcript_durable.signature.signature = sign_p256_record_v1(
            &host_key,
            &transcript_durable.schema_owner,
            &transcript_durable,
        );
        compare_and_swap_guest_publisher_pairing_host_record_v1(
            &hello_durable,
            &transcript_durable,
        )
        .expect("transcript durable transition");

        let mut consumed = transcript_durable.clone();
        consumed.record_generation = 6;
        consumed.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&transcript_durable).unwrap());
        consumed.state = "ticket_consumed".to_string();
        consumed.data_session.state = "ticket_consumed".to_string();
        consumed.data_session.guest_anchor_sha256 = Some("b".repeat(64));
        consumed.data_session.ended_observation = Some("guest-anchor-accepted".to_string());
        consumed.operator_tty_session.state = "closed".to_string();
        consumed.operator_tty_session.ended_observation =
            Some("operator-tty-confirmed".to_string());
        consumed.signature.signature =
            sign_p256_record_v1(&host_key, &consumed.schema_owner, &consumed);
        compare_and_swap_guest_publisher_pairing_host_record_v1(&transcript_durable, &consumed)
            .expect("terminal ticket consumption transition");

        let mut forged_hello = hello;
        forged_hello.nonce = "0".repeat(64);
        assert!(validate_guest_publisher_bootstrap_hello_v1(
            &ticket,
            &consumed.binding,
            &forged_hello,
        )
        .is_err());
    }

    #[test]
    fn r6_pairing_loss_observation_is_cas_bound_and_cannot_change_protocol_state() {
        let (host_key, ticket) = valid_mac_pairing_ticket_v1();
        let opened =
            signed_mac_pairing_host_record_v1(&host_key, ticket, 1, "sessions_opened", None);
        let mut lost = opened.clone();
        lost.record_generation = 2;
        lost.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&opened).unwrap());
        lost.data_session.failure_observation =
            Some("data-session-closed-or-cancelled".to_string());
        lost.signature.signature = sign_p256_record_v1(&host_key, &lost.schema_owner, &lost);
        compare_and_swap_guest_publisher_pairing_host_record_v1(&opened, &lost)
            .expect("one fixed loss observation advances the signed generation-CAS record");

        let mut altered = lost.clone();
        altered.record_generation = 3;
        altered.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&lost).unwrap());
        altered.data_session.state = "guest_state_root_durable".to_string();
        altered.signature.signature =
            sign_p256_record_v1(&host_key, &altered.schema_owner, &altered);
        assert!(compare_and_swap_guest_publisher_pairing_host_record_v1(&lost, &altered).is_err());
    }

    #[test]
    fn r6_operator_pre_intent_failure_is_a_signed_one_way_durable_closure() {
        let (host_key, ticket) = valid_mac_pairing_ticket_v1();
        let opened =
            signed_mac_pairing_host_record_v1(&host_key, ticket, 1, "sessions_opened", None);
        let mut closed = opened.clone();
        closed.record_generation = 2;
        closed.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&opened).unwrap());
        closed.state = "pre_intent_closed".to_string();
        closed.operator_tty_session.failure_observation =
            Some("operator-tty-direct-child-failed".to_string());
        closed.signature.signature = sign_p256_record_v1(&host_key, &closed.schema_owner, &closed);

        compare_and_swap_guest_publisher_pairing_host_record_v1(&opened, &closed)
            .expect("exact operator failure must become a durable pre-intent closure");

        let mut reopened = closed.clone();
        reopened.record_generation = 3;
        reopened.previous_record_sha256 =
            Some(guest_publisher_pairing_host_record_sha256_v1(&closed).unwrap());
        reopened.state = "operator_proof_verified".to_string();
        reopened.operator_tty_session.state = "closed".to_string();
        reopened.operator_tty_session.ended_observation =
            Some("operator-child-exited-successfully".to_string());
        reopened.operator_proof_sha256 = Some("f".repeat(64));
        reopened.signature.signature =
            sign_p256_record_v1(&host_key, &reopened.schema_owner, &reopened);
        assert!(
            compare_and_swap_guest_publisher_pairing_host_record_v1(&closed, &reopened).is_err()
        );
    }

    #[test]
    fn r6_pairing_host_record_requires_distinct_pm_bound_sessions() {
        let (signing_key, ticket) = valid_mac_pairing_ticket_v1();
        let binding = GuestPublisherPairingSessionBindingV1 {
            schema_owner: GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_OWNER_V1.to_string(),
            schema_version: GUEST_PUBLISHER_PAIRING_SESSION_BINDING_SCHEMA_VERSION_V1,
            scope_id: ticket.current_anchor.scope_id.clone(),
            platform_mapping_commitment: ticket.challenge.platform_mapping_commitment.clone(),
            guest_machine_identity: ticket.challenge.guest_machine_identity.clone(),
            source_commit: ticket.challenge.source_commit.clone(),
            source_tree: ticket.challenge.source_tree.clone(),
            source_ref: ticket.challenge.source_ref.clone(),
            staged_executor_sha256: ticket.challenge.executor_build_evidence_sha256.clone(),
            stage_one_record_sha256: "e".repeat(64),
            ticket_challenge_id: ticket.challenge.challenge_id.clone(),
            host_record_generation: ticket.host_generation,
            pairing_session_nonce: "f".repeat(64),
        };
        let data_session = GuestPublisherPairingDataSessionV1 {
            schema_owner: GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_OWNER_V1.to_string(),
            schema_version: GUEST_PUBLISHER_PAIRING_DATA_SESSION_SCHEMA_VERSION_V1,
            binding: binding.clone(),
            session_id: "data-session-r6".to_string(),
            ticket: ticket.clone(),
            state: "opened".to_string(),
            hello: None,
            transcript: None,
            guest_anchor_sha256: None,
            started_observation: "data-started".to_string(),
            ended_observation: None,
            failure_observation: None,
        };
        let operator_tty_session = GuestPublisherPairingOperatorTtySessionV1 {
            schema_owner: GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_OWNER_V1.to_string(),
            schema_version: GUEST_PUBLISHER_PAIRING_OPERATOR_TTY_SESSION_SCHEMA_VERSION_V1,
            binding: binding.clone(),
            session_id: "operator-session-r6".to_string(),
            controlling_tty_identity: "guest-tty-r6".to_string(),
            state: "opened".to_string(),
            started_observation: "operator-started".to_string(),
            ended_observation: None,
            failure_observation: None,
        };
        let mut record = GuestPublisherPairingHostRecordV1 {
            schema_owner: "substrate.guest-publisher-pairing-host-record".to_string(),
            schema_version: 1,
            current_anchor_counter: ticket.host_counter,
            current_anchor_sha256: ticket.current_anchor_sha256.clone(),
            ticket,
            binding,
            data_session,
            operator_tty_session,
            operator_launch_sha256: "a".repeat(64),
            operator_proof_sha256: None,
            record_generation: 1,
            state: "sessions_opened".to_string(),
            previous_record_sha256: None,
            signature: LifecycleSignatureV1 {
                algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
                public_key: String::new(),
                signature: String::new(),
            },
        };
        record.signature.public_key = record.ticket.signer_spki_der.clone();
        record.signature.signature =
            sign_p256_record_v1(&signing_key, &record.schema_owner, &record);
        validate_guest_publisher_pairing_host_record_v1(&record).expect("valid R6 host record");

        let mut third_session = serde_json::to_value(&record).expect("encode exact host record");
        third_session
            .as_object_mut()
            .expect("host record JSON object")
            .insert(
                "third_session".to_string(),
                serde_json::json!({"session_id": "forbidden-third-session"}),
            );
        assert!(
            serde_json::from_value::<GuestPublisherPairingHostRecordV1>(third_session).is_err()
        );

        let mut duplicate_identity = record;
        duplicate_identity.operator_tty_session.session_id =
            duplicate_identity.data_session.session_id.clone();
        duplicate_identity.signature.signature = sign_p256_record_v1(
            &signing_key,
            &duplicate_identity.schema_owner,
            &duplicate_identity,
        );
        assert!(
            validate_guest_publisher_pairing_host_record_v1(&duplicate_identity)
                .expect_err("R6 session identities must be distinct")
                .to_string()
                .contains("must be distinct")
        );
    }

    #[test]
    fn mac_protected_state_validation_rejects_malformed_p256_spki() {
        let (_, ticket) = valid_mac_pairing_ticket_v1();
        let mut state = LifecyclePublisherProtectedStateV1 {
            schema_owner: LIFECYCLE_PUBLISHER_PROTECTED_STATE_OWNER_V1.to_string(),
            schema_version: 1,
            current_anchor: ticket.current_anchor,
            counter: 1,
            prepared_record: None,
            previous_protected_state_sha256: None,
            state_revision: 1,
        };
        validate_lifecycle_publisher_protected_state_v1(&state)
            .expect("valid P-256 protected state");
        state.current_anchor.signature.public_key = URL_SAFE_NO_PAD.encode([0x30_u8, 0x00]);
        assert!(validate_lifecycle_publisher_protected_state_v1(&state).is_err());
    }

    #[test]
    fn bootstrap_component_target_derivation_covers_reserved_guest_scope() {
        let role = PublisherBootstrapComponentRoleV1(
            "GuestBootstrapIntent(mac_lima,018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a97)".to_string(),
        );
        let target = derive_publisher_bootstrap_component_target_v1(&role, "scope-1")
            .expect("derive target");
        assert_eq!(
            target,
            "/var/lib/substrate/.substrate-lifecycle-pairing-intent-v1.scope-1.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a97.json"
        );
    }

    #[test]
    fn complete_component_set_rejects_duplicate_targets() {
        let components = vec![
            PublisherBootstrapComponentV1 {
                component_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
                role: PublisherBootstrapComponentRoleV1("LinuxStateRoot".to_string()),
                target_identity: "/var/lib/substrate".to_string(),
                object_type: "directory".to_string(),
                expected_before: Value::Object(Map::new()),
                source_artifact_sha256: None,
                source_signature_sha256: None,
                owner: None,
                group_name: None,
                mode: Some("0755".to_string()),
                acl_or_security: None,
                code_requirement: None,
                dependency_component_ids: Vec::new(),
                durability_method: "fsync_parent".to_string(),
            },
            PublisherBootstrapComponentV1 {
                component_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string(),
                role: PublisherBootstrapComponentRoleV1("LinuxStateRoot".to_string()),
                target_identity: "/var/lib/substrate".to_string(),
                object_type: "directory".to_string(),
                expected_before: Value::Object(Map::new()),
                source_artifact_sha256: None,
                source_signature_sha256: None,
                owner: None,
                group_name: None,
                mode: Some("0755".to_string()),
                acl_or_security: None,
                code_requirement: None,
                dependency_component_ids: Vec::new(),
                durability_method: "fsync_parent".to_string(),
            },
        ];
        assert!(validate_complete_publisher_bootstrap_component_set_v1(&components).is_err());
    }

    #[test]
    fn canonical_json_rejects_floats() {
        let mut value = BTreeMap::new();
        value.insert("float".to_string(), Value::from(1.25_f64));
        let value = Value::Object(value.into_iter().collect());
        assert!(canonical_json_value_to_vec(&value).is_err());
    }

    #[test]
    fn executor_build_evidence_validation_accepts_minimal_shape() {
        let evidence = ExecutorBuildEvidenceV1 {
            schema_owner: "substrate.executor-build-evidence".to_string(),
            schema_version: 1,
            source_commit: "1".repeat(40),
            source_tree: "2".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            artifact_sha256: "3".repeat(64),
            artifact_identity: "inode:123".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            tool_versions: BTreeMap::new(),
            code_identity: None,
        };
        validate_executor_build_evidence_v1(&evidence).expect("evidence");
    }

    fn mac_bootstrap_authorization_with_bound_control_v1() -> PublisherBootstrapAuthorizationV1 {
        let scope_id = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string();
        let mut pre_pm_manifest = sample_manifest();
        pre_pm_manifest.manifest_id = format!("m1:{scope_id}:1");
        pre_pm_manifest.authority_domain = "mac_host_shared".to_string();
        pre_pm_manifest.platform_kind = "mac_lima".to_string();
        pre_pm_manifest.platform_mapping_commitment = None;
        pre_pm_manifest.installation_id = scope_id.clone();
        pre_pm_manifest.attempt_id = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string();
        pre_pm_manifest.intended_principal = "alice".to_string();
        pre_pm_manifest.manifest_generation = 1;
        pre_pm_manifest.previous_manifest_sha256 = None;
        pre_pm_manifest.entries.clear();
        pre_pm_manifest.planned_action_receipts.clear();
        pre_pm_manifest.manifest_sha256 =
            manifest_digest_v1(&pre_pm_manifest).expect("canonical pre-PM manifest digest");
        PublisherBootstrapAuthorizationV1 {
            schema_owner: "substrate.publisher-bootstrap-authorization".to_string(),
            schema_version: 1,
            authority_domain: "mac_host_shared".to_string(),
            scope_id: scope_id.clone(),
            host_context_commitment: "1".repeat(64),
            install_bootstrap_context_v1: "fixture-carrier".to_string(),
            platform_mapping_commitment: None,
            requester_principal: "alice".to_string(),
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            manifest_generation: 1,
            manifest_sha256: pre_pm_manifest.manifest_sha256.clone(),
            pre_pm_manifest: Some(pre_pm_manifest),
            issued_at_unix_ns: 1,
            expires_at_unix_ns: 2,
            attempt_nonce: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string(),
            publisher_expected_absent: true,
            executor_build_evidence: ExecutorBuildEvidenceV1 {
                schema_owner: "substrate.executor-build-evidence".to_string(),
                schema_version: 1,
                source_commit: "a".repeat(40),
                source_tree: "b".repeat(40),
                source_ref: "refs/heads/test".to_string(),
                artifact_sha256: "4".repeat(64),
                artifact_identity: "inode:123".to_string(),
                target_triple: "aarch64-apple-darwin".to_string(),
                tool_versions: BTreeMap::new(),
                code_identity: None,
            },
            mac_control_authority: Some(MacPublisherControlAuthorityV1 {
                schema_owner: MAC_PUBLISHER_CONTROL_AUTHORITY_OWNER_V1.to_string(),
                schema_version: 1,
                control_binary: "substrate-lifecycle-control".to_string(),
                source_commit: "a".repeat(40),
                source_tree: "b".repeat(40),
                source_ref: "refs/heads/test".to_string(),
                target_triple: "aarch64-apple-darwin".to_string(),
                artifact_sha256: "3".repeat(64),
                designated_requirement: concat!(
                    "anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\" ",
                    "and cdhash H\"0123456789abcdef0123456789abcdef01234567\""
                )
                .to_string(),
            }),
            components: vec![PublisherBootstrapComponentV1 {
                component_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92".to_string(),
                role: PublisherBootstrapComponentRoleV1("MacProtectedState".to_string()),
                target_identity: format!("{scope_id}:current-anchor"),
                object_type: "keychain-record".to_string(),
                expected_before: Value::Object(Map::new()),
                source_artifact_sha256: None,
                source_signature_sha256: None,
                owner: None,
                group_name: None,
                mode: None,
                acl_or_security: None,
                code_requirement: None,
                dependency_component_ids: Vec::new(),
                durability_method: "keychain_cas".to_string(),
            }],
            test_retirement_commitment: None,
            confirmation: "CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER".to_string(),
        }
    }

    fn mac_install_provenance_fixture_v1() -> MacPublisherInstallProvenanceV1 {
        let authorization = mac_bootstrap_authorization_with_bound_control_v1();
        let control_authority = authorization
            .mac_control_authority
            .expect("fixture mac control authority");
        let image = |artifact_sha256: String,
                     physical_identity: &str,
                     code_identity: &str,
                     code_requirement: String| {
            MacPublisherBootstrapImageProvenanceV1 {
                target_triple: "aarch64-apple-darwin".to_string(),
                artifact_sha256,
                physical_identity: physical_identity.to_string(),
                code_identity: code_identity.to_string(),
                code_requirement,
            }
        };
        MacPublisherInstallProvenanceV1 {
            schema_owner: MAC_PUBLISHER_INSTALL_PROVENANCE_OWNER_V1.to_string(),
            schema_version: 2,
            source_commit: control_authority.source_commit.clone(),
            source_tree: control_authority.source_tree.clone(),
            source_ref: control_authority.source_ref.clone(),
            review_record_sha256: "a".repeat(64),
            host_context_commitment: "b".repeat(64),
            selected_host_prefix: "/tmp/substrate-r5".to_string(),
            control_image: image(
                control_authority.artifact_sha256.clone(),
                "dev:1:ino:10",
                "cdhash:0123456789abcdef0123456789abcdef01234567",
                control_authority.designated_requirement.clone(),
            ),
            executor_image: image(
                "c".repeat(64),
                "dev:1:ino:11",
                "cdhash:1111111111111111111111111111111111111111",
                "identifier substrate-lifecycle-macos".to_string(),
            ),
            launch_daemon_plist_sha256: "d".repeat(64),
            lima_tool: MacLimaToolProvenanceV1 {
                absolute_path: "/usr/local/bin/limactl".to_string(),
                image: image(
                    "e".repeat(64),
                    "dev:1:ino:12",
                    "cdhash:2222222222222222222222222222222222222222",
                    "identifier io.lima-vm.lima".to_string(),
                ),
                version: "limactl 1.0.0".to_string(),
            },
            retained_linux_artifacts: vec![
                MacLimaRetainedArtifactProvenanceV1 {
                    logical_role: "mac.lima.publisher-executor".to_string(),
                    retained_relative_path: "bin/linux/substrate-lifecycle-linux".to_string(),
                    cargo_package: "substrate".to_string(),
                    cargo_binary: "substrate-lifecycle-linux".to_string(),
                    target_triple: "aarch64-unknown-linux-gnu".to_string(),
                    cargo_lock_sha256: "1".repeat(64),
                    toolchain: "rustc 1.89.0 (fixture)".to_string(),
                    build_command: "cargo build --locked --offline --target aarch64-unknown-linux-gnu --release -p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service -p substrate-gateway --bin substrate-gateway -p substrate --bin substrate".to_string(),
                    artifact_sha256: "2".repeat(64),
                    physical_identity: "dev:1:ino:20".to_string(),
                    mode: "0755".to_string(),
                },
                MacLimaRetainedArtifactProvenanceV1 {
                    logical_role: "mac.lima.guest-binary(substrate-world-service)".to_string(),
                    retained_relative_path: "bin/linux/world-service".to_string(),
                    cargo_package: "world-service".to_string(),
                    cargo_binary: "world-service".to_string(),
                    target_triple: "aarch64-unknown-linux-gnu".to_string(),
                    cargo_lock_sha256: "1".repeat(64),
                    toolchain: "rustc 1.89.0 (fixture)".to_string(),
                    build_command: "cargo build --locked --offline --target aarch64-unknown-linux-gnu --release -p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service -p substrate-gateway --bin substrate-gateway -p substrate --bin substrate".to_string(),
                    artifact_sha256: "3".repeat(64),
                    physical_identity: "dev:1:ino:21".to_string(),
                    mode: "0755".to_string(),
                },
                MacLimaRetainedArtifactProvenanceV1 {
                    logical_role: "mac.lima.guest-binary(substrate-gateway)".to_string(),
                    retained_relative_path: "bin/linux/substrate-gateway".to_string(),
                    cargo_package: "substrate-gateway".to_string(),
                    cargo_binary: "substrate-gateway".to_string(),
                    target_triple: "aarch64-unknown-linux-gnu".to_string(),
                    cargo_lock_sha256: "1".repeat(64),
                    toolchain: "rustc 1.89.0 (fixture)".to_string(),
                    build_command: "cargo build --locked --offline --target aarch64-unknown-linux-gnu --release -p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service -p substrate-gateway --bin substrate-gateway -p substrate --bin substrate".to_string(),
                    artifact_sha256: "4".repeat(64),
                    physical_identity: "dev:1:ino:22".to_string(),
                    mode: "0755".to_string(),
                },
                MacLimaRetainedArtifactProvenanceV1 {
                    logical_role: "mac.lima.guest-binary(substrate)".to_string(),
                    retained_relative_path: "bin/linux/substrate".to_string(),
                    cargo_package: "substrate".to_string(),
                    cargo_binary: "substrate".to_string(),
                    target_triple: "aarch64-unknown-linux-gnu".to_string(),
                    cargo_lock_sha256: "1".repeat(64),
                    toolchain: "rustc 1.89.0 (fixture)".to_string(),
                    build_command: "cargo build --locked --offline --target aarch64-unknown-linux-gnu --release -p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service -p substrate-gateway --bin substrate-gateway -p substrate --bin substrate".to_string(),
                    artifact_sha256: "5".repeat(64),
                    physical_identity: "dev:1:ino:23".to_string(),
                    mode: "0755".to_string(),
                },
            ],
            profile_template_algorithm: "substrate.mac-lima-stage-one-profile-template".to_string(),
            profile_template_version: 1,
            profile_template_sha256: "f".repeat(64),
            control_authority,
        }
    }

    #[test]
    fn mac_install_provenance_and_direct_request_are_closed_and_canonical() {
        let provenance = mac_install_provenance_fixture_v1();
        let canonical = canonical_mac_publisher_install_provenance_v1(&provenance)
            .expect("canonical install provenance");
        let decoded: MacPublisherInstallProvenanceV1 =
            serde_json::from_slice(&canonical).expect("decode canonical install provenance");
        assert_eq!(decoded, provenance);

        let mut relative_prefix = provenance.clone();
        relative_prefix.selected_host_prefix = "relative/prefix".to_string();
        assert!(validate_mac_publisher_install_provenance_v1(&relative_prefix).is_err());

        let mut wrong_tool_target = provenance.clone();
        wrong_tool_target.lima_tool.image.target_triple = "x86_64-apple-darwin".to_string();
        assert!(validate_mac_publisher_install_provenance_v1(&wrong_tool_target).is_err());

        let mut wrong_control_cdhash = provenance.clone();
        wrong_control_cdhash.control_image.code_identity =
            "cdhash:ffffffffffffffffffffffffffffffffffffffff".to_string();
        assert!(validate_mac_publisher_install_provenance_v1(&wrong_control_cdhash).is_err());

        let request = MacPublisherBootstrapRequestV1 {
            install_bootstrap_context_v1: "fixture-carrier".to_string(),
        };
        let request_bytes = canonical_mac_publisher_bootstrap_request_v1(&request)
            .expect("canonical direct bootstrap request");
        assert_eq!(
            parse_mac_publisher_bootstrap_request_v1(&request_bytes)
                .expect("parse canonical direct bootstrap request"),
            request
        );
        let mut unknown_request = serde_json::to_value(&request).expect("encode request");
        unknown_request.as_object_mut().unwrap().insert(
            "manifest_generation".to_string(),
            Value::Number(1_u64.into()),
        );
        assert!(serde_json::from_value::<MacPublisherBootstrapRequestV1>(unknown_request).is_err());
        assert!(parse_mac_publisher_bootstrap_request_v1(
            br#"{"install_bootstrap_context_v1":"fixture-carrier","scope_id":"caller-selected"}"#
        )
        .is_err());
    }

    #[test]
    fn mac_bootstrap_attempt_locator_is_canonical_closed_and_fail_closed() {
        let request = MacPublisherBootstrapRequestV1 {
            install_bootstrap_context_v1: "fixture-carrier".to_string(),
        };
        let request_sha256 =
            mac_publisher_bootstrap_request_sha256_v1(&request).expect("direct request digest");
        let locator = MacPublisherBootstrapAttemptLocatorV1 {
            schema_owner: "substrate.mac-publisher-bootstrap-attempt-locator".to_string(),
            schema_version: 1,
            attempt_key_sha256: "1".repeat(64),
            authority_domain: "mac_host_shared".to_string(),
            host_context_commitment: "2".repeat(64),
            requester_principal: "alice".to_string(),
            selected_host_prefix: "/tmp/substrate-r5".to_string(),
            install_provenance_sha256: "3".repeat(64),
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
            attempt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string(),
            issued_at_unix_ns: 1,
            expires_at_unix_ns: 2,
            bootstrap_request_sha256: request_sha256,
            bootstrap_authorization_sha256: "4".repeat(64),
            pre_pm_manifest_sha256: "5".repeat(64),
            initial_anchor_sha256: Some("6".repeat(64)),
            protected_state_sha256: Some("7".repeat(64)),
            capsule_sha256: Some("8".repeat(64)),
            terminal_state: "Completed".to_string(),
            canonical_response_b64: Some(URL_SAFE_NO_PAD.encode(br#"{"ok":true}"#)),
            locator_revision: 2,
        };
        let canonical = canonical_mac_publisher_bootstrap_attempt_locator_v1(&locator)
            .expect("canonical completed locator");
        let decoded: MacPublisherBootstrapAttemptLocatorV1 =
            serde_json::from_slice(&canonical).expect("decode canonical locator");
        assert_eq!(decoded, locator);
        assert_eq!(
            mac_publisher_bootstrap_attempt_locator_sha256_v1(&locator).unwrap(),
            lower_hex(&Sha256::digest(canonical))
        );

        let mut serialized_authority = locator.clone();
        serialized_authority.canonical_response_b64 = Some(
            URL_SAFE_NO_PAD
                .encode(br#"{"publisher_bootstrap_authorization":{"scope_id":"caller-carried"}}"#),
        );
        assert!(
            canonical_mac_publisher_bootstrap_attempt_locator_v1(&serialized_authority).is_err()
        );

        let mut incomplete = locator.clone();
        incomplete.terminal_state = "Allocated".to_string();
        incomplete.initial_anchor_sha256 = None;
        incomplete.protected_state_sha256 = None;
        incomplete.capsule_sha256 = None;
        incomplete.canonical_response_b64 = None;
        incomplete.locator_revision = 1;
        assert!(canonical_mac_publisher_bootstrap_attempt_locator_v1(&incomplete).is_ok());
        incomplete.canonical_response_b64 = Some(URL_SAFE_NO_PAD.encode(br#"{"ok":true}"#));
        assert!(canonical_mac_publisher_bootstrap_attempt_locator_v1(&incomplete).is_err());

        let mut unknown = serde_json::to_value(locator).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("complete_authorization".to_string(), Value::Null);
        assert!(serde_json::from_value::<MacPublisherBootstrapAttemptLocatorV1>(unknown).is_err());
    }

    #[test]
    fn mac_bootstrap_authority_is_required_joined_canonical_and_closed() {
        let authorization = mac_bootstrap_authorization_with_bound_control_v1();
        let canonical = canonical_publisher_bootstrap_authorization_v1(&authorization)
            .expect("canonical mac bootstrap authorization");
        assert_eq!(
            parse_publisher_bootstrap_authorization_v1(&canonical).unwrap(),
            authorization
        );

        let mut missing = authorization.clone();
        missing.mac_control_authority = None;
        assert!(validate_publisher_bootstrap_authorization_v1(&missing)
            .unwrap_err()
            .to_string()
            .contains("requires mac_control_authority"));

        let mut mismatched = authorization.clone();
        mismatched
            .mac_control_authority
            .as_mut()
            .unwrap()
            .source_tree = "c".repeat(40);
        assert!(validate_publisher_bootstrap_authorization_v1(&mismatched)
            .unwrap_err()
            .to_string()
            .contains("does not join authorization"));

        assert_ne!(
            authorization
                .mac_control_authority
                .as_ref()
                .unwrap()
                .artifact_sha256,
            authorization.executor_build_evidence.artifact_sha256,
            "control and executor artifacts are independently measured"
        );

        let mut non_mac = authorization.clone();
        non_mac.authority_domain = "linux_system".to_string();
        let non_mac_error = validate_publisher_bootstrap_authorization_v1(&non_mac).unwrap_err();
        assert!(
            non_mac_error
                .to_string()
                .contains("valid only for mac_host_shared"),
            "{non_mac_error:#}"
        );

        let mut unknown = serde_json::to_value(authorization).unwrap();
        unknown.as_object_mut().unwrap().insert(
            "unrecognized_bootstrap_authority".to_string(),
            Value::Bool(true),
        );
        assert!(serde_json::from_value::<PublisherBootstrapAuthorizationV1>(unknown).is_err());
    }

    #[test]
    fn mac_bootstrap_manifest_identity_is_required_canonical_and_digest_bound() {
        let authorization = mac_bootstrap_authorization_with_bound_control_v1();
        let digest = publisher_bootstrap_authorization_sha256_v1(&authorization)
            .expect("authorization digest");

        let mut next_generation = authorization.clone();
        next_generation.manifest_generation = 2;
        next_generation.pre_pm_manifest = None;
        next_generation.publisher_expected_absent = false;
        next_generation.platform_mapping_commitment = Some("2".repeat(64));
        assert_ne!(
            publisher_bootstrap_authorization_sha256_v1(&next_generation).unwrap(),
            digest,
            "the anchor retry digest must bind the manifest generation"
        );

        let mut next_digest = authorization.clone();
        next_digest.manifest_sha256 = "f".repeat(64);
        next_digest.pre_pm_manifest = None;
        next_digest.publisher_expected_absent = false;
        next_digest.platform_mapping_commitment = Some("2".repeat(64));
        assert_ne!(
            publisher_bootstrap_authorization_sha256_v1(&next_digest).unwrap(),
            digest,
            "the anchor retry digest must bind the manifest digest"
        );

        let mut zero_generation = authorization.clone();
        zero_generation.manifest_generation = 0;
        assert!(
            validate_publisher_bootstrap_authorization_v1(&zero_generation)
                .unwrap_err()
                .to_string()
                .contains("manifest_generation")
        );

        let mut malformed_digest = authorization.clone();
        malformed_digest.manifest_sha256 = "BAD".to_string();
        assert!(
            validate_publisher_bootstrap_authorization_v1(&malformed_digest)
                .unwrap_err()
                .to_string()
                .contains("manifest_sha256")
        );

        let mut source_mismatch = authorization.clone();
        source_mismatch.executor_build_evidence.source_ref = "refs/heads/other".to_string();
        assert!(
            validate_publisher_bootstrap_authorization_v1(&source_mismatch)
                .unwrap_err()
                .to_string()
                .contains("executor build evidence does not join authorization source identity")
        );

        let mut invalid_pre_pm = authorization.clone();
        invalid_pre_pm.platform_mapping_commitment = None;
        invalid_pre_pm.manifest_generation = 2;
        assert!(
            validate_publisher_bootstrap_authorization_v1(&invalid_pre_pm)
                .unwrap_err()
                .to_string()
                .contains("pre-PM mac_host_shared")
        );
        let mut valid_pre_pm_shape = authorization.clone();
        valid_pre_pm_shape.platform_mapping_commitment = None;
        validate_publisher_bootstrap_authorization_v1(&valid_pre_pm_shape)
            .expect("generation-one absent authorization may carry the pre-PM form");

        let mut missing = serde_json::to_value(&authorization).unwrap();
        missing
            .as_object_mut()
            .unwrap()
            .remove("manifest_generation");
        assert!(serde_json::from_value::<PublisherBootstrapAuthorizationV1>(missing).is_err());
    }

    #[test]
    fn mac_pre_pm_manifest_exception_is_exactly_empty_generation_one_durable() {
        fn redigest(manifest: &mut ManagedArtifactManifestV1) {
            manifest.manifest_sha256 = manifest_digest_v1(manifest).expect("manifest digest");
        }

        let mut manifest = sample_manifest();
        manifest.manifest_id = format!("m1:{}:1", manifest.installation_id);
        manifest.authority_domain = "mac_host_shared".to_string();
        manifest.platform_kind = "mac_lima".to_string();
        manifest.platform_mapping_commitment = None;
        manifest.manifest_generation = 1;
        manifest.lifecycle_state = ManagedLifecycleStateV1::ManifestDurable;
        manifest.previous_manifest_sha256 = None;
        manifest.entries.clear();
        manifest.planned_action_receipts.clear();
        redigest(&mut manifest);
        validate_manifest_v1(&manifest).expect("exact greenfield pre-PM manifest");

        let mut wrong_generation = manifest.clone();
        wrong_generation.manifest_generation = 2;
        wrong_generation.manifest_id = format!("m1:{}:2", wrong_generation.installation_id);
        redigest(&mut wrong_generation);
        assert!(validate_manifest_v1(&wrong_generation).is_err());

        let mut prior = manifest.clone();
        prior.previous_manifest_sha256 = Some("a".repeat(64));
        redigest(&mut prior);
        assert!(validate_manifest_v1(&prior).is_err());

        let mut wrong_state = manifest.clone();
        wrong_state.lifecycle_state = ManagedLifecycleStateV1::ManifestPrepared;
        redigest(&mut wrong_state);
        assert!(validate_manifest_v1(&wrong_state).is_err());

        let mut post_pm = manifest.clone();
        post_pm.platform_mapping_commitment = Some("b".repeat(64));
        redigest(&mut post_pm);
        validate_manifest_v1(&post_pm).expect("a full PM-bound manifest remains valid");
    }

    #[test]
    fn mac_lima_stage_one_successor_template_is_pre_effect_closed_and_allows_anchor_counter_zero() {
        let scope_id = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string();
        let attempt_id = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string();
        let nonce = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92".to_string();
        let evidence = ExecutorBuildEvidenceV1 {
            schema_owner: "substrate.executor-build-evidence".to_string(),
            schema_version: 1,
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            source_ref: "refs/heads/r5-fixture".to_string(),
            artifact_sha256: "c".repeat(64),
            artifact_identity: "executor-fixture".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
            tool_versions: BTreeMap::new(),
            code_identity: Some("cdhash:0123456789abcdef0123456789abcdef01234567".to_string()),
        };
        let authorization = LimaStageOneAuthorizationV1 {
            schema_owner: LIMA_STAGE_ONE_SCHEMA_OWNER_V1.to_string(),
            schema_version: 1,
            host_context_commitment: "d".repeat(64),
            lima_control_root_identity: "/Users/fixture/.lima".to_string(),
            instance_name: "substrate".to_string(),
            profile_sha256: "e".repeat(64),
            expected_absent: true,
            source_commit: evidence.source_commit.clone(),
            source_tree: evidence.source_tree.clone(),
            source_ref: evidence.source_ref.clone(),
            executor_receipt_sha256: "f".repeat(64),
            requester_principal: "fixture".to_string(),
            attempt_id: attempt_id.clone(),
            nonce: nonce.clone(),
            expires_at_unix_ns: 42,
            rendered_profile_b64: URL_SAFE_NO_PAD.encode(b"{}"),
            rendered_profile_sha256: lower_hex(&Sha256::digest(b"{}")),
            successor_template: MacLimaStageOneSuccessorTemplateV1 {
                schema_owner: "substrate.mac-lima-stage-one-successor-template".to_string(),
                schema_version: 1,
                derivation_algorithm:
                    "mac_lima_stage_one_successor_template_v1_platform_mapping_then_manifest"
                        .to_string(),
                profile_template_algorithm: "substrate.mac-lima-stage-one-profile-template"
                    .to_string(),
                profile_template_version: 1,
                profile_template_sha256: "0".repeat(64),
                current_pre_pm_manifest_generation: 1,
                current_pre_pm_manifest_sha256: "1".repeat(64),
                current_anchor_sha256: "2".repeat(64),
                current_anchor_counter: 0,
                next_manifest_generation: 2,
                previous_manifest_sha256: "1".repeat(64),
                host_context_commitment: "d".repeat(64),
                scope_id: scope_id.clone(),
                installation_id: scope_id,
                intended_principal: "fixture".to_string(),
                selected_host_prefix: "/tmp/substrate-r5".to_string(),
                host_platform_control_root: "/Users/fixture/.lima".to_string(),
                instance_name: "substrate".to_string(),
                profile_sha256: "e".repeat(64),
                source_commit: evidence.source_commit.clone(),
                source_tree: evidence.source_tree.clone(),
                source_ref: evidence.source_ref.clone(),
                executor_build_evidence: evidence,
                attempt_id,
                nonce,
                expires_at_unix_ns: 42,
                manifest_created_at_unix_ns: 1,
                manifest_lifecycle_state: ManagedLifecycleStateV1::ManifestDurable,
                ordered_non_machine_entries: Vec::new(),
                planned_receipt_id: "stage-one-receipt".to_string(),
                planned_receipt_relative_path: "receipts/2/stage-one-receipt.json".to_string(),
                post_effect_observation_slots: vec!["guest_machine_id".to_string()],
            },
            signature: LifecycleSignatureV1 {
                algorithm: "ed25519-v1".to_string(),
                public_key: "fixture".to_string(),
                signature: "fixture".to_string(),
            },
        };
        validate_mac_lima_stage_one_successor_template_v1(
            &authorization.successor_template,
            &authorization,
        )
        .expect("counter zero is a valid initial signed pre-PM anchor counter");

        let mut placeholder = authorization.successor_template.clone();
        placeholder.post_effect_observation_slots =
            vec!["guest_machine_id".to_string(), "other".to_string()];
        assert!(
            validate_mac_lima_stage_one_successor_template_v1(&placeholder, &authorization)
                .is_err()
        );
        placeholder = authorization.successor_template.clone();
        placeholder.next_manifest_generation = 3;
        assert!(
            validate_mac_lima_stage_one_successor_template_v1(&placeholder, &authorization)
                .is_err()
        );
        placeholder = authorization.successor_template.clone();
        placeholder.previous_manifest_sha256 = "3".repeat(64);
        assert!(
            validate_mac_lima_stage_one_successor_template_v1(&placeholder, &authorization)
                .is_err()
        );
    }
}
