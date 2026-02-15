# Kickoff: OR0-integ (integration)

## Scope
- Merge code + tests, resolve drift to spec, and make the slice green.
- Spec: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/OR0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/agent-hub-concurrent-execution-output-routing-or0-integ` on branch `agent-hub-concurrent-execution-output-routing-or0-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `OR0-spec.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/agent-hub-concurrent-execution-output-routing" TASK_ID="OR0-integ"`

## Requirements
- Reconcile code/tests to spec (spec wins).
- Merge OR0 branches:
  - `agent-hub-concurrent-execution-output-routing-or0-code`
  - `agent-hub-concurrent-execution-output-routing-or0-test`
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
- Cross-platform CI cadence:
  - Do not dispatch cross-platform gates from OR0.
  - Checkpoint CP1 (`CP1-ci-checkpoint`) runs after OR1-integ-core.
- Slice closeout report (execution gate; completed on orchestration branch):
  - `docs/project_management/next/agent-hub-concurrent-execution-output-routing/OR0-closeout_report.md`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="OR0-integ"`
3. Hand off closeout report completion requirements and key outputs to the operator (do not edit planning docs inside the worktree).

