# Kickoff: WDAP0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-deps-apt-provisioning-wdap0-code` on branch `world-deps-apt-provisioning-wdap0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`, `docs/project_management/packs/draft/world-deps-apt-provisioning/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" SLICE_ID="WDAP0"` (preferred; starts code+test in parallel)
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-deps-apt-provisioning" TASK_ID="WDAP0-code"` (single task only)

## Requirements
- Implement exactly the behaviors and error handling in the spec.
- If the spec requires broad refactors or multiple independent behavior changes, stop and ask the operator to split the slice into smaller triads before proceeding.
- If completing this task requires more than 108,800 tokens of context, stop and ask the operator to split the slice before proceeding.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- Tests boundary:
  - Do not add new tests or new test files.
  - Only update existing tests when required to restore baseline expectations after the spec’s behavior change (no new test cases).
- Baseline testing:
  - Run a targeted baseline test set before making changes, then re-run the same baseline after your changes and verify results.

## End Checklist
1. Run required commands and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDAP0-code"`
3. Hand off baseline test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

