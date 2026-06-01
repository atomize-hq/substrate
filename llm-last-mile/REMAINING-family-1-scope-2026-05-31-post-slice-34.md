# Remaining Family-1 Scope After Slice 34

Date: `2026-06-01`  
Historical filename retained from the post-Slice `34` checkpoint; current tree now also includes Slice `35`.  
Validated against:
- [NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md](./NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md)
- [PLAN-34.md](./PLAN-34.md)
- [TASKS-34.md](./TASKS-34.md)
- [PLAN-35.md](./PLAN-35.md)
- [TASKS-35.md](./TASKS-35.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- live runtime code in:
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)

## Objective

Record the repo-truth answer to two questions now that Slice `35` is landed on top of the post-Slice `34` checkpoint:

1. what still remains in Family 1,
2. what is the next honest execution-bearing slice after retained-worker inspect became runtime truth?

This note supersedes the old “after Slice 32” framing in [REMAINING-family-1-scope-2026-05-30.md](./REMAINING-family-1-scope-2026-05-30.md).

## Scope Definition

For this note, “Family 1” means the host-orchestrator to world control-plane design stack:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
2. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
3. [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
4. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md) where it defines Family-1 lifecycle truth

This note does not reopen Family 2 except for dependency boundaries:

1. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
2. [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
3. [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)

## Current Repo Floor

The current tree now has the Family-1 foundation that earlier notes were still waiting on.

### 1. The first four internal control verbs are runtime truth

The repo now lands:

1. `run_world_task`,
2. `spawn_world_worker`,
3. `continue_world_worker`,
4. `inspect_world_worker`

as typed internal dispatch actions with exact routing and exact identity validation.

Current landed posture note:

1. the action vocabulary, steering-policy allowlisting, and ingress validation are repo truth across the workspace,
2. retained-worker inspect snapshot routing itself is Linux-only in v1,
3. non-Linux builds fail closed instead of exposing partial inspect routing.

Repo-truth implication:

1. Family 1 is no longer missing dispatch/bootstrap,
2. Family 1 is no longer missing first retained-worker continue/bootstrap,
3. Family 1 is no longer missing its first retained-worker inspect surface,
4. the next missing seam is not “make the initial verbs real.”

### 2. The first steering-policy hardening layer is also runtime truth

The repo now also lands the first deny-by-default steering-policy layer for the current four-verb surface:

1. steering enablement,
2. action allowlisting,
3. mode allowlisting,
4. backend allowlisting,
5. same-session boundary enforcement,
6. same-world-binding boundary enforcement,
7. explicit capability-narrowing permission,
8. stable denial buckets for invalidated-worker and concurrency-cap cases in the current scope,
9. explicit allowlisting support for retained-only `inspect_world_worker`.

Repo-truth implication:

1. Family 1 is no longer missing its first policy floor,
2. later verbs can now widen the control plane on top of a real authorization layer,
3. the next missing seam is not “repair the first steering-policy hardening.”

### 3. Family 2 remains downstream

The obligation-ledger, auto-attach, and router/daemon designs still assume:

1. the control-plane identity model is already frozen,
2. worker lifecycle and follow-up truth already exist,
3. router-owned attach restores host execution posture but does not replace world-steering authorization.

Repo-truth implication:

1. Family 2 is still adjacent but downstream,
2. it is not the next honest execution-bearing slice,
3. router/attach work should not be used to skip the remaining Family-1 verbs.

## What Is Still Missing

Family 1 is no longer missing core control-plane foundation work, but it is still incomplete.

### 1. Later verb expansion is still deferred

The remaining later verbs are:

1. `cancel_world_work`
2. `stop_world_worker`
3. `fork_world_worker`

These are still absent from the live internal dispatch action set and from the effective steering-policy allowlist.

### 2. Broader approval and fork autonomy is still deferred

After later verbs begin landing, the repo still needs:

1. approval-request policy,
2. fork-request and fork-recommendation autonomy policy,
3. any policy widening those flows require.

These remain later than the next honest verb slice.

## What Is Not Left As Foundation Work

The following are no longer the missing Family-1 foundation:

1. dispatch/bootstrap,
2. retained-worker continue bootstrap,
3. minimal typed continue-event bootstrap,
4. first steering-policy hardening for the landed three-verb surface.

Any answer that still treats those as the next missing Family-1 slice is sequencing-stale.

## Why `inspect_world_worker` Was The Smallest Honest Slice 35

Among the later verbs that remained after Slice `34`, `inspect_world_worker` was the narrowest honest next step, and the current tree now lands that scope as an internal retained-worker snapshot surface.

### Why `inspect_world_worker` comes before `cancel_world_work`

1. `cancel_world_work` reaches into active in-flight execution semantics,
2. cancellation needs sharper distinction between active retained turns and active ephemeral work,
3. the lifecycle model explicitly treats cancellation as an execution-affecting action rather than a passive snapshot.

### Why `inspect_world_worker` comes before `stop_world_worker`

1. `stop_world_worker` is a durable lifecycle transition,
2. stopping must freeze terminal closeout behavior and future continuation denial rules,
3. that is a larger policy and state-transition commitment than first-class inspection.

### Why `inspect_world_worker` comes before `fork_world_worker`

1. `fork_world_worker` pulls in lineage, fork depth, child-count limits, and autonomy policy,
2. the retained-worker messaging design makes fork the sharpest policy edge,
3. fork therefore remains the least appropriate “smallest honest next slice.”

### Why retained-worker inspect comes before active-ephemeral inspect

1. retained-worker inspect can reuse exact `target_participant_id` and authoritative session/world-binding truth that already exists,
2. active-ephemeral inspect would need an exact task-run identity surface that is not yet part of the landed Family-1 control plane,
3. the smallest honest inspect slice is therefore retained-worker inspection first.

## Recommended Next Slice

With retained-only inspect now landed on Linux in v1, the next honest Family-1 slice should come from the remaining execution-affecting verbs:

1. **the next later-verb slice: `cancel_world_work` or `stop_world_worker`, whichever proves narrower against the live lifecycle model**

That follow-on slice should:

1. stay internal-only and orchestrator-only,
2. keep active-ephemeral inspect deferred,
3. sequence `fork_world_worker` after the narrower cancel-or-stop slice,
4. continue deferring broader approval policy and Family-2 routing work.

## Boundary Against Family 2

The landed inspect slice still came before Family 2 execution work because:

1. Family 2 depends on the control-plane vocabulary being frozen,
2. inspect widens the host-orchestrator control plane without requiring router ownership,
3. attach/review recovery should consume this verb family, not replace it.

What stays out of scope here:

1. router/daemon execution model changes,
2. obligation-ledger schema widening,
3. host-global inbox or cross-host ingress,
4. worker fork autonomy,
5. cancellation and durable stop execution.

## Bottom Line

After Slice `35`, Family 1 is no longer missing its core control-plane foundation or its first inspect surface:

1. the first four verbs are landed,
2. first steering-policy hardening is landed,
3. what remains is `cancel_world_work`, `stop_world_worker`, and `fork_world_worker`, then broader approval/fork-autonomy work,
4. active-ephemeral inspect and Family-2 routing work remain deferred.
