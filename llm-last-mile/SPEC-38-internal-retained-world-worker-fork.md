# Spec: Internal Retained World Worker Fork

Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Related design stack:
- [NOTE-36-family-1-ordering-after-stop-closeout.md](./NOTE-36-family-1-ordering-after-stop-closeout.md)
- [SPEC-37-internal-cancel-world-work.md](./SPEC-37-internal-cancel-world-work.md)
- [PLAN-37.md](./PLAN-37.md)
- [TASKS-37.md](./TASKS-37.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)  
Phase: `SPECIFY`  
Status: proposed on `2026-06-02`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `37` is fully landed on the current tree, so the live Family-1 action surface is `run_world_task`, `spawn_world_worker`, `continue_world_worker`, `inspect_world_worker`, `cancel_world_work`, and `stop_world_worker`.
2. The next honest Family-1 gap is the only design-stack verb still absent from live code and policy parsing: `fork_world_worker`.
3. Slice `38` remains internal-only and orchestrator-only. It does not widen public `substrate agent ...` caller surfaces and it does not convert deferred worker event classes into live autonomy.
4. The narrowest honest v1 fork scope is host-initiated, exact-source, retained-to-retained fork only:
   - exact retained source `target_participant_id` is mandatory,
   - the child remains in the same `orchestration_session_id`,
   - the child inherits the same authoritative `world_id` and `world_generation`,
   - the child is retained, not ephemeral, in v1.
5. This slice should reuse the existing retained spawn/bootstrap seam where possible:
   - `spawn_world_worker` bootstrap already exists,
   - spawn transport/outcome already carries `parent_participant_id` and `resumed_from_participant_id`,
   - direct spawn currently leaves those lineage-adjacent fields `None`,
   rather than inventing a second retained allocation plane.
6. Worker-requested fork, fork recommendations, auto-fork, approval autonomy, and dedicated fork policy-schema keys remain out of scope for this slice.
7. Active-ephemeral inspect/cancel widening remains separate later work because the live repo still lacks a typed exact task-identity surface for those verbs.

If any of these are wrong, correct them before implementation.

## Objective

Build the first fork-bearing Family-1 control-plane slice so the host orchestrator can issue an exact retained-source `fork_world_worker` request through the internal dispatch contract and receive a typed child-allocation outcome with explicit source-to-child lineage, without widening into worker-requested fork autonomy, active-ephemeral task identity, or Family-2 router/attach work.

Primary runtime story:

1. the host orchestrator issues an internal `fork_world_worker` request against one exact retained source worker,
2. Substrate validates exact orchestration-session identity, caller identity, retained source-worker identity, backend identity, and authoritative world binding,
3. Substrate evaluates the existing steering-policy layer before attempting fork allocation,
4. Substrate allocates one new retained child worker through the existing retained bootstrap seam,
5. the typed outcome returns explicit child identity plus exact source-to-child lineage,
6. worker-requested fork, fork recommendations, auto-fork, active-ephemeral inspect/cancel widening, approval/fork autonomy, and Family-2 routing work remain deferred.

Current repo-truth note:

