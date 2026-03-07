# Kickoff: PDLDPM2-integ-core (integration core)

## Scope
- Merge code and tests, resolve drift to the spec, and make PDLDPM2 locally green before CP1.
- Spec: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`
- Workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in `wt/persist-detected-linux-distro-pkg-manager-pdldpm2-integ-core` on branch `persist-detected-linux-distro-pkg-manager-pdldpm2-integ-core` and `.taskmeta.json` exists.
2. Read `plan.md`, `tasks.json`, `session_log.md`, `platform-parity-spec.md`, `pre-planning/ci_checkpoint_plan.md`, the slice spec, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager" TASK_ID="PDLDPM2-integ-core"`.

## Requirements
- Merge `PDLDPM2-code` and `PDLDPM2-test`, then reconcile code and tests to the spec.
- Do not dispatch cross-platform CI from this task. CP1 owns compile parity and Linux behavior smoke.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

## End Checklist
1. Ensure the merged state is committed and local integration gates are green.
2. From inside the worktree, run `make triad-task-finish TASK_ID="PDLDPM2-integ-core"`.
3. Hand off the next-step checkpoint requirements to the operator.
