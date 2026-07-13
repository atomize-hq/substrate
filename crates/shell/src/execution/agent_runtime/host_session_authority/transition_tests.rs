use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Barrier};

use super::facade::HostSessionAuthority;
use super::schema::{
    AgentDescriptorV1, AgentExecutionScopeV1, ApplicationResultHashInputV1,
    ApplicationResultPhaseV1, AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1,
    AuthorityObjectKindV1, DurableSessionAuthorityHashInputV1, HostAttachCapabilitiesV1,
    HostAttachContractHashInputV1, HostAttachExecutionClientStartV1, HostAttachLaunchKnobsV1,
    HostAttachModePreferenceV1, HostPostTurnDispositionV1, HostSessionAuthorityPreconditionV1,
    HostSessionPostureV1, HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
    HostSessionTransitionModeV1, PolicyObjectHashInputV1, PostTurnCompletionHashInputV1,
    PostTurnCompletionOutcomeV1, ResumeHandleHashInputV1, RuntimeBackendKindV1, TimestampV1,
    WorkspaceBindingV1,
};
use super::store::{ExpectedAuthorityRevisionV1, ExpectedRevisionsV1};
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
    HostSessionPostTurnApplicationV1, HostSessionTransitionIntentStateV1,
    PostTurnApplicationJournalV1, SessionNamespaceRecordV1,
};
use super::transition::{
    AcceptTransitionInputRequestV1, ApplicationCrashPointV1, ApplyHostSessionTransitionRequestV1,
    ApplyPostTurnCompletionRequestV1, ClaimHostSessionTransitionRequestV1,
    ExpireHostSessionTransitionRequestV1, InputAcceptanceOutcomeV1, IssueCrashPointV1,
    IssueHostSessionTransitionRequestV1, PostTurnCompletionOutcomeResultV1,
    StartContractMaterialV1, TransitionApplicationOutcomeV1, TransitionClaimOutcomeV1,
    TransitionIdentityRequestV1, TransitionIssueOutcomeV1, TransitionTerminalOutcomeV1,
    TransportReleaseCrashPointV1, TransportReleaseOutcomeV1,
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
        start_contract: Some(StartContractMaterialV1 {
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
        }),
        resume_handle_ref: None,
        transition_input: None,
        post_turn_disposition: None,
    }
}

fn issue_and_claim_start(
    authority: &HostSessionAuthority,
    request: &IssueHostSessionTransitionRequestV1,
) -> ApplyHostSessionTransitionRequestV1 {
    let issued = match authority
        .issue_transition_at(request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        TransitionIssueOutcomeV1::Joined(_) => panic!("first issuance joined"),
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-apply-1".into(),
        claimant_attempt_id: "attempt-apply-1".into(),
    };
    let claimed = match authority
        .claim_transition_at(&claim, timestamp("2026-07-13T12:01:00.000000000Z"), 30)
        .unwrap()
    {
        TransitionClaimOutcomeV1::Claimed(intent) => intent,
        other => panic!("unexpected claim result: {other:?}"),
    };
    ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: "claim-apply-1".into(),
        expected_claim_revision: 1,
    }
}

fn authority_commitment(value: &DurableSessionAuthorityV1) -> AuthorityObjectCommitmentV1 {
    AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: super::hash::canonical_sha256(&DurableSessionAuthorityHashInputV1 {
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
        })
        .unwrap(),
    }
}

fn parked_precondition(value: &DurableSessionAuthorityV1) -> HostSessionAuthorityPreconditionV1 {
    HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision: value.authority_revision,
        authority_record_commitment: authority_commitment(value),
        active_authoritative_participant_id: value
            .active_authoritative_participant_id
            .clone()
            .unwrap(),
        authoritative_lineage_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: super::hash::canonical_sha256(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: value.orchestration_session_id.clone(),
                participant_ids: value.authoritative_participant_lineage.clone(),
            })
            .unwrap(),
        },
        lifecycle_posture: value.lifecycle_posture,
    }
}

