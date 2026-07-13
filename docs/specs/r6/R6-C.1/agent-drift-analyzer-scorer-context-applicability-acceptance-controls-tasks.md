# Tasks: R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **ACTIVE — R6-C.1-SPEC COMPLETE; R6-C.1-CONTROLS ACTIVE** on 2026-07-13. The
specification-lock task and `CTX-R6-03` control are complete. All remaining controls, replay,
conditional-gap, production-fix, phase-close, and closure tasks remain open until exact live proof is recorded.

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

## Required Phase-Transition Authority Manifest

Every `SPEC -> CONTROLS`, `CONTROLS -> first GAP or REPLAY`, and
`GAP -> next GAP or REPLAY` task must reconcile these exact active-phase mirrors in one transition commit:

| Surface | Fields/checks that must move together |
|---|---|
| R6-C.1 SPEC | Update this packet SPEC header `Status`. At `SPEC -> CONTROLS`: `APPROVED / LANDED — R6-C.1-SPEC COMPLETE; R6-C.1-CONTROLS ACTIVE`. At `CONTROLS -> successor`: `APPROVED / LANDED — R6-C.1-CONTROLS COMPLETE; <successor> ACTIVE`. At later gap transitions: `APPROVED / LANDED — R6-C.1-CONTROLS COMPLETE; <completed-gap> COMPLETE; <successor> ACTIVE`. Do not rewrite decisions merely to move status. |
| R6-C.1 PLAN | Update this packet PLAN header `Status` to the same exact phase-aware value as the SPEC; do not rewrite plan decisions merely to move status. |
| R6-C.1 TASKS | At `SPEC -> CONTROLS`, check the docs-lock task, record its review-clean commit, and set this header to `ACTIVE — R6-C.1-SPEC COMPLETE; R6-C.1-CONTROLS ACTIVE`. Later use `HANDOFF TRACKING — R6-C.1-CONTROLS COMPLETE; <completed-predecessor> COMPLETE; <successor> ACTIVE`, omitting the completed-gap clause when CONTROLS is the predecessor. Check only actually proven work and record the sole next action. |
| Active gap packet docs, conditional | Entry into a named gap records the three canonical packet paths as non-link `TO CREATE` strings and authorizes only atomic docs creation/review. Do not cite or update a nonexistent gap TASKS. Once the docs gate is review-clean, the gap SPEC/PLAN/TASKS become required proof and exit-transition surfaces. |
| Control pack `00-README.md` | `Current work phase`; `Last repo-truth verification`. |
| Control pack `01-authority-and-status-map.md` | `Verified against`; `Current phase`; R6 `Current Status` row `Status` and `Next allowed action`. |
| Control pack `02-phase-and-gate-map.md` | Concrete predecessor/successor statuses and changed gates; exactly one concrete phase `ACTIVE`. Its generic `R6-GAP-*` row links to the named-gap subledger and carries only aggregate `CONDITIONAL` before route instantiation, `ACTIVE` while one named gap is active, or `COMPLETE` when no gap is required/all are complete. It never stores individual gap status. |
| Control pack `04-reusable-phase-runner.md` | At `SPEC -> CONTROLS`, reconcile `VERIFICATION AND COMMIT` to stage only intended files, run `npx gitnexus detect-changes --scope staged -r 97a0-substrate`, then run `git diff --cached --check` and inspect the complete `git diff --cached` before commit. Remove the unscoped `detect-changes` and working-tree `git diff --check` commit-gate prescriptions. |
| Control pack `05-proof-decision-regression-ledger.md` | `Ledger status`; `Verified against`; actually changed `CTX-R6-*` cells; `Update Record`; and a `Named R6 Gap Status Subledger`. At controls transition instantiate one ordered row per red route with columns `Phase ID`, `Preserved witness/control`, `Owning packet docs`, `Status`, `Predecessor`, `Successor`, `Evidence/commit`; use non-link `TO CREATE` paths until packet docs land. Every gap transition updates the subledger atomically. Do not churn unaffected row evidence. |
| Control pack `06-operator-prompt-library.md` | `Current First Invocation` sentence, `PHASE_ID`, `ACTIVE_PACKET`. |
| Root `SPEC.md`, `tasks/plan.md`, `tasks/todo.md` | `Status`; `Current phase`; completed/current task wording; sole next-authorized action/check box. At `SPEC -> CONTROLS`, replace the root `tasks/plan.md` GitNexus execution rule with the same staged-only sequence: stage intended files, run `npx gitnexus detect-changes --scope staged -r 97a0-substrate`, then `git diff --cached --check` and complete `git diff --cached` inspection. |
| Conditional canonical status/proof | Update `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`, `docs/specs/r6/MAP.md`, and `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` only when actual control/gap evidence changes their proof, status, or next-action wording. |

