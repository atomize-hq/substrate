# Kickoff: C7-code (Rollback CLI)

## Scope
- Implement rollback CLI using internal git per `C7-spec`. Production code only; no tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C7-spec.md, this prompt.
3. Set C7-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C7-code`).
4. Create branch `ws-c7-rollback-code`; worktree `wt/ws-c7-rollback-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Implement `substrate rollback` (last/checkpoint/session) per spec, including protected-path guards and world overlay refresh behavior.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check behavior manually if feasible.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C7-code status), add END entry to session_log.md (commands/results/blockers), ensure C7-test/C7-integ prompts exist.
5. Commit docs (`docs: finish C7-code`). Remove worktree if done.
