use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::execution::agent_runtime::obligation_ledger::{
    self, ObligationAttentionDispositionV1, ObligationLedgerSnapshotReadV1,
    ObligationMaterializationCutV1,
    ObligationSnapshotHashInputV1 as LedgerObligationSnapshotHashInputV1,
    SupervisorJournalEventRefV1,
};
use crate::execution::agent_runtime::retained_worker_runtime::{
    RetainedWorkerRegistrationPlanV1, RetainedWorkerRuntime,
};
use crate::execution::agent_runtime::state_store::AcceptedWorldWorkIdentityV1;

use super::facade::{
    exact_v3_authority_history_with_reader, AuthorityParticipantRoleV1, HostSessionAuthority,
    ResolvedCurrentAuthorityV1, RetainedWorkerAuthorityPreconditionV1,
};
use super::fork_successor::{
    AllocateForkSuccessorRequestV1, ForkSuccessorAllocationCrashPointV1,
    ForkSuccessorAllocationOutcomeV1,
};
use super::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AgentExecutionScopeV1,
    AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, AuthorityObjectKindV1,
    AuthorityObjectRefV1, DurableSessionAuthorityHashInputV1, HostAttachCapabilitiesV1,
    HostAttachContractHashInputV1, HostAttachExecutionClientStartV1, HostAttachLaunchKnobsV1,
    HostAttachModePreferenceV1, HostPostTurnDispositionV1, HostPostTurnProtocolActorV1,
    HostPostTurnProtocolEventKindV1, HostPostTurnTerminalReasonV1,
    HostSessionAuthorityPreconditionV1, HostSessionPostureV1, HostSessionTransitionCallerKindV1,
    HostSessionTransitionCallerV1, HostSessionTransitionModeV1,
    HostStartupOwnershipProtocolActorV1, HostStartupOwnershipProtocolEventV1,
    PolicyObjectHashInputV1, ResumeHandleHashInputV1, RetainedWorkerObjectHashInputV1,
    RuntimeBackendKindV1, StartupOwnershipOutcomeV1, StartupOwnershipResultHashInputV1,
    TerminalHandoffHashInputV2, TimestampV1, WorkspaceBindingV1, WorldBindingV1,
};
use super::start_continuity::{
    retryable_start_transaction_from_v1, EstablishStartContinuationCrashPointV1,
    EstablishStartContinuationRequestV1, SettleStartTurnCrashPointV1, SettleStartTurnRequestV1,
    StartContinuationOutcomeV1, StartTransactionBeginOutcomeV1, StartTurnCompletionKindV1,
    StartTurnSettlementOutcomeV1,
};
use super::stop::{
    AcceptHostSessionStopDeliveryRequestV1, CompleteHostSessionStopRequestV1,
    HostSessionStopCompletionOutcomeV1, HostSessionStopDeliveryOutcomeV1,
    HostSessionStopIssueOutcomeV1, IssueHostSessionStopRequestV1,
};
use super::store_schema::{
    AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1, DurableSessionAuthorityV1,
    HostSessionPostTurnApplicationV1, HostSessionPostTurnApplicationV2,
    HostSessionStartupOwnershipApplicationV1, HostSessionTransitionInputHandoffV1,
    HostSessionTransitionIntentStateV2, HostSessionTransitionIntentStateV3,
    HostSessionTransitionIntentV2, HostSessionTransitionIntentV3,
    HostSessionTransitionTransportPayloadStateV1,
    RetainedWorkerAuthorityRegistrationRequestStateV1,
    RetainedWorkerAuthorityRegistrationRequestV1, RetainedWorkerAuthorityRegistrationV1,
    SessionNamespaceRecordV1, StartTombstoneStateV1, StartTransactionRecordV1,
    StartTransactionStateV1, StateRootV3,
};
use super::transition::{
    AcceptTransitionInputRequestV1, ApplicationCrashPointV1, ApplyHostSessionTransitionRequestV1,
    ClaimHostSessionTransitionRequestV1, ConsumeObligationSnapshotRequestV1,
    ExpireHostSessionTransitionRequestV1, ExpiryCrashPointV1, InputAcceptanceOutcomeV1,
    IssueCrashPointV1, IssueHostSessionTransitionRequestV1, IssueSuccessorTransitionRequestV1,
    ObligationSnapshotConsumptionOutcomeV1, PostTurnResolutionOutcomeV1, ResolvePostTurnRequestV1,
    ResolveStartupOwnershipRequestV1, StartContractMaterialV1, StartupOwnershipResolutionOutcomeV1,
    SuccessorTransitionApplicationOutcomeV1, SuccessorTransitionClaimOutcomeV1,
    SuccessorTransitionIssueOutcomeV1, TransitionApplicationOutcomeV1, TransitionClaimOutcomeV1,
    TransitionIssueOutcomeV1, TransitionTerminalOutcomeV1,
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

fn opaque_commitment(
    value: &AuthorityObjectCommitmentV1,
) -> substrate_common::OpaqueAuthorityCommitmentV1 {
    match value {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => {
            substrate_common::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: digest_hex.clone(),
            }
        }
        AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id,
            domain,
            digest_hex,
        } => substrate_common::OpaqueAuthorityCommitmentV1::StoreHmacSha256 {
            key_id: key_id.clone(),
            domain: domain.clone(),
            digest_hex: digest_hex.clone(),
        },
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

fn prepare_start_continuity_transaction(
    authority: &HostSessionAuthority,
    request: &IssueHostSessionTransitionRequestV1,
    intent: &HostSessionTransitionIntentV2,
) -> EstablishStartContinuationRequestV1 {
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
        orchestration_session_id: request.orchestration_session_id.clone(),
        intent_id: request.intent_id.clone(),
        issuer_request_id: request.issuer_request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        application_result_ref: application_result_ref.clone(),
        run_id: request.run_id.clone(),
        expected_authority_revision: *authority_revision_after,
        authoritative_participant_id: request.target_authoritative_participant_id.clone(),
        backend_id: request.start_contract.descriptor.backend_id.clone(),
        protocol: request.start_contract.descriptor.protocol.clone(),
        exchange_id: "exchange-start-1".into(),
        exchange_sequence: 1,
        provider_event_kind: "turn.exchange_opened".into(),
        exchange_evidence_sha256: "a".repeat(64),
        internal_uaa_session_id: "uaa-session-start-1".into(),
        observed_at: timestamp("2026-07-14T12:01:20.000000000Z"),
    };
    let transaction = StartTransactionRecordV1 {
        schema_version: 1,
        transaction_id: registration.start_transaction_id.clone(),
        request_key_sha256: registration.request_key_sha256.clone(),
        prompt_sha256: "d".repeat(64),
        authority_store_id: registration.authority_store_id.clone(),
        orchestration_session_id: registration.orchestration_session_id.clone(),
        shell_trace_session_id: request.shell_trace_session_id.clone(),
        authoritative_participant_id: registration.authoritative_participant_id.clone(),
        backend_id: registration.backend_id.clone(),
        protocol: registration.protocol.clone(),
        workspace_root: request
            .workspace_binding
            .workspace_root
            .physical_path
            .clone(),
        world_id: request
            .world_binding
            .as_ref()
            .map(|binding| binding.world_id.clone()),
        world_generation: request
            .world_binding
            .as_ref()
            .map(|binding| binding.world_generation),
        public_backend_id: registration.backend_id.clone(),
        public_scope: if request.world_binding.is_some() {
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
        created_at: timestamp("2026-07-14T12:01:10.000000000Z"),
        updated_at: timestamp("2026-07-14T12:01:10.000000000Z"),
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
            timestamp("2026-07-14T12:01:15.000000000Z"),
        )
        .unwrap();
    registration
}

fn applied_start_for_continuity() -> (
    tempfile::TempDir,
    HostSessionAuthority,
    IssueHostSessionTransitionRequestV1,
    EstablishStartContinuationRequestV1,
) {
    applied_start_for_continuity_with_world(None)
}

fn applied_world_start_for_continuity() -> (
    tempfile::TempDir,
    HostSessionAuthority,
    IssueHostSessionTransitionRequestV1,
    EstablishStartContinuationRequestV1,
) {
    applied_start_for_continuity_with_world(Some(WorldBindingV1 {
        world_id: "world-start-continuity-1".into(),
        world_generation: 7,
    }))
}

fn applied_start_for_continuity_with_world(
    world_binding: Option<WorldBindingV1>,
) -> (
    tempfile::TempDir,
    HostSessionAuthority,
    IssueHostSessionTransitionRequestV1,
    EstablishStartContinuationRequestV1,
) {
    let (parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.world_binding = world_binding;
    let application = issue_and_claim_start(&authority, &request);
    let TransitionApplicationOutcomeV1::Applied(intent) = authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap()
    else {
        panic!("first Start application must commit")
    };
    let registration = prepare_start_continuity_transaction(&authority, &request, &intent);
    (parent, authority, request, registration)
}

fn clean_start_settlement_request(
    registration: &EstablishStartContinuationRequestV1,
    resume_handle_ref: AuthorityObjectRefV1,
    expected_authority_revision: u64,
) -> SettleStartTurnRequestV1 {
    SettleStartTurnRequestV1 {
        start_transaction_id: registration.start_transaction_id.clone(),
        request_key_sha256: registration.request_key_sha256.clone(),
        authority_store_id: registration.authority_store_id.clone(),
        orchestration_session_id: registration.orchestration_session_id.clone(),
        intent_id: registration.intent_id.clone(),
        issuer_request_id: registration.issuer_request_id.clone(),
        payload_commitment: registration.payload_commitment.clone(),
        application_result_ref: registration.application_result_ref.clone(),
        run_id: registration.run_id.clone(),
        authoritative_participant_id: registration.authoritative_participant_id.clone(),
        backend_id: registration.backend_id.clone(),
        protocol: registration.protocol.clone(),
        registered_resume_handle_ref: resume_handle_ref,
        expected_authority_revision,
        protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: registration.authoritative_participant_id.clone(),
        },
        event_id: "event-start-completed-1".into(),
        event_sequence: 2,
        provider_event_kind: "turn.completed".into(),
        thread_id: registration.internal_uaa_session_id.clone(),
        turn_id: "turn-start-1".into(),
        completion_evidence_sha256: "b".repeat(64),
        kind: StartTurnCompletionKindV1::ResumableClean,
        obligation_ledger_read: None,
        completed_at: timestamp("2026-07-14T12:01:30.000000000Z"),
    }
}

fn insert_present_v3(root: &mut StateRootV3, reference: &AuthorityObjectRefV1, byte_length: usize) {
    root.object_index.insert(
        reference.ref_id.clone(),
        super::store_schema::AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: reference.ref_id.clone(),
            object_kind: reference.object_kind,
            object_schema_version: reference.schema_version,
            byte_length: byte_length as u64,
            storage_state: super::store_schema::AuthorityObjectStorageStateV1::Present,
        },
    );
}

fn upgrade_to_detached_v3_authority(
    authority: &HostSessionAuthority,
    request: &IssueHostSessionTransitionRequestV1,
) -> ResolvedCurrentAuthorityV1 {
    let home = request
        .workspace_binding
        .authority_store_root
        .physical_path
        .clone();
    let application = issue_and_claim_start(authority, request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();

    let exact_v2 = authority.read_a12a_root().unwrap();
    let exact_v3 = super::store::upgrade_v2_root_to_v3_test(Path::new(&home), &exact_v2).unwrap();
    let trusted_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(&home)).unwrap();
    let start_authority = match exact_v3.session_namespace_map[&request.orchestration_session_id] {
        SessionNamespaceRecordV1::Authority(ref authority) => authority.as_ref().clone(),
        _ => panic!("expected durable start authority"),
    };
    let start_commitment = canonical_commitment(&authority_hash_input(&start_authority));
    let start_intent = exact_v3.transition_intent_map[&request.intent_id].clone();
    let super::store_schema::HostSessionTransitionIntentStateV2::Applied {
        claim_id,
        claimant_attempt_id,
        application_result_ref,
        ..
    } = &start_intent.state
    else {
        panic!("expected applied start")
    };
    let reconciled_at = timestamp("2026-07-14T12:01:20.000000000Z");
    let evidence_id = "startup-reject-1".to_string();
    let mut detached_authority = start_authority.clone();
    detached_authority.authority_revision = 2;
    detached_authority.lifecycle_posture = HostSessionPostureV1::DetachedReconciled;
    detached_authority.updated_at = reconciled_at.clone();
    let detached_commitment = canonical_commitment(&authority_hash_input(&detached_authority));
    let startup_result = super::schema::StartupOwnershipResultHashInputV1 {
        schema_version: 1,
        evidence: super::schema::HostStartupOwnershipEvidenceV1 {
            schema_version: 1,
            evidence_id: evidence_id.clone(),
            authority_store_id: exact_v3.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            intent_id: request.intent_id.clone(),
            claim_id: claim_id.clone(),
            claimant_attempt_id: claimant_attempt_id.clone(),
            run_id: request.run_id.clone(),
            application_result_ref: application_result_ref.clone(),
            expected_authority_revision: 1,
            active_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            protocol_actor:
                super::schema::HostStartupOwnershipProtocolActorV1::LaunchApplicationClaimant {
                    claim_id: claim_id.clone(),
                    claimant_attempt_id: claimant_attempt_id.clone(),
                },
            protocol_event:
                super::schema::HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected {
                    rejection_id: evidence_id.clone(),
                },
            observed_at: reconciled_at.clone(),
        },
        outcome: super::schema::StartupOwnershipOutcomeV1::TerminalReconciled {
            reason: super::schema::HostStartupTerminalReasonV1::RuntimeCreationRejected,
            authority_revision_after: 2,
            resulting_posture: HostSessionPostureV1::DetachedReconciled,
            authority_record_commitment: detached_commitment.clone(),
        },
        resolved_at: reconciled_at.clone(),
    };
    let startup_result_bytes = super::canonical_json::to_vec(&startup_result).unwrap();
    let startup_result_object = super::store::prepare_generated_object_v3_opened(
        &trusted_root,
        exact_v3.root_revision,
        AuthorityObjectKindV1::StartupOwnershipResult,
        &startup_result_bytes,
        None,
    )
    .unwrap();
    let mut detached_root = exact_v3.clone();
    detached_root.root_revision += 1;
    let super::store_schema::HostSessionTransitionIntentStateV2::Applied {
        startup_ownership, ..
    } = &mut detached_root
        .transition_intent_map
        .get_mut(&request.intent_id)
        .unwrap()
        .state
    else {
        panic!("expected applied start")
    };
    *startup_ownership = Box::new(
        super::store_schema::HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
            evidence_id: evidence_id.clone(),
            result_ref: startup_result_object.reference.clone(),
            authority_revision_before: 1,
            authority_revision_after: 2,
            resulting_posture: HostSessionPostureV1::DetachedReconciled,
            reconciled_at: reconciled_at.clone(),
        },
    );
    detached_root
        .transition_intent_map
        .get_mut(&request.intent_id)
        .unwrap()
        .intent_revision = 4;
    detached_root
        .transition_intent_map
        .get_mut(&request.intent_id)
        .unwrap()
        .updated_at = reconciled_at.clone();
    detached_root.session_namespace_map.insert(
        request.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(detached_authority.clone())),
    );
    detached_root
        .application_journal
        .get_mut(&request.intent_id)
        .unwrap()
        .startup_terminal_application = Some(
        super::store_schema::StartupOwnershipTerminalApplicationJournalV1 {
            schema_version: 1,
            startup_ownership_result_ref: startup_result_object.reference.clone(),
            evidence_id: evidence_id.clone(),
            authority_revision_before: 1,
            authority_record_commitment_before: start_commitment,
            authority_revision_after: 2,
            resulting_posture: HostSessionPostureV1::DetachedReconciled,
            authority_record_commitment_after: detached_commitment,
            applied_at: reconciled_at.clone(),
        },
    );
    insert_present_v3(
        &mut detached_root,
        &startup_result_object.reference,
        startup_result_object.byte_length as usize,
    );
    detached_root.validate().unwrap();
    super::store::commit_v3_root_exact_current_opened(
        &trusted_root,
        &exact_v3,
        &detached_root,
        || Ok(()),
    )
    .unwrap();

    authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap()
}

fn park_with_resume_handle(
    authority: &HostSessionAuthority,
    request: &IssueHostSessionTransitionRequestV1,
) -> (ResolvedCurrentAuthorityV1, AuthorityObjectRefV1) {
    let application = issue_and_claim_start(authority, request);
    let TransitionApplicationOutcomeV1::Applied(intent) = authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap()
    else {
        panic!("first Start application must commit")
    };
    let registration = prepare_start_continuity_transaction(authority, request, &intent);
    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("Start continuation registration must commit")
    };
    let settlement_request = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref,
        registered.authority_revision_after,
    );
    let StartTurnSettlementOutcomeV1::Applied(settled) =
        authority.settle_start_turn(&settlement_request).unwrap()
    else {
        panic!("Start continuation settlement must commit")
    };
    let current = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(
        current.authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
    );
    (current, settled.continuation_resume_handle_ref)
}

fn attach_successor_request(
    current: &ResolvedCurrentAuthorityV1,
    target_participant_id: &str,
) -> IssueSuccessorTransitionRequestV1 {
    let mut lineage = current.authority.authoritative_participant_lineage.clone();
    lineage.push(target_participant_id.to_string());
    IssueSuccessorTransitionRequestV1 {
        intent_id: "intent-attach-1".into(),
        issuer_request_id: "request-attach-1".into(),
        mode: HostSessionTransitionModeV1::Attach,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
            authority_revision: current.observation.authority_revision,
            authority_record_commitment: current.observation.authority_record_commitment.clone(),
            active_authoritative_participant_id: current
                .authority
                .active_authoritative_participant_id
                .clone()
                .unwrap(),
            authoritative_lineage_commitment: current
                .observation
                .authoritative_lineage_commitment
                .clone(),
            lifecycle_posture: current.authority.lifecycle_posture,
        },
        orchestration_session_id: current.authority.orchestration_session_id.clone(),
        shell_trace_session_id: current.authority.shell_trace_session_id.clone(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::Repl,
            caller_participant_id: current
                .authority
                .active_authoritative_participant_id
                .clone(),
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: current
            .authority
            .active_authoritative_participant_id
            .clone(),
        target_authoritative_participant_id: target_participant_id.into(),
        target_participant_lease_token: b"lease-attach-1".to_vec(),
        run_id: "run-attach-1".into(),
        resulting_authoritative_lineage: lineage,
        workspace_binding: current.authority.workspace_binding.clone(),
        world_binding: current.authority.world_binding.clone(),
        resume_handle_ref: None,
        transition_input: None,
        post_turn_disposition: None,
    }
}

