#![cfg(all(unix, target_os = "linux"))]

use base64::Engine as _;
use hyper::body::HttpBody;
use sha2::{Digest as _, Sha256};
use std::{
    collections::HashMap,
    fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tempfile::{tempdir, TempDir};
use tokio::{sync::Barrier, time::timeout};
use transport_api_types::{
    AuthorityObjectKindV1, AuthorityObjectRefV1, DispatchPolicyCommitmentRefCarrierV1,
    DispatchPolicySnapshotCarrierV1, E2DispatchPolicyReservationRefCarrierV1,
    E2LaunchRequestCommitmentV1, E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1,
    ExecuteRequest, ExecuteStreamFrame, MemberDispatchRequestV1, MemberRuntimeBackendKindV1,
    MemberTurnSubmitRequestV1, OpaqueAuthorityCommitmentV1, PolicySnapshotV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    ResolvedMemberRuntimeDescriptorV1, RetainedTurnPolicyCommitmentSubjectV1,
    RetainedWorkerAdmissionCommitmentCarrierV1, RetainedWorkerAuthorityObjectCommitmentV1,
    RetainedWorkerLaunchAuthorityProofV1, RetainedWorkerLaunchWorldBindingV1, WorldBindingRefV1,
    WorldWorkAcceptanceContextV1,
};
use world_api::{SharedWorldBindingSnapshot, SharedWorldBindingState, WorldReuseMode, WorldSpec};
use world_service::WorldService;

const SEED_HOME_ENV: &str = "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME";

struct Harness {
    _root: TempDir,
    service: Arc<WorldService>,
    binding: SharedWorldBindingSnapshot,
    session_id: String,
    participant_id: String,
    launch_count_path: PathBuf,
    binary_path: PathBuf,
    state_root: PathBuf,
}

fn policy_snapshot() -> PolicySnapshotV3 {
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: true,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
            deny_enforcement: None,
            caged_required: false,
            discover: None,
            read: None,
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: true,
                allow_list: vec![".".to_string()],
                deny_list: Vec::new(),
            },
        },
    }
    .canonicalize()
    .expect("canonical test policy")
}

fn write_member_binary(root: &Path) -> (PathBuf, PathBuf) {
    let binary = root.join("member-turn-idempotency-runtime.sh");
    let launch_count = root.join("member-turn-idempotency-runtime.count");
    let script = format!(
        r#"#!/bin/sh
if [ "${{1-}}" = "--version" ]; then
  printf 'codex 1.2.3\n'
  exit 0
fi
if [ "${{1-}}" = "features" ]; then
  printf '{{"features":["add_dir"]}}\n'
  exit 0
fi
if [ "${{1-}}" = "--help" ]; then
  printf 'Usage: codex --add-dir\n'
  exit 0
fi
cat >/dev/null
printf 'provider-child\n' >> '{}'
launch_number=$(wc -l < '{}')
if [ "$launch_number" -eq 1 ]; then
  printf '{{"type":"thread.started","thread_id":"thread-member-turn-idempotency"}}\n'
  printf '{{"type":"turn.started","thread_id":"thread-member-turn-idempotency","turn_id":"turn-bootstrap"}}\n'
  printf '{{"type":"turn.completed","thread_id":"thread-member-turn-idempotency","turn_id":"turn-bootstrap"}}\n'
  exit 0
fi
printf '{{"type":"thread.resumed","thread_id":"thread-member-turn-idempotency"}}\n'
printf '{{"type":"turn.started","thread_id":"thread-member-turn-idempotency","turn_id":"turn-retained"}}\n'
sleep 0.2
printf '{{"type":"item.completed","thread_id":"thread-member-turn-idempotency","turn_id":"turn-retained","item_id":"msg-retained","status":"completed","item_type":"agent_message","content":{{"text":"durable member turn completed"}}}}\n'
printf '{{"type":"turn.completed","thread_id":"thread-member-turn-idempotency","turn_id":"turn-retained"}}\n'
"#,
        launch_count.display(),
        launch_count.display()
    );
    fs::write(&binary, script).expect("write member runtime");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755))
        .expect("make member runtime executable");
    (binary, launch_count)
}

fn write_seed_home(root: &Path) -> PathBuf {
    let seed_home = root.join("seed-home");
    fs::create_dir(&seed_home).expect("create seed home");
    fs::write(
        seed_home.join("auth.json"),
        r#"{"account_id":"acct_test","access_token":"token_test"}"#,
    )
    .expect("write seed auth");
    fs::write(seed_home.join("config.toml"), "model = \"gpt-5.4\"\n").expect("write seed config");
    seed_home
}

