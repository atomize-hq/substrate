# Kickoff: NASP0-integ (integration)

## Scope
- Merge code and tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/add-non-apt-system-package-provisioning-support-nasp0-integ` on branch `add-non-apt-system-package-provisioning-support-nasp0-integ`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing, ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" TASK_ID="NASP0-integ"`.

## Requirements
- Merge `NASP0-code` and `NASP0-test`.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Do not dispatch cross-platform CI from this task; checkpoints run only from `pre-planning/ci_checkpoint_plan.md`.

## End Checklist
1. Run `make triad-task-finish TASK_ID="NASP0-integ"` inside the worktree.
2. Hand off the commands you ran and the green state to the operator.
