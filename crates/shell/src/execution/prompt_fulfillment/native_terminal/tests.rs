//! Glue barriers; actual SDK Child/pipe acknowledgment is a separate integration obligation.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

struct Resource(Arc<AtomicBool>);
impl Drop for Resource {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
fn cancelled() -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: "cancelled".into(),
    }
}

#[tokio::test]
async fn test_raw_cancelled_stays_exact_while_settlement_fails() {
    let (release, held) = tokio::sync::oneshot::channel();
    let (raw, terminal) = retain_native_completion(
        Box::pin(async { Err(cancelled()) }),
        Some(Box::pin(async move {
            held.await.unwrap();
            Err(backend_error("pipe acknowledgment failed"))
        })),
        (),
        || {},
        None,
    );
    let mut raw = raw;
    assert!(tokio::time::timeout(Duration::from_millis(10), &mut raw)
        .await
        .is_err());
    release.send(()).unwrap();
    assert!(
        matches!(raw.await, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
    let outcome = terminal.wait().await;
    assert!(
        matches!(&outcome.raw, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
    );
    assert!(outcome.require_checked().is_err());
    assert!(
        matches!(outcome.enclosing_result(outcome.raw.as_ref().cloned().map_err(clone_wrapper_error)),
        Err(AgentWrapperError::Backend { message }) if message.contains("submission_outcome_unknown") && message.contains("cancelled"))
    );
}

#[tokio::test]
async fn test_terminal_waits_for_settlement_and_retains_resources() {
    let dropped = Arc::new(AtomicBool::new(false));
    let (release, held) = tokio::sync::oneshot::channel();
    let (raw, terminal) = retain_native_completion(
        Box::pin(async { Err(backend_error("provider error")) }),
        Some(Box::pin(async move {
            held.await.unwrap();
            Ok(())
        })),
        Resource(Arc::clone(&dropped)),
        || {},
        None,
    );
    let mut observation = Box::pin(terminal.wait());
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut observation)
            .await
            .is_err()
    );
    assert!(!dropped.load(Ordering::SeqCst));
    release.send(()).unwrap();
    let outcome = observation.await;
    assert!(outcome.require_checked().is_ok());
    assert!(dropped.load(Ordering::SeqCst));
    assert!(
        matches!(raw.await, Err(AgentWrapperError::Backend { message }) if message == "provider error")
    );
}

#[tokio::test]
async fn test_dropped_completion_requests_cancel_and_keeps_owner() {
    let dropped = Arc::new(AtomicBool::new(false));
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let (release, held) = tokio::sync::oneshot::channel();
    let (raw, terminal) = retain_native_completion(
        Box::pin(async move {
            cancelled.await.unwrap();
            Err(backend_error("cancelled"))
        }),
        Some(Box::pin(async move {
            held.await.unwrap();
            Ok(())
        })),
        Resource(Arc::clone(&dropped)),
        move || {
            let _ = cancel.send(());
        },
        None,
    );
    drop(raw);
    let mut observation = Box::pin(terminal.wait());
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut observation)
            .await
            .is_err()
    );
    assert!(!dropped.load(Ordering::SeqCst));
    release.send(()).unwrap();
    assert!(observation.await.require_checked().is_ok());
    assert!(dropped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_unsupported_is_not_checked_success() {
    let (raw, terminal) =
        retain_native_completion(Box::pin(async { Err(cancelled()) }), None, (), || {}, None);
    let raw = raw.await;
    let outcome = terminal.wait().await;
    assert!(matches!(
        &outcome.settlement,
        NativeSettlementOutcome::Unsupported
    ));
    assert!(outcome.require_checked().is_err());
    // Existing unsupported Fork completion remains functional and is not fabricated as checked.
    assert!(matches!(outcome.enclosing_result(raw),
        Err(AgentWrapperError::Backend { message }) if message == "cancelled"));
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        let (raw, terminal) = retain_native_completion(
            Box::pin(async {
                Ok(AgentWrapperCompletion {
                    status: std::process::ExitStatus::from_raw(0),
                    final_text: Some("unchanged initial Fork result".into()),
                    data: Some(serde_json::json!({"session":{"id":"fork-session"}})),
                })
            }),
            None,
            (),
            || {},
            None,
        );
        let raw = raw.await;
        let observed = terminal.wait().await;
        assert!(observed.require_checked().is_err());
        let unqualified = observed.enclosing_result(raw).unwrap();
        assert!(unqualified.status.success());
        assert_eq!(
            unqualified.final_text.as_deref(),
            Some("unchanged initial Fork result")
        );
        assert_eq!(
            unqualified.data,
            Some(serde_json::json!({"session":{"id":"fork-session"}}))
        );
    }
}

