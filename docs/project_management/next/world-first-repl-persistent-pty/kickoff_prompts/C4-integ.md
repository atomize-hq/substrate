# Kickoff: C4-integ (integration final)

## Scope
- Final aggregator for C4: merge platform-fix branches, run required checks, and merge back to orchestration.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C4-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c4-integ` on branch `world-first-repl-persistent-pty-c4-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, `docs/project_management/next/world-first-repl-persistent-pty/C4-spec.md`, this prompt.

## Requirements
- Merge platform-fix branches and reconcile to the spec.
- Run all required checks listed in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json` for `C4-integ`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C4-integ"`

