# Findings: R6 Scorer-Context Cutover Closure

**Date:** 2026-07-12; terminal disposition and authority reconciliation completed 2026-07-15

**Status:** CLOSED

**Scope:** scorer applicability, behavioral-proof audit, control disposition, and bounded named-gap proof

## Decision

The scoped R6 packets and `R6-C.1-CONTROLS` are landed. The thirteen synthetic controls resolved as
`10 PASS / 3 preserved RED` at the
`5618f7864` controls wall, with no production change. In matrix order, those reds route to
`R6-GAP-DET-OPAQUE-PARENT` (`CTX-R6-04`, witness `87409b39a`),
`R6-GAP-TGG-TRUTH-PATH-ACTION` (`CTX-R6-12`, witness `e67d8b214`), and
`R6-GAP-WPB-EMPTY-AUTHORITY` (`CTX-R6-15`, witness `59f098b35`). Production series `bcd94bf4f` +
`931e50c85` + `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN`, making `CTX-R6-04`
proven focused and completing the first named gap. `R6-GAP-TGG-TRUTH-PATH-ACTION` is complete after
its Option-A implementation/proof series and final proof-receipt series `fee9c2b16` + `6674a8316`
received fresh independent built-in `default` `REVIEW CLEAN`. Authority-only transition series
`2937dbe5a` + `91f55f6bf` also received fresh independent built-in `default` `REVIEW CLEAN`. All
three named gaps are complete. Final gap implementation/review-fix
series `6b42e5476` + `e65df2561` + `cd4e24119` received fresh independent built-in `default`
`REVIEW CLEAN`. Authority transition series `56bb9966f` + `07a3b1fe5` received fresh independent
built-in `default` `REVIEW CLEAN` and made `R6-REPLAY` the sole active phase. `CTX-R6-01` is
complete through fresh independent review-clean series `a0089c8de` + `968a4377f`. Historical
`CTX-R6-02` witness `60cde3dd7` remains preserved. Bounded implementation/proof commit `6eda87e60`
passes exact `CTX-R6-02` at the locked `TroubleshootingFrontier / Stalled / Medium` and flagged
`Active / 30 / High` posture with truthful target attribution, plus the complete ordered packet wall
and full analyzer `402 / 402`; a fresh independent built-in `default` reviewer returned
`REVIEW CLEAN`. `CTX-R6-02` and `R6-GAP-DET-REPLAY-STALL` are complete. Packet transition series
`1ff592823` + `7839a7f47` clears active packet to `none`, keeps `R6-REPLAY` active, and received
fresh independent built-in `default` `REVIEW CLEAN`. Phase-owned exact `CTX-R6-01`, exact
`CTX-R6-02`, renamed sticky, and exact `CTX-R6-06` each pass `1 / 1`; the Manifest E family filters
pass `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`; full analyzer passes `402 / 402`;
diff check is green. Current sticky authority remains `HistoricalOnly / 20`, unflagged, while old
`Recovered / 20` remains historical clean-baseline evidence only. Phase-owned proof/fix series
`b1791c1e3` + `e6d43eee9` + `61c9d5074` received fresh independent built-in `default` `REVIEW
CLEAN`, so `R6-REPLAY` is complete with no ordinary replay gap open. On 2026-07-15, `CTX-R6-17`
re-ran the exact `CTX-R6-01`, exact `CTX-R6-02`, renamed sticky, and exact `CTX-R6-06` controls at
`1 / 1` each; the Manifest E filters passed `dead_end_thrash 21 / 21`,
`semantic_goal_drift 58 / 58`, `truth_grounding_gap 22 / 22`, `wrong_plan_branch 6 / 6`, and
`checkpoints 169 / 169`; the full
analyzer completed with all suites green; and `git diff --check` passed. That receipt closes the
terminal-disposition gate without opening any ordinary implementation, acceptance-proof,
merge/deprecation, or deferral route.

