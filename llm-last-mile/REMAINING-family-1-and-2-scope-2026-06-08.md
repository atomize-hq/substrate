# Remaining Family-1 And Family-2 Scope After Slice 51

Date: `2026-06-08`  
Validated against:
- [REMAINING-family-2-scope-2026-05-30.md](./REMAINING-family-2-scope-2026-05-30.md)
- [REMAINING-family-2-scope-2026-06-07.md](./REMAINING-family-2-scope-2026-06-07.md)
- [REMAINING-family-1-scope-2026-05-30.md](./REMAINING-family-1-scope-2026-05-30.md)
- [REMAINING-family-1-scope-2026-05-31-post-slice-34.md](./REMAINING-family-1-scope-2026-05-31-post-slice-34.md)
- [PLAN-47.md](./PLAN-47.md)
- [PLAN-51.md](./PLAN-51.md)
- [SPEC-46-internal-retained-host-progress-ack-bootstrap.md](./SPEC-46-internal-retained-host-progress-ack-bootstrap.md)
- live runtime code in:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/host_inbox_materialization.rs`](../crates/shell/src/execution/host_inbox_materialization.rs)
  - [`crates/shell/src/execution/agent_runtime/host_inbox.rs`](../crates/shell/src/execution/agent_runtime/host_inbox.rs)
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
  - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

## Objective

Record the repo-truth answer to one question after Slice `51` landed:

1. what work from the previously-cited Family-1 and Family-2 remaining-scope notes is now closed,
2. what work is still honestly remaining,
3. and what should be treated as the next execution-bearing seam rather than stale planning residue.

This note is a validation artifact, not a new implementation spec.

## Scope Definition

For this note:

1. "Family 1" means the host-orchestrator to world control-plane stack.
2. "Family 2" means the durable deferred-work, obligation-ledger, auto-attach, host-targeting, and host-global ingress/materialization stack.

## Repo-Truth Reconciliation

The three older notes named above are no longer equally current.

### 1. The `2026-05-30` Family-2 note is materially stale

That note still treated all of the following as open:

1. router-owned execution boundary,
2. deny-by-default router auto-attach policy gating,
3. host-targeting and wrong-host fail-closed behavior,
4. ingress-ready identity and causation envelope,
5. host-global inbox layering.

Repo truth after Slices `48` through `51` is narrower:

1. Slices `48`, `49`, and `50` already closed items `1` through `4`,
2. Slice `51` now closes item `5` by landing the bounded `host_inbox -> local obligation -> router` layering boundary.

### 2. The `2026-06-07` Family-2 note is now also stale at its main remaining seam

That note correctly narrowed Family 2 down to:

1. `SUBSTRATE_HOME/host_inbox/` layering,
2. exact local materialization into the canonical local obligation ledger,
3. deferral of broader cross-host delivery and federation work.

Repo truth after Slice `51`:

1. the host-global inbox layer is now landed,
2. exact local materialization is now landed,
3. router coexistence remains obligation-ledger-based rather than host-inbox-based,
4. the broader deferred work remains deferred.

So the `2026-06-07` note is directionally right, but its named "next seam" is now closed.

### 3. The `2026-05-30` Family-1 note is heavily stale

That note still treated the following as the remaining foundation:

1. retained-worker continue bootstrap,
2. minimal typed event bootstrap,
3. steering-policy hardening,
4. later verbs such as inspect, cancel, stop, and fork.

Repo truth is now much later than that note:

1. Family 1 continued through Slices `33` to `47`,
2. retained continue, policy hardening, inspect, cancel, stop, fork, host response classes, and active-ephemeral inspect/cancel widening are landed,
3. the repo also now contains the typed host `progress_ack` contract and policy/runtime support even though some later Family-1 planning artifacts still carry stale status text.

That means the older Family-1 note should not be used as the current sequencing source of truth.

## Closed Scope Since The Older Notes

### Family 2 now closed

The following Family-2 seams should be treated as landed rather than remaining:

1. router-owned session auto-attach execution boundary,
2. deny-by-default router gating for the landed local execution path,
3. host-targeted obligation identity with wrong-host fail-closed posture,
4. ingress-ready identity and causation envelope on local obligations,
5. bounded host-global inbox persistence under `SUBSTRATE_HOME/host_inbox/`,
6. idempotent local materialization from host-inbox record to exactly one canonical local obligation,
7. fail-closed local-boundary rejection before obligation creation,
8. proof that router/review projection still consumes only local obligations after materialization.

### Family 1 seams closed relative to the cited older notes

The following Family-1 seams should be treated as landed rather than remaining relative to the cited older remaining-scope notes:

1. internal dispatch/bootstrap for world work,
2. retained-worker continue bootstrap,
3. first steering-policy hardening layer,
4. retained inspect,
5. retained cancel,
6. retained stop,
7. retained fork,
8. typed host response/control loop widening,
9. active-ephemeral exact `task_run_id` identity plus dual-target inspect/cancel widening.

## Remaining Implementation Scope

### 1. Family 2 remaining scope

After Slice `51`, the remaining Family-2 work is no longer local-obligation shaping work. What is clearly still deferred has moved outward to broader multi-host delivery and federation concerns, with host-global lifecycle coordination as a plausible next seam rather than a proven mandatory one from the cited authority docs.

### A. Host-global ingress lifecycle coordination is a likely next seam if the repo still needs it

`PLAN-51.md` explicitly leaves this class of work deferred:

1. receive-cursor protocols,
2. sync-state coordination,
3. other broader host-global ingress lifecycle machinery beyond the first local materialization boundary.

Repo-truth consequence:

1. if the system still needs coordination around how host-global records are received, advanced, retried, or synchronized, that is a likely narrow next Family-2 seam,
2. this would be a follow-on to Slice `51`, not a reopening of local materialization semantics.

### B. Broader cross-host delivery and federation still remain

The landed tree still defers:

1. remote ingress delivery into another host's `host_inbox`,
2. remote ingress materialization beyond the local-host boundary now landed,
3. lease or lock coordination for cross-host delivery,
4. broader federation routing and productization.

Repo-truth consequence:

1. Family 2 is now blocked not on local semantics, but on the later multi-host delivery story,
2. that work should come only after any still-needed host-global ingress lifecycle coordination is frozen.

### C. Public/operator host-inbox UX remains deferred

The landed Slice `51` boundary still excludes:

1. public host-inbox review UX,
2. public host-inbox management UX,
3. broader public router or federation operator surfaces tied to that inbox.

Repo-truth consequence:

1. these are still remaining scope if product requirements need them,
2. they should not be conflated with the already-landed internal host-inbox layer.

### 2. Family 1 remaining scope

Relative to the cited older Family-1 remaining-scope notes, Family 1 no longer has an obvious required implementation-bearing slice on the critical path before Family 2 can continue.

### A. Optional richer message-envelope or autonomy widening may remain only if the repo still needs it

`PLAN-47.md` leaves only narrow optional follow-ons such as:

1. remaining worker-autonomy widening if still needed,
2. richer message-envelope widening if still needed,
3. any further active-ephemeral lifecycle broadening only if exact task identity exposed a real additional gap.

Repo-truth consequence:

1. there is no remaining Family-1 foundation gap comparable to the old `2026-05-30` note,
2. any further Family-1 work should now be justified by a concrete missing runtime story rather than by older sequencing assumptions.

### B. Family-1 planning-doc truth still has a small alignment gap

The runtime and later artifacts are ahead of some Family-1 planning status text:

1. `PLAN-44.md` still says drafted,
2. `PLAN-45.md` still says Packet `4` remains,
3. `PLAN-46.md` still says drafted,
4. live code and later planning artifacts indicate the control-plane is further along than those statuses imply.

Repo-truth consequence:

1. this is primarily a doc-truth alignment gap, not a missing core runtime slice from the cited older Family-1 notes,
2. if the team wants planning hygiene, the stale Family-1 status text around Slices `44` through `46` should be reconciled with the live runtime and later artifacts.

## Recommended Next Slice Order

If the team wants the next honest implementation-bearing follow-on after Slice `51`, the order should now be:

1. likely next, any still-needed Family-2 host-global ingress lifecycle coordination such as receive-cursor or sync-state coordination,
2. clearly still deferred, broader cross-host delivery, lease/lock coordination, remote ingress materialization, or federation routing/productization,
3. treat any further Family-1 work as optional or demand-driven unless a concrete missing runtime story appears.

## Bottom Line

After Slice `51`, the older remaining-scope notes should be read as historical checkpoints, not current next-step guidance.

Current repo truth is:

1. Family 2 local semantics are landed through `host_inbox -> local obligation -> router`,
2. the only clearly proven remaining Family-2 class from the cited authority trail is broader multi-host delivery and federation work, with host-global lifecycle coordination as a likely but not yet proven-required next seam,
3. there is no remaining mandatory scope from the cited older Family-1 notes,
4. Family-1 follow-on work is now optional or demand-driven, with a planning-doc truth-sync still needed around stale status text in Slices `44` through `46`.
