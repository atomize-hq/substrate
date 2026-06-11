# Spec: Inventory-Selected Second Runtime-Family Host-Orchestrator Tool Surface Parity

Source tracker note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior numbered slice: [SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)  
Companion inputs:
- [PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)
- [TASKS-53.md](./TASKS-53.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
Phase: `SPECIFY`  
Status: proposed for review on `2026-06-11`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The next honest slice after Slice `53` is second runtime-family parity for selected `claude_code` host orchestrators rather than another Codex-only hardening slice.
2. Slice `52` remains the semantic authority for the seven-tool contract, runtime-owned field injection, and receipt/follow-up handle rules.
3. Slice `53` already proved the first live host-tool floor through runtime-owned toolbox env injection plus startup/system-prompt contract disclosure, and Slice `54` should reuse that architecture rather than redesign it.
4. Live repo truth still says selected `claude_code` host starts keep the older non-tool-staged path today, so Slice `54` must explicitly replace that truth with parity coverage rather than merely changing reporting language.
5. Dynamic orchestrator selection remains authoritative:
   - effective config chooses the orchestrator id,
   - effective inventory resolves that id,
   - exact backend policy gates the selected backend,
   - `config.cli.runtime_family` realizes runtime-family behavior only after exact selection succeeds.
6. This slice should preserve the same internal toolbox transport and the same control-plane semantics for worker messaging, worker lifecycle, and steering policy rather than inventing a Claude-specific control plane.
7. Slice `54` should not reopen public toolbox CLI execution, MCP-first redesign, orchestrator selection semantics, transport redesign, or Family-2 host-global ingress follow-ons.
8. Any uplift from `not_yet_smoke_validated` / `not_yet_guaranteed` to a stronger `claude_code` posture must be earned by actual runtime validation and test evidence in this slice.

If any of these are wrong, correct them before implementation.

## Objective

Land truthful `claude_code` host-tool parity above the already-landed Slice `52`/`53` adapter path without reopening selection, transport, or receipt semantics.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/shell` host-runtime selection, startup/turn submission, reporting, and adapter wiring
- `crates/gateway` runtime-family adapter registration
- `unified-agent-api` run surface (`AgentWrapperRunRequest`)
- Existing session-scoped internal toolbox transport in `crates/shell/src/repl/async_repl.rs`

This slice is complete only when:

1. selected host orchestrators are still chosen from effective config + effective inventory + exact backend policy,
2. selected `claude_code` host starts and turns receive the same authoritative toolbox endpoint/version injection model already used by the first validated family floor,
3. selected `claude_code` host starts and turns receive the same startup/system-prompt contract disclosure model needed for the non-MCP live tool surface,
4. live `claude_code` tool calls route through the frozen Slice `52` contract into the landed internal toolbox transport,
5. live `claude_code` tool results preserve the same `task_run_id` and retained-worker receipt/follow-up semantics already frozen in Slice `52`,
6. worker messaging, worker lifecycle, and steering-policy semantics remain unchanged across runtime families,
7. operator/reporting surfaces upgrade `claude_code` posture only as far as validation truth actually proves,
8. the slice lands without hidden fallback to `codex`, without public toolbox verb expansion, and without reopening transport design.

Primary user story after this slice:

1. A user can still select a host orchestrator dynamically through config/inventory/policy.
2. If that selection resolves to `claude_code`, the live host session can use the same seven-tool orchestration surface already proven for the first runtime family.
3. The repo reports multi-family host-tool support truthfully instead of freezing `claude_code` in a permanent “ordinary host session only” posture after the underlying parity work has landed.

## Repo-Truth Gut Check

### 1. Slice 53 froze an explicit “first floor vs not-yet-parity” posture

Live `LiveToolSupportPosture::for_backend_kind(...)` in [`dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) currently says:

1. `codex` = `smoke_validated` / `first_supported_floor`,
2. `claude_code` = `not_yet_smoke_validated` / `not_yet_guaranteed`.

Repo-truth consequence:

1. Slice `54` must not pretend parity is already landed,
2. Slice `54` should explicitly replace this posture only when the runtime path and validation wall actually prove it.

### 2. The host prompt submission seam is already generic enough to target both families

Live `submit_host_prompt_turn(...)` in [`control.rs`](../crates/shell/src/execution/agent_runtime/control.rs) already:

1. composes the authoritative host-toolbox contract when enabled,
2. injects runtime-owned toolbox env through `AgentWrapperRunRequest.env`,
3. delegates through family-aware prompt fulfillment.

Repo-truth consequence:

1. Slice `54` should reuse this seam rather than invent a Claude-only submission path,
2. the likely parity work is in enablement, family-specific shaping, and regression coverage rather than in transport invention.

### 3. Selected `claude_code` host starts still preserve the old non-tool-staged truth

