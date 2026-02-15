# Kickoff: OR0-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/OR0-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/agent-hub-concurrent-execution-output-routing-or0-code` on branch `agent-hub-concurrent-execution-output-routing-or0-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `OR0-spec.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/agent-hub-concurrent-execution-output-routing" SLICE_ID="OR0"`

## Requirements
- Implement exactly OR0 behavior and invariants:
  - Envelope schema: `agent-hub-event-envelope-schema-spec.md`
  - Canonical trace record shapes: `telemetry-spec.md`
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- Baseline testing (required): pick a targeted baseline set, record before/after.
  - Suggested baseline:
    - `cargo test -p substrate-common --tests -- --nocapture`
    - `cargo test -p shell --tests -- --nocapture`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="OR0-code"`
3. Hand off baseline test commands and outcomes to the operator (do not edit planning docs inside the worktree).

