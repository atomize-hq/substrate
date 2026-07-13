# Plan: R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **DRAFT / DOCS ONLY** on 2026-07-13. This plan is ordered and test-first. It does not
authorize control implementation until the `R6-C.1-SPEC` docs lock is committed and review-clean.

## Plan Decisions

1. Land the docs lock first; then implement controls in scorer-family order:
   `dead_end_thrash`, `truth_grounding_gap`, `wrong_plan_branch`.
2. Within each family, every matrix row is one atomic test-only commit and one fresh independent review
   boundary. Do not batch green controls or red witnesses by family. A later family checkpoint may
   aggregate results only after every row commit in that family is review-clean.
3. A red control remains preserved in its row commit. Production does not change while
   `R6-C.1-CONTROLS` is active.
4. `R6-C.1-CONTROLS` ends as soon as every synthetic control has a deterministic preserved PASS or red
   result. Its transition commit activates the first named red `R6-GAP-*` in matrix/family order, or
   `R6-REPLAY` when no row is red.
5. Every red row is a distinct gap phase. Gap phases run sequentially: a review-clean gap transitions to
   the next named red gap, and the final review-clean gap transitions to `R6-REPLAY`. Never execute a gap
   while `R6-C.1-CONTROLS` remains active.
6. The two likely red semantics are already adjudicated by scorer responsibility: truth-path action
   before read is `80 / High / Active`; empty-authority wrong-branch is no-claim
   `0 / Low / Cleared / empty evidence`. Current source behavior does not override these contracts.
7. Synthetic `dead_end_thrash` exact triples must control the upstream bits that determine them:
   failure-only versus verification history, active repetition bits, direct frontier advancement, and
   prior `dead_end_thrash` state. No test may infer `HistoricalOnly` where prior `Active` history would
   resolve to `Recovered`.
8. Future integrated advancing/true-stall fixtures defer exact raw score, confidence, and historical state
   until trusted fixture selection. They require, respectively, unflagged/non-`Active` and flagged/`Active`
   contract behavior. Reject a fixture-shape mismatch; do not route it to a production gap.
9. `CTX-R6-06` is an executable preservation control owned by `R6-REPLAY`: run existing test
   `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture` and preserve three
   cleared final postures plus one recovered final posture.
10. Dispatcher exact ordering is source-proven infrastructure, not a closure behavior contract. No
    focused ordering test is planned.
11. Before every commit, stage only intended files and lock GitNexus and review to that staged diff:
    `npx gitnexus detect-changes --scope staged -r 97a0-substrate`, then
    `git diff --cached --check`, then complete staged-diff inspection.

## Dependency Graph

```text
R6-C.1-SPEC docs lock + fresh review
  -> R6-C.1-CONTROLS
       -> CTX-R6-03 row commit + fresh review
       -> CTX-R6-04 row commit + fresh review
       -> CTX-R6-05 row commit + fresh review
       -> dead_end_thrash family checkpoint
       -> CTX-R6-09..13 rows, one commit + fresh review each
       -> truth_grounding_gap family checkpoint
       -> CTX-R6-14..15 rows, one commit + fresh review each
       -> wrong_plan_branch family checkpoint
       -> controls wall + deterministic result reconciliation
       -> transition commit ends CONTROLS
            -> no red rows: activate R6-REPLAY
            -> red rows: activate first named R6-GAP-* phase
                 -> fix + fresh review
                 -> transition to next named R6-GAP-* phase
                 -> ...
                 -> final review-clean gap transitions to R6-REPLAY
  -> R6-REPLAY owns CTX-R6-01 / CTX-R6-02 / CTX-R6-06
```

## Staged Commit Gate

Use this gate for the docs lock, every row commit or row-scoped review fix, every gap fix, and every
phase-transition commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

The final command is a complete staged-diff inspection, not a working-tree approximation. Unrelated dirt
stays unstaged. Do not substitute unscoped `detect-changes`, `git diff --check`, or an unstaged diff for a
commit gate.