Live tests in [`agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs) still assert that selected `claude_code` host starts keep `startup_prompt.is_none()` so the repo does not overclaim parity.

Repo-truth consequence:

1. Slice `54` must deliberately change this repo truth,
2. the slice is not complete if only reporting changes while start/turn behavior stays on the old path.

### 4. The semantic contract is already frozen and runtime-family-independent

Live `tool_invocation_contract.rs` already freezes:

1. the seven tool names,
2. runtime-owned versus model-provided fields,
3. canonical receipt and follow-up semantics.

Repo-truth consequence:

1. Slice `54` should reuse this contract exactly,
2. any Claude-specific work should be adapter shaping only, not semantic drift.

### 5. Parity must preserve worker semantics, not just prompt injection

The tool-surface design stack already freezes:

1. retained worker messaging semantics,
2. worker lifecycle and invalidation semantics,
3. deny-by-default steering policy.

Repo-truth consequence:

1. Slice `54` must preserve the same control-plane meaning for `continue_world_worker`, `inspect_world_worker`, `fork_world_worker`, `cancel_world_work`, and `stop_world_worker`,
2. parity is not complete if only the startup prompt changes while receipt/follow-up semantics drift.

### 6. Operator truth already distinguishes “validated floor” from “not yet guaranteed”

Live status/doctor/toolbox reporting already has a vocabulary for support posture.

Repo-truth consequence:

1. Slice `54` should upgrade reporting from that existing posture model rather than invent a second reporting taxonomy,
2. any support-state uplift must match actual implementation and validation truth.

## Architectural Decision

### Dynamic selection stays authoritative

The selected host orchestrator for the parity landing remains:

1. chosen by `agents.hub.orchestrator_agent_id`,
2. resolved against effective inventory,
3. constrained by effective policy,
4. realized through `config.cli.runtime_family` only after exact selection succeeds.

This slice must not introduce:

1. a hard-coded preferred orchestrator id,
2. a hidden fallback from `claude_code` to `codex`,
3. a second runtime-family selection surface outside inventory.

### Slice 52 remains the semantic source of truth

The live `claude_code` parity landing must consume the frozen adapter contract in [`tool_invocation_contract.rs`](../crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs).

It must not:

1. rename the seven tools,
2. ask the model to supply runtime-owned ids,
3. alter receipt shapes,
4. redesign internal dispatch transport.

### Parity means same semantics, not necessarily byte-for-byte identical adapter internals

`claude_code` parity should mean:

1. the same toolbox endpoint/version injection model,
2. the same startup/system-prompt tool-contract disclosure model,
3. the same tool vocabulary,
4. the same runtime-owned field injection,
5. the same receipt/follow-up rules,
6. the same operator/support-posture truth model.

If `claude_code` requires bounded runtime-specific request shaping, that shaping must remain:

1. isolated to the runtime adapter seam,
2. explicitly documented,
3. semantically invisible above the adapter boundary.

### Worker messaging, lifecycle, and steering policy stay shared across runtime families

Slice `54` must preserve the already-frozen semantics from:

1. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md),
2. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md),
3. [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md).

This slice must not:

1. create Claude-specific message classes,
2. create Claude-specific lifecycle states,
3. widen steering authority beyond the existing deny-by-default matrix.

### Support-posture uplift must be earned, not assumed

Slice `54` should move `claude_code` from the current not-yet-validated posture only when:

1. selected host starts and turns actually use the live host-tool path,
2. live tool calls preserve Slice `52` semantics,
3. targeted tests and bounded smoke/validation are green,
4. reporting surfaces are updated to match that truth.

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Targeted tests:

```bash
cargo test -p shell --test agents_validate -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_runtime::dispatch_contract -- --nocapture
cargo test -p shell agent_runtime::tool_invocation_contract -- --nocapture
```

Focused operator-truth checks once implemented:

```bash
substrate agent doctor --json
substrate agent toolbox status --json
substrate agent toolbox env --json
```

## Project Structure

Relevant implementation seams:

```text
crates/shell/src/execution/agent_runtime/dispatch_contract.rs
  Live validation/support posture and exact backend/policy resolution truth.

crates/shell/src/execution/agent_runtime/control.rs
  Host prompt submission boundary; runtime-owned toolbox env injection and prompt-contract composition.

crates/shell/src/execution/prompt_fulfillment.rs
  Runtime-family adapter bridge for Codex and Claude Code launch/attach behavior.

crates/shell/src/execution/agents_cmd.rs
  Public start/turn planning and truthful operator/reporting surfaces.

crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs
  Frozen seven-tool contract, runtime-owned field injection, and receipt/result normalization.

crates/shell/src/repl/async_repl.rs
  Landed internal toolbox transport and event/result framing.

crates/shell/tests/agent_public_control_surface_v1.rs
  Public/operator truth and CLI behavior coverage.

crates/shell/tests/agent_successor_contract_ahcsitc0.rs
  Toolbox surface/read-path contract coverage.

crates/shell/tests/repl_world_first_routing_v1.rs
  End-to-end routing and receipt/follow-up behavior coverage.

llm-last-mile/REMAINING-overall-scope-2026-06-10.md
  Canonical remaining-scope tracker to update once parity lands.

AGENT_ORCHESTRATION_GAP_MATRIX.md
  Broader v1 gap tracker that should stay aligned with Slice 54 truth.
```

## Code Style

Prefer explicit runtime-family parity handling that reuses the same host-tool surface helpers.

Example style:

```rust
match runtime.descriptor.backend_kind {
    AgentRuntimeBackendKind::Codex | AgentRuntimeBackendKind::ClaudeCode => {
        enable_authoritative_host_tool_surface(&runtime, prompt)?
    }
}
```

Keep reporting truth derived from validated posture rather than literal string checks:

```rust
let posture = LiveToolSupportPosture::for_backend_kind(resolved.backend_kind);
render_tool_support_posture(posture);
```

And keep runtime-owned field injection centralized:

```rust
let request_prompt = maybe_compose_prompt_with_authoritative_host_toolbox_contract(
    prompt,
    host_toolbox_surface_authoritative,
);
let request_env = maybe_build_runtime_owned_toolbox_env(
    &orchestration_session_id,
    host_toolbox_surface_authoritative,
)?;
```

Do **not** replace shared semantics with family-specific ad hoc paths like:

```rust
if backend_id == "cli:claude_code" {
    // wrong: literal backend matching drifts away from inventory/runtime-family truth
}
```

## Testing Strategy

### Unit / focused contract coverage

1. preserve validator and dispatch-contract coverage proving exact backend selection and runtime-family posture remain separate concerns,
2. add focused coverage for selected `claude_code` host starts/turns taking the authoritative host-tool path,
3. add coverage that parity preserves runtime-owned env injection and startup/system-prompt contract disclosure,
4. keep coverage that exact backend ids survive even when multiple inventory entries realize the same runtime family.

### Runtime-family parity coverage

1. selected `claude_code` host starts and turns receive toolbox endpoint/version env injection,
2. selected `claude_code` host starts and turns receive the seven-tool contract disclosure needed for the live non-MCP surface,
3. live `claude_code` tool calls bridge through Slice `52` translation into internal toolbox transport,
4. returned receipts preserve `task_run_id` / retained-worker handles without runtime-family drift.

### Regression / truth-alignment coverage

1. `toolbox status` / `doctor` do not overclaim parity beyond what is actually validated,
2. no test reintroduces hidden fallback to `codex`,
3. no test reopens public toolbox verbs or transport redesign,
4. worker follow-up semantics remain shared across runtime families.

## Boundaries

- Always:
  - Preserve dynamic selection from config + effective inventory + exact backend policy.
  - Reuse Slice `52` tool names, injected-field ownership, and receipt semantics exactly.
  - Preserve the same worker messaging, lifecycle, and steering-policy semantics across runtime families.
  - Keep support-posture/reporting truth explicit and earned by validation.
  - Run focused shell tests plus workspace fmt/clippy before closing the slice.

- Ask first:
  - Any widening of `unified-agent-api` or gateway adapter request surfaces.
  - Any decision to keep `claude_code` on a degraded support posture after the parity path is implemented.
  - Any expansion beyond second-family parity into broader multi-family docs/smoke or public toolbox verbs.

- Never:
  - Hard-code `codex` as a fallback for selected `claude_code` sessions.
  - Reopen MCP as the required source of truth for v1.
  - Change Slice `52` receipt semantics for one runtime family only.
  - Reopen public toolbox CLI execution in this slice.

## Success Criteria

1. The repo still resolves available orchestrator candidates dynamically from inventory roots under `$SUBSTRATE_HOME` and workspace `.substrate`.
2. The selected host orchestrator is still dictated by effective config/policy/inventory truth rather than by hard-coded runtime-family preference.
3. Selected `claude_code` host starts and turns now use the same live host-tool surface architecture already proven for the first runtime family.
4. Live `claude_code` tool invocations preserve Slice `52` runtime-owned field injection and receipt/follow-up semantics.
5. Worker messaging, worker lifecycle, and steering policy semantics remain shared and unchanged across runtime families.
6. Operator/reporting surfaces no longer overstate `claude_code` as “not yet parity” once validation proves otherwise, and they do not overclaim more than the repo can actually prove.
7. No public toolbox execution verbs, MCP-first redesign, transport redesign, or hidden runtime fallback lands implicitly.

## Open Questions

1. Can `claude_code` parity be achieved entirely through the existing env-injection plus prompt-contract composition path, or does it require a bounded Claude-specific adapter/runtime request tweak?
2. What is the minimum validation wall required to upgrade `claude_code` from `not_yet_smoke_validated` / `not_yet_guaranteed` to a stronger posture:
   - targeted regression tests only,
   - targeted tests plus a bounded manual smoke,
   - or both?
3. Should Slice `54` also perform the bounded docs/tracker cleanup in `AGENT_ORCHESTRATION_GAP_MATRIX.md` for stale absolute links while touching parity-reporting truth, or leave that to a later docs-only follow-up?
