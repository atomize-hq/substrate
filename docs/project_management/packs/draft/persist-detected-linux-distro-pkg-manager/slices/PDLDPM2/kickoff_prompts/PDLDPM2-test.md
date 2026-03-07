# Kickoff: PDLDPM2-test (test)

## Scope
- Tests only; no production code changes.
- Spec: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`
- Workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-test` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-test` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `platform-parity-spec.md`, `pre-planning/ci_checkpoint_plan.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" SLICE_ID="PDLDPM2"`.

## Requirements
- Add or adjust tests that enforce AC-PDLDPM2-01 through AC-PDLDPM2-08.
- Cover metadata, cleanup, missing, prod-no-world, and dev smoke ownership exactly as the slice spec assigns it.
- Run:
  - `cargo fmt`
  - the targeted tests you add or touch

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="PDLDPM2-test"`.
3. Hand off the targeted test commands and outcomes to the operator.
