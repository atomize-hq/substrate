# Kickoff: WCU3-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Gate file: `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Requirements
- Implement Phase B requirements from `PHASE_A_B_GATES_ADR_0012.md` (config editor supports `world.deps.enabled`).
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU3-code"`