At `SPEC -> CONTROLS`, inspect only the other active-phase mirrors already named in this manifest for
commit-gate wording. If any still prescribes unscoped `detect-changes`, working-tree `git diff --check`,
or an unstaged diff as the commit gate, reconcile it to the same staged-only sequence in the transition
commit. Do not broaden this bounded check into inactive or historical documentation cleanup.

Transition result locks: `SPEC -> CONTROLS` updates the R6-C.1 SPEC/PLAN headers to the prescribed
`APPROVED / LANDED` value, checks TASKS docs-lock completion, and records its review-clean commit without
inventing control results; the same transition commit reconciles the reusable runner, root
`tasks/plan.md`, and any other stale active mirror found by the bounded manifest check to the staged-only
commit gate. `CONTROLS -> GAP/REPLAY` updates all three R6-C.1 status surfaces, records
every row result, creates every named-gap subledger entry, activates the first red route with later routes
`BLOCKED`, or activates replay; `GAP -> GAP/REPLAY` requires the active gap's review-clean packet docs
plus committed, fresh-review-clean fix or no-code receipt before atomically marking it `COMPLETE` and the
next route `ACTIVE`, or activating replay. A newly active gap authorizes only its docs gate. Do not churn
unchanged semantic authority, but every active-phase
mirror above must agree on the active phase, completed predecessor, verification commit, and next action.
No successor work starts until the reconciled transition commit passes the staged gate and a fresh
independent review says `REVIEW CLEAN`. `R6-C.1-CONTROLS` cannot activate and no control work may start
before that reconciled transition is committed and fresh-review-clean.

## R6-C.1.0 — Specification Lock

- [x] **R6-C.1.0.1 — Commit and independently review this SPEC/PLAN/TASKS family.**
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
    `R6-C.1-CONTROLS`; it must apply the Required Phase-Transition Authority Manifest, receive its own
    fresh `REVIEW CLEAN` before control work starts, and must not mark controls complete.

  - Result (2026-07-13): the artifact commit series `253e634fe`, `d3430eff3`, `58535df60`,
    `12f042f6b`, and `ea19b39a7` landed the three packet docs and their review fixes. The final fresh
    independent review of `ea19b39a7` returned `REVIEW CLEAN`. Docs coverage and cross-link checks were
    `PASS`. Targeted scorer/checkpoint checks were `PASS` for
    `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`,
    `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`,
    `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`, and
    `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`. The full
    `cargo test -p agent-drift-analyzer -- --nocapture` suite was `PASS`. No test, source, or fixture
    changed in that artifact series.
  - Sole next action: execute `R6-C.1.1.1`, the first row-atomic `R6-C.1-CONTROLS` acceptance control.
    Do not start another row, replay, a gap packet, or a production change first.

## R6-C.1.1 — `dead_end_thrash` Rows

