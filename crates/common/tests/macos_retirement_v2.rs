use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::SigningKey as Ed25519SigningKey;
use p256::{
    ecdsa::{Signature as P256Signature, SigningKey as P256SigningKey},
    pkcs8::EncodePublicKey,
    PublicKey as P256PublicKey,
};
use serde::Serialize;
use serde_json::Value;
use substrate_common::macos_retirement_v2::*;

fn digest(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

fn target(ordinal: u16, locator: HostResourceLocatorV2, byte: u8) -> HostTargetIdentityV2 {
    HostTargetIdentityV2 {
        ordinal,
        role: host_target_role_for_locator_v2(&locator),
        locator,
        expected_before_sha256: digest(byte),
    }
}

fn prospective_targets() -> Vec<HostTargetIdentityV2> {
    use FixedFileRoleV2 as F;
    use GenericPasswordRoleV2 as G;
    use HostResourceLocatorV2 as L;
    let locators = vec![
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
    ];
    locators
        .into_iter()
        .enumerate()
        .map(|(index, locator)| {
            target(
                u16::try_from(index + 1).unwrap(),
                locator,
                u8::try_from(index + 1).unwrap(),
            )
        })
        .collect()
}

#[test]
fn corrected_guest_and_host_chronologies_accept_only_exact_successors() {
    for current in GUEST_RETIREMENT_SEQUENCE_V2 {
        for next in GUEST_RETIREMENT_SEQUENCE_V2 {
            let expected = GUEST_RETIREMENT_SEQUENCE_V2
                .windows(2)
                .any(|pair| pair == [current, next]);
            assert_eq!(
                validate_guest_state_transition_v2(current, next).is_ok(),
                expected,
                "guest edge {current:?} -> {next:?}",
            );
        }
    }

    for current in HOST_RETIREMENT_SEQUENCE_V2 {
        for next in HOST_RETIREMENT_SEQUENCE_V2 {
            let expected = HOST_RETIREMENT_SEQUENCE_V2
                .windows(2)
                .any(|pair| pair == [current, next]);
            assert_eq!(
                validate_host_state_transition_v2(current, next).is_ok(),
                expected,
                "host edge {current:?} -> {next:?}",
            );
        }
    }

    assert_eq!(
        GUEST_RETIREMENT_SEQUENCE_V2.last(),
        Some(&GuestRetirementStateV2::ParityHostBound),
    );
    assert_eq!(
        HOST_RETIREMENT_SEQUENCE_V2.last(),
        Some(&HostRetirementStateV2::Complete),
    );
}

#[test]
fn fixed_effect_plan_is_the_exact_closed_dag() {
    use HostTargetRoleV2 as R;

    let targets = prospective_targets();
    let plan = derive_host_effect_plan_v2(TargetSetKindV2::ProspectiveHost, &targets)
        .expect("closed prospective plan");
    validate_host_effect_plan_v2(TargetSetKindV2::ProspectiveHost, &targets, &plan)
        .expect("exact derived plan");

    assert_eq!(
        plan.iter()
            .find(|effect| effect.role == R::Signer)
            .unwrap()
            .predecessor_ordinals,
        vec![8],
    );
    assert_eq!(
        plan.iter()
            .find(|effect| effect.role == R::LaunchdPlist)
            .unwrap()
            .predecessor_ordinals,
        vec![10],
    );
    assert_eq!(plan.last().unwrap().role, R::TerminalLatch);
    assert_eq!(plan.last().unwrap().predecessor_ordinals, vec![19],);
    assert_eq!(
        plan.iter()
            .find(|effect| effect.role == R::CurrentAnchorLock)
            .unwrap()
            .required_until_effect_ordinal,
        Some(12),
    );

    let mut altered_edge = plan.clone();
    altered_edge
        .iter_mut()
        .find(|effect| effect.role == R::Signer)
        .unwrap()
        .predecessor_ordinals
        .remove(0);
    assert!(validate_host_effect_plan_v2(
        TargetSetKindV2::ProspectiveHost,
        &targets,
        &altered_edge,
    )
    .is_err());

    let mut altered_order = plan.clone();
    altered_order.swap(5, 6);
    assert!(validate_host_effect_plan_v2(
        TargetSetKindV2::ProspectiveHost,
        &targets,
        &altered_order,
    )
    .is_err());

    let mut missing_role = targets.clone();
    missing_role.remove(2);
    assert!(derive_host_effect_plan_v2(TargetSetKindV2::ProspectiveHost, &missing_role).is_err());

    let mut duplicate_role = targets.clone();
    duplicate_role[3].locator = duplicate_role[2].locator.clone();
    duplicate_role[3].role = duplicate_role[2].role;
    assert!(derive_host_effect_plan_v2(TargetSetKindV2::ProspectiveHost, &duplicate_role).is_err());
}

#[test]
fn disposable_capability_plan_cannot_admit_prospective_host_roles() {
    use HostTargetRoleV2 as R;

    use DisposableCapabilityControlV2 as C;
    use HostResourceLocatorV2 as L;
    let targets = [
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
    ]
    .into_iter()
    .enumerate()
    .map(|(index, locator)| target((index + 1) as u16, locator, (index + 1) as u8))
    .collect::<Vec<_>>();
    let plan = derive_host_effect_plan_v2(TargetSetKindV2::DisposableCapability, &targets)
        .expect("closed disposable plan");
    assert_eq!(
        plan.iter().map(|entry| entry.role).collect::<Vec<_>>(),
        vec![
            R::CapabilitySignControl,
            R::CapabilityExportControl,
            R::CapabilityAclMutationControl,
            R::CapabilityWrongKeyDeleteControl,
            R::ProtectedWrapper,
            R::Signer,
            R::CurrentAnchorLock,
            R::TerminalLatch,
        ],
    );
    assert_eq!(plan[1].predecessor_ordinals, vec![1]);
    assert_eq!(plan[5].predecessor_ordinals, vec![5]);
    assert_eq!(plan[7].predecessor_ordinals, vec![7]);

    let mut cross_lane = targets;
    cross_lane[0].locator = L::PublisherService;
    cross_lane[0].role = R::PublisherService;
    assert!(
        derive_host_effect_plan_v2(TargetSetKindV2::DisposableCapability, &cross_lane).is_err()
    );
}

#[test]
fn canonical_parser_rejects_duplicates_noncanonical_bytes_and_unknown_fields() {
    let identity = target(1, HostResourceLocatorV2::SigningKey, 1);
    let canonical = canonical_bytes_v2(&identity).expect("canonical target");
    let decoded: HostTargetIdentityV2 =
        parse_canonical_v2(&canonical).expect("strict canonical target");
    assert_eq!(decoded, identity);

    let hash = digest(1);
    let reordered = format!(
        r#"{{"ordinal":1,"expected_before_sha256":"{hash}","locator":"signing_key","role":"signer"}}"#
    );
    assert!(parse_canonical_v2::<HostTargetIdentityV2>(reordered.as_bytes()).is_err());

    let duplicate = format!(
        r#"{{"expected_before_sha256":"{hash}","expected_before_sha256":"{hash}","role":"signer"}}"#
    );
    assert!(parse_canonical_v2::<Value>(duplicate.as_bytes()).is_err());

    let unknown =
        format!(r#"{{"expected_before_sha256":"{hash}","role":"signer","unexpected":true}}"#);
    assert!(parse_canonical_v2::<HostTargetIdentityV2>(unknown.as_bytes()).is_err());

    let floating_point = br#"{"value":1.0}"#;
    assert!(parse_canonical_v2::<Value>(floating_point).is_err());

    let mut trailing_newline = canonical;
    trailing_newline.push(b'\n');
    assert!(parse_canonical_v2::<HostTargetIdentityV2>(&trailing_newline).is_err());
}

#[test]
fn bounded_canonical_parser_preserves_the_default_and_exact_alternate_limit() {
    const OBSERVED_GLOBAL_PACKET_BYTES: usize = 1_175_768;
    const GLOBAL_PACKET_MAX_BYTES: usize = 16 * 1024 * 1024;

    fn canonical_string_document_with_size(size: usize) -> Vec<u8> {
        let empty = canonical_bytes_v2(&Value::String(String::new())).unwrap();
        assert!(size >= empty.len());
        let bytes = canonical_bytes_v2(&Value::String("x".repeat(size - empty.len()))).unwrap();
        assert_eq!(bytes.len(), size);
        bytes
    }

    let observed = canonical_string_document_with_size(OBSERVED_GLOBAL_PACKET_BYTES);
    assert!(parse_canonical_v2::<Value>(&observed).is_err());
    assert_eq!(
        parse_canonical_bounded_v2::<Value>(&observed, GLOBAL_PACKET_MAX_BYTES).unwrap(),
        Value::String("x".repeat(OBSERVED_GLOBAL_PACKET_BYTES - 2))
    );

    let overflow = canonical_string_document_with_size(GLOBAL_PACKET_MAX_BYTES + 1);
    assert!(parse_canonical_bounded_v2::<Value>(&overflow, GLOBAL_PACKET_MAX_BYTES).is_err());

    let mut noncanonical = observed;
    noncanonical.push(b'\n');
    assert!(parse_canonical_bounded_v2::<Value>(&noncanonical, GLOBAL_PACKET_MAX_BYTES).is_err());
}

#[test]
fn canonical_parser_rejects_cross_lane_documents() {
    let parity = HostParityProofV2 {
        schema_owner: "parity-owner".to_string(),
        schema_version: 2,
        signature_domain: "parity-domain".to_string(),
        evidence_id: "evidence".to_string(),
        scope_id: "019ffe3b-2f34-78d2-afca-8ecfb7b0280a".to_string(),
        request_digest: digest(1),
        effects_response_sha256: digest(2),
        journal_head_sha256: digest(3),
        baseline_sha256: digest(4),
        after_observation_sha256: digest(5),
        exact_parity: true,
        signature: MacR3SignatureV2::unsigned_ed25519("key".to_string()),
    };
    let bytes = canonical_bytes_v2(&parity).expect("canonical parity lane");
    assert!(parse_canonical_v2::<TerminalAcknowledgementV2>(&bytes).is_err());
}

#[test]
fn process_binding_is_to_the_exact_validated_executable_identity() {
    let executable = ExecutableIdentityV2 {
        source_commit: "1".repeat(40),
        source_tree: "2".repeat(40),
        source_hashes_sha256: digest(3),
        build_inputs_sha256: digest(4),
        executable_sha256: digest(5),
        executable_size: 4096,
        intended_path: "/fixed/helper".to_string(),
        physical_identity_sha256: digest(6),
        signing_identifier: "com.atomize.fixed-helper".to_string(),
        designated_requirement: "identifier com.atomize.fixed-helper".to_string(),
        cdhash: "a".repeat(40),
    };
    validate_executable_identity_v2(&executable, "/fixed/helper", "com.atomize.fixed-helper")
        .expect("closed executable identity");

    let process = ProcessIdentityV2 {
        effective_uid: 0,
        effective_gid: 20,
        canonical_account: "root".to_string(),
        pidversion_required: true,
        process_start_identity_sha256: digest(7),
        executable_identity_sha256: document_sha256_v2(&executable).unwrap(),
    };
    validate_process_executable_binding_v2(
        &process,
        &executable,
        "/fixed/helper",
        "com.atomize.fixed-helper",
    )
    .expect("process bound to exact executable");

    let mut wrong_gid = process.clone();
    wrong_gid.effective_gid = 80;
    assert!(validate_process_executable_binding_v2(
        &wrong_gid,
        &executable,
        "/fixed/helper",
        "com.atomize.fixed-helper",
    )
    .is_err());

    let mut mismatched_process = process;
    mismatched_process.executable_identity_sha256 = digest(8);
    assert!(validate_process_executable_binding_v2(
        &mismatched_process,
        &executable,
        "/fixed/helper",
        "com.atomize.fixed-helper",
    )
    .is_err());

    let mut uppercase_cdhash = executable;
    uppercase_cdhash.cdhash = "A".repeat(40);
    assert!(validate_executable_identity_v2(
        &uppercase_cdhash,
        "/fixed/helper",
        "com.atomize.fixed-helper",
    )
    .is_err());
}

fn ed25519_public_key(key: &Ed25519SigningKey) -> String {
    URL_SAFE_NO_PAD.encode(key.verifying_key().to_bytes())
}

fn host_spki_der(key: &P256SigningKey) -> String {
    let public_key =
        P256PublicKey::from_sec1_bytes(key.verifying_key().to_encoded_point(false).as_bytes())
            .unwrap();
    URL_SAFE_NO_PAD.encode(public_key.to_public_key_der().unwrap().as_ref())
}

fn sign_ed25519<T: Serialize>(
    value: &T,
    key: &Ed25519SigningKey,
    domain: &str,
    owner: &str,
) -> MacR3SignatureV2 {
    let payload = signature_payload_v2(domain, owner, value).unwrap();
    let signature = ed25519_dalek::Signer::sign(key, &payload);
    MacR3SignatureV2 {
        algorithm: "ed25519-v1".to_string(),
        public_key: ed25519_public_key(key),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    }
}

fn sign_p256<T: Serialize>(
    value: &T,
    key: &P256SigningKey,
    domain: &str,
    owner: &str,
) -> MacR3SignatureV2 {
    let payload = signature_payload_v2(domain, owner, value).unwrap();
    let signature: P256Signature = p256::ecdsa::signature::Signer::sign(key, &payload);
    let signature = signature.normalize_s().unwrap_or(signature);
    MacR3SignatureV2 {
        algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
        public_key: host_spki_der(key),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    }
}

fn encoded<T: Serialize>(value: &T) -> String {
    URL_SAFE_NO_PAD.encode(canonical_bytes_v2(value).unwrap())
}

struct GuestJournalFixtureContext<'a> {
    initial_predecessor: &'a str,
    evidence_id: &'a str,
    request: Option<&'a str>,
    acceptance_sha256: Option<&'a str>,
}

struct GuestJournalFixtureEntry {
    event: GuestJournalEventKindV2,
    state: GuestRetirementStateV2,
    target_identity_sha256: Option<String>,
    observation_sha256: Option<String>,
}

fn push_guest_journal(
    journal: &mut Vec<GuestJournalGenerationV2>,
    context: &GuestJournalFixtureContext<'_>,
    entry: GuestJournalFixtureEntry,
) {
    let generation = u64::try_from(journal.len() + 1).unwrap();
    let predecessor = journal
        .last()
        .map(|record| document_sha256_v2(record).unwrap())
        .unwrap_or_else(|| context.initial_predecessor.to_string());
    let effect_ordinal = matches!(
        entry.event,
        GuestJournalEventKindV2::EffectPrepared
            | GuestJournalEventKindV2::EffectInvoked
            | GuestJournalEventKindV2::EffectObserved
    )
    .then_some(1);
    let effect_invocation_attempt = match entry.event {
        GuestJournalEventKindV2::EffectPrepared => Some(0),
        GuestJournalEventKindV2::EffectInvoked => Some(
            journal
                .last()
                .filter(|record| record.event == GuestJournalEventKindV2::EffectInvoked)
                .and_then(|record| record.effect_invocation_attempt)
                .and_then(|attempt| attempt.checked_add(1))
                .unwrap_or(1),
        ),
        GuestJournalEventKindV2::EffectObserved => journal
            .last()
            .and_then(|record| record.effect_invocation_attempt),
        _ => None,
    };
    journal.push(GuestJournalGenerationV2 {
        schema_owner: MAC_R3_GUEST_JOURNAL_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        evidence_id: context.evidence_id.to_string(),
        scope_id: "019ffe3b-2f34-78d2-afca-8ecfb7b0280a".to_string(),
        generation,
        predecessor_head_sha256: predecessor,
        handoff_request_digest: context.request.map(ToOwned::to_owned),
        host_acceptance_sha256: context.acceptance_sha256.map(ToOwned::to_owned),
        event: entry.event,
        guest_state: entry.state,
        effect_ordinal,
        target_identity_sha256: entry.target_identity_sha256,
        effect_invocation_attempt,
        observation_sha256: entry.observation_sha256,
    });
}

fn guest_terminal_fixture_with_retry(
    retry_invocation: bool,
) -> (GuestTerminalBundleV2, String, String, String) {
    let guest_key = Ed25519SigningKey::from_bytes(&[0x11; 32]);
    let harness_key = Ed25519SigningKey::from_bytes(&[0x22; 32]);
    let host_key = P256SigningKey::from_bytes((&[0x33; 32]).into()).unwrap();
    let guest_public_key = ed25519_public_key(&guest_key);
    let harness_public_key = ed25519_public_key(&harness_key);
    let host_public_key = host_spki_der(&host_key);
    let scope = "019ffe3b-2f34-78d2-afca-8ecfb7b0280a";
    let evidence = "r3-guest-prospective-fixture";
    let target = GuestTargetIdentityV2 {
        ordinal: 1,
        role: GuestTargetRoleV2::LimaInstance,
        locator: GuestResourceLocatorV2::LimaInstance,
        expected_before_sha256: digest(1),
    };
    let target_ledger = vec![target.clone()];
    let effect_plan = derive_guest_effect_plan_v2(&target_ledger).unwrap();
    let initial_guest_cas_head = digest(6);
    let pre_handoff_context = GuestJournalFixtureContext {
        initial_predecessor: &initial_guest_cas_head,
        evidence_id: evidence,
        request: None,
        acceptance_sha256: None,
    };
    let mut journal = Vec::new();
    push_guest_journal(
        &mut journal,
        &pre_handoff_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::Prepared,
            state: GuestRetirementStateV2::Prepared,
            target_identity_sha256: None,
            observation_sha256: Some(digest(2)),
        },
    );
    push_guest_journal(
        &mut journal,
        &pre_handoff_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::QuiescePrepared,
            state: GuestRetirementStateV2::QuiescePrepared,
            target_identity_sha256: None,
            observation_sha256: Some(digest(17)),
        },
    );
    push_guest_journal(
        &mut journal,
        &pre_handoff_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::Quiesced,
            state: GuestRetirementStateV2::Quiesced,
            target_identity_sha256: None,
            observation_sha256: Some(digest(4)),
        },
    );
    let mut receipt = GuestPreRemovalReceiptV2 {
        schema_owner: MAC_R3_GUEST_RECEIPT_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_RECEIPT_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        issued_at_unix_ns: 100,
        expires_at_unix_ns: 1_000,
        guest_state: GuestRetirementStateV2::PreRemovalReceiptSigned,
        baseline_sha256: digest(2),
        before_observation_sha256: digest(3),
        quiesced_observation_sha256: digest(4),
        component_inventory_sha256: digest(5),
        target_ledger_sha256: document_sha256_v2(&target_ledger).unwrap(),
        effect_plan_sha256: document_sha256_v2(&effect_plan).unwrap(),
        target_ledger,
        protected_cas_generation: 7,
        protected_cas_head_sha256: initial_guest_cas_head.clone(),
        pre_removal_journal_head_sha256: document_sha256_v2(&journal[2]).unwrap(),
        r6_predecessor_consumed_sha256: digest(7),
        r6_terminal_host_record_sha256: digest(8),
        guest_anchor_acknowledgement_sha256: digest(9),
        guest_consumption_marker_acknowledgement_sha256: digest(10),
        retry_state_sha256: digest(11),
        guest_signer_public_key: guest_public_key.clone(),
        harness_public_key: harness_public_key.clone(),
        signature: MacR3SignatureV2::unsigned_ed25519(guest_public_key.clone()),
    };
    receipt.signature = sign_ed25519(
        &receipt,
        &guest_key,
        &receipt.signature_domain,
        &receipt.schema_owner,
    );
    push_guest_journal(
        &mut journal,
        &pre_handoff_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::PreRemovalReceiptSigned,
            state: GuestRetirementStateV2::PreRemovalReceiptSigned,
            target_identity_sha256: None,
            observation_sha256: Some(document_sha256_v2(&receipt).unwrap()),
        },
    );
    let mut acknowledgement = GuestDurabilityAcknowledgementV2 {
        schema_owner: MAC_R3_GUEST_ACK_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_ACK_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        receipt_sha256: document_sha256_v2(&receipt).unwrap(),
        external_store_identity_sha256: digest(12),
        durable_observation_sha256: digest(13),
        acknowledged_at_unix_ns: 200,
        signature: MacR3SignatureV2::unsigned_ed25519(harness_public_key.clone()),
    };
    acknowledgement.signature = sign_ed25519(
        &acknowledgement,
        &harness_key,
        &acknowledgement.signature_domain,
        &acknowledgement.schema_owner,
    );
    push_guest_journal(
        &mut journal,
        &pre_handoff_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::ReceiptExternallyDurable,
            state: GuestRetirementStateV2::ReceiptExternallyDurable,
            target_identity_sha256: None,
            observation_sha256: Some(acknowledgement.durable_observation_sha256.clone()),
        },
    );
    push_guest_journal(
        &mut journal,
        &pre_handoff_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::AcknowledgementExternallyDurable,
            state: GuestRetirementStateV2::AcknowledgementExternallyDurable,
            target_identity_sha256: None,
            observation_sha256: Some(document_sha256_v2(&acknowledgement).unwrap()),
        },
    );
    let mut retry = GuestRetryStateV2 {
        schema_owner: MAC_R3_GUEST_RETRY_STATE_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_RETRY_STATE_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        guest_state: GuestRetirementStateV2::AcknowledgementCasBound,
        expected_protected_cas_generation: 8,
        protected_cas_predecessor_head_sha256: receipt.protected_cas_head_sha256.clone(),
        receipt_sha256: document_sha256_v2(&receipt).unwrap(),
        acknowledgement_sha256: document_sha256_v2(&acknowledgement).unwrap(),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
        next_effect_ordinal: 1,
        predecessor_journal_head_sha256: document_sha256_v2(&journal[5]).unwrap(),
        next_journal_generation: 7,
        signature: MacR3SignatureV2::unsigned_ed25519(guest_public_key.clone()),
    };
    retry.signature = sign_ed25519(
        &retry,
        &guest_key,
        &retry.signature_domain,
        &retry.schema_owner,
    );
    let intent = derive_guest_handoff_intent_v2(&receipt, &acknowledgement, &retry).unwrap();
    let mut protected_cas = GuestProtectedCasBindingV2 {
        schema_owner: MAC_R3_GUEST_PROTECTED_CAS_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_PROTECTED_CAS_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        generation: retry.expected_protected_cas_generation,
        predecessor_head_sha256: retry.protected_cas_predecessor_head_sha256.clone(),
        receipt_sha256: intent.receipt_sha256.clone(),
        acknowledgement_sha256: intent.acknowledgement_sha256.clone(),
        retry_state_sha256: intent.retry_state_sha256.clone(),
        handoff_request_digest: document_sha256_v2(&intent).unwrap(),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
        predecessor_journal_head_sha256: retry.predecessor_journal_head_sha256.clone(),
        journal_generation: retry.next_journal_generation,
        signature: MacR3SignatureV2::unsigned_ed25519(guest_public_key.clone()),
    };
    protected_cas.signature = sign_ed25519(
        &protected_cas,
        &guest_key,
        &protected_cas.signature_domain,
        &protected_cas.schema_owner,
    );
    let handoff_context = GuestJournalFixtureContext {
        initial_predecessor: &initial_guest_cas_head,
        evidence_id: evidence,
        request: Some(&protected_cas.handoff_request_digest),
        acceptance_sha256: None,
    };
    push_guest_journal(
        &mut journal,
        &handoff_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::AcknowledgementCasBound,
            state: GuestRetirementStateV2::AcknowledgementCasBound,
            target_identity_sha256: None,
            observation_sha256: Some(document_sha256_v2(&protected_cas).unwrap()),
        },
    );
    let handoff = GuestHandoffCapsuleV2 {
        schema_owner: MAC_R3_GUEST_HANDOFF_CAPSULE_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        handoff_request_digest: protected_cas.handoff_request_digest.clone(),
        guest_receipt: encoded(&receipt),
        guest_receipt_sha256: document_sha256_v2(&receipt).unwrap(),
        guest_acknowledgement: encoded(&acknowledgement),
        guest_acknowledgement_sha256: document_sha256_v2(&acknowledgement).unwrap(),
        guest_retry_state: encoded(&retry),
        guest_retry_state_sha256: document_sha256_v2(&retry).unwrap(),
        guest_protected_cas_binding: encoded(&protected_cas),
        guest_protected_cas_binding_sha256: document_sha256_v2(&protected_cas).unwrap(),
        guest_journal: journal.iter().map(encoded).collect(),
        guest_journal_head_sha256: document_sha256_v2(&journal[6]).unwrap(),
    };
    let mut acceptance = GuestHostAcceptanceV2 {
        schema_owner: MAC_R3_GUEST_HOST_ACCEPTANCE_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_HOST_ACCEPTANCE_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        guest_state: GuestRetirementStateV2::HandoffHostBound,
        handoff_request_digest: handoff.handoff_request_digest.clone(),
        handoff_capsule_sha256: document_sha256_v2(&handoff).unwrap(),
        guest_protected_cas_head_sha256: document_sha256_v2(&protected_cas).unwrap(),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
        retry_state_sha256: document_sha256_v2(&retry).unwrap(),
        predecessor_journal_head_sha256: handoff.guest_journal_head_sha256.clone(),
        accepted_journal_generation: 8,
        accepted_at_unix_ns: 300,
        signature: MacR3SignatureV2::unsigned_p256(host_public_key.clone()),
    };
    acceptance.signature = sign_p256(
        &acceptance,
        &host_key,
        &acceptance.signature_domain,
        &acceptance.schema_owner,
    );
    let acceptance_sha256 = document_sha256_v2(&acceptance).unwrap();
    let target_sha256 = document_sha256_v2(&target).unwrap();
    let accepted_context = GuestJournalFixtureContext {
        initial_predecessor: &initial_guest_cas_head,
        evidence_id: evidence,
        request: Some(&handoff.handoff_request_digest),
        acceptance_sha256: Some(&acceptance_sha256),
    };
    push_guest_journal(
        &mut journal,
        &accepted_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::HostAccepted,
            state: GuestRetirementStateV2::HandoffHostBound,
            target_identity_sha256: None,
            observation_sha256: Some(acceptance_sha256.clone()),
        },
    );
    push_guest_journal(
        &mut journal,
        &accepted_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::EffectPrepared,
            state: GuestRetirementStateV2::Removing,
            target_identity_sha256: Some(target_sha256.clone()),
            observation_sha256: None,
        },
    );
    push_guest_journal(
        &mut journal,
        &accepted_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::EffectInvoked,
            state: GuestRetirementStateV2::Removing,
            target_identity_sha256: Some(target_sha256.clone()),
            observation_sha256: None,
        },
    );
    if retry_invocation {
        push_guest_journal(
            &mut journal,
            &accepted_context,
            GuestJournalFixtureEntry {
                event: GuestJournalEventKindV2::EffectInvoked,
                state: GuestRetirementStateV2::Removing,
                target_identity_sha256: Some(target_sha256.clone()),
                observation_sha256: None,
            },
        );
    }
    let removal_observation = digest(15);
    push_guest_journal(
        &mut journal,
        &accepted_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::EffectObserved,
            state: GuestRetirementStateV2::Removing,
            target_identity_sha256: Some(target_sha256),
            observation_sha256: Some(removal_observation.clone()),
        },
    );
    push_guest_journal(
        &mut journal,
        &accepted_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::GuestRemoved,
            state: GuestRetirementStateV2::Removed,
            target_identity_sha256: None,
            observation_sha256: Some(removal_observation.clone()),
        },
    );
    let mut response = GuestEffectsResponseV2 {
        schema_owner: MAC_R3_GUEST_EFFECTS_RESPONSE_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_EFFECTS_RESPONSE_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        guest_state: GuestRetirementStateV2::Removed,
        handoff_request_digest: handoff.handoff_request_digest.clone(),
        host_acceptance_sha256: acceptance_sha256.clone(),
        target_ledger_sha256: receipt.target_ledger_sha256.clone(),
        effect_plan_sha256: receipt.effect_plan_sha256.clone(),
        effects_observation_sha256: removal_observation,
        journal_head_sha256: document_sha256_v2(journal.last().unwrap()).unwrap(),
        signature: MacR3SignatureV2::unsigned_p256(host_public_key.clone()),
    };
    response.signature = sign_p256(
        &response,
        &host_key,
        &response.signature_domain,
        &response.schema_owner,
    );
    let mut parity = GuestParityProofV2 {
        schema_owner: MAC_R3_GUEST_PARITY_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_PARITY_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        handoff_request_digest: handoff.handoff_request_digest.clone(),
        effects_response_sha256: document_sha256_v2(&response).unwrap(),
        journal_head_sha256: response.journal_head_sha256.clone(),
        baseline_sha256: receipt.baseline_sha256.clone(),
        after_observation_sha256: digest(16),
        exact_parity: true,
        signature: MacR3SignatureV2::unsigned_ed25519(harness_public_key.clone()),
    };
    parity.signature = sign_ed25519(
        &parity,
        &harness_key,
        &parity.signature_domain,
        &parity.schema_owner,
    );
    push_guest_journal(
        &mut journal,
        &accepted_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::ParityExternallyDurable,
            state: GuestRetirementStateV2::ParityExternallyDurable,
            target_identity_sha256: None,
            observation_sha256: Some(document_sha256_v2(&parity).unwrap()),
        },
    );
    let mut parity_binding = GuestParityHostBindingV2 {
        schema_owner: MAC_R3_GUEST_PARITY_HOST_BINDING_OWNER_V2.to_string(),
        schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
        signature_domain: MAC_R3_GUEST_PARITY_HOST_BINDING_SIGNATURE_DOMAIN_V2.to_string(),
        evidence_id: evidence.to_string(),
        scope_id: scope.to_string(),
        guest_state: GuestRetirementStateV2::ParityHostBound,
        handoff_request_digest: handoff.handoff_request_digest.clone(),
        effects_response_sha256: document_sha256_v2(&response).unwrap(),
        parity_proof_sha256: document_sha256_v2(&parity).unwrap(),
        host_acceptance_sha256: acceptance_sha256.clone(),
        journal_head_sha256: document_sha256_v2(journal.last().unwrap()).unwrap(),
        generation: 2,
        predecessor_head_sha256: document_sha256_v2(&acceptance).unwrap(),
        signature: MacR3SignatureV2::unsigned_p256(host_public_key.clone()),
    };
    parity_binding.signature = sign_p256(
        &parity_binding,
        &host_key,
        &parity_binding.signature_domain,
        &parity_binding.schema_owner,
    );
    push_guest_journal(
        &mut journal,
        &accepted_context,
        GuestJournalFixtureEntry {
            event: GuestJournalEventKindV2::ParityHostBound,
            state: GuestRetirementStateV2::ParityHostBound,
            target_identity_sha256: None,
            observation_sha256: Some(document_sha256_v2(&parity_binding).unwrap()),
        },
    );
    let terminal_journal_head_sha256 = document_sha256_v2(journal.last().unwrap()).unwrap();
    let successor_capsule =
        derive_guest_to_host_successor_capsule_v2(GuestToHostSuccessorBindingV2 {
            handoff: &handoff,
            receipt: &receipt,
            retry: &retry,
            protected_cas: &protected_cas,
            acceptance: &acceptance,
            effects_response: &response,
            parity_proof: &parity,
            parity_host_binding: &parity_binding,
            terminal_journal_head_sha256: &terminal_journal_head_sha256,
        })
        .unwrap();
    (
        GuestTerminalBundleV2 {
            schema_owner: MAC_R3_GUEST_TERMINAL_BUNDLE_OWNER_V2.to_string(),
            schema_version: MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
            evidence_id: evidence.to_string(),
            scope_id: scope.to_string(),
            guest_handoff_capsule: encoded(&handoff),
            guest_host_acceptance: encoded(&acceptance),
            guest_effects_response: encoded(&response),
            guest_parity_proof: encoded(&parity),
            guest_parity_host_binding: encoded(&parity_binding),
            guest_journal: journal.iter().map(encoded).collect(),
            successor_capsule,
        },
        guest_public_key,
        harness_public_key,
        host_public_key,
    )
}

