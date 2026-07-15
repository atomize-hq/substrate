# R7 Map: Bounded Delegated-Session Semantics

Status: **IMPLEMENTATION-READY / R7-PROMOTE, R7-0, R7-1, AND R7-2 COMPLETE /
CHECKPOINT-DOC COMMIT `78a168c09` FRESH INDEPENDENT REVIEW CLEAN / `CTX-R7-03` PROVEN / R7-3
SOLE ACTIVE PHASE AT ENTRY ONLY / ACTIVE PACKET NONE / CURRENT TRANSITION RECEIPT PENDING FRESH
INDEPENDENT REVIEW / R7-3.1 NEXT, UNCHECKED, AND UNSTARTED / R7-3 ANALYZER/PRODUCTION
IMPLEMENTATION UNSTARTED / R7-4..R7-6 AND R8 BLOCKED / PROMPT 1 SELECTORS `PHASE_ID: R7-3` /
`ACTIVE_PACKET: none` PREPARED AND ELIGIBLE BUT NOT INVOKED**. Promotion series
`455d0ed90` + `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`. The narrow
`R7-PROMOTE -> R7-0` transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh
independent built-in `default` `REVIEW CLEAN`. The docs-only `R7-0.1` series `a9e75f149` +
`55bea5fa5` + `faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` each received fresh
independent built-in `default` `REVIEW CLEAN`, completing `R7-0`. Transition/fix series
`339744dff` + `d20cac6a9` also received fresh independent built-in `default` `REVIEW CLEAN`.
`R7-1.1` series `e65127720` + `685cf843b`, `R7-1.2` commit `4d122cd9f`, and `R7-1.3` commit
`e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`. R7-1 implementation and its
checkpoint are complete. Checkpoint-doc commit `1cae7d693` received fresh independent built-in
`default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Only R7-3 is active at entry with packet
`none`; current transition receipt pending fresh independent review. `R7-3.1` is next, unchecked,
and unstarted; R7-3 analyzer/production implementation has not started. `R7-4..R7-6` and R8 remain
blocked. Prompt 1 selectors `PHASE_ID: R7-3` / `ACTIVE_PACKET: none` are prepared and eligible but
have not been invoked.

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
Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-1 exit gate. R7-1 is complete. Operator decision
`R7-2-HIGH-IMPACT-ANALYZER-CONTRACT-01: A` authorized the bounded high-impact analyzer seam. R7-2
task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 implementation/fix series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN` after the fix reconciled
the summary-vs-checkpoint blocker. At implementation HEAD `75a353e46`, input passes `16 / 16`,
delegation matches pass `39` total, checkpoint matches pass `172` total, full analyzer passes `417 /
417`, and format, analyzer clippy `-D warnings`, and diff checks are green. Staged GitNexus gates
reported R7-2.1 LOW / `0` affected processes, R7-2.2 MEDIUM / `1`, R7-2.3 HIGH / `9` within the
authorized decision, and the fix MEDIUM / `2`. Public v0.8 with v0.7 compatibility, graph-derived
roles and ids, `Linked`/`Partial` visibility, fail-closed conflicts, deterministic `RowRef` evidence,
JSON-summary parity, and separate trajectories are proven. No R7-3, R7-4, sentinel, or R8 work
leaked into the series. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Only R7-3 is active at entry with packet
`none`; current transition receipt pending fresh independent review. `R7-3.1` is next, unchecked,
and unstarted; R7-3 analyzer/production implementation has not started. `R7-4..R7-6` and R8 remain
blocked. Prompt 1 selectors `PHASE_ID: R7-3` / `ACTIVE_PACKET: none` are prepared and eligible but
have not been invoked.

## Live Linkage Evidence

The current raw Codex rollout shape exposes a deterministic direct-link seam:

1. A parent `spawn_agent` function-call output contains the spawned child `agent_id`.
2. The child's `session_meta.payload.source.subagent.thread_spawn` contains the matching
   `parent_thread_id`, `depth`, nickname, and role.
3. The compactor now preserves structured parent spawn-result and child-origin metadata with row
   provenance, emits serde-defaulted typed delegation links, and supports an explicit verified
   direct-child closure without turning linkage into message text.
4. The analyzer loads and validates a deterministic direct link graph over included bundle sessions;
   missing and conflicting link states remain bounded and legacy manifests load with an empty graph.
5. Public checkpoint v0.8 exports `DelegationContext`, topology, graph-derived role ids, visibility,
   confidence, and deterministic `RowRef` evidence. Verified links populate `DelegatingParent` and
   `DelegatedChild`; heuristic markers are fallback-only, and v0.7 remains readable.
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
independent built-in `default` `REVIEW CLEAN`, and the R7-1 checkpoint is complete. Checkpoint-doc
commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-1 exit gate. Transition/fix series `6a8797c15` + `4ee469014` received fresh independent built-in
`default` `REVIEW CLEAN`; its first review's stale uncommitted-state finding is fixed. R7-1 remains
complete and `CTX-R7-02` remains proven. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Only R7-3 is active at entry with packet
`none`; current transition receipt pending fresh independent review. `R7-3.1` is next, unchecked,
and unstarted; R7-3 analyzer/production implementation has not started. `R7-4..R7-6` and R8 remain
blocked. Prompt 1 selectors `PHASE_ID: R7-3` / `ACTIVE_PACKET: none` are prepared and eligible but
have not been invoked.

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
