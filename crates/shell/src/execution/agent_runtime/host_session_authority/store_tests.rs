use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::*;
use crate::execution::agent_runtime::host_session_authority::canonical_json;
use crate::execution::agent_runtime::host_session_authority::hash::canonical_sha256;
use crate::execution::agent_runtime::host_session_authority::schema::{
    AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1,
    DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1,
    HostSessionAuthorityPreconditionV1, HostSessionPostureV1, HostSessionTransitionCallerKindV1,
    HostSessionTransitionCallerV1, HostSessionTransitionModeV1, PolicyObjectHashInputV1,
    WorkspaceBindingV1,
};
use crate::execution::agent_runtime::host_session_authority::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1,
    AuthorityStoreCommitmentAlgorithmV1, AuthorityStoreCommitmentKeyStateV1,
    AuthorityStoreCommitmentKeyV1, DurableSessionAuthorityV1, HostSessionPostTurnApplicationV1,
    HostSessionStartupOwnershipApplicationV1, HostSessionTransitionApplicationJournalV1,
    HostSessionTransitionApplicationJournalV2, HostSessionTransitionInputHandoffV1,
    HostSessionTransitionIntentStateV1, HostSessionTransitionIntentStateV2,
    HostSessionTransitionIntentV1, HostSessionTransitionIntentV2,
    HostSessionTransitionTransportPayloadStateV1, InitialTransitionApplicationJournalV1,
    IssuerRequestIndexEntryV1, SessionIdReservationV1, SessionNamespaceRecordV1, StateRootV2,
    VersionedStateRoot,
};
use crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot;

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires an explicitly owned private Root namespace and a real UID1000 publisher"]
fn test_existing_owner_storage_reopen_survives_root_service_principal() {
    use crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority;
    use crate::execution::agent_runtime::retained_worker_runtime::RetainedWorkerRuntime;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    const FILTER: &str = "execution::agent_runtime::host_session_authority::store::tests::test_existing_owner_storage_reopen_survives_root_service_principal";
    if let Some(base) = std::env::var_os("TASK20_NATIVE_STORAGE_CHILD") {
        assert_eq!(unsafe { libc::geteuid() }, 1000);
        let home = std::path::PathBuf::from(base).join(".substrate");
        if std::env::var("TASK20_NATIVE_STORAGE_MODE").unwrap() == "bootstrap" {
            fs::DirBuilder::new().mode(0o700).create(&home).unwrap();
            let authority =
                HostSessionAuthority::from_trusted_root(TrustedAuthorityRoot::open(&home).unwrap())
                    .unwrap();
            authority.bootstrap().unwrap();
        } else if std::env::var("TASK20_NATIVE_STORAGE_MODE").unwrap() == "registry" {
            let authority = HostSessionAuthority::open(&home).unwrap();
            initialize_owned_registry_for_test(&authority);
        } else if std::env::var("TASK20_NATIVE_STORAGE_MODE").unwrap() == "verify" {
            let authority = HostSessionAuthority::open(&home).unwrap();
            assert_eq!(
                RetainedWorkerRuntime
                    .read_existing_admission_record(&authority, "absent-session", "absent-worker")
                    .unwrap(),
                None
            );
        } else {
            let pending = home
                .join("authority-v1/retained-worker-admission-v1/tmp")
                .join(format!("admission-registry--{}.tmp", "a".repeat(32)));
            fs::write(&pending, b"deliberate native incomplete stage").unwrap();
            fs::set_permissions(pending, fs::Permissions::from_mode(0o600)).unwrap();
        }
        return;
    }
    assert_eq!(
        unsafe { libc::geteuid() },
        0,
        "private Root controller required"
    );
    let base = tempfile::tempdir().unwrap();
    let name = std::ffi::CString::new(base.path().as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
    let publish = |mode: &str| {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                FILTER,
                "--exact",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", base.path())
            .env("TMPDIR", base.path())
            .env("TASK20_NATIVE_STORAGE_CHILD", base.path())
            .env("TASK20_NATIVE_STORAGE_MODE", mode)
            .current_dir(base.path());
        unsafe {
            command.pre_exec(|| {
                if libc::setgroups(0, std::ptr::null()) != 0
                    || libc::setgid(1000) != 0
                    || libc::setuid(1000) != 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "native publisher failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        println!("actual native publisher {mode} wait={}", output.status);
    };
    publish("bootstrap");
    let home = base.path().join(".substrate");
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open_for_owner(&home, 1000).unwrap(),
    )
    .unwrap();
    let snapshot = || {
        let mut entries = std::collections::BTreeMap::new();
        let mut pending = vec![home.clone()];
        while let Some(path) = pending.pop() {
            let m = fs::symlink_metadata(&path).unwrap();
            let bytes = if m.is_file() {
                Some(fs::read(&path).unwrap())
            } else {
                None
            };
            entries.insert(
                path.clone(),
                (
                    m.dev(),
                    m.ino(),
                    m.uid(),
                    m.gid(),
                    m.mode(),
                    m.nlink(),
                    m.mtime(),
                    m.mtime_nsec(),
                    m.ctime(),
                    m.ctime_nsec(),
                    bytes,
                ),
            );
            if m.is_dir() {
                pending.extend(fs::read_dir(path).unwrap().map(|e| e.unwrap().path()));
            }
        }
        entries
    };
    let before = snapshot();
    assert_eq!(
        RetainedWorkerRuntime
            .read_existing_admission_record(&authority, "absent-session", "absent-worker")
            .expect("Root reader must retain authenticated developer ownership"),
        None
    );
    read_existing_accepted_work_authority_snapshot(&authority)
        .expect("held source read must retain authenticated developer ownership");
    let existing = retained_worker_admission_storage_existing_for_authority(&authority)
        .expect("existing lifecycle storage must retain the authenticated developer owner");
    assert_eq!(
        before,
        snapshot(),
        "existing lifecycle storage acquisition allocated or reconciled owner state"
    );
    drop(existing);
    assert!(
        retained_worker_admission_storage_for_authority(&authority).is_err(),
        "read permission must not grant native writer acquisition"
    );
    assert_eq!(
        before,
        snapshot(),
        "existing and absent reads changed durable storage"
    );
    publish("registry");
    let directory = home.join("authority-v1/retained-worker-admission-v1");
    let registry_path = directory.join("registry-v1.json");
    let original_registry = fs::read(&registry_path).unwrap();
    let key_before = snapshot()
        .into_iter()
        .filter(|(path, _)| path.starts_with(directory.join("keys")))
        .collect::<std::collections::BTreeMap<_, _>>();
    let existing = retained_worker_admission_storage_existing_for_authority(&authority).unwrap();
    let verify = || {
        authority
            .trusted_root()
            .revalidate()
            .map_err(|_| BootstrapError("Root unit source changed"))
    };
    existing
        .transaction_existing_control(&verify, |transaction| {
            assert_eq!(
                transaction.read_registry()?.as_deref(),
                Some(original_registry.as_slice())
            );
            transaction.replace_registry_with_source_guard(
                &format!("admission-registry--{}.tmp", "b".repeat(32)),
                &original_registry,
                &verify,
            )
        })
        .expect("Root must publish a lifecycle registry temp as its authenticated owner");
    let published = fs::metadata(&registry_path).unwrap();
    assert_eq!(published.uid(), 1000);
    assert_eq!(published.mode() & 0o7777, 0o600);
    assert_eq!(published.nlink(), 1);
    assert_eq!(fs::read(&registry_path).unwrap(), original_registry);
    let successful_inode = published.ino();
    let rejected_name = format!("admission-registry--{}.tmp", "c".repeat(32));
    let rejected_path = directory.join("tmp").join(&rejected_name);
    let changed_root = home.clone();
    let observed = std::rc::Rc::new(std::cell::Cell::new(false));
    let observed_hook = observed.clone();
    crate::execution::agent_runtime::retained_worker_runtime::set_ordinary_population_before_publication_hook(move || {
        let staged = fs::metadata(&rejected_path).unwrap();
        assert_eq!(staged.uid(), 1000);
        assert_eq!(staged.mode() & 0o7777, 0o600);
        observed_hook.set(true);
        fs::set_permissions(changed_root, fs::Permissions::from_mode(0o750)).unwrap();
    });
    let rejected = existing.transaction_existing_control(&verify, |transaction| {
        transaction.replace_registry_with_source_guard(&rejected_name, &original_registry, &verify)
    });
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        observed.get(),
        "actual Root staging did not reach the physical source invalidator"
    );
    assert!(rejected.is_err());
    assert_eq!(
        fs::metadata(&registry_path).unwrap().ino(),
        successful_inode
    );
    assert_eq!(fs::read(&registry_path).unwrap(), original_registry);
    assert_eq!(fs::read_dir(directory.join("tmp")).unwrap().count(), 0);
    assert_eq!(
        snapshot()
            .into_iter()
            .filter(|(path, _)| path.starts_with(directory.join("keys")))
            .collect::<std::collections::BTreeMap<_, _>>(),
        key_before
    );
    publish("verify");
    assert!(retained_worker_admission_storage_for_authority(&authority).is_err());
    publish("partial");
    let partial = snapshot();
    assert!(RetainedWorkerRuntime
        .read_existing_admission_record(&authority, "absent-session", "absent-worker")
        .is_err());
    assert_eq!(
        partial,
        snapshot(),
        "partial read initialized or repaired owner state"
    );
}

#[cfg(target_os = "linux")]
fn initialize_owned_registry_for_test(
    authority: &crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority,
) {
    use crate::execution::agent_runtime::host_session_authority::{
        schema::{
            AgentDescriptorV1, AgentExecutionScopeV1, HostAttachCapabilitiesV1,
            HostAttachExecutionClientStartV1, HostAttachLaunchKnobsV1, HostAttachModePreferenceV1,
            RuntimeBackendKindV1,
        },
        transition::{
            IssueHostSessionTransitionRequestV1, StartContractMaterialV1, TransitionIssueOutcomeV1,
        },
    };
    // Use the native issuer to establish strict versioned state. No runtime,
    // World, acceptance or execution fact is constructed by this storage fixture.
    let root = authority.bootstrap().unwrap();
    let request = IssueHostSessionTransitionRequestV1 {
        intent_id: "storage-component-intent".into(),
        issuer_request_id: "storage-component-issuer".into(),
        mode: HostSessionTransitionModeV1::Start,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
        orchestration_session_id: "storage-component-session".into(),
        shell_trace_session_id: "storage-component-trace".into(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::PublicCli,
            caller_participant_id: None,
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: None,
        target_authoritative_participant_id: "storage-component-participant".into(),
        target_participant_lease_token: b"storage-component-lease".to_vec(),
        run_id: "storage-component-run".into(),
        resulting_authoritative_lineage: vec!["storage-component-participant".into()],
        workspace_binding: WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home,
            authority_store_id: root.authority_store_id,
        },
        world_binding: None,
        start_contract: StartContractMaterialV1 {
            descriptor: AgentDescriptorV1 {
                schema_version: 1,
                agent_id: "codex".into(),
                backend_id: "cli:codex".into(),
                backend_kind: RuntimeBackendKindV1::Codex,
                protocol: "substrate.agent.session".into(),
                execution_scope: AgentExecutionScopeV1::Host,
                binary_path: std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            },
            policy: PolicyObjectHashInputV1 {
                schema_version: 1,
                policy_revision: "storage-component-policy".into(),
                canonical_policy_snapshot_sha256: "a".repeat(64),
            },
            capabilities: HostAttachCapabilitiesV1 {
                session_resume: true,
                session_fork: true,
                session_stop: true,
                status_snapshot: true,
                event_stream: true,
            },
            launch_knobs: HostAttachLaunchKnobsV1 {
                requested_execution_scope: AgentExecutionScopeV1::Host,
                host_execution_client_start: HostAttachExecutionClientStartV1::Defer,
                attach_mode_preference: HostAttachModePreferenceV1::ContinuityPreferred,
            },
        },
        transition_input: None,
    };
    let now =
        TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .unwrap();
    assert!(matches!(
        authority.issue_start_at(&request, now, 300).unwrap(),
        TransitionIssueOutcomeV1::Issued(_)
    ));
    crate::execution::agent_runtime::retained_worker_runtime::RetainedWorkerRuntime
        .initialize_admission_registry(authority)
        .unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn test_existing_registry_publication_cleans_failed_creation() {
    use crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority;
    const FILTER: &str = "execution::agent_runtime::host_session_authority::store::tests::test_existing_registry_publication_cleans_failed_creation";
    const CHILD: &str = "TASK20_ADMISSION_CREATE_UMASK_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([FILTER, "--exact", "--nocapture", "--test-threads=1"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "actual publication child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let home = root();
    let authority = HostSessionAuthority::open(home.path()).unwrap();
    authority.bootstrap().unwrap();
    initialize_owned_registry_for_test(&authority);
    let directory = home
        .path()
        .join("authority-v1/retained-worker-admission-v1");
    let original = fs::read(directory.join("registry-v1.json")).unwrap();
    let storage = retained_worker_admission_storage_existing_for_authority(&authority).unwrap();
    let verify = || {
        authority
            .trusted_root()
            .revalidate()
            .map_err(|_| BootstrapError("unit source changed"))
    };
    // Only this dedicated subprocess changes its process-wide umask.
    let previous = unsafe { libc::umask(0o777) };
    let result = storage.transaction_existing_control(&verify, |transaction| {
        transaction.replace_registry_with_source_guard(
            &format!("admission-registry--{}.tmp", "d".repeat(32)),
            &original,
            &verify,
        )
    });
    unsafe { libc::umask(previous) };
    assert!(
        result.is_err(),
        "exact0600 validation must refuse the restrictive creation"
    );
    assert_eq!(
        fs::read(directory.join("registry-v1.json")).unwrap(),
        original
    );
    assert_eq!(
        fs::read_dir(directory.join("tmp")).unwrap().count(),
        0,
        "failed temporary creation must dispose its own inode"
    );
}

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

fn placeholder_commitment() -> AuthorityObjectCommitmentV1 {
    AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "ab".repeat(32),
    }
}

fn authority_commitment(authority: &DurableSessionAuthorityV1) -> AuthorityObjectCommitmentV1 {
    AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&DurableSessionAuthorityHashInputV1 {
            schema_version: authority.schema_version,
            orchestration_session_id: authority.orchestration_session_id.clone(),
            shell_trace_session_id: authority.shell_trace_session_id.clone(),
            authority_revision: authority.authority_revision,
            origin: authority.origin.clone(),
            authoritative_participant_lineage: authority.authoritative_participant_lineage.clone(),
            active_authoritative_participant_id: authority
                .active_authoritative_participant_id
                .clone(),
            workspace_binding: authority.workspace_binding.clone(),
            world_binding: authority.world_binding.clone(),
            host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
            retained_worker_refs: authority.retained_worker_refs.clone(),
            internal_resume_handle_refs: authority.internal_resume_handle_refs.clone(),
            lifecycle_posture: authority.lifecycle_posture,
            current_policy_ref: authority.current_policy_ref.clone(),
            current_policy_revision: authority.current_policy_revision.clone(),
        })
        .unwrap(),
    }
}

fn placeholder_ref(ref_id: &str, object_kind: AuthorityObjectKindV1) -> AuthorityObjectRefV1 {
    AuthorityObjectRefV1 {
        ref_id: ref_id.into(),
        object_kind,
        schema_version: 1,
        commitment: placeholder_commitment(),
    }
}

fn placeholder_hmac_ref(
    ref_id: &str,
    object_kind: AuthorityObjectKindV1,
    key_id: &str,
    domain: &str,
) -> AuthorityObjectRefV1 {
    AuthorityObjectRefV1 {
        ref_id: ref_id.into(),
        object_kind,
        schema_version: 1,
        commitment: AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id: key_id.into(),
            domain: domain.into(),
            digest_hex: "ab".repeat(32),
        },
    }
}

fn placeholder_v1_intent(root: &StateRootV1) -> HostSessionTransitionIntentV1 {
    let timestamp = TimestampV1::parse("2026-07-11T12:49:11.000000000Z").unwrap();
    HostSessionTransitionIntentV1 {
        schema_version: 1,
        intent_id: "intent-upgrade-occupied".into(),
        issuer_request_id: "request-upgrade-occupied".into(),
        intent_revision: 1,
        mode: HostSessionTransitionModeV1::Start,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
        orchestration_session_id: "session-upgrade-occupied".into(),
        shell_trace_session_id: "trace-upgrade-occupied".into(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::PublicCli,
            caller_participant_id: None,
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: None,
        target_authoritative_participant_id: "participant-upgrade-occupied".into(),
        target_participant_lease_token_ref: placeholder_ref(
            "ao_11111111111111111111111111111111",
            AuthorityObjectKindV1::LeaseToken,
        ),
        run_id: "run-upgrade-occupied".into(),
        resulting_authoritative_lineage: vec!["participant-upgrade-occupied".into()],
        workspace_binding: WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home.clone(),
            authority_store_id: root.authority_store_id.clone(),
        },
        world_binding: None,
        descriptor_ref: placeholder_ref(
            "ao_22222222222222222222222222222222",
            AuthorityObjectKindV1::AgentDescriptor,
        ),
        host_attach_contract_ref: placeholder_ref(
            "ao_33333333333333333333333333333333",
            AuthorityObjectKindV1::HostAttachContract,
        ),
        resume_handle_ref: None,
        transition_input_ref: None,
        post_turn_disposition: None,
        transport_payload_ref: placeholder_ref(
            "ao_44444444444444444444444444444444",
            AuthorityObjectKindV1::TransitionTransportPayload,
        ),
        payload_commitment: placeholder_commitment(),
        issued_at: timestamp.clone(),
        expires_at: TimestampV1::parse("2026-07-11T12:54:11.000000000Z").unwrap(),
        state: HostSessionTransitionIntentStateV1::Issued,
        input_handoff: HostSessionTransitionInputHandoffV1::NotApplicable,
        transport_payload_state: HostSessionTransitionTransportPayloadStateV1::Retained,
        updated_at: timestamp,
    }
}

