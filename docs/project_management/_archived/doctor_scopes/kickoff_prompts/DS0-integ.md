# Kickoff: DS0-integ (integration final)

## Scope
- Final integration for DS0: merge platform-fix branches, confirm cross-platform green, and complete the DS0 closeout gate report.
- Spec: `docs/project_management/_archived/doctor_scopes/DS0-spec.md`
- Closeout report: `docs/project_management/_archived/doctor_scopes/DS0-closeout_report.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dsc-ds0-integ` on branch `dsc-ds0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: spec, manual playbook, smoke scripts, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/_archived/doctor_scopes" SLICE_ID="DS0" LAUNCH_CODEX=1`

## Requirements

- Merge any platform-fix branches for DS0 and ensure local gates are green:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Re-run CI compile parity (linux/macos/windows):
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/doctor-scopes" CI_REMOTE=origin CI_CLEANUP=1`
- Re-run behavioral smoke for behavior platforms:
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/doctor_scopes" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/doctor_scopes" PLATFORM=macos RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- Fill `docs/project_management/_archived/doctor_scopes/DS0-closeout_report.md` with evidence (run ids/URLs and gate results).

## End Checklist

1. Ensure all required CI is green and recorded in the closeout report.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="DS0-integ"`.
3. On the orchestration branch: mark DS0 tasks completed and add session log END entries; commit docs.
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