## Ordered Execution

### R6-C.1.0 — Docs Lock

- Stage and commit only the three files in this directory.
- Validate all named tests, all `CTX` IDs, current source/test paths, commands, and cross-document test
  inventory.
- Run the staged commit gate above and dispatch a fresh built-in `default` reviewer.
- Fix docs findings in a new docs-only commit and repeat with a fresh reviewer until clean.
- Transition authority may activate `R6-C.1-CONTROLS` only after that review-clean docs commit.

**Atomic boundary:** one docs commit; no root/control-pack status edit is part of the authoring batch.

### Common Row Protocol — `R6-C.1-CONTROLS`

For each matrix row, and for no other row in the same commit:

1. Load only the landed packet docs, the owning scorer/test file, the smallest required upstream builder,
   and one analogous existing test.
2. Add the exact named test and assert the row's complete deterministic input seam and expected public
   contract.
3. Run the exact focused command from the SPEC. Record PASS or the exact red output in TASKS and the
   corresponding ledger row; never infer a result from source.
4. Stage only that test plus result-only TASKS/ledger wording. Run the staged commit gate. Commit the row
   atomically as test-only evidence.
5. Dispatch a fresh built-in `default` reviewer. Keep review fixes row-scoped, gate them as staged diffs,
   and dispatch a fresh reviewer until the row is clean.
6. Only then begin the next matrix row.

**Red rule:** preserve the red witness and its named route. Do not edit production or start the gap while
`R6-C.1-CONTROLS` is active.

### R6-C.1.1 — `dead_end_thrash` Acceptance Rows

Execute the common row protocol in this order:

1. `CTX-R6-03` — `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity`.
   Build failure-only history that touches the current interval, set active repeated failure, exclude all
   repeated-verification history/activity, and supply regression/non-advancing frontier evidence with no
   direct advancement. Assert `30 / Medium / Active`, flagged.
2. `CTX-R6-04` —
   `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity`.
   Supply no attributable child command observations, no repetition history, and no active repetition.
   Assert `0 / Low / Cleared`, unflagged, empty evidence.
3. `CTX-R6-05` — `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes`.
   Give both shapes identical failure-only repeated history, active repeated failure, no verification
   history/activity, identical direct frontier advancement, and no prior `Active` score for this class.
   Assert both are `20 / Medium / HistoricalOnly`, unflagged.

