# Kickoff: WCU5-integ-windows (integration platform-fix — windows)

## Scope
- Ensure the slice is green for windows (behavior platform; smoke required).
- Spec: `docs/project_management/_archived/workspace-config-policy-unification/WCU5-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Run this task on a machine that matches the required platform: windows.
2. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu5-integ-windows` on branch `workspace_config_policy_unification-wcu5-integ-windows` and that `.taskmeta.json` exists.
3. Merge the slice core integration branch (`WCU5-integ-core`) before validating smoke or making fixes.

## Requirements
- Dispatch platform smoke via CI until green:
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" PLATFORM=windows SMOKE_SLICE_ID="WCU5" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/workspace-config-policy-unification" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU5-integ-windows"`

