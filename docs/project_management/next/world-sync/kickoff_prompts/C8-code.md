# Kickoff: C8-code (World-side internal git bridge)

## Scope
- Ensure world-side `.substrate-git` bootstrap/bridge per `C8-spec`. Production code only; no tests.

## Start Checklist
1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C8-spec.md, this prompt.
3. Set C8-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C8-code`).
4. Create branch `ws-c8-worldgit-code`; worktree `wt/ws-c8-worldgit-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- World-side `.substrate-git` is created/cloned/mirrored from host (or initialized) lazily on first world session; protected from touching user `.git`.
- World ops needing internal git fail clearly if repo cannot be prepared; alignment with host commits/mapping per spec.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check behavior manually if feasible.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C8-code status), add END entry to session_log.md (commands/results/blockers), ensure C8-test/C8-integ prompts exist.
5. Commit docs (`docs: finish C8-code`). Remove worktree if done.
