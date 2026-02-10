# Kickoff: WS7-test (test)

## Scope
- Tests only (plus minimal test-only helpers if absolutely needed); no production code.
- Spec: `docs/project_management/next/world-sync/WS7-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-sync-ws7-test` on branch `world-sync-ws7-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-sync/plan.md`, `docs/project_management/next/world-sync/tasks.json`, `docs/project_management/next/world-sync/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world-sync" SLICE_ID="WS7"` (preferred; starts code+test in parallel; `WS7` is `WS7-test` without `-test`)
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world-sync" TASK_ID="WS7-test"` (single task only)

## Requirements
- Add/modify tests that enforce the spec’s acceptance criteria.
- If the spec implies large/sweeping behavior changes, stop and ask the operator to split the slice so the test task can stay focused and reviewable.
- If completing this task requires more than 108,800 tokens of context (40% of a 272k token window), stop and ask the operator to split the slice before proceeding.
- Run: `cargo fmt`, plus the targeted tests you add/touch.
  - Note: your branch is not expected to be fully green until the code branch lands; tests must compile and fail deterministically for spec-driven reasons.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WS7-test"`
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
