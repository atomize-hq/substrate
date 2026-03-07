# Kickoff: PDLDPM1-integ (integration)

## Scope
- Merge PDLDPM1 code and test branches and make the slice green.
- Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, and `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/persist-detected-linux-distro-pkg-manager-pdldpm1-integ` on branch `persist-detected-linux-distro-pkg-manager-pdldpm1-integ` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`, the PDLDPM1 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM1-integ"`.

## Requirements
- Merge `PDLDPM1-code` and `PDLDPM1-test`.
- Reconcile conflicts in favor of the slice spec.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Run relevant tests.
- Run `make integ-checks`.
- Do not dispatch checkpoint CI from this task. `CP1-ci-checkpoint` owns that step.

## End Checklist
1. Capture the integration commands and final results for the operator.
2. Run `make triad-task-finish TASK_ID="PDLDPM1-integ"` from inside the worktree.
3. Leave the worktree in place.
