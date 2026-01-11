# Kickoff: WO0-integ-core (integration core)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/_archived/world-overlayfs-enumeration/WO0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/woe-wo0-integ-core` on branch `woe-wo0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/_archived/world-overlayfs-enumeration/plan.md`, `docs/project_management/_archived/world-overlayfs-enumeration/tasks.json`, `docs/project_management/_archived/world-overlayfs-enumeration/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration" TASK_ID="WO0-integ-core"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Merge code+test branches into this worktree, then run required integration gates (must be green before any CI smoke dispatch):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### Cross-platform compile parity (CI dispatch; required before smoke)

Before dispatching Feature Smoke, run a fast cross-platform compile parity preflight on GitHub-hosted runners to catch macOS/Windows compilation breaks early:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/world-overlayfs-enumeration" CI_REMOTE=origin CI_CLEANUP=1`

If it fails, fix compile parity in this integ-core worktree/branch, commit, and re-run until green; do not proceed to Feature Smoke until it is green.

### Cross-platform smoke (CI dispatch; validation-only)

Run CI smoke from this integration-core worktree, because the smoke dispatcher tests the current `HEAD` by creating a throwaway remote branch at that commit:
- `make feature-smoke FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-overlayfs-enumeration" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

If any platform smoke fails:
- Do not attempt platform-specific fixes in integ-core.
- Ask the operator to start only the failing platform-fix tasks from the orchestration checkout:
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="docs/project_management/_archived/world-overlayfs-enumeration" SLICE_ID="WO0" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="WO0-integ-core"`
2. Hand off smoke run id/URL and any notes to the operator (do not edit planning docs inside the worktree).

