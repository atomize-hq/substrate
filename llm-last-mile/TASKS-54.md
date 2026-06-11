# TASKS-54: Inventory-Selected Second Runtime-Family Host-Orchestrator Tool Surface Parity

Source spec: [SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md)  
Source plan: [PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md)
Execution model: four sequential implementation packets  
Phase: `TASKS`  
Status: landed and validated on `2026-06-11`

## Closeout note

Packet `4` aligned the Slice `54` ledger to live repo truth:

1. selected-host `claude_code` parity is landed and validated,
2. operator/reporting surfaces now publish the selected-host posture truthfully,
3. the remaining-scope note and gap matrix no longer treat `claude_code` parity as the next open seam.

## Current tree note

1. Slice `53` landed the first validated Codex-backed host-tool floor.
2. Slice `54` closes the selected-host `claude_code` runtime-family parity gap and aligns operator/reporting truth to that landed state.
3. Slice `54` does not widen into public toolbox CLI execution, MCP-first redesign, broader caller-surface productization, or Family-2 host-global ingress work.
4. Any later host-tool docs/smoke follow-through remains bounded to future live-surface widening rather than reopening parity itself.

## Task List

- [x] Task 54.1: Freeze the explicit `claude_code` parity target and support-posture uplift rules
  - Acceptance:
    - the repo defines exactly what must become true before `claude_code` stops being reported as `not_yet_smoke_validated` / `not_yet_guaranteed`
    - parity is keyed off resolved runtime family, not literal `agent_id` or hidden fallback
    - selected `claude_code` enablement is explicit and bounded rather than implied by doc wording alone
    - no hidden fallback to `codex` is introduced
  - Verify:
    - `cargo test -p shell --test agents_validate -- --nocapture`
    - `cargo test -p shell agent_runtime::dispatch_contract -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
    - `crates/shell/src/execution/agent_runtime/validator.rs` (only if needed)
    - `crates/shell/src/execution/agents_cmd.rs`

- [x] Task 54.2: Land selected `claude_code` host start/turn parity for authoritative toolbox env injection plus prompt-contract disclosure
  - Acceptance:
    - selected `claude_code` host starts no longer keep the old non-tool-staged path
    - selected `claude_code` host turns receive the same authoritative host-tool surface architecture already used by the first family floor
    - runtime-owned `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` / `SUBSTRATE_AGENT_TOOLBOX_VERSION` injection is preserved
    - startup/system-prompt composition preserves the seven-tool contract disclosure model for the live non-MCP surface
    - any Claude-specific adapter shaping stays bounded and does not change semantic contract truth
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/control.rs`
    - `crates/shell/src/execution/prompt_fulfillment.rs`
    - `crates/shell/src/execution/agents_cmd.rs`
    - `crates/gateway/src/adapter_runtime.rs` (only if a bounded adapter change is required)

- [x] Task 54.3: Reuse Slice `52` semantics for live `claude_code` tool invocation and receipt/follow-up parity
  - Acceptance:
    - live `claude_code` tool calls route through `tool_invocation_contract.rs`
    - runtime-owned request/session/caller/world fields remain shell-injected
    - returned results preserve frozen `task_run_id` / retained-worker receipt semantics
    - retained follow-up semantics remain shared across runtime families
    - worker messaging, lifecycle, and steering-policy semantics do not drift for `claude_code`
  - Verify:
    - `cargo test -p shell agent_runtime::tool_invocation_contract -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
    - `crates/shell/src/execution/prompt_fulfillment.rs`
    - `crates/shell/src/repl/async_repl.rs`
    - focused shell tests as needed

- [x] Task 54.4: Add selected `claude_code` parity regression coverage and remove the old non-tool-staged expectation
  - Acceptance:
    - tests now assert truthful selected `claude_code` host-tool parity instead of preserving the old startup-prompt omission
    - reporting does not overclaim more than the runtime actually proves
    - no test reintroduces hidden fallback to `codex`
    - exact backend-id and policy truth remain visible where applicable
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - `crates/shell/tests/agent_public_control_surface_v1.rs`
    - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
    - `crates/shell/tests/repl_world_first_routing_v1.rs`
    - `crates/shell/src/execution/agents_cmd.rs`

- [x] Task 54.5: Update operator/reporting surfaces and slice-tracker truth for second-family parity
  - Acceptance:
    - `substrate agent doctor --json` and/or `substrate agent toolbox status --json` reflect the new `claude_code` posture truthfully
    - remaining-scope docs no longer list `claude_code` parity as the active next seam once the slice is validated
    - the gap matrix remains aligned to the same repo truth and any bounded stale absolute links touched during closeout are corrected
    - Slice `54` artifacts describe exactly what landed and what remains deferred after parity
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `substrate agent doctor --json`
    - `substrate agent toolbox status --json`
  - Files:
    - `crates/shell/src/execution/agents_cmd.rs`
    - `llm-last-mile/REMAINING-overall-scope-2026-06-10.md`
    - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
    - `llm-last-mile/SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md`
    - `llm-last-mile/PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md`
    - `llm-last-mile/TASKS-54.md`

- [x] Task 54.6: Close the Slice `54` validation wall
  - Acceptance:
    - workspace fmt/clippy/targeted tests are green
    - the selected `claude_code` parity path is backed by validation evidence rather than by documentation-only uplift
    - repo reporting/docs remain consistent with the validated runtime truth
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - runtime/test files touched by Tasks `54.1` through `54.5`
    - closeout docs touched by Task `54.5`
