# Tasks: R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **HANDOFF TRACKING — R6-C.1-CONTROLS COMPLETE; R6-REPLAY ACTIVE; ACTIVE PACKET NONE; CTX-R6-02 AND R6-GAP-DET-REPLAY-STALL COMPLETE AT `6eda87e60` FRESH INDEPENDENT REVIEW-CLEAN; TRANSITION REVIEW PENDING** on 2026-07-14. The
specification-lock task, all thirteen synthetic controls, their reviewed family checkpoints, the
controls wall, and source-only `CTX-R6-16` dispatcher adjudication are complete. The controls wall
preserved exactly three named reds: `CTX-R6-04`, `CTX-R6-12`, and `CTX-R6-15`, requiring
`R6-GAP-DET-OPAQUE-PARENT`, `R6-GAP-TGG-TRUTH-PATH-ACTION`, and
`R6-GAP-WPB-EMPTY-AUTHORITY`, respectively. The first route is complete after the fresh built-in
`default` `REVIEW CLEAN` verdict for production series `bcd94bf4f` + `931e50c85` + `d13f0a71c`.
The second route is complete after final proof-receipt series `fee9c2b16` + `6674a8316` received
fresh independent built-in `default` `REVIEW CLEAN`. Transition series `2937dbe5a` + `91f55f6bf`
also received fresh independent built-in `default` `REVIEW CLEAN`, completing the second route and
activating the final gap. That route is now complete after implementation/review-fix series
`6b42e5476` + `e65df2561` + `cd4e24119` received fresh independent built-in `default` `REVIEW
CLEAN`. Authority transition series `56bb9966f` + `07a3b1fe5` also received fresh independent
built-in `default` `REVIEW CLEAN` and activates only `R6-REPLAY` with active packet `none`.
`CTX-R6-01` implementation/fix series `a0089c8de` + `968a4377f` is also fresh independent built-in
`default` `REVIEW CLEAN`. Historical `CTX-R6-02` witness `60cde3dd7` remains preserved. Commit
`6eda87e60` completes `CTX-R6-02` and `R6-GAP-DET-REPLAY-STALL`, passes the complete ordered packet
proof and full analyzer `402 / 402`, and received fresh independent built-in `default` `REVIEW CLEAN`.
Active packet is `none`; this transition awaits fresh review. Sticky authority remains
`HistoricalOnly / 20`, unflagged; old `Recovered / 20` remains historical baseline only. After
transition review-clean, `CTX-R6-06` replay proof and the R6 family wall are next. `R6-CLOSE` and
R7/R8 remain pending or blocked as owned.

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
  - Historical next action after the docs-lock result: execute `R6-C.1.4.2`, the narrow
    controls-to-first-gap transition. That transition is now represented by the checked task below.

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

- [x] **R6-C.1.1.2 — Add, commit, and review the opaque-parent row (`CTX-R6-04`).**
  - Test: `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity`.
  - Input lock: opaque parent with no attributable child command observation; no repeated-failure or
    repeated-verification history; both active repetition bits false.
  - Acceptance: exactly `0 / Low / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-DET-OPAQUE-PARENT` for later activation.
  - Commit/review: only this test plus result-only TASKS/ledger wording; required staged commit gate; fresh
    built-in `default` review until clean.
  - Result (2026-07-13): `FAIL — PRESERVED RED`. The exact focused command completed with `0 passed;
    1 failed; 15 filtered out`. The locked no-child-activity seam reached
    `ParentVisibleOrchestration` and produced `0 / Medium / Cleared`, unflagged, with empty evidence,
    rather than the required `0 / Low / Cleared`. The confidence-only mismatch requires
    `R6-GAP-DET-OPAQUE-PARENT`; no production code changed in `R6-C.1-CONTROLS`.