fn start_and_park(
    authority: &HostSessionAuthority,
    binding: WorkspaceBindingV1,
    with_resume_handle: bool,
) -> (
    DurableSessionAuthorityV1,
    Option<super::schema::AuthorityObjectRefV1>,
) {
    let start = start_request(binding);
    let application = issue_and_claim_start(authority, &start);
    authority
        .apply_transition_at(&application, timestamp("2026-07-13T12:01:10.000000000Z"))
        .unwrap();
    let current = authority.read_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) = current
        .session_namespace_map
        .get(&start.orchestration_session_id)
        .unwrap()
    else {
        panic!("Start did not create authority")
    };
    let mut parked = current_authority.as_ref().clone();
    parked.authority_revision += 1;
    parked.lifecycle_posture = HostSessionPostureV1::ParkedResumable;
    parked.updated_at = timestamp("2026-07-13T12:02:00.000000000Z");

    let mut proposed = current.clone();
    proposed.root_revision += 1;
    let resume_ref = if with_resume_handle {
        let attach_ref = parked.host_attach_contract_ref.clone().unwrap();
        let attach_bytes = authority
            .read_typed_object(current.root_revision, &attach_ref, None)
            .unwrap();
        let mut attach: HostAttachContractHashInputV1 =
            super::canonical_json::from_slice(&attach_bytes).unwrap();
        let resume_value = ResumeHandleHashInputV1 {
            schema_version: 1,
            orchestration_session_id: parked.orchestration_session_id.clone(),
            participant_id: parked.active_authoritative_participant_id.clone().unwrap(),
            backend_id: attach.contract.backend_id.clone(),
            protocol: attach.contract.protocol.clone(),
            internal_uaa_session_id: "uaa-session-parked-1".into(),
        };
        let resume_bytes = super::hash::canonical_object_bytes(
            AuthorityObjectKindV1::ResumeHandle,
            super::hash::CanonicalObjectHashInputV1::ResumeHandle(&resume_value),
        )
        .unwrap();
        let resume = authority
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::ResumeHandle,
                &resume_bytes,
                None,
            )
            .unwrap();
        attach.contract.continuity_resume_handle_ref = Some(resume.reference.clone());
        let next_attach_bytes = super::hash::canonical_object_bytes(
            AuthorityObjectKindV1::HostAttachContract,
            super::hash::CanonicalObjectHashInputV1::HostAttachContract(&attach),
        )
        .unwrap();
        let next_attach = authority
            .prepare_generated_object(
                current.root_revision,
                AuthorityObjectKindV1::HostAttachContract,
                &next_attach_bytes,
                None,
            )
            .unwrap();
        parked.host_attach_contract_ref = Some(next_attach.reference.clone());
        parked.internal_resume_handle_refs = vec![resume.reference.clone()];
        for (object, kind, byte_length) in [
            (
                &resume.reference,
                AuthorityObjectKindV1::ResumeHandle,
                resume.byte_length,
            ),
            (
                &next_attach.reference,
                AuthorityObjectKindV1::HostAttachContract,
                next_attach.byte_length,
            ),
        ] {
            proposed.object_index.insert(
                object.ref_id.clone(),
                AuthorityObjectIndexEntryV1 {
                    schema_version: 1,
                    ref_id: object.ref_id.clone(),
                    object_kind: kind,
                    object_schema_version: object.schema_version,
                    byte_length,
                    storage_state: AuthorityObjectStorageStateV1::Present,
                },
            );
        }
        Some(resume.reference)
    } else {
        None
    };
    // This fixture establishes a schema-valid current parked proof without
    // exercising any A1.3 producer. Production post-turn application is tested
    // through the protocol below; the fixture itself remains test-only.
    let start_intent = proposed
        .transition_intent_map
        .get(&start.intent_id)
        .unwrap()
        .clone();
    let completed_at = timestamp("2026-07-13T12:02:00.000000000Z");
    let completion_value = PostTurnCompletionHashInputV1 {
        schema_version: 1,
        intent_id: start_intent.intent_id.clone(),
        run_id: start_intent.run_id.clone(),
        authority_revision_observed: current_authority.authority_revision,
        outcome: PostTurnCompletionOutcomeV1::ResumableClean,
        completed_at: completed_at.clone(),
    };
    let completion_bytes = super::hash::canonical_object_bytes(
        AuthorityObjectKindV1::PostTurnCompletion,
        super::hash::CanonicalObjectHashInputV1::PostTurnCompletion(&completion_value),
    )
    .unwrap();
    let completion = authority
        .prepare_generated_object(
            current.root_revision,
            AuthorityObjectKindV1::PostTurnCompletion,
            &completion_bytes,
            None,
        )
        .unwrap();
    let parked_commitment = authority_commitment(&parked);
    let application_value = ApplicationResultHashInputV1 {
        schema_version: 1,
        intent_id: start_intent.intent_id.clone(),
        mode: start_intent.mode,
        run_id: start_intent.run_id.clone(),
        phase: ApplicationResultPhaseV1::PostTurn {
            completion_ref: completion.reference.clone(),
            authority_revision_before: current_authority.authority_revision,
            authority_revision_after: parked.authority_revision,
            active_authoritative_participant_id: parked
                .active_authoritative_participant_id
                .clone()
                .unwrap(),
            resulting_posture: HostSessionPostureV1::ParkedResumable,
            authority_record_commitment: parked_commitment.clone(),
        },
        applied_at: completed_at.clone(),
    };
    let application_bytes = super::hash::canonical_object_bytes(
        AuthorityObjectKindV1::ApplicationResult,
        super::hash::CanonicalObjectHashInputV1::ApplicationResult(&application_value),
    )
    .unwrap();
    let application = authority
        .prepare_generated_object(
            current.root_revision,
            AuthorityObjectKindV1::ApplicationResult,
            &application_bytes,
            None,
        )
        .unwrap();
    let intent = proposed
        .transition_intent_map
        .get_mut(&start.intent_id)
        .unwrap();
    let HostSessionTransitionIntentStateV1::Applied { post_turn, .. } = &mut intent.state else {
        panic!("Start fixture is not applied")
    };
    *post_turn = Box::new(HostSessionPostTurnApplicationV1::Applied {
        completion_ref: Box::new(completion.reference.clone()),
        authority_revision_before: current_authority.authority_revision,
        authority_revision_after: parked.authority_revision,
        resulting_posture: HostSessionPostureV1::ParkedResumable,
        application_result_ref: Box::new(application.reference.clone()),
        applied_at: completed_at.clone(),
    });
    intent.intent_revision += 1;
    intent.updated_at = completed_at.clone();
    proposed
        .application_journal
        .get_mut(&start.intent_id)
        .unwrap()
        .post_turn_application = Some(PostTurnApplicationJournalV1 {
        completion_ref: completion.reference.clone(),
        authority_revision_before: current_authority.authority_revision,
        authority_revision_after: parked.authority_revision,
        authority_record_commitment: parked_commitment,
        application_result_ref: application.reference.clone(),
        applied_at: completed_at,
    });
    for (object, kind, byte_length) in [
        (
            &completion.reference,
            AuthorityObjectKindV1::PostTurnCompletion,
            completion.byte_length,
        ),
        (
            &application.reference,
            AuthorityObjectKindV1::ApplicationResult,
            application.byte_length,
        ),
    ] {
        proposed.object_index.insert(
            object.ref_id.clone(),
            AuthorityObjectIndexEntryV1 {
                schema_version: 1,
                ref_id: object.ref_id.clone(),
                object_kind: kind,
                object_schema_version: object.schema_version,
                byte_length,
                storage_state: AuthorityObjectStorageStateV1::Present,
            },
        );
    }
    proposed.session_namespace_map.insert(
        parked.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(parked.clone())),
    );
    proposed.validate().unwrap();
    let workspace = super::trusted_fs::TrustedWorkspaceRoot::open_exact(
        &parked.workspace_binding.workspace_root,
    )
    .unwrap();
    authority
        .commit_transition_root(
            &current,
            &ExpectedRevisionsV1 {
                root_revision: current.root_revision,
                authority: Some(ExpectedAuthorityRevisionV1 {
                    orchestration_session_id: parked.orchestration_session_id.clone(),
                    authority_revision: current_authority.authority_revision,
                }),
            },
            &proposed,
            None,
            &workspace,
        )
        .unwrap();
    (parked, resume_ref)
}

