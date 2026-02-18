# Kickoff: WS6-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/packs/active/world-sync/WS6-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-sync-ws6-integ` on branch `world-sync-ws6-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/active/world-sync/plan.md`, `docs/project_management/packs/active/world-sync/tasks.json`, `docs/project_management/packs/active/world-sync/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/world-sync" TASK_ID="WS6-integ"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- If completing this task requires more than 108,800 tokens of context (40% of a 272k token window), stop and ask the operator to split the slice before proceeding.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Complete the slice closeout gate report:
  - `docs/project_management/packs/active/world-sync/WS6-closeout_report.md` (e.g., `docs/project_management/packs/active/world-sync/WS6-closeout_report.md`)

### CI checkpoints (when applicable)

If this is a cross-platform automation pack (and `docs/project_management/packs/active/world-sync/ci_checkpoint_plan.md` exists):
- Do not dispatch cross-platform CI (compile parity / Feature Smoke / CI Testing) from this task.
- Cross-platform CI runs only at the planned checkpoint ops tasks (`CPk-ci-checkpoint`) defined by `ci_checkpoint_plan.md`.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WS6-integ"`
3. Hand off closeout report completion and any key outputs to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
