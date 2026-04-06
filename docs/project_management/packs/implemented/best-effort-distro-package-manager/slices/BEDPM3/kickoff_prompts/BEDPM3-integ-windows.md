# Kickoff: BEDPM3-integ-windows (integration platform-fix — windows)

## Scope
- Resolve Windows compile or test parity failures for BEDPM3 after `CP1-ci-checkpoint`.
- Spec: `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a Windows host.
2. Verify you are in `wt/best-effort-distro-package-manager-bedpm3-integ-windows` on branch `best-effort-distro-package-manager-bedpm3-integ-windows` and that `.taskmeta.json` exists.
3. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM3 spec, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/implemented/best-effort-distro-package-manager" TASK_ID="BEDPM3-integ-windows" TASK_PLATFORM=windows`.

## Requirements
- Merge `BEDPM3-integ-core` into this worktree before fixing anything.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and the relevant tests for the failing Windows path.
- This task is parity-only. Do not dispatch feature smoke from this task.

## End Checklist
1. Capture the commands and results.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM3-integ-windows"`.
3. Hand off results and ask the operator to rerun `CP1-ci-checkpoint` when parity needs confirmation.
