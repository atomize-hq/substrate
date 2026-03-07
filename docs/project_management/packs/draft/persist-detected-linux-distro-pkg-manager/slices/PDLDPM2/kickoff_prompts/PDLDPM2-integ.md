# Kickoff: PDLDPM2-integ (integration final - cross-platform merge)

## Scope
- Merge the PDLDPM2 core and platform-fix branches and finalize the checkpoint-boundary slice.
- Spec: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`
- Workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-integ` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-integ` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `platform-parity-spec.md`, `pre-planning/ci_checkpoint_plan.md`, the CP1 results, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM2-integ"`.

## Requirements
- Merge `PDLDPM2-integ-core`, `PDLDPM2-integ-linux`, `PDLDPM2-integ-macos`, and `PDLDPM2-integ-windows`.
- Confirm `CP1-ci-checkpoint` is completed before finishing this task.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

## End Checklist
1. Ensure the merged state is committed and local integration gates are green.
2. From inside the worktree, run `make triad-task-finish TASK_ID="PDLDPM2-integ"`.
3. Hand off the final slice merge status to the operator.