fn cap_ref() -> DispatchPolicyCommitmentRefCarrierV1 {
    DispatchPolicyCommitmentRefCarrierV1 {
        authority_store_id: "authority-store-member-turn-idempotency".to_string(),
        commitment_id: "dpc_018f0f3a-9b2c-7def-8abc-0123456789ad".to_string(),
        exact_linkage_hash: "a".repeat(64),
    }
}

fn attach_launch_authority(request: &mut ExecuteRequest) {
    let bytes = serde_json::to_vec(&request.policy_snapshot).expect("launch policy bytes");
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let dispatch = request.member_dispatch.as_mut().expect("member dispatch");
    let object_commitment =
        |value: char| RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: value.to_string().repeat(64),
        };
    dispatch.retained_worker_launch_authority = Some(RetainedWorkerLaunchAuthorityProofV1 {
        schema_version: 1,
        authority_store_id: "has_member_turn_idempotency".to_string(),
        issuer_request_id: "req_member_turn_idempotency_bootstrap".to_string(),
        canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
            schema_version: 1,
            algorithm: "hmac-sha-256".to_string(),
            key_id: "adk_member_turn_idempotency".to_string(),
            digest_hex: "b".repeat(64),
        },
        registration_id: "rwr_member_turn_idempotency".to_string(),
        registration_commitment: object_commitment('c'),
        authority_revision_after: 2,
        authority_record_commitment_after: object_commitment('d'),
        orchestration_session_id: dispatch.orchestration_session_id.clone(),
        caller_participant_id: dispatch.orchestrator_participant_id.clone(),
        retained_participant_id: dispatch.participant_id.clone(),
        bootstrap_run_id: dispatch.run_id.clone(),
        transport_claim_id: "rtc_member_turn_idempotency".to_string(),
        backend_id: dispatch.backend_id.clone(),
        protocol: dispatch.protocol.clone(),
        world_binding: RetainedWorkerLaunchWorldBindingV1 {
            world_id: dispatch.world_id.clone(),
            world_generation: dispatch.world_generation,
        },
        current_policy_ref_id: "ao_policy_member_turn_idempotency".to_string(),
        current_policy_revision: hash.clone(),
        retained_worker_ref_id: "ao_worker_member_turn_idempotency".to_string(),
        retained_worker_commitment: object_commitment('e'),
    });
    let worker_cap = cap_ref();
    dispatch.e2_launch_activation = Some(E2MemberLaunchActivationCarrierV1 {
        schema_version: 1,
        activation_id: format!("e2a_{}", "a".repeat(32)),
        launch_kind: E2MemberLaunchKindV1::FreshSpawn,
        reservation_ref: Some(E2DispatchPolicyReservationRefCarrierV1 {
            authority_store_id: worker_cap.authority_store_id.clone(),
            reservation_id: "dpr_member_turn_idempotency".to_string(),
            reservation_hash: "f".repeat(64),
        }),
        commitment_ref: worker_cap.clone(),
        immutable_worker_cap_ref: worker_cap,
        immutable_worker_cap_created_revision: 7,
        immutable_worker_cap_application_revision: 9,
        policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
        policy_snapshot_byte_length: bytes.len() as u64,
        policy_snapshot_ref: AuthorityObjectRefV1 {
            ref_id: "ao_11111111111111111111111111111111".to_string(),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: "1".repeat(64),
            },
        },
        policy_snapshot_hash: hash.clone(),
        policy_snapshot_revision: hash,
        reason: Some("authenticated member-turn idempotency launch cap".to_string()),
        request_id: "req_member_turn_idempotency_bootstrap".to_string(),
        idempotency_key: "idem_member_turn_idempotency_bootstrap".to_string(),
        orchestration_session_id: dispatch.orchestration_session_id.clone(),
        caller_participant_id: dispatch.orchestrator_participant_id.clone(),
        caller_backend_id: dispatch.backend_id.clone(),
        target_backend_id: dispatch.backend_id.clone(),
        retained_participant_id: dispatch.participant_id.clone(),
        bootstrap_run_id: dispatch.run_id.clone(),
        source_participant_id: None,
        target_world: WorldBindingRefV1 {
            world_id: dispatch.world_id.clone(),
            world_generation: dispatch.world_generation,
        },
        parent_policy_ref: AuthorityObjectRefV1 {
            ref_id: "ao_22222222222222222222222222222222".to_string(),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: "2".repeat(64),
            },
        },
        parent_policy_revision: "parent-member-turn-idempotency".to_string(),
        request_commitment: E2LaunchRequestCommitmentV1::HmacSha256 {
            key_id: "dpk_member_turn_idempotency".to_string(),
            domain: "substrate.dispatch_policy_commitment.spawn_request.v1".to_string(),
            digest_hex: "3".repeat(64),
        },
        registry_publication_revision: 9,
    });
}

