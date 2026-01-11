# Kickoff: DS0-test (Doctor scope split — test)

## Scope
- Tests only; no production code.
- ADR: `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`
- Spec: `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dsc-ds0-test` on branch `dsc-ds0-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/_archived/doctor_scopes/plan.md`, `docs/project_management/_archived/doctor_scopes/tasks.json`, `docs/project_management/_archived/doctor_scopes/session_log.md`, ADR, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/doctor_scopes" SLICE_ID="DS0"`

## Requirements

Add/modify tests so the DS0 acceptance criteria are enforced:
- CLI wiring tests for the new `host doctor` surface.
- JSON schema tests for `HostDoctorEnvelopeV1` and `WorldDoctorEnvelopeV1`.
- `agent-api-types` schema round-trip for `WorldDoctorReportV1`.
- Update internal consumers’ tests/fixtures that parse world doctor JSON (health/shim snapshots, world verify).

## Commands (required)
- `cargo fmt`
- Targeted test runs for any suites you touch/add.

## End Checklist

1. Run required commands and targeted tests; capture outputs in your task notes.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DS0-test"`.
3. On the orchestration branch, update `docs/project_management/_archived/doctor_scopes/tasks.json` and add the END entry to `docs/project_management/_archived/doctor_scopes/session_log.md`; commit docs (`docs: finish DS0-test`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

