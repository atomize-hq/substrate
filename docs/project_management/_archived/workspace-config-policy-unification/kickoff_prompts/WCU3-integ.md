# Kickoff: WCU3-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize WCU3 with an auditable cross-platform green state.
- Spec: `docs/project_management/_archived/workspace-config-policy-unification/WCU3-spec.md`
- ADRs:
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- Gate file: `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
- Closeout report: `docs/project_management/_archived/workspace-config-policy-unification/WCU3-closeout_report.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu3-integ` on branch `workspace_config_policy_unification-wcu3-integ` and that `.taskmeta.json` exists.
2. Read: WCU3-spec, ADR-0008, ADR-0012, PHASE_A_B_GATES, plan/tasks/session log, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" TASK_ID="WCU3-integ"`

## Requirements
- Merge `WCU3-integ-core` and any platform-fix branches for WCU3 that produced commits.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Dispatch feature smoke for behavior platforms and record evidence:
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" PLATFORM=behavior SMOKE_SLICE_ID="WCU3" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/workspace-config-policy-unification" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Complete `WCU3-closeout_report.md` with Phase B evidence and smoke results.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU3-integ"`
