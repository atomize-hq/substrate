# Active Spec: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **CLOSED**

Current phase: **`R7-5` (SOLE ACTIVE PHASE AT ENTRY ONLY; active packet: `none`; R7-4
complete at checkpoint-doc commit `ca8467edda80f14b35f1a4d9a4c2192d43b217a2`, fresh independent
built-in `default` `CLEAN`; current R7-4 -> R7-5 transition candidate pending fresh independent
review and not yet review-clean; `R7-5.1` next, unchecked, and unstarted; no R7-5 acceptance or real-
corpus work started; `CTX-R7-05` blocked/pending until R7-5 acceptance evidence proves existing
classes plus typed delegation context; R7-6 and R8 blocked; Prompt 1 selectors `PHASE_ID: R7-5` /
`ACTIVE_PACKET: none` prepared and eligible but not invoked)**

The scoped R6 packets, acceptance controls, named gaps, replay closeout, and terminal-disposition
reconciliation are complete. R6 is closed for sequencing. Promotion series `455d0ed90` +
`876ac55de` reconciled the R7 authority family to implementation-ready content and received fresh
independent built-in `default` `REVIEW CLEAN`, completing `R7-PROMOTE`. The narrow status transition
series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in `default`
`REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` + `faff68ac6` and fixture-only
`R7-0.2` commit `fa85cd4b8` each received fresh independent built-in `default` `REVIEW CLEAN`, so
`R7-0` is complete. Transition/fix series `339744dff` + `d20cac6a9` received fresh independent
built-in `default` `REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and
`e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`. The R7-1 checkpoint is complete
with focused `6 / 6`, end-to-end `6 / 6`, CLI `2 / 2`, full compactor `36` unit/integration plus `3`
doctests, deterministic ordering, and green formatting/clippy/diff/GitNexus gates. No raw private
rollout data was added. Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943` remains fresh independent built-in
`default` `REVIEW CLEAN`. R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs
decision commit `8a0790a3d` each received fresh independent built-in `default` `REVIEW CLEAN`;
R7-4.1, R7-4.2, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. Only R7-5 is active at entry with packet `none`;
the current R7-4 -> R7-5 transition candidate is pending fresh independent review and is not yet
review-clean. `R7-5.1` is next, unchecked, and unstarted; no R7-5 acceptance or real-corpus work has
started. `CTX-R7-05` remains blocked/pending until R7-5 acceptance evidence proves existing classes
plus typed delegation context. R7-6 and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-5` /
`ACTIVE_PACKET: none` are prepared and eligible but have not been invoked.

R7-4 checkpoint proof at implementation/docs HEAD `8a0790a3d`: `dead_end_thrash` passes `19 / 19`;
the `semantic_goal_drift` filter passes `58 / 58` aggregate (`56` library plus `2` acceptance);
full analyzer passes `422 / 422`; formatting, analyzer clippy with `-D warnings`, and diff checks are
green. Existing `DriftClass` values plus typed delegation context are sufficient for the proven
R7-4 evidence; no new variant was added. The known workspace-clippy RED remains routed to R7-6.1
and was neither rerun nor fixed.

Operator decision `R7-2-HIGH-IMPACT-ANALYZER-CONTRACT-01: A` authorized the bounded high-impact
analyzer seam. At implementation HEAD `75a353e46`, input passes `16 / 16`; delegation matches pass
`39` total (`25` library + `4` checkpoint + `8` delegation-context + `2` export); checkpoint matches
pass `172` total (`36` library + `134` checkpoints + `1` export + `1` truth-grounding); full
analyzer passes `417 / 417`; formatting, analyzer clippy `-D warnings`, and diff checks are green.
Staged GitNexus gates were R7-2.1 LOW / `0` affected processes, R7-2.2 MEDIUM / `1`, R7-2.3 HIGH /
`9` within the authorized decision, and fix MEDIUM / `2`. Public v0.8 with readable v0.7,
graph-derived roles and ids, `Linked`/`Partial` visibility, fail-closed conflicts, deterministic
`RowRef` evidence, JSON-summary parity, and separate trajectories are proven. No R7-3, R7-4,
sentinel, or R8 work leaked in.

Hard decisions:

- `semantic_goal_drift` is cutover complete by design. It consumes structured objectives, stable
  target anchors, sanctioned replans, delegation visibility, and checkpoint history. Do not reopen
  it without new failing evidence.
- `dead_end_thrash` now has focused `CTX-R6-04` proof at `0 / Low / Cleared`, unflagged, with empty
  evidence after production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh built-in
  `default` `REVIEW CLEAN`; `R6-GAP-DET-OPAQUE-PARENT` is complete. Its frozen four-case corpus
  remains posture invariance, not comparative integrated improvement; its terminal disposition is
  **Cutover complete**.
- `truth_grounding_gap` now passes the bounded truth-path action-before-read and cross-checkpoint
  provenance controls; `R6-GAP-TGG-TRUTH-PATH-ACTION` is complete after final proof-receipt series
  `fee9c2b16` + `6674a8316` received fresh independent built-in `default` `REVIEW CLEAN`. Its
  terminal scorer disposition is **Fit-for-purpose exception**.
- `wrong_plan_branch` passed its read-only, sanctioned-replan, delegated-parent, and empty-authority
  controls. Historical `CTX-R6-15` remains preserved at `59f098b35`; implementation/review-fix series
  `6b42e5476` + `e65df2561` + `cd4e24119` received fresh independent built-in `default` `REVIEW
  CLEAN` with exact target `0 / Low / Cleared`, unflagged, empty evidence. Its terminal scorer
  disposition is **Fit-for-purpose exception**.
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
- `scoring/mod.rs` is dispatcher/routing infrastructure rather than a fifth scorer; its terminal
  disposition is **Fit-for-purpose exception**.
- The R7 authority family is implementation-ready after completed promotion series `455d0ed90` +
  `876ac55de` received fresh independent built-in `default` `REVIEW CLEAN`. `R7-PROMOTE` is complete.
  Transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in
  `default` `REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` + `faff68ac6` and fixture-only
  `R7-0.2` commit `fa85cd4b8` are fresh independent built-in `default` `REVIEW CLEAN`, completing
  `R7-0`. Transition/fix series `339744dff` + `d20cac6a9` is fresh independent built-in `default`
  `REVIEW CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh
  independent built-in `default` `REVIEW CLEAN`; the R7-1 implementation/checkpoint is complete.
  Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943` remains fresh independent built-in
`default` `REVIEW CLEAN`. R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs
decision commit `8a0790a3d` each received fresh independent built-in `default` `REVIEW CLEAN`;
R7-4.1, R7-4.2, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. Only R7-5 is active at entry with packet `none`;
the current R7-4 -> R7-5 transition candidate is pending fresh independent review and is not yet
review-clean. `R7-5.1` is next, unchecked, and unstarted; no R7-5 acceptance or real-corpus work has
started. `CTX-R7-05` remains blocked/pending until R7-5 acceptance evidence proves existing classes
plus typed delegation context. R7-6 and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-5` /
`ACTIVE_PACKET: none` are prepared and eligible but have not been invoked.

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
`402 / 402`; `git diff --check` is green. Phase-owned proof/fix series `b1791c1e3` + `e6d43eee9` +
`61c9d5074` received fresh independent built-in `default` `REVIEW CLEAN`. Current sticky authority
remains `HistoricalOnly / 20`, unflagged, with old `Recovered / 20` retained only as historical
baseline. On 2026-07-15, `CTX-R6-17` reconfirmed the four exact replay controls at `1 / 1` each, the
Manifest E filters at `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`, the full analyzer
with all suites green, and `git diff --check`. It assigns `dead_end_thrash` and
`semantic_goal_drift` **Cutover complete** plus `truth_grounding_gap`, `wrong_plan_branch`, and
`scoring/mod.rs` **Fit-for-purpose exception**. No merge/deprecation or deferral route is used.
The R6 finding is `CLOSED` and `R6-CLOSE` is complete. Promotion series `455d0ed90` + `876ac55de`
completed the R7 content/gate audit, made the R7 authority family implementation-ready, and received
fresh independent built-in `default` `REVIEW CLEAN`; `R7-PROMOTE` is complete. The narrow transition
series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in `default`
`REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` + `faff68ac6` and fixture-only `R7-0.2`
commit `fa85cd4b8` are fresh independent built-in `default` `REVIEW CLEAN`, completing `R7-0`.
Transition/fix series `339744dff` + `d20cac6a9` is fresh independent built-in `default` `REVIEW
CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh
independent built-in `default` `REVIEW CLEAN`; the R7-1 implementation/checkpoint is complete.
Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943` remains fresh independent built-in
`default` `REVIEW CLEAN`. R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs
decision commit `8a0790a3d` each received fresh independent built-in `default` `REVIEW CLEAN`;
R7-4.1, R7-4.2, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`ca8467edda80f14b35f1a4d9a4c2192d43b217a2` received fresh independent built-in `default` `CLEAN`,
satisfying the R7-4 exit gate. R7-4 is complete. Only R7-5 is active at entry with packet `none`;
the current R7-4 -> R7-5 transition candidate is pending fresh independent review and is not yet
review-clean. `R7-5.1` is next, unchecked, and unstarted; no R7-5 acceptance or real-corpus work has
started. `CTX-R7-05` remains blocked/pending until R7-5 acceptance evidence proves existing classes
plus typed delegation context. R7-6 and R8 remain blocked. Prompt 1 selectors `PHASE_ID: R7-5` /
`ACTIVE_PACKET: none` are prepared and eligible but have not been invoked.
