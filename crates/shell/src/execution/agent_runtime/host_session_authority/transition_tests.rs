use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::facade::{AuthorityParticipantRoleV1, HostSessionAuthority};
use super::schema::{
    AgentDescriptorV1, AgentExecutionScopeV1, AuthoritativeLineageHashInputV1,
    AuthorityObjectCommitmentV1, AuthorityObjectKindV1, AuthorityObjectRefV1,
    DurableSessionAuthorityHashInputV1, HostAttachCapabilitiesV1, HostAttachExecutionClientStartV1,
    HostAttachLaunchKnobsV1, HostAttachModePreferenceV1, HostSessionAuthorityPreconditionV1,
    HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1, HostSessionTransitionModeV1,
    PolicyObjectHashInputV1, RuntimeBackendKindV1, TimestampV1, WorkspaceBindingV1, WorldBindingV1,
};
use super::store_schema::{
    DurableSessionAuthorityV1, HostSessionPostTurnApplicationV1,
    HostSessionStartupOwnershipApplicationV1, HostSessionTransitionInputHandoffV1,
    HostSessionTransitionIntentStateV2, HostSessionTransitionTransportPayloadStateV1,
    RetainedWorkerAuthorityRegistrationRequestStateV1,
    RetainedWorkerAuthorityRegistrationRequestV1, RetainedWorkerAuthorityRegistrationV1,
    SessionNamespaceRecordV1, StartTombstoneStateV1,
};
use super::transition::{
    ApplicationCrashPointV1, ApplyHostSessionTransitionRequestV1,
    ClaimHostSessionTransitionRequestV1, ExpireHostSessionTransitionRequestV1, ExpiryCrashPointV1,
    IssueCrashPointV1, IssueHostSessionTransitionRequestV1, StartContractMaterialV1,
    TransitionApplicationOutcomeV1, TransitionClaimOutcomeV1, TransitionIssueOutcomeV1,
    TransitionTerminalOutcomeV1,
};

fn timestamp(value: &str) -> TimestampV1 {
    TimestampV1::parse(value).unwrap()
}

fn canonical_commitment<T: super::validation::CanonicalHashInputV1>(
    value: &T,
) -> AuthorityObjectCommitmentV1 {
    AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: super::hash::canonical_sha256(value).unwrap(),
    }
}

fn authority_hash_input(value: &DurableSessionAuthorityV1) -> DurableSessionAuthorityHashInputV1 {
    DurableSessionAuthorityHashInputV1 {
        schema_version: value.schema_version,
        orchestration_session_id: value.orchestration_session_id.clone(),
        shell_trace_session_id: value.shell_trace_session_id.clone(),
        authority_revision: value.authority_revision,
        origin: value.origin.clone(),
        authoritative_participant_lineage: value.authoritative_participant_lineage.clone(),
        active_authoritative_participant_id: value.active_authoritative_participant_id.clone(),
        workspace_binding: value.workspace_binding.clone(),
        world_binding: value.world_binding.clone(),
        host_attach_contract_ref: value.host_attach_contract_ref.clone(),
        retained_worker_refs: value.retained_worker_refs.clone(),
        internal_resume_handle_refs: value.internal_resume_handle_refs.clone(),
        lifecycle_posture: value.lifecycle_posture,
        current_policy_ref: value.current_policy_ref.clone(),
        current_policy_revision: value.current_policy_revision.clone(),
    }
}

fn placeholder_ref(ref_id: &str, object_kind: AuthorityObjectKindV1) -> AuthorityObjectRefV1 {
    AuthorityObjectRefV1 {
        schema_version: 1,
        ref_id: ref_id.into(),
        object_kind,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
        },
    }
}

fn authority() -> (tempfile::TempDir, HostSessionAuthority, WorkspaceBindingV1) {
    let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                .join(".cache")
        });
    fs::create_dir_all(&safe_parent).unwrap();
    let parent = tempfile::tempdir_in(safe_parent).unwrap();
    #[cfg(unix)]
    fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let home = parent.path().join("home");
    fs::create_dir(&home).unwrap();
    #[cfg(unix)]
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
    let authority = HostSessionAuthority::open(&home).unwrap();
    let root = authority.bootstrap().unwrap();
    let binding = WorkspaceBindingV1 {
        workspace_root: root.bootstrap_home.clone(),
        authority_store_root: root.bootstrap_home.clone(),
        authority_store_id: root.authority_store_id,
    };
    (parent, authority, binding)
}

