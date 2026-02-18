# Kickoff: WDP4-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP4-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp4-code` on branch `world-deps-packages-bundles-contract-wdp4-code` and that `.taskmeta.json` exists.
2. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Implement exactly the behaviors and error handling in the spec.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP4-code"`
2. Do not delete the worktree.