fn resume_successor_request(
    current: &ResolvedCurrentAuthorityV1,
    resume_handle_ref: AuthorityObjectRefV1,
    target_participant_id: &str,
) -> IssueSuccessorTransitionRequestV1 {
    let mut lineage = current.authority.authoritative_participant_lineage.clone();
    lineage.push(target_participant_id.to_string());
    IssueSuccessorTransitionRequestV1 {
        intent_id: "intent-resume-1".into(),
        issuer_request_id: "request-resume-1".into(),
        mode: HostSessionTransitionModeV1::ResumeOneTurn,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
            authority_revision: current.observation.authority_revision,
            authority_record_commitment: current.observation.authority_record_commitment.clone(),
            active_authoritative_participant_id: current
                .authority
                .active_authoritative_participant_id
                .clone()
                .unwrap(),
            authoritative_lineage_commitment: current
                .observation
                .authoritative_lineage_commitment
                .clone(),
            lifecycle_posture: current.authority.lifecycle_posture,
        },
        orchestration_session_id: current.authority.orchestration_session_id.clone(),
        shell_trace_session_id: current.authority.shell_trace_session_id.clone(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::Repl,
            caller_participant_id: current
                .authority
                .active_authoritative_participant_id
                .clone(),
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: current
            .authority
            .active_authoritative_participant_id
            .clone(),
        target_authoritative_participant_id: target_participant_id.into(),
        target_participant_lease_token: b"lease-resume-1".to_vec(),
        run_id: "run-resume-1".into(),
        resulting_authoritative_lineage: lineage,
        workspace_binding: current.authority.workspace_binding.clone(),
        world_binding: current.authority.world_binding.clone(),
        resume_handle_ref: Some(resume_handle_ref),
        transition_input: Some(br#"{"input":"resume"}"#.to_vec()),
        post_turn_disposition: Some(HostPostTurnDispositionV1::ReconcileToAttentionParkOrTerminal),
    }
}

fn applied_resume_successor(
    authority: &HostSessionAuthority,
    request: &IssueHostSessionTransitionRequestV1,
) -> IssueSuccessorTransitionRequestV1 {
    let (current, resume_handle_ref) = park_with_resume_handle(authority, request);
    let successor = resume_successor_request(&current, resume_handle_ref, "participant-resume-1");
    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:04:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("first resume issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-resume-1".into(),
        claimant_attempt_id: "attempt-resume-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:04:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("first resume claim must commit")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("resume claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id,
        expected_claim_revision: claim_revision,
    };
    let SuccessorTransitionApplicationOutcomeV1::Applied(_) = authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:04:20.000000000Z"))
        .unwrap()
    else {
        panic!("first resume application must commit")
    };
    successor
}

fn retained_work_identity(target_participant_id: &str) -> AcceptedWorldWorkIdentityV1 {
    AcceptedWorldWorkIdentityV1::RetainedTurn {
        active_run_id: "accepted-run-1".into(),
        message_id: "message-1".into(),
        target_participant_id: target_participant_id.into(),
    }
}

fn transition_correlation(
    intent: &super::store_schema::HostSessionTransitionIntentV3,
    authority_revision_observed: u64,
) -> substrate_common::HostTransitionWorkCorrelationV1 {
    substrate_common::HostTransitionWorkCorrelationV1 {
        schema_version: 1,
        authority_store_id: intent.workspace_binding.authority_store_id.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        transition_intent_id: intent.intent_id.clone(),
        transition_intent_revision_observed: intent.intent_revision,
        transition_run_id: intent.run_id.clone(),
        transition_payload_commitment: opaque_commitment(&intent.payload_commitment),
        authority_revision_observed,
    }
}

fn post_turn_request(
    intent: &super::store_schema::HostSessionTransitionIntentV3,
    authority_revision_observed: u64,
    kind: HostPostTurnProtocolEventKindV1,
    protocol_actor: HostPostTurnProtocolActorV1,
    emitted_at: &str,
    completed_at: &str,
) -> ResolvePostTurnRequestV1 {
    ResolvePostTurnRequestV1 {
        intent_id: intent.intent_id.clone(),
        issuer_request_id: intent.issuer_request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        protocol_actor,
        event_id: "terminal-event-1".into(),
        event_sequence: 9,
        kind,
        emitted_at: timestamp(emitted_at),
        acceptance_record_id: "acceptance-1".into(),
        acceptance_record_revision: 7,
        stream_id: "stream-1".into(),
        accepted_work_identity: retained_work_identity(&intent.target_authoritative_participant_id),
        host_transition_correlation: transition_correlation(intent, authority_revision_observed),
        completed_at: timestamp(completed_at),
    }
}

fn applied_successor_authority_revision(intent: &HostSessionTransitionIntentV3) -> u64 {
    let HostSessionTransitionIntentStateV3::Applied {
        authority_revision_after,
        ..
    } = intent.state
    else {
        panic!("successor transition must be applied")
    };
    authority_revision_after
}

fn pending_ledger_read(
    intent: &super::store_schema::HostSessionTransitionIntentV3,
    authority_revision_observed: u64,
) -> ObligationLedgerSnapshotReadV1 {
    ObligationLedgerSnapshotReadV1::Pending {
        authority_store_id: intent.workspace_binding.authority_store_id.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        acceptance_record_id: "acceptance-1".into(),
        acceptance_record_revision: 7,
        stream_id: "stream-1".into(),
        accepted_work_identity: retained_work_identity(&intent.target_authoritative_participant_id),
        host_transition_correlation: transition_correlation(intent, authority_revision_observed),
        transition_intent_id: intent.intent_id.clone(),
        transition_run_id: intent.run_id.clone(),
        authority_revision_observed,
        observed_session_ledger_revision: 21,
        required_terminal_event_id: "terminal-event-1".into(),
        required_terminal_event_sequence: 9,
    }
}

fn complete_ledger_snapshot(
    intent: &super::store_schema::HostSessionTransitionIntentV3,
    authority_revision_observed: u64,
    attention_disposition: ObligationAttentionDispositionV1,
) -> LedgerObligationSnapshotHashInputV1 {
    let unresolved_attention_obligations = match attention_disposition {
        ObligationAttentionDispositionV1::NoUnresolvedAttention => Vec::new(),
        ObligationAttentionDispositionV1::HasUnresolvedAttention => vec![
            obligation_ledger::UnresolvedAttentionObligationSnapshotEntryV1 {
                obligation_id: "obl-1".into(),
                obligation_revision: 3,
                canonical_record_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "e".repeat(64),
                },
            },
        ],
    };
    LedgerObligationSnapshotHashInputV1 {
        schema_version: 1,
        authority_store_id: intent.workspace_binding.authority_store_id.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        acceptance_record_id: "acceptance-1".into(),
        acceptance_record_revision: 7,
        stream_id: "stream-1".into(),
        accepted_work_identity: retained_work_identity(&intent.target_authoritative_participant_id),
        host_transition_correlation: transition_correlation(intent, authority_revision_observed),
        transition_intent_id: intent.intent_id.clone(),
        transition_run_id: intent.run_id.clone(),
        authority_revision_observed,
        materialization_cut: ObligationMaterializationCutV1 {
            acceptance_record_id: "acceptance-1".into(),
            acceptance_record_revision: 7,
            stream_id: "stream-1".into(),
            session_ledger_revision: 21,
            terminal_event_id: "terminal-event-1".into(),
            terminal_event_sequence: 9,
            materialized_through_event_sequence: 9,
        },
        materialized_journal_events: vec![SupervisorJournalEventRefV1 {
            schema_version: 1,
            stream_id: "stream-1".into(),
            journal_entry_id: "journal-1".into(),
            acceptance_record_id: "acceptance-1".into(),
            acceptance_record_revision: 7,
            accepted_work_identity: retained_work_identity(
                &intent.target_authoritative_participant_id,
            ),
            frame_sequence: 1,
            event_sequence: 9,
            event_id: "terminal-event-1".into(),
            transport_event_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "f".repeat(64),
            },
        }],
        attention_disposition,
        unresolved_attention_obligations,
        captured_at: chrono::DateTime::parse_from_rfc3339("2026-07-14T12:05:00.000000000Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
    }
}

fn start_attention_snapshot(
    registration: &EstablishStartContinuationRequestV1,
    authority_revision_observed: u64,
) -> LedgerObligationSnapshotHashInputV1 {
    let accepted_work_identity = AcceptedWorldWorkIdentityV1::RetainedTurn {
        active_run_id: registration.run_id.clone(),
        message_id: "message-start-1".into(),
        target_participant_id: registration.authoritative_participant_id.clone(),
    };
    let host_transition_correlation = substrate_common::HostTransitionWorkCorrelationV1 {
        schema_version: 1,
        authority_store_id: registration.authority_store_id.clone(),
        orchestration_session_id: registration.orchestration_session_id.clone(),
        authoritative_participant_id: registration.authoritative_participant_id.clone(),
        transition_intent_id: registration.intent_id.clone(),
        transition_intent_revision_observed: 3,
        transition_run_id: registration.run_id.clone(),
        transition_payload_commitment: opaque_commitment(&registration.payload_commitment),
        authority_revision_observed,
    };
    LedgerObligationSnapshotHashInputV1 {
        schema_version: 1,
        authority_store_id: registration.authority_store_id.clone(),
        orchestration_session_id: registration.orchestration_session_id.clone(),
        authoritative_participant_id: registration.authoritative_participant_id.clone(),
        acceptance_record_id: "acceptance-start-1".into(),
        acceptance_record_revision: 1,
        stream_id: "stream-start-1".into(),
        accepted_work_identity: accepted_work_identity.clone(),
        host_transition_correlation: host_transition_correlation.clone(),
        transition_intent_id: registration.intent_id.clone(),
        transition_run_id: registration.run_id.clone(),
        authority_revision_observed,
        materialization_cut: ObligationMaterializationCutV1 {
            acceptance_record_id: "acceptance-start-1".into(),
            acceptance_record_revision: 1,
            stream_id: "stream-start-1".into(),
            session_ledger_revision: 1,
            terminal_event_id: "ledger-terminal-start-1".into(),
            terminal_event_sequence: 41,
            materialized_through_event_sequence: 41,
        },
        materialized_journal_events: vec![SupervisorJournalEventRefV1 {
            schema_version: 1,
            stream_id: "stream-start-1".into(),
            journal_entry_id: "journal-start-1".into(),
            acceptance_record_id: "acceptance-start-1".into(),
            acceptance_record_revision: 1,
            accepted_work_identity,
            frame_sequence: 1,
            event_sequence: 41,
            event_id: "ledger-terminal-start-1".into(),
            transport_event_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "f".repeat(64),
            },
        }],
        attention_disposition: ObligationAttentionDispositionV1::HasUnresolvedAttention,
        unresolved_attention_obligations: vec![
            obligation_ledger::UnresolvedAttentionObligationSnapshotEntryV1 {
                obligation_id: "obligation-start-1".into(),
                obligation_revision: 1,
                canonical_record_commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "e".repeat(64),
                },
            },
        ],
        captured_at: chrono::DateTime::parse_from_rfc3339("2026-07-14T12:01:29.000000000Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
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

#[test]
fn strict_v3_publication_preserves_exact_current_authority_resolution() {
    let (_parent, authority, binding) = authority();
    let home = binding.authority_store_root.physical_path.clone();
    let mut request = start_request(binding.clone());
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-v3-upgrade-1".into(),
        world_generation: 9,
    });
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();

    let exact_v2 = authority.read_a12a_root().unwrap();
    let upgraded = super::store::upgrade_v2_root_to_v3_test(Path::new(&home), &exact_v2).unwrap();
    assert_eq!(
        super::store::upgrade_v2_root_to_v3_test(Path::new(&home), &exact_v2).unwrap(),
        upgraded
    );

    let resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(resolved.observation.root_revision, upgraded.root_revision);
    assert_eq!(resolved.observation.authority_revision, 1);
    assert_eq!(
        resolved.authority.orchestration_session_id,
        request.orchestration_session_id
    );
    assert_eq!(resolved.authority.world_binding, request.world_binding);
    assert_eq!(
        resolved.authority.authoritative_participant_lineage,
        request.resulting_authoritative_lineage
    );
    assert_eq!(
        resolved.caller.role,
        AuthorityParticipantRoleV1::Orchestrator
    );
}

#[test]
fn strict_v3_current_authority_resolution_accepts_detached_start_and_successor_attach() {
    let (_parent, authority, binding) = authority();
    let home = binding.authority_store_root.physical_path.clone();
    let request = start_request(binding.clone());
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();

    let exact_v2 = authority.read_a12a_root().unwrap();
    let exact_v3 = super::store::upgrade_v2_root_to_v3_test(Path::new(&home), &exact_v2).unwrap();
    let trusted_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(&home)).unwrap();
    let insert_present = |root: &mut super::store_schema::StateRootV3,
                          reference: &AuthorityObjectRefV1,
                          byte_length: usize| {
        root.object_index.insert(
            reference.ref_id.clone(),
            super::store_schema::AuthorityObjectIndexEntryV1 {
                schema_version: 1,
                ref_id: reference.ref_id.clone(),
                object_kind: reference.object_kind,
                object_schema_version: reference.schema_version,
                byte_length: byte_length as u64,
                storage_state: super::store_schema::AuthorityObjectStorageStateV1::Present,
            },
        );
    };
    let start_authority = match exact_v3.session_namespace_map[&request.orchestration_session_id] {
        SessionNamespaceRecordV1::Authority(ref authority) => authority.as_ref().clone(),
        _ => panic!("expected durable start authority"),
    };
    let start_commitment = canonical_commitment(&authority_hash_input(&start_authority));
    let start_intent = exact_v3.transition_intent_map[&request.intent_id].clone();
    let super::store_schema::HostSessionTransitionIntentStateV2::Applied {
        claim_id,
        claimant_attempt_id,
        application_result_ref,
        ..
    } = &start_intent.state
    else {
        panic!("expected applied start")
    };
    let reconciled_at = timestamp("2026-07-14T12:01:20.000000000Z");
    let evidence_id = "startup-reject-1".to_string();
    let mut detached_authority = start_authority.clone();
    detached_authority.authority_revision = 2;
    detached_authority.lifecycle_posture = super::schema::HostSessionPostureV1::DetachedReconciled;
    detached_authority.updated_at = reconciled_at.clone();
    let detached_commitment = canonical_commitment(&authority_hash_input(&detached_authority));
    let startup_result = super::schema::StartupOwnershipResultHashInputV1 {
        schema_version: 1,
        evidence: super::schema::HostStartupOwnershipEvidenceV1 {
            schema_version: 1,
            evidence_id: evidence_id.clone(),
            authority_store_id: exact_v3.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            intent_id: request.intent_id.clone(),
            claim_id: claim_id.clone(),
            claimant_attempt_id: claimant_attempt_id.clone(),
            run_id: request.run_id.clone(),
            application_result_ref: application_result_ref.clone(),
            expected_authority_revision: 1,
            active_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            protocol_actor:
                super::schema::HostStartupOwnershipProtocolActorV1::LaunchApplicationClaimant {
                    claim_id: claim_id.clone(),
                    claimant_attempt_id: claimant_attempt_id.clone(),
                },
            protocol_event:
                super::schema::HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected {
                    rejection_id: evidence_id.clone(),
                },
            observed_at: reconciled_at.clone(),
        },
        outcome: super::schema::StartupOwnershipOutcomeV1::TerminalReconciled {
            reason: super::schema::HostStartupTerminalReasonV1::RuntimeCreationRejected,
            authority_revision_after: 2,
            resulting_posture: super::schema::HostSessionPostureV1::DetachedReconciled,
            authority_record_commitment: detached_commitment.clone(),
        },
        resolved_at: reconciled_at.clone(),
    };
    let startup_result_bytes = super::canonical_json::to_vec(&startup_result).unwrap();
    let startup_result_object = super::store::prepare_generated_object_v3_opened(
        &trusted_root,
        exact_v3.root_revision,
        AuthorityObjectKindV1::StartupOwnershipResult,
        &startup_result_bytes,
        None,
    )
    .unwrap();
    let mut detached_root = exact_v3.clone();
    detached_root.root_revision += 1;
    let super::store_schema::HostSessionTransitionIntentStateV2::Applied {
        startup_ownership, ..
    } = &mut detached_root
        .transition_intent_map
        .get_mut(&request.intent_id)
        .unwrap()
        .state
    else {
        panic!("expected applied start")
    };
    *startup_ownership = Box::new(
        super::store_schema::HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
            evidence_id: evidence_id.clone(),
            result_ref: startup_result_object.reference.clone(),
            authority_revision_before: 1,
            authority_revision_after: 2,
            resulting_posture: super::schema::HostSessionPostureV1::DetachedReconciled,
            reconciled_at: reconciled_at.clone(),
        },
    );
    detached_root
        .transition_intent_map
        .get_mut(&request.intent_id)
        .unwrap()
        .intent_revision = 4;
    detached_root
        .transition_intent_map
        .get_mut(&request.intent_id)
        .unwrap()
        .updated_at = reconciled_at.clone();
    detached_root.session_namespace_map.insert(
        request.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(detached_authority.clone())),
    );
    detached_root
        .application_journal
        .get_mut(&request.intent_id)
        .unwrap()
        .startup_terminal_application = Some(
        super::store_schema::StartupOwnershipTerminalApplicationJournalV1 {
            schema_version: 1,
            startup_ownership_result_ref: startup_result_object.reference.clone(),
            evidence_id: evidence_id.clone(),
            authority_revision_before: 1,
            authority_record_commitment_before: start_commitment.clone(),
            authority_revision_after: 2,
            resulting_posture: super::schema::HostSessionPostureV1::DetachedReconciled,
            authority_record_commitment_after: detached_commitment.clone(),
            applied_at: reconciled_at.clone(),
        },
    );
    insert_present(
        &mut detached_root,
        &startup_result_object.reference,
        startup_result_object.byte_length as usize,
    );
    detached_root.validate().unwrap();
    super::store::commit_v3_root_exact_current_opened(
        &trusted_root,
        &exact_v3,
        &detached_root,
        || Ok(()),
    )
    .unwrap();
    let detached_resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(detached_resolved.authority.authority_revision, 2);
    assert_eq!(
        detached_resolved.authority.lifecycle_posture,
        super::schema::HostSessionPostureV1::DetachedReconciled
    );

    let detached_root = authority.read_a12b_root().unwrap();
    let detached_authority =
        match detached_root.session_namespace_map[&request.orchestration_session_id] {
            SessionNamespaceRecordV1::Authority(ref authority) => authority.as_ref().clone(),
            _ => panic!("expected detached authority"),
        };
    let detached_lineage_commitment = canonical_commitment(&AuthoritativeLineageHashInputV1 {
        schema_version: 1,
        orchestration_session_id: detached_authority.orchestration_session_id.clone(),
        participant_ids: detached_authority.authoritative_participant_lineage.clone(),
    });
    let successor_issued_at = timestamp("2026-07-14T12:02:00.000000000Z");
    let successor_applied_at = timestamp("2026-07-14T12:02:10.000000000Z");
    let successor_expires_at = timestamp("2026-07-14T12:07:00.000000000Z");
    let successor_intent_id = "intent-attach-successor-1".to_string();
    let successor_request_id = "request-attach-successor-1".to_string();
    let successor_run_id = "run-attach-successor-1".to_string();
    let successor_participant_id = "participant-successor-1".to_string();
    let object_context = super::store::ObjectVerificationContextV1 {
        intent_id: successor_intent_id.clone(),
        run_id: successor_run_id.clone(),
        parent_intent: None,
    };
    let lease_bytes = b"lease-successor-1".to_vec();
    let lease = super::store::prepare_generated_object_v3_opened(
        &trusted_root,
        detached_root.root_revision,
        AuthorityObjectKindV1::LeaseToken,
        &lease_bytes,
        Some(&object_context),
    )
    .unwrap();
    let start_descriptor_ref = detached_root.transition_intent_map[&request.intent_id]
        .descriptor_ref
        .clone();
    let start_attach_ref = detached_root.transition_intent_map[&request.intent_id]
        .host_attach_contract_ref
        .clone();
    let transport_value = super::schema::TransitionTransportPayloadObjectV1 {
        schema_version: 1,
        intent_id: successor_intent_id.clone(),
        mode: HostSessionTransitionModeV1::Attach,
        orchestration_session_id: request.orchestration_session_id.clone(),
        shell_trace_session_id: request.shell_trace_session_id.clone(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::Repl,
            caller_participant_id: Some(request.target_authoritative_participant_id.clone()),
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        source_authoritative_participant_id: Some(
            request.target_authoritative_participant_id.clone(),
        ),
        target_authoritative_participant_id: successor_participant_id.clone(),
        target_participant_lease_token_ref: lease.reference.clone(),
        run_id: successor_run_id.clone(),
        resulting_authoritative_lineage: vec![
            request.target_authoritative_participant_id.clone(),
            successor_participant_id.clone(),
        ],
        workspace_binding: request.workspace_binding.clone(),
        world_binding: request.world_binding.clone(),
        descriptor_ref: start_descriptor_ref.clone(),
        host_attach_contract_ref: start_attach_ref.clone(),
        resume_handle_ref: None,
        transition_input_ref: None,
        post_turn_disposition: None,
    };
    let transport_bytes = super::canonical_json::to_vec(&transport_value).unwrap();
    let transport_ref = super::store::allocate_sensitive_object_ref_v3_opened(
        &trusted_root,
        detached_root.root_revision,
        AuthorityObjectKindV1::TransitionTransportPayload,
        &transport_bytes,
        &object_context,
    )
    .unwrap();
    let mut successor_authority = detached_authority.clone();
    successor_authority.authority_revision = 3;
    successor_authority.active_authoritative_participant_id =
        Some(successor_participant_id.clone());
    successor_authority
        .authoritative_participant_lineage
        .push(successor_participant_id.clone());
    successor_authority.lifecycle_posture = super::schema::HostSessionPostureV1::ActiveAttached;
    successor_authority.updated_at = successor_applied_at.clone();
    let successor_authority_commitment =
        canonical_commitment(&authority_hash_input(&successor_authority));
    let payload_commitment =
        canonical_commitment(&super::schema::HostSessionTransitionPayloadHashInputV1 {
            schema_version: 1,
            intent_id: successor_intent_id.clone(),
            issuer_request_id: successor_request_id.clone(),
            mode: HostSessionTransitionModeV1::Attach,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision: 2,
                authority_record_commitment: detached_commitment.clone(),
                active_authoritative_participant_id: request
                    .target_authoritative_participant_id
                    .clone(),
                authoritative_lineage_commitment: detached_lineage_commitment.clone(),
                lifecycle_posture: super::schema::HostSessionPostureV1::DetachedReconciled,
            },
            orchestration_session_id: request.orchestration_session_id.clone(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            caller: transport_value.caller.clone(),
            source_authoritative_participant_id: transport_value
                .source_authoritative_participant_id
                .clone(),
            target_authoritative_participant_id: successor_participant_id.clone(),
            target_participant_lease_token_ref: lease.reference.clone(),
            run_id: successor_run_id.clone(),
            resulting_authoritative_lineage: transport_value
                .resulting_authoritative_lineage
                .clone(),
            workspace_binding: request.workspace_binding.clone(),
            world_binding: request.world_binding.clone(),
            descriptor_ref: start_descriptor_ref.clone(),
            host_attach_contract_ref: start_attach_ref.clone(),
            resume_handle_ref: None,
            transition_input_ref: None,
            post_turn_disposition: None,
            transport_payload_ref: transport_ref.clone(),
            issued_at: successor_issued_at.clone(),
            expires_at: successor_expires_at.clone(),
        });
    let successor_application = super::schema::ApplicationResultHashInputV1 {
        schema_version: 1,
        intent_id: successor_intent_id.clone(),
        mode: HostSessionTransitionModeV1::Attach,
        run_id: successor_run_id.clone(),
        phase: super::schema::ApplicationResultPhaseV1::InitialTransition {
            authority_revision_before: Some(2),
            authority_revision_after: 3,
            active_authoritative_participant_id: successor_participant_id.clone(),
            resulting_posture: super::schema::HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: successor_authority_commitment.clone(),
            post_turn_pending_run_id: None,
        },
        applied_at: successor_applied_at.clone(),
    };
    let successor_application_bytes =
        super::canonical_json::to_vec(&successor_application).unwrap();
    let successor_application_object = super::store::prepare_generated_object_v3_opened(
        &trusted_root,
        detached_root.root_revision,
        AuthorityObjectKindV1::ApplicationResult,
        &successor_application_bytes,
        None,
    )
    .unwrap();
    let successor_intent = super::store_schema::HostSessionTransitionIntentV3 {
        schema_version: 3,
        intent_id: successor_intent_id.clone(),
        issuer_request_id: successor_request_id.clone(),
        intent_revision: 3,
        mode: HostSessionTransitionModeV1::Attach,
        authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
            authority_revision: 2,
            authority_record_commitment: detached_commitment.clone(),
            active_authoritative_participant_id: request
                .target_authoritative_participant_id
                .clone(),
            authoritative_lineage_commitment: detached_lineage_commitment.clone(),
            lifecycle_posture: super::schema::HostSessionPostureV1::DetachedReconciled,
        },
        orchestration_session_id: request.orchestration_session_id.clone(),
        shell_trace_session_id: request.shell_trace_session_id.clone(),
        caller: transport_value.caller.clone(),
        source_authoritative_participant_id: transport_value
            .source_authoritative_participant_id
            .clone(),
        target_authoritative_participant_id: successor_participant_id.clone(),
        target_participant_lease_token_ref: lease.reference.clone(),
        run_id: successor_run_id.clone(),
        resulting_authoritative_lineage: transport_value.resulting_authoritative_lineage.clone(),
        workspace_binding: request.workspace_binding.clone(),
        world_binding: request.world_binding.clone(),
        descriptor_ref: start_descriptor_ref.clone(),
        host_attach_contract_ref: start_attach_ref.clone(),
        resume_handle_ref: None,
        transition_input_ref: None,
        post_turn_disposition: None,
        transport_payload_ref: transport_ref.clone(),
        payload_commitment: payload_commitment.clone(),
        issued_at: successor_issued_at.clone(),
        expires_at: successor_expires_at.clone(),
        state: super::store_schema::HostSessionTransitionIntentStateV3::Applied {
            claim_id: "claim-attach-successor-1".into(),
            claimant_attempt_id: "attempt-attach-successor-1".into(),
            authority_revision_before: Some(2),
            authority_revision_after: 3,
            active_authoritative_participant_id: successor_participant_id.clone(),
            resulting_posture: super::schema::HostSessionPostureV1::ActiveAttached,
            authority_record_commitment: successor_authority_commitment.clone(),
            application_result_ref: successor_application_object.reference.clone(),
            startup_ownership: Box::new(
                super::store_schema::HostSessionStartupOwnershipApplicationV1::Pending {
                    expected_run_id: successor_run_id.clone(),
                    expected_authority_revision: 3,
                    expected_active_authoritative_participant_id: successor_participant_id.clone(),
                },
            ),
            post_turn: Box::new(
                super::store_schema::HostSessionPostTurnApplicationV2::NotApplicable,
            ),
            applied_at: successor_applied_at.clone(),
        },
        input_handoff: super::store_schema::HostSessionTransitionInputHandoffV1::NotApplicable,
        transport_payload_state:
            super::store_schema::HostSessionTransitionTransportPayloadStateV1::Retained,
        updated_at: successor_applied_at.clone(),
    };
    let transport_context = super::store::ObjectVerificationContextV1 {
        intent_id: successor_intent_id.clone(),
        run_id: successor_run_id.clone(),
        parent_intent: Some(super::store::VersionedObjectVerificationParentIntentV1::V3(
            Box::new(successor_intent.clone()),
        )),
    };
    super::store::prepare_typed_object_v3_opened(
        &trusted_root,
        detached_root.root_revision,
        &transport_ref,
        &transport_bytes,
        Some(&transport_context),
    )
    .unwrap();
    let mut successor_root = detached_root.clone();
    successor_root.root_revision += 1;
    successor_root.session_namespace_map.insert(
        request.orchestration_session_id.clone(),
        SessionNamespaceRecordV1::Authority(Box::new(successor_authority.clone())),
    );
    successor_root
        .successor_transition_intent_map
        .insert(successor_intent_id.clone(), successor_intent.clone());
    successor_root.successor_issuer_request_index.insert(
        successor_request_id.clone(),
        super::store_schema::IssuerRequestIndexEntryV1 {
            schema_version: 1,
            issuer_request_id: successor_request_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            intent_id: successor_intent_id.clone(),
            payload_commitment: payload_commitment.clone(),
        },
    );
    successor_root.successor_application_journal.insert(
        successor_intent_id.clone(),
        super::store_schema::HostSessionTransitionApplicationJournalV3 {
            schema_version: 3,
            intent_id: successor_intent_id.clone(),
            initial_application: super::store_schema::InitialTransitionApplicationJournalV1 {
                authority_revision_before: Some(2),
                authority_revision_after: 3,
                authority_record_commitment: successor_authority_commitment.clone(),
                application_result_ref: successor_application_object.reference.clone(),
                applied_at: successor_applied_at.clone(),
            },
            startup_terminal_application: None,
            post_turn_application: None,
        },
    );
    insert_present(
        &mut successor_root,
        &lease.reference,
        lease.byte_length as usize,
    );
    insert_present(&mut successor_root, &transport_ref, transport_bytes.len());
    insert_present(
        &mut successor_root,
        &successor_application_object.reference,
        successor_application_object.byte_length as usize,
    );
    successor_root.validate().unwrap();
    super::store::commit_v3_root_exact_current_opened(
        &trusted_root,
        &detached_root,
        &successor_root,
        || Ok(()),
    )
    .unwrap();

    let resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(
        resolved.observation.root_revision,
        successor_root.root_revision
    );
    assert_eq!(resolved.observation.authority_revision, 3);
    assert_eq!(
        resolved
            .authority
            .active_authoritative_participant_id
            .as_deref(),
        Some(successor_participant_id.as_str())
    );
    assert_eq!(
        resolved.authority.lifecycle_posture,
        super::schema::HostSessionPostureV1::ActiveAttached
    );
    assert_eq!(
        resolved.authority.authoritative_participant_lineage,
        vec![
            request.target_authoritative_participant_id.clone(),
            successor_participant_id.clone(),
        ]
    );
    assert_eq!(resolved.caller.participant_id, successor_participant_id);
    assert_eq!(
        resolved.caller.role,
        AuthorityParticipantRoleV1::Orchestrator
    );
    assert_eq!(
        resolved.caller.descriptor,
        request.start_contract.descriptor
    );
    assert_eq!(resolved.current_policy, request.start_contract.policy);
}

