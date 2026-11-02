use super::support::*;
use serde_json::json;

const CLAUDE_MAPPING_CONTRACT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/specs/claude-code-session-mapping-contract.md"
));

fn idx(argv: &[String], needle: &str) -> usize {
    argv.iter()
        .position(|arg| arg == needle)
        .unwrap_or_else(|| panic!("missing argv token: {needle}"))
}

#[test]
fn claude_adapter_implements_backend_harness_adapter_contract() {
    fn assert_impl<T: crate::backend_harness::BackendHarnessAdapter>() {}
    assert_impl::<ClaudeHarnessAdapter>();
}

#[test]
fn claude_backend_routes_through_harness_and_does_not_reintroduce_orchestration_primitives() {
    const SOURCE: &str = include_str!("../backend.rs");

    assert!(
        SOURCE.contains("run_harnessed_backend("),
        "expected Claude backend to route through the harness entrypoint"
    );
    assert!(
        SOURCE.contains("run_harnessed_backend_control("),
        "expected Claude backend to route cancellation through the harness control entrypoint"
    );
    assert!(
        SOURCE.contains("TerminationState::new"),
        "expected Claude backend control path to register a termination hook"
    );

    assert!(
        !SOURCE.contains("build_gated_run_handle("),
        "expected Claude backend to not bypass harness-owned completion gating"
    );
    assert!(
        !SOURCE.contains("mpsc::channel::<AgentWrapperEvent>(32)"),
        "expected Claude backend to not create a backend-local events channel"
    );
    assert!(
        !SOURCE.contains("tokio::time::timeout("),
        "expected Claude backend to not wrap runs with backend-local timeout orchestration"
    );
}

#[test]
fn claude_backend_mcp_write_hooks_route_through_shared_mcp_runner() {
    const SOURCE: &str = include_str!("../backend.rs");

    assert!(SOURCE.contains("fn mcp_add("));
    assert!(SOURCE.contains("mcp_management::claude_mcp_add_argv"));
    assert!(SOURCE.contains("fn mcp_remove("));
    assert!(SOURCE.contains("mcp_management::claude_mcp_remove_argv"));
    assert!(
        SOURCE.matches("mcp_management::run_claude_mcp(").count() >= 4,
        "expected list/get/add/remove hooks to reuse the shared Claude MCP runner"
    );
}

#[test]
fn claude_downstream_mapping_surfaces_do_not_reopen_raw_add_dirs_parsing() {
    const RAW_KEY: &str = "agent_api.exec.add_dirs.v1";
    const BACKEND_SOURCE: &str = include_str!("../backend.rs");
    const HARNESS_SOURCE: &str = include_str!("../harness.rs");
    const MAPPING_SOURCE: &str = include_str!("../mapping.rs");
    const UTIL_SOURCE: &str = include_str!("../util.rs");
    const MCP_ARGV_SOURCE: &str = include_str!("../mcp_management/argv.rs");
    const MCP_RESOLVE_SOURCE: &str = include_str!("../mcp_management/resolve.rs");
    const MCP_RUNNER_SOURCE: &str = include_str!("../mcp_management/runner.rs");

    assert!(
        !BACKEND_SOURCE.contains(RAW_KEY),
        "expected backend.rs to avoid reopening raw add-dir payload parsing"
    );
    assert!(
        !MAPPING_SOURCE.contains(RAW_KEY),
        "expected mapping.rs to avoid reopening raw add-dir payload parsing"
    );
    assert!(
        !MCP_ARGV_SOURCE.contains(RAW_KEY),
        "expected mcp argv helpers to avoid raw add-dir payload parsing"
    );
    assert!(
        !MCP_RESOLVE_SOURCE.contains(RAW_KEY),
        "expected mcp resolve helpers to avoid raw add-dir payload parsing"
    );
    assert!(
        !MCP_RUNNER_SOURCE.contains(RAW_KEY),
        "expected mcp runner helpers to avoid raw add-dir payload parsing"
    );
    assert!(
        HARNESS_SOURCE.contains("normalize_add_dirs_v1"),
        "expected harness.rs to keep add-dir normalization on the shared helper path"
    );
    assert!(
        HARNESS_SOURCE.contains("build_fresh_run_print_request("),
        "expected harness.rs to route normalized add-dir mapping through the focused print-request helper"
    );
    assert!(
        UTIL_SOURCE.contains(".add_dirs("),
        "expected util.rs to map normalized add dirs into ClaudePrintRequest"
    );
}

