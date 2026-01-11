# Kickoff: WO0-integ-macos (integration platform-fix — macos)

## Scope
- Ensure the slice behaves correctly on macos.
- Spec: `docs/project_management/_archived/world-overlayfs-enumeration/WO0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: macos.
2. Verify you are in the task worktree `wt/woe-wo0-integ-macos` on branch `woe-wo0-integ-macos` and that `.taskmeta.json` exists at the worktree root.
3. Read: `docs/project_management/_archived/world-overlayfs-enumeration/plan.md`, `docs/project_management/_archived/world-overlayfs-enumeration/tasks.json`, `docs/project_management/_archived/world-overlayfs-enumeration/session_log.md`, spec, this prompt.

## Requirements
- Before validating smoke or making fixes, merge the slice’s core integration branch into this worktree.
- Run the platform-local Rust quality gates before finishing:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Validate platform smoke via CI (repeat until green if you make fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration" PLATFORM=macos RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-overlayfs-enumeration" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Ensure smoke is green for macos and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-integ-macos"`
3. Hand off run id/URL and any platform-specific notes to the operator (do not edit planning docs inside the worktree).

