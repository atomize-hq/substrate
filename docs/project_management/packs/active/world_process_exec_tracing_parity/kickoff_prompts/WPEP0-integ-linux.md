# Kickoff: WPEP0-integ-linux (integration)

## Scope

- Linux parity fixes for WPEP0 after `WPEP0-integ-core`.
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep0-integ-linux` and `.taskmeta.json`.
2. Read: WPEP0 spec and tasks.json acceptance criteria.

## Requirements

- Fix Linux-only failures surfaced by CI parity or local validation.

## End Checklist

1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP0-integ-linux"`
