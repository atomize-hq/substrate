# Task H1a-integ (Health manager parity – aggregation logic) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Ensure `H1a-code` and `H1a-test` are complete.
3. Set `H1a-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start H1a-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-h1a-health-integ
   git worktree add wt/ps-h1a-health-integ ps-h1a-health-integ
   cd wt/ps-h1a-health-integ
   ```

## Responsibilities
- Merge `ps-h1a-health-code` + `ps-h1a-health-test`, resolve conflicts, and fast-forward to `feat/p0-platform-stability` after validation.
- Re-run fmt/lint/tests and record `substrate health --json` output (or log skip) to ensure combined behavior.
- Update docs/tasks/session log, then prep H1b prompts.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell health
substrate health --json   # capture/skip
```

## End Checklist
1. Confirm commands completed; document skips.
2. Merge integration branch into `feat/p0-platform-stability`.
3. Update `tasks.json` (`H1a-integ` → completed) + session log END entry summarizing results.
4. Ensure H1b-code/test prompts incorporate any new assumptions.
5. Commit doc/task/log updates (`git commit -am "docs: finish H1a-integ"`), remove worktree, hand off.
