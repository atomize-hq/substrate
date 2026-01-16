# Kickoff: WCU1-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize WCU1 with an auditable cross-platform green state.
- Spec: `docs/project_management/next/workspace-config-policy-unification/WCU1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- Closeout report: `docs/project_management/next/workspace-config-policy-unification/WCU1-closeout_report.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu1-integ` on branch `workspace_config_policy_unification-wcu1-integ` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session log + WCU1 spec + this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" TASK_ID="WCU1-integ"`

## Requirements
- Merge `WCU1-integ-core` and any platform-fix branches for WCU1 that produced commits.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Dispatch behavioral smoke via CI from this worktree’s `HEAD`:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" PLATFORM=behavior SMOKE_SLICE_ID="WCU1" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/workspace-config-policy-unification" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Complete `WCU1-closeout_report.md` with commands run and smoke evidence.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU1-integ"`
