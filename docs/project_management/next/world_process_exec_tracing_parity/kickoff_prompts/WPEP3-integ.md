# Kickoff: WPEP3-integ (integration)

## Scope
- Final WPEP3 merge: core + platform parity fixes.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep3-integ` and `.taskmeta.json`.
2. Confirm `WPEP3-integ-core`, `WPEP3-integ-linux`, `WPEP3-integ-macos`, `WPEP3-integ-windows` are complete.

## Requirements
- Run: `make integ-checks`

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP3-integ"`