fn start_request(binding: WorkspaceBindingV1) -> IssueHostSessionTransitionRequestV1 {
    IssueHostSessionTransitionRequestV1 {
        intent_id: "intent-start-1".into(),
        issuer_request_id: "request-start-1".into(),
        mode: HostSessionTransitionModeV1::Start,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
        orchestration_session_id: "session-start-1".into(),
        shell_trace_session_id: "trace-start-1".into(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::PublicCli,
            caller_participant_id: None,
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: None,
        target_authoritative_participant_id: "participant-start-1".into(),
        target_participant_lease_token: b"lease-start-1".to_vec(),
        run_id: "run-start-1".into(),
        resulting_authoritative_lineage: vec!["participant-start-1".into()],
        workspace_binding: binding,
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
                policy_revision: "policy-1".into(),
                canonical_policy_snapshot_sha256:
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
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
        transition_input: None,
    }
}

fn issue_and_claim_start(
    authority: &HostSessionAuthority,
    request: &IssueHostSessionTransitionRequestV1,
) -> ApplyHostSessionTransitionRequestV1 {
    let TransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_start_at(request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
        .unwrap()
    else {
        panic!("first issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-apply-1".into(),
        claimant_attempt_id: "attempt-apply-1".into(),
    };
    let TransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_start_at(&claim, timestamp("2026-07-14T12:01:00.000000000Z"), 30)
        .unwrap()
    else {
        panic!("first claim must commit")
    };
    let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state else {
        panic!("claimed outcome must retain a claim")
    };
    ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id,
        issuer_request_id: claimed.issuer_request_id,
        payload_commitment: claimed.payload_commitment,
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id,
        expected_claim_revision: claim_revision,
    }
}

#[test]
fn certified_start_issuance_atomically_reserves_and_exact_retry_joins() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let issued_at = timestamp("2026-07-14T12:00:00.000000000Z");
    let issued = authority
        .issue_start_at(&request, issued_at.clone(), 300)
        .unwrap();
    let TransitionIssueOutcomeV1::Issued(intent) = issued else {
        panic!("first issuance must commit")
    };
    assert!(matches!(
        intent.state,
        HostSessionTransitionIntentStateV2::Issued
    ));

    let root = authority.read_a12a_root().unwrap();
    assert_eq!(root.schema_version, 2);
    assert_eq!(root.root_revision, 3);
    assert_eq!(root.transition_intent_map[&request.intent_id], intent);
    assert!(matches!(
        root.session_namespace_map[&request.orchestration_session_id],
        SessionNamespaceRecordV1::StartReservation(_)
    ));

    assert_eq!(
        authority.issue_start_at(&request, issued_at, 300).unwrap(),
        TransitionIssueOutcomeV1::Joined(intent)
    );
    assert_eq!(authority.read_a12a_root().unwrap(), root);

    assert!(authority
        .issue_start_at(&request, timestamp("2026-07-14T12:01:00.000000000Z"), 0,)
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), root);
}

#[test]
fn conflicting_start_retry_is_non_mutating() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    authority
        .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
        .unwrap();
    let before = authority.read_a12a_root().unwrap();
    let mut conflict = request;
    conflict.target_authoritative_participant_id = "participant-conflict".into();

    assert!(authority
        .issue_start_at(&conflict, timestamp("2026-07-14T12:00:00.000000000Z"), 300,)
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), before);
}

#[test]
fn start_world_binding_matrix_accepts_host_without_binding() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);

    assert!(matches!(
        authority
            .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300,)
            .unwrap(),
        TransitionIssueOutcomeV1::Issued(_)
    ));
}

#[test]
fn start_world_binding_matrix_accepts_host_with_exact_binding() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-host-1".into(),
        world_generation: 11,
    });

    assert!(matches!(
        authority
            .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300,)
            .unwrap(),
        TransitionIssueOutcomeV1::Issued(_)
    ));
}

#[test]
fn start_world_binding_matrix_accepts_world_with_exact_binding() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-runtime-1".into(),
        world_generation: 12,
    });

    assert!(matches!(
        authority
            .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300,)
            .unwrap(),
        TransitionIssueOutcomeV1::Issued(_)
    ));
}

#[test]
fn start_world_binding_matrix_rejects_world_without_binding_without_mutation() {
    let (parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    let root_file = parent.path().join("home/authority-v1/state-root-v1.json");
    let before = fs::read(&root_file).unwrap();

    assert!(authority
        .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300,)
        .is_err());
    assert_eq!(fs::read(&root_file).unwrap(), before);
    assert!(authority.read_a12a_root().is_err());
}

#[test]
fn start_world_binding_matrix_rejects_scope_mismatch_without_mutation() {
    let (parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    let root_file = parent.path().join("home/authority-v1/state-root-v1.json");
    let before = fs::read(&root_file).unwrap();

    assert!(authority
        .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300,)
        .is_err());
    assert_eq!(fs::read(&root_file).unwrap(), before);
    assert!(authority.read_a12a_root().is_err());
}

