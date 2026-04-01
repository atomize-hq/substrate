# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Feature-level cleanup at the end of the feature:
  - remove retained task worktrees
  - prune local task branches when safe
- This task runs on the orchestration checkout.

Do not edit planning docs inside the worktree.

## Preconditions
- All tasks for this feature are complete.
- The orchestration checkout is clean.

## How to run
- Dry run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Real run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## Output requirements
- Paste the cleanup summary into the END entry for this task in `session_log.md`.
