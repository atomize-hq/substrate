# Authority And Status Map

**Verified against:** preserved review-clean R6/R7 predecessor receipts; R7-3 entry-authority
repair `9fd9d9972` fresh independent built-in `default` `REVIEW CLEAN`; R7-3.1 test-only commit
`f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` fresh independent built-in `default` `REVIEW
CLEAN`. R7-3.1's placeholder acceptance scaffold intentionally produced test-scaffold RED `0 / 1`;
after replacement with the real acceptance assertion, its exact target passed `1 / 1` and full
`progress_acceptance` passed `4 / 4`. R7-3.2's placeholder acceptance scaffolds intentionally
produced test-scaffold RED `3 / 3`; after replacement with the real acceptance assertions, the
exact targets passed `3 / 3`, and the checkpoint filter passed `175` matched tests across targets.
`cargo test -p agent-drift-analyzer -- --nocapture` passes `421 / 421` aggregate; formatting,
`cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and diff checks are green. Both
staged GitNexus gates were LOW / `0` affected processes.
`cargo clippy --workspace --all-targets -- -D warnings` remains RED only in R7-6-owned sentinel test
constructors missing `Checkpoint.delegation`; that witness is routed to
already-planned R7-6.1 and does not reopen R7-3.

**Current phase:** `R7-4` (**SOLE ACTIVE PHASE AT ENTRY ONLY**; active packet: `none`; R7-3
complete at checkpoint-doc commit `931c2701c`, fresh independent review-clean; `CTX-R7-04` proven;
TRANSITION/FIX SERIES `e077de489` + `3dd5ba943` FRESH INDEPENDENT BUILT-IN `default` `REVIEW CLEAN`; FIRST REVIEW'S TWO P2 STALE-STATUS DEFECTS CORRECTED; `R7-4.1` next, unchecked, and
unstarted; no R7-4 production/scorer work has started; `R7-5..R7-6` and R8 blocked; Prompt 1
selectors `PHASE_ID: R7-4` / `ACTIVE_PACKET: none` prepared and eligible but not invoked)

## How To Resolve Truth

Use these layers together rather than silently choosing one:

1. **Repository rules:** `AGENTS.md` and the invoked skill contracts govern workflow and safety.
2. **Live behavior:** source and behavior-level tests establish what exists and what it does.
3. **Active phase authority:** the relevant finding, MAP, SPEC, PLAN, and TASKS establish intended
   scope, acceptance, and sequencing.
4. **Root mirrors:** root `SPEC.md`, `tasks/plan.md`, `tasks/todo.md`, and
   `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` must agree with the active phase.
5. **This pack:** points at the layers above and records gates; it never wins a conflict.

When live behavior and docs disagree, live behavior answers the existence question, but no new
implementation begins until the authority stack is corrected explicitly.

## Current Status

| Family | Status | Canonical status source | Next allowed action |
|---|---|---|---|
| R6 | **CLOSED — R6-CLOSE / CTX-R6-17 COMPLETE / ACTIVE PACKET NONE** | `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md` | Preserve the terminal table and green proof receipt. Do not reopen an ordinary R6 scorer without a new failing witness. |
| R7 | **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0..R7-3 COMPLETE AND REVIEW-CLEAN / CHECKPOINT-DOC COMMIT `931c2701c` FRESH INDEPENDENT BUILT-IN `default` `REVIEW CLEAN` / `CTX-R7-04` PROVEN / R7-4 SOLE ACTIVE PHASE AT ENTRY ONLY / ACTIVE PACKET NONE / TRANSITION/FIX SERIES `e077de489` + `3dd5ba943` FRESH INDEPENDENT BUILT-IN `default` `REVIEW CLEAN`; FIRST REVIEW'S TWO P2 STALE-STATUS DEFECTS CORRECTED / R7-4.1 NEXT, UNCHECKED, AND UNSTARTED / R7-4 PRODUCTION-SCORER WORK UNSTARTED / R7-5..R7-6 AND R8 BLOCKED** | `docs/specs/r7/MAP.md` and the R7 SPEC/PLAN/TASKS | Prompt 1 selectors `PHASE_ID: R7-4` / `ACTIVE_PACKET: none` are prepared and eligible but not invoked. Invoke them only in the next R7-4 phase run; preparation does not start R7-4. |
| R8 — Sentinel Interpretation Consolidation / Integration | **BOUNDARY DEFINED / NOT YET SPECCED** | Root landing-order R8 section | Wait for stable, closed R7 analyzer contract; then create R8 SPEC/PLAN/TASKS. |

## R6 Authority

Read in this order for active R6 work:

1. `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`
2. `docs/specs/r6/MAP.md`
3. `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`
4. the completed `R6-C.1` packet authority in `docs/specs/r6/R6-C.1/`: its SPEC, PLAN, then TASKS
5. the `Named R6 Gap Status Subledger` in `05-proof-decision-regression-ledger.md`
6. root `SPEC.md`, `tasks/plan.md`, and `tasks/todo.md`
7. the R6 section of `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Landed packet documents are historical authority for their bounded decisions. They do not by
themselves prove the broader R6 charter closed.

### Current scorer posture

| Surface | Interim audit posture | R6 terminal requirement |
|---|---|---|
| `dead_end_thrash` | Integrated advancing/stall replay, frozen-corpus invariance, and focused gap/family proof green | **Cutover complete**. |
| `semantic_goal_drift` | Cutover complete by design | **Cutover complete**. Revisit only if a new failing behavioral witness appears. |
| `truth_grounding_gap` | `CTX-R6-12`, cross-checkpoint provenance, applicability controls, and `22 / 22` family proof green | **Fit-for-purpose exception**. |
| `wrong_plan_branch` | Historical `CTX-R6-15` red preserved at `59f098b35`; review-clean fix makes exact target `0 / Low / Cleared`, unflagged, empty evidence; `6 / 6` family proof green | **Fit-for-purpose exception**. |
| `scoring/mod.rs` | Dispatcher infrastructure | **Fit-for-purpose exception** as routing infrastructure; not a fifth scorer. |