#[test]
fn successor_attach_issue_claim_apply_retries_to_one_initial_application() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let current = upgrade_to_detached_v3_authority(&authority, &request);
    let successor = attach_successor_request(&current, "participant-attach-1");

    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:03:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("first successor issuance must commit")
    };
    assert_eq!(
        authority
            .issue_successor_at(
                &current,
                &successor,
                timestamp("2026-07-14T12:03:00.000000000Z"),
                300,
            )
            .unwrap(),
        SuccessorTransitionIssueOutcomeV1::Joined(issued.clone())
    );

    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-attach-1".into(),
        claimant_attempt_id: "attempt-attach-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:03:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("first successor claim must commit")
    };
    assert_eq!(
        authority
            .claim_successor_at(&claim, timestamp("2026-07-14T12:03:11.000000000Z"), 30)
            .unwrap(),
        SuccessorTransitionClaimOutcomeV1::Joined(claimed.clone())
    );

    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("successor claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id.clone(),
        expected_claim_revision: claim_revision,
    };
    let SuccessorTransitionApplicationOutcomeV1::Applied(applied) = authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:03:20.000000000Z"))
        .unwrap()
    else {
        panic!("first successor application must commit")
    };
    assert_eq!(
        authority
            .apply_successor_at(&application, timestamp("2026-07-14T12:03:21.000000000Z"))
            .unwrap(),
        SuccessorTransitionApplicationOutcomeV1::Joined(applied.clone())
    );

    let root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("successor application must retain a durable authority")
    };
    assert_eq!(
        current_authority.authority_revision,
        applied_successor_authority_revision(&applied)
    );
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some("participant-attach-1")
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );
    let HostSessionTransitionIntentStateV3::Applied {
        startup_ownership,
        post_turn,
        ..
    } = &root.successor_transition_intent_map[&successor.intent_id].state
    else {
        panic!("successor intent must be applied")
    };
    assert!(matches!(
        startup_ownership.as_ref(),
        HostSessionStartupOwnershipApplicationV1::Pending {
            expected_run_id,
            expected_authority_revision: 3,
            expected_active_authoritative_participant_id,
        } if expected_run_id == "run-attach-1"
            && expected_active_authoritative_participant_id == "participant-attach-1"
    ));
    assert_eq!(
        post_turn.as_ref(),
        &HostSessionPostTurnApplicationV2::NotApplicable
    );
    assert_eq!(
        &root.successor_transition_intent_map[&successor.intent_id].transport_payload_state,
        &HostSessionTransitionTransportPayloadStateV1::Retained
    );
}

#[test]
fn successor_attach_issue_rejects_preserved_start_identity_collisions() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let current = upgrade_to_detached_v3_authority(&authority, &request);
    let root_before_rejection = authority.read_a12b_root().unwrap();

    let mut colliding_intent = attach_successor_request(&current, "participant-attach-1");
    colliding_intent.intent_id = request.intent_id.clone();
    colliding_intent.issuer_request_id = "request-attach-collide-intent".into();
    assert_eq!(
        authority
            .issue_successor_at(
                &current,
                &colliding_intent,
                timestamp("2026-07-14T12:04:00.000000000Z"),
                300,
            )
            .unwrap_err()
            .to_string(),
        "successor issuance identity collides with committed Start intent"
    );
    assert_eq!(authority.read_a12b_root().unwrap(), root_before_rejection);

    let mut colliding_request = attach_successor_request(&current, "participant-attach-2");
    colliding_request.intent_id = "intent-attach-collide-request".into();
    colliding_request.issuer_request_id = request.issuer_request_id.clone();
    colliding_request.run_id = "run-attach-collide-request".into();
    assert_eq!(
        authority
            .issue_successor_at(
                &current,
                &colliding_request,
                timestamp("2026-07-14T12:04:10.000000000Z"),
                300,
            )
            .unwrap_err()
            .to_string(),
        "successor issuance identity collides with committed Start intent"
    );
    assert_eq!(authority.read_a12b_root().unwrap(), root_before_rejection);
}

#[test]
fn successor_attach_startup_acceptance_preserves_current_authority_and_retries() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let current = upgrade_to_detached_v3_authority(&authority, &request);
    let successor = attach_successor_request(&current, "participant-attach-1");

    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:05:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("first successor issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-attach-startup-1".into(),
        claimant_attempt_id: "attempt-attach-startup-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:05:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("first successor claim must commit")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("successor claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id.clone(),
        expected_claim_revision: claim_revision,
    };
    authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:05:20.000000000Z"))
        .unwrap();

    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: successor.intent_id.clone(),
        issuer_request_id: successor.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: "participant-attach-1".into(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id: "startup-accept-attach-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:05:30.000000000Z"),
    };
    let StartupOwnershipResolutionOutcomeV1::ResolvedSuccessor(resolved) = authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:05:31.000000000Z"))
        .unwrap()
    else {
        panic!("first startup acceptance must commit")
    };
    assert_eq!(
        authority
            .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:05:31.000000000Z"),)
            .unwrap(),
        StartupOwnershipResolutionOutcomeV1::JoinedSuccessor(resolved.clone())
    );

    let root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("startup acceptance must retain a durable authority")
    };
    assert_eq!(current_authority.authority_revision, 3);
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some("participant-attach-1")
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );
    let journal = &root.successor_application_journal[&successor.intent_id];
    assert!(journal.startup_terminal_application.is_none());
    let HostSessionTransitionIntentStateV3::Applied {
        startup_ownership, ..
    } = &root.successor_transition_intent_map[&successor.intent_id].state
    else {
        panic!("resolved successor intent must remain applied")
    };
    let HostSessionStartupOwnershipApplicationV1::Accepted {
        evidence_id,
        result_ref,
        authority_revision,
        ..
    } = startup_ownership.as_ref()
    else {
        panic!("startup ownership must be accepted")
    };
    assert_eq!(evidence_id, "startup-accept-attach-1");
    assert_eq!(*authority_revision, 3);
    let trusted_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(
        &root.bootstrap_home.physical_path,
    ))
    .unwrap();
    let result_bytes = super::store::read_typed_object_v2_or_v3_opened(
        &trusted_root,
        root.root_revision,
        result_ref,
        None,
    )
    .unwrap();
    let result: StartupOwnershipResultHashInputV1 =
        super::canonical_json::from_slice(&result_bytes).unwrap();
    assert_eq!(result.outcome, StartupOwnershipOutcomeV1::Accepted);
    assert_eq!(
        result.evidence.protocol_event,
        HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id: "startup-accept-attach-1".into(),
        }
    );
}

#[test]
fn strict_v3_reopen_rejects_successor_lineage_drift() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let current = upgrade_to_detached_v3_authority(&authority, &request);
    let successor = attach_successor_request(&current, "participant-attach-1");

    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:05:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("first successor issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-attach-lineage-drift-1".into(),
        claimant_attempt_id: "attempt-attach-lineage-drift-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:05:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("successor claim must commit")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("successor claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id.clone(),
        expected_claim_revision: claim_revision,
    };
    authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:05:20.000000000Z"))
        .unwrap();

    let mut tampered_root = authority.read_a12b_root().unwrap();
    tampered_root
        .successor_transition_intent_map
        .get_mut(&successor.intent_id)
        .unwrap()
        .resulting_authoritative_lineage = vec![successor.target_authoritative_participant_id];
    let authority_root_path =
        Path::new(&request.workspace_binding.authority_store_root.physical_path);
    fs::write(
        authority_root_path.join("authority-v1/state-root-v1.json"),
        super::store_schema::VersionedStateRoot::V3(tampered_root)
            .to_canonical_bytes()
            .unwrap(),
    )
    .unwrap();

    let reopened = super::trusted_fs::TrustedAuthorityRoot::open(authority_root_path).unwrap();
    assert!(super::store::read_opened_root_v2_or_v3(&reopened).is_err());
}

#[test]
fn successor_attach_startup_retry_rejects_changed_resolution_metadata() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let current = upgrade_to_detached_v3_authority(&authority, &request);
    let successor = attach_successor_request(&current, "participant-attach-1");

    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:05:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("successor issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-attach-startup-retry-1".into(),
        claimant_attempt_id: "attempt-attach-startup-retry-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:05:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("successor claim must commit")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("successor claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id.clone(),
        expected_claim_revision: claim_revision,
    };
    authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:05:20.000000000Z"))
        .unwrap();

    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: successor.intent_id.clone(),
        issuer_request_id: successor.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: "participant-attach-1".into(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id: "startup-accept-attach-retry-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:05:30.000000000Z"),
    };
    authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:05:31.000000000Z"))
        .unwrap();

    let mut changed_observed = resolution.clone();
    changed_observed.observed_at = timestamp("2026-07-14T12:05:30.000000001Z");
    assert!(authority
        .resolve_startup_ownership_at(
            &changed_observed,
            timestamp("2026-07-14T12:05:31.000000000Z"),
        )
        .is_err());

    assert!(authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:05:31.000000001Z"),)
        .is_err());
}

#[test]
fn successor_attach_startup_terminal_reconciliation_detaches_and_retries() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let current = upgrade_to_detached_v3_authority(&authority, &request);
    let successor = attach_successor_request(&current, "participant-attach-1");

    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:06:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("first successor issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-attach-startup-terminal-1".into(),
        claimant_attempt_id: "attempt-attach-startup-terminal-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:06:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("first successor claim must commit")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("successor claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id.clone(),
        expected_claim_revision: claim_revision,
    };
    authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:06:20.000000000Z"))
        .unwrap();

    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: successor.intent_id.clone(),
        issuer_request_id: successor.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::LaunchApplicationClaimant {
            claim_id: claim.claim_id.clone(),
            claimant_attempt_id: claim.claimant_attempt_id.clone(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected {
            rejection_id: "startup-reject-attach-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:06:30.000000000Z"),
    };
    let StartupOwnershipResolutionOutcomeV1::ResolvedSuccessor(resolved) = authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:06:31.000000000Z"))
        .unwrap()
    else {
        panic!("first terminal startup reconciliation must commit")
    };
    assert_eq!(
        authority
            .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:06:31.000000000Z"),)
            .unwrap(),
        StartupOwnershipResolutionOutcomeV1::JoinedSuccessor(resolved.clone())
    );

    let root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("terminal startup reconciliation must retain a durable authority")
    };
    assert_eq!(current_authority.authority_revision, 4);
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some("participant-attach-1")
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::DetachedReconciled
    );
    let journal = &root.successor_application_journal[&successor.intent_id];
    assert!(journal.startup_terminal_application.is_some());
    let intent = &root.successor_transition_intent_map[&successor.intent_id];
    let HostSessionTransitionIntentStateV3::Applied {
        startup_ownership, ..
    } = &intent.state
    else {
        panic!("resolved successor intent must remain applied")
    };
    let HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
        evidence_id,
        result_ref,
        authority_revision_before,
        authority_revision_after,
        resulting_posture,
        ..
    } = startup_ownership.as_ref()
    else {
        panic!("startup ownership must be terminally reconciled")
    };
    assert_eq!(evidence_id, "startup-reject-attach-1");
    assert_eq!(*authority_revision_before, 3);
    assert_eq!(*authority_revision_after, 4);
    assert_eq!(*resulting_posture, HostSessionPostureV1::DetachedReconciled);
    assert!(matches!(
        intent.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible { .. }
    ));
    let trusted_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(
        &root.bootstrap_home.physical_path,
    ))
    .unwrap();
    let result_bytes = super::store::read_typed_object_v2_or_v3_opened(
        &trusted_root,
        root.root_revision,
        result_ref,
        None,
    )
    .unwrap();
    let result: StartupOwnershipResultHashInputV1 =
        super::canonical_json::from_slice(&result_bytes).unwrap();
    assert_eq!(
        result.outcome,
        StartupOwnershipOutcomeV1::TerminalReconciled {
            reason: super::schema::HostStartupTerminalReasonV1::RuntimeCreationRejected,
            authority_revision_after: 4,
            resulting_posture: HostSessionPostureV1::DetachedReconciled,
            authority_record_commitment: canonical_commitment(&authority_hash_input(
                current_authority
            )),
        }
    );
}

#[test]
fn reopen_rejects_v3_successor_attach_terminal_handoff_without_startup_result_ref() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let current = upgrade_to_detached_v3_authority(&authority, &request);
    let successor = attach_successor_request(&current, "participant-attach-1");

    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:06:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("successor issuance must commit")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-attach-startup-reopen-1".into(),
        claimant_attempt_id: "attempt-attach-startup-reopen-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:06:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("successor claim must commit")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("successor claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id.clone(),
        expected_claim_revision: claim_revision,
    };
    authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:06:20.000000000Z"))
        .unwrap();
    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: successor.intent_id.clone(),
        issuer_request_id: successor.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::LaunchApplicationClaimant {
            claim_id: claim.claim_id.clone(),
            claimant_attempt_id: claim.claimant_attempt_id.clone(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::RuntimeCreationRejected {
            rejection_id: "startup-reject-attach-reopen-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:06:30.000000000Z"),
    };
    authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:06:31.000000000Z"))
        .unwrap();

    let root = authority.read_a12b_root().unwrap();
    let trusted_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(
        &root.bootstrap_home.physical_path,
    ))
    .unwrap();
    let intent = &root.successor_transition_intent_map[&successor.intent_id];
    let HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
        terminal_handoff_ref,
    } = &intent.transport_payload_state
    else {
        panic!("terminalized Attach must retain its terminal handoff")
    };
    let terminal_bytes = super::store::read_typed_object_v2_or_v3_opened(
        &trusted_root,
        root.root_revision,
        terminal_handoff_ref,
        None,
    )
    .unwrap();
    let mut terminal: TerminalHandoffHashInputV2 =
        super::canonical_json::from_slice(&terminal_bytes).unwrap();
    terminal.startup_ownership_result_ref = None;
    let corrupted_terminal_bytes = super::canonical_json::to_vec(&terminal).unwrap();
    let corrupted_commitment = canonical_commitment(&terminal);

    let mut corrupted_root = root.clone();
    let mut corrupted_terminal_ref = terminal_handoff_ref.clone();
    corrupted_terminal_ref.commitment = corrupted_commitment;
    let successor_intent = corrupted_root
        .successor_transition_intent_map
        .get_mut(&successor.intent_id)
        .unwrap();
    successor_intent.transport_payload_state =
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
            terminal_handoff_ref: corrupted_terminal_ref.clone(),
        };
    corrupted_root
        .object_index
        .get_mut(&successor_intent.transport_payload_ref.ref_id)
        .unwrap()
        .storage_state = super::store_schema::AuthorityObjectStorageStateV1::ReleaseEligible {
        terminal_handoff_ref: corrupted_terminal_ref.clone(),
    };
    corrupted_root
        .object_index
        .get_mut(&corrupted_terminal_ref.ref_id)
        .unwrap()
        .byte_length = corrupted_terminal_bytes.len() as u64;

    let home = Path::new(&corrupted_root.bootstrap_home.physical_path);
    let object_path = home.join(format!(
        "authority-v1/objects/terminal-handoff/v1/{}.obj",
        corrupted_terminal_ref.ref_id
    ));
    fs::write(&object_path, &corrupted_terminal_bytes).unwrap();
    fs::write(
        home.join("authority-v1/state-root-v1.json"),
        super::canonical_json::to_vec(&corrupted_root).unwrap(),
    )
    .unwrap();

    let reopened = super::trusted_fs::TrustedAuthorityRoot::open(home).unwrap();
    assert!(super::store::read_opened_root_v2_or_v3(&reopened).is_err());
}

#[test]
fn strict_v3_startup_acceptance_preserves_applied_start_and_retries() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let exact_v2 = authority.read_a12a_root().unwrap();
    let exact_v3 = super::store::upgrade_v2_root_to_v3_test(
        Path::new(&request.workspace_binding.authority_store_root.physical_path),
        &exact_v2,
    )
    .unwrap();
    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: request.intent_id.clone(),
        issuer_request_id: request.issuer_request_id.clone(),
        payload_commitment: exact_v3.transition_intent_map[&request.intent_id]
            .payload_commitment
            .clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: request.target_authoritative_participant_id.clone(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id: "startup-accept-start-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:01:20.000000000Z"),
    };
    let StartupOwnershipResolutionOutcomeV1::ResolvedStart(resolved) = authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:01:21.000000000Z"))
        .unwrap()
    else {
        panic!("first preserved Start startup acceptance must commit")
    };
    assert_eq!(
        authority
            .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:01:21.000000000Z"),)
            .unwrap(),
        StartupOwnershipResolutionOutcomeV1::JoinedStart(resolved.clone())
    );

    let root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("accepted Start must retain current authority")
    };
    assert_eq!(current_authority.authority_revision, 1);
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some(request.target_authoritative_participant_id.as_str())
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );
    let journal = &root.application_journal[&request.intent_id];
    assert!(journal.startup_terminal_application.is_none());
    let HostSessionTransitionIntentStateV2::Applied {
        startup_ownership, ..
    } = &root.transition_intent_map[&request.intent_id].state
    else {
        panic!("preserved Start must remain applied")
    };
    let HostSessionStartupOwnershipApplicationV1::Accepted {
        evidence_id,
        result_ref,
        authority_revision,
        ..
    } = startup_ownership.as_ref()
    else {
        panic!("preserved Start startup must be accepted")
    };
    assert_eq!(evidence_id, "startup-accept-start-1");
    assert_eq!(*authority_revision, 1);
    let trusted_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(
        &root.bootstrap_home.physical_path,
    ))
    .unwrap();
    let result_bytes = super::store::read_typed_object_v2_or_v3_opened(
        &trusted_root,
        root.root_revision,
        result_ref,
        None,
    )
    .unwrap();
    let result: StartupOwnershipResultHashInputV1 =
        super::canonical_json::from_slice(&result_bytes).unwrap();
    assert_eq!(result.outcome, StartupOwnershipOutcomeV1::Accepted);
    assert_eq!(result.evidence.expected_authority_revision, 1);
    assert_eq!(
        result.evidence.protocol_event,
        HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id: "startup-accept-start-1".into(),
        }
    );
}

