# Kickoff: WPEP1-integ (integration)

## Scope
- Merge WPEP1 code+test.
- Spec: `docs/project_management/next/world_process_exec_tracing_parity/WPEP1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep1-integ` and `.taskmeta.json`.
2. Merge `WPEP1-code` + `WPEP1-test`.

## Requirements
- Run: `make integ-checks`

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP1-integ"`
