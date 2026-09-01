//! Shared request/response models and error types for the Agent API.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use substrate_common::agent_events::AgentEvent;
pub use substrate_common::agent_events::{
    RuntimeEventIdentityV1, RuntimeFrameIdentityV1, RuntimeTerminalIdentityV1,
    RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
};
pub use substrate_common::{
    validate_identity_tuple_and_placement_posture, FsDiff, HostTransitionWorkCorrelationV1,
    IdentityTuple, OpaqueAuthorityCommitmentV1, PlacementExecution, PlacementPosture, ProcessEvent,
    ProcessEventType, ProcessEventsStatus, ProcessTelemetry, WorldFsMode,
};
pub use world_api::{
    SharedWorldBindingSnapshot, SharedWorldBindingState, SharedWorldOwnerAction,
    SharedWorldOwnerSpec, WorldReuseMode,
};

const INSTALL_BOOTSTRAP_CONTEXT_DOMAIN_V1: &str = "substrate.install_bootstrap_context";
const PLATFORM_BOOTSTRAP_MAPPING_DOMAIN_V1: &str = "substrate.platform_bootstrap_mapping";
const WINDOWS_FORWARDER_SCOPE_DOMAIN_V1: &str = "substrate.windows_forwarder_scope";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlatformPrincipalV1 {
    Unix { account: String, uid: u32 },
    Windows { account: String, sid: String },
}

/// Bounded identity for the concrete world targeted by one narrowing request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldBindingRefV1 {
    pub world_id: String,
    pub world_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityObjectKindV1 {
    AgentDescriptor,
    RetainedWorker,
    ResumeHandle,
    Policy,
    HostAttachContract,
    TransitionTransportPayload,
    TransitionInput,
    LeaseToken,
    ApplicationResult,
    InputAcceptance,
    StartupOwnershipResult,
    ObligationSnapshot,
    PostTurnProtocolEvent,
    PostTurnCompletion,
    TerminalHandoff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorityObjectRefV1 {
    pub ref_id: String,
    pub object_kind: AuthorityObjectKindV1,
    pub schema_version: u32,
    pub commitment: OpaqueAuthorityCommitmentV1,
}

pub type PolicyRefV1 = AuthorityObjectRefV1;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityObjectRefV1Def {
    ref_id: String,
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
    commitment: OpaqueAuthorityCommitmentV1,
}

impl<'de> Deserialize<'de> for AuthorityObjectRefV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = AuthorityObjectRefV1Def::deserialize(deserializer)?;
        let reference = Self {
            ref_id: value.ref_id,
            object_kind: value.object_kind,
            schema_version: value.schema_version,
            commitment: value.commitment,
        };
        reference.validate().map_err(serde::de::Error::custom)?;
        Ok(reference)
    }
}

impl AuthorityObjectRefV1 {
    pub fn validate(&self) -> Result<(), String> {
        let suffix = self.ref_id.strip_prefix("ao_").ok_or_else(|| {
            "parent_policy_ref.ref_id must be 'ao_' plus 32 lowercase hexadecimal characters"
                .to_string()
        })?;
        if suffix.len() != 32
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "parent_policy_ref.ref_id must be 'ao_' plus 32 lowercase hexadecimal characters"
                    .to_string(),
            );
        }
        if self.object_kind != AuthorityObjectKindV1::Policy {
            return Err("parent_policy_ref.object_kind must be policy".to_string());
        }
        if self.schema_version == 0 {
            return Err("parent_policy_ref.schema_version must be positive".to_string());
        }
        self.commitment.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DispatchCapabilitySubjectV1 {
    EphemeralTask,
    RetainedWorkerSpawn,
    RetainedWorkerTurn { retained_participant_id: String },
    RetainedWorkerFork { source_participant_id: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedPolicyPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub world_fs: Option<RestrictedWorldFsPatchV1>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub host_visible: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub fail_closed: Option<RestrictedWorldFsFailClosedPatchV1>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub deny_enforcement: Option<WorldFsDenyEnforcementV3>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub caged_required: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub discover: Option<RestrictedWorldFsDimensionPatchV1>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub read: Option<RestrictedWorldFsDimensionPatchV1>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub write: Option<RestrictedWorldFsWritePatchV1>,
}

impl RestrictedWorldFsPatchV1 {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsFailClosedPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub routing: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsDimensionPatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub allow_list: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub deny_list: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedWorldFsWritePatchV1 {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub enabled: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub allow_list: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_non_null"
    )]
    pub deny_list: Option<Vec<String>>,
}

fn deserialize_optional_non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DispatchPolicyNarrowingPatchV1 {
    pub schema_version: u32,
    pub request_id: String,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub target_backend_id: String,
    pub target_world: WorldBindingRefV1,
    pub applies_to: DispatchCapabilitySubjectV1,
    pub parent_policy_ref: PolicyRefV1,
    pub parent_policy_revision: String,
    pub restricted_policy_patch: RestrictedPolicyPatchV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DispatchPolicyNarrowingPatchV1Def {
    schema_version: u32,
    request_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_backend_id: String,
    target_world: WorldBindingRefV1,
    applies_to: DispatchCapabilitySubjectV1,
    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    restricted_policy_patch: RestrictedPolicyPatchV1,
    reason: Option<String>,
}

impl<'de> Deserialize<'de> for DispatchPolicyNarrowingPatchV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = DispatchPolicyNarrowingPatchV1Def::deserialize(deserializer)?;
        let carrier = Self {
            schema_version: value.schema_version,
            request_id: value.request_id,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            target_backend_id: value.target_backend_id,
            target_world: value.target_world,
            applies_to: value.applies_to,
            parent_policy_ref: value.parent_policy_ref,
            parent_policy_revision: value.parent_policy_revision,
            restricted_policy_patch: value.restricted_policy_patch,
            reason: value.reason,
        };
        carrier.validate().map_err(serde::de::Error::custom)?;
        Ok(carrier)
    }
}

impl DispatchPolicyNarrowingPatchV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported dispatch narrowing schema_version {} (expected 1)",
                self.schema_version
            ));
        }
        for (name, value) in [
            ("request_id", self.request_id.as_str()),
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("caller_participant_id", self.caller_participant_id.as_str()),
            ("target_backend_id", self.target_backend_id.as_str()),
            ("target_world.world_id", self.target_world.world_id.as_str()),
            (
                "parent_policy_revision",
                self.parent_policy_revision.as_str(),
            ),
        ] {
            validate_narrowing_identity(name, value)?;
        }
        if self.target_world.world_generation == 0 {
            return Err("target_world.world_generation must be positive".to_string());
        }
        match &self.applies_to {
            DispatchCapabilitySubjectV1::RetainedWorkerTurn {
                retained_participant_id,
            } => validate_narrowing_identity(
                "applies_to.retained_participant_id",
                retained_participant_id,
            )?,
            DispatchCapabilitySubjectV1::RetainedWorkerFork {
                source_participant_id,
            } => validate_narrowing_identity(
                "applies_to.source_participant_id",
                source_participant_id,
            )?,
            DispatchCapabilitySubjectV1::EphemeralTask
            | DispatchCapabilitySubjectV1::RetainedWorkerSpawn => {}
        }
        self.parent_policy_ref.validate()?;
        if let Some(reason) = &self.reason {
            validate_narrowing_identity("reason", reason)?;
        }
        Ok(())
    }
}

fn validate_narrowing_identity(name: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.trim() != value
        || value.contains('\0')
        || value.chars().any(char::is_control)
    {
        return Err(format!("{name} must be a nonempty canonical identity"));
    }
    Ok(())
}

#[cfg(test)]
mod e1_dispatch_policy_narrowing_schema_tests {
    use super::*;
    use serde_json::{json, Value};

    fn valid_carrier_json() -> Value {
        json!({
            "schema_version": 1,
            "request_id": "req_e1_0001",
            "orchestration_session_id": "session_e1_0001",
            "caller_participant_id": "participant_e1_caller",
            "target_backend_id": "codex",
            "target_world": {
                "world_id": "world_e1_0001",
                "world_generation": 7
            },
            "applies_to": { "kind": "ephemeral_task" },
            "parent_policy_ref": {
                "ref_id": "ao_0123456789abcdef0123456789abcdef",
                "object_kind": "policy",
                "schema_version": 1,
                "commitment": {
                    "kind": "CanonicalSha256",
                    "value": {
                        "digest_hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    }
                }
            },
            "parent_policy_revision": "policy-revision-e1-0001",
            "restricted_policy_patch": {
                "world_fs": {
                    "host_visible": false,
                    "read": { "allow_list": ["src/lib.rs"] }
                }
            },
            "reason": "restrict delegated read authority"
        })
    }

    #[test]
    fn dispatch_narrowing_v1_decodes_only_the_strict_world_fs_schema() {
        let decoded: DispatchPolicyNarrowingPatchV1 =
            serde_json::from_value(valid_carrier_json()).expect("strict carrier");
        decoded.validate().expect("valid carrier");

        for mutation in [
            ("schema_version", json!(2)),
            ("request_id", json!(" request")),
            ("parent_policy_revision", json!("")),
        ] {
            let mut value = valid_carrier_json();
            value[mutation.0] = mutation.1;
            assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(value).is_err());
        }

        let mut unknown = valid_carrier_json();
        unknown["unexpected"] = json!(true);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(unknown).is_err());

        let mut non_world_fs = valid_carrier_json();
        non_world_fs["restricted_policy_patch"]["net_allowed"] = json!(["example.com"]);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(non_world_fs).is_err());

        let mut nested_unknown = valid_carrier_json();
        nested_unknown["restricted_policy_patch"]["world_fs"]["read"]["unknown"] = json!(true);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(nested_unknown).is_err());

        let mut unsupported_glob_material = valid_carrier_json();
        unsupported_glob_material["restricted_policy_patch"]["world_fs"]["read"]["deny_list"] =
            json!(["src/[ab].rs"]);
        let decoded: DispatchPolicyNarrowingPatchV1 =
            serde_json::from_value(unsupported_glob_material).expect("transport is not authority");
        assert_eq!(decoded.schema_version, 1);
    }

    #[test]
    fn dispatch_narrowing_v1_rejects_missing_substituted_or_ambiguous_identity() {
        for key in [
            "request_id",
            "orchestration_session_id",
            "caller_participant_id",
            "target_backend_id",
            "target_world",
            "applies_to",
            "parent_policy_ref",
            "parent_policy_revision",
            "restricted_policy_patch",
        ] {
            let mut value = valid_carrier_json();
            value.as_object_mut().unwrap().remove(key);
            assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(value).is_err());
        }

        let mut wrong_kind = valid_carrier_json();
        wrong_kind["parent_policy_ref"]["object_kind"] = json!("resume_handle");
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(wrong_kind).is_err());

        let mut null_patch_field = valid_carrier_json();
        null_patch_field["restricted_policy_patch"]["world_fs"]["host_visible"] = Value::Null;
        assert!(
            serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(null_patch_field).is_err()
        );

        let mut zero_generation = valid_carrier_json();
        zero_generation["target_world"]["world_generation"] = json!(0);
        assert!(serde_json::from_value::<DispatchPolicyNarrowingPatchV1>(zero_generation).is_err());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallBootstrapContextV1 {
    pub selected_host_prefix: String,
    pub host_substrate_home: String,
    pub host_substrate_root: String,
    pub intended_host_principal: PlatformPrincipalV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallBootstrapContextCarrierV1 {
    pub context: InstallBootstrapContextV1,
    pub host_context_commitment: String,
}

/// Identity observed for one concrete Lima VM or registered WSL distribution.
///
/// ```
/// use transport_api_types::PlatformInstanceIdentityV1;
///
/// let instance = PlatformInstanceIdentityV1::Lima {
///     vm_name: "substrate".to_string(),
///     guest_machine_id: "0123456789abcdef0123456789abcdef".to_string(),
/// };
/// assert!(matches!(instance, PlatformInstanceIdentityV1::Lima { .. }));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlatformInstanceIdentityV1 {
    Lima {
        vm_name: String,
        guest_machine_id: String,
    },
    Wsl {
        distro_name: String,
        guest_machine_id: String,
    },
}

/// Host and guest transport endpoints observed for a platform instance.
///
/// ```
/// use transport_api_types::PlatformTransportIdentityV1;
///
/// let transport = PlatformTransportIdentityV1::Wsl {
///     pipe_path: r"\\.\pipe\substrate-agent".to_string(),
///     guest_socket: "/run/substrate.sock".to_string(),
/// };
/// assert!(matches!(transport, PlatformTransportIdentityV1::Wsl { .. }));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlatformTransportIdentityV1 {
    Lima {
        host_socket: String,
        guest_socket: String,
    },
    Wsl {
        pipe_path: String,
        guest_socket: String,
    },
}

/// Canonical host-to-platform bootstrap mapping bound to one host context.
///
/// ```
/// use transport_api_types::{
///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
///     PlatformBootstrapMappingV1,
/// };
///
/// let host = InstallBootstrapContextCarrierV1::from_context(
///     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
/// )?;
/// let mapping = PlatformBootstrapMappingV1::new_lima(
///     &host,
///     "substrate",
///     "0123456789abcdef0123456789abcdef",
///     "/Users/alice/.lima",
///     "/home/substrate/.substrate",
///     "substrate",
///     1000,
///     "/opt/substrate/sock/agent.sock",
///     "/run/substrate.sock",
/// )?;
/// assert_eq!(mapping.host_context_commitment, host.host_context_commitment);
/// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformBootstrapMappingV1 {
    pub host_context_commitment: String,
    pub platform_instance: PlatformInstanceIdentityV1,
    pub host_platform_control_root: String,
    pub realized_substrate_home: String,
    pub realized_principal: PlatformPrincipalV1,
    pub realized_transport: PlatformTransportIdentityV1,
}

/// Deterministic digest scoping shared Windows forwarder state.
///
/// ```
/// use transport_api_types::WindowsForwarderScopeV1;
///
/// let scope = WindowsForwarderScopeV1::derive(
///     "S-1-5-21-1000",
///     "Substrate-WSL",
///     "abcdef0123456789abcdef0123456789",
///     r"\\.\pipe\substrate-agent",
/// )?;
/// assert_eq!(scope.0.len(), 64);
/// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WindowsForwarderScopeV1(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InstallBootstrapContextErrorV1 {
    #[error("invalid install bootstrap path")]
    InvalidPath,
    #[error("invalid install bootstrap principal")]
    InvalidPrincipal,
    #[error("invalid install bootstrap context")]
    InvalidContext,
    #[error("invalid install bootstrap carrier")]
    InvalidCarrier,
    #[error("install bootstrap commitment mismatch")]
    CommitmentMismatch,
}

impl InstallBootstrapContextV1 {
    pub fn new_unix(
        selected_host_prefix: &str,
        account: &str,
        uid: u32,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        let selected_host_prefix = normalize_unix_install_bootstrap_path(selected_host_prefix)?;
        let context = Self {
            host_substrate_home: selected_host_prefix.clone(),
            host_substrate_root: selected_host_prefix.clone(),
            selected_host_prefix,
            intended_host_principal: PlatformPrincipalV1::Unix {
                account: account.to_string(),
                uid,
            },
        };
        context.validate()?;
        Ok(context)
    }

    pub fn new_windows(
        selected_host_prefix: &str,
        account: &str,
        sid: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        let selected_host_prefix = normalize_windows_install_bootstrap_path(selected_host_prefix)?;
        let context = Self {
            host_substrate_home: selected_host_prefix.clone(),
            host_substrate_root: selected_host_prefix.clone(),
            selected_host_prefix,
            intended_host_principal: PlatformPrincipalV1::Windows {
                account: account.to_string(),
                sid: sid.to_string(),
            },
        };
        context.validate()?;
        Ok(context)
    }

    pub fn validate(&self) -> Result<(), InstallBootstrapContextErrorV1> {
        if self.selected_host_prefix != self.host_substrate_home
            || self.selected_host_prefix != self.host_substrate_root
        {
            return Err(InstallBootstrapContextErrorV1::InvalidContext);
        }

        match &self.intended_host_principal {
            PlatformPrincipalV1::Unix { account, uid } => {
                if normalize_unix_install_bootstrap_path(&self.selected_host_prefix)?
                    != self.selected_host_prefix
                    || !valid_principal_text(account)
                    || *uid == 0
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
                }
            }
            PlatformPrincipalV1::Windows { account, sid } => {
                if normalize_windows_install_bootstrap_path(&self.selected_host_prefix)?
                    != self.selected_host_prefix
                    || !valid_principal_text(account)
                    || !valid_windows_sid(sid)
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
                }
            }
        }
        Ok(())
    }
}

impl InstallBootstrapContextCarrierV1 {
    pub fn from_context(
        context: InstallBootstrapContextV1,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        context.validate()?;
        let host_context_commitment = commitment_for_context(&context)?;
        Ok(Self {
            context,
            host_context_commitment,
        })
    }

