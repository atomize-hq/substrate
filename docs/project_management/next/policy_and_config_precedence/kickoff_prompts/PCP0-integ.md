# PCP0-integ Kickoff — Workspace Config Precedence Over Env (integration)

You are the integration agent for `PCP0-integ`.

Scope:
- Merge `PCP0-code` + `PCP0-test`, reconcile to `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`, and validate behavior.

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
1. On the orchestration branch: `git checkout feat/policy_and_config_precedence && git pull --ff-only`
2. Confirm `F0-exec-preflight` is completed (execution gates are enabled for this feature).
3. Confirm `PCP0-code` and `PCP0-test` are completed and merged to the orchestration branch.
4. Update `docs/project_management/next/policy_and_config_precedence/tasks.json`:
   - set `PCP0-integ.status` to `in_progress`
5. Append a START entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: start PCP0-integ`)
6. Create a task branch and worktree:
   - `git checkout -b pcp-pcp0-precedence-integ`
   - `git worktree add wt/pcp0-precedence-integ -b pcp-pcp0-precedence-integ`
7. Enter the worktree: `cd wt/pcp0-precedence-integ`
8. Do not edit planning docs inside the worktree.

Required commands:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Run the relevant `cargo test` suites for affected areas
- `make integ-checks`

Validation requirements:
- Run the platform-local smoke script:
  - Linux: `bash docs/project_management/next/policy_and_config_precedence/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/next/policy_and_config_precedence/smoke/macos-smoke.sh`
  - Windows: `pwsh -File docs/project_management/next/policy_and_config_precedence/smoke/windows-smoke.ps1`
- Dispatch cross-platform smoke via GitHub Actions:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=all WORKFLOW_REF="feat/policy_and_config_precedence"`
- Record results (including run URLs/ids) in the END entry for `docs/project_management/next/policy_and_config_precedence/session_log.md`.

End checklist:
1. Reconcile code+tests to `PCP0-spec.md` and ensure the contract matches ADR-0005.
2. Run required commands and ensure they pass.
3. Run smoke validation and record results.
4. Fill `docs/project_management/next/policy_and_config_precedence/PCP0-closeout_report.md` with evidence (required end gate for PCP0).
5. Commit changes in the worktree to `pcp-pcp0-precedence-integ`.
6. On the orchestration branch, merge/fast-forward the task branch.
7. Update `docs/project_management/next/policy_and_config_precedence/tasks.json` to `completed` and append an END entry to `docs/project_management/next/policy_and_config_precedence/session_log.md` (include the closeout report status); commit docs (`docs: finish PCP0-integ`).
