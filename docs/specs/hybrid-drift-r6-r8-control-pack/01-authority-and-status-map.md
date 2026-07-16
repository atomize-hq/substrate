# Authority And Status Map

**Verified against:** preserved review-clean R6/R7 predecessor receipts; operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A`; R7-6.1 commit `7789fba4f`, R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486`, and bounded final-wall fix `bd743eacc`, each fresh
independent built-in `default` `CLEAN`. Focused sentinel proof passes `27 / 27`, `16 / 16`,
`16 / 16`, `12 / 12`, and `10 / 10`. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with
`-D warnings`, full compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full
workspace tests, and diff checks are green. Staged GitNexus stayed within the authorized HIGH helper
and otherwise MEDIUM/LOW, with no additional HIGH/CRITICAL symbol. Checkpoint-doc
receipt/review-fix series `0e5150945` + `e634ef324` + `8e39c109e` received fresh independent
built-in `default` `CLEAN`, satisfying the R7-6 exit gate; R7 is closed with a stable analyzer
contract.

**Current phase:** `R8-SPEC` (**SOLE ACTIVE PHASE — IN PROGRESS**; active packet: `none`)

R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. Fresh independent built-in `default` review of the complete family at
`0ed3d8f04` + `cfcf65507` returned `CHANGES_REQUIRED` with five scoped documentation findings.
Bounded docs-only fix `2b9565fb9` landed. Follow-up fix `b04207fb6` then received fresh
independent built-in `default` `CHANGES_REQUIRED` with one scoped conditional-acceptance finding.
Bounded conditional-acceptance fix `b9ce44c6f` then received fresh independent built-in `default`
`CHANGES_REQUIRED` with two scoped documentation findings. Bounded two-finding docs-only fix
`904c93d0d` then received fresh independent built-in `default` `CHANGES_REQUIRED` with one scoped
Option B call-path finding. This bounded Markdown-only fix addresses only that one finding and claims
no review result. All R8
implementation tasks remain unchecked and unstarted. `CTX-R8-02` is `OPEN` / `REVIEW PENDING` and
not proven; `CTX-R8-03` through `CTX-R8-06` remain `BLOCKED`. R8-4 and `CTX-R8-05` remain
decision-blocked by their future structured gates. R8-IMPLEMENT remains blocked/boundary-only, and
no R8 code has started. No phase transition, Prompt 1 eligibility,
implementation authorization, complete-family `CLEAN`, or review result for this progress receipt
is claimed.

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
| R7 | **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0..R7-6 COMPLETE / R7 CLOSED / `CTX-R7-06` PROVEN** | `docs/specs/r7/MAP.md` and the R7 SPEC/PLAN/TASKS | R8-SPEC is the sole active phase and is in progress with packet `none`; the R8 MAP/SPEC contract series `698c766f9` + `f5865fb7` + `95529809` is fresh independent built-in `default` `CLEAN` with no findings; `CTX-R8-01` is `PROVEN`. Complete-family series `0ed3d8f04` + `cfcf65507` returned `CHANGES_REQUIRED` with five docs findings; fixes `2b9565fb9` and `b04207fb6` landed; conditional fix `b9ce44c6f` received fresh independent built-in `default` `CHANGES_REQUIRED` with two docs findings; two-finding fix `904c93d0d` received fresh independent built-in `default` `CHANGES_REQUIRED` with one Option B call-path finding; the current one-finding Markdown fix has no review result. `CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven; `CTX-R8-03..06`, R8-4/`CTX-R8-05`, and R8-IMPLEMENT remain blocked. |
| R8 — Sentinel Interpretation Consolidation / Integration | **R8-SPEC ACTIVE / IN PROGRESS / ACTIVE PACKET NONE / `CTX-R8-01` PROVEN / `CTX-R8-02` OPEN / REVIEW PENDING / `CTX-R8-03..06` BLOCKED / R8-4 AND `CTX-R8-05` DECISION-BLOCKED / R8-IMPLEMENT BLOCKED/BOUNDARY-ONLY** | Root landing-order R8 section and the R8 MAP/SPEC/PLAN/TASKS family | MAP/SPEC series `698c766f9` + `f5865fb7` + `95529809` is clean with no findings. Complete-family series `0ed3d8f04` + `cfcf65507` received `CHANGES_REQUIRED` with five docs findings; `2b9565fb9` and `b04207fb6` landed; `b9ce44c6f` received fresh independent built-in `default` `CHANGES_REQUIRED` with two docs findings; `904c93d0d` received fresh independent built-in `default` `CHANGES_REQUIRED` with one Option B call-path finding; the current bounded fix addresses only that finding and claims no result. All implementation tasks remain unchecked/unstarted. Claim no phase transition, Prompt 1 eligibility, implementation authorization, full-family `CLEAN`, or review result for this receipt. |

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
satisfying the R7-4 exit gate. R7-5.1 commit `afb10827d` received fresh independent built-in `default` `CLEAN` with no findings,
and R7-5.2 commit `9c0690a02` received fresh independent built-in `default` `CLEAN` with no
actionable findings. The focused acceptance targets pass `2 / 2`, `8 / 8`, and `4 / 4`; the report
records `115` checkpoints across required strata `23 / 18 / 0 / 23 / 51 / 0`, `3,126` valid
evidence references, zero malformed references, and zero cross-trajectory progress/scorer ownership
violations. At final proof HEAD `9c0690a02`, formatting, compactor/analyzer clippy with `-D warnings`,
full compactor `39 / 39` aggregate, full analyzer `424 / 424` aggregate, and `git diff --check` are
green. R7-5.1, R7-5.2, and the Checkpoint R7-5 behavior items are complete; `CTX-R7-05` is
`PROVEN`. Checkpoint-doc commit `b4916e565cd48f0924fb720d633b57d31b0d624c` received fresh
independent built-in `default` `CLEAN` with no findings, satisfying the R7-5 exit gate. R7-5 is complete. Operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A` authorized only adding `v0.8` to the centralized
explicit-analyzer-state helper, proving explicit posture plus state-backed evidence, preserving
v0.2 and v0.3-v0.7 behavior, and forbidding generalized version parsing or R8 consolidation.
R7-6.1 commit `7789fba4f` received fresh independent built-in `default` `CLEAN`; live compatibility
passes `27 / 27`, replay input `16 / 16`, and operator surface `16 / 16`. R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486` received fresh independent built-in `default` `CLEAN` after
cursor-regression and naming fixes; `real_session_live` passes `12 / 12` and `live_end_to_end`
passes `10 / 10`, proving verified direct closure, per-session cursors, fail-closed unexpected-
session behavior, and unchanged scheduling. Bounded final-wall fix `bd743eacc` received fresh
independent built-in `default` `CLEAN` for the `serde_json` workspace feature-unification test-order
witness. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with `-D warnings`, full
compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full workspace tests,
and `git diff --check` are green. Staged GitNexus gates stayed within the authorized HIGH helper and
otherwise MEDIUM/LOW; no additional HIGH/CRITICAL symbol was edited. R7-6.1, R7-6.2, and all final
checkpoint items are complete, and `CTX-R7-06` is `PROVEN`. Checkpoint-doc receipt/review-fix series
`0e5150945` + `e634ef324` + `8e39c109e` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-6 exit gate; R7-6 is complete and R7 is closed with a stable analyzer contract.
R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. Fresh independent built-in `default` review of the complete family at
`0ed3d8f04` + `cfcf65507` returned `CHANGES_REQUIRED` with five scoped documentation findings.
Bounded docs-only fix `2b9565fb9` landed. Follow-up fix `b04207fb6` then received fresh
independent built-in `default` `CHANGES_REQUIRED` with one scoped conditional-acceptance finding.
Bounded conditional-acceptance fix `b9ce44c6f` then received fresh independent built-in `default`
`CHANGES_REQUIRED` with two scoped documentation findings. Bounded two-finding docs-only fix
`904c93d0d` then received fresh independent built-in `default` `CHANGES_REQUIRED` with one scoped
Option B call-path finding. This bounded Markdown-only fix addresses only that one finding and claims
no review result. All R8
implementation tasks remain unchecked and unstarted. `CTX-R8-02` is `OPEN` / `REVIEW PENDING` and
not proven; `CTX-R8-03` through `CTX-R8-06` remain `BLOCKED`. R8-4 and `CTX-R8-05` remain
decision-blocked by their future structured gates. R8-IMPLEMENT remains blocked/boundary-only, and
no R8 code has started. No phase transition, Prompt 1 eligibility,
implementation authorization, complete-family `CLEAN`, or review result for this progress receipt
is claimed.

Current R7 authority surfaces:

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
satisfying the R7-4 exit gate. R7-4 is complete. R7-5.1 commit `afb10827d` received fresh independent built-in `default` `CLEAN` with no findings,
and R7-5.2 commit `9c0690a02` received fresh independent built-in `default` `CLEAN` with no
actionable findings. The focused acceptance targets pass `2 / 2`, `8 / 8`, and `4 / 4`; the report
records `115` checkpoints across required strata `23 / 18 / 0 / 23 / 51 / 0`, `3,126` valid
evidence references, zero malformed references, and zero cross-trajectory progress/scorer ownership
violations. At final proof HEAD `9c0690a02`, formatting, compactor/analyzer clippy with `-D warnings`,
full compactor `39 / 39` aggregate, full analyzer `424 / 424` aggregate, and `git diff --check` are
green. R7-5.1, R7-5.2, and the Checkpoint R7-5 behavior items are complete; `CTX-R7-05` is
`PROVEN`. Checkpoint-doc commit `b4916e565cd48f0924fb720d633b57d31b0d624c` received fresh
independent built-in `default` `CLEAN` with no findings, satisfying the R7-5 exit gate. R7-5 is complete. Operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A` authorized only adding `v0.8` to the centralized
explicit-analyzer-state helper, proving explicit posture plus state-backed evidence, preserving
v0.2 and v0.3-v0.7 behavior, and forbidding generalized version parsing or R8 consolidation.
R7-6.1 commit `7789fba4f` received fresh independent built-in `default` `CLEAN`; live compatibility
passes `27 / 27`, replay input `16 / 16`, and operator surface `16 / 16`. R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486` received fresh independent built-in `default` `CLEAN` after
cursor-regression and naming fixes; `real_session_live` passes `12 / 12` and `live_end_to_end`
passes `10 / 10`, proving verified direct closure, per-session cursors, fail-closed unexpected-
session behavior, and unchanged scheduling. Bounded final-wall fix `bd743eacc` received fresh
independent built-in `default` `CLEAN` for the `serde_json` workspace feature-unification test-order
witness. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with `-D warnings`, full
compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full workspace tests,
and `git diff --check` are green. Staged GitNexus gates stayed within the authorized HIGH helper and
otherwise MEDIUM/LOW; no additional HIGH/CRITICAL symbol was edited. R7-6.1, R7-6.2, and all final
checkpoint items are complete, and `CTX-R7-06` is `PROVEN`. Checkpoint-doc receipt/review-fix series
`0e5150945` + `e634ef324` + `8e39c109e` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-6 exit gate; R7-6 is complete and R7 is closed with a stable analyzer contract.
R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. Fresh independent built-in `default` review of the complete family at
`0ed3d8f04` + `cfcf65507` returned `CHANGES_REQUIRED` with five scoped documentation findings.
Bounded docs-only fix `2b9565fb9` landed. Follow-up fix `b04207fb6` then received fresh
independent built-in `default` `CHANGES_REQUIRED` with one scoped conditional-acceptance finding.
Bounded conditional-acceptance fix `b9ce44c6f` then received fresh independent built-in `default`
`CHANGES_REQUIRED` with two scoped documentation findings. Bounded two-finding docs-only fix
`904c93d0d` then received fresh independent built-in `default` `CHANGES_REQUIRED` with one scoped
Option B call-path finding. This bounded Markdown-only fix addresses only that one finding and claims
no review result. All R8
implementation tasks remain unchecked and unstarted. `CTX-R8-02` is `OPEN` / `REVIEW PENDING` and
not proven; `CTX-R8-03` through `CTX-R8-06` remain `BLOCKED`. R8-4 and `CTX-R8-05` remain
decision-blocked by their future structured gates. R8-IMPLEMENT remains blocked/boundary-only, and
no R8 code has started. No phase transition, Prompt 1 eligibility,
implementation authorization, complete-family `CLEAN`, or review result for this progress receipt
is claimed.

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
