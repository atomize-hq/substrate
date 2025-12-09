# Kickoff: C4-integ (PTY overlay diff + sync)

## Scope
- Merge C4-code and C4-test; ensure PTY sync matches `C4-spec`.

## Start Checklist
1. Confirm C4-code and C4-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C4-spec.md, this prompt.
4. Set C4-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C4-integ`).
5. Create branch `ws-c4-pty-integ`; worktree `wt/ws-c4-pty-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure PTY and non-PTY parity per spec.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant PTY tests, then `make preflight`.

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make preflight`; capture outputs.
2. Commit integration worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C4-integ status), add END entry to session_log.md (commands/results/blockers).
5. Commit docs (`docs: finish C4-integ`). Remove worktree if done.
