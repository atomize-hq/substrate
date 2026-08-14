use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};

use anyhow::{bail, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, derive_frozen_finalization_intent_v2, derive_host_effect_plan_v2,
    document_sha256_v2, host_target_role_for_locator_v2, ExecutableIdentityV2, FixedFileRoleV2,
    GenericPasswordRoleV2, GuestRetirementStateV2, GuestToHostSuccessorCapsuleV2,
    HarnessDurabilityAcknowledgementV2, HostResourceLocatorV2, HostRetirementStateV2,
    HostTargetIdentityV2, LaunchIdentityV2, LifecycleDirectoryRoleV2, LifecycleFileRoleV2,
    MacR3SignatureV2, ProcessIdentityV2, ProtectedCasBindingV2, PublisherPreRemovalReceiptV2,
    TargetSetKindV2, MAC_R3_COORDINATOR_PATH_V2, MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
    MAC_R3_FINALIZER_ENDPOINT_V2, MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_PLIST_PATH_V2, MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
    MAC_R3_GUEST_TO_HOST_CAPSULE_OWNER_V2, MAC_R3_HARNESS_ACK_OWNER_V2,
    MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2, MAC_R3_HOST_RECEIPT_OWNER_V2,
    MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2, MAC_R3_PROTECTED_CAS_OWNER_V2,
    MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2,
};

use substrate_r3_macos_finalizer::authority::{
    freeze_finalization_request_v2, validate_authority_frontier_transition_v2, AuthorityGateV2,
    AuthorityPipelineV2, FaultCheckpointV2, ProspectiveAuthorityDocumentsV2,
    R6TerminalPairingHashInputV2, SignatureLaneV2, AUTHORITY_FRONTIERS_V2,
    R6_TERMINAL_PAIRING_INPUT_OWNER_V2,
};

const SCOPE: &str = "019ffe3b-2f34-78d2-afca-8ecfb7b0280a";
const EVIDENCE: &str = "r3-prospective-proof";

