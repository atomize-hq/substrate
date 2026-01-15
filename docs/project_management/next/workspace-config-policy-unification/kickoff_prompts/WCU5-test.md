# Kickoff: WCU5-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Requirements
- Add/modify tests that enforce WCU5 acceptance criteria in `tasks.json`.
- Run: `cargo fmt`, plus targeted tests you add/touch.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU5-test"`
