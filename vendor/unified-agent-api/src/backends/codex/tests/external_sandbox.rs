use super::support::*;
use serde_json::json;

#[cfg(windows)]
use std::path::{Component, Path, Prefix};

#[test]
fn codex_harness_supported_extension_keys_include_agent_api_config_model_v1() {
    let adapter = test_adapter();
    assert!(adapter
        .supported_extension_keys()
        .contains(&crate::EXT_AGENT_API_CONFIG_MODEL_V1));
}

#[test]
fn codex_normalize_request_accepts_agent_api_config_model_v1_and_trims_it() {
    let adapter = test_adapter();
    let defaults = BackendDefaults::default();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        crate::EXT_AGENT_API_CONFIG_MODEL_V1.to_string(),
        json!("  gpt-5-codex  "),
    );

    let normalized = crate::backend_harness::normalize_request(&adapter, &defaults, request)
        .expect("model-selection key should be accepted for codex");

    assert_eq!(normalized.model_id.as_deref(), Some("gpt-5-codex"));
}

#[test]
fn codex_backend_does_not_advertise_external_sandbox_exec_by_default() {
    assert!(!CodexBackendConfig::default().allow_external_sandbox_exec);

    let backend = CodexBackend::new(CodexBackendConfig::default());
    let capabilities = backend.capabilities();
    assert!(!capabilities.contains(EXT_EXTERNAL_SANDBOX_V1));

    let adapter = test_adapter();
    assert!(!adapter
        .supported_extension_keys()
        .contains(&EXT_EXTERNAL_SANDBOX_V1));
}

#[test]
fn codex_backend_advertises_external_sandbox_exec_when_opted_in_and_normalize_allows_key() {
    let config = CodexBackendConfig {
        allow_external_sandbox_exec: true,
        ..Default::default()
    };

    let backend = CodexBackend::new(config.clone());
    let capabilities = backend.capabilities();
    assert!(capabilities.contains(EXT_EXTERNAL_SANDBOX_V1));

    let adapter = test_adapter_with_config(config);
    assert!(adapter
        .supported_extension_keys()
        .contains(&EXT_EXTERNAL_SANDBOX_V1));

    let defaults = BackendDefaults::default();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request
        .extensions
        .insert(EXT_EXTERNAL_SANDBOX_V1.to_string(), json!(true));

    crate::backend_harness::normalize_request(&adapter, &defaults, request)
        .expect("expected external sandbox key to pass allowlist gate when opted in");
}

#[test]
fn external_sandbox_extension_key_fails_closed_when_opt_in_disabled() {
    let adapter = test_adapter();
    let defaults = BackendDefaults::default();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request
        .extensions
        .insert(EXT_EXTERNAL_SANDBOX_V1.to_string(), json!(true));

    let err = match crate::backend_harness::normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("expected UnsupportedCapability when opt-in is disabled"),
        Err(err) => err,
    };
    match err {
        AgentWrapperError::UnsupportedCapability { capability, .. } => {
            assert_eq!(capability, EXT_EXTERNAL_SANDBOX_V1);
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
}

#[tokio::test]
async fn external_sandbox_spawn_failure_emits_warning_before_terminal_error() {
    let adapter = test_adapter_with_config(CodexBackendConfig {
        allow_external_sandbox_exec: true,
        ..Default::default()
    });

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello".to_string(),
            model_id: None,
            working_dir: None,
            effective_timeout: None,
            env: std::collections::BTreeMap::new(),
            policy: CodexExecPolicy {
                add_dirs: Vec::new(),
                non_interactive: true,
                external_sandbox: true,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume: None,
                fork: None,
            },
        })
        .await
        .expect("startup failure should still return a stream when external sandbox is enabled");

    let backend_events: Vec<CodexBackendEvent> = spawned
        .events
        .map(|result| result.expect("synthetic startup-failure events should be infallible"))
        .collect()
        .await;
    assert_eq!(backend_events.len(), 2);
    assert!(matches!(
        backend_events[0],
        CodexBackendEvent::ExternalSandboxWarning
    ));
    assert!(matches!(
        &backend_events[1],
        CodexBackendEvent::TerminalError { message }
        if message == "codex backend failed to resolve working directory"
    ));

    let mapped: Vec<_> = backend_events
        .into_iter()
        .flat_map(|event| adapter.map_event(event))
        .collect();
    assert_eq!(mapped.len(), 2);
    assert_eq!(mapped[0].kind, AgentWrapperEventKind::Status);
    assert_eq!(
        mapped[0].message.as_deref(),
        Some(PINNED_EXTERNAL_SANDBOX_WARNING)
    );
    assert_eq!(mapped[1].kind, AgentWrapperEventKind::Error);
    assert_eq!(
        mapped[1].message.as_deref(),
        Some("codex backend failed to resolve working directory")
    );

    let err = spawned
        .completion
        .await
        .expect_err("startup failure completion should preserve the backend error");
    assert!(matches!(err, CodexBackendError::WorkingDirectoryUnresolved));
}

