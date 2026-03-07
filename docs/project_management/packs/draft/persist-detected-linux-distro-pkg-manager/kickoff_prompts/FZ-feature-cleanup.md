# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Run the feature-level cleanup after every slice integration task is complete.
- This task runs on `feat/persist-detected-linux-distro-pkg-manager` and does not use a task worktree.

Do not edit planning docs inside the worktree.

## Preconditions
- `PDLDPM0-integ`, `PDLDPM1-integ`, `PDLDPM3-integ`, and `PDLDPM2-integ` are completed.
- The orchestration checkout is clean before cleanup starts.

## How to run
- Dry run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Real run:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## End Checklist
1. Paste the cleanup summary into the `FZ-feature-cleanup` END entry in `session_log.md`.
2. Mark `FZ-feature-cleanup` completed in `tasks.json` and commit docs.
