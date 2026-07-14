# Proof, Decision, And Regression Ledger

**Ledger status:** ACTIVE — `R6-GAP-TGG-TRUTH-PATH-ACTION` (docs-only gate; active packet none)

**Verified against:** production series `bcd94bf4f` + `931e50c85` + `d13f0a71c`, fresh built-in `default` `REVIEW CLEAN`, with the exact ordered proof below

## Status Values

- `OPEN`: work or proof is required in the named phase.
- `DECISION`: a control must adjudicate the intended behavior.
- `BLOCKED`: depends on an earlier phase.
- `PROVEN`: behavior-level proof is recorded.
- `INVARIANCE`: a corpus posture is frozen but does not prove comparative improvement.
- `DEFERRED`: explicitly outside the active phase with owner and trigger.
- `CLOSED`: terminal disposition and authority reconciliation complete.

## R6 Closure Ledger

| ID | Surface / claim | Current state | Evidence now | Required next proof or decision | Owning phase |
|---|---|---|---|---|---|
| `CTX-R6-01` | `dead_end_thrash` advancing frontier suppression | PROVEN focused / OPEN integrated replay | Synthetic scorer control plus upstream real-rollout progress fixture | Integrated real-rollout-derived advancing repeated-failure score must be unflagged/historical. | `R6-C.1`, `R6-REPLAY` |
| `CTX-R6-02` | `dead_end_thrash` true stall | PROVEN focused / OPEN integrated replay | Synthetic no-frontier-movement scorer control | Annotated real-rollout-derived true-stall positive score. | `R6-REPLAY` |
| `CTX-R6-03` | `dead_end_thrash` regression | PROVEN focused | Exact control `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity` passed with `30 / Medium / Active`, flagged; `TroubleshootingFrontier` was `Regressing`, carried `PreviouslyCleanScopeBroken`, carried no direct frontier-advance signal, and the scorer named both frontier fallback and current repeated-failure evidence. | Preserve through the reviewed `dead_end_thrash` family wall; no `R6-GAP-DET-REGRESSION` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-04` | Opaque delegated parent does not become child thrash | PROVEN FOCUSED / GAP COMPLETE | The preserved pre-edit control was red at `0 / Medium / Cleared`, unflagged, empty evidence. Production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN`: the exact control is `0 / Low / Cleared`, unflagged, empty evidence; the new partial/mixed parent-visible regression is `0 / Medium / Cleared`, unflagged, empty evidence; `CTX-R6-03` remains `30 / Medium / Active`, flagged; all `21` matching `dead_end_thrash` tests and all `167` matching checkpoint tests passed; and format/check passed. | Preserve the focused proof through later walls. Assign no terminal scorer disposition and make no replay or R6-close claim here. | `R6-GAP-DET-OPAQUE-PARENT` (complete) |
| `CTX-R6-05` | Long autonomous vs many short conversational turns | PROVEN focused | Exact control `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes` passed for locked long-autonomous and many-short-conversational modes with identical advancing `TroubleshootingFrontier` progress and direct `FailureFrontierAdvanced` evidence; both failure-only histories scored `20 / Medium / HistoricalOnly`, unflagged, with no repeated-verification evidence. | Preserve through the reviewed `dead_end_thrash` family wall; no `R6-GAP-DET-TURN-EQUIVALENCE` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-06` | Frozen four-case dead-end corpus | INVARIANCE / OPEN replay preservation | Three cleared final postures and one recovered sticky tail; canonical wording now limits the claim to posture invariance | Preserve through replay; do not call it comparative integrated improvement. | `R6-REPLAY` |
| `CTX-R6-07` | `semantic_goal_drift` context applicability | PROVEN | Focused scorer/state tests prove replan, delegation, and internal routing; the separate live analyzer-path acceptance test proves only its 18 allowlisted fixtures | Preserve the cutover-complete disposition; no reopen absent a new failing witness. | `R6-CLOSE` |
| `CTX-R6-08` | Semantic acceptance fixture integrity | PROVEN integrity, not behavior | The bounded corpus-shape test proves fixture integrity; the separate live analyzer-path test proves its bounded live-path behavior | Preserve the integrity-versus-live-path distinction; do not promote fixture integrity into scorer behavior proof. | `R6-CLOSE` |
| `CTX-R6-09` | `truth_grounding_gap` no-action planning | PROVEN focused | Exact control `truth_grounding_gap_keeps_no_action_planning_clear` passed with `0 / Medium / Cleared`, unflagged; the declared truth path plus planning/research prose and no write/verification retained only `truth artifact hint:` authority evidence. | Preserve through the reviewed `truth_grounding_gap` family wall; no `R6-GAP-TGG-NO-ACTION` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-10` | Successful but ungrounded verification | PROVEN focused | Exact control `truth_grounding_gap_flags_successful_verification_without_truth_reads` passed with `80 / High / Active`, flagged; a typed successful `cargo test` outside the declared truth path and no earlier truth read retained both the `truth artifact hint:` authority and ungrounded `command family: cargo` evidence. | Preserve through the reviewed `truth_grounding_gap` family wall; no `R6-GAP-TGG-SUCCESS-WITHOUT-READ` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-11` | Truth grounding long-turn/delegation invariance | PROVEN focused | Exact controls passed both halves: `truth_grounding_gap_is_event_order_invariant_across_turn_shapes` produced `80 / High / Active`, flagged, in both locked turn representations with equivalent `truth artifact hint:` and `command family: cargo` evidence semantics; `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action` produced `0 / Medium / Cleared`, unflagged, for opaque parent orchestration with no attributable child action and retained authority evidence only. | Preserve both results through the reviewed `truth_grounding_gap` family wall; do not create `R6-GAP-TGG-TURN-INVARIANCE` or `R6-GAP-TGG-OPAQUE-PARENT`. | `R6-C.1-CONTROLS` |
| `CTX-R6-12` | Truth-path-touching action before read | PRESERVED RED / GAP ACTIVE — DOCS ONLY | Exact control `truth_grounding_gap_flags_truth_path_action_before_read` declared one truth path and made the first write-like `apply_patch` action touch that same path before any read, but failed the locked triple: actual `0 / Medium / Cleared`, unflagged, versus required `80 / High / Active`, flagged, with authority and action evidence. | Atomically create and freshly review only the three canonical non-link `TO CREATE` packet paths in the named-gap subledger. Do not execute the gap or cite a successor TASKS as existing first. | `R6-GAP-TGG-TRUTH-PATH-ACTION` |
| `CTX-R6-13` | Actionful planning/research grounding obligation | PROVEN focused | Exact control `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes` passed when task prose alone produced Planning and AutonomousImplementation frames that declared the same truth path and performed one byte-identical write-like `apply_patch` command against the same target before any truth read; both scored `80 / High / Active`, flagged, with equivalent `truth artifact hint:` and `command family: apply_patch` evidence semantics. | Preserve through the reviewed `truth_grounding_gap` family wall; no `R6-GAP-TGG-ARCHETYPE-INVARIANCE` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-14` | `wrong_plan_branch` read-only/replan/delegation | PROVEN focused | Exact controls passed all three halves: `wrong_plan_branch_ignores_read_only_out_of_scope_exploration` produced `0 / Medium / Cleared`, unflagged, with empty evidence for non-empty source/truth authority plus a read-only `sed` command naming a different path; `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope` produced the same exact disposition after a user steer updated current truth/working-set authority before an `apply_patch` write under the sanctioned new path; `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action` again produced the same exact disposition for non-empty parent authority plus `ParentVisibleOrchestration` and no attributable child path-bearing write/verification. | Preserve all three results through the reviewed `wrong_plan_branch` family wall; do not create `R6-GAP-WPB-READ-ONLY`, `R6-GAP-WPB-REPLAN-SCOPE`, or `R6-GAP-WPB-OPAQUE-PARENT`. | `R6-C.1-CONTROLS` |
| `CTX-R6-15` | Wrong-branch path-bearing action with empty authority | PRESERVED RED / GAP BLOCKED | Exact control `wrong_plan_branch_makes_no_claim_for_path_action_without_authority` constructed empty truth-artifact authority and exactly one `observed_command` working-set path from one path-bearing write-like `apply_patch` action, then failed the locked triple: actual `60 / Low / Active`, flagged, with command action evidence, versus required `0 / Low / Cleared`, unflagged, with empty evidence. | Preserve the committed witness; wait for `R6-GAP-TGG-TRUTH-PATH-ACTION` to complete and transition this distinct route active. | `R6-GAP-WPB-EMPTY-AUTHORITY` |
| `CTX-R6-16` | `scoring/mod.rs` exact order | PROVEN source / not retained as behavioral closure contract | Deterministic source sort proves exact order; the landed [R6-C.1 packet SPEC](../r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md) explicitly declines to retain exact dispatcher order as a behavioral closure contract. | Add no focused ordering control unless later behavior evidence makes order load-bearing; do not claim behavioral order proof. | `R6-CLOSE` |
| `CTX-R6-17` | R6 terminal disposition gate | OPEN for `R6-CLOSE` | The four terminal categories are now enforced across root/R6/R7 authority; ordinary “still open” is forbidden at closure | Complete the terminal table and reconcile the authority stack during `R6-CLOSE`; do not close it during planning. | `R6-CLOSE` |
| `CTX-R6-18` | Historical pre-cutover status prose | PROVEN | The R6 MAP preliminary investigation and root 2026-07-04 status note are explicitly labeled historical/superseded | Preserve the labels; historical status must not override live posture. | `R6-C.0A` (complete) |

