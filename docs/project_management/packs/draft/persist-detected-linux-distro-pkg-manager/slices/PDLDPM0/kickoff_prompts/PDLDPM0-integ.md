# Kickoff: PDLDPM0-integ (integration)

## Scope
- Merge code and tests, resolve drift to the spec, and make PDLDPM0 green.
- Spec: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
- Workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/persist-detected-linux-distro-pkg-manager-pdldpm0-integ` on branch `persist-detected-linux-distro-pkg-manager-pdldpm0-integ` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `install-state-schema-spec.md`, `contract.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM0-integ"`.

## Requirements
- Merge `PDLDPM0-code` and `PDLDPM0-test`, then reconcile code and tests to the spec.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

## End Checklist
1. Ensure the merged state is committed and local integration gates are green.
2. From inside the worktree, run `make triad-task-finish TASK_ID="PDLDPM0-integ"`.
3. Hand off any remaining notes to the operator.
