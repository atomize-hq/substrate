# Tasks: R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **OPEN / DOCS ONLY** on 2026-07-13. No control, test, fixture, production fix, replay, phase
transition, or closure task is complete. Check boxes change only after exact live proof is recorded.

## Required Staged Commit Gate

Every docs, matrix-row, row-review-fix, gap-fix, and phase-transition commit must stage only its intended
files and run, in order:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

The last command is the required complete staged-diff inspection. Keep unrelated dirt unstaged. Every
matrix row below is one atomic test-only commit plus a fresh built-in `default` review; do not begin the
next row until the current row is review-clean. A red row stays preserved and does not authorize a
production edit during `R6-C.1-CONTROLS`.

## R6-C.1.0 — Specification Lock

- [ ] **R6-C.1.0.1 — Commit and independently review this SPEC/PLAN/TASKS family.**
  - Acceptance: all required controls have exact names, inputs, expected dispositions, fixture choices,
    focused commands, and conditional routes; `CTX-R6-01` through `CTX-R6-16` ownership is honest;
    docs-focused checks and fresh review are clean.
  - Files: the three files in `docs/specs/r6/R6-C.1/` only for the authoring commit.
  - Verify:
    - enumerate exact planned test names across all three docs and compare sets;
    - prove `CTX-R6-03..05`, `CTX-R6-09..16`, and replay-owned `CTX-R6-01/02/06` are represented;
    - verify current command/file paths exist;
    - stage only the three docs and run the required staged commit gate;
    - run docs coverage/cross-reference checks.
  - Commit/review: atomic docs commit, fresh built-in `default` reviewer, docs-only fix commit(s), fresh
    reviewer until clean. The separate transition update may then mark `R6-C.1-SPEC` complete and activate
    `R6-C.1-CONTROLS`; it must not mark controls complete.

## R6-C.1.1 — `dead_end_thrash` Rows