#[tokio::test]
async fn test_publisher_abandonment_cannot_succeed() {
    let (sender, receiver) = tokio::sync::watch::channel(None);
    drop(sender);
    let outcome = NativeTerminalObservation { receiver }.wait().await;
    assert!(outcome.require_checked().is_err());
    assert!(outcome.raw.is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn test_observing_attempt_does_not_seal_live_startup() {
    use std::{collections::BTreeMap, os::unix::fs::PermissionsExt};
    let root = tempfile::tempdir().unwrap();
    let binary = root.path().join("after-gate.py");
    std::fs::write(
        &binary,
        r#"#!/usr/bin/python3
import sys
sys.stdin.read()
print('{"type":"thread.started","thread_id":"after-gate"}')
print('{"type":"turn.completed"}')
"#,
    )
    .unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let client = codex::CodexClient::builder()
        .binary(binary)
        .json(true)
        .mirror_stdout(false)
        .build();
    let attempt = NativeAttemptCustody::codex();
    let NativeAttemptCustody::Codex { process, .. } = &attempt else {
        unreachable!()
    };
    let process = process.clone();
    let (entered, started) = tokio::sync::oneshot::channel();
    let (continue_startup, gate) = tokio::sync::oneshot::channel();
    let inner_attempt = attempt.clone();
    let startup = tokio::spawn(async move {
        retain_native_startup(
            async move {
                entered.send(()).unwrap();
                gate.await.unwrap();
                client
                    .stream_exec_with_env_overrides_custody(
                        codex::ExecStreamRequest {
                            prompt: "after gate".into(),
                            ephemeral: false,
                            ignore_rules: false,
                            ignore_user_config: false,
                            idle_timeout: None,
                            output_last_message: None,
                            output_schema: None,
                            json_event_log: None,
                        },
                        &BTreeMap::new(),
                        &process,
                    )
                    .await
                    .map_err(|error| backend_error(error.to_string()))
            },
            inner_attempt,
            (),
        )
        .await
    });
    started.await.unwrap();
    let mut observation = attempt.settlement();
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut observation)
            .await
            .is_err(),
        "a normal observer must wait on the startup owner rather than seal empty process custody"
    );
    continue_startup.send(()).unwrap();
    let (control, resources) = startup.await.unwrap().unwrap();
    drop(control.events);
    let completion: DynAgentWrapperCompletion = Box::pin(async move {
        control
            .completion
            .await
            .map(|completion| AgentWrapperCompletion {
                status: completion.status,
                final_text: completion.last_message,
                data: None,
            })
            .map_err(|error| backend_error(error.to_string()))
    });
    let (completion, terminal) =
        resources.retain_completion(completion, Some(attempt.settlement()), || {}, None);
    assert!(completion.await.unwrap().status.success());
    assert!(terminal.wait().await.require_checked().is_ok());
    assert!(observation.await.is_ok());
}

#[tokio::test]
async fn test_startup_owner_abandonment_cannot_succeed() {
    let attempt = NativeAttemptCustody::codex();
    let owner = attempt.begin_startup().unwrap();
    drop(owner);
    assert!(attempt.settlement().await.is_err());
}

