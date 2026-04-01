# Kickoff: WDRA1-test (test)

## Scope
- Test changes only.
- Spec: `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA1/WDRA1-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-disabled-reason-attribution-wdra1-test` on branch `world-disabled-reason-attribution-wdra1-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`, `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`, `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" SLICE_ID="WDRA1"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" TASK_ID="WDRA1-test"`

## Requirements
- Add or extend tests required by the spec.
- Keep tests deterministic and temp-dir based.
- Run the targeted tests you add or update.
- Targeted test focus:
  - replay stderr copy and replay_strategy field assertions inside `crates/shell/tests/replay_world.rs`

## End Checklist
1. Run targeted tests and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDRA1-test"`
3. Hand off test commands and outcomes to the operator.
4. Do not delete the worktree.
