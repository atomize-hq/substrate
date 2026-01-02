# PCP0-integ-linux Kickoff — Workspace Config Precedence Over Env (Linux platform integ)

You are the integration agent for `PCP0-integ-linux`.

Scope:
- Ensure Linux smoke is green for PCP0. If smoke fails, apply the minimal Linux-specific fix (behind `#[cfg]` gates if needed), then re-run smoke.

Non-negotiable rule:
- Do not edit planning docs inside the worktree.

Start checklist:
1. Ensure the orchestration branch is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"`
2. Confirm `PCP0-integ-core` is completed.
3. Update `docs/project_management/next/policy_and_config_precedence/tasks.json`:
   - set `PCP0-integ-linux.status` to `in_progress`
4. Append a START entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: start PCP0-integ-linux`)
5. Create the worktree via automation:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" TASK_ID="PCP0-integ-linux"`
6. Enter the worktree: `cd wt/pcp0-precedence-integ-linux`
7. Do not edit planning docs inside the worktree.

Validation:
- Dispatch Linux smoke via CI:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy_and_config_precedence"`
- Record the run id/URL and outcome in the END entry.

End checklist:
1. If fixes were needed, run required formatting/lint/tests for touched areas.
2. From inside the worktree: `make triad-task-finish TASK_ID="PCP0-integ-linux"`.
3. On the orchestration branch: update `tasks.json` + append an END entry to `session_log.md`; commit docs (`docs: finish PCP0-integ-linux`).
