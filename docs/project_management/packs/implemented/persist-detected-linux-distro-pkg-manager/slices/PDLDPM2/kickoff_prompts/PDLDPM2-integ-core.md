# Kickoff: PDLDPM2-integ-core (integration core)

## Scope
- Merge PDLDPM2 code and test branches and make the core slice green before CP1.
- Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`, and `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-integ-core` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-integ-core` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`, the PDLDPM2 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM2-integ-core"`.

## Requirements
- Merge `PDLDPM2-code` and `PDLDPM2-test`.
- Reconcile conflicts in favor of the slice spec.
- Run `cargo fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings`.
- Run relevant tests.
- Run `make integ-checks`.
- Do not dispatch checkpoint CI from this task. `CP1-ci-checkpoint` owns that step.

## End Checklist
1. Capture the integration commands and final results for the operator.
2. Run `make triad-task-finish TASK_ID="PDLDPM2-integ-core"` from inside the worktree.
3. Leave the worktree in place.
