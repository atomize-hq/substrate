# Note: Family-1 Ordering After Dispatch Bootstrap

Date: `2026-05-30`

Validated against live code in:

- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
- [`docs/USAGE.md`](../docs/USAGE.md)
- [PLAN-32.md](./PLAN-32.md)
- [SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md](./SPEC-32-internal-host-orchestrator-world-dispatch-bootstrap.md)

## Purpose

Record the current repo truth after Slice `33`, and make explicit why fuller host-to-world steering-policy hardening now comes next.

## Current Repo Truth

### 1. Slice `32` is no longer design-only

The current tree now has real family-1 bootstrap code:

1. typed internal world-dispatch contract scaffolding exists,
2. authoritative internal orchestrator caller validation exists,
3. Linux `run_world_task` routing exists over the world-member dispatch seam,
4. `spawn_world_worker` now returns authoritative retained-worker bootstrap identity through the real runtime path.

### 2. Slice `32` now reaches the retained-worker bootstrap floor

The current tree now shows:

1. `spawn_world_worker` typed contract scaffolding exists,
2. live runtime dispatch launches retained worker bootstrap and returns authoritative receipt data,
3. the REPL-owned orchestrator runtime can materialize the same retained bootstrap through the internal dispatch surface.

Repo-truth implication:

1. planning the next slice is valid,
2. implementation of the next slice no longer depends on a missing retained-worker allocation prerequisite.

### 3. Exact retained follow-up primitives already exist below the missing family-1 seam

The current tree already supports:

1. exact retained member-turn submission through the existing runtime seam,
2. exact `(orchestration_session_id, backend_id, world_id, world_generation)` validation for retained member follow-up,
3. exact public retained-member follow-up on the narrow human/operator caller surface.

Repo-truth implication:

1. the next family-1 gap is not basic retained turn transport,
2. the gap is the internal orchestrator-facing retained-worker continue and typed event contract.

### 4. Slice `33` landed the narrow retained-worker continue and event bootstrap

The current tree now has:

1. `continue_world_worker` in the internal dispatch action set,
2. typed retained-worker message/event protocol shapes for the first in-scope subset,
3. exact retained-worker thread/attention contract semantics for `reply`, `progress_update`, `follow_up_question`, `blocked`, `result`, and `failure`,
4. fail-closed denials for deferred approval, fork, and control-directive worker-event classes.

The current tree still does not have:

1. fuller steering/action hardening for retained follow-up verbs,
2. inspect/cancel/stop/fork retained-worker verbs,
3. approval or fork autonomy event classes as accepted Slice `33` behavior.

### 5. Post-32 observability truth-sync does not change the ordering decision

The current tree and docs now distinguish:

1. `spawn_world_worker` bootstrap visibility through authoritative retained-worker state plus the existing `registered` runtime event path,
2. `run_world_task` terminal-only reduction rather than first-class dedicated internal dispatch trace publication.

Repo-truth implication:

1. this observability clarification does not reorder family-1 work,
2. the next missing seam is now steering-policy hardening against the concrete continue/event surface that Slice `33` landed.

## Ordering Decision

The next narrow family-1 slice should be:

1. fuller host-to-world steering-policy hardening first,
2. later verb expansion such as inspect/cancel/stop/fork only after that hardening exists.

Why:

1. the repo now has concrete exact-identity, retained turn-routing, and minimal typed event primitives to reuse,
2. the next missing seam is no longer contract bootstrap but policy hardening,
3. the policy matrix can now harden against a real continue/event surface and real denial buckets instead of a hypothetical future contract.

## Blocking Rule

The retained-worker allocation and first continue/event bootstrap prerequisites are satisfied on the current tree. Reopen this note only if a later regression removes the real `spawn_world_worker` runtime path, breaks the exact retained member-turn seam that Slice `33` reuses, or regresses the narrow typed continue/event contract that has now landed.
