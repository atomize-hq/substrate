# Kickoff: C3-integ-core (integration core)

## Scope
- Merge C3 code+tests; make the slice green; dispatch CI parity + behavioral smoke.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`
- Manual playbook: `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c3-integ-core` on branch `world-first-repl-persistent-pty-c3-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/plan.md`, `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, `docs/project_management/next/world-first-repl-persistent-pty/session_log.md`, `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`, this prompt.

## Requirements
- Merge C3 code+test branches and reconcile to spec.
- Run required checks and dispatch smoke/parity as listed in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C3-integ-core"`