    pub fn commitment_input(&self) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
        self.context.validate()?;
        commitment_input_for_context(&self.context)
    }

    pub fn validate(&self) -> Result<(), InstallBootstrapContextErrorV1> {
        self.context.validate()?;
        if !is_lower_hex_digest(&self.host_context_commitment) {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        if commitment_for_context(&self.context)? != self.host_context_commitment {
            return Err(InstallBootstrapContextErrorV1::CommitmentMismatch);
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<String, InstallBootstrapContextErrorV1> {
        self.validate()?;
        let mut record = self.commitment_input()?;
        record.extend_from_slice(b"host_context_commitment=");
        record.extend_from_slice(self.host_context_commitment.as_bytes());
        record.push(b'\n');
        Ok(URL_SAFE_NO_PAD.encode(record))
    }

    pub fn decode(encoded: &str) -> Result<Self, InstallBootstrapContextErrorV1> {
        if encoded.is_empty()
            || encoded
                .bytes()
                .any(|byte| !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'-')
        {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        if URL_SAFE_NO_PAD.encode(&bytes) != encoded {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let record = std::str::from_utf8(&bytes)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        parse_install_bootstrap_record(record)
    }
}

impl PlatformBootstrapMappingV1 {
    /// Constructs a canonical Lima mapping from already-observed values.
    ///
    /// ```
    /// use transport_api_types::{
    ///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    ///     PlatformBootstrapMappingV1,
    /// };
    ///
    /// let host = InstallBootstrapContextCarrierV1::from_context(
    ///     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// )?;
    /// let mapping = PlatformBootstrapMappingV1::new_lima(
    ///     &host,
    ///     "substrate",
    ///     "0123456789abcdef0123456789abcdef",
    ///     "/Users/alice/.lima",
    ///     "/home/substrate/.substrate",
    ///     "substrate",
    ///     1000,
    ///     "/opt/substrate/sock/agent.sock",
    ///     "/run/substrate.sock",
    /// )?;
    /// assert_eq!(mapping.host_platform_control_root, "/Users/alice/.lima");
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new_lima(
        host_carrier: &InstallBootstrapContextCarrierV1,
        vm_name: &str,
        guest_machine_id: &str,
        host_platform_control_root: &str,
        realized_substrate_home: &str,
        realized_principal_account: &str,
        realized_principal_uid: u32,
        transport_host: &str,
        transport_guest_socket: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        let mapping = Self {
            host_context_commitment: host_carrier.host_context_commitment.clone(),
            platform_instance: PlatformInstanceIdentityV1::Lima {
                vm_name: vm_name.to_string(),
                guest_machine_id: guest_machine_id.to_string(),
            },
            host_platform_control_root: normalize_unix_install_bootstrap_path(
                host_platform_control_root,
            )?,
            realized_substrate_home: normalize_unix_install_bootstrap_path(
                realized_substrate_home,
            )?,
            realized_principal: PlatformPrincipalV1::Unix {
                account: realized_principal_account.to_string(),
                uid: realized_principal_uid,
            },
            realized_transport: PlatformTransportIdentityV1::Lima {
                host_socket: normalize_unix_install_bootstrap_path(transport_host)?,
                guest_socket: normalize_unix_install_bootstrap_path(transport_guest_socket)?,
            },
        };
        mapping.validate(host_carrier)?;
        Ok(mapping)
    }

    /// Constructs a canonical WSL mapping from already-observed values.
    ///
    /// ```
    /// use transport_api_types::{
    ///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    ///     PlatformBootstrapMappingV1,
    /// };
    ///
    /// let host = InstallBootstrapContextCarrierV1::from_context(
    ///     InstallBootstrapContextV1::new_windows(
    ///         r"C:\Substrate",
    ///         r"ACME\Alice",
    ///         "S-1-5-21-1000",
    ///     )?,
    /// )?;
    /// let mapping = PlatformBootstrapMappingV1::new_wsl(
    ///     &host,
    ///     "Substrate-WSL",
    ///     "abcdef0123456789abcdef0123456789",
    ///     r"C:\Users\Alice\AppData\Local\Substrate\forwarder\scope",
    ///     "/home/substrate/.substrate",
    ///     "substrate",
    ///     1000,
    ///     r"\\.\pipe\Substrate-Agent",
    ///     "/run/substrate.sock",
    /// )?;
    /// assert_eq!(
    ///     mapping.host_platform_control_root,
    ///     r"C:\Users\Alice\AppData\Local\Substrate\forwarder\scope"
    /// );
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new_wsl(
        host_carrier: &InstallBootstrapContextCarrierV1,
        distro_name: &str,
        guest_machine_id: &str,
        host_platform_control_root: &str,
        realized_substrate_home: &str,
        realized_principal_account: &str,
        realized_principal_uid: u32,
        transport_host: &str,
        transport_guest_socket: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        let mapping = Self {
            host_context_commitment: host_carrier.host_context_commitment.clone(),
            platform_instance: PlatformInstanceIdentityV1::Wsl {
                distro_name: distro_name.to_string(),
                guest_machine_id: guest_machine_id.to_string(),
            },
            host_platform_control_root: normalize_windows_install_bootstrap_path(
                host_platform_control_root,
            )?,
            realized_substrate_home: normalize_unix_install_bootstrap_path(
                realized_substrate_home,
            )?,
            realized_principal: PlatformPrincipalV1::Unix {
                account: realized_principal_account.to_string(),
                uid: realized_principal_uid,
            },
            realized_transport: PlatformTransportIdentityV1::Wsl {
                pipe_path: normalize_windows_pipe_path(transport_host)?,
                guest_socket: normalize_unix_install_bootstrap_path(transport_guest_socket)?,
            },
        };
        mapping.validate(host_carrier)?;
        Ok(mapping)
    }

    /// Validates canonical fields and their binding to the supplied host carrier.
    ///
    /// ```
    /// use transport_api_types::{
    ///     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    ///     PlatformBootstrapMappingV1,
    /// };
    /// # let host = InstallBootstrapContextCarrierV1::from_context(
    /// #     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// # )?;
    /// # let mapping = PlatformBootstrapMappingV1::new_lima(
    /// #     &host, "substrate", "0123456789abcdef0123456789abcdef",
    /// #     "/Users/alice/.lima", "/home/substrate/.substrate", "substrate", 1000,
    /// #     "/opt/substrate/sock/agent.sock", "/run/substrate.sock",
    /// # )?;
    /// mapping.validate(&host)?;
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn validate(
        &self,
        host_carrier: &InstallBootstrapContextCarrierV1,
    ) -> Result<(), InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        if !is_lower_hex_digest(&self.host_context_commitment) {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        if self.host_context_commitment != host_carrier.host_context_commitment {
            return Err(InstallBootstrapContextErrorV1::CommitmentMismatch);
        }

        let PlatformPrincipalV1::Unix { account, .. } = &self.realized_principal else {
            return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
        };
        if !valid_principal_text(account) {
            return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
        }

        match (&self.platform_instance, &self.realized_transport) {
            (
                PlatformInstanceIdentityV1::Lima {
                    vm_name,
                    guest_machine_id,
                },
                PlatformTransportIdentityV1::Lima {
                    host_socket,
                    guest_socket,
                },
            ) => {
                if !valid_principal_text(vm_name)
                    || !is_lower_hex_machine_id(guest_machine_id)
                    || normalize_unix_install_bootstrap_path(&self.host_platform_control_root)?
                        != self.host_platform_control_root
                    || normalize_unix_install_bootstrap_path(&self.realized_substrate_home)?
                        != self.realized_substrate_home
                    || normalize_unix_install_bootstrap_path(host_socket)? != *host_socket
                    || normalize_unix_install_bootstrap_path(guest_socket)? != *guest_socket
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidContext);
                }
            }
            (
                PlatformInstanceIdentityV1::Wsl {
                    distro_name,
                    guest_machine_id,
                },
                PlatformTransportIdentityV1::Wsl {
                    pipe_path,
                    guest_socket,
                },
            ) => {
                if !valid_principal_text(distro_name)
                    || !is_lower_hex_machine_id(guest_machine_id)
                    || normalize_windows_install_bootstrap_path(&self.host_platform_control_root)?
                        != self.host_platform_control_root
                    || normalize_unix_install_bootstrap_path(&self.realized_substrate_home)?
                        != self.realized_substrate_home
                    || normalize_windows_pipe_path(pipe_path)? != *pipe_path
                    || normalize_unix_install_bootstrap_path(guest_socket)? != *guest_socket
                {
                    return Err(InstallBootstrapContextErrorV1::InvalidContext);
                }
            }
            _ => return Err(InstallBootstrapContextErrorV1::InvalidContext),
        }
        Ok(())
    }

    /// Encodes the exact canonical thirteen-line mapping record as unpadded base64url.
    ///
    /// ```
    /// # use transport_api_types::{
    /// #     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    /// #     PlatformBootstrapMappingV1,
    /// # };
    /// # let host = InstallBootstrapContextCarrierV1::from_context(
    /// #     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// # )?;
    /// # let mapping = PlatformBootstrapMappingV1::new_lima(
    /// #     &host, "substrate", "0123456789abcdef0123456789abcdef",
    /// #     "/Users/alice/.lima", "/home/substrate/.substrate", "substrate", 1000,
    /// #     "/opt/substrate/sock/agent.sock", "/run/substrate.sock",
    /// # )?;
    /// let encoded = mapping.encode(&host)?;
    /// assert!(!encoded.contains('='));
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn encode(
        &self,
        host_carrier: &InstallBootstrapContextCarrierV1,
    ) -> Result<String, InstallBootstrapContextErrorV1> {
        self.validate(host_carrier)?;
        Ok(URL_SAFE_NO_PAD.encode(platform_bootstrap_mapping_record(self)?))
    }

    /// Decodes and canonically re-encodes a mapping bound to the supplied host carrier.
    ///
    /// ```
    /// # use transport_api_types::{
    /// #     InstallBootstrapContextCarrierV1, InstallBootstrapContextV1,
    /// #     PlatformBootstrapMappingV1,
    /// # };
    /// # let host = InstallBootstrapContextCarrierV1::from_context(
    /// #     InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000)?,
    /// # )?;
    /// # let mapping = PlatformBootstrapMappingV1::new_lima(
    /// #     &host, "substrate", "0123456789abcdef0123456789abcdef",
    /// #     "/Users/alice/.lima", "/home/substrate/.substrate", "substrate", 1000,
    /// #     "/opt/substrate/sock/agent.sock", "/run/substrate.sock",
    /// # )?;
    /// # let encoded = mapping.encode(&host)?;
    /// let decoded = PlatformBootstrapMappingV1::decode(&encoded, &host)?;
    /// assert_eq!(decoded, mapping);
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn decode(
        encoded: &str,
        host_carrier: &InstallBootstrapContextCarrierV1,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        host_carrier.validate()?;
        if !valid_unpadded_base64url(encoded) {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        if URL_SAFE_NO_PAD.encode(&bytes) != encoded {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        let record = std::str::from_utf8(&bytes)
            .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
        let mapping = parse_platform_bootstrap_mapping_record(record, host_carrier)?;
        if mapping.encode(host_carrier)? != encoded {
            return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
        }
        Ok(mapping)
    }
}

impl WindowsForwarderScopeV1 {
    /// Derives the canonical six-line Windows forwarder scope digest.
    ///
    /// ```
    /// use transport_api_types::WindowsForwarderScopeV1;
    ///
    /// let scope = WindowsForwarderScopeV1::derive(
    ///     "S-1-5-21-1000",
    ///     "Substrate-WSL",
    ///     "abcdef0123456789abcdef0123456789",
    ///     r"\\.\pipe\Substrate-Agent",
    /// )?;
    /// assert_eq!(
    ///     scope.0,
    ///     "3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a"
    /// );
    /// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
    /// ```
    pub fn derive(
        windows_sid: &str,
        distro_name: &str,
        guest_machine_id: &str,
        pipe_path: &str,
    ) -> Result<Self, InstallBootstrapContextErrorV1> {
        let digest = Sha256::digest(windows_forwarder_scope_input(
            windows_sid,
            distro_name,
            guest_machine_id,
            pipe_path,
        )?);
        Ok(Self(format!("{digest:x}")))
    }
}

/// Normalizes the case-insensitive name in a canonical Windows named-pipe path.
///
/// ```
/// use transport_api_types::normalize_windows_pipe_path;
///
/// assert_eq!(
///     normalize_windows_pipe_path(r"\\.\pipe\Substrate-Agent")?,
///     r"\\.\pipe\substrate-agent"
/// );
/// # Ok::<(), transport_api_types::InstallBootstrapContextErrorV1>(())
/// ```
pub fn normalize_windows_pipe_path(raw: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    let Some(name) = raw.strip_prefix(r"\\.\pipe\") else {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    };
    if name.is_empty()
        || name.len() > 128
        || !name.is_ascii()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(format!(r"\\.\pipe\{}", name.to_ascii_lowercase()))
}

pub fn normalize_unix_install_bootstrap_path(
    raw: &str,
) -> Result<String, InstallBootstrapContextErrorV1> {
    if raw.is_empty()
        || raw == "/"
        || !raw.starts_with('/')
        || raw.starts_with("//")
        || raw.contains('\0')
    {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }

    let mut components = Vec::new();
    for component in raw[1..].split('/') {
        if component.is_empty() {
            continue;
        }
        if component == "." || component == ".." {
            return Err(InstallBootstrapContextErrorV1::InvalidPath);
        }
        components.push(component);
    }
    if components.is_empty() {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(format!("/{}", components.join("/")))
}

pub fn normalize_windows_install_bootstrap_path(
    raw: &str,
) -> Result<String, InstallBootstrapContextErrorV1> {
    if raw.is_empty() || raw.contains('\0') {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    let normalized_separators = raw.replace('/', "\\");
    let lower = normalized_separators.to_ascii_lowercase();
    if lower.starts_with(r"\\?\") || lower.starts_with(r"\\.\") {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }

    let normalized = if normalized_separators.starts_with(r"\\") {
        normalize_windows_unc_path(&normalized_separators)?
    } else {
        normalize_windows_drive_path(&normalized_separators)?
    };
    Ok(normalized)
}

fn normalize_windows_drive_path(raw: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    let bytes = raw.as_bytes();
    if bytes.len() < 4 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' || bytes[2] != b'\\' {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    let components = windows_components(&raw[3..])?;
    let drive = (bytes[0] as char).to_ascii_uppercase();
    Ok(format!("{drive}:\\{}", components.join("\\")))
}

fn normalize_windows_unc_path(raw: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    let components = windows_components(&raw[2..])?;
    if components.len() < 3 {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(format!(r"\\{}", components.join("\\")))
}

fn windows_components(raw: &str) -> Result<Vec<&str>, InstallBootstrapContextErrorV1> {
    let components = raw
        .split('\\')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| !valid_windows_component(component))
    {
        return Err(InstallBootstrapContextErrorV1::InvalidPath);
    }
    Ok(components)
}

fn valid_windows_component(component: &str) -> bool {
    if component == "."
        || component == ".."
        || component.ends_with('.')
        || component.ends_with(' ')
        || component.chars().any(|ch| {
            ch == '\0'
                || ('\u{1}'..='\u{1f}').contains(&ch)
                || matches!(ch, '<' | '>' | '"' | '|' | '?' | '*' | ':')
        })
    {
        return false;
    }
    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .to_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !reserved_numbered_windows_name(&stem, "COM")
        && !reserved_numbered_windows_name(&stem, "LPT")
}

fn reserved_numbered_windows_name(stem: &str, prefix: &str) -> bool {
    let Some(suffix) = stem.strip_prefix(prefix) else {
        return false;
    };
    matches!(
        suffix,
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
    )
}

fn valid_principal_text(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(|ch| matches!(ch, '\0' | '\n' | '\r'))
}

fn valid_windows_sid(sid: &str) -> bool {
    let mut parts = sid.split('-');
    if parts.next() != Some("S") {
        return false;
    }
    let numeric = parts.collect::<Vec<_>>();
    numeric.len() >= 2
        && numeric.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (*part == "0" || !part.starts_with('0'))
        })
}

fn commitment_input_for_context(
    context: &InstallBootstrapContextV1,
) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
    context.validate()?;
    let selected = encode_inner_field(&context.selected_host_prefix);
    let home = encode_inner_field(&context.host_substrate_home);
    let root = encode_inner_field(&context.host_substrate_root);
    let principal = match &context.intended_host_principal {
        PlatformPrincipalV1::Unix { account, uid } => format!(
            "principal_kind=unix\nprincipal_account={}\nprincipal_uid={}\n",
            encode_inner_field(account),
            uid
        ),
        PlatformPrincipalV1::Windows { account, sid } => format!(
            "principal_kind=windows\nprincipal_account={}\nprincipal_sid={}\n",
            encode_inner_field(account),
            encode_inner_field(sid)
        ),
    };
    Ok(format!(
        "domain={INSTALL_BOOTSTRAP_CONTEXT_DOMAIN_V1}\nversion=1\nselected_host_prefix={selected}\nhost_substrate_home={home}\nhost_substrate_root={root}\n{principal}"
    )
    .into_bytes())
}

fn commitment_for_context(
    context: &InstallBootstrapContextV1,
) -> Result<String, InstallBootstrapContextErrorV1> {
    let digest = Sha256::digest(commitment_input_for_context(context)?);
    Ok(format!("{digest:x}"))
}

fn encode_inner_field(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(value.as_bytes())
}

fn decode_inner_field(value: &str) -> Result<String, InstallBootstrapContextErrorV1> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'-')
    {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
    if URL_SAFE_NO_PAD.encode(&decoded) != value {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    String::from_utf8(decoded).map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)
}

fn valid_unpadded_base64url(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn platform_bootstrap_mapping_record(
    mapping: &PlatformBootstrapMappingV1,
) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
    let (platform_kind, instance_name, guest_machine_id) = match &mapping.platform_instance {
        PlatformInstanceIdentityV1::Lima {
            vm_name,
            guest_machine_id,
        } => ("lima", vm_name, guest_machine_id),
        PlatformInstanceIdentityV1::Wsl {
            distro_name,
            guest_machine_id,
        } => ("wsl", distro_name, guest_machine_id),
    };
    let (transport_kind, transport_host, transport_guest_socket) = match &mapping.realized_transport
    {
        PlatformTransportIdentityV1::Lima {
            host_socket,
            guest_socket,
        } => ("lima", host_socket, guest_socket),
        PlatformTransportIdentityV1::Wsl {
            pipe_path,
            guest_socket,
        } => ("wsl", pipe_path, guest_socket),
    };
    let PlatformPrincipalV1::Unix { account, uid } = &mapping.realized_principal else {
        return Err(InstallBootstrapContextErrorV1::InvalidPrincipal);
    };
    Ok(format!(
        "domain={PLATFORM_BOOTSTRAP_MAPPING_DOMAIN_V1}\n\
version=1\n\
host_context_commitment={}\n\
platform_kind={platform_kind}\n\
instance_name={}\n\
guest_machine_id={guest_machine_id}\n\
host_platform_control_root={}\n\
realized_substrate_home={}\n\
realized_principal_account={}\n\
realized_principal_uid={uid}\n\
transport_kind={transport_kind}\n\
transport_host={}\n\
transport_guest_socket={}\n",
        mapping.host_context_commitment,
        encode_inner_field(instance_name),
        encode_inner_field(&mapping.host_platform_control_root),
        encode_inner_field(&mapping.realized_substrate_home),
        encode_inner_field(account),
        encode_inner_field(transport_host),
        encode_inner_field(transport_guest_socket),
    )
    .into_bytes())
}

fn parse_platform_bootstrap_mapping_record(
    record: &str,
    host_carrier: &InstallBootstrapContextCarrierV1,
) -> Result<PlatformBootstrapMappingV1, InstallBootstrapContextErrorV1> {
    if !record.ends_with('\n') || record.contains('\r') || record.contains('\0') {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let lines = record.split_terminator('\n').collect::<Vec<_>>();
    if lines.len() != 13 {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    require_record_value(lines[0], "domain", PLATFORM_BOOTSTRAP_MAPPING_DOMAIN_V1)?;
    require_record_value(lines[1], "version", "1")?;
    let host_context_commitment = record_value(lines[2], "host_context_commitment")?.to_string();
    if !is_lower_hex_digest(&host_context_commitment) {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    if host_context_commitment != host_carrier.host_context_commitment {
        return Err(InstallBootstrapContextErrorV1::CommitmentMismatch);
    }

    let platform_kind = record_value(lines[3], "platform_kind")?;
    let instance_name = decode_inner_field(record_value(lines[4], "instance_name")?)?;
    let guest_machine_id = record_value(lines[5], "guest_machine_id")?;
    if !is_lower_hex_machine_id(guest_machine_id) {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let host_platform_control_root =
        decode_inner_field(record_value(lines[6], "host_platform_control_root")?)?;
    let realized_substrate_home =
        decode_inner_field(record_value(lines[7], "realized_substrate_home")?)?;
    let realized_principal_account =
        decode_inner_field(record_value(lines[8], "realized_principal_account")?)?;
    let realized_principal_uid =
        parse_canonical_u32(record_value(lines[9], "realized_principal_uid")?)?;
    let transport_kind = record_value(lines[10], "transport_kind")?;
    let transport_host = decode_inner_field(record_value(lines[11], "transport_host")?)?;
    let transport_guest_socket =
        decode_inner_field(record_value(lines[12], "transport_guest_socket")?)?;

    match (platform_kind, transport_kind) {
        ("lima", "lima") => PlatformBootstrapMappingV1::new_lima(
            host_carrier,
            &instance_name,
            guest_machine_id,
            &host_platform_control_root,
            &realized_substrate_home,
            &realized_principal_account,
            realized_principal_uid,
            &transport_host,
            &transport_guest_socket,
        ),
        ("wsl", "wsl") => PlatformBootstrapMappingV1::new_wsl(
            host_carrier,
            &instance_name,
            guest_machine_id,
            &host_platform_control_root,
            &realized_substrate_home,
            &realized_principal_account,
            realized_principal_uid,
            &transport_host,
            &transport_guest_socket,
        ),
        _ => Err(InstallBootstrapContextErrorV1::InvalidCarrier),
    }
}

fn parse_canonical_u32(value: &str) -> Result<u32, InstallBootstrapContextErrorV1> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    value
        .parse()
        .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)
}

fn is_lower_hex_machine_id(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn windows_forwarder_scope_input(
    windows_sid: &str,
    distro_name: &str,
    guest_machine_id: &str,
    pipe_path: &str,
) -> Result<Vec<u8>, InstallBootstrapContextErrorV1> {
    if !valid_windows_sid(windows_sid)
        || !valid_principal_text(distro_name)
        || !is_lower_hex_machine_id(guest_machine_id)
    {
        return Err(InstallBootstrapContextErrorV1::InvalidContext);
    }
    let pipe_path = normalize_windows_pipe_path(pipe_path)?;
    Ok(format!(
        "domain={WINDOWS_FORWARDER_SCOPE_DOMAIN_V1}\n\
version=1\n\
windows_sid={}\n\
distro_name={}\n\
guest_machine_id={guest_machine_id}\n\
pipe_path={}\n",
        encode_inner_field(windows_sid),
        encode_inner_field(distro_name),
        encode_inner_field(&pipe_path),
    )
    .into_bytes())
}

fn parse_install_bootstrap_record(
    record: &str,
) -> Result<InstallBootstrapContextCarrierV1, InstallBootstrapContextErrorV1> {
    if !record.ends_with('\n') || record.contains('\r') || record.contains('\0') {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let lines = record.split_terminator('\n').collect::<Vec<_>>();
    if lines.len() != 9 {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    require_record_value(lines[0], "domain", INSTALL_BOOTSTRAP_CONTEXT_DOMAIN_V1)?;
    require_record_value(lines[1], "version", "1")?;
    let selected_host_prefix = decode_inner_field(record_value(lines[2], "selected_host_prefix")?)?;
    let host_substrate_home = decode_inner_field(record_value(lines[3], "host_substrate_home")?)?;
    let host_substrate_root = decode_inner_field(record_value(lines[4], "host_substrate_root")?)?;
    let principal_kind = record_value(lines[5], "principal_kind")?;
    let account = decode_inner_field(record_value(lines[6], "principal_account")?)?;
    let intended_host_principal = match principal_kind {
        "unix" => {
            let raw_uid = record_value(lines[7], "principal_uid")?;
            if raw_uid.is_empty()
                || !raw_uid.bytes().all(|byte| byte.is_ascii_digit())
                || (raw_uid.len() > 1 && raw_uid.starts_with('0'))
            {
                return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
            }
            let uid = raw_uid
                .parse::<u32>()
                .map_err(|_| InstallBootstrapContextErrorV1::InvalidCarrier)?;
            PlatformPrincipalV1::Unix { account, uid }
        }
        "windows" => PlatformPrincipalV1::Windows {
            account,
            sid: decode_inner_field(record_value(lines[7], "principal_sid")?)?,
        },
        _ => return Err(InstallBootstrapContextErrorV1::InvalidCarrier),
    };
    let host_context_commitment = record_value(lines[8], "host_context_commitment")?.to_string();
    if !is_lower_hex_digest(&host_context_commitment) {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    let carrier = InstallBootstrapContextCarrierV1 {
        context: InstallBootstrapContextV1 {
            selected_host_prefix,
            host_substrate_home,
            host_substrate_root,
            intended_host_principal,
        },
        host_context_commitment,
    };
    carrier.validate()?;
    Ok(carrier)
}

fn record_value<'a>(
    line: &'a str,
    expected_key: &str,
) -> Result<&'a str, InstallBootstrapContextErrorV1> {
    let (key, value) = line
        .split_once('=')
        .ok_or(InstallBootstrapContextErrorV1::InvalidCarrier)?;
    if key != expected_key || value.contains('=') {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    Ok(value)
}

fn require_record_value(
    line: &str,
    key: &str,
    expected_value: &str,
) -> Result<(), InstallBootstrapContextErrorV1> {
    if record_value(line, key)? != expected_value {
        return Err(InstallBootstrapContextErrorV1::InvalidCarrier);
    }
    Ok(())
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicySnapshotWorldFsIsolationV2 {
    Workspace,
    Full,
}

fn default_true() -> bool {
    true
}

fn default_allow_list_dot() -> Vec<String> {
    vec![".".to_string()]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsEnforcementV2 {
    Strict,
    BestEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsDimensionV2 {
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsV2 {
    pub mode: WorldFsMode,
    pub isolation: PolicySnapshotWorldFsIsolationV2,
    pub require_world: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<WorldFsEnforcementV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discover: Option<PolicySnapshotWorldFsDimensionV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<PolicySnapshotWorldFsDimensionV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write: Option<PolicySnapshotWorldFsDimensionV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotLimitsV2 {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u32,
    pub max_runtime_ms: u64,
    pub max_egress_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotV2 {
    pub schema_version: u32,
    pub world_fs: PolicySnapshotWorldFsV2,
    pub net_allowed: Vec<String>,
    pub limits: PolicySnapshotLimitsV2,
}

impl PolicySnapshotV2 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 2 {
            return Err(format!(
                "unsupported policy_snapshot.schema_version: {} (expected 2)",
                self.schema_version
            ));
        }
        validate_world_fs_snapshot(&self.world_fs)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsDenyEnforcementV3 {
    Strict,
    PreferStrict,
    Weak,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsFailClosedV3 {
    #[serde(default)]
    pub routing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsDimensionV3 {
    #[serde(default = "default_allow_list_dot")]
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsWriteV3 {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_allow_list_dot")]
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
}

impl Default for PolicySnapshotWorldFsWriteV3 {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            allow_list: default_allow_list_dot(),
            deny_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotWorldFsV3 {
    #[serde(default = "default_true")]
    pub host_visible: bool,
    #[serde(default)]
    pub fail_closed: PolicySnapshotWorldFsFailClosedV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deny_enforcement: Option<WorldFsDenyEnforcementV3>,
    #[serde(default)]
    pub caged_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discover: Option<PolicySnapshotWorldFsDimensionV3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<PolicySnapshotWorldFsDimensionV3>,
    #[serde(default)]
    pub write: PolicySnapshotWorldFsWriteV3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySnapshotV3 {
    pub schema_version: u32,
    #[serde(default)]
    pub net_allowed: Vec<String>,
    pub world_fs: PolicySnapshotWorldFsV3,
}

impl PolicySnapshotV3 {
    pub fn canonicalize(&self) -> Result<Self, String> {
        if self.schema_version != 3 {
            return Err(format!(
                "unsupported policy_snapshot.schema_version: {} (expected 3)",
                self.schema_version
            ));
        }

        let mut snapshot = self.clone();
        snapshot.net_allowed = canonicalize_net_allowed(&snapshot.net_allowed);

        if snapshot.world_fs.read.is_none() {
            snapshot.world_fs.read = Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: default_allow_list_dot(),
                deny_list: Vec::new(),
            });
        }

        let read_clone =
            snapshot
                .world_fs
                .read
                .clone()
                .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: default_allow_list_dot(),
                    deny_list: Vec::new(),
                });

        if snapshot.world_fs.discover.is_none() {
            snapshot.world_fs.discover = Some(read_clone.clone());
        }

        if snapshot.world_fs.write.allow_list.is_empty() {
            snapshot.world_fs.write.allow_list = default_allow_list_dot();
        }

        normalize_and_validate_world_fs_snapshot_v3(&mut snapshot.world_fs)?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> Result<(), String> {
        let _ = self.canonicalize()?;
        Ok(())
    }

    pub fn resolve_world_network_routing(
        &self,
        world_net_filter: bool,
    ) -> Result<WorldNetworkRoutingV1, String> {
        let snapshot = self.canonicalize()?;
        let restrictive = snapshot.net_allowed.as_slice() != ["*"];
        let isolate_network = world_net_filter && restrictive;

        if isolate_network {
            validate_net_allowed_for_enforcement(&snapshot.net_allowed)?;
        }

        Ok(WorldNetworkRoutingV1 {
            isolate_network,
            allowed_domains: if isolate_network {
                snapshot.net_allowed
            } else {
                Vec::new()
            },
        })
    }
}

pub fn canonicalize_net_allowed(entries: &[String]) -> Vec<String> {
    let mut canonical = Vec::with_capacity(entries.len());

    for raw in entries {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut normalized = trimmed.to_ascii_lowercase();
        normalized.truncate(normalized.trim_end_matches('.').len());
        if normalized.is_empty() {
            continue;
        }

        if normalized.starts_with('[') && normalized.ends_with(']') {
            let inner = &normalized[1..normalized.len() - 1];
            if matches!(inner.parse::<IpAddr>(), Ok(IpAddr::V6(_))) {
                normalized = inner.to_string();
            }
        }

        if normalized == "*" {
            return vec!["*".to_string()];
        }

        if !canonical.contains(&normalized) {
            canonical.push(normalized);
        }
    }

    canonical
}

pub fn validate_net_allowed_for_enforcement(entries: &[String]) -> Result<(), String> {
    let canonical = canonicalize_net_allowed(entries);

    for entry in &canonical {
        validate_net_allowed_entry_for_enforcement(entry)?;
    }

    Ok(())
}

fn validate_net_allowed_entry_for_enforcement(entry: &str) -> Result<(), String> {
    if entry == "*" {
        return Ok(());
    }

    if !entry.is_ascii() {
        return Err(
            "net_allowed entries must be ASCII; use punycode A-labels for IDNs".to_string(),
        );
    }

    if entry.contains("://") {
        return Err("net_allowed entries must not include URL schemes".to_string());
    }
    if entry.contains('/') {
        return Err("net_allowed entries must not include paths".to_string());
    }
    if entry.contains('?') {
        return Err("net_allowed entries must not include query strings".to_string());
    }
    if entry.contains('#') {
        return Err("net_allowed entries must not include URL fragments".to_string());
    }
    if entry.contains(['*', '[', ']']) {
        return Err("net_allowed wildcard forms other than '*' are not supported".to_string());
    }

    if entry.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    if entry.contains(':') {
        return Err(
            "net_allowed entries must be hostnames or IP literals without ports".to_string(),
        );
    }

    validate_hostname(entry)
}

fn validate_hostname(entry: &str) -> Result<(), String> {
    if entry.starts_with('.') || entry.ends_with('.') {
        return Err("net_allowed hostnames must not start or end with '.'".to_string());
    }

    for label in entry.split('.') {
        if label.is_empty() {
            return Err("net_allowed hostnames must not contain empty labels".to_string());
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err("net_allowed hostname labels must not start or end with '-'".to_string());
        }
        if !label
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        {
            return Err(
                "net_allowed hostnames may contain only ASCII letters, digits, '-' and '.'"
                    .to_string(),
            );
        }
    }

    Ok(())
}

fn normalize_project_pattern(raw: &str) -> Result<String, String> {
    let mut pattern = raw.trim();
    if pattern.is_empty() {
        return Err("pattern must be non-empty".to_string());
    }
    if pattern.starts_with('/') {
        return Err("absolute paths are not allowed".to_string());
    }

    while let Some(stripped) = pattern.strip_prefix("./") {
        pattern = stripped;
    }

    let mut normalized = pattern.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        normalized = ".".to_string();
    }

    if normalized.split('/').any(|segment| segment == "..") {
        return Err("path segments must not be '..'".to_string());
    }

    Ok(normalized)
}

fn contains_any_glob_metacharacters(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[') || value.contains(']')
}

fn contains_unsupported_deny_metacharacters(value: &str) -> bool {
    value.contains('?') || value.contains('[') || value.contains(']')
}

fn validate_deny_wildcards(pattern: &str) -> Result<(), String> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            continue;
        }
        if run > 0 && run != 1 && run != 2 {
            return Err("deny_list wildcard runs must be '*' or '**' (no '***' or longer)".into());
        }
        run = 0;
    }
    if run > 0 && run != 1 && run != 2 {
        return Err("deny_list wildcard runs must be '*' or '**' (no '***' or longer)".into());
    }
    Ok(())
}

fn validate_dimension(prefix: &str, dim: &PolicySnapshotWorldFsDimensionV2) -> Result<(), String> {
    if dim.allow_list.is_empty() {
        return Err(format!("{prefix}.allow_list must be non-empty"));
    }

    for raw in &dim.allow_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| format!("{prefix}.allow_list: {e}"))?;
        if contains_any_glob_metacharacters(&normalized) {
            return Err(format!(
                "{prefix}.allow_list contains glob metacharacters; wildcards are not supported in allow_list"
            ));
        }
    }

    for raw in &dim.deny_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        if contains_unsupported_deny_metacharacters(&normalized) {
            return Err(format!(
                "{prefix}.deny_list contains unsupported glob metacharacters ('?' or character classes)"
            ));
        }
        validate_deny_wildcards(&normalized).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
    }

    Ok(())
}

fn validate_world_fs_snapshot(world_fs: &PolicySnapshotWorldFsV2) -> Result<(), String> {
    match world_fs.isolation {
        PolicySnapshotWorldFsIsolationV2::Workspace => {
            if world_fs.enforcement.is_some() {
                return Err(
                    "world_fs.enforcement must be omitted when world_fs.isolation=workspace"
                        .to_string(),
                );
            }
            if world_fs.discover.is_some() {
                return Err(
                    "world_fs.discover must be omitted when world_fs.isolation=workspace"
                        .to_string(),
                );
            }
            if world_fs.read.is_some() {
                return Err(
                    "world_fs.read must be omitted when world_fs.isolation=workspace".to_string(),
                );
            }
            if world_fs.write.is_some() {
                return Err(
                    "world_fs.write must be omitted when world_fs.isolation=workspace".to_string(),
                );
            }
            Ok(())
        }
        PolicySnapshotWorldFsIsolationV2::Full => {
            let read = world_fs.read.as_ref().ok_or_else(|| {
                "world_fs.read must be present when world_fs.isolation=full".to_string()
            })?;
            validate_dimension("world_fs.read", read)?;

            if let Some(discover) = world_fs.discover.as_ref() {
                validate_dimension("world_fs.discover", discover)?;
            }

            match world_fs.mode {
                WorldFsMode::ReadOnly => {
                    if world_fs.write.is_some() {
                        return Err(
                            "world_fs.write must be omitted when world_fs.mode=read_only"
                                .to_string(),
                        );
                    }
                }
                WorldFsMode::Writable => {
                    let write = world_fs.write.as_ref().ok_or_else(|| {
                        "world_fs.write must be present when world_fs.mode=writable".to_string()
                    })?;
                    validate_dimension("world_fs.write", write)?;
                }
            }

            let any_deny = world_fs
                .read
                .as_ref()
                .is_some_and(|d| !d.deny_list.is_empty())
                || world_fs
                    .discover
                    .as_ref()
                    .is_some_and(|d| !d.deny_list.is_empty())
                || world_fs
                    .write
                    .as_ref()
                    .is_some_and(|d| !d.deny_list.is_empty());

            if any_deny {
                if world_fs.enforcement.is_none() {
                    return Err(
                        "world_fs.enforcement must be present when any deny_list is non-empty"
                            .to_string(),
                    );
                }
                if !world_fs.require_world {
                    return Err("deny_list requires world_fs.require_world=true".to_string());
                }
            } else if world_fs.enforcement.is_some() {
                return Err(
                    "world_fs.enforcement is only valid when at least one deny_list is non-empty"
                        .to_string(),
                );
            }

            Ok(())
        }
    }
}

fn normalize_project_pattern_v3(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("pattern must not be empty".to_string());
    }

    if raw.starts_with('/') {
        return Err("absolute patterns are invalid".to_string());
    }

    let mut segments: Vec<&str> = Vec::new();
    for seg in raw.split('/') {
        let seg = seg.trim();
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err("pattern must not contain '..' segments".to_string());
        }
        segments.push(seg);
    }

    if segments.is_empty() {
        Ok(".".to_string())
    } else {
        Ok(segments.join("/"))
    }
}

fn validate_deny_wildcards_v3(pattern: &str) -> Result<(), String> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            if run > 2 {
                return Err(
                    "deny_list wildcard runs must be '*' or '**' (no '***' or longer)".to_string(),
                );
            }
        } else {
            run = 0;
        }
    }
    Ok(())
}

fn validate_dimension_v3(
    prefix: &str,
    dim: &mut PolicySnapshotWorldFsDimensionV3,
) -> Result<(), String> {
    if dim.allow_list.is_empty() {
        return Err(format!("{prefix}.allow_list must be non-empty"));
    }

    let mut allow_out = Vec::with_capacity(dim.allow_list.len());
    for raw in &dim.allow_list {
        let normalized =
            normalize_project_pattern_v3(raw).map_err(|e| format!("{prefix}.allow_list: {e}"))?;
        if normalized.contains(['*', '?', '[', ']']) {
            return Err(format!(
                "{prefix}.allow_list contains glob metacharacters; wildcards are not supported in allow_list"
            ));
        }
        allow_out.push(normalized);
    }
    dim.allow_list = allow_out;

    let mut deny_out = Vec::with_capacity(dim.deny_list.len());
    for raw in &dim.deny_list {
        let normalized =
            normalize_project_pattern_v3(raw).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        if normalized.contains(['?', '[', ']']) {
            return Err(format!(
                "{prefix}.deny_list contains unsupported glob metacharacters ('?' or character classes)"
            ));
        }
        validate_deny_wildcards_v3(&normalized).map_err(|e| format!("{prefix}.deny_list: {e}"))?;
        deny_out.push(normalized);
    }
    dim.deny_list = deny_out;

    Ok(())
}

fn normalize_and_validate_world_fs_snapshot_v3(
    world_fs: &mut PolicySnapshotWorldFsV3,
) -> Result<(), String> {
    if !world_fs.write.enabled && !world_fs.fail_closed.routing {
        return Err(
            "world_fs.write.enabled=false requires world_fs.fail_closed.routing=true".to_string(),
        );
    }

    let mut read = world_fs
        .read
        .clone()
        .unwrap_or(PolicySnapshotWorldFsDimensionV3 {
            allow_list: default_allow_list_dot(),
            deny_list: Vec::new(),
        });
    validate_dimension_v3("world_fs.read", &mut read)?;
    world_fs.read = Some(read.clone());

    let mut discover = world_fs.discover.clone().unwrap_or_else(|| read.clone());
    validate_dimension_v3("world_fs.discover", &mut discover)?;
    world_fs.discover = Some(discover.clone());

    let mut write_dim = PolicySnapshotWorldFsDimensionV3 {
        allow_list: world_fs.write.allow_list.clone(),
        deny_list: world_fs.write.deny_list.clone(),
    };
    validate_dimension_v3("world_fs.write", &mut write_dim)?;
    world_fs.write.allow_list = write_dim.allow_list;
    world_fs.write.deny_list = write_dim.deny_list;

    let any_deny = !read.deny_list.is_empty()
        || !discover.deny_list.is_empty()
        || !world_fs.write.deny_list.is_empty();

    if world_fs.host_visible && any_deny {
        return Err("deny_list usage requires world_fs.host_visible=false".to_string());
    }

    if any_deny && world_fs.deny_enforcement.is_none() {
        return Err(
            "world_fs.deny_enforcement must be present when any deny_list is non-empty".to_string(),
        );
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub max_execs: Option<u32>,
    pub max_runtime_ms: Option<u64>,
    pub max_egress_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldNetworkRoutingV1 {
    pub isolate_network: bool,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "ResolvedMemberRuntimeDescriptorDef")]
pub struct ResolvedMemberRuntimeDescriptorV1 {
    pub backend_kind: MemberRuntimeBackendKindV1,
    pub binary_path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRuntimeBackendKindV1 {
    Codex,
    ClaudeCode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolvedMemberRuntimeDescriptorDef {
    backend_kind: MemberRuntimeBackendKindV1,
    binary_path: String,
}

impl ResolvedMemberRuntimeDescriptorV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_non_empty_request_field(
            "member_dispatch.resolved_runtime.binary_path",
            &self.binary_path,
        )?;
        if !Path::new(&self.binary_path).is_absolute() {
            return Err(
                "member_dispatch.resolved_runtime.binary_path must be an absolute path".to_string(),
            );
        }
        Ok(())
    }
}

impl TryFrom<ResolvedMemberRuntimeDescriptorDef> for ResolvedMemberRuntimeDescriptorV1 {
    type Error = String;

    fn try_from(value: ResolvedMemberRuntimeDescriptorDef) -> Result<Self, Self::Error> {
        let descriptor = Self {
            backend_kind: value.backend_kind,
            binary_path: value.binary_path,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum RetainedWorkerAuthorityObjectCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

impl RetainedWorkerAuthorityObjectCommitmentV1 {
    fn validate(&self, field: &str) -> Result<(), String> {
        match self {
            Self::CanonicalSha256 { digest_hex } => {
                validate_lowercase_sha256_digest(field, digest_hex)
            }
            Self::StoreHmacSha256 {
                key_id,
                domain,
                digest_hex,
            } => {
                validate_non_empty_request_field(&format!("{field}.key_id"), key_id)?;
                validate_non_empty_request_field(&format!("{field}.domain"), domain)?;
                validate_lowercase_sha256_digest(field, digest_hex)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RetainedWorkerAdmissionCommitmentCarrierDef")]
pub struct RetainedWorkerAdmissionCommitmentCarrierV1 {
    pub schema_version: u32,
    pub algorithm: String,
    pub key_id: String,
    pub digest_hex: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerAdmissionCommitmentCarrierDef {
    schema_version: u32,
    algorithm: String,
    key_id: String,
    digest_hex: String,
}

impl RetainedWorkerAdmissionCommitmentCarrierV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported retained_worker_launch_authority.canonical_spawn_fingerprint.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        if self.algorithm != "hmac-sha-256" {
            return Err(
                "retained_worker_launch_authority.canonical_spawn_fingerprint.algorithm must be hmac-sha-256"
                    .to_string(),
            );
        }
        validate_non_empty_request_field(
            "retained_worker_launch_authority.canonical_spawn_fingerprint.key_id",
            &self.key_id,
        )?;
        validate_lowercase_sha256_digest(
            "retained_worker_launch_authority.canonical_spawn_fingerprint",
            &self.digest_hex,
        )
    }
}

impl TryFrom<RetainedWorkerAdmissionCommitmentCarrierDef>
    for RetainedWorkerAdmissionCommitmentCarrierV1
{
    type Error = String;

    fn try_from(value: RetainedWorkerAdmissionCommitmentCarrierDef) -> Result<Self, Self::Error> {
        let commitment = Self {
            schema_version: value.schema_version,
            algorithm: value.algorithm,
            key_id: value.key_id,
            digest_hex: value.digest_hex,
        };
        commitment.validate()?;
        Ok(commitment)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RetainedWorkerLaunchWorldBindingV1 {
    pub world_id: String,
    pub world_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "RetainedWorkerLaunchAuthorityProofDef")]
pub struct RetainedWorkerLaunchAuthorityProofV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub issuer_request_id: String,
    pub canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
    pub registration_id: String,
    pub registration_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
    pub authority_revision_after: u64,
    pub authority_record_commitment_after: RetainedWorkerAuthorityObjectCommitmentV1,
    pub orchestration_session_id: String,
    pub caller_participant_id: String,
    pub retained_participant_id: String,
    pub bootstrap_run_id: String,
    pub transport_claim_id: String,
    pub backend_id: String,
    pub protocol: String,
    pub world_binding: RetainedWorkerLaunchWorldBindingV1,
    pub current_policy_ref_id: String,
    pub current_policy_revision: String,
    pub retained_worker_ref_id: String,
    pub retained_worker_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedWorkerLaunchAuthorityProofDef {
    schema_version: u32,
    authority_store_id: String,
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
    registration_id: String,
    registration_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
    authority_revision_after: u64,
    authority_record_commitment_after: RetainedWorkerAuthorityObjectCommitmentV1,
    orchestration_session_id: String,
    caller_participant_id: String,
    retained_participant_id: String,
    bootstrap_run_id: String,
    transport_claim_id: String,
    backend_id: String,
    protocol: String,
    world_binding: RetainedWorkerLaunchWorldBindingV1,
    current_policy_ref_id: String,
    current_policy_revision: String,
    retained_worker_ref_id: String,
    retained_worker_commitment: RetainedWorkerAuthorityObjectCommitmentV1,
}

impl RetainedWorkerLaunchAuthorityProofV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported retained_worker_launch_authority.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        for (field, value) in [
            ("authority_store_id", self.authority_store_id.as_str()),
            ("issuer_request_id", self.issuer_request_id.as_str()),
            ("registration_id", self.registration_id.as_str()),
            (
                "orchestration_session_id",
                self.orchestration_session_id.as_str(),
            ),
            ("caller_participant_id", self.caller_participant_id.as_str()),
            (
                "retained_participant_id",
                self.retained_participant_id.as_str(),
            ),
            ("bootstrap_run_id", self.bootstrap_run_id.as_str()),
            ("transport_claim_id", self.transport_claim_id.as_str()),
            ("backend_id", self.backend_id.as_str()),
            ("protocol", self.protocol.as_str()),
            (
                "world_binding.world_id",
                self.world_binding.world_id.as_str(),
            ),
            ("current_policy_ref_id", self.current_policy_ref_id.as_str()),
            (
                "current_policy_revision",
                self.current_policy_revision.as_str(),
            ),
            (
                "retained_worker_ref_id",
                self.retained_worker_ref_id.as_str(),
            ),
        ] {
            validate_non_empty_request_field(
                &format!("retained_worker_launch_authority.{field}"),
                value,
            )?;
        }
        self.canonical_spawn_fingerprint.validate()?;
        self.registration_commitment
            .validate("retained_worker_launch_authority.registration_commitment")?;
        self.authority_record_commitment_after
            .validate("retained_worker_launch_authority.authority_record_commitment_after")?;
        self.retained_worker_commitment
            .validate("retained_worker_launch_authority.retained_worker_commitment")?;
        Ok(())
    }
}

impl TryFrom<RetainedWorkerLaunchAuthorityProofDef> for RetainedWorkerLaunchAuthorityProofV1 {
    type Error = String;

    fn try_from(value: RetainedWorkerLaunchAuthorityProofDef) -> Result<Self, Self::Error> {
        let proof = Self {
            schema_version: value.schema_version,
            authority_store_id: value.authority_store_id,
            issuer_request_id: value.issuer_request_id,
            canonical_spawn_fingerprint: value.canonical_spawn_fingerprint,
            registration_id: value.registration_id,
            registration_commitment: value.registration_commitment,
            authority_revision_after: value.authority_revision_after,
            authority_record_commitment_after: value.authority_record_commitment_after,
            orchestration_session_id: value.orchestration_session_id,
            caller_participant_id: value.caller_participant_id,
            retained_participant_id: value.retained_participant_id,
            bootstrap_run_id: value.bootstrap_run_id,
            transport_claim_id: value.transport_claim_id,
            backend_id: value.backend_id,
            protocol: value.protocol,
            world_binding: value.world_binding,
            current_policy_ref_id: value.current_policy_ref_id,
            current_policy_revision: value.current_policy_revision,
            retained_worker_ref_id: value.retained_worker_ref_id,
            retained_worker_commitment: value.retained_worker_commitment,
        };
        proof.validate()?;
        Ok(proof)
    }
}

fn validate_lowercase_sha256_digest(field: &str, digest_hex: &str) -> Result<(), String> {
    if digest_hex.len() != 64
        || !digest_hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "{field}.digest_hex must be exactly 64 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "MemberDispatchRequestDef")]
pub struct MemberDispatchRequestV1 {
    #[serde(default = "member_dispatch_request_v1_default_schema_version")]
    pub schema_version: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,
    pub backend_id: String,
    pub protocol: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
    pub resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberDispatchRequestDef {
    #[serde(default = "member_dispatch_request_v1_default_schema_version")]
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    #[serde(default)]
    parent_participant_id: Option<String>,
    #[serde(default)]
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    #[serde(default)]
    initial_prompt: Option<String>,
    resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    #[serde(default)]
    retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
}

fn member_dispatch_request_v1_default_schema_version() -> u32 {
    1
}

impl MemberDispatchRequestV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported member_dispatch.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }

        validate_non_empty_request_field(
            "member_dispatch.orchestration_session_id",
            &self.orchestration_session_id,
        )?;
        validate_non_empty_request_field("member_dispatch.participant_id", &self.participant_id)?;
        validate_non_empty_request_field(
            "member_dispatch.orchestrator_participant_id",
            &self.orchestrator_participant_id,
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.parent_participant_id",
            self.parent_participant_id.as_deref(),
        )?;
        validate_optional_non_empty_request_field(
            "member_dispatch.resumed_from_participant_id",
            self.resumed_from_participant_id.as_deref(),
        )?;
        validate_non_empty_request_field("member_dispatch.backend_id", &self.backend_id)?;
        validate_non_empty_request_field("member_dispatch.protocol", &self.protocol)?;
        validate_non_empty_request_field("member_dispatch.run_id", &self.run_id)?;
        validate_non_empty_request_field("member_dispatch.world_id", &self.world_id)?;
        validate_optional_non_empty_request_field(
            "member_dispatch.initial_prompt",
            self.initial_prompt.as_deref(),
        )?;
        self.resolved_runtime.validate()?;
        if let Some(proof) = self.retained_worker_launch_authority.as_ref() {
            proof.validate()?;
        }

        if self.orchestrator_participant_id == self.participant_id {
            return Err(
                "member_dispatch.orchestrator_participant_id must not equal participant_id"
                    .to_string(),
            );
        }

        if self.parent_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.parent_participant_id must not point to participant_id"
                    .to_string(),
            );
        }

        if self.resumed_from_participant_id.as_deref() == Some(self.participant_id.as_str()) {
            return Err(
                "member_dispatch.resumed_from_participant_id must not point to participant_id"
                    .to_string(),
            );
        }

        Ok(())
    }
}

impl TryFrom<MemberDispatchRequestDef> for MemberDispatchRequestV1 {
    type Error = String;

    fn try_from(value: MemberDispatchRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            orchestration_session_id: value.orchestration_session_id,
            participant_id: value.participant_id,
            orchestrator_participant_id: value.orchestrator_participant_id,
            parent_participant_id: value.parent_participant_id,
            resumed_from_participant_id: value.resumed_from_participant_id,
            backend_id: value.backend_id,
            protocol: value.protocol,
            run_id: value.run_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            initial_prompt: value.initial_prompt,
            resolved_runtime: value.resolved_runtime,
            retained_worker_launch_authority: value.retained_worker_launch_authority,
        };
        request.validate()?;
        Ok(request)
    }
}

/// Request-scoped proposal identity retained unchanged until runtime acknowledgement.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorldWorkAcceptanceContextV1 {
    pub schema_version: u32,
    pub proposed_acceptance_record_id: String,
    pub request_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    pub caller_backend_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorldWorkAcceptanceContextDef {
    schema_version: u32,
    proposed_acceptance_record_id: String,
    request_id: String,
    #[serde(default)]
    message_id: Option<String>,
    caller_backend_id: String,
    #[serde(default)]
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
}

impl WorldWorkAcceptanceContextV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported acceptance_context.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }
        validate_prefixed_uuid_v7(
            "acceptance_context.proposed_acceptance_record_id",
            &self.proposed_acceptance_record_id,
            "wwa_",
        )?;
        validate_non_empty_request_field("acceptance_context.request_id", &self.request_id)?;
        validate_non_empty_request_field(
            "acceptance_context.caller_backend_id",
            &self.caller_backend_id,
        )?;
        if let Some(message_id) = self.message_id.as_deref() {
            validate_prefixed_uuid_v7("acceptance_context.message_id", message_id, "wwm_")?;
        }
        if let Some(correlation) = self.host_transition_correlation.as_ref() {
            correlation.validate()?;
        }
        Ok(())
    }
}

impl TryFrom<WorldWorkAcceptanceContextDef> for WorldWorkAcceptanceContextV1 {
    type Error = String;

    fn try_from(value: WorldWorkAcceptanceContextDef) -> Result<Self, Self::Error> {
        let context = Self {
            schema_version: value.schema_version,
            proposed_acceptance_record_id: value.proposed_acceptance_record_id,
            request_id: value.request_id,
            message_id: value.message_id,
            caller_backend_id: value.caller_backend_id,
            host_transition_correlation: value.host_transition_correlation,
        };
        context.validate()?;
        Ok(context)
    }
}

impl<'de> Deserialize<'de> for WorldWorkAcceptanceContextV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = WorldWorkAcceptanceContextDef::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

fn validate_prefixed_uuid_v7(field: &str, value: &str, prefix: &str) -> Result<(), String> {
    let Some(uuid) = value.strip_prefix(prefix) else {
        return Err(format!("{field} must start with {prefix}"));
    };
    let bytes = uuid.as_bytes();
    let hyphens = [8usize, 13, 18, 23];
    if bytes.len() != 36
        || hyphens.iter().any(|index| bytes[*index] != b'-')
        || bytes.iter().enumerate().any(|(index, byte)| {
            !(hyphens.contains(&index) || byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        })
        || bytes[14] != b'7'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
    {
        return Err(format!("{field} must contain a lowercase UUIDv7"));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "MemberTurnSubmitRequestDef")]
pub struct MemberTurnSubmitRequestV1 {
    #[serde(default = "member_turn_submit_request_v1_default_schema_version")]
    pub schema_version: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    pub backend_id: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_context: Option<WorldWorkAcceptanceContextV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberTurnSubmitRequestDef {
    #[serde(default = "member_turn_submit_request_v1_default_schema_version")]
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    backend_id: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    prompt: String,
    #[serde(default)]
    acceptance_context: Option<WorldWorkAcceptanceContextV1>,
}

fn member_turn_submit_request_v1_default_schema_version() -> u32 {
    1
}

impl MemberTurnSubmitRequestV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported member_turn_submit.schema_version: {} (expected 1)",
                self.schema_version
            ));
        }

        validate_non_empty_request_field(
            "member_turn_submit.orchestration_session_id",
            &self.orchestration_session_id,
        )?;
        validate_non_empty_request_field(
            "member_turn_submit.participant_id",
            &self.participant_id,
        )?;
        validate_non_empty_request_field(
            "member_turn_submit.orchestrator_participant_id",
            &self.orchestrator_participant_id,
        )?;
        validate_non_empty_request_field("member_turn_submit.backend_id", &self.backend_id)?;
        validate_gateway_backend_id_selector(self.backend_id.trim()).map_err(|_| {
            format!(
                "invalid member_turn_submit.backend_id '{}'; expected <kind>:<name>",
                self.backend_id.trim()
            )
        })?;
        validate_non_empty_request_field("member_turn_submit.run_id", &self.run_id)?;
        validate_non_empty_request_field("member_turn_submit.world_id", &self.world_id)?;
        validate_non_empty_request_field("member_turn_submit.prompt", &self.prompt)?;

        if let Some(context) = self.acceptance_context.as_ref() {
            context.validate()?;
            if context.request_id != self.run_id {
                return Err(
                    "member_turn_submit.acceptance_context.request_id must equal run_id"
                        .to_string(),
                );
            }
            if context.message_id.is_none() {
                return Err(
                    "member_turn_submit.acceptance_context.message_id is required".to_string(),
                );
            }
        }

        if self.orchestrator_participant_id == self.participant_id {
            return Err(
                "member_turn_submit.orchestrator_participant_id must not equal participant_id"
                    .to_string(),
            );
        }

        Ok(())
    }
}

impl TryFrom<MemberTurnSubmitRequestDef> for MemberTurnSubmitRequestV1 {
    type Error = String;

    fn try_from(value: MemberTurnSubmitRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            orchestration_session_id: value.orchestration_session_id,
            participant_id: value.participant_id,
            orchestrator_participant_id: value.orchestrator_participant_id,
            backend_id: value.backend_id,
            run_id: value.run_id,
            world_id: value.world_id,
            world_generation: value.world_generation,
            prompt: value.prompt,
            acceptance_context: value.acceptance_context,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "ExecuteRequestDef")]
pub struct ExecuteRequest {
    pub profile: Option<String>,
    pub cmd: String,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub pty: bool,
    pub agent_id: String,
    pub budget: Option<Budget>,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_world: Option<SharedWorldOwnerSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_fs_mode: Option<WorldFsMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_dispatch: Option<MemberDispatchRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_context: Option<WorldWorkAcceptanceContextV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecuteRequestDef {
    profile: Option<String>,
    cmd: String,
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
    pty: bool,
    agent_id: String,
    budget: Option<Budget>,
    policy_snapshot: PolicySnapshotV3,
    #[serde(default)]
    shared_world: Option<SharedWorldOwnerSpec>,
    #[serde(default)]
    world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default)]
    world_fs_mode: Option<WorldFsMode>,
    #[serde(default)]
    member_dispatch: Option<MemberDispatchRequestV1>,
    #[serde(default)]
    acceptance_context: Option<WorldWorkAcceptanceContextV1>,
}

