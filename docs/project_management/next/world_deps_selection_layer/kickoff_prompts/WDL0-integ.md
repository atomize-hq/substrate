# Kickoff: WDL0-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable cross-platform green state.
- Spec: `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task merges back to the orchestration branch after all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world_deps_selection_layer-wdl0-integ` on branch `world_deps_selection_layer-wdl0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" TASK_ID="WDL0-integ"`

## Requirements
- Merge `WDL0-integ-core` and any platform-fix branches for WDL0 that produced commits.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Re-run behavioral smoke via CI from this worktree’s `HEAD`:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- Complete the slice closeout gate report:
  - `docs/project_management/next/world_deps_selection_layer/WDL0-closeout_report.md`

## End Checklist
1. Ensure all required platforms are green (include run ids/URLs).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDL0-integ"`.
3. Hand off run ids/URLs and closeout report completion to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
