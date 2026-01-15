# Kickoff: WCU1-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- Closeout report: `docs/project_management/next/workspace-config-policy-unification/WCU1-closeout_report.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu1-integ` on branch `workspace_config_policy_unification-wcu1-integ` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session log + ADR-0008 + this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" TASK_ID="WCU1-integ"`

## Requirements
- Reconcile code/tests to ADR-0008 (ADR wins).
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Run smoke scripts for behavior platforms and record evidence:
  - `docs/project_management/next/workspace-config-policy-unification/smoke/linux-smoke.sh`
  - `docs/project_management/next/workspace-config-policy-unification/smoke/macos-smoke.sh`
  - `docs/project_management/next/workspace-config-policy-unification/smoke/windows-smoke.ps1`

## End Checklist
1. Complete `WCU1-closeout_report.md` with commands run and smoke evidence.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU1-integ"`
