# Kickoff: C5-integ-linux (integration platform-fix)

## Scope
- Validate and fix slice C5 on Linux behavior platform via feature smoke and CI parity.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C5-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c5-integ-linux` on branch `world-first-repl-persistent-pty-c5-integ-linux` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/C5-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, this prompt.

## Requirements
- If smoke fails, apply fixes on the Linux environment and re-run until green.
- Record smoke run URLs and outcomes in `docs/project_management/next/world-first-repl-persistent-pty/session_log.md` on the orchestration branch.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C5-integ-linux"`

