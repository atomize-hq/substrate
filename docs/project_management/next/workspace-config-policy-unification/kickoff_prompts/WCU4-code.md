# Kickoff: WCU4-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Requirements
- Implement WCU4 acceptance criteria in `tasks.json`.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU4-code"`
