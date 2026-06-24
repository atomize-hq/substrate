#![cfg(all(unix, target_os = "linux"))]

use hyper::body::HttpBody;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tokio::time::timeout;
use transport_api_types::{
    ExecuteRequest, MemberDispatchRequestV1, MemberRuntimeBackendKindV1, MemberTurnSubmitRequestV1,
    PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1,
};
use world_api::{
    SharedWorldBindingSnapshot, SharedWorldOwnerAction, SharedWorldOwnerSpec, WorldReuseMode,
    WorldSpec,
};
use world_service::WorldService;

const SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV: &str = "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME";

fn minimal_policy_snapshot() -> PolicySnapshotV3 {
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
}

fn make_member_dispatch_request(
    cwd: &Path,
    binary_path: &Path,
    seed_home: &Path,
    binding: &SharedWorldBindingSnapshot,
    orchestration_session_id: &str,
    participant_id: &str,
    run_id: &str,
) -> ExecuteRequest {
    let mut env = HashMap::new();
    env.insert(
        "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT".to_string(),
        "1".to_string(),
    );
    env.insert(
        SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
        seed_home.display().to_string(),
    );
    ExecuteRequest {
        profile: None,
        cmd: String::new(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "member-runtime-retained-lifecycle-test".to_string(),
        budget: None,
        policy_snapshot: minimal_policy_snapshot(),
        shared_world: None,
        world_network: None,
        world_fs_mode: None,
        member_dispatch: Some(MemberDispatchRequestV1 {
            schema_version: 1,
            orchestration_session_id: orchestration_session_id.to_string(),
            participant_id: participant_id.to_string(),
            orchestrator_participant_id: "ash_orchestrator_retained_lifecycle_test".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: run_id.to_string(),
            world_id: binding.world_id.clone(),
            world_generation: binding.world_generation,
            initial_prompt: Some("bootstrap prompt".to_string()),
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: binary_path.display().to_string(),
            },
        }),
    }
}

fn write_seed_home(temp: &Path) -> PathBuf {
    let seed_home = temp.join("seed-home");
    fs::create_dir_all(&seed_home).expect("create seed home");
    fs::write(
        seed_home.join("auth.json"),
        r#"{"account_id":"acct_test","access_token":"token_test"}"#,
    )
    .expect("write seed auth");
    fs::write(seed_home.join("config.toml"), "model = \"gpt-5.4\"\n").expect("write seed config");
    seed_home
}

fn make_member_turn_submit_request(
    orchestration_session_id: &str,
    participant_id: &str,
    world_id: &str,
    world_generation: u64,
    run_id: &str,
    prompt: &str,
) -> MemberTurnSubmitRequestV1 {
    MemberTurnSubmitRequestV1 {
        schema_version: 1,
        orchestration_session_id: orchestration_session_id.to_string(),
        participant_id: participant_id.to_string(),
        orchestrator_participant_id: "ash_orchestrator_retained_lifecycle_test".to_string(),
        backend_id: "cli:codex".to_string(),
        run_id: run_id.to_string(),
        world_id: world_id.to_string(),
        world_generation,
        prompt: prompt.to_string(),
    }
}

fn write_clean_bootstrap_then_resume_member_runtime(temp: &Path) -> (PathBuf, PathBuf) {
    let path = temp.join("member-runtime-clean-bootstrap-then-resume.sh");
    let count_path = temp.join("member-runtime-clean-bootstrap-then-resume.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\nif [ \"${{1-}}\" = \"--version\" ]; then\n  printf 'codex 1.2.3\\n'\n  exit 0\nfi\nif [ \"${{1-}}\" = \"features\" ] && [ \"${{2-}}\" = \"list\" ]; then\n  if [ \"${{3-}}\" = \"--json\" ]; then\n    printf '{{\"features\":[\"add_dir\"]}}\\n'\n  else\n    printf 'add_dir\\n'\n  fi\n  exit 0\nfi\nif [ \"${{1-}}\" = \"--help\" ]; then\n  printf 'Usage: codex --add-dir\\n'\n  exit 0\nfi\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/member-runtime-$count.args\"\ncat > \"$SCRIPT_DIR/member-runtime-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-member-retained\"}}\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  exit 0\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-member-retained\"}}\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\"}}\\n'\nprintf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\",\"item_id\":\"msg-2\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success\"}}}}\\n'\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\"}}\\n'\nexit 0\n",
        count_path.display(),
        temp.display()
    );
    fs::write(&path, body).expect("write clean-bootstrap member runtime");
    let mut perms = fs::metadata(&path)
        .expect("clean-bootstrap member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set clean-bootstrap member runtime permissions");
    (path, count_path)
}