fn guest_terminal_fixture() -> (GuestTerminalBundleV2, String, String, String) {
    guest_terminal_fixture_with_retry(false)
}

#[test]
fn complete_guest_terminal_bundle_verifies_every_signed_artifact_and_journal_join() {
    let (bundle, guest_key, harness_key, host_key) = guest_terminal_fixture();
    let accepted =
        validate_guest_terminal_bundle_v2(&bundle, &guest_key, &harness_key, &host_key).unwrap();
    assert_eq!(accepted, bundle.successor_capsule);
    validate_guest_to_host_successor_capsule_v2(&accepted).unwrap();

    for lane in 0..5 {
        let mut altered = bundle.clone();
        match lane {
            0 => altered.guest_host_acceptance = URL_SAFE_NO_PAD.encode(b"{}"),
            1 => altered.guest_effects_response = URL_SAFE_NO_PAD.encode(b"{}"),
            2 => altered.guest_parity_proof = URL_SAFE_NO_PAD.encode(b"{}"),
            3 => altered.guest_parity_host_binding = URL_SAFE_NO_PAD.encode(b"{}"),
            4 => altered.guest_journal[3] = URL_SAFE_NO_PAD.encode(b"{}"),
            _ => unreachable!(),
        }
        assert!(
            validate_guest_terminal_bundle_v2(&altered, &guest_key, &harness_key, &host_key)
                .is_err()
        );
    }

    let mut handoff: GuestHandoffCapsuleV2 = parse_canonical_v2(
        &URL_SAFE_NO_PAD
            .decode(&bundle.guest_handoff_capsule)
            .unwrap(),
    )
    .unwrap();
    for lane in 0..4 {
        let mut altered_handoff = handoff.clone();
        match lane {
            0 => altered_handoff.guest_receipt = URL_SAFE_NO_PAD.encode(b"{}"),
            1 => altered_handoff.guest_acknowledgement = URL_SAFE_NO_PAD.encode(b"{}"),
            2 => altered_handoff.guest_retry_state = URL_SAFE_NO_PAD.encode(b"{}"),
            3 => altered_handoff.guest_protected_cas_binding = URL_SAFE_NO_PAD.encode(b"{}"),
            _ => unreachable!(),
        }
        let mut altered = bundle.clone();
        altered.guest_handoff_capsule = encoded(&altered_handoff);
        assert!(
            validate_guest_terminal_bundle_v2(&altered, &guest_key, &harness_key, &host_key)
                .is_err()
        );
    }
    handoff.handoff_request_digest = digest(99);
    let mut alternate_digest = bundle;
    alternate_digest.guest_handoff_capsule = encoded(&handoff);
    assert!(validate_guest_terminal_bundle_v2(
        &alternate_digest,
        &guest_key,
        &harness_key,
        &host_key
    )
    .is_err());
}

