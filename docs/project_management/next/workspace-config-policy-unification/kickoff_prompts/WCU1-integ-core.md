# Kickoff: WCU1-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to ADR/spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/workspace-config-policy-unification/WCU1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu1-integ-core` on branch `workspace_config_policy_unification-wcu1-integ-core` and that `.taskmeta.json` exists.
2. Read: `docs/project_management/next/workspace-config-policy-unification/plan.md`, `docs/project_management/next/workspace-config-policy-unification/tasks.json`, `docs/project_management/next/workspace-config-policy-unification/session_log.md`, WCU1 spec, and this prompt.

## Requirements
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Dispatch compile parity (fast fail): `make ci-compile-parity CI_WORKFLOW_REF="feat/workspace-config-policy-unification" CI_REMOTE=origin CI_CLEANUP=1`
- Dispatch behavioral smoke via CI: `make feature-smoke FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" PLATFORM=behavior SMOKE_SLICE_ID="WCU1" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/workspace-config-policy-unification" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU1-integ-core"`

