# Kickoff: WDP2-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable merged state.
- Spec: `docs/project_management/next/world-deps-packages-bundles-contract/WDP2-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-packages-bundles-contract-wdp2-integ` on branch `world-deps-packages-bundles-contract-wdp2-integ` and that `.taskmeta.json` exists.
2. Read: plan, tasks, session_log, spec, this prompt.

## Requirements
- Merge WDP2 integration branches:
  - `world-deps-packages-bundles-contract-wdp2-integ-core`
  - `world-deps-packages-bundles-contract-wdp2-integ-linux`
  - `world-deps-packages-bundles-contract-wdp2-integ-macos`
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Do not dispatch cross-platform CI from this task; CP1 owns checkpoint CI.
- Complete: `docs/project_management/next/world-deps-packages-bundles-contract/WDP2-closeout_report.md`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WDP2-integ"`
2. Hand off closeout report completion and checkpoint status to the operator.
