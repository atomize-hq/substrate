# Kickoff: C3-code (Auto-sync non-PTY)

## Scope
- Add auto-sync hook for non-PTY sessions per `C3-spec`. Production code only; no tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C3-spec.md, this prompt.
3. Set C3-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C3-code`).
4. Create branch `ws-c3-autosync-code`; worktree `wt/ws-c3-autosync-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Hook auto-sync on session close for non-PTY when enabled and direction includes from_world/both.
- Respect conflict policy, filters, size guard, dry-run setting; skip gracefully when overlay unavailable.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check behavior manually.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C3-code status), add END entry to session_log.md (commands/results/blockers), ensure C3-test/C3-integ prompts exist.
5. Commit docs (`docs: finish C3-code`). Remove worktree if done.
