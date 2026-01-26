# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Feature-level cleanup at the end of the feature:
  - remove all retained task worktrees
  - optionally prune task branches (local and/or remote)
- This task runs on the orchestration branch (no worktrees).

Do not edit planning docs inside the worktree.

## Start Checklist
1. Verify you are on the orchestration branch `feat/workspace-config-policy-unification` and the worktree is clean.
2. Read: `docs/project_management/_archived/workspace-config-policy-unification/tasks.json` and `docs/project_management/_archived/workspace-config-policy-unification/session_log.md`.

## Preconditions
- All tasks for this feature are completed and any required merges are done.
- Orchestration worktree is clean (no uncommitted changes).

## How to run (deterministic)
Dry-run first:
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Real run:
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/_archived/workspace-config-policy-unification" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## Output requirements
- Paste the script stdout summary block into the END entry for this task in `session_log.md`.