fn digest(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

fn executable(path: &str, signing_identifier: &str, seed: u8) -> ExecutableIdentityV2 {
    ExecutableIdentityV2 {
        source_commit: format!("{:x}", seed % 16).repeat(40),
        source_tree: format!("{:x}", (seed + 1) % 16).repeat(40),
        source_hashes_sha256: digest(seed + 2),
        build_inputs_sha256: digest(seed + 3),
        executable_sha256: digest(seed + 4),
        executable_size: 4096,
        intended_path: path.to_string(),
        physical_identity_sha256: digest(seed + 5),
        signing_identifier: signing_identifier.to_string(),
        designated_requirement: format!("identifier {signing_identifier}"),
        cdhash: format!("{:x}", (seed + 6) % 16).repeat(40),
    }
}

fn fixed_signature(algorithm: &str, public_key: &str) -> MacR3SignatureV2 {
    MacR3SignatureV2 {
        algorithm: algorithm.to_string(),
        public_key: public_key.to_string(),
        signature: URL_SAFE_NO_PAD.encode([0xa5_u8; 64]),
    }
}

fn documents() -> ProspectiveAuthorityDocumentsV2 {
    let r6_terminal_pairing = R6TerminalPairingHashInputV2 {
        schema_owner: R6_TERMINAL_PAIRING_INPUT_OWNER_V2.to_string(),
        schema_version: 2,
        scope_id: SCOPE.to_string(),
        predecessor_sha256: digest(1),
        terminal_host_record_sha256: digest(4),
        guest_anchor_acknowledgement_sha256: digest(48),
        guest_consumption_marker_response_sha256: digest(49),
    };
    let finalizer_identity = executable(
        MAC_R3_FINALIZER_PATH_V2,
        MAC_R3_FINALIZER_SIGNING_IDENTIFIER_V2,
        5,
    );
    let coordinator_identity = executable(
        MAC_R3_COORDINATOR_PATH_V2,
        MAC_R3_COORDINATOR_SIGNING_IDENTIFIER_V2,
        7,
    );
    let coordinator_process = ProcessIdentityV2 {
        effective_uid: 0,
        effective_gid: 20,
        canonical_account: "root".to_string(),
        pidversion_required: true,
        process_start_identity_sha256: digest(13),
        executable_identity_sha256: document_sha256_v2(&coordinator_identity).unwrap(),
    };
    use FixedFileRoleV2 as F;
    use GenericPasswordRoleV2 as G;
    use HostResourceLocatorV2 as L;
    let target_ledger: Vec<_> = vec![
        L::PublisherService,
        L::PublisherEndpoint,
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
        L::KeychainCasLock {
            account: G::RetirementResourceIndex,
        },
        L::KeychainCasLock {
            account: G::RetirementCas,
        },
        L::KeychainCasLock {
            account: G::R6TerminalAcknowledgement,
        },
        L::LifecycleTargetLock {
            target: LifecycleFileRoleV2::Head,
        },
        L::LifecycleDirectory {
            role: LifecycleDirectoryRoleV2::LifecycleRoot,
        },
        L::RetirementTerminalLatch,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, locator)| HostTargetIdentityV2 {
        ordinal: u16::try_from(index + 1).unwrap(),
        role: host_target_role_for_locator_v2(&locator),
        locator,
        expected_before_sha256: digest(u8::try_from(index + 20).unwrap()),
    })
    .collect();
    let effect_plan =
        derive_host_effect_plan_v2(TargetSetKindV2::ProspectiveHost, &target_ledger).unwrap();
    let publisher_key = URL_SAFE_NO_PAD.encode([0x04_u8; 65]);
    let harness_key = URL_SAFE_NO_PAD.encode([0x07_u8; 32]);
    let guest_parity_sha256 = digest(51);
    let guest_successor_capsule = GuestToHostSuccessorCapsuleV2 {
        schema_owner: MAC_R3_GUEST_TO_HOST_CAPSULE_OWNER_V2.to_string(),
        schema_version: 2,
        evidence_id: EVIDENCE.to_string(),
        scope_id: SCOPE.to_string(),
        handoff_request_digest: digest(52),
        guest_handoff_capsule_sha256: digest(53),
        guest_host_acceptance_sha256: digest(54),
        guest_receipt_sha256: digest(44),
        guest_acknowledgement_sha256: digest(45),
        guest_protected_cas_head_sha256: digest(46),
        guest_effect_plan_sha256: digest(55),
        guest_effects_response_sha256: digest(47),
        guest_parity_proof_sha256: guest_parity_sha256.clone(),
        guest_parity_host_binding_sha256: digest(56),
        guest_journal_head_sha256: digest(57),
        r6_predecessor_consumed_sha256: r6_terminal_pairing.predecessor_sha256.clone(),
        r6_terminal_host_record_sha256: r6_terminal_pairing.terminal_host_record_sha256.clone(),
        guest_anchor_acknowledgement_sha256: digest(48),
        guest_consumption_marker_acknowledgement_sha256: digest(49),
        retry_state_sha256: digest(50),
    };
    let guest_successor_capsule_sha256 = document_sha256_v2(&guest_successor_capsule).unwrap();
    let receipt = PublisherPreRemovalReceiptV2 {
        schema_owner: MAC_R3_HOST_RECEIPT_OWNER_V2.to_string(),
        schema_version: 2,
        signature_domain: MAC_R3_HOST_RECEIPT_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: EVIDENCE.to_string(),
        scope_id: SCOPE.to_string(),
        target_set_kind: TargetSetKindV2::ProspectiveHost,
        issued_at_unix_ns: 100,
        expires_at_unix_ns: 200,
        host_state: HostRetirementStateV2::PreRemovalReceiptSigned,
        guest_successor_capsule,
        guest_successor_capsule_sha256,
        guest_parity_sha256,
        before_observation_sha256: digest(31),
        quiesced_observation_sha256: digest(32),
        target_ledger_sha256: document_sha256_v2(&target_ledger).unwrap(),
        effect_plan_sha256: document_sha256_v2(&effect_plan).unwrap(),
        target_ledger,
        protected_cas_generation: 7,
        protected_cas_head_sha256: digest(33),
        current_lock_identity_sha256: digest(34),
        signer_access_control_sha256: digest(43),
        publisher_signer_spki_der: publisher_key.clone(),
        harness_public_key: harness_key.clone(),
        finalizer_identity,
        coordinator_identity,
        coordinator_process,
        launch_identity: LaunchIdentityV2 {
            launchd_label: MAC_R3_FINALIZER_LAUNCHD_LABEL_V2.to_string(),
            launchd_plist_path: MAC_R3_FINALIZER_PLIST_PATH_V2.to_string(),
            launchd_plist_sha256: digest(35),
            endpoint: MAC_R3_FINALIZER_ENDPOINT_V2.to_string(),
            endpoint_owner_uid: 0,
            endpoint_group_gid: 20,
            endpoint_mode: "0660".to_string(),
            launch_socket_name: "Listener".to_string(),
            finalizer_effective_uid: 0,
        },
        capability_digest: digest(36),
        signature: fixed_signature("ecdsa-p256-sha256-p1363-low-s-v1", &publisher_key),
    };
    let acknowledgement = HarnessDurabilityAcknowledgementV2 {
        schema_owner: MAC_R3_HARNESS_ACK_OWNER_V2.to_string(),
        schema_version: 2,
        signature_domain: MAC_R3_HARNESS_ACK_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: EVIDENCE.to_string(),
        scope_id: SCOPE.to_string(),
        receipt_sha256: document_sha256_v2(&receipt).unwrap(),
        external_store_identity_sha256: digest(37),
        durable_observation_sha256: digest(38),
        acknowledged_at_unix_ns: 150,
        signature: fixed_signature("ed25519-v1", &harness_key),
    };
    let mut protected_cas_binding = ProtectedCasBindingV2 {
        schema_owner: MAC_R3_PROTECTED_CAS_OWNER_V2.to_string(),
        schema_version: 2,
        signature_domain: MAC_R3_PROTECTED_CAS_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: EVIDENCE.to_string(),
        scope_id: SCOPE.to_string(),
        generation: receipt.protected_cas_generation + 1,
        predecessor_head_sha256: receipt.protected_cas_head_sha256.clone(),
        receipt_sha256: document_sha256_v2(&receipt).unwrap(),
        acknowledgement_sha256: document_sha256_v2(&acknowledgement).unwrap(),
        request_digest: digest(39),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
        guest_successor_capsule_sha256: receipt.guest_successor_capsule_sha256.clone(),
        guest_parity_sha256: receipt.guest_parity_sha256.clone(),
        current_lock_identity_sha256: receipt.current_lock_identity_sha256.clone(),
        signer_access_control_sha256: receipt.signer_access_control_sha256.clone(),
        signature: fixed_signature("ecdsa-p256-sha256-p1363-low-s-v1", &publisher_key),
    };
    let intent =
        derive_frozen_finalization_intent_v2(&receipt, &acknowledgement, &protected_cas_binding)
            .unwrap();
    protected_cas_binding.request_digest = document_sha256_v2(
        &substrate_common::macos_retirement_v2::HostToFinalizerSuccessorCapsuleV2 {
            predecessor_journal_head_sha256: digest(40),
            retry_state_sha256: digest(41),
            intent,
        },
    )
    .unwrap();

    ProspectiveAuthorityDocumentsV2 {
        r6_terminal_pairing,
        receipt,
        acknowledgement,
        protected_cas_binding,
        predecessor_journal_head_sha256: digest(40),
        retry_state_sha256: digest(41),
    }
}

