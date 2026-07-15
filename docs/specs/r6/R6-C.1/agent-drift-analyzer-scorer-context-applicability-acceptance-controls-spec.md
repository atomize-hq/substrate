# R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **APPROVED / LANDED — R6-C.1-CONTROLS COMPLETE; R6-REPLAY ACTIVE / PROOF COMPLETE / PROOF-RECEIPT REVIEW PENDING; ACTIVE PACKET NONE; PACKET TRANSITION `1ff592823` + `7839a7f47` FRESH INDEPENDENT REVIEW-CLEAN; CTX-R6-01/02/06 AND R6 FAMILY WALL GREEN** on 2026-07-14. Historical `CTX-R6-02` witness `60cde3dd7` remains preserved. Phase-owned exact replay controls pass `4 x 1 / 1`; family filters pass `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`; full analyzer passes `402 / 402`; diff check is green. Sticky `CTX-R6-06` current authority remains `HistoricalOnly / 20`, unflagged; old `Recovered / 20` is historical baseline only. The narrow `R6-CLOSE` transition is next only after this proof receipt is fresh-review-clean; R7 remains blocked.
All synthetic controls are complete in the TASKS ledger. The three preserved reds, `CTX-R6-04`,
`CTX-R6-12`, and `CTX-R6-15`, and their named-gap phases are complete. Their focused dispositions do
not assign terminal scorer categories; those remain reserved for `R6-CLOSE` after replay.

Authority order: the corrected
[`R6` closure finding](../FINDINGS-r6-scorer-context-cutover-closure.md) owns the scorer applicability
and remaining-gap judgment; [`R6/MAP.md`](../MAP.md) and
[`DESIGN-r6-scorer-cutover-and-objective-consumption.md`](../DESIGN-r6-scorer-cutover-and-objective-consumption.md)
own the family boundary; the
[`R6-R8` phase map](../../hybrid-drift-r6-r8-control-pack/02-phase-and-gate-map.md) and
[`proof ledger`](../../hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md) own phase
routing and open `CTX-R6-*` rows; this family owns the acceptance-control contract. Live source shows
current behavior, but current behavior is not authority for an unresolved semantic decision.

## Objective

Close the named scorer-context **acceptance-proof** gaps without manufacturing common context inputs.
Add deterministic behavior controls first for `dead_end_thrash`, `truth_grounding_gap`, and
`wrong_plan_branch`; preserve every honest failure as a red witness; and route each failing seam to one
bounded `R6-GAP-*` packet before any production change. `R6-C.1-CONTROLS` succeeds when every listed
synthetic control has a deterministic recorded pass or preserved fail result. Its transition commit
then ends `R6-C.1-CONTROLS` and activates the first named red `R6-GAP-*` phase, or `R6-REPLAY` when
none is red. It does not close R6.

## Locked Decisions

1. **Grounding is provenance, not outcome quality.** A write or verification touching a declared truth
   path before any read is still ungrounded. The expected result is `80 / High / Active`, flagged, with
   authority and action evidence. At controls-wall capture, source excluded that action from both the
   grounded-read and ungrounded-action buckets and produced the preserved red witness. That historical
   exclusion was not authority: path coincidence could not let the action that creates the obligation
   satisfy the obligation.
   The later `R6-GAP-TGG-TRUTH-PATH-ACTION` implementation and review-clean proof resolved the source
   mismatch while preserving the contract: `truth_grounding_gap` owns whether declared truth was read
   **before** action; success, path coincidence, progress, turn shape, and archetype do not establish prior
   grounding.
2. **No authority means no wrong-branch claim.** A path-bearing write or verification with empty truth
   and working-set authority expects `0 / Low / Cleared`, unflagged, with empty evidence. The live scorer
   can emit `60 / Low / Active`; that likely produces an honest red witness. It is not authority because
   `wrong_plan_branch` owns proof that an action is outside a known authorized scope. An empty comparison
   set supplies uncertainty, not evidence of the wrong branch.
3. **Turn shape is not independent scorer evidence.** For `dead_end_thrash`, turn context and archetype
   matter only through `SessionProgress`. Equal scorer inputs must score equally; a material difference is
   required only when upstream progress differs. For `truth_grounding_gap`, event order governs the
   read-before-action obligation, so equivalent events remain invariant across long and short turns and
   across planning/implementation archetypes.
