# Spec: Internal Retained World Worker Stop Closeout

Source validation note: [NOTE-35-family-1-ordering-after-inspect-snapshot.md](./NOTE-35-family-1-ordering-after-inspect-snapshot.md)  
Related design stack:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [SPEC-35-internal-retained-world-worker-inspect-snapshot.md](./SPEC-35-internal-retained-world-worker-inspect-snapshot.md)
- [PLAN-35.md](./PLAN-35.md)
- [TASKS-35.md](./TASKS-35.md)  
Phase: `SPECIFY`  
Status: implemented on `2026-06-01`
Landed posture note: the typed stop contract, steering-policy allowlisting, exact-target stop resolution, and durable retained-worker closeout routing are landed in the repo, but retained-worker stop routing is supported only on Linux in v1; non-Linux builds fail closed with `unsupported_platform_or_posture`.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `35` is fully landed on the current tree, so `inspect_world_worker` is runtime truth alongside `run_world_task`, `spawn_world_worker`, and `continue_world_worker`.
2. The next Family-1 slice remains internal-only and orchestrator-only. It does not widen public `substrate agent ...` caller surfaces and it does not create a second stop/control plane.
3. The narrowest next execution-affecting later verb is `stop_world_worker`, not `cancel_world_work`, because stop is retained-worker-only while cancel still spans active ephemeral or active retained-turn targets.
4. This slice should reuse existing durable stop-closeout truth where possible:
   - existing `Stopping` / `Stopped` participant and session states,
   - existing stop-closeout helpers,
   - existing Unix private owner stop transport posture where relevant,
   rather than inventing a new stop model.
5. `cancel_world_work`, `fork_world_worker`, active-ephemeral inspect, approval/fork autonomy, and Family-2 router/attach behavior remain out of scope for this slice.

If any of these are wrong, correct them before implementation.

## Objective

Build the first stop-bearing Family-1 control-plane slice so the host orchestrator can issue an exact retained-worker `stop_world_worker` request through the internal dispatch contract and cause durable retained-worker closeout without relying on the public human/operator stop surface.

Primary runtime story:

1. the host orchestrator issues an internal `stop_world_worker` request against one exact retained worker,
2. Substrate validates exact orchestration-session identity, caller identity, retained worker identity, backend identity, and authoritative world binding,
3. Substrate evaluates the existing steering-policy layer before executing stop behavior,
4. Substrate performs durable retained-worker stop using existing closeout/runtime truth rather than inventing a second lifecycle model,
5. the outcome surfaces a typed stopped closeout result for the exact retained worker,
6. `cancel_world_work`, `fork_world_worker`, active-ephemeral inspect, approval/fork autonomy, and Family-2 routing work remain deferred.

Current landed runtime note:

1. exact stop request validation, policy parsing, and retained-worker target resolution are repo-wide,
2. allowed retained-worker stop routing drives durable stopped closeout on Linux in v1 through the existing private owner stop surface,
3. non-Linux builds reject retained stop routing rather than widening into public stop or `cancel_world_work` semantics.

## Frozen Direction

This slice freezes the following:

1. `stop_world_worker` is the only new control-plane verb in scope,
2. v1 stop is retained-worker-only:
   - exact `target_participant_id` is mandatory,
   - `mode=retained` is mandatory,
   - active-ephemeral cancellation stays deferred,
3. stop is a durable lifecycle closeout action, not a snapshot and not a generic cancel:
   - it should leave no further continuation path for the stopped retained worker,
   - it should move the target into truthful terminal closeout state,
4. steering-policy enforcement still happens before stop is attempted,
5. exact identity remains mandatory:
   - `orchestration_session_id`
   - `caller_participant_id`
   - exact retained `target_participant_id`
   - exact `backend_id`
   - exact authoritative `world_id`
   - exact authoritative `world_generation`
6. already-terminal retained workers must fail closed rather than re-stopping silently,
7. this slice may reuse existing stop-closeout helpers and stop transport posture, but must not widen into a new public CLI contract,
8. `cancel_world_work`, `fork_world_worker`, approval/fork event classes, and Family-2 routing behavior remain out of scope.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing Family-1 dispatch/runtime truth:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- Existing steering-policy/config surfaces:
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
- Existing stop-closeout/runtime truth to reuse rather than replace:
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
  - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
  - [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)
- Existing public/runtime behavior that must not regress:
  - [`docs/USAGE.md`](../docs/USAGE.md)
  - [`docs/TRACE.md`](../docs/TRACE.md)
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)

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
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

