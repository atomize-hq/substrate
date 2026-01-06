# Kickoff: WO0-integ-linux (integration platform-fix: linux)

## Scope
- Validate Feature Smoke on Linux for WO0, and apply fixes only if Linux smoke fails.
- Spec: `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/woe-wo0-integ-linux` on branch `woe-wo0-integ-linux` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" TASK_ID="WO0-integ-linux" TASK_PLATFORM=linux LAUNCH_CODEX=1`

## Requirements
- Before validating smoke or making fixes, merge the core integration branch into this worktree:
  - `git merge --no-edit woe-wo0-integ-core`
- Run platform-local Rust quality gates before finishing:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Validate Linux smoke via CI (repeat until green if you make fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-overlayfs-enumeration" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Ensure smoke is green for linux and capture the run id/URL in your final message.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-integ-linux"`.
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).

