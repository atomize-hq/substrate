# Kickoff: C0-integ (integration final)

## Scope
- Final aggregator for slice C0.
- Spec: `docs/project_management/_archived/world-first-repl-persistent-pty/C0-spec.md`
- Manual playbook: `docs/project_management/_archived/world-first-repl-persistent-pty/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c0-integ` on branch `world-first-repl-persistent-pty-c0-integ` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session_log, manual_testing_playbook.md, this prompt.

## Requirements
- Merge platform-fix branches (linux/macos/windows) into this integration branch and reconcile to spec.
- Re-run `make integ-checks` and re-dispatch behavioral smoke (SMOKE_SLICE_ID=C0) as listed in `tasks.json`.
- This is the only C0 task with `merge_to_orchestration=true`.

## End Checklist
1. Ensure all required gates are green and recorded for handoff.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ"`