- [x] **R6-C.1.1.3 — Add, commit, and review the equal-progress row (`CTX-R6-05`).**
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
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 16
    filtered out`. The long-autonomous and many-short-conversational sessions reached their locked
    execution modes while producing identical advancing `TroubleshootingFrontier` progress with direct
    `FailureFrontierAdvanced` evidence. Their failure-only repeated history produced equal
    `20 / Medium / HistoricalOnly` scores, unflagged, with no repeated-verification evidence. No
    `R6-GAP-DET-TURN-EQUIVALENCE` route is required.

- [x] **R6-C.1.1.4 — Run the reviewed `dead_end_thrash` family checkpoint.**
  - Prerequisite: all three row commits above are independently review-clean.
  - Verify: `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`.
  - Acceptance: aggregate only reviewed row results; do not batch witnesses, edit production, or replace
    any row's commit/review boundary.
  - Result (2026-07-13): `COMPLETE — DETERMINISTIC REVIEWED-FAMILY AGGREGATION`. The exact command
    exited `101`. Its aggregate output included the acceptance fixture with `1 passed` and the
    `dead_end_thrash` test binary with `16 passed; 1 failed`. The sole failure was the already-preserved
    `CTX-R6-04` confidence mismatch: actual `0 / Medium / Cleared`, unflagged, with empty evidence,
    versus expected `0 / Low / Cleared`. `CTX-R6-03` and `CTX-R6-05` passed. No new witness was found,
    no production code changed, and `R6-C.1-CONTROLS` remains active.

## R6-C.1.2 — `truth_grounding_gap` Rows

- [x] **R6-C.1.2.1 — Add, commit, and review no-action planning (`CTX-R6-09`).**
  - Test: `truth_grounding_gap_keeps_no_action_planning_clear`.
  - Acceptance: declared truth plus no write/verification yields `0 / Medium / Cleared`, unflagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_no_action_planning_clear -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-NO-ACTION` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 4
    filtered out`. The declared truth path plus planning/research prose and no write-like or
    verification-like command produced exactly `0 / Medium / Cleared`, unflagged. The score retained
    only `truth artifact hint:` authority evidence and no action-gap evidence. No
    `R6-GAP-TGG-NO-ACTION` route is required, and no production code changed.

- [x] **R6-C.1.2.2 — Add, commit, and review successful ungrounded verification (`CTX-R6-10`).**
  - Test: `truth_grounding_gap_flags_successful_verification_without_truth_reads`.
  - Acceptance: typed success without an earlier truth read yields `80 / High / Active`, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_successful_verification_without_truth_reads -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-SUCCESS-WITHOUT-READ` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 5
    filtered out`. A declared truth path, a typed successful `cargo test` result outside that path, and
    no earlier truth read produced exactly `80 / High / Active`, flagged. Evidence retained both the
    `truth artifact hint:` authority and the ungrounded `command family: cargo` verification action. No
    `R6-GAP-TGG-SUCCESS-WITHOUT-READ` route is required, and no production code changed.

- [x] **R6-C.1.2.3 — Add, commit, and review turn-shape invariance (`CTX-R6-11`).**
  - Test: `truth_grounding_gap_is_event_order_invariant_across_turn_shapes`.
  - Acceptance: equivalent ungrounded event order in long/short turn representations yields
    `80 / High / Active` for both, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_is_event_order_invariant_across_turn_shapes -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-TURN-INVARIANCE` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 6
    filtered out`. Identical declared truth and event order reached the locked long-autonomous and
    many-short-conversational execution modes while producing exactly `80 / High / Active`, flagged,
    in both representations. Their evidence reasons were equivalent and retained both the
    `truth artifact hint:` authority and the ungrounded `command family: cargo` verification action. No
    `R6-GAP-TGG-TURN-INVARIANCE` route is required, and no production code changed.

- [x] **R6-C.1.2.4 — Add, commit, and review opaque-parent behavior (`CTX-R6-11`).**
  - Test: `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action`.
  - Acceptance: declared truth plus opaque parent and no attributable child action yields
    `0 / Medium / Cleared`, unflagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-OPAQUE-PARENT` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 7
    filtered out`. A declared truth path plus opaque parent-visible orchestration and no attributable
    child write or verification produced exactly `0 / Medium / Cleared`, unflagged. The score retained
    only `truth artifact hint:` authority evidence. No `R6-GAP-TGG-OPAQUE-PARENT` route is required,
    and no production code changed.

- [x] **R6-C.1.2.5 — Add, commit, and review truth-path action before read (`CTX-R6-12`).**
  - Test: `truth_grounding_gap_flags_truth_path_action_before_read`.
  - Controls-wall acceptance: write/verification touching declared truth before any read yields
    `80 / High / Active`, flagged, with authority and action evidence. At controls-wall capture, any red
    result was a witness, not permission to change production in `R6-C.1-CONTROLS`.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-TRUTH-PATH-ACTION` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `FAIL — PRESERVED RED`. The exact focused command completed with `0 passed;
    1 failed; 8 filtered out`. A declared truth path and the first write-like `apply_patch` action
    touching that same path, with no earlier read, produced actual `0 / Medium / Cleared`, unflagged,
    rather than the required `80 / High / Active`, flagged, with authority and action evidence. This
    witness requires `R6-GAP-TGG-TRUTH-PATH-ACTION`; no production code changed in
    `R6-C.1-CONTROLS`.
  - Later resolution (2026-07-14): `R6-GAP-TGG-TRUTH-PATH-ACTION` resolved the preserved witness, and
    review-clean final proof-receipt series `fee9c2b16` + `6674a8316` records the exact control passing
    at current source. This does not rewrite the historical controls-wall result above.

