#![deny(unsafe_op_in_unsafe_fn)]
#![cfg(target_os = "macos")]

//! Exact legacy `SecAccess` experiment for a System-Keychain P-256 signer.
//!
//! This crate is deliberately standalone and `publish = false`.  It provides an experiment
//! surface, not a production authority boundary.  Every native operation is explicit; unit tests
//! exercise only the pure configuration and canonical-posture logic.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path};

pub use substrate_r3_macos_finalizer::experiment::controls::{
    NobodyAuthorizationDenialV2, NobodyOwnerAuthorityOperationV2,
};
use substrate_r3_macos_finalizer::experiment::process::SupplementaryGroupAttestationV2;
pub use substrate_r3_macos_finalizer::experiment::publisher_protocol::DisposableSignerIdentityV2;

mod attestation;
#[doc(hidden)]
pub mod experiment;
mod ffi;
mod immutable_publish;
mod owner_probe;
#[path = "publisher_surface.rs"]
mod publisher;
pub mod runner;
mod runner_identity;
mod securityagent;
pub mod supervisor;

pub use owner_probe::{probe_nonmatch_owner_candidate, NonmatchOwnerProbeV2};

/// Create and copy-read back only an in-memory legacy SecAccess owner candidate. No Keychain is
/// opened and no item is created. A caller must retain the returned canonical digest as pre-effect
/// evidence; failure means the disposable ACL posture is not available on this host.
pub fn probe_nonmatch_owner_access_in_memory() -> Result<CanonicalAccessDigest> {
    ffi::probe_nonmatch_owner_access_in_memory_impl()
}

pub const SYSTEM_KEYCHAIN_PATH: &str = "/Library/Keychains/System.keychain";
/// Candidate legacy owner values. They are deliberately the special all-ones IDs, never an
/// ordinary account. Native use is gated by [`probe_nonmatch_owner_candidate`] and exact
/// SecAccess owner readback; until that live in-memory probe succeeds this posture is not frozen.
pub const NONMATCH_OWNER_UID: u32 = u32::MAX;
pub const NONMATCH_OWNER_GID: u32 = u32::MAX;
pub const OWNER_TYPE_USE_ONLY_UID_AND_GID: u32 = 1 | 2;
pub const PROMPT_SELECTOR_NONE: u16 = 0;
pub const MAX_APPLICATION_TAG_BYTES: usize = 1024;
pub const MAX_LABEL_BYTES: usize = 255;
pub const MAX_SIGNING_PAYLOAD_BYTES: usize = 1024 * 1024;

pub const CREATOR_EXECUTABLE_PATH: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-creator.v1";
pub const WRONG_IDENTITY_EXECUTABLE_PATH: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1";
pub const DISPOSABLE_PUBLISHER_EXECUTABLE_PATH: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-disposable-publisher.v2";
pub const EXPERIMENT_FINALIZER_EXECUTABLE_PATH: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-evidence-finalizer.v2";
pub const MARKER_ROOT: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-signer-acl-experiment.v2";
pub const MARKER_PATH: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-signer-acl-experiment.v2/creator-route.v2";
pub const EXPERIMENT_APPLICATION_TAG: &[u8] =
    b"com.atomize.substrate.r3-macos-signer-acl-experiment.v1:product-equivalent-p256";
pub const EXPERIMENT_LABEL: &str =
    "com.atomize.substrate.r3-macos-signer-acl-experiment.v1.product-equivalent-p256";
pub const EXPLICIT_ACL_APPLICATION_TAG: &[u8] =
    b"com.atomize.substrate.r3-macos-signer-acl-experiment.v2:explicit-acl-p256";
pub const EXPLICIT_ACL_LABEL: &str =
    "com.atomize.substrate.r3-macos-signer-acl-experiment.v2.explicit-acl-p256";

pub const CREATOR_REPETITION_SCOPE_1: &str = "019ffeb5-b24c-72a7-80bc-e629f85b37c3";
pub const CREATOR_REPETITION_SCOPE_2: &str = "019ffeb5-b24f-7834-bb4d-44885040002f";
pub const DISPOSABLE_FINALIZER_SCOPE_1: &str = "019ffeb5-b252-79ae-8f41-e161419fbbcd";
pub const DISPOSABLE_FINALIZER_SCOPE_2: &str = "019ffeb5-b255-75a5-870f-49323ebb2c19";
pub const DISPOSABLE_PUBLISHER_ROOT: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-publisher.v2";
pub const DISPOSABLE_PUBLISHER_STATE_PATH: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-publisher.v2/state.v2";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum FixedRepetitionV2 {
    First,
    Second,
}