fn verifier(
    seen: &mut Vec<SignatureLaneV2>,
) -> impl FnMut(SignatureLaneV2, &[u8], &MacR3SignatureV2) -> Result<()> + '_ {
    move |lane, payload, signature| {
        if payload.is_empty() || signature.signature != URL_SAFE_NO_PAD.encode([0xa5_u8; 64]) {
            bail!("fixed test signature bytes changed")
        }
        seen.push(lane);
        Ok(())
    }
}

fn no_fault(_: FaultCheckpointV2) -> Result<()> {
    Ok(())
}

#[test]
fn authority_frontiers_have_only_the_exact_successor_chain() {
    for current in AUTHORITY_FRONTIERS_V2 {
        for next in AUTHORITY_FRONTIERS_V2 {
            let current_index = AUTHORITY_FRONTIERS_V2
                .iter()
                .position(|candidate| *candidate == current)
                .unwrap();
            let next_index = AUTHORITY_FRONTIERS_V2
                .iter()
                .position(|candidate| *candidate == next)
                .unwrap();
            assert_eq!(
                validate_authority_frontier_transition_v2(current, next).is_ok(),
                next_index == current_index + 1,
                "frontier edge {:?} -> {:?}",
                current.gate,
                next.gate,
            );
        }
    }
    assert_eq!(
        AUTHORITY_FRONTIERS_V2.last().unwrap().guest,
        GuestRetirementStateV2::HandoffHostBound,
    );
    assert_eq!(
        AUTHORITY_FRONTIERS_V2.last().unwrap().host,
        HostRetirementStateV2::FinalizationRequestFrozen,
    );
}

