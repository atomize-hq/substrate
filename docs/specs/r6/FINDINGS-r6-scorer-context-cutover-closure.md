# Findings: R6 Scorer-Context Cutover Closure

**Date:** 2026-07-12

**Status:** PARTIAL / CLOSURE AUDIT REQUIRED

**Scope:** read-only scorer applicability and behavioral-proof audit; no scorer behavior changed

## Decision

The scoped R6 packets are landed, but that is not enough to close the broader R6 charter. R6 is
**PARTIAL** until the narrow acceptance packet named below resolves three proof gaps. R7 remains
useful, design-ready draft work, but it is **not implementation-ready**.

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
| `dead_end_thrash` | Distinguish active repeated failure/verification with no frontier movement from expected churn that advances or cleanly recovers. | Repetition history, recovery-active bits, `SessionProgress`, command observations for confidence. | Typed attempts and outcomes feed progress; turn context feeds archetype; archetype selects progress dimension; objective changes can bound comparable history. | **Indirect, relevant.** Failure/verification classification and frontier signals depend on it. | **Indirect, relevant.** | **Indirect, relevant.** | **Direct, required.** | **Boundary only.** | **Not applicable.** | **Indirect, relevant but under-proven.** Delegation can select `ParentVisibleOrchestration`, but the scorer has no delegated-parent control. | Fourteen focused tests cover advancement, stall, recovery/history, explicit errors, non-zero exits, neutral outputs, and replay-shaped cleanup. Frozen real-rollout derivatives cover three cleared controls plus one recovered sticky tail. | No scorer-level witness for regressing progress, opaque delegated-parent activity, or a long-autonomous versus many-short-conversational A/B. | **Acceptance-proof gap** |
| `semantic_goal_drift` | Detect an unsanctioned target pivot relative to kickoff or prior checkpoint while suppressing legitimate narrowing, role shifts, replans, and unsupported delegated-parent claims. | Current structured objective, kickoff anchor, previous structured objective/checkpoint, stable target anchors, sanctioned-replan bit, delegation topology/visibility. | Structured extraction and checkpoint history supply the compared goals. | **Not applicable.** Outcome success does not establish target continuity. | **Not applicable.** | **Not applicable.** | **Not applicable.** | **Direct, required.** | **Not applicable.** | **Direct, required for the bounded opaque-parent guard.** | 56 focused scorer/state tests plus two bounded acceptance tests exercise live analyzer construction, target hygiene, narrowing, role shifts, kickoff/rolling pivots, sanctioned replans, opaque/partial delegation, and explicit fire/suppress/no-claim routing. The R6-3.X.2D corpus closeout recorded 110/110 sessions analyzable with 0 emitted fires after remediation. | No new failing witness. Progress or archetype would not make target-continuity reasoning more honest. | **Cutover complete** |
| `truth_grounding_gap` | Detect write/verification action taken without first reading declared truth artifacts; preserve and recover history honestly. | Task-frame truth paths, interval command observations and event order, previous truth-gap score. | Working-set/task-frame extraction supplies the truth paths. | **Not applicable by design:** success/failure does not prove that required truth was read. | **Not applicable by design:** ordering is event-based, not turn-count based. | **Likely not applicable:** pure research/planning without write/verify already has no triggering action. | **Not applicable:** later progress cannot retroactively establish prior grounding. | **Indirect boundary/source only.** | **Direct, required.** | **Proof gap:** ordinary parent orchestration should remain quiet, but there is no explicit control. | Four tests prove ungrounded verification fires, grounded work stays clear, recovery, and historical-only downgrade. | Missing explicit controls for planning/research with no action, successful-but-ungrounded verification, long-turn invariance, and opaque parent orchestration. | **Acceptance-proof gap** |
| `wrong_plan_branch` | Detect write/verification paths outside the task frame's expected truth/working-set scope and clear after a return in scope. | Truth-artifact paths, working-set paths, interval command paths and write/verification classification. | Objective/working-set extraction supplies expected paths; checkpoint boundaries isolate the current interval. | **Not applicable:** command outcome does not change path scope. | **Not applicable:** path scope is event-local. | **Not applicable:** exploration is already ignored unless it writes or verifies. | **Not applicable:** healthy progress cannot excuse mutation outside the authorized branch. | **Potentially relevant through sanctioned continuity, not directly consumed.** | **Direct, required.** | **Proof gap:** parent/child ownership is not exercised. | Two tests prove out-of-scope mutation fires and later in-scope work clears. | Missing controls for read-only exploration, sanctioned replan/path pivot, and delegated-parent orchestration. A failing replan control would identify a bounded implementation gap. | **Acceptance-proof gap** |
| `scoring/mod.rs` | Deterministically build shared scorer inputs, invoke all material scorers, and order results. It is dispatcher infrastructure, not a fifth scorer. | `CheckpointAnalysis`, previous truth score, kickoff anchor. | Builds `SessionProgress` once and supplies it only to `dead_end_thrash`. | Applicable only through scorer-specific routing. | Same. | Same. | Same. | Same. | Same. | Same. | Full analyzer suite proves all four score records are constructed and ordered through the live path. | None; do not force a common mega-context argument into every scorer. | **Fit-for-purpose exception** |

