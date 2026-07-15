# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **CLOSED**

Current phase: **`R7-2` (SOLE ACTIVE PHASE; active packet: `none`; R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint complete; task commits `c60d05f77`, `9403c8a24`, and series `7af2ae517` + `75a353e46` fresh independent review-clean; current checkpoint-doc receipt pending fresh independent review; R7-2 exit gate and `CTX-R7-03` still open; `R7-3..R7-6` and R8 blocked; `R7-3.1` unchecked and unstarted; no next-phase selectors prepared or invoked)**

## Dependency Order

1. Preserve the 2026-07-12 authority correction and context-pack baseline.
2. **COMPLETE at `d3dcda785`:** docs-only `R6-C.0A` remediated all seven fresh closure-matrix review
   findings across the canonical finding and root/R6/R7 authority and received fresh review clean.
3. **COMPLETE at `ea19b39a7`:** the `R6-C.1 — Scorer Context Applicability Acceptance Controls`
   SPEC/PLAN/TASKS landed and received final fresh `REVIEW CLEAN`.
4. **COMPLETE at `5618f7864`:** the thirteen synthetic acceptance controls resolved as
   `10 PASS / 3 preserved RED`, with no production change.
5. **COMPLETE through fresh review-clean `d13f0a71c`:** the first named gap,
   `R6-GAP-DET-OPAQUE-PARENT`, landed its bounded scorer fix and focused/family/checkpoint proof.
6. **COMPLETE through final proof-receipt series `fee9c2b16` + `6674a8316`, fresh independent
   `REVIEW CLEAN`:** `R6-GAP-TGG-TRUTH-PATH-ACTION` landed its bounded Option-A fix and proof.
7. **COMPLETE through fresh review-clean `6b42e5476` + `e65df2561` + `cd4e24119`:** the final named
   gap, `R6-GAP-WPB-EMPTY-AUTHORITY`, landed its bounded scorer fix, focused/family/checkpoint/full
   proof, receipt corrections, and fresh independent `REVIEW CLEAN`.
8. **COMPLETE — R6-REPLAY:** authority transition series
   `56bb9966f` + `07a3b1fe5` is fresh independent built-in `default` `REVIEW CLEAN`. Replay completed
   `CTX-R6-01`, then bounded packet `R6-GAP-DET-REPLAY-STALL` closed `CTX-R6-02`. Historical witness
   `60cde3dd7` is preserved; implementation/proof commit `6eda87e60` passes exact `CTX-R6-02`, seven
   focused controls, `20 / 20` troubleshooting matches, exact sticky and frozen-corpus controls,
   progress acceptance, checkpoint matches, full analyzer `402 / 402`, compactor normalization, and
   static gates. A fresh independent built-in `default` reviewer returned `REVIEW CLEAN`. This
   authority-only transition series `1ff592823` + `7839a7f47` clears the active packet to `none`,
   keeps `R6-REPLAY` active, and is fresh independent built-in `default` `REVIEW CLEAN`.
   Phase-owned replay exact controls then passed `4 x 1 / 1`; the family filters passed
   `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169` in manifest order, full analyzer passed
   `402 / 402`, and diff check was green. Phase-owned proof/fix series `b1791c1e3` + `e6d43eee9` +
   `61c9d5074` received fresh independent built-in `default` `REVIEW CLEAN`; no ordinary replay gap
   remains.
9. **COMPLETE — R6-CLOSE / CTX-R6-17:** the 2026-07-15 exact replay controls pass `4 x 1 / 1`, the
   Manifest E filters pass `21 / 21`, `58 / 58`, `22 / 22`, `6 / 6`, and `169 / 169`, the full
   analyzer is green across all suites, and `git diff --check` passes. Terminal table:
   `dead_end_thrash` and `semantic_goal_drift` **Cutover complete**; `truth_grounding_gap`,
   `wrong_plan_branch`, and `scoring/mod.rs` **Fit-for-purpose exception**. Transition/review-fix
   series `13b14d5f1` + `50446e6d6` received fresh independent built-in `default` `REVIEW CLEAN`
   with no actionable findings after `50446e6d6` resolved the first review's two P1 findings and one
   P3 finding.
