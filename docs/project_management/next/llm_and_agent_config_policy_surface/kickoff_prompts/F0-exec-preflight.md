# Kickoff: F0-exec-preflight (ops)

## Scope
- Validate Planning Pack mechanical invariants before starting LACP0 execution triads.
- Standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Read: `docs/project_management/next/llm_and_agent_config_policy_surface/plan.md`, `docs/project_management/next/llm_and_agent_config_policy_surface/spec_manifest.md`, `docs/project_management/next/llm_and_agent_config_policy_surface/tasks.json`.
2. Confirm the feature directory is present in `docs/project_management/next/sequencing.json`.

## Required commands
- `make planning-lint FEATURE_DIR="docs/project_management/next/llm_and_agent_config_policy_surface"`
- `bash -n docs/project_management/next/llm_and_agent_config_policy_surface/smoke/linux-smoke.sh`
- `bash -n docs/project_management/next/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh`

## End Checklist
1. Record command outputs in `docs/project_management/next/llm_and_agent_config_policy_surface/session_log.md`.

