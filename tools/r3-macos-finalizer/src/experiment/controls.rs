use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, parse_canonical_v2, sha256_hex_v2,
    ExecutableIdentityV2, FinalizationRequestV2, FinalizerResponseStateV2, FinalizerResponseV2,
    PreservingStopClassificationV2, PublisherPreRemovalReceiptV2, TargetSetKindV2,
};

use super::process::{SupplementaryGroupAttestationV2, SupplementaryGroupEvidenceV2};

use super::{
    RepetitionV2, ALTERNATE_COORDINATOR_PATH_V2, BENIGN_INJECTION_LIBRARY_PATH_V2,
    BENIGN_INJECTION_LIBRARY_SIGNING_IDENTIFIER_V2, BENIGN_INJECTION_MARKER_FD_V2,
    CREATOR_EXECUTABLE_PATH_V2, CREATOR_EXECUTABLE_SIGNING_IDENTIFIER_V2, EXPERIMENT_ID_V2,
    EXPERIMENT_VERSION_V2, NOBODY_OWNER_PROBE_PATH_V2, NOBODY_OWNER_PROBE_SIGNING_IDENTIFIER_V2,
    PEER_CODE_PROBE_PATH_V2, PEER_CODE_PROBE_SIGNING_IDENTIFIER_V2, SECURITYAGENT_OBSERVER_PATH_V2,
    SECURITYAGENT_OBSERVER_SIGNING_IDENTIFIER_V2, WRONG_IDENTITY_EXECUTABLE_PATH_V2,
    WRONG_IDENTITY_EXECUTABLE_SIGNING_IDENTIFIER_V2,
};

pub const TRANSPORT_CONTROL_RECEIPT_OWNER_V2: &str =
    "substrate.r3-macos-disposable-transport-control-receipt";
pub const FINALIZER_PROTOCOL_DEADLINE_MS_V2: u64 = 30_000;
pub const FINALIZER_PROTOCOL_DEADLINE_EARLY_TOLERANCE_MS_V2: u64 = 1_000;
pub const FINALIZER_PROTOCOL_DEADLINE_GRACE_MS_V2: u64 = 5_000;
pub const SAFE_REJECTION_REQUEST_DOMAIN_V2: &[u8] =
    b"substrate.r3-macos-finalizer.safe-rejection.request.v2\0";
pub const NO_ACCEPTED_JOURNAL_DOMAIN_V2: &[u8] =
    b"substrate.r3-macos-finalizer.no-accepted-journal.v2\0";
pub const PEER_CONTROL_RECEIPT_OWNER_V2: &str =
    "substrate.r3-macos-disposable-peer-substitution-control";
pub const NOBODY_PRINCIPAL_UID_V2: u32 = 0xffff_fffe;
pub const NOBODY_PRINCIPAL_GID_V2: u32 = 0xffff_fffe;
pub const NOBODY_PRINCIPAL_ACCOUNT_V2: &str = "nobody";
pub const NOBODY_PRINCIPAL_GROUP_V2: &str = "nobody";
pub const NOBODY_PERSISTED_OWNER_UID_V2: u32 = u32::MAX;
pub const NOBODY_PERSISTED_OWNER_GID_V2: u32 = u32::MAX;
pub const NOBODY_PERSISTED_OWNER_TYPE_V2: u32 = 1 | 2;
pub const ERR_SEC_AUTH_FAILED_V2: i32 = -25_293;
pub const ERR_SEC_INTERACTION_NOT_ALLOWED_V2: i32 = -25_308;
pub const SECURITYAGENT_RAW_REPORT_MAX_BYTES_V2: usize = 256 * 1024;
pub const CONTROL_RAW_STREAM_MAX_BYTES_V2: usize = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BoundedRawStreamEvidenceV2 {
    pub raw_base64url: String,
    pub raw_sha256: String,
    pub raw_byte_length: u64,
}

