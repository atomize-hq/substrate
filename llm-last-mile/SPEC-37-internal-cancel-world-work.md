# Spec: Internal Cancel World Work

Source validation note: [NOTE-36-family-1-ordering-after-stop-closeout.md](./NOTE-36-family-1-ordering-after-stop-closeout.md)  
Related design stack:
- [NOTE-35-family-1-ordering-after-inspect-snapshot.md](./NOTE-35-family-1-ordering-after-inspect-snapshot.md)
- [SPEC-36-internal-retained-world-worker-stop-closeout.md](./SPEC-36-internal-retained-world-worker-stop-closeout.md)
- [PLAN-36.md](./PLAN-36.md)
- [TASKS-36.md](./TASKS-36.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)  
Phase: `SPECIFY`  
Status: drafted on `2026-06-01`  
Planned posture note: the repo has landed internal retained-worker stop closeout, but it still lacks a cancel action, a retained-worker cancelled terminal state, and an exact active-ephemeral task-target surface. Slice `37` therefore freezes `cancel_world_work` as retained active-turn cancel first in v1.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `36` is fully landed on the current tree, so `stop_world_worker` is runtime truth alongside `run_world_task`, `spawn_world_worker`, `continue_world_worker`, and `inspect_world_worker`.
2. The next Family-1 slice remains internal-only and orchestrator-only. It does not widen public `substrate agent ...` surfaces and it does not turn toolbox into a second execution plane.
3. `cancel_world_work` remains the next honest Family-1 slice, but the smallest honest v1 scope is retained active-turn cancel first rather than full dual-target cancel.
4. Active-ephemeral cancel stays deferred in this slice because the live repo still lacks a typed `task_run_id` style surface and authoritative state-store target resolution for active ephemeral work.
5. This slice should introduce cancel semantics that stay distinct from stop semantics:
   - cancel targets active retained work in flight,
   - cancel must not reuse `stopped` terminology or stop closeout outcomes,
   - cancel should default to explicit cancelled terminal truth for the retained worker/session records rather than silently mapping to `stopped` or `failed`.
6. The safest implementation seam for Slice `37` is a dedicated cancel-only internal transport for retained turns rather than a shared generic internal control bus.
7. `fork_world_worker`, approval/fork autonomy, and Family-2 router/attach execution remain out of scope for this slice.

If any of these are wrong, correct them before implementation.

## Objective

Build the first cancel-bearing Family-1 control-plane slice so the host orchestrator can issue an internal `cancel_world_work` request against one exact retained worker that currently has active work in flight and receive a typed cancel outcome that is distinct from stop closeout.

Primary runtime story:

1. the host orchestrator issues an internal `cancel_world_work` request against one exact retained worker,
2. Substrate validates exact orchestration-session identity, caller identity, retained worker identity, backend identity, authoritative world binding, and active cancel eligibility,
3. Substrate evaluates the existing steering-policy layer before attempting cancellation,
4. Substrate interrupts only active retained work in flight and persists explicit cancelled terminal truth distinct from stopped closeout,
5. the outcome surfaces a typed cancel result for the exact retained worker,
6. active-ephemeral cancel, `fork_world_worker`, approval/fork autonomy, and Family-2 routing work remain deferred.

Current repo-truth note:

1. exact stop request validation, policy parsing, retained-worker target resolution, and stop closeout routing are already landed,
2. exact active-ephemeral inspect/cancel identity is still not landed,
3. retained runtime/session enums do not yet model a cancelled terminal state, so Slice `37` must freeze that runtime truth instead of assuming it already exists,
4. the larger design docs freeze cancel semantics and policy boundaries, but they do not yet force a shared retained-control transport abstraction for implementation.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing Family-1 dispatch/runtime truth:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- Existing runtime control and closeout surfaces that cancel must stay distinct from:
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
- Existing lifecycle state surfaces that do not yet expose retained `cancelled` truth:
  - [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)
  - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- Existing steering-policy/config surfaces:
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)
- Existing internal ingress surface:
  - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)

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

Existing targeted validation floor:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - add `cancel_world_work` action, typed cancel payload, typed cancel outcome, and retained-only request validation
- `crates/shell/src/execution/policy_model.rs`
  - widen allowed action parsing/validation so the steering-policy layer can explicitly admit `cancel_world_work`
- `crates/broker/src/policy.rs`
  - admit `cancel_world_work` in broker policy parsing so YAML policy truth matches shell-local validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics and validation lists aligned with the new action id
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative retained-worker cancel target resolution and cancel-eligibility checks
- `crates/shell/src/execution/agent_runtime/session.rs`
  - add retained runtime cancelled-state truth distinct from stopped/failed if the implementation keeps terminal cancel state at the participant layer
