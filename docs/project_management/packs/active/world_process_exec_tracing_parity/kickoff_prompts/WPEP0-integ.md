# Kickoff: WPEP0-integ (integration)

## Scope
- Final WPEP0 merge: core + platform parity fixes.
- Spec: `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep0-integ` and `.taskmeta.json`.
2. Confirm `WPEP0-integ-core`, `WPEP0-integ-linux`, `WPEP0-integ-macos`, `WPEP0-integ-windows` are complete.

## Requirements
- Merge all WPEP0 branches.
- Run: `make integ-checks`

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP0-integ"`
2. Report integration + merge results to the operator.
