# Kickoff: WDRA2-integ-core (integration core)

## Scope
- Merge code and tests, resolve drift to spec, and make WDRA2 green on the primary dev platform before the checkpoint.
- Spec: `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA2/WDRA2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-disabled-reason-attribution-wdra2-integ-core` on branch `world-disabled-reason-attribution-wdra2-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`, `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`, `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" TASK_ID="WDRA2-integ-core"`

## Requirements
- Reconcile code and tests to spec.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant replay tests
  - `make integ-checks`
- Local smoke preflight on the current platform when possible:
  - Linux: `bash "docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh"`
  - macOS: `bash "docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh"`
  - Windows: `pwsh -File "docs/project_management/packs/draft/world-disabled-reason-attribution\smoke\windows-smoke.ps1"`
- Do not dispatch cross-platform CI from this task. Cross-platform gates run at `CP1-ci-checkpoint`.

## End Checklist
1. Ensure the merged state is committed and local gates are green.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDRA2-integ-core"`
3. Hand off the checkpoint-ready state to the operator.
4. Do not delete the worktree.