#[test]
fn strict_v3_startup_retry_rejects_changed_start_resolution_metadata() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let exact_v2 = authority.read_a12a_root().unwrap();
    let exact_v3 = super::store::upgrade_v2_root_to_v3_test(
        Path::new(&request.workspace_binding.authority_store_root.physical_path),
        &exact_v2,
    )
    .unwrap();
    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: request.intent_id.clone(),
        issuer_request_id: request.issuer_request_id.clone(),
        payload_commitment: exact_v3.transition_intent_map[&request.intent_id]
            .payload_commitment
            .clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: request.target_authoritative_participant_id.clone(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id: "startup-accept-start-retry-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:01:20.000000000Z"),
    };
    authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:01:21.000000000Z"))
        .unwrap();

    let mut changed_event = resolution.clone();
    changed_event.protocol_event = HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
        ownership_acknowledgement_id: "startup-accept-start-retry-2".into(),
    };
    assert!(authority
        .resolve_startup_ownership_at(&changed_event, timestamp("2026-07-14T12:01:21.000000000Z"),)
        .is_err());

    let mut changed_observed = resolution.clone();
    changed_observed.observed_at = timestamp("2026-07-14T12:01:20.000000001Z");
    assert!(authority
        .resolve_startup_ownership_at(
            &changed_observed,
            timestamp("2026-07-14T12:01:21.000000000Z"),
        )
        .is_err());

    assert!(authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:01:21.000000001Z"),)
        .is_err());
}

#[test]
fn strict_v3_startup_acceptance_preserves_r0_descendant_current_authority_resolution() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-start-accept-1".into(),
        world_generation: 1,
    });
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let root_before_r0 = authority.read_a12a_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root_before_r0.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("applied Start must have current authority")
    };
    let expected_authority = RetainedWorkerAuthorityPreconditionV1 {
        authority_store_id: root_before_r0.authority_store_id.clone(),
        authority_revision: current_authority.authority_revision,
        authority_record_commitment: canonical_commitment(&authority_hash_input(current_authority)),
    };
    let descriptor_bytes = super::canonical_json::to_vec(&AgentDescriptorHashInputV1 {
        schema_version: 1,
        descriptor: request.start_contract.descriptor.clone(),
    })
    .unwrap();
    let retained_participant_id = "participant-retained-accept-1".to_string();
    let session_id = request.orchestration_session_id.clone();
    let resume_handle_bytes = super::canonical_json::to_vec(&ResumeHandleHashInputV1 {
        schema_version: 1,
        orchestration_session_id: session_id.clone(),
        participant_id: retained_participant_id.clone(),
        backend_id: request.start_contract.descriptor.backend_id.clone(),
        protocol: request.start_contract.descriptor.protocol.clone(),
        internal_uaa_session_id: "uaa-retained-accept-1".into(),
    })
    .unwrap();
    let retained_participant_id_for_worker = retained_participant_id.clone();
    let session_id_for_worker = session_id.clone();
    let reserved = authority
        .reserve_retained_worker_registration_at(
            "r0-request-startup-accept-1",
            &session_id,
            &expected_authority,
            &retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
            timestamp("2026-07-14T12:01:30.000000000Z"),
            None,
            move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                super::canonical_json::to_vec(&RetainedWorkerObjectHashInputV1 {
                    schema_version: 1,
                    orchestration_session_id: session_id_for_worker.clone(),
                    participant_id: retained_participant_id_for_worker.clone(),
                    world_binding: world_binding.clone(),
                    descriptor_ref: descriptor_ref.clone(),
                    resume_handle_ref: resume_handle_ref.clone(),
                    policy_ref: policy_ref.clone(),
                })
                .map_err(|_| "serialize retained worker")
            },
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.descriptor_ref,
            &reserved.descriptor_bytes,
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.resume_handle_ref,
            &reserved.resume_handle_bytes,
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.retained_worker_ref,
            &reserved.retained_worker_bytes,
        )
        .unwrap();
    authority
        .apply_reserved_retained_worker_registration(&reserved)
        .unwrap();

    let root_after_r0 = authority.read_a12a_root().unwrap();
    super::store::upgrade_v2_root_to_v3_test(
        Path::new(&request.workspace_binding.authority_store_root.physical_path),
        &root_after_r0,
    )
    .unwrap();
    let root_after_upgrade = authority.read_a12b_root().unwrap();
    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: request.intent_id.clone(),
        issuer_request_id: request.issuer_request_id.clone(),
        payload_commitment: root_after_upgrade.transition_intent_map[&request.intent_id]
            .payload_commitment
            .clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: request.target_authoritative_participant_id.clone(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
            ownership_acknowledgement_id: "startup-accept-start-r0-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:01:40.000000000Z"),
    };
    authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:01:41.000000000Z"))
        .unwrap();

    let resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(resolved.observation.authority_revision, 2);
    assert_eq!(resolved.authority.authority_revision, 2);
    assert_eq!(
        resolved
            .authority
            .authoritative_participant_lineage
            .last()
            .map(String::as_str),
        Some(retained_participant_id.as_str())
    );
    assert_eq!(resolved.authority.retained_worker_refs.len(), 1);
}

#[test]
fn strict_v3_current_authority_resolution_rejects_unproven_r0_descendant_mutation() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-start-accept-invalid-r0".into(),
        world_generation: 1,
    });
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let root_before_r0 = authority.read_a12a_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root_before_r0.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("applied Start must have current authority")
    };
    let expected_authority = RetainedWorkerAuthorityPreconditionV1 {
        authority_store_id: root_before_r0.authority_store_id.clone(),
        authority_revision: current_authority.authority_revision,
        authority_record_commitment: canonical_commitment(&authority_hash_input(current_authority)),
    };
    let descriptor_bytes = super::canonical_json::to_vec(&AgentDescriptorHashInputV1 {
        schema_version: 1,
        descriptor: request.start_contract.descriptor.clone(),
    })
    .unwrap();
    let retained_participant_id = "participant-retained-invalid-r0".to_string();
    let session_id = request.orchestration_session_id.clone();
    let resume_handle_bytes = super::canonical_json::to_vec(&ResumeHandleHashInputV1 {
        schema_version: 1,
        orchestration_session_id: session_id.clone(),
        participant_id: retained_participant_id.clone(),
        backend_id: request.start_contract.descriptor.backend_id.clone(),
        protocol: request.start_contract.descriptor.protocol.clone(),
        internal_uaa_session_id: "uaa-retained-invalid-r0".into(),
    })
    .unwrap();
    let retained_participant_id_for_worker = retained_participant_id.clone();
    let session_id_for_worker = session_id.clone();
    let reserved = authority
        .reserve_retained_worker_registration_at(
            "r0-request-invalid-r0-1",
            &session_id,
            &expected_authority,
            &retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
            timestamp("2026-07-14T12:01:30.000000000Z"),
            None,
            move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                super::canonical_json::to_vec(&RetainedWorkerObjectHashInputV1 {
                    schema_version: 1,
                    orchestration_session_id: session_id_for_worker.clone(),
                    participant_id: retained_participant_id_for_worker.clone(),
                    world_binding: world_binding.clone(),
                    descriptor_ref: descriptor_ref.clone(),
                    resume_handle_ref: resume_handle_ref.clone(),
                    policy_ref: policy_ref.clone(),
                })
                .map_err(|_| "serialize retained worker")
            },
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.descriptor_ref,
            &reserved.descriptor_bytes,
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.resume_handle_ref,
            &reserved.resume_handle_bytes,
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.retained_worker_ref,
            &reserved.retained_worker_bytes,
        )
        .unwrap();
    authority
        .apply_reserved_retained_worker_registration(&reserved)
        .unwrap();
    super::store::upgrade_v2_root_to_v3_test(
        Path::new(&request.workspace_binding.authority_store_root.physical_path),
        &authority.read_a12a_root().unwrap(),
    )
    .unwrap();

    let mut tampered_root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(tampered_authority) = tampered_root
        .session_namespace_map
        .get_mut(&request.orchestration_session_id)
        .expect("R0 descendant must have current authority")
    else {
        panic!("expected current authority record")
    };
    tampered_authority.lifecycle_posture = HostSessionPostureV1::AwaitingAttention;
    let authority_root_path =
        Path::new(&request.workspace_binding.authority_store_root.physical_path);
    fs::write(
        authority_root_path.join("authority-v1/state-root-v1.json"),
        super::store_schema::VersionedStateRoot::V3(tampered_root)
            .to_canonical_bytes()
            .unwrap(),
    )
    .unwrap();

    assert!(authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .is_err());
}

#[test]
fn strict_v3_startup_terminal_reconciliation_advances_exact_r0_descendant_start() {
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
    request.transition_input = Some(b"startup-input-1".to_vec());
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let root_before_r0 = authority.read_a12a_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root_before_r0.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("applied Start must have current authority")
    };
    let expected_authority = RetainedWorkerAuthorityPreconditionV1 {
        authority_store_id: root_before_r0.authority_store_id.clone(),
        authority_revision: current_authority.authority_revision,
        authority_record_commitment: canonical_commitment(&authority_hash_input(current_authority)),
    };
    let descriptor_bytes = super::canonical_json::to_vec(&AgentDescriptorHashInputV1 {
        schema_version: 1,
        descriptor: request.start_contract.descriptor.clone(),
    })
    .unwrap();
    let retained_participant_id = "participant-retained-1".to_string();
    let session_id = request.orchestration_session_id.clone();
    let resume_handle_bytes = super::canonical_json::to_vec(&ResumeHandleHashInputV1 {
        schema_version: 1,
        orchestration_session_id: session_id.clone(),
        participant_id: retained_participant_id.clone(),
        backend_id: request.start_contract.descriptor.backend_id.clone(),
        protocol: request.start_contract.descriptor.protocol.clone(),
        internal_uaa_session_id: "uaa-retained-1".into(),
    })
    .unwrap();
    let retained_participant_id_for_worker = retained_participant_id.clone();
    let session_id_for_worker = session_id.clone();
    let reserved = authority
        .reserve_retained_worker_registration_at(
            "r0-request-startup-1",
            &session_id,
            &expected_authority,
            &retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
            timestamp("2026-07-14T12:01:30.000000000Z"),
            None,
            move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                super::canonical_json::to_vec(&RetainedWorkerObjectHashInputV1 {
                    schema_version: 1,
                    orchestration_session_id: session_id_for_worker.clone(),
                    participant_id: retained_participant_id_for_worker.clone(),
                    world_binding: world_binding.clone(),
                    descriptor_ref: descriptor_ref.clone(),
                    resume_handle_ref: resume_handle_ref.clone(),
                    policy_ref: policy_ref.clone(),
                })
                .map_err(|_| "serialize retained worker")
            },
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.descriptor_ref,
            &reserved.descriptor_bytes,
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.resume_handle_ref,
            &reserved.resume_handle_bytes,
        )
        .unwrap();
    authority
        .publish_reserved_retained_object(
            &reserved,
            &reserved.retained_worker_ref,
            &reserved.retained_worker_bytes,
        )
        .unwrap();
    let applied_registration = authority
        .apply_reserved_retained_worker_registration(&reserved)
        .unwrap();
    assert!(!applied_registration.joined);

    let root_after_r0 = authority.read_a12a_root().unwrap();
    let SessionNamespaceRecordV1::Authority(descendant_authority) =
        &root_after_r0.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("R0 must retain a descendant authority")
    };
    assert_eq!(descendant_authority.authority_revision, 2);
    assert_eq!(
        descendant_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some(request.target_authoritative_participant_id.as_str())
    );
    assert_eq!(
        descendant_authority
            .authoritative_participant_lineage
            .last()
            .map(String::as_str),
        Some(retained_participant_id.as_str())
    );
    super::store::upgrade_v2_root_to_v3_test(
        Path::new(&request.workspace_binding.authority_store_root.physical_path),
        &root_after_r0,
    )
    .unwrap();

    let root_after_upgrade = authority.read_a12b_root().unwrap();

    let resolution = ResolveStartupOwnershipRequestV1 {
        intent_id: request.intent_id.clone(),
        issuer_request_id: request.issuer_request_id.clone(),
        payload_commitment: root_after_upgrade.transition_intent_map[&request.intent_id]
            .payload_commitment
            .clone(),
        protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: request.target_authoritative_participant_id.clone(),
        },
        protocol_event: HostStartupOwnershipProtocolEventV1::StartupFailedBeforeOwnership {
            failure_id: "startup-failed-start-1".into(),
        },
        observed_at: timestamp("2026-07-14T12:01:40.000000000Z"),
    };
    let StartupOwnershipResolutionOutcomeV1::ResolvedStart(resolved) = authority
        .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:01:41.000000000Z"))
        .unwrap()
    else {
        panic!("terminal preserved Start startup reconciliation must commit")
    };
    assert_eq!(
        authority
            .resolve_startup_ownership_at(&resolution, timestamp("2026-07-14T12:01:41.000000000Z"),)
            .unwrap(),
        StartupOwnershipResolutionOutcomeV1::JoinedStart(resolved.clone())
    );

    let root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("terminal preserved Start must retain current authority")
    };
    assert_eq!(current_authority.authority_revision, 3);
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::Terminal
    );
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some(request.target_authoritative_participant_id.as_str())
    );
    assert_eq!(
        current_authority
            .authoritative_participant_lineage
            .last()
            .map(String::as_str),
        Some(retained_participant_id.as_str())
    );
    assert_eq!(current_authority.retained_worker_refs.len(), 1);
    let journal = &root.application_journal[&request.intent_id];
    assert!(journal.startup_terminal_application.is_some());
    let intent = &root.transition_intent_map[&request.intent_id];
    let HostSessionTransitionIntentStateV2::Applied {
        startup_ownership, ..
    } = &intent.state
    else {
        panic!("terminal preserved Start must remain applied")
    };
    let HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
        evidence_id,
        result_ref,
        authority_revision_before,
        authority_revision_after,
        resulting_posture,
        ..
    } = startup_ownership.as_ref()
    else {
        panic!("preserved Start startup must be terminally reconciled")
    };
    assert_eq!(evidence_id, "startup-failed-start-1");
    assert_eq!(*authority_revision_before, 2);
    assert_eq!(*authority_revision_after, 3);
    assert_eq!(*resulting_posture, HostSessionPostureV1::Terminal);
    assert!(matches!(
        intent.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible { .. }
    ));
    assert!(matches!(
        intent.input_handoff,
        HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance { .. }
    ));
    let trusted_root = super::trusted_fs::TrustedAuthorityRoot::open(Path::new(
        &root.bootstrap_home.physical_path,
    ))
    .unwrap();
    let result_bytes = super::store::read_typed_object_v2_or_v3_opened(
        &trusted_root,
        root.root_revision,
        result_ref,
        None,
    )
    .unwrap();
    let result: StartupOwnershipResultHashInputV1 =
        super::canonical_json::from_slice(&result_bytes).unwrap();
    assert_eq!(
        result.outcome,
        StartupOwnershipOutcomeV1::TerminalReconciled {
            reason: super::schema::HostStartupTerminalReasonV1::StartupFailedBeforeOwnership,
            authority_revision_after: 3,
            resulting_posture: HostSessionPostureV1::Terminal,
            authority_record_commitment: canonical_commitment(&authority_hash_input(
                current_authority
            )),
        }
    );
    assert_eq!(result.evidence.expected_authority_revision, 1);
}

#[test]
fn successor_resume_issue_claim_apply_retries_to_one_pending_post_turn() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let (current, resume_handle_ref) = park_with_resume_handle(&authority, &request);
    let successor =
        resume_successor_request(&current, resume_handle_ref.clone(), "participant-resume-1");

    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &current,
            &successor,
            timestamp("2026-07-14T12:04:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("first resume issuance must commit")
    };
    assert_eq!(
        authority
            .issue_successor_at(
                &current,
                &successor,
                timestamp("2026-07-14T12:04:00.000000000Z"),
                300,
            )
            .unwrap(),
        SuccessorTransitionIssueOutcomeV1::Joined(issued.clone())
    );

    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-resume-1".into(),
        claimant_attempt_id: "attempt-resume-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:04:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("first resume claim must commit")
    };
    assert_eq!(
        authority
            .claim_successor_at(&claim, timestamp("2026-07-14T12:04:11.000000000Z"), 30)
            .unwrap(),
        SuccessorTransitionClaimOutcomeV1::Joined(claimed.clone())
    );

    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("resume claim must retain a claim")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id.clone(),
        issuer_request_id: claimed.issuer_request_id.clone(),
        payload_commitment: claimed.payload_commitment.clone(),
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id.clone(),
        expected_claim_revision: claim_revision,
    };
    let SuccessorTransitionApplicationOutcomeV1::Applied(applied) = authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:04:20.000000000Z"))
        .unwrap()
    else {
        panic!("first resume application must commit")
    };
    assert_eq!(
        authority
            .apply_successor_at(&application, timestamp("2026-07-14T12:04:21.000000000Z"))
            .unwrap(),
        SuccessorTransitionApplicationOutcomeV1::Joined(applied.clone())
    );

    let root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("resume application must retain a durable authority")
    };
    assert_eq!(
        current_authority.authority_revision,
        applied_successor_authority_revision(&applied)
    );
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some("participant-resume-1")
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );
    assert!(current_authority
        .internal_resume_handle_refs
        .iter()
        .any(|current| current == &resume_handle_ref));
    let intent = &root.successor_transition_intent_map[&successor.intent_id];
    let HostSessionTransitionIntentStateV3::Applied {
        startup_ownership,
        post_turn,
        ..
    } = &intent.state
    else {
        panic!("resume successor intent must be applied")
    };
    assert_eq!(
        startup_ownership.as_ref(),
        &HostSessionStartupOwnershipApplicationV1::NotApplicable
    );
    assert!(matches!(
        post_turn.as_ref(),
        HostSessionPostTurnApplicationV2::Pending {
            expected_run_id,
            expected_authority_revision,
        } if expected_run_id == "run-resume-1"
            && *expected_authority_revision == applied_successor_authority_revision(&applied)
    ));
    assert!(matches!(
        intent.input_handoff,
        HostSessionTransitionInputHandoffV1::Pending {
            ref run_id, ..
        } if run_id == "run-resume-1"
    ));
}

#[test]
fn successor_resume_input_acceptance_and_obligation_cut_consumption_retries() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let successor = applied_resume_successor(&authority, &request);
    let applied_root = authority.read_a12b_root().unwrap();
    let applied_intent = applied_root.successor_transition_intent_map[&successor.intent_id].clone();

    let input_acceptance = AcceptTransitionInputRequestV1 {
        intent_id: applied_intent.intent_id.clone(),
        issuer_request_id: applied_intent.issuer_request_id.clone(),
        payload_commitment: applied_intent.payload_commitment.clone(),
        accepting_participant_id: applied_intent.target_authoritative_participant_id.clone(),
        accepted_at: timestamp("2026-07-14T12:04:30.000000000Z"),
    };
    let InputAcceptanceOutcomeV1::Accepted(accepted_intent) = authority
        .accept_transition_input(&input_acceptance)
        .unwrap()
    else {
        panic!("first resume input acceptance must commit")
    };
    assert_eq!(
        authority
            .accept_transition_input(&input_acceptance)
            .unwrap(),
        InputAcceptanceOutcomeV1::Joined(accepted_intent.clone())
    );
    let successor_authority_revision = applied_successor_authority_revision(&accepted_intent);
    let settled_authority_revision = successor_authority_revision.checked_add(1).unwrap();

    let resume_post_turn = post_turn_request(
        &accepted_intent,
        successor_authority_revision,
        HostPostTurnProtocolEventKindV1::ResumableClean,
        HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: accepted_intent.target_authoritative_participant_id.clone(),
        },
        "2026-07-14T12:04:40.000000000Z",
        "2026-07-14T12:04:41.000000000Z",
    );
    let PostTurnResolutionOutcomeV1::AwaitingObligationCut(awaiting_cut_intent) =
        authority.resolve_post_turn(&resume_post_turn).unwrap()
    else {
        panic!("resumable post-turn must await an obligation cut")
    };
    assert_eq!(
        authority.resolve_post_turn(&resume_post_turn).unwrap(),
        PostTurnResolutionOutcomeV1::Joined(awaiting_cut_intent.clone())
    );

    let root_before_pending = authority.read_a12b_root().unwrap();
    let ConsumeObligationSnapshotRequestV1 {
        intent_id,
        issuer_request_id,
        payload_commitment,
        ..
    } = ConsumeObligationSnapshotRequestV1 {
        intent_id: awaiting_cut_intent.intent_id.clone(),
        issuer_request_id: awaiting_cut_intent.issuer_request_id.clone(),
        payload_commitment: awaiting_cut_intent.payload_commitment.clone(),
        ledger_read: pending_ledger_read(&accepted_intent, successor_authority_revision),
    };
    let pending_cut = ConsumeObligationSnapshotRequestV1 {
        intent_id: intent_id.clone(),
        issuer_request_id: issuer_request_id.clone(),
        payload_commitment: payload_commitment.clone(),
        ledger_read: pending_ledger_read(&accepted_intent, successor_authority_revision),
    };
    assert_eq!(
        authority.consume_obligation_snapshot(&pending_cut).unwrap(),
        ObligationSnapshotConsumptionOutcomeV1::Pending(awaiting_cut_intent.clone())
    );
    assert_eq!(authority.read_a12b_root().unwrap(), root_before_pending);

    let complete_cut = ConsumeObligationSnapshotRequestV1 {
        intent_id,
        issuer_request_id,
        payload_commitment,
        ledger_read: ObligationLedgerSnapshotReadV1::Complete {
            snapshot: complete_ledger_snapshot(
                &accepted_intent,
                successor_authority_revision,
                ObligationAttentionDispositionV1::NoUnresolvedAttention,
            ),
        },
    };
    let ObligationSnapshotConsumptionOutcomeV1::Applied(applied_after_cut) = authority
        .consume_obligation_snapshot(&complete_cut)
        .unwrap()
    else {
        panic!("complete cut must apply exactly once")
    };
    assert_eq!(
        authority
            .consume_obligation_snapshot(&complete_cut)
            .unwrap(),
        ObligationSnapshotConsumptionOutcomeV1::Joined(applied_after_cut.clone())
    );

    let final_root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &final_root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("resume obligation-cut completion must retain a durable authority")
    };
    assert_eq!(
        current_authority.authority_revision,
        settled_authority_revision
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
    );
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some("participant-resume-1")
    );

    let final_intent = &final_root.successor_transition_intent_map[&successor.intent_id];
    assert!(matches!(
        final_intent.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible { .. }
    ));
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &final_intent.state else {
        panic!("resume successor must remain applied after cut completion")
    };
    assert!(matches!(
        post_turn.as_ref(),
        HostSessionPostTurnApplicationV2::Applied {
            obligation_snapshot_ref: Some(_),
            authority_revision_before,
            authority_revision_after,
            resulting_posture: HostSessionPostureV1::ParkedResumable,
            ..
        } if *authority_revision_before == successor_authority_revision
            && *authority_revision_after == settled_authority_revision
    ));
    assert!(matches!(
        final_intent.input_handoff,
        HostSessionTransitionInputHandoffV1::Accepted { ref run_id, .. }
            if run_id == "run-resume-1"
    ));
    assert!(
        final_root.successor_application_journal[&successor.intent_id]
            .post_turn_application
            .is_some()
    );
}

