# Kickoff: C0-integ-windows (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Confirm the merged slice is green on windows and fix if needed.

Steps:
1) Merge C0-integ-core into this branch.
2) Run CI smoke for windows until green:
   - `make feature-smoke FEATURE_DIR="docs/project_management/next/e2e-triad-smoke-20251231T140805Z" PLATFORM=windows`
3) Finish (commits to this task branch; does not merge back to orchestration):
   - `make triad-task-finish TASK_ID="C0-integ-windows" SMOKE=1 TASK_PLATFORM=windows`