impl FixedRepetitionV2 {
    pub const ALL: [Self; 2] = [Self::First, Self::Second];

    pub const fn ordinal(self) -> u8 {
        match self {
            Self::First => 1,
            Self::Second => 2,
        }
    }

    pub const fn creator_scope(self) -> &'static str {
        match self {
            Self::First => CREATOR_REPETITION_SCOPE_1,
            Self::Second => CREATOR_REPETITION_SCOPE_2,
        }
    }

    pub const fn finalizer_scope(self) -> &'static str {
        match self {
            Self::First => DISPOSABLE_FINALIZER_SCOPE_1,
            Self::Second => DISPOSABLE_FINALIZER_SCOPE_2,
        }
    }
}

pub const PUBLISHER_SIGN_DELETE_DESCRIPTION: &str = "publisher-sign-and-delete";
pub const FINALIZER_DELETE_DESCRIPTION: &str = "finalizer-delete-only";
pub const PRIVATE_OPERATION_DENY_DESCRIPTION: &str = "deny-private-key-operations";
pub const OWNER_MUTATION_DENY_DESCRIPTION: &str = "deny-owner-and-acl-mutation";

pub const AUTH_SIGN: &str = "sign";
pub const AUTH_DELETE: &str = "delete";
pub const AUTH_EXPORT_WRAPPED: &str = "export_wrapped";
pub const AUTH_EXPORT_CLEAR: &str = "export_clear";
pub const AUTH_IMPORT_WRAPPED: &str = "import_wrapped";
pub const AUTH_IMPORT_CLEAR: &str = "import_clear";
pub const AUTH_ENCRYPT: &str = "encrypt";
pub const AUTH_DECRYPT: &str = "decrypt";
pub const AUTH_MAC: &str = "mac";
pub const AUTH_DERIVE: &str = "derive";
pub const AUTH_CHANGE_ACL: &str = "change_acl";
pub const AUTH_CHANGE_OWNER: &str = "change_owner";
pub const AUTH_ANY: &str = "any";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactSignerConfig {
    system_keychain_path: String,
    publisher_path: String,
    finalizer_path: String,
    application_tag: Vec<u8>,
    label: String,
}

