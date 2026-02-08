# Kickoff: C1-integ-linux (integration Linux)

## Scope
- Platform-fix task for Linux smoke/CI parity results for slice C1.
- Smoke script: `docs/project_management/_archived/world-first-repl-persistent-pty/smoke/linux-smoke.sh`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c1-integ-linux` on branch `world-first-repl-persistent-pty-c1-integ-linux` and that `.taskmeta.json` exists.
2. Read: manual_testing_playbook.md and this prompt.

## CI audit + evidence ledger (recommended-first)
- Before dispatching CI/smoke, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/_archived/world-first-repl-persistent-pty/logs/C1/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C1-integ-linux"`
