# Proof, Decision, And Regression Ledger

**Ledger status:** ACTIVE — `R6-C.1-CONTROLS`

**Verified against:** `ea19b39a7`

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
| `CTX-R6-04` | Opaque delegated parent does not become child thrash | PRESERVED RED / GAP REQUIRED | Exact control `dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity` reached `ParentVisibleOrchestration` but failed the locked triple: actual `0 / Medium / Cleared`, unflagged, empty evidence, versus required `0 / Low / Cleared`. | Preserve the committed witness through the controls phase and route `R6-GAP-DET-OPAQUE-PARENT` for later activation; do not change production during `R6-C.1-CONTROLS`. | `R6-C.1-CONTROLS`, `R6-GAP-DET-OPAQUE-PARENT` |
| `CTX-R6-05` | Long autonomous vs many short conversational turns | PROVEN focused | Exact control `dead_end_thrash_scores_equal_progress_equally_across_turn_shapes` passed for locked long-autonomous and many-short-conversational modes with identical advancing `TroubleshootingFrontier` progress and direct `FailureFrontierAdvanced` evidence; both failure-only histories scored `20 / Medium / HistoricalOnly`, unflagged, with no repeated-verification evidence. | Preserve through the reviewed `dead_end_thrash` family wall; no `R6-GAP-DET-TURN-EQUIVALENCE` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-06` | Frozen four-case dead-end corpus | INVARIANCE / OPEN replay preservation | Three cleared final postures and one recovered sticky tail; canonical wording now limits the claim to posture invariance | Preserve through replay; do not call it comparative integrated improvement. | `R6-REPLAY` |
| `CTX-R6-07` | `semantic_goal_drift` context applicability | PROVEN | Focused scorer/state tests prove replan, delegation, and internal routing; the separate live analyzer-path acceptance test proves only its 18 allowlisted fixtures | Preserve the cutover-complete disposition; no reopen absent a new failing witness. | `R6-CLOSE` |
| `CTX-R6-08` | Semantic acceptance fixture integrity | PROVEN integrity, not behavior | The bounded corpus-shape test proves fixture integrity; the separate live analyzer-path test proves its bounded live-path behavior | Preserve the integrity-versus-live-path distinction; do not promote fixture integrity into scorer behavior proof. | `R6-CLOSE` |
| `CTX-R6-09` | `truth_grounding_gap` no-action planning | PROVEN focused | Exact control `truth_grounding_gap_keeps_no_action_planning_clear` passed with `0 / Medium / Cleared`, unflagged; the declared truth path plus planning/research prose and no write/verification retained only `truth artifact hint:` authority evidence. | Preserve through the reviewed `truth_grounding_gap` family wall; no `R6-GAP-TGG-NO-ACTION` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-10` | Successful but ungrounded verification | PROVEN focused | Exact control `truth_grounding_gap_flags_successful_verification_without_truth_reads` passed with `80 / High / Active`, flagged; a typed successful `cargo test` outside the declared truth path and no earlier truth read retained both the `truth artifact hint:` authority and ungrounded `command family: cargo` evidence. | Preserve through the reviewed `truth_grounding_gap` family wall; no `R6-GAP-TGG-SUCCESS-WITHOUT-READ` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-11` | Truth grounding long-turn/delegation invariance | PROVEN focused | Exact controls passed both halves: `truth_grounding_gap_is_event_order_invariant_across_turn_shapes` produced `80 / High / Active`, flagged, in both locked turn representations with equivalent `truth artifact hint:` and `command family: cargo` evidence semantics; `truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action` produced `0 / Medium / Cleared`, unflagged, for opaque parent orchestration with no attributable child action and retained authority evidence only. | Preserve both results through the reviewed `truth_grounding_gap` family wall; do not create `R6-GAP-TGG-TURN-INVARIANCE` or `R6-GAP-TGG-OPAQUE-PARENT`. | `R6-C.1-CONTROLS` |
| `CTX-R6-12` | Truth-path-touching action before read | PRESERVED RED / GAP REQUIRED | Exact control `truth_grounding_gap_flags_truth_path_action_before_read` declared one truth path and made the first write-like `apply_patch` action touch that same path before any read, but failed the locked triple: actual `0 / Medium / Cleared`, unflagged, versus required `80 / High / Active`, flagged, with authority and action evidence. | Preserve the committed witness through the controls phase and route `R6-GAP-TGG-TRUTH-PATH-ACTION` for later activation; do not change production during `R6-C.1-CONTROLS`. | `R6-C.1-CONTROLS`, `R6-GAP-TGG-TRUTH-PATH-ACTION` |
| `CTX-R6-13` | Actionful planning/research grounding obligation | PROVEN focused | Exact control `truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes` passed when task prose alone produced Planning and AutonomousImplementation frames that declared the same truth path and performed one byte-identical write-like `apply_patch` command against the same target before any truth read; both scored `80 / High / Active`, flagged, with equivalent `truth artifact hint:` and `command family: apply_patch` evidence semantics. | Preserve through the reviewed `truth_grounding_gap` family wall; no `R6-GAP-TGG-ARCHETYPE-INVARIANCE` route is required. | `R6-C.1-CONTROLS` |
| `CTX-R6-14` | `wrong_plan_branch` read-only/replan/delegation | PROVEN focused | Exact controls passed all three halves: `wrong_plan_branch_ignores_read_only_out_of_scope_exploration` produced `0 / Medium / Cleared`, unflagged, with empty evidence for non-empty source/truth authority plus a read-only `sed` command naming a different path; `wrong_plan_branch_accepts_write_under_sanctioned_replan_scope` produced the same exact disposition after a user steer updated current truth/working-set authority before an `apply_patch` write under the sanctioned new path; `wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action` again produced the same exact disposition for non-empty parent authority plus `ParentVisibleOrchestration` and no attributable child path-bearing write/verification. | Preserve all three results through the reviewed `wrong_plan_branch` family wall; do not create `R6-GAP-WPB-READ-ONLY`, `R6-GAP-WPB-REPLAN-SCOPE`, or `R6-GAP-WPB-OPAQUE-PARENT`. | `R6-C.1-CONTROLS` |
| `CTX-R6-15` | Wrong-branch path-bearing action with empty authority | PRESERVED RED / GAP REQUIRED | Exact control `wrong_plan_branch_makes_no_claim_for_path_action_without_authority` constructed empty truth-artifact authority and exactly one `observed_command` working-set path from one path-bearing write-like `apply_patch` action, then failed the locked triple: actual `60 / Low / Active`, flagged, with command action evidence, versus required `0 / Low / Cleared`, unflagged, with empty evidence. | Preserve the committed witness through the controls phase and route `R6-GAP-WPB-EMPTY-AUTHORITY` for later activation; do not change production during `R6-C.1-CONTROLS`. | `R6-C.1-CONTROLS`, `R6-GAP-WPB-EMPTY-AUTHORITY` |
| `CTX-R6-16` | `scoring/mod.rs` exact order | PROVEN source / not retained as behavioral closure contract | Deterministic source sort proves exact order; the landed [R6-C.1 packet SPEC](../r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md) explicitly declines to retain exact dispatcher order as a behavioral closure contract. | Add no focused ordering control unless later behavior evidence makes order load-bearing; do not claim behavioral order proof. | `R6-CLOSE` |
| `CTX-R6-17` | R6 terminal disposition gate | OPEN for `R6-CLOSE` | The four terminal categories are now enforced across root/R6/R7 authority; ordinary “still open” is forbidden at closure | Complete the terminal table and reconcile the authority stack during `R6-CLOSE`; do not close it during planning. | `R6-CLOSE` |
| `CTX-R6-18` | Historical pre-cutover status prose | PROVEN | The R6 MAP preliminary investigation and root 2026-07-04 status note are explicitly labeled historical/superseded | Preserve the labels; historical status must not override live posture. | `R6-C.0A` (complete) |

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
| 2026-07-13 | `ea19b39a7` | Completed `R6-C.1-SPEC` after the artifact series `253e634fe`, `d3430eff3`, `58535df60`, `12f042f6b`, and `ea19b39a7` landed the packet docs and received final fresh `REVIEW CLEAN`; activated controls without claiming behavioral proof or creating a named gap subledger. |
| 2026-07-13 | `d3dcda785` | Marked `R6-C.0A` complete after `959cc50cc` plus the final proof-attribution correction received fresh review clean; preserved the still-open R6 close/replay obligations. |
| 2026-07-13 | `12934a77d` | Bootstrapped the ledger from live repo truth and the fresh R6 closure-matrix review. |
