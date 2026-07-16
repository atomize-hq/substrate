# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **CLOSED**

Current phase: **`R7-6` (SOLE ACTIVE PHASE — CHECKPOINT-DOC RECEIPT GATE; active packet: `none`;
operator decision `R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A`; R7-6.1 commit `7789fba4f`
fresh independent built-in `default` `CLEAN`; R7-6.2 series `d2842f279` + `77ae455fe` +
`a333d8486` fresh independent built-in `default` `CLEAN`; final-wall fix `bd743eacc` fresh
independent built-in `default` `CLEAN`; `CTX-R7-06` `PROVEN`; implementation and final checkpoint
complete; checkpoint-doc receipt committed/landed and awaiting fresh independent review; R7-6 exit gate and
R8-SPEC entry pending/blocked; no next Prompt 1 selectors eligible or invoked)**

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
13. **COMPLETE — R7-2:** R7-2.1 commit `c60d05f77`, R7-2.2 commit `9403c8a24`, and R7-2.3
    series `7af2ae517` + `75a353e46` are fresh independent built-in `default` `REVIEW CLEAN`; the
    fix resolved the summary-vs-checkpoint blocker. The tasks and behavior/static checkpoint are
    complete. Checkpoint-doc commit `78a168c09` received fresh independent built-in `default`
    `REVIEW CLEAN`, satisfying the R7-2 exit gate and proving `CTX-R7-03`.
14. **COMPLETE — R7-3:** entry-authority repair `9fd9d9972`, R7-3.1 test-only commit `f8dd04549`,
    R7-3.2 test-only commit `c7c6f35b8`, and checkpoint-doc commit `931c2701c` are fresh independent
    built-in `default` `REVIEW CLEAN`. R7-3.1, R7-3.2, and the behavior/static checkpoint are
    complete; the exit gate is satisfied and `CTX-R7-04` is proven.
15. **COMPLETE — R7-4:** R7-4.1 commit/fix series `ebcb052b9` + `e7b65523f`, R7-4.2 docs decision
    commit `8a0790a3d`, and checkpoint-doc commit
    `ca8467edda80f14b35f1a4d9a4c2192d43b217a2` are fresh independent built-in `default` review-
    clean receipts, with the checkpoint receipt recorded accurately as `CLEAN`. R7-4.1, R7-4.2,
    and the behavior/static checkpoint are complete; the R7-4 exit gate is satisfied.
16. **COMPLETE — R7-5:** R7-5.1 commit `afb10827d`, R7-5.2 commit `9c0690a02`, and checkpoint-doc
    commit `b4916e565cd48f0924fb720d633b57d31b0d624c` are fresh independent built-in `default`
    `CLEAN` receipts with no actionable findings. R7-5.1, R7-5.2, and the behavior checkpoint are
    complete; `CTX-R7-05` is proven; the R7-5 exit gate is satisfied.
17. **ACTIVE — R7-6 CHECKPOINT-DOC RECEIPT GATE:** active packet is `none`. R7-6.1 commit
    `7789fba4f`, R7-6.2 series `d2842f279` + `77ae455fe` + `a333d8486`, and final-wall fix
    `bd743eacc` received fresh independent built-in `default` `CLEAN`. R7-6.1, R7-6.2, and all final
    checkpoint items are complete; `CTX-R7-06` is `PROVEN`. The landed checkpoint-doc receipt
    awaits fresh independent review, so the R7-6 exit gate and R8-SPEC entry remain pending/blocked.
    No next Prompt 1 selectors are eligible or invoked.

## Execution Rules

