# Kickoff: C3-integ-windows (integration platform-fix)

## Scope
- Validate CI parity for slice C3 on Windows.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c3-integ-windows` on branch `world-first-repl-persistent-pty-c3-integ-windows` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, this prompt.

## Requirements
- Dispatch CI parity gates as listed in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`.
- Record run URLs and outcomes in `docs/project_management/next/world-first-repl-persistent-pty/session_log.md` on the orchestration branch.

## CI audit + evidence ledger (recommended-first)
- Before dispatching CI, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/next/world-first-repl-persistent-pty/logs/C3/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C3-integ-windows"`
