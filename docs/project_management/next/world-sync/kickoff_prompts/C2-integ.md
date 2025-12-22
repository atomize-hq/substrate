# Kickoff: C2-integ (Manual world→host sync, non-PTY)

## Scope
- Merge C2-code and C2-test, align to `C2-spec`.

## Start Checklist
1. Confirm C2-code and C2-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C2-spec.md, this prompt.
4. Set C2-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C2-integ`).
5. Create branch `ws-c2-sync-integ`; worktree `wt/ws-c2-sync-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve mismatches to match spec.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests (at least those added in C2-test), then `make preflight`.
- You own ensuring behavior matches spec and protected-path safety.

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make preflight`; capture outputs.
2. Commit integration worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C2-integ status), add END entry to session_log.md (commands/results/blockers).
5. Commit docs (`docs: finish C2-integ`). Remove worktree if done.