#[test]
fn claude_harness_extracts_add_dirs_exactly_once_and_carries_policy_state_forward() {
    const SOURCE: &str = include_str!("../harness.rs");

    assert_eq!(
        SOURCE
            .matches("request.extensions.get(EXT_ADD_DIRS_V1)")
            .count(),
        1,
        "expected Claude harness to read the raw add-dir extension exactly once"
    );
    assert_eq!(
        SOURCE
            .matches("normalize_add_dirs_v1(Some(raw), resolved_working_dir.as_deref())")
            .count(),
        1,
        "expected Claude harness to normalize add-dir payloads exactly once"
    );
    assert!(
        SOURCE.contains("resolve_claude_effective_working_dir("),
        "expected Claude harness to resolve effective working dir through the shared helper"
    );
    assert!(
        SOURCE.contains("pub(super) add_dirs: Vec<PathBuf>"),
        "expected ClaudeExecPolicy to carry normalized add-dir policy state"
    );
    assert!(
        SOURCE.contains("pub(super) resolved_working_dir: Option<PathBuf>"),
        "expected ClaudeExecPolicy to carry the resolved working dir used at spawn time"
    );
    assert!(
        SOURCE.contains("build_fresh_run_print_request("),
        "expected spawn-time Claude wiring to route root-flags assembly through the focused helper"
    );
}

#[test]
fn claude_fresh_run_print_request_emits_one_variadic_add_dir_group_in_order() {
    let argv = super::super::util::build_fresh_run_print_request(
        "hello".to_string(),
        true,
        false,
        false,
        &[
            std::path::PathBuf::from("/tmp/alpha"),
            std::path::PathBuf::from("/tmp/beta"),
        ],
    )
    .argv();

    let add_dir_idx = idx(&argv, "--add-dir");
    let verbose_idx = idx(&argv, "--verbose");
    let prompt_idx = idx(&argv, "hello");

    assert_eq!(
        argv.iter()
            .filter(|arg| arg.as_str() == "--add-dir")
            .count(),
        1,
        "expected exactly one variadic add-dir group"
    );
    assert_eq!(
        &argv[(add_dir_idx + 1)..(add_dir_idx + 3)],
        ["/tmp/alpha".to_string(), "/tmp/beta".to_string()],
        "expected normalized add-dir values to follow the single flag in order"
    );
    assert!(
        add_dir_idx < verbose_idx,
        "expected add-dir group to stay before the final verbose flag"
    );
    assert!(
        verbose_idx < prompt_idx,
        "expected verbose to stay before the final prompt token"
    );
}

#[test]
fn claude_fresh_run_print_request_omits_add_dir_flag_when_policy_list_is_empty() {
    let argv = super::super::util::build_fresh_run_print_request(
        "hello".to_string(),
        true,
        false,
        false,
        &[],
    )
    .argv();

    assert!(
        !argv.iter().any(|arg| arg == "--add-dir"),
        "expected no add-dir flag when the normalized policy list is empty"
    );
    assert!(
        idx(&argv, "--verbose") < idx(&argv, "hello"),
        "expected prompt to remain final even when add dirs are absent"
    );
}

#[test]
fn claude_fresh_run_print_request_places_allow_flag_before_variadic_add_dir_group() {
    let argv = super::super::util::build_fresh_run_print_request(
        "hello".to_string(),
        true,
        true,
        true,
        &[
            std::path::PathBuf::from("/tmp/alpha"),
            std::path::PathBuf::from("/tmp/beta"),
        ],
    )
    .argv();

    let dangerous_idx = idx(&argv, "--dangerously-skip-permissions");
    let allow_idx = idx(&argv, "--allow-dangerously-skip-permissions");
    let add_dir_idx = idx(&argv, "--add-dir");
    let verbose_idx = idx(&argv, "--verbose");
    let prompt_idx = idx(&argv, "hello");

    assert!(
        dangerous_idx < allow_idx,
        "expected dangerous-skip flag to precede the allow flag"
    );
    assert!(
        allow_idx < add_dir_idx,
        "expected allow flag to stay in the root-flags segment before add-dir"
    );
    assert!(
        add_dir_idx < verbose_idx,
        "expected add-dir group to stay before the final verbose flag"
    );
    assert!(
        verbose_idx < prompt_idx,
        "expected verbose to stay before the final prompt token"
    );
}

