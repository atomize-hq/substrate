#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{
    ExecuteCancelRequestV1, ExecuteRequest, MemberDispatchRequestV1, MemberRuntimeBackendKindV1,
    PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1,
};
use hyper::body::HttpBody;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::timeout;
use world_agent::WorldAgentService;
use world_api::{SharedWorldOwnerAction, SharedWorldOwnerSpec, WorldReuseMode, WorldSpec};

const WORLD_PROJECT_DIR_OVERRIDE_ENV: &str = "SUBSTRATE_WORLD_PROJECT_DIR";

fn full_isolation_policy_snapshot() -> PolicySnapshotV3 {
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: false,
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
    world_id: &str,
    world_generation: u64,
    env: HashMap<String, String>,
) -> ExecuteRequest {
    ExecuteRequest {
        profile: None,
        cmd: String::new(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "member-runtime-world-placement-test".to_string(),
        budget: None,
        policy_snapshot: full_isolation_policy_snapshot(),
        shared_world: None,
        world_network: None,
        world_fs_mode: None,
        member_dispatch: Some(MemberDispatchRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch-member-runtime-world-placement".to_string(),
            participant_id: "ash_member_world_placement_test".to_string(),
            orchestrator_participant_id: "ash_orchestrator_world_placement_test".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "uaa.agent.session".to_string(),
            run_id: "run-member-world-placement-test".to_string(),
            world_id: world_id.to_string(),
            world_generation,
            initial_prompt: None,
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: binary_path.display().to_string(),
            },
        }),
    }
}

fn write_placement_member_runtime(temp: &Path) -> PathBuf {
    let path = temp.join("fake-member-runtime-placement.sh");
    let body = r#"#!/bin/sh
set -eu

proof_path="${PLACEMENT_PROOF_PATH:?}"
cgroup_procs="${EXPECTED_CGROUP_PROCS:?}"
attached=0
attempt=0
while [ "$attempt" -lt 100 ]; do
  if [ -f "$cgroup_procs" ] && grep -qx "$$" "$cgroup_procs"; then
    attached=1
    break
  fi
  attempt=$((attempt + 1))
  sleep 0.05
done

{
  printf 'cwd=%s\n' "$(pwd)"
  printf 'attached=%s\n' "$attached"
} > "$proof_path"

printf '{"type":"thread.started","thread_id":"thread-member-world-placement"}\r\n'
printf '{"type":"turn.started","thread_id":"thread-member-world-placement","turn_id":"turn-member-world-placement"}\r\n'
while :; do sleep 1; done
"#;
    fs::write(&path, body).expect("write fake member runtime");
    let mut perms = fs::metadata(&path)
        .expect("fake member runtime metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("set fake member runtime permissions");
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

fn is_event_frame(frame: &Value) -> bool {
    frame.get("type").and_then(Value::as_str) == Some("event") || frame.get("Event").is_some()
}

fn is_stream_chunk_frame(frame: &Value) -> bool {
    matches!(
        frame.get("type").and_then(Value::as_str),
        Some("stdout" | "stderr")
    ) || frame.get("Stdout").is_some()
        || frame.get("Stderr").is_some()
}

async fn wait_for_proof_file(path: &Path) -> String {
    timeout(Duration::from_secs(5), async {
        loop {
            match fs::read_to_string(path) {
                Ok(content) if !content.trim().is_empty() => return content,
                _ => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
    })
    .await
    .expect("timed out waiting for placement proof")
}

fn parse_proof_value<'a>(proof: &'a str, key: &str) -> Option<&'a str> {
    proof.lines().find_map(|line| {
        let (found_key, value) = line.split_once('=')?;
        (found_key == key).then_some(value)
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn member_runtime_launches_inside_authoritative_overlay_and_cgroup() {
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping member placement proof test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let nested = tmp.path().join("nested");
    fs::create_dir_all(&nested).expect("create nested cwd");
    let member_binary = write_placement_member_runtime(tmp.path());

    let world_spec = WorldSpec {
        reuse_session: true,
        reuse_mode: WorldReuseMode::SharedOrchestration(SharedWorldOwnerSpec {
            orchestration_session_id: "orch-member-runtime-world-placement".to_string(),
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
    let (world, overlay_root) = match service.ensure_session_overlay_root(&world_spec) {
        Ok(result) => result,
        Err(err) => {
            eprintln!(
                "skipping member placement proof test: failed to prepare overlay root: {err}"
            );
            return;
        }
    };
    let binding = world
        .shared_binding
        .clone()
        .expect("shared world binding should exist once authoritative world is prepared");
    let cgroup_procs = service
        .session_cgroup_path(&world)
        .expect("session cgroup path should resolve once authoritative world is prepared")
        .join("cgroup.procs");
    let proof_path = tmp.path().join("placement-proof.txt");

    let mut env = HashMap::new();
    env.insert(
        WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
        tmp.path().display().to_string(),
    );
    env.insert(
        "PLACEMENT_PROOF_PATH".to_string(),
        proof_path.display().to_string(),
    );
    env.insert(
        "EXPECTED_CGROUP_PROCS".to_string(),
        cgroup_procs.display().to_string(),
    );
    let request = make_member_dispatch_request(
        &nested,
        &member_binary,
        &binding.world_id,
        binding.world_generation,
        env,
    );

    let response = match service.execute_stream(request).await {
        Ok(response) => response,
        Err(err) => {
            panic!(
                "member placement proof test failed to establish streamed execute placement: {err}"
            );
        }
    };

    let mut body = response.into_body();
    let mut buffer = Vec::new();

    let start = next_stream_frame_value(&mut body, &mut buffer).await;
    let span_id = frame_start_span_id(&start)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| panic!("expected start frame, got {start:?}"));

    let proof = wait_for_proof_file(&proof_path).await;
    let expected_cwd = overlay_root.join("nested");
    let expected_cwd_display = expected_cwd.display().to_string();
    assert_ne!(
        expected_cwd, nested,
        "placement proof requires an overlay-root cwd distinct from the host cwd"
    );
    assert_eq!(
        parse_proof_value(&proof, "cwd"),
        Some(expected_cwd_display.as_str()),
        "member runtime should observe the authoritative overlay cwd"
    );
    assert_eq!(
        parse_proof_value(&proof, "attached"),
        Some("1"),
        "member runtime pid should be attached to the authoritative session cgroup"
    );

    let cancel = service
        .execute_cancel(ExecuteCancelRequestV1 {
            span_id: span_id.clone(),
            sig: "INT".to_string(),
        })
        .await
        .expect("member execute_cancel should succeed");
    assert!(cancel.delivered, "expected member cancel delivery");

    loop {
        let frame = next_stream_frame_value(&mut body, &mut buffer).await;
        if is_event_frame(&frame)
            || is_stream_chunk_frame(&frame)
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
            panic!("unexpected member streamed error: {message}");
        }
        panic!("unexpected member streamed frame: {frame:?}");
    }
}
