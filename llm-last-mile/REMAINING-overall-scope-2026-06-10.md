# Remaining Scope Consolidated View

Date: `2026-06-12`  
Provenance: originally consolidated on `2026-06-10`; refreshed on `2026-06-11` after Slice `55` Packet `4` closeout and refreshed again on `2026-06-12` after Slice `56` Packet `4` checkpoint-green closeout so this file remains the canonical current-state note even though the filename keeps the original consolidation date.  
Validated against:
- historical checkpoint `REMAINING-family-1-scope-2026-05-30.md` (now archived locally during cleanup)
- historical checkpoint `REMAINING-family-1-scope-2026-05-31-post-slice-34.md` (now archived locally during cleanup)
- historical checkpoint `REMAINING-family-2-scope-2026-05-30.md` (now archived locally during cleanup)
- historical checkpoint `REMAINING-family-2-scope-2026-06-07.md` (now archived locally during cleanup)
- historical checkpoint `REMAINING-family-1-and-2-scope-2026-06-08.md` (now archived locally during cleanup)
- historical checkpoint `REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md` (now archived locally during cleanup)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [PLAN-47.md](./PLAN-47.md)
- [PLAN-51.md](./PLAN-51.md)
- [PLAN-52.md](./PLAN-52.md)
- [PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./PLAN-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)
- [PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md)
- [PLAN-55-broader-caller-surface-contract-freeze.md](./PLAN-55-broader-caller-surface-contract-freeze.md)
- [SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md)
- [SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)
- [SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](./SPEC-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md)
- [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)
- [TASKS-54.md](./TASKS-54.md)
- [TASKS-55.md](./TASKS-55.md)
- live runtime code in:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/host_inbox.rs`](../crates/shell/src/execution/agent_runtime/host_inbox.rs)
  - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)

## Objective

Replace the scattered `REMAINING-*` picture with one canonical current-state note that answers:

1. what the older remaining-scope notes were each tracking,
2. what they collectively prove is already landed,
3. what is still honestly remaining now,
4. what the next execution-bearing seams are,
5. and which older `REMAINING-*` notes are now historical-only and ready for later archive.

This note is the current consolidated view, not a new implementation spec.

## Scope Definition

For this note:

1. **Family 1** means the host-orchestrator to world control-plane stack.
2. **Host-orchestrator tool surface** means the newer Family-1 sub-seam that exposes the landed internal dispatch/runtime to the live host runtime family.
3. **Family 2** means the durable deferred-work, obligation-ledger, auto-attach, host-targeting, and host-global ingress/materialization stack.
4. **Current** means repo truth after the landed Slice `56` Packet `4` checkpoint-green closeout reflected on `2026-06-12`.

## How To Use This Note

Treat this file as the single current remaining-scope snapshot for `llm-last-mile/`.

The older `REMAINING-*` files below should now be read as:

1. historical checkpoints for the slice windows they covered,
2. evidence for how sequencing changed over time,
3. and archive candidates once the team is ready to move them out of the top-level `llm-last-mile/` directory.

## Historical Map Of The Existing REMAINING Notes

### 1. Family-1 note after Slice 32

File:

1. `REMAINING-family-1-scope-2026-05-30.md`

Current status:

1. historical checkpoint only,
2. useful for explaining why Slice `33` was the honest next seam at that time,
3. no longer valid as a current remaining-scope source because Slices `33` through `47` and later Slices `52` through `53` moved the runtime far past that baseline.

### 2. Family-1 note after Slice 34

File:

1. `REMAINING-family-1-scope-2026-05-31-post-slice-34.md`

Current status:

1. historical checkpoint only,
2. useful for explaining why Slice `35` was the smallest honest next seam at that moment,
3. no longer valid as the current Family-1 remaining picture because later Family-1 follow-up verbs, policy/runtime widening, and the newer host-tool-surface work all landed after it.

### 3. Family-2 note after Slices 31 and 31.25

File:

1. `REMAINING-family-2-scope-2026-05-30.md`

Current status:

1. historical checkpoint only,
2. useful for explaining what was still open before Slices `48` through `51`,
3. materially stale as a current picture because router-owned execution, deny-by-default gating, bounded host-targeting, ingress-ready identity, and host-global inbox layering were all narrowed or landed later.

### 4. Family-2 note after Slices 48, 49, and 50

File:

1. `REMAINING-family-2-scope-2026-06-07.md`

Current status:

1. historical checkpoint only,
2. directionally useful because it correctly narrowed the next Family-2 seam to `host_inbox` layering,
3. superseded as current truth once Slice `51` landed that boundary.

### 5. Combined Family-1 and Family-2 note after Slice 51

File:

1. `REMAINING-family-1-and-2-scope-2026-06-08.md`

Current status:

1. historical checkpoint plus prior cross-family summary,
2. still useful because it reconciled the older Family-1 and Family-2 notes against the post-`51` tree,
3. superseded as the top-level current picture because Slice `53` has since landed and changed the immediate Family-1 next seam again.

### 6. Host-orchestrator tool-invocation seam note

File:

1. `REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md`

Current status:

1. historical checkpoint plus detailed seam ledger for Slices `52` and `53`,
2. still the best slice-local record of the contract-freeze and first-family landing discoveries,
3. superseded as the **overall** remaining-scope picture now that the Codex-backed first-family landing is closed, selected-host claude_code parity is landed through Slice `54`, and the live follow-through has moved to later docs/smoke or other product-priority choices.

## Current Repo Truth

### 1. Family 1 foundation work is no longer the main gap

The older Family-1 notes are stale as current sequencing because the repo has already landed:

1. internal dispatch/bootstrap for world work,
2. retained-worker continue bootstrap,
3. steering-policy hardening at the first shipped layer,
4. retained inspect, cancel, stop, and fork,
5. typed host-response/control-loop widening,
6. active-ephemeral exact `task_run_id` identity plus dual-target inspect/cancel widening.

Repo-truth consequence:

1. the current Family-1 gap is no longer missing dispatch foundations,
2. the active remaining Family-1 seam now sits **above** the landed transport/runtime rather than below it.

### 2. The internal host toolbox transport and the first runtime-family landing are landed

The repo now has:

1. a landed session-scoped internal toolbox transport,
2. a frozen seven-tool adapter contract from Slice `52`,
3. the first live Codex-backed host-tool landing from Slice `53`,
4. truthful operator/reporting posture that keeps non-Codex host sessions supported but not overclaimed as parity.

Repo-truth consequence:

1. the current question is no longer whether the first host-tool surface exists at all,
2. the current question is how and when the second runtime family reaches the same semantic floor.

### 3. Family 2 local semantics are landed through the first host-global inbox boundary

The repo now has:

1. router-owned session auto-attach execution boundary,
2. deny-by-default router gating for the landed local execution path,
3. bounded host-targeting with wrong-host fail-closed posture,
4. ingress-ready identity and causation envelope on obligations,
5. bounded `SUBSTRATE_HOME/host_inbox/` layering,
6. exact local materialization into the canonical local obligation ledger.

Repo-truth consequence:

1. Family 2 is no longer blocked on local obligation semantics,
2. its remaining scope has moved outward to later host-global lifecycle coordination and broader multi-host delivery questions.

## What Is Still Honestly Remaining

### 1. Family 1: second runtime-family parity is now landed baseline truth

Slice `54` closed the previously active Family-1 parity seam:

1. selected `claude_code` host starts and turns now take the same authoritative host-tool path already proven for the first validated family floor,
2. the same seven-tool semantic contract, runtime-owned identity injection, and receipt/follow-up rules remain shared,
3. operator/reporting surfaces can now publish selected-host `claude_code` posture as validated without forcing runtime selection or hard-coding orchestrator identity.

Repo-truth consequence:

1. `claude_code` parity should no longer be treated as the next open seam,
2. future host-tool work should start from multi-family parity as the landed baseline,
3. public toolbox CLI widening, MCP-first redesign, and hidden-fallback behavior remain out of scope.

### 2. Family 1: broader docs/smoke follow-through still remains, but only when later surface widening changes repo truth

After Slice `54`, the remaining bounded host-tool follow-through is:

1. broader runtime-family smoke coverage,
2. broader docs alignment,
3. operator/runtime truth updates only when some later live surface widening materially changes repo truth again.

Repo-truth consequence:

1. this is still real remaining scope,
2. but it is no longer evidence that selected-host `claude_code` parity is missing today.

### 3. Family 1: optional richer autonomy/message widening is no longer mandatory foundation work

The older Family-1 planning trail leaves room for narrower optional follow-ons such as:

1. richer message-envelope widening if the repo still needs it,
2. worker-autonomy widening if the repo still needs it,
3. further active-ephemeral lifecycle widening only if a new exact-identity/runtime gap appears.

Repo-truth consequence:

1. these are no longer mandatory foundation slices,
2. they should be reopened only if a concrete missing runtime story justifies them.

### 4. Family 2: host-global ingress lifecycle coordination is the next likely seam if the repo still needs it

After Slice `51`, the clearest remaining Family-2 seam is no longer materialization itself. It is the later coordination problem around host-global ingress lifecycle behavior, such as:

1. receive-cursor protocols,
2. sync-state coordination,
3. retry/advance semantics beyond the first local materialization boundary.

Repo-truth consequence:

1. if the repo still needs more than the landed `host_inbox -> local obligation -> router` boundary, this is the next likely Family-2 seam,
2. it should be treated as a follow-on above the landed local semantics, not a reopening of them.

### 5. Family 2: broader multi-host delivery and federation remain deferred

The later Family-2 remaining scope still includes:

1. remote ingress delivery into another host’s `host_inbox`,
2. remote ingress materialization beyond the local-host boundary,
3. lease or lock coordination for cross-host delivery,
4. broader federation routing and productization.

Repo-truth consequence:

1. these are still real remaining scope,
2. but they are later than the first host-global coordination question above.

### 6. Family 2: public/operator host-inbox UX remains deferred

Still deferred:

1. public host-inbox review UX,
2. public host-inbox management UX,
3. broader public router or federation operator surfaces tied to that inbox.

Repo-truth consequence:

1. this is still optional product-facing remaining scope,
2. it should not be confused with the already-landed internal Family-2 runtime boundary.

### 7. Planning/doc truth still has a small cleanup tail

The remaining-scope notes already identified a smaller doc-truth gap around older Family-1 planning artifacts such as:

1. `PLAN-44.md`,
2. `PLAN-45.md`,
3. `PLAN-46.md`.

Repo-truth consequence:

1. this is real cleanup work,
2. it does not change runtime sequencing or reopen Slice `30` semantics,
3. and it is not a production-runtime seam comparable to the remaining Family-1 docs/smoke follow-through or Family-2 coordination work.

## Recommended Next Slice Order

### Immediate next seam

1. **Slice `56` is now closed checkpoint-green**
   - Slice `54` closed selected-host `claude_code` parity and Slice `55` closed the caller-surface contract freeze.
   - Slice `56` Packets `1` through `4` are now landed in the current tree: degraded `agent status` stays readable with warnings, strict control surfaces stay fail-closed, participant-aware fallback stays sibling-distinct when evidence exists, and retained legacy handle names are explicitly bounded to compatibility/storage-only posture.
   - The immediate next Family-1 work is therefore no longer another Packet inside Slice `56`; any follow-on is broader docs/smoke/operator-truth work only when later live-surface widening changes repo truth again.

### After that

2. **Family 1: broader docs/smoke follow-through**
   - but only after later live surface widening changes repo truth again.

### If the team pivots back to Family 2

3. **Family 2: host-global ingress lifecycle coordination**
   - if the product/runtime still needs coordination beyond the landed local materialization boundary.

### Clearly later

4. **Family 2: broader cross-host delivery and federation**
5. **Public/operator UX follow-through**

This is the current recommended order from the remaining-scope notes themselves. It is not a claim that all later work must happen before any unrelated product priority.

## What Should No Longer Be Treated As The Current Missing Core

Do not treat the following as the current primary missing seams:

1. missing Family-1 dispatch/bootstrap foundations,
2. missing retained-worker continue as the next Family-1 slice,
3. missing pre-`51` local Family-2 materialization semantics,
4. prompt-only as a sufficient host-tool model,
5. subprocess `substrate ...` calls as the primary host-tool contract,
6. MCP as the internal source of truth for the first live host-tool surface.

## Archive Recommendation

If the team wants to archive the older `REMAINING-*` notes after this cleanup, the simplest honest approach is:

1. keep **this** file at the top level as the canonical current remaining-scope picture,
2. move the following files into `llm-last-mile/archive/`:
   - `REMAINING-family-1-scope-2026-05-30.md`
   - `REMAINING-family-1-scope-2026-05-31-post-slice-34.md`
   - `REMAINING-family-2-scope-2026-05-30.md`
   - `REMAINING-family-2-scope-2026-06-07.md`
   - `REMAINING-family-1-and-2-scope-2026-06-08.md`
   - `REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md`
3. treat those archived files as historical checkpoints rather than active current-state trackers.

## Bottom Line

Current repo truth is:

1. the older `REMAINING-*` notes are now best read as historical checkpoints, not the current top-level source of truth,
2. Family 1 foundation work is landed through the first Codex-backed host-tool floor,
3. selected-host `claude_code` parity is landed through Slice `54`,
4. Slice `56` Packet `4` is checkpoint-green in the current tree, so there is no remaining Packet inside that hardening seam,
5. broader Family-1 docs/smoke follow-through is a later bounded update only when some future live surface widening changes repo truth again,
6. Family 2 local semantics are landed through the first `host_inbox -> local obligation -> router` boundary,
7. the next likely Family-2 seam, if still needed, is host-global ingress lifecycle coordination rather than reopening local materialization,
8. broader cross-host delivery, federation, and public/operator UX remain later or optional work,
9. this document now contains the current overall picture and is the best candidate to keep at top level when the older `REMAINING-*` notes are archived.
