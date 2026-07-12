use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::execution::agent_runtime::host_session_authority::canonical_json;
use crate::execution::agent_runtime::host_session_authority::store_schema::{
    AuthorityStoreCommitmentAlgorithmV1, AuthorityStoreCommitmentKeyStateV1,
    AuthorityStoreCommitmentKeyV1,
};
use crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot;

fn safe_test_parent() -> std::path::PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                .join(".cache")
        })
}

fn root() -> tempfile::TempDir {
    fs::create_dir_all(safe_test_parent()).unwrap();
    let root = tempfile::tempdir_in(safe_test_parent()).unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn material(seed: u8) -> InitializationMaterialV1 {
    InitializationMaterialV1 {
        store_entropy: [seed; 16],
        key_entropy: [seed.wrapping_add(1); 16],
        marker_nonce: [seed.wrapping_add(2); 16],
        key_nonce: [seed.wrapping_add(3); 16],
        root_nonce: [seed.wrapping_add(4); 16],
        secret_key: [seed.wrapping_add(5); 32],
        created_at: TimestampV1::parse("2026-07-11T12:49:11.000000000Z").unwrap(),
    }
}

#[test]
fn classifier_distinguishes_fresh_legacy_and_corrupt_temp_state() {
    let fresh = root();
    assert_eq!(
        platform::classify_diagnostic(fresh.path()).unwrap(),
        BootstrapClassificationV1::FreshAbsent
    );
    for (name, bytes) in [
        (
            "init--as_11111111111111111111111111111111--11111111111111111111111111111111.tmp",
            b"".as_slice(),
        ),
        (
            "key--ak_11111111111111111111111111111111--11111111111111111111111111111111.tmp",
            b"partial".as_slice(),
        ),
        (
            "object--ao_11111111111111111111111111111111--11111111111111111111111111111111.tmp",
            b"complete-but-nonauthoritative".as_slice(),
        ),
        (
            "root--r1--11111111111111111111111111111111.tmp",
            b"partial".as_slice(),
        ),
    ] {
        let recognized_temp = fresh.path().join("authority-v1/tmp").join(name);
        fs::write(&recognized_temp, bytes).unwrap();
        fs::set_permissions(&recognized_temp, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            classify(fresh.path()),
            BootstrapClassificationV1::FreshAbsent
        );
        assert!(!recognized_temp.exists());
    }

    let legacy = root();
    let sessions = legacy.path().join("run/agent-hub/sessions");
    fs::create_dir_all(&sessions).unwrap();
    for component in [
        legacy.path().join("run"),
        legacy.path().join("run/agent-hub"),
        sessions.clone(),
    ] {
        fs::set_permissions(component, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(sessions.join("legacy.json"), b"legacy").unwrap();
    fs::set_permissions(
        sessions.join("legacy.json"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert_eq!(
        classify(legacy.path()),
        BootstrapClassificationV1::UnsupportedLegacyState
    );

    let corrupt = root();
    assert_eq!(
        classify(corrupt.path()),
        BootstrapClassificationV1::FreshAbsent
    );
    let invalid_temp = corrupt.path().join("authority-v1/tmp/not-a-temp");
    fs::write(&invalid_temp, b"partial").unwrap();
    fs::set_permissions(&invalid_temp, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        classify(corrupt.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );
}

#[test]
fn initialization_binds_the_exact_legacy_collection_observation() {
    let replaced = root();
    fs::create_dir_all(replaced.path().join("run/agent-hub/sessions")).unwrap();
    assert!(platform::legacy_observation_mutation_test(
        replaced.path(),
        LegacyMutationV1::ReplaceSessions,
    )
    .is_err());

    let appeared = root();
    fs::create_dir_all(appeared.path().join("run/agent-hub")).unwrap();
    assert!(platform::legacy_observation_mutation_test(
        appeared.path(),
        LegacyMutationV1::CreateMissingSessions,
    )
    .is_err());
}

#[test]
fn committed_layout_components_and_unregistered_keys_fail_closed() {
    for relative in ["lock/root.lock", "tmp", "objects", "keys"] {
        let root = root();
        platform::bootstrap_test(root.path(), material(0x31), None).unwrap();
        let target = root.path().join("authority-v1").join(relative);
        if target.is_dir() {
            fs::remove_dir_all(&target).unwrap();
        } else {
            fs::remove_file(&target).unwrap();
        }
        assert_eq!(
            classify(root.path()),
            BootstrapClassificationV1::CorruptOrUnsupported,
            "missing committed layout component {relative} was repaired"
        );
    }

    let malformed_key = root();
    platform::bootstrap_test(malformed_key.path(), material(0x32), None).unwrap();
    let key_path = malformed_key
        .path()
        .join("authority-v1/keys/ak_99999999999999999999999999999999.key");
    fs::write(&key_path, b"partial-final-key").unwrap();
    fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        classify(malformed_key.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );
    assert!(key_path.exists());
}

#[test]
fn initialization_crash_windows_reuse_persisted_marker_identity() {
    for stop in [
        InitializationCrashPointV1::Marker,
        InitializationCrashPointV1::Key,
        InitializationCrashPointV1::Root,
    ] {
        let root = root();
        let original = material(0x11);
        assert!(platform::bootstrap_test(root.path(), original.clone(), Some(stop)).is_err());

        let recovered = platform::bootstrap_test(root.path(), material(0x55), None).unwrap();
        assert_eq!(
            recovered.authority_store_id,
            "as_11111111111111111111111111111111"
        );
        assert_eq!(
            recovered.active_commitment_key_id,
            "ak_12121212121212121212121212121212"
        );
        assert_eq!(
            recovered.bootstrap_home,
            recovered.greenfield_namespace_certificate.bootstrap_home
        );
        assert_eq!(
            recovered.greenfield_namespace_certificate.certified_at,
            original.created_at
        );
        assert_eq!(recovered.root_revision, 1);
        assert_eq!(
            classify(root.path()),
            BootstrapClassificationV1::ValidExisting
        );
        assert!(!root.path().join("authority-v1/init-v1.json").exists());

        let joined = platform::bootstrap_test(root.path(), material(0x77), None).unwrap();
        assert_eq!(joined, recovered);
    }
}

#[test]
fn competing_initializers_converge_on_one_persisted_identity() {
    let root = root();
    let path = std::sync::Arc::new(root.path().to_path_buf());
    let start = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let path = std::sync::Arc::clone(&path);
        let start = std::sync::Arc::clone(&start);
        workers.push(std::thread::spawn(move || {
            start.wait();
            bootstrap(&path).unwrap()
        }));
    }
    start.wait();
    let first = workers.remove(0).join().unwrap();
    let second = workers.remove(0).join().unwrap();
    assert_eq!(first, second);
    assert_eq!(first.root_revision, 1);
    assert_eq!(
        fs::read_dir(root.path().join("authority-v1/keys"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn key_rotation_and_retirement_reconcile_each_committed_boundary() {
    use crate::execution::agent_runtime::host_session_authority::store_schema::AuthorityStoreCommitmentKeyStateV1;

    let root = root();
    let initial = platform::bootstrap_test(root.path(), material(0x51), None).unwrap();
    let initial_key = initial.active_commitment_key_id.clone();

    assert!(platform::rotate_commitment_key_test(
        root.path(),
        material(0x52),
        Some(KeyLifecycleCrashPointV1::KeyPublished),
    )
    .is_err());
    let recovered = platform::bootstrap_test(root.path(), material(0x53), None).unwrap();
    assert_eq!(recovered, initial);
    assert_eq!(
        fs::read_dir(root.path().join("authority-v1/keys"))
            .unwrap()
            .count(),
        1
    );

    let rotated = platform::rotate_commitment_key_test(root.path(), material(0x54), None).unwrap();
    assert_ne!(rotated.active_commitment_key_id, initial_key);
    assert_eq!(
        rotated.commitment_key_registry[&initial_key].state,
        AuthorityStoreCommitmentKeyStateV1::VerificationOnly
    );
    assert_eq!(rotated.root_revision, initial.root_revision + 1);

    assert!(platform::retire_commitment_key_test(
        root.path(),
        &initial_key,
        [0x55; 16],
        Some(KeyLifecycleCrashPointV1::RootPublished),
    )
    .is_err());
    assert!(root
        .path()
        .join(format!("authority-v1/keys/{initial_key}.key"))
        .exists());
    let retired = platform::bootstrap_test(root.path(), material(0x56), None).unwrap();
    assert_eq!(
        retired.commitment_key_registry[&initial_key].state,
        AuthorityStoreCommitmentKeyStateV1::Retired
    );
    assert!(!root
        .path()
        .join(format!("authority-v1/keys/{initial_key}.key"))
        .exists());
    assert_eq!(retired.root_revision, rotated.root_revision + 1);
}

#[test]
fn root_cas_reconciles_unregistered_key_before_candidate_validation() {
    let root = root();
    let initial = platform::bootstrap_test(root.path(), material(0x51), None).unwrap();
    let interrupted = material(0x52);
    assert!(platform::rotate_commitment_key_test(
        root.path(),
        interrupted.clone(),
        Some(KeyLifecycleCrashPointV1::KeyPublished),
    )
    .is_err());

    let new_key_id = "ak_53535353535353535353535353535353";
    let new_key_path = root
        .path()
        .join("authority-v1/keys")
        .join(format!("{new_key_id}.key"));
    assert!(new_key_path.exists());
    let root_path = root.path().join("authority-v1/state-root-v1.json");
    let original_root = fs::read(&root_path).unwrap();

    let mut proposed = initial.clone();
    proposed.root_revision += 1;
    proposed
        .commitment_key_registry
        .get_mut(&initial.active_commitment_key_id)
        .unwrap()
        .state = AuthorityStoreCommitmentKeyStateV1::VerificationOnly;
    proposed.commitment_key_registry.insert(
        new_key_id.into(),
        AuthorityStoreCommitmentKeyV1 {
            schema_version: 1,
            authority_store_id: initial.authority_store_id.clone(),
            key_id: new_key_id.into(),
            algorithm: AuthorityStoreCommitmentAlgorithmV1::HmacSha256,
            created_at: interrupted.created_at,
            state: AuthorityStoreCommitmentKeyStateV1::Active,
        },
    );
    proposed.active_commitment_key_id = new_key_id.into();
    proposed.validate().unwrap();

    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: initial.root_revision,
            authority: None,
        },
        &proposed,
    )
    .is_err());
    assert_eq!(fs::read(root_path).unwrap(), original_root);
    assert!(!new_key_path.exists());
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::ValidExisting
    );
}

#[test]
fn stale_key_rotation_rejects_before_key_root_or_temp_mutation() {
    let root = root();
    let current = platform::bootstrap_test(root.path(), material(0x57), None).unwrap();
    let root_path = root.path().join("authority-v1/state-root-v1.json");
    let root_bytes = fs::read(&root_path).unwrap();
    let key_names = fs::read_dir(root.path().join("authority-v1/keys"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();

    assert!(rotate_commitment_key(root.path(), current.root_revision + 1).is_err());
    assert_eq!(fs::read(root_path).unwrap(), root_bytes);
    assert_eq!(
        fs::read_dir(root.path().join("authority-v1/keys"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
        key_names
    );
    assert!(fs::read_dir(root.path().join("authority-v1/tmp"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn verification_only_keys_join_existing_orphans_but_cannot_publish_new_objects() {
    use crate::execution::agent_runtime::host_session_authority::hash::{
        store_hmac_sha256, SensitiveDomainV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1,
    };

    let root = root();
    let init = material(0x58);
    let state = platform::bootstrap_test(root.path(), init.clone(), None).unwrap();
    let context = ObjectVerificationContextV1 {
        intent_id: "intent-key-state".into(),
        run_id: "run-key-state".into(),
        parent_intent: None,
    };
    let bytes = b"verification-only-lease";
    let commitment = AuthorityObjectCommitmentV1::StoreHmacSha256 {
        key_id: state.active_commitment_key_id.clone(),
        domain: String::from_utf8(SensitiveDomainV1::ParticipantLeaseToken.as_bytes().to_vec())
            .unwrap(),
        digest_hex: store_hmac_sha256(
            &init.secret_key,
            SensitiveDomainV1::ParticipantLeaseToken,
            &state.authority_store_id,
            &context.intent_id,
            Some(&context.run_id),
            bytes,
        )
        .unwrap(),
    };
    let existing = AuthorityObjectRefV1 {
        ref_id: "ao_12121212121212121212121212121212".into(),
        object_kind: AuthorityObjectKindV1::LeaseToken,
        schema_version: 1,
        commitment: commitment.clone(),
    };
    assert_eq!(
        platform::publish_object_test(root.path(), &existing, bytes, Some(&context), [0x59; 16],)
            .unwrap(),
        ObjectPublicationOutcomeV1::PublishedOrphan
    );
    platform::rotate_commitment_key_test(root.path(), material(0x5a), None).unwrap();
    assert_eq!(
        platform::publish_object_test(root.path(), &existing, bytes, Some(&context), [0x5b; 16],)
            .unwrap(),
        ObjectPublicationOutcomeV1::JoinedExactOrphan
    );
    let absent = AuthorityObjectRefV1 {
        ref_id: "ao_13131313131313131313131313131313".into(),
        object_kind: AuthorityObjectKindV1::LeaseToken,
        schema_version: 1,
        commitment,
    };
    assert!(
        platform::publish_object_test(root.path(), &absent, bytes, Some(&context), [0x5c; 16],)
            .is_err()
    );
}

#[test]
fn mutators_run_closed_layout_and_temp_reconciliation_before_changes() {
    let reconciled = root();
    platform::bootstrap_test(reconciled.path(), material(0x5d), None).unwrap();
    let temp = reconciled.path().join(
        "authority-v1/tmp/key--ak_14141414141414141414141414141414--15151515151515151515151515151515.tmp",
    );
    fs::write(&temp, b"partial").unwrap();
    fs::set_permissions(&temp, fs::Permissions::from_mode(0o600)).unwrap();
    platform::rotate_commitment_key_test(reconciled.path(), material(0x5e), None).unwrap();
    assert!(!temp.exists());

    let blocked = root();
    platform::bootstrap_test(blocked.path(), material(0x5f), None).unwrap();
    let root_file = blocked.path().join("authority-v1/state-root-v1.json");
    let before = fs::read(&root_file).unwrap();
    let extra = blocked.path().join("authority-v1/unexpected");
    fs::write(&extra, b"unexpected").unwrap();
    fs::set_permissions(&extra, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(platform::rotate_commitment_key_test(blocked.path(), material(0x60), None).is_err());
    assert_eq!(fs::read(root_file).unwrap(), before);

    let mismatched_marker = root();
    let state = platform::bootstrap_test(mismatched_marker.path(), material(0x61), None).unwrap();
    let root_file = mismatched_marker
        .path()
        .join("authority-v1/state-root-v1.json");
    let before = fs::read(&root_file).unwrap();
    let marker = crate::execution::agent_runtime::host_session_authority::store_schema::AuthorityStoreInitializationV1 {
        schema_version: 1,
        authority_store_id: "as_99999999999999999999999999999999".into(),
        bootstrap_home: state.bootstrap_home.clone(),
        initial_key_id: state.active_commitment_key_id.clone(),
        created_at: state.greenfield_namespace_certificate.certified_at.clone(),
    };
    let marker_file = mismatched_marker.path().join("authority-v1/init-v1.json");
    fs::write(
        &marker_file,
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&marker)
            .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&marker_file, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(
        platform::rotate_commitment_key_test(mismatched_marker.path(), material(0x62), None)
            .is_err()
    );
    assert_eq!(fs::read(root_file).unwrap(), before);
    assert!(marker_file.exists());
}

#[test]
fn existing_store_rejects_missing_keys_and_invalid_object_routes() {
    let missing_key = root();
    platform::bootstrap_test(missing_key.path(), material(0x21), None).unwrap();
    fs::remove_file(
        missing_key
            .path()
            .join("authority-v1/keys/ak_22222222222222222222222222222222.key"),
    )
    .unwrap();
    assert_eq!(
        classify(missing_key.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );

    let unknown_route = root();
    platform::bootstrap_test(unknown_route.path(), material(0x31), None).unwrap();
    let unknown = unknown_route
        .path()
        .join("authority-v1/objects/unknown-kind");
    fs::create_dir(&unknown).unwrap();
    fs::set_permissions(&unknown, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        classify(unknown_route.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );

    let orphan = root();
    platform::bootstrap_test(orphan.path(), material(0x41), None).unwrap();
    let orphan_directory = orphan.path().join("authority-v1/objects/policy/v1");
    fs::create_dir_all(&orphan_directory).unwrap();
    for directory in [
        orphan_directory.parent().unwrap(),
        orphan_directory.as_path(),
    ] {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let orphan_file = orphan_directory.join("ao_99999999999999999999999999999999.obj");
    fs::write(&orphan_file, b"orphan").unwrap();
    fs::set_permissions(&orphan_file, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        classify(orphan.path()),
        BootstrapClassificationV1::ValidExisting
    );

    let missing_object = root();
    let mut state = platform::bootstrap_test(missing_object.path(), material(0x51), None).unwrap();
    let ref_id = "ao_88888888888888888888888888888888".to_string();
    state.object_index.insert(
            ref_id.clone(),
            crate::execution::agent_runtime::host_session_authority::store_schema::AuthorityObjectIndexEntryV1 {
                schema_version: 1,
                ref_id,
                object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy,
                object_schema_version: 1,
                byte_length: 3,
                storage_state: crate::execution::agent_runtime::host_session_authority::store_schema::AuthorityObjectStorageStateV1::Present,
            },
        );
    fs::write(
        missing_object
            .path()
            .join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&state)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(missing_object.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );
}

#[test]
fn typed_orphan_publication_joins_only_an_exact_retry() {
    let root = root();
    let current = platform::bootstrap_test(root.path(), material(0x61), None).unwrap();
    let value =
        crate::execution::agent_runtime::host_session_authority::schema::PolicyObjectHashInputV1 {
            schema_version: 1,
            policy_revision: "policy-1".into(),
            canonical_policy_snapshot_sha256:
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        };
    let bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&value)
            .unwrap();
    let digest =
        crate::execution::agent_runtime::host_session_authority::hash::canonical_sha256(&value)
            .unwrap();
    let reference = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
            ref_id: "ao_77777777777777777777777777777777".into(),
            object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex: digest },
        };
    assert!(prepare_typed_object(
        root.path(),
        current.root_revision + 1,
        &reference,
        &bytes,
        None,
    )
    .is_err());
    assert!(!root.path().join("authority-v1/objects/policy").exists());
    assert!(fs::read_dir(root.path().join("authority-v1/tmp"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(
        platform::publish_object_test(root.path(), &reference, &bytes, None, [0x71; 16],).unwrap(),
        ObjectPublicationOutcomeV1::PublishedOrphan
    );
    assert_eq!(
        platform::publish_object_test(root.path(), &reference, &bytes, None, [0x72; 16],).unwrap(),
        ObjectPublicationOutcomeV1::JoinedExactOrphan
    );
    platform::verify_orphan_test(root.path(), &reference, &bytes, None).unwrap();
    assert!(platform::publish_object_test(
        root.path(),
        &reference,
        b"substituted",
        None,
        [0x73; 16],
    )
    .is_err());
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::ValidExisting
    );
}

#[test]
fn typed_transport_and_nested_object_graphs_must_match_their_parent() {
    use crate::execution::agent_runtime::host_session_authority::hash::{
        canonical_sha256, store_hmac_sha256, SensitiveDomainV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AgentDescriptorHashInputV1, ApplicationResultHashInputV1, ApplicationResultPhaseV1,
        AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1,
        DurableSessionAuthorityOriginV1, HostAttachContractHashInputV1,
        HostSessionAuthorityPreconditionV1, HostSessionPostureV1,
        HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
        HostSessionTransitionModeV1, PolicyObjectHashInputV1, ResumeHandleHashInputV1,
        TerminalHandoffHashInputV1, TerminalHandoffStateV1, TransitionTransportPayloadObjectV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::{
        AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
        HostSessionTransitionApplicationJournalV1, HostSessionTransitionInputHandoffV1,
        HostSessionTransitionIntentStateV1, HostSessionTransitionIntentV1,
        HostSessionTransitionTransportPayloadStateV1, InitialTransitionApplicationJournalV1,
        IssuerRequestIndexEntryV1, SessionIdReservationV1, SessionIdTombstoneV1,
        SessionNamespaceRecordV1, StartTombstoneStateV1,
    };

        fn canonical_ref<T>(
            ref_id: &str,
            kind: AuthorityObjectKindV1,
            bytes: &[u8],
        ) -> AuthorityObjectRefV1
        where
            T: serde::de::DeserializeOwned
                + serde::Serialize
                + crate::execution::agent_runtime::host_session_authority::validation::CanonicalHashInputV1,
        {
        let value: T =
            crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(
                bytes,
            )
            .unwrap();
        AuthorityObjectRefV1 {
            ref_id: ref_id.into(),
            object_kind: kind,
            schema_version: 1,
            commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_sha256(&value).unwrap(),
            },
        }
    }

    fn index(reference: &AuthorityObjectRefV1, bytes: &[u8]) -> AuthorityObjectIndexEntryV1 {
        AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: reference.ref_id.clone(),
            object_kind: reference.object_kind,
            object_schema_version: reference.schema_version,
            byte_length: bytes.len() as u64,
            storage_state: AuthorityObjectStorageStateV1::Present,
        }
    }

    let root = root();
    let init = material(0x61);
    let mut candidate = platform::bootstrap_test(root.path(), init.clone(), None).unwrap();
    let initial_root = candidate.clone();
    let intent_id = "intent-vector";
    let run_id = "run-vector";
    let request_id = "request-vector";
    let session_id = "session-vector";
    let context = ObjectVerificationContextV1 {
        intent_id: intent_id.into(),
        run_id: run_id.into(),
        parent_intent: None,
    };
    let descriptor_bytes = include_bytes!("testdata/agent-descriptor.json").as_slice();
    let attach_bytes = include_bytes!("testdata/host-attach-contract.json").as_slice();
    let lease_bytes = include_bytes!("testdata/lease-token.bin").as_slice();
    let transport_bytes = include_bytes!("testdata/transport-payload.json").as_slice();
    let descriptor_ref = canonical_ref::<AgentDescriptorHashInputV1>(
        "ao_11111111111111111111111111111111",
        AuthorityObjectKindV1::AgentDescriptor,
        descriptor_bytes,
    );
    let attach_ref = canonical_ref::<HostAttachContractHashInputV1>(
        "ao_22222222222222222222222222222222",
        AuthorityObjectKindV1::HostAttachContract,
        attach_bytes,
    );
    let active_key = candidate.active_commitment_key_id.clone();
    let lease_ref = AuthorityObjectRefV1 {
        ref_id: "ao_33333333333333333333333333333333".into(),
        object_kind: AuthorityObjectKindV1::LeaseToken,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id: active_key.clone(),
            domain: String::from_utf8(SensitiveDomainV1::ParticipantLeaseToken.as_bytes().to_vec())
                .unwrap(),
            digest_hex: store_hmac_sha256(
                &init.secret_key,
                SensitiveDomainV1::ParticipantLeaseToken,
                &candidate.authority_store_id,
                intent_id,
                Some(run_id),
                lease_bytes,
            )
            .unwrap(),
        },
    };
    let transport_ref = AuthorityObjectRefV1 {
        ref_id: "ao_44444444444444444444444444444444".into(),
        object_kind: AuthorityObjectKindV1::TransitionTransportPayload,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id: active_key,
            domain: String::from_utf8(SensitiveDomainV1::RawTransportPayload.as_bytes().to_vec())
                .unwrap(),
            digest_hex: store_hmac_sha256(
                &init.secret_key,
                SensitiveDomainV1::RawTransportPayload,
                &candidate.authority_store_id,
                intent_id,
                Some(run_id),
                transport_bytes,
            )
            .unwrap(),
        },
    };

    for (reference, bytes, object_context, nonce) in [
        (&descriptor_ref, descriptor_bytes, None, [0x71; 16]),
        (&attach_ref, attach_bytes, None, [0x72; 16]),
        (&lease_ref, lease_bytes, Some(&context), [0x73; 16]),
    ] {
        platform::publish_object_test(root.path(), reference, bytes, object_context, nonce)
            .unwrap();
    }

    let payload_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
    };
    candidate.session_namespace_map.insert(
        session_id.into(),
        SessionNamespaceRecordV1::StartReservation(SessionIdReservationV1 {
            schema_version: 1,
            orchestration_session_id: session_id.into(),
            intent_id: intent_id.into(),
            issuer_request_id: request_id.into(),
            payload_commitment: payload_commitment.clone(),
            reserved_at: init.created_at.clone(),
        }),
    );
    candidate.transition_intent_map.insert(
            intent_id.into(),
            HostSessionTransitionIntentV1 {
                schema_version: 1,
                intent_id: intent_id.into(),
                issuer_request_id: request_id.into(),
                intent_revision: 1,
                mode: HostSessionTransitionModeV1::Start,
                authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
                orchestration_session_id: session_id.into(),
                shell_trace_session_id: "trace-vector".into(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::PublicCli,
                    caller_participant_id: None,
                    auto_attach_obligation_id: None,
                    auto_attach_claim_owner: None,
                },
                source_authoritative_participant_id: None,
                target_authoritative_participant_id: "participant-vector".into(),
                target_participant_lease_token_ref: lease_ref.clone(),
                run_id: run_id.into(),
                resulting_authoritative_lineage: vec!["participant-vector".into()],
                workspace_binding: crate::execution::agent_runtime::host_session_authority::schema::WorkspaceBindingV1 {
                    workspace_root: candidate.bootstrap_home.clone(),
                    authority_store_root: candidate.bootstrap_home.clone(),
                    authority_store_id: candidate.authority_store_id.clone(),
                },
                world_binding: None,
                descriptor_ref: descriptor_ref.clone(),
                host_attach_contract_ref: attach_ref.clone(),
                resume_handle_ref: None,
                transition_input_ref: None,
                post_turn_disposition: None,
                transport_payload_ref: transport_ref.clone(),
                payload_commitment: payload_commitment.clone(),
                issued_at: init.created_at.clone(),
                expires_at: TimestampV1::parse("2026-07-11T12:54:11.000000000Z").unwrap(),
                state: HostSessionTransitionIntentStateV1::Issued,
                input_handoff: HostSessionTransitionInputHandoffV1::NotApplicable,
                transport_payload_state: HostSessionTransitionTransportPayloadStateV1::Retained,
                updated_at: init.created_at.clone(),
            },
        );
    candidate.issuer_request_index.insert(
        request_id.into(),
        IssuerRequestIndexEntryV1 {
            schema_version: 1,
            issuer_request_id: request_id.into(),
            orchestration_session_id: session_id.into(),
            intent_id: intent_id.into(),
            payload_commitment,
        },
    );
    for (reference, bytes) in [
        (&descriptor_ref, descriptor_bytes),
        (&attach_ref, attach_bytes),
        (&lease_ref, lease_bytes),
        (&transport_ref, transport_bytes),
    ] {
        candidate
            .object_index
            .insert(reference.ref_id.clone(), index(reference, bytes));
    }
    let parent = candidate.transition_intent_map[intent_id].clone();
    let mismatched_context = ObjectVerificationContextV1 {
        intent_id: intent_id.into(),
        run_id: run_id.into(),
        parent_intent: Some(Box::new(parent.clone())),
    };
    assert!(platform::publish_object_test(
        root.path(),
        &transport_ref,
        transport_bytes,
        Some(&mismatched_context),
        [0x74; 16],
    )
    .is_err());

    let mut matching_payload: TransitionTransportPayloadObjectV1 =
        crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(
            transport_bytes,
        )
        .unwrap();
    matching_payload.intent_id = parent.intent_id.clone();
    matching_payload.mode = parent.mode;
    matching_payload.orchestration_session_id = parent.orchestration_session_id.clone();
    matching_payload.shell_trace_session_id = parent.shell_trace_session_id.clone();
    matching_payload.caller = parent.caller.clone();
    matching_payload.source_authoritative_participant_id =
        parent.source_authoritative_participant_id.clone();
    matching_payload.target_authoritative_participant_id =
        parent.target_authoritative_participant_id.clone();
    matching_payload.target_participant_lease_token_ref =
        parent.target_participant_lease_token_ref.clone();
    matching_payload.run_id = parent.run_id.clone();
    matching_payload.resulting_authoritative_lineage =
        parent.resulting_authoritative_lineage.clone();
    matching_payload.workspace_binding = parent.workspace_binding.clone();
    matching_payload.world_binding = parent.world_binding.clone();
    matching_payload.descriptor_ref = parent.descriptor_ref.clone();
    matching_payload.host_attach_contract_ref = parent.host_attach_contract_ref.clone();
    matching_payload.resume_handle_ref = parent.resume_handle_ref.clone();
    matching_payload.transition_input_ref = parent.transition_input_ref.clone();
    matching_payload.post_turn_disposition = parent.post_turn_disposition;
    let matching_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &matching_payload,
        )
        .unwrap();
    let matching_ref = AuthorityObjectRefV1 {
        ref_id: transport_ref.ref_id.clone(),
        object_kind: transport_ref.object_kind,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id: candidate.active_commitment_key_id.clone(),
            domain: String::from_utf8(SensitiveDomainV1::RawTransportPayload.as_bytes().to_vec())
                .unwrap(),
            digest_hex: store_hmac_sha256(
                &init.secret_key,
                SensitiveDomainV1::RawTransportPayload,
                &candidate.authority_store_id,
                intent_id,
                Some(run_id),
                &matching_bytes,
            )
            .unwrap(),
        },
    };
    candidate
        .transition_intent_map
        .get_mut(intent_id)
        .unwrap()
        .transport_payload_ref = matching_ref.clone();
    candidate.object_index.insert(
        matching_ref.ref_id.clone(),
        index(&matching_ref, &matching_bytes),
    );
    candidate.root_revision += 1;
    candidate.validate().unwrap();
    let exact_context = ObjectVerificationContextV1 {
        intent_id: intent_id.into(),
        run_id: run_id.into(),
        parent_intent: Some(Box::new(candidate.transition_intent_map[intent_id].clone())),
    };
    for bad_context in [
        ObjectVerificationContextV1 {
            intent_id: "different-intent".into(),
            ..exact_context.clone()
        },
        ObjectVerificationContextV1 {
            run_id: "different-run".into(),
            ..exact_context.clone()
        },
    ] {
        assert!(platform::publish_object_test(
            root.path(),
            &matching_ref,
            &matching_bytes,
            Some(&bad_context),
            [0x74; 16],
        )
        .is_err());
    }
    let mut wrong_ref = matching_ref.clone();
    wrong_ref.ref_id = "ao_45454545454545454545454545454545".into();
    assert!(platform::publish_object_test(
        root.path(),
        &wrong_ref,
        &matching_bytes,
        Some(&exact_context),
        [0x74; 16],
    )
    .is_err());
    assert_eq!(
        platform::publish_object_test(
            root.path(),
            &matching_ref,
            &matching_bytes,
            Some(&exact_context),
            [0x75; 16],
        )
        .unwrap(),
        ObjectPublicationOutcomeV1::PublishedOrphan
    );
    assert_eq!(
        platform::publish_object_test(
            root.path(),
            &matching_ref,
            &matching_bytes,
            Some(&exact_context),
            [0x76; 16],
        )
        .unwrap(),
        ObjectPublicationOutcomeV1::JoinedExactOrphan
    );
    assert_eq!(
        platform::bootstrap_test(root.path(), material(0x77), None)
            .unwrap()
            .root_revision,
        1
    );
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&candidate)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );

    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &initial_root,
        )
        .unwrap(),
    )
    .unwrap();
    let policy_bytes = include_bytes!("testdata/policy.json").as_slice();
    let policy_ref = canonical_ref::<PolicyObjectHashInputV1>(
        "ao_77777777777777777777777777777777",
        AuthorityObjectKindV1::Policy,
        policy_bytes,
    );
    let mut valid_attach: HostAttachContractHashInputV1 =
        crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(
            attach_bytes,
        )
        .unwrap();
    valid_attach.contract.descriptor_ref = descriptor_ref.clone();
    valid_attach.contract.policy_ref = policy_ref.clone();
    valid_attach.contract.continuity_resume_handle_ref = None;
    let valid_attach_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &valid_attach,
        )
        .unwrap();
    let valid_attach_ref = AuthorityObjectRefV1 {
        ref_id: "ao_55555555555555555555555555555555".into(),
        object_kind: AuthorityObjectKindV1::HostAttachContract,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&valid_attach).unwrap(),
        },
    };
    candidate.object_index.remove(&attach_ref.ref_id);
    candidate.object_index.remove(&matching_ref.ref_id);
    let intent = candidate.transition_intent_map.get_mut(intent_id).unwrap();
    intent.host_attach_contract_ref = valid_attach_ref.clone();
    intent.resume_handle_ref = None;
    let mut valid_payload = matching_payload;
    valid_payload.host_attach_contract_ref = valid_attach_ref.clone();
    valid_payload.resume_handle_ref = None;
    let valid_transport_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &valid_payload,
        )
        .unwrap();
    let valid_transport_ref = AuthorityObjectRefV1 {
        ref_id: "ao_66666666666666666666666666666666".into(),
        object_kind: AuthorityObjectKindV1::TransitionTransportPayload,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id: candidate.active_commitment_key_id.clone(),
            domain: String::from_utf8(SensitiveDomainV1::RawTransportPayload.as_bytes().to_vec())
                .unwrap(),
            digest_hex: store_hmac_sha256(
                &init.secret_key,
                SensitiveDomainV1::RawTransportPayload,
                &candidate.authority_store_id,
                intent_id,
                Some(run_id),
                &valid_transport_bytes,
            )
            .unwrap(),
        },
    };
    candidate
        .transition_intent_map
        .get_mut(intent_id)
        .unwrap()
        .transport_payload_ref = valid_transport_ref.clone();
    for (reference, bytes) in [
        (&policy_ref, policy_bytes),
        (&valid_attach_ref, valid_attach_bytes.as_slice()),
        (&valid_transport_ref, valid_transport_bytes.as_slice()),
    ] {
        candidate
            .object_index
            .insert(reference.ref_id.clone(), index(reference, bytes));
    }
    candidate.validate().unwrap();
    for (reference, bytes, nonce) in [
        (&policy_ref, policy_bytes, [0x81; 16]),
        (&valid_attach_ref, valid_attach_bytes.as_slice(), [0x82; 16]),
    ] {
        platform::publish_object_test(root.path(), reference, bytes, None, nonce).unwrap();
    }
    let valid_context = ObjectVerificationContextV1 {
        intent_id: intent_id.into(),
        run_id: run_id.into(),
        parent_intent: Some(Box::new(candidate.transition_intent_map[intent_id].clone())),
    };
    platform::publish_object_test(
        root.path(),
        &valid_transport_ref,
        &valid_transport_bytes,
        Some(&valid_context),
        [0x83; 16],
    )
    .unwrap();
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&candidate)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::ValidExisting
    );

    let resume_bytes = include_bytes!("testdata/resume-handle.json").as_slice();
    let resume_ref = canonical_ref::<ResumeHandleHashInputV1>(
        "ao_88888888888888888888888888888888",
        AuthorityObjectKindV1::ResumeHandle,
        resume_bytes,
    );
    let mut authority_attach = valid_attach.clone();
    authority_attach.contract.continuity_resume_handle_ref = Some(resume_ref.clone());
    let authority_attach_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &authority_attach,
        )
        .unwrap();
    let authority_attach_ref = AuthorityObjectRefV1 {
        ref_id: "ao_89898989898989898989898989898989".into(),
        object_kind: AuthorityObjectKindV1::HostAttachContract,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&authority_attach).unwrap(),
        },
    };
    let authority_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
    };
    let applied_at = TimestampV1::parse("2026-07-11T12:54:12.000000000Z").unwrap();
    let application = ApplicationResultHashInputV1 {
        schema_version: 1,
        intent_id: intent_id.into(),
        mode: HostSessionTransitionModeV1::Start,
        run_id: run_id.into(),
        phase: ApplicationResultPhaseV1::InitialTransition {
            authority_revision_before: None,
            authority_revision_after: 1,
            active_authoritative_participant_id: "participant-vector".into(),
            resulting_posture: HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: authority_commitment.clone(),
            post_turn_pending_run_id: None,
        },
        applied_at: applied_at.clone(),
    };
    let application_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &application,
        )
        .unwrap();
    let application_ref = AuthorityObjectRefV1 {
        ref_id: "ao_90909090909090909090909090909090".into(),
        object_kind: AuthorityObjectKindV1::ApplicationResult,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&application).unwrap(),
        },
    };
    let mut applied = candidate.clone();
    let intent = applied.transition_intent_map.get_mut(intent_id).unwrap();
    intent.state = HostSessionTransitionIntentStateV1::Applied {
        claim_id: "claim-vector".into(),
        authority_revision_before: None,
        authority_revision_after: 1,
        active_authoritative_participant_id: "participant-vector".into(),
        resulting_posture: HostSessionPostureV1::ActiveAttached,
        authority_record_commitment: authority_commitment.clone(),
        application_result_ref: application_ref.clone(),
        post_turn: Box::new(
            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionPostTurnApplicationV1::NotApplicable,
        ),
        applied_at: applied_at.clone(),
    };
    applied.session_namespace_map.insert(
        session_id.into(),
        SessionNamespaceRecordV1::Authority(Box::new(DurableSessionAuthorityV1 {
            schema_version: 1,
            orchestration_session_id: session_id.into(),
            shell_trace_session_id: "trace-vector".into(),
            authority_revision: 1,
            origin: DurableSessionAuthorityOriginV1::StartIntent {
                intent_id: intent_id.into(),
                issuer_request_id: request_id.into(),
                payload_commitment: intent.payload_commitment.clone(),
            },
            authoritative_participant_lineage: vec!["participant-vector".into()],
            active_authoritative_participant_id: Some("participant-vector".into()),
            workspace_binding: intent.workspace_binding.clone(),
            world_binding: None,
            host_attach_contract_ref: Some(authority_attach_ref.clone()),
            retained_worker_refs: Vec::new(),
            internal_resume_handle_refs: vec![resume_ref.clone()],
            lifecycle_posture: HostSessionPostureV1::ActiveAttached,
            current_policy_ref: Some(policy_ref.clone()),
            current_policy_revision: Some("policy-v1".into()),
            updated_at: applied_at.clone(),
        })),
    );
    applied.application_journal.insert(
        intent_id.into(),
        HostSessionTransitionApplicationJournalV1 {
            schema_version: 1,
            intent_id: intent_id.into(),
            initial_application: InitialTransitionApplicationJournalV1 {
                authority_revision_before: None,
                authority_revision_after: 1,
                authority_record_commitment: authority_commitment,
                application_result_ref: application_ref.clone(),
                applied_at: applied_at.clone(),
            },
            post_turn_application: None,
        },
    );
    for (reference, bytes) in [
        (&resume_ref, resume_bytes),
        (&authority_attach_ref, authority_attach_bytes.as_slice()),
        (&application_ref, application_bytes.as_slice()),
    ] {
        applied
            .object_index
            .insert(reference.ref_id.clone(), index(reference, bytes));
    }
    applied.root_revision += 1;
    applied.validate().unwrap();
    for (reference, bytes, nonce) in [
        (&resume_ref, resume_bytes, [0x84; 16]),
        (
            &authority_attach_ref,
            authority_attach_bytes.as_slice(),
            [0x85; 16],
        ),
        (&application_ref, application_bytes.as_slice(), [0x86; 16]),
    ] {
        platform::publish_object_test(root.path(), reference, bytes, None, nonce).unwrap();
    }
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&applied)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::ValidExisting
    );
    let applied_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: applied.root_revision - 1,
            authority: Some(ExpectedAuthorityRevisionV1 {
                orchestration_session_id: session_id.into(),
                authority_revision: 0,
            }),
        },
        &applied,
    )
    .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        applied_bytes
    );

    let mut authority_update = applied.clone();
    authority_update.root_revision += 1;
    let SessionNamespaceRecordV1::Authority(authority) = authority_update
        .session_namespace_map
        .get_mut(session_id)
        .unwrap()
    else {
        panic!("expected authority")
    };
    authority.authority_revision += 1;
    let authority_expectation = ExpectedRevisionsV1 {
        root_revision: applied.root_revision,
        authority: Some(ExpectedAuthorityRevisionV1 {
            orchestration_session_id: session_id.into(),
            authority_revision: 1,
        }),
    };
    assert_eq!(
        compare_and_swap_root(root.path(), &authority_expectation, &authority_update).unwrap(),
        TransactionCommitOutcomeV1::Committed(authority_update.clone())
    );
    assert!(compare_and_swap_root(root.path(), &authority_expectation, &authority_update).is_err());
    let committed_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    let stale_authority_expectation = ExpectedRevisionsV1 {
        root_revision: applied.root_revision,
        authority: Some(ExpectedAuthorityRevisionV1 {
            orchestration_session_id: session_id.into(),
            authority_revision: 0,
        }),
    };
    assert!(
        compare_and_swap_root(root.path(), &stale_authority_expectation, &authority_update,)
            .is_err()
    );
    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: applied.root_revision,
            authority: None,
        },
        &authority_update,
    )
    .is_err());
    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: applied.root_revision,
            authority: Some(ExpectedAuthorityRevisionV1 {
                orchestration_session_id: "other-session".into(),
                authority_revision: 1,
            }),
        },
        &authority_update,
    )
    .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        committed_bytes
    );
    assert!(fs::read_dir(root.path().join("authority-v1/tmp"))
        .unwrap()
        .next()
        .is_none());
    let mut unrelated_root_commit = authority_update.clone();
    unrelated_root_commit.root_revision += 1;
    assert_eq!(
        compare_and_swap_root(
            root.path(),
            &ExpectedRevisionsV1 {
                root_revision: authority_update.root_revision,
                authority: None,
            },
            &unrelated_root_commit,
        )
        .unwrap(),
        TransactionCommitOutcomeV1::Committed(unrelated_root_commit.clone())
    );
    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: authority_update.root_revision,
            authority: Some(ExpectedAuthorityRevisionV1 {
                orchestration_session_id: session_id.into(),
                authority_revision: 1,
            }),
        },
        &unrelated_root_commit,
    )
    .is_err());
    let mut multiple_authorities = authority_update.clone();
    multiple_authorities.root_revision += 1;
    let mut second_authority = match multiple_authorities
        .session_namespace_map
        .get(session_id)
        .unwrap()
        .clone()
    {
        SessionNamespaceRecordV1::Authority(authority) => authority,
        _ => panic!("expected authority"),
    };
    second_authority.orchestration_session_id = "session-2".into();
    second_authority.shell_trace_session_id = "trace-2".into();
    multiple_authorities.session_namespace_map.insert(
        "session-2".into(),
        SessionNamespaceRecordV1::Authority(second_authority),
    );
    assert!(platform::validate_exact_retry_expectation_test(
        &ExpectedRevisionsV1 {
            root_revision: authority_update.root_revision,
            authority: Some(ExpectedAuthorityRevisionV1 {
                orchestration_session_id: session_id.into(),
                authority_revision: 1,
            }),
        },
        &multiple_authorities,
    )
    .is_err());
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&applied)
            .unwrap(),
    )
    .unwrap();

    let mut wrong_resume: ResumeHandleHashInputV1 =
        crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(
            resume_bytes,
        )
        .unwrap();
    wrong_resume.orchestration_session_id = "other-session".into();
    let wrong_resume_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &wrong_resume,
        )
        .unwrap();
    let wrong_resume_ref = AuthorityObjectRefV1 {
        ref_id: "ao_91919191919191919191919191919191".into(),
        object_kind: AuthorityObjectKindV1::ResumeHandle,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&wrong_resume).unwrap(),
        },
    };
    let mut wrong_attach = authority_attach;
    wrong_attach.contract.continuity_resume_handle_ref = Some(wrong_resume_ref.clone());
    let wrong_attach_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &wrong_attach,
        )
        .unwrap();
    let wrong_attach_ref = AuthorityObjectRefV1 {
        ref_id: "ao_92929292929292929292929292929292".into(),
        object_kind: AuthorityObjectKindV1::HostAttachContract,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&wrong_attach).unwrap(),
        },
    };
    for (reference, bytes, nonce) in [
        (&wrong_resume_ref, wrong_resume_bytes.as_slice(), [0x87; 16]),
        (&wrong_attach_ref, wrong_attach_bytes.as_slice(), [0x88; 16]),
    ] {
        platform::publish_object_test(root.path(), reference, bytes, None, nonce).unwrap();
    }
    let mut wrong_graph = applied.clone();
    let SessionNamespaceRecordV1::Authority(authority) = wrong_graph
        .session_namespace_map
        .get_mut(session_id)
        .unwrap()
    else {
        panic!("expected authority")
    };
    authority.host_attach_contract_ref = Some(wrong_attach_ref.clone());
    authority.internal_resume_handle_refs = vec![wrong_resume_ref.clone()];
    wrong_graph.object_index.remove(&resume_ref.ref_id);
    wrong_graph
        .object_index
        .remove(&authority_attach_ref.ref_id);
    wrong_graph.object_index.insert(
        wrong_resume_ref.ref_id.clone(),
        index(&wrong_resume_ref, &wrong_resume_bytes),
    );
    wrong_graph.object_index.insert(
        wrong_attach_ref.ref_id.clone(),
        index(&wrong_attach_ref, &wrong_attach_bytes),
    );
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &wrong_graph,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&candidate)
            .unwrap(),
    )
    .unwrap();

    let terminal = TerminalHandoffHashInputV1 {
        schema_version: 1,
        intent_id: intent_id.into(),
        run_id: run_id.into(),
        payload_commitment: candidate.transition_intent_map[intent_id]
            .payload_commitment
            .clone(),
        terminal_state: TerminalHandoffStateV1::Expired,
        application_result_ref: None,
        input_acceptance_ref: None,
        post_turn_completion_ref: None,
        post_turn_application_result_ref: None,
        recorded_at: TimestampV1::parse("2026-07-11T12:55:11.000000000Z").unwrap(),
    };
    let terminal_bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&terminal)
            .unwrap();
    let terminal_ref = AuthorityObjectRefV1 {
        ref_id: "ao_99999999999999999999999999999999".into(),
        object_kind: AuthorityObjectKindV1::TerminalHandoff,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: canonical_sha256(&terminal).unwrap(),
        },
    };
    platform::publish_object_test(
        root.path(),
        &terminal_ref,
        &terminal_bytes,
        None,
        [0x84; 16],
    )
    .unwrap();
    let mut released = candidate.clone();
    released.root_revision += 1;
    let released_at = TimestampV1::parse("2026-07-11T12:55:12.000000000Z").unwrap();
    let intent = released.transition_intent_map.get_mut(intent_id).unwrap();
    intent.state = HostSessionTransitionIntentStateV1::Expired {
        terminal_handoff_ref: terminal_ref.clone(),
        expired_at: terminal.recorded_at.clone(),
    };
    intent.transport_payload_state = HostSessionTransitionTransportPayloadStateV1::Released {
        terminal_handoff_ref: terminal_ref.clone(),
        released_at: released_at.clone(),
    };
    released.session_namespace_map.insert(
        session_id.into(),
        SessionNamespaceRecordV1::StartTombstone(SessionIdTombstoneV1 {
            schema_version: 1,
            orchestration_session_id: session_id.into(),
            intent_id: intent_id.into(),
            issuer_request_id: request_id.into(),
            payload_commitment: intent.payload_commitment.clone(),
            terminal_state: StartTombstoneStateV1::Expired,
            terminal_handoff_ref: terminal_ref.clone(),
            tombstoned_at: terminal.recorded_at.clone(),
        }),
    );
    released.object_index.insert(
        terminal_ref.ref_id.clone(),
        index(&terminal_ref, &terminal_bytes),
    );
    released
        .object_index
        .get_mut(&valid_transport_ref.ref_id)
        .unwrap()
        .storage_state = AuthorityObjectStorageStateV1::Released {
        terminal_handoff_ref: terminal_ref.clone(),
        released_at,
    };
    released.validate().unwrap();
    let current_root_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    let transport_path = root.path().join(
        "authority-v1/objects/transition-transport-payload/v1/ao_66666666666666666666666666666666.obj",
    );
    assert!(transport_path.exists());
    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: candidate.root_revision,
            authority: None,
        },
        &released,
    )
    .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        current_root_bytes
    );
    assert!(transport_path.exists());

    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&released)
            .unwrap(),
    )
    .unwrap();
    assert!(transport_path.exists());
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::ValidExisting
    );
    assert!(!transport_path.exists());
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::ValidExisting
    );

    fs::write(&transport_path, &valid_transport_bytes).unwrap();
    fs::set_permissions(&transport_path, fs::Permissions::from_mode(0o600)).unwrap();
    let mut mismatched = released;
    let wrong_terminal = AuthorityObjectRefV1 {
        ref_id: "ao_98989898989898989898989898989898".into(),
        ..terminal_ref
    };
    mismatched
        .transition_intent_map
        .get_mut(intent_id)
        .unwrap()
        .transport_payload_state = HostSessionTransitionTransportPayloadStateV1::Released {
        terminal_handoff_ref: wrong_terminal.clone(),
        released_at: TimestampV1::parse("2026-07-11T12:55:12.000000000Z").unwrap(),
    };
    mismatched
        .object_index
        .get_mut(&valid_transport_ref.ref_id)
        .unwrap()
        .storage_state = AuthorityObjectStorageStateV1::Released {
        terminal_handoff_ref: wrong_terminal,
        released_at: TimestampV1::parse("2026-07-11T12:55:12.000000000Z").unwrap(),
    };
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &mismatched,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );
    assert!(transport_path.exists());
}

