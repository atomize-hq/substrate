# Note: Family-1 Ordering After Cancel Closeout

Date: `2026-06-02`

Validated against live code in:

- [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
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

Record the current repo truth after Slice `37`, and make explicit why the next honest Family-1 implementation-bearing slice is now `fork_world_worker`, while active-ephemeral inspect/cancel widening, broader fork autonomy, and Family-2 router/attach execution remain later work.

## Current Repo Truth

### 1. The live internal dispatch vocabulary now lands six verbs, not seven

The current tree now has:

1. `run_world_task`,
2. `spawn_world_worker`,
3. `continue_world_worker`,
4. `inspect_world_worker`,
5. `cancel_world_work`,
6. `stop_world_worker`

as typed internal dispatch actions and routed internal control-plane surfaces.

Repo-truth implication:

1. Slices `32` through `37` collectively froze six real Family-1 verbs,
2. the only design-stack verb still missing from live code is `fork_world_worker`,
3. the next Family-1 question is no longer “which later verb lands first between stop/cancel/fork.”

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

### 3. `fork_world_worker` is absent across the live contract, router, and policy parser

The current tree still lacks:

1. `ForkWorldWorker` in `WorldDispatchActionV1`,
2. any `fork_world_worker` route in the orchestrator dispatch match,
3. policy-model allowlist validation that recognizes `fork_world_worker`,
4. a typed fork payload/outcome contract.

Repo-truth implication:

1. `fork_world_worker` is now the only missing action in the Family-1 design verb set,
2. the repo is no longer blocked on “more cancel” before it can honestly plan the missing fork verb,
3. the post-Slice-37 grounding note should shift from cancel widening-first rhetoric to missing-verb-first rhetoric.

### 4. The spawn/bootstrap seam already exposes the narrow implementation thread for fork

The live runtime already has:

1. retained worker bootstrap through `spawn_world_worker`,
2. `SpawnWorldWorkerOutcomeV1` fields for `parent_participant_id` and `resumed_from_participant_id`,
3. transport/bootstrap requests that already carry those lineage-adjacent fields,
4. current direct spawn behavior that leaves those fields `None`.

Repo-truth implication:

1. the tree already has a natural child-allocation seam for fork,
2. Slice `38` can stay narrow by reusing retained spawn/bootstrap truth instead of inventing a second retained allocation plane,
3. the missing work is explicit source-to-child lineage and fork-specific routing, not basic retained-worker creation.

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

1. `fork_world_worker` as the next missing verb slice,
2. broader worker-requested fork autonomy and approval policy as later work,
3. active-ephemeral inspect/cancel widening as separate later identity-model work when exact task identity becomes runtime truth,
4. Family-2 router/attach execution only after Family-1 control-plane semantics are frozen enough for downstream consumers.

## Why `fork_world_worker` Is Now The Next Honest Slice

1. it is the only design-stack Family-1 verb still absent from the live typed contract and policy parser,
2. the repo already froze retained-only inspect/cancel/stop semantics, so the missing-verb frontier moved forward,
3. the retained spawn/bootstrap seam already exists and already carries lineage-adjacent fields that direct spawn leaves unset,
4. host-initiated explicit fork can stay materially narrower than worker-requested autonomy by reusing existing action allowlisting and retained concurrency caps,
5. active-ephemeral inspect/cancel widening would reopen exact task identity across already-landed verbs instead of closing the remaining verb gap.

## Why Slice `38` Should Stay Narrower Than Full Fork Autonomy

Slice `38` should freeze:

1. host-initiated `fork_world_worker` only,
2. exact retained source-worker targeting only,
3. same orchestration session and same authoritative world binding only,
4. retained child allocation only in v1,
5. explicit source-to-child lineage in the typed fork outcome,
6. worker-requested fork, fork recommendations, auto-fork, and approval/fork autonomy as explicitly deferred work.

That is the smallest honest fork slice consistent with the live tree.

## Reopen Rule

Reopen this ordering note only if one of these becomes true in follow-on grounding:

1. landing `fork_world_worker` honestly requires active-ephemeral task identity first,
2. host-initiated fork cannot be isolated from worker-requested autonomy without immediate policy-schema widening,
3. the retained spawn/bootstrap seam proves unable to carry exact source-to-child lineage without a larger architecture shift.

If none of those conditions is true, Slice `38` should remain the narrow host-initiated `fork_world_worker` slice.
