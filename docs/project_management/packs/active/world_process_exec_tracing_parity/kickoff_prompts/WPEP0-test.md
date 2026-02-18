# Kickoff: WPEP0-test (test)

## Scope
- Tests for WPEP0 behaviors in `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP0-spec.md`.
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep0-test` on branch `world-process-exec-tracing-parity-wpep0-test` and `.taskmeta.json` exists.
2. Read: WPEP0 spec and referenced DR entries.
3. If task metadata is missing, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity" SLICE_ID="WPEP0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity" TASK_ID="WPEP0-test"`

## Requirements
- Add tests that directly assert WPEP0 acceptance criteria.

## End Checklist
1. Run relevant tests (see tasks.json acceptance criteria).
2. From inside the worktree: `make triad-task-finish TASK_ID="WPEP0-test"`

