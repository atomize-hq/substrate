# Kickoff: C1-integ (integration final)

## Scope
- Final aggregator for slice C1.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C1-spec.md`
- Manual playbook: `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c1-integ` on branch `world-first-repl-persistent-pty-c1-integ` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session_log, manual_testing_playbook.md, this prompt.

## Requirements
- Merge platform-fix branches (linux/macos/windows) into this integration branch and reconcile to spec.
- Re-run `make integ-checks` and re-dispatch behavioral smoke (SMOKE_SLICE_ID=C1) as listed in `tasks.json`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C1-integ"`

