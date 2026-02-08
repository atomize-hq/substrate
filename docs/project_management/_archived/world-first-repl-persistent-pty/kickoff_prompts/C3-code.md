# Kickoff: C3-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/_archived/world-first-repl-persistent-pty/C3-spec.md`
- Authoritative behavior: `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md`
- Execution workflow: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c3-code` on branch `world-first-repl-persistent-pty-c3-code` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/_archived/world-first-repl-persistent-pty/plan.md`, `docs/project_management/_archived/world-first-repl-persistent-pty/tasks.json`, `docs/project_management/_archived/world-first-repl-persistent-pty/session_log.md`, `docs/project_management/_archived/world-first-repl-persistent-pty/C3-spec.md`, `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md`, this prompt.
3. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start-pair FEATURE_DIR="docs/project_management/_archived/world-first-repl-persistent-pty" SLICE_ID="C3"`

## Requirements
- Implement exactly C3-spec.md and all referenced requirements (routing, directives, lifecycle, drift restart).
- Do not introduce runtime fallbacks (DR-06).
- Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`.

## End Checklist
1. Run required commands; capture outputs.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C3-code"`
3. Hand off baseline test command(s) and outcomes to the operator (do not edit planning docs inside the worktree).
4. Do not delete the worktree (feature cleanup removes worktrees at feature end).

