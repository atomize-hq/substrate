# R7 Map: Bounded Delegated-Session Semantics

Status: **DRAFT / R7-PROMOTE ACTIVE AT ENTRY ONLY / R6 CLOSURE GATE SATISFIED / NOT
IMPLEMENTATION-READY**. This planning scaffold was created on 2026-07-12 and is preserved as useful
design work. `R7-PROMOTE` is the sole active phase with packet `none`; no R7 promotion or
implementation task has started.

## R6 Handoff

The attached planning direction correctly identified delegated-session opacity as the next major
architectural gap. Its proposed new `dead_end_thrash` cutover would duplicate the landed R6-1 core,
and the broader R6 closure charter is now closed:

- `R6-1` already cut `dead_end_thrash` over to analyzer-owned `SessionProgress`, including
  troubleshooting-frontier advancement suppression and non-advancing stall evidence.
- `R6-2`, `R6-3`, and the `R6-3.X.2B` / `2C` / `2D` chain landed the semantic-goal-drift consumer,
  rolling comparison, target hygiene, weighted relation routing, and explicit decision semantics.
- `R6-3.X.3` remains deferred. No new semantic-goal-drift work starts without new failing evidence.
- `truth_grounding_gap` and `wrong_plan_branch` completed their bounded gaps and are terminal
  **Fit-for-purpose exception** surfaces.
- conditional `R6-4` progress-reset migration remains deferred because the required reset-error
  evidence did not appear.

R6 is **CLOSED**. `R6-REPLAY`, `R6-CLOSE`, and `CTX-R6-17` are complete. The closure authority is
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`. Commit `99efda8f9` remains draft
planning history, not authority that the preserved R7 family is already implementation-ready.

The R6 promotion entry gate is satisfied: every material scoring surface has a terminal disposition,
the broad R6 acceptance claims are proven or narrowed honestly, the named closure controls are
resolved, and the R6 finding plus root/R6/R7 gate/status stack agree. `R7-PROMOTE` must now reconcile
the preserved MAP/SPEC/PLAN/TASKS to implementation-ready in a separate docs/status phase. This R6
closeout does not perform that promotion or begin implementation.

## Live Linkage Evidence

The current raw Codex rollout shape exposes a deterministic direct-link seam:

1. A parent `spawn_agent` function-call output contains the spawned child `agent_id`.
2. The child's `session_meta.payload.source.subagent.thread_spawn` contains the matching
   `parent_thread_id`, `depth`, nickname, and role.
3. The current compactor records the child session id from `session_meta`, but normalizes only base
   instructions from that event and therefore drops the structured parent-link metadata.
4. The analyzer can already load multiple sessions from one compactor bundle, but it has no link
   graph between those sessions.
5. The analyzer's existing `DelegationContext`, `DelegationTopology`, and `ChildWorkVisibility` are
   private and heuristic; `DelegatedChild` is defined but not populated from verified linkage.
6. Real-session sentinel coordination currently compacts one requested session and rejects analyzer
   output containing any additional session id.

This evidence fixes the ownership boundary: the compactor must preserve and validate raw linkage;
the analyzer must interpret linked trajectories; the sentinel must consume analyzer-owned semantics
rather than parse raw collaboration events itself.

## Non-Negotiable Invariant

> Never infer child implementation progress, child drift, or child completion from parent
> orchestration alone.

When child evidence is missing or a link is not verified, the supported result is bounded:
`delegating_parent`, `child_work_visibility=opaque|partial`, and insufficient evidence for child
semantics. Ordinary parent-visible orchestration may still be described as parent activity.

## Packet Order

The packet order below is design-ready but **inactive** until the active `R7-PROMOTE` phase itself
completes. Its R6 entry gate is satisfied; promotion has not yet been performed.

1. **R7-0 — docs lock and evidence fixtures.** Freeze the direct-link contract and sanitized
   positive/negative fixture matrix before production behavior changes.
2. **R7-1 — compactor linkage and bounded child closure.** Preserve parent/child metadata, verify
   reciprocal direct links, and add an explicit opt-in linked-child bundle closure.
3. **R7-2 — analyzer link graph and public trajectory roles.** Promote verified topology into an
   analyzer-owned public checkpoint surface while preserving separate session trajectories.
4. **R7-3 — child-visible progress separation.** Reuse each child's own `SessionProgress`; do not
   copy child progress into parent progress or synthesize it from waits/results.
5. **R7-4 — delegated scorer guardrails.** Apply existing scorers per trajectory and prevent
   parent-side opacity or waiting from becoming claims about child drift. No new drift class is
   required without acceptance evidence.
6. **R7-5 — delegated acceptance and real-corpus proof.** Prove linked, missing, conflicting,
   multi-child, and non-delegated cases with explicit topology strata.
7. **R7-6 — minimal sentinel compatibility.** Accept and render analyzer-owned linked trajectories
   in replay/live without performing the broader R8 interpretation consolidation.

## Deferred Beyond This Family

- recursive or arbitrary-depth execution graphs; the first supported model is direct parent to
  child, with deeper descendants remaining explicit bounded residue;
- inferred links based only on prose, filenames, or temporal proximity;
- aggregating child progress into one parent progress status;
- a new delegated-session drift class without real acceptance evidence;
- learned delegation or progress models;
- R8 replay/live interpretation consolidation;
- reopening `R6-3.X.3`, conditional `R6-4`, or closed semantic-goal-drift packets.
