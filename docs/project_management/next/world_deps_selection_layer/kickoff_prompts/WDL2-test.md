# Kickoff: WDL2-test (test)

## Scope
- Tests only (plus minimal test-only helpers if absolutely needed); no production code.
- Spec: `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world_deps_selection_layer-wdl2-test` on branch `world_deps_selection_layer-wdl2-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" SLICE_ID="WDL2"` (preferred)
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" TASK_ID="WDL2-test"` (single task)

## Requirements
- Add/modify tests that enforce S2 acceptance criteria (package list computation and exit codes).
- Run:
  - `cargo fmt`
  - the targeted tests you add/touch

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDL2-test"`.
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

