#![cfg(all(unix, target_os = "linux"))]

use hyper::body::HttpBody;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::timeout;
use transport_api_types::{
    ExecuteCancelRequestV1, ExecuteRequest, MemberDispatchRequestV1, MemberRuntimeBackendKindV1,
    PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1,
};
use world_api::{SharedWorldOwnerAction, SharedWorldOwnerSpec, WorldReuseMode, WorldSpec};
use world_service::WorldService;

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

fn make_request(cwd: &Path, cmd: &str) -> ExecuteRequest {
    let mut env = HashMap::new();
    env.insert(
        "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT".to_string(),
        "1".to_string(),
    );
    ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "streamed-execute-cancel-test".to_string(),
        budget: None,
        policy_snapshot: minimal_policy_snapshot(),
        shared_world: None,
        world_network: None,
        world_fs_mode: None,
        member_dispatch: None,
    }
}

fn make_member_dispatch_request(
    cwd: &Path,
    binary_path: &Path,
    world_id: &str,
    world_generation: u64,
) -> ExecuteRequest {
    make_member_dispatch_request_with_backend(
        cwd,
        binary_path,
        world_id,
        world_generation,
        "orch-streamed-member-cancel",
        "ash_member_cancel_test",
        "run-member-cancel-test",
        "cli:codex",
        MemberRuntimeBackendKindV1::Codex,
    )
}

#[allow(clippy::too_many_arguments)]
fn make_member_dispatch_request_with_backend(
    cwd: &Path,
    binary_path: &Path,
    world_id: &str,
    world_generation: u64,
    orchestration_session_id: &str,
    participant_id: &str,
    run_id: &str,
    backend_id: &str,
    backend_kind: MemberRuntimeBackendKindV1,
) -> ExecuteRequest {
    let mut request = make_request(cwd, "");
    request.member_dispatch = Some(MemberDispatchRequestV1 {
        schema_version: 1,
        orchestration_session_id: orchestration_session_id.to_string(),
        participant_id: participant_id.to_string(),
        orchestrator_participant_id: "ash_orchestrator_cancel_test".to_string(),
        parent_participant_id: None,
        resumed_from_participant_id: None,
        backend_id: backend_id.to_string(),
        protocol: "substrate.agent.session".to_string(),
        run_id: run_id.to_string(),
        world_id: world_id.to_string(),
        world_generation,
        initial_prompt: Some("first turn".to_string()),
        resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
            backend_kind,
            binary_path: binary_path.display().to_string(),
        },
    });
    request
}

fn write_fake_member_runtime(temp: &Path) -> std::path::PathBuf {
    let path = temp.join("fake-member-runtime.sh");
    let body = "#!/bin/sh\ntrap 'exit 0' INT TERM HUP QUIT\nprintf '{\"type\":\"thread.started\",\"thread_id\":\"thread-member-cancel\"}\\r\\n'\nprintf '{\"type\":\"turn.started\",\"thread_id\":\"thread-member-cancel\",\"turn_id\":\"turn-member-cancel\"}\\r\\n'\nwhile :; do sleep 1; done\n";
    fs::write(&path, body).expect("write fake member runtime");
    let mut perms = fs::metadata(&path)
        .expect("fake member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake member runtime permissions");
    path
}

fn write_fake_claude_member_runtime(temp: &Path) -> std::path::PathBuf {
    let path = temp.join("fake-member-runtime-claude.sh");
    let body = "#!/bin/sh\ntrap 'exit 0' INT TERM HUP QUIT\nprintf '{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"sess-member-cancel\"}\\n'\nwhile :; do sleep 1; done\n";
    fs::write(&path, body).expect("write fake claude member runtime");
    let mut perms = fs::metadata(&path)
        .expect("fake claude member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake claude member runtime permissions");
    path
}

