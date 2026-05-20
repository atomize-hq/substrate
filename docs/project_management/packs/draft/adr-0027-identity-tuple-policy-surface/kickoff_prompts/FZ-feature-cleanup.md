# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained worktrees and prune local task branches after the feature is complete.
- Run from the orchestration checkout, not a task worktree.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Read `plan.md`, `tasks.json`, and `session_log.md`.
2. Confirm every slice task and `CP1-ci-checkpoint` is completed.
3. Use the deterministic cleanup commands from `tasks.json`.

## Requirements
- Run the dry-run cleanup first.
- Run the real cleanup only after the dry run is acceptable.
- Paste the cleanup summary into `session_log.md`.

## End Checklist
1. Mark `FZ-feature-cleanup` completed in `tasks.json`.
2. Record the cleanup summary in `session_log.md`.
3. Leave the Planning Pack files on the orchestration checkout.
