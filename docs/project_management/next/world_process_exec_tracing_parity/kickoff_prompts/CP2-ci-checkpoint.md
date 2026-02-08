# Kickoff: CP2-ci-checkpoint (ops)

## Scope
- Run CI checkpoint gates after WPEP3 boundary (feature completion).
- Plan: `docs/project_management/next/world_process_exec_tracing_parity/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm orchestration branch: `feat/world-process-exec-tracing-parity`.
2. Confirm WPEP3 final merge is integrated and pushed.

## Required commands
Run the checkpoint gates recorded in `tasks.json`.

Suggested env setup (copy/paste):
```bash
export FEATURE_DIR="docs/project_management/next/world_process_exec_tracing_parity"
export ORCH_REF="feat/world-process-exec-tracing-parity"
export CHECKOUT_SHA="$(git rev-parse HEAD)"
```

Then run:
- `scripts/ci-audit/ci_audit.sh ...`
- `make ci-compile-parity CI_WORKFLOW_REF="$ORCH_REF" CI_REMOTE=origin CI_CLEANUP=1`
- `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=behavior SMOKE_SLICE_ID="WPEP3" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_REF" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## End Checklist
1. Record run URLs/outcomes in `docs/project_management/next/world_process_exec_tracing_parity/session_log.md`.
