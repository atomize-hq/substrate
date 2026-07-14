# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-REPLAY` (ACTIVE; active packet: `R6-GAP-DET-REPLAY-STALL`; trusted `CTX-R6-02` behavioral-RED witness `60cde3dd7`; Task `.0` docs-gate review-fix series pending fresh independent review, not review-clean)**

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
8. **ACTIVE / PACKET DOCS-GATE REVIEW-FIX PENDING FRESH REVIEW:** authority transition series
   `56bb9966f` + `07a3b1fe5` remains fresh independent built-in `default` `REVIEW CLEAN`. Replay then
   completed `CTX-R6-01` and preserved trusted `CTX-R6-02` behavioral RED at `60cde3dd7`. Active packet
   `R6-GAP-DET-REPLAY-STALL` has landed packet docs/annotation `200725001` and first authority
   correction `08fa86e94`; this final correction completes the review-fix candidate, but Task `.0`
   awaits a fresh clean verdict. Current action is fresh series review/fix, not Prompt 2 or Rust.
9. Update the finding to `CLOSED` only when every material scoring surface has exactly one terminal
   disposition — **Cutover complete**, **Fit-for-purpose exception**, **Merged/deprecated**, or
   **Explicitly deferred outside R6 with justification** — and every broad acceptance claim is
   proven or narrowed honestly. Ordinary “still open” is not a closure disposition.
10. Only then promote the preserved R7 drafts to implementation-ready.

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
`REVIEW CLEAN` and activated only `R6-REPLAY`. Current active packet is
`R6-GAP-DET-REPLAY-STALL`; packet docs/annotation `200725001` and first authority correction
`08fa86e94` are landed, while this final correction and the full Task `.0` series await fresh review.
No Rust edit, terminal scorer disposition, R6 close, or R7/R8 work is authorized yet.
