# Kickoff: BEDPM3-integ (integration final)

## Scope
- Merge BEDPM3 platform-fix branches, reconcile to spec, and finish the checkpoint-boundary slice.
- Spec: `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/best-effort-distro-package-manager-bedpm3-integ` on branch `best-effort-distro-package-manager-bedpm3-integ` and that `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM3 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/implemented/best-effort-distro-package-manager" TASK_ID="BEDPM3-integ"`.

## Requirements
- Merge the relevant BEDPM3 integration branches into this worktree:
  - `BEDPM3-integ-core`
  - `BEDPM3-integ-linux`
  - `BEDPM3-integ-macos`
  - `BEDPM3-integ-windows`
- Confirm `CP1-ci-checkpoint` completed before finishing this task.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.

## End Checklist
1. Capture the commands and results.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM3-integ"`.
3. Hand off results to the operator. Do not edit planning docs inside the worktree.