#[test]
fn durable_pipeline_freezes_one_request_and_same_bytes_rejoin_only() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("authority");
    let uid = unsafe { libc::geteuid() };
    let pipeline = AuthorityPipelineV2::new(&root, uid).unwrap();
    assert_eq!(pipeline.root(), fs::canonicalize(&root).unwrap());
    let documents = documents();
    let mut seen = Vec::new();

    let ready = pipeline
        .advance(&documents, verifier(&mut seen), no_fault)
        .unwrap();
    assert_eq!(
        ready.frontier.gate,
        AuthorityGateV2::FinalizationRequestFrozen
    );
    assert_eq!(
        seen,
        vec![
            SignatureLaneV2::PublisherReceipt,
            SignatureLaneV2::HarnessAcknowledgement,
            SignatureLaneV2::ProtectedCasBinding,
        ],
    );
    assert_eq!(
        ready.request,
        freeze_finalization_request_v2(&documents).unwrap()
    );
    assert_eq!(
        ready.canonical_request,
        canonical_bytes_v2(&ready.request).unwrap()
    );

    let progress = pipeline.inspect(&documents).unwrap();
    assert!(progress.ready_for_finalizer_delivery);
    assert_eq!(
        progress.request_digest.as_deref(),
        Some(ready.request.request_digest.as_str())
    );

    let mut replay_seen = Vec::new();
    let replay = pipeline
        .advance(&documents, verifier(&mut replay_seen), no_fault)
        .unwrap();
    assert_eq!(replay.canonical_request, ready.canonical_request);

    let mut alternate = documents;
    alternate.retry_state_sha256 = digest(42);
    assert!(pipeline
        .advance(&alternate, verifier(&mut Vec::new()), no_fault)
        .is_err());
}

#[test]
fn every_fault_checkpoint_preserves_the_exact_not_ready_frontier() {
    use FaultCheckpointV2 as C;

    let cases = [
        (
            C::BeforeReceiptDurability,
            AuthorityGateV2::ReceiptSigned,
            false,
        ),
        (
            C::AfterReceiptDurability,
            AuthorityGateV2::ReceiptExternallyDurable,
            false,
        ),
        (
            C::BeforeAcknowledgementDurability,
            AuthorityGateV2::ReceiptExternallyDurable,
            false,
        ),
        (
            C::AfterAcknowledgementDurability,
            AuthorityGateV2::AcknowledgementExternallyDurable,
            false,
        ),
        (
            C::BeforeProtectedCasBinding,
            AuthorityGateV2::AcknowledgementExternallyDurable,
            false,
        ),
        (
            C::AfterProtectedCasBinding,
            AuthorityGateV2::AcknowledgementCasBound,
            false,
        ),
        (
            C::BeforeFinalizationRequestDurability,
            AuthorityGateV2::AcknowledgementCasBound,
            false,
        ),
        (
            C::AfterFinalizationRequestDurability,
            AuthorityGateV2::FinalizationRequestFrozen,
            true,
        ),
    ];

    for (checkpoint, expected_gate, expected_ready) in cases {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("authority");
        let pipeline = AuthorityPipelineV2::new(&root, unsafe { libc::geteuid() }).unwrap();
        let documents = documents();
        let result = pipeline.advance(&documents, verifier(&mut Vec::new()), |observed| {
            if observed == checkpoint {
                bail!("injected {observed:?}")
            }
            Ok(())
        });
        assert!(result.is_err(), "checkpoint {checkpoint:?} must interrupt");
        let progress = pipeline.inspect(&documents).unwrap();
        assert_eq!(progress.frontier.gate, expected_gate, "{checkpoint:?}");
        assert_eq!(
            progress.ready_for_finalizer_delivery, expected_ready,
            "{checkpoint:?}"
        );
    }
}