fn successor_request(
    authority: &DurableSessionAuthorityV1,
    mode: HostSessionTransitionModeV1,
    resume_handle_ref: Option<super::schema::AuthorityObjectRefV1>,
) -> IssueHostSessionTransitionRequestV1 {
    let suffix = match mode {
        HostSessionTransitionModeV1::Attach => "attach",
        HostSessionTransitionModeV1::ResumeOneTurn => "resume",
        HostSessionTransitionModeV1::Start => panic!("successor helper does not build Start"),
    };
    let source = authority
        .active_authoritative_participant_id
        .clone()
        .unwrap();
    let target = format!("participant-{suffix}-2");
    let mut lineage = authority.authoritative_participant_lineage.clone();
    lineage.push(target.clone());
    IssueHostSessionTransitionRequestV1 {
        intent_id: format!("intent-{suffix}-2"),
        issuer_request_id: format!("request-{suffix}-2"),
        mode,
        authority_precondition: parked_precondition(authority),
        orchestration_session_id: authority.orchestration_session_id.clone(),
        shell_trace_session_id: authority.shell_trace_session_id.clone(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::PublicCli,
            caller_participant_id: Some(source.clone()),
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: Some(source),
        target_authoritative_participant_id: target,
        target_participant_lease_token: format!("lease-{suffix}-2").into_bytes(),
        run_id: format!("run-{suffix}-2"),
        resulting_authoritative_lineage: lineage,
        workspace_binding: authority.workspace_binding.clone(),
        world_binding: authority.world_binding.clone(),
        start_contract: None,
        resume_handle_ref,
        transition_input: (mode == HostSessionTransitionModeV1::ResumeOneTurn)
            .then(|| b"one exact turn".to_vec()),
        post_turn_disposition: (mode == HostSessionTransitionModeV1::ResumeOneTurn)
            .then_some(HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal),
    }
}

#[test]
fn certified_start_issuance_atomically_reserves_and_exact_retry_joins() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let issued = authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap();
    assert!(matches!(issued, TransitionIssueOutcomeV1::Issued(_)));

    let first_root = authority.read_root().unwrap();
    let intent = first_root
        .transition_intent_map
        .get("intent-start-1")
        .unwrap();
    assert_eq!(intent.state, HostSessionTransitionIntentStateV1::Issued);
    assert!(matches!(
        first_root.session_namespace_map.get("session-start-1"),
        Some(SessionNamespaceRecordV1::StartReservation(reservation))
            if reservation.intent_id == "intent-start-1"
                && reservation.issuer_request_id == "request-start-1"
                && reservation.payload_commitment == intent.payload_commitment
    ));

    let joined = authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:30:00.000000000Z"), 300)
        .unwrap();
    assert!(matches!(joined, TransitionIssueOutcomeV1::Joined(_)));
    assert_eq!(authority.read_root().unwrap(), first_root);
}

#[test]
fn conflicting_start_retry_is_non_mutating() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap();
    let before = authority.read_root().unwrap();
    let mut substituted = request;
    substituted.target_participant_lease_token = b"substituted-lease".to_vec();
    assert!(authority
        .issue_transition_at(
            &substituted,
            timestamp("2026-07-13T12:00:01.000000000Z"),
            300,
        )
        .is_err());
    assert_eq!(authority.read_root().unwrap(), before);
}

#[test]
fn start_application_atomically_births_revision_one_and_exact_retry_joins() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    let applied = authority
        .apply_transition_at(&application, timestamp("2026-07-13T12:01:10.000000000Z"))
        .unwrap();
    assert!(matches!(
        applied,
        TransitionApplicationOutcomeV1::Applied(_)
    ));
    let committed = authority.read_root().unwrap();
    let intent = &committed.transition_intent_map[&application.intent_id];
    assert!(matches!(
        &intent.state,
        HostSessionTransitionIntentStateV1::Applied {
            claim_id,
            authority_revision_before: None,
            authority_revision_after: 1,
            ..
        } if claim_id == &application.claim_id
    ));
    assert!(committed
        .application_journal
        .contains_key(&application.intent_id));
    assert!(matches!(
        committed
            .session_namespace_map
            .get(&request.orchestration_session_id),
        Some(SessionNamespaceRecordV1::Authority(value))
            if value.authority_revision == 1
                && value.orchestration_session_id == request.orchestration_session_id
                && value.shell_trace_session_id == request.shell_trace_session_id
                && value.authoritative_participant_lineage
                    == request.resulting_authoritative_lineage
                && value.active_authoritative_participant_id.as_deref()
                    == Some(request.target_authoritative_participant_id.as_str())
                && value.workspace_binding == request.workspace_binding
                && value.world_binding == request.world_binding
    ));

    assert!(matches!(
        authority
            .apply_transition_at(&application, timestamp("2026-07-13T12:01:11.000000000Z"),)
            .unwrap(),
        TransitionApplicationOutcomeV1::Joined(_)
    ));
    assert_eq!(authority.read_root().unwrap(), committed);
}

#[test]
fn start_application_rejects_stale_substituted_and_expired_claims_without_mutation() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    let claimed = authority.read_root().unwrap();
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
                    }
            }
            _ => unreachable!(),
        }
        assert!(authority
            .apply_transition_at(&invalid, timestamp("2026-07-13T12:01:10.000000000Z"),)
            .is_err());
        assert_eq!(
            authority.read_root().unwrap(),
            claimed,
            "mutation={mutation}"
        );
    }
    assert!(authority
        .apply_transition_at(&application, timestamp("2026-07-13T12:01:30.000000000Z"),)
        .is_err());
    assert_eq!(authority.read_root().unwrap(), claimed);
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
        let claimed = authority.read_root().unwrap();
        assert!(authority
            .apply_transition_at_with_crash_point(
                &application,
                timestamp("2026-07-13T12:01:10.000000000Z"),
                crash_point,
            )
            .is_err());
        drop(authority);
        let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
        let observed = reopened.read_root().unwrap();
        match crash_point {
            ApplicationCrashPointV1::ResultPublished => assert_eq!(observed, claimed),
            ApplicationCrashPointV1::RootCommitted => assert!(matches!(
                observed
                    .session_namespace_map
                    .get(&request.orchestration_session_id),
                Some(SessionNamespaceRecordV1::Authority(value))
                    if value.authority_revision == 1
            )),
        }
        let result = reopened
            .apply_transition_at(&application, timestamp("2026-07-13T12:01:11.000000000Z"))
            .unwrap();
        assert!(matches!(
            result,
            TransitionApplicationOutcomeV1::Applied(_) | TransitionApplicationOutcomeV1::Joined(_)
        ));
        let final_root = reopened.read_root().unwrap();
        assert_eq!(
            final_root.application_journal.len(),
            1,
            "crash_point={crash_point:?}"
        );
        assert!(matches!(
            final_root
                .session_namespace_map
                .get(&request.orchestration_session_id),
            Some(SessionNamespaceRecordV1::Authority(value))
                if value.authority_revision == 1
        ));
    }
}

