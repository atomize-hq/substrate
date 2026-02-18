# Kickoff: WDP1-test (test)

## Scope
- Tests only (plus minimal test-only helpers if absolutely needed); no production code.
- Spec: `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp1-test` on branch `world-deps-packages-bundles-contract-wdp1-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/active/world-deps-packages-bundles-contract/plan.md`, `docs/project_management/packs/active/world-deps-packages-bundles-contract/tasks.json`, `docs/project_management/packs/active/world-deps-packages-bundles-contract/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/world-deps-packages-bundles-contract" SLICE_ID="WDP1"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/world-deps-packages-bundles-contract" TASK_ID="WDP1-test"`

## Requirements
- Add/modify tests that enforce the spec’s acceptance criteria.
- If the spec implies large/sweeping behavior changes, stop and ask the operator to split the slice so the test task can stay focused and reviewable.
- If completing this task requires more than 108,800 tokens of context, stop and ask the operator to split the slice before proceeding.
- Run: `cargo fmt`, plus the targeted tests you add/touch.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP1-test"`
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

