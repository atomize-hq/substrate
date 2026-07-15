# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **CLOSED**

Current phase: **`R7-PROMOTE` (ACTIVE AT ENTRY ONLY; active packet: `none`; R6 is `CLOSED`; `R6-CLOSE` and `CTX-R6-17` are complete; promotion entry gate satisfied; R7 drafts not yet implementation-ready; no R7 task started)**

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
10. **ACTIVE AT ENTRY ONLY — R7-PROMOTE:** reconcile the preserved R7 drafts to
    implementation-ready only inside the separate promotion phase. No promotion or implementation
    task is started by the R6 closeout.

## Execution Rules

- Do not begin R7 implementation.
- Do not reopen `semantic_goal_drift` without a new failing witness.
- Do not add a common mega-context argument to every scorer.
- Treat `dead_end_thrash` regression/delegation/turn-shape; truth-grounding applicability,
  truth-path-touching action-before-read, and actionful-planning/research; and wrong-branch
  replan/delegation plus empty-authority path-bearing action controls as acceptance questions first.
- Treat the frozen dead-end corpus as posture invariance rather than comparative replay
  improvement. Keep broad replay honesty partially / bounded proven until integrated replay closes
  the claim or the wording is narrowed.
- Keep R7 reciprocal direct linkage, separate trajectories, direct-child-first support, and no-new-
  drift-class-by-default as draft design decisions, not current implementation authority.
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
findings. R6 is `CLOSED` and `R6-CLOSE` is complete. `R7-PROMOTE` is active at entry only with packet
`none`; the preserved R7 drafts remain not implementation-ready, and no R7 promotion or
implementation task has started.
