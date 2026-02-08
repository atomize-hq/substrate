# Kickoff: WPEP0-code (code)

## Scope
- Production code only; add tests only if required to maintain correctness.
- Spec: `docs/project_management/next/world_process_exec_tracing_parity/WPEP0-spec.md`
- Related contracts: `docs/project_management/next/world_process_exec_tracing_parity/SCHEMA.md`, `docs/project_management/next/world_process_exec_tracing_parity/SECURITY.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/world-process-exec-tracing-parity-wpep0-code` on branch `world-process-exec-tracing-parity-wpep0-code` and `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world_process_exec_tracing_parity/plan.md`, `docs/project_management/next/world_process_exec_tracing_parity/tasks.json`, `docs/project_management/next/world_process_exec_tracing_parity/session_log.md`, WPEP0 spec, decision register DR references.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world_process_exec_tracing_parity" SLICE_ID="WPEP0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world_process_exec_tracing_parity" TASK_ID="WPEP0-code"`

## Requirements
- Implement exactly the behaviors in `WPEP0-spec.md`.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WPEP0-code"`
3. Hand off validation notes to the operator (do not edit planning docs inside the worktree).