#[test]
fn complete_guest_terminal_bundle_rejects_an_external_process_reinvocation_lane() {
    let (bundle, guest_key, harness_key, host_key) = guest_terminal_fixture_with_retry(true);
    assert_eq!(
        bundle.guest_journal.len(),
        MAC_R3_GUEST_MAX_TERMINAL_JOURNAL_GENERATIONS_V2 + 1
    );
    let attempts = bundle
        .guest_journal
        .iter()
        .map(|encoded| {
            parse_canonical_v2::<GuestJournalGenerationV2>(
                &URL_SAFE_NO_PAD.decode(encoded).unwrap(),
            )
            .unwrap()
        })
        .filter(|record| record.event == GuestJournalEventKindV2::EffectInvoked)
        .map(|record| record.effect_invocation_attempt.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(attempts, vec![1, 2]);

    assert!(
        validate_guest_terminal_bundle_v2(&bundle, &guest_key, &harness_key, &host_key).is_err()
    );
}

#[test]
fn guest_removal_gate_and_crash_recovery_fail_closed_exhaustively() {
    for receipt in [false, true] {
        for acknowledgement in [false, true] {
            for cas in [false, true] {
                for host in [false, true] {
                    let admitted = guest_removal_is_admitted_v2(
                        GuestRetirementStateV2::HandoffHostBound,
                        receipt,
                        acknowledgement,
                        cas,
                        host,
                    );
                    assert_eq!(admitted, receipt && acknowledgement && cas && host);
                }
            }
        }
    }
    assert!(!guest_removal_is_admitted_v2(
        GuestRetirementStateV2::AcknowledgementCasBound,
        true,
        true,
        true,
        true,
    ));
    assert_eq!(
        guest_effect_recovery_decision_v2(
            GuestEffectRecoveryPhaseV2::NotPrepared,
            GuestEffectObservationV2::ExactBefore,
        ),
        GuestEffectRecoveryDecisionV2::PersistPrepared
    );
    assert_eq!(
        guest_effect_recovery_decision_v2(
            GuestEffectRecoveryPhaseV2::Prepared,
            GuestEffectObservationV2::ExactBefore,
        ),
        GuestEffectRecoveryDecisionV2::Invoke
    );
    assert_eq!(
        guest_effect_recovery_decision_v2(
            GuestEffectRecoveryPhaseV2::Invoked,
            GuestEffectObservationV2::ExactFinal,
        ),
        GuestEffectRecoveryDecisionV2::RecordObservedWithoutInvocation
    );
    for phase in [
        GuestEffectRecoveryPhaseV2::NotPrepared,
        GuestEffectRecoveryPhaseV2::Prepared,
        GuestEffectRecoveryPhaseV2::Invoked,
    ] {
        assert_eq!(
            guest_effect_recovery_decision_v2(phase, GuestEffectObservationV2::Ambiguous),
            GuestEffectRecoveryDecisionV2::Preserve
        );
    }
    assert_eq!(
        guest_effect_recovery_decision_v2(
            GuestEffectRecoveryPhaseV2::Prepared,
            GuestEffectObservationV2::ExactFinal,
        ),
        GuestEffectRecoveryDecisionV2::Preserve
    );
}

#[test]
fn guest_after_spawn_before_wait_crash_preserves_on_target_presence() {
    assert_eq!(MAC_R3_GUEST_MAX_EFFECT_INVOCATION_ATTEMPTS_V2, 1);
    assert_eq!(
        guest_effect_recovery_decision_v2(
            GuestEffectRecoveryPhaseV2::Invoked,
            GuestEffectObservationV2::ExactBefore,
        ),
        GuestEffectRecoveryDecisionV2::Preserve
    );
    assert_eq!(
        guest_effect_recovery_decision_v2(
            GuestEffectRecoveryPhaseV2::Invoked,
            GuestEffectObservationV2::Ambiguous,
        ),
        GuestEffectRecoveryDecisionV2::Preserve
    );
    assert_eq!(
        guest_effect_recovery_decision_v2(
            GuestEffectRecoveryPhaseV2::Invoked,
            GuestEffectObservationV2::ExactFinal,
        ),
        GuestEffectRecoveryDecisionV2::RecordObservedWithoutInvocation
    );
}

#[test]
fn guest_effect_reinvocation_generation_is_canonically_rejected() {
    let (bundle, _, _, _) = guest_terminal_fixture();
    let original = bundle
        .guest_journal
        .iter()
        .map(|encoded| {
            parse_canonical_v2::<GuestJournalGenerationV2>(
                &URL_SAFE_NO_PAD.decode(encoded).unwrap(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        original.len(),
        MAC_R3_GUEST_MIN_TERMINAL_JOURNAL_GENERATIONS_V2
    );
    assert_eq!(original[9].event, GuestJournalEventKindV2::EffectInvoked);
    assert_eq!(original[9].effect_invocation_attempt, Some(1));
    assert_eq!(
        original.len(),
        MAC_R3_GUEST_MAX_TERMINAL_JOURNAL_GENERATIONS_V2
    );

    let mut retry = original[9].clone();
    retry.generation = 11;
    retry.predecessor_head_sha256 = document_sha256_v2(&original[9]).unwrap();
    retry.effect_invocation_attempt = Some(2);
    assert!(validate_guest_journal_transition_v2(
        Some((&original[9], &retry.predecessor_head_sha256)),
        &retry,
    )
    .is_err());
    assert!(validate_guest_journal_generation_v2(&retry).is_err());
}

#[test]
fn guest_target_and_decoder_surface_is_closed_and_cross_lane_rejected() {
    let target = GuestTargetIdentityV2 {
        ordinal: 1,
        role: GuestTargetRoleV2::LimaInstance,
        locator: GuestResourceLocatorV2::LimaInstance,
        expected_before_sha256: digest(1),
    };
    let plan = derive_guest_effect_plan_v2(std::slice::from_ref(&target)).unwrap();
    validate_guest_effect_plan_v2(std::slice::from_ref(&target), &plan).unwrap();
    assert!(derive_guest_effect_plan_v2(&[]).is_err());
    let mut wrong_ordinal = target;
    wrong_ordinal.ordinal = 2;
    assert!(derive_guest_effect_plan_v2(&[wrong_ordinal]).is_err());

    let (bundle, _, _, _) = guest_terminal_fixture();
    let parity_bytes = URL_SAFE_NO_PAD.decode(bundle.guest_parity_proof).unwrap();
    assert!(parse_canonical_v2::<HostParityProofV2>(&parity_bytes).is_err());
    assert!(parse_canonical_v2::<TerminalAcknowledgementV2>(&parity_bytes).is_err());

    let canonical = canonical_bytes_v2(&bundle.successor_capsule).unwrap();
    let mut value: Value = serde_json::from_slice(&canonical).unwrap();
    value.as_object_mut().unwrap().insert(
        "target_path".to_string(),
        Value::String("/tmp/widen".to_string()),
    );
    assert!(parse_canonical_v2::<GuestToHostSuccessorCapsuleV2>(
        &serde_json::to_vec(&value).unwrap()
    )
    .is_err());
}