#[cfg(unix)]
#[test]
fn test_provider_terminal_fact_waits_for_finality_and_checked_success() {
    use std::os::unix::process::ExitStatusExt;
    for (provider, phase) in [
        ("codex", "completed"),
        ("codex", "failed"),
        ("claude_code", "completed"),
        ("claude_code", "failed"),
    ] {
        let event = agent_api::AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new(provider).unwrap(),
            kind: agent_api::AgentWrapperEventKind::Status,
            channel: None,
            text: None,
            message: Some("unaltered provider payload".into()),
            data: Some(serde_json::json!({"turn":{"phase":phase},"provider":provider})),
        };
        let live = event.clone();
        let mut pending = PendingProviderTerminal::default();
        pending.push(event, phase == "completed").unwrap();
        let native = NativeTerminalOutcome {
            raw: Ok(AgentWrapperCompletion {
                status: std::process::ExitStatus::from_raw(if phase == "completed" {
                    0
                } else {
                    7 << 8
                }),
                final_text: None,
                data: None,
            }),
            settlement: NativeSettlementOutcome::Checked(Ok(())),
        };
        assert!(pending.take_after_finality(false, &native).is_err());
        assert_eq!(
            pending.take_after_finality(true, &native).unwrap(),
            Some(live)
        );
        assert!(pending
            .take_after_finality(true, &native)
            .unwrap()
            .is_none());
    }
}

#[cfg(unix)]
#[test]
fn test_failed_or_missing_ack_does_not_consume_provider_terminal() {
    use std::os::unix::process::ExitStatusExt;
    let event = agent_api::AgentWrapperEvent {
        agent_kind: agent_api::AgentWrapperKind::new("codex").unwrap(),
        kind: agent_api::AgentWrapperEventKind::Status,
        channel: None,
        text: None,
        message: Some("provider completed".into()),
        data: None,
    };
    let mut pending = PendingProviderTerminal::default();
    pending.push(event.clone(), true).unwrap();
    let mut native = NativeTerminalOutcome {
        raw: Ok(AgentWrapperCompletion {
            status: std::process::ExitStatus::from_raw(0),
            final_text: None,
            data: None,
        }),
        settlement: NativeSettlementOutcome::Unsupported,
    };
    assert!(pending.take_after_finality(true, &native).is_err());
    native.settlement =
        NativeSettlementOutcome::Checked(Err(backend_error("wait acknowledgment failed")));
    assert!(pending.take_after_finality(true, &native).is_err());
    native.settlement = NativeSettlementOutcome::Checked(Ok(()));
    native.raw = Err(cancelled());
    assert!(pending.take_after_finality(true, &native).is_err());
    native.raw = Ok(AgentWrapperCompletion {
        status: std::process::ExitStatus::from_raw(7 << 8),
        final_text: None,
        data: None,
    });
    assert!(pending.take_after_finality(true, &native).is_err());
    native.raw = Ok(AgentWrapperCompletion {
        status: std::process::ExitStatus::from_raw(0),
        final_text: None,
        data: None,
    });
    assert_eq!(
        pending.take_after_finality(true, &native).unwrap(),
        Some(event)
    );
}

