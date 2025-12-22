# Kickoff: C0-integ (Init & Gating)

## Scope
- Merge C0-code and C0-test; ensure init/gating matches `C0-spec`.

## Start Checklist
1. Confirm C0-code and C0-test are completed.
2. `git checkout feat/world-sync && git pull --ff-only`
3. Read: plan.md, tasks.json, session_log.md, C0-spec.md, this prompt.
4. Set C0-integ status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-integ`).
5. Create branch `ws-c0-init-integ`; worktree `wt/ws-c0-init-integ`.
6. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Merge code+tests; resolve drift; ensure init and gating behave per spec.
- Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`.

## End Checklist
1. Ensure fmt/clippy/tests pass; run `make preflight`; capture outputs.
2. Commit integration worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C0-integ status), add END entry to session_log.md (commands/results/blockers).
5. Commit docs (`docs: finish C0-integ`). Remove worktree if done.
