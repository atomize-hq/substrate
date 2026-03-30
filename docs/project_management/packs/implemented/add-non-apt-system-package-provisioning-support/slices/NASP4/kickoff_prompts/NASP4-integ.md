# Kickoff: NASP4-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches and finalize NASP4 after the checkpoint gates are green.
- Spec: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/add-non-apt-system-package-provisioning-support-nasp4-integ` on branch `add-non-apt-system-package-provisioning-support-nasp4-integ`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing, ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" TASK_ID="NASP4-integ"`.

## Requirements
- Merge `NASP4-integ-core` and any platform-fix branches that produced commits.
- Do not merge the orchestration branch into this worktree.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- Verify `CP2-ci-checkpoint` is complete before finishing.

## End Checklist
1. Run `make triad-task-finish TASK_ID="NASP4-integ"` inside the worktree.
2. Hand off the merged state and checkpoint evidence to the operator.