fn placeholder_v2_intent(root: &StateRootV2) -> HostSessionTransitionIntentV2 {
    let timestamp = TimestampV1::parse("2026-07-11T12:49:11.000000000Z").unwrap();
    HostSessionTransitionIntentV2 {
        schema_version: 2,
        intent_id: "intent-v2-start".into(),
        issuer_request_id: "request-v2-start".into(),
        intent_revision: 1,
        mode: HostSessionTransitionModeV1::Start,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
        orchestration_session_id: "session-v2-start".into(),
        shell_trace_session_id: "trace-v2-start".into(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::PublicCli,
            caller_participant_id: None,
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: None,
        target_authoritative_participant_id: "participant-v2-start".into(),
        target_participant_lease_token_ref: placeholder_hmac_ref(
            "ao_81111111111111111111111111111111",
            AuthorityObjectKindV1::LeaseToken,
            &root.active_commitment_key_id,
            "substrate.a1.participant-lease-token.v1",
        ),
        run_id: "run-v2-start".into(),
        resulting_authoritative_lineage: vec!["participant-v2-start".into()],
        workspace_binding: WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home.clone(),
            authority_store_id: root.authority_store_id.clone(),
        },
        world_binding: None,
        descriptor_ref: placeholder_ref(
            "ao_82222222222222222222222222222222",
            AuthorityObjectKindV1::AgentDescriptor,
        ),
        host_attach_contract_ref: placeholder_ref(
            "ao_83333333333333333333333333333333",
            AuthorityObjectKindV1::HostAttachContract,
        ),
        resume_handle_ref: None,
        transition_input_ref: None,
        post_turn_disposition: None,
        transport_payload_ref: placeholder_hmac_ref(
            "ao_84444444444444444444444444444444",
            AuthorityObjectKindV1::TransitionTransportPayload,
            &root.active_commitment_key_id,
            "substrate.a1.raw-transport-payload.v1",
        ),
        payload_commitment: placeholder_commitment(),
        issued_at: timestamp.clone(),
        expires_at: TimestampV1::parse("2026-07-11T12:54:11.000000000Z").unwrap(),
        state: HostSessionTransitionIntentStateV2::Issued,
        input_handoff: HostSessionTransitionInputHandoffV1::NotApplicable,
        transport_payload_state: HostSessionTransitionTransportPayloadStateV1::Retained,
        updated_at: timestamp,
    }
}

fn v2_with_issued_start(v1: &StateRootV1) -> StateRootV2 {
    let mut root = StateRootV2::try_from_greenfield_v1(v1).unwrap();
    let intent = placeholder_v2_intent(&root);
    root.session_namespace_map.insert(
        intent.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::StartReservation(SessionIdReservationV1 {
            schema_version: 1,
            orchestration_session_id: intent.orchestration_session_id.clone(),
            intent_id: intent.intent_id.clone(),
            issuer_request_id: intent.issuer_request_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            reserved_at: intent.issued_at.clone(),
        }),
    );
    root.issuer_request_index.insert(
        intent.issuer_request_id.clone(),
        IssuerRequestIndexEntryV1 {
            schema_version: 1,
            issuer_request_id: intent.issuer_request_id.clone(),
            orchestration_session_id: intent.orchestration_session_id.clone(),
            intent_id: intent.intent_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
        },
    );
    root.object_index.insert(
        intent.transport_payload_ref.ref_id.clone(),
        AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: intent.transport_payload_ref.ref_id.clone(),
            object_kind: AuthorityObjectKindV1::TransitionTransportPayload,
            object_schema_version: 1,
            byte_length: 1,
            storage_state: AuthorityObjectStorageStateV1::Present,
        },
    );
    root.transition_intent_map
        .insert(intent.intent_id.clone(), intent);
    root
}

fn apply_placeholder_v2_start(root: &mut StateRootV2) {
    let mut intent = root.transition_intent_map["intent-v2-start"].clone();
    let application_result_ref = placeholder_ref(
        "ao_85555555555555555555555555555555",
        AuthorityObjectKindV1::ApplicationResult,
    );
    let applied_at = TimestampV1::parse("2026-07-11T12:50:11.000000000Z").unwrap();
    intent.intent_revision = 3;
    intent.state = HostSessionTransitionIntentStateV2::Applied {
        claim_id: "claim-v2-start".into(),
        claimant_attempt_id: "attempt-v2-start".into(),
        authority_revision_before: None,
        authority_revision_after: 1,
        active_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        resulting_posture: HostSessionPostureV1::ActiveAttached,
        authority_record_commitment: placeholder_commitment(),
        application_result_ref: application_result_ref.clone(),
        startup_ownership: Box::new(HostSessionStartupOwnershipApplicationV1::Pending {
            expected_run_id: intent.run_id.clone(),
            expected_authority_revision: 1,
            expected_active_authoritative_participant_id: intent
                .target_authoritative_participant_id
                .clone(),
        }),
        post_turn: Box::new(HostSessionPostTurnApplicationV1::NotApplicable),
        applied_at: applied_at.clone(),
    };
    let authority = DurableSessionAuthorityV1 {
        schema_version: 1,
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        authority_revision: 1,
        origin: DurableSessionAuthorityOriginV1::StartIntent {
            intent_id: intent.intent_id.clone(),
            issuer_request_id: intent.issuer_request_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
        },
        authoritative_participant_lineage: intent.resulting_authoritative_lineage.clone(),
        active_authoritative_participant_id: Some(
            intent.target_authoritative_participant_id.clone(),
        ),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        host_attach_contract_ref: Some(intent.host_attach_contract_ref.clone()),
        retained_worker_refs: Vec::new(),
        internal_resume_handle_refs: Vec::new(),
        lifecycle_posture: HostSessionPostureV1::ActiveAttached,
        current_policy_ref: None,
        current_policy_revision: None,
        updated_at: applied_at.clone(),
    };
    let exact_authority_commitment = authority_commitment(&authority);
    let HostSessionTransitionIntentStateV2::Applied {
        authority_record_commitment,
        ..
    } = &mut intent.state
    else {
        panic!("fixture must remain applied")
    };
    *authority_record_commitment = exact_authority_commitment.clone();
    root.session_namespace_map.insert(
        intent.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(authority)),
    );
    root.application_journal.insert(
        intent.intent_id.clone(),
        HostSessionTransitionApplicationJournalV2 {
            schema_version: 2,
            intent_id: intent.intent_id.clone(),
            initial_application: InitialTransitionApplicationJournalV1 {
                authority_revision_before: None,
                authority_revision_after: 1,
                authority_record_commitment: exact_authority_commitment,
                application_result_ref: application_result_ref.clone(),
                applied_at,
            },
            startup_terminal_application: None,
            post_turn_application: None,
        },
    );
    root.object_index.insert(
        application_result_ref.ref_id.clone(),
        AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: application_result_ref.ref_id.clone(),
            object_kind: AuthorityObjectKindV1::ApplicationResult,
            object_schema_version: 1,
            byte_length: 1,
            storage_state: AuthorityObjectStorageStateV1::Present,
        },
    );
    root.transition_intent_map
        .insert(intent.intent_id.clone(), intent);
}

#[test]
fn strict_root_version_discrimination_preserves_v1_bytes_and_rejects_mixed_shapes() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x01), None).unwrap();
    let v1_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    let decoded_v1 = VersionedStateRoot::decode(&v1_bytes).unwrap();
    assert_eq!(decoded_v1, VersionedStateRoot::V1(v1.clone()));
    assert_eq!(decoded_v1.to_canonical_bytes().unwrap(), v1_bytes);

    let v2 = StateRootV2::try_from_greenfield_v1(&v1).unwrap();
    let v2_bytes = canonical_json::to_vec(&v2).unwrap();
    let decoded_v2 = VersionedStateRoot::decode(&v2_bytes).unwrap();
    assert_eq!(decoded_v2, VersionedStateRoot::V2(v2));
    assert_eq!(decoded_v2.to_canonical_bytes().unwrap(), v2_bytes);

    let mut v1_with_v2_fields: serde_json::Value = serde_json::from_slice(&v1_bytes).unwrap();
    let object = v1_with_v2_fields.as_object_mut().unwrap();
    object.insert(
        "retained_worker_registration_request_index".into(),
        serde_json::json!({}),
    );
    object.insert(
        "retained_worker_registration_journal".into(),
        serde_json::json!({}),
    );
    assert!(
        VersionedStateRoot::decode(&canonical_json::to_vec(&v1_with_v2_fields).unwrap()).is_err()
    );

    let mut v2_missing_field: serde_json::Value = serde_json::from_slice(&v2_bytes).unwrap();
    v2_missing_field
        .as_object_mut()
        .unwrap()
        .remove("retained_worker_registration_journal");
    assert!(
        VersionedStateRoot::decode(&canonical_json::to_vec(&v2_missing_field).unwrap()).is_err()
    );

    let mut unknown_version: serde_json::Value = serde_json::from_slice(&v1_bytes).unwrap();
    unknown_version["schema_version"] = serde_json::json!(3);
    assert!(
        VersionedStateRoot::decode(&canonical_json::to_vec(&unknown_version).unwrap()).is_err()
    );
}

#[test]
fn greenfield_v1_to_v2_conversion_rejects_each_occupied_semantic_map() {
    let root = root();
    let empty = platform::bootstrap_test(root.path(), material(0x02), None).unwrap();

    let mut occupied = empty.clone();
    occupied.session_namespace_map.insert(
        "session-upgrade-occupied".into(),
        SessionNamespaceRecordV1::StartReservation(SessionIdReservationV1 {
            schema_version: 1,
            orchestration_session_id: "session-upgrade-occupied".into(),
            intent_id: "intent-upgrade-occupied".into(),
            issuer_request_id: "request-upgrade-occupied".into(),
            payload_commitment: placeholder_commitment(),
            reserved_at: material(0x02).created_at,
        }),
    );
    assert!(StateRootV2::try_from_greenfield_v1(&occupied).is_err());

    let mut occupied = empty.clone();
    occupied.transition_intent_map.insert(
        "intent-upgrade-occupied".into(),
        placeholder_v1_intent(&empty),
    );
    assert!(StateRootV2::try_from_greenfield_v1(&occupied).is_err());

    let mut occupied = empty.clone();
    occupied.issuer_request_index.insert(
        "request-upgrade-occupied".into(),
        IssuerRequestIndexEntryV1 {
            schema_version: 1,
            issuer_request_id: "request-upgrade-occupied".into(),
            orchestration_session_id: "session-upgrade-occupied".into(),
            intent_id: "intent-upgrade-occupied".into(),
            payload_commitment: placeholder_commitment(),
        },
    );
    assert!(StateRootV2::try_from_greenfield_v1(&occupied).is_err());

    let mut occupied = empty.clone();
    occupied.application_journal.insert(
        "intent-upgrade-occupied".into(),
        HostSessionTransitionApplicationJournalV1 {
            schema_version: 1,
            intent_id: "intent-upgrade-occupied".into(),
            initial_application: InitialTransitionApplicationJournalV1 {
                authority_revision_before: None,
                authority_revision_after: 1,
                authority_record_commitment: placeholder_commitment(),
                application_result_ref: placeholder_ref(
                    "ao_55555555555555555555555555555555",
                    AuthorityObjectKindV1::ApplicationResult,
                ),
                applied_at: material(0x02).created_at,
            },
            post_turn_application: None,
        },
    );
    assert!(StateRootV2::try_from_greenfield_v1(&occupied).is_err());

    let mut occupied = empty;
    occupied.object_index.insert(
        "ao_66666666666666666666666666666666".into(),
        AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: "ao_66666666666666666666666666666666".into(),
            object_kind: AuthorityObjectKindV1::Policy,
            object_schema_version: 1,
            byte_length: 1,
            storage_state: AuthorityObjectStorageStateV1::Present,
        },
    );
    assert!(StateRootV2::try_from_greenfield_v1(&occupied).is_err());
}

#[test]
fn strict_v2_accepts_only_start_and_matching_versioned_relations() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x12), None).unwrap();
    let issued = v2_with_issued_start(&v1);
    issued.validate().unwrap();

    let mut non_start = issued.clone();
    non_start
        .transition_intent_map
        .get_mut("intent-v2-start")
        .unwrap()
        .mode = HostSessionTransitionModeV1::Attach;
    assert!(non_start.validate().is_err());

    let mut wrong_member_version = issued.clone();
    wrong_member_version
        .transition_intent_map
        .get_mut("intent-v2-start")
        .unwrap()
        .schema_version = 1;
    assert!(wrong_member_version.validate().is_err());

    let bytes = canonical_json::to_vec(&issued).unwrap();
    let mut syntax: serde_json::Value = canonical_json::from_slice(&bytes).unwrap();
    syntax["transition_intent_map"]["intent-v2-start"]["schema_version"] = serde_json::json!(1);
    assert!(VersionedStateRoot::decode(&canonical_json::to_vec(&syntax).unwrap()).is_ok());
    let VersionedStateRoot::V2(decoded) =
        VersionedStateRoot::decode(&canonical_json::to_vec(&syntax).unwrap()).unwrap()
    else {
        panic!("root discriminator must remain V2")
    };
    assert!(decoded.validate().is_err());
}

#[test]
fn strict_v2_applied_start_requires_claimant_pending_startup_and_no_post_turn() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x13), None).unwrap();
    let mut applied = v2_with_issued_start(&v1);
    apply_placeholder_v2_start(&mut applied);
    applied.validate().unwrap();

    let mut mismatched_startup = applied.clone();
    let HostSessionTransitionIntentStateV2::Applied {
        startup_ownership, ..
    } = &mut mismatched_startup
        .transition_intent_map
        .get_mut("intent-v2-start")
        .unwrap()
        .state
    else {
        panic!("fixture must remain applied")
    };
    **startup_ownership = HostSessionStartupOwnershipApplicationV1::Pending {
        expected_run_id: "substituted-run".into(),
        expected_authority_revision: 1,
        expected_active_authoritative_participant_id: "participant-v2-start".into(),
    };
    assert!(mismatched_startup.validate().is_err());

    let mut post_turn = applied.clone();
    let HostSessionTransitionIntentStateV2::Applied {
        post_turn: state, ..
    } = &mut post_turn
        .transition_intent_map
        .get_mut("intent-v2-start")
        .unwrap()
        .state
    else {
        panic!("fixture must remain applied")
    };
    **state = HostSessionPostTurnApplicationV1::Pending {
        expected_run_id: "run-v2-start".into(),
        expected_authority_revision: 1,
    };
    assert!(post_turn.validate().is_err());

    let mut premature_release = applied.clone();
    let terminal = placeholder_ref(
        "ao_86666666666666666666666666666666",
        AuthorityObjectKindV1::TerminalHandoff,
    );
    let intent = premature_release
        .transition_intent_map
        .get_mut("intent-v2-start")
        .unwrap();
    let transport_ref_id = intent.transport_payload_ref.ref_id.clone();
    intent.transport_payload_state =
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
            terminal_handoff_ref: terminal.clone(),
        };
    premature_release
        .object_index
        .get_mut(&transport_ref_id)
        .unwrap()
        .storage_state = AuthorityObjectStorageStateV1::ReleaseEligible {
        terminal_handoff_ref: terminal,
    };
    assert!(premature_release.validate().is_err());

    let mut syntax: serde_json::Value =
        canonical_json::from_slice(&canonical_json::to_vec(&applied).unwrap()).unwrap();
    syntax["transition_intent_map"]["intent-v2-start"]["state"]["value"]
        .as_object_mut()
        .unwrap()
        .remove("claimant_attempt_id");
    assert!(VersionedStateRoot::decode(&canonical_json::to_vec(&syntax).unwrap()).is_err());
}

#[test]
fn strict_v2_reachability_dispatches_through_v2_start_state() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x14), None).unwrap();
    let issued = v2_with_issued_start(&v1);
    let issued_refs = platform::reachable_v2_ref_ids_test(&issued).unwrap();
    assert_eq!(issued_refs.len(), 4);
    assert!(!issued_refs.contains(&"ao_85555555555555555555555555555555".to_string()));

    let mut applied = issued;
    apply_placeholder_v2_start(&mut applied);
    let applied_refs = platform::reachable_v2_ref_ids_test(&applied).unwrap();
    assert_eq!(applied_refs.len(), 5);
    assert!(applied_refs.contains(&"ao_85555555555555555555555555555555".to_string()));
}

#[test]
fn strict_v2_key_lifecycle_preserves_root_version_and_semantic_maps() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x15), None).unwrap();
    let old_key_id = v1.active_commitment_key_id.clone();
    let RootUpgradeOutcomeV1::Upgraded(v2) =
        platform::upgrade_greenfield_root_test(root.path(), [0x71; 16], None).unwrap()
    else {
        panic!("fixture must publish strict V2")
    };
    let VersionedStateRoot::V2(rotated) =
        platform::rotate_commitment_key_versioned_test(root.path(), material(0x16), None).unwrap()
    else {
        panic!("rotation must preserve V2")
    };
    assert_eq!(rotated.schema_version, 2);
    assert_eq!(rotated.root_revision, v2.root_revision + 1);
    assert_eq!(rotated.session_namespace_map, v2.session_namespace_map);
    assert_eq!(rotated.transition_intent_map, v2.transition_intent_map);
    assert_eq!(rotated.application_journal, v2.application_journal);

    let VersionedStateRoot::V2(retired) =
        platform::retire_commitment_key_versioned_test(root.path(), &old_key_id, [0x72; 16], None)
            .unwrap()
    else {
        panic!("retirement must preserve V2")
    };
    assert_eq!(retired.schema_version, 2);
    assert_eq!(retired.root_revision, rotated.root_revision + 1);
    assert_eq!(
        retired.commitment_key_registry[&old_key_id].state,
        AuthorityStoreCommitmentKeyStateV1::Retired
    );
    assert_eq!(retired.session_namespace_map, v2.session_namespace_map);
    assert_eq!(retired.transition_intent_map, v2.transition_intent_map);
    assert_eq!(retired.application_journal, v2.application_journal);
}

#[test]
fn strict_v2_rejects_v1_only_key_lifecycle_without_mutation() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x17), None).unwrap();
    let old_key_id = v1.active_commitment_key_id.clone();
    let RootUpgradeOutcomeV1::Upgraded(v2) =
        platform::upgrade_greenfield_root_test(root.path(), [0x73; 16], None).unwrap()
    else {
        panic!("fixture must publish strict V2")
    };
    let before_root = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    let before_keys = fs::read_dir(root.path().join("authority-v1/keys"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();

    let rotation_error = rotate_commitment_key(root.path(), v2.root_revision).unwrap_err();
    assert_eq!(
        rotation_error.to_string(),
        "V1 key rotation caller encountered StateRootV2"
    );
    let retirement_error =
        retire_commitment_key(root.path(), &old_key_id, v2.root_revision).unwrap_err();
    assert_eq!(
        retirement_error.to_string(),
        "V1 key retirement caller encountered StateRootV2"
    );
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        before_root
    );
    assert_eq!(
        fs::read_dir(root.path().join("authority-v1/keys"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
        before_keys
    );
}

#[test]
fn strict_v2_applied_start_accepts_monotonic_revision_after_reclaim() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x18), None).unwrap();
    let mut v2 = v2_with_issued_start(&v1);
    apply_placeholder_v2_start(&mut v2);
    v2.transition_intent_map
        .get_mut("intent-v2-start")
        .unwrap()
        .intent_revision = 4;

    v2.validate().unwrap();
}

