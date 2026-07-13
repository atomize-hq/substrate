# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-GAP-DET-OPAQUE-PARENT` (ACTIVE; active packet: none; packet-docs gate only)**

## Dependency Order

1. Preserve the 2026-07-12 authority correction and context-pack baseline.
2. **COMPLETE at `d3dcda785`:** docs-only `R6-C.0A` remediated all seven fresh closure-matrix review
   findings across the canonical finding and root/R6/R7 authority and received fresh review clean.
3. **COMPLETE at `ea19b39a7`:** the `R6-C.1 — Scorer Context Applicability Acceptance Controls`
   SPEC/PLAN/TASKS landed and received final fresh `REVIEW CLEAN`.
4. **COMPLETE at `5618f7864`:** the thirteen synthetic acceptance controls resolved as
   `10 PASS / 3 preserved RED`, with no production change.
5. **ACTIVE:** atomically create and freshly review only the three canonical
   `R6-GAP-DET-OPAQUE-PARENT` packet docs recorded as `TO CREATE` in the named-gap subledger.
6. Resolve the three preserved red routes sequentially in matrix order; do not batch gaps or begin
   production/no-code proof before the active route's docs gate is review-clean.
7. Re-run the focused scorer wall, full analyzer wall, and bounded replay evidence after all named
   gaps are complete.
8. Update the finding to `CLOSED` only when every material scoring surface has exactly one terminal
   disposition — **Cutover complete**, **Fit-for-purpose exception**, **Merged/deprecated**, or
   **Explicitly deferred outside R6 with justification** — and every broad acceptance claim is
   proven or narrowed honestly. Ordinary “still open” is not a closure disposition.
9. Only then promote the preserved R7 drafts to implementation-ready.

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

No production code change, witness rerun, or no-code proof receipt is authorized until the active
`R6-GAP-DET-OPAQUE-PARENT` packet-docs gate is committed and fresh-review-clean. The later named gaps
and `R6-REPLAY` remain blocked.