impl ExecuteRequest {
    pub fn validate(&self) -> Result<(), String> {
        let cmd_is_empty = self.cmd.trim().is_empty();

        match self.member_dispatch.as_ref() {
            Some(member_dispatch) => {
                member_dispatch.validate()?;
                if !cmd_is_empty {
                    return Err(
                        "execute request member_dispatch requires cmd.trim().is_empty()"
                            .to_string(),
                    );
                }
                if self.pty {
                    return Err("execute request member_dispatch requires pty=false".to_string());
                }
                if let Some(context) = self.acceptance_context.as_ref() {
                    context.validate()?;
                    if context.request_id != member_dispatch.run_id {
                        return Err(
                            "execute request acceptance_context.request_id must equal member_dispatch.run_id"
                                .to_string(),
                        );
                    }
                    if context.message_id.is_some() {
                        return Err(
                            "execute request task acceptance_context.message_id must be absent"
                                .to_string(),
                        );
                    }
                }
            }
            None => {
                if cmd_is_empty {
                    return Err(
                        "execute request process exec requires a non-empty cmd when member_dispatch is absent"
                        .to_string(),
                    );
                }
                if self.acceptance_context.is_some() {
                    return Err(
                        "execute request acceptance_context requires member_dispatch".to_string(),
                    );
                }
            }
        }

        Ok(())
    }
}

