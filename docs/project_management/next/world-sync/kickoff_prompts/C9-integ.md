# Kickoff: C9-integ (Init UX & migration)

## Scope
- Merge C9-code and C9-test; ensure init UX/migration matches `C9-spec`.

## Start Checklist
1. Confirm C9-code and C9-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C9-spec.md, this prompt.
4. Set C9-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C9-integ`).
5. Create branch `ws-c9-initux-integ`; worktree `wt/ws-c9-initux-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure UX/migration behavior matches spec and is safe.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`.

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make preflight`; capture outputs.
2. Commit integration worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C9-integ status), add END entry to session_log.md (commands/results/blockers).
5. Commit docs (`docs: finish C9-integ`). Remove worktree if done.