#[test]
fn parked_attach_and_resume_apply_exact_successors_without_identity_reconstruction() {
    for mode in [
        HostSessionTransitionModeV1::Attach,
        HostSessionTransitionModeV1::ResumeOneTurn,
    ] {
        let (_parent, authority, binding) = authority();
        let (parked, resume_ref) = start_and_park(
            &authority,
            binding,
            mode == HostSessionTransitionModeV1::ResumeOneTurn,
        );
        let request = successor_request(&parked, mode, resume_ref);
        let issued = match authority
            .issue_transition_at(&request, timestamp("2026-07-13T12:03:00.000000000Z"), 300)
            .unwrap()
        {
            TransitionIssueOutcomeV1::Issued(intent) => intent,
            other => panic!("unexpected issuance result: {other:?}"),
        };
        assert!(matches!(
            authority
                .issue_transition_at(&request, timestamp("2026-07-13T12:03:01.000000000Z"), 300,)
                .unwrap(),
            TransitionIssueOutcomeV1::Joined(_)
        ));
        let claim = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: format!("claim-{mode:?}"),
            claimant_attempt_id: format!("attempt-{mode:?}"),
        };
        let claimed = match authority
            .claim_transition_at(&claim, timestamp("2026-07-13T12:04:00.000000000Z"), 30)
            .unwrap()
        {
            TransitionClaimOutcomeV1::Claimed(intent) => intent,
            other => panic!("unexpected claim result: {other:?}"),
        };
        let application = ApplyHostSessionTransitionRequestV1 {
            intent_id: claimed.intent_id.clone(),
            issuer_request_id: claimed.issuer_request_id.clone(),
            payload_commitment: claimed.payload_commitment.clone(),
            expected_intent_revision: claimed.intent_revision,
            claim_id: claim.claim_id.clone(),
            expected_claim_revision: 1,
        };
        authority
            .apply_transition_at(&application, timestamp("2026-07-13T12:04:10.000000000Z"))
            .unwrap();
        let committed = authority.read_root().unwrap();
        let SessionNamespaceRecordV1::Authority(successor) = committed
            .session_namespace_map
            .get(&request.orchestration_session_id)
            .unwrap()
        else {
            panic!("successor application lost authority")
        };
        assert_eq!(successor.authority_revision, parked.authority_revision + 1);
        assert_eq!(
            successor.orchestration_session_id,
            parked.orchestration_session_id
        );
        assert_eq!(
            successor.shell_trace_session_id,
            parked.shell_trace_session_id
        );
        assert_eq!(successor.workspace_binding, parked.workspace_binding);
        assert_eq!(successor.world_binding, parked.world_binding);
        assert_eq!(successor.origin, parked.origin);
        assert_eq!(
            successor.authoritative_participant_lineage,
            request.resulting_authoritative_lineage
        );
        assert_eq!(
            successor.active_authoritative_participant_id.as_deref(),
            Some(request.target_authoritative_participant_id.as_str())
        );
        assert_eq!(
            successor.lifecycle_posture,
            HostSessionPostureV1::ActiveAttached
        );
        let applied = &committed.transition_intent_map[&request.intent_id];
        match &applied.state {
            HostSessionTransitionIntentStateV1::Applied { post_turn, .. } => match mode {
                HostSessionTransitionModeV1::Attach => assert!(matches!(
                    post_turn.as_ref(),
                    HostSessionPostTurnApplicationV1::NotApplicable
                )),
                HostSessionTransitionModeV1::ResumeOneTurn => assert!(matches!(
                    post_turn.as_ref(),
                    HostSessionPostTurnApplicationV1::Pending {
                        expected_run_id,
                        expected_authority_revision,
                    } if expected_run_id == &request.run_id
                        && *expected_authority_revision == successor.authority_revision
                )),
                HostSessionTransitionModeV1::Start => unreachable!(),
            },
            other => panic!("unexpected application state: {other:?}"),
        }
        assert!(matches!(
            authority
                .apply_transition_at(&application, timestamp("2026-07-13T12:04:11.000000000Z"),)
                .unwrap(),
            TransitionApplicationOutcomeV1::Joined(_)
        ));
        assert_eq!(authority.read_root().unwrap(), committed);
    }
}

#[test]
fn parked_successor_rejects_stale_posture_binding_lineage_and_cross_kind_replay() {
    let (_parent, authority, binding) = authority();
    let (parked, _) = start_and_park(&authority, binding, false);
    let baseline = authority.read_root().unwrap();
    for mutation in ["posture", "world", "lineage", "mode"] {
        let mut request = successor_request(&parked, HostSessionTransitionModeV1::Attach, None);
        request.intent_id = format!("intent-invalid-{mutation}");
        request.issuer_request_id = format!("request-invalid-{mutation}");
        request.run_id = format!("run-invalid-{mutation}");
        request.target_authoritative_participant_id = format!("participant-invalid-{mutation}");
        request.resulting_authoritative_lineage = parked.authoritative_participant_lineage.clone();
        request
            .resulting_authoritative_lineage
            .push(request.target_authoritative_participant_id.clone());
        match mutation {
            "posture" => {
                let HostSessionAuthorityPreconditionV1::ExpectedRevision {
                    lifecycle_posture, ..
                } = &mut request.authority_precondition
                else {
                    unreachable!()
                };
                *lifecycle_posture = HostSessionPostureV1::ActiveAttached;
            }
            "world" => {
                request.world_binding = Some(super::schema::WorldBindingV1 {
                    world_id: "wrong-world".into(),
                    world_generation: 99,
                })
            }
            "lineage" => request.resulting_authoritative_lineage.swap(0, 1),
            "mode" => {
                request.mode = HostSessionTransitionModeV1::ResumeOneTurn;
                request.transition_input = Some(b"cross-kind".to_vec());
                request.post_turn_disposition =
                    Some(HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal);
            }
            _ => unreachable!(),
        }
        assert!(authority
            .issue_transition_at(&request, timestamp("2026-07-13T12:03:00.000000000Z"), 300,)
            .is_err());
        assert_eq!(
            authority.read_root().unwrap(),
            baseline,
            "mutation={mutation}"
        );
    }
}

