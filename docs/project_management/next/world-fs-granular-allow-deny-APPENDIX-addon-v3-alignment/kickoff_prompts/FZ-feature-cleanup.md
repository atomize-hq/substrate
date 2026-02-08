# Kickoff: FZ-feature-cleanup (feature cleanup)

## Scope
- Remove retained worktrees and optionally prune branches after the add-on pack is completed.
- Runs on orchestration checkout (no worktrees).

## Requirements
- Use the commands in `tasks.json` acceptance criteria for this task.
- Do not edit planning docs inside the worktree.
- Do not delete worktrees manually; use `make triad-feature-cleanup`.
