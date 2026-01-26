# Kickoff: C1-integ-core (integration core)

## Scope
- Merge C1 code+tests; make the slice green; dispatch CI parity + behavioral smoke.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C1-spec.md`
- Manual playbook: `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c1-integ-core` on branch `world-first-repl-persistent-pty-c1-integ-core` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session_log, C1-spec.md, manual_testing_playbook.md, this prompt.

## Requirements
- Merge C1 code+test branches and reconcile to spec.
- Run required checks and dispatch smoke/parity as listed in `tasks.json`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C1-integ-core"`

