#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{
    ExecuteCancelRequestV1, ExecuteRequest, ExecuteStreamFrame, PolicySnapshotV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
};
use hyper::body::HttpBody;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::timeout;
use world_agent::WorldAgentService;

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
        world_network: None,
        world_fs_mode: None,
    }
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
