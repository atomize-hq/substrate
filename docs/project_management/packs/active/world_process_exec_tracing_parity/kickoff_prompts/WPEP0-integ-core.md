# Kickoff: WPEP0-integ-core (integration)

## Scope

- Merge WPEP0 code+test worktrees and validate core invariants before platform parity fixes.
- Spec: `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-process-exec-tracing-parity-wpep0-integ-core` on branch `world-process-exec-tracing-parity-wpep0-integ-core` and `.taskmeta.json` exists.
2. Read: plan.md, tasks.json, WPEP0 spec.

## Requirements

- Merge `WPEP0-code` and `WPEP0-test`.
- Run core checks: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, tests listed in tasks.json, `make integ-checks`.

## End Checklist

1. From inside the worktree: `make triad-task-finish TASK_ID="WPEP0-integ-core"`
2. Report merge + test results to the operator.