#[test]
fn resume_input_acceptance_and_post_turn_are_revisioned_and_idempotent() {
    let (_parent, authority, binding) = authority();
    let (parked, resume_ref) = start_and_park(&authority, binding, true);
    let request = successor_request(
        &parked,
        HostSessionTransitionModeV1::ResumeOneTurn,
        resume_ref,
    );
    let issued = match authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:03:00.000000000Z"), 300)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        other => panic!("unexpected issuance: {other:?}"),
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-resume-post-turn".into(),
        claimant_attempt_id: "attempt-resume-post-turn".into(),
    };
    let claimed = match authority
        .claim_transition_at(&claim, timestamp("2026-07-13T12:04:00.000000000Z"), 30)
        .unwrap()
    {
        TransitionClaimOutcomeV1::Claimed(intent) => intent,
        other => panic!("unexpected claim: {other:?}"),
    };
    let apply = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id,
        expected_claim_revision: 1,
    };
    let applied = match authority
        .apply_transition_at(&apply, timestamp("2026-07-13T12:04:10.000000000Z"))
        .unwrap()
    {
        TransitionApplicationOutcomeV1::Applied(intent) => intent,
        other => panic!("unexpected application: {other:?}"),
    };
    let input_ref = applied.transition_input_ref.clone().unwrap();
    let accept = AcceptTransitionInputRequestV1 {
        intent_id: applied.intent_id.clone(),
        issuer_request_id: applied.issuer_request_id.clone(),
        payload_commitment: applied.payload_commitment.clone(),
        expected_intent_revision: applied.intent_revision,
        input_ref,
        run_id: applied.run_id.clone(),
        accepting_participant_id: applied.target_authoritative_participant_id.clone(),
    };
    let accepted = match authority
        .accept_transition_input_at(&accept, timestamp("2026-07-13T12:04:11.000000000Z"))
        .unwrap()
    {
        InputAcceptanceOutcomeV1::Accepted(intent) => intent,
        other => panic!("unexpected acceptance: {other:?}"),
    };
    let accepted_root = authority.read_root().unwrap();
    assert!(matches!(
        authority
            .accept_transition_input_at(&accept, timestamp("2026-07-13T12:04:12.000000000Z"),)
            .unwrap(),
        InputAcceptanceOutcomeV1::Joined(_)
    ));
    assert_eq!(authority.read_root().unwrap(), accepted_root);

    let post_turn = ApplyPostTurnCompletionRequestV1 {
        intent_id: accepted.intent_id.clone(),
        issuer_request_id: accepted.issuer_request_id.clone(),
        payload_commitment: accepted.payload_commitment.clone(),
        expected_intent_revision: accepted.intent_revision,
        expected_authority_revision: parked.authority_revision + 1,
        outcome: PostTurnCompletionOutcomeV1::ResumableClean,
    };
    assert!(matches!(
        authority
            .apply_post_turn_completion_at(&post_turn, timestamp("2026-07-13T12:04:20.000000000Z"),)
            .unwrap(),
        PostTurnCompletionOutcomeResultV1::Applied(_)
    ));
    let completed = authority.read_root().unwrap();
    let SessionNamespaceRecordV1::Authority(final_authority) = completed
        .session_namespace_map
        .get(&request.orchestration_session_id)
        .unwrap()
    else {
        panic!("post-turn lost authority")
    };
    assert_eq!(
        final_authority.authority_revision,
        parked.authority_revision + 2
    );
    assert_eq!(
        final_authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
    );
    assert_eq!(
        final_authority.orchestration_session_id,
        parked.orchestration_session_id
    );
    assert_eq!(final_authority.world_binding, parked.world_binding);
    assert!(matches!(
        authority
            .apply_post_turn_completion_at(&post_turn, timestamp("2026-07-13T12:04:21.000000000Z"),)
            .unwrap(),
        PostTurnCompletionOutcomeResultV1::Joined(_)
    ));
    assert_eq!(authority.read_root().unwrap(), completed);

    let mut conflicting = post_turn;
    conflicting.outcome = PostTurnCompletionOutcomeV1::TerminalFailure;
    assert!(authority
        .apply_post_turn_completion_at(&conflicting, timestamp("2026-07-13T12:04:22.000000000Z"),)
        .is_err());
    assert_eq!(authority.read_root().unwrap(), completed);
}

#[test]
fn retained_transport_reprojects_without_destructive_reads_and_stops_redelivery_after_apply() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let issued = match authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        other => panic!("unexpected issuance: {other:?}"),
    };
    let identity = TransitionIdentityRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
    };
    let first = authority.reproject_transition_handoff(&identity).unwrap();
    assert_eq!(
        first.participant_lease_token,
        Some(request.target_participant_lease_token.clone())
    );
    assert!(first.pending_input.is_none());
    let root = authority.read_root().unwrap();
    let second = authority.reproject_transition_handoff(&identity).unwrap();
    assert_eq!(first, second);
    assert_eq!(authority.read_root().unwrap(), root);

    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-projection".into(),
        claimant_attempt_id: "attempt-projection".into(),
    };
    let claimed = match authority
        .claim_transition_at(&claim, timestamp("2026-07-13T12:01:00.000000000Z"), 30)
        .unwrap()
    {
        TransitionClaimOutcomeV1::Claimed(intent) => intent,
        other => panic!("unexpected claim: {other:?}"),
    };
    authority
        .apply_transition_at(
            &ApplyHostSessionTransitionRequestV1 {
                intent_id: claimed.intent_id.clone(),
                issuer_request_id: claimed.issuer_request_id.clone(),
                payload_commitment: claimed.payload_commitment.clone(),
                expected_intent_revision: claimed.intent_revision,
                claim_id: claim.claim_id,
                expected_claim_revision: 1,
            },
            timestamp("2026-07-13T12:01:10.000000000Z"),
        )
        .unwrap();
    let after_apply = authority.reproject_transition_handoff(&identity).unwrap();
    assert!(after_apply.participant_lease_token.is_none());
    assert!(after_apply.pending_input.is_none());
    assert_eq!(after_apply.transport_payload.intent_id, request.intent_id);
}

