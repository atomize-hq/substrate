# R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **DRAFT / DOCS ONLY** on 2026-07-13. This SPEC/PLAN/TASKS family is the deliverable for
`R6-C.1-SPEC`; it authorizes no test, fixture, or production-code edit until the docs lock is committed
and independently review-clean. All controls remain open in the TASKS ledger.

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
control has a recorded pass or preserved fail result. It does not close R6.

## Locked Decisions

1. **Grounding is provenance, not outcome quality.** A write or verification touching a declared truth
   path before any read is still ungrounded. The expected result is `80 / High / Active`, flagged, with
   authority and action evidence. The live branch currently excludes that action from both grounded-read
   and ungrounded-action buckets; that likely produces an honest red witness. It is not authority because
   it would let the action that creates the obligation satisfy the obligation. `truth_grounding_gap`
   owns whether declared truth was read **before** action; success, path coincidence, progress, turn shape,
   and archetype do not establish prior grounding.
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

## Acceptance Control Matrix

All score triples below describe the final public `DriftScore` after state resolution. "None" under
fixture means an inline integration-test session built with existing row helpers. Every fail route is
conditional: create it only after the named control is committed as a failing witness.

| Ledger | Exact test function | Preconditions / input seam | Expected result / adjudication | Fixture | Focused command | Conditional gap route |
|---|---|---|---|---|---|---|
| `CTX-R6-03` | `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity` | `tests/dead_end_thrash.rs`; repeated active failure history plus `SessionProgress::TroubleshootingFrontier` with regression/no direct-advance signals. | `30 / Medium / Active`, flagged; evidence names stall/regression. Regression must not be suppressed as churn. | None | `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture` | `R6-GAP-DET-REGRESSION` |
| `CTX-R6-04` | `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity` | Opaque delegated-parent / `ParentVisibleOrchestration`; no attributable child command observations or repeated loops. | `0 / Low / Cleared`, unflagged, empty evidence. | None | `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture` | `R6-GAP-DET-OPAQUE-PARENT` |
| `CTX-R6-05` | `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes` | Long-autonomous and many-short-conversational sessions produce equal repeated-failure history and equal direct frontier advancement. | Both `20 / Medium / HistoricalOnly`, unflagged. Adjudication: turn shape is fully consumed upstream; only unequal `SessionProgress` may change the score. | None | `cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_scores_equal_progress_equally_across_turn_shapes -- --exact --nocapture` | `R6-GAP-DET-TURN-EQUIVALENCE` |
| `CTX-R6-09` | `truth_grounding_gap_keeps_no_action_planning_clear` | Declared truth path; planning/research prose; no write-like or verification-like command. | `0 / Medium / Cleared`, unflagged; authority evidence is allowed, no action-gap evidence. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_no_action_planning_clear -- --exact --nocapture` | `R6-GAP-TGG-NO-ACTION` |
| `CTX-R6-10` | `truth_grounding_gap_flags_successful_verification_without_truth_reads` | Declared truth path; typed successful verification outside that path; no earlier truth read. | `80 / High / Active`, flagged. Typed success must not fabricate grounding. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_successful_verification_without_truth_reads -- --exact --nocapture` | `R6-GAP-TGG-SUCCESS-WITHOUT-READ` |
| `CTX-R6-11` | `truth_grounding_gap_is_event_order_invariant_across_turn_shapes` | Same declared truth and same ungrounded action order, represented once as one long autonomous turn and once as many short turns. | Both `80 / High / Active`, flagged, with equivalent evidence semantics. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_is_event_order_invariant_across_turn_shapes -- --exact --nocapture` | `R6-GAP-TGG-TURN-INVARIANCE` |
| `CTX-R6-11` | `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action` | Declared truth path plus opaque parent orchestration; no attributable child write/verification. | `0 / Medium / Cleared`, unflagged; authority evidence only. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture` | `R6-GAP-TGG-OPAQUE-PARENT` |
| `CTX-R6-12` | `truth_grounding_gap_flags_truth_path_action_before_read` | Declared truth path; first write/verification touches that path; no earlier read. | `80 / High / Active`, flagged; evidence includes the authority and action. Likely intentional red witness at current source. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture` | `R6-GAP-TGG-TRUTH-PATH-ACTION` |
| `CTX-R6-13` | `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes` | Planning/research and implementation sessions differ only in archetype/task framing; both take the same write/verification action before reading declared truth. | Both `80 / High / Active`, flagged. Action creates the same provenance obligation in either archetype. | None | `cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture` | `R6-GAP-TGG-ARCHETYPE-INVARIANCE` |
| `CTX-R6-14` | `wrong_plan_branch_ignores_read_only_out_of_scope_exploration` | Non-empty authority; read-only command names an out-of-scope path. | `0 / Medium / Cleared`, unflagged, empty evidence. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_ignores_read_only_out_of_scope_exploration -- --exact --nocapture` | `R6-GAP-WPB-READ-ONLY` |
| `CTX-R6-14` | `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope` | A sanctioned path pivot updates current truth/working-set authority before a write under the new path. | `0 / Medium / Cleared`, unflagged, empty evidence. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_accepts_write_under_sanctioned_replan_scope -- --exact --nocapture` | `R6-GAP-WPB-REPLAN-SCOPE` |
| `CTX-R6-14` | `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action` | Non-empty parent authority plus opaque parent orchestration; no attributable child path-bearing write/verification. | `0 / Medium / Cleared`, unflagged, empty evidence. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture` | `R6-GAP-WPB-OPAQUE-PARENT` |
| `CTX-R6-15` | `wrong_plan_branch_makes_no_claim_for_path_action_without_authority` | Empty truth-artifact and non-observed working-set authority; one path-bearing write/verification. | `0 / Low / Cleared`, unflagged, empty evidence. Likely intentional red witness at current source. | None | `cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture` | `R6-GAP-WPB-EMPTY-AUTHORITY` |
| `CTX-R6-01` | `acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged` | Full analyzer path over an annotated real-rollout-derived session with repeated failures and a directly advancing troubleshooting frontier. | `20 / Medium / HistoricalOnly`, unflagged; fixture annotation must identify the advancement witness. | **Required in `R6-REPLAY`;** exact trusted session ID selected there. | `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged -- --exact --nocapture` | `R6-REPLAY`; if red there, `R6-GAP-DET-REPLAY-ADVANCING` |
| `CTX-R6-02` | `acceptance_fixtures_integrated_true_stall_stays_active` | Full analyzer path over an annotated real-rollout-derived session with repeated failure and no frontier movement. | `30 / Medium / Active`, flagged; fixture annotation must identify the true-stall witness. | **Required in `R6-REPLAY`;** exact trusted session ID selected there. | `cargo test -p agent-drift-analyzer --test acceptance_fixtures acceptance_fixtures_integrated_true_stall_stays_active -- --exact --nocapture` | `R6-REPLAY`; if red there, `R6-GAP-DET-REPLAY-STALL` |

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
git diff --check
```

Run only the active scorer command while iterating. Run the full list after all three scorer-control
commits or preserved witnesses exist. Replay commands run only in `R6-REPLAY` after trusted fixtures are
selected.

## Testing And Gap Strategy

- RED first: add a behavior assertion, run its exact focused command, and record PASS or the exact red
  result before touching production.
- A passing control is fit-for-purpose evidence; do not refactor production merely because a different
  implementation seems cleaner.
- A failing control is committed as a witness. Create one scorer/failure-seam packet named in the row;
  never batch independent failures. The gap packet owns impact analysis, the smallest production fix,
  focused proof, family non-regression, atomic commit, and fresh review.
- Run `npx gitnexus impact <symbol> -r 97a0-substrate --direction upstream --depth 3` before any later
  indexed-symbol edit. Warn/stop on HIGH or CRITICAL impact.
- Run `npx gitnexus detect-changes -r 97a0-substrate`, inspect staged scope, and run `git diff --check`
  before every commit.

## Full R6 Ledger Coverage

| IDs | R6-C.1 disposition |
|---|---|
| `CTX-R6-01`, `CTX-R6-02` | Exact integrated controls specified above; execution remains `R6-REPLAY`. |
| `CTX-R6-03` through `CTX-R6-05` | Open `dead_end_thrash` controls in `R6-C.1-CONTROLS`. |
| `CTX-R6-06` | Frozen four-case posture remains invariance only; preservation stays `R6-REPLAY`. |
| `CTX-R6-07`, `CTX-R6-08` | Preserve semantic scorer completion and fixture-integrity/live-path distinction; no reopen here. |
| `CTX-R6-09` through `CTX-R6-13` | Open `truth_grounding_gap` controls in `R6-C.1-CONTROLS`. |
| `CTX-R6-14`, `CTX-R6-15` | Open `wrong_plan_branch` controls in `R6-C.1-CONTROLS`. |
| `CTX-R6-16` | Source-only dispatcher ordering proof; no focused order test. |
| `CTX-R6-17` | Terminal scorer table remains open for `R6-CLOSE`; this packet cannot fill it. |
| `CTX-R6-18` | Preserve the proven historical/superseded labels. |

## Boundaries

**Always:** preserve failing witnesses; keep one scorer/failure seam per gap packet; use repo-relative
links; update TASKS and `CTX-R6-*` ledger rows only with actual results; dispatch a fresh built-in
`default` reviewer at every plan boundary.

**Escalate:** only for an unresolved authority/product choice, HIGH/CRITICAL GitNexus impact,
unisolatable unrelated work, unavailable trusted replay evidence, or a scope change. Use the structured
`DECISION REQUIRED` / `ACTION REQUIRED` forms in the operator prompt library.

**Never:** edit tests/code/fixtures during `R6-C.1-SPEC`; change production before a red witness; batch
independent gap fixes; absorb a baseline defect into R7; reopen `semantic_goal_drift` without a new
behavior-level failure; execute replay closeout; claim R6 closure; start R7 or R8.

## Success Criteria

1. This SPEC, its PLAN, and TASKS are committed and independently review-clean.
2. Every `CTX-R6-03` through `CTX-R6-05` and `CTX-R6-09` through `CTX-R6-15` control has an exact test,
   input seam, expected disposition, fixture decision, focused command, and one conditional gap route.
3. `CTX-R6-01`/`02` integrated replay ownership and `CTX-R6-16` no-order-test adjudication are explicit.
4. The next phase may activate only as `R6-C.1-CONTROLS`; no control is marked complete by this docs
   phase.

## Non-Goals

- Any code, test, or fixture change during `R6-C.1-SPEC`.
- Integrated replay closeout or trusted-session selection (`R6-REPLAY`).
- R6 terminal dispositions or closure (`R6-CLOSE`).
- R7 delegated-session implementation or R8 Sentinel consolidation.
- Any `semantic_goal_drift` change.
