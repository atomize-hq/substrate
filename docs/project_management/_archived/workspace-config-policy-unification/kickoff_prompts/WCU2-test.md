# Kickoff: WCU2-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- Gate file: `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu2-test` on branch `workspace_config_policy_unification-wcu2-test` and that `.taskmeta.json` exists.
2. Read: ADR-0012, PHASE_A_B_GATES, plan/tasks/session log, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" SLICE_ID="WCU2"`

## Requirements
- Add/modify tests that enforce Phase A acceptance criteria from `tasks.json` and `PHASE_A_B_GATES_ADR_0012.md`.
- Run: `cargo fmt`, plus targeted tests you add/touch.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU2-test"`
