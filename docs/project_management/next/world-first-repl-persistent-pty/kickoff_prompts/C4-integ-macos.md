# Kickoff: C4-integ-macos (integration platform-fix)

## Scope
- Validate and fix slice C4 on macOS behavior platform via feature smoke and CI parity.
- Spec: `docs/project_management/next/world-first-repl-persistent-pty/C4-spec.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in the task worktree `wt/world-first-repl-persistent-pty-c4-integ-macos` on branch `world-first-repl-persistent-pty-c4-integ-macos` and that `.taskmeta.json` exists at the worktree root.
2. Read: `docs/project_management/next/world-first-repl-persistent-pty/C4-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`, this prompt.

## Requirements
- If smoke fails, apply fixes on the macOS environment and re-run until green.
- Record smoke run URLs and outcomes in `docs/project_management/next/world-first-repl-persistent-pty/session_log.md` on the orchestration branch.

## CI audit + evidence ledger (recommended-first)
- Before dispatching CI/smoke, run `scripts/ci-audit/ci_audit.sh` and follow `RECOMMEND=...` (see `tasks.json` end checklist).
- Record each dispatch to the per-slice ledger (gitignored): `docs/project_management/next/world-first-repl-persistent-pty/logs/C4/ci-audit/ledger.jsonl`.

## End Checklist
1. From inside the worktree, run: `make triad-task-finish TASK_ID="C4-integ-macos"`
