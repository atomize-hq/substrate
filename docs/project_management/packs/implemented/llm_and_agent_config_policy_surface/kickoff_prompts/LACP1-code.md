# Kickoff: LACP1-code (code)

## Scope

- Production code only.
- Spec: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`
- Schema: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
- Standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/llm-and-agent-config-policy-surface-lacp1-code` on branch `llm-and-agent-config-policy-surface-lacp1-code` and `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, LACP1-spec.md, SCHEMA.md.

## Requirements

- Implement exactly the behaviors in `LACP1-spec.md`.
- Implement `substrate agents validate` per spec and contract.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist

1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LACP1-code"`
3. Hand off validation notes to the operator.
