# Kickoff: WDP3-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/next/world-deps-packages-bundles-contract/WDP3-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp3-integ` on branch `world-deps-packages-bundles-contract-wdp3-integ` and that `.taskmeta.json` exists.
2. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Reconcile code/tests to spec (spec wins).
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Complete: `docs/project_management/next/world-deps-packages-bundles-contract/WDP3-closeout_report.md`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP3-integ"`
2. Do not delete the worktree.

