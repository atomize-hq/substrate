#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("substrate R3 macOS disposable harness is available only on macOS");
    std::process::exit(78);
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    macos::run()
}

#[cfg(target_os = "macos")]
mod macos {
    use std::fmt;
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdout, Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use anyhow::{bail, Context, Result};
    use serde::{de::DeserializeOwned, Serialize};
    use substrate_common::macos_retirement_v2::{
        document_sha256_v2, parse_canonical_v2, validate_finalizer_response_v2,
        FinalizerResponseStateV2, FinalizerResponseV2, ProtectedCasBindingV2,
        PublisherPreRemovalReceiptV2, MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2,
    };
    use substrate_r3_macos_finalizer::experiment::controls::{
        single_securityagent_arm_evidence_v2, DisposableNativeArmReceiptV2, DisposableNativeArmV2,
        PeerControlIdentityPacketV2, PeerControlRestorationReceiptV2, PeerControlSetReceiptV2,
        SecurityAgentArmEvidenceV2, TransportControlReceiptV2, TransportControlSetReceiptV2,
        PEER_CONTROL_RECEIPT_OWNER_V2, TRANSPORT_CONTROL_SEQUENCE_V2,
    };
    use substrate_r3_macos_finalizer::experiment::documents::{
        build_finalization_request_v2, build_signed_harness_acknowledgement_v2,
        build_signed_parity_and_terminal_v2, build_terminal_binding_request_v2,
        build_unsigned_protected_cas_v2, build_unsigned_publisher_receipt_v2,
        predecessor_journal_head_v2, retry_state_v2, EphemeralHarnessSignerV2,
    };
    use substrate_r3_macos_finalizer::experiment::durable::{
        DurableFileObservationV2, ExperimentArtifactV2, ExperimentStoreV2,
    };
    use substrate_r3_macos_finalizer::experiment::evidence_export::{
        build_native_evidence_export_acknowledgement_v2, NativeEvidenceCleanupReceiptV2,
        NativeEvidenceExportAcknowledgementV2, NativeEvidenceExportV2, SecurityAgentEvidenceArmV2,
        SecurityAgentExpectedSourceV2,
    };
    use substrate_r3_macos_finalizer::experiment::freeze_manifest::{
        build_candidate_freeze_manifest_v2, build_candidate_freeze_supporting_manifests_v2,
        candidate_freeze_coordinator_provenance_input_v2,
        candidate_freeze_global_provenance_input_v2,
    };
    use substrate_r3_macos_finalizer::experiment::harness_protocol::{
        build_disposable_denylist_proof_v2, build_harness_transition_v2,
        build_restoration_manifest_v2, DisposableDenylistProofV2, ExperimentBaselineV2,
        HarnessInputArtifactDigestV2, HarnessInputArtifactV2, HarnessProgressV2,
        RestorationManifestInputV2, SigningSeedRemovalObservationV2,
    };
    use substrate_r3_macos_finalizer::experiment::pre_effect::{
        CreatorRouteReceiptSetV2, GlobalPreEffectPacketV2,
    };
    use substrate_r3_macos_finalizer::experiment::process::{
        attest_coordinator_process_v2, attest_fixed_peer_process_v2,
        CoordinatorProcessAttestationV2,
    };
    use substrate_r3_macos_finalizer::experiment::publisher_protocol::{
        CandidateIdentityPacketV2, EmergencyFailureClassificationV2, EmergencyRollbackMarkerV2,
        EmergencyRollbackReceiptV2, IdentityBindingPacketV2, PreCreationEmergencyRollbackMarkerV2,
        PublisherAdvanceMarkerV2, PublisherArtifactV2, PublisherInputArtifactDigestV2,
        PublisherInputArtifactV2, PublisherInputBundleV2, PublisherPreparedInputV2,
        PublisherProcessAttestationV2, PublisherProgressV2, PublisherSecurityAgentObservationV2,
        PublisherStageV2, PublisherStepReceiptV2, ResidualCleanupReceiptV2, RestorationReceiptV2,
        SurrogateCreationReceiptV2, PUBLISHER_PROTOCOL_OWNER_V2, PUBLISHER_STAGE_SEQUENCE_V2,
    };
    use substrate_r3_macos_finalizer::experiment::{
        HarnessStageV2, RepetitionV2, CANDIDATE_IDENTITY_PACKET_PATH_V2,
        DISPOSABLE_HARNESS_ACCOUNT_V2, DISPOSABLE_HARNESS_GID_V2, DISPOSABLE_HARNESS_PATH_V2,
        DISPOSABLE_HARNESS_UID_V2, DISPOSABLE_PREPARED_INPUT_PATH_V2, EXPERIMENT_ID_V2,
        EXPERIMENT_OWNER_V2, EXPERIMENT_VERSION_V2, PEER_CONTROL_IDENTITY_PACKET_PATH_V2,
    };

