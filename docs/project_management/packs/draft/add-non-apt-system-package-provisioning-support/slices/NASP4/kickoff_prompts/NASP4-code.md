# Kickoff: NASP4-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/add-non-apt-system-package-provisioning-support-nasp4-code` on branch `add-non-apt-system-package-provisioning-support-nasp4-code`.
2. Read `plan.md`, `tasks.json`, `session_log.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing, ask the operator to run `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support" SLICE_ID="NASP4"`.

## Requirements
- Implement exactly the NASP4 production behaviors.
- Run a targeted baseline command before edits, then rerun it after edits.
- Run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`.

## End Checklist
1. Run `make triad-task-finish TASK_ID="NASP4-code"` inside the worktree.
2. Hand off the baseline command and outcomes to the operator.
