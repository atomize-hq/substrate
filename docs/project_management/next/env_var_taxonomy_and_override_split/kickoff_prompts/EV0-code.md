# Kickoff: EV0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/env_var_taxonomy_and_override_split/EV0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/ev0-override-split-code` on branch `ev-ev0-override-split-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `EV0-spec.md`, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/env_var_taxonomy_and_override_split" SLICE_ID="EV0" LAUNCH_CODEX=1`

## Requirements
- Implement exactly the behaviors and error handling in `EV0-spec.md`.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Tests boundary:
  - Do not add new tests or new test files.
  - Only update existing tests if required to restore baseline expectations after the spec’s behavior change (no new test cases).
- Baseline testing (required):
  - Run a targeted baseline test set before making changes, then re-run the same test set after changes and ensure results are unchanged (or improved).

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="EV0-code"`
3. Hand off baseline test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

