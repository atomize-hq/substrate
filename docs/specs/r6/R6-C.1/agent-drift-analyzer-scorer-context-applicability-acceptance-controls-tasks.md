# Tasks: R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **OPEN / DOCS ONLY** on 2026-07-13. No control, test, fixture, production fix, replay, or
closure task is complete. Check boxes change only after exact live proof is recorded.

## R6-C.1.0 — Specification Lock

- [ ] **R6-C.1.0.1 — Commit and independently review this SPEC/PLAN/TASKS family.**
  - Acceptance: all required controls have exact names, inputs, expected dispositions, fixture choices,
    focused commands, and conditional gap routes; `CTX-R6-01` through `CTX-R6-16` ownership is honest;
    docs-focused checks and fresh review are clean.
  - Files: the three files in `docs/specs/r6/R6-C.1/` only for the authoring commit.
  - Verify:
    - enumerate exact planned test names across all three docs and compare sets;
    - prove `CTX-R6-03..05`, `CTX-R6-09..16`, and replay-owned `CTX-R6-01/02/06` are represented;
    - verify current command/file paths exist;
    - `npx gitnexus detect-changes -r 97a0-substrate`;
    - `git diff --check`.
  - Commit/review: atomic docs commit, fresh built-in `default` reviewer, fix commit(s), fresh reviewer
    until clean. The separate transition update may then mark `R6-C.1-SPEC` complete and activate
    `R6-C.1-CONTROLS`; it must not mark controls complete.

## R6-C.1.1 — `dead_end_thrash` Controls

- [ ] **R6-C.1.1.1 — Add the scorer-level regression control (`CTX-R6-03`).**
  - Test: `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity`.
  - Acceptance: regression/no direct advance plus active repeated failure yields
    `30 / Medium / Active`, flagged, with stall/regression evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-DET-REGRESSION`.

- [ ] **R6-C.1.1.2 — Add the opaque-parent control (`CTX-R6-04`).**
  - Test: `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity`.
  - Acceptance: opaque parent with no attributable child activity yields
    `0 / Low / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-DET-OPAQUE-PARENT`.

- [ ] **R6-C.1.1.3 — Add the turn-shape/equal-progress control (`CTX-R6-05`).**
  - Test: `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes`.
  - Acceptance: long-autonomous and many-short sessions with equal repeated-failure/progress inputs both
    yield `20 / Medium / HistoricalOnly`, unflagged; record the charter narrowing to upstream
    archetype/progress consumption.
  - Verify: `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_scores_equal_progress_equally_across_turn_shapes -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-DET-TURN-EQUIVALENCE`.

- [ ] **R6-C.1.1.4 — Commit and review the dead-end controls/witnesses atomically.**
  - Files: `crates/agent-drift-analyzer/tests/dead_end_thrash.rs` plus result-only TASKS/ledger wording.
  - Verify: `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`;
    `npx gitnexus detect-changes -r 97a0-substrate`; `git diff --check`.
  - Boundary: test-only; no production edit. Fresh reviewer; fixes in a new commit and fresh review.

## R6-C.1.2 — `truth_grounding_gap` Controls

- [ ] **R6-C.1.2.1 — Add no-action planning control (`CTX-R6-09`).**
  - Test: `truth_grounding_gap_keeps_no_action_planning_clear`.
  - Acceptance: declared truth plus no write/verification yields `0 / Medium / Cleared`, unflagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_no_action_planning_clear -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-TGG-NO-ACTION`.

- [ ] **R6-C.1.2.2 — Add successful-but-ungrounded verification control (`CTX-R6-10`).**
  - Test: `truth_grounding_gap_flags_successful_verification_without_truth_reads`.
  - Acceptance: typed success without an earlier truth read yields `80 / High / Active`, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_successful_verification_without_truth_reads -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-TGG-SUCCESS-WITHOUT-READ`.

- [ ] **R6-C.1.2.3 — Add long-turn invariance control (`CTX-R6-11`).**
  - Test: `truth_grounding_gap_is_event_order_invariant_across_turn_shapes`.
  - Acceptance: equivalent ungrounded event order in long/short turn representations yields
    `80 / High / Active` for both, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_is_event_order_invariant_across_turn_shapes -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-TGG-TURN-INVARIANCE`.

- [ ] **R6-C.1.2.4 — Add opaque-parent control (`CTX-R6-11`).**
  - Test: `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action`.
  - Acceptance: declared truth plus opaque parent and no attributable child action yields
    `0 / Medium / Cleared`, unflagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-TGG-OPAQUE-PARENT`.

- [ ] **R6-C.1.2.5 — Add truth-path action-before-read control (`CTX-R6-12`).**
  - Test: `truth_grounding_gap_flags_truth_path_action_before_read`.
  - Acceptance: write/verification touching declared truth before any read yields
    `80 / High / Active`, flagged, with authority and action evidence. Current source likely fails; that
    red result is the required witness, not permission to change production in this task.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-TGG-TRUTH-PATH-ACTION`.

