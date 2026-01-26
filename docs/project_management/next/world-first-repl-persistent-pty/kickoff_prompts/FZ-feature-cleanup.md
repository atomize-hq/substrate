# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Feature-end cleanup only (no production code changes).
- Removes retained worktrees and prunes local task branches using `scripts/triad/feature_cleanup.sh`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are on the orchestration branch `feat/world-first-repl-persistent-pty`.
2. Confirm all tasks are `completed` and merged as intended (final integration tasks are the only merge-back points).

## End Checklist
1. Run a dry run:
   - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
2. Run the real cleanup:
   - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
3. Record the cleanup summary in `docs/project_management/next/world-first-repl-persistent-pty/session_log.md` and mark the task completed in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`.

