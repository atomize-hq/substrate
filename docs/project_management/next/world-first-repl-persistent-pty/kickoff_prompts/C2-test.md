# Kickoff: C2-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C2-spec.md`
- Requirements matrix: `docs/project_management/next/world-first-repl-persistent-pty/requirements_traceability.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c2-test` on branch `world-first-repl-persistent-pty-c2-test` and that `.taskmeta.json` exists.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/C2-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`, `docs/project_management/next/world-first-repl-persistent-pty/requirements_traceability.md`, this prompt.

## Requirements
- Encode the C2 acceptance criteria as tests for fail-closed client protocol handling.
- Run: `cargo fmt`.
- Run the targeted tests you add/touch and capture output.

## End Checklist
1. Commit tests to the task branch.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C2-test"`
