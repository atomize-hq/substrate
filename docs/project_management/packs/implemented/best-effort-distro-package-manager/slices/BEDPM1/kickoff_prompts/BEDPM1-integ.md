# Kickoff: BEDPM1-integ (integration)

## Scope
- Merge BEDPM1 code and tests, reconcile to spec, and make the slice green locally.
- Spec: `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/best-effort-distro-package-manager-bedpm1-integ` on branch `best-effort-distro-package-manager-bedpm1-integ` and that `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM1 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" TASK_ID="BEDPM1-integ"`.

## Requirements
- Merge the BEDPM1 code and test branches into this worktree. The spec wins on drift.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Do not dispatch cross-platform CI from this task. Cross-platform CI runs only at `CP1-ci-checkpoint`.

## End Checklist
1. Capture the commands and results.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM1-integ"`.
3. Hand off results to the operator. Do not edit planning docs inside the worktree.
