# Kickoff: BEDPM1-integ (integration)

## Scope
- Merge BEDPM1 code and test branches and make the slice green.
- Read `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, and `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/best-effort-distro-package-manager-bedpm1-integ` on branch `best-effort-distro-package-manager-bedpm1-integ` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`, `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`, the BEDPM1 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" TASK_ID="BEDPM1-integ"`.

## Requirements
- Merge `BEDPM1-code` and `BEDPM1-test`.
- Reconcile conflicts in favor of the slice spec.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Run relevant tests.
- Run `make integ-checks`.

## End Checklist
1. Capture the integration commands and final results for the operator.
2. Run `make triad-task-finish TASK_ID="BEDPM1-integ"` from inside the worktree.
3. Leave the worktree in place.
