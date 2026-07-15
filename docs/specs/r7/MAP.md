# R7 Map: Bounded Delegated-Session Semantics

Status: **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0 COMPLETE / R7-1.1 SERIES `e65127720` +
`685cf843b`, R7-1.2 COMMIT `4d122cd9f`, AND R7-1.3 COMMIT `e865eee13` FRESH INDEPENDENT REVIEW
CLEAN / R7-1 IMPLEMENTATION AND CHECKPOINT COMPLETE / CHECKPOINT DOC REVIEW AND SEPARATE PHASE-
TRANSITION REVIEW PENDING / R7-1 SOLE ACTIVE PHASE / ACTIVE PACKET NONE / R7-2..R7-6 AND R8
BLOCKED / NO R7-2 WORK STARTED**. Promotion series
`455d0ed90` + `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`. The narrow
`R7-PROMOTE -> R7-0` transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh
independent built-in `default` `REVIEW CLEAN`. The docs-only `R7-0.1` series `a9e75f149` +
`55bea5fa5` + `faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` each received fresh
independent built-in `default` `REVIEW CLEAN`, completing `R7-0`. Transition/fix series
`339744dff` + `d20cac6a9` also received fresh independent built-in `default` `REVIEW CLEAN`.
`R7-1.1` series `e65127720` + `685cf843b`, `R7-1.2` commit `4d122cd9f`, and `R7-1.3` commit
`e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`. R7-1 implementation and its
checkpoint are complete, but R7-1 remains the sole active phase with packet `none` pending fresh
review of the checkpoint-doc receipt and a separate phase-transition commit/review. R7-2 remains
blocked and unstarted.

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
`default` `REVIEW CLEAN`. The exact `R7-0.1` contract `rg` passed, and series `a9e75f149` +
`55bea5fa5` + `faff68ac6` received fresh independent built-in `default` `REVIEW CLEAN`.
Fixture-only `R7-0.2` commit `fa85cd4b8` also received fresh independent built-in `default`
`REVIEW CLEAN`: all seven sanitized cases parse without failures, the focused target passes
`2 / 2`, the full compactor family passes `25 / 25` including end-to-end `2 / 2`, and all `24`
rows pass privacy scans with zero private markers and zero raw UUIDs. `R7-0` is complete and
review-clean. Transition/fix series `339744dff` + `d20cac6a9` received fresh independent built-in
`default` `REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13`
are fresh independent built-in `default` `REVIEW CLEAN`. Focused delegation-link proof passes `6 /
6`; linked-closure end-to-end and CLI proof pass `6 / 6` and `2 / 2`; the full compactor wall passes
`36` unit/integration tests plus `3` doctests; formatting, clippy, diff, and staged GitNexus gates are
green. Link/session/file ordering is deterministic, and no raw private rollout data was added.
R7-1 implementation and checkpoint are complete, but R7-1 remains the sole active phase with packet
`none` pending fresh review of this checkpoint-doc update and a separate committed and fresh-review-
clean phase transition. `R7-2..R7-6` plus R8 remain blocked; no R7-2 work has started.

## Live Linkage Evidence

The current raw Codex rollout shape exposes a deterministic direct-link seam:

1. A parent `spawn_agent` function-call output contains the spawned child `agent_id`.
2. The child's `session_meta.payload.source.subagent.thread_spawn` contains the matching
   `parent_thread_id`, `depth`, nickname, and role.
3. The compactor now preserves structured parent spawn-result and child-origin metadata with row
   provenance, emits serde-defaulted typed delegation links, and supports an explicit verified
   direct-child closure without turning linkage into message text.
4. The analyzer can load multiple sessions from one compactor bundle, but R7-2 has not yet added a
   link graph between those sessions.
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

The packet order below is implementation-ready. `R7-0` is **complete** after `R7-0.1` and
fixture-only `R7-0.2` commit `fa85cd4b8` each received fresh independent `REVIEW CLEAN`.
Transition/fix series `339744dff` + `d20cac6a9` is also fresh independent built-in `default`
`REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh
independent built-in `default` `REVIEW CLEAN`, and the R7-1 checkpoint is complete. `R7-1` remains
the sole active phase with packet `none` pending checkpoint-doc review and the separate phase-
transition review. `R7-2` is blocked and unstarted.

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
