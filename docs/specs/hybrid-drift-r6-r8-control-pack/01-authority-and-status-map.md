# Authority And Status Map

**Verified against:** `ea19b39a7`

**Current phase:** `R6-C.1-CONTROLS` (**ACTIVE**; active packet authority:
`docs/specs/r6/R6-C.1/`)

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
| R6 | **PARTIAL / CLOSURE AUDIT REQUIRED** | `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md` | Add the first row-atomic acceptance control from the landed `docs/specs/r6/R6-C.1/` packet docs; make no production change first. |
| R7 | **DRAFT / BLOCKED ON R6 CLOSURE DECISION** | `docs/specs/r7/MAP.md` and the R7 SPEC/PLAN/TASKS | Preserve draft design only. No implementation. |
| R8 — Sentinel Interpretation Consolidation / Integration | **BOUNDARY DEFINED / NOT YET SPECCED** | Root landing-order R8 section | Wait for stable, closed R7 analyzer contract; then create R8 SPEC/PLAN/TASKS. |

## R6 Authority

Read in this order for active R6 work:

1. `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`
2. `docs/specs/r6/MAP.md`
3. `docs/specs/r6/DESIGN-r6-scorer-cutover-and-objective-consumption.md`
4. the active packet authority in `docs/specs/r6/R6-C.1/`: its SPEC, PLAN, then TASKS
5. root `SPEC.md`, `tasks/plan.md`, and `tasks/todo.md`
6. the R6 section of `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

Landed packet documents are historical authority for their bounded decisions. They do not by
themselves prove the broader R6 charter closed.

### Current scorer posture

| Surface | Interim audit posture | R6 terminal requirement |
|---|---|---|
| `dead_end_thrash` | Acceptance-proof gap | TBD: **Cutover complete**, **Fit-for-purpose exception**, **Merged/deprecated**, or **Explicitly deferred outside R6 with justification**. |
| `semantic_goal_drift` | Cutover complete by design | **Cutover complete**. Revisit only if a new failing behavioral witness appears. |
| `truth_grounding_gap` | Acceptance-proof gap | TBD: one of the four exact terminal categories after the expanded `R6-C.1` controls. |
| `wrong_plan_branch` | Acceptance-proof gap | TBD: one of the four exact terminal categories after the expanded `R6-C.1` controls. |
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

R7 remains a preserved draft family:

- `docs/specs/r7/MAP.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-spec.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-plan.md`
- `docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-tasks.md`

Promotion requires an R6 `CLOSED` finding and only terminal scorer dispositions. Promotion is a
docs/status change before implementation begins.

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
