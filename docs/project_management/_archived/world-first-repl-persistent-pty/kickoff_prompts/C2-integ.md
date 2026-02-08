# Kickoff: C2-integ (integration final)

## Scope
- Final aggregator for slice C2.
- Spec: `docs/project_management/_archived/world-first-repl-persistent-pty/C2-spec.md`
- Manual playbook: `docs/project_management/_archived/world-first-repl-persistent-pty/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c2-integ` on branch `world-first-repl-persistent-pty-c2-integ` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session_log, manual_testing_playbook.md, this prompt.

## CI audit + evidence ledger (recommended-first)
- Before dispatching smoke, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/_archived/world-first-repl-persistent-pty/logs/C2/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C2-integ"`
