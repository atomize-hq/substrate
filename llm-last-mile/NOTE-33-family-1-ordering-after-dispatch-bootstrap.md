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

Record the current repo truth for the next family-1 slice after `32`, and make explicit why retained-worker continue/messaging still comes before fuller host-to-world steering-policy hardening.

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

### 4. Retained-worker messaging and steering remain genuinely unimplemented

The current tree still does not have:

1. `continue_world_worker` in the internal dispatch action set,
2. typed retained-worker message/event protocol shapes,
3. exact retained-worker thread/attention contract semantics,
4. fuller steering/action hardening for retained follow-up verbs.

### 5. Post-32 observability truth-sync does not change the ordering decision

The current tree and docs now distinguish:

1. `spawn_world_worker` bootstrap visibility through authoritative retained-worker state plus the existing `registered` runtime event path,
2. `run_world_task` terminal-only reduction rather than first-class dedicated internal dispatch trace publication.

Repo-truth implication:

1. this observability clarification does not reorder family-1 work,
2. the next missing seam is still internal retained-worker continue plus typed event bootstrap.

## Ordering Decision

The next narrow family-1 slice should be:

1. internal retained-world-worker continue and minimal typed event bootstrap first,
2. fuller host-to-world steering-policy hardening second.

Why:

1. the repo already has concrete exact-identity and retained turn-routing primitives to reuse,
2. the missing seam is the first internal retained-worker continue contract,
3. the policy matrix should harden against a real continue/event surface and real denial buckets rather than a hypothetical future contract.

## Blocking Rule

The retained-worker allocation prerequisite is satisfied on the current tree. Reopen this note only if a later regression removes the real `spawn_world_worker` runtime path or breaks the exact retained member-turn seam that Slice `33` plans to reuse.
