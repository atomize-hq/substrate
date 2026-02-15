# Kickoff: OR1-test (test)

## Scope
- Tests only (plus minimal test-only helpers if needed); no production code.
- Spec: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/OR1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/agent-hub-concurrent-execution-output-routing-or1-test` on branch `agent-hub-concurrent-execution-output-routing-or1-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `OR1-spec.md`, and this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/agent-hub-concurrent-execution-output-routing" SLICE_ID="OR1"`

## Requirements
- Add/modify tests that enforce OR1 acceptance criteria:
  - no structured output printed during PTY passthrough
  - deferred structured output printed after passthrough ends
  - warning record emitted exactly once when drops occur
  - clamp warning record emitted when config is out-of-range
- Run: `cargo fmt`, plus the targeted tests you add/touch.
  - Suggested targets:
    - `cargo test -p shell --test repl_output_routing -- --nocapture`
    - `cargo test -p shell --test repl_config_max_pty_buffered_lines -- --nocapture`

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="OR1-test"`
3. Hand off the targeted test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).

