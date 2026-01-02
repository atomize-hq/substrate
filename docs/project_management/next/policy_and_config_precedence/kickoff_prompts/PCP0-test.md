# PCP0-test Kickoff — Workspace Config Precedence Over Env (tests only)

You are the test agent for `PCP0-test`.

Scope:
- Update/add tests per `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`.
- Tests only. Do not write or modify production code.

Non-negotiable rule:
- Do not edit planning docs inside the worktree.

Required reading (end-to-end):
- `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- `docs/project_management/next/policy_and_config_precedence/decision_register.md`

Start checklist:
1. On the orchestration branch: `git checkout feat/policy_and_config_precedence && git pull --ff-only`
2. Update `docs/project_management/next/policy_and_config_precedence/tasks.json`:
   - set `PCP0-test.status` to `in_progress`
3. Append a START entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: start PCP0-test`)
4. Create a task branch and worktree:
   - `git checkout -b pcp-pcp0-precedence-test`
   - `git worktree add wt/pcp0-precedence-test -b pcp-pcp0-precedence-test`
5. Enter the worktree: `cd wt/pcp0-precedence-test`
6. Do not edit planning docs inside the worktree.

Test requirements:
- Update `crates/shell/tests/config_show.rs` precedence assertions to match ADR-0005.
- Preserve protected excludes assertions.

Required commands:
- `cargo fmt`
- Targeted `cargo test` commands for modified tests.

End checklist:
1. Run required commands and ensure they pass.
2. Commit changes in the worktree to `pcp-pcp0-precedence-test`.
3. On the orchestration branch, merge/fast-forward the task branch.
4. Update `docs/project_management/next/policy_and_config_precedence/tasks.json` to `completed` and append an END entry to `docs/project_management/next/policy_and_config_precedence/session_log.md`; commit docs (`docs: finish PCP0-test`).