#[test]
fn start_world_binding_matrix_rejects_empty_world_id_without_mutation() {
    let (parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.world_binding = Some(WorldBindingV1 {
        world_id: String::new(),
        world_generation: 13,
    });
    let root_file = parent.path().join("home/authority-v1/state-root-v1.json");
    let before = fs::read(&root_file).unwrap();

    assert!(authority
        .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300,)
        .is_err());
    assert_eq!(fs::read(&root_file).unwrap(), before);
    assert!(authority.read_a12a_root().is_err());
}

#[test]
fn host_world_binding_exact_retry_joins_and_changed_binding_conflicts_do_not_mutate() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-host-retry-1".into(),
        world_generation: 14,
    });
    let issued_at = timestamp("2026-07-14T12:00:00.000000000Z");
    let TransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_start_at(&request, issued_at.clone(), 300)
        .unwrap()
    else {
        panic!("first issuance must commit")
    };
    let before = authority.read_a12a_root().unwrap();

    assert_eq!(
        authority
            .issue_start_at(&request, issued_at.clone(), 300)
            .unwrap(),
        TransitionIssueOutcomeV1::Joined(issued)
    );
    assert_eq!(authority.read_a12a_root().unwrap(), before);

    let mut changed_world_id = request.clone();
    changed_world_id.world_binding.as_mut().unwrap().world_id = "world-host-retry-2".into();
    assert!(authority
        .issue_start_at(&changed_world_id, issued_at.clone(), 300)
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), before);

    let mut changed_generation = request;
    changed_generation
        .world_binding
        .as_mut()
        .unwrap()
        .world_generation += 1;
    assert!(authority
        .issue_start_at(&changed_generation, issued_at, 300)
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), before);
}

#[test]
fn applied_host_world_binding_preserves_host_placement_and_exact_session_binding() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-host-applied-1".into(),
        world_generation: 15,
    });
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();

    let resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(resolved.authority.authority_revision, 1);
    assert_eq!(
        resolved
            .authority
            .active_authoritative_participant_id
            .as_deref(),
        Some(request.target_authoritative_participant_id.as_str())
    );
    assert_eq!(
        resolved.caller.participant_id,
        request.target_authoritative_participant_id
    );
    assert_eq!(
        resolved.caller.role,
        AuthorityParticipantRoleV1::Orchestrator
    );
    assert_eq!(
        resolved.caller.descriptor,
        request.start_contract.descriptor
    );
    assert_eq!(
        resolved.caller.descriptor.execution_scope,
        AgentExecutionScopeV1::Host
    );
    assert_eq!(
        resolved.host_attach_contract.execution_scope,
        AgentExecutionScopeV1::Host
    );
    assert_eq!(
        resolved
            .host_attach_contract
            .attach_launch_knobs
            .requested_execution_scope,
        AgentExecutionScopeV1::Host
    );
    assert_eq!(
        resolved.host_attach_contract.attach_launch_knobs,
        request.start_contract.launch_knobs
    );
    assert_eq!(resolved.authority.world_binding, request.world_binding);

    let descriptor_json = serde_json::to_value(&resolved.caller.descriptor).unwrap();
    let descriptor = descriptor_json.as_object().unwrap();
    assert!(!descriptor.contains_key("world_binding"));
    assert!(!descriptor.contains_key("world_id"));
    assert!(!descriptor.contains_key("world_generation"));
}

#[test]
fn applied_host_without_world_binding_resolves_exact_host_authority() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();

    let resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(resolved.authority.authority_revision, 1);
    assert_eq!(resolved.authority.world_binding, None);
    assert_eq!(
        resolved.caller.descriptor,
        request.start_contract.descriptor
    );
    assert_eq!(
        resolved.host_attach_contract.execution_scope,
        AgentExecutionScopeV1::Host
    );
    assert_eq!(
        resolved.host_attach_contract.attach_launch_knobs,
        request.start_contract.launch_knobs
    );
    assert_eq!(
        resolved.caller.participant_id,
        request.target_authoritative_participant_id
    );
}

#[test]
fn exact_current_authority_read_rejects_corrupt_persisted_descriptor() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-host-corrupt-1".into(),
        world_generation: 16,
    });
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    assert!(authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .is_ok());

    let root = authority.read_a12a_root().unwrap();
    let descriptor_ref = &root.transition_intent_map[&request.intent_id].descriptor_ref;
    let descriptor_path = Path::new(&request.workspace_binding.authority_store_root.physical_path)
        .join("authority-v1")
        .join("objects")
        .join("agent-descriptor")
        .join("v1")
        .join(format!("{}.obj", descriptor_ref.ref_id));
    fs::write(descriptor_path, b"substituted descriptor bytes").unwrap();

    assert!(authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .is_err());
}

