# Kickoff: PCP0-integ-linux (integration platform-fix — linux)

## Scope
- Ensure PCP0 is green on Linux (no-op if already green).
- Spec: `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Linux machine.
2. Verify you are in the task worktree `wt/pcp0-precedence-integ-linux` on branch `pcp-pcp0-precedence-integ-linux` and that `.taskmeta.json` exists at the worktree root.
3. Read (end-to-end): `plan.md`, `tasks.json`, `session_log.md`, `PCP0-spec.md`, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" TASK_ID="PCP0-integ-linux" TASK_PLATFORM=linux LAUNCH_CODEX=1`

## Requirements
- Validate Linux smoke via CI (repeat until green if you make fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy_and_config_precedence"`
- If smoke passes: record the run id/URL for the operator and do not change code.
- If smoke fails:
  1) Fix the issue in this worktree while keeping the spec contract intact.
  2) Run the appropriate local checks for your change (fmt/clippy and targeted tests).
  3) Re-run the Linux CI smoke until green.

## End Checklist
1. Ensure Linux smoke is green; capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="PCP0-integ-linux"`.
3. Hand off run id/URL and any Linux-specific notes to the operator (do not edit planning docs inside the worktree).