impl BoundedRawStreamEvidenceV2 {
    pub fn validate(&self) -> Result<Vec<u8>> {
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.raw_base64url)
            .context("decode bounded raw control stream")?;
        if bytes.len() > CONTROL_RAW_STREAM_MAX_BYTES_V2
            || self.raw_byte_length
                != u64::try_from(bytes.len()).context("raw control stream length exceeds u64")?
            || self.raw_sha256 != sha256_hex_v2(&bytes)
        {
            bail!("bounded raw control stream changed its bytes, length, or digest")
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecurityAgentRawReportEvidenceV2 {
    pub raw_report_base64url: String,
    pub raw_report_sha256: String,
    pub raw_report_byte_length: u64,
}

impl SecurityAgentRawReportEvidenceV2 {
    pub fn validate(&self) -> Result<()> {
        self.validate_shape(false)
    }

    pub fn validate_terminal_alert(&self) -> Result<()> {
        self.validate_shape(true)
    }

    fn validate_shape(&self, terminal_alert: bool) -> Result<()> {
        let bytes = URL_SAFE_NO_PAD
            .decode(&self.raw_report_base64url)
            .context("decode raw SecurityAgent report evidence")?;
        if bytes.is_empty()
            || bytes.len() > SECURITYAGENT_RAW_REPORT_MAX_BYTES_V2
            || self.raw_report_byte_length
                != u64::try_from(bytes.len()).context("SecurityAgent report length exceeds u64")?
            || self.raw_report_sha256 != sha256_hex_v2(&bytes)
        {
            bail!("raw SecurityAgent report evidence changed its bounded bytes or digest")
        }
        let report: Value = parse_canonical_v2(&bytes)?;
        let object = report
            .as_object()
            .context("raw SecurityAgent report is not one canonical object")?;
        let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        keys.sort_unstable();
        let mut expected = vec![
            "activeTransitionAfterBaseline",
            "baseline",
            "distinctProcesses",
            "distinctWindows",
            "newProcessAfterBaseline",
            "sampleCount",
            "sampleIntervalMilliseconds",
            "schemaOwner",
            "schemaVersion",
            "unexpectedUiObserved",
            "windowAfterBaseline",
        ];
        expected.sort_unstable();
        let sample_count = object.get("sampleCount").and_then(Value::as_u64);
        let unexpected = object.get("unexpectedUiObserved").and_then(Value::as_bool);
        let alert_signal_present = [
            "newProcessAfterBaseline",
            "activeTransitionAfterBaseline",
            "windowAfterBaseline",
        ]
        .into_iter()
        .any(|key| object.get(key).and_then(Value::as_bool) == Some(true))
            || object
                .get("baseline")
                .and_then(Value::as_object)
                .is_some_and(|baseline| {
                    baseline
                        .get("windows")
                        .and_then(Value::as_array)
                        .is_some_and(|windows| !windows.is_empty())
                        || baseline
                            .get("processes")
                            .and_then(Value::as_array)
                            .is_some_and(|processes| {
                                processes.iter().any(|process| {
                                    process
                                        .as_object()
                                        .and_then(|value| value.get("active"))
                                        .and_then(Value::as_bool)
                                        == Some(true)
                                })
                            })
                });
        if keys != expected
            || object.get("schemaOwner").and_then(Value::as_str)
                != Some("substrate.r3-macos-securityagent-observation")
            || object.get("schemaVersion").and_then(Value::as_u64) != Some(1)
            || if terminal_alert {
                !sample_count.is_some_and(|count| (1..=101).contains(&count))
                    || unexpected != Some(true)
                    || !alert_signal_present
            } else {
                sample_count != Some(101) || unexpected != Some(false)
            }
            || object
                .get("sampleIntervalMilliseconds")
                .and_then(Value::as_u64)
                != Some(50)
            || !object.get("baseline").is_some_and(Value::is_object)
            || !object.get("distinctProcesses").is_some_and(Value::is_array)
            || !object.get("distinctWindows").is_some_and(Value::is_array)
            || (!terminal_alert
                && [
                    "newProcessAfterBaseline",
                    "activeTransitionAfterBaseline",
                    "windowAfterBaseline",
                ]
                .into_iter()
                .any(|key| object.get(key).and_then(Value::as_bool) != Some(false)))
        {
            bail!("raw SecurityAgent report is not the exact bounded expected observation")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SecurityAgentArmEvidenceV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub observer_path: String,
    pub reports: Vec<SecurityAgentRawReportEvidenceV2>,
    pub report_set_sha256: String,
    pub overlap_rearm_count: u32,
    pub gap_free_rearm_coverage: bool,
    pub unexpected_ui_observed: bool,
}

impl SecurityAgentArmEvidenceV2 {
    pub fn validate(&self) -> Result<()> {
        if self.schema_owner != "substrate.r3-macos-securityagent-arm-observation"
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.observer_path != SECURITYAGENT_OBSERVER_PATH_V2
            || self.reports.is_empty()
            || self.reports.len() > 1024
            || self.report_set_sha256 != document_sha256_v2(&self.reports)?
            || usize::try_from(self.overlap_rearm_count)
                .context("SecurityAgent rearm count exceeds usize")?
                != self.reports.len() - 1
            || !self.gap_free_rearm_coverage
            || self.unexpected_ui_observed
        {
            bail!("SecurityAgent arm evidence omitted raw reports or gap-free overlap coverage")
        }
        for report in &self.reports {
            report.validate()?;
        }
        Ok(())
    }

    pub fn validate_terminal_alert(&self) -> Result<()> {
        if self.schema_owner != "substrate.r3-macos-securityagent-arm-observation"
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.observer_path != SECURITYAGENT_OBSERVER_PATH_V2
            || self.reports.is_empty()
            || self.reports.len() > 1024
            || self.report_set_sha256 != document_sha256_v2(&self.reports)?
            || usize::try_from(self.overlap_rearm_count)
                .context("SecurityAgent rearm count exceeds usize")?
                != self.reports.len() - 1
            || !self.gap_free_rearm_coverage
            || !self.unexpected_ui_observed
        {
            bail!("SecurityAgent alert evidence omitted its raw terminal report")
        }
        let (last, preceding) = self
            .reports
            .split_last()
            .context("SecurityAgent terminal alert evidence is empty")?;
        for report in preceding {
            report.validate()?;
        }
        last.validate_terminal_alert()
    }
}

pub fn single_securityagent_terminal_alert_evidence_v2(
    report: SecurityAgentRawReportEvidenceV2,
) -> Result<SecurityAgentArmEvidenceV2> {
    report.validate_terminal_alert()?;
    let reports = vec![report];
    let evidence = SecurityAgentArmEvidenceV2 {
        schema_owner: "substrate.r3-macos-securityagent-arm-observation".to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        observer_path: SECURITYAGENT_OBSERVER_PATH_V2.to_string(),
        report_set_sha256: document_sha256_v2(&reports)?,
        reports,
        overlap_rearm_count: 0,
        gap_free_rearm_coverage: true,
        unexpected_ui_observed: true,
    };
    evidence.validate_terminal_alert()?;
    Ok(evidence)
}

pub fn single_securityagent_arm_evidence_v2(
    report: SecurityAgentRawReportEvidenceV2,
) -> Result<SecurityAgentArmEvidenceV2> {
    report.validate()?;
    let reports = vec![report];
    let evidence = SecurityAgentArmEvidenceV2 {
        schema_owner: "substrate.r3-macos-securityagent-arm-observation".to_string(),
        schema_version: EXPERIMENT_VERSION_V2,
        observer_path: SECURITYAGENT_OBSERVER_PATH_V2.to_string(),
        report_set_sha256: document_sha256_v2(&reports)?,
        reports,
        overlap_rearm_count: 0,
        gap_free_rearm_coverage: true,
        unexpected_ui_observed: false,
    };
    evidence.validate()?;
    Ok(evidence)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum NobodyOwnerAuthorityOperationV2 {
    SetOwnerAndAclReplacement,
    SetAclReplacement,
    DeleteExactTarget,
    SignExactTarget,
}

pub const NOBODY_OWNER_AUTHORITY_SEQUENCE_V2: [NobodyOwnerAuthorityOperationV2; 4] = [
    NobodyOwnerAuthorityOperationV2::SetOwnerAndAclReplacement,
    NobodyOwnerAuthorityOperationV2::SetAclReplacement,
    NobodyOwnerAuthorityOperationV2::DeleteExactTarget,
    NobodyOwnerAuthorityOperationV2::SignExactTarget,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PeerSubstitutionControlV2 {
    AlternateCaller,
    AlternateExecutableCodeIdentity,
    AlternatePhysicalPath,
}

pub const PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2: [PeerSubstitutionControlV2; 3] = [
    PeerSubstitutionControlV2::AlternateCaller,
    PeerSubstitutionControlV2::AlternateExecutableCodeIdentity,
    PeerSubstitutionControlV2::AlternatePhysicalPath,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PeerProbeTerminationV2 {
    ConnectionClosedBeforeResponse,
    CanonicalSafePreAcceptanceStop,
}

pub const PEER_EXPECTED_TERMINATION_SEQUENCE_V2: [PeerProbeTerminationV2; 3] = [
    PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
    PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
    PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeerControlIdentityPacketV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub harness_identity: ExecutableIdentityV2,
    pub harness_signing_posture: super::CodeSigningPostureV2,
    pub runner_identity: ExecutableIdentityV2,
    pub runner_signing_posture: super::CodeSigningPostureV2,
    pub alternate_caller_identity: ExecutableIdentityV2,
    pub alternate_caller_signing_posture: super::CodeSigningPostureV2,
    pub alternate_code_identity: ExecutableIdentityV2,
    pub alternate_code_signing_posture: super::CodeSigningPostureV2,
    pub alternate_path_identity: ExecutableIdentityV2,
    pub alternate_path_signing_posture: super::CodeSigningPostureV2,
    pub nobody_owner_probe_identity: ExecutableIdentityV2,
    pub nobody_owner_probe_signing_posture: super::CodeSigningPostureV2,
    pub creator_identity: ExecutableIdentityV2,
    pub creator_signing_posture: super::CodeSigningPostureV2,
    pub wrong_identity: ExecutableIdentityV2,
    pub wrong_signing_posture: super::CodeSigningPostureV2,
    pub securityagent_observer_identity: ExecutableIdentityV2,
    pub securityagent_observer_signing_posture: super::CodeSigningPostureV2,
    pub benign_injection_library_identity: ExecutableIdentityV2,
    pub benign_injection_library_signing_posture: super::CodeSigningPostureV2,
    pub expected_termination: Vec<PeerProbeTerminationV2>,
}

impl PeerControlIdentityPacketV2 {
    pub fn validate(
        &self,
        candidate: &super::publisher_protocol::CandidateIdentityPacketV2,
    ) -> Result<()> {
        candidate.validate()?;
        if self.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.expected_termination.len() != PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2.len()
            || self.expected_termination != PEER_EXPECTED_TERMINATION_SEQUENCE_V2
        {
            bail!("peer-control identity packet owner, version, experiment, or arm count changed")
        }
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            &self.alternate_caller_identity,
            substrate_common::macos_retirement_v2::MAC_R3_COORDINATOR_PATH_V2,
            substrate_common::macos_retirement_v2::MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
        )?;
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            &self.harness_identity,
            super::DISPOSABLE_HARNESS_PATH_V2,
            super::DISPOSABLE_HARNESS_SIGNING_IDENTIFIER_V2,
        )?;
        self.harness_signing_posture
            .validate_for(&self.harness_identity)?;
        substrate_common::macos_retirement_v2::validate_executable_identity_v2(
            &self.runner_identity,
            super::DISPOSABLE_EXPERIMENT_RUNNER_PATH_V2,
            super::DISPOSABLE_EXPERIMENT_RUNNER_SIGNING_IDENTIFIER_V2,
        )?;
        self.runner_signing_posture
            .validate_for(&self.runner_identity)?;
        self.alternate_caller_signing_posture
            .validate_for(&self.alternate_caller_identity)?;
        validate_probe_executable(
            &self.alternate_code_identity,
            PEER_CODE_PROBE_PATH_V2,
            PEER_CODE_PROBE_SIGNING_IDENTIFIER_V2,
        )?;
        self.alternate_code_signing_posture
            .validate_for(&self.alternate_code_identity)?;
        validate_probe_executable(
            &self.alternate_path_identity,
            ALTERNATE_COORDINATOR_PATH_V2,
            &candidate.coordinator_identity.signing_identifier,
        )?;
        self.alternate_path_signing_posture
            .validate_for(&self.alternate_path_identity)?;
        validate_probe_executable(
            &self.nobody_owner_probe_identity,
            NOBODY_OWNER_PROBE_PATH_V2,
            NOBODY_OWNER_PROBE_SIGNING_IDENTIFIER_V2,
        )?;
        self.nobody_owner_probe_signing_posture
            .validate_for(&self.nobody_owner_probe_identity)?;
        validate_probe_executable(
            &self.creator_identity,
            CREATOR_EXECUTABLE_PATH_V2,
            CREATOR_EXECUTABLE_SIGNING_IDENTIFIER_V2,
        )?;
        self.creator_signing_posture
            .validate_for(&self.creator_identity)?;
        validate_probe_executable(
            &self.wrong_identity,
            WRONG_IDENTITY_EXECUTABLE_PATH_V2,
            WRONG_IDENTITY_EXECUTABLE_SIGNING_IDENTIFIER_V2,
        )?;
        self.wrong_signing_posture
            .validate_for(&self.wrong_identity)?;
        validate_probe_executable(
            &self.securityagent_observer_identity,
            SECURITYAGENT_OBSERVER_PATH_V2,
            SECURITYAGENT_OBSERVER_SIGNING_IDENTIFIER_V2,
        )?;
        self.securityagent_observer_signing_posture
            .validate_for(&self.securityagent_observer_identity)?;
        validate_probe_executable(
            &self.benign_injection_library_identity,
            BENIGN_INJECTION_LIBRARY_PATH_V2,
            BENIGN_INJECTION_LIBRARY_SIGNING_IDENTIFIER_V2,
        )?;
        self.benign_injection_library_signing_posture
            .validate_for(&self.benign_injection_library_identity)?;
        if self.alternate_caller_identity != candidate.coordinator_identity
            || self.alternate_caller_signing_posture != candidate.coordinator_signing_posture
            || self.alternate_path_identity.executable_sha256
                != candidate.coordinator_identity.executable_sha256
            || self.alternate_path_identity.executable_size
                != candidate.coordinator_identity.executable_size
            || self.alternate_path_identity.designated_requirement
                != candidate.coordinator_identity.designated_requirement
            || self.alternate_path_identity.cdhash != candidate.coordinator_identity.cdhash
            || self.alternate_path_identity.physical_identity_sha256
                == candidate.coordinator_identity.physical_identity_sha256
            || self.alternate_code_identity.executable_sha256
                == candidate.coordinator_identity.executable_sha256
            || self.nobody_owner_probe_identity.executable_sha256
                == candidate.coordinator_identity.executable_sha256
            || self.nobody_owner_probe_identity.executable_sha256
                == self.alternate_code_identity.executable_sha256
            || self.benign_injection_library_identity.executable_sha256
                == self.alternate_path_identity.executable_sha256
        {
            bail!("peer-control identities do not separate caller, code, and physical path")
        }
        Ok(())
    }

    pub fn identity_for(&self, control: PeerSubstitutionControlV2) -> &ExecutableIdentityV2 {
        match control {
            PeerSubstitutionControlV2::AlternateCaller => &self.alternate_caller_identity,
            PeerSubstitutionControlV2::AlternateExecutableCodeIdentity => {
                &self.alternate_code_identity
            }
            PeerSubstitutionControlV2::AlternatePhysicalPath => &self.alternate_path_identity,
        }
    }

    pub fn expected_termination_for(
        &self,
        control: PeerSubstitutionControlV2,
    ) -> PeerProbeTerminationV2 {
        let index = PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2
            .iter()
            .position(|candidate| *candidate == control)
            .expect("closed peer control is in its sequence");
        self.expected_termination[index]
    }
}

pub const NOBODY_OWNER_CONTROL_RECEIPT_OWNER_V2: &str =
    "substrate.r3-macos-disposable-nobody-owner-authority-control";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NobodyAuthorizationDenialV2 {
    ErrSecAuthFailed,
    ErrSecInteractionNotAllowed,
}

impl NobodyAuthorizationDenialV2 {
    pub const fn raw_os_status(self) -> i32 {
        match self {
            Self::ErrSecAuthFailed => ERR_SEC_AUTH_FAILED_V2,
            Self::ErrSecInteractionNotAllowed => ERR_SEC_INTERACTION_NOT_ALLOWED_V2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NobodyOwnerProbeProcessAttestationV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub pid: i32,
    pub process_start_identity_sha256: String,
    pub executable_identity_sha256: String,
    pub effective_uid: u32,
    pub effective_gid: u32,
    pub supplementary_groups: SupplementaryGroupAttestationV2,
    pub canonical_account: String,
    pub canonical_group: String,
    pub argv_count: u8,
    pub environment_variable_count: u32,
    pub stdin_is_dev_null: bool,
    pub cwd: String,
}

impl NobodyOwnerProbeProcessAttestationV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        identities: &PeerControlIdentityPacketV2,
    ) -> Result<()> {
        if self.schema_owner != NOBODY_OWNER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("nobody owner probe process attestation identity changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.pid <= 0
            || self.executable_identity_sha256
                != document_sha256_v2(&identities.nobody_owner_probe_identity)?
            || self.effective_uid != NOBODY_PRINCIPAL_UID_V2
            || self.effective_gid != NOBODY_PRINCIPAL_GID_V2
            || self.supplementary_groups.validate_empty().is_err()
            || self.canonical_account != NOBODY_PRINCIPAL_ACCOUNT_V2
            || self.canonical_group != NOBODY_PRINCIPAL_GROUP_V2
            || self.argv_count != 0
            || self.environment_variable_count != 0
            || !self.stdin_is_dev_null
            || self.cwd != "/"
        {
            bail!("nobody owner probe was not the fixed sealed nobody process")
        }
        require_digest(
            &self.process_start_identity_sha256,
            "nobody owner probe process-start identity",
        )?;
        require_digest(
            &self.executable_identity_sha256,
            "nobody owner probe executable identity",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NobodyOwnerAuthorityAttemptV2 {
    pub sequence_ordinal: u8,
    pub operation: NobodyOwnerAuthorityOperationV2,
    pub probe_process: NobodyOwnerProbeProcessAttestationV2,
    pub probe_process_attestation_sha256: String,
    pub probe_self_supplementary_groups: SupplementaryGroupAttestationV2,
    pub raw_os_status: i32,
    pub classification: NobodyAuthorizationDenialV2,
    pub root_before_observation_sha256: String,
    pub root_after_observation_sha256: String,
    pub securityagent_report: SecurityAgentRawReportEvidenceV2,
    pub securityagent_observation_sha256: String,
}

impl NobodyOwnerAuthorityAttemptV2 {
    fn validate(
        &self,
        repetition: RepetitionV2,
        identities: &PeerControlIdentityPacketV2,
        index: usize,
    ) -> Result<()> {
        let expected_operation = NOBODY_OWNER_AUTHORITY_SEQUENCE_V2
            .get(index)
            .copied()
            .context("nobody owner operation is outside the closed sequence")?;
        if self.sequence_ordinal != u8::try_from(index + 1).expect("four controls fit u8")
            || self.operation != expected_operation
            || self.probe_process_attestation_sha256 != document_sha256_v2(&self.probe_process)?
            || self.raw_os_status != self.classification.raw_os_status()
            || self.root_before_observation_sha256 != self.root_after_observation_sha256
        {
            bail!("nobody owner operation was not an exact no-mutation authorization denial")
        }
        self.probe_process.validate(repetition, identities)?;
        self.probe_self_supplementary_groups.validate_empty()?;
        if self.probe_self_supplementary_groups.evidence
            != SupplementaryGroupEvidenceV2::CurrentProcessGetgroups
        {
            bail!("nobody owner probe did not self-measure its supplementary groups")
        }
        self.securityagent_report.validate()?;
        if self.securityagent_observation_sha256 != self.securityagent_report.raw_report_sha256 {
            bail!("nobody owner operation did not retain its raw SecurityAgent report")
        }
        require_digest(
            &self.probe_process_attestation_sha256,
            "nobody owner operation process",
        )?;
        require_digest(
            &self.root_before_observation_sha256,
            "nobody owner operation root-before observation",
        )?;
        require_digest(
            &self.root_after_observation_sha256,
            "nobody owner operation root-after observation",
        )?;
        require_digest(
            &self.securityagent_observation_sha256,
            "nobody owner operation SecurityAgent observation",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NobodyOwnerAuthorityControlReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub request_digest: String,
    pub persisted_owner_uid: u32,
    pub persisted_owner_gid: u32,
    pub persisted_owner_type: u32,
    pub target_identity_sha256: String,
    pub wrong_identity_sha256: String,
    pub signer_access_control_sha256: String,
    pub before_observation_sha256: String,
    pub attempts: Vec<NobodyOwnerAuthorityAttemptV2>,
    pub attempt_set_sha256: String,
    pub after_observation_sha256: String,
    pub accepted_journal_absent_before: bool,
    pub accepted_journal_absent_after: bool,
    pub securityagent_observation_set_sha256: String,
    pub no_mutation_binding_sha256: String,
}

impl NobodyOwnerAuthorityControlReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        request: &FinalizationRequestV2,
        identities: &PeerControlIdentityPacketV2,
    ) -> Result<()> {
        if self.schema_owner != NOBODY_OWNER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("nobody owner-control receipt identity changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.request_digest != request.request_digest
            || self.persisted_owner_uid != NOBODY_PERSISTED_OWNER_UID_V2
            || self.persisted_owner_gid != NOBODY_PERSISTED_OWNER_GID_V2
            || self.persisted_owner_type != NOBODY_PERSISTED_OWNER_TYPE_V2
            || self.attempts.len() != NOBODY_OWNER_AUTHORITY_SEQUENCE_V2.len()
            || self.before_observation_sha256 != self.after_observation_sha256
            || !self.accepted_journal_absent_before
            || !self.accepted_journal_absent_after
        {
            bail!("nobody owner-control receipt changed its exact principal, owner, or no-mutation state")
        }
        for (index, attempt) in self.attempts.iter().enumerate() {
            attempt.validate(repetition, identities, index)?;
            if attempt.root_before_observation_sha256 != self.before_observation_sha256
                || attempt.root_after_observation_sha256 != self.after_observation_sha256
            {
                bail!("nobody owner operation did not preserve the exact aggregate target state")
            }
        }
        let ui_digests = self
            .attempts
            .iter()
            .map(|attempt| &attempt.securityagent_observation_sha256)
            .collect::<Vec<_>>();
        if self.attempt_set_sha256 != document_sha256_v2(&self.attempts)?
            || self.securityagent_observation_set_sha256 != document_sha256_v2(&ui_digests)?
            || self.no_mutation_binding_sha256 != nobody_owner_no_mutation_binding_sha256_v2(self)?
        {
            bail!("nobody owner-control receipt does not exact-bind its attempts and observations")
        }
        for (digest, label) in [
            (&self.request_digest, "nobody owner request"),
            (&self.target_identity_sha256, "nobody owner target identity"),
            (&self.wrong_identity_sha256, "nobody owner wrong identity"),
            (
                &self.signer_access_control_sha256,
                "nobody owner signer access",
            ),
            (&self.before_observation_sha256, "nobody owner before state"),
            (&self.attempt_set_sha256, "nobody owner attempt set"),
            (&self.after_observation_sha256, "nobody owner after state"),
            (
                &self.securityagent_observation_set_sha256,
                "nobody owner SecurityAgent set",
            ),
            (
                &self.no_mutation_binding_sha256,
                "nobody owner no-mutation binding",
            ),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }

    pub fn validate_against_creation(
        &self,
        repetition: RepetitionV2,
        creation: &super::publisher_protocol::SurrogateCreationReceiptV2,
    ) -> Result<()> {
        creation.validate(repetition)?;
        if self.target_identity_sha256 != creation.target.identity_sha256
            || self.wrong_identity_sha256 != creation.wrong.identity_sha256
            || self.signer_access_control_sha256 != creation.target.access_control_sha256
        {
            bail!(
                "nobody owner-control receipt did not bind the exact created target and wrong key"
            )
        }
        Ok(())
    }
}

pub fn nobody_owner_no_mutation_binding_sha256_v2(
    receipt: &NobodyOwnerAuthorityControlReceiptV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Binding<'a> {
        domain: &'static str,
        repetition: u8,
        scope_id: &'a str,
        request_digest: &'a str,
        target_identity_sha256: &'a str,
        wrong_identity_sha256: &'a str,
        signer_access_control_sha256: &'a str,
        before_observation_sha256: &'a str,
        attempt_set_sha256: &'a str,
        after_observation_sha256: &'a str,
        accepted_journal_absent_before: bool,
        accepted_journal_absent_after: bool,
    }
    document_sha256_v2(&Binding {
        domain: "substrate.r3-macos-disposable-nobody-owner-no-mutation.v2",
        repetition: receipt.repetition,
        scope_id: &receipt.scope_id,
        request_digest: &receipt.request_digest,
        target_identity_sha256: &receipt.target_identity_sha256,
        wrong_identity_sha256: &receipt.wrong_identity_sha256,
        signer_access_control_sha256: &receipt.signer_access_control_sha256,
        before_observation_sha256: &receipt.before_observation_sha256,
        attempt_set_sha256: &receipt.attempt_set_sha256,
        after_observation_sha256: &receipt.after_observation_sha256,
        accepted_journal_absent_before: receipt.accepted_journal_absent_before,
        accepted_journal_absent_after: receipt.accepted_journal_absent_after,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeerControlsReadyMarkerV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub request_digest: String,
    pub request_control_receipt_sha256: Vec<String>,
    pub before_observation_sha256: String,
}

impl PeerControlsReadyMarkerV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        request: &FinalizationRequestV2,
        controls: &[TransportControlReceiptV2],
    ) -> Result<()> {
        if self.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("peer-control ready marker owner, version, or experiment changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.request_digest != request.request_digest
            || controls.len() != TRANSPORT_CONTROL_SEQUENCE_V2.len()
            || self.request_control_receipt_sha256.len() != controls.len()
            || !controls
                .iter()
                .zip(&self.request_control_receipt_sha256)
                .all(|(control, digest)| {
                    control.validate(repetition).is_ok()
                        && document_sha256_v2(control).is_ok_and(|actual| actual == *digest)
                })
        {
            bail!("peer-control ready marker does not bind all request/framing controls")
        }
        require_digest(
            &self.before_observation_sha256,
            "peer-control before observation",
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeerProbeExchangeResultV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub request_sha256: String,
    pub request_eof_sent: bool,
    pub termination: PeerProbeTerminationV2,
    pub response: Option<FinalizerResponseV2>,
    pub response_eof_observed: bool,
}

impl PeerProbeExchangeResultV2 {
    pub fn validate(
        &self,
        request: &FinalizationRequestV2,
        expected: PeerProbeTerminationV2,
    ) -> Result<()> {
        if self.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || self.request_sha256 != document_sha256_v2(request)?
            || !self.request_eof_sent
            || self.termination != expected
            || !self.response_eof_observed
        {
            bail!("peer probe did not exact-bind its one request, EOF, and frozen outcome")
        }
        match (self.termination, &self.response) {
            (PeerProbeTerminationV2::ConnectionClosedBeforeResponse, None) => Ok(()),
            (PeerProbeTerminationV2::CanonicalSafePreAcceptanceStop, Some(response)) => {
                substrate_common::macos_retirement_v2::validate_finalizer_response_v2(response)?;
                if response.state != FinalizerResponseStateV2::SafePreAcceptanceStop
                    || response.request_digest
                        != safe_rejection_request_digest_v2(&canonical_bytes_v2(request)?)
                    || response.journal_head_sha256 != sha256_hex_v2(NO_ACCEPTED_JOURNAL_DOMAIN_V2)
                {
                    bail!("peer probe canonical response is not the exact safe pre-acceptance stop")
                }
                Ok(())
            }
            _ => bail!("peer probe termination and response presence disagree"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeerSubstitutionControlReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub sequence_ordinal: u8,
    pub control: PeerSubstitutionControlV2,
    pub executable_identity_sha256: String,
    pub process_attestation_sha256: String,
    pub exchange: PeerProbeExchangeResultV2,
    pub securityagent_evidence: SecurityAgentArmEvidenceV2,
    pub securityagent_observation_sha256: String,
    pub before_observation_sha256: String,
    pub after_observation_sha256: String,
    pub accepted_journal_absent_before: bool,
    pub accepted_journal_absent_after: bool,
    pub no_mutation_binding_sha256: String,
}

impl PeerSubstitutionControlReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        request: &FinalizationRequestV2,
        identities: &PeerControlIdentityPacketV2,
    ) -> Result<()> {
        if self.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("peer-control receipt owner, version, or experiment changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        let expected_control = PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2
            .get(usize::from(self.sequence_ordinal.saturating_sub(1)))
            .copied()
            .context("peer-control receipt ordinal is outside the closed sequence")?;
        if self.sequence_ordinal == 0
            || self.control != expected_control
            || self.executable_identity_sha256
                != document_sha256_v2(identities.identity_for(self.control))?
        {
            bail!("peer-control receipt does not bind its exact ordered executable identity")
        }
        require_digest(&self.process_attestation_sha256, "peer process attestation")?;
        self.securityagent_evidence.validate()?;
        if self.securityagent_observation_sha256
            != document_sha256_v2(&self.securityagent_evidence)?
        {
            bail!("peer-control receipt did not retain its raw SecurityAgent arm evidence")
        }
        require_digest(
            &self.securityagent_observation_sha256,
            "peer SecurityAgent observation",
        )?;
        require_digest(&self.before_observation_sha256, "peer before observation")?;
        require_digest(&self.after_observation_sha256, "peer after observation")?;
        require_digest(&self.no_mutation_binding_sha256, "peer no-mutation binding")?;
        if self.before_observation_sha256 != self.after_observation_sha256
            || !self.accepted_journal_absent_before
            || !self.accepted_journal_absent_after
            || self.no_mutation_binding_sha256
                != peer_substitution_no_mutation_binding_sha256_v2(self)?
        {
            bail!("peer-control receipt does not prove exact no-mutation state")
        }
        self.exchange
            .validate(request, identities.expected_termination_for(self.control))
    }
}

pub fn peer_substitution_no_mutation_binding_sha256_v2(
    receipt: &PeerSubstitutionControlReceiptV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Binding<'a> {
        domain: &'static str,
        repetition: u8,
        scope_id: &'a str,
        sequence_ordinal: u8,
        control: PeerSubstitutionControlV2,
        executable_identity_sha256: &'a str,
        process_attestation_sha256: &'a str,
        canonical_exchange_sha256: String,
        before_observation_sha256: &'a str,
        after_observation_sha256: &'a str,
        accepted_journal_absent_before: bool,
        accepted_journal_absent_after: bool,
        securityagent_arm_evidence_sha256: String,
    }
    document_sha256_v2(&Binding {
        domain: "substrate.r3-macos-disposable-peer-substitution-no-mutation.v2",
        repetition: receipt.repetition,
        scope_id: &receipt.scope_id,
        sequence_ordinal: receipt.sequence_ordinal,
        control: receipt.control,
        executable_identity_sha256: &receipt.executable_identity_sha256,
        process_attestation_sha256: &receipt.process_attestation_sha256,
        canonical_exchange_sha256: document_sha256_v2(&receipt.exchange)?,
        before_observation_sha256: &receipt.before_observation_sha256,
        after_observation_sha256: &receipt.after_observation_sha256,
        accepted_journal_absent_before: receipt.accepted_journal_absent_before,
        accepted_journal_absent_after: receipt.accepted_journal_absent_after,
        securityagent_arm_evidence_sha256: document_sha256_v2(&receipt.securityagent_evidence)?,
    })
}

pub const BENIGN_INJECTION_DYLD_ENVIRONMENT_KEY_V2: &str = "DYLD_INSERT_LIBRARIES";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenignInjectionTerminationV2 {
    EnvironmentNeutralized,
    LoaderRejectedBeforeMain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DynamicLibraryInjectionControlReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub request_digest: String,
    pub target_executable_identity_sha256: String,
    pub injection_library_identity_sha256: String,
    pub process_attestation_sha256: String,
    pub dyld_environment_key: String,
    pub dyld_environment_value: String,
    pub marker_fd: i32,
    pub termination: BenignInjectionTerminationV2,
    pub child_exit_code: Option<i32>,
    pub child_signal: Option<i32>,
    pub process_surface_environment_empty: Option<bool>,
    pub exchange: Option<PeerProbeExchangeResultV2>,
    pub loader_diagnostic_sha256: Option<String>,
    pub child_stdout: BoundedRawStreamEvidenceV2,
    pub child_stderr: BoundedRawStreamEvidenceV2,
    pub marker_before: BoundedRawStreamEvidenceV2,
    pub marker_after: BoundedRawStreamEvidenceV2,
    pub injection_signal_sha256: String,
    pub injection_signal_observed: bool,
    pub before_observation_sha256: String,
    pub after_observation_sha256: String,
    pub accepted_journal_absent: bool,
    pub securityagent_report: SecurityAgentRawReportEvidenceV2,
    pub no_mutation_binding_sha256: String,
}

impl DynamicLibraryInjectionControlReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        request: &FinalizationRequestV2,
        identities: &PeerControlIdentityPacketV2,
    ) -> Result<()> {
        if self.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("dynamic-library injection receipt identity changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        self.securityagent_report.validate()?;
        let child_stdout = self.child_stdout.validate()?;
        let child_stderr = self.child_stderr.validate()?;
        let marker_before = self.marker_before.validate()?;
        let marker_after = self.marker_after.validate()?;
        if self.request_digest != request.request_digest
            || self.target_executable_identity_sha256
                != document_sha256_v2(&identities.alternate_path_identity)?
            || self.injection_library_identity_sha256
                != document_sha256_v2(&identities.benign_injection_library_identity)?
            || self.dyld_environment_key != BENIGN_INJECTION_DYLD_ENVIRONMENT_KEY_V2
            || self.dyld_environment_value != BENIGN_INJECTION_LIBRARY_PATH_V2
            || self.marker_fd != BENIGN_INJECTION_MARKER_FD_V2
            || self.injection_signal_observed
            || !marker_before.is_empty()
            || !marker_after.is_empty()
            || self.marker_before.raw_sha256 != self.marker_after.raw_sha256
            || self.injection_signal_sha256 != self.marker_after.raw_sha256
            || self.before_observation_sha256 != self.after_observation_sha256
            || !self.accepted_journal_absent
            || self.no_mutation_binding_sha256
                != dynamic_library_no_mutation_binding_sha256_v2(self)?
        {
            bail!(
                "benign dynamic-library injection was not rejected or neutralized without mutation"
            )
        }
        match self.termination {
            BenignInjectionTerminationV2::EnvironmentNeutralized => {
                if self.child_exit_code != Some(0)
                    || self.child_signal.is_some()
                    || self.loader_diagnostic_sha256.is_some()
                    || self.process_surface_environment_empty != Some(true)
                    || !child_stdout.is_empty()
                    || !child_stderr.is_empty()
                {
                    bail!("neutralized injection target did not exit through its exact safe route")
                }
                self.exchange
                    .as_ref()
                    .context("neutralized injection target lacks its closed exchange")?
                    .validate(
                        request,
                        PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
                    )?;
            }
            BenignInjectionTerminationV2::LoaderRejectedBeforeMain => {
                let rejected = self.child_exit_code.is_some_and(|status| status != 0)
                    ^ self.child_signal.is_some_and(|signal| signal > 0);
                if !rejected
                    || self.exchange.is_some()
                    || self.process_surface_environment_empty.is_some()
                    || !child_stdout.is_empty()
                {
                    bail!(
                        "loader-rejected injection did not have one exact non-success termination"
                    )
                }
                let diagnostic = self
                    .loader_diagnostic_sha256
                    .as_deref()
                    .context("loader rejection lacks its bounded diagnostic digest")?;
                require_digest(diagnostic, "loader rejection diagnostic")?;
                if diagnostic != self.child_stderr.raw_sha256 {
                    bail!("loader rejection diagnostic does not bind the retained raw stderr")
                }
            }
        }
        for (digest, label) in [
            (&self.request_digest, "injection request"),
            (
                &self.target_executable_identity_sha256,
                "injection target identity",
            ),
            (
                &self.injection_library_identity_sha256,
                "injection library identity",
            ),
            (&self.process_attestation_sha256, "injection process"),
            (&self.injection_signal_sha256, "injection signal"),
            (&self.before_observation_sha256, "injection before state"),
            (&self.after_observation_sha256, "injection after state"),
            (
                &self.no_mutation_binding_sha256,
                "injection no-mutation binding",
            ),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }
}

pub fn dynamic_library_no_mutation_binding_sha256_v2(
    receipt: &DynamicLibraryInjectionControlReceiptV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Binding<'a> {
        domain: &'static str,
        repetition: u8,
        scope_id: &'a str,
        request_digest: &'a str,
        target_executable_identity_sha256: &'a str,
        injection_library_identity_sha256: &'a str,
        process_attestation_sha256: &'a str,
        termination: BenignInjectionTerminationV2,
        child_exit_code: Option<i32>,
        child_signal: Option<i32>,
        process_surface_environment_empty: Option<bool>,
        exchange_sha256: Option<String>,
        loader_diagnostic_sha256: Option<&'a str>,
        child_stdout_sha256: &'a str,
        child_stderr_sha256: &'a str,
        marker_before_sha256: &'a str,
        marker_after_sha256: &'a str,
        injection_signal_sha256: &'a str,
        before_observation_sha256: &'a str,
        after_observation_sha256: &'a str,
        accepted_journal_absent: bool,
        securityagent_report_sha256: &'a str,
    }
    document_sha256_v2(&Binding {
        domain: "substrate.r3-macos-disposable-benign-injection-no-mutation.v2",
        repetition: receipt.repetition,
        scope_id: &receipt.scope_id,
        request_digest: &receipt.request_digest,
        target_executable_identity_sha256: &receipt.target_executable_identity_sha256,
        injection_library_identity_sha256: &receipt.injection_library_identity_sha256,
        process_attestation_sha256: &receipt.process_attestation_sha256,
        termination: receipt.termination,
        child_exit_code: receipt.child_exit_code,
        child_signal: receipt.child_signal,
        process_surface_environment_empty: receipt.process_surface_environment_empty,
        exchange_sha256: receipt
            .exchange
            .as_ref()
            .map(document_sha256_v2)
            .transpose()?,
        loader_diagnostic_sha256: receipt.loader_diagnostic_sha256.as_deref(),
        child_stdout_sha256: &receipt.child_stdout.raw_sha256,
        child_stderr_sha256: &receipt.child_stderr.raw_sha256,
        marker_before_sha256: &receipt.marker_before.raw_sha256,
        marker_after_sha256: &receipt.marker_after.raw_sha256,
        injection_signal_sha256: &receipt.injection_signal_sha256,
        before_observation_sha256: &receipt.before_observation_sha256,
        after_observation_sha256: &receipt.after_observation_sha256,
        accepted_journal_absent: receipt.accepted_journal_absent,
        securityagent_report_sha256: &receipt.securityagent_report.raw_report_sha256,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeerControlSetReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub request_digest: String,
    pub identity_packet_sha256: String,
    pub runner_identity_sha256: String,
    pub runner_process_attestation_sha256: String,
    pub receipts: Vec<PeerSubstitutionControlReceiptV2>,
    pub dynamic_library_injection: DynamicLibraryInjectionControlReceiptV2,
    pub nobody_owner_authority: NobodyOwnerAuthorityControlReceiptV2,
    pub receipt_set_sha256: String,
    pub before_observation_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PeerControlRestorationReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub identity_packet_sha256: String,
    pub runner_identity_sha256: String,
    pub runner_process_attestation_sha256: String,
    pub alternate_path_identity_sha256: String,
    pub runner_control_set_sha256: Vec<String>,
    pub complete_response_sha256: Vec<String>,
    pub alternate_path_absent: bool,
    pub exact_removal_observation_sha256: String,
    pub securityagent_report: SecurityAgentRawReportEvidenceV2,
    pub securityagent_observation_sha256: String,
}

impl PeerControlRestorationReceiptV2 {
    pub fn validate(
        &self,
        identities: &PeerControlIdentityPacketV2,
        requests: &[FinalizationRequestV2],
        runner_sets: &[PeerControlSetReceiptV2],
        completes: &[FinalizerResponseV2],
    ) -> Result<()> {
        if self.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
            || identities.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || identities.schema_version != EXPERIMENT_VERSION_V2
            || identities.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("peer-control restoration owner, version, or experiment changed")
        }
        if requests.len() != RepetitionV2::ALL.len()
            || runner_sets.len() != RepetitionV2::ALL.len()
            || completes.len() != RepetitionV2::ALL.len()
        {
            bail!("global peer-control restoration does not bind both repetitions")
        }
        for (((repetition, request), runner_set), complete) in RepetitionV2::ALL
            .into_iter()
            .zip(requests)
            .zip(runner_sets)
            .zip(completes)
        {
            runner_set.validate(repetition, request, identities)?;
            substrate_common::macos_retirement_v2::validate_finalizer_response_v2(complete)?;
            if complete.state != FinalizerResponseStateV2::HostComplete
                || complete.request_digest != request.request_digest
            {
                bail!("global peer-control restoration precedes one repetition's HostComplete")
            }
        }
        if self.identity_packet_sha256 != document_sha256_v2(identities)?
            || self.runner_identity_sha256 != document_sha256_v2(&identities.runner_identity)?
            || self.alternate_path_identity_sha256
                != document_sha256_v2(&identities.alternate_path_identity)?
            || self.runner_control_set_sha256
                != runner_sets
                    .iter()
                    .map(document_sha256_v2)
                    .collect::<Result<Vec<_>>>()?
            || self.complete_response_sha256
                != completes
                    .iter()
                    .map(document_sha256_v2)
                    .collect::<Result<Vec<_>>>()?
            || !runner_sets.iter().all(|set| {
                set.runner_identity_sha256 == self.runner_identity_sha256
                    && set.runner_process_attestation_sha256
                        == self.runner_process_attestation_sha256
            })
            || !self.alternate_path_absent
        {
            bail!("global peer-control restoration does not prove exact alternate-path absence after both Complete states")
        }
        self.securityagent_report.validate()?;
        if self.securityagent_observation_sha256 != self.securityagent_report.raw_report_sha256 {
            bail!("global peer-control restoration did not retain its raw SecurityAgent report")
        }
        for digest in self
            .runner_control_set_sha256
            .iter()
            .chain(&self.complete_response_sha256)
        {
            require_digest(digest, "global peer-control causal input")?;
        }
        for (digest, label) in [
            (
                &self.runner_process_attestation_sha256,
                "global runner process attestation",
            ),
            (
                &self.exact_removal_observation_sha256,
                "alternate-path removal observation",
            ),
            (
                &self.securityagent_observation_sha256,
                "alternate-path restoration SecurityAgent observation",
            ),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }
}

impl PeerControlSetReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        request: &FinalizationRequestV2,
        identities: &PeerControlIdentityPacketV2,
    ) -> Result<()> {
        if self.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("peer-control set owner, version, or experiment changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.request_digest != request.request_digest
            || self.identity_packet_sha256 != document_sha256_v2(identities)?
            || self.runner_identity_sha256 != document_sha256_v2(&identities.runner_identity)?
            || self.receipts.len() != PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2.len()
            || !self
                .receipts
                .iter()
                .all(|receipt| receipt.validate(repetition, request, identities).is_ok())
            || self
                .dynamic_library_injection
                .validate(repetition, request, identities)
                .is_err()
            || self
                .nobody_owner_authority
                .validate(repetition, request, identities)
                .is_err()
            || self.receipt_set_sha256
                != peer_control_receipt_set_sha256_v2(
                    &self.receipts,
                    &self.dynamic_library_injection,
                    &self.nobody_owner_authority,
                )?
        {
            bail!(
                "peer-control set does not bind its peer, injection, and owner-authority receipts"
            )
        }
        require_digest(&self.identity_packet_sha256, "peer-control identity packet")?;
        require_digest(&self.runner_identity_sha256, "peer-control runner identity")?;
        require_digest(
            &self.runner_process_attestation_sha256,
            "peer-control runner process attestation",
        )?;
        require_digest(&self.receipt_set_sha256, "peer-control receipt set")?;
        require_digest(
            &self.before_observation_sha256,
            "peer-control before observation",
        )
    }
}

pub fn peer_control_receipt_set_sha256_v2(
    peer_receipts: &[PeerSubstitutionControlReceiptV2],
    dynamic_library_injection: &DynamicLibraryInjectionControlReceiptV2,
    nobody_owner_authority: &NobodyOwnerAuthorityControlReceiptV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct ReceiptSet<'a> {
        domain: &'static str,
        peer_receipts: &'a [PeerSubstitutionControlReceiptV2],
        dynamic_library_injection: &'a DynamicLibraryInjectionControlReceiptV2,
        nobody_owner_authority: &'a NobodyOwnerAuthorityControlReceiptV2,
    }
    document_sha256_v2(&ReceiptSet {
        domain: "substrate.r3-macos-disposable-peer-and-owner-controls.v2",
        peer_receipts,
        dynamic_library_injection,
        nobody_owner_authority,
    })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TransportControlV2 {
    AlternateRequestDigest,
    UnknownField,
    DuplicateField,
    BroadenedTargetLedger,
    ExplicitPathField,
    ExplicitTagField,
    ExplicitServiceField,
    ExplicitAccountField,
    ExplicitPredicateField,
    TrailingByte,
    SecondFrame,
    MissingEof,
}

pub const TRANSPORT_CONTROL_SEQUENCE_V2: [TransportControlV2; 12] = [
    TransportControlV2::AlternateRequestDigest,
    TransportControlV2::UnknownField,
    TransportControlV2::DuplicateField,
    TransportControlV2::BroadenedTargetLedger,
    TransportControlV2::ExplicitPathField,
    TransportControlV2::ExplicitTagField,
    TransportControlV2::ExplicitServiceField,
    TransportControlV2::ExplicitAccountField,
    TransportControlV2::ExplicitPredicateField,
    TransportControlV2::TrailingByte,
    TransportControlV2::SecondFrame,
    TransportControlV2::MissingEof,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlFramingV2 {
    OneFrameAndEof,
    OneFrameTrailingByteAndEof,
    TwoFramesAndEof,
    OneFrameWithoutRequestEof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlProbeV2 {
    pub control: TransportControlV2,
    pub body: Vec<u8>,
    pub framing: ControlFramingV2,
    pub second_body: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlDeadlineClassificationV2 {
    ReturnedBeforeFinalizerDeadline,
    ReturnedAfterFinalizerDeadline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransportControlReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub sequence_ordinal: u8,
    pub control: TransportControlV2,
    pub framing: ControlFramingV2,
    pub sent_material_sha256: String,
    pub expected_rejection_request_digest: String,
    pub response: FinalizerResponseV2,
    pub canonical_response_sha256: String,
    pub response_eof_observed: bool,
    pub finalizer_protocol_deadline_ms: u64,
    pub elapsed_ms: u64,
    pub deadline_classification: ControlDeadlineClassificationV2,
    pub securityagent_evidence: SecurityAgentArmEvidenceV2,
    pub securityagent_observation_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisposableNativeArmV2 {
    FinalizationEffects,
    TerminalBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DisposableNativeArmReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub arm: DisposableNativeArmV2,
    pub request_digest: String,
    pub canonical_response_sha256: String,
    pub response_eof_observed: bool,
    pub securityagent_evidence: SecurityAgentArmEvidenceV2,
    pub securityagent_observation_sha256: String,
}

impl DisposableNativeArmReceiptV2 {
    pub fn validate(&self, repetition: RepetitionV2, response: &FinalizerResponseV2) -> Result<()> {
        if self.schema_owner != TRANSPORT_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("disposable native-arm receipt owner, version, or experiment changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        self.securityagent_evidence.validate()?;
        if !self.response_eof_observed
            || self.request_digest != response.request_digest
            || self.canonical_response_sha256 != sha256_hex_v2(&canonical_bytes_v2(response)?)
            || self.securityagent_observation_sha256
                != document_sha256_v2(&self.securityagent_evidence)?
        {
            bail!("disposable native-arm receipt does not bind response and EOF")
        }
        require_digest(
            &self.securityagent_observation_sha256,
            "native-arm SecurityAgent observation",
        )
    }
}

impl TransportControlReceiptV2 {
    pub fn validate(&self, repetition: RepetitionV2) -> Result<()> {
        if self.schema_owner != TRANSPORT_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("transport-control receipt owner, version, or experiment changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        self.securityagent_evidence.validate()?;
        let expected = TRANSPORT_CONTROL_SEQUENCE_V2
            .get(usize::from(self.sequence_ordinal.saturating_sub(1)))
            .copied()
            .context("transport-control receipt sequence ordinal is outside the closed list")?;
        if self.sequence_ordinal == 0
            || self.control != expected
            || self.finalizer_protocol_deadline_ms != FINALIZER_PROTOCOL_DEADLINE_MS_V2
            || !self.response_eof_observed
            || self.canonical_response_sha256 != sha256_hex_v2(&canonical_bytes_v2(&self.response)?)
            || self.securityagent_observation_sha256
                != document_sha256_v2(&self.securityagent_evidence)?
        {
            bail!("transport-control receipt does not bind exact framing, order, and response EOF")
        }
        require_digest(&self.sent_material_sha256, "transport sent material")?;
        require_digest(
            &self.expected_rejection_request_digest,
            "expected safe-rejection request",
        )?;
        require_digest(
            &self.securityagent_observation_sha256,
            "transport SecurityAgent observation",
        )?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(&self.response)?;
        if self.response.state != FinalizerResponseStateV2::SafePreAcceptanceStop
            || self.response.request_digest != self.expected_rejection_request_digest
            || self.response.journal_head_sha256 != sha256_hex_v2(NO_ACCEPTED_JOURNAL_DOMAIN_V2)
            || self.response.preserving_classification
                != Some(PreservingStopClassificationV2::IdentityOrAuthorityMismatch)
        {
            bail!("negative transport control was not an exact safe pre-acceptance stop")
        }
        match self.control {
            TransportControlV2::MissingEof => {
                if self.framing != ControlFramingV2::OneFrameWithoutRequestEof
                    || self.deadline_classification
                        != ControlDeadlineClassificationV2::ReturnedAfterFinalizerDeadline
                    || self.elapsed_ms
                        < FINALIZER_PROTOCOL_DEADLINE_MS_V2
                            - FINALIZER_PROTOCOL_DEADLINE_EARLY_TOLERANCE_MS_V2
                    || self.elapsed_ms
                        > FINALIZER_PROTOCOL_DEADLINE_MS_V2
                            + FINALIZER_PROTOCOL_DEADLINE_GRACE_MS_V2
                {
                    bail!("missing-EOF control did not return the fixed post-deadline stop")
                }
            }
            _ => {
                if self.deadline_classification
                    != ControlDeadlineClassificationV2::ReturnedBeforeFinalizerDeadline
                    || self.elapsed_ms >= FINALIZER_PROTOCOL_DEADLINE_MS_V2
                {
                    bail!("well-framed negative control did not stop before the protocol deadline")
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransportControlSetReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub control_receipt_sha256: Vec<String>,
    pub before_observation_sha256: String,
    pub after_observation_sha256: String,
    pub accepted_journal_absent: bool,
    pub securityagent_observation_set_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransportControlsCompleteMarkerV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub request_digest: String,
    pub control_receipt_sha256: Vec<String>,
    pub peer_control_set_sha256: String,
    pub before_observation_sha256: String,
}

impl TransportControlsCompleteMarkerV2 {
    pub fn validate_shape(&self, repetition: RepetitionV2) -> Result<()> {
        if self.schema_owner != TRANSPORT_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("transport controls marker identity changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.control_receipt_sha256.len() != TRANSPORT_CONTROL_SEQUENCE_V2.len() {
            bail!("transport controls marker does not contain the fixed receipt count")
        }
        let mut unique = self.control_receipt_sha256.clone();
        unique.sort();
        unique.dedup();
        if unique.len() != self.control_receipt_sha256.len() {
            bail!("transport controls marker repeats a receipt digest")
        }
        for digest in &self.control_receipt_sha256 {
            require_digest(digest, "transport marker control receipt")?;
        }
        require_digest(&self.request_digest, "transport marker request")?;
        require_digest(
            &self.peer_control_set_sha256,
            "transport marker peer-control set",
        )?;
        require_digest(
            &self.before_observation_sha256,
            "transport marker before observation",
        )
    }

    pub fn validate(
        &self,
        repetition: RepetitionV2,
        receipts: &[TransportControlReceiptV2],
    ) -> Result<()> {
        self.validate_shape(repetition)?;
        if receipts.len() != TRANSPORT_CONTROL_SEQUENCE_V2.len()
            || self.control_receipt_sha256.len() != receipts.len()
            || !receipts
                .iter()
                .zip(&self.control_receipt_sha256)
                .all(|(receipt, expected)| {
                    receipt.validate(repetition).is_ok()
                        && substrate_common::macos_retirement_v2::document_sha256_v2(receipt)
                            .is_ok_and(|actual| actual == *expected)
                })
        {
            bail!("transport controls marker does not bind the full exact sequence")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransportControlsObservationReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub repetition: u8,
    pub scope_id: String,
    pub marker_sha256: String,
    pub after_observation_sha256: String,
    pub accepted_journal_absent: bool,
    pub exact_target_state_unchanged: bool,
    pub securityagent_observation_sha256: String,
}

impl TransportControlsObservationReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        marker: &TransportControlsCompleteMarkerV2,
    ) -> Result<()> {
        if self.schema_owner != TRANSPORT_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("transport observation receipt identity changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if self.marker_sha256 != substrate_common::macos_retirement_v2::document_sha256_v2(marker)?
            || self.after_observation_sha256 != marker.before_observation_sha256
            || !self.accepted_journal_absent
            || !self.exact_target_state_unchanged
        {
            bail!("transport observation does not prove exact no-mutation rejection")
        }
        require_digest(
            &self.securityagent_observation_sha256,
            "transport observation SecurityAgent report",
        )
    }
}

impl TransportControlSetReceiptV2 {
    pub fn validate(
        &self,
        repetition: RepetitionV2,
        receipts: &[TransportControlReceiptV2],
    ) -> Result<()> {
        if self.schema_owner != TRANSPORT_CONTROL_RECEIPT_OWNER_V2
            || self.schema_version != EXPERIMENT_VERSION_V2
            || self.experiment_id != EXPERIMENT_ID_V2
        {
            bail!("transport-control set receipt identity changed")
        }
        repetition.validate_binding(self.repetition, &self.scope_id)?;
        if receipts.len() != TRANSPORT_CONTROL_SEQUENCE_V2.len()
            || self.control_receipt_sha256.len() != receipts.len()
            || !receipts
                .iter()
                .zip(&self.control_receipt_sha256)
                .all(|(receipt, expected)| {
                    receipt.validate(repetition).is_ok()
                        && substrate_common::macos_retirement_v2::document_sha256_v2(receipt)
                            .is_ok_and(|actual| actual == *expected)
                })
            || self.before_observation_sha256 != self.after_observation_sha256
            || !self.accepted_journal_absent
        {
            bail!("transport controls do not prove exact no-mutation safe rejection")
        }
        for (digest, label) in [
            (
                &self.before_observation_sha256,
                "transport before observation",
            ),
            (
                &self.after_observation_sha256,
                "transport after observation",
            ),
            (
                &self.securityagent_observation_set_sha256,
                "transport SecurityAgent set",
            ),
        ] {
            require_digest(digest, label)?;
        }
        Ok(())
    }
}

pub fn build_transport_control_probe_v2(
    request: &FinalizationRequestV2,
    receipt: &PublisherPreRemovalReceiptV2,
    control: TransportControlV2,
) -> Result<ControlProbeV2> {
    let (validated_receipt, _, _) =
        substrate_common::macos_retirement_v2::validate_finalization_request_v2(request, None)?;
    if validated_receipt != *receipt
        || receipt.target_set_kind != TargetSetKindV2::DisposableCapability
    {
        bail!("transport controls are available only for the validated disposable request")
    }
    build_probe_from_validated_v2(request, receipt, control)
}

fn build_probe_from_validated_v2(
    request: &FinalizationRequestV2,
    receipt: &PublisherPreRemovalReceiptV2,
    control: TransportControlV2,
) -> Result<ControlProbeV2> {
    let canonical = canonical_bytes_v2(request)?;
    let (body, framing, second_body) = match control {
        TransportControlV2::AlternateRequestDigest => {
            let mut changed = request.clone();
            changed.request_digest = sha256_hex_v2(
                format!(
                    "substrate.r3-macos-disposable.alternate-request-digest.v2\0{}",
                    receipt.scope_id
                )
                .as_bytes(),
            );
            (
                canonical_bytes_v2(&changed)?,
                ControlFramingV2::OneFrameAndEof,
                None,
            )
        }
        TransportControlV2::UnknownField => (
            canonical_request_with_field(
                request,
                "unknown_transport_control_v2",
                Value::Bool(true),
            )?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::DuplicateField => (
            duplicate_schema_version_field(&canonical)?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::BroadenedTargetLedger => (
            broaden_target_ledger(request)?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::ExplicitPathField => (
            canonical_request_with_field(
                request,
                "path",
                Value::String(format!(
                    "/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/disposable/{}/forbidden-target",
                    receipt.scope_id
                )),
            )?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::ExplicitTagField => (
            canonical_request_with_field(
                request,
                "application_tag",
                Value::String(format!("{}:alternate-signing-key", receipt.scope_id)),
            )?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::ExplicitServiceField => (
            canonical_request_with_field(
                request,
                "keychain_service",
                Value::String(format!(
                    "com.atomize.substrate.r3-macos-disposable.v2.{}",
                    receipt.scope_id
                )),
            )?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::ExplicitAccountField => (
            canonical_request_with_field(
                request,
                "keychain_account",
                Value::String(format!("{}:alternate-account", receipt.scope_id)),
            )?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::ExplicitPredicateField => (
            canonical_request_with_field(
                request,
                "keychain_predicate",
                Value::String("kSecMatchLimitAll".to_string()),
            )?,
            ControlFramingV2::OneFrameAndEof,
            None,
        ),
        TransportControlV2::TrailingByte => (
            canonical,
            ControlFramingV2::OneFrameTrailingByteAndEof,
            None,
        ),
        TransportControlV2::SecondFrame => (
            canonical.clone(),
            ControlFramingV2::TwoFramesAndEof,
            Some(canonical),
        ),
        TransportControlV2::MissingEof => (
            canonical,
            ControlFramingV2::OneFrameWithoutRequestEof,
            None,
        ),
    };
    Ok(ControlProbeV2 {
        control,
        body,
        framing,
        second_body,
    })
}

pub fn sent_material_sha256_v2(probe: &ControlProbeV2) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct SentMaterial {
        control: TransportControlV2,
        framing: ControlFramingV2,
        body_sha256: String,
        second_body_sha256: Option<String>,
        trailing_byte: Option<u8>,
    }
    substrate_common::macos_retirement_v2::document_sha256_v2(&SentMaterial {
        control: probe.control,
        framing: probe.framing,
        body_sha256: sha256_hex_v2(&probe.body),
        second_body_sha256: probe.second_body.as_ref().map(|body| sha256_hex_v2(body)),
        trailing_byte: (probe.framing == ControlFramingV2::OneFrameTrailingByteAndEof).then_some(0),
    })
}

pub fn expected_safe_rejection_request_digest_v2(probe: &ControlProbeV2) -> String {
    let observed = match probe.framing {
        ControlFramingV2::OneFrameAndEof => probe.body.as_slice(),
        ControlFramingV2::OneFrameTrailingByteAndEof
        | ControlFramingV2::TwoFramesAndEof
        | ControlFramingV2::OneFrameWithoutRequestEof => &[],
    };
    safe_rejection_request_digest_v2(observed)
}

pub fn safe_rejection_request_digest_v2(observed: &[u8]) -> String {
    let mut material = Vec::with_capacity(SAFE_REJECTION_REQUEST_DOMAIN_V2.len() + observed.len());
    material.extend_from_slice(SAFE_REJECTION_REQUEST_DOMAIN_V2);
    material.extend_from_slice(observed);
    sha256_hex_v2(&material)
}

fn validate_probe_executable(
    identity: &ExecutableIdentityV2,
    expected_path: &str,
    expected_signing_identifier: &str,
) -> Result<()> {
    if identity.intended_path != expected_path
        || identity.signing_identifier != expected_signing_identifier
        || identity.executable_size == 0
        || identity.source_commit.len() != 40
        || identity.source_tree.len() != 40
        || identity.cdhash.len() != 40
        || identity.designated_requirement.is_empty()
    {
        bail!("peer probe executable identity path, code identity, or provenance changed")
    }
    for (digest, label) in [
        (&identity.source_hashes_sha256, "peer probe source hashes"),
        (&identity.build_inputs_sha256, "peer probe build inputs"),
        (&identity.executable_sha256, "peer probe executable"),
        (
            &identity.physical_identity_sha256,
            "peer probe physical identity",
        ),
    ] {
        require_digest(digest, label)?;
    }
    if !identity
        .source_commit
        .bytes()
        .chain(identity.source_tree.bytes())
        .chain(identity.cdhash.bytes())
        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("peer probe commit, tree, or CDHash is not lowercase hexadecimal")
    }
    Ok(())
}

fn canonical_request_with_field(
    request: &FinalizationRequestV2,
    name: &str,
    value: Value,
) -> Result<Vec<u8>> {
    let mut object = request_object(request)?;
    if object.insert(name.to_string(), value).is_some() {
        bail!("transport control attempted to replace an existing request field")
    }
    canonical_bytes_v2(&Value::Object(object))
}

fn request_object(request: &FinalizationRequestV2) -> Result<Map<String, Value>> {
    serde_json::to_value(request)?
        .as_object()
        .cloned()
        .context("finalization request is not an object")
}

fn duplicate_schema_version_field(canonical: &[u8]) -> Result<Vec<u8>> {
    let mut text =
        String::from_utf8(canonical.to_vec()).context("canonical request is not UTF-8")?;
    let needle = "\"schema_version\":";
    let start = text
        .find(needle)
        .context("canonical request lacks schema_version")?;
    let value_start = start + needle.len();
    let value_end = text[value_start..]
        .find([',', '}'])
        .map(|offset| value_start + offset)
        .context("canonical schema_version lacks a delimiter")?;
    let duplicate = format!(",\"schema_version\":{}", &text[value_start..value_end]);
    let close = text.pop().filter(|character| *character == '}');
    if close.is_none() {
        bail!("canonical request object lacks its closing delimiter")
    }
    text.push_str(&duplicate);
    text.push('}');
    Ok(text.into_bytes())
}

fn broaden_target_ledger(request: &FinalizationRequestV2) -> Result<Vec<u8>> {
    let mut receipt_value: Value = parse_canonical_v2(
        &URL_SAFE_NO_PAD
            .decode(request.publisher_receipt.as_bytes())
            .context("decode embedded disposable publisher receipt")?,
    )?;
    let receipt = receipt_value
        .as_object_mut()
        .context("embedded publisher receipt is not an object")?;
    let ledger = receipt
        .get_mut("target_ledger")
        .and_then(Value::as_array_mut)
        .context("embedded publisher receipt lacks target ledger")?;
    let duplicated = ledger
        .last()
        .cloned()
        .context("embedded publisher target ledger is empty")?;
    ledger.push(duplicated);
    let mut changed = request.clone();
    changed.publisher_receipt = URL_SAFE_NO_PAD.encode(canonical_bytes_v2(&receipt_value)?);
    canonical_bytes_v2(&changed)
}

fn require_digest(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not an exact lowercase SHA-256 digest")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use substrate_common::macos_retirement_v2::{
        FrozenFinalizationIntentV2, HostToFinalizerSuccessorCapsuleV2,
    };

    fn digest(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn request() -> FinalizationRequestV2 {
        FinalizationRequestV2 {
            schema_owner: "substrate.r3-macos-evidence-finalizer".to_string(),
            schema_version: 2,
            request_digest: digest(1),
            publisher_receipt: URL_SAFE_NO_PAD.encode(b"{}"),
            harness_acknowledgement: URL_SAFE_NO_PAD.encode(b"{}"),
            protected_cas_binding: URL_SAFE_NO_PAD.encode(b"{}"),
            successor_capsule: HostToFinalizerSuccessorCapsuleV2 {
                predecessor_journal_head_sha256: digest(2),
                retry_state_sha256: digest(3),
                intent: FrozenFinalizationIntentV2 {
                    evidence_id: "fixed".to_string(),
                    scope_id: RepetitionV2::One.scope_id().to_string(),
                    receipt_sha256: digest(4),
                    acknowledgement_sha256: digest(5),
                    protected_cas_generation: 2,
                    protected_cas_predecessor_head_sha256: digest(6),
                    target_ledger_sha256: digest(7),
                    effect_plan_sha256: digest(8),
                    guest_successor_capsule_sha256: digest(9),
                    guest_parity_sha256: digest(10),
                    current_lock_identity_sha256: digest(11),
                    signer_access_control_sha256: digest(12),
                    finalizer_identity_sha256: digest(13),
                    coordinator_identity_sha256: digest(14),
                    launch_identity_sha256: digest(15),
                    capability_digest: digest(16),
                },
            },
        }
    }

    #[test]
    fn malformed_controls_are_deterministic_and_field_specific() {
        let request = request();
        for control in [
            TransportControlV2::AlternateRequestDigest,
            TransportControlV2::UnknownField,
            TransportControlV2::DuplicateField,
            TransportControlV2::ExplicitPathField,
            TransportControlV2::ExplicitTagField,
            TransportControlV2::ExplicitServiceField,
            TransportControlV2::ExplicitAccountField,
            TransportControlV2::ExplicitPredicateField,
            TransportControlV2::TrailingByte,
            TransportControlV2::SecondFrame,
            TransportControlV2::MissingEof,
        ] {
            let first = build_probe_from_validated_v2(&request, &dummy_receipt(), control).unwrap();
            let second =
                build_probe_from_validated_v2(&request, &dummy_receipt(), control).unwrap();
            assert_eq!(first, second);
            assert!(!first.body.is_empty());
            assert_eq!(
                first.framing == ControlFramingV2::OneFrameWithoutRequestEof,
                control == TransportControlV2::MissingEof
            );
        }
        let duplicate = build_probe_from_validated_v2(
            &request,
            &dummy_receipt(),
            TransportControlV2::DuplicateField,
        )
        .unwrap();
        assert!(parse_canonical_v2::<FinalizationRequestV2>(&duplicate.body).is_err());
    }

    fn dummy_receipt() -> PublisherPreRemovalReceiptV2 {
        // Only fields read by the unchecked mutation test are meaningful.
        serde_json::from_value(serde_json::json!({
            "schema_owner":"x","schema_version":2,"signature_domain":"x",
            "evidence_id":"x","scope_id":RepetitionV2::One.scope_id(),
            "target_set_kind":"disposable_capability","issued_at_unix_ns":1,
            "expires_at_unix_ns":2,"host_state":"pre_removal_receipt_signed",
            "guest_successor_capsule":{"schema_owner":"x","schema_version":2,"evidence_id":"x","scope_id":RepetitionV2::One.scope_id(),"handoff_request_digest":digest(28),"guest_handoff_capsule_sha256":digest(29),"guest_host_acceptance_sha256":digest(30),"guest_receipt_sha256":digest(1),"guest_acknowledgement_sha256":digest(2),"guest_protected_cas_head_sha256":digest(3),"guest_effect_plan_sha256":digest(31),"guest_effects_response_sha256":digest(4),"guest_parity_proof_sha256":digest(5),"guest_parity_host_binding_sha256":digest(32),"guest_journal_head_sha256":digest(33),"r6_predecessor_consumed_sha256":digest(6),"r6_terminal_host_record_sha256":digest(7),"guest_anchor_acknowledgement_sha256":digest(8),"guest_consumption_marker_acknowledgement_sha256":digest(9),"retry_state_sha256":digest(10)},
            "guest_successor_capsule_sha256":digest(11),"guest_parity_sha256":digest(12),
            "before_observation_sha256":digest(13),"quiesced_observation_sha256":digest(14),
            "target_ledger":[],"target_ledger_sha256":digest(15),"effect_plan_sha256":digest(16),
            "protected_cas_generation":1,"protected_cas_head_sha256":digest(17),
            "current_lock_identity_sha256":digest(18),"signer_access_control_sha256":digest(19),
            "publisher_signer_spki_der":"AA","harness_public_key":"AA",
            "finalizer_identity":{"source_commit":"1".repeat(40),"source_tree":"2".repeat(40),"source_hashes_sha256":digest(20),"build_inputs_sha256":digest(21),"executable_sha256":digest(22),"executable_size":1,"intended_path":"x","physical_identity_sha256":digest(23),"signing_identifier":"x","designated_requirement":"x","cdhash":"3".repeat(40)},
            "coordinator_identity":{"source_commit":"1".repeat(40),"source_tree":"2".repeat(40),"source_hashes_sha256":digest(20),"build_inputs_sha256":digest(21),"executable_sha256":digest(22),"executable_size":1,"intended_path":"x","physical_identity_sha256":digest(23),"signing_identifier":"x","designated_requirement":"x","cdhash":"3".repeat(40)},
            "coordinator_process":{"effective_uid":501,"effective_gid":20,"canonical_account":"x","pidversion_required":true,"process_start_identity_sha256":digest(24),"executable_identity_sha256":digest(25)},
            "launch_identity":{"launchd_label":"x","launchd_plist_path":"x","launchd_plist_sha256":digest(26),"endpoint":"x","endpoint_owner_uid":0,"endpoint_group_gid":20,"endpoint_mode":"0660","launch_socket_name":"Listener","finalizer_effective_uid":0},
            "capability_digest":digest(27),"signature":{"algorithm":"x","public_key":"AA","signature":"AA"}
        })).unwrap()
    }

    #[test]
    fn sequence_contains_every_required_rejection_once() {
        let mut ordered = TRANSPORT_CONTROL_SEQUENCE_V2.to_vec();
        ordered.sort();
        ordered.dedup();
        assert_eq!(ordered.len(), TRANSPORT_CONTROL_SEQUENCE_V2.len());
        assert_eq!(
            TRANSPORT_CONTROL_SEQUENCE_V2.last(),
            Some(&TransportControlV2::MissingEof)
        );
        assert_eq!(
            PEER_EXPECTED_TERMINATION_SEQUENCE_V2,
            [PeerProbeTerminationV2::ConnectionClosedBeforeResponse; 3]
        );
    }

    #[test]
    fn securityagent_observer_alerts_before_ready_and_coordinator_exits_immediately() {
        let observer = include_str!("../../native/securityagent_observer.swift");
        let freeze = include_str!("../../../../scripts/mac/freeze-r3-macos-finalizer-candidate.sh");
        assert!(observer.contains("@main"));
        assert!(freeze.contains("\"${SWIFTC}\" -parse-as-library -O -whole-module-optimization"));
        let prearm_alert = observer
            .find("current.processes.contains(where: { $0.active })")
            .unwrap();
        let ready = observer.find("Data(\"READY\\n\".utf8)").unwrap();
        assert!(prearm_alert < ready);
        assert!(observer.contains("Data(\"ALERT\\n\".utf8)"));
        assert!(observer.contains("exit(86)"));
        assert!(!observer.contains("standardOutput.synchronizeFile()"));
        assert!(!observer.contains("standardError.synchronizeFile()"));

        let coordinator = include_str!("../bin/coordinator.rs");
        assert!(coordinator.contains("line == \"ALERT\\n\""));
        assert!(coordinator.contains("std::process::exit(86)"));
        let replacement = coordinator
            .find("let (replacement, replacement_started)")
            .unwrap();
        let finish_current = coordinator[replacement..]
            .find("finish_securityagent_observer(observer)")
            .unwrap()
            + replacement;
        assert!(replacement < finish_current);
        assert!(coordinator.contains("OBSERVER_REARM_READY_TIMEOUT"));
        assert!(coordinator.contains("unwrap_or_else(|_| std::process::exit(78))"));
        assert_eq!(
            coordinator.matches("clear_process_environment()?").count(),
            2
        );
        let ambient = include_str!("../ambient.rs");
        assert!(ambient.contains("libc::unsetenv(name.as_ptr())"));
        assert!(ambient.contains("vars_os().next().is_some()"));
        assert!(coordinator.contains(".env_clear()"));
    }
}
