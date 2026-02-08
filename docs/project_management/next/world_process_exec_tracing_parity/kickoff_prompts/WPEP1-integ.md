# Kickoff: WPEP1-integ (integration)

## Scope
- Merge WPEP1 code+test and run smoke with slice mode `WPEP1`.
- Spec: `docs/project_management/next/world_process_exec_tracing_parity/WPEP1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep1-integ` and `.taskmeta.json`.
2. Merge `WPEP1-code` + `WPEP1-test`.

## Requirements
- Run smoke:
  - `SUBSTRATE_SMOKE_SLICE_ID=WPEP1 bash docs/project_management/next/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
  - `SUBSTRATE_SMOKE_SLICE_ID=WPEP1 bash docs/project_management/next/world_process_exec_tracing_parity/smoke/macos-smoke.sh`

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP1-integ"`

