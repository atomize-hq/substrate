use super::support::*;
use codex::ExecStreamError;

#[test]
fn codex_adapter_implements_backend_harness_adapter_contract() {
    fn assert_impl<T: crate::backend_harness::BackendHarnessAdapter>() {}
    assert_impl::<CodexHarnessAdapter>();
}

#[test]
fn codex_backend_routes_through_harness_and_does_not_reintroduce_orchestration_primitives() {
    const SOURCE: &str = include_str!("../backend.rs");

    assert!(
        SOURCE.contains("run_harnessed_backend("),
        "expected Codex backend to route through the harness entrypoint"
    );
    assert!(
        SOURCE.contains("run_harnessed_backend_control("),
        "expected Codex backend to route cancellation through the harness control entrypoint"
    );
    assert!(
        SOURCE.contains("TerminationState::new"),
        "expected Codex backend control path to register a termination hook"
    );

    assert!(
        !SOURCE.contains("build_gated_run_handle("),
        "expected Codex backend to not bypass harness-owned completion gating"
    );
    assert!(
        !SOURCE.contains("mpsc::channel::<AgentWrapperEvent>(32)"),
        "expected Codex backend to not create a backend-local events channel"
    );
    assert!(
        !SOURCE.contains("tokio::time::timeout("),
        "expected Codex backend to not wrap runs with backend-local timeout orchestration"
    );
}

#[test]
fn codex_backend_mcp_write_hooks_route_through_shared_mcp_runner() {
    const SOURCE: &str = include_str!("../backend.rs");

    assert!(SOURCE.contains("fn mcp_add("));
    assert!(SOURCE.contains("mcp_management::codex_mcp_add_argv"));
    assert!(SOURCE.contains("fn mcp_remove("));
    assert!(SOURCE.contains("mcp_management::codex_mcp_remove_argv"));
    assert!(
        SOURCE.matches("mcp_management::run_codex_mcp(").count() >= 4,
        "expected list/get/add/remove hooks to reuse the shared Codex MCP runner"
    );
}

#[test]
fn codex_downstream_mapping_surfaces_do_not_reopen_raw_add_dirs_parsing() {
    const RAW_KEY: &str = "agent_api.exec.add_dirs.v1";
    const EXEC_SOURCE: &str = include_str!("../exec.rs");
    const FORK_SOURCE: &str = include_str!("../fork.rs");
    const MAPPING_SOURCE: &str = include_str!("../mapping.rs");

    assert!(
        !EXEC_SOURCE.contains(RAW_KEY),
        "expected exec.rs to consume policy.add_dirs without raw add-dir key parsing"
    );
    assert!(
        !FORK_SOURCE.contains(RAW_KEY),
        "expected fork.rs to avoid reopening raw add-dir payload parsing"
    );
    assert!(
        !MAPPING_SOURCE.contains(RAW_KEY),
        "expected mapping.rs to avoid reopening raw add-dir payload parsing"
    );
}

#[test]
fn redact_exec_stream_error_does_not_leak_raw_jsonl_line() {
    let secret = "RAW-LINE-SECRET-DO-NOT-LEAK";
    let err = ExecStreamError::Normalize {
        line: secret.to_string(),
        message: "missing required context".to_string(),
    };

    let redacted = redact_exec_stream_error(&err);
    assert!(!redacted.contains(secret));
    assert!(redacted.contains("line_bytes="));
}

#[test]
fn codex_add_dirs_runtime_rejection_classifier_requires_exact_safe_message_match() {
    assert!(super::super::is_add_dirs_runtime_rejection_signal(
        super::super::PINNED_ADD_DIRS_RUNTIME_REJECTION
    ));
}

#[test]
fn codex_add_dirs_runtime_rejection_classifier_does_not_match_generic_or_prefixed_messages() {
    assert!(!super::super::is_add_dirs_runtime_rejection_signal(
        "codex generic failure"
    ));
    assert!(!super::super::is_add_dirs_runtime_rejection_signal(
        "prefix add_dirs rejected by runtime"
    ));
}

#[test]
fn codex_model_runtime_rejection_classifier_requires_explicit_code() {
    assert!(super::super::exec::is_model_runtime_rejection_signal(Some(
        "model_runtime_rejection"
    )));
}

#[test]
fn codex_model_runtime_rejection_classifier_does_not_match_short_model_substrings() {
    assert!(!super::super::exec::is_model_runtime_rejection_signal(None));
    assert!(!super::super::exec::is_model_runtime_rejection_signal(
        Some("transport_error")
    ));
}
