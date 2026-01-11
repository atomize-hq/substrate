# Kickoff: DS0-integ-core (integration core)

## Scope
- Merge DS0 code + tests, resolve drift to the spec (spec wins), and make the slice green on the primary dev platform.
- Spec: `docs/project_management/_archived/doctor_scopes/DS0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/dsc-ds0-integ-core` on branch `dsc-ds0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/_archived/doctor_scopes/plan.md`, `docs/project_management/_archived/doctor_scopes/tasks.json`, `docs/project_management/_archived/doctor_scopes/session_log.md`, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/doctor_scopes" TASK_ID="DS0-integ-core"`

## Requirements

- Merge `dsc-ds0-code` and `dsc-ds0-test` into this branch/worktree and reconcile to spec.
- Local integration gates MUST be green before any smoke dispatch:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Dispatch CI compile parity (GitHub-hosted runners) for fast cross-platform feedback:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/doctor-scopes" CI_REMOTE=origin CI_CLEANUP=1`
- Dispatch Feature Smoke for behavior platforms:
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/doctor_scopes" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/doctor_scopes" PLATFORM=macos RUNNER_KIND=self-hosted WORKFLOW_REF="feat/doctor-scopes" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- If smoke fails on a behavior platform:
  - Do not fix platform-specific issues in integ-core.
  - Start the corresponding platform-fix task(s) from the orchestration checkout.

## End Checklist

1. Ensure local integration gates are green and committed.
2. Record CI parity run id/URL and smoke run id/URL(s) in the session log END entry for this task.
3. From inside the worktree, run: `make triad-task-finish TASK_ID="DS0-integ-core"`.
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