4. **Opaque parents do not imply child misconduct.** With no attributable child write/verification or
   repeated-failure activity, parent orchestration stays clear. This is an R6 baseline control, not R7
   child-link implementation.
5. **Dispatcher exact order is not an R6 closure contract (`CTX-R6-16`).** The explicit source sort in
   `scoring/mod.rs` remains deterministic infrastructure/source proof. The analyzer wall proves live-path
   carriage, not exact order. Add no focused ordering test unless later behavior evidence makes ordering
   load-bearing.
6. **Integrated replay stays in `R6-REPLAY`.** The real-rollout-derived advancing repeated-failure and
   true-stall controls below are specified now but are not implemented or closed by `R6-C.1-CONTROLS`.
   Trusted fixture selection owns their exact raw score, confidence, and historical state. The advancing
   contract is unflagged/non-`Active`; it may be `HistoricalOnly` or `Recovered` only as prior score
   history determines. The true-stall contract is flagged/`Active`. A candidate that does not satisfy the
   annotated row shape is rejected or replaced as a fixture mismatch, not routed to a production gap.
7. **Frozen-corpus preservation is executable but replay-owned (`CTX-R6-06`).** `R6-REPLAY` must run
   existing test
   `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture` and preserve three
   `Cleared / 0 / unflagged` cases plus one `HistoricalOnly / 20 / unflagged` sticky case. The prior
   `Recovered / 20 / unflagged` result is historical baseline only, not current authority. This remains
   posture invariance, not comparative integrated improvement.
8. **Phase transitions are exclusive.** No `R6-GAP-*` phase runs while `R6-C.1-CONTROLS` remains
   active. Every red matrix row gets a distinct named gap phase. Gap phases transition sequentially in
   matrix/family order; the final review-clean gap transitions to `R6-REPLAY`.
9. **A later gap may close with proof instead of production churn.** If an earlier sequential gap commit
   also turns a later gap's exact preserved witness green, the later gap still activates in order and owns
   a no-code proof receipt. Rerun its exact row and family controls, cite the earlier causal commit, commit
   and freshly review the result/ledger reconciliation, and only then perform the normal transition. Do
   not edit production unnecessarily, or silently delete, merge, or relabel the named gap or witness.
10. **Named gaps have first-class status, not an overloaded wildcard status.** The
    `CONTROLS -> first GAP` transition creates a `Named R6 Gap Status Subledger` in control-pack
    `05-proof-decision-regression-ledger.md` with one row per preserved red route in matrix/family order.
    Every row records phase ID, exact witness/control, owning packet-doc paths, status,
    predecessor/successor, and evidence/commit. The first row alone is `ACTIVE`, its predecessor is
    `COMPLETE`, and every later row is `BLOCKED`; each later transition atomically makes the old active
    row `COMPLETE` and only its immediate successor `ACTIVE`. The generic `R6-GAP-*` row in
    `02-phase-and-gate-map.md` is only the linked aggregate (`CONDITIONAL`, `ACTIVE`, or `COMPLETE`),
    never the status store for individual gaps.
11. **Every activated gap begins with its own docs gate.** The first and only authorized task after a
    named gap activates is to create its scorer-specific SPEC/PLAN/TASKS together, commit the three docs
    atomically, and use fresh built-in `default` review/fix cycles until clean. Only then may that gap
    make a production change or commit a no-code proof receipt. This applies even when the witness is
    already green.

## Acceptance Control Matrix

All score triples below describe the final public `DriftScore` after state resolution. "None" under
fixture means an inline integration-test session built with existing row helpers. Every fail route is
conditional: create it only after the named control is committed as a failing witness.