async fn next_stream_frame_value<B>(body: &mut B, buffer: &mut Vec<u8>) -> Value
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    loop {
        if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            let payload = &line[..line.len() - 1];
            return serde_json::from_slice(payload).expect("valid stream frame json value");
        }

        let chunk = timeout(Duration::from_secs(5), body.data())
            .await
            .expect("timed out waiting for stream chunk")
            .expect("stream ended unexpectedly")
            .expect("stream chunk error");
        buffer.extend_from_slice(&chunk);
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

fn frame_error_message(frame: &Value) -> Option<&str> {
    if frame.get("type")?.as_str() == Some("error") {
        return frame.get("message")?.as_str();
    }
    frame.get("Error")?.get("message")?.as_str()
}

fn frame_event(frame: &Value) -> Option<&Value> {
    if frame.get("type").and_then(Value::as_str) == Some("event") {
        return frame.get("event");
    }
    frame.get("Event")?.get("event")
}

fn assert_registered_event(
    frame: &Value,
    expected_participant_id: &str,
    expected_backend_id: &str,
    expected_world_id: &str,
    expected_world_generation: u64,
    expected_span_id: &str,
) {
    let event = frame
        .get("event")
        .or_else(|| frame.pointer("/Event/event"))
        .unwrap_or_else(|| panic!("expected member registered event, got {frame:?}"));
    assert_eq!(
        event.get("kind").and_then(Value::as_str),
        Some("registered")
    );
    assert_eq!(
        event.pointer("/data/schema").and_then(Value::as_str),
        Some("agent_api.session.handle.v1")
    );
    assert_eq!(
        event.get("participant_id").and_then(Value::as_str),
        Some(expected_participant_id)
    );
    assert_eq!(
        event.get("backend_id").and_then(Value::as_str),
        Some(expected_backend_id)
    );
    assert_eq!(
        event.get("world_id").and_then(Value::as_str),
        Some(expected_world_id)
    );
    assert_eq!(
        event.get("world_generation").and_then(Value::as_u64),
        Some(expected_world_generation)
    );
    assert_eq!(
        event.get("span_id").and_then(Value::as_str),
        Some(expected_span_id)
    );
}

fn is_stream_chunk_frame(frame: &Value) -> bool {
    matches!(
        frame.get("type").and_then(Value::as_str),
        Some("stdout" | "stderr")
    ) || frame.get("Stdout").is_some()
        || frame.get("Stderr").is_some()
}

async fn next_registered_frame<B>(body: &mut B, buffer: &mut Vec<u8>, span_id: &str) -> Value
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    loop {
        let frame = next_stream_frame_value(body, buffer).await;
        if let Some(event) = frame_event(&frame) {
            if event.get("kind").and_then(Value::as_str) == Some("registered") {
                return frame;
            }
            continue;
        }
        if is_stream_chunk_frame(&frame) || frame_start_span_id(&frame).is_some() {
            continue;
        }
        if let Some((exit, exit_span)) = frame_exit(&frame) {
            panic!(
                "expected member registered event before exit, got exit={exit} span_id={exit_span} expected_span_id={span_id}"
            );
        }
        if let Some(message) = frame_error_message(&frame) {
            panic!("unexpected streamed execute error before registered event: {message}");
        }
        panic!("unexpected streamed execute frame before registered event: {frame:?}");
    }
}

