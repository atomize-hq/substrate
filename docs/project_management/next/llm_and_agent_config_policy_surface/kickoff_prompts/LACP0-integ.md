# Kickoff: LACP0-integ (integration)

## Scope
- Merge LACP0 code+test and validate per spec.
- Spec: `docs/project_management/next/llm_and_agent_config_policy_surface/LACP0-spec.md`
- Manual playbook: `docs/project_management/next/llm_and_agent_config_policy_surface/manual_testing_playbook.md`
- Standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/llm-and-agent-config-policy-surface-lacp0-integ` on branch `llm-and-agent-config-policy-surface-lacp0-integ` and `.taskmeta.json` exists at the worktree root.
2. Merge `LACP0-code` and `LACP0-test` branches into this integration branch.

## Required commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p shell`
- `cargo test -p substrate-broker`
- `make integ-checks`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LACP0-integ"`
3. Hand off results to the operator and confirm merge-back to orchestration is unblocked.

