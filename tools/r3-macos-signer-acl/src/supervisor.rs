//! Long-lived, closed root publisher for the two disposable finalizer repetitions.
//!
//! The supervisor has no caller-selected action or path. It consumes only the shared protocol's
//! fixed per-repetition artifacts, in the shared stage order, after process interaction denial has
//! already been established as the process's first Security call.

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::ffi::CString;
use std::fs::File;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, parse_canonical_v2, sha256_hex_v2,
    FinalizationRequestV2, FinalizerResponseStateV2, FinalizerResponseV2,
    HarnessDurabilityAcknowledgementV2, HostParityProofV2, ProtectedCasBindingV2,
    PublisherPreRemovalReceiptV2, TargetSetKindV2, TerminalAcknowledgementV2,
    MAC_R3_COORDINATOR_INBOX_DIRECTORY_MODE_V2, MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2,
    MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2, MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2,
    MAC_R3_FINALIZER_JOURNAL_ROOT_V2, MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
};
use substrate_r3_macos_finalizer::contract::{
    TerminalBindingRequest, TERMINAL_BINDING_OWNER, TERMINAL_BINDING_VERSION,
};
use substrate_r3_macos_finalizer::experiment::controls::{
    TransportControlsCompleteMarkerV2, TransportControlsObservationReceiptV2,
    TRANSPORT_CONTROL_RECEIPT_OWNER_V2,
};
use substrate_r3_macos_finalizer::experiment::publisher_protocol::{
    expected_input_artifacts_v2, external_exchange_path_v2, installed_finalizer_inbox_path_v2,
    root_publisher_path_v2, validate_published_request_v2, validate_signed_cas_output_v2,
    validate_signed_receipt_output_v2, validate_terminal_publication_v2,
    DisposableSignerIdentityV2, EmergencyRollbackMarkerV2, EmergencyRollbackReceiptV2,
    IdentityBindingPacketV2, PreCreationEmergencyRollbackMarkerV2, PublisherAdvanceMarkerV2,
    PublisherArtifactV2, PublisherInputArtifactDigestV2, PublisherInputArtifactV2,
    PublisherInputBundleV2, PublisherPreparedInputV2, PublisherProcessAttestationV2,
    PublisherProgressV2, PublisherSecurityAgentObservationV2, PublisherStageV2,
    PublisherStepReceiptV2, ResidualCleanupReceiptV2, RestorationReceiptV2,
    SurrogateCreationReceiptV2, PUBLISHER_PROTOCOL_OWNER_V2, PUBLISHER_STAGE_SEQUENCE_V2,
};
use substrate_r3_macos_finalizer::experiment::{
    RepetitionV2, DISPOSABLE_HARNESS_UID_V2, DISPOSABLE_PUBLISHER_ROOT_V2, EXPERIMENT_ID_V2,
    EXPERIMENT_VERSION_V2,
};

use crate::attestation::attest_publisher_process;
use crate::ffi::DisposableAclKind;
use crate::immutable_publish::{
    publish_exact_file, read_exact_published_file, PublishIdentityV2, PublishMode,
};
use crate::publisher::{
    apply_signature, fill_or_verify_wrapper, prepare_empty_wrapper, prevalidate_unsigned_binding,
    prevalidate_unsigned_receipt, wrapper_path,
};
use crate::securityagent::{observe_stage, SecurityAgentTerminalAlertV2};
use crate::{
    compiled_disposable_target_config, compiled_disposable_wrong_config,
    DisposableKeyPairCreationReceiptV2, ExactDeleteClassification, ExactDeleteReceipt,
    FixedRepetitionV2, NonInteractiveSecurity,
};

const ROOT_DIRECTORY_MODE: libc::mode_t = libc::S_IFDIR | 0o700;
const ROOT_FILE_MODE: libc::mode_t = libc::S_IFREG | 0o600;
const EXTERNAL_INPUT_MODE: libc::mode_t = libc::S_IFREG | 0o400;
const EXTERNAL_OUTPUT_MODE: libc::mode_t = libc::S_IFREG | 0o444;
const INSTALLED_INBOX_DIRECTORY_MODE: libc::mode_t =
    libc::S_IFDIR | MAC_R3_COORDINATOR_INBOX_DIRECTORY_MODE_V2 as libc::mode_t;
