# Kickoff: WS5-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable merged state.
- Spec: `docs/project_management/packs/active/world-sync/WS5-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task is responsible for merging back to the orchestration branch after all platforms are green (fast-forward when possible; otherwise a merge commit, preserving the orchestration branch’s Planning Pack files under the feature dir).

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-sync-ws5-integ` on branch `world-sync-ws5-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/active/world-sync/plan.md`, `docs/project_management/packs/active/world-sync/tasks.json`, `docs/project_management/packs/active/world-sync/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/world-sync" TASK_ID="WS5-integ"`

## Requirements
- Merge the relevant integration branches for this slice:
  - The core integration branch (e.g., `*-integ-core`) and any platform-fix integration branches (`*-integ-linux|macos`) that produced commits.
- Do not merge the orchestration branch into this worktree to “pick up task status/docs updates”; the finisher merges back while preserving the orchestration branch’s Planning Pack files.
- If the integration state has grown too large/unstable (many conflicts, large refactors, multiple unrelated changes), stop and ask the operator to split follow-up triads rather than forcing everything through a single final merge.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### CI checkpoints (required; cross-platform CI is not a per-slice step)

For cross-platform automation packs, cross-platform CI gates (compile parity + Feature Smoke) run only at the checkpoint boundaries defined in:
- `docs/project_management/packs/active/world-sync/ci_checkpoint_plan.md`

Rules:
- Do not dispatch cross-platform CI from this integration-final task.
- Verify that the checkpoint(s) that cover this slice are completed and that run ids/URLs are recorded in `docs/project_management/packs/active/world-sync/session_log.md`.
- Complete the slice closeout gate report:
  - `docs/project_management/packs/active/world-sync/WS5-closeout_report.md` (e.g., `docs/project_management/packs/active/world-sync/WS5-closeout_report.md`)

## End Checklist
1. Ensure your merged state is committed and local integration gates are green:
   - From inside the worktree, run: `make triad-task-finish TASK_ID="WS5-integ"`
2. Hand off closeout report completion and any remaining checkpoint requirements to the operator (do not edit planning docs inside the worktree).
3. Do not delete the worktree (feature cleanup removes worktrees at feature end).

Naming note:
- The task id for the final aggregator is `WS5-integ` (this prompt’s `WS5-integ`). The helper command to start it is named `triad-task-start-integ-final`.