#[test]
fn invalid_start_rejects_before_upgrade_or_semantic_mutation() {
    let (parent, authority, binding) = authority();
    let request = start_request(binding);
    let root_file = parent.path().join("home/authority-v1/state-root-v1.json");
    let before = fs::read(&root_file).unwrap();

    assert!(authority
        .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 0,)
        .is_err());
    assert_eq!(fs::read(&root_file).unwrap(), before);
    assert!(authority.read_a12a_root().is_err());

    let mut wrong_store = request.clone();
    wrong_store.workspace_binding.authority_store_id = "store_wrong".into();
    assert!(authority
        .issue_start_at(
            &wrong_store,
            timestamp("2026-07-14T12:00:00.000000000Z"),
            300,
        )
        .is_err());
    assert_eq!(fs::read(&root_file).unwrap(), before);
    assert!(authority.read_a12a_root().is_err());

    let mut missing_workspace = start_request(wrong_store.workspace_binding.clone());
    missing_workspace.workspace_binding.authority_store_id =
        request.workspace_binding.authority_store_id.clone();
    missing_workspace
        .workspace_binding
        .workspace_root
        .physical_path = parent
        .path()
        .join("missing-workspace")
        .display()
        .to_string();
    assert!(authority
        .issue_start_at(
            &missing_workspace,
            timestamp("2026-07-14T12:00:00.000000000Z"),
            300,
        )
        .is_err());
    assert_eq!(fs::read(&root_file).unwrap(), before);
    assert!(authority.read_a12a_root().is_err());

    let mut invalid_world = request;
    invalid_world.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    invalid_world
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    invalid_world.world_binding = Some(super::schema::WorldBindingV1 {
        world_id: String::new(),
        world_generation: 1,
    });
    assert!(authority
        .issue_start_at(
            &invalid_world,
            timestamp("2026-07-14T12:00:00.000000000Z"),
            300,
        )
        .is_err());
    assert_eq!(fs::read(root_file).unwrap(), before);
    assert!(authority.read_a12a_root().is_err());
}

#[test]
fn issuance_crash_windows_leave_orphans_or_one_exact_joinable_root() {
    for crash_point in [
        IssueCrashPointV1::DescriptorPublished,
        IssueCrashPointV1::PolicyPublished,
        IssueCrashPointV1::AttachPublished,
        IssueCrashPointV1::LeasePublished,
        IssueCrashPointV1::InputPublished,
        IssueCrashPointV1::TransportPublished,
    ] {
        let (_parent, authority, binding) = authority();
        let mut request = start_request(binding);
        request.transition_input = Some(b"initial prompt".to_vec());
        let issued_at = timestamp("2026-07-14T12:00:00.000000000Z");
        assert!(authority
            .issue_start_at_with_crash_point(&request, issued_at.clone(), 300, crash_point,)
            .is_err());
        let interrupted = authority.read_a12a_root().unwrap();
        assert_eq!(interrupted.root_revision, 2);
        assert!(interrupted.session_namespace_map.is_empty());
        assert!(interrupted.transition_intent_map.is_empty());
        assert!(interrupted.issuer_request_index.is_empty());
        assert!(interrupted.object_index.is_empty());

        let TransitionIssueOutcomeV1::Issued(intent) =
            authority.issue_start_at(&request, issued_at, 300).unwrap()
        else {
            panic!("pre-root interruption must permit a fresh issuance")
        };
        assert_eq!(
            authority.read_a12a_root().unwrap().transition_intent_map[&request.intent_id],
            intent
        );
    }

    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let issued_at = timestamp("2026-07-14T12:00:00.000000000Z");
    assert!(authority
        .issue_start_at_with_crash_point(
            &request,
            issued_at.clone(),
            300,
            IssueCrashPointV1::RootCommitted,
        )
        .is_err());
    let committed = authority.read_a12a_root().unwrap();
    let intent = committed.transition_intent_map[&request.intent_id].clone();
    assert_eq!(
        authority.issue_start_at(&request, issued_at, 300).unwrap(),
        TransitionIssueOutcomeV1::Joined(intent)
    );
    assert_eq!(authority.read_a12a_root().unwrap(), committed);
}

