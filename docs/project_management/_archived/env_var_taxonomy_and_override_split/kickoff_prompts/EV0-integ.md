# Kickoff: EV0-integ (integration final)

## Scope
- Merge core + platform-fix branches for EV0, run integration gates, confirm cross-platform smoke is green, and complete the closeout report.
- Spec: `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`
- Closeout report: `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-closeout_report.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/ev0-override-split-integ` on branch `ev-ev0-override-split-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `EV0-spec.md`, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/_archived/env_var_taxonomy_and_override_split" TASK_ID="EV0-integ" LAUNCH_CODEX=1`

## Requirements
- Merge the relevant EV0 branches into this worktree:
  - `ev-ev0-override-split-integ-core`
  - any platform-fix integration branches that produced commits
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Re-run cross-platform smoke via CI to confirm the merged result is green:
  - `make feature-smoke FEATURE_DIR="docs/project_management/_archived/env_var_taxonomy_and_override_split" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/env_var_taxonomy_and_override_split" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- Confirm smoke/manual parity and key coverage:
  - Smoke must validate policy.mode plus non-policy keys (minimum: `world.caged` and `world.anchor_mode`).
- Complete the closeout report:
  - `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-closeout_report.md`
  - Include the required repo-wide grep/audit evidence summary (hits + disposition).

## End Checklist
1. Ensure all required platforms are green (capture run ids/URLs).
2. From inside this worktree, run: `make triad-task-finish TASK_ID="EV0-integ"`
3. Hand off run ids/URLs and closeout completion to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).