#[test]
fn greenfield_upgrade_is_atomic_crash_recoverable_and_exactly_retryable() {
    let before_publication = root();
    let v1 = platform::bootstrap_test(before_publication.path(), material(0x03), None).unwrap();
    let v1_bytes = fs::read(
        before_publication
            .path()
            .join("authority-v1/state-root-v1.json"),
    )
    .unwrap();
    assert!(platform::upgrade_greenfield_root_test(
        before_publication.path(),
        [0x31; 16],
        Some(GreenfieldUpgradeCrashPointV1::BeforeRootPublication),
    )
    .is_err());
    assert_eq!(
        fs::read(
            before_publication
                .path()
                .join("authority-v1/state-root-v1.json")
        )
        .unwrap(),
        v1_bytes
    );
    let upgraded =
        platform::upgrade_greenfield_root_test(before_publication.path(), [0x32; 16], None)
            .unwrap();
    let RootUpgradeOutcomeV1::Upgraded(v2) = upgraded else {
        panic!("first complete upgrade must publish V2")
    };
    assert_eq!(v2.schema_version, 2);
    assert_eq!(v2.root_revision, v1.root_revision + 1);
    assert!(v2.retained_worker_registration_request_index.is_empty());
    assert!(v2.retained_worker_registration_journal.is_empty());

    let committed_bytes = fs::read(
        before_publication
            .path()
            .join("authority-v1/state-root-v1.json"),
    )
    .unwrap();
    assert_eq!(
        platform::upgrade_greenfield_root_test(before_publication.path(), [0x33; 16], None,)
            .unwrap(),
        RootUpgradeOutcomeV1::JoinedExact(v2.clone())
    );
    assert_eq!(
        fs::read(
            before_publication
                .path()
                .join("authority-v1/state-root-v1.json")
        )
        .unwrap(),
        committed_bytes
    );

    let after_publication = root();
    platform::bootstrap_test(after_publication.path(), material(0x04), None).unwrap();
    assert!(platform::upgrade_greenfield_root_test(
        after_publication.path(),
        [0x41; 16],
        Some(GreenfieldUpgradeCrashPointV1::AfterRootPublication),
    )
    .is_err());
    let published = VersionedStateRoot::decode(
        &fs::read(
            after_publication
                .path()
                .join("authority-v1/state-root-v1.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let VersionedStateRoot::V2(published) = published else {
        panic!("post-publication crash must leave exact V2")
    };
    assert_eq!(
        platform::upgrade_greenfield_root_test(after_publication.path(), [0x42; 16], None).unwrap(),
        RootUpgradeOutcomeV1::JoinedExact(published)
    );

    let rejected = root();
    platform::bootstrap_test(rejected.path(), material(0x43), None).unwrap();
    let root_path = rejected.path().join("authority-v1/state-root-v1.json");
    let before = fs::read(&root_path).unwrap();
    let error = platform::upgrade_greenfield_root_test(
        rejected.path(),
        [0x44; 16],
        Some(GreenfieldUpgradeCrashPointV1::FinalRevalidationMismatch),
    )
    .unwrap_err();
    assert_eq!(error.to_string(), "UnsupportedNonGreenfieldRootV1");
    assert_eq!(fs::read(root_path).unwrap(), before);
    assert_eq!(
        fs::read_dir(rejected.path().join("authority-v1/tmp"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn strict_v3_upgrade_preserves_v2_maps_and_joins_exact_retry_only() {
    let root = root();
    platform::bootstrap_test(root.path(), material(0x45), None).unwrap();
    let RootUpgradeOutcomeV1::Upgraded(v2) =
        platform::upgrade_greenfield_root_test(root.path(), [0x46; 16], None).unwrap()
    else {
        panic!("fixture must publish strict V2")
    };

    let upgraded = platform::upgrade_v2_root_to_v3_test(root.path(), &v2).unwrap();
    assert_eq!(upgraded.schema_version, 3);
    assert_eq!(upgraded.root_revision, v2.root_revision + 1);
    assert_eq!(upgraded.authority_store_id, v2.authority_store_id);
    assert_eq!(upgraded.bootstrap_home, v2.bootstrap_home);
    assert_eq!(
        upgraded.greenfield_namespace_certificate,
        v2.greenfield_namespace_certificate
    );
    assert_eq!(upgraded.session_namespace_map, v2.session_namespace_map);
    assert_eq!(upgraded.transition_intent_map, v2.transition_intent_map);
    assert_eq!(upgraded.issuer_request_index, v2.issuer_request_index);
    assert_eq!(upgraded.application_journal, v2.application_journal);
    assert_eq!(
        upgraded.retained_worker_registration_request_index,
        v2.retained_worker_registration_request_index
    );
    assert_eq!(
        upgraded.retained_worker_registration_journal,
        v2.retained_worker_registration_journal
    );
    assert_eq!(upgraded.object_index, v2.object_index);
    assert!(upgraded.successor_transition_intent_map.is_empty());
    assert!(upgraded.successor_issuer_request_index.is_empty());
    assert!(upgraded.successor_application_journal.is_empty());

    let mut preserved_expected = v2.clone();
    preserved_expected.root_revision = upgraded.root_revision;
    assert_eq!(upgraded.preserved_v2_view(), preserved_expected);

    let VersionedStateRoot::V3(decoded) = VersionedStateRoot::decode(
        &fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
    )
    .unwrap() else {
        panic!("published V3 root must decode through the V3 discriminator")
    };
    assert_eq!(decoded, upgraded);
    assert_eq!(
        platform::upgrade_v2_root_to_v3_test(root.path(), &v2).unwrap(),
        upgraded
    );

    let mut stale = v2.clone();
    stale.root_revision += 1;
    let error = platform::upgrade_v2_root_to_v3_test(root.path(), &stale).unwrap_err();
    assert_eq!(
        error.to_string(),
        "current authority root does not match the exact V3 retry candidate"
    );

    let reopened = TrustedAuthorityRoot::open(root.path()).unwrap();
    let VersionedStateRoot::V3(reopened_root) = read_opened_root_v2_or_v3(&reopened).unwrap()
    else {
        panic!("reopened exact root must remain V3")
    };
    assert_eq!(reopened_root, upgraded);
}

#[test]
fn strict_v3_validation_rejects_no_successor_runtime_authority_drift() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x47), None).unwrap();
    let mut v2 = v2_with_issued_start(&v1);
    apply_placeholder_v2_start(&mut v2);

    let mut upgraded = crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV3::try_from_v2(&v2).unwrap();
    let authority_session_id = upgraded
        .session_namespace_map
        .iter()
        .find_map(|(session_id, record)| {
            matches!(record, SessionNamespaceRecordV1::Authority(_)).then(|| session_id.clone())
        })
        .unwrap();
    let SessionNamespaceRecordV1::Authority(authority) = upgraded
        .session_namespace_map
        .get_mut(&authority_session_id)
        .unwrap()
    else {
        panic!("upgraded root must retain the V2 start authority")
    };
    authority.authority_revision += 1;
    authority.lifecycle_posture = HostSessionPostureV1::AwaitingAttention;

    assert_eq!(
        upgraded.validate().unwrap_err().to_string(),
        "current V3 authority is not the exact reconstructed V2 runtime state"
    );
}

#[test]
fn greenfield_upgrade_rejects_orphans_without_any_mutation() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x05), None).unwrap();
    let value = PolicyObjectHashInputV1 {
        schema_version: 1,
        policy_revision: "policy-upgrade-orphan".into(),
        canonical_policy_snapshot_sha256: "cd".repeat(32),
    };
    let bytes = canonical_json::to_vec(&value).unwrap();
    let reference = AuthorityObjectRefV1 {
        ref_id: "ao_77777777777777777777777777777777".into(),
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
    platform::publish_object_test(root.path(), &reference, &bytes, None, [0x51; 16]).unwrap();
    let root_path = root.path().join("authority-v1/state-root-v1.json");
    let before = fs::read(&root_path).unwrap();
    let object_path = root
        .path()
        .join("authority-v1/objects/policy/v1/ao_77777777777777777777777777777777.obj");
    let error = platform::upgrade_greenfield_root_test(root.path(), [0x52; 16], None).unwrap_err();
    assert_eq!(error.to_string(), "UnsupportedNonGreenfieldRootV1");
    assert_eq!(fs::read(root_path).unwrap(), before);
    assert_eq!(fs::read(object_path).unwrap(), bytes);
    assert_eq!(before, canonical_json::to_vec(&v1).unwrap());
}

#[test]
fn reserved_ref_id_vacancy_rejects_v3_only_orphan_kinds() {
    let root = root();
    platform::bootstrap_test(root.path(), material(0x54), None).unwrap();
    platform::upgrade_greenfield_root_test(root.path(), [0x55; 16], None).unwrap();
    let ref_id = "ao_88888888888888888888888888888888";
    for kind_directory in [
        "startup-ownership-result",
        "obligation-snapshot",
        "post-turn-protocol-event",
    ] {
        let directory = root
            .path()
            .join(format!("authority-v1/objects/{kind_directory}/v1"));
        fs::create_dir_all(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
        let orphan = directory.join(format!("{ref_id}.obj"));
        fs::write(&orphan, b"orphan").unwrap();
        fs::set_permissions(&orphan, fs::Permissions::from_mode(0o600)).unwrap();
        let error = platform::reserved_object_ref_id_is_globally_absent_test(root.path(), ref_id)
            .expect_err("a V3-only orphan must fail strict V2 preflight closed");
        assert_eq!(error.to_string(), "read existing versioned authority root");
        fs::remove_file(orphan).unwrap();
    }
}

#[test]
fn greenfield_upgrade_rejects_legacy_artifacts_without_any_mutation() {
    let root = root();
    let v1 = platform::bootstrap_test(root.path(), material(0x53), None).unwrap();
    let root_path = root.path().join("authority-v1/state-root-v1.json");
    let before = fs::read(&root_path).unwrap();
    let sessions = root.path().join("run/agent-hub/sessions");
    fs::create_dir_all(&sessions).unwrap();
    for directory in [
        root.path().join("run"),
        root.path().join("run/agent-hub"),
        sessions.clone(),
    ] {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let artifact = sessions.join("legacy.json");
    fs::write(&artifact, b"legacy").unwrap();
    fs::set_permissions(&artifact, fs::Permissions::from_mode(0o600)).unwrap();
    let preserved_temp = root
        .path()
        .join("authority-v1/tmp/root--r2--54545454545454545454545454545454.tmp");
    fs::write(&preserved_temp, b"unpublished").unwrap();
    fs::set_permissions(&preserved_temp, fs::Permissions::from_mode(0o600)).unwrap();

    let error = platform::upgrade_greenfield_root_test(root.path(), [0x54; 16], None).unwrap_err();
    assert_eq!(error.to_string(), "UnsupportedNonGreenfieldRootV1");
    assert_eq!(fs::read(root_path).unwrap(), before);
    assert_eq!(fs::read(artifact).unwrap(), b"legacy");
    assert_eq!(fs::read(preserved_temp).unwrap(), b"unpublished");
    assert_eq!(before, canonical_json::to_vec(&v1).unwrap());
}

#[test]
fn greenfield_upgrade_classifies_unsafe_roots_without_reconciling_temps() {
    let root = root();
    platform::bootstrap_test(root.path(), material(0x55), None).unwrap();
    let root_path = root.path().join("authority-v1/state-root-v1.json");
    let mut syntax: serde_json::Value =
        canonical_json::from_slice(&fs::read(&root_path).unwrap()).unwrap();
    syntax
        .as_object_mut()
        .unwrap()
        .insert("unexpected_v2_field".into(), serde_json::json!({}));
    let invalid_bytes = canonical_json::to_vec(&syntax).unwrap();
    fs::write(&root_path, &invalid_bytes).unwrap();
    let preserved_temp = root
        .path()
        .join("authority-v1/tmp/root--r2--55555555555555555555555555555555.tmp");
    fs::write(&preserved_temp, b"unpublished").unwrap();
    fs::set_permissions(&preserved_temp, fs::Permissions::from_mode(0o600)).unwrap();

    let error = platform::upgrade_greenfield_root_test(root.path(), [0x56; 16], None).unwrap_err();
    assert_eq!(error.to_string(), "UnsupportedNonGreenfieldRootV1");
    assert_eq!(fs::read(root_path).unwrap(), invalid_bytes);
    assert_eq!(fs::read(preserved_temp).unwrap(), b"unpublished");
}

#[test]
fn greenfield_upgrade_rejects_absent_store_without_creating_scaffold() {
    let root = root();
    let error = platform::upgrade_greenfield_root_test(root.path(), [0x57; 16], None).unwrap_err();
    assert_eq!(error.to_string(), "UnsupportedNonGreenfieldRootV1");
    assert!(!root.path().join("authority-v1").exists());
}

#[test]
fn greenfield_upgrade_preserves_retired_key_registry_and_rejects_unknown_v2_routes() {
    let root = root();
    let initial = platform::bootstrap_test(root.path(), material(0x06), None).unwrap();
    let retired_key_id = initial.active_commitment_key_id.clone();
    let rotated = platform::rotate_commitment_key_test(root.path(), material(0x07), None).unwrap();
    let retired =
        platform::retire_commitment_key_test(root.path(), &retired_key_id, [0x61; 16], None)
            .unwrap();
    assert_eq!(
        retired.commitment_key_registry[&retired_key_id].state,
        AuthorityStoreCommitmentKeyStateV1::Retired
    );
    assert!(!root
        .path()
        .join(format!("authority-v1/keys/{retired_key_id}.key"))
        .exists());

    let RootUpgradeOutcomeV1::Upgraded(v2) =
        platform::upgrade_greenfield_root_test(root.path(), [0x62; 16], None).unwrap()
    else {
        panic!("eligible retired-key V1 root must upgrade")
    };
    assert_eq!(v2.commitment_key_registry, retired.commitment_key_registry);
    assert_eq!(
        v2.active_commitment_key_id,
        rotated.active_commitment_key_id
    );

    let unknown = root.path().join("authority-v1/objects/unknown-empty-kind");
    fs::create_dir(&unknown).unwrap();
    fs::set_permissions(&unknown, fs::Permissions::from_mode(0o700)).unwrap();
    let before = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    assert!(platform::upgrade_greenfield_root_test(root.path(), [0x63; 16], None).is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        before
    );
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
        Some(KeyLifecycleCrashPointV1::Reconciled),
    )
    .is_err());
    assert_eq!(read_root(root.path()).unwrap(), initial);

    assert!(platform::rotate_commitment_key_test(
        root.path(),
        material(0x53),
        Some(KeyLifecycleCrashPointV1::KeyPublished),
    )
    .is_err());
    let recovered = platform::bootstrap_test(root.path(), material(0x54), None).unwrap();
    assert_eq!(recovered, initial);
    assert_eq!(
        fs::read_dir(root.path().join("authority-v1/keys"))
            .unwrap()
            .count(),
        1
    );

    let rotated = platform::rotate_commitment_key_test(root.path(), material(0x55), None).unwrap();
    assert_ne!(rotated.active_commitment_key_id, initial_key);
    assert_eq!(
        rotated.commitment_key_registry[&initial_key].state,
        AuthorityStoreCommitmentKeyStateV1::VerificationOnly
    );
    assert_eq!(rotated.root_revision, initial.root_revision + 1);

    assert!(platform::retire_commitment_key_test(
        root.path(),
        &initial_key,
        [0x56; 16],
        Some(KeyLifecycleCrashPointV1::Reconciled),
    )
    .is_err());
    assert_eq!(read_root(root.path()).unwrap(), rotated);

    assert!(platform::retire_commitment_key_test(
        root.path(),
        &initial_key,
        [0x57; 16],
        Some(KeyLifecycleCrashPointV1::RootPublished),
    )
    .is_err());
    assert!(root
        .path()
        .join(format!("authority-v1/keys/{initial_key}.key"))
        .exists());
    let retired = platform::bootstrap_test(root.path(), material(0x58), None).unwrap();
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
fn key_lifecycle_revalidates_invalid_candidates_after_reconciliation() {
    let rotation = root();
    let initial = platform::bootstrap_test(rotation.path(), material(0x59), None).unwrap();
    let root_path = rotation.path().join("authority-v1/state-root-v1.json");
    let root_bytes = fs::read(&root_path).unwrap();
    let temp = rotation.path().join(
        "authority-v1/tmp/key--ak_61616161616161616161616161616161--62626262626262626262626262626262.tmp",
    );
    fs::write(&temp, b"partial").unwrap();
    fs::set_permissions(&temp, fs::Permissions::from_mode(0o600)).unwrap();
    let mut invalid_rotation = initial.clone();
    invalid_rotation.root_revision += 1;
    invalid_rotation.active_commitment_key_id = "ak_63636363636363636363636363636363".into();
    assert!(platform::publish_key_lifecycle_candidate_test(
        rotation.path(),
        initial.root_revision,
        &invalid_rotation,
        [0x68; 16],
    )
    .is_err());
    assert!(!temp.exists(), "recognized temp must reconcile first");
    assert_eq!(fs::read(&root_path).unwrap(), root_bytes);
    let failed_rotation_temp = rotation
        .path()
        .join("authority-v1/tmp/root--r2--68686868686868686868686868686868.tmp");
    assert!(!failed_rotation_temp.exists());
    assert_eq!(
        platform::bootstrap_test(rotation.path(), material(0x69), None).unwrap(),
        initial,
    );
    assert!(!failed_rotation_temp.exists());

    let retirement = root();
    let first = platform::bootstrap_test(retirement.path(), material(0x64), None).unwrap();
    let rotated =
        platform::rotate_commitment_key_test(retirement.path(), material(0x65), None).unwrap();
    let root_path = retirement.path().join("authority-v1/state-root-v1.json");
    let root_bytes = fs::read(&root_path).unwrap();
    let keys_before = fs::read_dir(retirement.path().join("authority-v1/keys"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    let temp = retirement.path().join(
        "authority-v1/tmp/key--ak_66666666666666666666666666666666--67676767676767676767676767676767.tmp",
    );
    fs::write(&temp, b"partial").unwrap();
    fs::set_permissions(&temp, fs::Permissions::from_mode(0o600)).unwrap();
    let mut invalid_retirement = rotated.clone();
    invalid_retirement.root_revision += 1;
    invalid_retirement
        .commitment_key_registry
        .get_mut(&first.active_commitment_key_id)
        .unwrap()
        .state = AuthorityStoreCommitmentKeyStateV1::Retired;
    let active_key = invalid_retirement.active_commitment_key_id.clone();
    invalid_retirement
        .commitment_key_registry
        .remove(&active_key);
    assert!(platform::publish_key_lifecycle_candidate_test(
        retirement.path(),
        rotated.root_revision,
        &invalid_retirement,
        [0x6a; 16],
    )
    .is_err());
    assert!(!temp.exists(), "recognized temp must reconcile first");
    assert_eq!(fs::read(&root_path).unwrap(), root_bytes);
    let failed_retirement_temp = retirement
        .path()
        .join("authority-v1/tmp/root--r3--6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a6a.tmp");
    assert!(!failed_retirement_temp.exists());
    assert_eq!(
        fs::read_dir(retirement.path().join("authority-v1/keys"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
        keys_before,
        "invalid candidate validation must not remove commitment keys",
    );
    assert_eq!(
        platform::bootstrap_test(retirement.path(), material(0x6b), None).unwrap(),
        rotated,
    );
    assert!(!failed_retirement_temp.exists());
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
        AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, AuthorityObjectKindV1,
        AuthorityObjectRefV1, DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1,
        HostAttachContractHashInputV1, HostSessionAuthorityPreconditionV1, HostSessionPostureV1,
        HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
        HostSessionTransitionModeV1, PolicyObjectHashInputV1, ResumeHandleHashInputV1,
        TerminalHandoffHashInputV1, TerminalHandoffStateV1, TransitionTransportPayloadObjectV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::{
        AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
        HostSessionTransitionApplicationJournalV1, HostSessionTransitionInputHandoffV1,
        HostSessionTransitionIntentStateV1, HostSessionTransitionIntentV1,
        HostSessionTransitionTransportPayloadStateV1, InitialTransitionApplicationJournalV1,
        IssuerRequestIndexEntryV1, PostTurnApplicationJournalV1, SessionIdReservationV1,
        SessionIdTombstoneV1, SessionNamespaceRecordV1, StartTombstoneStateV1,
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
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V1(Box::new(
            parent.clone(),
        ))),
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
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V1(Box::new(
            candidate.transition_intent_map[intent_id].clone(),
        ))),
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
        parent_intent: Some(VersionedObjectVerificationParentIntentV1::V1(Box::new(
            candidate.transition_intent_map[intent_id].clone(),
        ))),
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
    let reserved_facade = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(root.path()).unwrap();
    assert!(reserved_facade.resolve_exact(session_id, None).is_err());

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
    let applied_at = TimestampV1::parse("2026-07-11T12:54:12.000000000Z").unwrap();
    let authority_intent = candidate.transition_intent_map[intent_id].clone();
    let authority_record = DurableSessionAuthorityV1 {
        schema_version: 1,
        orchestration_session_id: session_id.into(),
        shell_trace_session_id: "trace-vector".into(),
        authority_revision: 1,
        origin: DurableSessionAuthorityOriginV1::StartIntent {
            intent_id: intent_id.into(),
            issuer_request_id: request_id.into(),
            payload_commitment: authority_intent.payload_commitment.clone(),
        },
        authoritative_participant_lineage: vec!["participant-vector".into()],
        active_authoritative_participant_id: Some("participant-vector".into()),
        workspace_binding: authority_intent.workspace_binding.clone(),
        world_binding: None,
        host_attach_contract_ref: Some(authority_attach_ref.clone()),
        retained_worker_refs: Vec::new(),
        internal_resume_handle_refs: vec![resume_ref.clone()],
        lifecycle_posture: HostSessionPostureV1::ActiveAttached,
        current_policy_ref: Some(policy_ref.clone()),
        current_policy_revision: Some("policy-v1".into()),
        updated_at: applied_at.clone(),
    };
    let expected_authority_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&DurableSessionAuthorityHashInputV1 {
            schema_version: authority_record.schema_version,
            orchestration_session_id: authority_record.orchestration_session_id.clone(),
            shell_trace_session_id: authority_record.shell_trace_session_id.clone(),
            authority_revision: authority_record.authority_revision,
            origin: authority_record.origin.clone(),
            authoritative_participant_lineage: authority_record
                .authoritative_participant_lineage
                .clone(),
            active_authoritative_participant_id: authority_record
                .active_authoritative_participant_id
                .clone(),
            workspace_binding: authority_record.workspace_binding.clone(),
            world_binding: authority_record.world_binding.clone(),
            host_attach_contract_ref: authority_record.host_attach_contract_ref.clone(),
            retained_worker_refs: authority_record.retained_worker_refs.clone(),
            internal_resume_handle_refs: authority_record.internal_resume_handle_refs.clone(),
            lifecycle_posture: authority_record.lifecycle_posture,
            current_policy_ref: authority_record.current_policy_ref.clone(),
            current_policy_revision: authority_record.current_policy_revision.clone(),
        })
        .unwrap(),
    };
    let expected_lineage_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
            schema_version: 1,
            orchestration_session_id: authority_record.orchestration_session_id.clone(),
            participant_ids: authority_record.authoritative_participant_lineage.clone(),
        })
        .unwrap(),
    };
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
            authority_record_commitment: expected_authority_commitment.clone(),
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
        authority_record_commitment: expected_authority_commitment.clone(),
        application_result_ref: application_ref.clone(),
        post_turn: Box::new(
            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionPostTurnApplicationV1::NotApplicable,
        ),
        applied_at: applied_at.clone(),
    };
    applied.session_namespace_map.insert(
        session_id.into(),
        SessionNamespaceRecordV1::Authority(Box::new(authority_record.clone())),
    );
    applied.application_journal.insert(
        intent_id.into(),
        HostSessionTransitionApplicationJournalV1 {
            schema_version: 1,
            intent_id: intent_id.into(),
            initial_application: InitialTransitionApplicationJournalV1 {
                authority_revision_before: None,
                authority_revision_after: 1,
                authority_record_commitment: expected_authority_commitment.clone(),
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
    let mut cross_session_proof = applied.clone();
    let mut other_intent = cross_session_proof.transition_intent_map[intent_id].clone();
    other_intent.intent_id = "intent-other-session".into();
    other_intent.orchestration_session_id = "session-other".into();
    let mut other_journal = cross_session_proof.application_journal[intent_id].clone();
    other_journal.intent_id = other_intent.intent_id.clone();
    cross_session_proof
        .transition_intent_map
        .insert(other_intent.intent_id.clone(), other_intent.clone());
    cross_session_proof
        .application_journal
        .insert(other_journal.intent_id.clone(), other_journal);
    assert_eq!(
        crate::execution::agent_runtime::host_session_authority::facade::exact_current_authority_proof(
            &cross_session_proof,
            session_id,
            1,
        )
        .unwrap(),
        &expected_authority_commitment
    );
    other_intent.orchestration_session_id = session_id.into();
    cross_session_proof
        .transition_intent_map
        .insert(other_intent.intent_id.clone(), other_intent);
    assert!(
        crate::execution::agent_runtime::host_session_authority::facade::exact_current_authority_proof(
            &cross_session_proof,
            session_id,
            1,
        )
        .is_err()
    );
    let mut higher_same_session_proof = applied.clone();
    higher_same_session_proof
        .application_journal
        .get_mut(intent_id)
        .unwrap()
        .post_turn_application = Some(PostTurnApplicationJournalV1 {
        completion_ref: application_ref.clone(),
        authority_revision_before: 1,
        authority_revision_after: 2,
        authority_record_commitment: expected_authority_commitment.clone(),
        application_result_ref: application_ref.clone(),
        applied_at: applied_at.clone(),
    });
    assert!(
        crate::execution::agent_runtime::host_session_authority::facade::exact_current_authority_proof(
            &higher_same_session_proof,
            session_id,
            1,
        )
        .is_err()
    );
    assert_eq!(
        crate::execution::agent_runtime::host_session_authority::facade::exact_current_authority_proof(
            &higher_same_session_proof,
            session_id,
            2,
        )
        .unwrap(),
        &expected_authority_commitment
    );
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
    let authority_facade = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(root.path()).unwrap();
    let mut mismatched_authority_proof = applied.clone();
    let SessionNamespaceRecordV1::Authority(authority) = mismatched_authority_proof
        .session_namespace_map
        .get_mut(session_id)
        .unwrap()
    else {
        panic!("expected authority")
    };
    authority.current_policy_revision = Some("policy-mismatched-with-proof".into());
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(
            &mismatched_authority_proof,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        classify(root.path()),
        BootstrapClassificationV1::ValidExisting
    );
    assert!(authority_facade.resolve_exact(session_id, None).is_err());
    fs::write(
        root.path().join("authority-v1/state-root-v1.json"),
        crate::execution::agent_runtime::host_session_authority::canonical_json::to_vec(&applied)
            .unwrap(),
    )
    .unwrap();
    let resolved = authority_facade.resolve_exact(session_id, None).unwrap();
    assert_eq!(resolved.root_revision, applied.root_revision);
    assert_eq!(resolved.authority.authority_revision, 1);
    assert_eq!(
        resolved.authority.workspace_binding.authority_store_id,
        applied.authority_store_id
    );
    assert_eq!(resolved.authority.world_binding, None);
    assert_eq!(
        resolved.authority_record_commitment,
        expected_authority_commitment
    );
    assert_eq!(
        resolved.authoritative_lineage_commitment,
        expected_lineage_commitment
    );
    let reopened = crate::execution::agent_runtime::host_session_authority::facade::HostSessionAuthority::open(root.path()).unwrap();
    assert_eq!(
        reopened
            .resolve_exact(session_id, None)
            .unwrap()
            .observation(),
        resolved.observation()
    );
    let exact_observation = resolved.observation();
    let applied_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    let trusted_root = TrustedAuthorityRoot::open(root.path()).unwrap();
    let mut mismatched_exact_current = applied.clone();
    mismatched_exact_current.application_journal.clear();
    let mut exact_current_candidate = applied.clone();
    exact_current_candidate.root_revision += 1;
    assert!(platform::compare_and_swap_opened_root_exact_current(
        &trusted_root,
        &mismatched_exact_current,
        &ExpectedRevisionsV1 {
            root_revision: applied.root_revision,
            authority: None,
        },
        &exact_current_candidate,
    )
    .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        applied_bytes
    );
    assert!(fs::read_dir(root.path().join("authority-v1/tmp"))
        .unwrap()
        .next()
        .is_none());
    let mut rewritten_history = applied.clone();
    rewritten_history.root_revision += 1;
    rewritten_history
        .application_journal
        .get_mut(intent_id)
        .unwrap()
        .initial_application
        .authority_record_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "ab".repeat(32),
    };
    assert!(authority_facade
        .compare_and_swap_root(
            &ExpectedRevisionsV1 {
                root_revision: applied.root_revision,
                authority: None,
            },
            &rewritten_history,
        )
        .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        applied_bytes
    );
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
    authority.current_policy_ref = None;
    authority.current_policy_revision = None;
    let authority_expectation = ExpectedRevisionsV1 {
        root_revision: applied.root_revision,
        authority: Some(ExpectedAuthorityRevisionV1 {
            orchestration_session_id: session_id.into(),
            authority_revision: 1,
        }),
    };
    let authority_bytes_before_rejected_update =
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    assert!(authority_facade
        .compare_and_swap_root(&authority_expectation, &authority_update)
        .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        authority_bytes_before_rejected_update
    );
    assert_eq!(
        authority_facade
            .resolve_exact(session_id, Some(&exact_observation))
            .unwrap()
            .observation(),
        exact_observation
    );
    assert!(authority_facade
        .resolve_exact("missing-session", None)
        .is_err());
    assert_eq!(
        compare_and_swap_root(root.path(), &authority_expectation, &authority_update).unwrap(),
        TransactionCommitOutcomeV1::Committed(authority_update.clone())
    );
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
    let mut root_only_update = applied.clone();
    root_only_update.root_revision += 1;
    assert_eq!(
        authority_facade
            .compare_and_swap_root(
                &ExpectedRevisionsV1 {
                    root_revision: applied.root_revision,
                    authority: None,
                },
                &root_only_update,
            )
            .unwrap(),
        TransactionCommitOutcomeV1::Committed(root_only_update.clone())
    );
    assert_eq!(
        authority_facade
            .resolve_exact(session_id, None)
            .unwrap()
            .authority_record_commitment,
        expected_authority_commitment
    );
    let root_only_bytes = fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap();
    assert!(authority_facade
        .resolve_exact(session_id, Some(&exact_observation))
        .is_err());
    assert_eq!(
        fs::read(root.path().join("authority-v1/state-root-v1.json")).unwrap(),
        root_only_bytes
    );
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

    let authority_free_expectation = ExpectedRevisionsV1 {
        root_revision: candidate.root_revision,
        authority: None,
    };
    let mut reservation_only = candidate.clone();
    reservation_only.transition_intent_map.clear();
    reservation_only.issuer_request_index.clear();
    reservation_only.application_journal.clear();
    reservation_only.object_index.clear();
    assert!(platform::validate_exact_retry_expectation_test(
        &authority_free_expectation,
        &reservation_only,
    )
    .is_err());

    let mut tombstone_only = released.clone();
    tombstone_only.transition_intent_map.clear();
    tombstone_only.issuer_request_index.clear();
    tombstone_only.application_journal.clear();
    tombstone_only.object_index.clear();
    assert!(platform::validate_exact_retry_expectation_test(
        &authority_free_expectation,
        &tombstone_only,
    )
    .is_err());

    let mut intent_only = candidate.clone();
    intent_only.session_namespace_map.clear();
    intent_only.issuer_request_index.clear();
    intent_only.application_journal.clear();
    intent_only.object_index.clear();
    assert!(platform::validate_exact_retry_expectation_test(
        &authority_free_expectation,
        &intent_only,
    )
    .is_err());

    let mut issuer_only = candidate.clone();
    issuer_only.session_namespace_map.clear();
    issuer_only.transition_intent_map.clear();
    issuer_only.application_journal.clear();
    issuer_only.object_index.clear();
    assert!(platform::validate_exact_retry_expectation_test(
        &authority_free_expectation,
        &issuer_only,
    )
    .is_err());

    let mut journal_only = applied.clone();
    journal_only.session_namespace_map.clear();
    journal_only.transition_intent_map.clear();
    journal_only.issuer_request_index.clear();
    journal_only.object_index.clear();
    assert!(platform::validate_exact_retry_expectation_test(
        &authority_free_expectation,
        &journal_only,
    )
    .is_err());

    let mut object_only = candidate.clone();
    object_only.session_namespace_map.clear();
    object_only.transition_intent_map.clear();
    object_only.issuer_request_index.clear();
    object_only.application_journal.clear();
    assert!(platform::validate_exact_retry_expectation_test(
        &authority_free_expectation,
        &object_only,
    )
    .is_err());

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
    use std::io::{BufRead as _, Read as _, Write as _};

    const CHILD_TEST: &str = "execution::agent_runtime::host_session_authority::store::tests::legacy_owned_lock_is_released_when_holder_process_exits";
    const CHILD_SENTINEL: &str = "A1_OWNED_LOCK_CHILD_HOLDING";
    if let Some(root_path) = std::env::var_os("SUBSTRATE_A1_OWNED_LOCK_CHILD_ROOT") {
        let _guard = legacy_writer_guard(std::path::Path::new(&root_path)).unwrap();
        println!("{CHILD_SENTINEL}");
        std::io::stdout().flush().unwrap();
        let mut release = [0_u8; 1];
        std::io::stdin()
            .read_exact(&mut release)
            .expect("legacy lock child release pipe");
        return;
    }

    let root = root();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env("SUBSTRATE_A1_OWNED_LOCK_CHILD_ROOT", root.path())
        .stdin(std::process::Stdio::piped())
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
    let _authority_boundary = crate::execution::AuthorityEnvTestGuard::preserve();
    let bootstrap = root();
    let unrelated = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();

    let outcome = {
        let _cwd = crate::execution::ProcessCwdTestGuard::change_to(unrelated.path());
        let _authority_env = crate::execution::AuthorityEnvTestGuard::set_home(unrelated.path());
        transaction.write_file(
            LegacyStateStoreCollectionV1::Participants,
            &["retained-root.json"],
            br#"{"root":"retained"}"#,
            [0x11; 16],
        )
    };

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
fn execution_supervisor_storage_requires_exact_activated_store_and_preserves_legacy_exclusion() {
    let bootstrap = root();
    let activated = platform::bootstrap_test(bootstrap.path(), material(0xd1), None).unwrap();
    let activated_root = TrustedAuthorityRoot::open(bootstrap.path()).unwrap();
    let expected_identity = activated_root.identity().clone();
    let original_root_bytes =
        fs::read(bootstrap.path().join("authority-v1/state-root-v1.json")).unwrap();

    let other = root();
    let other_identity = TrustedAuthorityRoot::open(other.path())
        .unwrap()
        .identity()
        .clone();
    assert!(WorldWorkExecutionSupervisorStorageV1::bind(
        bootstrap.path(),
        &other_identity,
        &activated.authority_store_id,
    )
    .is_err());
    assert!(WorldWorkExecutionSupervisorStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        "as_ffffffffffffffffffffffffffffffff",
    )
    .is_err());

    let storage = WorldWorkExecutionSupervisorStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();
    let mut transaction = storage.begin_transaction().unwrap();
    assert_eq!(transaction.read_supervisor().unwrap(), None);
    transaction
        .replace_supervisor(br#"{"schema_version":1}"#)
        .unwrap();
    assert_eq!(
        transaction.read_supervisor().unwrap(),
        Some(br#"{"schema_version":1}"#.to_vec())
    );
    transaction.finish().unwrap();

    assert_eq!(
        fs::read(bootstrap.path().join("authority-v1/state-root-v1.json")).unwrap(),
        original_root_bytes
    );
    assert!(legacy_writer_guard(bootstrap.path()).is_err());
    assert_eq!(
        fs::read(
            bootstrap
                .path()
                .join("run/agent-hub/world-work-execution-supervisor-v1.json")
        )
        .unwrap(),
        br#"{"schema_version":1}"#
    );
}

#[test]
fn world_work_storage_transactions_accept_versioned_authority_without_mutating_it() {
    let bootstrap = root();
    let v1 = platform::bootstrap_test(bootstrap.path(), material(0xde), None).unwrap();
    let upgraded =
        platform::upgrade_greenfield_root_test(bootstrap.path(), [0xdf; 16], None).unwrap();
    let RootUpgradeOutcomeV1::Upgraded(v2) = upgraded else {
        panic!("fresh greenfield authority must upgrade to V2")
    };
    assert_eq!(v2.authority_store_id, v1.authority_store_id);
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let root_path = bootstrap.path().join("authority-v1/state-root-v1.json");
    let root_before = fs::read(&root_path).unwrap();

    let receipt = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &v2.authority_store_id,
    )
    .unwrap();
    let mut receipt_transaction = receipt.begin_transaction().unwrap();
    receipt_transaction
        .replace_registry(br#"{"schema_version":1}"#)
        .unwrap();
    receipt_transaction.finish().unwrap();

    let supervisor = WorldWorkExecutionSupervisorStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &v2.authority_store_id,
    )
    .unwrap();
    let mut supervisor_transaction = supervisor.begin_transaction().unwrap();
    supervisor_transaction
        .replace_supervisor(br#"{"schema_version":1}"#)
        .unwrap();
    supervisor_transaction.finish().unwrap();

    assert_eq!(fs::read(root_path).unwrap(), root_before);
    assert!(legacy_writer_guard(bootstrap.path()).is_err());
}

#[test]
fn identity_bound_legacy_transactions_reject_activated_v2_and_v3_authority() {
    let bootstrap = root();
    platform::bootstrap_test(bootstrap.path(), material(0xe2), None).unwrap();
    let RootUpgradeOutcomeV1::Upgraded(_) =
        platform::upgrade_greenfield_root_test(bootstrap.path(), [0xe3; 16], None).unwrap()
    else {
        panic!("fresh greenfield authority must upgrade to V2")
    };
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    assert!(
        begin_legacy_transaction_for_identity_test(bootstrap.path(), &expected_identity).is_err()
    );
    assert!(legacy_writer_guard(bootstrap.path()).is_err());

    let upgraded = root();
    platform::bootstrap_test(upgraded.path(), material(0xe5), None).unwrap();
    let RootUpgradeOutcomeV1::Upgraded(v2) =
        platform::upgrade_greenfield_root_test(upgraded.path(), [0xe6; 16], None).unwrap()
    else {
        panic!("fresh greenfield authority must upgrade to V2")
    };
    platform::upgrade_v2_root_to_v3_test(upgraded.path(), &v2).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(upgraded.path())
        .unwrap()
        .identity()
        .clone();
    assert!(
        begin_legacy_transaction_for_identity_test(upgraded.path(), &expected_identity).is_err()
    );
    assert!(legacy_writer_guard(upgraded.path()).is_err());
}

#[test]
fn receipt_registry_storage_rejects_post_begin_v2_authority_temp_injection() {
    let bootstrap = root();
    platform::bootstrap_test(bootstrap.path(), material(0xe0), None).unwrap();
    let RootUpgradeOutcomeV1::Upgraded(v2) =
        platform::upgrade_greenfield_root_test(bootstrap.path(), [0xe1; 16], None).unwrap()
    else {
        panic!("fresh greenfield authority must upgrade to V2")
    };
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &v2.authority_store_id,
    )
    .unwrap();
    let mut transaction = storage.begin_transaction().unwrap();
    let injected = bootstrap.path().join("authority-v1/tmp/not-a-temp");
    fs::write(&injected, b"hostile post-begin temp").unwrap();
    fs::set_permissions(&injected, fs::Permissions::from_mode(0o600)).unwrap();

    assert!(transaction
        .replace_registry(br#"{"must_not":"publish"}"#)
        .is_err());
    assert!(transaction.finish().is_err());
    assert_eq!(fs::read(injected).unwrap(), b"hostile post-begin temp");
    assert!(!bootstrap
        .path()
        .join("run/agent-hub/world-work-receipt-registry-v1.json")
        .exists());
}

#[test]
fn execution_supervisor_storage_rejects_post_begin_v2_legacy_and_marker_injection() {
    for injection in ["legacy", "marker"] {
        let bootstrap = root();
        platform::bootstrap_test(bootstrap.path(), material(0xe2), None).unwrap();
        let RootUpgradeOutcomeV1::Upgraded(v2) =
            platform::upgrade_greenfield_root_test(bootstrap.path(), [0xe3; 16], None).unwrap()
        else {
            panic!("fresh greenfield authority must upgrade to V2")
        };
        let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
            .unwrap()
            .identity()
            .clone();
        let storage = WorldWorkExecutionSupervisorStorageV1::bind(
            bootstrap.path(),
            &expected_identity,
            &v2.authority_store_id,
        )
        .unwrap();
        let mut transaction = storage.begin_transaction().unwrap();
        match injection {
            "legacy" => {
                let sessions = bootstrap.path().join("run/agent-hub/sessions");
                fs::create_dir(&sessions).unwrap();
                fs::set_permissions(&sessions, fs::Permissions::from_mode(0o700)).unwrap();
                let artifact = sessions.join("legacy.json");
                fs::write(&artifact, b"hostile post-begin legacy state").unwrap();
                fs::set_permissions(&artifact, fs::Permissions::from_mode(0o600)).unwrap();
            }
            "marker" => {
                let marker = bootstrap.path().join("authority-v1/init-v1.json");
                fs::write(&marker, br#"{"malformed":true}"#).unwrap();
                fs::set_permissions(&marker, fs::Permissions::from_mode(0o600)).unwrap();
            }
            _ => unreachable!(),
        }

        assert!(
            transaction
                .replace_supervisor(br#"{"must_not":"publish"}"#)
                .is_err(),
            "injection {injection}"
        );
        assert!(transaction.finish().is_err(), "injection {injection}");
        assert!(
            !bootstrap
                .path()
                .join("run/agent-hub/world-work-execution-supervisor-v1.json")
                .exists(),
            "injection {injection}"
        );
    }
}

#[test]
fn execution_supervisor_storage_reconciles_only_exact_safe_temps() {
    let bootstrap = root();
    let activated = platform::bootstrap_test(bootstrap.path(), material(0xd2), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkExecutionSupervisorStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();
    storage.begin_transaction().unwrap().finish().unwrap();

    let agent_hub = bootstrap.path().join("run/agent-hub");
    let recognized =
        agent_hub.join("world-work-execution-supervisor-v1--11111111111111111111111111111111.tmp");
    fs::write(&recognized, b"partial and non-authoritative").unwrap();
    fs::set_permissions(&recognized, fs::Permissions::from_mode(0o600)).unwrap();
    storage.begin_transaction().unwrap().finish().unwrap();
    assert!(!recognized.exists());

    let malformed = agent_hub.join("world-work-execution-supervisor-v1--NOT-HEX.tmp");
    fs::write(&malformed, b"unsafe temp name").unwrap();
    fs::set_permissions(&malformed, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(storage.begin_transaction().is_err());
    assert_eq!(fs::read(&malformed).unwrap(), b"unsafe temp name");
}

#[test]
fn execution_supervisor_storage_holds_root_lock_and_rejects_rebound_root() {
    let parent = root();
    let lexical_root = parent.path().join("bootstrap");
    fs::create_dir(&lexical_root).unwrap();
    fs::set_permissions(&lexical_root, fs::Permissions::from_mode(0o700)).unwrap();
    let activated = platform::bootstrap_test(&lexical_root, material(0xd3), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(&lexical_root)
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkExecutionSupervisorStorageV1::bind(
        &lexical_root,
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();

    let mut transaction = storage.begin_transaction().unwrap();
    let contender_root = TrustedAuthorityRoot::open(&lexical_root).unwrap();
    let contender_authority = contender_root
        .directory()
        .open_directory("authority-v1")
        .unwrap();
    let contender_lock = contender_authority
        .open_directory("lock")
        .unwrap()
        .open_file("root.lock")
        .unwrap();
    assert!(contender_lock.try_lock_exclusive().unwrap().is_none());

    let retained_root = parent.path().join("bootstrap-retained");
    fs::rename(&lexical_root, &retained_root).unwrap();
    fs::create_dir(&lexical_root).unwrap();
    fs::set_permissions(&lexical_root, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(transaction
        .replace_supervisor(br#"{"must_not":"publish"}"#)
        .is_err());
    assert!(transaction.finish().is_err());
    assert!(fs::read_dir(&lexical_root).unwrap().next().is_none());
    assert!(!retained_root
        .join("run/agent-hub/world-work-execution-supervisor-v1.json")
        .exists());
}

#[test]
fn receipt_registry_storage_requires_exact_activated_store_and_preserves_legacy_exclusion() {
    let bootstrap = root();
    let activated = platform::bootstrap_test(bootstrap.path(), material(0xc1), None).unwrap();
    let activated_root = TrustedAuthorityRoot::open(bootstrap.path()).unwrap();
    let expected_identity = activated_root.identity().clone();
    let original_root_bytes =
        fs::read(bootstrap.path().join("authority-v1/state-root-v1.json")).unwrap();

    let other = root();
    let other_identity = TrustedAuthorityRoot::open(other.path())
        .unwrap()
        .identity()
        .clone();
    assert!(WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &other_identity,
        &activated.authority_store_id,
    )
    .is_err());
    assert!(WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        "as_ffffffffffffffffffffffffffffffff",
    )
    .is_err());

    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();
    let mut transaction = storage.begin_transaction().unwrap();
    assert_eq!(transaction.read_registry().unwrap(), None);
    transaction
        .replace_registry(br#"{"schema_version":1}"#)
        .unwrap();
    assert_eq!(
        transaction.read_registry().unwrap(),
        Some(br#"{"schema_version":1}"#.to_vec())
    );
    transaction.finish().unwrap();

    assert_eq!(
        fs::read(bootstrap.path().join("authority-v1/state-root-v1.json")).unwrap(),
        original_root_bytes
    );
    assert!(legacy_writer_guard(bootstrap.path()).is_err());
    assert_eq!(
        fs::read(
            bootstrap
                .path()
                .join("run/agent-hub/world-work-receipt-registry-v1.json")
        )
        .unwrap(),
        br#"{"schema_version":1}"#
    );
}

#[test]
fn receipt_registry_storage_reconciles_only_exact_safe_temps() {
    let bootstrap = root();
    let activated = platform::bootstrap_test(bootstrap.path(), material(0xc2), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();
    storage.begin_transaction().unwrap().finish().unwrap();

    let agent_hub = bootstrap.path().join("run/agent-hub");
    let recognized =
        agent_hub.join("world-work-receipt-registry-v1--11111111111111111111111111111111.tmp");
    fs::write(&recognized, b"partial and non-authoritative").unwrap();
    fs::set_permissions(&recognized, fs::Permissions::from_mode(0o600)).unwrap();
    storage.begin_transaction().unwrap().finish().unwrap();
    assert!(!recognized.exists());

    let malformed = agent_hub.join("world-work-receipt-registry-v1--NOT-HEX.tmp");
    fs::write(&malformed, b"unsafe temp name").unwrap();
    fs::set_permissions(&malformed, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(storage.begin_transaction().is_err());
    assert_eq!(fs::read(&malformed).unwrap(), b"unsafe temp name");
}

#[test]
fn receipt_registry_storage_holds_root_lock_and_rejects_rebound_root() {
    let parent = root();
    let lexical_root = parent.path().join("bootstrap");
    fs::create_dir(&lexical_root).unwrap();
    fs::set_permissions(&lexical_root, fs::Permissions::from_mode(0o700)).unwrap();
    let activated = platform::bootstrap_test(&lexical_root, material(0xc3), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(&lexical_root)
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        &lexical_root,
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();

    let mut transaction = storage.begin_transaction().unwrap();
    let contender_root = TrustedAuthorityRoot::open(&lexical_root).unwrap();
    let contender_authority = contender_root
        .directory()
        .open_directory("authority-v1")
        .unwrap();
    let contender_lock = contender_authority
        .open_directory("lock")
        .unwrap()
        .open_file("root.lock")
        .unwrap();
    assert!(contender_lock.try_lock_exclusive().unwrap().is_none());

    let retained_root = parent.path().join("bootstrap-retained");
    fs::rename(&lexical_root, &retained_root).unwrap();
    fs::create_dir(&lexical_root).unwrap();
    fs::set_permissions(&lexical_root, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(transaction
        .replace_registry(br#"{"must_not":"publish"}"#)
        .is_err());
    assert!(transaction.finish().is_err());
    assert!(fs::read_dir(&lexical_root).unwrap().next().is_none());
    assert!(!retained_root
        .join("run/agent-hub/world-work-receipt-registry-v1.json")
        .exists());
}

#[test]
fn receipt_registry_storage_accepts_later_valid_root_revision_only_between_transactions() {
    let bootstrap = root();
    let current = platform::bootstrap_test(bootstrap.path(), material(0xc4), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &current.authority_store_id,
    )
    .unwrap();
    storage.begin_transaction().unwrap().finish().unwrap();

    let mut next = current.clone();
    next.root_revision += 1;
    assert!(matches!(
        compare_and_swap_root(
            bootstrap.path(),
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: None,
            },
            &next,
        )
        .unwrap(),
        TransactionCommitOutcomeV1::Committed(_)
    ));

    let mut transaction = storage.begin_transaction().unwrap();
    transaction
        .replace_registry(b"opaque receipt bytes")
        .unwrap();
    transaction.finish().unwrap();
    assert_eq!(read_root(bootstrap.path()).unwrap(), next);
}

#[test]
fn receipt_registry_storage_rejects_unsafe_final_entry_without_following_it() {
    use std::os::unix::fs::symlink;

    let bootstrap = root();
    let activated = platform::bootstrap_test(bootstrap.path(), material(0xc5), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();
    storage.begin_transaction().unwrap().finish().unwrap();

    let external = root();
    let external_file = external.path().join("outside.json");
    fs::write(&external_file, b"outside must remain unchanged").unwrap();
    fs::set_permissions(&external_file, fs::Permissions::from_mode(0o600)).unwrap();
    let registry = bootstrap
        .path()
        .join("run/agent-hub/world-work-receipt-registry-v1.json");
    symlink(&external_file, &registry).unwrap();

    assert!(storage.begin_transaction().is_err());
    assert_eq!(
        fs::read(&external_file).unwrap(),
        b"outside must remain unchanged"
    );
    assert!(fs::symlink_metadata(&registry)
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn receipt_registry_storage_rejects_named_root_lock_replacement() {
    let bootstrap = root();
    let activated = platform::bootstrap_test(bootstrap.path(), material(0xc6), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();
    let mut transaction = storage.begin_transaction().unwrap();

    let lock_directory = bootstrap.path().join("authority-v1/lock");
    fs::remove_file(lock_directory.join("root.lock")).unwrap();
    fs::write(lock_directory.join("root.lock"), b"").unwrap();
    fs::set_permissions(
        lock_directory.join("root.lock"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let replacement_root = TrustedAuthorityRoot::open(bootstrap.path()).unwrap();
    let replacement_lock = replacement_root
        .directory()
        .open_directory("authority-v1")
        .unwrap()
        .open_directory("lock")
        .unwrap()
        .open_file("root.lock")
        .unwrap();
    let replacement_guard = replacement_lock.try_lock_exclusive().unwrap().unwrap();

    assert!(transaction
        .replace_registry(b"must not publish after lock replacement")
        .is_err());
    assert!(transaction.finish().is_err());
    drop(replacement_guard);
    assert!(!bootstrap
        .path()
        .join("run/agent-hub/world-work-receipt-registry-v1.json")
        .exists());
}

#[test]
fn receipt_registry_storage_does_not_partially_clean_mixed_unsafe_temps() {
    let bootstrap = root();
    let activated = platform::bootstrap_test(bootstrap.path(), material(0xc7), None).unwrap();
    let expected_identity = TrustedAuthorityRoot::open(bootstrap.path())
        .unwrap()
        .identity()
        .clone();
    let storage = WorldWorkReceiptRegistryStorageV1::bind(
        bootstrap.path(),
        &expected_identity,
        &activated.authority_store_id,
    )
    .unwrap();
    storage.begin_transaction().unwrap().finish().unwrap();

    let agent_hub = bootstrap.path().join("run/agent-hub");
    let recognized =
        agent_hub.join("world-work-receipt-registry-v1--11111111111111111111111111111111.tmp");
    let malformed = agent_hub.join("world-work-receipt-registry-v1--zzzz.tmp");
    fs::write(&recognized, b"safe exact-name temp").unwrap();
    fs::write(&malformed, b"unsafe malformed temp").unwrap();
    fs::set_permissions(&recognized, fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(&malformed, fs::Permissions::from_mode(0o600)).unwrap();

    assert!(storage.begin_transaction().is_err());
    assert_eq!(fs::read(&recognized).unwrap(), b"safe exact-name temp");
    assert_eq!(fs::read(&malformed).unwrap(), b"unsafe malformed temp");
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

#[test]
fn legacy_transaction_directory_read_retains_enumerated_descendant_identity() {
    let bootstrap = root();
    let mut transaction = begin_legacy_state_store_transaction(bootstrap.path()).unwrap();
    transaction
        .write_file(
            LegacyStateStoreCollectionV1::Sessions,
            &["session-a", "snapshot.json"],
            b"original",
            [0x20; 16],
        )
        .unwrap();
    let entries = transaction
        .read_directory(LegacyStateStoreCollectionV1::Sessions, &[])
        .unwrap();
    assert!(entries
        .iter()
        .any(|entry| entry.name == "session-a" && entry.is_directory));

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
            [0x21; 16],
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

#[cfg(target_os = "linux")]
mod typed_object_snapshot {
    use super::*;
    use crate::execution::agent_runtime::host_session_authority::{
        facade::HostSessionAuthority,
        schema::{
            AgentDescriptorV1, AgentExecutionScopeV1, HostAttachCapabilitiesV1,
            HostAttachExecutionClientStartV1, HostAttachLaunchKnobsV1, HostAttachModePreferenceV1,
            RuntimeBackendKindV1,
        },
        store_format::{AuthorityStoreCommitmentKeyFileV1, TempNameV1},
        transition::{
            IssueHostSessionTransitionRequestV1, StartContractMaterialV1, TransitionIssueOutcomeV1,
        },
    };
    use std::collections::BTreeMap;
    use std::os::unix::fs::{symlink, MetadataExt};

    struct Fixture {
        home: tempfile::TempDir,
        opened: TrustedAuthorityRoot,
        frozen: VersionedStateRoot,
        intent: HostSessionTransitionIntentV2,
    }

    impl Fixture {
        fn published(v3: bool) -> Self {
            let home = root();
            let authority = HostSessionAuthority::open(home.path()).unwrap();
            let initial = authority.bootstrap().unwrap();
            let request = IssueHostSessionTransitionRequestV1 {
                intent_id: "snapshot-intent".into(),
                issuer_request_id: "snapshot-request".into(),
                mode: HostSessionTransitionModeV1::Start,
                authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
                orchestration_session_id: "snapshot-session".into(),
                shell_trace_session_id: "snapshot-trace".into(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::PublicCli,
                    caller_participant_id: None,
                    auto_attach_obligation_id: None,
                    auto_attach_claim_owner: None,
                },
                source_authoritative_participant_id: None,
                target_authoritative_participant_id: "snapshot-participant".into(),
                target_participant_lease_token: b"snapshot-lease".to_vec(),
                run_id: "snapshot-run".into(),
                resulting_authoritative_lineage: vec!["snapshot-participant".into()],
                workspace_binding: WorkspaceBindingV1 {
                    workspace_root: initial.bootstrap_home.clone(),
                    authority_store_root: initial.bootstrap_home,
                    authority_store_id: initial.authority_store_id,
                },
                world_binding: None,
                start_contract: StartContractMaterialV1 {
                    descriptor: AgentDescriptorV1 {
                        schema_version: 1,
                        agent_id: "codex".into(),
                        backend_id: "cli:codex".into(),
                        backend_kind: RuntimeBackendKindV1::Codex,
                        protocol: "substrate.agent.session".into(),
                        execution_scope: AgentExecutionScopeV1::Host,
                        binary_path: "/usr/bin/codex".into(),
                    },
                    policy: PolicyObjectHashInputV1 {
                        schema_version: 1,
                        policy_revision: "snapshot-policy".into(),
                        canonical_policy_snapshot_sha256: "aa".repeat(32),
                    },
                    capabilities: HostAttachCapabilitiesV1 {
                        session_resume: true,
                        session_fork: true,
                        session_stop: true,
                        status_snapshot: true,
                        event_stream: true,
                    },
                    launch_knobs: HostAttachLaunchKnobsV1 {
                        requested_execution_scope: AgentExecutionScopeV1::Host,
                        host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                        attach_mode_preference: HostAttachModePreferenceV1::ContinuityPreferred,
                    },
                },
                transition_input: Some(b"snapshot-input".to_vec()),
            };
            let TransitionIssueOutcomeV1::Issued(intent) = authority
                .issue_start_at(
                    &request,
                    TimestampV1::parse("2026-10-27T12:00:00.000000000Z").unwrap(),
                    300,
                )
                .unwrap()
            else {
                panic!("production start must be published")
            };
            let opened = TrustedAuthorityRoot::open(home.path()).unwrap();
            let mut frozen = read_opened_root_v2_or_v3(&opened).unwrap();
            if v3 {
                let VersionedStateRoot::V2(current) = &frozen else {
                    panic!("expected V2")
                };
                frozen = VersionedStateRoot::V3(
                    upgrade_v2_root_to_v3_exact_current_opened(&opened, current, || Ok(()))
                        .unwrap(),
                );
            }
            Self {
                home,
                opened,
                frozen,
                intent,
            }
        }

        fn context(&self, transport: bool) -> ObjectVerificationContextV1 {
            ObjectVerificationContextV1 {
                intent_id: self.intent.intent_id.clone(),
                run_id: self.intent.run_id.clone(),
                parent_intent: transport.then(|| {
                    VersionedObjectVerificationParentIntentV1::V2(Box::new(self.intent.clone()))
                }),
            }
        }

        fn object_path(&self, reference: &AuthorityObjectRefV1) -> std::path::PathBuf {
            let kind = match reference.object_kind {
                AuthorityObjectKindV1::AgentDescriptor => "agent-descriptor",
                AuthorityObjectKindV1::LeaseToken => "lease-token",
                AuthorityObjectKindV1::TransitionInput => "transition-input",
                AuthorityObjectKindV1::TransitionTransportPayload => "transition-transport-payload",
                _ => panic!("fixture object kind"),
            };
            self.home
                .path()
                .join("authority-v1/objects")
                .join(kind)
                .join(format!("v{}", reference.schema_version))
                .join(format!("{}.obj", reference.ref_id))
        }

        fn key_path(&self) -> std::path::PathBuf {
            let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } =
                &self.intent.target_participant_lease_token_ref.commitment
            else {
                panic!("HMAC fixture")
            };
            self.home
                .path()
                .join("authority-v1/keys")
                .join(format!("{key_id}.key"))
        }

        fn read(
            &self,
            reference: &AuthorityObjectRefV1,
            context: Option<&ObjectVerificationContextV1>,
        ) -> Result<Vec<u8>, ReadOnlyAuthoritySnapshotErrorV1> {
            with_existing_typed_object_snapshot_opened(&self.opened, |snapshot| {
                snapshot.read_typed_object(
                    self.frozen.authority_store_id(),
                    self.frozen.root_revision(),
                    reference,
                    context,
                )
            })
        }

        fn unchanged_error(
            &self,
            reference: &AuthorityObjectRefV1,
            context: Option<&ObjectVerificationContextV1>,
        ) -> ReadOnlyAuthoritySnapshotErrorV1 {
            let before = tree(self.home.path());
            let error = self.read(reference, context).unwrap_err();
            assert!(
                before == tree(self.home.path()),
                "read failure changed persistent state"
            );
            error
        }

        // Negative-only mutation of an already production-published root. Root
        // validation remains enabled; malformed relations may refuse before read.
        fn mutate_root(&self, change: impl FnOnce(&mut StateRootV2)) {
            let VersionedStateRoot::V2(mut root) = self.frozen.clone() else {
                panic!("V2 fault fixture")
            };
            change(&mut root);
            fs::write(
                self.home.path().join("authority-v1/state-root-v1.json"),
                canonical_json::to_vec(&root).unwrap(),
            )
            .unwrap();
        }
    }

    #[derive(PartialEq, Eq)]
    struct EntryState {
        metadata: (u64, u64, u32, u32, u32, u64, u64, i64, i64, i64, i64),
        bytes: Vec<u8>,
        link: Option<std::path::PathBuf>,
    }

    fn tree(root: &std::path::Path) -> BTreeMap<std::path::PathBuf, EntryState> {
        fn visit(
            root: &std::path::Path,
            path: &std::path::Path,
            result: &mut BTreeMap<std::path::PathBuf, EntryState>,
        ) {
            let metadata = fs::symlink_metadata(path).unwrap();
            result.insert(
                path.strip_prefix(root).unwrap().to_owned(),
                EntryState {
                    metadata: (
                        metadata.dev(),
                        metadata.ino(),
                        metadata.mode(),
                        metadata.uid(),
                        metadata.gid(),
                        metadata.nlink(),
                        metadata.len(),
                        metadata.mtime(),
                        metadata.mtime_nsec(),
                        metadata.ctime(),
                        metadata.ctime_nsec(),
                    ),
                    bytes: if metadata.is_file() {
                        fs::read(path).unwrap()
                    } else {
                        Vec::new()
                    },
                    link: metadata
                        .file_type()
                        .is_symlink()
                        .then(|| fs::read_link(path).unwrap()),
                },
            );
            if metadata.is_dir() {
                for entry in fs::read_dir(path).unwrap() {
                    visit(root, &entry.unwrap().path(), result);
                }
            }
        }
        let mut result = BTreeMap::new();
        visit(root, root, &mut result);
        result
    }

    fn io(operation: &'static str) -> ReadOnlyAuthoritySnapshotErrorV1 {
        ReadOnlyAuthoritySnapshotErrorV1::Io { operation }
    }

    #[test]
    fn test_production_published_v2_v3_multi_object_reads_are_unchanged() {
        for v3 in [false, true] {
            let fixture = Fixture::published(v3);
            let refs = [
                (&fixture.intent.descriptor_ref, None),
                (
                    &fixture.intent.target_participant_lease_token_ref,
                    Some(fixture.context(false)),
                ),
                (
                    fixture.intent.transition_input_ref.as_ref().unwrap(),
                    Some(fixture.context(false)),
                ),
                (
                    &fixture.intent.transport_payload_ref,
                    Some(fixture.context(true)),
                ),
            ];
            let before = tree(fixture.home.path());
            with_existing_typed_object_snapshot_opened(&fixture.opened, |snapshot| {
                for (reference, context) in &refs {
                    let bytes = snapshot.read_typed_object(
                        fixture.frozen.authority_store_id(),
                        fixture.frozen.root_revision(),
                        reference,
                        context.as_ref(),
                    )?;
                    assert!(bytes == fs::read(fixture.object_path(reference)).unwrap());
                }
                Ok(())
            })
            .unwrap();
            assert!(
                before == tree(fixture.home.path()),
                "successful reads changed persistent state"
            );
        }
    }

    #[test]
    fn test_frozen_binding_and_exact_reference_fail_without_mutation() {
        let fixture = Fixture::published(false);
        let before = tree(fixture.home.path());
        for (store, revision, expected) in [
            (
                "as_00000000000000000000000000000000",
                fixture.frozen.root_revision(),
                io("frozen HSA authority store mismatch"),
            ),
            (
                fixture.frozen.authority_store_id(),
                fixture.frozen.root_revision() + 1,
                io("frozen HSA root revision mismatch"),
            ),
        ] {
            let error = with_existing_typed_object_snapshot_opened(&fixture.opened, |snapshot| {
                snapshot.read_typed_object(store, revision, &fixture.intent.descriptor_ref, None)
            })
            .unwrap_err();
            assert_eq!(error, expected);
        }
        assert!(before == tree(fixture.home.path()));
        for case in 0..5 {
            let mut reference = fixture.intent.descriptor_ref.clone();
            match case {
                0 => reference.ref_id = "ao_00000000000000000000000000000000".into(),
                1 => reference.object_kind = AuthorityObjectKindV1::Policy,
                2 => reference.schema_version = 2,
                3 => reference.ref_id = "../unsafe".into(),
                _ => {
                    reference.commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
                        digest_hex: "00".repeat(32),
                    }
                }
            }
            let expected = match case {
                0..=2 => "typed object has no exact present index entry",
                3 => "typed object ref ID is invalid",
                _ => "canonical object commitment mismatch",
            };
            assert_eq!(fixture.unchanged_error(&reference, None), io(expected));
        }
        assert_eq!(
            fixture.unchanged_error(
                &fixture.intent.descriptor_ref,
                Some(&fixture.context(false))
            ),
            io("canonical object has unexpected parent context")
        );
    }

    #[test]
    fn test_hmac_context_domain_commitment_and_parent_are_bound() {
        let fixture = Fixture::published(true);
        let lease = &fixture.intent.target_participant_lease_token_ref;
        assert_eq!(
            fixture.unchanged_error(lease, None),
            io("sensitive object parent context is missing")
        );
        for case in 0..5 {
            let mut reference = lease.clone();
            let mut context = fixture.context(false);
            match case {
                0 => context.intent_id.push_str("-other"),
                1 => context.run_id.push_str("-other"),
                2 => {
                    let AuthorityObjectCommitmentV1::StoreHmacSha256 { domain, .. } =
                        &mut reference.commitment
                    else {
                        unreachable!()
                    };
                    domain.push_str("-other");
                }
                3 => {
                    let AuthorityObjectCommitmentV1::StoreHmacSha256 { digest_hex, .. } =
                        &mut reference.commitment
                    else {
                        unreachable!()
                    };
                    *digest_hex = "00".repeat(32);
                }
                _ => context.parent_intent = fixture.context(true).parent_intent,
            }
            let expected = match case {
                2 => "typed object commitment rule is invalid",
                4 => "sensitive object has an unexpected full parent",
                _ => "sensitive object commitment mismatch",
            };
            assert_eq!(
                fixture.unchanged_error(&reference, Some(&context)),
                io(expected)
            );
        }
        let transport = &fixture.intent.transport_payload_ref;
        assert_eq!(
            fixture.unchanged_error(transport, Some(&fixture.context(false))),
            io("transport payload parent is missing")
        );
        for case in 0..4 {
            let mut context = fixture.context(true);
            let mut reference = transport.clone();
            let Some(VersionedObjectVerificationParentIntentV1::V2(parent)) =
                &mut context.parent_intent
            else {
                unreachable!()
            };
            match case {
                0 => context.run_id.push_str("-other"),
                1 => {
                    parent.transport_payload_ref.ref_id =
                        "ao_00000000000000000000000000000000".into()
                }
                2 => parent
                    .target_authoritative_participant_id
                    .push_str("-other"),
                _ => {
                    let AuthorityObjectCommitmentV1::StoreHmacSha256 { digest_hex, .. } =
                        &mut reference.commitment
                    else {
                        unreachable!()
                    };
                    *digest_hex = "00".repeat(32);
                }
            }
            let expected = if case == 2 {
                "transport payload and V2 parent intent disagree"
            } else {
                "transport ref or HMAC context disagrees with V2 parent"
            };
            assert_eq!(
                fixture.unchanged_error(&reference, Some(&context)),
                io(expected)
            );
        }
    }

    #[test]
    fn test_key_envelope_identity_and_bytes_are_checked_after_capture() {
        for case in 0..6 {
            let fixture = Fixture::published(false);
            let path = fixture.key_path();
            let mut envelope =
                AuthorityStoreCommitmentKeyFileV1::decode(&fs::read(&path).unwrap()).unwrap();
            match case {
                0 => envelope.authority_store_id = "as_00000000000000000000000000000000".into(),
                1 => envelope.key_id = "ak_00000000000000000000000000000000".into(),
                2 => {
                    envelope.created_at =
                        TimestampV1::parse("2026-10-01T12:00:00.000000000Z").unwrap()
                }
                3 => envelope.secret_key[0] ^= 1,
                4 => {
                    fs::write(&path, b"corrupt envelope").unwrap();
                }
                _ => {
                    fs::remove_file(&path).unwrap();
                }
            }
            if case < 4 {
                fs::write(&path, envelope.encode().unwrap()).unwrap();
            }
            let expected = match case {
                0..=2 => "sensitive object key identity mismatch",
                3 => "sensitive object commitment mismatch",
                4 => "decode sensitive object key",
                _ => "sensitive object key file is absent",
            };
            assert_eq!(
                fixture.unchanged_error(
                    &fixture.intent.target_participant_lease_token_ref,
                    Some(&fixture.context(false))
                ),
                io(expected)
            );
        }
    }

    #[test]
    fn test_index_faults_keep_root_validation_and_present_checks() {
        for case in 0..6 {
            let fixture = Fixture::published(false);
            fixture.mutate_root(|root| {
                let entry = root
                    .object_index
                    .get_mut(&fixture.intent.descriptor_ref.ref_id)
                    .unwrap();
                match case {
                    0 => entry.ref_id = "ao_00000000000000000000000000000000".into(),
                    1 => entry.object_kind = AuthorityObjectKindV1::Policy,
                    2 => entry.object_schema_version = 2,
                    3 => {
                        entry.storage_state = AuthorityObjectStorageStateV1::Released {
                            terminal_handoff_ref: placeholder_ref(
                                "ao_00000000000000000000000000000000",
                                AuthorityObjectKindV1::TerminalHandoff,
                            ),
                            released_at: TimestampV1::parse("2026-10-27T12:00:00.000000000Z")
                                .unwrap(),
                        }
                    }
                    4 => entry.byte_length += 1,
                    _ => {
                        root.object_index
                            .remove(&fixture.intent.descriptor_ref.ref_id);
                    }
                }
            });
            let error = fixture.unchanged_error(&fixture.intent.descriptor_ref, None);
            // ID/schema/release conflicts refuse during root decoding. A wrong
            // descriptor kind or missing descriptor index reaches the new reader.
            let expected = match case {
                0 | 2 | 3 => ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid,
                4 => io("typed object byte length mismatch"),
                _ => io("typed object has no exact present index entry"),
            };
            assert_eq!(error, expected, "index fault {case}");
        }
    }

    #[test]
    fn test_missing_and_corrupt_object_material_is_nonmutating() {
        for case in 0..4 {
            let fixture = Fixture::published(false);
            let path = fixture.object_path(&fixture.intent.descriptor_ref);
            match case {
                0 => fs::remove_file(&path).unwrap(),
                1 => {
                    let mut bytes = fs::read(&path).unwrap();
                    bytes[0] = b'!';
                    fs::write(&path, bytes).unwrap();
                }
                2 => fs::write(&path, b"short").unwrap(),
                _ => fs::remove_dir_all(path.parent().unwrap()).unwrap(),
            }
            let expected = match case {
                0 => "typed object file is absent",
                1 => "canonical object bytes are invalid",
                2 => "typed object byte length mismatch",
                _ => "typed object namespace is absent",
            };
            assert_eq!(
                fixture.unchanged_error(&fixture.intent.descriptor_ref, None),
                io(expected)
            );
        }
    }

    #[test]
    fn test_unsafe_object_key_and_namespace_material_refuses_without_repair() {
        for is_key in [false, true] {
            for case in 0..4 {
                let fixture = Fixture::published(false);
                let path = if is_key {
                    fixture.key_path()
                } else {
                    fixture.object_path(&fixture.intent.target_participant_lease_token_ref)
                };
                match case {
                    0 => {
                        let held = path.with_extension("held");
                        fs::rename(&path, &held).unwrap();
                        symlink(&held, &path).unwrap();
                    }
                    1 => fs::hard_link(&path, path.with_extension("hardlink")).unwrap(),
                    2 => fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap(),
                    _ => {
                        fs::remove_file(&path).unwrap();
                        fs::create_dir(&path).unwrap();
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
                    }
                }
                let error = fixture.unchanged_error(
                    &fixture.intent.target_participant_lease_token_ref,
                    Some(&fixture.context(false)),
                );
                assert!(
                    matches!(
                        error,
                        ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry { .. }
                            | ReadOnlyAuthoritySnapshotErrorV1::UnsafeFileMetadata { .. }
                    ),
                    "unsafe material case {case}: {error:?}"
                );
            }
        }
        let fixture = Fixture::published(false);
        let path = fixture
            .object_path(&fixture.intent.descriptor_ref)
            .parent()
            .unwrap()
            .to_owned();
        let held = path.with_file_name("held-version");
        fs::rename(&path, &held).unwrap();
        symlink(&held, &path).unwrap();
        assert!(matches!(
            fixture.unchanged_error(&fixture.intent.descriptor_ref, None),
            ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry { .. }
        ));
    }

    #[test]
    fn test_pending_transaction_refuses_before_callback_without_repair() {
        let fixture = Fixture::published(false);
        let name = TempNameV1::Root {
            root_revision: fixture.frozen.root_revision() + 1,
            nonce: "ab".repeat(16),
        }
        .file_name();
        let path = fixture.home.path().join("authority-v1/tmp").join(&name);
        fs::write(&path, b"pending replacement").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let before = tree(fixture.home.path());
        let error =
            with_existing_typed_object_snapshot_opened(&fixture.opened, |_| -> Result<(), _> {
                panic!("pending transaction must not invoke reader")
            })
            .unwrap_err();
        assert_eq!(
            error,
            ReadOnlyAuthoritySnapshotErrorV1::AuthorityTemporaryMaterialPresent { name }
        );
        assert!(before == tree(fixture.home.path()));
    }

    #[test]
    fn test_success_and_error_exits_revalidate_captured_files_and_bindings() {
        for callback_error in [false, true] {
            for case in 0..7 {
                let fixture = Fixture::published(false);
                let object_path =
                    fixture.object_path(&fixture.intent.target_participant_lease_token_ref);
                let key_path = fixture.key_path();
                let before = tree(fixture.home.path());
                let expected_after = std::rc::Rc::new(std::cell::RefCell::new(None));
                let changed = expected_after.clone();
                let home = fixture.home.path().to_owned();
                set_e2_rm_before_final_verify_hook(move || {
                    match case {
                        0 => {
                            let bytes = fs::read(&object_path).unwrap();
                            fs::rename(&object_path, object_path.with_extension("held")).unwrap();
                            fs::write(&object_path, bytes).unwrap();
                            fs::set_permissions(&object_path, fs::Permissions::from_mode(0o600))
                                .unwrap();
                        }
                        1 => {
                            let bytes = fs::read(&key_path).unwrap();
                            fs::rename(&key_path, key_path.with_extension("held")).unwrap();
                            fs::write(&key_path, bytes).unwrap();
                            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))
                                .unwrap();
                        }
                        2 => {
                            let directory = object_path.parent().unwrap();
                            let held = directory.with_file_name("held-version");
                            fs::rename(directory, &held).unwrap();
                            fs::create_dir(directory).unwrap();
                            fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
                                .unwrap();
                            fs::copy(held.join(object_path.file_name().unwrap()), &object_path)
                                .unwrap();
                        }
                        3 => {
                            let mut bytes = fs::read(&object_path).unwrap();
                            bytes[0] ^= 1;
                            fs::write(&object_path, bytes).unwrap();
                        }
                        4 => {
                            let mut bytes = fs::read(&key_path).unwrap();
                            let last = bytes.len() - 1;
                            bytes[last] ^= 1;
                            fs::write(&key_path, bytes).unwrap();
                        }
                        5 => {
                            let directory = key_path.parent().unwrap();
                            let held = directory.with_file_name("held-keys");
                            fs::rename(directory, &held).unwrap();
                            fs::create_dir(directory).unwrap();
                            fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
                                .unwrap();
                            fs::copy(held.join(key_path.file_name().unwrap()), &key_path).unwrap();
                        }
                        _ => {
                            let directory = home.join("authority-v1/objects");
                            let held = directory.with_file_name("held-objects");
                            fs::rename(&directory, &held).unwrap();
                            fs::create_dir(&directory).unwrap();
                            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
                                .unwrap();
                        }
                    }
                    *changed.borrow_mut() = Some(tree(&home));
                });
                let mut context = fixture.context(false);
                if callback_error {
                    context.run_id.push_str("-wrong");
                }
                let error = fixture
                    .read(
                        &fixture.intent.target_participant_lease_token_ref,
                        Some(&context),
                    )
                    .unwrap_err();
                assert!(
                    matches!(
                        error,
                        ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged { .. }
                            | ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged { .. }
                    ),
                    "exit case {case}: {error:?}"
                );
                let after = tree(fixture.home.path());
                assert!(before != after, "attack must change captured material");
                assert!(
                    expected_after.borrow().as_ref().unwrap() == &after,
                    "reader changed attack state"
                );
            }
        }
    }

    #[test]
    fn test_missing_file_error_retains_namespace_and_error_exit_precedence() {
        for is_key in [false, true] {
            let fixture = Fixture::published(false);
            let reference = &fixture.intent.target_participant_lease_token_ref;
            let path = if is_key {
                fixture.key_path()
            } else {
                fixture.object_path(reference)
            };
            let bytes = fs::read(&path).unwrap();
            fs::remove_file(&path).unwrap();
            // Establish the actual missing-file failure before arranging substitution.
            let expected = if is_key {
                "sensitive object key file is absent"
            } else {
                "typed object file is absent"
            };
            assert_eq!(
                fixture.unchanged_error(reference, Some(&fixture.context(false))),
                io(expected)
            );
            let after_attack = std::rc::Rc::new(std::cell::RefCell::new(None));
            let changed = after_attack.clone();
            let home = fixture.home.path().to_owned();
            set_e2_rm_before_final_verify_hook(move || {
                fs::write(&path, bytes).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
                *changed.borrow_mut() = Some(tree(&home));
            });
            let error = fixture
                .read(reference, Some(&fixture.context(false)))
                .unwrap_err();
            assert!(matches!(
                error,
                ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged { .. }
                    | ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged { .. }
            ));
            assert!(after_attack.borrow().as_ref().unwrap() == &tree(fixture.home.path()));
        }
    }

    #[test]
    fn test_verification_only_keys_read_existing_objects_and_missing_retired_keys_refuse() {
        for missing in [false, true] {
            let mut fixture = Fixture::published(false);
            let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } =
                &fixture.intent.target_participant_lease_token_ref.commitment
            else {
                panic!("HMAC fixture")
            };
            let old_key_id = key_id.clone();
            fixture.frozen = platform::rotate_commitment_key_versioned_test(
                fixture.home.path(),
                material(0x9a),
                None,
            )
            .unwrap();
            assert_eq!(
                fixture.frozen.commitment_key_registry()[&old_key_id].state,
                AuthorityStoreCommitmentKeyStateV1::VerificationOnly
            );
            let before = tree(fixture.home.path());
            assert!(
                fixture
                    .read(
                        &fixture.intent.target_participant_lease_token_ref,
                        Some(&fixture.context(false))
                    )
                    .unwrap()
                    == b"snapshot-lease"
            );
            assert!(before == tree(fixture.home.path()));
            fixture.mutate_root(|root| {
                if missing {
                    root.commitment_key_registry.remove(&old_key_id);
                } else {
                    root.commitment_key_registry
                        .get_mut(&old_key_id)
                        .unwrap()
                        .state = AuthorityStoreCommitmentKeyStateV1::Retired;
                }
            });
            let expected = if missing {
                // Removing the initial registry key also breaks the production
                // greenfield certificate; this refuses before any object read.
                ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid
            } else {
                io("sensitive object key is unavailable")
            };
            assert_eq!(
                fixture.unchanged_error(
                    &fixture.intent.target_participant_lease_token_ref,
                    Some(&fixture.context(false))
                ),
                expected,
                "missing registry key: {missing}"
            );
        }
        // A caller-supplied reference naming an unregistered key reaches the
        // new reader on a structurally valid production root.
        let fixture = Fixture::published(false);
        let mut reference = fixture.intent.target_participant_lease_token_ref.clone();
        let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } = &mut reference.commitment
        else {
            unreachable!()
        };
        *key_id = "ak_00000000000000000000000000000000".into();
        assert_eq!(
            fixture.unchanged_error(&reference, Some(&fixture.context(false))),
            io("sensitive object key is unavailable")
        );
    }

    #[test]
    fn test_v1_frozen_roots_are_refused_without_upgrade_or_initialization() {
        let home = root();
        let authority = HostSessionAuthority::open(home.path()).unwrap();
        let initial = authority.bootstrap().unwrap();
        let opened = TrustedAuthorityRoot::open(home.path()).unwrap();
        let before = tree(home.path());
        let reference = placeholder_ref(
            "ao_00000000000000000000000000000000",
            AuthorityObjectKindV1::Policy,
        );
        let error = with_existing_typed_object_snapshot_opened(&opened, |snapshot| {
            snapshot.read_typed_object(
                &initial.authority_store_id,
                initial.root_revision,
                &reference,
                None,
            )
        })
        .unwrap_err();
        assert_eq!(
            error,
            io("typed object snapshot requires StateRootV2 or StateRootV3")
        );
        assert!(before == tree(home.path()));
    }

    #[test]
    fn test_production_terminal_transport_is_not_present_and_is_not_repaired() {
        use crate::execution::agent_runtime::host_session_authority::{
            schema::{HostStartupOwnershipProtocolActorV1, HostStartupOwnershipProtocolEventV1},
            transition::{
                ApplyHostSessionTransitionRequestV1, ClaimHostSessionTransitionRequestV1,
                ResolveStartupOwnershipRequestV1, TransitionClaimOutcomeV1,
            },
        };
        let mut fixture = Fixture::published(false);
        let authority = HostSessionAuthority::open(fixture.home.path()).unwrap();
        let claim = ClaimHostSessionTransitionRequestV1 {
            intent_id: fixture.intent.intent_id.clone(),
            issuer_request_id: fixture.intent.issuer_request_id.clone(),
            payload_commitment: fixture.intent.payload_commitment.clone(),
            expected_intent_revision: fixture.intent.intent_revision,
            claim_id: "snapshot-claim".into(),
            claimant_attempt_id: "snapshot-attempt".into(),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(
                &claim,
                TimestampV1::parse("2026-10-27T12:01:00.000000000Z").unwrap(),
                30,
            )
            .unwrap()
        else {
            panic!("production claim must publish")
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            unreachable!()
        };
        authority
            .apply_start_at(
                &ApplyHostSessionTransitionRequestV1 {
                    intent_id: claimed.intent_id.clone(),
                    issuer_request_id: claimed.issuer_request_id.clone(),
                    payload_commitment: claimed.payload_commitment.clone(),
                    expected_intent_revision: claimed.intent_revision,
                    claim_id: claim.claim_id.clone(),
                    expected_claim_revision: claim_revision,
                },
                TimestampV1::parse("2026-10-27T12:01:10.000000000Z").unwrap(),
            )
            .unwrap();
        // Publish actual terminal startup truth through the production protocol;
        // no runtime process is created by these authority publication methods.
        authority
            .resolve_startup_ownership_at(
                &ResolveStartupOwnershipRequestV1 {
                    intent_id: claimed.intent_id,
                    issuer_request_id: claimed.issuer_request_id,
                    payload_commitment: claimed.payload_commitment,
                    protocol_actor:
                        HostStartupOwnershipProtocolActorV1::LaunchApplicationClaimant {
                            claim_id: claim.claim_id,
                            claimant_attempt_id: claim.claimant_attempt_id,
                        },
                    protocol_event: HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected {
                        rejection_id: "snapshot-runtime-rejected".into(),
                    },
                    observed_at: TimestampV1::parse("2026-10-27T12:01:20.000000000Z").unwrap(),
                },
                TimestampV1::parse("2026-10-27T12:01:21.000000000Z").unwrap(),
            )
            .unwrap();
        fixture.frozen = read_opened_root_v2_or_v3(&fixture.opened).unwrap();
        // Production startup publication upgrades the retained Start root to V3.
        let VersionedStateRoot::V3(root) = &fixture.frozen else {
            unreachable!()
        };
        assert_ne!(
            root.object_index[&fixture.intent.transport_payload_ref.ref_id].storage_state,
            AuthorityObjectStorageStateV1::Present
        );
        assert_eq!(
            fixture.unchanged_error(
                &fixture.intent.transport_payload_ref,
                Some(&fixture.context(true))
            ),
            io("typed object has no exact present index entry")
        );
    }

    #[test]
    fn test_noncanonical_json_and_cross_store_hmac_are_rejected_after_capture() {
        use crate::execution::agent_runtime::host_session_authority::hash::{
            store_hmac_sha256, SensitiveDomainV1,
        };
        let fixture = Fixture::published(false);
        let mut reference = fixture.intent.target_participant_lease_token_ref.clone();
        let envelope =
            AuthorityStoreCommitmentKeyFileV1::decode(&fs::read(fixture.key_path()).unwrap())
                .unwrap();
        let AuthorityObjectCommitmentV1::StoreHmacSha256 { digest_hex, .. } =
            &mut reference.commitment
        else {
            unreachable!()
        };
        *digest_hex = store_hmac_sha256(
            &envelope.secret_key,
            SensitiveDomainV1::ParticipantLeaseToken,
            "as_00000000000000000000000000000000",
            &fixture.intent.intent_id,
            Some(&fixture.intent.run_id),
            b"snapshot-lease",
        )
        .unwrap();
        assert_eq!(
            fixture.unchanged_error(&reference, Some(&fixture.context(false))),
            io("sensitive object commitment mismatch")
        );
        let path = fixture.object_path(&fixture.intent.descriptor_ref);
        let original = fs::read(&path).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        let fields = value
            .as_object()
            .unwrap()
            .iter()
            .rev()
            .map(|(key, value)| {
                format!(
                    "{}:{}",
                    serde_json::to_string(key).unwrap(),
                    serde_json::to_string(value).unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let noncanonical = format!("{{{fields}}}").into_bytes();
        assert_eq!(original.len(), noncanonical.len());
        assert!(original != noncanonical);
        fs::write(path, noncanonical).unwrap();
        assert_eq!(
            fixture.unchanged_error(&fixture.intent.descriptor_ref, None),
            io("canonical object bytes are invalid")
        );
    }

    mod schema_binding {
        use super::*;
        use crate::execution::agent_runtime::host_session_authority::{
            schema::{
                ResumeHandleHashInputV1, StartContinuationHandleHashInputV2,
                StartContinuationHandleStateV2,
            },
            validation::CanonicalHashInputV1,
        };

        fn object<T: serde::Serialize + CanonicalHashInputV1>(
            kind: AuthorityObjectKindV1,
            schema_version: u32,
            id: u8,
            value: &T,
        ) -> (AuthorityObjectRefV1, Vec<u8>) {
            (
                AuthorityObjectRefV1 {
                    ref_id: format!("ao_{}", format!("{id:02x}").repeat(16)),
                    object_kind: kind,
                    schema_version,
                    commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                        digest_hex: canonical_sha256(value).unwrap(),
                    },
                },
                canonical_json::to_vec(value).unwrap(),
            )
        }

        // Existing Start expiry and Start-continuation protocols publish the
        // positive schema1 terminal and schema2 resume objects, respectively.
        fn published(v3: bool) -> (Fixture, Vec<AuthorityObjectRefV1>) {
            use crate::execution::agent_runtime::host_session_authority::transition::{
                ApplyHostSessionTransitionRequestV1, ClaimHostSessionTransitionRequestV1,
                ExpireHostSessionTransitionRequestV1, TransitionClaimOutcomeV1,
            };
            let mut fixture = Fixture::published(false);
            let authority = HostSessionAuthority::open(fixture.home.path()).unwrap();
            let mut continuation_ref = None;
            if v3 {
                let claim = ClaimHostSessionTransitionRequestV1 {
                    intent_id: fixture.intent.intent_id.clone(),
                    issuer_request_id: fixture.intent.issuer_request_id.clone(),
                    payload_commitment: fixture.intent.payload_commitment.clone(),
                    expected_intent_revision: fixture.intent.intent_revision,
                    claim_id: "snapshot-claim".into(),
                    claimant_attempt_id: "snapshot-attempt".into(),
                };
                let TransitionClaimOutcomeV1::Claimed(claimed) = authority
                    .claim_start_at(
                        &claim,
                        TimestampV1::parse("2026-10-27T12:01:00.000000000Z").unwrap(),
                        30,
                    )
                    .unwrap()
                else {
                    panic!("production claim must publish")
                };
                let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } =
                    claimed.state
                else {
                    unreachable!()
                };
                authority
                    .apply_start_at(
                        &ApplyHostSessionTransitionRequestV1 {
                            intent_id: claimed.intent_id.clone(),
                            issuer_request_id: claimed.issuer_request_id.clone(),
                            payload_commitment: claimed.payload_commitment.clone(),
                            expected_intent_revision: claimed.intent_revision,
                            claim_id: claim.claim_id.clone(),
                            expected_claim_revision: claim_revision,
                        },
                        TimestampV1::parse("2026-10-27T12:01:10.000000000Z").unwrap(),
                    )
                    .unwrap();
                let root = authority.read_a12a_root().unwrap();
                continuation_ref = Some(publish_continuation(
                    &authority,
                    &root.transition_intent_map[&fixture.intent.intent_id],
                ));
            } else {
                authority
                    .expire_start_at(
                        &ExpireHostSessionTransitionRequestV1 {
                            intent_id: fixture.intent.intent_id.clone(),
                            issuer_request_id: fixture.intent.issuer_request_id.clone(),
                            payload_commitment: fixture.intent.payload_commitment.clone(),
                            expected_intent_revision: fixture.intent.intent_revision,
                        },
                        TimestampV1::parse("2026-10-27T12:05:01.000000000Z").unwrap(),
                    )
                    .unwrap();
            }
            fixture.frozen = read_opened_root_v2_or_v3(&fixture.opened).unwrap();
            let intents = match &fixture.frozen {
                VersionedStateRoot::V2(root) => &root.transition_intent_map,
                VersionedStateRoot::V3(root) => &root.transition_intent_map,
                _ => unreachable!(),
            };
            let intent = &intents[&fixture.intent.intent_id];
            let reference = if let Some(reference) = continuation_ref {
                reference
            } else {
                match &intent.state {
                    HostSessionTransitionIntentStateV2::Expired {
                        terminal_handoff_ref,
                        ..
                    } => terminal_handoff_ref.clone(),
                    _ => match &intent.transport_payload_state {
                        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                            terminal_handoff_ref,
                        }
                        | HostSessionTransitionTransportPayloadStateV1::Released {
                            terminal_handoff_ref,
                            ..
                        } => terminal_handoff_ref.clone(),
                        _ => panic!("production terminal publication must retain its reference"),
                    },
                }
            };
            assert_eq!(reference.schema_version, if v3 { 2 } else { 1 });
            (fixture, vec![reference])
        }

        // Adapted from transition_tests::prepare_start_continuity_transaction.
        // All registration/root/object state is published and validated by production APIs.
        fn publish_continuation(
            authority: &HostSessionAuthority,
            intent: &HostSessionTransitionIntentV2,
        ) -> AuthorityObjectRefV1 {
            use crate::execution::agent_runtime::host_session_authority::{
                start_continuity::{
                    EstablishStartContinuationRequestV1, StartContinuationOutcomeV1,
                    StartTransactionBeginOutcomeV1,
                },
                store_schema::{StartTransactionRecordV1, StartTransactionStateV1},
            };
            fn timestamp(value: &str) -> TimestampV1 {
                TimestampV1::parse(value).unwrap()
            }
            let HostSessionTransitionIntentStateV2::Applied {
                application_result_ref,
                authority_revision_after,
                ..
            } = &intent.state
            else {
                panic!("Start must remain applied")
            };
            let root = authority.read_a12a_root().unwrap();
            let registration = EstablishStartContinuationRequestV1 {
                start_transaction_id: "stx_start_1".into(),
                request_key_sha256: "c".repeat(64),
                authority_store_id: root.authority_store_id,
                orchestration_session_id: intent.orchestration_session_id.clone(),
                intent_id: intent.intent_id.clone(),
                issuer_request_id: intent.issuer_request_id.clone(),
                payload_commitment: intent.payload_commitment.clone(),
                application_result_ref: application_result_ref.clone(),
                run_id: intent.run_id.clone(),
                expected_authority_revision: *authority_revision_after,
                authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
                backend_id: "cli:codex".into(),
                protocol: "substrate.agent.session".into(),
                exchange_id: "exchange-start-1".into(),
                exchange_sequence: 1,
                provider_event_kind: "turn.exchange_opened".into(),
                exchange_evidence_sha256: "a".repeat(64),
                internal_uaa_session_id: "uaa-session-start-1".into(),
                observed_at: timestamp("2026-10-27T12:01:20.000000000Z"),
            };
            let transaction = StartTransactionRecordV1 {
                schema_version: 1,
                transaction_id: registration.start_transaction_id.clone(),
                request_key_sha256: registration.request_key_sha256.clone(),
                prompt_sha256: "d".repeat(64),
                authority_store_id: registration.authority_store_id.clone(),
                orchestration_session_id: registration.orchestration_session_id.clone(),
                shell_trace_session_id: intent.shell_trace_session_id.clone(),
                authoritative_participant_id: registration.authoritative_participant_id.clone(),
                backend_id: registration.backend_id.clone(),
                protocol: registration.protocol.clone(),
                workspace_root: intent
                    .workspace_binding
                    .workspace_root
                    .physical_path
                    .clone(),
                world_id: intent
                    .world_binding
                    .as_ref()
                    .map(|binding| binding.world_id.clone()),
                world_generation: intent
                    .world_binding
                    .as_ref()
                    .map(|binding| binding.world_generation),
                public_backend_id: registration.backend_id.clone(),
                public_scope: if intent.world_binding.is_some() {
                    "world".into()
                } else {
                    "host".into()
                },
                start_intent_id: registration.intent_id.clone(),
                start_issuer_request_id: registration.issuer_request_id.clone(),
                start_payload_commitment: registration.payload_commitment.clone(),
                start_application_result_ref: registration.application_result_ref.clone(),
                start_run_id: registration.run_id.clone(),
                start_authority_revision: registration.expected_authority_revision,
                created_at: timestamp("2026-10-27T12:01:10.000000000Z"),
                updated_at: timestamp("2026-10-27T12:01:10.000000000Z"),
                state: StartTransactionStateV1::PromptNotSubmitted,
            };
            assert!(matches!(
                authority.begin_start_transaction(&transaction).unwrap(),
                StartTransactionBeginOutcomeV1::Applied(_)
            ));
            authority
                .mark_start_prompt_submission_no_replay_barrier(
                    &registration.start_transaction_id,
                    &registration.request_key_sha256,
                    timestamp("2026-10-27T12:01:15.000000000Z"),
                )
                .unwrap();
            match authority
                .establish_start_continuation(&registration)
                .unwrap()
            {
                StartContinuationOutcomeV1::Applied(value) => value.resume_handle_ref,
                _ => panic!("production continuation must publish"),
            }
        }

        // Resume objects are validated and published by the real canonical orphan
        // publisher. They become indexed only as an explicitly adverse root fault.
        fn adverse(position: usize) -> (Fixture, AuthorityObjectRefV1) {
            if position >= 2 {
                let (mut fixture, references) = published(false);
                if let VersionedStateRoot::V2(root) = &fixture.frozen {
                    fixture.frozen = VersionedStateRoot::V3(
                        upgrade_v2_root_to_v3_exact_current_opened(
                            &fixture.opened,
                            root,
                            || Ok(()),
                        )
                        .unwrap(),
                    );
                }
                if position == 3 {
                    use crate::execution::agent_runtime::host_session_authority::schema::{
                        TerminalHandoffHashInputV1, TerminalHandoffHashInputV2,
                    };
                    let old: TerminalHandoffHashInputV1 = canonical_json::from_slice(
                        &fs::read(path(&fixture, &references[0])).unwrap(),
                    )
                    .unwrap();
                    let value = TerminalHandoffHashInputV2 {
                        schema_version: 2,
                        intent_id: old.intent_id,
                        run_id: old.run_id,
                        payload_commitment: old.payload_commitment,
                        terminal_state: old.terminal_state,
                        application_result_ref: old.application_result_ref,
                        input_acceptance_ref: old.input_acceptance_ref,
                        startup_ownership_result_ref: None,
                        post_turn_completion_ref: old.post_turn_completion_ref,
                        post_turn_application_result_ref: old.post_turn_application_result_ref,
                        recorded_at: old.recorded_at,
                    };
                    let (reference, bytes) =
                        object(AuthorityObjectKindV1::TerminalHandoff, 2, 0x93, &value);
                    prepare_typed_object_v3_opened(
                        &fixture.opened,
                        fixture.frozen.root_revision(),
                        &reference,
                        &bytes,
                        None,
                    )
                    .unwrap();
                    return (fixture, reference);
                }
                return (fixture, references[0].clone());
            }
            let fixture = Fixture::published(true);
            let resume = ResumeHandleHashInputV1 {
                schema_version: 1,
                orchestration_session_id: fixture.intent.orchestration_session_id.clone(),
                participant_id: fixture.intent.target_authoritative_participant_id.clone(),
                backend_id: "cli:codex".into(),
                protocol: "substrate.agent.session".into(),
                internal_uaa_session_id: "schema-binding-session".into(),
            };
            let resume_v2 = StartContinuationHandleHashInputV2 {
                schema_version: 2,
                authority_store_id: fixture.frozen.authority_store_id().into(),
                orchestration_session_id: resume.orchestration_session_id.clone(),
                participant_id: resume.participant_id.clone(),
                backend_id: resume.backend_id.clone(),
                protocol: resume.protocol.clone(),
                internal_uaa_session_id: resume.internal_uaa_session_id.clone(),
                start_intent_id: fixture.intent.intent_id.clone(),
                start_issuer_request_id: fixture.intent.issuer_request_id.clone(),
                start_payload_commitment: fixture.intent.payload_commitment.clone(),
                start_application_result_ref: placeholder_ref(
                    "ao_96969696969696969696969696969696",
                    AuthorityObjectKindV1::ApplicationResult,
                ),
                start_run_id: fixture.intent.run_id.clone(),
                authority_revision_before: 1,
                authority_record_commitment_before: fixture.intent.payload_commitment.clone(),
                authority_revision_after: 2,
                state: StartContinuationHandleStateV2::Registered {
                    exchange_id: "schema-binding-exchange".into(),
                    exchange_sequence: 1,
                    provider_event_kind: "thread.started".into(),
                    evidence_sha256: "ab".repeat(32),
                    observed_at: TimestampV1::parse("2026-10-27T12:05:00.000000000Z").unwrap(),
                },
            };
            let (reference, bytes) = if position == 0 {
                object(AuthorityObjectKindV1::ResumeHandle, 1, 0x91, &resume)
            } else {
                object(AuthorityObjectKindV1::ResumeHandle, 2, 0x92, &resume_v2)
            };
            prepare_typed_object_v3_opened(
                &fixture.opened,
                fixture.frozen.root_revision(),
                &reference,
                &bytes,
                None,
            )
            .unwrap();
            (fixture, reference)
        }

        fn path(fixture: &Fixture, reference: &AuthorityObjectRefV1) -> std::path::PathBuf {
            fixture
                .home
                .path()
                .join("authority-v1/objects")
                .join(match reference.object_kind {
                    AuthorityObjectKindV1::ResumeHandle => "resume-handle",
                    AuthorityObjectKindV1::TerminalHandoff => "terminal-handoff",
                    _ => unreachable!(),
                })
                .join(format!("v{}", reference.schema_version))
                .join(format!("{}.obj", reference.ref_id))
        }

        #[test]
        fn test_schema_1_and_2_production_published_objects_read_in_v2_and_v3() {
            for v3 in [false, true] {
                let (fixture, references) = published(v3);
                let before = tree(fixture.home.path());
                for reference in references {
                    assert_eq!(
                        fixture.read(&reference, None).unwrap(),
                        fs::read(path(&fixture, &reference)).unwrap()
                    );
                }
                assert!(before == tree(fixture.home.path()));
            }
        }

        #[test]
        fn test_cross_schema_payloads_refuse_even_with_matching_digest_index_and_length() {
            for position in [1, 0, 3, 2] {
                let (fixture, original) = adverse(position);
                let bytes = fs::read(path(&fixture, &original)).unwrap();
                let mut wrong = original.clone();
                wrong.ref_id = "ao_95959595959595959595959595959595".into();
                wrong.schema_version = 3 - original.schema_version;
                let wrong_path = path(&fixture, &wrong);
                fs::create_dir_all(wrong_path.parent().unwrap()).unwrap();
                fs::set_permissions(
                    wrong_path.parent().unwrap(),
                    fs::Permissions::from_mode(0o700),
                )
                .unwrap();
                fs::write(&wrong_path, &bytes).unwrap();
                fs::set_permissions(&wrong_path, fs::Permissions::from_mode(0o600)).unwrap();
                // Negative-only extra index entry: the digest still authenticates
                // the unchanged production-published bytes under the wrong schema.
                let mut root = fixture.frozen.clone();
                let index = match &mut root {
                    VersionedStateRoot::V2(root) => &mut root.object_index,
                    VersionedStateRoot::V3(root) => &mut root.object_index,
                    _ => unreachable!(),
                };
                index.insert(
                    wrong.ref_id.clone(),
                    AuthorityObjectIndexEntryV1 {
                        schema_version: 1,
                        ref_id: wrong.ref_id.clone(),
                        object_kind: wrong.object_kind,
                        object_schema_version: wrong.schema_version,
                        byte_length: bytes.len() as u64,
                        storage_state: AuthorityObjectStorageStateV1::Present,
                    },
                );
                fs::write(
                    fixture.home.path().join("authority-v1/state-root-v1.json"),
                    root.to_canonical_bytes().unwrap(),
                )
                .unwrap();
                assert_eq!(
                    fixture.unchanged_error(&wrong, None),
                    io("canonical object schema version mismatch")
                );
            }
        }
        #[test]
        fn test_absent_or_conflicting_schema_retains_guard_and_final_error_precedence() {
            for schema in [
                None,
                Some(serde_json::json!(2)),
                Some(serde_json::json!("1")),
            ] {
                let fixture = Fixture::published(false);
                let reference = &fixture.intent.descriptor_ref;
                let path = fixture.object_path(reference);
                let mut value: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                let expected = if let Some(schema) = schema {
                    let expected = if schema.is_number() {
                        "canonical object schema version mismatch"
                    } else {
                        "canonical object schema version is absent"
                    };
                    value["schema_version"] = schema;
                    io(expected)
                } else {
                    value.as_object_mut().unwrap().remove("schema_version");
                    io("canonical object schema version is absent")
                };
                let bytes = canonical_json::to_vec(&value).unwrap();
                fs::write(&path, &bytes).unwrap();
                fixture.mutate_root(|root| {
                    root.object_index
                        .get_mut(&reference.ref_id)
                        .unwrap()
                        .byte_length = bytes.len() as u64
                });
                assert_eq!(fixture.unchanged_error(reference, None), expected);
                let mut attack_state = None;
                let error =
                    with_existing_typed_object_snapshot_opened(&fixture.opened, |snapshot| {
                        let error = snapshot
                            .read_typed_object(
                                fixture.frozen.authority_store_id(),
                                fixture.frozen.root_revision(),
                                reference,
                                None,
                            )
                            .unwrap_err();
                        assert_eq!(error, expected);
                        fs::write(&path, b"changed after schema failure").unwrap();
                        attack_state = Some(tree(fixture.home.path()));
                        Err::<Vec<u8>, _>(error)
                    })
                    .unwrap_err();
                assert!(matches!(
                    error,
                    ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged { .. }
                ));
                assert!(attack_state.unwrap() == tree(fixture.home.path()));
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod direct_host_inbox_storage_tests {
    use super::*;

    #[test]
    fn test_direct_host_inbox_storage_pre_activation_and_activated_scope() {
        let home = root();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            assert!(tx.authority_root().is_none());
            tx.write_file(
                DirectHostInboxStorageAreaV1::RawInbox,
                &["request.json"],
                None,
                b"raw request",
            )
        })
        .unwrap();
        let activated = bootstrap(home.path()).unwrap();
        with_locked_direct_host_inbox_transaction(
            home.path(),
            Some(&activated.bootstrap_home),
            |tx| {
                assert_eq!(
                    tx.authority_root().unwrap().authority_store_id(),
                    activated.authority_store_id
                );
                assert_eq!(
                    tx.read_file(DirectHostInboxStorageAreaV1::RawInbox, &["request.json"])?,
                    Some(b"raw request".to_vec())
                );
                tx.write_file(
                    DirectHostInboxStorageAreaV1::Source,
                    &["receipts", "one.json"],
                    None,
                    b"immutable receipt",
                )?;
                assert_eq!(
                    tx.list_files(DirectHostInboxStorageAreaV1::Source, &["receipts"])?,
                    vec![("one.json".to_string(), b"immutable receipt".to_vec())]
                );
                Ok(())
            },
        )
        .unwrap();
        let mut wrong = activated.bootstrap_home;
        wrong.physical_path.push_str("-copied");
        assert!(
            with_locked_direct_host_inbox_transaction(home.path(), Some(&wrong), |_| Ok(()))
                .is_err()
        );
    }

    #[test]
    fn test_direct_host_inbox_storage_exact_replace_and_retained_bytes() {
        let home = root();
        bootstrap(home.path()).unwrap();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            assert_eq!(
                tx.list_files(DirectHostInboxStorageAreaV1::Source, &[])?,
                vec![]
            );
            tx.write_file(
                DirectHostInboxStorageAreaV1::Source,
                &["head.json"],
                None,
                b"head one",
            )?;
            assert!(tx
                .write_file(
                    DirectHostInboxStorageAreaV1::Source,
                    &["head.json"],
                    None,
                    b"head two"
                )
                .is_err());
            tx.write_file(
                DirectHostInboxStorageAreaV1::Source,
                &["head.json"],
                Some(b"head one"),
                b"head two",
            )
        })
        .unwrap();
        let result = with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            assert_eq!(
                tx.read_file(DirectHostInboxStorageAreaV1::Source, &["head.json"])?,
                Some(b"head two".to_vec())
            );
            fs::write(
                home.path().join("direct-host-inbox-ledger-v1/head.json"),
                b"changed outside lock",
            )
            .unwrap();
            Ok(())
        });
        assert!(
            result.is_err(),
            "an out-of-protocol write cannot validate a retained cut"
        );
    }

    #[test]
    fn test_direct_host_inbox_storage_serializes_on_physical_root_lock() {
        use std::sync::mpsc;
        use std::time::Duration;
        let home = root();
        bootstrap(home.path()).unwrap();
        let path = home.path().to_path_buf();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let first = std::thread::spawn(move || {
            with_locked_direct_host_inbox_transaction(&path, None, |tx| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                tx.write_file(
                    DirectHostInboxStorageAreaV1::Source,
                    &["head.json"],
                    None,
                    b"first",
                )
            })
            .unwrap();
        });
        entered_rx.recv().unwrap();
        let path = home.path().to_path_buf();
        let (attempt_tx, attempt_rx) = mpsc::channel();
        let (second_tx, second_rx) = mpsc::channel();
        let second = std::thread::spawn(move || {
            attempt_tx.send(()).unwrap();
            with_locked_direct_host_inbox_transaction(&path, None, |tx| {
                second_tx.send(()).unwrap();
                assert_eq!(
                    tx.read_file(DirectHostInboxStorageAreaV1::Source, &["head.json"])?,
                    Some(b"first".to_vec())
                );
                Ok(())
            })
            .unwrap();
        });
        attempt_rx.recv().unwrap();
        assert!(matches!(
            second_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        release_tx.send(()).unwrap();
        second_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        first.join().unwrap();
        second.join().unwrap();
    }

    #[test]
    fn test_direct_host_inbox_storage_rejects_replaced_root_lock() {
        let home = root();
        bootstrap(home.path()).unwrap();
        let result = with_locked_direct_host_inbox_transaction(home.path(), None, |_| {
            let lock = home.path().join("authority-v1/lock/root.lock");
            fs::rename(&lock, lock.with_extension("old")).unwrap();
            fs::write(lock, b"").unwrap();
            Ok(())
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_direct_host_inbox_storage_rejects_unsafe_and_oversized_artifacts() {
        let home = root();
        bootstrap(home.path()).unwrap();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            assert!(tx
                .write_file(
                    DirectHostInboxStorageAreaV1::Source,
                    &["too-big.json"],
                    None,
                    &vec![0; 32 * 1024 * 1024 + 1]
                )
                .is_err());
            assert!(tx
                .read_file(
                    DirectHostInboxStorageAreaV1::Source,
                    &["..", "escaped.json"]
                )
                .is_err());
            tx.write_file(
                DirectHostInboxStorageAreaV1::Source,
                &["original.json"],
                None,
                b"original",
            )
        })
        .unwrap();
        let source = home.path().join("direct-host-inbox-ledger-v1");
        fs::hard_link(source.join("original.json"), source.join("alias.json")).unwrap();
        assert!(
            with_locked_direct_host_inbox_transaction(home.path(), None, |tx| tx
                .read_file(DirectHostInboxStorageAreaV1::Source, &["original.json"]))
            .is_err()
        );
        fs::remove_file(source.join("alias.json")).unwrap();
        std::os::unix::fs::symlink(source.join("original.json"), source.join("link.json")).unwrap();
        assert!(
            with_locked_direct_host_inbox_transaction(home.path(), None, |tx| tx
                .read_file(DirectHostInboxStorageAreaV1::Source, &["link.json"]))
            .is_err()
        );
    }
    #[test]
    fn test_direct_host_inbox_storage_replacement_bounds_pending_temp() {
        let home = root();
        bootstrap(home.path()).unwrap();
        let original = vec![b'a'; 16 * 1024 * 1024 + 1];
        let replacement = vec![b'b'; original.len()];
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            tx.write_file(
                DirectHostInboxStorageAreaV1::Source,
                &["large.json"],
                None,
                &original,
            )?;
            let refused = tx.write_file(
                DirectHostInboxStorageAreaV1::Source,
                &["large.json"],
                Some(&original),
                &replacement,
            );
            assert!(
                refused.is_err(),
                "old artifact plus pending temp exceeds 32 MiB"
            );
            assert!(
                tx.read_file(DirectHostInboxStorageAreaV1::Source, &["large.json"])?
                    .as_deref()
                    == Some(original.as_slice()),
                "refusal must preserve the original complete bytes"
            );
            assert_eq!(
                tx.list_files(DirectHostInboxStorageAreaV1::Source, &[])?
                    .len(),
                1,
                "refusal must allocate no publication temp"
            );
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_direct_host_inbox_storage_refuses_excessive_inventory_depth() {
        let home = root();
        bootstrap(home.path()).unwrap();
        with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            tx.write_file(
                DirectHostInboxStorageAreaV1::Source,
                &["original.json"],
                None,
                b"original",
            )
        })
        .unwrap();
        let mut nested = home.path().join("direct-host-inbox-ledger-v1");
        for _ in 0..17 {
            nested = nested.join("nested");
            fs::create_dir(&nested).unwrap();
            fs::set_permissions(&nested, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let error = with_locked_direct_host_inbox_transaction(home.path(), None, |tx| {
            tx.write_file(
                DirectHostInboxStorageAreaV1::Source,
                &["next.json"],
                None,
                b"next",
            )
        })
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "direct-host inbox inventory exceeds its depth bound"
        );
        assert!(!home
            .path()
            .join("direct-host-inbox-ledger-v1/next.json")
            .exists());
        assert_eq!(
            fs::read(
                home.path()
                    .join("direct-host-inbox-ledger-v1/original.json")
            )
            .unwrap(),
            b"original"
        );
    }
}

#[cfg(target_os = "linux")]
mod direct_host_resume_recovery_tests {
    use super::*;

    fn recovery_home() -> tempfile::TempDir {
        let parent = std::env::var_os("XDG_RUNTIME_DIR").expect("owned nonwriting fixture parent");
        let home = tempfile::tempdir_in(parent).unwrap();
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        home
    }

    fn snapshot(
        home: &std::path::Path,
    ) -> std::collections::BTreeMap<std::path::PathBuf, (u32, Option<Vec<u8>>)> {
        let mut result = std::collections::BTreeMap::new();
        let mut pending = vec![home.to_path_buf()];
        while let Some(path) = pending.pop() {
            let info = std::fs::symlink_metadata(&path).unwrap();
            if info.is_dir() {
                pending.extend(std::fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
            }
            result.insert(
                path.strip_prefix(home).unwrap().to_path_buf(),
                (
                    info.permissions().mode(),
                    if info.is_dir() {
                        None
                    } else {
                        Some(std::fs::read(&path).unwrap())
                    },
                ),
            );
        }
        result
    }

    #[test]
    fn test_direct_host_resume_recovery_missing_authority_has_no_effect() {
        let home = recovery_home();
        let root = TrustedAuthorityRoot::open(home.path()).unwrap();
        let before = snapshot(home.path());
        let result = super::super::with_direct_host_resume_recovery::<()>(
            &root,
            |_| panic!("committed callback requires existing authority"),
            |_| panic!("active callback requires existing authority"),
        );
        assert!(result.is_err());
        assert_eq!(snapshot(home.path()), before);
    }

    #[test]
    fn test_direct_host_resume_recovery_missing_lock_is_not_repaired() {
        let home = recovery_home();
        bootstrap(home.path()).unwrap();
        let root = TrustedAuthorityRoot::open(home.path()).unwrap();
        std::fs::remove_file(home.path().join("authority-v1/lock/root.lock")).unwrap();
        let before = snapshot(home.path());
        let result = super::super::with_direct_host_resume_recovery::<()>(
            &root,
            |_| panic!("committed callback requires original retained root lock"),
            |_| panic!("active callback requires original retained root lock"),
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("root lock is absent"));
        assert_eq!(snapshot(home.path()), before);
    }
}
