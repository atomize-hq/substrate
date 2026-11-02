use super::super::test_support::{prelude::*, *};
use super::super::*;

#[tokio::test]
async fn app_flow_streams_notifications_and_response() {
    let (_dir, server) = start_fake_app_server().await;

    let thread_params = ThreadStartParams {
        thread_id: None,
        metadata: Value::Null,
    };
    let thread_handle = server
        .thread_start(thread_params)
        .await
        .expect("thread start");
    let thread_response = time::timeout(Duration::from_secs(2), thread_handle.response)
        .await
        .expect("thread response timeout")
        .expect("thread response recv")
        .expect("thread response ok");
    let thread_id = thread_response
        .get("thread_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    assert!(!thread_id.is_empty());

    let params = TurnStartParams {
        thread_id: thread_id.clone(),
        input: vec![TurnInput {
            kind: "text".to_string(),
            text: Some("hi".to_string()),
        }],
        model: None,
        config: BTreeMap::new(),
    };
    let mut handle = server.turn_start(params).await.expect("turn start");

    let first_event = time::timeout(Duration::from_secs(2), handle.events.recv())
        .await
        .expect("event timeout")
        .expect("event value");
    let turn_id = match first_event {
        AppNotification::Item {
            thread_id: tid,
            turn_id: Some(turn),
            item,
        } => {
            assert_eq!(tid, thread_id);
            assert!(item.get("message").is_some());
            turn
        }
        other => panic!("unexpected event: {other:?}"),
    };

    let second_event = time::timeout(Duration::from_secs(2), handle.events.recv())
        .await
        .expect("event timeout")
        .expect("event value");
    match second_event {
        AppNotification::TaskComplete {
            thread_id: tid,
            turn_id: event_turn,
            result,
        } => {
            assert_eq!(tid, thread_id);
            assert_eq!(event_turn.as_deref(), Some(turn_id.as_str()));
            assert_eq!(result, serde_json::json!({ "ok": true }));
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let response = time::timeout(Duration::from_secs(2), handle.response)
        .await
        .expect("response timeout")
        .expect("response recv");
    let response = response.expect("response ok");
    assert_eq!(
        response
            .get("turn_id")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        turn_id
    );

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn canceling_app_request_returns_cancelled_error() {
    let (_dir, server) = start_fake_app_server().await;

    let thread_params = ThreadStartParams {
        thread_id: None,
        metadata: Value::Null,
    };
    let thread_handle = server
        .thread_start(thread_params)
        .await
        .expect("thread start");
    let thread_response = time::timeout(Duration::from_secs(2), thread_handle.response)
        .await
        .expect("thread response timeout")
        .expect("thread response recv")
        .expect("thread response ok");
    let thread_id = thread_response
        .get("thread_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let params = TurnStartParams {
        thread_id: thread_id.clone(),
        input: vec![TurnInput {
            kind: "text".to_string(),
            text: Some("cancel me".to_string()),
        }],
        model: None,
        config: BTreeMap::new(),
    };

    let mut handle = server.turn_start(params).await.expect("turn start");
    server.cancel(handle.request_id).expect("send cancel");

    let cancel_event = time::timeout(Duration::from_secs(2), handle.events.recv())
        .await
        .expect("event timeout")
        .expect("cancel event");
    match cancel_event {
        AppNotification::TaskComplete {
            thread_id: tid,
            turn_id,
            result,
        } => {
            assert_eq!(tid, thread_id);
            assert!(turn_id.is_some());
            assert_eq!(result.get("cancelled"), Some(&Value::Bool(true)));
            assert_eq!(
                result.get("reason"),
                Some(&Value::String("client_cancel".into()))
            );
        }
        other => panic!("unexpected cancellation notification: {other:?}"),
    }

    let response = time::timeout(Duration::from_secs(2), handle.response)
        .await
        .expect("response timeout")
        .expect("recv");
    assert!(matches!(response, Err(McpError::Cancelled)));

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn thread_resume_allows_follow_up_turns() {
    let (_dir, server) = start_fake_app_server().await;

    let thread_params = ThreadStartParams {
        thread_id: None,
        metadata: Value::Null,
    };
    let thread_handle = server
        .thread_start(thread_params)
        .await
        .expect("thread start");
    let thread_response = time::timeout(Duration::from_secs(2), thread_handle.response)
        .await
        .expect("thread response timeout")
        .expect("recv")
        .expect("ok");
    let thread_id = thread_response
        .get("thread_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let resume_params = ThreadResumeParams {
        thread_id: thread_id.clone(),
    };
    let resume_handle = server
        .thread_resume(resume_params)
        .await
        .expect("thread resume");
    let resume_response = time::timeout(Duration::from_secs(2), resume_handle.response)
        .await
        .expect("resume response timeout")
        .expect("recv")
        .expect("ok");
    assert_eq!(
        resume_response
            .get("thread_id")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        thread_id
    );
    assert!(resume_response
        .get("resumed")
        .and_then(Value::as_bool)
        .unwrap_or(false));

    let params = TurnStartParams {
        thread_id: thread_id.clone(),
        input: vec![TurnInput {
            kind: "text".to_string(),
            text: Some("resume flow".to_string()),
        }],
        model: None,
        config: BTreeMap::new(),
    };
    let mut turn = server.turn_start(params).await.expect("turn start");

    let item = time::timeout(Duration::from_secs(2), turn.events.recv())
        .await
        .expect("event timeout")
        .expect("item event");
    let turn_id = match item {
        AppNotification::Item {
            thread_id: tid,
            turn_id: Some(turn_id),
            ..
        } => {
            assert_eq!(tid, thread_id);
            turn_id
        }
        other => panic!("unexpected event: {other:?}"),
    };

    let complete = time::timeout(Duration::from_secs(2), turn.events.recv())
        .await
        .expect("event timeout")
        .expect("completion event");
    match complete {
        AppNotification::TaskComplete {
            thread_id: tid,
            turn_id: event_turn,
            result,
        } => {
            assert_eq!(tid, thread_id);
            assert_eq!(event_turn.as_deref(), Some(turn_id.as_str()));
            assert_eq!(result, serde_json::json!({ "ok": true }));
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let turn_response = time::timeout(Duration::from_secs(2), turn.response)
        .await
        .expect("response timeout")
        .expect("recv")
        .expect("ok");
    assert_eq!(
        turn_response
            .get("turn_id")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        turn_id
    );

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn turn_interrupt_sends_cancel_notification() {
    let (_dir, server) = start_fake_app_server().await;

    let thread_params = ThreadStartParams {
        thread_id: None,
        metadata: Value::Null,
    };
    let thread_handle = server
        .thread_start(thread_params)
        .await
        .expect("thread start");
    let thread_response = time::timeout(Duration::from_secs(2), thread_handle.response)
        .await
        .expect("thread response timeout")
        .expect("recv")
        .expect("ok");
    let thread_id = thread_response
        .get("thread_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let params = TurnStartParams {
        thread_id: thread_id.clone(),
        input: vec![TurnInput {
            kind: "text".to_string(),
            text: Some("please interrupt".to_string()),
        }],
        model: None,
        config: BTreeMap::new(),
    };
    let mut turn = server.turn_start(params).await.expect("turn start");

    let first_event = time::timeout(Duration::from_secs(2), turn.events.recv())
        .await
        .expect("event timeout")
        .expect("event value");
    let turn_id = match first_event {
        AppNotification::Item {
            thread_id: tid,
            turn_id: Some(turn),
            ..
        } => {
            assert_eq!(tid, thread_id);
            turn
        }
        other => panic!("unexpected event: {other:?}"),
    };

    let interrupt = server
        .turn_interrupt(TurnInterruptParams {
            thread_id: Some(thread_id.clone()),
            turn_id: turn_id.clone(),
        })
        .await
        .expect("send interrupt");

    let cancel_event = time::timeout(Duration::from_secs(2), turn.events.recv())
        .await
        .expect("event timeout")
        .expect("cancel event");
    match cancel_event {
        AppNotification::TaskComplete {
            thread_id: tid,
            turn_id: event_turn,
            result,
        } => {
            assert_eq!(tid, thread_id);
            assert_eq!(event_turn.as_deref(), Some(turn_id.as_str()));
            assert_eq!(result.get("cancelled"), Some(&Value::Bool(true)));
            assert_eq!(
                result.get("reason"),
                Some(&Value::String("interrupted".into()))
            );
        }
        other => panic!("unexpected cancel notification: {other:?}"),
    }

    let turn_response = time::timeout(Duration::from_secs(2), turn.response)
        .await
        .expect("response timeout")
        .expect("recv");
    assert!(matches!(turn_response, Err(McpError::Cancelled)));

    let interrupt_response = time::timeout(Duration::from_secs(2), interrupt.response)
        .await
        .expect("interrupt response timeout")
        .expect("recv")
        .expect("ok");
    assert!(interrupt_response
        .get("interrupted")
        .and_then(Value::as_bool)
        .unwrap_or(false));

    let _ = server.shutdown().await;
}