`CTX-R6-17` assigns the terminal table exactly as follows: `dead_end_thrash` and
`semantic_goal_drift` are **Cutover complete**; `truth_grounding_gap`, `wrong_plan_branch`, and
dispatcher infrastructure `scoring/mod.rs` are **Fit-for-purpose exception**. R6 is **CLOSED** and
`R6-CLOSE` is **COMPLETE**. The R7 promotion entry gate is satisfied. Promotion series `455d0ed90` +
`876ac55de` completed the R7 content/gate audit, made the R7 authority family implementation-ready,
and received fresh independent built-in `default` `REVIEW CLEAN`, so `R7-PROMOTE` is complete. The
narrow status transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent
built-in `default` `REVIEW CLEAN`. At that historical transition boundary, `R7-0` was active at
entry only with packet `none`, `R7-0.1` was next, and no R7 task had started. This R6 closeout itself
began no R7 work; current R7 task state is recorded below.

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
| `dead_end_thrash` | Distinguish active repeated failure/verification with no frontier movement from expected churn that advances or cleanly recovers. | Repetition history, recovery-active bits, `SessionProgress`, command observations for confidence. | Typed attempts/outcomes feed progress; turn context feeds archetype; archetype selects progress. | **Indirect, relevant.** | **Indirect, relevant.** | **Indirect, relevant.** | **Direct, required.** | **Boundary only.** | **Not applicable.** | **Indirect, relevant.** | Focused gap proof remains review-clean. `CTX-R6-01` integrated advancing replay is review-clean. Historical `CTX-R6-02` witness `60cde3dd7` is preserved; commit `6eda87e60` closes the concurrent-attribution red while preserving exact `Stalled / Active`, passes the ordered packet wall and full analyzer `402 / 402`, and is fresh independent `REVIEW CLEAN`. | `CTX-R6-02` and `R6-GAP-DET-REPLAY-STALL` complete; packet transition `1ff592823` + `7839a7f47` and replay proof/fix series `b1791c1e3` + `e6d43eee9` + `61c9d5074` review-clean; active packet `none`; `CTX-R6-06` and the R6 family wall green. | **Cutover complete** |
| `semantic_goal_drift` | Detect an unsanctioned target pivot relative to kickoff or prior checkpoint while suppressing legitimate narrowing, role shifts, replans, and unsupported delegated-parent claims. | Current structured objective, kickoff anchor, previous structured objective/checkpoint, stable target anchors, sanctioned-replan bit, delegation topology/visibility. | Structured extraction and checkpoint history supply the compared goals. | **Not applicable.** Outcome success does not establish target continuity. | **Not applicable.** | **Not applicable.** | **Not applicable.** | **Direct, required.** | **Not applicable.** | **Direct, required for the bounded opaque-parent guard.** | 56 focused scorer/state tests cover sanctioned replans, opaque/partial delegation, and internal `Fire` / `Suppress` / `NoClaim` routing. Separately, the bounded corpus-shape test proves fixture integrity only. The live analyzer-path acceptance test exercises only its 18 allowlisted pivot, narrowing, progression, and role-shift fixtures; it does not prove replan, delegation, or internal routing. The R6-3.X.2D corpus closeout recorded 110/110 sessions analyzable with 0 emitted fires after remediation. | No new failing witness. Progress or archetype would not make target-continuity reasoning more honest. | **Cutover complete** |
| `truth_grounding_gap` | Detect write/verification action taken without first reading declared truth artifacts; preserve and recover history honestly. | Task-frame truth paths, interval command observations and event order, previous truth-gap score. | Working-set/task-frame extraction supplies the truth paths. | **Not applicable by design:** success/failure does not prove that required truth was read. | **Not applicable by design:** ordering is event-based, not turn-count based. | **Fit-for-purpose for archetype:** equivalent actions scored equally across planning and implementation frames. | **Not applicable:** later progress cannot retroactively establish prior grounding. | **Indirect boundary/source only.** | **Direct, required.** | **Proven for the bounded opaque-parent no-action case.** | Option-A internal path-scoped provenance now preserves action-before-read, historical-only non-grounding, same-path cross-checkpoint carry, path isolation, declaration pruning, session/trajectory isolation, multi-checkpoint carry, and non-consuming reads. Packet-locked controls passed `9 / 9`; the full family passed `22 / 22`; matching checkpoints passed `35` unit + `131` integration plus matching export/provenance tests; exact dead-end regressions and static gates remained green. Final proof-receipt series `fee9c2b16` + `6674a8316` received fresh `REVIEW CLEAN`. | `R6-GAP-TGG-TRUTH-PATH-ACTION` complete; the scorer's path-scoped, event-ordered boundary is behaviorally proven without irrelevant common-context injection. | **Fit-for-purpose exception** |
| `wrong_plan_branch` | Detect write/verification paths outside the task frame's expected truth/working-set scope and clear after a return in scope. | Truth-artifact paths, working-set paths, interval command paths and write/verification classification. | Objective/working-set extraction supplies expected paths; checkpoint boundaries isolate the current interval. | **Not applicable:** command outcome does not change path scope. | **Not applicable:** path scope is event-local. | **Not applicable:** exploration is already ignored unless it writes or verifies. | **Not applicable:** healthy progress cannot excuse mutation outside the authorized branch. | **Relevant through sanctioned continuity; the sanctioned-replan control passed.** | **Direct, required.** | **Proven for the bounded opaque-parent no-action case.** | Read-only exploration, sanctioned replan/path pivot, opaque-parent, and empty-authority controls pass. Historical witness `59f098b35` remains the preserved red receipt; after production fix `6b42e5476`, exact `CTX-R6-15` is `0 / Low / Cleared`, unflagged, with empty evidence, four protected controls pass, family is `6 / 6`, and checkpoint/full-analyzer/static walls are green. | Implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` is fresh independent built-in `default` `REVIEW CLEAN`; `R6-GAP-WPB-EMPTY-AUTHORITY` is complete, and the path-scope boundary is behaviorally proven. | **Fit-for-purpose exception** |
| `scoring/mod.rs` | Deterministically build shared scorer inputs, invoke all material scorers, and order results. It is dispatcher infrastructure, not a fifth scorer. | `CheckpointAnalysis`, previous truth score, kickoff anchor. | Builds `SessionProgress` once and supplies it only to `dead_end_thrash`. | Applicable only through scorer-specific routing. | Same. | Same. | Same. | Same. | Same. | Same. | Source inspection proves the explicit four-class `sort_by_key` order. The full analyzer suite proves the score records travel through the live path, but no focused behavioral assertion proves their exact order. | Exact ordering is source-proven, not behavior-tested. Add a focused assertion only if exact order is retained as a closure contract; do not force a common mega-context argument into every scorer. | **Fit-for-purpose exception** |

