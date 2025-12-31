# Kickoff: C0-integ-core (E2E smoke)

Do not edit planning docs inside the worktree.

Goal:
- Merge C0-code + C0-test branches, make the slice green, and dispatch cross-platform smoke via CI.

Steps:
1) Merge the task branches:
   - Merge the code branch and the test branch into this worktree.
2) Run required checks:
   - `make integ-checks`
3) Dispatch smoke:
   - `make feature-smoke FEATURE_DIR="docs/project_management/next/e2e-triad-smoke-20251231T141542Z" PLATFORM=all`
4) If any platform smoke fails, start only failing platform-fix tasks:
   - `make triad-task-start-platform-fixes FEATURE_DIR="docs/project_management/next/e2e-triad-smoke-20251231T141542Z" SLICE_ID="C0" PLATFORMS="linux,macos,windows"`
5) After all failing platforms are green, start the final aggregator:
   - `make triad-task-start-integ-final FEATURE_DIR="docs/project_management/next/e2e-triad-smoke-20251231T141542Z" SLICE_ID="C0"`

Finish:
- From inside this worktree run: `make triad-task-finish TASK_ID="C0-integ-core"`