#[test]
fn transport_release_crash_windows_reconcile_eligible_delete_and_released_states() {
    for crash_point in [
        TransportReleaseCrashPointV1::EligibleCommitted,
        TransportReleaseCrashPointV1::PayloadDeleted,
        TransportReleaseCrashPointV1::ReleasedCommitted,
    ] {
        let (_parent, authority, binding) = authority();
        let home = binding.authority_store_root.physical_path.clone();
        let request = start_request(binding);
        let application = issue_and_claim_start(&authority, &request);
        let applied = match authority
            .apply_transition_at(&application, timestamp("2026-07-13T12:01:10.000000000Z"))
            .unwrap()
        {
            TransitionApplicationOutcomeV1::Applied(intent) => intent,
            other => panic!("unexpected application: {other:?}"),
        };
        let identity = TransitionIdentityRequestV1 {
            intent_id: applied.intent_id.clone(),
            issuer_request_id: applied.issuer_request_id.clone(),
            payload_commitment: applied.payload_commitment.clone(),
        };
        assert!(authority
            .release_transition_transport_at_with_crash_point(
                &identity,
                timestamp("2026-07-13T12:02:00.000000000Z"),
                crash_point,
            )
            .is_err());
        drop(authority);
        let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
        reopened.read_root().unwrap();
        let outcome = reopened
            .release_transition_transport_at(&identity, timestamp("2026-07-13T12:02:01.000000000Z"))
            .unwrap();
        assert!(matches!(
            outcome,
            TransportReleaseOutcomeV1::Released(_) | TransportReleaseOutcomeV1::Joined(_)
        ));
        let released = reopened.read_root().unwrap();
        assert!(matches!(
            released.transition_intent_map[&identity.intent_id].transport_payload_state,
            super::store_schema::HostSessionTransitionTransportPayloadStateV1::Released { .. }
        ));
        assert!(reopened.reproject_transition_handoff(&identity).is_err());
    }
}

#[test]
fn fabricated_workspace_and_reused_start_identities_fail_without_root_mutation() {
    let (_parent, authority, binding) = authority();
    let first = start_request(binding.clone());
    authority
        .issue_transition_at(&first, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap();
    let committed = authority.read_root().unwrap();

    let mut fabricated = distinct_start_request(binding.clone());
    match &mut fabricated
        .workspace_binding
        .workspace_root
        .physical_identity
    {
        super::schema::DirectoryPhysicalIdentityV1::Linux { inode, .. } => *inode += 1,
        super::schema::DirectoryPhysicalIdentityV1::MacOs { file_id, .. } => *file_id += 1,
    }
    assert!(authority
        .issue_transition_at(
            &fabricated,
            timestamp("2026-07-13T12:01:00.000000000Z"),
            300,
        )
        .is_err());
    assert_eq!(authority.read_root().unwrap(), committed);

    for alias in ["participant", "run", "trace"] {
        let mut conflicting = distinct_start_request(binding.clone());
        match alias {
            "participant" => {
                conflicting.target_authoritative_participant_id =
                    first.target_authoritative_participant_id.clone();
                conflicting.resulting_authoritative_lineage =
                    vec![first.target_authoritative_participant_id.clone()];
            }
            "run" => conflicting.run_id = first.run_id.clone(),
            "trace" => conflicting.shell_trace_session_id = first.shell_trace_session_id.clone(),
            _ => unreachable!(),
        }
        assert!(authority
            .issue_transition_at(
                &conflicting,
                timestamp("2026-07-13T12:01:00.000000000Z"),
                300,
            )
            .is_err());
        assert_eq!(authority.read_root().unwrap(), committed, "alias={alias}");
    }
}

#[test]
fn claim_exact_retry_joins_and_expired_claim_reclaims_without_namespace_mutation() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let issued = match authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        TransitionIssueOutcomeV1::Joined(_) => panic!("first issuance joined"),
    };
    let reservation = authority.read_root().unwrap().session_namespace_map
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
    let before_backdated = authority.read_root().unwrap();
    assert!(authority
        .claim_transition_at(&claim, timestamp("2026-07-13T11:59:59.000000000Z"), 30,)
        .is_err());
    assert_eq!(authority.read_root().unwrap(), before_backdated);
    let first = authority
        .claim_transition_at(&claim, timestamp("2026-07-13T12:01:00.000000000Z"), 30)
        .unwrap();
    let claimed = match first {
        TransitionClaimOutcomeV1::Claimed(intent) => intent,
        other => panic!("unexpected claim result: {other:?}"),
    };
    assert_eq!(claimed.intent_revision, 2);
    assert!(matches!(
        claimed.state,
        HostSessionTransitionIntentStateV1::Claimed {
            ref claim_id,
            claim_revision: 1,
            ..
        } if claim_id == "claim-1"
    ));
    let after_first = authority.read_root().unwrap();
    assert_eq!(
        after_first.session_namespace_map[&request.orchestration_session_id],
        reservation
    );

    assert!(matches!(
        authority
            .claim_transition_at(&claim, timestamp("2026-07-13T12:01:01.000000000Z"), 30,)
            .unwrap(),
        TransitionClaimOutcomeV1::Joined(_)
    ));
    assert_eq!(authority.read_root().unwrap(), after_first);

    let foreign = ClaimHostSessionTransitionRequestV1 {
        expected_intent_revision: claimed.intent_revision,
        claim_id: "claim-2".into(),
        claimant_attempt_id: "attempt-2".into(),
        ..claim.clone()
    };
    assert!(authority
        .claim_transition_at(&foreign, timestamp("2026-07-13T12:01:29.000000000Z"), 30,)
        .is_err());
    assert_eq!(authority.read_root().unwrap(), after_first);

    let expired_same_claim = ClaimHostSessionTransitionRequestV1 {
        expected_intent_revision: claimed.intent_revision,
        ..claim.clone()
    };
    assert!(authority
        .claim_transition_at(
            &expired_same_claim,
            timestamp("2026-07-13T12:01:31.000000000Z"),
            30,
        )
        .is_err());
    assert_eq!(authority.read_root().unwrap(), after_first);

    let reclaimed = match authority
        .claim_transition_at(&foreign, timestamp("2026-07-13T12:01:31.000000000Z"), 30)
        .unwrap()
    {
        TransitionClaimOutcomeV1::Reclaimed(intent) => intent,
        other => panic!("unexpected reclaim result: {other:?}"),
    };
    assert_eq!(reclaimed.intent_revision, 3);
    assert!(matches!(
        reclaimed.state,
        HostSessionTransitionIntentStateV1::Claimed {
            ref claim_id,
            claim_revision: 2,
            ..
        } if claim_id == "claim-2"
    ));
    assert_eq!(
        authority.read_root().unwrap().session_namespace_map[&request.orchestration_session_id],
        reservation
    );
}

