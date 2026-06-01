# Spec: Internal Retained World Worker Inspect Snapshot

Source validation note: [REMAINING-family-1-scope-2026-05-31-post-slice-34.md](./REMAINING-family-1-scope-2026-05-31-post-slice-34.md)  
Related design stack:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md](./NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md)
- [SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md](./SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md)
- [PLAN-34.md](./PLAN-34.md)
- [TASKS-34.md](./TASKS-34.md)  
Phase: `SPECIFY`  
Status: implemented on `2026-06-01`  
Landed posture note: the typed inspect contract, steering-policy allowlisting, and ingress validation are landed in the repo, but retained-worker inspect snapshot routing is supported only on Linux in v1; non-Linux builds fail closed with `unsupported_platform_or_posture`.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `34` is already landed on the current tree, so `run_world_task`, `spawn_world_worker`, and `continue_world_worker` plus first steering-policy hardening are runtime truth rather than planned-only surfaces.
2. The next Family-1 slice remains internal-only and orchestrator-only. It does not widen public `substrate agent ...` caller surfaces and it does not turn toolbox into a new execution plane.
3. The smallest honest next verb is `inspect_world_worker`, and the smallest honest form of that verb is retained-worker inspection first rather than active-ephemeral task inspection.
4. This slice should reuse authoritative state-store truth and existing session/participant snapshots rather than adding a new live world-side RPC or transport path.
5. `cancel_world_work`, `stop_world_worker`, `fork_world_worker`, approval-request policy, fork autonomy, and Family-2 router/attach behavior remain out of scope for this slice.

If any of these are wrong, correct them before implementation.

## Objective

Build the first inspect-bearing Family-1 control-plane slice so the host orchestrator can request an exact retained-worker status snapshot through the internal dispatch contract and receive typed authoritative lifecycle truth without relying on public human/operator status surfaces.

Primary runtime story:

1. the host orchestrator issues an internal `inspect_world_worker` request against one exact retained worker,
2. Substrate validates exact orchestration-session identity, caller identity, retained worker identity, backend identity, and authoritative world binding,
3. Substrate evaluates the existing steering-policy layer before serving the snapshot,
4. Substrate returns a typed inspect outcome built from authoritative persisted runtime truth,
5. the outcome is snapshot-only and does not continue, cancel, stop, or fork the worker,
6. active-ephemeral inspect, cancellation, stopping, forking, approval/fork autonomy, and Family-2 routing work remain deferred.

Current landed runtime note:

1. exact inspect request validation and policy parsing are repo-wide,
2. authoritative retained-worker snapshot routing returns a real inspect outcome on Linux in v1,
3. non-Linux builds reject retained inspect routing rather than widening into a second transport or partial runtime behavior.

## Frozen Direction

This slice freezes the following:

1. `inspect_world_worker` is the only new control-plane verb in scope,
2. v1 inspect is retained-worker-only:
   - exact `target_participant_id` is mandatory,
   - active-ephemeral task inspection stays deferred,
3. inspect is authoritative snapshot lookup, not live runtime steering:
   - no new world-service RPC,
   - no new execution transport,
   - no new worker-side message classes,
4. steering-policy enforcement still happens before the inspect result is surfaced,
5. exact identity remains mandatory:
   - `orchestration_session_id`
   - `caller_participant_id`
   - exact retained `target_participant_id`
   - exact `backend_id`
   - exact authoritative `world_id`
   - exact authoritative `world_generation`
6. inspect may surface live, attention-pending, detached, invalidated, or terminal retained-worker truth from authoritative snapshots,
7. inspect must not mutate lifecycle state,
8. `cancel_world_work`, `stop_world_worker`, `fork_world_worker`, approval/fork event classes, and Family-2 routing behavior remain out of scope.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Existing Family-1 dispatch/runtime truth:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- Existing steering-policy/config surfaces:
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
- Existing status/snapshot truth that should be reused rather than replaced:
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
  - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
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
cargo test -p substrate-broker inspect_world_worker -- --nocapture
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
  - add `inspect_world_worker` action, typed inspect payload, typed inspect outcome, and stable request validation
- `crates/shell/src/execution/policy_model.rs`
  - widen allowed action parsing/validation so the steering-policy layer can explicitly admit `inspect_world_worker`
- `crates/broker/src/policy.rs`
  - admit `inspect_world_worker` in broker policy parsing so YAML policy truth matches shell-local policy validation
