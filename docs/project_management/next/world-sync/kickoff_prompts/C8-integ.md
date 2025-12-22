# Kickoff: C8-integ (World-side internal git bridge)

## Scope
- Merge C8-code and C8-test; ensure world-side git behavior matches `C8-spec`.

## Start Checklist
1. Confirm C8-code and C8-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C8-spec.md, this prompt.
4. Set C8-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C8-integ`).
5. Create branch `ws-c8-worldgit-integ`; worktree `wt/ws-c8-worldgit-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure host/world internal git alignment per spec.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`.

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make preflight`; capture outputs.
2. Commit integration worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C8-integ status), add END entry to session_log.md (commands/results/blockers).
5. Commit docs (`docs: finish C8-integ`). Remove worktree if done.
