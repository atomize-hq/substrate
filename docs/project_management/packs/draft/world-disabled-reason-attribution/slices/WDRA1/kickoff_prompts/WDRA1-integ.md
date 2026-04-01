# Kickoff: WDRA1-integ (integration)

## Scope
- Merge code and tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA1/WDRA1-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-disabled-reason-attribution-wdra1-integ` on branch `world-disabled-reason-attribution-wdra1-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`, `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`, `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" TASK_ID="WDRA1-integ"`

## Requirements
- Reconcile code and tests to spec.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant replay tests
  - `make integ-checks`
- Local smoke preflight, when the current platform is a behavior platform:
  - Linux: `bash "docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh"`
  - macOS: `bash "docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh"`
  - Windows: `pwsh -File "docs/project_management/packs/draft/world-disabled-reason-attribution\smoke\windows-smoke.ps1"`

## End Checklist
1. Ensure the merged state is committed and local gates are green.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDRA1-integ"`
3. Update tasks and session log on the orchestration branch.
4. Do not delete the worktree.
