# Kickoff: WDD2-integ (integration final — cross-platform merge)

## Scope
- Merge platform-fix branches (if any) and finalize the slice with a clean, auditable merged state.
- Spec: `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/WDD2-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- This task is responsible for merging back to the orchestration branch after all platforms are green (fast-forward when possible; otherwise a merge commit, preserving the orchestration branch’s Planning Pack files under the feature dir).

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-disabled-diagnostics-wdd2-integ` on branch `world-disabled-diagnostics-wdd2-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md`, `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`, `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md`, spec, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/draft/world-disabled-diagnostics" TASK_ID="WDD2-integ"`

## Requirements
- Merge the relevant integration branches for this slice:
  - the core integration branch (`WDD2-integ-core`), and
  - any platform-fix integration branches (`WDD2-integ-linux|macos|windows`) that produced commits.
- Do not merge the orchestration branch into this worktree to “pick up task status/docs updates”; the finisher merges back while preserving the orchestration branch’s Planning Pack files.
- If the integration state has grown too large/unstable (many conflicts, large refactors, multiple unrelated changes), stop and ask the operator to split follow-up triads rather than forcing everything through a single final merge.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### CI checkpoints (required; cross-platform CI is not a per-slice step)

For this cross-platform automation pack, cross-platform CI gates (compile parity + Feature Smoke) run only at the checkpoint boundary defined in:
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`

Rules:
- Do not dispatch cross-platform CI from this integration-final task.
- Verify `CP1-ci-checkpoint` is completed and that run ids/URLs are recorded in `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md`.

## End Checklist
1. Ensure your merged state is committed and local integration gates are green:
   - From inside the worktree, run: `make triad-task-finish TASK_ID="WDD2-integ"`
2. Hand off any remaining checkpoint requirements to the operator (do not edit planning docs inside the worktree).
3. Do not delete the worktree (feature cleanup removes worktrees at feature end).

Naming note:
- The task id for the final aggregator is `WDD2-integ`. The helper command to start it is named `triad-task-start-integ-final`.
