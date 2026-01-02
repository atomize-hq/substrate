# Kickoff: PCP0-integ-core (integration core) — Workspace Config Precedence Over Env

## Scope
- Merge code + tests, reconcile to spec, and make PCP0 green on the primary dev platform.
- Spec: `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/pcp0-precedence-integ-core` on branch `pcp-pcp0-precedence-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read (end-to-end): `plan.md`, `tasks.json`, `session_log.md`, `PCP0-spec.md`, `manual_testing_playbook.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" TASK_ID="PCP0-integ-core" LAUNCH_CODEX=1`

## Requirements
- Reconcile code/tests to `PCP0-spec.md` (spec wins).
- Merge the task branches for this slice into this worktree (resolve conflicts/drift):
  - `pcp-pcp0-precedence-code`
  - `pcp-pcp0-precedence-test`
- Run integration gates:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant `cargo test` suites for affected areas
  - `make integ-checks`
- Dispatch cross-platform smoke via CI:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy_and_config_precedence"`
  - If any platform fails, ask the operator to start only the failing platform-fix tasks from the emitted `RUN_ID`:
    - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" SLICE_ID="PCP0" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
  - After all required platforms are green, ask the operator to start `PCP0-integ` (integration final):
    - `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" SLICE_ID="PCP0" LAUNCH_CODEX=1`

## End Checklist
1. Ensure `make integ-checks` is green; capture key outputs and smoke run id/URL (if dispatched).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="PCP0-integ-core"`.
3. Hand off key outputs + smoke run ids/URLs to the operator (do not edit planning docs inside the worktree).
