# Kickoff: WDAP1-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable merged state.
- Spec: `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task merges back to the orchestration branch after all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-apt-provisioning-wdap1-integ` on branch `world-deps-apt-provisioning-wdap1-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`, `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP1-integ"`

## Requirements
- Merge the relevant integration branches for this slice:
  - core integration branch: `WDAP1-integ-core`
  - platform-fix branches: `WDAP1-integ-linux`, `WDAP1-integ-macos`, `WDAP1-integ-windows` (when they produced commits)
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### CI checkpoints (required)

For this cross-platform automation pack, cross-platform CI gates run only at checkpoint boundaries defined in:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md`

Rules:
- Do not dispatch cross-platform CI from this integration-final task.
- Verify `CP2-ci-checkpoint` is completed and that run ids/URLs are recorded in `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`.

## End Checklist
1. Ensure merged state is committed and local integration gates are green:
   - From inside the worktree, run: `make triad-task-finish TASK_ID="WDAP1-integ"`
2. Hand off any remaining checkpoint requirements to the operator (do not edit planning docs inside the worktree).
3. Do not delete the worktree (feature cleanup removes worktrees at feature end).

Naming note:
- The task id for the final aggregator is `WDAP1-integ`.

