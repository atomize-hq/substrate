# PCP0-code Kickoff — Workspace Config Precedence Over Env (production code only)

You are the code agent for `PCP0-code`.

Scope:
- Implement the precedence change defined in `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`.
- Production code only. Do not write or modify tests.

Non-negotiable rule:
- Do not edit planning docs inside the worktree.

Required reading (end-to-end):
- `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
- `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- `docs/project_management/next/policy_and_config_precedence/decision_register.md`

Start checklist:
1. On the orchestration branch: `git checkout feat/policy_and_config_precedence && git pull --ff-only`
2. Confirm `F0-exec-preflight` is completed (execution gates are enabled for this feature).
3. Update `docs/project_management/next/policy_and_config_precedence/tasks.json`:
   - set `PCP0-code.status` to `in_progress`
4. Append a START entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: start PCP0-code`)
5. Create a task branch and worktree:
   - `git checkout -b pcp-pcp0-precedence-code`
   - `git worktree add wt/pcp0-precedence-code -b pcp-pcp0-precedence-code`
6. Enter the worktree: `cd wt/pcp0-precedence-code`
7. Do not edit planning docs inside the worktree.

Implementation requirements:
- Update the effective-config resolver so that, when a workspace exists, workspace config overrides `SUBSTRATE_*` env vars.
- Preserve strict parsing, legacy `.substrate/settings.yaml` hard error behavior, and protected sync excludes behavior.

Required commands:
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

End checklist:
1. Run the required commands and ensure they pass.
2. Commit changes in the worktree to `pcp-pcp0-precedence-code`.
3. On the orchestration branch, merge/fast-forward the task branch.
4. Update `docs/project_management/next/policy_and_config_precedence/tasks.json` to `completed` and append an END entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: finish PCP0-code`).