#[cfg(windows)]
#[tokio::test]
async fn exec_cross_drive_drive_relative_working_dir_fails_before_spawn() {
    let run_start_cwd = std::env::temp_dir().join("codex-cross-drive-exec");
    let adapter = test_adapter_with_run_start_cwd(Some(run_start_cwd.clone()));

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello".to_string(),
            model_id: None,
            working_dir: Some(windows_drive_relative_on_other_drive(
                "repo",
                &run_start_cwd,
            )),
            effective_timeout: None,
            env: std::collections::BTreeMap::new(),
            policy: CodexExecPolicy {
                add_dirs: Vec::new(),
                non_interactive: true,
                external_sandbox: false,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume: None,
                fork: None,
            },
        })
        .await
        .expect("startup failure should still return a stream");

    let backend_events: Vec<CodexBackendEvent> = spawned
        .events
        .map(|result| result.expect("synthetic startup-failure events should be infallible"))
        .collect()
        .await;
    assert_eq!(backend_events.len(), 1);
    assert!(matches!(
        &backend_events[0],
        CodexBackendEvent::TerminalError { message }
        if message == "codex backend failed to resolve working directory"
    ));

    let err = spawned
        .completion
        .await
        .expect_err("startup failure completion should preserve the backend error");
    assert!(matches!(err, CodexBackendError::WorkingDirectoryUnresolved));
}

#[cfg(windows)]
#[tokio::test]
async fn fork_cross_drive_drive_relative_working_dir_fails_before_app_server_start() {
    let run_start_cwd = std::env::temp_dir().join("codex-cross-drive-fork");
    let adapter = test_adapter_with_run_start_cwd(Some(run_start_cwd.clone()));

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello".to_string(),
            model_id: None,
            working_dir: Some(windows_drive_relative_on_other_drive(
                "repo",
                &run_start_cwd,
            )),
            effective_timeout: None,
            env: std::collections::BTreeMap::new(),
            policy: CodexExecPolicy {
                add_dirs: Vec::new(),
                non_interactive: true,
                external_sandbox: false,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume: None,
                fork: Some(crate::backends::session_selectors::SessionSelectorV1::Last),
            },
        })
        .await
        .expect("startup failure should still return a stream");

    let backend_events: Vec<CodexBackendEvent> = spawned
        .events
        .map(|result| result.expect("synthetic startup-failure events should be infallible"))
        .collect()
        .await;
    assert_eq!(backend_events.len(), 1);
    assert!(matches!(
        &backend_events[0],
        CodexBackendEvent::TerminalError { message }
        if message == "codex backend failed to resolve working directory"
    ));

    let err = spawned
        .completion
        .await
        .expect_err("startup failure completion should preserve the backend error");
    assert!(matches!(err, CodexBackendError::WorkingDirectoryUnresolved));
}

#[cfg(windows)]
fn windows_drive_relative_on_other_drive(
    relative: &str,
    absolute_path: &Path,
) -> std::path::PathBuf {
    let current_drive = absolute_path
        .components()
        .find_map(|component| match component {
            Component::Prefix(value) => match value.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => {
                    Some(drive.to_ascii_lowercase())
                }
                _ => None,
            },
            _ => None,
        })
        .expect("absolute windows path should include a disk prefix");
    let alternate_drive = if current_drive == b'c' { 'd' } else { 'c' };
    std::path::PathBuf::from(format!("{alternate_drive}:{relative}"))
}
