# Kickoff: C0-integ (E2E smoke final aggregator)

Do not edit planning docs inside the worktree.

Goal:
- Merge C0-integ-core and any platform-fix branches, re-run checks + smoke, and fast-forward merge back to orchestration.

Required:
- `make integ-checks`
- `make feature-smoke FEATURE_DIR="docs/project_management/next/e2e-triad-smoke-20251231T140022Z" PLATFORM=all`

Finish:
- From inside this worktree run: `make triad-task-finish TASK_ID="C0-integ"`