#[test]
fn successor_resume_pre_acceptance_terminal_failure_terminalizes_input_and_retries() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let successor = applied_resume_successor(&authority, &request);
    let applied_root = authority.read_a12b_root().unwrap();
    let applied_intent = applied_root.successor_transition_intent_map[&successor.intent_id].clone();
    let successor_authority_revision = applied_successor_authority_revision(&applied_intent);
    let terminal_authority_revision = successor_authority_revision.checked_add(1).unwrap();

    let terminal_failure = post_turn_request(
        &applied_intent,
        successor_authority_revision,
        HostPostTurnProtocolEventKindV1::TerminalFailure {
            reason: HostPostTurnTerminalReasonV1::ResumeRuntimeCreationRejected,
        },
        HostPostTurnProtocolActorV1::LaunchApplicationClaimant {
            claim_id: "claim-resume-1".into(),
            claimant_attempt_id: "attempt-resume-1".into(),
        },
        "2026-07-14T12:04:30.000000000Z",
        "2026-07-14T12:04:31.000000000Z",
    );
    let PostTurnResolutionOutcomeV1::Applied(applied_terminal_intent) =
        authority.resolve_post_turn(&terminal_failure).unwrap()
    else {
        panic!("pre-acceptance runtime rejection must terminalize exactly once")
    };
    assert_eq!(
        authority.resolve_post_turn(&terminal_failure).unwrap(),
        PostTurnResolutionOutcomeV1::Joined(applied_terminal_intent.clone())
    );

    let final_root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &final_root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("terminal failure must retain a durable authority")
    };
    assert_eq!(
        current_authority.authority_revision,
        terminal_authority_revision
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::Terminal
    );
    assert_eq!(
        current_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some("participant-resume-1")
    );

    let final_intent = &final_root.successor_transition_intent_map[&successor.intent_id];
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &final_intent.state else {
        panic!("terminal failure successor must remain applied")
    };
    assert!(matches!(
        post_turn.as_ref(),
        HostSessionPostTurnApplicationV2::Applied {
            obligation_snapshot_ref: None,
            authority_revision_before,
            authority_revision_after,
            resulting_posture: HostSessionPostureV1::Terminal,
            ..
        } if *authority_revision_before == successor_authority_revision
            && *authority_revision_after == terminal_authority_revision
    ));
    assert!(matches!(
        final_intent.input_handoff,
        HostSessionTransitionInputHandoffV1::TerminalWithoutAcceptance {
            ref run_id, ..
        } if run_id == "run-resume-1"
    ));
    assert!(matches!(
        final_intent.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible { .. }
    ));
    assert!(
        final_root.successor_application_journal[&successor.intent_id]
            .post_turn_application
            .is_some()
    );
}

#[test]
fn successor_resume_complete_cut_with_unresolved_attention_advances_to_awaiting_attention() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let successor = applied_resume_successor(&authority, &request);
    let applied_root = authority.read_a12b_root().unwrap();
    let applied_intent = applied_root.successor_transition_intent_map[&successor.intent_id].clone();

    let input_acceptance = AcceptTransitionInputRequestV1 {
        intent_id: applied_intent.intent_id.clone(),
        issuer_request_id: applied_intent.issuer_request_id.clone(),
        payload_commitment: applied_intent.payload_commitment.clone(),
        accepting_participant_id: applied_intent.target_authoritative_participant_id.clone(),
        accepted_at: timestamp("2026-07-14T12:04:30.000000000Z"),
    };
    authority
        .accept_transition_input(&input_acceptance)
        .unwrap();

    let accepted_root = authority.read_a12b_root().unwrap();
    let accepted_intent =
        accepted_root.successor_transition_intent_map[&successor.intent_id].clone();
    let successor_authority_revision = applied_successor_authority_revision(&accepted_intent);
    let settled_authority_revision = successor_authority_revision.checked_add(1).unwrap();
    let awaiting_cut_request = post_turn_request(
        &accepted_intent,
        successor_authority_revision,
        HostPostTurnProtocolEventKindV1::ResumableClean,
        HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: accepted_intent.target_authoritative_participant_id.clone(),
        },
        "2026-07-14T12:04:40.000000000Z",
        "2026-07-14T12:04:41.000000000Z",
    );
    authority.resolve_post_turn(&awaiting_cut_request).unwrap();

    let complete_cut = ConsumeObligationSnapshotRequestV1 {
        intent_id: accepted_intent.intent_id.clone(),
        issuer_request_id: accepted_intent.issuer_request_id.clone(),
        payload_commitment: accepted_intent.payload_commitment.clone(),
        ledger_read: ObligationLedgerSnapshotReadV1::Complete {
            snapshot: complete_ledger_snapshot(
                &accepted_intent,
                successor_authority_revision,
                ObligationAttentionDispositionV1::HasUnresolvedAttention,
            ),
        },
    };
    let ObligationSnapshotConsumptionOutcomeV1::Applied(applied_attention_intent) = authority
        .consume_obligation_snapshot(&complete_cut)
        .unwrap()
    else {
        panic!("unresolved-attention cut must apply exactly once")
    };
    assert_eq!(
        authority
            .consume_obligation_snapshot(&complete_cut)
            .unwrap(),
        ObligationSnapshotConsumptionOutcomeV1::Joined(applied_attention_intent.clone())
    );

    let final_root = authority.read_a12b_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &final_root.session_namespace_map[&request.orchestration_session_id]
    else {
        panic!("attention cut must retain a durable authority")
    };
    assert_eq!(
        current_authority.authority_revision,
        settled_authority_revision
    );
    assert_eq!(
        current_authority.lifecycle_posture,
        HostSessionPostureV1::AwaitingAttention
    );

    let final_intent = &final_root.successor_transition_intent_map[&successor.intent_id];
    let HostSessionTransitionIntentStateV3::Applied { post_turn, .. } = &final_intent.state else {
        panic!("attention successor must remain applied")
    };
    assert!(matches!(
        post_turn.as_ref(),
        HostSessionPostTurnApplicationV2::Applied {
            obligation_snapshot_ref: Some(_),
            authority_revision_before,
            authority_revision_after,
            resulting_posture: HostSessionPostureV1::AwaitingAttention,
            ..
        } if *authority_revision_before == successor_authority_revision
            && *authority_revision_after == settled_authority_revision
    ));
    assert!(matches!(
        final_intent.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible { .. }
    ));
}

#[test]
fn successor_resume_complete_cut_rejects_materialization_watermark_beyond_terminal_event() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let successor = applied_resume_successor(&authority, &request);
    let applied_root = authority.read_a12b_root().unwrap();
    let applied_intent = applied_root.successor_transition_intent_map[&successor.intent_id].clone();

    let input_acceptance = AcceptTransitionInputRequestV1 {
        intent_id: applied_intent.intent_id.clone(),
        issuer_request_id: applied_intent.issuer_request_id.clone(),
        payload_commitment: applied_intent.payload_commitment.clone(),
        accepting_participant_id: applied_intent.target_authoritative_participant_id.clone(),
        accepted_at: timestamp("2026-07-14T12:04:30.000000000Z"),
    };
    authority
        .accept_transition_input(&input_acceptance)
        .unwrap();

    let accepted_root = authority.read_a12b_root().unwrap();
    let accepted_intent =
        accepted_root.successor_transition_intent_map[&successor.intent_id].clone();
    let successor_authority_revision = applied_successor_authority_revision(&accepted_intent);
    let awaiting_cut_request = post_turn_request(
        &accepted_intent,
        successor_authority_revision,
        HostPostTurnProtocolEventKindV1::ResumableClean,
        HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: accepted_intent.target_authoritative_participant_id.clone(),
        },
        "2026-07-14T12:04:40.000000000Z",
        "2026-07-14T12:04:41.000000000Z",
    );
    authority.resolve_post_turn(&awaiting_cut_request).unwrap();

    let mut overrun_snapshot = complete_ledger_snapshot(
        &accepted_intent,
        successor_authority_revision,
        ObligationAttentionDispositionV1::NoUnresolvedAttention,
    );
    overrun_snapshot
        .materialization_cut
        .materialized_through_event_sequence += 1;
    let complete_cut = ConsumeObligationSnapshotRequestV1 {
        intent_id: accepted_intent.intent_id.clone(),
        issuer_request_id: accepted_intent.issuer_request_id.clone(),
        payload_commitment: accepted_intent.payload_commitment.clone(),
        ledger_read: ObligationLedgerSnapshotReadV1::Complete {
            snapshot: overrun_snapshot,
        },
    };

    let root_before_rejection = authority.read_a12b_root().unwrap();
    let error = authority
        .consume_obligation_snapshot(&complete_cut)
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "complete obligation snapshot conflicts with AwaitingObligationCut commitments"
    );
    assert_eq!(authority.read_a12b_root().unwrap(), root_before_rejection);
}

#[test]
fn successor_resume_complete_cut_join_and_current_resolution_survive_released_transport_payload() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let successor = applied_resume_successor(&authority, &request);
    let applied_root = authority.read_a12b_root().unwrap();
    let applied_intent = applied_root.successor_transition_intent_map[&successor.intent_id].clone();

    let input_acceptance = AcceptTransitionInputRequestV1 {
        intent_id: applied_intent.intent_id.clone(),
        issuer_request_id: applied_intent.issuer_request_id.clone(),
        payload_commitment: applied_intent.payload_commitment.clone(),
        accepting_participant_id: applied_intent.target_authoritative_participant_id.clone(),
        accepted_at: timestamp("2026-07-14T12:04:30.000000000Z"),
    };
    authority
        .accept_transition_input(&input_acceptance)
        .unwrap();

    let accepted_root = authority.read_a12b_root().unwrap();
    let accepted_intent =
        accepted_root.successor_transition_intent_map[&successor.intent_id].clone();
    let successor_authority_revision = applied_successor_authority_revision(&accepted_intent);
    let settled_authority_revision = successor_authority_revision.checked_add(1).unwrap();
    let awaiting_cut_request = post_turn_request(
        &accepted_intent,
        successor_authority_revision,
        HostPostTurnProtocolEventKindV1::ResumableClean,
        HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
            participant_id: accepted_intent.target_authoritative_participant_id.clone(),
        },
        "2026-07-14T12:04:40.000000000Z",
        "2026-07-14T12:04:41.000000000Z",
    );
    authority.resolve_post_turn(&awaiting_cut_request).unwrap();

    let complete_cut = ConsumeObligationSnapshotRequestV1 {
        intent_id: accepted_intent.intent_id.clone(),
        issuer_request_id: accepted_intent.issuer_request_id.clone(),
        payload_commitment: accepted_intent.payload_commitment.clone(),
        ledger_read: ObligationLedgerSnapshotReadV1::Complete {
            snapshot: complete_ledger_snapshot(
                &accepted_intent,
                successor_authority_revision,
                ObligationAttentionDispositionV1::NoUnresolvedAttention,
            ),
        },
    };
    authority
        .consume_obligation_snapshot(&complete_cut)
        .unwrap();

    let release_eligible_root = authority.read_a12b_root().unwrap();
    let release_eligible_intent =
        release_eligible_root.successor_transition_intent_map[&successor.intent_id].clone();
    let HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
        terminal_handoff_ref: expected_terminal_handoff_ref,
    } = &release_eligible_intent.transport_payload_state
    else {
        panic!("completed resume cut must mark transport payload release-eligible")
    };
    let released_at = timestamp("2026-07-14T12:05:10.000000000Z");
    let mut released_root = release_eligible_root.clone();
    released_root.root_revision += 1;
    released_root
        .successor_transition_intent_map
        .get_mut(&successor.intent_id)
        .unwrap()
        .transport_payload_state = HostSessionTransitionTransportPayloadStateV1::Released {
        terminal_handoff_ref: expected_terminal_handoff_ref.clone(),
        released_at: released_at.clone(),
    };
    released_root
        .object_index
        .get_mut(&release_eligible_intent.transport_payload_ref.ref_id)
        .unwrap()
        .storage_state = super::store_schema::AuthorityObjectStorageStateV1::Released {
        terminal_handoff_ref: expected_terminal_handoff_ref.clone(),
        released_at: released_at.clone(),
    };
    released_root.validate().unwrap();

    let authority_root_path = Path::new(&release_eligible_root.bootstrap_home.physical_path);
    let transport_path = authority_root_path.join(format!(
        "authority-v1/objects/transition-transport-payload/v1/{}.obj",
        release_eligible_intent.transport_payload_ref.ref_id
    ));
    assert!(transport_path.exists());
    fs::remove_file(&transport_path).unwrap();
    fs::write(
        authority_root_path.join("authority-v1/state-root-v1.json"),
        super::store_schema::VersionedStateRoot::V3(released_root.clone())
            .to_canonical_bytes()
            .unwrap(),
    )
    .unwrap();
    assert!(!transport_path.exists());

    let ObligationSnapshotConsumptionOutcomeV1::Joined(joined_after_release) = authority
        .consume_obligation_snapshot(&complete_cut)
        .unwrap()
    else {
        panic!("released transport payload must still join the exact completed cut")
    };
    assert!(matches!(
        joined_after_release.transport_payload_state,
        HostSessionTransitionTransportPayloadStateV1::Released {
            ref terminal_handoff_ref,
            ref released_at,
        } if terminal_handoff_ref == expected_terminal_handoff_ref
            && released_at == &timestamp("2026-07-14T12:05:10.000000000Z")
    ));

    let resolved = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(
        resolved.authority.authority_revision,
        settled_authority_revision
    );
    assert_eq!(
        resolved.authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
    );
    assert_eq!(
        resolved
            .authority
            .active_authoritative_participant_id
            .as_deref(),
        Some("participant-resume-1")
    );
}

#[test]
fn start_submission_barrier_durably_fails_closed_as_indeterminate() {
    let (_parent, authority, _request, registration) = applied_start_for_continuity();
    let barrier = authority
        .retryable_start_transaction(&registration.request_key_sha256)
        .unwrap()
        .unwrap();
    assert!(matches!(
        barrier.state,
        StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
    ));

    assert!(
        authority
            .mark_start_submission_outcome_indeterminate(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                timestamp("2026-07-14T12:01:14.000000000Z"),
            )
            .is_err(),
        "a declaration reordered before its barrier must fail closed"
    );
    let declared_at = timestamp("2026-07-14T12:01:16.000000000Z");
    let indeterminate = authority
        .mark_start_submission_outcome_indeterminate(
            &registration.start_transaction_id,
            &registration.request_key_sha256,
            declared_at.clone(),
        )
        .unwrap();
    assert!(matches!(
        indeterminate.state,
        StartTransactionStateV1::PromptSubmissionIndeterminate {
            ref barrier_committed_at,
            ref declared_at,
        } if barrier_committed_at == &timestamp("2026-07-14T12:01:15.000000000Z")
            && declared_at == &timestamp("2026-07-14T12:01:16.000000000Z")
    ));
    assert_eq!(
        authority
            .mark_start_submission_outcome_indeterminate(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                declared_at,
            )
            .unwrap(),
        indeterminate,
        "an exact retry must join the durable indeterminate result"
    );
    assert!(
        authority
            .mark_start_submission_outcome_indeterminate(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                timestamp("2026-07-14T12:01:17.000000000Z"),
            )
            .is_err(),
        "a conflicting declaration must fail closed"
    );
    assert!(
        authority
            .establish_start_continuation(&registration)
            .is_err(),
        "indeterminate submission is not authenticated provider acceptance"
    );
    assert!(
        authority
            .mark_start_prompt_submission_no_replay_barrier(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                timestamp("2026-07-14T12:01:15.000000000Z"),
            )
            .is_err(),
        "retry from indeterminate must not authorize another provider submission"
    );
    assert_eq!(
        authority
            .retryable_start_transaction(&registration.request_key_sha256)
            .unwrap(),
        Some(indeterminate)
    );
}

#[test]
fn fresh_greenfield_v1_has_no_retryable_start_transaction() {
    let (_parent, authority, _binding) = authority();
    let root = authority.read_root().unwrap();
    root.validate().unwrap();
    assert_eq!(root.schema_version, 1);
    assert!(root.session_namespace_map.is_empty());
    assert!(root.transition_intent_map.is_empty());
    assert!(root.issuer_request_index.is_empty());
    assert!(root.application_journal.is_empty());
    assert!(root.object_index.is_empty());

    assert_eq!(
        authority
            .retryable_start_transaction(&"a".repeat(64))
            .unwrap(),
        None
    );
}

#[test]
fn invalid_non_greenfield_v1_is_not_treated_as_an_empty_start_lookup() {
    let (_parent, authority, _binding) = authority();
    let mut root = authority.read_root().unwrap();
    root.object_index.insert(
        "ao_non_greenfield_start_lookup".into(),
        AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: "ao_non_greenfield_start_lookup".into(),
            object_kind: AuthorityObjectKindV1::Policy,
            object_schema_version: 1,
            byte_length: 1,
            storage_state: AuthorityObjectStorageStateV1::Present,
        },
    );
    let error =
        retryable_start_transaction_from_v1(&root).expect_err("non-greenfield V1 must fail closed");
    assert!(error.to_string().contains("UnsupportedNonGreenfieldRootV1"));
}

#[test]
fn start_continuity_registration_and_settlement_are_durable_and_resumable() {
    let (_parent, authority, request, registration) = applied_start_for_continuity();
    assert!(matches!(
        authority.read_a12b_root().unwrap().start_transaction_map
            [&registration.start_transaction_id]
            .state,
        StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
    ));
    assert!(
        authority
            .mark_start_prompt_submission_no_replay_barrier(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                timestamp("2026-07-14T12:01:16.000000000Z"),
            )
            .is_err(),
        "an in-progress retry must not authorize another prompt submission"
    );

    let error = authority
        .establish_start_continuation_at_with_crash_point(
            &registration,
            EstablishStartContinuationCrashPointV1::HandlePublished,
        )
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "injected crash after Start handle publication"
    );
    let after_orphan = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(after_orphan.authority.authority_revision, 1);
    assert!(after_orphan
        .authority
        .internal_resume_handle_refs
        .is_empty());
    assert_eq!(
        after_orphan.authority.lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );
    assert!(matches!(
        authority.read_a12b_root().unwrap().start_transaction_map
            [&registration.start_transaction_id]
            .state,
        StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
    ));

    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("first continuation registration must commit")
    };
    let after_registration = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(after_registration.authority.authority_revision, 2);
    assert_eq!(
        after_registration.authority.lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );
    assert_eq!(
        after_registration.authority.internal_resume_handle_refs,
        vec![registered.resume_handle_ref.clone()]
    );
    assert_eq!(
        authority
            .establish_start_continuation(&registration)
            .unwrap(),
        StartContinuationOutcomeV1::Joined(registered.clone())
    );
    assert!(matches!(
        authority.read_a12b_root().unwrap().start_transaction_map
            [&registration.start_transaction_id]
            .state,
        StartTransactionStateV1::ContinuationRegistered { .. }
    ));

    let settlement = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref.clone(),
        registered.authority_revision_after,
    );
    let error = authority
        .settle_start_turn_at_with_crash_point(
            &settlement,
            SettleStartTurnCrashPointV1::RootCommitted,
        )
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "injected crash after Start settlement commit"
    );
    let StartTurnSettlementOutcomeV1::Joined(settled) =
        authority.settle_start_turn(&settlement).unwrap()
    else {
        panic!("retry after committed settlement must join")
    };
    let parked = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(parked.authority.authority_revision, 3);
    assert_eq!(
        parked.authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
    );
    assert!(parked
        .authority
        .internal_resume_handle_refs
        .contains(&registered.resume_handle_ref));
    assert!(parked
        .authority
        .internal_resume_handle_refs
        .contains(&settled.continuation_resume_handle_ref));
    assert!(matches!(
        authority.read_a12b_root().unwrap().start_transaction_map
            [&registration.start_transaction_id]
            .state,
        StartTransactionStateV1::TurnSettledAwaitingResponse { .. }
    ));
    let delivered_at = timestamp("2026-07-14T12:01:31.000000000Z");
    let delivered = authority
        .mark_start_response_delivered(
            &registration.start_transaction_id,
            &registration.request_key_sha256,
            delivered_at.clone(),
        )
        .unwrap();
    assert!(matches!(
        delivered.state,
        StartTransactionStateV1::PublicResponseDelivered { .. }
    ));
    assert_eq!(
        authority
            .mark_start_response_delivered(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                delivered_at,
            )
            .unwrap(),
        delivered,
        "an exact public-response retry must converge"
    );
    assert_eq!(
        authority
            .establish_start_continuation(&registration)
            .unwrap(),
        StartContinuationOutcomeV1::Joined(registered.clone()),
        "an exact registration retry must join after settlement and response delivery"
    );
    assert_eq!(
        authority.settle_start_turn(&settlement).unwrap(),
        StartTurnSettlementOutcomeV1::Joined(settled.clone()),
        "an exact settlement retry must join after response delivery"
    );
    let committed = authority
        .committed_start_public_result(&delivered)
        .expect("delivered transaction must authenticate its exact settlement result");
    assert_eq!(
        committed.completion_kind,
        StartTurnCompletionKindV1::ResumableClean
    );
    assert_eq!(
        committed.resulting_posture,
        HostSessionPostureV1::ParkedResumable
    );
    assert!(authority
        .retryable_start_transaction(&registration.request_key_sha256)
        .unwrap()
        .is_none());

    let successor = resume_successor_request(
        &parked,
        settled.continuation_resume_handle_ref,
        "participant-resume-from-start-1",
    );
    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &parked,
            &successor,
            timestamp("2026-07-14T12:02:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("ResumeOneTurn must consume the continuation produced by Start")
    };
    assert_eq!(issued.resume_handle_ref, successor.resume_handle_ref);

    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-resume-from-start-1".into(),
        claimant_attempt_id: "attempt-resume-from-start-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:02:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("ResumeOneTurn claim must consume Start-produced continuity")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("ResumeOneTurn claim must retain exact claim evidence")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id,
        issuer_request_id: claimed.issuer_request_id,
        payload_commitment: claimed.payload_commitment,
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id,
        expected_claim_revision: claim_revision,
    };
    let SuccessorTransitionApplicationOutcomeV1::Applied(applied) = authority
        .apply_successor_at(&application, timestamp("2026-07-14T12:02:20.000000000Z"))
        .unwrap()
    else {
        panic!("ResumeOneTurn application must consume Start-produced continuity")
    };
    assert_eq!(
        applied
            .resume_handle_ref
            .as_ref()
            .expect("applied ResumeOneTurn continuity"),
        successor
            .resume_handle_ref
            .as_ref()
            .expect("issued ResumeOneTurn continuity")
    );
}

