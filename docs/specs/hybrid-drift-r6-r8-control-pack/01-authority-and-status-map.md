# Authority And Status Map

**Verified against:** preserved review-clean R6/R7 predecessor receipts; R7-4.1 commit/fix series
`ebcb052b9` + `e7b65523f` fresh independent built-in `default` `REVIEW CLEAN` after one
P2/Important untruthful-linkage-provenance contract failure was fixed through production
`export_bundle`; R7-4.2 docs decision commit `8a0790a3d` fresh independent built-in `default`
`CLEAN` with no findings. At implementation/docs HEAD `8a0790a3d`, `dead_end_thrash` passes `19 /
19`; the semantic filter passes `58 / 58` aggregate (`56` library plus `2` acceptance); full
analyzer passes `422 / 422`; formatting, analyzer clippy with `-D warnings`, and diff checks are
green. The known workspace-clippy RED remains routed to already-planned R7-6.1 and was neither
rerun nor fixed.

**Current phase:** `R7-5` (**SOLE ACTIVE PHASE AT ENTRY ONLY**; active packet: `none`; R7-4
complete at checkpoint-doc commit `ca8467edda80f14b35f1a4d9a4c2192d43b217a2`, fresh independent
built-in `default` `CLEAN`; current R7-4 -> R7-5 transition candidate pending fresh independent
review and not yet review-clean; `R7-5.1` next, unchecked, and unstarted; no R7-5 acceptance or real-
corpus work started; `CTX-R7-05` blocked/pending until R7-5 acceptance evidence proves existing
classes plus typed delegation context; R7-6 and R8 blocked; Prompt 1 selectors `PHASE_ID: R7-5` /
`ACTIVE_PACKET: none` prepared and eligible but not invoked)

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
| R7 | **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0..R7-4 COMPLETE / R7-4 CHECKPOINT-DOC COMMIT `ca8467edda80f14b35f1a4d9a4c2192d43b217a2` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` / R7-5 SOLE ACTIVE PHASE AT ENTRY ONLY / ACTIVE PACKET NONE / TRANSITION CANDIDATE PENDING FRESH INDEPENDENT REVIEW AND NOT YET REVIEW-CLEAN / R7-5.1 NEXT, UNCHECKED, AND UNSTARTED / R7-5 ACCEPTANCE AND REAL-CORPUS WORK UNSTARTED / `CTX-R7-05` BLOCKED/PENDING UNTIL R7-5 ACCEPTANCE EVIDENCE / R7-6 AND R8 BLOCKED** | `docs/specs/r7/MAP.md` and the R7 SPEC/PLAN/TASKS | Freshly review only the narrow R7-4 -> R7-5 transition candidate. Do not invoke the prepared R7-5 selectors or start R7-5 work before that review boundary is clean. |
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

R7 is now an implementation-ready authority family. `R7-PROMOTE` and `R7-0..R7-4` are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-4.1 commit/fix series `ebcb052b9` +
`e7b65523f` and R7-4.2 docs decision commit `8a0790a3d` are fresh independent review-clean; the
R7-4 behavior/static checkpoint is complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. Only R7-5 is active at entry with packet `none`; the current
transition candidate awaits fresh independent review and is not yet review-clean. `R7-5.1` is next,
unchecked, and unstarted; no R7-5 acceptance or real-corpus work has started. `CTX-R7-05` remains
blocked/pending until R7-5 acceptance evidence proves existing classes plus typed delegation
context:

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
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943` remains fresh independent built-in
`default` `REVIEW CLEAN`. R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs
decision commit `8a0790a3d` each received fresh independent built-in `default` `REVIEW CLEAN`;
R7-4.1, R7-4.2, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. Only R7-5 is active at entry with packet `none`;
the current R7-4 -> R7-5 transition candidate is pending fresh independent review and is not yet
review-clean. `R7-5.1` is next, unchecked, and unstarted; no R7-5 acceptance or real-corpus work has
started. `CTX-R7-05` remains blocked/pending until R7-5 acceptance evidence proves existing classes
plus typed delegation context. R7-6 and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-5` /
`ACTIVE_PACKET: none` are prepared and eligible but have not been invoked.

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
