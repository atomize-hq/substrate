# Kickoff: NASP4-integ-core (integration core)

## Scope
- Merge code and tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/add-non-apt-system-package-provisioning-support-nasp4-integ-core` on branch `add-non-apt-system-package-provisioning-support-nasp4-integ-core`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing, ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" TASK_ID="NASP4-integ-core"`.

## Requirements
- Merge `NASP4-code` and `NASP4-test`.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, and `make integ-checks`.
- If this machine matches a behavior platform, run the local smoke preflight before finishing.
- Do not dispatch cross-platform CI from this task; finish it first, then run `CP2-ci-checkpoint` from the orchestration checkout.

## End Checklist
1. Run `make triad-task-finish TASK_ID="NASP4-integ-core"` inside the worktree.
2. Hand off the local results and the next-step checkpoint instruction to the operator.
