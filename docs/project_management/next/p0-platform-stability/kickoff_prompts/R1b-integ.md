# Task R1b-integ (Replay verbose scopes & warnings) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Verify `R1b-code` and `R1b-test` are done.
3. Set `R1b-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1b-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1b-verbosity-integ
   git worktree add wt/ps-r1b-verbosity-integ ps-r1b-verbosity-integ
   cd wt/ps-r1b-verbosity-integ
   ```

## Responsibilities
- Merge `ps-r1b-verbosity-code` + `ps-r1b-verbosity-test`, resolve conflicts, and fast-forward to `feat/p0-platform-stability` once validated.
- Re-run fmt/lint/tests and capture at least one `substrate --replay --replay-verbose` sample (or document skip).
- Update docs/tasks/session log, then prep R1c prompts.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
substrate --replay --replay-verbose --span <fixture>   # capture output/skip
```

## End Checklist
1. Confirm commands completed; document skips.
2. Merge integration branch into `feat/p0-platform-stability`.
3. Update `tasks.json` (`R1b-integ` → completed) + session log END entry.
4. Ensure R1c-code/test prompts reflect the new verbose behavior.
5. Commit doc/task/log updates (`git commit -am "docs: finish R1b-integ"`), remove worktree, hand off.
