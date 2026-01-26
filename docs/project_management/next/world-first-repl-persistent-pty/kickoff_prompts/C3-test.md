# Kickoff: C3-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`
- Requirements matrix: `docs/project_management/next/world-first-repl-persistent-pty/requirements_traceability.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c3-test` on branch `world-first-repl-persistent-pty-c3-test` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md`, `docs/project_management/next/world-first-repl-persistent-pty/requirements_traceability.md`, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/next/world-first-repl-persistent-pty" SLICE_ID="C3"`

## Requirements
- Encode the C3 acceptance criteria as tests (directive parsing, gating, routing, drift restart).
- Run: `cargo fmt`.
- Run the targeted tests you add/touch and capture output.

## End Checklist
1. Commit tests to the task branch.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C3-test"`

