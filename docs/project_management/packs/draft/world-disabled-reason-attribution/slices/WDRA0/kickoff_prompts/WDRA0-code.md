# Kickoff: WDRA0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-disabled-reason-attribution-wdra0-code` on branch `world-disabled-reason-attribution-wdra0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`, `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`, `docs/project_management/packs/draft/world-disabled-reason-attribution/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" SLICE_ID="WDRA0"`
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-reason-attribution" TASK_ID="WDRA0-code"`

## Requirements
- Implement exactly the behavior in the spec.
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- Tests boundary:
  - do not add new tests or new test files
  - only update existing tests when baseline expectations need alignment after the behavior change
- Baseline testing:
  - run a targeted baseline before edits, then re-run it after edits
  - suggested baseline: `cargo test -p shell --test replay_world replay_no_world_flag_reports_world_toggle -- --exact --nocapture`

## End Checklist
1. Run required commands and capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="WDRA0-code"`
3. Hand off baseline outcomes to the operator.
4. Do not delete the worktree.
