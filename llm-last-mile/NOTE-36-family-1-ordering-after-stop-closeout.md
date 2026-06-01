# Note: Family-1 Ordering After Stop Closeout

Date: `2026-06-01`

Validated against live code in:

- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
- [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
- [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
- [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)
- [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
- [NOTE-35-family-1-ordering-after-inspect-snapshot.md](./NOTE-35-family-1-ordering-after-inspect-snapshot.md)
- [SPEC-36-internal-retained-world-worker-stop-closeout.md](./SPEC-36-internal-retained-world-worker-stop-closeout.md)
- [PLAN-36.md](./PLAN-36.md)
- [TASKS-36.md](./TASKS-36.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)

## Purpose

Record the current repo truth after Slice `36`, and make explicit why the next implementation-bearing Family-1 work remains `cancel_world_work` after stop closeout lands.

Validated outcome after Slice `36`:

1. Slice `36` is actually landed on the current tree,
2. `stop_world_worker` is now real internal dispatch/runtime truth,
3. `cancel_world_work` is still absent from the internal dispatch surface,
4. the honest next slice is still `cancel_world_work`, but repo truth now narrows the smallest honest v1 scope to retained active-turn cancel first rather than full dual-target cancel.

## Current Repo Truth

### 1. Slice `36` is actually landed

The current tree now has:

1. `stop_world_worker` in `WorldDispatchActionV1`,
2. retained-only request validation and typed stop payload/outcome shapes,
3. steering-policy allowlisting for `stop_world_worker` in both shell-local and broker policy validation,
4. authoritative retained-worker stop target resolution in the state store,
5. routed retained-worker stop handling in the orchestrator dispatch layer,
6. internal toolbox ingress and regression coverage for stop routing.

Repo-truth implication:

1. Slice `36` is not planning-only anymore,
2. Family 1 now includes landed internal stop closeout in addition to run, spawn, continue, and inspect,
3. the next gap is the cancel verb family rather than more stop cleanup.

### 2. `stop_world_worker` is now present in the routed internal action set

The live repo now admits and routes:

1. `run_world_task`,
2. `spawn_world_worker`,
3. `continue_world_worker`,
4. `inspect_world_worker`,
5. `stop_world_worker`.

The stop route is now distinct runtime truth:

1. Linux v1 routes exact retained-worker stop through the existing private owner stop surface,
2. durable closeout waits for authoritative stopped-state persistence,
3. non-Linux builds fail closed with `unsupported_platform_or_posture`.

Repo-truth implication:

1. stop is no longer a candidate next slice,
2. stop closeout semantics are frozen enough to compare cancel against them explicitly,
3. Slice `37` must keep cancel distinct from stop instead of reusing stopped terminology.

### 3. `cancel_world_work` is still not landed in the internal dispatch surface

The current tree does not yet have:

1. `cancel_world_work` in `WorldDispatchActionV1`,
2. a typed cancel payload or cancel outcome shape,
3. policy validation or allowlisting for `cancel_world_work`,
4. state-store target resolution for cancel,
5. orchestrator dispatch routing for cancel,
6. internal toolbox ingress handling for cancel.

Repo-truth implication:

1. cancel remains a real missing verb,
2. the next Family-1 slice still needs to create a new internal cancel contract rather than widening existing stop behavior,
3. Slice `37` should stay named `cancel_world_work` unless the repo also disproves it as the next honest scope.

### 4. Full dual-target cancel is not yet the smallest honest Slice `37`

The design stack still says:

1. `cancel_world_work` may target active ephemeral work or an active retained worker turn,
2. active-ephemeral inspect/cancel requires exact runtime-owned task identity,
3. outcome design leaves room for `task_run_id`.

The current repo, however, still has:

1. `run_world_task` as a one-shot terminal outcome surface with no typed `task_run_id`,
2. no state-store resolver for active-ephemeral inspect or cancel targets,
3. retained-worker-only target resolvers for `continue_world_worker`, `inspect_world_worker`, and `stop_world_worker`,
4. retained runtime/session enums that still model `Stopping` and `Stopped`, but not retained-worker `Cancelled` terminal truth.

Repo-truth implication:

1. the full design-level dual-target cancel surface is not yet implementation-ready,
2. the smallest honest Slice `37` is retained active-turn cancel first, because exact retained worker identity is already frozen while exact active-ephemeral task identity is not,
3. Slice `37` must explicitly defer active-ephemeral cancel rather than pretending both cancel target families are equally ready.

### 5. `fork_world_worker` remains later and not the smallest honest next slice

The current tree still lacks:

1. `fork_world_worker` in the internal dispatch action vocabulary,
2. fork payload/outcome routing,
3. lineage-aware fork state and allocation behavior in the internal world dispatch path,
4. worker-requested fork autonomy policy.

The design stack still requires:

1. lineage recording,
2. source-to-child identity rules,
3. fork depth and concurrency policy,
4. later approval/autonomy decisions.

Repo-truth implication:

1. fork is still later than cancel,
2. Slice `37` should not widen into fork or approval/fork autonomy work,
3. Family 2 router/attach work remains downstream of these remaining Family-1 control-plane semantics.

## Ordering Decision

The next narrow Family-1 slice should now be:

1. `cancel_world_work` first, but frozen to retained active-turn cancel in v1,
2. active-ephemeral cancel widening only after the repo gains an exact task-identity surface,
3. `fork_world_worker` only after the cancel family is real,
4. broader approval/fork autonomy after the later verb family is in place,
5. Family-2 router/attach work after Family 1 control-plane semantics are frozen.

## Why `cancel_world_work` Next

1. stop is already landed, so the next remaining execution-affecting verb is cancel,
2. retained active-turn cancel can reuse exact retained participant identity that is already frozen by continue, inspect, and stop,
3. active-ephemeral dual-target cancel is broader because the repo still lacks authoritative task-target resolution and a typed task-run identity,
4. fork remains later because it introduces lineage and autonomy policy on top of a still-incomplete cancel family,
5. Family 2 remains later because it depends on the remaining Family-1 verb semantics being frozen first.

## Why Slice `37` Must Be Narrower Than Full Dual-Target Cancel

Slice `37` should freeze:

1. `cancel_world_work` as the next slice name,
2. exact retained worker targeting as the only in-scope target family in v1,
3. active work in flight as the only valid retained-worker cancel posture,
4. cancel outcome semantics as distinct from stop closeout semantics,
5. active-ephemeral cancel as explicitly deferred follow-on work rather than hidden implied scope.

That is the smallest honest slice consistent with the current tree.

## Blocking Rule

Reopen this ordering note only if one of these becomes true:

1. the live repo gains an exact active-ephemeral task resolver and typed task identity before Slice `37` implementation begins,
2. retained active-turn cancel proves impossible without landing active-ephemeral cancel in the same slice,
3. cancel semantics cannot be expressed distinctly from stop without widening into broader lifecycle or public-surface redesign,
4. fork or Family-2 work unexpectedly becomes a prerequisite for cancel routing.

If none of those conditions is true, Slice `37` should remain `cancel_world_work` with retained active-turn scope first.

## Follow-On Truth After Slice `37` Planning

With Slice `36` landed and Slice `37` now framed honestly:

1. `cancel_world_work` remains the next implementation-bearing Family-1 slice,
2. active-ephemeral target identity and dual-target cancel remain a later widening step unless implementation proves otherwise,
3. `fork_world_worker` remains later because lineage and autonomy policy are still deferred,
4. approval/fork autonomy and Family-2 router/attach execution remain downstream work rather than current repo truth.
