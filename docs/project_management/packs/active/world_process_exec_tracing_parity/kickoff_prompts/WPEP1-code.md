# Kickoff: WPEP1-code (code)

## Scope
- Production code for WPEP1 protocol + persistence plumbing.
- Spec: `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP1-spec.md`
- Protocol: `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep1-code` and `.taskmeta.json`.
2. Read: plan.md, tasks.json, WPEP1 spec, PROTOCOL.md.

## Requirements
- Implement exactly WPEP1 spec acceptance behaviors.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`.

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP1-code"`

