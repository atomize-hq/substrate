# Kickoff: C5-integ-core (integration core)

## Scope
- Merge C5 code+tests; make the slice green; dispatch CI parity + behavioral smoke.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C5-spec.md`
- Manual playbook: `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c5-integ-core` on branch `world-first-repl-persistent-pty-c5-integ-core` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/plan.md`, `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, `docs/project_management/next/world-first-repl-persistent-pty/session_log.md`, `docs/project_management/next/world-first-repl-persistent-pty/C5-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`, this prompt.

## Requirements
- Merge C5 code+test branches and reconcile to spec.
- Run required checks and dispatch smoke/parity as listed in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`.

## CI audit + evidence ledger (recommended-first)
- Before dispatching CI/smoke, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/next/world-first-repl-persistent-pty/logs/C5/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C5-integ-core"`
