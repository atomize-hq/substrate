# Kickoff: C3-integ (integration final)

## Scope
- Final aggregator for C3: merge platform-fix branches, run required checks, and merge back to orchestration.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c3-integ` on branch `world-first-repl-persistent-pty-c3-integ` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`, this prompt.

## Requirements
- Merge platform-fix branches and reconcile to the spec.
- Run all required checks listed in `docs/project_management/next/world-first-repl-persistent-pty/tasks.json` for `C3-integ`.

## CI audit + evidence ledger (recommended-first)
- Before dispatching smoke, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/next/world-first-repl-persistent-pty/logs/C3/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C3-integ"`
