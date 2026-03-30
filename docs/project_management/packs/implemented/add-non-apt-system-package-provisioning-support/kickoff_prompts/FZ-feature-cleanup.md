# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained task worktrees and finish the feature once all slice and checkpoint tasks are complete.
- This task runs on the orchestration branch.

Do not edit planning docs inside the worktree.

## Preconditions
- All tasks are completed and merged as intended.
- The orchestration checkout is clean.

## How to run
- Dry run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Real run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## End Checklist
1. Paste the cleanup summary into the `session_log.md` END entry.
2. Mark the task completed on the orchestration branch.