#[test]
fn start_claim_exact_retry_and_expired_reclaim_preserve_reservation() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let TransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
        .unwrap()
    else {
        panic!("first issuance must commit")
    };
    let reservation = authority.read_a12a_root().unwrap().session_namespace_map
        [&request.orchestration_session_id]
        .clone();
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-1".into(),
        claimant_attempt_id: "attempt-1".into(),
    };
    let TransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_start_at(&claim, timestamp("2026-07-14T12:01:00.000000000Z"), 30)
        .unwrap()
    else {
        panic!("first claim must commit")
    };
    assert_eq!(claimed.intent_revision, 2);
    assert!(matches!(
        claimed.state,
        HostSessionTransitionIntentStateV2::Claimed {
            ref claim_id,
            ref claimant_attempt_id,
            claim_revision: 2,
            ..
        } if claim_id == "claim-1" && claimant_attempt_id == "attempt-1"
    ));
    let after_first = authority.read_a12a_root().unwrap();
    assert_eq!(
        after_first.session_namespace_map[&request.orchestration_session_id],
        reservation
    );

    assert!(matches!(
        authority
            .claim_start_at(&claim, timestamp("2026-07-14T12:01:01.000000000Z"), 30,)
            .unwrap(),
        TransitionClaimOutcomeV1::Joined(_)
    ));
    assert_eq!(authority.read_a12a_root().unwrap(), after_first);

    let foreign = ClaimHostSessionTransitionRequestV1 {
        expected_intent_revision: claimed.intent_revision,
        claim_id: "claim-2".into(),
        claimant_attempt_id: "attempt-2".into(),
        ..claim.clone()
    };
    assert!(authority
        .claim_start_at(&foreign, timestamp("2026-07-14T12:01:29.000000000Z"), 30,)
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), after_first);
    assert!(authority
        .claim_start_at(&claim, timestamp("2026-07-14T12:01:31.000000000Z"), 30,)
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), after_first);

    let TransitionClaimOutcomeV1::Reclaimed(reclaimed) = authority
        .claim_start_at(&foreign, timestamp("2026-07-14T12:01:31.000000000Z"), 30)
        .unwrap()
    else {
        panic!("expired foreign claim must be reclaimable")
    };
    assert_eq!(reclaimed.intent_revision, 3);
    assert!(matches!(
        reclaimed.state,
        HostSessionTransitionIntentStateV2::Claimed {
            ref claim_id,
            ref claimant_attempt_id,
            claim_revision: 3,
            ..
        } if claim_id == "claim-2" && claimant_attempt_id == "attempt-2"
    ));
    assert_eq!(
        authority.read_a12a_root().unwrap().session_namespace_map
            [&request.orchestration_session_id],
        reservation
    );
}

#[test]
fn start_application_atomically_births_authority_with_pending_startup_ownership() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    let TransitionApplicationOutcomeV1::Applied(applied) = authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap()
    else {
        panic!("first application must commit")
    };
    assert!(matches!(
        applied.state,
        HostSessionTransitionIntentStateV2::Applied {
            ref claim_id,
            ref claimant_attempt_id,
            authority_revision_before: None,
            authority_revision_after: 1,
            startup_ownership,
            post_turn,
            ..
        } if claim_id == "claim-apply-1"
            && claimant_attempt_id == "attempt-apply-1"
            && matches!(
                startup_ownership.as_ref(),
                HostSessionStartupOwnershipApplicationV1::Pending {
                    expected_run_id,
                    expected_authority_revision: 1,
                    expected_active_authoritative_participant_id,
                } if expected_run_id == &request.run_id
                    && expected_active_authoritative_participant_id
                        == &request.target_authoritative_participant_id
            )
            && post_turn.as_ref() == &HostSessionPostTurnApplicationV1::NotApplicable
    ));

    let committed = authority.read_a12a_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current) =
        &committed.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("applied Start must replace its reservation with authority")
    };
    assert_eq!(current.authority_revision, 1);
    assert_eq!(
        current.active_authoritative_participant_id.as_deref(),
        Some(request.target_authoritative_participant_id.as_str())
    );
    assert_eq!(
        current.authoritative_participant_lineage,
        request.resulting_authoritative_lineage
    );
    assert_eq!(current.workspace_binding, request.workspace_binding);
    assert_eq!(current.world_binding, request.world_binding);
    assert_eq!(
        current.current_policy_revision.as_deref(),
        Some(request.start_contract.policy.policy_revision.as_str())
    );
    assert!(committed
        .application_journal
        .contains_key(&request.intent_id));

    assert!(matches!(
        authority
            .apply_start_at(&application, timestamp("2026-07-14T12:01:11.000000000Z"),)
            .unwrap(),
        TransitionApplicationOutcomeV1::Joined(_)
    ));
    assert_eq!(authority.read_a12a_root().unwrap(), committed);
}

