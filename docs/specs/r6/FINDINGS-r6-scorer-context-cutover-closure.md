# Findings: R6 Scorer-Context Cutover Closure

**Date:** 2026-07-12; control and first-gap disposition updated 2026-07-13

**Status:** PARTIAL / CLOSURE AUDIT REQUIRED

**Scope:** scorer applicability, behavioral-proof audit, control disposition, and bounded first-gap proof

## Decision

The scoped R6 packets and `R6-C.1-CONTROLS` are landed, but that is not enough to close the broader
R6 charter. The thirteen synthetic controls resolved as `10 PASS / 3 preserved RED` at the
`5618f7864` controls wall, with no production change. In matrix order, those reds route to
`R6-GAP-DET-OPAQUE-PARENT` (`CTX-R6-04`, witness `87409b39a`),
`R6-GAP-TGG-TRUTH-PATH-ACTION` (`CTX-R6-12`, witness `e67d8b214`), and
`R6-GAP-WPB-EMPTY-AUTHORITY` (`CTX-R6-15`, witness `59f098b35`). Production series `bcd94bf4f` +
`931e50c85` + `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN`, making `CTX-R6-04`
proven focused and completing the first named gap. R6 remains **PARTIAL**; only
`R6-GAP-TGG-TRUTH-PATH-ACTION` is active, at its docs-only gate, while
`R6-GAP-WPB-EMPTY-AUTHORITY` and `R6-REPLAY` remain blocked. R7 remains useful, design-ready draft
work, but it is **not implementation-ready**.

This audit does **not** interpret R6 as requiring every scorer to consume typed outcomes, turn
context, archetype, and progress. A scorer is complete when its chosen inputs match the behavior it
owns and behavior-level tests prove that match. Transitive availability alone is not integration.

## Context Applicability Legend

- **Direct:** the scorer reads the layer.
- **Indirect:** an upstream module derives an input the scorer reads.
- **Boundary:** the layer can affect checkpoint/history boundaries but is not scoring evidence.
- **Not applicable:** the layer does not answer the scorer's semantic question and must not be
  injected merely to satisfy the charter wording.
- **Proof gap:** relevance is plausible or the exception is reasonable, but a behavioral control is
  missing.

## Scorer-by-Context Applicability Matrix