- [x] **R6-C.1.1.1 — Add, commit, and review the regression row (`CTX-R6-03`).**
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
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 14
    filtered out`. The locked failure-only/current-repetition seam produced exactly
    `30 / Medium / Active`, flagged. Its `TroubleshootingFrontier` was `Regressing`, carried
    `PreviouslyCleanScopeBroken`, carried no direct frontier-advance signal, and named both the frontier
    fallback and the current repeated-failure evidence. No `R6-GAP-DET-REGRESSION` route is required.

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
    `R6-REPLAY`. When red routes exist, the same transition instantiates the `Named R6 Gap Status
    Subledger` in control-pack `05-proof-decision-regression-ledger.md`, one row per route with phase ID,
    exact preserved witness/control, canonical owning packet-doc paths, status, predecessor/successor,
    and evidence/commit. Only the first is `ACTIVE`, with CONTROLS `COMPLETE`; every later gap is
    `BLOCKED`. The generic `R6-GAP-*` map row links to this subledger and reports aggregate `ACTIVE` only;
    with no red routes it reports `COMPLETE` and no named rows are created.
  - Packet status: update this SPEC and PLAN headers plus this TASKS header/checks exactly as required by
    the manifest. The newly active gap's packet paths are non-link `TO CREATE` strings, and its sole next
    action is atomic docs creation/review; do not cite a nonexistent gap TASKS.
  - Verify: apply every field in the Required Phase-Transition Authority Manifest; required staged commit
    gate; fresh independent review of the transition commit.
  - Boundary: never leave `R6-C.1-CONTROLS` and any `R6-GAP-*` simultaneously active. Do not execute a
    gap before the transition is committed and review-clean.

- [ ] **R6-C.1.4.3 — Preserve dispatcher adjudication (`CTX-R6-16`).**
  - Acceptance: exact order remains deterministic source/infrastructure proof only. Add no focused order
    test unless later behavior evidence makes order load-bearing. Do not call the family wall order proof.

## R6-C.1.5+ — Conditional Gap Phases

- [ ] **R6-C.1.5.1 — Create and review the active gap's scorer-specific packet docs first.**
  - Prerequisite: `R6-C.1-CONTROLS` is complete and exactly one named `R6-GAP-*` is active.
  - Files: create exactly
    `docs/specs/r6/R6-GAP-<SCORER>-<SEAM>/R6-GAP-<SCORER>-<SEAM>-spec.md`,
    `docs/specs/r6/R6-GAP-<SCORER>-<SEAM>/R6-GAP-<SCORER>-<SEAM>-plan.md`, and
    `docs/specs/r6/R6-GAP-<SCORER>-<SEAM>/R6-GAP-<SCORER>-<SEAM>-tasks.md` in one atomic docs commit.
  - Required content: the exact preserved witness/control; exact owning symbol and GitNexus impact
    command; minimal allowed file set; production-fix acceptance and no-code proof criteria; exact
    focused witness command; owning scorer-family and affected checkpoint walls; and exact transition to
    the next named gap or `R6-REPLAY`.
  - Commit/review: stage all three docs together, run the required staged gate, commit atomically, and
    dispatch a fresh built-in `default` reviewer. Keep fixes docs-only and fresh-review until clean. Then
    replace the named-gap subledger's non-link `TO CREATE` path markers with the actual packet-doc paths
    and review-clean commit.
  - Boundary: this is the first and only authorized task after activation. No production edit, witness
    rerun for a no-code receipt, or proof receipt may begin until these docs are committed and
    independently review-clean. The no-code path does not bypass this task.

- [ ] **R6-C.1.5.2 — Execute the review-clean active gap as one distinct phase.**
  - Prerequisite: R6-C.1.5.1 is review-clean for this exact named gap; its SPEC/PLAN/TASKS now exist; the
    named-gap subledger still shows exactly this gap `ACTIVE` and later gaps `BLOCKED`.
  - Code-changing path, only while the witness remains red: use only its preserved witness and owning
    seam; run GitNexus impact on its owning exact symbol (`score_dead_end_thrash`,
    `score_truth_grounding_gap`, or `score_wrong_plan_branch`) and every upstream symbol it would edit;
    warn/stop on HIGH/CRITICAL; make the smallest fix; run exact witness, scorer family, and affected
    checkpoints; update the now-existing gap TASKS, named-gap subledger entry, and actual ledger proof;
    run the required staged commit gate; commit atomically; fresh review/fix/fresh-review until clean.
  - No-code alternative: if an earlier sequential gap commit already made this active gap's exact
    preserved witness green, do not touch production. Rerun the exact row test and owning scorer-family
    controls; record actual output and the earlier causal commit in the now-existing active gap
    TASKS/proof receipt, named-gap subledger entry, and only the corresponding changed ledger evidence;
    stage only those result/ledger docs; run the required staged commit gate; commit the no-code proof
    receipt/status reconciliation; fresh built-in `default` review until clean. Preserve the distinct
    named gap and original witness; never silently delete, merge, or relabel them. The receipt commit
    leaves the gap active and activates no successor; the normal transition remains separate.
  - Boundary: no second gap is active or executed concurrently; no return to `R6-C.1-CONTROLS`.

- [ ] **R6-C.1.5.3 — Transition sequentially between gaps, then to `R6-REPLAY`.**
  - Acceptance: after the active gap fix or no-code proof receipt is committed and review-clean, a
    separate narrow transition commit closes it and activates the next named red gap in matrix/family
    order. In the named-gap subledger, atomically mark the predecessor `COMPLETE`, only its immediate
    successor `ACTIVE`, and all later rows `BLOCKED`; the successor authorizes only R6-C.1.5.1. The final
    review-clean gap activates `R6-REPLAY` and makes the generic phase-map gap row `COMPLETE`.
  - Packet status: update this SPEC and PLAN header statuses plus this TASKS header/checks with the exact
    phase-aware manifest wording; update the now-existing completed gap SPEC/PLAN/TASKS as required by
    that packet. Do not reference the successor gap TASKS until R6-C.1.5.1 creates it.
  - Verify: apply every field in the Required Phase-Transition Authority Manifest; required staged commit
    gate and fresh independent review for every gap transition.

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
