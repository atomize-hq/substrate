# Kickoff: LACP1-integ-core (integration)

## Scope

- Merge LACP1 code+test and validate core invariants before CI checkpoint gates.
- Spec: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`
- Standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/llm-and-agent-config-policy-surface-lacp1-integ-core` on branch `llm-and-agent-config-policy-surface-lacp1-integ-core` and `.taskmeta.json` exists at the worktree root.
2. Merge `LACP1-code` and `LACP1-test` branches into this integration branch.

## Required commands

- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p shell`
- `cargo test -p substrate-broker`
- `make integ-checks`

## End Checklist

1. Run required commands; capture outputs.
2. Push the branch so CP1 can dispatch smoke on this commit.
3. From inside the worktree, run: `make triad-task-finish TASK_ID="LACP1-integ-core"`