No scorer is currently a justified merge/deprecation candidate. `truth_grounding_gap` asks whether
declared authority was read before action; `wrong_plan_branch` asks whether action stayed inside the
expected path scope. Those are separate failure modes.

## Behavioral-Proof Inventory

### `dead_end_thrash`

| Required behavior | Exact proof | Result |
|---|---|---|
| Advancing troubleshooting frontier tolerates expected repeated failures. | `dead_end_thrash_suppresses_repeated_activity_when_the_frontier_advances`; upstream `checkpoints_mark_troubleshooting_frontier_advancement_from_compile_to_test_failure`; real-rollout-derived progress case `019e899c-453f-71f2-a99d-155848c7b081`. | **Proven** for the troubleshooting-frontier path. |
| Stalled progress increases thrash concern. | `dead_end_thrash_names_stalls_when_repeated_activity_has_no_frontier_movement`; `checkpoints_mark_repeated_same_troubleshooting_signature_as_stalled`. | **Proven** for no-frontier-movement stall. |
| Regressing progress increases/retains concern. | `checkpoints_mark_troubleshooting_regression_when_frontier_falls_back` proves progress construction only. | **Acceptance-proof gap:** no scorer-level regression witness. |
| Clean recovery or verification clears/downgrades honestly. | `dead_end_thrash_clears_after_one_clean_in_scope_verification_interval`; `dead_end_thrash_downgrades_to_historical_only_after_the_recovery_transition`; `dead_end_thrash_clears_replay_shaped_memsrc_verifier_tail`. | **Proven.** |
| Typed outcomes distinguish failure from neutral/success evidence. | `dead_end_thrash_treats_explicit_error_rows_as_repeated_failure_evidence`; `dead_end_thrash_treats_non_zero_exit_code_tool_output_as_failure_evidence`; `dead_end_thrash_ignores_repeated_neutral_tool_output_evidence`; `dead_end_thrash_keeps_repeated_successful_verification_as_historical_context`. | **Proven.** |
| Opaque delegated-parent activity does not become child thrash. | Upstream delegation/progress controls include `checkpoints_progress_falls_back_to_parent_visible_orchestration_for_opaque_parent_work` and `progress_acceptance_r5_75_witnesses_preserve_r6_1_1_frontier_boundary`. | **Acceptance-proof gap:** neither asserts the `DeadEndThrash` score. This is ordinary baseline proof, not an R7 child-link implementation. |
| One long autonomous turn differs from many short conversational turns where relevant. | `checkpoints_recognize_multiple_checkpoints_in_one_long_autonomous_turn`, `checkpoints_mark_tool_free_short_turns_as_conversational`, and `checkpoints_compute_turn_activity_mix_and_execution_modes`. | **Partially proven:** context/archetype construction is proven; no scorer-level A/B demonstrates a material scoring difference. |
| Known replay artifacts become more honest. | `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture`, the three named cleared controls, and `acceptance_fixtures_representative_sticky_success_tail_stays_recovered`. | **Proven for the bounded frozen corpus:** three cleared final states and one recovered sticky-success tail. Do not generalize beyond that corpus. |

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
- real analyzer path: both tests in `semantic_goal_drift_acceptance.rs`.

No new evidence justifies reopening the recently closed semantic-goal-drift family.

### `truth_grounding_gap`

