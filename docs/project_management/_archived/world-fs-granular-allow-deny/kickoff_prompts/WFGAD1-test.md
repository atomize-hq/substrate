# Kickoff: WFGAD1-test (test)

## Scope
- Tests only (plus minimal test-only helpers if absolutely needed); no production code.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-wfgad1-test` on branch `world-fs-granular-allow-deny-wfgad1-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/_archived/world-fs-granular-allow-deny/plan.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/tasks.json`, `docs/project_management/_archived/world-fs-granular-allow-deny/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny" SLICE_ID="WFGAD1"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny" TASK_ID="WFGAD1-test"`

## Requirements
- Add/modify tests that enforce the spec’s acceptance criteria.
- Run: `cargo fmt`, plus the targeted tests you add/touch.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGAD1-test"`
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
