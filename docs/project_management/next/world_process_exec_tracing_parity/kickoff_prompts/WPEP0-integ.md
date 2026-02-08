# Kickoff: WPEP0-integ (integration)

## Scope
- Final WPEP0 merge: core + platform parity fixes.
- Run smoke with slice mode `WPEP0`.
- Spec: `docs/project_management/next/world_process_exec_tracing_parity/WPEP0-spec.md`
- Smoke scripts:
  - `docs/project_management/next/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
  - `docs/project_management/next/world_process_exec_tracing_parity/smoke/macos-smoke.sh`
  - `docs/project_management/next/world_process_exec_tracing_parity/smoke/windows-smoke.ps1`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep0-integ` and `.taskmeta.json`.
2. Confirm `WPEP0-integ-core`, `WPEP0-integ-linux`, `WPEP0-integ-macos`, `WPEP0-integ-windows` are complete.

## Requirements
- Merge all WPEP0 branches.
- Run smoke:
  - `SUBSTRATE_SMOKE_SLICE_ID=WPEP0 bash docs/project_management/next/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
  - `SUBSTRATE_SMOKE_SLICE_ID=WPEP0 bash docs/project_management/next/world_process_exec_tracing_parity/smoke/macos-smoke.sh`

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP0-integ"`
2. Report smoke + merge results to the operator.

