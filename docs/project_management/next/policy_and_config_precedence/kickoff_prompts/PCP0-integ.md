# Kickoff: PCP0-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize PCP0 with a clean, auditable cross-platform green state.
- Spec: `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task is responsible for the fast-forward merge back to the orchestration branch after all required platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/pcp0-precedence-integ` on branch `pcp-pcp0-precedence-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read (end-to-end): `plan.md`, `tasks.json`, `session_log.md`, `PCP0-spec.md`, `PCP0-closeout_report.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" TASK_ID="PCP0-integ" LAUNCH_CODEX=1`

## Requirements
- Merge the relevant integration branches for this slice:
  - `pcp-pcp0-precedence-integ-core`
  - any platform-fix branches that produced commits:
    - `pcp-pcp0-precedence-integ-linux`
    - `pcp-pcp0-precedence-integ-macos`
    - `pcp-pcp0-precedence-integ-windows`
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Re-run cross-platform smoke via CI to confirm the merged result is green:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy_and_config_precedence"`
- Complete the slice closeout gate report:
  - `docs/project_management/next/policy_and_config_precedence/PCP0-closeout_report.md`

## End Checklist
1. Ensure all required platforms are green (capture run ids/URLs).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="PCP0-integ"`.
3. Hand off run ids/URLs and closeout report completion to the operator (do not edit planning docs inside the worktree).