async fn collect_body<B>(mut body: B) -> Vec<u8>
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut bytes = Vec::new();
    while let Some(chunk) = timeout(Duration::from_secs(10), body.data())
        .await
        .expect("member-turn stream timed out")
    {
        bytes.extend_from_slice(&chunk.expect("member-turn stream chunk"));
    }
    bytes
}

fn frames(bytes: &[u8]) -> Vec<ExecuteStreamFrame> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("canonical member-turn frame"))
        .collect()
}

fn provider_launch_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .expect("read provider launch count")
        .lines()
        .count()
}

async fn launch_harness() -> Option<Harness> {
    let root = tempdir().expect("member-turn integration root");
    let state_root = root.path().join("member-turn-state");
    fs::create_dir(&state_root).expect("create member-turn state root");
    fs::set_permissions(&state_root, fs::Permissions::from_mode(0o700))
        .expect("secure member-turn state root");
    let service = match WorldService::new_with_member_turn_state_root_for_test(&state_root) {
        Ok(service) => Arc::new(service),
        Err(error) => {
            eprintln!("skipping member-turn idempotency integration: {error}");
            return None;
        }
    };
    let (binary, launch_count_path) = write_member_binary(root.path());
    let seed_home = write_seed_home(root.path());
    let fixture_id = uuid::Uuid::now_v7();
    let session_id = format!("orch-member-turn-idempotency-{fixture_id}");
    let participant_id = format!("ash_member_turn_idempotency_{fixture_id}");
    let world_spec = WorldSpec {
        reuse_session: true,
        reuse_mode: WorldReuseMode::GenericCompatible,
        isolate_network: false,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: Vec::new(),
        project_dir: root.path().to_path_buf(),
        always_isolate: true,
        fs_mode: substrate_common::WorldFsMode::Writable,
        backend_policy: None,
    };
    let world = match service.ensure_session_world(&world_spec) {
        Ok(world) => world,
        Err(error) => {
            eprintln!("skipping member-turn idempotency integration: {error}");
            return None;
        }
    };
    let binding = SharedWorldBindingSnapshot {
        orchestration_session_id: session_id.clone(),
        world_id: world.id,
        world_generation: 1,
        binding_state: SharedWorldBindingState::Active,
    };
    let mut env = HashMap::new();
    env.insert(
        "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT".to_string(),
        "1".to_string(),
    );
    env.insert(SEED_HOME_ENV.to_string(), seed_home.display().to_string());
    env.insert(
        "SUBSTRATE_WORLD_PROJECT_DIR".to_string(),
        root.path().display().to_string(),
    );
    let mut request = ExecuteRequest {
        profile: None,
        cmd: String::new(),
        cwd: Some(root.path().display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "member-turn-idempotency-test".to_string(),
        budget: None,
        policy_snapshot: policy_snapshot(),
        shared_world: None,
        world_network: None,
        world_fs_mode: None,
        acceptance_context: None,
        member_dispatch: Some(MemberDispatchRequestV1 {
            schema_version: 1,
            orchestration_session_id: session_id.clone(),
            participant_id: participant_id.clone(),
            orchestrator_participant_id: "ash_orchestrator_member_turn_idempotency".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: "run-member-turn-idempotency-bootstrap".to_string(),
            world_id: binding.world_id.clone(),
            world_generation: binding.world_generation,
            initial_prompt: Some("bootstrap member-turn idempotency runtime".to_string()),
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: binary.display().to_string(),
            },
            retained_worker_launch_authority: None,
            e2_launch_activation: None,
        }),
    };
    attach_launch_authority(&mut request);
    let bootstrap = service
        .execute_stream(request)
        .await
        .expect("launch retained member");
    let bootstrap_bytes = collect_body(bootstrap.into_body()).await;
    assert!(
        frames(&bootstrap_bytes)
            .iter()
            .any(|frame| matches!(frame, ExecuteStreamFrame::Exit { exit: 0, .. })),
        "retained bootstrap must complete successfully"
    );
    assert_eq!(provider_launch_count(&launch_count_path), 1);
    Some(Harness {
        _root: root,
        service,
        binding,
        session_id,
        participant_id,
        launch_count_path,
        binary_path: binary,
        state_root,
    })
}