- `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
  - add orchestration-session cancelled-state truth distinct from stopped/failed if session state mirrors the authoritative retained worker result
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - steering enforcement, cancel dispatch handling, and typed cancel outcome routing
- `crates/shell/src/execution/agent_runtime/control.rs`
  - add the retained-turn cancel control seam and cancel-closeout persistence through a dedicated cancel-only internal transport without reusing stop transport semantics
- `crates/shell/src/repl/async_repl.rs`
  - route internal cancel ingress through the same dispatch validation path and clean up retained member runtime handles when cancellation goes terminal
- `crates/shell/tests/`
  - cancel request validation, target-eligibility checks, cancel routing, and regression coverage
- `crates/broker/src/tests.rs`
  - pin broker-side allowlisting and diagnostics for the new cancel action
- `docs/CONFIGURATION.md`
  - document that `cancel_world_work` is a valid allowlisted action where relevant
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, typed contracts, fail-closed policy checks, and explicit lifecycle vocabulary. Cancel must stay semantically distinct from stop.

Preferred style:

```rust
if request.action == WorldDispatchActionV1::CancelWorldWork
    && request.target_participant_id.is_none()
{
    anyhow::bail!(
        "missing_dispatch_field: cancel_world_work requires target_participant_id"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at control-plane boundaries,
2. fail closed on missing, mismatched, or non-live retained-worker identity,
3. reject idle, parked, stopped, failed, and invalidated retained workers for v1 cancel,
4. keep cancel distinct from stop:
   - do not return stopped closeout types,
   - do not persist `stopped` as the terminal reason,
5. prefer a dedicated cancel-only internal transport over a shared generic control bus for this slice,
6. keep active-ephemeral cancel, fork, and approval autonomy out of this slice,
7. reuse existing authoritative runtime/session helpers where possible, but only if they preserve explicit cancelled semantics.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for `cancel_world_work` request validation:
   - exact required fields,
   - retained-only mode validity,
   - typed cancel payload acceptance,
   - exact target requirement
2. unit tests for steering-policy parsing and gating:
   - `cancel_world_work` may be allowlisted explicitly,
   - deny-by-default behavior remains intact when it is not allowlisted
3. unit tests for authoritative retained-worker cancel target resolution:
   - same-session only,
   - same-world-binding only,
   - authoritative orchestrator caller only,
   - exact retained worker only,
   - target must still be actively cancelable,
   - parked, stopped, failed, invalidated, or otherwise non-cancelable workers fail closed
4. lifecycle tests for cancel-closeout truth:
   - cancelled state is distinct from stopped state,
   - retained session/participant records surface explicit cancelled terminal truth,
   - repeat cancel against already-terminal workers is denied
5. integration tests for routed cancel outcomes:
   - allowed cancel requests interrupt active retained work in flight,
   - runtime cleanup and stored state converge on the typed cancel result,
   - cancel does not widen into stop, continue, inspect, or fork behavior
6. regression tests proving:
   - active-ephemeral cancel is still rejected or unsupported in Slice `37`,
   - public CLI behavior does not regress,
   - no Family-2 ledger, inbox, or router widening is required

Coverage expectations:

1. every accepted cancel path has exact-identity tests,
2. steering-policy denial and allowlisting remain pinned explicitly,
3. cancelled terminal truth is explicit and reviewable,
4. stop and cancel outcomes remain disambiguated in tests and summaries,
5. public `agent start|turn|reattach|fork|stop` behavior remains green.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - keep `cancel_world_work` as the only new control-plane verb in scope
  - keep v1 cancel retained-worker-only and active-work-only
  - implement retained-turn cancel with a dedicated cancel-only internal transport in this slice
  - require exact retained worker identity before cancel execution
  - keep cancel semantics distinct from stop semantics
  - defer active-ephemeral cancel until exact task identity exists in repo truth
- Ask first:
  - adding active-ephemeral cancel in the same slice
  - adding `fork_world_worker` or approval/fork autonomy in the same slice
  - replacing the dedicated cancel transport with a shared generic retained-control bus
  - widening public CLI or toolbox posture
  - coupling cancel to new Family-2 schema or router behavior
- Never:
  - infer targets from fuzzy worker lookup
  - silently map cancel to stop
  - silently treat already-terminal or idle retained workers as successful cancellations
  - introduce a second generic execution plane just to carry cancel

## Success Criteria

This slice is complete only when all of the following are true:

1. `cancel_world_work` exists as an internal typed world-dispatch action,
2. the action requires exact retained-worker targeting and retained mode in v1,
3. steering policy can explicitly allow or deny `cancel_world_work`,
4. allowed Linux cancel requests interrupt active retained work in flight and return a typed cancel outcome that is distinct from stop closeout,
5. authoritative runtime/session truth exposes cancelled terminal state explicitly rather than reusing stopped or failed state,
6. idle or already-terminal retained workers fail closed,
7. active-ephemeral cancel, `fork_world_worker`, approval/fork autonomy, and Family-2 routing work remain deferred.

## Open Questions

1. Whether the dedicated cancel transport should mirror the stop transport’s request/response shape closely for implementation symmetry, or intentionally use a distinct payload shape immediately to make cancel/stop divergence more obvious in code review.
