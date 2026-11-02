use super::super::test_support::{prelude::*, *};
use super::super::*;

#[tokio::test]
async fn codex_flow_streams_events_and_response() {
    let (_dir, server) = start_fake_mcp_server().await;

    let params = CodexCallParams {
        prompt: "hello".into(),
        model: None,
        cwd: None,
        sandbox: None,
        approval_policy: None,
        profile: None,
        config: BTreeMap::new(),
    };

    let mut handle = server.codex(params).await.expect("codex call");

    let first_event = time::timeout(Duration::from_secs(2), handle.events.recv())
        .await
        .expect("event timeout")
        .expect("event value");
    match first_event {
        CodexEvent::ApprovalRequired(req) => {
            assert!(req.approval_id.starts_with("ap-"));
            assert_eq!(req.kind, ApprovalKind::Exec);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let second_event = time::timeout(Duration::from_secs(2), handle.events.recv())
        .await
        .expect("event timeout")
        .expect("event value");
    let event_conversation = match second_event {
        CodexEvent::TaskComplete {
            conversation_id, ..
        } => {
            assert!(!conversation_id.is_empty());
            conversation_id
        }
        other => panic!("unexpected event: {other:?}"),
    };

    let response = time::timeout(Duration::from_secs(2), handle.response)
        .await
        .expect("response timeout")
        .expect("response recv");
    let response = response.expect("response ok");
    assert_eq!(
        response.conversation_id.as_deref(),
        Some(event_conversation.as_str())
    );
    assert_eq!(response.output, serde_json::json!({ "ok": true }));

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn canceling_request_returns_cancelled_error() {
    let (_dir, server) = start_fake_mcp_server().await;

    let params = CodexCallParams {
        prompt: "cancel me".into(),
        model: None,
        cwd: None,
        sandbox: None,
        approval_policy: None,
        profile: None,
        config: BTreeMap::new(),
    };

    let mut handle = server.codex(params).await.expect("codex call");
    server.cancel(handle.request_id).expect("cancel send");

    let expected_conversation = format!("conv-{}", handle.request_id);
    let cancel_event = time::timeout(Duration::from_secs(2), handle.events.recv())
        .await
        .expect("event timeout")
        .expect("cancel notification");
    match cancel_event {
        CodexEvent::Cancelled {
            conversation_id,
            reason,
        } => {
            assert_eq!(
                conversation_id.as_deref(),
                Some(expected_conversation.as_str())
            );
            assert_eq!(reason.as_deref(), Some("client_cancel"));
        }
        other => panic!("expected cancellation event, got {other:?}"),
    }

    let response = time::timeout(Duration::from_secs(2), handle.response)
        .await
        .expect("response timeout")
        .expect("recv");
    assert!(matches!(response, Err(McpError::Cancelled)));

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn codex_reply_streams_follow_up_notifications() {
    let (_dir, server) = start_fake_mcp_server().await;

    let params = CodexCallParams {
        prompt: "hello".into(),
        model: None,
        cwd: None,
        sandbox: None,
        approval_policy: None,
        profile: None,
        config: BTreeMap::new(),
    };
    let first = server.codex(params).await.expect("start codex");
    let first_response = time::timeout(Duration::from_secs(2), first.response)
        .await
        .expect("response timeout")
        .expect("recv")
        .expect("ok");
    let conversation_id = first_response.conversation_id.expect("conversation id set");
    assert!(!conversation_id.is_empty());

    let reply_params = CodexReplyParams {
        conversation_id: conversation_id.clone(),
        prompt: "follow up".into(),
    };
    let mut reply = server.codex_reply(reply_params).await.expect("codex reply");

    let expected_approval = format!("ap-{}", reply.request_id);
    let approval = time::timeout(Duration::from_secs(2), reply.events.recv())
        .await
        .expect("event timeout")
        .expect("approval");
    match approval {
        CodexEvent::ApprovalRequired(req) => {
            assert_eq!(req.approval_id, expected_approval);
            assert_eq!(req.kind, ApprovalKind::Exec);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let complete = time::timeout(Duration::from_secs(2), reply.events.recv())
        .await
        .expect("event timeout")
        .expect("task completion");
    match complete {
        CodexEvent::TaskComplete {
            conversation_id: event_conv,
            ..
        } => assert_eq!(event_conv, conversation_id),
        other => panic!("unexpected event: {other:?}"),
    }

    let reply_response = time::timeout(Duration::from_secs(2), reply.response)
        .await
        .expect("response timeout")
        .expect("recv")
        .expect("ok");
    assert_eq!(
        reply_response.conversation_id.as_deref(),
        Some(conversation_id.as_str())
    );
    assert_eq!(reply_response.output, serde_json::json!({ "ok": true }));

    let _ = server.shutdown().await;
}
