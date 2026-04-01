# Kickoff: WDRA2-integ (integration final)

## Scope
- Finalize WDRA2 after `CP1-ci-checkpoint` and any platform-fix tasks.
- Spec: `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA2/WDRA2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-disabled-reason-attribution-wdra2-integ` on branch `world-disabled-reason-attribution-wdra2-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`, `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`, `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`, spec, this prompt.
3. Ensure `WDRA2-integ-core`, `CP1-ci-checkpoint`, and all required platform-fix tasks are complete.

## Requirements
- Merge the checkpoint-ready core integration branch and any platform-fix branches.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant replay tests
  - `make integ-checks`
- Re-run one local smoke wrapper on the current platform when possible.

## End Checklist
1. Ensure the final merged state is committed and green.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDRA2-integ"`
3. Hand off final integration evidence to the operator.
4. Do not delete the worktree.
