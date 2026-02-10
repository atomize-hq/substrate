# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Feature-level cleanup at the end of the feature:
  - remove all retained task worktrees
- Prune task branches locally (do not prune remote branches; keep remote branches for auditability).
- This task runs on the orchestration branch (no worktrees).

Do not edit planning docs inside the worktree.

## Preconditions
- All tasks for this feature are completed and any required merges are done.
- Orchestration worktree is clean (no uncommitted changes).

## How to run (deterministic)
This feature uses the triad automation registry stored in the shared git directory:
- `GIT_COMMON_DIR="$(git rev-parse --git-common-dir)"`
- Registry: `"$GIT_COMMON_DIR/triad/features/world-sync/worktrees.json"`

Dry-run (required first):
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/world-sync" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Real run (only after dry-run output is clean):
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/world-sync" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

If any worktree is dirty or any branch is unmerged/unpushed, the cleanup script refuses unless forced:
- add `FORCE=1` to the make invocation.

## Output requirements
- Paste the script stdout summary block into the END entry for this task in `session_log.md`.
