# Kickoff: WCU3-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- Gate file: `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Requirements
- Add/modify tests that enforce Phase B acceptance criteria from `tasks.json` and `PHASE_A_B_GATES_ADR_0012.md`.
- Run: `cargo fmt`, plus targeted tests you add/touch.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU3-test"`