- [ ] **R6-C.1.2.6 — Add actionful archetype-equivalence control (`CTX-R6-13`).**
  - Test: `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes`.
  - Acceptance: equivalent pre-read actions in planning/research and implementation both yield
    `80 / High / Active`, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-TGG-ARCHETYPE-INVARIANCE`.

- [ ] **R6-C.1.2.7 — Commit and review truth controls/witnesses atomically.**
  - Files: `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs` plus result-only TASKS/ledger wording.
  - Verify: `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`;
    `npx gitnexus detect-changes -r 97a0-substrate`; `git diff --check`.
  - Boundary: test-only; no production edit. Fresh reviewer; fixes in a new commit and fresh review.

## R6-C.1.3 — `wrong_plan_branch` Controls

- [ ] **R6-C.1.3.1 — Add read-only exploration control (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_ignores_read_only_out_of_scope_exploration`.
  - Acceptance: non-empty authority plus out-of-scope read yields
    `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_ignores_read_only_out_of_scope_exploration -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-WPB-READ-ONLY`.

- [ ] **R6-C.1.3.2 — Add sanctioned-replan scope control (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope`.
  - Acceptance: current authority reflects the sanctioned pivot before a write; result is
    `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_accepts_write_under_sanctioned_replan_scope -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-WPB-REPLAN-SCOPE`.

- [ ] **R6-C.1.3.3 — Add opaque-parent control (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action`.
  - Acceptance: non-empty parent authority plus opaque orchestration and no attributable child action
    yields `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-WPB-OPAQUE-PARENT`.

- [ ] **R6-C.1.3.4 — Add empty-authority no-claim control (`CTX-R6-15`).**
  - Test: `wrong_plan_branch_makes_no_claim_for_path_action_without_authority`.
  - Acceptance: path-bearing write/verification with empty authority yields
    `0 / Low / Cleared`, unflagged, empty evidence. Current source likely returns raw 60; that red result
    is the required witness, not permission to change production here.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture`.
  - Gap if red: preserve witness; open only `R6-GAP-WPB-EMPTY-AUTHORITY`.

- [ ] **R6-C.1.3.5 — Commit and review wrong-branch controls/witnesses atomically.**
  - Files: `crates/agent-drift-analyzer/tests/wrong_plan_branch.rs` plus result-only TASKS/ledger wording.
  - Verify: `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`;
    `npx gitnexus detect-changes -r 97a0-substrate`; `git diff --check`.
  - Boundary: test-only; no production edit. Fresh reviewer; fixes in a new commit and fresh review.

## R6-C.1.4 — Packet Checkpoint And Conditional Gap Routing

- [ ] **R6-C.1.4.1 — Run the controls family wall and record deterministic results.**
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`
    - `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `git diff --check`
  - Acceptance: every control is PASS or a committed red witness; no result is inferred from current
    source or a construction test.

- [ ] **R6-C.1.4.2 — Open one bounded packet for every red scorer/failure seam.**
  - Acceptance: each failing matrix row gets only its named `R6-GAP-*` packet; packets are executed
    sequentially; no production fix begins before its witness commit/review boundary.
  - Each gap must: run GitNexus impact on its owning exact symbol (`score_dead_end_thrash`,
    `score_truth_grounding_gap`, or `score_wrong_plan_branch`) and separately on any upstream symbol it
    would edit; stop/warn on HIGH/CRITICAL; make the smallest fix; run exact witness then scorer family;
    update actual TASKS/ledger proof; run
    `npx gitnexus detect-changes -r 97a0-substrate` and `git diff --check`; commit atomically; fresh
    review/fix/fresh-review until clean.

- [ ] **R6-C.1.4.3 — Record dispatcher adjudication (`CTX-R6-16`).**
  - Acceptance: keep exact order as deterministic source/infrastructure proof only. Add no focused order
    test unless later behavior evidence makes order load-bearing. Do not call the family wall order proof.
  - Verify: docs/ledger consistency review; no production/test file is needed for this row.

## R6-C.1.5 — Controls Exit And Next Routing

- [ ] **R6-C.1.5.1 — Re-run the family wall after conditional fixes and reconcile actual results.**
  - Acceptance: all controls are deterministic; every conditional gap is review-clean; canonical TASKS,
    proof ledger, and status wording agree without assigning terminal scorer dispositions.
  - Verify: the four family commands above; `npx gitnexus detect-changes -r 97a0-substrate`;
    `git diff --check`; fresh independent review of the transition commit.

- [ ] **R6-C.1.5.2 — Route, but do not start, `R6-REPLAY`.**
  - `CTX-R6-01`: future exact test
    `acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged`, trusted annotated
    real-rollout-derived fixture, expected `20 / Medium / HistoricalOnly`, unflagged.
  - `CTX-R6-02`: future exact test `acceptance_fixtures_integrated_true_stall_stays_active`, trusted
    annotated real-rollout-derived fixture, expected `30 / Medium / Active`, flagged.
  - `CTX-R6-06`: preserve the frozen four-case posture as invariance only.
  - Acceptance: replay remains open and owned by `R6-REPLAY`; no fixture is selected or edited here.

## Explicit Exclusions

- [ ] `CTX-R6-07` and `CTX-R6-08` remain preserved semantic completion/integrity proof; no
  `semantic_goal_drift` work occurs absent new failing behavior evidence.
- [ ] No replay-closeout execution occurs in this packet.
- [ ] `CTX-R6-17` remains open for `R6-CLOSE`; no R6 terminal disposition or closure is claimed.
- [ ] `CTX-R6-18` historical/superseded labels remain proven and unchanged.
- [ ] No R7 or R8 task starts or absorbs an R6 baseline gap.
