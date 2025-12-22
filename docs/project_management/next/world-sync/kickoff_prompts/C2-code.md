# Kickoff: C2-code (Manual world→host sync, non-PTY)

## Scope
- Implement manual world→host sync apply for non-PTY sessions per `C2-spec`.
- Production code only; no tests. Manual command only; no auto-sync.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C2-spec.md, this prompt.
3. Set C2-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C2-code`).
4. Create branch `ws-c2-sync-code`; worktree `wt/ws-c2-sync-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Implement `substrate sync` world→host path for non-PTY, respecting conflict policy, filters, size guard, protected paths, and direction handling per spec.
- `from_host` path should error clearly; `both` runs world→host then reports host→world unimplemented.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Manually sanity-check command behavior.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C2-code status), add END entry to session_log.md (commands/results/blockers), ensure C2-test/C2-integ prompts exist.
5. Commit docs (`docs: finish C2-code`). Remove worktree if done.
