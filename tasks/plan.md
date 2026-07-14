# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

Current phase: **`R6-GAP-WPB-EMPTY-AUTHORITY` (ACTIVE; active packet: `R6-GAP-WPB-EMPTY-AUTHORITY`; exact `CTX-R6-15` witness reconfirmation gate)**

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
7. **ACTIVE / PACKET DOCS REVIEW CLEAN:** transition series `2937dbe5a` + `91f55f6bf` and
   packet-doc series `8734f4dbe` + `334e7c6ac` each received fresh independent built-in `default`
   `REVIEW CLEAN`. Active packet is `R6-GAP-WPB-EMPTY-AUTHORITY`. After the separate
   authority-only reconciliation is committed and fresh-review-clean, reconfirm the exact
   `CTX-R6-15` witness before selecting the already-authorized production-fix or eligible no-code
   route. Do not begin either route first.
8. Re-run the focused scorer wall, full analyzer wall, and bounded replay evidence after all named
   gaps are complete.
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

Transition series `2937dbe5a` + `91f55f6bf` and packet-doc series `8734f4dbe` + `334e7c6ac` are
fresh independent built-in `default` `REVIEW CLEAN`. The active packet is
`R6-GAP-WPB-EMPTY-AUTHORITY`. Exact `CTX-R6-15` witness reconfirmation is the sole next execution
action after the current authority-only reconciliation is committed and fresh-review-clean. No
production change or no-code receipt is authorized first. `R6-REPLAY` remains blocked.
