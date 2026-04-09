# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the boundary checkpoint defined in `pre-planning/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout.

## Start Checklist
1. Ensure you are on `feat/substrate-gateway-boundary-and-runtime-ownership`.
2. Read `pre-planning/ci_checkpoint_plan.md`, `tasks.json`, `session_log.md`, and `quality_gate_report.md`.
3. Compute the checkpoint checkout SHA from `SGBRO4-integ-core`.

## Required gates
- Compile parity for the boundary slice.
- Feature smoke for `linux`, `macos`, and `windows`.
- Planning validators if the prompt/spec tree changed.

## End Checklist
1. Record run ids and URLs in `session_log.md`.
2. Mark the task completed in `tasks.json`.
3. Do not start final boundary aggregation until the checkpoint is green.