- Keep the operator-decision boundaries intact: only v0.8 was added to the centralized helper;
  v0.8 uses explicit posture and state-backed evidence; v0.2 and v0.3-v0.7 behavior remain
  preserved; generalized parsing and R8 consolidation remain out of scope. Keep `CTX-R7-06`
  `PROVEN`, R7-6 active only for the checkpoint-doc receipt gate, R8-SPEC blocked, and no next
  Prompt 1 selectors eligible until the landed receipt is fresh-review-clean.
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
  static checkpoint complete. Keep checkpoint-doc commit `78a168c09` fresh independent `REVIEW
  CLEAN`, R7-2 complete, and `CTX-R7-03` proven. Keep entry-authority repair `9fd9d9972`, test-only
  commits `f8dd04549` and `c7c6f35b8`, and checkpoint-doc commit `931c2701c` fresh independent
  `REVIEW CLEAN`; keep R7-3 complete and `CTX-R7-04` proven. Keep R7-4.1 commit/fix series
  `ebcb052b9` + `e7b65523f`, R7-4.2 docs decision commit `8a0790a3d`, and checkpoint-doc commit
  `ca8467edda80f14b35f1a4d9a4c2192d43b217a2` fresh independent review-clean; keep R7-4 complete.
  Keep transition commit `1b746a2a` fresh independent built-in `default` `REVIEW CLEAN` with no
  findings; keep R7-5.1 commit `afb10827d` and R7-5.2 commit `9c0690a02` fresh independent `CLEAN`;
  keep both tasks and the behavior checkpoint complete and `CTX-R7-05` proven. Keep checkpoint-doc
  commit `b4916e565cd48f0924fb720d633b57d31b0d624c` fresh independent built-in `default` `CLEAN`
  with no findings, the R7-5 exit gate satisfied, and R7-5 complete. Keep R7-6.1 commit `7789fba4f`,
  R7-6.2 series `d2842f279` + `77ae455fe` + `a333d8486`, and final-wall fix `bd743eacc` fresh
  independent built-in `default` `CLEAN`; keep R7-6.1, R7-6.2, and all final checkpoint items
  complete and `CTX-R7-06` `PROVEN`. Keep R7-6 the sole active phase at the checkpoint-doc receipt
  gate, R8-SPEC blocked, and no next Prompt 1 selectors eligible or invoked.
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
satisfying the R7-4 exit gate. R7-4 is complete. R7-5.1 commit `afb10827d` received fresh independent built-in `default` `CLEAN` with no findings,
and R7-5.2 commit `9c0690a02` received fresh independent built-in `default` `CLEAN` with no
actionable findings. The focused acceptance targets pass `2 / 2`, `8 / 8`, and `4 / 4`; the report
records `115` checkpoints across required strata `23 / 18 / 0 / 23 / 51 / 0`, `3,126` valid
evidence references, zero malformed references, and zero cross-trajectory progress/scorer ownership
violations. At final proof HEAD `9c0690a02`, formatting, compactor/analyzer clippy with `-D warnings`,
full compactor `39 / 39` aggregate, full analyzer `424 / 424` aggregate, and `git diff --check` are
green. R7-5.1, R7-5.2, and the Checkpoint R7-5 behavior items are complete; `CTX-R7-05` is
`PROVEN`. Checkpoint-doc commit `b4916e565cd48f0924fb720d633b57d31b0d624c` received fresh
independent built-in `default` `CLEAN` with no findings, satisfying the R7-5 exit gate. R7-5 is complete. Operator decision
`R7-6-HIGH-IMPACT-SENTINEL-EXPLICIT-STATE-01: A` authorized only adding `v0.8` to the centralized
explicit-analyzer-state helper, proving explicit posture plus state-backed evidence, preserving
v0.2 and v0.3-v0.7 behavior, and forbidding generalized version parsing or R8 consolidation.
R7-6.1 commit `7789fba4f` received fresh independent built-in `default` `CLEAN`; live compatibility
passes `27 / 27`, replay input `16 / 16`, and operator surface `16 / 16`. R7-6.2 series
`d2842f279` + `77ae455fe` + `a333d8486` received fresh independent built-in `default` `CLEAN` after
cursor-regression and naming fixes; `real_session_live` passes `12 / 12` and `live_end_to_end`
passes `10 / 10`, proving verified direct closure, per-session cursors, fail-closed unexpected-
session behavior, and unchanged scheduling. Bounded final-wall fix `bd743eacc` received fresh
independent built-in `default` `CLEAN` for the `serde_json` workspace feature-unification test-order
witness. At code/proof HEAD `bd743eacc`, formatting, workspace clippy with `-D warnings`, full
compactor `39 / 39`, full analyzer `424 / 424`, full sentinel `105 / 105`, full workspace tests,
and `git diff --check` are green. Staged GitNexus gates stayed within the authorized HIGH helper and
otherwise MEDIUM/LOW; no additional HIGH/CRITICAL symbol was edited. R7-6.1, R7-6.2, and all final
checkpoint items are complete, and `CTX-R7-06` is `PROVEN`. R7-6 remains the sole active phase with
packet `none` while the committed/landed checkpoint-doc receipt awaits fresh independent review. The
implementation/final checkpoint is complete, but the R7-6 exit gate and R8-SPEC entry remain
pending/blocked until the landed receipt is fresh-review-clean. No next Prompt 1 selectors are
eligible or invoked.
