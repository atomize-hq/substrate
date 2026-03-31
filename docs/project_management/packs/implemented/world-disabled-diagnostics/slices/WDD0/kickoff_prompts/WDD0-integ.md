# Kickoff: WDD0-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-disabled-diagnostics-wdd0-integ` on branch `world-disabled-diagnostics-wdd0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md`, `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`, `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-diagnostics" TASK_ID="WDD0-integ"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- If completing this task requires more than 108,800 tokens of context (40% of a 272k token window), stop and ask the operator to split the slice before proceeding.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.

### CI checkpoints (cross-platform packs)

This is a cross-platform pack with bounded CI checkpoints:
- Cross-platform CI runs only at the planned checkpoint ops tasks (`CPk-ci-checkpoint`) per `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`.
- Do not dispatch compile parity / feature smoke / CI testing from this task.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDD0-integ"`
3. Hand off any key outputs to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