No scorer is currently a justified merge/deprecation candidate. `truth_grounding_gap` asks whether
declared authority was read before action; `wrong_plan_branch` asks whether action stayed inside the
expected path scope. Those are separate failure modes.

## Behavioral-Proof Inventory

### `dead_end_thrash`

| Required behavior | Exact proof | Result |
|---|---|---|
| Advancing troubleshooting frontier tolerates expected repeated failures. | Exact `CTX-R6-01` integrated control over trusted rollout `019f1ecb-b93a-7570-8d8d-9ce4e711880b`. | **Proven integrated:** series `a0089c8de` + `968a4377f` fresh independent `REVIEW CLEAN`; checkpoint `7` is advancing and score is `HistoricalOnly / 20 / High`, unflagged. |
| Stalled progress increases thrash concern. | Synthetic stall controls plus trusted depth-1 subagent rollout `019eb311-c7ce-7f50-ae13-b51a5b5461c3`. | **Proven integrated:** historical `60cde3dd7` is preserved; review-clean `6eda87e60` keeps checkpoint `5` `TroubleshootingFrontier / Stalled / Medium`, flagged `Active / 30 / High`, with truthful concurrent output attribution. |
| Regressing progress increases/retains concern. | `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity`; upstream `checkpoints_mark_troubleshooting_regression_when_frontier_falls_back`. | **Proven:** the scorer control passed at `30 / Medium / Active`, flagged. |
| Clean recovery or verification clears/downgrades honestly. | `dead_end_thrash_clears_after_one_clean_in_scope_verification_interval`; `dead_end_thrash_downgrades_to_historical_only_after_the_recovery_transition`; `dead_end_thrash_clears_replay_shaped_memsrc_verifier_tail`. | **Proven.** |
| Typed outcomes distinguish failure from neutral/success evidence. | `dead_end_thrash_treats_explicit_error_rows_as_repeated_failure_evidence`; `dead_end_thrash_treats_non_zero_exit_code_tool_output_as_failure_evidence`; `dead_end_thrash_ignores_repeated_neutral_tool_output_evidence`; `dead_end_thrash_keeps_repeated_successful_verification_as_historical_context`. | **Proven.** |
| Opaque delegated-parent activity does not become child thrash. | `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity`; upstream delegation/progress controls include `checkpoints_progress_falls_back_to_parent_visible_orchestration_for_opaque_parent_work`; the partial/mixed regression protects typed Medium confidence. | **Proven focused:** production series through fresh review-clean `d13f0a71c` yields `0 / Low / Cleared`, unflagged, empty evidence for `CTX-R6-04`, while partial/mixed parent-visible progress stays `0 / Medium / Cleared`, unflagged, empty evidence. |
| Turn-shape relevance is consumed upstream without changing equal-progress scorer output. | `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes`; upstream turn/archetype construction controls. | **Proven focused:** locked long-autonomous and many-short-conversational cases both scored `20 / Medium / HistoricalOnly`, unflagged. The broad acceptance sentence must not claim a scorer-level difference. |
| Frozen replay postures remain invariant. | `acceptance_fixtures_frozen_dead_end_thrash_corpus_keeps_explicit_r6_1_3_posture`. | **Phase-owned `CTX-R6-06` replay proof green at `1 / 1`; proof/fix series fresh independent `REVIEW CLEAN`:** invariance only, not comparative improvement. |

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
preserved action-before-read red is now resolved by typed, session-local, path-scoped grounding
provenance derived only from qualifying reads and event order. The original `CTX-R6-12`,
historical-only non-grounding, same-path cross-checkpoint carry, cross-path isolation, declaration
pruning, session/trajectory isolation, multi-checkpoint carry, and non-consuming reads all pass.
`R6-GAP-TGG-TRUTH-PATH-ACTION` is complete after its final proof-receipt series received fresh
independent `REVIEW CLEAN`. `CTX-R6-17` records the scorer as a **Fit-for-purpose exception**: its
path-scoped, event-ordered inputs match the behavior it owns, and unrelated common context would not
make that behavior more honest.

