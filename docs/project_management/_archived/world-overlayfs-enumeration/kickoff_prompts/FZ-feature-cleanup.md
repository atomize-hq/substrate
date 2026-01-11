# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Feature-level cleanup at the end of the feature:
  - remove retained task worktrees
  - optionally prune task branches (local and or remote)
- This task runs on the orchestration branch (no worktrees).

Do not edit planning docs inside the worktree.

## Preconditions
- All tasks for this feature are completed and any required merges are complete.
- Orchestration worktree is clean (no uncommitted changes).

## How to run (deterministic)

Dry-run (recommended first):
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Real run:
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Remote branch pruning (optional, destructive):
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration" PRUNE_REMOTE=origin PRUNE_LOCAL=1 REMOVE_WORKTREES=1`

If any worktree is dirty or any branch is unmerged or unpushed, the cleanup script refuses unless forced:
- add `FORCE=1` to the make invocation.

## Output requirements
- Paste the cleanup script stdout summary block into the END entry for this task in `docs/project_management/_archived/world-overlayfs-enumeration/session_log.md`.