async fn assert_stream_exit_for_span<B>(body: &mut B, buffer: &mut Vec<u8>, span_id: &str)
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    loop {
        let frame = next_stream_frame_value(body, buffer).await;
        if is_stream_chunk_frame(&frame)
            || frame_event(&frame).is_some()
            || frame_start_span_id(&frame).is_some()
        {
            continue;
        }
        if let Some((exit, exit_span)) = frame_exit(&frame) {
            assert_eq!(exit_span, span_id);
            assert_eq!(exit, 130, "SIGINT exit should follow shell convention");
            return;
        }
        if let Some(message) = frame_error_message(&frame) {
            panic!("unexpected streamed execute error: {message}");
        }
        panic!("unexpected streamed execute frame: {frame:?}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn execute_stream_cancel_interrupts_live_non_pty_command() {
    let service = match WorldService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping streamed execute cancel test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let request = make_request(tmp.path(), "sleep 10");
    let response = match service.execute_stream(request).await {
        Ok(response) => response,
        Err(err) => {
            eprintln!("skipping streamed execute cancel test: execute_stream failed: {err}");
            return;
        }
    };

    let mut body = response.into_body();
    let mut buffer = Vec::new();

    let start = next_stream_frame_value(&mut body, &mut buffer).await;
    let span_id = frame_start_span_id(&start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {start:?}"));

    tokio::time::sleep(Duration::from_millis(500)).await;

    let cancel = service
        .execute_cancel(ExecuteCancelRequestV1 {
            span_id: span_id.clone(),
            sig: "INT".to_string(),
        })
        .await
        .expect("execute_cancel should succeed");
    assert!(cancel.delivered, "expected cancel signal delivery");

    loop {
        let frame = next_stream_frame_value(&mut body, &mut buffer).await;
        if is_stream_chunk_frame(&frame)
            || frame_event(&frame).is_some()
            || frame_start_span_id(&frame).is_some()
        {
            continue;
        }
        if let Some((exit, exit_span)) = frame_exit(&frame) {
            assert_eq!(exit_span, span_id);
            assert_eq!(exit, 130, "SIGINT exit should follow shell convention");
            break;
        }
        if let Some(message) = frame_error_message(&frame) {
            panic!("unexpected streamed execute error: {message}");
        }
        panic!("unexpected streamed execute frame: {frame:?}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn execute_stream_cancel_interrupts_live_member_runtime() {
    let service = match WorldService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping member streamed cancel test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let codex_member_binary = write_fake_member_runtime(tmp.path());
    let world_spec = WorldSpec {
        reuse_session: true,
        reuse_mode: WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: "orch-streamed-member-cancel".to_string(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        }),
        isolate_network: false,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains: Vec::new(),
        project_dir: tmp.path().to_path_buf(),
        always_isolate: true,
        fs_mode: substrate_common::WorldFsMode::Writable,
    };
    let world = match service.ensure_session_world(&world_spec) {
        Ok(world) => world,
        Err(err) => {
            eprintln!("skipping member streamed cancel test: failed to ensure shared world: {err}");
            return;
        }
    };
    let Some(binding) = world.shared_binding.clone() else {
        eprintln!("skipping member streamed cancel test: shared world binding missing");
        return;
    };

    let request = make_member_dispatch_request(
        tmp.path(),
        &codex_member_binary,
        &binding.world_id,
        binding.world_generation,
    );
    let response = match service.execute_stream(request).await {
        Ok(response) => response,
        Err(err) => {
            eprintln!("skipping member streamed cancel test: execute_stream failed: {err}");
            return;
        }
    };

    let mut body = response.into_body();
    let mut buffer = Vec::new();

    let start = next_stream_frame_value(&mut body, &mut buffer).await;
    let span_id = frame_start_span_id(&start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {start:?}"));

    let ready = next_registered_frame(&mut body, &mut buffer, &span_id).await;
    assert_registered_event(
        &ready,
        "ash_member_cancel_test",
        "cli:codex",
        &binding.world_id,
        binding.world_generation,
        &span_id,
    );

    let cancel = service
        .execute_cancel(ExecuteCancelRequestV1 {
            span_id: span_id.clone(),
            sig: "INT".to_string(),
        })
        .await
        .expect("member execute_cancel should succeed");
    assert!(cancel.delivered, "expected member cancel delivery");

    assert_stream_exit_for_span(&mut body, &mut buffer, &span_id).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_backend_slots_allow_distinct_backends_and_reject_duplicates() {
    let service = match WorldService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping member backend slot test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let codex_member_binary = write_fake_member_runtime(tmp.path());
    let claude_member_binary = write_fake_claude_member_runtime(tmp.path());
    let orchestration_session_id = "orch-streamed-member-backend-slots";
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
    };
    let world = match service.ensure_session_world(&world_spec) {
        Ok(world) => world,
        Err(err) => {
            eprintln!("skipping member backend slot test: failed to ensure shared world: {err}");
            return;
        }
    };
    let Some(binding) = world.shared_binding.clone() else {
        eprintln!("skipping member backend slot test: shared world binding missing");
        return;
    };

    let codex_request = make_member_dispatch_request_with_backend(
        tmp.path(),
        &codex_member_binary,
        &binding.world_id,
        binding.world_generation,
        orchestration_session_id,
        "ash_member_codex_slot_test",
        "run-member-codex-slot-test",
        "cli:codex",
        MemberRuntimeBackendKindV1::Codex,
    );
    let codex_response = service
        .execute_stream(codex_request)
        .await
        .expect("codex member launch should succeed");
    let mut codex_body = codex_response.into_body();
    let mut codex_buffer = Vec::new();
    let codex_start = next_stream_frame_value(&mut codex_body, &mut codex_buffer).await;
    let codex_span_id = frame_start_span_id(&codex_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {codex_start:?}"));
    let codex_ready =
        next_registered_frame(&mut codex_body, &mut codex_buffer, &codex_span_id).await;
    assert_registered_event(
        &codex_ready,
        "ash_member_codex_slot_test",
        "cli:codex",
        &binding.world_id,
        binding.world_generation,
        &codex_span_id,
    );

    let claude_request = make_member_dispatch_request_with_backend(
        tmp.path(),
        &claude_member_binary,
        &binding.world_id,
        binding.world_generation,
        orchestration_session_id,
        "ash_member_claude_slot_test",
        "run-member-claude-slot-test",
        "cli:claude_code",
        MemberRuntimeBackendKindV1::ClaudeCode,
    );
    let claude_response = service
        .execute_stream(claude_request)
        .await
        .expect("claude_code member launch should succeed");
    let mut claude_body = claude_response.into_body();
    let mut claude_buffer = Vec::new();
    let claude_start = next_stream_frame_value(&mut claude_body, &mut claude_buffer).await;
    let claude_span_id = frame_start_span_id(&claude_start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {claude_start:?}"));
    let claude_ready =
        next_registered_frame(&mut claude_body, &mut claude_buffer, &claude_span_id).await;
    assert_registered_event(
        &claude_ready,
        "ash_member_claude_slot_test",
        "cli:claude_code",
        &binding.world_id,
        binding.world_generation,
        &claude_span_id,
    );

    let duplicate_request = make_member_dispatch_request_with_backend(
        tmp.path(),
        &codex_member_binary,
        &binding.world_id,
        binding.world_generation,
        orchestration_session_id,
        "ash_member_codex_duplicate_slot_test",
        "run-member-codex-duplicate-slot-test",
        "cli:codex",
        MemberRuntimeBackendKindV1::Codex,
    );
    let duplicate_err = service
        .execute_stream(duplicate_request)
        .await
        .expect_err("duplicate cli:codex slot should fail closed");
    let duplicate_message = duplicate_err.to_string();
    assert!(
        duplicate_message.contains("a retained world member is already active"),
        "unexpected duplicate error: {duplicate_message}"
    );
    assert!(
        duplicate_message.contains("backend_id cli:codex"),
        "unexpected duplicate error: {duplicate_message}"
    );

    let codex_cancel = service
        .execute_cancel(ExecuteCancelRequestV1 {
            span_id: codex_span_id.clone(),
            sig: "INT".to_string(),
        })
        .await
        .expect("codex execute_cancel should succeed");
    assert!(codex_cancel.delivered, "expected codex cancel delivery");

    let claude_cancel = service
        .execute_cancel(ExecuteCancelRequestV1 {
            span_id: claude_span_id.clone(),
            sig: "INT".to_string(),
        })
        .await
        .expect("claude_code execute_cancel should succeed");
    assert!(
        claude_cancel.delivered,
        "expected claude_code cancel delivery"
    );

    assert_stream_exit_for_span(&mut codex_body, &mut codex_buffer, &codex_span_id).await;
    assert_stream_exit_for_span(&mut claude_body, &mut claude_buffer, &claude_span_id).await;
}
