# Kickoff: WCU3-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/_archived/workspace-config-policy-unification/WCU3-spec.md`
- ADR: `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- Gate file: `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu3-test` on branch `workspace_config_policy_unification-wcu3-test` and that `.taskmeta.json` exists.
2. Read: WCU3-spec, ADR-0012, PHASE_A_B_GATES, world_deps_packages_bundles_contract.md, plan/tasks/session log, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" SLICE_ID="WCU3"`

## Requirements
- Add/modify tests that enforce Phase B acceptance criteria from `tasks.json` and `PHASE_A_B_GATES_ADR_0012.md`.
- Run: `cargo fmt`, plus targeted tests you add/touch.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU3-test"`
