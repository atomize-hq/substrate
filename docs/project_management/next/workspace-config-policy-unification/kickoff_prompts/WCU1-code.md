# Kickoff: WCU1-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu1-code` on branch `workspace_config_policy_unification-wcu1-code` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session log + ADR-0008 + this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" SLICE_ID="WCU1"`

## Requirements
- Implement exactly the ADR-0008 WCU1 contract for workspace layout and discovery.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- Baseline testing (required): run a targeted baseline test set before changes, then re-run after changes.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU1-code"`
