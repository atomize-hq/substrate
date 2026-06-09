# TASKS-53: Inventory-Selected First Runtime-Family Host-Orchestrator Tool Surface Landing

Source spec: [SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)  
Source plan: [PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)

## Task List

- [ ] Task 53.1: Preserve dynamic orchestrator selection and add explicit live-tool validation/support posture
  - Acceptance:
    - selected orchestrator resolution still flows through effective config + effective inventory + exact backend policy
    - validation/support posture is derived from resolved runtime family, not literal `agent_id`
    - Codex is treated as the first real smoke/validation floor without turning that into doc-driven blocking for other selected runtimes
    - non-Codex runtimes keep ordinary host-session behavior unless implementation truth proves a real incompatibility
  - Verify:
    - `cargo test -p shell --test agents_validate -- --nocapture`
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/validator.rs`
    - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
    - `crates/shell/src/execution/agents_cmd.rs`

- [ ] Task 53.2: Add a bounded first-validated Codex-backed host toolbox-env plus prompt-contract injection path at the prompt-submission boundary
  - Acceptance:
    - the Codex-backed path receives a real seven-tool surface in the first smoke-validated landing
    - that surface is enabled by runtime-owned `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` / `SUBSTRATE_AGENT_TOOLBOX_VERSION` injection via UAA rather than assuming direct generic tool-list injection
    - startup/system-prompt composition includes the seven-tool contract details because the first landing is not MCP-server-based registration
    - the likely concrete seams are `AgentWrapperRunRequest.env` in `submit_host_prompt_turn(...)` for toolbox env injection and `HiddenOwnerHelperStartupPromptPlan.prompt_text` for prompt-contract composition
    - any Codex-specific config shaping is secondary to that endpoint injection and stays bounded
    - the implementation keeps the adapter generic where already honest rather than introducing a documentation-only runtime block
    - no hard-coded orchestrator id is required to enable that path
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/prompt_fulfillment.rs`
    - `crates/shell/src/execution/agent_runtime/control.rs`
    - `crates/shell/src/execution/agents_cmd.rs`
    - `crates/gateway/src/adapter_runtime.rs`
    - `crates/shell/Cargo.toml` (only if a bounded UAA dependency adjustment is required)

- [ ] Task 53.3: Reuse Slice 52 tool semantics for live first-validated host tool invocation
  - Acceptance:
    - live tool calls on the first validated Codex-backed path route through `tool_invocation_contract.rs`
    - runtime-owned request/session/caller/world fields remain shell-injected
    - returned results preserve frozen `task_run_id` / retained-worker receipt semantics
  - Verify:
    - `cargo test -p shell agent_runtime::tool_invocation_contract -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
    - `crates/shell/src/execution/prompt_fulfillment.rs`
    - `crates/shell/src/repl/async_repl.rs`

- [ ] Task 53.4: Add non-Codex validation/support-posture truth coverage for selected `claude_code` orchestrators
  - Acceptance:
    - selected `claude_code` host sessions keep existing prompt-turn behavior
    - reporting does not overclaim parity or guaranteed host-tool support for that family
    - docs/tests do not force a Codex-only runtime gate unless implementation truth requires it
    - no fallback to `codex` occurs implicitly
  - Verify:
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
    - `crates/shell/tests/agent_public_control_surface_v1.rs`
    - `crates/shell/src/execution/agents_cmd.rs`

- [ ] Task 53.5: Add truthful operator/reporting surfaces for first-family landing state
  - Acceptance:
    - `substrate agent doctor --json` and/or `substrate agent toolbox status --json` distinguish the first validated Codex-backed path from broader runtime validation/support posture
    - reporting does not overclaim `claude_code` parity
    - exact backend ids remain visible where applicable
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agents_cmd.rs`
    - `crates/shell/tests/agent_public_control_surface_v1.rs`
    - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

- [ ] Task 53.6: Close validation wall and update slice tracker truth
  - Acceptance:
    - workspace fmt/clippy/targeted tests are green
    - remaining-scope tracker records the dynamic-selection clarification and any dependency/parity deferrals
    - Slice `53` artifacts describe exactly what landed and what remains deferred
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - `llm-last-mile/REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md`
    - `llm-last-mile/SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md`
    - `llm-last-mile/PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md`
    - `llm-last-mile/TASKS-53.md`
