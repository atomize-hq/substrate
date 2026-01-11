# Kickoff: WDL0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world_deps_selection_layer-wdl0-code` on branch `world_deps_selection_layer-wdl0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, the spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" SLICE_ID="WDL0"` (preferred)
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" TASK_ID="WDL0-code"` (single task)

## Requirements
- Implement exactly the behaviors and error handling in S0 (selection gating, init/select, precedence, `--all`, exit codes).
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Baseline testing (required): run a targeted baseline test set before changes, then re-run the same set after changes and preserve or improve the failure set.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDL0-code"`.
3. Hand off the baseline test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