| Scorer | Intended behavior | Current direct inputs | Current indirect inputs | Typed outcomes | Turn context | Archetype | Progress | Structured objective | Working-set / truth evidence | Delegation visibility | Behavioral proof | Remaining gap | Disposition |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `dead_end_thrash` | Distinguish active repeated failure/verification with no frontier movement from expected churn that advances or cleanly recovers. | Repetition history, recovery-active bits, `SessionProgress`, command observations for confidence. | Typed attempts and outcomes feed progress; turn context feeds archetype; archetype selects progress dimension; objective changes can bound comparable history. | **Indirect, relevant.** Failure/verification classification and frontier signals depend on it. | **Indirect, relevant.** | **Indirect, relevant.** | **Direct, required.** | **Boundary only.** | **Not applicable.** | **Indirect, relevant.** The exact opaque-parent control preserves `ParentVisibleOrchestration` without attributing child misconduct. | Production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh `REVIEW CLEAN`. `CTX-R6-04` is `0 / Low / Cleared`, unflagged, empty evidence; the partial/mixed parent-visible regression is `0 / Medium / Cleared`, unflagged, empty evidence; and `CTX-R6-03` remains `30 / Medium / Active`, flagged. All `21` matching family tests and all `167` matching checkpoints passed; format/check passed. Frozen real-rollout derivatives remain invariance evidence only. | `R6-GAP-DET-OPAQUE-PARENT` complete; integrated replay proof remains bounded and incomplete. | **Focused gap proven / terminal disposition pending R6-CLOSE** |
| `semantic_goal_drift` | Detect an unsanctioned target pivot relative to kickoff or prior checkpoint while suppressing legitimate narrowing, role shifts, replans, and unsupported delegated-parent claims. | Current structured objective, kickoff anchor, previous structured objective/checkpoint, stable target anchors, sanctioned-replan bit, delegation topology/visibility. | Structured extraction and checkpoint history supply the compared goals. | **Not applicable.** Outcome success does not establish target continuity. | **Not applicable.** | **Not applicable.** | **Not applicable.** | **Direct, required.** | **Not applicable.** | **Direct, required for the bounded opaque-parent guard.** | 56 focused scorer/state tests cover sanctioned replans, opaque/partial delegation, and internal `Fire` / `Suppress` / `NoClaim` routing. Separately, the bounded corpus-shape test proves fixture integrity only. The live analyzer-path acceptance test exercises only its 18 allowlisted pivot, narrowing, progression, and role-shift fixtures; it does not prove replan, delegation, or internal routing. The R6-3.X.2D corpus closeout recorded 110/110 sessions analyzable with 0 emitted fires after remediation. | No new failing witness. Progress or archetype would not make target-continuity reasoning more honest. | **Cutover complete** |
| `truth_grounding_gap` | Detect write/verification action taken without first reading declared truth artifacts; preserve and recover history honestly. | Task-frame truth paths, interval command observations and event order, previous truth-gap score. | Working-set/task-frame extraction supplies the truth paths. | **Not applicable by design:** success/failure does not prove that required truth was read. | **Not applicable by design:** ordering is event-based, not turn-count based. | **Fit-for-purpose for archetype:** equivalent actions scored equally across planning and implementation frames. | **Not applicable:** later progress cannot retroactively establish prior grounding. | **Indirect boundary/source only.** | **Direct, required.** | **Proven for the bounded opaque-parent no-action case.** | No-action planning, successful ungrounded verification, turn-shape invariance, opaque parent no-action, and actionful archetype-equivalence controls passed. `truth_grounding_gap_flags_truth_path_action_before_read` preserved the sole family red: actual `0 / Medium / Cleared` versus required `80 / High / Active`. | `R6-GAP-TGG-TRUTH-PATH-ACTION` is active at its docs-only gate. | **Preserved red / active named gap** |
| `wrong_plan_branch` | Detect write/verification paths outside the task frame's expected truth/working-set scope and clear after a return in scope. | Truth-artifact paths, working-set paths, interval command paths and write/verification classification. | Objective/working-set extraction supplies expected paths; checkpoint boundaries isolate the current interval. | **Not applicable:** command outcome does not change path scope. | **Not applicable:** path scope is event-local. | **Not applicable:** exploration is already ignored unless it writes or verifies. | **Not applicable:** healthy progress cannot excuse mutation outside the authorized branch. | **Relevant through sanctioned continuity; the sanctioned-replan control passed.** | **Direct, required.** | **Proven for the bounded opaque-parent no-action case.** | Read-only exploration, sanctioned replan/path pivot, and opaque-parent controls passed. `wrong_plan_branch_makes_no_claim_for_path_action_without_authority` preserved the sole family red: actual `60 / Low / Active` versus required `0 / Low / Cleared`. | `R6-GAP-WPB-EMPTY-AUTHORITY` is blocked behind the grounding gap. | **Preserved red / blocked named gap** |
| `scoring/mod.rs` | Deterministically build shared scorer inputs, invoke all material scorers, and order results. It is dispatcher infrastructure, not a fifth scorer. | `CheckpointAnalysis`, previous truth score, kickoff anchor. | Builds `SessionProgress` once and supplies it only to `dead_end_thrash`. | Applicable only through scorer-specific routing. | Same. | Same. | Same. | Same. | Same. | Same. | Source inspection proves the explicit four-class `sort_by_key` order. The full analyzer suite proves the score records travel through the live path, but no focused behavioral assertion proves their exact order. | Exact ordering is source-proven, not behavior-tested. Add a focused assertion only if exact order is retained as a closure contract; do not force a common mega-context argument into every scorer. | **Fit-for-purpose exception** |