After all three row commits are review-clean, run the family checkpoint:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
```

This checkpoint may aggregate reviewed row results. It cannot replace or combine their commits/reviews.

### R6-C.1.2 — `truth_grounding_gap` Acceptance Rows

Execute the common row protocol in this order:

1. `CTX-R6-09` — `truth_grounding_gap_keeps_no_action_planning_clear`.
2. `CTX-R6-10` — `truth_grounding_gap_flags_successful_verification_without_truth_reads`.
3. `CTX-R6-11` — `truth_grounding_gap_is_event_order_invariant_across_turn_shapes`.
4. `CTX-R6-11` —
   `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action`.
5. `CTX-R6-12` — `truth_grounding_gap_flags_truth_path_action_before_read`.
6. `CTX-R6-13` — `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes`.

After all six row commits are review-clean, run the family checkpoint:

```bash
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
```

Preserve any red result, including the likely current-source conflict for truth-path action before read.
The checkpoint may aggregate reviewed results only; it authorizes no production edit.

### R6-C.1.3 — `wrong_plan_branch` Acceptance Rows

Execute the common row protocol in this order:

1. `CTX-R6-14` — `wrong_plan_branch_ignores_read_only_out_of_scope_exploration`.
2. `CTX-R6-14` — `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope`.
3. `CTX-R6-14` —
   `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action`.
4. `CTX-R6-15` — `wrong_plan_branch_makes_no_claim_for_path_action_without_authority`.

After all four row commits are review-clean, run the family checkpoint:

```bash
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
```

Preserve any red result, including the likely current-source conflict for empty authority. The checkpoint
may aggregate reviewed results only; it authorizes no production edit.

### R6-C.1.4 — Controls Wall And Phase Transition

After all thirteen synthetic row commits are independently review-clean, run:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

- Reconcile every row to deterministic PASS or preserved red. A red witness does not keep
  `R6-C.1-CONTROLS` active after this reconciliation.
- Order the red routes exactly by matrix/family order.
- Make a narrow transition commit that marks `R6-C.1-CONTROLS` complete and activates the first named
  red `R6-GAP-*`. If there is no red row, activate `R6-REPLAY` instead.
- Run the staged commit gate and a fresh independent transition review.
- Do not run any gap until this transition is committed and review-clean.

### R6-C.1.5+ — Conditional `R6-GAP-*` Phases, Sequential

For the one named gap that is active:

1. Use only its preserved witness, owning scorer, directly relevant upstream context, and one regression
   pattern. Exclude R7 and unrelated scorers.
2. Before editing, run only the active scorer command below plus a separate exact-symbol impact command
   for every directly relevant upstream function the gap proposes to edit:

```bash
npx gitnexus impact score_dead_end_thrash -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_wrong_plan_branch -r 97a0-substrate --direction upstream --depth 3
```

3. Warn and stop on HIGH/CRITICAL impact. Otherwise make the smallest production change that turns the
   preserved witness green without weakening other controls.
4. Run the exact witness, owning scorer family, and affected checkpoint commands.
5. Update only the active gap TASKS and corresponding `CTX-R6-*` proof with actual output.
6. Run the staged commit gate, commit the gap fix atomically, and dispatch a fresh built-in `default`
   reviewer. Fix findings in a new gap-scoped commit and repeat fresh review until clean.
7. In a separate staged and reviewed transition commit, close this gap and activate the next named red gap.
   When this is the final gap, activate `R6-REPLAY`.

**No batching and no overlap:** one scorer/failure seam is one active phase. `R6-C.1-CONTROLS` is already
complete and is never active alongside a gap.

### R6-C.1.6 — `R6-REPLAY` Handoff, Not Execution Here

`R6-REPLAY` owns all three replay controls:

- `CTX-R6-01` — select and annotate a trusted real-rollout-derived advancing repeated-failure fixture;
  require unflagged/non-`Active`, with `HistoricalOnly` or `Recovered` determined only by prior score
  history. Defer raw score and confidence until selection.
- `CTX-R6-02` — select and annotate a trusted real-rollout-derived true-stall fixture; require
  flagged/`Active`. Defer raw score and confidence until selection.
- `CTX-R6-06` — run the existing frozen-corpus preservation control:

```bash
cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture -- --exact --nocapture
```

The exact expected posture is three `Cleared / 0 / unflagged` cases and one
`Recovered / 20 / unflagged` case. Fixture-shape mismatches for `CTX-R6-01`/`02` are rejected or
replaced in replay selection; they do not create a production gap.

## Ledger And Status Update Points

| Boundary | Allowed updates |
|---|---|
| Docs lock | These three docs; phase transition separately marks `R6-C.1-SPEC` complete and `R6-C.1-CONTROLS` active. No control result changes. |
| Matrix-row commit | Only the one test row, its TASKS result, and its `CTX-R6-*` result wording justified by exact output. Fresh review before the next row. |
| Family checkpoint | Aggregate only review-clean row results; no batched witness commit or production edit. |
| Controls transition | Mark `R6-C.1-CONTROLS` complete; activate first named red gap, or `R6-REPLAY` if none is red. |
| Active gap | Owning gap docs/code/control row and actual proof receipts only. No other gap and no terminal scorer disposition. |
| Gap transition | Close only the review-clean active gap; activate the next named gap or, after the final gap, `R6-REPLAY`. |
| Replay / close | `CTX-R6-01`/`02`/`06` remain `R6-REPLAY`; terminal scorer disposition remains `R6-CLOSE`. |

## Full R6 Ledger Coverage

| IDs | Planned disposition |
|---|---|
| `CTX-R6-01`, `CTX-R6-02` | Trusted integrated controls execute only in `R6-REPLAY`; fixture-dependent precision remains deferred. |
| `CTX-R6-03` through `CTX-R6-05` | Row-atomic `dead_end_thrash` controls execute in `R6-C.1-CONTROLS`. |
| `CTX-R6-06` | Existing frozen-corpus preservation control executes only in `R6-REPLAY`. |
| `CTX-R6-07`, `CTX-R6-08` | Preserve semantic scorer completion and fixture-integrity/live-path distinction; no `semantic_goal_drift` reopening. |
| `CTX-R6-09` through `CTX-R6-13` | Row-atomic `truth_grounding_gap` controls execute in `R6-C.1-CONTROLS`. |
| `CTX-R6-14`, `CTX-R6-15` | Row-atomic `wrong_plan_branch` controls execute in `R6-C.1-CONTROLS`. |
| `CTX-R6-16` | Source-only dispatcher ordering proof; no focused behavior test. |
| `CTX-R6-17` | Terminal scorer table remains open for `R6-CLOSE`. |
| `CTX-R6-18` | Preserve proven historical/superseded labels. |

## Exact Planned Test Inventory

```text
dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity
dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity
dead_end_thrash_scores_equal_progress_equally_across_turn_shapes
truth_grounding_gap_keeps_no_action_planning_clear
truth_grounding_gap_flags_successful_verification_without_truth_reads
truth_grounding_gap_is_event_order_invariant_across_turn_shapes
truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action
truth_grounding_gap_flags_truth_path_action_before_read
truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes
wrong_plan_branch_ignores_read_only_out_of_scope_exploration
wrong_plan_branch_accepts_write_under_sanctioned_replan_scope
wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action
wrong_plan_branch_makes_no_claim_for_path_action_without_authority
acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged
acceptance_fixtures_integrated_true_stall_stays_active
acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture
```

The first thirteen names belong to `R6-C.1-CONTROLS`. The last three are specified here but owned and
executed by `R6-REPLAY`; `CTX-R6-06` already exists and is rerun as preservation proof.

## Risks And Mitigations

| Risk | Mitigation |
|---|---|
| Current behavior is mistaken for desired semantics. | The SPEC locks responsibility-derived expectations before tests are written. |
| A failing control prompts an immediate production patch. | Commit/review its red witness, finish CONTROLS, then activate its distinct gap by transition. |
| Several witnesses or fixes are batched. | One atomic row commit/review and one distinct gap phase per matrix row; retain family/matrix order. |
| A dead-end exact triple varies unexpectedly. | Pin failure/verification history, active repetition bits, frontier signals, and prior class state. |
| A replay candidate has the wrong shape. | Reject or replace it during trusted fixture selection; do not open a gap for fixture mismatch. |
| R7 absorbs an R6 baseline gap. | Gap routes remain `R6-GAP-*`; R7 stays blocked until R6 closes. |
| Source order is overclaimed as behavior proof. | Keep `CTX-R6-16` source-only; no order test absent load-bearing evidence. |
| GitNexus reports unrelated scope. | Stage intended files first and run staged-scope detection plus cached diff checks. |

## Stop And Escalation Rules

Stop only the affected scope and use the operator library's structured escalation when authority cannot
choose between materially different product meanings, GitNexus is HIGH/CRITICAL, unrelated work cannot
be isolated, a trusted replay artifact is unavailable, or review proves the packet boundary invalid.
Ordinary red controls, bounded fixes, tests, commits, and review findings are autonomous work. Do not
ask for routine approval.

## Non-Goals

No code/test/fixture change in `R6-C.1-SPEC`; no replay closeout; no R6 closure; no R7/R8 work; no
`semantic_goal_drift`; no dispatcher-order contract.
