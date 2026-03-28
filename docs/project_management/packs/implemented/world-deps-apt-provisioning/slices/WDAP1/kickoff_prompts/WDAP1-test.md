# Kickoff: WDAP1-test (test)

## Scope
- Tests only (plus minimal test-only helpers if needed); no production code.
- Spec: `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-apt-provisioning-wdap1-test` on branch `world-deps-apt-provisioning-wdap1-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`, `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP1"` (preferred; starts code+test in parallel)
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP1-test"` (single task only)

## Requirements
- Add/modify tests that enforce the spec’s acceptance criteria.
- If the spec implies large behavior changes across unrelated subsystems, stop and ask the operator to split the slice before proceeding.
- If completing this task requires more than 108,800 tokens of context, stop and ask the operator to split the slice before proceeding.
- Run: `cargo fmt`, plus the targeted tests you add/touch.

## End Checklist
1. Run required commands and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDAP1-test"`
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

