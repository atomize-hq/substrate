# Kickoff: C0-code (Init & Gating)

## Scope
- Implement `substrate init` and world gating per `C0-spec`. Production code only; no tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C0-spec.md, this prompt.
3. Set C0-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C0-code`).
4. Create branch `ws-c0-init-code`; worktree `wt/ws-c0-init-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- `substrate init` creates `.substrate/` and `.substrate-git/` (internal git scaffold), idempotent, safe, reports actions.
- Gate world mode (REPL and non-PTY/PTY world commands) unless init has been run; host-only mode still works.
- Write default config and ensure `.substrate-git` is ignored by user git.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check init/gating manually if feasible.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C0-code status), add END entry to session_log.md (commands/results/blockers), ensure C0-test/C0-integ prompts exist.
5. Commit docs (`docs: finish C0-code`). Remove worktree if done.