fn turn_request(harness: &Harness, run_id: &str, prompt: &str) -> MemberTurnSubmitRequestV1 {
    let message_id = format!("wwm_{}", uuid::Uuid::now_v7());
    let acceptance_record_id = format!("wwa_{}", uuid::Uuid::now_v7());
    let snapshot = policy_snapshot();
    let bytes = serde_json::to_vec(&snapshot).expect("turn snapshot bytes");
    let mut request = MemberTurnSubmitRequestV1 {
        schema_version: 1,
        orchestration_session_id: harness.session_id.clone(),
        participant_id: harness.participant_id.clone(),
        orchestrator_participant_id: "ash_orchestrator_member_turn_idempotency".to_string(),
        backend_id: "cli:codex".to_string(),
        run_id: run_id.to_string(),
        world_id: harness.binding.world_id.clone(),
        world_generation: harness.binding.world_generation,
        prompt: prompt.to_string(),
        policy_snapshot_carrier: None,
        acceptance_context: Some(WorldWorkAcceptanceContextV1 {
            schema_version: 1,
            proposed_acceptance_record_id: acceptance_record_id,
            request_id: run_id.to_string(),
            message_id: Some(message_id.clone()),
            caller_backend_id: "cli:codex".to_string(),
            host_transition_correlation: None,
        }),
    };
    request.policy_snapshot_carrier = Some(DispatchPolicySnapshotCarrierV1 {
        schema_version: 1,
        immutable_worker_cap_ref: cap_ref(),
        immutable_worker_cap_created_revision: 7,
        immutable_worker_cap_application_revision: 9,
        subject: RetainedTurnPolicyCommitmentSubjectV1 {
            retained_participant_id: harness.participant_id.clone(),
            active_run_id: run_id.to_string(),
            message_id: Some(message_id),
        },
        orchestration_session_id: harness.session_id.clone(),
        caller_participant_id: "ash_orchestrator_member_turn_idempotency".to_string(),
        caller_backend_id: "cli:codex".to_string(),
        target_backend_id: "cli:codex".to_string(),
        target_world: WorldBindingRefV1 {
            world_id: harness.binding.world_id.clone(),
            world_generation: harness.binding.world_generation,
        },
        policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
        policy_snapshot_byte_length: bytes.len() as u64,
        policy_snapshot_ref: AuthorityObjectRefV1 {
            ref_id: "ao_33333333333333333333333333333333".to_string(),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: "4".repeat(64),
            },
        },
        policy_snapshot_hash: format!("{:x}", Sha256::digest(&bytes)),
        policy_snapshot_revision: "member-turn-idempotency-revision".to_string(),
        reason: Some("authenticated member-turn idempotency request".to_string()),
    });
    request.validate().expect("valid E2 member-turn request");
    request
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn first_writer_and_retry_return_the_same_durable_prelaunch_failure() {
    let Some(harness) = launch_harness().await else {
        return;
    };
    fs::remove_file(&harness.binary_path).expect("remove retained binary after bootstrap");
    let request = turn_request(
        &harness,
        "run-member-turn-idempotency-prelaunch-failure",
        "fail before launching the submitted turn",
    );

    let first = harness
        .service
        .submit_member_turn_stream(request.clone())
        .await
        .expect_err("first writer must publish a typed prelaunch failure");
    let retry = harness
        .service
        .submit_member_turn_stream(request)
        .await
        .expect_err("retry must replay the typed prelaunch failure");
    assert_eq!(first.to_string(), "turn_request_preparation_failed_v1");
    assert_eq!(retry.to_string(), first.to_string());
    assert_eq!(
        provider_launch_count(&harness.launch_count_path),
        1,
        "prelaunch failure and retry must not start a submitted-turn child"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn thirty_two_identical_requests_launch_one_child_and_replay_exactly() {
    let Some(harness) = launch_harness().await else {
        return;
    };
    let request = turn_request(
        &harness,
        "run-member-turn-idempotency-32-way",
        "execute the durable idempotency turn",
    );
    let acceptance_record_id = request
        .acceptance_context
        .as_ref()
        .expect("acceptance context")
        .proposed_acceptance_record_id
        .clone();
    let barrier = Arc::new(Barrier::new(32));
    let mut tasks = Vec::new();
    for _ in 0..32 {
        let service = harness.service.clone();
        let request = request.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            let response = service
                .submit_member_turn_stream(request)
                .await
                .expect("exact concurrent request must join");
            collect_body(response.into_body()).await
        }));
    }
    let mut responses = Vec::new();
    for task in tasks {
        responses.push(task.await.expect("concurrent request task"));
    }
    assert!(
        responses
            .windows(2)
            .all(|pair| pair[0].as_slice() == pair[1].as_slice()),
        "all exact joiners must receive byte-identical durable frames"
    );
    let replayed_frames = frames(&responses[0]);
    assert!(matches!(
        replayed_frames.first(),
        Some(ExecuteStreamFrame::Start { .. })
    ));
    assert!(matches!(
        replayed_frames.last(),
        Some(ExecuteStreamFrame::Exit { exit: 0, .. })
    ));
    assert_eq!(
        provider_launch_count(&harness.launch_count_path),
        2,
        "bootstrap plus exactly one submitted-turn provider child"
    );

    let completed_retry = harness
        .service
        .submit_member_turn_stream(request.clone())
        .await
        .expect("completed exact retry");
    assert_eq!(
        collect_body(completed_retry.into_body()).await,
        responses[0]
    );
    assert_eq!(provider_launch_count(&harness.launch_count_path), 2);

    let stream_id = match replayed_frames.first().expect("durable Start") {
        ExecuteStreamFrame::Start { frame_identity, .. } => frame_identity.stream_id.clone(),
        other => panic!("durable replay did not begin with Start: {other:?}"),
    };
    let restarted = WorldService::new_with_member_turn_state_root_for_test(&harness.state_root)
        .expect("restart WorldService with the same durable member-turn root");
    let restarted_post = restarted
        .submit_member_turn_stream(request.clone())
        .await
        .expect("completed exact POST retry survives WorldService restart");
    assert_eq!(collect_body(restarted_post.into_body()).await, responses[0]);
    assert_eq!(provider_launch_count(&harness.launch_count_path), 2);

    let mut changed = request.clone();
    changed.prompt = "changed private prompt".to_string();
    let error = match restarted.submit_member_turn_stream(changed).await {
        Ok(_) => panic!("changed restarted POST must conflict before provider launch"),
        Err(error) => error,
    };
    assert!(error
        .to_string()
        .contains("member_turn_request_conflict_v1"));

    let absent = turn_request(
        &harness,
        "run-member-turn-idempotency-absent-after-restart",
        "this request has no durable reservation",
    );
    let error = match restarted.submit_member_turn_stream(absent).await {
        Ok(_) => panic!("absent durable request must still require a live retained member"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("is not retained"));

    let replay = restarted
        .replay_execute_stream(transport_api_types::ExecuteStreamReplayRequestV1 {
            schema_version: 1,
            acceptance_record_id,
            stream_id,
            after_frame_sequence: 0,
        })
        .await
        .expect("completed durable replay remains available after restart");
    assert_eq!(collect_body(replay.into_body()).await, responses[0]);
    assert_eq!(provider_launch_count(&harness.launch_count_path), 2);

    let mut changed = request;
    changed.prompt = "changed private prompt".to_string();
    let error = match harness.service.submit_member_turn_stream(changed).await {
        Ok(_) => panic!("changed request must conflict before provider launch"),
        Err(error) => error,
    };
    assert!(error
        .to_string()
        .contains("member_turn_request_conflict_v1"));
    assert_eq!(provider_launch_count(&harness.launch_count_path), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn caller_disconnect_does_not_cancel_durable_launch_authority() {
    let Some(harness) = launch_harness().await else {
        return;
    };
    let request = turn_request(
        &harness,
        "run-member-turn-idempotency-disconnect",
        "continue after the original caller disconnects",
    );
    let response = harness
        .service
        .submit_member_turn_stream(request.clone())
        .await
        .expect("initial submitted turn");
    drop(response);

    let restarted = WorldService::new_with_member_turn_state_root_for_test(&harness.state_root)
        .expect("restart while the durable submitted turn is Started");
    let joined = restarted
        .submit_member_turn_stream(request)
        .await
        .expect("same POST joins the durable Started turn after restart");
    let joined_bytes = collect_body(joined.into_body()).await;
    assert!(matches!(
        frames(&joined_bytes).last(),
        Some(ExecuteStreamFrame::Exit { exit: 0, .. })
    ));
    assert_eq!(
        provider_launch_count(&harness.launch_count_path),
        2,
        "dropping the original response must not launch a second submitted child"
    );
}