#[test]
fn exact_applied_start_accepts_only_unique_contiguous_r0_registration_ancestry() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-start-1".into(),
        world_generation: 1,
    });
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let mut descendant_root = authority.read_a12a_root().unwrap();
    let SessionNamespaceRecordV1::Authority(initial) =
        descendant_root.session_namespace_map[&request.orchestration_session_id].clone()
    else {
        panic!("applied Start must have authority")
    };
    let initial_commitment = match &descendant_root.transition_intent_map[&request.intent_id].state
    {
        HostSessionTransitionIntentStateV2::Applied {
            authority_record_commitment,
            ..
        } => authority_record_commitment.clone(),
        _ => panic!("Start must be applied"),
    };
    assert_eq!(
        canonical_commitment(&authority_hash_input(&initial)),
        initial_commitment.clone()
    );

    let descriptor_ref = placeholder_ref(
        "ao_91111111111111111111111111111111",
        AuthorityObjectKindV1::AgentDescriptor,
    );
    let resume_handle_ref = placeholder_ref(
        "ao_92222222222222222222222222222222",
        AuthorityObjectKindV1::ResumeHandle,
    );
    let retained_worker_ref = placeholder_ref(
        "ao_93333333333333333333333333333333",
        AuthorityObjectKindV1::RetainedWorker,
    );
    let registered_at = timestamp("2026-07-14T12:02:00.000000000Z");
    let mut descendant = (*initial).clone();
    descendant.authority_revision = 2;
    descendant
        .authoritative_participant_lineage
        .push("participant-retained-1".into());
    descendant
        .retained_worker_refs
        .push(retained_worker_ref.clone());
    descendant.updated_at = registered_at.clone();
    let descendant_commitment = canonical_commitment(&authority_hash_input(&descendant));
    let lineage_commitment = canonical_commitment(&AuthoritativeLineageHashInputV1 {
        schema_version: 1,
        orchestration_session_id: request.orchestration_session_id.clone(),
        participant_ids: descendant.authoritative_participant_lineage.clone(),
    });
    let policy_ref = initial.current_policy_ref.clone().unwrap();
    let world_binding = initial.world_binding.clone().unwrap();
    let registration = RetainedWorkerAuthorityRegistrationV1 {
        schema_version: 1,
        issuer_request_id: "r0-request-1".into(),
        registration_id: "r0-registration-1".into(),
        orchestration_session_id: request.orchestration_session_id.clone(),
        authority_revision_before: 1,
        authority_record_commitment_before: initial_commitment.clone(),
        authority_revision_after: 2,
        authority_record_commitment_after: descendant_commitment.clone(),
        retained_participant_id: "participant-retained-1".into(),
        authoritative_lineage_commitment_after: lineage_commitment,
        descriptor_ref: descriptor_ref.clone(),
        resume_handle_ref: resume_handle_ref.clone(),
        retained_worker_ref: retained_worker_ref.clone(),
        current_policy_ref: policy_ref.clone(),
        world_binding: world_binding.clone(),
        registered_at: registered_at.clone(),
    };
    descendant_root
        .retained_worker_registration_request_index
        .insert(
            registration.issuer_request_id.clone(),
            RetainedWorkerAuthorityRegistrationRequestV1 {
                schema_version: 1,
                issuer_request_id: registration.issuer_request_id.clone(),
                registration_id: registration.registration_id.clone(),
                orchestration_session_id: registration.orchestration_session_id.clone(),
                authority_revision_before: 1,
                authority_record_commitment_before: initial_commitment.clone(),
                retained_participant_id: registration.retained_participant_id.clone(),
                descriptor_ref_id: descriptor_ref.ref_id.clone(),
                descriptor_commitment: descriptor_ref.commitment.clone(),
                resume_handle_ref_id: resume_handle_ref.ref_id.clone(),
                resume_handle_commitment: resume_handle_ref.commitment.clone(),
                retained_worker_ref_id: retained_worker_ref.ref_id.clone(),
                retained_worker_commitment: retained_worker_ref.commitment.clone(),
                current_policy_ref: policy_ref,
                world_binding,
                registered_at,
                state: RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                    authority_revision_after: 2,
                    authority_record_commitment_after: descendant_commitment,
                },
            },
        );
    descendant_root
        .retained_worker_registration_journal
        .insert(registration.registration_id.clone(), registration.clone());
    descendant_root.session_namespace_map.insert(
        request.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(descendant.clone())),
    );
    super::transition::verify_retained_registration_descendant(
        &descendant_root,
        &initial,
        &descendant,
        &initial_commitment,
    )
    .unwrap();

    let mut ambiguous = descendant_root;
    let mut fork = registration;
    fork.registration_id = "r0-registration-fork".into();
    fork.issuer_request_id = "r0-request-fork".into();
    ambiguous
        .retained_worker_registration_journal
        .insert(fork.registration_id.clone(), fork);
    assert!(super::transition::verify_retained_registration_descendant(
        &ambiguous,
        &initial,
        &descendant,
        &initial_commitment,
    )
    .is_err());
}

#[test]
fn start_application_rejects_stale_substituted_and_expired_claims_without_mutation() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    let claimed = authority.read_a12a_root().unwrap();
    for mutation in ["intent", "claim", "claim_revision", "payload"] {
        let mut invalid = application.clone();
        match mutation {
            "intent" => invalid.expected_intent_revision += 1,
            "claim" => invalid.claim_id = "substituted-claim".into(),
            "claim_revision" => invalid.expected_claim_revision += 1,
            "payload" => {
                invalid.payload_commitment =
                    super::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                        digest_hex:
                            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                                .into(),
                    };
            }
            _ => unreachable!(),
        }
        assert!(authority
            .apply_start_at(&invalid, timestamp("2026-07-14T12:01:10.000000000Z"),)
            .is_err());
        assert_eq!(authority.read_a12a_root().unwrap(), claimed, "{mutation}");
    }
    assert!(authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:30.000000000Z"),)
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), claimed);
}

