# Kickoff: WDL2-integ-linux (integration platform-fix — linux)

## Scope
- Ensure the slice is green for linux (behavior platform; smoke required).
- Spec: `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task must not merge back to the orchestration branch; the final aggregator integration task performs the merge once all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: linux.
2. Verify you are in the task worktree `wt/world_deps_selection_layer-wdl2-integ-linux` on branch `world_deps_selection_layer-wdl2-integ-linux` and that `.taskmeta.json` exists at the worktree root.
3. Read: `docs/project_management/next/world_deps_selection_layer/plan.md`, `docs/project_management/next/world_deps_selection_layer/tasks.json`, `docs/project_management/next/world_deps_selection_layer/session_log.md`, the spec, and this prompt.

## Requirements
- Merge the slice’s core integration branch (`WDL2-integ-core`) into this worktree before validating smoke or making fixes.
- Run local quality gates: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Dispatch platform smoke via CI until green:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Ensure smoke is green and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDL2-integ-linux"`.
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