## R6-C.0A Remediation Result

`R6-C.0A` is **COMPLETE**. The remediation commits `959cc50cc` and `d3dcda785` corrected the seven
2026-07-13 fresh Pass 1 documentation defects below, culminating in fresh `REVIEW CLEAN` at
`d3dcda785`:

1. frozen replay fixtures prove posture invariance, not comparative integrated scorer improvement;
2. `truth_grounding_gap` needs controls for truth-touching action-before-read and actionful
   planning/research;
3. `wrong_plan_branch` needs an empty-authority path-bearing action control;
4. R6 closure and R7 promotion gates must not allow an ordinary scorer to remain “still open”;
5. one semantic acceptance test is fixture integrity, not live scorer behavior;
6. dispatcher ordering is visible in source but lacks a focused behavioral assertion; and
7. the R6 MAP preliminary investigation and root 2026-07-04 status note need explicit
   historical/superseded labels.

These corrected statements remain constraints for `R6-C.1-SPEC`; their correction does not close
R6, complete the terminal scorer-disposition table, or unblock R7.

## R7 Authority

R7 is now an implementation-ready authority family. `R7-PROMOTE` and `R7-0..R7-3` are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. Only R7-4 is active at entry with packet
`none`; transition/fix series `e077de489` + `3dd5ba943` received fresh independent built-in `default` `REVIEW CLEAN`. Its first review found exactly two P2 stale-status defects—the root landing-order narrative retained an R7-2-era paragraph, and the R7 spec retained a stale R7-3 behavior/receipt-review promotion-gate heading—and fix `3dd5ba943` corrected both. `R7-4.1` is next,
unchecked, and unstarted; no R7-4 production/scorer work has started:

- `docs/specs/r7/MAP.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-spec.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-plan.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-tasks.md`

The R6 `CLOSED` and terminal-disposition entry gate is satisfied. Promotion series `455d0ed90` +
`876ac55de` completes `R7-PROMOTE`, makes the four R7 authority documents implementation-ready, and
received fresh independent built-in `default` `REVIEW CLEAN`. Transition series `6bf0ac6ad` +
`4a887ee0c` + `e83ebb430` received fresh independent built-in `default` `REVIEW CLEAN`. `R7-0.1`
series `a9e75f149` + `55bea5fa5` + `faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` are
fresh independent built-in `default` `REVIEW CLEAN`, completing `R7-0`. The focused fixture target
passes `2 / 2`, full compactor passes `25 / 25` including end-to-end `2 / 2`, and privacy scans over
`24` rows found zero private markers and zero raw UUIDs. Transition/fix series `339744dff` +
`d20cac6a9` is fresh independent built-in `default` `REVIEW CLEAN`. R7-1 task series `e65127720` +
`685cf843b`, `4d122cd9f`, and `e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`.
The R7-1 checkpoint is complete with focused `6 / 6`, end-to-end `6 / 6`, CLI `2 / 2`, full
compactor `36` unit/integration plus `3` doctests, deterministic ordering, and green static/diff/
GitNexus gates. No raw private rollout data was added. Checkpoint-doc commit `1cae7d693` received
fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. Transition commit
`6a8797c15` received `CHANGES REQUIRED` for stale uncommitted-state wording; fix `4ee469014`
corrected it, and a fresh independent built-in `default` re-review returned `REVIEW CLEAN` for the
full series. R7-1 remains complete and `CTX-R7-02` remains proven. R7-2 task commits `c60d05f77`
and `9403c8a24`, plus implementation/fix series `7af2ae517` + `75a353e46`, received fresh
independent built-in `default` `REVIEW CLEAN`; the fix resolved the summary-vs-checkpoint blocker.
The R7-2 task and behavior/static proof is complete at the counts recorded above. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-2
exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Only R7-4 is active at
entry with packet `none`; transition/fix series `e077de489` + `3dd5ba943` received fresh independent built-in `default` `REVIEW CLEAN`. Its first review found exactly two P2 stale-status defects—the root landing-order narrative retained an R7-2-era paragraph, and the R7 spec retained a stale R7-3 behavior/receipt-review promotion-gate heading—and fix `3dd5ba943` corrected both.
`R7-4.1` is next, unchecked, and unstarted; no R7-4 production/scorer work has started.
`R7-5..R7-6` and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-4` / `ACTIVE_PACKET: none` are
prepared and eligible but have not been invoked.

## R8 Authority

The root landing-order document currently defines only this boundary:

- objective: collapse replay/live checkpoint interpretation duplication in sentinel;
- scope: replay input, live input, and operator-surface interpretation helpers;
- acceptance: one shared checkpoint-interpretation seam, presentation-first operator surface, and
  centralized compatibility logic.

The family name for this pack is **R8 Sentinel Interpretation Consolidation / Integration**. The
word “integration” makes the cross-path outcome explicit but does not expand the root scope.

No detailed R8 MAP/SPEC/PLAN/TASKS exists at the verified commit. The control pack must not invent
interfaces or packet order for R8 before R7 closes.

## Historical Material

Use archived specs, old packet prompts, handoffs, and target artifacts only when the active manifest
names them. Historical status prose never overrides a later closure finding or live code.