impl ExactSignerConfig {
    pub fn new(
        system_keychain_path: impl Into<String>,
        publisher_path: impl Into<String>,
        finalizer_path: impl Into<String>,
        application_tag: Vec<u8>,
        label: impl Into<String>,
    ) -> Result<Self> {
        let config = Self {
            system_keychain_path: system_keychain_path.into(),
            publisher_path: publisher_path.into(),
            finalizer_path: finalizer_path.into(),
            application_tag,
            label: label.into(),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.system_keychain_path != SYSTEM_KEYCHAIN_PATH {
            bail!("experiment must use the one explicit System Keychain path")
        }
        validate_exact_absolute_path(&self.publisher_path, "publisher")?;
        validate_exact_absolute_path(&self.finalizer_path, "finalizer")?;
        if self.publisher_path == self.finalizer_path {
            bail!("publisher and finalizer trusted applications must be distinct")
        }
        if self.application_tag.is_empty() || self.application_tag.len() > MAX_APPLICATION_TAG_BYTES
        {
            bail!("application tag is empty or exceeds its fixed bound")
        }
        if self.label.is_empty()
            || self.label.len() > MAX_LABEL_BYTES
            || self.label.contains(['\0', '\n', '\r'])
        {
            bail!("key label is empty, unsafe, or exceeds its fixed bound")
        }
        Ok(())
    }

    pub fn system_keychain_path(&self) -> &str {
        &self.system_keychain_path
    }

    pub fn publisher_path(&self) -> &str {
        &self.publisher_path
    }

    pub fn finalizer_path(&self) -> &str {
        &self.finalizer_path
    }

    pub fn application_tag(&self) -> &[u8] {
        &self.application_tag
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Return the one compiled product-equivalent experiment identity.  The binaries do not accept
/// target, tag, label, path, or action arguments.
pub fn compiled_product_equivalent_config() -> Result<ExactSignerConfig> {
    ExactSignerConfig::new(
        SYSTEM_KEYCHAIN_PATH,
        CREATOR_EXECUTABLE_PATH,
        EXPERIMENT_FINALIZER_EXECUTABLE_PATH,
        EXPERIMENT_APPLICATION_TAG.to_vec(),
        EXPERIMENT_LABEL,
    )
}

pub fn compiled_creator_route_config(repetition: FixedRepetitionV2) -> Result<ExactSignerConfig> {
    let (tag, label) = match repetition {
        FixedRepetitionV2::First => (
            b"019ffeb5-b24c-72a7-80bc-e629f85b37c3:product-equivalent-p256".to_vec(),
            "019ffeb5-b24c-72a7-80bc-e629f85b37c3.product-equivalent-p256",
        ),
        FixedRepetitionV2::Second => (
            b"019ffeb5-b24f-7834-bb4d-44885040002f:product-equivalent-p256".to_vec(),
            "019ffeb5-b24f-7834-bb4d-44885040002f.product-equivalent-p256",
        ),
    };
    ExactSignerConfig::new(
        SYSTEM_KEYCHAIN_PATH,
        CREATOR_EXECUTABLE_PATH,
        EXPERIMENT_FINALIZER_EXECUTABLE_PATH,
        tag,
        label,
    )
}

/// Return the fixed explicit-ACL disposable signer identity. This lane deliberately does not use
/// the creator-route, coordinator, product publisher, or finalizer as its publisher principal.
pub fn compiled_explicit_acl_config() -> Result<ExactSignerConfig> {
    ExactSignerConfig::new(
        SYSTEM_KEYCHAIN_PATH,
        DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
        EXPERIMENT_FINALIZER_EXECUTABLE_PATH,
        EXPLICIT_ACL_APPLICATION_TAG.to_vec(),
        EXPLICIT_ACL_LABEL,
    )
}

/// The target signer for one of the two closed disposable finalizer repetitions.
pub(crate) fn compiled_disposable_target_config(
    repetition: FixedRepetitionV2,
) -> Result<ExactSignerConfig> {
    let (tag, label) = match repetition {
        FixedRepetitionV2::First => (
            b"019ffeb5-b252-79ae-8f41-e161419fbbcd:signing-key".to_vec(),
            "019ffeb5-b252-79ae-8f41-e161419fbbcd:signing-key",
        ),
        FixedRepetitionV2::Second => (
            b"019ffeb5-b255-75a5-870f-49323ebb2c19:signing-key".to_vec(),
            "019ffeb5-b255-75a5-870f-49323ebb2c19:signing-key",
        ),
    };
    ExactSignerConfig::new(
        SYSTEM_KEYCHAIN_PATH,
        DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
        EXPERIMENT_FINALIZER_EXECUTABLE_PATH,
        tag,
        label,
    )
}

/// The wrong-surrogate signer for one of the two closed disposable finalizer repetitions.
pub(crate) fn compiled_disposable_wrong_config(
    repetition: FixedRepetitionV2,
) -> Result<ExactSignerConfig> {
    let (tag, label) = match repetition {
        FixedRepetitionV2::First => (
            b"019ffeb5-b252-79ae-8f41-e161419fbbcd:wrong-surrogate-signing-key".to_vec(),
            "019ffeb5-b252-79ae-8f41-e161419fbbcd:wrong-surrogate-signing-key",
        ),
        FixedRepetitionV2::Second => (
            b"019ffeb5-b255-75a5-870f-49323ebb2c19:wrong-surrogate-signing-key".to_vec(),
            "019ffeb5-b255-75a5-870f-49323ebb2c19:wrong-surrogate-signing-key",
        ),
    };
    ExactSignerConfig::new(
        SYSTEM_KEYCHAIN_PATH,
        DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
        EXPERIMENT_FINALIZER_EXECUTABLE_PATH,
        tag,
        label,
    )
}

fn validate_exact_absolute_path(value: &str, label: &str) -> Result<()> {
    if value.is_empty() || value.contains('\0') || value.len() >= libc::PATH_MAX as usize {
        bail!("{label} trusted-application path is empty, unsafe, or too long")
    }
    let path = Path::new(value);
    if !path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
    {
        bail!("{label} trusted-application path is not exact and absolute")
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct TrustedApplicationDigest {
    pub data_base64url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct AclEntrySnapshot {
    pub description: String,
    pub prompt_selector: u16,
    pub authorizations: Vec<String>,
    pub trusted_applications: Vec<TrustedApplicationDigest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AccessSnapshot {
    pub owner_uid: u32,
    pub owner_gid: u32,
    pub owner_type: u32,
    pub entries: Vec<AclEntrySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CanonicalAccessDigest {
    pub snapshot: AccessSnapshot,
    pub canonical_json: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignerReadback {
    pub application_tag_base64url: String,
    pub label: String,
    pub access: CanonicalAccessDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignerCreationReadback {
    pub in_memory_access: CanonicalAccessDigest,
    pub persisted: SignerReadback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DisposableKeyPairCreationReceiptV2 {
    pub(crate) schema_owner: String,
    pub(crate) schema_version: u32,
    pub(crate) repetition: FixedRepetitionV2,
    pub(crate) scope_id: String,
    pub(crate) target: DisposableSignerIdentityV2,
    pub(crate) wrong_surrogate: DisposableSignerIdentityV2,
    pub(crate) wrapper_identity_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NegativeControl {
    ExportClear,
    ReplaceAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum NegativeControlOutcome {
    Denied { code: i64 },
    UnexpectedlySucceeded,
}

/// Frozen classification for an exact-reference deletion attempt.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExactDeleteClassification {
    DeletedAndAbsent,
    AlreadyAbsent,
    InteractionNotAllowed,
    AuthorizationDenied,
    OtherFailure,
}

/// Raw, key-material-free receipt for an exact-reference deletion attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExactDeleteReceipt {
    pub raw_os_status: i32,
    pub classification: ExactDeleteClassification,
    pub present_after: bool,
}

/// Key-material-free receipt for product-equivalent creation without `kSecAttrAccess`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProductEquivalentCreationReceipt {
    pub raw_cferror_code: i64,
    pub present_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NobodyOwnerProbeCommandV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub repetition: FixedRepetitionV2,
    pub scope_id: String,
    pub operation: NobodyOwnerAuthorityOperationV2,
    pub request_digest: String,
}

impl NobodyOwnerProbeCommandV2 {
    pub fn validate(&self) -> Result<()> {
        if self.schema_owner != "substrate.r3-macos-disposable-nobody-owner-probe-command"
            || self.schema_version != 2
            || self.scope_id != self.repetition.finalizer_scope()
            || self.request_digest.len() != 64
            || !self
                .request_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            bail!("nobody-owner probe command changed its closed scope or request")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NobodyOwnerNativeReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub repetition: FixedRepetitionV2,
    pub scope_id: String,
    pub operation: NobodyOwnerAuthorityOperationV2,
    pub request_digest: String,
    pub supplementary_groups: SupplementaryGroupAttestationV2,
    pub raw_os_status: i32,
    pub classification: NobodyAuthorizationDenialV2,
}

pub fn invoke_compiled_nobody_owner_authority_control(
    command: &NobodyOwnerProbeCommandV2,
) -> Result<NobodyOwnerNativeReceiptV2> {
    command.validate()?;
    let supplementary_groups = SupplementaryGroupAttestationV2::current_process()?;
    let mut security = NonInteractiveSecurity::establish_first()?;
    let config = compiled_disposable_target_config(command.repetition)?;
    let raw_os_status =
        security.run_nobody_owner_authority_operation(&config, command.operation)?;
    let classification = match raw_os_status {
        -25_293 => NobodyAuthorizationDenialV2::ErrSecAuthFailed,
        -25_308 => NobodyAuthorizationDenialV2::ErrSecInteractionNotAllowed,
        _ => bail!("nobody-owner native operation returned unfrozen OSStatus {raw_os_status}"),
    };
    Ok(NobodyOwnerNativeReceiptV2 {
        schema_owner: "substrate.r3-macos-disposable-nobody-owner-native-receipt".to_owned(),
        schema_version: 2,
        repetition: command.repetition,
        scope_id: command.scope_id.clone(),
        operation: command.operation,
        request_digest: command.request_digest.clone(),
        supplementary_groups,
        raw_os_status,
        classification,
    })
}

pub use ffi::{NonInteractiveSecurity, QueryUiFailSecurity};

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ExactSignerConfig {
        ExactSignerConfig::new(
            SYSTEM_KEYCHAIN_PATH,
            "/private/tmp/acl-experiment/publisher",
            "/private/tmp/acl-experiment/finalizer",
            b"acl-experiment:signing-key".to_vec(),
            "acl-experiment-signer",
        )
        .unwrap()
    }

    #[test]
    fn exact_config_rejects_broadened_paths_and_tags() {
        assert!(config().validate().is_ok());
        assert!(ExactSignerConfig::new(
            "/Library/Keychains/login.keychain-db",
            "/private/tmp/publisher",
            "/private/tmp/finalizer",
            b"tag".to_vec(),
            "label",
        )
        .is_err());
        assert!(ExactSignerConfig::new(
            SYSTEM_KEYCHAIN_PATH,
            "relative/publisher",
            "/private/tmp/finalizer",
            b"tag".to_vec(),
            "label",
        )
        .is_err());
        assert!(ExactSignerConfig::new(
            SYSTEM_KEYCHAIN_PATH,
            "/private/tmp/same",
            "/private/tmp/same",
            b"tag".to_vec(),
            "label",
        )
        .is_err());
        assert!(ExactSignerConfig::new(
            SYSTEM_KEYCHAIN_PATH,
            "/private/tmp/publisher",
            "/private/tmp/finalizer",
            Vec::new(),
            "label",
        )
        .is_err());
    }

    #[test]
    fn canonical_expected_posture_has_no_any_or_honor_root() {
        let snapshot = ffi::expected_snapshot_for_test(b"publisher", b"finalizer");
        assert_eq!(snapshot.owner_uid, NONMATCH_OWNER_UID);
        assert_eq!(snapshot.owner_gid, NONMATCH_OWNER_GID);
        assert_eq!(snapshot.owner_type, OWNER_TYPE_USE_ONLY_UID_AND_GID);
        assert_eq!(snapshot.entries.len(), 4);
        assert!(snapshot.entries.iter().all(|entry| {
            entry.prompt_selector == PROMPT_SELECTOR_NONE
                && !entry.authorizations.iter().any(|value| value == AUTH_ANY)
        }));
    }

    #[test]
    fn publisher_has_minimal_rollback_and_finalizer_remains_delete_only() {
        let snapshot = ffi::expected_snapshot_for_test(b"publisher", b"finalizer");
        let publisher = snapshot
            .entries
            .iter()
            .find(|entry| entry.description == PUBLISHER_SIGN_DELETE_DESCRIPTION)
            .unwrap();
        let finalizer = snapshot
            .entries
            .iter()
            .find(|entry| entry.description == FINALIZER_DELETE_DESCRIPTION)
            .unwrap();
        assert_eq!(publisher.authorizations, [AUTH_DELETE, AUTH_SIGN]);
        assert_eq!(finalizer.authorizations, [AUTH_DELETE]);
        assert_ne!(
            publisher.trusted_applications,
            finalizer.trusted_applications
        );
    }

    #[test]
    fn explicit_deny_entries_cover_private_and_owner_mutations() {
        let snapshot = ffi::expected_snapshot_for_test(b"publisher", b"finalizer");
        let private = snapshot
            .entries
            .iter()
            .find(|entry| entry.description == PRIVATE_OPERATION_DENY_DESCRIPTION)
            .unwrap();
        let owner = snapshot
            .entries
            .iter()
            .find(|entry| entry.description == OWNER_MUTATION_DENY_DESCRIPTION)
            .unwrap();
        assert!(private.trusted_applications.is_empty());
        assert_eq!(
            private.authorizations,
            [
                AUTH_DECRYPT,
                AUTH_DERIVE,
                AUTH_ENCRYPT,
                AUTH_EXPORT_CLEAR,
                AUTH_EXPORT_WRAPPED,
                AUTH_IMPORT_CLEAR,
                AUTH_IMPORT_WRAPPED,
                AUTH_MAC,
            ]
        );
        assert!(owner.trusted_applications.is_empty());
        assert_eq!(owner.authorizations, [AUTH_CHANGE_ACL, AUTH_CHANGE_OWNER]);
    }

    #[test]
    fn canonical_digest_is_order_independent() {
        let mut left = ffi::expected_snapshot_for_test(b"publisher", b"finalizer");
        let mut right = left.clone();
        right.entries.reverse();
        for entry in &mut right.entries {
            entry.authorizations.reverse();
            entry.trusted_applications.reverse();
        }
        assert_eq!(
            ffi::canonical_digest_for_test(&mut left).unwrap(),
            ffi::canonical_digest_for_test(&mut right).unwrap()
        );
    }

    #[test]
    fn production_surface_uses_direct_access_without_any_or_honor_root() {
        let source = include_str!("ffi.rs");
        assert!(source.contains("SecAccessCreateWithOwnerAndACL"));
        assert!(source.contains("SecACLCreateWithSimpleContents"));
        assert!(source.contains("kSecAttrAccess }, access.cast()"));
        assert!(!source.contains("kSecHonorRoot"));
        // One occurrence maps Any during strict readback and one declares the framework static;
        // construction never places it in an authorization array.
        assert_eq!(source.matches("kSecACLAuthorizationAny").count(), 2);
        assert!(!source.contains("fn SecAccessCreate("));
    }

    #[test]
    fn product_equivalent_constructor_omits_access_and_routes_are_sealed() {
        let source = include_str!("ffi.rs");
        assert!(source.contains("create_product_equivalent_signer_without_access"));
        assert!(source.contains("create_exact_key(&mut owned, keychain, config, None)"));

        let creator = include_str!("bin/creator_route.rs");
        let wrong = include_str!("bin/wrong_identity.rs");
        for binary in [creator, wrong] {
            assert!(!binary.contains("std::env::args"));
            assert!(!binary.contains("std::env::var"));
            assert!(binary.contains("verify_closed_process_surface"));
            assert!(binary.contains("FixedRepetitionV2"));
            assert!(binary.contains("compiled_creator_route_config"));
        }
        let process_surface = include_str!("experiment.rs");
        assert!(process_surface.contains("std::env::args_os().count() != 1"));
        assert!(process_surface.contains("std::env::vars_os()"));
        assert!(process_surface.contains("starts_with(b\"SUBSTRATE_\")"));
        assert!(process_surface.contains("std::env::current_dir()"));
        assert!(creator.contains("CREATOR_EXECUTABLE_PATH"));
        assert!(creator.contains("MARKER_PATH"));
        assert!(wrong.contains("WRONG_IDENTITY_EXECUTABLE_PATH"));
        assert!(wrong.contains("EXPECTED_DELETE_STATUS"));
        assert!(wrong.contains("-25_308"));
        for repetition in FixedRepetitionV2::ALL {
            let config = compiled_creator_route_config(repetition).unwrap();
            assert!(config
                .application_tag()
                .starts_with(repetition.creator_scope().as_bytes()));
        }
    }

    #[test]
    fn explicit_acl_disposable_publisher_is_a_distinct_fixed_identity() {
        const COORDINATOR: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator";
        const PRODUCT_PUBLISHER: &str =
            "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1";
        let identities = [
            CREATOR_EXECUTABLE_PATH,
            DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
            COORDINATOR,
            PRODUCT_PUBLISHER,
            EXPERIMENT_FINALIZER_EXECUTABLE_PATH,
        ];
        for (index, identity) in identities.iter().enumerate() {
            assert!(identities
                .iter()
                .skip(index + 1)
                .all(|other| identity != other));
        }
        let config = compiled_explicit_acl_config().unwrap();
        assert_eq!(
            config.publisher_path(),
            DISPOSABLE_PUBLISHER_EXECUTABLE_PATH
        );
        assert_eq!(
            config.finalizer_path(),
            EXPERIMENT_FINALIZER_EXECUTABLE_PATH
        );
    }

    #[test]
    fn nonmatch_owner_requires_in_memory_create_and_copy_readback_before_key_create() {
        let source = include_str!("ffi.rs");
        let probe = source
            .find("fn probe_nonmatch_owner_access_in_memory_impl")
            .unwrap();
        let key_create = source.find("fn create_exact_key").unwrap();
        let surface = &source[probe..key_create];
        assert!(surface.contains("SecAccessCreateWithOwnerAndACL"));
        assert!(surface.contains("snapshot_access"));
        assert!(surface.contains("snapshot.entries.is_empty()"));
        assert!(!surface.contains("SecKeyCreateRandomKey"));
    }
}