fn write_clean_bootstrap_fail_submit_turn_then_resume_member_runtime(
    temp: &Path,
) -> (PathBuf, PathBuf) {
    let path = temp.join("member-runtime-clean-bootstrap-fail-submit-turn-then-resume.sh");
    let count_path = temp.join("member-runtime-clean-bootstrap-fail-submit-turn-then-resume.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\nif [ \"${{1-}}\" = \"--version\" ]; then\n  printf 'codex 1.2.3\\n'\n  exit 0\nfi\nif [ \"${{1-}}\" = \"features\" ] && [ \"${{2-}}\" = \"list\" ]; then\n  if [ \"${{3-}}\" = \"--json\" ]; then\n    printf '{{\"features\":[\"add_dir\"]}}\\n'\n  else\n    printf 'add_dir\\n'\n  fi\n  exit 0\nfi\nif [ \"${{1-}}\" = \"--help\" ]; then\n  printf 'Usage: codex --add-dir\\n'\n  exit 0\nfi\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/member-runtime-fail-then-resume-$count.args\"\ncat > \"$SCRIPT_DIR/member-runtime-fail-then-resume-$count.stdin\"\nif [ \"$count\" -eq 1 ]; then\n  printf '{{\"type\":\"thread.started\",\"thread_id\":\"thread-member-retained\"}}\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  printf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-1\"}}\\n'\n  exit 0\nfi\nif [ \"$count\" -eq 2 ]; then\n  printf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-member-retained\"}}\\n'\n  printf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\"}}\\n'\n  printf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-2\",\"item_id\":\"msg-2\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt failure\"}}}}\\n'\n  exit 17\nfi\nprintf '{{\"type\":\"thread.resumed\",\"thread_id\":\"thread-member-retained\"}}\\n'\nprintf '{{\"type\":\"turn.started\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-3\"}}\\n'\nprintf '{{\"type\":\"item.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-3\",\"item_id\":\"msg-3\",\"status\":\"completed\",\"item_type\":\"agent_message\",\"content\":{{\"text\":\"follow-up prompt success after failure\"}}}}\\n'\nprintf '{{\"type\":\"turn.completed\",\"thread_id\":\"thread-member-retained\",\"turn_id\":\"turn-3\"}}\\n'\nexit 0\n",
        count_path.display(),
        temp.display()
    );
    fs::write(&path, body).expect("write fail-then-resume member runtime");
    let mut perms = fs::metadata(&path)
        .expect("fail-then-resume member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fail-then-resume member runtime permissions");
    (path, count_path)
}

fn write_member_runtime_without_session_handle(temp: &Path) -> (PathBuf, PathBuf) {
    let path = temp.join("member-runtime-without-session-handle.sh");
    let count_path = temp.join("member-runtime-without-session-handle.count");
    let body = format!(
        "#!/bin/sh\nSTATE_FILE='{}'\nSCRIPT_DIR='{}'\nif [ \"${{1-}}\" = \"--version\" ]; then\n  printf 'codex 1.2.3\\n'\n  exit 0\nfi\nif [ \"${{1-}}\" = \"features\" ] && [ \"${{2-}}\" = \"list\" ]; then\n  if [ \"${{3-}}\" = \"--json\" ]; then\n    printf '{{\"features\":[\"add_dir\"]}}\\n'\n  else\n    printf 'add_dir\\n'\n  fi\n  exit 0\nfi\nif [ \"${{1-}}\" = \"--help\" ]; then\n  printf 'Usage: codex --add-dir\\n'\n  exit 0\nfi\ncount=0\nif [ -f \"$STATE_FILE\" ]; then\n  count=$(cat \"$STATE_FILE\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$STATE_FILE\"\nprintf '%s\\n' \"$@\" > \"$SCRIPT_DIR/no-session-handle-$count.args\"\ncat > \"$SCRIPT_DIR/no-session-handle-$count.stdin\"\nprintf 'bootstrap-without-session-handle\\n'\nexit 0\n",
        count_path.display(),
        temp.display()
    );
    fs::write(&path, body).expect("write no-session-handle member runtime");
    let mut perms = fs::metadata(&path)
        .expect("no-session-handle member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set no-session-handle member runtime permissions");
    (path, count_path)
}