#[test]
fn resume_handle_identity_matches_every_parent_dimension() {
    let resume: crate::execution::agent_runtime::host_session_authority::schema::ResumeHandleHashInputV1 =
        crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(
            include_bytes!("testdata/resume-handle.json"),
        )
        .unwrap();
    assert!(platform::validate_resume_identity_test(
        &resume,
        "session-vector",
        "participant-vector",
        "codex",
        "codex-json",
    )
    .is_ok());
    for (session, participant, backend, protocol) in [
        ("other-session", "participant-vector", "codex", "codex-json"),
        ("session-vector", "other-participant", "codex", "codex-json"),
        (
            "session-vector",
            "participant-vector",
            "other",
            "codex-json",
        ),
        ("session-vector", "participant-vector", "codex", "other"),
    ] {
        assert!(platform::validate_resume_identity_test(
            &resume,
            session,
            participant,
            backend,
            protocol,
        )
        .is_err());
    }
}
#[test]
fn sensitive_orphans_require_exact_parent_context_and_key_cleanup_is_durable() {
    let root = root();
    let initial = material(0x62);
    let state = platform::bootstrap_test(root.path(), initial.clone(), None).unwrap();
    let context = ObjectVerificationContextV1 {
        intent_id: "intent-1".into(),
        run_id: "run-1".into(),
        parent_intent: None,
    };
    let raw = b"lease-token";
    let digest = crate::execution::agent_runtime::host_session_authority::hash::store_hmac_sha256(
            &initial.secret_key,
            crate::execution::agent_runtime::host_session_authority::hash::SensitiveDomainV1::ParticipantLeaseToken,
            &state.authority_store_id,
            &context.intent_id,
            Some(&context.run_id),
            raw,
        )
        .unwrap();
    let reference = crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1 {
            ref_id: "ao_66666666666666666666666666666666".into(),
            object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1::LeaseToken,
            schema_version: 1,
            commitment: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::StoreHmacSha256 {
                key_id: state.active_commitment_key_id.clone(),
                domain: "substrate.a1.participant-lease-token.v1".into(),
                digest_hex: digest,
            },
        };
    assert_eq!(
        platform::publish_object_test(root.path(), &reference, raw, Some(&context), [0x81; 16],)
            .unwrap(),
        ObjectPublicationOutcomeV1::PublishedOrphan
    );
    let wrong_context = ObjectVerificationContextV1 {
        intent_id: context.intent_id.clone(),
        run_id: "different-run".into(),
        parent_intent: None,
    };
    assert!(platform::publish_object_test(
        root.path(),
        &reference,
        raw,
        Some(&wrong_context),
        [0x82; 16],
    )
    .is_err());

    let orphan_key_id = "ak_99999999999999999999999999999999";
    let orphan_key = crate::execution::agent_runtime::host_session_authority::store_format::AuthorityStoreCommitmentKeyFileV1 {
            authority_store_id: state.authority_store_id.clone(),
            key_id: orphan_key_id.into(),
            created_at: initial.created_at,
            secret_key: [0x99; 32],
        };
    let orphan_key_path = root
        .path()
        .join(format!("authority-v1/keys/{orphan_key_id}.key"));
    fs::write(&orphan_key_path, orphan_key.encode().unwrap()).unwrap();
    fs::set_permissions(&orphan_key_path, fs::Permissions::from_mode(0o600)).unwrap();
    platform::bootstrap_test(root.path(), material(0x83), None).unwrap();
    assert!(!orphan_key_path.exists());
}