### `wrong_plan_branch`

The controls confirm that read-only exploration stays quiet, sanctioned replans update the expected
scope for the bounded write case, opaque parent orchestration without attributable child action stays
clear, and empty authority makes no claim. Historical `CTX-R6-15` witness `59f098b35` remains the
preserved red receipt. Production commit `6b42e5476` makes command observations no-claim when the
effective-authority set is empty; exact target is now `0 / Low / Cleared`, unflagged, with empty
evidence. Its canonical
[`SPEC`](R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-spec.md),
[`PLAN`](R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-plan.md), and
[`TASKS`](R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-tasks.md) landed in packet-doc
series `8734f4dbe` + `334e7c6ac` and received fresh independent built-in `default` `REVIEW CLEAN`.
Implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` also received fresh
independent built-in `default` `REVIEW CLEAN`, completing the named gap. Authority transition series
`56bb9966f` + `07a3b1fe5` received fresh independent built-in `default` `REVIEW CLEAN` and activates
only `R6-REPLAY`. Historical `CTX-R6-02` witness `60cde3dd7` remains preserved; bounded
implementation/proof commit `6eda87e60` closes its attribution red, passes the complete ordered
packet wall and full analyzer `402 / 402`, and received fresh independent built-in `default`
`REVIEW CLEAN`. Packet transition series `1ff592823` + `7839a7f47` clears active packet to `none`
and is fresh independent `REVIEW CLEAN`; phase-owned `CTX-R6-06` replay and the R6 family wall are
green, and replay proof/fix series `b1791c1e3` + `e6d43eee9` + `61c9d5074` is fresh independent
built-in `default` `REVIEW CLEAN`. `R6-REPLAY` and `R6-CLOSE` are complete; `CTX-R6-17` records
`wrong_plan_branch` as a **Fit-for-purpose exception**.

## Broad R6 Acceptance-Claim Audit

| Root acceptance claim | Owning scorer/module | Focused proof | Replay/bounded evidence | Status and honest wording |
|---|---|---|---|---|
| Troubleshooting tolerates expected failures while the frontier advances. | `dead_end_thrash` over `SessionProgress`. | Focused advancement suppression plus upstream frontier tests. | Trusted integrated `CTX-R6-01` rollout is review-clean; historical `CTX-R6-02` red `60cde3dd7` is closed by review-clean implementation/proof commit `6eda87e60`. | **Advancing, true-stall, and frozen-corpus cases proven integrated; family wall and replay proof/fix series review-clean.** |
| Long autonomous turns are evaluated differently from multi-turn conversational sessions. | Turn context + archetype + progress; `dead_end_thrash` consumes the derived progress rather than raw turn shape. | Construction tests prove the turn/archetype distinction; `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes` proves equal derived progress produces equal scorer output. | No bounded replay A/B requires a different scorer disposition. | **Narrowed honestly:** turn structure informs archetype/progress construction; it does not independently change `dead_end_thrash` when the relevant derived progress is equal. |
| Flagged sessions are materially more honest on known replay artifacts. | Primarily `dead_end_thrash`; semantic drift has its separate corpus. | Focused scorer controls plus exact trusted replay cases. | `CTX-R6-01` and `CTX-R6-02` are review-clean; `6eda87e60` preserves the correct flagged `Active / 30 / High` disposition with truthful evidence attribution; frozen phase-owned invariance proof and the family wall are green. | **Replay proof complete / proof-fix series review-clean.** Bounded wording is retained; `CTX-R6-17` assigns `dead_end_thrash` **Cutover complete**. |

## R6-C.1 Control Disposition (2026-07-13)

`R6-C.1-CONTROLS` is **COMPLETE**. All thirteen synthetic controls were committed and freshly
reviewed row-by-row; the wall receipt `5618f7864` records `10 PASS / 3 preserved RED`, no new red,
passing checkpoints, and no production change. `semantic_goal_drift` remains excluded absent new
failing evidence.

The three distinct routes must execute sequentially in matrix order:

1. **COMPLETE:** `R6-GAP-DET-OPAQUE-PARENT` for `CTX-R6-04` at witness `87409b39a`; production
   series through `d13f0a71c` is fresh `REVIEW CLEAN` with focused/family/checkpoint proof green.
2. **COMPLETE:** `R6-GAP-TGG-TRUTH-PATH-ACTION` for `CTX-R6-12`; final proof-receipt series
   `fee9c2b16` + `6674a8316` received fresh independent built-in `default` `REVIEW CLEAN`.
3. **COMPLETE:** `R6-GAP-WPB-EMPTY-AUTHORITY` for `CTX-R6-15` at witness `59f098b35`;
   implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` received fresh
   independent built-in `default` `REVIEW CLEAN` with exact, protected, family, checkpoint,
   full-analyzer, and static proof green.