- [x] **R6-C.1.2.6 — Add, commit, and review archetype equivalence (`CTX-R6-13`).**
  - Test: `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes`.
  - Acceptance: equivalent pre-read actions in planning/research and implementation both yield
    `80 / High / Active`, flagged.
  - Verify: `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-TGG-ARCHETYPE-INVARIANCE` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 9
    filtered out`. Task prose alone produced Planning and AutonomousImplementation frames that each
    declared the same truth path and performed one byte-identical write-like `apply_patch` command
    against the same target before any truth read; both produced exactly `80 / High / Active`, flagged.
    Their evidence reasons were equivalent and retained both the `truth artifact hint:` authority and
    ungrounded `command family: apply_patch` action. No
    `R6-GAP-TGG-ARCHETYPE-INVARIANCE` route is required, and no production code changed.

- [x] **R6-C.1.2.7 — Run the reviewed `truth_grounding_gap` family checkpoint.**
  - Prerequisite: all six row commits above are independently review-clean.
  - Verify: `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`.
  - Acceptance: aggregate only reviewed row results; do not batch witnesses, edit production, or replace
    any row's commit/review boundary.
  - Result (2026-07-13): deterministic reviewed aggregation completed. The exact family command exited
    `101`; the `truth_grounding_gap` test binary reported `9 passed; 1 failed`. The sole failure was the
    already-preserved `CTX-R6-12` witness: actual `0 / Medium / Cleared`, unflagged, versus required
    `80 / High / Active`, flagged. All other family controls passed. No new witness was found, no
    production code changed, and `R6-C.1-CONTROLS` remains active.

## R6-C.1.3 — `wrong_plan_branch` Rows

- [x] **R6-C.1.3.1 — Add, commit, and review read-only exploration (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_ignores_read_only_out_of_scope_exploration`.
  - Acceptance: non-empty authority plus out-of-scope read yields
    `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_ignores_read_only_out_of_scope_exploration -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-READ-ONLY` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 2
    filtered out`. A non-empty task frame declared the authorized source and truth paths, while a
    read-only `sed` command named a different path. The score produced exactly
    `0 / Medium / Cleared`, unflagged, with empty evidence. No `R6-GAP-WPB-READ-ONLY` route is required,
    and no production code changed.

- [x] **R6-C.1.3.2 — Add, commit, and review sanctioned-replan scope (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope`.
  - Acceptance: current authority reflects the sanctioned pivot before a write; result is
    `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_accepts_write_under_sanctioned_replan_scope -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-REPLAN-SCOPE` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 3
    filtered out`. A user steer explicitly sanctioned a path pivot and updated current truth/working-set
    authority before an `apply_patch` write under the new path. The score produced exactly
    `0 / Medium / Cleared`, unflagged, with empty evidence. No `R6-GAP-WPB-REPLAN-SCOPE` route is
    required, and no production code changed.

- [x] **R6-C.1.3.3 — Add, commit, and review opaque-parent behavior (`CTX-R6-14`).**
  - Test: `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action`.
  - Acceptance: non-empty parent authority plus opaque orchestration and no attributable child action
    yields `0 / Medium / Cleared`, unflagged, empty evidence.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-OPAQUE-PARENT` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `PASS`. The exact focused command completed with `1 passed; 0 failed; 4
    filtered out`. A non-empty parent task frame declared source/truth authority, opaque delegated-parent
    orchestration resolved to `ParentVisibleOrchestration`, and no attributable child path-bearing
    write/verification was present. The score produced exactly `0 / Medium / Cleared`, unflagged, with
    empty evidence. No `R6-GAP-WPB-OPAQUE-PARENT` route is required, and no production code changed.

