# Kickoff: OR0-test (test)

## Scope

- Tests only (plus minimal test-only helpers if needed); no production code.
- Spec: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/agent-hub-concurrent-execution-output-routing-or0-test` on branch `agent-hub-concurrent-execution-output-routing-or0-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `OR0-spec.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing" SLICE_ID="OR0"`

## Requirements

- Add/modify tests that enforce OR0 acceptance criteria:
  - envelope required fields
  - unsafe channel dropped deterministically
  - canonical trace `agent_event` record is flattened and contains required join keys
- Run: `cargo fmt`, plus the targeted tests you add/touch.
  - Suggested targets:
    - `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`
    - `cargo test -p shell --test agent_hub_trace_persistence -- --nocapture`

## End Checklist

1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="OR0-test"`
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
