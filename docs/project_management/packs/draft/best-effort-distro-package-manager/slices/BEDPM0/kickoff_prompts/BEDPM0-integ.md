# Kickoff: BEDPM0-integ (integration final)

## Scope
- Merge `BEDPM0-integ-core` and all platform-fix branches and merge BEDPM0 back to orchestration.
- Read `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`, and `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/best-effort-distro-package-manager-bedpm0-integ` on branch `best-effort-distro-package-manager-bedpm0-integ` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`, `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`, the BEDPM0 spec, the CP1 results, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" TASK_ID="BEDPM0-integ"`.

## Requirements
- Merge `BEDPM0-integ-core`, `BEDPM0-integ-linux`, `BEDPM0-integ-macos`, and `BEDPM0-integ-windows`.
- Reconcile conflicts in favor of the slice spec.
- Verify `CP1-ci-checkpoint` is complete and recorded in `session_log.md`.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Run relevant tests.
- Run `make integ-checks`.

## End Checklist
1. Capture the integration commands and final results for the operator.
2. Run `make triad-task-finish TASK_ID="BEDPM0-integ"` from inside the worktree.
3. Leave the worktree in place.