- [ ] **R6-C.1.1.1 — Add, commit, and review the regression row (`CTX-R6-03`).**
  - Test: `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity`.
  - Input lock: failure-only repeated history touches the current interval;
    `active_repeated_failure=true`; no repeated-verification history;
    `active_repeated_verification=false`; troubleshooting-frontier regression/non-advancing evidence; no
    direct frontier-advance signal.
  - Acceptance: exactly `30 / Medium / Active`, flagged, with stall/regression evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-DET-REGRESSION` for later activation.
  - Commit/review: only this test plus result-only TASKS/ledger wording; required staged commit gate; fresh
    built-in `default` review until clean.

- [ ] **R6-C.1.1.2 — Add, commit, and review the opaque-parent row (`CTX-R6-04`).**
  - Test: `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity`.
  - Input lock: opaque parent with no attributable child command observation; no repeated-failure or
    repeated-verification history; both active repetition bits false.
  - Acceptance: exactly `0 / Low / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-DET-OPAQUE-PARENT` for later activation.
  - Commit/review: only this test plus result-only TASKS/ledger wording; required staged commit gate; fresh
    built-in `default` review until clean.

- [ ] **R6-C.1.1.3 — Add, commit, and review the equal-progress row (`CTX-R6-05`).**
  - Test: `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes`.
  - Input lock: long-autonomous and many-short sessions receive identical failure-only repeated history,
    `active_repeated_failure=true`, no repeated-verification history/activity, identical direct frontier
    advancement, and no prior `Active` `dead_end_thrash` score.
  - Acceptance: both exactly `20 / Medium / HistoricalOnly`, unflagged. No prior active state is what makes
    `HistoricalOnly`, rather than `Recovered`, deterministic.
  - Verify: `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_scores_equal_progress_equally_across_turn_shapes -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-DET-TURN-EQUIVALENCE` for later activation.
  - Commit/review: only this test plus result-only TASKS/ledger wording; required staged commit gate; fresh
    built-in `default` review until clean.

- [ ] **R6-C.1.1.4 — Run the reviewed `dead_end_thrash` family checkpoint.**
  - Prerequisite: all three row commits above are independently review-clean.
  - Verify: `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`.
  - Acceptance: aggregate only reviewed row results; do not batch witnesses, edit production, or replace
    any row's commit/review boundary.

## R6-C.1.2 — `truth_grounding_gap` Rows

- [ ] **R6-C.1.2.1 — Add, commit, and review no-action planning (`CTX-R6-09`).**
  - Test: `truth_grounding_gap_keeps_no_action_planning_clear`.
  - Acceptance: declared truth plus no write/verification yields `0 / Medium / Cleared`, unflagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_no_action_planning_clear -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-NO-ACTION` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.2.2 — Add, commit, and review successful ungrounded verification (`CTX-R6-10`).**
  - Test: `truth_grounding_gap_flags_successful_verification_without_truth_reads`.
  - Acceptance: typed success without an earlier truth read yields `80 / High / Active`, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_successful_verification_without_truth_reads -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-SUCCESS-WITHOUT-READ` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.2.3 — Add, commit, and review turn-shape invariance (`CTX-R6-11`).**
  - Test: `truth_grounding_gap_is_event_order_invariant_across_turn_shapes`.
  - Acceptance: equivalent ungrounded event order in long/short turn representations yields
    `80 / High / Active` for both, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_is_event_order_invariant_across_turn_shapes -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-TURN-INVARIANCE` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.2.4 — Add, commit, and review opaque-parent behavior (`CTX-R6-11`).**
  - Test: `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action`.
  - Acceptance: declared truth plus opaque parent and no attributable child action yields
    `0 / Medium / Cleared`, unflagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-OPAQUE-PARENT` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.2.5 — Add, commit, and review truth-path action before read (`CTX-R6-12`).**
  - Test: `truth_grounding_gap_flags_truth_path_action_before_read`.
  - Acceptance: write/verification touching declared truth before any read yields
    `80 / High / Active`, flagged, with authority and action evidence. Current source likely fails; that
    red result is the witness, not permission to change production in `R6-C.1-CONTROLS`.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-TRUTH-PATH-ACTION` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.2.6 — Add, commit, and review archetype equivalence (`CTX-R6-13`).**
  - Test: `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes`.
  - Acceptance: equivalent pre-read actions in planning/research and implementation both yield
    `80 / High / Active`, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-ARCHETYPE-INVARIANCE` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.2.7 — Run the reviewed `truth_grounding_gap` family checkpoint.**
  - Prerequisite: all six row commits above are independently review-clean.
  - Verify: `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`.
  - Acceptance: aggregate only reviewed row results; do not batch witnesses, edit production, or replace
    any row's commit/review boundary.

## R6-C.1.3 — `wrong_plan_branch` Rows

- [ ] **R6-C.1.3.1 — Add, commit, and review read-only exploration (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_ignores_read_only_out_of_scope_exploration`.
  - Acceptance: non-empty authority plus out-of-scope read yields
    `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_ignores_read_only_out_of_scope_exploration -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-READ-ONLY` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.3.2 — Add, commit, and review sanctioned-replan scope (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope`.
  - Acceptance: current authority reflects the sanctioned pivot before a write; result is
    `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_accepts_write_under_sanctioned_replan_scope -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-REPLAN-SCOPE` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.3.3 — Add, commit, and review opaque-parent behavior (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action`.
  - Acceptance: non-empty parent authority plus opaque orchestration and no attributable child action
    yields `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-OPAQUE-PARENT` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.3.4 — Add, commit, and review empty-authority no-claim (`CTX-R6-15`).**
  - Test: `wrong_plan_branch_makes_no_claim_for_path_action_without_authority`.
  - Acceptance: path-bearing write/verification with empty authority yields
    `0 / Low / Cleared`, unflagged, empty evidence. Current source likely returns raw 60; that red result
    is the witness, not permission to change production in `R6-C.1-CONTROLS`.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-EMPTY-AUTHORITY` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.

- [ ] **R6-C.1.3.5 — Run the reviewed `wrong_plan_branch` family checkpoint.**
  - Prerequisite: all four row commits above are independently review-clean.
  - Verify: `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`.
  - Acceptance: aggregate only reviewed row results; do not batch witnesses, edit production, or replace
    any row's commit/review boundary.

## R6-C.1.4 — Controls Wall And Transition

- [ ] **R6-C.1.4.1 — Run the controls wall and preserve every deterministic result.**
  - Prerequisite: all thirteen synthetic matrix-row commits are independently review-clean.
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`
    - `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Acceptance: every synthetic control is PASS or a committed red witness; no result is inferred from
    source or construction proof.

- [ ] **R6-C.1.4.2 — End `R6-C.1-CONTROLS` and activate exactly one next phase.**
  - Acceptance: order red routes by matrix/family order. The narrow transition commit marks
    `R6-C.1-CONTROLS` complete and activates the first named red `R6-GAP-*`; if no row is red, it activates
    `R6-REPLAY`.
  - Verify: required staged commit gate; fresh independent review of the transition commit.
  - Boundary: never leave `R6-C.1-CONTROLS` and any `R6-GAP-*` simultaneously active. Do not execute a
    gap before the transition is committed and review-clean.

- [ ] **R6-C.1.4.3 — Preserve dispatcher adjudication (`CTX-R6-16`).**
  - Acceptance: exact order remains deterministic source/infrastructure proof only. Add no focused order
    test unless later behavior evidence makes order load-bearing. Do not call the family wall order proof.

## R6-C.1.5+ — Conditional Gap Phases

- [ ] **R6-C.1.5.1 — Execute each activated red route as one distinct, sequential phase.**
  - Prerequisite: `R6-C.1-CONTROLS` is complete and exactly one named `R6-GAP-*` is active.
  - Each gap must: use only its preserved witness and owning seam; run GitNexus impact on its owning exact
    symbol (`score_dead_end_thrash`, `score_truth_grounding_gap`, or `score_wrong_plan_branch`) and every
    upstream symbol it would edit; warn/stop on HIGH/CRITICAL; make the smallest fix; run exact witness,
    scorer family, and affected checkpoints; update actual TASKS/ledger proof; run the required staged
    commit gate; commit atomically; fresh review/fix/fresh-review until clean.
  - Boundary: no second gap is active or executed concurrently; no return to `R6-C.1-CONTROLS`.

- [ ] **R6-C.1.5.2 — Transition sequentially between gaps, then to `R6-REPLAY`.**
  - Acceptance: after the active gap is review-clean, a separate narrow transition commit closes it and
    activates the next named red gap in matrix/family order. The final review-clean gap activates
    `R6-REPLAY`.
  - Verify: required staged commit gate and fresh independent review for every gap transition.

## R6-C.1.6 — Replay-Owned Controls, Not Started Here

- [ ] **R6-C.1.6.1 — Select and execute advancing replay (`CTX-R6-01`) in `R6-REPLAY`.**
  - Future exact test: `acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged`.
  - Contract: a trusted annotated real-rollout-derived fixture proves repeated failures plus direct
    frontier advancement; final result is unflagged/non-`Active`. Exact raw score/confidence/state wait for
    selection; `HistoricalOnly` is allowed with no prior active score and `Recovered` only with prior
    `Active` history.
  - Verify: `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged -- --exact --nocapture`.
  - Boundary: reject or replace a fixture-shape mismatch. Do not open a production gap for bad fixture
    selection; only a behavior red after trusted input validation may use `R6-GAP-DET-REPLAY-ADVANCING`.

- [ ] **R6-C.1.6.2 — Select and execute true-stall replay (`CTX-R6-02`) in `R6-REPLAY`.**
  - Future exact test: `acceptance_fixtures_integrated_true_stall_stays_active`.
  - Contract: a trusted annotated real-rollout-derived fixture proves repeated failure with no frontier
    movement; final result is flagged/`Active`. Exact raw score/confidence wait for selection.
  - Verify: `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_true_stall_stays_active -- --exact --nocapture`.
  - Boundary: reject or replace a fixture-shape mismatch. Do not open a production gap for bad fixture
    selection; only a behavior red after trusted input validation may use `R6-GAP-DET-REPLAY-STALL`.

- [ ] **R6-C.1.6.3 — Execute frozen-corpus preservation (`CTX-R6-06`) in `R6-REPLAY`.**
  - Existing exact test:
    `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture`.
  - Acceptance: preserve exactly three `Cleared / 0 / unflagged` final postures and one
    `Recovered / 20 / unflagged` final posture. This is invariance, not comparative improvement.
  - Verify: `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture -- --exact --nocapture`.
  - Ownership: execution and any preservation follow-up remain entirely in `R6-REPLAY`.

## Explicit Exclusions

- [ ] `CTX-R6-07` and `CTX-R6-08` remain preserved semantic completion/integrity proof; no
  `semantic_goal_drift` work occurs absent new failing behavior evidence.
- [ ] No replay-closeout execution occurs in `R6-C.1-SPEC` or `R6-C.1-CONTROLS`.
- [ ] `CTX-R6-17` remains open for `R6-CLOSE`; no R6 terminal disposition or closure is claimed.
- [ ] `CTX-R6-18` historical/superseded labels remain proven and unchanged.
- [ ] No R7 or R8 task starts or absorbs an R6 baseline gap.
