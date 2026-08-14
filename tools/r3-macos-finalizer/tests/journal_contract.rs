use std::fs;
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, parse_canonical_v2, sha256_hex_v2, HostRetirementStateV2, HostTargetRoleV2,
};
use substrate_r3_macos_finalizer::contract::{
    JournalEvent, JournalEventKind, JOURNAL_OWNER, JOURNAL_VERSION,
};
use substrate_r3_macos_finalizer::journal::{
    JournalClaimIdentity, JournalGeneration, LockedJournal,
};

const SCOPE: &str = "019fffe0-0000-7000-8000-000000000001";

fn digest(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

fn claim() -> JournalClaimIdentity {
    JournalClaimIdentity {
        request_digest: digest(1),
        authority_bytes_sha256: digest(2),
        successor_capsule_sha256: digest(3),
        accepted_peer_attestation_sha256: digest(4),
        accepted_peer_identity_sha256: digest(5),
    }
}

fn event(kind: JournalEventKind) -> JournalEvent {
    let is_effect = matches!(
        kind,
        JournalEventKind::EffectPrepared
            | JournalEventKind::EffectInvoked
            | JournalEventKind::EffectObserved
    );
    JournalEvent {
        kind,
        host_state: if kind == JournalEventKind::FinalizerAccepted {
            HostRetirementStateV2::FinalizerAccepted
        } else {
            HostRetirementStateV2::Removing
        },
        effect_ordinal: is_effect.then_some(1),
        effect_role: is_effect.then_some(HostTargetRoleV2::ProtectedWrapper),
        effect_identity_sha256: is_effect.then(|| digest(8)),
        effect_invocation_attempt: match kind {
            JournalEventKind::EffectPrepared => Some(0),
            JournalEventKind::EffectInvoked | JournalEventKind::EffectObserved => Some(1),
            _ => None,
        },
        observation_sha256: (kind == JournalEventKind::EffectObserved).then(|| digest(9)),
        response_sha256: None,
        preserving_classification: None,
    }
}

#[test]
fn generations_are_canonical_hash_chained_and_generation_cas_rejects_a_stale_head() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();

    let first = journal
        .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
        .unwrap();
    let second = journal
        .append(
            Some(&first),
            &claim(),
            event(JournalEventKind::EffectPrepared),
        )
        .unwrap();

    assert_eq!(first.generation, 1);
    assert_eq!(second.generation, 2);
    assert_eq!(second.record.predecessor_head_sha256, first.sha256);
    let generation_path = root
        .join(SCOPE)
        .join("generation-00000000000000000002.json");
    let bytes = fs::read(generation_path).unwrap();
    assert_eq!(sha256_hex_v2(&bytes), second.sha256);
    assert_eq!(
        canonical_bytes_v2(&parse_canonical_v2::<JournalGeneration>(&bytes).unwrap()).unwrap(),
        bytes
    );

    let error = journal
        .append(
            Some(&first),
            &claim(),
            event(JournalEventKind::EffectInvoked),
        )
        .unwrap_err();
    assert!(error.to_string().contains("generation CAS"));
    assert_eq!(journal.head().unwrap(), Some(second));
}

#[test]
fn one_exact_unreferenced_crash_successor_is_reconciled() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();
    let first = journal
        .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
        .unwrap();

    let successor = JournalGeneration {
        schema_owner: JOURNAL_OWNER.to_string(),
        schema_version: JOURNAL_VERSION,
        scope_id: SCOPE.to_string(),
        generation: 2,
        predecessor_head_sha256: first.sha256,
        request_digest: digest(1),
        authority_bytes_sha256: digest(2),
        successor_capsule_sha256: digest(3),
        accepted_peer_attestation_sha256: digest(4),
        accepted_peer_identity_sha256: digest(5),
        event: event(JournalEventKind::EffectPrepared),
    };
    let successor_bytes = canonical_bytes_v2(&successor).unwrap();
    let successor_path = root
        .join(SCOPE)
        .join("generation-00000000000000000002.json");
    fs::write(&successor_path, &successor_bytes).unwrap();
    fs::set_permissions(&successor_path, fs::Permissions::from_mode(0o400)).unwrap();

    let recovered = journal.head().unwrap().unwrap();
    assert_eq!(recovered.generation, 2);
    assert_eq!(recovered.record, successor);
    assert_eq!(recovered.sha256, sha256_hex_v2(&successor_bytes));
}

#[test]
fn alternate_unreferenced_crash_successor_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();
    let first = journal
        .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
        .unwrap();

    let alternate = JournalGeneration {
        schema_owner: JOURNAL_OWNER.to_string(),
        schema_version: JOURNAL_VERSION,
        scope_id: SCOPE.to_string(),
        generation: 2,
        predecessor_head_sha256: first.sha256,
        request_digest: digest(1),
        authority_bytes_sha256: digest(10),
        successor_capsule_sha256: digest(3),
        accepted_peer_attestation_sha256: digest(4),
        accepted_peer_identity_sha256: digest(5),
        event: event(JournalEventKind::EffectPrepared),
    };

    let alternate_path = root
        .join(SCOPE)
        .join("generation-00000000000000000002.json");
    fs::write(&alternate_path, canonical_bytes_v2(&alternate).unwrap()).unwrap();
    fs::set_permissions(&alternate_path, fs::Permissions::from_mode(0o400)).unwrap();

    let error = journal.head().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not exact-continue the claim"),
        "unexpected alternate-orphan error: {error:#}"
    );
}