All three original named gap routes are complete. **Historical transition-boundary receipt:**
authority transition series `56bb9966f` + `07a3b1fe5` received fresh independent built-in `default`
`REVIEW CLEAN`, marked aggregate `R6-GAP-*` complete, and activated only `R6-REPLAY` with active
packet `none`; Prompt 1 was then the sole next eligible invocation. The later replay-stall packet is
now complete: `CTX-R6-01` and `CTX-R6-02` are complete, historical witness `60cde3dd7` is preserved,
and implementation/proof commit `6eda87e60` plus its ordered packet proof and full analyzer
`402 / 402` received fresh independent built-in `default` `REVIEW CLEAN`. This authority transition
restores active packet `none` while keeping `R6-REPLAY` active. Current sticky authority remains
`HistoricalOnly / 20`, unflagged, and historical baseline `Recovered / 20` is not current authority.
Packet transition series `1ff592823` + `7839a7f47` is fresh independent `REVIEW CLEAN`; phase-owned
`CTX-R6-06` replay and the R6 family wall are green, and proof/fix series `b1791c1e3` + `e6d43eee9` +
`61c9d5074` received fresh independent built-in `default` `REVIEW CLEAN`. `R6-REPLAY` is complete,
active packet is `none`, and the later 2026-07-15 `CTX-R6-17` receipt assigns the complete terminal
table and closes `R6-CLOSE`. That closeout marks this finding `CLOSED` and activates only
`R7-PROMOTE`; it does not promote the preserved R7 drafts or begin R7 implementation.

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