#[test]
fn mixed_start_continuation_and_r0_ancestry_resolves_every_exact_revision() {
    let (_parent, authority, request, registration) = applied_world_start_for_continuity();
    let revision_1 = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(revision_1.authority.authority_revision, 1);

    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("first continuation registration must commit")
    };
    let revision_2 = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(revision_2.authority.authority_revision, 2);

    let retained_runtime = RetainedWorkerRuntime;
    let first_plan = RetainedWorkerRegistrationPlanV1 {
        registration_request_id: "mixed-ancestry-r0-1".into(),
        orchestration_session_id: request.orchestration_session_id.clone(),
        expected_authority: RetainedWorkerAuthorityPreconditionV1 {
            authority_store_id: revision_2.observation.authority_store_id.clone(),
            authority_revision: revision_2.observation.authority_revision,
            authority_record_commitment: revision_2.observation.authority_record_commitment.clone(),
        },
        retained_participant_id: "mixed-ancestry-worker-1".into(),
        descriptor: AgentDescriptorV1 {
            schema_version: 1,
            agent_id: "mixed-worker-1".into(),
            backend_id: "cli:codex-world".into(),
            backend_kind: RuntimeBackendKindV1::Codex,
            protocol: "substrate.agent.session".into(),
            execution_scope: AgentExecutionScopeV1::World,
            binary_path: "/usr/bin/codex".into(),
        },
        internal_uaa_session_id: "uaa-mixed-worker-1".into(),
    };
    let first_retained = retained_runtime
        .register_retained_target(&authority, &first_plan)
        .expect("R0 registration must follow an authenticated continuation edge");
    let revision_3 = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(revision_3.authority.authority_revision, 3);

    let first_registered_at = authority
        .read_a12b_root()
        .unwrap()
        .retained_worker_registration_journal
        .get(&first_retained.registration_id)
        .expect("first R0 journal")
        .registered_at
        .clone();
    let mut settlement = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref.clone(),
        revision_3.observation.authority_revision,
    );
    settlement.completed_at = first_registered_at;
    let StartTurnSettlementOutcomeV1::Applied(settled) =
        authority.settle_start_turn(&settlement).unwrap()
    else {
        panic!("Start settlement must follow the interleaved R0 edge")
    };
    let revision_4 = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(revision_4.authority.authority_revision, 4);

    let second_plan = RetainedWorkerRegistrationPlanV1 {
        registration_request_id: "mixed-ancestry-r0-2".into(),
        orchestration_session_id: request.orchestration_session_id.clone(),
        expected_authority: RetainedWorkerAuthorityPreconditionV1 {
            authority_store_id: revision_4.observation.authority_store_id.clone(),
            authority_revision: revision_4.observation.authority_revision,
            authority_record_commitment: revision_4.observation.authority_record_commitment.clone(),
        },
        retained_participant_id: "mixed-ancestry-worker-2".into(),
        descriptor: AgentDescriptorV1 {
            agent_id: "mixed-worker-2".into(),
            ..first_plan.descriptor.clone()
        },
        internal_uaa_session_id: "uaa-mixed-worker-2".into(),
    };
    retained_runtime
        .register_retained_target(&authority, &second_plan)
        .expect("later R0 registration must follow authenticated Start settlement");
    let revision_5 = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert_eq!(revision_5.authority.authority_revision, 5);

    for expected in [
        &revision_1,
        &revision_2,
        &revision_3,
        &revision_4,
        &revision_5,
    ] {
        let resolved = authority
            .resolve_exact_at_revision(
                &request.orchestration_session_id,
                expected.authority.authority_revision,
            )
            .expect("every mixed typed revision must resolve on the exact chain");
        let resolved_observation = resolved.observation();
        assert_eq!(resolved.authority, expected.authority);
        assert_eq!(
            resolved.authority_record_commitment,
            expected.observation.authority_record_commitment
        );
        assert_eq!(
            resolved.authoritative_lineage_commitment,
            expected.observation.authoritative_lineage_commitment
        );
        assert_eq!(
            resolved_observation.authority_store_id,
            expected.observation.authority_store_id
        );
        assert_eq!(
            resolved_observation.bootstrap_home,
            expected.observation.bootstrap_home
        );
    }
    assert_eq!(
        revision_5.authority.authoritative_participant_lineage,
        vec![
            request.target_authoritative_participant_id,
            first_plan.retained_participant_id,
            second_plan.retained_participant_id,
        ]
    );
    assert!(revision_5
        .authority
        .internal_resume_handle_refs
        .contains(&registered.resume_handle_ref));
    assert!(revision_5
        .authority
        .internal_resume_handle_refs
        .contains(&settled.continuation_resume_handle_ref));
    assert_eq!(revision_5.authority.retained_worker_refs.len(), 2);
    assert_eq!(
        revision_5.authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
    );

    let root = authority.read_a12b_root().unwrap();
    let exact_history_is_rejected = |candidate: &StateRootV3| {
        exact_v3_authority_history_with_reader(
            candidate,
            &request.orchestration_session_id,
            |reference| {
                authority
                    .read_authority_object_v2_at(root.root_revision, reference)
                    .map_err(|error| error.to_string())
            },
        )
        .is_err()
    };

    let mut missing = root.clone();
    missing
        .retained_worker_registration_journal
        .remove(&first_retained.registration_id);
    assert!(
        exact_history_is_rejected(&missing),
        "a missing mixed authority edge must fail closed"
    );

    let original_registration = root
        .retained_worker_registration_journal
        .get(&first_retained.registration_id)
        .expect("first mixed R0 registration")
        .clone();
    let original_request = root
        .retained_worker_registration_request_index
        .get(&original_registration.issuer_request_id)
        .expect("first mixed R0 request")
        .clone();
    let mut forked = root.clone();
    let mut forked_registration = original_registration.clone();
    forked_registration.registration_id = "mixed-ancestry-fork".into();
    forked_registration.issuer_request_id = "mixed-ancestry-fork-request".into();
    let mut forked_request = original_request.clone();
    forked_request.registration_id = forked_registration.registration_id.clone();
    forked_request.issuer_request_id = forked_registration.issuer_request_id.clone();
    forked
        .retained_worker_registration_request_index
        .insert(forked_request.issuer_request_id.clone(), forked_request);
    forked.retained_worker_registration_journal.insert(
        forked_registration.registration_id.clone(),
        forked_registration,
    );
    assert!(
        exact_history_is_rejected(&forked),
        "two authenticated candidates at one authority revision must fail closed"
    );

    let mut substituted = root.clone();
    substituted
        .retained_worker_registration_journal
        .get_mut(&first_retained.registration_id)
        .expect("substituted mixed R0 registration")
        .retained_participant_id = "substituted-mixed-worker".into();
    assert!(
        exact_history_is_rejected(&substituted),
        "a substituted authority edge must fail closed"
    );

    let mut reordered = root.clone();
    let reordered_registration = reordered
        .retained_worker_registration_journal
        .get_mut(&first_retained.registration_id)
        .expect("reordered mixed R0 registration");
    reordered_registration.authority_revision_before += 1;
    assert!(
        exact_history_is_rejected(&reordered),
        "a reordered authority edge must fail closed"
    );

    let mismatched_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "0".repeat(64),
    };
    let mut commitment_mismatched = root.clone();
    let commitment_registration = commitment_mismatched
        .retained_worker_registration_journal
        .get_mut(&first_retained.registration_id)
        .expect("commitment-mismatched mixed R0 registration");
    commitment_registration.authority_record_commitment_after = mismatched_commitment.clone();
    let commitment_request = commitment_mismatched
        .retained_worker_registration_request_index
        .get_mut(&commitment_registration.issuer_request_id)
        .expect("commitment-mismatched mixed R0 request");
    let RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
        authority_record_commitment_after,
        ..
    } = &mut commitment_request.state
    else {
        panic!("mixed R0 request must be applied")
    };
    *authority_record_commitment_after = mismatched_commitment;
    assert!(
        exact_history_is_rejected(&commitment_mismatched),
        "an authority edge with a mismatched resulting commitment must fail closed"
    );
}

#[test]
fn start_turn_settlement_uses_only_exact_completion_and_canonical_attention_evidence() {
    let (_parent, authority, request, registration) = applied_start_for_continuity();
    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("first continuation registration must commit")
    };
    let mut attention = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref,
        registered.authority_revision_after,
    );
    attention.obligation_ledger_read = Some(ObligationLedgerSnapshotReadV1::Complete {
        snapshot: start_attention_snapshot(&registration, registered.authority_revision_after),
    });
    let ObligationLedgerSnapshotReadV1::Complete { snapshot } = attention
        .obligation_ledger_read
        .as_ref()
        .expect("canonical attention evidence")
    else {
        panic!("attention settlement must use a complete canonical ledger cut")
    };
    assert_ne!(
        snapshot.materialization_cut.terminal_event_id, attention.event_id,
        "ledger and wrapper terminal evidence are independent authenticated identities"
    );
    assert_ne!(
        snapshot.materialization_cut.terminal_event_sequence, attention.event_sequence,
        "ledger and wrapper terminal sequences must not be conflated"
    );
    let StartTurnSettlementOutcomeV1::Applied(settled) =
        authority.settle_start_turn(&attention).unwrap()
    else {
        panic!("attention-bearing settlement must commit")
    };
    assert_eq!(
        settled.resulting_posture,
        HostSessionPostureV1::AwaitingAttention
    );
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::AwaitingAttention
    );

    let (_parent, authority, request, registration) = applied_start_for_continuity();
    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("first continuation registration must commit")
    };
    let mut terminal = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref,
        registered.authority_revision_after,
    );
    terminal.kind = StartTurnCompletionKindV1::TerminalFailure {
        reason: "wrapper reported exact inaugural-turn failure".into(),
    };
    let StartTurnSettlementOutcomeV1::Applied(settled) =
        authority.settle_start_turn(&terminal).unwrap()
    else {
        panic!("terminal settlement must commit")
    };
    assert_eq!(settled.resulting_posture, HostSessionPostureV1::Terminal);
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::Terminal
    );
    let committed_transaction = authority.read_a12b_root().unwrap().start_transaction_map
        [&registration.start_transaction_id]
        .clone();
    let committed = authority
        .committed_start_public_result(&committed_transaction)
        .expect("terminal retry must authenticate exact committed failure result");
    assert_eq!(committed.resulting_posture, HostSessionPostureV1::Terminal);
    assert_eq!(
        committed.completion_kind,
        StartTurnCompletionKindV1::TerminalFailure {
            reason: "wrapper reported exact inaugural-turn failure".into()
        }
    );
}

#[test]
fn start_attention_rejects_a_noncanonical_materialization_cut() {
    let (_parent, authority, _request, registration) = applied_start_for_continuity();
    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("first continuation registration must commit")
    };
    let mut settlement = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref,
        registered.authority_revision_after,
    );
    let mut snapshot = start_attention_snapshot(&registration, registered.authority_revision_after);
    snapshot
        .materialization_cut
        .materialized_through_event_sequence = 1;
    settlement.obligation_ledger_read = Some(ObligationLedgerSnapshotReadV1::Complete { snapshot });

    assert!(
        authority.settle_start_turn(&settlement).is_err(),
        "Start must reject a raw snapshot whose journal outruns the authenticated cut"
    );
}

#[test]
fn start_continuity_rejects_reordered_stale_and_substituted_evidence() {
    let (_parent, authority, request, registration) = applied_start_for_continuity();
    let mut fabricated_ref = placeholder_ref(
        "ao_dddddddddddddddddddddddddddddddd",
        AuthorityObjectKindV1::ResumeHandle,
    );
    fabricated_ref.schema_version = 2;
    let settlement = clean_start_settlement_request(&registration, fabricated_ref, 2);
    assert!(authority.settle_start_turn(&settlement).is_err());
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );

    let mut wrong_run = registration.clone();
    wrong_run.run_id = "substituted-run".into();
    assert!(authority.establish_start_continuation(&wrong_run).is_err());
    let mut wrong_session = registration.clone();
    wrong_session.orchestration_session_id = "session-substituted".into();
    assert!(authority
        .establish_start_continuation(&wrong_session)
        .is_err());
    let mut wrong_backend = registration.clone();
    wrong_backend.backend_id = "cli:substituted".into();
    assert!(authority
        .establish_start_continuation(&wrong_backend)
        .is_err());
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::ActiveAttached,
        "backend identity alone cannot change HSA posture"
    );
    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("exact continuation must commit")
    };
    let mut substituted = registration.clone();
    substituted.internal_uaa_session_id = "uaa-substituted".into();
    assert!(authority
        .establish_start_continuation(&substituted)
        .is_err());
    let mut substituted_exchange_evidence = registration.clone();
    substituted_exchange_evidence.exchange_evidence_sha256 = "9".repeat(64);
    assert!(authority
        .establish_start_continuation(&substituted_exchange_evidence)
        .is_err());

    let mut stale_settlement = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref.clone(),
        registered.authority_revision_after,
    );
    stale_settlement.expected_authority_revision = 1;
    assert!(authority.settle_start_turn(&stale_settlement).is_err());
    let mut mismatched_actor = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref,
        registered.authority_revision_after,
    );
    mismatched_actor.protocol_actor = HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
        participant_id: "participant-substituted".into(),
    };
    assert!(authority.settle_start_turn(&mismatched_actor).is_err());
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );
}

#[test]
fn start_continuity_exact_retry_never_joins_across_authority_revisions() {
    let (_parent, authority, _request, registration) = applied_start_for_continuity();
    let StartContinuationOutcomeV1::Applied(registered) = authority
        .establish_start_continuation(&registration)
        .unwrap()
    else {
        panic!("first continuation registration must commit")
    };

    let mut stale_registration = registration.clone();
    stale_registration.expected_authority_revision = registered.authority_revision_after;
    assert!(
        authority
            .establish_start_continuation(&stale_registration)
            .is_err(),
        "registration retry at a substituted authority revision must fail closed"
    );

    let settlement = clean_start_settlement_request(
        &registration,
        registered.resume_handle_ref,
        registered.authority_revision_after,
    );
    let StartTurnSettlementOutcomeV1::Applied(settled) =
        authority.settle_start_turn(&settlement).unwrap()
    else {
        panic!("first Start settlement must commit")
    };
    let mut stale_settlement = settlement;
    stale_settlement.expected_authority_revision = settled.authority_revision_after;
    assert!(
        authority.settle_start_turn(&stale_settlement).is_err(),
        "settlement retry at a substituted authority revision must fail closed"
    );
    let mut substituted_completion_evidence = stale_settlement;
    substituted_completion_evidence.expected_authority_revision = 2;
    substituted_completion_evidence.completion_evidence_sha256 = "8".repeat(64);
    assert!(
        authority
            .settle_start_turn(&substituted_completion_evidence)
            .is_err(),
        "settlement retry with substituted typed completion evidence must fail closed"
    );
}

fn stop_request(current: &ResolvedCurrentAuthorityV1) -> IssueHostSessionStopRequestV1 {
    IssueHostSessionStopRequestV1 {
        intent_id: "stop-intent-1".into(),
        request_id: "stop-request-1".into(),
        orchestration_session_id: current.authority.orchestration_session_id.clone(),
        caller: HostSessionTransitionCallerV1 {
            kind: HostSessionTransitionCallerKindV1::PublicCli,
            caller_participant_id: None,
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        expected_authority: current.observation.clone(),
        authoritative_participant_id: current.caller.participant_id.clone(),
        authoritative_lineage: current.authority.authoritative_participant_lineage.clone(),
        issued_at: timestamp("2026-07-14T12:10:00.000000000Z"),
    }
}

fn stop_completion_request(
    intent: &super::store_schema::HostSessionStopIntentV1,
    delivery_acceptance_id: Option<&str>,
) -> CompleteHostSessionStopRequestV1 {
    let active_delivery = delivery_acceptance_id.is_some();
    CompleteHostSessionStopRequestV1 {
        intent_id: intent.intent_id.clone(),
        request_id: intent.request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        authoritative_participant_id: intent.authoritative_participant_id.clone(),
        caller: HostSessionTransitionCallerV1 {
            kind: if active_delivery {
                HostSessionTransitionCallerKindV1::Repl
            } else {
                HostSessionTransitionCallerKindV1::PublicCli
            },
            caller_participant_id: active_delivery
                .then(|| intent.authoritative_participant_id.clone()),
            auto_attach_obligation_id: None,
            auto_attach_claim_owner: None,
        },
        delivery_acceptance_id: delivery_acceptance_id.map(str::to_string),
        result_id: "stop-result-1".into(),
        completed_at: timestamp("2026-07-14T12:10:20.000000000Z"),
    }
}

#[test]
fn parked_hsa_stop_terminalizes_without_delivery_and_exact_retry_joins() {
    let (_parent, authority, binding) = authority();
    let request = start_request(binding);
    let (current, _) = park_with_resume_handle(&authority, &request);
    let stop_request = stop_request(&current);

    let HostSessionStopIssueOutcomeV1::Issued(intent) =
        authority.issue_stop(&current, &stop_request).unwrap()
    else {
        panic!("first parked Stop issuance must commit")
    };
    let completion_request = stop_completion_request(&intent, None);
    let HostSessionStopCompletionOutcomeV1::Completed(result) =
        authority.complete_stop(&completion_request).unwrap()
    else {
        panic!("first parked Stop closeout must commit")
    };
    assert_eq!(result.resulting_posture, HostSessionPostureV1::Terminal);
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::Terminal
    );

    assert!(matches!(
        authority.complete_stop(&completion_request).unwrap(),
        HostSessionStopCompletionOutcomeV1::Joined(joined) if joined == result
    ));
    assert!(matches!(
        authority.issue_stop(&current, &stop_request).unwrap(),
        HostSessionStopIssueOutcomeV1::AlreadyTerminal(joined) if joined == result
    ));

    let mut substituted = stop_request;
    substituted.request_id = "stop-request-substituted".into();
    assert!(authority.issue_stop(&current, &substituted).is_err());
}

#[test]
fn active_hsa_stop_requires_exact_delivery_and_authenticated_closeout() {
    let (_parent, authority, request, _registration) = applied_start_for_continuity();
    let current = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    let HostSessionStopIssueOutcomeV1::Issued(intent) = authority
        .issue_stop(&current, &stop_request(&current))
        .unwrap()
    else {
        panic!("first active Stop issuance must commit")
    };

    assert!(authority
        .complete_stop(&stop_completion_request(&intent, None))
        .is_err());
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );

    let delivery = AcceptHostSessionStopDeliveryRequestV1 {
        intent_id: intent.intent_id.clone(),
        request_id: intent.request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        authoritative_participant_id: intent.authoritative_participant_id.clone(),
        authority_revision: intent.authority_before.authority_revision,
        authority_record_commitment: intent.authority_record_commitment_before.clone(),
        acceptance_id: "stop-acceptance-1".into(),
        accepted_at: timestamp("2026-07-14T12:10:10.000000000Z"),
    };
    let HostSessionStopDeliveryOutcomeV1::Accepted(accepted) =
        authority.accept_stop_delivery(&delivery).unwrap()
    else {
        panic!("first active Stop delivery must commit")
    };
    assert!(matches!(
        authority.accept_stop_delivery(&delivery).unwrap(),
        HostSessionStopDeliveryOutcomeV1::Joined(joined) if joined == accepted
    ));
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::ActiveAttached,
        "delivery acceptance is not terminal authority"
    );

    let mut wrong_participant = delivery.clone();
    wrong_participant.authoritative_participant_id = "participant-substituted".into();
    assert!(authority.accept_stop_delivery(&wrong_participant).is_err());
    let mut stale_revision = delivery.clone();
    stale_revision.authority_revision += 1;
    assert!(authority.accept_stop_delivery(&stale_revision).is_err());
    let mut wrong_request = delivery.clone();
    wrong_request.request_id = "stop-request-substituted".into();
    assert!(authority.accept_stop_delivery(&wrong_request).is_err());
    let mut wrong_payload = delivery.clone();
    wrong_payload.payload_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
    };
    assert!(authority.accept_stop_delivery(&wrong_payload).is_err());
    let mut regressing_delivery = delivery.clone();
    regressing_delivery.accepted_at = timestamp("2026-07-14T12:09:59.000000000Z");
    assert!(authority
        .accept_stop_delivery(&regressing_delivery)
        .is_err());
    let mut wrong_acceptance = stop_completion_request(&intent, Some(&delivery.acceptance_id));
    wrong_acceptance.delivery_acceptance_id = Some("stop-acceptance-substituted".into());
    assert!(authority.complete_stop(&wrong_acceptance).is_err());
    let mut wrong_completion_request =
        stop_completion_request(&intent, Some(&delivery.acceptance_id));
    wrong_completion_request.request_id = "stop-request-substituted".into();
    assert!(authority.complete_stop(&wrong_completion_request).is_err());
    let mut wrong_completion_participant =
        stop_completion_request(&intent, Some(&delivery.acceptance_id));
    wrong_completion_participant.authoritative_participant_id = "participant-substituted".into();
    assert!(authority
        .complete_stop(&wrong_completion_participant)
        .is_err());
    let mut regressing_completion = stop_completion_request(&intent, Some(&delivery.acceptance_id));
    regressing_completion.completed_at = timestamp("2026-07-14T12:10:09.000000000Z");
    assert!(authority.complete_stop(&regressing_completion).is_err());

    let completion = stop_completion_request(&intent, Some(&delivery.acceptance_id));
    let HostSessionStopCompletionOutcomeV1::Completed(result) =
        authority.complete_stop(&completion).unwrap()
    else {
        panic!("authenticated active Stop closeout must commit")
    };
    assert_eq!(result.resulting_posture, HostSessionPostureV1::Terminal);
    assert!(matches!(
        authority.complete_stop(&completion).unwrap(),
        HostSessionStopCompletionOutcomeV1::Joined(joined) if joined == result
    ));
    let mut conflicting_retry = completion;
    conflicting_retry.completed_at = timestamp("2026-07-14T12:10:21.000000000Z");
    assert!(authority.complete_stop(&conflicting_retry).is_err());
}