## Named R6 Gap Status Subledger

| Phase ID | Preserved witness/control | Owning packet docs | Status | Predecessor | Successor | Evidence/commit |
|---|---|---|---|---|---|---|
| `R6-GAP-DET-OPAQUE-PARENT` | `CTX-R6-04` — `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity` | `docs/specs/r6/R6-GAP-DET-OPAQUE-PARENT/R6-GAP-DET-OPAQUE-PARENT-spec.md`<br>`docs/specs/r6/R6-GAP-DET-OPAQUE-PARENT/R6-GAP-DET-OPAQUE-PARENT-plan.md`<br>`docs/specs/r6/R6-GAP-DET-OPAQUE-PARENT/R6-GAP-DET-OPAQUE-PARENT-tasks.md` | COMPLETE | `R6-C.1-CONTROLS` | `R6-GAP-TGG-TRUTH-PATH-ACTION` | Witness `87409b39a`; controls wall `5618f7864`; packet docs `59092df2a` `REVIEW CLEAN`; ledger reconciliation `beed76446` `REVIEW CLEAN`; production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` fresh built-in `default` `REVIEW CLEAN`. Focused `CTX-R6-04`, the partial/mixed regression, and `CTX-R6-03` passed their exact dispositions; all `21` matching `dead_end_thrash` tests, all `167` matching checkpoint tests, and format/check passed. |
| `R6-GAP-TGG-TRUTH-PATH-ACTION` | `CTX-R6-12` — `truth_grounding_gap_flags_truth_path_action_before_read` | TO CREATE `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-spec.md`<br>TO CREATE `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-plan.md`<br>TO CREATE `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-tasks.md` | ACTIVE | `R6-GAP-DET-OPAQUE-PARENT` | `R6-GAP-WPB-EMPTY-AUTHORITY` | Witness `e67d8b214`; controls wall `5618f7864`. Sole next action: atomically create and freshly review the three non-link `TO CREATE` docs; no packet file exists yet. |
| `R6-GAP-WPB-EMPTY-AUTHORITY` | `CTX-R6-15` — `wrong_plan_branch_makes_no_claim_for_path_action_without_authority` | TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-spec.md`<br>TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-plan.md`<br>TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-tasks.md` | BLOCKED | `R6-GAP-TGG-TRUTH-PATH-ACTION` | `R6-REPLAY` | Witness `59f098b35`; controls wall `5618f7864`. |

