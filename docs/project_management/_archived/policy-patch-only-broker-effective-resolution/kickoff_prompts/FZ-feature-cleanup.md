# Kickoff: FZ-feature-cleanup (ops)

## Scope
- Remove retained worktrees and optionally prune task branches at feature end.
- Tooling: `make triad-feature-cleanup`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run from the orchestration checkout (not a task worktree).
2. Confirm `C0-integ` is completed and merged to orchestration.

## End Checklist
1. Dry-run cleanup: `make triad-feature-cleanup FEATURE_DIR="docs/project_management/_archived/policy-patch-only-broker-effective-resolution" DRY_RUN=1 REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
2. Execute cleanup: `make triad-feature-cleanup FEATURE_DIR="docs/project_management/_archived/policy-patch-only-broker-effective-resolution" REMOVE_WORKTREES=1 PRUNE_LOCAL=1`