#[test]
fn claude_add_dirs_runtime_rejection_classifier_requires_exact_safe_message_match() {
    let payload = json!({
        "type": "result",
        "subtype": "error",
        "message": super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE,
        "details": {
            "stderr": "backend-private sentinel",
        }
    });

    assert!(super::super::util::json_contains_add_dirs_runtime_rejection_signal(&payload));
}

#[test]
fn claude_add_dirs_runtime_rejection_classifier_does_not_match_generic_or_selector_failures() {
    let generic_payload = json!({
        "type": "result",
        "subtype": "error",
        "message": "claude generic failure",
    });
    let selector_payload = json!({
        "type": "result",
        "subtype": "error",
        "message": "session not found",
    });
    let almost_payload = json!({
        "type": "result",
        "subtype": "error",
        "message": "prefix add_dirs rejected by runtime",
    });

    assert!(!super::super::util::json_contains_add_dirs_runtime_rejection_signal(&generic_payload));
    assert!(
        !super::super::util::json_contains_add_dirs_runtime_rejection_signal(&selector_payload)
    );
    assert!(!super::super::util::json_contains_add_dirs_runtime_rejection_signal(&almost_payload));
}

#[test]
fn claude_add_dirs_runtime_rejection_classifier_ignores_nested_detail_traps() {
    let payload = json!({
        "type": "result",
        "subtype": "error",
        "message": "other failure",
        "details": {
            "stderr": super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE,
        }
    });

    assert!(!super::super::util::json_contains_add_dirs_runtime_rejection_signal(&payload));
}

#[test]
fn claude_harness_prioritizes_top_level_add_dirs_runtime_rejection_over_selector_miss_traps() {
    const SOURCE: &str = include_str!("../harness.rs");
    const UTIL_SOURCE: &str = include_str!("../util.rs");

    let selector_classifier_idx = SOURCE
        .find("json_contains_not_found_signal(raw)")
        .expect("expected selector-failure classifier");
    let add_dirs_classifier_idx = SOURCE
        .find("json_contains_add_dirs_runtime_rejection_signal(raw)")
        .expect("expected add-dir runtime rejection classifier");

    assert!(
        add_dirs_classifier_idx < selector_classifier_idx,
        "expected top-level add-dir runtime rejection classification to run before selector-miss scanning"
    );
    assert!(
        SOURCE.contains("ADD_DIRS_RUNTIME_REJECTION_MESSAGE"),
        "expected harness to use the pinned add-dir runtime rejection message"
    );
    assert!(
        UTIL_SOURCE.contains("json_contains_add_dirs_runtime_rejection_signal"),
        "expected the classifier contract to stay anchored to the exact top-level add-dir rejection helper"
    );
    assert!(
        UTIL_SOURCE.contains("Some(SessionSelectorV1::Last) => Some(\"no session found\".to_string())"),
        "expected selector='last' failure to keep the pinned safe message in completion message resolution"
    );
    assert!(
        UTIL_SOURCE.contains("Some(SessionSelectorV1::Id { .. }) => Some(\"session not found\".to_string())"),
        "expected selector='id' failure to keep the pinned safe message in completion message resolution"
    );
}