## R6 Terminal Disposition Table

Complete this table during `R6-CLOSE`. Interim values are not closure values.

| Surface | Interim posture | Terminal disposition | Proof artifact | Closure commit |
|---|---|---|---|---|
| `dead_end_thrash` | Acceptance-proof gap | TBD | TBD | TBD |
| `semantic_goal_drift` | Cutover complete | **Cutover complete** | Existing R6 closeout + corrected proof inventory; reopen only on a new failing witness | TBD |
| `truth_grounding_gap` | Acceptance-proof gap | TBD | TBD | TBD |
| `wrong_plan_branch` | Acceptance-proof gap | TBD | TBD | TBD |
| `scoring/mod.rs` | Dispatcher / fit-for-purpose routing | TBD | Source + any contract test deemed necessary | TBD |

## R7 Gate Ledger

| ID | Gate | Status | Required proof |
|---|---|---|---|
| `CTX-R7-01` | R6 baseline closed | BLOCKED | R6 finding `CLOSED`, terminal scorer table complete, authority stack aligned. |
| `CTX-R7-02` | Reciprocal direct linkage contract | BLOCKED | Sanitized positive/negative fixtures and compactor typed contract. |
| `CTX-R7-03` | Separate trajectories | BLOCKED | Parent/child progress and scorer ownership controls. |
| `CTX-R7-04` | Direct-child first boundary | BLOCKED | Deeper descendants remain explicit unsupported residue. |
| `CTX-R7-05` | No new drift class by default | BLOCKED | Acceptance evidence supports existing classes plus delegation context. |
| `CTX-R7-06` | Minimal sentinel compatibility | BLOCKED | Typed analyzer semantics carried end to end without R8 consolidation. |

