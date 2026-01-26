# Kickoff: WCU4-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/_archived/workspace-config-policy-unification/WCU4-spec.md`
- ADR: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu4-code` on branch `workspace_config_policy_unification-wcu4-code` and that `.taskmeta.json` exists.
2. Read: WCU4-spec, ADR-0008, plan/tasks/session log, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" SLICE_ID="WCU4"`

## Requirements
- Implement WCU4 acceptance criteria in `tasks.json`.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU4-code"`
