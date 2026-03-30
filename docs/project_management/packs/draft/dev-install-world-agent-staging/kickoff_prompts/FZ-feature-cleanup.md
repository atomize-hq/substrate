# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained worktrees and close out the feature after `DIWAS1-integ` and `CP1-ci-checkpoint` are complete.
- This task runs on the orchestration checkout (no worktree).

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/dev-install-world-agent-staging`.
2. Confirm all tasks are complete and merged as intended:
   - `DIWAS0-integ`
   - `DIWAS1-integ`
   - `CP1-ci-checkpoint`
3. Read `docs/project_management/packs/draft/dev-install-world-agent-staging/session_log.md` and ensure it has an END entry for the boundary slice.

## Commands

Dry run:
```bash
make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-agent-staging" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1
```

Execute:
```bash
make triad-feature-cleanup FEATURE_DIR="docs/project_management/packs/draft/dev-install-world-agent-staging" REMOVE_WORKTREES=1 PRUNE_LOCAL=1
```

## End Checklist
1. Paste the cleanup summary into the END entry for this task in `docs/project_management/packs/draft/dev-install-world-agent-staging/session_log.md`.
2. Mark `FZ-feature-cleanup` complete via `make triad-task-finish TASK_ID="FZ-feature-cleanup"` (or the pack’s standard closeout path).
