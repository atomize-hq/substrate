# Kickoff: WFGAD0-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny/WFGAD0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-fs-granular-allow-deny-wfgad0-integ` on branch `world-fs-granular-allow-deny-wfgad0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/_archived/world-fs-granular-allow-deny/plan.md`, `docs/project_management/_archived/world-fs-granular-allow-deny/tasks.json`, `docs/project_management/_archived/world-fs-granular-allow-deny/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny" TASK_ID="WFGAD0-integ"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Record key evidence in `docs/project_management/_archived/world-fs-granular-allow-deny/session_log.md`.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WFGAD0-integ"`
3. Hand off key outputs to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
