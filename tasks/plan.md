# Active Plan: R6 Scorer-Context Cutover Closure

Canonical authority:
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`

Status: **PARTIAL / CLOSURE AUDIT REQUIRED**

## Dependency Order

1. Preserve the 2026-07-12 closure audit and authority correction as the current truth.
2. Plan `R6-C.1 — Scorer Context Applicability Acceptance Controls` from the exact missing controls
   in the finding.
3. Add acceptance controls before production behavior changes.
4. If a control fails, split only that scorer/failure into a bounded R6 implementation packet.
5. Re-run the focused scorer wall, full analyzer wall, and bounded replay evidence.
6. Update the finding to `CLOSED` only when every scorer is complete, intentionally exempt, or
   resolved and every broad acceptance claim is proven or narrowed honestly.
7. Only then promote the preserved R7 drafts to implementation-ready.

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
