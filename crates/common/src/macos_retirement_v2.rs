//! Closed schemas and pure state-machine rules for the prospective R3 macOS evidence finalizer.
//!
//! This module deliberately contains no native effects. The external evidence finalizer consumes
//! these canonical documents, but target paths, Keychain predicates, launchd labels, and effect
//! operations are derived from closed roles rather than decoded from a caller-supplied action.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{
    de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

use crate::managed_artifact::{verify_ed25519_fixed_v1, verify_p256_p1363_low_s_v1};

pub const MAC_R3_FINALIZER_PROTOCOL_OWNER_V2: &str = "substrate.r3-macos-evidence-finalizer";
pub const MAC_R3_FINALIZER_PROTOCOL_VERSION_V2: u32 = 2;
pub const MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2: usize = 1024 * 1024;
pub const MAC_R3_HOST_RECEIPT_OWNER_V2: &str = "substrate.r3-macos-host-pre-removal-receipt";
pub const MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2: &str = "R3_MAC_HOST_PRE_REMOVAL_RECEIPT_V2";
pub const MAC_R3_HARNESS_ACK_OWNER_V2: &str =
    "substrate.r3-macos-receipt-durability-acknowledgement";
pub const MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2: &str =
    "R3_MAC_RECEIPT_DURABILITY_ACKNOWLEDGEMENT_V2";
pub const MAC_R3_PROTECTED_CAS_OWNER_V2: &str = "substrate.r3-macos-retirement-protected-cas";
pub const MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2: &str = "R3_MAC_RETIREMENT_PROTECTED_CAS_V2";
pub const MAC_R3_GUEST_TO_HOST_CAPSULE_OWNER_V2: &str =
    "substrate.r3-macos-guest-to-host-successor-capsule";
pub const MAC_R3_GUEST_RECEIPT_OWNER_V2: &str = "substrate.r3-macos-guest-pre-removal-receipt";
pub const MAC_R3_GUEST_RECEIPT_SIGNATURE_DOMAIN_V2: &str = "R3_MAC_GUEST_PRE_REMOVAL_RECEIPT_V2";
pub const MAC_R3_GUEST_ACK_OWNER_V2: &str =
    "substrate.r3-macos-guest-receipt-durability-acknowledgement";
pub const MAC_R3_GUEST_ACK_SIGNATURE_DOMAIN_V2: &str =
    "R3_MAC_GUEST_RECEIPT_DURABILITY_ACKNOWLEDGEMENT_V2";
pub const MAC_R3_GUEST_PROTECTED_CAS_OWNER_V2: &str =
    "substrate.r3-macos-guest-retirement-protected-cas";
pub const MAC_R3_GUEST_PROTECTED_CAS_SIGNATURE_DOMAIN_V2: &str =
    "R3_MAC_GUEST_RETIREMENT_PROTECTED_CAS_V2";
pub const MAC_R3_GUEST_RETRY_STATE_OWNER_V2: &str =
    "substrate.r3-macos-guest-retirement-retry-state";
pub const MAC_R3_GUEST_RETRY_STATE_SIGNATURE_DOMAIN_V2: &str =
    "R3_MAC_GUEST_RETIREMENT_RETRY_STATE_V2";
pub const MAC_R3_GUEST_HANDOFF_CAPSULE_OWNER_V2: &str = "substrate.r3-macos-guest-handoff-capsule";
pub const MAC_R3_GUEST_HOST_ACCEPTANCE_OWNER_V2: &str =
    "substrate.r3-macos-guest-handoff-host-acceptance";
pub const MAC_R3_GUEST_HOST_ACCEPTANCE_SIGNATURE_DOMAIN_V2: &str =
    "R3_MAC_GUEST_HANDOFF_HOST_ACCEPTANCE_V2";
pub const MAC_R3_GUEST_JOURNAL_OWNER_V2: &str = "substrate.r3-macos-guest-retirement-journal";
/// The external limactl child has no independently durable process-liveness binding. The guest
/// suffix therefore permits exactly one invocation; recovery from `EffectInvoked` may observe
/// exact final absence but must preserve on an exact-before or ambiguous target observation.
pub const MAC_R3_GUEST_MAX_EFFECT_INVOCATION_ATTEMPTS_V2: u16 = 1;
pub const MAC_R3_GUEST_MIN_TERMINAL_JOURNAL_GENERATIONS_V2: usize = 14;
pub const MAC_R3_GUEST_MAX_TERMINAL_JOURNAL_GENERATIONS_V2: usize = 14;
pub const MAC_R3_GUEST_EFFECTS_RESPONSE_OWNER_V2: &str =
    "substrate.r3-macos-guest-effects-response";
pub const MAC_R3_GUEST_EFFECTS_RESPONSE_SIGNATURE_DOMAIN_V2: &str =
    "R3_MAC_GUEST_EFFECTS_RESPONSE_V2";
pub const MAC_R3_GUEST_PARITY_OWNER_V2: &str = "substrate.r3-macos-guest-parity-proof";
pub const MAC_R3_GUEST_PARITY_SIGNATURE_DOMAIN_V2: &str = "R3_MAC_GUEST_PARITY_PROOF_V2";
pub const MAC_R3_GUEST_PARITY_HOST_BINDING_OWNER_V2: &str =
    "substrate.r3-macos-guest-parity-host-binding";
pub const MAC_R3_GUEST_PARITY_HOST_BINDING_SIGNATURE_DOMAIN_V2: &str =
    "R3_MAC_GUEST_PARITY_HOST_BINDING_V2";
pub const MAC_R3_GUEST_TERMINAL_BUNDLE_OWNER_V2: &str = "substrate.r3-macos-guest-terminal-bundle";
pub const MAC_R3_GUEST_RETIREMENT_PRECOMMIT_OWNER_V2: &str =
    "substrate.r3-macos-guest-retirement-precommit";
pub const MAC_R3_GUEST_RETIREMENT_AUTHORITY_OWNER_V2: &str =
    "substrate.r3-macos-guest-retirement-authority";
pub const MAC_R3_GUEST_RETIREMENT_AUTHORITY_SIGNATURE_DOMAIN_V2: &str =
    "substrate.r3-macos-guest-retirement-authority.signature.v2";
pub const MAC_R3_GUEST_PREPARE_RESPONSE_OWNER_V2: &str =
    "substrate.r3-macos-guest-retirement-prepare-response";
pub const MAC_R3_GUEST_BIND_REQUEST_OWNER_V2: &str =
    "substrate.r3-macos-guest-retirement-bind-request";
pub const MAC_R3_GUEST_BIND_RESPONSE_OWNER_V2: &str =
    "substrate.r3-macos-guest-retirement-bind-response";
pub const MAC_R3_PARITY_OWNER_V2: &str = "substrate.r3-macos-host-parity-proof";
pub const MAC_R3_PARITY_SIGNATURE_DOMAIN_V2: &str = "R3_MAC_HOST_PARITY_PROOF_V2";
pub const MAC_R3_TERMINAL_ACK_OWNER_V2: &str = "substrate.r3-macos-terminal-acknowledgement";
pub const MAC_R3_TERMINAL_ACK_SIGNATURE_DOMAIN_V2: &str = "R3_MAC_TERMINAL_ACKNOWLEDGEMENT_V2";

pub const MAC_R3_FINALIZER_PATH_V2: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-evidence-finalizer.v2";
pub const MAC_R3_COORDINATOR_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator";
pub const MAC_R3_DISPOSABLE_PUBLISHER_PATH_V2: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-disposable-publisher.v2";
pub const MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-disposable-publisher.v2";
pub const MAC_R3_PRODUCT_PUBLISHER_PATH_V2: &str =
    "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1";
pub const MAC_R3_FINALIZER_LAUNCHD_LABEL_V2: &str =
    "com.atomize.substrate.r3-macos-evidence-finalizer.v2";
pub const MAC_R3_FINALIZER_PLIST_PATH_V2: &str =
    "/Library/LaunchDaemons/com.atomize.substrate.r3-macos-evidence-finalizer.v2.plist";
pub const MAC_R3_FINALIZER_ENDPOINT_V2: &str =
    "/private/var/run/com.atomize.substrate.r3-macos-evidence-finalizer.v2.sock";
pub const MAC_R3_FINALIZER_JOURNAL_ROOT_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2";
pub const MAC_R3_COORDINATOR_INBOX_ROOT_V2: &str =
    "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox";
pub const MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2: u32 = 0;
pub const MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2: u32 = 20;
pub const MAC_R3_COORDINATOR_EFFECTIVE_GID_V2: u32 = 20;
pub const MAC_R3_COORDINATOR_INBOX_DIRECTORY_MODE_V2: u32 = 0o750;
pub const MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2: u32 = 0o440;
pub const MAC_R3_FINALIZER_REQUEST_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox/finalization-request.v2.json";
pub const MAC_R3_TERMINAL_BINDING_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/inbox/terminal-binding-request.v1.json";
pub const MAC_R3_RETIREMENT_LATCH_ROOT_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/latches";
pub const MAC_R3_RETIREMENT_RESOURCE_INDEX_ACCOUNT_SUFFIX_V2: &str = "retirement-resource-index-v2";
pub const MAC_R3_GUEST_RETIREMENT_CAS_ACCOUNT_SUFFIX_V2: &str = "guest-retirement-cas-v2";
pub const MAC_R3_RETIREMENT_CAS_ACCOUNT_SUFFIX_V2: &str = "retirement-cas-v2";
pub const MAC_R3_R6_TERMINAL_ACK_ACCOUNT_SUFFIX_V2: &str =
    "r6-pairing-terminal-successor-acknowledgement-v2";
pub const MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-evidence-finalizer.v2";
pub const MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-evidence-coordinator.v2";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GuestRetirementStateV2 {
    Prepared,
    QuiescePrepared,
    Quiesced,
    PreRemovalReceiptSigned,
    ReceiptExternallyDurable,
    AcknowledgementExternallyDurable,
    AcknowledgementCasBound,
    HandoffHostBound,
    Removing,
    Removed,
    ParityExternallyDurable,
    ParityHostBound,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HostRetirementStateV2 {
    Prepared,
    QuiescePrepared,
    Quiesced,
    PreRemovalReceiptSigned,
    ReceiptExternallyDurable,
    AcknowledgementExternallyDurable,
    AcknowledgementCasBound,
    FinalizationRequestFrozen,
    FinalizerAccepted,
    Removing,
    EffectsComplete,
    HarnessResidualRemoving,
    ParityExternallyDurable,
    TerminalAcknowledgementBound,
    Complete,
}

pub const GUEST_RETIREMENT_SEQUENCE_V2: [GuestRetirementStateV2; 12] = [
    GuestRetirementStateV2::Prepared,
    GuestRetirementStateV2::QuiescePrepared,
    GuestRetirementStateV2::Quiesced,
    GuestRetirementStateV2::PreRemovalReceiptSigned,
    GuestRetirementStateV2::ReceiptExternallyDurable,
    GuestRetirementStateV2::AcknowledgementExternallyDurable,
    GuestRetirementStateV2::AcknowledgementCasBound,
    GuestRetirementStateV2::HandoffHostBound,
    GuestRetirementStateV2::Removing,
    GuestRetirementStateV2::Removed,
    GuestRetirementStateV2::ParityExternallyDurable,
    GuestRetirementStateV2::ParityHostBound,
];

pub const HOST_RETIREMENT_SEQUENCE_V2: [HostRetirementStateV2; 15] = [
    HostRetirementStateV2::Prepared,
    HostRetirementStateV2::QuiescePrepared,
    HostRetirementStateV2::Quiesced,
    HostRetirementStateV2::PreRemovalReceiptSigned,
    HostRetirementStateV2::ReceiptExternallyDurable,
    HostRetirementStateV2::AcknowledgementExternallyDurable,
    HostRetirementStateV2::AcknowledgementCasBound,
    HostRetirementStateV2::FinalizationRequestFrozen,
    HostRetirementStateV2::FinalizerAccepted,
    HostRetirementStateV2::Removing,
    HostRetirementStateV2::EffectsComplete,
    HostRetirementStateV2::HarnessResidualRemoving,
    HostRetirementStateV2::ParityExternallyDurable,
    HostRetirementStateV2::TerminalAcknowledgementBound,
    HostRetirementStateV2::Complete,
];

pub fn validate_guest_state_transition_v2(
    current: GuestRetirementStateV2,
    next: GuestRetirementStateV2,
) -> Result<()> {
    if GUEST_RETIREMENT_SEQUENCE_V2
        .windows(2)
        .any(|pair| pair == [current, next])
    {
        Ok(())
    } else {
        bail!("invalid prospective guest-retirement transition")
    }
}

