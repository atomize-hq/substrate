# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Execution context router:
`docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

## Dependency Order

1. Preserve the 2026-07-12 authority correction and context-pack baseline.
2. Complete docs-only `R6-C.0A`: remediate every fresh closure-matrix review finding across the
   canonical finding and root/R6/R7 authority.
3. Plan `R6-C.1 — Scorer Context Applicability Acceptance Controls` from the corrected inventory.
4. Add acceptance controls before production behavior changes.
5. If a control fails, split only that scorer/failure into a bounded R6 implementation packet.
6. Re-run the focused scorer wall, full analyzer wall, and bounded replay evidence.
7. Update the finding to `CLOSED` only when every scorer is complete, intentionally exempt, or
   resolved and every broad acceptance claim is proven or narrowed honestly.
8. Only then promote the preserved R7 drafts to implementation-ready.

## Execution Rules

- Do not begin R7 implementation.
- Do not reopen `semantic_goal_drift` without a new failing witness.
- Do not add a common mega-context argument to every scorer.
- Treat `dead_end_thrash` regression/delegation/turn-shape, truth-grounding applicability, and
  wrong-branch replan/delegation controls as acceptance questions first.
- Keep R7 reciprocal direct linkage, separate trajectories, direct-child-first support, and no-new-
  drift-class-by-default as draft design decisions, not current implementation authority.
- Run GitNexus impact analysis before any later symbol edit and `npx gitnexus detect-changes -r
  97a0-substrate` before every commit.
- Preserve unrelated worktree changes.

No production code changes are authorized by this plan until a failing `R6-C.1` control proves a
bounded gap.