#[test]
fn start_application_crash_windows_reconcile_without_duplicate_birth() {
    for crash_point in [
        ApplicationCrashPointV1::ResultPublished,
        ApplicationCrashPointV1::RootCommitted,
    ] {
        let (_parent, authority, binding) = authority();
        let home = binding.authority_store_root.physical_path.clone();
        let request = start_request(binding);
        let application = issue_and_claim_start(&authority, &request);
        let claimed = authority.read_a12a_root().unwrap();
        assert!(authority
            .apply_start_at_with_crash_point(
                &application,
                timestamp("2026-07-14T12:01:10.000000000Z"),
                crash_point,
            )
            .is_err());
        drop(authority);
        let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
        let observed = reopened.read_a12a_root().unwrap();
        match crash_point {
            ApplicationCrashPointV1::ResultPublished => assert_eq!(observed, claimed),
            ApplicationCrashPointV1::RootCommitted => assert!(matches!(
                observed.session_namespace_map.get(&request.orchestration_session_id),
                Some(SessionNamespaceRecordV1::Authority(value))
                    if value.authority_revision == 1
            )),
        }
        let result = reopened
            .apply_start_at(&application, timestamp("2026-07-14T12:01:11.000000000Z"))
            .unwrap();
        assert!(matches!(
            result,
            TransitionApplicationOutcomeV1::Applied(_) | TransitionApplicationOutcomeV1::Joined(_)
        ));
        let final_root = reopened.read_a12a_root().unwrap();
        assert_eq!(final_root.application_journal.len(), 1);
        assert!(matches!(
            final_root.session_namespace_map.get(&request.orchestration_session_id),
            Some(SessionNamespaceRecordV1::Authority(value))
                if value.authority_revision == 1
        ));
    }
}

#[test]
fn start_expiry_atomically_terminalizes_input_and_burns_the_namespace() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.transition_input = Some(b"initial prompt".to_vec());
    let issued_at = timestamp("2026-07-14T12:00:00.000000000Z");
    let TransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_start_at(&request, issued_at.clone(), 300)
        .unwrap()
    else {
        panic!("first issuance must commit")
    };
    let expiry = ExpireHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
    };
    let before = authority.read_a12a_root().unwrap();
    assert!(authority
        .expire_start_at(&expiry, timestamp("2026-07-14T12:04:59.999999999Z"))
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), before);

    let TransitionTerminalOutcomeV1::Expired(expired) = authority
        .expire_start_at(&expiry, timestamp("2026-07-14T12:05:00.000000000Z"))
        .unwrap()
    else {
        panic!("fixed expiry must commit terminal Start truth")
    };
    let terminal_ref = match &expired.state {
        HostSessionTransitionIntentStateV2::Expired {
            terminal_handoff_ref,
            expired_at,
        } if expired_at.as_str() == "2026-07-14T12:05:00.000000000Z" => {
            terminal_handoff_ref.clone()
        }
        state => panic!("unexpected terminal state: {state:?}"),
    };
    assert!(matches!(
        expired.input_handoff,
        HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
            ref terminal_handoff_ref,
            ..
        } if terminal_handoff_ref == &terminal_ref
    ));
    assert_eq!(
        expired.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::Retained
    );
    let terminal = authority.read_a12a_root().unwrap();
    assert!(terminal.application_journal.is_empty());
    assert!(matches!(
        terminal.session_namespace_map[&request.orchestration_session_id],
        SessionNamespaceRecordV1::StartTombstone(ref tombstone)
            if tombstone.intent_id == request.intent_id
                && tombstone.terminal_state == StartTombstoneStateV1::Expired
                && tombstone.terminal_handoff_ref == terminal_ref
    ));

    assert!(matches!(
        authority
            .expire_start_at(&expiry, timestamp("2026-07-14T12:06:00.000000000Z"))
            .unwrap(),
        TransitionTerminalOutcomeV1::Joined(ref joined) if joined == &expired
    ));
    assert_eq!(authority.read_a12a_root().unwrap(), terminal);
    assert!(matches!(
        authority
            .issue_start_at(&request, issued_at, 300)
            .unwrap(),
        TransitionIssueOutcomeV1::Joined(ref joined) if joined == &expired
    ));
    assert_eq!(authority.read_a12a_root().unwrap(), terminal);
}

