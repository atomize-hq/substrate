# Kickoff: LACP1-test (test)

## Scope
- Tests only.
- Spec: `docs/project_management/next/llm_and_agent_config_policy_surface/LACP1-spec.md`
- Standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/llm-and-agent-config-policy-surface-lacp1-test` on branch `llm-and-agent-config-policy-surface-lacp1-test` and `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, LACP1-spec.md.

## Requirements
- Encode LACP1 acceptance criteria as tests.
- Run: `cargo fmt`, targeted `cargo test …`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LACP1-test"`
3. Hand off notes to integration (expected red/green transitions).