| Ledger | Exact test function | Preconditions / input seam | Expected result / adjudication | Fixture | Focused command | Conditional gap route |
|---|---|---|---|---|---|---|
| `CTX-R6-03` | `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity` | `tests/dead_end_thrash.rs`; failure-only repeated history touches the current interval (`active_repeated_failure=true`), with no repeated-verification history and `active_repeated_verification=false`; `SessionProgress::TroubleshootingFrontier` has regression/non-advancing evidence and no direct-advance signal. | `30 / Medium / Active`, flagged; evidence names stall/regression. Those upstream bits make the exact triple deterministic: regression must not be suppressed as churn. | None | `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture` | `R6-GAP-DET-REGRESSION` |
| `CTX-R6-04` | `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity` | Opaque delegated-parent / `ParentVisibleOrchestration`; no attributable child command observations, no repeated-failure or repeated-verification history, and both active repetition bits false. | `0 / Low / Cleared`, unflagged, empty evidence. The empty history/observation seam fixes confidence and avoids historical state resolution. | None | `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture` | `R6-GAP-DET-OPAQUE-PARENT` |
| `CTX-R6-05` | `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes` | Long-autonomous and many-short-conversational sessions supply identical failure-only repeated history, `active_repeated_failure=true`, no repeated-verification history/activity, and identical direct frontier-advance signals; neither session has a prior `Active` `dead_end_thrash` score. | Both `20 / Medium / HistoricalOnly`, unflagged. Failure-only history fixes confidence; no prior active score fixes `HistoricalOnly` rather than `Recovered`. Adjudication: turn shape is fully consumed upstream; only unequal `SessionProgress` may change the score. | None | `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_scores_equal_progress_equally_across_turn_shapes -- --exact --nocapture` | `R6-GAP-DET-TURN-EQUIVALENCE` |
| `CTX-R6-09` | `truth_grounding_gap_keeps_no_action_planning_clear` | Declared truth path; planning/research prose; no write-like or verification-like command. | `0 / Medium / Cleared`, unflagged; authority evidence is allowed, no action-gap evidence. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_no_action_planning_clear -- --exact --nocapture` | `R6-GAP-TGG-NO-ACTION` |
| `CTX-R6-10` | `truth_grounding_gap_flags_successful_verification_without_truth_reads` | Declared truth path; typed successful verification outside that path; no earlier truth read. | `80 / High / Active`, flagged. Typed success must not fabricate grounding. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_successful_verification_without_truth_reads -- --exact --nocapture` | `R6-GAP-TGG-SUCCESS-WITHOUT-READ` |
| `CTX-R6-11` | `truth_grounding_gap_is_event_order_invariant_across_turn_shapes` | Same declared truth and same ungrounded action order, represented once as one long autonomous turn and once as many short turns. | Both `80 / High / Active`, flagged, with equivalent evidence semantics. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_is_event_order_invariant_across_turn_shapes -- --exact --nocapture` | `R6-GAP-TGG-TURN-INVARIANCE` |
| `CTX-R6-11` | `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action` | Declared truth path plus opaque parent orchestration; no attributable child write/verification. | `0 / Medium / Cleared`, unflagged; authority evidence only. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture` | `R6-GAP-TGG-OPAQUE-PARENT` |
| `CTX-R6-12` | `truth_grounding_gap_flags_truth_path_action_before_read` | Declared truth path; first write/verification touches that path; no earlier read. | `80 / High / Active`, flagged; evidence includes the authority and action. Historical controls-wall result: preserved red. Current source: pass at the review-clean final proof receipt. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture` | `R6-GAP-TGG-TRUTH-PATH-ACTION` |
| `CTX-R6-13` | `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes` | Planning/research and implementation sessions differ only in archetype/task framing; both take the same write/verification action before reading declared truth. | Both `80 / High / Active`, flagged. Action creates the same provenance obligation in either archetype. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture` | `R6-GAP-TGG-ARCHETYPE-INVARIANCE` |
| `CTX-R6-14` | `wrong_plan_branch_ignores_read_only_out_of_scope_exploration` | Non-empty authority; read-only command names an out-of-scope path. | `0 / Medium / Cleared`, unflagged, empty evidence. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_ignores_read_only_out_of_scope_exploration -- --exact --nocapture` | `R6-GAP-WPB-READ-ONLY` |
| `CTX-R6-14` | `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope` | A sanctioned path pivot updates current truth/working-set authority before a write under the new path. | `0 / Medium / Cleared`, unflagged, empty evidence. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_accepts_write_under_sanctioned_replan_scope -- --exact --nocapture` | `R6-GAP-WPB-REPLAN-SCOPE` |
| `CTX-R6-14` | `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action` | Non-empty parent authority plus opaque parent orchestration; no attributable child path-bearing write/verification. | `0 / Medium / Cleared`, unflagged, empty evidence. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture` | `R6-GAP-WPB-OPAQUE-PARENT` |
| `CTX-R6-15` | `wrong_plan_branch_makes_no_claim_for_path_action_without_authority` | Empty truth-artifact and non-observed working-set authority; one path-bearing write/verification. | `0 / Low / Cleared`, unflagged, empty evidence. Likely intentional red witness at current source. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture` | `R6-GAP-WPB-EMPTY-AUTHORITY` |
| `CTX-R6-01` | `acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged` | Trusted real-rollout-derived fixture `019f1ecb-b93a-7570-8d8d-9ce4e711880b` proves repeated failures before direct frontier advancement. | `HistoricalOnly / 20 / High`, unflagged, with no earlier `Active`; series `a0089c8de` + `968a4377f` fresh independent `REVIEW CLEAN`. | Complete in `R6-REPLAY`. | `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged -- --exact --nocapture` | Preserve through replay family wall; no terminal disposition before `R6-CLOSE`. |
| `CTX-R6-02` | `acceptance_fixtures_integrated_true_stall_stays_active` | Trusted depth-1 built-in `default` subagent rollout `019eb311-c7ce-7f50-ae13-b51a5b5461c3`; checkpoint `5` must be `TroubleshootingFrontier / Stalled / Medium`, with repeated failed-call evidence. | Locked flagged `Active / 30 / High`. Historical witness `60cde3dd7` preserves the attribution red; review-clean commit `6eda87e60` makes exact control green with truthful target attribution. | Complete; active packet `none`. Preserve through the replay family wall. | `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_true_stall_stays_active -- --exact --nocapture` | No terminal disposition before `R6-CLOSE`. |
| `CTX-R6-06` | `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture` | Existing full-analyzer frozen corpus: three named cleared controls plus the sticky success-tail witness. | Current selected authority is three `Cleared / 0 / unflagged` plus sticky `HistoricalOnly / 20`, unflagged. Clean `f898d61e7` `Recovered / 20` is historical baseline evidence only. | Phase-owned exact frozen corpus and renamed sticky controls pass `1 / 1` each; family wall and full analyzer are green. Event `831 -> 837` and `recovery_state` remain non-causal. Proof receipt awaits fresh review. | `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture -- --exact --nocapture` | Preserve selected authority without comparative-improvement or terminal-disposition claim. |

