# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained worktrees and close out the feature task graph after all integration work is complete.
- This task runs on the orchestration checkout. It does not use a task worktree.

Do not edit planning docs inside the worktree.

## Preconditions
- All slice tasks and checkpoint tasks are completed.
- The orchestration checkout is clean before cleanup begins.

## Requirements
- Run the dry-run cleanup command first.
- Run the real cleanup command only after the dry-run output looks correct.
- Record the cleanup summary block in `session_log.md`.

## End Checklist
1. `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
2. `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
3. Mark `FZ-feature-cleanup` completed in `tasks.json` and add the END entry to `session_log.md`.
