# Kickoff: NASP0-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/add-non-apt-system-package-provisioning-support-nasp0-test` on branch `add-non-apt-system-package-provisioning-support-nasp0-test`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing, ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" SLICE_ID="NASP0"`.

## Requirements
- Add or update tests that enforce the NASP0 acceptance criteria.
- Run `cargo fmt` and the targeted tests you add or touch.

## End Checklist
1. Run `make triad-task-finish TASK_ID="NASP0-test"` inside the worktree.
2. Hand off the targeted test commands and outcomes to the operator.