## Files And Commands

Current control files:

```text
crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs
crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs
crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs
crates/agent-drift-analyzer/tests/dead_end_thrash.rs
crates/agent-drift-analyzer/tests/truth_grounding_gap.rs
crates/agent-drift-analyzer/tests/wrong_plan_branch.rs
crates/agent-drift-analyzer/tests/acceptance_fixtures.rs
crates/agent-drift-analyzer/tests/fixtures/acceptance/README.md
```

Packet checkpoint / family wall:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Run only the active row command while iterating. Each matrix row is its own atomic test-only
commit/fresh-review boundary; do not batch a scorer family. Run the owning family command only after all
of that family's row commits are independently review-clean. Replay commands, including the existing
`CTX-R6-06` preservation command, run only in `R6-REPLAY`.

## Testing And Gap Strategy

- RED first: add a behavior assertion, run its exact focused command, and record PASS or the exact red
  result before touching production.
- A passing control is fit-for-purpose evidence; do not refactor production merely because a different
  implementation seems cleaner.
- A failing control is committed as a witness in that row's own atomic commit. Every matrix row, green or
  red, receives a fresh built-in `default` review before the next row begins. Family checkpoints may
  aggregate only after every row commit in that family is review-clean.
- `R6-C.1-CONTROLS` ends as soon as every synthetic row has a deterministic preserved result. Its
  transition commit activates the first named red `R6-GAP-*` phase in matrix/family order, or
  `R6-REPLAY` when no row is red. Never run a gap while `R6-C.1-CONTROLS` is active.
- At `CONTROLS -> first GAP`, instantiate the `Named R6 Gap Status Subledger` in
  `05-proof-decision-regression-ledger.md`. Each red row owns one distinct subledger entry with: phase
  ID; exact preserved witness/control; all three canonical owning packet-doc paths; current status;
  predecessor and successor; and evidence/commit. The first gap alone is `ACTIVE`,
  `R6-C.1-CONTROLS` is its `COMPLETE` predecessor, and later gaps are `BLOCKED`. At each gap transition,
  atomically set the active gap `COMPLETE`, its immediate successor `ACTIVE`, and leave all later gaps
  `BLOCKED`. A review-clean final gap transitions to `R6-REPLAY`. Gap phases never batch independent
  failures or reactivate `R6-C.1-CONTROLS`.
