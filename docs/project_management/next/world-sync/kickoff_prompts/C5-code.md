# Kickoff: C5-code (Host→world pre-sync)

## Scope
- Implement host→world pre-sync and direction handling per `C5-spec`. Production code only; no tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C5-spec.md, this prompt.
3. Set C5-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C5-code`).
4. Create branch `ws-c5-hostsync-code`; worktree `wt/ws-c5-hostsync-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Implement host→world pre-sync for `from_host` and `both`, with conflict policy, filters, size guard, and protected paths per spec.
- Clear logging and graceful no-op on unsupported platforms.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check behavior manually if feasible.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C5-code status), add END entry to session_log.md (commands/results/blockers), ensure C5-test/C5-integ prompts exist.
5. Commit docs (`docs: finish C5-code`). Remove worktree if done.