#[test]
fn indexed_objects_require_an_exact_parent_ref_and_commitment() {
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1,
        DurableSessionAuthorityOriginV1, HostSessionPostureV1, PolicyObjectHashInputV1,
        WorkspaceBindingV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::{
        AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
        SessionNamespaceRecordV1,
    };

    let root = root();
    let mut state = platform::bootstrap_test(root.path(), material(0x91), None).unwrap();
    let value = PolicyObjectHashInputV1 {
        schema_version: 1,
        policy_revision: "policy-1".into(),
        canonical_policy_snapshot_sha256:
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
    };
    let bytes =
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&value)
            .unwrap();
    let reference = AuthorityObjectRefV1 {
        ref_id: "ao_55555555555555555555555555555555".into(),
        object_kind: AuthorityObjectKindV1::Policy,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex:
                crate::execution::agent_runtime::host_session_authority::hash::canonical_sha256(
                    &value,
                )
                .unwrap(),
        },
    };
    platform::publish_object_test(root.path(), &reference, &bytes, None, [0x92; 16]).unwrap();

    let session_id = "session-1".to_string();
    state.session_namespace_map.insert(
        session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(DurableSessionAuthorityV1 {
            schema_version: 1,
            orchestration_session_id: session_id,
            shell_trace_session_id: "trace-1".into(),
            authority_revision: 1,
            origin: DurableSessionAuthorityOriginV1::StartIntent {
                intent_id: "intent-1".into(),
                issuer_request_id: "request-1".into(),
                payload_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                        .into(),
                },
            },
            authoritative_participant_lineage: vec!["participant-1".into()],
            active_authoritative_participant_id: Some("participant-1".into()),
            workspace_binding: WorkspaceBindingV1 {
                workspace_root: state.bootstrap_home.clone(),
                authority_store_root: state.bootstrap_home.clone(),
                authority_store_id: state.authority_store_id.clone(),
            },
            world_binding: None,
            host_attach_contract_ref: None,
            retained_worker_refs: Vec::new(),
            internal_resume_handle_refs: Vec::new(),
            lifecycle_posture: HostSessionPostureV1::ActiveAttached,
            current_policy_ref: Some(reference.clone()),
            current_policy_revision: Some("policy-1".into()),
            updated_at: material(0x91).created_at,
        })),
    );
    state.object_index.insert(
        reference.ref_id.clone(),
        AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: reference.ref_id.clone(),
            object_kind: reference.object_kind,
            object_schema_version: 1,
            byte_length: bytes.len() as u64,
            storage_state: AuthorityObjectStorageStateV1::Present,
        },
    );
    state.root_revision = 2;
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&state)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );
}

