# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained task worktrees for this feature and optionally prune local branches via triad automation.
- Standard: `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify all tasks in `docs/project_management/next/doctor_scopes/tasks.json` are `completed`.
2. Ensure the orchestration checkout is clean (no staged or unstaged changes).
3. Dry-run cleanup:
   - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/doctor_scopes" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## Requirements

- Execute cleanup and paste the printed summary block into the session log END entry for this task:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/doctor_scopes" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## End Checklist

1. Paste cleanup summary block into `docs/project_management/next/doctor_scopes/session_log.md`.
2. Mark `FZ-feature-cleanup` as `completed` in `docs/project_management/next/doctor_scopes/tasks.json` and add an END entry.
3. Commit docs (`docs: finish FZ-feature-cleanup`).