No scorer is currently a justified merge/deprecation candidate. `truth_grounding_gap` asks whether
declared authority was read before action; `wrong_plan_branch` asks whether action stayed inside the
expected path scope. Those are separate failure modes.

## Behavioral-Proof Inventory

### `dead_end_thrash`

| Required behavior | Exact proof | Result |
|---|---|---|
| Advancing troubleshooting frontier tolerates expected repeated failures. | `dead_end_thrash_suppresses_repeated_activity_when_the_frontier_advances`; upstream `checkpoints_mark_troubleshooting_frontier_advancement_from_compile_to_test_failure`; real-rollout-derived progress case `019e899c-453f-71f2-a99d-155848c7b081`. | **Proven** for the troubleshooting-frontier path. |
| Stalled progress increases thrash concern. | `dead_end_thrash_names_stalls_when_repeated_activity_has_no_frontier_movement`; `checkpoints_mark_repeated_same_troubleshooting_signature_as_stalled`. | **Proven** for no-frontier-movement stall. |
| Regressing progress increases/retains concern. | `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity`; upstream `checkpoints_mark_troubleshooting_regression_when_frontier_falls_back`. | **Proven:** the scorer control passed at `30 / Medium / Active`, flagged. |
| Clean recovery or verification clears/downgrades honestly. | `dead_end_thrash_clears_after_one_clean_in_scope_verification_interval`; `dead_end_thrash_downgrades_to_historical_only_after_the_recovery_transition`; `dead_end_thrash_clears_replay_shaped_memsrc_verifier_tail`. | **Proven.** |
| Typed outcomes distinguish failure from neutral/success evidence. | `dead_end_thrash_treats_explicit_error_rows_as_repeated_failure_evidence`; `dead_end_thrash_treats_non_zero_exit_code_tool_output_as_failure_evidence`; `dead_end_thrash_ignores_repeated_neutral_tool_output_evidence`; `dead_end_thrash_keeps_repeated_successful_verification_as_historical_context`. | **Proven.** |
| Opaque delegated-parent activity does not become child thrash. | `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity`; upstream delegation/progress controls include `checkpoints_progress_falls_back_to_parent_visible_orchestration_for_opaque_parent_work`; the partial/mixed regression protects typed Medium confidence. | **Proven focused:** production series through fresh review-clean `d13f0a71c` yields `0 / Low / Cleared`, unflagged, empty evidence for `CTX-R6-04`, while partial/mixed parent-visible progress stays `0 / Medium / Cleared`, unflagged, empty evidence. |
| Turn-shape relevance is consumed upstream without changing equal-progress scorer output. | `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes`; upstream turn/archetype construction controls. | **Proven focused:** locked long-autonomous and many-short-conversational cases both scored `20 / Medium / HistoricalOnly`, unflagged. The broad acceptance sentence must not claim a scorer-level difference. |
| Frozen replay postures remain invariant. | `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture`, the three named cleared controls, and `acceptance_fixtures_representative_sticky_success_tail_stays_recovered`. | **Invariance only for the bounded frozen corpus:** three cleared final states and one recovered sticky-success tail remain pinned. This is not a before/after comparator and does not prove integrated scorer improvement. |

### `semantic_goal_drift`

The fit-for-purpose decision is **complete by design**. The scorer consumes the semantic evidence
appropriate to goal continuity: structured current/kickoff/previous objectives, stable target
anchors, sanctioned replans, delegation visibility, and checkpoint history. Representative proof:

- sanctioned pivots: `semantic_goal_drift_skips_sanctioned_replan_pivots` and
  `semantic_goal_drift_skips_rolling_pivots_when_sanctioned_replan_is_present`;
