# Kickoff: C0-integ-core (integration core)

## Scope
- Merge C0 code+tests; make the slice green; dispatch CI parity + behavioral smoke.
- Spec: `docs/project_management/_archived/world-first-repl-persistent-pty/C0-spec.md`
- Manual playbook: `docs/project_management/_archived/world-first-repl-persistent-pty/manual_testing_playbook.md`
- Execution workflow: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c0-integ-core` on branch `world-first-repl-persistent-pty-c0-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: plan/tasks/session_log, C0-spec.md, manual_testing_playbook.md, this prompt.

## Requirements
- Merge C0 code+test branches and reconcile to spec (spec is source of truth).
- Run required local checks:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - targeted tests listed in the task end checklist
  - `make integ-checks`
- Dispatch CI parity and behavioral smoke per the task end checklist; record run URLs in the orchestration session log (operator-owned).

## End Checklist
1. Ensure the end checklist in `tasks.json` is completed with commands + results recorded for handoff.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-core"`
3. Do not delete the worktree.