- `crates/broker/src/effective_policy.rs`
  - keep effective-policy diagnostics and validation lists aligned with the landed inspect action vocabulary
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - authoritative retained-worker inspect target resolution and snapshot projection
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - pre-routing steering enforcement and inspect dispatch handling
- `crates/shell/src/repl/async_repl.rs`
  - route internal inspect ingress through the same dispatch validation path and keep deferred ingress fail-closed
- `crates/shell/tests/`
  - inspect request validation, identity checks, snapshot projection, and regression coverage
- `crates/broker/src/tests.rs`
  - pin broker-side allowlisting and diagnostics for the landed inspect action
- `docs/CONFIGURATION.md`
  - document that `inspect_world_worker` is now a valid allowlisted action where relevant
- `llm-last-mile/`
  - this spec and the matching plan/tasks artifacts

## Code Style

Follow the existing shell/runtime style: exact identity, typed contracts, fail-closed policy checks, and authoritative snapshot reuse rather than heuristic lookups.

Preferred style:

```rust
if request.action == WorldDispatchActionV1::InspectWorldWorker
    && request.target_participant_id.is_none()
{
    anyhow::bail!(
        "missing_dispatch_field: inspect_world_worker requires target_participant_id"
    );
}
```

Conventions:

1. use `Result<T, anyhow::Error>` with `Context` at control-plane boundaries,
2. fail closed on missing or mismatched retained-worker identity,
3. build inspect results from authoritative stored session/participant truth,
4. keep snapshot behavior separate from execution-affecting actions,
5. do not infer targets from role labels, recency, or fuzzy worker selection,
6. keep active-ephemeral inspect, cancel, stop, and fork out of this slice.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- broker policy/unit tests
- existing shell regression suites

Test levels for this slice:

1. unit tests for `inspect_world_worker` request validation:
   - exact required fields,
   - retained-only mode validity,
   - typed inspect payload acceptance,
   - exact target requirement
2. unit tests for steering-policy parsing and gating:
   - `inspect_world_worker` may be allowlisted explicitly,
   - deny-by-default behavior remains intact when it is not allowlisted
3. unit tests for authoritative snapshot resolution:
   - same-session only,
   - same-world-binding only,
   - authoritative orchestrator caller only,
   - exact retained worker only
4. integration tests for inspect snapshot outcomes:
   - live retained worker snapshot,
   - awaiting-attention or detached retained worker snapshot,
   - invalidated retained worker snapshot,
   - terminal retained worker snapshot
5. regression tests proving:
   - inspect does not continue, cancel, stop, or fork work,
   - no public CLI behavior changes,
   - no Family-2 ledger, inbox, or router widening is required

Coverage expectations:

1. every accepted inspect path has exact-identity tests,
2. steering-policy denial and allowlisting remain pinned explicitly,
3. invalidated or terminal workers return truthful snapshots rather than accidental continuation behavior,
4. Linux inspect routing returns authoritative snapshots while non-Linux builds fail closed with `unsupported_platform_or_posture`,
5. public `agent start|turn|reattach|fork|stop` behavior remains green.

## Boundaries

- Always:
  - keep this slice internal-only and orchestrator-only
  - keep `inspect_world_worker` as the only new control-plane verb in scope
  - require exact retained worker identity before serving a snapshot
  - reuse authoritative stored runtime truth instead of adding live inspect RPCs
  - keep inspect snapshot-only and non-mutating
- Ask first:
  - adding active-ephemeral inspect in the same slice
  - adding `cancel_world_work`, `stop_world_worker`, or `fork_world_worker`
  - widening public CLI or toolbox posture
  - coupling inspect to new Family-2 schema or router behavior
- Never:
  - infer targets from fuzzy worker lookup
  - mutate retained lifecycle state as part of inspection
  - silently widen into approval or fork autonomy policy
  - introduce a second world execution plane just for inspect

## Success Criteria

This slice is complete only when all of the following are true:

1. `inspect_world_worker` exists as an internal typed world-dispatch action,
2. the action requires exact retained-worker targeting and retained mode,
3. steering policy can explicitly allow or deny `inspect_world_worker`,
4. allowed Linux inspect requests return an authoritative typed retained-worker snapshot from stored runtime truth,
5. inspect covers live, detached/attention, invalidated, and terminal retained-worker states without mutating them, while non-Linux builds fail closed instead of returning partial inspect behavior,
6. active-ephemeral inspect, cancel, stop, fork, approval/fork autonomy, and Family-2 routing work remain deferred.

## Open Questions

1. Whether the typed inspect payload should ship with only a default snapshot scope in v1 or also include an explicit detail level field. Default assumption for this spec: keep payload typed but narrow, and defer richer scopes unless implementation proves they are essentially free.