async fn next_optional_stream_frame_value<B>(body: &mut B, buffer: &mut Vec<u8>) -> Option<Value>
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    loop {
        if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            let payload = &line[..line.len() - 1];
            return Some(serde_json::from_slice(payload).expect("valid stream frame json value"));
        }

        match timeout(Duration::from_secs(5), body.data())
            .await
            .expect("timed out waiting for stream chunk")
        {
            Some(Ok(chunk)) => buffer.extend_from_slice(&chunk),
            Some(Err(err)) => panic!("stream chunk error: {err}"),
            None => {
                assert!(
                    buffer.is_empty(),
                    "stream ended mid-frame with leftover bytes: {buffer:?}"
                );
                return None;
            }
        }
    }
}

fn frame_start_span_id(frame: &Value) -> Option<&str> {
    if frame.get("type")?.as_str() == Some("start") {
        return frame.get("span_id")?.as_str();
    }
    frame.get("Start")?.get("span_id")?.as_str()
}

fn frame_exit(frame: &Value) -> Option<(i32, &str)> {
    if frame.get("type")?.as_str() == Some("exit") {
        let exit = frame.get("exit")?.as_i64()?;
        let span_id = frame.get("span_id")?.as_str()?;
        return Some((exit as i32, span_id));
    }
    let exit = frame.get("Exit")?.get("exit")?.as_i64()?;
    let span_id = frame.get("Exit")?.get("span_id")?.as_str()?;
    Some((exit as i32, span_id))
}

fn frame_event(frame: &Value) -> Option<&Value> {
    if frame.get("type").and_then(Value::as_str) == Some("event") {
        return frame.get("event");
    }
    frame.get("Event")?.get("event")
}

struct StreamSummary {
    frames: Vec<Value>,
    saw_registered: bool,
    exit: Option<i32>,
}

struct RetainedLifecycleHarness {
    _tempdir: TempDir,
    service: WorldService,
    binding: SharedWorldBindingSnapshot,
    count_path: PathBuf,
    orchestration_session_id: &'static str,
    participant_id: &'static str,
}

async fn collect_stream_summary<B>(body: &mut B, span_id: &str) -> StreamSummary
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    let mut buffer = Vec::new();
    let mut frames = Vec::new();
    let mut saw_registered = false;
    let mut exit = None;

    while let Some(frame) = next_optional_stream_frame_value(body, &mut buffer).await {
        frames.push(frame.clone());
        if let Some(event) = frame_event(&frame) {
            if event.get("kind").and_then(Value::as_str) == Some("registered") {
                saw_registered = true;
            }
            continue;
        }
        if let Some((code, exit_span_id)) = frame_exit(&frame) {
            assert_eq!(exit_span_id, span_id, "unexpected exit span: {frame:?}");
            exit = Some(code);
        }
    }

    StreamSummary {
        frames,
        saw_registered,
        exit,
    }
}

fn read_invocation_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .expect("read invocation count")
        .trim()
        .parse()
        .expect("parse invocation count")
}

type MemberRuntimeScriptWriter = fn(&Path) -> (PathBuf, PathBuf);

async fn launch_retained_lifecycle_harness(
    write_member_runtime: MemberRuntimeScriptWriter,
    orchestration_session_id: &'static str,
    participant_id: &'static str,
    run_id: &str,
) -> Option<RetainedLifecycleHarness> {
    let service = match WorldService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping retained member lifecycle test: service init failed: {err}");
            return None;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let seed_home = write_seed_home(tmp.path());
    let (member_binary, count_path) = write_member_runtime(tmp.path());
    let world_spec = WorldSpec {
        reuse_session: true,
        reuse_mode: WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: orchestration_session_id.to_string(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        }),
        isolate_network: false,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: Vec::new(),
        project_dir: tmp.path().to_path_buf(),
        always_isolate: true,
        fs_mode: substrate_common::WorldFsMode::Writable,
        backend_policy: None,
    };
    let world = match service.ensure_session_world(&world_spec) {
        Ok(world) => world,
        Err(err) => {
            eprintln!(
                "skipping retained member lifecycle test: failed to ensure shared world: {err}"
            );
            return None;
        }
    };
    let Some(binding) = world.shared_binding.clone() else {
        eprintln!("skipping retained member lifecycle test: shared world binding missing");
        return None;
    };

    let launch_response = service
        .execute_stream(make_member_dispatch_request(
            tmp.path(),
            &member_binary,
            &seed_home,
            &binding,
            orchestration_session_id,
            participant_id,
            run_id,
        ))
        .await
        .expect("member launch should succeed");
    let mut launch_body = launch_response.into_body();
    let mut launch_buffer = Vec::new();
    let launch_start = next_optional_stream_frame_value(&mut launch_body, &mut launch_buffer)
        .await
        .expect("expected launch start frame");
    let launch_span_id = frame_start_span_id(&launch_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {launch_start:?}"));
    let launch_summary = collect_stream_summary(&mut launch_body, &launch_span_id).await;
    assert!(
        launch_summary.saw_registered,
        "bootstrap with a surfaced session handle must register before it exits; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        launch_summary.exit,
        Some(0),
        "bootstrap must exit cleanly before later submit-turn proof begins; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        read_invocation_count(&count_path),
        1,
        "bootstrap exit proof must complete before the follow-up submit turn runs"
    );

    Some(RetainedLifecycleHarness {
        _tempdir: tmp,
        service,
        binding,
        count_path,
        orchestration_session_id,
        participant_id,
    })
}