1. `fork_world_worker` is absent from `WorldDispatchActionV1`, from the orchestrator dispatch router, and from policy-model action validation,
2. `inspect_world_worker` and `cancel_world_work` still reject `mode=ephemeral`, so active-ephemeral widening is not yet live contract truth,
3. `RunWorldTaskOutcomeV1` still exposes no typed `task_run_id`, which keeps active-ephemeral exact targeting out of runtime truth,
4. retained spawn/bootstrap already exists and already carries lineage-adjacent `parent_participant_id` / `resumed_from_participant_id` fields that direct spawn leaves unset,
5. deferred worker event classes still include `fork_request`, `fork_recommendation`, `fork_command`, `approval_request`, and `approval_response`, which keeps broader fork autonomy and approval flows out of Slice `38`.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing Family-1 dispatch/runtime truth:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
- Existing retained-worker bootstrap seam expected to be reused:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Existing steering-policy/config surfaces:
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)
- Existing authoritative runtime/session state expected to own exact source-target resolution and child registration truth:
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/session.rs`](../crates/shell/src/execution/agent_runtime/session.rs)
  - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
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
  - add `fork_world_worker` action, retained-only validation, typed fork payload, and typed fork outcome with explicit source/child lineage
- `crates/shell/src/execution/policy_model.rs`
  - widen allowed action parsing/validation so the steering-policy layer can explicitly admit `fork_world_worker`
- `crates/broker/src/policy.rs`
  - admit `fork_world_worker` in broker policy parsing so YAML policy truth matches shell-local validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics and validation lists aligned with the new action id
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative retained source-worker resolution, non-terminal eligibility checks, and source-to-child lineage registration truth
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - steering enforcement, fork dispatch handling, and reuse of retained bootstrap routing
- `crates/shell/src/repl/async_repl.rs`
  - route internal fork ingress through the same dispatch validation path if required
- `crates/shell/tests/`
  - fork request validation, exact source-target checks, child-lineage outcomes, and regression coverage
- `docs/CONFIGURATION.md`
  - document that `fork_world_worker` is a valid allowlisted action where relevant
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, typed contracts, fail-closed policy checks, and reuse of authoritative retained bootstrap truth rather than inventing a second child-allocation plane.

Preferred style:

```rust
if request.action == WorldDispatchActionV1::ForkWorldWorker
    && request.target_participant_id.is_none()
{
    anyhow::bail!(
        "missing_dispatch_field: fork_world_worker requires target_participant_id"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at control-plane boundaries,
2. fail closed on missing, mismatched, invalidated, or terminal retained source-worker identity,
3. keep fork distinct from spawn and continue:
   - direct `spawn_world_worker` has no source worker,
   - `fork_world_worker` must surface explicit source-to-child lineage,
4. keep v1 fork host-initiated and retained-to-retained only,
5. reuse existing retained bootstrap truth where possible, but make lineage explicit rather than inferred,
6. keep worker-requested fork, approval autonomy, active-ephemeral widening, and Family-2 work out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for `fork_world_worker` request validation:
   - exact required fields,
   - retained-only mode validity,
   - exact source `target_participant_id`,
   - typed fork payload acceptance,
   - typed fork outcome with explicit source and child identities
2. unit tests for steering-policy parsing and gating:
   - `fork_world_worker` may be allowlisted explicitly,
   - deny-by-default behavior remains intact when it is not allowlisted
3. REPL-facing internal ingress/gating tests:
   - well-formed fork requests reach the Packet 1 unsupported-dispatch stub only after shared dispatch validation succeeds,
   - malformed fork requests fail contract validation before the Packet 1 unsupported-dispatch stub,
   - denied fork requests fail at steering-policy gating before the Packet 1 unsupported-dispatch stub
4. unit tests for authoritative retained source-target resolution:
   - same-session only,
   - same-world-binding only,
   - authoritative orchestrator caller only,
   - exact retained source worker only,
   - invalidated or terminal source workers fail closed
5. integration tests for child allocation and lineage:
   - allowed fork requests allocate one new retained child,
   - the child stays in the same authoritative session/world binding,
   - the outcome surfaces source identity plus child identity explicitly,
   - retained worker caps remain enforced
6. regression tests proving:
   - fork does not widen into worker-requested autonomy,
   - fork does not silently become plain spawn without lineage,
   - active-ephemeral inspect/cancel still remains deferred,
   - no Family-2 ledger, inbox, or router widening is required

Coverage expectations:

1. every accepted fork path has exact-identity tests,
2. steering-policy denial and allowlisting remain pinned explicitly,
3. source-to-child lineage is explicit and reviewable,
4. broader fork autonomy remains deferred in tests and summaries,
5. public `agent start|turn|reattach|fork|stop` behavior remains green.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - keep `fork_world_worker` as the only new control-plane verb in scope
  - keep v1 fork host-initiated and retained-to-retained only
  - require exact retained source-worker identity before child allocation
  - reuse existing retained bootstrap truth instead of inventing a second child-allocation plane
  - keep explicit source-to-child lineage in the typed outcome
- Ask first:
  - adding worker-requested fork, fork recommendations, or auto-fork in the same slice
  - adding active-ephemeral child mode in the same slice
  - introducing dedicated fork policy-schema keys in the same slice
  - widening public CLI or toolbox posture
  - coupling fork to new Family-2 schema or router behavior
- Never:
  - infer the source worker from fuzzy lookup
  - silently treat fork as plain spawn with no lineage
  - silently widen into approval/fork autonomy
  - widen inspect/cancel into active-ephemeral task identity as part of this slice

## Success Criteria

This slice is complete only when all of the following are true:

1. `fork_world_worker` exists as an internal typed world-dispatch action,
2. the action requires exact retained source-worker targeting and retained mode in v1,
3. steering policy can explicitly allow or deny `fork_world_worker`,
4. allowed Linux fork requests allocate one retained child through the existing retained bootstrap seam and return a typed fork outcome with explicit source-to-child lineage,
5. the child stays in the same authoritative `orchestration_session_id`, `world_id`, and `world_generation`,
6. worker-requested fork, fork recommendations, auto-fork, active-ephemeral task identity widening, approval/fork autonomy, and Family-2 routing work remain deferred.

## Open Questions

1. Whether the typed fork outcome should expose a dedicated `forked_from_participant_id` field immediately, or whether `parent_participant_id` is sufficient if the semantics are made explicit and stable in v1. Default assumption for this spec: use an explicit source-to-child lineage field if implementation can do so narrowly; do not rely on ambiguous spawn terminology if it weakens audit clarity.