This slice is expected to touch these areas:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - add `stop_world_worker` action, typed stop payload, typed stop outcome, and stable request validation
- `crates/shell/src/execution/policy_model.rs`
  - widen allowed action parsing/validation so the steering-policy layer can explicitly admit `stop_world_worker`
- `crates/broker/src/policy.rs`
  - admit `stop_world_worker` in broker policy parsing so YAML policy truth matches shell-local policy validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics and validation lists aligned with the new action id
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative retained-worker stop target resolution and terminal-state rejection
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - steering enforcement and stop dispatch handling
- `crates/shell/src/execution/agent_runtime/control.rs`
  - reuse or narrow extension of existing stop-closeout helpers for internal retained-worker stop
- `crates/shell/src/repl/async_repl.rs`
  - route internal stop ingress through the same dispatch validation path if required
- `crates/shell/tests/`
  - stop request validation, identity checks, terminal-closeout behavior, and regression coverage
- `crates/broker/src/tests.rs`
  - pin broker-side allowlisting and diagnostics for the new stop action
- `docs/CONFIGURATION.md`
  - document that `stop_world_worker` is now a valid allowlisted action where relevant
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, typed contracts, fail-closed policy checks, and reuse of authoritative lifecycle helpers rather than ad hoc stop mutations.

Preferred style:

```rust
if request.action == WorldDispatchActionV1::StopWorldWorker
    && request.target_participant_id.is_none()
{
    anyhow::bail!(
        "missing_dispatch_field: stop_world_worker requires target_participant_id"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at control-plane boundaries,
2. fail closed on missing or mismatched retained-worker identity,
3. reject already-terminal retained workers explicitly,
4. keep durable stop distinct from active cancel semantics,
5. reuse existing stop-closeout helpers and terminal state truth where possible,
6. keep cancel, fork, and approval autonomy out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for `stop_world_worker` request validation:
   - exact required fields,
   - retained-only mode validity,
   - typed stop payload acceptance,
   - exact target requirement
2. unit tests for steering-policy parsing and gating:
   - `stop_world_worker` may be allowlisted explicitly,
   - deny-by-default behavior remains intact when it is not allowlisted
3. unit tests for authoritative stop target resolution:
   - same-session only,
   - same-world-binding only,
   - authoritative orchestrator caller only,
   - exact retained worker only,
   - already-terminal retained workers fail closed
4. integration tests for stop outcomes:
   - detached retained worker stop uses durable closeout truth,
   - reachable retained worker stop drives the expected closeout path,
   - stopped state is reflected in authoritative runtime snapshots,
   - repeat stop on an already-terminal worker is denied
5. regression tests proving:
   - stop does not widen into cancel semantics,
   - stop does not fork or continue work,
   - no public CLI behavior regresses,
   - no Family-2 ledger, inbox, or router widening is required

Coverage expectations:

1. every accepted stop path has exact-identity tests,
2. steering-policy denial and allowlisting remain pinned explicitly,
3. already-terminal workers fail with a stable, reviewable error path,
4. public `agent start|turn|reattach|fork|stop` behavior remains green.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - keep `stop_world_worker` as the only new control-plane verb in scope
  - require exact retained worker identity before stop execution
  - keep durable stop distinct from active cancel semantics
  - reuse existing stop-closeout truth instead of inventing a new lifecycle model
- Ask first:
  - adding `cancel_world_work` or `fork_world_worker` in the same slice
  - adding active-ephemeral stop/cancel targets
  - widening public CLI or toolbox posture
  - coupling stop to new Family-2 schema or router behavior
- Never:
  - infer targets from fuzzy worker lookup
  - silently treat already-terminal workers as successful fresh stops
  - widen into approval or fork autonomy policy
  - introduce a second stop transport or second lifecycle model just for the internal verb

## Success Criteria

This slice is complete only when all of the following are true:

1. `stop_world_worker` exists as an internal typed world-dispatch action,
2. the action requires exact retained-worker targeting and retained mode,
3. steering policy can explicitly allow or deny `stop_world_worker`,
4. allowed Linux stop requests produce durable retained-worker closeout using authoritative existing runtime truth, while non-Linux builds fail closed instead of approximating stop behavior,
5. already-terminal retained workers fail closed,
6. `cancel_world_work`, `fork_world_worker`, approval/fork autonomy, and Family-2 routing work remain deferred.

## Open Questions

1. Whether the internal stop slice should surface a dedicated stop outcome state enum immediately or reuse a narrower terminal summary/result shape in v1. Default assumption for this spec: keep the outcome typed and explicit, but avoid inventing a larger new state taxonomy unless implementation needs it.