#[test]
fn causally_invalid_unreferenced_successor_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();
    let first = journal
        .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
        .unwrap();
    let invalid = JournalGeneration {
        schema_owner: JOURNAL_OWNER.to_string(),
        schema_version: JOURNAL_VERSION,
        scope_id: SCOPE.to_string(),
        generation: 2,
        predecessor_head_sha256: first.sha256,
        request_digest: digest(1),
        authority_bytes_sha256: digest(2),
        successor_capsule_sha256: digest(3),
        accepted_peer_attestation_sha256: digest(4),
        accepted_peer_identity_sha256: digest(5),
        event: JournalEvent {
            kind: JournalEventKind::Complete,
            host_state: HostRetirementStateV2::Complete,
            effect_ordinal: None,
            effect_role: None,
            effect_identity_sha256: None,
            effect_invocation_attempt: None,
            observation_sha256: Some(digest(5)),
            response_sha256: Some(digest(6)),
            preserving_classification: None,
        },
    };
    let invalid_path = root
        .join(SCOPE)
        .join("generation-00000000000000000002.json");
    fs::write(&invalid_path, canonical_bytes_v2(&invalid).unwrap()).unwrap();
    fs::set_permissions(&invalid_path, fs::Permissions::from_mode(0o400)).unwrap();

    let error = journal.head().unwrap_err();
    assert!(error.to_string().contains("causally valid"));
}

#[test]
fn fixed_root_and_scope_locks_are_root_first_and_mode_bounded() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let _journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();
    for path in [
        root.join("journal-root.lock"),
        root.join(SCOPE).join("journal.lock"),
    ] {
        let metadata = fs::symlink_metadata(path).unwrap();
        assert!(metadata.file_type().is_file());
        assert_eq!(metadata.uid(), uid);
        assert_eq!(metadata.mode() & 0o7777, 0o600);
        assert_eq!(metadata.nlink(), 1);
    }
}

#[test]
fn journal_artifacts_are_reopened_no_follow() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();
    journal
        .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
        .unwrap();

    let generation = root
        .join(SCOPE)
        .join("generation-00000000000000000001.json");
    let substitution = temp.path().join("substituted.json");
    fs::write(&substitution, b"substituted generation").unwrap();
    fs::remove_file(&generation).unwrap();
    symlink(&substitution, &generation).unwrap();

    assert!(journal.head().is_err());
}

#[test]
fn journal_replay_rejects_artifact_mode_substitution() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();
    journal
        .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
        .unwrap();

    let generation = root
        .join(SCOPE)
        .join("generation-00000000000000000001.json");
    fs::set_permissions(&generation, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(journal.head().is_err());
}

#[test]
fn rejoin_attestation_requires_and_preserves_the_exact_initial_attestation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("journal");
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&root, SCOPE, uid).unwrap();
    let initial = b"canonical initial process attestation";
    let rejoin = b"canonical restarted process attestation";
    assert!(journal
        .persist_rejoin_peer_attestation(rejoin, &sha256_hex_v2(rejoin), &sha256_hex_v2(initial),)
        .is_err());

    journal
        .persist_acceptance_artifact("peer.attestation", initial, &sha256_hex_v2(initial))
        .unwrap();
    journal
        .persist_rejoin_peer_attestation(rejoin, &sha256_hex_v2(rejoin), &sha256_hex_v2(initial))
        .unwrap();
    assert_eq!(
        fs::read(
            root.join(SCOPE)
                .join(format!("peer.rejoin.{}.attestation", sha256_hex_v2(rejoin)))
        )
        .unwrap(),
        rejoin
    );
    assert!(journal
        .persist_rejoin_peer_attestation(rejoin, &sha256_hex_v2(rejoin), &digest(99),)
        .is_err());
    assert_eq!(
        journal.acceptance_artifact("peer.attestation").unwrap(),
        Some(initial.to_vec())
    );
}

#[test]
fn terminal_response_replay_is_byte_identical_and_immutable() {
    let temp = tempfile::tempdir().unwrap();
    let uid = unsafe { libc::geteuid() };
    let journal = LockedJournal::open_fixed(&temp.path().join("journal"), SCOPE, uid).unwrap();
    let bytes = br#"{"request_digest":"fixed","state":"host_complete"}"#;

    let first_digest = journal
        .persist_terminal_response(&digest(1), bytes)
        .unwrap();
    let replay_digest = journal
        .persist_terminal_response(&digest(1), bytes)
        .unwrap();
    assert_eq!(first_digest, replay_digest);
    assert_eq!(
        journal.terminal_response().unwrap().as_deref(),
        Some(bytes.as_slice())
    );
    assert!(journal
        .persist_terminal_response(&digest(1), b"different terminal response")
        .is_err());

    journal
        .persist_terminal_binding_artifact("parity.proof", b"proof", &sha256_hex_v2(b"proof"))
        .unwrap();
    assert_eq!(
        journal
            .terminal_binding_artifact("parity.proof")
            .unwrap()
            .as_deref(),
        Some(b"proof".as_slice())
    );
    assert!(journal
        .persist_terminal_binding_artifact(
            "parity.proof",
            b"alternate",
            &sha256_hex_v2(b"alternate"),
        )
        .is_err());
}
