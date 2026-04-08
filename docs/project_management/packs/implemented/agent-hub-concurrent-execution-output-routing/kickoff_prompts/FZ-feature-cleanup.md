# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Feature-level cleanup at the end of the feature:
  - remove all retained task worktrees
  - prune local task branches
- This task runs on the orchestration branch (no worktrees).

Do not edit planning docs inside the worktree.

## Preconditions
- All tasks for this feature are completed and required merges are done.
- Orchestration checkout is clean (no uncommitted changes).

## How to run (deterministic)

Dry-run:
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Real run:
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

Remote branch pruning mode (destructive):
- `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing" PRUNE_REMOTE=origin PRUNE_LOCAL=1 REMOVE_WORKTREES=1`

If any worktree is dirty or any branch is unmerged/unpushed, cleanup refuses unless forced:
- add `FORCE=1` to the make invocation.

## Output requirements
- Paste the script stdout summary block into the END entry for this task in `session_log.md`.