fn distinct_start_request(binding: WorkspaceBindingV1) -> IssueHostSessionTransitionRequestV1 {
    let mut request = start_request(binding);
    request.intent_id = "intent-start-2".into();
    request.issuer_request_id = "request-start-2".into();
    request.orchestration_session_id = "session-start-2".into();
    request.shell_trace_session_id = "trace-start-2".into();
    request.target_authoritative_participant_id = "participant-start-2".into();
    request.target_participant_lease_token = b"lease-start-2".to_vec();
    request.run_id = "run-start-2".into();
    request.resulting_authoritative_lineage = vec!["participant-start-2".into()];
    request
}

#[test]
fn fixed_expiry_tombstones_start_and_exact_retry_joins() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.transition_input = Some(b"one startup input".to_vec());
    let issued = match authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 60)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        TransitionIssueOutcomeV1::Joined(_) => panic!("first issuance joined"),
    };
    let expiry = ExpireHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
    };
    let before = authority.read_root().unwrap();
    assert!(authority
        .expire_transition_at(&expiry, timestamp("2026-07-13T12:00:59.999999999Z"),)
        .is_err());
    assert_eq!(authority.read_root().unwrap(), before);

    assert!(matches!(
        authority
            .expire_transition_at(&expiry, timestamp("2026-07-13T12:01:00.000000000Z"))
            .unwrap(),
        TransitionTerminalOutcomeV1::Expired(_)
    ));
    let terminal = authority.read_root().unwrap();
    let intent = &terminal.transition_intent_map[&issued.intent_id];
    let terminal_ref = match &intent.state {
        HostSessionTransitionIntentStateV1::Expired {
            terminal_handoff_ref,
            ..
        } => terminal_handoff_ref,
        other => panic!("unexpected terminal state: {other:?}"),
    };
    assert!(matches!(
        &intent.input_handoff,
        super::store_schema::HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
            terminal_handoff_ref,
            ..
        } if terminal_handoff_ref == terminal_ref
    ));
    assert!(matches!(
        terminal.session_namespace_map.get(&request.orchestration_session_id),
        Some(SessionNamespaceRecordV1::StartTombstone(tombstone))
            if tombstone.terminal_handoff_ref == *terminal_ref
    ));

    assert!(matches!(
        authority
            .expire_transition_at(&expiry, timestamp("2026-07-13T12:30:00.000000000Z"))
            .unwrap(),
        TransitionTerminalOutcomeV1::Joined(_)
    ));
    assert_eq!(authority.read_root().unwrap(), terminal);
}

#[test]
fn restart_joins_claim_reclaim_and_expiry_from_durable_truth() {
    let (_parent, authority, binding) = authority();
    let home = binding.authority_store_root.physical_path.clone();
    let request = start_request(binding);
    let issued = match authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        TransitionIssueOutcomeV1::Joined(_) => panic!("first issuance joined"),
    };
    drop(authority);

    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-restart".into(),
        claimant_attempt_id: "attempt-restart".into(),
    };
    let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
    assert!(matches!(
        reopened
            .claim_transition_at(&claim, timestamp("2026-07-13T12:00:10.000000000Z"), 30,)
            .unwrap(),
        TransitionClaimOutcomeV1::Claimed(_)
    ));
    drop(reopened);

    let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
    assert!(matches!(
        reopened
            .claim_transition_at(&claim, timestamp("2026-07-13T12:00:20.000000000Z"), 30,)
            .unwrap(),
        TransitionClaimOutcomeV1::Joined(_)
    ));
    let current = reopened.read_root().unwrap();
    let expiry = ExpireHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: current.transition_intent_map[&issued.intent_id].intent_revision,
    };
    drop(reopened);

    let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
    assert!(matches!(
        reopened
            .expire_transition_at(&expiry, timestamp("2026-07-13T12:05:00.000000000Z"))
            .unwrap(),
        TransitionTerminalOutcomeV1::Expired(_)
    ));
    let terminal = reopened.read_root().unwrap();
    drop(reopened);
    let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
    assert!(matches!(
        reopened
            .expire_transition_at(&expiry, timestamp("2026-07-13T12:06:00.000000000Z"))
            .unwrap(),
        TransitionTerminalOutcomeV1::Joined(_)
    ));
    assert_eq!(reopened.read_root().unwrap(), terminal);
}

#[test]
fn issuance_crash_windows_leave_only_orphans_or_one_joinable_root() {
    for crash_point in [
        IssueCrashPointV1::DescriptorPublished,
        IssueCrashPointV1::PolicyPublished,
        IssueCrashPointV1::AttachPublished,
        IssueCrashPointV1::LeasePublished,
        IssueCrashPointV1::InputPublished,
        IssueCrashPointV1::TransportPublished,
        IssueCrashPointV1::RootCommitted,
    ] {
        let (_parent, authority, binding) = authority();
        let home = binding.authority_store_root.physical_path.clone();
        let mut request = start_request(binding);
        request.transition_input = Some(b"crash-window input".to_vec());
        let initial = authority.read_root().unwrap();
        assert!(authority
            .issue_transition_at_with_crash_point(
                &request,
                timestamp("2026-07-13T12:00:00.000000000Z"),
                300,
                crash_point,
            )
            .is_err());
        let observed = authority.read_root().unwrap();
        if crash_point == IssueCrashPointV1::RootCommitted {
            assert!(observed
                .transition_intent_map
                .contains_key(&request.intent_id));
        } else {
            assert_eq!(observed, initial, "crash_point={crash_point:?}");
        }
        drop(authority);
        let reopened = HostSessionAuthority::open(Path::new(&home)).unwrap();
        let retried = reopened
            .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
            .unwrap();
        if crash_point == IssueCrashPointV1::RootCommitted {
            assert!(matches!(retried, TransitionIssueOutcomeV1::Joined(_)));
        } else {
            assert!(matches!(retried, TransitionIssueOutcomeV1::Issued(_)));
        }
    }
}