- The canonical gap packet paths are
  `docs/specs/r6/R6-GAP-<SCORER>-<SEAM>/R6-GAP-<SCORER>-<SEAM>-spec.md`,
  `docs/specs/r6/R6-GAP-<SCORER>-<SEAM>/R6-GAP-<SCORER>-<SEAM>-plan.md`, and
  `docs/specs/r6/R6-GAP-<SCORER>-<SEAM>/R6-GAP-<SCORER>-<SEAM>-tasks.md`. At gap activation those paths
  are recorded in the named-gap
  subledger as plain `TO CREATE` path strings, not links or claims that the files exist. The transition
  must not update or cite a nonexistent gap TASKS file. The first task after activation creates all three
  files atomically. Each packet must name the exact preserved witness, exact GitNexus symbol/impact
  command, minimal allowed files, production-fix acceptance and no-code acceptance, focused witness
  command, owning-family/checkpoint walls, and the exact successor transition. Commit and fresh-review
  those docs until clean before either execution path; after the docs gate lands, replace `TO CREATE`
  with the actual packet-doc paths and their review-clean commit in the subledger.
- When an earlier gap fix also makes a later sequential gap's exact preserved witness green, activate the
  later gap normally but make no production edit. Rerun that exact row test and the owning scorer-family
  controls, record the earlier causal commit in the now-existing gap TASKS/receipt and changed ledger
  row, stage only those result/ledger docs, run the staged commit gate, commit the no-code proof receipt/status
  reconciliation, and obtain fresh independent review. Preserve the named gap and original witness; do
  not silently delete or merge either. The receipt commit leaves that gap active and activates no
  successor. Only after the receipt is review-clean may the separate normal transition commit activate
  the next gap or `R6-REPLAY`.
- Run `npx gitnexus impact <symbol> -r 97a0-substrate --direction upstream --depth 3` before any later
  indexed-symbol edit. Warn/stop on HIGH or CRITICAL impact.
- Before every commit, run the exact staged gate below. `<intended-files-only>` must exclude unrelated
  dirt, and the final command is a complete staged-diff inspection:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

## Phase-Transition Authority Manifest

Every `SPEC -> CONTROLS`, `CONTROLS -> first GAP or REPLAY`, and
`GAP -> next GAP or REPLAY` transition is one reconciled authority update. The transition commit must
update these exact active-phase mirrors together:

