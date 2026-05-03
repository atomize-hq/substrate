#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{
    ExecuteCancelRequestV1, ExecuteRequest, ExecuteStreamFrame, MemberDispatchRequestV1,
    MemberRuntimeBackendKindV1, PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3,
    PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1,
};
use hyper::body::HttpBody;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::timeout;
use world_agent::WorldAgentService;
use world_api::{SharedWorldOwnerAction, SharedWorldOwnerSpec, WorldReuseMode, WorldSpec};

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
    let mut request = make_request(cwd, "");
    request.member_dispatch = Some(MemberDispatchRequestV1 {
        schema_version: 1,
        orchestration_session_id: "orch-streamed-member-cancel".to_string(),
        participant_id: "ash_member_cancel_test".to_string(),
        orchestrator_participant_id: "ash_orchestrator_cancel_test".to_string(),
        parent_participant_id: None,
        resumed_from_participant_id: None,
        backend_id: "cli:codex".to_string(),
        protocol: "uaa.agent.session".to_string(),
        run_id: "run-member-cancel-test".to_string(),
        world_id: world_id.to_string(),
        world_generation,
        resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
            backend_kind: MemberRuntimeBackendKindV1::Codex,
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

async fn next_stream_frame<B>(body: &mut B, buffer: &mut Vec<u8>) -> ExecuteStreamFrame
where
    B: HttpBody<Data = hyper::body::Bytes> + Unpin,
    B::Error: std::fmt::Debug + std::fmt::Display,
{
    loop {
        if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            let payload = &line[..line.len() - 1];
            return serde_json::from_slice(payload).expect("valid stream frame json");
        }

        let chunk = timeout(Duration::from_secs(5), body.data())
            .await
            .expect("timed out waiting for stream chunk")
            .expect("stream ended unexpectedly")
            .expect("stream chunk error");
        buffer.extend_from_slice(&chunk);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn execute_stream_cancel_interrupts_live_non_pty_command() {
    let service = match WorldAgentService::new() {
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

    let start = next_stream_frame(&mut body, &mut buffer).await;
    let span_id = match start {
        ExecuteStreamFrame::Start { span_id } => span_id,
        other => panic!("expected start frame, got {other:?}"),
    };

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
        match next_stream_frame(&mut body, &mut buffer).await {
            ExecuteStreamFrame::Stdout { .. }
            | ExecuteStreamFrame::Stderr { .. }
            | ExecuteStreamFrame::Event { .. } => continue,
            ExecuteStreamFrame::Exit {
                exit,
                span_id: exit_span,
                process_telemetry: _,
                ..
            } => {
                assert_eq!(exit_span, span_id);
                assert_eq!(exit, 130, "SIGINT exit should follow shell convention");
                break;
            }
            ExecuteStreamFrame::Error { message } => {
                panic!("unexpected streamed execute error: {message}");
            }
            ExecuteStreamFrame::Start { .. } => {
                panic!("unexpected duplicate start frame");
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn execute_stream_cancel_interrupts_live_member_runtime() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping member streamed cancel test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let member_binary = write_fake_member_runtime(tmp.path());
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
        always_isolate: false,
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
        &member_binary,
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

    let start = next_stream_frame(&mut body, &mut buffer).await;
    let span_id = match start {
        ExecuteStreamFrame::Start { span_id } => span_id,
        other => panic!("expected start frame, got {other:?}"),
    };

    let ready = next_stream_frame(&mut body, &mut buffer).await;
    match ready {
        ExecuteStreamFrame::Event { event } => {
            assert_eq!(
                event.kind,
                substrate_common::agent_events::AgentEventKind::Registered
            );
            assert_eq!(
                event.data.get("schema").and_then(serde_json::Value::as_str),
                Some("agent_api.session.handle.v1")
            );
            assert_eq!(
                event.participant_id.as_deref(),
                Some("ash_member_cancel_test")
            );
            assert_eq!(event.world_id.as_deref(), Some(binding.world_id.as_str()));
            assert_eq!(event.world_generation, Some(binding.world_generation));
            assert_eq!(event.span_id.as_deref(), Some(span_id.as_str()));
        }
        other => panic!("expected member registered event, got {other:?}"),
    }

    let cancel = service
        .execute_cancel(ExecuteCancelRequestV1 {
            span_id: span_id.clone(),
            sig: "INT".to_string(),
        })
        .await
        .expect("member execute_cancel should succeed");
    assert!(cancel.delivered, "expected member cancel delivery");

    loop {
        match next_stream_frame(&mut body, &mut buffer).await {
            ExecuteStreamFrame::Event { .. } => continue,
            ExecuteStreamFrame::Exit {
                exit,
                span_id: exit_span,
                ..
            } => {
                assert_eq!(exit_span, span_id);
                assert_eq!(
                    exit, 130,
                    "member runtime cancel should report SIGINT shell exit"
                );
                break;
            }
            ExecuteStreamFrame::Error { message } => {
                panic!("unexpected member streamed error: {message}");
            }
            ExecuteStreamFrame::Start { .. }
            | ExecuteStreamFrame::Stdout { .. }
            | ExecuteStreamFrame::Stderr { .. } => continue,
        }
    }
}