#[test]
fn claude_mapping_contract_pins_add_dirs_session_ordering_clauses() {
    assert!(
        CLAUDE_MAPPING_CONTRACT.contains("emit exactly one `--add-dir <DIR...>` argv group"),
        "expected canonical Claude mapping contract to pin a single variadic add-dir group"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT
            .contains("The group MUST appear after any accepted `--model <trimmed-id>` pair."),
        "expected canonical Claude mapping contract to pin model-before-add-dir ordering"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT.contains("[--add-dir <DIR...>] --continue --verbose PROMPT"),
        "expected canonical Claude mapping contract to pin resume(last) add-dir ordering"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT.contains("[--add-dir <DIR...>] --resume ID --verbose PROMPT"),
        "expected canonical Claude mapping contract to pin resume(id) add-dir ordering"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT
            .contains("[--add-dir <DIR...>] --continue --fork-session --verbose PROMPT"),
        "expected canonical Claude mapping contract to pin fork(last) add-dir ordering"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT
            .contains("[--add-dir <DIR...>] --fork-session --resume ID --verbose PROMPT"),
        "expected canonical Claude mapping contract to pin fork(id) add-dir ordering"
    );
}

#[test]
fn claude_mapping_contract_pins_add_dirs_runtime_rejection_parity() {
    assert!(
        CLAUDE_MAPPING_CONTRACT.contains("Runtime rejection parity (pinned):"),
        "expected canonical Claude mapping contract to define add-dir runtime rejection parity"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT.contains("`add_dirs rejected by runtime`"),
        "expected canonical Claude mapping contract to pin the backend-owned add-dir runtime rejection message"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT
            .contains("emit exactly one terminal `AgentWrapperEventKind::Error` event"),
        "expected canonical Claude mapping contract to pin one terminal error event"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT.contains("message` surfaced through the completion error"),
        "expected canonical Claude mapping contract to pin event/completion message parity"
    );
    assert!(
        CLAUDE_MAPPING_CONTRACT
            .contains("MUST NOT classify selector misses (`\"no session found\"` / `\"session not found\"`) as"),
        "expected canonical Claude mapping contract to keep selector misses distinct from add-dir runtime rejection"
    );
}

#[test]
fn claude_completion_returns_backend_error_when_backend_error_message_is_present() {
    let adapter = new_adapter();

    let err = adapter
        .map_completion(super::super::harness::ClaudeBackendCompletion {
            status: exit_status_with_code(1),
            final_text: None,
            backend_error_message: Some(
                super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE.to_string(),
            ),
        })
        .expect_err("completion should surface a backend error");

    match err {
        AgentWrapperError::Backend { message } => assert_eq!(
            message,
            super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE
        ),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[test]
fn claude_completion_message_resolution_promotes_runtime_backend_error_on_zero_exit() {
    let (backend_error_message, terminal_error_event_message) =
        super::super::util::resolve_completion_messages(
            &success_exit_status(),
            None,
            false,
            false,
            Some(super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE.to_string()),
        );

    assert_eq!(
        backend_error_message.as_deref(),
        Some(super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
    );
    assert_eq!(
        terminal_error_event_message.as_deref(),
        Some(super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
    );
}

#[test]
fn claude_completion_message_resolution_keeps_selector_not_found_non_zero_only() {
    let (zero_exit_backend_error, zero_exit_terminal_error) =
        super::super::util::resolve_completion_messages(
            &success_exit_status(),
            Some(&super::super::super::session_selectors::SessionSelectorV1::Last),
            false,
            true,
            None,
        );
    assert_eq!(zero_exit_backend_error, None);
    assert_eq!(zero_exit_terminal_error, None);

    let (non_zero_backend_error, non_zero_terminal_error) =
        super::super::util::resolve_completion_messages(
            &exit_status_with_code(1),
            Some(&super::super::super::session_selectors::SessionSelectorV1::Last),
            false,
            true,
            None,
        );
    assert_eq!(non_zero_backend_error.as_deref(), Some("no session found"));
    assert_eq!(non_zero_terminal_error.as_deref(), Some("no session found"));
}

#[test]
fn claude_terminal_error_backend_event_maps_to_one_error_wrapper_event() {
    let adapter = new_adapter();

    let mapped = adapter.map_event(ClaudeBackendEvent::TerminalError {
        message: super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE.to_string(),
    });

    assert_eq!(mapped.len(), 1);
    assert_eq!(mapped[0].kind, AgentWrapperEventKind::Error);
    assert_eq!(
        mapped[0].message.as_deref(),
        Some(super::super::util::ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
    );
}
