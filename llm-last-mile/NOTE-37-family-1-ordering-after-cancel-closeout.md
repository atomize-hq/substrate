# Note: Family-1 Ordering After Cancel Closeout

Date: `2026-06-02`

Validated against live code in:

- [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
- [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
- [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)
- [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
- [PLAN-37.md](./PLAN-37.md)
- [TASKS-37.md](./TASKS-37.md)
- [SPEC-37-internal-cancel-world-work.md](./SPEC-37-internal-cancel-world-work.md)
- [PLAN-36.md](./PLAN-36.md)
- [SPEC-36-internal-retained-world-worker-stop-closeout.md](./SPEC-36-internal-retained-world-worker-stop-closeout.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)

## Purpose

Record the current repo truth after Slice `38`, and preserve why `fork_world_worker` was the next honest Family-1 implementation-bearing slice after cancel closeout while making explicit that the narrow host-initiated retained-to-retained fork is now landed. Broader fork autonomy, active-ephemeral inspect/cancel widening, and Family-2 router/attach execution remain later work.

Validated outcome after Slice `38`:

1. that ordering has now landed as planned,
2. `fork_world_worker` is now the internal host-initiated retained-to-retained v1 fork surface for one exact retained source worker,
3. allowed Linux requests reuse the retained bootstrap seam, preserve explicit source-to-child lineage, and keep the child in the same authoritative session and world binding,
4. worker-requested fork, fork recommendations, auto-fork, approval/fork autonomy, active-ephemeral inspect/cancel widening, and Family-2 router/attach work remain deferred.

## Current Repo Truth

### 1. The live internal dispatch vocabulary now lands seven verbs

The current tree now has:

1. `run_world_task`,
2. `spawn_world_worker`,
3. `fork_world_worker`,
4. `continue_world_worker`,
5. `inspect_world_worker`,
6. `cancel_world_work`,
7. `stop_world_worker`

as typed internal dispatch actions and routed internal control-plane surfaces.

Repo-truth implication:

1. Slices `32` through `38` collectively froze seven real Family-1 verbs,
2. the missing-verb gap is now closed for the current design-stack Family-1 vocabulary,
3. the next Family-1 question is now about later widening work rather than which remaining verb lands first.

### 2. Active-ephemeral inspect/cancel is still design truth, not runtime truth

The design stack still allows:

1. `inspect_world_worker` against active ephemeral work with exact task identity,
2. `cancel_world_work` against active ephemeral work in flight.

The live contract, however, still does all of the following:

1. accepts `inspect_world_worker` only with `mode=retained`,
2. accepts `cancel_world_work` only with `mode=retained`,
3. requires exact retained `target_participant_id` for inspect/cancel/stop,
4. returns `RunWorldTaskOutcomeV1` without any typed `task_run_id` surface.

Repo-truth implication:

1. active-ephemeral inspect/cancel widening is still missing exact task identity,
2. it is a follow-on widening of already-landed verbs rather than the next missing Family-1 verb,
3. it should not be mistaken for the smallest next slice just because the design docs leave room for it.

### 3. `fork_world_worker` is now present across the live contract, router, and policy parser

The current tree now has:

1. `ForkWorldWorker` in `WorldDispatchActionV1`,
2. routed `fork_world_worker` handling in the orchestrator dispatch match,
3. shell-local plus broker allowlist validation that recognizes `fork_world_worker`,
4. a typed fork payload/outcome contract with explicit source and child identities.

Repo-truth implication:

1. `fork_world_worker` is no longer the missing action in the Family-1 design verb set,
2. the repo is no longer blocked on fork-contract cleanup before it can honestly sequence later widening work,
3. broader fork autonomy remains a follow-on scope decision rather than a missing contract/parser gap.

### 4. The retained spawn/bootstrap seam now carries the landed narrow implementation thread for fork

The live runtime already has:

1. retained worker bootstrap through `spawn_world_worker`,
2. a routed `fork_world_worker` path that reuses that bootstrap seam on Linux in v1,
3. explicit authoritative source-to-child lineage persistence distinct from plain spawn semantics,
4. direct spawn behavior that still leaves lineage-adjacent `parent_participant_id` and `resumed_from_participant_id` fields unset when there is no source worker.

Repo-truth implication:

1. the tree used the natural child-allocation seam for fork without inventing a second retained allocation plane,
2. Slice `38` stayed narrow by reusing retained spawn/bootstrap truth instead of widening into a broader retained allocation redesign,
3. the landed fork path is reviewable because explicit source-to-child lineage is now distinct from basic retained-worker creation.

### 5. Broader fork autonomy is still explicitly deferred

The current tree still shows:

1. continued worker event classification treats `approval_request`, `approval_response`, `fork_request`, `fork_recommendation`, `fork_command`, `control_directive`, and `control_ack` as deferred wire labels,
2. the live policy model still exposes only the generic `agents.world_dispatch` gates already landed for enablement, backends, actions, modes, same-session/world-binding, capability narrowing, and concurrency caps,
3. there is still no dedicated fork-autonomy policy surface in runtime truth.

Repo-truth implication:

1. a narrow host-initiated `fork_world_worker` slice can land before broader worker-requested fork autonomy,
2. approval/fork autonomy should remain later work rather than being pulled into Slice `38`,
3. Family 2 router/attach work is still downstream of these control-plane semantics.

## Ordering Outcome

The current honest Family-1 order is now:

1. `fork_world_worker` as the now-landed missing verb slice,
2. broader worker-requested fork autonomy and approval policy as later work if still needed,
3. active-ephemeral inspect/cancel widening as separate later identity-model work when exact task identity becomes runtime truth,
4. Family-2 router/attach execution only after Family-1 control-plane semantics are frozen enough for downstream consumers.

## Why `fork_world_worker` Was The Next Honest Slice

1. it was the only design-stack Family-1 verb still absent from the live typed contract and policy parser,
2. the repo had already frozen retained-only inspect/cancel/stop semantics, so the missing-verb frontier had moved forward,
3. the retained spawn/bootstrap seam already existed and already carried lineage-adjacent fields that direct spawn leaves unset,
4. host-initiated explicit fork could stay materially narrower than worker-requested autonomy by reusing existing action allowlisting and retained concurrency caps,
5. active-ephemeral inspect/cancel widening would have reopened exact task identity across already-landed verbs instead of closing the remaining verb gap.

## Why Slice `38` Had To Stay Narrower Than Full Fork Autonomy

Slice `38` should freeze:

1. host-initiated `fork_world_worker` only,
2. exact retained source-worker targeting only,
3. same orchestration session and same authoritative world binding only,
4. retained child allocation only in v1,
5. explicit source-to-child lineage in the typed fork outcome,
6. worker-requested fork, fork recommendations, auto-fork, and approval/fork autonomy as explicitly deferred work.

That was the smallest honest fork slice consistent with the live tree, and it is now the landed Slice `38` posture.

## Reopen Rule

Reopen this ordering note only if one of these becomes true in follow-on grounding:

1. landing `fork_world_worker` honestly requires active-ephemeral task identity first,
2. host-initiated fork cannot be isolated from worker-requested autonomy without immediate policy-schema widening,
3. the retained spawn/bootstrap seam proves unable to carry exact source-to-child lineage without a larger architecture shift.

If none of those conditions is true, Slice `38` should remain the landed narrow host-initiated `fork_world_worker` slice and later work should stay sequenced after it.