- delegation: `semantic_goal_drift_suppressed_on_opaque_delegated_parent_without_current_target_anchor`,
  `semantic_goal_drift_still_fires_on_opaque_delegated_parent_with_target_anchors_both_sides`, and
  `semantic_goal_drift_not_hard_suppressed_on_partial_delegated_parent`;
- stable continuity: the structural narrowing, doc-bundle, work-item-family, and plan/code plus
  review/fix/verify role-shift tests;
- fixture integrity: `semantic_goal_drift_acceptance_corpus_stays_bounded_and_bundle_shaped` proves
  only that the bounded corpus remains well-shaped;
- live analyzer behavior:
  `semantic_goal_drift_acceptance_fixture_runs_through_live_analyzer_checkpoint_path` exercises the
  fixture through the analyzer checkpoint path.

No new evidence justifies reopening the recently closed semantic-goal-drift family.

### `truth_grounding_gap`

The controls confirm the fit-for-purpose direction for no-action planning/research,
successful-but-ungrounded verification, turn-shape invariance, opaque parent orchestration without
attributable child action, and equivalent action across planning/implementation archetypes. The
remaining behavior gap is narrower: a first write-like action touching the declared truth path before
any read currently clears at `0 / Medium / Cleared` instead of the locked `80 / High / Active` claim.
That preserved `CTX-R6-12` witness owns active `R6-GAP-TGG-TRUTH-PATH-ACTION`; its three canonical
packet paths remain non-link `TO CREATE` entries, and its docs-only gate is the sole next action.

### `wrong_plan_branch`

The controls confirm that read-only exploration stays quiet, sanctioned replans update the expected
scope for the bounded write case, and opaque parent orchestration without attributable child action
stays clear. The remaining behavior gap is the empty-authority path-bearing action: current behavior
claims `60 / Low / Active` with action evidence instead of the locked `0 / Low / Cleared` no-claim.
That preserved `CTX-R6-15` witness owns `R6-GAP-WPB-EMPTY-AUTHORITY`, blocked until the grounding gap
completes.

## Broad R6 Acceptance-Claim Audit

| Root acceptance claim | Owning scorer/module | Focused proof | Replay/bounded evidence | Status and honest wording |
|---|---|---|---|---|
| Troubleshooting tolerates expected failures while the frontier advances. | `dead_end_thrash` over `SessionProgress`. | Advancement suppression plus upstream frontier tests named above. | Annotated real-rollout troubleshooting fixture `019e899c-...` supplies upstream progress evidence; the frozen dead-end corpus protects final posture. Neither is an integrated advancing repeated-failure scorer replay. | **Proven focused / bounded; integrated replay remains open.** `CTX-R6-01` still requires an integrated real-rollout-derived advancing repeated-failure score to remain unflagged or historical. |
| Long autonomous turns are evaluated differently from multi-turn conversational sessions. | Turn context + archetype + progress; `dead_end_thrash` consumes the derived progress rather than raw turn shape. | Construction tests prove the turn/archetype distinction; `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes` proves equal derived progress produces equal scorer output. | No bounded replay A/B requires a different scorer disposition. | **Narrowed honestly:** turn structure informs archetype/progress construction; it does not independently change `dead_end_thrash` when the relevant derived progress is equal. |
| Flagged sessions are materially more honest on known replay artifacts. | Primarily `dead_end_thrash`; semantic drift has its separate corpus. | Per-case frozen posture assertions; no before/after integrated comparator. | Four real-rollout-derived dead-end cases preserve three cleared and one recovered posture; the separate semantic closeout corpus recorded 110 sessions with no post-remediation fires. | **Partially / bounded proven.** The frozen dead-end corpus proves invariance, not comparative integrated improvement. Retain partial wording until integrated advancing and true-stall replay witnesses close the claim or the charter is narrowed. |

## R6-C.1 Control Disposition (2026-07-13)

`R6-C.1-CONTROLS` is **COMPLETE**. All thirteen synthetic controls were committed and freshly
reviewed row-by-row; the wall receipt `5618f7864` records `10 PASS / 3 preserved RED`, no new red,
passing checkpoints, and no production change. `semantic_goal_drift` remains excluded absent new
failing evidence.

