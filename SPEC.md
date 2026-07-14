# Active Spec: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-REPLAY` (ACTIVE; active packet: `R6-GAP-DET-REPLAY-STALL`; trusted `CTX-R6-02` behavioral-RED witness `60cde3dd7`; Task `.0` series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` fresh independent `REVIEW CLEAN`; Task `.1` Option A accepted; Task `.2` next)**

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
- `wrong_plan_branch` passed its read-only, sanctioned-replan, delegated-parent, and empty-authority
  controls. Historical `CTX-R6-15` remains preserved at `59f098b35`; implementation/review-fix series
  `6b42e5476` + `e65df2561` + `cd4e24119` received fresh independent built-in `default` `REVIEW
  CLEAN` with exact target `0 / Low / Cleared`, unflagged, empty evidence. Its terminal scorer
  disposition remains reserved for `R6-CLOSE`.
- Transitive data availability is not behavioral integration; non-applicable context is an explicit
  fit-for-purpose decision, not missing plumbing.
- The semantic acceptance corpus-shape test proves fixture integrity; the separate live
  analyzer-path test proves behavior. Dispatcher order is explicit in source but not covered by a
  focused behavioral order assertion.
- Trusted depth-1 built-in `default` subagent rollout
  `019eb311-c7ce-7f50-ae13-b51a5b5461c3` satisfies the selected-checkpoint `CTX-R6-02` input
  contract. Witness `60cde3dd7` preserves the behavioral red; Task `.0` series `200725001` +
  `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW
  CLEAN`. On 2026-07-14 the operator explicitly replied
  `DECISION R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE: A`, accepting the locked `attempt.rs`-only fix
  and packet proof wall. Task `.1` is complete and Task `.2` is next.
- R7 remains **DRAFT / BLOCKED ON R6 CLOSURE DECISION** and must not absorb unresolved ordinary
  single-session scorer semantics.

`R6-C.0A` is complete at `d3dcda785`; `R6-C.1-SPEC` is complete through review-clean `ea19b39a7`;
and `R6-C.1-CONTROLS` is complete against the wall receipt `5618f7864`. The thirteen synthetic
controls resolved as `10 PASS / 3 preserved RED`, with no production change in that controls wall.
All three named gaps are now complete with review-clean focused proof. The final
`R6-GAP-WPB-EMPTY-AUTHORITY` implementation/review-fix series `6b42e5476` + `e65df2561` +
`cd4e24119` received fresh independent built-in `default` `REVIEW CLEAN`. The authority transition
in this review series is landed, marks the prior aggregate `R6-GAP-*` set complete, and activates
`R6-REPLAY`. Transition series `56bb9966f` + `07a3b1fe5` received fresh independent built-in
`default` `REVIEW CLEAN`. Replay has since completed `CTX-R6-01` and preserved trusted `CTX-R6-02`
behavioral RED at `60cde3dd7`; `R6-GAP-DET-REPLAY-STALL` is the active packet. Task `.0` docs-gate/review-fix series `200725001` + `08fa86e94` + `d03f5a355` + `9edf564d3` received fresh independent built-in `default` `REVIEW CLEAN`. The operator accepted Option A for `R6-REPLAY-STALL-HIGH-IMPACT-ACCEPTANCE` on 2026-07-14; Task `.1` is complete, Tasks `.2`-`.4` are authorized in order, and Task `.2` is next. `CTX-R6-06`, R6 closure, terminal
dispositions, successor execution, and all R7/R8 work remain blocked.

R7 promotion requires the applicability audit to be complete, broad R6 acceptance claims
behaviorally proven or narrowed honestly, the R6 finding updated to `CLOSED`, and all root/R6/R7
authority documents reconciled. Each material scoring surface must end in exactly one terminal
category: **Cutover complete**, **Fit-for-purpose exception**,
**Merged/deprecated**, or **Explicitly deferred outside R6 with justification**. Ordinary “still
open” is not a closure disposition.
