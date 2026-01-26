# Kickoff: C4-integ-windows (integration platform-fix)

## Scope
- Validate CI parity for slice C4 on Windows.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C4-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c4-integ-windows` on branch `world-first-repl-persistent-pty-c4-integ-windows` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/C4-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, this prompt.

## Requirements
- Dispatch CI parity gates as listed in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`.
- Record run URLs and outcomes in `docs/project_management/next/world-first-repl-persistent-pty/session_log.md` on the orchestration branch.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C4-integ-windows"`

