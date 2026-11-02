use super::*;

#[tokio::test]
async fn spawn_failures_surface_via_run_handle() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: true });
    let request = crate::AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let mut handle = run_harnessed_backend(adapter, BackendDefaults::default(), request)
        .await
        .expect("run should return a handle before startup resolves");

    let event = handle
        .events
        .next()
        .await
        .expect("spawn failure should surface as an error event");
    assert_eq!(event.kind, AgentWrapperEventKind::Error);
    assert_eq!(
        event.message.as_deref(),
        Some("toy backend error (redacted): phase=spawn")
    );

    assert!(
        handle.events.next().await.is_none(),
        "error event should be terminal for spawn failure"
    );

    let err = handle
        .completion
        .await
        .expect_err("completion should surface the spawn failure");
    assert!(matches!(
        err,
        crate::AgentWrapperError::Backend { ref message }
            if message == "toy backend error (redacted): phase=spawn"
    ));
}

#[tokio::test]
async fn spawn_failures_surface_via_control_handle_after_return() {
    let adapter = std::sync::Arc::new(ToyAdapter { fail_spawn: true });
    let request = crate::AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };

    let crate::AgentWrapperRunControl {
        mut handle,
        cancel: _,
    } = run_harnessed_backend_control(adapter, BackendDefaults::default(), request, None)
        .await
        .expect("control entrypoint should return before startup resolves");

    let event = handle
        .events
        .next()
        .await
        .expect("spawn failure should surface as an error event");
    assert_eq!(event.kind, AgentWrapperEventKind::Error);
    assert_eq!(
        event.message.as_deref(),
        Some("toy backend error (redacted): phase=spawn")
    );

    assert!(
        handle.events.next().await.is_none(),
        "error event should be terminal for spawn failure"
    );

    let err = handle
        .completion
        .await
        .expect_err("completion should be backend error");
    assert!(matches!(
        err,
        crate::AgentWrapperError::Backend { ref message }
        if message == "toy backend error (redacted): phase=spawn"
    ));
}
