# Remaining Family-2 Scope After Slices 48, 49, And 50

Date: `2026-06-07`  
Validated against:
- [REMAINING-family-2-scope-2026-05-30.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-family-2-scope-2026-05-30.md)
- [PLAN-48.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md)
- [PLAN-49.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md)
- [PLAN-50.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-50.md)
- [obligation_ledger.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
- [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [auto_attach.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
- [orchestrator_world_dispatch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs)
- [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md)

## Objective

Record the repo-truth answer to one question:

1. after the post-`31.25` Family-2 slices now landed in the tree, what scope from the `2026-05-30` remaining-scope note is still actually open,
2. and what is the next honest implementation seam now that `PLAN-48`, `PLAN-49`, and `PLAN-50` are in the codebase.

This note is a validation artifact, not a new implementation spec.

## Closed Seams From The 2026-05-30 Note

The older remaining-scope note is now stale in three important ways.

### 1. Router-owned execution boundary is no longer remaining

The `2026-05-30` note said the production router/daemon execution boundary was still unresolved.

That is no longer true in the current tree:

1. [PLAN-48.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md) is marked implemented and validation-green.
2. [orchestrator_world_dispatch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs) now has a non-test host-side router-owned discovery trigger and one-shot execution entrypoint.
3. [auto_attach.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs) now enforces deny-by-default router execution through `workflow.router.enabled` plus the relevant existing retained-obligation gate.

Repo-truth consequence:

1. the Family-2 local execution boundary is shipped,
2. policy gating for router-owned auto-attach is shipped,
3. this is no longer the next remaining Family-2 slice.

### 2. Host-targeting and wrong-host fail-closed behavior are no longer remaining

The `2026-05-30` note said host-targeting and wrong-host handling were future scope.

That is no longer true in the current tree:

1. [PLAN-49.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md) is marked implemented and validation-aligned.
2. [obligation_ledger.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs) now carries `origin_host_id` and `target_host_id`, validates them, and classifies untargeted versus same-host versus wrong-host obligations.
3. [auto_attach.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs) now fails closed before claim or launch when an obligation targets a different host.
4. [orchestrator_world_dispatch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs) stamps exact local host identity on locally produced obligations when available.

Repo-truth consequence:

1. bounded host-targeting truth is shipped,
2. wrong-host posture is shipped,
3. what remains is not "host targeting" generally, but the later ingress/materialization path that would make foreign-targeted delivery legitimate.

### 3. The ingress-ready identity and causation envelope is no longer remaining

The `2026-05-30` note said most of the forward identity envelope was still design-only.

That is no longer true in the current tree:

1. [PLAN-50.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-50.md) is marked implemented and validation-aligned.
2. [obligation_ledger.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs) now carries `ingress_source_kind`, `ingress_source_id`, `ingress_received_at`, `causation_event_id`, `causation_message_id`, and `causation_request_id`.
3. [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) round-trips those fields and preserves backward compatibility for older persisted artifacts that lack them.
4. [orchestrator_world_dispatch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs) now stamps exact local ingress truth plus exact request causation, and preserves event/message causation only when the worker seam surfaces exact identifiers directly.
5. [docs/CONFIGURATION.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md) now documents the landed local-only ingress/causation envelope and explicitly keeps `thread_id` payload-only.

Repo-truth consequence:

1. the local obligation ledger is already widened for ingress-ready identity,
2. the next Family-2 slice should not reopen envelope questions that Slice `50` already settled,
3. the remaining scope has narrowed again.

## Remaining Implementation Scope

After Slices `48`, `49`, and `50`, the next honest remaining Family-2 seam is now narrower than the `2026-05-30` note described.

### 1. Host-global inbox layering under `SUBSTRATE_HOME/host_inbox/` is still not landed

This remains the first real implementation-bearing follow-on:

1. the current obligation ledger is still a local-session artifact, not a host-global ingress materialization layer,
2. the runtime still treats a foreign-targeted obligation that appears locally as an invalid local-only materialization and fails closed accordingly,
3. there is still no shipped `SUBSTRATE_HOME/host_inbox/` artifact or equivalent host-global ingress store in the current tree.

Repo-truth consequence:

1. Slice `49`'s wrong-host fail-closed posture is still transitional,
2. the next Family-2 slice should define how host-global ingress records materialize into the already-landed canonical local obligation ledger,
3. that next slice should preserve the local ledger as the sink rather than replacing it with a different artifact family.

### 2. Broader cross-host delivery and federation work still remains after `host_inbox`, not before it

The current tree and plan stack keep this ordering explicit:

1. host-global inbox layering comes next,
2. only after that should broader cross-host delivery, remote ingress materialization, or router/federation productization be considered.

Repo-truth consequence:

1. it would be scope inflation to jump from Slice `50` straight into broad federation or router lifecycle work,
2. the next honest slice should stay bounded to host-global inbox/materialization semantics.

## Recommended Next Slice Shape

If this becomes the next `SPEC/PLAN/TASKS` family, the narrowest honest objective is:

1. land host-global inbox layering under `SUBSTRATE_HOME/host_inbox/`,
2. define the exact local materialization boundary from host-global ingress records into the already-landed local obligation ledger,
3. keep wrong-host fail-closed posture as the pre-`host_inbox` fallback only,
4. leave broader cross-host delivery, sync, lease, federation routing, and public router lifecycle UX out of scope.

## Bottom Line

The `2026-05-30` remaining-scope note is now materially outdated.

After `PLAN-48`, `PLAN-49`, and `PLAN-50`, Family 2 is no longer waiting on:

1. router-owned execution boundary work,
2. deny-by-default router auto-attach policy gating,
3. bounded host-targeting and wrong-host fail-closed posture,
4. ingress-ready identity and causation envelope widening.

The next honest remaining Family-2 implementation seam is:

1. host-global inbox layering under `SUBSTRATE_HOME/host_inbox/`,
2. then, only after that, any broader cross-host delivery or federation/productization work.
