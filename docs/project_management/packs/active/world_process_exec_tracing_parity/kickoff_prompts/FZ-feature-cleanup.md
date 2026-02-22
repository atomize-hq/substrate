# Kickoff: FZ-feature-cleanup (ops)

## Scope

- Final planning lint and feature cleanup.
- Standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Confirm final integration task is complete and merged to orchestration.
2. Run planning lint and fix any issues before cleanup.

## Required commands

- `make planning-lint FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity"`
- Dry run cleanup:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Execute cleanup:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## End Checklist

1. Record cleanup outcome in `docs/project_management/packs/active/world_process_exec_tracing_parity/session_log.md`.
