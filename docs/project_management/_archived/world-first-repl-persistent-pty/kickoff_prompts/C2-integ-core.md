# Kickoff: C2-integ-core (integration core)

## Scope
- Merge C2 code+tests; make the slice green; dispatch CI parity + behavioral smoke.
- Spec: `docs/project_management/_archived/world-first-repl-persistent-pty/C2-spec.md`
- Smoke scripts: `docs/project_management/_archived/world-first-repl-persistent-pty/smoke/*`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c2-integ-core` on branch `world-first-repl-persistent-pty-c2-integ-core` and that `.taskmeta.json` exists.
2. Read: plan/tasks/session_log, C2-spec.md, manual_testing_playbook.md, this prompt.

## CI audit + evidence ledger (recommended-first)
- Before dispatching CI/smoke, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `docs/project_management/_archived/world-first-repl-persistent-pty/tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/_archived/world-first-repl-persistent-pty/logs/C2/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C2-integ-core"`
