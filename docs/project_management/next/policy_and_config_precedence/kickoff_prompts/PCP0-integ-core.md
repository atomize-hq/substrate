# PCP0-integ-core Kickoff — Workspace Config Precedence Over Env (integration core)

You are the integration agent for `PCP0-integ-core`.

Scope:
- Merge `PCP0-code` + `PCP0-test` and make the slice green on the primary dev platform.

Non-negotiable rule:
- Do not edit planning docs inside the worktree.

Required reading (end-to-end):
- `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
- `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- `docs/project_management/next/policy_and_config_precedence/manual_testing_playbook.md`
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`

Start checklist:
1. Ensure the orchestration branch is checked out:
   - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"`
2. Confirm `PCP0-code` and `PCP0-test` are completed (and merged to the orchestration branch).
3. Update `docs/project_management/next/policy_and_config_precedence/tasks.json`:
   - set `PCP0-integ-core.status` to `in_progress`
4. Append a START entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: start PCP0-integ-core`)
5. Create the worktree via automation:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" TASK_ID="PCP0-integ-core"`
6. Enter the worktree: `cd wt/pcp0-precedence-integ-core`
7. Do not edit planning docs inside the worktree.

Required commands:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Run relevant `cargo test` suites for affected areas
- `make integ-checks`

Cross-platform smoke (CI):
- Dispatch cross-platform smoke via GitHub Actions:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy_and_config_precedence"`
- If any platform fails, start only the failing platform-fix tasks from the emitted `RUN_ID`:
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" SLICE_ID="PCP0" SMOKE_RUN_ID="<run-id>"`
- After all required platforms are green, start `PCP0-integ` (integration final):
  - `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" SLICE_ID="PCP0"`

End checklist:
1. Run required commands and ensure they pass.
2. From inside the worktree: `make triad-task-finish TASK_ID="PCP0-integ-core"`.
3. On the orchestration branch: update `tasks.json` + append an END entry to `session_log.md`; commit docs (`docs: finish PCP0-integ-core`).
