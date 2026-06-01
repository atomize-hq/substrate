# Note: Family-1 Ordering After Inspect Snapshot

Date: `2026-06-01`

Validated against live code in:

- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
- [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
- [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [SPEC-35-internal-retained-world-worker-inspect-snapshot.md](./SPEC-35-internal-retained-world-worker-inspect-snapshot.md)
- [PLAN-35.md](./PLAN-35.md)
- [TASKS-35.md](./TASKS-35.md)

## Purpose

Record the current repo truth after Slice `35`, and make explicit why the next implementation-bearing Family-1 work should be `stop_world_worker` before `cancel_world_work` or `fork_world_worker`.

Validated outcome after Slice `36`:

1. that ordering has now landed as planned,
2. `stop_world_worker` is now the internal retained-worker-only v1 stop surface,
3. routed stop closeout remains Linux-only in v1 and stays distinct from `cancel_world_work`.

## Current Repo Truth

### 1. `inspect_world_worker` is now real Family-1 runtime truth

The current tree now has:

1. `inspect_world_worker` in the internal dispatch action vocabulary,
2. retained-only request validation with exact `target_participant_id`,
3. steering-policy allowlisting for `inspect_world_worker`,
4. authoritative retained-worker inspect target resolution,
5. Linux-routed retained-worker snapshot outcomes that stay read-only and fail closed on non-Linux builds.

Repo-truth implication:

1. Family 1 now has a fourth landed verb,
2. later verb expansion is no longer blocked on missing status/snapshot visibility,
3. the next remaining gap is now an execution-affecting control verb rather than another read-only verb.

### 2. The current repo already contains durable stop-closeout building blocks

The tree already has:

1. `apply_runtime_stop_closeout` and `persist_runtime_stop_closeout` for durable stopped-terminal closeout,
2. public `run_stop` flow that splits between detached closeout persistence and live-owner private stop transport,
3. session/participant states that already model `Stopping` and `Stopped`,
4. public stop target resolution and capability narrowing through `session_stop`.

Repo-truth implication:

1. the repo is not missing the concept of durable stop,
2. the missing seam is making that closeout available through the internal host-orchestrator to world control plane with exact retained worker identity,
3. the next slice can reuse existing stop-closeout truth instead of inventing a new lifecycle model.

### 3. `cancel_world_work` is still broader than `stop_world_worker`

The design stack still says:

1. `cancel_world_work` may target active ephemeral work or an active retained worker turn,
2. cancellation is primarily about stopping active work in flight,
3. stop is an explicit durable closeout for retained workers.

Repo-truth implication:

1. `cancel_world_work` needs a broader exact target surface than retained-worker stop,
2. cancellation would have to speak to active ephemeral identity that Family 1 still has not frozen,
3. `stop_world_worker` remains the narrower execution-affecting next slice.

### 4. `fork_world_worker` remains the sharpest policy edge

The design stack still requires:

1. lineage recording,
2. fork depth and child-count policy,
3. worker-requested fork autonomy decisions.

Repo-truth implication:

1. fork is still not an honest “smallest next slice,”
2. the repo should keep fork after stop/cancel,
3. approval/fork autonomy remains later than the next verb slice.

## Ordering Decision

The next narrow Family-1 slice should be:

1. `stop_world_worker` first,
2. `cancel_world_work` after that,
3. `fork_world_worker` only after the execution-affecting stop/cancel family is real,
4. broader approval/fork autonomy after the later verb family is in place.

## Why `stop_world_worker` Next

1. it is retained-worker-only in the design contract,
2. it reuses exact retained participant identity rather than needing active ephemeral task identity,
3. it can build directly on the repo’s existing stop-closeout helpers and public stop transport posture,
4. it freezes durable closeout semantics without yet taking on dual-target cancellation semantics,
5. it is materially narrower than fork because it avoids lineage and autonomy policy.

## Blocking Rule

Reopen this ordering note only if one of these becomes true:

1. the landed inspect slice proves retained-worker stop cannot honestly reuse the existing exact target and closeout truth,
2. implementing `stop_world_worker` turns out to require active-ephemeral task identity first,
3. stop semantics force an immediate broader redesign of Family-2 review/attach or fork autonomy policy.

If none of those conditions is true, the next slice should remain `stop_world_worker`.

## Follow-On Truth After Slice 36

With Slice `36` now landed:

1. `cancel_world_work` remains the next broader execution-affecting Family-1 verb,
2. `fork_world_worker` remains later because lineage and autonomy policy are still deferred,
3. approval/fork autonomy and Family-2 router/attach execution remain downstream work rather than landed repo truth.