#[test]
fn frozen_authority_cannot_self_declare_finalizer_acceptance() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("authority");
    let pipeline = AuthorityPipelineV2::new(&root, unsafe { libc::geteuid() }).unwrap();
    let documents = documents();

    pipeline
        .advance(&documents, verifier(&mut Vec::new()), no_fault)
        .unwrap();
    let progress = pipeline.inspect(&documents).unwrap();
    assert_eq!(
        progress.frontier.gate,
        AuthorityGateV2::FinalizationRequestFrozen,
    );
    assert!(progress.ready_for_finalizer_delivery);
    assert!(!root.join("inbox/finalizer-acceptance.v2.json").exists());
}

#[test]
fn alternate_cas_or_r6_pairing_is_rejected_before_any_durability() {
    let mut alternate_cas = documents();
    alternate_cas.protected_cas_binding.acknowledgement_sha256 = digest(55);
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("authority");
    let pipeline = AuthorityPipelineV2::new(&root, unsafe { libc::geteuid() }).unwrap();
    assert!(pipeline
        .advance(&alternate_cas, verifier(&mut Vec::new()), no_fault)
        .is_err());
    assert_eq!(
        pipeline.inspect(&alternate_cas).unwrap().frontier.gate,
        AuthorityGateV2::ReceiptSigned,
    );

    for mutate in [
        |documents: &mut ProspectiveAuthorityDocumentsV2| {
            documents.r6_terminal_pairing.predecessor_sha256 = digest(56)
        },
        |documents: &mut ProspectiveAuthorityDocumentsV2| {
            documents.r6_terminal_pairing.terminal_host_record_sha256 = digest(56)
        },
        |documents: &mut ProspectiveAuthorityDocumentsV2| {
            documents
                .r6_terminal_pairing
                .guest_anchor_acknowledgement_sha256 = digest(56)
        },
        |documents: &mut ProspectiveAuthorityDocumentsV2| {
            documents
                .r6_terminal_pairing
                .guest_consumption_marker_response_sha256 = digest(56)
        },
    ] {
        let mut alternate_r6 = documents();
        mutate(&mut alternate_r6);
        let temp = tempfile::tempdir().unwrap();
        let pipeline =
            AuthorityPipelineV2::new(&temp.path().join("authority"), unsafe { libc::geteuid() })
                .unwrap();
        assert!(pipeline
            .advance(&alternate_r6, verifier(&mut Vec::new()), no_fault)
            .is_err());
    }
}

#[test]
fn no_follow_persistence_cannot_escape_the_constructor_bound_root() {
    assert!(
        AuthorityPipelineV2::new(std::path::Path::new("relative-authority"), unsafe {
            libc::geteuid()
        },)
        .is_err()
    );

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("authority");
    let pipeline = AuthorityPipelineV2::new(&root, unsafe { libc::geteuid() }).unwrap();
    let documents = documents();
    assert!(pipeline
        .advance(&documents, verifier(&mut Vec::new()), |checkpoint| {
            if checkpoint == FaultCheckpointV2::BeforeReceiptDurability {
                bail!("prepare fixed directories")
            }
            Ok(())
        },)
        .is_err());

    let outside = temp.path().join("outside");
    fs::write(&outside, b"outside remains unchanged").unwrap();
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o400)).unwrap();
    symlink(
        &outside,
        root.join("external").join("publisher-receipt.v2.json"),
    )
    .unwrap();

    assert!(pipeline
        .advance(&documents, verifier(&mut Vec::new()), no_fault)
        .is_err());
    assert_eq!(fs::read(&outside).unwrap(), b"outside remains unchanged");
}
