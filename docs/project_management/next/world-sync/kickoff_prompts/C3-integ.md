# Kickoff: C3-integ (Auto-sync non-PTY)

## Scope
- Merge C3-code and C3-test; align to `C3-spec`.

## Start Checklist
1. Confirm C3-code and C3-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C3-spec.md, this prompt.
4. Set C3-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C3-integ`).
5. Create branch `ws-c3-autosync-integ`; worktree `wt/ws-c3-autosync-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure behavior matches spec.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests (at least those added in C3-test), then `make preflight`.

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make preflight`; capture outputs.
2. Commit integration worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C3-integ status), add END entry to session_log.md (commands/results/blockers).
5. Commit docs (`docs: finish C3-integ`). Remove worktree if done.
