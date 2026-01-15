# Kickoff: WCU1-test (test)

## Scope
- Tests only (plus minimal test-only helpers if absolutely needed); no production code.
- Spec: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu1-test` on branch `workspace_config_policy_unification-wcu1-test` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session log + ADR-0008 + this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" SLICE_ID="WCU1"`

## Requirements
- Add/modify tests that enforce the WCU1 acceptance criteria in `tasks.json`.
- Run: `cargo fmt`, plus the targeted tests you add/touch.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU1-test"`
