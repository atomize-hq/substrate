# Active Spec: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-GAP-DET-OPAQUE-PARENT` (ACTIVE; active packet: none; packet-docs gate only)**

The scoped R6 packets have landed, but the broader context-aware scorer-cutover charter is not
closed for sequencing. The active objective is to close the smallest remaining behavioral-proof
gaps without mechanically injecting typed outcomes, turn context, archetype, or progress into
scorers where those layers are not semantically relevant.

Hard decisions:

- `semantic_goal_drift` is cutover complete by design. It consumes structured objectives, stable
  target anchors, sanctioned replans, delegation visibility, and checkpoint history. Do not reopen
  it without new failing evidence.
- `dead_end_thrash` passed its regression and turn-shape-equivalence controls but preserved the
  opaque-parent confidence red `CTX-R6-04`; `R6-GAP-DET-OPAQUE-PARENT` is the sole active named route.
  Its frozen four-case corpus remains posture invariance, not comparative integrated improvement.
- `truth_grounding_gap` passed its other applicability controls but preserved the truth-path action
  red `CTX-R6-12`; `R6-GAP-TGG-TRUTH-PATH-ACTION` is blocked behind the active dead-end gap.
- `wrong_plan_branch` passed its read-only, sanctioned-replan, and delegated-parent controls but
  preserved the empty-authority red `CTX-R6-15`; `R6-GAP-WPB-EMPTY-AUTHORITY` is blocked behind the
  grounding gap.
- Transitive data availability is not behavioral integration; non-applicable context is an explicit
  fit-for-purpose decision, not missing plumbing.
- The semantic acceptance corpus-shape test proves fixture integrity; the separate live
  analyzer-path test proves behavior. Dispatcher order is explicit in source but not covered by a
  focused behavioral order assertion.
- R7 remains **DRAFT / BLOCKED ON R6 CLOSURE DECISION** and must not absorb unresolved ordinary
  single-session scorer semantics.

`R6-C.0A` is complete at `d3dcda785`; `R6-C.1-SPEC` is complete through review-clean `ea19b39a7`;
and `R6-C.1-CONTROLS` is complete against the wall receipt `5618f7864`. The thirteen synthetic
controls resolved as `10 PASS / 3 preserved RED`, with no production change. The sole authorized
current action is atomic creation and fresh review of the three canonical
`R6-GAP-DET-OPAQUE-PARENT` packet docs recorded as non-link `TO CREATE` paths in the named-gap
subledger; those files do not yet exist. Gap execution, replay closeout, R6 closure, and all R7/R8
work remain blocked.

R7 promotion requires the applicability audit to be complete, broad R6 acceptance claims
behaviorally proven or narrowed honestly, the R6 finding updated to `CLOSED`, and all root/R6/R7
authority documents reconciled. Each material scoring surface must end in exactly one terminal
category: **Cutover complete**, **Fit-for-purpose exception**,
**Merged/deprecated**, or **Explicitly deferred outside R6 with justification**. Ordinary “still
open” is not a closure disposition.
