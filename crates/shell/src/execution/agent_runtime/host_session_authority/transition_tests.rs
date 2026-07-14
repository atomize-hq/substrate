use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::facade::HostSessionAuthority;
use super::schema::{
    AgentDescriptorV1, AgentExecutionScopeV1, HostAttachCapabilitiesV1,
    HostAttachExecutionClientStartV1, HostAttachLaunchKnobsV1, HostAttachModePreferenceV1,
    HostSessionAuthorityPreconditionV1, HostSessionTransitionCallerKindV1,
    HostSessionTransitionCallerV1, HostSessionTransitionModeV1, PolicyObjectHashInputV1,
    RuntimeBackendKindV1, TimestampV1, WorkspaceBindingV1,
};
use super::store_schema::{HostSessionTransitionIntentStateV2, SessionNamespaceRecordV1};
use super::transition::{
    IssueCrashPointV1, IssueHostSessionTransitionRequestV1, StartContractMaterialV1,
    TransitionIssueOutcomeV1,
};

fn timestamp(value: &str) -> TimestampV1 {
    TimestampV1::parse(value).unwrap()
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
    fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let home = parent.path().join("home");
    fs::create_dir(&home).unwrap();
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