**R6 status: CLOSED. `R6-REPLAY`, `R6-CLOSE`, and `CTX-R6-17` are complete; active packet is
`none`.** `CTX-R6-02` and `R6-GAP-DET-REPLAY-STALL` are complete after implementation/proof commit
`6eda87e60` received fresh independent built-in `default` `REVIEW CLEAN`; packet transition series
`1ff592823` + `7839a7f47` and replay proof/fix series `b1791c1e3` + `e6d43eee9` + `61c9d5074` are
fresh independent `REVIEW CLEAN`. `CTX-R6-01` is also fresh independent `REVIEW CLEAN`, and
historical witness `60cde3dd7` remains preserved. Current sticky authority is `HistoricalOnly / 20`,
unflagged; old `Recovered / 20` remains historical baseline only. The 2026-07-15 `CTX-R6-17` receipt
reconfirms all four exact replay controls, the five Manifest E family filters, the full analyzer wall,
and `git diff --check`, then assigns the terminal table without a new source/test change.

The R7 promotion entry gate is now satisfied because:

1. the R6 scorer-by-context applicability audit is complete;
2. every material scoring surface has exactly one terminal disposition: **Cutover complete**,
   **Fit-for-purpose exception**, **Merged/deprecated**, or **Explicitly deferred outside R6 with
   justification**; an ordinary “still open” state is not a closure disposition;
3. the broad R6 acceptance claims have behavioral proof or are narrowed honestly;
4. `R6-C.1-CONTROLS` is complete, all three preserved reds have been resolved in their distinct
   bounded named gaps, replay closeout is complete, and this finding is **CLOSED**; and
5. root landing-order authority, R6 MAP, root SPEC/tasks, and R7 status all agree.

That R6 closeout supplied only the `R7-PROMOTE` entry authority and completed no R7 task. Since then,
promotion series `455d0ed90` + `876ac55de` completed the R7 content/gate audit, made the R7
MAP/SPEC/PLAN/TASKS implementation-ready, and received fresh independent built-in `default` `REVIEW
CLEAN`; `R7-PROMOTE` is complete. The narrow status transition series `6bf0ac6ad` + `4a887ee0c` +
`e83ebb430` received fresh independent built-in `default` `REVIEW CLEAN`. Docs-only `R7-0.1` series
`a9e75f149` + `55bea5fa5` + `faff68ac6` and fixture-only `R7-0.2` commit `fa85cd4b8` each received
fresh independent built-in `default` `REVIEW CLEAN`, completing `R7-0`. Transition/fix series
`339744dff` + `d20cac6a9` received fresh independent built-in `default` `REVIEW CLEAN`. R7-1 task
series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh independent built-in
`default` `REVIEW CLEAN`; Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are
complete. R7-2 remains the sole active phase with packet `none` while this checkpoint-doc receipt
still requires fresh independent review. The R7-2 exit gate and `CTX-R7-03`
remain open until the receipt itself is fresh-review-clean. `R7-3..R7-6` and R8 remain
blocked; `R7-3.1` is unchecked and unstarted; no next-phase selectors are prepared or invoked.
