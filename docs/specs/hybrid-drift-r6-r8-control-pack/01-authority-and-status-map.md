# Authority And Status Map

**Verified against:** preserved review-clean packet/proof series through `b1791c1e3` + `e6d43eee9` + `61c9d5074`; on 2026-07-15 exact `CTX-R6-01`/`02`, renamed sticky, and exact `CTX-R6-06` each passed `1 / 1`; Manifest E family filters passed `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`; full analyzer completed with all suites green; diff check green; R6-close transition/review-fix series `13b14d5f1` + `50446e6d6`, R7 promotion series `455d0ed90` + `876ac55de`, R7 entry transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430`, R7-0.1 series `a9e75f149` + `55bea5fa5` + `faff68ac6`, R7-0.2 commit `fa85cd4b8`, and R7-0 -> R7-1 transition/fix series `339744dff` + `d20cac6a9` each received fresh independent built-in `default` `REVIEW CLEAN`. R7-0.2 focused parser/privacy proof is `2 / 2`, full compactor proof is `25 / 25` including end-to-end `2 / 2`, and privacy scans over `24` rows found zero markers and zero raw UUIDs without any production-symbol edit.

**Current phase:** `R7-1` (**ACTIVE AT ENTRY ONLY**; active packet: `none`; R6 is `CLOSED`; `R7-PROMOTE` and `R7-0` are complete and review-clean; `R7-0.1` series, `R7-0.2` commit `fa85cd4b8`, and transition/fix series `339744dff` + `d20cac6a9` are fresh independent built-in `default` `REVIEW CLEAN`; `R7-1.1` is next, unchecked, and unstarted; production implementation remains unstarted; `R7-2..R7-6` and R8 remain blocked; Prompt 1 for `R7-1` / `none` is prepared but not started)

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
| R7 | **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0 COMPLETE AND REVIEW-CLEAN / R7-0.1 SERIES, R7-0.2 COMMIT `fa85cd4b8`, AND TRANSITION/FIX SERIES `339744dff` + `d20cac6a9` FRESH INDEPENDENT REVIEW CLEAN / R7-1 ACTIVE AT ENTRY ONLY / ACTIVE PACKET NONE / R7-1.1 NEXT, UNCHECKED, AND UNSTARTED** | `docs/specs/r7/MAP.md` and the R7 SPEC/PLAN/TASKS | Preserve the review-clean boundary. Prompt 1 for `R7-1` / `none` is prepared but not started. |
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

R7 is now an implementation-ready authority family. `R7-PROMOTE` and `R7-0` are complete, and
`R7-1` is active at entry only with packet `none`:

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
`d20cac6a9` is fresh independent built-in `default` `REVIEW CLEAN`. Only `R7-1` is active at entry
with packet `none`; `R7-1.1` is next, unchecked, and unstarted.
Production implementation remains unstarted; `R7-2..R7-6` plus R8 remain blocked.

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
