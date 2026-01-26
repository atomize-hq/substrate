# Kickoff: C0-integ-windows (integration CI parity: windows)

## Scope
- Windows CI parity task for slice C0.
- Windows smoke script is a no-op for this feature: `docs/project_management/next/world-first-repl-persistent-pty/smoke/windows-smoke.ps1`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c0-integ-windows` on branch `world-first-repl-persistent-pty-c0-integ-windows` and that `.taskmeta.json` exists.
2. Read: manual_testing_playbook.md and this prompt.

## Requirements
- Own Windows CI parity results for slice C0 (compile/test parity).
- Apply Windows-only fixes on this branch only.

## End Checklist
1. Dispatch CI compile parity per `tasks.json` and record the run URL for handoff.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="C0-integ-windows"`

