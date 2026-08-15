//! Closed, disposable-only experiment protocol for the R3 macOS finalizer.
//!
//! This module deliberately contains no Security.framework, Keychain, or launchd calls.  It
//! defines the fixed identities, artifacts, control sequence, and causal state machine shared by
//! the UID-501 harness, the exact coordinator, and the separately installed root publisher.

pub mod controls;
#[cfg(feature = "experiment-harness")]
pub mod documents;
pub mod durable;
pub mod evidence_export;
pub mod freeze_manifest;
pub mod freeze_provenance;
pub mod harness_protocol;
#[cfg(target_os = "macos")]
pub mod peer_probe;
pub mod pre_effect;
#[cfg(target_os = "macos")]
pub mod process;
pub mod publisher_protocol;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

pub const EXPERIMENT_OWNER_V2: &str = "substrate.r3-macos-disposable-finalizer-experiment";
pub const EXPERIMENT_VERSION_V2: u32 = 2;
pub const EXPERIMENT_ID_V2: &str = "01a006ed-1dbc-7d16-b88d-a2b1818ce801";
pub const EXPERIMENT_ROOT_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a006ed-1dbc-7d16-b88d-a2b1818ce801";
pub const GLOBAL_PUBLISHER_EXCHANGE_ROOT_V2: &str = "/Users/spensermcconnell/Library/Application Support/Atomize/R3MacEvidenceFinalizer/experiments/01a006ed-1dbc-7d16-b88d-a2b1818ce801/global-publisher-exchange";
pub const GLOBAL_PUBLISHER_EXCHANGE_UID_V2: u32 = DISPOSABLE_HARNESS_UID_V2;
pub const GLOBAL_PUBLISHER_EXCHANGE_GID_V2: u32 = 0;
pub const GLOBAL_PUBLISHER_EXCHANGE_MODE_V2: u32 = 0o700;
pub const DISPOSABLE_HARNESS_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-disposable-harness";
pub const DISPOSABLE_PREPARED_INPUT_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/disposable-publisher-prepared-input.v2.json";
pub const CANDIDATE_IDENTITY_PACKET_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/candidate-identity-packet.v2.json";
pub const PEER_CONTROL_IDENTITY_PACKET_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/peer-control-identity-packet.v2.json";
pub const ALTERNATE_COORDINATOR_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-evidence-coordinator-alternate-path";
pub const PEER_CODE_PROBE_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-peer-code-probe";
pub const DISPOSABLE_HARNESS_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-disposable-harness.v2";
pub const PEER_CODE_PROBE_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-peer-code-probe.v2";
pub const DISPOSABLE_EXPERIMENT_RUNNER_PATH_V2: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2";
pub const DISPOSABLE_EXPERIMENT_RUNNER_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-disposable-experiment-runner.v2";
pub const DISPOSABLE_EXPERIMENT_RUNNER_ROOT_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2";
pub const SECURITYAGENT_OBSERVER_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-securityagent-observer";
pub const SECURITYAGENT_OBSERVER_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-securityagent-observer.v2";
pub const BENIGN_INJECTION_LIBRARY_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-benign-injection-probe.dylib";
pub const BENIGN_INJECTION_LIBRARY_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-benign-injection-probe.v2";
pub const BENIGN_INJECTION_SIGNAL_V2: &[u8] = b"SUBSTRATE_R3_BENIGN_INJECTION_LOADED_V2\n";
pub const BENIGN_INJECTION_MARKER_FD_V2: i32 = 4;
pub const DISPOSABLE_HARNESS_UID_V2: u32 = 501;
pub const DISPOSABLE_HARNESS_GID_V2: u32 =
    substrate_common::macos_retirement_v2::MAC_R3_COORDINATOR_EFFECTIVE_GID_V2;
pub const DISPOSABLE_HARNESS_ACCOUNT_V2: &str = "spensermcconnell";
pub const DISPOSABLE_PUBLISHER_ROOT_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-publisher.v2";
pub const DISPOSABLE_PUBLISHER_CWD_V2: &str = DISPOSABLE_PUBLISHER_ROOT_V2;
/// `CS_ADHOC | CS_REQUIRE_LV | CS_RUNTIME`, produced only by
/// `codesign --sign - --options runtime,library` for this frozen experiment.
pub const AD_HOC_HARDENED_RUNTIME_FLAGS_V2: u32 = 0x0001_2002;
pub const EMPTY_ENTITLEMENTS_SHA256_V2: &str =
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeSignatureKindV2 {
    AdHoc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CodeSigningPostureV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub executable_identity_sha256: String,
    pub signature_kind: CodeSignatureKindV2,
    pub code_directory_flags: u32,
    pub hardened_runtime: bool,
    pub library_validation: bool,
    pub team_identifier: Option<String>,
    pub entitlements_blob_size: u64,
    pub entitlements_blob_sha256: String,
    pub entitlement_keys: Vec<String>,
    pub verification_observation_sha256: String,
}

