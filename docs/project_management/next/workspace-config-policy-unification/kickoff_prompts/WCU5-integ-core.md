# Kickoff: WCU5-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to ADR/spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/workspace-config-policy-unification/WCU5-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are in the task worktree `wt/workspace_config_policy_unification-wcu5-integ-core` on branch `workspace_config_policy_unification-wcu5-integ-core` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session log + WCU5 spec + this prompt.

## Requirements
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Dispatch compile parity (fast fail): `make ci-compile-parity CI_WORKFLOW_REF="feat/workspace-config-policy-unification" CI_REMOTE=origin CI_CLEANUP=1`
- Dispatch behavioral smoke via CI: `make feature-smoke FEATURE_DIR="docs/project_management/next/workspace-config-policy-unification" PLATFORM=behavior SMOKE_SLICE_ID="WCU5" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/workspace-config-policy-unification" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WCU5-integ-core"`

