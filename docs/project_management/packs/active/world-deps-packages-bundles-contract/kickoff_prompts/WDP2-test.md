# Kickoff: WDP2-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP2-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp2-test` on branch `world-deps-packages-bundles-contract-wdp2-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/active/world-deps-packages-bundles-contract/plan.md`, `docs/project_management/packs/active/world-deps-packages-bundles-contract/tasks.json`, `docs/project_management/packs/active/world-deps-packages-bundles-contract/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/world-deps-packages-bundles-contract" SLICE_ID="WDP2"`

## Requirements
- Add/modify tests that enforce the WDP2 acceptance criteria.
- Run: `cargo fmt`, plus the targeted tests you add/touch.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP2-test"`
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree.