impl CodeSigningPostureV2 {
    pub fn validate_for(
        &self,
        identity: &substrate_common::macos_retirement_v2::ExecutableIdentityV2,
    ) -> Result<()> {
        if self.schema_owner != EXPERIMENT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.executable_identity_sha256
                != substrate_common::macos_retirement_v2::document_sha256_v2(identity)?
            || self.signature_kind != CodeSignatureKindV2::AdHoc
            || self.code_directory_flags != AD_HOC_HARDENED_RUNTIME_FLAGS_V2
            || !self.hardened_runtime
            || !self.library_validation
            || self.team_identifier.is_some()
            || self.entitlements_blob_size != 0
            || self.entitlements_blob_sha256 != EMPTY_ENTITLEMENTS_SHA256_V2
            || !self.entitlement_keys.is_empty()
            || !is_sha256(&self.verification_observation_sha256)
        {
            bail!("executable is not exact no-entitlement ad-hoc Hardened Runtime code")
        }
        Ok(())
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub const DISPOSABLE_SCOPES_V2: [&str; 2] = [
    "01a006ed-1dbd-78a2-8d18-1f9c039634da",
    "01a006ed-1dbe-7ea2-a3e9-81225eba65f7",
];
pub const CREATOR_SCOPES_V2: [&str; 2] = [
    "019ffeb5-b24c-72a7-80bc-e629f85b37c3",
    "019ffeb5-b24f-7834-bb4d-44885040002f",
];
pub const CREATOR_EXECUTABLE_PATH_V2: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-creator.v1";
pub const WRONG_IDENTITY_EXECUTABLE_PATH_V2: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1";
pub const CREATOR_EXECUTABLE_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-signer-acl-creator.v1";
pub const WRONG_IDENTITY_EXECUTABLE_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1";
pub const CREATOR_MARKER_ROOT_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-signer-acl-experiment.v2";
pub const CREATOR_MARKER_PATH_V2: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-signer-acl-experiment.v2/creator-route.v2";
pub const NOBODY_OWNER_PROBE_PATH_V2: &str = "/Library/Application Support/Atomize/R3MacEvidenceFinalizer/v2/substrate-r3-macos-nobody-owner-probe";
pub const NOBODY_OWNER_PROBE_SIGNING_IDENTIFIER_V2: &str =
    "com.atomize.substrate.r3-macos-nobody-owner-probe.v2";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RepetitionV2 {
    One,
    Two,
}

impl RepetitionV2 {
    pub const ALL: [Self; 2] = [Self::One, Self::Two];

    pub const fn ordinal(self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
        }
    }

    pub const fn scope_id(self) -> &'static str {
        DISPOSABLE_SCOPES_V2[(self.ordinal() - 1) as usize]
    }

    pub fn directory_name(self) -> String {
        format!("{:02}-{}", self.ordinal(), self.scope_id())
    }

