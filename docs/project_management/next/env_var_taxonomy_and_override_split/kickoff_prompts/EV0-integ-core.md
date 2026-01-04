# Kickoff: EV0-integ-core (integration core)

## Scope
- Merge EV0 code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/next/env_var_taxonomy_and_override_split/EV0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/ev0-override-split-integ-core` on branch `ev-ev0-override-split-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `EV0-spec.md`, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/env_var_taxonomy_and_override_split" TASK_ID="EV0-integ-core" LAUNCH_CODEX=1`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Merge `EV0-code` and `EV0-test` task branches into this worktree.
- Run required integration gates (must be green before CI smoke dispatch):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Dispatch cross-platform smoke via CI from this worktree:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/env_var_taxonomy_and_override_split" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/env_var_taxonomy_and_override_split" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Capture smoke run ids/URLs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="EV0-integ-core"`
3. Hand off run ids/URLs and any parity notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

