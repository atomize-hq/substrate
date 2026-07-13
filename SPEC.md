# Active Spec: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-C.1-SPEC` (ACTIVE, docs only; active packet: none)**

The scoped R6 packets have landed, but the broader context-aware scorer-cutover charter is not
closed for sequencing. The active objective is to close the smallest remaining behavioral-proof
gaps without mechanically injecting typed outcomes, turn context, archetype, or progress into
scorers where those layers are not semantically relevant.

Hard decisions:

- `semantic_goal_drift` is cutover complete by design. It consumes structured objectives, stable
  target anchors, sanctioned replans, delegation visibility, and checkpoint history. Do not reopen
  it without new failing evidence.
- `dead_end_thrash` has landed its progress-aware core, but still needs scorer-level acceptance
  controls for regression, opaque delegated-parent activity, and the long-autonomous versus
  many-short-conversational claim. Its frozen four-case corpus is posture invariance, not
  comparative integrated improvement; broad replay honesty remains partially / bounded proven.
- `truth_grounding_gap` retains its current behavior unless controls for truth-path-touching action
  before a read and actionful planning/research, plus the previously named applicability controls,
  expose a failing witness.
- `wrong_plan_branch` likewise needs an empty-authority path-bearing action control in addition to
  the previously named read-only, replan, and delegated-parent controls.
- Transitive data availability is not behavioral integration; non-applicable context is an explicit
  fit-for-purpose decision, not missing plumbing.
- The semantic acceptance corpus-shape test proves fixture integrity; the separate live
  analyzer-path test proves behavior. Dispatcher order is explicit in source but not covered by a
  focused behavioral order assertion.
- R7 remains **DRAFT / BLOCKED ON R6 CLOSURE DECISION** and must not absorb unresolved ordinary
  single-session scorer semantics.

`R6-C.0A` is complete at `d3dcda785`: its seven authority defects were corrected and the remediation
received fresh `REVIEW CLEAN`. The sole authorized current action is docs-only `R6-C.1-SPEC`: write
and review the bounded SPEC/PLAN/TASKS for `R6-C.1 — Scorer Context Applicability Acceptance
Controls`. Acceptance tests, production changes, replay closeout, R6 closure, and all R7/R8 work
remain blocked. If the later controls pass, close the relevant exceptions without production
changes. If a control fails, create only the bounded scorer-specific R6 gap packet that the failing
witness proves necessary.

R7 promotion requires the applicability audit to be complete, broad R6 acceptance claims
behaviorally proven or narrowed honestly, the R6 finding updated to `CLOSED`, and all root/R6/R7
authority documents reconciled. Each material scoring surface must end in exactly one terminal
category: **Cutover complete**, **Fit-for-purpose exception**,
**Merged/deprecated**, or **Explicitly deferred outside R6 with justification**. Ordinary “still
open” is not a closure disposition.
