# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained task worktrees and close out the automation registry for this feature.
- This task runs on the orchestration checkout. No worktree is used.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on `feat/best-effort-distro-package-manager`.
2. Confirm all feature tasks are completed and merged as intended.
3. Read `plan.md`, `tasks.json`, `session_log.md`, and this prompt.

## Requirements
- Run the cleanup dry run first:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
- Run the cleanup after the dry run is clean:
  - `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`

## End Checklist
1. Paste the cleanup summary into `session_log.md`.
2. Mark `FZ-feature-cleanup` completed in `tasks.json`.
3. Hand off any retained worktrees or branches that still need operator action.