    const WAIT_TIMEOUT: Duration = Duration::from_secs(3_600);
    const POLL_INTERVAL: Duration = Duration::from_millis(100);
    const INSTALLED_PACKET_MODE: u32 = 0o444;
    const SECURITYAGENT_ALERT_EXIT_CODE: i32 = 86;

    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct SurrogatePredicateSetV2<'a> {
        schema_owner: &'static str,
        schema_version: u32,
        experiment_id: &'static str,
        scope_id: &'a str,
        target_label: String,
        wrong_label: String,
    }

    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct CoordinatorReceiptSetV2<'a> {
        schema_owner: &'static str,
        schema_version: u32,
        controls: &'a [TransportControlReceiptV2],
        set: &'a TransportControlSetReceiptV2,
    }

    #[derive(Serialize)]
    #[serde(deny_unknown_fields)]
    struct FailureObservationV2<'a> {
        schema_owner: &'static str,
        schema_version: u32,
        experiment_id: &'static str,
        repetition: u8,
        scope_id: &'a str,
        publisher_stage: PublisherStageV2,
        failure_classification: EmergencyFailureClassificationV2,
        error: &'a str,
    }

    #[derive(Debug)]
    struct SecurityAgentAlertStopV2;

    impl fmt::Display for SecurityAgentAlertStopV2 {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("SecurityAgent ALERT terminated the exact coordinator with exit 86")
        }
    }

    impl std::error::Error for SecurityAgentAlertStopV2 {}

    struct CoordinatorChildV2 {
        child: Child,
        stdout: ChildStdout,
        attestation: CoordinatorProcessAttestationV2,
    }

    struct PendingRestorationV2 {
        repetition: RepetitionV2,
        request: substrate_common::macos_retirement_v2::FinalizationRequestV2,
        peer_set: PeerControlSetReceiptV2,
        transport_controls: Vec<TransportControlReceiptV2>,
        effects_arm: DisposableNativeArmReceiptV2,
        complete_arm: DisposableNativeArmReceiptV2,
        complete: FinalizerResponseV2,
        baseline_packet_sha256: String,
        surrogate_before_observation_sha256: String,
        exact_after_observation_sha256: String,
        restoration: RestorationReceiptV2,
        complete_response_sha256: String,
        denylist_proof: DisposableDenylistProofV2,
        seed_observation: DurableFileObservationV2,
        harness_progress: HarnessProgressV2,
    }

    pub fn run() -> Result<()> {
        require_closed_harness_surface()?;
        let prepared: PublisherPreparedInputV2 =
            read_installed_packet(Path::new(DISPOSABLE_PREPARED_INPUT_PATH_V2))?;
        prepared.validate()?;
        let candidate: CandidateIdentityPacketV2 =
            read_installed_packet(Path::new(CANDIDATE_IDENTITY_PACKET_PATH_V2))?;
        candidate.validate()?;
        let peer_identities: PeerControlIdentityPacketV2 =
            read_installed_packet(Path::new(PEER_CONTROL_IDENTITY_PACKET_PATH_V2))?;
        peer_identities.validate(&candidate)?;
        let harness_attestation = attest_fixed_peer_process_v2(
            i32::try_from(std::process::id()).context("harness PID exceeds i32")?,
            &peer_identities.harness_identity,
            DISPOSABLE_HARNESS_UID_V2,
            DISPOSABLE_HARNESS_GID_V2,
            DISPOSABLE_HARNESS_ACCOUNT_V2,
        )?;
        clear_process_environment()?;
        let store = ExperimentStoreV2::open_fixed()?;
        let (candidate_freeze_manifest, candidate_freeze_manifest_observation) =
            store.read_candidate_freeze_manifest()?;
        candidate_freeze_manifest.validate()?;
        let (candidate_freeze_input, _) = store.read_candidate_freeze_manifest_input()?;
        let supporting = build_candidate_freeze_supporting_manifests_v2(&candidate_freeze_input)?;
        let (coordinator_provenance_input, _) =
            store.read_candidate_freeze_coordinator_provenance_input()?;
        let (global_provenance_input, _) = store.read_candidate_freeze_global_provenance_input()?;
        let (source_hashes, source_hashes_observation) =
            store.read_candidate_freeze_source_hashes()?;
        let (coordinator_build_inputs, coordinator_build_observation) =
            store.read_candidate_freeze_coordinator_build_inputs()?;
        let (global_build_inputs, global_build_observation) =
            store.read_candidate_freeze_global_build_inputs()?;
        if source_hashes != supporting.source_hashes
            || coordinator_provenance_input
                != candidate_freeze_coordinator_provenance_input_v2(&candidate_freeze_input)
            || global_provenance_input
                != candidate_freeze_global_provenance_input_v2(&candidate_freeze_input)?
            || coordinator_build_inputs != supporting.coordinator_build_inputs
            || global_build_inputs != supporting.global_build_inputs
            || candidate_freeze_manifest
                != build_candidate_freeze_manifest_v2(candidate_freeze_input)?
            || candidate_freeze_manifest.source_hashes_manifest_sha256
                != source_hashes_observation.sha256
            || candidate_freeze_manifest.coordinator_provenance_input_sha256
                != document_sha256_v2(&coordinator_provenance_input)?
            || candidate_freeze_manifest.global_provenance_input_sha256
                != document_sha256_v2(&global_provenance_input)?
            || candidate_freeze_manifest.coordinator_build_input_manifest_sha256
                != coordinator_build_observation.sha256
            || candidate_freeze_manifest.global_build_input_manifest_sha256
                != global_build_observation.sha256
            || candidate_freeze_manifest.coordinator_build_digest
                != supporting.coordinator_build_inputs.input_set_sha256
            || candidate_freeze_manifest.global_build_digest
                != supporting.global_build_inputs.input_set_sha256
        {
            bail!("candidate freeze manifest does not reproduce from its reopened immutable input")
        }
        let (_, reviewed_admin_block_observation) = store.read_candidate_freeze_admin_block()?;
        let (global_pre_effect, global_pre_effect_observation) = store
            .read_global_publisher_output::<GlobalPreEffectPacketV2>(
                PublisherArtifactV2::GlobalPreEffectPacket,
            )?;
        global_pre_effect.validate(&prepared, &candidate, &peer_identities)?;
        if global_pre_effect_observation.sha256 != document_sha256_v2(&global_pre_effect)? {
            bail!("global pre-effect packet bytes changed during canonical reopen")
        }
        if global_pre_effect.candidate_freeze_manifest != candidate_freeze_manifest
            || global_pre_effect.candidate_freeze_manifest_sha256
                != candidate_freeze_manifest_observation.sha256
            || global_pre_effect.reviewed_admin_block_sha256
                != reviewed_admin_block_observation.sha256
        {
            bail!("global pre-effect packet does not bind the reopened candidate freeze and reviewed admin block bytes")
        }
        let (creator_receipts, creator_receipts_observation) = store
            .read_global_publisher_output::<CreatorRouteReceiptSetV2>(
                PublisherArtifactV2::CreatorRouteReceiptSet,
            )?;
        creator_receipts.validate(&global_pre_effect, &peer_identities)?;
        if creator_receipts_observation.sha256 != document_sha256_v2(&creator_receipts)? {
            bail!("creator-route receipt-set bytes changed during canonical reopen")
        }
        let mut pending = Vec::with_capacity(RepetitionV2::ALL.len());
        for repetition in RepetitionV2::ALL {
            match run_repetition(
                &store,
                repetition,
                &prepared,
                &candidate,
                &peer_identities,
                &harness_attestation,
                &global_pre_effect,
                &creator_receipts,
            ) {
                Ok(value) => pending.push(value),
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!(
                            "disposable exact-finalizer repetition {} stopped",
                            repetition.ordinal()
                        )
                    })
                }
            }
        }
        let requests = pending
            .iter()
            .map(|value| value.request.clone())
            .collect::<Vec<_>>();
        let peer_sets = pending
            .iter()
            .map(|value| value.peer_set.clone())
            .collect::<Vec<_>>();
        let completes = pending
            .iter()
            .map(|value| value.complete.clone())
            .collect::<Vec<_>>();
        let native_evidence_export: NativeEvidenceExportV2 = wait_global_publisher_output(
            &store,
            PublisherArtifactV2::NativeEvidenceExport,
            "canonical native evidence export",
        )?;
        native_evidence_export.validate(&completes)?;
        native_evidence_export.validate_root_install_claims(&global_pre_effect)?;
        let securityagent_sources = build_securityagent_expected_sources(
            &store,
            &global_pre_effect,
            &creator_receipts,
            &pending,
        )?;
        native_evidence_export.validate_securityagent_sources(&securityagent_sources)?;
        let native_evidence_acknowledgement = build_native_evidence_export_acknowledgement_v2(
            &native_evidence_export,
            document_sha256_v2(&harness_attestation)?,
        )?;
        store.persist_global_publisher_input(
            PublisherArtifactV2::NativeEvidenceExportAcknowledgement,
            &native_evidence_acknowledgement,
        )?;
        let native_evidence_cleanup: NativeEvidenceCleanupReceiptV2 = wait_global_publisher_output(
            &store,
            PublisherArtifactV2::NativeEvidenceCleanupReceipt,
            "acknowledged finalizer service and journal cleanup receipt",
        )?;
        native_evidence_cleanup.validate(
            &native_evidence_export,
            &native_evidence_acknowledgement,
            &global_pre_effect,
        )?;
        let peer_restoration: PeerControlRestorationReceiptV2 =
            wait_global_peer_restoration(&store)?;
        peer_restoration.validate(&peer_identities, &requests, &peer_sets, &completes)?;
        for value in pending {
            finalize_restoration(
                &store,
                value,
                &native_evidence_export,
                &native_evidence_acknowledgement,
                &native_evidence_cleanup,
                &peer_restoration,
            )?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn run_repetition(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        prepared: &PublisherPreparedInputV2,
        candidate: &CandidateIdentityPacketV2,
        peer_identities: &PeerControlIdentityPacketV2,
        harness_attestation: &CoordinatorProcessAttestationV2,
        global_pre_effect: &GlobalPreEffectPacketV2,
        creator_receipts: &CreatorRouteReceiptSetV2,
    ) -> Result<PendingRestorationV2> {
        store.prepare_repetition(repetition)?;
        let mut publisher_stage = PublisherStageV2::Prepared;
        let mut creation: Option<SurrogateCreationReceiptV2> = None;
        let result = run_repetition_inner(
            store,
            repetition,
            prepared,
            candidate,
            peer_identities,
            harness_attestation,
            global_pre_effect,
            creator_receipts,
            &mut publisher_stage,
            &mut creation,
        );
        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                let failure_classification = emergency_failure_classification(&error);
                let rollback = request_emergency_rollback(
                    store,
                    repetition,
                    publisher_stage,
                    creation.as_ref(),
                    failure_classification,
                    &format!("{error:#}"),
                );
                match rollback {
                    Ok(()) => Err(error).context("exact emergency rollback completed"),
                    Err(rollback_error) => Err(error).context(format!(
                        "exact emergency rollback also failed: {rollback_error:#}"
                    )),
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn run_repetition_inner(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        prepared: &PublisherPreparedInputV2,
        candidate: &CandidateIdentityPacketV2,
        peer_identities: &PeerControlIdentityPacketV2,
        harness_attestation: &CoordinatorProcessAttestationV2,
        global_pre_effect: &GlobalPreEffectPacketV2,
        creator_receipts: &CreatorRouteReceiptSetV2,
        publisher_stage: &mut PublisherStageV2,
        creation_slot: &mut Option<SurrogateCreationReceiptV2>,
    ) -> Result<PendingRestorationV2> {
        let prepared_observation = store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::PreparedInput,
            prepared,
        )?;
        let publisher_attestation: PublisherProcessAttestationV2 = wait_publisher_output(
            store,
            repetition,
            PublisherArtifactV2::PublisherProcessAttestation,
            "publisher process attestation",
        )?;
        publisher_attestation.validate(prepared)?;
        let mut publisher_progress: PublisherProgressV2 =
            wait_publisher_progress(store, repetition, PublisherStageV2::Prepared)?;
        publisher_progress.validate(repetition)?;

        let generated_seed = EphemeralHarnessSignerV2::generate_seed_in_memory();
        let (seed, seed_observation) =
            store.load_or_create_signing_seed(repetition, generated_seed)?;
        let harness_signer = EphemeralHarnessSignerV2::from_seed(seed);
        harness_attestation.validate_fixed(
            &peer_identities.harness_identity,
            DISPOSABLE_HARNESS_UID_V2,
            DISPOSABLE_HARNESS_GID_V2,
            DISPOSABLE_HARNESS_ACCOUNT_V2,
        )?;
        let harness_attestation_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::HarnessProcessAttestation,
            harness_attestation,
        )?;
        let mut coordinator = spawn_and_attest_coordinator(&candidate.coordinator_identity)?;
        store.persist_canonical(
            repetition,
            ExperimentArtifactV2::CoordinatorProcessAttestation,
            &coordinator.attestation,
        )?;

        let experiment_root_identity_sha256 = store.root_identity_sha256()?;
        let exact_surrogate_predicates_sha256 = document_sha256_v2(&SurrogatePredicateSetV2 {
            schema_owner: EXPERIMENT_OWNER_V2,
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2,
            scope_id: repetition.scope_id(),
            target_label: format!("{}:signing-key", repetition.scope_id()),
            wrong_label: format!("{}:wrong-surrogate-signing-key", repetition.scope_id()),
        })?;
        let denylist_proof = build_disposable_denylist_proof_v2(repetition)?;
        let denylist_proof_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::DisposableDenylistProof,
            &denylist_proof,
        )?;
        let baseline = ExperimentBaselineV2 {
            schema_owner: EXPERIMENT_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            experiment_root_identity_sha256: experiment_root_identity_sha256.clone(),
            prepared_input_sha256: document_sha256_v2(prepared)?,
            candidate_identity_packet_sha256: document_sha256_v2(candidate)?,
            peer_control_identity_packet_sha256: document_sha256_v2(peer_identities)?,
            global_pre_effect_packet_sha256: document_sha256_v2(global_pre_effect)?,
            creator_route_receipt_set_sha256: document_sha256_v2(creator_receipts)?,
            global_created_object_inventory_sha256: global_pre_effect
                .created_object_inventory_sha256
                .clone(),
            global_native_arm_plan_sha256: global_pre_effect.native_arm_plan_sha256.clone(),
            publisher_process_attestation_sha256: document_sha256_v2(&publisher_attestation)?,
            harness_process_attestation_sha256: harness_attestation_observation.sha256.clone(),
            publisher_genesis_progress_sha256: document_sha256_v2(&publisher_progress)?,
            exact_surrogate_predicates_sha256,
            frozen_inventory_sha256: denylist_proof.frozen_inventory_sha256.clone(),
            compiled_preserved_denylist_sha256: denylist_proof
                .compiled_preserved_denylist_sha256
                .clone(),
            denylist_proof_sha256: denylist_proof_observation.sha256.clone(),
        };
        baseline.validate(
            repetition,
            &denylist_proof,
            global_pre_effect,
            creator_receipts,
        )?;
        let baseline_observation =
            store.persist_canonical(repetition, ExperimentArtifactV2::Baseline, &baseline)?;

        let mut harness_progress = HarnessProgressV2::genesis(repetition);
        store.persist_canonical(
            repetition,
            ExperimentArtifactV2::HarnessProgress(HarnessStageV2::Prepared),
            &harness_progress,
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::CoordinatorAttested,
            vec![
                harness_artifact(
                    HarnessInputArtifactV2::PreparedInput,
                    document_sha256_v2(prepared)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::CandidateIdentity,
                    document_sha256_v2(candidate)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::PeerControlIdentity,
                    document_sha256_v2(peer_identities)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::GlobalPreEffectPacket,
                    document_sha256_v2(global_pre_effect)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::CreatorRouteReceiptSet,
                    document_sha256_v2(creator_receipts)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::PublisherProcessAttestation,
                    document_sha256_v2(&publisher_attestation)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::HarnessProcessAttestation,
                    harness_attestation_observation.sha256,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::HarnessSigningSeedObservation,
                    seed_observation.identity_sha256()?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::DisposableDenylistProof,
                    denylist_proof_observation.sha256,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::CoordinatorProcessAttestation,
                    document_sha256_v2(&coordinator.attestation)?,
                ),
            ],
        )?;

        let creation: SurrogateCreationReceiptV2 = publisher_transition_with_output(
            store,
            repetition,
            &mut publisher_progress,
            PublisherStageV2::SurrogatesCreated,
            vec![
                publisher_artifact(
                    PublisherInputArtifactV2::PreparedInput,
                    prepared_observation.sha256,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::PublisherProcessAttestation,
                    document_sha256_v2(&publisher_attestation)?,
                ),
            ],
            PublisherArtifactV2::CreationReceipt,
            "surrogate creation receipt",
        )?;
        *publisher_stage = PublisherStageV2::SurrogatesCreated;
        creation.validate(repetition)?;
        creation.validate_prepared_input(prepared)?;
        if creation.publisher_process_attestation_sha256
            != document_sha256_v2(&publisher_attestation)?
        {
            bail!("creation receipt did not bind the live publisher process attestation")
        }
        *creation_slot = Some(creation.clone());
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::SurrogatesCreated,
            vec![harness_artifact(
                HarnessInputArtifactV2::SurrogateCreationReceipt,
                document_sha256_v2(&creation)?,
            )],
        )?;

        let identity = IdentityBindingPacketV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            creation_receipt_sha256: document_sha256_v2(&creation)?,
            candidate_identity_packet_sha256: document_sha256_v2(candidate)?,
            finalizer_identity: candidate.finalizer_identity.clone(),
            coordinator_identity: candidate.coordinator_identity.clone(),
            coordinator_process: coordinator.attestation.process_identity(),
            launch_identity: candidate.launch_identity.clone(),
            capability_digest: candidate.capability_digest.clone(),
            current_lock_identity_sha256: creation.current_lock_identity_sha256.clone(),
            signer_access_control_sha256: creation.target.access_control_sha256.clone(),
        };
        identity.validate_against_creation(repetition, &creation)?;
        identity.validate_candidate(candidate)?;
        let identity_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::IdentityBinding,
            &identity,
        )?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::IdentityBinding,
            &identity,
        )?;
        let unsigned_receipt = build_unsigned_publisher_receipt_v2(
            repetition,
            &creation,
            &identity,
            &harness_signer,
            unix_time_ns()?,
        )?;
        let unsigned_receipt_observation = store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::UnsignedReceipt,
            &unsigned_receipt,
        )?;
        let signed_receipt: PublisherPreRemovalReceiptV2 = publisher_transition_with_output(
            store,
            repetition,
            &mut publisher_progress,
            PublisherStageV2::ReceiptSigned,
            vec![
                publisher_artifact(
                    PublisherInputArtifactV2::CreationReceipt,
                    document_sha256_v2(&creation)?,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::IdentityBinding,
                    identity_observation.sha256,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::UnsignedReceipt,
                    unsigned_receipt_observation.sha256,
                ),
            ],
            PublisherArtifactV2::SignedReceipt,
            "publisher-signed receipt",
        )?;
        *publisher_stage = PublisherStageV2::ReceiptSigned;
        substrate_r3_macos_finalizer::experiment::publisher_protocol::validate_signed_receipt_output_v2(
            repetition,
            &creation,
            &identity,
            &signed_receipt,
        )?;
        let signed_receipt_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::SignedPublisherReceipt,
            &signed_receipt,
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::ReceiptExternallyDurable,
            vec![harness_artifact(
                HarnessInputArtifactV2::SignedPublisherReceipt,
                signed_receipt_observation.sha256.clone(),
            )],
        )?;

        let acknowledgement = build_signed_harness_acknowledgement_v2(
            &harness_signer,
            &signed_receipt,
            &signed_receipt_observation,
            experiment_root_identity_sha256,
            unix_time_ns()?,
        )?;
        let acknowledgement_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::HarnessAcknowledgement,
            &acknowledgement,
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::AcknowledgementExternallyDurable,
            vec![harness_artifact(
                HarnessInputArtifactV2::HarnessAcknowledgement,
                acknowledgement_observation.sha256.clone(),
            )],
        )?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::HarnessAcknowledgement,
            &acknowledgement,
        )?;
        let unsigned_cas = build_unsigned_protected_cas_v2(
            &signed_receipt,
            &acknowledgement,
            &predecessor_journal_head_v2(repetition),
            &retry_state_v2(repetition),
        )?;
        let unsigned_cas_observation = store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::UnsignedProtectedCas,
            &unsigned_cas,
        )?;
        let signed_cas: ProtectedCasBindingV2 = publisher_transition_with_output(
            store,
            repetition,
            &mut publisher_progress,
            PublisherStageV2::ProtectedCasSigned,
            vec![
                publisher_artifact(
                    PublisherInputArtifactV2::SignedReceipt,
                    document_sha256_v2(&signed_receipt)?,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::HarnessAcknowledgement,
                    acknowledgement_observation.sha256,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::UnsignedProtectedCas,
                    unsigned_cas_observation.sha256,
                ),
            ],
            PublisherArtifactV2::SignedProtectedCas,
            "signed protected CAS binding",
        )?;
        *publisher_stage = PublisherStageV2::ProtectedCasSigned;
        substrate_r3_macos_finalizer::experiment::publisher_protocol::validate_signed_cas_output_v2(
            &signed_receipt,
            &acknowledgement,
            &signed_cas,
        )?;
        let signed_cas_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::SignedProtectedCas,
            &signed_cas,
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::AcknowledgementCasBound,
            vec![harness_artifact(
                HarnessInputArtifactV2::SignedProtectedCas,
                signed_cas_observation.sha256.clone(),
            )],
        )?;

        let request = build_finalization_request_v2(
            &signed_receipt,
            &acknowledgement,
            &signed_cas,
            predecessor_journal_head_v2(repetition),
            retry_state_v2(repetition),
        )?;
        let request_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::FrozenFinalizationRequest,
            &request,
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::FinalizationRequestFrozen,
            vec![harness_artifact(
                HarnessInputArtifactV2::FrozenFinalizationRequest,
                request_observation.sha256.clone(),
            )],
        )?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::FinalizationRequest,
            &request,
        )?;
        let request_publication_step = publisher_transition_without_output(
            store,
            repetition,
            &mut publisher_progress,
            PublisherStageV2::FinalizationRequestPublished,
            vec![
                publisher_artifact(
                    PublisherInputArtifactV2::SignedReceipt,
                    document_sha256_v2(&signed_receipt)?,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::HarnessAcknowledgement,
                    document_sha256_v2(&acknowledgement)?,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::SignedProtectedCas,
                    signed_cas_observation.sha256,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::FinalizationRequest,
                    request_observation.sha256,
                ),
            ],
            document_sha256_v2(&request)?,
        )?;
        *publisher_stage = PublisherStageV2::FinalizationRequestPublished;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::FinalizationRequestPublished,
            vec![harness_artifact(
                HarnessInputArtifactV2::RequestPublicationReceipt,
                document_sha256_v2(&request_publication_step)?,
            )],
        )?;

        let mut control_receipts = Vec::with_capacity(TRANSPORT_CONTROL_SEQUENCE_V2.len());
        for _ in TRANSPORT_CONTROL_SEQUENCE_V2 {
            let receipt: TransportControlReceiptV2 = read_coordinator_frame(&mut coordinator)?;
            receipt.validate(repetition)?;
            control_receipts.push(receipt);
        }
        let ready = substrate_r3_macos_finalizer::experiment::controls::PeerControlsReadyMarkerV2 {
            schema_owner: PEER_CONTROL_RECEIPT_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            request_digest: request.request_digest.clone(),
            request_control_receipt_sha256: control_receipts
                .iter()
                .map(document_sha256_v2)
                .collect::<Result<Vec<_>>>()?,
            before_observation_sha256: signed_receipt.quiesced_observation_sha256.clone(),
        };
        ready.validate(repetition, &request, &control_receipts)?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::PeerControlsReadyMarker,
            &ready,
        )?;
        let peer_set: PeerControlSetReceiptV2 = wait_publisher_output(
            store,
            repetition,
            PublisherArtifactV2::PeerControlRunnerSetReceipt,
            "root-runner peer-control receipt set",
        )?;
        peer_set.validate(repetition, &request, peer_identities)?;
        peer_set
            .nobody_owner_authority
            .validate_against_creation(repetition, &creation)?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::PeerControlSetReceipt,
            &peer_set,
        )?;
        let control_set: TransportControlSetReceiptV2 = read_coordinator_frame(&mut coordinator)?;
        control_set.validate(repetition, &control_receipts)?;
        store.persist_canonical(
            repetition,
            ExperimentArtifactV2::TransportControlReceipts,
            &CoordinatorReceiptSetV2 {
                schema_owner: EXPERIMENT_OWNER_V2,
                schema_version: EXPERIMENT_VERSION_V2,
                controls: &control_receipts,
                set: &control_set,
            },
        )?;
        let effects_arm: DisposableNativeArmReceiptV2 = read_coordinator_frame(&mut coordinator)?;
        let effects: FinalizerResponseV2 = read_coordinator_frame(&mut coordinator)?;
        validate_finalizer_response_v2(&effects)?;
        effects_arm.validate(repetition, &effects)?;
        if effects_arm.arm != DisposableNativeArmV2::FinalizationEffects
            || effects.state != FinalizerResponseStateV2::EffectsComplete
            || effects.request_digest != request.request_digest
        {
            bail!("exact finalizer did not reach EffectsComplete for the frozen request")
        }
        let effects_observation =
            store.persist_canonical(repetition, ExperimentArtifactV2::EffectsResponse, &effects)?;
        let effects_arm_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::EffectsNativeArmReceipt,
            &effects_arm,
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::FinalizerAccepted,
            vec![harness_artifact(
                HarnessInputArtifactV2::FinalizerAcceptanceEvidence,
                effects_observation.sha256.clone(),
            )],
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::EffectsComplete,
            vec![
                harness_artifact(
                    HarnessInputArtifactV2::EffectsResponse,
                    effects_observation.sha256.clone(),
                ),
                harness_artifact(
                    HarnessInputArtifactV2::EffectsNativeArmReceipt,
                    effects_arm_observation.sha256,
                ),
            ],
        )?;

        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::EffectsCompleteResponse,
            &effects,
        )?;
        let residual: ResidualCleanupReceiptV2 = publisher_transition_with_output(
            store,
            repetition,
            &mut publisher_progress,
            PublisherStageV2::ResidualRemoved,
            vec![publisher_artifact(
                PublisherInputArtifactV2::EffectsCompleteResponse,
                effects_observation.sha256.clone(),
            )],
            PublisherArtifactV2::ResidualCleanupReceipt,
            "residual cleanup receipt",
        )?;
        *publisher_stage = PublisherStageV2::ResidualRemoved;
        residual.validate(repetition, &effects_observation.sha256)?;
        if residual.exact_after_observation_sha256 != creation.before_observation_sha256 {
            bail!("post-effects exact surrogate observation differs from the captured baseline")
        }
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::HarnessResidualRemoving,
            vec![harness_artifact(
                HarnessInputArtifactV2::ResidualCleanupReceipt,
                document_sha256_v2(&residual)?,
            )],
        )?;

        let (parity, terminal) = build_signed_parity_and_terminal_v2(
            repetition,
            &harness_signer,
            &request,
            &effects,
            creation.before_observation_sha256.clone(),
            residual.exact_after_observation_sha256.clone(),
            unix_time_ns()?,
        )?;
        let parity_observation =
            store.persist_canonical(repetition, ExperimentArtifactV2::ParityProof, &parity)?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::ParityExternallyDurable,
            vec![harness_artifact(
                HarnessInputArtifactV2::ParityProof,
                parity_observation.sha256,
            )],
        )?;
        let terminal_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::TerminalAcknowledgement,
            &terminal,
        )?;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::TerminalAcknowledgementBound,
            vec![harness_artifact(
                HarnessInputArtifactV2::TerminalAcknowledgement,
                terminal_observation.sha256.clone(),
            )],
        )?;
        let terminal_binding =
            build_terminal_binding_request_v2(&request, &effects, &parity, &terminal)?;
        store.persist_canonical(
            repetition,
            ExperimentArtifactV2::TerminalBinding,
            &terminal_binding,
        )?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::TerminalAcknowledgement,
            &terminal,
        )?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::TerminalBinding,
            &terminal_binding,
        )?;
        let terminal_publication_step = publisher_transition_without_output(
            store,
            repetition,
            &mut publisher_progress,
            PublisherStageV2::TerminalBindingPublished,
            vec![
                publisher_artifact(
                    PublisherInputArtifactV2::TerminalAcknowledgement,
                    terminal_observation.sha256,
                ),
                publisher_artifact(
                    PublisherInputArtifactV2::TerminalBinding,
                    document_sha256_v2(&terminal_binding)?,
                ),
            ],
            document_sha256_v2(&terminal_binding)?,
        )?;
        *publisher_stage = PublisherStageV2::TerminalBindingPublished;
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::TerminalBindingPublished,
            vec![harness_artifact(
                HarnessInputArtifactV2::TerminalPublicationReceipt,
                document_sha256_v2(&terminal_publication_step)?,
            )],
        )?;

        let complete_arm: DisposableNativeArmReceiptV2 = read_coordinator_frame(&mut coordinator)?;
        let complete: FinalizerResponseV2 = read_coordinator_frame(&mut coordinator)?;
        validate_finalizer_response_v2(&complete)?;
        complete_arm.validate(repetition, &complete)?;
        if complete_arm.arm != DisposableNativeArmV2::TerminalBinding
            || complete.state != FinalizerResponseStateV2::HostComplete
            || complete.request_digest != request.request_digest
        {
            bail!("exact finalizer did not return HostComplete for the frozen request")
        }
        let complete_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::CompleteResponse,
            &complete,
        )?;
        let complete_arm_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::CompleteNativeArmReceipt,
            &complete_arm,
        )?;
        let status = coordinator
            .child
            .wait()
            .context("wait for exact coordinator")?;
        if !status.success() {
            return Err(coordinator_exit_error(
                status,
                "after exact terminal framing",
            ));
        }
        advance_harness(
            store,
            repetition,
            &mut harness_progress,
            HarnessStageV2::Complete,
            vec![
                harness_artifact(
                    HarnessInputArtifactV2::CompleteResponse,
                    complete_observation.sha256.clone(),
                ),
                harness_artifact(
                    HarnessInputArtifactV2::CompleteNativeArmReceipt,
                    complete_arm_observation.sha256,
                ),
            ],
        )?;

        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::CompleteResponse,
            &complete,
        )?;
        let restoration: RestorationReceiptV2 = publisher_transition_with_output(
            store,
            repetition,
            &mut publisher_progress,
            PublisherStageV2::RestorationComplete,
            vec![publisher_artifact(
                PublisherInputArtifactV2::CompleteResponse,
                complete_observation.sha256.clone(),
            )],
            PublisherArtifactV2::RestorationReceipt,
            "restoration receipt",
        )?;
        *publisher_stage = PublisherStageV2::RestorationComplete;
        restoration.validate(repetition, &complete_observation.sha256)?;
        Ok(PendingRestorationV2 {
            repetition,
            request,
            peer_set,
            transport_controls: control_receipts,
            effects_arm,
            complete_arm,
            complete,
            baseline_packet_sha256: baseline_observation.sha256,
            surrogate_before_observation_sha256: creation.before_observation_sha256,
            exact_after_observation_sha256: residual.exact_after_observation_sha256,
            restoration,
            complete_response_sha256: complete_observation.sha256,
            denylist_proof,
            seed_observation,
            harness_progress,
        })
    }

    fn build_securityagent_expected_sources(
        store: &ExperimentStoreV2,
        global_pre_effect: &GlobalPreEffectPacketV2,
        creator_receipts: &CreatorRouteReceiptSetV2,
        pending: &[PendingRestorationV2],
    ) -> Result<Vec<SecurityAgentExpectedSourceV2>> {
        let mut sources = Vec::new();
        push_single_ui_source(
            &mut sources,
            0,
            SecurityAgentEvidenceArmV2::GlobalNonceBaseline,
            document_sha256_v2(&global_pre_effect.nonce_absence_baseline)?,
            global_pre_effect
                .nonce_absence_baseline
                .securityagent_report
                .clone(),
        )?;
        for receipt in &creator_receipts.receipts {
            push_single_ui_source(
                &mut sources,
                receipt.repetition,
                SecurityAgentEvidenceArmV2::Creator {
                    control: receipt.arm,
                },
                document_sha256_v2(receipt)?,
                receipt.securityagent_report.clone(),
            )?;
        }
        for pending in pending {
            for stage in PUBLISHER_STAGE_SEQUENCE_V2.into_iter().skip(1) {
                let (observation, file_observation) = store
                    .read_publisher_output::<PublisherSecurityAgentObservationV2>(
                        pending.repetition,
                        PublisherArtifactV2::SecurityAgentObservation(stage),
                    )?;
                observation.validate(pending.repetition, stage)?;
                if file_observation.sha256 != document_sha256_v2(&observation)? {
                    bail!("publisher SecurityAgent observation changed after canonical reopen")
                }
                push_ui_source(
                    &mut sources,
                    pending.repetition.ordinal(),
                    SecurityAgentEvidenceArmV2::Publisher { stage },
                    file_observation.sha256,
                    observation.arm_evidence,
                )?;
            }
            for receipt in &pending.transport_controls {
                push_ui_source(
                    &mut sources,
                    pending.repetition.ordinal(),
                    SecurityAgentEvidenceArmV2::Transport {
                        control: receipt.control,
                    },
                    document_sha256_v2(receipt)?,
                    receipt.securityagent_evidence.clone(),
                )?;
            }
            for receipt in &pending.peer_set.receipts {
                push_ui_source(
                    &mut sources,
                    pending.repetition.ordinal(),
                    SecurityAgentEvidenceArmV2::Peer {
                        control: receipt.control,
                    },
                    document_sha256_v2(receipt)?,
                    receipt.securityagent_evidence.clone(),
                )?;
            }
            push_single_ui_source(
                &mut sources,
                pending.repetition.ordinal(),
                SecurityAgentEvidenceArmV2::BenignInjection,
                document_sha256_v2(&pending.peer_set.dynamic_library_injection)?,
                pending
                    .peer_set
                    .dynamic_library_injection
                    .securityagent_report
                    .clone(),
            )?;
            for attempt in &pending.peer_set.nobody_owner_authority.attempts {
                push_single_ui_source(
                    &mut sources,
                    pending.repetition.ordinal(),
                    SecurityAgentEvidenceArmV2::NobodyOwner {
                        operation: attempt.operation,
                    },
                    document_sha256_v2(attempt)?,
                    attempt.securityagent_report.clone(),
                )?;
            }
            push_ui_source(
                &mut sources,
                pending.repetition.ordinal(),
                SecurityAgentEvidenceArmV2::FinalizationEffects,
                document_sha256_v2(&pending.effects_arm)?,
                pending.effects_arm.securityagent_evidence.clone(),
            )?;
            push_ui_source(
                &mut sources,
                pending.repetition.ordinal(),
                SecurityAgentEvidenceArmV2::TerminalBinding,
                document_sha256_v2(&pending.complete_arm)?,
                pending.complete_arm.securityagent_evidence.clone(),
            )?;
        }
        Ok(sources)
    }

    fn push_single_ui_source(
        sources: &mut Vec<SecurityAgentExpectedSourceV2>,
        repetition: u8,
        arm: SecurityAgentEvidenceArmV2,
        source_artifact_sha256: String,
        report: substrate_r3_macos_finalizer::experiment::controls::SecurityAgentRawReportEvidenceV2,
    ) -> Result<()> {
        push_ui_source(
            sources,
            repetition,
            arm,
            source_artifact_sha256,
            single_securityagent_arm_evidence_v2(report)?,
        )
    }

    fn push_ui_source(
        sources: &mut Vec<SecurityAgentExpectedSourceV2>,
        repetition: u8,
        arm: SecurityAgentEvidenceArmV2,
        source_artifact_sha256: String,
        arm_evidence: SecurityAgentArmEvidenceV2,
    ) -> Result<()> {
        arm_evidence.validate()?;
        sources.push(SecurityAgentExpectedSourceV2 {
            repetition,
            arm,
            source_artifact_sha256,
            arm_evidence_sha256: document_sha256_v2(&arm_evidence)?,
        });
        Ok(())
    }

    fn finalize_restoration(
        store: &ExperimentStoreV2,
        mut pending: PendingRestorationV2,
        native_evidence_export: &NativeEvidenceExportV2,
        native_evidence_acknowledgement: &NativeEvidenceExportAcknowledgementV2,
        native_evidence_cleanup: &NativeEvidenceCleanupReceiptV2,
        peer_restoration: &PeerControlRestorationReceiptV2,
    ) -> Result<()> {
        let repetition = pending.repetition;
        let repetition_index = usize::from(repetition.ordinal() - 1);
        let manifest = build_restoration_manifest_v2(
            repetition,
            RestorationManifestInputV2 {
                baseline_packet_sha256: pending.baseline_packet_sha256,
                surrogate_before_observation_sha256: pending.surrogate_before_observation_sha256,
                exact_after_observation_sha256: pending.exact_after_observation_sha256,
                restoration_receipt_sha256: document_sha256_v2(&pending.restoration)?,
                native_evidence_export_sha256: document_sha256_v2(native_evidence_export)?,
                native_evidence_artifact_set_sha256: native_evidence_export
                    .artifact_set_sha256
                    .clone(),
                raw_securityagent_archive_sha256: native_evidence_export
                    .raw_securityagent_archive_sha256
                    .clone(),
                native_evidence_acknowledgement_sha256: document_sha256_v2(
                    native_evidence_acknowledgement,
                )?,
                native_evidence_cleanup_receipt_sha256: document_sha256_v2(
                    native_evidence_cleanup,
                )?,
                journal_absence_observation_sha256: native_evidence_cleanup
                    .journal_absence_observation_sha256[repetition_index]
                    .clone(),
                capability_absence_observation_sha256: native_evidence_cleanup
                    .capability_absence_observation_sha256[repetition_index]
                    .clone(),
                latch_absence_observation_sha256: native_evidence_cleanup
                    .latch_absence_observation_sha256[repetition_index]
                    .clone(),
                peer_control_restoration_receipt_sha256: document_sha256_v2(peer_restoration)?,
                complete_response_sha256: pending.complete_response_sha256,
            },
            &pending.denylist_proof,
        )?;
        manifest.validate(repetition, &pending.denylist_proof)?;
        let manifest_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::RestorationManifest,
            &manifest,
        )?;
        let seed_identity_sha256 = pending.seed_observation.identity_sha256()?;
        store.remove_exact_signing_seed(repetition, &pending.seed_observation)?;
        let seed_removal = SigningSeedRemovalObservationV2 {
            schema_owner: EXPERIMENT_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            seed_file_identity_sha256: seed_identity_sha256,
            removed_after_complete_and_restoration: true,
            exact_path_absent: true,
        };
        seed_removal.validate(repetition)?;
        let seed_removal_observation = store.persist_canonical(
            repetition,
            ExperimentArtifactV2::HarnessSigningSeedRemoval,
            &seed_removal,
        )?;
        advance_harness(
            store,
            repetition,
            &mut pending.harness_progress,
            HarnessStageV2::RestorationComplete,
            vec![
                harness_artifact(
                    HarnessInputArtifactV2::RestorationReceipt,
                    document_sha256_v2(&pending.restoration)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::NativeEvidenceExport,
                    document_sha256_v2(native_evidence_export)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::NativeEvidenceExportAcknowledgement,
                    document_sha256_v2(native_evidence_acknowledgement)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::NativeEvidenceCleanupReceipt,
                    document_sha256_v2(native_evidence_cleanup)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::PeerControlRestorationReceipt,
                    document_sha256_v2(&peer_restoration)?,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::RestorationManifest,
                    manifest_observation.sha256,
                ),
                harness_artifact(
                    HarnessInputArtifactV2::SigningSeedRemovalObservation,
                    seed_removal_observation.sha256,
                ),
            ],
        )?;
        Ok(())
    }

    fn publisher_transition_with_output<T: DeserializeOwned + Serialize>(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        progress: &mut PublisherProgressV2,
        next: PublisherStageV2,
        artifacts: Vec<PublisherInputArtifactDigestV2>,
        output_artifact: PublisherArtifactV2,
        label: &str,
    ) -> Result<T> {
        let (bundle, marker) =
            publish_publisher_transition(store, repetition, progress, next, artifacts)?;
        let output: T = wait_publisher_output(store, repetition, output_artifact, label)?;
        let expected_output_sha256 = document_sha256_v2(&output)?;
        finish_publisher_transition(
            store,
            repetition,
            progress,
            &bundle,
            &marker,
            expected_output_sha256,
        )?;
        Ok(output)
    }

    fn publisher_transition_without_output(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        progress: &mut PublisherProgressV2,
        next: PublisherStageV2,
        artifacts: Vec<PublisherInputArtifactDigestV2>,
        expected_output_sha256: String,
    ) -> Result<PublisherStepReceiptV2> {
        let (bundle, marker) =
            publish_publisher_transition(store, repetition, progress, next, artifacts)?;
        finish_publisher_transition(
            store,
            repetition,
            progress,
            &bundle,
            &marker,
            expected_output_sha256,
        )
    }

    fn publish_publisher_transition(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        progress: &PublisherProgressV2,
        next: PublisherStageV2,
        artifacts: Vec<PublisherInputArtifactDigestV2>,
    ) -> Result<(PublisherInputBundleV2, PublisherAdvanceMarkerV2)> {
        progress.stage.require_successor(next)?;
        let bundle = PublisherInputBundleV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            prior_stage: progress.stage,
            next_stage: next,
            artifacts,
        };
        bundle.validate(repetition)?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::InputBundle(next),
            &bundle,
        )?;
        let marker = PublisherAdvanceMarkerV2 {
            schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2.to_string(),
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id().to_string(),
            prior_stage: progress.stage,
            next_stage: next,
            fixed_input_sha256: document_sha256_v2(&bundle)?,
        };
        marker.validate_input_bundle(repetition, &bundle)?;
        store.persist_publisher_input(
            repetition,
            PublisherArtifactV2::AdvanceMarker(progress.stage),
            &marker,
        )?;
        Ok((bundle, marker))
    }

    fn finish_publisher_transition(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        progress: &mut PublisherProgressV2,
        bundle: &PublisherInputBundleV2,
        marker: &PublisherAdvanceMarkerV2,
        expected_output_sha256: String,
    ) -> Result<PublisherStepReceiptV2> {
        let step: PublisherStepReceiptV2 = wait_publisher_output(
            store,
            repetition,
            PublisherArtifactV2::StepReceipt(bundle.next_stage),
            "publisher step receipt",
        )?;
        step.validate(repetition, marker, &expected_output_sha256)?;
        let ui: PublisherSecurityAgentObservationV2 = wait_publisher_output(
            store,
            repetition,
            PublisherArtifactV2::SecurityAgentObservation(bundle.next_stage),
            "publisher SecurityAgent observation",
        )?;
        step.validate_securityagent_observation(repetition, &ui)?;
        let successor = wait_publisher_progress(store, repetition, bundle.next_stage)?;
        successor.validate_successor(repetition, progress, marker, bundle, &step)?;
        *progress = successor;
        Ok(step)
    }

    fn advance_harness(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        progress: &mut HarnessProgressV2,
        next: HarnessStageV2,
        artifacts: Vec<HarnessInputArtifactDigestV2>,
    ) -> Result<()> {
        let (bundle, step, successor) =
            build_harness_transition_v2(repetition, progress, next, artifacts)?;
        store.persist_canonical(
            repetition,
            ExperimentArtifactV2::HarnessInputBundle(next),
            &bundle,
        )?;
        store.persist_canonical(
            repetition,
            ExperimentArtifactV2::HarnessStepReceipt(next),
            &step,
        )?;
        store.persist_canonical(
            repetition,
            ExperimentArtifactV2::HarnessProgress(next),
            &successor,
        )?;
        *progress = successor;
        Ok(())
    }

    fn spawn_and_attest_coordinator(
        identity: &substrate_common::macos_retirement_v2::ExecutableIdentityV2,
    ) -> Result<CoordinatorChildV2> {
        let mut command = Command::new(MAC_R3_COORDINATOR_PATH_V2);
        command
            .current_dir("/")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .env_clear();
        let mut child = command
            .spawn()
            .context("spawn exact evidence coordinator")?;
        let pid = i32::try_from(child.id()).context("coordinator child PID exceeds i32")?;
        let attestation = match attest_coordinator_process_v2(pid, identity) {
            Ok(attestation) => attestation,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error).context("attest exact evidence coordinator");
            }
        };
        let stdout = child
            .stdout
            .take()
            .context("exact coordinator has no receipt stream")?;
        Ok(CoordinatorChildV2 {
            child,
            stdout,
            attestation,
        })
    }

    fn read_coordinator_frame<T: DeserializeOwned + Serialize>(
        coordinator: &mut CoordinatorChildV2,
    ) -> Result<T> {
        let mut prefix = [0_u8; 8];
        if let Err(error) = coordinator.stdout.read_exact(&mut prefix) {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                let status = coordinator
                    .child
                    .wait()
                    .context("reap coordinator after receipt EOF")?;
                return Err(coordinator_exit_error(
                    status,
                    "while reading receipt prefix",
                ));
            }
            if let Some(status) = coordinator.child.try_wait()? {
                return Err(coordinator_exit_error(
                    status,
                    "while reading receipt prefix",
                ));
            }
            return Err(error).context("read coordinator frame prefix");
        }
        let length = usize::try_from(u64::from_be_bytes(prefix))
            .context("coordinator frame length exceeds usize")?;
        if length == 0 || length > MAC_R3_FINALIZER_MAX_FRAME_BYTES_V2 {
            bail!("coordinator frame length is outside the fixed bound")
        }
        let mut bytes = vec![0_u8; length];
        if let Err(error) = coordinator.stdout.read_exact(&mut bytes) {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                let status = coordinator
                    .child
                    .wait()
                    .context("reap coordinator after receipt-body EOF")?;
                return Err(coordinator_exit_error(status, "while reading receipt body"));
            }
            if let Some(status) = coordinator.child.try_wait()? {
                return Err(coordinator_exit_error(status, "while reading receipt body"));
            }
            return Err(error).context("read coordinator frame body");
        }
        parse_canonical_v2(&bytes)
    }

    fn coordinator_exit_error(
        status: std::process::ExitStatus,
        phase: &'static str,
    ) -> anyhow::Error {
        if status.code() == Some(SECURITYAGENT_ALERT_EXIT_CODE) {
            return SecurityAgentAlertStopV2.into();
        }
        anyhow::anyhow!("exact coordinator exited {phase} with status {status}")
    }

    fn emergency_failure_classification(error: &anyhow::Error) -> EmergencyFailureClassificationV2 {
        if error
            .chain()
            .any(|cause| cause.downcast_ref::<SecurityAgentAlertStopV2>().is_some())
        {
            EmergencyFailureClassificationV2::SecurityAgentAlertExit86
        } else {
            EmergencyFailureClassificationV2::ClosedFailure
        }
    }

    fn request_emergency_rollback(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        publisher_stage: PublisherStageV2,
        creation: Option<&SurrogateCreationReceiptV2>,
        failure_classification: EmergencyFailureClassificationV2,
        error: &str,
    ) -> Result<()> {
        let failure_observation_sha256 = document_sha256_v2(&FailureObservationV2 {
            schema_owner: EXPERIMENT_OWNER_V2,
            schema_version: EXPERIMENT_VERSION_V2,
            experiment_id: EXPERIMENT_ID_V2,
            repetition: repetition.ordinal(),
            scope_id: repetition.scope_id(),
            publisher_stage,
            failure_classification,
            error,
        })?;
        if let Some(creation) = creation {
            let marker = EmergencyRollbackMarkerV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                last_durable_stage: publisher_stage,
                failure_classification,
                failure_observation_sha256,
                target_identity_sha256: creation.target.identity_sha256.clone(),
                wrong_identity_sha256: creation.wrong.identity_sha256.clone(),
            };
            marker.validate(repetition, creation)?;
            store.persist_publisher_input(
                repetition,
                PublisherArtifactV2::EmergencyRollbackMarker,
                &marker,
            )?;
            let receipt: EmergencyRollbackReceiptV2 = wait_publisher_output(
                store,
                repetition,
                PublisherArtifactV2::EmergencyRollbackReceipt,
                "emergency rollback receipt",
            )?;
            receipt.validate(repetition, &marker)
        } else {
            let marker = PreCreationEmergencyRollbackMarkerV2 {
                schema_owner: PUBLISHER_PROTOCOL_OWNER_V2.to_string(),
                schema_version: EXPERIMENT_VERSION_V2,
                experiment_id: EXPERIMENT_ID_V2.to_string(),
                repetition: repetition.ordinal(),
                scope_id: repetition.scope_id().to_string(),
                target_label: format!("{}:signing-key", repetition.scope_id()),
                wrong_label: format!("{}:wrong-surrogate-signing-key", repetition.scope_id()),
                failure_classification,
                failure_observation_sha256,
            };
            marker.validate(repetition)?;
            store.persist_publisher_input(
                repetition,
                PublisherArtifactV2::PreCreationEmergencyRollbackMarker,
                &marker,
            )?;
            let receipt: EmergencyRollbackReceiptV2 = wait_publisher_output(
                store,
                repetition,
                PublisherArtifactV2::EmergencyRollbackReceipt,
                "pre-creation emergency rollback receipt",
            )?;
            receipt.validate_precreation(repetition, &marker)
        }
    }

    fn wait_publisher_output<T: DeserializeOwned + Serialize>(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        artifact: PublisherArtifactV2,
        label: &str,
    ) -> Result<T> {
        let deadline = Instant::now() + WAIT_TIMEOUT;
        loop {
            match store.read_publisher_output::<T>(repetition, artifact) {
                Ok((value, _)) => return Ok(value),
                Err(error) if is_not_found(&error) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                bail!("{label} did not arrive before the fixed handoff deadline; inconclusive")
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn wait_global_peer_restoration(
        store: &ExperimentStoreV2,
    ) -> Result<PeerControlRestorationReceiptV2> {
        let deadline = Instant::now() + WAIT_TIMEOUT;
        loop {
            match store.read_global_publisher_output::<PeerControlRestorationReceiptV2>(
                PublisherArtifactV2::PeerControlRestorationReceipt,
            ) {
                Ok((value, _)) => return Ok(value),
                Err(error) if is_not_found(&error) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                bail!("global peer-control restoration did not arrive after both HostComplete states; inconclusive")
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn wait_global_publisher_output<T: DeserializeOwned + Serialize>(
        store: &ExperimentStoreV2,
        artifact: PublisherArtifactV2,
        label: &str,
    ) -> Result<T> {
        let deadline = Instant::now() + WAIT_TIMEOUT;
        loop {
            match store.read_global_publisher_output::<T>(artifact) {
                Ok((value, _)) => return Ok(value),
                Err(error) if is_not_found(&error) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                bail!("{label} did not arrive before the fixed handoff deadline; inconclusive")
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn wait_publisher_progress(
        store: &ExperimentStoreV2,
        repetition: RepetitionV2,
        stage: PublisherStageV2,
    ) -> Result<PublisherProgressV2> {
        let deadline = Instant::now() + WAIT_TIMEOUT;
        loop {
            match store.read_publisher_output::<PublisherProgressV2>(
                repetition,
                PublisherArtifactV2::Progress,
            ) {
                Ok((value, _)) if value.stage == stage => return Ok(value),
                Ok((value, _)) if value.stage > stage => {
                    bail!("publisher progress skipped the expected closed stage")
                }
                Ok(_) => {}
                Err(error) if is_not_found(&error) => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                bail!("publisher progress did not reach the expected stage; inconclusive")
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn publisher_artifact(
        artifact: PublisherInputArtifactV2,
        sha256: String,
    ) -> PublisherInputArtifactDigestV2 {
        PublisherInputArtifactDigestV2 { artifact, sha256 }
    }

    fn harness_artifact(
        artifact: HarnessInputArtifactV2,
        sha256: String,
    ) -> HarnessInputArtifactDigestV2 {
        HarnessInputArtifactDigestV2 { artifact, sha256 }
    }

    fn read_installed_packet<T: DeserializeOwned + Serialize>(path: &Path) -> Result<T> {
        require_root_owned_immutable_path(path)?;
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .with_context(|| format!("open installed packet {}", path.display()))?;
        let before = file.metadata().context("fstat installed packet")?;
        if !before.file_type().is_file()
            || before.uid() != 0
            || before.mode() & 0o7777 != INSTALLED_PACKET_MODE
            || before.nlink() != 1
        {
            bail!("installed packet owner, mode, type, or link count changed")
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .context("read installed packet")?;
        let after = file.metadata().context("refstat installed packet")?;
        if physical_fields(&before) != physical_fields(&after) {
            bail!("installed packet changed while reading")
        }
        parse_canonical_v2(&bytes)
    }

    fn require_root_owned_immutable_path(path: &Path) -> Result<()> {
        if !path.is_absolute() {
            bail!("installed packet path is not absolute")
        }
        let mut current = PathBuf::from("/");
        for component in path.components().skip(1) {
            current.push(component.as_os_str());
            let metadata = std::fs::symlink_metadata(&current)
                .with_context(|| format!("inspect installed path {}", current.display()))?;
            if metadata.file_type().is_symlink()
                || metadata.uid() != 0
                || metadata.mode() & 0o022 != 0
            {
                bail!("installed packet path is not root-owned immutable no-follow state")
            }
            if current == path {
                if !metadata.file_type().is_file() || metadata.nlink() != 1 {
                    bail!("installed packet is not one regular-file identity")
                }
            } else if !metadata.is_dir() {
                bail!("installed packet parent is not a directory")
            }
        }
        Ok(())
    }

    fn require_closed_harness_surface() -> Result<()> {
        if std::env::args_os().count() != 1 {
            bail!("disposable harness accepts no argv")
        }
        if unsafe { libc::geteuid() } != DISPOSABLE_HARNESS_UID_V2 {
            bail!("disposable harness is not running as the fixed UID501 coordinator account")
        }
        if std::env::current_dir().context("read harness cwd")? != Path::new("/") {
            bail!("disposable harness cwd is not the fixed root directory")
        }
        let stdin = std::fs::metadata("/dev/fd/0").context("inspect harness stdin")?;
        let dev_null = std::fs::metadata("/dev/null").context("inspect fixed /dev/null")?;
        if stdin.file_type() != dev_null.file_type()
            || stdin.dev() != dev_null.dev()
            || stdin.ino() != dev_null.ino()
            || stdin.rdev() != dev_null.rdev()
        {
            bail!("disposable harness stdin is not the fixed /dev/null EOF source")
        }
        if std::env::vars_os().any(|(key, _)| key.to_string_lossy().starts_with("SUBSTRATE_")) {
            bail!("disposable harness rejects ambient SUBSTRATE_* input")
        }
        let executable = std::fs::canonicalize(std::env::current_exe()?)?;
        if executable != Path::new(DISPOSABLE_HARNESS_PATH_V2) {
            bail!("disposable harness is not running from its frozen physical path")
        }
        require_root_owned_immutable_path(&executable)?;
        let _ = DISPOSABLE_HARNESS_ACCOUNT_V2;
        Ok(())
    }

    fn clear_process_environment() -> Result<()> {
        substrate_r3_macos_finalizer::ambient::clear_and_require_empty_v2("disposable harness")
    }

    fn unix_time_ns() -> Result<u64> {
        u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("system clock precedes Unix epoch")?
                .as_nanos(),
        )
        .context("Unix nanoseconds exceed u64")
    }

    fn is_not_found(error: &anyhow::Error) -> bool {
        error
            .chain()
            .filter_map(|cause| cause.downcast_ref::<std::io::Error>())
            .any(|cause| cause.kind() == std::io::ErrorKind::NotFound)
    }

    fn physical_fields(
        metadata: &std::fs::Metadata,
    ) -> (u64, u64, u32, u32, u32, u64, u64, i64, i64) {
        (
            metadata.dev(),
            metadata.ino(),
            metadata.uid(),
            metadata.gid(),
            metadata.mode(),
            metadata.nlink(),
            metadata.size(),
            metadata.mtime(),
            metadata.mtime_nsec(),
        )
    }

    #[cfg(test)]
    mod tests {
        use std::os::unix::process::ExitStatusExt;

        use super::*;

        #[test]
        fn harness_source_has_no_arbitrary_mode_action_or_target_surface() {
            let source = include_str!("experiment_harness.rs");
            let production = source.split("#[cfg(test)]").next().unwrap();
            assert!(production.contains("args_os().count() != 1"));
            assert!(!production.contains("match std::env::args"));
            assert!(!production.contains("--action"));
            assert!(!production.contains("--path"));
            assert!(!production.contains("--target"));
            assert!(production.contains("PublisherInputArtifactV2"));
            assert!(production.contains("PreCreationEmergencyRollbackMarkerV2"));
            assert!(production.contains("load_or_create_signing_seed"));
            assert!(production.contains("clear_process_environment()?"));
            let ambient = include_str!("../ambient.rs");
            assert!(ambient.contains("libc::unsetenv(name.as_ptr())"));
            assert!(ambient.contains("vars_os().next().is_some()"));
            assert!(production.contains(".env_clear()"));
            assert!(production.contains("/dev/fd/0"));
        }

        #[test]
        fn securityagent_exit86_is_terminal_and_selects_only_alert_rollback() {
            let status = std::process::ExitStatus::from_raw(SECURITYAGENT_ALERT_EXIT_CODE << 8);
            let error = coordinator_exit_error(status, "test").context("receipt read stopped");
            assert_eq!(
                emergency_failure_classification(&error),
                EmergencyFailureClassificationV2::SecurityAgentAlertExit86
            );

            let ordinary = anyhow::anyhow!("ordinary closed failure");
            assert_eq!(
                emergency_failure_classification(&ordinary),
                EmergencyFailureClassificationV2::ClosedFailure
            );

            let source = include_str!("experiment_harness.rs");
            let production = source.split("#[cfg(test)]").next().unwrap();
            assert_eq!(
                production.matches("spawn_and_attest_coordinator(").count(),
                2
            );
            assert!(production.contains("status.code() == Some(SECURITYAGENT_ALERT_EXIT_CODE)"));
            assert!(production.contains("request_emergency_rollback("));
            assert!(!production.contains("SecurityAgentAlertExit86 => continue"));
        }

        #[test]
        fn global_alternate_path_restoration_gates_both_repetition_manifests() {
            let source = include_str!("experiment_harness.rs");
            let production = source.split("#[cfg(test)]").next().unwrap();
            let run = production.find("pub fn run() -> Result<()>").unwrap();
            let both = production[run..]
                .find("for repetition in RepetitionV2::ALL")
                .unwrap()
                + run;
            let global = production[run..]
                .find("wait_global_peer_restoration(&store)")
                .unwrap()
                + run;
            let manifests = production[global..].find("for value in pending").unwrap() + global;
            assert!(both < global && global < manifests);
            assert!(production[manifests..].contains("&peer_restoration,"));
            assert!(production.contains("read_global_publisher_output"));
        }
    }
}
