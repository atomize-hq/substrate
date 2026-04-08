# Kickoff: OR1-integ-core (integration core)

## Scope

- Merge code + tests, resolve drift to spec, and make the slice green on the primary dev platform.
- Spec: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/agent-hub-concurrent-execution-output-routing-or1-integ-core` on branch `agent-hub-concurrent-execution-output-routing-or1-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `OR1-spec.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing" TASK_ID="OR1-integ-core"`

## Requirements

- Reconcile code/tests to spec (spec wins).
- Merge OR1 branches:
  - `agent-hub-concurrent-execution-output-routing-or1-code`
  - `agent-hub-concurrent-execution-output-routing-or1-test`
- Run required integration gates (must be green before finishing this task):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`

### CI checkpoints (required; cross-platform CI is not a per-slice step)

For this feature, cross-platform gates run only at the checkpoint boundary:

- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/ci_checkpoint_plan.md`
- Next step after finishing this task: `CP1-ci-checkpoint`

## End Checklist

1. From inside the worktree, run: `make triad-task-finish TASK_ID="OR1-integ-core"`
2. Hand off next-step instructions to run `CP1-ci-checkpoint` from the orchestration checkout.