#[test]
fn concurrent_exact_start_issuance_has_one_commit_and_one_join() {
    let (_parent, authority, binding) = authority();
    let home = binding.authority_store_root.physical_path.clone();
    let request = start_request(binding);
    drop(authority);
    let barrier = Arc::new(Barrier::new(2));
    let handles = (0..2)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            let home = home.clone();
            let request = request.clone();
            std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
                barrier.wait();
                authority
                    .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, TransitionIssueOutcomeV1::Issued(_)))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, TransitionIssueOutcomeV1::Joined(_)))
            .count(),
        1
    );
}

#[test]
fn workspace_replacement_after_issuance_blocks_claim_without_mutation() {
    let (parent, authority, mut binding) = authority();
    let workspace = parent.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    fs::set_permissions(&workspace, fs::Permissions::from_mode(0o700)).unwrap();
    let workspace_root = super::trusted_fs::TrustedAuthorityRoot::open(&workspace).unwrap();
    binding.workspace_root = workspace_root.identity().clone();
    drop(workspace_root);
    let request = start_request(binding);
    let issued = match authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 300)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        TransitionIssueOutcomeV1::Joined(_) => panic!("first issuance joined"),
    };
    let before = authority.read_root().unwrap();
    fs::rename(&workspace, parent.path().join("workspace-replaced")).unwrap();
    fs::create_dir(&workspace).unwrap();
    fs::set_permissions(&workspace, fs::Permissions::from_mode(0o700)).unwrap();
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id,
        issuer_request_id: issued.issuer_request_id,
        payload_commitment: issued.payload_commitment,
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-after-workspace-replacement".into(),
        claimant_attempt_id: "attempt-after-workspace-replacement".into(),
    };
    assert!(authority
        .claim_transition_at(&claim, timestamp("2026-07-13T12:01:00.000000000Z"), 30)
        .is_err());
    assert_eq!(authority.read_root().unwrap(), before);
}

#[test]
fn concurrent_exact_claim_and_expiry_each_commit_once_and_join_once() {
    let (_parent, authority, binding) = authority();
    let home = binding.authority_store_root.physical_path.clone();
    let request = start_request(binding);
    let issued = match authority
        .issue_transition_at(&request, timestamp("2026-07-13T12:00:00.000000000Z"), 60)
        .unwrap()
    {
        TransitionIssueOutcomeV1::Issued(intent) => intent,
        TransitionIssueOutcomeV1::Joined(_) => panic!("first issuance joined"),
    };
    drop(authority);
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-concurrent".into(),
        claimant_attempt_id: "attempt-concurrent".into(),
    };
    let barrier = Arc::new(Barrier::new(2));
    let claim_handles = (0..2)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            let home = home.clone();
            let claim = claim.clone();
            std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
                barrier.wait();
                authority
                    .claim_transition_at(&claim, timestamp("2026-07-13T12:00:10.000000000Z"), 30)
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    let claim_outcomes = claim_handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        claim_outcomes
            .iter()
            .filter(|outcome| matches!(outcome, TransitionClaimOutcomeV1::Claimed(_)))
            .count(),
        1
    );
    assert_eq!(
        claim_outcomes
            .iter()
            .filter(|outcome| matches!(outcome, TransitionClaimOutcomeV1::Joined(_)))
            .count(),
        1
    );

    let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
    let root = authority.read_root().unwrap();
    let expiry = ExpireHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id,
        issuer_request_id: issued.issuer_request_id,
        payload_commitment: issued.payload_commitment,
        expected_intent_revision: root.transition_intent_map[&request.intent_id].intent_revision,
    };
    drop(authority);
    let barrier = Arc::new(Barrier::new(2));
    let expiry_handles = (0..2)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            let home = home.clone();
            let expiry = expiry.clone();
            std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
                barrier.wait();
                authority
                    .expire_transition_at(&expiry, timestamp("2026-07-13T12:01:00.000000000Z"))
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    let expiry_outcomes = expiry_handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        expiry_outcomes
            .iter()
            .filter(|outcome| matches!(outcome, TransitionTerminalOutcomeV1::Expired(_)))
            .count(),
        1
    );
    assert_eq!(
        expiry_outcomes
            .iter()
            .filter(|outcome| matches!(outcome, TransitionTerminalOutcomeV1::Joined(_)))
            .count(),
        1
    );
}

#[test]
fn workspace_publication_guard_rejects_rebinding_before_root_replace() {
    let (parent, authority, binding) = authority();
    let workspace_path = parent.path().join("guarded-workspace");
    fs::create_dir(&workspace_path).unwrap();
    fs::set_permissions(&workspace_path, fs::Permissions::from_mode(0o700)).unwrap();
    let workspace_identity = super::trusted_fs::TrustedAuthorityRoot::open(&workspace_path)
        .unwrap()
        .identity()
        .clone();
    let workspace =
        super::trusted_fs::TrustedWorkspaceRoot::open_exact(&workspace_identity).unwrap();
    let authority_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(
        &binding.authority_store_root.physical_path,
    ))
    .unwrap();
    let current = authority.read_root().unwrap();
    let mut proposed = current.clone();
    proposed.root_revision += 1;
    let calls = std::cell::Cell::new(0_u8);
    let result = super::store::compare_and_swap_opened_root_exact_current_guarded(
        &authority_root,
        &current,
        &super::store::ExpectedRevisionsV1 {
            root_revision: current.root_revision,
            authority: None,
        },
        &proposed,
        None,
        || {
            let call = calls.get() + 1;
            calls.set(call);
            if call == 2 {
                fs::rename(
                    &workspace_path,
                    parent.path().join("guarded-workspace-rebound"),
                )
                .unwrap();
                fs::create_dir(&workspace_path).unwrap();
                fs::set_permissions(&workspace_path, fs::Permissions::from_mode(0o700)).unwrap();
            }
            workspace
                .revalidate()
                .map_err(|_| super::store::BootstrapError::transition_guard())
        },
    );
    assert!(result.is_err());
    assert_eq!(calls.get(), 2);
    assert_eq!(authority.read_root().unwrap(), current);
}