#[test]
fn classifier_rejects_unknown_layout_and_mismatched_pending_artifacts() {
    let unknown = root();
    assert_eq!(
        classify(unknown.path()),
        BootstrapClassificationV1::FreshAbsent
    );
    let extra = unknown.path().join("authority-v1/extra");
    fs::write(&extra, b"extra").unwrap();
    fs::set_permissions(&extra, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        classify(unknown.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );

    let malformed_key = root();
    let original = material(0xa1);
    assert!(platform::bootstrap_test(
        malformed_key.path(),
        original,
        Some(InitializationCrashPointV1::Marker),
    )
    .is_err());
    let key_path = malformed_key
        .path()
        .join("authority-v1/keys/ak_a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2.key");
    fs::write(&key_path, b"malformed").unwrap();
    fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        classify(malformed_key.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );

    let mismatched_marker = root();
    let state = platform::bootstrap_test(mismatched_marker.path(), material(0xb1), None).unwrap();
    let marker = crate::execution::agent_runtime::host_session_authority::store_schema::AuthorityStoreInitializationV1 {
            schema_version: 1,
            authority_store_id: state.authority_store_id,
            bootstrap_home: state.bootstrap_home,
            initial_key_id: "ak_eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
            created_at: material(0xb1).created_at,
        };
    let marker_path = mismatched_marker.path().join("authority-v1/init-v1.json");
    fs::write(
        &marker_path,
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&marker)
            .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&marker_path, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        classify(mismatched_marker.path()),
        BootstrapClassificationV1::CorruptOrUnsupported
    );
}

