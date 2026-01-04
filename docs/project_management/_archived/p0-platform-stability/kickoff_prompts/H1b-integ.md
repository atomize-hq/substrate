# Task H1b-integ (Health manager parity – UX & docs) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Verify `H1b-code` and `H1b-test` are complete.
3. Set `H1b-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start H1b-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-h1b-healthux-integ
   git worktree add wt/ps-h1b-healthux-integ ps-h1b-healthux-integ
   cd wt/ps-h1b-healthux-integ
   ```

## Responsibilities
- Merge `ps-h1b-healthux-code` + `ps-h1b-healthux-test`, resolve conflicts, fast-forward onto `feat/p0-platform-stability` once checked.
- Re-run fmt/lint/tests and capture `substrate health --json` output reflecting the final UX.
- Update docs/tasks/session log and mark the P0 program complete (capture outstanding follow-ups if any).

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell health
substrate health --json
```

## End Checklist
1. Verify commands succeeded; document skips.
2. Merge integration branch back to `feat/p0-platform-stability`.
3. Update `tasks.json` (`H1b-integ` → completed) + session log END entry summarizing final outputs and next steps/follow-ups.
4. Ensure plan/docs mention completion status; capture any carry-over items in the log.
5. Commit doc/task/log updates (`git commit -am "docs: finish H1b-integ"`), remove worktree, and hand off artifacts for review.


Do not edit planning docs inside the worktree.
