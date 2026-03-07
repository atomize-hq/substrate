# Kickoff: PDLDPM1-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
- Workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/persist-detected-linux-distro-pkg-manager-pdldpm1-code` on branch `persist-detected-linux-distro-pkg-manager-pdldpm1-code` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `compatibility-spec.md`, `contract.md`, `decision_register.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" SLICE_ID="PDLDPM1"`.

## Requirements
- Implement AC-PDLDPM1-01 through AC-PDLDPM1-07 exactly.
- Keep `--dry-run` non-mutating, keep `--no-world` persistent on Linux, and keep metadata failures warning-only.
- Run a targeted baseline command before changes, then rerun it after changes.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="PDLDPM1-code"`.
3. Hand off the baseline command and outcomes to the operator.
