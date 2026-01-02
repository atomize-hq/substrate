# PCP0-integ Kickoff — Workspace Config Precedence Over Env (integration final)

You are the integration agent for `PCP0-integ` (final aggregator).

Scope:
- Merge any platform fixes, reconcile to `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`, and confirm all required platforms are green.

Non-negotiable rule:
- Do not edit planning docs inside the worktree.

Required reading (end-to-end):
- `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
- `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- `docs/project_management/next/policy_and_config_precedence/PCP0-closeout_report.md`
- `docs/project_management/next/policy_and_config_precedence/manual_testing_playbook.md`
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`

Start checklist:
1. On the orchestration branch: `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"`
2. Confirm `F0-exec-preflight` is completed (execution gates are enabled for this feature).
3. Confirm `PCP0-integ-core` is completed, and `PCP0-integ-{linux,macos,windows}` are completed for the required platforms.
4. Update `docs/project_management/next/policy_and_config_precedence/tasks.json`:
   - set `PCP0-integ.status` to `in_progress`
5. Append a START entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: start PCP0-integ`)
6. Create the worktree via automation:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" TASK_ID="PCP0-integ"`
7. Enter the worktree: `cd wt/pcp0-precedence-integ`
8. Do not edit planning docs inside the worktree.

Required commands:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Run the relevant `cargo test` suites for affected areas
- `make integ-checks`

Validation requirements:
- Re-run cross-platform smoke via GitHub Actions:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy_and_config_precedence"`
- Record results (including run URLs/ids) in the END entry for `docs/project_management/next/policy_and_config_precedence/session_log.md`.

End checklist:
1. Reconcile code+tests to `PCP0-spec.md` and ensure the contract matches ADR-0005.
2. Merge platform-fix branches (if any) and resolve drift.
3. Run required commands and ensure they pass.
4. Run cross-platform smoke and record results.
5. Fill `docs/project_management/next/policy_and_config_precedence/PCP0-closeout_report.md` with evidence (required end gate for PCP0).
6. From inside the worktree: `make triad-task-finish TASK_ID="PCP0-integ"`.
7. On the orchestration branch: update `tasks.json` to `completed` and append an END entry to `session_log.md` (include the closeout report status); commit docs (`docs: finish PCP0-integ`).