pub fn validate_host_state_transition_v2(
    current: HostRetirementStateV2,
    next: HostRetirementStateV2,
) -> Result<()> {
    if HOST_RETIREMENT_SEQUENCE_V2
        .windows(2)
        .any(|pair| pair == [current, next])
    {
        Ok(())
    } else {
        bail!("invalid prospective host-retirement transition")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetSetKindV2 {
    ProspectiveHost,
    DisposableCapability,
}

/// The guest terminal suffix deliberately has one destructive object: the fixed `substrate`
/// Lima instance.  Component files inside that instance are measured into the receipt inventory,
/// but are never exposed as separately selectable removal paths.  Quiescence uses the existing
/// fixed world-service Stop route and is not a destructive target.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GuestTargetRoleV2 {
    LimaInstance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GuestResourceLocatorV2 {
    LimaInstance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestTargetIdentityV2 {
    pub ordinal: u16,
    pub role: GuestTargetRoleV2,
    pub locator: GuestResourceLocatorV2,
    pub expected_before_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestEffectPlanEntryV2 {
    pub ordinal: u16,
    pub role: GuestTargetRoleV2,
    pub target_identity_sha256: String,
    pub predecessor_ordinals: Vec<u16>,
    pub required_until: GuestRetirementStateV2,
    pub required_until_effect_ordinal: Option<u16>,
}

pub const MAC_R3_MAX_GUEST_TARGETS_V2: usize = 1;

pub fn derive_guest_effect_plan_v2(
    targets: &[GuestTargetIdentityV2],
) -> Result<Vec<GuestEffectPlanEntryV2>> {
    if targets.len() != MAC_R3_MAX_GUEST_TARGETS_V2 {
        bail!("guest target ledger is not the one fixed Lima-instance target")
    }
    let target = &targets[0];
    if target.ordinal != 1
        || target.role != GuestTargetRoleV2::LimaInstance
        || target.locator != GuestResourceLocatorV2::LimaInstance
    {
        bail!("guest target ordinal, role, or locator is not the closed terminal suffix")
    }
    require_digest(
        &target.expected_before_sha256,
        "guest target expected-before",
    )?;
    Ok(vec![GuestEffectPlanEntryV2 {
        ordinal: 1,
        role: GuestTargetRoleV2::LimaInstance,
        target_identity_sha256: document_sha256_v2(target)?,
        predecessor_ordinals: Vec::new(),
        required_until: GuestRetirementStateV2::HandoffHostBound,
        required_until_effect_ordinal: None,
    }])
}

pub fn validate_guest_effect_plan_v2(
    targets: &[GuestTargetIdentityV2],
    plan: &[GuestEffectPlanEntryV2],
) -> Result<()> {
    if plan != derive_guest_effect_plan_v2(targets)? {
        bail!("guest effect plan is not the exact derived Lima-instance suffix")
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HostTargetRoleV2 {
    PublisherService,
    PublisherEndpoint,
    LifecycleRecords,
    ProtectedWrapper,
    CurrentAnchor,
    CapabilitySignControl,
    CapabilityExportControl,
    CapabilityAclMutationControl,
    CapabilityWrongKeyDeleteControl,
    Signer,
    Helper,
    LaunchdPlist,
    InstallProvenance,
    CurrentAnchorLock,
    ResourceLock,
    TerminalLatch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DisposableCapabilityControlV2 {
    Sign,
    ExportPrivate,
    ReplaceAccess,
    DeleteWrongKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GenericPasswordRoleV2 {
    ControlAdmission,
    CurrentAnchor,
    PublisherServiceState,
    PublisherBootstrapIntent,
    BootstrapAttemptLocator { attempt_key_sha256: String },
    LimaStageOneCapsule,
    R6PredecessorState,
    R6Predecessor { generation: u64 },
    R6Continuation { generation: u64 },
    R6Activation { generation: u64 },
    R6Tombstone { generation: u64 },
    R6MissedActivation { generation: u64 },
    R6ActiveGuestRecord,
    R6GuestHostRecord { challenge_id: String },
    R6OperatorLaunch { challenge_id: String },
    GuestRetirementCas,
    RetirementResourceIndex,
    RetirementCas,
    R6TerminalAcknowledgement,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PostPmJournalStateV2 {
    Prepared,
    EffectStarted,
    EffectObserved,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PostPmArtifactLabelV2 {
    MeasuredArtifact { entry_id: String },
    GuestServiceUnit,
    GuestSocketUnit,
    FixedPublisherArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleFileRoleV2 {
    Manifest {
        generation: u64,
    },
    ReceiptIndex {
        generation: u64,
    },
    Head,
    Receipt {
        generation: u64,
        receipt_id: String,
    },
    StageOneProfile {
        attempt_id: String,
    },
    PostPmJournal {
        receipt_id: String,
        state: PostPmJournalStateV2,
    },
    PostPmArtifact {
        receipt_id: String,
        label: PostPmArtifactLabelV2,
        sha256: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleDirectoryRoleV2 {
    ReceiptGeneration { generation: u64 },
    ReceiptsRoot,
    StageOneScope,
    StageOneProfilesRoot,
    PostPmAttempt { receipt_id: String },
    PostPmAttemptsRoot,
    PostPmArtifact { receipt_id: String },
    PostPmArtifactsRoot,
    GuestPairingsRoot,
    LifecycleRoot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum FixedFileRoleV2 {
    PublisherHelper,
    PublisherLaunchdPlist,
    InstallProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HostResourceLocatorV2 {
    PublisherService,
    PublisherEndpoint,
    GenericPassword {
        role: GenericPasswordRoleV2,
    },
    LifecycleFile {
        role: LifecycleFileRoleV2,
    },
    LifecycleDirectory {
        role: LifecycleDirectoryRoleV2,
    },
    SigningKey,
    FixedFile {
        role: FixedFileRoleV2,
    },
    KeychainCasLock {
        account: GenericPasswordRoleV2,
    },
    LifecycleTargetLock {
        target: LifecycleFileRoleV2,
    },
    RetirementTerminalLatch,
    DisposableCapabilityControl {
        control: DisposableCapabilityControlV2,
    },
    DisposableProtectedWrapper,
    DisposableCurrentLock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostTargetIdentityV2 {
    pub ordinal: u16,
    pub role: HostTargetRoleV2,
    pub locator: HostResourceLocatorV2,
    pub expected_before_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostEffectPlanEntryV2 {
    pub ordinal: u16,
    pub role: HostTargetRoleV2,
    pub target_identity_sha256: String,
    pub predecessor_ordinals: Vec<u16>,
    pub required_until: HostRetirementStateV2,
    pub required_until_effect_ordinal: Option<u16>,
}

pub const MAC_R3_MAX_HOST_TARGETS_V2: usize = 4096;

pub fn host_target_role_for_locator_v2(locator: &HostResourceLocatorV2) -> HostTargetRoleV2 {
    match locator {
        HostResourceLocatorV2::PublisherService => HostTargetRoleV2::PublisherService,
        HostResourceLocatorV2::PublisherEndpoint => HostTargetRoleV2::PublisherEndpoint,
        HostResourceLocatorV2::GenericPassword {
            role: GenericPasswordRoleV2::CurrentAnchor,
        } => HostTargetRoleV2::CurrentAnchor,
        HostResourceLocatorV2::GenericPassword { .. }
        | HostResourceLocatorV2::DisposableProtectedWrapper => HostTargetRoleV2::ProtectedWrapper,
        HostResourceLocatorV2::LifecycleFile { .. }
        | HostResourceLocatorV2::LifecycleDirectory { .. } => HostTargetRoleV2::LifecycleRecords,
        HostResourceLocatorV2::SigningKey => HostTargetRoleV2::Signer,
        HostResourceLocatorV2::FixedFile {
            role: FixedFileRoleV2::PublisherHelper,
        } => HostTargetRoleV2::Helper,
        HostResourceLocatorV2::FixedFile {
            role: FixedFileRoleV2::PublisherLaunchdPlist,
        } => HostTargetRoleV2::LaunchdPlist,
        HostResourceLocatorV2::FixedFile {
            role: FixedFileRoleV2::InstallProvenance,
        } => HostTargetRoleV2::InstallProvenance,
        HostResourceLocatorV2::KeychainCasLock {
            account: GenericPasswordRoleV2::CurrentAnchor,
        }
        | HostResourceLocatorV2::DisposableCurrentLock => HostTargetRoleV2::CurrentAnchorLock,
        HostResourceLocatorV2::KeychainCasLock { .. }
        | HostResourceLocatorV2::LifecycleTargetLock { .. } => HostTargetRoleV2::ResourceLock,
        HostResourceLocatorV2::RetirementTerminalLatch => HostTargetRoleV2::TerminalLatch,
        HostResourceLocatorV2::DisposableCapabilityControl {
            control: DisposableCapabilityControlV2::Sign,
        } => HostTargetRoleV2::CapabilitySignControl,
        HostResourceLocatorV2::DisposableCapabilityControl {
            control: DisposableCapabilityControlV2::ExportPrivate,
        } => HostTargetRoleV2::CapabilityExportControl,
        HostResourceLocatorV2::DisposableCapabilityControl {
            control: DisposableCapabilityControlV2::ReplaceAccess,
        } => HostTargetRoleV2::CapabilityAclMutationControl,
        HostResourceLocatorV2::DisposableCapabilityControl {
            control: DisposableCapabilityControlV2::DeleteWrongKey,
        } => HostTargetRoleV2::CapabilityWrongKeyDeleteControl,
    }
}

pub fn validate_host_resource_locator_v2(locator: &HostResourceLocatorV2) -> Result<()> {
    match locator {
        HostResourceLocatorV2::GenericPassword { role }
        | HostResourceLocatorV2::KeychainCasLock { account: role } => {
            validate_generic_password_role_v2(role)
        }
        HostResourceLocatorV2::LifecycleFile { role }
        | HostResourceLocatorV2::LifecycleTargetLock { target: role } => {
            validate_lifecycle_file_role_v2(role)
        }
        HostResourceLocatorV2::LifecycleDirectory { role } => {
            validate_lifecycle_directory_role_v2(role)
        }
        HostResourceLocatorV2::PublisherService
        | HostResourceLocatorV2::PublisherEndpoint
        | HostResourceLocatorV2::SigningKey
        | HostResourceLocatorV2::FixedFile { .. }
        | HostResourceLocatorV2::RetirementTerminalLatch
        | HostResourceLocatorV2::DisposableCapabilityControl { .. }
        | HostResourceLocatorV2::DisposableProtectedWrapper
        | HostResourceLocatorV2::DisposableCurrentLock => Ok(()),
    }
}

fn validate_generic_password_role_v2(role: &GenericPasswordRoleV2) -> Result<()> {
    match role {
        GenericPasswordRoleV2::BootstrapAttemptLocator { attempt_key_sha256 } => {
            require_digest(attempt_key_sha256, "bootstrap attempt key")
        }
        GenericPasswordRoleV2::R6Predecessor { generation }
        | GenericPasswordRoleV2::R6Continuation { generation }
        | GenericPasswordRoleV2::R6Activation { generation }
        | GenericPasswordRoleV2::R6Tombstone { generation }
        | GenericPasswordRoleV2::R6MissedActivation { generation } => {
            if *generation == 0 {
                bail!("dynamic Keychain generation must be nonzero")
            }
            Ok(())
        }
        GenericPasswordRoleV2::R6GuestHostRecord { challenge_id }
        | GenericPasswordRoleV2::R6OperatorLaunch { challenge_id } => {
            require_uuid_v7(challenge_id, "R6 challenge")
        }
        _ => Ok(()),
    }
}

fn validate_lifecycle_file_role_v2(role: &LifecycleFileRoleV2) -> Result<()> {
    match role {
        LifecycleFileRoleV2::Manifest { generation }
        | LifecycleFileRoleV2::ReceiptIndex { generation } => {
            require_nonzero_generation(*generation)
        }
        LifecycleFileRoleV2::Receipt {
            generation,
            receipt_id,
        } => {
            require_nonzero_generation(*generation)?;
            require_uuid_v7(receipt_id, "lifecycle receipt ID")
        }
        LifecycleFileRoleV2::StageOneProfile { attempt_id } => {
            require_uuid_v7(attempt_id, "Stage-1 attempt ID")
        }
        LifecycleFileRoleV2::PostPmJournal { receipt_id, .. } => {
            require_uuid_v7(receipt_id, "post-PM receipt ID")
        }
        LifecycleFileRoleV2::PostPmArtifact {
            receipt_id,
            label,
            sha256,
        } => {
            require_uuid_v7(receipt_id, "post-PM artifact receipt ID")?;
            validate_post_pm_artifact_label_v2(label)?;
            require_digest(sha256, "post-PM artifact")
        }
        LifecycleFileRoleV2::Head => Ok(()),
    }
}

fn validate_post_pm_artifact_label_v2(label: &PostPmArtifactLabelV2) -> Result<()> {
    if let PostPmArtifactLabelV2::MeasuredArtifact { entry_id } = label {
        require_uuid_v7(entry_id, "post-PM measured-artifact entry ID")?;
    }
    Ok(())
}

fn validate_lifecycle_directory_role_v2(role: &LifecycleDirectoryRoleV2) -> Result<()> {
    match role {
        LifecycleDirectoryRoleV2::ReceiptGeneration { generation } => {
            require_nonzero_generation(*generation)
        }
        LifecycleDirectoryRoleV2::PostPmAttempt { receipt_id }
        | LifecycleDirectoryRoleV2::PostPmArtifact { receipt_id } => {
            require_uuid_v7(receipt_id, "lifecycle directory receipt ID")
        }
        _ => Ok(()),
    }
}

fn require_nonzero_generation(generation: u64) -> Result<()> {
    if generation == 0 {
        bail!("lifecycle generation must be nonzero")
    }
    Ok(())
}

fn validate_disposable_target_order_v2(targets: &[HostTargetIdentityV2]) -> Result<()> {
    use DisposableCapabilityControlV2 as C;
    use HostResourceLocatorV2 as L;
    let expected = [
        L::DisposableCapabilityControl { control: C::Sign },
        L::DisposableCapabilityControl {
            control: C::ExportPrivate,
        },
        L::DisposableCapabilityControl {
            control: C::ReplaceAccess,
        },
        L::DisposableCapabilityControl {
            control: C::DeleteWrongKey,
        },
        L::DisposableProtectedWrapper,
        L::SigningKey,
        L::DisposableCurrentLock,
        L::RetirementTerminalLatch,
    ];
    if targets.len() != expected.len()
        || targets
            .iter()
            .zip(expected)
            .any(|(target, expected)| target.locator != expected)
    {
        bail!("disposable target ledger is not the exact sealed capability sequence")
    }
    Ok(())
}

fn validate_prospective_target_order_v2(targets: &[HostTargetIdentityV2]) -> Result<()> {
    use FixedFileRoleV2 as F;
    use GenericPasswordRoleV2 as G;
    use HostResourceLocatorV2 as L;

    let stage = |locator: &HostResourceLocatorV2| -> Result<u8> {
        Ok(match locator {
            L::PublisherService => 0,
            L::PublisherEndpoint => 1,
            L::GenericPassword { .. } => 2,
            L::LifecycleFile { .. } => 3,
            L::SigningKey => 4,
            L::FixedFile {
                role: F::PublisherHelper,
            } => 5,
            L::FixedFile {
                role: F::PublisherLaunchdPlist,
            } => 6,
            L::FixedFile {
                role: F::InstallProvenance,
            } => 7,
            L::KeychainCasLock { .. } | L::LifecycleTargetLock { .. } => 8,
            L::LifecycleDirectory { .. } => 9,
            L::RetirementTerminalLatch => 10,
            _ => bail!("prospective target ledger contains a disposable locator"),
        })
    };
    let stages: Vec<_> = targets
        .iter()
        .map(|target| stage(&target.locator))
        .collect::<Result<_>>()?;
    if stages.windows(2).any(|pair| pair[0] > pair[1])
        || targets.first().map(|target| &target.locator) != Some(&L::PublisherService)
        || targets.get(1).map(|target| &target.locator) != Some(&L::PublisherEndpoint)
        || targets.last().map(|target| &target.locator) != Some(&L::RetirementTerminalLatch)
    {
        bail!("prospective target ledger is not in the fixed terminal suffix order")
    }

    for required in [
        L::GenericPassword {
            role: G::CurrentAnchor,
        },
        L::GenericPassword {
            role: G::GuestRetirementCas,
        },
        L::GenericPassword {
            role: G::RetirementResourceIndex,
        },
        L::GenericPassword {
            role: G::RetirementCas,
        },
        L::GenericPassword {
            role: G::R6TerminalAcknowledgement,
        },
        L::LifecycleFile {
            role: LifecycleFileRoleV2::Head,
        },
        L::SigningKey,
        L::FixedFile {
            role: F::PublisherHelper,
        },
        L::FixedFile {
            role: F::PublisherLaunchdPlist,
        },
        L::FixedFile {
            role: F::InstallProvenance,
        },
        L::KeychainCasLock {
            account: G::CurrentAnchor,
        },
        L::KeychainCasLock {
            account: G::GuestRetirementCas,
        },
        L::LifecycleDirectory {
            role: LifecycleDirectoryRoleV2::LifecycleRoot,
        },
        L::RetirementTerminalLatch,
    ] {
        if targets
            .iter()
            .filter(|target| target.locator == required)
            .count()
            != 1
        {
            bail!("prospective target ledger lacks one mandatory exact locator")
        }
    }

    let locators: BTreeSet<_> = targets.iter().map(|target| &target.locator).collect();
    for target in targets {
        match &target.locator {
            L::GenericPassword { role } => {
                if !locators.contains(&L::KeychainCasLock {
                    account: role.clone(),
                }) {
                    bail!("generic-password target lacks its exact retained CAS lock")
                }
            }
            L::LifecycleFile { role } if lifecycle_file_requires_target_lock_v2(role) => {
                if !locators.contains(&L::LifecycleTargetLock {
                    target: role.clone(),
                }) {
                    bail!("lifecycle target lacks its deterministic retained lock")
                }
            }
            _ => {}
        }
    }
    let mut prior_depth = u8::MAX;
    for target in targets
        .iter()
        .filter(|target| matches!(target.locator, L::LifecycleDirectory { .. }))
    {
        let L::LifecycleDirectory { role } = &target.locator else {
            unreachable!()
        };
        let depth = lifecycle_directory_depth_v2(role);
        if depth > prior_depth {
            bail!("lifecycle directories are not removed deepest-first")
        }
        prior_depth = depth;
    }
    Ok(())
}

fn lifecycle_file_requires_target_lock_v2(role: &LifecycleFileRoleV2) -> bool {
    matches!(
        role,
        LifecycleFileRoleV2::Manifest { .. }
            | LifecycleFileRoleV2::ReceiptIndex { .. }
            | LifecycleFileRoleV2::Head
            | LifecycleFileRoleV2::Receipt { .. }
    )
}

fn lifecycle_directory_depth_v2(role: &LifecycleDirectoryRoleV2) -> u8 {
    match role {
        LifecycleDirectoryRoleV2::ReceiptGeneration { .. }
        | LifecycleDirectoryRoleV2::StageOneScope
        | LifecycleDirectoryRoleV2::PostPmAttempt { .. }
        | LifecycleDirectoryRoleV2::PostPmArtifact { .. } => 2,
        LifecycleDirectoryRoleV2::ReceiptsRoot
        | LifecycleDirectoryRoleV2::StageOneProfilesRoot
        | LifecycleDirectoryRoleV2::PostPmAttemptsRoot
        | LifecycleDirectoryRoleV2::PostPmArtifactsRoot
        | LifecycleDirectoryRoleV2::GuestPairingsRoot => 1,
        LifecycleDirectoryRoleV2::LifecycleRoot => 0,
    }
}

pub fn derive_host_effect_plan_v2(
    kind: TargetSetKindV2,
    targets: &[HostTargetIdentityV2],
) -> Result<Vec<HostEffectPlanEntryV2>> {
    if targets.is_empty() || targets.len() > MAC_R3_MAX_HOST_TARGETS_V2 {
        bail!("host target ledger length is outside the closed bound")
    }
    let mut seen = BTreeSet::new();
    for (index, target) in targets.iter().enumerate() {
        if target.ordinal != u16::try_from(index + 1).expect("bounded target count fits u16")
            || target.role != host_target_role_for_locator_v2(&target.locator)
        {
            bail!("host target ordinal, role, or locator join changed")
        }
        validate_host_resource_locator_v2(&target.locator)?;
        require_digest(&target.expected_before_sha256, "target expected-before")?;
        if !seen.insert(&target.locator) {
            bail!("host target ledger contains a duplicate exact locator")
        }
    }
    match kind {
        TargetSetKindV2::ProspectiveHost => validate_prospective_target_order_v2(targets)?,
        TargetSetKindV2::DisposableCapability => validate_disposable_target_order_v2(targets)?,
    }
    let plan: Vec<_> = targets
        .iter()
        .enumerate()
        .map(|(index, target)| HostEffectPlanEntryV2 {
            ordinal: u16::try_from(index + 1).expect("fixed plan length fits u16"),
            role: target.role,
            target_identity_sha256: document_sha256_v2(target)
                .expect("validated target has canonical bytes"),
            predecessor_ordinals: if index == 0 {
                Vec::new()
            } else {
                vec![u16::try_from(index).expect("bounded predecessor fits u16")]
            },
            required_until: if index == 0 {
                HostRetirementStateV2::FinalizerAccepted
            } else {
                HostRetirementStateV2::Removing
            },
            required_until_effect_ordinal: if index == 0 {
                None
            } else {
                Some(u16::try_from(index).expect("bounded holdback fits u16"))
            },
        })
        .collect();
    validate_closed_host_effect_dag_v2(&plan)?;
    Ok(plan)
}

pub fn validate_host_effect_plan_v2(
    kind: TargetSetKindV2,
    targets: &[HostTargetIdentityV2],
    plan: &[HostEffectPlanEntryV2],
) -> Result<()> {
    let expected = derive_host_effect_plan_v2(kind, targets)?;
    if plan != expected {
        bail!("host effect plan is not the exact derived DAG")
    }
    Ok(())
}

fn validate_closed_host_effect_dag_v2(plan: &[HostEffectPlanEntryV2]) -> Result<()> {
    for (index, entry) in plan.iter().enumerate() {
        if entry.ordinal != u16::try_from(index + 1).expect("fixed plan length fits u16")
            || entry.predecessor_ordinals
                != if index == 0 {
                    Vec::new()
                } else {
                    vec![u16::try_from(index).expect("bounded predecessor fits u16")]
                }
            || entry.required_until
                != if index == 0 {
                    HostRetirementStateV2::FinalizerAccepted
                } else {
                    HostRetirementStateV2::Removing
                }
            || entry.required_until_effect_ordinal
                != if index == 0 {
                    None
                } else {
                    Some(u16::try_from(index).expect("bounded holdback fits u16"))
                }
            || require_digest(&entry.target_identity_sha256, "effect target identity").is_err()
        {
            bail!("host effect plan is not the closed contiguous target DAG")
        }
    }
    if plan.last().map(|entry| entry.role) != Some(HostTargetRoleV2::TerminalLatch) {
        bail!("host effect plan does not remove the terminal latch last")
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MacR3SignatureV2 {
    pub algorithm: String,
    pub public_key: String,
    pub signature: String,
}

impl MacR3SignatureV2 {
    pub fn unsigned_p256(public_key: String) -> Self {
        Self {
            algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
            public_key,
            signature: String::new(),
        }
    }

    pub fn unsigned_ed25519(public_key: String) -> Self {
        Self {
            algorithm: "ed25519-v1".to_string(),
            public_key,
            signature: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecutableIdentityV2 {
    pub source_commit: String,
    pub source_tree: String,
    pub source_hashes_sha256: String,
    pub build_inputs_sha256: String,
    pub executable_sha256: String,
    pub executable_size: u64,
    pub intended_path: String,
    pub physical_identity_sha256: String,
    pub signing_identifier: String,
    pub designated_requirement: String,
    pub cdhash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProcessIdentityV2 {
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub canonical_account: String,
    pub pidversion_required: bool,
    pub process_start_identity_sha256: String,
    pub executable_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LaunchIdentityV2 {
    pub launchd_label: String,
    pub launchd_plist_path: String,
    pub launchd_plist_sha256: String,
    pub endpoint: String,
    pub endpoint_owner_uid: u32,
    pub endpoint_group_gid: u32,
    pub endpoint_mode: String,
    pub launch_socket_name: String,
    pub finalizer_effective_uid: u32,
}

/// Root-owned external input used only to ask the still-authorized host publisher to construct
/// one guest-retirement authority.  It deliberately carries no target, action, path, Lima name,
/// command, or guest artifact selector.  The publisher derives all of those from the fixed R6
/// terminal record and the compiled guest target plan before it signs the authority below.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestRetirementPrecommitV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub issued_at_unix_ns: u64,
    pub expires_at_unix_ns: u64,
    pub baseline_sha256: String,
    pub harness_public_key: String,
    pub r6_terminal_successor_acknowledgement_sha256: String,
}

/// Host-publisher-signed authorization consumed by the fixed installed guest executor.  It is a
/// closed capability for exactly one `substrate` Lima instance: the caller cannot provide an
/// operation, command, path, instance name, or target predicate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestRetirementAuthorityV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub r6_challenge_id: String,
    pub issued_at_unix_ns: u64,
    pub expires_at_unix_ns: u64,
    pub baseline_sha256: String,
    pub before_observation_sha256: String,
    pub component_inventory_sha256: String,
    pub quiesce_plan_sha256: String,
    pub target_ledger: Vec<GuestTargetIdentityV2>,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub r6_predecessor_consumed_sha256: String,
    pub r6_terminal_host_record_sha256: String,
    pub guest_anchor_acknowledgement_sha256: String,
    pub guest_consumption_marker_acknowledgement_sha256: String,
    pub guest_signer_public_key: String,
    pub harness_public_key: String,
    pub signature: MacR3SignatureV2,
}

/// Canonical stdout document returned by the fixed guest prepare entrypoint.  All embedded bytes
/// are independently canonical and signed; this wrapper is only a bounded transport envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestRetirementPrepareResponseV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub guest_receipt: String,
    pub guest_journal: Vec<String>,
}

/// The second and only other guest entrypoint input.  It carries the exact harness-signed
/// acknowledgement bytes and no action or target surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestRetirementBindRequestV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub r6_challenge_id: String,
    pub guest_acknowledgement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestRetirementBindResponseV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub guest_handoff_capsule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestPreRemovalReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub issued_at_unix_ns: u64,
    pub expires_at_unix_ns: u64,
    pub guest_state: GuestRetirementStateV2,
    pub baseline_sha256: String,
    pub before_observation_sha256: String,
    pub quiesced_observation_sha256: String,
    pub component_inventory_sha256: String,
    pub target_ledger: Vec<GuestTargetIdentityV2>,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub protected_cas_generation: u64,
    pub protected_cas_head_sha256: String,
    pub pre_removal_journal_head_sha256: String,
    pub r6_predecessor_consumed_sha256: String,
    pub r6_terminal_host_record_sha256: String,
    pub guest_anchor_acknowledgement_sha256: String,
    pub guest_consumption_marker_acknowledgement_sha256: String,
    pub retry_state_sha256: String,
    pub guest_signer_public_key: String,
    pub harness_public_key: String,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestDurabilityAcknowledgementV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub receipt_sha256: String,
    pub external_store_identity_sha256: String,
    pub durable_observation_sha256: String,
    pub acknowledged_at_unix_ns: u64,
    pub signature: MacR3SignatureV2,
}

/// Exact restart authority persisted by the guest publisher before the acknowledgement-CAS
/// replacement.  It contains no path or operation and cannot select another effect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestRetryStateV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub guest_state: GuestRetirementStateV2,
    pub expected_protected_cas_generation: u64,
    pub protected_cas_predecessor_head_sha256: String,
    pub receipt_sha256: String,
    pub acknowledgement_sha256: String,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub next_effect_ordinal: u16,
    pub predecessor_journal_head_sha256: String,
    pub next_journal_generation: u64,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestHandoffIntentV2 {
    pub evidence_id: String,
    pub scope_id: String,
    pub receipt_sha256: String,
    pub acknowledgement_sha256: String,
    pub retry_state_sha256: String,
    pub protected_cas_generation: u64,
    pub protected_cas_predecessor_head_sha256: String,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestProtectedCasBindingV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub generation: u64,
    pub predecessor_head_sha256: String,
    pub receipt_sha256: String,
    pub acknowledgement_sha256: String,
    pub retry_state_sha256: String,
    pub handoff_request_digest: String,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub predecessor_journal_head_sha256: String,
    pub journal_generation: u64,
    pub signature: MacR3SignatureV2,
}

/// Self-contained authority accepted by the surviving host.  Encoded documents are canonical
/// base64url bytes, not paths; the validator decodes and verifies every one before acceptance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestHandoffCapsuleV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub handoff_request_digest: String,
    pub guest_receipt: String,
    pub guest_receipt_sha256: String,
    pub guest_acknowledgement: String,
    pub guest_acknowledgement_sha256: String,
    pub guest_retry_state: String,
    pub guest_retry_state_sha256: String,
    pub guest_protected_cas_binding: String,
    pub guest_protected_cas_binding_sha256: String,
    pub guest_journal: Vec<String>,
    pub guest_journal_head_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestHostAcceptanceV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub guest_state: GuestRetirementStateV2,
    pub handoff_request_digest: String,
    pub handoff_capsule_sha256: String,
    pub guest_protected_cas_head_sha256: String,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub retry_state_sha256: String,
    pub predecessor_journal_head_sha256: String,
    pub accepted_journal_generation: u64,
    pub accepted_at_unix_ns: u64,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuestJournalEventKindV2 {
    Prepared,
    QuiescePrepared,
    Quiesced,
    PreRemovalReceiptSigned,
    ReceiptExternallyDurable,
    AcknowledgementExternallyDurable,
    AcknowledgementCasBound,
    HostAccepted,
    EffectPrepared,
    EffectInvoked,
    EffectObserved,
    GuestRemoved,
    ParityExternallyDurable,
    ParityHostBound,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestJournalGenerationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub generation: u64,
    pub predecessor_head_sha256: String,
    pub handoff_request_digest: Option<String>,
    pub host_acceptance_sha256: Option<String>,
    pub guest_state: GuestRetirementStateV2,
    pub event: GuestJournalEventKindV2,
    pub effect_ordinal: Option<u16>,
    pub target_identity_sha256: Option<String>,
    /// Zero for `EffectPrepared`; otherwise the one-based immutable invocation attempt for
    /// `EffectInvoked` and `EffectObserved`.
    pub effect_invocation_attempt: Option<u16>,
    pub observation_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestEffectsResponseV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub guest_state: GuestRetirementStateV2,
    pub handoff_request_digest: String,
    pub host_acceptance_sha256: String,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub effects_observation_sha256: String,
    pub journal_head_sha256: String,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestParityProofV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub handoff_request_digest: String,
    pub effects_response_sha256: String,
    pub journal_head_sha256: String,
    pub baseline_sha256: String,
    pub after_observation_sha256: String,
    pub exact_parity: bool,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestParityHostBindingV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub guest_state: GuestRetirementStateV2,
    pub handoff_request_digest: String,
    pub effects_response_sha256: String,
    pub parity_proof_sha256: String,
    pub host_acceptance_sha256: String,
    pub journal_head_sha256: String,
    pub generation: u64,
    pub predecessor_head_sha256: String,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestToHostSuccessorCapsuleV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub handoff_request_digest: String,
    pub guest_handoff_capsule_sha256: String,
    pub guest_host_acceptance_sha256: String,
    pub guest_receipt_sha256: String,
    pub guest_acknowledgement_sha256: String,
    pub guest_protected_cas_head_sha256: String,
    pub guest_effect_plan_sha256: String,
    pub guest_effects_response_sha256: String,
    pub guest_parity_proof_sha256: String,
    pub guest_parity_host_binding_sha256: String,
    pub guest_journal_head_sha256: String,
    pub r6_predecessor_consumed_sha256: String,
    pub r6_terminal_host_record_sha256: String,
    pub guest_anchor_acknowledgement_sha256: String,
    pub guest_consumption_marker_acknowledgement_sha256: String,
    pub retry_state_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GuestTerminalBundleV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub evidence_id: String,
    pub scope_id: String,
    pub guest_handoff_capsule: String,
    pub guest_host_acceptance: String,
    pub guest_effects_response: String,
    pub guest_parity_proof: String,
    pub guest_parity_host_binding: String,
    pub guest_journal: Vec<String>,
    pub successor_capsule: GuestToHostSuccessorCapsuleV2,
}

/// Typed, non-serializable derivation membrane for the complete guest terminal artifact set.
/// Callers cannot omit one verified document, swap in a hash-only precommit, or supply an
/// operation/target selector when deriving the surviving-host capsule.
#[derive(Debug, Clone, Copy)]
pub struct GuestToHostSuccessorBindingV2<'a> {
    pub handoff: &'a GuestHandoffCapsuleV2,
    pub receipt: &'a GuestPreRemovalReceiptV2,
    pub retry: &'a GuestRetryStateV2,
    pub protected_cas: &'a GuestProtectedCasBindingV2,
    pub acceptance: &'a GuestHostAcceptanceV2,
    pub effects_response: &'a GuestEffectsResponseV2,
    pub parity_proof: &'a GuestParityProofV2,
    pub parity_host_binding: &'a GuestParityHostBindingV2,
    pub terminal_journal_head_sha256: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublisherPreRemovalReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub target_set_kind: TargetSetKindV2,
    pub issued_at_unix_ns: u64,
    pub expires_at_unix_ns: u64,
    pub host_state: HostRetirementStateV2,
    pub guest_successor_capsule: GuestToHostSuccessorCapsuleV2,
    pub guest_successor_capsule_sha256: String,
    pub guest_parity_sha256: String,
    pub before_observation_sha256: String,
    pub quiesced_observation_sha256: String,
    pub target_ledger: Vec<HostTargetIdentityV2>,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub protected_cas_generation: u64,
    pub protected_cas_head_sha256: String,
    pub current_lock_identity_sha256: String,
    pub signer_access_control_sha256: String,
    pub publisher_signer_spki_der: String,
    pub harness_public_key: String,
    pub finalizer_identity: ExecutableIdentityV2,
    pub coordinator_identity: ExecutableIdentityV2,
    pub coordinator_process: ProcessIdentityV2,
    pub launch_identity: LaunchIdentityV2,
    pub capability_digest: String,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HarnessDurabilityAcknowledgementV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub receipt_sha256: String,
    pub external_store_identity_sha256: String,
    pub durable_observation_sha256: String,
    pub acknowledged_at_unix_ns: u64,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FrozenFinalizationIntentV2 {
    pub evidence_id: String,
    pub scope_id: String,
    pub receipt_sha256: String,
    pub acknowledgement_sha256: String,
    pub protected_cas_generation: u64,
    pub protected_cas_predecessor_head_sha256: String,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub guest_successor_capsule_sha256: String,
    pub guest_parity_sha256: String,
    pub current_lock_identity_sha256: String,
    pub signer_access_control_sha256: String,
    pub finalizer_identity_sha256: String,
    pub coordinator_identity_sha256: String,
    pub launch_identity_sha256: String,
    pub capability_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProtectedCasBindingV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub generation: u64,
    pub predecessor_head_sha256: String,
    pub receipt_sha256: String,
    pub acknowledgement_sha256: String,
    pub request_digest: String,
    pub target_ledger_sha256: String,
    pub effect_plan_sha256: String,
    pub guest_successor_capsule_sha256: String,
    pub guest_parity_sha256: String,
    pub current_lock_identity_sha256: String,
    pub signer_access_control_sha256: String,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostToFinalizerSuccessorCapsuleV2 {
    pub predecessor_journal_head_sha256: String,
    pub retry_state_sha256: String,
    pub intent: FrozenFinalizationIntentV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FinalizationRequestV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub request_digest: String,
    pub publisher_receipt: String,
    pub harness_acknowledgement: String,
    pub protected_cas_binding: String,
    pub successor_capsule: HostToFinalizerSuccessorCapsuleV2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalizerResponseStateV2 {
    SafePreAcceptanceStop,
    RejoinAcceptedRequest,
    PreservingStop,
    EffectsComplete,
    HostComplete,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreservingStopClassificationV2 {
    IdentityOrAuthorityMismatch,
    InteractionRequired,
    AmbiguousEffectState,
    TerminalProofMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FinalizerResponseV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub request_digest: String,
    pub state: FinalizerResponseStateV2,
    pub journal_head_sha256: String,
    pub effects_observation_sha256: Option<String>,
    pub terminal_acknowledgement_sha256: Option<String>,
    pub preserving_classification: Option<PreservingStopClassificationV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostParityProofV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub request_digest: String,
    pub effects_response_sha256: String,
    pub journal_head_sha256: String,
    pub baseline_sha256: String,
    pub after_observation_sha256: String,
    pub exact_parity: bool,
    pub signature: MacR3SignatureV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TerminalAcknowledgementV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub signature_domain: String,
    pub evidence_id: String,
    pub scope_id: String,
    pub request_digest: String,
    pub effects_response_sha256: String,
    pub parity_proof_sha256: String,
    pub journal_head_sha256: String,
    pub acknowledged_at_unix_ns: u64,
    pub signature: MacR3SignatureV2,
}

pub fn canonical_bytes_v2<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value).context("serialize canonical R3 JSON")?;
    let mut output = Vec::new();
    encode_canonical_json_v2(&value, &mut output)?;
    Ok(output)
}

pub fn parse_canonical_bounded_v2<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<T> {
    if bytes.is_empty() || bytes.len() > maximum_bytes {
        bail!("R3 canonical document is empty or exceeds the fixed frame limit")
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let decoded = StrictCanonicalJsonValueV2::deserialize(&mut deserializer)
        .context("decode strict R3 canonical JSON")?
        .0;
    deserializer
        .end()
        .context("reject trailing R3 JSON content")?;
    let mut canonical = Vec::new();
    encode_canonical_json_v2(&decoded, &mut canonical)?;
    if canonical != bytes {
        bail!("R3 canonical document is not the exact canonical encoding")
    }
    serde_json::from_value(decoded).context("decode typed R3 canonical JSON")
}

pub fn parse_canonical_v2<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> Result<T> {
    parse_canonical_bounded_v2(bytes, MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2)
}

pub fn sha256_hex_v2(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn document_sha256_v2<T: Serialize>(value: &T) -> Result<String> {
    Ok(sha256_hex_v2(&canonical_bytes_v2(value)?))
}

pub fn signature_payload_v2<T: Serialize>(domain: &str, owner: &str, value: &T) -> Result<Vec<u8>> {
    require_plain(domain, "signature domain")?;
    require_plain(owner, "signature owner")?;
    let mut value = serde_json::to_value(value).context("serialize R3 signed document")?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("R3 signed document must be an object"))?;
    if object.remove("signature").is_none() {
        bail!("R3 signed document lacks its signature field")
    }
    let mut payload = Vec::new();
    payload.extend_from_slice(b"SUBSTRATE_R3_MACOS_FINALIZER_SIGNATURE\0");
    payload.extend_from_slice(domain.as_bytes());
    payload.push(0);
    payload.extend_from_slice(owner.as_bytes());
    payload.push(0);
    encode_canonical_json_v2(&value, &mut payload)?;
    Ok(payload)
}

fn verify_guest_ed25519_document_v2<T: Serialize>(
    document: &T,
    domain: &str,
    owner: &str,
    signature: &MacR3SignatureV2,
    expected_public_key: &str,
    label: &str,
) -> Result<()> {
    if signature.algorithm != "ed25519-v1" || signature.public_key != expected_public_key {
        bail!("{label} signer is not the exact committed Ed25519 key")
    }
    let key = decode_base64url(expected_public_key, label)?;
    if key.len() != 32 {
        bail!("{label} public key is not one Ed25519 key")
    }
    let signature_bytes = decode_base64url(&signature.signature, label)?;
    verify_ed25519_fixed_v1(
        &key,
        &signature_payload_v2(domain, owner, document)?,
        &signature_bytes,
    )
}

fn verify_guest_host_p256_document_v2<T: Serialize>(
    document: &T,
    domain: &str,
    owner: &str,
    signature: &MacR3SignatureV2,
    expected_spki_der: &str,
    label: &str,
) -> Result<()> {
    if signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || signature.public_key != expected_spki_der
    {
        bail!("{label} signer is not the exact committed host publisher key")
    }
    let key = decode_base64url(expected_spki_der, label)?;
    let signature_bytes = decode_base64url(&signature.signature, label)?;
    verify_p256_p1363_low_s_v1(
        &key,
        &signature_payload_v2(domain, owner, document)?,
        &signature_bytes,
    )
}

pub fn validate_guest_retirement_precommit_v2(
    precommit: &GuestRetirementPrecommitV2,
) -> Result<()> {
    require_schema(
        &precommit.schema_owner,
        precommit.schema_version,
        MAC_R3_GUEST_RETIREMENT_PRECOMMIT_OWNER_V2,
    )?;
    require_plain(
        &precommit.evidence_id,
        "guest retirement precommit evidence ID",
    )?;
    require_uuid_v7(&precommit.scope_id, "guest retirement precommit scope")?;
    if precommit.issued_at_unix_ns == 0
        || precommit.issued_at_unix_ns >= precommit.expires_at_unix_ns
    {
        bail!("guest retirement precommit authority window is invalid")
    }
    require_digest(&precommit.baseline_sha256, "guest retirement baseline")?;
    require_digest(
        &precommit.r6_terminal_successor_acknowledgement_sha256,
        "guest retirement R6 terminal acknowledgement",
    )?;
    let harness = decode_base64url(&precommit.harness_public_key, "guest harness public key")?;
    if harness.len() != 32 {
        bail!("guest retirement precommit harness key is not one Ed25519 key")
    }
    Ok(())
}

pub fn guest_quiesce_plan_sha256_v2() -> Result<String> {
    document_sha256_v2(&serde_json::json!({
        "effect": "stop",
        "executor": "/usr/bin/systemctl",
        "preserved_service": "substrate-lifecycle-publisher-v1.service",
        "targets": [
            "substrate-world-service.socket",
            "substrate-world-service.service"
        ]
    }))
}

pub fn guest_component_inventory_sha256_v2(
    r6_challenge_id: &str,
    guest_signer_public_key: &str,
    r6_predecessor_consumed_sha256: &str,
    r6_terminal_host_record_sha256: &str,
    guest_anchor_acknowledgement_sha256: &str,
    guest_consumption_marker_acknowledgement_sha256: &str,
) -> Result<String> {
    require_uuid_v7(r6_challenge_id, "guest inventory R6 challenge")?;
    for (digest, label) in [
        (
            r6_predecessor_consumed_sha256,
            "guest inventory R6 predecessor",
        ),
        (
            r6_terminal_host_record_sha256,
            "guest inventory host record",
        ),
        (
            guest_anchor_acknowledgement_sha256,
            "guest inventory anchor acknowledgement",
        ),
        (
            guest_consumption_marker_acknowledgement_sha256,
            "guest inventory consumption acknowledgement",
        ),
    ] {
        require_digest(digest, label)?;
    }
    let key = decode_base64url(guest_signer_public_key, "guest inventory signing key")?;
    if key.len() != 32 {
        bail!("guest inventory signing key is not one Ed25519 key")
    }
    document_sha256_v2(&serde_json::json!({
        "guest_anchor_acknowledgement_sha256": guest_anchor_acknowledgement_sha256,
        "guest_consumption_marker_acknowledgement_sha256": guest_consumption_marker_acknowledgement_sha256,
        "guest_signer_public_key": guest_signer_public_key,
        "r6_challenge_id": r6_challenge_id,
        "r6_predecessor_consumed_sha256": r6_predecessor_consumed_sha256,
        "r6_terminal_host_record_sha256": r6_terminal_host_record_sha256,
    }))
}

pub fn validate_guest_retirement_authority_v2(
    authority: &GuestRetirementAuthorityV2,
    expected_guest_public_key: &str,
    expected_host_publisher_spki_der: &str,
) -> Result<()> {
    require_schema(
        &authority.schema_owner,
        authority.schema_version,
        MAC_R3_GUEST_RETIREMENT_AUTHORITY_OWNER_V2,
    )?;
    require_plain(
        &authority.evidence_id,
        "guest retirement authority evidence ID",
    )?;
    require_uuid_v7(&authority.scope_id, "guest retirement authority scope")?;
    require_uuid_v7(&authority.r6_challenge_id, "guest retirement R6 challenge")?;
    if authority.signature_domain != MAC_R3_GUEST_RETIREMENT_AUTHORITY_SIGNATURE_DOMAIN_V2
        || authority.issued_at_unix_ns == 0
        || authority.issued_at_unix_ns >= authority.expires_at_unix_ns
        || authority.guest_signer_public_key != expected_guest_public_key
    {
        bail!("guest retirement authority header, window, or signer changed")
    }
    for (digest, label) in [
        (&authority.baseline_sha256, "guest retirement baseline"),
        (
            &authority.before_observation_sha256,
            "guest retirement before observation",
        ),
        (
            &authority.component_inventory_sha256,
            "guest retirement component inventory",
        ),
        (&authority.quiesce_plan_sha256, "guest quiescence plan"),
        (&authority.target_ledger_sha256, "guest target ledger"),
        (&authority.effect_plan_sha256, "guest effect plan"),
        (
            &authority.r6_predecessor_consumed_sha256,
            "guest R6 consumed predecessor",
        ),
        (
            &authority.r6_terminal_host_record_sha256,
            "guest R6 terminal host record",
        ),
        (
            &authority.guest_anchor_acknowledgement_sha256,
            "guest anchor acknowledgement",
        ),
        (
            &authority.guest_consumption_marker_acknowledgement_sha256,
            "guest consumption-marker acknowledgement",
        ),
    ] {
        require_digest(digest, label)?;
    }
    let plan = derive_guest_effect_plan_v2(&authority.target_ledger)?;
    if authority.target_ledger_sha256 != document_sha256_v2(&authority.target_ledger)?
        || authority.effect_plan_sha256 != document_sha256_v2(&plan)?
        || authority.quiesce_plan_sha256 != guest_quiesce_plan_sha256_v2()?
        || authority.component_inventory_sha256
            != guest_component_inventory_sha256_v2(
                &authority.r6_challenge_id,
                &authority.guest_signer_public_key,
                &authority.r6_predecessor_consumed_sha256,
                &authority.r6_terminal_host_record_sha256,
                &authority.guest_anchor_acknowledgement_sha256,
                &authority.guest_consumption_marker_acknowledgement_sha256,
            )?
    {
        bail!(
            "guest retirement authority changed its derived inventory, quiescence, or target plan"
        )
    }
    let harness = decode_base64url(&authority.harness_public_key, "guest harness public key")?;
    if harness.len() != 32 {
        bail!("guest retirement authority harness key is not one Ed25519 key")
    }
    verify_guest_host_p256_document_v2(
        authority,
        &authority.signature_domain,
        &authority.schema_owner,
        &authority.signature,
        expected_host_publisher_spki_der,
        "guest retirement authority",
    )
}

pub fn validate_guest_retirement_prepare_response_v2(
    response: &GuestRetirementPrepareResponseV2,
    authority: &GuestRetirementAuthorityV2,
) -> Result<GuestPreRemovalReceiptV2> {
    require_schema(
        &response.schema_owner,
        response.schema_version,
        MAC_R3_GUEST_PREPARE_RESPONSE_OWNER_V2,
    )?;
    if response.evidence_id != authority.evidence_id
        || response.scope_id != authority.scope_id
        || response.guest_journal.len() != 4
    {
        bail!("guest prepare response crossed authority or chronology")
    }
    let receipt_bytes = decode_base64url(&response.guest_receipt, "guest receipt bytes")?;
    let receipt: GuestPreRemovalReceiptV2 = parse_canonical_v2(&receipt_bytes)?;
    validate_guest_pre_removal_receipt_v2(&receipt)?;
    if receipt.evidence_id != authority.evidence_id
        || receipt.scope_id != authority.scope_id
        || receipt.issued_at_unix_ns < authority.issued_at_unix_ns
        || receipt.expires_at_unix_ns != authority.expires_at_unix_ns
        || receipt.baseline_sha256 != authority.baseline_sha256
        || receipt.before_observation_sha256 != authority.before_observation_sha256
        || receipt.component_inventory_sha256 != authority.component_inventory_sha256
        || receipt.target_ledger != authority.target_ledger
        || receipt.target_ledger_sha256 != authority.target_ledger_sha256
        || receipt.effect_plan_sha256 != authority.effect_plan_sha256
        || receipt.r6_predecessor_consumed_sha256 != authority.r6_predecessor_consumed_sha256
        || receipt.r6_terminal_host_record_sha256 != authority.r6_terminal_host_record_sha256
        || receipt.guest_anchor_acknowledgement_sha256
            != authority.guest_anchor_acknowledgement_sha256
        || receipt.guest_consumption_marker_acknowledgement_sha256
            != authority.guest_consumption_marker_acknowledgement_sha256
        || receipt.guest_signer_public_key != authority.guest_signer_public_key
        || receipt.harness_public_key != authority.harness_public_key
        || receipt.retry_state_sha256 != document_sha256_v2(authority)?
        || receipt.protected_cas_generation != 1
        || receipt.protected_cas_head_sha256 != document_sha256_v2(authority)?
    {
        bail!("guest prepare receipt does not exact-bind its signed authority")
    }
    let authority_sha256 = document_sha256_v2(authority)?;
    let receipt_sha256 = document_sha256_v2(&receipt)?;
    let mut journal = Vec::with_capacity(4);
    for encoded in &response.guest_journal {
        let bytes = decode_base64url(encoded, "guest prepare journal bytes")?;
        journal.push((
            parse_canonical_v2::<GuestJournalGenerationV2>(&bytes)?,
            sha256_hex_v2(&bytes),
        ));
    }
    for index in 0..journal.len() {
        let previous = index
            .checked_sub(1)
            .map(|previous| (&journal[previous].0, journal[previous].1.as_str()));
        validate_guest_journal_transition_v2(previous, &journal[index].0)?;
        if journal[index].0.evidence_id != authority.evidence_id
            || journal[index].0.scope_id != authority.scope_id
        {
            bail!("guest prepare journal crossed authority scope")
        }
    }
    if journal[0].0.predecessor_head_sha256 != authority_sha256
        || journal[0].0.observation_sha256.as_deref() != Some(authority.baseline_sha256.as_str())
        || journal[1].0.observation_sha256.as_deref()
            != Some(authority.quiesce_plan_sha256.as_str())
        || journal[2].0.observation_sha256.as_deref()
            != Some(receipt.quiesced_observation_sha256.as_str())
        || journal[2].1 != receipt.pre_removal_journal_head_sha256
        || journal[3].0.observation_sha256.as_deref() != Some(receipt_sha256.as_str())
    {
        bail!("guest prepare journal does not bind authority, quiescence, CAS, and receipt")
    }
    Ok(receipt)
}

pub fn validate_guest_retirement_bind_request_v2(
    request: &GuestRetirementBindRequestV2,
    authority: &GuestRetirementAuthorityV2,
) -> Result<GuestDurabilityAcknowledgementV2> {
    require_schema(
        &request.schema_owner,
        request.schema_version,
        MAC_R3_GUEST_BIND_REQUEST_OWNER_V2,
    )?;
    if request.evidence_id != authority.evidence_id
        || request.scope_id != authority.scope_id
        || request.r6_challenge_id != authority.r6_challenge_id
    {
        bail!("guest bind request crossed its signed authority")
    }
    let bytes = decode_base64url(
        &request.guest_acknowledgement,
        "guest durability acknowledgement bytes",
    )?;
    parse_canonical_v2(&bytes)
}

pub fn validate_guest_retirement_bind_response_v2(
    response: &GuestRetirementBindResponseV2,
    authority: &GuestRetirementAuthorityV2,
    expected_harness_public_key: &str,
) -> Result<GuestHandoffCapsuleV2> {
    require_schema(
        &response.schema_owner,
        response.schema_version,
        MAC_R3_GUEST_BIND_RESPONSE_OWNER_V2,
    )?;
    if response.evidence_id != authority.evidence_id || response.scope_id != authority.scope_id {
        bail!("guest bind response crossed its signed authority")
    }
    let bytes = decode_base64url(
        &response.guest_handoff_capsule,
        "guest handoff capsule bytes",
    )?;
    let capsule: GuestHandoffCapsuleV2 = parse_canonical_v2(&bytes)?;
    validate_guest_handoff_capsule_v2(
        &capsule,
        &authority.guest_signer_public_key,
        expected_harness_public_key,
    )?;
    Ok(capsule)
}

pub fn validate_guest_pre_removal_receipt_v2(receipt: &GuestPreRemovalReceiptV2) -> Result<()> {
    require_schema(
        &receipt.schema_owner,
        receipt.schema_version,
        MAC_R3_GUEST_RECEIPT_OWNER_V2,
    )?;
    require_plain(&receipt.evidence_id, "guest receipt evidence ID")?;
    require_uuid_v7(&receipt.scope_id, "guest receipt scope")?;
    if receipt.signature_domain != MAC_R3_GUEST_RECEIPT_SIGNATURE_DOMAIN_V2
        || receipt.guest_state != GuestRetirementStateV2::PreRemovalReceiptSigned
        || receipt.issued_at_unix_ns == 0
        || receipt.issued_at_unix_ns >= receipt.expires_at_unix_ns
        || receipt.protected_cas_generation == 0
    {
        bail!("guest pre-removal receipt chronology or authority window is invalid")
    }
    for (digest, label) in [
        (&receipt.baseline_sha256, "guest baseline"),
        (
            &receipt.before_observation_sha256,
            "guest before observation",
        ),
        (
            &receipt.quiesced_observation_sha256,
            "guest quiesced observation",
        ),
        (
            &receipt.component_inventory_sha256,
            "guest component inventory",
        ),
        (&receipt.target_ledger_sha256, "guest target ledger"),
        (&receipt.effect_plan_sha256, "guest effect plan"),
        (
            &receipt.protected_cas_head_sha256,
            "guest protected CAS head",
        ),
        (
            &receipt.pre_removal_journal_head_sha256,
            "guest pre-removal journal head",
        ),
        (
            &receipt.r6_predecessor_consumed_sha256,
            "guest R6 consumed predecessor",
        ),
        (
            &receipt.r6_terminal_host_record_sha256,
            "guest R6 terminal host record",
        ),
        (
            &receipt.guest_anchor_acknowledgement_sha256,
            "guest anchor acknowledgement",
        ),
        (
            &receipt.guest_consumption_marker_acknowledgement_sha256,
            "guest consumption-marker acknowledgement",
        ),
        (&receipt.retry_state_sha256, "guest pre-receipt retry state"),
    ] {
        require_digest(digest, label)?;
    }
    let plan = derive_guest_effect_plan_v2(&receipt.target_ledger)?;
    if receipt.target_ledger_sha256 != document_sha256_v2(&receipt.target_ledger)?
        || receipt.effect_plan_sha256 != document_sha256_v2(&plan)?
    {
        bail!("guest receipt target ledger or fixed effect plan digest changed")
    }
    let harness = decode_base64url(&receipt.harness_public_key, "guest harness public key")?;
    if harness.len() != 32 {
        bail!("guest receipt harness key is not one Ed25519 public key")
    }
    verify_guest_ed25519_document_v2(
        receipt,
        &receipt.signature_domain,
        &receipt.schema_owner,
        &receipt.signature,
        &receipt.guest_signer_public_key,
        "guest receipt",
    )
}

pub fn validate_guest_durability_acknowledgement_v2(
    acknowledgement: &GuestDurabilityAcknowledgementV2,
    receipt: &GuestPreRemovalReceiptV2,
) -> Result<()> {
    require_schema(
        &acknowledgement.schema_owner,
        acknowledgement.schema_version,
        MAC_R3_GUEST_ACK_OWNER_V2,
    )?;
    if acknowledgement.signature_domain != MAC_R3_GUEST_ACK_SIGNATURE_DOMAIN_V2
        || acknowledgement.evidence_id != receipt.evidence_id
        || acknowledgement.scope_id != receipt.scope_id
        || acknowledgement.receipt_sha256 != document_sha256_v2(receipt)?
        || acknowledgement.acknowledged_at_unix_ns < receipt.issued_at_unix_ns
        || acknowledgement.acknowledged_at_unix_ns > receipt.expires_at_unix_ns
    {
        bail!("guest durability acknowledgement does not exact-bind the durable receipt")
    }
    require_digest(
        &acknowledgement.external_store_identity_sha256,
        "guest receipt external store",
    )?;
    require_digest(
        &acknowledgement.durable_observation_sha256,
        "guest receipt durable observation",
    )?;
    verify_guest_ed25519_document_v2(
        acknowledgement,
        &acknowledgement.signature_domain,
        &acknowledgement.schema_owner,
        &acknowledgement.signature,
        &receipt.harness_public_key,
        "guest durability acknowledgement",
    )
}

pub fn validate_guest_retry_state_v2(
    retry: &GuestRetryStateV2,
    receipt: &GuestPreRemovalReceiptV2,
    acknowledgement: &GuestDurabilityAcknowledgementV2,
) -> Result<()> {
    require_schema(
        &retry.schema_owner,
        retry.schema_version,
        MAC_R3_GUEST_RETRY_STATE_OWNER_V2,
    )?;
    if retry.signature_domain != MAC_R3_GUEST_RETRY_STATE_SIGNATURE_DOMAIN_V2
        || retry.evidence_id != receipt.evidence_id
        || retry.scope_id != receipt.scope_id
        || retry.guest_state != GuestRetirementStateV2::AcknowledgementCasBound
        || retry.expected_protected_cas_generation != receipt.protected_cas_generation + 1
        || retry.protected_cas_predecessor_head_sha256 != receipt.protected_cas_head_sha256
        || retry.receipt_sha256 != document_sha256_v2(receipt)?
        || retry.acknowledgement_sha256 != document_sha256_v2(acknowledgement)?
        || retry.target_ledger_sha256 != receipt.target_ledger_sha256
        || retry.effect_plan_sha256 != receipt.effect_plan_sha256
        || retry.next_effect_ordinal != 1
        || retry.next_journal_generation != 7
    {
        bail!("guest retry state does not exact-bind acknowledgement-CAS authority")
    }
    require_digest(
        &retry.predecessor_journal_head_sha256,
        "guest retry predecessor journal head",
    )?;
    verify_guest_ed25519_document_v2(
        retry,
        &retry.signature_domain,
        &retry.schema_owner,
        &retry.signature,
        &receipt.guest_signer_public_key,
        "guest retry state",
    )
}

pub fn derive_guest_handoff_intent_v2(
    receipt: &GuestPreRemovalReceiptV2,
    acknowledgement: &GuestDurabilityAcknowledgementV2,
    retry: &GuestRetryStateV2,
) -> Result<GuestHandoffIntentV2> {
    Ok(GuestHandoffIntentV2 {
        evidence_id: receipt.evidence_id.clone(),
        scope_id: receipt.scope_id.clone(),
        receipt_sha256: document_sha256_v2(receipt)?,
        acknowledgement_sha256: document_sha256_v2(acknowledgement)?,
        retry_state_sha256: document_sha256_v2(retry)?,
        protected_cas_generation: retry.expected_protected_cas_generation,
        protected_cas_predecessor_head_sha256: retry.protected_cas_predecessor_head_sha256.clone(),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
    })
}

pub fn validate_guest_protected_cas_binding_v2(
    binding: &GuestProtectedCasBindingV2,
    receipt: &GuestPreRemovalReceiptV2,
    acknowledgement: &GuestDurabilityAcknowledgementV2,
    retry: &GuestRetryStateV2,
) -> Result<()> {
    require_schema(
        &binding.schema_owner,
        binding.schema_version,
        MAC_R3_GUEST_PROTECTED_CAS_OWNER_V2,
    )?;
    let intent = derive_guest_handoff_intent_v2(receipt, acknowledgement, retry)?;
    let request_digest = document_sha256_v2(&intent)?;
    if binding.signature_domain != MAC_R3_GUEST_PROTECTED_CAS_SIGNATURE_DOMAIN_V2
        || binding.evidence_id != receipt.evidence_id
        || binding.scope_id != receipt.scope_id
        || binding.generation != retry.expected_protected_cas_generation
        || binding.predecessor_head_sha256 != receipt.protected_cas_head_sha256
        || binding.receipt_sha256 != intent.receipt_sha256
        || binding.acknowledgement_sha256 != intent.acknowledgement_sha256
        || binding.retry_state_sha256 != intent.retry_state_sha256
        || binding.handoff_request_digest != request_digest
        || binding.target_ledger_sha256 != receipt.target_ledger_sha256
        || binding.effect_plan_sha256 != receipt.effect_plan_sha256
        || binding.predecessor_journal_head_sha256 != retry.predecessor_journal_head_sha256
        || binding.journal_generation != retry.next_journal_generation
    {
        bail!("guest protected CAS does not exact-bind receipt, acknowledgement, and retry state")
    }
    verify_guest_ed25519_document_v2(
        binding,
        &binding.signature_domain,
        &binding.schema_owner,
        &binding.signature,
        &receipt.guest_signer_public_key,
        "guest protected CAS",
    )
}

pub fn validate_guest_handoff_capsule_v2(
    capsule: &GuestHandoffCapsuleV2,
    expected_guest_public_key: &str,
    expected_harness_public_key: &str,
) -> Result<(
    GuestPreRemovalReceiptV2,
    GuestDurabilityAcknowledgementV2,
    GuestRetryStateV2,
    GuestProtectedCasBindingV2,
)> {
    require_schema(
        &capsule.schema_owner,
        capsule.schema_version,
        MAC_R3_GUEST_HANDOFF_CAPSULE_OWNER_V2,
    )?;
    require_uuid_v7(&capsule.scope_id, "guest handoff scope")?;
    require_plain(&capsule.evidence_id, "guest handoff evidence ID")?;
    require_digest(&capsule.handoff_request_digest, "guest handoff request")?;
    let receipt_bytes = decode_base64url(&capsule.guest_receipt, "guest receipt bytes")?;
    let acknowledgement_bytes = decode_base64url(
        &capsule.guest_acknowledgement,
        "guest acknowledgement bytes",
    )?;
    let retry_bytes = decode_base64url(&capsule.guest_retry_state, "guest retry-state bytes")?;
    let binding_bytes = decode_base64url(
        &capsule.guest_protected_cas_binding,
        "guest protected-CAS bytes",
    )?;
    let receipt: GuestPreRemovalReceiptV2 = parse_canonical_v2(&receipt_bytes)?;
    let acknowledgement: GuestDurabilityAcknowledgementV2 =
        parse_canonical_v2(&acknowledgement_bytes)?;
    let retry: GuestRetryStateV2 = parse_canonical_v2(&retry_bytes)?;
    let binding: GuestProtectedCasBindingV2 = parse_canonical_v2(&binding_bytes)?;
    validate_guest_pre_removal_receipt_v2(&receipt)?;
    validate_guest_durability_acknowledgement_v2(&acknowledgement, &receipt)?;
    validate_guest_retry_state_v2(&retry, &receipt, &acknowledgement)?;
    validate_guest_protected_cas_binding_v2(&binding, &receipt, &acknowledgement, &retry)?;
    if capsule.guest_journal.len() != 7 {
        bail!("guest handoff capsule lacks the exact seven-generation pre-handoff journal")
    }
    let journal = capsule
        .guest_journal
        .iter()
        .map(|encoded| {
            let bytes = decode_base64url(encoded, "guest pre-handoff journal generation")?;
            Ok((
                parse_canonical_v2::<GuestJournalGenerationV2>(&bytes)?,
                sha256_hex_v2(&bytes),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    for (index, (generation, _)) in journal.iter().enumerate() {
        let previous = index
            .checked_sub(1)
            .map(|previous| (&journal[previous].0, journal[previous].1.as_str()));
        validate_guest_journal_transition_v2(previous, generation)?;
        if generation.evidence_id != receipt.evidence_id || generation.scope_id != receipt.scope_id
        {
            bail!("guest pre-handoff journal crossed its receipt evidence or scope")
        }
    }
    let receipt_sha256 = sha256_hex_v2(&receipt_bytes);
    let acknowledgement_sha256 = sha256_hex_v2(&acknowledgement_bytes);
    let binding_sha256 = sha256_hex_v2(&binding_bytes);
    if receipt.guest_signer_public_key != expected_guest_public_key
        || receipt.harness_public_key != expected_harness_public_key
        || capsule.evidence_id != receipt.evidence_id
        || capsule.scope_id != receipt.scope_id
        || capsule.handoff_request_digest != binding.handoff_request_digest
        || capsule.guest_receipt_sha256 != receipt_sha256
        || capsule.guest_acknowledgement_sha256 != acknowledgement_sha256
        || capsule.guest_retry_state_sha256 != sha256_hex_v2(&retry_bytes)
        || capsule.guest_protected_cas_binding_sha256 != binding_sha256
        || journal[0].0.predecessor_head_sha256 != receipt.protected_cas_head_sha256
        || journal[0].0.observation_sha256.as_deref() != Some(receipt.baseline_sha256.as_str())
        || journal[2].0.observation_sha256.as_deref()
            != Some(receipt.quiesced_observation_sha256.as_str())
        || journal[2].1 != receipt.pre_removal_journal_head_sha256
        || journal[3].0.observation_sha256.as_deref() != Some(receipt_sha256.as_str())
        || journal[4].0.observation_sha256.as_deref()
            != Some(acknowledgement.durable_observation_sha256.as_str())
        || journal[5].0.observation_sha256.as_deref() != Some(acknowledgement_sha256.as_str())
        || journal[5].1 != retry.predecessor_journal_head_sha256
        || journal[6].0.handoff_request_digest.as_deref()
            != Some(binding.handoff_request_digest.as_str())
        || journal[6].0.observation_sha256.as_deref() != Some(binding_sha256.as_str())
        || binding.predecessor_journal_head_sha256 != journal[5].1
        || capsule.guest_journal_head_sha256 != journal[6].1
    {
        bail!("guest handoff capsule does not carry its exact independently verified documents")
    }
    Ok((receipt, acknowledgement, retry, binding))
}

pub fn validate_guest_host_acceptance_v2(
    acceptance: &GuestHostAcceptanceV2,
    capsule: &GuestHandoffCapsuleV2,
    receipt: &GuestPreRemovalReceiptV2,
    retry: &GuestRetryStateV2,
    binding: &GuestProtectedCasBindingV2,
    expected_host_publisher_spki_der: &str,
) -> Result<()> {
    require_schema(
        &acceptance.schema_owner,
        acceptance.schema_version,
        MAC_R3_GUEST_HOST_ACCEPTANCE_OWNER_V2,
    )?;
    if acceptance.signature_domain != MAC_R3_GUEST_HOST_ACCEPTANCE_SIGNATURE_DOMAIN_V2
        || acceptance.evidence_id != receipt.evidence_id
        || acceptance.scope_id != receipt.scope_id
        || acceptance.guest_state != GuestRetirementStateV2::HandoffHostBound
        || acceptance.handoff_request_digest != capsule.handoff_request_digest
        || acceptance.handoff_capsule_sha256 != document_sha256_v2(capsule)?
        || acceptance.guest_protected_cas_head_sha256 != document_sha256_v2(binding)?
        || acceptance.target_ledger_sha256 != receipt.target_ledger_sha256
        || acceptance.effect_plan_sha256 != receipt.effect_plan_sha256
        || acceptance.retry_state_sha256 != document_sha256_v2(retry)?
        || acceptance.predecessor_journal_head_sha256 != capsule.guest_journal_head_sha256
        || acceptance.accepted_journal_generation != 8
        || acceptance.accepted_at_unix_ns < receipt.issued_at_unix_ns
        || acceptance.accepted_at_unix_ns > receipt.expires_at_unix_ns
    {
        bail!("surviving-host acceptance does not exact-bind the complete guest handoff")
    }
    require_digest(
        &acceptance.predecessor_journal_head_sha256,
        "guest predecessor journal head",
    )?;
    verify_guest_host_p256_document_v2(
        acceptance,
        &acceptance.signature_domain,
        &acceptance.schema_owner,
        &acceptance.signature,
        expected_host_publisher_spki_der,
        "guest host acceptance",
    )
}

pub fn validate_guest_journal_transition_v2(
    previous: Option<(&GuestJournalGenerationV2, &str)>,
    next: &GuestJournalGenerationV2,
) -> Result<()> {
    validate_guest_journal_generation_v2(next)?;
    match previous {
        None => {
            if next.generation != 1 {
                bail!("guest journal must begin with GuestPrepared")
            }
        }
        Some((previous, previous_sha256)) => {
            validate_guest_journal_generation_v2(previous)?;
            require_digest(previous_sha256, "guest previous journal generation")?;
            if next.generation != previous.generation + 1
                || next.predecessor_head_sha256 != previous_sha256
                || next.evidence_id != previous.evidence_id
                || next.scope_id != previous.scope_id
            {
                bail!("guest journal generation does not exact-chain its predecessor")
            }
            if next.generation != 7
                && next.handoff_request_digest != previous.handoff_request_digest
            {
                bail!("guest journal request digest changed outside acknowledgement-CAS binding")
            }
            if next.generation != 8
                && next.host_acceptance_sha256 != previous.host_acceptance_sha256
            {
                bail!("guest journal host acceptance changed outside successor acceptance")
            }
            use GuestJournalEventKindV2 as E;
            let same_effect = previous.effect_ordinal == next.effect_ordinal
                && previous.target_identity_sha256 == next.target_identity_sha256;
            let causal = match (previous.event, next.event) {
                (E::Prepared, E::QuiescePrepared)
                | (E::QuiescePrepared, E::Quiesced)
                | (E::Quiesced, E::PreRemovalReceiptSigned)
                | (E::PreRemovalReceiptSigned, E::ReceiptExternallyDurable)
                | (E::ReceiptExternallyDurable, E::AcknowledgementExternallyDurable)
                | (E::AcknowledgementExternallyDurable, E::AcknowledgementCasBound)
                | (E::AcknowledgementCasBound, E::HostAccepted)
                | (E::HostAccepted, E::EffectPrepared)
                | (E::EffectObserved, E::GuestRemoved)
                | (E::GuestRemoved, E::ParityExternallyDurable)
                | (E::ParityExternallyDurable, E::ParityHostBound) => true,
                (E::EffectPrepared, E::EffectInvoked) => {
                    same_effect
                        && previous.effect_invocation_attempt == Some(0)
                        && next.effect_invocation_attempt == Some(1)
                }
                (E::EffectInvoked, E::EffectObserved) => {
                    same_effect
                        && previous.effect_invocation_attempt == next.effect_invocation_attempt
                }
                _ => false,
            };
            if !causal {
                bail!("guest journal transition is outside the bounded causal chronology")
            }
        }
    }
    Ok(())
}

pub fn validate_guest_journal_generation_v2(record: &GuestJournalGenerationV2) -> Result<()> {
    require_schema(
        &record.schema_owner,
        record.schema_version,
        MAC_R3_GUEST_JOURNAL_OWNER_V2,
    )?;
    require_plain(&record.evidence_id, "guest journal evidence ID")?;
    require_uuid_v7(&record.scope_id, "guest journal scope")?;
    require_digest(&record.predecessor_head_sha256, "guest journal predecessor")?;
    let expected = match record.event {
        GuestJournalEventKindV2::Prepared => (
            record.generation == 1,
            GuestRetirementStateV2::Prepared,
            None,
            false,
            true,
            false,
            false,
        ),
        GuestJournalEventKindV2::QuiescePrepared => (
            record.generation == 2,
            GuestRetirementStateV2::QuiescePrepared,
            None,
            false,
            true,
            false,
            false,
        ),
        GuestJournalEventKindV2::Quiesced => (
            record.generation == 3,
            GuestRetirementStateV2::Quiesced,
            None,
            false,
            true,
            false,
            false,
        ),
        GuestJournalEventKindV2::PreRemovalReceiptSigned => (
            record.generation == 4,
            GuestRetirementStateV2::PreRemovalReceiptSigned,
            None,
            false,
            true,
            false,
            false,
        ),
        GuestJournalEventKindV2::ReceiptExternallyDurable => (
            record.generation == 5,
            GuestRetirementStateV2::ReceiptExternallyDurable,
            None,
            false,
            true,
            false,
            false,
        ),
        GuestJournalEventKindV2::AcknowledgementExternallyDurable => (
            record.generation == 6,
            GuestRetirementStateV2::AcknowledgementExternallyDurable,
            None,
            false,
            true,
            false,
            false,
        ),
        GuestJournalEventKindV2::AcknowledgementCasBound => (
            record.generation == 7,
            GuestRetirementStateV2::AcknowledgementCasBound,
            None,
            false,
            true,
            true,
            false,
        ),
        GuestJournalEventKindV2::HostAccepted => (
            record.generation == 8,
            GuestRetirementStateV2::HandoffHostBound,
            None,
            false,
            true,
            true,
            true,
        ),
        GuestJournalEventKindV2::EffectPrepared => (
            record.generation == 9,
            GuestRetirementStateV2::Removing,
            Some(1),
            true,
            false,
            true,
            true,
        ),
        GuestJournalEventKindV2::EffectInvoked => (
            (record.generation, record.effect_invocation_attempt) == (10, Some(1)),
            GuestRetirementStateV2::Removing,
            Some(1),
            true,
            false,
            true,
            true,
        ),
        GuestJournalEventKindV2::EffectObserved => (
            (record.generation, record.effect_invocation_attempt) == (11, Some(1)),
            GuestRetirementStateV2::Removing,
            Some(1),
            true,
            true,
            true,
            true,
        ),
        GuestJournalEventKindV2::GuestRemoved => (
            record.generation == 12,
            GuestRetirementStateV2::Removed,
            None,
            false,
            true,
            true,
            true,
        ),
        GuestJournalEventKindV2::ParityExternallyDurable => (
            record.generation == 13,
            GuestRetirementStateV2::ParityExternallyDurable,
            None,
            false,
            true,
            true,
            true,
        ),
        GuestJournalEventKindV2::ParityHostBound => (
            record.generation == 14,
            GuestRetirementStateV2::ParityHostBound,
            None,
            false,
            true,
            true,
            true,
        ),
    };
    let effect_invocation_attempt_is_valid = match record.event {
        GuestJournalEventKindV2::EffectPrepared => record.effect_invocation_attempt == Some(0),
        GuestJournalEventKindV2::EffectInvoked | GuestJournalEventKindV2::EffectObserved => {
            record.effect_invocation_attempt.is_some_and(|attempt| {
                (1..=MAC_R3_GUEST_MAX_EFFECT_INVOCATION_ATTEMPTS_V2).contains(&attempt)
            })
        }
        _ => record.effect_invocation_attempt.is_none(),
    };
    if !expected.0
        || record.guest_state != expected.1
        || record.effect_ordinal != expected.2
        || record.target_identity_sha256.is_some() != expected.3
        || record.observation_sha256.is_some() != expected.4
        || record.handoff_request_digest.is_some() != expected.5
        || record.host_acceptance_sha256.is_some() != expected.6
        || !effect_invocation_attempt_is_valid
    {
        bail!("guest journal generation is outside the bounded terminal chronology")
    }
    for (digest, label) in [
        (
            record.target_identity_sha256.as_deref(),
            "guest journal target",
        ),
        (
            record.observation_sha256.as_deref(),
            "guest journal observation",
        ),
        (
            record.handoff_request_digest.as_deref(),
            "guest journal request",
        ),
        (
            record.host_acceptance_sha256.as_deref(),
            "guest journal acceptance",
        ),
    ] {
        if let Some(digest) = digest {
            require_digest(digest, label)?;
        }
    }
    Ok(())
}

pub fn validate_guest_effects_response_v2(
    response: &GuestEffectsResponseV2,
    receipt: &GuestPreRemovalReceiptV2,
    acceptance: &GuestHostAcceptanceV2,
    expected_host_publisher_spki_der: &str,
) -> Result<()> {
    require_schema(
        &response.schema_owner,
        response.schema_version,
        MAC_R3_GUEST_EFFECTS_RESPONSE_OWNER_V2,
    )?;
    if response.signature_domain != MAC_R3_GUEST_EFFECTS_RESPONSE_SIGNATURE_DOMAIN_V2
        || response.evidence_id != receipt.evidence_id
        || response.scope_id != receipt.scope_id
        || response.guest_state != GuestRetirementStateV2::Removed
        || response.handoff_request_digest != acceptance.handoff_request_digest
        || response.host_acceptance_sha256 != document_sha256_v2(acceptance)?
        || response.target_ledger_sha256 != receipt.target_ledger_sha256
        || response.effect_plan_sha256 != receipt.effect_plan_sha256
    {
        bail!("guest effects response does not exact-bind the accepted frozen suffix")
    }
    require_digest(
        &response.effects_observation_sha256,
        "guest effects observation",
    )?;
    require_digest(&response.journal_head_sha256, "guest effects journal head")?;
    verify_guest_host_p256_document_v2(
        response,
        &response.signature_domain,
        &response.schema_owner,
        &response.signature,
        expected_host_publisher_spki_der,
        "guest effects response",
    )
}

pub fn validate_guest_parity_proof_v2(
    proof: &GuestParityProofV2,
    receipt: &GuestPreRemovalReceiptV2,
    response: &GuestEffectsResponseV2,
) -> Result<()> {
    require_schema(
        &proof.schema_owner,
        proof.schema_version,
        MAC_R3_GUEST_PARITY_OWNER_V2,
    )?;
    if proof.signature_domain != MAC_R3_GUEST_PARITY_SIGNATURE_DOMAIN_V2
        || proof.evidence_id != receipt.evidence_id
        || proof.scope_id != receipt.scope_id
        || proof.handoff_request_digest != response.handoff_request_digest
        || proof.effects_response_sha256 != document_sha256_v2(response)?
        || proof.journal_head_sha256 != response.journal_head_sha256
        || proof.baseline_sha256 != receipt.baseline_sha256
        || !proof.exact_parity
    {
        bail!("guest parity is not the exact post-removal proof for the accepted suffix")
    }
    require_digest(
        &proof.after_observation_sha256,
        "guest parity after observation",
    )?;
    verify_guest_ed25519_document_v2(
        proof,
        &proof.signature_domain,
        &proof.schema_owner,
        &proof.signature,
        &receipt.harness_public_key,
        "guest parity proof",
    )
}

pub fn validate_guest_parity_host_binding_v2(
    binding: &GuestParityHostBindingV2,
    response: &GuestEffectsResponseV2,
    parity: &GuestParityProofV2,
    acceptance: &GuestHostAcceptanceV2,
    parity_durable_journal_head_sha256: &str,
    expected_host_publisher_spki_der: &str,
) -> Result<()> {
    require_schema(
        &binding.schema_owner,
        binding.schema_version,
        MAC_R3_GUEST_PARITY_HOST_BINDING_OWNER_V2,
    )?;
    if binding.signature_domain != MAC_R3_GUEST_PARITY_HOST_BINDING_SIGNATURE_DOMAIN_V2
        || binding.evidence_id != response.evidence_id
        || binding.scope_id != response.scope_id
        || binding.guest_state != GuestRetirementStateV2::ParityHostBound
        || binding.handoff_request_digest != response.handoff_request_digest
        || binding.effects_response_sha256 != document_sha256_v2(response)?
        || binding.parity_proof_sha256 != document_sha256_v2(parity)?
        || binding.host_acceptance_sha256 != document_sha256_v2(acceptance)?
        || binding.journal_head_sha256 != parity_durable_journal_head_sha256
        || binding.generation != 2
        || binding.predecessor_head_sha256 != document_sha256_v2(acceptance)?
    {
        bail!("surviving host CAS does not exact-bind guest parity and effects")
    }
    verify_guest_host_p256_document_v2(
        binding,
        &binding.signature_domain,
        &binding.schema_owner,
        &binding.signature,
        expected_host_publisher_spki_der,
        "guest parity host binding",
    )
}

pub fn derive_guest_to_host_successor_capsule_v2(
    binding: GuestToHostSuccessorBindingV2<'_>,
) -> Result<GuestToHostSuccessorCapsuleV2> {
    require_digest(
        binding.terminal_journal_head_sha256,
        "guest terminal journal head",
    )?;
    Ok(GuestToHostSuccessorCapsuleV2 {
        schema_owner: MAC_R3_GUEST_TO_HOST_CAPSULE_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        evidence_id: binding.receipt.evidence_id.clone(),
        scope_id: binding.receipt.scope_id.clone(),
        handoff_request_digest: binding.handoff.handoff_request_digest.clone(),
        guest_handoff_capsule_sha256: document_sha256_v2(binding.handoff)?,
        guest_host_acceptance_sha256: document_sha256_v2(binding.acceptance)?,
        guest_receipt_sha256: document_sha256_v2(binding.receipt)?,
        guest_acknowledgement_sha256: binding.handoff.guest_acknowledgement_sha256.clone(),
        guest_protected_cas_head_sha256: document_sha256_v2(binding.protected_cas)?,
        guest_effect_plan_sha256: binding.receipt.effect_plan_sha256.clone(),
        guest_effects_response_sha256: document_sha256_v2(binding.effects_response)?,
        guest_parity_proof_sha256: document_sha256_v2(binding.parity_proof)?,
        guest_parity_host_binding_sha256: document_sha256_v2(binding.parity_host_binding)?,
        guest_journal_head_sha256: binding.terminal_journal_head_sha256.to_string(),
        r6_predecessor_consumed_sha256: binding.receipt.r6_predecessor_consumed_sha256.clone(),
        r6_terminal_host_record_sha256: binding.receipt.r6_terminal_host_record_sha256.clone(),
        guest_anchor_acknowledgement_sha256: binding
            .receipt
            .guest_anchor_acknowledgement_sha256
            .clone(),
        guest_consumption_marker_acknowledgement_sha256: binding
            .receipt
            .guest_consumption_marker_acknowledgement_sha256
            .clone(),
        retry_state_sha256: document_sha256_v2(binding.retry)?,
    })
}

pub fn validate_guest_terminal_bundle_v2(
    bundle: &GuestTerminalBundleV2,
    expected_guest_public_key: &str,
    expected_harness_public_key: &str,
    expected_host_publisher_spki_der: &str,
) -> Result<GuestToHostSuccessorCapsuleV2> {
    require_schema(
        &bundle.schema_owner,
        bundle.schema_version,
        MAC_R3_GUEST_TERMINAL_BUNDLE_OWNER_V2,
    )?;
    require_uuid_v7(&bundle.scope_id, "guest terminal bundle scope")?;
    require_plain(&bundle.evidence_id, "guest terminal bundle evidence ID")?;
    let handoff_bytes = decode_base64url(&bundle.guest_handoff_capsule, "guest handoff bytes")?;
    let acceptance_bytes =
        decode_base64url(&bundle.guest_host_acceptance, "guest host-acceptance bytes")?;
    let response_bytes = decode_base64url(
        &bundle.guest_effects_response,
        "guest effects-response bytes",
    )?;
    let parity_bytes = decode_base64url(&bundle.guest_parity_proof, "guest parity bytes")?;
    let parity_binding_bytes = decode_base64url(
        &bundle.guest_parity_host_binding,
        "guest parity host-binding bytes",
    )?;
    let handoff: GuestHandoffCapsuleV2 = parse_canonical_v2(&handoff_bytes)?;
    let acceptance: GuestHostAcceptanceV2 = parse_canonical_v2(&acceptance_bytes)?;
    let response: GuestEffectsResponseV2 = parse_canonical_v2(&response_bytes)?;
    let parity: GuestParityProofV2 = parse_canonical_v2(&parity_bytes)?;
    let parity_binding: GuestParityHostBindingV2 = parse_canonical_v2(&parity_binding_bytes)?;
    if !(MAC_R3_GUEST_MIN_TERMINAL_JOURNAL_GENERATIONS_V2
        ..=MAC_R3_GUEST_MAX_TERMINAL_JOURNAL_GENERATIONS_V2)
        .contains(&bundle.guest_journal.len())
    {
        bail!("guest terminal bundle is outside the exact fourteen-generation journal")
    }
    let journal = bundle
        .guest_journal
        .iter()
        .map(|encoded| {
            let bytes = decode_base64url(encoded, "guest journal generation bytes")?;
            let generation = parse_canonical_v2::<GuestJournalGenerationV2>(&bytes)?;
            Ok((generation, sha256_hex_v2(&bytes)))
        })
        .collect::<Result<Vec<_>>>()?;
    let (receipt, _acknowledgement, retry, protected_cas) = validate_guest_handoff_capsule_v2(
        &handoff,
        expected_guest_public_key,
        expected_harness_public_key,
    )?;
    validate_guest_host_acceptance_v2(
        &acceptance,
        &handoff,
        &receipt,
        &retry,
        &protected_cas,
        expected_host_publisher_spki_der,
    )?;
    validate_guest_effects_response_v2(
        &response,
        &receipt,
        &acceptance,
        expected_host_publisher_spki_der,
    )?;
    validate_guest_parity_proof_v2(&parity, &receipt, &response)?;
    let acceptance_sha256 = document_sha256_v2(&acceptance)?;
    let parity_sha256 = document_sha256_v2(&parity)?;
    let parity_binding_sha256 = document_sha256_v2(&parity_binding)?;
    let target_sha256 = document_sha256_v2(
        receipt
            .target_ledger
            .first()
            .context("guest receipt lacks its fixed target")?,
    )?;
    for (index, (generation, _generation_sha256)) in journal.iter().enumerate() {
        let previous = index
            .checked_sub(1)
            .map(|previous| (&journal[previous].0, journal[previous].1.as_str()));
        validate_guest_journal_transition_v2(previous, generation)?;
        if generation.evidence_id != receipt.evidence_id || generation.scope_id != receipt.scope_id
        {
            bail!("guest journal generation crossed its accepted evidence or scope")
        }
        if generation.generation >= 7
            && generation.handoff_request_digest.as_deref()
                != Some(handoff.handoff_request_digest.as_str())
        {
            bail!("guest journal generation changed its acknowledgement-CAS request")
        }
        if generation.generation >= 8
            && generation.host_acceptance_sha256.as_deref() != Some(acceptance_sha256.as_str())
        {
            bail!("guest journal generation changed its surviving-host acceptance")
        }
        if matches!(
            generation.event,
            GuestJournalEventKindV2::EffectPrepared
                | GuestJournalEventKindV2::EffectInvoked
                | GuestJournalEventKindV2::EffectObserved
        ) && generation.target_identity_sha256.as_deref() != Some(target_sha256.as_str())
        {
            bail!("guest journal effect generation does not name the one fixed target")
        }
        if matches!(
            generation.event,
            GuestJournalEventKindV2::EffectObserved | GuestJournalEventKindV2::GuestRemoved
        ) && generation.observation_sha256.as_deref()
            != Some(response.effects_observation_sha256.as_str())
        {
            bail!("guest journal removal observation does not match the signed response")
        }
    }
    let observed_index = journal
        .iter()
        .position(|(generation, _)| generation.event == GuestJournalEventKindV2::EffectObserved)
        .context("guest terminal journal lacks EffectObserved")?;
    let removed_index = observed_index + 1;
    let parity_index = observed_index + 2;
    let parity_bound_index = observed_index + 3;
    if parity_bound_index + 1 != journal.len()
        || bundle.guest_journal[..7] != handoff.guest_journal
        || journal[6].1 != handoff.guest_journal_head_sha256
        || journal[7].0.predecessor_head_sha256 != acceptance.predecessor_journal_head_sha256
        || journal[7].0.observation_sha256.as_deref() != Some(acceptance_sha256.as_str())
        || journal[removed_index].1 != response.journal_head_sha256
        || journal[parity_index].0.observation_sha256.as_deref() != Some(parity_sha256.as_str())
        || journal[parity_index].1 != parity_binding.journal_head_sha256
        || journal[parity_bound_index].0.observation_sha256.as_deref()
            != Some(parity_binding_sha256.as_str())
    {
        bail!("guest journal does not exact-bind acceptance, effects, parity, and host CAS")
    }
    validate_guest_parity_host_binding_v2(
        &parity_binding,
        &response,
        &parity,
        &acceptance,
        &journal[parity_index].1,
        expected_host_publisher_spki_der,
    )?;
    let expected = derive_guest_to_host_successor_capsule_v2(GuestToHostSuccessorBindingV2 {
        handoff: &handoff,
        receipt: &receipt,
        retry: &retry,
        protected_cas: &protected_cas,
        acceptance: &acceptance,
        effects_response: &response,
        parity_proof: &parity,
        parity_host_binding: &parity_binding,
        terminal_journal_head_sha256: &journal[parity_bound_index].1,
    })?;
    if bundle.evidence_id != receipt.evidence_id
        || bundle.scope_id != receipt.scope_id
        || bundle.successor_capsule != expected
    {
        bail!("guest terminal bundle does not exact-bind every accepted terminal artifact")
    }
    Ok(expected)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestEffectRecoveryPhaseV2 {
    NotPrepared,
    Prepared,
    Invoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestEffectObservationV2 {
    ExactBefore,
    ExactFinal,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestEffectRecoveryDecisionV2 {
    PersistPrepared,
    Invoke,
    RecordObservedWithoutInvocation,
    Preserve,
}

pub fn guest_effect_recovery_decision_v2(
    phase: GuestEffectRecoveryPhaseV2,
    observation: GuestEffectObservationV2,
) -> GuestEffectRecoveryDecisionV2 {
    match (phase, observation) {
        (GuestEffectRecoveryPhaseV2::NotPrepared, GuestEffectObservationV2::ExactBefore) => {
            GuestEffectRecoveryDecisionV2::PersistPrepared
        }
        (GuestEffectRecoveryPhaseV2::Prepared, GuestEffectObservationV2::ExactBefore) => {
            GuestEffectRecoveryDecisionV2::Invoke
        }
        // The one guest target is removed by an external limactl process. A publisher crash
        // after spawn can leave that exact child in flight, so target presence alone cannot
        // prove that a second invocation is safe. ExactBefore after durable EffectInvoked is
        // therefore ambiguity-preserving; a later ExactFinal observation may still complete.
        (GuestEffectRecoveryPhaseV2::Invoked, GuestEffectObservationV2::ExactBefore) => {
            GuestEffectRecoveryDecisionV2::Preserve
        }
        (GuestEffectRecoveryPhaseV2::Invoked, GuestEffectObservationV2::ExactFinal) => {
            GuestEffectRecoveryDecisionV2::RecordObservedWithoutInvocation
        }
        _ => GuestEffectRecoveryDecisionV2::Preserve,
    }
}

pub fn guest_removal_is_admitted_v2(
    state: GuestRetirementStateV2,
    receipt_externally_durable: bool,
    acknowledgement_externally_durable: bool,
    acknowledgement_cas_bound: bool,
    surviving_host_accepted: bool,
) -> bool {
    receipt_externally_durable
        && acknowledgement_externally_durable
        && acknowledgement_cas_bound
        && surviving_host_accepted
        && matches!(
            state,
            GuestRetirementStateV2::HandoffHostBound | GuestRetirementStateV2::Removing
        )
}

pub fn validate_guest_to_host_successor_capsule_v2(
    capsule: &GuestToHostSuccessorCapsuleV2,
) -> Result<()> {
    require_schema(
        &capsule.schema_owner,
        capsule.schema_version,
        MAC_R3_GUEST_TO_HOST_CAPSULE_OWNER_V2,
    )?;
    require_plain(&capsule.evidence_id, "guest successor evidence ID")?;
    require_uuid_v7(&capsule.scope_id, "guest successor scope")?;
    for (value, label) in [
        (&capsule.handoff_request_digest, "guest handoff request"),
        (
            &capsule.guest_handoff_capsule_sha256,
            "guest handoff capsule",
        ),
        (
            &capsule.guest_host_acceptance_sha256,
            "guest host acceptance",
        ),
        (&capsule.guest_receipt_sha256, "guest receipt"),
        (
            &capsule.guest_acknowledgement_sha256,
            "guest acknowledgement",
        ),
        (
            &capsule.guest_protected_cas_head_sha256,
            "guest protected CAS head",
        ),
        (&capsule.guest_effect_plan_sha256, "guest effect plan"),
        (
            &capsule.guest_effects_response_sha256,
            "guest effects response",
        ),
        (&capsule.guest_parity_proof_sha256, "guest parity proof"),
        (
            &capsule.guest_parity_host_binding_sha256,
            "guest parity host binding",
        ),
        (&capsule.guest_journal_head_sha256, "guest journal head"),
        (
            &capsule.r6_predecessor_consumed_sha256,
            "R6 consumed predecessor",
        ),
        (
            &capsule.r6_terminal_host_record_sha256,
            "R6 terminal host record",
        ),
        (
            &capsule.guest_anchor_acknowledgement_sha256,
            "guest anchor acknowledgement",
        ),
        (
            &capsule.guest_consumption_marker_acknowledgement_sha256,
            "guest consumption-marker acknowledgement",
        ),
        (&capsule.retry_state_sha256, "guest retry state"),
    ] {
        require_digest(value, label)?;
    }
    Ok(())
}

pub fn validate_publisher_pre_removal_receipt_v2(
    receipt: &PublisherPreRemovalReceiptV2,
) -> Result<()> {
    require_schema(
        &receipt.schema_owner,
        receipt.schema_version,
        MAC_R3_HOST_RECEIPT_OWNER_V2,
    )?;
    if receipt.signature_domain != MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2
        || receipt.host_state != HostRetirementStateV2::PreRemovalReceiptSigned
        || receipt.issued_at_unix_ns >= receipt.expires_at_unix_ns
        || receipt.protected_cas_generation == 0
    {
        bail!("host pre-removal receipt chronology or authority window is invalid")
    }
    require_uuid_v7(&receipt.scope_id, "receipt scope")?;
    validate_guest_to_host_successor_capsule_v2(&receipt.guest_successor_capsule)?;
    if receipt.guest_successor_capsule.scope_id != receipt.scope_id
        || receipt.guest_successor_capsule_sha256
            != document_sha256_v2(&receipt.guest_successor_capsule)?
        || receipt.guest_parity_sha256 != receipt.guest_successor_capsule.guest_parity_proof_sha256
    {
        bail!("host receipt does not exact-bind the accepted guest successor capsule")
    }
    for (value, label) in [
        (
            &receipt.guest_successor_capsule_sha256,
            "guest successor capsule",
        ),
        (&receipt.guest_parity_sha256, "guest parity"),
        (&receipt.before_observation_sha256, "before observation"),
        (&receipt.quiesced_observation_sha256, "quiesced observation"),
        (&receipt.target_ledger_sha256, "target ledger"),
        (&receipt.effect_plan_sha256, "effect plan"),
        (&receipt.protected_cas_head_sha256, "protected CAS head"),
        (&receipt.current_lock_identity_sha256, "current lock"),
        (
            &receipt.signer_access_control_sha256,
            "signer access control",
        ),
        (&receipt.capability_digest, "capability"),
    ] {
        require_digest(value, label)?;
    }
    validate_executable_identity_v2(
        &receipt.finalizer_identity,
        MAC_R3_FINALIZER_PATH_V2,
        MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
    )?;
    validate_executable_identity_v2(
        &receipt.coordinator_identity,
        MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
    )?;
    validate_process_identity_v2(&receipt.coordinator_process)?;
    if receipt.coordinator_process.executable_identity_sha256
        != document_sha256_v2(&receipt.coordinator_identity)?
    {
        bail!("coordinator process does not bind the exact executable identity")
    }
    validate_launch_identity_v2(&receipt.launch_identity)?;
    let plan = derive_host_effect_plan_v2(receipt.target_set_kind, &receipt.target_ledger)?;
    if document_sha256_v2(&receipt.target_ledger)? != receipt.target_ledger_sha256
        || document_sha256_v2(&plan)? != receipt.effect_plan_sha256
    {
        bail!("receipt target ledger or fixed effect plan digest changed")
    }
    let spki = decode_base64url(&receipt.publisher_signer_spki_der, "publisher SPKI")?;
    let harness = decode_base64url(&receipt.harness_public_key, "harness public key")?;
    if harness.len() != 32
        || receipt.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || receipt.signature.public_key != receipt.publisher_signer_spki_der
    {
        bail!("receipt signer or harness key identity is invalid")
    }
    let signature = decode_base64url(&receipt.signature.signature, "receipt signature")?;
    verify_p256_p1363_low_s_v1(
        &spki,
        &signature_payload_v2(&receipt.signature_domain, &receipt.schema_owner, receipt)?,
        &signature,
    )
}

pub fn validate_harness_acknowledgement_v2(
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
    receipt: &PublisherPreRemovalReceiptV2,
) -> Result<()> {
    require_schema(
        &acknowledgement.schema_owner,
        acknowledgement.schema_version,
        MAC_R3_HARNESS_ACK_OWNER_V2,
    )?;
    if acknowledgement.signature_domain != MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2
        || acknowledgement.evidence_id != receipt.evidence_id
        || acknowledgement.scope_id != receipt.scope_id
        || acknowledgement.receipt_sha256 != document_sha256_v2(receipt)?
        || acknowledgement.acknowledged_at_unix_ns < receipt.issued_at_unix_ns
        || acknowledgement.acknowledged_at_unix_ns > receipt.expires_at_unix_ns
        || acknowledgement.signature.algorithm != "ed25519-v1"
        || acknowledgement.signature.public_key != receipt.harness_public_key
    {
        bail!("harness acknowledgement does not bind the exact durable receipt")
    }
    require_digest(
        &acknowledgement.external_store_identity_sha256,
        "acknowledgement external store",
    )?;
    require_digest(
        &acknowledgement.durable_observation_sha256,
        "acknowledgement durable observation",
    )?;
    let key = decode_base64url(&acknowledgement.signature.public_key, "harness public key")?;
    let signature = decode_base64url(&acknowledgement.signature.signature, "harness signature")?;
    verify_ed25519_fixed_v1(
        &key,
        &signature_payload_v2(
            &acknowledgement.signature_domain,
            &acknowledgement.schema_owner,
            acknowledgement,
        )?,
        &signature,
    )
}

pub fn validate_protected_cas_binding_v2(
    binding: &ProtectedCasBindingV2,
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
) -> Result<()> {
    require_schema(
        &binding.schema_owner,
        binding.schema_version,
        MAC_R3_PROTECTED_CAS_OWNER_V2,
    )?;
    if binding.signature_domain != MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2
        || binding.evidence_id != receipt.evidence_id
        || binding.scope_id != receipt.scope_id
        || binding.generation != receipt.protected_cas_generation + 1
        || binding.predecessor_head_sha256 != receipt.protected_cas_head_sha256
        || binding.receipt_sha256 != document_sha256_v2(receipt)?
        || binding.acknowledgement_sha256 != document_sha256_v2(acknowledgement)?
        || binding.target_ledger_sha256 != receipt.target_ledger_sha256
        || binding.effect_plan_sha256 != receipt.effect_plan_sha256
        || binding.guest_successor_capsule_sha256 != receipt.guest_successor_capsule_sha256
        || binding.guest_parity_sha256 != receipt.guest_parity_sha256
        || binding.current_lock_identity_sha256 != receipt.current_lock_identity_sha256
        || binding.signer_access_control_sha256 != receipt.signer_access_control_sha256
        || binding.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || binding.signature.public_key != receipt.publisher_signer_spki_der
    {
        bail!("protected CAS does not exact-bind the receipt and acknowledgement")
    }
    require_digest(&binding.request_digest, "protected CAS request digest")?;
    let key = decode_base64url(&binding.signature.public_key, "protected CAS SPKI")?;
    let signature = decode_base64url(&binding.signature.signature, "protected CAS signature")?;
    verify_p256_p1363_low_s_v1(
        &key,
        &signature_payload_v2(&binding.signature_domain, &binding.schema_owner, binding)?,
        &signature,
    )
}

pub fn validate_finalization_request_v2(
    request: &FinalizationRequestV2,
    first_acceptance_at_unix_ns: Option<u64>,
) -> Result<(
    PublisherPreRemovalReceiptV2,
    HarnessDurabilityAcknowledgementV2,
    ProtectedCasBindingV2,
)> {
    require_schema(
        &request.schema_owner,
        request.schema_version,
        MAC_R3_FINALIZER_PROTOCOL_OWNER_V2,
    )?;
    require_digest(&request.request_digest, "finalization request digest")?;
    let receipt_bytes = decode_base64url(&request.publisher_receipt, "publisher receipt bytes")?;
    let acknowledgement_bytes = decode_base64url(
        &request.harness_acknowledgement,
        "harness acknowledgement bytes",
    )?;
    let binding_bytes = decode_base64url(&request.protected_cas_binding, "protected CAS bytes")?;
    let receipt: PublisherPreRemovalReceiptV2 = parse_canonical_v2(&receipt_bytes)?;
    let acknowledgement: HarnessDurabilityAcknowledgementV2 =
        parse_canonical_v2(&acknowledgement_bytes)?;
    let binding: ProtectedCasBindingV2 = parse_canonical_v2(&binding_bytes)?;
    validate_publisher_pre_removal_receipt_v2(&receipt)?;
    validate_harness_acknowledgement_v2(&acknowledgement, &receipt)?;
    validate_protected_cas_binding_v2(&binding, &receipt, &acknowledgement)?;
    if let Some(now) = first_acceptance_at_unix_ns {
        if now < receipt.issued_at_unix_ns || now > receipt.expires_at_unix_ns {
            bail!("finalization request is outside its first-acceptance window")
        }
    }
    let capsule = &request.successor_capsule;
    for (value, label) in [
        (
            &capsule.predecessor_journal_head_sha256,
            "predecessor journal head",
        ),
        (&capsule.retry_state_sha256, "retry state"),
    ] {
        require_digest(value, label)?;
    }
    let intent = &capsule.intent;
    validate_frozen_finalization_intent_v2(intent, &receipt, &acknowledgement, &binding)?;
    let digest = document_sha256_v2(capsule)?;
    if request.request_digest != digest || binding.request_digest != digest {
        bail!("alternate finalization request digest is rejected")
    }
    Ok((receipt, acknowledgement, binding))
}

pub fn canonical_finalization_request_v2(request: &FinalizationRequestV2) -> Result<Vec<u8>> {
    validate_finalization_request_v2(request, None)?;
    canonical_bytes_v2(request)
}

pub fn validate_finalizer_response_v2(response: &FinalizerResponseV2) -> Result<()> {
    require_schema(
        &response.schema_owner,
        response.schema_version,
        MAC_R3_FINALIZER_PROTOCOL_OWNER_V2,
    )?;
    require_digest(&response.request_digest, "response request")?;
    require_digest(&response.journal_head_sha256, "response journal head")?;
    match response.state {
        FinalizerResponseStateV2::SafePreAcceptanceStop => {
            if response.effects_observation_sha256.is_some()
                || response.terminal_acknowledgement_sha256.is_some()
                || response.preserving_classification.is_none()
            {
                bail!("safe pre-acceptance response is not preserving-only")
            }
        }
        FinalizerResponseStateV2::RejoinAcceptedRequest => {
            if response.effects_observation_sha256.is_some()
                || response.terminal_acknowledgement_sha256.is_some()
                || response.preserving_classification.is_some()
            {
                bail!("accepted rejoin response contains a terminal or preserving result")
            }
        }
        FinalizerResponseStateV2::PreservingStop => {
            if response.terminal_acknowledgement_sha256.is_some()
                || response.preserving_classification.is_none()
            {
                bail!("post-acceptance preserving stop is not closed")
            }
            if let Some(observation) = response.effects_observation_sha256.as_deref() {
                require_digest(observation, "preserving-stop observation")?;
            }
        }
        FinalizerResponseStateV2::EffectsComplete => {
            let observation = response
                .effects_observation_sha256
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("effects-complete response lacks observation"))?;
            require_digest(observation, "effects observation")?;
            if response.terminal_acknowledgement_sha256.is_some()
                || response.preserving_classification.is_some()
            {
                bail!("effects-complete response contains a terminal or preserving result")
            }
        }
        FinalizerResponseStateV2::HostComplete => {
            let observation = response
                .effects_observation_sha256
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("host-complete response lacks observation"))?;
            let acknowledgement = response
                .terminal_acknowledgement_sha256
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("host-complete response lacks terminal ack"))?;
            require_digest(observation, "effects observation")?;
            require_digest(acknowledgement, "terminal acknowledgement")?;
            if response.preserving_classification.is_some() {
                bail!("host-complete response contains a preserving classification")
            }
        }
    }
    Ok(())
}

pub fn validate_host_parity_proof_v2(
    proof: &HostParityProofV2,
    harness_public_key: &str,
) -> Result<()> {
    require_schema(
        &proof.schema_owner,
        proof.schema_version,
        MAC_R3_PARITY_OWNER_V2,
    )?;
    if proof.signature_domain != MAC_R3_PARITY_SIGNATURE_DOMAIN_V2 || !proof.exact_parity {
        bail!("host parity proof is not an exact successful parity observation")
    }
    for (value, label) in [
        (&proof.request_digest, "parity request"),
        (&proof.effects_response_sha256, "parity response"),
        (&proof.journal_head_sha256, "parity journal head"),
        (&proof.baseline_sha256, "parity baseline"),
        (&proof.after_observation_sha256, "parity after observation"),
    ] {
        require_digest(value, label)?;
    }
    if proof.signature.algorithm != "ed25519-v1" || proof.signature.public_key != harness_public_key
    {
        bail!("parity proof signer is not the committed harness")
    }
    let key = decode_base64url(&proof.signature.public_key, "parity public key")?;
    let signature = decode_base64url(&proof.signature.signature, "parity signature")?;
    verify_ed25519_fixed_v1(
        &key,
        &signature_payload_v2(&proof.signature_domain, &proof.schema_owner, proof)?,
        &signature,
    )
}

pub fn validate_terminal_acknowledgement_v2(
    acknowledgement: &TerminalAcknowledgementV2,
    parity: &HostParityProofV2,
    harness_public_key: &str,
) -> Result<()> {
    require_schema(
        &acknowledgement.schema_owner,
        acknowledgement.schema_version,
        MAC_R3_TERMINAL_ACK_OWNER_V2,
    )?;
    if acknowledgement.signature_domain != MAC_R3_TERMINAL_ACK_SIGNATURE_DOMAIN_V2
        || acknowledgement.evidence_id != parity.evidence_id
        || acknowledgement.scope_id != parity.scope_id
        || acknowledgement.request_digest != parity.request_digest
        || acknowledgement.effects_response_sha256 != parity.effects_response_sha256
        || acknowledgement.parity_proof_sha256 != document_sha256_v2(parity)?
        || acknowledgement.journal_head_sha256 != parity.journal_head_sha256
        || acknowledgement.signature.algorithm != "ed25519-v1"
        || acknowledgement.signature.public_key != harness_public_key
    {
        bail!("terminal acknowledgement does not exact-bind parity and effects")
    }
    let key = decode_base64url(&acknowledgement.signature.public_key, "terminal public key")?;
    let signature = decode_base64url(&acknowledgement.signature.signature, "terminal signature")?;
    verify_ed25519_fixed_v1(
        &key,
        &signature_payload_v2(
            &acknowledgement.signature_domain,
            &acknowledgement.schema_owner,
            acknowledgement,
        )?,
        &signature,
    )
}

pub fn encode_base64url_v2(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn derive_frozen_finalization_intent_v2(
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
    binding: &ProtectedCasBindingV2,
) -> Result<FrozenFinalizationIntentV2> {
    Ok(FrozenFinalizationIntentV2 {
        evidence_id: receipt.evidence_id.clone(),
        scope_id: receipt.scope_id.clone(),
        receipt_sha256: document_sha256_v2(receipt)?,
        acknowledgement_sha256: document_sha256_v2(acknowledgement)?,
        protected_cas_generation: binding.generation,
        protected_cas_predecessor_head_sha256: binding.predecessor_head_sha256.clone(),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
        guest_successor_capsule_sha256: receipt.guest_successor_capsule_sha256.clone(),
        guest_parity_sha256: receipt.guest_parity_sha256.clone(),
        current_lock_identity_sha256: receipt.current_lock_identity_sha256.clone(),
        signer_access_control_sha256: receipt.signer_access_control_sha256.clone(),
        finalizer_identity_sha256: document_sha256_v2(&receipt.finalizer_identity)?,
        coordinator_identity_sha256: document_sha256_v2(&receipt.coordinator_identity)?,
        launch_identity_sha256: document_sha256_v2(&receipt.launch_identity)?,
        capability_digest: receipt.capability_digest.clone(),
    })
}

pub fn validate_finalization_authority_binding_v2(
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
    binding: &ProtectedCasBindingV2,
) -> Result<FrozenFinalizationIntentV2> {
    validate_publisher_pre_removal_receipt_v2(receipt)?;
    validate_harness_acknowledgement_v2(acknowledgement, receipt)?;
    validate_protected_cas_binding_v2(binding, receipt, acknowledgement)?;
    derive_frozen_finalization_intent_v2(receipt, acknowledgement, binding)
}

pub fn validate_frozen_finalization_intent_v2(
    intent: &FrozenFinalizationIntentV2,
    receipt: &PublisherPreRemovalReceiptV2,
    acknowledgement: &HarnessDurabilityAcknowledgementV2,
    binding: &ProtectedCasBindingV2,
) -> Result<()> {
    let expected = derive_frozen_finalization_intent_v2(receipt, acknowledgement, binding)?;
    if *intent != expected {
        bail!("successor capsule intent does not exact-bind accepted authority")
    }
    Ok(())
}

pub fn validate_executable_identity_v2(
    identity: &ExecutableIdentityV2,
    expected_path: &str,
    expected_identifier: &str,
) -> Result<()> {
    if identity.intended_path != expected_path
        || identity.signing_identifier != expected_identifier
        || identity.executable_size == 0
        || identity.designated_requirement.is_empty()
        || identity.designated_requirement.contains('\0')
        || identity.cdhash.len() != 40
        || !identity
            .cdhash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("executable path, code identity, or size is not the closed literal")
    }
    require_git_oid(&identity.source_commit, "executable source commit")?;
    require_git_oid(&identity.source_tree, "executable source tree")?;
    for (value, label) in [
        (&identity.source_hashes_sha256, "source hashes"),
        (&identity.build_inputs_sha256, "build inputs"),
        (&identity.executable_sha256, "executable image"),
        (&identity.physical_identity_sha256, "physical identity"),
    ] {
        require_digest(value, label)?;
    }
    Ok(())
}

pub fn validate_process_identity_v2(identity: &ProcessIdentityV2) -> Result<()> {
    if identity.canonical_account.is_empty()
        || identity.canonical_account.contains(['\0', '\n', '\r'])
        || identity.effective_gid != MAC_R3_COORDINATOR_EFFECTIVE_GID_V2
        || !identity.pidversion_required
    {
        bail!("process identity lacks a canonical account or PID-version requirement")
    }
    require_digest(
        &identity.process_start_identity_sha256,
        "process start identity",
    )?;
    require_digest(
        &identity.executable_identity_sha256,
        "process executable identity",
    )
}

pub fn validate_process_executable_binding_v2(
    process: &ProcessIdentityV2,
    executable: &ExecutableIdentityV2,
    expected_path: &str,
    expected_identifier: &str,
) -> Result<()> {
    validate_process_identity_v2(process)?;
    validate_executable_identity_v2(executable, expected_path, expected_identifier)?;
    if process.executable_identity_sha256 != document_sha256_v2(executable)? {
        bail!("process does not bind the exact executable identity")
    }
    Ok(())
}

pub fn validate_launch_identity_v2(identity: &LaunchIdentityV2) -> Result<()> {
    if identity.launchd_label != MAC_R3_FINALIZER_LAUNCHD_LABEL_V2
        || identity.launchd_plist_path != MAC_R3_FINALIZER_PLIST_PATH_V2
        || identity.endpoint != MAC_R3_FINALIZER_ENDPOINT_V2
        || identity.endpoint_owner_uid != 0
        || identity.endpoint_group_gid != 20
        || identity.endpoint_mode != "0660"
        || identity.launch_socket_name != "Listener"
        || identity.finalizer_effective_uid != 0
    {
        bail!("launchd route or endpoint identity is not the fixed privilege route")
    }
    require_digest(&identity.launchd_plist_sha256, "launchd plist")
}

fn require_schema(owner: &str, version: u32, expected_owner: &str) -> Result<()> {
    if owner != expected_owner || version != MAC_R3_FINALIZER_PROTOCOL_VERSION_V2 {
        bail!("R3 schema owner or version is not accepted")
    }
    Ok(())
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} must be lowercase SHA-256")
    }
    Ok(())
}

fn require_git_oid(value: &str, label: &str) -> Result<()> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} must be a lowercase Git SHA-1")
    }
    Ok(())
}

fn require_plain(value: &str, label: &str) -> Result<()> {
    if value.is_empty() || value.contains(['\0', '\n', '\r']) {
        bail!("{label} is empty or contains a control separator")
    }
    Ok(())
}

fn require_uuid_v7(value: &str, label: &str) -> Result<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || ![8_usize, 13, 18, 23]
            .iter()
            .all(|offset| bytes[*offset] == b'-')
        || bytes[14] != b'7'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
        || bytes.iter().enumerate().any(|(offset, byte)| {
            !matches!(offset, 8 | 13 | 18 | 23)
                && (!byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        })
    {
        bail!("{label} must be an exact UUIDv7")
    }
    Ok(())
}

fn decode_base64url(value: &str, label: &str) -> Result<Vec<u8>> {
    if value.is_empty() || value.contains('=') {
        bail!("{label} is empty or padded")
    }
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .with_context(|| format!("decode {label}"))?;
    if URL_SAFE_NO_PAD.encode(&decoded) != value {
        bail!("{label} is not canonical base64url")
    }
    Ok(decoded)
}

struct StrictCanonicalJsonValueV2(Value);

impl<'de> Deserialize<'de> for StrictCanonicalJsonValueV2 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictCanonicalJsonVisitorV2)
    }
}

struct StrictCanonicalJsonVisitorV2;

impl<'de> Visitor<'de> for StrictCanonicalJsonVisitorV2 {
    type Value = StrictCanonicalJsonValueV2;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("canonical JSON without duplicate keys or floating-point values")
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictCanonicalJsonValueV2(Value::Null))
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_unit()
    }

    fn visit_bool<E>(self, value: bool) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictCanonicalJsonValueV2(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictCanonicalJsonValueV2(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictCanonicalJsonValueV2(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, _value: f64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(E::custom("canonical R3 JSON rejects floating-point values"))
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_string(value.to_owned())
    }

    fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictCanonicalJsonValueV2(Value::String(value)))
    }

    fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(value) = sequence.next_element::<StrictCanonicalJsonValueV2>()? {
            values.push(value.0);
        }
        Ok(StrictCanonicalJsonValueV2(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = serde_json::Map::new();
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!(
                    "canonical R3 JSON rejects duplicate key {key:?}"
                )));
            }
            let value = map.next_value::<StrictCanonicalJsonValueV2>()?;
            object.insert(key, value.0);
        }
        Ok(StrictCanonicalJsonValueV2(Value::Object(object)))
    }
}

fn encode_canonical_json_v2(value: &Value, output: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => {
            if number.is_f64() {
                bail!("canonical R3 JSON rejects floating-point values")
            }
            output.extend_from_slice(number.to_string().as_bytes());
        }
        Value::String(string) => output.extend_from_slice(
            serde_json::to_string(string)
                .context("encode canonical R3 JSON string")?
                .as_bytes(),
        ),
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                encode_canonical_json_v2(value, output)?;
            }
            output.push(b']');
        }
        Value::Object(object) => {
            output.push(b'{');
            let mut keys: Vec<_> = object.keys().collect();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                output.extend_from_slice(
                    serde_json::to_string(key)
                        .context("encode canonical R3 JSON key")?
                        .as_bytes(),
                );
                output.push(b':');
                encode_canonical_json_v2(&object[key], output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}