#[cfg(unix)]
#[test]
fn test_terminal_fact_is_bounded_and_stream_loss_is_not_success() {
    let mut pending = PendingProviderTerminal::default();
    let native = NativeTerminalOutcome {
        raw: Err(backend_error("stream lost")),
        settlement: NativeSettlementOutcome::Checked(Ok(())),
    };
    assert!(pending.take_after_finality(true, &native).is_err());
    let first = agent_api::AgentWrapperEvent {
        agent_kind: agent_api::AgentWrapperKind::new("claude_code").unwrap(),
        kind: agent_api::AgentWrapperEventKind::Error,
        channel: None,
        text: None,
        message: Some("exact provider error".into()),
        data: None,
    };
    pending.push(first.clone(), false).unwrap();
    assert!(pending.push(first, false).is_err());
    assert!(pending.take_after_finality(true, &native).is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn test_startup_drop_retains_resources_until_custody_settles() {
    use super::super::PromptFulfillmentBridge;
    use crate::execution::agent_runtime::{
        mapping::AgentRuntimeBackendKind, validator::RuntimeSelectionDescriptor,
    };
    use crate::execution::config_model::AgentExecutionScope;
    use std::{collections::BTreeMap, os::unix::fs::PermissionsExt};
    for mode in [
        "codex_start",
        "codex_resume",
        "claude_start",
        "claude_resume",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let marker = temp.path().join("admitted");
        let binary = temp.path().join("native.py");
        std::fs::write(
            &binary,
            r#"#!/usr/bin/python3
import os, pathlib, time
pathlib.Path(os.environ["CUSTODY_MARKER"]).write_text("real-child")
time.sleep(30)
"#,
        )
        .unwrap();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let codex = mode.starts_with("codex");
        let bridge = Arc::new(
            PromptFulfillmentBridge::for_descriptor(&RuntimeSelectionDescriptor {
                agent_id: "custody-test".into(),
                backend_id: if codex {
                    "cli:codex-test"
                } else {
                    "cli:claude-test"
                }
                .into(),
                backend_kind: if codex {
                    AgentRuntimeBackendKind::Codex
                } else {
                    AgentRuntimeBackendKind::ClaudeCode
                },
                protocol: crate::execution::agent_runtime::mapping::PURE_AGENT_PROTOCOL.into(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: binary,
            })
            .unwrap(),
        );
        let attempt = bridge.native_attempt();
        let inner_attempt = attempt.clone();
        let resume = mode.ends_with("resume");
        let task_marker = marker.clone();
        let task_root = Arc::new(temp);
        let _held_root = Arc::clone(&task_root);
        let task = tokio::spawn(async move {
            let request = agent_api::AgentWrapperRunRequest {
                prompt: if codex {
                    "x".repeat(2 * 1024 * 1024)
                } else {
                    "argv prompt".into()
                },
                working_dir: Some(task_root.path().to_path_buf()),
                timeout: None,
                env: BTreeMap::from([("CUSTODY_MARKER".into(), task_marker.display().to_string())]),
                extensions: if resume {
                    BTreeMap::from([(
                        "agent_api.session.resume.v1".into(),
                        serde_json::json!({"selector":"id","id":"exact-session"}),
                    )])
                } else {
                    BTreeMap::new()
                },
            };
            let control = if resume {
                bridge
                    .run_authority_managed_resume_one_turn_control(
                        "exact-session",
                        request,
                        &inner_attempt,
                    )
                    .await
            } else {
                bridge
                    .run_durable_start_control(request, &inner_attempt)
                    .await
            };
            // Keep the fixture root until the public control/startup observation ends.
            (control, task_root)
        });
        let ready = tokio::time::timeout(Duration::from_secs(5), async {
            while !marker.exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
        if ready.is_err() {
            task.abort();
            let _ = task.await;
            attempt.request_termination();
            let _ = tokio::time::timeout(Duration::from_secs(5), attempt.settlement()).await;
            panic!("actual native fixture did not reach startup admission");
        }
        if codex {
            // The fixture never reads the multi-MiB prompt. The direct API is still
            // in genuine startup when its caller is disposed.
            assert!(
                !task.is_finished(),
                "blocked Codex stdin must keep startup pending"
            );
            task.abort();
            let _ = task.await;
            attempt.request_termination();
            assert!(
                tokio::time::timeout(Duration::from_secs(5), attempt.settlement())
                    .await
                    .unwrap()
                    .is_ok()
            );
        } else {
            let (control, _root) = task.await.unwrap();
            let control = control.unwrap();
            let terminal = control.terminal.clone();
            control.cancel.cancel();
            drop(control.handle);
            let observed = tokio::time::timeout(Duration::from_secs(5), terminal.wait())
                .await
                .unwrap();
            assert!(observed.require_checked().is_ok());
        }
    }
}

#[cfg(unix)]
#[tokio::test]
async fn test_native_start_resume_startup_failure_is_joined() {
    use super::super::PromptFulfillmentBridge;
    use crate::execution::agent_runtime::{
        mapping::AgentRuntimeBackendKind, validator::RuntimeSelectionDescriptor,
    };
    use crate::execution::config_model::AgentExecutionScope;
    use std::collections::BTreeMap;
    for (codex, resume) in [(true, false), (true, true), (false, false), (false, true)] {
        let temp = tempfile::tempdir().unwrap();
        let bridge = PromptFulfillmentBridge::for_descriptor(&RuntimeSelectionDescriptor {
            agent_id: "startup-failure".into(),
            backend_id: if codex {
                "cli:codex-test"
            } else {
                "cli:claude-test"
            }
            .into(),
            backend_kind: if codex {
                AgentRuntimeBackendKind::Codex
            } else {
                AgentRuntimeBackendKind::ClaudeCode
            },
            protocol: crate::execution::agent_runtime::mapping::PURE_AGENT_PROTOCOL.into(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: temp.path().join("missing-native-binary"),
        })
        .unwrap();
        let attempt = bridge.native_attempt();
        let request = agent_api::AgentWrapperRunRequest {
            prompt: "startup failure fixture".into(),
            working_dir: Some(temp.path().into()),
            timeout: None,
            env: BTreeMap::new(),
            extensions: if resume {
                BTreeMap::from([(
                    "agent_api.session.resume.v1".into(),
                    serde_json::json!({"selector":"id","id":"exact-session"}),
                )])
            } else {
                BTreeMap::new()
            },
        };
        let result = if resume {
            bridge
                .run_authority_managed_resume_one_turn_control("exact-session", request, &attempt)
                .await
        } else {
            bridge.run_durable_start_control(request, &attempt).await
        };
        let error = match result {
            Ok(_) => panic!("missing executable unexpectedly started"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains(if resume { "Resume" } else { "Start" }),
            "{error}"
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(5), attempt.settlement())
                .await
                .unwrap()
                .is_ok()
        );
    }
}

#[test]
fn test_checked_failed_provider_fact_preserves_backend_error() {
    for provider in ["codex", "claude_code"] {
        let event = agent_api::AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new(provider).unwrap(),
            kind: agent_api::AgentWrapperEventKind::Error,
            channel: None,
            text: None,
            message: Some("exact provider failure payload".into()),
            data: Some(serde_json::json!({"provider":provider,"turn":{"phase":"failed"}})),
        };
        let mut pending = PendingProviderTerminal::default();
        pending.push(event.clone(), false).unwrap();
        let native = NativeTerminalOutcome {
            raw: Err(backend_error("exact raw backend failure")),
            settlement: NativeSettlementOutcome::Checked(Ok(())),
        };
        assert!(pending.take_after_finality(false, &native).is_err());
        assert_eq!(
            pending.take_after_finality(true, &native).unwrap(),
            Some(event)
        );
        assert!(
            matches!(&native.raw, Err(AgentWrapperError::Backend { message }) if message == "exact raw backend failure")
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn test_failed_native_ack_still_joins_real_toolbox_disposal() {
    // Component ordering proof using a real listener/task, not HSA admission or
    // a fabricated native Child acknowledgment. Production supplies actual close().
    for mode in ["native_failed", "unsupported", "checked"] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("toolbox.sock");
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let connected = tokio::net::UnixStream::connect(&path).await.unwrap();
        drop(connected);
        let authoritative = Arc::new(AtomicBool::new(true));
        let joined = Arc::new(AtomicBool::new(false));
        let (shutdown, stopped) = tokio::sync::oneshot::channel();
        let (shutdown_seen, observed_shutdown) = tokio::sync::oneshot::channel();
        let (release, teardown_allowed) = tokio::sync::oneshot::channel();
        let path_for_task = path.clone();
        let task = tokio::spawn(async move {
            stopped.await.unwrap();
            shutdown_seen.send(()).unwrap();
            // A separately owned cleanup task has not acknowledged teardown yet.
            teardown_allowed.await.unwrap();
            drop(listener);
            tokio::fs::remove_file(path_for_task).await.unwrap();
        });
        let native = NativeTerminalOutcome {
            raw: Err(cancelled()),
            settlement: match mode {
                "native_failed" => NativeSettlementOutcome::Checked(Err(backend_error(
                    "exact native acknowledgment failure",
                ))),
                "unsupported" => NativeSettlementOutcome::Unsupported,
                _ => NativeSettlementOutcome::Checked(Ok(())),
            },
        };
        let revoke = Arc::clone(&authoritative);
        let task_joined = Arc::clone(&joined);
        let (release_native, native_pending) = tokio::sync::oneshot::channel();
        let (cleanup_joined, cleanup_join_observed) = tokio::sync::oneshot::channel();
        let mut finishing = Box::pin(finish_after_native_ack(
            async {
                native_pending.await.unwrap();
                native.require_checked()
            },
            async move {
                revoke.store(false, Ordering::SeqCst);
                shutdown.send(()).unwrap();
                task.await.unwrap();
                task_joined.store(true, Ordering::SeqCst);
                cleanup_joined.send(()).unwrap();
            },
        ));
        tokio::time::timeout(Duration::from_secs(5), async {
            tokio::select! {
                result = observed_shutdown => result.unwrap(),
                result = &mut finishing => panic!("native wait must stay pending until explicit release: {result:?}"),
            }
        }).await.unwrap();
        assert!(!authoritative.load(Ordering::SeqCst));
        assert!(!joined.load(Ordering::SeqCst));
        assert!(path.exists(), "the held listener teardown is still pending");
        release.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            tokio::select! {
                result = cleanup_join_observed => result.unwrap(),
                result = &mut finishing => panic!("native wait must stay pending after actual route teardown: {result:?}"),
            }
        }).await.unwrap();
        assert!(joined.load(Ordering::SeqCst));
        assert!(!path.exists());
        assert!(tokio::net::UnixStream::connect(&path).await.is_err());
        release_native.send(()).unwrap();
        let result = tokio::time::timeout(Duration::from_secs(5), finishing)
            .await
            .unwrap();
        match mode {
            "native_failed" => assert!(
                matches!(result, Err(AgentWrapperError::Backend { message }) if message == "exact native acknowledgment failure")
            ),
            "unsupported" => assert!(
                matches!(result, Err(AgentWrapperError::Backend { message }) if message == "checked native settlement unavailable")
            ),
            _ => assert!(result.is_ok()),
        }
        assert!(joined.load(Ordering::SeqCst));
        assert!(!path.exists());
        assert!(tokio::net::UnixStream::connect(&path).await.is_err());
        assert!(
            matches!(&native.raw, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
        );
    }
}

#[cfg(unix)]
#[tokio::test(flavor = "current_thread")]
async fn test_queued_startup_delivery_drop_retains_resources_through_checked_settlement() {
    use std::{
        collections::BTreeMap,
        future::Future,
        os::unix::fs::PermissionsExt,
        pin::Pin,
        task::{Context, Poll},
    };
    struct StartupProbe<F> {
        inner: Pin<Box<F>>,
        disposed: Arc<AtomicBool>,
        signal: Option<tokio::sync::oneshot::Sender<()>>,
    }
    impl<F: Future> Future for StartupProbe<F> {
        type Output = F::Output;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.get_mut().inner.as_mut().poll(cx)
        }
    }
    impl<F> Drop for StartupProbe<F> {
        fn drop(&mut self) {
            self.disposed.store(true, Ordering::SeqCst);
            if let Some(signal) = self.signal.take() {
                let _ = signal.send(());
            }
        }
    }
    struct ControlProbe<T> {
        _control: T,
        dropped: Arc<AtomicBool>,
    }
    impl<T> Drop for ControlProbe<T> {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }
    struct ResourceProbe {
        attempt: NativeAttemptCustody,
        startup_disposed: Arc<AtomicBool>,
        control_dropped: Arc<AtomicBool>,
        dropped: Arc<AtomicBool>,
        premature: Arc<AtomicBool>,
    }
    impl Drop for ResourceProbe {
        fn drop(&mut self) {
            let checked = self
                .attempt
                .latch()
                .checked_process_settlement
                .load(Ordering::SeqCst);
            self.premature.store(
                !checked
                    || !self.startup_disposed.load(Ordering::SeqCst)
                    || !self.control_dropped.load(Ordering::SeqCst),
                Ordering::SeqCst,
            );
            self.dropped.store(true, Ordering::SeqCst);
        }
    }
    let root = tempfile::tempdir().unwrap();
    let binary = root.path().join("queued.py");
    let marker = root.path().join("admitted");
    std::fs::write(
        &binary,
        r#"#!/usr/bin/python3
import os, pathlib, sys, time
sys.stdin.read()
with open(os.environ["QUEUED_MARKER"], "a") as marker:
    marker.write("one-child\n")
print('{"type":"thread.started","thread_id":"queued"}', flush=True)
time.sleep(30)
"#,
    )
    .unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
    let client = codex::CodexClient::builder()
        .binary(binary)
        .working_dir(root.path())
        .json(true)
        .mirror_stdout(false)
        .build();
    let attempt = NativeAttemptCustody::codex();
    let NativeAttemptCustody::Codex { process, .. } = &attempt else {
        unreachable!()
    };
    let process = process.clone();
    let disposed = Arc::new(AtomicBool::new(false));
    let control_dropped = Arc::new(AtomicBool::new(false));
    let released = Arc::new(AtomicBool::new(false));
    let premature = Arc::new(AtomicBool::new(false));
    let deliveries = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (signal, disposed_ack) = tokio::sync::oneshot::channel();
    let child_marker = marker.clone();
    let delivered = Arc::clone(&deliveries);
    let dropped_control = Arc::clone(&control_dropped);
    let startup = StartupProbe {
        inner: Box::pin(async move {
            let control = client
                .stream_exec_with_env_overrides_custody(
                    codex::ExecStreamRequest {
                        prompt: "queued handoff".into(),
                        ephemeral: false,
                        ignore_rules: false,
                        ignore_user_config: false,
                        idle_timeout: None,
                        output_last_message: None,
                        output_schema: None,
                        json_event_log: None,
                    },
                    &BTreeMap::from([("QUEUED_MARKER".into(), child_marker.display().to_string())]),
                    &process,
                )
                .await
                .map_err(|error| backend_error(error.to_string()))?;
            tokio::time::timeout(Duration::from_secs(5), async {
                while !child_marker.exists() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .map_err(|_| backend_error("real queued fixture never admitted"))?;
            delivered.fetch_add(1, Ordering::SeqCst);
            Ok(ControlProbe {
                _control: control,
                dropped: dropped_control,
            })
        }),
        disposed: Arc::clone(&disposed),
        signal: Some(signal),
    };
    let resources = ResourceProbe {
        attempt: attempt.clone(),
        startup_disposed: Arc::clone(&disposed),
        control_dropped: Arc::clone(&control_dropped),
        dropped: Arc::clone(&released),
        premature: Arc::clone(&premature),
    };
    let mut public_startup = Box::pin(retain_native_startup(startup, attempt.clone(), resources));
    std::future::poll_fn(|cx| {
        assert!(public_startup.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    tokio::time::timeout(Duration::from_secs(5), disposed_ack)
        .await
        .unwrap()
        .unwrap();
    // On this current-thread runtime, the owned startup task performs no await
    // between actual Box disposal and send(), then yields waiting for transfer.
    // The public receiver has not been polled since its first Pending result.
    assert!(disposed.load(Ordering::SeqCst));
    assert_eq!(deliveries.load(Ordering::SeqCst), 1);
    assert!(
        !control_dropped.load(Ordering::SeqCst),
        "successful control is still queued"
    );
    assert!(!released.load(Ordering::SeqCst));
    assert!(!attempt
        .latch()
        .checked_process_settlement
        .load(Ordering::SeqCst));
    drop(public_startup);
    assert!(control_dropped.load(Ordering::SeqCst));
    assert!(
        !released.load(Ordering::SeqCst),
        "queued resource drop must retain the startup owner's custody"
    );
    assert!(
        tokio::time::timeout(Duration::from_secs(5), attempt.settlement())
            .await
            .unwrap()
            .is_ok()
    );
    assert!(released.load(Ordering::SeqCst));
    assert!(
        !premature.load(Ordering::SeqCst),
        "C must outlive actual boxed startup/control disposal and checked SDK settlement"
    );
    assert_eq!(deliveries.load(Ordering::SeqCst), 1);
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "one-child\n");
}
