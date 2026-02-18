# Kickoff: WDP4-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP4-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp4-test` on branch `world-deps-packages-bundles-contract-wdp4-test` and that `.taskmeta.json` exists.
2. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Add/modify tests that enforce the WDP4 acceptance criteria.
- Run: `cargo fmt`, plus the targeted tests you add/touch.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP4-test"`
2. Do not delete the worktree.

