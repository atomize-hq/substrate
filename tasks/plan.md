# Active Plan: R7 Bounded Delegated-Session Support

Canonical authority:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-plan.md`

Status: PLANNED / NOT IMPLEMENTED

## Dependency Order

1. `R7-0`: docs lock and sanitized linkage fixtures.
2. `R7-1`: compactor reciprocal link extraction and explicit direct-child closure.
3. `R7-2`: analyzer link graph and checkpoint v0.8 public delegation contract.
4. `R7-3`: separate parent-visible and child-visible progress trajectories.
5. `R7-4`: trajectory-local scorer guardrails, with no new drift class by default.
6. `R7-5`: delegated acceptance matrix and real-session validation.
7. `R7-6`: minimal replay/live sentinel compatibility; R8 remains separate.

## Execution Rules

- Follow the canonical task ledger one bounded task at a time.
- Run GitNexus upstream impact analysis before editing every implementation symbol.
- Run the task-local verification wall before advancing.
- Keep parent and child checkpoints separate; never flatten or copy child progress into the parent.
- Run `npx gitnexus detect-changes -r 97a0-substrate` before every commit.
- Preserve unrelated worktree changes.

Read the canonical plan for architecture decisions, packet checkpoints, risks, parallelization, and
the final verification wall.
