# Kickoff: DS0-integ-windows (integration platform-fix — windows CI parity only)

## Scope
- Fix Windows compilation/lint/test parity issues only.
- Windows is CI-parity-only for DS0; no behavioral smoke is required.
- Spec: `docs/project_management/next/doctor_scopes/DS0-spec.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dsc-ds0-integ-windows` on branch `dsc-ds0-integ-windows` and that `.taskmeta.json` exists at the worktree root.
2. Read: spec and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/doctor_scopes" TASK_ID="DS0-integ-windows"`

## Requirements

- Make CI compile parity green for this slice (GitHub-hosted runners):
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/doctor-scopes" CI_REMOTE=origin CI_CLEANUP=1`

## End Checklist

1. Record the CI parity run id/URL in the session log END entry for this task.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DS0-integ-windows"`.
3. Do not delete the worktree (feature cleanup removes worktrees at feature end).