- [x] **R6-C.1.3.4 — Add, commit, and review empty-authority no-claim (`CTX-R6-15`).**
  - Test: `wrong_plan_branch_makes_no_claim_for_path_action_without_authority`.
  - Acceptance: path-bearing write/verification with empty authority yields
    `0 / Low / Cleared`, unflagged, empty evidence. Current source likely returns raw 60; that red result
    is the witness, not permission to change production in `R6-C.1-CONTROLS`.
  - Verify: `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture`.
  - If red: preserve witness and record only `R6-GAP-WPB-EMPTY-AUTHORITY` for later activation.
  - Commit/review: one-row test-only commit; required staged commit gate; fresh built-in `default` review
    until clean.
  - Result (2026-07-13): `FAIL — PRESERVED RED`. The exact focused command completed with `0 passed;
    1 failed; 5 filtered out`. Empty truth-artifact authority plus one working-set path sourced only from
    the observed command and one write-like `apply_patch` action produced actual
    `60 / Low / Active`, flagged, rather than the required `0 / Low / Cleared`, unflagged, with empty
    evidence. This witness requires `R6-GAP-WPB-EMPTY-AUTHORITY`; no production code changed in
    `R6-C.1-CONTROLS`.

- [x] **R6-C.1.3.5 — Run the reviewed `wrong_plan_branch` family checkpoint.**
  - Prerequisite: all four row commits above are independently review-clean.
  - Verify: `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`.
  - Acceptance: aggregate only reviewed row results; do not batch witnesses, edit production, or replace
    any row's commit/review boundary.
  - Result (2026-07-13): deterministic reviewed aggregation completed. The exact family command exited
    `101`; the `wrong_plan_branch` test binary reported `5 passed; 1 failed`. The sole failure was the
    already-preserved `CTX-R6-15` witness: actual `60 / Low / Active`, flagged, versus required
    `0 / Low / Cleared`, unflagged. All other family controls passed. No new witness was found, no
    production code changed, and `R6-C.1-CONTROLS` remains active.

## R6-C.1.4 — Controls Wall And Transition