| Authority surface | Exact fields / wording that move together |
|---|---|
| This packet SPEC | Update this file's header `Status`. At `SPEC -> CONTROLS`, use `APPROVED / LANDED — R6-C.1-SPEC COMPLETE; R6-C.1-CONTROLS ACTIVE`. At `CONTROLS -> successor`, use `APPROVED / LANDED — R6-C.1-CONTROLS COMPLETE; <successor> ACTIVE`. At a later gap transition, use `APPROVED / LANDED — R6-C.1-CONTROLS COMPLETE; <completed-gap> COMPLETE; <successor> ACTIVE`; after the final gap, `<successor>` is `R6-REPLAY`. Change status wording only; do not rewrite locked decisions. |
| This packet PLAN | Update the PLAN header `Status` with the same exact phase-aware value as the SPEC. Do not rewrite plan decisions merely because the active phase moved. |
| This packet TASKS | Check the docs-lock task at `SPEC -> CONTROLS`, record actual completed task/result checks, and set the header to `ACTIVE — R6-C.1-SPEC COMPLETE; R6-C.1-CONTROLS ACTIVE`. At later transitions use `HANDOFF TRACKING — R6-C.1-CONTROLS COMPLETE; <completed-predecessor> COMPLETE; <successor> ACTIVE` (omit the completed-gap clause when the predecessor is CONTROLS). Record the sole next authorized action. |
| Active gap packet docs, conditional | On entry to a named gap, no gap TASKS is required or referenced: the sole next action is atomic creation/review of the three canonical `TO CREATE` paths. After that docs gate is committed and review-clean, its SPEC/PLAN/TASKS become required gap surfaces for actual proof and for the transition out of that gap. |
| `docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md` | `Current work phase` and `Last repo-truth verification`. |
| `docs/specs/hybrid-drift-r6-r8-control-pack/01-authority-and-status-map.md` | `Verified against`, `Current phase`, and the R6 `Current Status` row's `Status` and `Next allowed action`. |
| `docs/specs/hybrid-drift-r6-r8-control-pack/02-phase-and-gate-map.md` | The `Master Sequence` status cells for the completed and newly active concrete phases, plus entry/exit-gate wording when proof changes. Its generic `R6-GAP-*` row links to the named-gap subledger and reports only aggregate `CONDITIONAL` before route instantiation, `ACTIVE` while any named gap is active, or `COMPLETE` when no gap is required or every named gap is complete. It never carries individual gap statuses. Exactly one concrete phase is `ACTIVE`. |
| `docs/specs/hybrid-drift-r6-r8-control-pack/04-reusable-phase-runner.md` | At `SPEC -> CONTROLS`, reconcile `VERIFICATION AND COMMIT` to stage only intended files, run `npx gitnexus detect-changes --scope staged -r 97a0-substrate`, then run `git diff --cached --check` and inspect the complete `git diff --cached` before commit. Remove the unscoped `detect-changes` and working-tree `git diff --check` commit-gate prescriptions. |
| `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md` | `Ledger status`; `Verified against`; changed `CTX-R6-*` cells; and `Update Record`. At `CONTROLS -> first GAP`, create/update the `Named R6 Gap Status Subledger` with one ordered row per red route and columns `Phase ID`, `Preserved witness/control`, `Owning packet docs`, `Status`, `Predecessor`, `Successor`, and `Evidence/commit`. Record canonical docs paths as non-link `TO CREATE` strings until they land. Every gap transition atomically updates these rows with the other mirrors. Do not rewrite unaffected row evidence. |
| `docs/specs/hybrid-drift-r6-r8-control-pack/06-operator-prompt-library.md` | `Current First Invocation`: its current-phase sentence plus `PHASE_ID` and `ACTIVE_PACKET`. |
| Root `SPEC.md`, `tasks/plan.md`, and `tasks/todo.md` | `Status`, `Current phase`, completed/current task wording, and sole next-authorized action/check box. At `SPEC -> CONTROLS`, replace the root `tasks/plan.md` GitNexus execution rule with the same staged-only sequence: stage intended files, run `npx gitnexus detect-changes --scope staged -r 97a0-substrate`, then `git diff --cached --check` and complete `git diff --cached` inspection. |
| Canonical R6 finding/MAP and landing order | Update `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`, `docs/specs/r6/MAP.md`, and `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` only when actual control or gap evidence changes their proof, status, or next-action wording. |

At `SPEC -> CONTROLS`, inspect only the other active-phase mirrors already named in this manifest for
commit-gate wording. If any still prescribes unscoped `detect-changes`, working-tree `git diff --check`,
or an unstaged diff as the commit gate, reconcile it to the same staged-only sequence in the transition
commit. Do not expand this requirement into cleanup of inactive or historical docs.

The three transition-specific result locks are:

1. **`R6-C.1-SPEC -> R6-C.1-CONTROLS`:** this packet's SPEC and PLAN receive the exact
   `APPROVED / LANDED — R6-C.1-SPEC COMPLETE; R6-C.1-CONTROLS ACTIVE` status; TASKS checks the
   docs-lock task, records its review-clean commit, and reports `R6-C.1-CONTROLS` as the sole active
   phase. The same transition commit must reconcile the reusable runner and root `tasks/plan.md` staged
   commit-gate authority above, plus any other stale active mirror found by the bounded manifest check.
   No control row is promoted without actual test output.
2. **`R6-C.1-CONTROLS -> first R6-GAP-* or R6-REPLAY`:** every synthetic row has a committed PASS or
   preserved red result; controls become complete; the three R6-C.1 packet headers use the prescribed
   phase-aware values; all named-gap subledger rows are instantiated; the first red route in
   matrix/family order becomes active and later rows remain blocked, or `R6-REPLAY` becomes active when
   no row is red. The active gap's sole next action is its atomic scorer-specific docs gate.
