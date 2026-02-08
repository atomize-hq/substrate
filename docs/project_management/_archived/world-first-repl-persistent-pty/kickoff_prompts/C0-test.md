# Kickoff: C0-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/_archived/world-first-repl-persistent-pty/C0-spec.md`
- Requirements matrix: `docs/project_management/_archived/world-first-repl-persistent-pty/requirements_traceability.md`
- Execution workflow: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c0-test` on branch `world-first-repl-persistent-pty-c0-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan/tasks/session_log, C0-spec.md, requirements_traceability.md, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/world-first-repl-persistent-pty" SLICE_ID="C0"`

## Requirements
- Encode the MUST-level invariants as tests (fail-closed posture, no stdout marker parsing, DR-22, DR-23).
- Run: `cargo fmt`.
- Run the targeted tests you add/touch and capture output.

## End Checklist
1. Commit tests to the task branch.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-test"`
3. Hand off test commands/outcomes to integration (do not edit planning docs inside the worktree).