- [x] **R6-C.1.4.1 — Run the controls wall and preserve every deterministic result.**
  - Prerequisite: all thirteen synthetic matrix-row commits are independently review-clean.
  - Verify:
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`
    - `cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture`
    - `cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture`
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Acceptance: every synthetic control is PASS or a committed red witness; no result is inferred from
    source or construction proof.
  - Result (2026-07-13): `COMPLETE — EXPECTED PRESERVED REDS ONLY`. The exact `dead_end_thrash`
    command exited `101`; the acceptance fixture reported `1 passed`, and the family test binary
    reported `16 passed; 1 failed`, with only the already-preserved `CTX-R6-04` red. The exact
    `truth_grounding_gap` command exited `101` with `9 passed; 1 failed`, with only the
    already-preserved `CTX-R6-12` red. The exact `wrong_plan_branch` command exited `101` with
    `5 passed; 1 failed`, with only the already-preserved `CTX-R6-15` red. The exact `checkpoints`
    command exited `0`; all matching tests passed across unit (`35 passed`), integration (`131
    passed`), and export-bundle match (`1 passed`) coverage. No new witness was found and no
    production code changed.

- [x] **R6-C.1.4.2 — End `R6-C.1-CONTROLS` and activate exactly one next phase.**
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
  - Result (2026-07-13): the deterministic controls wall at `5618f7864` reconciled all thirteen
    synthetic controls as `10 PASS / 3 preserved RED`, with no production change. In matrix order,
    witness `87409b39a` routes `CTX-R6-04` to active `R6-GAP-DET-OPAQUE-PARENT`; witness
    `e67d8b214` routes `CTX-R6-12` to blocked `R6-GAP-TGG-TRUTH-PATH-ACTION`; and witness
    `59f098b35` routes `CTX-R6-15` to blocked `R6-GAP-WPB-EMPTY-AUTHORITY`. This transition update
    records those statuses and the required non-link `TO CREATE` paths without claiming an unknown
    transition commit or review verdict. The sole next authorized action is atomic creation and fresh
    review of the active gap's three canonical docs; do not execute the gap first.

- [x] **R6-C.1.4.3 — Preserve dispatcher adjudication (`CTX-R6-16`).**
  - Acceptance: exact order remains deterministic source/infrastructure proof only. Add no focused order
    test unless later behavior evidence makes order load-bearing. Do not call the family wall order proof.
  - Result (2026-07-13): `COMPLETE — SOURCE-ONLY ADJUDICATION PRESERVED`. The explicit dispatcher
    sort in `crates/agent-drift-analyzer/src/scoring/mod.rs` still deterministically orders
    `WrongPlanBranch`, `TruthGroundingGap`, `DeadEndThrash`, then `SemanticGoalDrift`. This remains
    source/infrastructure proof only; no focused dispatcher-order test was added, and the family wall is
    not treated as order proof.

## R6-C.1.5+ — Conditional Gap Phases

- [x] **R6-C.1.5.1 — Create and review the active gap's scorer-specific packet docs first.**
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
  - Sequential result through 2026-07-14: **COMPLETE FOR ALL THREE NAMED GAPS**.
    `R6-GAP-DET-OPAQUE-PARENT` completed its packet-docs gate
    at `59092df2a` and ledger reconciliation at `beed76446`; `R6-GAP-TGG-TRUTH-PATH-ACTION`
    completed its packet-docs gate at `03754a2de` and ledger reconciliation at `a5380c04e`; all four
    gates received fresh `REVIEW CLEAN`. Transition series `2937dbe5a` + `91f55f6bf` then received
    fresh independent built-in `default` `REVIEW CLEAN`. Active successor
    `R6-GAP-WPB-EMPTY-AUTHORITY` then landed packet-doc series `8734f4dbe` + `334e7c6ac`, which
    received fresh independent built-in `default` `REVIEW CLEAN`. Its packet-gate authority series
    `eb24b59da` + `e7006f4b2` also received fresh independent built-in `default` `REVIEW CLEAN`.
    All three scorer-specific packet-doc and packet-gate requirements are therefore complete.

- [x] **R6-C.1.5.2 — Execute the review-clean active gap as one distinct phase.**
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
  - Sequential result through 2026-07-14: the first gap instance completed execution. Production series
    `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN` with
    `CTX-R6-04` at `0 / Low / Cleared`, unflagged, empty evidence; the new partial/mixed regression at
    `0 / Medium / Cleared`, unflagged, empty evidence; `CTX-R6-03` preserved at
    `30 / Medium / Active`, flagged; all `21` matching `dead_end_thrash` tests and all `167` matching
    checkpoint tests passing; and format/check passing. The second gap instance also completed:
    Option-A implementation series `4ba9f2647` + `1f6e863bf` + `9565fb805`, source commit
    `5622ddb73`, and final proof-receipt series `fee9c2b16` + `6674a8316` received fresh independent
    built-in `default` `REVIEW CLEAN`; packet-locked grounding controls passed `9 / 9`, full
    `truth_grounding_gap` passed `22 / 22`, exact dead-end controls passed `3 / 3`, full
    `dead_end_thrash` passed `18 / 18`, checkpoints passed `35` unit + `131` integration plus
    matching export/provenance tests, and format/check/literal-clippy/diff passed. The final gap
    instance completed through implementation/review-fix series `6b42e5476` + `e65df2561` +
    `cd4e24119`, fresh independent built-in `default` `REVIEW CLEAN`: exact `CTX-R6-15` and four
    protected controls passed `1 / 1`, full `wrong_plan_branch` passed `6 / 6`, checkpoints passed
    `35` unit + `131` integration plus matching export/truth tests, and the full analyzer plus
    format/check/literal-clippy/diff walls passed. All three named-gap execution instances are complete.

- [x] **R6-C.1.5.3 — Transition sequentially between gaps, then to `R6-REPLAY`.**
  - Acceptance: after the active gap fix or no-code proof receipt is committed and review-clean, a
    separate narrow transition commit closes it and activates the next named red gap in matrix/family
    order. In the named-gap subledger, atomically mark the predecessor `COMPLETE`, only its immediate
    successor `ACTIVE`, and all later rows `BLOCKED`; the successor authorizes only R6-C.1.5.1. The final
    review-clean gap activates `R6-REPLAY` and makes the generic phase-map gap row `COMPLETE`.
  - Packet status: update this SPEC and PLAN header statuses plus this TASKS header/checks with the exact
    phase-aware manifest wording; update the now-existing completed gap SPEC/PLAN/TASKS as required by
    that packet. Record only the successor TASKS path as a non-link `TO CREATE` string until
    R6-C.1.5.1 creates it; do not cite that TASKS as existing.
  - Verify: apply every field in the Required Phase-Transition Authority Manifest; required staged commit
    gate and fresh independent review for every gap transition.
  - Sequential result (2026-07-14): authority-only transition series `2937dbe5a` + `91f55f6bf`
    received fresh independent built-in `default` `REVIEW CLEAN`, marks
    `R6-GAP-TGG-TRUTH-PATH-ACTION` complete, activates only `R6-GAP-WPB-EMPTY-AUTHORITY` at its
    docs-only packet-creation gate with active packet `none`, and kept `R6-REPLAY` blocked. The
    final authority transition series `56bb9966f` + `07a3b1fe5` marks
    `R6-GAP-WPB-EMPTY-AUTHORITY` and aggregate `R6-GAP-*` complete and activates only `R6-REPLAY`
    with active packet `none`. The first fresh independent built-in `default` reviewer returned one
    P2 finding for a stale historical Step 1 gate description in the final gap PLAN; `07a3b1fe5`
    corrected it, and a fresh independent built-in `default` reviewer returned `REVIEW CLEAN` for
    the complete transition series. **Historical transition-boundary receipt:** at the
    `56bb9966f` + `07a3b1fe5` boundary, Prompt 1 for `R6-REPLAY` with active packet `none` was the
    sole next eligible invocation, and replay work had not yet started. Current replay state is
    recorded below: `CTX-R6-01` and `CTX-R6-02` are complete; historical witness `60cde3dd7` is
    preserved; implementation/proof commit `6eda87e60` and its full ordered proof are fresh
    independent `REVIEW CLEAN`; active packet is `none`; this transition awaits fresh review before
    `CTX-R6-06` replay proof and the family wall.

## R6-C.1.6 — Replay-Owned Controls, Handoff Status

- [x] **R6-C.1.6.1 — Select and execute advancing replay (`CTX-R6-01`) in `R6-REPLAY`.**
  - Exact test: `acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged`.
  - Contract: a trusted annotated real-rollout-derived fixture proves repeated failures plus direct
    frontier advancement; final result is unflagged/non-`Active`. Exact raw score/confidence/state wait for
    selection; `HistoricalOnly` is allowed with no prior active score and `Recovered` only with prior
    `Active` history.
  - Verify: `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged -- --exact --nocapture`.
  - Boundary: reject or replace a fixture-shape mismatch. Do not open a production gap for bad fixture
    selection; only a behavior red after trusted input validation may use `R6-GAP-DET-REPLAY-ADVANCING`.
  - Result (2026-07-14): implementation/fix series `a0089c8de` + `968a4377f` received fresh
    independent built-in `default` `REVIEW CLEAN`. Trusted real rollout
    `019f1ecb-b93a-7570-8d8d-9ce4e711880b` retains identical failing verifier calls at compact
    events `164` and `183` with exit-`101` outputs at `165` and `184`. Full-analyzer checkpoint `2`
    exposes `FailureSignatureRepeated`; checkpoint `7` is `TroubleshootingFrontier / Advancing`
    with `VerificationClean` and `VerificationScopeBroadened`. Its `dead_end_thrash` result is
    `HistoricalOnly / 20 / High`, unflagged, with no earlier `Active` score.

- [x] **R6-C.1.6.2 — Resolve true-stall replay (`CTX-R6-02`) in `R6-REPLAY`.**
  - Exact test: `acceptance_fixtures_integrated_true_stall_stays_active`.
  - Trusted witness: depth-1 built-in `default` subagent rollout
    `019eb311-c7ce-7f50-ae13-b51a5b5461c3`; checkpoint `5` is
    `TroubleshootingFrontier / Stalled / Medium`, exposes `FailureSignatureRepeated`, and scores
    flagged `Active / 30 / High`.
  - Preserved RED (2026-07-14): commit `60cde3dd7`; the analyzer misattributes truthful failed calls
    `420`/`474` to successful siblings `421`/`475`. The input/progress/score statement was correct
    only for the committed baseline; an uncommitted truthful-pairing candidate exposes a separate
    `InsufficientEvidence` progress red.
  - Authoritative screen: `3,423` total (`1,112` user, `2,298` subagent, `13` unspecified), `1,611`
    verifier-bearing, `225` with at least two failures regardless of command, `161` subagent,
    `189` non-identical failed-command candidates, `0` malformed, `224` analyzed; this was the sole
    exact match.
  - Result: implementation/proof commit `6eda87e60` makes exact `CTX-R6-02` green at the locked
    `TroubleshootingFrontier / Stalled / Medium` and flagged `Active / 30 / High` posture with
    truthful target attribution. The complete ordered packet wall and full analyzer `402 / 402` pass,
    and a fresh independent built-in `default` reviewer returned `REVIEW CLEAN`. Packet is complete;
    active packet is `none`.
  - Verify: `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_true_stall_stays_active -- --exact --nocapture`.
  - Boundary: no terminal disposition, `R6-CLOSE`, or R7/R8 work yet. After this authority
    transition is fresh-review-clean, `CTX-R6-06` and the family wall are next.

- [ ] **R6-C.1.6.3 — Execute frozen-corpus preservation (`CTX-R6-06`) in `R6-REPLAY`.**
  - Existing exact test:
    `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture`.
  - Acceptance authority: preserve exactly three `Cleared / 0 / unflagged` final postures and one
    `HistoricalOnly / 20 / unflagged` sticky posture. The old `Recovered / 20 / unflagged` result is
    historical clean-baseline evidence only. This is invariance, not comparative improvement.
  - Current candidate red: clean `f898d61e7` transitions checkpoint `9`
    `Regressing / Active 40` to checkpoint `10` `Recovered 20`; truthful pairing instead yields
    checkpoint `9` `Advancing / HistoricalOnly 20` then checkpoint `10` `HistoricalOnly 20`.
    Expected-negative event `831 -> 837` and `recovery_state` are non-causal. Task `.2B` Option A
    resolved expected authority. Task `.3` must complete source/test/helper/expected-disposition
    preparation and focused TDD unit red/green; Task `.4` must then complete integrated proof, result
    recording, staging/gates, atomic commit, and fresh review before this replay-owned control runs.
  - Verify: `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture -- --exact --nocapture`.
  - Ownership: execution and any preservation follow-up remain entirely in `R6-REPLAY`.

## Explicit Exclusions

- [ ] `CTX-R6-07` and `CTX-R6-08` remain preserved semantic completion/integrity proof; no
  `semantic_goal_drift` work occurs absent new failing behavior evidence.
- [ ] No replay-closeout execution occurs in `R6-C.1-SPEC` or `R6-C.1-CONTROLS`.
- [ ] `CTX-R6-17` remains open for `R6-CLOSE`; no R6 terminal disposition or closure is claimed.
- [ ] `CTX-R6-18` historical/superseded labels remain proven and unchanged.
- [ ] No R7 or R8 task starts or absorbs an R6 baseline gap.