The current design treats grounding as a provenance obligation, not a result-quality claim. A clean
verification can still be ungrounded; a failed verification can still be properly grounded. Turn
length, archetype, and later progress do not alter whether declared truth was read before action.
That is the correct fit-for-purpose direction, but four small behavioral controls are still needed
before the exception is promoted from reasoned design to closure proof.

### `wrong_plan_branch`

The current design treats branch correctness as path authorization. Typed outcomes and progress do
not legalize an out-of-scope write. Read-only exploration should stay quiet because the scorer only
considers write/verification observations. The unresolved question is whether a sanctioned replan
updates the expected path set early and reliably enough; that needs a behavioral witness rather
than an assumption.

## Broad R6 Acceptance-Claim Audit

| Root acceptance claim | Owning scorer/module | Focused proof | Replay/bounded evidence | Status and honest wording |
|---|---|---|---|---|
| Troubleshooting tolerates expected failures while the frontier advances. | `dead_end_thrash` over `SessionProgress`. | Advancement suppression plus upstream frontier tests named above. | Annotated real-rollout troubleshooting fixture `019e899c-...`; frozen dead-end corpus protects final posture. | **Fully proven for troubleshooting-frontier advancement.** |
| Long autonomous turns are evaluated differently from multi-turn conversational sessions. | Turn context + archetype + progress; materially relevant downstream scorer expected to be `dead_end_thrash`. | Construction tests prove long-turn checkpointing and conversational/autonomous classification. | No bounded scorer A/B fixture. | **Partially proven.** Narrow to “turn structure informs archetype/progress construction” unless the closure packet adds a scorer-level witness showing a material difference. |
| Flagged sessions are materially more honest on known replay artifacts. | Primarily `dead_end_thrash`; semantic drift has its separate corpus. | Frozen acceptance assertions. | Four real-rollout-derived dead-end cases: three cleared, one recovered; semantic closeout corpus 110 sessions with no post-remediation fires. | **Proven only for the named bounded corpora.** The root must not imply an unbounded production-quality result. |

## Required Narrow Closure Packet

Open **`R6-C.1 — Scorer Context Applicability Acceptance Controls`**. It is acceptance-first and must
not add context inputs unless a control fails.

Required controls:

1. `dead_end_thrash`: scorer-level regression witness; opaque delegated-parent witness; and one
   long-autonomous versus many-short-conversational A/B, or an explicit finding that the distinction
   is fully consumed upstream and does not change this scorer.
2. `truth_grounding_gap`: planning/research no-action control; successful-but-ungrounded verification
   control; long-turn invariance control; opaque parent-orchestration control.
3. `wrong_plan_branch`: read-only exploration control; sanctioned replan/path-pivot control; opaque
   parent-orchestration control.
4. If every control passes, close as fit-for-purpose without production changes. If any fails, split
   only that failure into a bounded implementation packet with a failing witness and controls.

`semantic_goal_drift` is excluded from this packet absent new failing evidence.

## Verification Run For This Audit

All requested commands passed on 2026-07-12:

```text
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture       PASS (14 focused integration tests; related filtered/unit tests also ran)
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture  PASS (56 focused unit/state tests + 2 acceptance tests)
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture  PASS (4 focused integration tests)
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture    PASS (2 focused integration tests)
cargo test -p agent-drift-analyzer checkpoints -- --nocapture          PASS (131 unit + 35 integration matches, plus related filtered suites)
cargo test -p agent-drift-analyzer -- --nocapture                      PASS (all analyzer test binaries; 0 failures)
```

Passing tests establish the behavior they assert. They do not fill the named acceptance gaps.

## Final R6 Status And R7 Promotion Gate

**R6 status: PARTIAL with named packet `R6-C.1`.** The landed R6 packet history remains intact;
commit `99efda8f9` is not closure authority.

R7 may be promoted from **DRAFT / BLOCKED ON R6 CLOSURE DECISION** to implementation-ready only when:

1. the R6 scorer-by-context applicability audit is complete;
2. every material scorer is classified as cutover complete, intentionally fit-for-purpose exempt,
   or still open with a named bounded packet;
3. the broad R6 acceptance claims have behavioral proof or are narrowed honestly;
4. `R6-C.1` is green, any failing controls have been resolved in bounded R6 packets, and this
   finding is updated to **CLOSED**; and
5. root landing-order authority, R6 MAP, root SPEC/tasks, and R7 status all agree.

Until then, R7 must not absorb unresolved ordinary single-session scorer semantics.
