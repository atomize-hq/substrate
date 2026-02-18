# Kickoff: WS1-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/active/world-sync/WS1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-sync-ws1-code` on branch `world-sync-ws1-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/active/world-sync/plan.md`, `docs/project_management/packs/active/world-sync/tasks.json`, `docs/project_management/packs/active/world-sync/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/world-sync" SLICE_ID="WS1"` (preferred; starts code+test in parallel; `WS1` is `WS1-code` without `-code`)
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/world-sync" TASK_ID="WS1-code"` (single task only)

## Requirements
- Implement exactly the behaviors and error handling in the spec.
- If the spec requires broad refactors or multiple independent behavior changes, stop and ask the operator to split the slice into smaller triads before proceeding.
- If completing this task requires more than 108,800 tokens of context (40% of a 272k token window), stop and ask the operator to split the slice before proceeding.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- Tests boundary:
  - Do not add new tests or new test files.
  - Only update existing tests if required to restore baseline expectations after the spec’s behavior change (still no new test cases).
- Baseline testing (required):
  - Run a targeted baseline test set before making changes, then re-run the same test set after your changes and ensure results are unchanged (or improved).

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WS1-code"`
3. Hand off the baseline test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
