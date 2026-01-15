# Kickoff: WCU4-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/next/workspace-config-policy-unification/WCU4-spec.md`
- ADR: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Closeout report: `docs/project_management/next/workspace-config-policy-unification/WCU4-closeout_report.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu4-integ` on branch `workspace_config_policy_unification-wcu4-integ` and that `.taskmeta.json` exists.
2. Read: WCU4-spec, ADR-0008, plan/tasks/session log, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" TASK_ID="WCU4-integ"`

## Requirements
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Dispatch feature smoke for behavior platforms and record evidence:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" PLATFORM=behavior WORKFLOW_REF="feat/workspace-config-policy-unification"`

## End Checklist
1. Complete `WCU4-closeout_report.md` with evidence and smoke results.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU4-integ"`
