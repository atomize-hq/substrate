# Kickoff: EV0-test (test)

## Scope
- Tests only (plus minimal test-only helpers if required); no production code.
- Spec: `docs/project_management/next/env_var_taxonomy_and_override_split/EV0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/ev0-override-split-test` on branch `ev-ev0-override-split-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `EV0-spec.md`, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/env_var_taxonomy_and_override_split" SLICE_ID="EV0" LAUNCH_CODEX=1`

## Requirements
- Add/modify tests that enforce the acceptance criteria in `EV0-spec.md`.
- Ensure tests cover policy.mode plus multiple non-policy keys (minimum: `world.caged` and `world.anchor_mode`) across:
  - legacy exported-state `SUBSTRATE_*` does not override,
  - `SUBSTRATE_OVERRIDE_*` does override (when no workspace exists),
  - workspace config wins over overrides.
- Run:
  - `cargo fmt`
  - the targeted tests you add/touch

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="EV0-test"`
3. Hand off targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
