# Kickoff: C2-integ-windows (integration CI parity: windows)

## Scope
- Windows CI parity task for slice C2.
- Windows smoke script is a no-op for this feature: `docs/project_management/_archived/world-first-repl-persistent-pty/smoke/windows-smoke.ps1`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c2-integ-windows` on branch `world-first-repl-persistent-pty-c2-integ-windows` and that `.taskmeta.json` exists.
2. Read: manual_testing_playbook.md and this prompt.

## CI audit + evidence ledger (recommended-first)
- Before dispatching CI, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/_archived/world-first-repl-persistent-pty/logs/C2/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C2-integ-windows"`
