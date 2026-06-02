# Note: Family-1 Ordering After Stop Closeout

Date: `2026-06-02`

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

Record the current repo truth after Slice `37`, and make explicit why retained active-turn `cancel_world_work` landed before active-ephemeral cancel widening or `fork_world_worker`.

Validated outcome after Slice `37`:

1. that ordering has now landed as planned,
2. `cancel_world_work` is now the internal retained-worker-only v1 cancel surface for active work in flight,
3. routed cancel closeout remains Linux-only in v1 and stays distinct from `stop_world_worker`,
4. active-ephemeral cancel, `fork_world_worker`, approval/fork autonomy, and Family-2 router/attach work remain deferred.

## Current Repo Truth

### 1. Slice `36` and Slice `37` are now both landed

The current tree now has:

1. `stop_world_worker` in `WorldDispatchActionV1`,
2. `cancel_world_work` in `WorldDispatchActionV1`,
3. retained-only request validation and typed stop/cancel payload/outcome shapes,
4. steering-policy allowlisting for `stop_world_worker` and `cancel_world_work` in both shell-local and broker policy validation,
5. authoritative retained-worker stop/cancel target resolution in the state store,
6. routed retained-worker stop/cancel handling in the orchestrator dispatch layer,
7. internal toolbox ingress and regression coverage for both later verbs.

Repo-truth implication:

1. Slices `36` and `37` are not planning-only anymore,
2. Family 1 now includes landed internal cancel and stop verbs in addition to run, spawn, continue, and inspect,
3. the next gap is follow-on widening work rather than missing retained-turn cancel semantics.

### 2. `cancel_world_work` is now present in the routed internal action set

The live repo now admits and routes:

1. `run_world_task`,
2. `spawn_world_worker`,
3. `continue_world_worker`,
4. `inspect_world_worker`,
5. `cancel_world_work`,
6. `stop_world_worker`.

The cancel route is now distinct runtime truth:

1. Linux v1 routes exact retained-worker cancel through the dedicated private owner cancel surface,
2. active work in flight is interrupted before authoritative cancelled-state closeout is returned,
3. non-Linux builds fail closed with `unsupported_platform_or_posture`.

Repo-truth implication:

1. cancel is no longer a missing Family-1 verb,
2. cancel closeout semantics are frozen enough to compare against stop explicitly,
3. Slice `37` landed without reusing stopped terminology or widening into a second generic control plane.

### 3. Full dual-target cancel is still not the landed Slice `37` scope

The design stack still says:

1. `cancel_world_work` may target active ephemeral work or an active retained worker turn,
2. active-ephemeral inspect/cancel requires exact runtime-owned task identity,
3. outcome design leaves room for `task_run_id`.

The current repo, however, still has:

1. `run_world_task` as a one-shot terminal outcome surface with no typed `task_run_id`,
2. no state-store resolver for active-ephemeral inspect or cancel targets,
3. retained-worker-only target resolvers for `continue_world_worker`, `inspect_world_worker`, `cancel_world_work`, and `stop_world_worker`.

Repo-truth implication:

1. the full design-level dual-target cancel surface is still not implementation-ready,
2. Slice `37` landed as retained active-turn cancel first because exact retained worker identity was already frozen while exact active-ephemeral task identity is not,
3. the current repo still explicitly defers active-ephemeral cancel rather than pretending both cancel target families are equally ready.

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

## Ordering Outcome

The landed narrow Family-1 order is now:

1. `stop_world_worker` first,
2. `cancel_world_work` next, frozen to retained active-turn cancel in v1,
3. active-ephemeral cancel widening only after the repo gains an exact task-identity surface,
4. `fork_world_worker` only after the cancel family is real,
5. broader approval/fork autonomy and then Family-2 router/attach work after Family 1 control-plane semantics are frozen.

## Why Retained-Only Cancel Was The Right Next Slice

1. stop is already landed, so the next remaining execution-affecting verb is cancel,
2. retained active-turn cancel can reuse exact retained participant identity that is already frozen by continue, inspect, and stop,
3. Slice `37` proves that explicit cancelled terminal truth can stay distinct from stopped closeout without widening into active-ephemeral identity work,
4. active-ephemeral dual-target cancel is still broader because the repo still lacks authoritative task-target resolution and a typed task-run identity,
5. fork remains later because it introduces lineage and autonomy policy on top of a now-real but still intentionally bounded cancel family.

## Why Slice `37` Must Be Narrower Than Full Dual-Target Cancel

Slice `37` should freeze:

1. `cancel_world_work` as the next slice name,
2. exact retained worker targeting as the only in-scope target family in v1,
3. active work in flight as the only valid retained-worker cancel posture,
4. cancel outcome semantics as distinct from stop closeout semantics,
5. active-ephemeral cancel as explicitly deferred follow-on work rather than hidden implied scope.

That is the smallest honest slice consistent with the current tree.

## Reopen Rule

Reopen this ordering note only if one of these becomes true in follow-on work:

1. the live repo gains an exact active-ephemeral task resolver and typed task identity that makes dual-target cancel the honest next widening step,
2. retained active-turn cancel semantics prove insufficient for future caller needs without broader lifecycle redesign,
3. fork or Family-2 work unexpectedly becomes a prerequisite for widening cancel beyond retained active-turn scope.

If none of those conditions is true, Slice `37` should remain the landed retained active-turn cancel slice and later work should stay sequenced after it.

## Follow-On Truth After Slice `37` Lands

With Slice `37` now landed:

1. active-ephemeral target identity and dual-target cancel remain a later widening step unless implementation proves otherwise,
2. `fork_world_worker` remains later because lineage and autonomy policy are still deferred,
3. approval/fork autonomy and Family-2 router/attach execution remain downstream work rather than current repo truth.