#[test]
fn hsa_stop_restart_exact_joins_each_committed_stage() {
    let (_parent, authority, request, _registration) = applied_start_for_continuity();
    let current = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    let home = current.observation.bootstrap_home.physical_path.clone();
    let stop_request = stop_request(&current);
    let HostSessionStopIssueOutcomeV1::Issued(intent) =
        authority.issue_stop(&current, &stop_request).unwrap()
    else {
        panic!("first active Stop issuance must commit")
    };

    drop(authority);
    let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
    assert!(matches!(
        authority.issue_stop(&current, &stop_request).unwrap(),
        HostSessionStopIssueOutcomeV1::Joined(joined) if joined == intent
    ));

    let delivery = AcceptHostSessionStopDeliveryRequestV1 {
        intent_id: intent.intent_id.clone(),
        request_id: intent.request_id.clone(),
        payload_commitment: intent.payload_commitment.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        authoritative_participant_id: intent.authoritative_participant_id.clone(),
        authority_revision: intent.authority_before.authority_revision,
        authority_record_commitment: intent.authority_record_commitment_before.clone(),
        acceptance_id: "stop-acceptance-restart".into(),
        accepted_at: timestamp("2026-07-14T12:10:10.000000000Z"),
    };
    let HostSessionStopDeliveryOutcomeV1::Accepted(accepted) =
        authority.accept_stop_delivery(&delivery).unwrap()
    else {
        panic!("first Stop delivery must commit")
    };

    drop(authority);
    let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
    assert!(matches!(
        authority.accept_stop_delivery(&delivery).unwrap(),
        HostSessionStopDeliveryOutcomeV1::Joined(joined) if joined == accepted
    ));
    let completion = stop_completion_request(&intent, Some(&delivery.acceptance_id));
    let HostSessionStopCompletionOutcomeV1::Completed(result) =
        authority.complete_stop(&completion).unwrap()
    else {
        panic!("first Stop completion must commit")
    };

    drop(authority);
    let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
    assert!(matches!(
        authority.complete_stop(&completion).unwrap(),
        HostSessionStopCompletionOutcomeV1::Joined(joined) if joined == result
    ));
    assert_eq!(
        authority
            .resolve_current_exact(&request.orchestration_session_id, None)
            .unwrap()
            .authority
            .lifecycle_posture,
        HostSessionPostureV1::Terminal
    );
}

#[test]
fn hsa_stop_capability_denial_precedes_intent_mutation() {
    let (_parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.start_contract.capabilities.session_stop = false;
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let current = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    assert!(authority
        .issue_stop(&current, &stop_request(&current))
        .is_err());
    assert!(authority
        .read_a12b_root()
        .unwrap()
        .stop_transaction_map
        .is_empty());
}

fn fork_successor_source() -> (
    tempfile::TempDir,
    HostSessionAuthority,
    IssueHostSessionTransitionRequestV1,
    ResolvedCurrentAuthorityV1,
) {
    let (parent, authority, binding) = authority();
    let request = start_request(binding);
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let home = request
        .workspace_binding
        .authority_store_root
        .physical_path
        .clone();
    let exact_v2 = authority.read_a12a_root().unwrap();
    super::store::upgrade_v2_root_to_v3_test(Path::new(&home), &exact_v2).unwrap();
    let source = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    (parent, authority, request, source)
}

fn world_fork_successor_source() -> (
    tempfile::TempDir,
    HostSessionAuthority,
    IssueHostSessionTransitionRequestV1,
    ResolvedCurrentAuthorityV1,
) {
    let (parent, authority, binding) = authority();
    let mut request = start_request(binding);
    request.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    request
        .start_contract
        .launch_knobs
        .requested_execution_scope = AgentExecutionScopeV1::World;
    request.world_binding = Some(WorldBindingV1 {
        world_id: "world-fork-successor-1".into(),
        world_generation: 1,
    });
    let application = issue_and_claim_start(&authority, &request);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let home = request
        .workspace_binding
        .authority_store_root
        .physical_path
        .clone();
    let exact_v2 = authority.read_a12a_root().unwrap();
    super::store::upgrade_v2_root_to_v3_test(Path::new(&home), &exact_v2).unwrap();
    let source = authority
        .resolve_current_exact(&request.orchestration_session_id, None)
        .unwrap();
    (parent, authority, request, source)
}

fn fork_successor_request(source: &ResolvedCurrentAuthorityV1) -> AllocateForkSuccessorRequestV1 {
    AllocateForkSuccessorRequestV1 {
        schema_version: 1,
        allocation_id: "fork-allocation-1".into(),
        request_id: "fork-request-1".into(),
        authority_store_id: source.observation.authority_store_id.clone(),
        bootstrap_home: source.observation.bootstrap_home.clone(),
        expected_source_root_revision: source.observation.root_revision,
        source_orchestration_session_id: source.authority.orchestration_session_id.clone(),
        source_shell_trace_session_id: source.authority.shell_trace_session_id.clone(),
        source_authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
            authority_revision: source.observation.authority_revision,
            authority_record_commitment: source.observation.authority_record_commitment.clone(),
            active_authoritative_participant_id: source
                .authority
                .active_authoritative_participant_id
                .clone()
                .unwrap(),
            authoritative_lineage_commitment: source
                .observation
                .authoritative_lineage_commitment
                .clone(),
            lifecycle_posture: source.authority.lifecycle_posture,
        },
        source_authoritative_participant_lineage: source
            .authority
            .authoritative_participant_lineage
            .clone(),
        target_orchestration_session_id: "session-fork-successor-1".into(),
        target_shell_trace_session_id: "trace-fork-successor-1".into(),
        target_authoritative_participant_id: "participant-fork-successor-1".into(),
        resulting_authoritative_lineage: source
            .authority
            .authoritative_participant_lineage
            .iter()
            .cloned()
            .chain(std::iter::once("participant-fork-successor-1".into()))
            .collect(),
        workspace_binding: source.authority.workspace_binding.clone(),
        world_binding: source.authority.world_binding.clone(),
        allocated_at: timestamp("2026-07-14T12:02:00.000000000Z"),
    }
}

fn set_fork_source_revision(request: &mut AllocateForkSuccessorRequestV1, revision: u64) {
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_revision, ..
    } = &mut request.source_authority_precondition
    else {
        panic!("fork successor request must use ExpectedRevision")
    };
    *authority_revision = revision;
}

fn set_fork_source_commitment(
    request: &mut AllocateForkSuccessorRequestV1,
    commitment: AuthorityObjectCommitmentV1,
) {
    let HostSessionAuthorityPreconditionV1::ExpectedRevision {
        authority_record_commitment,
        ..
    } = &mut request.source_authority_precondition
    else {
        panic!("fork successor request must use ExpectedRevision")
    };
    *authority_record_commitment = commitment;
}

fn retarget_fork_request(
    request: &mut AllocateForkSuccessorRequestV1,
    suffix: &str,
    target_trace: &str,
    target_participant: &str,
) {
    request.allocation_id = format!("fork-allocation-{suffix}");
    request.request_id = format!("fork-request-{suffix}");
    request.target_orchestration_session_id = format!("session-fork-successor-{suffix}");
    request.target_shell_trace_session_id = target_trace.into();
    request.target_authoritative_participant_id = target_participant.into();
    *request.resulting_authoritative_lineage.last_mut().unwrap() = target_participant.into();
}

#[test]
fn hsa_fork_successor_allocates_parked_authority_and_exact_retry_joins() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let source_before = source.authority.clone();
    let request = fork_successor_request(&source);
    let root_before = authority.read_a12b_root().unwrap();

    let ForkSuccessorAllocationOutcomeV1::Allocated(result) =
        authority.allocate_fork_successor(&request).unwrap()
    else {
        panic!("first fork successor allocation must commit")
    };
    assert_eq!(
        result.target_authority.lifecycle_posture,
        HostSessionPostureV1::ParkedResumable
    );
    assert_eq!(result.target_authority.authority_revision, 1);
    assert_eq!(
        result
            .target_authority
            .active_authoritative_participant_id
            .as_deref(),
        Some(request.target_authoritative_participant_id.as_str())
    );
    assert!(result.target_authority.retained_worker_refs.is_empty());
    assert!(result
        .target_authority
        .internal_resume_handle_refs
        .is_empty());
    assert_eq!(
        authority
            .resolve_current_exact(&request.source_orchestration_session_id, None)
            .unwrap()
            .authority,
        source_before,
        "allocation must not revise source authority"
    );
    let target = authority
        .resolve_current_exact(&request.target_orchestration_session_id, None)
        .unwrap();
    assert_eq!(target.authority, result.target_authority);
    assert!(target
        .host_attach_contract
        .continuity_resume_handle_ref
        .is_none());

    let root_after = authority.read_a12b_root().unwrap();
    assert_eq!(root_after.root_revision, root_before.root_revision + 1);
    assert!(matches!(
        authority.allocate_fork_successor(&request).unwrap(),
        ForkSuccessorAllocationOutcomeV1::Joined(joined) if joined == result
    ));
    assert_eq!(authority.read_a12b_root().unwrap(), root_after);

    let attach = attach_successor_request(&target, "participant-fork-attach-1");
    assert!(matches!(
        authority
            .issue_successor_at(
                &target,
                &attach,
                timestamp("2026-07-14T12:03:00.000000000Z"),
                300,
            )
            .unwrap(),
        SuccessorTransitionIssueOutcomeV1::Issued(_)
    ));
}

#[test]
fn hsa_fork_successor_target_composes_attach_and_retained_registration_history() {
    let (_parent, authority, _start, source) = world_fork_successor_source();
    let request = fork_successor_request(&source);
    let ForkSuccessorAllocationOutcomeV1::Allocated(allocation) =
        authority.allocate_fork_successor(&request).unwrap()
    else {
        panic!("first fork successor allocation must commit")
    };
    let parked = authority
        .resolve_current_exact(&request.target_orchestration_session_id, None)
        .unwrap();
    let attach = attach_successor_request(&parked, "participant-fork-attached-1");
    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &parked,
            &attach,
            timestamp("2026-07-14T12:03:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("fork target Attach must issue")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-fork-attach-1".into(),
        claimant_attempt_id: "attempt-fork-attach-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:03:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("fork target Attach must claim")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("fork target Attach claim must retain claim evidence")
    };
    let application = ApplyHostSessionTransitionRequestV1 {
        intent_id: claimed.intent_id,
        issuer_request_id: claimed.issuer_request_id,
        payload_commitment: claimed.payload_commitment,
        expected_intent_revision: claimed.intent_revision,
        claim_id: claim.claim_id,
        expected_claim_revision: claim_revision,
    };
    assert!(matches!(
        authority
            .apply_successor_at(&application, timestamp("2026-07-14T12:03:20.000000000Z"))
            .unwrap(),
        SuccessorTransitionApplicationOutcomeV1::Applied(_)
    ));
    let attached = authority
        .resolve_current_exact(&request.target_orchestration_session_id, None)
        .unwrap();
    assert_eq!(attached.authority.authority_revision, 2);
    assert_eq!(
        attached.authority.lifecycle_posture,
        HostSessionPostureV1::ActiveAttached
    );

    let plan = RetainedWorkerRegistrationPlanV1 {
        registration_request_id: "fork-target-retained-registration-1".into(),
        orchestration_session_id: request.target_orchestration_session_id.clone(),
        expected_authority: RetainedWorkerAuthorityPreconditionV1 {
            authority_store_id: attached.observation.authority_store_id.clone(),
            authority_revision: attached.observation.authority_revision,
            authority_record_commitment: attached.observation.authority_record_commitment.clone(),
        },
        retained_participant_id: "participant-fork-retained-target-1".into(),
        descriptor: AgentDescriptorV1 {
            schema_version: 1,
            agent_id: "fork-retained-worker-1".into(),
            backend_id: "cli:codex-world".into(),
            backend_kind: RuntimeBackendKindV1::Codex,
            protocol: "substrate.agent.session".into(),
            execution_scope: AgentExecutionScopeV1::World,
            binary_path: "/usr/bin/codex".into(),
        },
        internal_uaa_session_id: "uaa-fork-retained-target-1".into(),
    };
    let runtime = RetainedWorkerRuntime;
    let first = runtime
        .register_retained_target(&authority, &plan)
        .expect("fork target retained registration must commit");
    let joined = runtime
        .register_retained_target(&authority, &plan)
        .expect("fork target retained registration exact retry must join");
    assert_eq!(joined, first);
    let retained = authority
        .resolve_current_exact(&request.target_orchestration_session_id, None)
        .unwrap();
    assert_eq!(retained.authority.authority_revision, 3);
    assert_eq!(
        retained.authority.authoritative_participant_lineage.last(),
        Some(&plan.retained_participant_id)
    );
    assert!(matches!(
        authority.allocate_fork_successor(&request).unwrap(),
        ForkSuccessorAllocationOutcomeV1::Joined(joined) if joined == allocation
    ));
}

#[test]
fn hsa_fork_successor_recurses_through_descendant_transition_and_exact_retries() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let parent_request = fork_successor_request(&source);
    let ForkSuccessorAllocationOutcomeV1::Allocated(parent_allocation) =
        authority.allocate_fork_successor(&parent_request).unwrap()
    else {
        panic!("parent fork allocation must commit")
    };
    let parent_target = authority
        .resolve_current_exact(&parent_request.target_orchestration_session_id, None)
        .unwrap();
    let mut child_request = fork_successor_request(&parent_target);
    retarget_fork_request(
        &mut child_request,
        "recursive-child-1",
        "trace-fork-recursive-child-1",
        "participant-fork-recursive-child-1",
    );
    child_request.allocated_at = timestamp("2026-07-14T12:02:30.000000000Z");
    let ForkSuccessorAllocationOutcomeV1::Allocated(child_allocation) =
        authority.allocate_fork_successor(&child_request).unwrap()
    else {
        panic!("recursive child fork allocation must commit")
    };
    let child_target = authority
        .resolve_current_exact(&child_request.target_orchestration_session_id, None)
        .unwrap();
    assert!(child_target
        .authority
        .authoritative_participant_lineage
        .starts_with(
            &parent_allocation
                .target_authority
                .authoritative_participant_lineage
        ));

    let attach = attach_successor_request(&child_target, "participant-recursive-attach-1");
    let SuccessorTransitionIssueOutcomeV1::Issued(issued) = authority
        .issue_successor_at(
            &child_target,
            &attach,
            timestamp("2026-07-14T12:03:00.000000000Z"),
            300,
        )
        .unwrap()
    else {
        panic!("recursive child transition must issue")
    };
    let claim = ClaimHostSessionTransitionRequestV1 {
        intent_id: issued.intent_id.clone(),
        issuer_request_id: issued.issuer_request_id.clone(),
        payload_commitment: issued.payload_commitment.clone(),
        expected_intent_revision: issued.intent_revision,
        claim_id: "claim-recursive-attach-1".into(),
        claimant_attempt_id: "attempt-recursive-attach-1".into(),
    };
    let SuccessorTransitionClaimOutcomeV1::Claimed(claimed) = authority
        .claim_successor_at(&claim, timestamp("2026-07-14T12:03:10.000000000Z"), 30)
        .unwrap()
    else {
        panic!("recursive child transition must claim")
    };
    let HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } = claimed.state else {
        panic!("recursive child transition must retain claim evidence")
    };
    assert!(matches!(
        authority
            .apply_successor_at(
                &ApplyHostSessionTransitionRequestV1 {
                    intent_id: claimed.intent_id,
                    issuer_request_id: claimed.issuer_request_id,
                    payload_commitment: claimed.payload_commitment,
                    expected_intent_revision: claimed.intent_revision,
                    claim_id: claim.claim_id,
                    expected_claim_revision: claim_revision,
                },
                timestamp("2026-07-14T12:03:20.000000000Z"),
            )
            .unwrap(),
        SuccessorTransitionApplicationOutcomeV1::Applied(_)
    ));
    assert!(matches!(
        authority.allocate_fork_successor(&parent_request).unwrap(),
        ForkSuccessorAllocationOutcomeV1::Joined(joined) if joined == parent_allocation
    ));
    assert!(matches!(
        authority.allocate_fork_successor(&child_request).unwrap(),
        ForkSuccessorAllocationOutcomeV1::Joined(joined) if joined == child_allocation
    ));
}

#[test]
fn hsa_fork_successor_durable_publication_proof_rejects_duplicate_and_reversed_revisions() {
    {
        let (_parent, authority, _start, source) = fork_successor_source();
        let first_request = fork_successor_request(&source);
        authority.allocate_fork_successor(&first_request).unwrap();
        let refreshed_source = authority
            .resolve_current_exact(&source.authority.orchestration_session_id, None)
            .unwrap();
        let mut second_request = fork_successor_request(&refreshed_source);
        retarget_fork_request(
            &mut second_request,
            "duplicate-publication-1",
            "trace-fork-duplicate-publication-1",
            "participant-fork-duplicate-publication-1",
        );
        authority.allocate_fork_successor(&second_request).unwrap();
        let mut substituted = authority.read_a12b_root().unwrap();
        let first_root_revision_after = substituted.fork_successor_allocation_map
            [&first_request.allocation_id]
            .root_revision_after;
        let second = substituted
            .fork_successor_allocation_map
            .get_mut(&second_request.allocation_id)
            .unwrap();
        second.request.expected_source_root_revision = first_root_revision_after - 1;
        second.root_revision_after = first_root_revision_after;
        second.request_commitment = canonical_commitment(&second.request);
        assert!(
            substituted.validate().is_err(),
            "two allocations cannot claim the same root publication revision"
        );
    }

    {
        let (_parent, authority, _start, source) = fork_successor_source();
        let parent_request = fork_successor_request(&source);
        authority.allocate_fork_successor(&parent_request).unwrap();
        let parent_target = authority
            .resolve_current_exact(&parent_request.target_orchestration_session_id, None)
            .unwrap();
        let mut child_request = fork_successor_request(&parent_target);
        retarget_fork_request(
            &mut child_request,
            "reversed-child-1",
            "trace-fork-reversed-child-1",
            "participant-fork-reversed-child-1",
        );
        child_request.allocated_at = timestamp("2026-07-14T12:02:30.000000000Z");
        authority.allocate_fork_successor(&child_request).unwrap();
        let mut substituted = authority.read_a12b_root().unwrap();
        let parent_root_revision_after = substituted.fork_successor_allocation_map
            [&parent_request.allocation_id]
            .root_revision_after;
        assert!(parent_root_revision_after >= 5);
        let child = substituted
            .fork_successor_allocation_map
            .get_mut(&child_request.allocation_id)
            .unwrap();
        child.request.expected_source_root_revision = parent_root_revision_after - 2;
        child.root_revision_after = parent_root_revision_after - 1;
        child.request_commitment = canonical_commitment(&child.request);
        assert!(
            substituted.validate().is_err(),
            "a recursive child cannot claim publication before its source allocation"
        );
    }
}

#[test]
fn hsa_fork_successor_rejects_stale_commitment_and_identity_substitution() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let base = fork_successor_request(&source);
    let root_before = authority.read_a12b_root().unwrap();
    let wrong_commitment = AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".into(),
    };

    let mut stale = base.clone();
    set_fork_source_revision(&mut stale, source.observation.authority_revision + 1);
    assert!(authority.allocate_fork_successor(&stale).is_err());
    let mut mismatched = base.clone();
    set_fork_source_commitment(&mut mismatched, wrong_commitment);
    assert!(authority.allocate_fork_successor(&mismatched).is_err());
    let mut substituted_source = base.clone();
    substituted_source.source_orchestration_session_id = "session-substituted".into();
    assert!(authority
        .allocate_fork_successor(&substituted_source)
        .is_err());
    let mut substituted_participant = base.clone();
    substituted_participant.target_authoritative_participant_id = "participant-substituted".into();
    assert!(authority
        .allocate_fork_successor(&substituted_participant)
        .is_err());
    let mut substituted_lineage = base.clone();
    substituted_lineage
        .resulting_authoritative_lineage
        .reverse();
    assert!(authority
        .allocate_fork_successor(&substituted_lineage)
        .is_err());
    let mut substituted_binding = base;
    substituted_binding
        .workspace_binding
        .workspace_root
        .physical_path
        .push_str("-substituted");
    assert!(authority
        .allocate_fork_successor(&substituted_binding)
        .is_err());
    let mut reordered = fork_successor_request(&source);
    reordered.allocated_at = timestamp("2026-07-14T12:01:09.999999999Z");
    assert!(authority.allocate_fork_successor(&reordered).is_err());
    assert_eq!(authority.read_a12b_root().unwrap(), root_before);
}