The three distinct routes must execute sequentially in matrix order:

1. **COMPLETE:** `R6-GAP-DET-OPAQUE-PARENT` for `CTX-R6-04` at witness `87409b39a`; production
   series through `d13f0a71c` is fresh `REVIEW CLEAN` with focused/family/checkpoint proof green.
2. **ACTIVE, docs-only gate:** `R6-GAP-TGG-TRUTH-PATH-ACTION` for `CTX-R6-12` at witness
   `e67d8b214`.
3. **BLOCKED:** `R6-GAP-WPB-EMPTY-AUTHORITY` for `CTX-R6-15` at witness `59f098b35`.

The sole next authorized action is atomic creation and fresh review of the three canonical
`R6-GAP-TGG-TRUTH-PATH-ACTION` packet docs recorded as non-link `TO CREATE` paths in the named-gap
subledger. Do not claim those files exist, execute the gap, or begin production/no-code proof first.
`R6-GAP-WPB-EMPTY-AUTHORITY` and `R6-REPLAY` remain blocked.

## Verification Run For This Audit

The original audit commands below passed on 2026-07-12. This is historical baseline evidence, not the
current control disposition:

```text
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture       PASS (14 focused integration tests; related filtered/unit tests also ran)
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture  PASS (56 focused unit/state tests + 2 acceptance tests)
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture  PASS (4 focused integration tests)
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture    PASS (2 focused integration tests)
cargo test -p agent-drift-analyzer checkpoints -- --nocapture          PASS (131 unit + 35 integration matches, plus related filtered suites)
cargo test -p agent-drift-analyzer -- --nocapture                      PASS (all analyzer test binaries; 0 failures)
```

Those passing tests establish the behavior they assert. They did not fill the acceptance gaps that
the later controls made explicit.

The later `R6-C.1` wall at `5618f7864` produced the current disposition:

```text
thirteen planned synthetic controls                                     10 PASS / 3 preserved RED
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture       expected CTX-R6-04 RED only
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture   expected CTX-R6-12 RED only
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture     expected CTX-R6-15 RED only
cargo test -p agent-drift-analyzer checkpoints -- --nocapture           PASS
```

The exact preserved witnesses are committed at `87409b39a`, `e67d8b214`, and `59f098b35`,
respectively. The expected red exits are preserved proof, not a green family wall, and no production
code changed.

## Final R6 Status And R7 Promotion Gate

**R6 status: PARTIAL with `R6-GAP-DET-OPAQUE-PARENT` complete and
`R6-GAP-TGG-TRUTH-PATH-ACTION` active at its docs-only gate.** The landed R6 packet history and
completed `R6-C.1-CONTROLS` remain intact; commit `99efda8f9` is not closure authority.
`R6-GAP-WPB-EMPTY-AUTHORITY` and `R6-REPLAY` remain blocked. No terminal scorer disposition,
replay close, or R6 close is claimed.

R7 may be promoted from **DRAFT / BLOCKED ON R6 CLOSURE DECISION** to implementation-ready only when:

1. the R6 scorer-by-context applicability audit is complete;
2. every material scoring surface has exactly one terminal disposition: **Cutover complete**,
   **Fit-for-purpose exception**, **Merged/deprecated**, or **Explicitly deferred outside R6 with
   justification**; an ordinary “still open” state is not a closure disposition;
3. the broad R6 acceptance claims have behavioral proof or are narrowed honestly;
4. `R6-C.1-CONTROLS` is complete, all three preserved reds have been resolved in their distinct
   bounded named gaps, replay closeout is complete, and this finding is updated to **CLOSED**; and
5. root landing-order authority, R6 MAP, root SPEC/tasks, and R7 status all agree.

Until then, R7 must not absorb unresolved ordinary single-session scorer semantics.