const INSTALLED_INBOX_GID: u32 = MAC_R3_COORDINATOR_INBOX_GROUP_GID_V2;
const INSTALLED_INBOX_UID: u32 = MAC_R3_COORDINATOR_INBOX_OWNER_UID_V2;
const LOCK_FILE_MODE: libc::mode_t = libc::S_IFREG | 0o600;
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const CURRENT_LOCK_BYTES: &[u8] = b"substrate-r3-disposable-current-lock-v2\n";
const TERMINAL_LATCH_BYTES: &[u8] = b"substrate-r3-retirement-terminal-latch-v2\n";
const CAPABILITY_REPORT_OWNER: &str = "substrate.r3-macos-disposable-capability";
const SURROGATE_OBSERVATION_OWNER: &str = "substrate.r3-macos-disposable-surrogate-observation";
const RESIDUAL_CURSOR_NAME: &str = "residual-cleanup.cursor.v2.json";
const RESTORATION_CURSOR_NAME: &str = "restoration-inbox-cleanup.cursor.v2.json";
const TRANSPORT_UI_OBSERVATION_NAME: &str = "transport-controls.ui.observation.v2.json";
const EMERGENCY_UI_OBSERVATION_NAME: &str = "emergency-rollback.ui.observation.v2.json";

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ExactFileObservation<'a> {
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

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct FinalizerKeyObservation<'a> {
    application_tag_sha256: &'a str,
    label: &'a str,
    application_label_base64url: &'a str,
    persistent_reference_sha256: &'a str,
    public_spki_sha256: &'a str,
    access_control_sha256: &'a str,
    identity_sha256: &'a str,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct CapabilityObservation<'a> {
    schema_owner: &'static str,
    schema_version: u32,
    target_set_kind: TargetSetKindV2,
    scope_id: &'a str,
    signer_access_control_sha256: &'a str,
    target: FinalizerKeyObservation<'a>,
    wrong_surrogate: FinalizerKeyObservation<'a>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct SurrogateSetObservation<'a> {
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

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct AbsenceObservation<'a> {
    schema_owner: &'static str,
    schema_version: u32,
    scope_id: &'a str,
    target_absent: bool,
    wrong_absent: bool,
    wrapper_absent: bool,
    current_lock_absent: bool,
    terminal_latch_absent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ResidualCursorPhaseV2 {
    Prepared,
    Invoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ResidualCursorV2 {
    schema_owner: String,
    schema_version: u32,
    repetition: u8,
    scope_id: String,
    effects_response_sha256: String,
    wrong_identity_sha256: String,
    phase: ResidualCursorPhaseV2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RestorationCursorPhaseV2 {
    Prepared,
    RequestInvoked,
    RequestObserved,
    TerminalInvoked,
    TerminalObserved,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RestorationCursorV2 {
    schema_owner: String,
    schema_version: u32,
    repetition: u8,
    scope_id: String,
    complete_response_sha256: String,
    request_sha256: String,
    terminal_sha256: String,
    phase: RestorationCursorPhaseV2,
}

#[derive(Debug)]
enum StageOutputV2 {
    Bytes(Vec<u8>),
    Residual(ResidualAfterObservationV2),
    Restoration(RestorationAfterObservationV2),
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ResidualAfterObservationV2 {
    effects_response_sha256: String,
    target_absent: bool,
    wrong_absent: bool,
    wrong_identity_sha256: String,
    exact_after_observation_sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RestorationAfterObservationV2 {
    complete_response_sha256: String,
    target_absent: bool,
    wrong_absent: bool,
    wrapper_absent: bool,
    current_lock_absent: bool,
    terminal_latch_absent: bool,
    request_inbox_removed: bool,
    terminal_inbox_removed: bool,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct StagePrearmBindingV2<'a> {
    prepared_securityagent_baseline_sha256: &'a str,
    process_attestation_chain_head_sha256: &'a str,
    progress_generation: u64,
    stage: PublisherStageV2,
    marker_sha256: &'a str,
    input_bundle_sha256: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProcessAttestationChainEntryV2 {
    schema_owner: String,
    schema_version: u32,
    experiment_id: String,
    repetition: u8,
    scope_id: String,
    ordinal: u64,
    predecessor_entry_sha256: String,
    progress_generation: u64,
    stage: PublisherStageV2,
    marker_sha256: String,
    input_bundle_sha256: String,
    process_attestation: PublisherProcessAttestationV2,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct EmergencyRestorationObservationV2 {
    repetition: u8,
    scope_id: String,
    target_absent: bool,
    wrong_absent: bool,
    wrapper_absent: bool,
    current_lock_absent: bool,
    terminal_latch_absent: bool,
    accepted_journal_absent: bool,
    request_inbox_absent: bool,
    terminal_inbox_absent: bool,
}

/// Run both fixed repetitions in one root process. There is no action selector: each transition is
/// admitted only by the shared, fixed advance-marker/input-bundle pair for the current stage.
pub fn run_disposable_publisher_supervisor(security: &mut NonInteractiveSecurity) -> Result<()> {
    for repetition in RepetitionV2::ALL {
        let pending = PublisherStore { repetition };
        let prepared = pending.wait_for_prepared_input()?;
        let current_attestation = attest_publisher_process(security, &prepared)?;
        let store = PublisherStore::open_or_prepare(repetition)?;
        store.publish_initial_process_attestation(&prepared, &current_attestation)?;
        if !store.run_repetition(security)? {
            let later_emergency_exists = RepetitionV2::ALL
                .into_iter()
                .filter(|candidate| candidate.ordinal() > repetition.ordinal())
                .map(|candidate| PublisherStore {
                    repetition: candidate,
                })
                .map(|candidate| candidate.emergency_marker_present())
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .any(|present| present);
            if later_emergency_exists {
                continue;
            }
            return Ok(());
        }
    }
    Ok(())
}

/// Verify the one compiled publisher executable/cwd/root-account surface without a Security call.
pub fn verify_closed_publisher_process_surface() -> Result<()> {
    crate::publisher::verify_closed_publisher_process_surface()
}

struct PublisherStore {
    repetition: RepetitionV2,
}

impl PublisherStore {
    fn emergency_marker_present(&self) -> Result<bool> {
        Ok(self
            .try_read_external::<PreCreationEmergencyRollbackMarkerV2>(
                PublisherArtifactV2::PreCreationEmergencyRollbackMarker,
            )?
            .is_some()
            || self
                .try_read_external::<EmergencyRollbackMarkerV2>(
                    PublisherArtifactV2::EmergencyRollbackMarker,
                )?
                .is_some())
    }

    fn open_or_prepare(repetition: RepetitionV2) -> Result<Self> {
        prepare_root_repetition(repetition)?;
        let store = Self { repetition };
        store.ensure_initial_progress()?;
        Ok(store)
    }

    fn wait_for_prepared_input(&self) -> Result<PublisherPreparedInputV2> {
        loop {
            if let Some(prepared) = self
                .try_read_external::<PublisherPreparedInputV2>(PublisherArtifactV2::PreparedInput)?
            {
                prepared.validate()?;
                return Ok(prepared);
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn publish_initial_process_attestation(
        &self,
        prepared: &PublisherPreparedInputV2,
        current: &PublisherProcessAttestationV2,
    ) -> Result<()> {
        current.validate(prepared)?;
        let value = if let Some(existing) = self.try_read_root::<PublisherProcessAttestationV2>(
            PublisherArtifactV2::PublisherProcessAttestation,
        )? {
            // A restart necessarily has a different PID/start. The original immutable attestation
            // remains bound to SurrogatesCreated; current invokers go into the append-only chain.
            existing.validate(prepared)?;
            existing
        } else {
            current.clone()
        };
        let bytes = canonical_bytes_v2(&value)?;
        self.persist_root(
            PublisherArtifactV2::PublisherProcessAttestation,
            &bytes,
            None,
        )?;
        self.publish_external_output(PublisherArtifactV2::PublisherProcessAttestation, &bytes)
    }

    fn append_process_attestation(
        &self,
        progress: &PublisherProgressV2,
        stage: PublisherStageV2,
        marker: &PublisherAdvanceMarkerV2,
        bundle: &PublisherInputBundleV2,
        attestation: &PublisherProcessAttestationV2,
    ) -> Result<String> {
        let prepared = self.prepared_input()?;
        attestation.validate(&prepared)?;
        let marker_sha256 = document_sha256_v2(marker)?;
        let input_bundle_sha256 = document_sha256_v2(bundle)?;
        let mut predecessor = scope_digest(self.repetition, "process-attestation-chain-genesis");
        let mut ordinal = 1_u64;
        while ordinal <= 64 {
            let path = self.internal_path(&format!("process-attestation-{ordinal:02}.v2.json"));
            let Some(bytes) = read_exact_path(&path, 0, ROOT_FILE_MODE)? else {
                break;
            };
            let existing: ProcessAttestationChainEntryV2 = parse_canonical_v2(&bytes)?;
            validate_process_chain_entry(
                &existing,
                self.repetition,
                ordinal,
                &predecessor,
                &prepared,
            )?;
            let existing_sha256 = document_sha256_v2(&existing)?;
            if existing.progress_generation == progress.generation
                && existing.stage == stage
                && existing.marker_sha256 == marker_sha256
                && existing.input_bundle_sha256 == input_bundle_sha256
                && existing.process_attestation == *attestation
            {
                return Ok(existing_sha256);
            }
            predecessor = existing_sha256;
            ordinal += 1;
        }
        if ordinal > 64 {
            bail!("publisher process-attestation chain exceeded its fixed bound")
        }
        let entry = ProcessAttestationChainEntryV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: self.repetition.ordinal(),
            scope_id: self.repetition.scope_id().to_owned(),
            ordinal,
            predecessor_entry_sha256: predecessor,
            progress_generation: progress.generation,
            stage,
            marker_sha256,
            input_bundle_sha256,
            process_attestation: attestation.clone(),
        };
        validate_process_chain_entry(
            &entry,
            self.repetition,
            ordinal,
            &entry.predecessor_entry_sha256,
            &prepared,
        )?;
        write_exact_path(
            &self.internal_path(&format!("process-attestation-{ordinal:02}.v2.json")),
            &canonical_bytes_v2(&entry)?,
            0o600,
            None,
            ParentOwnerV2::Root,
        )?;
        document_sha256_v2(&entry)
    }

    fn internal_path(&self, name: &str) -> PathBuf {
        root_publisher_path_v2(self.repetition, PublisherArtifactV2::Progress)
            .parent()
            .expect("root publisher progress has a parent")
            .join(name)
    }

    fn persist_terminal_alert(
        &self,
        stage: PublisherStageV2,
        alert: &SecurityAgentTerminalAlertV2,
    ) -> Result<()> {
        alert.raw_report.validate_terminal_alert()?;
        alert.arm_evidence.validate_terminal_alert()?;
        let name = format!(
            "terminal-alert-{}.arm-evidence.v2.json",
            PublisherArtifactV2::SecurityAgentObservation(stage).filename()
        );
        let path = self.internal_path(&name);
        let bytes = canonical_bytes_v2(&alert.arm_evidence)?;
        if let Some(existing) = read_exact_path(&path, 0, ROOT_FILE_MODE)? {
            if existing != bytes {
                bail!("publisher terminal ALERT evidence changed during recovery")
            }
            return Ok(());
        }
        write_exact_path(&path, &bytes, 0o600, None, ParentOwnerV2::Root)
    }

    fn persist_recovery_observation(
        &self,
        stage: PublisherStageV2,
        purpose: &'static str,
        observation: &PublisherSecurityAgentObservationV2,
    ) -> Result<()> {
        observation.validate(self.repetition, stage)?;
        if !matches!(
            purpose,
            "residual-readback"
                | "restoration-readback"
                | "emergency-readback"
                | "transport-readback"
                | "unobserved-state-preflight"
        ) {
            bail!("publisher recovery observation purpose is outside the closed set")
        }
        let bytes = canonical_bytes_v2(observation)?;
        let name = format!(
            "recovery-{}-{}-{}.ui-observation.v2.json",
            PublisherArtifactV2::SecurityAgentObservation(stage).filename(),
            purpose,
            sha256_hex_v2(&bytes)
        );
        let path = self.internal_path(&name);
        if let Some(existing) = read_exact_path(&path, 0, ROOT_FILE_MODE)? {
            if existing != bytes {
                bail!("publisher recovery SecurityAgent observation changed")
            }
            return Ok(());
        }
        write_exact_path(&path, &bytes, 0o600, None, ParentOwnerV2::Root)
    }

    fn run_repetition(&self, security: &mut NonInteractiveSecurity) -> Result<bool> {
        loop {
            let progress = self.read_progress()?;
            if progress.stage == PublisherStageV2::RestorationComplete {
                return Ok(true);
            }
            if self.handle_emergency_if_present(security, &progress)? {
                return Ok(false);
            }
            if !self.handle_transport_checkpoint_if_present(security, &progress)? {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
            let next = successor_stage(progress.stage)?;
            let (marker, bundle) = self.wait_for_transition(progress.stage, next)?;
            if self.recover_observed_stage(security, &progress, next, &marker, &bundle)? {
                continue;
            }
            if self.stage_has_unobserved_state(security, next)? {
                bail!("publisher stage has unobserved state and cannot be blindly reinvoked")
            }
            let prepared = self.prepared_input()?;
            let current_attestation = attest_publisher_process(security, &prepared)?;
            let process_chain_head = self.append_process_attestation(
                &progress,
                next,
                &marker,
                &bundle,
                &current_attestation,
            )?;
            let prearm_baseline_sha256 = document_sha256_v2(&StagePrearmBindingV2 {
                prepared_securityagent_baseline_sha256: &prepared.securityagent_baseline_sha256,
                process_attestation_chain_head_sha256: &process_chain_head,
                progress_generation: progress.generation,
                stage: next,
                marker_sha256: &document_sha256_v2(&marker)?,
                input_bundle_sha256: &document_sha256_v2(&bundle)?,
            })?;
            let (stage_output, observation) = observe_stage(
                self.repetition,
                next,
                &prearm_baseline_sha256,
                || self.invoke_stage(security, next, &bundle),
                |alert| self.persist_terminal_alert(next, alert),
            )?;
            let observation_bytes = canonical_bytes_v2(&observation)?;
            self.publish_protocol_output(
                PublisherArtifactV2::SecurityAgentObservation(next),
                &observation_bytes,
            )?;
            let securityagent_observation_sha256 = document_sha256_v2(&observation)?;
            let stage_output = stage_output?;
            let output =
                self.finalize_stage_output(stage_output, &securityagent_observation_sha256)?;
            let output_sha256 = sha256_hex_v2(&output);
            let step = PublisherStepReceiptV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_owned(),
                repetition: self.repetition.ordinal(),
                scope_id: self.repetition.scope_id().to_owned(),
                prior_stage: progress.stage,
                new_stage: next,
                marker_sha256: document_sha256_v2(&marker)?,
                output_sha256: output_sha256.clone(),
                securityagent_observation_sha256,
            };
            step.validate(self.repetition, &marker, &output_sha256)?;
            step.validate_securityagent_observation(self.repetition, &observation)?;
            self.publish_protocol_output(
                PublisherArtifactV2::StepReceipt(next),
                &canonical_bytes_v2(&step)?,
            )?;
            let next_progress = PublisherProgressV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_owned(),
                repetition: self.repetition.ordinal(),
                scope_id: self.repetition.scope_id().to_owned(),
                generation: progress
                    .generation
                    .checked_add(1)
                    .context("publisher progress generation overflow")?,
                stage: next,
                predecessor_progress_sha256: document_sha256_v2(&progress)?,
                accepted_input_bundle_sha256: document_sha256_v2(&bundle)?,
                step_receipt_sha256: document_sha256_v2(&step)?,
            };
            next_progress.validate_successor(
                self.repetition,
                &progress,
                &marker,
                &bundle,
                &step,
            )?;
            self.replace_progress(&progress, &next_progress)?;
        }
    }

    fn recover_observed_stage(
        &self,
        security: &mut NonInteractiveSecurity,
        progress: &PublisherProgressV2,
        next: PublisherStageV2,
        marker: &PublisherAdvanceMarkerV2,
        bundle: &PublisherInputBundleV2,
    ) -> Result<bool> {
        let observation = self.try_read_root::<PublisherSecurityAgentObservationV2>(
            PublisherArtifactV2::SecurityAgentObservation(next),
        )?;
        let step =
            self.try_read_root::<PublisherStepReceiptV2>(PublisherArtifactV2::StepReceipt(next))?;
        if observation.is_none() && step.is_none() {
            return Ok(false);
        }
        let observation = observation
            .context("publisher step receipt exists without its immutable UI observation")?;
        observation.validate(self.repetition, next)?;
        let observation_sha256 = document_sha256_v2(&observation)?;
        let output = match self.try_read_stage_output(next)? {
            Some(bytes) => bytes,
            None if next == PublisherStageV2::ResidualRemoved => {
                let prepared = self.prepared_input()?;
                let (after, recovery_observation) = observe_stage(
                    self.repetition,
                    next,
                    &prepared.securityagent_baseline_sha256,
                    || self.reconstruct_residual_after_absence(security),
                    |alert| self.persist_terminal_alert(next, alert),
                )?;
                self.persist_recovery_observation(
                    next,
                    "residual-readback",
                    &recovery_observation,
                )?;
                let after = after?;
                self.finalize_stage_output(StageOutputV2::Residual(after), &observation_sha256)?
            }
            None if next == PublisherStageV2::RestorationComplete => {
                let prepared = self.prepared_input()?;
                let (after, recovery_observation) = observe_stage(
                    self.repetition,
                    next,
                    &prepared.securityagent_baseline_sha256,
                    || self.reconstruct_restoration_after_absence(security),
                    |alert| self.persist_terminal_alert(next, alert),
                )?;
                self.persist_recovery_observation(
                    next,
                    "restoration-readback",
                    &recovery_observation,
                )?;
                let after = after?;
                self.finalize_stage_output(StageOutputV2::Restoration(after), &observation_sha256)?
            }
            None => bail!("observed publisher stage lacks its exact durable output"),
        };
        let output_sha256 = sha256_hex_v2(&output);
        let step = if let Some(existing) = step {
            existing.validate(self.repetition, marker, &output_sha256)?;
            existing.validate_securityagent_observation(self.repetition, &observation)?;
            existing
        } else {
            let value = PublisherStepReceiptV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_owned(),
                repetition: self.repetition.ordinal(),
                scope_id: self.repetition.scope_id().to_owned(),
                prior_stage: progress.stage,
                new_stage: next,
                marker_sha256: document_sha256_v2(marker)?,
                output_sha256,
                securityagent_observation_sha256: observation_sha256,
            };
            value.validate(self.repetition, marker, &value.output_sha256)?;
            value.validate_securityagent_observation(self.repetition, &observation)?;
            self.persist_root(
                PublisherArtifactV2::StepReceipt(next),
                &canonical_bytes_v2(&value)?,
                None,
            )?;
            value
        };
        self.publish_external_output(
            PublisherArtifactV2::SecurityAgentObservation(next),
            &canonical_bytes_v2(&observation)?,
        )?;
        self.publish_external_output(
            PublisherArtifactV2::StepReceipt(next),
            &canonical_bytes_v2(&step)?,
        )?;
        let next_progress = PublisherProgressV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: self.repetition.ordinal(),
            scope_id: self.repetition.scope_id().to_owned(),
            generation: progress.generation + 1,
            stage: next,
            predecessor_progress_sha256: document_sha256_v2(progress)?,
            accepted_input_bundle_sha256: document_sha256_v2(bundle)?,
            step_receipt_sha256: document_sha256_v2(&step)?,
        };
        next_progress.validate_successor(self.repetition, progress, marker, bundle, &step)?;
        self.replace_progress(progress, &next_progress)?;
        Ok(true)
    }

    fn try_read_stage_output(&self, stage: PublisherStageV2) -> Result<Option<Vec<u8>>> {
        let artifact = match stage {
            PublisherStageV2::Prepared => return Ok(None),
            PublisherStageV2::SurrogatesCreated => PublisherArtifactV2::CreationReceipt,
            PublisherStageV2::ReceiptSigned => PublisherArtifactV2::SignedReceipt,
            PublisherStageV2::ProtectedCasSigned => PublisherArtifactV2::SignedProtectedCas,
            PublisherStageV2::FinalizationRequestPublished => {
                PublisherArtifactV2::FinalizationRequest
            }
            PublisherStageV2::ResidualRemoved => PublisherArtifactV2::ResidualCleanupReceipt,
            PublisherStageV2::TerminalBindingPublished => PublisherArtifactV2::TerminalBinding,
            PublisherStageV2::RestorationComplete => PublisherArtifactV2::RestorationReceipt,
        };
        read_exact_path(
            &root_publisher_path_v2(self.repetition, artifact),
            0,
            ROOT_FILE_MODE,
        )
    }

    fn stage_has_unobserved_state(
        &self,
        security: &mut NonInteractiveSecurity,
        stage: PublisherStageV2,
    ) -> Result<bool> {
        let prepared = self.prepared_input()?;
        let (present, observation) = observe_stage(
            self.repetition,
            stage,
            &prepared.securityagent_baseline_sha256,
            || {
                if self.try_read_stage_output(stage)?.is_some() {
                    return Ok(true);
                }
                match stage {
                    PublisherStageV2::SurrogatesCreated => {
                        let fixed = fixed_repetition(self.repetition);
                        Ok(security
                            .read_disposable_signer(
                                &compiled_disposable_target_config(fixed)?,
                                DisposableAclKind::Target,
                            )?
                            .is_some()
                            || security
                                .read_disposable_signer(
                                    &compiled_disposable_wrong_config(fixed)?,
                                    DisposableAclKind::WrongSurrogate,
                                )?
                                .is_some()
                            || path_present(Path::new(&wrapper_path(fixed)))?
                            || path_present(&current_lock_path(self.repetition))?
                            || path_present(&terminal_latch_path(self.repetition))?)
                    }
                    PublisherStageV2::ResidualRemoved => {
                        Ok(path_present(&self.internal_path(RESIDUAL_CURSOR_NAME))?)
                    }
                    PublisherStageV2::RestorationComplete => {
                        if path_present(&self.internal_path(RESTORATION_CURSOR_NAME))? {
                            // A cursor-first partial restoration is resumed only inside the fresh
                            // observer opened by invoke_stage; the cursor binds both exact leaves.
                            Ok(false)
                        } else {
                            Ok(!path_present(installed_finalizer_inbox_path_v2(
                                PublisherArtifactV2::FinalizationRequest,
                            )?)? || !path_present(installed_finalizer_inbox_path_v2(
                                PublisherArtifactV2::TerminalBinding,
                            )?)?)
                        }
                    }
                    _ => Ok(false),
                }
            },
            |alert| self.persist_terminal_alert(stage, alert),
        )?;
        self.persist_recovery_observation(stage, "unobserved-state-preflight", &observation)?;
        present
    }

    fn invoke_stage(
        &self,
        security: &mut NonInteractiveSecurity,
        next: PublisherStageV2,
        bundle: &PublisherInputBundleV2,
    ) -> Result<StageOutputV2> {
        match next {
            PublisherStageV2::Prepared => bail!("Prepared is not an invocable successor stage"),
            PublisherStageV2::SurrogatesCreated => self
                .create_surrogates(security, bundle)
                .map(StageOutputV2::Bytes),
            PublisherStageV2::ReceiptSigned => {
                self.sign_receipt(security).map(StageOutputV2::Bytes)
            }
            PublisherStageV2::ProtectedCasSigned => {
                self.sign_protected_cas(security).map(StageOutputV2::Bytes)
            }
            PublisherStageV2::FinalizationRequestPublished => {
                self.publish_request().map(StageOutputV2::Bytes)
            }
            PublisherStageV2::ResidualRemoved => {
                self.remove_residual(security).map(StageOutputV2::Residual)
            }
            PublisherStageV2::TerminalBindingPublished => {
                self.publish_terminal_binding().map(StageOutputV2::Bytes)
            }
            PublisherStageV2::RestorationComplete => self
                .finish_restoration(security)
                .map(StageOutputV2::Restoration),
        }
    }

    fn finalize_stage_output(
        &self,
        output: StageOutputV2,
        securityagent_observation_sha256: &str,
    ) -> Result<Vec<u8>> {
        match output {
            StageOutputV2::Bytes(bytes) => Ok(bytes),
            StageOutputV2::Residual(after) => {
                let receipt = ResidualCleanupReceiptV2 {
                    schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
                    schema_version: EXPERIMENT_VERSION_V2,
                    experiment_id: EXPERIMENT_ID_V2.to_owned(),
                    repetition: self.repetition.ordinal(),
                    scope_id: self.repetition.scope_id().to_owned(),
                    effects_response_sha256: after.effects_response_sha256.clone(),
                    target_absent: after.target_absent,
                    wrong_absent: after.wrong_absent,
                    exact_after_observation_sha256: after.exact_after_observation_sha256.clone(),
                    securityagent_observation_sha256: securityagent_observation_sha256.to_owned(),
                };
                receipt.validate(self.repetition, &after.effects_response_sha256)?;
                let bytes = canonical_bytes_v2(&receipt)?;
                self.persist_root(PublisherArtifactV2::ResidualCleanupReceipt, &bytes, None)?;
                self.publish_external_output(PublisherArtifactV2::ResidualCleanupReceipt, &bytes)?;
                Ok(bytes)
            }
            StageOutputV2::Restoration(after) => {
                let scope_resources_absent = after.wrapper_absent
                    && after.current_lock_absent
                    && after.terminal_latch_absent
                    && after.request_inbox_removed
                    && after.terminal_inbox_removed;
                let receipt = RestorationReceiptV2 {
                    schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
                    schema_version: EXPERIMENT_VERSION_V2,
                    experiment_id: EXPERIMENT_ID_V2.to_owned(),
                    repetition: self.repetition.ordinal(),
                    scope_id: self.repetition.scope_id().to_owned(),
                    complete_response_sha256: after.complete_response_sha256.clone(),
                    target_absent: after.target_absent,
                    wrong_absent: after.wrong_absent,
                    scope_resources_absent,
                    securityagent_observation_sha256: securityagent_observation_sha256.to_owned(),
                    restoration_sha256: document_sha256_v2(&after)?,
                };
                receipt.validate(self.repetition, &after.complete_response_sha256)?;
                let bytes = canonical_bytes_v2(&receipt)?;
                self.persist_root(PublisherArtifactV2::RestorationReceipt, &bytes, None)?;
                self.publish_external_output(PublisherArtifactV2::RestorationReceipt, &bytes)?;
                Ok(bytes)
            }
        }
    }

    fn create_surrogates(
        &self,
        security: &mut NonInteractiveSecurity,
        _bundle: &PublisherInputBundleV2,
    ) -> Result<Vec<u8>> {
        let prepared = self.prepared_input()?;
        let process_attestation: PublisherProcessAttestationV2 =
            self.read_root(PublisherArtifactV2::PublisherProcessAttestation)?;
        process_attestation.validate(&prepared)?;
        let process_attestation_sha256 = document_sha256_v2(&process_attestation)?;
        let fixed = fixed_repetition(self.repetition);
        let target_config = compiled_disposable_target_config(fixed)?;
        let wrong_config = compiled_disposable_wrong_config(fixed)?;
        let wrapper = PathBuf::from(wrapper_path(fixed));
        let current_lock = current_lock_path(self.repetition);
        let terminal_latch = terminal_latch_path(self.repetition);

        if let Some(existing) =
            self.try_read_root::<SurrogateCreationReceiptV2>(PublisherArtifactV2::CreationReceipt)?
        {
            existing.validate(self.repetition)?;
            existing.validate_prepared_input(&prepared)?;
            if existing.publisher_process_attestation_sha256 != process_attestation_sha256 {
                bail!("recovered creation receipt binds a different initial publisher process")
            }
            let target = security
                .read_disposable_signer(&target_config, DisposableAclKind::Target)?
                .context("recovered target signer is absent")?;
            let wrong = security
                .read_disposable_signer(&wrong_config, DisposableAclKind::WrongSurrogate)?
                .context("recovered wrong-surrogate signer is absent")?;
            if target != existing.target
                || wrong != existing.wrong
                || physical_wrapper_digest(&wrapper)? != existing.protected_wrapper_identity_sha256
                || file_observation(&current_lock)? != existing.current_lock_identity_sha256
                || file_observation(&terminal_latch)? != existing.terminal_latch_identity_sha256
            {
                bail!("recovered surrogate creation state differs from its durable receipt")
            }
            let bytes = canonical_bytes_v2(&existing)?;
            self.publish_external_output(PublisherArtifactV2::CreationReceipt, &bytes)?;
            return Ok(bytes);
        }

        if security
            .read_disposable_signer(&target_config, DisposableAclKind::Target)?
            .is_some()
            || security
                .read_disposable_signer(&wrong_config, DisposableAclKind::WrongSurrogate)?
                .is_some()
            || path_present(&wrapper)?
            || path_present(&current_lock)?
            || path_present(&terminal_latch)?
        {
            bail!("surrogate create stage did not begin from exact absence")
        }
        let before_observation_sha256 = document_sha256_v2(&AbsenceObservation {
            schema_owner: SURROGATE_OBSERVATION_OWNER,
            schema_version: EXPERIMENT_VERSION_V2,
            scope_id: self.repetition.scope_id(),
            target_absent: true,
            wrong_absent: true,
            wrapper_absent: true,
            current_lock_absent: true,
            terminal_latch_absent: true,
        })?;

        let protected_wrapper_identity_sha256 = prepare_empty_wrapper(fixed)?;
        let current_lock_identity_sha256 =
            create_or_measure_exact_file(&current_lock, CURRENT_LOCK_BYTES, 0o600)?;
        let terminal_latch_identity_sha256 =
            create_or_measure_exact_file(&terminal_latch, TERMINAL_LATCH_BYTES, 0o600)?;
        let target =
            security.create_disposable_signer(&target_config, DisposableAclKind::Target)?;
        let wrong = match security
            .create_disposable_signer(&wrong_config, DisposableAclKind::WrongSurrogate)
        {
            Ok(value) => value,
            Err(error) => {
                let rollback = security.delete_disposable_signer(
                    &target_config,
                    DisposableAclKind::Target,
                    &target.identity_sha256,
                )?;
                require_deleted(&rollback, "target rollback after wrong creation failure")?;
                return Err(error).context("create exact wrong-surrogate signer");
            }
        };
        require_disjoint_keys(&target, &wrong)?;
        let capability_pre_observation_sha256 =
            capability_observation_sha256(self.repetition.scope_id(), &target, &wrong)?;
        let quiesced_observation_sha256 = document_sha256_v2(&SurrogateSetObservation {
            schema_owner: SURROGATE_OBSERVATION_OWNER,
            schema_version: EXPERIMENT_VERSION_V2,
            scope_id: self.repetition.scope_id(),
            target_identity_sha256: &target.identity_sha256,
            wrong_identity_sha256: &wrong.identity_sha256,
            capability_pre_observation_sha256: &capability_pre_observation_sha256,
            wrapper_identity_sha256: &protected_wrapper_identity_sha256,
            current_lock_identity_sha256: &current_lock_identity_sha256,
            terminal_latch_identity_sha256: &terminal_latch_identity_sha256,
        })?;
        let receipt = SurrogateCreationReceiptV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: self.repetition.ordinal(),
            scope_id: self.repetition.scope_id().to_owned(),
            target_set_kind: TargetSetKindV2::DisposableCapability,
            publisher_identity: prepared.publisher_identity.clone(),
            publisher_process_attestation_sha256: process_attestation_sha256,
            target,
            wrong,
            capability_pre_observation_sha256,
            protected_wrapper_identity_sha256,
            current_lock_identity_sha256,
            terminal_latch_identity_sha256,
            before_observation_sha256,
            quiesced_observation_sha256,
            protected_cas_generation: 1,
            protected_cas_head_sha256: scope_digest(self.repetition, "protected-cas-genesis"),
            securityagent_baseline_sha256: prepared.securityagent_baseline_sha256.clone(),
        };
        receipt.validate(self.repetition)?;
        receipt.validate_prepared_input(&prepared)?;
        let bytes = canonical_bytes_v2(&receipt)?;
        self.persist_root(PublisherArtifactV2::CreationReceipt, &bytes, None)?;
        self.publish_external_output(PublisherArtifactV2::CreationReceipt, &bytes)?;
        Ok(bytes)
    }

    fn sign_receipt(&self, security: &NonInteractiveSecurity) -> Result<Vec<u8>> {
        let creation = self.creation_receipt()?;
        let identity: IdentityBindingPacketV2 =
            self.read_external(PublisherArtifactV2::IdentityBinding)?;
        identity.validate_against_creation(self.repetition, &creation)?;
        if let Some(existing) =
            self.try_read_root::<PublisherPreRemovalReceiptV2>(PublisherArtifactV2::SignedReceipt)?
        {
            validate_signed_receipt_output_v2(self.repetition, &creation, &identity, &existing)?;
            let bytes = canonical_bytes_v2(&existing)?;
            self.publish_external_output(PublisherArtifactV2::SignedReceipt, &bytes)?;
            return Ok(bytes);
        }
        let mut receipt: PublisherPreRemovalReceiptV2 =
            self.read_external(PublisherArtifactV2::UnsignedReceipt)?;
        let internal = internal_creation(self.repetition, &creation);
        prevalidate_unsigned_receipt(&receipt, fixed_repetition(self.repetition), &internal)?;
        if receipt.finalizer_identity != identity.finalizer_identity
            || receipt.coordinator_identity != identity.coordinator_identity
            || receipt.coordinator_process != identity.coordinator_process
            || receipt.launch_identity != identity.launch_identity
            || receipt.capability_digest != identity.capability_digest
        {
            bail!("unsigned receipt differs from the exact identity-binding packet")
        }
        let payload = substrate_common::macos_retirement_v2::signature_payload_v2(
            &receipt.signature_domain,
            &receipt.schema_owner,
            &receipt,
        )?;
        let signature = security.sign_disposable_payload(
            &compiled_disposable_target_config(fixed_repetition(self.repetition))?,
            &payload,
        )?;
        apply_signature(&mut receipt.signature, &signature, &creation.target)?;
        validate_signed_receipt_output_v2(self.repetition, &creation, &identity, &receipt)?;
        let bytes = canonical_bytes_v2(&receipt)?;
        self.persist_root(PublisherArtifactV2::SignedReceipt, &bytes, None)?;
        self.publish_external_output(PublisherArtifactV2::SignedReceipt, &bytes)?;
        Ok(bytes)
    }

    fn sign_protected_cas(&self, security: &NonInteractiveSecurity) -> Result<Vec<u8>> {
        let creation = self.creation_receipt()?;
        let receipt: PublisherPreRemovalReceiptV2 =
            self.read_root(PublisherArtifactV2::SignedReceipt)?;
        let acknowledgement: HarnessDurabilityAcknowledgementV2 =
            self.read_external(PublisherArtifactV2::HarnessAcknowledgement)?;
        substrate_common::macos_retirement_v2::validate_harness_acknowledgement_v2(
            &acknowledgement,
            &receipt,
        )?;
        if let Some(existing) =
            self.try_read_root::<ProtectedCasBindingV2>(PublisherArtifactV2::SignedProtectedCas)?
        {
            validate_signed_cas_output_v2(&receipt, &acknowledgement, &existing)?;
            let bytes = canonical_bytes_v2(&existing)?;
            fill_or_verify_wrapper(
                fixed_repetition(self.repetition),
                &creation.protected_wrapper_identity_sha256,
                &bytes,
            )?;
            self.publish_external_output(PublisherArtifactV2::SignedProtectedCas, &bytes)?;
            return Ok(bytes);
        }
        let mut binding: ProtectedCasBindingV2 =
            self.read_external(PublisherArtifactV2::UnsignedProtectedCas)?;
        prevalidate_unsigned_binding(
            &binding,
            fixed_repetition(self.repetition),
            &creation.target,
            &receipt,
            &acknowledgement,
        )?;
        let payload = substrate_common::macos_retirement_v2::signature_payload_v2(
            &binding.signature_domain,
            &binding.schema_owner,
            &binding,
        )?;
        let signature = security.sign_disposable_payload(
            &compiled_disposable_target_config(fixed_repetition(self.repetition))?,
            &payload,
        )?;
        apply_signature(&mut binding.signature, &signature, &creation.target)?;
        validate_signed_cas_output_v2(&receipt, &acknowledgement, &binding)?;
        let bytes = canonical_bytes_v2(&binding)?;
        self.persist_root(PublisherArtifactV2::SignedProtectedCas, &bytes, None)?;
        fill_or_verify_wrapper(
            fixed_repetition(self.repetition),
            &creation.protected_wrapper_identity_sha256,
            &bytes,
        )?;
        self.publish_external_output(PublisherArtifactV2::SignedProtectedCas, &bytes)?;
        Ok(bytes)
    }

    fn publish_request(&self) -> Result<Vec<u8>> {
        let request: FinalizationRequestV2 =
            self.read_external(PublisherArtifactV2::FinalizationRequest)?;
        validate_published_request_v2(&request, self.repetition.scope_id())?;
        let (receipt, acknowledgement, binding) =
            substrate_common::macos_retirement_v2::validate_finalization_request_v2(
                &request, None,
            )?;
        let expected_receipt: PublisherPreRemovalReceiptV2 =
            self.read_root(PublisherArtifactV2::SignedReceipt)?;
        let expected_binding: ProtectedCasBindingV2 =
            self.read_root(PublisherArtifactV2::SignedProtectedCas)?;
        let expected_ack: HarnessDurabilityAcknowledgementV2 =
            self.read_external(PublisherArtifactV2::HarnessAcknowledgement)?;
        if receipt != expected_receipt
            || acknowledgement != expected_ack
            || binding != expected_binding
        {
            bail!("finalization request embedded artifacts differ from fixed signed inputs")
        }
        let bytes = canonical_bytes_v2(&request)?;
        install_inbox(PublisherArtifactV2::FinalizationRequest, &bytes)?;
        self.persist_root(PublisherArtifactV2::FinalizationRequest, &bytes, None)?;
        Ok(bytes)
    }

    fn remove_residual(
        &self,
        security: &mut NonInteractiveSecurity,
    ) -> Result<ResidualAfterObservationV2> {
        let response: FinalizerResponseV2 =
            self.read_external(PublisherArtifactV2::EffectsCompleteResponse)?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(&response)?;
        let request: FinalizationRequestV2 =
            self.read_root(PublisherArtifactV2::FinalizationRequest)?;
        if response.state != FinalizerResponseStateV2::EffectsComplete
            || response.request_digest != request.request_digest
        {
            bail!("residual cleanup did not receive exact EffectsComplete for its request")
        }
        let creation = self.creation_receipt()?;
        let fixed = fixed_repetition(self.repetition);
        if security
            .read_disposable_signer(
                &compiled_disposable_target_config(fixed)?,
                DisposableAclKind::Target,
            )?
            .is_some()
        {
            bail!("target signer remains after exact EffectsComplete")
        }
        let wrong_config = compiled_disposable_wrong_config(fixed)?;
        let effects_response_sha256 = document_sha256_v2(&response)?;
        let cursor_path = self.internal_path(RESIDUAL_CURSOR_NAME);
        let existing_cursor = read_exact_path(&cursor_path, 0, ROOT_FILE_MODE)?
            .map(|bytes| parse_canonical_v2::<ResidualCursorV2>(&bytes))
            .transpose()?;
        let observed_wrong =
            security.read_disposable_signer(&wrong_config, DisposableAclKind::WrongSurrogate)?;
        if let Some(wrong) = &observed_wrong {
            if wrong != &creation.wrong {
                bail!("wrong surrogate identity changed before post-effects cleanup")
            }
        }

        let cursor = if let Some(cursor) = existing_cursor {
            validate_residual_cursor(
                &cursor,
                self.repetition,
                &effects_response_sha256,
                &creation.wrong.identity_sha256,
            )?;
            cursor
        } else {
            if observed_wrong.is_none() {
                bail!("wrong surrogate disappeared before a durable cleanup cursor")
            }
            let cursor = ResidualCursorV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
                schema_version: EXPERIMENT_VERSION_V2,
                repetition: self.repetition.ordinal(),
                scope_id: self.repetition.scope_id().to_owned(),
                effects_response_sha256: effects_response_sha256.clone(),
                wrong_identity_sha256: creation.wrong.identity_sha256.clone(),
                phase: ResidualCursorPhaseV2::Prepared,
            };
            self.write_residual_cursor(&cursor, None)?;
            cursor
        };

        match cursor.phase {
            ResidualCursorPhaseV2::Prepared => {
                if observed_wrong.is_none() {
                    bail!("Prepared residual cursor found wrong surrogate already absent")
                }
                let invoked = ResidualCursorV2 {
                    phase: ResidualCursorPhaseV2::Invoked,
                    ..cursor.clone()
                };
                self.write_residual_cursor(&invoked, Some(&cursor))?;
                let exact_delete = security.delete_disposable_signer(
                    &wrong_config,
                    DisposableAclKind::WrongSurrogate,
                    &creation.wrong.identity_sha256,
                )?;
                require_deleted(&exact_delete, "post-effects wrong-surrogate cleanup")?;
            }
            ResidualCursorPhaseV2::Invoked => {
                if observed_wrong.is_some() {
                    bail!(
                        "Invoked residual cleanup remains present and must never be blindly reinvoked"
                    )
                }
            }
        }
        if security
            .read_disposable_signer(&wrong_config, DisposableAclKind::WrongSurrogate)?
            .is_some()
        {
            bail!("wrong surrogate remains after exact residual cleanup")
        }
        let wrapper_absent = !path_present(Path::new(&wrapper_path(fixed)))?;
        let current_lock_absent = !path_present(&current_lock_path(self.repetition))?;
        let terminal_latch_absent = !path_present(&terminal_latch_path(self.repetition))?;
        if !(wrapper_absent && current_lock_absent && terminal_latch_absent) {
            bail!("EffectsComplete left disposable wrapper/lock/latch state present")
        }
        let exact_after_observation_sha256 = document_sha256_v2(&AbsenceObservation {
            schema_owner: SURROGATE_OBSERVATION_OWNER,
            schema_version: EXPERIMENT_VERSION_V2,
            scope_id: self.repetition.scope_id(),
            target_absent: true,
            wrong_absent: true,
            wrapper_absent,
            current_lock_absent,
            terminal_latch_absent,
        })?;
        if exact_after_observation_sha256 != creation.before_observation_sha256 {
            bail!("residual cleanup did not restore the exact creation baseline")
        }
        Ok(ResidualAfterObservationV2 {
            effects_response_sha256,
            target_absent: true,
            wrong_absent: true,
            wrong_identity_sha256: creation.wrong.identity_sha256,
            exact_after_observation_sha256,
        })
    }

    fn reconstruct_residual_after_absence(
        &self,
        security: &NonInteractiveSecurity,
    ) -> Result<ResidualAfterObservationV2> {
        let response: FinalizerResponseV2 =
            self.read_external(PublisherArtifactV2::EffectsCompleteResponse)?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(&response)?;
        let request: FinalizationRequestV2 =
            self.read_root(PublisherArtifactV2::FinalizationRequest)?;
        if response.state != FinalizerResponseStateV2::EffectsComplete
            || response.request_digest != request.request_digest
        {
            bail!("residual recovery lacks exact EffectsComplete authority")
        }
        let creation = self.creation_receipt()?;
        let fixed = fixed_repetition(self.repetition);
        let target_absent = security
            .read_disposable_signer(
                &compiled_disposable_target_config(fixed)?,
                DisposableAclKind::Target,
            )?
            .is_none();
        let wrong_absent = security
            .read_disposable_signer(
                &compiled_disposable_wrong_config(fixed)?,
                DisposableAclKind::WrongSurrogate,
            )?
            .is_none();
        let wrapper_absent = !path_present(Path::new(&wrapper_path(fixed)))?;
        let current_lock_absent = !path_present(&current_lock_path(self.repetition))?;
        let terminal_latch_absent = !path_present(&terminal_latch_path(self.repetition))?;
        if !(target_absent
            && wrong_absent
            && wrapper_absent
            && current_lock_absent
            && terminal_latch_absent)
        {
            bail!("observed residual stage is ambiguous; exact absence is not established")
        }
        let exact_after_observation_sha256 = document_sha256_v2(&AbsenceObservation {
            schema_owner: SURROGATE_OBSERVATION_OWNER,
            schema_version: EXPERIMENT_VERSION_V2,
            scope_id: self.repetition.scope_id(),
            target_absent,
            wrong_absent,
            wrapper_absent,
            current_lock_absent,
            terminal_latch_absent,
        })?;
        if exact_after_observation_sha256 != creation.before_observation_sha256 {
            bail!("residual recovery exact absence differs from creation baseline")
        }
        Ok(ResidualAfterObservationV2 {
            effects_response_sha256: document_sha256_v2(&response)?,
            target_absent,
            wrong_absent,
            wrong_identity_sha256: creation.wrong.identity_sha256,
            exact_after_observation_sha256,
        })
    }

    fn write_residual_cursor(
        &self,
        cursor: &ResidualCursorV2,
        predecessor: Option<&ResidualCursorV2>,
    ) -> Result<()> {
        let predecessor_bytes = predecessor.map(canonical_bytes_v2).transpose()?;
        write_exact_path(
            &self.internal_path(RESIDUAL_CURSOR_NAME),
            &canonical_bytes_v2(cursor)?,
            0o600,
            predecessor_bytes.as_deref(),
            ParentOwnerV2::Root,
        )
    }

    fn handle_emergency_if_present(
        &self,
        security: &mut NonInteractiveSecurity,
        progress: &PublisherProgressV2,
    ) -> Result<bool> {
        let precreation = self.try_read_external::<PreCreationEmergencyRollbackMarkerV2>(
            PublisherArtifactV2::PreCreationEmergencyRollbackMarker,
        )?;
        let completed = self.try_read_external::<EmergencyRollbackMarkerV2>(
            PublisherArtifactV2::EmergencyRollbackMarker,
        )?;
        if precreation.is_some() && completed.is_some() {
            bail!("both emergency rollback marker forms are present")
        }
        let marker_sha256 = match (&precreation, &completed) {
            (None, None) => return Ok(false),
            (Some(marker), None) => {
                marker.validate(self.repetition)?;
                document_sha256_v2(marker)?
            }
            (None, Some(marker)) => {
                let creation = self.creation_receipt()?;
                marker.validate(self.repetition, &creation)?;
                if marker.last_durable_stage != progress.stage {
                    bail!("emergency rollback marker differs from durable publisher progress")
                }
                document_sha256_v2(marker)?
            }
            (Some(_), Some(_)) => unreachable!(),
        };
        if progress.stage >= PublisherStageV2::FinalizationRequestPublished {
            bail!(
                "publisher refuses emergency mutation after request publication/possible acceptance"
            )
        }
        if let Some(existing) = self.try_read_root::<EmergencyRollbackReceiptV2>(
            PublisherArtifactV2::EmergencyRollbackReceipt,
        )? {
            match (&precreation, &completed) {
                (Some(marker), None) => existing.validate_precreation(self.repetition, marker)?,
                (None, Some(marker)) => existing.validate(self.repetition, marker)?,
                _ => unreachable!(),
            }
            self.publish_external_output(
                PublisherArtifactV2::EmergencyRollbackReceipt,
                &canonical_bytes_v2(&existing)?,
            )?;
            return Ok(true);
        }

        let creation =
            self.try_read_root::<SurrogateCreationReceiptV2>(PublisherArtifactV2::CreationReceipt)?;
        if let Some(value) = &creation {
            value.validate(self.repetition)?;
        }
        let observation_path = self.internal_path(EMERGENCY_UI_OBSERVATION_NAME);
        let (restoration, observation) =
            if let Some(bytes) = read_exact_path(&observation_path, 0, ROOT_FILE_MODE)? {
                let observation: PublisherSecurityAgentObservationV2 = parse_canonical_v2(&bytes)?;
                observation.validate(self.repetition, progress.stage)?;
                // A completed immutable observer record means the earlier mutation arm returned.
                // Recovery never invokes deletion again. Its fresh readback gets a distinct raw
                // observer record rather than borrowing the pre-crash mutation observation.
                let prepared = self.prepared_input()?;
                let (restoration, recovery_observation) = observe_stage(
                    self.repetition,
                    progress.stage,
                    &prepared.securityagent_baseline_sha256,
                    || self.observe_exact_scope_absence(security),
                    |alert| self.persist_terminal_alert(progress.stage, alert),
                )?;
                self.persist_recovery_observation(
                    progress.stage,
                    "emergency-readback",
                    &recovery_observation,
                )?;
                (restoration?, recovery_observation)
            } else {
                let prepared = self.prepared_input()?;
                let current_attestation = attest_publisher_process(security, &prepared)?;
                let prearm = document_sha256_v2(&current_attestation)?;
                let (restoration, observation) = observe_stage(
                    self.repetition,
                    progress.stage,
                    &prearm,
                    || self.rollback_exact_scope(security, creation.as_ref()),
                    |alert| self.persist_terminal_alert(progress.stage, alert),
                )?;
                write_exact_path(
                    &observation_path,
                    &canonical_bytes_v2(&observation)?,
                    0o600,
                    None,
                    ParentOwnerV2::Root,
                )?;
                (restoration?, observation)
            };
        let observation_sha256 = document_sha256_v2(&observation)?;
        if !path_present(&observation_path)? {
            bail!("emergency rollback lost its durable SecurityAgent observation")
        }
        let receipt = EmergencyRollbackReceiptV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: self.repetition.ordinal(),
            scope_id: self.repetition.scope_id().to_owned(),
            rollback_marker_sha256: marker_sha256,
            target_absent: restoration.target_absent,
            wrong_absent: restoration.wrong_absent,
            scope_resources_absent: restoration.wrapper_absent
                && restoration.current_lock_absent
                && restoration.terminal_latch_absent
                && restoration.accepted_journal_absent
                && restoration.request_inbox_absent
                && restoration.terminal_inbox_absent,
            securityagent_observation_sha256: observation_sha256,
            restoration_sha256: document_sha256_v2(&restoration)?,
        };
        match (&precreation, &completed) {
            (Some(marker), None) => receipt.validate_precreation(self.repetition, marker)?,
            (None, Some(marker)) => receipt.validate(self.repetition, marker)?,
            _ => unreachable!(),
        }
        let bytes = canonical_bytes_v2(&receipt)?;
        self.persist_root(PublisherArtifactV2::EmergencyRollbackReceipt, &bytes, None)?;
        self.publish_external_output(PublisherArtifactV2::EmergencyRollbackReceipt, &bytes)?;
        Ok(true)
    }

    fn rollback_exact_scope(
        &self,
        security: &mut NonInteractiveSecurity,
        creation: Option<&SurrogateCreationReceiptV2>,
    ) -> Result<EmergencyRestorationObservationV2> {
        let fixed = fixed_repetition(self.repetition);
        for (config, kind, expected) in [
            (
                compiled_disposable_target_config(fixed)?,
                DisposableAclKind::Target,
                creation.map(|value| &value.target),
            ),
            (
                compiled_disposable_wrong_config(fixed)?,
                DisposableAclKind::WrongSurrogate,
                creation.map(|value| &value.wrong),
            ),
        ] {
            if let Some(observed) = security.read_disposable_signer(&config, kind)? {
                if expected.is_some_and(|expected| expected != &observed) {
                    bail!("emergency rollback found a substituted exact-tag signer")
                }
                let deleted =
                    security.delete_disposable_signer(&config, kind, &observed.identity_sha256)?;
                require_deleted(&deleted, "emergency exact-ref signer rollback")?;
            }
            if security.read_disposable_signer(&config, kind)?.is_some() {
                bail!("emergency rollback signer remains present")
            }
        }

        let wrapper = PathBuf::from(wrapper_path(fixed));
        remove_exact_scope_file(
            &wrapper,
            libc::S_IFREG | 0o400,
            creation.map(|value| value.protected_wrapper_identity_sha256.as_str()),
            ScopeFileKindV2::Wrapper,
        )?;
        remove_exact_scope_file(
            &current_lock_path(self.repetition),
            LOCK_FILE_MODE,
            creation.map(|value| value.current_lock_identity_sha256.as_str()),
            ScopeFileKindV2::CurrentLock,
        )?;
        remove_exact_scope_file(
            &terminal_latch_path(self.repetition),
            LOCK_FILE_MODE,
            creation.map(|value| value.terminal_latch_identity_sha256.as_str()),
            ScopeFileKindV2::TerminalLatch,
        )?;

        self.observe_exact_scope_absence(security)
    }

    fn observe_exact_scope_absence(
        &self,
        security: &NonInteractiveSecurity,
    ) -> Result<EmergencyRestorationObservationV2> {
        let fixed = fixed_repetition(self.repetition);
        let target_absent = security
            .read_disposable_signer(
                &compiled_disposable_target_config(fixed)?,
                DisposableAclKind::Target,
            )?
            .is_none();
        let wrong_absent = security
            .read_disposable_signer(
                &compiled_disposable_wrong_config(fixed)?,
                DisposableAclKind::WrongSurrogate,
            )?
            .is_none();
        let wrapper = PathBuf::from(wrapper_path(fixed));
        let journal = Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(self.repetition.scope_id());
        let request_inbox_absent = !path_present(installed_finalizer_inbox_path_v2(
            PublisherArtifactV2::FinalizationRequest,
        )?)?;
        let terminal_inbox_absent = !path_present(installed_finalizer_inbox_path_v2(
            PublisherArtifactV2::TerminalBinding,
        )?)?;
        let value = EmergencyRestorationObservationV2 {
            repetition: self.repetition.ordinal(),
            scope_id: self.repetition.scope_id().to_owned(),
            target_absent,
            wrong_absent,
            wrapper_absent: !path_present(&wrapper)?,
            current_lock_absent: !path_present(&current_lock_path(self.repetition))?,
            terminal_latch_absent: !path_present(&terminal_latch_path(self.repetition))?,
            accepted_journal_absent: !path_present(&journal)?,
            request_inbox_absent,
            terminal_inbox_absent,
        };
        if !value.target_absent
            || !value.wrong_absent
            || !value.wrapper_absent
            || !value.current_lock_absent
            || !value.terminal_latch_absent
            || !value.accepted_journal_absent
            || !value.request_inbox_absent
            || !value.terminal_inbox_absent
        {
            bail!("emergency rollback did not restore exact pre-request absence")
        }
        Ok(value)
    }

    fn handle_transport_checkpoint_if_present(
        &self,
        security: &mut NonInteractiveSecurity,
        progress: &PublisherProgressV2,
    ) -> Result<bool> {
        if progress.stage != PublisherStageV2::FinalizationRequestPublished {
            return Ok(true);
        }
        let Some(marker) = self.try_read_external::<TransportControlsCompleteMarkerV2>(
            PublisherArtifactV2::TransportControlsCompleteMarker,
        )?
        else {
            return Ok(false);
        };
        marker.validate_shape(self.repetition)?;
        let request: FinalizationRequestV2 =
            self.read_root(PublisherArtifactV2::FinalizationRequest)?;
        let creation = self.creation_receipt()?;
        if marker.request_digest != request.request_digest
            || marker.before_observation_sha256 != creation.quiesced_observation_sha256
        {
            bail!("transport checkpoint does not bind request and quiesced surrogate state")
        }
        if let Some(existing) = self.try_read_root::<TransportControlsObservationReceiptV2>(
            PublisherArtifactV2::TransportControlsObservation,
        )? {
            existing.validate(self.repetition, &marker)?;
            self.publish_external_output(
                PublisherArtifactV2::TransportControlsObservation,
                &canonical_bytes_v2(&existing)?,
            )?;
            return Ok(true);
        }

        let ui_path = self.internal_path(TRANSPORT_UI_OBSERVATION_NAME);
        let (after_observation_sha256, ui_observation) =
            if let Some(bytes) = read_exact_path(&ui_path, 0, ROOT_FILE_MODE)? {
                let observation: PublisherSecurityAgentObservationV2 = parse_canonical_v2(&bytes)?;
                observation.validate(
                    self.repetition,
                    PublisherStageV2::FinalizationRequestPublished,
                )?;
                // Recovery from the immutable observer result is read-only and never reruns a
                // native control. Strict same-state observation gets a fresh raw observer record.
                let prepared = self.prepared_input()?;
                let (after, recovery_observation) = observe_stage(
                    self.repetition,
                    PublisherStageV2::FinalizationRequestPublished,
                    &prepared.securityagent_baseline_sha256,
                    || self.observe_unchanged_transport_state(security, &creation),
                    |alert| {
                        self.persist_terminal_alert(
                            PublisherStageV2::FinalizationRequestPublished,
                            alert,
                        )
                    },
                )?;
                self.persist_recovery_observation(
                    PublisherStageV2::FinalizationRequestPublished,
                    "transport-readback",
                    &recovery_observation,
                )?;
                (after?, recovery_observation)
            } else {
                let prepared = self.prepared_input()?;
                let attestation = attest_publisher_process(security, &prepared)?;
                let prearm = document_sha256_v2(&attestation)?;
                let (after, observation) = observe_stage(
                    self.repetition,
                    PublisherStageV2::FinalizationRequestPublished,
                    &prearm,
                    || self.observe_unchanged_transport_state(security, &creation),
                    |alert| {
                        self.persist_terminal_alert(
                            PublisherStageV2::FinalizationRequestPublished,
                            alert,
                        )
                    },
                )?;
                write_exact_path(
                    &ui_path,
                    &canonical_bytes_v2(&observation)?,
                    0o600,
                    None,
                    ParentOwnerV2::Root,
                )?;
                (after?, observation)
            };
        if after_observation_sha256 != marker.before_observation_sha256 {
            bail!("transport controls changed exact surrogate physical/key state")
        }
        let accepted_scope =
            Path::new(MAC_R3_FINALIZER_JOURNAL_ROOT_V2).join(self.repetition.scope_id());
        let accepted_journal_absent = !path_present(&accepted_scope)?;
        if !accepted_journal_absent {
            bail!("transport controls unexpectedly created an accepted finalizer journal")
        }
        let ui_sha256 = document_sha256_v2(&ui_observation)?;
        if !path_present(&ui_path)? {
            bail!("transport checkpoint lost its durable SecurityAgent observation")
        }
        let receipt = TransportControlsObservationReceiptV2 {
            schema_owner: TRANSPORT_CONTROL_RECEIPT_OWNER_V2.to_owned(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_owned(),
            repetition: self.repetition.ordinal(),
            scope_id: self.repetition.scope_id().to_owned(),
            marker_sha256: document_sha256_v2(&marker)?,
            after_observation_sha256,
            accepted_journal_absent,
            exact_target_state_unchanged: true,
            securityagent_observation_sha256: ui_sha256,
        };
        receipt.validate(self.repetition, &marker)?;
        let bytes = canonical_bytes_v2(&receipt)?;
        self.persist_root(
            PublisherArtifactV2::TransportControlsObservation,
            &bytes,
            None,
        )?;
        self.publish_external_output(PublisherArtifactV2::TransportControlsObservation, &bytes)?;
        Ok(true)
    }

    fn observe_unchanged_transport_state(
        &self,
        security: &NonInteractiveSecurity,
        creation: &SurrogateCreationReceiptV2,
    ) -> Result<String> {
        let fixed = fixed_repetition(self.repetition);
        let target = security
            .read_disposable_signer(
                &compiled_disposable_target_config(fixed)?,
                DisposableAclKind::Target,
            )?
            .context("transport checkpoint target signer is absent")?;
        let wrong = security
            .read_disposable_signer(
                &compiled_disposable_wrong_config(fixed)?,
                DisposableAclKind::WrongSurrogate,
            )?
            .context("transport checkpoint wrong signer is absent")?;
        if target != creation.target || wrong != creation.wrong {
            bail!("transport checkpoint signer identity changed")
        }
        let wrapper_identity_sha256 = physical_wrapper_digest(Path::new(&wrapper_path(fixed)))?;
        let current_lock_identity_sha256 = file_observation(&current_lock_path(self.repetition))?;
        let terminal_latch_identity_sha256 =
            file_observation(&terminal_latch_path(self.repetition))?;
        if wrapper_identity_sha256 != creation.protected_wrapper_identity_sha256
            || current_lock_identity_sha256 != creation.current_lock_identity_sha256
            || terminal_latch_identity_sha256 != creation.terminal_latch_identity_sha256
        {
            bail!("transport checkpoint wrapper/lock/latch physical identity changed")
        }
        document_sha256_v2(&SurrogateSetObservation {
            schema_owner: SURROGATE_OBSERVATION_OWNER,
            schema_version: EXPERIMENT_VERSION_V2,
            scope_id: self.repetition.scope_id(),
            target_identity_sha256: &target.identity_sha256,
            wrong_identity_sha256: &wrong.identity_sha256,
            capability_pre_observation_sha256: &creation.capability_pre_observation_sha256,
            wrapper_identity_sha256: &wrapper_identity_sha256,
            current_lock_identity_sha256: &current_lock_identity_sha256,
            terminal_latch_identity_sha256: &terminal_latch_identity_sha256,
        })
    }

    fn publish_terminal_binding(&self) -> Result<Vec<u8>> {
        let terminal: TerminalAcknowledgementV2 =
            self.read_external(PublisherArtifactV2::TerminalAcknowledgement)?;
        validate_terminal_publication_v2(&terminal, self.repetition.scope_id())?;
        let binding: TerminalBindingRequest =
            self.read_external(PublisherArtifactV2::TerminalBinding)?;
        if binding.schema_owner != TERMINAL_BINDING_OWNER
            || binding.schema_version != TERMINAL_BINDING_VERSION
            || binding.request_digest != terminal.request_digest
        {
            bail!("terminal binding header differs from the exact acknowledgement")
        }
        let embedded_terminal: TerminalAcknowledgementV2 =
            parse_canonical_v2(&URL_SAFE_NO_PAD.decode(&binding.terminal_acknowledgement)?)?;
        if embedded_terminal != terminal {
            bail!("terminal binding embeds a different terminal acknowledgement")
        }
        let request: FinalizationRequestV2 =
            parse_canonical_v2(&URL_SAFE_NO_PAD.decode(&binding.authority_request)?)?;
        let effects: FinalizerResponseV2 =
            parse_canonical_v2(&URL_SAFE_NO_PAD.decode(&binding.effects_response)?)?;
        let parity: HostParityProofV2 =
            parse_canonical_v2(&URL_SAFE_NO_PAD.decode(&binding.parity_proof)?)?;
        let signed_receipt: PublisherPreRemovalReceiptV2 =
            self.read_root(PublisherArtifactV2::SignedReceipt)?;
        substrate_common::macos_retirement_v2::validate_host_parity_proof_v2(
            &parity,
            &signed_receipt.harness_public_key,
        )?;
        substrate_common::macos_retirement_v2::validate_terminal_acknowledgement_v2(
            &terminal,
            &parity,
            &signed_receipt.harness_public_key,
        )?;
        if request != self.read_root(PublisherArtifactV2::FinalizationRequest)?
            || effects.state != FinalizerResponseStateV2::EffectsComplete
            || effects.request_digest != request.request_digest
        {
            bail!("terminal binding does not exact-bind the accepted request and EffectsComplete")
        }
        let bytes = canonical_bytes_v2(&binding)?;
        install_inbox(PublisherArtifactV2::TerminalBinding, &bytes)?;
        self.persist_root(PublisherArtifactV2::TerminalBinding, &bytes, None)?;
        Ok(bytes)
    }

    fn finish_restoration(
        &self,
        security: &mut NonInteractiveSecurity,
    ) -> Result<RestorationAfterObservationV2> {
        let complete: FinalizerResponseV2 =
            self.read_external(PublisherArtifactV2::CompleteResponse)?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(&complete)?;
        let request: FinalizationRequestV2 =
            self.read_root(PublisherArtifactV2::FinalizationRequest)?;
        if complete.state != FinalizerResponseStateV2::HostComplete
            || complete.request_digest != request.request_digest
        {
            bail!("restoration did not receive exact Complete for its request")
        }
        let fixed = fixed_repetition(self.repetition);
        let target_absent = security
            .read_disposable_signer(
                &compiled_disposable_target_config(fixed)?,
                DisposableAclKind::Target,
            )?
            .is_none();
        let wrong_absent = security
            .read_disposable_signer(
                &compiled_disposable_wrong_config(fixed)?,
                DisposableAclKind::WrongSurrogate,
            )?
            .is_none();
        let wrapper_absent = !path_present(Path::new(&wrapper_path(fixed)))?;
        let current_lock_absent = !path_present(&current_lock_path(self.repetition))?;
        let terminal_latch_absent = !path_present(&terminal_latch_path(self.repetition))?;
        if !(target_absent
            && wrong_absent
            && wrapper_absent
            && current_lock_absent
            && terminal_latch_absent)
        {
            bail!("restoration found a surviving exact surrogate target")
        }
        let terminal: TerminalBindingRequest =
            self.read_root(PublisherArtifactV2::TerminalBinding)?;
        let request_bytes = canonical_bytes_v2(&request)?;
        let terminal_bytes = canonical_bytes_v2(&terminal)?;
        let complete_response_sha256 = document_sha256_v2(&complete)?;
        let cursor_path = self.internal_path(RESTORATION_CURSOR_NAME);
        let cursor = match read_exact_path(&cursor_path, 0, ROOT_FILE_MODE)? {
            Some(bytes) => {
                let cursor: RestorationCursorV2 = parse_canonical_v2(&bytes)?;
                validate_restoration_cursor(
                    &cursor,
                    self.repetition,
                    &complete_response_sha256,
                    &sha256_hex_v2(&request_bytes),
                    &sha256_hex_v2(&terminal_bytes),
                )?;
                cursor
            }
            None => {
                if !path_present(installed_finalizer_inbox_path_v2(
                    PublisherArtifactV2::FinalizationRequest,
                )?)? || !path_present(installed_finalizer_inbox_path_v2(
                    PublisherArtifactV2::TerminalBinding,
                )?)? {
                    bail!("restoration inbox changed before its durable cursor")
                }
                let cursor = RestorationCursorV2 {
                    schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
                    schema_version: EXPERIMENT_VERSION_V2,
                    repetition: self.repetition.ordinal(),
                    scope_id: self.repetition.scope_id().to_owned(),
                    complete_response_sha256: complete_response_sha256.clone(),
                    request_sha256: sha256_hex_v2(&request_bytes),
                    terminal_sha256: sha256_hex_v2(&terminal_bytes),
                    phase: RestorationCursorPhaseV2::Prepared,
                };
                self.write_restoration_cursor(&cursor, None)?;
                cursor
            }
        };
        let cursor = self.advance_restoration_request(cursor, &request_bytes)?;
        let cursor = self.advance_restoration_terminal(cursor, &terminal_bytes)?;
        if cursor.phase != RestorationCursorPhaseV2::TerminalObserved {
            bail!("restoration cursor did not reach exact terminal observation")
        }
        Ok(RestorationAfterObservationV2 {
            complete_response_sha256,
            target_absent,
            wrong_absent,
            wrapper_absent,
            current_lock_absent,
            terminal_latch_absent,
            request_inbox_removed: true,
            terminal_inbox_removed: true,
        })
    }

    fn advance_restoration_request(
        &self,
        mut cursor: RestorationCursorV2,
        expected: &[u8],
    ) -> Result<RestorationCursorV2> {
        if cursor.phase == RestorationCursorPhaseV2::Prepared {
            let next = RestorationCursorV2 {
                phase: RestorationCursorPhaseV2::RequestInvoked,
                ..cursor.clone()
            };
            self.write_restoration_cursor(&next, Some(&cursor))?;
            cursor = next;
        }
        if cursor.phase == RestorationCursorPhaseV2::RequestInvoked {
            let path = installed_finalizer_inbox_path_v2(PublisherArtifactV2::FinalizationRequest)?;
            if path_present(path)? {
                remove_installed_inbox(PublisherArtifactV2::FinalizationRequest, expected)?;
            }
            if path_present(path)? {
                bail!("request inbox remains after cursor-bound exact removal")
            }
            let next = RestorationCursorV2 {
                phase: RestorationCursorPhaseV2::RequestObserved,
                ..cursor.clone()
            };
            self.write_restoration_cursor(&next, Some(&cursor))?;
            cursor = next;
        }
        Ok(cursor)
    }

    fn advance_restoration_terminal(
        &self,
        mut cursor: RestorationCursorV2,
        expected: &[u8],
    ) -> Result<RestorationCursorV2> {
        if cursor.phase == RestorationCursorPhaseV2::RequestObserved {
            let next = RestorationCursorV2 {
                phase: RestorationCursorPhaseV2::TerminalInvoked,
                ..cursor.clone()
            };
            self.write_restoration_cursor(&next, Some(&cursor))?;
            cursor = next;
        }
        if cursor.phase == RestorationCursorPhaseV2::TerminalInvoked {
            let path = installed_finalizer_inbox_path_v2(PublisherArtifactV2::TerminalBinding)?;
            if path_present(path)? {
                remove_installed_inbox(PublisherArtifactV2::TerminalBinding, expected)?;
            }
            if path_present(path)? {
                bail!("terminal inbox remains after cursor-bound exact removal")
            }
            let next = RestorationCursorV2 {
                phase: RestorationCursorPhaseV2::TerminalObserved,
                ..cursor.clone()
            };
            self.write_restoration_cursor(&next, Some(&cursor))?;
            cursor = next;
        }
        Ok(cursor)
    }

    fn write_restoration_cursor(
        &self,
        cursor: &RestorationCursorV2,
        predecessor: Option<&RestorationCursorV2>,
    ) -> Result<()> {
        let predecessor_bytes = predecessor.map(canonical_bytes_v2).transpose()?;
        write_exact_path(
            &self.internal_path(RESTORATION_CURSOR_NAME),
            &canonical_bytes_v2(cursor)?,
            0o600,
            predecessor_bytes.as_deref(),
            ParentOwnerV2::Root,
        )
    }

    fn reconstruct_restoration_after_absence(
        &self,
        security: &NonInteractiveSecurity,
    ) -> Result<RestorationAfterObservationV2> {
        let complete: FinalizerResponseV2 =
            self.read_external(PublisherArtifactV2::CompleteResponse)?;
        substrate_common::macos_retirement_v2::validate_finalizer_response_v2(&complete)?;
        let request: FinalizationRequestV2 =
            self.read_root(PublisherArtifactV2::FinalizationRequest)?;
        if complete.state != FinalizerResponseStateV2::HostComplete
            || complete.request_digest != request.request_digest
        {
            bail!("restoration recovery lacks exact HostComplete authority")
        }
        let fixed = fixed_repetition(self.repetition);
        let target_absent = security
            .read_disposable_signer(
                &compiled_disposable_target_config(fixed)?,
                DisposableAclKind::Target,
            )?
            .is_none();
        let wrong_absent = security
            .read_disposable_signer(
                &compiled_disposable_wrong_config(fixed)?,
                DisposableAclKind::WrongSurrogate,
            )?
            .is_none();
        let wrapper_absent = !path_present(Path::new(&wrapper_path(fixed)))?;
        let current_lock_absent = !path_present(&current_lock_path(self.repetition))?;
        let terminal_latch_absent = !path_present(&terminal_latch_path(self.repetition))?;
        let request_inbox_removed = !path_present(installed_finalizer_inbox_path_v2(
            PublisherArtifactV2::FinalizationRequest,
        )?)?;
        let terminal_inbox_removed = !path_present(installed_finalizer_inbox_path_v2(
            PublisherArtifactV2::TerminalBinding,
        )?)?;
        if !(target_absent
            && wrong_absent
            && wrapper_absent
            && current_lock_absent
            && terminal_latch_absent
            && request_inbox_removed
            && terminal_inbox_removed)
        {
            bail!("observed restoration stage is ambiguous; exact absence is not established")
        }
        Ok(RestorationAfterObservationV2 {
            complete_response_sha256: document_sha256_v2(&complete)?,
            target_absent,
            wrong_absent,
            wrapper_absent,
            current_lock_absent,
            terminal_latch_absent,
            request_inbox_removed,
            terminal_inbox_removed,
        })
    }

    fn wait_for_transition(
        &self,
        prior: PublisherStageV2,
        next: PublisherStageV2,
    ) -> Result<(PublisherAdvanceMarkerV2, PublisherInputBundleV2)> {
        loop {
            let marker = self.try_read_external::<PublisherAdvanceMarkerV2>(
                PublisherArtifactV2::AdvanceMarker(prior),
            )?;
            let bundle = self.try_read_external::<PublisherInputBundleV2>(
                PublisherArtifactV2::InputBundle(next),
            )?;
            match (marker, bundle) {
                (None, None) | (Some(_), None) | (None, Some(_)) => thread::sleep(POLL_INTERVAL),
                (Some(marker), Some(bundle)) => {
                    if marker.prior_stage != prior || marker.next_stage != next {
                        bail!("publisher advance marker crossed its current fixed stage")
                    }
                    marker.validate_input_bundle(self.repetition, &bundle)?;
                    self.verify_bundle_artifacts(&bundle)?;
                    return Ok((marker, bundle));
                }
            }
        }
    }

    fn verify_bundle_artifacts(&self, bundle: &PublisherInputBundleV2) -> Result<()> {
        let expected = expected_input_artifacts_v2(bundle.next_stage);
        if bundle.artifacts.len() != expected.len() {
            bail!("publisher input bundle artifact count changed")
        }
        for PublisherInputArtifactDigestV2 { artifact, sha256 } in &bundle.artifacts {
            let bytes = self.read_external_bytes(input_artifact(*artifact))?;
            if sha256_hex_v2(&bytes) != *sha256 {
                bail!("publisher input artifact bytes differ from the bound bundle digest")
            }
        }
        Ok(())
    }

    fn prepared_input(&self) -> Result<PublisherPreparedInputV2> {
        let value: PublisherPreparedInputV2 =
            self.read_external(PublisherArtifactV2::PreparedInput)?;
        value.validate()?;
        Ok(value)
    }

    fn creation_receipt(&self) -> Result<SurrogateCreationReceiptV2> {
        let value: SurrogateCreationReceiptV2 =
            self.read_root(PublisherArtifactV2::CreationReceipt)?;
        value.validate(self.repetition)?;
        value.validate_prepared_input(&self.prepared_input()?)?;
        Ok(value)
    }

    fn read_progress(&self) -> Result<PublisherProgressV2> {
        let value: PublisherProgressV2 = self.read_root(PublisherArtifactV2::Progress)?;
        value.validate(self.repetition)?;
        Ok(value)
    }

    fn ensure_initial_progress(&self) -> Result<()> {
        if self
            .try_read_root::<PublisherProgressV2>(PublisherArtifactV2::Progress)?
            .is_some()
        {
            let progress = self.read_progress()?;
            return self.recover_external_progress(&progress);
        }
        let progress = PublisherProgressV2::genesis(self.repetition);
        progress.validate(self.repetition)?;
        self.persist_root(
            PublisherArtifactV2::Progress,
            &canonical_bytes_v2(&progress)?,
            None,
        )?;
        self.publish_external_progress(&progress, None)
    }

    fn replace_progress(
        &self,
        predecessor: &PublisherProgressV2,
        progress: &PublisherProgressV2,
    ) -> Result<()> {
        let predecessor_bytes = canonical_bytes_v2(predecessor)?;
        self.persist_root(
            PublisherArtifactV2::Progress,
            &canonical_bytes_v2(progress)?,
            Some(&predecessor_bytes),
        )?;
        self.publish_external_progress(progress, Some(predecessor))
    }

    fn publish_external_progress(
        &self,
        progress: &PublisherProgressV2,
        predecessor: Option<&PublisherProgressV2>,
    ) -> Result<()> {
        let predecessor_bytes = predecessor.map(canonical_bytes_v2).transpose()?;
        write_exact_path(
            &external_exchange_path_v2(self.repetition, PublisherArtifactV2::Progress),
            &canonical_bytes_v2(progress)?,
            0o444,
            predecessor_bytes.as_deref(),
            ParentOwnerV2::Harness,
        )
    }

    fn recover_external_progress(&self, progress: &PublisherProgressV2) -> Result<()> {
        let path = external_exchange_path_v2(self.repetition, PublisherArtifactV2::Progress);
        let Some(existing) = read_exact_path(&path, 0, EXTERNAL_OUTPUT_MODE)? else {
            return self.publish_external_progress(progress, None);
        };
        let expected = canonical_bytes_v2(progress)?;
        if existing == expected {
            return self.publish_external_progress(progress, None);
        }
        if sha256_hex_v2(&existing) != progress.predecessor_progress_sha256 {
            bail!("external progress is neither the exact predecessor nor successor")
        }
        let predecessor: PublisherProgressV2 = parse_canonical_v2(&existing)?;
        predecessor.validate(self.repetition)?;
        self.publish_external_progress(progress, Some(&predecessor))
    }

    fn read_external<T: DeserializeOwned + Serialize>(
        &self,
        artifact: PublisherArtifactV2,
    ) -> Result<T> {
        parse_canonical_v2(&self.read_external_bytes(artifact)?)
    }

    fn try_read_external<T: DeserializeOwned + Serialize>(
        &self,
        artifact: PublisherArtifactV2,
    ) -> Result<Option<T>> {
        match read_exact_path(
            &external_exchange_path_v2(self.repetition, artifact),
            expected_external_owner(artifact),
            expected_external_mode(artifact),
        )? {
            Some(bytes) => Ok(Some(parse_canonical_v2(&bytes)?)),
            None => Ok(None),
        }
    }

    fn read_external_bytes(&self, artifact: PublisherArtifactV2) -> Result<Vec<u8>> {
        read_exact_path(
            &external_exchange_path_v2(self.repetition, artifact),
            expected_external_owner(artifact),
            expected_external_mode(artifact),
        )?
        .context("fixed publisher exchange artifact is absent")
    }

    fn read_root<T: DeserializeOwned + Serialize>(
        &self,
        artifact: PublisherArtifactV2,
    ) -> Result<T> {
        read_exact_path(
            &root_publisher_path_v2(self.repetition, artifact),
            0,
            ROOT_FILE_MODE,
        )?
        .context("fixed root publisher artifact is absent")
        .and_then(|bytes| parse_canonical_v2(&bytes))
    }

    fn try_read_root<T: DeserializeOwned + Serialize>(
        &self,
        artifact: PublisherArtifactV2,
    ) -> Result<Option<T>> {
        read_exact_path(
            &root_publisher_path_v2(self.repetition, artifact),
            0,
            ROOT_FILE_MODE,
        )?
        .map(|bytes| parse_canonical_v2(&bytes))
        .transpose()
    }

    fn persist_root(
        &self,
        artifact: PublisherArtifactV2,
        bytes: &[u8],
        predecessor: Option<&[u8]>,
    ) -> Result<()> {
        write_exact_path(
            &root_publisher_path_v2(self.repetition, artifact),
            bytes,
            0o600,
            predecessor,
            ParentOwnerV2::Root,
        )
    }

    fn publish_external_output(&self, artifact: PublisherArtifactV2, bytes: &[u8]) -> Result<()> {
        write_exact_path(
            &external_exchange_path_v2(self.repetition, artifact),
            bytes,
            0o444,
            None,
            ParentOwnerV2::Harness,
        )
    }

    fn publish_protocol_output(&self, artifact: PublisherArtifactV2, bytes: &[u8]) -> Result<()> {
        self.persist_root(artifact, bytes, None)?;
        self.publish_external_output(artifact, bytes)
    }
}

fn fixed_repetition(value: RepetitionV2) -> FixedRepetitionV2 {
    match value {
        RepetitionV2::One => FixedRepetitionV2::First,
        RepetitionV2::Two => FixedRepetitionV2::Second,
    }
}

fn validate_process_chain_entry(
    value: &ProcessAttestationChainEntryV2,
    repetition: RepetitionV2,
    ordinal: u64,
    predecessor: &str,
    prepared: &PublisherPreparedInputV2,
) -> Result<()> {
    repetition.validate_binding(value.repetition, &value.scope_id)?;
    value.process_attestation.validate(prepared)?;
    if value.schema_owner != PUBLISHER_PROTOCOL_OWNER_V2
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.experiment_id != EXPERIMENT_ID_V2
        || value.ordinal != ordinal
        || value.predecessor_entry_sha256 != predecessor
        || value.progress_generation == 0
    {
        bail!("publisher process-attestation chain entry changed identity")
    }
    for digest in [
        &value.predecessor_entry_sha256,
        &value.marker_sha256,
        &value.input_bundle_sha256,
    ] {
        require_sha256(digest, "publisher process-attestation chain")?;
    }
    Ok(())
}

fn validate_residual_cursor(
    value: &ResidualCursorV2,
    repetition: RepetitionV2,
    effects_response_sha256: &str,
    wrong_identity_sha256: &str,
) -> Result<()> {
    repetition.validate_binding(value.repetition, &value.scope_id)?;
    if value.schema_owner != PUBLISHER_PROTOCOL_OWNER_V2
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.effects_response_sha256 != effects_response_sha256
        || value.wrong_identity_sha256 != wrong_identity_sha256
    {
        bail!("residual cleanup cursor changed its exact response or wrong-key identity")
    }
    require_sha256(&value.effects_response_sha256, "residual effects response")?;
    require_sha256(&value.wrong_identity_sha256, "residual wrong identity")
}

fn validate_restoration_cursor(
    value: &RestorationCursorV2,
    repetition: RepetitionV2,
    complete_response_sha256: &str,
    request_sha256: &str,
    terminal_sha256: &str,
) -> Result<()> {
    repetition.validate_binding(value.repetition, &value.scope_id)?;
    if value.schema_owner != PUBLISHER_PROTOCOL_OWNER_V2
        || value.schema_version != EXPERIMENT_VERSION_V2
        || value.complete_response_sha256 != complete_response_sha256
        || value.request_sha256 != request_sha256
        || value.terminal_sha256 != terminal_sha256
    {
        bail!("restoration cursor changed its exact HostComplete or inbox identities")
    }
    for digest in [
        &value.complete_response_sha256,
        &value.request_sha256,
        &value.terminal_sha256,
    ] {
        require_sha256(digest, "restoration cursor")?;
    }
    Ok(())
}

fn require_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is not one lowercase SHA-256")
    }
    Ok(())
}

fn successor_stage(value: PublisherStageV2) -> Result<PublisherStageV2> {
    let index = PUBLISHER_STAGE_SEQUENCE_V2
        .iter()
        .position(|candidate| *candidate == value)
        .context("publisher progress contains an unknown stage")?;
    PUBLISHER_STAGE_SEQUENCE_V2
        .get(index + 1)
        .copied()
        .context("publisher repetition is already restored")
}

fn input_artifact(value: PublisherInputArtifactV2) -> PublisherArtifactV2 {
    match value {
        PublisherInputArtifactV2::PreparedInput => PublisherArtifactV2::PreparedInput,
        PublisherInputArtifactV2::PublisherProcessAttestation => {
            PublisherArtifactV2::PublisherProcessAttestation
        }
        PublisherInputArtifactV2::CreationReceipt => PublisherArtifactV2::CreationReceipt,
        PublisherInputArtifactV2::IdentityBinding => PublisherArtifactV2::IdentityBinding,
        PublisherInputArtifactV2::UnsignedReceipt => PublisherArtifactV2::UnsignedReceipt,
        PublisherInputArtifactV2::SignedReceipt => PublisherArtifactV2::SignedReceipt,
        PublisherInputArtifactV2::HarnessAcknowledgement => {
            PublisherArtifactV2::HarnessAcknowledgement
        }
        PublisherInputArtifactV2::UnsignedProtectedCas => PublisherArtifactV2::UnsignedProtectedCas,
        PublisherInputArtifactV2::SignedProtectedCas => PublisherArtifactV2::SignedProtectedCas,
        PublisherInputArtifactV2::FinalizationRequest => PublisherArtifactV2::FinalizationRequest,
        PublisherInputArtifactV2::EffectsCompleteResponse => {
            PublisherArtifactV2::EffectsCompleteResponse
        }
        PublisherInputArtifactV2::TerminalAcknowledgement => {
            PublisherArtifactV2::TerminalAcknowledgement
        }
        PublisherInputArtifactV2::TerminalBinding => PublisherArtifactV2::TerminalBinding,
        PublisherInputArtifactV2::CompleteResponse => PublisherArtifactV2::CompleteResponse,
    }
}

fn expected_external_owner(artifact: PublisherArtifactV2) -> u32 {
    match artifact {
        PublisherArtifactV2::CreationReceipt
        | PublisherArtifactV2::PublisherProcessAttestation
        | PublisherArtifactV2::SignedReceipt
        | PublisherArtifactV2::SignedProtectedCas
        | PublisherArtifactV2::ResidualCleanupReceipt
        | PublisherArtifactV2::RestorationReceipt
        | PublisherArtifactV2::EmergencyRollbackReceipt
        | PublisherArtifactV2::TransportControlsObservation
        | PublisherArtifactV2::SecurityAgentObservation(_)
        | PublisherArtifactV2::Progress
        | PublisherArtifactV2::StepReceipt(_) => 0,
        _ => DISPOSABLE_HARNESS_UID_V2,
    }
}

fn expected_external_mode(artifact: PublisherArtifactV2) -> libc::mode_t {
    if expected_external_owner(artifact) == 0 {
        EXTERNAL_OUTPUT_MODE
    } else {
        EXTERNAL_INPUT_MODE
    }
}

fn internal_creation(
    repetition: RepetitionV2,
    value: &SurrogateCreationReceiptV2,
) -> DisposableKeyPairCreationReceiptV2 {
    DisposableKeyPairCreationReceiptV2 {
        schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        repetition: fixed_repetition(repetition),
        scope_id: repetition.scope_id().to_owned(),
        target: value.target.clone(),
        wrong_surrogate: value.wrong.clone(),
        wrapper_identity_sha256: value.protected_wrapper_identity_sha256.clone(),
    }
}

fn require_disjoint_keys(
    target: &DisposableSignerIdentityV2,
    wrong: &DisposableSignerIdentityV2,
) -> Result<()> {
    if target.application_tag_sha256 == wrong.application_tag_sha256
        || target.application_label == wrong.application_label
        || target.persistent_reference_sha256 == wrong.persistent_reference_sha256
        || target.spki_der == wrong.spki_der
        || target.identity_sha256 == wrong.identity_sha256
        || target.access_control_sha256 == wrong.access_control_sha256
    {
        bail!("target and wrong-surrogate public/full identities are not strictly disjoint")
    }
    Ok(())
}

fn finalizer_observation(value: &DisposableSignerIdentityV2) -> FinalizerKeyObservation<'_> {
    FinalizerKeyObservation {
        application_tag_sha256: &value.application_tag_sha256,
        label: &value.label,
        application_label_base64url: &value.application_label,
        persistent_reference_sha256: &value.persistent_reference_sha256,
        public_spki_sha256: &value.spki_der_sha256,
        access_control_sha256: &value.access_control_sha256,
        identity_sha256: &value.identity_sha256,
    }
}

fn capability_observation_sha256(
    scope_id: &str,
    target: &DisposableSignerIdentityV2,
    wrong: &DisposableSignerIdentityV2,
) -> Result<String> {
    document_sha256_v2(&CapabilityObservation {
        schema_owner: CAPABILITY_REPORT_OWNER,
        schema_version: 2,
        target_set_kind: TargetSetKindV2::DisposableCapability,
        scope_id,
        signer_access_control_sha256: &target.access_control_sha256,
        target: finalizer_observation(target),
        wrong_surrogate: finalizer_observation(wrong),
    })
}

fn current_lock_path(repetition: RepetitionV2) -> PathBuf {
    Path::new("/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/capability")
        .join(repetition.scope_id())
        .join("surrogate.lock")
}

fn terminal_latch_path(repetition: RepetitionV2) -> PathBuf {
    Path::new("/private/var/db/com.atomize.substrate.r3-macos-evidence-finalizer.v2/latches").join(
        format!("{}.retirement-terminal.v2.latch", repetition.scope_id()),
    )
}

fn scope_digest(repetition: RepetitionV2, label: &str) -> String {
    sha256_hex_v2(
        format!(
            "substrate.r3-macos-disposable-publisher.v2\0{}\0{label}",
            repetition.scope_id()
        )
        .as_bytes(),
    )
}

fn require_deleted(receipt: &ExactDeleteReceipt, label: &str) -> Result<()> {
    if receipt.raw_os_status != 0
        || receipt.classification != ExactDeleteClassification::DeletedAndAbsent
        || receipt.present_after
    {
        bail!("{label} did not return exact deletion and absence")
    }
    Ok(())
}

fn prepare_root_repetition(repetition: RepetitionV2) -> Result<()> {
    require_directory(
        Path::new(DISPOSABLE_PUBLISHER_ROOT_V2),
        0,
        ROOT_DIRECTORY_MODE,
    )?;
    let repetitions = Path::new(DISPOSABLE_PUBLISHER_ROOT_V2).join("repetitions");
    ensure_root_directory(&repetitions)?;
    ensure_root_directory(&repetitions.join(repetition.directory_name()))
}

fn ensure_root_directory(path: &Path) -> Result<()> {
    match std::fs::create_dir(path) {
        Ok(()) => {
            std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o700))?;
            let parent = File::open(path.parent().context("root directory has no parent")?)?;
            parent.sync_all()?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).context("create fixed root publisher directory"),
    }
    require_directory(path, 0, ROOT_DIRECTORY_MODE)
}

fn require_directory(path: &Path, uid: u32, mode: libc::mode_t) -> Result<()> {
    let canonical = path
        .canonicalize()
        .context("canonicalize fixed directory")?;
    if canonical != path {
        bail!("fixed directory has a symlink or alternate spelling")
    }
    let metadata = std::fs::symlink_metadata(path).context("inspect fixed directory")?;
    use std::os::unix::fs::MetadataExt;
    if metadata.uid() != uid
        || metadata.gid() != 0
        || metadata.mode() != u32::from(mode)
        || !metadata.file_type().is_dir()
    {
        bail!("fixed directory identity, owner, group, or mode differs")
    }
    Ok(())
}

fn read_exact_path(path: &Path, uid: u32, mode: libc::mode_t) -> Result<Option<Vec<u8>>> {
    let parent_uid = if path.starts_with(DISPOSABLE_PUBLISHER_ROOT_V2) {
        0
    } else {
        DISPOSABLE_HARNESS_UID_V2
    };
    read_exact_published_file(
        path,
        PublishIdentityV2 {
            owner_uid: uid,
            owner_gid: 0,
            permissions: mode & 0o7777,
            parent_uid,
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParentOwnerV2 {
    Root,
    Harness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeFileKindV2 {
    Wrapper,
    CurrentLock,
    TerminalLatch,
}

impl ParentOwnerV2 {
    const fn uid(self) -> u32 {
        match self {
            Self::Root => 0,
            Self::Harness => DISPOSABLE_HARNESS_UID_V2,
        }
    }
}

fn remove_exact_scope_file(
    path: &Path,
    expected_mode: libc::mode_t,
    expected_identity_sha256: Option<&str>,
    kind: ScopeFileKindV2,
) -> Result<()> {
    let Some(before) = stat_nofollow(path)? else {
        return Ok(());
    };
    require_file_stat(&before, 0, expected_mode)?;
    let observed_identity_sha256 = match kind {
        ScopeFileKindV2::Wrapper => physical_wrapper_digest(path)?,
        ScopeFileKindV2::CurrentLock | ScopeFileKindV2::TerminalLatch => file_observation(path)?,
    };
    if expected_identity_sha256.is_some_and(|expected| expected != observed_identity_sha256) {
        bail!("emergency rollback scope-file identity changed")
    }
    let after = stat_nofollow(path)?.context("emergency rollback scope file disappeared")?;
    if !same_stat(&before, &after) {
        bail!("emergency rollback scope file changed before unlink")
    }
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: exact compiled path, no-follow identity was measured immediately before unlink.
    if unsafe { libc::unlink(encoded.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("unlink exact emergency scope file");
    }
    File::open(
        path.parent()
            .context("emergency scope file has no parent")?,
    )?
    .sync_all()?;
    if path_present(path)? {
        bail!("emergency rollback scope file remains present")
    }
    Ok(())
}

fn write_exact_path(
    path: &Path,
    bytes: &[u8],
    permissions: libc::mode_t,
    predecessor: Option<&[u8]>,
    parent_owner: ParentOwnerV2,
) -> Result<()> {
    require_fixed_parent(path, parent_owner.uid())?;
    let mode = predecessor.map_or(PublishMode::Immutable, |predecessor| {
        PublishMode::ReplaceExact { predecessor }
    });
    publish_exact_file(
        path,
        bytes,
        PublishIdentityV2 {
            owner_uid: 0,
            owner_gid: 0,
            permissions,
            parent_uid: parent_owner.uid(),
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
        },
        mode,
    )
}

fn require_fixed_parent(path: &Path, uid: u32) -> Result<()> {
    let parent = path.parent().context("fixed artifact has no parent")?;
    require_directory(parent, uid, libc::S_IFDIR | 0o700)
}

fn create_or_measure_exact_file(
    path: &Path,
    bytes: &[u8],
    permissions: libc::mode_t,
) -> Result<String> {
    require_fixed_parent(path, 0)?;
    publish_exact_file(
        path,
        bytes,
        PublishIdentityV2 {
            owner_uid: 0,
            owner_gid: 0,
            permissions,
            parent_uid: 0,
            parent_gid: 0,
            parent_mode: libc::S_IFDIR | 0o700,
            maximum_bytes: MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
        },
        PublishMode::Immutable,
    )?;
    let observed = read_exact_path(path, 0, libc::S_IFREG | permissions)?
        .context("created exact file is absent")?;
    if observed != bytes {
        bail!("created exact file contains different bytes")
    }
    file_observation(path)
}

fn file_observation(path: &Path) -> Result<String> {
    let bytes = read_exact_path(path, 0, LOCK_FILE_MODE)?.context("exact file is absent")?;
    let stat = stat_nofollow(path)?.context("exact file disappeared")?;
    let path = path.to_str().context("exact file path is not UTF-8")?;
    document_sha256_v2(&ExactFileObservation {
        path,
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

fn physical_wrapper_digest(path: &Path) -> Result<String> {
    let stat = stat_nofollow(path)?.context("exact wrapper is absent")?;
    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct Wrapper<'a> {
        path: &'a str,
        device: u64,
        inode: u64,
        uid: u32,
        gid: u32,
        mode: u32,
        link_count: u64,
    }
    document_sha256_v2(&Wrapper {
        path: path.to_str().context("wrapper path is not UTF-8")?,
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        uid: stat.st_uid,
        gid: stat.st_gid,
        mode: u32::from(stat.st_mode),
        link_count: stat.st_nlink as u64,
    })
}

fn path_present(path: &Path) -> Result<bool> {
    Ok(stat_nofollow(path)?.is_some())
}

fn stat_nofollow(path: &Path) -> Result<Option<libc::stat>> {
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    let mut value = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: exact path and writable output; lstat never follows the terminal component.
    if unsafe { libc::lstat(encoded.as_ptr(), value.as_mut_ptr()) } != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error).context("lstat exact path");
    }
    // SAFETY: successful lstat initialized value.
    Ok(Some(unsafe { value.assume_init() }))
}

fn require_file_stat(value: &libc::stat, uid: u32, mode: libc::mode_t) -> Result<()> {
    if value.st_uid != uid
        || value.st_gid != 0
        || value.st_nlink != 1
        || (value.st_mode & (libc::S_IFMT | 0o7777)) != mode
    {
        bail!("exact artifact owner, group, type, mode, or link count differs")
    }
    Ok(())
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

fn install_inbox(artifact: PublisherArtifactV2, bytes: &[u8]) -> Result<()> {
    let path = installed_finalizer_inbox_path_v2(artifact)?;
    require_inbox_parent(path)?;
    publish_exact_file(
        path,
        bytes,
        PublishIdentityV2 {
            owner_uid: INSTALLED_INBOX_UID,
            owner_gid: INSTALLED_INBOX_GID,
            permissions: MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2 as libc::mode_t,
            parent_uid: INSTALLED_INBOX_UID,
            parent_gid: INSTALLED_INBOX_GID,
            parent_mode: INSTALLED_INBOX_DIRECTORY_MODE,
            maximum_bytes: MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
        },
        PublishMode::Immutable,
    )?;
    if read_installed_inbox(path)? != Some(bytes.to_vec()) {
        bail!("installed finalizer inbox leaf failed exact durable readback")
    }
    Ok(())
}

fn remove_installed_inbox(artifact: PublisherArtifactV2, expected: &[u8]) -> Result<bool> {
    let path = installed_finalizer_inbox_path_v2(artifact)?;
    let Some(observed) = read_installed_inbox(path)? else {
        return Ok(true);
    };
    if observed != expected {
        bail!("installed finalizer inbox artifact changed before exact cleanup")
    }
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: exact compiled file path, previously read and content-conditioned.
    if unsafe { libc::unlink(encoded.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("unlink exact finalizer inbox artifact");
    }
    File::open(path.parent().context("inbox artifact lacks parent")?)?.sync_all()?;
    Ok(!path_present(path)?)
}

fn require_inbox_parent(path: &Path) -> Result<()> {
    let parent = path.parent().context("inbox artifact lacks parent")?;
    let canonical = parent
        .canonicalize()
        .context("canonicalize fixed finalizer inbox")?;
    let metadata = std::fs::symlink_metadata(parent)?;
    if canonical != parent
        || !metadata.file_type().is_dir()
        || metadata.uid() != INSTALLED_INBOX_UID
        || metadata.gid() != INSTALLED_INBOX_GID
        || metadata.mode() != u32::from(INSTALLED_INBOX_DIRECTORY_MODE)
    {
        bail!("finalizer inbox is not exact root:staff 0750 no-follow state")
    }
    Ok(())
}

fn read_installed_inbox(path: &Path) -> Result<Option<Vec<u8>>> {
    require_inbox_parent(path)?;
    read_exact_published_file(
        path,
        PublishIdentityV2 {
            owner_uid: INSTALLED_INBOX_UID,
            owner_gid: INSTALLED_INBOX_GID,
            permissions: MAC_R3_COORDINATOR_INBOX_FILE_MODE_V2 as libc::mode_t,
            parent_uid: INSTALLED_INBOX_UID,
            parent_gid: INSTALLED_INBOX_GID,
            parent_mode: INSTALLED_INBOX_DIRECTORY_MODE,
            maximum_bytes: MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supervisor_sequence_and_artifact_mapping_are_closed() {
        assert_eq!(RepetitionV2::ALL.len(), 2);
        for pair in PUBLISHER_STAGE_SEQUENCE_V2.windows(2) {
            assert_eq!(successor_stage(pair[0]).unwrap(), pair[1]);
            assert!(!expected_input_artifacts_v2(pair[1]).is_empty());
        }
        for repetition in RepetitionV2::ALL {
            assert!(current_lock_path(repetition)
                .to_str()
                .unwrap()
                .contains(repetition.scope_id()));
            assert!(terminal_latch_path(repetition)
                .to_str()
                .unwrap()
                .contains(repetition.scope_id()));
        }
        assert_eq!(
            input_artifact(PublisherInputArtifactV2::EffectsCompleteResponse),
            PublisherArtifactV2::EffectsCompleteResponse
        );
    }

    #[test]
    fn supervisor_surface_has_no_action_target_or_environment_selector() {
        let source = include_str!("supervisor.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("std::env::args"));
        assert!(!production.contains("std::env::var"));
        for forbidden in [
            "caller_path",
            "caller_tag",
            "caller_action",
            "caller_predicate",
        ] {
            assert!(!production.contains(forbidden));
        }
        assert!(production.contains("PUBLISHER_STAGE_SEQUENCE_V2"));
        assert!(production.contains("EffectsComplete"));
        assert!(production.contains("wrong surrogate disappeared before a durable cleanup cursor"));
    }

    #[test]
    fn restoration_inbox_removal_is_cursor_first_and_resumable() {
        let source = include_str!("supervisor.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(production.contains("RestorationCursorPhaseV2::RequestInvoked"));
        assert!(production.contains("RestorationCursorPhaseV2::RequestObserved"));
        assert!(production.contains("RestorationCursorPhaseV2::TerminalInvoked"));
        assert!(production.contains("RestorationCursorPhaseV2::TerminalObserved"));
        assert!(production.contains("write_restoration_cursor(&next, Some(&cursor))"));
    }
}
