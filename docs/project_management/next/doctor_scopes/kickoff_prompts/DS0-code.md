# Kickoff: DS0-code (Doctor scope split — code)

## Scope
- Production code only; no tests.
- ADR: `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`
- Spec: `docs/project_management/next/doctor_scopes/DS0-spec.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dsc-ds0-code` on branch `dsc-ds0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/doctor_scopes/plan.md`, `docs/project_management/next/doctor_scopes/tasks.json`, `docs/project_management/next/doctor_scopes/session_log.md`, ADR, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/doctor_scopes" SLICE_ID="DS0"`

## Requirements

- Implement the DS0 production code changes required by `docs/project_management/next/doctor_scopes/DS0-spec.md`:
  - Add `substrate host doctor` CLI surface.
  - Add world-agent endpoint `GET /v1/doctor/world`.
  - Update `substrate world doctor` to consume the agent endpoint and emit the new JSON envelope schema.
- Maintain the “doctor is passive” contract:
  - No provisioning side effects.
  - No agent spawning.
  - No VM start.

## Commands (required)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist

1. Run required commands and capture outputs in your task notes.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DS0-code"`.
3. On the orchestration branch, update `docs/project_management/next/doctor_scopes/tasks.json` and add the END entry to `docs/project_management/next/doctor_scopes/session_log.md`; commit docs (`docs: finish DS0-code`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

