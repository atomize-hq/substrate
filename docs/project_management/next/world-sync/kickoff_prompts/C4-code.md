# Kickoff: C4-code (PTY overlay diff + sync)

## Scope
- Expose PTY diffs and enable PTY world→host manual/auto sync per `C4-spec`. Production code only; no tests.

## Start Checklist

Do not edit planning docs inside the worktree.

1. `git checkout feat/world-sync && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, C4-spec.md, this prompt.
3. Set C4-code status to `in_progress` in tasks.json; add START entry to session_log.md; commit docs (`docs: start C4-code`).
4. Create branch `ws-c4-pty-code`; worktree `wt/ws-c4-pty-code`.
5. Do not edit docs/tasks/logs inside the worktree.

## Requirements
- Provide PTY fs_diff/overlay access; integrate manual/auto sync paths for PTY with conflict/filter/size guard semantics matching non-PTY.
- Skip gracefully when overlay/privileges unavailable.
- No tests added/modified.
- Not required to run unit/integration suites; do run `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Sanity-check behavior manually if feasible.

## End Checklist
1. Run fmt/clippy; capture outputs.
2. Commit worktree changes.
3. Merge back to feat/world-sync (ff-only).
4. Update tasks.json (C4-code status), add END entry to session_log.md (commands/results/blockers), ensure C4-test/C4-integ prompts exist.
5. Commit docs (`docs: finish C4-code`). Remove worktree if done.
