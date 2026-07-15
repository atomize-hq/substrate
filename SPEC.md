# Active Spec: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-REPLAY` (ACTIVE / PROOF COMPLETE / PROOF-RECEIPT REVIEW PENDING; active packet: `none`; packet transition series `1ff592823` + `7839a7f47` fresh independent `REVIEW CLEAN`; `CTX-R6-01`/`02`/`06` and the R6 family wall green; narrow `R6-CLOSE` transition next only after this proof receipt is fresh-review-clean)**

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
  and packet proof wall; receipt `d788f45c9` is fresh independent `REVIEW CLEAN`. Task `.2`
  reconfirmed the exact pre-edit red. The operator then accepted Option A for Task `.2A`; the
  docs-first amendment series `d631e0c56` + `6498c343f` received fresh independent `REVIEW CLEAN`.
  That amendment proves truthful pairing changes the sticky rollout from checkpoint `9`
  `Regressing / Active 40` then checkpoint `10` `Recovered 20` to checkpoint `9`
  `Advancing / HistoricalOnly 20` then checkpoint `10` `HistoricalOnly 20`. Event `831 -> 837` and
  `recovery_state` are non-causal; canonical `Recovered` requires an immediately previous same-class
  `Active` score. The operator replied exactly
  `DECISION R6-REPLAY-STALL-POST-PAIRING-RECOVERED-SEMANTICS-02: A`. Task `.2B` is complete:
  sticky `CTX-R6-06` authority is now `HistoricalOnly / 20`, unflagged, while `Recovered / 20`
  remains historical clean-baseline evidence only. Commit `6eda87e60` landed the selected bounded
  implementation and complete ordered proof; exact `CTX-R6-02` is green at its locked
  `Stalled / Active` contract, full analyzer proof is `402 / 402`, and a fresh independent built-in
  `default` reviewer returned `REVIEW CLEAN`.
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
`default` `REVIEW CLEAN`. Replay has since completed `CTX-R6-01` and `CTX-R6-02`. Historical witness
`60cde3dd7` remains the preserved behavioral-red receipt; bounded implementation/proof commit
`6eda87e60` passes exact `CTX-R6-02`, all ordered packet proof, full analyzer `402 / 402`, and static
gates, and received fresh independent built-in `default` `REVIEW CLEAN`. Packet transition series
`1ff592823` + `7839a7f47` marks `R6-GAP-DET-REPLAY-STALL` complete, clears the active packet to
`none`, and received fresh independent built-in `default` `REVIEW CLEAN`. Phase-owned replay then
passed exact `CTX-R6-01`, exact `CTX-R6-02`, the renamed sticky control, and exact `CTX-R6-06` at
`1 / 1` each. The R6 family wall passed `dead_end_thrash 21 / 21`, `semantic_goal_drift 58 / 58`,
`truth_grounding_gap 22 / 22`, `wrong_plan_branch 6 / 6`, `checkpoints 169 / 169`, and full analyzer
`402 / 402`; `git diff --check` is green. Current sticky authority remains `HistoricalOnly / 20`,
unflagged, with old `Recovered / 20` retained only as historical baseline. `R6-REPLAY` remains
active while this proof receipt awaits fresh independent review. After review-clean, a narrow
transition may activate `R6-CLOSE`; no terminal disposition, successor execution, or R7/R8 work is
authorized by this receipt.

R7 promotion requires the applicability audit to be complete, broad R6 acceptance claims
behaviorally proven or narrowed honestly, the R6 finding updated to `CLOSED`, and all root/R6/R7
authority documents reconciled. Each material scoring surface must end in exactly one terminal
category: **Cutover complete**, **Fit-for-purpose exception**,
**Merged/deprecated**, or **Explicitly deferred outside R6 with justification**. Ordinary “still
open” is not a closure disposition.
