# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained triad worktrees after all slice integrations finish.
- Stay on the orchestration checkout for `feat/persist-detected-linux-distro-pkg-manager`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm every task in `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` is completed and merged as intended.
2. Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md` and this prompt.

## Requirements
- Run `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`.
- Run `make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`.
- Capture the cleanup summary for the END entry.

## End Checklist
1. Record the cleanup summary in `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`.
2. Set task status to `completed` in `tasks.json`, add an END entry, and commit docs on the orchestration branch.
