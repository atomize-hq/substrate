use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    HostRetirementStateV2, HostTargetRoleV2, MAC_R3_COORDINATOR_INBOX_ROOT_V2,
    MAC_R3_COORDINATOR_PATH_V2, MAC_R3_FINALIZER_ENDPOINT_V2, MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
    MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2, MAC_R3_FINALIZER_PLIST_PATH_V2,
    MAC_R3_FINALIZER_PROTOCOL_OWNER_V2, MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
    MAC_R3_FINALIZER_REQUEST_PATH_V2, MAC_R3_TERMINAL_BINDING_PATH_V2,
};

pub const FRAME_PREFIX_BYTES: usize = 8;
pub const FRAME_MAX_BYTES: usize = 1_048_576;
pub const LISTENER_SOCKET_NAME: &str = "Listener";
pub const ENDPOINT_OWNER_UID: u32 = 0;
pub const ENDPOINT_GROUP_GID: u32 = 20;
pub const ENDPOINT_MODE: u32 = 0o660;
pub const FINALIZER_UID: u32 = 0;
pub const FINALIZER_ACCOUNT: &str = "root";
pub const FIXED_STDIN_PATH: &str = "/dev/null";
pub const FIXED_CWD: &str = "/";
pub const JOURNAL_OWNER: &str = "substrate.r3-macos-finalizer-journal";
pub const JOURNAL_VERSION: u32 = 2;
/// One initial invocation plus one observe-before-reinvoke recovery attempt.
///
/// Every attempt has its own immutable `EffectInvoked` generation. Exhausting this fixed budget
/// produces a preserving stop before another system call, so an accepted claim can never grow its
/// journal without bound or perform an unjournaled invocation.
pub const MAX_EFFECT_INVOCATION_ATTEMPTS_V2: u16 = 2;
pub const CAPABILITY_GATE_OWNER: &str = "substrate.r3-macos-finalizer-capability-gate";
pub const CAPABILITY_GATE_VERSION: u32 = 1;
pub const TERMINAL_BINDING_OWNER: &str = "substrate.r3-macos-finalizer-terminal-binding-request";
pub const TERMINAL_BINDING_VERSION: u32 = 1;
pub const CAPABILITY_GATE_PATH: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/capability/gate.v1.json";
pub const TERMINAL_BINDING_PATH: &str = MAC_R3_TERMINAL_BINDING_PATH_V2;

pub fn validate_compiled_literals() -> Result<()> {
    if MAC_R3_FINALIZER_PROTOCOL_OWNER_V2 != "substrate.r3-macos-evidence-finalizer"
        || MAC_R3_FINALIZER_PROTOCOL_VERSION_V2 != 2
        || MAC_R3_FINALIZER_PATH_V2
            != "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-evidence-finalizer.v2"
        || MAC_R3_COORDINATOR_PATH_V2
            != "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator"
        || MAC_R3_FINALIZER_LAUNCHD_LABEL_V2
            != "com.atomize.substrate.r3-macos-evidence-finalizer.v2"
        || MAC_R3_FINALIZER_PLIST_PATH_V2
            != "/Library/LaunchDaemons/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist"
        || MAC_R3_FINALIZER_ENDPOINT_V2
            != "/private/var/run/com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock"
        || MAC_R3_FINALIZER_JOURNAL_ROOT_V2
            != "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2"
        || MAC_R3_COORDINATOR_INBOX_ROOT_V2
            != "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox"
        || MAC_R3_FINALIZER_REQUEST_PATH_V2
            != "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox/finalization-request.v2.json"
        || MAC_R3_TERMINAL_BINDING_PATH_V2
            != "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox/terminal-binding-request.v1.json"
    {
        bail!("compiled R3 finalizer literals drifted")
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JournalEventKind {
    FinalizerAccepted,
    EffectPrepared,
    EffectInvoked,
    EffectObserved,
    EffectsComplete,
    TerminalAcknowledgementBound,
    Complete,
    PreservingStop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JournalEvent {
    pub kind: JournalEventKind,
    pub host_state: HostRetirementStateV2,
    pub effect_ordinal: Option<u16>,
    pub effect_role: Option<HostTargetRoleV2>,
    pub effect_identity_sha256: Option<String>,
    /// Zero for `EffectPrepared` and an already-final `EffectObserved`; otherwise the exact
    /// one-based immutable invocation attempt carried by `EffectInvoked`/`EffectObserved`.
    pub effect_invocation_attempt: Option<u16>,
    pub observation_sha256: Option<String>,
    pub response_sha256: Option<String>,
    pub preserving_classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TerminalBindingRequest {
    pub schema_owner: String,
    pub schema_version: u32,
    pub request_digest: String,
    pub authority_request: String,
    pub effects_response: String,
    pub parity_proof: String,
    pub terminal_acknowledgement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeerAttestation {
    pub schema_owner: String,
    pub schema_version: u32,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub canonical_account: String,
    pub pid: i32,
    pub audit_token_sha256: String,
    pub process_start_sha256: String,
    pub executable_path: String,
    pub executable_physical_identity_sha256: String,
    pub executable_sha256: String,
    pub cdhash: String,
}
