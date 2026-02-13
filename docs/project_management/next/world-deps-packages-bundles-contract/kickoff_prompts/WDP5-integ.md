# Kickoff: WDP5-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable merged state.
- Spec: `docs/project_management/next/world-deps-packages-bundles-contract/WDP5-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp5-integ` on branch `world-deps-packages-bundles-contract-wdp5-integ` and that `.taskmeta.json` exists.
2. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Merge WDP5 integration branches:
  - `world-deps-packages-bundles-contract-wdp5-integ-core`
  - `world-deps-packages-bundles-contract-wdp5-integ-linux`
  - `world-deps-packages-bundles-contract-wdp5-integ-macos`
  - `world-deps-packages-bundles-contract-wdp5-integ-windows`
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Do not dispatch cross-platform CI from this task; CP2 owns checkpoint CI.
- Complete: `docs/project_management/next/world-deps-packages-bundles-contract/WDP5-closeout_report.md`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP5-integ"`
2. Hand off closeout report completion and checkpoint status to the operator.

