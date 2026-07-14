# Active Spec: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-GAP-WPB-EMPTY-AUTHORITY` (ACTIVE; active packet: `R6-GAP-WPB-EMPTY-AUTHORITY`; exact `CTX-R6-15` witness reconfirmation gate)**

The scoped R6 packets have landed, but the broader context-aware scorer-cutover charter is not
closed for sequencing. The active objective is to close the smallest remaining behavioral-proof
gaps without mechanically injecting typed outcomes, turn context, archetype, or progress into
scorers where those layers are not semantically relevant.

Hard decisions:

- `semantic_goal_drift` is cutover complete by design. It consumes structured objectives, stable
  target anchors, sanctioned replans, delegation visibility, and checkpoint history. Do not reopen
  it without new failing evidence.
- `dead_end_thrash` now has focused `CTX-R6-04` proof at `0 / Low / Cleared`, unflagged, with empty
  evidence after production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh built-in
  `default` `REVIEW CLEAN`; `R6-GAP-DET-OPAQUE-PARENT` is complete. Its frozen four-case corpus
  remains posture invariance, not comparative integrated improvement, and no terminal scorer
  disposition is assigned before `R6-CLOSE`.
- `truth_grounding_gap` now passes the bounded truth-path action-before-read and cross-checkpoint
  provenance controls; `R6-GAP-TGG-TRUTH-PATH-ACTION` is complete after final proof-receipt series
  `fee9c2b16` + `6674a8316` received fresh independent built-in `default` `REVIEW CLEAN`. Its
  terminal scorer disposition remains reserved for `R6-CLOSE`.
- `wrong_plan_branch` passed its read-only, sanctioned-replan, and delegated-parent controls but
  preserved the empty-authority red `CTX-R6-15`; review-clean transition series `2937dbe5a` +
  `91f55f6bf` activated only `R6-GAP-WPB-EMPTY-AUTHORITY`, and packet-doc series `8734f4dbe` +
  `334e7c6ac` received fresh independent built-in `default` `REVIEW CLEAN`.
- Transitive data availability is not behavioral integration; non-applicable context is an explicit
  fit-for-purpose decision, not missing plumbing.
- The semantic acceptance corpus-shape test proves fixture integrity; the separate live
  analyzer-path test proves behavior. Dispatcher order is explicit in source but not covered by a
  focused behavioral order assertion.
- R7 remains **DRAFT / BLOCKED ON R6 CLOSURE DECISION** and must not absorb unresolved ordinary
  single-session scorer semantics.

`R6-C.0A` is complete at `d3dcda785`; `R6-C.1-SPEC` is complete through review-clean `ea19b39a7`;
and `R6-C.1-CONTROLS` is complete against the wall receipt `5618f7864`. The thirteen synthetic
controls resolved as `10 PASS / 3 preserved RED`, with no production change in that controls wall.
The first two named gaps are now complete with review-clean proof. Transition series `2937dbe5a` +
`91f55f6bf` received fresh independent built-in `default` `REVIEW CLEAN`, so only
`R6-GAP-WPB-EMPTY-AUTHORITY` is active with active packet
`R6-GAP-WPB-EMPTY-AUTHORITY`. Its canonical SPEC/PLAN/TASKS landed in packet-doc series
`8734f4dbe` + `334e7c6ac` and received fresh independent built-in `default` `REVIEW CLEAN`.
After the separate authority-only reconciliation is committed and fresh-review-clean, the sole next
execution action is exact `CTX-R6-15` witness reconfirmation. Replay closeout, R6 closure, successor
execution, and all R7/R8 work remain blocked; `R6-REPLAY` is explicitly `BLOCKED`.

R7 promotion requires the applicability audit to be complete, broad R6 acceptance claims
behaviorally proven or narrowed honestly, the R6 finding updated to `CLOSED`, and all root/R6/R7
authority documents reconciled. Each material scoring surface must end in exactly one terminal
category: **Cutover complete**, **Fit-for-purpose exception**,
**Merged/deprecated**, or **Explicitly deferred outside R6 with justification**. Ordinary “still
open” is not a closure disposition.