10. **COMPLETE — R7-PROMOTE:** promotion series `455d0ed90` + `876ac55de` reconciled the R7
    authority family to implementation-ready content and received fresh independent built-in
    `default` `REVIEW CLEAN`.
11. **COMPLETE — R7-0:** transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` and `R7-0.1`
    series `a9e75f149` + `55bea5fa5` + `faff68ac6` are fresh independent built-in `default`
    `REVIEW CLEAN`. Fixture-only `R7-0.2` commit `fa85cd4b8` also received fresh independent
    built-in `default` `REVIEW CLEAN`; focused `2 / 2`, compactor `25 / 25` including end-to-end
    `2 / 2`, and privacy scans over `24` rows are green with zero markers or raw UUIDs.
12. **COMPLETE — R7-1:** transition/fix series `339744dff`
    + `d20cac6a9` is fresh independent built-in `default` `REVIEW CLEAN`; active packet is `none`.
    R7-1.1 series `e65127720` + `685cf843b`, R7-1.2 commit `4d122cd9f`, and R7-1.3 commit
    `e865eee13` are fresh independent built-in `default` `REVIEW CLEAN`. Focused `6 / 6`, end-to-end
    `6 / 6`, CLI `2 / 2`, full compactor `36` unit/integration plus `3` doctests, deterministic
    ordering, and static/diff/GitNexus gates are green. Checkpoint-doc commit `1cae7d693` is fresh
    independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate.
13. **ACTIVE — R7-2 CHECKPOINT-DOC RECEIPT GATE:** active packet is `none`. R7-2.1 commit
    `c60d05f77`, R7-2.2 commit `9403c8a24`, and R7-2.3 series `7af2ae517` + `75a353e46` are fresh
    independent built-in `default` `REVIEW CLEAN`; the fix resolved the summary-vs-checkpoint
    blocker. The tasks and behavior/static checkpoint are complete. This checkpoint-doc receipt is
    pending fresh independent review, so R7-2 and `CTX-R7-03` remain open. `R7-3..R7-6` plus R8
    remain blocked; `R7-3.1` is unchecked and unstarted; no next-phase selectors are prepared or
    invoked.

## Execution Rules

- Obtain fresh independent review for only this R7-2 checkpoint-doc receipt before any phase transition. Do
  not complete `CTX-R7-03`, start R7-3 work, or prepare/invoke R7-3 selectors until that receipt is
  fresh-review-clean.
- Do not reopen `semantic_goal_drift` without a new failing witness.
- Do not add a common mega-context argument to every scorer.
- Treat `dead_end_thrash` regression/delegation/turn-shape; truth-grounding applicability,
  truth-path-touching action-before-read, and actionful-planning/research; and wrong-branch
  replan/delegation plus empty-authority path-bearing action controls as acceptance questions first.
- Treat the frozen dead-end corpus as posture invariance rather than comparative replay
  improvement. Keep broad replay honesty partially / bounded proven until integrated replay closes
  the claim or the wording is narrowed.
- Keep R7 reciprocal direct linkage, separate trajectories, direct-child-first support, and no-new-
  drift-class-by-default as implementation-ready authority decisions. `R7-0` is complete after
  `R7-0.1` and fixture-only `R7-0.2` commit `fa85cd4b8` each received fresh independent `REVIEW
  CLEAN`. Transition/fix series `339744dff` + `d20cac6a9` is fresh independent `REVIEW CLEAN`.
  R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh independent
  `REVIEW CLEAN`, and the R7-1 checkpoint is complete. Checkpoint-doc commit `1cae7d693` is fresh
  independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. Keep transition/fix
  series `6a8797c15` + `4ee469014` fresh independent built-in `default` `REVIEW CLEAN`; keep R7-1
  complete and `CTX-R7-02` proven. Keep R7-2 task commits `c60d05f77` and `9403c8a24` plus series
  `7af2ae517` + `75a353e46` fresh independent `REVIEW CLEAN`; keep R7-2.1/.2/.3 and the behavior/
  static checkpoint complete. Keep only R7-2 active with packet `none`; keep this checkpoint-doc
  receipt pending fresh independent review and `CTX-R7-03` open; keep `R7-3..R7-6` plus R8 blocked,
  `R7-3.1` unchecked/unstarted, and next-phase selectors neither prepared nor invoked.
- Run GitNexus impact analysis before any later symbol edit. Before every commit, stage only intended
  files with `git add -- <intended-files-only>`, run
  `npx gitnexus detect-changes --scope staged -r 97a0-substrate`, run
  `git diff --cached --check`, and inspect the complete `git diff --cached` before committing.
- Preserve unrelated worktree changes.

All three named gaps are complete. Final implementation/review-fix series `6b42e5476` +
`e65df2561` + `cd4e24119` is fresh independent built-in `default` `REVIEW CLEAN`. Authority
transition series `56bb9966f` + `07a3b1fe5` also received fresh independent built-in `default`
`REVIEW CLEAN` and activated only `R6-REPLAY`. Commit `6eda87e60` now completes `CTX-R6-02` and
`R6-GAP-DET-REPLAY-STALL` with the full ordered packet proof, full analyzer `402 / 402`, static gates,
and fresh independent built-in `default` `REVIEW CLEAN`. Packet transition series `1ff592823` +
`7839a7f47` clears active packet to `none`, keeps `R6-REPLAY` active, and is fresh independent
built-in `default` `REVIEW CLEAN`. Phase-owned exact `CTX-R6-01`, exact `CTX-R6-02`, renamed sticky,
and exact frozen-corpus controls each pass `1 / 1`. The manifest family wall passes
`dead_end_thrash 21 / 21`, `semantic_goal_drift 58 / 58`, `truth_grounding_gap 22 / 22`,
`wrong_plan_branch 6 / 6`, `checkpoints 169 / 169`, full analyzer `402 / 402`, and diff check. Sticky
authority remains `HistoricalOnly / 20`, unflagged; old `Recovered / 20` remains historical baseline
only. Phase-owned proof/fix series `b1791c1e3` + `e6d43eee9` + `61c9d5074` is fresh independent
built-in `default` `REVIEW CLEAN`, so `R6-REPLAY` is complete and no ordinary replay gap remains.
The 2026-07-15 `CTX-R6-17` receipt reconfirms the four exact replay controls, the five Manifest E
family filters, full analyzer all-suites green, and `git diff --check`. Transition/review-fix series
`13b14d5f1` + `50446e6d6` is fresh independent built-in `default` `REVIEW CLEAN` with no actionable
findings. R6 is `CLOSED` and `R6-CLOSE` is complete. Promotion series `455d0ed90` + `876ac55de`
completed the R7 content/gate audit, made the R7 authority family implementation-ready, and received
fresh independent built-in `default` `REVIEW CLEAN`; `R7-PROMOTE` is complete. The narrow transition
series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in `default`
`REVIEW CLEAN`. `R7-0.1` series `a9e75f149` + `55bea5fa5` + `faff68ac6` and fixture-only `R7-0.2`
commit `fa85cd4b8` are fresh independent built-in `default` `REVIEW CLEAN`, completing `R7-0`.
Transition/fix series `339744dff` + `d20cac6a9` is fresh independent built-in `default` `REVIEW
CLEAN`. R7-1 task series `e65127720` + `685cf843b`, `4d122cd9f`, and `e865eee13` are fresh
independent built-in `default` `REVIEW CLEAN`; the R7-1 checkpoint is complete. Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` +
`75a353e46`, received fresh independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the
summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are
complete. R7-2 remains the sole active phase with packet `none` while this checkpoint-doc receipt
still requires fresh independent review. The R7-2 exit gate and `CTX-R7-03`
remain open until the receipt itself is fresh-review-clean. `R7-3..R7-6` and R8 remain
blocked; `R7-3.1` is unchecked and unstarted; no next-phase selectors are prepared or invoked.