#[test]
fn claimed_start_expires_only_at_fixed_intent_expiry_and_conflicts_do_not_mutate() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let TransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
        .unwrap()
    else {
        panic!("first issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-expiry-1".into(),
        claimant_attempt_id: "attempt-expiry-1".into(),
    };
    let TransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_start_at(&claim, timestamp("2026-07-14T12:01:00.000000000Z"), 30)
        .unwrap()
    else {
        panic!("claim must commit")
    };
    let expiry = ExpireHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
    };
    let claimed_root = authority.read_a12a_root().unwrap();
    let mut conflict = expiry.clone();
    conflict.payload_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
    };
    assert!(authority
        .expire_start_at(&conflict, timestamp("2026-07-14T12:05:00.000000000Z"))
        .is_err());
    assert_eq!(authority.read_a12a_root().unwrap(), claimed_root);
    assert!(matches!(
        authority
            .expire_start_at(&expiry, timestamp("2026-07-14T12:05:00.000000000Z"))
            .unwrap(),
        TransitionTerminalOutcomeV1::Expired(_)
    ));
}

#[test]
fn start_expiry_crash_windows_reconcile_to_one_terminal_root() {
    for crash_point in [
        ExpiryCrashPointV1::TerminalPublished,
        ExpiryCrashPointV1::RootCommitted,
    ] {
        let (_parent, authority, binding) = authority();
        let home = binding.authority_store_root.physical_path.clone();
        let request = start_request(binding);
        let TransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_start_at(&request, timestamp("2026-07-14T12:00:00.000000000Z"), 300)
            .unwrap()
        else {
            panic!("first issuance must commit")
        };
        let expiry = ExpireHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
        };
        let issued_root = authority.read_a12a_root().unwrap();
        assert!(authority
            .expire_start_at_with_crash_point(
                &expiry,
                timestamp("2026-07-14T12:05:00.000000000Z"),
                crash_point,
            )
            .is_err());
        drop(authority);
        let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
        let observed = reopened.read_a12a_root().unwrap();
        match crash_point {
            ExpiryCrashPointV1::TerminalPublished => assert_eq!(observed, issued_root),
            ExpiryCrashPointV1::RootCommitted => assert!(matches!(
                observed.transition_intent_map[&request.intent_id].state,
                HostSessionTransitionIntentStateV2::Expired { .. }
            )),
        }
        assert!(matches!(
            reopened
                .expire_start_at(&expiry, timestamp("2026-07-14T12:06:00.000000000Z"))
                .unwrap(),
            TransitionTerminalOutcomeV1::Expired(_) | TransitionTerminalOutcomeV1::Joined(_)
        ));
        let final_root = reopened.read_a12a_root().unwrap();
        assert!(matches!(
            final_root.session_namespace_map[&request.orchestration_session_id],
            SessionNamespaceRecordV1::StartTombstone(_)
        ));
        assert_eq!(
            final_root
                .object_index
                .values()
                .filter(|entry| entry.object_kind == AuthorityObjectKindV1::TerminalHandoff)
                .count(),
            1
        );
    }
}

#[test]
fn typed_current_authority_read_joins_applied_descriptor_and_bound_store() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding.clone());
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-current-1".into(),
        world_generation: 7,
    });
    let application = issue_and_claim_start(&authority, &request);
    assert!(authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .is_err());
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();

    let resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(
        resolved.authority.orchestration_session_id,
        request.orchestration_session_id
    );
    assert_eq!(resolved.authority.authority_revision, 1);
    assert_eq!(
        resolved.authority.authoritative_participant_lineage,
        request.resulting_authoritative_lineage
    );
    assert_eq!(
        resolved.authority.workspace_binding,
        request.workspace_binding
    );
    assert_eq!(resolved.authority.world_binding, request.world_binding);
    assert_eq!(
        resolved.caller.participant_id,
        request.target_authoritative_participant_id
    );
    assert_eq!(
        resolved.caller.role,
        AuthorityParticipantRoleV1::Orchestrator
    );
    assert_eq!(
        resolved.caller.descriptor,
        request.start_contract.descriptor
    );
    assert_eq!(
        resolved.host_attach_contract.policy_ref,
        resolved.authority.current_policy_ref.clone().unwrap()
    );
    assert_eq!(resolved.current_policy, request.start_contract.policy);
    assert_eq!(
        resolved.bound_state_store.bootstrap_home_identity(),
        &binding.authority_store_root
    );

    let observation = resolved.observation.clone();
    let exact = authority
        .resolve_current_exact(&request.orchestration_session_id, Some(&observation))
        .unwrap();
    assert_eq!(exact.observation, observation);
    let mut stale = observation;
    stale.authority_revision += 1;
    assert!(authority
        .resolve_current_exact(&request.orchestration_session_id, Some(&stale))
        .is_err());
}
