# Kickoff: WO0-integ-core (integration core)

## Scope
- Merge `WO0-code` + `WO0-test`, reconcile drift to spec, and make WO0 green on the primary dev platform.
- Then dispatch cross-platform Feature Smoke (`PLATFORM=all`) to select any needed platform-fix tasks.
- Spec: `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/woe-wo0-integ-core` on branch `woe-wo0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, ADR, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" TASK_ID="WO0-integ-core" LAUNCH_CODEX=1`

## Requirements
- Merge the task branches:
  - `woe-wo0-code`
  - `woe-wo0-test`
- Run required integration gates and ensure they are green before dispatching smoke:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### Cross-platform smoke (CI dispatch; validation-only)
- Run from this **integration-core worktree**, because the smoke dispatcher tests the current `HEAD` by creating a throwaway remote branch at that commit.
- Dispatch:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-overlayfs-enumeration" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-integ-core"`.
2. In your final message, include these exact keys (one per line), populated from the `make feature-smoke PLATFORM=all` output:
   - `RUN_ID=<id>`
   - `RUN_URL=<url>`
   - `SMOKE_PASSED_PLATFORMS=<csv>`
   - `SMOKE_FAILED_PLATFORMS=<csv>`
3. Hand off any notes to the operator (do not edit planning docs inside the worktree).

