# Kickoff: BEDPM0-test (test)

## Scope
- Tests only.
- Read `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, and `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm the worktree is `wt/best-effort-distro-package-manager-bedpm0-test` on branch `best-effort-distro-package-manager-bedpm0-test` and `.taskmeta.json` exists at the worktree root.
2. Read `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`, `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`, the BEDPM0 spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager" SLICE_ID="BEDPM0"`.

## Requirements
- Add or update tests that enforce the BEDPM0 AC set.
- Keep test changes deterministic and scoped to the slice.
- Do not edit production code.
- Run `cargo fmt`.
- Run the targeted tests you add or touch.

## End Checklist
1. Capture the targeted test commands and final results for the operator.
2. Run `make triad-task-finish TASK_ID="BEDPM0-test"` from inside the worktree.
3. Leave the worktree in place.
