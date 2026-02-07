# Kickoff: CP1-ci-checkpoint (ops)

## Scope
- Run CI checkpoint gates after WPEP0 boundary.
- Plan: `docs/project_management/next/world_process_exec_tracing_parity/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm orchestration branch: `feat/world-process-exec-tracing-parity`.
2. Confirm WPEP0 final merge is integrated and pushed.

## Required commands
Run the checkpoint gates recorded in `tasks.json`:
- `scripts/ci-audit/ci_audit.sh ...`
- `make ci-compile-parity ...`
- `make ci-testing ...`

## End Checklist
1. Record run URLs/outcomes in `docs/project_management/next/world_process_exec_tracing_parity/session_log.md`.

