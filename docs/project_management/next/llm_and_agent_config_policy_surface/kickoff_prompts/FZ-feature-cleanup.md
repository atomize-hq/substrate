# Kickoff: FZ-feature-cleanup (ops)

## Scope
- Run planning lint and clean up worktrees/branches after final integration.
- Standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm final integration is merged back to orchestration.

## Required commands
- `make planning-lint FEATURE_DIR="docs/project_management/next/llm_and_agent_config_policy_surface"`
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/llm_and_agent_config_policy_surface" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/llm_and_agent_config_policy_surface" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## End Checklist
1. Record cleanup outcome in `docs/project_management/next/llm_and_agent_config_policy_surface/session_log.md`.

