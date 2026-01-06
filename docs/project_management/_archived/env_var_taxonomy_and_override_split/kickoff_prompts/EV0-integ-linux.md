# Kickoff: EV0-integ-linux (integration platform-fix — linux)

## Scope
- Ensure the slice behaves correctly on linux.
- This task is allowed to make production-code and/or test changes as needed to achieve cross-platform parity, but must not edit planning docs inside the worktree.
- Spec: `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task must not merge back to the orchestration branch; the final aggregator integration task performs the merge once all platforms are green.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a machine that matches the required platform: linux.
2. Verify you are in the task worktree `wt/ev0-override-split-integ-linux` on branch `ev-ev0-override-split-integ-linux` and that `.taskmeta.json` exists at the worktree root.
3. Read: `plan.md`, `tasks.json`, `session_log.md`, `EV0-spec.md`, this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/env_var_taxonomy_and_override_split" TASK_ID="EV0-integ-linux" TASK_PLATFORM=linux LAUNCH_CODEX=1`

## Requirements
- Merge the core integration branch into this worktree branch:
  - `ev-ev0-override-split-integ-core`
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Ensure linux smoke validates policy.mode plus non-policy keys (minimum: `world.caged` and `world.anchor_mode`) and that failures are treated as parity bugs.
- Validate platform smoke via CI (repeat until green if you make fixes):
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/env_var_taxonomy_and_override_split" PLATFORM=linux RUNNER_KIND=self-hosted WORKFLOW_REF="feat/env_var_taxonomy_and_override_split" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

## End Checklist
1. Ensure smoke is green for linux and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="EV0-integ-linux"`
3. Hand off run id/URL and any linux notes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
