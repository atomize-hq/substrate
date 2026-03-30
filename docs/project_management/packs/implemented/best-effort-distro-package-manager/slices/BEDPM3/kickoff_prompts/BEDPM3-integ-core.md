# Kickoff: BEDPM3-integ-core (integration core)

## Scope
- Merge BEDPM3 code and tests, reconcile to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/best-effort-distro-package-manager-bedpm3-integ-core` on branch `best-effort-distro-package-manager-bedpm3-integ-core` and that `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the BEDPM3 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" TASK_ID="BEDPM3-integ-core"`.

## Requirements
- Merge the BEDPM3 code and test branches into this worktree. The spec wins on drift.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, `bash "docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh"`, and `make integ-checks`.
- Do not dispatch cross-platform CI from this task. Finish this task first, then run `CP1-ci-checkpoint` from the orchestration checkout.

## End Checklist
1. Capture the commands and results.
2. From inside the worktree, run `make triad-task-finish TASK_ID="BEDPM3-integ-core"`.
3. Hand off results plus the next-step instruction to run `CP1-ci-checkpoint`.
