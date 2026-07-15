# R7 Map: Bounded Delegated-Session Semantics

Status: **IMPLEMENTATION-READY / R7-PROMOTE COMPLETE / R7-0 ACTIVE AT ENTRY ONLY / ACTIVE PACKET
NONE / R7-0.1 NEXT / R7-0.1, R7-0.2, AND IMPLEMENTATION NOT STARTED**. Promotion series
`455d0ed90` + `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`. The narrow
`R7-PROMOTE -> R7-0` transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh
independent built-in `default` `REVIEW CLEAN`. It starts no `R7-0` task.

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
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`. Commit `99efda8f9` remains historical
planning input, not authority for the current R7 phase status.

The R6 promotion entry gate is satisfied: every material scoring surface has a terminal disposition,
the broad R6 acceptance claims are proven or narrowed honestly, the named closure controls are
resolved, and the R6 finding plus root/R6/R7 gate/status stack agree. Promotion series `455d0ed90`
+ `876ac55de` reconciled the preserved MAP/SPEC/PLAN/TASKS to implementation-ready authority and
received fresh independent built-in `default` `REVIEW CLEAN`, completing `R7-PROMOTE`. The narrow
status transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in
`default` `REVIEW CLEAN`. `R7-0` is active at entry only with packet `none`; `R7-0.1` is next, and
neither `R7-0.1` nor `R7-0.2` has started.

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

The packet order below is implementation-ready. `R7-0` is **active at entry only** with packet
`none`; its transition is fresh independent `REVIEW CLEAN`, and `R7-0.1` is next but remains
unstarted. No `R7-0.1`, `R7-0.2`, fixture, or implementation work has started.

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