## R8 Gate Ledger

| ID | Gate | Status | Required proof |
|---|---|---|---|
| `CTX-R8-01` | Stable analyzer/delegation contract | BLOCKED | R7 closeout and compatibility contract. |
| `CTX-R8-02` | R8 MAP/SPEC/PLAN/TASKS | BLOCKED | Reviewed Sentinel Interpretation Consolidation / Integration family. |
| `CTX-R8-03` | Shared replay/live interpretation seam | BLOCKED | Both paths use one typed checkpoint-interpretation seam. |
| `CTX-R8-04` | Central compatibility | BLOCKED | Version/legacy logic is localized and behaviorally covered. |
| `CTX-R8-05` | Presentation-first operator surface | BLOCKED | No analyzer semantic inference leaks into rendering. |
| `CTX-R8-06` | Scheduling/adjudication boundary | BLOCKED | No redesign unless separately specced and approved. |

## Regression Rules

- Never upgrade `INVARIANCE` to comparative improvement without an integrated before/after or
  expected-disposition witness.
- Never mark a construction/serialization test as scorer behavior proof.
- Never close an acceptance-proof gap with transitive field availability.
- Never convert missing proof into a production change without a failing control.
- Never mark R7 or R8 ready by editing only this ledger; canonical authority must change first.

## Update Record

| Date | Commit verified | Change |
|---|---|---|
| 2026-07-13 | `d13f0a71c` (production-series review correction) | Production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN` with the exact focused, family, checkpoint, and format/check proof green. This authority update marks `R6-GAP-DET-OPAQUE-PARENT` complete, activates only `R6-GAP-TGG-TRUTH-PATH-ACTION` at its docs-only gate, and keeps the later gap plus replay blocked, without claiming an unknown transition commit or review verdict. |
| 2026-07-13 | `931e50c85` (landed review-fix candidate) | Narrowed the confidence exception to typed `ParentVisibleOrchestration + Low`, added the partial/mixed parent-visible Medium-confidence regression, and retained exact ordered proof passing all three focused controls, all `21` matching `dead_end_thrash` tests, and all `167` matching checkpoint tests. Fresh re-review returned one P2 bookkeeping finding because the TASKS and ledger still described this landed candidate as an uncommitted worktree. At that commit, the bookkeeping was corrected while `R6-GAP-DET-OPAQUE-PARENT` remained ACTIVE pending fresh re-review and a separate transition; the later `d13f0a71c` row supersedes that phase status. |
| 2026-07-13 | `bcd94bf4f` | Landed the initial scorer-local production fix with green focused and family proof. Fresh review returned two required findings: the no-history `ParentVisibleOrchestration` exception also suppressed command confidence for partial/mixed Medium progress, and the TASKS/ledger still described the landed commit as pending worktree. The commit remains review-blocked. |
| 2026-07-13 | `beed76446` | Committed the ledger-only post-docs reconciliation and received fresh `REVIEW CLEAN`; retained the preserved red and authorized witness reconfirmation as the next action. |
| 2026-07-13 | `59092df2a` | Reconciled the `R6-GAP-DET-OPAQUE-PARENT` packet docs after fresh `REVIEW CLEAN`; at that commit the gap remained active and preserved red, with exact witness/family reconfirmation next and no-code closure unavailable. |
| 2026-07-13 | `5618f7864` | Closed `R6-C.1-CONTROLS` after the wall reconciled all thirteen synthetic controls as `10 PASS / 3 preserved RED`; at that transition, instantiated the three matrix-ordered named gap routes, activated only `R6-GAP-DET-OPAQUE-PARENT` at its packet-docs gate, kept the later gaps and `R6-REPLAY` blocked, and recorded no production change. |
| 2026-07-13 | `ea19b39a7` | Completed `R6-C.1-SPEC` after the artifact series `253e634fe`, `d3430eff3`, `58535df60`, `12f042f6b`, and `ea19b39a7` landed the packet docs and received final fresh `REVIEW CLEAN`; activated controls without claiming behavioral proof or creating a named gap subledger. |
| 2026-07-13 | `d3dcda785` | Marked `R6-C.0A` complete after `959cc50cc` plus the final proof-attribution correction received fresh review clean; preserved the still-open R6 close/replay obligations. |
| 2026-07-13 | `12934a77d` | Bootstrapped the ledger from live repo truth and the fresh R6 closure-matrix review. |
