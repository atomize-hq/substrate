//! One-admin, no-selector root runner for the complete disposable macOS experiment.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::{size_of, MaybeUninit};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use substrate_common::macos_retirement_v2::ExecutableIdentityV2;
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, parse_canonical_bounded_v2, parse_canonical_v2,
    sha256_hex_v2, validate_launch_identity_v2, FinalizationRequestV2, FinalizerResponseV2,
    LaunchIdentityV2, MAC_R3_COORDINATOR_INBOX_ROOT_V2, MAC_R3_COORDINATOR_PATH_V2,
    MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2, MAC_R3_FINALIZER_ENDPOINT_V2,
    MAC_R3_FINALIZER_JOURNAL_ROOT_V2, MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_PLIST_PATH_V2, MAC_R3_FINALIZER_REQUEST_PATH_V2,
    MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2, MAC_R3_RETIREMENT_LATCH_ROOT_V2,
    MAC_R3_TERMINAL_BINDING_PATH_V2,
};
use substrate_r3_macos_finalizer::contract::JournalEventKind;
use substrate_r3_macos_finalizer::experiment::controls::{
    dynamic_library_no_mutation_binding_sha256_v2, nobody_owner_no_mutation_binding_sha256_v2,
    peer_control_receipt_set_sha256_v2, peer_substitution_no_mutation_binding_sha256_v2,
    single_securityagent_arm_evidence_v2, single_securityagent_terminal_alert_evidence_v2,
    BenignInjectionTerminationV2, BoundedRawStreamEvidenceV2, DisposableNativeArmReceiptV2,
    DynamicLibraryInjectionControlReceiptV2, NobodyOwnerAuthorityAttemptV2,
    NobodyOwnerAuthorityControlReceiptV2, NobodyOwnerAuthorityOperationV2,
    NobodyOwnerProbeProcessAttestationV2, PeerControlIdentityPacketV2,
    PeerControlRestorationReceiptV2, PeerControlSetReceiptV2, PeerControlsReadyMarkerV2,
    PeerProbeExchangeResultV2, PeerProbeTerminationV2, PeerSubstitutionControlReceiptV2,
    PeerSubstitutionControlV2, SecurityAgentArmEvidenceV2, SecurityAgentRawReportEvidenceV2,
    TransportControlReceiptV2, BENIGN_INJECTION_DYLD_ENVIRONMENT_KEY_V2,
    NOBODY_OWNER_AUTHORITY_SEQUENCE_V2, NOBODY_OWNER_CONTROL_RECEIPT_OWNER_V2,
    NOBODY_PERSISTED_OWNER_GID_V2, NOBODY_PERSISTED_OWNER_TYPE_V2, NOBODY_PERSISTED_OWNER_UID_V2,
    NOBODY_PRINCIPAL_ACCOUNT_V2, NOBODY_PRINCIPAL_GID_V2, NOBODY_PRINCIPAL_GROUP_V2,
    NOBODY_PRINCIPAL_UID_V2, PEER_CONTROL_RECEIPT_OWNER_V2, PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2,
};
use substrate_r3_macos_finalizer::experiment::durable::{ExperimentArtifactV2, ExperimentStoreV2};
use substrate_r3_macos_finalizer::experiment::evidence_export::{
    build_native_evidence_export_v2, build_repetition_native_evidence_export_v2,
    build_runner_private_archive_v2, build_securityagent_report_archive_v2,
    native_evidence_cleanup_artifact_set_sha256_v2, runner_private_archive_entry_max_bytes_v2,
    runner_private_archive_union_v2, NativeEvidenceCleanupReceiptV2,
    NativeEvidenceExportAcknowledgementV2, NativeEvidenceExportV2,
    RepetitionNativeEvidenceExportV2, RunnerPrivateArchiveEntryV2, RunnerPrivateArchiveV2,
    SecurityAgentEvidenceArmV2, SecurityAgentEvidenceBindingV2, SecurityAgentReportArchiveV2,
    GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2, GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
    NATIVE_EVIDENCE_EXPORT_OWNER_V2, RUNNER_PRIVATE_ARCHIVE_MAX_BYTES_V2, RUNNER_PRIVATE_ROOT_V2,
};
use substrate_r3_macos_finalizer::experiment::freeze_manifest::{
    build_candidate_freeze_manifest_v2, build_candidate_freeze_supporting_manifests_v2,
    candidate_freeze_coordinator_provenance_input_v2, candidate_freeze_global_provenance_input_v2,
    expected_candidate_freeze_install_directories_v2, CandidateFreezeArtifactRoleV2,
    CandidateFreezeManifestV2, CANDIDATE_FREEZE_INSTALLED_ARTIFACT_MAX_BYTES_V2,
    CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2, CANDIDATE_FREEZE_MANIFEST_PATH_V2,
};
use substrate_r3_macos_finalizer::experiment::pre_effect::{
    build_global_pre_effect_packet_v2, global_nonce_absence_plan_v2, CreatorNativeArmReceiptV2,
    CreatorNativeArmV2, CreatorNativeClassificationV2, CreatorNativeOperationReceiptV2,
    CreatorNativeOperationV2, CreatorRouteReceiptSetV2, GlobalExactAbsenceClassificationV2,
    GlobalNonceAbsenceBaselineV2, GlobalNonceAbsenceObservationV2, GlobalNonceObjectKindV2,
    GlobalPreEffectDynamicBindingsV2, GlobalPreEffectPacketV2, InstalledCandidateManifestBindingV2,
    RootInstallClaimsBindingV2, RootInstallCompletionV2, RootInstallPhysicalIdentityV2,
    RootInstallPreclaimV2, GLOBAL_PRE_EFFECT_PACKET_OWNER_V2,
    MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2, ROOT_INSTALL_CLAIM_ROOT_V2,
    ROOT_INSTALL_COMPLETION_PATH_V2, ROOT_INSTALL_PRECLAIM_PATH_V2,
};
use substrate_r3_macos_finalizer::experiment::process::{
    CoordinatorProcessAttestationV2, SupplementaryGroupAttestationV2,
};
use substrate_r3_macos_finalizer::experiment::publisher_protocol::{
    external_exchange_path_v2, global_external_exchange_path_v2, root_publisher_path_v2,
    CandidateIdentityPacketV2, EmergencyFailureClassificationV2, EmergencyRollbackMarkerV2,
    EmergencyRollbackReceiptV2, PreCreationEmergencyRollbackMarkerV2, PublisherArtifactV2,
    PublisherPreparedInputV2, PublisherProgressV2, PublisherSecurityAgentObservationV2,
    PublisherStageV2, RestorationReceiptV2, SurrogateCreationReceiptV2,
    PUBLISHER_PROTOCOL_OWNER_V2, PUBLISHER_STAGE_SEQUENCE_V2,
};
use substrate_r3_macos_finalizer::experiment::{
    CodeSigningPostureV2, RepetitionV2, ALTERNATE_COORDINATOR_PATH_V2,
    BENIGN_INJECTION_LIBRARY_PATH_V2, BENIGN_INJECTION_LIBRARY_SIGNING_IDENTIFIER_V2,
    BENIGN_INJECTION_MARKER_FD_V2, CANDIDATE_IDENTITY_PACKET_PATH_V2,
    DISPOSABLE_HARNESS_ACCOUNT_V2, DISPOSABLE_HARNESS_PATH_V2,
    DISPOSABLE_HARNESS_SIGNING_IDENTIFIER_V2, DISPOSABLE_HARNESS_UID_V2,
    DISPOSABLE_PREPARED_INPUT_PATH_V2, EXPERIMENT_ID_V2, EXPERIMENT_ROOT_V2, EXPERIMENT_VERSION_V2,
    NOBODY_OWNER_PROBE_PATH_V2, NOBODY_OWNER_PROBE_SIGNING_IDENTIFIER_V2, PEER_CODE_PROBE_PATH_V2,
    PEER_CODE_PROBE_SIGNING_IDENTIFIER_V2, PEER_CONTROL_IDENTITY_PACKET_PATH_V2,
    SECURITYAGENT_OBSERVER_PATH_V2,
};
use substrate_r3_macos_finalizer::journal::LockedJournal;

use crate::experiment::{
    CreatorRollbackMarkerV2, CreatorRollbackPhaseV2, CreatorRollbackReceiptV2, MarkerRoot,
    MarkerState,
};
use crate::ffi::{DisposableAclKind, ExactAbsentKeyAttribute, NonInteractiveSecurity};
use crate::immutable_publish::{
    publish_exact_file, read_exact_published_file, read_staged_file, stage_exact_file,
    PublishIdentityV2, PublishMode,
};
use crate::runner_identity::{
    benign_injection_library_code, capability_digest, coordinator_code, creator_code,
    finalizer_code, harness_code, install_candidate_freeze_authority, launch_plist_sha256,
    measure_frozen_executable, measure_frozen_executable_and_posture, measure_runner_executable,
    nobody_owner_probe_code, observer_code, peer_probe_code, preflight_frozen_executable,
    publisher_code, require_non_placeholder_digest, wrong_code, FrozenCode,
};
use crate::securityagent::{
    measured_child_supplementary_groups_v2, observe_idle_baseline_sha256,
    observe_rearmed_root_operation_with_raw, observe_root_operation_with_raw,
    observe_sealed_child_with, observe_stopped_child_startup, MeasuredCommandV2, ObservedChildV2,
    SEALED_GROUP_MEASUREMENT_FRAME_BYTES_V2, SEALED_GROUP_MEASUREMENT_MAGIC_V2,
};
use crate::{
    compiled_disposable_target_config, compiled_disposable_wrong_config,
    probe_nonmatch_owner_access_in_memory, probe_nonmatch_owner_candidate, FixedRepetitionV2,
    NobodyOwnerNativeReceiptV2, NobodyOwnerProbeCommandV2, CREATOR_EXECUTABLE_PATH,
    CREATOR_REPETITION_SCOPE_1, CREATOR_REPETITION_SCOPE_2, DISPOSABLE_FINALIZER_SCOPE_1,
    DISPOSABLE_FINALIZER_SCOPE_2, DISPOSABLE_PUBLISHER_EXECUTABLE_PATH, DISPOSABLE_PUBLISHER_ROOT,
    MARKER_PATH, MARKER_ROOT, SYSTEM_KEYCHAIN_PATH, WRONG_IDENTITY_EXECUTABLE_PATH,
};

pub const EXPERIMENT_RUNNER_EXECUTABLE_PATH: &str =
    "/Library/PrivilegedHelperTools/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2";
const EXPERIMENT_RUNNER_SIGNING_IDENTIFIER: &str =
    "com.atomize.substrate.r3-macos-disposable-experiment-runner.v2";
const CREATOR_SIGNING_IDENTIFIER: &str = "com.atomize.substrate.r3-macos-signer-acl-creator.v1";
const WRONG_SIGNING_IDENTIFIER: &str =
    "com.atomize.substrate.r3-macos-signer-acl-wrong-identity.v1";
const OBSERVER_SIGNING_IDENTIFIER: &str =
    "com.atomize.substrate.r3-macos-securityagent-observer.v2";
const RUNNER_ROOT: &str =
    "/private/var/db/com.atomize.substrate.r3-macos-disposable-experiment-runner.v2";
const ROOT_DIRECTORY_MODE: libc::mode_t = libc::S_IFDIR | 0o700;
const EXTERNAL_INPUT_MODE: libc::mode_t = libc::S_IFREG | 0o400;
const POLL: Duration = Duration::from_millis(50);
const MAX_CHILD_OUTPUT: usize = 1024 * 1024;
const INSTALLED_ARTIFACT_MAX_BYTES: u64 = CANDIDATE_FREEZE_INSTALLED_ARTIFACT_MAX_BYTES_V2;
const MAX_NATIVE_EVIDENCE_DOCUMENT: usize = 96 * 1024 * 1024;
const SECURITYAGENT_ALERT_EXIT_CODE: i32 = 86;
const PROCESS_STATUS_STOPPED_V2: u32 = 4;
const PUBLISHER_UI_STOP_CURSOR_NAME: &str = "publisher-ui-terminal-stop.cursor.v2.json";
const PUBLISHER_UI_HARNESS_TERMINATION_NAME: &str =
    "publisher-ui-terminal-harness-termination.v2.json";
const PUBLISHER_UI_PUBLISHER_TERMINATION_NAME: &str =
    "publisher-ui-terminal-publisher-termination.v2.json";
const PUBLISHER_UI_TRIGGER_EVIDENCE_NAME: &str =
    "publisher-ui-terminal-trigger.securityagent-arm.v2.json";
const PUBLISHER_UI_ROLLBACK_PREPARED_NAME: &str = "publisher-ui-terminal-rollback-prepared.v2.json";
const PUBLISHER_UI_ROLLBACK_PROCESS_NAME: &str = "publisher-ui-terminal-rollback-process.v2.json";
const PUBLISHER_UI_TERMINAL_RECEIPT_NAME: &str = "publisher-ui-terminal-stop.receipt.v2.json";
const GENERAL_FAILURE_CURSOR_NAME: &str = "general-failure-restoration.cursor.v2.json";
const GENERAL_FAILURE_PEER_TERMINATION_NAME: &str = "general-failure-peer-termination.v2.json";
const GENERAL_FAILURE_ROLLBACK_PREPARED_NAME: &str = "general-failure-rollback-prepared.v2.json";
const GENERAL_FAILURE_ROLLBACK_INVOKED_NAME: &str = "general-failure-rollback-invoked.v2.json";
const GENERAL_FAILURE_ROLLBACK_RESULT_NAME: &str = "general-failure-rollback-result.v2.json";
const GENERAL_FAILURE_RECEIPT_NAME: &str = "general-failure-restoration.receipt.v2.json";
const GENERAL_FAILURE_ACCEPTED_REJOIN_NAME: &str =
    "general-failure-authenticated-accepted-rejoin.v2.json";
const FINALIZER_BOOTSTRAP_PREPARED_NAME: &str = "finalizer-bootstrap-prepared.v2.json";
const FINALIZER_BOOTSTRAP_RESULT_NAME: &str = "finalizer-bootstrap-result.v2.json";
const FINALIZER_SERVICE_OBSERVATION_NAME: &str = "finalizer-service-observation.v2.json";
const FINALIZER_BOOTOUT_PREPARED_NAME: &str = "finalizer-bootout-prepared.v2.json";
const FINALIZER_BOOTOUT_RESULT_NAME: &str = "finalizer-bootout-result.v2.json";
const NATIVE_CLEANUP_PLAN_NAME: &str = "native-evidence-cleanup-plan.v2.json";
const NATIVE_CLEANUP_STEP_PREFIX: &str = "native-evidence-cleanup-step";
const ROOT_OPERATION_UI_STOP_CURSOR_NAME: &str = "root-operation-ui-terminal-stop.cursor.v2.json";
const ROOT_OPERATION_UI_LIVE_PEERS_NAME: &str = "root-operation-live-peers.v2.json";
const ROOT_OPERATION_UI_LIVE_PEER_TERMINATION_NAME: &str =
    "root-operation-ui-live-peer-termination.v2.json";
const ROOT_OPERATION_UI_TERMINAL_RECEIPT_NAME: &str =
    "root-operation-ui-terminal-stop.receipt.v2.json";
const TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME: &str =
    "terminal-admin-cleanup-authorization.v2.json";
const TERMINAL_RUNNER_CHILD_ARCHIVE_MAX_BYTES_V2: u64 = 67_108_864;
const CREATOR_TERMINAL_CLEANUP_CURSOR_NAME: &str = "creator-terminal-cleanup.cursor.v2.json";
const CREATOR_TERMINAL_RECEIPT_NAME: &str = "creator-terminal-cleanup.receipt.v2.json";
const DISPOSABLE_CURRENT_LOCK_BYTES: &[u8] = b"substrate-r3-disposable-current-lock-v2\n";
type CompletedJournalScopeExpectedV2 = BTreeMap<String, (Option<Vec<u8>>, libc::mode_t)>;
const DISPOSABLE_TERMINAL_LATCH_BYTES: &[u8] = b"substrate-r3-retirement-terminal-latch-v2\n";

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct CreatorArmObservationV2 {
    schema_owner: &'static str,
    schema_version: u32,
    ordinal: u8,
    state_before: String,
    state_after: String,
    executable_path: &'static str,
    exit_code: Option<i32>,
    stdout_sha256: String,
    stderr_sha256: String,
    securityagent_report_sha256: String,
    unexpected_ui_observed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreatorArmPreparedCursorV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    global_ordinal: u8,
    repetition: u8,
    state_before: String,
    executable_path: String,
    executable_identity_sha256: String,
    invocation_may_begin: bool,
    missing_observation_requires_terminal_rollback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreatorArmObservedCursorV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    prepared_sha256: String,
    shared_receipt_sha256: String,
    raw_securityagent_report_sha256: String,
    terminal_no_reinvoke: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreatorChildIdentityV2 {
    pid: i32,
    effective_uid: u32,
    effective_gid: u32,
    process_start_identity_sha256: String,
    executable_path: String,
    executable_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreatorChildDispositionPreparedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    arm_prepared_sha256: String,
    durable_process_attestation_sha256: Option<String>,
    exact_child: Option<CreatorChildIdentityV2>,
    individual_sigkill_authorized: bool,
    normal_arm_resume_forbidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreatorChildDispositionObservedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    disposition_prepared_sha256: String,
    individual_sigkill_sent: bool,
    exact_creator_and_wrong_processes_absent: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct CreatorTerminalAlertBindingV2<'a> {
    schema_owner: &'static str,
    schema_version: u32,
    experiment_id: &'static str,
    arm_observation_sha256: String,
    arm_evidence_sha256: String,
    arm_evidence: &'a SecurityAgentArmEvidenceV2,
    terminal_no_normal_retry: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct CreatorProcessAttestationV2 {
    schema_owner: &'static str,
    schema_version: u32,
    experiment_id: &'static str,
    repetition: u8,
    sequence_ordinal: u8,
    arm: CreatorNativeArmV2,
    pid: i32,
    effective_uid: u32,
    effective_gid: u32,
    supplementary_groups: SupplementaryGroupAttestationV2,
    canonical_account: String,
    process_start_identity_sha256: String,
    executable_identity_sha256: String,
    executable_path: &'static str,
    argv_count: u8,
    environment_variable_count: u32,
    stdin_is_dev_null: bool,
    cwd: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct CreatorRollbackProcessAttestationV2 {
    schema_owner: &'static str,
    schema_version: u32,
    experiment_id: &'static str,
    repetition: FixedRepetitionV2,
    rollback_phase_before: CreatorRollbackPhaseV2,
    failure_observation_sha256: String,
    pid: i32,
    effective_uid: u32,
    effective_gid: u32,
    supplementary_groups: SupplementaryGroupAttestationV2,
    canonical_account: String,
    process_start_identity_sha256: String,
    executable_identity_sha256: String,
    executable_path: &'static str,
    argv_count: u8,
    environment_variable_count: u32,
    stdin_is_dev_null: bool,
    cwd: &'static str,
    resume_signal_authorized_count: u8,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RunnerPeerProcessAttestationV2 {
    schema_owner: &'static str,
    schema_version: u32,
    repetition: u8,
    control: PeerSubstitutionControlV2,
    pid: i32,
    effective_uid: u32,
    effective_gid: u32,
    supplementary_groups: SupplementaryGroupAttestationV2,
    canonical_account: String,
    process_start_identity_sha256: String,
    executable_identity_sha256: String,
    before_observation_sha256: String,
    accepted_journal_absent_after: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublisherExitDispositionV2 {
    Completed,
    TerminalSecurityAgentAlert,
    TerminalClosedFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublisherUiStopCursorV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    exit_code: i32,
    progress_sha256: String,
    progress_stage: PublisherStageV2,
    creation_receipt_sha256: Option<String>,
    failure_classification: EmergencyFailureClassificationV2,
    trigger_origin: PublisherUiTriggerOriginV2,
    triggering_securityagent_evidence_sha256: String,
    publisher_exit: Option<ChildExitObservationV2>,
    harness_exit: Option<ChildExitObservationV2>,
    publisher_pid: i32,
    harness_process_group_id: i32,
    publisher_live_peer: Option<GeneralFailureLivePeerV2>,
    harness_live_peer: Option<GeneralFailureLivePeerV2>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PublisherUiTriggerOriginV2 {
    PublisherProcess,
    RunnerObservedControl,
}

#[derive(Debug, Clone)]
struct RunnerSecurityAgentAlertV2 {
    repetition: RepetitionV2,
    arm: String,
    evidence: SecurityAgentArmEvidenceV2,
}

impl std::fmt::Display for RunnerSecurityAgentAlertV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "runner observed a terminal SecurityAgent ALERT in {} for repetition {}",
            self.arm,
            self.repetition.ordinal()
        )
    }
}

impl std::error::Error for RunnerSecurityAgentAlertV2 {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RootOperationLivePeersV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    publisher: GeneralFailureLivePeerV2,
    harness: GeneralFailureLivePeerV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RootOperationUiStopCursorV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    operation: String,
    active_repetition: Option<u8>,
    active_scope_id: Option<String>,
    creation_receipt_sha256: Option<String>,
    journal_absent_at_alert: Option<bool>,
    journal_observation_sha256: Option<String>,
    triggering_securityagent_evidence_sha256: String,
    triggering_securityagent_evidence: SecurityAgentArmEvidenceV2,
    live_peers: Vec<GeneralFailureLivePeerV2>,
    terminal_no_normal_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RootOperationUiTerminalReceiptV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    cursor_sha256: String,
    live_peer_termination_sha256: String,
    rollback_result_sha256: String,
    repetitions: Vec<FailedRepetitionRestorationV2>,
    creator_marker_restoration_sha256: String,
    finalizer_cleanup_sha256: Option<String>,
    root_install_claims_binding_sha256: String,
    claims_retained: bool,
    admin_cleanup_authorized: bool,
    terminal_no_normal_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RootOperationLivePeerTerminationV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    cursor_sha256: String,
    peers_sha256: String,
    sigterm_sent: Vec<bool>,
    sigkill_sent: Vec<bool>,
    all_exact_peers_absent_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TerminalAdminCleanupAuthorizationV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    terminal_outcome_kind: String,
    authorization_path: String,
    runner_root_path: String,
    runner_root_stable_identity_sha256: String,
    terminal_receipt_path: String,
    terminal_receipt_sha256: String,
    terminal_receipt_physical_identity_sha256: String,
    runner_child_inventory: Vec<TerminalRunnerChildIdentityV2>,
    runner_child_inventory_sha256: String,
    runner_child_inventory_cardinality: u32,
    runner_child_archive_total_bytes: u64,
    runner_child_archive_max_bytes: u64,
    surrogate_restoration_sha256: String,
    finalizer_cleanup_sha256: String,
    creator_marker_restoration_sha256: String,
    root_install_claims_binding_sha256: String,
    root_install_claims_retention_observation_sha256: String,
    claims_retained: bool,
    admin_cleanup_authorized: bool,
    terminal_no_normal_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TerminalRunnerChildIdentityV2 {
    name: String,
    canonical_byte_length: u64,
    canonical_sha256: String,
    physical_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreatorTerminalCleanupCursorV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    creator_rollback_receipt_sha256: String,
    global_pre_effect_packet_sha256: Option<String>,
    root_install_claims_binding_sha256: String,
    cleanup_may_begin: bool,
    terminal_no_normal_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CreatorTerminalCleanupReceiptV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    cursor_sha256: String,
    creator_rollback_receipt_sha256: String,
    surrogate_restoration_sha256: String,
    finalizer_cleanup_sha256: String,
    creator_marker_restoration_sha256: String,
    installed_packets_absent: bool,
    root_install_claims_binding_sha256: String,
    root_install_claims_retention_observation_sha256: String,
    claims_retained: bool,
    admin_cleanup_authorized: bool,
    terminal_no_normal_resume: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct TerminalRunnerRootStableIdentityV2<'a> {
    path: &'a str,
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublisherUiRollbackPreparedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    ui_stop_cursor_sha256: String,
    rollback_marker_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublisherUiRollbackProcessV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    ui_stop_cursor_sha256: String,
    rollback_prepared_sha256: String,
    pid: i32,
    executable_path: String,
    effective_uid: u32,
    effective_gid: u32,
    canonical_account: String,
    process_start_identity_sha256: String,
    executable_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct HarnessTerminationObservationV2 {
    schema_owner: String,
    schema_version: u32,
    process_group_id: i32,
    sigterm_sent: bool,
    sigkill_sent: bool,
    exit_code: Option<i32>,
    terminating_signal: Option<i32>,
    process_group_absent_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublisherUiTerminalReceiptV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    ui_stop_cursor_sha256: String,
    harness_termination_sha256: String,
    publisher_termination_sha256: String,
    triggering_securityagent_evidence_sha256: String,
    rollback_marker_sha256: String,
    rollback_prepared_sha256: String,
    rollback_process_attestation_sha256: String,
    emergency_rollback_receipt_sha256: String,
    securityagent_observation_sha256: String,
    restoration_sha256: String,
    terminal_no_retry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ChildExitObservationV2 {
    success: bool,
    exit_code: Option<i32>,
    terminating_signal: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FailedRepetitionRestorationKindV2 {
    Untouched,
    EmergencyRollback,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FailedRepetitionRestorationV2 {
    repetition: u8,
    scope_id: String,
    kind: FailedRepetitionRestorationKindV2,
    progress_sha256: Option<String>,
    restoration_evidence_sha256: Option<String>,
    signing_seed_identity_sha256: Option<String>,
    signing_seed_absent_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureRestorationReceiptV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    publisher_exit: Option<ChildExitObservationV2>,
    harness_exit: Option<ChildExitObservationV2>,
    rollback_result_sha256: String,
    repetitions: Vec<FailedRepetitionRestorationV2>,
    alternate_path_absent: bool,
    installed_packets_absent: bool,
    securityagent_observation_sha256: String,
    terminal_no_resume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureRestorationCursorV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    trigger: GeneralFailureTriggerV2,
    publisher_exit: Option<ChildExitObservationV2>,
    harness_exit: Option<ChildExitObservationV2>,
    live_peer: Option<GeneralFailureLivePeerV2>,
    root_operation_ui_stop_cursor_sha256: Option<String>,
    terminal_no_resume: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum GeneralFailureTriggerV2 {
    PublisherClosedFailure,
    HarnessClosedFailure,
    RootOperationSecurityAgentAlert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum GeneralFailureLivePeerKindV2 {
    PublisherProcess,
    HarnessProcessGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureLivePeerV2 {
    kind: GeneralFailureLivePeerKindV2,
    pid: i32,
    process_group_id: Option<i32>,
    process_start_identity_sha256: String,
    executable_path: String,
    executable_identity_sha256: String,
    effective_uid: u32,
    supplementary_groups: SupplementaryGroupAttestationV2,
    process_group_members: Vec<GeneralFailureGroupMemberV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureGroupMemberV2 {
    pid: i32,
    effective_uid: u32,
    supplementary_groups: SupplementaryGroupAttestationV2,
    process_start_identity_sha256: String,
    executable_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailurePeerTerminationV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    cursor_sha256: String,
    live_peer_sha256: Option<String>,
    sigterm_sent: bool,
    sigkill_sent: bool,
    exit_observation: Option<ChildExitObservationV2>,
    recovered_without_child_handle: bool,
    peer_absent_after: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum GeneralFailureRollbackDispositionV2 {
    Completed,
    AlreadyEmergencyRestored,
    RollbackPrecreation,
    RollbackCreated,
    AcceptedAuthorityAmbiguous,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureRollbackPlanEntryV2 {
    repetition: u8,
    scope_id: String,
    progress_sha256: Option<String>,
    progress_stage: Option<PublisherStageV2>,
    creation_receipt_sha256: Option<String>,
    accepted_journal_generation: Option<u64>,
    accepted_journal_head_sha256: Option<String>,
    accepted_request_digest: Option<String>,
    disposition: GeneralFailureRollbackDispositionV2,
    rollback_marker_artifact_filename: Option<String>,
    rollback_marker_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureRollbackPreparedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    cursor_sha256: String,
    peer_termination_sha256: String,
    plan: Vec<GeneralFailureRollbackPlanEntryV2>,
    plan_sha256: String,
    accepted_authority_ambiguity: bool,
    rollback_publisher_invocation_required: bool,
    terminal_no_normal_retry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureRollbackInvokedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    prepared_sha256: String,
    invocation_ordinal: u8,
    publisher_executable_identity_sha256: String,
    invocation_may_begin: bool,
    normal_stage_retry_forbidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureRollbackResultV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    prepared_sha256: String,
    invoked_sha256: Option<String>,
    publisher_exit: Option<ChildExitObservationV2>,
    emergency_receipt_sha256: Vec<String>,
    reconstructed_after_invoked: bool,
    exact_restoration_complete: bool,
    accepted_authority_ambiguity: bool,
    terminal_no_reinvoke: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct GeneralFailureAcceptedRejoinV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    failure_cursor_sha256: String,
    peer_termination_sha256: String,
    accepted: Vec<AcceptedJournalAuthorityV2>,
    same_digest_request_sha256: Vec<String>,
    exact_same_digest_rejoin_authorized: bool,
    rollback_forbidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FinalizerBootstrapPreparedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    launchd_label: String,
    launchd_plist_path: String,
    launchd_plist_sha256: String,
    endpoint_path: String,
    global_pre_effect_sha256: String,
    creator_route_receipt_set_sha256: String,
    service_absent_before: bool,
    endpoint_absent_before: bool,
    invocation_may_begin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FinalizerBootstrapResultV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    prepared_sha256: String,
    exit_status: i32,
    stdout: BoundedRawStreamEvidenceV2,
    stderr: BoundedRawStreamEvidenceV2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FinalizerServiceObservationV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    prepared_sha256: String,
    bootstrap_result_sha256: String,
    launchd_label: String,
    print_exit_status: i32,
    print_stdout: BoundedRawStreamEvidenceV2,
    print_stderr: BoundedRawStreamEvidenceV2,
    endpoint_path: String,
    endpoint_owner_uid: u32,
    endpoint_group_gid: u32,
    endpoint_mode: u32,
    endpoint_device: u64,
    endpoint_inode: u64,
    endpoint_link_count: u64,
    endpoint_is_socket: bool,
    service_loaded_and_endpoint_exact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootstrapRecoveryDecisionV2 {
    Invoke,
    ObserveDurableResult,
    AcceptObserved,
    AmbiguousNoReinvoke,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FinalizerBootoutPreparedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    launchd_label: String,
    service_observation_sha256: String,
    native_evidence_export_sha256: String,
    acknowledgement_sha256: String,
    invocation_may_begin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct FinalizerBootoutResultV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    prepared_sha256: String,
    exit_status: i32,
    stdout: BoundedRawStreamEvidenceV2,
    stderr: BoundedRawStreamEvidenceV2,
}

struct CleanupNativeOutcomeV2 {
    bootout: FinalizerBootoutResultV2,
    print_exit_status: i32,
    print_stdout: BoundedRawStreamEvidenceV2,
    print_stderr: BoundedRawStreamEvidenceV2,
    endpoint_absence_observation_sha256: String,
    coordinator_inbox_observation_sha256: String,
    journal_absence_observation_sha256: Vec<String>,
    capability_absence_observation_sha256: Vec<String>,
    latch_absence_observation_sha256: Vec<String>,
    journal_root_lock_absence_observation_sha256: String,
    journal_root_absence_observation_sha256: String,
    root_install_claims_retention_observation_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum NativeCleanupObjectKindV2 {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NativeCleanupObjectV2 {
    ordinal: u32,
    path: String,
    kind: NativeCleanupObjectKindV2,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    canonical_sha256: Option<String>,
    physical_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NativeEvidenceCleanupPlanV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    native_evidence_export_sha256: String,
    native_evidence_export_physical_identity_sha256: String,
    acknowledgement_sha256: String,
    acknowledgement_physical_identity_sha256: String,
    global_pre_effect_packet_sha256: String,
    journal_root_activation_identity_sha256: String,
    journal_root_lock_identity_sha256: String,
    objects: Vec<NativeCleanupObjectV2>,
    object_set_sha256: String,
    deletion_may_begin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NativeEvidenceCleanupStepV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    plan_sha256: String,
    ordinal: u32,
    object_sha256: String,
    exact_absence_observed: bool,
    parent_fsynced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TerminalFinalizerFailureCleanupReceiptV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    failure_sha256: String,
    launchd_service_was_loaded: bool,
    launchctl_bootout: Option<FinalizerBootoutResultV2>,
    launchctl_print_exit_status: i32,
    launchctl_print_stdout: BoundedRawStreamEvidenceV2,
    launchctl_print_stderr: BoundedRawStreamEvidenceV2,
    endpoint_absent: bool,
    finalizer_process_absent_sha256: String,
    journal_root_absent: bool,
    journal_root_preserved_for_unacknowledged_evidence: bool,
    journal_root_disposition_observation_sha256: String,
    securityagent_report: SecurityAgentRawReportEvidenceV2,
}

struct TerminalJournalRootDispositionV2 {
    absent: bool,
    preserved_for_unacknowledged_evidence: bool,
    observation_sha256: String,
}

#[derive(Debug)]
struct StoppedPeerProcessV2 {
    pid: i32,
    effective_uid: u32,
    effective_gid: u32,
    supplementary_groups: SupplementaryGroupAttestationV2,
    canonical_account: String,
    process_start_identity_sha256: String,
    executable_identity_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PeerNativeArmV2 {
    DynamicLibraryInjection,
    NobodyOwnerAuthority {
        operation: NobodyOwnerAuthorityOperationV2,
    },
    PeerSubstitution {
        control: PeerSubstitutionControlV2,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PeerNativeArmPreparedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    sequence_ordinal: u8,
    arm: PeerNativeArmV2,
    request_digest: String,
    ready_marker_sha256: String,
    creation_receipt_sha256: String,
    invocation_requires_durable_invoked_cursor: bool,
    missing_receipt_after_invoked_is_ambiguous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PeerNativeArmInvokedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    sequence_ordinal: u8,
    arm: PeerNativeArmV2,
    prepared_sha256: String,
    before_observation_sha256: String,
    accepted_journal_absent_before: bool,
    invocation_may_begin: bool,
    blind_reinvoke_forbidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PeerNativeArmObservedV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    sequence_ordinal: u8,
    arm: PeerNativeArmV2,
    invoked_sha256: String,
    receipt_sha256: String,
    exact_observed_no_reinvoke: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeerNativeArmRecoveryDecisionV2 {
    Invoke,
    ReconstructObserved,
    AcceptObserved,
    AmbiguousNoReinvoke,
}

struct PeerNativeArmBeforeV2<'a> {
    observation_sha256: &'a str,
    accepted_journal_absent: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ProcessStartJoinV2 {
    pid: i32,
    seconds: u64,
    microseconds: u64,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RunnerRestorationReceiptV2 {
    schema_owner: &'static str,
    schema_version: u32,
    experiment_id: &'static str,
    creator_complete: bool,
    publisher_restart_count: u8,
    publisher_exit_success: bool,
    harness_exit_success: bool,
    alternate_coordinator_absent: bool,
    installed_packets_absent: bool,
    prepared_input_sha256: String,
    candidate_identity_sha256: String,
    peer_identity_sha256: String,
    securityagent_baseline_sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct AlternatePathRemovalObservationV2 {
    schema_owner: &'static str,
    schema_version: u32,
    path: &'static str,
    before_identity_sha256: String,
    before_physical_identity_sha256: String,
    exact_path_absent_after: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RunnerFilePhysicalIdentityV2 {
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    link_count: u64,
    size: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RunnerSelfObservationV2 {
    schema_owner: &'static str,
    schema_version: u32,
    executable_identity: ExecutableIdentityV2,
    signing_posture: CodeSigningPostureV2,
    effective_uid: u32,
    effective_gid: u32,
    canonical_account: String,
    pid: i32,
    process_start_identity_sha256: String,
    environment_empty: bool,
    stdin_is_dev_null: bool,
}

struct FrozenRunnerInputs {
    candidate: CandidateIdentityPacketV2,
    prepared: PublisherPreparedInputV2,
    peer: PeerControlIdentityPacketV2,
    creator: FrozenCode<'static>,
    wrong: FrozenCode<'static>,
    publisher: FrozenCode<'static>,
    harness: FrozenCode<'static>,
    creator_identity: ExecutableIdentityV2,
    creator_signing_posture: CodeSigningPostureV2,
    wrong_identity: ExecutableIdentityV2,
    wrong_signing_posture: CodeSigningPostureV2,
    publisher_signing_posture: CodeSigningPostureV2,
    observer_identity: ExecutableIdentityV2,
    observer_signing_posture: CodeSigningPostureV2,
    runner_identity_sha256: String,
    runner_process_attestation_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublisherStartupAttestationV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    purpose: String,
    startup_ordinal: u8,
    pid: i32,
    effective_uid: u32,
    effective_gid: u32,
    supplementary_groups: SupplementaryGroupAttestationV2,
    canonical_account: String,
    process_start_identity_sha256: String,
    executable_identity_sha256: String,
    securityagent_report: SecurityAgentRawReportEvidenceV2,
    exact_post_denial_sigstop_observed: bool,
    resume_authorized: bool,
}

struct ActivationMembraneGuardV2 {
    descriptor: OwnedFd,
    identity_sha256: String,
    root_identity_sha256: String,
    journal_root_lock_identity: RootInstallPhysicalIdentityV2,
    journal_root_lock_identity_sha256: String,
    root_absent_cleanup_recovery: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ActivationMembraneIdentityV2<'a> {
    path: &'a str,
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    link_count: u64,
    exclusive_lock_held: bool,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ActivationMembraneRootIdentityV2<'a> {
    path: &'a str,
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    link_count: u64,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RunnerDisposableBaselineV2<'a> {
    schema_owner: &'static str,
    schema_version: u32,
    experiment_id: &'static str,
    runner_self_observation_sha256: String,
    owner_account_and_setid_probe_sha256: String,
    owner_secaccess_readback_sha256: String,
    journal_root_lock_identity_sha256: String,
    creator_identity: &'a ExecutableIdentityV2,
    creator_signing_posture: &'a CodeSigningPostureV2,
    wrong_identity: &'a ExecutableIdentityV2,
    wrong_signing_posture: &'a CodeSigningPostureV2,
    publisher_identity: &'a ExecutableIdentityV2,
    publisher_signing_posture: &'a CodeSigningPostureV2,
    observer_identity: &'a ExecutableIdentityV2,
    observer_signing_posture: &'a CodeSigningPostureV2,
    creator_marker_root: &'static str,
    creator_marker_path: &'static str,
    system_keychain_path: &'static str,
    creator_scopes: [&'static str; 2],
    finalizer_scopes: [&'static str; 2],
}

pub fn run_fixed_experiment() -> Result<()> {
    verify_runner_process_surface()?;
    let candidate_freeze_authority = load_candidate_freeze_authority()?;
    let root_install_claims = load_root_install_claims(&candidate_freeze_authority.manifest)?;
    install_candidate_freeze_authority(candidate_freeze_authority.manifest.clone())?;
    let activation_membrane = acquire_activation_membrane_exclusive()?;
    if activation_membrane.root_absent_cleanup_recovery
        && read_staged_native_cleanup_recovery_documents()?.is_some()
    {
        return resume_staged_native_cleanup_without_runner_root();
    }
    // The alert cursor and raw trigger must have one durable private destination before the first
    // Security call.  This creates only the exact precommitted root and performs no Security or
    // target operation.
    prepare_runner_root()?;
    // This is deliberately the runner's first Security.framework call. All SecCode measurement,
    // in-memory legacy-access probing, and exact UI-fail Keychain observations follow it.  The
    // gap-free external observer begins before the call and its canonical raw report is durable.
    let (security, report_sha256, report_bytes) =
        observe_root_operation_with_raw(NonInteractiveSecurity::establish_first, |alert| {
            persist_root_operation_terminal_alert("runner-first-security-interaction-denial", alert)
        })?;
    persist_next_root_operation_securityagent_report(
        "runner-first-security-interaction-denial",
        &report_sha256,
        &report_bytes,
    )?;
    let mut security = security?;
    let (runner_identity, runner_signing_posture) = measure_runner_executable(
        EXPERIMENT_RUNNER_EXECUTABLE_PATH,
        EXPERIMENT_RUNNER_SIGNING_IDENTIFIER,
    )?;
    write_runner_receipt(
        "activation-journal-root-lock-observation.v2.json",
        &canonical_bytes_v2(&serde_json::json!({
            "schema_owner": "substrate.r3-macos-activation-journal-root-lock-observation",
            "schema_version": EXPERIMENT_VERSION_V2,
            "path": format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/journal-root.lock"),
            "identity_sha256": activation_membrane.journal_root_lock_identity_sha256,
        }))?,
    )?;
    let pid = unsafe { libc::getpid() };
    let process = process_info(pid)?;
    let runner_observation = RunnerSelfObservationV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-self-observation",
        schema_version: 2,
        executable_identity: runner_identity,
        signing_posture: runner_signing_posture,
        effective_uid: process.uid,
        effective_gid: process.gid,
        canonical_account: canonical_account(process.uid)?.0,
        pid,
        process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
            pid,
            seconds: process.start_seconds,
            microseconds: process.start_microseconds,
        })?,
        environment_empty: std::env::vars_os().next().is_none(),
        stdin_is_dev_null: true,
    };
    let runner_process_attestation_sha256 = document_sha256_v2(&runner_observation)?;
    write_runner_receipt(
        "runner-self-observation.v2.json",
        &canonical_bytes_v2(&runner_observation)?,
    )?;
    let owner_probe = probe_nonmatch_owner_candidate()?;
    write_runner_receipt(
        "legacy-access-nonmatch-owner-probe.v3.json",
        &canonical_bytes_v2(&owner_probe)?,
    )?;
    let inputs = build_and_install_frozen_inputs(
        &runner_observation.executable_identity,
        &runner_observation.signing_posture,
        &runner_process_attestation_sha256,
    )?;
    recover_root_operation_ui_terminal_stop_if_present(
        &inputs,
        &root_install_claims,
        &activation_membrane,
    )?;
    if activation_membrane.root_absent_cleanup_recovery {
        return recover_acknowledged_native_cleanup_after_root_absence(
            &inputs,
            &root_install_claims,
            &activation_membrane,
        );
    }
    let owner_access_probe = probe_nonmatch_owner_access_in_memory()?;
    write_runner_receipt(
        "legacy-access-nonmatch-owner-readback.v2.json",
        &canonical_bytes_v2(&owner_access_probe)?,
    )?;
    prepare_creator_marker()?;
    // The globally durable pre-effect proof is installed immediately before this release. Until
    // its shared schema lands, this call remains an explicit compile-time integration stop rather
    // than allowing creator native effects under a partial packet.
    let global_pre_effect = install_and_validate_global_pre_effect_proof(
        &mut security,
        &inputs,
        &runner_observation,
        &owner_probe,
        &owner_access_probe,
        &activation_membrane,
        &candidate_freeze_authority,
        &root_install_claims,
    )?;
    release_activation_membrane_exclusive(&activation_membrane)?;
    if let Err(error) = run_creator_arms(&inputs, &global_pre_effect) {
        return match complete_creator_terminal_failure(
            &inputs,
            Some(&global_pre_effect),
            &activation_membrane,
            &root_install_claims,
        ) {
            Ok(_) => Err(error).context(
                "creator route stopped after exact rollback and terminal admin cleanup authorization",
            ),
            Err(cleanup) => Err(error).context(format!(
                "creator terminal cleanup also stopped without authorization: {cleanup:#}"
            )),
        };
    }
    let finalizer_run = (|| -> Result<()> {
        let service_observation =
            ensure_finalizer_service_bootstrapped(&inputs, &global_pre_effect)?;
        recover_publisher_ui_terminal_stop_if_present(&inputs)?;
        let _accepted_same_digest_rejoin = recover_general_failure_restoration_if_present(&inputs)?;

        let mut publisher = spawn_publisher(&inputs)?;
        let mut harness = spawn_harness(&inputs)?;
        persist_root_operation_live_peers(&publisher, &harness, &inputs)?;
        let mut peer_sets: [Option<PeerControlSetReceiptV2>; 2] = std::array::from_fn(|_| None);
        let mut global_peer_restoration_published = false;
        let (publisher_exit, harness_exit) = 'experiment: loop {
            for repetition in RepetitionV2::ALL {
                let index = usize::from(repetition.ordinal() - 1);
                if peer_sets[index].is_none() && peer_ready_present(repetition)? {
                    match load_or_run_peer_controls(repetition, &inputs, &mut security) {
                        Ok(receipt) => peer_sets[index] = Some(receipt),
                        Err(error) => {
                            if let Some(alert) = error.downcast_ref::<RunnerSecurityAgentAlertV2>()
                            {
                                return terminal_runner_securityagent_alert(
                                    &inputs,
                                    alert,
                                    &mut publisher,
                                    &mut harness,
                                );
                            }
                            return Err(error);
                        }
                    }
                }
            }
            if !global_peer_restoration_published
                && peer_sets.iter().all(Option::is_some)
                && both_complete_responses_present()?
            {
                publish_global_peer_restoration(&inputs, &peer_sets)?;
                global_peer_restoration_published = true;
            }
            let publisher_status = publisher.try_wait()?;
            let harness_status = harness.try_wait()?;
            if publisher_status.as_ref().is_some_and(|status| {
                publisher_exit_disposition(status.success(), status.code())
                    == PublisherExitDispositionV2::TerminalSecurityAgentAlert
            }) {
                let active = active_publisher_repetition()?;
                let evidence = load_publisher_terminal_alert_evidence(active)?;
                let harness_exit_observation = harness_status.as_ref().map(child_exit_observation);
                let harness_live_peer = if harness_exit_observation.is_none() {
                    Some(observe_general_failure_live_peer(
                        &harness,
                        GeneralFailureLivePeerKindV2::HarnessProcessGroup,
                        &inputs.peer.harness_identity,
                        DISPOSABLE_HARNESS_UID_V2,
                    )?)
                } else {
                    None
                };
                // Cursor first: a runner crash from this point onward can never fall back into the
                // normal spawn/restart path on recovery.
                persist_publisher_ui_stop_cursor(
                    active,
                    PublisherUiTriggerOriginV2::PublisherProcess,
                    &evidence,
                    publisher_status.as_ref().map(child_exit_observation),
                    harness_exit_observation,
                    i32::try_from(publisher.id())?,
                    i32::try_from(harness.id())?,
                    None,
                    harness_live_peer,
                )?;
                let publisher_termination = terminated_child_observation(
                    &publisher,
                    publisher_status
                        .as_ref()
                        .expect("exit86 predicate established publisher status"),
                )?;
                write_runner_receipt(
                    PUBLISHER_UI_PUBLISHER_TERMINATION_NAME,
                    &canonical_bytes_v2(&publisher_termination)?,
                )?;
                let harness_termination = terminate_harness_process_group(&mut harness)?;
                write_runner_receipt(
                    PUBLISHER_UI_HARNESS_TERMINATION_NAME,
                    &canonical_bytes_v2(&harness_termination)?,
                )?;
                return complete_publisher_ui_terminal_rollback(
                    &inputs,
                    &harness_termination,
                    &publisher_termination,
                );
            }
            if let Some(harness_status) = harness_status {
                if !harness_status.success() {
                    return terminal_general_failure_from_harness(
                        &inputs,
                        harness_status,
                        &mut publisher,
                    );
                }
                let publisher_status = if let Some(status) = publisher_status {
                    status
                } else {
                    publisher.wait()?
                };
                break (publisher_status, harness_status);
            }
            if let Some(status) = publisher_status {
                match publisher_exit_disposition(status.success(), status.code()) {
                    PublisherExitDispositionV2::Completed => {
                        // A successful publisher may finish just before the harness consumes its last
                        // immutable output. Keep polling the harness without another publisher process.
                        loop {
                            if let Some(harness_status) = harness.try_wait()? {
                                break 'experiment (status, harness_status);
                            }
                            thread::sleep(POLL);
                        }
                    }
                    PublisherExitDispositionV2::TerminalSecurityAgentAlert => unreachable!(
                        "exit86 is handled before every harness-exit and publisher-restart branch"
                    ),
                    PublisherExitDispositionV2::TerminalClosedFailure => {
                        return terminal_general_failure_from_publisher(
                            &inputs,
                            status,
                            &mut harness,
                        );
                    }
                }
            }
            thread::sleep(POLL);
        };

        let publisher_success = publisher_exit.success();
        let harness_success = harness_exit.success();
        if publisher_success
            && harness_success
            && !global_peer_restoration_published
            && peer_sets.iter().all(Option::is_some)
            && both_complete_responses_present()?
        {
            publish_global_peer_restoration(&inputs, &peer_sets)?;
            global_peer_restoration_published = true;
        }
        if !publisher_success || !harness_success {
            bail!("child failure escaped its cursor-first terminal rollback route")
        }
        if !global_peer_restoration_published {
            bail!("sealed experiment ended before global alternate-path restoration")
        }
        let prepared_bytes = canonical_bytes_v2(&inputs.prepared)?;
        let candidate_bytes = canonical_bytes_v2(&inputs.candidate)?;
        let peer_bytes = canonical_bytes_v2(&inputs.peer)?;
        remove_exact_installed_file(
            Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2),
            Some(&prepared_bytes),
            None,
        )?;
        remove_exact_installed_file(
            Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2),
            Some(&candidate_bytes),
            None,
        )?;
        remove_exact_installed_file(
            Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
            Some(&peer_bytes),
            None,
        )?;
        let receipt = RunnerRestorationReceiptV2 {
            schema_owner: "substrate.r3-macos-disposable-experiment-runner-restoration",
            schema_version: 2,
            experiment_id: EXPERIMENT_ID_V2,
            creator_complete: MarkerRoot::open()?.read_state()? == MarkerState::Complete,
            publisher_restart_count: 0,
            publisher_exit_success: publisher_success,
            harness_exit_success: harness_success,
            alternate_coordinator_absent: !path_present(Path::new(ALTERNATE_COORDINATOR_PATH_V2))?,
            installed_packets_absent: !path_present(Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2))?
                && !path_present(Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2))?
                && !path_present(Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2))?,
            prepared_input_sha256: sha256_hex_v2(&prepared_bytes),
            candidate_identity_sha256: sha256_hex_v2(&candidate_bytes),
            peer_identity_sha256: sha256_hex_v2(&peer_bytes),
            securityagent_baseline_sha256: inputs.prepared.securityagent_baseline_sha256.clone(),
        };
        let bytes = canonical_bytes_v2(&receipt)?;
        write_runner_receipt("restoration.v2.json", &bytes)?;
        publish_native_evidence_and_cleanup(
            &global_pre_effect,
            &service_observation,
            &peer_sets,
            &activation_membrane,
        )?;
        println!("{}", String::from_utf8(bytes)?);
        if !(publisher_success && harness_success) {
            bail!("sealed experiment stopped after exact emergency restoration")
        }
        Ok(())
    })();
    match finalizer_run {
        Ok(()) => Ok(()),
        Err(error) => {
            // Acceptance transfers authority to the launchd finalizer.  Classify the authenticated
            // journals before any bootout or endpoint removal: a same-digest restart must retain
            // the exact service, endpoint, journal, and signer state needed by engine rejoin.
            match exact_accepted_authority_before_terminal_cleanup() {
                Ok(authority) if !authority.is_empty() => Err(error).context(format!(
                    "authenticated FinalizerAccepted authority is preserved for same-digest rejoin: {}",
                    document_sha256_v2(&authority)?
                )),
                Err(classification) => Err(error).context(format!(
                    "terminal cleanup refused because finalizer authority could not be classified without mutation: {classification:#}"
                )),
                Ok(_) => match terminal_finalizer_cleanup_receipt_sha256(
                    Some(&global_pre_effect),
                    &activation_membrane,
                    &format!("{error:#}"),
                ) {
                    Ok(_) => {
                        persist_existing_terminal_admin_cleanup_authorization(
                            &inputs,
                            &root_install_claims,
                        )?;
                        Err(error).context("terminal finalizer failure cleanup completed")
                    }
                    Err(cleanup) => Err(error).context(format!(
                        "terminal finalizer failure cleanup also stopped: {cleanup:#}"
                    )),
                },
            }
        }
    }
}

fn exact_accepted_authority_before_terminal_cleanup() -> Result<Vec<AcceptedJournalAuthorityV2>> {
    let mut accepted = Vec::new();
    for repetition in RepetitionV2::ALL {
        if let Some(authority) = observe_general_failure_accepted_journal(repetition)? {
            accepted.push(authority);
        }
    }
    Ok(accepted)
}

fn verify_runner_process_surface() -> Result<()> {
    if std::env::args_os().count() != 1 {
        bail!("sealed root experiment runner rejects all arguments")
    }
    if std::env::vars_os().any(|(key, _)| key.as_bytes().starts_with(b"SUBSTRATE_")) {
        bail!("sealed root experiment runner rejects SUBSTRATE_* input")
    }
    if std::env::current_dir()? != Path::new("/")
        || std::env::current_exe()? != Path::new(EXPERIMENT_RUNNER_EXECUTABLE_PATH)
        || unsafe { libc::geteuid() } != 0
        || canonical_account(0)?.0 != "root"
    {
        bail!("sealed root experiment runner process identity changed")
    }
    let stdin = std::fs::metadata("/dev/fd/0")?;
    let null = std::fs::metadata("/dev/null")?;
    if stdin.dev() != null.dev() || stdin.ino() != null.ino() || stdin.rdev() != null.rdev() {
        bail!("sealed root experiment runner stdin is not /dev/null")
    }
    crate::experiment::clear_and_verify_environment()?;
    Ok(())
}

fn build_and_install_frozen_inputs(
    runner_identity: &ExecutableIdentityV2,
    runner_signing_posture: &CodeSigningPostureV2,
    runner_process_attestation_sha256: &str,
) -> Result<FrozenRunnerInputs> {
    require_non_placeholder_digest(capability_digest(), "capability digest")?;
    require_non_placeholder_digest(launch_plist_sha256(), "launchd plist digest")?;
    let creator = creator_code(CREATOR_EXECUTABLE_PATH, CREATOR_SIGNING_IDENTIFIER);
    let wrong = wrong_code(WRONG_IDENTITY_EXECUTABLE_PATH, WRONG_SIGNING_IDENTIFIER);
    let publisher = publisher_code(
        DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
        substrate_common::macos_retirement_v2::MAC_R3_DISPOSABLE_PUBLISHER_SIGNING_IDENTIFIER_V2,
    );
    let harness = harness_code(
        DISPOSABLE_HARNESS_PATH_V2,
        DISPOSABLE_HARNESS_SIGNING_IDENTIFIER_V2,
    );
    let nobody_owner_probe = nobody_owner_probe_code(
        NOBODY_OWNER_PROBE_PATH_V2,
        NOBODY_OWNER_PROBE_SIGNING_IDENTIFIER_V2,
    );
    let finalizer = finalizer_code(
        MAC_R3_FINALIZER_PATH_V2,
        MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
    );
    let coordinator = coordinator_code(
        MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
    );
    let peer_code = peer_probe_code(
        PEER_CODE_PROBE_PATH_V2,
        PEER_CODE_PROBE_SIGNING_IDENTIFIER_V2,
    );
    let observer = observer_code(SECURITYAGENT_OBSERVER_PATH_V2, OBSERVER_SIGNING_IDENTIFIER);
    let benign_injection_library = benign_injection_library_code(
        BENIGN_INJECTION_LIBRARY_PATH_V2,
        BENIGN_INJECTION_LIBRARY_SIGNING_IDENTIFIER_V2,
    );
    // This entire pass is Security.framework-free. All installed child roles, fixed pathnames,
    // byte hashes, and sizes must join the reviewed manifest before the first child SecCode query.
    for code in [
        creator,
        wrong,
        publisher,
        harness,
        finalizer,
        coordinator,
        peer_code,
        nobody_owner_probe,
        observer,
        benign_injection_library,
    ] {
        preflight_frozen_executable(code)?;
    }
    let (creator_identity, creator_signing_posture) =
        measure_frozen_executable_and_posture(creator)?;
    let (wrong_identity, wrong_signing_posture) = measure_frozen_executable_and_posture(wrong)?;
    let (publisher_identity, publisher_signing_posture) =
        measure_frozen_executable_and_posture(publisher)?;
    let (harness_identity, harness_signing_posture) =
        measure_frozen_executable_and_posture(harness)?;
    let (finalizer_identity, finalizer_signing_posture) =
        measure_frozen_executable_and_posture(finalizer)?;
    let (coordinator_identity, coordinator_signing_posture) =
        measure_frozen_executable_and_posture(coordinator)?;
    let (peer_code_identity, peer_code_signing_posture) =
        measure_frozen_executable_and_posture(peer_code)?;
    let (nobody_owner_probe_identity, nobody_owner_probe_signing_posture) =
        measure_frozen_executable_and_posture(nobody_owner_probe)?;
    let (observer_identity, observer_signing_posture) =
        measure_frozen_executable_and_posture(observer)?;
    let (benign_injection_library_identity, benign_injection_library_signing_posture) =
        measure_frozen_executable_and_posture(benign_injection_library)?;
    install_alternate_coordinator_copy(&coordinator_identity)?;
    let alternate_path = FrozenCode {
        path: ALTERNATE_COORDINATOR_PATH_V2,
        signing_identifier: MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
        role: CandidateFreezeArtifactRoleV2::AlternateCoordinatorExecutable,
    };
    preflight_frozen_executable(alternate_path)?;
    let (alternate_path_identity, alternate_path_signing_posture) =
        measure_frozen_executable_and_posture(alternate_path)?;
    if alternate_path_identity.executable_sha256 != coordinator_identity.executable_sha256
        || alternate_path_identity.physical_identity_sha256
            == coordinator_identity.physical_identity_sha256
    {
        bail!("alternate coordinator copy does not separate only physical path identity")
    }
    let plist_bytes = stable_read_file(Path::new(MAC_R3_FINALIZER_PLIST_PATH_V2), 0, None)?;
    if sha256_hex_v2(&plist_bytes) != launch_plist_sha256() {
        bail!("installed finalizer launch plist differs from its compiled SHA-256")
    }
    let launch_identity = LaunchIdentityV2 {
        launchd_label: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_owned(),
        launchd_plist_path: MAC_R3_FINALIZER_PLIST_PATH_V2.to_owned(),
        launchd_plist_sha256: launch_plist_sha256().to_owned(),
        endpoint: substrate_common::macos_retirement_v2::MAC_R3_FINALIZER_ENDPOINT_V2.to_owned(),
        endpoint_owner_uid: 0,
        endpoint_group_gid: 20,
        endpoint_mode: "0660".to_owned(),
        launch_socket_name: "Listener".to_owned(),
        finalizer_effective_uid: 0,
    };
    validate_launch_identity_v2(&launch_identity)?;
    let candidate = CandidateIdentityPacketV2 {
        schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        finalizer_identity,
        finalizer_signing_posture,
        coordinator_identity: coordinator_identity.clone(),
        coordinator_signing_posture: coordinator_signing_posture.clone(),
        launch_identity,
        capability_digest: capability_digest().to_owned(),
    };
    candidate.validate()?;
    let prepared = PublisherPreparedInputV2 {
        schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        publisher_identity: publisher_identity.clone(),
        publisher_identity_packet_sha256: document_sha256_v2(&publisher_identity)?,
        securityagent_baseline_sha256: observe_idle_baseline_sha256()?,
    };
    prepared.validate()?;
    let peer = PeerControlIdentityPacketV2 {
        schema_owner: PEER_CONTROL_RECEIPT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        harness_identity,
        harness_signing_posture,
        runner_identity: runner_identity.clone(),
        runner_signing_posture: runner_signing_posture.clone(),
        alternate_caller_identity: coordinator_identity,
        alternate_caller_signing_posture: coordinator_signing_posture,
        alternate_code_identity: peer_code_identity,
        alternate_code_signing_posture: peer_code_signing_posture,
        alternate_path_identity,
        alternate_path_signing_posture,
        nobody_owner_probe_identity,
        nobody_owner_probe_signing_posture,
        creator_identity: creator_identity.clone(),
        creator_signing_posture: creator_signing_posture.clone(),
        wrong_identity: wrong_identity.clone(),
        wrong_signing_posture: wrong_signing_posture.clone(),
        securityagent_observer_identity: observer_identity.clone(),
        securityagent_observer_signing_posture: observer_signing_posture.clone(),
        benign_injection_library_identity,
        benign_injection_library_signing_posture,
        expected_termination: vec![
            PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
            PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
            PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
        ],
    };
    peer.validate(&candidate)?;
    install_packet(
        Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2),
        &canonical_bytes_v2(&prepared)?,
    )?;
    install_packet(
        Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2),
        &canonical_bytes_v2(&candidate)?,
    )?;
    install_packet(
        Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
        &canonical_bytes_v2(&peer)?,
    )?;
    Ok(FrozenRunnerInputs {
        candidate,
        prepared,
        peer,
        creator,
        wrong,
        publisher,
        harness,
        creator_identity,
        creator_signing_posture,
        wrong_identity,
        wrong_signing_posture,
        publisher_signing_posture,
        observer_identity,
        observer_signing_posture,
        runner_identity_sha256: document_sha256_v2(runner_identity)?,
        runner_process_attestation_sha256: runner_process_attestation_sha256.to_owned(),
    })
}

fn frozen_code_for_identity<'a>(
    path: &'a str,
    identity: &'a ExecutableIdentityV2,
) -> Result<FrozenCode<'a>> {
    if path != identity.intended_path {
        bail!("requested executable path differs from its frozen identity")
    }
    let role = match path {
        MAC_R3_FINALIZER_PATH_V2 => CandidateFreezeArtifactRoleV2::FinalizerExecutable,
        MAC_R3_COORDINATOR_PATH_V2 => CandidateFreezeArtifactRoleV2::CoordinatorExecutable,
        DISPOSABLE_HARNESS_PATH_V2 => CandidateFreezeArtifactRoleV2::DisposableHarnessExecutable,
        PEER_CODE_PROBE_PATH_V2 => CandidateFreezeArtifactRoleV2::PeerCodeProbeExecutable,
        ALTERNATE_COORDINATOR_PATH_V2 => {
            CandidateFreezeArtifactRoleV2::AlternateCoordinatorExecutable
        }
        CREATOR_EXECUTABLE_PATH => CandidateFreezeArtifactRoleV2::CreatorExecutable,
        WRONG_IDENTITY_EXECUTABLE_PATH => CandidateFreezeArtifactRoleV2::WrongIdentityExecutable,
        DISPOSABLE_PUBLISHER_EXECUTABLE_PATH => {
            CandidateFreezeArtifactRoleV2::DisposablePublisherExecutable
        }
        EXPERIMENT_RUNNER_EXECUTABLE_PATH => {
            CandidateFreezeArtifactRoleV2::DisposableExperimentRunnerExecutable
        }
        NOBODY_OWNER_PROBE_PATH_V2 => CandidateFreezeArtifactRoleV2::NobodyOwnerProbeExecutable,
        SECURITYAGENT_OBSERVER_PATH_V2 => {
            CandidateFreezeArtifactRoleV2::SecurityAgentObserverExecutable
        }
        BENIGN_INJECTION_LIBRARY_PATH_V2 => CandidateFreezeArtifactRoleV2::BenignInjectionLibrary,
        _ => bail!("executable path is outside the closed candidate role set"),
    };
    Ok(FrozenCode {
        path,
        signing_identifier: &identity.signing_identifier,
        role,
    })
}

fn prepare_creator_marker() -> Result<()> {
    ensure_root_directory(Path::new(MARKER_ROOT))?;
    let marker = Path::new(MARKER_PATH);
    if !path_present(marker)? {
        write_fixed_file(
            marker,
            MarkerState::FirstQueryPrepared.marker(),
            0o600,
            0,
            0,
            libc::S_IFDIR | 0o700,
        )?;
    }
    MarkerRoot::open()?.read_state()?;
    Ok(())
}

fn run_creator_arms(
    inputs: &FrozenRunnerInputs,
    global_pre_effect: &GlobalPreEffectPacketV2,
) -> Result<()> {
    let marker = MarkerRoot::open()?;
    if marker.read_rollback()?.is_some() {
        return recover_creator_rollback(&marker, inputs);
    }
    recover_unobserved_creator_arm(&marker, inputs)?;
    loop {
        let before = marker.read_state()?;
        let completed = completed_creator_arm_count(before)?;
        let mut receipts = load_creator_native_receipts(completed)?;
        if before == MarkerState::Complete {
            let set = CreatorRouteReceiptSetV2 {
                schema_owner: GLOBAL_PRE_EFFECT_PACKET_OWNER_V2.to_owned(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_owned(),
                global_pre_effect_packet_sha256: document_sha256_v2(global_pre_effect)?,
                runner_process_attestation_sha256: inputs.runner_process_attestation_sha256.clone(),
                receipt_set_sha256: document_sha256_v2(&receipts)?,
                receipts,
            };
            set.validate(global_pre_effect, &inputs.peer)?;
            let bytes = canonical_bytes_v2(&set)?;
            write_runner_receipt("creator-route-receipts.v2.json", &bytes)?;
            write_global_external_root_output(PublisherArtifactV2::CreatorRouteReceiptSet, &bytes)?;
            return Ok(());
        }
        let (path, code) = if matches!(
            before,
            MarkerState::FirstWrongPrepared
                | MarkerState::FirstWrongInvoked
                | MarkerState::SecondWrongPrepared
                | MarkerState::SecondWrongInvoked
        ) {
            (WRONG_IDENTITY_EXECUTABLE_PATH, inputs.wrong)
        } else {
            (CREATOR_EXECUTABLE_PATH, inputs.creator)
        };
        let sequence_ordinal = creator_arm_sequence_ordinal(before)?;
        let global_ordinal = creator_arm_ordinal(before)?;
        let repetition = state_repetition(before).context("creator state has no repetition")?;
        let shared_repetition = repetition.ordinal();
        let arm_kind = creator_native_arm(before)?;
        let executable_identity = if path == WRONG_IDENTITY_EXECUTABLE_PATH {
            &inputs.wrong_identity
        } else {
            &inputs.creator_identity
        };
        measure_frozen_executable(code)?;
        let prepared_cursor = CreatorArmPreparedCursorV2 {
            schema_owner: GLOBAL_PRE_EFFECT_PACKET_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            global_ordinal,
            repetition: shared_repetition,
            state_before: marker_text(before),
            executable_path: path.to_owned(),
            executable_identity_sha256: document_sha256_v2(executable_identity)?,
            invocation_may_begin: true,
            missing_observation_requires_terminal_rollback: true,
        };
        write_runner_receipt(
            &format!("creator-arm-{global_ordinal:02}.prepared.cursor.v2.json"),
            &canonical_bytes_v2(&prepared_cursor)?,
        )?;
        let (observed, process_attestation_sha256) =
            observe_sealed_child_with(sealed_command(path, MARKER_ROOT, None, None)?, |pid| {
                attest_stopped_creator_process(
                    pid as i32,
                    shared_repetition,
                    sequence_ordinal,
                    arm_kind,
                    path,
                    executable_identity,
                )
            })?;
        let after = marker.read_state()?;
        let arm = CreatorArmObservationV2 {
            schema_owner: "substrate.r3-macos-disposable-runner-creator-arm",
            schema_version: 2,
            ordinal: global_ordinal,
            state_before: marker_text(before),
            state_after: marker_text(after),
            executable_path: path,
            exit_code: observed.status.code(),
            stdout_sha256: sha256_hex_v2(&observed.stdout),
            stderr_sha256: sha256_hex_v2(&observed.stderr),
            securityagent_report_sha256: observed.securityagent_report_sha256.clone(),
            unexpected_ui_observed: observed.unexpected_ui_observed,
        };
        let arm_bytes = canonical_bytes_v2(&arm)?;
        write_runner_receipt(
            &format!("creator-arm-{global_ordinal:02}.v2.json"),
            &arm_bytes,
        )?;
        if observed.unexpected_ui_observed || !observed.status.success() {
            let repetition =
                state_repetition(before).context("failed creator state has no scope")?;
            let failure_sha256 = if observed.unexpected_ui_observed {
                let raw = SecurityAgentRawReportEvidenceV2 {
                    raw_report_base64url: URL_SAFE_NO_PAD.encode(&observed.securityagent_report),
                    raw_report_sha256: observed.securityagent_report_sha256.clone(),
                    raw_report_byte_length: u64::try_from(observed.securityagent_report.len())?,
                };
                let evidence = single_securityagent_terminal_alert_evidence_v2(raw)?;
                let binding = CreatorTerminalAlertBindingV2 {
                    schema_owner:
                        "substrate.r3-macos-disposable-runner-creator-terminal-alert-binding",
                    schema_version: 2,
                    experiment_id: EXPERIMENT_ID_V2,
                    arm_observation_sha256: document_sha256_v2(&arm)?,
                    arm_evidence_sha256: document_sha256_v2(&evidence)?,
                    arm_evidence: &evidence,
                    terminal_no_normal_retry: true,
                };
                let binding_bytes = canonical_bytes_v2(&binding)?;
                write_runner_receipt(
                    &format!("creator-arm-{global_ordinal:02}.terminal-alert-binding.v2.json"),
                    &binding_bytes,
                )?;
                write_runner_receipt(
                    &format!("creator-arm-{global_ordinal:02}.terminal-alert.raw-report.v2.json"),
                    &observed.securityagent_report,
                )?;
                sha256_hex_v2(&binding_bytes)
            } else {
                document_sha256_v2(&arm)?
            };
            let original_failure = creator_failure_diagnostic(&observed);
            marker.prepare_rollback(repetition, &failure_sha256)?;
            observe_attested_creator_rollback(&marker, inputs).with_context(|| {
                format!("rollback after original creator failure: {original_failure}")
            })?;
            bail!(
                "original creator failure: {original_failure}; exact emergency rollback completed"
            )
        }
        if after == before {
            bail!("sealed creator arm succeeded without advancing its fixed marker")
        }
        let process_attestation_sha256 = process_attestation_sha256
            .context("successful creator arm lacks its stopped-process attestation")?;
        let typed_receipt_bytes = one_stdout_line(&observed.stdout)?;
        let typed_receipt: serde_json::Value = parse_canonical_v2(typed_receipt_bytes)?;
        validate_creator_typed_receipt(&typed_receipt, before, after, path)?;
        write_runner_receipt(
            &format!("creator-arm-{global_ordinal:02}.native-receipt.v2.json"),
            typed_receipt_bytes,
        )?;
        if observed.securityagent_report.is_empty()
            || sha256_hex_v2(&observed.securityagent_report) != observed.securityagent_report_sha256
        {
            bail!("creator arm lacks its exact raw canonical SecurityAgent observation")
        }
        parse_canonical_v2::<serde_json::Value>(&observed.securityagent_report)?;
        write_runner_receipt(
            &format!("creator-arm-{global_ordinal:02}.securityagent-observation.v2.json"),
            &observed.securityagent_report,
        )?;
        let shared_receipt = build_creator_native_arm_receipt(
            shared_repetition,
            sequence_ordinal,
            arm_kind,
            CreatorArmEvidenceV2 {
                before,
                after,
                executable_identity,
                process_attestation_sha256,
                native_receipt: typed_receipt_bytes,
                securityagent_report: &observed.securityagent_report,
            },
        )?;
        shared_receipt.validate(shared_repetition, &inputs.peer)?;
        let shared_bytes = canonical_bytes_v2(&shared_receipt)?;
        write_runner_receipt(
            &format!("creator-arm-{global_ordinal:02}.shared-receipt.v2.json"),
            &shared_bytes,
        )?;
        let observed_cursor = CreatorArmObservedCursorV2 {
            schema_owner: GLOBAL_PRE_EFFECT_PACKET_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            prepared_sha256: document_sha256_v2(&prepared_cursor)?,
            shared_receipt_sha256: document_sha256_v2(&shared_receipt)?,
            raw_securityagent_report_sha256: observed.securityagent_report_sha256,
            terminal_no_reinvoke: true,
        };
        write_runner_receipt(
            &format!("creator-arm-{global_ordinal:02}.observed.cursor.v2.json"),
            &canonical_bytes_v2(&observed_cursor)?,
        )?;
        receipts.push(shared_receipt);
    }
}

fn validate_creator_typed_receipt(
    value: &serde_json::Value,
    before: MarkerState,
    after: MarkerState,
    executable_path: &str,
) -> Result<()> {
    let object = value
        .as_object()
        .context("creator native receipt is not an object")?;
    let schema = object.get("schema").and_then(serde_json::Value::as_str);
    let final_present = object
        .get("final_present")
        .and_then(serde_json::Value::as_bool);
    if executable_path == WRONG_IDENTITY_EXECUTABLE_PATH {
        let (repetition, expected_after, repetition_name) = match before {
            MarkerState::FirstWrongPrepared => (
                FixedRepetitionV2::First,
                MarkerState::FirstFreshDeletePrepared,
                "first",
            ),
            MarkerState::SecondWrongPrepared => (
                FixedRepetitionV2::Second,
                MarkerState::SecondFreshDeletePrepared,
                "second",
            ),
            _ => bail!("wrong-identity typed receipt is outside a closed prepared route"),
        };
        if object.len() != 8
            || after != expected_after
            || schema != Some("substrate.r3-macos-signer-acl.wrong-identity-receipt.v4")
            || object.get("repetition").and_then(serde_json::Value::as_str) != Some(repetition_name)
            || object
                .get("creator_scope_id")
                .and_then(serde_json::Value::as_str)
                != Some(repetition.creator_scope())
            || object
                .get("executable_path")
                .and_then(serde_json::Value::as_str)
                != Some(WRONG_IDENTITY_EXECUTABLE_PATH)
            || object
                .get("marker_path")
                .and_then(serde_json::Value::as_str)
                != Some(MARKER_PATH)
            || object
                .get("process_interaction_disable_raw_os_status")
                .and_then(serde_json::Value::as_i64)
                != Some(0)
            || object
                .get("precommitted_expected_sign_raw_cferror_code")
                .and_then(serde_json::Value::as_i64)
                != Some(-25_293)
            || object
                .get("tag_scoped_private_key_sign_raw_cferror_code")
                .and_then(serde_json::Value::as_i64)
                != Some(-25_293)
        {
            bail!("wrong-identity typed receipt differs from the global native arm plan")
        }
        return Ok(());
    }
    if schema != Some("substrate.r3-macos-signer-acl.creator-route-receipt.v2")
        || object
            .get("creator_executable_path")
            .and_then(serde_json::Value::as_str)
            != Some(CREATOR_EXECUTABLE_PATH)
        || object
            .get("state_after")
            .and_then(serde_json::Value::as_str)
            != Some(marker_text(after).as_str())
    {
        bail!("creator typed receipt identity or durable successor changed")
    }
    let delete = object
        .get("exact_delete")
        .and_then(serde_json::Value::as_object);
    let creation = object
        .get("first_creation")
        .and_then(serde_json::Value::as_object);
    let disable = object
        .get("process_interaction_disable_raw_os_status")
        .and_then(serde_json::Value::as_i64);
    let (
        expected_phase,
        expected_delete,
        expected_class,
        expected_creation,
        expected_disable,
        present,
    ) = match before {
        MarkerState::FirstQueryPrepared | MarkerState::SecondQueryPrepared => (
            "query_ui_fail_create_then_delete",
            Some(0),
            Some("deleted_and_absent"),
            true,
            None,
            false,
        ),
        MarkerState::FirstFreshCreatePrepared | MarkerState::SecondFreshCreatePrepared => (
            "fresh_process_create_then_exit",
            None,
            None,
            true,
            None,
            true,
        ),
        MarkerState::FirstFreshDeletePrepared | MarkerState::SecondFreshDeletePrepared => (
            "fresh_process_first_call_disable_then_delete",
            Some(0),
            Some("deleted_and_absent"),
            false,
            Some(0),
            false,
        ),
        MarkerState::FirstAbsentRetryPrepared | MarkerState::SecondAbsentRetryPrepared => (
            "already_absent_retry",
            Some(-25_300),
            Some("already_absent"),
            false,
            None,
            false,
        ),
        _ => bail!("creator typed receipt is outside a closed prepared route"),
    };
    if object.get("phase").and_then(serde_json::Value::as_str) != Some(expected_phase)
        || final_present != Some(present)
        || creation.is_some() != expected_creation
        || disable != expected_disable
        || delete
            .and_then(|entry| entry.get("raw_os_status"))
            .and_then(serde_json::Value::as_i64)
            != expected_delete
        || delete
            .and_then(|entry| entry.get("classification"))
            .and_then(serde_json::Value::as_str)
            != expected_class
    {
        bail!("creator typed receipt raw result differs from the global native arm plan")
    }
    if creation.is_some_and(|entry| {
        entry
            .get("raw_cferror_code")
            .and_then(serde_json::Value::as_i64)
            != Some(0)
            || entry
                .get("present_after")
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            || entry
                .get("persisted_sensitive")
                .and_then(serde_json::Value::as_bool)
                .is_none()
            || entry
                .get("persisted_extractable")
                .and_then(serde_json::Value::as_bool)
                .is_none()
    }) {
        bail!("creator typed receipt creation result changed")
    }
    Ok(())
}

fn completed_creator_arm_count(state: MarkerState) -> Result<usize> {
    let index = MarkerState::all()
        .iter()
        .position(|candidate| *candidate == state)
        .context("creator marker is outside its closed sequence")?;
    Ok(index / 2)
}

fn creator_arm_sequence_ordinal(state: MarkerState) -> Result<u8> {
    let global = creator_arm_ordinal(state)?;
    Ok((global - 1) % 5 + 1)
}

fn creator_native_arm(state: MarkerState) -> Result<CreatorNativeArmV2> {
    match creator_arm_sequence_ordinal(state)? {
        1 => Ok(CreatorNativeArmV2::QueryUiFailCreateThenDelete),
        2 => Ok(CreatorNativeArmV2::FreshProcessCreateThenExit),
        3 => Ok(CreatorNativeArmV2::WrongIdentityProcessInteractionDeniedSign),
        4 => Ok(CreatorNativeArmV2::FreshProcessFirstCallDisableThenDelete),
        5 => Ok(CreatorNativeArmV2::AlreadyAbsentRetry),
        _ => unreachable!("closed creator arm ordinal is in 1..=5"),
    }
}

fn load_creator_native_receipts(count: usize) -> Result<Vec<CreatorNativeArmReceiptV2>> {
    let mut receipts = Vec::with_capacity(count);
    for ordinal in 1..=count {
        let path =
            Path::new(RUNNER_ROOT).join(format!("creator-arm-{ordinal:02}.shared-receipt.v2.json"));
        let bytes = stable_read_file(&path, 0, Some(libc::S_IFREG | 0o600))
            .with_context(|| format!("creator marker advanced without receipt {ordinal}"))?;
        receipts.push(parse_canonical_v2(&bytes)?);
    }
    if count < 10 {
        let unexpected = Path::new(RUNNER_ROOT).join(format!(
            "creator-arm-{:02}.shared-receipt.v2.json",
            count + 1
        ));
        if path_present(&unexpected)? {
            bail!("creator receipt exists ahead of its durable marker")
        }
    }
    Ok(receipts)
}

struct CreatorArmEvidenceV2<'a> {
    before: MarkerState,
    after: MarkerState,
    executable_identity: &'a ExecutableIdentityV2,
    process_attestation_sha256: String,
    native_receipt: &'a [u8],
    securityagent_report: &'a [u8],
}

fn build_creator_native_arm_receipt(
    repetition: u8,
    sequence_ordinal: u8,
    arm: CreatorNativeArmV2,
    evidence: CreatorArmEvidenceV2<'_>,
) -> Result<CreatorNativeArmReceiptV2> {
    use CreatorNativeClassificationV2 as C;
    use CreatorNativeOperationV2 as O;
    let (present_before, present_after, values) = match arm {
        CreatorNativeArmV2::QueryUiFailCreateThenDelete => (
            false,
            false,
            vec![
                (O::CreateProductEquivalentSigner, 0, C::CreatedAndPresent),
                (O::DeleteExactSigner, 0, C::DeletedAndAbsent),
            ],
        ),
        CreatorNativeArmV2::FreshProcessCreateThenExit => (
            false,
            true,
            vec![(O::CreateProductEquivalentSigner, 0, C::CreatedAndPresent)],
        ),
        CreatorNativeArmV2::WrongIdentityProcessInteractionDeniedSign => (
            true,
            true,
            vec![
                (O::DisableProcessInteractionFirst, 0, C::InteractionDisabled),
                (
                    O::SignTagScopedPrivateKey,
                    -25_293,
                    C::AuthFailed,
                ),
            ],
        ),
        CreatorNativeArmV2::WrongIdentityDelete
        | CreatorNativeArmV2::WrongIdentityProcessInteractionDeniedLookup => {
            bail!("historical wrong-identity arm is not valid in the current experiment")
        }
        CreatorNativeArmV2::FreshProcessFirstCallDisableThenDelete => (
            true,
            false,
            vec![
                (O::DisableProcessInteractionFirst, 0, C::InteractionDisabled),
                (O::DeleteExactSigner, 0, C::DeletedAndAbsent),
            ],
        ),
        CreatorNativeArmV2::AlreadyAbsentRetry => (
            false,
            false,
            vec![
                (O::DisableProcessInteractionFirst, 0, C::InteractionDisabled),
                (O::DeleteExactSigner, -25_300, C::AlreadyAbsent),
            ],
        ),
    };
    let operations = values
        .into_iter()
        .enumerate()
        .map(
            |(index, (operation, raw_status, classification))| CreatorNativeOperationReceiptV2 {
                sequence_ordinal: u8::try_from(index + 1).expect("creator operation count fits u8"),
                operation,
                raw_status,
                classification,
            },
        )
        .collect();
    let receipt = CreatorNativeArmReceiptV2 {
        schema_owner: GLOBAL_PRE_EFFECT_PACKET_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition,
        creator_scope_id: match repetition {
            1 => CREATOR_REPETITION_SCOPE_1,
            2 => CREATOR_REPETITION_SCOPE_2,
            _ => bail!("creator repetition is outside the two fixed scopes"),
        }
        .to_owned(),
        sequence_ordinal,
        arm,
        executable_identity_sha256: document_sha256_v2(evidence.executable_identity)?,
        process_attestation_sha256: evidence.process_attestation_sha256,
        marker_before: marker_text(evidence.before),
        marker_after: marker_text(evidence.after),
        target_present_before: present_before,
        target_present_after: present_after,
        operations,
        native_receipt_base64url: URL_SAFE_NO_PAD.encode(evidence.native_receipt),
        native_receipt_sha256: sha256_hex_v2(evidence.native_receipt),
        securityagent_report: securityagent_raw_evidence(evidence.securityagent_report)?,
    };
    Ok(receipt)
}

fn attest_stopped_creator_process(
    pid: i32,
    repetition: u8,
    sequence_ordinal: u8,
    arm: CreatorNativeArmV2,
    expected_path: &'static str,
    identity: &ExecutableIdentityV2,
) -> Result<String> {
    wait_for_sigstop(pid)?;
    let first = process_info(pid)?;
    if first.pid != pid as u32
        || first.uid != 0
        || first.gid != 0
        || pid_path(pid)? != Path::new(expected_path)
    {
        bail!("stopped creator process differs from its exact root exec identity")
    }
    let account = canonical_account(0)?.0;
    if account != "root" {
        bail!("stopped creator process canonical account is not root")
    }
    let measured = measure_frozen_executable(frozen_code_for_identity(expected_path, identity)?)?;
    if measured != *identity {
        bail!("stopped creator executable identity changed after exec")
    }
    let attestation = CreatorProcessAttestationV2 {
        schema_owner: "substrate.r3-macos-disposable-creator-process-attestation",
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2,
        repetition,
        sequence_ordinal,
        arm,
        pid,
        effective_uid: first.uid,
        effective_gid: first.gid,
        supplementary_groups: measured_child_supplementary_groups_v2(pid)?,
        canonical_account: account,
        process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
            pid,
            seconds: first.start_seconds,
            microseconds: first.start_microseconds,
        })?,
        executable_identity_sha256: document_sha256_v2(identity)?,
        executable_path: expected_path,
        argv_count: 0,
        environment_variable_count: 0,
        stdin_is_dev_null: true,
        cwd: MARKER_ROOT,
    };
    let digest = document_sha256_v2(&attestation)?;
    let global_ordinal = usize::from((repetition - 1) * 5 + sequence_ordinal);
    write_runner_receipt(
        &format!("creator-arm-{global_ordinal:02}.process-attestation.v2.json"),
        &canonical_bytes_v2(&attestation)?,
    )?;
    // SAFETY: the exact stopped child is resumed once after its attestation is durable.
    if unsafe { libc::kill(pid, libc::SIGCONT) } != 0 {
        return Err(std::io::Error::last_os_error()).context("resume exact creator process");
    }
    Ok(digest)
}

fn attest_stopped_creator_rollback_process(
    pid: i32,
    cursor: &CreatorRollbackMarkerV2,
    identity: &ExecutableIdentityV2,
) -> Result<String> {
    cursor.validate()?;
    if !matches!(
        cursor.phase,
        CreatorRollbackPhaseV2::Prepared | CreatorRollbackPhaseV2::Invoked
    ) {
        bail!("creator rollback process was spawned outside an open rollback phase")
    }
    wait_for_sigstop(pid)?;
    let first = process_info(pid)?;
    if first.pid != pid as u32
        || first.uid != 0
        || first.gid != 0
        || pid_path(pid)? != Path::new(CREATOR_EXECUTABLE_PATH)
    {
        bail!("stopped creator rollback differs from its exact root exec identity")
    }
    let account = canonical_account(0)?.0;
    if account != "root" {
        bail!("stopped creator rollback canonical account is not root")
    }
    let measured =
        measure_frozen_executable(frozen_code_for_identity(CREATOR_EXECUTABLE_PATH, identity)?)?;
    if measured != *identity {
        bail!("stopped creator rollback executable identity changed after exec")
    }
    let attestation = CreatorRollbackProcessAttestationV2 {
        schema_owner: "substrate.r3-macos-disposable-creator-rollback-process-attestation",
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2,
        repetition: cursor.repetition,
        rollback_phase_before: cursor.phase,
        failure_observation_sha256: cursor.failure_observation_sha256.clone(),
        pid,
        effective_uid: first.uid,
        effective_gid: first.gid,
        supplementary_groups: measured_child_supplementary_groups_v2(pid)?,
        canonical_account: account,
        process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
            pid,
            seconds: first.start_seconds,
            microseconds: first.start_microseconds,
        })?,
        executable_identity_sha256: document_sha256_v2(identity)?,
        executable_path: CREATOR_EXECUTABLE_PATH,
        argv_count: 0,
        environment_variable_count: 0,
        stdin_is_dev_null: true,
        cwd: MARKER_ROOT,
        resume_signal_authorized_count: 1,
    };
    let bytes = canonical_bytes_v2(&attestation)?;
    let digest = sha256_hex_v2(&bytes);
    write_runner_receipt(
        &format!("creator-emergency-rollback.process-attestation.{digest}.v2.json"),
        &bytes,
    )?;
    // SAFETY: the exact stopped rollback child is resumed once, and only after the complete
    // attestation above is durable.
    if unsafe { libc::kill(pid, libc::SIGCONT) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("resume exact creator rollback process");
    }
    Ok(digest)
}

fn securityagent_raw_evidence(bytes: &[u8]) -> Result<SecurityAgentRawReportEvidenceV2> {
    let evidence = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(bytes),
        raw_report_sha256: sha256_hex_v2(bytes),
        raw_report_byte_length: u64::try_from(bytes.len())
            .context("SecurityAgent report length exceeds u64")?,
    };
    evidence.validate()?;
    Ok(evidence)
}

fn persist_root_operation_securityagent_report(
    name: &str,
    expected_sha256: &str,
    bytes: &[u8],
) -> Result<SecurityAgentRawReportEvidenceV2> {
    let evidence = securityagent_raw_evidence(bytes)?;
    if evidence.raw_report_sha256 != expected_sha256 {
        bail!("root operation SecurityAgent report digest changed")
    }
    let canonical = canonical_bytes_v2(&evidence)?;
    let path = Path::new(RUNNER_ROOT).join(name);
    if let Some(existing) = stable_read_file_optional(&path, 0, Some(libc::S_IFREG | 0o600))? {
        if existing != canonical {
            bail!("root operation SecurityAgent report changed during recovery")
        }
    } else {
        write_runner_receipt(name, &canonical)?;
    }
    Ok(evidence)
}

fn persist_next_root_operation_securityagent_report(
    prefix: &str,
    expected_sha256: &str,
    bytes: &[u8],
) -> Result<SecurityAgentRawReportEvidenceV2> {
    if prefix.is_empty() || prefix.contains(['/', '\0', '\n', '\r']) {
        bail!("root-operation SecurityAgent report prefix is invalid")
    }
    for ordinal in 1_u8..=32 {
        let name = format!("{prefix}-{ordinal:02}.ui.observation.v2.json");
        if read_runner_private_bytes_optional(&name)?.is_none() {
            return persist_root_operation_securityagent_report(&name, expected_sha256, bytes);
        }
    }
    bail!("root-operation SecurityAgent report exceeded its 32-process recovery bound")
}

fn persist_root_operation_terminal_alert(
    operation: &'static str,
    alert: &crate::securityagent::SecurityAgentTerminalAlertV2,
) -> Result<()> {
    persist_root_operation_terminal_alert_with_context(operation, alert, None)
}

fn persist_root_operation_terminal_alert_with_context(
    operation: &'static str,
    alert: &crate::securityagent::SecurityAgentTerminalAlertV2,
    active: Option<(RepetitionV2, &SurrogateCreationReceiptV2)>,
) -> Result<()> {
    if !matches!(
        operation,
        "general-failure-restoration"
            | "peer-control-global-restoration"
            | "native-evidence-cleanup"
            | "terminal-finalizer-failure-cleanup"
            | "global-nonce-absence-baseline"
            | "peer-surrogate-state"
            | "runner-first-security-interaction-denial"
            | "nobody-owner-first-security-interaction-denial"
            | "finalizer-service-bootstrap-startup"
            | "publisher-startup"
    ) {
        bail!("root-operation ALERT purpose is outside the closed set")
    }
    alert.raw_report.validate_terminal_alert()?;
    alert.arm_evidence.validate_terminal_alert()?;
    write_runner_receipt(
        &format!("root-operation-{operation}.terminal-alert.arm-evidence.v2.json"),
        &canonical_bytes_v2(&alert.arm_evidence)?,
    )?;
    let live_peers = match read_runner_private_optional::<RootOperationLivePeersV2>(
        ROOT_OPERATION_UI_LIVE_PEERS_NAME,
    )? {
        Some(peers) => refresh_root_operation_live_peers(&peers)?,
        None => Vec::new(),
    };
    let (
        active_repetition,
        active_scope_id,
        creation_receipt_sha256,
        journal_absent_at_alert,
        journal_observation_sha256,
    ) = match active {
        Some((repetition, creation)) => {
            creation.validate(repetition)?;
            if observe_general_failure_accepted_journal(repetition)?.is_some() {
                bail!("root-operation ALERT reached an authenticated accepted journal")
            }
            let observation = document_sha256_v2(&(
                "substrate.r3-macos-root-operation-ui-journal-absence.v2",
                repetition.ordinal(),
                repetition.scope_id(),
                libc::ENOENT,
            ))?;
            (
                Some(repetition.ordinal()),
                Some(repetition.scope_id().to_owned()),
                Some(document_sha256_v2(creation)?),
                Some(true),
                Some(observation),
            )
        }
        None => (None, None, None, None, None),
    };
    let cursor = RootOperationUiStopCursorV2 {
        schema_owner: "substrate.r3-macos-disposable-root-operation-ui-stop".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        operation: operation.to_owned(),
        active_repetition,
        active_scope_id,
        creation_receipt_sha256,
        journal_absent_at_alert,
        journal_observation_sha256,
        triggering_securityagent_evidence_sha256: document_sha256_v2(&alert.arm_evidence)?,
        triggering_securityagent_evidence: alert.arm_evidence.clone(),
        live_peers,
        terminal_no_normal_resume: true,
    };
    validate_root_operation_ui_stop_cursor(&cursor)?;
    match read_runner_private_optional::<RootOperationUiStopCursorV2>(
        ROOT_OPERATION_UI_STOP_CURSOR_NAME,
    )? {
        Some(existing) if existing == cursor => Ok(()),
        Some(_) => bail!("root-operation UI-stop cursor changed after terminal ALERT"),
        None => write_runner_receipt(
            ROOT_OPERATION_UI_STOP_CURSOR_NAME,
            &canonical_bytes_v2(&cursor)?,
        ),
    }
}

fn validate_root_operation_ui_stop_cursor(value: &RootOperationUiStopCursorV2) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-root-operation-ui-stop"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || !matches!(
            value.operation.as_str(),
            "general-failure-restoration"
                | "peer-control-global-restoration"
                | "native-evidence-cleanup"
                | "terminal-finalizer-failure-cleanup"
                | "global-nonce-absence-baseline"
                | "peer-surrogate-state"
                | "runner-first-security-interaction-denial"
                | "nobody-owner-first-security-interaction-denial"
                | "finalizer-service-bootstrap-startup"
                | "publisher-startup"
        )
        || !value.terminal_no_normal_resume
        || value.triggering_securityagent_evidence_sha256
            != document_sha256_v2(&value.triggering_securityagent_evidence)?
    {
        bail!("root-operation UI-stop cursor changed its terminal authority")
    }
    value
        .triggering_securityagent_evidence
        .validate_terminal_alert()?;
    let has_active = value.active_repetition.is_some();
    let operation_requires_active = matches!(
        value.operation.as_str(),
        "peer-surrogate-state" | "nobody-owner-first-security-interaction-denial"
    );
    if has_active
        != (value.active_scope_id.is_some()
            && value.creation_receipt_sha256.is_some()
            && value.journal_absent_at_alert == Some(true)
            && value.journal_observation_sha256.is_some())
        || (operation_requires_active && !has_active)
        || (!operation_requires_active && has_active)
    {
        bail!("root-operation UI-stop cursor changed its active repetition binding")
    }
    if let Some(ordinal) = value.active_repetition {
        let repetition = match ordinal {
            1 => RepetitionV2::One,
            2 => RepetitionV2::Two,
            _ => bail!("root-operation UI-stop cursor has an invalid repetition"),
        };
        repetition.validate_binding(
            ordinal,
            value
                .active_scope_id
                .as_deref()
                .context("active UI-stop cursor lacks a scope")?,
        )?;
    }
    for digest in [
        value.creation_receipt_sha256.as_ref(),
        value.journal_observation_sha256.as_ref(),
        Some(&value.triggering_securityagent_evidence_sha256),
    ]
    .into_iter()
    .flatten()
    {
        if !is_sha256(digest) {
            bail!("root-operation UI-stop cursor contains a non-digest binding")
        }
    }
    for peer in &value.live_peers {
        validate_root_operation_live_peer(peer)?;
    }
    Ok(())
}

fn securityagent_arm_evidence(bytes: &[u8]) -> Result<SecurityAgentArmEvidenceV2> {
    let reports = vec![securityagent_raw_evidence(bytes)?];
    let evidence = SecurityAgentArmEvidenceV2 {
        schema_owner: "substrate.r3-macos-securityagent-arm-observation".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        observer_path: SECURITYAGENT_OBSERVER_PATH_V2.to_owned(),
        report_set_sha256: document_sha256_v2(&reports)?,
        reports,
        overlap_rearm_count: 0,
        gap_free_rearm_coverage: true,
        unexpected_ui_observed: false,
    };
    evidence.validate()?;
    Ok(evidence)
}

fn runner_securityagent_alert(
    repetition: RepetitionV2,
    arm: &'static str,
    observed: &ObservedChildV2,
) -> Result<anyhow::Error> {
    if !observed.unexpected_ui_observed {
        bail!("runner alert constructor received a no-UI child")
    }
    let raw_report = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(&observed.securityagent_report),
        raw_report_sha256: observed.securityagent_report_sha256.clone(),
        raw_report_byte_length: u64::try_from(observed.securityagent_report.len())?,
    };
    let evidence = single_securityagent_terminal_alert_evidence_v2(raw_report)?;
    Ok(anyhow::Error::new(RunnerSecurityAgentAlertV2 {
        repetition,
        arm: arm.to_owned(),
        evidence,
    }))
}

fn recover_creator_rollback(marker: &MarkerRoot, inputs: &FrozenRunnerInputs) -> Result<()> {
    let cursor = marker
        .read_rollback()?
        .context("creator rollback recovery cursor is absent")?;
    if cursor.phase == CreatorRollbackPhaseV2::Observed {
        let bytes = stable_read_file(
            &Path::new(RUNNER_ROOT).join("creator-emergency-rollback.receipt.v2.json"),
            0,
            Some(libc::S_IFREG | 0o600),
        )?;
        let receipt: CreatorRollbackReceiptV2 = parse_canonical_v2(&bytes)?;
        receipt.validate(&cursor)?;
        bail!("creator route previously stopped after exact emergency rollback")
    }
    if !matches!(
        cursor.phase,
        CreatorRollbackPhaseV2::Prepared | CreatorRollbackPhaseV2::Invoked
    ) {
        bail!("creator rollback recovery cursor is outside its two open phases")
    }
    observe_attested_creator_rollback(marker, inputs)?;
    bail!("creator route recovered exact emergency rollback and stopped")
}

fn observe_attested_creator_rollback(
    marker: &MarkerRoot,
    inputs: &FrozenRunnerInputs,
) -> Result<CreatorRollbackReceiptV2> {
    let before = marker
        .read_rollback()?
        .context("creator rollback cursor is absent before attested invocation")?;
    if !matches!(
        before.phase,
        CreatorRollbackPhaseV2::Prepared | CreatorRollbackPhaseV2::Invoked
    ) {
        bail!("creator rollback invocation is outside its two open phases")
    }
    let (rollback, process_attestation_sha256) = observe_sealed_child_with(
        sealed_command(CREATOR_EXECUTABLE_PATH, MARKER_ROOT, None, None)?,
        |pid| {
            attest_stopped_creator_rollback_process(pid as i32, &before, &inputs.creator_identity)
        },
    )?;
    process_attestation_sha256
        .context("creator rollback lacks its durable stopped-process attestation")?;
    if rollback.unexpected_ui_observed {
        bail!("creator rollback observed unexpected SecurityAgent UI; preserving state")
    }
    if !rollback.status.success() {
        bail!(
            "creator rollback failed after attested resume: {}",
            creator_failure_diagnostic(&rollback)
        )
    }
    let receipt: CreatorRollbackReceiptV2 = parse_stdout_json(&rollback)?;
    let observed = marker
        .read_rollback()?
        .context("creator rollback recovery cursor disappeared")?;
    receipt.validate(&observed)?;
    write_runner_receipt(
        "creator-emergency-rollback.receipt.v2.json",
        &canonical_bytes_v2(&receipt)?,
    )?;
    Ok(receipt)
}

fn creator_failure_diagnostic(observed: &ObservedChildV2) -> String {
    let stderr = String::from_utf8_lossy(&observed.stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        format!("exit_status={:?}; stderr=<empty>", observed.status.code())
    } else {
        format!(
            "exit_status={:?}; stderr={stderr:?}",
            observed.status.code()
        )
    }
}

fn recover_unobserved_creator_arm(marker: &MarkerRoot, inputs: &FrozenRunnerInputs) -> Result<()> {
    for ordinal in 1_u8..=10 {
        let Some(prepared) = read_runner_private_optional::<CreatorArmPreparedCursorV2>(&format!(
            "creator-arm-{ordinal:02}.prepared.cursor.v2.json"
        ))?
        else {
            continue;
        };
        if prepared.global_ordinal != ordinal
            || prepared.schema_owner != GLOBAL_PRE_EFFECT_PACKET_OWNER_V2
            || prepared.schema_version != EXPERIMENT_VERSION_V2
            || prepared.experiment_id != EXPERIMENT_ID_V2
            || !prepared.invocation_may_begin
            || !prepared.missing_observation_requires_terminal_rollback
        {
            bail!("creator pre-arm cursor changed its closed authority")
        }
        let shared = read_runner_private_optional::<CreatorNativeArmReceiptV2>(&format!(
            "creator-arm-{ordinal:02}.shared-receipt.v2.json"
        ))?;
        let observed = read_runner_private_optional::<CreatorArmObservedCursorV2>(&format!(
            "creator-arm-{ordinal:02}.observed.cursor.v2.json"
        ))?;
        if let Some(observed) = observed {
            let shared = shared.context("observed creator cursor lacks its shared receipt")?;
            if observed.schema_owner != GLOBAL_PRE_EFFECT_PACKET_OWNER_V2
                || observed.schema_version != EXPERIMENT_VERSION_V2
                || observed.experiment_id != EXPERIMENT_ID_V2
                || observed.prepared_sha256 != document_sha256_v2(&prepared)?
                || observed.shared_receipt_sha256 != document_sha256_v2(&shared)?
                || !observed.terminal_no_reinvoke
                || !is_sha256(&observed.raw_securityagent_report_sha256)
            {
                bail!("observed creator cursor changed its exact arm evidence")
            }
            continue;
        }
        if let Some(shared) = shared {
            let raw = read_runner_private_bytes_optional(&format!(
                "creator-arm-{ordinal:02}.securityagent-observation.v2.json"
            ))?
            .context("creator shared receipt lacks its durable raw UI report")?;
            let raw_sha256 = sha256_hex_v2(&raw);
            if raw_sha256 != shared.securityagent_report.raw_report_sha256 {
                bail!("creator shared receipt differs from its raw UI report")
            }
            let observed = CreatorArmObservedCursorV2 {
                schema_owner: GLOBAL_PRE_EFFECT_PACKET_OWNER_V2.to_owned(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_owned(),
                prepared_sha256: document_sha256_v2(&prepared)?,
                shared_receipt_sha256: document_sha256_v2(&shared)?,
                raw_securityagent_report_sha256: raw_sha256,
                terminal_no_reinvoke: true,
            };
            write_runner_receipt(
                &format!("creator-arm-{ordinal:02}.observed.cursor.v2.json"),
                &canonical_bytes_v2(&observed)?,
            )?;
            continue;
        }

        // The child may have performed a native call or advanced its marker, but no canonical raw
        // UI/native receipt survived. Never rerun that arm or synthesize evidence: cursor-first
        // emergency rollback is the only authorized continuation.
        neutralize_unobserved_creator_child(&prepared, inputs)?;
        let repetition = match prepared.repetition {
            1 => FixedRepetitionV2::First,
            2 => FixedRepetitionV2::Second,
            _ => bail!("creator pre-arm cursor has an invalid repetition"),
        };
        let failure_sha256 = document_sha256_v2(&prepared)?;
        if marker.read_rollback()?.is_none() {
            marker.prepare_rollback(repetition, &failure_sha256)?;
        }
        return recover_creator_rollback(marker, inputs);
    }
    Ok(())
}

fn neutralize_unobserved_creator_child(
    arm: &CreatorArmPreparedCursorV2,
    inputs: &FrozenRunnerInputs,
) -> Result<()> {
    let expected_identity = match arm.executable_path.as_str() {
        CREATOR_EXECUTABLE_PATH => &inputs.creator_identity,
        WRONG_IDENTITY_EXECUTABLE_PATH => &inputs.wrong_identity,
        _ => bail!("creator recovery prepared cursor has an alternate executable path"),
    };
    if arm.executable_identity_sha256 != document_sha256_v2(expected_identity)? {
        bail!("creator recovery prepared cursor changed its executable identity")
    }
    let prepared_name = format!(
        "creator-arm-{:02}.child-disposition.prepared.v2.json",
        arm.global_ordinal
    );
    let observed_name = format!(
        "creator-arm-{:02}.child-disposition.observed.v2.json",
        arm.global_ordinal
    );
    let disposition = if let Some(existing) =
        read_runner_private_optional::<CreatorChildDispositionPreparedV2>(&prepared_name)?
    {
        validate_creator_child_disposition_prepared(&existing, arm)?;
        existing
    } else {
        let durable_attestation_name = format!(
            "creator-arm-{:02}.process-attestation.v2.json",
            arm.global_ordinal
        );
        let durable_attestation = read_runner_private_bytes_optional(&durable_attestation_name)?;
        let processes = observe_compiled_creator_processes(inputs)?;
        if processes.len() > 1 {
            bail!("creator recovery found multiple compiled creator/wrong processes")
        }
        let exact_child = processes.into_iter().next();
        if let Some(child) = exact_child.as_ref() {
            if child.executable_path != arm.executable_path
                || child.executable_identity_sha256 != arm.executable_identity_sha256
            {
                bail!("creator recovery found a process outside the prepared arm identity")
            }
        }
        if let Some(bytes) = durable_attestation.as_ref() {
            validate_creator_process_attestation_for_recovery(bytes, arm, exact_child.as_ref())?;
        }
        let value = CreatorChildDispositionPreparedV2 {
            schema_owner: "substrate.r3-macos-disposable-creator-child-disposition".to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            arm_prepared_sha256: document_sha256_v2(arm)?,
            durable_process_attestation_sha256: durable_attestation
                .as_ref()
                .map(|bytes| sha256_hex_v2(bytes)),
            exact_child,
            individual_sigkill_authorized: true,
            normal_arm_resume_forbidden: true,
        };
        validate_creator_child_disposition_prepared(&value, arm)?;
        write_runner_receipt(&prepared_name, &canonical_bytes_v2(&value)?)?;
        value
    };
    if let Some(existing) =
        read_runner_private_optional::<CreatorChildDispositionObservedV2>(&observed_name)?
    {
        validate_creator_child_disposition_observed(&existing, &disposition)?;
        if !observe_compiled_creator_processes(inputs)?.is_empty() {
            bail!("creator child reappeared after its terminal disposition")
        }
        return Ok(());
    }

    let mut sent = false;
    if let Some(expected) = disposition.exact_child.as_ref() {
        if let Some(current) = observe_exact_creator_pid(expected.pid, inputs)? {
            if current != *expected {
                bail!("creator child PID/start/path identity changed before individual SIGKILL")
            }
            // SAFETY: the exact one PID/start/path/code identity was just reattested; no process
            // group or pathname-derived signal authority is used.
            if unsafe { libc::kill(expected.pid, libc::SIGKILL) } != 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(error).context("kill exact unobserved creator child");
                }
            } else {
                sent = true;
            }
        }
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        let remaining = observe_compiled_creator_processes(inputs)?;
        if remaining.is_empty() {
            break;
        }
        if std::time::Instant::now() >= deadline {
            bail!("compiled creator/wrong process remains after exact child disposition")
        }
        thread::sleep(Duration::from_millis(10));
    }
    let observed = CreatorChildDispositionObservedV2 {
        schema_owner: "substrate.r3-macos-disposable-creator-child-disposition".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        disposition_prepared_sha256: document_sha256_v2(&disposition)?,
        individual_sigkill_sent: sent,
        exact_creator_and_wrong_processes_absent: true,
    };
    validate_creator_child_disposition_observed(&observed, &disposition)?;
    write_runner_receipt(&observed_name, &canonical_bytes_v2(&observed)?)
}

fn validate_creator_child_disposition_prepared(
    value: &CreatorChildDispositionPreparedV2,
    arm: &CreatorArmPreparedCursorV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-creator-child-disposition"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.arm_prepared_sha256 != document_sha256_v2(arm)?
        || !value.individual_sigkill_authorized
        || !value.normal_arm_resume_forbidden
        || value
            .durable_process_attestation_sha256
            .as_ref()
            .is_some_and(|digest| !is_sha256(digest))
    {
        bail!("creator child disposition prepared cursor changed")
    }
    if let Some(child) = value.exact_child.as_ref() {
        if child.pid <= 0
            || child.effective_uid != 0
            || child.effective_gid != 0
            || child.executable_path != arm.executable_path
            || child.executable_identity_sha256 != arm.executable_identity_sha256
            || !is_sha256(&child.process_start_identity_sha256)
        {
            bail!("creator child disposition identity changed")
        }
    }
    Ok(())
}

fn validate_creator_child_disposition_observed(
    value: &CreatorChildDispositionObservedV2,
    prepared: &CreatorChildDispositionPreparedV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-creator-child-disposition"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.disposition_prepared_sha256 != document_sha256_v2(prepared)?
        || !value.exact_creator_and_wrong_processes_absent
    {
        bail!("creator child disposition observation changed")
    }
    Ok(())
}

fn validate_creator_process_attestation_for_recovery(
    bytes: &[u8],
    arm: &CreatorArmPreparedCursorV2,
    child: Option<&CreatorChildIdentityV2>,
) -> Result<()> {
    let value: serde_json::Value = parse_canonical_v2(bytes)?;
    let object = value
        .as_object()
        .context("creator process attestation is not an object")?;
    let pid = object.get("pid").and_then(serde_json::Value::as_i64);
    let uid = object
        .get("effective_uid")
        .and_then(serde_json::Value::as_u64);
    let gid = object
        .get("effective_gid")
        .and_then(serde_json::Value::as_u64);
    let start = object
        .get("process_start_identity_sha256")
        .and_then(serde_json::Value::as_str);
    let path = object
        .get("executable_path")
        .and_then(serde_json::Value::as_str);
    let identity = object
        .get("executable_identity_sha256")
        .and_then(serde_json::Value::as_str);
    if pid.is_none_or(|pid| pid <= 0)
        || !start.is_some_and(is_sha256)
        || uid != Some(0)
        || gid != Some(0)
        || path != Some(arm.executable_path.as_str())
        || identity != Some(arm.executable_identity_sha256.as_str())
        || child.is_some_and(|child| {
            pid != Some(i64::from(child.pid))
                || start != Some(child.process_start_identity_sha256.as_str())
        })
    {
        bail!("durable creator process attestation changed before recovery")
    }
    Ok(())
}

fn observe_compiled_creator_processes(
    inputs: &FrozenRunnerInputs,
) -> Result<Vec<CreatorChildIdentityV2>> {
    let mut observed = Vec::new();
    for pid in list_process_ids()? {
        let path = match pid_path(pid) {
            Ok(path) => path,
            Err(error) if is_process_disappearance_error(&error) => continue,
            Err(error) => return Err(error),
        };
        let identity = if path == Path::new(CREATOR_EXECUTABLE_PATH) {
            &inputs.creator_identity
        } else if path == Path::new(WRONG_IDENTITY_EXECUTABLE_PATH) {
            &inputs.wrong_identity
        } else {
            continue;
        };
        let process = match process_info(pid) {
            Ok(process) => process,
            Err(error) if is_process_disappearance_error(&error) => continue,
            Err(error) => return Err(error),
        };
        if process.uid != 0 || process.gid != 0 {
            bail!("compiled creator/wrong path is live under an alternate principal")
        }
        let measured = measure_frozen_executable(frozen_code_for_identity(
            path.to_str().context("creator process path is not UTF-8")?,
            identity,
        )?)?;
        if measured != *identity {
            bail!("compiled creator/wrong executable identity changed while live")
        }
        observed.push(CreatorChildIdentityV2 {
            pid,
            effective_uid: process.uid,
            effective_gid: process.gid,
            process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
                pid,
                seconds: process.start_seconds,
                microseconds: process.start_microseconds,
            })?,
            executable_path: path.to_string_lossy().into_owned(),
            executable_identity_sha256: document_sha256_v2(identity)?,
        });
    }
    observed.sort_by_key(|value| value.pid);
    Ok(observed)
}

fn observe_exact_creator_pid(
    pid: i32,
    inputs: &FrozenRunnerInputs,
) -> Result<Option<CreatorChildIdentityV2>> {
    Ok(observe_compiled_creator_processes(inputs)?
        .into_iter()
        .find(|value| value.pid == pid))
}

fn spawn_publisher(inputs: &FrozenRunnerInputs) -> Result<Child> {
    spawn_observed_publisher(inputs, "normal")
}

fn spawn_observed_publisher(inputs: &FrozenRunnerInputs, purpose: &'static str) -> Result<Child> {
    if !matches!(
        purpose,
        "normal" | "publisher_ui_rollback" | "general_failure_rollback"
    ) {
        bail!("publisher startup purpose is outside the closed experiment sequence")
    }
    let startup_ordinal = next_publisher_startup_ordinal(purpose)?;
    measure_frozen_executable(inputs.publisher)?;
    let command = sealed_command(
        DISPOSABLE_PUBLISHER_EXECUTABLE_PATH,
        DISPOSABLE_PUBLISHER_ROOT,
        None,
        None,
    )?;
    let observed = observe_stopped_child_startup(
        command,
        |pid| {
            let pid = i32::try_from(pid).context("publisher startup PID exceeds i32")?;
            let process = process_info(pid)?;
            if process.pid != u32::try_from(pid)?
                || process.uid != 0
                || process.gid != 0
                || process.status != PROCESS_STATUS_STOPPED_V2
                || canonical_account(process.uid)?.0 != "root"
                || pid_path(pid)? != Path::new(DISPOSABLE_PUBLISHER_EXECUTABLE_PATH)
            {
                bail!("stopped publisher startup process identity changed")
            }
            let measured = measure_frozen_executable(inputs.publisher)?;
            Ok((
                pid,
                process.uid,
                process.gid,
                document_sha256_v2(&ProcessStartJoinV2 {
                    pid,
                    seconds: process.start_seconds,
                    microseconds: process.start_microseconds,
                })?,
                document_sha256_v2(&measured)?,
            ))
        },
        |alert| persist_root_operation_terminal_alert("publisher-startup", alert),
    )?;
    let (pid, uid, gid, process_start_identity_sha256, executable_identity_sha256) =
        observed.attestation;
    let securityagent_report = securityagent_raw_evidence(&observed.securityagent_report)?;
    if securityagent_report.raw_report_sha256 != observed.securityagent_report_sha256 {
        bail!("publisher startup raw SecurityAgent report digest changed")
    }
    let attestation = PublisherStartupAttestationV2 {
        schema_owner: "substrate.r3-macos-disposable-publisher-startup-attestation".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        purpose: purpose.to_owned(),
        startup_ordinal,
        pid,
        effective_uid: uid,
        effective_gid: gid,
        supplementary_groups: measured_child_supplementary_groups_v2(pid)?,
        canonical_account: "root".to_owned(),
        process_start_identity_sha256,
        executable_identity_sha256,
        securityagent_report,
        exact_post_denial_sigstop_observed: true,
        resume_authorized: true,
    };
    write_runner_receipt(
        &format!("publisher-startup-{purpose}-{startup_ordinal:02}.attestation.v2.json"),
        &canonical_bytes_v2(&attestation)?,
    )?;
    let mut child = observed.child;
    // SAFETY: the exact stopped PID/start/path/code identity and complete no-UI report are durable.
    if unsafe { libc::kill(pid, libc::SIGCONT) } != 0 {
        let error = std::io::Error::last_os_error();
        let _ = child.kill();
        let _ = child.wait();
        return Err(error).context("resume exact observed publisher after startup denial");
    }
    Ok(child)
}

fn next_publisher_startup_ordinal(purpose: &str) -> Result<u8> {
    for ordinal in 1_u8..=4 {
        let name = format!("publisher-startup-{purpose}-{ordinal:02}.attestation.v2.json");
        match read_runner_private_optional::<PublisherStartupAttestationV2>(&name)? {
            Some(existing)
                if existing.schema_owner
                    == "substrate.r3-macos-disposable-publisher-startup-attestation"
                    && existing.schema_version == EXPERIMENT_VERSION_V2
                    && existing.experiment_id == EXPERIMENT_ID_V2
                    && existing.purpose == purpose
                    && existing.startup_ordinal == ordinal
                    && existing.exact_post_denial_sigstop_observed
                    && existing.resume_authorized =>
            {
                existing.securityagent_report.validate()?;
            }
            Some(_) => bail!("publisher startup attestation chain changed"),
            None => return Ok(ordinal),
        }
    }
    bail!("publisher startup attestation exceeded its four-process closed bound")
}

fn spawn_harness(inputs: &FrozenRunnerInputs) -> Result<Child> {
    measure_frozen_executable(inputs.harness)?;
    let (_, gid) = canonical_account(DISPOSABLE_HARNESS_UID_V2)?;
    let mut command = sealed_command(
        DISPOSABLE_HARNESS_PATH_V2,
        "/",
        Some(DISPOSABLE_HARNESS_UID_V2),
        Some(gid),
    )?;
    command.process_group(0);
    command
        .spawn()
        .context("spawn exact UID501 disposable harness process group")
}

fn publisher_exit_disposition(success: bool, exit_code: Option<i32>) -> PublisherExitDispositionV2 {
    if exit_code == Some(SECURITYAGENT_ALERT_EXIT_CODE) {
        PublisherExitDispositionV2::TerminalSecurityAgentAlert
    } else if success {
        PublisherExitDispositionV2::Completed
    } else {
        PublisherExitDispositionV2::TerminalClosedFailure
    }
}

fn both_complete_responses_present() -> Result<bool> {
    for repetition in RepetitionV2::ALL {
        if !complete_response_present(repetition)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn terminate_harness_process_group(child: &mut Child) -> Result<HarnessTerminationObservationV2> {
    let process_group_id = i32::try_from(child.id()).context("harness PID exceeds i32")?;
    let initial_status = child.try_wait()?;
    if process_group_members(process_group_id)?.is_empty() {
        let status = initial_status.context(
            "harness process group is absent but its exact leader lacks one exit observation",
        )?;
        return Ok(HarnessTerminationObservationV2 {
            schema_owner: "substrate.r3-macos-disposable-experiment-runner-harness-termination"
                .to_owned(),
            schema_version: 2,
            process_group_id,
            sigterm_sent: false,
            sigkill_sent: false,
            exit_code: status.code(),
            terminating_signal: status.signal(),
            process_group_absent_after: true,
        });
    }
    let members = observe_process_group_members(process_group_id)?;
    let sigkill_sent = signal_exact_process_members_and_wait_absent(process_group_id, &members)?;
    let status = match initial_status {
        Some(status) => status,
        None => child.wait()?,
    };
    Ok(HarnessTerminationObservationV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-harness-termination"
            .to_owned(),
        schema_version: 2,
        process_group_id,
        sigterm_sent: true,
        sigkill_sent,
        exit_code: status.code(),
        terminating_signal: status.signal(),
        process_group_absent_after: true,
    })
}

fn signal_exact_process_members_and_wait_absent(
    process_group_id: i32,
    expected: &[GeneralFailureGroupMemberV2],
) -> Result<bool> {
    if process_group_id <= 0 {
        bail!("sealed process-group identifier is not positive")
    }
    validate_exact_process_members_before_signal(process_group_id, expected)?;
    for member in expected {
        // Reattest the individual PID immediately before its signal.  A prior group snapshot is
        // not signal authority: membership can change and a PID can be reused after the snapshot.
        reattest_exact_process_member(process_group_id, member)?;
        // SAFETY: this one PID/start/path/PGID identity was immediately reattested above.
        if unsafe { libc::kill(member.pid, libc::SIGTERM) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error).context("terminate exact harness member");
            }
        }
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        if process_group_members(process_group_id)?.is_empty() {
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(10));
    }
    let remaining = observe_process_group_members(process_group_id)?;
    for member in &remaining {
        let recorded = expected
            .iter()
            .find(|expected| expected.pid == member.pid)
            .context("harness process group gained an unrecorded member before SIGKILL")?;
        if recorded != member {
            bail!("harness process-group member changed identity before SIGKILL")
        }
    }
    for member in &remaining {
        reattest_exact_process_member(process_group_id, member)?;
        // SAFETY: this one PID/start/path/PGID identity was immediately reattested above.
        if unsafe { libc::kill(member.pid, libc::SIGKILL) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error).context("kill exact harness member after deadline");
            }
        }
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        if process_group_members(process_group_id)?.is_empty() {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(10));
    }
    bail!("exact harness process group remained live after bounded SIGKILL")
}

fn validate_exact_process_members_before_signal(
    process_group_id: i32,
    expected: &[GeneralFailureGroupMemberV2],
) -> Result<()> {
    let current = observe_process_group_members(process_group_id)?;
    if current != expected {
        bail!("process group membership changed before exact per-PID signal")
    }
    Ok(())
}

fn reattest_exact_process_member(
    process_group_id: i32,
    expected: &GeneralFailureGroupMemberV2,
) -> Result<()> {
    // SAFETY: getpgid performs a read-only lookup of the one recorded PID.
    let pgid = unsafe { libc::getpgid(expected.pid) };
    if pgid < 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            return Ok(());
        }
        return Err(error).context("reattest exact process member group");
    }
    if pgid != process_group_id {
        bail!("exact process member changed group before individual signal")
    }
    let process = process_info(expected.pid)?;
    let path = pid_path(expected.pid)?;
    let observed = GeneralFailureGroupMemberV2 {
        pid: expected.pid,
        effective_uid: process.uid,
        supplementary_groups: measured_child_supplementary_groups_v2(expected.pid)?,
        process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
            pid: expected.pid,
            seconds: process.start_seconds,
            microseconds: process.start_microseconds,
        })?,
        executable_path: path.to_string_lossy().into_owned(),
    };
    if observed != *expected {
        bail!("exact process member PID/start/path identity changed before individual signal")
    }
    Ok(())
}

fn process_group_members(process_group_id: i32) -> Result<Vec<i32>> {
    let mut members = Vec::new();
    for pid in list_process_ids()? {
        // SAFETY: getpgid is a read-only lookup for one enumerated PID.
        let pgid = unsafe { libc::getpgid(pid) };
        if pgid == process_group_id {
            members.push(pid);
        } else if pgid < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error).context("measure sealed process-group membership");
            }
        }
    }
    members.sort_unstable();
    members.dedup();
    Ok(members)
}

fn observe_process_group_members(
    process_group_id: i32,
) -> Result<Vec<GeneralFailureGroupMemberV2>> {
    let mut observations = Vec::new();
    for pid in process_group_members(process_group_id)? {
        let process = match process_info(pid) {
            Ok(value) => value,
            Err(error) if is_process_disappearance_error(&error) => continue,
            Err(error) => return Err(error),
        };
        let executable_path = match pid_path(pid) {
            Ok(value) => value,
            Err(error) if is_process_disappearance_error(&error) => continue,
            Err(error) => return Err(error),
        };
        observations.push(GeneralFailureGroupMemberV2 {
            pid,
            effective_uid: process.uid,
            supplementary_groups: measured_child_supplementary_groups_v2(pid)?,
            process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
                pid,
                seconds: process.start_seconds,
                microseconds: process.start_microseconds,
            })?,
            executable_path: executable_path.to_string_lossy().into_owned(),
        });
    }
    observations.sort_by_key(|value| value.pid);
    Ok(observations)
}

fn is_process_disappearance_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
        .any(|cause| {
            matches!(
                cause.raw_os_error(),
                Some(code) if code == libc::ESRCH || code == libc::ENOENT
            )
        })
}

fn validate_recovered_process_group_members(
    peer: &GeneralFailureLivePeerV2,
) -> Result<Vec<GeneralFailureGroupMemberV2>> {
    let process_group_id = peer
        .process_group_id
        .context("recovered process group lacks its recorded identifier")?;
    let current = observe_process_group_members(process_group_id)?;
    for member in &current {
        let Some(recorded) = peer
            .process_group_members
            .iter()
            .find(|recorded| recorded.pid == member.pid)
        else {
            bail!("recovered process group contains an unrecorded member; refusing signal")
        };
        if recorded != member {
            bail!("recovered process-group member PID/start/path identity changed")
        }
    }
    Ok(current)
}

fn list_process_ids() -> Result<Vec<i32>> {
    let capacity = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if capacity <= 0 {
        return Err(std::io::Error::last_os_error()).context("size exact process list");
    }
    let capacity = usize::try_from(capacity)?
        .checked_add(128)
        .context("process list capacity overflow")?;
    let mut pids = vec![0_i32; capacity];
    let bytes = i32::try_from(
        pids.len()
            .checked_mul(size_of::<i32>())
            .context("process list bytes overflow")?,
    )?;
    let count = unsafe { proc_listallpids(pids.as_mut_ptr().cast(), bytes) };
    if count < 0 || usize::try_from(count)? >= pids.len() {
        return Err(std::io::Error::last_os_error()).context("read exact process list");
    }
    pids.truncate(usize::try_from(count)?);
    pids.retain(|pid| *pid > 0);
    pids.sort_unstable();
    pids.dedup();
    Ok(pids)
}

enum InstalledEmergencyMarkerV2 {
    PreCreation(PreCreationEmergencyRollbackMarkerV2),
    Created(EmergencyRollbackMarkerV2),
}

impl InstalledEmergencyMarkerV2 {
    fn artifact(&self) -> PublisherArtifactV2 {
        match self {
            Self::PreCreation(_) => PublisherArtifactV2::PreCreationEmergencyRollbackMarker,
            Self::Created(_) => PublisherArtifactV2::EmergencyRollbackMarker,
        }
    }

    fn bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::PreCreation(value) => canonical_bytes_v2(value),
            Self::Created(value) => canonical_bytes_v2(value),
        }
    }

    fn validate_receipt(
        &self,
        repetition: RepetitionV2,
        receipt: &EmergencyRollbackReceiptV2,
    ) -> Result<()> {
        match self {
            Self::PreCreation(marker) => receipt.validate_precreation(repetition, marker),
            Self::Created(marker) => receipt.validate(repetition, marker),
        }
    }
}

fn persist_publisher_ui_trigger_evidence(evidence: &SecurityAgentArmEvidenceV2) -> Result<()> {
    evidence.validate_terminal_alert()?;
    let bytes = canonical_bytes_v2(evidence)?;
    let path = Path::new(RUNNER_ROOT).join(PUBLISHER_UI_TRIGGER_EVIDENCE_NAME);
    if let Some(existing) = stable_read_file_optional(&path, 0, Some(libc::S_IFREG | 0o600))? {
        if existing != bytes {
            bail!("terminal SecurityAgent trigger evidence changed during recovery")
        }
        return Ok(());
    }
    write_runner_receipt(PUBLISHER_UI_TRIGGER_EVIDENCE_NAME, &bytes)
}

fn active_publisher_repetition() -> Result<RepetitionV2> {
    let mut active = None;
    for repetition in RepetitionV2::ALL {
        let Some(progress) = read_external_root_optional::<PublisherProgressV2>(
            repetition,
            PublisherArtifactV2::Progress,
        )?
        else {
            continue;
        };
        progress.validate(repetition)?;
        if progress.stage != PublisherStageV2::RestorationComplete
            && active.replace(repetition).is_some()
        {
            bail!("terminal SecurityAgent ALERT found multiple active repetitions")
        }
    }
    active.context("terminal SecurityAgent ALERT lacks one active repetition")
}

fn load_publisher_terminal_alert_evidence(
    repetition: RepetitionV2,
) -> Result<SecurityAgentArmEvidenceV2> {
    let parent = root_publisher_path_v2(repetition, PublisherArtifactV2::Progress)
        .parent()
        .context("publisher progress lacks its private parent")?
        .to_path_buf();
    let mut found = Vec::new();
    for stage in PUBLISHER_STAGE_SEQUENCE_V2 {
        let name = format!(
            "terminal-alert-{}.arm-evidence.v2.json",
            PublisherArtifactV2::SecurityAgentObservation(stage).filename()
        );
        if let Some(bytes) =
            stable_read_file_optional(&parent.join(name), 0, Some(libc::S_IFREG | 0o600))?
        {
            let evidence: SecurityAgentArmEvidenceV2 = parse_canonical_v2(&bytes)?;
            evidence.validate_terminal_alert()?;
            found.push(evidence);
        }
    }
    if found.len() != 1 {
        bail!("publisher exit86 does not bind exactly one immutable terminal ALERT report")
    }
    Ok(found.remove(0))
}

fn terminated_child_observation(
    child: &Child,
    status: &ExitStatus,
) -> Result<HarnessTerminationObservationV2> {
    let pid = i32::try_from(child.id()).context("terminated child PID exceeds i32")?;
    // SAFETY: read-only existence probe for the already-reaped exact child PID.
    if unsafe { libc::kill(pid, 0) } == 0
        || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
    {
        bail!("terminated child PID remains live or was reused before terminal cursor")
    }
    Ok(HarnessTerminationObservationV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-child-termination"
            .to_owned(),
        schema_version: 2,
        process_group_id: pid,
        sigterm_sent: false,
        sigkill_sent: false,
        exit_code: status.code(),
        terminating_signal: status.signal(),
        process_group_absent_after: true,
    })
}

fn terminal_runner_securityagent_alert(
    inputs: &FrozenRunnerInputs,
    alert: &RunnerSecurityAgentAlertV2,
    publisher: &mut Child,
    harness: &mut Child,
) -> Result<()> {
    alert.evidence.validate_terminal_alert()?;
    let publisher_live_peer = observe_general_failure_live_peer(
        publisher,
        GeneralFailureLivePeerKindV2::PublisherProcess,
        &inputs.prepared.publisher_identity,
        0,
    )?;
    let harness_live_peer = observe_general_failure_live_peer(
        harness,
        GeneralFailureLivePeerKindV2::HarnessProcessGroup,
        &inputs.peer.harness_identity,
        DISPOSABLE_HARNESS_UID_V2,
    )?;
    persist_publisher_ui_stop_cursor(
        alert.repetition,
        PublisherUiTriggerOriginV2::RunnerObservedControl,
        &alert.evidence,
        None,
        None,
        i32::try_from(publisher.id())?,
        i32::try_from(harness.id())?,
        Some(publisher_live_peer),
        Some(harness_live_peer),
    )?;
    let harness_termination = terminate_harness_process_group(harness)?;
    write_runner_receipt(
        PUBLISHER_UI_HARNESS_TERMINATION_NAME,
        &canonical_bytes_v2(&harness_termination)?,
    )?;
    let publisher_termination = terminate_single_child(publisher)?;
    write_runner_receipt(
        PUBLISHER_UI_PUBLISHER_TERMINATION_NAME,
        &canonical_bytes_v2(&publisher_termination)?,
    )?;
    complete_publisher_ui_terminal_rollback(inputs, &harness_termination, &publisher_termination)
}

fn recover_root_operation_ui_terminal_stop_if_present(
    inputs: &FrozenRunnerInputs,
    root_install_claims: &RootInstallClaimsBindingV2,
    activation_membrane: &ActivationMembraneGuardV2,
) -> Result<()> {
    let Some(cursor) = read_runner_private_optional::<RootOperationUiStopCursorV2>(
        ROOT_OPERATION_UI_STOP_CURSOR_NAME,
    )?
    else {
        return Ok(());
    };
    validate_root_operation_ui_stop_cursor(&cursor)?;
    if let Some(existing) = read_runner_private_optional::<RootOperationUiTerminalReceiptV2>(
        ROOT_OPERATION_UI_TERMINAL_RECEIPT_NAME,
    )? {
        validate_root_operation_ui_terminal_receipt(&existing, &cursor, root_install_claims)?;
        persist_terminal_admin_cleanup_authorization(
            "root_operation_securityagent_alert",
            document_sha256_v2(&existing)?,
            existing.rollback_result_sha256.clone(),
            existing
                .finalizer_cleanup_sha256
                .clone()
                .context("root-operation UI terminal receipt lacks finalizer cleanup")?,
            existing.creator_marker_restoration_sha256.clone(),
            root_install_claims,
            observe_retained_root_install_claims(root_install_claims)?,
        )?;
        bail!("root-operation SecurityAgent ALERT was already restored and is terminal")
    }
    if let Some(ordinal) = cursor.active_repetition {
        let repetition = if ordinal == 1 {
            RepetitionV2::One
        } else {
            RepetitionV2::Two
        };
        let creation: SurrogateCreationReceiptV2 =
            read_external_root_optional(repetition, PublisherArtifactV2::CreationReceipt)?
                .context("root-operation UI recovery lost its exact creation receipt")?;
        creation.validate(repetition)?;
        if Some(document_sha256_v2(&creation)?) != cursor.creation_receipt_sha256
            || observe_general_failure_accepted_journal(repetition)?.is_some()
        {
            bail!("root-operation UI recovery no longer proves pre-acceptance identity")
        }
    }

    let peer_termination = if let Some(existing) =
        read_runner_private_optional::<RootOperationLivePeerTerminationV2>(
            ROOT_OPERATION_UI_LIVE_PEER_TERMINATION_NAME,
        )? {
        validate_root_operation_live_peer_termination(&existing, &cursor)?;
        existing
    } else {
        let mut sigterm_sent = Vec::with_capacity(cursor.live_peers.len());
        let mut sigkill_sent = Vec::with_capacity(cursor.live_peers.len());
        for peer in &cursor.live_peers {
            let (sigterm, sigkill) = terminate_recovered_live_peer(peer)?;
            sigterm_sent.push(sigterm);
            sigkill_sent.push(sigkill);
        }
        let value = RootOperationLivePeerTerminationV2 {
            schema_owner: "substrate.r3-macos-disposable-root-operation-live-peer-termination"
                .to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            cursor_sha256: document_sha256_v2(&cursor)?,
            peers_sha256: document_sha256_v2(&cursor.live_peers)?,
            sigterm_sent,
            sigkill_sent,
            all_exact_peers_absent_after: true,
        };
        validate_root_operation_live_peer_termination(&value, &cursor)?;
        write_runner_receipt(
            ROOT_OPERATION_UI_LIVE_PEER_TERMINATION_NAME,
            &canonical_bytes_v2(&value)?,
        )?;
        value
    };

    let rollback_result = if let Some(restoration) = read_runner_private_optional::<
        GeneralFailureRestorationReceiptV2,
    >(GENERAL_FAILURE_RECEIPT_NAME)?
    {
        let prior_cursor: GeneralFailureRestorationCursorV2 = read_runner_private_optional(
            GENERAL_FAILURE_CURSOR_NAME,
        )?
        .context("root-operation ALERT after restoration lacks its prior terminal cursor")?;
        validate_general_failure_receipt(&restoration, &prior_cursor)?;
        let result: GeneralFailureRollbackResultV2 =
            read_runner_private_optional(GENERAL_FAILURE_ROLLBACK_RESULT_NAME)?
                .context("restored general failure lacks its exact rollback result")?;
        if !result.exact_restoration_complete || result.accepted_authority_ambiguity {
            bail!("prior general-failure restoration did not prove exact surrogate absence")
        }
        result
    } else {
        let expected_general_cursor = GeneralFailureRestorationCursorV2 {
            schema_owner: "substrate.r3-macos-disposable-experiment-runner-general-failure-cursor"
                .to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            trigger: GeneralFailureTriggerV2::RootOperationSecurityAgentAlert,
            publisher_exit: None,
            harness_exit: None,
            live_peer: None,
            root_operation_ui_stop_cursor_sha256: Some(document_sha256_v2(&cursor)?),
            terminal_no_resume: true,
        };
        validate_general_failure_cursor(&expected_general_cursor)?;
        let general_cursor = match read_runner_private_optional::<GeneralFailureRestorationCursorV2>(
            GENERAL_FAILURE_CURSOR_NAME,
        )? {
            Some(existing) if existing == expected_general_cursor => existing,
            Some(_) => bail!("root-operation UI recovery conflicts with another terminal cursor"),
            None => {
                write_runner_receipt(
                    GENERAL_FAILURE_CURSOR_NAME,
                    &canonical_bytes_v2(&expected_general_cursor)?,
                )?;
                expected_general_cursor
            }
        };
        ensure_general_failure_peer_terminated(&general_cursor, None)?;
        complete_general_failure_rollback(inputs, &general_cursor)?
    };
    if !rollback_result.exact_restoration_complete || rollback_result.accepted_authority_ambiguity {
        bail!("root-operation UI recovery could not prove exact pre-acceptance restoration")
    }
    let mut repetitions = Vec::with_capacity(RepetitionV2::ALL.len());
    for repetition in RepetitionV2::ALL {
        repetitions.push(validate_and_restore_failed_repetition(repetition)?);
    }
    remove_exact_installed_file(
        Path::new(ALTERNATE_COORDINATOR_PATH_V2),
        None,
        Some(&inputs.peer.alternate_path_identity.physical_identity_sha256),
    )?;
    remove_exact_installed_file(
        Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.prepared)?),
        None,
    )?;
    remove_exact_installed_file(
        Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.candidate)?),
        None,
    )?;
    remove_exact_installed_file(
        Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.peer)?),
        None,
    )?;
    let creator_marker_restoration_sha256 = restore_creator_marker_after_terminal_ui()?;
    let global_pre_effect = read_runner_global_pre_effect_packet_optional()?;
    if let Some(packet) = global_pre_effect.as_ref() {
        packet.validate(&inputs.prepared, &inputs.candidate, &inputs.peer)?;
    }
    let finalizer_cleanup_sha256 = terminal_finalizer_cleanup_receipt_sha256(
        global_pre_effect.as_ref(),
        activation_membrane,
        &document_sha256_v2(&cursor)?,
    )?;
    let root_install_claims_retention_observation_sha256 =
        observe_retained_root_install_claims(root_install_claims)?;
    let receipt = RootOperationUiTerminalReceiptV2 {
        schema_owner: "substrate.r3-macos-disposable-root-operation-ui-terminal".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        cursor_sha256: document_sha256_v2(&cursor)?,
        live_peer_termination_sha256: document_sha256_v2(&peer_termination)?,
        rollback_result_sha256: document_sha256_v2(&rollback_result)?,
        repetitions,
        creator_marker_restoration_sha256,
        finalizer_cleanup_sha256: Some(finalizer_cleanup_sha256.clone()),
        root_install_claims_binding_sha256: document_sha256_v2(root_install_claims)?,
        claims_retained: true,
        admin_cleanup_authorized: true,
        terminal_no_normal_resume: true,
    };
    validate_root_operation_ui_terminal_receipt(&receipt, &cursor, root_install_claims)?;
    write_runner_receipt(
        ROOT_OPERATION_UI_TERMINAL_RECEIPT_NAME,
        &canonical_bytes_v2(&receipt)?,
    )?;
    persist_terminal_admin_cleanup_authorization(
        "root_operation_securityagent_alert",
        document_sha256_v2(&receipt)?,
        document_sha256_v2(&rollback_result)?,
        finalizer_cleanup_sha256,
        receipt.creator_marker_restoration_sha256.clone(),
        root_install_claims,
        root_install_claims_retention_observation_sha256,
    )?;
    bail!("root-operation SecurityAgent ALERT completed exact terminal restoration")
}

fn validate_root_operation_live_peer_termination(
    value: &RootOperationLivePeerTerminationV2,
    cursor: &RootOperationUiStopCursorV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-root-operation-live-peer-termination"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.cursor_sha256 != document_sha256_v2(cursor)?
        || value.peers_sha256 != document_sha256_v2(&cursor.live_peers)?
        || value.sigterm_sent.len() != cursor.live_peers.len()
        || value.sigkill_sent.len() != cursor.live_peers.len()
        || !value.all_exact_peers_absent_after
    {
        bail!("root-operation live-peer termination evidence changed")
    }
    Ok(())
}

fn validate_root_operation_ui_terminal_receipt(
    value: &RootOperationUiTerminalReceiptV2,
    cursor: &RootOperationUiStopCursorV2,
    root_install_claims: &RootInstallClaimsBindingV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-root-operation-ui-terminal"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.cursor_sha256 != document_sha256_v2(cursor)?
        || value.repetitions.len() != RepetitionV2::ALL.len()
        || value.root_install_claims_binding_sha256 != document_sha256_v2(root_install_claims)?
        || !value.claims_retained
        || !value.admin_cleanup_authorized
        || !value.terminal_no_normal_resume
    {
        bail!("root-operation UI terminal receipt changed its cleanup authority")
    }
    for digest in [
        Some(&value.live_peer_termination_sha256),
        Some(&value.rollback_result_sha256),
        Some(&value.creator_marker_restoration_sha256),
        value.finalizer_cleanup_sha256.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if !is_sha256(digest) {
            bail!("root-operation UI terminal receipt contains a non-digest binding")
        }
    }
    for (restoration, repetition) in value.repetitions.iter().zip(RepetitionV2::ALL) {
        repetition.validate_binding(restoration.repetition, &restoration.scope_id)?;
        if !restoration.signing_seed_absent_after {
            bail!("root-operation UI terminal receipt retained a harness signing seed")
        }
    }
    Ok(())
}

fn recover_publisher_ui_terminal_stop_if_present(inputs: &FrozenRunnerInputs) -> Result<()> {
    let Some(cursor) =
        read_runner_private_optional::<PublisherUiStopCursorV2>(PUBLISHER_UI_STOP_CURSOR_NAME)?
    else {
        return Ok(());
    };
    validate_publisher_ui_stop_cursor(&cursor)?;
    let harness_termination = recover_publisher_ui_peer_termination(&cursor, false)?;
    let publisher_termination = recover_publisher_ui_peer_termination(&cursor, true)?;
    complete_publisher_ui_terminal_rollback(inputs, &harness_termination, &publisher_termination)
}

fn recover_publisher_ui_peer_termination(
    cursor: &PublisherUiStopCursorV2,
    publisher: bool,
) -> Result<HarnessTerminationObservationV2> {
    let (name, recorded_id, recorded_exit, live_peer) = if publisher {
        (
            PUBLISHER_UI_PUBLISHER_TERMINATION_NAME,
            cursor.publisher_pid,
            cursor.publisher_exit.as_ref(),
            cursor.publisher_live_peer.as_ref(),
        )
    } else {
        (
            PUBLISHER_UI_HARNESS_TERMINATION_NAME,
            cursor.harness_process_group_id,
            cursor.harness_exit.as_ref(),
            cursor.harness_live_peer.as_ref(),
        )
    };
    if let Some(existing) = read_runner_private_optional::<HarnessTerminationObservationV2>(name)? {
        if existing.process_group_id != recorded_id || !existing.process_group_absent_after {
            bail!("publisher UI peer-termination receipt changed")
        }
        return Ok(existing);
    }
    let (sigterm_sent, sigkill_sent) = if let Some(peer) = live_peer {
        terminate_recovered_live_peer(peer)?
    } else {
        if recorded_exit.is_none() {
            bail!("publisher UI cursor lacks both live peer and exact exit observation")
        }
        if publisher {
            // SAFETY: read-only existence probe for the exact exited publisher PID.
            if unsafe { libc::kill(recorded_id, 0) } == 0
                || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
            {
                bail!("publisher UI recovery found exited publisher PID live or reused")
            }
        } else if !process_group_members(recorded_id)?.is_empty() {
            bail!("publisher UI recovery found descendants in an exited harness process group")
        }
        (false, false)
    };
    let receipt = HarnessTerminationObservationV2 {
        schema_owner: if publisher {
            "substrate.r3-macos-disposable-experiment-runner-child-termination"
        } else {
            "substrate.r3-macos-disposable-experiment-runner-harness-termination"
        }
        .to_owned(),
        schema_version: 2,
        process_group_id: recorded_id,
        sigterm_sent,
        sigkill_sent,
        exit_code: recorded_exit.and_then(|exit| exit.exit_code),
        terminating_signal: recorded_exit.and_then(|exit| exit.terminating_signal),
        process_group_absent_after: true,
    };
    write_runner_receipt(name, &canonical_bytes_v2(&receipt)?)?;
    Ok(receipt)
}

#[allow(clippy::too_many_arguments)]
fn persist_publisher_ui_stop_cursor(
    expected_repetition: RepetitionV2,
    trigger_origin: PublisherUiTriggerOriginV2,
    evidence: &SecurityAgentArmEvidenceV2,
    publisher_exit: Option<ChildExitObservationV2>,
    harness_exit: Option<ChildExitObservationV2>,
    publisher_pid: i32,
    harness_process_group_id: i32,
    publisher_live_peer: Option<GeneralFailureLivePeerV2>,
    harness_live_peer: Option<GeneralFailureLivePeerV2>,
) -> Result<PublisherUiStopCursorV2> {
    evidence.validate_terminal_alert()?;
    let evidence_sha256 = document_sha256_v2(evidence)?;
    persist_publisher_ui_trigger_evidence(evidence)?;
    if let Some(existing) =
        read_runner_private_optional::<PublisherUiStopCursorV2>(PUBLISHER_UI_STOP_CURSOR_NAME)?
    {
        validate_publisher_ui_stop_cursor(&existing)?;
        if existing.repetition != expected_repetition.ordinal()
            || existing.trigger_origin != trigger_origin
            || existing.triggering_securityagent_evidence_sha256 != evidence_sha256
            || existing.publisher_exit != publisher_exit
            || existing.harness_exit != harness_exit
            || existing.publisher_pid != publisher_pid
            || existing.harness_process_group_id != harness_process_group_id
            || existing.publisher_live_peer != publisher_live_peer
            || existing.harness_live_peer != harness_live_peer
        {
            bail!("publisher UI-stop cursor changed its triggering ALERT evidence")
        }
        return Ok(existing);
    }
    let mut active: Option<(RepetitionV2, PublisherProgressV2)> = None;
    for repetition in RepetitionV2::ALL {
        let Some(progress) = read_external_root_optional::<PublisherProgressV2>(
            repetition,
            PublisherArtifactV2::Progress,
        )?
        else {
            continue;
        };
        progress.validate(repetition)?;
        if progress.stage != PublisherStageV2::RestorationComplete {
            if active.is_some() {
                bail!("publisher exit86 has more than one active repetition")
            }
            active = Some((repetition, progress));
        }
    }
    let (repetition, progress) = active.context(
        "publisher exit86 lacks one exact active validated publisher progress observation",
    )?;
    if repetition != expected_repetition {
        bail!("terminal SecurityAgent ALERT repetition differs from active publisher state")
    }
    let creation = read_external_root_optional::<SurrogateCreationReceiptV2>(
        repetition,
        PublisherArtifactV2::CreationReceipt,
    )?;
    if let Some(value) = &creation {
        value.validate(repetition)?;
    }
    let cursor = PublisherUiStopCursorV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-publisher-ui-stop"
            .to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        exit_code: SECURITYAGENT_ALERT_EXIT_CODE,
        progress_sha256: document_sha256_v2(&progress)?,
        progress_stage: progress.stage,
        creation_receipt_sha256: creation.as_ref().map(document_sha256_v2).transpose()?,
        failure_classification: EmergencyFailureClassificationV2::SecurityAgentAlertExit86,
        trigger_origin,
        triggering_securityagent_evidence_sha256: evidence_sha256,
        publisher_exit,
        harness_exit,
        publisher_pid,
        harness_process_group_id,
        publisher_live_peer,
        harness_live_peer,
    };
    validate_publisher_ui_stop_cursor(&cursor)?;
    write_runner_receipt(PUBLISHER_UI_STOP_CURSOR_NAME, &canonical_bytes_v2(&cursor)?)?;
    Ok(cursor)
}

fn validate_publisher_ui_stop_cursor(value: &PublisherUiStopCursorV2) -> Result<RepetitionV2> {
    let repetition = RepetitionV2::ALL
        .into_iter()
        .find(|candidate| candidate.ordinal() == value.repetition)
        .context("publisher UI-stop cursor repetition is outside the closed set")?;
    repetition.validate_binding(value.repetition, &value.scope_id)?;
    let trigger_shape_valid = match value.trigger_origin {
        PublisherUiTriggerOriginV2::PublisherProcess => {
            value.publisher_exit.as_ref().is_some_and(|exit| {
                !exit.success && exit.exit_code == Some(SECURITYAGENT_ALERT_EXIT_CODE)
            }) && value.publisher_live_peer.is_none()
                && (value.harness_exit.is_some() ^ value.harness_live_peer.is_some())
                && value.harness_live_peer.as_ref().is_none_or(|peer| {
                    peer.kind == GeneralFailureLivePeerKindV2::HarnessProcessGroup
                })
        }
        PublisherUiTriggerOriginV2::RunnerObservedControl => {
            value.publisher_exit.is_none()
                && value.harness_exit.is_none()
                && value
                    .publisher_live_peer
                    .as_ref()
                    .is_some_and(|peer| peer.kind == GeneralFailureLivePeerKindV2::PublisherProcess)
                && value.harness_live_peer.as_ref().is_some_and(|peer| {
                    peer.kind == GeneralFailureLivePeerKindV2::HarnessProcessGroup
                })
        }
    };
    if value.schema_owner != "substrate.r3-macos-disposable-experiment-runner-publisher-ui-stop"
        || value.schema_version != 2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.exit_code != SECURITYAGENT_ALERT_EXIT_CODE
        || value.progress_stage == PublisherStageV2::RestorationComplete
        || value.failure_classification
            != EmergencyFailureClassificationV2::SecurityAgentAlertExit86
        || !is_sha256(&value.triggering_securityagent_evidence_sha256)
        || value.publisher_pid <= 0
        || value.harness_process_group_id <= 0
        || !trigger_shape_valid
        || !is_sha256(&value.progress_sha256)
        || value
            .creation_receipt_sha256
            .as_ref()
            .is_some_and(|digest| !is_sha256(digest))
    {
        bail!("publisher UI-stop cursor changed its exact terminal exit86 shape")
    }
    for peer in value
        .publisher_live_peer
        .iter()
        .chain(value.harness_live_peer.iter())
    {
        if peer.pid <= 0
            || !is_sha256(&peer.process_start_identity_sha256)
            || !is_sha256(&peer.executable_identity_sha256)
            || !Path::new(&peer.executable_path).is_absolute()
            || (peer.kind == GeneralFailureLivePeerKindV2::HarnessProcessGroup
                && (peer.process_group_id != Some(peer.pid)
                    || peer.process_group_members.is_empty()))
            || (peer.kind == GeneralFailureLivePeerKindV2::PublisherProcess
                && (!peer.process_group_members.is_empty() || peer.process_group_id.is_some()))
        {
            bail!("publisher UI-stop cursor contains an invalid live peer")
        }
    }
    let evidence: SecurityAgentArmEvidenceV2 =
        read_runner_private_optional(PUBLISHER_UI_TRIGGER_EVIDENCE_NAME)?
            .context("publisher UI-stop cursor lacks its raw terminal ALERT evidence")?;
    evidence.validate_terminal_alert()?;
    if document_sha256_v2(&evidence)? != value.triggering_securityagent_evidence_sha256 {
        bail!("publisher UI-stop cursor differs from its raw terminal ALERT evidence")
    }
    Ok(repetition)
}

fn complete_publisher_ui_terminal_rollback(
    inputs: &FrozenRunnerInputs,
    harness_termination: &HarnessTerminationObservationV2,
    publisher_termination: &HarnessTerminationObservationV2,
) -> Result<()> {
    let cursor: PublisherUiStopCursorV2 =
        read_runner_private_optional(PUBLISHER_UI_STOP_CURSOR_NAME)?
            .context("publisher UI terminal rollback lacks its durable cursor")?;
    let repetition = validate_publisher_ui_stop_cursor(&cursor)?;
    if let Some(receipt) = read_runner_private_optional::<PublisherUiTerminalReceiptV2>(
        PUBLISHER_UI_TERMINAL_RECEIPT_NAME,
    )? {
        validate_publisher_ui_terminal_receipt(
            &receipt,
            &cursor,
            harness_termination,
            publisher_termination,
        )?;
        bail!("publisher SecurityAgent ALERT was already restored and is terminal")
    }

    let progress: PublisherProgressV2 =
        read_external_root_optional(repetition, PublisherArtifactV2::Progress)?
            .context("publisher UI terminal rollback lost exact progress")?;
    progress.validate(repetition)?;
    if document_sha256_v2(&progress)? != cursor.progress_sha256
        || progress.stage != cursor.progress_stage
    {
        bail!("publisher progressed after the terminal SecurityAgent ALERT")
    }
    let creation = read_external_root_optional::<SurrogateCreationReceiptV2>(
        repetition,
        PublisherArtifactV2::CreationReceipt,
    )?;
    if let Some(value) = &creation {
        value.validate(repetition)?;
    }
    if creation.as_ref().map(document_sha256_v2).transpose()? != cursor.creation_receipt_sha256 {
        bail!("publisher creation identity changed after the terminal SecurityAgent ALERT")
    }
    let marker = build_ui_alert_emergency_marker(repetition, &cursor, creation.as_ref())?;
    let marker_bytes = marker.bytes()?;
    write_external_harness_input(repetition, marker.artifact(), &marker_bytes)?;
    let marker_sha256 = sha256_hex_v2(&marker_bytes);
    let prepared = PublisherUiRollbackPreparedV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-ui-rollback-prepared"
            .to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        ui_stop_cursor_sha256: document_sha256_v2(&cursor)?,
        rollback_marker_sha256: marker_sha256.clone(),
    };
    if let Some(existing) = read_runner_private_optional::<PublisherUiRollbackPreparedV2>(
        PUBLISHER_UI_ROLLBACK_PREPARED_NAME,
    )? {
        if existing != prepared {
            bail!("publisher UI rollback prepared cursor changed")
        }
        if let Some(rollback) = read_external_root_optional::<EmergencyRollbackReceiptV2>(
            repetition,
            PublisherArtifactV2::EmergencyRollbackReceipt,
        )? {
            let process: PublisherUiRollbackProcessV2 =
                read_runner_private_optional(PUBLISHER_UI_ROLLBACK_PROCESS_NAME)?
                    .context("completed UI rollback lacks its publisher process attestation")?;
            return finalize_publisher_ui_terminal_receipt(
                &cursor,
                harness_termination,
                publisher_termination,
                &prepared,
                &process,
                &marker,
                &rollback,
            );
        }
        bail!("publisher UI rollback was Invoked without a durable result; refusing reinvocation")
    }
    write_runner_receipt(
        PUBLISHER_UI_ROLLBACK_PREPARED_NAME,
        &canonical_bytes_v2(&prepared)?,
    )?;

    let mut rollback_child = spawn_publisher_rollback_only(inputs, "publisher_ui_rollback")?;
    let rollback_process =
        attest_rollback_publisher_process(&rollback_child, inputs, repetition, &cursor, &prepared)?;
    write_runner_receipt(
        PUBLISHER_UI_ROLLBACK_PROCESS_NAME,
        &canonical_bytes_v2(&rollback_process)?,
    )?;
    let rollback_status = rollback_child.wait()?;
    if rollback_status.code() == Some(SECURITYAGENT_ALERT_EXIT_CODE) {
        bail!(
            "SecurityAgent ALERT recurred during the one rollback-only publisher invocation; terminal no-retry"
        )
    }
    if !rollback_status.success() {
        bail!(
            "one rollback-only publisher invocation failed with {:?}; terminal no-retry",
            rollback_status.code()
        )
    }
    let rollback: EmergencyRollbackReceiptV2 =
        read_external_root_optional(repetition, PublisherArtifactV2::EmergencyRollbackReceipt)?
            .context("rollback-only publisher exited without its shared emergency receipt")?;
    finalize_publisher_ui_terminal_receipt(
        &cursor,
        harness_termination,
        publisher_termination,
        &prepared,
        &rollback_process,
        &marker,
        &rollback,
    )
}

fn build_ui_alert_emergency_marker(
    repetition: RepetitionV2,
    cursor: &PublisherUiStopCursorV2,
    creation: Option<&SurrogateCreationReceiptV2>,
) -> Result<InstalledEmergencyMarkerV2> {
    let failure_observation_sha256 = document_sha256_v2(cursor)?;
    if let Some(creation) = creation {
        let marker = EmergencyRollbackMarkerV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_owned(),
            last_durable_stage: cursor.progress_stage,
            failure_classification: EmergencyFailureClassificationV2::SecurityAgentAlertExit86,
            failure_observation_sha256,
            target_identity_sha256: creation.target.identity_sha256.clone(),
            wrong_identity_sha256: creation.wrong.identity_sha256.clone(),
        };
        marker.validate(repetition, creation)?;
        Ok(InstalledEmergencyMarkerV2::Created(marker))
    } else {
        let marker = PreCreationEmergencyRollbackMarkerV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_owned(),
            target_label: format!("{}:signing-key", repetition.scope_id()),
            wrong_label: format!("{}:wrong-surrogate-signing-key", repetition.scope_id()),
            failure_classification: EmergencyFailureClassificationV2::SecurityAgentAlertExit86,
            failure_observation_sha256,
        };
        marker.validate(repetition)?;
        Ok(InstalledEmergencyMarkerV2::PreCreation(marker))
    }
}

fn spawn_publisher_rollback_only(
    inputs: &FrozenRunnerInputs,
    purpose: &'static str,
) -> Result<Child> {
    if !matches!(
        purpose,
        "publisher_ui_rollback" | "general_failure_rollback"
    ) {
        bail!("rollback-only publisher startup purpose changed")
    }
    spawn_observed_publisher(inputs, purpose)
}

fn attest_rollback_publisher_process(
    child: &Child,
    inputs: &FrozenRunnerInputs,
    repetition: RepetitionV2,
    cursor: &PublisherUiStopCursorV2,
    prepared: &PublisherUiRollbackPreparedV2,
) -> Result<PublisherUiRollbackProcessV2> {
    let pid = i32::try_from(child.id()).context("rollback publisher PID exceeds i32")?;
    let process = process_info(pid)?;
    if process.uid != 0
        || canonical_account(process.uid)?.0 != "root"
        || pid_path(pid)? != Path::new(DISPOSABLE_PUBLISHER_EXECUTABLE_PATH)
    {
        bail!("rollback-only publisher process identity changed after exec")
    }
    let measured = measure_frozen_executable(inputs.publisher)?;
    Ok(PublisherUiRollbackProcessV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-ui-rollback-process"
            .to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        ui_stop_cursor_sha256: document_sha256_v2(cursor)?,
        rollback_prepared_sha256: document_sha256_v2(prepared)?,
        pid,
        executable_path: DISPOSABLE_PUBLISHER_EXECUTABLE_PATH.to_owned(),
        effective_uid: process.uid,
        effective_gid: process.gid,
        canonical_account: "root".to_owned(),
        process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
            pid,
            seconds: process.start_seconds,
            microseconds: process.start_microseconds,
        })?,
        executable_identity_sha256: document_sha256_v2(&measured)?,
    })
}

fn finalize_publisher_ui_terminal_receipt(
    cursor: &PublisherUiStopCursorV2,
    harness_termination: &HarnessTerminationObservationV2,
    publisher_termination: &HarnessTerminationObservationV2,
    prepared: &PublisherUiRollbackPreparedV2,
    process: &PublisherUiRollbackProcessV2,
    marker: &InstalledEmergencyMarkerV2,
    rollback: &EmergencyRollbackReceiptV2,
) -> Result<()> {
    let repetition = validate_publisher_ui_stop_cursor(cursor)?;
    marker.validate_receipt(repetition, rollback)?;
    let receipt = PublisherUiTerminalReceiptV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-publisher-ui-terminal"
            .to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        ui_stop_cursor_sha256: document_sha256_v2(cursor)?,
        harness_termination_sha256: document_sha256_v2(harness_termination)?,
        publisher_termination_sha256: document_sha256_v2(publisher_termination)?,
        triggering_securityagent_evidence_sha256: cursor
            .triggering_securityagent_evidence_sha256
            .clone(),
        rollback_marker_sha256: sha256_hex_v2(&marker.bytes()?),
        rollback_prepared_sha256: document_sha256_v2(prepared)?,
        rollback_process_attestation_sha256: document_sha256_v2(process)?,
        emergency_rollback_receipt_sha256: document_sha256_v2(rollback)?,
        securityagent_observation_sha256: rollback.securityagent_observation_sha256.clone(),
        restoration_sha256: rollback.restoration_sha256.clone(),
        terminal_no_retry: true,
    };
    validate_publisher_ui_terminal_receipt(
        &receipt,
        cursor,
        harness_termination,
        publisher_termination,
    )?;
    write_runner_receipt(
        PUBLISHER_UI_TERMINAL_RECEIPT_NAME,
        &canonical_bytes_v2(&receipt)?,
    )?;
    bail!("publisher SecurityAgent ALERT triggered exact rollback and terminal no-retry stop")
}

fn validate_publisher_ui_terminal_receipt(
    value: &PublisherUiTerminalReceiptV2,
    cursor: &PublisherUiStopCursorV2,
    harness_termination: &HarnessTerminationObservationV2,
    publisher_termination: &HarnessTerminationObservationV2,
) -> Result<()> {
    let repetition = validate_publisher_ui_stop_cursor(cursor)?;
    repetition.validate_binding(value.repetition, &value.scope_id)?;
    if value.schema_owner != "substrate.r3-macos-disposable-experiment-runner-publisher-ui-terminal"
        || value.schema_version != 2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.ui_stop_cursor_sha256 != document_sha256_v2(cursor)?
        || value.harness_termination_sha256 != document_sha256_v2(harness_termination)?
        || value.publisher_termination_sha256 != document_sha256_v2(publisher_termination)?
        || value.triggering_securityagent_evidence_sha256
            != cursor.triggering_securityagent_evidence_sha256
        || !harness_termination.process_group_absent_after
        || !publisher_termination.process_group_absent_after
        || !value.terminal_no_retry
    {
        bail!("publisher UI terminal receipt changed its exact cursor/termination binding")
    }
    for digest in [
        &value.rollback_marker_sha256,
        &value.publisher_termination_sha256,
        &value.triggering_securityagent_evidence_sha256,
        &value.rollback_prepared_sha256,
        &value.rollback_process_attestation_sha256,
        &value.emergency_rollback_receipt_sha256,
        &value.securityagent_observation_sha256,
        &value.restoration_sha256,
    ] {
        if !is_sha256(digest) {
            bail!("publisher UI terminal receipt contains a non-SHA-256 binding")
        }
    }
    Ok(())
}

fn recover_general_failure_restoration_if_present(inputs: &FrozenRunnerInputs) -> Result<bool> {
    let Some(cursor) = read_runner_private_optional::<GeneralFailureRestorationCursorV2>(
        GENERAL_FAILURE_CURSOR_NAME,
    )?
    else {
        return Ok(false);
    };
    validate_general_failure_cursor(&cursor)?;
    let termination = ensure_general_failure_peer_terminated(&cursor, None)?;
    let mut accepted = Vec::new();
    let mut request_sha256 = Vec::new();
    for repetition in RepetitionV2::ALL {
        let Some(authority) = observe_general_failure_accepted_journal(repetition)? else {
            continue;
        };
        let request: FinalizationRequestV2 =
            read_external_root_optional(repetition, PublisherArtifactV2::FinalizationRequest)?
                .context(
                    "authenticated accepted journal lacks its immutable finalization request",
                )?;
        substrate_common::macos_retirement_v2::validate_finalization_request_v2(&request, None)?;
        if request.request_digest != authority.request_digest {
            bail!("accepted journal request digest differs from the immutable rejoin request")
        }
        request_sha256.push(document_sha256_v2(&request)?);
        accepted.push(authority);
    }
    if !accepted.is_empty() {
        let rejoin = GeneralFailureAcceptedRejoinV2 {
            schema_owner:
                "substrate.r3-macos-disposable-general-failure-authenticated-accepted-rejoin"
                    .to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            failure_cursor_sha256: document_sha256_v2(&cursor)?,
            peer_termination_sha256: document_sha256_v2(&termination)?,
            accepted,
            same_digest_request_sha256: request_sha256,
            exact_same_digest_rejoin_authorized: true,
            rollback_forbidden: true,
        };
        validate_general_failure_accepted_rejoin(&rejoin, &cursor, &termination)?;
        match read_runner_private_optional::<GeneralFailureAcceptedRejoinV2>(
            GENERAL_FAILURE_ACCEPTED_REJOIN_NAME,
        )? {
            Some(existing) if existing == rejoin => {}
            Some(_) => bail!("authenticated accepted rejoin evidence changed"),
            None => write_runner_receipt(
                GENERAL_FAILURE_ACCEPTED_REJOIN_NAME,
                &canonical_bytes_v2(&rejoin)?,
            )?,
        }
        return Ok(true);
    }
    complete_general_failure_restoration(inputs, &cursor)?;
    Ok(false)
}

fn validate_general_failure_accepted_rejoin(
    value: &GeneralFailureAcceptedRejoinV2,
    cursor: &GeneralFailureRestorationCursorV2,
    termination: &GeneralFailurePeerTerminationV2,
) -> Result<()> {
    if value.schema_owner
        != "substrate.r3-macos-disposable-general-failure-authenticated-accepted-rejoin"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.failure_cursor_sha256 != document_sha256_v2(cursor)?
        || value.peer_termination_sha256 != document_sha256_v2(termination)?
        || value.accepted.is_empty()
        || value.accepted.len() != value.same_digest_request_sha256.len()
        || !value.exact_same_digest_rejoin_authorized
        || !value.rollback_forbidden
    {
        bail!("authenticated accepted rejoin changed its exact cursor authority")
    }
    for (authority, request_sha256) in value.accepted.iter().zip(&value.same_digest_request_sha256)
    {
        let repetition = match authority.repetition {
            1 => RepetitionV2::One,
            2 => RepetitionV2::Two,
            _ => bail!("accepted rejoin contains an invalid repetition"),
        };
        repetition.validate_binding(authority.repetition, &authority.scope_id)?;
        if authority.generation == 0
            || !is_sha256(&authority.head_sha256)
            || !is_sha256(&authority.request_digest)
            || !is_sha256(request_sha256)
        {
            bail!("accepted rejoin contains an invalid authenticated journal binding")
        }
    }
    Ok(())
}

fn terminal_general_failure_from_publisher(
    inputs: &FrozenRunnerInputs,
    publisher_status: ExitStatus,
    harness: &mut Child,
) -> Result<()> {
    let (harness_exit, live_peer) = match harness.try_wait()? {
        Some(status) => (Some(child_exit_observation(&status)), None),
        None => (
            None,
            Some(observe_general_failure_live_peer(
                harness,
                GeneralFailureLivePeerKindV2::HarnessProcessGroup,
                &inputs.peer.harness_identity,
                DISPOSABLE_HARNESS_UID_V2,
            )?),
        ),
    };
    let cursor = GeneralFailureRestorationCursorV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-general-failure-cursor"
            .to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        trigger: GeneralFailureTriggerV2::PublisherClosedFailure,
        publisher_exit: Some(child_exit_observation(&publisher_status)),
        harness_exit,
        live_peer,
        root_operation_ui_stop_cursor_sha256: None,
        terminal_no_resume: true,
    };
    validate_general_failure_cursor(&cursor)?;
    write_runner_receipt(GENERAL_FAILURE_CURSOR_NAME, &canonical_bytes_v2(&cursor)?)?;
    ensure_general_failure_peer_terminated(&cursor, Some(harness))?;
    complete_general_failure_restoration(inputs, &cursor)
}

fn terminal_general_failure_from_harness(
    inputs: &FrozenRunnerInputs,
    harness_status: ExitStatus,
    publisher: &mut Child,
) -> Result<()> {
    let (publisher_exit, live_peer) = match publisher.try_wait()? {
        Some(status) => (Some(child_exit_observation(&status)), None),
        None => (
            None,
            Some(observe_general_failure_live_peer(
                publisher,
                GeneralFailureLivePeerKindV2::PublisherProcess,
                &inputs.prepared.publisher_identity,
                0,
            )?),
        ),
    };
    let cursor = GeneralFailureRestorationCursorV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-general-failure-cursor"
            .to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        trigger: GeneralFailureTriggerV2::HarnessClosedFailure,
        publisher_exit,
        harness_exit: Some(child_exit_observation(&harness_status)),
        live_peer,
        root_operation_ui_stop_cursor_sha256: None,
        terminal_no_resume: true,
    };
    validate_general_failure_cursor(&cursor)?;
    write_runner_receipt(GENERAL_FAILURE_CURSOR_NAME, &canonical_bytes_v2(&cursor)?)?;
    ensure_general_failure_peer_terminated(&cursor, Some(publisher))?;
    complete_general_failure_restoration(inputs, &cursor)
}

fn validate_general_failure_cursor(value: &GeneralFailureRestorationCursorV2) -> Result<()> {
    let trigger_valid = match value.trigger {
        GeneralFailureTriggerV2::PublisherClosedFailure => value
            .publisher_exit
            .as_ref()
            .is_some_and(|status| !status.success),
        GeneralFailureTriggerV2::HarnessClosedFailure => value
            .harness_exit
            .as_ref()
            .is_some_and(|status| !status.success),
        GeneralFailureTriggerV2::RootOperationSecurityAgentAlert => {
            value.publisher_exit.is_none()
                && value.harness_exit.is_none()
                && value.live_peer.is_none()
                && value
                    .root_operation_ui_stop_cursor_sha256
                    .as_ref()
                    .is_some_and(|digest| is_sha256(digest))
        }
    };
    if value.schema_owner
        != "substrate.r3-macos-disposable-experiment-runner-general-failure-cursor"
        || value.schema_version != 2
        || value.experiment_id != EXPERIMENT_ID_V2
        || !value.terminal_no_resume
        || !trigger_valid
        || (value.trigger != GeneralFailureTriggerV2::RootOperationSecurityAgentAlert
            && value.root_operation_ui_stop_cursor_sha256.is_some())
        || (value.live_peer.is_some()
            && value.publisher_exit.is_some()
            && value.harness_exit.is_some())
    {
        bail!("general-failure restoration cursor is not one terminal child failure")
    }
    if let Some(peer) = &value.live_peer {
        if peer.pid <= 0
            || peer.supplementary_groups.validate_empty().is_err()
            || peer
                .process_group_members
                .iter()
                .any(|member| member.supplementary_groups.validate_empty().is_err())
            || !Path::new(&peer.executable_path).is_absolute()
            || !is_sha256(&peer.process_start_identity_sha256)
            || !is_sha256(&peer.executable_identity_sha256)
            || match peer.kind {
                GeneralFailureLivePeerKindV2::PublisherProcess => {
                    peer.process_group_id.is_some() || !peer.process_group_members.is_empty()
                }
                GeneralFailureLivePeerKindV2::HarnessProcessGroup => {
                    peer.process_group_id != Some(peer.pid)
                        || peer.process_group_members.is_empty()
                        || !peer
                            .process_group_members
                            .iter()
                            .any(|member| member.pid == peer.pid)
                }
            }
        {
            bail!("general-failure cursor live-peer identity changed")
        }
    }
    Ok(())
}

fn child_exit_observation(status: &ExitStatus) -> ChildExitObservationV2 {
    ChildExitObservationV2 {
        success: status.success(),
        exit_code: status.code(),
        terminating_signal: status.signal(),
    }
}

fn observe_general_failure_live_peer(
    child: &Child,
    kind: GeneralFailureLivePeerKindV2,
    identity: &ExecutableIdentityV2,
    expected_uid: u32,
) -> Result<GeneralFailureLivePeerV2> {
    let pid = i32::try_from(child.id()).context("general-failure live-peer PID exceeds i32")?;
    let process = process_info(pid)?;
    let path = pid_path(pid)?;
    if process.uid != expected_uid || path != Path::new(&identity.intended_path) {
        bail!("general-failure live peer changed before terminal cursor durability")
    }
    let process_group_members = if kind == GeneralFailureLivePeerKindV2::HarnessProcessGroup {
        observe_process_group_members(pid)?
    } else {
        Vec::new()
    };
    Ok(GeneralFailureLivePeerV2 {
        kind,
        pid,
        process_group_id: match kind {
            GeneralFailureLivePeerKindV2::PublisherProcess => None,
            GeneralFailureLivePeerKindV2::HarnessProcessGroup => {
                if process.process_group_id != pid {
                    bail!("harness is not the exact leader of its sealed process group")
                }
                Some(process.process_group_id)
            }
        },
        process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
            pid,
            seconds: process.start_seconds,
            microseconds: process.start_microseconds,
        })?,
        executable_path: identity.intended_path.clone(),
        executable_identity_sha256: document_sha256_v2(identity)?,
        effective_uid: process.uid,
        supplementary_groups: measured_child_supplementary_groups_v2(pid)?,
        process_group_members,
    })
}

fn persist_root_operation_live_peers(
    publisher: &Child,
    harness: &Child,
    inputs: &FrozenRunnerInputs,
) -> Result<()> {
    let value = RootOperationLivePeersV2 {
        schema_owner: "substrate.r3-macos-disposable-root-operation-live-peers".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        publisher: observe_general_failure_live_peer(
            publisher,
            GeneralFailureLivePeerKindV2::PublisherProcess,
            &inputs.prepared.publisher_identity,
            0,
        )?,
        harness: observe_general_failure_live_peer(
            harness,
            GeneralFailureLivePeerKindV2::HarnessProcessGroup,
            &inputs.peer.harness_identity,
            DISPOSABLE_HARNESS_UID_V2,
        )?,
    };
    validate_root_operation_live_peers(&value)?;
    match read_runner_private_optional::<RootOperationLivePeersV2>(
        ROOT_OPERATION_UI_LIVE_PEERS_NAME,
    )? {
        Some(existing) if existing == value => Ok(()),
        Some(_) => bail!("root-operation live-peer authority already belongs to another process"),
        None => write_runner_receipt(
            ROOT_OPERATION_UI_LIVE_PEERS_NAME,
            &canonical_bytes_v2(&value)?,
        ),
    }
}

fn validate_root_operation_live_peers(value: &RootOperationLivePeersV2) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-root-operation-live-peers"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.publisher.kind != GeneralFailureLivePeerKindV2::PublisherProcess
        || value.harness.kind != GeneralFailureLivePeerKindV2::HarnessProcessGroup
    {
        bail!("root-operation live-peer authority changed")
    }
    validate_root_operation_live_peer(&value.publisher)?;
    validate_root_operation_live_peer(&value.harness)
}

fn validate_root_operation_live_peer(peer: &GeneralFailureLivePeerV2) -> Result<()> {
    if peer.pid <= 0
        || !Path::new(&peer.executable_path).is_absolute()
        || !is_sha256(&peer.process_start_identity_sha256)
        || !is_sha256(&peer.executable_identity_sha256)
        || match peer.kind {
            GeneralFailureLivePeerKindV2::PublisherProcess => {
                peer.effective_uid != 0
                    || peer.supplementary_groups.validate_empty().is_err()
                    || peer.process_group_id.is_some()
                    || !peer.process_group_members.is_empty()
            }
            GeneralFailureLivePeerKindV2::HarnessProcessGroup => {
                peer.effective_uid != DISPOSABLE_HARNESS_UID_V2
                    || peer.supplementary_groups.validate_empty().is_err()
                    || peer.process_group_id != Some(peer.pid)
                    || peer.process_group_members.is_empty()
            }
        }
    {
        bail!("root-operation live-peer identity changed")
    }
    Ok(())
}

fn refresh_root_operation_live_peers(
    value: &RootOperationLivePeersV2,
) -> Result<Vec<GeneralFailureLivePeerV2>> {
    validate_root_operation_live_peers(value)?;
    let mut refreshed = Vec::with_capacity(2);
    // SAFETY: signal zero is a read-only existence probe for the exact recorded publisher PID.
    if unsafe { libc::kill(value.publisher.pid, 0) } == 0 {
        validate_recovered_general_failure_leader(&value.publisher)?;
        refreshed.push(value.publisher.clone());
    } else if std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
        return Err(std::io::Error::last_os_error())
            .context("probe root-operation publisher before ALERT cursor");
    }
    let pgid = value
        .harness
        .process_group_id
        .context("root-operation harness lacks its process group")?;
    let members = observe_process_group_members(pgid)?;
    if !members.is_empty() {
        for member in &members {
            let path = member.executable_path.as_str();
            if member.effective_uid != DISPOSABLE_HARNESS_UID_V2
                || !matches!(
                    path,
                    DISPOSABLE_HARNESS_PATH_V2
                        | MAC_R3_COORDINATOR_PATH_V2
                        | PEER_CODE_PROBE_PATH_V2
                        | ALTERNATE_COORDINATOR_PATH_V2
                        | NOBODY_OWNER_PROBE_PATH_V2
                )
            {
                bail!("root-operation harness group contains an unsealed process identity")
            }
        }
        if members.iter().any(|member| member.pid == value.harness.pid) {
            validate_recovered_general_failure_leader(&value.harness)?;
        }
        let mut harness = value.harness.clone();
        harness.process_group_members = members;
        refreshed.push(harness);
    }
    Ok(refreshed)
}

fn ensure_general_failure_peer_terminated(
    cursor: &GeneralFailureRestorationCursorV2,
    child: Option<&mut Child>,
) -> Result<GeneralFailurePeerTerminationV2> {
    if let Some(existing) = read_runner_private_optional::<GeneralFailurePeerTerminationV2>(
        GENERAL_FAILURE_PEER_TERMINATION_NAME,
    )? {
        validate_general_failure_peer_termination(&existing, cursor)?;
        return Ok(existing);
    }
    let (sigterm_sent, sigkill_sent, exit_observation, recovered_without_child_handle) =
        match (&cursor.live_peer, child) {
            (None, None) => (false, false, None, false),
            (None, Some(_)) => bail!("general-failure termination received an unbound child"),
            (Some(peer), Some(child)) => {
                if i32::try_from(child.id())? != peer.pid {
                    bail!("general-failure live child PID changed from its terminal cursor")
                }
                let termination = match peer.kind {
                    GeneralFailureLivePeerKindV2::HarnessProcessGroup => {
                        terminate_harness_process_group(child)?
                    }
                    GeneralFailureLivePeerKindV2::PublisherProcess => {
                        terminate_single_child(child)?
                    }
                };
                (
                    termination.sigterm_sent,
                    termination.sigkill_sent,
                    Some(ChildExitObservationV2 {
                        success: false,
                        exit_code: termination.exit_code,
                        terminating_signal: termination.terminating_signal,
                    }),
                    false,
                )
            }
            (Some(peer), None) => {
                let (sigterm, sigkill) = terminate_recovered_live_peer(peer)?;
                (sigterm, sigkill, None, true)
            }
        };
    let receipt = GeneralFailurePeerTerminationV2 {
        schema_owner: "substrate.r3-macos-disposable-general-failure-peer-termination".to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        cursor_sha256: document_sha256_v2(cursor)?,
        live_peer_sha256: cursor
            .live_peer
            .as_ref()
            .map(document_sha256_v2)
            .transpose()?,
        sigterm_sent,
        sigkill_sent,
        exit_observation,
        recovered_without_child_handle,
        peer_absent_after: true,
    };
    validate_general_failure_peer_termination(&receipt, cursor)?;
    write_runner_receipt(
        GENERAL_FAILURE_PEER_TERMINATION_NAME,
        &canonical_bytes_v2(&receipt)?,
    )?;
    Ok(receipt)
}

fn validate_general_failure_peer_termination(
    value: &GeneralFailurePeerTerminationV2,
    cursor: &GeneralFailureRestorationCursorV2,
) -> Result<()> {
    validate_general_failure_cursor(cursor)?;
    if value.schema_owner != "substrate.r3-macos-disposable-general-failure-peer-termination"
        || value.schema_version != 2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.cursor_sha256 != document_sha256_v2(cursor)?
        || value.live_peer_sha256
            != cursor
                .live_peer
                .as_ref()
                .map(document_sha256_v2)
                .transpose()?
        || !value.peer_absent_after
        || (cursor.live_peer.is_none()
            && (value.sigterm_sent
                || value.sigkill_sent
                || value.exit_observation.is_some()
                || value.recovered_without_child_handle))
    {
        bail!("general-failure peer-termination receipt changed")
    }
    Ok(())
}

fn terminate_single_child(child: &mut Child) -> Result<HarnessTerminationObservationV2> {
    let pid = i32::try_from(child.id()).context("publisher PID exceeds i32")?;
    if let Some(status) = child.try_wait()? {
        return Ok(HarnessTerminationObservationV2 {
            schema_owner: "substrate.r3-macos-disposable-experiment-runner-child-termination"
                .to_owned(),
            schema_version: 2,
            process_group_id: pid,
            sigterm_sent: false,
            sigkill_sent: false,
            exit_code: status.code(),
            terminating_signal: status.signal(),
            process_group_absent_after: true,
        });
    }
    // SAFETY: the PID is the exact still-owned publisher child recorded before the call.
    if unsafe { libc::kill(pid, libc::SIGTERM) } != 0 {
        return Err(std::io::Error::last_os_error()).context("terminate exact publisher child");
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Ok(HarnessTerminationObservationV2 {
                schema_owner: "substrate.r3-macos-disposable-experiment-runner-child-termination"
                    .to_owned(),
                schema_version: 2,
                process_group_id: pid,
                sigterm_sent: true,
                sigkill_sent: false,
                exit_code: status.code(),
                terminating_signal: status.signal(),
                process_group_absent_after: true,
            });
        }
        thread::sleep(Duration::from_millis(10));
    }
    // SAFETY: same exact still-owned child after the bounded SIGTERM deadline.
    if unsafe { libc::kill(pid, libc::SIGKILL) } != 0 {
        return Err(std::io::Error::last_os_error()).context("kill exact publisher child");
    }
    let status = child.wait()?;
    Ok(HarnessTerminationObservationV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-child-termination"
            .to_owned(),
        schema_version: 2,
        process_group_id: pid,
        sigterm_sent: true,
        sigkill_sent: true,
        exit_code: status.code(),
        terminating_signal: status.signal(),
        process_group_absent_after: true,
    })
}

fn terminate_recovered_live_peer(peer: &GeneralFailureLivePeerV2) -> Result<(bool, bool)> {
    if peer.kind == GeneralFailureLivePeerKindV2::HarnessProcessGroup {
        let pgid = peer
            .process_group_id
            .context("recovered harness cursor lacks its exact process group")?;
        let members = validate_recovered_process_group_members(peer)?;
        if members.is_empty() {
            return Ok((false, false));
        }
        if members.iter().any(|member| member.pid == peer.pid) {
            validate_recovered_general_failure_leader(peer)?;
        }
        let sigkill_sent = signal_exact_process_members_and_wait_absent(pgid, &members)?;
        return Ok((true, sigkill_sent));
    }
    // SAFETY: signal 0 performs a read-only existence/permission probe.
    if unsafe { libc::kill(peer.pid, 0) } != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            return Ok((false, false));
        }
        return Err(error).context("probe recovered general-failure live peer");
    }
    validate_recovered_general_failure_leader(peer)?;
    let target = peer.pid;
    // SAFETY: exact PID/start/path identity was revalidated immediately above.
    if unsafe { libc::kill(target, libc::SIGTERM) } != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error).context("terminate recovered general-failure live peer");
        }
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        // SAFETY: read-only existence check for the exact recorded process leader.
        if unsafe { libc::kill(peer.pid, 0) } != 0
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        {
            return Ok((true, false));
        }
        thread::sleep(Duration::from_millis(10));
    }
    // SAFETY: exact identity remained live through the bounded deadline.
    if unsafe { libc::kill(target, libc::SIGKILL) } != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error).context("kill recovered general-failure live peer");
        }
    }
    Ok((true, true))
}

fn validate_recovered_general_failure_leader(peer: &GeneralFailureLivePeerV2) -> Result<()> {
    let process = process_info(peer.pid)?;
    let start = document_sha256_v2(&ProcessStartJoinV2 {
        pid: peer.pid,
        seconds: process.start_seconds,
        microseconds: process.start_microseconds,
    })?;
    if process.uid != peer.effective_uid
        || start != peer.process_start_identity_sha256
        || pid_path(peer.pid)? != Path::new(&peer.executable_path)
        || peer
            .process_group_id
            .is_some_and(|pgid| process.process_group_id != pgid)
    {
        bail!("recovered general-failure PID was reused or substituted; refusing signal")
    }
    Ok(())
}

fn complete_general_failure_rollback(
    inputs: &FrozenRunnerInputs,
    cursor: &GeneralFailureRestorationCursorV2,
) -> Result<GeneralFailureRollbackResultV2> {
    let termination: GeneralFailurePeerTerminationV2 =
        read_runner_private_optional(GENERAL_FAILURE_PEER_TERMINATION_NAME)?
            .context("general-failure rollback lacks peer-termination durability")?;
    validate_general_failure_peer_termination(&termination, cursor)?;
    let expected_prepared = build_general_failure_rollback_prepared(cursor, &termination)?;
    let prepared = if let Some(existing) = read_runner_private_optional::<
        GeneralFailureRollbackPreparedV2,
    >(GENERAL_FAILURE_ROLLBACK_PREPARED_NAME)?
    {
        if existing != expected_prepared {
            bail!("general-failure rollback prepared plan changed during recovery")
        }
        existing
    } else {
        write_runner_receipt(
            GENERAL_FAILURE_ROLLBACK_PREPARED_NAME,
            &canonical_bytes_v2(&expected_prepared)?,
        )?;
        expected_prepared
    };
    validate_general_failure_rollback_prepared(&prepared, cursor, &termination)?;
    if let Some(existing) = read_runner_private_optional::<GeneralFailureRollbackResultV2>(
        GENERAL_FAILURE_ROLLBACK_RESULT_NAME,
    )? {
        validate_general_failure_rollback_result(&existing, &prepared)?;
        if !existing.exact_restoration_complete {
            bail!("general-failure rollback previously reached a terminal non-restored result")
        }
        return Ok(existing);
    }
    if prepared.accepted_authority_ambiguity {
        let result = GeneralFailureRollbackResultV2 {
            schema_owner: "substrate.r3-macos-disposable-general-failure-rollback-result"
                .to_owned(),
            schema_version: 2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            prepared_sha256: document_sha256_v2(&prepared)?,
            invoked_sha256: None,
            publisher_exit: None,
            emergency_receipt_sha256: Vec::new(),
            reconstructed_after_invoked: false,
            exact_restoration_complete: false,
            accepted_authority_ambiguity: true,
            terminal_no_reinvoke: true,
        };
        validate_general_failure_rollback_result(&result, &prepared)?;
        write_runner_receipt(
            GENERAL_FAILURE_ROLLBACK_RESULT_NAME,
            &canonical_bytes_v2(&result)?,
        )?;
        bail!("general-failure rollback preserved possible accepted authority without mutation")
    }
    if !prepared.rollback_publisher_invocation_required {
        let emergency_receipt_sha256 = collect_general_failure_emergency_receipts(&prepared)?;
        let result = GeneralFailureRollbackResultV2 {
            schema_owner: "substrate.r3-macos-disposable-general-failure-rollback-result"
                .to_owned(),
            schema_version: 2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            prepared_sha256: document_sha256_v2(&prepared)?,
            invoked_sha256: None,
            publisher_exit: None,
            emergency_receipt_sha256,
            reconstructed_after_invoked: false,
            exact_restoration_complete: true,
            accepted_authority_ambiguity: false,
            terminal_no_reinvoke: true,
        };
        validate_general_failure_rollback_result(&result, &prepared)?;
        write_runner_receipt(
            GENERAL_FAILURE_ROLLBACK_RESULT_NAME,
            &canonical_bytes_v2(&result)?,
        )?;
        return Ok(result);
    }

    let expected_invoked = GeneralFailureRollbackInvokedV2 {
        schema_owner: "substrate.r3-macos-disposable-general-failure-rollback-invoked".to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        prepared_sha256: document_sha256_v2(&prepared)?,
        invocation_ordinal: 1,
        publisher_executable_identity_sha256: document_sha256_v2(
            &inputs.prepared.publisher_identity,
        )?,
        invocation_may_begin: true,
        normal_stage_retry_forbidden: true,
    };
    let invoked = if let Some(existing) = read_runner_private_optional::<
        GeneralFailureRollbackInvokedV2,
    >(GENERAL_FAILURE_ROLLBACK_INVOKED_NAME)?
    {
        if existing != expected_invoked {
            bail!("general-failure rollback invoked cursor changed")
        }
        let receipts = collect_general_failure_emergency_receipts(&prepared);
        let Ok(emergency_receipt_sha256) = receipts else {
            bail!("general-failure rollback was Invoked without complete receipts; refusing reinvocation")
        };
        let result = GeneralFailureRollbackResultV2 {
            schema_owner: "substrate.r3-macos-disposable-general-failure-rollback-result"
                .to_owned(),
            schema_version: 2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            prepared_sha256: document_sha256_v2(&prepared)?,
            invoked_sha256: Some(document_sha256_v2(&existing)?),
            publisher_exit: None,
            emergency_receipt_sha256,
            reconstructed_after_invoked: true,
            exact_restoration_complete: true,
            accepted_authority_ambiguity: false,
            terminal_no_reinvoke: true,
        };
        validate_general_failure_rollback_result(&result, &prepared)?;
        write_runner_receipt(
            GENERAL_FAILURE_ROLLBACK_RESULT_NAME,
            &canonical_bytes_v2(&result)?,
        )?;
        return Ok(result);
    } else {
        write_runner_receipt(
            GENERAL_FAILURE_ROLLBACK_INVOKED_NAME,
            &canonical_bytes_v2(&expected_invoked)?,
        )?;
        expected_invoked
    };

    let mut rollback = spawn_publisher_rollback_only(inputs, "general_failure_rollback")?;
    let status = rollback.wait()?;
    let publisher_exit = child_exit_observation(&status);
    let receipts = if status.success() {
        collect_general_failure_emergency_receipts(&prepared)
    } else {
        Err(anyhow::anyhow!(
            "rollback-only publisher failed with {:?}",
            status.code()
        ))
    };
    let exact_restoration_complete = receipts.is_ok();
    let result = GeneralFailureRollbackResultV2 {
        schema_owner: "substrate.r3-macos-disposable-general-failure-rollback-result".to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        prepared_sha256: document_sha256_v2(&prepared)?,
        invoked_sha256: Some(document_sha256_v2(&invoked)?),
        publisher_exit: Some(publisher_exit),
        emergency_receipt_sha256: receipts.unwrap_or_default(),
        reconstructed_after_invoked: false,
        exact_restoration_complete,
        accepted_authority_ambiguity: false,
        terminal_no_reinvoke: true,
    };
    validate_general_failure_rollback_result(&result, &prepared)?;
    write_runner_receipt(
        GENERAL_FAILURE_ROLLBACK_RESULT_NAME,
        &canonical_bytes_v2(&result)?,
    )?;
    if !result.exact_restoration_complete {
        if result
            .publisher_exit
            .as_ref()
            .and_then(|value| value.exit_code)
            == Some(SECURITYAGENT_ALERT_EXIT_CODE)
        {
            bail!("SecurityAgent ALERT occurred during general-failure rollback; terminal no-reinvoke")
        }
        bail!("general-failure rollback-only publisher failed; terminal no-reinvoke")
    }
    Ok(result)
}

fn build_general_failure_rollback_prepared(
    cursor: &GeneralFailureRestorationCursorV2,
    termination: &GeneralFailurePeerTerminationV2,
) -> Result<GeneralFailureRollbackPreparedV2> {
    let mut plan = Vec::with_capacity(RepetitionV2::ALL.len());
    for repetition in RepetitionV2::ALL {
        let progress = read_external_root_optional::<PublisherProgressV2>(
            repetition,
            PublisherArtifactV2::Progress,
        )?;
        if let Some(value) = &progress {
            value.validate(repetition)?;
        }
        let creation = read_external_root_optional::<SurrogateCreationReceiptV2>(
            repetition,
            PublisherArtifactV2::CreationReceipt,
        )?;
        if let Some(value) = &creation {
            value.validate(repetition)?;
        }
        if progress.is_none() && creation.is_some() {
            bail!("general-failure rollback found creation without durable publisher progress")
        }
        let progress_sha256 = progress.as_ref().map(document_sha256_v2).transpose()?;
        let progress_stage = progress.as_ref().map(|value| value.stage);
        let creation_receipt_sha256 = creation.as_ref().map(document_sha256_v2).transpose()?;
        let accepted_journal = observe_general_failure_accepted_journal(repetition)?;
        let (disposition, marker_artifact, marker_sha256) =
            if progress_stage == Some(PublisherStageV2::RestorationComplete) {
                (GeneralFailureRollbackDispositionV2::Completed, None, None)
            } else if accepted_journal.is_some() {
                (
                    GeneralFailureRollbackDispositionV2::AcceptedAuthorityAmbiguous,
                    None,
                    None,
                )
            } else {
                let marker = build_general_failure_emergency_marker(
                    repetition,
                    cursor,
                    progress.as_ref(),
                    creation.as_ref(),
                )?;
                let marker_bytes = marker.bytes()?;
                write_external_harness_input(repetition, marker.artifact(), &marker_bytes)?;
                let already_restored = read_external_root_optional::<EmergencyRollbackReceiptV2>(
                    repetition,
                    PublisherArtifactV2::EmergencyRollbackReceipt,
                )?;
                if let Some(receipt) = &already_restored {
                    marker.validate_receipt(repetition, receipt)?;
                }
                let disposition = if already_restored.is_some() {
                    GeneralFailureRollbackDispositionV2::AlreadyEmergencyRestored
                } else if creation.is_some() {
                    GeneralFailureRollbackDispositionV2::RollbackCreated
                } else {
                    GeneralFailureRollbackDispositionV2::RollbackPrecreation
                };
                (
                    disposition,
                    Some(marker.artifact()),
                    Some(sha256_hex_v2(&marker_bytes)),
                )
            };
        plan.push(GeneralFailureRollbackPlanEntryV2 {
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_owned(),
            progress_sha256,
            progress_stage,
            creation_receipt_sha256,
            accepted_journal_generation: accepted_journal
                .as_ref()
                .map(|journal| journal.generation),
            accepted_journal_head_sha256: accepted_journal
                .as_ref()
                .map(|journal| journal.head_sha256.clone()),
            accepted_request_digest: accepted_journal
                .as_ref()
                .map(|journal| journal.request_digest.clone()),
            disposition,
            rollback_marker_artifact_filename: marker_artifact
                .map(|artifact| artifact.filename().to_owned()),
            rollback_marker_sha256: marker_sha256,
        });
    }
    let accepted_authority_ambiguity = plan.iter().any(|entry| {
        entry.disposition == GeneralFailureRollbackDispositionV2::AcceptedAuthorityAmbiguous
    });
    let rollback_publisher_invocation_required = !accepted_authority_ambiguity
        && plan.iter().any(|entry| {
            matches!(
                entry.disposition,
                GeneralFailureRollbackDispositionV2::RollbackPrecreation
                    | GeneralFailureRollbackDispositionV2::RollbackCreated
            )
        });
    let value = GeneralFailureRollbackPreparedV2 {
        schema_owner: "substrate.r3-macos-disposable-general-failure-rollback-prepared".to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        cursor_sha256: document_sha256_v2(cursor)?,
        peer_termination_sha256: document_sha256_v2(termination)?,
        plan_sha256: document_sha256_v2(&plan)?,
        plan,
        accepted_authority_ambiguity,
        rollback_publisher_invocation_required,
        terminal_no_normal_retry: true,
    };
    validate_general_failure_rollback_prepared(&value, cursor, termination)?;
    Ok(value)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AcceptedJournalAuthorityV2 {
    repetition: u8,
    scope_id: String,
    generation: u64,
    head_sha256: String,
    request_digest: String,
}

/// Classify acceptance from the authenticated finalizer journal, never from publisher progress.
/// An absent root/scope or an exact empty scope is pre-acceptance. Any malformed or indeterminate
/// journal returns an error, preserving the surrogate rather than guessing rollback authority.
fn observe_general_failure_accepted_journal(
    repetition: RepetitionV2,
) -> Result<Option<AcceptedJournalAuthorityV2>> {
    let Some(journal) = LockedJournal::open_existing_fixed(
        Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2),
        repetition.scope_id(),
        0,
    )?
    else {
        return Ok(None);
    };
    let Some(head) = journal.head()? else {
        return Ok(None);
    };
    let first = journal.generation(1)?;
    if first.event.kind != JournalEventKind::FinalizerAccepted
        || first.request_digest != head.record.request_digest
    {
        bail!("nonempty finalizer journal does not prove one exact FinalizerAccepted claim")
    }
    Ok(Some(AcceptedJournalAuthorityV2 {
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        generation: head.generation,
        head_sha256: head.sha256,
        request_digest: head.record.request_digest,
    }))
}

fn build_general_failure_emergency_marker(
    repetition: RepetitionV2,
    cursor: &GeneralFailureRestorationCursorV2,
    progress: Option<&PublisherProgressV2>,
    creation: Option<&SurrogateCreationReceiptV2>,
) -> Result<InstalledEmergencyMarkerV2> {
    let failure_observation_sha256 = document_sha256_v2(cursor)?;
    let failure_classification =
        if cursor.trigger == GeneralFailureTriggerV2::RootOperationSecurityAgentAlert {
            EmergencyFailureClassificationV2::SecurityAgentAlertExit86
        } else {
            EmergencyFailureClassificationV2::ClosedFailure
        };
    if let (Some(progress), Some(creation)) = (progress, creation) {
        let marker = EmergencyRollbackMarkerV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_owned(),
            last_durable_stage: progress.stage,
            failure_classification,
            failure_observation_sha256,
            target_identity_sha256: creation.target.identity_sha256.clone(),
            wrong_identity_sha256: creation.wrong.identity_sha256.clone(),
        };
        marker.validate(repetition, creation)?;
        Ok(InstalledEmergencyMarkerV2::Created(marker))
    } else {
        let marker = PreCreationEmergencyRollbackMarkerV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_owned(),
            target_label: format!("{}:signing-key", repetition.scope_id()),
            wrong_label: format!("{}:wrong-surrogate-signing-key", repetition.scope_id()),
            failure_classification,
            failure_observation_sha256,
        };
        marker.validate(repetition)?;
        Ok(InstalledEmergencyMarkerV2::PreCreation(marker))
    }
}

fn validate_general_failure_rollback_prepared(
    value: &GeneralFailureRollbackPreparedV2,
    cursor: &GeneralFailureRestorationCursorV2,
    termination: &GeneralFailurePeerTerminationV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-general-failure-rollback-prepared"
        || value.schema_version != 2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.cursor_sha256 != document_sha256_v2(cursor)?
        || value.peer_termination_sha256 != document_sha256_v2(termination)?
        || value.plan.len() != RepetitionV2::ALL.len()
        || value.plan_sha256 != document_sha256_v2(&value.plan)?
        || !value.terminal_no_normal_retry
        || value.accepted_authority_ambiguity
            != value.plan.iter().any(|entry| {
                entry.disposition == GeneralFailureRollbackDispositionV2::AcceptedAuthorityAmbiguous
            })
        || (value.accepted_authority_ambiguity && value.rollback_publisher_invocation_required)
    {
        bail!("general-failure rollback prepared plan changed")
    }
    for (entry, repetition) in value.plan.iter().zip(RepetitionV2::ALL) {
        repetition.validate_binding(entry.repetition, &entry.scope_id)?;
        for digest in [
            entry.progress_sha256.as_ref(),
            entry.creation_receipt_sha256.as_ref(),
            entry.accepted_journal_head_sha256.as_ref(),
            entry.accepted_request_digest.as_ref(),
            entry.rollback_marker_sha256.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if !is_sha256(digest) {
                bail!("general-failure rollback plan contains a non-digest binding")
            }
        }
        let marker_required = matches!(
            entry.disposition,
            GeneralFailureRollbackDispositionV2::AlreadyEmergencyRestored
                | GeneralFailureRollbackDispositionV2::RollbackPrecreation
                | GeneralFailureRollbackDispositionV2::RollbackCreated
        );
        if marker_required
            != (entry.rollback_marker_artifact_filename.is_some()
                && entry.rollback_marker_sha256.is_some())
        {
            bail!("general-failure rollback plan marker surface changed")
        }
        let accepted =
            entry.disposition == GeneralFailureRollbackDispositionV2::AcceptedAuthorityAmbiguous;
        if accepted
            != (entry.accepted_journal_generation.is_some()
                && entry.accepted_journal_head_sha256.is_some()
                && entry.accepted_request_digest.is_some())
            || (!accepted
                && (entry.accepted_journal_generation.is_some()
                    || entry.accepted_journal_head_sha256.is_some()
                    || entry.accepted_request_digest.is_some()))
        {
            bail!("general-failure rollback plan changed its authenticated journal authority")
        }
    }
    Ok(())
}

fn collect_general_failure_emergency_receipts(
    prepared: &GeneralFailureRollbackPreparedV2,
) -> Result<Vec<String>> {
    let mut digests = Vec::new();
    for (entry, repetition) in prepared.plan.iter().zip(RepetitionV2::ALL) {
        if matches!(
            entry.disposition,
            GeneralFailureRollbackDispositionV2::AlreadyEmergencyRestored
                | GeneralFailureRollbackDispositionV2::RollbackPrecreation
                | GeneralFailureRollbackDispositionV2::RollbackCreated
        ) {
            let receipt: EmergencyRollbackReceiptV2 = read_external_root_optional(
                repetition,
                PublisherArtifactV2::EmergencyRollbackReceipt,
            )?
            .context("general-failure rollback lacks one exact emergency receipt")?;
            validate_external_emergency_receipt(repetition, &receipt)?;
            digests.push(document_sha256_v2(&receipt)?);
        }
    }
    Ok(digests)
}

fn validate_general_failure_rollback_result(
    value: &GeneralFailureRollbackResultV2,
    prepared: &GeneralFailureRollbackPreparedV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-general-failure-rollback-result"
        || value.schema_version != 2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.prepared_sha256 != document_sha256_v2(prepared)?
        || !value.terminal_no_reinvoke
        || value.accepted_authority_ambiguity != prepared.accepted_authority_ambiguity
        || value
            .invoked_sha256
            .as_ref()
            .is_some_and(|digest| !is_sha256(digest))
        || value
            .emergency_receipt_sha256
            .iter()
            .any(|digest| !is_sha256(digest))
        || (value.exact_restoration_complete && value.accepted_authority_ambiguity)
        || (value.reconstructed_after_invoked && value.publisher_exit.is_some())
    {
        bail!("general-failure rollback result changed its exact terminal classification")
    }
    Ok(())
}

fn complete_general_failure_restoration(
    inputs: &FrozenRunnerInputs,
    cursor: &GeneralFailureRestorationCursorV2,
) -> Result<()> {
    validate_general_failure_cursor(cursor)?;
    if let Some(existing) = read_runner_private_optional::<GeneralFailureRestorationReceiptV2>(
        GENERAL_FAILURE_RECEIPT_NAME,
    )? {
        validate_general_failure_receipt(&existing, cursor)?;
        bail!("sealed experiment previously completed terminal general-failure restoration")
    }
    let rollback_result = complete_general_failure_rollback(inputs, cursor)?;
    if !rollback_result.exact_restoration_complete || rollback_result.accepted_authority_ambiguity {
        bail!("general-failure rollback did not authorize exact restoration")
    }
    let (repetitions, securityagent_observation_sha256, securityagent_report_bytes) =
        observe_root_operation_with_raw(
            || {
                let mut repetitions = Vec::with_capacity(RepetitionV2::ALL.len());
                for repetition in RepetitionV2::ALL {
                    repetitions.push(validate_and_restore_failed_repetition(repetition)?);
                }
                remove_exact_installed_file(
                    Path::new(ALTERNATE_COORDINATOR_PATH_V2),
                    None,
                    Some(&inputs.peer.alternate_path_identity.physical_identity_sha256),
                )?;
                remove_exact_installed_file(
                    Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2),
                    Some(&canonical_bytes_v2(&inputs.prepared)?),
                    None,
                )?;
                remove_exact_installed_file(
                    Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2),
                    Some(&canonical_bytes_v2(&inputs.candidate)?),
                    None,
                )?;
                remove_exact_installed_file(
                    Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
                    Some(&canonical_bytes_v2(&inputs.peer)?),
                    None,
                )?;
                Ok(repetitions)
            },
            |alert| persist_root_operation_terminal_alert("general-failure-restoration", alert),
        )?;
    persist_root_operation_securityagent_report(
        "general-failure-restoration.ui.observation.v2.json",
        &securityagent_observation_sha256,
        &securityagent_report_bytes,
    )?;
    let repetitions = repetitions?;
    let receipt = GeneralFailureRestorationReceiptV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-general-failure-restoration"
            .to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        publisher_exit: cursor.publisher_exit.clone(),
        harness_exit: cursor.harness_exit.clone(),
        rollback_result_sha256: document_sha256_v2(&rollback_result)?,
        repetitions,
        alternate_path_absent: !path_present(Path::new(ALTERNATE_COORDINATOR_PATH_V2))?,
        installed_packets_absent: !path_present(Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2))?
            && !path_present(Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2))?
            && !path_present(Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2))?,
        securityagent_observation_sha256,
        terminal_no_resume: true,
    };
    validate_general_failure_receipt(&receipt, cursor)?;
    write_runner_receipt(GENERAL_FAILURE_RECEIPT_NAME, &canonical_bytes_v2(&receipt)?)?;
    bail!("sealed experiment completed terminal general-failure restoration")
}

fn validate_general_failure_receipt(
    value: &GeneralFailureRestorationReceiptV2,
    cursor: &GeneralFailureRestorationCursorV2,
) -> Result<()> {
    validate_general_failure_cursor(cursor)?;
    if value.schema_owner
        != "substrate.r3-macos-disposable-experiment-runner-general-failure-restoration"
        || value.schema_version != 2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.publisher_exit != cursor.publisher_exit
        || value.harness_exit != cursor.harness_exit
        || !is_sha256(&value.rollback_result_sha256)
        || value.repetitions.len() != RepetitionV2::ALL.len()
        || !value.alternate_path_absent
        || !value.installed_packets_absent
        || !value.terminal_no_resume
        || !is_sha256(&value.securityagent_observation_sha256)
        || value.repetitions.iter().any(|entry| {
            !entry.signing_seed_absent_after
                || entry
                    .progress_sha256
                    .as_ref()
                    .is_some_and(|digest| !is_sha256(digest))
                || entry
                    .restoration_evidence_sha256
                    .as_ref()
                    .is_some_and(|digest| !is_sha256(digest))
                || entry
                    .signing_seed_identity_sha256
                    .as_ref()
                    .is_some_and(|digest| !is_sha256(digest))
        })
    {
        bail!("general-failure restoration receipt is incomplete or changed")
    }
    for (entry, repetition) in value.repetitions.iter().zip(RepetitionV2::ALL) {
        repetition.validate_binding(entry.repetition, &entry.scope_id)?;
    }
    Ok(())
}

fn validate_and_restore_failed_repetition(
    repetition: RepetitionV2,
) -> Result<FailedRepetitionRestorationV2> {
    let progress = read_external_root_optional::<PublisherProgressV2>(
        repetition,
        PublisherArtifactV2::Progress,
    )?;
    let (kind, progress_sha256, restoration_evidence_sha256) = match progress {
        None => {
            if read_external_root_optional::<SurrogateCreationReceiptV2>(
                repetition,
                PublisherArtifactV2::CreationReceipt,
            )?
            .is_some()
            {
                bail!("failed untouched repetition has a creation receipt without progress")
            }
            (FailedRepetitionRestorationKindV2::Untouched, None, None)
        }
        Some(progress) => {
            progress.validate(repetition)?;
            let progress_sha256 = Some(document_sha256_v2(&progress)?);
            if progress.stage == PublisherStageV2::RestorationComplete {
                let complete: FinalizerResponseV2 =
                    read_external(repetition, PublisherArtifactV2::CompleteResponse)?;
                let restoration: RestorationReceiptV2 = read_external_root_optional(
                    repetition,
                    PublisherArtifactV2::RestorationReceipt,
                )?
                .context("completed failed repetition lacks publisher restoration receipt")?;
                restoration.validate(repetition, &document_sha256_v2(&complete)?)?;
                (
                    FailedRepetitionRestorationKindV2::Completed,
                    progress_sha256,
                    Some(document_sha256_v2(&restoration)?),
                )
            } else {
                let rollback: EmergencyRollbackReceiptV2 = read_external_root_optional(
                    repetition,
                    PublisherArtifactV2::EmergencyRollbackReceipt,
                )?
                .context("failed repetition is neither complete nor durably emergency-restored")?;
                validate_external_emergency_receipt(repetition, &rollback)?;
                (
                    FailedRepetitionRestorationKindV2::EmergencyRollback,
                    progress_sha256,
                    Some(document_sha256_v2(&rollback)?),
                )
            }
        }
    };
    let signing_seed_identity_sha256 = remove_failed_harness_seed(repetition)?;
    Ok(FailedRepetitionRestorationV2 {
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        kind,
        progress_sha256,
        restoration_evidence_sha256,
        signing_seed_identity_sha256,
        signing_seed_absent_after: !path_present(&harness_seed_path(repetition))?,
    })
}

fn validate_external_emergency_receipt(
    repetition: RepetitionV2,
    receipt: &EmergencyRollbackReceiptV2,
) -> Result<()> {
    let created = read_external_optional::<EmergencyRollbackMarkerV2>(
        repetition,
        PublisherArtifactV2::EmergencyRollbackMarker,
    )?;
    let precreation = read_external_optional::<PreCreationEmergencyRollbackMarkerV2>(
        repetition,
        PublisherArtifactV2::PreCreationEmergencyRollbackMarker,
    )?;
    match (created, precreation) {
        (Some(marker), None) => receipt.validate(repetition, &marker),
        (None, Some(marker)) => receipt.validate_precreation(repetition, &marker),
        _ => bail!("failed repetition has zero or multiple emergency marker forms"),
    }
}

fn harness_seed_path(repetition: RepetitionV2) -> PathBuf {
    Path::new(EXPERIMENT_ROOT_V2)
        .join("repetitions")
        .join(repetition.directory_name())
        .join("protected/harness-ed25519-seed.v2.bin")
}

fn remove_failed_harness_seed(repetition: RepetitionV2) -> Result<Option<String>> {
    let path = harness_seed_path(repetition);
    let Some(bytes) = stable_read_file_optional(
        &path,
        DISPOSABLE_HARNESS_UID_V2,
        Some(libc::S_IFREG | 0o400),
    )?
    else {
        return Ok(None);
    };
    if bytes.len() != 32 {
        bail!("failed harness signing seed is not exactly 32 bytes")
    }
    let stat = lstat(&path)?.context("failed harness signing seed disappeared")?;
    let identity = document_sha256_v2(&(
        path.to_str().context("harness seed path is not UTF-8")?,
        runner_file_physical_identity_sha256(&stat)?,
        sha256_hex_v2(&bytes),
    ))?;
    remove_exact_owned_file(
        &path,
        Some(&bytes),
        Some(&runner_file_physical_identity_sha256(&stat)?),
        DISPOSABLE_HARNESS_UID_V2,
        Some(libc::S_IFREG | 0o400),
    )?;
    Ok(Some(identity))
}

fn peer_ready_present(repetition: RepetitionV2) -> Result<bool> {
    path_present(&external_exchange_path_v2(
        repetition,
        PublisherArtifactV2::PeerControlsReadyMarker,
    ))
}

fn complete_response_present(repetition: RepetitionV2) -> Result<bool> {
    path_present(&external_exchange_path_v2(
        repetition,
        PublisherArtifactV2::CompleteResponse,
    ))
}

fn peer_native_arm_file_name(
    repetition: RepetitionV2,
    sequence_ordinal: u8,
    suffix: &str,
) -> Result<String> {
    if !(1..=8).contains(&sequence_ordinal)
        || !matches!(suffix, "prepared" | "invoked" | "receipt" | "observed")
    {
        bail!("peer native arm cursor escaped its closed eight-arm sequence")
    }
    Ok(format!(
        "peer-native-arm-{}-{sequence_ordinal:02}.{suffix}.v2.json",
        repetition.ordinal()
    ))
}

fn expected_peer_native_arm_prepared(
    repetition: RepetitionV2,
    sequence_ordinal: u8,
    arm: PeerNativeArmV2,
    request: &FinalizationRequestV2,
    ready: &PeerControlsReadyMarkerV2,
    creation: &SurrogateCreationReceiptV2,
) -> Result<PeerNativeArmPreparedV2> {
    Ok(PeerNativeArmPreparedV2 {
        schema_owner: "substrate.r3-macos-disposable-peer-native-arm".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        sequence_ordinal,
        arm,
        request_digest: request.request_digest.clone(),
        ready_marker_sha256: document_sha256_v2(ready)?,
        creation_receipt_sha256: document_sha256_v2(creation)?,
        invocation_requires_durable_invoked_cursor: true,
        missing_receipt_after_invoked_is_ambiguous: true,
    })
}

fn classify_peer_native_arm_recovery(
    invoked_present: bool,
    receipt_present: bool,
    observed_present: bool,
) -> Result<PeerNativeArmRecoveryDecisionV2> {
    match (invoked_present, receipt_present, observed_present) {
        (false, false, false) => Ok(PeerNativeArmRecoveryDecisionV2::Invoke),
        (true, true, false) => Ok(PeerNativeArmRecoveryDecisionV2::ReconstructObserved),
        (true, true, true) => Ok(PeerNativeArmRecoveryDecisionV2::AcceptObserved),
        (true, false, false) => Ok(PeerNativeArmRecoveryDecisionV2::AmbiguousNoReinvoke),
        _ => bail!("peer native arm cursor/receipt prefix is not one valid durable state"),
    }
}

fn load_or_prepare_peer_native_arm<T: DeserializeOwned + Serialize>(
    repetition: RepetitionV2,
    sequence_ordinal: u8,
    arm: PeerNativeArmV2,
    request: &FinalizationRequestV2,
    ready: &PeerControlsReadyMarkerV2,
    creation: &SurrogateCreationReceiptV2,
) -> Result<Option<T>> {
    let prepared = expected_peer_native_arm_prepared(
        repetition,
        sequence_ordinal,
        arm,
        request,
        ready,
        creation,
    )?;
    write_runner_receipt(
        &peer_native_arm_file_name(repetition, sequence_ordinal, "prepared")?,
        &canonical_bytes_v2(&prepared)?,
    )?;
    let invoked = read_runner_private_optional::<PeerNativeArmInvokedV2>(
        &peer_native_arm_file_name(repetition, sequence_ordinal, "invoked")?,
    )?;
    let receipt = read_runner_private_optional::<T>(&peer_native_arm_file_name(
        repetition,
        sequence_ordinal,
        "receipt",
    )?)?;
    let observed = read_runner_private_optional::<PeerNativeArmObservedV2>(
        &peer_native_arm_file_name(repetition, sequence_ordinal, "observed")?,
    )?;
    if let Some(invoked) = &invoked {
        validate_peer_native_arm_invoked(invoked, &prepared)?;
    }
    match classify_peer_native_arm_recovery(
        invoked.is_some(),
        receipt.is_some(),
        observed.is_some(),
    )? {
        PeerNativeArmRecoveryDecisionV2::Invoke => Ok(None),
        PeerNativeArmRecoveryDecisionV2::ReconstructObserved => {
            let invoked = invoked.context("reconstructible peer arm lacks Invoked cursor")?;
            let receipt = receipt.context("reconstructible peer arm lacks receipt")?;
            persist_peer_native_arm_observed(
                repetition,
                sequence_ordinal,
                arm,
                &invoked,
                &receipt,
            )?;
            Ok(Some(receipt))
        }
        PeerNativeArmRecoveryDecisionV2::AcceptObserved => {
            let invoked = invoked.context("observed peer arm lacks Invoked cursor")?;
            let receipt = receipt.context("observed peer arm lacks receipt")?;
            let observed = observed.context("observed peer arm lacks Observed cursor")?;
            validate_peer_native_arm_observed(&observed, &invoked, &receipt)?;
            Ok(Some(receipt))
        }
        PeerNativeArmRecoveryDecisionV2::AmbiguousNoReinvoke => {
            bail!("peer native arm is Invoked without a durable result; blind reinvoke forbidden")
        }
    }
}

fn persist_peer_native_arm_invoked(
    repetition: RepetitionV2,
    sequence_ordinal: u8,
    arm: PeerNativeArmV2,
    request: &FinalizationRequestV2,
    ready: &PeerControlsReadyMarkerV2,
    creation: &SurrogateCreationReceiptV2,
    before: PeerNativeArmBeforeV2<'_>,
) -> Result<PeerNativeArmInvokedV2> {
    if !before.accepted_journal_absent || !is_sha256(before.observation_sha256) {
        bail!("peer native arm lacks exact pre-invocation state and journal absence")
    }
    let prepared = expected_peer_native_arm_prepared(
        repetition,
        sequence_ordinal,
        arm,
        request,
        ready,
        creation,
    )?;
    let invoked = PeerNativeArmInvokedV2 {
        schema_owner: "substrate.r3-macos-disposable-peer-native-arm".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        sequence_ordinal,
        arm,
        prepared_sha256: document_sha256_v2(&prepared)?,
        before_observation_sha256: before.observation_sha256.to_owned(),
        accepted_journal_absent_before: before.accepted_journal_absent,
        invocation_may_begin: true,
        blind_reinvoke_forbidden: true,
    };
    validate_peer_native_arm_invoked(&invoked, &prepared)?;
    write_runner_receipt(
        &peer_native_arm_file_name(repetition, sequence_ordinal, "invoked")?,
        &canonical_bytes_v2(&invoked)?,
    )?;
    Ok(invoked)
}

fn validate_peer_native_arm_invoked(
    value: &PeerNativeArmInvokedV2,
    prepared: &PeerNativeArmPreparedV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-peer-native-arm"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.repetition != prepared.repetition
        || value.scope_id != prepared.scope_id
        || value.sequence_ordinal != prepared.sequence_ordinal
        || value.arm != prepared.arm
        || value.prepared_sha256 != document_sha256_v2(prepared)?
        || !is_sha256(&value.before_observation_sha256)
        || !value.accepted_journal_absent_before
        || !value.invocation_may_begin
        || !value.blind_reinvoke_forbidden
    {
        bail!("peer native arm Invoked cursor changed or broadened authority")
    }
    Ok(())
}

fn persist_peer_native_arm_observed<T: Serialize>(
    repetition: RepetitionV2,
    sequence_ordinal: u8,
    arm: PeerNativeArmV2,
    invoked: &PeerNativeArmInvokedV2,
    receipt: &T,
) -> Result<()> {
    let receipt_bytes = canonical_bytes_v2(receipt)?;
    write_runner_receipt(
        &peer_native_arm_file_name(repetition, sequence_ordinal, "receipt")?,
        &receipt_bytes,
    )?;
    let observed = PeerNativeArmObservedV2 {
        schema_owner: "substrate.r3-macos-disposable-peer-native-arm".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        sequence_ordinal,
        arm,
        invoked_sha256: document_sha256_v2(invoked)?,
        receipt_sha256: sha256_hex_v2(&receipt_bytes),
        exact_observed_no_reinvoke: true,
    };
    validate_peer_native_arm_observed(&observed, invoked, receipt)?;
    write_runner_receipt(
        &peer_native_arm_file_name(repetition, sequence_ordinal, "observed")?,
        &canonical_bytes_v2(&observed)?,
    )
}

fn validate_peer_native_arm_observed<T: Serialize>(
    value: &PeerNativeArmObservedV2,
    invoked: &PeerNativeArmInvokedV2,
    receipt: &T,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-peer-native-arm"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.repetition != invoked.repetition
        || value.scope_id != invoked.scope_id
        || value.sequence_ordinal != invoked.sequence_ordinal
        || value.arm != invoked.arm
        || value.invoked_sha256 != document_sha256_v2(invoked)?
        || value.receipt_sha256 != document_sha256_v2(receipt)?
        || !value.exact_observed_no_reinvoke
    {
        bail!("peer native arm Observed cursor changed or lost its exact durable receipt")
    }
    Ok(())
}

fn load_or_run_peer_controls(
    repetition: RepetitionV2,
    inputs: &FrozenRunnerInputs,
    security: &mut NonInteractiveSecurity,
) -> Result<PeerControlSetReceiptV2> {
    if let Some(existing) = read_external_root_optional::<PeerControlSetReceiptV2>(
        repetition,
        PublisherArtifactV2::PeerControlRunnerSetReceipt,
    )? {
        let request: FinalizationRequestV2 =
            read_external(repetition, PublisherArtifactV2::FinalizationRequest)?;
        existing.validate(repetition, &request, &inputs.peer)?;
        return Ok(existing);
    }
    run_peer_controls(repetition, inputs, security)
}

fn run_peer_controls(
    repetition: RepetitionV2,
    inputs: &FrozenRunnerInputs,
    security: &mut NonInteractiveSecurity,
) -> Result<PeerControlSetReceiptV2> {
    let ready: PeerControlsReadyMarkerV2 =
        read_external(repetition, PublisherArtifactV2::PeerControlsReadyMarker)?;
    let request: FinalizationRequestV2 =
        read_external(repetition, PublisherArtifactV2::FinalizationRequest)?;
    validate_ready_marker_shape(&ready, repetition, &request)?;
    let dynamic_library_injection =
        run_dynamic_library_injection_control(repetition, inputs, security, &request, &ready)?;
    let nobody_owner_authority =
        run_nobody_owner_authority_control(repetition, inputs, &request, &ready)?;
    let mut receipts = Vec::new();
    for (index, control) in PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2
        .into_iter()
        .enumerate()
    {
        let creation: SurrogateCreationReceiptV2 =
            read_external(repetition, PublisherArtifactV2::CreationReceipt)?;
        creation.validate(repetition)?;
        let sequence_ordinal = u8::try_from(index + 6)?;
        let arm = PeerNativeArmV2::PeerSubstitution { control };
        if let Some(receipt) = load_or_prepare_peer_native_arm::<PeerSubstitutionControlReceiptV2>(
            repetition,
            sequence_ordinal,
            arm,
            &request,
            &ready,
            &creation,
        )? {
            receipt.validate(repetition, &request, &inputs.peer)?;
            receipts.push(receipt);
            continue;
        }
        let before_observation_sha256 = observe_exact_surrogate_state_under_ui(
            repetition,
            security,
            &creation,
            "peer-substitution",
            u8::try_from(index + 1)?,
            "before",
        )?;
        let accepted_journal_absent_before = !path_present(
            &Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id()),
        )?;
        if !accepted_journal_absent_before {
            bail!("peer substitution control found an accepted finalizer journal before invoke")
        }
        let invoked = persist_peer_native_arm_invoked(
            repetition,
            sequence_ordinal,
            arm,
            &request,
            &ready,
            &creation,
            PeerNativeArmBeforeV2 {
                observation_sha256: &before_observation_sha256,
                accepted_journal_absent: accepted_journal_absent_before,
            },
        )?;
        let (path, uid, gid, identity) = match control {
            PeerSubstitutionControlV2::AlternateCaller => (
                MAC_R3_COORDINATOR_PATH_V2,
                None,
                None,
                &inputs.peer.alternate_caller_identity,
            ),
            PeerSubstitutionControlV2::AlternateExecutableCodeIdentity => {
                let (_, gid) = canonical_account(DISPOSABLE_HARNESS_UID_V2)?;
                (
                    PEER_CODE_PROBE_PATH_V2,
                    Some(DISPOSABLE_HARNESS_UID_V2),
                    Some(gid),
                    &inputs.peer.alternate_code_identity,
                )
            }
            PeerSubstitutionControlV2::AlternatePhysicalPath => {
                let (_, gid) = canonical_account(DISPOSABLE_HARNESS_UID_V2)?;
                (
                    ALTERNATE_COORDINATOR_PATH_V2,
                    Some(DISPOSABLE_HARNESS_UID_V2),
                    Some(gid),
                    &inputs.peer.alternate_path_identity,
                )
            }
        };
        let command = sealed_command(path, "/", uid, gid)?;
        let (observed, stopped_process) = observe_sealed_child_with(command, |pid| {
            attest_stopped_peer_process(
                pid as i32,
                uid.unwrap_or(0),
                path,
                identity,
                repetition,
                control,
                &before_observation_sha256,
            )
        })?;
        if observed.unexpected_ui_observed {
            return Err(runner_securityagent_alert(
                repetition,
                "peer-substitution",
                &observed,
            )?);
        }
        if !observed.status.success() {
            bail!("peer substitution child did not complete without UI")
        }
        let stopped_process = stopped_process
            .context("successful peer substitution lacks its stopped-process attestation")?;
        let exchange = parse_peer_frame(&observed.stdout)?;
        let expected = inputs.peer.expected_termination_for(control);
        if expected != PeerProbeTerminationV2::ConnectionClosedBeforeResponse {
            bail!("runner peer identity packet changed the frozen close-before-response class")
        }
        exchange.validate(&request, expected)?;
        let after_identity = measure_frozen_executable(frozen_code_for_identity(path, identity)?)?;
        if after_identity != *identity {
            bail!("peer executable identity changed after its closed exchange")
        }
        let after_observation_sha256 = observe_exact_surrogate_state_under_ui(
            repetition,
            security,
            &creation,
            "peer-substitution",
            u8::try_from(index + 1)?,
            "after",
        )?;
        let accepted_journal_absent_after = !path_present(
            &Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id()),
        )?;
        if before_observation_sha256 != after_observation_sha256 || !accepted_journal_absent_after {
            bail!("peer substitution control changed exact surrogate state or journal absence")
        }
        let process = RunnerPeerProcessAttestationV2 {
            schema_owner: "substrate.r3-macos-disposable-runner-peer-process",
            schema_version: 2,
            repetition: repetition.ordinal(),
            control,
            pid: stopped_process.pid,
            effective_uid: stopped_process.effective_uid,
            effective_gid: stopped_process.effective_gid,
            supplementary_groups: stopped_process.supplementary_groups,
            canonical_account: stopped_process.canonical_account,
            process_start_identity_sha256: stopped_process.process_start_identity_sha256,
            executable_identity_sha256: stopped_process.executable_identity_sha256,
            before_observation_sha256: before_observation_sha256.clone(),
            accepted_journal_absent_after,
        };
        let process_sha256 = document_sha256_v2(&process)?;
        write_runner_receipt(
            &format!(
                "peer-{}-{:02}.process-attestation.v2.json",
                repetition.ordinal(),
                index + 1
            ),
            &canonical_bytes_v2(&process)?,
        )?;
        let securityagent_evidence = securityagent_arm_evidence(&observed.securityagent_report)?;
        let mut receipt = PeerSubstitutionControlReceiptV2 {
            schema_owner: PEER_CONTROL_RECEIPT_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_owned(),
            sequence_ordinal: (index + 1) as u8,
            control,
            executable_identity_sha256: document_sha256_v2(identity)?,
            process_attestation_sha256: process_sha256,
            exchange,
            securityagent_observation_sha256: document_sha256_v2(&securityagent_evidence)?,
            securityagent_evidence,
            before_observation_sha256,
            after_observation_sha256,
            accepted_journal_absent_before,
            accepted_journal_absent_after,
            no_mutation_binding_sha256: String::new(),
        };
        receipt.no_mutation_binding_sha256 =
            peer_substitution_no_mutation_binding_sha256_v2(&receipt)?;
        receipt.validate(repetition, &request, &inputs.peer)?;
        persist_peer_native_arm_observed(repetition, sequence_ordinal, arm, &invoked, &receipt)?;
        receipts.push(receipt);
    }
    let set = PeerControlSetReceiptV2 {
        schema_owner: PEER_CONTROL_RECEIPT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        request_digest: request.request_digest.clone(),
        identity_packet_sha256: document_sha256_v2(&inputs.peer)?,
        runner_identity_sha256: inputs.runner_identity_sha256.clone(),
        runner_process_attestation_sha256: inputs.runner_process_attestation_sha256.clone(),
        receipt_set_sha256: peer_control_receipt_set_sha256_v2(
            &receipts,
            &dynamic_library_injection,
            &nobody_owner_authority,
        )?,
        receipts,
        dynamic_library_injection,
        nobody_owner_authority,
        before_observation_sha256: ready.before_observation_sha256,
    };
    set.validate(repetition, &request, &inputs.peer)?;
    let bytes = canonical_bytes_v2(&set)?;
    write_runner_receipt(
        &format!("peer-{}-set.receipt.v2.json", repetition.ordinal()),
        &bytes,
    )?;
    write_external_root_output(
        repetition,
        PublisherArtifactV2::PeerControlRunnerSetReceipt,
        &bytes,
    )?;
    Ok(set)
}

fn run_dynamic_library_injection_control(
    repetition: RepetitionV2,
    inputs: &FrozenRunnerInputs,
    security: &mut NonInteractiveSecurity,
    request: &FinalizationRequestV2,
    ready: &PeerControlsReadyMarkerV2,
) -> Result<DynamicLibraryInjectionControlReceiptV2> {
    let creation: SurrogateCreationReceiptV2 =
        read_external(repetition, PublisherArtifactV2::CreationReceipt)?;
    creation.validate(repetition)?;
    let arm = PeerNativeArmV2::DynamicLibraryInjection;
    if let Some(receipt) = load_or_prepare_peer_native_arm::<DynamicLibraryInjectionControlReceiptV2>(
        repetition, 1, arm, request, ready, &creation,
    )? {
        receipt.validate(repetition, request, &inputs.peer)?;
        return Ok(receipt);
    }
    let before_observation_sha256 = observe_exact_surrogate_state_under_ui(
        repetition,
        security,
        &creation,
        "dynamic-injection",
        1,
        "before",
    )?;
    let accepted_journal_absent_before =
        !path_present(&Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id()))?;
    let invoked = persist_peer_native_arm_invoked(
        repetition,
        1,
        arm,
        request,
        ready,
        &creation,
        PeerNativeArmBeforeV2 {
            observation_sha256: &before_observation_sha256,
            accepted_journal_absent: accepted_journal_absent_before,
        },
    )?;
    let (command, mut marker_reader) = sealed_dynamic_library_command()?;
    let (observed, stopped_process) = observe_sealed_child_with(command, |pid| {
        attest_dynamic_peer_or_loader_exit(
            pid as i32,
            DISPOSABLE_HARNESS_UID_V2,
            ALTERNATE_COORDINATOR_PATH_V2,
            &inputs.peer.alternate_path_identity,
        )
    })?;
    if observed.unexpected_ui_observed {
        return Err(runner_securityagent_alert(
            repetition,
            "dynamic-library-injection",
            &observed,
        )?);
    }
    let stopped_process = stopped_process
        .context("completed dynamic-loader control lacks its rendezvous classification")?;
    let mut marker_after = Vec::new();
    std::io::Read::by_ref(&mut marker_reader)
        .take(8_193)
        .read_to_end(&mut marker_after)?;
    if marker_after.len() > 8_192 {
        bail!("benign injection marker stream exceeded its fixed bound")
    }
    let after_observation_sha256 = observe_exact_surrogate_state_under_ui(
        repetition,
        security,
        &creation,
        "dynamic-injection",
        1,
        "after",
    )?;
    let accepted_journal_absent =
        !path_present(&Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id()))?;
    let (termination, process_attestation_sha256, exchange, process_surface_environment_empty) =
        if observed.status.success() {
            let stopped_process = stopped_process.context(
                "neutralized injection target reached main without exact stopped attestation",
            )?;
            let exchange = parse_peer_frame(&observed.stdout)?;
            exchange.validate(
                request,
                PeerProbeTerminationV2::ConnectionClosedBeforeResponse,
            )?;
            let process = RunnerPeerProcessAttestationV2 {
                schema_owner: "substrate.r3-macos-disposable-runner-dynamic-injection-process",
                schema_version: 2,
                repetition: repetition.ordinal(),
                control: PeerSubstitutionControlV2::AlternatePhysicalPath,
                pid: stopped_process.pid,
                effective_uid: stopped_process.effective_uid,
                effective_gid: stopped_process.effective_gid,
                supplementary_groups: stopped_process.supplementary_groups,
                canonical_account: stopped_process.canonical_account,
                process_start_identity_sha256: stopped_process.process_start_identity_sha256,
                executable_identity_sha256: stopped_process.executable_identity_sha256,
                before_observation_sha256: ready.before_observation_sha256.clone(),
                accepted_journal_absent_after: accepted_journal_absent,
            };
            write_runner_receipt(
                &format!(
                    "dynamic-injection-{}.process-attestation.v2.json",
                    repetition.ordinal()
                ),
                &canonical_bytes_v2(&process)?,
            )?;
            (
                BenignInjectionTerminationV2::EnvironmentNeutralized,
                document_sha256_v2(&process)?,
                Some(exchange),
                Some(true),
            )
        } else {
            if stopped_process.is_some() {
                bail!("attested main process returned a non-success loader classification")
            }
            let attempt = document_sha256_v2(&(
                "substrate.r3-macos-disposable-loader-rejected-attempt.v2",
                repetition.ordinal(),
                ALTERNATE_COORDINATOR_PATH_V2,
                document_sha256_v2(&inputs.peer.alternate_path_identity)?,
                BENIGN_INJECTION_LIBRARY_PATH_V2,
                document_sha256_v2(&inputs.peer.benign_injection_library_identity)?,
                observed.status.code(),
                observed.status.signal(),
            ))?;
            (
                BenignInjectionTerminationV2::LoaderRejectedBeforeMain,
                attempt,
                None,
                None,
            )
        };
    let empty = bounded_raw_stream(&[])?;
    let marker = bounded_raw_stream(&marker_after)?;
    let child_stdout = bounded_raw_stream(&observed.stdout)?;
    let child_stderr = bounded_raw_stream(&observed.stderr)?;
    let loader_diagnostic_sha256 = (termination
        == BenignInjectionTerminationV2::LoaderRejectedBeforeMain)
        .then(|| child_stderr.raw_sha256.clone());
    let securityagent_report = securityagent_raw_evidence(&observed.securityagent_report)?;
    let mut receipt = DynamicLibraryInjectionControlReceiptV2 {
        schema_owner: PEER_CONTROL_RECEIPT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        request_digest: request.request_digest.clone(),
        target_executable_identity_sha256: document_sha256_v2(
            &inputs.peer.alternate_path_identity,
        )?,
        injection_library_identity_sha256: document_sha256_v2(
            &inputs.peer.benign_injection_library_identity,
        )?,
        process_attestation_sha256,
        dyld_environment_key: BENIGN_INJECTION_DYLD_ENVIRONMENT_KEY_V2.to_owned(),
        dyld_environment_value: BENIGN_INJECTION_LIBRARY_PATH_V2.to_owned(),
        marker_fd: BENIGN_INJECTION_MARKER_FD_V2,
        termination,
        child_exit_code: observed.status.code(),
        child_signal: observed.status.signal(),
        process_surface_environment_empty,
        exchange,
        loader_diagnostic_sha256,
        child_stdout,
        child_stderr,
        marker_before: empty,
        marker_after: marker.clone(),
        injection_signal_sha256: marker.raw_sha256.clone(),
        injection_signal_observed: !marker_after.is_empty(),
        before_observation_sha256,
        after_observation_sha256,
        accepted_journal_absent,
        securityagent_report,
        no_mutation_binding_sha256: String::new(),
    };
    receipt.no_mutation_binding_sha256 = dynamic_library_no_mutation_binding_sha256_v2(&receipt)?;
    receipt.validate(repetition, request, &inputs.peer)?;
    persist_peer_native_arm_observed(repetition, 1, arm, &invoked, &receipt)?;
    Ok(receipt)
}

fn run_nobody_owner_authority_control(
    repetition: RepetitionV2,
    inputs: &FrozenRunnerInputs,
    request: &FinalizationRequestV2,
    ready: &PeerControlsReadyMarkerV2,
) -> Result<NobodyOwnerAuthorityControlReceiptV2> {
    let creation: SurrogateCreationReceiptV2 =
        read_external(repetition, PublisherArtifactV2::CreationReceipt)?;
    creation.validate(repetition)?;
    let (security, report_sha256, report_bytes) =
        observe_root_operation_with_raw(NonInteractiveSecurity::establish_first, |alert| {
            persist_root_operation_terminal_alert_with_context(
                "nobody-owner-first-security-interaction-denial",
                alert,
                Some((repetition, &creation)),
            )
        })?;
    persist_root_operation_securityagent_report(
        &format!(
            "nobody-owner-first-security-interaction-denial-repetition-{}.ui.observation.v2.json",
            repetition.ordinal()
        ),
        &report_sha256,
        &report_bytes,
    )?;
    let mut security = security?;
    let before_observation_sha256 = observe_exact_surrogate_state_under_ui(
        repetition,
        &mut security,
        &creation,
        "nobody-owner",
        0,
        "aggregate-before",
    )?;
    let accepted_journal_absent_before =
        !path_present(&Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id()))?;
    if !accepted_journal_absent_before {
        bail!("nobody owner-authority control found an accepted journal before its attempts")
    }
    let fixed = fixed_repetition(repetition);
    let mut attempts = Vec::with_capacity(NOBODY_OWNER_AUTHORITY_SEQUENCE_V2.len());
    for (index, operation) in NOBODY_OWNER_AUTHORITY_SEQUENCE_V2.into_iter().enumerate() {
        let sequence_ordinal = u8::try_from(index + 2)?;
        let arm = PeerNativeArmV2::NobodyOwnerAuthority { operation };
        if let Some(attempt) = load_or_prepare_peer_native_arm::<NobodyOwnerAuthorityAttemptV2>(
            repetition,
            sequence_ordinal,
            arm,
            request,
            ready,
            &creation,
        )? {
            validate_nobody_owner_attempt(
                &attempt,
                repetition,
                index,
                operation,
                inputs,
                &before_observation_sha256,
            )?;
            attempts.push(attempt);
            continue;
        }
        let root_before = observe_exact_surrogate_state_under_ui(
            repetition,
            &mut security,
            &creation,
            "nobody-owner",
            u8::try_from(index + 1)?,
            "before",
        )?;
        if root_before != before_observation_sha256 {
            bail!("nobody owner-authority root state changed before one exact operation")
        }
        let invoked = persist_peer_native_arm_invoked(
            repetition,
            sequence_ordinal,
            arm,
            request,
            ready,
            &creation,
            PeerNativeArmBeforeV2 {
                observation_sha256: &root_before,
                accepted_journal_absent: accepted_journal_absent_before,
            },
        )?;
        let command = NobodyOwnerProbeCommandV2 {
            schema_owner: "substrate.r3-macos-disposable-nobody-owner-probe-command".to_owned(),
            schema_version: 2,
            repetition: fixed,
            scope_id: repetition.scope_id().to_owned(),
            operation,
            request_digest: request.request_digest.clone(),
        };
        command.validate()?;
        let (observed, probe_process) =
            observe_sealed_child_with(sealed_nobody_owner_command(&command)?, |pid| {
                attest_stopped_nobody_owner_process(
                    pid as i32,
                    repetition,
                    u8::try_from(index + 1).expect("four nobody controls fit u8"),
                    inputs,
                )
            })?;
        if observed.unexpected_ui_observed {
            return Err(runner_securityagent_alert(
                repetition,
                "nobody-owner-authority",
                &observed,
            )?);
        }
        if !observed.status.success() {
            bail!("nobody owner-authority child did not return one no-UI denial")
        }
        let probe_process = probe_process
            .context("successful nobody control lacks its stopped-process attestation")?;
        let native: NobodyOwnerNativeReceiptV2 = parse_stdout_json(&observed)?;
        if native.repetition != fixed
            || native.scope_id != repetition.scope_id()
            || native.operation != operation
            || native.request_digest != request.request_digest
            || native.supplementary_groups.validate_empty().is_err()
            || native.raw_os_status != native.classification.raw_os_status()
        {
            bail!("nobody owner-authority native receipt changed its exact operation or denial")
        }
        let root_after = observe_exact_surrogate_state_under_ui(
            repetition,
            &mut security,
            &creation,
            "nobody-owner",
            u8::try_from(index + 1)?,
            "after",
        )?;
        if root_after != root_before {
            bail!("nobody owner-authority operation changed exact aggregate surrogate state")
        }
        let securityagent_report = securityagent_raw_evidence(&observed.securityagent_report)?;
        let attempt = NobodyOwnerAuthorityAttemptV2 {
            sequence_ordinal: u8::try_from(index + 1).expect("four nobody controls fit u8"),
            operation,
            probe_process_attestation_sha256: document_sha256_v2(&probe_process)?,
            probe_process,
            probe_self_supplementary_groups: native.supplementary_groups,
            raw_os_status: native.raw_os_status,
            classification: native.classification,
            root_before_observation_sha256: root_before,
            root_after_observation_sha256: root_after,
            securityagent_observation_sha256: securityagent_report.raw_report_sha256.clone(),
            securityagent_report,
        };
        validate_nobody_owner_attempt(
            &attempt,
            repetition,
            index,
            operation,
            inputs,
            &before_observation_sha256,
        )?;
        persist_peer_native_arm_observed(repetition, sequence_ordinal, arm, &invoked, &attempt)?;
        attempts.push(attempt);
    }
    let after_observation_sha256 = observe_exact_surrogate_state_under_ui(
        repetition,
        &mut security,
        &creation,
        "nobody-owner",
        0,
        "aggregate-after",
    )?;
    let accepted_journal_absent_after =
        !path_present(&Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id()))?;
    let ui_digests = attempts
        .iter()
        .map(|attempt| attempt.securityagent_observation_sha256.clone())
        .collect::<Vec<_>>();
    let mut receipt = NobodyOwnerAuthorityControlReceiptV2 {
        schema_owner: NOBODY_OWNER_CONTROL_RECEIPT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        request_digest: request.request_digest.clone(),
        persisted_owner_uid: NOBODY_PERSISTED_OWNER_UID_V2,
        persisted_owner_gid: NOBODY_PERSISTED_OWNER_GID_V2,
        persisted_owner_type: NOBODY_PERSISTED_OWNER_TYPE_V2,
        target_identity_sha256: creation.target.identity_sha256.clone(),
        wrong_identity_sha256: creation.wrong.identity_sha256.clone(),
        signer_access_control_sha256: creation.target.access_control_sha256.clone(),
        before_observation_sha256,
        attempt_set_sha256: document_sha256_v2(&attempts)?,
        attempts,
        after_observation_sha256,
        accepted_journal_absent_before,
        accepted_journal_absent_after,
        securityagent_observation_set_sha256: document_sha256_v2(&ui_digests)?,
        no_mutation_binding_sha256: String::new(),
    };
    receipt.no_mutation_binding_sha256 = nobody_owner_no_mutation_binding_sha256_v2(&receipt)?;
    receipt.validate(repetition, request, &inputs.peer)?;
    receipt.validate_against_creation(repetition, &creation)?;
    Ok(receipt)
}

fn validate_nobody_owner_attempt(
    attempt: &NobodyOwnerAuthorityAttemptV2,
    repetition: RepetitionV2,
    index: usize,
    operation: NobodyOwnerAuthorityOperationV2,
    inputs: &FrozenRunnerInputs,
    aggregate_before_observation_sha256: &str,
) -> Result<()> {
    if attempt.sequence_ordinal != u8::try_from(index + 1)?
        || attempt.operation != operation
        || attempt.probe_process_attestation_sha256 != document_sha256_v2(&attempt.probe_process)?
        || attempt.raw_os_status != attempt.classification.raw_os_status()
        || attempt.root_before_observation_sha256 != aggregate_before_observation_sha256
        || attempt.root_after_observation_sha256 != aggregate_before_observation_sha256
        || attempt.securityagent_observation_sha256
            != attempt.securityagent_report.raw_report_sha256
    {
        bail!("nobody owner attempt changed its exact denial or no-mutation evidence")
    }
    attempt.probe_process.validate(repetition, &inputs.peer)?;
    attempt.securityagent_report.validate()
}

fn fixed_repetition(repetition: RepetitionV2) -> FixedRepetitionV2 {
    match repetition {
        RepetitionV2::One => FixedRepetitionV2::First,
        RepetitionV2::Two => FixedRepetitionV2::Second,
    }
}

fn bounded_raw_stream(bytes: &[u8]) -> Result<BoundedRawStreamEvidenceV2> {
    let evidence = BoundedRawStreamEvidenceV2 {
        raw_base64url: URL_SAFE_NO_PAD.encode(bytes),
        raw_sha256: sha256_hex_v2(bytes),
        raw_byte_length: u64::try_from(bytes.len()).context("raw stream length exceeds u64")?,
    };
    evidence.validate()?;
    Ok(evidence)
}

fn sealed_dynamic_library_command() -> Result<(MeasuredCommandV2, UnixStream)> {
    let (reader, writer) = UnixStream::pair().context("create benign injection marker socket")?;
    let mut child = sealed_command(
        ALTERNATE_COORDINATOR_PATH_V2,
        "/",
        Some(DISPOSABLE_HARNESS_UID_V2),
        Some(canonical_account(DISPOSABLE_HARNESS_UID_V2)?.1),
    )?;
    child.env(
        BENIGN_INJECTION_DYLD_ENVIRONMENT_KEY_V2,
        BENIGN_INJECTION_LIBRARY_PATH_V2,
    );
    // SAFETY: after fork and before exec, only async-signal-safe descriptor operations occur. FD4
    // is the sole compiled constructor signal sink and carries no authority input.
    unsafe {
        child.pre_exec(move || {
            let source = writer.as_raw_fd();
            if source != BENIGN_INJECTION_MARKER_FD_V2
                && libc::dup2(source, BENIGN_INJECTION_MARKER_FD_V2) < 0
            {
                return Err(std::io::Error::last_os_error());
            }
            if libc::fcntl(BENIGN_INJECTION_MARKER_FD_V2, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok((child, reader))
}

fn sealed_nobody_owner_command(command: &NobodyOwnerProbeCommandV2) -> Result<MeasuredCommandV2> {
    let (mut writer, reader) = UnixStream::pair().context("create nobody-owner command socket")?;
    writer.write_all(&canonical_bytes_v2(command)?)?;
    writer.shutdown(std::net::Shutdown::Write)?;
    let mut child = sealed_command(
        NOBODY_OWNER_PROBE_PATH_V2,
        "/",
        Some(NOBODY_PRINCIPAL_UID_V2),
        Some(NOBODY_PRINCIPAL_GID_V2),
    )?;
    // SAFETY: after fork and before exec, only async-signal-safe descriptor operations occur. The
    // captured socket keeps the exact root-authored canonical command live until exec.
    unsafe {
        child.pre_exec(move || {
            let source = reader.as_raw_fd();
            if source != 3 && libc::dup2(source, 3) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::fcntl(3, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok(child)
}

fn attest_stopped_nobody_owner_process(
    pid: i32,
    repetition: RepetitionV2,
    sequence_ordinal: u8,
    inputs: &FrozenRunnerInputs,
) -> Result<NobodyOwnerProbeProcessAttestationV2> {
    wait_for_sigstop(pid)?;
    let process = process_info(pid)?;
    if process.pid != pid as u32
        || process.uid != NOBODY_PRINCIPAL_UID_V2
        || process.gid != NOBODY_PRINCIPAL_GID_V2
        || pid_path(pid)? != Path::new(NOBODY_OWNER_PROBE_PATH_V2)
    {
        bail!("stopped nobody owner probe changed uid, gid, or exact executable path")
    }
    let (account, _) = canonical_account(process.uid)?;
    let group = canonical_group(process.gid)?;
    if account != NOBODY_PRINCIPAL_ACCOUNT_V2 || group != NOBODY_PRINCIPAL_GROUP_V2 {
        bail!("stopped nobody owner probe principal does not resolve canonically")
    }
    let identity = &inputs.peer.nobody_owner_probe_identity;
    let measured = measure_frozen_executable(frozen_code_for_identity(
        NOBODY_OWNER_PROBE_PATH_V2,
        identity,
    )?)?;
    if measured != *identity {
        bail!("stopped nobody owner probe executable identity changed after exec")
    }
    let attestation = NobodyOwnerProbeProcessAttestationV2 {
        schema_owner: NOBODY_OWNER_CONTROL_RECEIPT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        repetition: repetition.ordinal(),
        scope_id: repetition.scope_id().to_owned(),
        pid,
        process_start_identity_sha256: document_sha256_v2(&ProcessStartJoinV2 {
            pid,
            seconds: process.start_seconds,
            microseconds: process.start_microseconds,
        })?,
        executable_identity_sha256: document_sha256_v2(identity)?,
        effective_uid: process.uid,
        effective_gid: process.gid,
        supplementary_groups: measured_child_supplementary_groups_v2(pid)?,
        canonical_account: account,
        canonical_group: group,
        argv_count: 0,
        environment_variable_count: 0,
        stdin_is_dev_null: true,
        cwd: "/".to_owned(),
    };
    attestation.validate(repetition, &inputs.peer)?;
    write_runner_receipt(
        &format!(
            "nobody-owner-{}-{sequence_ordinal:02}.process-attestation.v2.json",
            repetition.ordinal()
        ),
        &canonical_bytes_v2(&attestation)?,
    )?;
    // SAFETY: this exact stopped child is resumed once only after durable root attestation.
    if unsafe { libc::kill(pid, libc::SIGCONT) } != 0 {
        return Err(std::io::Error::last_os_error()).context("resume nobody owner probe");
    }
    Ok(attestation)
}

fn observe_exact_surrogate_state(
    repetition: RepetitionV2,
    security: &mut NonInteractiveSecurity,
    creation: &SurrogateCreationReceiptV2,
) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Observation<'a> {
        schema_owner: &'static str,
        schema_version: u32,
        scope_id: &'a str,
        target_identity_sha256: &'a str,
        wrong_identity_sha256: &'a str,
        capability_pre_observation_sha256: &'a str,
        wrapper_identity_sha256: &'a str,
        current_lock_identity_sha256: &'a str,
        terminal_latch_identity_sha256: &'a str,
    }
    let fixed = fixed_repetition(repetition);
    let target = security
        .read_disposable_signer(
            &compiled_disposable_target_config(fixed)?,
            DisposableAclKind::Target,
        )?
        .context("exact disposable target disappeared during root observation")?;
    let wrong = security
        .read_disposable_signer(
            &compiled_disposable_wrong_config(fixed)?,
            DisposableAclKind::WrongSurrogate,
        )?
        .context("exact disposable wrong key disappeared during root observation")?;
    if target != creation.target || wrong != creation.wrong {
        bail!("exact disposable key identity or strict SecAccess digest changed")
    }
    let wrapper_identity_sha256 =
        runner_wrapper_physical_digest(Path::new(&crate::publisher::wrapper_path(fixed)))?;
    let current_lock_identity_sha256 = runner_exact_file_observation(
        &disposable_current_lock_path(repetition),
        DISPOSABLE_CURRENT_LOCK_BYTES,
    )?;
    let terminal_latch_identity_sha256 = runner_exact_file_observation(
        &disposable_terminal_latch_path(repetition),
        DISPOSABLE_TERMINAL_LATCH_BYTES,
    )?;
    if wrapper_identity_sha256 != creation.protected_wrapper_identity_sha256
        || current_lock_identity_sha256 != creation.current_lock_identity_sha256
        || terminal_latch_identity_sha256 != creation.terminal_latch_identity_sha256
    {
        bail!("disposable wrapper/current-lock/latch physical identity changed")
    }
    let digest = document_sha256_v2(&Observation {
        schema_owner: "substrate.r3-macos-disposable-surrogate-observation",
        schema_version: EXPERIMENT_VERSION_V2,
        scope_id: repetition.scope_id(),
        target_identity_sha256: &target.identity_sha256,
        wrong_identity_sha256: &wrong.identity_sha256,
        capability_pre_observation_sha256: &creation.capability_pre_observation_sha256,
        wrapper_identity_sha256: &wrapper_identity_sha256,
        current_lock_identity_sha256: &current_lock_identity_sha256,
        terminal_latch_identity_sha256: &terminal_latch_identity_sha256,
    })?;
    if digest != creation.quiesced_observation_sha256 {
        bail!("fresh root surrogate observation differs from the signed creation baseline")
    }
    Ok(digest)
}

fn observe_exact_surrogate_state_under_ui(
    repetition: RepetitionV2,
    security: &mut NonInteractiveSecurity,
    creation: &SurrogateCreationReceiptV2,
    arm: &'static str,
    sequence_ordinal: u8,
    phase: &'static str,
) -> Result<String> {
    if !matches!(
        arm,
        "peer-substitution" | "dynamic-injection" | "nobody-owner"
    ) || !matches!(
        phase,
        "before" | "after" | "aggregate-before" | "aggregate-after"
    ) || (arm != "nobody-owner" && sequence_ordinal == 0)
        || sequence_ordinal > 4
    {
        bail!("root surrogate observation escaped its closed arm sequence")
    }
    let observed = observe_root_operation_with_raw(
        || observe_exact_surrogate_state(repetition, security, creation),
        |alert| {
            persist_root_operation_terminal_alert_with_context(
                "peer-surrogate-state",
                alert,
                Some((repetition, creation)),
            )
        },
    );
    let (result, report_sha256, report_bytes) = match observed {
        Ok(value) => value,
        Err(error) => {
            if let Some(alert) =
                error.downcast_ref::<crate::securityagent::SecurityAgentTerminalAlertV2>()
            {
                return Err(anyhow::Error::new(RunnerSecurityAgentAlertV2 {
                    repetition,
                    arm: arm.to_owned(),
                    evidence: alert.arm_evidence.clone(),
                }));
            }
            return Err(error);
        }
    };
    persist_root_operation_securityagent_report(
        &format!(
            "root-{arm}-{}-{sequence_ordinal:02}-{phase}.securityagent-observation.v2.json",
            repetition.ordinal()
        ),
        &report_sha256,
        &report_bytes,
    )?;
    result
}

fn disposable_current_lock_path(repetition: RepetitionV2) -> PathBuf {
    Path::new("/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/capability")
        .join(repetition.scope_id())
        .join("surrogate.lock")
}

fn disposable_terminal_latch_path(repetition: RepetitionV2) -> PathBuf {
    Path::new("/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/latches").join(
        format!("{}.retirement-terminal.v2.latch", repetition.scope_id()),
    )
}

fn runner_exact_file_observation(path: &Path, expected: &[u8]) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Observation<'a> {
        path: &'a str,
        device: u64,
        inode: u64,
        uid: u32,
        gid: u32,
        mode: u32,
        link_count: u64,
        size: u64,
        content_sha256: String,
    }
    let bytes = stable_read_file(path, 0, Some(libc::S_IFREG | 0o600))?;
    if bytes != expected {
        bail!("exact disposable lock/latch content changed")
    }
    let stat = lstat(path)?.context("exact disposable lock/latch disappeared")?;
    document_sha256_v2(&Observation {
        path: path
            .to_str()
            .context("disposable lock/latch path is not UTF-8")?,
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        uid: stat.st_uid,
        gid: stat.st_gid,
        mode: u32::from(stat.st_mode),
        link_count: stat.st_nlink as u64,
        size: stat.st_size as u64,
        content_sha256: sha256_hex_v2(&bytes),
    })
}

fn runner_wrapper_physical_digest(path: &Path) -> Result<String> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Observation<'a> {
        path: &'a str,
        device: u64,
        inode: u64,
        uid: u32,
        gid: u32,
        mode: u32,
        link_count: u64,
    }
    let stat = lstat(path)?.context("exact disposable wrapper is absent")?;
    if stat.st_uid != 0
        || stat.st_gid != 0
        || stat.st_nlink != 1
        || (stat.st_mode & (libc::S_IFMT | 0o7777)) != (libc::S_IFREG | 0o400)
    {
        bail!("exact disposable wrapper physical posture changed")
    }
    document_sha256_v2(&Observation {
        path: path
            .to_str()
            .context("disposable wrapper path is not UTF-8")?,
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        uid: stat.st_uid,
        gid: stat.st_gid,
        mode: u32::from(stat.st_mode),
        link_count: stat.st_nlink as u64,
    })
}

fn validate_ready_marker_shape(
    value: &PeerControlsReadyMarkerV2,
    repetition: RepetitionV2,
    request: &FinalizationRequestV2,
) -> Result<()> {
    repetition.validate_binding(value.repetition, &value.scope_id)?;
    if value.schema_owner != PEER_CONTROL_RECEIPT_OWNER_V2
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.request_digest != request.request_digest
        || value.request_control_receipt_sha256.len() != 12
        || value
            .request_control_receipt_sha256
            .iter()
            .any(|digest| !is_sha256(digest))
        || !is_sha256(&value.before_observation_sha256)
    {
        bail!("peer-controls ready marker changed its closed request/control shape")
    }
    Ok(())
}

fn publish_global_peer_restoration(
    inputs: &FrozenRunnerInputs,
    peer_sets: &[Option<PeerControlSetReceiptV2>; 2],
) -> Result<()> {
    let existing_path =
        global_external_exchange_path_v2(PublisherArtifactV2::PeerControlRestorationReceipt)?;
    let requests = RepetitionV2::ALL
        .into_iter()
        .map(|repetition| read_external(repetition, PublisherArtifactV2::FinalizationRequest))
        .collect::<Result<Vec<FinalizationRequestV2>>>()?;
    let completes = RepetitionV2::ALL
        .into_iter()
        .map(|repetition| read_external(repetition, PublisherArtifactV2::CompleteResponse))
        .collect::<Result<Vec<FinalizerResponseV2>>>()?;
    let sets = peer_sets
        .iter()
        .map(|value| value.clone().context("global peer set is absent"))
        .collect::<Result<Vec<_>>>()?;
    if let Some(bytes) = stable_read_file_optional(&existing_path, 0, Some(libc::S_IFREG | 0o444))?
    {
        let existing: PeerControlRestorationReceiptV2 = parse_canonical_v2(&bytes)?;
        existing.validate(&inputs.peer, &requests, &sets, &completes)?;
        return Ok(());
    }

    let frozen_alternate = FrozenCode {
        path: ALTERNATE_COORDINATOR_PATH_V2,
        signing_identifier: MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
        role: CandidateFreezeArtifactRoleV2::AlternateCoordinatorExecutable,
    };
    let (before, posture) = measure_frozen_executable_and_posture(frozen_alternate)?;
    if before != inputs.peer.alternate_path_identity
        || posture != inputs.peer.alternate_path_signing_posture
    {
        bail!("alternate coordinator identity changed before global exact removal")
    }
    let expected_bytes = stable_read_file(Path::new(MAC_R3_COORDINATOR_PATH_V2), 0, None)?;
    if sha256_hex_v2(&expected_bytes) != before.executable_sha256 {
        bail!("coordinator source bytes changed before alternate-path removal")
    }
    let (removal_result, securityagent_observation_sha256, securityagent_report_bytes) =
        observe_root_operation_with_raw(
            || {
                remove_exact_installed_file(
                    Path::new(ALTERNATE_COORDINATOR_PATH_V2),
                    Some(&expected_bytes),
                    Some(&before.physical_identity_sha256),
                )
            },
            |alert| persist_root_operation_terminal_alert("peer-control-global-restoration", alert),
        )?;
    persist_root_operation_securityagent_report(
        "peer-control-global-restoration.ui.observation.v2.json",
        &securityagent_observation_sha256,
        &securityagent_report_bytes,
    )?;
    removal_result?;
    let removal = AlternatePathRemovalObservationV2 {
        schema_owner: "substrate.r3-macos-disposable-alternate-path-removal-observation",
        schema_version: 2,
        path: ALTERNATE_COORDINATOR_PATH_V2,
        before_identity_sha256: document_sha256_v2(&before)?,
        before_physical_identity_sha256: before.physical_identity_sha256,
        exact_path_absent_after: !path_present(Path::new(ALTERNATE_COORDINATOR_PATH_V2))?,
    };
    if !removal.exact_path_absent_after {
        bail!("alternate coordinator path remains after exact global removal")
    }
    let securityagent_report = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(&securityagent_report_bytes),
        raw_report_sha256: securityagent_observation_sha256.clone(),
        raw_report_byte_length: u64::try_from(securityagent_report_bytes.len())?,
    };
    securityagent_report.validate()?;
    let receipt = PeerControlRestorationReceiptV2 {
        schema_owner: PEER_CONTROL_RECEIPT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        identity_packet_sha256: document_sha256_v2(&inputs.peer)?,
        runner_identity_sha256: inputs.runner_identity_sha256.clone(),
        runner_process_attestation_sha256: inputs.runner_process_attestation_sha256.clone(),
        alternate_path_identity_sha256: document_sha256_v2(&inputs.peer.alternate_path_identity)?,
        runner_control_set_sha256: sets
            .iter()
            .map(document_sha256_v2)
            .collect::<Result<Vec<_>>>()?,
        complete_response_sha256: completes
            .iter()
            .map(document_sha256_v2)
            .collect::<Result<Vec<_>>>()?,
        alternate_path_absent: true,
        exact_removal_observation_sha256: document_sha256_v2(&removal)?,
        securityagent_report,
        securityagent_observation_sha256,
    };
    receipt.validate(&inputs.peer, &requests, &sets, &completes)?;
    let bytes = canonical_bytes_v2(&receipt)?;
    write_runner_receipt("peer-controls-global-restoration.receipt.v2.json", &bytes)?;
    write_global_external_root_output(PublisherArtifactV2::PeerControlRestorationReceipt, &bytes)
}

fn build_securityagent_report_archive(
    global_pre_effect: &GlobalPreEffectPacketV2,
    peer_sets: &[Option<PeerControlSetReceiptV2>; 2],
) -> Result<SecurityAgentReportArchiveV2> {
    let store = ExperimentStoreV2::open_fixed()?;
    let (creator_receipts, creator_observation) = store
        .read_global_publisher_output::<CreatorRouteReceiptSetV2>(
            PublisherArtifactV2::CreatorRouteReceiptSet,
        )?;
    creator_receipts.validate(
        global_pre_effect,
        &read_peer_identity_packet_from_installed_path()?,
    )?;
    if creator_observation.sha256 != document_sha256_v2(&creator_receipts)? {
        bail!("creator receipt-set bytes changed before raw UI archive")
    }
    let mut bindings = Vec::new();
    push_securityagent_binding(
        &mut bindings,
        0,
        SecurityAgentEvidenceArmV2::GlobalNonceBaseline,
        document_sha256_v2(&global_pre_effect.nonce_absence_baseline)?,
        single_securityagent_arm_evidence_v2(
            global_pre_effect
                .nonce_absence_baseline
                .securityagent_report
                .clone(),
        )?,
    )?;
    for receipt in &creator_receipts.receipts {
        push_securityagent_binding(
            &mut bindings,
            receipt.repetition,
            SecurityAgentEvidenceArmV2::Creator {
                control: receipt.arm,
            },
            document_sha256_v2(receipt)?,
            single_securityagent_arm_evidence_v2(receipt.securityagent_report.clone())?,
        )?;
    }
    for repetition in RepetitionV2::ALL {
        let index = usize::from(repetition.ordinal() - 1);
        let peer_set = peer_sets[index]
            .as_ref()
            .context("raw UI archive lacks its exact peer-control set")?;
        for stage in PUBLISHER_STAGE_SEQUENCE_V2.into_iter().skip(1) {
            let (observation, file_observation) = store
                .read_publisher_output::<PublisherSecurityAgentObservationV2>(
                    repetition,
                    PublisherArtifactV2::SecurityAgentObservation(stage),
                )?;
            observation.validate(repetition, stage)?;
            if file_observation.sha256 != document_sha256_v2(&observation)? {
                bail!("publisher raw UI artifact changed before archive")
            }
            push_securityagent_binding(
                &mut bindings,
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::Publisher { stage },
                file_observation.sha256,
                observation.arm_evidence,
            )?;
        }
        let (transport, transport_observation) = store
            .read_canonical::<Vec<TransportControlReceiptV2>>(
                repetition,
                ExperimentArtifactV2::TransportControlReceipts,
            )?;
        if transport_observation.sha256 != document_sha256_v2(&transport)? {
            bail!("transport receipt set changed before raw UI archive")
        }
        for receipt in &transport {
            receipt.validate(repetition)?;
            push_securityagent_binding(
                &mut bindings,
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::Transport {
                    control: receipt.control,
                },
                document_sha256_v2(receipt)?,
                receipt.securityagent_evidence.clone(),
            )?;
        }
        for receipt in &peer_set.receipts {
            push_securityagent_binding(
                &mut bindings,
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::Peer {
                    control: receipt.control,
                },
                document_sha256_v2(receipt)?,
                receipt.securityagent_evidence.clone(),
            )?;
        }
        push_securityagent_binding(
            &mut bindings,
            repetition.ordinal(),
            SecurityAgentEvidenceArmV2::BenignInjection,
            document_sha256_v2(&peer_set.dynamic_library_injection)?,
            single_securityagent_arm_evidence_v2(
                peer_set
                    .dynamic_library_injection
                    .securityagent_report
                    .clone(),
            )?,
        )?;
        for attempt in &peer_set.nobody_owner_authority.attempts {
            push_securityagent_binding(
                &mut bindings,
                repetition.ordinal(),
                SecurityAgentEvidenceArmV2::NobodyOwner {
                    operation: attempt.operation,
                },
                document_sha256_v2(attempt)?,
                single_securityagent_arm_evidence_v2(attempt.securityagent_report.clone())?,
            )?;
        }
        let (effects, effects_observation) = store.read_canonical::<DisposableNativeArmReceiptV2>(
            repetition,
            ExperimentArtifactV2::EffectsNativeArmReceipt,
        )?;
        if effects_observation.sha256 != document_sha256_v2(&effects)? {
            bail!("effects native-arm receipt changed before raw UI archive")
        }
        push_securityagent_binding(
            &mut bindings,
            repetition.ordinal(),
            SecurityAgentEvidenceArmV2::FinalizationEffects,
            effects_observation.sha256,
            effects.securityagent_evidence,
        )?;
        let (terminal, terminal_observation) = store
            .read_canonical::<DisposableNativeArmReceiptV2>(
                repetition,
                ExperimentArtifactV2::CompleteNativeArmReceipt,
            )?;
        if terminal_observation.sha256 != document_sha256_v2(&terminal)? {
            bail!("terminal native-arm receipt changed before raw UI archive")
        }
        push_securityagent_binding(
            &mut bindings,
            repetition.ordinal(),
            SecurityAgentEvidenceArmV2::TerminalBinding,
            terminal_observation.sha256,
            terminal.securityagent_evidence,
        )?;
    }
    build_securityagent_report_archive_v2(bindings)
}

fn push_securityagent_binding(
    bindings: &mut Vec<SecurityAgentEvidenceBindingV2>,
    repetition: u8,
    arm: SecurityAgentEvidenceArmV2,
    source_artifact_sha256: String,
    arm_evidence: SecurityAgentArmEvidenceV2,
) -> Result<()> {
    arm_evidence.validate()?;
    bindings.push(SecurityAgentEvidenceBindingV2 {
        repetition,
        sequence_ordinal: u16::try_from(bindings.len() + 1)?,
        arm,
        source_artifact_sha256,
        arm_evidence,
    });
    Ok(())
}

fn read_peer_identity_packet_from_installed_path() -> Result<PeerControlIdentityPacketV2> {
    let bytes = stable_read_file(
        Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
        0,
        Some(libc::S_IFREG | 0o444),
    )?;
    let packet: PeerControlIdentityPacketV2 = parse_canonical_v2(&bytes)?;
    if bytes != canonical_bytes_v2(&packet)? {
        bail!("installed peer-control identity packet is not canonical")
    }
    Ok(packet)
}

fn build_repetition_native_evidence_exports() -> Result<(
    Vec<RepetitionNativeEvidenceExportV2>,
    Vec<FinalizerResponseV2>,
)> {
    let mut exports = Vec::with_capacity(RepetitionV2::ALL.len());
    let mut completes = Vec::with_capacity(RepetitionV2::ALL.len());
    for repetition in RepetitionV2::ALL {
        let complete: FinalizerResponseV2 =
            read_external(repetition, PublisherArtifactV2::CompleteResponse)?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(&complete)?;
        let journal = LockedJournal::open_existing_fixed(
            Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2),
            repetition.scope_id(),
            0,
        )?
        .context("completed repetition lacks its exact finalizer journal")?;
        let head = journal
            .head()?
            .context("completed repetition lacks its exact journal HEAD")?;
        if head.record.event.kind != JournalEventKind::Complete
            || head.sha256 != complete.journal_head_sha256
            || head.record.request_digest != complete.request_digest
        {
            bail!("completed response differs from its exact terminal journal head")
        }
        let mut generations = Vec::with_capacity(usize::try_from(head.generation)?);
        let mut effect_ordinals = Vec::new();
        for generation in 1..=head.generation {
            let record = journal.generation(generation)?;
            if record.event.kind == JournalEventKind::EffectObserved {
                effect_ordinals.push(
                    record
                        .event
                        .effect_ordinal
                        .context("EffectObserved generation lacks its effect ordinal")?,
                );
            }
            generations.push(canonical_bytes_v2(&record)?);
        }
        let mut observations = Vec::with_capacity(effect_ordinals.len());
        for ordinal in effect_ordinals {
            observations.push(
                journal
                    .effect_observation_artifact(ordinal)?
                    .context("EffectObserved journal generation lacks its canonical artifact")?,
            );
        }
        exports.push(build_repetition_native_evidence_export_v2(
            repetition,
            &complete,
            generations,
            observations,
        )?);
        completes.push(complete);
    }
    Ok((exports, completes))
}

fn publish_native_evidence_and_cleanup(
    global_pre_effect: &GlobalPreEffectPacketV2,
    service_observation: &FinalizerServiceObservationV2,
    peer_sets: &[Option<PeerControlSetReceiptV2>; 2],
    activation_membrane: &ActivationMembraneGuardV2,
) -> Result<NativeEvidenceCleanupReceiptV2> {
    let export_path = global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExport)?;
    let export = if let Some(export_bytes) = stable_read_file_optional_bounded(
        &export_path,
        0,
        Some(libc::S_IFREG | 0o444),
        MAX_NATIVE_EVIDENCE_DOCUMENT,
    )? {
        let export: NativeEvidenceExportV2 = parse_canonical_v2(&export_bytes)?;
        if canonical_bytes_v2(&export)? != export_bytes {
            bail!("durable native evidence export is not exact canonical bytes")
        }
        let completes = read_complete_responses()?;
        export.validate(&completes)?;
        export.validate_root_install_claims(global_pre_effect)?;
        export
    } else {
        let (repetitions, completes) = build_repetition_native_evidence_exports()?;
        let securityagent_archive =
            build_securityagent_report_archive(global_pre_effect, peer_sets)?;
        let runner_private_archive = build_runner_private_archive_excluding(&BTreeSet::new())?;
        let export = build_native_evidence_export_v2(
            repetitions,
            securityagent_archive,
            global_pre_effect,
            runner_private_archive,
        )?;
        export.validate(&completes)?;
        export.validate_root_install_claims(global_pre_effect)?;
        let export_bytes = canonical_bytes_v2(&export)?;
        write_global_external_root_output_bounded(
            PublisherArtifactV2::NativeEvidenceExport,
            &export_bytes,
            MAX_NATIVE_EVIDENCE_DOCUMENT,
        )?;
        if stable_read_file_bounded(
            &export_path,
            0,
            Some(libc::S_IFREG | 0o444),
            MAX_NATIVE_EVIDENCE_DOCUMENT,
        )? != export_bytes
        {
            bail!("native evidence export changed after durable external publication")
        }
        export
    };

    let store = ExperimentStoreV2::open_fixed()?;
    let (harness_attestation, harness_attestation_observation) = store
        .read_canonical::<CoordinatorProcessAttestationV2>(
        RepetitionV2::One,
        ExperimentArtifactV2::HarnessProcessAttestation,
    )?;
    if harness_attestation_observation.sha256 != document_sha256_v2(&harness_attestation)? {
        bail!("harness process attestation changed before native evidence acknowledgement")
    }
    let acknowledgement =
        wait_for_native_evidence_acknowledgement(&export, &harness_attestation_observation.sha256)?;
    perform_acknowledged_service_and_journal_cleanup(
        global_pre_effect,
        service_observation,
        &export,
        &acknowledgement,
        activation_membrane,
    )
}

fn recover_acknowledged_native_cleanup_after_root_absence(
    inputs: &FrozenRunnerInputs,
    root_install_claims: &RootInstallClaimsBindingV2,
    activation_membrane: &ActivationMembraneGuardV2,
) -> Result<()> {
    if !activation_membrane.root_absent_cleanup_recovery {
        bail!("native cleanup root-absence recovery lacks its explicit membrane state")
    }
    let (_, global_pre_effect) = activation_cleanup_recovery_documents()?
        .context("native cleanup recovery lacks its staged recovery documents")?;
    global_pre_effect.validate(&inputs.prepared, &inputs.candidate, &inputs.peer)?;
    if global_pre_effect.root_install_claims != *root_install_claims {
        bail!("native cleanup recovery root-install claims changed")
    }
    let service_observation: FinalizerServiceObservationV2 =
        read_runner_or_archived_canonical(FINALIZER_SERVICE_OBSERVATION_NAME)?;
    let peer_sets: [Option<PeerControlSetReceiptV2>; 2] = [None, None];
    let receipt = publish_native_evidence_and_cleanup(
        &global_pre_effect,
        &service_observation,
        &peer_sets,
        activation_membrane,
    )?;
    receipt.validate(
        &read_global_native_evidence_export()?,
        &read_native_evidence_acknowledgement_for_cleanup()?,
        &global_pre_effect,
    )?;
    println!("{}", String::from_utf8(canonical_bytes_v2(&receipt)?)?);
    Ok(())
}

fn resume_staged_native_cleanup_without_runner_root() -> Result<()> {
    let (receipt, export, _) = read_staged_native_cleanup_recovery_documents()?
        .context("absent runner root lacks its exact staged cleanup receipt")?;
    let bytes = canonical_bytes_v2(&receipt)?;
    stage_cleanup_receipt_remove_runner_root_and_publish(&receipt, &export, &bytes)?;
    println!("{}", String::from_utf8(bytes)?);
    Ok(())
}

fn read_global_native_evidence_export() -> Result<NativeEvidenceExportV2> {
    let path = global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExport)?;
    let bytes = stable_read_file_bounded(
        &path,
        0,
        Some(libc::S_IFREG | 0o444),
        MAX_NATIVE_EVIDENCE_DOCUMENT,
    )?;
    let value: NativeEvidenceExportV2 = parse_canonical_v2(&bytes)?;
    if canonical_bytes_v2(&value)? != bytes {
        bail!("native evidence export changed during cleanup recovery")
    }
    Ok(value)
}

fn read_native_evidence_acknowledgement_for_cleanup(
) -> Result<NativeEvidenceExportAcknowledgementV2> {
    let path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExportAcknowledgement)?;
    let bytes = stable_read_file(&path, DISPOSABLE_HARNESS_UID_V2, Some(EXTERNAL_INPUT_MODE))?;
    let value: NativeEvidenceExportAcknowledgementV2 = parse_canonical_v2(&bytes)?;
    if canonical_bytes_v2(&value)? != bytes {
        bail!("native evidence acknowledgement changed during cleanup recovery")
    }
    Ok(value)
}

fn read_complete_responses() -> Result<Vec<FinalizerResponseV2>> {
    let mut completes = Vec::with_capacity(RepetitionV2::ALL.len());
    for repetition in RepetitionV2::ALL {
        let complete: FinalizerResponseV2 =
            read_external(repetition, PublisherArtifactV2::CompleteResponse)?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(&complete)?;
        completes.push(complete);
    }
    Ok(completes)
}

fn wait_for_native_evidence_acknowledgement(
    export: &NativeEvidenceExportV2,
    harness_process_attestation_sha256: &str,
) -> Result<NativeEvidenceExportAcknowledgementV2> {
    let path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExportAcknowledgement)?;
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(bytes) =
            stable_read_file_optional(&path, DISPOSABLE_HARNESS_UID_V2, Some(EXTERNAL_INPUT_MODE))?
        {
            let acknowledgement: NativeEvidenceExportAcknowledgementV2 =
                parse_canonical_v2(&bytes)?;
            if bytes != canonical_bytes_v2(&acknowledgement)? {
                bail!("native evidence acknowledgement is not exact canonical bytes")
            }
            acknowledgement.validate(export, harness_process_attestation_sha256)?;
            return Ok(acknowledgement);
        }
        if std::time::Instant::now() >= deadline {
            bail!("timed out waiting for exact UID501 native evidence acknowledgement")
        }
        thread::sleep(POLL);
    }
}

fn perform_acknowledged_service_and_journal_cleanup(
    global_pre_effect: &GlobalPreEffectPacketV2,
    service_observation: &FinalizerServiceObservationV2,
    export: &NativeEvidenceExportV2,
    acknowledgement: &NativeEvidenceExportAcknowledgementV2,
    activation_membrane: &ActivationMembraneGuardV2,
) -> Result<NativeEvidenceCleanupReceiptV2> {
    let global_path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceCleanupReceipt)?;
    if let Some(bytes) = stable_read_file_optional_bounded(
        &global_path,
        0,
        Some(libc::S_IFREG | 0o444),
        MAX_NATIVE_EVIDENCE_DOCUMENT,
    )? {
        let existing: NativeEvidenceCleanupReceiptV2 = parse_canonical_v2(&bytes)?;
        existing.validate(export, acknowledgement, global_pre_effect)?;
        return Ok(existing);
    }
    if let Some((receipt, staged_export, packet)) = read_staged_native_cleanup_recovery_documents()?
    {
        if staged_export != *export || packet != *global_pre_effect {
            bail!("staged native cleanup recovery differs from its validated documents")
        }
        let bytes = canonical_bytes_v2(&receipt)?;
        stage_cleanup_receipt_remove_runner_root_and_publish(&receipt, export, &bytes)?;
        return Ok(receipt);
    }
    let cleanup_plan =
        load_or_create_native_evidence_cleanup_plan(global_pre_effect, export, acknowledgement)?;
    let prepared = FinalizerBootoutPreparedV2 {
        schema_owner: NATIVE_EVIDENCE_EXPORT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        launchd_label: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_owned(),
        service_observation_sha256: document_sha256_v2(service_observation)?,
        native_evidence_export_sha256: document_sha256_v2(export)?,
        acknowledgement_sha256: document_sha256_v2(acknowledgement)?,
        invocation_may_begin: true,
    };
    let existing = read_runner_private_optional::<FinalizerBootoutPreparedV2>(
        FINALIZER_BOOTOUT_PREPARED_NAME,
    )?;
    let created_now = match existing {
        Some(existing) if existing == prepared => false,
        Some(_) => bail!("finalizer bootout prepared cursor changed"),
        None => {
            write_runner_receipt(
                FINALIZER_BOOTOUT_PREPARED_NAME,
                &canonical_bytes_v2(&prepared)?,
            )?;
            true
        }
    };
    let durable_result =
        read_runner_private_optional::<FinalizerBootoutResultV2>(FINALIZER_BOOTOUT_RESULT_NAME)?;
    if !created_now && durable_result.is_none() {
        bail!(
            "finalizer bootout was prepared without a durable result; refusing blind reinvocation"
        )
    }
    let (native, report_sha256, report_bytes) = observe_root_operation_with_raw(
        || {
            let bootout = match durable_result {
                Some(result) => {
                    validate_bootout_result(&result, &prepared)?;
                    result
                }
                None => {
                    let target = format!("system/{}", MAC_R3_FINALIZER_LAUNCHD_LABEL_V2);
                    let output = Command::new("/bin/launchctl")
                        .arg("bootout")
                        .arg(&target)
                        .current_dir("/")
                        .env_clear()
                        .stdin(Stdio::null())
                        .output()?;
                    let result = FinalizerBootoutResultV2 {
                        schema_owner: NATIVE_EVIDENCE_EXPORT_OWNER_V2.to_owned(),
                        schema_version: EXPERIMENT_VERSION_V2,
                        experiment_id: EXPERIMENT_ID_V2.to_owned(),
                        prepared_sha256: document_sha256_v2(&prepared)?,
                        exit_status: output
                            .status
                            .code()
                            .context("launchctl bootout was signaled")?,
                        stdout: bounded_raw_stream_evidence(&output.stdout)?,
                        stderr: bounded_raw_stream_evidence(&output.stderr)?,
                    };
                    write_runner_receipt(
                        FINALIZER_BOOTOUT_RESULT_NAME,
                        &canonical_bytes_v2(&result)?,
                    )?;
                    validate_bootout_result(&result, &prepared)?;
                    result
                }
            };
            let (print_exit_status, print_stdout, print_stderr) =
                observe_exact_launchd_service_absence()?;
            require_exact_path_absent(Path::new(MAC_R3_FINALIZER_ENDPOINT_V2))?;
            let process_absence =
                require_exact_process_path_absent(Path::new(MAC_R3_FINALIZER_PATH_V2))?;
            let endpoint_absence_observation_sha256 = document_sha256_v2(&(
                "finalizer-service-absent-v2",
                MAC_R3_FINALIZER_ENDPOINT_V2,
                libc::ENOENT,
                process_absence,
            ))?;
            let coordinator_inbox_observation_sha256 = observe_empty_coordinator_inbox()?;
            let (
                journal_absence_observation_sha256,
                capability_absence_observation_sha256,
                latch_absence_observation_sha256,
                journal_root_lock_absence_observation_sha256,
                journal_root_absence_observation_sha256,
            ) = remove_exported_journal_root(
                global_pre_effect,
                export,
                activation_membrane,
                &cleanup_plan,
            )?;
            let root_install_claims_retention_observation_sha256 =
                observe_retained_root_install_claims(&global_pre_effect.root_install_claims)?;
            Ok(CleanupNativeOutcomeV2 {
                bootout,
                print_exit_status,
                print_stdout,
                print_stderr,
                endpoint_absence_observation_sha256,
                coordinator_inbox_observation_sha256,
                journal_absence_observation_sha256,
                capability_absence_observation_sha256,
                latch_absence_observation_sha256,
                journal_root_lock_absence_observation_sha256,
                journal_root_absence_observation_sha256,
                root_install_claims_retention_observation_sha256,
            })
        },
        |alert| persist_root_operation_terminal_alert("native-evidence-cleanup", alert),
    )?;
    persist_root_operation_securityagent_report(
        "native-evidence-cleanup.ui.observation.v2.json",
        &report_sha256,
        &report_bytes,
    )?;
    let native = native?;
    let securityagent_report = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(&report_bytes),
        raw_report_sha256: report_sha256,
        raw_report_byte_length: u64::try_from(report_bytes.len())?,
    };
    securityagent_report.validate()?;
    let scopes = RepetitionV2::ALL
        .into_iter()
        .map(|value| value.scope_id().to_owned())
        .collect::<Vec<_>>();
    let journal_scope_paths = scopes
        .iter()
        .map(|scope| format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/{scope}"))
        .collect::<Vec<_>>();
    let capability_scope_paths = scopes
        .iter()
        .map(|scope| format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/capability/{scope}"))
        .collect::<Vec<_>>();
    let latch_paths = scopes
        .iter()
        .map(|scope| {
            format!("{MAC_R3_RETIREMENT_LATCH_ROOT_V2}/{scope}.retirement-terminal.v2.latch")
        })
        .collect::<Vec<_>>();
    let export_names = export
        .runner_private_archive
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<BTreeSet<_>>();
    let cleanup_private_archive = build_runner_private_archive_excluding(&export_names)?;
    let cleanup_private_archive_sha256 = document_sha256_v2(&cleanup_private_archive)?;
    let (
        runner_private_archive_union_sha256,
        runner_private_archive_union_cardinality,
        runner_private_archive_total_bytes,
    ) = runner_private_archive_union_v2(&export.runner_private_archive, &cleanup_private_archive)?;
    let native_cleanup_plan_sha256 = cleanup_private_archive
        .entries
        .iter()
        .find(|entry| entry.name == NATIVE_CLEANUP_PLAN_NAME)
        .map(|entry| entry.canonical_sha256.clone())
        .context("cleanup private archive lacks its exact cleanup plan")?;
    let native_cleanup_step_sha256 = cleanup_private_archive
        .entries
        .iter()
        .filter(|entry| {
            entry.name.starts_with(NATIVE_CLEANUP_STEP_PREFIX)
                && entry.name.ends_with(".observed.v2.json")
        })
        .map(|entry| entry.canonical_sha256.clone())
        .collect::<Vec<_>>();
    if native_cleanup_step_sha256.len() != cleanup_plan.objects.len() {
        bail!("cleanup private archive does not bind every cleanup plan step")
    }
    let runner_root_stat = lstat(Path::new(RUNNER_ROOT))?
        .context("runner root disappeared before success archive closure")?;
    let runner_root_stable_identity_sha256 =
        document_sha256_v2(&TerminalRunnerRootStableIdentityV2 {
            path: RUNNER_ROOT,
            device: runner_root_stat.st_dev as u64,
            inode: runner_root_stat.st_ino,
            owner_uid: runner_root_stat.st_uid,
            owner_gid: runner_root_stat.st_gid,
            mode: u32::from(runner_root_stat.st_mode),
        })?;
    let runner_root_absence_observation_sha256 = document_sha256_v2(&(
        "substrate.r3-macos-success-runner-root-absent.v2",
        RUNNER_ROOT,
        &runner_root_stable_identity_sha256,
        libc::ENOENT,
    ))?;
    let mut receipt = NativeEvidenceCleanupReceiptV2 {
        schema_owner: NATIVE_EVIDENCE_EXPORT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        native_evidence_export_sha256: document_sha256_v2(export)?,
        acknowledgement_sha256: document_sha256_v2(acknowledgement)?,
        launchd_label: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_owned(),
        launchctl_bootout_exit_status: native.bootout.exit_status,
        launchctl_bootout_stdout: native.bootout.stdout,
        launchctl_bootout_stderr: native.bootout.stderr,
        launchctl_print_not_found_exit_status: native.print_exit_status,
        launchctl_print_stdout: native.print_stdout,
        launchctl_print_stderr: native.print_stderr,
        endpoint_path: MAC_R3_FINALIZER_ENDPOINT_V2.to_owned(),
        endpoint_absent_without_cleanup_unlink: true,
        endpoint_absence_observation_sha256: native.endpoint_absence_observation_sha256,
        coordinator_inbox_root: MAC_R3_COORDINATOR_INBOX_ROOT_V2.to_owned(),
        request_inbox_path: MAC_R3_FINALIZER_REQUEST_PATH_V2.to_owned(),
        terminal_inbox_path: MAC_R3_TERMINAL_BINDING_PATH_V2.to_owned(),
        request_inbox_absent: true,
        terminal_inbox_absent: true,
        coordinator_inbox_empty: true,
        coordinator_inbox_observation_sha256: native.coordinator_inbox_observation_sha256,
        scope_ids: scopes,
        journal_scope_paths,
        journal_scope_absent: vec![true; RepetitionV2::ALL.len()],
        capability_scope_paths,
        capability_scope_absent: vec![true; RepetitionV2::ALL.len()],
        latch_paths,
        latch_absent: vec![true; RepetitionV2::ALL.len()],
        journal_absence_observation_sha256: native.journal_absence_observation_sha256,
        capability_absence_observation_sha256: native.capability_absence_observation_sha256,
        latch_absence_observation_sha256: native.latch_absence_observation_sha256,
        journal_root_path: MAC_R3_FINALIZER_JOURNAL_ROOT_V2.to_owned(),
        journal_root_activation_identity_sha256: global_pre_effect
            .activation_membrane_root_identity_sha256
            .clone(),
        journal_root_lock_path: MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2.to_owned(),
        journal_root_lock_identity_sha256: global_pre_effect
            .journal_root_lock_identity_sha256
            .clone(),
        journal_root_lock_absent: true,
        journal_root_lock_absence_observation_sha256: native
            .journal_root_lock_absence_observation_sha256,
        journal_root_empty_before_removal: true,
        journal_root_removed_via_held_parent_dirfd: true,
        journal_root_parent_fsynced: true,
        journal_root_absent: true,
        journal_root_absence_observation_sha256: native.journal_root_absence_observation_sha256,
        root_install_claim_root_path: ROOT_INSTALL_CLAIM_ROOT_V2.to_owned(),
        root_install_preclaim_path: ROOT_INSTALL_PRECLAIM_PATH_V2.to_owned(),
        root_install_completion_path: ROOT_INSTALL_COMPLETION_PATH_V2.to_owned(),
        root_install_claims_binding_sha256: export.root_install_claims_binding_sha256.clone(),
        root_install_preclaim_sha256: export.root_install_preclaim_sha256.clone(),
        root_install_completion_sha256: export.root_install_completion_sha256.clone(),
        root_install_claim_root_identity_sha256: document_sha256_v2(
            &global_pre_effect
                .root_install_claims
                .claim_root_current_identity,
        )?,
        root_install_claims_retained: true,
        admin_cleanup_authorized: true,
        root_install_claims_retention_observation_sha256: native
            .root_install_claims_retention_observation_sha256,
        cleanup_private_archive,
        cleanup_private_archive_sha256,
        runner_private_archive_union_sha256,
        runner_private_archive_union_cardinality,
        runner_private_archive_total_bytes,
        runner_private_archive_max_bytes: RUNNER_PRIVATE_ARCHIVE_MAX_BYTES_V2,
        native_cleanup_plan_sha256,
        native_cleanup_step_sha256,
        runner_root_path: RUNNER_ROOT.to_owned(),
        runner_root_stable_identity_sha256,
        runner_root_empty_before_removal: true,
        runner_root_removed_via_held_parent_dirfd: true,
        runner_root_parent_fsynced: true,
        runner_root_absent: true,
        runner_root_absence_observation_sha256,
        securityagent_report,
        cleanup_artifact_set_sha256: String::new(),
    };
    receipt.cleanup_artifact_set_sha256 = native_evidence_cleanup_artifact_set_sha256_v2(&receipt)?;
    receipt.validate(export, acknowledgement, global_pre_effect)?;
    let bytes = canonical_bytes_v2(&receipt)?;
    stage_cleanup_receipt_remove_runner_root_and_publish(&receipt, export, &bytes)?;
    Ok(receipt)
}

fn stage_cleanup_receipt_remove_runner_root_and_publish(
    receipt: &NativeEvidenceCleanupReceiptV2,
    export: &NativeEvidenceExportV2,
    receipt_bytes: &[u8],
) -> Result<()> {
    let final_path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceCleanupReceipt)?;
    let identity = PublishIdentityV2 {
        owner_uid: 0,
        owner_gid: 0,
        permissions: 0o444,
        parent_uid: DISPOSABLE_HARNESS_UID_V2,
        parent_gid: 0,
        parent_mode: libc::S_IFDIR | 0o700,
        maximum_bytes: MAX_NATIVE_EVIDENCE_DOCUMENT,
    };
    stage_exact_file(&final_path, receipt_bytes, identity)?;

    let mut entries = export.runner_private_archive.entries.clone();
    entries.extend(receipt.cleanup_private_archive.entries.clone());
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    if entries.len() != usize::try_from(receipt.runner_private_archive_union_cardinality)?
        || document_sha256_v2(&entries)? != receipt.runner_private_archive_union_sha256
    {
        bail!("runner-root cleanup entries changed after external staging")
    }
    let root = Path::new(RUNNER_ROOT);
    let Some(root_stat) = lstat(root)? else {
        publish_exact_file(&final_path, receipt_bytes, identity, PublishMode::Immutable)?;
        return Ok(());
    };
    if document_sha256_v2(&TerminalRunnerRootStableIdentityV2 {
        path: RUNNER_ROOT,
        device: root_stat.st_dev as u64,
        inode: root_stat.st_ino,
        owner_uid: root_stat.st_uid,
        owner_gid: root_stat.st_gid,
        mode: u32::from(root_stat.st_mode),
    })? != receipt.runner_root_stable_identity_sha256
    {
        bail!("runner root stable identity changed before prefix cleanup")
    }
    let mut live_suffix_seen = false;
    for entry in &entries {
        let present = path_present(&root.join(&entry.name))?;
        if present {
            live_suffix_seen = true;
        } else if live_suffix_seen {
            bail!("runner-root cleanup is not one exact contiguous removed prefix")
        }
    }
    for entry in &entries {
        let path = root.join(&entry.name);
        if !path_present(&path)? {
            continue;
        }
        let expected = entry.validate()?;
        let bytes = stable_read_file_bounded(
            &path,
            0,
            Some(libc::S_IFREG | 0o600),
            runner_private_archive_entry_max_bytes_v2(&entry.name),
        )?;
        let stat = lstat(&path)?.context("runner-root cleanup child disappeared")?;
        if bytes != expected
            || runner_file_physical_identity_sha256(&stat)? != entry.physical_identity_sha256
        {
            bail!("runner-root cleanup child differs from its archived identity")
        }
        remove_exact_owned_file(
            &path,
            Some(&expected),
            Some(&entry.physical_identity_sha256),
            0,
            Some(libc::S_IFREG | 0o600),
        )?;
    }
    if !exact_directory_names(root)?.is_empty() {
        bail!("runner root is not empty after exact archived-prefix cleanup")
    }
    let root_after = lstat(root)?.context("runner root disappeared before exact rmdir")?;
    if cleanup_directory_stable_identity_sha256(root, &root_after)?
        != receipt.runner_root_stable_identity_sha256
    {
        bail!("runner root identity changed during exact archived-prefix cleanup")
    }
    remove_exact_empty_directory(root, 0, 0, libc::S_IFDIR | 0o700)?;
    require_exact_path_absent(root)?;
    fsync_exact_parent(root)?;
    publish_exact_file(&final_path, receipt_bytes, identity, PublishMode::Immutable)?;
    Ok(())
}

fn observe_retained_root_install_claims(claims: &RootInstallClaimsBindingV2) -> Result<String> {
    let root = Path::new(ROOT_INSTALL_CLAIM_ROOT_V2);
    let root_fd = open_exact_directory(root, 0, 0, libc::S_IFDIR | 0o700)?;
    let held_root = fstat(root_fd.as_raw_fd())?;
    let root_identity = root_install_physical_identity(ROOT_INSTALL_CLAIM_ROOT_V2, &held_root)?;
    if root_identity != claims.claim_root_current_identity {
        bail!("root-install claim root identity changed before admin cleanup authorization")
    }
    let preclaim_identity = observe_retained_root_install_claim_leaf(
        root_fd.as_raw_fd(),
        Path::new(ROOT_INSTALL_PRECLAIM_PATH_V2),
        &canonical_bytes_v2(&claims.preclaim)?,
        &claims.completion.preclaim_leaf_identity,
    )?;
    let completion_identity = observe_retained_root_install_claim_leaf(
        root_fd.as_raw_fd(),
        Path::new(ROOT_INSTALL_COMPLETION_PATH_V2),
        &canonical_bytes_v2(&claims.completion)?,
        &claims.completion_leaf_identity,
    )?;
    let held_after = fstat(root_fd.as_raw_fd())?;
    let pathname_after =
        lstat(root)?.context("root-install claim root disappeared during retention")?;
    if !same_stat(&held_root, &held_after) || !same_stat(&held_root, &pathname_after) {
        bail!("root-install claim root changed during retention observation")
    }
    document_sha256_v2(&(
        "substrate.r3-macos-root-install-claims-retained-for-admin-cleanup.v2",
        document_sha256_v2(&claims.claim_root_current_identity)?,
        &claims.preclaim_sha256,
        &claims.completion_sha256,
        preclaim_identity,
        completion_identity,
    ))
}

fn observe_retained_root_install_claim_leaf(
    root_fd: i32,
    path: &Path,
    expected_bytes: &[u8],
    expected_identity: &RootInstallPhysicalIdentityV2,
) -> Result<RootInstallPhysicalIdentityV2> {
    let leaf = CString::new(
        path.file_name()
            .context("root-install claim leaf lacks a file name")?
            .as_bytes(),
    )?;
    let before = fstatat_nofollow(root_fd, &leaf)?
        .context("root-install claim leaf is absent before admin cleanup authorization")?;
    let identity = root_install_physical_identity(
        path.to_str()
            .context("root-install claim path is not UTF-8")?,
        &before,
    )?;
    if identity != *expected_identity
        || stable_read_file(path, 0, Some(libc::S_IFREG | 0o400))? != expected_bytes
    {
        bail!("root-install claim leaf changed before admin cleanup authorization")
    }
    let after = fstatat_nofollow(root_fd, &leaf)?
        .context("root-install claim leaf disappeared during retention observation")?;
    if !same_stat(&before, &after) {
        bail!("root-install claim leaf changed during retention observation")
    }
    Ok(identity)
}

fn terminal_finalizer_failure_cleanup(
    global_pre_effect: Option<&GlobalPreEffectPacketV2>,
    activation_membrane: &ActivationMembraneGuardV2,
    failure: &str,
) -> Result<()> {
    let accepted = exact_accepted_authority_before_terminal_cleanup()
        .context("classify finalizer authority before terminal service cleanup")?;
    if !accepted.is_empty() {
        bail!(
            "terminal service cleanup is forbidden after authenticated FinalizerAccepted authority; same-digest rejoin evidence {}",
            document_sha256_v2(&accepted)?
        )
    }
    let failure_sha256 = sha256_hex_v2(failure.as_bytes());
    let (terminal_cleanup, report_sha256, report_bytes) = observe_root_operation_with_raw(
        || {
            let target = format!("system/{}", MAC_R3_FINALIZER_LAUNCHD_LABEL_V2);
            let initial = Command::new("/bin/launchctl")
                .arg("print")
                .arg(&target)
                .current_dir("/")
                .env_clear()
                .stdin(Stdio::null())
                .output()?;
            let initial_status = initial
                .status
                .code()
                .context("launchctl failure-state print was signaled")?;
            let loaded = if initial_status == 0 {
                let stdout = std::str::from_utf8(&initial.stdout)?;
                if !initial.stderr.is_empty()
                    || !stdout.contains(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)
                    || !stdout.contains(MAC_R3_FINALIZER_PLIST_PATH_V2)
                    || !stdout.contains(MAC_R3_FINALIZER_PATH_V2)
                {
                    bail!("terminal cleanup found an alternate loaded launchd service")
                }
                true
            } else if initial_status == 113 {
                if !initial.stdout.is_empty()
                    || initial.stderr
                        != exact_launchctl_service_not_found_stderr(
                            MAC_R3_FINALIZER_LAUNCHD_LABEL_V2,
                        )
                        .as_bytes()
                {
                    bail!("terminal cleanup found an alternate launchd absence classification")
                }
                false
            } else {
                bail!("terminal cleanup could not classify the exact launchd service")
            };
            let bootout = if loaded {
                let output = Command::new("/bin/launchctl")
                    .arg("bootout")
                    .arg(&target)
                    .current_dir("/")
                    .env_clear()
                    .stdin(Stdio::null())
                    .output()?;
                let result = FinalizerBootoutResultV2 {
                    schema_owner: NATIVE_EVIDENCE_EXPORT_OWNER_V2.to_owned(),
                    schema_version: EXPERIMENT_VERSION_V2,
                    experiment_id: EXPERIMENT_ID_V2.to_owned(),
                    prepared_sha256: failure_sha256.clone(),
                    exit_status: output
                        .status
                        .code()
                        .context("terminal launchctl bootout was signaled")?,
                    stdout: bounded_raw_stream_evidence(&output.stdout)?,
                    stderr: bounded_raw_stream_evidence(&output.stderr)?,
                };
                if result.exit_status != 0
                    || !result.stdout.validate()?.is_empty()
                    || !result.stderr.validate()?.is_empty()
                {
                    bail!("terminal launchctl bootout did not return exact success")
                }
                Some(result)
            } else {
                None
            };
            let (print_status, print_stdout, print_stderr) =
                observe_exact_launchd_service_absence()?;
            require_exact_path_absent(Path::new(MAC_R3_FINALIZER_ENDPOINT_V2))?;
            let process_absence =
                require_exact_process_path_absent(Path::new(MAC_R3_FINALIZER_PATH_V2))?;
            Ok((
                loaded,
                bootout,
                print_status,
                print_stdout,
                print_stderr,
                process_absence,
            ))
        },
        |alert| persist_root_operation_terminal_alert("terminal-finalizer-failure-cleanup", alert),
    )?;
    persist_root_operation_securityagent_report(
        "terminal-finalizer-failure-cleanup.ui.observation.v2.json",
        &report_sha256,
        &report_bytes,
    )?;
    let (loaded, bootout, print_status, print_stdout, print_stderr, process_absence) =
        terminal_cleanup?;
    let securityagent_report = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(&report_bytes),
        raw_report_sha256: report_sha256,
        raw_report_byte_length: u64::try_from(report_bytes.len())?,
    };
    securityagent_report.validate()?;
    let journal_root = terminal_journal_root_disposition(global_pre_effect, activation_membrane)?;
    let receipt = TerminalFinalizerFailureCleanupReceiptV2 {
        schema_owner: "substrate.r3-macos-disposable-terminal-service-failure-cleanup".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        failure_sha256,
        launchd_service_was_loaded: loaded,
        launchctl_bootout: bootout,
        launchctl_print_exit_status: print_status,
        launchctl_print_stdout: print_stdout,
        launchctl_print_stderr: print_stderr,
        endpoint_absent: true,
        finalizer_process_absent_sha256: process_absence,
        journal_root_absent: journal_root.absent,
        journal_root_preserved_for_unacknowledged_evidence: journal_root
            .preserved_for_unacknowledged_evidence,
        journal_root_disposition_observation_sha256: journal_root.observation_sha256,
        securityagent_report,
    };
    write_runner_receipt(
        "terminal-finalizer-failure-cleanup.v2.json",
        &canonical_bytes_v2(&receipt)?,
    )?;
    if !journal_root.absent {
        bail!("terminal cleanup stopped after bootout because unacknowledged journal evidence remains")
    }
    Ok(())
}

fn validate_terminal_finalizer_launchd_cleanup_evidence(
    service_was_loaded: bool,
    bootout: Option<&FinalizerBootoutResultV2>,
    print_exit_status: i32,
    print_stdout: &BoundedRawStreamEvidenceV2,
    print_stderr: &BoundedRawStreamEvidenceV2,
    failure_sha256: &str,
) -> Result<()> {
    let print_stdout = print_stdout.validate()?;
    let print_stderr = print_stderr.validate()?;
    if print_exit_status != 113
        || !print_stdout.is_empty()
        || print_stderr
            != exact_launchctl_service_not_found_stderr(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)
                .as_bytes()
    {
        bail!("durable terminal cleanup print evidence is not the exact service-not-found tuple")
    }
    match (service_was_loaded, bootout) {
        (false, None) => Ok(()),
        (true, Some(result)) => {
            if result.schema_owner != NATIVE_EVIDENCE_EXPORT_OWNER_V2
                || result.schema_version != EXPERIMENT_VERSION_V2
                || result.experiment_id != EXPERIMENT_ID_V2
                || result.prepared_sha256 != failure_sha256
                || result.exit_status != 0
                || !result.stdout.validate()?.is_empty()
                || !result.stderr.validate()?.is_empty()
            {
                bail!("durable terminal cleanup bootout evidence is not the exact success tuple")
            }
            Ok(())
        }
        _ => bail!("durable terminal cleanup bootout presence changed loaded-service history"),
    }
}

fn terminal_finalizer_cleanup_receipt_sha256(
    global_pre_effect: Option<&GlobalPreEffectPacketV2>,
    activation_membrane: &ActivationMembraneGuardV2,
    failure_binding: &str,
) -> Result<String> {
    if let Some(bytes) =
        read_runner_private_bytes_optional("terminal-finalizer-failure-cleanup.v2.json")?
    {
        let receipt: TerminalFinalizerFailureCleanupReceiptV2 = parse_canonical_v2(&bytes)?;
        validate_terminal_finalizer_launchd_cleanup_evidence(
            receipt.launchd_service_was_loaded,
            receipt.launchctl_bootout.as_ref(),
            receipt.launchctl_print_exit_status,
            &receipt.launchctl_print_stdout,
            &receipt.launchctl_print_stderr,
            &receipt.failure_sha256,
        )?;
        if canonical_bytes_v2(&receipt)? != bytes
            || receipt.schema_owner
                != "substrate.r3-macos-disposable-terminal-service-failure-cleanup"
            || receipt.schema_version != EXPERIMENT_VERSION_V2
            || receipt.experiment_id != EXPERIMENT_ID_V2
            || receipt.failure_sha256 != sha256_hex_v2(failure_binding.as_bytes())
            || receipt.launchctl_print_exit_status != 113
            || !receipt.endpoint_absent
            || !receipt.journal_root_absent
            || receipt.journal_root_preserved_for_unacknowledged_evidence
            || !is_sha256(&receipt.finalizer_process_absent_sha256)
            || !is_sha256(&receipt.journal_root_disposition_observation_sha256)
        {
            bail!("terminal finalizer cleanup receipt changed during UI recovery")
        }
        receipt.securityagent_report.validate()?;
        return Ok(sha256_hex_v2(&bytes));
    }
    terminal_finalizer_failure_cleanup(global_pre_effect, activation_membrane, failure_binding)?;
    let bytes = read_runner_private_bytes_optional("terminal-finalizer-failure-cleanup.v2.json")?
        .context("terminal finalizer cleanup returned without its durable receipt")?;
    Ok(sha256_hex_v2(&bytes))
}

fn complete_creator_terminal_failure(
    inputs: &FrozenRunnerInputs,
    global_pre_effect: Option<&GlobalPreEffectPacketV2>,
    activation_membrane: &ActivationMembraneGuardV2,
    root_install_claims: &RootInstallClaimsBindingV2,
) -> Result<TerminalAdminCleanupAuthorizationV2> {
    let marker = MarkerRoot::open()?;
    let rollback_cursor = marker
        .read_rollback()?
        .context("creator terminal cleanup lacks its durable rollback cursor")?;
    let rollback_bytes =
        read_runner_private_bytes_optional("creator-emergency-rollback.receipt.v2.json")?
            .context("creator terminal cleanup lacks its durable rollback receipt")?;
    let rollback_receipt: CreatorRollbackReceiptV2 = parse_canonical_v2(&rollback_bytes)?;
    rollback_receipt.validate(&rollback_cursor)?;
    let rollback_receipt_sha256 = sha256_hex_v2(&rollback_bytes);
    let expected_cursor = CreatorTerminalCleanupCursorV2 {
        schema_owner: "substrate.r3-macos-disposable-creator-terminal-cleanup-cursor".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        creator_rollback_receipt_sha256: rollback_receipt_sha256.clone(),
        global_pre_effect_packet_sha256: global_pre_effect.map(document_sha256_v2).transpose()?,
        root_install_claims_binding_sha256: document_sha256_v2(root_install_claims)?,
        cleanup_may_begin: true,
        terminal_no_normal_resume: true,
    };
    validate_creator_terminal_cleanup_cursor(&expected_cursor, root_install_claims)?;
    let cursor = match read_runner_private_optional::<CreatorTerminalCleanupCursorV2>(
        CREATOR_TERMINAL_CLEANUP_CURSOR_NAME,
    )? {
        Some(existing) if existing == expected_cursor => existing,
        Some(_) => bail!("creator terminal cleanup cursor changed its rollback authority"),
        None => {
            write_runner_receipt(
                CREATOR_TERMINAL_CLEANUP_CURSOR_NAME,
                &canonical_bytes_v2(&expected_cursor)?,
            )?;
            expected_cursor
        }
    };

    let finalizer_cleanup_sha256 = terminal_finalizer_cleanup_receipt_sha256(
        global_pre_effect,
        activation_membrane,
        &document_sha256_v2(&cursor)?,
    )?;
    remove_exact_installed_file(
        Path::new(ALTERNATE_COORDINATOR_PATH_V2),
        None,
        Some(&inputs.peer.alternate_path_identity.physical_identity_sha256),
    )?;
    remove_exact_installed_file(
        Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.prepared)?),
        None,
    )?;
    remove_exact_installed_file(
        Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.candidate)?),
        None,
    )?;
    remove_exact_installed_file(
        Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.peer)?),
        None,
    )?;
    let creator_marker_restoration_sha256 = restore_creator_marker_after_observed_rollback(
        &rollback_receipt,
        &rollback_receipt_sha256,
    )?;
    let claims_observation = observe_retained_root_install_claims(root_install_claims)?;
    let receipt = CreatorTerminalCleanupReceiptV2 {
        schema_owner: "substrate.r3-macos-disposable-creator-terminal-cleanup".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        cursor_sha256: document_sha256_v2(&cursor)?,
        creator_rollback_receipt_sha256: rollback_receipt_sha256.clone(),
        surrogate_restoration_sha256: rollback_receipt_sha256,
        finalizer_cleanup_sha256: finalizer_cleanup_sha256.clone(),
        creator_marker_restoration_sha256: creator_marker_restoration_sha256.clone(),
        installed_packets_absent: true,
        root_install_claims_binding_sha256: document_sha256_v2(root_install_claims)?,
        root_install_claims_retention_observation_sha256: claims_observation.clone(),
        claims_retained: true,
        admin_cleanup_authorized: true,
        terminal_no_normal_resume: true,
    };
    validate_creator_terminal_cleanup_receipt(&receipt, &cursor, root_install_claims)?;
    match read_runner_private_optional::<CreatorTerminalCleanupReceiptV2>(
        CREATOR_TERMINAL_RECEIPT_NAME,
    )? {
        Some(existing) if existing == receipt => {}
        Some(_) => bail!("creator terminal cleanup receipt changed during recovery"),
        None => write_runner_receipt(
            CREATOR_TERMINAL_RECEIPT_NAME,
            &canonical_bytes_v2(&receipt)?,
        )?,
    }
    persist_terminal_admin_cleanup_authorization(
        "creator_closed_failure",
        document_sha256_v2(&receipt)?,
        receipt.surrogate_restoration_sha256,
        finalizer_cleanup_sha256,
        creator_marker_restoration_sha256,
        root_install_claims,
        claims_observation,
    )
}

fn validate_creator_terminal_cleanup_cursor(
    value: &CreatorTerminalCleanupCursorV2,
    root_install_claims: &RootInstallClaimsBindingV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-creator-terminal-cleanup-cursor"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || !is_sha256(&value.creator_rollback_receipt_sha256)
        || value
            .global_pre_effect_packet_sha256
            .as_ref()
            .is_some_and(|digest| !is_sha256(digest))
        || value.root_install_claims_binding_sha256 != document_sha256_v2(root_install_claims)?
        || !value.cleanup_may_begin
        || !value.terminal_no_normal_resume
    {
        bail!("creator terminal cleanup cursor changed its closed authority")
    }
    Ok(())
}

fn validate_creator_terminal_cleanup_receipt(
    value: &CreatorTerminalCleanupReceiptV2,
    cursor: &CreatorTerminalCleanupCursorV2,
    root_install_claims: &RootInstallClaimsBindingV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-disposable-creator-terminal-cleanup"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.cursor_sha256 != document_sha256_v2(cursor)?
        || value.creator_rollback_receipt_sha256 != cursor.creator_rollback_receipt_sha256
        || value.surrogate_restoration_sha256 != value.creator_rollback_receipt_sha256
        || value.root_install_claims_binding_sha256 != document_sha256_v2(root_install_claims)?
        || !value.installed_packets_absent
        || !value.claims_retained
        || !value.admin_cleanup_authorized
        || !value.terminal_no_normal_resume
    {
        bail!("creator terminal cleanup receipt changed its exact restoration authority")
    }
    for digest in [
        &value.finalizer_cleanup_sha256,
        &value.creator_marker_restoration_sha256,
        &value.root_install_claims_retention_observation_sha256,
    ] {
        if !is_sha256(digest) {
            bail!("creator terminal cleanup receipt contains a non-digest binding")
        }
    }
    Ok(())
}

fn restore_creator_marker_after_observed_rollback(
    rollback_receipt: &CreatorRollbackReceiptV2,
    rollback_receipt_sha256: &str,
) -> Result<String> {
    if !is_sha256(rollback_receipt_sha256) {
        bail!("creator marker restoration lacks its rollback receipt digest")
    }
    let restoration_sha256 = document_sha256_v2(&(
        "substrate.r3-macos-terminal-creator-observed-rollback-restored.v2",
        MARKER_ROOT,
        MARKER_PATH,
        rollback_receipt_sha256,
        libc::ENOENT,
    ))?;
    let root = Path::new(MARKER_ROOT);
    if !path_present(root)? {
        return Ok(restoration_sha256);
    }
    let marker = MarkerRoot::open()?;
    marker.remove_after_observed_rollback(rollback_receipt)?;
    drop(marker);
    if !exact_directory_names(root)?.is_empty() {
        bail!("creator rollback root contains an alternate artifact during restoration")
    }
    remove_exact_empty_directory(root, 0, 0, libc::S_IFDIR | 0o700)?;
    require_exact_path_absent(root)?;
    Ok(restoration_sha256)
}

fn restore_creator_marker_after_terminal_ui() -> Result<String> {
    let root = Path::new(MARKER_ROOT);
    if !path_present(root)? {
        return document_sha256_v2(&(
            "substrate.r3-macos-terminal-creator-marker-already-absent.v2",
            MARKER_ROOT,
            MARKER_PATH,
            libc::ENOENT,
        ));
    }
    let marker = MarkerRoot::open()?;
    if marker.read_rollback()?.is_some() {
        bail!("terminal root-operation recovery found a separate creator rollback cursor")
    }
    let state = marker.read_state()?;
    if !matches!(
        state,
        MarkerState::FirstQueryPrepared | MarkerState::Complete
    ) {
        bail!("terminal root-operation recovery found an incomplete creator native arm")
    }
    remove_exact_owned_file(
        Path::new(MARKER_PATH),
        Some(state.marker()),
        None,
        0,
        Some(libc::S_IFREG | 0o600),
    )?;
    if !exact_directory_names(root)?.is_empty() {
        bail!("terminal creator marker root contains an alternate artifact")
    }
    remove_exact_empty_directory(root, 0, 0, libc::S_IFDIR | 0o700)?;
    document_sha256_v2(&(
        "substrate.r3-macos-terminal-creator-marker-restored.v2",
        MARKER_ROOT,
        MARKER_PATH,
        std::str::from_utf8(state.marker())?,
        libc::ENOENT,
    ))
}

#[allow(clippy::too_many_arguments)]
fn persist_terminal_admin_cleanup_authorization(
    terminal_outcome_kind: &str,
    terminal_receipt_sha256: String,
    surrogate_restoration_sha256: String,
    finalizer_cleanup_sha256: String,
    creator_marker_restoration_sha256: String,
    root_install_claims: &RootInstallClaimsBindingV2,
    root_install_claims_retention_observation_sha256: String,
) -> Result<TerminalAdminCleanupAuthorizationV2> {
    if !matches!(
        terminal_outcome_kind,
        "root_operation_securityagent_alert"
            | "publisher_securityagent_alert"
            | "general_closed_failure"
            | "creator_closed_failure"
    ) {
        bail!("terminal admin cleanup authorization has an unknown outcome")
    }
    let terminal_receipt_name = match terminal_outcome_kind {
        "root_operation_securityagent_alert" => ROOT_OPERATION_UI_TERMINAL_RECEIPT_NAME,
        "publisher_securityagent_alert" => PUBLISHER_UI_TERMINAL_RECEIPT_NAME,
        "general_closed_failure" => GENERAL_FAILURE_RECEIPT_NAME,
        "creator_closed_failure" => CREATOR_TERMINAL_RECEIPT_NAME,
        _ => unreachable!("closed terminal outcome checked above"),
    };
    let terminal_receipt_path = Path::new(RUNNER_ROOT).join(terminal_receipt_name);
    let terminal_receipt_bytes =
        stable_read_file(&terminal_receipt_path, 0, Some(libc::S_IFREG | 0o600))?;
    if sha256_hex_v2(&terminal_receipt_bytes) != terminal_receipt_sha256 {
        bail!("terminal cleanup receipt bytes changed before admin authorization")
    }
    let terminal_receipt_stat = lstat(&terminal_receipt_path)?
        .context("terminal cleanup receipt disappeared before admin authorization")?;
    let runner_root_stat = lstat(Path::new(RUNNER_ROOT))?
        .context("runner root disappeared before admin cleanup authorization")?;
    if runner_root_stat.st_uid != 0
        || runner_root_stat.st_gid != 0
        || (runner_root_stat.st_mode & (libc::S_IFMT | 0o7777)) != (libc::S_IFDIR | 0o700)
    {
        bail!("terminal admin cleanup runner root identity changed")
    }
    let runner_root_stable_identity_sha256 =
        document_sha256_v2(&TerminalRunnerRootStableIdentityV2 {
            path: RUNNER_ROOT,
            device: runner_root_stat.st_dev as u64,
            inode: runner_root_stat.st_ino,
            owner_uid: runner_root_stat.st_uid,
            owner_gid: runner_root_stat.st_gid,
            mode: u32::from(runner_root_stat.st_mode),
        })?;
    if let Some(existing) = read_runner_private_optional::<TerminalAdminCleanupAuthorizationV2>(
        TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME,
    )? {
        validate_terminal_admin_cleanup_authorization(&existing, root_install_claims)?;
        if existing.terminal_outcome_kind != terminal_outcome_kind
            || existing.terminal_receipt_sha256 != terminal_receipt_sha256
            || existing.surrogate_restoration_sha256 != surrogate_restoration_sha256
            || existing.finalizer_cleanup_sha256 != finalizer_cleanup_sha256
            || existing.creator_marker_restoration_sha256 != creator_marker_restoration_sha256
            || existing.root_install_claims_retention_observation_sha256
                != root_install_claims_retention_observation_sha256
            || existing.runner_root_stable_identity_sha256 != runner_root_stable_identity_sha256
        {
            bail!("terminal admin cleanup authorization changed during recovery")
        }
        validate_live_terminal_runner_child_inventory(&existing)?;
        return Ok(existing);
    }
    let runner_child_inventory = collect_terminal_runner_child_inventory()?;
    let runner_child_inventory_sha256 = document_sha256_v2(&runner_child_inventory)?;
    let runner_child_inventory_cardinality = u32::try_from(runner_child_inventory.len())
        .context("terminal runner child inventory exceeds u32")?;
    let runner_child_archive_total_bytes =
        terminal_runner_child_archive_total_bytes(&runner_child_inventory)?;
    let value = TerminalAdminCleanupAuthorizationV2 {
        schema_owner: "substrate.r3-macos-terminal-admin-cleanup-authorization".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        terminal_outcome_kind: terminal_outcome_kind.to_owned(),
        authorization_path: Path::new(RUNNER_ROOT)
            .join(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME)
            .to_string_lossy()
            .into_owned(),
        runner_root_path: RUNNER_ROOT.to_owned(),
        runner_root_stable_identity_sha256,
        terminal_receipt_path: terminal_receipt_path.to_string_lossy().into_owned(),
        terminal_receipt_sha256,
        terminal_receipt_physical_identity_sha256: runner_file_physical_identity_sha256(
            &terminal_receipt_stat,
        )?,
        runner_child_inventory,
        runner_child_inventory_sha256,
        runner_child_inventory_cardinality,
        runner_child_archive_total_bytes,
        runner_child_archive_max_bytes: TERMINAL_RUNNER_CHILD_ARCHIVE_MAX_BYTES_V2,
        surrogate_restoration_sha256,
        finalizer_cleanup_sha256,
        creator_marker_restoration_sha256,
        root_install_claims_binding_sha256: document_sha256_v2(root_install_claims)?,
        root_install_claims_retention_observation_sha256,
        claims_retained: true,
        admin_cleanup_authorized: true,
        terminal_no_normal_resume: true,
    };
    validate_terminal_admin_cleanup_authorization(&value, root_install_claims)?;
    if collect_terminal_runner_child_inventory()? != value.runner_child_inventory {
        bail!("terminal runner child inventory changed immediately before authorization")
    }
    write_runner_receipt(
        TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME,
        &canonical_bytes_v2(&value)?,
    )?;
    validate_live_terminal_runner_child_inventory(&value)?;
    Ok(value)
}

fn collect_terminal_runner_child_inventory() -> Result<Vec<TerminalRunnerChildIdentityV2>> {
    let mut result = Vec::new();
    for name in exact_directory_names(Path::new(RUNNER_ROOT))? {
        if name == TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME
            || name.starts_with('.')
            || name.contains(['/', '\0', '\n', '\r'])
        {
            bail!("terminal runner root contains an unauthorized child name")
        }
        let path = Path::new(RUNNER_ROOT).join(&name);
        let bytes = stable_read_file_bounded(
            &path,
            0,
            Some(libc::S_IFREG | 0o600),
            runner_private_archive_entry_max_bytes_v2(&name),
        )?;
        let stat = lstat(&path)?.context("terminal runner child disappeared during inventory")?;
        if stat.st_uid != 0
            || stat.st_gid != 0
            || stat.st_nlink != 1
            || (stat.st_mode & (libc::S_IFMT | 0o7777)) != (libc::S_IFREG | 0o600)
        {
            bail!("terminal runner child identity changed during inventory")
        }
        result.push(TerminalRunnerChildIdentityV2 {
            name,
            canonical_byte_length: u64::try_from(bytes.len())
                .context("terminal runner child byte length exceeds u64")?,
            canonical_sha256: sha256_hex_v2(&bytes),
            physical_identity_sha256: runner_file_physical_identity_sha256(&stat)?,
        });
    }
    result.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(result)
}

fn build_runner_private_archive_excluding(
    excluded_names: &BTreeSet<String>,
) -> Result<RunnerPrivateArchiveV2> {
    if RUNNER_ROOT != RUNNER_PRIVATE_ROOT_V2 {
        bail!("runner private archive root differs from its shared fixed literal")
    }
    let mut entries = Vec::new();
    for name in exact_directory_names(Path::new(RUNNER_ROOT))? {
        if excluded_names.contains(&name) {
            continue;
        }
        if name.starts_with('.') || name.contains(['/', '\0', '\n', '\r']) {
            bail!("runner private archive found an invalid child name")
        }
        let path = Path::new(RUNNER_ROOT).join(&name);
        let bytes = stable_read_file_bounded(
            &path,
            0,
            Some(libc::S_IFREG | 0o600),
            runner_private_archive_entry_max_bytes_v2(&name),
        )?;
        let stat = lstat(&path)?.context("runner private archive child disappeared")?;
        entries.push(RunnerPrivateArchiveEntryV2 {
            name,
            canonical_byte_length: u64::try_from(bytes.len())?,
            canonical_sha256: sha256_hex_v2(&bytes),
            physical_identity_sha256: runner_file_physical_identity_sha256(&stat)?,
            canonical_base64url: URL_SAFE_NO_PAD.encode(bytes),
        });
    }
    build_runner_private_archive_v2(entries)
}

fn terminal_runner_child_archive_total_bytes(
    inventory: &[TerminalRunnerChildIdentityV2],
) -> Result<u64> {
    let mut total = 0_u64;
    for entry in inventory {
        if entry.canonical_byte_length == 0
            || entry.canonical_byte_length
                > u64::try_from(runner_private_archive_entry_max_bytes_v2(&entry.name))?
        {
            bail!("terminal runner child is outside its exact per-document byte bound")
        }
        total = total
            .checked_add(entry.canonical_byte_length)
            .context("terminal runner child archive byte total overflowed")?;
    }
    if total > TERMINAL_RUNNER_CHILD_ARCHIVE_MAX_BYTES_V2 {
        bail!("terminal runner child archive exceeds its closed 64 MiB bound")
    }
    Ok(total)
}

fn validate_live_terminal_runner_child_inventory(
    value: &TerminalAdminCleanupAuthorizationV2,
) -> Result<()> {
    let mut names = exact_directory_names(Path::new(RUNNER_ROOT))?;
    let authorization_index = names
        .iter()
        .position(|name| name == TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME)
        .context("terminal runner root lacks its cleanup authorization leaf")?;
    names.remove(authorization_index);
    let expected_names = value
        .runner_child_inventory
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    if names != expected_names {
        bail!("terminal runner root differs from its exact authorized child inventory")
    }
    for entry in &value.runner_child_inventory {
        let path = Path::new(RUNNER_ROOT).join(&entry.name);
        let bytes = stable_read_file_bounded(
            &path,
            0,
            Some(libc::S_IFREG | 0o600),
            runner_private_archive_entry_max_bytes_v2(&entry.name),
        )?;
        let stat = lstat(&path)?.context("terminal runner inventory child disappeared")?;
        if u64::try_from(bytes.len())? != entry.canonical_byte_length
            || sha256_hex_v2(&bytes) != entry.canonical_sha256
            || runner_file_physical_identity_sha256(&stat)? != entry.physical_identity_sha256
        {
            bail!("terminal runner inventory child changed after authorization")
        }
    }
    Ok(())
}

fn persist_existing_terminal_admin_cleanup_authorization(
    inputs: &FrozenRunnerInputs,
    root_install_claims: &RootInstallClaimsBindingV2,
) -> Result<TerminalAdminCleanupAuthorizationV2> {
    let mut repetitions = Vec::with_capacity(RepetitionV2::ALL.len());
    for repetition in RepetitionV2::ALL {
        repetitions.push(validate_and_restore_failed_repetition(repetition)?);
    }
    remove_exact_installed_file(
        Path::new(ALTERNATE_COORDINATOR_PATH_V2),
        None,
        Some(&inputs.peer.alternate_path_identity.physical_identity_sha256),
    )?;
    remove_exact_installed_file(
        Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.prepared)?),
        None,
    )?;
    remove_exact_installed_file(
        Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.candidate)?),
        None,
    )?;
    remove_exact_installed_file(
        Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2),
        Some(&canonical_bytes_v2(&inputs.peer)?),
        None,
    )?;
    let creator_marker_restoration_sha256 = restore_creator_marker_after_terminal_ui()?;
    let publisher = read_runner_private_optional::<PublisherUiTerminalReceiptV2>(
        PUBLISHER_UI_TERMINAL_RECEIPT_NAME,
    )?;
    let general = read_runner_private_optional::<GeneralFailureRestorationReceiptV2>(
        GENERAL_FAILURE_RECEIPT_NAME,
    )?;
    let (terminal_outcome_kind, terminal_receipt_sha256) = match (publisher, general) {
        (Some(receipt), None) => {
            let cursor: PublisherUiStopCursorV2 =
                read_runner_private_optional(PUBLISHER_UI_STOP_CURSOR_NAME)?
                    .context("publisher terminal receipt lacks its UI-stop cursor")?;
            let harness: HarnessTerminationObservationV2 =
                read_runner_private_optional(PUBLISHER_UI_HARNESS_TERMINATION_NAME)?
                    .context("publisher terminal receipt lacks harness termination")?;
            let publisher: HarnessTerminationObservationV2 =
                read_runner_private_optional(PUBLISHER_UI_PUBLISHER_TERMINATION_NAME)?
                    .context("publisher terminal receipt lacks publisher termination")?;
            validate_publisher_ui_terminal_receipt(&receipt, &cursor, &harness, &publisher)?;
            (
                "publisher_securityagent_alert",
                document_sha256_v2(&receipt)?,
            )
        }
        (None, Some(receipt)) => {
            let cursor: GeneralFailureRestorationCursorV2 =
                read_runner_private_optional(GENERAL_FAILURE_CURSOR_NAME)?
                    .context("general terminal receipt lacks its restoration cursor")?;
            validate_general_failure_receipt(&receipt, &cursor)?;
            ("general_closed_failure", document_sha256_v2(&receipt)?)
        }
        _ => bail!("terminal cleanup lacks exactly one restored terminal failure receipt"),
    };
    let finalizer_cleanup_bytes =
        read_runner_private_bytes_optional("terminal-finalizer-failure-cleanup.v2.json")?
            .context("terminal admin cleanup authorization lacks finalizer cleanup evidence")?;
    let finalizer_cleanup: TerminalFinalizerFailureCleanupReceiptV2 =
        parse_canonical_v2(&finalizer_cleanup_bytes)?;
    if finalizer_cleanup.schema_owner
        != "substrate.r3-macos-disposable-terminal-service-failure-cleanup"
        || finalizer_cleanup.schema_version != EXPERIMENT_VERSION_V2
        || finalizer_cleanup.experiment_id != EXPERIMENT_ID_V2
        || !finalizer_cleanup.endpoint_absent
        || !finalizer_cleanup.journal_root_absent
        || finalizer_cleanup.journal_root_preserved_for_unacknowledged_evidence
    {
        bail!("terminal admin cleanup authorization found incomplete service cleanup")
    }
    let claims_observation = observe_retained_root_install_claims(root_install_claims)?;
    persist_terminal_admin_cleanup_authorization(
        terminal_outcome_kind,
        terminal_receipt_sha256,
        document_sha256_v2(&repetitions)?,
        sha256_hex_v2(&finalizer_cleanup_bytes),
        creator_marker_restoration_sha256,
        root_install_claims,
        claims_observation,
    )
}

fn validate_terminal_admin_cleanup_authorization(
    value: &TerminalAdminCleanupAuthorizationV2,
    root_install_claims: &RootInstallClaimsBindingV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-terminal-admin-cleanup-authorization"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || !matches!(
            value.terminal_outcome_kind.as_str(),
            "root_operation_securityagent_alert"
                | "publisher_securityagent_alert"
                | "general_closed_failure"
                | "creator_closed_failure"
        )
        || value.authorization_path
            != Path::new(RUNNER_ROOT)
                .join(TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME)
                .to_string_lossy()
        || value.runner_root_path != RUNNER_ROOT
        || value.terminal_receipt_path
            != Path::new(RUNNER_ROOT)
                .join(match value.terminal_outcome_kind.as_str() {
                    "root_operation_securityagent_alert" => ROOT_OPERATION_UI_TERMINAL_RECEIPT_NAME,
                    "publisher_securityagent_alert" => PUBLISHER_UI_TERMINAL_RECEIPT_NAME,
                    "general_closed_failure" => GENERAL_FAILURE_RECEIPT_NAME,
                    "creator_closed_failure" => CREATOR_TERMINAL_RECEIPT_NAME,
                    _ => unreachable!("outcome was checked above"),
                })
                .to_string_lossy()
        || value.root_install_claims_binding_sha256 != document_sha256_v2(root_install_claims)?
        || value.runner_child_inventory.is_empty()
        || value.runner_child_inventory_cardinality
            != u32::try_from(value.runner_child_inventory.len())?
        || value.runner_child_inventory_sha256 != document_sha256_v2(&value.runner_child_inventory)?
        || value.runner_child_archive_max_bytes != TERMINAL_RUNNER_CHILD_ARCHIVE_MAX_BYTES_V2
        || value.runner_child_archive_total_bytes
            != terminal_runner_child_archive_total_bytes(&value.runner_child_inventory)?
        || !value.claims_retained
        || !value.admin_cleanup_authorized
        || !value.terminal_no_normal_resume
    {
        bail!("terminal admin cleanup authorization changed its closed authority")
    }
    for digest in [
        &value.terminal_receipt_sha256,
        &value.runner_root_stable_identity_sha256,
        &value.terminal_receipt_physical_identity_sha256,
        &value.runner_child_inventory_sha256,
        &value.surrogate_restoration_sha256,
        &value.finalizer_cleanup_sha256,
        &value.creator_marker_restoration_sha256,
        &value.root_install_claims_binding_sha256,
        &value.root_install_claims_retention_observation_sha256,
    ] {
        if !is_sha256(digest) {
            bail!("terminal admin cleanup authorization contains a non-digest binding")
        }
    }
    let mut prior: Option<&str> = None;
    for entry in &value.runner_child_inventory {
        if entry.name.is_empty()
            || entry.name == TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME
            || entry.name.starts_with('.')
            || entry.name.contains(['/', '\0', '\n', '\r'])
            || prior.is_some_and(|prior| prior >= entry.name.as_str())
            || entry.canonical_byte_length == 0
            || entry.canonical_byte_length
                > u64::try_from(runner_private_archive_entry_max_bytes_v2(&entry.name))?
            || !is_sha256(&entry.canonical_sha256)
            || !is_sha256(&entry.physical_identity_sha256)
        {
            bail!("terminal runner child inventory is not exact sorted immutable file state")
        }
        prior = Some(&entry.name);
    }
    let terminal_leaf = Path::new(&value.terminal_receipt_path)
        .file_name()
        .and_then(|name| name.to_str())
        .context("terminal receipt path lacks a UTF-8 leaf")?;
    let terminal_entry = value
        .runner_child_inventory
        .iter()
        .find(|entry| entry.name == terminal_leaf)
        .context("terminal runner inventory omits its exact terminal receipt")?;
    if terminal_entry.canonical_sha256 != value.terminal_receipt_sha256
        || terminal_entry.physical_identity_sha256
            != value.terminal_receipt_physical_identity_sha256
    {
        bail!("terminal runner inventory differs from its terminal receipt anchor")
    }
    Ok(())
}

fn terminal_journal_root_disposition(
    global_pre_effect: Option<&GlobalPreEffectPacketV2>,
    activation_membrane: &ActivationMembraneGuardV2,
) -> Result<TerminalJournalRootDispositionV2> {
    if global_pre_effect.is_some_and(|packet| {
        activation_membrane.root_identity_sha256 != packet.activation_membrane_root_identity_sha256
    }) {
        bail!("terminal cleanup activation membrane differs from the global pre-effect packet")
    }
    let root = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2);
    let held = fstat(activation_membrane.descriptor.as_raw_fd())?;
    let Some(live) = lstat(root)? else {
        return Ok(TerminalJournalRootDispositionV2 {
            absent: true,
            preserved_for_unacknowledged_evidence: false,
            observation_sha256: document_sha256_v2(&(
                "terminal-journal-root-already-absent-v2",
                MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
                &activation_membrane.root_identity_sha256,
                held.st_dev as u64,
                held.st_ino,
                libc::ENOENT,
            ))?,
        });
    };
    if held.st_dev != live.st_dev
        || held.st_ino != live.st_ino
        || held.st_uid != live.st_uid
        || held.st_gid != live.st_gid
        || held.st_mode != live.st_mode
        || live.st_uid != 0
        || live.st_gid != 0
        || (live.st_mode & (libc::S_IFMT | 0o7777)) != (libc::S_IFDIR | 0o700)
    {
        bail!("terminal cleanup journal root differs from its held activation identity")
    }
    let names = exact_directory_names(root)?;
    if names != ["journal-root.lock"] {
        return Ok(TerminalJournalRootDispositionV2 {
            absent: false,
            preserved_for_unacknowledged_evidence: true,
            observation_sha256: document_sha256_v2(&(
                "terminal-journal-root-preserved-v2",
                MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
                &activation_membrane.root_identity_sha256,
                names,
            ))?,
        });
    }
    let lock_path = root.join("journal-root.lock");
    if path_present(&lock_path)? {
        let encoded = CString::new(lock_path.as_os_str().as_bytes())?;
        // SAFETY: the only accepted child has one compiled path and a terminal no-follow open.
        let raw = unsafe {
            libc::open(
                encoded.as_ptr(),
                libc::O_RDWR | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error())
                .context("open terminal cleanup activation lock");
        }
        // SAFETY: successful open transferred ownership.
        let lock = unsafe { OwnedFd::from_raw_fd(raw) };
        let stat = fstat(lock.as_raw_fd())?;
        let identity =
            root_install_physical_identity(MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2, &stat)?;
        let identity_sha256 = document_sha256_v2(&identity)?;
        let packet_mismatch = global_pre_effect.is_some_and(|packet| {
            identity != packet.journal_root_lock_identity
                || identity_sha256 != packet.journal_root_lock_identity_sha256
        });
        if identity != activation_membrane.journal_root_lock_identity
            || identity_sha256 != activation_membrane.journal_root_lock_identity_sha256
            || packet_mismatch
        {
            bail!("terminal cleanup activation lock changed identity")
        }
        // A busy lock means another exact finalizer admission still owns the activation membrane;
        // never block, retry, or unlink underneath that owner.
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("terminal cleanup activation lock is still owned");
        }
        remove_exact_owned_file(&lock_path, None, None, 0, Some(libc::S_IFREG | 0o600))?;
        drop(lock);
        require_exact_path_absent(&lock_path)?;
    }
    if !exact_directory_names(root)?.is_empty() {
        bail!("terminal cleanup activation root gained unexpected evidence before removal")
    }
    remove_exact_empty_directory(root, 0, 0, libc::S_IFDIR | 0o700)?;
    require_exact_path_absent(root)?;
    Ok(TerminalJournalRootDispositionV2 {
        absent: true,
        preserved_for_unacknowledged_evidence: false,
        observation_sha256: document_sha256_v2(&(
            "terminal-journal-root-empty-removed-v2",
            MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
            &activation_membrane.root_identity_sha256,
            names,
            libc::ENOENT,
        ))?,
    })
}

fn validate_bootout_result(
    result: &FinalizerBootoutResultV2,
    prepared: &FinalizerBootoutPreparedV2,
) -> Result<()> {
    if result.schema_owner != NATIVE_EVIDENCE_EXPORT_OWNER_V2
        || result.schema_version != EXPERIMENT_VERSION_V2
        || result.experiment_id != EXPERIMENT_ID_V2
        || result.prepared_sha256 != document_sha256_v2(prepared)?
        || result.exit_status != 0
        || !result.stdout.validate()?.is_empty()
        || !result.stderr.validate()?.is_empty()
    {
        bail!("launchctl bootout did not return the frozen successful classification")
    }
    Ok(())
}

fn exact_launchctl_service_not_found_stderr(label: &str) -> String {
    format!("Bad request.\nCould not find service \"{label}\" in domain for system\n")
}

fn observe_exact_launchd_service_absence(
) -> Result<(i32, BoundedRawStreamEvidenceV2, BoundedRawStreamEvidenceV2)> {
    let target = format!("system/{}", MAC_R3_FINALIZER_LAUNCHD_LABEL_V2);
    let output = Command::new("/bin/launchctl")
        .arg("print")
        .arg(&target)
        .current_dir("/")
        .env_clear()
        .stdin(Stdio::null())
        .output()?;
    let status = output
        .status
        .code()
        .context("launchctl absent print was signaled")?;
    if status != 113
        || !output.stdout.is_empty()
        || output.stderr
            != exact_launchctl_service_not_found_stderr(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)
                .as_bytes()
    {
        bail!("launchctl print did not return the exact service-not-found classification")
    }
    Ok((
        status,
        bounded_raw_stream_evidence(&output.stdout)?,
        bounded_raw_stream_evidence(&output.stderr)?,
    ))
}

fn observe_empty_coordinator_inbox() -> Result<String> {
    require_directory(
        Path::new(MAC_R3_COORDINATOR_INBOX_ROOT_V2),
        0,
        20,
        libc::S_IFDIR | 0o750,
    )?;
    require_exact_path_absent(Path::new(MAC_R3_FINALIZER_REQUEST_PATH_V2))?;
    require_exact_path_absent(Path::new(MAC_R3_TERMINAL_BINDING_PATH_V2))?;
    let mut names = std::fs::read_dir(MAC_R3_COORDINATOR_INBOX_ROOT_V2)?
        .map(|entry| {
            entry?
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("coordinator inbox contains a non-UTF8 leaf"))
        })
        .collect::<Result<Vec<_>>>()?;
    names.sort();
    if !names.is_empty() {
        bail!("coordinator inbox contains an unexpected residual leaf")
    }
    document_sha256_v2(&(
        "coordinator-inbox-empty-v2",
        MAC_R3_COORDINATOR_INBOX_ROOT_V2,
        MAC_R3_FINALIZER_REQUEST_PATH_V2,
        MAC_R3_TERMINAL_BINDING_PATH_V2,
        names,
    ))
}

type JournalCleanupDigestsV2 = (Vec<String>, Vec<String>, Vec<String>, String, String);

fn load_or_create_native_evidence_cleanup_plan(
    global_pre_effect: &GlobalPreEffectPacketV2,
    export: &NativeEvidenceExportV2,
    acknowledgement: &NativeEvidenceExportAcknowledgementV2,
) -> Result<NativeEvidenceCleanupPlanV2> {
    if let Some(existing) =
        read_runner_private_optional::<NativeEvidenceCleanupPlanV2>(NATIVE_CLEANUP_PLAN_NAME)?
    {
        validate_native_evidence_cleanup_plan(
            &existing,
            global_pre_effect,
            export,
            acknowledgement,
        )?;
        return Ok(existing);
    }
    let export_path = global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExport)?;
    let acknowledgement_path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExportAcknowledgement)?;
    let export_bytes = stable_read_file_bounded(
        &export_path,
        0,
        Some(libc::S_IFREG | 0o444),
        MAX_NATIVE_EVIDENCE_DOCUMENT,
    )?;
    let acknowledgement_bytes = stable_read_file(
        &acknowledgement_path,
        DISPOSABLE_HARNESS_UID_V2,
        Some(EXTERNAL_INPUT_MODE),
    )?;
    if export_bytes != canonical_bytes_v2(export)?
        || acknowledgement_bytes != canonical_bytes_v2(acknowledgement)?
    {
        bail!("native cleanup plan inputs changed before cleanup authorization")
    }
    let export_stat = lstat(&export_path)?.context("native evidence export disappeared")?;
    let acknowledgement_stat =
        lstat(&acknowledgement_path)?.context("native evidence acknowledgement disappeared")?;
    let mut objects = Vec::new();
    for (repetition, repetition_export) in RepetitionV2::ALL.into_iter().zip(&export.repetitions) {
        let scope = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id());
        let expected = completed_journal_scope_expected(repetition, repetition_export)?;
        for (name, (bytes, mode)) in expected {
            objects.push(native_cleanup_file_object(
                u32::try_from(objects.len() + 1)?,
                &scope.join(name),
                bytes.as_deref(),
                mode,
            )?);
        }
        objects.push(native_cleanup_directory_object(
            u32::try_from(objects.len() + 1)?,
            &scope,
        )?);
        require_exact_path_absent(
            &Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2)
                .join("capability")
                .join(repetition.scope_id()),
        )?;
        require_exact_path_absent(&Path::new(MAC_R3_RETIREMENT_LATCH_ROOT_V2).join(format!(
            "{}.retirement-terminal.v2.latch",
            repetition.scope_id()
        )))?;
    }
    for directory in [
        Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join("capability"),
        Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join("latches"),
    ] {
        if path_present(&directory)? {
            if !exact_directory_names(&directory)?.is_empty() {
                bail!("native cleanup auxiliary directory is not exactly empty")
            }
            objects.push(native_cleanup_directory_object(
                u32::try_from(objects.len() + 1)?,
                &directory,
            )?);
        }
    }
    objects.push(native_cleanup_file_object(
        u32::try_from(objects.len() + 1)?,
        Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2),
        None,
        libc::S_IFREG | 0o600,
    )?);
    objects.push(native_cleanup_directory_object(
        u32::try_from(objects.len() + 1)?,
        Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2),
    )?);
    let object_set_sha256 = document_sha256_v2(&objects)?;
    let value = NativeEvidenceCleanupPlanV2 {
        schema_owner: "substrate.r3-macos-native-evidence-cleanup-plan".to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        native_evidence_export_sha256: document_sha256_v2(export)?,
        native_evidence_export_physical_identity_sha256: runner_file_physical_identity_sha256(
            &export_stat,
        )?,
        acknowledgement_sha256: document_sha256_v2(acknowledgement)?,
        acknowledgement_physical_identity_sha256: runner_file_physical_identity_sha256(
            &acknowledgement_stat,
        )?,
        global_pre_effect_packet_sha256: document_sha256_v2(global_pre_effect)?,
        journal_root_activation_identity_sha256: global_pre_effect
            .activation_membrane_root_identity_sha256
            .clone(),
        journal_root_lock_identity_sha256: global_pre_effect
            .journal_root_lock_identity_sha256
            .clone(),
        objects,
        object_set_sha256,
        deletion_may_begin: true,
    };
    validate_native_evidence_cleanup_plan(&value, global_pre_effect, export, acknowledgement)?;
    write_runner_receipt(NATIVE_CLEANUP_PLAN_NAME, &canonical_bytes_v2(&value)?)?;
    Ok(value)
}

fn validate_native_evidence_cleanup_plan(
    value: &NativeEvidenceCleanupPlanV2,
    global_pre_effect: &GlobalPreEffectPacketV2,
    export: &NativeEvidenceExportV2,
    acknowledgement: &NativeEvidenceExportAcknowledgementV2,
) -> Result<()> {
    if value.schema_owner != "substrate.r3-macos-native-evidence-cleanup-plan"
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.native_evidence_export_sha256 != document_sha256_v2(export)?
        || value.acknowledgement_sha256 != document_sha256_v2(acknowledgement)?
        || value.global_pre_effect_packet_sha256 != document_sha256_v2(global_pre_effect)?
        || value.journal_root_activation_identity_sha256
            != global_pre_effect.activation_membrane_root_identity_sha256
        || value.journal_root_lock_identity_sha256
            != global_pre_effect.journal_root_lock_identity_sha256
        || value.objects.is_empty()
        || value.object_set_sha256 != document_sha256_v2(&value.objects)?
        || !value.deletion_may_begin
    {
        bail!("native evidence cleanup plan changed its acknowledged authority")
    }
    let export_path = global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExport)?;
    let acknowledgement_path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExportAcknowledgement)?;
    let export_bytes = stable_read_file_bounded(
        &export_path,
        0,
        Some(libc::S_IFREG | 0o444),
        MAX_NATIVE_EVIDENCE_DOCUMENT,
    )?;
    let acknowledgement_bytes = stable_read_file(
        &acknowledgement_path,
        DISPOSABLE_HARNESS_UID_V2,
        Some(EXTERNAL_INPUT_MODE),
    )?;
    let export_stat = lstat(&export_path)?.context("native evidence export disappeared")?;
    let acknowledgement_stat =
        lstat(&acknowledgement_path)?.context("native evidence acknowledgement disappeared")?;
    if export_bytes != canonical_bytes_v2(export)?
        || acknowledgement_bytes != canonical_bytes_v2(acknowledgement)?
        || runner_file_physical_identity_sha256(&export_stat)?
            != value.native_evidence_export_physical_identity_sha256
        || runner_file_physical_identity_sha256(&acknowledgement_stat)?
            != value.acknowledgement_physical_identity_sha256
    {
        bail!("native cleanup plan external evidence identity changed")
    }
    let mut prior_path = None;
    for (index, object) in value.objects.iter().enumerate() {
        if object.ordinal != u32::try_from(index + 1)?
            || !object
                .path
                .starts_with(&format!("{MAC_R3_FINALIZER_JOURNAL_ROOT_V2}/"))
                && object.path != MAC_R3_FINALIZER_JOURNAL_ROOT_V2
            || !is_sha256(&object.physical_identity_sha256)
            || object
                .canonical_sha256
                .as_ref()
                .is_some_and(|digest| !is_sha256(digest))
            || prior_path.as_ref().is_some_and(|path| path == &object.path)
        {
            bail!("native cleanup plan contains an invalid ordered object")
        }
        match object.kind {
            NativeCleanupObjectKindV2::File if object.canonical_sha256.is_none() => {
                bail!("native cleanup file lacks its exact canonical byte digest")
            }
            NativeCleanupObjectKindV2::Directory if object.canonical_sha256.is_some() => {
                bail!("native cleanup directory unexpectedly carries bytes")
            }
            _ => {}
        }
        prior_path = Some(object.path.clone());
    }
    if value.objects.last().map(|entry| entry.path.as_str())
        != Some(MAC_R3_FINALIZER_JOURNAL_ROOT_V2)
    {
        bail!("native cleanup plan does not remove its exact journal root last")
    }
    Ok(())
}

fn native_cleanup_file_object(
    ordinal: u32,
    path: &Path,
    expected_bytes: Option<&[u8]>,
    expected_mode: libc::mode_t,
) -> Result<NativeCleanupObjectV2> {
    let bytes = stable_read_file(path, 0, Some(expected_mode))?;
    if expected_bytes.is_some_and(|expected| expected != bytes) {
        bail!("native cleanup plan file differs from exported canonical bytes")
    }
    let stat = lstat(path)?.context("native cleanup plan file disappeared")?;
    Ok(NativeCleanupObjectV2 {
        ordinal,
        path: path.to_string_lossy().into_owned(),
        kind: NativeCleanupObjectKindV2::File,
        owner_uid: 0,
        owner_gid: 0,
        mode: u32::from(expected_mode),
        canonical_sha256: Some(sha256_hex_v2(&bytes)),
        physical_identity_sha256: runner_file_physical_identity_sha256(&stat)?,
    })
}

fn native_cleanup_directory_object(ordinal: u32, path: &Path) -> Result<NativeCleanupObjectV2> {
    require_directory(path, 0, 0, libc::S_IFDIR | 0o700)?;
    let stat = lstat(path)?.context("native cleanup plan directory disappeared")?;
    Ok(NativeCleanupObjectV2 {
        ordinal,
        path: path.to_string_lossy().into_owned(),
        kind: NativeCleanupObjectKindV2::Directory,
        owner_uid: 0,
        owner_gid: 0,
        mode: u32::from(libc::S_IFDIR | 0o700),
        canonical_sha256: None,
        physical_identity_sha256: cleanup_directory_stable_identity_sha256(path, &stat)?,
    })
}

fn cleanup_directory_stable_identity_sha256(path: &Path, stat: &libc::stat) -> Result<String> {
    document_sha256_v2(&TerminalRunnerRootStableIdentityV2 {
        path: path
            .to_str()
            .context("native cleanup directory path is not UTF-8")?,
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        owner_uid: stat.st_uid,
        owner_gid: stat.st_gid,
        mode: u32::from(stat.st_mode),
    })
}

fn remove_exported_journal_root(
    global_pre_effect: &GlobalPreEffectPacketV2,
    export: &NativeEvidenceExportV2,
    activation_membrane: &ActivationMembraneGuardV2,
    cleanup_plan: &NativeEvidenceCleanupPlanV2,
) -> Result<JournalCleanupDigestsV2> {
    validate_native_evidence_cleanup_plan(
        cleanup_plan,
        global_pre_effect,
        export,
        &read_native_evidence_acknowledgement_for_plan(cleanup_plan)?,
    )?;
    if activation_membrane.root_identity_sha256
        != global_pre_effect.activation_membrane_root_identity_sha256
    {
        bail!("journal cleanup activation membrane differs from its global packet")
    }
    let root = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2);
    let lock_path = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2);
    let held_lock = if path_present(lock_path)? {
        let lock_encoded = CString::new(lock_path.as_os_str().as_bytes())?;
        // SAFETY: exact no-follow lock path from the frozen cleanup plan.
        let raw_lock = unsafe {
            libc::open(
                lock_encoded.as_ptr(),
                libc::O_RDWR | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw_lock < 0 {
            return Err(std::io::Error::last_os_error()).context("open exact journal-root lock");
        }
        // SAFETY: successful open transferred ownership.
        let lock = unsafe { OwnedFd::from_raw_fd(raw_lock) };
        let lock_stat = fstat(lock.as_raw_fd())?;
        let lock_identity =
            root_install_physical_identity(MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2, &lock_stat)?;
        if lock_identity != global_pre_effect.journal_root_lock_identity
            || document_sha256_v2(&lock_identity)?
                != global_pre_effect.journal_root_lock_identity_sha256
        {
            bail!("journal-root lock identity changed before cleanup")
        }
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error()).context("lock journal root for cleanup");
        }
        Some(lock)
    } else {
        None
    };
    execute_native_evidence_cleanup_plan(cleanup_plan)?;
    drop(held_lock);
    require_exact_path_absent(root)?;

    let mut journal_absence = Vec::with_capacity(RepetitionV2::ALL.len());
    let mut capability_absence = Vec::with_capacity(RepetitionV2::ALL.len());
    let mut latch_absence = Vec::with_capacity(RepetitionV2::ALL.len());
    for repetition in RepetitionV2::ALL {
        let journal_path = root.join(repetition.scope_id());
        require_exact_path_absent(&journal_path)?;
        journal_absence.push(document_sha256_v2(&(
            "journal-scope-absent-v2",
            journal_path.to_string_lossy(),
            libc::ENOENT,
        ))?);
        let capability_path = root.join("capability").join(repetition.scope_id());
        require_exact_path_absent(&capability_path)?;
        capability_absence.push(document_sha256_v2(&(
            "capability-scope-absent-v2",
            capability_path.to_string_lossy(),
            libc::ENOENT,
        ))?);
        let latch_path = Path::new(MAC_R3_RETIREMENT_LATCH_ROOT_V2).join(format!(
            "{}.retirement-terminal.v2.latch",
            repetition.scope_id()
        ));
        require_exact_path_absent(&latch_path)?;
        latch_absence.push(document_sha256_v2(&(
            "retirement-latch-absent-v2",
            latch_path.to_string_lossy(),
            libc::ENOENT,
        ))?);
    }
    let lock_absence = document_sha256_v2(&(
        "journal-root-lock-absent-v2",
        MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2,
        &global_pre_effect.journal_root_lock_identity_sha256,
        libc::ENOENT,
    ))?;
    let root_absence = document_sha256_v2(&(
        "journal-root-absent-v2",
        MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
        &activation_membrane.root_identity_sha256,
        libc::ENOENT,
    ))?;
    Ok((
        journal_absence,
        capability_absence,
        latch_absence,
        lock_absence,
        root_absence,
    ))
}

fn read_native_evidence_acknowledgement_for_plan(
    plan: &NativeEvidenceCleanupPlanV2,
) -> Result<NativeEvidenceExportAcknowledgementV2> {
    let path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceExportAcknowledgement)?;
    let bytes = stable_read_file(&path, DISPOSABLE_HARNESS_UID_V2, Some(EXTERNAL_INPUT_MODE))?;
    let value: NativeEvidenceExportAcknowledgementV2 = parse_canonical_v2(&bytes)?;
    if canonical_bytes_v2(&value)? != bytes
        || document_sha256_v2(&value)? != plan.acknowledgement_sha256
    {
        bail!("native cleanup acknowledgement changed during prefix recovery")
    }
    Ok(value)
}

fn execute_native_evidence_cleanup_plan(plan: &NativeEvidenceCleanupPlanV2) -> Result<()> {
    let prefix = plan
        .objects
        .iter()
        .map(|object| {
            let step_name = format!(
                "{NATIVE_CLEANUP_STEP_PREFIX}-{:05}.observed.v2.json",
                object.ordinal
            );
            Ok((
                read_runner_private_optional::<NativeEvidenceCleanupStepV2>(&step_name)?.is_some(),
                path_present(Path::new(&object.path))?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    validate_native_cleanup_recovery_prefix(&prefix)?;
    for object in &plan.objects {
        let step_name = format!(
            "{NATIVE_CLEANUP_STEP_PREFIX}-{:05}.observed.v2.json",
            object.ordinal
        );
        let expected_step = NativeEvidenceCleanupStepV2 {
            schema_owner: "substrate.r3-macos-native-evidence-cleanup-step".to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            plan_sha256: document_sha256_v2(plan)?,
            ordinal: object.ordinal,
            object_sha256: document_sha256_v2(object)?,
            exact_absence_observed: true,
            parent_fsynced: true,
        };
        if let Some(existing) =
            read_runner_private_optional::<NativeEvidenceCleanupStepV2>(&step_name)?
        {
            if existing != expected_step {
                bail!("native cleanup step receipt changed during recovery")
            }
            require_exact_path_absent(Path::new(&object.path))?;
            continue;
        }
        let path = Path::new(&object.path);
        match object.kind {
            NativeCleanupObjectKindV2::File => {
                if path_present(path)? {
                    let bytes = stable_read_file(
                        path,
                        object.owner_uid,
                        Some(libc::mode_t::try_from(object.mode)?),
                    )?;
                    let stat = lstat(path)?.context("planned cleanup file disappeared")?;
                    if object.owner_gid != stat.st_gid
                        || object.canonical_sha256.as_deref() != Some(&sha256_hex_v2(&bytes))
                        || runner_file_physical_identity_sha256(&stat)?
                            != object.physical_identity_sha256
                    {
                        bail!("planned cleanup file changed before exact removal")
                    }
                    remove_exact_owned_file(
                        path,
                        Some(&bytes),
                        Some(&object.physical_identity_sha256),
                        object.owner_uid,
                        Some(libc::mode_t::try_from(object.mode)?),
                    )?;
                }
            }
            NativeCleanupObjectKindV2::Directory => {
                if path_present(path)? {
                    let stat = lstat(path)?.context("planned cleanup directory disappeared")?;
                    if object.owner_uid != stat.st_uid
                        || object.owner_gid != stat.st_gid
                        || u32::from(stat.st_mode) != object.mode
                        || cleanup_directory_stable_identity_sha256(path, &stat)?
                            != object.physical_identity_sha256
                        || !exact_directory_names(path)?.is_empty()
                    {
                        bail!("planned cleanup directory changed before exact removal")
                    }
                    remove_exact_empty_directory(
                        path,
                        object.owner_uid,
                        object.owner_gid,
                        libc::mode_t::try_from(object.mode)?,
                    )?;
                }
            }
        }
        require_exact_path_absent(path)?;
        fsync_exact_parent(path)?;
        write_runner_receipt(&step_name, &canonical_bytes_v2(&expected_step)?)?;
    }
    Ok(())
}

fn validate_native_cleanup_recovery_prefix(prefix: &[(bool, bool)]) -> Result<()> {
    let mut unreceipted_absence_seen = false;
    let mut live_suffix_seen = false;
    for &(step_observed, object_present) in prefix {
        match (step_observed, object_present) {
            (true, false) if !unreceipted_absence_seen && !live_suffix_seen => {}
            (false, false) if !unreceipted_absence_seen && !live_suffix_seen => {
                unreceipted_absence_seen = true;
            }
            (false, true) => {
                live_suffix_seen = true;
            }
            _ => bail!("native cleanup state is not one exact durable removal prefix"),
        }
    }
    Ok(())
}

fn fsync_exact_parent(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .context("cleanup object lacks its exact parent")?;
    let encoded = CString::new(parent.as_os_str().as_bytes())?;
    // SAFETY: exact compiled/planned parent and no-follow directory open.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error()).context("open cleanup object parent");
    }
    // SAFETY: successful open transferred ownership.
    let parent = unsafe { OwnedFd::from_raw_fd(raw) };
    if unsafe { libc::fsync(parent.as_raw_fd()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("fsync cleanup object parent");
    }
    Ok(())
}

fn completed_journal_scope_expected(
    repetition: RepetitionV2,
    export: &RepetitionNativeEvidenceExportV2,
) -> Result<CompletedJournalScopeExpectedV2> {
    let scope = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(repetition.scope_id());
    require_directory(&scope, 0, 0, libc::S_IFDIR | 0o700)?;
    let mut expected = CompletedJournalScopeExpectedV2::new();
    expected.insert("journal.lock".to_owned(), (None, libc::S_IFREG | 0o600));
    let head = export
        .generations
        .last()
        .context("completed journal export has no generation")?;
    expected.insert(
        "HEAD".to_owned(),
        (
            Some(format!("{:020} {}\n", head.generation, head.canonical_sha256).into_bytes()),
            libc::S_IFREG | 0o600,
        ),
    );
    for generation in &export.generations {
        expected.insert(
            format!("generation-{:020}.json", generation.generation),
            (
                Some(canonical_bytes_v2(&generation.record)?),
                libc::S_IFREG | 0o400,
            ),
        );
    }
    for observation in &export.effect_observations {
        expected.insert(
            format!("effect-observation-{:05}.json", observation.effect_ordinal),
            (
                Some(canonical_bytes_v2(&observation.artifact)?),
                libc::S_IFREG | 0o400,
            ),
        );
    }
    let required_ancillary = [
        "authority.request",
        "peer.attestation",
        "effects.response",
        "parity.proof",
        "terminal.acknowledgement",
        "terminal.response",
    ];
    for name in required_ancillary {
        expected.insert(name.to_owned(), (None, libc::S_IFREG | 0o400));
    }
    let names = exact_directory_names(&scope)?;
    let actual = names.iter().cloned().collect::<BTreeSet<_>>();
    for name in &names {
        if expected.contains_key(name) {
            continue;
        }
        let digest = name
            .strip_prefix("peer.rejoin.")
            .and_then(|value| value.strip_suffix(".attestation"))
            .filter(|value| is_sha256(value))
            .context("journal scope contains an unexpected artifact name")?;
        let bytes = stable_read_file(&scope.join(name), 0, Some(libc::S_IFREG | 0o400))?;
        if sha256_hex_v2(&bytes) != digest {
            bail!("rejoin attestation filename does not bind its exact bytes")
        }
        expected.insert(name.clone(), (Some(bytes), libc::S_IFREG | 0o400));
    }
    if !expected.keys().all(|name| actual.contains(name)) || expected.len() != actual.len() {
        bail!("completed journal scope is missing or adds one exact artifact")
    }
    Ok(expected)
}

fn exact_directory_names(path: &Path) -> Result<Vec<String>> {
    let before = lstat(path)?.context("exact directory disappeared before enumeration")?;
    let mut names = std::fs::read_dir(path)?
        .map(|entry| {
            entry?
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("exact directory contains a non-UTF8 leaf"))
        })
        .collect::<Result<Vec<_>>>()?;
    names.sort();
    let after = lstat(path)?.context("exact directory disappeared after enumeration")?;
    if !same_stat(&before, &after) {
        bail!("exact directory identity changed during enumeration")
    }
    Ok(names)
}

fn remove_exact_empty_directory(
    path: &Path,
    expected_uid: u32,
    expected_gid: u32,
    expected_mode: libc::mode_t,
) -> Result<()> {
    if !exact_directory_names(path)?.is_empty() {
        bail!("exact directory is not empty before removal")
    }
    let parent = path.parent().context("exact directory lacks a parent")?;
    let leaf = path.file_name().context("exact directory lacks a leaf")?;
    let parent_encoded = CString::new(parent.as_os_str().as_bytes())?;
    let leaf_encoded = CString::new(leaf.as_bytes())?;
    let raw_parent = unsafe {
        libc::open(
            parent_encoded.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw_parent < 0 {
        return Err(std::io::Error::last_os_error()).context("open exact directory parent");
    }
    let parent_fd = unsafe { OwnedFd::from_raw_fd(raw_parent) };
    let before = fstatat_nofollow(parent_fd.as_raw_fd(), &leaf_encoded)?
        .context("exact directory disappeared before held-parent removal")?;
    if before.st_uid != expected_uid
        || before.st_gid != expected_gid
        || (before.st_mode & (libc::S_IFMT | 0o7777)) != expected_mode
    {
        bail!("exact directory identity changed before held-parent removal")
    }
    if !exact_directory_names(path)?.is_empty() {
        bail!("exact directory gained a leaf before held-parent removal")
    }
    let after = fstatat_nofollow(parent_fd.as_raw_fd(), &leaf_encoded)?
        .context("exact directory disappeared before unlinkat")?;
    if !same_stat(&before, &after) {
        bail!("exact directory changed before unlinkat")
    }
    if unsafe {
        libc::unlinkat(
            parent_fd.as_raw_fd(),
            leaf_encoded.as_ptr(),
            libc::AT_REMOVEDIR,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error()).context("remove exact empty directory");
    }
    File::from(parent_fd).sync_all()?;
    if path_present(path)? {
        bail!("exact directory remains after held-parent removal")
    }
    Ok(())
}

fn attest_stopped_peer_process(
    pid: i32,
    expected_uid: u32,
    expected_path: &str,
    identity: &substrate_common::macos_retirement_v2::ExecutableIdentityV2,
    _repetition: RepetitionV2,
    _control: PeerSubstitutionControlV2,
    _before_observation_sha256: &str,
) -> Result<StoppedPeerProcessV2> {
    wait_for_sigstop(pid)?;
    attest_observed_stopped_peer_process(pid, expected_uid, expected_path, identity)
}

fn attest_dynamic_peer_or_loader_exit(
    pid: i32,
    expected_uid: u32,
    expected_path: &str,
    identity: &ExecutableIdentityV2,
) -> Result<Option<StoppedPeerProcessV2>> {
    const SSTOP_V2: u32 = 4;
    const SZOMB_V2: u32 = 5;
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match process_info(pid) {
            Ok(process) if process.status == SSTOP_V2 => {
                return attest_observed_stopped_peer_process(
                    pid,
                    expected_uid,
                    expected_path,
                    identity,
                )
                .map(Some);
            }
            Ok(process) if process.status == SZOMB_V2 => return Ok(None),
            Ok(_) => {}
            Err(_) => {
                // SAFETY: signal zero performs only one existence check on the freshly spawned pid.
                if unsafe { libc::kill(pid, 0) } != 0
                    && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                {
                    return Ok(None);
                }
            }
        }
        if std::time::Instant::now() >= deadline {
            bail!("dynamic injection target neither stopped at main nor exited in the fixed bound")
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn attest_observed_stopped_peer_process(
    pid: i32,
    expected_uid: u32,
    expected_path: &str,
    identity: &ExecutableIdentityV2,
) -> Result<StoppedPeerProcessV2> {
    let first = process_info(pid)?;
    if first.status != 4
        || first.pid != pid as u32
        || first.uid != expected_uid
        || pid_path(pid)? != Path::new(expected_path)
    {
        bail!("stopped peer probe differs from its exact exec/uid/path identity")
    }
    let account = canonical_account(expected_uid)?.0;
    let process_start_identity_sha256 = document_sha256_v2(&ProcessStartJoinV2 {
        pid,
        seconds: first.start_seconds,
        microseconds: first.start_microseconds,
    })?;
    let executable_identity_sha256 = document_sha256_v2(identity)?;
    let measured = measure_frozen_executable(frozen_code_for_identity(expected_path, identity)?)?;
    if measured != *identity {
        bail!("stopped peer executable bytes/code/physical identity changed")
    }
    // SAFETY: the exact stopped child is resumed once after its attestation is durable in memory.
    if unsafe { libc::kill(pid, libc::SIGCONT) } != 0 {
        return Err(std::io::Error::last_os_error()).context("resume exact peer probe");
    }
    Ok(StoppedPeerProcessV2 {
        pid,
        effective_uid: first.uid,
        effective_gid: first.gid,
        supplementary_groups: measured_child_supplementary_groups_v2(pid)?,
        canonical_account: account,
        process_start_identity_sha256,
        executable_identity_sha256,
    })
}

fn wait_for_sigstop(pid: i32) -> Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let mut status = 0;
        // SAFETY: pid is the exact freshly spawned child; nohang avoids an unbounded rendezvous.
        let observed = unsafe { libc::waitpid(pid, &mut status, libc::WUNTRACED | libc::WNOHANG) };
        if observed == pid {
            if libc::WIFSTOPPED(status) && libc::WSTOPSIG(status) == libc::SIGSTOP {
                return Ok(());
            }
            bail!("peer probe exited or changed state before its exact SIGSTOP rendezvous")
        }
        if observed < 0 {
            return Err(std::io::Error::last_os_error())
                .context("wait for peer SIGSTOP rendezvous");
        }
        if std::time::Instant::now() >= deadline {
            bail!("peer probe did not enter its exact SIGSTOP rendezvous before the deadline")
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn parse_peer_frame(bytes: &[u8]) -> Result<PeerProbeExchangeResultV2> {
    if bytes.len() < 8 {
        bail!("peer probe output lacks its frame prefix")
    }
    let length = usize::try_from(u64::from_be_bytes(bytes[..8].try_into().unwrap()))?;
    if length == 0 || length > MAX_CHILD_OUTPUT || bytes.len() != length + 8 {
        bail!("peer probe output is not exactly one bounded frame")
    }
    parse_canonical_v2(&bytes[8..])
}

fn parse_stdout_json<T: DeserializeOwned + Serialize>(observed: &ObservedChildV2) -> Result<T> {
    let bytes = one_stdout_line(&observed.stdout)?;
    parse_canonical_v2(bytes)
}

fn one_stdout_line(bytes: &[u8]) -> Result<&[u8]> {
    if bytes.len() < 2 || bytes.len() > MAX_CHILD_OUTPUT || bytes.last() != Some(&b'\n') {
        bail!("sealed child stdout is not one bounded newline receipt")
    }
    let body = &bytes[..bytes.len() - 1];
    if body.contains(&b'\n') {
        bail!("sealed child stdout contains multiple records")
    }
    Ok(body)
}

fn normalize_post_drop_getgroups_v2(
    raw_group_count: libc::c_int,
    only_group: Option<libc::gid_t>,
    effective_gid: libc::gid_t,
) -> std::io::Result<u32> {
    match (raw_group_count, only_group) {
        (0, None) => Ok(0),
        (1, Some(group)) if group == effective_gid => Ok(0),
        _ => Err(std::io::Error::from_raw_os_error(libc::EPERM)),
    }
}

fn sealed_command(
    path: &'static str,
    cwd: &'static str,
    uid: Option<u32>,
    gid: Option<u32>,
) -> Result<MeasuredCommandV2> {
    let (group_measurement_reader, group_measurement_writer) =
        UnixStream::pair().context("create sealed child getgroups measurement socket")?;
    let mut command = Command::new(path);
    command
        .current_dir(cwd)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // SAFETY: this pre-exec membrane uses only async-signal-safe credential syscalls. It clears
    // every inherited supplementary group before selecting the effective GID/UID and refuses to
    // exec unless the kernel reports the resulting supplementary vector is exactly empty.
    unsafe {
        command.pre_exec(move || {
            if uid.is_some() != gid.is_some() {
                return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
            }
            if libc::setgroups(0, std::ptr::null()) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if let Some(gid) = gid {
                if libc::setgid(gid) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            if let Some(uid) = uid {
                if libc::setuid(uid) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            let raw_group_count = libc::getgroups(0, std::ptr::null_mut());
            if raw_group_count < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let effective_gid = libc::getegid();
            let normalized_group_count = if raw_group_count == 0 {
                normalize_post_drop_getgroups_v2(0, None, effective_gid)?
            } else if raw_group_count == 1 {
                let mut only_group = 0;
                let read_group_count = libc::getgroups(1, &mut only_group);
                if read_group_count < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                normalize_post_drop_getgroups_v2(read_group_count, Some(only_group), effective_gid)?
            } else {
                normalize_post_drop_getgroups_v2(raw_group_count, None, effective_gid)?
            };
            let mut frame = [0_u8; SEALED_GROUP_MEASUREMENT_FRAME_BYTES_V2];
            frame[..8].copy_from_slice(&SEALED_GROUP_MEASUREMENT_MAGIC_V2);
            frame[8..12].copy_from_slice(&libc::geteuid().to_ne_bytes());
            frame[12..16].copy_from_slice(&effective_gid.to_ne_bytes());
            frame[16..20].copy_from_slice(&normalized_group_count.to_ne_bytes());
            let mut written = 0_usize;
            while written < frame.len() {
                let count = libc::write(
                    group_measurement_writer.as_raw_fd(),
                    frame[written..].as_ptr().cast(),
                    frame.len() - written,
                );
                if count <= 0 {
                    return Err(std::io::Error::last_os_error());
                }
                written += usize::try_from(count)
                    .map_err(|_| std::io::Error::from_raw_os_error(libc::EOVERFLOW))?;
            }
            Ok(())
        });
    }
    Ok(MeasuredCommandV2::new(command, group_measurement_reader))
}

fn prepare_runner_root() -> Result<()> {
    ensure_root_directory(Path::new(RUNNER_ROOT))
}

fn acquire_activation_membrane_exclusive() -> Result<ActivationMembraneGuardV2> {
    let path = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2);
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: the exact compiled activation membrane is opened as a directory with terminal
    // no-follow. Absence is deliberately fatal; this runner never creates the product root.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return acquire_absent_activation_membrane_for_cleanup_recovery();
        }
        return Err(error).context("open existing finalizer activation membrane");
    }
    // SAFETY: successful open transferred descriptor ownership.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let before = fstat(descriptor.as_raw_fd())?;
    if before.st_uid != 0
        || before.st_gid != 0
        || before.st_nlink < 1
        || (before.st_mode & (libc::S_IFMT | 0o7777)) != libc::S_IFDIR | 0o700
    {
        bail!("finalizer activation membrane is not exact root:wheel 0700 directory state")
    }
    // SAFETY: the live exact directory descriptor is held exclusively across preparation.
    if unsafe { libc::flock(descriptor.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("lock finalizer activation membrane exclusively");
    }
    let after = lstat(path)?.context("finalizer activation membrane disappeared after lock")?;
    if !same_stat(&before, &after) {
        bail!("finalizer activation membrane inode changed while acquiring its exclusive lock")
    }
    if let Some((plan, packet)) = activation_cleanup_recovery_documents()? {
        let root_object = plan
            .objects
            .last()
            .context("native cleanup plan lacks its final root object")?;
        if root_object.path != MAC_R3_FINALIZER_JOURNAL_ROOT_V2
            || root_object.kind != NativeCleanupObjectKindV2::Directory
            || cleanup_directory_stable_identity_sha256(path, &before)?
                != root_object.physical_identity_sha256
        {
            bail!("partial native cleanup activation root changed identity")
        }
        return Ok(ActivationMembraneGuardV2 {
            descriptor,
            identity_sha256: packet.activation_membrane_lock_identity_sha256.clone(),
            root_identity_sha256: packet.activation_membrane_root_identity_sha256.clone(),
            journal_root_lock_identity: packet.journal_root_lock_identity,
            journal_root_lock_identity_sha256: packet.journal_root_lock_identity_sha256,
            root_absent_cleanup_recovery: true,
        });
    }
    let journal_root_lock_identity = initialize_empty_activation_root_lock(&descriptor, path)?;
    let journal_root_lock_identity_sha256 = document_sha256_v2(&journal_root_lock_identity)?;
    let identity_sha256 = document_sha256_v2(&ActivationMembraneIdentityV2 {
        path: MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
        device: before.st_dev as u64,
        inode: before.st_ino,
        owner_uid: before.st_uid,
        owner_gid: before.st_gid,
        mode: u32::from(before.st_mode),
        link_count: u64::from(before.st_nlink),
        exclusive_lock_held: true,
    })?;
    let root_identity_sha256 = document_sha256_v2(&ActivationMembraneRootIdentityV2 {
        path: MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
        device: before.st_dev as u64,
        inode: before.st_ino,
        owner_uid: before.st_uid,
        owner_gid: before.st_gid,
        mode: u32::from(before.st_mode),
        link_count: u64::from(before.st_nlink),
    })?;
    Ok(ActivationMembraneGuardV2 {
        descriptor,
        identity_sha256,
        root_identity_sha256,
        journal_root_lock_identity,
        journal_root_lock_identity_sha256,
        root_absent_cleanup_recovery: false,
    })
}

fn acquire_absent_activation_membrane_for_cleanup_recovery() -> Result<ActivationMembraneGuardV2> {
    let (plan, packet) = activation_cleanup_recovery_documents()?
        .context("absent activation membrane lacks an acknowledged cleanup plan")?;
    if plan.schema_owner != "substrate.r3-macos-native-evidence-cleanup-plan"
        || plan.schema_version != EXPERIMENT_VERSION_V2
        || plan.experiment_id != EXPERIMENT_ID_V2
        || !plan.deletion_may_begin
        || plan.global_pre_effect_packet_sha256 != document_sha256_v2(&packet)?
        || plan.journal_root_activation_identity_sha256
            != packet.activation_membrane_root_identity_sha256
        || plan.journal_root_lock_identity_sha256 != packet.journal_root_lock_identity_sha256
        || plan.objects.last().map(|entry| entry.path.as_str())
            != Some(MAC_R3_FINALIZER_JOURNAL_ROOT_V2)
    {
        bail!("absent activation membrane cleanup recovery changed its durable authority")
    }
    require_exact_path_absent(Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2))?;
    let parent = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2)
        .parent()
        .context("activation membrane lacks its fixed parent")?;
    let encoded = CString::new(parent.as_os_str().as_bytes())?;
    // SAFETY: fixed no-follow parent is held only as an absence-recovery anchor.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open absent activation membrane recovery parent");
    }
    // SAFETY: successful open transferred ownership.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    Ok(ActivationMembraneGuardV2 {
        descriptor,
        identity_sha256: packet.activation_membrane_lock_identity_sha256.clone(),
        root_identity_sha256: packet.activation_membrane_root_identity_sha256.clone(),
        journal_root_lock_identity: packet.journal_root_lock_identity,
        journal_root_lock_identity_sha256: packet.journal_root_lock_identity_sha256,
        root_absent_cleanup_recovery: true,
    })
}

fn activation_cleanup_recovery_documents(
) -> Result<Option<(NativeEvidenceCleanupPlanV2, GlobalPreEffectPacketV2)>> {
    if let Some(plan) = read_runner_private_cleanup_recovery_optional::<NativeEvidenceCleanupPlanV2>(
        NATIVE_CLEANUP_PLAN_NAME,
    )? {
        let packet = read_runner_global_pre_effect_packet_cleanup_recovery_optional()?
            .context("native cleanup plan lacks its global pre-effect packet")?;
        return Ok(Some((plan, packet)));
    }
    let Some((receipt, export, packet)) = read_staged_native_cleanup_recovery_documents()? else {
        return Ok(None);
    };
    let plan_entry = receipt
        .cleanup_private_archive
        .entries
        .iter()
        .find(|entry| entry.name == NATIVE_CLEANUP_PLAN_NAME)
        .context("staged native cleanup receipt lacks its cleanup plan bytes")?;
    let plan: NativeEvidenceCleanupPlanV2 = parse_canonical_v2(&plan_entry.validate()?)?;
    if document_sha256_v2(&plan)? != receipt.native_cleanup_plan_sha256
        || export.global_pre_effect_sha256 != document_sha256_v2(&packet)?
    {
        bail!("staged native cleanup recovery documents changed their exact joins")
    }
    Ok(Some((plan, packet)))
}

fn read_staged_native_cleanup_recovery_documents() -> Result<
    Option<(
        NativeEvidenceCleanupReceiptV2,
        NativeEvidenceExportV2,
        GlobalPreEffectPacketV2,
    )>,
> {
    let final_path =
        global_external_exchange_path_v2(PublisherArtifactV2::NativeEvidenceCleanupReceipt)?;
    let identity = PublishIdentityV2 {
        owner_uid: 0,
        owner_gid: 0,
        permissions: 0o444,
        parent_uid: DISPOSABLE_HARNESS_UID_V2,
        parent_gid: 0,
        parent_mode: libc::S_IFDIR | 0o700,
        maximum_bytes: MAX_NATIVE_EVIDENCE_DOCUMENT,
    };
    let Some(bytes) = read_staged_file(&final_path, identity)? else {
        return Ok(None);
    };
    let receipt: NativeEvidenceCleanupReceiptV2 = parse_canonical_v2(&bytes)?;
    if canonical_bytes_v2(&receipt)? != bytes {
        bail!("staged native cleanup receipt is not exact canonical bytes")
    }
    let export = read_global_native_evidence_export()?;
    let packet_entry = export
        .runner_private_archive
        .entries
        .iter()
        .find(|entry| entry.name == GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2)
        .context("native evidence export lacks its global pre-effect packet bytes")?;
    let packet: GlobalPreEffectPacketV2 = parse_runner_private_canonical(
        GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
        &packet_entry.validate()?,
    )?;
    let acknowledgement = read_native_evidence_acknowledgement_for_cleanup()?;
    receipt.validate(&export, &acknowledgement, &packet)?;
    Ok(Some((receipt, export, packet)))
}

fn read_runner_or_archived_canonical<T: DeserializeOwned + Serialize>(name: &str) -> Result<T> {
    if let Some(value) = read_runner_private_optional::<T>(name)? {
        return Ok(value);
    }
    let (receipt, export, _) = read_staged_native_cleanup_recovery_documents()?
        .context("runner archive recovery lacks its staged cleanup receipt")?;
    let entry = export
        .runner_private_archive
        .entries
        .iter()
        .chain(&receipt.cleanup_private_archive.entries)
        .find(|entry| entry.name == name)
        .context("runner private archives omit the requested recovery document")?;
    parse_canonical_v2(&entry.validate()?)
}

fn initialize_empty_activation_root_lock(
    activation_root: &OwnedFd,
    root_path: &Path,
) -> Result<RootInstallPhysicalIdentityV2> {
    let lock_name = CString::new("journal-root.lock")?;
    let names = exact_directory_names(root_path)?;
    if !(names.is_empty() || names == ["journal-root.lock"]) {
        bail!("activation root lock initialization found alternate sibling state")
    }
    let flags = libc::O_RDWR | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    let mut raw = unsafe { libc::openat(activation_root.as_raw_fd(), lock_name.as_ptr(), flags) };
    if raw < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
        if !names.is_empty() {
            bail!("activation root lock disappeared beside nonempty state")
        }
        raw = unsafe {
            libc::openat(
                activation_root.as_raw_fd(),
                lock_name.as_ptr(),
                flags | libc::O_CREAT | libc::O_EXCL,
                0o600,
            )
        };
    }
    if raw < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open or initialize exact activation journal root lock");
    }
    let lock = unsafe { File::from_raw_fd(raw) };
    if unsafe { libc::fchown(lock.as_raw_fd(), 0, 0) } != 0
        || unsafe { libc::fchmod(lock.as_raw_fd(), 0o600) } != 0
    {
        return Err(std::io::Error::last_os_error())
            .context("freeze activation journal root lock owner/mode");
    }
    let before = lock
        .metadata()
        .context("inspect activation journal root lock")?;
    if !before.file_type().is_file()
        || before.uid() != 0
        || before.gid() != 0
        || before.nlink() != 1
        || before.mode() & (libc::S_IFMT as u32 | 0o7777) != libc::S_IFREG as u32 | 0o600
        || before.len() != 0
    {
        bail!("activation journal root lock identity changed")
    }
    lock.sync_all()
        .context("fsync initialized activation journal root lock")?;
    if unsafe { libc::fsync(activation_root.as_raw_fd()) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("fsync activation root after lock initialization");
    }
    let reopened_raw = unsafe {
        libc::openat(
            activation_root.as_raw_fd(),
            lock_name.as_ptr(),
            libc::O_RDWR | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if reopened_raw < 0 {
        return Err(std::io::Error::last_os_error())
            .context("reopen initialized activation journal root lock");
    }
    let reopened = unsafe { File::from_raw_fd(reopened_raw) };
    let after = reopened
        .metadata()
        .context("reinspect activation journal root lock")?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.uid() != after.uid()
        || before.gid() != after.gid()
        || before.mode() != after.mode()
        || before.nlink() != after.nlink()
        || after.len() != 0
        || exact_directory_names(root_path)? != ["journal-root.lock"]
    {
        bail!("activation journal root lock changed after durable reopen")
    }
    root_install_physical_identity(
        MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2,
        &fstat(reopened.as_raw_fd())?,
    )
}

fn reattest_activation_journal_root_lock(guard: &ActivationMembraneGuardV2) -> Result<()> {
    let name = CString::new("journal-root.lock")?;
    let raw = unsafe {
        libc::openat(
            guard.descriptor.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDWR | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error())
            .context("reattest exact activation journal-root lock");
    }
    let reopened = unsafe { OwnedFd::from_raw_fd(raw) };
    let stat = fstat(reopened.as_raw_fd())?;
    let identity =
        root_install_physical_identity(MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2, &stat)?;
    let live = lstat(Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2))?
        .context("activation journal-root lock disappeared during reattestation")?;
    if identity != guard.journal_root_lock_identity
        || document_sha256_v2(&identity)? != guard.journal_root_lock_identity_sha256
        || !same_stat(&stat, &live)
    {
        bail!("activation journal-root lock differs from its durable created identity")
    }
    Ok(())
}

fn release_activation_membrane_exclusive(guard: &ActivationMembraneGuardV2) -> Result<()> {
    if unsafe { libc::flock(guard.descriptor.as_raw_fd(), libc::LOCK_UN) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("release finalizer activation membrane exclusive lock");
    }
    let held = fstat(guard.descriptor.as_raw_fd())?;
    let live = lstat(Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2))?
        .context("activation membrane disappeared while releasing preparation lock")?;
    if held.st_dev != live.st_dev
        || held.st_ino != live.st_ino
        || held.st_uid != live.st_uid
        || held.st_gid != live.st_gid
        || held.st_mode != live.st_mode
    {
        bail!("activation membrane path changed while releasing its exclusive lock")
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn install_and_validate_global_pre_effect_proof(
    security: &mut NonInteractiveSecurity,
    inputs: &FrozenRunnerInputs,
    runner: &RunnerSelfObservationV2,
    owner_probe: &crate::NonmatchOwnerProbeV3,
    owner_access_probe: &crate::CanonicalAccessDigest,
    activation_membrane: &ActivationMembraneGuardV2,
    candidate_freeze_authority: &CandidateFreezeAuthorityV2,
    root_install_claims: &RootInstallClaimsBindingV2,
) -> Result<GlobalPreEffectPacketV2> {
    reattest_activation_journal_root_lock(activation_membrane)?;
    let disposable_baseline = RunnerDisposableBaselineV2 {
        schema_owner: "substrate.r3-macos-disposable-experiment-runner-baseline",
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2,
        runner_self_observation_sha256: document_sha256_v2(runner)?,
        owner_account_and_setid_probe_sha256: document_sha256_v2(owner_probe)?,
        owner_secaccess_readback_sha256: document_sha256_v2(owner_access_probe)?,
        journal_root_lock_identity_sha256: activation_membrane
            .journal_root_lock_identity_sha256
            .clone(),
        creator_identity: &inputs.creator_identity,
        creator_signing_posture: &inputs.creator_signing_posture,
        wrong_identity: &inputs.wrong_identity,
        wrong_signing_posture: &inputs.wrong_signing_posture,
        publisher_identity: &inputs.prepared.publisher_identity,
        publisher_signing_posture: &inputs.publisher_signing_posture,
        observer_identity: &inputs.observer_identity,
        observer_signing_posture: &inputs.observer_signing_posture,
        creator_marker_root: MARKER_ROOT,
        creator_marker_path: MARKER_PATH,
        system_keychain_path: SYSTEM_KEYCHAIN_PATH,
        creator_scopes: [CREATOR_REPETITION_SCOPE_1, CREATOR_REPETITION_SCOPE_2],
        finalizer_scopes: [DISPOSABLE_FINALIZER_SCOPE_1, DISPOSABLE_FINALIZER_SCOPE_2],
    };
    let disposable_baseline_bytes = canonical_bytes_v2(&disposable_baseline)?;
    write_runner_receipt(
        "global-pre-effect-disposable-baseline.v2.json",
        &disposable_baseline_bytes,
    )?;
    let nonce_absence_baseline =
        build_global_nonce_absence_baseline(security, &sha256_hex_v2(&disposable_baseline_bytes))?;
    let packet = build_global_pre_effect_packet_v2(
        &inputs.prepared,
        &inputs.candidate,
        &inputs.peer,
        GlobalPreEffectDynamicBindingsV2 {
            candidate_freeze_manifest: candidate_freeze_authority.manifest.clone(),
            installed_candidate_manifest: candidate_freeze_authority.binding.clone(),
            root_install_claims: root_install_claims.clone(),
            runner_process_attestation_sha256: inputs.runner_process_attestation_sha256.clone(),
            activation_membrane_lock_identity_sha256: activation_membrane.identity_sha256.clone(),
            activation_membrane_root_identity_sha256: activation_membrane
                .root_identity_sha256
                .clone(),
            journal_root_lock_identity: activation_membrane.journal_root_lock_identity.clone(),
            nonce_absence_baseline,
        },
    )?;
    packet.validate(&inputs.prepared, &inputs.candidate, &inputs.peer)?;
    let bytes = canonical_bytes_v2(&packet)?;
    write_runner_receipt(GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2, &bytes)?;
    publish_global_pre_effect_packet(&packet, &bytes)?;
    reattest_activation_journal_root_lock(activation_membrane)?;
    Ok(packet)
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct CandidateManifestAncestorIdentityV2 {
    path: String,
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    link_count: u64,
}

struct OpenCandidateManifestV2 {
    file: File,
    stat: libc::stat,
    ancestor_chain_sha256: String,
    _directory_descriptors: Vec<OwnedFd>,
}

struct CandidateFreezeAuthorityV2 {
    manifest: CandidateFreezeManifestV2,
    binding: InstalledCandidateManifestBindingV2,
}

fn load_root_install_claims(
    manifest: &CandidateFreezeManifestV2,
) -> Result<RootInstallClaimsBindingV2> {
    manifest.validate()?;
    let root = Path::new(ROOT_INSTALL_CLAIM_ROOT_V2);
    require_directory(root, 0, 0, libc::S_IFDIR | 0o700)?;
    let root_encoded = CString::new(root.as_os_str().as_bytes())?;
    // SAFETY: one fixed root-owned claim directory, opened no-follow as a directory.
    let root_fd = unsafe {
        libc::open(
            root_encoded.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if root_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open root-install claim directory");
    }
    // SAFETY: successful open transferred descriptor ownership.
    let descriptor = unsafe { OwnedFd::from_raw_fd(root_fd) };
    let before = lstat(root)?.context("root-install claim directory is absent")?;
    let held = fstat(descriptor.as_raw_fd())?;
    let after = lstat(root)?.context("root-install claim directory disappeared")?;
    if !same_stat(&before, &held) || !same_stat(&held, &after) {
        bail!("root-install claim directory changed across pathname/descriptor reattestation")
    }
    let current_root_identity = root_install_physical_identity(ROOT_INSTALL_CLAIM_ROOT_V2, &held)?;
    let current_root_sha256 = document_sha256_v2(&current_root_identity)?;

    let preclaim_bytes = stable_read_file(
        Path::new(ROOT_INSTALL_PRECLAIM_PATH_V2),
        0,
        Some(libc::S_IFREG | 0o400),
    )?;
    let preclaim: RootInstallPreclaimV2 = parse_canonical_v2(&preclaim_bytes)?;
    if canonical_bytes_v2(&preclaim)? != preclaim_bytes {
        bail!("root-install preclaim is not canonical")
    }
    preclaim.validate(manifest)?;

    let completion_bytes = stable_read_file(
        Path::new(ROOT_INSTALL_COMPLETION_PATH_V2),
        0,
        Some(libc::S_IFREG | 0o400),
    )?;
    let completion: RootInstallCompletionV2 = parse_canonical_v2(&completion_bytes)?;
    if canonical_bytes_v2(&completion)? != completion_bytes {
        bail!("root-install completion is not canonical")
    }
    completion.validate(manifest, &preclaim)?;
    // The completion document is evidence, not authority by itself. Before the first native
    // Security query, join every installed directory and artifact back to its live no-follow
    // pathname/descriptor/pathname identity and (for files) its exact bytes.
    for expected in &completion.installed_directories {
        reattest_root_install_identity(expected, None)?;
    }
    for expected in &completion.installed_artifacts {
        reattest_root_install_identity(&expected.physical_identity, Some(&expected.sha256))?;
    }

    let preclaim_stat = lstat(Path::new(ROOT_INSTALL_PRECLAIM_PATH_V2))?
        .context("root-install preclaim disappeared after read")?;
    let observed_preclaim_identity =
        root_install_physical_identity(ROOT_INSTALL_PRECLAIM_PATH_V2, &preclaim_stat)?;
    if observed_preclaim_identity != completion.preclaim_leaf_identity {
        bail!("root-install preclaim leaf physical identity changed after completion")
    }
    let completion_stat = lstat(Path::new(ROOT_INSTALL_COMPLETION_PATH_V2))?
        .context("root-install completion disappeared after read")?;
    let completion_leaf_identity =
        root_install_physical_identity(ROOT_INSTALL_COMPLETION_PATH_V2, &completion_stat)?;
    let final_root =
        lstat(root)?.context("root-install claim directory disappeared after reads")?;
    if !same_stat(&held, &final_root) {
        bail!("root-install claim directory changed while reading its immutable claims")
    }
    let binding = RootInstallClaimsBindingV2 {
        preclaim_sha256: sha256_hex_v2(&preclaim_bytes),
        completion_sha256: sha256_hex_v2(&completion_bytes),
        preclaim,
        completion,
        claim_root_current_identity: current_root_identity,
        claim_root_pathname_before_sha256: current_root_sha256.clone(),
        claim_root_descriptor_sha256: current_root_sha256.clone(),
        claim_root_pathname_after_sha256: current_root_sha256,
        completion_leaf_identity,
    };
    binding.validate(manifest)?;
    Ok(binding)
}

fn reattest_root_install_identity(
    expected: &RootInstallPhysicalIdentityV2,
    expected_sha256: Option<&str>,
) -> Result<()> {
    let path = Path::new(&expected.path);
    let before = lstat(path)?.context("root-install completion path is absent")?;
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    let directory = (expected.mode & u32::from(libc::S_IFMT)) == u32::from(libc::S_IFDIR);
    let flags = libc::O_RDONLY
        | libc::O_CLOEXEC
        | libc::O_NOFOLLOW
        | if directory { libc::O_DIRECTORY } else { 0 };
    // SAFETY: one manifest-validated exact path, opened with terminal no-follow.
    let raw = unsafe { libc::open(encoded.as_ptr(), flags) };
    if raw < 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("open installed root-install identity {}", expected.path));
    }
    // SAFETY: successful open transfers descriptor ownership.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let held = fstat(descriptor.as_raw_fd())?;
    let after = lstat(path)?.context("root-install completion path disappeared")?;
    if !same_stat(&before, &held) || !same_stat(&held, &after) {
        bail!("root-install completion path changed across descriptor reattestation")
    }
    match expected_sha256 {
        None if directory => {
            // These six directories are activation membranes whose contents, mtime, size, and
            // nlink legitimately evolve under the closed runner journals. Rejoin retains the
            // immutable inode/owner/type/mode authority and validates contents via their typed
            // cursors, rather than falsely demanding the pre-run directory metadata snapshot.
            if held.st_dev as u64 != expected.device
                || held.st_ino != expected.inode
                || held.st_uid != expected.uid
                || held.st_gid != expected.gid
                || u32::from(held.st_mode) != expected.mode
                || held.st_nlink < 2
            {
                bail!("live root-install directory stable identity differs from completion")
            }
        }
        Some(expected_sha256) if !directory => {
            if root_install_physical_identity(&expected.path, &held)? != *expected {
                bail!("live root-install file identity differs from completion evidence")
            }
            if expected.size > INSTALLED_ARTIFACT_MAX_BYTES {
                bail!("installed root-install artifact exceeds its compiled read bound")
            }
            let expected_size = usize::try_from(expected.size)
                .context("installed root-install artifact size exceeds usize")?;
            let mut bytes = Vec::with_capacity(expected_size);
            let mut file = File::from(descriptor);
            Read::by_ref(&mut file)
                .take(expected.size + 1)
                .read_to_end(&mut bytes)?;
            let descriptor_after_read = fstat(file.as_raw_fd())?;
            let final_path =
                lstat(path)?.context("root-install artifact disappeared during read")?;
            if !same_stat(&held, &descriptor_after_read)
                || !same_stat(&descriptor_after_read, &final_path)
                || bytes.len() != expected_size
                || sha256_hex_v2(&bytes) != expected_sha256
            {
                bail!("live root-install artifact bytes or identity differ from completion")
            }
        }
        _ => bail!("root-install completion mixed a file and directory identity"),
    }
    Ok(())
}

fn root_install_physical_identity(
    path: &str,
    stat: &libc::stat,
) -> Result<RootInstallPhysicalIdentityV2> {
    Ok(RootInstallPhysicalIdentityV2 {
        path: path.to_owned(),
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        uid: stat.st_uid,
        gid: stat.st_gid,
        mode: u32::from(stat.st_mode),
        link_count: u64::from(stat.st_nlink),
        size: u64::try_from(stat.st_size).context("root-install physical size is negative")?,
        modified_seconds: stat.st_mtime,
        modified_nanoseconds: stat.st_mtime_nsec,
    })
}

fn load_candidate_freeze_authority() -> Result<CandidateFreezeAuthorityV2> {
    let installed_path = Path::new(CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2);
    let mut installed_before = open_candidate_manifest_nofollow(installed_path)?;
    let descriptor_before = installed_before.stat;
    if descriptor_before.st_size < 0 || descriptor_before.st_size as usize > MAX_CHILD_OUTPUT {
        bail!("installed candidate manifest exceeds its fixed one-MiB bound")
    }
    let mut installed_bytes = Vec::with_capacity(descriptor_before.st_size as usize);
    installed_before.file.read_to_end(&mut installed_bytes)?;
    let descriptor_after_read = fstat(installed_before.file.as_raw_fd())?;
    if !same_stat(&descriptor_before, &descriptor_after_read)
        || descriptor_after_read.st_size != installed_bytes.len() as libc::off_t
    {
        bail!("installed candidate manifest changed during descriptor read")
    }
    let installed_manifest: CandidateFreezeManifestV2 = parse_canonical_v2(&installed_bytes)?;
    installed_manifest.validate()?;
    if canonical_bytes_v2(&installed_manifest)? != installed_bytes {
        bail!("installed candidate manifest is not exact canonical JSON")
    }
    let installed_sha256 = sha256_hex_v2(&installed_bytes);
    if installed_sha256 != document_sha256_v2(&installed_manifest)? {
        bail!("installed candidate manifest byte and document digests differ")
    }

    let installed_after = open_candidate_manifest_nofollow(installed_path)?;
    if installed_before.ancestor_chain_sha256 != installed_after.ancestor_chain_sha256
        || !same_stat(&descriptor_before, &installed_after.stat)
    {
        bail!("installed candidate manifest ancestry or file identity changed during reopen")
    }

    let store = ExperimentStoreV2::open_fixed()?;
    let (manifest, manifest_observation) = store.read_candidate_freeze_manifest()?;
    manifest.validate()?;
    if manifest != installed_manifest
        || manifest_observation.absolute_path != CANDIDATE_FREEZE_MANIFEST_PATH_V2
        || manifest_observation.sha256 != installed_sha256
        || manifest_observation.sha256 != document_sha256_v2(&manifest)?
    {
        bail!("candidate freeze manifest changed during canonical stable reopen")
    }
    let (reviewed, reviewed_observation) = store.read_candidate_freeze_admin_block()?;
    if reviewed.is_empty()
        || reviewed_observation.sha256 != manifest.reviewed_admin_block_sha256
        || sha256_hex_v2(&reviewed) != manifest.reviewed_admin_block_sha256
    {
        bail!("candidate freeze reviewed administrator block changed before pre-effect binding")
    }
    let (source_hashes, source_hashes_observation) = store.read_candidate_freeze_source_hashes()?;
    let (coordinator_build, coordinator_build_observation) =
        store.read_candidate_freeze_coordinator_build_inputs()?;
    let (global_build, global_build_observation) =
        store.read_candidate_freeze_global_build_inputs()?;
    let (coordinator_provenance, coordinator_provenance_observation) =
        store.read_candidate_freeze_coordinator_provenance_input()?;
    let (global_provenance, global_provenance_observation) =
        store.read_candidate_freeze_global_provenance_input()?;
    let (manifest_input, manifest_input_observation) =
        store.read_candidate_freeze_manifest_input()?;
    let rebuilt_supporting = build_candidate_freeze_supporting_manifests_v2(&manifest_input)?;
    let rebuilt_manifest = build_candidate_freeze_manifest_v2(manifest_input.clone())?;
    let rebuilt_coordinator_provenance =
        candidate_freeze_coordinator_provenance_input_v2(&manifest_input);
    let rebuilt_global_provenance = candidate_freeze_global_provenance_input_v2(&manifest_input)?;
    if rebuilt_manifest != installed_manifest
        || manifest_input_observation.sha256 != installed_manifest.manifest_input_sha256
        || rebuilt_supporting.source_hashes != source_hashes
        || rebuilt_supporting.coordinator_build_inputs != coordinator_build
        || rebuilt_supporting.global_build_inputs != global_build
        || rebuilt_coordinator_provenance != coordinator_provenance
        || rebuilt_global_provenance != global_provenance
        || source_hashes_observation.sha256 != document_sha256_v2(&source_hashes)?
        || source_hashes_observation.sha256 != installed_manifest.source_hashes_manifest_sha256
        || coordinator_build_observation.sha256 != document_sha256_v2(&coordinator_build)?
        || coordinator_build_observation.sha256
            != installed_manifest.coordinator_build_input_manifest_sha256
        || coordinator_build.input_set_sha256 != installed_manifest.coordinator_build_digest
        || global_build_observation.sha256 != document_sha256_v2(&global_build)?
        || global_build_observation.sha256 != installed_manifest.global_build_input_manifest_sha256
        || global_build.input_set_sha256 != installed_manifest.global_build_digest
        || coordinator_provenance_observation.sha256 != document_sha256_v2(&coordinator_provenance)?
        || coordinator_provenance_observation.sha256
            != installed_manifest.coordinator_provenance_input_sha256
        || global_provenance_observation.sha256 != document_sha256_v2(&global_provenance)?
        || global_provenance_observation.sha256 != installed_manifest.global_provenance_input_sha256
    {
        bail!("candidate freeze supporting manifests changed from the root-installed authority")
    }

    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct PhysicalIdentityV2<'a> {
        path: &'a str,
        sha256: &'a str,
        device: u64,
        inode: u64,
        owner_uid: u32,
        owner_gid: u32,
        mode: u32,
        link_count: u64,
        size: u64,
    }
    let mode = u32::from(descriptor_before.st_mode & 0o7777);
    let size = u64::try_from(descriptor_before.st_size)
        .context("installed candidate manifest has a negative size")?;
    let physical_identity_sha256 = document_sha256_v2(&PhysicalIdentityV2 {
        path: CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2,
        sha256: &installed_sha256,
        device: descriptor_before.st_dev as u64,
        inode: descriptor_before.st_ino,
        owner_uid: descriptor_before.st_uid,
        owner_gid: descriptor_before.st_gid,
        mode,
        link_count: u64::from(descriptor_before.st_nlink),
        size,
    })?;
    let binding = InstalledCandidateManifestBindingV2 {
        external_manifest_path: CANDIDATE_FREEZE_MANIFEST_PATH_V2.to_owned(),
        installed_manifest_path: CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2.to_owned(),
        external_manifest_sha256: manifest_observation.sha256,
        installed_manifest_sha256: installed_sha256,
        reviewed_admin_block_sha256: reviewed_observation.sha256,
        ancestor_chain_before_sha256: installed_before.ancestor_chain_sha256,
        ancestor_chain_after_sha256: installed_after.ancestor_chain_sha256,
        pathname_before_identity_sha256: physical_identity_sha256.clone(),
        descriptor_identity_sha256: physical_identity_sha256.clone(),
        pathname_after_identity_sha256: physical_identity_sha256,
        device: descriptor_before.st_dev as u64,
        inode: descriptor_before.st_ino,
        owner_uid: descriptor_before.st_uid,
        owner_gid: descriptor_before.st_gid,
        mode,
        link_count: u64::from(descriptor_before.st_nlink),
        size,
    };
    binding.validate(&installed_manifest)?;
    Ok(CandidateFreezeAuthorityV2 {
        manifest: installed_manifest,
        binding,
    })
}

fn open_candidate_manifest_nofollow(path: &Path) -> Result<OpenCandidateManifestV2> {
    if path != Path::new(CANDIDATE_FREEZE_INSTALLED_MANIFEST_PATH_V2) || !path.is_absolute() {
        bail!("candidate manifest open escaped its one compiled installed path")
    }
    let components = path
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_os_string()),
            std::path::Component::RootDir => None,
            _ => Some(std::ffi::OsString::new()),
        })
        .collect::<Vec<_>>();
    if components.is_empty() || components.iter().any(|value| value.is_empty()) {
        bail!("installed candidate manifest path has a nonphysical component")
    }
    let root = CString::new("/")?;
    // SAFETY: the fixed root path is NUL-terminated and opened no-follow as a directory.
    let root_fd = unsafe {
        libc::open(
            root.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if root_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("open filesystem root no-follow");
    }
    // SAFETY: successful open transferred ownership.
    let mut directories = vec![unsafe { OwnedFd::from_raw_fd(root_fd) }];
    let root_stat = fstat(directories[0].as_raw_fd())?;
    validate_candidate_manifest_ancestor(Path::new("/"), &root_stat)?;
    let mut identities = vec![candidate_manifest_ancestor_identity(
        Path::new("/"),
        &root_stat,
    )];
    let mut current = PathBuf::from("/");
    for component in &components[..components.len() - 1] {
        let encoded = CString::new(component.as_bytes())?;
        let parent = directories
            .last()
            .context("candidate manifest directory chain lost its parent descriptor")?;
        // SAFETY: exact single component, held parent descriptor, and O_NOFOLLOW directory open.
        let raw = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                encoded.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error())
                .context("open installed candidate manifest ancestor no-follow");
        }
        // SAFETY: successful openat transferred ownership.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        let stat = fstat(descriptor.as_raw_fd())?;
        current.push(component);
        validate_candidate_manifest_ancestor(&current, &stat)?;
        identities.push(candidate_manifest_ancestor_identity(&current, &stat));
        directories.push(descriptor);
    }
    let leaf = CString::new(
        components
            .last()
            .context("candidate manifest path lacks a leaf")?
            .as_bytes(),
    )?;
    let parent = directories
        .last()
        .context("candidate manifest path lacks its held parent")?;
    // SAFETY: exact compiled leaf, held no-follow ancestry, and terminal O_NOFOLLOW open.
    let raw = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            leaf.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error())
            .context("open installed candidate manifest leaf no-follow");
    }
    // SAFETY: successful openat transferred ownership.
    let file = unsafe { File::from_raw_fd(raw) };
    let stat = fstat(file.as_raw_fd())?;
    if stat.st_uid != 0
        || stat.st_gid != 0
        || stat.st_nlink != 1
        || (stat.st_mode & (libc::S_IFMT | 0o7777)) != (libc::S_IFREG | 0o400)
    {
        bail!("installed candidate manifest leaf is not root:wheel 0400 one-link regular state")
    }
    Ok(OpenCandidateManifestV2 {
        file,
        stat,
        ancestor_chain_sha256: document_sha256_v2(&identities)?,
        _directory_descriptors: directories,
    })
}

fn validate_candidate_manifest_ancestor(path: &Path, stat: &libc::stat) -> Result<()> {
    if stat.st_uid != 0
        || (stat.st_mode & libc::S_IFMT) != libc::S_IFDIR
        || (stat.st_mode & 0o022) != 0
    {
        bail!("installed candidate manifest ancestry is not root-owned immutable directory state")
    }
    if let Some(expected) = expected_candidate_freeze_install_directories_v2()
        .iter()
        .find(|expected| Path::new(&expected.path) == path)
    {
        if stat.st_uid != expected.uid
            || stat.st_gid != expected.gid
            || u32::from(stat.st_mode & 0o7777) != expected.mode
        {
            bail!("installed candidate manifest support directory differs from the freeze plan")
        }
    }
    Ok(())
}

fn candidate_manifest_ancestor_identity(
    path: &Path,
    stat: &libc::stat,
) -> CandidateManifestAncestorIdentityV2 {
    CandidateManifestAncestorIdentityV2 {
        path: path.to_string_lossy().into_owned(),
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        owner_uid: stat.st_uid,
        owner_gid: stat.st_gid,
        mode: u32::from(stat.st_mode),
        link_count: u64::from(stat.st_nlink),
    }
}

fn build_global_nonce_absence_baseline(
    security: &mut NonInteractiveSecurity,
    supporting_baseline_sha256: &str,
) -> Result<GlobalNonceAbsenceBaselineV2> {
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct ExactObservation<'a> {
        domain: &'static str,
        kind: GlobalNonceObjectKindV2,
        repetition: u8,
        identity: &'a str,
        classification: &'a GlobalExactAbsenceClassificationV2,
        native_evidence: String,
    }

    let plan = global_nonce_absence_plan_v2();
    let (baseline_result, report_sha256, report_bytes) = observe_root_operation_with_raw(
        || {
            let mut observations = Vec::with_capacity(plan.len());
            for probe in &plan {
                let native_evidence = match probe.kind {
                    GlobalNonceObjectKindV2::KeychainApplicationTag => {
                        let status = security.require_exact_key_attribute_absent(
                            ExactAbsentKeyAttribute::ApplicationTag,
                            probe.identity.as_bytes(),
                        )?;
                        if probe.classification
                            != (GlobalExactAbsenceClassificationV2::SecItemNotFound {
                                raw_os_status: status,
                            })
                        {
                            bail!("application-tag absence status changed from the frozen plan")
                        }
                        document_sha256_v2(&("SecItemCopyMatching", status, &probe.identity))?
                    }
                    GlobalNonceObjectKindV2::KeychainLabel => {
                        let status = security.require_exact_key_attribute_absent(
                            ExactAbsentKeyAttribute::Label,
                            probe.identity.as_bytes(),
                        )?;
                        if probe.classification
                            != (GlobalExactAbsenceClassificationV2::SecItemNotFound {
                                raw_os_status: status,
                            })
                        {
                            bail!("key-label absence status changed from the frozen plan")
                        }
                        document_sha256_v2(&("SecItemCopyMatching", status, &probe.identity))?
                    }
                    GlobalNonceObjectKindV2::FilesystemPath => {
                        require_exact_path_absent(Path::new(&probe.identity))?;
                        document_sha256_v2(&("lstat", libc::ENOENT, &probe.identity))?
                    }
                    GlobalNonceObjectKindV2::LaunchdLabel => {
                        require_exact_launchd_label_absent(&probe.identity)?
                    }
                    GlobalNonceObjectKindV2::UnixEndpoint => {
                        require_exact_path_absent(Path::new(&probe.identity))?;
                        document_sha256_v2(&("unix-endpoint-lstat", libc::ENOENT, &probe.identity))?
                    }
                    GlobalNonceObjectKindV2::ProcessExecutablePath => {
                        require_exact_process_path_absent(Path::new(&probe.identity))?
                    }
                };
                let observation_sha256 = document_sha256_v2(&ExactObservation {
                    domain: "substrate.r3-macos-global-exact-absence-observation.v2",
                    kind: probe.kind,
                    repetition: probe.repetition,
                    identity: &probe.identity,
                    classification: &probe.classification,
                    native_evidence,
                })?;
                observations.push(GlobalNonceAbsenceObservationV2 {
                    kind: probe.kind,
                    repetition: probe.repetition,
                    identity: probe.identity.clone(),
                    exact_predicate_sha256: document_sha256_v2(probe)?,
                    classification: probe.classification,
                    observation_sha256,
                });
            }
            let observation_set_sha256 = document_sha256_v2(&observations)?;
            Ok((observations, observation_set_sha256))
        },
        |alert| persist_root_operation_terminal_alert("global-nonce-absence-baseline", alert),
    )?;
    persist_root_operation_securityagent_report(
        "global-nonce-absence-baseline.ui.observation.v2.json",
        &report_sha256,
        &report_bytes,
    )?;
    let (observations, observation_set_sha256) = baseline_result?;
    let securityagent_report = SecurityAgentRawReportEvidenceV2 {
        raw_report_base64url: URL_SAFE_NO_PAD.encode(&report_bytes),
        raw_report_sha256: report_sha256,
        raw_report_byte_length: u64::try_from(report_bytes.len())?,
    };
    let baseline = GlobalNonceAbsenceBaselineV2 {
        schema_owner:
            substrate_r3_macos_finalizer::experiment::pre_effect::GLOBAL_PRE_EFFECT_PACKET_OWNER_V2
                .to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        observations,
        observation_set_sha256,
        supporting_baseline_sha256: supporting_baseline_sha256.to_owned(),
        securityagent_report,
    };
    baseline.validate()?;
    Ok(baseline)
}

fn require_exact_path_absent(path: &Path) -> Result<()> {
    if lstat(path)?.is_some() {
        bail!("frozen pre-effect filesystem identity is unexpectedly present")
    }
    Ok(())
}

fn ensure_finalizer_service_bootstrapped(
    inputs: &FrozenRunnerInputs,
    global_pre_effect: &GlobalPreEffectPacketV2,
) -> Result<FinalizerServiceObservationV2> {
    let creator_path =
        global_external_exchange_path_v2(PublisherArtifactV2::CreatorRouteReceiptSet)?;
    let creator_bytes = stable_read_file(&creator_path, 0, Some(libc::S_IFREG | 0o444))?;
    let creator_receipts: CreatorRouteReceiptSetV2 = parse_canonical_v2(&creator_bytes)?;
    creator_receipts.validate(global_pre_effect, &inputs.peer)?;
    if creator_bytes != canonical_bytes_v2(&creator_receipts)? {
        bail!("creator-route receipt set changed before finalizer bootstrap")
    }

    let plist_bytes = stable_read_file(
        Path::new(MAC_R3_FINALIZER_PLIST_PATH_V2),
        0,
        Some(libc::S_IFREG | 0o644),
    )?;
    if sha256_hex_v2(&plist_bytes) != launch_plist_sha256() {
        bail!("installed finalizer launchd plist changed before bootstrap")
    }
    let prepared = FinalizerBootstrapPreparedV2 {
        schema_owner: "substrate.r3-macos-disposable-finalizer-bootstrap".to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        launchd_label: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_owned(),
        launchd_plist_path: MAC_R3_FINALIZER_PLIST_PATH_V2.to_owned(),
        launchd_plist_sha256: sha256_hex_v2(&plist_bytes),
        endpoint_path: MAC_R3_FINALIZER_ENDPOINT_V2.to_owned(),
        global_pre_effect_sha256: document_sha256_v2(global_pre_effect)?,
        creator_route_receipt_set_sha256: document_sha256_v2(&creator_receipts)?,
        service_absent_before: true,
        endpoint_absent_before: true,
        invocation_may_begin: true,
    };
    validate_bootstrap_prepared(&prepared, global_pre_effect, &creator_receipts)?;
    let existing_prepared = read_runner_private_optional::<FinalizerBootstrapPreparedV2>(
        FINALIZER_BOOTSTRAP_PREPARED_NAME,
    )?;
    let created_now = match existing_prepared {
        Some(existing) => {
            if existing != prepared {
                bail!("finalizer bootstrap prepared cursor changed")
            }
            false
        }
        None => {
            require_exact_launchd_label_absent(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)?;
            require_exact_path_absent(Path::new(MAC_R3_FINALIZER_ENDPOINT_V2))?;
            write_runner_receipt(
                FINALIZER_BOOTSTRAP_PREPARED_NAME,
                &canonical_bytes_v2(&prepared)?,
            )?;
            true
        }
    };

    let result = read_runner_private_optional::<FinalizerBootstrapResultV2>(
        FINALIZER_BOOTSTRAP_RESULT_NAME,
    )?;
    let observation = read_runner_private_optional::<FinalizerServiceObservationV2>(
        FINALIZER_SERVICE_OBSERVATION_NAME,
    )?;
    match bootstrap_recovery_decision(created_now, result.is_some(), observation.is_some()) {
        BootstrapRecoveryDecisionV2::AcceptObserved => {
            let result = result.context("bootstrap observation lacks exact invocation result")?;
            let observation = observation.context("bootstrap observation disappeared")?;
            validate_bootstrap_result(&result, &prepared)?;
            validate_service_observation(&observation, &prepared, &result)?;
            observe_loaded_finalizer_service(&prepared, &result, false)?;
            Ok(observation)
        }
        BootstrapRecoveryDecisionV2::ObserveDurableResult => {
            let result = result.context("bootstrap durable result disappeared")?;
            validate_bootstrap_result(&result, &prepared)?;
            let observation = observe_loaded_finalizer_service(&prepared, &result, true)?;
            write_runner_receipt(
                FINALIZER_SERVICE_OBSERVATION_NAME,
                &canonical_bytes_v2(&observation)?,
            )?;
            Ok(observation)
        }
        BootstrapRecoveryDecisionV2::AmbiguousNoReinvoke => {
            bail!("finalizer bootstrap was prepared without a durable result; refusing blind reinvocation")
        }
        BootstrapRecoveryDecisionV2::Invoke => {
            // The observer starts before launchd can create/activate the service and stays armed
            // through exact service/endpoint readiness.  Any activation is a durable terminal
            // alert; an invalid observer is process-fatal rather than an unobserved bootstrap.
            let (invocation, reports) = observe_rearmed_root_operation_with_raw(
                || {
                    let output = Command::new("/bin/launchctl")
                        .arg("bootstrap")
                        .arg("system")
                        .arg(MAC_R3_FINALIZER_PLIST_PATH_V2)
                        .current_dir("/")
                        .env_clear()
                        .stdin(Stdio::null())
                        .output()?;
                    let result = FinalizerBootstrapResultV2 {
                        schema_owner: "substrate.r3-macos-disposable-finalizer-bootstrap"
                            .to_owned(),
                        schema_version: 2,
                        experiment_id: EXPERIMENT_ID_V2.to_owned(),
                        prepared_sha256: document_sha256_v2(&prepared)?,
                        exit_status: output
                            .status
                            .code()
                            .context("launchctl bootstrap was signaled")?,
                        stdout: bounded_raw_stream_evidence(&output.stdout)?,
                        stderr: bounded_raw_stream_evidence(&output.stderr)?,
                    };
                    validate_bootstrap_result(&result, &prepared)?;
                    let observation = observe_loaded_finalizer_service(&prepared, &result, true)?;
                    Ok((result, observation))
                },
                |alert| {
                    persist_root_operation_terminal_alert(
                        "finalizer-service-bootstrap-startup",
                        alert,
                    )
                },
            )?;
            for (index, (report_sha256, report_bytes)) in reports.iter().enumerate() {
                persist_root_operation_securityagent_report(
                    &format!(
                        "finalizer-service-bootstrap-startup-window-{index:02}.ui.observation.v2.json"
                    ),
                    report_sha256,
                    report_bytes,
                )?;
            }
            let (result, observation) = invocation?;
            write_runner_receipt(
                FINALIZER_BOOTSTRAP_RESULT_NAME,
                &canonical_bytes_v2(&result)?,
            )?;
            validate_bootstrap_result(&result, &prepared)?;
            write_runner_receipt(
                FINALIZER_SERVICE_OBSERVATION_NAME,
                &canonical_bytes_v2(&observation)?,
            )?;
            Ok(observation)
        }
    }
}

fn bootstrap_recovery_decision(
    prepared_created_now: bool,
    result_present: bool,
    observation_present: bool,
) -> BootstrapRecoveryDecisionV2 {
    match (prepared_created_now, result_present, observation_present) {
        (_, true, true) => BootstrapRecoveryDecisionV2::AcceptObserved,
        (_, false, true) => BootstrapRecoveryDecisionV2::AmbiguousNoReinvoke,
        (true, false, false) => BootstrapRecoveryDecisionV2::Invoke,
        (false, false, false) => BootstrapRecoveryDecisionV2::AmbiguousNoReinvoke,
        (_, true, false) => BootstrapRecoveryDecisionV2::ObserveDurableResult,
    }
}

fn validate_bootstrap_prepared(
    prepared: &FinalizerBootstrapPreparedV2,
    global_pre_effect: &GlobalPreEffectPacketV2,
    creator_receipts: &CreatorRouteReceiptSetV2,
) -> Result<()> {
    if prepared.schema_owner != "substrate.r3-macos-disposable-finalizer-bootstrap"
        || prepared.schema_version != 2
        || prepared.experiment_id != EXPERIMENT_ID_V2
        || prepared.launchd_label != MAC_R3_FINALIZER_LAUNCHD_LABEL_V2
        || prepared.launchd_plist_path != MAC_R3_FINALIZER_PLIST_PATH_V2
        || prepared.launchd_plist_sha256 != launch_plist_sha256()
        || prepared.endpoint_path != MAC_R3_FINALIZER_ENDPOINT_V2
        || prepared.global_pre_effect_sha256 != document_sha256_v2(global_pre_effect)?
        || prepared.creator_route_receipt_set_sha256 != document_sha256_v2(creator_receipts)?
        || !prepared.service_absent_before
        || !prepared.endpoint_absent_before
        || !prepared.invocation_may_begin
    {
        bail!("finalizer bootstrap prepared cursor changed its closed prerequisites")
    }
    Ok(())
}

fn validate_bootstrap_result(
    result: &FinalizerBootstrapResultV2,
    prepared: &FinalizerBootstrapPreparedV2,
) -> Result<()> {
    let stdout = result.stdout.validate()?;
    let stderr = result.stderr.validate()?;
    if result.schema_owner != "substrate.r3-macos-disposable-finalizer-bootstrap"
        || result.schema_version != 2
        || result.experiment_id != EXPERIMENT_ID_V2
        || result.prepared_sha256 != document_sha256_v2(prepared)?
        || result.exit_status != 0
        || !stdout.is_empty()
        || !stderr.is_empty()
    {
        bail!("launchctl bootstrap did not return the frozen successful classification")
    }
    Ok(())
}

fn observe_loaded_finalizer_service(
    prepared: &FinalizerBootstrapPreparedV2,
    result: &FinalizerBootstrapResultV2,
    allow_bounded_wait: bool,
) -> Result<FinalizerServiceObservationV2> {
    let deadline = std::time::Instant::now()
        + if allow_bounded_wait {
            Duration::from_secs(10)
        } else {
            Duration::ZERO
        };
    loop {
        match observe_loaded_finalizer_service_once(prepared, result) {
            Ok(value) => return Ok(value),
            Err(error) if std::time::Instant::now() < deadline => {
                let _ = error;
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error),
        }
    }
}

fn observe_loaded_finalizer_service_once(
    prepared: &FinalizerBootstrapPreparedV2,
    result: &FinalizerBootstrapResultV2,
) -> Result<FinalizerServiceObservationV2> {
    let target = format!("system/{}", MAC_R3_FINALIZER_LAUNCHD_LABEL_V2);
    let output = Command::new("/bin/launchctl")
        .arg("print")
        .arg(&target)
        .current_dir("/")
        .env_clear()
        .stdin(Stdio::null())
        .output()?;
    let exit_status = output
        .status
        .code()
        .context("launchctl print was signaled")?;
    let stdout_text = std::str::from_utf8(&output.stdout)?;
    if exit_status != 0
        || !output.stderr.is_empty()
        || !stdout_text.contains(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)
        || !stdout_text.contains(MAC_R3_FINALIZER_PLIST_PATH_V2)
        || !stdout_text.contains(MAC_R3_FINALIZER_PATH_V2)
    {
        bail!("launchctl print does not prove the exact loaded finalizer service")
    }
    let endpoint = lstat(Path::new(MAC_R3_FINALIZER_ENDPOINT_V2))?
        .context("loaded finalizer endpoint is absent")?;
    let endpoint_is_socket = (endpoint.st_mode & libc::S_IFMT) == libc::S_IFSOCK;
    if !endpoint_is_socket
        || endpoint.st_uid != 0
        || endpoint.st_gid != 20
        || endpoint.st_nlink != 1
        || (endpoint.st_mode & 0o7777) != 0o660
    {
        bail!("loaded finalizer endpoint owner/group/mode/type identity changed")
    }
    let observation = FinalizerServiceObservationV2 {
        schema_owner: "substrate.r3-macos-disposable-finalizer-bootstrap".to_owned(),
        schema_version: 2,
        experiment_id: EXPERIMENT_ID_V2.to_owned(),
        prepared_sha256: document_sha256_v2(prepared)?,
        bootstrap_result_sha256: document_sha256_v2(result)?,
        launchd_label: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_owned(),
        print_exit_status: exit_status,
        print_stdout: bounded_raw_stream_evidence(&output.stdout)?,
        print_stderr: bounded_raw_stream_evidence(&output.stderr)?,
        endpoint_path: MAC_R3_FINALIZER_ENDPOINT_V2.to_owned(),
        endpoint_owner_uid: endpoint.st_uid,
        endpoint_group_gid: endpoint.st_gid,
        endpoint_mode: u32::from(endpoint.st_mode & 0o7777),
        endpoint_device: endpoint.st_dev as u64,
        endpoint_inode: endpoint.st_ino,
        endpoint_link_count: endpoint.st_nlink as u64,
        endpoint_is_socket,
        service_loaded_and_endpoint_exact: true,
    };
    validate_service_observation(&observation, prepared, result)?;
    Ok(observation)
}

fn validate_service_observation(
    observation: &FinalizerServiceObservationV2,
    prepared: &FinalizerBootstrapPreparedV2,
    result: &FinalizerBootstrapResultV2,
) -> Result<()> {
    let stdout = observation.print_stdout.validate()?;
    let stderr = observation.print_stderr.validate()?;
    let stdout_text = std::str::from_utf8(&stdout)?;
    if observation.schema_owner != "substrate.r3-macos-disposable-finalizer-bootstrap"
        || observation.schema_version != 2
        || observation.experiment_id != EXPERIMENT_ID_V2
        || observation.prepared_sha256 != document_sha256_v2(prepared)?
        || observation.bootstrap_result_sha256 != document_sha256_v2(result)?
        || observation.launchd_label != MAC_R3_FINALIZER_LAUNCHD_LABEL_V2
        || observation.print_exit_status != 0
        || !stderr.is_empty()
        || !stdout_text.contains(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)
        || !stdout_text.contains(MAC_R3_FINALIZER_PLIST_PATH_V2)
        || !stdout_text.contains(MAC_R3_FINALIZER_PATH_V2)
        || observation.endpoint_path != MAC_R3_FINALIZER_ENDPOINT_V2
        || observation.endpoint_owner_uid != 0
        || observation.endpoint_group_gid != 20
        || observation.endpoint_mode != 0o660
        || observation.endpoint_link_count != 1
        || !observation.endpoint_is_socket
        || !observation.service_loaded_and_endpoint_exact
    {
        bail!("finalizer service observation is not the exact loaded service and endpoint")
    }
    Ok(())
}

fn bounded_raw_stream_evidence(bytes: &[u8]) -> Result<BoundedRawStreamEvidenceV2> {
    if bytes.len() > 256 * 1024 {
        bail!("launchctl raw stream exceeds the closed evidence bound")
    }
    let evidence = BoundedRawStreamEvidenceV2 {
        raw_base64url: URL_SAFE_NO_PAD.encode(bytes),
        raw_sha256: sha256_hex_v2(bytes),
        raw_byte_length: u64::try_from(bytes.len())?,
    };
    evidence.validate()?;
    Ok(evidence)
}

fn require_exact_launchd_label_absent(label: &str) -> Result<String> {
    if label != MAC_R3_FINALIZER_LAUNCHD_LABEL_V2 {
        bail!("launchd absence probe label is not the one compiled finalizer label")
    }
    let target = format!("system/{label}");
    let output = Command::new("/bin/launchctl")
        .arg("print")
        .arg(&target)
        .current_dir("/")
        .env_clear()
        .stdin(Stdio::null())
        .output()?;
    let code = output.status.code();
    if code != Some(113)
        || !output.stdout.is_empty()
        || output.stderr != exact_launchctl_service_not_found_stderr(label).as_bytes()
    {
        bail!("launchctl did not return the frozen service-not-found classification")
    }
    document_sha256_v2(&(
        "/bin/launchctl",
        "print",
        target,
        code,
        sha256_hex_v2(&output.stdout),
        sha256_hex_v2(&output.stderr),
    ))
}

fn require_exact_process_path_absent(path: &Path) -> Result<String> {
    let capacity = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if capacity <= 0 {
        return Err(std::io::Error::last_os_error()).context("size exact process list");
    }
    let capacity = usize::try_from(capacity)?
        .checked_add(128)
        .context("process list capacity overflow")?;
    let mut pids = vec![0_i32; capacity];
    let bytes = i32::try_from(
        pids.len()
            .checked_mul(size_of::<i32>())
            .context("process list bytes overflow")?,
    )?;
    let count = unsafe { proc_listallpids(pids.as_mut_ptr().cast(), bytes) };
    if count < 0 || usize::try_from(count)? >= pids.len() {
        return Err(std::io::Error::last_os_error()).context("read exact process list");
    }
    pids.truncate(usize::try_from(count)?);
    pids.sort_unstable();
    pids.dedup();
    let mut measured = 0_u64;
    for pid in pids.into_iter().filter(|pid| *pid > 0) {
        let mut bytes = vec![0_u8; 4096];
        let length = unsafe { proc_pidpath(pid, bytes.as_mut_ptr().cast(), bytes.len() as u32) };
        if length <= 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ESRCH)
                || error.raw_os_error() == Some(libc::ENOENT)
            {
                continue;
            }
            return Err(error).context("measure process path for exact absence");
        }
        bytes.truncate(usize::try_from(length)?);
        measured = measured
            .checked_add(1)
            .context("process observation overflow")?;
        if Path::new(std::ffi::OsStr::from_bytes(&bytes)) == path {
            bail!("frozen pre-effect executable has a live process")
        }
    }
    document_sha256_v2(&("proc_listallpids", path, measured))
}

fn publish_global_pre_effect_packet(packet: &GlobalPreEffectPacketV2, bytes: &[u8]) -> Result<()> {
    write_global_external_root_output_bounded(
        PublisherArtifactV2::GlobalPreEffectPacket,
        bytes,
        GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2,
    )?;
    let path = global_external_exchange_path_v2(PublisherArtifactV2::GlobalPreEffectPacket)?;
    let reopened = stable_read_file_bounded(
        &path,
        0,
        Some(libc::S_IFREG | 0o444),
        GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2,
    )?;
    if reopened != bytes
        || parse_runner_private_canonical::<GlobalPreEffectPacketV2>(
            GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
            &reopened,
        )? != *packet
        || document_sha256_v2(packet)? != sha256_hex_v2(&reopened)
    {
        bail!("global pre-effect packet failed exact durable reopen/hash validation")
    }
    Ok(())
}

fn ensure_root_directory(path: &Path) -> Result<()> {
    if !path_present(path)? {
        std::fs::create_dir(path)?;
        std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o700))?;
        File::open(path.parent().context("fixed root directory lacks parent")?)?.sync_all()?;
    }
    require_directory(path, 0, 0, ROOT_DIRECTORY_MODE)
}

fn install_alternate_coordinator_copy(
    coordinator: &substrate_common::macos_retirement_v2::ExecutableIdentityV2,
) -> Result<()> {
    let destination = Path::new(ALTERNATE_COORDINATOR_PATH_V2);
    if path_present(destination)? {
        return Ok(());
    }
    let bytes = stable_read_file(Path::new(MAC_R3_COORDINATOR_PATH_V2), 0, None)?;
    if sha256_hex_v2(&bytes) != coordinator.executable_sha256 {
        bail!("coordinator changed before alternate-path copy")
    }
    write_fixed_file(destination, &bytes, 0o555, 0, 0, libc::S_IFDIR | 0o755)
}

fn install_packet(path: &Path, bytes: &[u8]) -> Result<()> {
    write_fixed_file(path, bytes, 0o444, 0, 0, libc::S_IFDIR | 0o755)
}

fn write_external_root_output(
    repetition: RepetitionV2,
    artifact: PublisherArtifactV2,
    bytes: &[u8],
) -> Result<()> {
    let path = external_exchange_path_v2(repetition, artifact);
    require_directory(
        path.parent().unwrap(),
        DISPOSABLE_HARNESS_UID_V2,
        0,
        libc::S_IFDIR | 0o700,
    )?;
    write_fixed_file(
        &path,
        bytes,
        0o444,
        DISPOSABLE_HARNESS_UID_V2,
        0,
        libc::S_IFDIR | 0o700,
    )
}

fn write_external_harness_input(
    repetition: RepetitionV2,
    artifact: PublisherArtifactV2,
    bytes: &[u8],
) -> Result<()> {
    let path = external_exchange_path_v2(repetition, artifact);
    let parent = path
        .parent()
        .context("emergency marker external path lacks parent")?;
    require_directory(parent, DISPOSABLE_HARNESS_UID_V2, 0, libc::S_IFDIR | 0o700)?;
    publish_exact_file(
        &path,
        bytes,
        PublishIdentityV2 {
            owner_uid: DISPOSABLE_HARNESS_UID_V2,
            owner_gid: 0,
            permissions: 0o400,
            parent_uid: DISPOSABLE_HARNESS_UID_V2,
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: MAX_CHILD_OUTPUT,
        },
        PublishMode::Immutable,
    )
}

fn write_global_external_root_output(artifact: PublisherArtifactV2, bytes: &[u8]) -> Result<()> {
    write_global_external_root_output_bounded(artifact, bytes, MAX_CHILD_OUTPUT)
}

fn write_global_external_root_output_bounded(
    artifact: PublisherArtifactV2,
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<()> {
    let path = global_external_exchange_path_v2(artifact)?;
    publish_exact_file(
        &path,
        bytes,
        PublishIdentityV2 {
            owner_uid: 0,
            owner_gid: 0,
            permissions: 0o444,
            parent_uid: DISPOSABLE_HARNESS_UID_V2,
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes,
        },
        PublishMode::Immutable,
    )
}

fn write_runner_receipt(name: &str, bytes: &[u8]) -> Result<()> {
    if name.contains(['/', '\0', '\n', '\r']) {
        bail!("runner receipt component is invalid")
    }
    write_fixed_file_bounded(
        &Path::new(RUNNER_ROOT).join(name),
        bytes,
        0o600,
        0,
        0,
        libc::S_IFDIR | 0o700,
        runner_private_archive_entry_max_bytes_v2(name),
    )
}

fn read_runner_private_optional<T: DeserializeOwned + Serialize>(name: &str) -> Result<Option<T>> {
    if name.contains(['/', '\0', '\n', '\r']) {
        bail!("runner private receipt component is invalid")
    }
    read_runner_private_bytes_optional(name)?
        .map(|bytes| parse_canonical_v2(&bytes))
        .transpose()
}

fn parse_runner_private_canonical<T: DeserializeOwned + Serialize>(
    name: &str,
    bytes: &[u8],
) -> Result<T> {
    parse_canonical_bounded_v2(bytes, runner_private_archive_entry_max_bytes_v2(name))
}

fn read_runner_global_pre_effect_packet_optional() -> Result<Option<GlobalPreEffectPacketV2>> {
    read_runner_private_bytes_optional(GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2)?
        .map(|bytes| {
            parse_runner_private_canonical(GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2, &bytes)
        })
        .transpose()
}

fn read_runner_global_pre_effect_packet_cleanup_recovery_optional(
) -> Result<Option<GlobalPreEffectPacketV2>> {
    read_runner_private_cleanup_recovery_bytes_optional_at(
        Path::new(RUNNER_ROOT),
        GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
        PublishIdentityV2 {
            owner_uid: 0,
            owner_gid: 0,
            permissions: 0o600,
            parent_uid: 0,
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: runner_private_archive_entry_max_bytes_v2(
                GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
            ),
        },
    )?
    .map(|bytes| {
        parse_runner_private_canonical(GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2, &bytes)
    })
    .transpose()
}

fn read_runner_private_cleanup_recovery_optional<T: DeserializeOwned + Serialize>(
    name: &str,
) -> Result<Option<T>> {
    read_runner_private_cleanup_recovery_bytes_optional_at(
        Path::new(RUNNER_ROOT),
        name,
        PublishIdentityV2 {
            owner_uid: 0,
            owner_gid: 0,
            permissions: 0o600,
            parent_uid: 0,
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: runner_private_archive_entry_max_bytes_v2(name),
        },
    )?
    .map(|bytes| parse_canonical_v2(&bytes))
    .transpose()
}

fn read_runner_private_cleanup_recovery_bytes_optional_at(
    runner_root: &Path,
    name: &str,
    identity: PublishIdentityV2,
) -> Result<Option<Vec<u8>>> {
    if name.contains(['/', '\0', '\n', '\r']) {
        bail!("runner private cleanup-recovery component is invalid")
    }
    if lstat(runner_root)?.is_none() {
        return Ok(None);
    }
    read_exact_published_file(&runner_root.join(name), identity)
}

fn read_runner_private_bytes_optional(name: &str) -> Result<Option<Vec<u8>>> {
    if name.contains(['/', '\0', '\n', '\r']) {
        bail!("runner private receipt component is invalid")
    }
    read_exact_published_file(
        &Path::new(RUNNER_ROOT).join(name),
        PublishIdentityV2 {
            owner_uid: 0,
            owner_gid: 0,
            permissions: 0o600,
            parent_uid: 0,
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: runner_private_archive_entry_max_bytes_v2(name),
        },
    )
}

fn write_fixed_file(
    path: &Path,
    bytes: &[u8],
    mode: libc::mode_t,
    parent_uid: u32,
    parent_gid: u32,
    parent_mode: libc::mode_t,
) -> Result<()> {
    write_fixed_file_bounded(
        path,
        bytes,
        mode,
        parent_uid,
        parent_gid,
        parent_mode,
        MAX_CHILD_OUTPUT,
    )
}

fn write_fixed_file_bounded(
    path: &Path,
    bytes: &[u8],
    mode: libc::mode_t,
    parent_uid: u32,
    parent_gid: u32,
    parent_mode: libc::mode_t,
    maximum_bytes: usize,
) -> Result<()> {
    publish_exact_file(
        path,
        bytes,
        PublishIdentityV2 {
            owner_uid: 0,
            owner_gid: 0,
            permissions: mode,
            parent_uid,
            parent_gid,
            parent_mode,
            maximum_bytes,
        },
        PublishMode::Immutable,
    )
}

fn remove_exact_installed_file(
    path: &Path,
    expected: Option<&[u8]>,
    expected_physical_identity_sha256: Option<&str>,
) -> Result<()> {
    remove_exact_owned_file_bounded(
        path,
        expected,
        expected_physical_identity_sha256,
        0,
        None,
        usize::try_from(INSTALLED_ARTIFACT_MAX_BYTES)
            .context("installed-artifact read bound exceeds usize")?,
    )
}

fn remove_exact_owned_file(
    path: &Path,
    expected: Option<&[u8]>,
    expected_physical_identity_sha256: Option<&str>,
    expected_uid: u32,
    expected_mode: Option<libc::mode_t>,
) -> Result<()> {
    remove_exact_owned_file_bounded(
        path,
        expected,
        expected_physical_identity_sha256,
        expected_uid,
        expected_mode,
        MAX_CHILD_OUTPUT,
    )
}

fn remove_exact_owned_file_bounded(
    path: &Path,
    expected: Option<&[u8]>,
    expected_physical_identity_sha256: Option<&str>,
    expected_uid: u32,
    expected_mode: Option<libc::mode_t>,
    maximum_bytes: usize,
) -> Result<()> {
    let Some(bytes) =
        stable_read_file_optional_bounded(path, expected_uid, expected_mode, maximum_bytes)?
    else {
        return Ok(());
    };
    if expected.is_some_and(|expected| expected != bytes) {
        bail!("runner cleanup file content changed before removal")
    }
    let parent = path.parent().context("runner cleanup path lacks parent")?;
    let leaf = path.file_name().context("runner cleanup path lacks leaf")?;
    let parent_encoded = CString::new(parent.as_os_str().as_bytes())?;
    let leaf_encoded = CString::new(leaf.as_bytes())?;
    // SAFETY: exact compiled parent and terminal no-follow directory open.
    let raw_parent = unsafe {
        libc::open(
            parent_encoded.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw_parent < 0 {
        return Err(std::io::Error::last_os_error()).context("open exact cleanup parent");
    }
    // SAFETY: successful open transferred ownership.
    let parent_fd = unsafe { OwnedFd::from_raw_fd(raw_parent) };
    let before = fstatat_nofollow(parent_fd.as_raw_fd(), &leaf_encoded)?
        .context("exact cleanup leaf disappeared after stable read")?;
    if before.st_uid != expected_uid
        || before.st_gid != 0
        || before.st_nlink != 1
        || (before.st_mode & libc::S_IFMT) != libc::S_IFREG
        || expected_mode.is_some_and(|mode| (before.st_mode & (libc::S_IFMT | 0o7777)) != mode)
    {
        bail!("exact cleanup leaf owner/type/link identity changed")
    }
    if let Some(expected) = expected_physical_identity_sha256 {
        if runner_file_physical_identity_sha256(&before)? != expected {
            bail!("exact cleanup leaf physical identity changed")
        }
    }
    let after = fstatat_nofollow(parent_fd.as_raw_fd(), &leaf_encoded)?
        .context("exact cleanup leaf disappeared before unlink")?;
    if !same_stat(&before, &after) {
        bail!("exact cleanup leaf changed before unlink")
    }
    // SAFETY: held exact parent and one fixed previously measured leaf.
    if unsafe { libc::unlinkat(parent_fd.as_raw_fd(), leaf_encoded.as_ptr(), 0) } != 0 {
        return Err(std::io::Error::last_os_error()).context("remove exact runner artifact");
    }
    File::from(parent_fd).sync_all()?;
    if path_present(path)? {
        bail!("exact runner artifact remains after removal")
    }
    Ok(())
}

fn fstatat_nofollow(parent_fd: i32, leaf: &CString) -> Result<Option<libc::stat>> {
    let mut value = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: held parent descriptor, fixed leaf, and writable output.
    if unsafe {
        libc::fstatat(
            parent_fd,
            leaf.as_ptr(),
            value.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error).context("fstatat exact cleanup leaf");
    }
    // SAFETY: successful fstatat initialized output.
    Ok(Some(unsafe { value.assume_init() }))
}

fn runner_file_physical_identity_sha256(value: &libc::stat) -> Result<String> {
    document_sha256_v2(&RunnerFilePhysicalIdentityV2 {
        device: value.st_dev as u64,
        inode: value.st_ino,
        owner_uid: value.st_uid,
        owner_gid: value.st_gid,
        mode: u32::from(value.st_mode),
        link_count: u64::from(value.st_nlink),
        size: u64::try_from(value.st_size).context("cleanup file size is negative")?,
        modified_seconds: value.st_mtime,
        modified_nanoseconds: value.st_mtime_nsec,
    })
}

fn read_external<T: DeserializeOwned + Serialize>(
    repetition: RepetitionV2,
    artifact: PublisherArtifactV2,
) -> Result<T> {
    let bytes = stable_read_file(
        &external_exchange_path_v2(repetition, artifact),
        DISPOSABLE_HARNESS_UID_V2,
        Some(EXTERNAL_INPUT_MODE),
    )?;
    parse_canonical_v2(&bytes)
}

fn read_external_optional<T: DeserializeOwned + Serialize>(
    repetition: RepetitionV2,
    artifact: PublisherArtifactV2,
) -> Result<Option<T>> {
    stable_read_file_optional(
        &external_exchange_path_v2(repetition, artifact),
        DISPOSABLE_HARNESS_UID_V2,
        Some(EXTERNAL_INPUT_MODE),
    )?
    .map(|bytes| parse_canonical_v2(&bytes))
    .transpose()
}

fn read_external_root_optional<T: DeserializeOwned + Serialize>(
    repetition: RepetitionV2,
    artifact: PublisherArtifactV2,
) -> Result<Option<T>> {
    stable_read_file_optional(
        &external_exchange_path_v2(repetition, artifact),
        0,
        Some(libc::S_IFREG | 0o444),
    )?
    .map(|bytes| parse_canonical_v2(&bytes))
    .transpose()
}

fn stable_read_file(path: &Path, uid: u32, mode: Option<libc::mode_t>) -> Result<Vec<u8>> {
    stable_read_file_bounded(path, uid, mode, MAX_CHILD_OUTPUT)
}

fn stable_read_file_bounded(
    path: &Path,
    uid: u32,
    mode: Option<libc::mode_t>,
    maximum_bytes: usize,
) -> Result<Vec<u8>> {
    stable_read_file_optional_bounded(path, uid, mode, maximum_bytes)?
        .context("exact runner input is absent")
}

fn stable_read_file_optional(
    path: &Path,
    uid: u32,
    mode: Option<libc::mode_t>,
) -> Result<Option<Vec<u8>>> {
    stable_read_file_optional_bounded(path, uid, mode, MAX_CHILD_OUTPUT)
}

fn stable_read_file_optional_bounded(
    path: &Path,
    uid: u32,
    mode: Option<libc::mode_t>,
    maximum_bytes: usize,
) -> Result<Option<Vec<u8>>> {
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: exact path and terminal no-follow.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error).context("open exact runner input");
    }
    // SAFETY: successful open transferred ownership.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let before = fstat(descriptor.as_raw_fd())?;
    if before.st_uid != uid
        || before.st_gid != 0
        || before.st_nlink != 1
        || (before.st_mode & libc::S_IFMT) != libc::S_IFREG
        || mode.is_some_and(|mode| (before.st_mode & (libc::S_IFMT | 0o7777)) != mode)
        || before.st_size < 0
        || before.st_size as usize > maximum_bytes
    {
        bail!("exact runner input owner/type/mode/link/size changed")
    }
    let mut bytes = Vec::with_capacity(before.st_size as usize);
    File::from(descriptor).read_to_end(&mut bytes)?;
    let after = lstat(path)?.context("runner input disappeared during read")?;
    if !same_stat(&before, &after) || after.st_size != bytes.len() as libc::off_t {
        bail!("runner input changed during stable read")
    }
    Ok(Some(bytes))
}

fn require_directory(path: &Path, uid: u32, gid: u32, mode: libc::mode_t) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || metadata.uid() != uid
        || metadata.gid() != gid
        || metadata.mode() != u32::from(mode)
        || !metadata.is_dir()
    {
        bail!("runner directory owner/type/mode changed")
    }
    Ok(())
}

fn open_exact_directory(path: &Path, uid: u32, gid: u32, mode: libc::mode_t) -> Result<OwnedFd> {
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: exact fixed directory, opened with terminal no-follow.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error()).context("open exact runner directory");
    }
    // SAFETY: successful open transfers descriptor ownership.
    let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
    let held = fstat(descriptor.as_raw_fd())?;
    let pathname = lstat(path)?.context("exact runner directory disappeared")?;
    if !same_stat(&held, &pathname)
        || held.st_uid != uid
        || held.st_gid != gid
        || (held.st_mode & (libc::S_IFMT | 0o7777)) != mode
    {
        bail!("exact runner directory identity, owner, group, or mode changed")
    }
    Ok(descriptor)
}

fn path_present(path: &Path) -> Result<bool> {
    Ok(lstat(path)?.is_some())
}

fn lstat(path: &Path) -> Result<Option<libc::stat>> {
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    let mut value = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: exact path and writable output.
    if unsafe { libc::lstat(encoded.as_ptr(), value.as_mut_ptr()) } != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error).context("lstat runner path");
    }
    // SAFETY: successful lstat initialized output.
    Ok(Some(unsafe { value.assume_init() }))
}

fn fstat(fd: i32) -> Result<libc::stat> {
    let mut value = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: live descriptor and writable output.
    if unsafe { libc::fstat(fd, value.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("fstat runner object");
    }
    // SAFETY: successful fstat initialized output.
    Ok(unsafe { value.assume_init() })
}

fn same_stat(left: &libc::stat, right: &libc::stat) -> bool {
    left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_uid == right.st_uid
        && left.st_gid == right.st_gid
        && left.st_mode == right.st_mode
        && left.st_nlink == right.st_nlink
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
}

fn canonical_account(uid: u32) -> Result<(String, u32)> {
    let mut record = MaybeUninit::<libc::passwd>::uninit();
    let mut result = std::ptr::null_mut();
    let mut buffer = vec![0_u8; 1024 * 1024];
    // SAFETY: all output storage is live for the exact buffer length.
    let status = unsafe {
        libc::getpwuid_r(
            uid,
            record.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 || result.is_null() {
        bail!("resolve exact runner account failed with errno {status}")
    }
    // SAFETY: successful getpwuid_r initialized the record into live buffer.
    let record = unsafe { record.assume_init() };
    if record.pw_uid != uid || record.pw_name.is_null() {
        bail!("runner account record changed")
    }
    // SAFETY: pw_name is NUL-terminated inside live buffer.
    let name = unsafe { std::ffi::CStr::from_ptr(record.pw_name) }
        .to_str()?
        .to_owned();
    if uid == DISPOSABLE_HARNESS_UID_V2 && name != DISPOSABLE_HARNESS_ACCOUNT_V2 {
        bail!("UID501 canonical account differs from frozen harness account")
    }
    Ok((name, record.pw_gid))
}

fn canonical_group(gid: u32) -> Result<String> {
    let mut record = MaybeUninit::<libc::group>::uninit();
    let mut result = std::ptr::null_mut();
    let mut buffer = vec![0_u8; 1024 * 1024];
    // SAFETY: all output storage remains live for the exact lookup buffer length.
    let status = unsafe {
        libc::getgrgid_r(
            gid,
            record.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 || result.is_null() {
        bail!("resolve exact runner group failed with errno {status}")
    }
    // SAFETY: successful getgrgid_r initialized the record into the live buffer.
    let record = unsafe { record.assume_init() };
    if record.gr_gid != gid || record.gr_name.is_null() {
        bail!("runner group record changed")
    }
    // SAFETY: gr_name is NUL-terminated inside the live lookup buffer.
    Ok(unsafe { std::ffi::CStr::from_ptr(record.gr_name) }
        .to_str()?
        .to_owned())
}

#[derive(Debug)]
struct ProcessInfoV2 {
    pid: u32,
    uid: u32,
    gid: u32,
    process_group_id: i32,
    status: u32,
    start_seconds: u64,
    start_microseconds: u64,
}

#[repr(C)]
struct ProcBsdInfo {
    pbi_flags: u32,
    pbi_status: u32,
    pbi_xstatus: u32,
    pbi_pid: u32,
    pbi_ppid: u32,
    pbi_uid: libc::uid_t,
    pbi_gid: libc::gid_t,
    pbi_ruid: libc::uid_t,
    pbi_rgid: libc::gid_t,
    pbi_svuid: libc::uid_t,
    pbi_svgid: libc::gid_t,
    rfu_1: u32,
    pbi_comm: [libc::c_char; 16],
    pbi_name: [libc::c_char; 32],
    pbi_nfiles: u32,
    pbi_pgid: u32,
    pbi_pjobc: u32,
    e_tdev: u32,
    e_tpgid: u32,
    pbi_nice: i32,
    pbi_start_tvsec: u64,
    pbi_start_tvusec: u64,
}

fn process_info(pid: i32) -> Result<ProcessInfoV2> {
    let mut value = MaybeUninit::<ProcBsdInfo>::zeroed();
    // SAFETY: writable output matches installed SDK PROC_PIDTBSDINFO layout.
    let returned = unsafe {
        proc_pidinfo(
            pid,
            3,
            0,
            value.as_mut_ptr().cast(),
            size_of::<ProcBsdInfo>() as i32,
        )
    };
    if returned <= 0 {
        return Err(std::io::Error::last_os_error()).context("read exact peer process info");
    }
    if returned as usize != size_of::<ProcBsdInfo>() {
        bail!("peer process info was truncated")
    }
    // SAFETY: full structure was initialized.
    let value = unsafe { value.assume_init() };
    Ok(ProcessInfoV2 {
        pid: value.pbi_pid,
        uid: value.pbi_uid,
        gid: value.pbi_gid,
        process_group_id: i32::try_from(value.pbi_pgid)
            .context("process-group identifier exceeds i32")?,
        status: value.pbi_status,
        start_seconds: value.pbi_start_tvsec,
        start_microseconds: value.pbi_start_tvusec,
    })
}

fn pid_path(pid: i32) -> Result<PathBuf> {
    let mut bytes = vec![0_u8; 4096];
    // SAFETY: writable output buffer is exact length.
    let length = unsafe { proc_pidpath(pid, bytes.as_mut_ptr().cast(), bytes.len() as u32) };
    if length <= 0 || length as usize >= bytes.len() {
        return Err(std::io::Error::last_os_error()).context("resolve peer process path");
    }
    bytes.truncate(length as usize);
    Ok(PathBuf::from(std::ffi::OsString::from_vec(bytes)))
}

fn state_repetition(state: MarkerState) -> Option<crate::FixedRepetitionV2> {
    let ordinal = MarkerState::all()
        .iter()
        .position(|candidate| *candidate == state)?;
    if ordinal < 10 {
        Some(crate::FixedRepetitionV2::First)
    } else if ordinal < 20 {
        Some(crate::FixedRepetitionV2::Second)
    } else {
        None
    }
}

fn creator_arm_ordinal(state: MarkerState) -> Result<u8> {
    let index = MarkerState::all()
        .iter()
        .position(|candidate| *candidate == state)
        .context("creator marker is outside its closed sequence")?;
    if state == MarkerState::Complete || index % 2 != 0 && index >= 20 {
        bail!("creator marker has no invocable arm ordinal")
    }
    u8::try_from(index / 2 + 1).context("creator arm ordinal exceeds u8")
}

fn marker_text(state: MarkerState) -> String {
    std::str::from_utf8(state.marker())
        .expect("compiled marker is UTF-8")
        .trim_end()
        .to_owned()
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[link(name = "proc")]
unsafe extern "C" {
    fn proc_listallpids(buffer: *mut libc::c_void, buffer_size: i32) -> i32;
    fn proc_pidinfo(
        pid: i32,
        flavor: i32,
        arg: u64,
        buffer: *mut libc::c_void,
        buffer_size: i32,
    ) -> i32;
    fn proc_pidpath(pid: i32, buffer: *mut libc::c_void, buffer_size: u32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    struct InstalledArtifactFixture {
        root: PathBuf,
        path: PathBuf,
    }

    impl InstalledArtifactFixture {
        fn create(name: &str, bytes: &[u8]) -> Self {
            let root = std::env::temp_dir().join(format!(
                "substrate-r3-installed-artifact-{}-{name}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir(&root).expect("create installed-artifact test root");
            let path = root.join("artifact");
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
                .expect("create installed-artifact test file");
            file.write_all(bytes)
                .expect("write installed-artifact fixture bytes");
            file.sync_all()
                .expect("sync installed-artifact fixture bytes");
            Self { root, path }
        }

        fn identity(&self) -> RootInstallPhysicalIdentityV2 {
            let observed = lstat(&self.path)
                .expect("lstat installed-artifact fixture")
                .expect("installed-artifact fixture is present");
            root_install_physical_identity(
                self.path.to_str().expect("fixture path is UTF-8"),
                &observed,
            )
            .expect("build installed-artifact fixture identity")
        }
    }

    impl Drop for InstalledArtifactFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn installed_artifact_read_uses_exact_manifest_size_not_child_document_bound() {
        // Match the observed frozen experiment runner, proving that installed
        // executable reads do not reuse the 1 MiB child-document ceiling.
        let bytes = vec![0x5a; 7_212_192];
        let fixture = InstalledArtifactFixture::create("multi-megabyte", &bytes);
        let expected = fixture.identity();
        assert!(expected.size > u64::try_from(MAX_CHILD_OUTPUT).unwrap());
        reattest_root_install_identity(&expected, Some(&sha256_hex_v2(&bytes)))
            .expect("multi-megabyte installed artifact remains within its distinct ceiling");
    }

    #[test]
    fn alternate_coordinator_cleanup_uses_installed_artifact_bound_only() {
        let bytes = vec![0x43; 2_398_672];
        let fixture = InstalledArtifactFixture::create("alternate-coordinator", &bytes);
        let expected = fixture.identity();
        assert!(expected.size > u64::try_from(MAX_CHILD_OUTPUT).unwrap());
        reattest_root_install_identity(&expected, Some(&sha256_hex_v2(&bytes)))
            .expect("manifest-bound alternate coordinator fits the installed-artifact ceiling");

        let source = include_str!("runner.rs");
        let installed_cleanup = source
            .split("fn remove_exact_installed_file(")
            .nth(1)
            .expect("installed cleanup function remains present")
            .split("\nfn ")
            .next()
            .expect("installed cleanup function has a closed body");
        assert!(installed_cleanup.contains("INSTALLED_ARTIFACT_MAX_BYTES"));
        assert!(installed_cleanup.contains("remove_exact_owned_file_bounded("));
        let document_read = source
            .split("fn stable_read_file_optional(")
            .nth(1)
            .expect("document read function remains present")
            .split("\nfn ")
            .next()
            .expect("document read function has a closed body");
        assert!(document_read.contains("MAX_CHILD_OUTPUT"));
        assert!(!document_read.contains("INSTALLED_ARTIFACT_MAX_BYTES"));

        std::fs::hard_link(&fixture.path, fixture.root.join("identity-drift"))
            .expect("create installed-artifact link-count drift");
        assert!(reattest_root_install_identity(&expected, Some(&sha256_hex_v2(&bytes))).is_err());
    }

    #[test]
    fn installed_artifact_read_rejects_size_hash_and_ceiling_divergence() {
        let original = vec![0x41; 64 * 1024];
        let fixture = InstalledArtifactFixture::create("divergence", &original);
        let original_identity = fixture.identity();

        let mut mismatched_manifest_size = original_identity.clone();
        mismatched_manifest_size.size += 1;
        assert!(reattest_root_install_identity(
            &mismatched_manifest_size,
            Some(&sha256_hex_v2(&original)),
        )
        .is_err());

        std::fs::write(&fixture.path, &original[..original.len() - 1])
            .expect("truncate installed-artifact fixture");
        assert!(reattest_root_install_identity(
            &original_identity,
            Some(&sha256_hex_v2(&original)),
        )
        .is_err());

        std::fs::write(&fixture.path, vec![0x41; original.len() + 1])
            .expect("grow installed-artifact fixture");
        assert!(reattest_root_install_identity(
            &original_identity,
            Some(&sha256_hex_v2(&original)),
        )
        .is_err());

        std::fs::write(&fixture.path, vec![0x42; original.len()])
            .expect("substitute installed-artifact fixture bytes");
        let substituted_identity = fixture.identity();
        assert!(reattest_root_install_identity(
            &substituted_identity,
            Some(&sha256_hex_v2(&original)),
        )
        .is_err());

        let ceiling = InstalledArtifactFixture::create("ceiling", &[]);
        OpenOptions::new()
            .write(true)
            .open(&ceiling.path)
            .expect("open sparse ceiling fixture")
            .set_len(INSTALLED_ARTIFACT_MAX_BYTES + 1)
            .expect("grow sparse ceiling fixture");
        let ceiling_identity = ceiling.identity();
        assert!(
            reattest_root_install_identity(&ceiling_identity, Some(&sha256_hex_v2(&[]))).is_err()
        );
    }

    #[test]
    fn root_runner_surface_is_closed_and_distinct_from_acl_principals() {
        assert_ne!(EXPERIMENT_RUNNER_EXECUTABLE_PATH, CREATOR_EXECUTABLE_PATH);
        assert_ne!(
            EXPERIMENT_RUNNER_EXECUTABLE_PATH,
            DISPOSABLE_PUBLISHER_EXECUTABLE_PATH
        );
        assert_ne!(EXPERIMENT_RUNNER_EXECUTABLE_PATH, MAC_R3_FINALIZER_PATH_V2);
        let source = include_str!("runner.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("std::env::args_os().count() != 1"));
        assert!(production.contains("env_clear()"));
        assert!(production.contains("Stdio::null()"));
        assert!(!production.contains("std::env::var("));
    }

    #[test]
    fn wrong_identity_typed_receipt_accepts_only_process_denied_sign_v4() {
        let current = serde_json::json!({
            "schema": "substrate.r3-macos-signer-acl.wrong-identity-receipt.v4",
            "repetition": "first",
            "creator_scope_id": CREATOR_REPETITION_SCOPE_1,
            "executable_path": WRONG_IDENTITY_EXECUTABLE_PATH,
            "marker_path": MARKER_PATH,
            "process_interaction_disable_raw_os_status": 0,
            "precommitted_expected_sign_raw_cferror_code": -25_293,
            "tag_scoped_private_key_sign_raw_cferror_code": -25_293,
        });
        validate_creator_typed_receipt(
            &current,
            MarkerState::FirstWrongPrepared,
            MarkerState::FirstFreshDeletePrepared,
            WRONG_IDENTITY_EXECUTABLE_PATH,
        )
        .expect("accept exact process-denied sign receipt");

        let historical_lookup = serde_json::json!({
            "schema": "substrate.r3-macos-signer-acl.wrong-identity-receipt.v3",
            "repetition": "first",
            "creator_scope_id": CREATOR_REPETITION_SCOPE_1,
            "executable_path": WRONG_IDENTITY_EXECUTABLE_PATH,
            "marker_path": MARKER_PATH,
            "process_interaction_disable_raw_os_status": 0,
            "precommitted_expected_lookup_raw_os_status": -25_308,
            "tag_scoped_private_key_lookup_raw_os_status": -25_308,
        });
        assert!(validate_creator_typed_receipt(
            &historical_lookup,
            MarkerState::FirstWrongPrepared,
            MarkerState::FirstFreshDeletePrepared,
            WRONG_IDENTITY_EXECUTABLE_PATH,
        )
        .is_err());

        for rejected in [-25_308, 0, -50] {
            let mut wrong_result = current.clone();
            wrong_result.as_object_mut().unwrap().insert(
                "precommitted_expected_sign_raw_cferror_code".to_owned(),
                serde_json::json!(rejected),
            );
            wrong_result.as_object_mut().unwrap().insert(
                "tag_scoped_private_key_sign_raw_cferror_code".to_owned(),
                serde_json::json!(rejected),
            );
            assert!(validate_creator_typed_receipt(
                &wrong_result,
                MarkerState::FirstWrongPrepared,
                MarkerState::FirstFreshDeletePrepared,
                WRONG_IDENTITY_EXECUTABLE_PATH,
            )
            .is_err());
        }

        let runner_source = include_str!("runner.rs");
        let build_start = runner_source
            .find("fn build_creator_native_arm_receipt(")
            .unwrap();
        let build_end = runner_source[build_start..]
            .find("\nfn attest_stopped_creator_process(")
            .map(|offset| build_start + offset)
            .unwrap();
        let build_source = &runner_source[build_start..build_end];
        assert!(build_source.contains("O::SignTagScopedPrivateKey,\n                    -25_293,\n                    C::AuthFailed,"));
        assert!(!build_source.contains("O::SignTagScopedPrivateKey,\n                    -25_308,"));

        let mut extra = current;
        extra
            .as_object_mut()
            .unwrap()
            .insert("presence_claim".to_owned(), serde_json::json!(true));
        assert!(validate_creator_typed_receipt(
            &extra,
            MarkerState::FirstWrongPrepared,
            MarkerState::FirstFreshDeletePrepared,
            WRONG_IDENTITY_EXECUTABLE_PATH,
        )
        .is_err());
    }

    #[test]
    fn creator_rollback_attests_durably_before_exactly_one_resume() {
        let source = include_str!("runner.rs");
        let start = source
            .find("fn attest_stopped_creator_rollback_process")
            .unwrap();
        let end = source[start..]
            .find("fn securityagent_raw_evidence")
            .map(|offset| start + offset)
            .unwrap();
        let surface = &source[start..end];
        let persist = surface.find("write_runner_receipt").unwrap();
        let resume = surface.find("libc::SIGCONT").unwrap();
        assert!(persist < resume);
        assert_eq!(surface.matches("libc::SIGCONT").count(), 1);
        assert!(surface.contains("wait_for_sigstop"));
        assert!(surface.contains("measured_child_supplementary_groups_v2"));
        assert!(surface.contains("measure_frozen_executable"));
    }

    #[test]
    fn creator_rollback_recovery_uses_the_attested_path_for_both_open_phases() {
        let source = include_str!("runner.rs");
        let start = source.find("fn recover_creator_rollback").unwrap();
        let end = source[start..]
            .find("fn recover_unobserved_creator_arm")
            .map(|offset| start + offset)
            .unwrap();
        let recovery = &source[start..end];
        assert!(recovery.contains("CreatorRollbackPhaseV2::Prepared"));
        assert!(recovery.contains("CreatorRollbackPhaseV2::Invoked"));
        assert!(recovery.contains("observe_attested_creator_rollback"));

        let immediate = &source[source.find("fn run_creator_arms").unwrap()..start];
        assert!(immediate.contains("observe_attested_creator_rollback"));
        assert!(immediate.contains("original creator failure"));
        assert!(!immediate.contains(
            "observe_sealed_child(sealed_command(\n                CREATOR_EXECUTABLE_PATH"
        ));
    }

    #[test]
    fn stdout_parser_accepts_canonical_creator_rollback_receipt() {
        let expected = CreatorRollbackReceiptV2 {
            schema_owner: "substrate.r3-macos-signer-acl.creator-emergency-rollback-receipt"
                .to_owned(),
            schema_version: 2,
            repetition: FixedRepetitionV2::First,
            failure_observation_sha256: "11".repeat(32),
            exact_delete: Some(crate::ExactDeleteReceipt {
                raw_os_status: 0,
                classification: crate::ExactDeleteClassification::DeletedAndAbsent,
                present_after: false,
            }),
            exact_identity_absent: true,
        };
        let mut stdout = canonical_bytes_v2(&expected).expect("canonical rollback receipt");
        assert_ne!(
            stdout,
            serde_json::to_vec(&expected).expect("declaration-order rollback receipt")
        );
        stdout.push(b'\n');
        let observed = ObservedChildV2 {
            status: std::process::ExitStatus::from_raw(0),
            stdout,
            stderr: Vec::new(),
            securityagent_report_sha256: "22".repeat(32),
            securityagent_report: Vec::new(),
            unexpected_ui_observed: false,
        };

        let parsed: CreatorRollbackReceiptV2 =
            parse_stdout_json(&observed).expect("accept canonical sorted rollback receipt");
        assert_eq!(parsed, expected);
    }

    #[test]
    fn peer_controls_are_exact_order_and_close_before_response() {
        assert_eq!(PEER_SUBSTITUTION_CONTROL_SEQUENCE_V2.len(), 3);
        assert!(include_str!("runner.rs")
            .contains("PeerProbeTerminationV2::ConnectionClosedBeforeResponse"));
    }

    #[test]
    fn peer_native_arms_never_reinvoke_an_ambiguous_invoked_control() {
        assert_eq!(
            classify_peer_native_arm_recovery(false, false, false).unwrap(),
            PeerNativeArmRecoveryDecisionV2::Invoke
        );
        assert_eq!(
            classify_peer_native_arm_recovery(true, true, false).unwrap(),
            PeerNativeArmRecoveryDecisionV2::ReconstructObserved
        );
        assert_eq!(
            classify_peer_native_arm_recovery(true, true, true).unwrap(),
            PeerNativeArmRecoveryDecisionV2::AcceptObserved
        );
        assert_eq!(
            classify_peer_native_arm_recovery(true, false, false).unwrap(),
            PeerNativeArmRecoveryDecisionV2::AmbiguousNoReinvoke
        );
        for invalid in [
            (false, true, false),
            (false, false, true),
            (false, true, true),
            (true, false, true),
        ] {
            assert!(classify_peer_native_arm_recovery(invalid.0, invalid.1, invalid.2).is_err());
        }
        let production = include_str!("runner.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(production.contains("PeerNativeArmV2::DynamicLibraryInjection"));
        assert!(production.contains("PeerNativeArmV2::NobodyOwnerAuthority"));
        assert!(production.contains("PeerNativeArmV2::PeerSubstitution"));
        assert!(production.contains("blind reinvoke forbidden"));
    }

    #[test]
    fn benign_injection_library_has_only_the_fixed_fd4_constructor_signal() {
        let source = include_str!("../native/benign_injection_probe.c");
        assert!(source.contains("SUBSTRATE_R3_BENIGN_INJECTION_LOADED_V2\\n"));
        assert!(source.contains("write(4, cursor, remaining)"));
        assert!(!source.contains("open("));
        assert!(!source.contains("getenv("));
        assert!(!source.contains("STDERR_FILENO"));
    }

    #[test]
    fn runner_removes_only_compiled_temporary_install_artifacts() {
        let source = include_str!("runner.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("ALTERNATE_COORDINATOR_PATH_V2"));
        assert!(production.contains("DISPOSABLE_PREPARED_INPUT_PATH_V2"));
        assert!(production.contains("CANDIDATE_IDENTITY_PACKET_PATH_V2"));
        assert!(production.contains("PEER_CONTROL_IDENTITY_PACKET_PATH_V2"));
        assert!(!production.contains("remove_dir_all"));
        assert!(production.contains("observe_retained_root_install_claims"));
        assert!(production.contains("root_install_claims_retained: true"));
        assert!(production.contains("admin_cleanup_authorized: true"));
        assert!(!production.contains("remove_root_install_claim_leaf"));
        assert!(!production.contains("complete_root_install_claim_cleanup"));
    }

    #[test]
    fn publisher_exit86_is_terminal_and_cannot_reach_normal_restart() {
        assert_eq!(
            publisher_exit_disposition(false, Some(SECURITYAGENT_ALERT_EXIT_CODE)),
            PublisherExitDispositionV2::TerminalSecurityAgentAlert
        );
        assert_eq!(
            publisher_exit_disposition(false, Some(78)),
            PublisherExitDispositionV2::TerminalClosedFailure
        );
        let source = include_str!("runner.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains(
            "return complete_publisher_ui_terminal_rollback(\n                    &inputs,\n                    &harness_termination,\n                    &publisher_termination"
        ));
        assert!(production
            .contains("exit86 is handled before every harness-exit and publisher-restart branch"));
        assert!(production.contains(
            "publisher UI rollback was Invoked without a durable result; refusing reinvocation"
        ));
        let cursor = production
            .find("persist_publisher_ui_stop_cursor(")
            .expect("terminal cursor must be durable");
        let terminate = production
            .find("terminate_harness_process_group(&mut harness)")
            .expect("harness group must terminate");
        let rollback = production
            .find("spawn_publisher_rollback_only(inputs, \"publisher_ui_rollback\")")
            .expect("one rollback-only spawn must exist");
        assert!(cursor < terminate);
        assert!(terminate < rollback);
    }

    #[test]
    fn preparation_holds_activation_membrane_and_only_authenticated_acceptance_rejoins() {
        let source = include_str!("runner.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("libc::O_DIRECTORY | libc::O_NOFOLLOW"));
        assert!(production.contains("libc::flock(descriptor.as_raw_fd(), libc::LOCK_EX)"));
        assert!(production.contains("initialize_empty_activation_root_lock(&descriptor, path)"));
        assert!(production.contains("fsync initialized activation journal root lock"));
        assert!(production.contains("reopen initialized activation journal root lock"));
        assert!(production.contains("journal_root_lock_identity_sha256"));
        assert!(
            production.contains("lock_identity != global_pre_effect.journal_root_lock_identity")
        );
        assert!(production.contains("MAC_R3_FINALIZER_JOURNAL_ROOT_LOCK_PATH_V2"));
        let proof = production
            .find("install_and_validate_global_pre_effect_proof(")
            .unwrap();
        let release = production
            .find("release_activation_membrane_exclusive(&activation_membrane)")
            .unwrap();
        let creator = production
            .find("run_creator_arms(&inputs, &global_pre_effect)")
            .unwrap();
        assert!(proof < release && release < creator);
        assert!(production.contains("GENERAL_FAILURE_CURSOR_NAME"));
        assert!(production.contains("terminal_no_resume: true"));
        assert!(production.contains("exact_same_digest_rejoin_authorized: true"));
        assert!(production.contains("rollback_forbidden: true"));
    }

    #[test]
    fn activation_recovery_fresh_start_preserves_exact_parent_authority() {
        let production = include_str!("runner.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let run = production
            .split("pub fn run_fixed_experiment() -> Result<()> {")
            .nth(1)
            .unwrap()
            .split("fn verify_runner_process_surface(")
            .next()
            .unwrap();
        assert!(
            run.find("acquire_activation_membrane_exclusive()?")
                .unwrap()
                < run.find("prepare_runner_root()?").unwrap()
        );
        let recovery = production
            .split("fn activation_cleanup_recovery_documents(")
            .nth(1)
            .unwrap()
            .split("fn read_staged_native_cleanup_recovery_documents(")
            .next()
            .unwrap();
        assert!(
            recovery
                .find("read_runner_private_cleanup_recovery_optional")
                .unwrap()
                < recovery
                    .find("read_staged_native_cleanup_recovery_documents()?")
                    .unwrap()
        );
        let root = std::env::temp_dir().join(format!(
            "substrate-r3-activation-recovery-fresh-start-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).expect("create activation-recovery fixture root");
        std::fs::set_permissions(&root, std::os::unix::fs::PermissionsExt::from_mode(0o700))
            .expect("set activation-recovery fixture mode");
        let global_exchange = root.join("global-publisher-exchange");
        std::fs::create_dir(&global_exchange).expect("create exact global exchange");
        std::fs::set_permissions(
            &global_exchange,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        )
        .expect("set exact global exchange mode");
        // SAFETY: read-only process credential queries for the isolated filesystem fixture.
        let uid = unsafe { libc::geteuid() };
        // SAFETY: read-only process credential queries for the isolated filesystem fixture.
        let gid = unsafe { libc::getegid() };
        let private_identity = PublishIdentityV2 {
            owner_uid: uid,
            owner_gid: gid,
            permissions: 0o600,
            parent_uid: uid,
            parent_gid: gid,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: 1024,
        };
        let external_identity = PublishIdentityV2 {
            permissions: 0o444,
            ..private_identity
        };
        let runner_root = root.join("runner-private");
        let cleanup_name = "native-evidence-cleanup-plan.v2.json";

        // This is the real fresh-start order: the exact external exchange exists, the runner
        // root does not exist until prepare_runner_root(), and no staged recovery is present.
        assert!(read_runner_private_cleanup_recovery_bytes_optional_at(
            &runner_root,
            cleanup_name,
            private_identity,
        )
        .expect("missing fresh runner root is no private recovery document")
        .is_none());
        let external_cleanup = global_exchange.join("native-evidence-cleanup-receipt.v2.json");
        assert!(read_staged_file(&external_cleanup, external_identity)
            .expect("exact empty external exchange remains available")
            .is_none());

        // Presence never relaxes identity: a wrong runner parent remains a hard stop.
        std::fs::create_dir(&runner_root).expect("create wrong runner parent");
        std::fs::set_permissions(
            &runner_root,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .expect("set wrong runner parent mode");
        assert!(read_runner_private_cleanup_recovery_bytes_optional_at(
            &runner_root,
            cleanup_name,
            private_identity,
        )
        .is_err());

        // An absent runner root still falls through to the existing exact staged-recovery lane.
        std::fs::remove_dir(&runner_root).expect("restore absent runner root");
        let staged = br#"{"staged":true}"#;
        stage_exact_file(&external_cleanup, staged, external_identity)
            .expect("stage exact external cleanup recovery");
        assert_eq!(
            read_staged_file(&external_cleanup, external_identity)
                .expect("read exact staged external cleanup recovery"),
            Some(staged.to_vec())
        );
        std::fs::remove_dir_all(root).expect("remove activation-recovery fixture");
    }

    #[test]
    fn early_bootstrap_failure_removes_only_empty_activation_state() {
        let source = include_str!("runner.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let terminal = production
            .find("fn terminal_finalizer_failure_cleanup(")
            .expect("terminal cleanup must be explicit");
        let disposition = production
            .find("fn terminal_journal_root_disposition(")
            .expect("terminal cleanup must classify the activation root");
        assert!(terminal < disposition);
        assert!(production.contains("names.is_empty()"));
        assert!(production.contains("names != [\"journal-root.lock\"]"));
        assert!(production.contains("preserved_for_unacknowledged_evidence: true"));
        assert!(production.contains("libc::LOCK_EX | libc::LOCK_NB"));
        assert!(production.contains(
            "terminal cleanup stopped after bootout because unacknowledged journal evidence remains"
        ));
        assert!(
            production.contains("remove_exact_empty_directory(root, 0, 0, libc::S_IFDIR | 0o700)")
        );
    }

    #[test]
    fn creator_and_loader_controls_have_closed_crash_paths() {
        let source = include_str!("runner.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("CreatorArmPreparedCursorV2"));
        assert!(production.contains("recover_unobserved_creator_arm(&marker, inputs)"));
        assert!(production.contains("CreatorChildDispositionPreparedV2"));
        assert!(production.contains("individual_sigkill_authorized: true"));
        assert!(production.contains("observe_compiled_creator_processes(inputs)"));
        assert!(!production.contains("libc::kill(-"));
        assert!(production.contains("missing_observation_requires_terminal_rollback: true"));
        assert!(production.contains("BenignInjectionTerminationV2::LoaderRejectedBeforeMain"));
        assert!(production.contains("attest_dynamic_peer_or_loader_exit"));
    }

    #[test]
    fn root_operation_alert_recovers_before_baseline_and_never_resumes_normal_work() {
        let source = include_str!("runner.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let recovery = production
            .find("recover_root_operation_ui_terminal_stop_if_present(\n        &inputs")
            .expect("startup must consume the root-operation UI cursor");
        let owner_access = production
            .find("probe_nonmatch_owner_access_in_memory()")
            .expect("owner SecAccess probe must remain ordered");
        let global_baseline = production
            .find("install_and_validate_global_pre_effect_proof(")
            .expect("global baseline must remain explicit");
        assert!(recovery < owner_access && recovery < global_baseline);
        assert!(production.contains("RootOperationSecurityAgentAlert"));
        assert!(production.contains("EmergencyFailureClassificationV2::SecurityAgentAlertExit86"));
        assert!(production.contains("terminal_no_normal_resume: true"));
        assert!(production.contains("complete_general_failure_rollback(inputs, &general_cursor)"));
        assert!(production.contains("root-operation UI recovery could not prove exact"));
        assert!(production.contains("TERMINAL_ADMIN_CLEANUP_AUTHORIZATION_NAME"));
        assert!(production.contains("claims_retained: true"));
        assert!(production.contains("admin_cleanup_authorized: true"));
    }

    #[test]
    fn native_cleanup_recovery_accepts_only_one_contiguous_durable_prefix() {
        assert!(validate_native_cleanup_recovery_prefix(&[
            (true, false),
            (true, false),
            (false, false),
            (false, true),
            (false, true),
        ])
        .is_ok());
        assert!(validate_native_cleanup_recovery_prefix(&[
            (true, false),
            (false, true),
            (false, true),
        ])
        .is_ok());
        for invalid in [
            vec![(true, true)],
            vec![(false, false), (false, false)],
            vec![(false, true), (true, false)],
            vec![(true, false), (false, true), (false, false)],
        ] {
            assert!(validate_native_cleanup_recovery_prefix(&invalid).is_err());
        }
    }

    #[test]
    fn terminal_admin_archive_has_one_exact_64_mib_bound() {
        let entry = |index: usize| TerminalRunnerChildIdentityV2 {
            name: format!("child-{index:02}.json"),
            canonical_byte_length: u64::try_from(MAX_CHILD_OUTPUT).unwrap(),
            canonical_sha256: "a".repeat(64),
            physical_identity_sha256: "b".repeat(64),
        };
        let exact = (0..64).map(entry).collect::<Vec<_>>();
        assert_eq!(
            terminal_runner_child_archive_total_bytes(&exact).unwrap(),
            TERMINAL_RUNNER_CHILD_ARCHIVE_MAX_BYTES_V2
        );
        let too_large = (0..65).map(entry).collect::<Vec<_>>();
        assert!(terminal_runner_child_archive_total_bytes(&too_large).is_err());
        assert_eq!(TERMINAL_RUNNER_CHILD_ARCHIVE_MAX_BYTES_V2, 67_108_864);
    }

    #[test]
    fn global_pre_effect_packet_has_one_dedicated_runner_bound() {
        assert_eq!(
            runner_private_archive_entry_max_bytes_v2(
                GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
            ),
            GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2
        );
        assert_eq!(
            runner_private_archive_entry_max_bytes_v2("other.json"),
            MAX_CHILD_OUTPUT
        );

        let above_default = TerminalRunnerChildIdentityV2 {
            name: GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2.to_owned(),
            canonical_byte_length: u64::try_from(MAX_CHILD_OUTPUT + 1).unwrap(),
            canonical_sha256: "a".repeat(64),
            physical_identity_sha256: "b".repeat(64),
        };
        assert!(terminal_runner_child_archive_total_bytes(&[above_default]).is_ok());

        let above_global = TerminalRunnerChildIdentityV2 {
            name: GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2.to_owned(),
            canonical_byte_length: u64::try_from(GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2 + 1)
                .unwrap(),
            canonical_sha256: "a".repeat(64),
            physical_identity_sha256: "b".repeat(64),
        };
        assert!(terminal_runner_child_archive_total_bytes(&[above_global]).is_err());

        const OBSERVED_GLOBAL_PACKET_BYTES: usize = 1_175_768;
        let empty = canonical_bytes_v2(&serde_json::Value::String(String::new())).unwrap();
        let observed = canonical_bytes_v2(&serde_json::Value::String(
            "x".repeat(OBSERVED_GLOBAL_PACKET_BYTES - empty.len()),
        ))
        .unwrap();
        assert_eq!(observed.len(), OBSERVED_GLOBAL_PACKET_BYTES);
        assert!(parse_canonical_v2::<serde_json::Value>(&observed).is_err());
        assert!(
            parse_runner_private_canonical::<serde_json::Value>("other.json", &observed,).is_err()
        );
        assert!(parse_runner_private_canonical::<serde_json::Value>(
            GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
            &observed,
        )
        .is_ok());

        let overflow = canonical_bytes_v2(&serde_json::Value::String(
            "x".repeat(GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2 - empty.len() + 1),
        ))
        .unwrap();
        assert_eq!(overflow.len(), GLOBAL_PRE_EFFECT_PACKET_MAX_BYTES_V2 + 1);
        assert!(parse_runner_private_canonical::<serde_json::Value>(
            GLOBAL_PRE_EFFECT_PACKET_RUNNER_PRIVATE_NAME_V2,
            &overflow,
        )
        .is_err());
    }

    #[test]
    fn process_cleanup_is_saturation_checked_and_never_signals_a_negative_pgid() {
        let production = include_str!("runner.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(production.matches(">= pids.len()").count() >= 2);
        assert!(production.contains("is_process_disappearance_error(&error)"));
        assert!(production.contains("reattest_exact_process_member(process_group_id, member)"));
        assert!(production.contains("libc::kill(member.pid, libc::SIGTERM)"));
        assert!(production.contains("libc::kill(member.pid, libc::SIGKILL)"));
        assert!(!production.contains("libc::kill(-process_group_id"));
        assert!(!production.contains("libc::kill(-pgid"));
    }

    #[test]
    fn sealed_children_drop_and_attest_an_exact_empty_supplementary_group_vector() {
        let production = include_str!("runner.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let sealed = production
            .split("fn sealed_command(")
            .nth(1)
            .unwrap()
            .split("fn prepare_runner_root(")
            .next()
            .unwrap();
        let setgroups = sealed.find("libc::setgroups(0").unwrap();
        let setgid = sealed.find("libc::setgid(gid)").unwrap();
        let setuid = sealed.find("libc::setuid(uid)").unwrap();
        let getgroups = sealed.find("libc::getgroups(0").unwrap();
        assert!(setgroups < setgid && setgid < setuid && setuid < getgroups);
        assert!(sealed.contains("if raw_group_count < 0"));
        assert!(sealed.contains("libc::getgroups(1, &mut only_group)"));
        assert!(sealed.contains("Some(only_group)"));
        assert!(sealed.contains("normalized_group_count.to_ne_bytes()"));
        assert!(!sealed.contains("command.gid("));
        assert!(!sealed.contains("command.uid("));
        assert!(sealed.contains("SEALED_GROUP_MEASUREMENT_MAGIC_V2"));
        assert!(sealed.contains("group_measurement_writer.as_raw_fd()"));
        assert!(production
            .contains("supplementary_groups: measured_child_supplementary_groups_v2(pid)?"));
        assert!(production.contains("supplementary_groups.validate_empty().is_err()"));
        assert!(!production.contains("inherited_from_unprivileged_parent"));
        let shared = include_str!("../../r3-macos-finalizer/src/experiment/process.rs");
        assert!(shared.contains("SupplementaryGroupEvidenceV2::CurrentProcessGetgroups"));
        assert!(shared.contains("SupplementaryGroupEvidenceV2::SealedPreExecPostDropGetgroups"));
        assert!(!shared.contains("SupplementaryGroupEvidenceV2::UnprivilegedParentInheritance"));
    }

    #[test]
    fn post_drop_getgroups_normalizes_only_the_effective_gid() {
        assert_eq!(normalize_post_drop_getgroups_v2(0, None, 0).unwrap(), 0);
        assert_eq!(
            normalize_post_drop_getgroups_v2(1, Some(20), 20).unwrap(),
            0
        );
        assert!(normalize_post_drop_getgroups_v2(1, Some(21), 20).is_err());
        assert!(normalize_post_drop_getgroups_v2(2, None, 20).is_err());
        assert!(normalize_post_drop_getgroups_v2(0, Some(20), 20).is_err());
    }

    #[test]
    fn accepted_authority_is_classified_before_any_terminal_bootout() {
        let production = include_str!("runner.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let outer_classification = production
            .find("match exact_accepted_authority_before_terminal_cleanup()")
            .unwrap();
        let outer_cleanup = production[outer_classification..]
            .find("terminal_finalizer_cleanup_receipt_sha256(")
            .unwrap();
        assert!(outer_cleanup > 0);
        let terminal = production
            .split("fn terminal_finalizer_failure_cleanup(")
            .nth(1)
            .unwrap();
        let classify = terminal
            .find("exact_accepted_authority_before_terminal_cleanup()")
            .unwrap();
        let bootout = terminal.find(".arg(\"bootout\")").unwrap();
        assert!(classify < bootout);
        assert!(terminal.contains("same-digest rejoin"));
    }

    #[test]
    fn all_runner_and_publisher_first_security_calls_are_observer_spanned() {
        let runner = include_str!("runner.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let compact_runner = runner.split_whitespace().collect::<String>();
        assert!(compact_runner.contains(
            "observe_root_operation_with_raw(NonInteractiveSecurity::establish_first,|alert|"
        ));
        assert!(runner.contains("nobody-owner-first-security-interaction-denial"));
        assert!(runner.contains("finalizer-service-bootstrap-startup"));
        assert!(runner.contains("observe_stopped_child_startup("));
        let publisher = include_str!("bin/disposable_publisher.rs");
        let establish = publisher
            .find("NonInteractiveSecurity::establish_first()?")
            .unwrap();
        let stop = publisher.find("libc::raise(libc::SIGSTOP)").unwrap();
        let supervisor = publisher
            .find("run_disposable_publisher_supervisor(&mut security)")
            .unwrap();
        assert!(establish < stop && stop < supervisor);
    }

    #[test]
    fn launchd_not_found_classifier_has_one_exact_raw_tuple() {
        let label = MAC_R3_FINALIZER_LAUNCHD_LABEL_V2;
        let exact = exact_launchctl_service_not_found_stderr(label);
        assert_eq!(
            exact.as_bytes(),
            format!("Bad request.\nCould not find service \"{label}\" in domain for system\n")
                .as_bytes()
        );
        assert_ne!(format!("prefix{exact}"), exact);
        assert_ne!(format!("{exact}suffix"), exact);
    }

    #[test]
    fn terminal_cleanup_recovery_rejects_any_noncanonical_launchd_evidence() {
        let failure_sha256 = "a".repeat(64);
        let empty = bounded_raw_stream_evidence(b"").unwrap();
        let exact_stderr = bounded_raw_stream_evidence(
            exact_launchctl_service_not_found_stderr(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2).as_bytes(),
        )
        .unwrap();
        assert!(validate_terminal_finalizer_launchd_cleanup_evidence(
            false,
            None,
            113,
            &empty,
            &exact_stderr,
            &failure_sha256,
        )
        .is_ok());

        for altered in [
            b"Could not find service \"com.atomize.substrate.r3-macos-evidence-finalizer.v2\" in domain for system\n".to_vec(),
            format!(
                "prefix{}",
                exact_launchctl_service_not_found_stderr(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)
            )
            .into_bytes(),
            format!(
                "{}suffix",
                exact_launchctl_service_not_found_stderr(MAC_R3_FINALIZER_LAUNCHD_LABEL_V2)
            )
            .into_bytes(),
        ] {
            let altered = bounded_raw_stream_evidence(&altered).unwrap();
            assert!(validate_terminal_finalizer_launchd_cleanup_evidence(
                false,
                None,
                113,
                &empty,
                &altered,
                &failure_sha256,
            )
            .is_err());
        }
        let nonempty_stdout = bounded_raw_stream_evidence(b"unexpected").unwrap();
        assert!(validate_terminal_finalizer_launchd_cleanup_evidence(
            false,
            None,
            113,
            &nonempty_stdout,
            &exact_stderr,
            &failure_sha256,
        )
        .is_err());
        let mut malformed = exact_stderr.clone();
        malformed.raw_sha256 = "b".repeat(64);
        assert!(validate_terminal_finalizer_launchd_cleanup_evidence(
            false,
            None,
            113,
            &empty,
            &malformed,
            &failure_sha256,
        )
        .is_err());

        let bootout = FinalizerBootoutResultV2 {
            schema_owner: NATIVE_EVIDENCE_EXPORT_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            prepared_sha256: failure_sha256.clone(),
            exit_status: 0,
            stdout: empty.clone(),
            stderr: empty.clone(),
        };
        assert!(validate_terminal_finalizer_launchd_cleanup_evidence(
            true,
            Some(&bootout),
            113,
            &empty,
            &exact_stderr,
            &failure_sha256,
        )
        .is_ok());
        assert!(validate_terminal_finalizer_launchd_cleanup_evidence(
            false,
            Some(&bootout),
            113,
            &empty,
            &exact_stderr,
            &failure_sha256,
        )
        .is_err());
        let mut bad_bootout = bootout;
        bad_bootout.stdout = nonempty_stdout;
        assert!(validate_terminal_finalizer_launchd_cleanup_evidence(
            true,
            Some(&bad_bootout),
            113,
            &empty,
            &exact_stderr,
            &failure_sha256,
        )
        .is_err());
    }
}
