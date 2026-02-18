# Kickoff: LACP0-code (code)

## Scope
- Production code only.
- Spec: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP0-spec.md`
- Schema: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
- Standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/llm-and-agent-config-policy-surface-lacp0-code` on branch `llm-and-agent-config-policy-surface-lacp0-code` and `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, LACP0-spec.md, SCHEMA.md.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/llm_and_agent_config_policy_surface" SLICE_ID="LACP0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/llm_and_agent_config_policy_surface" TASK_ID="LACP0-code"`

## Requirements
- Implement exactly the behaviors in `LACP0-spec.md`.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="LACP0-code"`
3. Hand off validation notes to the operator.

