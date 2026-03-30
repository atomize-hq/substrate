# Kickoff: PDLDPM2-integ (integration final)

## Scope
- Merge `PDLDPM2-integ-core` and all platform-fix branches and merge PDLDPM2 back to orchestration.
- Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`, and `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-integ` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-integ` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`, the PDLDPM2 spec, the CP1 results, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM2-integ"`.

## Requirements
- Merge `PDLDPM2-integ-core`, `PDLDPM2-integ-linux`, `PDLDPM2-integ-macos`, and `PDLDPM2-integ-windows`.
- Reconcile conflicts in favor of the slice spec.
- Verify `CP1-ci-checkpoint` is complete and recorded in `session_log.md`.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Run relevant tests.
- Run `make integ-checks`.

## End Checklist
1. Capture the integration commands and final results for the operator.
2. Run `make triad-task-finish TASK_ID="PDLDPM2-integ"` from inside the worktree.
3. Leave the worktree in place.
