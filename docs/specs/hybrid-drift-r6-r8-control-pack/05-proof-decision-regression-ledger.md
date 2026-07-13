# Proof, Decision, And Regression Ledger

**Ledger status:** ACTIVE — `R6-C.1-SPEC`

**Verified against:** `d3dcda785d02b1cb98c27e4b701435fb3cebbf6c`

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
| `CTX-R6-03` | `dead_end_thrash` regression | OPEN | Upstream progress regression only | Scorer-level regression disposition. | `R6-C.1` |
| `CTX-R6-04` | Opaque delegated parent does not become child thrash | OPEN | Delegation/progress construction controls only | Scorer-level parent-visible/opaque control. | `R6-C.1` |
| `CTX-R6-05` | Long autonomous vs many short conversational turns | DECISION | Turn/archetype construction tests | Material scorer A/B or proven upstream equivalence plus narrowed charter wording. | `R6-C.1` |
| `CTX-R6-06` | Frozen four-case dead-end corpus | INVARIANCE / OPEN replay preservation | Three cleared final postures and one recovered sticky tail; canonical wording now limits the claim to posture invariance | Preserve through replay; do not call it comparative integrated improvement. | `R6-REPLAY` |
| `CTX-R6-07` | `semantic_goal_drift` context applicability | PROVEN | Focused scorer/state tests prove replan, delegation, and internal routing; the separate live analyzer-path acceptance test proves only its 18 allowlisted fixtures | Preserve the cutover-complete disposition; no reopen absent a new failing witness. | `R6-CLOSE` |
| `CTX-R6-08` | Semantic acceptance fixture integrity | PROVEN integrity, not behavior | The bounded corpus-shape test proves fixture integrity; the separate live analyzer-path test proves its bounded live-path behavior | Preserve the integrity-versus-live-path distinction; do not promote fixture integrity into scorer behavior proof. | `R6-CLOSE` |
| `CTX-R6-09` | `truth_grounding_gap` no-action planning | OPEN | No exact control | No triggering action stays clear. | `R6-C.1` |
| `CTX-R6-10` | Successful but ungrounded verification | OPEN | Ungrounded verification exists without explicit typed-success contrast | Outcome must not fabricate prior grounding. | `R6-C.1` |
| `CTX-R6-11` | Truth grounding long-turn/delegation invariance | OPEN | No exact scorer controls | Long-turn and opaque-parent controls. | `R6-C.1` |
| `CTX-R6-12` | Truth-path-touching action before read | DECISION | Current branch skips both grounded-read and ungrounded-action buckets | Decide and prove whether action implicitly grounds or is a gap; narrow responsibility wording if intentional. | `R6-C.1` |
| `CTX-R6-13` | Actionful planning/research grounding obligation | DECISION | No archetype A/B | Prove action-bound archetype invariance or justify different expectation. | `R6-C.1` |
| `CTX-R6-14` | `wrong_plan_branch` read-only/replan/delegation | OPEN | Two explicit scope/recovery tests only | Three focused controls. | `R6-C.1` |
| `CTX-R6-15` | Wrong-branch path-bearing action with empty authority | DECISION | Current scorer can emit a low-confidence raw-60 flag | Decide no-claim vs intentionally accepted low-confidence claim and prove it. | `R6-C.1` |
| `CTX-R6-16` | `scoring/mod.rs` exact order | PROVEN source / not behavior-tested | Deterministic source sort proves exact order; the full analyzer suite proves live-path carriage, not exact ordering | Do not claim behavioral order proof; add a focused test only if `R6-C.1-SPEC` retains exact order as a closure contract. | `R6-C.1-SPEC`, `R6-CLOSE` |
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
| 2026-07-13 | `d3dcda785` | Marked `R6-C.0A` complete after `959cc50cc` plus the final proof-attribution correction received fresh review clean; preserved the still-open R6 close/replay obligations. |
| 2026-07-13 | `12934a77d` | Bootstrapped the ledger from live repo truth and the fresh R6 closure-matrix review. |
