# Kickoff: WO0-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize WO0 with a clean, auditable cross-platform green state.
- Spec: `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- Closeout gate report: `docs/project_management/next/world-overlayfs-enumeration/WO0-closeout_report.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/woe-wo0-integ` on branch `woe-wo0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan.md, tasks.json, session_log.md, ADR, spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" TASK_ID="WO0-integ"`
4. Merge the integration branches for this slice:
   - `git merge --no-edit woe-wo0-integ-core`
   - `git merge --no-edit woe-wo0-integ-linux` (if it exists and has commits)
   - `git merge --no-edit woe-wo0-integ-macos` (if it exists and has commits)
   - `git merge --no-edit woe-wo0-integ-windows` (if it exists and has commits)

## Requirements
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- relevant tests
- `make integ-checks`

### Cross-platform smoke (CI dispatch; validation-only)
- Run from this final integration worktree (smoke validates current `HEAD` via a throwaway remote branch).
- Dispatch:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world-overlayfs-enumeration" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-overlayfs-enumeration" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Run required commands and capture outputs (including smoke results).
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-integ"`.
3. In your final message, include the `make feature-smoke PLATFORM=all` output keys:
   - `RUN_ID=<id>`
   - `RUN_URL=<url>`
   - `SMOKE_PASSED_PLATFORMS=<csv>`
   - `SMOKE_FAILED_PLATFORMS=<csv>`
4. Hand off any notes to the operator (do not edit planning docs inside the worktree).
