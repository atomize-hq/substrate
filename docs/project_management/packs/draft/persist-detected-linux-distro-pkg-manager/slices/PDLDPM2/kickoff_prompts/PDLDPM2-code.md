# Kickoff: PDLDPM2-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`
- Workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-code` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-code` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `platform-parity-spec.md`, `pre-planning/ci_checkpoint_plan.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" SLICE_ID="PDLDPM2"`.

## Requirements
- Implement AC-PDLDPM2-01 through AC-PDLDPM2-08 exactly.
- Keep Linux as the only behavior-delta platform and leave macOS and Windows as parity-only platforms.
- Run a targeted baseline command before changes, then rerun it after changes.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`

## End Checklist
1. Run the required commands and capture outputs.
2. From inside the worktree, run `make triad-task-finish TASK_ID="PDLDPM2-code"`.
3. Hand off the baseline command and outcomes to the operator.