#[test]
fn hsa_fork_successor_accepts_exact_source_with_retained_lineage() {
    let (_parent, authority, binding) = authority();
    let mut start = start_request(binding);
    start.start_contract.descriptor.execution_scope = AgentExecutionScopeV1::World;
    start.start_contract.launch_knobs.requested_execution_scope = AgentExecutionScopeV1::World;
    start.world_binding = Some(WorldBindingV1 {
        world_id: "world-fork-retained-source-1".into(),
        world_generation: 1,
    });
    let application = issue_and_claim_start(&authority, &start);
    authority
        .apply_start_at(&application, timestamp("2026-07-14T12:01:10.000000000Z"))
        .unwrap();
    let root_before_retained = authority.read_a12a_root().unwrap();
    let SessionNamespaceRecordV1::Authority(current_authority) =
        &root_before_retained.session_namespace_map[&start.orchestration_session_id]
    else {
        panic!("applied Start must have current authority")
    };
    let expected_authority = RetainedWorkerAuthorityPreconditionV1 {
        authority_store_id: root_before_retained.authority_store_id.clone(),
        authority_revision: current_authority.authority_revision,
        authority_record_commitment: canonical_commitment(&authority_hash_input(current_authority)),
    };
    let descriptor_bytes = super::canonical_json::to_vec(&AgentDescriptorHashInputV1 {
        schema_version: 1,
        descriptor: start.start_contract.descriptor.clone(),
    })
    .unwrap();
    let retained_participant_id = "participant-fork-retained-source-1".to_string();
    let session_id = start.orchestration_session_id.clone();
    let resume_handle_bytes = super::canonical_json::to_vec(&ResumeHandleHashInputV1 {
        schema_version: 1,
        orchestration_session_id: session_id.clone(),
        participant_id: retained_participant_id.clone(),
        backend_id: start.start_contract.descriptor.backend_id.clone(),
        protocol: start.start_contract.descriptor.protocol.clone(),
        internal_uaa_session_id: "uaa-fork-retained-source-1".into(),
    })
    .unwrap();
    let participant_for_worker = retained_participant_id.clone();
    let session_for_worker = session_id.clone();
    let reserved = authority
        .reserve_retained_worker_registration_at(
            "fork-retained-registration-1",
            &session_id,
            &expected_authority,
            &retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
            timestamp("2026-07-14T12:01:30.000000000Z"),
            None,
            move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                super::canonical_json::to_vec(&RetainedWorkerObjectHashInputV1 {
                    schema_version: 1,
                    orchestration_session_id: session_for_worker.clone(),
                    participant_id: participant_for_worker.clone(),
                    world_binding: world_binding.clone(),
                    descriptor_ref: descriptor_ref.clone(),
                    resume_handle_ref: resume_handle_ref.clone(),
                    policy_ref: policy_ref.clone(),
                })
                .map_err(|_| "serialize retained worker")
            },
        )
        .unwrap();
    for (reference, bytes) in [
        (
            &reserved.descriptor_ref,
            reserved.descriptor_bytes.as_slice(),
        ),
        (
            &reserved.resume_handle_ref,
            reserved.resume_handle_bytes.as_slice(),
        ),
        (
            &reserved.retained_worker_ref,
            reserved.retained_worker_bytes.as_slice(),
        ),
    ] {
        authority
            .publish_reserved_retained_object(&reserved, reference, bytes)
            .unwrap();
    }
    authority
        .apply_reserved_retained_worker_registration(&reserved)
        .unwrap();
    let root_after_retained = authority.read_a12a_root().unwrap();
    super::store::upgrade_v2_root_to_v3_test(
        Path::new(&start.workspace_binding.authority_store_root.physical_path),
        &root_after_retained,
    )
    .unwrap();
    let source = authority
        .resolve_current_exact(&start.orchestration_session_id, None)
        .unwrap();
    assert_eq!(
        source
            .authority
            .authoritative_participant_lineage
            .last()
            .map(String::as_str),
        Some(retained_participant_id.as_str())
    );
    assert_ne!(
        source
            .authority
            .active_authoritative_participant_id
            .as_deref(),
        Some(retained_participant_id.as_str())
    );
    let mut request = fork_successor_request(&source);
    request.allocated_at = source.authority.updated_at.clone();
    assert!(matches!(
        authority.allocate_fork_successor(&request).unwrap(),
        ForkSuccessorAllocationOutcomeV1::Allocated(_)
    ));
}

#[test]
fn hsa_fork_successor_durable_source_proof_rejects_coherent_snapshot_substitution() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let request = fork_successor_request(&source);
    authority.allocate_fork_successor(&request).unwrap();
    let mut substituted = authority.read_a12b_root().unwrap();
    let allocation = substituted
        .fork_successor_allocation_map
        .get_mut(&request.allocation_id)
        .unwrap();
    allocation.source_authority_before.updated_at = timestamp("2026-07-14T12:01:11.000000000Z");
    let source_commitment = canonical_commitment(&authority_hash_input(
        allocation.source_authority_before.as_ref(),
    ));
    set_fork_source_commitment(&mut allocation.request, source_commitment);
    allocation.request_commitment = canonical_commitment(&allocation.request);
    assert!(
        substituted.validate().is_err(),
        "a re-committed allocation snapshot must still match reconstructed source history"
    );
}

#[test]
fn hsa_fork_successor_exact_retry_joins_after_source_authority_advances() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let request = fork_successor_request(&source);
    let ForkSuccessorAllocationOutcomeV1::Allocated(result) =
        authority.allocate_fork_successor(&request).unwrap()
    else {
        panic!("first fork allocation must commit")
    };
    let current_source = authority
        .resolve_current_exact(&request.source_orchestration_session_id, None)
        .unwrap();
    let HostSessionStopIssueOutcomeV1::Issued(stop) = authority
        .issue_stop(&current_source, &stop_request(&current_source))
        .unwrap()
    else {
        panic!("source Stop must issue")
    };
    let delivery = AcceptHostSessionStopDeliveryRequestV1 {
        intent_id: stop.intent_id.clone(),
        request_id: stop.request_id.clone(),
        payload_commitment: stop.payload_commitment.clone(),
        orchestration_session_id: stop.orchestration_session_id.clone(),
        authoritative_participant_id: stop.authoritative_participant_id.clone(),
        authority_revision: stop.authority_before.authority_revision,
        authority_record_commitment: stop.authority_record_commitment_before.clone(),
        acceptance_id: "fork-source-stop-acceptance-1".into(),
        accepted_at: timestamp("2026-07-14T12:10:10.000000000Z"),
    };
    assert!(matches!(
        authority.accept_stop_delivery(&delivery).unwrap(),
        HostSessionStopDeliveryOutcomeV1::Accepted(_)
    ));
    assert!(matches!(
        authority
            .complete_stop(&stop_completion_request(
                &stop,
                Some(&delivery.acceptance_id),
            ))
            .unwrap(),
        HostSessionStopCompletionOutcomeV1::Completed(_)
    ));
    assert!(matches!(
        authority.allocate_fork_successor(&request).unwrap(),
        ForkSuccessorAllocationOutcomeV1::Joined(joined) if joined == result
    ));
}

#[test]
fn hsa_fork_successor_fails_closed_for_conflicting_existing_target() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let request = fork_successor_request(&source);
    assert!(matches!(
        authority.allocate_fork_successor(&request).unwrap(),
        ForkSuccessorAllocationOutcomeV1::Allocated(_)
    ));
    let root_before = authority.read_a12b_root().unwrap();
    let mut conflict = request;
    conflict.allocation_id = "fork-allocation-conflict".into();
    conflict.request_id = "fork-request-conflict".into();
    assert!(authority.allocate_fork_successor(&conflict).is_err());
    assert_eq!(authority.read_a12b_root().unwrap(), root_before);
}

#[test]
fn hsa_fork_successor_rejects_pending_start_successor_and_retained_identity_reservations() {
    {
        let (_parent, authority, binding) = authority();
        let source_start = start_request(binding.clone());
        let source_application = issue_and_claim_start(&authority, &source_start);
        authority
            .apply_start_at(
                &source_application,
                timestamp("2026-07-14T12:01:10.000000000Z"),
            )
            .unwrap();
        let mut pending_start = start_request(binding);
        pending_start.intent_id = "intent-pending-start-identity-1".into();
        pending_start.issuer_request_id = "request-pending-start-identity-1".into();
        pending_start.orchestration_session_id = "session-pending-start-identity-1".into();
        pending_start.shell_trace_session_id = "trace-pending-start-identity-1".into();
        pending_start.target_authoritative_participant_id =
            "participant-pending-start-identity-1".into();
        pending_start.resulting_authoritative_lineage =
            vec![pending_start.target_authoritative_participant_id.clone()];
        pending_start.run_id = "run-pending-start-identity-1".into();
        assert!(matches!(
            authority
                .issue_start_at(
                    &pending_start,
                    timestamp("2026-07-14T12:01:30.000000000Z"),
                    300,
                )
                .unwrap(),
            TransitionIssueOutcomeV1::Issued(_)
        ));
        let exact_v2 = authority.read_a12a_root().unwrap();
        super::store::upgrade_v2_root_to_v3_test(
            Path::new(
                &source_start
                    .workspace_binding
                    .authority_store_root
                    .physical_path,
            ),
            &exact_v2,
        )
        .unwrap();
        let source = authority
            .resolve_current_exact(&source_start.orchestration_session_id, None)
            .unwrap();
        let refreshed_source = authority
            .resolve_current_exact(&source.authority.orchestration_session_id, None)
            .unwrap();
        let root_before = authority.read_a12b_root().unwrap();

        let mut trace_conflict = fork_successor_request(&refreshed_source);
        retarget_fork_request(
            &mut trace_conflict,
            "pending-start-trace",
            &pending_start.shell_trace_session_id,
            "participant-fork-pending-start-trace-1",
        );
        assert!(authority.allocate_fork_successor(&trace_conflict).is_err());

        let mut participant_conflict = fork_successor_request(&refreshed_source);
        retarget_fork_request(
            &mut participant_conflict,
            "pending-start-participant",
            "trace-fork-pending-start-participant-1",
            &pending_start.target_authoritative_participant_id,
        );
        assert!(authority
            .allocate_fork_successor(&participant_conflict)
            .is_err());
        assert_eq!(authority.read_a12b_root().unwrap(), root_before);
    }

    {
        let (_parent, authority, _start, source) = fork_successor_source();
        let first_request = fork_successor_request(&source);
        authority.allocate_fork_successor(&first_request).unwrap();
        let first_target = authority
            .resolve_current_exact(&first_request.target_orchestration_session_id, None)
            .unwrap();
        let pending_attach =
            attach_successor_request(&first_target, "participant-pending-successor-identity-1");
        assert!(matches!(
            authority
                .issue_successor_at(
                    &first_target,
                    &pending_attach,
                    timestamp("2026-07-14T12:03:00.000000000Z"),
                    300,
                )
                .unwrap(),
            SuccessorTransitionIssueOutcomeV1::Issued(_)
        ));
        let refreshed_source = authority
            .resolve_current_exact(&source.authority.orchestration_session_id, None)
            .unwrap();
        let mut conflict = fork_successor_request(&refreshed_source);
        retarget_fork_request(
            &mut conflict,
            "pending-successor-participant",
            "trace-fork-pending-successor-participant-1",
            &pending_attach.target_authoritative_participant_id,
        );
        let root_before = authority.read_a12b_root().unwrap();
        assert!(authority.allocate_fork_successor(&conflict).is_err());
        assert_eq!(authority.read_a12b_root().unwrap(), root_before);
    }

    {
        let (_parent, authority, start, source) = world_fork_successor_source();
        let expected_authority = RetainedWorkerAuthorityPreconditionV1 {
            authority_store_id: source.observation.authority_store_id.clone(),
            authority_revision: source.observation.authority_revision,
            authority_record_commitment: source.observation.authority_record_commitment.clone(),
        };
        let descriptor_bytes = super::canonical_json::to_vec(&AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: start.start_contract.descriptor.clone(),
        })
        .unwrap();
        let retained_participant = "participant-pending-retained-identity-1".to_string();
        let resume_handle_bytes = super::canonical_json::to_vec(&ResumeHandleHashInputV1 {
            schema_version: 1,
            orchestration_session_id: source.authority.orchestration_session_id.clone(),
            participant_id: retained_participant.clone(),
            backend_id: start.start_contract.descriptor.backend_id.clone(),
            protocol: start.start_contract.descriptor.protocol.clone(),
            internal_uaa_session_id: "uaa-pending-retained-identity-1".into(),
        })
        .unwrap();
        let retained_for_worker = retained_participant.clone();
        let session_for_worker = source.authority.orchestration_session_id.clone();
        authority
            .reserve_retained_worker_registration_at(
                "fork-pending-retained-identity-1",
                &source.authority.orchestration_session_id,
                &expected_authority,
                &retained_participant,
                descriptor_bytes,
                resume_handle_bytes,
                timestamp("2026-07-14T12:01:30.000000000Z"),
                None,
                move |descriptor_ref, resume_handle_ref, policy_ref, world_binding| {
                    super::canonical_json::to_vec(&RetainedWorkerObjectHashInputV1 {
                        schema_version: 1,
                        orchestration_session_id: session_for_worker.clone(),
                        participant_id: retained_for_worker.clone(),
                        world_binding: world_binding.clone(),
                        descriptor_ref: descriptor_ref.clone(),
                        resume_handle_ref: resume_handle_ref.clone(),
                        policy_ref: policy_ref.clone(),
                    })
                    .map_err(|_| "serialize retained worker")
                },
            )
            .unwrap();
        let refreshed_source = authority
            .resolve_current_exact(&source.authority.orchestration_session_id, None)
            .unwrap();
        let mut conflict = fork_successor_request(&refreshed_source);
        retarget_fork_request(
            &mut conflict,
            "pending-retained-participant",
            "trace-fork-pending-retained-participant-1",
            &retained_participant,
        );
        let root_before = authority.read_a12b_root().unwrap();
        assert!(authority.allocate_fork_successor(&conflict).is_err());
        assert_eq!(authority.read_a12b_root().unwrap(), root_before);
    }
}

#[test]
fn hsa_fork_successor_concurrent_duplicates_create_one_allocation() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let home = source.observation.bootstrap_home.physical_path.clone();
    let request = std::sync::Arc::new(fork_successor_request(&source));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let home = home.clone();
        let request = std::sync::Arc::clone(&request);
        let barrier = std::sync::Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            let authority = HostSessionAuthority::open(Path::new(&home)).unwrap();
            barrier.wait();
            authority.allocate_fork_successor(request.as_ref()).unwrap()
        }));
    }
    let outcomes: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, ForkSuccessorAllocationOutcomeV1::Allocated(_)))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, ForkSuccessorAllocationOutcomeV1::Joined(_)))
            .count(),
        1
    );
    assert_eq!(
        authority
            .read_a12b_root()
            .unwrap()
            .fork_successor_allocation_map
            .len(),
        1
    );
}

#[test]
fn hsa_fork_successor_crash_retry_converges_at_publication_boundary() {
    for (suffix, crash_point, published) in [
        (
            "before",
            ForkSuccessorAllocationCrashPointV1::BeforeRootPublication,
            false,
        ),
        (
            "after",
            ForkSuccessorAllocationCrashPointV1::AfterRootPublication,
            true,
        ),
    ] {
        let (_parent, authority, _start, source) = fork_successor_source();
        let mut request = fork_successor_request(&source);
        request.allocation_id.push_str(suffix);
        request.request_id.push_str(suffix);
        request.target_orchestration_session_id.push_str(suffix);
        request.target_shell_trace_session_id.push_str(suffix);
        request.target_authoritative_participant_id.push_str(suffix);
        *request.resulting_authoritative_lineage.last_mut().unwrap() =
            request.target_authoritative_participant_id.clone();
        assert!(authority
            .allocate_fork_successor_with_crash_point(&request, crash_point)
            .is_err());
        let outcome = authority.allocate_fork_successor(&request).unwrap();
        assert_eq!(
            matches!(outcome, ForkSuccessorAllocationOutcomeV1::Joined(_)),
            published
        );
        assert_eq!(
            authority
                .read_a12b_root()
                .unwrap()
                .fork_successor_allocation_map
                .len(),
            1
        );
    }
}

#[test]
fn hsa_fork_successor_continuity_stripping_object_retries_and_reuses_authoritative_bytes() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let initial_root = authority.read_a12b_root().unwrap();
    let mut continuity_source = source.clone();
    continuity_source
        .host_attach_contract
        .continuity_resume_handle_ref = Some(AuthorityObjectRefV1 {
        schema_version: 1,
        ref_id: "ao_12121212121212121212121212121212".into(),
        object_kind: AuthorityObjectKindV1::ResumeHandle,
        commitment: AuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: "12".repeat(32),
        },
    });

    let orphan = super::fork_successor::prepare_fork_successor_attach(
        &authority,
        &initial_root,
        &continuity_source,
    )
    .unwrap();
    assert!(orphan.requires_index);
    let retried_orphan = super::fork_successor::prepare_fork_successor_attach(
        &authority,
        &initial_root,
        &continuity_source,
    )
    .unwrap();
    assert_eq!(retried_orphan.reference, orphan.reference);
    assert_eq!(retried_orphan.bytes, orphan.bytes);
    assert!(retried_orphan.requires_index);

    let request = fork_successor_request(&source);
    authority.allocate_fork_successor(&request).unwrap();
    let current = authority.read_a12b_root().unwrap();
    let prepared = super::fork_successor::prepare_fork_successor_attach(
        &authority,
        &current,
        &continuity_source,
    )
    .unwrap();
    assert!(prepared.requires_index);

    let mut indexed = current.clone();
    indexed.root_revision += 1;
    insert_present_v3(&mut indexed, &prepared.reference, prepared.bytes.len());
    let allocation = indexed
        .fork_successor_allocation_map
        .get_mut(&request.allocation_id)
        .unwrap();
    allocation.target_authority.host_attach_contract_ref = Some(prepared.reference.clone());
    allocation.target_authority_record_commitment =
        canonical_commitment(&authority_hash_input(allocation.target_authority.as_ref()));
    let SessionNamespaceRecordV1::Authority(target) = indexed
        .session_namespace_map
        .get_mut(&request.target_orchestration_session_id)
        .unwrap()
    else {
        panic!("allocated fork target must be current authority")
    };
    target.host_attach_contract_ref = Some(prepared.reference.clone());
    indexed.validate().unwrap();
    super::store::commit_v3_root_exact_current_opened(
        authority.trusted_root(),
        &current,
        &indexed,
        || Ok(()),
    )
    .unwrap();

    let authoritative = super::fork_successor::prepare_fork_successor_attach(
        &authority,
        &indexed,
        &continuity_source,
    )
    .unwrap();
    assert_eq!(authoritative.reference, prepared.reference);
    assert_eq!(authoritative.bytes, prepared.bytes);
    assert!(!authoritative.requires_index);
}

#[test]
fn hsa_fork_successor_durable_attach_proof_rejects_coherent_target_substitution() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let request = fork_successor_request(&source);
    authority.allocate_fork_successor(&request).unwrap();
    let current = authority.read_a12b_root().unwrap();

    let mut substituted_attach = HostAttachContractHashInputV1 {
        schema_version: 1,
        contract: source.host_attach_contract.clone(),
    };
    substituted_attach.contract.continuity_resume_handle_ref = None;
    substituted_attach.contract.capabilities.event_stream =
        !substituted_attach.contract.capabilities.event_stream;
    let substituted_commitment = canonical_commitment(&substituted_attach);
    let AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } = &substituted_commitment
    else {
        panic!("attach contract commitment must be canonical")
    };
    let substituted_ref = AuthorityObjectRefV1 {
        schema_version: 1,
        ref_id: format!("ao_{}", &digest_hex[..32]),
        object_kind: AuthorityObjectKindV1::HostAttachContract,
        commitment: substituted_commitment,
    };
    let substituted_bytes = super::canonical_json::to_vec(&substituted_attach).unwrap();
    super::store::prepare_typed_object_v3_opened(
        authority.trusted_root(),
        current.root_revision,
        &substituted_ref,
        &substituted_bytes,
        None,
    )
    .unwrap();

    let mut proposed = current.clone();
    proposed.root_revision += 1;
    insert_present_v3(&mut proposed, &substituted_ref, substituted_bytes.len());
    let allocation = proposed
        .fork_successor_allocation_map
        .get_mut(&request.allocation_id)
        .unwrap();
    allocation.target_authority.host_attach_contract_ref = Some(substituted_ref.clone());
    allocation.target_authority_record_commitment =
        canonical_commitment(&authority_hash_input(allocation.target_authority.as_ref()));
    let SessionNamespaceRecordV1::Authority(target) = proposed
        .session_namespace_map
        .get_mut(&request.target_orchestration_session_id)
        .unwrap()
    else {
        panic!("allocated fork target must be current authority")
    };
    target.host_attach_contract_ref = Some(substituted_ref);
    proposed.validate().unwrap();
    assert!(super::store::commit_v3_root_exact_current_opened(
        authority.trusted_root(),
        &current,
        &proposed,
        || Ok(()),
    )
    .is_err());
    assert_eq!(authority.read_a12b_root().unwrap(), current);
}

#[test]
fn hsa_fork_successor_attach_semantics_reject_disabled_source_capability() {
    let (_parent, _authority, start, source) = fork_successor_source();
    let mut source_attach = HostAttachContractHashInputV1 {
        schema_version: 1,
        contract: source.host_attach_contract,
    };
    source_attach.contract.capabilities.session_fork = false;
    let mut target_attach = source_attach.clone();
    target_attach.contract.continuity_resume_handle_ref = None;
    assert!(super::validation::validate_fork_successor_attach_semantics(
        &source_attach,
        &target_attach
    )
    .is_err());
    assert!(start.start_contract.capabilities.session_fork);
}

#[test]
fn hsa_fork_successor_records_no_process_or_fabricated_start_evidence() {
    let (_parent, authority, _start, source) = fork_successor_source();
    let request = fork_successor_request(&source);
    let before = authority.read_a12b_root().unwrap();
    let result = match authority.allocate_fork_successor(&request).unwrap() {
        ForkSuccessorAllocationOutcomeV1::Allocated(result) => result,
        ForkSuccessorAllocationOutcomeV1::Joined(_) => panic!("first allocation must commit"),
    };
    let after = authority.read_a12b_root().unwrap();
    assert_eq!(after.transition_intent_map, before.transition_intent_map);
    assert_eq!(after.application_journal, before.application_journal);
    assert_eq!(after.start_transaction_map, before.start_transaction_map);
    let encoded = serde_json::to_string(&result).unwrap();
    for forbidden in [
        "pid",
        "process",
        "readiness",
        "helper",
        "endpoint",
        "timeout",
        "prompt",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "parked authority must not fabricate {forbidden} evidence"
        );
    }
    assert!(!encoded.contains("\"pid\":0"));
}