#[test]
fn first_lock_creation_race_joins_only_the_existing_durable_lock() {
    let root = root();
    platform::first_lock_creation_join_test(root.path()).unwrap();
}

#[test]
fn root_cas_commits_once_and_exact_retry_joins_without_mutation() {
    let root = root();
    let current = platform::bootstrap_test(root.path(), material(0xa1), None).unwrap();
    let initial_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: 0,
            authority: None,
        },
        &current,
    )
    .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        initial_bytes
    );
    let mut proposed = current.clone();
    proposed.root_revision += 1;

    let expectation = ExpectedRevisionsV1 {
        root_revision: current.root_revision,
        authority: None,
    };
    assert_eq!(
        compare_and_swap_root(root.path(), &expectation, &proposed).unwrap(),
        TransactionCommitOutcomeV1::Committed(proposed.clone())
    );
    let committed_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();

    assert_eq!(
        compare_and_swap_root(root.path(), &expectation, &proposed).unwrap(),
        TransactionCommitOutcomeV1::JoinedExact(proposed.clone())
    );
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        committed_bytes
    );
    assert!(fs::read_dir(root.path().join("authority-v1/tmp"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(read_root(root.path()).unwrap(), proposed);
}

#[test]
fn stale_and_conflicting_root_cas_reject_with_zero_mutation() {
    let root = root();
    let current = platform::bootstrap_test(root.path(), material(0xa2), None).unwrap();
    let mut committed = current.clone();
    committed.root_revision += 1;
    compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: current.root_revision,
            authority: None,
        },
        &committed,
    )
    .unwrap();

    let before = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    let mut conflicting = committed.clone();
    conflicting.root_revision = committed.root_revision + 1;
    conflicting.active_commitment_key_id = "ak_ffffffffffffffffffffffffffffffff".into();
    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: current.root_revision,
            authority: None,
        },
        &conflicting,
    )
    .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        before
    );
    assert!(fs::read_dir(root.path().join("authority-v1/tmp"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn initialization_marker_disables_legacy_writer_before_compatibility_mutation() {
    let root = root();
    assert!(platform::bootstrap_test(
        root.path(),
        material(0xa3),
        Some(InitializationCrashPointV1::Marker),
    )
    .is_err());
    let marker = fs::read(root.path().join("authority-v1/init-v1.json")).unwrap();

    assert!(legacy_writer_guard(root.path()).is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/init-v1.json")).unwrap(),
        marker
    );
    assert!(!root.path().join("run/agent-hub/sessions").exists());
    assert!(!root.path().join("run/agent-hub/participants").exists());
    assert!(fs::read_dir(root.path().join("authority-v1/tmp"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn conflicting_subprocess_cas_has_one_winner_and_exact_restart_join() {
    const CHILD_TEST: &str = "execution::agent_runtime::host_session_authority::store::tests::conflicting_subprocess_cas_has_one_winner_and_exact_restart_join";
    const CHILD_SENTINEL: &str = "A1_CAS_CHILD_EXECUTED";
    if let Some(root_path) = std::env::var_os("SUBSTRATE_A1_CAS_CHILD_ROOT") {
        println!("{CHILD_SENTINEL}");
        let expected = std::env::var("SUBSTRATE_A1_CAS_EXPECTED")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let proposed: StateRootV1 = canonical_json::from_slice(
            std::env::var("SUBSTRATE_A1_CAS_PROPOSED")
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        let outcome = compare_and_swap_root(
            std::path::Path::new(&root_path),
            &ExpectedRevisionsV1 {
                root_revision: expected,
                authority: None,
            },
            &proposed,
        );
        match outcome {
            Ok(TransactionCommitOutcomeV1::Committed(_)) => println!("committed"),
            Ok(TransactionCommitOutcomeV1::JoinedExact(_)) => println!("joined"),
            Err(_) => println!("rejected"),
        }
        return;
    }

    let root = root();
    let initial = platform::bootstrap_test(root.path(), material(0xb1), None).unwrap();
    let rotated = platform::rotate_commitment_key_test(root.path(), material(0xb2), None).unwrap();
    let former_key = initial.active_commitment_key_id;
    let current_key = rotated.active_commitment_key_id.clone();
    let mut first = rotated.clone();
    first.root_revision += 1;
    let mut second = first.clone();
    second.active_commitment_key_id = former_key.clone();
    second
        .commitment_key_registry
        .get_mut(&former_key)
        .unwrap()
        .state = AuthorityStoreCommitmentKeyStateV1::Active;
    second
        .commitment_key_registry
        .get_mut(&current_key)
        .unwrap()
        .state = AuthorityStoreCommitmentKeyStateV1::VerificationOnly;
    first.validate().unwrap();
    second.validate().unwrap();

    let spawn = |candidate: &StateRootV1| {
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture"])
            .env("SUBSTRATE_A1_CAS_CHILD_ROOT", root.path())
            .env(
                "SUBSTRATE_A1_CAS_EXPECTED",
                rotated.root_revision.to_string(),
            )
            .env(
                "SUBSTRATE_A1_CAS_PROPOSED",
                String::from_utf8(canonical_json::to_vec(candidate).unwrap()).unwrap(),
            )
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap()
    };
    let first_child = spawn(&first);
    let second_child = spawn(&second);
    let first_output = first_child.wait_with_output().unwrap();
    let second_output = second_child.wait_with_output().unwrap();
    assert!(first_output.status.success());
    assert!(second_output.status.success());
    let first_stdout = String::from_utf8(first_output.stdout).unwrap();
    let second_stdout = String::from_utf8(second_output.stdout).unwrap();
    assert!(first_stdout.contains(CHILD_SENTINEL));
    assert!(second_stdout.contains(CHILD_SENTINEL));
    assert_eq!(
        usize::from(first_stdout.contains("committed"))
            + usize::from(second_stdout.contains("committed")),
        1
    );
    assert_eq!(
        usize::from(first_stdout.contains("rejected"))
            + usize::from(second_stdout.contains("rejected")),
        1
    );

    let winner = if first_stdout.contains("committed") {
        &first
    } else {
        &second
    };
    let restart_output = spawn(winner).wait_with_output().unwrap();
    assert!(restart_output.status.success());
    let restart_stdout = String::from_utf8(restart_output.stdout).unwrap();
    assert!(restart_stdout.contains(CHILD_SENTINEL));
    assert!(restart_stdout.contains("joined"));
    assert_eq!(read_root(root.path()).unwrap(), *winner);
}

#[test]
fn legacy_owned_lock_is_released_when_holder_process_exits() {
    use std::io::{BufRead as _, Write as _};

    const CHILD_TEST: &str = "execution::agent_runtime::host_session_authority::store::tests::legacy_owned_lock_is_released_when_holder_process_exits";
    const CHILD_SENTINEL: &str = "A1_OWNED_LOCK_CHILD_HOLDING";
    if let Some(root_path) = std::env::var_os("SUBSTRATE_A1_OWNED_LOCK_CHILD_ROOT") {
        let _guard = legacy_writer_guard(std::path::Path::new(&root_path)).unwrap();
        println!("{CHILD_SENTINEL}");
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(60));
        return;
    }

    let root = root();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env("SUBSTRATE_A1_OWNED_LOCK_CHILD_ROOT", root.path())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = std::io::BufReader::new(child.stdout.take().unwrap());
    let mut output = String::new();
    for _ in 0..32 {
        let mut line = String::new();
        assert_ne!(stdout.read_line(&mut line).unwrap(), 0, "{output}");
        output.push_str(&line);
        if line.contains(CHILD_SENTINEL) {
            break;
        }
    }
    assert!(output.contains(CHILD_SENTINEL), "child output: {output}");
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());

    drop(legacy_writer_guard(root.path()).unwrap());
}

#[test]
fn post_root_legacy_insertion_blocks_cas_without_mutating_root() {
    let root = root();
    let current = platform::bootstrap_test(root.path(), material(0xb3), None).unwrap();
    let root_path = root.path().join("authority-v1/state-root-v1.json");
    let root_bytes = fs::read(&root_path).unwrap();
    let sessions = root.path().join("run/agent-hub/sessions");
    fs::create_dir_all(&sessions).unwrap();
    for directory in [
        root.path().join("run"),
        root.path().join("run/agent-hub"),
        sessions.clone(),
    ] {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::write(sessions.join("inserted.json"), b"legacy").unwrap();
    fs::set_permissions(
        sessions.join("inserted.json"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let mut proposed = current.clone();
    proposed.root_revision += 1;

    assert!(compare_and_swap_root(
        root.path(),
        &ExpectedRevisionsV1 {
            root_revision: current.root_revision,
            authority: None,
        },
        &proposed,
    )
    .is_err());
    assert_eq!(fs::read(root_path).unwrap(), root_bytes);
    assert!(sessions.join("inserted.json").exists());
}

#[test]
fn semantic_preflight_rejects_unknown_tree_key_object_root_and_temp_without_cleanup() {
    for case in ["tree", "key", "object", "root", "temp"] {
        let root = root();
        let current = platform::bootstrap_test(root.path(), material(0xc1), None).unwrap();
        let mut proposed = current.clone();
        proposed.root_revision += 1;
        let root_path = root.path().join("authority-v1/state-root-v1.json");
        let original_root = fs::read(&root_path).unwrap();
        let artifact = match case {
            "tree" => {
                let path = root.path().join("authority-v1/unknown");
                fs::write(&path, b"unknown").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                path
            }
            "key" => {
                let path = root.path().join("authority-v1/keys/not-a-key.key");
                fs::write(&path, b"unknown").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                path
            }
            "object" => {
                let path = root.path().join("authority-v1/objects/unknown-kind");
                fs::create_dir(&path).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
                path
            }
            "root" => {
                let mut malformed = original_root.clone();
                let insert = malformed.len() - 1;
                malformed.splice(insert..insert, b",\"unknown\":true".iter().copied());
                fs::write(&root_path, &malformed).unwrap();
                root_path.clone()
            }
            "temp" => {
                let path = root.path().join("authority-v1/tmp/not-a-temp");
                fs::write(&path, b"unknown").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                path
            }
            _ => unreachable!(),
        };
        let bytes_before = fs::read(&root_path).unwrap();

        assert!(read_root(root.path()).is_err(), "case {case}");
        assert_eq!(
            classify(root.path()),
            BootstrapClassificationV1::CorruptOrUnsupported,
            "case {case}"
        );
        assert!(bootstrap(root.path()).is_err(), "case {case}");
        assert!(
            rotate_commitment_key(root.path(), current.root_revision).is_err(),
            "case {case}"
        );
        assert!(
            compare_and_swap_root(
                root.path(),
                &ExpectedRevisionsV1 {
                    root_revision: current.root_revision,
                    authority: None,
                },
                &proposed,
            )
            .is_err(),
            "case {case}"
        );
        assert_eq!(fs::read(&root_path).unwrap(), bytes_before, "case {case}");
        assert!(artifact.exists(), "case {case}");
    }
}

#[test]
fn authority_preflight_reconciles_recognized_temp_before_reporting_corruption() {
    for case in ["tree", "key", "object", "root"] {
        let root = root();
        platform::bootstrap_test(root.path(), material(0xd1), None).unwrap();
        let root_path = root.path().join("authority-v1/state-root-v1.json");
        let artifact = match case {
            "tree" => {
                let path = root.path().join("authority-v1/unknown");
                fs::write(&path, b"unknown").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                path
            }
            "key" => {
                let path = root.path().join("authority-v1/keys/not-a-key.key");
                fs::write(&path, b"unknown").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                path
            }
            "object" => {
                let path = root.path().join("authority-v1/objects/unknown-kind");
                fs::create_dir(&path).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
                path
            }
            "root" => {
                let mut malformed = fs::read(&root_path).unwrap();
                let insert = malformed.len() - 1;
                malformed.splice(insert..insert, b",\"unknown\":true".iter().copied());
                fs::write(&root_path, &malformed).unwrap();
                root_path.clone()
            }
            _ => unreachable!(),
        };
        let artifact_bytes = artifact.is_file().then(|| fs::read(&artifact).unwrap());
        let recognized_temp = root
            .path()
            .join("authority-v1/tmp/root--r2--33333333333333333333333333333333.tmp");
        fs::write(&recognized_temp, b"recognized interrupted temp").unwrap();
        fs::set_permissions(&recognized_temp, fs::Permissions::from_mode(0o600)).unwrap();

        assert_eq!(
            classify(root.path()),
            BootstrapClassificationV1::CorruptOrUnsupported,
            "case {case}"
        );
        assert!(!recognized_temp.exists(), "case {case}");
        assert!(artifact.exists(), "case {case}");
        if let Some(bytes) = artifact_bytes {
            assert_eq!(fs::read(artifact).unwrap(), bytes, "case {case}");
        }
    }
}

#[test]
fn authority_preflight_reconciles_temp_before_missing_or_unsafe_strict_components() {
    use std::os::unix::fs::symlink;

    for case in [
        "missing-objects",
        "missing-keys",
        "unsafe-objects",
        "unsafe-keys",
    ] {
        let root = root();
        platform::bootstrap_test(root.path(), material(0xd2), None).unwrap();
        let component_name = if case.ends_with("objects") {
            "objects"
        } else {
            "keys"
        };
        let component = root.path().join("authority-v1").join(component_name);
        if case.starts_with("missing") {
            fs::remove_dir_all(&component).unwrap();
        } else {
            let retained = root
                .path()
                .join("authority-v1")
                .join(format!("{component_name}-retained"));
            fs::rename(&component, &retained).unwrap();
            symlink(&retained, &component).unwrap();
        }
        let recognized_temp = root
            .path()
            .join("authority-v1/tmp/root--r2--44444444444444444444444444444444.tmp");
        fs::write(&recognized_temp, b"recognized interrupted temp").unwrap();
        fs::set_permissions(&recognized_temp, fs::Permissions::from_mode(0o600)).unwrap();

        assert_eq!(
            classify(root.path()),
            BootstrapClassificationV1::CorruptOrUnsupported,
            "case {case}"
        );
        assert!(!recognized_temp.exists(), "case {case}");
        if case.starts_with("missing") {
            assert!(!component.exists(), "case {case} repaired the component");
        } else {
            assert!(
                fs::symlink_metadata(&component)
                    .unwrap()
                    .file_type()
                    .is_symlink(),
                "case {case} replaced the unsafe component"
            );
        }
        assert!(read_root(root.path()).is_err(), "case {case}");
    }
}

#[test]
#[serial_test::serial]
fn legacy_transaction_ignores_environment_and_cwd_after_admission() {
    let bootstrap = root();
    let unrelated = root();
    let original_cwd = std::env::current_dir().unwrap();
    let original_home = std::env::var_os("SUBSTRATE_HOME");
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();

    std::env::set_var("SUBSTRATE_HOME", unrelated.path());
    std::env::set_current_dir(unrelated.path()).unwrap();
    let outcome = transaction.write_file(
        LegacyStateStoreCollectionV1::Participants,
        &["retained-root.json"],
        br#"{"root":"retained"}"#,
        [0x11; 16],
    );
    std::env::set_current_dir(original_cwd).unwrap();
    if let Some(value) = original_home {
        std::env::set_var("SUBSTRATE_HOME", value);
    } else {
        std::env::remove_var("SUBSTRATE_HOME");
    }

    outcome.unwrap();
    transaction.finish().unwrap();
    assert_eq!(
        fs::read(
            bootstrap
                .path()
                .join("run/agent-hub/participants/retained-root.json"),
        )
        .unwrap(),
        br#"{"root":"retained"}"#
    );
    assert!(!unrelated.path().join("run").exists());
}

#[test]
fn legacy_transaction_rejects_lexical_root_replacement_without_touching_replacement() {
    let parent = root();
    let lexical_root = parent.path().join("bootstrap");
    fs::create_dir(&lexical_root).unwrap();
    fs::set_permissions(&lexical_root, fs::Permissions::from_mode(0o700)).unwrap();
    let mut transaction = begin_legacy_state_store_transaction(&lexical_root).unwrap();
    let retained_root = parent.path().join("bootstrap-retained");
    fs::rename(&lexical_root, &retained_root).unwrap();
    fs::create_dir(&lexical_root).unwrap();
    fs::set_permissions(&lexical_root, fs::Permissions::from_mode(0o700)).unwrap();

    assert!(transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["replacement.json"],
            b"replacement must remain untouched",
            [0x12; 16],
        )
        .is_err());
    assert!(transaction.finish().is_err());
    assert!(fs::read_dir(&lexical_root).unwrap().next().is_none());
    assert!(!retained_root
        .join("run/agent-hub/sessions/replacement.json")
        .exists());
}

#[test]
fn legacy_transaction_rejects_symlink_descendant_traversal() {
    use std::os::unix::fs::symlink;

    let bootstrap = root();
    let external = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    let run = bootstrap.path().join("run");
    fs::create_dir(&run).unwrap();
    fs::set_permissions(&run, fs::Permissions::from_mode(0o700)).unwrap();
    symlink(external.path(), run.join("agent-hub")).unwrap();

    assert!(transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["symlink.json"],
            b"must not escape",
            [0x13; 16],
        )
        .is_err());
    assert!(fs::read_dir(external.path()).unwrap().next().is_none());
}

#[test]
fn legacy_transaction_rejects_wrong_physical_root_identity() {
    let bootstrap = root();
    let other = root();
    let transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    let other_root = TrustedAuthorityRoot::open(other.path()).unwrap();

    assert!(transaction
        .verify_physical_root(other_root.identity())
        .is_err());
    transaction.finish().unwrap();
}

#[test]
fn legacy_transaction_holds_root_lock_until_final_sync_completion() {
    let bootstrap = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    let contender_root = TrustedAuthorityRoot::open(bootstrap.path()).unwrap();
    let contender_authority = contender_root
        .directory()
        .open_directory("authority-v1")
        .unwrap();
    let contender_lock_directory = contender_authority.open_directory("lock").unwrap();
    let contender_lock = contender_lock_directory.open_file("root.lock").unwrap();

    assert!(contender_lock.try_lock_exclusive().unwrap().is_none());
    transaction
        .write_file(
            LegacyStateStoreCollectionV1::Participants,
            &["lock-lifetime.json"],
            b"durable",
            [0x14; 16],
        )
        .unwrap();
    assert!(contender_lock.try_lock_exclusive().unwrap().is_none());

    transaction.finish().unwrap();
    drop(contender_lock.try_lock_exclusive().unwrap().unwrap());
}

#[test]
fn legacy_transaction_reads_and_removes_only_directory_relative_files() {
    let bootstrap = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["session-a", "snapshot.json"],
            b"snapshot",
            [0x15; 16],
        )
        .unwrap();

    assert_eq!(
        transaction
            .read_file(
                LegacyStateStoreCollectionV1::Sessions,
                &["session-a", "snapshot.json"],
            )
            .unwrap(),
        Some(b"snapshot".to_vec())
    );
    assert!(transaction
        .remove_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["session-a", "snapshot.json"],
        )
        .unwrap());
    assert_eq!(
        transaction
            .read_file(
                LegacyStateStoreCollectionV1::Sessions,
                &["session-a", "snapshot.json"],
            )
            .unwrap(),
        None
    );
    transaction.finish().unwrap();
}

#[test]
fn legacy_transaction_rejects_descendant_replacement_between_read_and_write() {
    let bootstrap = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["session-a", "snapshot.json"],
            b"original",
            [0x16; 16],
        )
        .unwrap();
    assert_eq!(
        transaction
            .read_file(
                LegacyStateStoreCollectionV1::Sessions,
                &["session-a", "snapshot.json"],
            )
            .unwrap(),
        Some(b"original".to_vec())
    );

    let sessions = bootstrap.path().join("run/agent-hub/sessions");
    let retained = sessions.join("session-a-retained");
    fs::rename(sessions.join("session-a"), &retained).unwrap();
    fs::create_dir(sessions.join("session-a")).unwrap();
    fs::set_permissions(
        sessions.join("session-a"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();

    assert!(transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["session-a", "snapshot.json"],
            b"replacement",
            [0x17; 16],
        )
        .is_err());
    assert!(fs::read_dir(sessions.join("session-a"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(
        fs::read(retained.join("snapshot.json")).unwrap(),
        b"original"
    );
}

#[test]
fn legacy_transaction_finish_rejects_descendant_replacement_after_write() {
    let bootstrap = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["session-a", "snapshot.json"],
            b"original",
            [0x18; 16],
        )
        .unwrap();

    let sessions = bootstrap.path().join("run/agent-hub/sessions");
    let retained = sessions.join("session-a-retained");
    fs::rename(sessions.join("session-a"), &retained).unwrap();
    fs::create_dir(sessions.join("session-a")).unwrap();
    fs::set_permissions(
        sessions.join("session-a"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();

    assert!(transaction.finish().is_err());
    assert!(fs::read_dir(sessions.join("session-a"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(
        fs::read(retained.join("snapshot.json")).unwrap(),
        b"original"
    );
}

#[test]
fn legacy_transaction_admission_rejects_classified_absence_replacement() {
    let bootstrap = root();
    assert!(
        legacy_transaction_admission_handoff_test(bootstrap.path(), || {
            fs::create_dir(bootstrap.path().join("run")).unwrap();
            fs::set_permissions(
                bootstrap.path().join("run"),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        })
        .is_err()
    );
}

#[test]
fn legacy_transaction_rejects_later_classified_missing_component_appearance() {
    let bootstrap = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    transaction.create_classified_run_directory_test().unwrap();
    let replacement = bootstrap.path().join("run/agent-hub");
    fs::create_dir(&replacement).unwrap();
    fs::set_permissions(&replacement, fs::Permissions::from_mode(0o700)).unwrap();

    assert!(transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["must-not-write.json"],
            b"replacement",
            [0x19; 16],
        )
        .is_err());
    assert!(fs::read_dir(replacement).unwrap().next().is_none());
}
