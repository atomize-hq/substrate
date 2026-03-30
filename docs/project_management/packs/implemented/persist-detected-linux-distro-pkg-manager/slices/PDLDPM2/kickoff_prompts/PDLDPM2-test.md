# Kickoff: PDLDPM2-test (test)

## Scope
- Tests only.
- Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, and `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-test` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-test` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`, the PDLDPM2 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" SLICE_ID="PDLDPM2"`.

## Requirements
- Add or update tests that enforce the PDLDPM2 AC set.
- Keep test changes deterministic and scoped to the slice.
- Do not edit production code or operator docs.
- Run `cargo fmt`.
- Run the targeted tests you add or touch.

## End Checklist
1. Capture the targeted test commands and final results for the operator.
2. Run `make triad-task-finish TASK_ID="PDLDPM2-test"` from inside the worktree.
3. Leave the worktree in place.
