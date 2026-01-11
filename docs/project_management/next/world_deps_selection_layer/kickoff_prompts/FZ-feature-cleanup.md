# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Feature-level cleanup at the end of the feature:
  - remove all retained task worktrees
  - optionally prune task branches (local and/or remote)
- This task runs on the orchestration branch (no worktrees).

Do not edit planning docs inside the worktree.

## Preconditions
- All tasks for this feature are completed and any required merges are done.
- Orchestration worktree is clean (no uncommitted changes).

## How to run (deterministic)
This feature uses the triad automation registry stored in the shared git directory:
- `<git-common-dir>/triad/features/world_deps_selection_layer/worktrees.json`

Dry-run (required first):
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Real run:
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Remote branch pruning (destructive; do not run unless explicitly required by the operator):
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PRUNE_REMOTE=origin PRUNE_LOCAL=1 REMOVE_WORKTREES=1`

If any worktree is dirty or any branch is unmerged/unpushed, the cleanup script refuses unless forced:
- add `FORCE=1` to the make invocation.

## Output requirements
- Paste the script stdout summary block into the END entry for this task in `docs/project_management/next/world_deps_selection_layer/session_log.md`.

