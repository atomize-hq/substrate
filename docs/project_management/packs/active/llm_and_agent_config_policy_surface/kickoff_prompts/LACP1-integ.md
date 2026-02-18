# Kickoff: LACP1-integ (integration final)

## Scope
- Merge LACP1 core + platform-fix branches, re-run integration gates, and merge back to orchestration.
- Spec: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`
- Manual playbook: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/llm-and-agent-config-policy-surface-lacp1-integ` on branch `llm-and-agent-config-policy-surface-lacp1-integ` and `.taskmeta.json` exists at the worktree root.
2. Merge `LACP1-integ-core`, `LACP1-integ-linux`, and `LACP1-integ-macos` into this branch.

## Required commands
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `make integ-checks`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LACP1-integ"`
3. Hand off results to the operator and confirm merge-back to orchestration is unblocked.

