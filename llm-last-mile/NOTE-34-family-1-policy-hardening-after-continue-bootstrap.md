# Note: Family-1 Ordering After Steering Policy Hardening

Date: `2026-05-31`

Validated against live code in:

- [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
- [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
- [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md](./SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md)
- [PLAN-33.md](./PLAN-33.md)
- [TASKS-33.md](./TASKS-33.md)

## Purpose

Record the current repo truth after Slice `34`, and make explicit why the next implementation-bearing family-1 work is later verb expansion on top of the landed steering-policy floor rather than more policy repair or family-2 router/attach work.

## Current Repo Truth

### 1. The first three family-1 verbs are now real runtime truth

The current tree now has:

1. `run_world_task`,
2. `spawn_world_worker`,
3. `continue_world_worker`

as landed internal dispatch actions with typed request/outcome handling and exact routing over the real world-member runtime seam.

Repo-truth implication:

1. family 1 is no longer blocked on dispatch bootstrap,
2. family 1 is no longer blocked on first retained-worker continue/messaging bootstrap,
3. the next missing seam is no longer "make the verbs real."

### 2. Exact identity, world-binding, and steering-policy checks now exist together

The current tree already enforces:

1. authoritative orchestrator caller validation,
2. exact orchestration-session targeting,
3. exact retained-worker targeting for `continue_world_worker`,
4. exact authoritative world-binding checks,
5. fail-closed stale-linkage rejection.

The current tree now also has the distinct deny-by-default steering-policy layer described in the design stack:

1. global steering enablement,
2. action allowlisting,
3. mode allowlisting,
4. backend allowlisting,
5. explicit capability-narrowing permission,
6. explicit worker/session concurrency caps,
7. explanation-ready denial buckets tied to those policy dimensions.

Repo-truth implication:

1. the first implementation-bearing steering-policy surface is no longer missing,
2. later family-1 verbs now have a real deny-by-default floor to build on,
3. the next remaining gap is no longer "add policy hardening for current verbs."

### 3. The lifecycle model now has a concrete control-plane surface to harden

The lifecycle design is no longer abstract relative to the runtime.

The current tree now has enough landed behavior to harden against concrete lifecycle states for current verbs:

1. `run_world_task` is the in-scope `ephemeral` path,
2. `spawn_world_worker` is the retained allocation path,
3. `continue_world_worker` is the retained follow-up path,
4. invalidated or stale retained workers are already meaningful deny cases.

Repo-truth implication:

1. the lifecycle doc should now feed directly into policy enforcement,
2. the next slice can use real invalidated/terminal worker truth instead of hypothetical future state machines.

### 4. Family 2 docs remain downstream projections, not reasons to reorder the next slice

The obligation-ledger, auto-attach, and router/daemon docs all assume:

1. control-plane identity and worker event truth already exist,
2. obligations are downstream durable artifacts,
3. attach processing restores a host execution client but does not continue workers or resolve review,
4. router-owned attach is session-scoped recovery, not world-steering logic.

Repo-truth implication:

1. family 2 remains adjacent but downstream,
2. it should not be pulled ahead of the missing family-1 steering-policy layer,
3. router/attach work is not the next honest slice after Slice `33`.

## Ordering Decision

The next narrow family-1 slice should be:

1. later verb expansion such as `inspect_world_worker`, `cancel_world_work`, `stop_world_worker`, and `fork_world_worker` on top of the landed steering-policy floor,
2. later approval/fork autonomy after those verb contracts require it,
3. broader family-2 producer coupling and router/attach execution after that.

## Why Verb Expansion Next

1. the dispatch contract is already landed and concrete,
2. the lifecycle model now has real verbs, real invalidation/routability checks, and a real deny-by-default steering layer,
3. later verbs can now widen the family-1 control plane without inheriting an ad hoc authorization model,
4. the router/attach designs still do not own worker continuation or policy authorization,
5. family 2 remains downstream of the now-landed family-1 policy floor.

## Blocking Rule

Reopen this ordering note only if one of these becomes true:

1. current `run_world_task`, `spawn_world_worker`, or `continue_world_worker` runtime truth regresses,
2. a later family-1 verb proves the landed steering-policy surface is still missing a prerequisite contract,
3. family-2 obligation or attach work turns out to require policy/schema decisions that must precede later family-1 verb expansion.

If none of those conditions is true, the next slice should remain later family-1 verb expansion first.