3. **Active `R6-GAP-* -> next R6-GAP-* or R6-REPLAY`:** the named gap has a committed, fresh-review-clean
   packet docs plus fix or no-code proof receipt; that gap and the R6-C.1 packet headers become
   phase-current, the subledger marks that gap complete and exactly the next red route active, or final
   `R6-REPLAY` active. The newly active gap again authorizes docs creation/review only.

Do not churn unchanged semantic authority, but do not leave any active-phase mirror stale: every mirror
listed above must agree on the one active phase, completed predecessor, verification commit, and next
action. `R6-C.1-CONTROLS` cannot activate and no control work may start until the fully reconciled
transition is committed and a fresh independent reviewer says `REVIEW CLEAN`.

## Full R6 Ledger Coverage

| IDs | R6-C.1 disposition |
|---|---|
| `CTX-R6-01`, `CTX-R6-02` | Exact integrated controls specified above; execution remains `R6-REPLAY`. |
| `CTX-R6-03` through `CTX-R6-05` | Open `dead_end_thrash` controls in `R6-C.1-CONTROLS`. |
| `CTX-R6-06` | Existing exact frozen-corpus preservation control is specified above; execution stays `R6-REPLAY` and must preserve three cleared plus one sticky `HistoricalOnly / 20 / unflagged` posture. `Recovered / 20 / unflagged` is historical baseline only, not current authority. |
| `CTX-R6-07`, `CTX-R6-08` | Preserve semantic scorer completion and fixture-integrity/live-path distinction; no reopen here. |
| `CTX-R6-09` through `CTX-R6-13` | Open `truth_grounding_gap` controls in `R6-C.1-CONTROLS`. |
| `CTX-R6-14`, `CTX-R6-15` | Open `wrong_plan_branch` controls in `R6-C.1-CONTROLS`. |
| `CTX-R6-16` | Source-only dispatcher ordering proof; no focused order test. |
| `CTX-R6-17` | Terminal scorer table remains open for `R6-CLOSE`; this packet cannot fill it. |
| `CTX-R6-18` | Preserve the proven historical/superseded labels. |

## Boundaries

**Always:** preserve failing witnesses; make one atomic commit and fresh review per matrix row; keep one
scorer/failure seam per gap phase; use repo-relative links; update TASKS and `CTX-R6-*` ledger rows only
with actual results; dispatch a fresh built-in `default` reviewer at every row, gap, and transition
boundary; stage only intended files and validate the staged diff before commit.

**Escalate:** only for an unresolved authority/product choice, HIGH/CRITICAL GitNexus impact,
unisolatable unrelated work, unavailable trusted replay evidence, or a scope change. Use the structured
`DECISION REQUIRED` / `ACTION REQUIRED` forms in the operator prompt library.

**Never:** edit tests/code/fixtures during `R6-C.1-SPEC`; change production before a red witness; batch
matrix-row witness commits or independent gap fixes; run a gap while `R6-C.1-CONTROLS` is active; route a
fixture-shape mismatch to a production gap; absorb a baseline defect into R7; reopen
`semantic_goal_drift` without a new behavior-level failure; execute replay closeout; claim R6 closure;
start R7 or R8.

## Success Criteria

1. This SPEC, its PLAN, and TASKS are committed and independently review-clean.
2. Every `CTX-R6-03` through `CTX-R6-05` and `CTX-R6-09` through `CTX-R6-15` control has an exact test,
   input seam, expected disposition, fixture decision, focused command, and one conditional gap route.
3. `CTX-R6-01`/`02` defer fixture-dependent precision, `CTX-R6-06` names the executable frozen-corpus
   preservation control, and all three remain owned by `R6-REPLAY`.
4. `CTX-R6-16` remains a no-order-test adjudication.
5. The next phase may activate only as `R6-C.1-CONTROLS`; no control is marked complete by this docs
   phase. Later transitions follow `CONTROLS -> first red gap -> ... -> final red gap -> R6-REPLAY`, or
   `CONTROLS -> R6-REPLAY` when all synthetic rows are green.

## Non-Goals

- Any code, test, or fixture change during `R6-C.1-SPEC`.
- Integrated replay closeout or trusted-session selection (`R6-REPLAY`).
- R6 terminal dispositions or closure (`R6-CLOSE`).
- R7 delegated-session implementation or R8 Sentinel consolidation.
- Any `semantic_goal_drift` change.
