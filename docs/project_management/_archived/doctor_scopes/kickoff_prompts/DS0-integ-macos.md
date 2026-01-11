# Kickoff: DS0-integ-macos (integration platform-fix — macos)

## Scope
- Fix macOS-only regressions surfaced by Feature Smoke for this slice.
- Spec: `docs/project_management/_archived/doctor_scopes/DS0-spec.md`
- Smoke script: `docs/project_management/_archived/doctor_scopes/smoke/macos-smoke.sh`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dsc-ds0-integ-macos` on branch `dsc-ds0-integ-macos` and that `.taskmeta.json` exists at the worktree root.
2. Read: spec, manual playbook, smoke script, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/doctor_scopes" TASK_ID="DS0-integ-macos"`

## Requirements

- Make macOS Feature Smoke green for this slice:
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/doctor_scopes" PLATFORM=macos RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- When fixing:
  - keep behavior aligned to `DS0-spec.md`;
  - add minimal tests only when required to prevent regression.

## End Checklist

1. Ensure macOS smoke is green; record `RUN_ID`/`RUN_URL` in the session log END entry for this task.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DS0-integ-macos"`.
3. Do not delete the worktree (feature cleanup removes worktrees at feature end).