impl TryFrom<ExecuteRequestDef> for ExecuteRequest {
    type Error = String;

    fn try_from(value: ExecuteRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            profile: value.profile,
            cmd: value.cmd,
            cwd: value.cwd,
            env: value.env,
            pty: value.pty,
            agent_id: value.agent_id,
            budget: value.budget,
            policy_snapshot: value.policy_snapshot,
            shared_world: value.shared_world,
            world_network: value.world_network,
            world_fs_mode: value.world_fs_mode,
            member_dispatch: value.member_dispatch,
            acceptance_context: value.acceptance_context,
        };
        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub exit: i32,
    pub span_id: String,
    pub stdout_b64: String,
    pub stderr_b64: String,
    pub scopes_used: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs_diff: Option<FsDiff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_world: Option<SharedWorldBindingSnapshot>,
    #[serde(flatten, default)]
    pub process_telemetry: ProcessTelemetry,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecuteCancelRequestV1 {
    pub span_id: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecuteCancelResponseV1 {
    #[serde(default = "execute_cancel_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub delivered: bool,
}

fn execute_cancel_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecuteStreamReplayRequestV1 {
    #[serde(default = "execute_stream_replay_request_v1_default_schema_version")]
    pub schema_version: u32,
    pub acceptance_record_id: String,
    pub stream_id: String,
    pub after_frame_sequence: u64,
}

fn execute_stream_replay_request_v1_default_schema_version() -> u32 {
    1
}

impl ExecuteStreamReplayRequestV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported execute stream replay schema version {}",
                self.schema_version
            ));
        }
        for (field, value, prefix) in [
            (
                "acceptance_record_id",
                self.acceptance_record_id.as_str(),
                "wwa_",
            ),
            ("stream_id", self.stream_id.as_str(), "rts_"),
        ] {
            if value.trim() != value || !value.starts_with(prefix) || value.len() == prefix.len() {
                return Err(format!(
                    "execute stream replay {field} must be exact, trimmed, and {prefix}-prefixed"
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingDiffBucketV1 {
    pub writes: Vec<String>,
    pub mods: Vec<String>,
    pub deletes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffRecordV1 {
    #[serde(default = "pending_diff_record_v1_default_schema_version")]
    pub schema_version: u32,
    pub session_started_at: String,
    pub diff_id: String,
    pub non_pty: PendingDiffBucketV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pty: Option<PendingDiffBucketV1>,
}

fn pending_diff_record_v1_default_schema_version() -> u32 {
    1
}

fn validate_non_empty_request_field(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(())
}

fn validate_optional_non_empty_request_field(
    field: &str,
    value: Option<&str>,
) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_non_empty_request_field(field, value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffClearRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub diff_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffReconcileRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub diff_id: String,
    pub discard_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffReconcileResponseV1 {
    #[serde(default = "pending_diff_reconcile_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub reconciled: bool,
    pub discarded: u32,
}

fn pending_diff_reconcile_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDiffClearResponseV1 {
    #[serde(default = "pending_diff_clear_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub cleared: bool,
}

fn pending_diff_clear_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsEntryTypeV1 {
    RegularFile,
    Directory,
    Symlink,
    Socket,
    Fifo,
    BlockDevice,
    CharDevice,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFsReadRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    pub path: String,
    #[serde(default)]
    pub include_contents: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFsReadResponseV1 {
    #[serde(default = "world_fs_read_response_v1_default_schema_version")]
    pub schema_version: u32,
    pub path: String,
    pub entry_type: WorldFsEntryTypeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contents_b64: Option<String>,
}

fn world_fs_read_response_v1_default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayCliCodexIntegratedAuthV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayApiEnvIntegratedAuthV1 {
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayIntegratedAuthPayloadV1 {
    pub backend_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_codex: Option<GatewayCliCodexIntegratedAuthV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_env: Option<GatewayApiEnvIntegratedAuthV1>,
}

impl GatewayIntegratedAuthPayloadV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_gateway_integrated_auth_payload(self)
    }

    pub fn validate_for_selected_backend(&self, selected_backend: &str) -> Result<(), String> {
        validate_gateway_integrated_auth_payload_for_selected_backend(self, selected_backend)
    }
}

pub fn validate_gateway_backend_id_selector(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    let Some((kind, name)) = trimmed.split_once(':') else {
        return Err(format!(
            "invalid gateway backend_id '{}'; expected <kind>:<name>",
            trimmed
        ));
    };

    if !matches_backend_kind(kind) || !matches_backend_name(name) || name.contains(':') {
        return Err(format!(
            "invalid gateway backend_id '{}'; expected <kind>:<name> with kind [a-z0-9_]+ and name [a-z0-9_-]+",
            trimmed
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "GatewayLifecycleRequestDef")]
pub struct GatewayLifecycleRequestV1 {
    pub profile: Option<String>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub agent_id: String,
    pub policy_snapshot: PolicySnapshotV3,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrated_auth: Option<GatewayIntegratedAuthPayloadV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_tuple: Option<IdentityTuple>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_posture: Option<PlacementPosture>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatewayLifecycleRequestDef {
    profile: Option<String>,
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
    agent_id: String,
    policy_snapshot: PolicySnapshotV3,
    #[serde(default)]
    world_network: Option<WorldNetworkRoutingV1>,
    #[serde(default)]
    integrated_auth: Option<GatewayIntegratedAuthPayloadV1>,
    #[serde(default)]
    identity_tuple: Option<IdentityTuple>,
    #[serde(default)]
    placement_posture: Option<PlacementPosture>,
}

impl GatewayLifecycleRequestV1 {
    pub fn validate_identity_contract(&self) -> Result<(), String> {
        validate_identity_tuple_and_placement_posture(
            self.identity_tuple.as_ref(),
            self.placement_posture.as_ref(),
        )
    }
}

impl TryFrom<GatewayLifecycleRequestDef> for GatewayLifecycleRequestV1 {
    type Error = String;

    fn try_from(value: GatewayLifecycleRequestDef) -> Result<Self, Self::Error> {
        let request = Self {
            profile: value.profile,
            cwd: value.cwd,
            env: value.env,
            agent_id: value.agent_id,
            policy_snapshot: value.policy_snapshot,
            world_network: value.world_network,
            integrated_auth: value.integrated_auth,
            identity_tuple: value.identity_tuple,
            placement_posture: value.placement_posture,
        };
        request.validate_identity_contract()?;
        Ok(request)
    }
}

pub fn validate_gateway_integrated_auth_payload(
    payload: &GatewayIntegratedAuthPayloadV1,
) -> Result<(), String> {
    let backend_id = payload.backend_id.trim();
    validate_gateway_backend_id_selector(backend_id).map_err(|err| {
        if backend_id.is_empty() {
            "request-provided integrated auth payload is missing backend_id".to_string()
        } else {
            err
        }
    })?;

    let cli_codex = payload.cli_codex.as_ref();
    let api_env = payload.api_env.as_ref();
    let facet_count = usize::from(cli_codex.is_some()) + usize::from(api_env.is_some());

    if facet_count != 1 {
        return Err(format!(
            "request-provided integrated auth payload must contain exactly one auth facet (found {facet_count})"
        ));
    }

    if let Some(cli_codex) = cli_codex {
        if !is_cli_codex_gateway_backend(backend_id) {
            return Err(format!(
                "request-provided integrated auth payload for '{}' uses incompatible auth facet 'cli_codex'",
                backend_id
            ));
        }

        if cli_codex
            .account_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(
                "request-provided integrated auth payload contains empty cli_codex.account_id"
                    .to_string(),
            );
        }

        if cli_codex.access_token.trim().is_empty() {
            return Err(
                "request-provided integrated auth payload contains empty cli_codex.access_token"
                    .to_string(),
            );
        }
    }

    if let Some(api_env) = api_env {
        if !backend_id.starts_with("api:") && !is_cli_claude_code_gateway_backend(backend_id) {
            return Err(format!(
                "request-provided integrated auth payload for '{}' uses incompatible auth facet 'api_env'",
                backend_id
            ));
        }

        if api_env.env.is_empty() {
            return Err(
                "request-provided integrated auth payload contains empty api_env.env".to_string(),
            );
        }

        for (name, value) in &api_env.env {
            let trimmed_name = name.trim();
            if trimmed_name.is_empty() {
                return Err(
                    "request-provided integrated auth payload contains blank api_env env name"
                        .to_string(),
                );
            }
            if trimmed_name != name
                || trimmed_name.contains(char::is_whitespace)
                || trimmed_name.contains('=')
            {
                return Err(format!(
                    "request-provided integrated auth payload contains invalid api_env env name '{}'",
                    name
                ));
            }
            if value.trim().is_empty() {
                return Err(format!(
                    "request-provided integrated auth payload contains empty api_env value for '{}'",
                    name
                ));
            }
        }
    }

    Ok(())
}

fn is_cli_codex_gateway_backend(backend_id: &str) -> bool {
    matches!(
        backend_id,
        "cli:codex" | "cli:codex-host" | "cli:codex-world"
    )
}

fn is_cli_claude_code_gateway_backend(backend_id: &str) -> bool {
    matches!(
        backend_id,
        "cli:claude_code" | "cli:claude_code-host" | "cli:claude_code-world"
    )
}

fn matches_backend_kind(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn matches_backend_name(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

pub fn validate_gateway_integrated_auth_payload_for_selected_backend(
    payload: &GatewayIntegratedAuthPayloadV1,
    selected_backend: &str,
) -> Result<(), String> {
    validate_gateway_integrated_auth_payload(payload)?;

    let selected_backend = selected_backend.trim();
    if payload.backend_id.trim() != selected_backend {
        return Err(format!(
            "request-provided integrated auth payload for '{}' does not match selected backend '{}'",
            payload.backend_id.trim(),
            selected_backend
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayStatusV1 {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayClientWiringV1 {
    pub openai_base_url: String,
    pub anthropic_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "GatewayLifecycleResponseDef")]
pub struct GatewayLifecycleResponseV1 {
    pub status: GatewayStatusV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_wiring: Option<GatewayClientWiringV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_tuple: Option<IdentityTuple>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_posture: Option<PlacementPosture>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct GatewayLifecycleResponseDef {
    status: GatewayStatusV1,
    #[serde(default)]
    client_wiring: Option<GatewayClientWiringV1>,
    #[serde(default)]
    identity_tuple: Option<IdentityTuple>,
    #[serde(default)]
    placement_posture: Option<PlacementPosture>,
}

impl GatewayLifecycleResponseV1 {
    pub fn validate_identity_contract(&self) -> Result<(), String> {
        validate_identity_tuple_and_placement_posture(
            self.identity_tuple.as_ref(),
            self.placement_posture.as_ref(),
        )
    }
}

impl TryFrom<GatewayLifecycleResponseDef> for GatewayLifecycleResponseV1 {
    type Error = String;

    fn try_from(value: GatewayLifecycleResponseDef) -> Result<Self, Self::Error> {
        let response = Self {
            status: value.status,
            client_wiring: value.client_wiring,
            identity_tuple: value.identity_tuple,
            placement_posture: value.placement_posture,
        };
        response.validate_identity_contract()?;
        Ok(response)
    }
}

/// Streaming frame describing incremental execution output.
#[derive(Debug, Clone, Serialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecuteStreamFrame {
    /// Initial handshake announcing the span identifier for this execution.
    Start {
        frame_identity: RuntimeFrameIdentityV1,
        span_id: String,
    },
    /// Incremental stdout data (base64 encoded for transport safety).
    Stdout {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    /// Incremental stderr data (base64 encoded for transport safety).
    Stderr {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    /// Optional higher-level agent event forwarded from the world.
    Event {
        frame_identity: RuntimeFrameIdentityV1,
        event: AgentEvent,
    },
    /// Terminal frame with exit metadata and optional filesystem diff.
    Exit {
        frame_identity: RuntimeFrameIdentityV1,
        event_identity: RuntimeEventIdentityV1,
        terminal_identity: RuntimeTerminalIdentityV1,
        exit: i32,
        span_id: String,
        scopes_used: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fs_diff: Option<FsDiff>,
        #[serde(flatten, default)]
        process_telemetry: ProcessTelemetry,
    },
    /// Error reported while attempting to execute the command.
    Error {
        frame_identity: RuntimeFrameIdentityV1,
        message: String,
    },
}

#[derive(Debug, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ExecuteStreamFrameDef {
    Start {
        frame_identity: RuntimeFrameIdentityV1,
        span_id: String,
    },
    Stdout {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    Stderr {
        frame_identity: RuntimeFrameIdentityV1,
        chunk_b64: String,
    },
    Event {
        frame_identity: RuntimeFrameIdentityV1,
        event: AgentEvent,
        #[serde(default)]
        event_identity: Option<RuntimeEventIdentityV1>,
    },
    Exit {
        frame_identity: RuntimeFrameIdentityV1,
        event_identity: RuntimeEventIdentityV1,
        terminal_identity: RuntimeTerminalIdentityV1,
        exit: i32,
        span_id: String,
        scopes_used: Vec<String>,
        #[serde(default)]
        fs_diff: Option<FsDiff>,
        #[serde(flatten, default)]
        process_telemetry: ProcessTelemetry,
    },
    Error {
        frame_identity: RuntimeFrameIdentityV1,
        message: String,
    },
}

impl ExecuteStreamFrame {
    pub fn validate_identity_contract(&self) -> Result<(), String> {
        let frame_identity = match self {
            Self::Start { frame_identity, .. }
            | Self::Stdout { frame_identity, .. }
            | Self::Stderr { frame_identity, .. }
            | Self::Event { frame_identity, .. }
            | Self::Exit { frame_identity, .. }
            | Self::Error { frame_identity, .. } => frame_identity,
        };
        frame_identity.validate()?;

        match self {
            Self::Event { event, .. } => event
                .event_identity
                .as_ref()
                .ok_or_else(|| "runtime Event frame requires event.event_identity".to_string())
                .and_then(RuntimeEventIdentityV1::validate),
            Self::Exit {
                event_identity,
                terminal_identity,
                ..
            } => {
                event_identity.validate()?;
                terminal_identity.validate()?;
                if !terminal_identity.matches_event(event_identity) {
                    return Err(
                        "runtime Exit event_identity must equal terminal_identity".to_string()
                    );
                }
                Ok(())
            }
            Self::Start { .. } | Self::Stdout { .. } | Self::Stderr { .. } | Self::Error { .. } => {
                Ok(())
            }
        }
    }

    pub fn terminal_identity(&self) -> Option<&RuntimeTerminalIdentityV1> {
        match self {
            Self::Exit {
                terminal_identity, ..
            } => Some(terminal_identity),
            _ => None,
        }
    }

    /// Serialize one already-identified frame to its canonical replayable NDJSON bytes.
    pub fn canonical_ndjson_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate_identity_contract()?;
        let mut bytes = serde_json::to_vec(self).map_err(|err| err.to_string())?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

impl TryFrom<ExecuteStreamFrameDef> for ExecuteStreamFrame {
    type Error = String;

    fn try_from(value: ExecuteStreamFrameDef) -> Result<Self, String> {
        let frame = match value {
            ExecuteStreamFrameDef::Start {
                frame_identity,
                span_id,
            } => Self::Start {
                frame_identity,
                span_id,
            },
            ExecuteStreamFrameDef::Stdout {
                frame_identity,
                chunk_b64,
            } => Self::Stdout {
                frame_identity,
                chunk_b64,
            },
            ExecuteStreamFrameDef::Stderr {
                frame_identity,
                chunk_b64,
            } => Self::Stderr {
                frame_identity,
                chunk_b64,
            },
            ExecuteStreamFrameDef::Event {
                frame_identity,
                event,
                event_identity,
            } => {
                if event_identity.is_some() {
                    return Err(
                        "runtime Event frame must not duplicate event_identity outside event"
                            .to_string(),
                    );
                }
                Self::Event {
                    frame_identity,
                    event,
                }
            }
            ExecuteStreamFrameDef::Exit {
                frame_identity,
                event_identity,
                terminal_identity,
                exit,
                span_id,
                scopes_used,
                fs_diff,
                process_telemetry,
            } => Self::Exit {
                frame_identity,
                event_identity,
                terminal_identity,
                exit,
                span_id,
                scopes_used,
                fs_diff,
                process_telemetry,
            },
            ExecuteStreamFrameDef::Error {
                frame_identity,
                message,
            } => Self::Error {
                frame_identity,
                message,
            },
        };
        frame.validate_identity_contract()?;
        Ok(frame)
    }
}

impl<'de> Deserialize<'de> for ExecuteStreamFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = ExecuteStreamFrameDef::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("bad_request: {0}")]
    BadRequest(String),
    #[error("not_found: {0}")]
    NotFound(String),
    #[error("rate_limited: {0}")]
    RateLimited(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Agent-reported world enforcement readiness (world scope).
///
/// This response is produced by `GET /v1/doctor/world`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorReportV1 {
    pub schema_version: u32,
    pub ok: bool,
    pub collected_at_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_host_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_context_commitment: Option<String>,
    /// Whether the connected world-service supports ingesting `PolicySnapshotV1` on execution requests.
    #[serde(default)]
    pub policy_snapshot_v1_supported: bool,
    /// The policy resolution mode most recently used by the world-service (when known).
    #[serde(default)]
    pub policy_resolution_mode: Option<PolicyResolutionModeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netfilter_status: Option<WorldDoctorNetfilterStatusV1>,
    pub landlock: WorldDoctorLandlockV1,
    pub world_fs_strategy: WorldDoctorWorldFsStrategyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldDoctorNetfilterStatusV1 {
    pub requested: bool,
    pub enabled: bool,
    pub world_netfilter_enable_present: bool,
    #[serde(default)]
    pub last_failure_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyResolutionModeV1 {
    SnapshotV3,
    LegacyLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorLandlockV1 {
    pub supported: bool,
    pub abi: Option<u32>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorWorldFsStrategyV1 {
    pub primary: WorldDoctorWorldFsStrategyKindV1,
    pub fallback: WorldDoctorWorldFsStrategyKindV1,
    pub probe: WorldDoctorWorldFsStrategyProbeV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorWorldFsStrategyKindV1 {
    Overlay,
    Fuse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldDoctorWorldFsStrategyProbeV1 {
    pub id: String,
    pub probe_file: String,
    pub result: WorldDoctorWorldFsStrategyProbeResultV1,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorWorldFsStrategyProbeResultV1 {
    Pass,
    Fail,
}

#[cfg(test)]
mod tests {
    use super::*;

    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use serde_json::{json, Value};

    const UNIX_IH_FRAME: &str = "domain=substrate.install_bootstrap_context\n\
version=1\n\
selected_host_prefix=L29wdC9zdWJzdHJhdGUtZGV2\n\
host_substrate_home=L29wdC9zdWJzdHJhdGUtZGV2\n\
host_substrate_root=L29wdC9zdWJzdHJhdGUtZGV2\n\
principal_kind=unix\n\
principal_account=YWxpY2U\n\
principal_uid=1000\n";
    const UNIX_IH_COMMITMENT: &str =
        "0320704f788ba8e9f13b2eed7af83ad3f33fc999ab80f21abc83722314c9a79d";
    const UNIX_IH_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5pbnN0YWxsX2Jvb3RzdHJhcF9jb250ZXh0CnZlcnNpb249MQpzZWxlY3RlZF9ob3N0X3ByZWZpeD1MMjl3ZEM5emRXSnpkSEpoZEdVdFpHVjIKaG9zdF9zdWJzdHJhdGVfaG9tZT1MMjl3ZEM5emRXSnpkSEpoZEdVdFpHVjIKaG9zdF9zdWJzdHJhdGVfcm9vdD1MMjl3ZEM5emRXSnpkSEpoZEdVdFpHVjIKcHJpbmNpcGFsX2tpbmQ9dW5peApwcmluY2lwYWxfYWNjb3VudD1ZV3hwWTJVCnByaW5jaXBhbF91aWQ9MTAwMApob3N0X2NvbnRleHRfY29tbWl0bWVudD0wMzIwNzA0Zjc4OGJhOGU5ZjEzYjJlZWQ3YWY4M2FkM2YzM2ZjOTk5YWI4MGYyMWFiYzgzNzIyMzE0YzlhNzlkCg";
    const WINDOWS_IH_COMMITMENT: &str =
        "3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7";
    const WINDOWS_IH_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5pbnN0YWxsX2Jvb3RzdHJhcF9jb250ZXh0CnZlcnNpb249MQpzZWxlY3RlZF9ob3N0X3ByZWZpeD1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKaG9zdF9zdWJzdHJhdGVfaG9tZT1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKaG9zdF9zdWJzdHJhdGVfcm9vdD1RenBjVlhObGNuTmNRV3hwWTJWY1FYQndSR0YwWVZ4TWIyTmhiRnhUZFdKemRISmhkR1UKcHJpbmNpcGFsX2tpbmQ9d2luZG93cwpwcmluY2lwYWxfYWNjb3VudD1RVU5OUlZ4QmJHbGpaUQpwcmluY2lwYWxfc2lkPVV5MHhMVFV0TWpFdE1UQXdNQQpob3N0X2NvbnRleHRfY29tbWl0bWVudD0zZTFlNzFiMzI1ZTkyYjE2ZjViZmMwZjNkODc1ZmQxNWYwNGExNjE1YWM5MGIxNDM5ZWIzN2FmZGY5MGE1YWM3Cg";
    const LIMA_PM_FRAME: &str = "domain=substrate.platform_bootstrap_mapping\n\
version=1\n\
host_context_commitment=0320704f788ba8e9f13b2eed7af83ad3f33fc999ab80f21abc83722314c9a79d\n\
platform_kind=lima\n\
instance_name=c3Vic3RyYXRl\n\
guest_machine_id=0123456789abcdef0123456789abcdef\n\
host_platform_control_root=L1VzZXJzL2FsaWNlLy5saW1h\n\
realized_substrate_home=L2hvbWUvc3Vic3RyYXRlLy5zdWJzdHJhdGU\n\
realized_principal_account=c3Vic3RyYXRl\n\
realized_principal_uid=1000\n\
transport_kind=lima\n\
transport_host=L29wdC9zdWJzdHJhdGUtZGV2L3NvY2svYWdlbnQuc29jaw\n\
transport_guest_socket=L3J1bi9zdWJzdHJhdGUuc29jaw\n";
    const LIMA_PM_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5wbGF0Zm9ybV9ib290c3RyYXBfbWFwcGluZwp2ZXJzaW9uPTEKaG9zdF9jb250ZXh0X2NvbW1pdG1lbnQ9MDMyMDcwNGY3ODhiYThlOWYxM2IyZWVkN2FmODNhZDNmMzNmYzk5OWFiODBmMjFhYmM4MzcyMjMxNGM5YTc5ZApwbGF0Zm9ybV9raW5kPWxpbWEKaW5zdGFuY2VfbmFtZT1jM1ZpYzNSeVlYUmwKZ3Vlc3RfbWFjaGluZV9pZD0wMTIzNDU2Nzg5YWJjZGVmMDEyMzQ1Njc4OWFiY2RlZgpob3N0X3BsYXRmb3JtX2NvbnRyb2xfcm9vdD1MMVZ6WlhKekwyRnNhV05sTHk1c2FXMWgKcmVhbGl6ZWRfc3Vic3RyYXRlX2hvbWU9TDJodmJXVXZjM1ZpYzNSeVlYUmxMeTV6ZFdKemRISmhkR1UKcmVhbGl6ZWRfcHJpbmNpcGFsX2FjY291bnQ9YzNWaWMzUnlZWFJsCnJlYWxpemVkX3ByaW5jaXBhbF91aWQ9MTAwMAp0cmFuc3BvcnRfa2luZD1saW1hCnRyYW5zcG9ydF9ob3N0PUwyOXdkQzl6ZFdKemRISmhkR1V0WkdWMkwzTnZZMnN2WVdkbGJuUXVjMjlqYXcKdHJhbnNwb3J0X2d1ZXN0X3NvY2tldD1MM0oxYmk5emRXSnpkSEpoZEdVdWMyOWphdwo";
    const WSL_PM_FRAME: &str = "domain=substrate.platform_bootstrap_mapping\n\
version=1\n\
host_context_commitment=3e1e71b325e92b16f5bfc0f3d875fd15f04a1615ac90b1439eb37afdf90a5ac7\n\
platform_kind=wsl\n\
instance_name=U3Vic3RyYXRlLVdTTA\n\
guest_machine_id=abcdef0123456789abcdef0123456789\n\
host_platform_control_root=QzpcVXNlcnNcQWxpY2VcQXBwRGF0YVxMb2NhbFxTdWJzdHJhdGVcZm9yd2FyZGVyXHNjb3Bl\n\
realized_substrate_home=L2hvbWUvYm9iLy5zdWJzdHJhdGU\n\
realized_principal_account=Ym9i\n\
realized_principal_uid=1001\n\
transport_kind=wsl\n\
transport_host=XFwuXHBpcGVcc3Vic3RyYXRlLWFnZW50\n\
transport_guest_socket=L3J1bi9zdWJzdHJhdGUuc29jaw\n";
    const WSL_PM_CARRIER: &str = "ZG9tYWluPXN1YnN0cmF0ZS5wbGF0Zm9ybV9ib290c3RyYXBfbWFwcGluZwp2ZXJzaW9uPTEKaG9zdF9jb250ZXh0X2NvbW1pdG1lbnQ9M2UxZTcxYjMyNWU5MmIxNmY1YmZjMGYzZDg3NWZkMTVmMDRhMTYxNWFjOTBiMTQzOWViMzdhZmRmOTBhNWFjNwpwbGF0Zm9ybV9raW5kPXdzbAppbnN0YW5jZV9uYW1lPVUzVmljM1J5WVhSbExWZFRUQQpndWVzdF9tYWNoaW5lX2lkPWFiY2RlZjAxMjM0NTY3ODlhYmNkZWYwMTIzNDU2Nzg5Cmhvc3RfcGxhdGZvcm1fY29udHJvbF9yb290PVF6cGNWWE5sY25OY1FXeHBZMlZjUVhCd1JHRjBZVnhNYjJOaGJGeFRkV0p6ZEhKaGRHVmNabTl5ZDJGeVpHVnlYSE5qYjNCbApyZWFsaXplZF9zdWJzdHJhdGVfaG9tZT1MMmh2YldVdlltOWlMeTV6ZFdKemRISmhkR1UKcmVhbGl6ZWRfcHJpbmNpcGFsX2FjY291bnQ9WW05aQpyZWFsaXplZF9wcmluY2lwYWxfdWlkPTEwMDEKdHJhbnNwb3J0X2tpbmQ9d3NsCnRyYW5zcG9ydF9ob3N0PVhGd3VYSEJwY0dWY2MzVmljM1J5WVhSbExXRm5aVzUwCnRyYW5zcG9ydF9ndWVzdF9zb2NrZXQ9TDNKMWJpOXpkV0p6ZEhKaGRHVXVjMjlqYXcK";
    const WINDOWS_FORWARDER_SCOPE_FRAME: &str = "domain=substrate.windows_forwarder_scope\n\
version=1\n\
windows_sid=Uy0xLTUtMjEtMTAwMA\n\
distro_name=U3Vic3RyYXRlLVdTTA\n\
guest_machine_id=abcdef0123456789abcdef0123456789\n\
pipe_path=XFwuXHBpcGVcc3Vic3RyYXRlLWFnZW50\n";
    const WINDOWS_FORWARDER_SCOPE_DIGEST: &str =
        "3b3405b2cf309c050f4ba7acb5f43a2348babf18d6be3c426d066ed058a5e75a";

    fn encode_ih_record(record: &str) -> String {
        URL_SAFE_NO_PAD.encode(record.as_bytes())
    }

    fn unix_ih_carrier() -> InstallBootstrapContextCarrierV1 {
        InstallBootstrapContextCarrierV1::decode(UNIX_IH_CARRIER).unwrap()
    }

    fn windows_ih_carrier() -> InstallBootstrapContextCarrierV1 {
        InstallBootstrapContextCarrierV1::decode(WINDOWS_IH_CARRIER).unwrap()
    }

    #[test]
    fn install_bootstrap_context_unix_golden_vector_is_exact() {
        let context =
            InstallBootstrapContextV1::new_unix("/opt//substrate-dev/", "alice", 1000).unwrap();
        let carrier = InstallBootstrapContextCarrierV1::from_context(context.clone()).unwrap();

        assert_eq!(context.selected_host_prefix, "/opt/substrate-dev");
        assert_eq!(
            carrier.commitment_input().unwrap(),
            UNIX_IH_FRAME.as_bytes()
        );
        assert_eq!(carrier.host_context_commitment, UNIX_IH_COMMITMENT);
        assert_eq!(carrier.encode().unwrap(), UNIX_IH_CARRIER);
        assert_eq!(
            InstallBootstrapContextCarrierV1::decode(UNIX_IH_CARRIER).unwrap(),
            carrier
        );
    }

    #[test]
    fn install_bootstrap_context_windows_golden_vector_is_exact() {
        let context = InstallBootstrapContextV1::new_windows(
            r"c:/Users//Alice/AppData/Local/Substrate/",
            r"ACME\Alice",
            "S-1-5-21-1000",
        )
        .unwrap();
        let carrier = InstallBootstrapContextCarrierV1::from_context(context.clone()).unwrap();

        assert_eq!(
            context.selected_host_prefix,
            r"C:\Users\Alice\AppData\Local\Substrate"
        );
        assert_eq!(carrier.host_context_commitment, WINDOWS_IH_COMMITMENT);
        assert_eq!(carrier.encode().unwrap(), WINDOWS_IH_CARRIER);
        assert_eq!(
            InstallBootstrapContextCarrierV1::decode(WINDOWS_IH_CARRIER).unwrap(),
            carrier
        );
    }

    #[test]
    fn install_bootstrap_context_rejects_noncanonical_paths() {
        for raw in [
            "",
            "/",
            "//server/path",
            "relative/path",
            "/a/./b",
            "/a/../b",
        ] {
            assert!(InstallBootstrapContextV1::new_unix(raw, "alice", 1000).is_err());
        }

        for raw in [
            r"C:relative",
            r"\root-relative",
            r"C:\",
            r"\\server\share",
            r"\\?\C:\Substrate",
            r"C:\a\..\b",
            r"C:\CON\Substrate",
            r"C:\bad.\Substrate",
            r"C:\bad:name\Substrate",
        ] {
            assert!(
                InstallBootstrapContextV1::new_windows(raw, "ACME\\Alice", "S-1-5-21-1").is_err()
            );
        }
    }

    #[test]
    fn install_bootstrap_context_rejects_record_and_commitment_tampering() {
        let canonical_record =
            format!("{UNIX_IH_FRAME}host_context_commitment={UNIX_IH_COMMITMENT}\n");
        let cases = [
            canonical_record.replacen("version=1\n", "", 1),
            canonical_record.replacen("version=1\n", "version=1\nversion=1\n", 1),
            canonical_record.replacen("version=1", "unknown=1", 1),
            canonical_record.replacen(
                "version=1\nselected_host_prefix=",
                "selected_host_prefix=version=1\n",
                1,
            ),
            canonical_record.replacen("L29wdC9zdWJzdHJhdGUtZGV2", "***", 1),
            canonical_record.replacen("principal_uid=1000", "principal_uid=01000", 1),
            canonical_record.replacen("principal_uid=1000\n", "principal_uid=1000\r\n", 1),
            canonical_record.replacen(UNIX_IH_COMMITMENT, &"0".repeat(64), 1),
            format!("{canonical_record}extra=field\n"),
        ];

        for record in cases {
            assert!(
                InstallBootstrapContextCarrierV1::decode(&encode_ih_record(&record)).is_err(),
                "accepted tampered record: {record:?}"
            );
        }
        assert!(InstallBootstrapContextCarrierV1::decode("not+base64").is_err());
        assert!(InstallBootstrapContextCarrierV1::decode(&format!("{UNIX_IH_CARRIER}=")).is_err());
    }

    #[test]
    fn platform_bootstrap_mapping_lima_golden_vector_is_exact() {
        let carrier = unix_ih_carrier();
        let mapping = PlatformBootstrapMappingV1::new_lima(
            &carrier,
            "substrate",
            "0123456789abcdef0123456789abcdef",
            "/Users//alice/.lima/",
            "/home/substrate/.substrate",
            "substrate",
            1000,
            "/opt/substrate-dev/sock/agent.sock",
            "/run/substrate.sock",
        )
        .unwrap();

        assert_eq!(mapping.host_platform_control_root, "/Users/alice/.lima");
        assert_eq!(
            URL_SAFE_NO_PAD
                .decode(mapping.encode(&carrier).unwrap())
                .unwrap(),
            LIMA_PM_FRAME.as_bytes()
        );
        assert_eq!(LIMA_PM_FRAME.lines().count(), 13);
        assert!(LIMA_PM_FRAME.ends_with('\n'));
        assert_eq!(mapping.encode(&carrier).unwrap(), LIMA_PM_CARRIER);
        assert_eq!(
            PlatformBootstrapMappingV1::decode(LIMA_PM_CARRIER, &carrier).unwrap(),
            mapping
        );
        assert_ne!(
            mapping.host_platform_control_root,
            mapping.realized_substrate_home
        );
        assert_ne!(
            carrier.context.intended_host_principal,
            mapping.realized_principal
        );
    }

    #[test]
    fn platform_bootstrap_mapping_wsl_golden_vector_is_exact() {
        let carrier = windows_ih_carrier();
        let mapping = PlatformBootstrapMappingV1::new_wsl(
            &carrier,
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"c:/Users/Alice/AppData/Local/Substrate/forwarder/scope/",
            "/home/bob/.substrate",
            "bob",
            1001,
            r"\\.\pipe\Substrate-Agent",
            "/run/substrate.sock",
        )
        .unwrap();

        assert_eq!(
            mapping.host_platform_control_root,
            r"C:\Users\Alice\AppData\Local\Substrate\forwarder\scope"
        );
        assert_eq!(
            mapping.realized_transport,
            PlatformTransportIdentityV1::Wsl {
                pipe_path: r"\\.\pipe\substrate-agent".to_string(),
                guest_socket: "/run/substrate.sock".to_string(),
            }
        );
        assert_eq!(
            URL_SAFE_NO_PAD
                .decode(mapping.encode(&carrier).unwrap())
                .unwrap(),
            WSL_PM_FRAME.as_bytes()
        );
        assert_eq!(WSL_PM_FRAME.lines().count(), 13);
        assert!(WSL_PM_FRAME.ends_with('\n'));
        assert_eq!(mapping.encode(&carrier).unwrap(), WSL_PM_CARRIER);
        assert_eq!(
            PlatformBootstrapMappingV1::decode(WSL_PM_CARRIER, &carrier).unwrap(),
            mapping
        );
    }

    #[test]
    fn platform_bootstrap_mapping_rejects_noncanonical_or_tampered_records() {
        let carrier = unix_ih_carrier();
        let canonical = LIMA_PM_FRAME;
        let noncanonical_host_path = encode_inner_field("/Users/alice//.lima");
        let noncanonical_guest_path = encode_inner_field("/home/substrate//.substrate");
        let cases = [
            canonical.replacen("version=1\n", "", 1),
            canonical.replacen("version=1\n", "version=1\nversion=1\n", 1),
            canonical.replacen("version=1", "unknown=1", 1),
            canonical.replacen(
                "version=1\nhost_context_commitment=",
                "host_context_commitment=version=1\n",
                1,
            ),
            canonical.replacen("c3Vic3RyYXRl", "***", 1),
            canonical.replacen("c3Vic3RyYXRl", "c3Vic3RyYXRl=", 1),
            canonical.replacen(UNIX_IH_COMMITMENT, &"0".repeat(64), 1),
            canonical.replacen(UNIX_IH_COMMITMENT, "abc", 1),
            canonical.replacen(UNIX_IH_COMMITMENT, &UNIX_IH_COMMITMENT.to_uppercase(), 1),
            canonical.replacen(
                "0123456789abcdef0123456789abcdef",
                "0123456789ABCDEF0123456789ABCDEF",
                1,
            ),
            canonical.replacen("0123456789abcdef0123456789abcdef", "0123456789abcdef", 1),
            canonical.replacen(
                "realized_principal_uid=1000",
                "realized_principal_uid=01000",
                1,
            ),
            canonical.replacen(
                "realized_principal_uid=1000",
                "realized_principal_uid=-1",
                1,
            ),
            canonical.replacen(
                "realized_principal_uid=1000",
                "realized_principal_uid=4294967296",
                1,
            ),
            canonical.replacen(
                "host_platform_control_root=L1VzZXJzL2FsaWNlLy5saW1h",
                &format!("host_platform_control_root={noncanonical_host_path}"),
                1,
            ),
            canonical.replacen(
                "realized_substrate_home=L2hvbWUvc3Vic3RyYXRlLy5zdWJzdHJhdGU",
                &format!("realized_substrate_home={noncanonical_guest_path}"),
                1,
            ),
            canonical.replacen("transport_kind=lima", "transport_kind=wsl", 1),
            canonical.replacen("transport_host=", "extra=field\ntransport_host=", 1),
            canonical.trim_end_matches('\n').to_string(),
            canonical.replace('\n', "\r\n"),
        ];

        for record in cases {
            assert!(
                PlatformBootstrapMappingV1::decode(&encode_ih_record(&record), &carrier).is_err(),
                "accepted tampered record: {record:?}"
            );
        }
        assert!(
            PlatformBootstrapMappingV1::decode(&format!("{LIMA_PM_CARRIER}="), &carrier).is_err()
        );
        let windows_carrier = windows_ih_carrier();
        let noncanonical_windows_path =
            encode_inner_field(r"C:\Users\Alice\..\Alice\AppData\Local\Substrate");
        let noncanonical_windows_record = WSL_PM_FRAME.replacen(
            "host_platform_control_root=QzpcVXNlcnNcQWxpY2VcQXBwRGF0YVxMb2NhbFxTdWJzdHJhdGVcZm9yd2FyZGVyXHNjb3Bl",
            &format!("host_platform_control_root={noncanonical_windows_path}"),
            1,
        );
        assert!(PlatformBootstrapMappingV1::decode(
            &encode_ih_record(&noncanonical_windows_record),
            &windows_carrier,
        )
        .is_err());

        let mut mapping = PlatformBootstrapMappingV1::decode(LIMA_PM_CARRIER, &carrier).unwrap();
        mapping.realized_transport = PlatformTransportIdentityV1::Wsl {
            pipe_path: r"\\.\pipe\substrate-agent".to_string(),
            guest_socket: "/run/substrate.sock".to_string(),
        };
        assert!(mapping.validate(&carrier).is_err());

        let wrong_carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/opt/other", "alice", 1000).unwrap(),
        )
        .unwrap();
        assert!(PlatformBootstrapMappingV1::decode(LIMA_PM_CARRIER, &wrong_carrier).is_err());
    }

    #[test]
    fn windows_forwarder_scope_and_pipe_normalization_are_canonical() {
        assert_eq!(
            normalize_windows_pipe_path(r"\\.\pipe\Substrate-Agent").unwrap(),
            r"\\.\pipe\substrate-agent"
        );
        assert_eq!(
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\Substrate-Agent",
            )
            .unwrap()
            .0,
            WINDOWS_FORWARDER_SCOPE_DIGEST
        );
        assert_eq!(
            windows_forwarder_scope_input(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WINDOWS_FORWARDER_SCOPE_FRAME.as_bytes()
        );
        assert_eq!(
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\SUBSTRATE-AGENT",
            )
            .unwrap()
            .0,
            WINDOWS_FORWARDER_SCOPE_DIGEST
        );

        for invalid in [
            "",
            r"\\.\pipe\",
            r"//./pipe/substrate-agent",
            r"\\server\pipe\substrate-agent",
            r"\\.\PIPE\substrate-agent",
            r"\\.\pipe\nested\name",
            r"\\.\pipe\has space",
            "\\\\.\\pipe\\line\nbreak",
            r"\\.\pipe\name$",
        ] {
            assert!(
                normalize_windows_pipe_path(invalid).is_err(),
                "accepted invalid pipe: {invalid:?}"
            );
        }
        assert!(normalize_windows_pipe_path(&format!(r"\\.\pipe\{}", "a".repeat(129))).is_err());

        let canonical = WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .unwrap();
        for tampered in [
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1001",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL-2",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456780",
                r"\\.\pipe\substrate-agent",
            )
            .unwrap(),
            WindowsForwarderScopeV1::derive(
                "S-1-5-21-1000",
                "Substrate-WSL",
                "abcdef0123456789abcdef0123456789",
                r"\\.\pipe\substrate-agent-2",
            )
            .unwrap(),
        ] {
            assert_ne!(tampered, canonical);
        }

        assert!(WindowsForwarderScopeV1::derive(
            "s-1-5-21-1000",
            "Substrate-WSL",
            "abcdef0123456789abcdef0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .is_err());
        assert!(WindowsForwarderScopeV1::derive(
            "S-1-5-21-1000",
            "Substrate-WSL",
            "ABCDEF0123456789ABCDEF0123456789",
            r"\\.\pipe\substrate-agent",
        )
        .is_err());
    }

    const LAITDP2_CLIENT: &str = "codex";
    const LAITDP2_ROUTER: &str = "substrate_gateway";
    const LAITDP2_PROVIDER: &str = "openai";
    const LAITDP2_AUTH_AUTHORITY: &str = "codex_subscription";
    const LAITDP2_PROTOCOL: &str = "openai.responses";

    #[derive(Clone, Copy)]
    struct Laitdp2PlatformCase {
        platform: &'static str,
        hidden_transport: &'static str,
    }

    const LAITDP2_PLATFORM_CASES: [Laitdp2PlatformCase; 3] = [
        Laitdp2PlatformCase {
            platform: "linux",
            hidden_transport: "unix_socket",
        },
        Laitdp2PlatformCase {
            platform: "macos",
            hidden_transport: "lima_forwarding",
        },
        Laitdp2PlatformCase {
            platform: "windows",
            hidden_transport: "wsl_named_pipe_or_tcp",
        },
    ];

    fn valid_cli_codex_payload() -> GatewayIntegratedAuthPayloadV1 {
        GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:codex".to_string(),
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "header.payload.signature".to_string(),
            }),
            api_env: None,
        }
    }

    fn valid_api_openai_payload() -> GatewayIntegratedAuthPayloadV1 {
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());

        GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:openai".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        }
    }

    fn laitdp2_policy_snapshot_with_restrictive_net_allowed() -> Value {
        json!({
            "schema_version": 3,
            "net_allowed": ["api.openai.com"],
            "world_fs": {
                "host_visible": true,
                "fail_closed": { "routing": false },
                "caged_required": false,
                "write": {
                    "enabled": true,
                    "allow_list": ["."],
                    "deny_list": []
                }
            }
        })
    }

    #[test]
    fn gateway_lifecycle_request_rejects_unknown_fields() {
        let err = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/tmp",
            "env": null,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
                }
            },
            "world_network": null,
            "integrated_auth": null,
            "unexpected": true
        }))
        .expect_err("unknown request field should fail");

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn gateway_lifecycle_response_round_trips_canonical_identity_objects() {
        let response = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "available",
            "client_wiring": {
                "openai_base_url": "http://127.0.0.1:4040",
                "anthropic_base_url": "http://127.0.0.1:4040"
            },
            "identity_tuple": {
                "client": "codex",
                "router": "substrate_gateway",
                "provider": "openai",
                "auth_authority": "codex_subscription",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect("valid lifecycle response should deserialize");

        let roundtrip = serde_json::to_value(&response).expect("serialize lifecycle response");
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/router")
                .and_then(Value::as_str),
            Some("substrate_gateway")
        );
        assert_eq!(
            roundtrip
                .pointer("/placement_posture/execution")
                .and_then(Value::as_str),
            Some("in_world")
        );
    }

    #[test]
    fn gateway_lifecycle_response_rejects_direct_provider_path_with_bridge_transport() {
        let err = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "available",
            "identity_tuple": {
                "client": "codex",
                "router": "direct_provider_path",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "host_only",
                "host_to_world_bridge": true
            }
        }))
        .expect_err("invalid routing/posture combination should fail");

        assert!(err.to_string().contains("host_to_world_bridge"));
    }

    #[test]
    fn gateway_lifecycle_response_keeps_tuple_metadata_top_level_when_unavailable() {
        let response = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "unavailable",
            "identity_tuple": {
                "client": "codex",
                "router": "substrate_gateway",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect("unavailable lifecycle response should keep additive tuple metadata");

        let roundtrip = serde_json::to_value(&response).expect("serialize lifecycle response");
        assert_eq!(roundtrip.pointer("/status"), Some(&json!("unavailable")));
        assert_eq!(roundtrip.pointer("/client_wiring"), None);
        assert_eq!(
            roundtrip.pointer("/identity_tuple/client"),
            Some(&json!("codex"))
        );
        assert_eq!(
            roundtrip.pointer("/placement_posture/execution"),
            Some(&json!("in_world"))
        );
    }

    #[test]
    fn gateway_lifecycle_response_rejects_secret_like_tuple_values() {
        let err = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
            "status": "available",
            "identity_tuple": {
                "client": "codex",
                "router": "substrate_gateway",
                "provider": "https://api.openai.com/v1",
                "auth_authority": "~/.codex/auth.json",
                "protocol": "openai.responses"
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect_err("secret-like tuple metadata should fail validation");

        let error_text = err.to_string();
        assert!(
            error_text.contains("provider") || error_text.contains("auth_authority"),
            "expected validation error to cite the rejected tuple fields, got: {error_text}"
        );
    }

    #[test]
    fn laitdp2_lifecycle_response_publishes_one_tuple_and_posture_meaning_across_platforms() {
        let mut canonical_tuple = None;
        let mut canonical_posture = None;

        for case in LAITDP2_PLATFORM_CASES {
            let response = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
                "status": "available",
                "client_wiring": {
                    "openai_base_url": "http://gateway.test/openai",
                    "anthropic_base_url": "http://gateway.test/anthropic"
                },
                "identity_tuple": {
                    "client": LAITDP2_CLIENT,
                    "router": LAITDP2_ROUTER,
                    "provider": LAITDP2_PROVIDER,
                    "auth_authority": LAITDP2_AUTH_AUTHORITY,
                    "protocol": LAITDP2_PROTOCOL
                },
                "placement_posture": {
                    "execution": "in_world",
                    "host_to_world_bridge": true
                }
            }))
            .unwrap_or_else(|err| {
                panic!(
                    "{} should accept the shared tuple/posture contract despite hidden {} transport: {err}",
                    case.platform, case.hidden_transport
                )
            });

            let roundtrip = serde_json::to_value(response).expect("serialize lifecycle response");
            let tuple = roundtrip
                .pointer("/identity_tuple")
                .cloned()
                .expect("identity tuple is published");
            let posture = roundtrip
                .pointer("/placement_posture")
                .cloned()
                .expect("placement posture is published");

            assert_eq!(
                tuple.pointer("/client").and_then(Value::as_str),
                Some(LAITDP2_CLIENT)
            );
            assert_eq!(
                tuple.pointer("/router").and_then(Value::as_str),
                Some(LAITDP2_ROUTER)
            );
            assert_eq!(
                tuple.pointer("/provider").and_then(Value::as_str),
                Some(LAITDP2_PROVIDER)
            );
            assert_eq!(
                tuple.pointer("/auth_authority").and_then(Value::as_str),
                Some(LAITDP2_AUTH_AUTHORITY)
            );
            assert_eq!(
                tuple.pointer("/protocol").and_then(Value::as_str),
                Some(LAITDP2_PROTOCOL)
            );
            assert_eq!(
                posture.pointer("/execution").and_then(Value::as_str),
                Some("in_world")
            );
            assert_eq!(
                posture
                    .pointer("/host_to_world_bridge")
                    .and_then(Value::as_bool),
                Some(true)
            );

            if let Some(expected) = canonical_tuple.as_ref() {
                assert_eq!(
                    &tuple, expected,
                    "{} must not give tuple vocabulary platform-specific meaning",
                    case.platform
                );
            } else {
                canonical_tuple = Some(tuple);
            }

            if let Some(expected) = canonical_posture.as_ref() {
                assert_eq!(
                    &posture, expected,
                    "{} must not give placement posture platform-specific meaning",
                    case.platform
                );
            } else {
                canonical_posture = Some(posture);
            }

            assert!(
                roundtrip.pointer("/client_wiring/identity_tuple").is_none(),
                "{} must keep additive tuple metadata outside client_wiring: {roundtrip}",
                case.platform
            );
            assert!(
                roundtrip
                    .pointer("/client_wiring/placement_posture")
                    .is_none(),
                "{} must keep additive placement metadata outside client_wiring: {roundtrip}",
                case.platform
            );
        }
    }

    #[test]
    fn laitdp2_direct_provider_path_posture_invariants_are_platform_independent() {
        for case in LAITDP2_PLATFORM_CASES {
            for invalid_posture in [
                json!({ "execution": "in_world" }),
                json!({ "execution": "host_only", "host_to_world_bridge": true }),
            ] {
                let result = serde_json::from_value::<GatewayLifecycleResponseV1>(json!({
                    "status": "available",
                    "identity_tuple": {
                        "client": LAITDP2_CLIENT,
                        "router": "direct_provider_path",
                        "protocol": LAITDP2_PROTOCOL
                    },
                    "placement_posture": invalid_posture
                }));

                assert!(
                    result.is_err(),
                    "{} must reject direct_provider_path unless execution=host_only and bridge transport is absent",
                    case.platform
                );
            }
        }
    }

    #[test]
    fn laitdp2_bridge_transport_does_not_rewrite_router_or_in_world_net_allowed_governance() {
        for case in LAITDP2_PLATFORM_CASES {
            let request = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
                "profile": null,
                "cwd": "/workspace",
                "env": {},
                "agent_id": LAITDP2_CLIENT,
                "policy_snapshot": laitdp2_policy_snapshot_with_restrictive_net_allowed(),
                "world_network": {
                    "isolate_network": true,
                    "allowed_domains": ["api.openai.com"]
                },
                "integrated_auth": null,
                "identity_tuple": {
                    "client": LAITDP2_CLIENT,
                    "router": LAITDP2_ROUTER,
                    "provider": LAITDP2_PROVIDER,
                    "auth_authority": LAITDP2_AUTH_AUTHORITY,
                    "protocol": LAITDP2_PROTOCOL
                },
                "placement_posture": {
                    "execution": "in_world",
                    "host_to_world_bridge": true
                }
            }))
            .unwrap_or_else(|err| {
                panic!(
                    "{} should allow bridge transport only as transport detail: {err}",
                    case.platform
                )
            });

            let roundtrip = serde_json::to_value(request).expect("serialize lifecycle request");
            assert_eq!(
                roundtrip
                    .pointer("/identity_tuple/router")
                    .and_then(Value::as_str),
                Some(LAITDP2_ROUTER),
                "{} must not convert bridge transport into router identity",
                case.platform
            );
            assert_eq!(
                roundtrip
                    .pointer("/policy_snapshot/net_allowed")
                    .and_then(Value::as_array)
                    .expect("net_allowed array"),
                &[json!("api.openai.com")],
                "{} must preserve policy net_allowed when bridge transport participates",
                case.platform
            );
            assert_eq!(
                roundtrip
                    .pointer("/world_network/allowed_domains")
                    .and_then(Value::as_array)
                    .expect("allowed domains array"),
                &[json!("api.openai.com")],
                "{} must preserve runtime network isolation request when bridge transport participates",
                case.platform
            );
        }
    }

    #[test]
    fn laitdp2_backend_id_remains_adapter_selector_not_tuple_semantics() {
        let request = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/workspace",
            "env": {},
            "agent_id": LAITDP2_CLIENT,
            "policy_snapshot": laitdp2_policy_snapshot_with_restrictive_net_allowed(),
            "world_network": null,
            "integrated_auth": {
                "backend_id": "cli:codex",
                "cli_codex": {
                    "account_id": "acct_test",
                    "access_token": "test-token"
                }
            },
            "identity_tuple": {
                "client": LAITDP2_CLIENT,
                "router": LAITDP2_ROUTER,
                "provider": LAITDP2_PROVIDER,
                "auth_authority": LAITDP2_AUTH_AUTHORITY,
                "protocol": LAITDP2_PROTOCOL
            },
            "placement_posture": {
                "execution": "in_world"
            }
        }))
        .expect("backend_id selector should coexist with separate tuple semantics");

        let roundtrip = serde_json::to_value(request).expect("serialize lifecycle request");
        assert_eq!(
            roundtrip
                .pointer("/integrated_auth/backend_id")
                .and_then(Value::as_str),
            Some("cli:codex")
        );
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/client")
                .and_then(Value::as_str),
            Some(LAITDP2_CLIENT)
        );
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/router")
                .and_then(Value::as_str),
            Some(LAITDP2_ROUTER)
        );
        assert_eq!(
            roundtrip
                .pointer("/identity_tuple/auth_authority")
                .and_then(Value::as_str),
            Some(LAITDP2_AUTH_AUTHORITY)
        );
        assert!(
            !roundtrip
                .pointer("/identity_tuple")
                .expect("identity tuple")
                .to_string()
                .contains("cli:codex"),
            "backend_id grammar must not substitute for tuple fields: {roundtrip}"
        );

        let invalid_tuple = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/workspace",
            "env": {},
            "agent_id": LAITDP2_CLIENT,
            "policy_snapshot": laitdp2_policy_snapshot_with_restrictive_net_allowed(),
            "world_network": null,
            "integrated_auth": null,
            "identity_tuple": {
                "client": "cli:codex",
                "router": "direct_provider_path",
                "protocol": LAITDP2_PROTOCOL
            },
            "placement_posture": {
                "execution": "host_only"
            }
        }));
        assert!(
            invalid_tuple.is_err(),
            "tuple fields must reject backend-id grammar so backend_id cannot become semantic identity"
        );
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_unknown_facet_fields() {
        let err = serde_json::from_value::<GatewayIntegratedAuthPayloadV1>(json!({
            "backend_id": "api:openai",
            "api_env": {
                "env": {
                    "OPENAI_API_KEY": "sk-test"
                },
                "unexpected": true
            }
        }))
        .expect_err("unknown facet field should fail");

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_multi_facet_payloads() {
        let mut payload = valid_cli_codex_payload();
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        payload.api_env = Some(GatewayApiEnvIntegratedAuthV1 { env });

        let err = payload
            .validate()
            .expect_err("multi-facet payload should fail");
        assert!(err.contains("exactly one auth facet"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_facet_backend_mismatch() {
        let mut payload = valid_cli_codex_payload();
        payload.backend_id = "api:openai".to_string();

        let err = payload.validate().expect_err("mismatch should fail");
        assert!(err.contains("incompatible auth facet"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_missing_facet_payloads() {
        let payload = GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:openai".to_string(),
            cli_codex: None,
            api_env: None,
        };

        let err = payload
            .validate()
            .expect_err("missing-facet payload should fail");
        assert!(err.contains("exactly one auth facet"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_blank_required_values() {
        let mut payload = valid_api_openai_payload();
        payload
            .api_env
            .as_mut()
            .expect("api_env")
            .env
            .insert("OPENAI_API_KEY".to_string(), "   ".to_string());

        let err = payload
            .validate()
            .expect_err("blank required value should fail");
        assert!(err.contains("empty api_env value"));
    }

    #[test]
    fn gateway_integrated_auth_validation_rejects_invalid_api_env_names() {
        let mut payload = valid_api_openai_payload();
        payload
            .api_env
            .as_mut()
            .expect("api_env")
            .env
            .insert("OPENAI API KEY".to_string(), "sk-test".to_string());

        let err = payload
            .validate()
            .expect_err("invalid api env name should fail");
        assert!(err.contains("invalid api_env env name"));
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_valid_cli_codex() {
        valid_cli_codex_payload()
            .validate()
            .expect("valid cli:codex");
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_realized_cli_codex_backends() {
        for backend_id in ["cli:codex-host", "cli:codex-world"] {
            GatewayIntegratedAuthPayloadV1 {
                backend_id: backend_id.to_string(),
                cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                    account_id: Some("acct_test".to_string()),
                    access_token: "header.payload.signature".to_string(),
                }),
                api_env: None,
            }
            .validate()
            .unwrap_or_else(|err| panic!("valid {backend_id} cli_codex payload: {err}"));
        }
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_valid_api_openai() {
        valid_api_openai_payload()
            .validate()
            .expect("valid api:openai");
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_cli_claude_code_api_env() {
        GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:claude_code".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 {
                env: HashMap::from([("ANTHROPIC_API_KEY".to_string(), "sk-ant-test".to_string())]),
            }),
        }
        .validate()
        .expect("valid cli:claude_code api_env");
    }

    #[test]
    fn gateway_integrated_auth_validation_accepts_realized_cli_claude_code_backends() {
        for backend_id in ["cli:claude_code-host", "cli:claude_code-world"] {
            GatewayIntegratedAuthPayloadV1 {
                backend_id: backend_id.to_string(),
                cli_codex: None,
                api_env: Some(GatewayApiEnvIntegratedAuthV1 {
                    env: HashMap::from([(
                        "ANTHROPIC_API_KEY".to_string(),
                        "sk-ant-test".to_string(),
                    )]),
                }),
            }
            .validate()
            .unwrap_or_else(|err| panic!("valid {backend_id} api_env payload: {err}"));
        }
    }

    #[test]
    fn runtime_identity_v1_types_are_shared_without_transport_aliases() {
        let frame = RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "rts_transport".to_string(),
            frame_sequence: 1,
        };
        let event = RuntimeEventIdentityV1 {
            event_id: "evt_transport".to_string(),
            event_sequence: 1,
        };
        let terminal = RuntimeTerminalIdentityV1::from(&event);

        assert!(frame.validate().is_ok());
        assert!(event.validate().is_ok());
        assert!(terminal.matches_event(&event));
    }

    fn runtime_frame_identity(sequence: u64) -> RuntimeFrameIdentityV1 {
        RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "rts_transport".to_string(),
            frame_sequence: sequence,
        }
    }

    fn runtime_event_identity(sequence: u64) -> RuntimeEventIdentityV1 {
        RuntimeEventIdentityV1 {
            event_id: format!("evt_transport_{sequence}"),
            event_sequence: sequence,
        }
    }

    fn identified_agent_event(sequence: u64) -> AgentEvent {
        let mut event = AgentEvent::message(
            "agent",
            "session",
            "run",
            substrate_common::agent_events::MessageEventKind::Status,
            "ok",
        );
        event.event_identity = Some(runtime_event_identity(sequence));
        event
    }

    #[test]
    fn execute_stream_frame_v1_serializes_one_canonical_identity_representation() {
        let event = ExecuteStreamFrame::Event {
            frame_identity: runtime_frame_identity(2),
            event: identified_agent_event(1),
        };
        let event_json = serde_json::to_value(&event).expect("serialize Event frame");
        assert_eq!(event_json["frame_identity"]["schema_version"], json!(1));
        assert_eq!(
            event_json["frame_identity"]["stream_id"],
            json!("rts_transport")
        );
        assert_eq!(event_json["frame_identity"]["frame_sequence"], json!(2));
        assert_eq!(
            event_json["event"]["event_identity"]["event_sequence"],
            json!(1)
        );
        assert!(event_json.get("event_identity").is_none());

        let terminal_event = runtime_event_identity(2);
        let exit = ExecuteStreamFrame::Exit {
            frame_identity: runtime_frame_identity(3),
            event_identity: terminal_event.clone(),
            terminal_identity: RuntimeTerminalIdentityV1::from(&terminal_event),
            exit: 0,
            span_id: "spn_test".to_string(),
            scopes_used: Vec::new(),
            fs_diff: None,
            process_telemetry: ProcessTelemetry::default(),
        };
        let exit_json = serde_json::to_value(&exit).expect("serialize Exit frame");
        assert_eq!(
            exit_json["event_identity"]["event_id"],
            json!("evt_transport_2")
        );
        assert_eq!(
            exit_json["terminal_identity"]["terminal_event_id"],
            json!("evt_transport_2")
        );

        serde_json::from_value::<ExecuteStreamFrame>(event_json).expect("Event roundtrip");
        serde_json::from_value::<ExecuteStreamFrame>(exit_json).expect("Exit roundtrip");
    }

    #[test]
    fn execute_stream_frame_v1_rejects_missing_malformed_or_conflicting_identity() {
        let valid_event = ExecuteStreamFrame::Event {
            frame_identity: runtime_frame_identity(2),
            event: identified_agent_event(1),
        };
        let mut missing_event_identity = serde_json::to_value(&valid_event).expect("serialize");
        missing_event_identity["event"]
            .as_object_mut()
            .expect("event object")
            .remove("event_identity");
        assert!(serde_json::from_value::<ExecuteStreamFrame>(missing_event_identity).is_err());

        let mut duplicate_event_identity = serde_json::to_value(&valid_event).expect("serialize");
        duplicate_event_identity
            .as_object_mut()
            .expect("frame object")
            .insert(
                "event_identity".to_string(),
                json!({"event_id": "evt_duplicate", "event_sequence": 1}),
            );
        assert!(serde_json::from_value::<ExecuteStreamFrame>(duplicate_event_identity).is_err());

        let mut zero_frame_sequence = serde_json::to_value(&valid_event).expect("serialize");
        zero_frame_sequence["frame_identity"]["frame_sequence"] = json!(0);
        assert!(serde_json::from_value::<ExecuteStreamFrame>(zero_frame_sequence).is_err());

        let terminal_event = runtime_event_identity(2);
        let valid_exit = ExecuteStreamFrame::Exit {
            frame_identity: runtime_frame_identity(3),
            event_identity: terminal_event.clone(),
            terminal_identity: RuntimeTerminalIdentityV1::from(&terminal_event),
            exit: 0,
            span_id: "spn_test".to_string(),
            scopes_used: Vec::new(),
            fs_diff: None,
            process_telemetry: ProcessTelemetry::default(),
        };
        let mut mismatched_terminal = serde_json::to_value(valid_exit).expect("serialize");
        mismatched_terminal["terminal_identity"]["terminal_event_id"] = json!("evt_other");
        assert!(serde_json::from_value::<ExecuteStreamFrame>(mismatched_terminal).is_err());
    }

    #[test]
    fn execute_stream_frame_replay_preserves_identity_and_canonical_bytes() {
        let frame = ExecuteStreamFrame::Event {
            frame_identity: runtime_frame_identity(2),
            event: identified_agent_event(1),
        };
        let original = frame.canonical_ndjson_bytes().expect("canonical bytes");
        let replay = frame
            .clone()
            .canonical_ndjson_bytes()
            .expect("replay bytes");
        assert_eq!(replay, original);

        let transport_error = ExecuteStreamFrame::Error {
            frame_identity: runtime_frame_identity(3),
            message: "transport lost".to_string(),
        };
        assert!(transport_error.terminal_identity().is_none());
    }

    #[test]
    fn serialize_stream_frame_roundtrip() {
        let terminal_event = runtime_event_identity(1);
        let frame = ExecuteStreamFrame::Exit {
            frame_identity: runtime_frame_identity(1),
            event_identity: terminal_event.clone(),
            terminal_identity: RuntimeTerminalIdentityV1::from(&terminal_event),
            exit: 0,
            span_id: "spn_test".into(),
            scopes_used: vec!["tcp:example.com:443".into()],
            fs_diff: None,
            process_telemetry: ProcessTelemetry {
                process_events: vec![ProcessEvent {
                    event_type: ProcessEventType::WorldProcessStart,
                    ts: "2026-04-01T00:00:00Z".into(),
                    ts_unix_ns: 1_743_465_600_000_000_000,
                    session_id: "ses_test".into(),
                    world_id: "wld_test".into(),
                    pid: 42,
                    ppid: 1,
                    cwd: "/tmp".into(),
                    parent_span: "spn_parent".into(),
                    parent_cmd_id: Some("cmd_test".into()),
                    argv: None,
                    argv_omitted: Some(true),
                    exe: None,
                    exit_code: None,
                    signal: None,
                    duration_ms: None,
                    env: None,
                }],
                process_events_status: ProcessEventsStatus::Truncated,
                process_events_reason: Some("capture_overflow".into()),
                process_events_dropped: Some(3),
                process_events_max: None,
                process_events_backend: None,
                process_events_error: None,
            },
        };

        let json = serde_json::to_string(&frame).expect("serialize");
        let back: ExecuteStreamFrame = serde_json::from_str(&json).expect("deserialize");

        match back {
            ExecuteStreamFrame::Exit {
                exit,
                span_id,
                scopes_used,
                fs_diff,
                process_telemetry,
                ..
            } => {
                assert_eq!(exit, 0);
                assert_eq!(span_id, "spn_test");
                assert_eq!(scopes_used, vec!["tcp:example.com:443".to_string()]);
                assert!(fs_diff.is_none());
                assert_eq!(process_telemetry.process_events.len(), 1);
                assert_eq!(
                    process_telemetry.process_events_status,
                    ProcessEventsStatus::Truncated
                );
                assert_eq!(
                    process_telemetry.process_events_reason.as_deref(),
                    Some("capture_overflow")
                );
                assert_eq!(process_telemetry.process_events_dropped, Some(3));
            }
            other => panic!("unexpected frame: {:?}", other),
        }
    }

    #[test]
    fn execute_request_world_fs_mode_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["Github.COM.".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let req = ExecuteRequest {
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            shared_world: Some(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_123".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
            world_network: Some(WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["github.com".to_string()],
            }),
            world_fs_mode: Some(WorldFsMode::ReadOnly),
            member_dispatch: None,
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("read_only"),
            "expected world_fs_mode to serialize"
        );
        assert!(
            json.contains("policy_snapshot"),
            "expected policy_snapshot to serialize"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        assert_eq!(back.world_fs_mode, Some(WorldFsMode::ReadOnly));
        assert_eq!(
            back.shared_world,
            Some(SharedWorldOwnerSpec {
                orchestration_session_id: "orch_123".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            })
        );
        assert_eq!(back.policy_snapshot.schema_version, 3);
        assert_eq!(
            back.policy_snapshot.net_allowed,
            vec!["Github.COM.".to_string()]
        );
        assert_eq!(
            back.world_network,
            Some(WorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["github.com".to_string()],
            })
        );
    }

    #[test]
    fn execute_request_policy_snapshot_round_trip() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["github.com".to_string(), "crates.io".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: Some(WorldFsDenyEnforcementV3::Strict),
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec!["src".to_string()],
                    deny_list: vec!["**/*.pem".to_string()],
                },
            },
        };

        let req = ExecuteRequest {
            profile: None,
            cmd: "echo hi".into(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            shared_world: None,
            world_network: None,
            world_fs_mode: None,
            member_dispatch: None,
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(
            json.contains("policy_snapshot"),
            "expected policy_snapshot to serialize"
        );
        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        let snapshot = back.policy_snapshot;
        assert_eq!(snapshot.schema_version, 3);
        assert!(!snapshot.world_fs.host_visible);
        assert_eq!(
            snapshot.world_fs.deny_enforcement,
            Some(WorldFsDenyEnforcementV3::Strict)
        );
        assert_eq!(
            snapshot.world_fs.read.as_ref().expect("read").allow_list,
            vec!["src".to_string()]
        );
        assert_eq!(
            snapshot.net_allowed,
            vec!["github.com".to_string(), "crates.io".to_string()]
        );
    }

    #[test]
    fn execute_response_shared_world_round_trip() {
        let response = ExecuteResponse {
            exit: 0,
            span_id: "spn_123".into(),
            stdout_b64: "aGVsbG8=".into(),
            stderr_b64: String::new(),
            scopes_used: vec!["github.com".into()],
            fs_diff: None,
            shared_world: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch_123".into(),
                world_id: "wld_123".into(),
                world_generation: 3,
                binding_state: SharedWorldBindingState::Active,
            }),
            process_telemetry: ProcessTelemetry::default(),
        };

        let json = serde_json::to_string(&response).expect("serialize response");
        assert!(
            json.contains("\"shared_world\""),
            "expected shared_world to serialize"
        );
        let back: ExecuteResponse =
            serde_json::from_str(&json).expect("deserialize execute response");
        assert_eq!(
            back.shared_world,
            Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch_123".into(),
                world_id: "wld_123".into(),
                world_generation: 3,
                binding_state: SharedWorldBindingState::Active,
            })
        );
    }

    #[test]
    fn execute_cancel_request_round_trip() {
        let req = ExecuteCancelRequestV1 {
            span_id: "spn_cancel".to_string(),
            sig: "INT".to_string(),
        };

        let json = serde_json::to_string(&req).expect("serialize cancel request");
        let back: ExecuteCancelRequestV1 =
            serde_json::from_str(&json).expect("deserialize cancel request");
        assert_eq!(back, req);
    }

    fn test_absolute_binary_path() -> String {
        std::env::current_exe()
            .expect("current_exe")
            .to_string_lossy()
            .into_owned()
    }

    fn sample_retained_worker_launch_authority_proof() -> RetainedWorkerLaunchAuthorityProofV1 {
        RetainedWorkerLaunchAuthorityProofV1 {
            schema_version: 1,
            authority_store_id: "store_123".into(),
            issuer_request_id: "request_123".into(),
            canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
                schema_version: 1,
                algorithm: "hmac-sha-256".into(),
                key_id: "admission_key_123".into(),
                digest_hex: "a".repeat(64),
            },
            registration_id: "registration_123".into(),
            registration_commitment: RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "b".repeat(64),
            },
            authority_revision_after: 7,
            authority_record_commitment_after:
                RetainedWorkerAuthorityObjectCommitmentV1::StoreHmacSha256 {
                    key_id: "authority_key_123".into(),
                    domain: "substrate.host-session-authority.authority-record.v1".into(),
                    digest_hex: "c".repeat(64),
                },
            orchestration_session_id: "orch_123".into(),
            caller_participant_id: "ash_orch_123".into(),
            retained_participant_id: "ash_member_123".into(),
            bootstrap_run_id: "run_123".into(),
            transport_claim_id: "transport_claim_123".into(),
            backend_id: "cli:codex".into(),
            protocol: "substrate.agent.session".into(),
            world_binding: RetainedWorkerLaunchWorldBindingV1 {
                world_id: "world_123".into(),
                world_generation: 7,
            },
            current_policy_ref_id: "policy_ref_123".into(),
            current_policy_revision: "policy_revision_123".into(),
            retained_worker_ref_id: "retained_worker_ref_123".into(),
            retained_worker_commitment:
                RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "d".repeat(64),
                },
        }
    }

    #[test]
    fn retained_worker_launch_authority_proof_is_closed_and_strictly_validated() {
        let proof = sample_retained_worker_launch_authority_proof();
        proof.validate().expect("valid launch-authority proof");

        let json = serde_json::to_value(&proof).expect("serialize launch-authority proof");
        let decoded: RetainedWorkerLaunchAuthorityProofV1 =
            serde_json::from_value(json.clone()).expect("deserialize launch-authority proof");
        assert_eq!(decoded, proof);

        let mut changed_algorithm = json.clone();
        changed_algorithm["canonical_spawn_fingerprint"]["algorithm"] =
            serde_json::json!("sha-256");
        assert!(
            serde_json::from_value::<RetainedWorkerLaunchAuthorityProofV1>(changed_algorithm)
                .is_err()
        );

        let mut uppercase_digest = json.clone();
        uppercase_digest["canonical_spawn_fingerprint"]["digest_hex"] =
            serde_json::json!("A".repeat(64));
        assert!(
            serde_json::from_value::<RetainedWorkerLaunchAuthorityProofV1>(uppercase_digest)
                .is_err()
        );

        let mut unknown_field = json;
        unknown_field["unexpected"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<RetainedWorkerLaunchAuthorityProofV1>(unknown_field).is_err()
        );
    }

    #[test]
    fn execute_request_member_dispatch_round_trip() {
        let binary_path = test_absolute_binary_path();
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: None,
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let req = ExecuteRequest {
            profile: None,
            cmd: String::new(),
            cwd: Some("/tmp".into()),
            env: None,
            pty: false,
            agent_id: "tester".into(),
            budget: None,
            policy_snapshot: snapshot,
            shared_world: None,
            world_network: None,
            world_fs_mode: None,
            member_dispatch: Some(MemberDispatchRequestV1 {
                schema_version: 1,
                orchestration_session_id: "orch_123".into(),
                participant_id: "ash_member_123".into(),
                orchestrator_participant_id: "ash_orch_123".into(),
                parent_participant_id: Some("ash_parent_123".into()),
                resumed_from_participant_id: Some("ash_old_123".into()),
                backend_id: "cli:codex".into(),
                protocol: "substrate.agent.session".into(),
                run_id: "run_123".into(),
                world_id: "world_123".into(),
                world_generation: 7,
                initial_prompt: Some("first turn".into()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: binary_path.clone(),
                },
                retained_worker_launch_authority: None,
            }),
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize request");
        assert!(json.contains("\"member_dispatch\""));
        assert!(!json.contains("\"acceptance_context\""));
        assert!(
            !json.contains("retained_worker_launch_authority"),
            "legacy/Run/Fork None compatibility must preserve the pre-carrier wire shape"
        );

        let mut acceptance_context = sample_world_work_acceptance_context();
        acceptance_context.request_id = "run_123".to_string();
        acceptance_context.message_id = None;
        let mut with_acceptance = serde_json::to_value(&req).expect("shape execute request");
        with_acceptance["acceptance_context"] =
            serde_json::to_value(&acceptance_context).expect("shape task acceptance context");
        let accepted: ExecuteRequest = serde_json::from_value(with_acceptance.clone())
            .expect("decode task acceptance context");
        assert_eq!(accepted.acceptance_context, Some(acceptance_context));

        let mut wrong_request = with_acceptance.clone();
        wrong_request["acceptance_context"]["request_id"] =
            serde_json::Value::String("run_other".to_string());
        assert!(serde_json::from_value::<ExecuteRequest>(wrong_request).is_err());

        with_acceptance["acceptance_context"]["message_id"] =
            serde_json::Value::String("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string());
        assert!(serde_json::from_value::<ExecuteRequest>(with_acceptance).is_err());

        let back: ExecuteRequest = serde_json::from_str(&json).expect("deserialize request");
        assert!(back.cmd.is_empty());
        assert_eq!(
            back.member_dispatch,
            Some(MemberDispatchRequestV1 {
                schema_version: 1,
                orchestration_session_id: "orch_123".into(),
                participant_id: "ash_member_123".into(),
                orchestrator_participant_id: "ash_orch_123".into(),
                parent_participant_id: Some("ash_parent_123".into()),
                resumed_from_participant_id: Some("ash_old_123".into()),
                backend_id: "cli:codex".into(),
                protocol: "substrate.agent.session".into(),
                run_id: "run_123".into(),
                world_id: "world_123".into(),
                world_generation: 7,
                initial_prompt: Some("first turn".into()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path,
                },
                retained_worker_launch_authority: None,
            })
        );
    }

    #[test]
    fn member_turn_submit_request_round_trip() {
        let req = MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_123".into(),
            participant_id: "ash_member_123".into(),
            orchestrator_participant_id: "ash_orch_123".into(),
            backend_id: "cli:codex".into(),
            run_id: "run_123".into(),
            world_id: "world_123".into(),
            world_generation: 7,
            prompt: "summarize the failure".into(),
            acceptance_context: None,
        };

        let json = serde_json::to_string(&req).expect("serialize member turn submit request");
        let back: MemberTurnSubmitRequestV1 =
            serde_json::from_str(&json).expect("deserialize member turn submit request");
        assert_eq!(back, req);
    }

    fn sample_world_work_acceptance_context() -> WorldWorkAcceptanceContextV1 {
        WorldWorkAcceptanceContextV1 {
            schema_version: 1,
            proposed_acceptance_record_id: "wwa_018f0f3a-9b2c-7def-8abc-0123456789ab".to_string(),
            request_id: "request-123".to_string(),
            message_id: Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string()),
            caller_backend_id: "cli:codex".to_string(),
            host_transition_correlation: None,
        }
    }

    #[test]
    fn world_work_acceptance_context_v1_round_trips_exact_schema() {
        let context = sample_world_work_acceptance_context();
        let json = serde_json::to_value(&context).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["request_id"], "request-123");
        assert_eq!(
            serde_json::from_value::<WorldWorkAcceptanceContextV1>(json).unwrap(),
            context
        );
    }

    #[test]
    fn world_work_acceptance_context_v1_rejects_malformed_or_unknown_fields() {
        let context = sample_world_work_acceptance_context();
        let mut wrong_version = serde_json::to_value(&context).unwrap();
        wrong_version["schema_version"] = serde_json::Value::from(2);
        assert!(serde_json::from_value::<WorldWorkAcceptanceContextV1>(wrong_version).is_err());

        let mut malformed_id = serde_json::to_value(&context).unwrap();
        malformed_id["proposed_acceptance_record_id"] =
            serde_json::Value::String("wwa_not-a-uuid".to_string());
        assert!(serde_json::from_value::<WorldWorkAcceptanceContextV1>(malformed_id).is_err());

        let mut unknown = serde_json::to_value(&context).unwrap();
        unknown["accepted"] = serde_json::Value::Bool(true);
        assert!(serde_json::from_value::<WorldWorkAcceptanceContextV1>(unknown).is_err());
    }

    #[test]
    fn optional_acceptance_context_is_omitted_for_legacy_requests() {
        let mut turn_json = serde_json::to_value(MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_123".into(),
            participant_id: "ash_member_123".into(),
            orchestrator_participant_id: "ash_orch_123".into(),
            backend_id: "cli:codex".into(),
            run_id: "request-123".into(),
            world_id: "world_123".into(),
            world_generation: 7,
            prompt: "resume".into(),
            acceptance_context: None,
        })
        .unwrap();
        assert!(turn_json.get("acceptance_context").is_none());
        turn_json["acceptance_context"] =
            serde_json::to_value(sample_world_work_acceptance_context()).unwrap();
        let decoded: MemberTurnSubmitRequestV1 = serde_json::from_value(turn_json).unwrap();
        assert_eq!(
            decoded.acceptance_context,
            Some(sample_world_work_acceptance_context())
        );
    }

    #[test]
    fn member_turn_submit_rejects_invalid_backend_id() {
        let err = serde_json::from_value::<MemberTurnSubmitRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_orch_123",
            "backend_id": "codex",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "prompt": "resume"
        }))
        .expect_err("invalid backend id should fail");

        assert!(
            err.to_string()
                .contains("invalid member_turn_submit.backend_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn member_turn_submit_rejects_empty_prompt() {
        let err = serde_json::from_value::<MemberTurnSubmitRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_orch_123",
            "backend_id": "cli:codex",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "prompt": "   "
        }))
        .expect_err("blank prompt should fail");

        assert!(
            err.to_string()
                .contains("member_turn_submit.prompt must not be empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_empty_process_exec_cmd_at_boundary() {
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "profile": null,
            "cmd": "   ",
            "cwd": "/tmp",
            "pty": false,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            }
        }))
        .expect_err("empty process cmd should fail");

        assert!(
            err.to_string().contains("non-empty cmd"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_member_dispatch_with_non_empty_cmd_at_boundary() {
        let binary_path = test_absolute_binary_path();
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "cmd": "echo hi",
            "cwd": "/tmp",
            "pty": false,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            },
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "orch_123",
                "participant_id": "ash_member_123",
                "orchestrator_participant_id": "ash_orch_123",
                "backend_id": "cli:codex",
                "protocol": "substrate.agent.session",
                "run_id": "run_123",
                "world_id": "world_123",
                "world_generation": 7,
                "resolved_runtime": {
                    "backend_kind": "codex",
                    "binary_path": binary_path
                }
            }
        }))
        .expect_err("mixed cmd + member dispatch should fail");

        assert!(
            err.to_string()
                .contains("member_dispatch requires cmd.trim().is_empty()"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_member_dispatch_with_pty_at_boundary() {
        let binary_path = test_absolute_binary_path();
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "cmd": "",
            "cwd": "/tmp",
            "pty": true,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            },
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "orch_123",
                "participant_id": "ash_member_123",
                "orchestrator_participant_id": "ash_orch_123",
                "backend_id": "cli:codex",
                "protocol": "substrate.agent.session",
                "run_id": "run_123",
                "world_id": "world_123",
                "world_generation": 7,
                "resolved_runtime": {
                    "backend_kind": "codex",
                    "binary_path": binary_path
                }
            }
        }))
        .expect_err("pty member dispatch should fail");

        assert!(
            err.to_string()
                .contains("member_dispatch requires pty=false"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn member_dispatch_rejects_self_referential_lineage() {
        let binary_path = test_absolute_binary_path();
        let err = serde_json::from_value::<MemberDispatchRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_member_123",
            "backend_id": "cli:codex",
            "protocol": "substrate.agent.session",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "resolved_runtime": {
                "backend_kind": "codex",
                "binary_path": binary_path
            }
        }))
        .expect_err("self-referential lineage should fail");

        assert!(
            err.to_string()
                .contains("orchestrator_participant_id must not equal participant_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_request_rejects_member_dispatch_without_resolved_runtime_at_boundary() {
        let err = serde_json::from_value::<ExecuteRequest>(serde_json::json!({
            "cmd": "",
            "cwd": "/tmp",
            "pty": false,
            "agent_id": "tester",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": {
                        "enabled": true,
                        "allow_list": ["."],
                        "deny_list": []
                    }
                }
            },
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "orch_123",
                "participant_id": "ash_member_123",
                "orchestrator_participant_id": "ash_orch_123",
                "backend_id": "cli:codex",
                "protocol": "substrate.agent.session",
                "run_id": "run_123",
                "world_id": "world_123",
                "world_generation": 7
            }
        }))
        .expect_err("member dispatch without resolved_runtime should fail");

        assert!(
            err.to_string().contains("missing field `resolved_runtime`"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn member_dispatch_rejects_non_absolute_resolved_runtime_binary_path() {
        let err = serde_json::from_value::<MemberDispatchRequestV1>(serde_json::json!({
            "schema_version": 1,
            "orchestration_session_id": "orch_123",
            "participant_id": "ash_member_123",
            "orchestrator_participant_id": "ash_orch_123",
            "backend_id": "cli:codex",
            "protocol": "substrate.agent.session",
            "run_id": "run_123",
            "world_id": "world_123",
            "world_generation": 7,
            "resolved_runtime": {
                "backend_kind": "codex",
                "binary_path": "codex"
            }
        }))
        .expect_err("relative binary path should fail");

        assert!(
            err.to_string()
                .contains("member_dispatch.resolved_runtime.binary_path must be an absolute path"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn execute_cancel_response_defaults_schema_version() {
        let response: ExecuteCancelResponseV1 = serde_json::from_value(serde_json::json!({
            "delivered": true
        }))
        .expect("deserialize cancel response");

        assert_eq!(response.schema_version, 1);
        assert!(response.delivered);
    }

    #[test]
    fn policy_snapshot_v3_missing_net_allowed_defaults_to_empty() {
        let snapshot: PolicySnapshotV3 = serde_json::from_value(serde_json::json!({
            "schema_version": 3,
            "world_fs": {
                "host_visible": true,
                "fail_closed": { "routing": false },
                "caged_required": false,
                "discover": { "allow_list": ["."], "deny_list": [] },
                "read": { "allow_list": ["."], "deny_list": [] },
                "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
            }
        }))
        .expect("deserialize snapshot");

        assert!(snapshot.net_allowed.is_empty());
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_canonicalizes_trim_case_trailing_dot_and_dedupe() {
        let canonical = canonicalize_net_allowed(&[
            " GitHub.COM. ".to_string(),
            "github.com".to_string(),
            "CRATES.IO".to_string(),
            "".to_string(),
            "   ".to_string(),
            "crates.io.".to_string(),
        ]);

        assert_eq!(
            canonical,
            vec!["github.com".to_string(), "crates.io".to_string()]
        );
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_collapses_star_to_singleton() {
        let canonical = canonicalize_net_allowed(&[
            "github.com".to_string(),
            " * ".to_string(),
            "crates.io".to_string(),
        ]);

        assert_eq!(canonical, vec!["*".to_string()]);
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_canonicalizes_bracketed_ipv6() {
        let canonical = canonicalize_net_allowed(&[" [2001:DB8::1] ".to_string()]);
        assert_eq!(canonical, vec!["2001:db8::1".to_string()]);
    }

    #[test]
    fn policy_snapshot_v3_validate_does_not_reject_enforcement_only_net_allowed_shapes() {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec![
                "*.example.com".to_string(),
                "https://example.com".to_string(),
            ],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        snapshot.validate().expect("snapshot validates");
    }

    #[test]
    fn policy_snapshot_v3_resolve_world_network_routing_matches_four_case_matrix() {
        let allow_all = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["*".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let restrictive = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec![" Example.COM. ".to_string(), "api.example.com".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let deny_all = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        let allow_all_disabled = allow_all
            .resolve_world_network_routing(false)
            .expect("allow-all with filter off");
        assert!(!allow_all_disabled.isolate_network);
        assert!(allow_all_disabled.allowed_domains.is_empty());

        let allow_all_enabled = allow_all
            .resolve_world_network_routing(true)
            .expect("allow-all with filter on");
        assert!(!allow_all_enabled.isolate_network);
        assert!(allow_all_enabled.allowed_domains.is_empty());

        let deny_all_enabled = deny_all
            .resolve_world_network_routing(true)
            .expect("deny-all with filter on");
        assert!(deny_all_enabled.isolate_network);
        assert!(deny_all_enabled.allowed_domains.is_empty());

        let restrictive_enabled = restrictive
            .resolve_world_network_routing(true)
            .expect("restrictive with filter on");
        assert!(restrictive_enabled.isolate_network);
        assert_eq!(
            restrictive_enabled.allowed_domains,
            vec!["example.com".to_string(), "api.example.com".to_string()]
        );
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_enforcement_validator_rejects_invalid_shapes() {
        for entries in [
            vec!["*.example.com".to_string()],
            vec!["https://example.com".to_string()],
            vec!["example.com:443".to_string()],
            vec!["example.com/path".to_string()],
            vec!["example.com?query".to_string()],
            vec!["example.com#fragment".to_string()],
            vec!["bücher.example".to_string()],
        ] {
            assert!(
                validate_net_allowed_for_enforcement(&entries).is_err(),
                "expected invalid net_allowed entries to fail: {entries:?}"
            );
        }
    }

    #[test]
    fn policy_snapshot_v3_net_allowed_enforcement_validator_accepts_punycode_and_ips() {
        validate_net_allowed_for_enforcement(&[
            "XN--BCHER-KVA.EXAMPLE.".to_string(),
            "1.2.3.4".to_string(),
            "[2001:db8::1]".to_string(),
        ])
        .expect("valid entries");
    }

    #[test]
    fn world_doctor_report_v1_schema_round_trip() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            selected_host_prefix: Some("/opt/substrate".to_string()),
            host_context_commitment: Some("a".repeat(64)),
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: true,
                world_netfilter_enable_present: true,
                last_failure_reason: Some(
                    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules"
                        .to_string(),
                ),
            }),
            landlock: super::WorldDoctorLandlockV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: super::WorldDoctorWorldFsStrategyV1 {
                primary: super::WorldDoctorWorldFsStrategyKindV1::Overlay,
                fallback: super::WorldDoctorWorldFsStrategyKindV1::Fuse,
                probe: super::WorldDoctorWorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: super::WorldDoctorWorldFsStrategyProbeResultV1::Pass,
                    failure_reason: None,
                },
            },
        };

        let json = serde_json::to_string(&report).expect("serialize report");
        let back: super::WorldDoctorReportV1 =
            serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(back.schema_version, report.schema_version);
        assert_eq!(back.ok, report.ok);
        assert_eq!(back.collected_at_utc, report.collected_at_utc);
        assert_eq!(back.selected_host_prefix, report.selected_host_prefix);
        assert_eq!(back.host_context_commitment, report.host_context_commitment);
        assert_eq!(
            back.policy_snapshot_v1_supported,
            report.policy_snapshot_v1_supported
        );
        assert_eq!(back.policy_resolution_mode, report.policy_resolution_mode);
        assert_eq!(back.netfilter_status, report.netfilter_status);
        assert_eq!(back.landlock.supported, report.landlock.supported);
        assert_eq!(back.landlock.abi, report.landlock.abi);
        assert_eq!(back.landlock.reason, report.landlock.reason);
        assert_eq!(
            back.world_fs_strategy.primary,
            report.world_fs_strategy.primary
        );
        assert_eq!(
            back.world_fs_strategy.fallback,
            report.world_fs_strategy.fallback
        );
        assert_eq!(
            back.world_fs_strategy.probe.id,
            report.world_fs_strategy.probe.id
        );
        assert_eq!(
            back.world_fs_strategy.probe.probe_file,
            report.world_fs_strategy.probe.probe_file
        );
        assert_eq!(
            back.world_fs_strategy.probe.result,
            report.world_fs_strategy.probe.result
        );
        assert_eq!(
            back.world_fs_strategy.probe.failure_reason,
            report.world_fs_strategy.probe.failure_reason
        );
    }

    #[test]
    fn world_doctor_report_v1_serializes_null_last_failure_reason_when_absent() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            selected_host_prefix: None,
            host_context_commitment: None,
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: None,
            }),
            landlock: super::WorldDoctorLandlockV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: super::WorldDoctorWorldFsStrategyV1 {
                primary: super::WorldDoctorWorldFsStrategyKindV1::Overlay,
                fallback: super::WorldDoctorWorldFsStrategyKindV1::Fuse,
                probe: super::WorldDoctorWorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: super::WorldDoctorWorldFsStrategyProbeResultV1::Pass,
                    failure_reason: None,
                },
            },
        };

        let value = serde_json::to_value(&report).expect("serialize report");
        assert_eq!(
            value["netfilter_status"]["last_failure_reason"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn world_doctor_report_v1_serializes_exact_netfilter_status_field_names() {
        let report = super::WorldDoctorReportV1 {
            schema_version: 2,
            ok: true,
            collected_at_utc: "2026-01-08T00:00:00Z".to_string(),
            selected_host_prefix: None,
            host_context_commitment: None,
            policy_snapshot_v1_supported: true,
            policy_resolution_mode: Some(super::PolicyResolutionModeV1::SnapshotV3),
            netfilter_status: Some(super::WorldDoctorNetfilterStatusV1 {
                requested: true,
                enabled: false,
                world_netfilter_enable_present: false,
                last_failure_reason: None,
            }),
            landlock: super::WorldDoctorLandlockV1 {
                supported: true,
                abi: Some(3),
                reason: None,
            },
            world_fs_strategy: super::WorldDoctorWorldFsStrategyV1 {
                primary: super::WorldDoctorWorldFsStrategyKindV1::Overlay,
                fallback: super::WorldDoctorWorldFsStrategyKindV1::Fuse,
                probe: super::WorldDoctorWorldFsStrategyProbeV1 {
                    id: "enumeration_v1".to_string(),
                    probe_file: ".substrate_enum_probe".to_string(),
                    result: super::WorldDoctorWorldFsStrategyProbeResultV1::Pass,
                    failure_reason: None,
                },
            },
        };

        let value = serde_json::to_value(&report).expect("serialize report");
        let netfilter_status = value["netfilter_status"]
            .as_object()
            .expect("netfilter_status should serialize as object");
        assert_eq!(netfilter_status.len(), 4);
        assert!(netfilter_status.contains_key("requested"));
        assert!(netfilter_status.contains_key("enabled"));
        assert!(netfilter_status.contains_key("world_netfilter_enable_present"));
        assert!(netfilter_status.contains_key("last_failure_reason"));
    }

    #[test]
    fn world_doctor_report_v1_defaults_snapshot_fields_when_missing() {
        // Legacy world-services may omit snapshot fields; the client schema must default safely.
        let json = r#"{
            "schema_version": 1,
            "ok": true,
            "collected_at_utc": "2026-01-08T00:00:00Z",
            "landlock": { "supported": true, "abi": 3, "reason": null },
            "world_fs_strategy": {
                "primary": "overlay",
                "fallback": "fuse",
                "probe": {
                    "id": "enumeration_v1",
                    "probe_file": ".substrate_enum_probe",
                    "result": "pass",
                    "failure_reason": null
                }
            }
        }"#;

        let report: super::WorldDoctorReportV1 = serde_json::from_str(json).expect("deserialize");
        assert!(report.ok);
        assert!(!report.policy_snapshot_v1_supported);
        assert!(report.policy_resolution_mode.is_none());
        assert!(report.netfilter_status.is_none());
        assert!(report.selected_host_prefix.is_none());
        assert!(report.host_context_commitment.is_none());
    }

    #[test]
    fn world_doctor_report_v1_defaults_last_failure_reason_when_missing() {
        let json = r#"{
            "schema_version": 2,
            "ok": true,
            "collected_at_utc": "2026-01-08T00:00:00Z",
            "policy_snapshot_v1_supported": true,
            "policy_resolution_mode": "snapshot_v3",
            "netfilter_status": {
                "requested": true,
                "enabled": false,
                "world_netfilter_enable_present": false
            },
            "landlock": { "supported": true, "abi": 3, "reason": null },
            "world_fs_strategy": {
                "primary": "overlay",
                "fallback": "fuse",
                "probe": {
                    "id": "enumeration_v1",
                    "probe_file": ".substrate_enum_probe",
                    "result": "pass",
                    "failure_reason": null
                }
            }
        }"#;

        let report: super::WorldDoctorReportV1 = serde_json::from_str(json).expect("deserialize");
        let status = report.netfilter_status.expect("netfilter status");
        assert!(status.requested);
        assert!(!status.enabled);
        assert!(!status.world_netfilter_enable_present);
        assert!(status.last_failure_reason.is_none());
    }

    #[test]
    fn execute_stream_replay_request_round_trips_only_exact_identity() {
        let request = super::ExecuteStreamReplayRequestV1 {
            schema_version: 1,
            acceptance_record_id: "wwa_exact-replay".to_string(),
            stream_id: "rts_exact-replay".to_string(),
            after_frame_sequence: 17,
        };
        request.validate().expect("validate exact replay request");
        let bytes = serde_json::to_vec(&request).expect("serialize exact replay request");
        let decoded: super::ExecuteStreamReplayRequestV1 =
            serde_json::from_slice(&bytes).expect("decode exact replay request");
        assert_eq!(decoded, request);

        for invalid in [
            super::ExecuteStreamReplayRequestV1 {
                schema_version: 2,
                ..request.clone()
            },
            super::ExecuteStreamReplayRequestV1 {
                acceptance_record_id: "session-only".to_string(),
                ..request.clone()
            },
            super::ExecuteStreamReplayRequestV1 {
                stream_id: "rts_other ".to_string(),
                ..request
            },
        ] {
            assert!(invalid.validate().is_err());
        }
    }
}
