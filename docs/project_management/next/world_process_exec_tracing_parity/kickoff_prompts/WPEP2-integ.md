# Kickoff: WPEP2-integ (integration)

## Scope
- Merge WPEP2 code+test and validate WPEP2 behaviors.
- Run smoke with slice mode `WPEP2`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep2-integ` and `.taskmeta.json`.
2. Merge `WPEP2-code` + `WPEP2-test`.

## Requirements
- Run smoke:
  - `SUBSTRATE_SMOKE_SLICE_ID=WPEP2 bash docs/project_management/next/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
  - `SUBSTRATE_SMOKE_SLICE_ID=WPEP2 bash docs/project_management/next/world_process_exec_tracing_parity/smoke/macos-smoke.sh`

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP2-integ"`
