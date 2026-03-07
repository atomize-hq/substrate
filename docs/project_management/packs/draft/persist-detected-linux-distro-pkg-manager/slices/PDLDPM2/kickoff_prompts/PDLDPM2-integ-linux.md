# Kickoff: PDLDPM2-integ-linux (integration platform-fix - linux)

## Scope
- Resolve Linux-specific failures reported by `CP1-ci-checkpoint` for PDLDPM2.
- This task may adjust production code and tests as needed for Linux green state, but it must not edit planning docs inside the worktree.
- Spec: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on Linux.
2. Verify you are in `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-integ-linux` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-integ-linux` and `.taskmeta.json` exists.
3. Read `plan.md`, `tasks.json`, `session_log.md`, `platform-parity-spec.md`, `pre-planning/ci_checkpoint_plan.md`, the CP1 results, the slice spec, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM2-integ-linux"`.

## Requirements
- Merge `PDLDPM2-integ-core` into this worktree before fixing Linux failures.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Re-run the failing Linux command set until it passes.

## End Checklist
1. Ensure Linux failures are resolved and capture the rerun evidence.
2. From inside the worktree, run `make triad-task-finish TASK_ID="PDLDPM2-integ-linux"`.
3. Hand off the Linux-specific results to the operator.
