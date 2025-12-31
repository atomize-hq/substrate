# Kickoff: C0-integ-windows (integration platform-fix — windows)

## Scope
- Ensure the slice behaves correctly on **windows**.
- This task is allowed to make production-code and/or test changes as needed to achieve cross-platform parity, but must not edit planning docs inside the worktree.
- Spec: `docs/project_management/next/tmp-make-scaffold/C0-spec.md`
- This task must not merge back to the orchestration branch; the final aggregator integration task performs the merge once all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: **windows**.
2. Verify you are in the task worktree `wt/tmp-make-scaffold-c0-integ-windows` on branch `tmp-make-scaffold-c0-integ-windows` and that `.taskmeta.json` exists at the worktree root.
3. Read: plan.md, tasks.json, session_log.md, spec, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" TASK_ID="C0-integ-windows" TASK_PLATFORM=windows`

## Requirements
- Validate platform smoke via CI for this platform (repeat until green if you make fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/tmp-make-scaffold" PLATFORM=windows WORKFLOW_REF="feat/tmp-make-scaffold"`
- If smoke passes: record run id/URL in the END entry and do not change code.
- If smoke fails:
  1) Fix the issue in this worktree (platform-specific guards, path handling, deps) while keeping the spec contract intact.
  2) Run the appropriate local checks for your change (fmt/clippy and targeted tests).
  3) Re-run the CI smoke for this platform until green.

## End Checklist
1. Ensure smoke is green for windows and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-windows"`
3. On the orchestration branch, update tasks.json + session_log.md END entry; commit docs (`docs: finish C0-integ-windows`).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