async fn launch_clean_bootstrap_exit_harness() -> Option<RetainedLifecycleHarness> {
    launch_retained_lifecycle_harness(
        write_clean_bootstrap_then_resume_member_runtime,
        "orch-member-runtime-retained-clean-exit",
        "ash_member_runtime_retained_clean_exit",
        "run-member-runtime-retained-bootstrap",
    )
    .await
}

async fn launch_failed_submit_turn_harness() -> Option<RetainedLifecycleHarness> {
    launch_retained_lifecycle_harness(
        write_clean_bootstrap_fail_submit_turn_then_resume_member_runtime,
        "orch-member-runtime-retained-failed-turn",
        "ash_member_runtime_retained_failed_turn",
        "run-member-runtime-retained-failed-turn-bootstrap",
    )
    .await
}

async fn submit_turn_summary(
    harness: &RetainedLifecycleHarness,
    run_id: &str,
    prompt: &str,
) -> StreamSummary {
    let submit_response = harness
        .service
        .submit_member_turn_stream(make_member_turn_submit_request(
            harness.orchestration_session_id,
            harness.participant_id,
            &harness.binding.world_id,
            harness.binding.world_generation,
            run_id,
            prompt,
        ))
        .await
        .expect("submitted turn should succeed");
    let mut submit_body = submit_response.into_body();
    let mut submit_buffer = Vec::new();
    let submit_start = next_optional_stream_frame_value(&mut submit_body, &mut submit_buffer)
        .await
        .expect("expected submit_turn start frame");
    let submit_span_id = frame_start_span_id(&submit_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {submit_start:?}"));
    collect_stream_summary(&mut submit_body, &submit_span_id).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_clean_bootstrap_exit_with_session_handle_registers_then_leaves_parked_worker_ready_for_follow_up(
) {
    let Some(harness) = launch_clean_bootstrap_exit_harness().await else {
        return;
    };
    assert_eq!(
        read_invocation_count(&harness.count_path),
        1,
        "bootstrap harness must stop after clean exit and leave the parked retained worker ready for Packet 3 follow-up proofs"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_clean_bootstrap_exit_with_session_handle_requires_later_submit_turn_resumability(
) {
    let Some(harness) = launch_clean_bootstrap_exit_harness().await else {
        return;
    };

    let submit_summary = submit_turn_summary(
        &harness,
        "run-member-runtime-retained-follow-up",
        "follow-up prompt",
    )
    .await;
    assert_eq!(
        submit_summary.exit,
        Some(0),
        "parked retained follow-up submit_turn must exit cleanly; frames: {:?}",
        submit_summary.frames
    );
    assert!(
        submit_summary
            .frames
            .iter()
            .any(|frame| frame.to_string().contains("follow-up prompt success")),
        "parked retained follow-up submit_turn must surface the resumed follow-up output; frames: {:?}",
        submit_summary.frames
    );
    assert_eq!(
        read_invocation_count(&harness.count_path),
        2,
        "parked retained follow-up submit_turn must invoke the resumed member runtime after bootstrap exit"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_non_zero_submitted_turn_exit_cleans_active_turn_slot_without_deleting_retained_worker(
) {
    let Some(harness) = launch_failed_submit_turn_harness().await else {
        return;
    };

    let failed_turn_summary = submit_turn_summary(
        &harness,
        "run-member-runtime-retained-failed-follow-up",
        "follow-up prompt that fails",
    )
    .await;
    assert_eq!(
        failed_turn_summary.exit,
        Some(17),
        "non-zero submitted turn should surface its failure exit without implicitly closing retained continuity; frames: {:?}",
        failed_turn_summary.frames
    );
    assert_eq!(
        read_invocation_count(&harness.count_path),
        2,
        "the failed submitted turn should run exactly once before the recovery follow-up"
    );

    let resumed_turn_summary = submit_turn_summary(
        &harness,
        "run-member-runtime-retained-recovery-follow-up",
        "follow-up prompt after failure",
    )
    .await;
    assert_eq!(
        resumed_turn_summary.exit,
        Some(0),
        "a later submitted turn should still resume the retained worker after a non-zero turn exit; frames: {:?}",
        resumed_turn_summary.frames
    );
    assert!(
        resumed_turn_summary
            .frames
            .iter()
            .any(|frame| frame
                .to_string()
                .contains("follow-up prompt success after failure")),
        "retained follow-up after a failed turn must surface resumed output instead of acting like the worker was deleted; frames: {:?}",
        resumed_turn_summary.frames
    );
    assert_eq!(
        read_invocation_count(&harness.count_path),
        3,
        "non-zero submitted turn exit must clear active-turn bookkeeping without unregistering the retained worker"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_clean_bootstrap_exit_without_session_handle_fails_closed_for_submit_turn() {
    let service = match WorldService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!(
                "skipping retained member no-session-handle test: service init failed: {err}"
            );
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let seed_home = write_seed_home(tmp.path());
    let (member_binary, count_path) = write_member_runtime_without_session_handle(tmp.path());
    let orchestration_session_id = "orch-member-runtime-no-session-handle";
    let participant_id = "ash_member_runtime_no_session_handle";
    let world_spec = WorldSpec {
        reuse_session: true,
        reuse_mode: WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: orchestration_session_id.to_string(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        }),
        isolate_network: false,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: Vec::new(),
        project_dir: tmp.path().to_path_buf(),
        always_isolate: true,
        fs_mode: substrate_common::WorldFsMode::Writable,
        backend_policy: None,
    };
    let world = match service.ensure_session_world(&world_spec) {
        Ok(world) => world,
        Err(err) => {
            eprintln!(
                "skipping retained member no-session-handle test: failed to ensure shared world: {err}"
            );
            return;
        }
    };
    let Some(binding) = world.shared_binding.clone() else {
        eprintln!("skipping retained member no-session-handle test: shared world binding missing");
        return;
    };

    let launch_response = service
        .execute_stream(make_member_dispatch_request(
            tmp.path(),
            &member_binary,
            &seed_home,
            &binding,
            orchestration_session_id,
            participant_id,
            "run-member-runtime-no-session-handle-bootstrap",
        ))
        .await
        .expect("member launch should succeed");
    let mut launch_body = launch_response.into_body();
    let mut launch_buffer = Vec::new();
    let launch_start = next_optional_stream_frame_value(&mut launch_body, &mut launch_buffer)
        .await
        .expect("expected launch start frame");
    let launch_span_id = frame_start_span_id(&launch_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {launch_start:?}"));
    let launch_summary = collect_stream_summary(&mut launch_body, &launch_span_id).await;
    assert!(
        !launch_summary.saw_registered,
        "bootstrap without a surfaced session handle must not promise retained registration; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        launch_summary.exit,
        Some(0),
        "negative coverage must still exercise a clean bootstrap exit; frames: {:?}",
        launch_summary.frames
    );
    assert_eq!(
        read_invocation_count(&count_path),
        1,
        "inverse coverage must verify bootstrap exited before submit_turn is attempted"
    );

    let err = service
        .submit_member_turn_stream(make_member_turn_submit_request(
            orchestration_session_id,
            participant_id,
            &binding.world_id,
            binding.world_generation,
            "run-member-runtime-no-session-handle-follow-up",
            "follow-up prompt",
        ))
        .await
        .expect_err("missing session handle must fail closed for submit_turn");
    assert!(
        err.to_string().contains("is not retained"),
        "unexpected submit_turn error: {err}"
    );
    assert_eq!(
        read_invocation_count(&count_path),
        1,
        "fail-closed inverse must not run a resumptive follow-up invocation"
    );
}
