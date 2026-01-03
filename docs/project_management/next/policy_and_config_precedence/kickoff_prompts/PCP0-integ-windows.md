# Kickoff: PCP0-integ-windows (integration platform-fix — windows)

## Scope
- Ensure PCP0 is green on Windows (no-op if already green).
- Spec: `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Windows machine.
2. Verify you are in the task worktree `wt/pcp0-precedence-integ-windows` on branch `pcp-pcp0-precedence-integ-windows` and that `.taskmeta.json` exists at the worktree root.
   - Do all work (edits, builds/tests, commits, and `make triad-task-finish`) from inside this worktree.
3. Read (end-to-end): `plan.md`, `tasks.json`, `session_log.md`, `PCP0-spec.md`, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" TASK_ID="PCP0-integ-windows" TASK_PLATFORM=windows LAUNCH_CODEX=1`

## Requirements
- Before validating smoke or making fixes, merge the slice’s core integration branch into this worktree:
  - Merge `pcp-pcp0-precedence-integ-core` into `pcp-pcp0-precedence-integ-windows`.
- Run platform-local Rust quality gates before finishing (CI Testing parity on Windows):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Validate Windows smoke via CI (repeat until green if you make fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/policy_and_config_precedence" PLATFORM=windows RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy_and_config_precedence" REMOTE=origin CLEANUP=1`
- If smoke passes: record the run id/URL for the operator and do not change code.
- If smoke fails:
  1) Fix the issue in this worktree while keeping the spec contract intact.
  2) Run the appropriate local checks for your change (fmt/clippy and targeted tests).
  3) Re-run the Windows CI smoke until green.

## End Checklist
1. Ensure Windows smoke is green; capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="PCP0-integ-windows"`.
3. Hand off run id/URL and any Windows-specific notes to the operator (do not edit planning docs inside the worktree).