    pub fn validate_binding(self, ordinal: u8, scope_id: &str) -> Result<()> {
        if ordinal != self.ordinal() || scope_id != self.scope_id() {
            bail!("disposable experiment repetition binding changed")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HarnessStageV2 {
    Prepared,
    CoordinatorAttested,
    SurrogatesCreated,
    ReceiptExternallyDurable,
    AcknowledgementExternallyDurable,
    AcknowledgementCasBound,
    FinalizationRequestFrozen,
    FinalizationRequestPublished,
    FinalizerAccepted,
    EffectsComplete,
    HarnessResidualRemoving,
    ParityExternallyDurable,
    TerminalAcknowledgementBound,
    TerminalBindingPublished,
    Complete,
    RestorationComplete,
}

impl HarnessStageV2 {
    pub fn require_successor(self, next: Self) -> Result<()> {
        let current = HARNESS_STAGE_SEQUENCE_V2
            .iter()
            .position(|candidate| *candidate == self)
            .expect("closed harness stage is in its sequence");
        let next = HARNESS_STAGE_SEQUENCE_V2
            .iter()
            .position(|candidate| *candidate == next)
            .expect("closed harness stage is in its sequence");
        if next != current + 1 {
            bail!("disposable harness stage is not the exact causal successor")
        }
        Ok(())
    }
}

pub const HARNESS_STAGE_SEQUENCE_V2: [HarnessStageV2; 16] = [
    HarnessStageV2::Prepared,
    HarnessStageV2::CoordinatorAttested,
    HarnessStageV2::SurrogatesCreated,
    HarnessStageV2::ReceiptExternallyDurable,
    HarnessStageV2::AcknowledgementExternallyDurable,
    HarnessStageV2::AcknowledgementCasBound,
    HarnessStageV2::FinalizationRequestFrozen,
    HarnessStageV2::FinalizationRequestPublished,
    HarnessStageV2::FinalizerAccepted,
    HarnessStageV2::EffectsComplete,
    HarnessStageV2::HarnessResidualRemoving,
    HarnessStageV2::ParityExternallyDurable,
    HarnessStageV2::TerminalAcknowledgementBound,
    HarnessStageV2::TerminalBindingPublished,
    HarnessStageV2::Complete,
    HarnessStageV2::RestorationComplete,
];

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn identity() -> substrate_common::macos_retirement_v2::ExecutableIdentityV2 {
        substrate_common::macos_retirement_v2::ExecutableIdentityV2 {
            source_commit: "1".repeat(40),
            source_tree: "2".repeat(40),
            source_hashes_sha256: digest(1),
            build_inputs_sha256: digest(2),
            executable_sha256: digest(3),
            executable_size: 1,
            intended_path: "/fixed".to_string(),
            physical_identity_sha256: digest(4),
            signing_identifier: "fixed".to_string(),
            designated_requirement: "identifier fixed".to_string(),
            cdhash: "5".repeat(40),
        }
    }

    fn signing_posture(
        identity: &substrate_common::macos_retirement_v2::ExecutableIdentityV2,
    ) -> CodeSigningPostureV2 {
        CodeSigningPostureV2 {
            schema_owner: EXPERIMENT_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            executable_identity_sha256: substrate_common::macos_retirement_v2::document_sha256_v2(
                identity,
            )
            .unwrap(),
            signature_kind: CodeSignatureKindV2::AdHoc,
            code_directory_flags: AD_HOC_HARDENED_RUNTIME_FLAGS_V2,
            hardened_runtime: true,
            library_validation: true,
            team_identifier: None,
            entitlements_blob_size: 0,
            entitlements_blob_sha256: EMPTY_ENTITLEMENTS_SHA256_V2.to_string(),
            entitlement_keys: Vec::new(),
            verification_observation_sha256: digest(6),
        }
    }

    #[test]
    fn exact_repetitions_and_causal_sequence_are_closed() {
        assert_eq!(RepetitionV2::One.ordinal(), 1);
        assert_eq!(RepetitionV2::Two.scope_id(), DISPOSABLE_SCOPES_V2[1]);
        assert!(RepetitionV2::One
            .validate_binding(2, RepetitionV2::One.scope_id())
            .is_err());
        for pair in HARNESS_STAGE_SEQUENCE_V2.windows(2) {
            pair[0].require_successor(pair[1]).unwrap();
        }
        assert!(HarnessStageV2::Prepared
            .require_successor(HarnessStageV2::ReceiptExternallyDurable)
            .is_err());
    }

    #[test]
    fn terminal_disposable_scopes_are_refused_after_rotation() {
        for terminal_scope in [
            "019ffeb5-b252-79ae-8f41-e161419fbbcd",
            "019ffeb5-b255-75a5-870f-49323ebb2c19",
            "01a0033f-9faa-75e4-afb2-f96731adb7de",
            "01a0033f-9fac-79f4-a137-391728ab2f76",
            "01a003da-91f9-712b-b261-fe3be9b5550d",
            "01a003da-91fc-7aad-83e2-75cd7b026fe7",
        ] {
            assert!(RepetitionV2::One
                .validate_binding(RepetitionV2::One.ordinal(), terminal_scope)
                .is_err());
            assert!(RepetitionV2::Two
                .validate_binding(RepetitionV2::Two.ordinal(), terminal_scope)
                .is_err());
        }
    }

    #[test]
    fn signing_posture_rejects_entitlements_and_non_runtime_or_non_adhoc_code() {
        let identity = identity();
        let posture = signing_posture(&identity);
        posture.validate_for(&identity).unwrap();

        let mut entitled = posture.clone();
        entitled
            .entitlement_keys
            .push("com.apple.security.cs.allow-dyld-environment-variables".to_string());
        assert!(entitled.validate_for(&identity).is_err());
        let mut entitled = posture.clone();
        entitled
            .entitlement_keys
            .push("com.apple.security.cs.disable-library-validation".to_string());
        assert!(entitled.validate_for(&identity).is_err());
        let mut not_runtime = posture.clone();
        not_runtime.hardened_runtime = false;
        assert!(not_runtime.validate_for(&identity).is_err());
        let mut no_library_validation = posture.clone();
        no_library_validation.library_validation = false;
        assert!(no_library_validation.validate_for(&identity).is_err());
        let mut wrong_flags = posture.clone();
        wrong_flags.code_directory_flags = 0x2;
        assert!(wrong_flags.validate_for(&identity).is_err());
        let mut team_signed = posture;
        team_signed.team_identifier = Some("NOT-AD-HOC".to_string());
        assert!(team_signed.validate_for(&identity).is_err());
    }

    #[test]
    fn harness_signing_key_code_and_direct_dependencies_are_feature_gated() {
        let manifest = include_str!("../../Cargo.toml");
        let module = include_str!("mod.rs");
        assert!(manifest.contains("required-features = [\"experiment-harness\"]"));
        assert!(manifest.contains("ed25519-dalek = { workspace = true, optional = true }"));
        assert!(manifest.contains("rand_core = { workspace = true, optional = true }"));
        assert!(
            manifest.contains("experiment-harness = [\"dep:ed25519-dalek\", \"dep:rand_core\"]")
        );
        assert!(module.contains("#[cfg(feature = \"experiment-harness\")]\npub mod documents;"));
        assert!(include_str!("documents.rs").contains("SigningKey"));
    }
}
